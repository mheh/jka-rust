// PORT-COMPLETE: g_misc.c
//! FAITHFUL port of `oracle/codemp/game/g_misc.c`.
//!
//! Filled by the jampgame mega-pass.
#![allow(non_snake_case, unused, clippy::all)]


use crate::prelude::*;
use crate::g_utils::G_BSPIndex;
use native_string::atof_bytes;
use native_string::atoi_bytes;

use crate::ent_fn_enums::{EntDie, EntThink, EntTouch, EntUse};
use crate::g_client::SetClientViewAngle;
use crate::g_combat::AddScore;
use crate::g_exphysics::G_RunExPhys;
use crate::g_local_consts::{START_TIME_FIND_LINKS, START_TIME_LINK_ENTS};
use crate::g_main::{Com_Printf, G_Printf, LogExit};
use crate::g_mover::G_FindDoorTrigger;
use crate::g_object::G_RunObject;
use crate::g_spawn::{G_SpawnFloat, G_SpawnInt};
use crate::g_utils::{
    vtos, G_AddEvent, G_EffectIndex, G_EntitySound, G_Find, G_FreeEntity, G_IconIndex, G_KillBox,
    G_ModelIndex, G_PickTarget, G_ScreenShake, G_SetAngles, G_SetMovedir, G_SetOrigin, G_Sound,
    G_SoundIndex, G_SoundSetIndex, G_Spawn, G_TempEntity, G_UseTargets, G_UseTargets2,
};
use crate::level::reference_tag::MAX_REFNAME;
use crate::level::tag_owner::{MAX_TAGS, MAX_TAG_OWNERS, TAG_GENERIC_NAME, TAG_GENERIC_NAME_C};
use crate::q_math::vec3_origin;
use crate::q_math::{DirToByte, PerpendicularVector, VectorNormalize};
use crate::q_shared::{GetIDForString, Info_SetValueForKey, Q_strlwr};
use native_string::Q_stricmp;
use native_string::strncpyz_string;
use crate::trap;
use crate::NPC_utils::G_ActivateBehavior;
use mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs;
use mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use mp_bg::public::configstring::{CS_SKYBOXORG, CS_TERRAINS};
use mp_bg::public::entity_type::entityType_t;
use mp_bg::public::fx_state::{
    FX_STATE_CONTINUOUS, FX_STATE_OFF, FX_STATE_ONE_SHOT, FX_STATE_ONE_SHOT_LIMIT,
};
use mp_bg::public::means_of_death::meansOfDeath_t::MOD_UNKNOWN;
use mp_bg::public::viewheight::{DEFAULT_MAXS_2, DEFAULT_MINS_2};
use mp_qshared::common::mp::qcommon::pm_flags::PMF_FOLLOW;
use crate::q_shared;

/// Raven `HOLOCRON_RESPAWN_TIME`.
///
/// Source: `oracle/codemp/game/g_misc.c:10`
pub const HOLOCRON_RESPAWN_TIME: c_int = 30000;

/// Raven `MAX_AMMO_GIVE`.
///
/// Source: `oracle/codemp/game/g_misc.c:11`
pub const MAX_AMMO_GIVE: c_int = 2;

/// Raven `STATION_RECHARGE_TIME`.
///
/// Source: `oracle/codemp/game/g_misc.c:12`
pub const STATION_RECHARGE_TIME: c_int = 100;

/// Raven `FX_ENT_RADIUS`.
///
/// Source: `oracle/codemp/game/g_misc.c:2261`
pub const FX_ENT_RADIUS: c_float = 32.0;

/// Raven `MAX_SHOOTERS`.
///
/// Source: `oracle/codemp/game/g_misc.c:3345`
pub const MAX_SHOOTERS: c_int = 16;

/// Raven `shooterClient_t`.
///
/// Type definition source: `oracle/codemp/game/g_misc.c:3346-3350`
///
/// Not `Copy`: `cl.pers.netname` is an owned `String` (§13).
#[derive(Clone)]
pub struct shooterClient_t {
    pub cl: gclient_t,
    pub inuse: qboolean,
}

impl Default for shooterClient_t {
    fn default() -> Self {
        // Raven's `memset(g_shooterClients, 0, ...)` init: `gclient_t::default()`
        // is that zero image (its `pers.netname` `String` empty, all else 0).
        Self {
            cl: gclient_t::default(),
            inuse: qfalse,
        }
    }
}

// Raven `qboolean` is `c_int`; keep the source spelling at assignment sites.
// Source: `oracle/codemp/game/q_shared.h`

/// Raven `SP_info_camp`.
///
/// Used as a positional target for calculations in the utilities
/// (spotlights, etc), but removed during gameplay.
/// Source: `oracle/codemp/game/g_misc.c:25-27`
pub fn SP_info_camp(self_: &mut gentity_t) {
    // STAGE-1: ctx-free leaf borrows &mut gentity_t (Stage-1 rule 2).
    let origin = self_.s.origin;
    G_SetOrigin(self_, origin);
}

/// Raven `SP_info_null`.
///
/// Used as a positional target for calculations in the utilities
/// (spotlights, etc), but removed during gameplay.
/// Source: `oracle/codemp/game/g_misc.c:33-35`
pub fn SP_info_null(ctx: &mut GameContext, self_: EntityId) {
    G_FreeEntity(ctx, Some(self_));
}

/// Raven `SP_info_notnull`.
///
/// Used as a positional target for in-game calculation, like jumppad
/// targets. `target_position` does the same thing.
/// Source: `oracle/codemp/game/g_misc.c:42-44`
pub fn SP_info_notnull(self_: &mut gentity_t) {
    // STAGE-1: ctx-free leaf borrows &mut gentity_t (Stage-1 rule 2).
    let origin = self_.s.origin;
    G_SetOrigin(self_, origin);
}

/// Raven `misc_lightstyle_set`.
///
/// Source: `oracle/codemp/game/g_misc.c:86-132`
pub fn misc_lightstyle_set(ctx: &mut GameContext, ent: EntityId) {
    unsafe {
        let m_light_style = ctx.world.entity(ent).count;
        let m_light_switch_style = ctx.world.entity(ent).bounceCount;
        let m_light_off_style = ctx.world.entity(ent).fly_sound_debounce_time;
        if ctx.world.entity(ent).alt_fire == qfalse {
            // turn off
            if m_light_off_style != 0 {
                for slot in 0..3 {
                    let s = trap::GetConfigstring(
                        ctx.engine,
                        CS_LIGHT_STYLES + (m_light_off_style * 3) + slot,
                        32,
                    );
                    trap::SetConfigstring(
                        ctx.engine,
                        CS_LIGHT_STYLES + (m_light_style * 3) + slot,
                        &s,
                    );
                }
            } else {
                trap::SetConfigstring(ctx.engine, CS_LIGHT_STYLES + (m_light_style * 3) + 0, "a");
                trap::SetConfigstring(ctx.engine, CS_LIGHT_STYLES + (m_light_style * 3) + 1, "a");
                trap::SetConfigstring(ctx.engine, CS_LIGHT_STYLES + (m_light_style * 3) + 2, "a");
            }
        } else {
            // Turn myself on now
            if m_light_switch_style != 0 {
                for slot in 0..3 {
                    let s = trap::GetConfigstring(
                        ctx.engine,
                        CS_LIGHT_STYLES + (m_light_switch_style * 3) + slot,
                        32,
                    );
                    trap::SetConfigstring(
                        ctx.engine,
                        CS_LIGHT_STYLES + (m_light_style * 3) + slot,
                        &s,
                    );
                }
            } else {
                trap::SetConfigstring(ctx.engine, CS_LIGHT_STYLES + (m_light_style * 3) + 0, "z");
                trap::SetConfigstring(ctx.engine, CS_LIGHT_STYLES + (m_light_style * 3) + 1, "z");
                trap::SetConfigstring(ctx.engine, CS_LIGHT_STYLES + (m_light_style * 3) + 2, "z");
            }
        }
    }
}

/// Raven `misc_dlight_use`.
///
/// Source: `oracle/codemp/game/g_misc.c:134-140`
pub fn misc_dlight_use(
    ctx: &mut GameContext,
    ent: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    G_ActivateBehavior(ctx, Some(ent), bSet_t::BSET_USE as c_int);
    let e = ctx.world.entity_mut(ent);
    e.alt_fire = if e.alt_fire != qfalse { qfalse } else { qtrue };
    misc_lightstyle_set(ctx, ent);
}

/// Raven `SP_light`.
///
/// Source: `oracle/codemp/game/g_misc.c:142-166`
pub fn SP_light(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).targetname_str().is_none() {
        // if i don't have a light style switch, then i go away
        G_FreeEntity(ctx, Some(self_));
        return;
    }

    let mut count: c_int = 0;
    G_SpawnInt(ctx, c"style".as_ptr(), c"0".as_ptr(), &mut count);
    ctx.world.entity_mut(self_).count = count;
    let mut switch_style: c_int = 0;
    G_SpawnInt(
        ctx,
        c"switch_style".as_ptr(),
        c"0".as_ptr(),
        &mut switch_style,
    );
    ctx.world.entity_mut(self_).bounceCount = switch_style;
    let mut style_off: c_int = 0;
    G_SpawnInt(ctx, c"style_off".as_ptr(), c"0".as_ptr(), &mut style_off);
    ctx.world.entity_mut(self_).fly_sound_debounce_time = style_off;

    let origin = ctx.world.entity(self_).s.origin;
    G_SetOrigin(ctx.world.entity_mut(self_), origin);
    let sp: *mut gentity_t = ctx.world.entity_mut(self_);
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(sp.cast()));

    let e = ctx.world.entity_mut(self_);
    e.use_ = Some(EntUse::misc_dlight_use).into();

    e.s.eType = entityType_t::ET_GENERAL as c_int;
    e.alt_fire = qfalse;
    e.r.svFlags |= SVF_NOCLIENT;

    if e.spawnflags & 4 == 0 {
        // turn myself on now
        e.alt_fire = qtrue;
    }
    misc_lightstyle_set(ctx, self_);
}

/// Raven `TeleportPlayer`.
///
/// Source: `oracle/codemp/game/g_misc.c:177-231`
pub fn TeleportPlayer(ctx: &mut GameContext, player: EntityId, origin: vec3_t, angles: vec3_t) {
    use mp_abi::game::syscalls::G_UNLINKENTITY::GUnlinkentityArgs;
    use mp_bg::public::team::TEAM_SPECTATOR;
    // FLAG (task #7): player->client is a real/NPC-pool gclient_t ptr with no
    // accessor for ps; dereffed raw exactly as Raven does (copied pointer value).
    let client = ctx.world.entity(player).client;
    unsafe {
        let mut is_npc = qfalse;
        if ctx.world.entity(player).s.eType == entityType_t::ET_NPC as c_int {
            is_npc = qtrue;
        }

        // use temp events at source and destination to prevent the effect
        // from getting dropped by a second player event
        if (*client).sess.sessionTeam != TEAM_SPECTATOR {
            let cl_origin = (*client).ps.origin;
            let tent = G_TempEntity(ctx, cl_origin, EV_PLAYER_TELEPORT_OUT as c_int);
            let cnum = ctx.world.entity(player).s.clientNum;
            let tid = tent;
            ctx.world.entity_mut(tid).s.clientNum = cnum;

            let tent = G_TempEntity(ctx, origin, EV_PLAYER_TELEPORT_IN as c_int);
            let cnum = ctx.world.entity(player).s.clientNum;
            let tid = tent;
            ctx.world.entity_mut(tid).s.clientNum = cnum;
        }

        // unlink to make sure it can't possibly interfere with G_KillBox
        let pp: *mut gentity_t = ctx.world.entity_mut(player);
        trap::UnlinkEntity(ctx.engine, GUnlinkentityArgs::new(pp.cast()));

        crate::q_math::_VectorCopy(origin, &mut (*client).ps.origin);
        (*client).ps.origin[2] += 1.0;

        // spit the player out
        let mut vel: vec3_t = [0.0, 0.0, 0.0];
        AngleVectors(angles, Some(&mut vel), None, None);
        crate::q_math::_VectorScale(vel, 400.0, &mut (*client).ps.velocity);
        (*client).ps.pm_time = 160; // hold time
        (*client).ps.pm_flags |= PMF_TIME_KNOCKBACK;

        // toggle the teleport bit so the client knows to not lerp
        (*client).ps.eFlags ^= EF_TELEPORT_BIT;

        // set angles
        SetClientViewAngle(ctx.world.entity_mut(player), angles);

        // kill anything at the destination
        if (*client).sess.sessionTeam != TEAM_SPECTATOR {
            G_KillBox(ctx, player);
        }

        // save results of pmove
        BG_PlayerStateToEntityState(
            &mut (*client).ps,
            &mut ctx.world.entity_mut(player).s,
            qtrue,
        );
        if is_npc != qfalse {
            ctx.world.entity_mut(player).s.eType = entityType_t::ET_NPC as c_int;
        }

        // use the precise origin for linking
        let ps_origin = (*client).ps.origin;
        crate::q_math::_VectorCopy(ps_origin, &mut ctx.world.entity_mut(player).r.currentOrigin);

        if (*client).sess.sessionTeam != TEAM_SPECTATOR {
            let pp: *mut gentity_t = ctx.world.entity_mut(player);
            trap::LinkEntity(ctx.engine, GLinkentityArgs::new(pp.cast()));
        }
    }
}

/// Raven `SP_misc_teleporter_dest`.
///
/// Point teleporters at these. Now that we don't have teleport destination
/// pads, this is just an info_notnull.
/// Source: `oracle/codemp/game/g_misc.c:239-240`
pub fn SP_misc_teleporter_dest(ent: &mut gentity_t) {}

/// Raven `SP_misc_model`.
///
/// The live (non-`#if 0`) path just frees the entity — map triangle
/// generation was compiled out.
/// Source: `oracle/codemp/game/g_misc.c:249-262`
pub fn SP_misc_model(ctx: &mut GameContext, ent: EntityId) {
    G_FreeEntity(ctx, Some(ent));
}

/// Raven `SP_misc_model_static`.
///
/// Source: `oracle/codemp/game/g_misc.c:277-280`
pub fn SP_misc_model_static(ctx: &mut GameContext, ent: EntityId) {
    G_FreeEntity(ctx, Some(ent));
}

/// Raven `SP_misc_G2model`.
///
/// The live (non-`#if 0`) path just frees the entity.
/// Source: `oracle/codemp/game/g_misc.c:285-301`
pub fn SP_misc_G2model(ctx: &mut GameContext, ent: EntityId) {
    G_FreeEntity(ctx, Some(ent));
}

/// Raven `locateCamera`.
///
/// Source: `oracle/codemp/game/g_misc.c:305-349`
pub fn locateCamera(ctx: &mut GameContext, ent: EntityId) {
    let ent_target = ctx.world.entity(ent).target.clone();
    let owner = G_PickTarget(ctx, ent_target.as_deref());
    if owner.is_null() {
        G_Printf(ctx, "Couldn't find target for misc_partal_surface\n");
        G_FreeEntity(ctx, Some(ent));
        return;
    }
    let owner_id = ctx.entity_id_of(owner).unwrap();
    let owner_number = ctx.world.entity(owner_id).s.number;
    ctx.world.entity_mut(ent).r.ownerNum = owner_number;

    // frame holds the rotate speed
    let owner_spawnflags = ctx.world.entity(owner_id).spawnflags;
    if owner_spawnflags & 1 != 0 {
        ctx.world.entity_mut(ent).s.frame = 25;
    } else if owner_spawnflags & 2 != 0 {
        ctx.world.entity_mut(ent).s.frame = 75;
    }

    // swing camera ?
    if owner_spawnflags & 4 != 0 {
        // set to 0 for no rotation at all
        ctx.world.entity_mut(ent).s.powerups = 0;
    } else {
        ctx.world.entity_mut(ent).s.powerups = 1;
    }

    // clientNum holds the rotate offset
    let owner_clientnum = ctx.world.entity(owner_id).s.clientNum;
    ctx.world.entity_mut(ent).s.clientNum = owner_clientnum;

    let owner_origin = ctx.world.entity(owner_id).s.origin;
    ctx.world.entity_mut(ent).s.origin2 = owner_origin;

    // see if the portal_camera has a target
    let owner_target = ctx.world.entity(owner_id).target.clone();
    let target = G_PickTarget(ctx, owner_target.as_deref());
    let mut dir: vec3_t = [0.0, 0.0, 0.0];
    if !target.is_null() {
        let target_id = ctx.entity_id_of(target).unwrap();
        let target_origin = ctx.world.entity(target_id).s.origin;
        let owner_origin = ctx.world.entity(owner_id).s.origin;
        crate::q_math::_VectorSubtract(target_origin, owner_origin, &mut dir);
        VectorNormalize(&mut dir);
    } else {
        let mut owner_angles = ctx.world.entity(owner_id).s.angles;
        G_SetMovedir(&mut owner_angles, &mut dir);
        ctx.world.entity_mut(owner_id).s.angles = owner_angles;
    }

    ctx.world.entity_mut(ent).s.eventParm = DirToByte(dir);
}

/// Raven `SP_misc_portal_surface`.
///
/// Source: `oracle/codemp/game/g_misc.c:355-369`
pub fn SP_misc_portal_surface(ctx: &mut GameContext, ent: EntityId) {
    {
        let e = ctx.world.entity_mut(ent);
        e.r.mins = [0.0, 0.0, 0.0];
        e.r.maxs = [0.0, 0.0, 0.0];
    }
    let ep: *mut gentity_t = ctx.world.entity_mut(ent);
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ep.cast()));

    let time = ctx.world.level.time;
    let e = ctx.world.entity_mut(ent);
    e.r.svFlags = SVF_PORTAL;
    e.s.eType = entityType_t::ET_PORTAL as c_int;

    if e.target.is_none() {
        e.s.origin2 = e.s.origin;
    } else {
        e.think = Some(EntThink::locateCamera).into();
        e.nextthink = time + 100;
    }
}

/// Raven `SP_misc_portal_camera`.
///
/// Source: `oracle/codemp/game/g_misc.c:375-385`
pub fn SP_misc_portal_camera(ctx: &mut GameContext, ent: EntityId) {
    {
        let e = ctx.world.entity_mut(ent);
        e.r.mins = [0.0, 0.0, 0.0];
        e.r.maxs = [0.0, 0.0, 0.0];
    }
    let ep: *mut gentity_t = ctx.world.entity_mut(ent);
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ep.cast()));

    let mut roll: f32 = 0.0;
    G_SpawnFloat(ctx, c"roll".as_ptr(), c"0".as_ptr(), &mut roll);

    // C evaluates `roll/360.0 * 256` in double (360.0 is a double literal),
    // then truncates to int.
    ctx.world.entity_mut(ent).s.clientNum = (roll as f64 / 360.0 * 256.0) as c_int;
}

/// Raven `SP_misc_bsp`.
///
/// Source: `oracle/codemp/game/g_misc.c:390-462`
pub fn SP_misc_bsp(ctx: &mut GameContext, ent: EntityId) {
    use mp_abi::game::syscalls::G_SET_ACTIVE_SUBBSP::GSetActiveSubbspArgs;
    use mp_qshared::shared::MAX_QPATH;
    unsafe {
        let mut new_angle: f32 = 0.0;
        G_SpawnFloat(ctx, c"angle".as_ptr(), c"0".as_ptr(), &mut new_angle);
        if new_angle != 0.0 {
            ctx.world.entity_mut(ent).s.angles[1] = new_angle;
        }
        // don't support rotation any other way
        {
            let e = ctx.world.entity_mut(ent);
            e.s.angles[0] = 0.0;
            e.s.angles[2] = 0.0;
        }

        let (_, out) = G_SpawnString(ctx, "bspmodel", "");

        ctx.world.entity_mut(ent).s.eFlags = EF_PERMANENT;

        // Mainly for debugging
        let mut tempint: c_int = 0;
        G_SpawnInt(ctx, c"spacing".as_ptr(), c"0".as_ptr(), &mut tempint);
        ctx.world.entity_mut(ent).s.time2 = tempint;
        G_SpawnInt(ctx, c"flatten".as_ptr(), c"0".as_ptr(), &mut tempint);
        ctx.world.entity_mut(ent).s.time = tempint;

        // NOTE: Raven's own `char temp[MAX_QPATH]` is a stack local later
        // assigned into `level.mTargetAdjust` (a persistent `char *`) — the
        // pointer dangles once this fn returns. Faithful UB per porting
        // rules S19; we keep the one Raven-defined behavior rather than
        // invent a fix.
        let mut temp: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
        write_cstr_field(&mut temp, &format!("#{}", out));
        let ep: *mut gentity_t = ctx.world.entity_mut(ent);
        trap::SetBrushModel(ctx.engine, ep.cast(), &cstr_to_str(temp.as_ptr())); // SV_SetBrushModel -- sets mins and maxs
        G_BSPIndex(ctx, &cstr_to_str(temp.as_ptr()));

        ctx.world.level.mNumBSPInstances += 1;
        write_cstr_field(&mut temp, &format!("{}-", ctx.world.level.mNumBSPInstances));
        let origin = ctx.world.entity(ent).s.origin;
        ctx.world.level.mOriginAdjust = origin;
        ctx.world.level.mRotationAdjust = ctx.world.entity(ent).s.angles[1];
        ctx.world.level.mTargetAdjust = temp.as_mut_ptr();
        ctx.world.level.mBSPInstanceDepth += 1;

        let (_, teamfilter_out) = G_SpawnString(ctx, "teamfilter", "");
        ctx.world.level.mTeamFilter =
            strncpyz_string(teamfilter_out.as_bytes(), MAX_QPATH as usize);

        {
            let e = ctx.world.entity_mut(ent);
            e.s.pos.trBase = e.s.origin;
            e.r.currentOrigin = e.s.origin;
            e.s.apos.trBase = e.s.angles;
            e.r.currentAngles = e.s.angles;
            e.s.eType = entityType_t::ET_MOVER as c_int;
        }

        let ep: *mut gentity_t = ctx.world.entity_mut(ent);
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ep.cast()));

        let modelindex = ctx.world.entity(ent).s.modelindex;
        trap::SetActiveSubBSP(ctx.engine, GSetActiveSubbspArgs::new(modelindex));
        crate::g_spawn::G_SpawnEntitiesFromString(ctx, qtrue);
        trap::SetActiveSubBSP(ctx.engine, GSetActiveSubbspArgs::new(-1));

        ctx.world.level.mBSPInstanceDepth -= 1;
        ctx.world.level.mTeamFilter.clear();
    }
}

/// Raven `SP_terrain`.
///
/// Source: `oracle/codemp/game/g_misc.c:484-631`
pub fn SP_terrain(ctx: &mut GameContext, ent: EntityId) {
    use mp_abi::game::syscalls::G_RMG_INIT::GRmgInitArgs;
    use mp_qshared::shared::MAX_QPATH;
    // `MAX_INFO_STRING` resolves via the crate prelude glob
    // (`mp_qshared::shared::limits`).
    unsafe {
        // Force it to 1 when there is terrain on the level.
        trap::Cvar_Set(ctx.engine, "RMG", "1");
        ctx.world.cvars.g_RMG.integer = 1;

        ctx.world.entity_mut(ent).s.angles = [0.0, 0.0, 0.0];
        let model = ctx.world.entity(ent).model.clone();
        let ep: *mut gentity_t = ctx.world.entity_mut(ent);
        trap::SetBrushModel(ctx.engine, ep.cast(), model.as_deref().unwrap_or(""));

        // Get the shader from the top of the brush
        let shader_num: c_int = 0;

        let mut seed: String = String::new();
        let mut mission_type: String = String::new();
        if ctx.world.cvars.g_RMG.integer != 0 {
            seed = trap::Cvar_VariableStringBuffer(ctx.engine, "RMG_seed", MAX_QPATH as usize);
            mission_type =
                trap::Cvar_VariableStringBuffer(ctx.engine, "RMG_mission", MAX_QPATH as usize);
        }

        // Get info required for the common init
        let mut temp: String = String::new();

        let mut value: String;
        value = G_SpawnString(ctx, "heightmap", "").1;
        Info_SetValueForKey(&mut temp, "heightMap", &value);

        value = G_SpawnString(ctx, "numpatches", "400").1;
        Info_SetValueForKey(&mut temp, "numPatches", &format!("{}", atoi_bytes(value.as_bytes())));

        value = G_SpawnString(ctx, "terxels", "4").1;
        Info_SetValueForKey(&mut temp, "terxels", &format!("{}", atoi_bytes(value.as_bytes())));

        Info_SetValueForKey(&mut temp, "seed", &seed);
        Info_SetValueForKey(
            &mut temp,
            "minx",
            &format!("{:.6}", ctx.world.entity(ent).r.mins[0]),
        );
        Info_SetValueForKey(
            &mut temp,
            "miny",
            &format!("{:.6}", ctx.world.entity(ent).r.mins[1]),
        );
        Info_SetValueForKey(
            &mut temp,
            "minz",
            &format!("{:.6}", ctx.world.entity(ent).r.mins[2]),
        );
        Info_SetValueForKey(
            &mut temp,
            "maxx",
            &format!("{:.6}", ctx.world.entity(ent).r.maxs[0]),
        );
        Info_SetValueForKey(
            &mut temp,
            "maxy",
            &format!("{:.6}", ctx.world.entity(ent).r.maxs[1]),
        );
        Info_SetValueForKey(
            &mut temp,
            "maxz",
            &format!("{:.6}", ctx.world.entity(ent).r.maxs[2]),
        );

        Info_SetValueForKey(
            &mut temp,
            "modelIndex",
            &format!("{}", ctx.world.entity(ent).s.modelindex),
        );

        value = G_SpawnString(ctx, "terraindef", "grassyhills").1;
        Info_SetValueForKey(&mut temp, "terrainDef", &value);

        value = G_SpawnString(ctx, "instancedef", "").1;
        Info_SetValueForKey(&mut temp, "instanceDef", &value);

        value = G_SpawnString(ctx, "miscentdef", "").1;
        Info_SetValueForKey(&mut temp, "miscentDef", &value);

        Info_SetValueForKey(&mut temp, "missionType", &mission_type);

        // `#define MAX_INSTANCE_TYPES 16` at g_misc.c:483.
        const MAX_INSTANCE_TYPES: c_int = 16;
        let mut i: c_int = 0;
        while i < MAX_INSTANCE_TYPES {
            let final_ = trap::Cvar_VariableStringBuffer(
                ctx.engine,
                &format!("RMG_instance{}", i),
                MAX_QPATH as usize,
            );
            if !final_.is_empty() {
                Info_SetValueForKey(&mut temp, &format!("inst{}", i), &final_);
            }
            i += 1;
        }

        // Set additional data required on the client only
        value = G_SpawnString(ctx, "densitymap", "").1;
        Info_SetValueForKey(&mut temp, "densityMap", &value);

        Info_SetValueForKey(&mut temp, "shader", &format!("{}", shader_num));
        value = G_SpawnString(ctx, "texturescale", "0.005").1;
        Info_SetValueForKey(
            &mut temp,
            "texturescale",
            &format!("{:.6}", atof_bytes(value.as_bytes())),
        );

        // Initialise the common aspects of the terrain
        let terrain_id = trap::CM_RegisterTerrain(ctx.engine, &temp);

        Info_SetValueForKey(&mut temp, "terrainId", &format!("{}", terrain_id));

        // Send all the data down to the client
        trap::SetConfigstring(ctx.engine, CS_TERRAINS + terrain_id, &temp);

        // Make sure the contents are properly set
        {
            let e = ctx.world.entity_mut(ent);
            e.r.contents = mp_qshared::shared::surface_flags::CONTENTS_TERRAIN;
            e.r.svFlags = SVF_NOCLIENT;
            e.s.eFlags = EF_PERMANENT;
            e.s.eType = entityType_t::ET_TERRAIN as c_int;
        }

        // Hook into the world so physics will work
        let ep: *mut gentity_t = ctx.world.entity_mut(ent);
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ep.cast()));

        // If running RMG then initialize the terrain and handle team skins
        if ctx.world.cvars.g_RMG.integer != 0 {
            trap::RMG_Init(ctx.engine, GRmgInitArgs::new(terrain_id));
        }
    }
}

/// Raven `G_PortalifyEntities`.
///
/// Source: `oracle/codemp/game/g_misc.c:638-667`
pub fn G_PortalifyEntities(ctx: &mut GameContext, ent: EntityId) {
    use mp_qshared::shared::limits::{ENTITYNUM_NONE, ENTITYNUM_WORLD};
    unsafe {
        let ent_number = ctx.world.entity(ent).s.number;
        let ent_origin = ctx.world.entity(ent).s.origin;
        let mut i: usize = 0;
        while i < mp_qshared::shared::MAX_GENTITIES {
            let scan_id = EntityId(i as u32);
            let inuse = ctx.world.entity(scan_id).inuse;
            let scan_number = ctx.world.entity(scan_id).s.number;
            if inuse != 0 && scan_number != ent_number && {
                let scan_origin = ctx.world.entity(scan_id).r.currentOrigin;
                trap::InPVS(
                    ctx.engine,
                    GInPvsArgs::new(&ent_origin as *const vec3_t, &scan_origin as *const vec3_t),
                ) != 0
            } {
                let scan_origin = ctx.world.entity(scan_id).r.currentOrigin;
                let mut tr: trace_t = core::mem::zeroed();
                trap::Trace(
                    ctx.engine,
                    GTraceArgs::new(
                        &mut tr as *mut trace_t,
                        &ent_origin as *const vec3_t,
                        &vec3_origin as *const vec3_t,
                        &vec3_origin as *const vec3_t,
                        &scan_origin as *const vec3_t,
                        ent_number,
                        mp_qshared::shared::surface_flags::CONTENTS_SOLID,
                    ),
                );
                if tr.fraction == 1.0
                    || (tr.entityNum == (scan_number) as i16
                        && tr.entityNum != (ENTITYNUM_NONE) as i16
                        && tr.entityNum != (ENTITYNUM_WORLD) as i16)
                {
                    let client = ctx.world.entity(scan_id).client;
                    let etype = ctx.world.entity(scan_id).s.eType;
                    if client.is_null() || etype == entityType_t::ET_NPC as c_int {
                        ctx.world.entity_mut(scan_id).s.isPortalEnt = qtrue;
                    }
                }
            }
            i += 1;
        }

        let level_time = ctx.world.level.time;
        let e = ctx.world.entity_mut(ent);
        e.think = Some(EntThink::G_FreeEntity).into();
        e.nextthink = level_time;
    }
}

/// Raven `SP_misc_skyportal_orient`.
///
/// Source: `oracle/codemp/game/g_misc.c:675-678`
pub fn SP_misc_skyportal_orient(ctx: &mut GameContext, ent: EntityId) {
    G_FreeEntity(ctx, Some(ent));
}

/// Raven `SP_misc_skyportal`.
///
/// Source: `oracle/codemp/game/g_misc.c:694-715`
pub fn SP_misc_skyportal(ctx: &mut GameContext, ent: EntityId) {
    let (_, fov) = G_SpawnString(ctx, "fov", "80");
    let fov_x = atof_bytes(fov.as_bytes()) as f32;

    let mut fogv: vec3_t = [0.0, 0.0, 0.0];
    let mut isfog: c_int = 0;
    isfog += G_SpawnVector(
        ctx,
        c"fogcolor".as_ptr(),
        c"0 0 0".as_ptr(),
        fogv.as_mut_ptr(),
    );
    let mut fogn: c_int = 0;
    isfog += G_SpawnInt(ctx, c"fognear".as_ptr(), c"0".as_ptr(), &mut fogn);
    let mut fogf: c_int = 0;
    isfog += G_SpawnInt(ctx, c"fogfar".as_ptr(), c"300".as_ptr(), &mut fogf);

    let origin = ctx.world.entity(ent).s.origin;
    let s = format!(
        "{:.2} {:.2} {:.2} {:.1} {} {:.2} {:.2} {:.2} {} {}",
        origin[0], origin[1], origin[2], fov_x, isfog, fogv[0], fogv[1], fogv[2], fogn, fogf
    );
    trap::SetConfigstring(ctx.engine, CS_SKYBOXORG, &s);

    let level_time = ctx.world.level.time;
    let e = ctx.world.entity_mut(ent);
    e.think = Some(EntThink::G_PortalifyEntities).into();
    e.nextthink = level_time + 1050; // give it some time first so that all other entities are spawned.
}

/// Raven `HolocronRespawn`.
///
/// Source: `oracle/codemp/game/g_misc.c:760-763`
pub fn HolocronRespawn(self_: &mut gentity_t) {
    // STAGE-1: ctx-free leaf borrows &mut gentity_t (Stage-1 rule 2).
    self_.s.modelindex = self_.count - 128;
}

/// Raven `HolocronPopOut`.
///
/// Source: `oracle/codemp/game/g_misc.c:765-784`
pub fn HolocronPopOut(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.bg_state.rng.Q_irand(1, 10) < 5 {
        let v = ctx.world.bg_state.rng.Q_irand(1, 100) as f32;
        ctx.world.entity_mut(self_).s.pos.trDelta[0] = 150.0 + v;
    } else {
        let v = ctx.world.bg_state.rng.Q_irand(1, 100) as f32;
        ctx.world.entity_mut(self_).s.pos.trDelta[0] = -150.0 - v;
    }
    if ctx.world.bg_state.rng.Q_irand(1, 10) < 5 {
        let v = ctx.world.bg_state.rng.Q_irand(1, 100) as f32;
        ctx.world.entity_mut(self_).s.pos.trDelta[1] = 150.0 + v;
    } else {
        let v = ctx.world.bg_state.rng.Q_irand(1, 100) as f32;
        ctx.world.entity_mut(self_).s.pos.trDelta[1] = -150.0 - v;
    }
    let v = ctx.world.bg_state.rng.Q_irand(1, 100) as f32;
    ctx.world.entity_mut(self_).s.pos.trDelta[2] = 150.0 + v;
}

/// Raven `HolocronTouch`.
///
/// Source: `oracle/codemp/game/g_misc.c:786-905`
pub fn HolocronTouch(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    trace: *mut trace_t,
) {
    use mp_qshared::shared::sound_channel::CHAN_AUTO;
    let mut i: c_int = 0;
    let mut othercarrying: c_int = 0;
    let mut time_lowest: f32 = 0.0;
    let mut index_lowest: c_int = -1;
    let mut hasall = true;
    let force_reselect = WP_NONE;

    if !trace.is_null() {
        let en = unsafe { (*trace).entityNum } as i32;
        ctx.world.entity_mut(self_).s.groundEntityNum = en;
    }

    let other = match other {
        Some(o) => o,
        None => return,
    };
    // FLAG (task #7): other->client is a player/NPC-pool gclient_t ptr; ps is
    // dereffed raw exactly as Raven does (copied pointer value).
    let client = ctx.world.entity(other).client;
    if client.is_null() || ctx.world.entity(other).health < 1 {
        return;
    }

    if ctx.world.entity(self_).s.modelindex == 0 {
        return;
    }

    if ctx.world.entity(self_).enemy.is_some() {
        return;
    }

    let self_count = ctx.world.entity(self_).count;
    if unsafe { (*client).ps.holocronsCarried[self_count as usize] } != 0.0 {
        return;
    }

    let self_number = ctx.world.entity(self_).s.number;
    let level_time = ctx.world.level.time;
    if unsafe { (*client).ps.holocronCantTouch } == self_number
        && unsafe { (*client).ps.holocronCantTouchTime } > level_time as f32
    {
        return;
    }

    while i < (NUM_FORCE_POWERS) as i32 {
        let carried = unsafe { (*client).ps.holocronsCarried[i as usize] };
        if carried != 0.0 {
            othercarrying += 1;

            if index_lowest == -1 || carried < time_lowest {
                index_lowest = i;
                time_lowest = carried;
            }
        } else if i != self_count {
            hasall = false;
        }
        i += 1;
    }

    if hasall {
        // once we pick up this holocron we'll have all of them, so give us super special best prize!
        //G_Printf("You deserve a pat on the back.\n");
    }

    let fpa = unsafe { (*client).ps.fd.forcePowersActive };
    let fps = unsafe { (*client).ps.fd.forcePowerSelected };
    if fpa & (1 << fps) == 0 {
        // If the player isn't using his currently selected force power, select this one
        if self_count != FP_SABER_OFFENSE
            && self_count != FP_SABER_DEFENSE
            && self_count != FP_SABERTHROW
            && self_count != FP_LEVITATION
        {
            unsafe {
                (*client).ps.fd.forcePowerSelected = self_count;
            }
        }
    }

    let maxcarry = ctx.world.cvars.g_MaxHolocronCarry.integer;
    if maxcarry != 0 && othercarrying >= maxcarry {
        // make the oldest holocron carried by the player pop out to make room for this one
        unsafe {
            (*client).ps.holocronsCarried[index_lowest as usize] = 0.0;
        }
        //NOTE: No longer valid as we are now always giving a force level 1 saber attack level in holocron
    }

    //G_Sound(other, CHAN_AUTO, G_SoundIndex("sound/weapons/w_pkup.wav"));
    let self_number = ctx.world.entity(self_).s.number;
    G_AddEvent(
        ctx.world.entity_mut(other),
        mp_bg::public::entity_event::entity_event_t::EV_ITEM_PICKUP as c_int,
        self_number,
    );

    let level_time = ctx.world.level.time;
    unsafe {
        (*client).ps.holocronsCarried[self_count as usize] = level_time as f32;
    }
    ctx.world.entity_mut(self_).s.modelindex = 0;
    ctx.world.entity_mut(self_).enemy = Some(other);

    {
        let e = ctx.world.entity_mut(self_);
        e.pos2[0] = 1.0;
        e.pos2[1] = (level_time + HOLOCRON_RESPAWN_TIME) as f32;
    }

    if force_reselect != WP_NONE {
        G_AddEvent(
            ctx.world.entity_mut(other),
            mp_bg::public::entity_event::entity_event_t::EV_NOAMMO as c_int,
            force_reselect,
        );
    }

    //G_Printf("DON'T TOUCH ME\n");
}

// Raven's `goto justthink;` is ported as an early return after inlining the
// shared tail (porting-rules §C10 — preserve behavior, not shape).
/// Raven `HolocronThink`.
///
/// Source: `oracle/codemp/game/g_misc.c:907-991`
pub fn HolocronThink(ctx: &mut GameContext, ent: EntityId) {
    // FLAG (task #7): the holocron carrier (`enemy`) is a player/NPC-pool
    // client; its `client->ps` is dereffed raw exactly as Raven does (copied
    // pointer value), while the holocron entity itself rides the accessors.
    let justthink = |id: EntityId, ctx: &mut GameContext| {
        let time = ctx.world.level.time;
        ctx.world.entity_mut(id).nextthink = time + 50;
        let td = ctx.world.entity(id).s.pos.trDelta;
        if td[0] != 0.0 || td[1] != 0.0 || td[2] != 0.0 {
            G_RunObject(ctx, id);
        }
    };

    let pos2_0 = ctx.world.entity(ent).pos2[0];
    let enemy = ctx.world.entity(ent).enemy;
    let cond1 = pos2_0 != 0.0
        && match enemy {
            None => true,
            Some(e) => {
                let c = ctx.world.entity(e).client;
                c.is_null() || ctx.world.entity(e).health < 1
            }
        };
    if cond1 {
        if let Some(e) = enemy {
            let c = ctx.world.entity(e).client;
            if !c.is_null() {
                HolocronRespawn(ctx.world.entity_mut(ent));
                let cl_origin = unsafe { (*c).ps.origin };
                {
                    let en = ctx.world.entity_mut(ent);
                    en.s.pos.trBase = cl_origin;
                    en.s.origin = cl_origin;
                    en.r.currentOrigin = cl_origin;
                }
                // copy to person carrying's origin before popping out of them
                HolocronPopOut(ctx, ent);
                let count = ctx.world.entity(ent).count;
                unsafe {
                    (*c).ps.holocronsCarried[count as usize] = 0.0;
                }
                ctx.world.entity_mut(ent).enemy = None;

                justthink(ent, ctx);
                return;
            }
        }
    } else {
        let cond2 = pos2_0 != 0.0
            && match enemy {
                Some(e) => !ctx.world.entity(e).client.is_null(),
                None => false,
            };
        if cond2 {
            let time = ctx.world.level.time;
            ctx.world.entity_mut(ent).pos2[1] = (time + HOLOCRON_RESPAWN_TIME) as f32;
        }
    }

    let enemy = ctx.world.entity(ent).enemy;
    if let Some(e) = enemy {
        let c = ctx.world.entity(e).client;
        if !c.is_null() {
            let count = ctx.world.entity(ent).count;
            if unsafe { (*c).ps.holocronsCarried[count as usize] } == 0.0 {
                let self_number = ctx.world.entity(ent).s.number;
                let time = ctx.world.level.time;
                unsafe {
                    (*c).ps.holocronCantTouch = self_number;
                    (*c).ps.holocronCantTouchTime = (time + 5000) as f32;
                }

                HolocronRespawn(ctx.world.entity_mut(ent));
                let cl_origin = unsafe { (*c).ps.origin };
                {
                    let en = ctx.world.entity_mut(ent);
                    en.s.pos.trBase = cl_origin;
                    en.s.origin = cl_origin;
                    en.r.currentOrigin = cl_origin;
                }
                // copy to person carrying's origin before popping out of them
                HolocronPopOut(ctx, ent);
                ctx.world.entity_mut(ent).enemy = None;

                justthink(ent, ctx);
                return;
            }

            let inuse = ctx.world.entity(e).inuse;
            let falling = unsafe { (*c).ps.fallingToDeath } != 0;
            if inuse == 0 || falling {
                if inuse != 0 && !ctx.world.entity(e).client.is_null() {
                    let count = ctx.world.entity(ent).count;
                    unsafe {
                        (*c).ps.holocronBits &= !(1 << count);
                        (*c).ps.holocronsCarried[count as usize] = 0.0;
                    }
                }
                ctx.world.entity_mut(ent).enemy = None;
                HolocronRespawn(ctx.world.entity_mut(ent));
                let origin2 = ctx.world.entity(ent).s.origin2;
                {
                    let en = ctx.world.entity_mut(ent);
                    en.s.pos.trBase = origin2;
                    en.s.origin = origin2;
                    en.r.currentOrigin = origin2;
                }

                let time = ctx.world.level.time;
                ctx.world.entity_mut(ent).s.pos.trTime = time;

                ctx.world.entity_mut(ent).pos2[0] = 0.0;

                let ep: *mut gentity_t = ctx.world.entity_mut(ent);
                trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ep.cast()));

                justthink(ent, ctx);
                return;
            }
        }
    }

    let pos2 = ctx.world.entity(ent).pos2;
    let time = ctx.world.level.time;
    if pos2[0] != 0.0 && pos2[1] < time as f32 {
        // isn't in original place and has been there for (HOLOCRON_RESPAWN_TIME) seconds without being picked up, so respawn
        let origin2 = ctx.world.entity(ent).s.origin2;
        {
            let en = ctx.world.entity_mut(ent);
            en.s.pos.trBase = origin2;
            en.s.origin = origin2;
            en.r.currentOrigin = origin2;
        }

        let time = ctx.world.level.time;
        ctx.world.entity_mut(ent).s.pos.trTime = time;

        ctx.world.entity_mut(ent).pos2[0] = 0.0;

        let ep: *mut gentity_t = ctx.world.entity_mut(ent);
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ep.cast()));
    }

    justthink(ent, ctx);
}

/// Raven `SP_misc_holocron`.
///
/// Source: `oracle/codemp/game/g_misc.c:993-1097`
pub fn SP_misc_holocron(ctx: &mut GameContext, ent: EntityId) {
    use mp_bg::public::gametype::GT_HOLOCRON;
    unsafe {
        let mut dest: vec3_t;
        let mut tr: trace_t = core::mem::zeroed();

        if ctx.world.cvars.g_gametype.integer != GT_HOLOCRON {
            G_FreeEntity(ctx, Some(ent));
            return;
        }

        if crate::w_saber::HasSetSaberOnly(ctx) {
            let count = ctx.world.entity(ent).count;
            if count == FP_SABER_OFFENSE || count == FP_SABER_DEFENSE || count == FP_SABERTHROW {
                // having saber holocrons in saber only mode is pointless
                G_FreeEntity(ctx, Some(ent));
                return;
            }
        }

        {
            let e = ctx.world.entity_mut(ent);
            e.s.isJediMaster = qtrue;

            e.r.maxs = [8.0, 8.0, 8.0];
            e.r.mins = [-8.0, -8.0, -8.0];

            // `0.1` is a bare double in the oracle; add in f64, narrow once at
            // the f32 store. Source: g_misc.c:1020-1021
            e.s.origin[2] = (e.s.origin[2] as f64 + 0.1) as f32;
            e.r.maxs[2] = (e.r.maxs[2] as f64 - 0.1) as f32;
        }

        let origin = ctx.world.entity(ent).s.origin;
        let mins = ctx.world.entity(ent).r.mins;
        let maxs = ctx.world.entity(ent).r.maxs;
        let number = ctx.world.entity(ent).s.number;
        dest = [origin[0], origin[1], origin[2] - 4096.0];
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &origin as *const vec3_t,
                &mins as *const vec3_t,
                &maxs as *const vec3_t,
                &dest as *const vec3_t,
                number,
                mp_qshared::shared::surface_flags::MASK_SOLID,
            ),
        );
        if tr.startsolid != 0 {
            let msg = format!(
                "SP_misc_holocron: misc_holocron startsolid at {}\n",
                vtos(ctx, origin)
            );
            G_Printf(ctx, &msg);
            G_FreeEntity(ctx, Some(ent));
            return;
        }

        // add the 0.1 back after the trace (bare double; g_misc.c:1033)
        {
            let e = ctx.world.entity_mut(ent);
            e.r.maxs[2] = (e.r.maxs[2] as f64 + 0.1) as f32;
        }

        // allow to ride movers
        //	ent->s.groundEntityNum = tr.entityNum;

        G_SetOrigin(ctx.world.entity_mut(ent), tr.endpos);

        if ctx.world.entity(ent).count < 0 {
            ctx.world.entity_mut(ent).count = 0;
        }

        if ctx.world.entity(ent).count >= (NUM_FORCE_POWERS) as i32 {
            ctx.world.entity_mut(ent).count = (NUM_FORCE_POWERS - 1) as i32;
        }
        //No longer doing this, causing too many complaints about accidentally setting no force powers at all
        //and starting a holocron game (making it basically just FFA)

        let count = ctx.world.entity(ent).count;
        let level_time = ctx.world.level.time;
        {
            let e = ctx.world.entity_mut(ent);
            e.enemy = None;

            e.flags = FL_BOUNCE_HALF;

            e.s.modelindex = count - 128; //G_ModelIndex(holocronTypeModels[ent->count]);
            e.s.eType = entityType_t::ET_HOLOCRON as c_int;
            e.s.pos.trType = TR_GRAVITY;
            e.s.pos.trTime = level_time;

            e.r.contents = mp_qshared::shared::surface_flags::CONTENTS_TRIGGER;
            e.clipmask = mp_qshared::shared::surface_flags::MASK_SOLID;

            e.s.trickedentindex4 = count;
        }

        let dark_light = mp_bg::bg_misc::forcePowerDarkLight[count as usize];
        if dark_light == FORCE_DARKSIDE {
            ctx.world.entity_mut(ent).s.trickedentindex3 = 1;
        } else if dark_light == FORCE_LIGHTSIDE {
            ctx.world.entity_mut(ent).s.trickedentindex3 = 2;
        } else {
            ctx.world.entity_mut(ent).s.trickedentindex3 = 3;
        }

        {
            let e = ctx.world.entity_mut(ent);
            e.physicsObject = qtrue;

            e.s.origin2 = e.s.pos.trBase; // remember the spawn spot

            e.touch = Some(EntTouch::HolocronTouch).into();
        }

        let ep: *mut gentity_t = ctx.world.entity_mut(ent);
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ep.cast()));

        let level_time = ctx.world.level.time;
        let e = ctx.world.entity_mut(ent);
        e.think = Some(EntThink::HolocronThink).into();
        e.nextthink = level_time + 50;
    }
}

/// Raven `Use_Shooter`.
///
/// Source: `oracle/codemp/game/g_misc.c:1107-1139`
pub fn Use_Shooter(
    ctx: &mut GameContext,
    ent: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    let mut dir: vec3_t;
    let mut up: vec3_t = [0.0, 0.0, 0.0];
    let mut right: vec3_t = [0.0, 0.0, 0.0];

    // see if we have a target
    if let Some(e) = ctx.world.entity(ent).enemy {
        let enemy_origin = ctx.world.entity(e).r.currentOrigin;
        let ent_origin = ctx.world.entity(ent).s.origin;
        let mut d: vec3_t = [0.0, 0.0, 0.0];
        crate::q_math::_VectorSubtract(enemy_origin, ent_origin, &mut d);
        VectorNormalize(&mut d);
        dir = d;
    } else {
        dir = ctx.world.entity(ent).movedir;
    }

    // randomize a bit
    PerpendicularVector(&mut up, dir);
    CrossProduct(up, dir, &mut right);

    // C `float deg = crandom() * ent->random`: the `double` product narrows
    // to `float deg`, then feeds `VectorMA` as the scale.
    let random = ctx.world.entity(ent).random;
    let mut deg = (ctx.world.bg_state.rng.crandom() * random as f64) as f32;
    let mut new_dir: vec3_t = [0.0, 0.0, 0.0];
    crate::q_math::_VectorMA(dir, deg, up, &mut new_dir);
    dir = new_dir;

    deg = (ctx.world.bg_state.rng.crandom() * random as f64) as f32;
    crate::q_math::_VectorMA(dir, deg, right, &mut new_dir);
    dir = new_dir;

    VectorNormalize(&mut dir);

    match ctx.world.entity(ent).s.weapon {
        w if w == mp_bg::weapons::weapon_t::WP_BLASTER => {
            let ent_origin = ctx.world.entity(ent).s.origin;
            crate::g_weapon::WP_FireBlasterMissile(ctx, ent, ent_origin, dir, false);
        }
        _ => {}
    }

    G_AddEvent(
        ctx.world.entity_mut(ent),
        mp_bg::public::entity_event::entity_event_t::EV_FIRE_WEAPON as c_int,
        0,
    );
}

/// Raven `InitShooter_Finish`.
///
/// Source: `oracle/codemp/game/g_misc.c:1142-1146`
pub fn InitShooter_Finish(ctx: &mut GameContext, ent: EntityId) {
    let target = ctx.world.entity(ent).target.clone();
    let picked = G_PickTarget(ctx, target.as_deref());
    let enemy = ctx.entity_id_of(picked);
    let e = ctx.world.entity_mut(ent);
    e.enemy = enemy;
    e.think = FnId::NONE;
    e.nextthink = 0;
}

/// Raven `InitShooter`.
///
/// Source: `oracle/codemp/game/g_misc.c:1148-1166`
pub fn InitShooter(ctx: &mut GameContext, ent: EntityId, weapon: c_int) {
    {
        let e = ctx.world.entity_mut(ent);
        e.use_ = Some(EntUse::Use_Shooter).into();
        e.s.weapon = weapon;
    }

    crate::g_items::RegisterItem(ctx, mp_bg::bg_misc::BG_FindItemForWeapon(weapon));

    let mut angles = ctx.world.entity(ent).s.angles;
    let mut movedir = ctx.world.entity(ent).movedir;
    G_SetMovedir(&mut angles, &mut movedir);
    {
        let e = ctx.world.entity_mut(ent);
        e.s.angles = angles;
        e.movedir = movedir;
    }

    if ctx.world.entity(ent).random == 0.0 {
        ctx.world.entity_mut(ent).random = 1.0;
    }
    // C evaluates `sin( M_PI * ent->random / 180 )` in double (M_PI and the
    // libm sin are double); narrow only on store.
    let random = ctx.world.entity(ent).random;
    ctx.world.entity_mut(ent).random = (std::f64::consts::PI * random as f64 / 180.0).sin() as f32;
    // target might be a moving object, so we can't set movedir for it
    if ctx.world.entity(ent).target.is_some() {
        let time = ctx.world.level.time;
        let e = ctx.world.entity_mut(ent);
        e.think = Some(EntThink::InitShooter_Finish).into();
        e.nextthink = time + 500;
    }
    let ep: *mut gentity_t = ctx.world.entity_mut(ent);
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ep.cast()));
}

/// Raven `SP_shooter_blaster`.
///
/// Source: `oracle/codemp/game/g_misc.c:1172-1174`
pub fn SP_shooter_blaster(ctx: &mut GameContext, ent: EntityId) {
    InitShooter(ctx, ent, mp_bg::weapons::weapon_t::WP_BLASTER);
}

/// Raven `check_recharge`.
///
/// Source: `oracle/codemp/game/g_misc.c:1176-1206`
pub fn check_recharge(ctx: &mut GameContext, ent: EntityId) {
    use mp_qshared::common::mp::qcommon::usercmd_button::BUTTON_USE;
    use mp_qshared::shared::sound_channel::CHAN_AUTO;

    let activator = ctx.world.entity(ent).activator;
    // FLAG (task #7): activator->client is a player/NPC-pool gclient_t ptr;
    // pers.cmd dereffed raw exactly as Raven does (copied pointer value).
    let activator_cl = match activator {
        Some(a) => ctx.world.entity(a).client,
        None => core::ptr::null_mut(),
    };
    let fsdt = ctx.world.entity(ent).fly_sound_debounce_time;
    let level_time = ctx.world.level.time;
    let buttons_use_clear =
        activator_cl.is_null() || unsafe { (*activator_cl).pers.cmd.buttons } & BUTTON_USE == 0;
    if fsdt < level_time || activator.is_none() || buttons_use_clear {
        if activator.is_some() {
            let gv7 = ctx.world.entity(ent).genericValue7;
            G_Sound(ctx, Some(ent), CHAN_AUTO as c_int, gv7);
        }
        let e = ctx.world.entity_mut(ent);
        e.s.loopSound = 0;
        e.s.loopIsSoundset = qfalse;
        e.activator = None;
        e.fly_sound_debounce_time = 0;
    }

    if ctx.world.entity(ent).activator.is_none() {
        let level_time = ctx.world.level.time;
        if ctx.world.entity(ent).genericValue8 < level_time {
            if ctx.world.entity(ent).count < ctx.world.entity(ent).genericValue4 {
                ctx.world.entity_mut(ent).count += 1;
            }
            let gv5 = ctx.world.entity(ent).genericValue5;
            ctx.world.entity_mut(ent).genericValue8 = level_time + gv5;
        }
    }
    let count = ctx.world.entity(ent).count;
    ctx.world.entity_mut(ent).s.health = count;
    let level_time = ctx.world.level.time;
    ctx.world.entity_mut(ent).nextthink = level_time;
}

/// Raven `EnergyShieldStationSettings`.
///
/// Source: `oracle/codemp/game/g_misc.c:1213-1223`
pub fn EnergyShieldStationSettings(ctx: &mut GameContext, ent: EntityId) {
    let mut count: c_int = 0;
    G_SpawnInt(ctx, c"count".as_ptr(), c"200".as_ptr(), &mut count);
    ctx.world.entity_mut(ent).count = count;

    let mut chargerate: c_int = 0;
    G_SpawnInt(ctx, c"chargerate".as_ptr(), c"0".as_ptr(), &mut chargerate);
    ctx.world.entity_mut(ent).genericValue5 = chargerate;

    if ctx.world.entity(ent).genericValue5 == 0 {
        ctx.world.entity_mut(ent).genericValue5 = STATION_RECHARGE_TIME;
    }
}

/// Raven `shield_power_converter_use`.
///
/// Source: `oracle/codemp/game/g_misc.c:1230-1328`
pub fn shield_power_converter_use(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    use mp_bg::public::gametype::GT_SIEGE;
    use mp_bg::public::stat_index::statIndex_t::{STAT_ARMOR, STAT_MAX_HEALTH};
    use mp_qshared::shared::sound_channel::CHAN_AUTO;

    let mut stop = true;

    let activator = match activator {
        Some(a) => a,
        None => return,
    };
    // FLAG (task #7): activator/other ->client are player/NPC-pool gclient_t
    // ptrs; ps/siegeClass dereffed raw exactly as Raven does (copied pointers).
    let act_cl = ctx.world.entity(activator).client;
    if act_cl.is_null() {
        return;
    }
    let other_cl = match other {
        Some(o) => ctx.world.entity(o).client,
        None => core::ptr::null_mut(),
    };
    let gametype = ctx.world.cvars.g_gametype.integer;

    if gametype == GT_SIEGE && !other_cl.is_null() && unsafe { (*other_cl).siegeClass } != 0 {
        let sc = unsafe { (*other_cl).siegeClass };
        if ctx.world.bg_state.bgSiegeClasses[sc as usize].maxarmor == 0 {
            // can't use it!
            let snd = G_SoundIndex("sound/interface/shieldcon_empty");
            G_Sound(ctx, Some(self_), CHAN_AUTO as c_int, snd);
            return;
        }
    }

    let set_time = ctx.world.entity(self_).setTime;
    let level_time = ctx.world.level.time;
    if set_time < level_time {
        let max_armor: c_int;
        if ctx.world.entity(self_).s.loopSound == 0 {
            let snd = G_SoundIndex("sound/interface/shieldcon_run");
            let e = ctx.world.entity_mut(self_);
            e.s.loopSound = snd;
            e.s.loopIsSoundset = qfalse;
        }
        ctx.world.entity_mut(self_).setTime = level_time + 100;

        if gametype == GT_SIEGE && !other_cl.is_null() && unsafe { (*other_cl).siegeClass } != -1 {
            let sc = unsafe { (*other_cl).siegeClass };
            max_armor = ctx.world.bg_state.bgSiegeClasses[sc as usize].maxarmor;
        } else {
            max_armor = unsafe { (*act_cl).ps.stats[STAT_MAX_HEALTH as usize] };
        }
        let dif = max_armor - unsafe { (*act_cl).ps.stats[STAT_ARMOR as usize] };

        if dif > 0 {
            // Already at full armor?
            let mut add = if dif > MAX_AMMO_GIVE {
                MAX_AMMO_GIVE
            } else {
                dif
            };

            if ctx.world.entity(self_).count < add {
                add = ctx.world.entity(self_).count;
            }

            if ctx.world.entity(self_).genericValue12 == 0 {
                ctx.world.entity_mut(self_).count -= add;
            }
            if ctx.world.entity(self_).count <= 0 {
                ctx.world.entity_mut(self_).setTime = 0;
            }
            stop = false;

            ctx.world.entity_mut(self_).fly_sound_debounce_time = level_time + 500;
            ctx.world.entity_mut(self_).activator = Some(activator);

            unsafe {
                (*act_cl).ps.stats[STAT_ARMOR as usize] += add;
            }
        }
    }

    if stop || ctx.world.entity(self_).count <= 0 {
        let loop_sound = ctx.world.entity(self_).s.loopSound;
        let set_time = ctx.world.entity(self_).setTime;
        let level_time = ctx.world.level.time;
        if loop_sound != 0 && set_time < level_time {
            if ctx.world.entity(self_).count <= 0 {
                let snd = G_SoundIndex("sound/interface/shieldcon_empty");
                G_Sound(ctx, Some(self_), CHAN_AUTO as c_int, snd);
            } else {
                let gv7 = ctx.world.entity(self_).genericValue7;
                G_Sound(ctx, Some(self_), CHAN_AUTO as c_int, gv7);
            }
        }
        {
            let e = ctx.world.entity_mut(self_);
            e.s.loopSound = 0;
            e.s.loopIsSoundset = qfalse;
        }
        let set_time = ctx.world.entity(self_).setTime;
        let level_time = ctx.world.level.time;
        if set_time < level_time {
            let gv5 = ctx.world.entity(self_).genericValue5;
            ctx.world.entity_mut(self_).setTime = level_time + gv5 + 100;
        }
    }
}

/// Raven `ammo_generic_power_converter_use`.
///
/// Source: `oracle/codemp/game/g_misc.c:1331-1505`
pub fn ammo_generic_power_converter_use(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    use mp_bg::public::gametype::GT_SIEGE;
    use mp_bg::weapons::ammo_t::ammo_t::{AMMO_BLASTER, AMMO_MAX, AMMO_ROCKETS};
    use mp_qshared::shared::sound_channel::CHAN_AUTO;

    let mut stop = true;

    let activator = match activator {
        Some(a) => a,
        None => return,
    };
    // FLAG (task #7): activator->client is a player/NPC-pool gclient_t ptr; ps
    // eFlags/ammo dereffed raw exactly as Raven does (copied pointer value).
    let acl = ctx.world.entity(activator).client;
    if acl.is_null() {
        return;
    }
    let gametype = ctx.world.cvars.g_gametype.integer;

    let set_time = ctx.world.entity(self_).setTime;
    let level_time = ctx.world.level.time;
    if set_time < level_time {
        let mut gave_some = false;

        let mut i = AMMO_BLASTER as c_int;
        if ctx.world.entity(self_).s.loopSound == 0 {
            let snd = G_SoundIndex("sound/interface/ammocon_run");
            let e = ctx.world.entity_mut(self_);
            e.s.loopSound = snd;
            e.s.loopIsSoundset = qfalse;
        }
        ctx.world.entity_mut(self_).fly_sound_debounce_time = level_time + 500;
        ctx.world.entity_mut(self_).activator = Some(activator);
        while i < AMMO_MAX as c_int {
            let mut add = (ammoData[i as usize].max as f32 * 0.05) as c_int;
            if add < 1 {
                add = 1;
            }
            let max = ammoData[i as usize].max;
            let eflags = unsafe { (*acl).ps.eFlags };
            let ammo_i = unsafe { (*acl).ps.ammo[i as usize] };
            if (eflags & EF_DOUBLE_AMMO != 0 && ammo_i < max * 2) || ammo_i < max {
                gave_some = true;
                if gametype == GT_SIEGE && i == AMMO_ROCKETS as c_int && ammo_i >= 10 {
                    // this stuff is already a freaking mess, so..
                    gave_some = false;
                }
                unsafe {
                    (*acl).ps.ammo[i as usize] += add;
                }
                let ammo_i2 = unsafe { (*acl).ps.ammo[i as usize] };
                if gametype == GT_SIEGE && i == AMMO_ROCKETS as c_int && ammo_i2 >= 10 {
                    // fixme - this should SERIOUSLY be externed.
                    unsafe {
                        (*acl).ps.ammo[i as usize] = 10;
                    }
                } else if eflags & EF_DOUBLE_AMMO != 0 {
                    if ammo_i2 >= max * 2 {
                        unsafe {
                            (*acl).ps.ammo[i as usize] = max * 2;
                        }
                    } else {
                        stop = false;
                    }
                } else {
                    if ammo_i2 >= max {
                        unsafe {
                            (*acl).ps.ammo[i as usize] = max;
                        }
                    } else {
                        stop = false;
                    }
                }
            }
            i += 1;
            if ctx.world.entity(self_).genericValue12 == 0 && gave_some {
                let mut sub = (add as f32 * 0.2) as c_int;
                if sub < 1 {
                    sub = 1;
                }
                ctx.world.entity_mut(self_).count -= sub;
                if ctx.world.entity(self_).count <= 0 {
                    ctx.world.entity_mut(self_).count = 0;
                    stop = true;
                    break;
                }
            }
        }
    }

    if stop || ctx.world.entity(self_).count <= 0 {
        let loop_sound = ctx.world.entity(self_).s.loopSound;
        let set_time = ctx.world.entity(self_).setTime;
        let level_time = ctx.world.level.time;
        if loop_sound != 0 && set_time < level_time {
            if ctx.world.entity(self_).count <= 0 {
                let snd = G_SoundIndex("sound/interface/ammocon_empty");
                G_Sound(ctx, Some(self_), CHAN_AUTO as c_int, snd);
            } else {
                let gv7 = ctx.world.entity(self_).genericValue7;
                G_Sound(ctx, Some(self_), CHAN_AUTO as c_int, gv7);
            }
        }
        {
            let e = ctx.world.entity_mut(self_);
            e.s.loopSound = 0;
            e.s.loopIsSoundset = qfalse;
        }
        let set_time = ctx.world.entity(self_).setTime;
        let level_time = ctx.world.level.time;
        if set_time < level_time {
            let gv5 = ctx.world.entity(self_).genericValue5;
            ctx.world.entity_mut(self_).setTime = level_time + gv5 + 100;
        }
    }
}

/// Raven `SP_misc_ammo_floor_unit`.
///
/// Source: `oracle/codemp/game/g_misc.c:1515-1592`
pub fn SP_misc_ammo_floor_unit(ctx: &mut GameContext, ent: EntityId) {
    use mp_bg::public::gametype::GT_SIEGE;
    use mp_qshared::shared::limits::ENTITYNUM_NONE;
    unsafe {
        let mut tr: trace_t = core::mem::zeroed();

        {
            let e = ctx.world.entity_mut(ent);
            e.r.mins = [-16.0, -16.0, 0.0];
            e.r.maxs = [16.0, 16.0, 40.0];

            e.s.origin[2] += 0.1;
            e.r.maxs[2] -= 0.1;
        }

        let origin = ctx.world.entity(ent).s.origin;
        let mins = ctx.world.entity(ent).r.mins;
        let maxs = ctx.world.entity(ent).r.maxs;
        let number = ctx.world.entity(ent).s.number;
        let dest: vec3_t = [origin[0], origin[1], origin[2] - 4096.0];
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &origin as *const vec3_t,
                &mins as *const vec3_t,
                &maxs as *const vec3_t,
                &dest as *const vec3_t,
                number,
                MASK_SOLID,
            ),
        );
        if tr.startsolid != 0 {
            let msg = format!(
                "SP_misc_ammo_floor_unit: misc_ammo_floor_unit startsolid at {}\n",
                vtos(ctx, origin)
            );
            G_Printf(ctx, &msg);
            G_FreeEntity(ctx, Some(ent));
            return;
        }

        // add the 0.1 back after the trace
        ctx.world.entity_mut(ent).r.maxs[2] += 0.1;

        // allow to ride movers
        ctx.world.entity_mut(ent).s.groundEntityNum = (tr.entityNum) as i32;

        G_SetOrigin(ctx.world.entity_mut(ent), tr.endpos);

        if ctx.world.entity(ent).health == 0 {
            ctx.world.entity_mut(ent).health = 60;
        }

        // `None`/`""` ≡ Raven's `!ent->model || !ent->model[0]` guard.
        if ctx.world.entity(ent).model.as_deref().unwrap_or("").is_empty() {
            ctx.world.entity_mut(ent).model = Some("/models/items/a_pwr_converter.md3".to_owned());
        }

        let model = ctx.world.entity(ent).model.clone();
        let mi = G_ModelIndex(model.as_deref().unwrap_or(""));
        {
            let e = ctx.world.entity_mut(ent);
            e.s.modelindex = mi;

            e.s.eFlags = 0;
            e.r.svFlags |= SVF_PLAYER_USABLE;
            e.r.contents = CONTENTS_SOLID;
            e.clipmask = MASK_SOLID;
        }

        EnergyShieldStationSettings(ctx, ent);

        let count = ctx.world.entity(ent).count;
        {
            let e = ctx.world.entity_mut(ent);
            e.genericValue4 = count; // initial value
            e.think = Some(EntThink::check_recharge).into();
        }

        let mut nodrain: c_int = 0;
        G_SpawnInt(ctx, c"nodrain".as_ptr(), c"0".as_ptr(), &mut nodrain);
        ctx.world.entity_mut(ent).genericValue12 = nodrain;

        if ctx.world.entity(ent).genericValue12 == 0 {
            let count = ctx.world.entity(ent).count;
            let e = ctx.world.entity_mut(ent);
            e.s.maxhealth = count;
            e.s.health = count;
        }
        let level_time = ctx.world.level.time;
        {
            let e = ctx.world.entity_mut(ent);
            e.s.shouldtarget = qtrue;
            e.s.teamowner = 0;
            e.s.owner = ENTITYNUM_NONE as c_int;

            e.nextthink = level_time + 200; // + STATION_RECHARGE_TIME

            e.use_ = Some(EntUse::ammo_generic_power_converter_use).into();

            e.s.apos.trBase = e.s.angles;
        }
        let ep: *mut gentity_t = ctx.world.entity_mut(ent);
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ep.cast()));

        G_SoundIndex("sound/interface/ammocon_run");
        let snd = G_SoundIndex("sound/interface/ammocon_done");
        ctx.world.entity_mut(ent).genericValue7 = snd;
        G_SoundIndex("sound/interface/ammocon_empty");

        if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
            // show on radar from everywhere
            {
                let e = ctx.world.entity_mut(ent);
                e.r.svFlags |= SVF_BROADCAST;
                e.s.eFlags |= EF_RADAROBJECT;
            }
            let idx = G_IconIndex(ctx, "gfx/mp/siegeicons/desert/weapon_recharge");
            ctx.world.entity_mut(ent).s.genericenemyindex = idx;
        }
    }
}

/// Raven `SP_misc_shield_floor_unit`.
///
/// Source: `oracle/codemp/game/g_misc.c:1602-1687`
pub fn SP_misc_shield_floor_unit(ctx: &mut GameContext, ent: EntityId) {
    use mp_bg::public::gametype::{GT_CTF, GT_CTY, GT_SIEGE};
    use mp_qshared::shared::limits::ENTITYNUM_NONE;
    unsafe {
        let mut tr: trace_t = core::mem::zeroed();

        let gametype = ctx.world.cvars.g_gametype.integer;
        if gametype != GT_CTF && gametype != GT_CTY && gametype != GT_SIEGE {
            G_FreeEntity(ctx, Some(ent));
            return;
        }

        {
            let e = ctx.world.entity_mut(ent);
            e.r.mins = [-16.0, -16.0, 0.0];
            e.r.maxs = [16.0, 16.0, 40.0];

            // `0.1` is a bare double in the oracle; add in f64, narrow once at
            // the f32 store. Source: g_misc.c:1618-1619
            e.s.origin[2] = (e.s.origin[2] as f64 + 0.1) as f32;
            e.r.maxs[2] = (e.r.maxs[2] as f64 - 0.1) as f32;
        }

        let origin = ctx.world.entity(ent).s.origin;
        let mins = ctx.world.entity(ent).r.mins;
        let maxs = ctx.world.entity(ent).r.maxs;
        let number = ctx.world.entity(ent).s.number;
        let dest: vec3_t = [origin[0], origin[1], origin[2] - 4096.0];
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &origin as *const vec3_t,
                &mins as *const vec3_t,
                &maxs as *const vec3_t,
                &dest as *const vec3_t,
                number,
                MASK_SOLID,
            ),
        );
        if tr.startsolid != 0 {
            let msg = format!(
                "SP_misc_shield_floor_unit: misc_shield_floor_unit startsolid at {}\n",
                vtos(ctx, origin)
            );
            G_Printf(ctx, &msg);
            G_FreeEntity(ctx, Some(ent));
            return;
        }

        // add the 0.1 back after the trace (bare double; g_misc.c:1631)
        {
            let e = ctx.world.entity_mut(ent);
            e.r.maxs[2] = (e.r.maxs[2] as f64 + 0.1) as f32;

            // allow to ride movers
            e.s.groundEntityNum = (tr.entityNum) as i32;
        }

        G_SetOrigin(ctx.world.entity_mut(ent), tr.endpos);

        if ctx.world.entity(ent).health == 0 {
            ctx.world.entity_mut(ent).health = 60;
        }

        // `None`/`""` ≡ Raven's `!ent->model || !ent->model[0]` guard.
        if ctx.world.entity(ent).model.as_deref().unwrap_or("").is_empty() {
            ctx.world.entity_mut(ent).model =
                Some("/models/items/a_shield_converter.md3".to_owned());
        }

        let model = ctx.world.entity(ent).model.clone();
        let mi = G_ModelIndex(model.as_deref().unwrap_or(""));
        {
            let e = ctx.world.entity_mut(ent);
            e.s.modelindex = mi;

            e.s.eFlags = 0;
            e.r.svFlags |= SVF_PLAYER_USABLE;
            e.r.contents = CONTENTS_SOLID;
            e.clipmask = MASK_SOLID;
        }

        EnergyShieldStationSettings(ctx, ent);

        let count = ctx.world.entity(ent).count;
        {
            let e = ctx.world.entity_mut(ent);
            e.genericValue4 = count;
            e.think = Some(EntThink::check_recharge).into();
        }

        let mut nodrain: c_int = 0;
        G_SpawnInt(ctx, c"nodrain".as_ptr(), c"0".as_ptr(), &mut nodrain);
        ctx.world.entity_mut(ent).genericValue12 = nodrain;

        if ctx.world.entity(ent).genericValue12 == 0 {
            let count = ctx.world.entity(ent).count;
            let e = ctx.world.entity_mut(ent);
            e.s.maxhealth = count;
            e.s.health = count;
        }
        let level_time = ctx.world.level.time;
        {
            let e = ctx.world.entity_mut(ent);
            e.s.shouldtarget = qtrue;
            e.s.teamowner = 0;
            e.s.owner = ENTITYNUM_NONE as c_int;

            e.nextthink = level_time + 200;

            e.use_ = Some(EntUse::shield_power_converter_use).into();

            e.s.apos.trBase = e.s.angles;
        }
        let ep: *mut gentity_t = ctx.world.entity_mut(ent);
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ep.cast()));

        G_SoundIndex("sound/interface/shieldcon_run");
        let snd = G_SoundIndex("sound/interface/shieldcon_done");
        ctx.world.entity_mut(ent).genericValue7 = snd;
        G_SoundIndex("sound/interface/shieldcon_empty");

        if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
            {
                let e = ctx.world.entity_mut(ent);
                e.r.svFlags |= SVF_BROADCAST;
                e.s.eFlags |= EF_RADAROBJECT;
            }
            let idx = G_IconIndex(ctx, "gfx/mp/siegeicons/desert/shield_recharge");
            ctx.world.entity_mut(ent).s.genericenemyindex = idx;
        }
    }
}

/// Raven `SP_misc_model_shield_power_converter`.
///
/// Source: `oracle/codemp/game/g_misc.c:1697-1735`
pub fn SP_misc_model_shield_power_converter(ctx: &mut GameContext, ent: EntityId) {
    use mp_qshared::shared::limits::ENTITYNUM_NONE;

    if ctx.world.entity(ent).health == 0 {
        ctx.world.entity_mut(ent).health = 60;
    }

    {
        let e = ctx.world.entity_mut(ent);
        e.r.mins = [-16.0, -16.0, -16.0];
        e.r.maxs = [16.0, 16.0, 16.0];
    }

    let model = ctx.world.entity(ent).model.clone();
    let mi = G_ModelIndex(model.as_deref().unwrap_or(""));
    {
        let e = ctx.world.entity_mut(ent);
        e.s.modelindex = mi;

        e.s.eFlags = 0;
        e.r.svFlags |= SVF_PLAYER_USABLE;
        e.r.contents = CONTENTS_SOLID;
        e.clipmask = MASK_SOLID;
    }

    EnergyShieldStationSettings(ctx, ent);

    let count = ctx.world.entity(ent).count;
    let level_time = ctx.world.level.time;
    {
        let e = ctx.world.entity_mut(ent);
        e.genericValue4 = count;
        e.think = Some(EntThink::check_recharge).into();

        e.s.maxhealth = count;
        e.s.health = count;
        e.s.shouldtarget = qtrue;
        e.s.teamowner = 0;
        e.s.owner = ENTITYNUM_NONE as c_int;

        e.nextthink = level_time + 200;

        e.use_ = Some(EntUse::shield_power_converter_use).into();
    }

    let origin = ctx.world.entity(ent).s.origin;
    G_SetOrigin(ctx.world.entity_mut(ent), origin);
    let angles = ctx.world.entity(ent).s.angles;
    ctx.world.entity_mut(ent).s.apos.trBase = angles;
    let ep: *mut gentity_t = ctx.world.entity_mut(ent);
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ep.cast()));

    //G_SoundIndex("sound/movers/objects/useshieldstation.wav");

    let mi2 = G_ModelIndex("/models/items/psd_big.md3");
    ctx.world.entity_mut(ent).s.modelindex2 = mi2;
    // Precache model
}

/// Raven `EnergyAmmoStationSettings`.
///
/// Source: `oracle/codemp/game/g_misc.c:1743-1746`
pub fn EnergyAmmoStationSettings(ctx: &mut GameContext, ent: EntityId) {
    let mut count: c_int = 0;
    G_SpawnInt(ctx, c"count".as_ptr(), c"200".as_ptr(), &mut count);
    ctx.world.entity_mut(ent).count = count;
}

/// Raven `ammo_power_converter_use`.
///
/// Source: `oracle/codemp/game/g_misc.c:1753-1853`
pub fn ammo_power_converter_use(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    use mp_bg::weapons::ammo_t::ammo_t::{AMMO_BLASTER, AMMO_MAX};

    let mut stop = true;

    let activator = match activator {
        Some(a) => a,
        None => return,
    };
    // FLAG (task #7): activator->client is a player/NPC-pool gclient_t ptr; ps
    // ammo dereffed raw exactly as Raven does (copied pointer value).
    let acl = ctx.world.entity(activator).client;
    if acl.is_null() {
        return;
    }

    let set_time = ctx.world.entity(self_).setTime;
    let level_time = ctx.world.level.time;
    if set_time < level_time {
        if ctx.world.entity(self_).s.loopSound == 0 {
            let snd = G_SoundIndex("sound/player/pickupshield.wav");
            ctx.world.entity_mut(self_).s.loopSound = snd;
        }

        ctx.world.entity_mut(self_).setTime = level_time + 100;

        if ctx.world.entity(self_).count != 0 {
            // Has it got any power left?
            let mut i = AMMO_BLASTER as c_int;
            let mut add: c_int = 0;
            while i < AMMO_MAX as c_int {
                add = (ammoData[i as usize].max as f32 * 0.1) as c_int;
                if add < 1 {
                    add = 1;
                }
                let max = ammoData[i as usize].max;
                let ammo_i = unsafe { (*acl).ps.ammo[i as usize] };
                if ammo_i < max {
                    unsafe {
                        (*acl).ps.ammo[i as usize] += add;
                    }
                    if unsafe { (*acl).ps.ammo[i as usize] } > max {
                        unsafe {
                            (*acl).ps.ammo[i as usize] = max;
                        }
                    }
                }
                i += 1;
            }
            if ctx.world.entity(self_).genericValue12 == 0 {
                ctx.world.entity_mut(self_).count -= add;
            }
            stop = false;

            ctx.world.entity_mut(self_).fly_sound_debounce_time = level_time + 500;
            ctx.world.entity_mut(self_).activator = Some(activator);
        }
    }

    if stop {
        let e = ctx.world.entity_mut(self_);
        e.s.loopSound = 0;
        e.s.loopIsSoundset = qfalse;
    }
}

/// Raven `SP_misc_model_ammo_power_converter`.
///
/// Source: `oracle/codemp/game/g_misc.c:1864-1904`
pub fn SP_misc_model_ammo_power_converter(ctx: &mut GameContext, ent: EntityId) {
    use mp_qshared::shared::limits::ENTITYNUM_NONE;

    if ctx.world.entity(ent).health == 0 {
        ctx.world.entity_mut(ent).health = 60;
    }

    {
        let e = ctx.world.entity_mut(ent);
        e.r.mins = [-16.0, -16.0, -16.0];
        e.r.maxs = [16.0, 16.0, 16.0];
    }

    let model = ctx.world.entity(ent).model.clone();
    let mi = G_ModelIndex(model.as_deref().unwrap_or(""));
    {
        let e = ctx.world.entity_mut(ent);
        e.s.modelindex = mi;

        e.s.eFlags = 0;
        e.r.svFlags |= SVF_PLAYER_USABLE;
        e.r.contents = CONTENTS_SOLID;
        e.clipmask = MASK_SOLID;
    }

    let mut nodrain: c_int = 0;
    G_SpawnInt(ctx, c"nodrain".as_ptr(), c"0".as_ptr(), &mut nodrain);
    {
        let e = ctx.world.entity_mut(ent);
        e.genericValue12 = nodrain;
        e.use_ = Some(EntUse::ammo_power_converter_use).into();
    }

    EnergyAmmoStationSettings(ctx, ent);

    let count = ctx.world.entity(ent).count;
    {
        let e = ctx.world.entity_mut(ent);
        e.genericValue4 = count;
        e.think = Some(EntThink::check_recharge).into();
    }

    if ctx.world.entity(ent).genericValue12 == 0 {
        let count = ctx.world.entity(ent).count;
        let e = ctx.world.entity_mut(ent);
        e.s.maxhealth = count;
        e.s.health = count;
    }
    let level_time = ctx.world.level.time;
    {
        let e = ctx.world.entity_mut(ent);
        e.s.shouldtarget = qtrue;
        e.s.teamowner = 0;
        e.s.owner = ENTITYNUM_NONE as c_int;

        e.nextthink = level_time + 200;
    }

    let origin = ctx.world.entity(ent).s.origin;
    G_SetOrigin(ctx.world.entity_mut(ent), origin);
    let angles = ctx.world.entity(ent).s.angles;
    ctx.world.entity_mut(ent).s.apos.trBase = angles;
    let ep: *mut gentity_t = ctx.world.entity_mut(ent);
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ep.cast()));

    //G_SoundIndex("sound/movers/objects/useshieldstation.wav");
}

/// Raven `EnergyHealthStationSettings`.
///
/// Source: `oracle/codemp/game/g_misc.c:1911-1914`
pub fn EnergyHealthStationSettings(ctx: &mut GameContext, ent: EntityId) {
    let mut count: c_int = 0;
    G_SpawnInt(ctx, c"count".as_ptr(), c"200".as_ptr(), &mut count);
    ctx.world.entity_mut(ent).count = count;
}

/// Raven `health_power_converter_use`.
///
/// Source: `oracle/codemp/game/g_misc.c:1921-1972`
pub fn health_power_converter_use(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    use mp_bg::public::stat_index::statIndex_t::STAT_MAX_HEALTH;
    use mp_qshared::shared::sound_channel::CHAN_AUTO;

    let mut stop = true;

    let activator = match activator {
        Some(a) => a,
        None => return,
    };
    // FLAG (task #7): activator->client is a player/NPC-pool gclient_t ptr; ps
    // stats dereffed raw exactly as Raven does (copied pointer value).
    let acl = ctx.world.entity(activator).client;
    if acl.is_null() {
        return;
    }

    let set_time = ctx.world.entity(self_).setTime;
    let level_time = ctx.world.level.time;
    if set_time < level_time {
        if ctx.world.entity(self_).s.loopSound == 0 {
            let snd = G_SoundIndex("sound/player/pickuphealth.wav");
            ctx.world.entity_mut(self_).s.loopSound = snd;
        }
        ctx.world.entity_mut(self_).setTime = level_time + 100;

        let max_health = unsafe { (*acl).ps.stats[STAT_MAX_HEALTH as usize] };
        let dif = max_health - ctx.world.entity(activator).health;

        if dif > 0 {
            let mut add = if dif > 5 { 5 } else { dif };
            if ctx.world.entity(self_).count < add {
                add = ctx.world.entity(self_).count;
            }

            stop = false;

            ctx.world.entity_mut(self_).fly_sound_debounce_time = level_time + 500;
            ctx.world.entity_mut(self_).activator = Some(activator);

            ctx.world.entity_mut(activator).health += add;
        }
    }

    if stop {
        let e = ctx.world.entity_mut(self_);
        e.s.loopSound = 0;
        e.s.loopIsSoundset = qfalse;
    }
}

/// Raven `SP_misc_model_health_power_converter`.
///
/// Source: `oracle/codemp/game/g_misc.c:1982-2027`
pub fn SP_misc_model_health_power_converter(ctx: &mut GameContext, ent: EntityId) {
    use mp_bg::public::gametype::GT_SIEGE;
    use mp_qshared::shared::limits::ENTITYNUM_NONE;

    if ctx.world.entity(ent).health == 0 {
        ctx.world.entity_mut(ent).health = 60;
    }

    {
        let e = ctx.world.entity_mut(ent);
        e.r.mins = [-16.0, -16.0, -16.0];
        e.r.maxs = [16.0, 16.0, 16.0];
    }

    let model = ctx.world.entity(ent).model.clone();
    let mi = G_ModelIndex(model.as_deref().unwrap_or(""));
    {
        let e = ctx.world.entity_mut(ent);
        e.s.modelindex = mi;

        e.s.eFlags = 0;
        e.r.svFlags |= SVF_PLAYER_USABLE;
        e.r.contents = CONTENTS_SOLID;
        e.clipmask = MASK_SOLID;

        e.use_ = Some(EntUse::health_power_converter_use).into();
    }

    EnergyHealthStationSettings(ctx, ent);

    let count = ctx.world.entity(ent).count;
    let level_time = ctx.world.level.time;
    {
        let e = ctx.world.entity_mut(ent);
        e.genericValue4 = count;
        e.think = Some(EntThink::check_recharge).into();

        //ent->s.maxhealth = ent->s.health = ent->count;
        e.s.shouldtarget = qtrue;
        e.s.teamowner = 0;
        e.s.owner = ENTITYNUM_NONE as c_int;

        e.nextthink = level_time + 200;
    }

    let origin = ctx.world.entity(ent).s.origin;
    G_SetOrigin(ctx.world.entity_mut(ent), origin);
    let angles = ctx.world.entity(ent).s.angles;
    ctx.world.entity_mut(ent).s.apos.trBase = angles;
    let ep: *mut gentity_t = ctx.world.entity_mut(ent);
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ep.cast()));

    //G_SoundIndex("sound/movers/objects/useshieldstation.wav");
    G_SoundIndex("sound/player/pickuphealth.wav");
    let snd = G_SoundIndex("sound/interface/shieldcon_done");
    ctx.world.entity_mut(ent).genericValue7 = snd;

    if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
        // show on radar from everywhere
        {
            let e = ctx.world.entity_mut(ent);
            e.r.svFlags |= SVF_BROADCAST;
            e.s.eFlags |= EF_RADAROBJECT;
        }
        let idx = G_IconIndex(ctx, "gfx/mp/siegeicons/desert/bacta");
        ctx.world.entity_mut(ent).s.genericenemyindex = idx;
    }
}

/// Raven `fx_runner_think`.
///
/// Source: `oracle/codemp/game/g_misc.c:2266-2310`
pub fn fx_runner_think(ctx: &mut GameContext, ent: EntityId) {
    let pos = ctx.world.entity(ent).s.pos;
    let level_time = ctx.world.level.time;
    mp_bg::bg_misc::BG_EvaluateTrajectory(
        &pos,
        level_time,
        &mut ctx.world.entity_mut(ent).r.currentOrigin,
    );
    let apos = ctx.world.entity(ent).s.apos;
    mp_bg::bg_misc::BG_EvaluateTrajectory(
        &apos,
        level_time,
        &mut ctx.world.entity_mut(ent).r.currentAngles,
    );

    // call the effect with the desired position and orientation
    if ctx.world.entity(ent).s.isPortalEnt != 0 {
        //		G_AddEvent( ent, EV_PLAY_PORTAL_EFFECT_ID, ent->genericValue5 );
    } else {
        //		G_AddEvent( ent, EV_PLAY_EFFECT_ID, ent->genericValue5 );
    }

    // start the fx on the client (continuous)
    ctx.world.entity_mut(ent).s.modelindex2 = FX_STATE_CONTINUOUS;

    let ca = ctx.world.entity(ent).r.currentAngles;
    ctx.world.entity_mut(ent).s.angles = ca;
    let co = ctx.world.entity(ent).r.currentOrigin;
    ctx.world.entity_mut(ent).s.origin = co;

    let level_time = ctx.world.level.time;
    let delay = ctx.world.entity(ent).delay;
    let random = ctx.world.entity(ent).random;
    let rnd = ctx.world.bg_state.rng.random();
    ctx.world.entity_mut(ent).nextthink = level_time + delay + (rnd * random) as c_int;

    if ctx.world.entity(ent).spawnflags & 4 != 0 {
        // damage
        let co = ctx.world.entity(ent).r.currentOrigin;
        let sd = ctx.world.entity(ent).splashDamage as f32;
        let sr = ctx.world.entity(ent).splashRadius as f32;
        G_RadiusDamage(
            ctx,
            co,
            Some(ent),
            sd,
            sr,
            Some(ent),
            Some(ent),
            MOD_UNKNOWN as c_int,
        );
    }

    let target2 = ctx.world.entity(ent).target2.clone();
    if target2.as_deref().is_some_and(|s| !s.is_empty()) {
        // let our target know that we have spawned an effect
        G_UseTargets2(ctx, Some(ent), Some(ent), target2.as_deref());
    }

    if ctx.world.entity(ent).spawnflags & 2 == 0 && ctx.world.entity(ent).s.loopSound == 0 {
        // NOT ONESHOT...this is an assy thing to do
        let sound_set = ctx.world.entity(ent).soundSet.clone();
        if !sound_set.is_empty() {
            let ssi = G_SoundSetIndex(ctx, &sound_set);
            let e = ctx.world.entity_mut(ent);
            e.s.soundSetIndex = ssi;
            e.s.loopIsSoundset = qtrue;
            e.s.loopSound = BMS_MID;
        }
    }
}

/// Raven `fx_runner_use`.
///
/// Source: `oracle/codemp/game/g_misc.c:2313-2384`
pub fn fx_runner_use(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    if ctx.world.entity(self_).s.isPortalEnt != 0 {
        // rww - mark it as broadcast upon first use if it's within the area of a skyportal
        ctx.world.entity_mut(self_).r.svFlags |= SVF_BROADCAST;
    }

    if ctx.world.entity(self_).spawnflags & 2 != 0 {
        // ONESHOT
        // call the effect with the desired position and orientation, as a safety thing,
        //	make sure we aren't thinking at all.
        let save_state = ctx.world.entity(self_).s.modelindex2 + 1;

        fx_runner_think(ctx, self_);
        ctx.world.entity_mut(self_).nextthink = -1;
        // one shot indicator
        ctx.world.entity_mut(self_).s.modelindex2 = save_state;
        if ctx.world.entity(self_).s.modelindex2 > FX_STATE_ONE_SHOT_LIMIT {
            ctx.world.entity_mut(self_).s.modelindex2 = FX_STATE_ONE_SHOT;
        }

        let target2 = ctx.world.entity(self_).target2.clone();
        if target2.is_some() {
            // let our target know that we have spawned an effect
            G_UseTargets2(ctx, Some(self_), Some(self_), target2.as_deref());
        }

        let sound_set = ctx.world.entity(self_).soundSet.clone();
        if !sound_set.is_empty() {
            let ssi = G_SoundSetIndex(ctx, &sound_set);
            ctx.world.entity_mut(self_).s.soundSetIndex = ssi;
            G_AddEvent(
                ctx.world.entity_mut(self_),
                mp_bg::public::entity_event::entity_event_t::EV_BMODEL_SOUND as c_int,
                BMS_START,
            );
        }
    } else {
        // ensure we are working with the right think function
        ctx.world.entity_mut(self_).think = Some(EntThink::fx_runner_think).into();

        // toggle our state
        if ctx.world.entity(self_).nextthink == -1 {
            // NOTE: we fire the effect immediately on use, the fx_runner_think func will set
            //	up the nextthink time.
            fx_runner_think(ctx, self_);

            let sound_set = ctx.world.entity(self_).soundSet.clone();
            if !sound_set.is_empty() {
                let ssi = G_SoundSetIndex(ctx, &sound_set);
                ctx.world.entity_mut(self_).s.soundSetIndex = ssi;
                G_AddEvent(
                    ctx.world.entity_mut(self_),
                    mp_bg::public::entity_event::entity_event_t::EV_BMODEL_SOUND as c_int,
                    BMS_START,
                );
                let e = ctx.world.entity_mut(self_);
                e.s.loopSound = BMS_MID;
                e.s.loopIsSoundset = qtrue;
            }
        } else {
            // turn off for now
            ctx.world.entity_mut(self_).nextthink = -1;

            // turn off fx on client
            ctx.world.entity_mut(self_).s.modelindex2 = FX_STATE_OFF;

            let sound_set = ctx.world.entity(self_).soundSet.clone();
            if !sound_set.is_empty() {
                let ssi = G_SoundSetIndex(ctx, &sound_set);
                ctx.world.entity_mut(self_).s.soundSetIndex = ssi;
                G_AddEvent(
                    ctx.world.entity_mut(self_),
                    mp_bg::public::entity_event::entity_event_t::EV_BMODEL_SOUND as c_int,
                    BMS_END,
                );
                let e = ctx.world.entity_mut(self_);
                e.s.loopSound = 0;
                e.s.loopIsSoundset = qfalse;
            }
        }
    }
}

/// Raven `fx_runner_link`.
///
/// Source: `oracle/codemp/game/g_misc.c:2387-2453`
pub fn fx_runner_link(ctx: &mut GameContext, ent: EntityId) {
    let mut dir: vec3_t;

    let target_field = ctx.world.entity(ent).target.clone();
    if target_field.as_deref().is_some_and(|s| !s.is_empty()) {
        // try to use the target to override the orientation
        let target = G_Find(ctx, None, EntFindField::Targetname, target_field.as_deref().unwrap());

        if target.is_null() {
            // Bah, no good, dump a warning, but continue on and use the UP vector
            Com_Printf(&format!(
                "fx_runner_link: target specified but not found: {}\n",
                target_field.as_deref().unwrap()
            ));
            Com_Printf("  -assuming UP orientation.\n");
        } else {
            // Our target is valid so let's override the default UP vector
            let target_id = ctx.entity_id_of(target).unwrap();
            let target_origin = ctx.world.entity(target_id).s.origin;
            let ent_origin = ctx.world.entity(ent).s.origin;
            let mut d: vec3_t = [0.0, 0.0, 0.0];
            crate::q_math::_VectorSubtract(target_origin, ent_origin, &mut d);
            VectorNormalize(&mut d);
            vectoangles(d, &mut ctx.world.entity_mut(ent).s.angles);
        }
    }

    // don't really do anything with this right now other than do a check to warn the designers if the target2 is bogus
    let target2_field = ctx.world.entity(ent).target2.clone();
    if target2_field.as_deref().is_some_and(|s| !s.is_empty()) {
        let target =
            G_Find(ctx, None, EntFindField::Targetname, target2_field.as_deref().unwrap());

        if target.is_null() {
            // Target2 is bogus, but we can still continue
            Com_Printf(&format!(
                "fx_runner_link: target2 was specified but is not valid: {}\n",
                target2_field.as_deref().unwrap()
            ));
        }
    }

    let angles = ctx.world.entity(ent).s.angles;
    G_SetAngles(ctx.world.entity_mut(ent), angles);

    if ctx.world.entity(ent).spawnflags & 1 != 0 || ctx.world.entity(ent).spawnflags & 2 != 0 {
        // STARTOFF || ONESHOT
        // We won't even consider thinking until we are used
        ctx.world.entity_mut(ent).nextthink = -1;
    } else {
        let sound_set = ctx.world.entity(ent).soundSet.clone();
        if !sound_set.is_empty() {
            let ssi = G_SoundSetIndex(ctx, &sound_set);
            let e = ctx.world.entity_mut(ent);
            e.s.soundSetIndex = ssi;
            e.s.loopSound = BMS_MID;
            e.s.loopIsSoundset = qtrue;
        }

        // Let's get to work right now!
        let level_time = ctx.world.level.time;
        let e = ctx.world.entity_mut(ent);
        e.think = Some(EntThink::fx_runner_think).into();
        e.nextthink = level_time + 200; // wait a small bit, then start working
    }

    // make us useable if we can be targeted
    let targetname = ctx.world.entity(ent).targetname_str();
    if targetname.as_deref().is_some_and(|s| !s.is_empty()) {
        ctx.world.entity_mut(ent).use_ = Some(EntUse::fx_runner_use).into();
    }
}

/// Raven `SP_fx_runner`.
///
/// Source: `oracle/codemp/game/g_misc.c:2456-2501`
pub fn SP_fx_runner(ctx: &mut GameContext, ent: EntityId) {
    let (_, fx_file) = G_SpawnString(ctx, "fxFile", "");
    // Get our defaults
    let mut delay: c_int = 0;
    G_SpawnInt(ctx, c"delay".as_ptr(), c"200".as_ptr(), &mut delay);
    ctx.world.entity_mut(ent).delay = delay;
    let mut random: f32 = 0.0;
    G_SpawnFloat(ctx, c"random".as_ptr(), c"0".as_ptr(), &mut random);
    ctx.world.entity_mut(ent).random = random;
    let mut splash_radius: c_int = 0;
    G_SpawnInt(
        ctx,
        c"splashRadius".as_ptr(),
        c"16".as_ptr(),
        &mut splash_radius,
    );
    ctx.world.entity_mut(ent).splashRadius = splash_radius;
    let mut splash_damage: c_int = 0;
    G_SpawnInt(
        ctx,
        c"splashDamage".as_ptr(),
        c"5".as_ptr(),
        &mut splash_damage,
    );
    ctx.world.entity_mut(ent).splashDamage = splash_damage;

    {
        let e = ctx.world.entity_mut(ent);
        if e.s.angles[0] == 0.0 && e.s.angles[1] == 0.0 && e.s.angles[2] == 0.0 {
            // didn't have angles, so give us the default of up
            e.s.angles = [-90.0, 0.0, 0.0];
        }
    }

    if fx_file.is_empty() {
        let targetname = ctx.world.entity(ent).targetname_str();
        let origin = ctx.world.entity(ent).s.origin;
        Com_Printf(&format!(
            "^1ERROR: fx_runner {} at {} has no fxFile specified\n",
            targetname.as_deref().unwrap_or(""),
            vtos(ctx, origin)
        ));
        G_FreeEntity(ctx, Some(ent));
        return;
    }

    // Try and associate an effect file, unfortunately we won't know if this worked or not
    //	until the CGAME trys to register it...
    let mi = G_EffectIndex(&fx_file);
    let level_time = ctx.world.level.time;
    {
        let e = ctx.world.entity_mut(ent);
        e.s.modelindex = mi;

        // important info transmitted
        e.s.eType = entityType_t::ET_FX as c_int;
        e.s.speed = e.delay as f32;
        e.s.time = e.random as c_int;
        e.s.modelindex2 = FX_STATE_OFF;

        // Give us a bit of time to spawn in the other entities, since we may have to target one of 'em
        e.think = Some(EntThink::fx_runner_link).into();
        e.nextthink = level_time + 400;
    }

    // Save our position and link us up!
    let origin = ctx.world.entity(ent).s.origin;
    G_SetOrigin(ctx.world.entity_mut(ent), origin);

    ctx.world.entity_mut(ent).r.maxs = [FX_ENT_RADIUS, FX_ENT_RADIUS, FX_ENT_RADIUS];
    let maxs = ctx.world.entity(ent).r.maxs;
    crate::q_math::_VectorScale(maxs, -1.0, &mut ctx.world.entity_mut(ent).r.mins);

    let ep: *mut gentity_t = ctx.world.entity_mut(ent);
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ep.cast()));
}

/// Raven `SP_CreateSpaceDust`.
///
/// Source: `oracle/codemp/game/g_misc.c:2509-2513`
pub fn SP_CreateSpaceDust(ctx: &mut GameContext, ent: EntityId) {
    let count = ctx.world.entity(ent).count;
    G_EffectIndex(&format!("*spacedust {}", count));
    //G_EffectIndex("*constantwind ( 10 -10 0 )");
}

/// Raven `SP_CreateSnow`.
///
/// Source: `oracle/codemp/game/g_misc.c:2522-2527`
pub fn SP_CreateSnow(ctx: &mut GameContext, ent: EntityId) {
    G_EffectIndex("*snow");
    G_EffectIndex("*fog");
    G_EffectIndex("*constantwind (100 100 -100)");
}

/// Raven `SP_CreateRain`.
///
/// Source: `oracle/codemp/game/g_misc.c:2535-2538`
pub fn SP_CreateRain(ctx: &mut GameContext, ent: EntityId) {
    let count = ctx.world.entity(ent).count;
    G_EffectIndex(&format!("*rain init {}", count));
}

/// Raven `Use_Target_Screenshake`.
///
/// Source: `oracle/codemp/game/g_misc.c:2543-2553`
pub fn Use_Target_Screenshake(
    ctx: &mut GameContext,
    ent: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    let gv6 = ctx.world.entity(ent).genericValue6;
    let bGlobal: qboolean = if gv6 != 0 { qtrue } else { qfalse };
    let origin = ctx.world.entity(ent).s.origin;
    let speed = ctx.world.entity(ent).speed;
    let gv5 = ctx.world.entity(ent).genericValue5;
    G_ScreenShake(ctx, origin, None, speed, gv5, bGlobal);
}

/// Raven `SP_target_screenshake`.
///
/// Source: `oracle/codemp/game/g_misc.c:2555-2565`
pub fn SP_target_screenshake(ctx: &mut GameContext, ent: EntityId) {
    let mut speed: f32 = 0.0;
    G_SpawnFloat(ctx, c"intensity".as_ptr(), c"10".as_ptr(), &mut speed);
    ctx.world.entity_mut(ent).speed = speed;
    let mut duration: c_int = 0;
    G_SpawnInt(ctx, c"duration".as_ptr(), c"800".as_ptr(), &mut duration);
    ctx.world.entity_mut(ent).genericValue5 = duration;
    let mut globalshake: c_int = 0;
    G_SpawnInt(
        ctx,
        c"globalshake".as_ptr(),
        c"1".as_ptr(),
        &mut globalshake,
    );
    ctx.world.entity_mut(ent).genericValue6 = globalshake;

    ctx.world.entity_mut(ent).use_ = Some(EntUse::Use_Target_Screenshake).into();
}

/// Raven `Use_Target_Escapetrig`.
///
/// Source: `oracle/codemp/game/g_misc.c:2569-2597`
pub fn Use_Target_Escapetrig(
    ctx: &mut GameContext,
    ent: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    use mp_bg::public::team::TEAM_SPECTATOR;

    let gv6 = ctx.world.entity(ent).genericValue6;
    if gv6 == 0 {
        ctx.world.globals.gEscaping = qtrue;
        let level_time = ctx.world.level.time;
        let gv5 = ctx.world.entity(ent).genericValue5;
        ctx.world.globals.gEscapeTime = level_time + gv5;
    } else if ctx.world.globals.gEscaping != qfalse {
        let mut i: c_int = 0;
        ctx.world.globals.gEscaping = qfalse;
        while i < mp_qshared::shared::MAX_CLIENTS_I32 {
            let e_id = EntityId(i as u32);
            // FLAG (task #7): client-slot pool `gclient_t` — dereffed raw exactly
            // as Raven does (copied pointer value); i < MAX_CLIENTS is a real slot.
            let c = ctx.world.entity(e_id).client;
            let inuse = ctx.world.entity(e_id).inuse;
            let health = ctx.world.entity(e_id).health;
            if inuse != 0
                && !c.is_null()
                && health > 0
                && unsafe { (*c).sess.sessionTeam } != TEAM_SPECTATOR
                && unsafe { (*c).ps.pm_flags } & PMF_FOLLOW == 0
            {
                // all of the survivors get 100 points!
                let ps_origin = unsafe { (*c).ps.origin };
                AddScore(ctx, e_id, ps_origin, 100);
            }
            i += 1;
        }
        if let Some(a) = activator {
            let c = ctx.world.entity(a).client;
            if ctx.world.entity(a).inuse != 0 && !c.is_null() {
                // the one who escaped gets 500
                let ps_origin = unsafe { (*c).ps.origin };
                AddScore(ctx, a, ps_origin, 500);
            }
        }

        LogExit(ctx, "Escaped!");
    }
}

/// Raven `SP_target_escapetrig`.
///
/// Source: `oracle/codemp/game/g_misc.c:2599-2613`
pub fn SP_target_escapetrig(ctx: &mut GameContext, ent: EntityId) {
    use mp_bg::public::gametype::GT_SINGLE_PLAYER;

    if ctx.world.cvars.g_gametype.integer != GT_SINGLE_PLAYER {
        G_FreeEntity(ctx, Some(ent));
        return;
    }

    let mut escapetime: c_int = 0;
    G_SpawnInt(
        ctx,
        c"escapetime".as_ptr(),
        c"60000".as_ptr(),
        &mut escapetime,
    );
    ctx.world.entity_mut(ent).genericValue5 = escapetime;
    let mut escapegoal: c_int = 0;
    G_SpawnInt(ctx, c"escapegoal".as_ptr(), c"0".as_ptr(), &mut escapegoal);
    ctx.world.entity_mut(ent).genericValue6 = escapegoal;

    ctx.world.entity_mut(ent).use_ = Some(EntUse::Use_Target_Escapetrig).into();
}

/// Raven `maglock_die`.
///
/// Unlocks our door if we're the last lock pointed at the door, then fires
/// this maglock's targets. `WP_Explode` was already dead code upstream
/// (`//rwwFIXMEFIXME - weap expl func`).
/// Source: `oracle/codemp/game/g_misc.c:2623-2640`
pub fn maglock_die(
    ctx: &mut GameContext,
    self_: EntityId,
    inflictor: Option<EntityId>,
    attacker: Option<EntityId>,
    damage: c_int,
    r#mod: c_int,
) {
    use crate::entity::flags::FL_INACTIVE;

    if let Some(door_id) = ctx.world.entity(self_).activator {
        ctx.world.entity_mut(door_id).lockCount -= 1;
        if ctx.world.entity(door_id).lockCount == 0 {
            ctx.world.entity_mut(door_id).flags &= !FL_INACTIVE;
        }
    }
    G_UseTargets(ctx, Some(self_), attacker);
}

/// Raven `SP_misc_maglock`.
///
/// Source: `oracle/codemp/game/g_misc.c:2645-2658`
pub fn SP_misc_maglock(ctx: &mut GameContext, self_: EntityId) {
    // NOTE: May have to make these only work on doors that are either untargeted
    //		or are targeted by a trigger, not doors fired off by scripts, counters
    //		or other such things?
    let mi = G_ModelIndex("models/map_objects/imp_detention/door_lock.md3");
    ctx.world.entity_mut(self_).s.modelindex = mi;
    let ei = G_EffectIndex("maglock/explosion");
    ctx.world.entity_mut(self_).genericValue1 = ei;

    let origin = ctx.world.entity(self_).s.origin;
    G_SetOrigin(ctx.world.entity_mut(self_), origin);

    let level_time = ctx.world.level.time;
    let e = ctx.world.entity_mut(self_);
    e.think = Some(EntThink::maglock_link).into();
    //FIXME: for some reason, when you re-load a level, these fail to find their doors...?  Random?  Testing an additional 200ms after the START_TIME_FIND_LINKS
    e.nextthink = level_time + START_TIME_FIND_LINKS + 200;
    //because we need to let the doors link up and spawn their triggers first!
}

/// Raven `maglock_link`.
///
/// Source: `oracle/codemp/game/g_misc.c:2659-2728`
pub fn maglock_link(ctx: &mut GameContext, self_: EntityId) {
    use crate::q_math::vectoangles;
    use mp_qshared::shared::error_parm::errorParm_t::ERR_DROP;
    use mp_qshared::shared::limits::ENTITYNUM_WORLD;

    // find what we're supposed to be attached to
    let mut forward: vec3_t = [0.0, 0.0, 0.0];
    let mut start: vec3_t = [0.0, 0.0, 0.0];
    let mut end: vec3_t = [0.0, 0.0, 0.0];
    let mut trace: trace_t = unsafe { core::mem::zeroed() };

    let angles = ctx.world.entity(self_).s.angles;
    AngleVectors(angles, Some(&mut forward), None, None);
    let origin = ctx.world.entity(self_).s.origin;
    crate::q_math::_VectorMA(origin, 128.0, forward, &mut end);
    crate::q_math::_VectorMA(origin, -4.0, forward, &mut start);

    let number = ctx.world.entity(self_).s.number;
    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut trace as *mut trace_t,
            &start as *const vec3_t,
            &vec3_origin as *const vec3_t,
            &vec3_origin as *const vec3_t,
            &end as *const vec3_t,
            number,
            MASK_SHOT,
        ),
    );

    if trace.allsolid != 0 || trace.startsolid != 0 {
        let origin = ctx.world.entity(self_).s.origin;
        crate::g_main::Com_Error(
            ERR_DROP as c_int,
            cstr(&format!("misc_maglock at {} in solid\n", unsafe {
                vtos(ctx, origin)
            }))
            .as_ptr(),
        );
        G_FreeEntity(ctx, Some(self_));
        return;
    }
    if trace.fraction == 1.0 {
        let level_time = ctx.world.level.time;
        let e = ctx.world.entity_mut(self_);
        e.think = Some(EntThink::maglock_link).into();
        e.nextthink = level_time + 100;
        return;
    }
    let trace_ent = EntityId(trace.entityNum as u32);
    let is_bad = trace.entityNum >= (ENTITYNUM_WORLD as c_int) as i16 || {
        Q_stricmp("func_door", &ctx.world.entity(trace_ent).classname_str()) != 0
    };
    if is_bad {
        let level_time = ctx.world.level.time;
        let e = ctx.world.entity_mut(self_);
        e.think = Some(EntThink::maglock_link).into();
        e.nextthink = level_time + 100;
        return;
    }

    // check the traceEnt, make sure it's a door and give it a lockCount and deactivate it
    // find the trigger for the door
    let door_trigger = G_FindDoorTrigger(ctx, trace_ent);
    let activator_id = if !door_trigger.is_null() {
        ctx.entity_id_of(door_trigger).unwrap()
    } else {
        trace_ent
    };
    ctx.world.entity_mut(self_).activator = Some(activator_id);
    ctx.world.entity_mut(activator_id).lockCount += 1;
    ctx.world.entity_mut(activator_id).flags |= FL_INACTIVE;

    // now position and orient it
    vectoangles(trace.plane.normal, &mut end);
    let endpos = trace.endpos;
    G_SetOrigin(ctx.world.entity_mut(self_), endpos);
    G_SetAngles(ctx.world.entity_mut(self_), end);

    {
        let e = ctx.world.entity_mut(self_);
        // make it hittable
        e.r.mins = [-8.0, -8.0, -8.0];
        e.r.maxs = [8.0, 8.0, 8.0];
        e.r.contents = CONTENTS_CORPSE;

        // make it destroyable
        e.flags |= FL_SHIELDED; // only damagable by lightsabers
        e.takedamage = qtrue;
        e.health = 10;
        e.die = Some(EntDie::maglock_die).into();
    }

    let ep: *mut gentity_t = ctx.world.entity_mut(self_);
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ep.cast()));
}

/// Raven `faller_touch`.
///
/// Source: `oracle/codemp/game/g_misc.c:2730-2756`
pub fn faller_touch(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    trace: *mut trace_t,
) {
    use mp_qshared::shared::sound_channel::{CHAN_AUTO, CHAN_VOICE};

    let epv2 = ctx.world.entity(self_).epVelocity[2];
    let gv7 = ctx.world.entity(self_).genericValue7;
    let level_time = ctx.world.level.time;
    if epv2 < -100.0 && gv7 < level_time {
        let r = ctx.world.bg_state.rng.Q_irand(1, 3);

        let snd = if r == 1 {
            G_SoundIndex("sound/chars/stofficer1/misc/pain25")
        } else if r == 2 {
            G_SoundIndex("sound/chars/stofficer1/misc/pain50")
        } else {
            G_SoundIndex("sound/chars/stofficer1/misc/pain75")
        };
        ctx.world.entity_mut(self_).genericValue11 = snd;

        let gv11 = ctx.world.entity(self_).genericValue11;
        G_EntitySound(ctx, self_, CHAN_VOICE as c_int, gv11);
        let gv10 = ctx.world.entity(self_).genericValue10;
        G_EntitySound(ctx, self_, CHAN_AUTO as c_int, gv10);

        let level_time = ctx.world.level.time;
        ctx.world.entity_mut(self_).genericValue6 = level_time + 3000;
        ctx.world.entity_mut(self_).genericValue7 = level_time + 200;
    }
}

/// Raven `faller_think`.
///
/// Source: `oracle/codemp/game/g_misc.c:2758-2787`
pub fn faller_think(ctx: &mut GameContext, ent: EntityId) {
    use mp_qshared::shared::sound_channel::CHAN_VOICE;

    let gravity: f32 = 3.0;
    let mass: f32 = 0.09;
    let bounce: f32 = 1.1;

    let gv6 = ctx.world.entity(ent).genericValue6;
    let level_time = ctx.world.level.time;
    if gv6 < level_time {
        let e = ctx.world.entity_mut(ent);
        e.think = Some(EntThink::G_FreeEntity).into();
        e.nextthink = level_time;
        return;
    }

    if ctx.world.entity(ent).epVelocity[2] < -100.0 {
        if ctx.world.entity(ent).genericValue8 == 0 {
            let gv9 = ctx.world.entity(ent).genericValue9;
            G_EntitySound(ctx, ent, CHAN_VOICE as c_int, gv9);
            ctx.world.entity_mut(ent).genericValue8 = 1;
        }
    } else {
        ctx.world.entity_mut(ent).genericValue8 = 0;
    }

    G_RunExPhys(
        ctx,
        ent,
        gravity,
        mass,
        bounce,
        true,
        core::ptr::null_mut(),
        0,
    );
    let epv = ctx.world.entity(ent).epVelocity;
    ctx.world.entity_mut(ent).s.pos.trDelta = [epv[0] * 10.0, epv[1] * 10.0, epv[2] * 10.0];
    let level_time = ctx.world.level.time;
    ctx.world.entity_mut(ent).nextthink = level_time + 25;
}

/// Raven `misc_faller_create`.
///
/// Source: `oracle/codemp/game/g_misc.c:2789-2828`
pub fn misc_faller_create(
    ctx: &mut GameContext,
    ent: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    let faller_id = G_Spawn(ctx);

    let s10 = G_SoundIndex("sound/player/fallsplat");
    ctx.world.entity_mut(faller_id).genericValue10 = s10;
    let s9 = G_SoundIndex("sound/chars/stofficer1/misc/falling1");
    {
        let f = ctx.world.entity_mut(faller_id);
        f.genericValue9 = s9;
        f.genericValue8 = 0;
        f.genericValue7 = 0;
    }

    let level_time = ctx.world.level.time;
    ctx.world.entity_mut(faller_id).genericValue6 = level_time + 15000;

    let origin = ctx.world.entity(ent).s.origin;
    G_SetOrigin(ctx.world.entity_mut(faller_id), origin);

    ctx.world.entity_mut(faller_id).s.modelGhoul2 = 1;
    let mi = G_ModelIndex("models/players/stormtrooper/model.glm");
    {
        let f = ctx.world.entity_mut(faller_id);
        f.s.modelindex = mi;
        f.s.g2radius = 100;
    }

    let c0 = (ctx.world.bg_state.rng.Q_irand(1, 255) as u8) as i32;
    ctx.world.entity_mut(faller_id).s.customRGBA[0] = c0;
    let c1 = (ctx.world.bg_state.rng.Q_irand(1, 255) as u8) as i32;
    ctx.world.entity_mut(faller_id).s.customRGBA[1] = c1;
    let c2 = (ctx.world.bg_state.rng.Q_irand(1, 255) as u8) as i32;
    ctx.world.entity_mut(faller_id).s.customRGBA[2] = c2;
    ctx.world.entity_mut(faller_id).s.customRGBA[3] = 255;

    let level_time = ctx.world.level.time;
    {
        let f = ctx.world.entity_mut(faller_id);
        f.r.mins = [-15.0, -15.0, DEFAULT_MINS_2 as f32];
        f.r.maxs = [15.0, 15.0, DEFAULT_MAXS_2 as f32];

        f.clipmask = MASK_PLAYERSOLID;
        f.r.contents = MASK_PLAYERSOLID;

        f.s.eFlags = EF_RAG | EF_CLIENTSMOOTH;

        f.think = Some(EntThink::faller_think).into();
        f.nextthink = level_time;

        f.touch = Some(EntTouch::faller_touch).into();
    }

    let v0 = ctx.world.bg_state.rng.flrand(-256.0, 256.0);
    ctx.world.entity_mut(faller_id).epVelocity[0] = v0;
    let v1 = ctx.world.bg_state.rng.flrand(-256.0, 256.0);
    ctx.world.entity_mut(faller_id).epVelocity[1] = v1;

    let fp: *mut gentity_t = ctx.world.entity_mut(faller_id);
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(fp.cast()));
}

/// Raven `misc_faller_think`.
///
/// Source: `oracle/codemp/game/g_misc.c:2830-2834`
pub fn misc_faller_think(ctx: &mut GameContext, ent: EntityId) {
    misc_faller_create(ctx, ent, Some(ent), Some(ent));
    let level_time = ctx.world.level.time;
    let gv1 = ctx.world.entity(ent).genericValue1;
    let gv2 = ctx.world.entity(ent).genericValue2;
    let r = ctx.world.bg_state.rng.Q_irand(0, gv2);
    ctx.world.entity_mut(ent).nextthink = level_time + gv1 + r;
}

/// Raven `SP_misc_faller`.
///
/// Source: `oracle/codemp/game/g_misc.c:2844-2865`
pub fn SP_misc_faller(ctx: &mut GameContext, ent: EntityId) {
    G_ModelIndex("models/players/stormtrooper/model.glm");
    G_SoundIndex("sound/chars/stofficer1/misc/pain25");
    G_SoundIndex("sound/chars/stofficer1/misc/pain50");
    G_SoundIndex("sound/chars/stofficer1/misc/pain75");
    G_SoundIndex("sound/chars/stofficer1/misc/falling1");
    G_SoundIndex("sound/player/fallsplat");

    let mut interval: c_int = 0;
    G_SpawnInt(ctx, c"interval".as_ptr(), c"500".as_ptr(), &mut interval);
    ctx.world.entity_mut(ent).genericValue1 = interval;
    let mut fudge: c_int = 0;
    G_SpawnInt(ctx, c"fudgefactor".as_ptr(), c"0".as_ptr(), &mut fudge);
    ctx.world.entity_mut(ent).genericValue2 = fudge;

    let targetname = ctx.world.entity(ent).targetname_str();
    if targetname.as_deref().map_or(true, |s| s.is_empty()) {
        let level_time = ctx.world.level.time;
        let gv1 = ctx.world.entity(ent).genericValue1;
        let gv2 = ctx.world.entity(ent).genericValue2;
        let r = ctx.world.bg_state.rng.Q_irand(0, gv2);
        let e = ctx.world.entity_mut(ent);
        e.think = Some(EntThink::misc_faller_think).into();
        e.nextthink = level_time + gv1 + r;
    } else {
        ctx.world.entity_mut(ent).use_ = Some(EntUse::misc_faller_create).into();
    }
}

/// Raven `FirstFreeTagOwner`.
///
/// Source: `oracle/codemp/game/g_misc.c:2888-2903`
pub fn FirstFreeTagOwner(ctx: &mut GameContext) -> *mut crate::level::tag_owner::tagOwner_t {
    let mut i: c_int = 0;
    while i < MAX_TAG_OWNERS as c_int {
        if ctx.world.refTagOwnerMap[i as usize].inuse == 0 {
            return &mut ctx.world.refTagOwnerMap[i as usize] as *mut _;
        }
        i += 1;
    }

    crate::g_main::Com_Printf(&format!(
        "WARNING: MAX_TAG_OWNERS ({}) REF TAG LIMIT HIT\n",
        MAX_TAG_OWNERS
    ));
    core::ptr::null_mut()
}

/// Raven `FirstFreeRefTag`.
///
/// Source: `oracle/codemp/game/g_misc.c:2905-2922`
pub fn FirstFreeRefTag(
    ctx: &mut GameContext,
    tagOwner: *mut crate::level::tag_owner::tagOwner_t,
) -> *mut reference_tag_t {
    unsafe {
        assert!(!tagOwner.is_null());
        let owner = tagOwner;
        let mut i: c_int = 0;

        while i < MAX_TAGS as c_int {
            if (*owner).tags[i as usize].inuse == 0 {
                return &mut (*owner).tags[i as usize] as *mut reference_tag_t;
            }
            i += 1;
        }

        crate::g_main::Com_Printf(&format!("WARNING: MAX_TAGS ({}) REF TAG LIMIT HIT\n", MAX_TAGS));
        core::ptr::null_mut()
    }
}

/// Raven `TAG_Init`.
///
/// Source: `oracle/codemp/game/g_misc.c:2930-2945`
pub fn TAG_Init(ctx: &mut GameContext) {
    unsafe {
        let mut i: c_int = 0;
        while i < MAX_TAG_OWNERS as c_int {
            let mut x: c_int = 0;
            while x < MAX_TAGS as c_int {
                ctx.world.refTagOwnerMap[i as usize].tags[x as usize] = core::mem::zeroed();
                x += 1;
            }
            ctx.world.refTagOwnerMap[i as usize] = core::mem::zeroed();
            i += 1;
        }
    }
}

/// Raven `TAG_FindOwner`.
///
/// Source: `oracle/codemp/game/g_misc.c:2953-2967`
pub fn TAG_FindOwner(
    ctx: &mut GameContext,
    owner: *const c_char,
) -> *mut crate::level::tag_owner::tagOwner_t {
    let mut i: c_int = 0;
    while i < MAX_TAG_OWNERS as c_int {
        if ctx.world.refTagOwnerMap[i as usize].inuse != 0
            && q_shared::Q_stricmp(ctx.world.refTagOwnerMap[i as usize].name.as_ptr(), owner) == 0
        {
            return &mut ctx.world.refTagOwnerMap[i as usize] as *mut _;
        }
        i += 1;
    }

    core::ptr::null_mut()
}

/// Raven `TAG_Find`.
///
/// Source: `oracle/codemp/game/g_misc.c:2975-3028`
pub fn TAG_Find(
    ctx: &mut GameContext,
    owner: *const c_char,
    name: *const c_char,
) -> *mut reference_tag_t {
    unsafe {
        let mut tag_owner: *mut crate::level::tag_owner::tagOwner_t = core::ptr::null_mut();
        let mut i: c_int = 0;

        if !owner.is_null() && *owner != 0 {
            tag_owner = TAG_FindOwner(ctx, owner);
        }
        if tag_owner.is_null() {
            tag_owner = TAG_FindOwner(ctx, cstr(TAG_GENERIC_NAME).as_ptr());
        }

        // Not found...
        if tag_owner.is_null() {
            tag_owner = TAG_FindOwner(ctx, cstr(TAG_GENERIC_NAME).as_ptr());

            if tag_owner.is_null() {
                return core::ptr::null_mut();
            }
        }

        let owner_ptr = tag_owner as *mut crate::level::tag_owner::tagOwner_t;
        while i < MAX_TAGS as c_int {
            if (*owner_ptr).tags[i as usize].inuse != 0
                && q_shared::Q_stricmp((*owner_ptr).tags[i as usize].name.as_ptr(), name) == 0
            {
                return &mut (*owner_ptr).tags[i as usize] as *mut reference_tag_t;
            }
            i += 1;
        }

        // Try the generic owner instead
        let generic = TAG_FindOwner(ctx, cstr(TAG_GENERIC_NAME).as_ptr());

        if generic.is_null() {
            return core::ptr::null_mut();
        }

        let generic_ptr = generic as *mut crate::level::tag_owner::tagOwner_t;
        i = 0;
        while i < MAX_TAGS as c_int {
            if (*generic_ptr).tags[i as usize].inuse != 0
                && q_shared::Q_stricmp((*generic_ptr).tags[i as usize].name.as_ptr(), name) == 0
            {
                return &mut (*generic_ptr).tags[i as usize] as *mut reference_tag_t;
            }
            i += 1;
        }

        core::ptr::null_mut()
    }
}

/// Raven `TAG_Add`.
///
/// Source: `oracle/codemp/game/g_misc.c:3036-3104`
pub fn TAG_Add(
    ctx: &mut GameContext,
    name: *const c_char,
    owner: *const c_char,
    origin: vec3_t,
    angles: vec3_t,
    radius: c_int,
    flags: c_int,
) -> *mut reference_tag_t {
    unsafe {
        let mut owner = owner;
        // Make sure this tag's name isn't already in use
        if !TAG_Find(ctx, owner, name).is_null() {
            crate::g_main::Com_Printf(&format!(
                "^1Duplicate tag name \"{}\"\n",
                cstr_to_str(name)
            ));
            return core::ptr::null_mut();
        }

        // Attempt to add this to the owner's list
        if owner.is_null() || *owner == 0 {
            // If the owner isn't found, use the generic world name
            owner = TAG_GENERIC_NAME_C.as_ptr();
        }

        let mut tag_owner = TAG_FindOwner(ctx, owner);

        if tag_owner.is_null() {
            // Create a new owner list
            tag_owner = FirstFreeTagOwner(ctx);

            if tag_owner.is_null() {
                debug_assert!(false);
                return core::ptr::null_mut();
            }
        }

        // This is actually reverse order of how SP does it because of the way we're storing/allocating.
        // Now that we have the owner, we want to get the first free reftag on the owner itself.
        let tag = FirstFreeRefTag(ctx, tag_owner);

        if tag.is_null() {
            debug_assert!(false);
            return core::ptr::null_mut();
        }

        // Copy the information
        crate::q_math::_VectorCopy(origin, &mut (*tag).origin);
        crate::q_math::_VectorCopy(angles, &mut (*tag).angles);
        (*tag).radius = radius;
        (*tag).flags = flags;

        if name.is_null() || *name == 0 {
            crate::g_main::Com_Printf(&format!(
                "^1ERROR: Nameless ref_tag found at ({} {} {})\n",
                origin[0] as c_int, origin[1] as c_int, origin[2] as c_int
            ));
            return core::ptr::null_mut();
        }

        let owner_ptr = tag_owner as *mut crate::level::tag_owner::tagOwner_t;
        // Copy the name
        Q_strncpyz((*owner_ptr).name.as_mut_ptr(), owner, MAX_REFNAME as c_int);
        Q_strlwr((*owner_ptr).name.as_mut_ptr()); //NOTENOTE: For case insensitive searches on a map

        // Copy the name
        Q_strncpyz((*tag).name.as_mut_ptr(), name, MAX_REFNAME as c_int);
        Q_strlwr((*tag).name.as_mut_ptr());

        (*owner_ptr).inuse = qtrue;
        (*tag).inuse = qtrue;

        tag
    }
}

// vec3 out-param reshape: `origin` is written through
// (`VectorClear`/`VectorCopy` in every branch), never read — reshaped to
// `&mut vec3_t` (no same-file callers to fix; cross-file callers are the
// fixer's job per the packet).
/// Raven `TAG_GetOrigin`.
///
/// Source: `oracle/codemp/game/g_misc.c:3112-3125`
pub fn TAG_GetOrigin(
    ctx: &mut GameContext,
    owner: *const c_char,
    name: *const c_char,
    origin: &mut vec3_t,
) -> c_int {
    let tag = TAG_Find(ctx, owner, name);
    unsafe {
        if tag.is_null() {
            *origin = [0.0, 0.0, 0.0];
            return 0;
        }
        *origin = (*tag).origin;
    }
    1
}

// vec3 out-param reshape (as `TAG_GetOrigin`): `origin` is written
// through, never read.
/// Raven `TAG_GetOrigin2`.
///
/// Source: `oracle/codemp/game/g_misc.c:3134-3146`
pub fn TAG_GetOrigin2(
    ctx: &mut GameContext,
    owner: *const c_char,
    name: *const c_char,
    origin: &mut vec3_t,
) -> c_int {
    let tag = TAG_Find(ctx, owner, name);
    if tag.is_null() {
        return 0;
    }
    unsafe {
        *origin = (*tag).origin;
    }
    1
}

// vec3 out-param reshape (as `TAG_GetOrigin`): `angles` is written
// through, never read.
/// Raven `TAG_GetAngles`.
///
/// Source: `oracle/codemp/game/g_misc.c:3153-3166`
pub fn TAG_GetAngles(
    ctx: &mut GameContext,
    owner: *const c_char,
    name: *const c_char,
    angles: &mut vec3_t,
) -> c_int {
    let tag = TAG_Find(ctx, owner, name);
    if tag.is_null() {
        // Raven `assert(0)` on the not-found path (UB in a release build,
        // porting-rules §19); we take the one defined behavior — report failure.
        return 0;
    }
    unsafe {
        *angles = (*tag).angles;
    }
    1
}

/// Raven `TAG_GetRadius`.
///
/// Source: `oracle/codemp/game/g_misc.c:3174-3185`
pub fn TAG_GetRadius(ctx: &mut GameContext, owner: *const c_char, name: *const c_char) -> c_int {
    let tag = TAG_Find(ctx, owner, name);
    if tag.is_null() {
        // Raven `assert(0)` on the not-found path (UB in a release build,
        // porting-rules §19); we take the one defined behavior — report failure.
        return 0;
    }
    unsafe { (*tag).radius }
}

/// Raven `TAG_GetFlags`.
///
/// Source: `oracle/codemp/game/g_misc.c:3193-3204`
pub fn TAG_GetFlags(ctx: &mut GameContext, owner: *const c_char, name: *const c_char) -> c_int {
    let tag = TAG_Find(ctx, owner, name);
    if tag.is_null() {
        // Raven `assert(0)` on the not-found path (UB in a release build,
        // porting-rules §19); we take the one defined behavior — report failure.
        return 0;
    }
    unsafe { (*tag).flags }
}

/// Raven `ref_link`.
///
/// Source: `oracle/codemp/game/g_misc.c:3267-3298`
pub fn ref_link(ctx: &mut GameContext, ent: EntityId) {
    use crate::q_math::vectoangles;

    let target_field = ctx.world.entity(ent).target.clone();
    if target_field.is_some() {
        //TODO: Find the target and set our angles to that direction
        let target = G_Find(ctx, None, EntFindField::Targetname, target_field.as_deref().unwrap());
        let mut dir: vec3_t = [0.0, 0.0, 0.0];

        if !target.is_null() {
            // Find the direction to the target
            let target_id = ctx.entity_id_of(target).unwrap();
            let target_origin = ctx.world.entity(target_id).s.origin;
            let ent_origin = ctx.world.entity(ent).s.origin;
            crate::q_math::_VectorSubtract(target_origin, ent_origin, &mut dir);
            VectorNormalize(&mut dir);
            vectoangles(dir, &mut ctx.world.entity_mut(ent).s.angles);
            //FIXME: Does pitch get flipped?
        } else {
            let targetname = ctx.world.entity(ent).targetname_str();
            Com_Printf(&format!(
                "^1ERROR: ref_tag ({}) has invalid target ({})",
                targetname.as_deref().unwrap_or(""),
                target_field.as_deref().unwrap()
            ));
        }
    }

    // Add the tag
    let targetname = ctx.world.entity(ent).targetname_str();
    let targetname_c = targetname.as_deref().map(cstr);
    let targetname_ptr = targetname_c
        .as_ref()
        .map_or(core::ptr::null(), |c| c.as_ptr());
    let ownername = ctx.world.entity(ent).ownername.clone();
    let ownername_c = cstr(&ownername);
    let origin = ctx.world.entity(ent).s.origin;
    let angles = ctx.world.entity(ent).s.angles;
    TAG_Add(ctx, targetname_ptr, ownername_c.as_ptr(), origin, angles, 16, 0);

    // Delete immediately, cannot be refered to as an entity again
    // NOTE: this means if you wanted to link them in a chain for, say, a path, you can't
    G_FreeEntity(ctx, Some(ent));
}

/// Raven `SP_reference_tag`.
///
/// Source: `oracle/codemp/game/g_misc.c:3300-3312`
pub fn SP_reference_tag(ctx: &mut GameContext, ent: EntityId) {
    if ctx.world.entity(ent).target.is_some() {
        // Init cannot occur until all entities have been spawned
        let level_time = ctx.world.level.time;
        let e = ctx.world.entity_mut(ent);
        e.think = Some(EntThink::ref_link).into();
        e.nextthink = level_time + START_TIME_LINK_ENTS;
    } else {
        ref_link(ctx, ent);
    }
}

/// Raven `G_ClientForShooter`.
///
/// Source: `oracle/codemp/game/g_misc.c:3354-3375`
pub fn G_ClientForShooter(ctx: &mut GameContext) -> *mut gclient_t {
    unsafe {
        let mut i: c_int = 0;

        if ctx.world.globals.g_shooterClientInit == qfalse {
            // in theory it should be initialized to 0 on the stack, but just in case.
            for slot in ctx.world.globals.g_shooterClients.iter_mut() {
                // `shooterClient_t::default()` is the memset-0 image (its
                // `cl.pers.netname` `String` is not zero-valid, so reset the
                // whole slot rather than byte-zeroing over it).
                *slot = shooterClient_t::default();
            }
            ctx.world.globals.g_shooterClientInit = qtrue;
        }

        while (i as usize) < (MAX_SHOOTERS) as usize {
            if ctx.world.globals.g_shooterClients[i as usize].inuse == qfalse {
                return &mut ctx.world.globals.g_shooterClients[i as usize].cl as *mut gclient_t;
            }
            i += 1;
        }

        crate::g_main::Com_Error(
            mp_qshared::shared::error_parm::errorParm_t::ERR_DROP as c_int,
            c"No free shooter clients - hit MAX_SHOOTERS".as_ptr(),
        );
        core::ptr::null_mut()
    }
}

/// Raven `G_FreeClientForShooter`.
///
/// Source: `oracle/codemp/game/g_misc.c:3377-3389`
pub fn G_FreeClientForShooter(ctx: &mut GameContext, cl: *mut gclient_t) {
    let mut i: usize = 0;
    while i < (MAX_SHOOTERS) as usize {
        if &mut ctx.world.globals.g_shooterClients[i].cl as *mut gclient_t == cl {
            ctx.world.globals.g_shooterClients[i].inuse = qfalse;
            return;
        }
        i += 1;
    }
}

/// Raven `misc_weapon_shooter_fire`.
///
/// Source: `oracle/codemp/game/g_misc.c:3391-3399`
pub fn misc_weapon_shooter_fire(ctx: &mut GameContext, self_: EntityId) {
    use crate::g_weapon::FireWeapon;

    let spawnflags = ctx.world.entity(self_).spawnflags;
    FireWeapon(ctx, Some(self_), (spawnflags & 1) != 0);
    if ctx.world.entity(self_).spawnflags & 2 != 0 {
        let level_time = ctx.world.level.time;
        let wait = ctx.world.entity(self_).wait;
        let e = ctx.world.entity_mut(self_);
        e.think = Some(EntThink::misc_weapon_shooter_fire).into();
        e.nextthink = level_time + wait as c_int;
    }
}

/// Raven `misc_weapon_shooter_use`.
///
/// Source: `oracle/codemp/game/g_misc.c:3401-3415`
pub fn misc_weapon_shooter_use(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    if ctx.world.entity(self_).think.get() == Some(EntThink::misc_weapon_shooter_fire) {
        // repeating fire, stop
        /*
        G_FreeClientForShooter(self->client);
        self->think = G_FreeEntity;
        self->nextthink = level.time;
        */
        ctx.world.entity_mut(self_).nextthink = 0;
        return;
    }
    // otherwise, fire
    misc_weapon_shooter_fire(ctx, self_);
}

/// Raven `misc_weapon_shooter_aim`.
///
/// Source: `oracle/codemp/game/g_misc.c:3417-3438`
pub fn misc_weapon_shooter_aim(ctx: &mut GameContext, self_: EntityId) {
    use crate::q_math::vectoangles;

    // update my aim
    let target_field = ctx.world.entity(self_).target.clone();
    if target_field.is_some() {
        let targ = G_Find(ctx, None, EntFindField::Targetname, target_field.as_deref().unwrap());
        if !targ.is_null() {
            let targ_id = ctx.entity_id_of(targ).unwrap();
            ctx.world.entity_mut(self_).enemy = Some(targ_id);
            let targ_origin = ctx.world.entity(targ_id).r.currentOrigin;
            let self_origin = ctx.world.entity(self_).r.currentOrigin;
            crate::q_math::_VectorSubtract(
                targ_origin,
                self_origin,
                &mut ctx.world.entity_mut(self_).pos1,
            );
            ctx.world.entity_mut(self_).pos1 = targ_origin;
            // FLAG (task #7): self_->client is a shooter-pool gclient_t; ps
            // viewangles dereffed raw exactly as Raven does (copied pointer value).
            let client = ctx.world.entity(self_).client;
            let pos1 = ctx.world.entity(self_).pos1;
            unsafe {
                vectoangles(pos1, &mut (*client).ps.viewangles);
            }
            let viewangles = unsafe { (*client).ps.viewangles };
            SetClientViewAngle(ctx.world.entity_mut(self_), viewangles);
            //FIXME: don't keep doing this unless target is a moving target?
            let level_time = ctx.world.level.time;
            ctx.world.entity_mut(self_).nextthink = level_time + FRAMETIME;
        } else {
            ctx.world.entity_mut(self_).enemy = None;
        }
    }
}

/// Raven `SP_misc_weapon_shooter`.
///
/// Source: `oracle/codemp/game/g_misc.c:3444-3486`
pub fn SP_misc_weapon_shooter(ctx: &mut GameContext, self_: EntityId) {
    // alloc a client just for the weapon code to use
    let cl = G_ClientForShooter(ctx);
    ctx.world.entity_mut(self_).client = cl;

    let (_, s) = G_SpawnString(ctx, "weapon", "");

    // FLAG (task #7): self_->client is a shooter-pool gclient_t; ps dereffed raw
    // exactly as Raven does (copied pointer value).
    let client = ctx.world.entity(self_).client;

    // set weapon
    ctx.world.entity_mut(self_).s.weapon = mp_bg::weapons::weapon_t::WP_BLASTER;
    unsafe {
        (*client).ps.weapon = mp_bg::weapons::weapon_t::WP_BLASTER;
    }
    if !s.is_empty() {
        // use a different weapon
        let w = GetIDForString(WPTable.as_ptr() as *mut _, &s);
        ctx.world.entity_mut(self_).s.weapon = w;
        unsafe {
            (*client).ps.weapon = w;
        }
    }

    let weapon = ctx.world.entity(self_).s.weapon;
    crate::g_items::RegisterItem(ctx, mp_bg::bg_misc::BG_FindItemForWeapon(weapon));

    // set where our muzzle is
    let origin = ctx.world.entity(self_).s.origin;
    unsafe {
        crate::q_math::_VectorCopy(origin, &mut (*client).renderInfo.muzzlePoint);
    }
    // permanently updated (don't need for MP)
    //self->client->renderInfo.mPCalcTime = Q3_INFINITE;

    // set up to link
    if ctx.world.entity(self_).target.is_some() {
        let level_time = ctx.world.level.time;
        let e = ctx.world.entity_mut(self_);
        e.think = Some(EntThink::misc_weapon_shooter_aim).into();
        e.nextthink = level_time + START_TIME_LINK_ENTS;
    } else {
        // just set aim angles
        let angles = ctx.world.entity(self_).s.angles;
        unsafe {
            crate::q_math::_VectorCopy(angles, &mut (*client).ps.viewangles);
        }
        AngleVectors(
            angles,
            Some(&mut ctx.world.entity_mut(self_).pos1),
            None,
            None,
        );
    }

    // set up to fire when used
    ctx.world.entity_mut(self_).use_ = Some(EntUse::misc_weapon_shooter_use).into();

    if ctx.world.entity(self_).wait == 0.0 {
        ctx.world.entity_mut(self_).wait = 500.0;
    }
}

/// Raven `SP_misc_weather_zone`.
///
/// Source: `oracle/codemp/game/g_misc.c:3491-3494`
pub fn SP_misc_weather_zone(ctx: &mut GameContext, ent: EntityId) {
    G_FreeEntity(ctx, Some(ent));
}

// The local `G_SpawnInt` shim formerly here (adapting byte-string literals to
// `*const c_char`) is dropped: its only callers (`EnergyShieldStationSettings`/
// `EnergyAmmoStationSettings`/`EnergyHealthStationSettings`) are parked
// (`seam-threading` — `G_SpawnInt` needs a `GameContext` this shim had no way
// to supply), so it has zero live callers (porting-rules §20).
