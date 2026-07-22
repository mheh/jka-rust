// PORT-COMPLETE: g_client.c
//! FAITHFUL port of `oracle/codemp/game/g_client.c` — client functions
//! that don't happen every frame.
//!
//! Filled by the jampgame mega-pass.
//!
//! Safe-state migration **Stage 1**. Entity params crossing this file's ABI
//! seam are `EntityId`/`Option<EntityId>` handles (§B5) instead of raw
//! `gentity_t*`, and `gclient_t*` params become their owning entity's
//! `EntityId`; ctx-free leaves borrow `&mut gentity_t`. The pilot is
//! `crate::g_object`. These bodies are saturated with still-raw seam derefs
//! (gclient/`ps` walks, ghoul2/vehicle chases, spawn-point loops), so per the
//! landed-shard "mega-fn" precedent they convert at the **signature only**: each
//! fn re-derives its raw pointer(s) at the top of the body
//! (`let ent: *mut gentity_t = ctx.entity_mut(id);`) and leaves the
//! referee-verified body verbatim. The remaining raw bodies are Stage-2 debt.
//! Behavior is byte-identical — a mechanical reshape, referee-verified.
//! Unconverted callers bridge their raw pointer at the boundary with
//! `ctx.entity_id_of(ptr)`.
#![allow(non_snake_case, unused, clippy::all)]

use crate::ai_main::BotAIShutdownClient;
use crate::g_utils::G_Find;
use crate::entity::flags::{FL_NO_BOTS, FL_NO_HUMANS};
use crate::g_bot::G_RemoveQueuedBotBegin;
use crate::g_cmds::{BroadcastTeamChange, G_SetSaber, SetTeam, StopFollowing};
use crate::g_combat::TossClientItems;
use crate::g_main::{CalculateRanks, G_GetStringEdString, G_LogPrintf};
use crate::g_saga::{G_ValidateSiegeClassForTeam, SetTeamQuick};
use crate::g_session::{G_InitSessionData, G_ReadSessionData, G_WriteClientSessionData};
use crate::g_team::{SelectCTFSpawnPoint, SelectSiegeSpawnPoint, TeamName};
use crate::g_utils::{G_EntitySound, G_MuteSound, G_PlayerHasCustomSkeleton};
use crate::prelude::*;
use crate::g_utils::G_ModelIndex;
use crate::q_shared::Info_SetValueForKey;
use crate::trap;
use crate::w_force::{WP_ForcePowerStop, WP_HasForcePowers, WP_InitForcePowers};
use crate::w_saber::HasSetSaberOnly;
use crate::world::GameContext;
use mp_bg::bg_misc::{
    BG_IsValidCharacterModel, BG_PlayerStateToEntityState, BG_ValidateSkinForTeam, WeaponReadyAnim,
};
use mp_bg::bg_saga::{BG_SiegeCheckClassLegality, BG_SiegeFindClassIndexByName};
use mp_bg::bg_vehicleLoad::BG_GetVehicleModelName;
use mp_bg::public::duel_team::duelTeam_t::{DUELTEAM_FREE, DUELTEAM_LONE, DUELTEAM_SINGLE};
use mp_qshared::common::mp::qcommon::saber::saber_styles::saber_styles_t::{
    SS_DUAL, SS_FAST, SS_STAFF, SS_STRONG,
};
use native_string::atoi::atoi;
use native_string::strncpyz_string;
use native_string::Q_stricmp;

use crate::client::client_persistant::MAX_NETNAME;

// `MAX_INFO_STRING` resolves via the crate prelude glob
// (`mp_qshared::shared::limits`); the shadowing local copy was removed by the
// placeholder-const sweep.

use crate::client::client_connected::CON_DISCONNECTED;
use crate::ent_fn_enums::{EntDie, EntThink, EntTouch, EntUse};
use crate::level::level_locals::BODY_QUEUE_SIZE;
use mp_abi::game::syscalls::G_ENTITIES_IN_BOX::GEntitiesInBoxArgs;
use mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs;
use mp_abi::game::syscalls::G_POINT_CONTENTS::GPointContentsArgs;
use mp_abi::game::syscalls::G_UNLINKENTITY::GUnlinkentityArgs;
use mp_bg::public::entity_event::entity_event_t;
use mp_bg::public::gametype::GT_POWERDUEL;
use mp_qshared::shared::{MAX_CLIENTS, MAX_GENTITIES};

// Ghoul2 `BONE_ANIM_*`/`BONE_ANGLES_POSTMULT` flags (used by
// `G_UpdateClientAnims`/`SetupGameGhoul2Model`) resolve via the canonical
// `mp_qshared::common::mp::ghoul2::bone_flags` module (crate prelude glob).

/// Raven `BODY_SINK_TIME` — how long a corpse persists before it is unlinked.
/// Source: `oracle/codemp/game/g_client.c:946`
pub const BODY_SINK_TIME: c_int = 30000;

/// Raven `JMSABER_RESPAWN_TIME` — fallback respawn delay for a stuck JM saber.
/// Source: `oracle/codemp/game/g_client.c:256`
pub const JMSABER_RESPAWN_TIME: c_int = 20000;

// `CS_CLIENT_JEDIMASTER` resolves via the crate prelude glob
// (`mp_bg::public::configstring`); `DEFAULT_MINS_2`/`DEFAULT_MAXS_2` are
// imported from their ported home below. Shadowing local copies removed by the
// placeholder-const sweep.
use mp_bg::public::viewheight::{DEFAULT_MAXS_2, DEFAULT_MINS_2};

/// Raven `CROUCH_MAXS_2` — the crouched player bbox max Z.
/// Source: `oracle/codemp/game/bg_public.h:44`
pub const CROUCH_MAXS_2: c_int = 16;

// `S_COLOR_WHITE` is `&CStr` (`g_team`) crate-wide; these `format!` sites need the
// `&str` spelling for `Display`. Shadows the prelude glob (glob is lower priority).
// Source: `oracle/codemp/game/q_shared.h` (`"^7"`)
const S_COLOR_WHITE: &str = "^7";

// Raven `qboolean` is `c_int`; keep the source spelling at assignment sites.
// Source: `oracle/codemp/game/q_shared.h`

/// Raven `playerMins` — the standard player bounding-box mins.
/// Source: `oracle/codemp/game/g_client.c:9`
pub static playerMins: vec3_t = [-15.0, -15.0, DEFAULT_MINS_2 as vec_t];
/// Raven `playerMaxs` — the standard player bounding-box maxs.
/// Source: `oracle/codemp/game/g_client.c:10`
pub static playerMaxs: vec3_t = [15.0, 15.0, DEFAULT_MAXS_2 as vec_t];

/// `FOFS(x)` — byte offset of field `x` within `gentity_t` (Raven macro,
/// `g_local.h`). Used as the `fieldofs` argument to `G_Find`.
#[inline]
fn fofs_classname() -> c_int {
    core::mem::offset_of!(gentity_t, classname) as c_int
}

/// Raven `SP_info_player_duel`.
///
/// Source: `oracle/codemp/game/g_client.c:27-39`
pub fn SP_info_player_duel(ctx: &mut GameContext, ent: EntityId) {
    let mut i: c_int = 0;
    crate::g_spawn::G_SpawnInt(
        ctx,
        b"nobots\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        &mut i,
    );
    if i != 0 {
        ctx.world.entity_mut(ent).flags |= FL_NO_BOTS;
    }
    crate::g_spawn::G_SpawnInt(
        ctx,
        b"nohumans\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        &mut i,
    );
    if i != 0 {
        ctx.world.entity_mut(ent).flags |= FL_NO_HUMANS;
    }
}

/// Raven `SP_info_player_duel1`.
///
/// Source: `oracle/codemp/game/g_client.c:47-59`
pub fn SP_info_player_duel1(ctx: &mut GameContext, ent: EntityId) {
    let mut i: c_int = 0;
    crate::g_spawn::G_SpawnInt(
        ctx,
        b"nobots\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        &mut i,
    );
    if i != 0 {
        ctx.world.entity_mut(ent).flags |= FL_NO_BOTS;
    }
    crate::g_spawn::G_SpawnInt(
        ctx,
        b"nohumans\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        &mut i,
    );
    if i != 0 {
        ctx.world.entity_mut(ent).flags |= FL_NO_HUMANS;
    }
}

/// Raven `SP_info_player_duel2`.
///
/// Source: `oracle/codemp/game/g_client.c:67-79`
pub fn SP_info_player_duel2(ctx: &mut GameContext, ent: EntityId) {
    let mut i: c_int = 0;
    crate::g_spawn::G_SpawnInt(
        ctx,
        b"nobots\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        &mut i,
    );
    if i != 0 {
        ctx.world.entity_mut(ent).flags |= FL_NO_BOTS;
    }
    crate::g_spawn::G_SpawnInt(
        ctx,
        b"nohumans\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        &mut i,
    );
    if i != 0 {
        ctx.world.entity_mut(ent).flags |= FL_NO_HUMANS;
    }
}

/// Raven `SP_info_player_deathmatch`.
///
/// Source: `oracle/codemp/game/g_client.c:88-99`
pub fn SP_info_player_deathmatch(ctx: &mut GameContext, ent: EntityId) {
    let mut i: c_int = 0;
    crate::g_spawn::G_SpawnInt(
        ctx,
        b"nobots\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        &mut i,
    );
    if i != 0 {
        ctx.world.entity_mut(ent).flags |= FL_NO_BOTS;
    }
    crate::g_spawn::G_SpawnInt(
        ctx,
        b"nohumans\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        &mut i,
    );
    if i != 0 {
        ctx.world.entity_mut(ent).flags |= FL_NO_HUMANS;
    }
}

/// Raven `SP_info_player_start` — equivalent to `info_player_deathmatch`.
///
/// Source: `oracle/codemp/game/g_client.c:105-108`
pub fn SP_info_player_start(ctx: &mut GameContext, ent: EntityId) {
    ctx.world.entity_mut(ent).classname = b"info_player_deathmatch\0".as_ptr() as *mut c_char;
    SP_info_player_deathmatch(ctx, ent);
}

/// Raven `SP_info_player_start_red` — Red Team DM start.
///
/// Source: `oracle/codemp/game/g_client.c:121-123`
pub fn SP_info_player_start_red(ctx: &mut GameContext, ent: EntityId) {
    SP_info_player_deathmatch(ctx, ent);
}

/// Raven `SP_info_player_start_blue` — Blue Team DM start.
///
/// Source: `oracle/codemp/game/g_client.c:136-138`
pub fn SP_info_player_start_blue(ctx: &mut GameContext, ent: EntityId) {
    SP_info_player_deathmatch(ctx, ent);
}

/// Raven `SiegePointUse` — toggle the siege spawn point on/off.
///
/// Source: `oracle/codemp/game/g_client.c:140-151`
pub fn SiegePointUse(self_: &mut gentity_t, other: Option<EntityId>, activator: Option<EntityId>) {
    // Toggle the point on/off
    if self_.genericValue1 != 0 {
        self_.genericValue1 = 0;
    } else {
        self_.genericValue1 = 1;
    }
}

/// Raven `SP_info_player_siegeteam1` — siege start point, team1.
///
/// Source: `oracle/codemp/game/g_client.c:164-187`
pub fn SP_info_player_siegeteam1(ctx: &mut GameContext, ent: EntityId) {
    let mut soff: c_int = 0;
    if ctx.world.cvars.g_gametype.integer != GT_SIEGE {
        // turn into a DM spawn if not in siege game mode
        ctx.world.entity_mut(ent).classname = b"info_player_deathmatch\0".as_ptr() as *mut c_char;
        SP_info_player_deathmatch(ctx, ent);
        return;
    }

    G_SpawnInt(
        ctx,
        c"startoff".as_ptr(),
        c"0".as_ptr(),
        &mut soff as *mut c_int,
    );

    if soff != 0 {
        // start disabled
        ctx.world.entity_mut(ent).genericValue1 = 0;
    } else {
        ctx.world.entity_mut(ent).genericValue1 = 1;
    }

    ctx.world.entity_mut(ent).use_ = Some(EntUse::SiegePointUse).into();
}

/// Raven `SP_info_player_siegeteam2` — siege start point, team2.
///
/// Source: `oracle/codemp/game/g_client.c:200-223`
pub fn SP_info_player_siegeteam2(ctx: &mut GameContext, ent: EntityId) {
    let mut soff: c_int = 0;
    if ctx.world.cvars.g_gametype.integer != GT_SIEGE {
        // turn into a DM spawn if not in siege game mode
        ctx.world.entity_mut(ent).classname = b"info_player_deathmatch\0".as_ptr() as *mut c_char;
        SP_info_player_deathmatch(ctx, ent);
        return;
    }

    G_SpawnInt(
        ctx,
        c"startoff".as_ptr(),
        c"0".as_ptr(),
        &mut soff as *mut c_int,
    );

    if soff != 0 {
        // start disabled
        ctx.world.entity_mut(ent).genericValue1 = 0;
    } else {
        ctx.world.entity_mut(ent).genericValue1 = 1;
    }

    ctx.world.entity_mut(ent).use_ = Some(EntUse::SiegePointUse).into();
}

/// Raven `SP_info_player_intermission` — the intermission view point (no-op
/// spawn; the point is only read by the intermission code).
///
/// Source: `oracle/codemp/game/g_client.c:230-232`
pub fn SP_info_player_intermission(ent: &gentity_t) {}

/// Raven `SP_info_player_intermission_red`.
///
/// Source: `oracle/codemp/game/g_client.c:241-243`
pub fn SP_info_player_intermission_red(ent: &gentity_t) {}

/// Raven `SP_info_player_intermission_blue`.
///
/// Source: `oracle/codemp/game/g_client.c:252-254`
pub fn SP_info_player_intermission_blue(ent: &gentity_t) {}

/// Raven `ThrowSaberToAttacker` — drop the Jedi-Master saber toward the killer.
///
/// Source: `oracle/codemp/game/g_client.c:258-344`
//
// The `#ifdef _DEBUG` `Com_Printf` diagnostics are release-build no-ops and are
// omitted (§20 dead-surface). `gJMSaberEnt` is the resolved `Option<*mut
// gentity_t>` global (see game_globals.rs).
pub fn ThrowSaberToAttacker(ctx: &mut GameContext, self_: EntityId, attacker: Option<EntityId>) {
    // STAGE-1: EntityId self_ + Option<EntityId> attacker (body null-checks
    // attacker); raw re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    let attacker: *mut gentity_t =
        unsafe { crate::ent_id::resolve(ctx.world.g_entities.as_mut_ptr(), attacker) };
    unsafe {
        let base = ctx.world.g_entities.as_ptr();
        let client = (*self_).client;
        let mut ent = &mut ctx.world.g_entities[(*client).ps.saberIndex as usize] as *mut gentity_t;
        let mut altVelocity: c_int = 0;

        if ent.is_null() || (*ent).enemy != ent_id_opt(base, self_) {
            // something has gone very wrong (this should never happen)
            // but in case it does.. find the saber manually
            ent = ctx
                .world
                .globals
                .gJMSaberEnt
                .unwrap_or(core::ptr::null_mut());

            if ent.is_null() {
                return;
            }

            (*ent).enemy = ent_id_opt(base, self_);
            (*client).ps.saberIndex = (*ent).s.number;
        }

        trap::SetConfigstring(ctx.engine, CS_CLIENT_JEDIMASTER, "-1");

        if !attacker.is_null()
            && !(*attacker).client.is_null()
            && (*client).ps.saberInFlight != qfalse
        {
            // someone killed us and we had the saber thrown, so actually move this
            // saber to the saber location
            let flyingsaber =
                &mut ctx.world.g_entities[(*client).ps.saberEntityNum as usize] as *mut gentity_t;

            if !flyingsaber.is_null() && (*flyingsaber).inuse != qfalse {
                (*ent).s.pos.trBase = (*flyingsaber).s.pos.trBase;
                (*ent).s.pos.trDelta = (*flyingsaber).s.pos.trDelta;
                (*ent).s.apos.trBase = (*flyingsaber).s.apos.trBase;
                (*ent).s.apos.trDelta = (*flyingsaber).s.apos.trDelta;

                (*ent).r.currentOrigin = (*flyingsaber).r.currentOrigin;
                (*ent).r.currentAngles = (*flyingsaber).r.currentAngles;
                altVelocity = 1;
            }
        }

        // say he threw it anyway in order to properly remove from dead body
        (*client).ps.saberInFlight = qtrue;

        crate::w_saber::WP_SaberAddG2Model(
            ctx,
            ctx.entity_id_of(ent).unwrap(),
            (*client).saber[0].model.as_ptr(),
            (*client).saber[0].skin,
        );

        (*ent).s.eFlags &= !EF_NODRAW;
        (*ent).s.modelGhoul2 = 1;
        (*ent).s.eType = ET_MISSILE as c_int;
        (*ent).enemy = None;

        if attacker.is_null() || (*attacker).client.is_null() {
            (*ent).s.pos.trBase = (*ent).s.origin2;
            (*ent).s.origin = (*ent).s.origin2;
            (*ent).r.currentOrigin = (*ent).s.origin2;
            (*ent).pos2[0] = 0.0;
            trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent.cast()));
            return;
        }

        if altVelocity == 0 {
            (*ent).s.pos.trBase = (*self_).s.pos.trBase;
            (*ent).s.origin = (*self_).s.pos.trBase;
            (*ent).r.currentOrigin = (*self_).s.pos.trBase;

            let mut a: vec3_t = [0.0; 3];
            for k in 0..3 {
                a[k] = (*((*attacker).client)).ps.origin[k] - (*ent).s.pos.trBase[k];
            }

            crate::q_math::VectorNormalize(&mut a);

            (*ent).s.pos.trDelta[0] = a[0] * 256.0;
            (*ent).s.pos.trDelta[1] = a[1] * 256.0;
            (*ent).s.pos.trDelta[2] = 256.0;
        }

        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent.cast()));
    }
}

/// Raven `JMSaberThink`.
///
/// Source: `oracle/codemp/game/g_client.c:346-383`
//
// `JMSaberThink` is stored elsewhere as an `EntThink` dispatch target
// (`ent_fn_enums::EntThink::JMSaberThink`); this is its body. `pos2` is a
// `vec3_t`, so the `pos2[0/1]` flag/timer reads/writes are `f32` exactly as C's
// float-slot arithmetic.
pub fn JMSaberThink(ctx: &mut GameContext, ent: EntityId) {
    // `gJMSaberEnt` is the resolved `Option<*mut gentity_t>` global; store the raw
    // pointer for this frame's saber entity.
    let ent_ptr: *mut gentity_t = ctx.entity_mut(ent);
    ctx.world.globals.gJMSaberEnt = Some(ent_ptr);

    let enemy = ctx.world.entity(ent).enemy;
    if let Some(enemy_id) = enemy {
        // `enemy` is read-only here; its `.client` is a raw pointer (pool or level
        // slot) — read the value and null-check it exactly as Raven does (recipe 2b).
        let enemy_client = ctx.world.entity(enemy_id).client;
        let enemy_inuse = ctx.world.entity(enemy_id).inuse;
        if enemy_client.is_null() || enemy_inuse == qfalse {
            // disconnected?
            let enemy_trbase = ctx.world.entity(enemy_id).s.pos.trBase;
            let e = ctx.world.entity_mut(ent);
            e.s.pos.trBase = enemy_trbase;
            e.s.origin = enemy_trbase;
            e.r.currentOrigin = enemy_trbase;
            e.s.modelindex = G_ModelIndex("models/weapons2/saber/saber_w.glm",
            );
            e.s.eFlags &= !EF_NODRAW;
            e.s.modelGhoul2 = 1;
            e.s.eType = ET_MISSILE as c_int;
            e.enemy = None;

            e.pos2[0] = 1.0;
            e.pos2[1] = 0.0; // respawn next think
            let ent_ptr: *mut gentity_t = ctx.entity_mut(ent);
            trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent_ptr.cast()));
        } else {
            let respawn_at = (ctx.world.level.time + JMSABER_RESPAWN_TIME) as f32;
            ctx.world.entity_mut(ent).pos2[1] = respawn_at;
        }
    } else if ctx.world.entity(ent).pos2[0] != 0.0
        && ctx.world.entity(ent).pos2[1] < ctx.world.level.time as f32
    {
        let origin2 = ctx.world.entity(ent).s.origin2;
        let e = ctx.world.entity_mut(ent);
        e.s.pos.trBase = origin2;
        e.s.origin = origin2;
        e.r.currentOrigin = origin2;
        e.pos2[0] = 0.0;
        let ent_ptr: *mut gentity_t = ctx.entity_mut(ent);
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent_ptr.cast()));
    }

    let nextthink = ctx.world.level.time + 50;
    ctx.world.entity_mut(ent).nextthink = nextthink;
    crate::g_object::G_RunObject(ctx, ent);
}

/// Raven `JMSaberTouch` — pick up the JM saber, become the Jedi Master.
///
/// Source: `oracle/codemp/game/g_client.c:385-469`
pub fn JMSaberTouch(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    trace: *mut trace_t,
) {
    let Some(other_id) = other else {
        return;
    };
    // `other.client` is a raw `gclient_t` pointer (the toucher is a real player
    // here, but read the value and deref it raw exactly as Raven does — recipe 2b).
    let other_client = ctx.world.entity(other_id).client;
    if other_client.is_null() || ctx.world.entity(other_id).health < 1 {
        return;
    }

    if ctx.world.entity(self_).enemy.is_some() {
        return;
    }

    if ctx.world.entity(self_).s.modelindex == 0 {
        return;
    }

    if unsafe { (*other_client).ps.stats[STAT_WEAPONS as usize] } & (1 << WP_SABER) != 0 {
        return;
    }

    if unsafe { (*other_client).ps.isJediMaster } != 0 {
        return;
    }

    ctx.world.entity_mut(self_).enemy = Some(other_id);
    unsafe {
        (*other_client).ps.stats[STAT_WEAPONS as usize] = 1 << WP_SABER;
        (*other_client).ps.weapon = WP_SABER;
    }
    ctx.world.entity_mut(other_id).s.weapon = WP_SABER;
    G_AddEvent(
        ctx.world.entity_mut(other_id),
        (EV_BECOME_JEDIMASTER) as i32,
        0,
    );

    // Track the jedi master
    let cs = format!("{}", ctx.world.entity(other_id).s.number);
    trap::SetConfigstring(ctx.engine, CS_CLIENT_JEDIMASTER, &cs);

    if ctx.world.cvars.g_spawnInvulnerability.integer != 0 {
        let invulnerable_at = ctx.world.level.time + ctx.world.cvars.g_spawnInvulnerability.integer;
        unsafe {
            (*other_client).ps.eFlags |= EF_INVULNERABLE;
            (*other_client).invulnerableTimer = invulnerable_at;
        }
    }

    let netname = unsafe { (*other_client).pers.netname.clone() };
    let becomejm = G_GetStringEdString(ctx, "MP_SVGAME", "BECOMEJM");
    let msg = format!("cp \"{} {}\n\"", netname, becomejm);
    trap::SendServerCommand(ctx.engine, -1, &msg);

    let self_number = ctx.world.entity(self_).s.number;
    unsafe {
        (*other_client).ps.isJediMaster = qtrue;
        (*other_client).ps.saberIndex = self_number;
    }

    if ctx.world.entity(other_id).health < 200 && ctx.world.entity(other_id).health > 0 {
        // full health when you become the Jedi Master
        unsafe {
            (*other_client).ps.stats[STAT_HEALTH as usize] = 200;
        }
        ctx.world.entity_mut(other_id).health = 200;
    }

    if unsafe { (*other_client).ps.fd.forcePower } < 100 {
        unsafe {
            (*other_client).ps.fd.forcePower = 100;
        }
    }

    let mut i = 0;
    while i < NUM_FORCE_POWERS {
        unsafe {
            (*other_client).ps.fd.forcePowersKnown |= 1 << i;
            (*other_client).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_3;
        }
        i += 1;
    }

    let respawn_at = (ctx.world.level.time + JMSABER_RESPAWN_TIME) as f32;
    {
        let s = ctx.world.entity_mut(self_);
        s.pos2[0] = 1.0;
        s.pos2[1] = respawn_at;

        s.s.modelindex = 0;
        s.s.eFlags |= EF_NODRAW;
        s.s.modelGhoul2 = 0;
        s.s.eType = ET_GENERAL as c_int;
    }

    // The `te = G_TempEntity(...)` broadcast block is commented out in the
    // oracle (g_client.c:461-465); dropped per §20.
    let self_number = ctx.world.entity(self_).s.number;
    crate::g_utils::G_KillG2Queue(ctx, self_number);
}

/// Raven `SP_info_jedimaster_start` — the JM saber spawn point.
///
/// Source: `oracle/codemp/game/g_client.c:476-516`
pub fn SP_info_jedimaster_start(ctx: &mut GameContext, ent: EntityId) {
    if ctx.world.cvars.g_gametype.integer != GT_JEDIMASTER {
        ctx.world.globals.gJMSaberEnt = None;
        crate::g_utils::G_FreeEntity(ctx, Some(ent));
        return;
    }

    ctx.world.entity_mut(ent).enemy = None;

    ctx.world.entity_mut(ent).flags = FL_BOUNCE_HALF;

    ctx.world.entity_mut(ent).s.modelindex =
        G_ModelIndex("models/weapons2/saber/saber_w.glm");
    ctx.world.entity_mut(ent).s.modelGhoul2 = 1;
    ctx.world.entity_mut(ent).s.g2radius = 20;
    // (*ent).s.eType = ET_GENERAL;
    ctx.world.entity_mut(ent).s.eType = ET_MISSILE as c_int;
    ctx.world.entity_mut(ent).s.weapon = WP_SABER;
    ctx.world.entity_mut(ent).s.pos.trType = TR_GRAVITY;
    let level_time = ctx.world.level.time;
    ctx.world.entity_mut(ent).s.pos.trTime = level_time;
    crate::q_math::VectorSet(&mut ctx.world.entity_mut(ent).r.maxs, 3.0, 3.0, 3.0);
    crate::q_math::VectorSet(&mut ctx.world.entity_mut(ent).r.mins, -3.0, -3.0, -3.0);
    ctx.world.entity_mut(ent).r.contents = CONTENTS_TRIGGER;
    ctx.world.entity_mut(ent).clipmask = MASK_SOLID;

    ctx.world.entity_mut(ent).isSaberEntity = qtrue;

    ctx.world.entity_mut(ent).bounceCount = -5;

    ctx.world.entity_mut(ent).physicsObject = qtrue;

    // remember the spawn spot
    let trbase = ctx.world.entity(ent).s.pos.trBase;
    crate::q_math::_VectorCopy(trbase, &mut ctx.world.entity_mut(ent).s.origin2);

    ctx.world.entity_mut(ent).touch = Some(EntTouch::JMSaberTouch).into();

    let ent_ptr: *mut gentity_t = ctx.entity_mut(ent);
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent_ptr.cast()));

    ctx.world.entity_mut(ent).think = Some(EntThink::JMSaberThink).into();
    let level_time = ctx.world.level.time;
    ctx.world.entity_mut(ent).nextthink = level_time + 50;
}

/// Raven `SpotWouldTelefrag` — would a client spawn on `spot` overlap another
/// client?
///
/// Source: `oracle/codemp/game/g_client.c:532-552`
pub fn SpotWouldTelefrag(ctx: &mut GameContext, spot: EntityId) -> qboolean {
    let mut touch: [c_int; MAX_GENTITIES] = [0; MAX_GENTITIES];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];

    let origin = ctx.world.entity(spot).s.origin;
    for k in 0..3 {
        mins[k] = origin[k] + playerMins[k];
        maxs[k] = origin[k] + playerMaxs[k];
    }
    let num = trap::EntitiesInBox(
        ctx.engine,
        GEntitiesInBoxArgs::new(
            &mins as *const vec3_t,
            &maxs as *const vec3_t,
            touch.as_mut_ptr(),
            MAX_GENTITIES as c_int,
        ),
    );

    let mut i = 0;
    while i < num {
        // if ( hit->client && hit->client->ps.stats[STAT_HEALTH] > 0 ) — the
        // health test is commented out in Raven; any client presence telefrags.
        // Reading the `client` pointer value (null test) needs no deref.
        if !ctx.world.g_entities[touch[i as usize] as usize]
            .client
            .is_null()
        {
            return qtrue;
        }
        i += 1;
    }

    qfalse
}

/// Raven `SpotWouldTelefrag2` — would `mover` moved to `dest` overlap a solid?
///
/// Source: `oracle/codemp/game/g_client.c:554-580`
pub fn SpotWouldTelefrag2(ctx: &mut GameContext, mover: EntityId, dest: vec3_t) -> qboolean {
    let mut touch: [c_int; MAX_GENTITIES] = [0; MAX_GENTITIES];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];

    let r_mins = ctx.world.entity(mover).r.mins;
    let r_maxs = ctx.world.entity(mover).r.maxs;
    for k in 0..3 {
        mins[k] = dest[k] + r_mins[k];
        maxs[k] = dest[k] + r_maxs[k];
    }
    let num = trap::EntitiesInBox(
        ctx.engine,
        GEntitiesInBoxArgs::new(
            &mins as *const vec3_t,
            &maxs as *const vec3_t,
            touch.as_mut_ptr(),
            MAX_GENTITIES as c_int,
        ),
    );

    let mut i = 0;
    while i < num {
        let hit_idx = touch[i as usize] as usize;
        if hit_idx == mover.index() {
            i += 1;
            continue;
        }
        if ctx.world.g_entities[hit_idx].r.contents & ctx.world.entity(mover).r.contents != 0 {
            return qtrue;
        }
        i += 1;
    }

    qfalse
}

/// Raven `SelectNearestDeathmatchSpawnPoint` — find the spot we DON'T want.
///
/// Source: `oracle/codemp/game/g_client.c:590-611`
pub fn SelectNearestDeathmatchSpawnPoint(ctx: &mut GameContext, from: vec3_t) -> *mut gentity_t {
    unsafe {
        let mut nearestDist: f32 = 999999.0;
        let mut nearestSpot: *mut gentity_t = core::ptr::null_mut();
        let mut spot: *mut gentity_t = core::ptr::null_mut();

        loop {
            spot = G_Find(
                ctx,
                ctx.entity_id_of(spot),
                fofs_classname(),
                "info_player_deathmatch",
            );
            if spot.is_null() {
                break;
            }
            let mut delta: vec3_t = [0.0; 3];
            for k in 0..3 {
                delta[k] = (*spot).s.origin[k] - from[k];
            }
            let dist = VectorLength(delta);
            if dist < nearestDist {
                nearestDist = dist;
                nearestSpot = spot;
            }
        }

        nearestSpot
    }
}

/// Raven `SelectRandomDeathmatchSpawnPoint` — a random non-telefragging spot.
///
/// Source: `oracle/codemp/game/g_client.c:622-645`
pub fn SelectRandomDeathmatchSpawnPoint(ctx: &mut GameContext) -> *mut gentity_t {
    pub const MAX_SPAWN_POINTS: usize = 128;
    let mut count: c_int = 0;
    let mut spot: *mut gentity_t = core::ptr::null_mut();
    let mut spots: [*mut gentity_t; MAX_SPAWN_POINTS] = [core::ptr::null_mut(); MAX_SPAWN_POINTS];

    loop {
        spot = G_Find(
            ctx,
            ctx.entity_id_of(spot),
            fofs_classname(),
            "info_player_deathmatch",
        );
        if spot.is_null() {
            break;
        }
        if SpotWouldTelefrag(ctx, ctx.entity_id_of(spot).unwrap()) != 0 {
            continue;
        }
        spots[count as usize] = spot;
        count += 1;
    }

    if count == 0 {
        // no spots that won't telefrag
        return G_Find(
            ctx,
            ctx.entity_id_of(core::ptr::null_mut()),
            fofs_classname(),
            "info_player_deathmatch",
        );
    }

    let selection = ctx.world.bg_state.rng.rand() % count;
    spots[selection as usize]
}

/// Raven `SelectRandomFurthestSpawnPoint` — a start furthest from the avoid
/// point.
///
/// Source: `oracle/codemp/game/g_client.c:654-758`
//
// fork-9: `origin`/`angles` are written through (out-params) → `&mut [f32;3]`;
// `avoidPoint` is read-only → kept by-value.
pub fn SelectRandomFurthestSpawnPoint(
    ctx: &mut GameContext,
    avoidPoint: vec3_t,
    origin: &mut [f32; 3],
    angles: &mut [f32; 3],
    team: team_t,
) -> *mut gentity_t {
    unsafe {
        let mut list_dist = [0.0f32; 64];
        let mut list_spot: [*mut gentity_t; 64] = [core::ptr::null_mut(); 64];
        let mut numSpots: c_int = 0;
        let mut spot: *mut gentity_t = core::ptr::null_mut();

        // in Team DM, look for a team start spot first, if any
        if ctx.world.cvars.g_gametype.integer == GT_TEAM
            && team != TEAM_FREE
            && team != TEAM_SPECTATOR
        {
            let classname = if team == TEAM_RED {
                "info_player_start_red"
            } else {
                "info_player_start_blue"
            };
            loop {
                spot = G_Find(
                    ctx,
                    ctx.entity_id_of(spot),
                    fofs_classname(),
                    classname,
                );
                if spot.is_null() {
                    break;
                }
                if SpotWouldTelefrag(ctx, ctx.entity_id_of(spot).unwrap()) != 0 {
                    continue;
                }
                let mut delta: vec3_t = [0.0; 3];
                for k in 0..3 {
                    delta[k] = (*spot).s.origin[k] - avoidPoint[k];
                }
                let dist = VectorLength(delta);
                let mut i = 0;
                while i < numSpots {
                    if dist > list_dist[i as usize] {
                        if numSpots >= 64 {
                            numSpots = 64 - 1;
                        }
                        let mut j = numSpots;
                        while j > i {
                            list_dist[j as usize] = list_dist[(j - 1) as usize];
                            list_spot[j as usize] = list_spot[(j - 1) as usize];
                            j -= 1;
                        }
                        list_dist[i as usize] = dist;
                        list_spot[i as usize] = spot;
                        numSpots += 1;
                        if numSpots > 64 {
                            numSpots = 64;
                        }
                        break;
                    }
                    i += 1;
                }
                if i >= numSpots && numSpots < 64 {
                    list_dist[numSpots as usize] = dist;
                    list_spot[numSpots as usize] = spot;
                    numSpots += 1;
                }
            }
        }

        if numSpots == 0 {
            // couldn't find any of the above
            spot = core::ptr::null_mut();
            loop {
                spot = G_Find(
                    ctx,
                    ctx.entity_id_of(spot),
                    fofs_classname(),
                    "info_player_deathmatch",
                );
                if spot.is_null() {
                    break;
                }
                if SpotWouldTelefrag(ctx, ctx.entity_id_of(spot).unwrap()) != 0 {
                    continue;
                }
                let mut delta: vec3_t = [0.0; 3];
                for k in 0..3 {
                    delta[k] = (*spot).s.origin[k] - avoidPoint[k];
                }
                let dist = VectorLength(delta);
                let mut i = 0;
                while i < numSpots {
                    if dist > list_dist[i as usize] {
                        if numSpots >= 64 {
                            numSpots = 64 - 1;
                        }
                        let mut j = numSpots;
                        while j > i {
                            list_dist[j as usize] = list_dist[(j - 1) as usize];
                            list_spot[j as usize] = list_spot[(j - 1) as usize];
                            j -= 1;
                        }
                        list_dist[i as usize] = dist;
                        list_spot[i as usize] = spot;
                        numSpots += 1;
                        if numSpots > 64 {
                            numSpots = 64;
                        }
                        break;
                    }
                    i += 1;
                }
                if i >= numSpots && numSpots < 64 {
                    list_dist[numSpots as usize] = dist;
                    list_spot[numSpots as usize] = spot;
                    numSpots += 1;
                }
            }
            if numSpots == 0 {
                spot = G_Find(
                    ctx,
                    ctx.entity_id_of(core::ptr::null_mut()),
                    fofs_classname(),
                    "info_player_deathmatch",
                );
                if spot.is_null() {
                    // Raven `G_Error("Couldn't find a spawn point")` drops the game
                    // (frozen Group A: `Com_Error`/`G_Error` → panic).
                    panic!("Couldn't find a spawn point");
                }
                *origin = (*spot).s.origin;
                origin[2] += 9.0;
                *angles = (*spot).s.angles;
                return spot;
            }
        }

        // select a random spot from the spawn points furthest away
        let rnd = (ctx.world.bg_state.rng.random() * ((numSpots / 2) as f32)) as c_int;

        *origin = (*list_spot[rnd as usize]).s.origin;
        origin[2] += 9.0;
        *angles = (*list_spot[rnd as usize]).s.angles;

        list_spot[rnd as usize]
    }
}

/// Raven `SelectDuelSpawnPoint` — a duel/powerduel start furthest from the avoid
/// point.
///
/// Source: `oracle/codemp/game/g_client.c:760-845`
//
// fork-9: `origin`/`angles` are written through → `&mut [f32;3]`; `avoidPoint`
// is read-only → kept by-value.
pub fn SelectDuelSpawnPoint(
    ctx: &mut GameContext,
    team: c_int,
    avoidPoint: vec3_t,
    origin: &mut [f32; 3],
    angles: &mut [f32; 3],
) -> *mut gentity_t {
    // Raven `duelTeam_t` (`bg_public.h:1019-1025`).
    const DUELTEAM_LONE: c_int = 1;
    const DUELTEAM_DOUBLE: c_int = 2;
    const DUELTEAM_SINGLE: c_int = 3;
    unsafe {
        let mut spotName: &str = if team == DUELTEAM_LONE {
            "info_player_duel1"
        } else if team == DUELTEAM_DOUBLE {
            "info_player_duel2"
        } else if team == DUELTEAM_SINGLE {
            "info_player_duel"
        } else {
            "info_player_deathmatch"
        };

        // Raven `tryAgain:` — the goto retarget becomes a loop restart.
        loop {
            let mut list_dist = [0.0f32; 64];
            let mut list_spot: [*mut gentity_t; 64] = [core::ptr::null_mut(); 64];
            let mut numSpots: c_int = 0;
            let mut spot: *mut gentity_t = core::ptr::null_mut();

            loop {
                spot =
                    G_Find(ctx, ctx.entity_id_of(spot), fofs_classname(), spotName);
                if spot.is_null() {
                    break;
                }
                if SpotWouldTelefrag(ctx, ctx.entity_id_of(spot).unwrap()) != 0 {
                    continue;
                }
                let mut delta: vec3_t = [0.0; 3];
                for k in 0..3 {
                    delta[k] = (*spot).s.origin[k] - avoidPoint[k];
                }
                let dist = VectorLength(delta);
                let mut i = 0;
                while i < numSpots {
                    if dist > list_dist[i as usize] {
                        if numSpots >= 64 {
                            numSpots = 64 - 1;
                        }
                        let mut j = numSpots;
                        while j > i {
                            list_dist[j as usize] = list_dist[(j - 1) as usize];
                            list_spot[j as usize] = list_spot[(j - 1) as usize];
                            j -= 1;
                        }
                        list_dist[i as usize] = dist;
                        list_spot[i as usize] = spot;
                        numSpots += 1;
                        if numSpots > 64 {
                            numSpots = 64;
                        }
                        break;
                    }
                    i += 1;
                }
                if i >= numSpots && numSpots < 64 {
                    list_dist[numSpots as usize] = dist;
                    list_spot[numSpots as usize] = spot;
                    numSpots += 1;
                }
            }
            if numSpots == 0 {
                if !spotName.eq_ignore_ascii_case("info_player_deathmatch") {
                    // try the loop again with info_player_deathmatch as the target
                    spotName = "info_player_deathmatch";
                    continue;
                }

                // no free duel or DM spots, just try the first DM spot
                spot = G_Find(
                    ctx,
                    ctx.entity_id_of(core::ptr::null_mut()),
                    fofs_classname(),
                    "info_player_deathmatch",
                );
                if spot.is_null() {
                    // Raven `G_Error("Couldn't find a spawn point")` drops the game
                    // (frozen Group A: `Com_Error`/`G_Error` → panic).
                    panic!("Couldn't find a spawn point");
                }
                *origin = (*spot).s.origin;
                origin[2] += 9.0;
                *angles = (*spot).s.angles;
                return spot;
            }

            // select a random spot from the spawn points furthest away
            let rnd = (ctx.world.bg_state.rng.random() * ((numSpots / 2) as f32)) as c_int;

            *origin = (*list_spot[rnd as usize]).s.origin;
            origin[2] += 9.0;
            *angles = (*list_spot[rnd as usize]).s.angles;

            return list_spot[rnd as usize];
        }
    }
}

/// Raven `SelectSpawnPoint` — chooses a player start, deathmatch start, etc.
///
/// Source: `oracle/codemp/game/g_client.c:854-884`
//
// fork-9 caller-fix: `origin`/`angles` forward to the reshaped
// `SelectRandomFurthestSpawnPoint` out-params → `&mut [f32;3]`.
pub fn SelectSpawnPoint(
    ctx: &mut GameContext,
    avoidPoint: vec3_t,
    origin: &mut [f32; 3],
    angles: &mut [f32; 3],
    team: team_t,
) -> *mut gentity_t {
    SelectRandomFurthestSpawnPoint(ctx, avoidPoint, origin, angles, team)
}

/// Raven `SelectInitialSpawnPoint` — a spawn marked 'initial', else normal
/// selection.
///
/// Source: `oracle/codemp/game/g_client.c:894-913`
//
// fork-9: `origin`/`angles` are written through → `&mut [f32;3]`.
pub fn SelectInitialSpawnPoint(
    ctx: &mut GameContext,
    origin: &mut [f32; 3],
    angles: &mut [f32; 3],
    team: team_t,
) -> *mut gentity_t {
    unsafe {
        let mut spot: *mut gentity_t = core::ptr::null_mut();
        loop {
            spot = G_Find(
                ctx,
                ctx.entity_id_of(spot),
                fofs_classname(),
                "info_player_deathmatch",
            );
            if spot.is_null() {
                break;
            }
            if (*spot).spawnflags & 1 != 0 {
                break;
            }
        }

        if spot.is_null() || SpotWouldTelefrag(ctx, ctx.entity_id_of(spot).unwrap()) != 0 {
            return SelectSpawnPoint(ctx, vec3_origin, origin, angles, team);
        }

        *origin = (*spot).s.origin;
        origin[2] += 9.0;
        *angles = (*spot).s.angles;

        spot
    }
}

/// Raven `SelectSpectatorSpawnPoint`.
///
/// Source: `oracle/codemp/game/g_client.c:921-928`
//
// fork-9: `origin`/`angles` are written through → `&mut [f32;3]`.
pub fn SelectSpectatorSpawnPoint(
    ctx: &mut GameContext,
    origin: &mut [f32; 3],
    angles: &mut [f32; 3],
) -> *mut gentity_t {
    crate::g_main::FindIntermissionPoint(ctx);

    *origin = ctx.world.level.intermission_origin;
    *angles = ctx.world.level.intermission_angle;

    core::ptr::null_mut()
}

/// Raven `InitBodyQue` — allocate the body-queue entities.
///
/// Source: `oracle/codemp/game/g_client.c:953-964`
pub fn InitBodyQue(ctx: &mut GameContext) {
    unsafe {
        ctx.world.level.bodyQueIndex = 0;
        for i in 0..BODY_QUEUE_SIZE {
            let ent_eid = crate::g_utils::G_Spawn(ctx);
            let ent = ctx.entity_mut(ent_eid) as *mut gentity_t;
            (*ent).classname = b"bodyque\0".as_ptr() as *mut c_char;
            (*ent).neverFree = qtrue;
            ctx.world.level.bodyQue[i] = ent;
        }
    }
}

/// Raven `BodySink` — after death, sink the body into the ground and remove.
///
/// Source: `oracle/codemp/game/g_client.c:973-986`
pub fn BodySink(ctx: &mut GameContext, ent: EntityId) {
    if ctx.world.level.time - ctx.world.entity(ent).timestamp > BODY_SINK_TIME + 2500 {
        // the body ques are never actually freed, they are just unlinked
        let ent_ptr: *mut gentity_t = ctx.entity_mut(ent);
        trap::UnlinkEntity(ctx.engine, GUnlinkentityArgs::new(ent_ptr.cast()));
        ctx.world.entity_mut(ent).physicsObject = qfalse;
        return;
    }

    crate::g_utils::G_AddEvent(
        ctx.world.entity_mut(ent),
        entity_event_t::EV_BODYFADE as c_int,
        0,
    );
    let nextthink = ctx.world.level.time + 18000;
    ctx.world.entity_mut(ent).nextthink = nextthink;
    ctx.world.entity_mut(ent).takedamage = qfalse;
}

/// Raven `CopyToBodyQue` — copy a dead client into the body queue.
///
/// Source: `oracle/codemp/game/g_client.c:996-1105`
pub fn CopyToBodyQue(ctx: &mut GameContext, ent: EntityId) -> qboolean {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);
    unsafe {
        if ctx.world.level.intermissiontime != 0 {
            return qfalse;
        }

        trap::UnlinkEntity(ctx.engine, GUnlinkentityArgs::new(ent.cast()));

        // if client is in a nodrop area, don't leave the body
        let contents = trap::PointContents(
            ctx.engine,
            GPointContentsArgs::new(&(*ent).s.origin as *const vec3_t, -1),
        );
        if contents & CONTENTS_NODROP != 0 {
            return qfalse;
        }

        if !(*ent).client.is_null() && (*((*ent).client)).ps.eFlags & EF_DISINTEGRATION != 0 {
            // for now, just don't spawn a body if you got disint'd
            return qfalse;
        }

        // grab a body que and cycle to the next one
        let body = ctx.world.level.bodyQue[ctx.world.level.bodyQueIndex as usize];
        ctx.world.level.bodyQueIndex =
            (ctx.world.level.bodyQueIndex + 1) % (BODY_QUEUE_SIZE) as i32;

        trap::UnlinkEntity(ctx.engine, GUnlinkentityArgs::new(body.cast()));
        (*body).s = (*ent).s;

        // avoid oddly angled corpses floating around
        (*body).s.angles[PITCH as usize] = 0.0;
        (*body).s.angles[ROLL as usize] = 0.0;
        (*body).s.apos.trBase[PITCH as usize] = 0.0;
        (*body).s.apos.trBase[ROLL as usize] = 0.0;

        (*body).s.g2radius = 100;

        (*body).s.eType = ET_BODY as c_int;
        (*body).s.eFlags = EF_DEAD; // clear EF_TALK, etc

        if !(*ent).client.is_null() && (*((*ent).client)).ps.eFlags & EF_DISINTEGRATION != 0 {
            (*body).s.eFlags |= EF_DISINTEGRATION;
        }

        crate::q_math::_VectorCopy((*((*ent).client)).ps.lastHitLoc, &mut (*body).s.origin2);

        (*body).s.powerups = 0; // clear powerups
        (*body).s.loopSound = 0; // clear lava burning
        (*body).s.loopIsSoundset = qfalse;
        (*body).s.number = ent_id(ctx.world.g_entities.as_ptr(), body).index() as c_int;
        (*body).timestamp = ctx.world.level.time;
        (*body).physicsObject = qtrue;
        (*body).physicsBounce = (0) as f32; // don't bounce
        if (*body).s.groundEntityNum == ENTITYNUM_NONE {
            (*body).s.pos.trType = TR_GRAVITY;
            (*body).s.pos.trTime = ctx.world.level.time;
            crate::q_math::_VectorCopy((*((*ent).client)).ps.velocity, &mut (*body).s.pos.trDelta);
        } else {
            (*body).s.pos.trType = TR_STATIONARY;
        }
        (*body).s.event = 0;

        (*body).s.weapon = (*ent).s.bolt2;

        if (*body).s.weapon == WP_SABER && (*((*ent).client)).ps.saberInFlight != 0 {
            // lie to keep from putting a saber on the corpse, because it was thrown at death
            (*body).s.weapon = WP_BLASTER;
        }

        // Now doing this through a modified version of the rcg reliable command.
        let islight =
            if !(*ent).client.is_null() && (*((*ent).client)).ps.fd.forceSide == FORCE_LIGHTSIDE {
                1
            } else {
                0
            };
        let cmd = format!(
            "ircg {} {} {} {}",
            (*ent).s.number,
            (*body).s.number,
            (*body).s.weapon,
            islight
        );
        trap::SendServerCommand(ctx.engine, -1, &cmd);

        (*body).r.svFlags = (*ent).r.svFlags | SVF_BROADCAST;
        crate::q_math::_VectorCopy((*ent).r.mins, &mut (*body).r.mins);
        crate::q_math::_VectorCopy((*ent).r.maxs, &mut (*body).r.maxs);
        crate::q_math::_VectorCopy((*ent).r.absmin, &mut (*body).r.absmin);
        crate::q_math::_VectorCopy((*ent).r.absmax, &mut (*body).r.absmax);

        (*body).s.torsoAnim = (*((*ent).client)).ps.legsAnim;
        (*body).s.legsAnim = (*((*ent).client)).ps.legsAnim;

        (*body).s.customRGBA[0] = (*((*ent).client)).ps.customRGBA[0];
        (*body).s.customRGBA[1] = (*((*ent).client)).ps.customRGBA[1];
        (*body).s.customRGBA[2] = (*((*ent).client)).ps.customRGBA[2];
        (*body).s.customRGBA[3] = (*((*ent).client)).ps.customRGBA[3];

        (*body).clipmask = CONTENTS_SOLID | CONTENTS_PLAYERCLIP;
        (*body).r.contents = CONTENTS_CORPSE;
        (*body).r.ownerNum = (*ent).s.number;

        (*body).nextthink = ctx.world.level.time + BODY_SINK_TIME;
        (*body).think = Some(EntThink::BodySink).into();

        (*body).die = Some(EntDie::body_die).into();

        // don't take more damage if already gibbed
        if (*ent).health <= GIB_HEALTH {
            (*body).takedamage = qfalse;
        } else {
            (*body).takedamage = qtrue;
        }

        crate::q_math::_VectorCopy((*body).s.pos.trBase, &mut (*body).r.currentOrigin);
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(body.cast()));

        qtrue
    }
}

/// Raven `MaintainBodyQueue`.
///
/// Source: `oracle/codemp/game/g_client.c:1130-1159`
pub fn MaintainBodyQueue(ctx: &mut GameContext, ent: EntityId) {
    // do whatever should be done taking ragdoll and dismemberment states into account.
    let mut do_rcg = qfalse;

    // `ent.client` is a raw `gclient_t` pointer (pool or level slot); deref it raw
    // through a copied pointer value exactly as Raven does (recipe 2b).
    let client = ctx.world.entity(ent).client;
    assert!(!client.is_null());
    let level_time = ctx.world.level.time;
    if unsafe { (*client).tempSpectate } > level_time
        || unsafe { (*client).ps.eFlags2 } & EF2_SHIP_DEATH != 0
    {
        unsafe {
            (*client).noCorpse = qtrue;
        }
    }

    if unsafe { (*client).noCorpse } == qfalse && unsafe { (*client).ps.fallingToDeath } == qfalse {
        if CopyToBodyQue(ctx, ent) == qfalse {
            do_rcg = qtrue;
        }
    } else {
        unsafe {
            (*client).noCorpse = qfalse; // clear it for next time
            (*client).ps.fallingToDeath = qfalse;
        }
        do_rcg = qtrue;
    }

    if do_rcg != qfalse {
        // bodyque func didn't manage to call ircg so call this to assure our limbs and
        // ragdoll states are proper on the client.
        let cmd = format!("rcg {}", ctx.world.entity(ent).s.clientNum);
        trap::SendServerCommand(ctx.engine, -1, &cmd);
    }
}

/// Raven `respawn` — respawn a client (or queue the body).
///
/// Source: `oracle/codemp/game/g_client.c:1167-1228`
pub fn respawn(ctx: &mut GameContext, ent: EntityId) {
    MaintainBodyQueue(ctx, ent);

    // `ent.client` is a raw `gclient_t` pointer (pool or level slot); deref it raw
    // through a copied pointer value exactly as Raven does (recipe 2b).
    let client = ctx.world.entity(ent).client;

    if ctx.world.globals.gEscaping != 0 || ctx.world.cvars.g_gametype.integer == GT_POWERDUEL {
        unsafe {
            (*client).sess.sessionTeam = TEAM_SPECTATOR;
            (*client).sess.spectatorState =
                crate::client::spectator_state::spectatorState_t::SPECTATOR_FREE;
            (*client).sess.spectatorClient = 0;

            (*client).pers.teamState.state =
                crate::client::player_team_state::playerTeamStateState_t::TEAM_BEGIN;
            (*client).sess.spectatorTime = ctx.world.level.time;
        }
        ClientSpawn(ctx, ent);
        unsafe {
            (*client).iAmALoser = qtrue;
        }
        return;
    }

    let ent_ptr: *mut gentity_t = ctx.entity_mut(ent);
    trap::UnlinkEntity(ctx.engine, GUnlinkentityArgs::new(ent_ptr.cast()));

    if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
        if ctx.world.cvars.g_siegeRespawn.integer != 0 {
            if unsafe { (*client).tempSpectate } <= ctx.world.level.time {
                let mut minDel = ctx.world.cvars.g_siegeRespawn.integer * 2000;
                if minDel < 20000 {
                    minDel = 20000;
                }
                let level_time = ctx.world.level.time;
                unsafe {
                    (*client).tempSpectate = level_time + minDel;
                    (*client).ps.stats[STAT_HEALTH as usize] = 1;
                }
                ctx.world.entity_mut(ent).health = 1;
                unsafe {
                    (*client).ps.weapon = WP_NONE;
                    (*client).ps.stats[STAT_WEAPONS as usize] = 0;
                    (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] = 0;
                    (*client).ps.stats[STAT_HOLDABLE_ITEM as usize] = 0;
                }
                ctx.world.entity_mut(ent).takedamage = qfalse;
                let ent_ptr: *mut gentity_t = ctx.entity_mut(ent);
                trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent_ptr.cast()));

                // Respawn time.
                if ctx.world.entity(ent).s.number < MAX_CLIENTS as c_int {
                    let origin = unsafe { (*client).ps.origin };
                    let te_eid = crate::g_utils::G_TempEntity(
                        ctx,
                        origin,
                        entity_event_t::EV_SIEGESPEC as c_int,
                    );
                    let te = ctx.entity_mut(te_eid) as *mut gentity_t;
                    unsafe {
                        (*te).s.time = ctx.world.globals.g_siegeRespawnCheck;
                        (*te).s.owner = ctx.world.entity(ent).s.number;
                    }
                }

                return;
            }
        }
        crate::g_saga::SiegeRespawn(ctx, ent);
    } else {
        ClientSpawn(ctx, ent);

        // add a teleportation effect
        let origin = unsafe { (*client).ps.origin };
        let tent_eid = crate::g_utils::G_TempEntity(
            ctx,
            origin,
            entity_event_t::EV_PLAYER_TELEPORT_IN as c_int,
        );
        let tent = ctx.entity_mut(tent_eid) as *mut gentity_t;
        let client_num = ctx.world.entity(ent).s.clientNum;
        unsafe {
            (*tent).s.clientNum = client_num;
        }
    }
}

/// Raven `TeamCount` — count players on a team.
///
/// Source: `oracle/codemp/game/g_client.c:1237-1259`
pub fn TeamCount(ctx: &mut GameContext, ignoreClientNum: c_int, team: c_int) -> team_t {
    let mut count: c_int = 0;
    let mut i: c_int = 0;
    while i < ctx.world.level.maxclients {
        if i == ignoreClientNum {
            i += 1;
            continue;
        }
        // i < maxclients: a real client slot, so the owned clients arena aliases
        // `level.clients[i]` byte-for-byte (recipe 2b).
        if ctx.world.client(i as usize).pers.connected == CON_DISCONNECTED {
            i += 1;
            continue;
        }
        if ctx.world.client(i as usize).sess.sessionTeam == team {
            count += 1;
        } else if ctx.world.cvars.g_gametype.integer == GT_SIEGE
            && ctx.world.client(i as usize).sess.siegeDesiredTeam == team
        {
            count += 1;
        }
        i += 1;
    }
    count
}

/// Raven `TeamLeader` — find a team's leader client number.
///
/// Source: `oracle/codemp/game/g_client.c:1268-1282`
pub fn TeamLeader(ctx: &mut GameContext, team: c_int) -> c_int {
    let mut i: c_int = 0;
    while i < ctx.world.level.maxclients {
        // i < maxclients: a real client slot (recipe 2b).
        if ctx.world.client(i as usize).pers.connected == CON_DISCONNECTED {
            i += 1;
            continue;
        }
        if ctx.world.client(i as usize).sess.sessionTeam == team {
            if ctx.world.client(i as usize).sess.teamLeader != 0 {
                return i;
            }
        }
        i += 1;
    }
    -1
}

/// Raven `PickTeam` — pick the emptier team.
///
/// Source: `oracle/codemp/game/g_client.c:1291-1308`
pub fn PickTeam(ctx: &mut GameContext, ignoreClientNum: c_int) -> team_t {
    let mut counts = [0 as c_int; TEAM_NUM_TEAMS as usize];

    counts[TEAM_BLUE as usize] = TeamCount(ctx, ignoreClientNum, TEAM_BLUE);
    counts[TEAM_RED as usize] = TeamCount(ctx, ignoreClientNum, TEAM_RED);

    if counts[TEAM_BLUE as usize] > counts[TEAM_RED as usize] {
        return TEAM_RED;
    }
    if counts[TEAM_RED as usize] > counts[TEAM_BLUE as usize] {
        return TEAM_BLUE;
    }
    // equal team count, so join the team with the lowest score
    if ctx.world.level.teamScores[TEAM_BLUE as usize]
        > ctx.world.level.teamScores[TEAM_RED as usize]
    {
        return TEAM_RED;
    }
    TEAM_BLUE
}

/// Raven `ClientCleanName` — sanitize a player name (colors, spaces, length).
///
/// Source: `oracle/codemp/game/g_client.c:1335-1410`
pub fn ClientCleanName(ctx: &mut GameContext, r#in: &str, outSize: c_int) -> String {
    let _ = ctx;
    // Q_COLOR_ESCAPE == '^'; ColorIndex(c) == ((c - '0') & 0x07). All logic is
    // byte-positional (Raven counts bytes and the length bounds are byte
    // bounds): iterate the input bytes, build a `Vec<u8>`, lossy-decode once at
    // the end so byte positions match Raven exactly (§13 convention 6).
    const Q_COLOR_ESCAPE: u8 = b'^';
    #[inline]
    fn ColorIndex(c: u8) -> c_int {
        (c as c_int - '0' as c_int) & 0x07
    }

    // save room for trailing null byte
    let outSize = outSize - 1;

    let in_bytes = r#in.as_bytes();
    let mut out: Vec<u8> = Vec::new(); // `*p == 0` <=> `out.is_empty()` (only non-NUL bytes are ever written)
    let mut len: c_int = 0;
    let mut colorlessLen: c_int = 0;
    let mut spaces: c_int = 0;

    let mut i = 0usize;
    while i < in_bytes.len() {
        let ch = in_bytes[i];
        i += 1;

        // don't allow leading spaces
        if out.is_empty() && ch == b' ' {
            continue;
        }

        // check colors
        if ch == Q_COLOR_ESCAPE {
            // solo trailing carat is not a color prefix (Raven: `*inp == 0`)
            if i >= in_bytes.len() {
                break;
            }

            // don't allow black in a name, period
            if ColorIndex(in_bytes[i]) == 0 {
                i += 1;
                continue;
            }

            // make sure room in dest for both chars
            if len > outSize - 2 {
                break;
            }

            out.push(ch);
            out.push(in_bytes[i]);
            i += 1;
            len += 2;
            continue;
        }

        // don't allow too many consecutive spaces
        if ch == b' ' {
            spaces += 1;
            if spaces > 3 {
                continue;
            }
        } else {
            spaces = 0;
        }

        if len > outSize - 1 {
            break;
        }

        out.push(ch);
        colorlessLen += 1;
        len += 1;
    }

    // don't allow empty names
    if out.is_empty() || colorlessLen == 0 {
        // Raven `Q_strncpyz(p, "Padawan", outSize)` with the already-decremented
        // `outSize` as the buffer bound.
        return strncpyz_string(b"Padawan", outSize.max(0) as usize);
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Raven `G_SaberModelSetup`.
///
/// Source: `oracle/codemp/game/g_client.c:1423-1499`
pub fn G_SaberModelSetup(ctx: &mut GameContext, ent: EntityId) -> qboolean {
    use mp_abi::game::syscalls::G_G2_CLEANMODELS::GG2CleanmodelsArgs;
    use mp_abi::game::syscalls::G_G2_COPYSPECIFICGHOUL2MODEL::GG2Copyspecificghoul2ModelArgs;
    use mp_abi::game::syscalls::G_G2_SETBOLTINFO::GG2SetboltinfoArgs;
    use mp_abi::game::syscalls::G_G2_SETSKIN::GG2SetskinArgs;

    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);
    unsafe {
        let mut i: usize = 0;
        let mut fallback_for_saber = qtrue;

        while i < MAX_SABERS as usize {
            if (*((*ent).client)).saber[i].model[0] != 0 {
                // first kill it off if we've already got it
                if !(*((*ent).client)).weaponGhoul2[i].is_null() {
                    trap::G2API_CleanGhoul2Models(
                        ctx.engine,
                        GG2CleanmodelsArgs::new(
                            &mut (*((*ent).client)).weaponGhoul2[i] as *mut *mut c_void,
                        ),
                    );
                }
                let model_name = cstr_to_str((*((*ent).client)).saber[i].model.as_ptr());
                trap::G2API_InitGhoul2Model(
                    ctx.engine,
                    &mut (*((*ent).client)).weaponGhoul2[i] as *mut *mut c_void,
                    &model_name,
                    0,
                    0,
                    -20,
                    0,
                    0,
                );

                if !(*((*ent).client)).weaponGhoul2[i].is_null() {
                    let mut j: usize = 0;
                    let mut tag_bolt: c_int;

                    if (*((*ent).client)).saber[i].skin != 0 {
                        trap::G2API_SetSkin(
                            ctx.engine,
                            GG2SetskinArgs::new(
                                (*((*ent).client)).weaponGhoul2[i],
                                0,
                                (*((*ent).client)).saber[i].skin,
                                (*((*ent).client)).saber[i].skin,
                            ),
                        );
                    }

                    if (*((*ent).client)).saber[i].saberFlags & SFL_BOLT_TO_WRIST != 0 {
                        trap::G2API_SetBoltInfo(
                            ctx.engine,
                            GG2SetboltinfoArgs::new(
                                (*((*ent).client)).weaponGhoul2[i],
                                0,
                                3 + i as c_int,
                            ),
                        );
                    } else {
                        // bolt to right hand for 0, or left hand for 1
                        trap::G2API_SetBoltInfo(
                            ctx.engine,
                            GG2SetboltinfoArgs::new(
                                (*((*ent).client)).weaponGhoul2[i],
                                0,
                                i as c_int,
                            ),
                        );
                    }

                    // Add all the bolt points
                    while j < (*((*ent).client)).saber[i].numBlades as usize {
                        let tag_name = format!("*blade{}", j + 1);
                        tag_bolt = trap::G2API_AddBolt(
                            ctx.engine,
                            (*((*ent).client)).weaponGhoul2[i],
                            0,
                            &tag_name,
                        );

                        if tag_bolt == -1 {
                            if j == 0 {
                                // guess this is an 0ldsk3wl saber
                                let _ = trap::G2API_AddBolt(
                                    ctx.engine,
                                    (*((*ent).client)).weaponGhoul2[i],
                                    0,
                                    "*flash",
                                );
                                fallback_for_saber = qfalse;
                                break;
                            }

                            if tag_bolt == -1 {
                                assert!(false);
                                break;
                            }
                        }
                        j += 1;

                        // got at least one custom saber so don't need default
                        fallback_for_saber = qfalse;
                    }

                    // Copy it into the main instance
                    trap::G2API_CopySpecificGhoul2Model(
                        ctx.engine,
                        GG2Copyspecificghoul2ModelArgs::new(
                            (*((*ent).client)).weaponGhoul2[i],
                            0,
                            (*ent).ghoul2,
                            i as c_int + 1,
                        ),
                    );
                }
            } else {
                break;
            }

            i += 1;
        }

        fallback_for_saber
    }
}

/// Raven `ClientConnect`.
///
/// Source: `oracle/codemp/game/g_client.c:2258-2373`
pub fn ClientConnect(
    ctx: &mut GameContext,
    clientNum: c_int,
    firstTime: qboolean,
    isBot: qboolean,
) -> *mut c_char {
    unsafe {
        let ent = ctx.world.g_entities.as_mut_ptr().add(clientNum as usize);

        let userinfo = trap::GetUserinfo(ctx.engine, clientNum, MAX_INFO_STRING as usize);

        // check to see if they are on the banned IP list
        let value = Info_ValueForKey(&userinfo, "ip");
        let ip_string = value.clone();

        if crate::g_svcmds::G_FilterPacket(ctx, cstr(&value).as_ptr() as *mut c_char) != qfalse {
            return b"Banned.\0".as_ptr() as *mut c_char;
        }

        if (*ent).r.svFlags & SVF_BOT == 0
            && isBot == qfalse
            && ctx.world.cvars.g_needpass.integer != 0
        {
            // check for a password
            let value = Info_ValueForKey(&userinfo, "password");
            let g_password = cstr_to_str(ctx.world.cvars.g_password.string.as_ptr());
            if !g_password.is_empty()
                && !g_password.eq_ignore_ascii_case("none")
                && g_password != value
            {
                // Raven returns a `static char sTemp[1024]` here; a leaked owned buffer
                // is the defined-behavior stand-in. Source: oracle/codemp/game/g_client.c:2285
                let s = G_GetStringEdString(ctx, "MP_SVGAME", "INVALID_ESCAPE_TO_MAIN");
                return cstr(&s).into_raw();
            }
        }

        // they can connect
        (*ent).client = ctx.world.clients.as_mut_ptr().add(clientNum as usize);
        let client = (*ent).client;

        // assign the pointer for bg entity access
        (*ent).playerState = &mut (*client).ps;

        // Raven `memset(client, 0, sizeof(*client))`: reset the whole gclient to
        // its zero image. `pers.netname` is a `String` (not zero-valid), so the
        // assignment drops the prior occupant's name and installs the empty
        // default rather than byte-zeroing over a live `String`.
        *client = gclient_t::default();

        (*client).pers.connected = CON_CONNECTING as _;

        // read or initialize the session data
        if firstTime != qfalse || ctx.world.level.newSession != qfalse {
            G_InitSessionData(ctx, clientNum as usize, &userinfo, isBot);
        }
        G_ReadSessionData(ctx, clientNum as usize);

        (*client).sess.IPstring = strncpyz_string(ip_string.as_bytes(), 32);

        if ctx.world.cvars.g_gametype.integer == GT_SIEGE
            && (firstTime != qfalse || ctx.world.level.newSession != qfalse)
        {
            // if this is the first time then auto-assign a desired siege team and show
            // briefing for that team
            (*client).sess.siegeDesiredTeam = 0;
        }

        if ctx.world.cvars.g_gametype.integer == GT_SIEGE
            && (*client).sess.sessionTeam != TEAM_SPECTATOR
        {
            if firstTime != qfalse || ctx.world.level.newSession != qfalse {
                // start as spec
                (*client).sess.siegeDesiredTeam = (*client).sess.sessionTeam as c_int;
                (*client).sess.sessionTeam = TEAM_SPECTATOR;
            }
        } else if ctx.world.cvars.g_gametype.integer == GT_POWERDUEL
            && (*client).sess.sessionTeam != TEAM_SPECTATOR
        {
            (*client).sess.sessionTeam = TEAM_SPECTATOR;
        }

        if isBot != qfalse {
            (*ent).r.svFlags |= SVF_BOT;
            (*ent).inuse = qtrue;
            if !crate::g_bot::G_BotConnect(ctx, clientNum, firstTime == qfalse) {
                return b"BotConnectfailed\0".as_ptr() as *mut c_char;
            }
        }

        // get and distribute relevant parameters
        G_LogPrintf(ctx, &format!("ClientConnect: {}\n", clientNum));
        ClientUserinfoChanged(ctx, clientNum);
        G_LogPrintf(
            ctx,
            &format!(
                "{} connected with IP: {}\n",
                (*client).pers.netname.clone(),
                (*client).sess.IPstring.clone(),
            ),
        );

        // don't do the "xxx connected" messages if they were carried over from previous level
        if firstTime != qfalse {
            let m = format!(
                "print \"{}{} {}\n\"",
                (*client).pers.netname.clone(),
                S_COLOR_WHITE,
                G_GetStringEdString(ctx, "MP_SVGAME", "PLCONNECT"),
            );
            trap::SendServerCommand(ctx.engine, -1, &m);
        }

        if ctx.world.cvars.g_gametype.integer >= GT_TEAM
            && (*client).sess.sessionTeam != TEAM_SPECTATOR
        {
            BroadcastTeamChange(ctx, EntityId(clientNum as u32), -1);
        }

        // count current clients and rank for scoreboard
        CalculateRanks(ctx);

        let te_eid = G_TempEntity(ctx, [0.0, 0.0, 0.0], (EV_CLIENTJOIN) as i32);
        let te = ctx.entity_mut(te_eid) as *mut gentity_t;
        (*te).r.svFlags |= SVF_BROADCAST;
        (*te).s.eventParm = clientNum;

        core::ptr::null_mut()
    }
}

/// Raven `ClientBegin`.
///
/// Source: `oracle/codemp/game/g_client.c:2393-2593`
pub fn ClientBegin(ctx: &mut GameContext, clientNum: c_int, allowTeamReset: qboolean) {
    unsafe {
        let ent = ctx.world.g_entities.as_mut_ptr().add(clientNum as usize);

        if (*ent).r.svFlags & SVF_BOT != 0 && ctx.world.cvars.g_gametype.integer >= GT_TEAM {
            if allowTeamReset != qfalse {
                let mut team_str = "Red";

                (*((*ent).client)).sess.sessionTeam = PickTeam(ctx, -1);
                let mut userinfo = trap::GetUserinfo(ctx.engine, clientNum, MAX_INFO_STRING as usize);
                if (*((*ent).client)).sess.sessionTeam == TEAM_SPECTATOR {
                    (*((*ent).client)).sess.sessionTeam = TEAM_RED;
                }

                if (*((*ent).client)).sess.sessionTeam == TEAM_RED {
                    team_str = "Red";
                } else {
                    team_str = "Blue";
                }

                Info_SetValueForKey(&mut userinfo, "team", team_str);

                trap::SetUserinfo(ctx.engine, clientNum, &userinfo);

                (*((*ent).client)).ps.persistant[PERS_TEAM as usize] =
                    (*((*ent).client)).sess.sessionTeam as c_int;

                let pre_sess = (*((*ent).client)).sess.sessionTeam;
                G_ReadSessionData(ctx, clientNum as usize);
                (*((*ent).client)).sess.sessionTeam = pre_sess;
                G_WriteClientSessionData(ctx, clientNum as usize);
                ClientUserinfoChanged(ctx, clientNum);
                ClientBegin(ctx, clientNum, qfalse);
                return;
            }
        }

        let client = ctx.world.clients.as_mut_ptr().add(clientNum as usize);

        if (*ent).r.linked != qfalse {
            trap::UnlinkEntity(ctx.engine, GUnlinkentityArgs::new(ent.cast()));
        }
        crate::g_utils::G_InitGentity(ctx, ctx.entity_id_of(ent).unwrap());
        (*ent).touch = FnId::NONE;
        (*ent).pain = FnId::NONE;
        (*ent).client = client;

        // assign the pointer for bg entity access
        (*ent).playerState = &mut (*((*ent).client)).ps;

        (*client).pers.connected = CON_CONNECTED as _;
        (*client).pers.enterTime = ctx.world.level.time;
        (*client).pers.teamState.state =
            crate::client::player_team_state::playerTeamStateState_t::TEAM_BEGIN;

        // save eflags around this, because changing teams will cause this to happen with a
        // valid entity, and we want to make sure the teleport bit is set right so the
        // viewpoint doesn't interpolate through the world to the new position
        let flags = (*client).ps.eFlags;

        let mut i = 0;
        while i < NUM_FORCE_POWERS {
            if (*((*ent).client)).ps.fd.forcePowersActive & (1 << i) != 0 {
                WP_ForcePowerStop(ctx, ctx.entity_id_of(ent).unwrap(), i as forcePowers_t);
            }
            i += 1;
        }

        i = (TRACK_CHANNEL_1) as usize;
        while i < (NUM_TRACK_CHANNELS) as usize {
            let idx = (i - 50) as usize;
            if (*((*ent).client)).ps.fd.killSoundEntIndex[idx] != 0
                && (*((*ent).client)).ps.fd.killSoundEntIndex[idx] < MAX_GENTITIES as c_int
                && (*((*ent).client)).ps.fd.killSoundEntIndex[idx] > 0
            {
                G_MuteSound(
                    ctx,
                    (*((*ent).client)).ps.fd.killSoundEntIndex[idx],
                    CHAN_VOICE,
                );
            }
            i += 1;
        }

        let ps_ptr = &mut (*client).ps as *mut playerState_t;
        core::ptr::write_bytes(ps_ptr, 0, 1);
        (*client).ps.eFlags = flags;

        (*client).ps.hasDetPackPlanted = qfalse;

        // first-time force power initialization
        WP_InitForcePowers(ctx, ctx.entity_id_of(ent));

        // init saber ent
        crate::w_saber::WP_SaberInitBladeData(ctx, ctx.entity_id_of(ent).unwrap());

        // First time model setup for that player.
        let mut userinfo = trap::GetUserinfo(ctx.engine, clientNum, MAX_INFO_STRING as usize);
        let modelname = Info_ValueForKey(&userinfo, "model");
        SetupGameGhoul2Model(
            ctx,
            ctx.entity_id_of(ent).unwrap(),
            cstr(&modelname).as_ptr() as *mut c_char,
            core::ptr::null_mut(),
        );

        if !(*ent).ghoul2.is_null() && !(*ent).client.is_null() {
            (*((*ent).client)).renderInfo.lastG2 = core::ptr::null_mut();
            // update the renderinfo bolts next update.
        }

        if ctx.world.cvars.g_gametype.integer == GT_POWERDUEL
            && (*client).sess.sessionTeam != TEAM_SPECTATOR
            && (*client).sess.duelTeam == DUELTEAM_FREE as c_int
        {
            SetTeam(ctx, ctx.entity_id_of(ent).unwrap(), "s");
        } else {
            if ctx.world.cvars.g_gametype.integer == GT_SIEGE
                && (ctx.world.globals.gSiegeRoundBegun == qfalse
                    || ctx.world.globals.gSiegeRoundEnded != qfalse)
            {
                SetTeamQuick(
                    ctx,
                    ctx.entity_id_of(ent).unwrap(),
                    TEAM_SPECTATOR as c_int,
                    qfalse,
                );
            }

            if (*ent).r.svFlags & SVF_BOT != 0 && ctx.world.cvars.g_gametype.integer != GT_SIEGE {
                let saber_val = Info_ValueForKey(&userinfo, "saber1");
                let saber2_val = Info_ValueForKey(&userinfo, "saber2");

                if saber_val.is_empty() {
                    // blah, set em up with a random saber
                    let r = ctx.world.bg_state.rng.rand() % 50;
                    let (sab1, sab2);
                    if r <= 17 {
                        sab1 = "Katarn";
                        sab2 = "none";
                    } else if r <= 34 {
                        sab1 = "Katarn";
                        sab2 = "Katarn";
                    } else {
                        sab1 = "dual_1";
                        sab2 = "none";
                    }
                    G_SetSaber(ctx, ctx.entity_id_of(ent).unwrap(), 0, sab1, qfalse);
                    G_SetSaber(ctx, ctx.entity_id_of(ent).unwrap(), 0, sab2, qfalse);
                    Info_SetValueForKey(&mut userinfo, "saber1", sab1);
                    Info_SetValueForKey(&mut userinfo, "saber2", sab2);
                    trap::SetUserinfo(ctx.engine, clientNum, &userinfo);
                } else {
                    G_SetSaber(ctx, ctx.entity_id_of(ent).unwrap(), 0, &saber_val, qfalse);
                }

                if !saber_val.is_empty() && saber2_val.is_empty() {
                    G_SetSaber(ctx, ctx.entity_id_of(ent).unwrap(), 0, "none", qfalse);
                    Info_SetValueForKey(&mut userinfo, "saber2", "none");
                    trap::SetUserinfo(ctx.engine, clientNum, &userinfo);
                } else {
                    G_SetSaber(ctx, ctx.entity_id_of(ent).unwrap(), 0, &saber2_val, qfalse);
                }
            }

            // locate ent at a spawn point
            ClientSpawn(ctx, ctx.entity_id_of(ent).unwrap());
        }

        if (*client).sess.sessionTeam != TEAM_SPECTATOR {
            // send event
            let tent_eid = G_TempEntity(ctx, (*client).ps.origin, (EV_PLAYER_TELEPORT_IN) as i32);
            let tent = ctx.entity_mut(tent_eid) as *mut gentity_t;
            (*tent).s.clientNum = (*ent).s.clientNum;

            if ctx.world.cvars.g_gametype.integer != GT_DUEL
                || ctx.world.cvars.g_gametype.integer == GT_POWERDUEL
            {
                let m = format!(
                    "print \"{}{} {}\n\"",
                    (*client).pers.netname.clone(),
                    S_COLOR_WHITE,
                    G_GetStringEdString(ctx, "MP_SVGAME", "PLENTER"),
                );
                trap::SendServerCommand(ctx.engine, -1, &m);
            }
        }
        G_LogPrintf(ctx, &format!("ClientBegin: {}\n", clientNum));

        // count current clients and rank for scoreboard
        CalculateRanks(ctx);

        crate::g_log::G_ClearClientLog(ctx, clientNum);
    }
}

/// Raven `AllForceDisabled` — are all force powers disabled by the mask?
///
/// Source: `oracle/codemp/game/g_client.c:2595-2613`
pub fn AllForceDisabled(force: c_int) -> qboolean {
    if force != 0 {
        for i in 0..NUM_FORCE_POWERS {
            if force & (1 << i) == 0 {
                return qfalse;
            }
        }
        return qtrue;
    }

    qfalse
}

/// Raven `G_BreakArm` — set up a broken-limb state and pain animation.
///
/// Source: `oracle/codemp/game/g_client.c:2616-2674`
pub fn G_BreakArm(ctx: &mut GameContext, ent: EntityId, arm: c_int) {
    let mut anim: c_int = -1;

    // `ent.client` is a raw `gclient_t` pointer (pool or level slot); deref it raw
    // through a copied pointer value exactly as Raven does (recipe 2b).
    let client = ctx.world.entity(ent).client;
    assert!(!client.is_null());

    if ctx.world.entity(ent).s.NPC_class == CLASS_VEHICLE as c_int
        || ctx.world.entity(ent).localAnimIndex > 1
    {
        // no broken limbs for vehicles and non-humanoids
        return;
    }

    if arm == 0 {
        // repair him
        unsafe {
            (*client).ps.brokenLimbs = 0;
        }
        return;
    }

    if unsafe { (*client).ps.fd.saberAnimLevel } == SS_STAFF as c_int {
        // I'm too lazy to deal with this as well for now.
        return;
    }

    if arm == (BROKENLIMB_LARM) as i32 {
        if unsafe {
            (*client).saber[1].model[0] != 0
                && (*client).ps.weapon == WP_SABER
                && (*client).ps.saberHolstered == 0
                && (*client).saber[1].soundOff != 0
        } {
            // the left arm shuts off its saber upon being broken
            let sound_off = unsafe { (*client).saber[1].soundOff };
            G_Sound(ctx, Some(ent), CHAN_AUTO, sound_off);
        }
    }

    unsafe {
        (*client).ps.brokenLimbs = 0; // make sure it's cleared out
        (*client).ps.brokenLimbs |= 1 << arm; // this arm is now marked as broken
    }

    // Do a pain anim based on the side. Since getting your arm broken does tend to hurt.
    if arm == (BROKENLIMB_LARM) as i32 {
        anim = (BOTH_PAIN2) as i32;
    } else if arm == (BROKENLIMB_RARM) as i32 {
        anim = (BOTH_PAIN3) as i32;
    }

    if anim == -1 {
        return;
    }

    let cmd_ptr: *mut usercmd_t = unsafe { &raw mut (*client).pers.cmd };
    G_SetAnim(
        ctx,
        ent,
        cmd_ptr,
        SETANIM_BOTH,
        anim,
        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        0,
    );

    // This could be combined into a single event. But I guess limbs don't break often
    // enough to worry about it.
    G_EntitySound(
        ctx,
        ent,
        CHAN_VOICE,
        G_SoundIndex("*pain25.wav"),
    );
    // FIXME: A nice bone snapping sound instead if possible
    let n = ctx.world.bg_state.rng.Q_irand(1, 3);
    G_Sound(
        ctx,
        Some(ent),
        CHAN_AUTO,
        G_SoundIndex(&format!("sound/player/bodyfall_human{}.wav", n)),
    );
}

/// Raven `G_UpdateClientAnims`.
///
/// Source: `oracle/codemp/game/g_client.c:2681-2926`
pub fn G_UpdateClientAnims(ctx: &mut GameContext, self_: EntityId, mut animSpeedScale: f32) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    // The `#if 0` broken-limb bone block (g_client.c:2804-2925) is disabled in
    // the oracle itself; dropped per §20.
    unsafe {
        let torso_anim = ((*self_).client).as_ref().unwrap().ps.torsoAnim;
        let legs_anim = (*((*self_).client)).ps.legsAnim;
        let mut set_torso = qfalse;
        let mut first_frame: c_int = 0;
        let mut last_frame: c_int = 0;
        let mut a_flags: c_int;
        // C declares `animSpeed`/`lAnimSpeedScale` as `float`; the divide and the
        // `*= animSpeedScale` must both round to f32, so keep the intermediate f32.
        let mut anim_speed: f32;
        let mut l_anim_speed_scale: f32;

        if (*((*self_).client)).ps.saberLockFrame != 0 {
            let frame = (*((*self_).client)).ps.saberLockFrame;
            trap::G2API_SetBoneAnim(
                ctx.engine,
                (*self_).ghoul2,
                0,
                "model_root",
                frame,
                frame + 1,
                BONE_ANIM_OVERRIDE_FREEZE | BONE_ANIM_BLEND,
                animSpeedScale,
                ctx.world.level.time,
                -1.0,
                150,
            );
            trap::G2API_SetBoneAnim(
                ctx.engine,
                (*self_).ghoul2,
                0,
                "lower_lumbar",
                frame,
                frame + 1,
                BONE_ANIM_OVERRIDE_FREEZE | BONE_ANIM_BLEND,
                animSpeedScale,
                ctx.world.level.time,
                -1.0,
                150,
            );
            trap::G2API_SetBoneAnim(
                ctx.engine,
                (*self_).ghoul2,
                0,
                "Motion",
                frame,
                frame + 1,
                BONE_ANIM_OVERRIDE_FREEZE | BONE_ANIM_BLEND,
                animSpeedScale,
                ctx.world.level.time,
                -1.0,
                150,
            );
            return;
        }

        let all_anims = &(&ctx.world.bg_state.bgAllAnims)[(*self_).localAnimIndex as usize];

        let skip_legs = (*self_).localAnimIndex > 1
            && (*all_anims.anims.add(legs_anim as usize)).firstFrame == 0
            && (*all_anims.anims.add(legs_anim as usize)).numFrames == 0;

        if !skip_legs {
            if (*((*self_).client)).legsAnimExecute != legs_anim
                || (*((*self_).client)).legsLastFlip != (*((*self_).client)).ps.legsFlip
            {
                let anim = &*all_anims.anims.add(legs_anim as usize);
                anim_speed = 50.0f32 / anim.frameLerp as f32;
                anim_speed *= animSpeedScale;
                l_anim_speed_scale = anim_speed;

                a_flags = if anim.loopFrames != -1 {
                    BONE_ANIM_OVERRIDE_LOOP
                } else {
                    BONE_ANIM_OVERRIDE_FREEZE
                };

                if anim_speed < 0.0 {
                    last_frame = anim.firstFrame as i32;
                    first_frame = anim.firstFrame as i32 + anim.numFrames as i32;
                } else {
                    first_frame = anim.firstFrame as i32;
                    last_frame = anim.firstFrame as i32 + anim.numFrames as i32;
                }

                a_flags |= BONE_ANIM_BLEND;

                trap::G2API_SetBoneAnim(
                    ctx.engine,
                    (*self_).ghoul2,
                    0,
                    "model_root",
                    first_frame,
                    last_frame,
                    a_flags,
                    l_anim_speed_scale,
                    ctx.world.level.time,
                    -1.0,
                    150,
                );
                (*((*self_).client)).legsAnimExecute = legs_anim;
                (*((*self_).client)).legsLastFlip = (*((*self_).client)).ps.legsFlip;
            }
        }

        // tryTorso:
        let all_anims = &(&ctx.world.bg_state.bgAllAnims)[(*self_).localAnimIndex as usize];
        if (*self_).localAnimIndex > 1
            && (*all_anims.anims.add(torso_anim as usize)).firstFrame == 0
            && (*all_anims.anims.add(torso_anim as usize)).numFrames == 0
        {
            // If this fails as well just return.
            return;
        } else if (*self_).s.number >= MAX_CLIENTS as c_int
            && (*self_).s.NPC_class == CLASS_VEHICLE as c_int
        {
            // we only want to set the root bone for vehicles
            return;
        }

        let (mut a_flags2, mut first2, mut last2, mut speed2) = (0, 0, 0, 0.0f32);

        if ((*((*self_).client)).torsoAnimExecute != torso_anim
            || (*((*self_).client)).torsoLastFlip != (*((*self_).client)).ps.torsoFlip)
            && (*self_).noLumbar == qfalse
        {
            let mut f = torso_anim;

            mp_bg::bg_panimate::BG_SaberStartTransAnim(
                (*self_).s.number,
                (*((*self_).client)).ps.fd.saberAnimLevel,
                (*((*self_).client)).ps.weapon,
                f,
                &mut animSpeedScale as *mut f32,
                (*((*self_).client)).ps.brokenLimbs,
                // STAGE-2b: irreducible — the ruling-21 `GameCallbacksImpl` seam
                // adapter holds a raw `*mut GameWorld`; `my_saber` reaches the game
                // arena by client number (replaces the old `g_entities` base arg).
                &mut crate::bg_channel::GameCallbacksImpl {
                    world: ctx.world_raw(),
                    engine: ctx.engine,
                },
            );

            let all_anims = &(&ctx.world.bg_state.bgAllAnims)[(*self_).localAnimIndex as usize];
            let anim = &*all_anims.anims.add(f as usize);
            anim_speed = 50.0f32 / anim.frameLerp as f32;
            anim_speed *= animSpeedScale;
            speed2 = anim_speed;

            a_flags2 = if anim.loopFrames != -1 {
                BONE_ANIM_OVERRIDE_LOOP
            } else {
                BONE_ANIM_OVERRIDE_FREEZE
            };
            a_flags2 |= BONE_ANIM_BLEND;

            if anim_speed < 0.0 {
                last2 = anim.firstFrame as i32;
                first2 = anim.firstFrame as i32 + anim.numFrames as i32;
            } else {
                first2 = anim.firstFrame as i32;
                last2 = anim.firstFrame as i32 + anim.numFrames as i32;
            }

            trap::G2API_SetBoneAnim(
                ctx.engine,
                (*self_).ghoul2,
                0,
                "lower_lumbar",
                first2,
                last2,
                a_flags2,
                speed2,
                ctx.world.level.time,
                -1.0,
                150,
            );

            (*((*self_).client)).torsoAnimExecute = torso_anim;
            (*((*self_).client)).torsoLastFlip = (*((*self_).client)).ps.torsoFlip;

            set_torso = qtrue;
            first_frame = first2;
            last_frame = last2;
            a_flags = a_flags2;
            l_anim_speed_scale = speed2;
        } else {
            a_flags = a_flags2;
            l_anim_speed_scale = speed2;
        }

        if set_torso != qfalse && (*self_).localAnimIndex <= 1 {
            // only set the motion bone for humanoids.
            trap::G2API_SetBoneAnim(
                ctx.engine,
                (*self_).ghoul2,
                0,
                "Motion",
                first_frame,
                last_frame,
                a_flags,
                l_anim_speed_scale,
                ctx.world.level.time,
                -1.0,
                150,
            );
        }
    }
}

/// Raven `ClientSpawn` — spawn/respawn a client into the world (864 LOC).
///
/// Source: `oracle/codemp/game/g_client.c:2938-3801`
pub fn ClientSpawn(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);
    unsafe {
        let index = ent_id(ctx.world.g_entities.as_ptr(), ent).index() as c_int;
        let client = (*ent).client;

        // first we want the userinfo so we can see if we should update this client's saber
        let mut userinfo = trap::GetUserinfo(ctx.engine, index, MAX_INFO_STRING as usize);
        let mut changed_saber = qfalse;

        let mut l: c_int = 0;
        while l < (MAX_SABERS) as i32 {
            let saber = match l {
                0 => Some((*client).sess.saberType.clone()),
                1 => Some((*client).sess.saber2Type.clone()),
                _ => None,
            };

            let key = format!("saber{}", l + 1);
            let value = Info_ValueForKey(&userinfo, &key);
            // Raven's `value &&` is always true (Info_ValueForKey never
            // returns NULL) — the empty string still enters here.
            if let Some(ref saber) = saber {
                if !value.eq_ignore_ascii_case(saber)
                    || saber.is_empty()
                    || (*client).saber[0].model[0] == 0
                {
                    if G_SetSaber(ctx, ctx.entity_id_of(ent).unwrap(), l, &value, qfalse)
                        != qfalse
                    {
                        changed_saber = qtrue;
                    } else if saber.is_empty() || (*client).saber[0].model[0] == 0 {
                        changed_saber = qtrue;
                    }
                }
            }
            l += 1;
        }

        if changed_saber != qfalse {
            // make sure our new info is sent out to all the other clients, and give us a
            // valid stance
            ClientUserinfoChanged(ctx, (*ent).s.number);

            // make sure the saber models are updated
            G_SaberModelSetup(ctx, ctx.entity_id_of(ent).unwrap());

            l = 0;
            while l < (MAX_SABERS) as i32 {
                let saber = match l {
                    0 => Some((*client).sess.saberType.clone()),
                    1 => Some((*client).sess.saber2Type.clone()),
                    _ => None,
                };
                let key = format!("saber{}", l + 1);
                let value = Info_ValueForKey(&userinfo, &key);

                if let Some(ref saber) = saber {
                    if !value.eq_ignore_ascii_case(saber) {
                        Info_SetValueForKey(&mut userinfo, &key, saber);
                        trap::SetUserinfo(ctx.engine, (*ent).s.number, &userinfo);
                    }
                }
                l += 1;
            }

            if (*client).saber[0].model[0] != 0 && (*client).saber[1].model[0] != 0 {
                // dual
                (*client).ps.fd.saberAnimLevelBase = SS_DUAL as c_int;
                (*client).ps.fd.saberAnimLevel = SS_DUAL as c_int;
                (*client).ps.fd.saberDrawAnimLevel = SS_DUAL as c_int;
            } else if (*client).saber[0].saberFlags & SFL_TWO_HANDED != 0 {
                // staff
                (*client).ps.fd.saberAnimLevel = SS_STAFF as c_int;
                (*client).ps.fd.saberDrawAnimLevel = SS_STAFF as c_int;
            } else {
                if (*client).sess.saberLevel < SS_FAST as c_int {
                    (*client).sess.saberLevel = SS_FAST as c_int;
                } else if (*client).sess.saberLevel > SS_STRONG as c_int {
                    (*client).sess.saberLevel = SS_STRONG as c_int;
                }
                (*client).ps.fd.saberAnimLevelBase = (*client).sess.saberLevel;
                (*client).ps.fd.saberAnimLevel = (*client).sess.saberLevel;
                (*client).ps.fd.saberDrawAnimLevel = (*client).sess.saberLevel;

                if ctx.world.cvars.g_gametype.integer != GT_SIEGE
                    && (*client).ps.fd.saberAnimLevel
                        > (*client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize]
                {
                    let lvl = (*client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize];
                    (*client).ps.fd.saberAnimLevelBase = lvl;
                    (*client).ps.fd.saberAnimLevel = lvl;
                    (*client).ps.fd.saberDrawAnimLevel = lvl;
                    (*client).sess.saberLevel = lvl;
                }
            }
            if ctx.world.cvars.g_gametype.integer != GT_SIEGE {
                // let's just make sure the styles we chose are cool
                if mp_bg::bg_saberLoad::WP_SaberStyleValidForSaber(
                    &mut (*client).saber[0],
                    &mut (*client).saber[1],
                    (*client).ps.saberHolstered,
                    (*client).ps.fd.saberAnimLevel,
                ) == qfalse
                {
                    mp_bg::bg_saberLoad::WP_UseFirstValidSaberStyle(
                        &mut (*client).saber[0],
                        &mut (*client).saber[1],
                        (*client).ps.saberHolstered,
                        &mut (*client).ps.fd.saberAnimLevel,
                    );
                    (*client).ps.fd.saberAnimLevelBase = (*client).ps.fd.saberAnimLevel;
                    (*client).saberCycleQueue = (*client).ps.fd.saberAnimLevel;
                }
            }
        }
        l = 0;

        if (*client).ps.fd.forceDoInit != 0 {
            // force a reread of force powers
            WP_InitForcePowers(ctx, ctx.entity_id_of(ent));
            (*client).ps.fd.forceDoInit = 0;
        }

        if (*client).ps.fd.saberAnimLevel != SS_STAFF as c_int
            && (*client).ps.fd.saberAnimLevel != SS_DUAL as c_int
            && (*client).ps.fd.saberAnimLevel == (*client).ps.fd.saberDrawAnimLevel
            && (*client).ps.fd.saberAnimLevel == (*client).sess.saberLevel
        {
            if (*client).sess.saberLevel < SS_FAST as c_int {
                (*client).sess.saberLevel = SS_FAST as c_int;
            } else if (*client).sess.saberLevel > SS_STRONG as c_int {
                (*client).sess.saberLevel = SS_STRONG as c_int;
            }
            (*client).ps.fd.saberAnimLevel = (*client).sess.saberLevel;
            (*client).ps.fd.saberDrawAnimLevel = (*client).sess.saberLevel;

            if ctx.world.cvars.g_gametype.integer != GT_SIEGE
                && (*client).ps.fd.saberAnimLevel
                    > (*client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize]
            {
                let lvl = (*client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize];
                (*client).ps.fd.saberAnimLevel = lvl;
                (*client).ps.fd.saberDrawAnimLevel = lvl;
                (*client).sess.saberLevel = lvl;
            }
        }

        // find a spawn point — do it before setting health back up, so farthest
        // ranging doesn't count this client
        let mut spawn_origin: vec3_t = [0.0; 3];
        let mut spawn_angles: vec3_t = [0.0; 3];
        let spawn_point: *mut gentity_t;
        if (*client).sess.sessionTeam == TEAM_SPECTATOR {
            spawn_point = SelectSpectatorSpawnPoint(ctx, &mut spawn_origin, &mut spawn_angles);
        } else if ctx.world.cvars.g_gametype.integer == GT_CTF
            || ctx.world.cvars.g_gametype.integer == GT_CTY
        {
            spawn_point = SelectCTFSpawnPoint(
                ctx,
                (*client).sess.sessionTeam,
                (*client).pers.teamState.state as c_int,
                &mut spawn_origin,
                &mut spawn_angles,
            );
        } else if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
            spawn_point = SelectSiegeSpawnPoint(
                ctx,
                (*client).siegeClass,
                (*client).sess.sessionTeam,
                (*client).pers.teamState.state as c_int,
                &mut spawn_origin,
                &mut spawn_angles,
            );
        } else {
            let mut sp;
            loop {
                if ctx.world.cvars.g_gametype.integer == GT_POWERDUEL {
                    sp = SelectDuelSpawnPoint(
                        ctx,
                        (*client).sess.duelTeam,
                        (*client).ps.origin,
                        &mut spawn_origin,
                        &mut spawn_angles,
                    );
                } else if ctx.world.cvars.g_gametype.integer == GT_DUEL {
                    sp = SelectDuelSpawnPoint(
                        ctx,
                        DUELTEAM_SINGLE as c_int,
                        (*client).ps.origin,
                        &mut spawn_origin,
                        &mut spawn_angles,
                    );
                } else {
                    // the first spawn should be at a good looking spot
                    if (*client).pers.initialSpawn == qfalse && (*client).pers.localClient != qfalse
                    {
                        (*client).pers.initialSpawn = qtrue;
                        sp = SelectInitialSpawnPoint(
                            ctx,
                            &mut spawn_origin,
                            &mut spawn_angles,
                            (*client).sess.sessionTeam,
                        );
                    } else {
                        // don't spawn near existing origin if possible
                        sp = SelectSpawnPoint(
                            ctx,
                            (*client).ps.origin,
                            &mut spawn_origin,
                            &mut spawn_angles,
                            (*client).sess.sessionTeam,
                        );
                    }
                }

                // Tim needs to prevent bots from spawning at the initial point on q3dm0...
                if (*sp).flags & FL_NO_BOTS != 0 && (*ent).r.svFlags & SVF_BOT != 0 {
                    continue; // try again
                }
                // just to be symmetric, we have a nohumans option...
                if (*sp).flags & FL_NO_HUMANS != 0 && (*ent).r.svFlags & SVF_BOT == 0 {
                    continue; // try again
                }

                break;
            }
            spawn_point = sp;
        }
        (*client).pers.teamState.state =
            crate::client::player_team_state::playerTeamStateState_t::TEAM_ACTIVE;

        // toggle the teleport bit so the client knows to not lerp and never clear the
        // voted flag
        let mut flags = (*client).ps.eFlags & EF_TELEPORT_BIT;
        flags ^= EF_TELEPORT_BIT;
        let game_flags = (*client).mGameFlags & (PSG_VOTED | PSG_TEAMVOTED) as u32;

        // clear everything but the persistant data
        // `pers` and `sess` own `String`s (netname; siege/saber/IP), so move
        // them out with `ptr::read` (the source slots are left bit-stale; the
        // `write_bytes` below zeroes them harmlessly and the `ptr::write`
        // restores avoid dropping that garbage).
        let saved = core::ptr::read(core::ptr::addr_of!((*client).pers));
        let saved_sess = core::ptr::read(core::ptr::addr_of!((*client).sess));
        // `modelname` is NOT in Raven's preserve set (the `memset` clears it to
        // ""); drop the old `String` here so the wholesale `write_bytes` below
        // doesn't leak it, then a valid empty `String` is installed after.
        drop(core::ptr::read(core::ptr::addr_of!((*client).modelname)));
        let saved_ping = (*client).ps.ping;
        let accuracy_hits = (*client).accuracy_hits;
        let accuracy_shots = (*client).accuracy_shots;
        let mut persistant: [c_int; MAX_PERSISTANT as usize] = [0; MAX_PERSISTANT as usize];
        let mut i = 0;
        while i < MAX_PERSISTANT {
            persistant[i as usize] = (*client).ps.persistant[i as usize];
            i += 1;
        }
        let event_sequence = (*client).ps.eventSequence;

        let saved_force = (*client).ps.fd;

        let save_saber_num = (*client).ps.saberEntityNum;

        let saved_siege_index = (*client).siegeClass;

        let mut saber_saved: [saberInfo_t; MAX_SABERS as usize] = (*client).saber;
        let mut g2_weapon_ptrs: [*mut c_void; MAX_SABERS as usize] = (*client).weaponGhoul2;

        i = 0;
        while i < (crate::entity::hit_location::HL_MAX) as usize {
            (*ent).locationDamage[i as usize] = 0;
            i += 1;
        }

        core::ptr::write_bytes(client, 0, 1);
        (*client).bodyGrabIndex = ENTITYNUM_NONE;

        // Get the skin RGB based on his userinfo
        let value = Info_ValueForKey(&userinfo, "char_color_red");
        // Raven's `value ? atoi(value) : 255` never takes the 255 arm
        // (Info_ValueForKey never returns NULL); empty -> atoi("") == 0.
        (*client).ps.customRGBA[0] = atoi(&value);
        let value = Info_ValueForKey(&userinfo, "char_color_green");
        (*client).ps.customRGBA[1] = atoi(&value);
        let value = Info_ValueForKey(&userinfo, "char_color_blue");
        (*client).ps.customRGBA[2] = atoi(&value);

        if ((*client).ps.customRGBA[0] as c_int
            + (*client).ps.customRGBA[1] as c_int
            + (*client).ps.customRGBA[2] as c_int)
            < 100
        {
            (*client).ps.customRGBA[0] = 255;
            (*client).ps.customRGBA[1] = 255;
            (*client).ps.customRGBA[2] = 255;
        }
        (*client).ps.customRGBA[3] = 255;

        (*client).siegeClass = saved_siege_index;

        (*client).saber = saber_saved;
        (*client).weaponGhoul2 = g2_weapon_ptrs;

        (*client).ps.saberEntityNum = save_saber_num;
        (*client).saberStoredIndex = save_saber_num;

        (*client).ps.fd = saved_force;

        (*client).ps.duelIndex = ENTITYNUM_NONE;

        // spawn with 100
        (*client).ps.jetpackFuel = 100;
        (*client).ps.cloakFuel = 100;

        // restore pers/sess with `ptr::write` — the current slots hold the
        // zeroed (invalid) `String`s from `write_bytes`, which must not be
        // dropped; `modelname` gets a fresh valid empty `String` (Raven's "").
        core::ptr::write(core::ptr::addr_of_mut!((*client).pers), saved);
        core::ptr::write(core::ptr::addr_of_mut!((*client).sess), saved_sess);
        core::ptr::write(core::ptr::addr_of_mut!((*client).modelname), String::new());
        (*client).ps.ping = saved_ping;
        (*client).accuracy_hits = accuracy_hits;
        (*client).accuracy_shots = accuracy_shots;
        (*client).lastkilled_client = -1;

        i = 0;
        while i < MAX_PERSISTANT {
            (*client).ps.persistant[i as usize] = persistant[i as usize];
            i += 1;
        }
        (*client).ps.eventSequence = event_sequence;
        // increment the spawncount so the client will detect the respawn
        (*client).ps.persistant[PERS_SPAWN_COUNT as usize] += 1;
        (*client).ps.persistant[PERS_TEAM as usize] = (*client).sess.sessionTeam as c_int;

        (*client).airOutTime = ctx.world.level.time + 12000;

        // set max health
        let max_health;
        if ctx.world.cvars.g_gametype.integer == GT_SIEGE && (*client).siegeClass != -1 {
            let scl = &(&ctx.world.bg_state.bgSiegeClasses)[(*client).siegeClass as usize];
            max_health = if scl.maxhealth != 0 {
                scl.maxhealth
            } else {
                100
            };
        } else {
            max_health = 100;
        }
        (*client).pers.maxHealth = max_health;
        if (*client).pers.maxHealth < 1 || (*client).pers.maxHealth > max_health {
            (*client).pers.maxHealth = 100;
        }
        (*client).ps.stats[STAT_MAX_HEALTH as usize] = (*client).pers.maxHealth;
        (*client).ps.eFlags = flags;
        (*client).mGameFlags = game_flags;

        (*ent).s.groundEntityNum = ENTITYNUM_NONE;
        (*ent).client = ctx.world.clients.as_mut_ptr().add(index as usize);
        (*ent).playerState = &mut (*((*ent).client)).ps;
        (*ent).takedamage = qtrue;
        (*ent).inuse = qtrue;
        (*ent).classname = b"player\0".as_ptr() as *mut c_char;
        (*ent).r.contents = CONTENTS_BODY;
        (*ent).clipmask = MASK_PLAYERSOLID;
        (*ent).die = Some(EntDie::player_die).into();
        (*ent).waterlevel = 0;
        (*ent).watertype = 0;
        (*ent).flags = 0;

        crate::q_math::_VectorCopy(playerMins, &mut (*ent).r.mins);
        crate::q_math::_VectorCopy(playerMaxs, &mut (*ent).r.maxs);
        (*client).ps.crouchheight = CROUCH_MAXS_2;
        (*client).ps.standheight = DEFAULT_MAXS_2;

        (*client).ps.clientNum = index;
        // give default weapons
        (*client).ps.stats[STAT_WEAPONS as usize] = 1 << WP_NONE;

        let w_disable = if ctx.world.cvars.g_gametype.integer == GT_DUEL
            || ctx.world.cvars.g_gametype.integer == GT_POWERDUEL
        {
            ctx.world.cvars.g_duelWeaponDisable.integer
        } else {
            ctx.world.cvars.g_weaponDisable.integer
        };

        let mut in_siege_with_class = qfalse;

        if ctx.world.cvars.g_gametype.integer != GT_HOLOCRON
            && ctx.world.cvars.g_gametype.integer != GT_JEDIMASTER
            && !HasSetSaberOnly(ctx)
            && AllForceDisabled(ctx.world.cvars.g_forcePowerDisable.integer) == qfalse
            && ctx.world.cvars.g_trueJedi.integer != 0
        {
            if ctx.world.cvars.g_gametype.integer >= GT_TEAM
                && ((*client).sess.sessionTeam == TEAM_BLUE
                    || (*client).sess.sessionTeam == TEAM_RED)
            {
                if ctx.world.level.numPlayingClients > 0 {
                    let mut force_team = TEAM_SPECTATOR;
                    let mut i = 0;
                    while i < ctx.world.level.maxclients {
                        let cl = &ctx.world.clients[i as usize];
                        if cl.pers.connected == CON_DISCONNECTED as _ {
                            i += 1;
                            continue;
                        }
                        if cl.sess.sessionTeam == TEAM_BLUE || cl.sess.sessionTeam == TEAM_RED {
                            if WP_HasForcePowers(&cl.ps) {
                                force_team = cl.sess.sessionTeam;
                            } else if cl.sess.sessionTeam == TEAM_BLUE {
                                force_team = TEAM_RED;
                            } else {
                                force_team = TEAM_BLUE;
                            }
                            break;
                        }
                        i += 1;
                    }
                    if WP_HasForcePowers(&(*client).ps) && (*client).sess.sessionTeam != force_team
                    {
                        let team_name = cstr_to_str(TeamName(force_team as c_int));
                        SetTeam(ctx, ctx.entity_id_of(ent).unwrap(), &team_name);
                        return;
                    }
                }
            }

            if WP_HasForcePowers(&(*client).ps) {
                (*client).ps.trueNonJedi = qfalse;
                (*client).ps.trueJedi = qtrue;
                (*client).ps.weapon = WP_SABER;
                (*client).ps.stats[STAT_WEAPONS as usize] = 1 << WP_SABER;
            } else {
                (*client).ps.trueNonJedi = qtrue;
                (*client).ps.trueJedi = qfalse;
                if w_disable == 0 || w_disable & (1 << WP_BRYAR_PISTOL) == 0 {
                    (*client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_BRYAR_PISTOL;
                }
                if w_disable == 0 || w_disable & (1 << WP_BLASTER) == 0 {
                    (*client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_BLASTER;
                }
                if w_disable == 0 || w_disable & (1 << WP_BOWCASTER) == 0 {
                    (*client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_BOWCASTER;
                }
                (*client).ps.stats[STAT_WEAPONS as usize] &= !(1 << WP_SABER);
                (*client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_MELEE;
                (*client).ps.ammo[AMMO_POWERCELL as usize] = ammoData[AMMO_POWERCELL as usize].max;
                (*client).ps.weapon = WP_BRYAR_PISTOL;
            }
        } else {
            // jediVmerc is incompatible with this gametype, turn it off!
            trap::Cvar_Set(ctx.engine, "g_jediVmerc", "0");
            if ctx.world.cvars.g_gametype.integer == GT_HOLOCRON {
                (*client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_SABER;
            } else if (*client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize] != 0 {
                (*client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_SABER;
            } else {
                (*client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_MELEE;
            }

            if ctx.world.cvars.g_gametype.integer != GT_SIEGE {
                if w_disable == 0 || w_disable & (1 << WP_BRYAR_PISTOL) == 0 {
                    (*client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_BRYAR_PISTOL;
                } else if ctx.world.cvars.g_gametype.integer == GT_JEDIMASTER {
                    (*client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_BRYAR_PISTOL;
                }
            }

            if ctx.world.cvars.g_gametype.integer == GT_JEDIMASTER {
                (*client).ps.stats[STAT_WEAPONS as usize] &= !(1 << WP_SABER);
                (*client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_MELEE;
            }

            if (*client).ps.stats[STAT_WEAPONS as usize] & (1 << WP_SABER) != 0 {
                (*client).ps.weapon = WP_SABER;
            } else if (*client).ps.stats[STAT_WEAPONS as usize] & (1 << WP_BRYAR_PISTOL) != 0 {
                (*client).ps.weapon = WP_BRYAR_PISTOL;
            } else {
                (*client).ps.weapon = WP_MELEE;
            }
        }

        if ctx.world.cvars.g_gametype.integer == GT_SIEGE
            && (*client).siegeClass != -1
            && (*client).sess.sessionTeam != TEAM_SPECTATOR
        {
            // well then, we will use a custom weaponset for our class
            let scl = &(&ctx.world.bg_state.bgSiegeClasses)[(*client).siegeClass as usize];
            (*client).ps.stats[STAT_WEAPONS as usize] = scl.weapons;

            if (*client).ps.stats[STAT_WEAPONS as usize] & (1 << WP_SABER) != 0 {
                (*client).ps.weapon = WP_SABER;
            } else if (*client).ps.stats[STAT_WEAPONS as usize] & (1 << WP_BRYAR_PISTOL) != 0 {
                (*client).ps.weapon = WP_BRYAR_PISTOL;
            } else {
                (*client).ps.weapon = WP_MELEE;
            }
            in_siege_with_class = qtrue;

            let mut m = 0;
            while m < WP_NUM_WEAPONS {
                if (*client).ps.stats[STAT_WEAPONS as usize] & (1 << m) != 0 {
                    if (*client).ps.weapon != WP_SABER && m > (*client).ps.weapon {
                        // try to find the highest ranking weapon we have
                        (*client).ps.weapon = m;
                    }

                    if m >= WP_BRYAR_PISTOL {
                        // Max his ammo out for all the weapons he has.
                        let scl =
                            &(&ctx.world.bg_state.bgSiegeClasses)[(*client).siegeClass as usize];
                        if ctx.world.cvars.g_gametype.integer == GT_SIEGE && m == WP_ROCKET_LAUNCHER
                        {
                            // don't give full ammo!
                            if scl.classflags & (1 << CFL_SINGLE_ROCKET as c_int) != 0 {
                                (*client).ps.ammo[weaponData[m as usize].ammoIndex as usize] = 1;
                            } else {
                                (*client).ps.ammo[weaponData[m as usize].ammoIndex as usize] = 10;
                            }
                        } else if ctx.world.cvars.g_gametype.integer == GT_SIEGE
                            && scl.classflags & (1 << CFL_EXTRA_AMMO as c_int) != 0
                        {
                            // double ammo
                            (*client).ps.ammo[weaponData[m as usize].ammoIndex as usize] =
                                ammoData[weaponData[m as usize].ammoIndex as usize].max * 2;
                            (*client).ps.eFlags |= EF_DOUBLE_AMMO;
                        } else {
                            (*client).ps.ammo[weaponData[m as usize].ammoIndex as usize] =
                                ammoData[weaponData[m as usize].ammoIndex as usize].max;
                        }
                    }
                }
                m += 1;
            }
        }

        if ctx.world.cvars.g_gametype.integer == GT_SIEGE
            && (*client).siegeClass != -1
            && (*client).sess.sessionTeam != TEAM_SPECTATOR
        {
            let scl = &(&ctx.world.bg_state.bgSiegeClasses)[(*client).siegeClass as usize];
            (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] = scl.invenItems;
            (*client).ps.stats[STAT_HOLDABLE_ITEM as usize] = 0;
        } else {
            (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] = 0;
            (*client).ps.stats[STAT_HOLDABLE_ITEM as usize] = 0;
        }

        if ctx.world.cvars.g_gametype.integer == GT_SIEGE
            && (*client).siegeClass != -1
            && (&ctx.world.bg_state.bgSiegeClasses)[(*client).siegeClass as usize].powerups != 0
            && (*client).sess.sessionTeam != TEAM_SPECTATOR
        {
            let scl = &(&ctx.world.bg_state.bgSiegeClasses)[(*client).siegeClass as usize];
            let mut i = 0;
            while i < PW_NUM_POWERUPS {
                if scl.powerups & (1 << i) != 0 {
                    (*client).ps.powerups[i as usize] = Q3_INFINITE;
                }
                i += 1;
            }
        }

        if (*client).sess.sessionTeam == TEAM_SPECTATOR {
            (*client).ps.stats[STAT_WEAPONS as usize] = 0;
            (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] = 0;
            (*client).ps.stats[STAT_HOLDABLE_ITEM as usize] = 0;
        }

        if in_siege_with_class == qfalse {
            (*client).ps.ammo[AMMO_BLASTER as usize] = 100;
        }

        (*client).ps.rocketLockIndex = ENTITYNUM_NONE;
        (*client).ps.rocketLockTime = (0) as f32;

        (*client).ps.genericEnemyIndex = -1;

        (*client).ps.isJediMaster = qfalse;

        if (*client).ps.fallingToDeath != 0 {
            (*client).ps.fallingToDeath = 0;
            (*client).noCorpse = qtrue;
        }

        // Do per-spawn force power initialization
        crate::w_force::WP_SpawnInitForcePowers(ctx, ctx.entity_id_of(ent).unwrap());

        // health will count down towards max_health
        if ctx.world.cvars.g_gametype.integer == GT_SIEGE
            && (*client).siegeClass != -1
            && (&ctx.world.bg_state.bgSiegeClasses)[(*client).siegeClass as usize].starthealth != 0
        {
            let h = (&ctx.world.bg_state.bgSiegeClasses)[(*client).siegeClass as usize].starthealth;
            (*ent).health = h;
            (*client).ps.stats[STAT_HEALTH as usize] = h;
        } else if ctx.world.cvars.g_gametype.integer == GT_DUEL
            || ctx.world.cvars.g_gametype.integer == GT_POWERDUEL
        {
            if ctx.world.cvars.g_gametype.integer == GT_POWERDUEL
                && (*client).sess.duelTeam == DUELTEAM_LONE as c_int
            {
                if ctx.world.cvars.g_duel_fraglimit.integer != 0 {
                    let h = (ctx.world.cvars.g_powerDuelStartHealth.integer as f32
                        - ((ctx.world.cvars.g_powerDuelStartHealth.integer
                            - ctx.world.cvars.g_powerDuelEndHealth.integer)
                            as f32
                            * (*client).sess.wins as f32
                            / ctx.world.cvars.g_duel_fraglimit.integer as f32))
                        as c_int;
                    (*ent).health = h;
                    (*client).ps.stats[STAT_HEALTH as usize] = h;
                    (*client).ps.stats[STAT_MAX_HEALTH as usize] = h;
                } else {
                    (*ent).health = 150;
                    (*client).ps.stats[STAT_HEALTH as usize] = 150;
                    (*client).ps.stats[STAT_MAX_HEALTH as usize] = 150;
                }
            } else {
                (*ent).health = 100;
                (*client).ps.stats[STAT_HEALTH as usize] = 100;
                (*client).ps.stats[STAT_MAX_HEALTH as usize] = 100;
            }
        } else if (*client).ps.stats[STAT_MAX_HEALTH as usize] <= 100 {
            let h = ((*client).ps.stats[STAT_MAX_HEALTH as usize] as f32 * 1.25) as c_int;
            (*ent).health = h;
            (*client).ps.stats[STAT_HEALTH as usize] = h;
        } else if (*client).ps.stats[STAT_MAX_HEALTH as usize] < 125 {
            (*ent).health = 125;
            (*client).ps.stats[STAT_HEALTH as usize] = 125;
        } else {
            let h = (*client).ps.stats[STAT_MAX_HEALTH as usize];
            (*ent).health = h;
            (*client).ps.stats[STAT_HEALTH as usize] = h;
        }

        // Start with a small amount of armor as well.
        if ctx.world.cvars.g_gametype.integer == GT_SIEGE && (*client).siegeClass != -1 {
            (*client).ps.stats[STAT_ARMOR as usize] =
                (&ctx.world.bg_state.bgSiegeClasses)[(*client).siegeClass as usize].startarmor;
        } else if ctx.world.cvars.g_gametype.integer == GT_DUEL
            || ctx.world.cvars.g_gametype.integer == GT_POWERDUEL
        {
            (*client).ps.stats[STAT_ARMOR as usize] = 0;
        } else {
            (*client).ps.stats[STAT_ARMOR as usize] =
                ((*client).ps.stats[STAT_MAX_HEALTH as usize] as f32 * 0.25) as c_int;
        }

        crate::g_utils::G_SetOrigin(&mut *(ent), spawn_origin);
        crate::q_math::_VectorCopy(spawn_origin, &mut (*client).ps.origin);

        // the respawned flag will be cleared after the attack and jump keys come up
        (*client).ps.pm_flags |= PMF_RESPAWNED;

        // Raven `client - level.clients` — pointer difference in gclient_t
        // units (ent_id would divide the byte offset by gentity_t's stride and
        // produce a wrong, potentially out-of-range clientNum).
        let client_num = (client as usize - ctx.world.clients.as_ptr() as usize)
            / core::mem::size_of::<gclient_t>();
        trap::GetUsercmd(
            ctx.engine,
            mp_abi::game::syscalls::G_GET_USERCMD::GGetUsercmdArgs::new(
                client_num as c_int,
                &mut (*client).pers.cmd,
            ),
        );
        SetClientViewAngle(&mut *ent, spawn_angles);

        if (*((*ent).client)).sess.sessionTeam == TEAM_SPECTATOR {
            // (nothing)
        } else {
            crate::g_utils::G_KillBox(ctx, ctx.entity_id_of(ent).unwrap());
            trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent.cast()));

            if (*client).ps.weapon <= WP_NONE {
                (*client).ps.weapon = WP_BRYAR_PISTOL;
            }

            (*client).ps.torsoTimer = 0;
            (*client).ps.legsTimer = 0;

            if (*client).ps.weapon == WP_SABER {
                G_SetAnim(
                    ctx,
                    ctx.entity_id_of(ent).unwrap(),
                    core::ptr::null_mut(),
                    SETANIM_BOTH,
                    (BOTH_STAND1TO2) as i32,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_HOLDLESS,
                    0,
                );
            } else {
                G_SetAnim(
                    ctx,
                    ctx.entity_id_of(ent).unwrap(),
                    core::ptr::null_mut(),
                    SETANIM_TORSO,
                    (TORSO_RAISEWEAP1) as i32,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_HOLDLESS,
                    0,
                );
                (*client).ps.legsAnim = WeaponReadyAnim[(*client).ps.weapon as usize];
            }
            (*client).ps.weaponstate = (WEAPON_RAISING) as i32;
            (*client).ps.weaponTime = (*client).ps.torsoTimer;
        }

        // don't allow full run speed for a bit
        (*client).ps.pm_flags |= PMF_TIME_KNOCKBACK;
        (*client).ps.pm_time = 100;

        (*client).respawnTime = ctx.world.level.time;
        (*client).inactivityTime =
            ctx.world.level.time + ctx.world.cvars.g_inactivity.integer * 1000;
        (*client).latched_buttons = 0;

        if ctx.world.level.intermissiontime != 0 {
            crate::g_main::MoveClientToIntermission(ctx, ctx.entity_id_of(ent).unwrap());
        } else {
            // fire the targets of the spawn point
            crate::g_utils::G_UseTargets(ctx, ctx.entity_id_of(spawn_point), ctx.entity_id_of(ent));
        }

        // set teams for NPCs to recognize
        if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
            // Imperial (team1) team is allied with "enemy" NPCs in this mode
            if (*client).sess.sessionTeam == SIEGETEAM_TEAM1 {
                (*client).playerTeam = NPCTEAM_ENEMY;
                (*ent).s.teamowner = NPCTEAM_ENEMY as c_int;
                (*client).enemyTeam = NPCTEAM_PLAYER;
            } else {
                (*client).playerTeam = NPCTEAM_PLAYER;
                (*ent).s.teamowner = NPCTEAM_PLAYER as c_int;
                (*client).enemyTeam = NPCTEAM_ENEMY;
            }
        } else {
            (*client).playerTeam = NPCTEAM_PLAYER;
            (*ent).s.teamowner = NPCTEAM_PLAYER as c_int;
            (*client).enemyTeam = NPCTEAM_ENEMY;
        }

        // run a client frame to drop exactly to the floor, initialize animations and
        // other things
        (*client).ps.commandTime = ctx.world.level.time - 100;
        (*((*ent).client)).pers.cmd.serverTime = ctx.world.level.time;
        crate::g_active::ClientThink(ctx, index, core::ptr::null_mut());

        // positively link the client, even if the command times are weird
        if (*((*ent).client)).sess.sessionTeam != TEAM_SPECTATOR {
            BG_PlayerStateToEntityState(&mut (*client).ps, &mut (*ent).s, qtrue);
            crate::q_math::_VectorCopy((*((*ent).client)).ps.origin, &mut (*ent).r.currentOrigin);
            trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent.cast()));
        }

        if ctx.world.cvars.g_spawnInvulnerability.integer != 0 {
            (*((*ent).client)).ps.eFlags |= EF_INVULNERABLE;
            (*((*ent).client)).invulnerableTimer =
                ctx.world.level.time + ctx.world.cvars.g_spawnInvulnerability.integer;
        }

        // run the presend to set anything else
        crate::g_active::ClientEndFrame(ctx, ctx.entity_id_of(ent).unwrap());

        // clear entity state values
        BG_PlayerStateToEntityState(&mut (*client).ps, &mut (*ent).s, qtrue);

        // rww - make sure client has a valid icarus instance
        trap::ICARUS_FreeEnt(
            ctx.engine,
            mp_abi::game::syscalls::G_ICARUS_FREEENT::GIcarusFreeentArgs::new(ent.cast()),
        );
        trap::ICARUS_InitEnt(
            ctx.engine,
            mp_abi::game::syscalls::G_ICARUS_INITENT::GIcarusInitentArgs::new(ent.cast()),
        );
    }
}

/// Raven `ClientDisconnect`.
///
/// Source: `oracle/codemp/game/g_client.c:3816-3938`
pub fn ClientDisconnect(ctx: &mut GameContext, clientNum: c_int) {
    use mp_abi::game::syscalls::G_G2_CLEANMODELS::GG2CleanmodelsArgs;
    use mp_abi::game::syscalls::G_G2_HAVEWEGHOULMODELS::GG2HaveweghoulmodelsArgs as GG2Haveweghoul2ModelsArgs;

    unsafe {
        // cleanup if we are kicking a bot that hasn't spawned yet
        G_RemoveQueuedBotBegin(ctx, clientNum);

        let ent = ctx.world.g_entities.as_mut_ptr().add(clientNum as usize);
        if (*ent).client.is_null() {
            return;
        }

        let mut i: c_int = 0;
        while i < (NUM_FORCE_POWERS) as i32 {
            if (*((*ent).client)).ps.fd.forcePowersActive & (1 << i) != 0 {
                WP_ForcePowerStop(ctx, ctx.entity_id_of(ent).unwrap(), i as forcePowers_t);
            }
            i += 1;
        }

        i = (TRACK_CHANNEL_1) as i32;
        while i < (NUM_TRACK_CHANNELS) as i32 {
            let idx = (i - 50) as usize;
            if (*((*ent).client)).ps.fd.killSoundEntIndex[idx] != 0
                && (*((*ent).client)).ps.fd.killSoundEntIndex[idx] < MAX_GENTITIES as c_int
                && (*((*ent).client)).ps.fd.killSoundEntIndex[idx] > 0
            {
                G_MuteSound(
                    ctx,
                    (*((*ent).client)).ps.fd.killSoundEntIndex[idx],
                    CHAN_VOICE,
                );
            }
            i += 1;
        }

        if (*((*ent).client)).ps.m_iVehicleNum != 0 {
            // tell it I'm getting off
            let veh = ctx
                .world
                .g_entities
                .as_mut_ptr()
                .add((*((*ent).client)).ps.m_iVehicleNum as usize);

            if (*veh).inuse != qfalse && !(*veh).client.is_null() && !(*veh).m_pVehicle.is_null() {
                let p_con = (*((*ent).client)).pers.connected;
                (*((*ent).client)).pers.connected = CON_DISCONNECTED as _;
                crate::veh_dispatch::eject(ctx, (*veh).m_pVehicle, ent as *mut bgEntity_t, qtrue);
                (*((*ent).client)).pers.connected = p_con;
            }
        }

        // stop any following clients
        i = 0;
        while i < ctx.world.level.maxclients {
            let cl = &ctx.world.clients[i as usize];
            if cl.sess.sessionTeam == TEAM_SPECTATOR
                && cl.sess.spectatorState
                    == crate::client::spectator_state::spectatorState_t::SPECTATOR_FOLLOW
                && cl.sess.spectatorClient == clientNum
            {
                StopFollowing(ctx, EntityId(i as u32));
            }
            i += 1;
        }

        // send effect if they were completely connected
        if (*((*ent).client)).pers.connected == CON_CONNECTED as _
            && (*((*ent).client)).sess.sessionTeam != TEAM_SPECTATOR
        {
            let tent_eid = G_TempEntity(
                ctx,
                (*((*ent).client)).ps.origin,
                (EV_PLAYER_TELEPORT_OUT) as i32,
            );
            let tent = ctx.entity_mut(tent_eid) as *mut gentity_t;
            (*tent).s.clientNum = (*ent).s.clientNum;

            // They don't get to take powerups with them!
            // Especially important for stuff like CTF flags
            TossClientItems(ctx, ctx.entity_id_of(ent).unwrap());
        }

        G_LogPrintf(ctx, &format!("ClientDisconnect: {}\n", clientNum));
        G_LogPrintf(
            ctx,
            &format!(
                "{} disconnected with IP: {}\n",
                (*((*ent).client)).pers.netname.clone(),
                (*((*ent).client)).sess.IPstring.clone(),
            ),
        );

        // if we are playing in tourney mode, give a win to the other player and clear his
        // frags for this round
        if ctx.world.cvars.g_gametype.integer == GT_DUEL
            && ctx.world.level.intermissiontime == 0
            && ctx.world.level.warmupTime == 0
        {
            if ctx.world.level.sortedClients[1] == clientNum {
                let idx0 = ctx.world.level.sortedClients[0] as usize;
                ctx.world.clients[idx0].ps.persistant[PERS_SCORE as usize] = 0;
                ctx.world.clients[idx0].sess.wins += 1;
                let sorted_client_0 = ctx.world.level.sortedClients[0];
                ClientUserinfoChanged(ctx, sorted_client_0);
            } else if ctx.world.level.sortedClients[0] == clientNum {
                let idx1 = ctx.world.level.sortedClients[1] as usize;
                ctx.world.clients[idx1].ps.persistant[PERS_SCORE as usize] = 0;
                ctx.world.clients[idx1].sess.wins += 1;
                let sorted_client_1 = ctx.world.level.sortedClients[1];
                ClientUserinfoChanged(ctx, sorted_client_1);
            }
        }

        if !(*ent).ghoul2.is_null()
            && trap::G2_HaveWeGhoul2Models(
                ctx.engine,
                GG2Haveweghoul2ModelsArgs::new((*ent).ghoul2),
            ) != qfalse
        {
            trap::G2API_CleanGhoul2Models(
                ctx.engine,
                GG2CleanmodelsArgs::new(&mut (*ent).ghoul2 as *mut *mut c_void),
            );
        }
        i = 0;
        while i < (MAX_SABERS) as i32 {
            let idx = i as usize;
            if !(*((*ent).client)).weaponGhoul2[idx].is_null()
                && trap::G2_HaveWeGhoul2Models(
                    ctx.engine,
                    GG2Haveweghoul2ModelsArgs::new((*((*ent).client)).weaponGhoul2[idx]),
                ) != qfalse
            {
                trap::G2API_CleanGhoul2Models(
                    ctx.engine,
                    GG2CleanmodelsArgs::new(
                        &mut (*((*ent).client)).weaponGhoul2[idx] as *mut *mut c_void,
                    ),
                );
            }
            i += 1;
        }

        trap::UnlinkEntity(ctx.engine, GUnlinkentityArgs::new(ent.cast()));
        (*ent).s.modelindex = 0;
        (*ent).inuse = qfalse;
        (*ent).classname = b"disconnected\0".as_ptr() as *mut c_char;
        (*((*ent).client)).pers.connected = CON_DISCONNECTED as _;
        (*((*ent).client)).ps.persistant[PERS_TEAM as usize] = TEAM_FREE as c_int;
        (*((*ent).client)).sess.sessionTeam = TEAM_FREE;
        (*ent).r.contents = 0;

        trap::SetConfigstring(ctx.engine, CS_PLAYERS + clientNum, "");

        CalculateRanks(ctx);

        if (*ent).r.svFlags & SVF_BOT != 0 {
            BotAIShutdownClient(ctx, clientNum, qfalse);
        }

        crate::g_log::G_ClearClientLog(ctx, clientNum);
    }
}

/// Raven `SetupGameGhoul2Model`.
///
/// There are two ghoul2 model instances per player (actually three). One is on
/// the clientinfo (the base for the client side player, and copied for player
/// spawns and for corpses). One is attached to the centity itself, which is the
/// model actually animated and rendered by the system. The final is the game
/// ghoul2 model. This is animated by pmove on the server, and is used for
/// determining where the lightsaber should be, and for per-poly collision tests.
///
/// Source: `oracle/codemp/game/g_client.c:1519-1861`
pub fn SetupGameGhoul2Model(
    ctx: &mut GameContext,
    ent: EntityId,
    modelname: *mut c_char,
    skinName: *mut c_char,
) {
    use mp_abi::game::syscalls::G_G2_ATTACHINSTANCETOENTNUM::GG2AttachinstancetoentnumArgs as GG2AttachInstanceToEntNumArgs;
    use mp_abi::game::syscalls::G_G2_CLEANMODELS::GG2CleanmodelsArgs;
    use mp_abi::game::syscalls::G_G2_COPYSPECIFICGHOUL2MODEL::GG2Copyspecificghoul2ModelArgs as GG2CopySpecificGhoul2ModelArgs;
    use mp_abi::game::syscalls::G_G2_DUPLICATEGHOUL2INSTANCE::GG2Duplicateghoul2InstanceArgs as GG2DuplicateGhoul2InstanceArgs;
    use mp_abi::game::syscalls::G_G2_HAVEWEGHOULMODELS::GG2HaveweghoulmodelsArgs as GG2Haveweghoul2ModelsArgs;
    use mp_abi::game::syscalls::G_G2_SETBOLTINFO::GG2SetboltinfoArgs as GG2SetBoltInfoArgs;
    use mp_abi::game::syscalls::G_G2_SETSKIN::GG2SetskinArgs as GG2SetSkinArgs;

    // STAGE-1: EntityId param (char* modelname/skinName stay raw), raw body
    // re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);
    unsafe {
        let mut handle: c_int = 0;
        let mut afilename: [c_char; 260] = [0; 260]; // MAX_QPATH
        let mut GLAName: [c_char; 260] = [0; 260];
        let mut tempVec: vec3_t = [0.0; 3];

        // First things first. If this is a ghoul2 model, then let's make sure we demolish this first.
        if !(*ent).ghoul2.is_null()
            && trap::G2_HaveWeGhoul2Models(
                ctx.engine,
                GG2Haveweghoul2ModelsArgs::new((*ent).ghoul2),
            ) != qfalse
        {
            trap::G2API_CleanGhoul2Models(
                ctx.engine,
                GG2CleanmodelsArgs::new(&mut (*ent).ghoul2 as *mut *mut c_void),
            );
        }

        // rww - just load the "standard" model for the server
        if ctx.world.globals.precachedKyle.is_null() {
            let mut defSkin: c_int = 0;

            write_cstr_field(&mut afilename, "models/players/kyle/model.glm");
            handle = trap::G2API_InitGhoul2Model(
                ctx.engine,
                &mut ctx.world.globals.precachedKyle as *mut *mut c_void,
                &cstr_to_str(afilename.as_ptr()),
                0,
                0,
                -20,
                0,
                0,
            );

            if handle < 0 {
                return;
            }

            defSkin =
                trap::R_RegisterSkin(ctx.engine, "models/players/kyle/model_default.skin");
            trap::G2API_SetSkin(
                ctx.engine,
                GG2SetSkinArgs::new(ctx.world.globals.precachedKyle, 0, defSkin, defSkin),
            );
        }

        if !ctx.world.globals.precachedKyle.is_null()
            && trap::G2_HaveWeGhoul2Models(
                ctx.engine,
                GG2Haveweghoul2ModelsArgs::new(ctx.world.globals.precachedKyle),
            ) != qfalse
        {
            if ctx.world.cvars.d_perPlayerGhoul2.integer != 0
                || (*ent).s.number >= (MAX_CLIENTS) as i32
                || G_PlayerHasCustomSkeleton(&*(ent)) != qfalse
            {
                // rww - allow option for perplayer models on server for collision and bolt stuff.
                let mut modelFullPath: [c_char; 260] = [0; 260];
                let mut truncModelName: [c_char; 260] = [0; 260];
                let mut skin: [c_char; 260] = [0; 260];
                let mut vehicleName: [c_char; 260] = [0; 260];
                let mut skinHandle: c_int = 0;
                let mut i: c_int = 0;
                let mut p: *mut c_char = core::ptr::null_mut();

                // If this is a vehicle, get its model name.
                if !(*ent).client.is_null() && (*((*ent).client)).NPC_class == CLASS_VEHICLE {
                    write_cstr_field(&mut vehicleName, &cstr_to_str(modelname));
                    let mut callbacks = crate::bg_channel::GameCallbacksImpl {
                        // SEAM-BG-REENTRY (DEC-28, sanctioned) — GameCallbacksImpl.world is a `*mut GameWorld`
                        // field aliasing bg_state; a raw store is required (bg-seam re-entry).
                        world: ctx.world_raw(),
                        engine: ctx.engine,
                    };
                    BG_GetVehicleModelName(
                        modelname,
                        &mut ctx.world.bg_state,
                        &crate::bg_channel::GameBgTraps::new(ctx.engine),
                        &mut callbacks,
                    );
                    write_cstr_field(&mut truncModelName, &cstr_to_str(modelname));
                    skin[0] = 0;
                    if !(*ent).m_pVehicle.is_null()
                        && !(*((*ent).m_pVehicle)).m_pVehicleInfo.is_null()
                        && !(*(*((*ent).m_pVehicle)).m_pVehicleInfo).skin.is_null()
                        && *(*(*((*ent).m_pVehicle)).m_pVehicleInfo).skin as c_int != 0
                    {
                        let skin_str = format!(
                            "models/players/{}/model_{}.skin",
                            cstr_to_str(modelname),
                            cstr_to_str((*(*((*ent).m_pVehicle)).m_pVehicleInfo).skin)
                        );
                        skinHandle = trap::R_RegisterSkin(ctx.engine, &skin_str);
                    } else {
                        let skin_str = format!(
                            "models/players/{}/model_default.skin",
                            cstr_to_str(modelname)
                        );
                        skinHandle = trap::R_RegisterSkin(ctx.engine, &skin_str);
                    }
                } else {
                    if !skinName.is_null() && *skinName as c_int != 0 {
                        write_cstr_field(&mut skin, &cstr_to_str(skinName));
                        write_cstr_field(&mut truncModelName, &cstr_to_str(modelname));
                    } else {
                        write_cstr_field(&mut skin, "default");
                        write_cstr_field(&mut truncModelName, &cstr_to_str(modelname));
                        p = crate::q_shared::Q_strrchr(truncModelName.as_ptr(), '/' as c_int);

                        if !p.is_null() {
                            *p = 0;
                            p = p.add(1);

                            while !p.is_null() && *p as c_int != 0 {
                                skin[i as usize] = *p;
                                i += 1;
                                p = p.add(1);
                            }
                            skin[i as usize] = 0;
                            i = 0;
                        }

                        if BG_IsValidCharacterModel(truncModelName.as_ptr(), skin.as_ptr())
                            == qfalse
                        {
                            write_cstr_field(&mut truncModelName, "kyle");
                            write_cstr_field(&mut skin, "default");
                        }

                        if ctx.world.cvars.g_gametype.integer >= GT_TEAM
                            && ctx.world.cvars.g_gametype.integer != GT_SIEGE
                            && ctx.world.cvars.g_trueJedi.integer == 0
                        {
                            BG_ValidateSkinForTeam(
                                truncModelName.as_ptr(),
                                skin.as_mut_ptr(),
                                (*((*ent).client)).sess.sessionTeam,
                                core::ptr::null_mut(),
                                &ctx.world.bg_state,
                                &crate::bg_channel::GameBgTraps::new(ctx.engine),
                            );
                        } else if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
                            // force skin for class if appropriate
                            if (*((*ent).client)).siegeClass != -1 {
                                let scl = &(&ctx.world.bg_state.bgSiegeClasses)
                                    [(*((*ent).client)).siegeClass as usize];
                                if !scl.forcedSkin.is_empty() {
                                    write_cstr_field(&mut skin, &scl.forcedSkin);
                                }
                            }
                        }
                    }
                }

                if skin[0] as c_int != 0 {
                    let useSkinName = if crate::q_shared::Q_strchr(skin.as_ptr(), '|' as c_int)
                        .is_null()
                        == false
                    {
                        // three part skin
                        format!(
                            "models/players/{}/|{}",
                            cstr_to_str(truncModelName.as_ptr()),
                            cstr_to_str(skin.as_ptr())
                        )
                    } else {
                        format!(
                            "models/players/{}/model_{}.skin",
                            cstr_to_str(truncModelName.as_ptr()),
                            cstr_to_str(skin.as_ptr())
                        )
                    };

                    skinHandle = trap::R_RegisterSkin(ctx.engine, &useSkinName);
                }

                write_cstr_field(
                    &mut modelFullPath,
                    &format!(
                        "models/players/{}/model.glm",
                        cstr_to_str(truncModelName.as_ptr())
                    ),
                );
                handle = trap::G2API_InitGhoul2Model(
                    ctx.engine,
                    &mut (*ent).ghoul2 as *mut *mut c_void,
                    &cstr_to_str(modelFullPath.as_ptr()),
                    0,
                    skinHandle,
                    -20,
                    0,
                    0,
                );

                if handle < 0 {
                    // Huh. Guess we don't have this model. Use the default.
                    if !(*ent).ghoul2.is_null()
                        && trap::G2_HaveWeGhoul2Models(
                            ctx.engine,
                            GG2Haveweghoul2ModelsArgs::new((*ent).ghoul2),
                        ) != qfalse
                    {
                        trap::G2API_CleanGhoul2Models(
                            ctx.engine,
                            GG2CleanmodelsArgs::new(&mut (*ent).ghoul2 as *mut *mut c_void),
                        );
                    }
                    (*ent).ghoul2 = core::ptr::null_mut();
                    trap::G2API_DuplicateGhoul2Instance(
                        ctx.engine,
                        GG2DuplicateGhoul2InstanceArgs::new(
                            ctx.world.globals.precachedKyle,
                            &mut (*ent).ghoul2 as *mut *mut c_void,
                        ),
                    );
                } else {
                    trap::G2API_SetSkin(
                        ctx.engine,
                        GG2SetSkinArgs::new((*ent).ghoul2, 0, skinHandle, skinHandle),
                    );

                    GLAName[0] = 0;
                    let glaName = trap::G2API_GetGLAName(ctx.engine, (*ent).ghoul2, 0, 260);
                        write_cstr_field(&mut GLAName, &glaName);

                    if GLAName[0] as c_int == 0
                        || crate::q_shared::Q_strstr(
                            GLAName.as_ptr(),
                            b"players/_humanoid/\0".as_ptr() as *const c_char,
                        )
                        .is_null()
                            && (*ent).s.number < (MAX_CLIENTS) as i32
                            && G_PlayerHasCustomSkeleton(&*(ent)) == qfalse
                    {
                        // a bad model
                        trap::G2API_CleanGhoul2Models(
                            ctx.engine,
                            GG2CleanmodelsArgs::new(&mut (*ent).ghoul2 as *mut *mut c_void),
                        );
                        (*ent).ghoul2 = core::ptr::null_mut();
                        trap::G2API_DuplicateGhoul2Instance(
                            ctx.engine,
                            GG2DuplicateGhoul2InstanceArgs::new(
                                ctx.world.globals.precachedKyle,
                                &mut (*ent).ghoul2 as *mut *mut c_void,
                            ),
                        );
                    }

                    if (*ent).s.number >= (MAX_CLIENTS) as i32 {
                        (*ent).s.modelGhoul2 = 1; // so we know to free it on the client when we're removed.

                        if skin[0] as c_int != 0 {
                            // append it after a *
                            let tail_str = format!(
                                "{}*{}",
                                cstr_to_str(modelFullPath.as_ptr()),
                                cstr_to_str(skin.as_ptr())
                            );
                            write_cstr_field(&mut modelFullPath, &tail_str);
                        }

                        if !(*ent).client.is_null() && (*((*ent).client)).NPC_class == CLASS_VEHICLE
                        {
                            // vehicles are tricky and send over their vehicle names as the model
                            (*ent).s.modelindex = G_ModelIndex(&cstr_to_str(vehicleName.as_ptr()));
                        } else {
                            (*ent).s.modelindex = G_ModelIndex(&cstr_to_str(modelFullPath.as_ptr()));
                        }
                    }
                }
            } else {
                trap::G2API_DuplicateGhoul2Instance(
                    ctx.engine,
                    GG2DuplicateGhoul2InstanceArgs::new(
                        ctx.world.globals.precachedKyle,
                        &mut (*ent).ghoul2 as *mut *mut c_void,
                    ),
                );
            }
        } else {
            return;
        }

        // Attach the instance to this entity num so we can make use of client-server
        // shared operations if possible.
        trap::G2API_AttachInstanceToEntNum(
            ctx.engine,
            GG2AttachInstanceToEntNumArgs::new((*ent).ghoul2, (*ent).s.number, qtrue),
        );

        // The model is now loaded.
        GLAName[0] = 0;

        if ctx.world.bg_state.BGPAFtextLoaded == qfalse {
            let humanoid_anims = ctx.world.bg_state.bgHumanoidAnimations.as_mut_ptr();
            if mp_bg::bg_panimate::BG_ParseAnimationFile(
                // STAGE-2b: irreducible — the ruling-21 `GameCallbacksImpl` seam
                // adapter holds a raw `*mut GameWorld`, so `bg_state` is read raw
                // to coexist with the `world:` field it fills in the same call.
                &mut (*ctx.world_raw()).bg_state,
                &crate::bg_channel::GameBgTraps::new(ctx.engine),
                &mut crate::bg_channel::GameCallbacksImpl {
                    world: ctx.world_raw(),
                    engine: ctx.engine,
                },
                cstr("models/players/_humanoid/animation.cfg").as_ptr(),
                humanoid_anims,
                qtrue,
            ) == -1
            {
                crate::g_main::Com_Printf("Failed to load humanoid animation file\n");
                return;
            }
        }

        if (*ent).s.number >= (MAX_CLIENTS) as i32 || G_PlayerHasCustomSkeleton(&*(ent)) != qfalse {
            (*ent).localAnimIndex = -1;

            GLAName[0] = 0;
            let glaName = trap::G2API_GetGLAName(ctx.engine, (*ent).ghoul2, 0, 260);
                write_cstr_field(&mut GLAName, &glaName);

            if GLAName[0] as c_int != 0
                && crate::q_shared::Q_strstr(
                    GLAName.as_ptr(),
                    b"players/_humanoid/\0".as_ptr() as *const c_char,
                )
                .is_null()
            {
                // it doesn't use humanoid anims.
                let slash = crate::q_shared::Q_strrchr(GLAName.as_ptr(), '/' as c_int);
                if !slash.is_null() {
                    // Raven: `strcpy(slash, "/animation.cfg")` — overwrites from
                    // the last '/' onward (e.g. `models/players/swoop/foo` ->
                    // `models/players/swoop/animation.cfg`), NOT the buffer start.
                    // Source: `oracle/codemp/game/g_client.c:1741`
                    let repl = b"/animation.cfg\0";
                    for (k, &c) in repl.iter().enumerate() {
                        *slash.add(k) = c as c_char;
                    }

                    (*ent).localAnimIndex = mp_bg::bg_panimate::BG_ParseAnimationFile(
                        // STAGE-2b: irreducible — the ruling-21 `GameCallbacksImpl` seam
                        // adapter holds a raw `*mut GameWorld`, so `bg_state` is read raw
                        // to coexist with the `world:` field it fills in the same call.
                        &mut (*ctx.world_raw()).bg_state,
                        &crate::bg_channel::GameBgTraps::new(ctx.engine),
                        &mut crate::bg_channel::GameCallbacksImpl {
                            world: ctx.world_raw(),
                            engine: ctx.engine,
                        },
                        GLAName.as_ptr(),
                        core::ptr::null_mut(),
                        qfalse,
                    );
                }
            } else {
                // humanoid index.
                if !crate::q_shared::Q_strstr(
                    GLAName.as_ptr(),
                    b"players/rockettrooper/\0".as_ptr() as *const c_char,
                )
                .is_null()
                {
                    (*ent).localAnimIndex = 1;
                } else {
                    (*ent).localAnimIndex = 0;
                }
            }

            if (*ent).localAnimIndex == -1 {
                crate::g_main::Com_Error(
                    (ERR_DROP) as i32,
                    cstr("NPC had an invalid GLA\n").as_ptr(),
                );
            }
        } else {
            GLAName[0] = 0;
            let glaName = trap::G2API_GetGLAName(ctx.engine, (*ent).ghoul2, 0, 260);
                write_cstr_field(&mut GLAName, &glaName);

            if !crate::q_shared::Q_strstr(
                GLAName.as_ptr(),
                b"players/rockettrooper/\0".as_ptr() as *const c_char,
            )
            .is_null()
            {
                // assert(!"Should not have gotten in here with rockettrooper skel");
                (*ent).localAnimIndex = 1;
            } else {
                (*ent).localAnimIndex = 0;
            }
        }

        if (*ent).s.NPC_class == (CLASS_VEHICLE) as i32 && !(*ent).m_pVehicle.is_null() {
            // do special vehicle stuff
            let mut strTemp: [c_char; 128] = [0; 128];
            let mut i: c_int = 0;

            // Setup the default first bolt
            i = trap::G2API_AddBolt(ctx.engine, (*ent).ghoul2, 0, "model_root");

            // Setup the droid unit.
            (*((*ent).m_pVehicle)).m_iDroidUnitTag = trap::G2API_AddBolt(ctx.engine, (*ent).ghoul2, 0, "*droidunit");

            // Setup the Exhausts.
            i = 0;
            while i < (MAX_VEHICLE_EXHAUSTS) as i32 {
                write_cstr_field(&mut strTemp, &format!("*exhaust{}", i + 1));
                (*((*ent).m_pVehicle)).m_iExhaustTag[i as usize] = trap::G2API_AddBolt(ctx.engine, (*ent).ghoul2, 0, &cstr_to_str(strTemp.as_ptr()));
                i += 1;
            }

            // Setup the Muzzles.
            i = 0;
            while i < (MAX_VEHICLE_MUZZLES) as i32 {
                write_cstr_field(&mut strTemp, &format!("*muzzle{}", i + 1));
                (*((*ent).m_pVehicle)).m_iMuzzleTag[i as usize] = trap::G2API_AddBolt(ctx.engine, (*ent).ghoul2, 0, &cstr_to_str(strTemp.as_ptr()));
                if (*((*ent).m_pVehicle)).m_iMuzzleTag[i as usize] == -1 {
                    // ergh, try *flash?
                    write_cstr_field(&mut strTemp, &format!("*flash{}", i + 1));
                    (*((*ent).m_pVehicle)).m_iMuzzleTag[i as usize] = trap::G2API_AddBolt(ctx.engine, (*ent).ghoul2, 0, &cstr_to_str(strTemp.as_ptr()));
                }
                i += 1;
            }

            // Setup the Turrets.
            i = 0;
            while i < (MAX_VEHICLE_TURRET_MUZZLES) as i32 {
                if !(*(*((*ent).m_pVehicle)).m_pVehicleInfo).turret[i as usize]
                    .gunnerViewTag
                    .is_null()
                {
                    (*((*ent).m_pVehicle)).m_iGunnerViewTag[i as usize] = trap::G2API_AddBolt(
                        ctx.engine,
                        (*ent).ghoul2,
                        0,
                        &cstr_to_str(
                            (*(*((*ent).m_pVehicle)).m_pVehicleInfo).turret[i as usize]
                                .gunnerViewTag,
                        ),
                    );
                } else {
                    (*((*ent).m_pVehicle)).m_iGunnerViewTag[i as usize] = -1;
                }
                i += 1;
            }
        }

        if !(*ent).client.is_null()
            && ((*((*ent).client)).ps.weapon == WP_SABER || (*ent).s.number < (MAX_CLIENTS) as i32)
        {
            // a player or NPC saber user
            trap::G2API_AddBolt(ctx.engine, (*ent).ghoul2, 0, "*r_hand");
            trap::G2API_AddBolt(ctx.engine, (*ent).ghoul2, 0, "*l_hand");

            // rhand must always be first bolt. lhand always second. Whichever you want the
            // jetpack bolted to must always be third.
            trap::G2API_AddBolt(ctx.engine, (*ent).ghoul2, 0, "*chestg");

            // claw bolts
            trap::G2API_AddBolt(ctx.engine, (*ent).ghoul2, 0, "*r_hand_cap_r_arm");
            trap::G2API_AddBolt(ctx.engine, (*ent).ghoul2, 0, "*l_hand_cap_l_arm");

            trap::G2API_SetBoneAnim(
                ctx.engine,
                (*ent).ghoul2,
                0,
                "model_root",
                0,
                12,
                BONE_ANIM_OVERRIDE_LOOP,
                1.0f32,
                ctx.world.level.time,
                -1.0,
                -1,
            );
            trap::G2API_SetBoneAngles(
                ctx.engine,
                (*ent).ghoul2,
                0,
                "upper_lumbar",
                &tempVec as *const vec3_t,
                BONE_ANGLES_POSTMULT,
                POSITIVE_X as c_int,
                NEGATIVE_Y as c_int,
                NEGATIVE_Z as c_int,
                core::ptr::null_mut(),
                0,
                ctx.world.level.time,
            );
            trap::G2API_SetBoneAngles(
                ctx.engine,
                (*ent).ghoul2,
                0,
                "cranium",
                &tempVec as *const vec3_t,
                BONE_ANGLES_POSTMULT,
                POSITIVE_Z as c_int,
                NEGATIVE_Y as c_int,
                POSITIVE_X as c_int,
                core::ptr::null_mut(),
                0,
                ctx.world.level.time,
            );

            if ctx.world.globals.g2SaberInstance.is_null() {
                trap::G2API_InitGhoul2Model(
                    ctx.engine,
                    &mut ctx.world.globals.g2SaberInstance as *mut *mut c_void,
                    "models/weapons2/saber/saber_w.glm",
                    0,
                    0,
                    -20,
                    0,
                    0,
                );

                if !ctx.world.globals.g2SaberInstance.is_null() {
                    // indicate we will be bolted to model 0 (ie the player) on bolt 0
                    // (always the right hand) when we get copied
                    trap::G2API_SetBoltInfo(
                        ctx.engine,
                        GG2SetBoltInfoArgs::new(ctx.world.globals.g2SaberInstance, 0, 0),
                    );
                    // now set up the gun bolt on it
                    trap::G2API_AddBolt(ctx.engine, ctx.world.globals.g2SaberInstance, 0, "*blade1");
                }
            }

            if G_SaberModelSetup(ctx, ctx.entity_id_of(ent).unwrap()) != qfalse {
                if !ctx.world.globals.g2SaberInstance.is_null() {
                    trap::G2API_CopySpecificGhoul2Model(
                        ctx.engine,
                        GG2CopySpecificGhoul2ModelArgs::new(
                            ctx.world.globals.g2SaberInstance,
                            0,
                            (*ent).ghoul2,
                            1,
                        ),
                    );
                }
            }
        }

        if (*ent).s.number >= (MAX_CLIENTS) as i32 {
            // some extra NPC stuff
            if trap::G2API_AddBolt(ctx.engine, (*ent).ghoul2, 0, "lower_lumbar") == -1
            {
                // check now to see if we have this bone for setting anims and such
                (*ent).noLumbar = qtrue;
            }
        }
    }
}

/// Raven `ClientUserinfoChanged`.
///
/// Source: `oracle/codemp/game/g_client.c:1888-2235`
pub fn ClientUserinfoChanged(ctx: &mut GameContext, clientNum: c_int) {
    unsafe {
        let ent: *mut gentity_t = ctx.world.g_entities.as_mut_ptr().add(clientNum as usize);
        let client: *mut gclient_t = (*ent).client;

        let mut teamTask: c_int = 0;
        let mut teamLeader: c_int = 0;
        let mut team: c_int = 0;
        let mut health: c_int = 0;

        let mut model: [c_char; 260] = [0; 260];
        let mut forcePowers: [c_char; 260] = [0; 260];
        let oldname: String;
        // `MAX_INFO_STRING` is 1024; these must match so long userinfo strings
        // (and their color keys) are not truncated before parsing.
        let mut c1: [c_char; 1024] = [0; 1024];
        let mut c2: [c_char; 1024] = [0; 1024];
        let mut className = String::new();
        let mut saberName: [c_char; 260] = [0; 260];
        let mut saber2Name: [c_char; 260] = [0; 260];
        let mut maxHealth: c_int = 0;
        let mut modelChanged: qboolean = qfalse;

        let mut userinfo = trap::GetUserinfo(ctx.engine, clientNum, 1024);

        // check for malformed or illegal info strings
        if !Info_Validate(&userinfo) {
            userinfo = "\\name\\badinfo".to_string();
        }

        // check for local client
        let s = Info_ValueForKey(&userinfo, "ip");
        if s == "localhost" {
            (*client).pers.localClient = qtrue;
        }

        // check the item prediction
        let s = Info_ValueForKey(&userinfo, "cg_predictItems");
        if atoi(&s) == 0 {
            (*client).pers.predictItemPickup = qfalse;
        } else {
            (*client).pers.predictItemPickup = qtrue;
        }

        // set name
        // Raven `Q_strncpyz(oldname, netname, sizeof(oldname))` (1024): `netname`
        // is ≤ MAX_NETNAME bytes, so a clone is byte-identical to the truncation.
        oldname = (*client).pers.netname.clone();
        let s = Info_ValueForKey(&userinfo, "name");
        (*client).pers.netname = ClientCleanName(ctx, &s, MAX_NETNAME as c_int);

        if (*client).sess.sessionTeam == TEAM_SPECTATOR {
            if (*client).sess.spectatorState
                == crate::client::spectator_state::spectatorState_t::SPECTATOR_SCOREBOARD
            {
                (*client).pers.netname = strncpyz_string(b"scoreboard", MAX_NETNAME);
            }
        }

        if (*client).pers.connected == CON_CONNECTED {
            // Raven `strcmp(oldname, netname)` — case-sensitive byte compare.
            if oldname != (*client).pers.netname {
                if (*client).pers.netnameTime > ctx.world.level.time {
                    let msg = format!(
                        "print \"{}\n\"",
                        G_GetStringEdString(ctx, "MP_SVGAME", "NONAMECHANGE")
                    );
                    trap::SendServerCommand(ctx.engine, clientNum, &msg);

                    Info_SetValueForKey(&mut userinfo, "name", &oldname);
                    trap::SetUserinfo(ctx.engine, clientNum, &userinfo);
                    (*client).pers.netname = strncpyz_string(oldname.as_bytes(), MAX_NETNAME);
                } else {
                    let msg = format!(
                        "print \"{}{} {} {}\n\"",
                        oldname,
                        S_COLOR_WHITE,
                        G_GetStringEdString(ctx, "MP_SVGAME", "PLRENAME"),
                        (*client).pers.netname
                    );
                    trap::SendServerCommand(ctx.engine, -1, &msg);
                    (*client).pers.netnameTime = ctx.world.level.time + 5000;
                }
            }
        }

        // set model
        let modelname_kv = Info_ValueForKey(&userinfo, "model");
        crate::q_shared::Q_strncpyz(model.as_mut_ptr(), cstr(&modelname_kv).as_ptr(), 260);

        if ctx.world.cvars.d_perPlayerGhoul2.integer != 0 {
            // `modelname` is a `String`; `model` stays a C buffer (feeds pointer
            // sinks), so compare via the &str `Q_stricmp` (ASCII case-fold) and
            // keep the field's `MAX_QPATH - 1` byte write bound.
            let model_str = cstr_to_str(model.as_ptr());
            if Q_stricmp(&model_str, &(*client).modelname) != 0 {
                (*client).modelname = strncpyz_string(model_str.as_bytes(), MAX_QPATH);
                modelChanged = qtrue;
            }
        }

        // Get the skin RGB based on his userinfo
        // Raven's `if (value)` arms never take the 255 branch
        // (Info_ValueForKey never returns NULL); empty -> atoi("") == 0.
        let value = Info_ValueForKey(&userinfo, "char_color_red");
        (*client).ps.customRGBA[0] = atoi(&value) as c_int;

        let value = Info_ValueForKey(&userinfo, "char_color_green");
        (*client).ps.customRGBA[1] = atoi(&value) as c_int;

        let value = Info_ValueForKey(&userinfo, "char_color_blue");
        (*client).ps.customRGBA[2] = atoi(&value) as c_int;

        if ((*client).ps.customRGBA[0] + (*client).ps.customRGBA[1] + (*client).ps.customRGBA[2])
            < 100
        {
            // hmm, too dark!
            (*client).ps.customRGBA[0] = 255;
            (*client).ps.customRGBA[1] = 255;
            (*client).ps.customRGBA[2] = 255;
        }

        (*client).ps.customRGBA[3] = 255;

        let forcepowers_kv = Info_ValueForKey(&userinfo, "forcepowers");
        crate::q_shared::Q_strncpyz(forcePowers.as_mut_ptr(), cstr(&forcepowers_kv).as_ptr(), 260);

        // bots set their team a few frames later
        if ctx.world.cvars.g_gametype.integer >= GT_TEAM
            && ctx.world.g_entities[clientNum as usize].r.svFlags & SVF_BOT != 0
        {
            let s = Info_ValueForKey(&userinfo, "team");
            if Q_stricmp(&s, "red") == 0 || Q_stricmp(&s, "r") == 0 {
                team = TEAM_RED;
            } else if Q_stricmp(&s, "blue") == 0 || Q_stricmp(&s, "b") == 0 {
                team = TEAM_BLUE;
            } else {
                // pick the team with the least number of players
                team = PickTeam(ctx, clientNum) as c_int;
            }
        } else {
            team = (*client).sess.sessionTeam;
        }

        // Set the siege class
        if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
            className = strncpyz_string((*client).sess.siegeClass.as_bytes(), 260);

            // This function will see if the given class is legal for the given team.
            // If not className will be filled in with the first legal class for this team.
            // Now that the team is legal for sure, we'll go ahead and get an index for it.
            (*client).siegeClass = BG_SiegeFindClassIndexByName(&className, &ctx.world.bg_state);
            if (*client).siegeClass == -1 {
                // ok, get the first valid class for the team you're on then, I guess.
                BG_SiegeCheckClassLegality(team, &mut className, &ctx.world.bg_state);
                (*client).sess.siegeClass = strncpyz_string(className.as_bytes(), 64);
                (*client).siegeClass = BG_SiegeFindClassIndexByName(&className, &ctx.world.bg_state);
            } else {
                // otherwise, make sure the class we are using is legal.
                G_ValidateSiegeClassForTeam(ctx, ctx.entity_id_of(ent).unwrap(), team);
                className = strncpyz_string((*client).sess.siegeClass.as_bytes(), 260);
            }

            // Set the sabers if the class dictates
            if (*client).siegeClass != -1 {
                // STAGE-2b: irreducible — `scl` is a raw view into world state held
                // live across the G_SetSaber/G_SaberModelSetup calls that take `&mut
                // ctx`, so it stays raw-derived rather than a tracked borrow.
                let scl =
                    &(&(*ctx.world_raw()).bg_state.bgSiegeClasses)[(*client).siegeClass as usize];

                if !scl.saber1.is_empty() {
                    G_SetSaber(ctx, ctx.entity_id_of(ent).unwrap(), 0, &scl.saber1, qtrue);
                } else {
                    // default I guess
                    G_SetSaber(ctx, ctx.entity_id_of(ent).unwrap(), 0, "Kyle", qtrue);
                }
                if !scl.saber2.is_empty() {
                    G_SetSaber(ctx, ctx.entity_id_of(ent).unwrap(), 1, &scl.saber2, qtrue);
                } else {
                    // no second saber then
                    G_SetSaber(ctx, ctx.entity_id_of(ent).unwrap(), 1, "none", qtrue);
                }

                // make sure the saber models are updated
                G_SaberModelSetup(ctx, ctx.entity_id_of(ent).unwrap());

                if !scl.forcedModel.is_empty() {
                    // be sure to override the model we actually use
                    write_cstr_field(&mut model, &scl.forcedModel);
                    if ctx.world.cvars.d_perPlayerGhoul2.integer != 0 {
                        let model_str = cstr_to_str(model.as_ptr());
                        if Q_stricmp(&model_str, &(*client).modelname) != 0 {
                            (*client).modelname = strncpyz_string(model_str.as_bytes(), MAX_QPATH);
                            modelChanged = qtrue;
                        }
                    }
                }

                // force them to use their class model on the server, if the class dictates
                if G_PlayerHasCustomSkeleton(&*(ent)) != qfalse {
                    let model_str = cstr_to_str(model.as_ptr());
                    if Q_stricmp(&model_str, &(*client).modelname) != 0
                        || (*ent).localAnimIndex == 0
                    {
                        (*client).modelname = strncpyz_string(model_str.as_bytes(), MAX_QPATH);
                        modelChanged = qtrue;
                    }
                }
            }
        } else {
            className = "none".to_string();
        }

        // Set the saber name
        write_cstr_field(&mut saberName, &(*client).sess.saberType);
        write_cstr_field(&mut saber2Name, &(*client).sess.saber2Type);

        // set max health
        if ctx.world.cvars.g_gametype.integer == GT_SIEGE && (*client).siegeClass != -1 {
            let scl = &(&ctx.world.bg_state.bgSiegeClasses)[(*client).siegeClass as usize];
            maxHealth = 100;

            if scl.maxhealth != 0 {
                maxHealth = scl.maxhealth;
            }

            health = maxHealth;
        } else {
            maxHealth = 100;
            health = 100;
        }
        (*client).pers.maxHealth = health;
        if (*client).pers.maxHealth < 1 || (*client).pers.maxHealth > maxHealth {
            (*client).pers.maxHealth = 100;
        }
        (*client).ps.stats[STAT_MAX_HEALTH as usize] = (*client).pers.maxHealth;

        if ctx.world.cvars.g_gametype.integer >= GT_TEAM {
            (*client).pers.teamInfo = qtrue;
        } else {
            let s = Info_ValueForKey(&userinfo, "teamoverlay");
            if s.is_empty() || atoi(&s) != 0 {
                (*client).pers.teamInfo = qtrue;
            } else {
                (*client).pers.teamInfo = qfalse;
            }
        }

        // team task (0 = none, 1 = offence, 2 = defence)
        teamTask = atoi(&Info_ValueForKey(&userinfo, "teamtask")) as c_int;
        // team Leader (1 = leader, 0 is normal player)
        teamLeader = (*client).sess.teamLeader as c_int;

        // colors
        write_cstr_field(&mut c1, &Info_ValueForKey(&userinfo, "color1"));
        write_cstr_field(&mut c2, &Info_ValueForKey(&userinfo, "color2"));

        // send over a subset of the userinfo keys so other clients can
        // print scoreboards, display models, and play custom sounds
        let configstring_s = if (*ent).r.svFlags & SVF_BOT != 0 {
            format!(
                "n\\{}\\t\\{}\\model\\{}\\c1\\{}\\c2\\{}\\hc\\{}\\w\\{}\\l\\{}\\skill\\{}\\tt\\{}\\tl\\{}\\siegeclass\\{}\\st\\{}\\st2\\{}\\dt\\{}\\sdt\\{}",
                (*client).pers.netname.clone(),
                team,
                cstr_to_str(model.as_ptr()),
                cstr_to_str(c1.as_ptr()),
                cstr_to_str(c2.as_ptr()),
                (*client).pers.maxHealth,
                (*client).sess.wins,
                (*client).sess.losses,
                Info_ValueForKey(&userinfo, "skill"),
                teamTask,
                teamLeader,
                className,
                cstr_to_str(saberName.as_ptr()),
                cstr_to_str(saber2Name.as_ptr()),
                (*client).sess.duelTeam,
                (*client).sess.siegeDesiredTeam
            )
        } else if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
            // more crap to send
            format!(
                "n\\{}\\t\\{}\\model\\{}\\c1\\{}\\c2\\{}\\hc\\{}\\w\\{}\\l\\{}\\tt\\{}\\tl\\{}\\siegeclass\\{}\\st\\{}\\st2\\{}\\dt\\{}\\sdt\\{}",
                (*client).pers.netname.clone(),
                (*client).sess.sessionTeam,
                cstr_to_str(model.as_ptr()),
                cstr_to_str(c1.as_ptr()),
                cstr_to_str(c2.as_ptr()),
                (*client).pers.maxHealth,
                (*client).sess.wins,
                (*client).sess.losses,
                teamTask,
                teamLeader,
                className,
                cstr_to_str(saberName.as_ptr()),
                cstr_to_str(saber2Name.as_ptr()),
                (*client).sess.duelTeam,
                (*client).sess.siegeDesiredTeam
            )
        } else {
            format!(
                "n\\{}\\t\\{}\\model\\{}\\c1\\{}\\c2\\{}\\hc\\{}\\w\\{}\\l\\{}\\tt\\{}\\tl\\{}\\st\\{}\\st2\\{}\\dt\\{}",
                (*client).pers.netname.clone(),
                (*client).sess.sessionTeam,
                cstr_to_str(model.as_ptr()),
                cstr_to_str(c1.as_ptr()),
                cstr_to_str(c2.as_ptr()),
                (*client).pers.maxHealth,
                (*client).sess.wins,
                (*client).sess.losses,
                teamTask,
                teamLeader,
                cstr_to_str(saberName.as_ptr()),
                cstr_to_str(saber2Name.as_ptr()),
                (*client).sess.duelTeam
            )
        };

        trap::SetConfigstring(ctx.engine, CS_PLAYERS + clientNum, &configstring_s);

        if modelChanged != qfalse {
            // only going to be true for allowable server-side custom skeleton cases
            // update the server g2 instance if appropriate
            let modelname_kv2 = Info_ValueForKey(&userinfo, "model");
            SetupGameGhoul2Model(
                ctx,
                ctx.entity_id_of(ent).unwrap(),
                cstr(&modelname_kv2).as_ptr() as *mut c_char,
                core::ptr::null_mut(),
            );

            if !(*ent).ghoul2.is_null() && !(*ent).client.is_null() {
                (*((*ent).client)).renderInfo.lastG2 = core::ptr::null_mut();
                // update the renderinfo bolts next update.
            }

            (*client).torsoAnimExecute = -1;
            (*client).legsAnimExecute = -1;
            (*client).torsoLastFlip = qfalse;
            (*client).legsLastFlip = qfalse;
        }

        if ctx.world.cvars.g_logClientInfo.integer != 0 {
            G_LogPrintf(
                ctx,
                &format!(
                    "ClientUserinfoChanged: {} {}\n",
                    clientNum,
                    cstr_to_str(configstring_s.as_ptr() as *const c_char)
                ),
            );
        }
    }
}

/// Raven `SetClientViewAngle`. No `level`/cvar/trap access, so no `GameContext`
/// (unlike most of this file's spine).
///
/// Source: `oracle/codemp/game/g_client.c:1109-1125`
pub fn SetClientViewAngle(ent: &mut gentity_t, angle: vec3_t) {
    // `ent.client` is a raw `gclient_t` pointer (pool or level slot); deref it raw
    // through a copied pointer value exactly as Raven does (recipe 2b).
    let client = ent.client;
    // Raven `ANGLE2SHORT(x)` == `((int)((x)*65536/360) & 65535)`.
    for i in 0..3 {
        let cmd_angle = ((angle[i] * 65536.0 / 360.0) as c_int) & 65535;
        unsafe {
            (*client).ps.delta_angles[i] = cmd_angle - (*client).pers.cmd.angles[i] as c_int;
        }
    }
    ent.s.angles = angle;
    unsafe {
        (*client).ps.viewangles = ent.s.angles;
    }
}
