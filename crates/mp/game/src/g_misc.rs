// PORT-COMPLETE: pass-3 blind transcription filled the remaining 47 fns
// (zero-park policy — no fn left `todo!()`-bodied). `g_shooterClients`/
// `g_shooterClientInit` are now ported as a real `GameWorld` field
// (`shooterClient_t` array, this file).
//! FAITHFUL port of `oracle/codemp/game/g_misc.c`.
//!
//! Filled by the jampgame mega-pass.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

use crate::ent_fn_enums::{EntDie, EntThink, EntTouch, EntUse};
use crate::ent_id::resolve;
use crate::g_client::SetClientViewAngle;
use crate::g_combat::AddScore;
use crate::g_exphysics::G_RunExPhys;
use crate::g_local_consts::{START_TIME_FIND_LINKS, START_TIME_LINK_ENTS};
use crate::g_main::{G_Printf, LogExit};
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
use crate::q_shared::{Info_SetValueForKey, Q_strlwr};
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
#[derive(Clone, Copy)]
pub struct shooterClient_t {
    pub cl: gclient_t,
    pub inuse: qboolean,
}

impl Default for shooterClient_t {
    fn default() -> Self {
        // `gclient_t` has no library `Default`; zeroed matches Raven's
        // `memset(g_shooterClients, 0, ...)` init.
        unsafe { core::mem::zeroed() }
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
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    use mp_abi::game::syscalls::G_GET_CONFIGSTRING::GGetConfigstringArgs;
    use mp_abi::game::syscalls::G_SET_CONFIGSTRING::GSetConfigstringArgs;
    unsafe {
        let m_light_style = (*ent).count;
        let m_light_switch_style = (*ent).bounceCount;
        let m_light_off_style = (*ent).fly_sound_debounce_time;
        if (*ent).alt_fire == qfalse {
            // turn off
            if m_light_off_style != 0 {
                for slot in 0..3 {
                    let mut lightstyle: [c_char; 32] = [0; 32];
                    trap::GetConfigstring(
                        ctx.engine,
                        GGetConfigstringArgs::new(
                            CS_LIGHT_STYLES + (m_light_off_style * 3) + slot,
                            lightstyle.as_mut_ptr(),
                            32,
                        ),
                    );
                    let s = cstr_to_str(lightstyle.as_ptr());
                    trap::SetConfigstring(
                        ctx.engine,
                        GSetConfigstringArgs::new(
                            CS_LIGHT_STYLES + (m_light_style * 3) + slot,
                            cstr(&s),
                        ),
                    );
                }
            } else {
                trap::SetConfigstring(
                    ctx.engine,
                    GSetConfigstringArgs::new(CS_LIGHT_STYLES + (m_light_style * 3) + 0, cstr("a")),
                );
                trap::SetConfigstring(
                    ctx.engine,
                    GSetConfigstringArgs::new(CS_LIGHT_STYLES + (m_light_style * 3) + 1, cstr("a")),
                );
                trap::SetConfigstring(
                    ctx.engine,
                    GSetConfigstringArgs::new(CS_LIGHT_STYLES + (m_light_style * 3) + 2, cstr("a")),
                );
            }
        } else {
            // Turn myself on now
            if m_light_switch_style != 0 {
                for slot in 0..3 {
                    let mut lightstyle: [c_char; 32] = [0; 32];
                    trap::GetConfigstring(
                        ctx.engine,
                        GGetConfigstringArgs::new(
                            CS_LIGHT_STYLES + (m_light_switch_style * 3) + slot,
                            lightstyle.as_mut_ptr(),
                            32,
                        ),
                    );
                    let s = cstr_to_str(lightstyle.as_ptr());
                    trap::SetConfigstring(
                        ctx.engine,
                        GSetConfigstringArgs::new(
                            CS_LIGHT_STYLES + (m_light_style * 3) + slot,
                            cstr(&s),
                        ),
                    );
                }
            } else {
                trap::SetConfigstring(
                    ctx.engine,
                    GSetConfigstringArgs::new(CS_LIGHT_STYLES + (m_light_style * 3) + 0, cstr("z")),
                );
                trap::SetConfigstring(
                    ctx.engine,
                    GSetConfigstringArgs::new(CS_LIGHT_STYLES + (m_light_style * 3) + 1, cstr("z")),
                );
                trap::SetConfigstring(
                    ctx.engine,
                    GSetConfigstringArgs::new(CS_LIGHT_STYLES + (m_light_style * 3) + 2, cstr("z")),
                );
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
    // STAGE-1: EntityId ent + Option<EntityId> other/activator; raw body re-derived verbatim (Stage-2 debt).
    let base = ctx.world.g_entities.as_mut_ptr();
    let ent: *mut gentity_t = ctx.entity_mut(ent);
    let other: *mut gentity_t = unsafe { resolve(base, other) };
    let activator: *mut gentity_t = unsafe { resolve(base, activator) };

    unsafe {
        G_ActivateBehavior(ctx, ctx.entity_id_of(ent), bSet_t::BSET_USE as c_int);
        (*ent).alt_fire = if (*ent).alt_fire != qfalse {
            qfalse
        } else {
            qtrue
        };
        misc_lightstyle_set(ctx, ctx.entity_id_of(ent).unwrap());
    }
}

/// Raven `SP_light`.
///
/// Source: `oracle/codemp/game/g_misc.c:142-166`
pub fn SP_light(ctx: &mut GameContext, self_: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);

    unsafe {
        if (*self_).targetname.is_null() {
            // if i don't have a light style switch, then i go away
            G_FreeEntity(ctx, ctx.entity_id_of(self_));
            return;
        }

        G_SpawnInt(
            ctx,
            c"style".as_ptr(),
            c"0".as_ptr(),
            &mut (*self_).count as *mut c_int,
        );
        G_SpawnInt(
            ctx,
            c"switch_style".as_ptr(),
            c"0".as_ptr(),
            &mut (*self_).bounceCount as *mut c_int,
        );
        G_SpawnInt(
            ctx,
            c"style_off".as_ptr(),
            c"0".as_ptr(),
            &mut (*self_).fly_sound_debounce_time as *mut c_int,
        );
        G_SetOrigin(&mut *(self_), (*self_).s.origin);
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(self_));

        (*self_).use_ = Some(EntUse::misc_dlight_use).into();

        (*self_).s.eType = entityType_t::ET_GENERAL as c_int;
        (*self_).alt_fire = qfalse;
        (*self_).r.svFlags |= SVF_NOCLIENT;

        if (*self_).spawnflags & 4 == 0 {
            // turn myself on now
            (*self_).alt_fire = qtrue;
        }
        misc_lightstyle_set(ctx, ctx.entity_id_of(self_).unwrap());
    }
}

/// Raven `TeleportPlayer`.
///
/// Source: `oracle/codemp/game/g_misc.c:177-231`
pub fn TeleportPlayer(ctx: &mut GameContext, player: EntityId, origin: vec3_t, angles: vec3_t) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let player: *mut gentity_t = ctx.entity_mut(player);

    use mp_abi::game::syscalls::G_UNLINKENTITY::GUnlinkentityArgs;
    use mp_bg::public::team::TEAM_SPECTATOR;
    unsafe {
        let mut is_npc = qfalse;
        if (*player).s.eType == entityType_t::ET_NPC as c_int {
            is_npc = qtrue;
        }

        // use temp events at source and destination to prevent the effect
        // from getting dropped by a second player event
        if (*((*player).client as *mut gclient_t)).sess.sessionTeam != TEAM_SPECTATOR {
            let tent = G_TempEntity(
                ctx,
                (*((*player).client as *mut gclient_t)).ps.origin,
                EV_PLAYER_TELEPORT_OUT as c_int,
            );
            (*tent).s.clientNum = (*player).s.clientNum;

            let tent = G_TempEntity(ctx, origin, EV_PLAYER_TELEPORT_IN as c_int);
            (*tent).s.clientNum = (*player).s.clientNum;
        }

        // unlink to make sure it can't possibly interfere with G_KillBox
        trap::UnlinkEntity(ctx.engine, GUnlinkentityArgs::new(player));

        crate::q_math::_VectorCopy(
            origin,
            &mut (*((*player).client as *mut gclient_t)).ps.origin,
        );
        (*((*player).client as *mut gclient_t)).ps.origin[2] += 1.0;

        // spit the player out
        let mut vel: vec3_t = [0.0, 0.0, 0.0];
        AngleVectors(angles, Some(&mut vel), None, None);
        crate::q_math::_VectorScale(
            vel,
            400.0,
            &mut (*((*player).client as *mut gclient_t)).ps.velocity,
        );
        (*((*player).client as *mut gclient_t)).ps.pm_time = 160; // hold time
        (*((*player).client as *mut gclient_t)).ps.pm_flags |= PMF_TIME_KNOCKBACK;

        // toggle the teleport bit so the client knows to not lerp
        (*((*player).client as *mut gclient_t)).ps.eFlags ^= EF_TELEPORT_BIT;

        // set angles
        SetClientViewAngle(&mut *player, angles);

        // kill anything at the destination
        if (*((*player).client as *mut gclient_t)).sess.sessionTeam != TEAM_SPECTATOR {
            G_KillBox(ctx, ctx.entity_id_of(player).unwrap());
        }

        // save results of pmove
        BG_PlayerStateToEntityState(
            &mut (*((*player).client as *mut gclient_t)).ps,
            &mut (*player).s,
            qtrue,
        );
        if is_npc != qfalse {
            (*player).s.eType = entityType_t::ET_NPC as c_int;
        }

        // use the precise origin for linking
        crate::q_math::_VectorCopy(
            (*((*player).client as *mut gclient_t)).ps.origin,
            &mut (*player).r.currentOrigin,
        );

        if (*((*player).client as *mut gclient_t)).sess.sessionTeam != TEAM_SPECTATOR {
            trap::LinkEntity(ctx.engine, GLinkentityArgs::new(player));
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
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    G_FreeEntity(ctx, ctx.entity_id_of(ent));
}

/// Raven `SP_misc_model_static`.
///
/// Source: `oracle/codemp/game/g_misc.c:277-280`
pub fn SP_misc_model_static(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    G_FreeEntity(ctx, ctx.entity_id_of(ent));
}

/// Raven `SP_misc_G2model`.
///
/// The live (non-`#if 0`) path just frees the entity.
/// Source: `oracle/codemp/game/g_misc.c:285-301`
pub fn SP_misc_G2model(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    G_FreeEntity(ctx, ctx.entity_id_of(ent));
}

/// Raven `locateCamera`.
///
/// Source: `oracle/codemp/game/g_misc.c:305-349`
pub fn locateCamera(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    unsafe {
        let owner = G_PickTarget(ctx, (*ent).target);
        if owner.is_null() {
            G_Printf(
                ctx,
                c"Couldn't find target for misc_partal_surface\n".as_ptr(),
            );
            G_FreeEntity(ctx, ctx.entity_id_of(ent));
            return;
        }
        (*ent).r.ownerNum = (*owner).s.number;

        // frame holds the rotate speed
        if (*owner).spawnflags & 1 != 0 {
            (*ent).s.frame = 25;
        } else if (*owner).spawnflags & 2 != 0 {
            (*ent).s.frame = 75;
        }

        // swing camera ?
        if (*owner).spawnflags & 4 != 0 {
            // set to 0 for no rotation at all
            (*ent).s.powerups = 0;
        } else {
            (*ent).s.powerups = 1;
        }

        // clientNum holds the rotate offset
        (*ent).s.clientNum = (*owner).s.clientNum;

        crate::q_math::_VectorCopy((*owner).s.origin, &mut (*ent).s.origin2);

        // see if the portal_camera has a target
        let target = G_PickTarget(ctx, (*owner).target);
        let mut dir: vec3_t = [0.0, 0.0, 0.0];
        if !target.is_null() {
            crate::q_math::_VectorSubtract((*target).s.origin, (*owner).s.origin, &mut dir);
            VectorNormalize(&mut dir);
        } else {
            G_SetMovedir(&mut (*owner).s.angles, &mut dir);
        }

        (*ent).s.eventParm = DirToByte(dir);
    }
}

/// Raven `SP_misc_portal_surface`.
///
/// Source: `oracle/codemp/game/g_misc.c:355-369`
pub fn SP_misc_portal_surface(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    unsafe {
        (*ent).r.mins = [0.0, 0.0, 0.0];
        (*ent).r.maxs = [0.0, 0.0, 0.0];
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));

        (*ent).r.svFlags = SVF_PORTAL;
        (*ent).s.eType = entityType_t::ET_PORTAL as c_int;

        if (*ent).target.is_null() {
            crate::q_math::_VectorCopy((*ent).s.origin, &mut (*ent).s.origin2);
        } else {
            (*ent).think = Some(EntThink::locateCamera).into();
            (*ent).nextthink = ctx.world.level.time + 100;
        }
    }
}

// PORT-NOTE(raw-ptr-skeleton-no-world-handle): calls `trap_LinkEntity`.
/// Raven `SP_misc_portal_camera`.
///
/// Source: `oracle/codemp/game/g_misc.c:375-385`
pub fn SP_misc_portal_camera(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    unsafe {
        (*ent).r.mins = [0.0, 0.0, 0.0];
        (*ent).r.maxs = [0.0, 0.0, 0.0];
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));

        let mut roll: f32 = 0.0;
        G_SpawnFloat(ctx, c"roll".as_ptr(), c"0".as_ptr(), &mut roll as *mut f32);

        // C evaluates `roll/360.0 * 256` in double (360.0 is a double literal),
        // then truncates to int.
        (*ent).s.clientNum = (roll as f64 / 360.0 * 256.0) as c_int;
    }
}

/// Raven `SP_misc_bsp`.
///
/// Source: `oracle/codemp/game/g_misc.c:390-462`
pub fn SP_misc_bsp(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    use mp_abi::game::syscalls::G_SET_ACTIVE_SUBBSP::GSetActiveSubbspArgs;
    use mp_abi::game::syscalls::G_SET_BRUSH_MODEL::GSetBrushModelArgs;
    use mp_qshared::shared::MAX_QPATH;
    unsafe {
        let mut new_angle: f32 = 0.0;
        G_SpawnFloat(
            ctx,
            c"angle".as_ptr(),
            c"0".as_ptr(),
            &mut new_angle as *mut f32,
        );
        if new_angle != 0.0 {
            (*ent).s.angles[1] = new_angle;
        }
        // don't support rotation any other way
        (*ent).s.angles[0] = 0.0;
        (*ent).s.angles[2] = 0.0;

        let mut out: *mut c_char = core::ptr::null_mut();
        G_SpawnString(ctx, c"bspmodel".as_ptr(), c"".as_ptr(), &mut out);

        (*ent).s.eFlags = EF_PERMANENT;

        // Mainly for debugging
        let mut tempint: c_int = 0;
        G_SpawnInt(
            ctx,
            c"spacing".as_ptr(),
            c"0".as_ptr(),
            &mut tempint as *mut c_int,
        );
        (*ent).s.time2 = tempint;
        G_SpawnInt(
            ctx,
            c"flatten".as_ptr(),
            c"0".as_ptr(),
            &mut tempint as *mut c_int,
        );
        (*ent).s.time = tempint;

        // NOTE: Raven's own `char temp[MAX_QPATH]` is a stack local later
        // assigned into `level.mTargetAdjust` (a persistent `char *`) — the
        // pointer dangles once this fn returns. Faithful UB per porting
        // rules S19; we keep the one Raven-defined behavior rather than
        // invent a fix.
        let mut temp: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
        write_cstr_field(&mut temp, &format!("#{}", cstr_to_str(out)));
        trap::SetBrushModel(
            ctx.engine,
            GSetBrushModelArgs::new(ent, cstr(&cstr_to_str(temp.as_ptr()))),
        ); // SV_SetBrushModel -- sets mins and maxs
        crate::g_utils::G_BSPIndex(ctx, temp.as_ptr());

        ctx.world.level.mNumBSPInstances += 1;
        write_cstr_field(&mut temp, &format!("{}-", ctx.world.level.mNumBSPInstances));
        crate::q_math::_VectorCopy((*ent).s.origin, &mut ctx.world.level.mOriginAdjust);
        ctx.world.level.mRotationAdjust = (*ent).s.angles[1];
        ctx.world.level.mTargetAdjust = temp.as_mut_ptr();
        ctx.world.level.mBSPInstanceDepth += 1;

        let mut teamfilter_out: *mut c_char = core::ptr::null_mut();
        G_SpawnString(
            ctx,
            c"teamfilter".as_ptr(),
            c"".as_ptr(),
            &mut teamfilter_out,
        );
        write_cstr_field(
            &mut ctx.world.level.mTeamFilter,
            &cstr_to_str(teamfilter_out),
        );

        crate::q_math::_VectorCopy((*ent).s.origin, &mut (*ent).s.pos.trBase);
        crate::q_math::_VectorCopy((*ent).s.origin, &mut (*ent).r.currentOrigin);
        crate::q_math::_VectorCopy((*ent).s.angles, &mut (*ent).s.apos.trBase);
        crate::q_math::_VectorCopy((*ent).s.angles, &mut (*ent).r.currentAngles);

        (*ent).s.eType = entityType_t::ET_MOVER as c_int;

        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));

        trap::SetActiveSubBSP(ctx.engine, GSetActiveSubbspArgs::new((*ent).s.modelindex));
        crate::g_spawn::G_SpawnEntitiesFromString(ctx, qtrue);
        trap::SetActiveSubBSP(ctx.engine, GSetActiveSubbspArgs::new(-1));

        ctx.world.level.mBSPInstanceDepth -= 1;
        ctx.world.level.mTeamFilter[0] = 0;
    }
}

// PORT-NOTE(unported-const): `MAX_INFO_STRING` has no ported home; the 1024
// literal below is the oracle's value, used only for local scratch-buffer sizing.
/// Raven `SP_terrain`.
///
/// Source: `oracle/codemp/game/g_misc.c:484-631`
pub fn SP_terrain(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    use mp_abi::game::syscalls::G_CM_REGISTER_TERRAIN::GCmRegisterTerrainArgs;
    use mp_abi::game::syscalls::G_CVAR_SET::GCvarSetArgs;
    use mp_abi::game::syscalls::G_CVAR_VARIABLE_STRING_BUFFER::GCvarVariableStringBufferArgs;
    use mp_abi::game::syscalls::G_RMG_INIT::GRmgInitArgs;
    use mp_abi::game::syscalls::G_SET_BRUSH_MODEL::GSetBrushModelArgs;
    use mp_qshared::shared::MAX_QPATH;
    // `MAX_INFO_STRING` resolves via the crate prelude glob
    // (`mp_qshared::shared::limits`).
    unsafe {
        // Force it to 1 when there is terrain on the level.
        trap::Cvar_Set(ctx.engine, GCvarSetArgs::new(cstr("RMG"), cstr("1")));
        ctx.world.cvars.g_RMG.integer = 1;

        (*ent).s.angles = [0.0, 0.0, 0.0];
        trap::SetBrushModel(
            ctx.engine,
            GSetBrushModelArgs::new(ent, cstr(&cstr_to_str((*ent).model))),
        );

        // Get the shader from the top of the brush
        let shader_num: c_int = 0;

        let mut seed: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
        let mut mission_type: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
        if ctx.world.cvars.g_RMG.integer != 0 {
            trap::Cvar_VariableStringBuffer(
                ctx.engine,
                GCvarVariableStringBufferArgs::new(
                    cstr("RMG_seed"),
                    seed.as_mut_ptr(),
                    (MAX_QPATH) as i32,
                ),
            );
            trap::Cvar_VariableStringBuffer(
                ctx.engine,
                GCvarVariableStringBufferArgs::new(
                    cstr("RMG_mission"),
                    mission_type.as_mut_ptr(),
                    (MAX_QPATH) as i32,
                ),
            );
        }

        // Get info required for the common init
        let mut temp: [c_char; MAX_INFO_STRING] = [0; MAX_INFO_STRING];
        temp[0] = 0;

        let mut value: *mut c_char = core::ptr::null_mut();
        G_SpawnString(ctx, c"heightmap".as_ptr(), c"".as_ptr(), &mut value);
        Info_SetValueForKey(temp.as_mut_ptr(), c"heightMap".as_ptr(), value);

        G_SpawnString(ctx, c"numpatches".as_ptr(), c"400".as_ptr(), &mut value);
        Info_SetValueForKey(
            temp.as_mut_ptr(),
            c"numPatches".as_ptr(),
            cstr(&format!("{}", atoi(value))).as_ptr(),
        );

        G_SpawnString(ctx, c"terxels".as_ptr(), c"4".as_ptr(), &mut value);
        Info_SetValueForKey(
            temp.as_mut_ptr(),
            c"terxels".as_ptr(),
            cstr(&format!("{}", atoi(value))).as_ptr(),
        );

        Info_SetValueForKey(temp.as_mut_ptr(), c"seed".as_ptr(), seed.as_ptr());
        Info_SetValueForKey(
            temp.as_mut_ptr(),
            c"minx".as_ptr(),
            cstr(&format!("{:.6}", (*ent).r.mins[0])).as_ptr(),
        );
        Info_SetValueForKey(
            temp.as_mut_ptr(),
            c"miny".as_ptr(),
            cstr(&format!("{:.6}", (*ent).r.mins[1])).as_ptr(),
        );
        Info_SetValueForKey(
            temp.as_mut_ptr(),
            c"minz".as_ptr(),
            cstr(&format!("{:.6}", (*ent).r.mins[2])).as_ptr(),
        );
        Info_SetValueForKey(
            temp.as_mut_ptr(),
            c"maxx".as_ptr(),
            cstr(&format!("{:.6}", (*ent).r.maxs[0])).as_ptr(),
        );
        Info_SetValueForKey(
            temp.as_mut_ptr(),
            c"maxy".as_ptr(),
            cstr(&format!("{:.6}", (*ent).r.maxs[1])).as_ptr(),
        );
        Info_SetValueForKey(
            temp.as_mut_ptr(),
            c"maxz".as_ptr(),
            cstr(&format!("{:.6}", (*ent).r.maxs[2])).as_ptr(),
        );

        Info_SetValueForKey(
            temp.as_mut_ptr(),
            c"modelIndex".as_ptr(),
            cstr(&format!("{}", (*ent).s.modelindex)).as_ptr(),
        );

        G_SpawnString(
            ctx,
            c"terraindef".as_ptr(),
            c"grassyhills".as_ptr(),
            &mut value,
        );
        Info_SetValueForKey(temp.as_mut_ptr(), c"terrainDef".as_ptr(), value);

        G_SpawnString(ctx, c"instancedef".as_ptr(), c"".as_ptr(), &mut value);
        Info_SetValueForKey(temp.as_mut_ptr(), c"instanceDef".as_ptr(), value);

        G_SpawnString(ctx, c"miscentdef".as_ptr(), c"".as_ptr(), &mut value);
        Info_SetValueForKey(temp.as_mut_ptr(), c"miscentDef".as_ptr(), value);

        Info_SetValueForKey(
            temp.as_mut_ptr(),
            c"missionType".as_ptr(),
            mission_type.as_ptr(),
        );

        // `#define MAX_INSTANCE_TYPES 16` at g_misc.c:483.
        const MAX_INSTANCE_TYPES: c_int = 16;
        let mut i: c_int = 0;
        while i < MAX_INSTANCE_TYPES {
            let mut final_: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
            trap::Cvar_VariableStringBuffer(
                ctx.engine,
                GCvarVariableStringBufferArgs::new(
                    cstr(&format!("RMG_instance{}", i)),
                    final_.as_mut_ptr(),
                    (MAX_QPATH) as i32,
                ),
            );
            if *final_.as_ptr() != 0 {
                Info_SetValueForKey(
                    temp.as_mut_ptr(),
                    cstr(&format!("inst{}", i)).as_ptr(),
                    final_.as_ptr(),
                );
            }
            i += 1;
        }

        // Set additional data required on the client only
        G_SpawnString(ctx, c"densitymap".as_ptr(), c"".as_ptr(), &mut value);
        Info_SetValueForKey(temp.as_mut_ptr(), c"densityMap".as_ptr(), value);

        Info_SetValueForKey(
            temp.as_mut_ptr(),
            c"shader".as_ptr(),
            cstr(&format!("{}", shader_num)).as_ptr(),
        );
        G_SpawnString(ctx, c"texturescale".as_ptr(), c"0.005".as_ptr(), &mut value);
        Info_SetValueForKey(
            temp.as_mut_ptr(),
            c"texturescale".as_ptr(),
            cstr(&format!("{:.6}", crate::bg_lib::atof(value))).as_ptr(),
        );

        // Initialise the common aspects of the terrain
        let terrain_id = trap::CM_RegisterTerrain(
            ctx.engine,
            GCmRegisterTerrainArgs::new(cstr(&cstr_to_str(temp.as_ptr()))),
        );

        Info_SetValueForKey(
            temp.as_mut_ptr(),
            c"terrainId".as_ptr(),
            cstr(&format!("{}", terrain_id)).as_ptr(),
        );

        // Send all the data down to the client
        trap::SetConfigstring(
            ctx.engine,
            mp_abi::game::syscalls::G_SET_CONFIGSTRING::GSetConfigstringArgs::new(
                CS_TERRAINS + terrain_id,
                cstr(&cstr_to_str(temp.as_ptr())),
            ),
        );

        // Make sure the contents are properly set
        (*ent).r.contents = mp_qshared::shared::surface_flags::CONTENTS_TERRAIN;
        (*ent).r.svFlags = SVF_NOCLIENT;
        (*ent).s.eFlags = EF_PERMANENT;
        (*ent).s.eType = entityType_t::ET_TERRAIN as c_int;

        // Hook into the world so physics will work
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));

        // If running RMG then initialize the terrain and handle team skins
        if ctx.world.cvars.g_RMG.integer != 0 {
            trap::RMG_Init(ctx.engine, GRmgInitArgs::new(terrain_id));
        }
    }
}

// PORT-NOTE(raw-ptr-skeleton-no-world-handle): indexes `g_entities`,
// calls `trap_InPVS`/`trap_Trace`; also a fn-pointer write
// (`think = G_FreeEntity`).
/// Raven `G_PortalifyEntities`.
///
/// Source: `oracle/codemp/game/g_misc.c:638-667`
pub fn G_PortalifyEntities(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    use mp_qshared::shared::limits::{ENTITYNUM_NONE, ENTITYNUM_WORLD};
    unsafe {
        let mut i: usize = 0;
        while i < mp_qshared::shared::MAX_GENTITIES {
            let scan = &mut ctx.world.g_entities[i] as *mut gentity_t;
            if (*scan).inuse != 0
                && (*scan).s.number != (*ent).s.number
                && trap::InPVS(
                    ctx.engine,
                    GInPvsArgs::new(
                        &(*ent).s.origin as *const vec3_t,
                        &(*scan).r.currentOrigin as *const vec3_t,
                    ),
                ) != 0
            {
                let mut tr: trace_t = core::mem::zeroed();
                trap::Trace(
                    ctx.engine,
                    GTraceArgs::new(
                        &mut tr as *mut trace_t,
                        &(*ent).s.origin as *const vec3_t,
                        &vec3_origin as *const vec3_t,
                        &vec3_origin as *const vec3_t,
                        &(*scan).r.currentOrigin as *const vec3_t,
                        (*ent).s.number,
                        mp_qshared::shared::surface_flags::CONTENTS_SOLID,
                    ),
                );
                if tr.fraction == 1.0
                    || (tr.entityNum == ((*scan).s.number) as i16
                        && tr.entityNum != (ENTITYNUM_NONE) as i16
                        && tr.entityNum != (ENTITYNUM_WORLD) as i16)
                {
                    if (*scan).client.is_null() || (*scan).s.eType == entityType_t::ET_NPC as c_int
                    {
                        (*scan).s.isPortalEnt = qtrue;
                    }
                }
            }
            i += 1;
        }

        (*ent).think = Some(EntThink::G_FreeEntity).into();
        (*ent).nextthink = ctx.world.level.time;
    }
}

/// Raven `SP_misc_skyportal_orient`.
///
/// Source: `oracle/codemp/game/g_misc.c:675-678`
pub fn SP_misc_skyportal_orient(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    G_FreeEntity(ctx, ctx.entity_id_of(ent));
}

/// Raven `SP_misc_skyportal`.
///
/// Source: `oracle/codemp/game/g_misc.c:694-715`
pub fn SP_misc_skyportal(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    use mp_abi::game::syscalls::G_SET_CONFIGSTRING::GSetConfigstringArgs;
    unsafe {
        let mut fov: *mut c_char = core::ptr::null_mut();
        G_SpawnString(ctx, c"fov".as_ptr(), c"80".as_ptr(), &mut fov);
        let fov_x = crate::bg_lib::atof(fov) as f32;

        let mut fogv: vec3_t = [0.0, 0.0, 0.0];
        let mut isfog: c_int = 0;
        isfog += G_SpawnVector(
            ctx,
            c"fogcolor".as_ptr(),
            c"0 0 0".as_ptr(),
            fogv.as_mut_ptr(),
        );
        let mut fogn: c_int = 0;
        isfog += G_SpawnInt(
            ctx,
            c"fognear".as_ptr(),
            c"0".as_ptr(),
            &mut fogn as *mut c_int,
        );
        let mut fogf: c_int = 0;
        isfog += G_SpawnInt(
            ctx,
            c"fogfar".as_ptr(),
            c"300".as_ptr(),
            &mut fogf as *mut c_int,
        );

        let s = format!(
            "{:.2} {:.2} {:.2} {:.1} {} {:.2} {:.2} {:.2} {} {}",
            (*ent).s.origin[0],
            (*ent).s.origin[1],
            (*ent).s.origin[2],
            fov_x,
            isfog,
            fogv[0],
            fogv[1],
            fogv[2],
            fogn,
            fogf
        );
        trap::SetConfigstring(
            ctx.engine,
            GSetConfigstringArgs::new(CS_SKYBOXORG, cstr(&s)),
        );

        (*ent).think = Some(EntThink::G_PortalifyEntities).into();
        (*ent).nextthink = ctx.world.level.time + 1050; // give it some time first so that all other entities are spawned.
    }
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
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);

    unsafe {
        if ctx.world.bg_state.rng.Q_irand(1, 10) < 5 {
            (*self_).s.pos.trDelta[0] = 150.0 + ctx.world.bg_state.rng.Q_irand(1, 100) as f32;
        } else {
            (*self_).s.pos.trDelta[0] = -150.0 - ctx.world.bg_state.rng.Q_irand(1, 100) as f32;
        }
        if ctx.world.bg_state.rng.Q_irand(1, 10) < 5 {
            (*self_).s.pos.trDelta[1] = 150.0 + ctx.world.bg_state.rng.Q_irand(1, 100) as f32;
        } else {
            (*self_).s.pos.trDelta[1] = -150.0 - ctx.world.bg_state.rng.Q_irand(1, 100) as f32;
        }
        (*self_).s.pos.trDelta[2] = 150.0 + ctx.world.bg_state.rng.Q_irand(1, 100) as f32;
    }
}

// PORT-NOTE(unported-const): `HOLOCRON_RESPAWN_TIME` (`g_local.h`) has no
// ported home anywhere in the crate graph; referenced verbatim per the
// zero-park policy — a fixer ports the const.
/// Raven `HolocronTouch`.
///
/// Source: `oracle/codemp/game/g_misc.c:786-905`
pub fn HolocronTouch(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    trace: *mut trace_t,
) {
    // STAGE-1: EntityId self_ + Option<EntityId> other; raw body re-derived verbatim (Stage-2 debt).
    let base = ctx.world.g_entities.as_mut_ptr();
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    let other: *mut gentity_t = unsafe { resolve(base, other) };

    use mp_qshared::shared::sound_channel::CHAN_AUTO;
    unsafe {
        let mut i: c_int = 0;
        let mut othercarrying: c_int = 0;
        let mut time_lowest: f32 = 0.0;
        let mut index_lowest: c_int = -1;
        let mut hasall = true;
        let mut force_reselect = WP_NONE;

        if !trace.is_null() {
            (*self_).s.groundEntityNum = ((*trace).entityNum) as i32;
        }

        if other.is_null() || (*other).client.is_null() || (*other).health < 1 {
            return;
        }

        if (*self_).s.modelindex == 0 {
            return;
        }

        if (*self_).enemy.is_some() {
            return;
        }

        if (*((*other).client as *mut gclient_t)).ps.holocronsCarried[(*self_).count as usize]
            != 0.0
        {
            return;
        }

        if (*((*other).client as *mut gclient_t)).ps.holocronCantTouch == (*self_).s.number
            && (*((*other).client as *mut gclient_t))
                .ps
                .holocronCantTouchTime
                > ctx.world.level.time as f32
        {
            return;
        }

        while i < (NUM_FORCE_POWERS) as i32 {
            if (*((*other).client as *mut gclient_t)).ps.holocronsCarried[i as usize] != 0.0 {
                othercarrying += 1;

                if index_lowest == -1
                    || (*((*other).client as *mut gclient_t)).ps.holocronsCarried[i as usize]
                        < time_lowest
                {
                    index_lowest = i;
                    time_lowest =
                        (*((*other).client as *mut gclient_t)).ps.holocronsCarried[i as usize];
                }
            } else if i != (*self_).count {
                hasall = false;
            }
            i += 1;
        }

        if hasall {
            // once we pick up this holocron we'll have all of them, so give us super special best prize!
            //G_Printf("You deserve a pat on the back.\n");
        }

        if (*((*other).client as *mut gclient_t))
            .ps
            .fd
            .forcePowersActive
            & (1 << (*((*other).client as *mut gclient_t))
                .ps
                .fd
                .forcePowerSelected)
            == 0
        {
            // If the player isn't using his currently selected force power, select this one
            if (*self_).count != FP_SABER_OFFENSE
                && (*self_).count != FP_SABER_DEFENSE
                && (*self_).count != FP_SABERTHROW
                && (*self_).count != FP_LEVITATION
            {
                (*((*other).client as *mut gclient_t))
                    .ps
                    .fd
                    .forcePowerSelected = (*self_).count;
            }
        }

        if ctx.world.cvars.g_MaxHolocronCarry.integer != 0
            && othercarrying >= ctx.world.cvars.g_MaxHolocronCarry.integer
        {
            // make the oldest holocron carried by the player pop out to make room for this one
            (*((*other).client as *mut gclient_t)).ps.holocronsCarried[index_lowest as usize] = 0.0;
            //NOTE: No longer valid as we are now always giving a force level 1 saber attack level in holocron
        }

        //G_Sound(other, CHAN_AUTO, G_SoundIndex("sound/weapons/w_pkup.wav"));
        G_AddEvent(
            &mut *(other),
            mp_bg::public::entity_event::entity_event_t::EV_ITEM_PICKUP as c_int,
            (*self_).s.number,
        );

        (*((*other).client as *mut gclient_t)).ps.holocronsCarried[(*self_).count as usize] =
            ctx.world.level.time as f32;
        (*self_).s.modelindex = 0;
        (*self_).enemy = Some(ent_id(ctx.world.g_entities.as_mut_ptr(), other));

        (*self_).pos2[0] = 1.0;
        (*self_).pos2[1] = (ctx.world.level.time + HOLOCRON_RESPAWN_TIME) as f32;

        if force_reselect != WP_NONE {
            G_AddEvent(
                &mut *(other),
                mp_bg::public::entity_event::entity_event_t::EV_NOAMMO as c_int,
                force_reselect,
            );
        }

        //G_Printf("DON'T TOUCH ME\n");
    }
}

// PORT-NOTE(control-flow): Raven's `goto justthink;` is ported as an early
// `return` after inlining the shared tail (porting-rules §C10 — preserve
// behavior, not shape).
/// Raven `HolocronThink`.
///
/// Source: `oracle/codemp/game/g_misc.c:907-991`
pub fn HolocronThink(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    unsafe {
        let base = ctx.world.g_entities.as_mut_ptr();

        let justthink = |ent: *mut gentity_t, ctx: &mut GameContext| {
            (*ent).nextthink = ctx.world.level.time + 50;
            if (*ent).s.pos.trDelta[0] != 0.0
                || (*ent).s.pos.trDelta[1] != 0.0
                || (*ent).s.pos.trDelta[2] != 0.0
            {
                G_RunObject(ctx, ctx.entity_id_of(ent).unwrap());
            }
        };

        if (*ent).pos2[0] != 0.0
            && ((*ent).enemy.is_none()
                || (*ent)
                    .enemy
                    .map_or(true, |e| (*(base.add(e.index()))).client.is_null())
                || (*ent)
                    .enemy
                    .map_or(false, |e| (*(base.add(e.index()))).health < 1))
        {
            if let Some(e) = (*ent).enemy {
                let enemy_ptr = base.add(e.index());
                if !(*enemy_ptr).client.is_null() {
                    HolocronRespawn(&mut *ent);
                    crate::q_math::_VectorCopy(
                        (*((*enemy_ptr).client as *mut gclient_t)).ps.origin,
                        &mut (*ent).s.pos.trBase,
                    );
                    crate::q_math::_VectorCopy(
                        (*((*enemy_ptr).client as *mut gclient_t)).ps.origin,
                        &mut (*ent).s.origin,
                    );
                    crate::q_math::_VectorCopy(
                        (*((*enemy_ptr).client as *mut gclient_t)).ps.origin,
                        &mut (*ent).r.currentOrigin,
                    );
                    // copy to person carrying's origin before popping out of them
                    HolocronPopOut(ctx, ctx.entity_id_of(ent).unwrap());
                    (*((*enemy_ptr).client as *mut gclient_t))
                        .ps
                        .holocronsCarried[(*ent).count as usize] = 0.0;
                    (*ent).enemy = None;

                    justthink(ent, ctx);
                    return;
                }
            }
        } else if (*ent).pos2[0] != 0.0
            && (*ent)
                .enemy
                .map_or(false, |e| !(*(base.add(e.index()))).client.is_null())
        {
            (*ent).pos2[1] = (ctx.world.level.time + HOLOCRON_RESPAWN_TIME) as f32;
        }

        if let Some(e) = (*ent).enemy {
            let enemy_ptr = base.add(e.index());
            if !(*enemy_ptr).client.is_null() {
                if (*((*enemy_ptr).client as *mut gclient_t))
                    .ps
                    .holocronsCarried[(*ent).count as usize]
                    == 0.0
                {
                    (*((*enemy_ptr).client as *mut gclient_t))
                        .ps
                        .holocronCantTouch = (*ent).s.number;
                    (*((*enemy_ptr).client as *mut gclient_t))
                        .ps
                        .holocronCantTouchTime = (ctx.world.level.time + 5000) as f32;

                    HolocronRespawn(&mut *ent);
                    crate::q_math::_VectorCopy(
                        (*((*enemy_ptr).client as *mut gclient_t)).ps.origin,
                        &mut (*ent).s.pos.trBase,
                    );
                    crate::q_math::_VectorCopy(
                        (*((*enemy_ptr).client as *mut gclient_t)).ps.origin,
                        &mut (*ent).s.origin,
                    );
                    crate::q_math::_VectorCopy(
                        (*((*enemy_ptr).client as *mut gclient_t)).ps.origin,
                        &mut (*ent).r.currentOrigin,
                    );
                    // copy to person carrying's origin before popping out of them
                    HolocronPopOut(ctx, ctx.entity_id_of(ent).unwrap());
                    (*ent).enemy = None;

                    justthink(ent, ctx);
                    return;
                }

                if (*enemy_ptr).inuse == 0
                    || ((*((*enemy_ptr).client as *mut gclient_t)).ps.fallingToDeath != 0)
                {
                    if (*enemy_ptr).inuse != 0 && !(*enemy_ptr).client.is_null() {
                        (*((*enemy_ptr).client as *mut gclient_t)).ps.holocronBits &=
                            !(1 << (*ent).count);
                        (*((*enemy_ptr).client as *mut gclient_t))
                            .ps
                            .holocronsCarried[(*ent).count as usize] = 0.0;
                    }
                    (*ent).enemy = None;
                    HolocronRespawn(&mut *ent);
                    crate::q_math::_VectorCopy((*ent).s.origin2, &mut (*ent).s.pos.trBase);
                    crate::q_math::_VectorCopy((*ent).s.origin2, &mut (*ent).s.origin);
                    crate::q_math::_VectorCopy((*ent).s.origin2, &mut (*ent).r.currentOrigin);

                    (*ent).s.pos.trTime = ctx.world.level.time;

                    (*ent).pos2[0] = 0.0;

                    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));

                    justthink(ent, ctx);
                    return;
                }
            }
        }

        if (*ent).pos2[0] != 0.0 && (*ent).pos2[1] < ctx.world.level.time as f32 {
            // isn't in original place and has been there for (HOLOCRON_RESPAWN_TIME) seconds without being picked up, so respawn
            crate::q_math::_VectorCopy((*ent).s.origin2, &mut (*ent).s.pos.trBase);
            crate::q_math::_VectorCopy((*ent).s.origin2, &mut (*ent).s.origin);
            crate::q_math::_VectorCopy((*ent).s.origin2, &mut (*ent).r.currentOrigin);

            (*ent).s.pos.trTime = ctx.world.level.time;

            (*ent).pos2[0] = 0.0;

            trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));
        }

        justthink(ent, ctx);
    }
}

/// Raven `SP_misc_holocron`.
///
/// Source: `oracle/codemp/game/g_misc.c:993-1097`
pub fn SP_misc_holocron(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    use mp_bg::public::gametype::GT_HOLOCRON;
    unsafe {
        let mut dest: vec3_t;
        let mut tr: trace_t = core::mem::zeroed();

        if ctx.world.cvars.g_gametype.integer != GT_HOLOCRON {
            G_FreeEntity(ctx, ctx.entity_id_of(ent));
            return;
        }

        if crate::w_saber::HasSetSaberOnly(ctx) != qfalse {
            if (*ent).count == FP_SABER_OFFENSE
                || (*ent).count == FP_SABER_DEFENSE
                || (*ent).count == FP_SABERTHROW
            {
                // having saber holocrons in saber only mode is pointless
                G_FreeEntity(ctx, ctx.entity_id_of(ent));
                return;
            }
        }

        (*ent).s.isJediMaster = qtrue;

        (*ent).r.maxs = [8.0, 8.0, 8.0];
        (*ent).r.mins = [-8.0, -8.0, -8.0];

        (*ent).s.origin[2] += 0.1;
        (*ent).r.maxs[2] -= 0.1;

        dest = [
            (*ent).s.origin[0],
            (*ent).s.origin[1],
            (*ent).s.origin[2] - 4096.0,
        ];
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &(*ent).s.origin as *const vec3_t,
                &(*ent).r.mins as *const vec3_t,
                &(*ent).r.maxs as *const vec3_t,
                &dest as *const vec3_t,
                (*ent).s.number,
                mp_qshared::shared::surface_flags::MASK_SOLID,
            ),
        );
        if tr.startsolid != 0 {
            let msg = cstr(&format!(
                "SP_misc_holocron: misc_holocron startsolid at {}\n",
                cstr_to_str(vtos(ctx, (*ent).s.origin))
            ));
            G_Printf(ctx, msg.as_ptr());
            G_FreeEntity(ctx, ctx.entity_id_of(ent));
            return;
        }

        // add the 0.1 back after the trace
        (*ent).r.maxs[2] += 0.1;

        // allow to ride movers
        //	ent->s.groundEntityNum = tr.entityNum;

        G_SetOrigin(&mut *(ent), tr.endpos);

        if (*ent).count < 0 {
            (*ent).count = 0;
        }

        if (*ent).count >= (NUM_FORCE_POWERS) as i32 {
            (*ent).count = (NUM_FORCE_POWERS - 1) as i32;
        }
        //No longer doing this, causing too many complaints about accidentally setting no force powers at all
        //and starting a holocron game (making it basically just FFA)

        (*ent).enemy = None;

        (*ent).flags = FL_BOUNCE_HALF;

        (*ent).s.modelindex = (*ent).count - 128; //G_ModelIndex(holocronTypeModels[ent->count]);
        (*ent).s.eType = entityType_t::ET_HOLOCRON as c_int;
        (*ent).s.pos.trType = TR_GRAVITY;
        (*ent).s.pos.trTime = ctx.world.level.time;

        (*ent).r.contents = mp_qshared::shared::surface_flags::CONTENTS_TRIGGER;
        (*ent).clipmask = mp_qshared::shared::surface_flags::MASK_SOLID;

        (*ent).s.trickedentindex4 = (*ent).count;

        if crate::bg_misc::forcePowerDarkLight[(*ent).count as usize] == FORCE_DARKSIDE {
            (*ent).s.trickedentindex3 = 1;
        } else if crate::bg_misc::forcePowerDarkLight[(*ent).count as usize] == FORCE_LIGHTSIDE {
            (*ent).s.trickedentindex3 = 2;
        } else {
            (*ent).s.trickedentindex3 = 3;
        }

        (*ent).physicsObject = qtrue;

        crate::q_math::_VectorCopy((*ent).s.pos.trBase, &mut (*ent).s.origin2); // remember the spawn spot

        (*ent).touch = Some(EntTouch::HolocronTouch).into();

        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));

        (*ent).think = Some(EntThink::HolocronThink).into();
        (*ent).nextthink = ctx.world.level.time + 50;
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
    // STAGE-1: EntityId ent + Option<EntityId> other/activator; raw body re-derived verbatim (Stage-2 debt).
    let base = ctx.world.g_entities.as_mut_ptr();
    let ent: *mut gentity_t = ctx.entity_mut(ent);
    let other: *mut gentity_t = unsafe { resolve(base, other) };
    let activator: *mut gentity_t = unsafe { resolve(base, activator) };

    unsafe {
        let mut dir: vec3_t;
        let mut up: vec3_t = [0.0, 0.0, 0.0];
        let mut right: vec3_t = [0.0, 0.0, 0.0];

        // see if we have a target
        if let Some(e) = (*ent).enemy {
            let enemy_ptr = ctx.world.g_entities.as_mut_ptr().add(e.index());
            let mut d: vec3_t = [0.0, 0.0, 0.0];
            crate::q_math::_VectorSubtract((*enemy_ptr).r.currentOrigin, (*ent).s.origin, &mut d);
            VectorNormalize(&mut d);
            dir = d;
        } else {
            dir = (*ent).movedir;
        }

        // randomize a bit
        PerpendicularVector(&mut up, dir);
        CrossProduct(up, dir, &mut right);

        // C `float deg = crandom() * ent->random`: the `double` product narrows
        // to `float deg`, then feeds `VectorMA` as the scale.
        let mut deg = (ctx.world.bg_state.rng.crandom() * (*ent).random as f64) as f32;
        let mut new_dir: vec3_t = [0.0, 0.0, 0.0];
        crate::q_math::_VectorMA(dir, deg, up, &mut new_dir);
        dir = new_dir;

        deg = (ctx.world.bg_state.rng.crandom() * (*ent).random as f64) as f32;
        crate::q_math::_VectorMA(dir, deg, right, &mut new_dir);
        dir = new_dir;

        VectorNormalize(&mut dir);

        match (*ent).s.weapon {
            w if w == mp_bg::weapons::weapon_t::WP_BLASTER => {
                crate::g_weapon::WP_FireBlasterMissile(
                    ctx,
                    ctx.entity_id_of(ent).unwrap(),
                    (*ent).s.origin,
                    dir,
                    qfalse,
                );
            }
            _ => {}
        }

        G_AddEvent(
            &mut *(ent),
            mp_bg::public::entity_event::entity_event_t::EV_FIRE_WEAPON as c_int,
            0,
        );
    }
}

/// Raven `InitShooter_Finish`.
///
/// Source: `oracle/codemp/game/g_misc.c:1142-1146`
pub fn InitShooter_Finish(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    unsafe {
        (*ent).enemy = ent_id_opt(
            ctx.world.g_entities.as_mut_ptr(),
            G_PickTarget(ctx, (*ent).target),
        );
        (*ent).think = FnId::NONE;
        (*ent).nextthink = 0;
    }
}

/// Raven `InitShooter`.
///
/// Source: `oracle/codemp/game/g_misc.c:1148-1166`
pub fn InitShooter(ctx: &mut GameContext, ent: EntityId, weapon: c_int) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    unsafe {
        (*ent).use_ = Some(EntUse::Use_Shooter).into();
        (*ent).s.weapon = weapon;

        crate::g_items::RegisterItem(ctx, crate::bg_misc::BG_FindItemForWeapon(weapon));

        G_SetMovedir(&mut (*ent).s.angles, &mut (*ent).movedir);

        if (*ent).random == 0.0 {
            (*ent).random = 1.0;
        }
        // C evaluates `sin( M_PI * ent->random / 180 )` in double (M_PI and the
        // libm sin are double); narrow only on store.
        (*ent).random = (std::f64::consts::PI * (*ent).random as f64 / 180.0).sin() as f32;
        // target might be a moving object, so we can't set movedir for it
        if !(*ent).target.is_null() {
            (*ent).think = Some(EntThink::InitShooter_Finish).into();
            (*ent).nextthink = ctx.world.level.time + 500;
        }
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));
    }
}

/// Raven `SP_shooter_blaster`.
///
/// Source: `oracle/codemp/game/g_misc.c:1172-1174`
pub fn SP_shooter_blaster(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    InitShooter(
        ctx,
        ctx.entity_id_of(ent).unwrap(),
        mp_bg::weapons::weapon_t::WP_BLASTER,
    );
}

// PORT-NOTE(raw-ptr-skeleton-no-world-handle): reads `level.time`.
/// Raven `check_recharge`.
///
/// Source: `oracle/codemp/game/g_misc.c:1176-1206`
pub fn check_recharge(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    use mp_qshared::common::mp::qcommon::usercmd_button::BUTTON_USE;
    use mp_qshared::shared::sound_channel::CHAN_AUTO;
    unsafe {
        let activator = match (*ent).activator {
            Some(id) => &mut ctx.world.g_entities[id.index()] as *mut gentity_t,
            None => core::ptr::null_mut(),
        };
        let activator_cl = if activator.is_null() {
            core::ptr::null_mut()
        } else {
            (*activator).client as *mut gclient_t
        };
        if (*ent).fly_sound_debounce_time < ctx.world.level.time
            || activator.is_null()
            || activator_cl.is_null()
            || (*activator_cl).pers.cmd.buttons & BUTTON_USE == 0
        {
            if !activator.is_null() {
                G_Sound(
                    ctx,
                    ctx.entity_id_of(ent),
                    CHAN_AUTO as c_int,
                    (*ent).genericValue7,
                );
            }
            (*ent).s.loopSound = 0;
            (*ent).s.loopIsSoundset = qfalse;
            (*ent).activator = None;
            (*ent).fly_sound_debounce_time = 0;
        }

        if (*ent).activator.is_none() {
            if (*ent).genericValue8 < ctx.world.level.time {
                if (*ent).count < (*ent).genericValue4 {
                    (*ent).count += 1;
                }
                (*ent).genericValue8 = ctx.world.level.time + (*ent).genericValue5;
            }
        }
        (*ent).s.health = (*ent).count;
        (*ent).nextthink = ctx.world.level.time;
    }
}

/// Raven `EnergyShieldStationSettings`.
///
/// Source: `oracle/codemp/game/g_misc.c:1213-1223`
// PORT-NOTE(unported-const): `STATION_RECHARGE_TIME` (`g_local.h`) has no
// ported home anywhere in the crate graph; referenced verbatim.
pub fn EnergyShieldStationSettings(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    unsafe {
        G_SpawnInt(
            ctx,
            c"count".as_ptr(),
            c"200".as_ptr(),
            &mut (*ent).count as *mut c_int,
        );

        G_SpawnInt(
            ctx,
            c"chargerate".as_ptr(),
            c"0".as_ptr(),
            &mut (*ent).genericValue5 as *mut c_int,
        );

        if (*ent).genericValue5 == 0 {
            (*ent).genericValue5 = STATION_RECHARGE_TIME;
        }
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
    // STAGE-1: EntityId self_ + Option<EntityId> other/activator; raw body re-derived verbatim (Stage-2 debt).
    let base = ctx.world.g_entities.as_mut_ptr();
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    let other: *mut gentity_t = unsafe { resolve(base, other) };
    let activator: *mut gentity_t = unsafe { resolve(base, activator) };

    use mp_bg::public::gametype::GT_SIEGE;
    use mp_bg::public::stat_index::statIndex_t::{STAT_ARMOR, STAT_MAX_HEALTH};
    use mp_qshared::shared::sound_channel::CHAN_AUTO;
    unsafe {
        let mut dif: c_int;
        let mut add: c_int;
        let mut stop = true;

        if activator.is_null() || (*activator).client.is_null() {
            return;
        }

        if ctx.world.cvars.g_gametype.integer == GT_SIEGE
            && !other.is_null()
            && !(*other).client.is_null()
            && (*((*other).client as *mut gclient_t)).siegeClass != 0
        {
            if (&ctx.world.bg_state.bgSiegeClasses)
                [(*((*other).client as *mut gclient_t)).siegeClass as usize]
                .maxarmor
                == 0
            {
                // can't use it!
                G_Sound(
                    ctx,
                    ctx.entity_id_of(self_),
                    CHAN_AUTO as c_int,
                    G_SoundIndex(c"sound/interface/shieldcon_empty".as_ptr()),
                );
                return;
            }
        }

        if (*self_).setTime < ctx.world.level.time {
            let max_armor: c_int;
            if (*self_).s.loopSound == 0 {
                (*self_).s.loopSound = G_SoundIndex(c"sound/interface/shieldcon_run".as_ptr());
                (*self_).s.loopIsSoundset = qfalse;
            }
            (*self_).setTime = ctx.world.level.time + 100;

            if ctx.world.cvars.g_gametype.integer == GT_SIEGE
                && !other.is_null()
                && !(*other).client.is_null()
                && (*((*other).client as *mut gclient_t)).siegeClass != -1
            {
                max_armor = (&ctx.world.bg_state.bgSiegeClasses)
                    [(*((*other).client as *mut gclient_t)).siegeClass as usize]
                    .maxarmor;
            } else {
                max_armor =
                    (*((*activator).client as *mut gclient_t)).ps.stats[STAT_MAX_HEALTH as usize];
            }
            dif = max_armor
                - (*((*activator).client as *mut gclient_t)).ps.stats[STAT_ARMOR as usize];

            if dif > 0 {
                // Already at full armor?
                if dif > MAX_AMMO_GIVE {
                    add = MAX_AMMO_GIVE;
                } else {
                    add = dif;
                }

                if (*self_).count < add {
                    add = (*self_).count;
                }

                if (*self_).genericValue12 == 0 {
                    (*self_).count -= add;
                }
                if (*self_).count <= 0 {
                    (*self_).setTime = 0;
                }
                stop = false;

                (*self_).fly_sound_debounce_time = ctx.world.level.time + 500;
                (*self_).activator = Some(ent_id(ctx.world.g_entities.as_mut_ptr(), activator));

                (*((*activator).client as *mut gclient_t)).ps.stats[STAT_ARMOR as usize] += add;
            }
        }

        if stop || (*self_).count <= 0 {
            if (*self_).s.loopSound != 0 && (*self_).setTime < ctx.world.level.time {
                if (*self_).count <= 0 {
                    G_Sound(
                        ctx,
                        ctx.entity_id_of(self_),
                        CHAN_AUTO as c_int,
                        G_SoundIndex(c"sound/interface/shieldcon_empty".as_ptr()),
                    );
                } else {
                    G_Sound(
                        ctx,
                        ctx.entity_id_of(self_),
                        CHAN_AUTO as c_int,
                        (*self_).genericValue7,
                    );
                }
            }
            (*self_).s.loopSound = 0;
            (*self_).s.loopIsSoundset = qfalse;
            if (*self_).setTime < ctx.world.level.time {
                (*self_).setTime = ctx.world.level.time + (*self_).genericValue5 + 100;
            }
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
    // STAGE-1: EntityId self_ + Option<EntityId> other/activator; raw body re-derived verbatim (Stage-2 debt).
    let base = ctx.world.g_entities.as_mut_ptr();
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    let other: *mut gentity_t = unsafe { resolve(base, other) };
    let activator: *mut gentity_t = unsafe { resolve(base, activator) };

    use mp_bg::public::gametype::GT_SIEGE;
    use mp_bg::weapons::ammo_t::ammo_t::{AMMO_BLASTER, AMMO_MAX, AMMO_ROCKETS};
    use mp_qshared::shared::sound_channel::CHAN_AUTO;
    unsafe {
        let mut add: c_int;
        let mut stop = true;

        if activator.is_null() || (*activator).client.is_null() {
            return;
        }

        if (*self_).setTime < ctx.world.level.time {
            let mut gave_some = false;

            let mut i = AMMO_BLASTER as c_int;
            if (*self_).s.loopSound == 0 {
                (*self_).s.loopSound = G_SoundIndex(c"sound/interface/ammocon_run".as_ptr());
                (*self_).s.loopIsSoundset = qfalse;
            }
            (*self_).fly_sound_debounce_time = ctx.world.level.time + 500;
            (*self_).activator = Some(ent_id(ctx.world.g_entities.as_mut_ptr(), activator));
            while i < AMMO_MAX as c_int {
                add = (ammoData[i as usize].max as f32 * 0.05) as c_int;
                if add < 1 {
                    add = 1;
                }
                if ((*((*activator).client as *mut gclient_t)).ps.eFlags & EF_DOUBLE_AMMO != 0
                    && (*((*activator).client as *mut gclient_t)).ps.ammo[i as usize]
                        < ammoData[i as usize].max * 2)
                    || (*((*activator).client as *mut gclient_t)).ps.ammo[i as usize]
                        < ammoData[i as usize].max
                {
                    gave_some = true;
                    if ctx.world.cvars.g_gametype.integer == GT_SIEGE
                        && i == AMMO_ROCKETS as c_int
                        && (*((*activator).client as *mut gclient_t)).ps.ammo[i as usize] >= 10
                    {
                        // this stuff is already a freaking mess, so..
                        gave_some = false;
                    }
                    (*((*activator).client as *mut gclient_t)).ps.ammo[i as usize] += add;
                    if ctx.world.cvars.g_gametype.integer == GT_SIEGE
                        && i == AMMO_ROCKETS as c_int
                        && (*((*activator).client as *mut gclient_t)).ps.ammo[i as usize] >= 10
                    {
                        // fixme - this should SERIOUSLY be externed.
                        (*((*activator).client as *mut gclient_t)).ps.ammo[i as usize] = 10;
                    } else if (*((*activator).client as *mut gclient_t)).ps.eFlags & EF_DOUBLE_AMMO
                        != 0
                    {
                        if (*((*activator).client as *mut gclient_t)).ps.ammo[i as usize]
                            >= ammoData[i as usize].max * 2
                        {
                            (*((*activator).client as *mut gclient_t)).ps.ammo[i as usize] =
                                ammoData[i as usize].max * 2;
                        } else {
                            stop = false;
                        }
                    } else {
                        if (*((*activator).client as *mut gclient_t)).ps.ammo[i as usize]
                            >= ammoData[i as usize].max
                        {
                            (*((*activator).client as *mut gclient_t)).ps.ammo[i as usize] =
                                ammoData[i as usize].max;
                        } else {
                            stop = false;
                        }
                    }
                }
                i += 1;
                if (*self_).genericValue12 == 0 && gave_some {
                    let mut sub = (add as f32 * 0.2) as c_int;
                    if sub < 1 {
                        sub = 1;
                    }
                    (*self_).count -= sub;
                    if (*self_).count <= 0 {
                        (*self_).count = 0;
                        stop = true;
                        break;
                    }
                }
            }
        }

        if stop || (*self_).count <= 0 {
            if (*self_).s.loopSound != 0 && (*self_).setTime < ctx.world.level.time {
                if (*self_).count <= 0 {
                    G_Sound(
                        ctx,
                        ctx.entity_id_of(self_),
                        CHAN_AUTO as c_int,
                        G_SoundIndex(c"sound/interface/ammocon_empty".as_ptr()),
                    );
                } else {
                    G_Sound(
                        ctx,
                        ctx.entity_id_of(self_),
                        CHAN_AUTO as c_int,
                        (*self_).genericValue7,
                    );
                }
            }
            (*self_).s.loopSound = 0;
            (*self_).s.loopIsSoundset = qfalse;
            if (*self_).setTime < ctx.world.level.time {
                (*self_).setTime = ctx.world.level.time + (*self_).genericValue5 + 100;
            }
        }
    }
}

/// Raven `SP_misc_ammo_floor_unit`.
///
/// Source: `oracle/codemp/game/g_misc.c:1515-1592`
pub fn SP_misc_ammo_floor_unit(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    use mp_bg::public::gametype::GT_SIEGE;
    use mp_qshared::shared::limits::ENTITYNUM_NONE;
    unsafe {
        let mut dest: vec3_t;
        let mut tr: trace_t = core::mem::zeroed();

        (*ent).r.mins = [-16.0, -16.0, 0.0];
        (*ent).r.maxs = [16.0, 16.0, 40.0];

        (*ent).s.origin[2] += 0.1;
        (*ent).r.maxs[2] -= 0.1;

        dest = [
            (*ent).s.origin[0],
            (*ent).s.origin[1],
            (*ent).s.origin[2] - 4096.0,
        ];
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &(*ent).s.origin as *const vec3_t,
                &(*ent).r.mins as *const vec3_t,
                &(*ent).r.maxs as *const vec3_t,
                &dest as *const vec3_t,
                (*ent).s.number,
                MASK_SOLID,
            ),
        );
        if tr.startsolid != 0 {
            let msg = cstr(&format!(
                "SP_misc_ammo_floor_unit: misc_ammo_floor_unit startsolid at {}\n",
                cstr_to_str(vtos(ctx, (*ent).s.origin))
            ));
            G_Printf(ctx, msg.as_ptr());
            G_FreeEntity(ctx, ctx.entity_id_of(ent));
            return;
        }

        // add the 0.1 back after the trace
        (*ent).r.maxs[2] += 0.1;

        // allow to ride movers
        (*ent).s.groundEntityNum = (tr.entityNum) as i32;

        G_SetOrigin(&mut *(ent), tr.endpos);

        if (*ent).health == 0 {
            (*ent).health = 60;
        }

        if (*ent).model.is_null() || *(*ent).model == 0 {
            (*ent).model = c"/models/items/a_pwr_converter.md3".as_ptr() as *mut c_char;
        }

        (*ent).s.modelindex = G_ModelIndex((*ent).model);

        (*ent).s.eFlags = 0;
        (*ent).r.svFlags |= SVF_PLAYER_USABLE;
        (*ent).r.contents = CONTENTS_SOLID;
        (*ent).clipmask = MASK_SOLID;

        EnergyShieldStationSettings(ctx, ctx.entity_id_of(ent).unwrap());

        (*ent).genericValue4 = (*ent).count; // initial value
        (*ent).think = Some(EntThink::check_recharge).into();

        G_SpawnInt(
            ctx,
            c"nodrain".as_ptr(),
            c"0".as_ptr(),
            &mut (*ent).genericValue12 as *mut c_int,
        );

        if (*ent).genericValue12 == 0 {
            (*ent).s.maxhealth = (*ent).count;
            (*ent).s.health = (*ent).count;
        }
        (*ent).s.shouldtarget = qtrue;
        (*ent).s.teamowner = 0;
        (*ent).s.owner = ENTITYNUM_NONE as c_int;

        (*ent).nextthink = ctx.world.level.time + 200; // + STATION_RECHARGE_TIME

        (*ent).use_ = Some(EntUse::ammo_generic_power_converter_use).into();

        crate::q_math::_VectorCopy((*ent).s.angles, &mut (*ent).s.apos.trBase);
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));

        G_SoundIndex(c"sound/interface/ammocon_run".as_ptr());
        (*ent).genericValue7 = G_SoundIndex(c"sound/interface/ammocon_done".as_ptr());
        G_SoundIndex(c"sound/interface/ammocon_empty".as_ptr());

        if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
            // show on radar from everywhere
            (*ent).r.svFlags |= SVF_BROADCAST;
            (*ent).s.eFlags |= EF_RADAROBJECT;
            (*ent).s.genericenemyindex =
                G_IconIndex(ctx, c"gfx/mp/siegeicons/desert/weapon_recharge".as_ptr());
        }
    }
}

/// Raven `SP_misc_shield_floor_unit`.
///
/// Source: `oracle/codemp/game/g_misc.c:1602-1687`
pub fn SP_misc_shield_floor_unit(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    use mp_bg::public::gametype::{GT_CTF, GT_CTY, GT_SIEGE};
    use mp_qshared::shared::limits::ENTITYNUM_NONE;
    unsafe {
        let mut dest: vec3_t;
        let mut tr: trace_t = core::mem::zeroed();

        if ctx.world.cvars.g_gametype.integer != GT_CTF
            && ctx.world.cvars.g_gametype.integer != GT_CTY
            && ctx.world.cvars.g_gametype.integer != GT_SIEGE
        {
            G_FreeEntity(ctx, ctx.entity_id_of(ent));
            return;
        }

        (*ent).r.mins = [-16.0, -16.0, 0.0];
        (*ent).r.maxs = [16.0, 16.0, 40.0];

        (*ent).s.origin[2] += 0.1;
        (*ent).r.maxs[2] -= 0.1;

        dest = [
            (*ent).s.origin[0],
            (*ent).s.origin[1],
            (*ent).s.origin[2] - 4096.0,
        ];
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &(*ent).s.origin as *const vec3_t,
                &(*ent).r.mins as *const vec3_t,
                &(*ent).r.maxs as *const vec3_t,
                &dest as *const vec3_t,
                (*ent).s.number,
                MASK_SOLID,
            ),
        );
        if tr.startsolid != 0 {
            let msg = cstr(&format!(
                "SP_misc_shield_floor_unit: misc_shield_floor_unit startsolid at {}\n",
                cstr_to_str(vtos(ctx, (*ent).s.origin))
            ));
            G_Printf(ctx, msg.as_ptr());
            G_FreeEntity(ctx, ctx.entity_id_of(ent));
            return;
        }

        // add the 0.1 back after the trace
        (*ent).r.maxs[2] += 0.1;

        // allow to ride movers
        (*ent).s.groundEntityNum = (tr.entityNum) as i32;

        G_SetOrigin(&mut *(ent), tr.endpos);

        if (*ent).health == 0 {
            (*ent).health = 60;
        }

        if (*ent).model.is_null() || *(*ent).model == 0 {
            (*ent).model = c"/models/items/a_shield_converter.md3".as_ptr() as *mut c_char;
        }

        (*ent).s.modelindex = G_ModelIndex((*ent).model);

        (*ent).s.eFlags = 0;
        (*ent).r.svFlags |= SVF_PLAYER_USABLE;
        (*ent).r.contents = CONTENTS_SOLID;
        (*ent).clipmask = MASK_SOLID;

        EnergyShieldStationSettings(ctx, ctx.entity_id_of(ent).unwrap());

        (*ent).genericValue4 = (*ent).count;
        (*ent).think = Some(EntThink::check_recharge).into();

        G_SpawnInt(
            ctx,
            c"nodrain".as_ptr(),
            c"0".as_ptr(),
            &mut (*ent).genericValue12 as *mut c_int,
        );

        if (*ent).genericValue12 == 0 {
            (*ent).s.maxhealth = (*ent).count;
            (*ent).s.health = (*ent).count;
        }
        (*ent).s.shouldtarget = qtrue;
        (*ent).s.teamowner = 0;
        (*ent).s.owner = ENTITYNUM_NONE as c_int;

        (*ent).nextthink = ctx.world.level.time + 200;

        (*ent).use_ = Some(EntUse::shield_power_converter_use).into();

        crate::q_math::_VectorCopy((*ent).s.angles, &mut (*ent).s.apos.trBase);
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));

        G_SoundIndex(c"sound/interface/shieldcon_run".as_ptr());
        (*ent).genericValue7 = G_SoundIndex(c"sound/interface/shieldcon_done".as_ptr());
        G_SoundIndex(c"sound/interface/shieldcon_empty".as_ptr());

        if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
            (*ent).r.svFlags |= SVF_BROADCAST;
            (*ent).s.eFlags |= EF_RADAROBJECT;
            (*ent).s.genericenemyindex =
                G_IconIndex(ctx, c"gfx/mp/siegeicons/desert/shield_recharge".as_ptr());
        }
    }
}

/// Raven `SP_misc_model_shield_power_converter`.
///
/// Source: `oracle/codemp/game/g_misc.c:1697-1735`
pub fn SP_misc_model_shield_power_converter(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    use mp_qshared::shared::limits::ENTITYNUM_NONE;
    unsafe {
        if (*ent).health == 0 {
            (*ent).health = 60;
        }

        (*ent).r.mins = [-16.0, -16.0, -16.0];
        (*ent).r.maxs = [16.0, 16.0, 16.0];

        (*ent).s.modelindex = G_ModelIndex((*ent).model);

        (*ent).s.eFlags = 0;
        (*ent).r.svFlags |= SVF_PLAYER_USABLE;
        (*ent).r.contents = CONTENTS_SOLID;
        (*ent).clipmask = MASK_SOLID;

        EnergyShieldStationSettings(ctx, ctx.entity_id_of(ent).unwrap());

        (*ent).genericValue4 = (*ent).count;
        (*ent).think = Some(EntThink::check_recharge).into();

        (*ent).s.maxhealth = (*ent).count;
        (*ent).s.health = (*ent).count;
        (*ent).s.shouldtarget = qtrue;
        (*ent).s.teamowner = 0;
        (*ent).s.owner = ENTITYNUM_NONE as c_int;

        (*ent).nextthink = ctx.world.level.time + 200;

        (*ent).use_ = Some(EntUse::shield_power_converter_use).into();

        G_SetOrigin(&mut *(ent), (*ent).s.origin);
        crate::q_math::_VectorCopy((*ent).s.angles, &mut (*ent).s.apos.trBase);
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));

        //G_SoundIndex("sound/movers/objects/useshieldstation.wav");

        (*ent).s.modelindex2 = G_ModelIndex(c"/models/items/psd_big.md3".as_ptr());
        // Precache model
    }
}

/// Raven `EnergyAmmoStationSettings`.
///
/// Source: `oracle/codemp/game/g_misc.c:1743-1746`
// PORT-NOTE(seam-threading): faithful skeleton signature carries no
// `GameContext`/`&Engine` receiver, but `G_SpawnInt` needs one —
// how is state threaded in?
pub fn EnergyAmmoStationSettings(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    unsafe {
        G_SpawnInt(
            ctx,
            c"count".as_ptr(),
            c"200".as_ptr(),
            &mut (*ent).count as *mut c_int,
        );
    }
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
    // STAGE-1: EntityId self_ + Option<EntityId> other/activator; raw body re-derived verbatim (Stage-2 debt).
    let base = ctx.world.g_entities.as_mut_ptr();
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    let other: *mut gentity_t = unsafe { resolve(base, other) };
    let activator: *mut gentity_t = unsafe { resolve(base, activator) };

    use mp_bg::weapons::ammo_t::ammo_t::{AMMO_BLASTER, AMMO_MAX};
    unsafe {
        let mut add: c_int = 0;
        let mut stop = true;

        if activator.is_null() || (*activator).client.is_null() {
            return;
        }

        if (*self_).setTime < ctx.world.level.time {
            if (*self_).s.loopSound == 0 {
                (*self_).s.loopSound = G_SoundIndex(c"sound/player/pickupshield.wav".as_ptr());
            }

            (*self_).setTime = ctx.world.level.time + 100;

            if (*self_).count != 0 {
                // Has it got any power left?
                let mut i = AMMO_BLASTER as c_int;
                while i < AMMO_MAX as c_int {
                    add = (ammoData[i as usize].max as f32 * 0.1) as c_int;
                    if add < 1 {
                        add = 1;
                    }
                    if (*((*activator).client as *mut gclient_t)).ps.ammo[i as usize]
                        < ammoData[i as usize].max
                    {
                        (*((*activator).client as *mut gclient_t)).ps.ammo[i as usize] += add;
                        if (*((*activator).client as *mut gclient_t)).ps.ammo[i as usize]
                            > ammoData[i as usize].max
                        {
                            (*((*activator).client as *mut gclient_t)).ps.ammo[i as usize] =
                                ammoData[i as usize].max;
                        }
                    }
                    i += 1;
                }
                if (*self_).genericValue12 == 0 {
                    (*self_).count -= add;
                }
                stop = false;

                (*self_).fly_sound_debounce_time = ctx.world.level.time + 500;
                (*self_).activator = Some(ent_id(ctx.world.g_entities.as_mut_ptr(), activator));
            }
        }

        if stop {
            (*self_).s.loopSound = 0;
            (*self_).s.loopIsSoundset = qfalse;
        }
    }
}

/// Raven `SP_misc_model_ammo_power_converter`.
///
/// Source: `oracle/codemp/game/g_misc.c:1864-1904`
pub fn SP_misc_model_ammo_power_converter(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    use mp_qshared::shared::limits::ENTITYNUM_NONE;
    unsafe {
        if (*ent).health == 0 {
            (*ent).health = 60;
        }

        (*ent).r.mins = [-16.0, -16.0, -16.0];
        (*ent).r.maxs = [16.0, 16.0, 16.0];

        (*ent).s.modelindex = G_ModelIndex((*ent).model);

        (*ent).s.eFlags = 0;
        (*ent).r.svFlags |= SVF_PLAYER_USABLE;
        (*ent).r.contents = CONTENTS_SOLID;
        (*ent).clipmask = MASK_SOLID;

        G_SpawnInt(
            ctx,
            c"nodrain".as_ptr(),
            c"0".as_ptr(),
            &mut (*ent).genericValue12 as *mut c_int,
        );
        (*ent).use_ = Some(EntUse::ammo_power_converter_use).into();

        EnergyAmmoStationSettings(ctx, ctx.entity_id_of(ent).unwrap());

        (*ent).genericValue4 = (*ent).count;
        (*ent).think = Some(EntThink::check_recharge).into();

        if (*ent).genericValue12 == 0 {
            (*ent).s.maxhealth = (*ent).count;
            (*ent).s.health = (*ent).count;
        }
        (*ent).s.shouldtarget = qtrue;
        (*ent).s.teamowner = 0;
        (*ent).s.owner = ENTITYNUM_NONE as c_int;

        (*ent).nextthink = ctx.world.level.time + 200;

        G_SetOrigin(&mut *(ent), (*ent).s.origin);
        crate::q_math::_VectorCopy((*ent).s.angles, &mut (*ent).s.apos.trBase);
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));

        //G_SoundIndex("sound/movers/objects/useshieldstation.wav");
    }
}

/// Raven `EnergyHealthStationSettings`.
///
/// Source: `oracle/codemp/game/g_misc.c:1911-1914`
// PORT-NOTE(seam-threading): faithful skeleton signature carries no
// `GameContext`/`&Engine` receiver, but `G_SpawnInt` needs one —
// how is state threaded in?
pub fn EnergyHealthStationSettings(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    unsafe {
        G_SpawnInt(
            ctx,
            c"count".as_ptr(),
            c"200".as_ptr(),
            &mut (*ent).count as *mut c_int,
        );
    }
}

// PORT-NOTE(raw-ptr-skeleton-no-world-handle): reads `level.time`.
/// Raven `health_power_converter_use`.
///
/// Source: `oracle/codemp/game/g_misc.c:1921-1972`
pub fn health_power_converter_use(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    // STAGE-1: EntityId self_ + Option<EntityId> other/activator; raw body re-derived verbatim (Stage-2 debt).
    let base = ctx.world.g_entities.as_mut_ptr();
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    let other: *mut gentity_t = unsafe { resolve(base, other) };
    let activator: *mut gentity_t = unsafe { resolve(base, activator) };

    use mp_bg::public::stat_index::statIndex_t::STAT_MAX_HEALTH;
    use mp_qshared::shared::sound_channel::CHAN_AUTO;
    unsafe {
        let mut stop = true;

        if activator.is_null() || (*activator).client.is_null() {
            return;
        }

        if (*self_).setTime < ctx.world.level.time {
            if (*self_).s.loopSound == 0 {
                (*self_).s.loopSound = G_SoundIndex(c"sound/player/pickuphealth.wav".as_ptr());
            }
            (*self_).setTime = ctx.world.level.time + 100;

            let cl = &mut *((*activator).client as *mut gclient_t);
            let dif = cl.ps.stats[STAT_MAX_HEALTH as usize] - (*activator).health;

            if dif > 0 {
                let mut add = if dif > 5 { 5 } else { dif };
                if (*self_).count < add {
                    add = (*self_).count;
                }

                stop = false;

                (*self_).fly_sound_debounce_time = ctx.world.level.time + 500;
                (*self_).activator = ent_id_opt(ctx.world.g_entities.as_mut_ptr(), activator);

                (*activator).health += add;
            }
        }

        if stop {
            (*self_).s.loopSound = 0;
            (*self_).s.loopIsSoundset = qfalse;
        }
    }
}

/// Raven `SP_misc_model_health_power_converter`.
///
/// Source: `oracle/codemp/game/g_misc.c:1982-2027`
pub fn SP_misc_model_health_power_converter(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    use mp_bg::public::gametype::GT_SIEGE;
    use mp_qshared::shared::limits::ENTITYNUM_NONE;
    unsafe {
        if (*ent).health == 0 {
            (*ent).health = 60;
        }

        (*ent).r.mins = [-16.0, -16.0, -16.0];
        (*ent).r.maxs = [16.0, 16.0, 16.0];

        (*ent).s.modelindex = G_ModelIndex((*ent).model);

        (*ent).s.eFlags = 0;
        (*ent).r.svFlags |= SVF_PLAYER_USABLE;
        (*ent).r.contents = CONTENTS_SOLID;
        (*ent).clipmask = MASK_SOLID;

        (*ent).use_ = Some(EntUse::health_power_converter_use).into();

        EnergyHealthStationSettings(ctx, ctx.entity_id_of(ent).unwrap());

        (*ent).genericValue4 = (*ent).count;
        (*ent).think = Some(EntThink::check_recharge).into();

        //ent->s.maxhealth = ent->s.health = ent->count;
        (*ent).s.shouldtarget = qtrue;
        (*ent).s.teamowner = 0;
        (*ent).s.owner = ENTITYNUM_NONE as c_int;

        (*ent).nextthink = ctx.world.level.time + 200;

        G_SetOrigin(&mut *(ent), (*ent).s.origin);
        crate::q_math::_VectorCopy((*ent).s.angles, &mut (*ent).s.apos.trBase);
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));

        //G_SoundIndex("sound/movers/objects/useshieldstation.wav");
        G_SoundIndex(c"sound/player/pickuphealth.wav".as_ptr());
        (*ent).genericValue7 = G_SoundIndex(c"sound/interface/shieldcon_done".as_ptr());

        if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
            // show on radar from everywhere
            (*ent).r.svFlags |= SVF_BROADCAST;
            (*ent).s.eFlags |= EF_RADAROBJECT;
            (*ent).s.genericenemyindex =
                G_IconIndex(ctx, c"gfx/mp/siegeicons/desert/bacta".as_ptr());
        }
    }
}

/// Raven `fx_runner_think`.
///
/// Source: `oracle/codemp/game/g_misc.c:2266-2310`
pub fn fx_runner_think(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    unsafe {
        crate::bg_misc::BG_EvaluateTrajectory(
            &(*ent).s.pos as *const trajectory_t,
            ctx.world.level.time,
            &mut (*ent).r.currentOrigin,
        );
        crate::bg_misc::BG_EvaluateTrajectory(
            &(*ent).s.apos as *const trajectory_t,
            ctx.world.level.time,
            &mut (*ent).r.currentAngles,
        );

        // call the effect with the desired position and orientation
        if (*ent).s.isPortalEnt != 0 {
            //		G_AddEvent( ent, EV_PLAY_PORTAL_EFFECT_ID, ent->genericValue5 );
        } else {
            //		G_AddEvent( ent, EV_PLAY_EFFECT_ID, ent->genericValue5 );
        }

        // start the fx on the client (continuous)
        (*ent).s.modelindex2 = FX_STATE_CONTINUOUS;

        crate::q_math::_VectorCopy((*ent).r.currentAngles, &mut (*ent).s.angles);
        crate::q_math::_VectorCopy((*ent).r.currentOrigin, &mut (*ent).s.origin);

        (*ent).nextthink = ctx.world.level.time
            + (*ent).delay
            + (ctx.world.bg_state.rng.random() * (*ent).random) as c_int;

        if (*ent).spawnflags & 4 != 0 {
            // damage
            G_RadiusDamage(
                ctx,
                (*ent).r.currentOrigin,
                ctx.entity_id_of(ent),
                (*ent).splashDamage as f32,
                (*ent).splashRadius as f32,
                ctx.entity_id_of(ent),
                ctx.entity_id_of(ent),
                MOD_UNKNOWN as c_int,
            );
        }

        if !(*ent).target2.is_null() && *(*ent).target2 != 0 {
            // let our target know that we have spawned an effect
            G_UseTargets2(
                ctx,
                ctx.entity_id_of(ent),
                ctx.entity_id_of(ent),
                (*ent).target2,
            );
        }

        if (*ent).spawnflags & 2 == 0 && (*ent).s.loopSound == 0 {
            // NOT ONESHOT...this is an assy thing to do
            if !(*ent).soundSet.is_null() && *(*ent).soundSet != 0 {
                (*ent).s.soundSetIndex = G_SoundSetIndex(ctx, (*ent).soundSet);
                (*ent).s.loopIsSoundset = qtrue;
                (*ent).s.loopSound = BMS_MID;
            }
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
    // STAGE-1: EntityId self_ + Option<EntityId> other/activator; raw body re-derived verbatim (Stage-2 debt).
    let base = ctx.world.g_entities.as_mut_ptr();
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    let other: *mut gentity_t = unsafe { resolve(base, other) };
    let activator: *mut gentity_t = unsafe { resolve(base, activator) };

    unsafe {
        if (*self_).s.isPortalEnt != 0 {
            // rww - mark it as broadcast upon first use if it's within the area of a skyportal
            (*self_).r.svFlags |= SVF_BROADCAST;
        }

        if (*self_).spawnflags & 2 != 0 {
            // ONESHOT
            // call the effect with the desired position and orientation, as a safety thing,
            //	make sure we aren't thinking at all.
            let save_state = (*self_).s.modelindex2 + 1;

            fx_runner_think(ctx, ctx.entity_id_of(self_).unwrap());
            (*self_).nextthink = -1;
            // one shot indicator
            (*self_).s.modelindex2 = save_state;
            if (*self_).s.modelindex2 > FX_STATE_ONE_SHOT_LIMIT {
                (*self_).s.modelindex2 = FX_STATE_ONE_SHOT;
            }

            if !(*self_).target2.is_null() {
                // let our target know that we have spawned an effect
                G_UseTargets2(
                    ctx,
                    ctx.entity_id_of(self_),
                    ctx.entity_id_of(self_),
                    (*self_).target2,
                );
            }

            if !(*self_).soundSet.is_null() && *(*self_).soundSet != 0 {
                (*self_).s.soundSetIndex = G_SoundSetIndex(ctx, (*self_).soundSet);
                G_AddEvent(
                    &mut *(self_),
                    mp_bg::public::entity_event::entity_event_t::EV_BMODEL_SOUND as c_int,
                    BMS_START,
                );
            }
        } else {
            // ensure we are working with the right think function
            (*self_).think = Some(EntThink::fx_runner_think).into();

            // toggle our state
            if (*self_).nextthink == -1 {
                // NOTE: we fire the effect immediately on use, the fx_runner_think func will set
                //	up the nextthink time.
                fx_runner_think(ctx, ctx.entity_id_of(self_).unwrap());

                if !(*self_).soundSet.is_null() && *(*self_).soundSet != 0 {
                    (*self_).s.soundSetIndex = G_SoundSetIndex(ctx, (*self_).soundSet);
                    G_AddEvent(
                        &mut *(self_),
                        mp_bg::public::entity_event::entity_event_t::EV_BMODEL_SOUND as c_int,
                        BMS_START,
                    );
                    (*self_).s.loopSound = BMS_MID;
                    (*self_).s.loopIsSoundset = qtrue;
                }
            } else {
                // turn off for now
                (*self_).nextthink = -1;

                // turn off fx on client
                (*self_).s.modelindex2 = FX_STATE_OFF;

                if !(*self_).soundSet.is_null() && *(*self_).soundSet != 0 {
                    (*self_).s.soundSetIndex = G_SoundSetIndex(ctx, (*self_).soundSet);
                    G_AddEvent(
                        &mut *(self_),
                        mp_bg::public::entity_event::entity_event_t::EV_BMODEL_SOUND as c_int,
                        BMS_END,
                    );
                    (*self_).s.loopSound = 0;
                    (*self_).s.loopIsSoundset = qfalse;
                }
            }
        }
    }
}

/// Raven `fx_runner_link`.
///
/// Source: `oracle/codemp/game/g_misc.c:2387-2453`
pub fn fx_runner_link(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    unsafe {
        let mut dir: vec3_t;

        if !(*ent).target.is_null() && *(*ent).target != 0 {
            // try to use the target to override the orientation
            let target = G_Find(
                ctx,
                ctx.entity_id_of(core::ptr::null_mut()),
                core::mem::offset_of!(gentity_t, targetname) as c_int,
                (*ent).target,
            );

            if target.is_null() {
                // Bah, no good, dump a warning, but continue on and use the UP vector
                crate::g_main::Com_Printf(
                    cstr(&format!(
                        "fx_runner_link: target specified but not found: {}\n",
                        cstr_to_str((*ent).target)
                    ))
                    .as_ptr(),
                );
                crate::g_main::Com_Printf(c"  -assuming UP orientation.\n".as_ptr());
            } else {
                // Our target is valid so let's override the default UP vector
                let mut d: vec3_t = [0.0, 0.0, 0.0];
                crate::q_math::_VectorSubtract((*target).s.origin, (*ent).s.origin, &mut d);
                VectorNormalize(&mut d);
                vectoangles(d, &mut (*ent).s.angles);
            }
        }

        // don't really do anything with this right now other than do a check to warn the designers if the target2 is bogus
        if !(*ent).target2.is_null() && *(*ent).target2 != 0 {
            let target = G_Find(
                ctx,
                ctx.entity_id_of(core::ptr::null_mut()),
                core::mem::offset_of!(gentity_t, targetname) as c_int,
                (*ent).target2,
            );

            if target.is_null() {
                // Target2 is bogus, but we can still continue
                crate::g_main::Com_Printf(
                    cstr(&format!(
                        "fx_runner_link: target2 was specified but is not valid: {}\n",
                        cstr_to_str((*ent).target2)
                    ))
                    .as_ptr(),
                );
            }
        }

        G_SetAngles(&mut *(ent), (*ent).s.angles);

        if (*ent).spawnflags & 1 != 0 || (*ent).spawnflags & 2 != 0 {
            // STARTOFF || ONESHOT
            // We won't even consider thinking until we are used
            (*ent).nextthink = -1;
        } else {
            if !(*ent).soundSet.is_null() && *(*ent).soundSet != 0 {
                (*ent).s.soundSetIndex = G_SoundSetIndex(ctx, (*ent).soundSet);
                (*ent).s.loopSound = BMS_MID;
                (*ent).s.loopIsSoundset = qtrue;
            }

            // Let's get to work right now!
            (*ent).think = Some(EntThink::fx_runner_think).into();
            (*ent).nextthink = ctx.world.level.time + 200; // wait a small bit, then start working
        }

        // make us useable if we can be targeted
        if !(*ent).targetname.is_null() && *(*ent).targetname != 0 {
            (*ent).use_ = Some(EntUse::fx_runner_use).into();
        }
    }
}

/// Raven `SP_fx_runner`.
///
/// Source: `oracle/codemp/game/g_misc.c:2456-2501`
pub fn SP_fx_runner(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    unsafe {
        let mut fx_file: *mut c_char = core::ptr::null_mut();

        G_SpawnString(ctx, c"fxFile".as_ptr(), c"".as_ptr(), &mut fx_file);
        // Get our defaults
        G_SpawnInt(
            ctx,
            c"delay".as_ptr(),
            c"200".as_ptr(),
            &mut (*ent).delay as *mut c_int,
        );
        G_SpawnFloat(
            ctx,
            c"random".as_ptr(),
            c"0".as_ptr(),
            &mut (*ent).random as *mut f32,
        );
        G_SpawnInt(
            ctx,
            c"splashRadius".as_ptr(),
            c"16".as_ptr(),
            &mut (*ent).splashRadius as *mut c_int,
        );
        G_SpawnInt(
            ctx,
            c"splashDamage".as_ptr(),
            c"5".as_ptr(),
            &mut (*ent).splashDamage as *mut c_int,
        );

        if (*ent).s.angles[0] == 0.0 && (*ent).s.angles[1] == 0.0 && (*ent).s.angles[2] == 0.0 {
            // didn't have angles, so give us the default of up
            (*ent).s.angles = [-90.0, 0.0, 0.0];
        }

        if fx_file.is_null() || *fx_file == 0 {
            crate::g_main::Com_Printf(
                cstr(&format!(
                    "^1ERROR: fx_runner {} at {} has no fxFile specified\n",
                    cstr_to_str((*ent).targetname),
                    cstr_to_str(vtos(ctx, (*ent).s.origin))
                ))
                .as_ptr(),
            );
            G_FreeEntity(ctx, ctx.entity_id_of(ent));
            return;
        }

        // Try and associate an effect file, unfortunately we won't know if this worked or not
        //	until the CGAME trys to register it...
        (*ent).s.modelindex = G_EffectIndex(fx_file);

        // important info transmitted
        (*ent).s.eType = entityType_t::ET_FX as c_int;
        (*ent).s.speed = (*ent).delay as f32;
        (*ent).s.time = (*ent).random as c_int;
        (*ent).s.modelindex2 = FX_STATE_OFF;

        // Give us a bit of time to spawn in the other entities, since we may have to target one of 'em
        (*ent).think = Some(EntThink::fx_runner_link).into();
        (*ent).nextthink = ctx.world.level.time + 400;

        // Save our position and link us up!
        G_SetOrigin(&mut *(ent), (*ent).s.origin);

        (*ent).r.maxs = [FX_ENT_RADIUS, FX_ENT_RADIUS, FX_ENT_RADIUS];
        crate::q_math::_VectorScale((*ent).r.maxs, -1.0, &mut (*ent).r.mins);

        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));
    }
}

/// Raven `SP_CreateSpaceDust`.
///
/// Source: `oracle/codemp/game/g_misc.c:2509-2513`
pub fn SP_CreateSpaceDust(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    unsafe {
        G_EffectIndex(cstr(&format!("*spacedust {}", (*ent).count)).as_ptr());
        //G_EffectIndex("*constantwind ( 10 -10 0 )");
    }
}

/// Raven `SP_CreateSnow`.
///
/// Source: `oracle/codemp/game/g_misc.c:2522-2527`
pub fn SP_CreateSnow(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    G_EffectIndex(b"*snow\0".as_ptr() as *const c_char);
    G_EffectIndex(b"*fog\0".as_ptr() as *const c_char);
    G_EffectIndex(b"*constantwind (100 100 -100)\0".as_ptr() as *const c_char);
}

/// Raven `SP_CreateRain`.
///
/// Source: `oracle/codemp/game/g_misc.c:2535-2538`
pub fn SP_CreateRain(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    unsafe {
        G_EffectIndex(cstr(&format!("*rain init {}", (*ent).count)).as_ptr());
    }
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
    // STAGE-1: EntityId ent + Option<EntityId> other/activator; raw body re-derived verbatim (Stage-2 debt).
    let base = ctx.world.g_entities.as_mut_ptr();
    let ent: *mut gentity_t = ctx.entity_mut(ent);
    let other: *mut gentity_t = unsafe { resolve(base, other) };
    let activator: *mut gentity_t = unsafe { resolve(base, activator) };

    unsafe {
        let bGlobal: qboolean = if (*ent).genericValue6 != 0 {
            qtrue
        } else {
            qfalse
        };
        G_ScreenShake(
            ctx,
            (*ent).s.origin,
            ctx.entity_id_of(std::ptr::null_mut()),
            (*ent).speed,
            (*ent).genericValue5,
            bGlobal,
        );
    }
}

// PORT-NOTE(raw-ptr-skeleton-no-world-handle): faithful body has no
// trap/level dependency itself, but is a fn-pointer write
// (`use_ = Use_Target_Screenshake`) the `gentity_t` field type
// cannot yet express.
/// Raven `SP_target_screenshake`.
///
/// Source: `oracle/codemp/game/g_misc.c:2555-2565`
pub fn SP_target_screenshake(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    unsafe {
        G_SpawnFloat(
            ctx,
            c"intensity".as_ptr(),
            c"10".as_ptr(),
            &mut (*ent).speed as *mut f32,
        );
        G_SpawnInt(
            ctx,
            c"duration".as_ptr(),
            c"800".as_ptr(),
            &mut (*ent).genericValue5 as *mut c_int,
        );
        G_SpawnInt(
            ctx,
            c"globalshake".as_ptr(),
            c"1".as_ptr(),
            &mut (*ent).genericValue6 as *mut c_int,
        );

        (*ent).use_ = Some(EntUse::Use_Target_Screenshake).into();
    }
}

// PORT-NOTE(unported-const): `PMF_FOLLOW` (`bg_public.h` pmove flags) has no
// ported home anywhere in the crate graph; referenced verbatim.
/// Raven `Use_Target_Escapetrig`.
///
/// Source: `oracle/codemp/game/g_misc.c:2569-2597`
pub fn Use_Target_Escapetrig(
    ctx: &mut GameContext,
    ent: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    // STAGE-1: EntityId ent + Option<EntityId> other/activator; raw body re-derived verbatim (Stage-2 debt).
    let base = ctx.world.g_entities.as_mut_ptr();
    let ent: *mut gentity_t = ctx.entity_mut(ent);
    let other: *mut gentity_t = unsafe { resolve(base, other) };
    let activator: *mut gentity_t = unsafe { resolve(base, activator) };

    use mp_bg::public::team::TEAM_SPECTATOR;
    unsafe {
        if (*ent).genericValue6 == 0 {
            ctx.world.globals.gEscaping = qtrue;
            ctx.world.globals.gEscapeTime = ctx.world.level.time + (*ent).genericValue5;
        } else if ctx.world.globals.gEscaping != qfalse {
            let mut i: c_int = 0;
            ctx.world.globals.gEscaping = qfalse;
            while i < mp_qshared::shared::MAX_CLIENTS_I32 {
                let e = &mut ctx.world.g_entities[i as usize] as *mut gentity_t;
                if (*e).inuse != 0
                    && !(*e).client.is_null()
                    && (*e).health > 0
                    && (*((*e).client as *mut gclient_t)).sess.sessionTeam != TEAM_SPECTATOR
                    && (*((*e).client as *mut gclient_t)).ps.pm_flags & PMF_FOLLOW == 0
                {
                    // all of the survivors get 100 points!
                    AddScore(
                        ctx,
                        ctx.entity_id_of(e).unwrap(),
                        (*((*e).client as *mut gclient_t)).ps.origin,
                        100,
                    );
                }
                i += 1;
            }
            if !activator.is_null() && (*activator).inuse != 0 && !(*activator).client.is_null() {
                // the one who escaped gets 500
                AddScore(
                    ctx,
                    ctx.entity_id_of(activator).unwrap(),
                    (*((*activator).client as *mut gclient_t)).ps.origin,
                    500,
                );
            }

            LogExit(ctx, c"Escaped!".as_ptr());
        }
    }
}

// PORT-NOTE(raw-ptr-skeleton-no-world-handle): reads `g_gametype`; also
// a fn-pointer write (`use_ = Use_Target_Escapetrig`).
/// Raven `SP_target_escapetrig`.
///
/// Source: `oracle/codemp/game/g_misc.c:2599-2613`
pub fn SP_target_escapetrig(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    use mp_bg::public::gametype::GT_SINGLE_PLAYER;
    unsafe {
        if ctx.world.cvars.g_gametype.integer != GT_SINGLE_PLAYER {
            G_FreeEntity(ctx, ctx.entity_id_of(ent));
            return;
        }

        G_SpawnInt(
            ctx,
            c"escapetime".as_ptr(),
            c"60000".as_ptr(),
            &mut (*ent).genericValue5 as *mut c_int,
        );
        G_SpawnInt(
            ctx,
            c"escapegoal".as_ptr(),
            c"0".as_ptr(),
            &mut (*ent).genericValue6 as *mut c_int,
        );

        (*ent).use_ = Some(EntUse::Use_Target_Escapetrig).into();
    }
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
    // STAGE-1: EntityId self_ + Option<EntityId> inflictor/attacker; raw body re-derived verbatim (Stage-2 debt).
    let base = ctx.world.g_entities.as_mut_ptr();
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    let inflictor: *mut gentity_t = unsafe { resolve(base, inflictor) };
    let attacker: *mut gentity_t = unsafe { resolve(base, attacker) };

    use crate::entity::flags::FL_INACTIVE;
    unsafe {
        if let Some(door_id) = (*self_).activator {
            let door = &mut ctx.world.g_entities[door_id.index()] as *mut gentity_t;
            (*door).lockCount -= 1;
            if (*door).lockCount == 0 {
                (*door).flags &= !FL_INACTIVE;
            }
        }
        G_UseTargets(ctx, ctx.entity_id_of(self_), ctx.entity_id_of(attacker));
    }
}

// PORT-NOTE(unported-const): `START_TIME_FIND_LINKS` has no ported home;
// referenced verbatim.
/// Raven `SP_misc_maglock`.
///
/// Source: `oracle/codemp/game/g_misc.c:2645-2658`
pub fn SP_misc_maglock(ctx: &mut GameContext, self_: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);

    unsafe {
        // NOTE: May have to make these only work on doors that are either untargeted
        //		or are targeted by a trigger, not doors fired off by scripts, counters
        //		or other such things?
        (*self_).s.modelindex =
            G_ModelIndex(c"models/map_objects/imp_detention/door_lock.md3".as_ptr());
        (*self_).genericValue1 = G_EffectIndex(c"maglock/explosion".as_ptr());

        G_SetOrigin(&mut *(self_), (*self_).s.origin);

        (*self_).think = Some(EntThink::maglock_link).into();
        //FIXME: for some reason, when you re-load a level, these fail to find their doors...?  Random?  Testing an additional 200ms after the START_TIME_FIND_LINKS
        (*self_).nextthink = ctx.world.level.time + START_TIME_FIND_LINKS + 200;
        //because we need to let the doors link up and spawn their triggers first!
    }
}

/// Raven `maglock_link`.
///
/// Source: `oracle/codemp/game/g_misc.c:2659-2728`
pub fn maglock_link(ctx: &mut GameContext, self_: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);

    use crate::q_math::vectoangles;
    use mp_qshared::shared::error_parm::errorParm_t::ERR_DROP;
    use mp_qshared::shared::limits::ENTITYNUM_WORLD;
    unsafe {
        // find what we're supposed to be attached to
        let mut forward: vec3_t = [0.0, 0.0, 0.0];
        let mut start: vec3_t = [0.0, 0.0, 0.0];
        let mut end: vec3_t = [0.0, 0.0, 0.0];
        let mut trace: trace_t = core::mem::zeroed();

        AngleVectors((*self_).s.angles, Some(&mut forward), None, None);
        crate::q_math::_VectorMA((*self_).s.origin, 128.0, forward, &mut end);
        crate::q_math::_VectorMA((*self_).s.origin, -4.0, forward, &mut start);

        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut trace as *mut trace_t,
                &start as *const vec3_t,
                &vec3_origin as *const vec3_t,
                &vec3_origin as *const vec3_t,
                &end as *const vec3_t,
                (*self_).s.number,
                MASK_SHOT,
            ),
        );

        if trace.allsolid != 0 || trace.startsolid != 0 {
            crate::g_main::Com_Error(
                ERR_DROP as c_int,
                cstr(&format!(
                    "misc_maglock at {} in solid\n",
                    cstr_to_str(vtos(ctx, (*self_).s.origin))
                ))
                .as_ptr(),
            );
            G_FreeEntity(ctx, ctx.entity_id_of(self_));
            return;
        }
        if trace.fraction == 1.0 {
            (*self_).think = Some(EntThink::maglock_link).into();
            (*self_).nextthink = ctx.world.level.time + 100;
            return;
        }
        let trace_ent = &mut ctx.world.g_entities[trace.entityNum as usize] as *mut gentity_t;
        if trace.entityNum >= (ENTITYNUM_WORLD as c_int) as i16
            || trace_ent.is_null()
            || Q_stricmp(c"func_door".as_ptr(), (*trace_ent).classname) != 0
        {
            (*self_).think = Some(EntThink::maglock_link).into();
            (*self_).nextthink = ctx.world.level.time + 100;
            return;
        }

        // check the traceEnt, make sure it's a door and give it a lockCount and deactivate it
        // find the trigger for the door
        let door_trigger = G_FindDoorTrigger(ctx, ctx.entity_id_of(trace_ent).unwrap());
        (*self_).activator = if !door_trigger.is_null() {
            Some(ent_id(ctx.world.g_entities.as_mut_ptr(), door_trigger))
        } else {
            Some(ent_id(ctx.world.g_entities.as_mut_ptr(), trace_ent))
        };
        let activator_ptr = ctx
            .world
            .g_entities
            .as_mut_ptr()
            .add((*self_).activator.unwrap().index());
        (*activator_ptr).lockCount += 1;
        (*activator_ptr).flags |= FL_INACTIVE;

        // now position and orient it
        vectoangles(trace.plane.normal, &mut end);
        G_SetOrigin(&mut *(self_), trace.endpos);
        G_SetAngles(&mut *(self_), end);

        // make it hittable
        (*self_).r.mins = [-8.0, -8.0, -8.0];
        (*self_).r.maxs = [8.0, 8.0, 8.0];
        (*self_).r.contents = CONTENTS_CORPSE;

        // make it destroyable
        (*self_).flags |= FL_SHIELDED; // only damagable by lightsabers
        (*self_).takedamage = qtrue;
        (*self_).health = 10;
        (*self_).die = Some(EntDie::maglock_die).into();

        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(self_));
    }
}

// PORT-NOTE(raw-ptr-skeleton-no-world-handle): reads `level.time`.
/// Raven `faller_touch`.
///
/// Source: `oracle/codemp/game/g_misc.c:2730-2756`
pub fn faller_touch(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    trace: *mut trace_t,
) {
    // STAGE-1: EntityId self_ + Option<EntityId> other; raw body re-derived verbatim (Stage-2 debt).
    let base = ctx.world.g_entities.as_mut_ptr();
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    let other: *mut gentity_t = unsafe { resolve(base, other) };

    use mp_qshared::shared::sound_channel::{CHAN_AUTO, CHAN_VOICE};
    unsafe {
        if (*self_).epVelocity[2] < -100.0 && (*self_).genericValue7 < ctx.world.level.time {
            let r = ctx.world.bg_state.rng.Q_irand(1, 3);

            (*self_).genericValue11 = if r == 1 {
                G_SoundIndex(c"sound/chars/stofficer1/misc/pain25".as_ptr())
            } else if r == 2 {
                G_SoundIndex(c"sound/chars/stofficer1/misc/pain50".as_ptr())
            } else {
                G_SoundIndex(c"sound/chars/stofficer1/misc/pain75".as_ptr())
            };

            G_EntitySound(
                ctx,
                ctx.entity_id_of(self_).unwrap(),
                CHAN_VOICE as c_int,
                (*self_).genericValue11,
            );
            G_EntitySound(
                ctx,
                ctx.entity_id_of(self_).unwrap(),
                CHAN_AUTO as c_int,
                (*self_).genericValue10,
            );

            (*self_).genericValue6 = ctx.world.level.time + 3000;
            (*self_).genericValue7 = ctx.world.level.time + 200;
        }
    }
}

// PORT-NOTE(raw-ptr-skeleton-no-world-handle): reads `level.time`; also
// a fn-pointer write (`think = G_FreeEntity`).
/// Raven `faller_think`.
///
/// Source: `oracle/codemp/game/g_misc.c:2758-2787`
pub fn faller_think(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    use mp_qshared::shared::sound_channel::CHAN_VOICE;
    unsafe {
        let gravity: f32 = 3.0;
        let mass: f32 = 0.09;
        let bounce: f32 = 1.1;

        if (*ent).genericValue6 < ctx.world.level.time {
            (*ent).think = Some(EntThink::G_FreeEntity).into();
            (*ent).nextthink = ctx.world.level.time;
            return;
        }

        if (*ent).epVelocity[2] < -100.0 {
            if (*ent).genericValue8 == 0 {
                G_EntitySound(
                    ctx,
                    ctx.entity_id_of(ent).unwrap(),
                    CHAN_VOICE as c_int,
                    (*ent).genericValue9,
                );
                (*ent).genericValue8 = 1;
            }
        } else {
            (*ent).genericValue8 = 0;
        }

        G_RunExPhys(
            ctx,
            ctx.entity_id_of(ent).unwrap(),
            gravity,
            mass,
            bounce,
            qtrue,
            core::ptr::null_mut(),
            0,
        );
        (*ent).s.pos.trDelta = [
            (*ent).epVelocity[0] * 10.0,
            (*ent).epVelocity[1] * 10.0,
            (*ent).epVelocity[2] * 10.0,
        ];
        (*ent).nextthink = ctx.world.level.time + 25;
    }
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
    // STAGE-1: EntityId ent + Option<EntityId> other/activator; raw body re-derived verbatim (Stage-2 debt).
    let base = ctx.world.g_entities.as_mut_ptr();
    let ent: *mut gentity_t = ctx.entity_mut(ent);
    let other: *mut gentity_t = unsafe { resolve(base, other) };
    let activator: *mut gentity_t = unsafe { resolve(base, activator) };

    unsafe {
        let faller = G_Spawn(ctx);

        (*faller).genericValue10 = G_SoundIndex(c"sound/player/fallsplat".as_ptr());
        (*faller).genericValue9 = G_SoundIndex(c"sound/chars/stofficer1/misc/falling1".as_ptr());
        (*faller).genericValue8 = 0;
        (*faller).genericValue7 = 0;

        (*faller).genericValue6 = ctx.world.level.time + 15000;

        G_SetOrigin(&mut *(faller), (*ent).s.origin);

        (*faller).s.modelGhoul2 = 1;
        (*faller).s.modelindex = G_ModelIndex(c"models/players/stormtrooper/model.glm".as_ptr());
        (*faller).s.g2radius = 100;

        (*faller).s.customRGBA[0] = (ctx.world.bg_state.rng.Q_irand(1, 255) as u8) as i32;
        (*faller).s.customRGBA[1] = (ctx.world.bg_state.rng.Q_irand(1, 255) as u8) as i32;
        (*faller).s.customRGBA[2] = (ctx.world.bg_state.rng.Q_irand(1, 255) as u8) as i32;
        (*faller).s.customRGBA[3] = 255;

        (*faller).r.mins = [-15.0, -15.0, DEFAULT_MINS_2 as f32];
        (*faller).r.maxs = [15.0, 15.0, DEFAULT_MAXS_2 as f32];

        (*faller).clipmask = MASK_PLAYERSOLID;
        (*faller).r.contents = MASK_PLAYERSOLID;

        (*faller).s.eFlags = EF_RAG | EF_CLIENTSMOOTH;

        (*faller).think = Some(EntThink::faller_think).into();
        (*faller).nextthink = ctx.world.level.time;

        (*faller).touch = Some(EntTouch::faller_touch).into();

        (*faller).epVelocity[0] = ctx.world.bg_state.rng.flrand(-256.0, 256.0);
        (*faller).epVelocity[1] = ctx.world.bg_state.rng.flrand(-256.0, 256.0);

        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(faller));
    }
}

// PORT-NOTE(raw-ptr-skeleton-no-world-handle): reads `level.time`.
/// Raven `misc_faller_think`.
///
/// Source: `oracle/codemp/game/g_misc.c:2830-2834`
pub fn misc_faller_think(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    unsafe {
        misc_faller_create(
            ctx,
            ctx.entity_id_of(ent).unwrap(),
            ctx.entity_id_of(ent),
            ctx.entity_id_of(ent),
        );
        (*ent).nextthink = ctx.world.level.time
            + (*ent).genericValue1
            + ctx.world.bg_state.rng.Q_irand(0, (*ent).genericValue2);
    }
}

// PORT-NOTE(raw-ptr-skeleton-no-world-handle): reads `level.time`; also
// fn-pointer writes (`think = misc_faller_think`, `use_ = misc_faller_create`).
/// Raven `SP_misc_faller`.
///
/// Source: `oracle/codemp/game/g_misc.c:2844-2865`
pub fn SP_misc_faller(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    unsafe {
        G_ModelIndex(c"models/players/stormtrooper/model.glm".as_ptr());
        G_SoundIndex(c"sound/chars/stofficer1/misc/pain25".as_ptr());
        G_SoundIndex(c"sound/chars/stofficer1/misc/pain50".as_ptr());
        G_SoundIndex(c"sound/chars/stofficer1/misc/pain75".as_ptr());
        G_SoundIndex(c"sound/chars/stofficer1/misc/falling1".as_ptr());
        G_SoundIndex(c"sound/player/fallsplat".as_ptr());

        G_SpawnInt(
            ctx,
            c"interval".as_ptr(),
            c"500".as_ptr(),
            &mut (*ent).genericValue1 as *mut c_int,
        );
        G_SpawnInt(
            ctx,
            c"fudgefactor".as_ptr(),
            c"0".as_ptr(),
            &mut (*ent).genericValue2 as *mut c_int,
        );

        if (*ent).targetname.is_null() || *(*ent).targetname == 0 {
            (*ent).think = Some(EntThink::misc_faller_think).into();
            (*ent).nextthink = ctx.world.level.time
                + (*ent).genericValue1
                + ctx.world.bg_state.rng.Q_irand(0, (*ent).genericValue2);
        } else {
            (*ent).use_ = Some(EntUse::misc_faller_create).into();
        }
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

    crate::g_main::Com_Printf(
        cstr(&format!(
            "WARNING: MAX_TAG_OWNERS ({}) REF TAG LIMIT HIT\n",
            MAX_TAG_OWNERS
        ))
        .as_ptr(),
    );
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

        crate::g_main::Com_Printf(
            cstr(&format!(
                "WARNING: MAX_TAGS ({}) REF TAG LIMIT HIT\n",
                MAX_TAGS
            ))
            .as_ptr(),
        );
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
            && Q_stricmp(ctx.world.refTagOwnerMap[i as usize].name.as_ptr(), owner) == 0
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
                && Q_stricmp((*owner_ptr).tags[i as usize].name.as_ptr(), name) == 0
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
                && Q_stricmp((*generic_ptr).tags[i as usize].name.as_ptr(), name) == 0
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
            crate::g_main::Com_Printf(
                cstr(&format!("^1Duplicate tag name \"{}\"\n", cstr_to_str(name))).as_ptr(),
            );
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
            crate::g_main::Com_Printf(
                cstr(&format!(
                    "^1ERROR: Nameless ref_tag found at ({} {} {})\n",
                    origin[0] as c_int, origin[1] as c_int, origin[2] as c_int
                ))
                .as_ptr(),
            );
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

// PORT-NOTE(bg-dep): depends on `TAG_Find`/`tagOwner_t` (unported).
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

// PORT-NOTE(bg-dep): depends on `TAG_Find`/`tagOwner_t` (unported).
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
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    use crate::q_math::vectoangles;
    unsafe {
        if !(*ent).target.is_null() {
            //TODO: Find the target and set our angles to that direction
            let target = G_Find(
                ctx,
                ctx.entity_id_of(core::ptr::null_mut()),
                core::mem::offset_of!(gentity_t, targetname) as c_int,
                (*ent).target,
            );
            let mut dir: vec3_t = [0.0, 0.0, 0.0];

            if !target.is_null() {
                // Find the direction to the target
                crate::q_math::_VectorSubtract((*target).s.origin, (*ent).s.origin, &mut dir);
                VectorNormalize(&mut dir);
                vectoangles(dir, &mut (*ent).s.angles);
                //FIXME: Does pitch get flipped?
            } else {
                crate::g_main::Com_Printf(
                    cstr(&format!(
                        "^1ERROR: ref_tag ({}) has invalid target ({})",
                        cstr_to_str((*ent).targetname),
                        cstr_to_str((*ent).target)
                    ))
                    .as_ptr(),
                );
            }
        }

        // Add the tag
        TAG_Add(
            ctx,
            (*ent).targetname,
            (*ent).ownername,
            (*ent).s.origin,
            (*ent).s.angles,
            16,
            0,
        );

        // Delete immediately, cannot be refered to as an entity again
        // NOTE: this means if you wanted to link them in a chain for, say, a path, you can't
        G_FreeEntity(ctx, ctx.entity_id_of(ent));
    }
}

// PORT-NOTE(unported-const): `START_TIME_LINK_ENTS` has no ported home;
// referenced verbatim.
/// Raven `SP_reference_tag`.
///
/// Source: `oracle/codemp/game/g_misc.c:3300-3312`
pub fn SP_reference_tag(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    unsafe {
        if !(*ent).target.is_null() {
            // Init cannot occur until all entities have been spawned
            (*ent).think = Some(EntThink::ref_link).into();
            (*ent).nextthink = ctx.world.level.time + START_TIME_LINK_ENTS;
        } else {
            ref_link(ctx, ctx.entity_id_of(ent).unwrap());
        }
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
                *slot = core::mem::zeroed();
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

// PORT-NOTE(raw-ptr-skeleton-no-world-handle): reads `level.time`; also
// a fn-pointer write (`think = misc_weapon_shooter_fire`).
/// Raven `misc_weapon_shooter_fire`.
///
/// Source: `oracle/codemp/game/g_misc.c:3391-3399`
pub fn misc_weapon_shooter_fire(ctx: &mut GameContext, self_: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);

    use crate::g_weapon::FireWeapon;
    unsafe {
        FireWeapon(
            ctx,
            ctx.entity_id_of(self_),
            (((*self_).spawnflags & 1) != 0) as qboolean,
        );
        if (*self_).spawnflags & 2 != 0 {
            (*self_).think = Some(EntThink::misc_weapon_shooter_fire).into();
            (*self_).nextthink = ctx.world.level.time + (*self_).wait as c_int;
        }
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
    // STAGE-1: EntityId self_ + Option<EntityId> other/activator; raw body re-derived verbatim (Stage-2 debt).
    let base = ctx.world.g_entities.as_mut_ptr();
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    let other: *mut gentity_t = unsafe { resolve(base, other) };
    let activator: *mut gentity_t = unsafe { resolve(base, activator) };

    unsafe {
        if (*self_).think.get() == Some(EntThink::misc_weapon_shooter_fire) {
            // repeating fire, stop
            /*
            G_FreeClientForShooter(self->client);
            self->think = G_FreeEntity;
            self->nextthink = level.time;
            */
            (*self_).nextthink = 0;
            return;
        }
        // otherwise, fire
        misc_weapon_shooter_fire(ctx, ctx.entity_id_of(self_).unwrap());
    }
}

/// Raven `misc_weapon_shooter_aim`.
///
/// Source: `oracle/codemp/game/g_misc.c:3417-3438`
pub fn misc_weapon_shooter_aim(ctx: &mut GameContext, self_: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);

    use crate::q_math::vectoangles;
    unsafe {
        // update my aim
        if !(*self_).target.is_null() {
            let targ = G_Find(
                ctx,
                ctx.entity_id_of(core::ptr::null_mut()),
                core::mem::offset_of!(gentity_t, targetname) as c_int,
                (*self_).target,
            );
            if !targ.is_null() {
                (*self_).enemy = Some(ent_id(ctx.world.g_entities.as_mut_ptr(), targ));
                crate::q_math::_VectorSubtract(
                    (*targ).r.currentOrigin,
                    (*self_).r.currentOrigin,
                    &mut (*self_).pos1,
                );
                crate::q_math::_VectorCopy((*targ).r.currentOrigin, &mut (*self_).pos1);
                vectoangles(
                    (*self_).pos1,
                    &mut (*((*self_).client as *mut gclient_t)).ps.viewangles,
                );
                SetClientViewAngle(
                    &mut *self_,
                    (*((*self_).client as *mut gclient_t)).ps.viewangles,
                );
                //FIXME: don't keep doing this unless target is a moving target?
                (*self_).nextthink = ctx.world.level.time + FRAMETIME;
            } else {
                (*self_).enemy = None;
            }
        }
    }
}

/// Raven `SP_misc_weapon_shooter`.
///
/// Source: `oracle/codemp/game/g_misc.c:3444-3486`
pub fn SP_misc_weapon_shooter(ctx: &mut GameContext, self_: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);

    unsafe {
        // alloc a client just for the weapon code to use
        (*self_).client = G_ClientForShooter(ctx) as *mut c_void;

        let mut s: *mut c_char = core::ptr::null_mut();
        G_SpawnString(ctx, c"weapon".as_ptr(), c"".as_ptr(), &mut s);

        // set weapon
        (*self_).s.weapon = mp_bg::weapons::weapon_t::WP_BLASTER;
        (*((*self_).client as *mut gclient_t)).ps.weapon = mp_bg::weapons::weapon_t::WP_BLASTER;
        if !s.is_null() && *s != 0 {
            // use a different weapon
            let w = crate::q_shared::GetIDForString(WPTable.as_ptr() as *mut _, s);
            (*self_).s.weapon = w;
            (*((*self_).client as *mut gclient_t)).ps.weapon = w;
        }

        crate::g_items::RegisterItem(ctx, crate::bg_misc::BG_FindItemForWeapon((*self_).s.weapon));

        // set where our muzzle is
        crate::q_math::_VectorCopy(
            (*self_).s.origin,
            &mut (*((*self_).client as *mut gclient_t))
                .renderInfo
                .muzzlePoint,
        );
        // permanently updated (don't need for MP)
        //self->client->renderInfo.mPCalcTime = Q3_INFINITE;

        // set up to link
        if !(*self_).target.is_null() {
            (*self_).think = Some(EntThink::misc_weapon_shooter_aim).into();
            (*self_).nextthink = ctx.world.level.time + START_TIME_LINK_ENTS;
        } else {
            // just set aim angles
            crate::q_math::_VectorCopy(
                (*self_).s.angles,
                &mut (*((*self_).client as *mut gclient_t)).ps.viewangles,
            );
            AngleVectors((*self_).s.angles, Some(&mut (*self_).pos1), None, None);
        }

        // set up to fire when used
        (*self_).use_ = Some(EntUse::misc_weapon_shooter_use).into();

        if (*self_).wait == 0.0 {
            (*self_).wait = 500.0;
        }
    }
}

/// Raven `SP_misc_weather_zone`.
///
/// Source: `oracle/codemp/game/g_misc.c:3491-3494`
pub fn SP_misc_weather_zone(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    G_FreeEntity(ctx, ctx.entity_id_of(ent));
}

// The local `G_SpawnInt` shim formerly here (adapting byte-string literals to
// `*const c_char`) is dropped: its only callers (`EnergyShieldStationSettings`/
// `EnergyAmmoStationSettings`/`EnergyHealthStationSettings`) are parked
// (`seam-threading` — `G_SpawnInt` needs a `GameContext` this shim had no way
// to supply), so it has zero live callers (porting-rules §20).
