// PORT-COMPLETE: NPC_utils.c
//! Port of `oracle/codemp/game/NPC_utils.c` (jampgame mega-pass).
//!
//! Generated from `tools/closure-prototype/fnskel.py`; bodies filled per the
//! jampgame mega-pass (settled fork rulings,
//! `docs/handoffs/jampgame-fork-discovery.md`), then the pass-2 sweep
//! (`packets/NPC_utils.md`) that resolved the `ai-context` park class below.
//!
//! SPINE (`docs/architecture/engine-seam.md`, precedent
//! `w_force.rs`/`g_client.rs`): logic fns that reach `level`/cvars/`g_entities`/
//! traps thread the `GameContext<'_>` receiver (`.world: *mut GameWorld`,
//! `.engine`) as an ADDITIVE first parameter (the faithful C signature carries
//! none). Globals are `GameWorld` fields: `level` →
//! `ctx.world.level`, `g_entities[i]` → `ctx.world.g_entities[i]`; this
//! file's own `teamNumbers`/`teamStrength`/`teamCounter` file-scope globals
//! were added to `GameWorld` (additive, Raven names kept — see
//! `world/game_world.rs`). Traps go through `trap::X(ctx.engine, …)`.
//! Cross-file callees are invoked with the packet's resolved raw-pointer
//! signatures verbatim (their own porters thread the spine). Raw
//! `gentity_t*`/`gclient_t*` chains are transcribed as `unsafe` raw-pointer
//! field access mirroring the C exactly.
//!
//! PASS-2 (ai-context resolved): `NPC.c`'s ambient "current actor" globals
//! (`NPC`, `NPCInfo`, `client`, `ucmd`) are now real `ctx.world.globals`
//! fields (`GameGlobals`, backfilled from their `()` placeholders) — the
//! bulk of this file's fns that were parked `ai-context` in the mega pass are
//! ported reaching them through `ctx`. `CalcEntitySpot`'s `point` and
//! `G_GetBoltPosition`'s `pos` got the vec3 out-param reshape
//! (`&mut vec3_t` / `Option<&mut vec3_t>`) and same-file callers were fixed up.
//!
//! The gaps the mega pass parked are all closed and these functions are ported
//! live: `NPC_SetSurfaceOnOff` (`bgToggleableSurfaces`), `G_ActivateBehavior`
//! (`BSTable`) and `G_GetBoltPosition` (`BG_GiveMeVectorFromMatrix`).
//!
//! Safe-state migration **Stage 1**: entity-pointer params are `EntityId` /
//! `Option<EntityId>` handles (§B5), not raw `gentity_t*`; ctx-free leaf helpers
//! take `&mut`/`&gentity_t`. Callers bridge at the boundary via
//! `ctx.entity_id_of(ptr)`.
//!
//! Safe-state migration **campaign 2c** (deref regime): the Stage-1 fn-top raw
//! re-derives are gone — entity fields are read/written through
//! `ctx.world.entity(id)`/`entity_mut(id)` at the point of use. The only raw
//! derefs left carry a one-line `FLAG` and are the sanctioned categories:
//! `gNPC_t` (`NPCInfo`, no accessor) and the BG_Alloc'd pool clients an NPC/
//! vehicle entity carries (`ent.client`, `level.clients` is not valid for them —
//! trap 2b); each is a tight unsafe deref through a pointer value copied out of
//! the safe entity borrow.
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_target::Q3_SCRIPT_DIR;
use crate::g_utils::{G_BoneIndex, GetAnglesForDirection};
use crate::level::alert_event::{alertEventLevel_e::AEL_DISCOVERED, alertEvent_t};
use crate::prelude::*;
use crate::q_math::{
    _VectorCopy, vec3_origin, AngleDelta, AngleNormalize360, AngleVectors, Distance, PITCH, ROLL,
    YAW,
};
use crate::q_shared::{va, GetIDForString, GetStringForID, Q_stricmp};
use crate::teams::npcteam::{NPCTEAM_ENEMY, NPCTEAM_FREE, NPCTEAM_NEUTRAL, NPCTEAM_PLAYER};
use crate::trap;
use crate::world::GameContext;
use crate::NPC_combat::{G_ClearEnemy, G_SetEnemy, NPC_ClearShot};
pub use crate::NPC_goal::UpdateGoal;
use crate::NPC_senses::{
    G_ClearLOS, G_ClearLOS2, G_ClearLOS3, G_ClearLOS4, G_ClearLOS5, InFOV, NPC_CheckAlertEvents,
};
use crate::NPC_sounds::G_AddVoiceEvent;
use mp_qshared::shared::force_powers::FP_SPEED;
use native_string::atof::atof;

use mp_abi::game::syscalls::G_ENTITIES_IN_BOX::GEntitiesInBoxArgs;
use mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs;
use mp_abi::game::syscalls::G_ICARUS_TASKIDCOMPLETE::GIcarusTaskidcompleteArgs;
use mp_abi::game::syscalls::G_ICARUS_TASKIDPENDING::GIcarusTaskidpendingArgs;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;

// Raven `#define VALID_ATTACK_CONE 2.0f` (this file's own macro).
// Source: `oracle/codemp/game/NPC_utils.c:11`
pub const VALID_ATTACK_CONE: f32 = 2.0;

// Raven `#define MIN_ANGLE_ERROR 0.01f` (`b_local.h`).
// Source: `oracle/codemp/game/b_local.h:29`
const MIN_ANGLE_ERROR: f32 = 0.01;

// Raven `#define Q3_INFINITE 16777216` (`g_public.h`).
// Source: `oracle/codemp/game/g_public.h:9`
const Q3_INFINITE: f32 = 16777216.0;

// Raven `#define WORLD_SIZE ( MAX_WORLD_COORD - MIN_WORLD_COORD )` (65536 -
// (-65536) = 131072). Per-file local const, same idiom as `NPC_combat.rs`.
// Source: `oracle/codemp/game/q_shared.h`
const WORLD_SIZE: f32 = 131072.0;

// Raven `#define MAX_RADIUS_ENTS 256` (per-file local const, scopes
// `NPC_FindNearestEnemy`; distinct from the 128 value in `NPC_AI_Utils.rs`).
// Source: `oracle/codemp/game/NPC_utils.c:1243`
const MAX_RADIUS_ENTS: usize = 256;

// `DistanceSquared` is the canonical `crate::q_math::DistanceSquared`, reached
// via the prelude glob (no per-file copy).

// Raven `BONE_ANGLES_POSTMULT` (ghoul2 bone-angle apply mode) resolves via the
// canonical `mp_qshared::common::mp::ghoul2::bone_flags` module (crate prelude
// glob).

/// Raven `TURN_ON` flag for surface toggling.
/// Source: `oracle/codemp/game/NPC_utils.c:1022`
const TURN_ON: c_int = 0x00000000;

/// Raven `ORIGIN` — extract the origin vector from a bolt matrix.
/// Source: `oracle/codemp/game/ghoul2_shared.h` (Eorientations enum)
const ORIGIN: c_int = Eorientations::ORIGIN as c_int;

/// Raven `q_shared.h:30` `VALIDSTRING(a)` macro.
/// Source: `oracle/codemp/game/q_shared.h:30`
unsafe fn VALIDSTRING(a: *const c_char) -> bool {
    !a.is_null() && *a as c_int != 0
}

/// Raven `BG_NUM_TOGGLEABLE_SURFACES`.
/// Source: `oracle/codemp/game/bg_public.h:138`
pub const BG_NUM_TOGGLEABLE_SURFACES: c_int = 31;

use crate::npc::script_flags::SCF_DONT_FIRE;
use mp_bg::public::team::{TEAM_BLUE, TEAM_FREE, TEAM_RED, TEAM_SPECTATOR};
use mp_qshared::common::mp::qcommon::pm_flags::PMF_FOLLOW;
use mp_qshared::shared::{MASK_PLAYERSOLID, MAX_CLIENTS};

use mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs;

/// Raven `CalcEntitySpot`.
///
/// Out-param reshape: `point` is written unconditionally on every branch (no
/// oracle caller passes NULL), so it becomes the non-nullable `&mut vec3_t`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:20-168`
pub fn CalcEntitySpot(
    ctx: &mut GameContext,
    ent: Option<EntityId>,
    spot: spot_t,
    point: &mut vec3_t,
) {
    let ent_id = match ent {
        Some(i) => i,
        None => return,
    };

    match spot {
        spot_t::SPOT_ORIGIN => {
            if ctx.world.entity(ent_id).r.currentOrigin == vec3_origin {
                //brush
                let e = ctx.world.entity(ent_id);
                let size = [
                    e.r.absmax[0] - e.r.absmin[0],
                    e.r.absmax[1] - e.r.absmin[1],
                    e.r.absmax[2] - e.r.absmin[2],
                ];
                let absmin = e.r.absmin;
                for i in 0..3 {
                    point[i] = absmin[i] + 0.5 * size[i];
                }
            } else {
                *point = ctx.world.entity(ent_id).r.currentOrigin;
            }
        }
        spot_t::SPOT_CHEST | spot_t::SPOT_HEAD => {
            // FLAG: entity may be an NPC carrying a BG_Alloc'd pool client; deref
            // the client pointer raw via the safe entity borrow (trap 2b).
            let client = ctx.world.entity(ent_id).client;
            //Actual tag_head eyespot!
            //FIXME: Stasis aliens may have a problem here...
            if !client.is_null()
                && unsafe { VectorLengthSquared((*client).renderInfo.eyePoint) } != 0.0
            {
                *point = unsafe { (*client).renderInfo.eyePoint };
                if unsafe { (*client).NPC_class } == CLASS_ATST {
                    //adjust up some
                    point[2] += 28.0; //magic number :)
                }
                if !ctx.world.entity(ent_id).NPC.is_null() {
                    //always aim from the center of my bbox, so we don't wiggle when we lean forward or backwards
                    let origin = ctx.world.entity(ent_id).r.currentOrigin;
                    point[0] = origin[0];
                    point[1] = origin[1];
                }
            } else {
                *point = ctx.world.entity(ent_id).r.currentOrigin;
                if !client.is_null() {
                    point[2] += unsafe { (*client).ps.viewheight } as f32;
                }
            }
            if matches!(spot, spot_t::SPOT_CHEST) && !client.is_null() {
                if unsafe { (*client).NPC_class } != CLASS_ATST {
                    //adjust up some
                    point[2] -= ctx.world.entity(ent_id).r.maxs[2] * 0.2;
                }
            }
        }
        spot_t::SPOT_HEAD_LEAN => {
            // FLAG: entity may be an NPC carrying a BG_Alloc'd pool client; deref
            // the client pointer raw via the safe entity borrow (trap 2b).
            let client = ctx.world.entity(ent_id).client;
            if !client.is_null()
                && unsafe { VectorLengthSquared((*client).renderInfo.eyePoint) } != 0.0
            {
                //Actual tag_head eyespot!
                *point = unsafe { (*client).renderInfo.eyePoint };
                if unsafe { (*client).NPC_class } == CLASS_ATST {
                    point[2] += 28.0;
                }
                if !ctx.world.entity(ent_id).NPC.is_null() {
                    let origin = ctx.world.entity(ent_id).r.currentOrigin;
                    point[0] = origin[0];
                    point[1] = origin[1];
                }
                //NOTE: automatically takes leaning into account!
            } else {
                *point = ctx.world.entity(ent_id).r.currentOrigin;
                if !client.is_null() {
                    point[2] += unsafe { (*client).ps.viewheight } as f32;
                }
            }
        }
        spot_t::SPOT_LEGS => {
            *point = ctx.world.entity(ent_id).r.currentOrigin;
            point[2] += ctx.world.entity(ent_id).r.mins[2] * 0.5;
        }
        spot_t::SPOT_WEAPON => {
            let mut forward: vec3_t = [0.0; 3];
            let mut right: vec3_t = [0.0; 3];
            let mut up: vec3_t = [0.0; 3];
            // FLAG: gNPC_t + pool client derefs stay raw (recipe 2c / trap 2b).
            let npc = ctx.world.entity(ent_id).NPC;
            let client = ctx.world.entity(ent_id).client;
            let use_shoot_angles = unsafe {
                !npc.is_null()
                    && (*npc).shootAngles != vec3_origin
                    && (*npc).shootAngles != (*client).ps.viewangles
            };
            if use_shoot_angles {
                AngleVectors(
                    unsafe { (*npc).shootAngles },
                    Some(&mut forward),
                    Some(&mut right),
                    Some(&mut up),
                );
            } else {
                AngleVectors(
                    unsafe { (*client).ps.viewangles },
                    Some(&mut forward),
                    Some(&mut right),
                    Some(&mut up),
                );
            }
            crate::g_weapon::CalcMuzzlePoint(ctx, ent_id, forward, right, up, point);
            //NOTE: automatically takes leaning into account!
        }
        spot_t::SPOT_GROUND => {
            // if entity is on the ground, just use it's absmin
            if ctx.world.entity(ent_id).s.groundEntityNum != -1 {
                *point = ctx.world.entity(ent_id).r.currentOrigin;
                point[2] = ctx.world.entity(ent_id).r.absmin[2];
                return;
            }

            // if it is reasonably close to the ground, give the point underneath of it
            let mut start = ctx.world.entity(ent_id).r.currentOrigin;
            start[2] = ctx.world.entity(ent_id).r.absmin[2];
            let mut end = start;
            end[2] -= 64.0;
            let mut tr: trace_t = unsafe { core::mem::zeroed() };
            let mins = ctx.world.entity(ent_id).r.mins;
            let maxs = ctx.world.entity(ent_id).r.maxs;
            let number = ctx.world.entity(ent_id).s.number;
            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut tr,
                    &start,
                    &mins,
                    &maxs,
                    &end,
                    number,
                    MASK_PLAYERSOLID,
                ),
            );
            if tr.fraction < 1.0 {
                *point = tr.endpos;
                return;
            }

            // otherwise just use the origin
            *point = ctx.world.entity(ent_id).r.currentOrigin;
        }
    }
}

/// Raven `NPC_UpdateAngles`.
///
/// Raven: the `#if 1` branch is the compiled one (the `#else` branch below it
/// is dead source, per house ruling on `#if 0`/`#if 1` branches) — only that
/// branch is transcribed.
///
/// Source: `oracle/codemp/game/NPC_utils.c:182-517`
pub fn NPC_UpdateAngles(ctx: &mut GameContext, doPitch: qboolean, doYaw: qboolean) -> qboolean {
    unsafe {
        let npc = ctx.world.globals.NPC;
        let npc_id = ctx.entity_id_of(npc).unwrap();
        // FLAG: gNPC_t (NPCInfo) + pool client (NPC) derefs stay raw (recipe 2c / trap 2b).
        let npc_info = ctx.world.globals.NPCInfo;
        let client = ctx.world.globals.client;

        let mut target_pitch: f32 = 0.0;
        let mut target_yaw: f32 = 0.0;
        let mut exact = qtrue;

        // if angle changes are locked; just keep the current angles
        // aimTime isn't even set anymore... so this code was never reached, but I need a way to lock NPC's yaw, so instead of making a new SCF_ flag, just use the existing render flag... - dmv
        if ctx.world.entity(npc_id).enemy.is_none() && ctx.world.level.time < (*npc_info).aimTime {
            if doPitch != qfalse {
                target_pitch = (*npc_info).lockedDesiredPitch;
            }
            if doYaw != qfalse {
                target_yaw = (*npc_info).lockedDesiredYaw;
            }
        } else {
            // we're changing the lockedDesired Pitch/Yaw below so it's lost it's original meaning, get rid of the lock flag
            if doPitch != qfalse {
                target_pitch = (*npc_info).desiredPitch;
                (*npc_info).lockedDesiredPitch = (*npc_info).desiredPitch;
            }
            if doYaw != qfalse {
                target_yaw = (*npc_info).desiredYaw;
                (*npc_info).lockedDesiredYaw = (*npc_info).desiredYaw;
            }
        }

        let mut yaw_speed: f32;
        if ctx.world.entity(npc_id).s.weapon == WP_EMPLACED_GUN {
            // FIXME: this seems to do nothing, actually...
            yaw_speed = 20.0;
        } else {
            yaw_speed = (*npc_info).stats.yawSpeed;
        }

        let npc_client = ctx.world.entity(npc_id).client;
        if ctx.world.entity(npc_id).s.weapon == WP_SABER
            && ((*npc_client).ps.fd.forcePowersActive & (1 << (FP_SPEED as c_int))) != 0
        {
            let buf = trap::Cvar_VariableStringBuffer(ctx.engine, "timescale", 128);
            let t_f_val = atof(&buf);
            yaw_speed *= 1.0 / (t_f_val as f32);
        }

        if doYaw != qfalse {
            // decay yaw error
            let mut error = AngleDelta((*npc_client).ps.viewangles[YAW], target_yaw);
            if error.abs() > MIN_ANGLE_ERROR {
                if error != 0.0 {
                    exact = qfalse;

                    let mut decay = 60.0 + yaw_speed * 3.0;
                    decay *= 50.0 / 1000.0; //msec

                    if error < 0.0 {
                        error += decay;
                        if error > 0.0 {
                            error = 0.0;
                        }
                    } else {
                        error -= decay;
                        if error < 0.0 {
                            error = 0.0;
                        }
                    }
                }
            }

            ctx.world.globals.ucmd.angles[YAW] =
                ANGLE2SHORT(target_yaw + error) - (*client).ps.delta_angles[YAW];
        }

        //FIXME: have a pitchSpeed?
        if doPitch != qfalse {
            // decay pitch error
            let mut error = AngleDelta((*npc_client).ps.viewangles[PITCH], target_pitch);
            if error.abs() > MIN_ANGLE_ERROR {
                if error != 0.0 {
                    exact = qfalse;

                    let mut decay = 60.0 + yaw_speed * 3.0;
                    decay *= 50.0 / 1000.0; //msec

                    if error < 0.0 {
                        error += decay;
                        if error > 0.0 {
                            error = 0.0;
                        }
                    } else {
                        error -= decay;
                        if error < 0.0 {
                            error = 0.0;
                        }
                    }
                }
            }

            ctx.world.globals.ucmd.angles[PITCH] =
                ANGLE2SHORT(target_pitch + error) - (*client).ps.delta_angles[PITCH];
        }

        ctx.world.globals.ucmd.angles[ROLL] =
            ANGLE2SHORT((*npc_client).ps.viewangles[ROLL]) - (*client).ps.delta_angles[ROLL];

        if exact != qfalse
            && trap::ICARUS_TaskIDPending(
                ctx.engine,
                GIcarusTaskidpendingArgs::new(npc.cast(), taskID_t::TID_ANGLE_FACE as c_int),
            ) != 0
        {
            trap::ICARUS_TaskIDComplete(
                ctx.engine,
                GIcarusTaskidcompleteArgs::new(npc.cast(), taskID_t::TID_ANGLE_FACE as c_int),
            );
        }
        exact
    }
}

/// Raven `NPC_AimWiggle`.
///
/// Out-param reshape: `enemy_org` is mutated in place (`VectorAdd(enemy_org,
/// NPCInfo->aimOfs, enemy_org)`), so it becomes `&mut vec3_t`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:519-533`
pub fn NPC_AimWiggle(ctx: &mut GameContext, enemy_org: &mut vec3_t) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: gNPC_t (NPCInfo) derefs stay raw (recipe 2c).
    let npc_info = ctx.world.globals.NPCInfo;

    //shoot for somewhere between the head and torso
    //NOTE: yes, I know this looks weird, but it works
    if unsafe { (*npc_info).aimErrorDebounceTime } < ctx.world.level.time {
        // Raven derefs `NPC->enemy` unconditionally here (assumed non-null
        // by the caller).
        let enemy_id = ctx.world.entity(npc_id).enemy.unwrap();
        let mins = ctx.world.entity(enemy_id).r.mins;
        let maxs = ctx.world.entity(enemy_id).r.maxs;
        // C's `0.3` is a double literal: `0.3*flrand(...)` evaluates in f64,
        // narrowing to the float `aimOfs` only at the assignment.
        unsafe {
            (*npc_info).aimOfs[0] =
                (0.3 * ctx.world.bg_state.rng.flrand(mins[0], maxs[0]) as f64) as f32;
            (*npc_info).aimOfs[1] =
                (0.3 * ctx.world.bg_state.rng.flrand(mins[1], maxs[1]) as f64) as f32;
            if maxs[2] > 0.0 {
                (*npc_info).aimOfs[2] = maxs[2] * ctx.world.bg_state.rng.flrand(0.0, -1.0);
            }
        }
    }
    for i in 0..3 {
        enemy_org[i] += unsafe { (*npc_info).aimOfs[i] };
    }
}

/// Raven `NPC_UpdateFiringAngles`.
///
/// Raven: the `#else` branch is the compiled one (`#if 0` above it is dead
/// source, per house ruling on `#if 0` branches) — only that branch is
/// transcribed.
///
/// Source: `oracle/codemp/game/NPC_utils.c:540-731`
pub fn NPC_UpdateFiringAngles(
    ctx: &mut GameContext,
    doPitch: qboolean,
    doYaw: qboolean,
) -> qboolean {
    unsafe {
        let npc = ctx.world.globals.NPC;
        let npc_id = ctx.entity_id_of(npc).unwrap();
        // FLAG: gNPC_t (NPCInfo) + pool client (NPC) derefs stay raw (recipe 2c / trap 2b).
        let npc_info = ctx.world.globals.NPCInfo;
        let client = ctx.world.globals.client;

        let mut target_pitch: f32 = 0.0;
        let mut target_yaw: f32 = 0.0;
        let mut exact = qtrue;

        // if angle changes are locked; just keep the current angles
        if ctx.world.level.time < (*npc_info).aimTime {
            if doPitch != qfalse {
                target_pitch = (*npc_info).lockedDesiredPitch;
            }
            if doYaw != qfalse {
                target_yaw = (*npc_info).lockedDesiredYaw;
            }
        } else {
            if doPitch != qfalse {
                target_pitch = (*npc_info).desiredPitch;
            }
            if doYaw != qfalse {
                target_yaw = (*npc_info).desiredYaw;
            }

            if doPitch != qfalse {
                (*npc_info).lockedDesiredPitch = (*npc_info).desiredPitch;
            }
            if doYaw != qfalse {
                (*npc_info).lockedDesiredYaw = (*npc_info).desiredYaw;
            }
        }

        if (*npc_info).aimErrorDebounceTime < ctx.world.level.time {
            if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
                (*npc_info).lastAimErrorYaw =
                    ((6 - (*npc_info).stats.aim) as f32) * ctx.world.bg_state.rng.flrand(-1.0, 1.0);
            }
            if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
                (*npc_info).lastAimErrorPitch =
                    ((6 - (*npc_info).stats.aim) as f32) * ctx.world.bg_state.rng.flrand(-1.0, 1.0);
            }
            (*npc_info).aimErrorDebounceTime =
                ctx.world.level.time + ctx.world.bg_state.rng.Q_irand(250, 2000);
        }

        let npc_client = ctx.world.entity(npc_id).client;

        if doYaw != qfalse {
            // decay yaw diff
            let mut diff = AngleDelta((*npc_client).ps.viewangles[YAW], target_yaw);

            if diff != 0.0 {
                exact = qfalse;

                let mut decay = 60.0 + 80.0;
                decay *= 50.0 / 1000.0; //msec
                if diff < 0.0 {
                    diff += decay;
                    if diff > 0.0 {
                        diff = 0.0;
                    }
                } else {
                    diff -= decay;
                    if diff < 0.0 {
                        diff = 0.0;
                    }
                }
            }

            // add yaw error based on NPCInfo->aim value
            let error = (*npc_info).lastAimErrorYaw;

            ctx.world.globals.ucmd.angles[YAW] =
                ANGLE2SHORT(target_yaw + diff + error) - (*client).ps.delta_angles[YAW];
        }

        if doPitch != qfalse {
            // decay pitch diff
            let mut diff = AngleDelta((*npc_client).ps.viewangles[PITCH], target_pitch);
            if diff != 0.0 {
                exact = qfalse;

                let mut decay = 60.0 + 80.0;
                decay *= 50.0 / 1000.0; //msec
                if diff < 0.0 {
                    diff += decay;
                    if diff > 0.0 {
                        diff = 0.0;
                    }
                } else {
                    diff -= decay;
                    if diff < 0.0 {
                        diff = 0.0;
                    }
                }
            }

            let error = (*npc_info).lastAimErrorPitch;

            ctx.world.globals.ucmd.angles[PITCH] =
                ANGLE2SHORT(target_pitch + diff + error) - (*client).ps.delta_angles[PITCH];
        }

        ctx.world.globals.ucmd.angles[ROLL] =
            ANGLE2SHORT((*npc_client).ps.viewangles[ROLL]) - (*client).ps.delta_angles[ROLL];

        exact
    }
}

/// Raven `NPC_UpdateShootAngles`.
///
/// Raven: FIXME: shoot angles either not set right or not used! `angles` is
/// read-only here (never written), so the out-param reshape does not
/// apply — kept by-value.
///
/// Source: `oracle/codemp/game/NPC_utils.c:740-808`
pub fn NPC_UpdateShootAngles(
    ctx: &mut GameContext,
    angles: vec3_t,
    doPitch: qboolean,
    doYaw: qboolean,
) {
    unsafe {
        // FLAG: gNPC_t (NPCInfo) derefs stay raw (recipe 2c).
        let npc_info = ctx.world.globals.NPCInfo;

        let mut target_pitch: f32 = 0.0;
        let mut target_yaw: f32 = 0.0;

        if doPitch != qfalse {
            target_pitch = angles[PITCH];
        }
        if doYaw != qfalse {
            target_yaw = angles[YAW];
        }

        if doYaw != qfalse {
            // decay yaw error
            let mut error = AngleDelta((*npc_info).shootAngles[YAW], target_yaw);
            if error != 0.0 {
                let mut decay = 60.0 + 80.0 * ((*npc_info).stats.aim as f32);
                decay *= 100.0 / 1000.0; //msec
                if error < 0.0 {
                    error += decay;
                    if error > 0.0 {
                        error = 0.0;
                    }
                } else {
                    error -= decay;
                    if error < 0.0 {
                        error = 0.0;
                    }
                }
            }
            (*npc_info).shootAngles[YAW] = target_yaw + error;
        }

        if doPitch != qfalse {
            // decay pitch error
            let mut error = AngleDelta((*npc_info).shootAngles[PITCH], target_pitch);
            if error != 0.0 {
                let mut decay = 60.0 + 80.0 * ((*npc_info).stats.aim as f32);
                decay *= 100.0 / 1000.0; //msec
                if error < 0.0 {
                    error += decay;
                    if error > 0.0 {
                        error = 0.0;
                    }
                } else {
                    error -= decay;
                    if error < 0.0 {
                        error = 0.0;
                    }
                }
            }
            (*npc_info).shootAngles[PITCH] = target_pitch + error;
        }
    }
}

/// Raven `SetTeamNumbers`.
///
/// Raven: Sets the number of living clients on each team. FIXME: Does not
/// account for non-respawned players! FIXME: Don't include medics?
///
/// Raven's outer loop is `for ( i = 0; i < 1 ; i++ )` — a known-dead loop
/// bound (only ever checks `g_entities[0]`), preserved faithfully (§C10/§19).
/// The average-health division is UB in Raven when `teamNumbers[i] == 0`
/// (int/int, then implicit float->int on the `floor()` result); Rust's
/// `f32` division yields `NaN`/`inf`, and `as c_int` on those saturates to 0
/// — the one defined behavior, picked per §19.
///
/// Source: `oracle/codemp/game/NPC_utils.c:818-847`
pub fn SetTeamNumbers(ctx: &mut GameContext) {
    for i in 0..4usize {
        ctx.world.teamNumbers[i] = 0;
        ctx.world.teamStrength[i] = 0;
    }

    for i in 0..1usize {
        let found_id = EntityId(i as u32);
        // FLAG: entity's client pointer dereffed raw via the safe borrow (trap 2b).
        let client = ctx.world.entity(found_id).client;
        if !client.is_null() {
            if ctx.world.entity(found_id).health > 0 {
                let team = unsafe { (*client).playerTeam } as usize;
                let health = ctx.world.entity(found_id).health;
                ctx.world.teamNumbers[team] += 1;
                ctx.world.teamStrength[team] += health;
            }
        }
    }

    for i in 0..4usize {
        // Raven: `floor( ((float)(teamStrength[i])) / ((float)(teamNumbers[i])) )`.
        let strength = ctx.world.teamStrength[i] as f32;
        let count = ctx.world.teamNumbers[i] as f32;
        ctx.world.teamStrength[i] = (strength / count).floor() as c_int;
    }
}

/// Raven `G_ActivateBehavior`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:851-894`
pub fn G_ActivateBehavior(ctx: &mut GameContext, self_: Option<EntityId>, bset: c_int) -> qboolean {
    let self_id = match self_ {
        Some(i) => i,
        None => return qfalse,
    };

    let bs_name = ctx.world.entity(self_id).behaviorSet[bset as usize];

    if !unsafe { VALIDSTRING(bs_name) } {
        return qfalse;
    }

    let mut bSID: c_int = -1;
    // FLAG: gNPC_t (NPC) deref stays raw (recipe 2c).
    let npc = ctx.world.entity(self_id).NPC;
    if !npc.is_null() {
        bSID = GetIDForString(BSTable.as_ptr() as *mut stringID_table_t, bs_name);
    }

    if bSID > -1 {
        unsafe {
            (*npc).tempBehavior = bState_t::BS_DEFAULT;
            (*npc).behaviorState = core::mem::transmute::<c_int, bState_t>(bSID);
        }
    } else {
        // if (0) branch is dead code in oracle
        let script_path = unsafe {
            format!(
                "{}/{}",
                cstr_to_str(Q3_SCRIPT_DIR.as_ptr()),
                cstr_to_str(bs_name)
            )
        };
        let self_ptr = ctx.world.entity_mut(self_id) as *mut gentity_t;
        trap::ICARUS_RunScript(ctx.engine, self_ptr.cast(), &script_path);
    }
    qtrue
}

/// Raven `NPC_SetBoneAngles`.
///
/// Raven: rww - special system for sync'ing bone angles between client and
/// server. The `#ifdef _XBOX` byte-index branch is dead on this build; the
/// plain `int *` branch below is the compiled one (per house ruling on
/// `_XBOX` branches).
///
/// Source: `oracle/codemp/game/NPC_utils.c:906-995`
pub fn NPC_SetBoneAngles(ctx: &mut GameContext, ent: EntityId, bone: *mut c_char, angles: vec3_t) {
    let boneIndex = G_BoneIndex(ctx, bone as *const c_char);

    // Walk the 4 fixed bone-index/bone-angle slot pairs looking for `boneIndex`
    // (or the first free slot if not already present). Raven walks raw pointers
    // into `ent->s.boneIndexN`/`boneAnglesN`; the slot index does the same job
    // through the entity accessor.
    let mut first_free: Option<usize> = None;
    let mut found_slot: Option<usize> = None;
    for slot in 0..4usize {
        let idx = {
            let e = ctx.world.entity(ent);
            match slot {
                0 => e.s.boneIndex1,
                1 => e.s.boneIndex2,
                2 => e.s.boneIndex3,
                _ => e.s.boneIndex4,
            }
        };
        if idx == 0 && first_free.is_none() {
            first_free = Some(slot);
        } else if idx != 0 && idx == boneIndex {
            found_slot = Some(slot);
            break;
        }
    }

    let target_slot = match found_slot {
        Some(s) => s,
        None => {
            // didn't find it, create it
            match first_free {
                None => {
                    crate::g_main::Com_Printf("WARNING: NPC has no free bone indexes\n");
                    return;
                }
                Some(s) => {
                    let e = ctx.world.entity_mut(ent);
                    match s {
                        0 => e.s.boneIndex1 = boneIndex,
                        1 => e.s.boneIndex2 = boneIndex,
                        2 => e.s.boneIndex3 = boneIndex,
                        _ => e.s.boneIndex4 = boneIndex,
                    }
                    s
                }
            }
        }
    };

    // Copy the angles over the vector in the entitystate, so we can use
    // the corresponding index to set the bone angles on the client.
    {
        let e = ctx.world.entity_mut(ent);
        match target_slot {
            0 => e.s.boneAngles1 = angles,
            1 => e.s.boneAngles2 = angles,
            2 => e.s.boneAngles3 = angles,
            _ => e.s.boneAngles4 = angles,
        }
    }

    // Now set the angles on our server instance if we have one.
    if ctx.world.entity(ent).ghoul2.is_null() {
        return;
    }

    let flags = BONE_ANGLES_POSTMULT;
    let up = POSITIVE_X as c_int;
    let right = NEGATIVE_Y as c_int;
    let forward = NEGATIVE_Z as c_int;

    //first 3 bits is forward, second 3 bits is right, third 3 bits is up
    ctx.world.entity_mut(ent).s.boneOrient = forward | (right << 3) | (up << 6);

    let ghoul2 = ctx.world.entity(ent).ghoul2;
    let level_time = ctx.world.level.time;
    trap::G2API_SetBoneAngles(
        ctx.engine,
        ghoul2,
        0,
        unsafe { &cstr_to_str(bone as *const c_char) },
        &angles as *const vec3_t,
        flags,
        up,
        right,
        forward,
        core::ptr::null_mut(),
        100,
        level_time,
    );
}

/// Raven `NPC_SetSurfaceOnOff`.
///
/// Raven: rww - and another method of automatically managing surface status
/// for the client and server at once.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1001-1039`
pub fn NPC_SetSurfaceOnOff(
    ctx: &mut GameContext,
    ent: EntityId,
    surfaceName: *const c_char,
    surfaceFlags: c_int,
) {
    let mut i: c_int = 0;
    let mut foundIt = qfalse;

    while i < BG_NUM_TOGGLEABLE_SURFACES {
        if let Some(surf_name) = bgToggleableSurfaces[i as usize] {
            if Q_stricmp(surfaceName, surf_name.as_ptr()) == 0 {
                foundIt = qtrue;
                break;
            }
        } else {
            break;
        }
        i += 1;
    }

    if foundIt == qfalse {
        let msg = format!(
            "WARNING: Tried to toggle NPC surface that isn't in toggleable surface list ({})\n",
            unsafe { cstr_to_str(surfaceName) }
        );
        crate::g_main::Com_Printf(&msg);
        return;
    }

    if surfaceFlags == TURN_ON {
        ctx.world.entity_mut(ent).s.surfacesOn |= 1 << i;
        ctx.world.entity_mut(ent).s.surfacesOff &= !(1 << i);
    } else {
        ctx.world.entity_mut(ent).s.surfacesOn &= !(1 << i);
        ctx.world.entity_mut(ent).s.surfacesOff |= 1 << i;
    }

    if ctx.world.entity(ent).ghoul2.is_null() {
        return;
    }

    let ghoul2 = ctx.world.entity(ent).ghoul2;
    trap::G2API_SetSurfaceOnOff(ctx.engine, ghoul2, unsafe { &cstr_to_str(surfaceName) }, surfaceFlags);
}

/// Raven `NPC_SomeoneLookingAtMe`.
///
/// Raven: rww - cheap check to see if an armed client is looking in our
/// general direction.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1042-1067`
pub fn NPC_SomeoneLookingAtMe(ctx: &mut GameContext, ent: EntityId) -> qboolean {
    let mut i: usize = 0;
    while i < MAX_CLIENTS {
        let pEnt_id = EntityId(i as u32);
        // FLAG: pEnt's client pointer dereffed raw via the safe borrow (trap 2b).
        let cl = ctx.world.entity(pEnt_id).client;

        let eligible = ctx.world.entity(pEnt_id).inuse != qfalse
            && !cl.is_null()
            && unsafe {
                (*cl).sess.sessionTeam != TEAM_SPECTATOR && ((*cl).ps.pm_flags & PMF_FOLLOW) == 0
            }
            && ctx.world.entity(pEnt_id).s.weapon != WP_NONE;

        if eligible {
            let ent_origin = ctx.world.entity(ent).r.currentOrigin;
            let pent_origin = ctx.world.entity(pEnt_id).r.currentOrigin;
            if trap::InPVS(
                ctx.engine,
                GInPvsArgs::new(
                    &ent_origin as *const vec3_t,
                    &pent_origin as *const vec3_t,
                ),
            ) != 0
                //I'm in a 30 fov or so cone from this player.. that's enough I guess.
                && InFOV(ctx, Some(ent), pEnt_id, 30, 30) != 0
            {
                return qtrue;
            }
        }

        i += 1;
    }

    qfalse
}

/// Raven `NPC_ClearLOS`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1069-1072`
pub fn NPC_ClearLOS(ctx: &mut GameContext, start: vec3_t, end: vec3_t) -> qboolean {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    G_ClearLOS(ctx, npc_id, start, end)
}

/// Raven `NPC_ClearLOS5`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1073-1076`
pub fn NPC_ClearLOS5(ctx: &mut GameContext, end: vec3_t) -> qboolean {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    G_ClearLOS5(ctx, npc_id, end)
}

/// Raven `NPC_ClearLOS4`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1077-1080`
pub fn NPC_ClearLOS4(ctx: &mut GameContext, ent: Option<EntityId>) -> qboolean {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    G_ClearLOS4(ctx, npc_id, ent)
}

/// Raven `NPC_ClearLOS3`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1081-1084`
pub fn NPC_ClearLOS3(ctx: &mut GameContext, start: vec3_t, ent: Option<EntityId>) -> qboolean {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    G_ClearLOS3(ctx, npc_id, start, ent)
}

/// Raven `NPC_ClearLOS2`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1085-1088`
pub fn NPC_ClearLOS2(ctx: &mut GameContext, ent: Option<EntityId>, end: vec3_t) -> qboolean {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    G_ClearLOS2(ctx, npc_id, ent, end)
}

/// Raven `NPC_ValidEnemy`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1096-1187`
pub fn NPC_ValidEnemy(ctx: &mut GameContext, ent: Option<EntityId>) -> qboolean {
    // FLAG: gNPC_t (NPC) + pool client (NPC/ent) derefs stay raw (recipe 2c / trap 2b).
    unsafe {
        //Must be a valid pointer
        let ent_id = match ent {
            Some(i) => i,
            None => return qfalse,
        };
        let npc = ctx.world.globals.NPC;
        let npc_id = ctx.entity_id_of(npc).unwrap();
        let mut ent_team: c_int = TEAM_FREE as c_int;

        //Must not be me
        if ent_id == npc_id {
            return qfalse;
        }

        //Must not be deleted
        if ctx.world.entity(ent_id).inuse == qfalse {
            return qfalse;
        }

        //Must be alive
        if ctx.world.entity(ent_id).health <= 0 {
            return qfalse;
        }

        //In case they're in notarget mode
        if (ctx.world.entity(ent_id).flags & FL_NOTARGET) != 0 {
            return qfalse;
        }

        let npc_client = ctx.world.entity(npc_id).client;

        //Must be an NPC
        if ctx.world.entity(ent_id).client.is_null() {
            //	if ( ent->svFlags&SVF_NONNPC_ENEMY )
            if ctx.world.entity(ent_id).s.eType != ET_NPC as c_int {
                //still potentially valid
                if ctx.world.entity(ent_id).alliedTeam == (*npc_client).playerTeam as c_int {
                    return qfalse;
                } else {
                    return qtrue;
                }
            } else {
                return qfalse;
            }
        } else if (*ctx.world.entity(ent_id).client).sess.sessionTeam == TEAM_SPECTATOR {
            //don't go after spectators
            return qfalse;
        }

        let ent_client = ctx.world.entity(ent_id).client;

        if !ctx.world.entity(ent_id).NPC.is_null() && !ctx.world.entity(ent_id).client.is_null() {
            ent_team = (*ent_client).playerTeam as c_int;
        } else if !ctx.world.entity(ent_id).client.is_null() {
            if ctx.world.cvars.g_gametype.integer < GT_TEAM {
                ent_team = NPCTEAM_PLAYER;
            } else {
                if (*ent_client).sess.sessionTeam == TEAM_BLUE {
                    ent_team = NPCTEAM_PLAYER;
                } else if (*ent_client).sess.sessionTeam == TEAM_RED {
                    ent_team = NPCTEAM_ENEMY;
                } else {
                    ent_team = NPCTEAM_NEUTRAL;
                }
            }
        }

        //Can't be on the same team
        if (*ent_client).playerTeam == (*npc_client).playerTeam {
            return qfalse;
        }

        let ent_enemy_id = ctx.world.entity(ent_id).enemy;

        //if haven't seen him in a while, give up
        if ent_team == (*npc_client).enemyTeam as c_int //simplest case: they're on my enemy team
            || ((*npc_client).enemyTeam as c_int == NPCTEAM_FREE && (*ent_client).NPC_class != (*npc_client).NPC_class) //I get mad at anyone and this guy isn't the same class as me
            || ((*ent_client).NPC_class == CLASS_WAMPA && ent_enemy_id.is_some()) //a rampaging wampa
            || ((*ent_client).NPC_class == CLASS_RANCOR && ent_enemy_id.is_some()) //a rampaging rancor
            || (ent_team == NPCTEAM_FREE
                && (*ent_client).enemyTeam as c_int == NPCTEAM_FREE
                && ent_enemy_id.is_some()
                && !ctx.world.entity(ent_enemy_id.unwrap()).client.is_null()
                && ({
                    let enemy_client = ctx.world.entity(ent_enemy_id.unwrap()).client;
                    (*enemy_client).playerTeam == (*npc_client).playerTeam
                        || ((*enemy_client).playerTeam as c_int != NPCTEAM_ENEMY
                            && (*npc_client).playerTeam as c_int == NPCTEAM_PLAYER)
                }))
        //enemy is a rampaging non-aligned creature who is attacking someone on our team or a non-enemy (this last condition is used only if we're a good guy - in effect, we protect the innocent)
        {
            return qtrue;
        }

        qfalse
    }
}

/// Raven `NPC_TargetVisible`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1195-1210`
pub fn NPC_TargetVisible(ctx: &mut GameContext, ent: Option<EntityId>) -> qboolean {
    let ent_id = ent.unwrap();
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: gNPC_t (NPCInfo) derefs stay raw (recipe 2c).
    let npc_info = ctx.world.globals.NPCInfo;

    let ent_origin = ctx.world.entity(ent_id).r.currentOrigin;
    let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;

    //Make sure we're in a valid range
    if DistanceSquared(ent_origin, npc_origin)
        > unsafe { (*npc_info).stats.visrange * (*npc_info).stats.visrange }
    {
        return qfalse;
    }

    //Check our FOV
    if InFOV(
        ctx,
        ent,
        npc_id,
        unsafe { (*npc_info).stats.hfov },
        unsafe { (*npc_info).stats.vfov },
    ) == qfalse
    {
        return qfalse;
    }

    //Check for sight
    if NPC_ClearLOS4(ctx, ent) == qfalse {
        return qfalse;
    }

    qtrue
}

/// Raven `NPC_FindNearestEnemy`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1246-1294`
pub fn NPC_FindNearestEnemy(ctx: &mut GameContext, ent: EntityId) -> c_int {
    // FLAG: gNPC_t (NPCInfo) derefs stay raw (recipe 2c).
    let npc_info = ctx.world.globals.NPCInfo;

    let mut nearest_ent_id: c_int = -1;
    let mut nearest_dist = WORLD_SIZE * WORLD_SIZE;

    //Setup the bbox to search in
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let ent_origin = ctx.world.entity(ent).r.currentOrigin;
    let visrange = unsafe { (*npc_info).stats.visrange };
    for i in 0..3 {
        mins[i] = ent_origin[i] - visrange;
        maxs[i] = ent_origin[i] + visrange;
    }

    //Get a number of entities in a given space
    let mut iradius_ents = [0i32; MAX_RADIUS_ENTS];
    let num_ents = trap::EntitiesInBox(
        ctx.engine,
        GEntitiesInBoxArgs::new(
            &mins as *const vec3_t,
            &maxs as *const vec3_t,
            iradius_ents.as_mut_ptr(),
            MAX_RADIUS_ENTS as c_int,
        ),
    );

    let mut i = 0;
    while i < num_ents {
        let rad_ent_id = EntityId(iradius_ents[i as usize] as u32);

        //Don't consider self
        if rad_ent_id == ent {
            i += 1;
            continue;
        }

        //Must be valid
        if NPC_ValidEnemy(ctx, Some(rad_ent_id)) == qfalse {
            i += 1;
            continue;
        }

        //Must be visible
        if NPC_TargetVisible(ctx, Some(rad_ent_id)) == qfalse {
            i += 1;
            continue;
        }

        let ent_origin = ctx.world.entity(ent).r.currentOrigin;
        let rad_origin = ctx.world.entity(rad_ent_id).r.currentOrigin;
        let distance = DistanceSquared(ent_origin, rad_origin);

        //Found one closer to us
        if distance < nearest_dist {
            nearest_ent_id = ctx.world.entity(rad_ent_id).s.number;
            nearest_dist = distance;
        }

        i += 1;
    }

    nearest_ent_id
}

/// Raven `NPC_PickEnemyExt`.
///
/// Raven: the "Hazard Team status" `NPC_FindPlayer` shortcut above is `/*
/// */`-commented out in the oracle — dead source, not transcribed.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1302-1348`
pub fn NPC_PickEnemyExt(ctx: &mut GameContext, checkAlerts: qboolean) -> *mut gentity_t {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    //If we've asked for the closest enemy
    let ent_id = NPC_FindNearestEnemy(ctx, npc_id);

    //If we have a valid enemy, use it
    if ent_id >= 0 {
        return ctx.world.entity_mut(EntityId(ent_id as u32)) as *mut gentity_t;
    }

    if checkAlerts != qfalse {
        let alert_event =
            NPC_CheckAlertEvents(ctx, qtrue, qtrue, -1, qtrue, AEL_DISCOVERED as c_int);

        //There is an event to look at
        if alert_event >= 0 {
            let event_owner = ctx.world.level.alertEvents[alert_event as usize].owner;
            let event_level = ctx.world.level.alertEvents[alert_event as usize].level;

            //Don't pay attention to our own alerts
            if event_owner == npc {
                return core::ptr::null_mut();
            }

            if (event_level as c_int) >= (AEL_DISCOVERED as c_int) {
                //If it's the player, attack him
                if ctx.entity_id_of(event_owner) == Some(EntityId(0)) {
                    return event_owner;
                }

                //If it's on our team, then take its enemy as well
                let owner_id = ctx.entity_id_of(event_owner).unwrap();
                // FLAG: owner/NPC pool client derefs stay raw (trap 2b).
                let owner_client = ctx.world.entity(owner_id).client;
                if !owner_client.is_null() {
                    let npc_client = ctx.world.entity(npc_id).client;
                    if unsafe { (*owner_client).playerTeam == (*npc_client).playerTeam } {
                        return match ctx.world.entity(owner_id).enemy {
                            Some(id) => ctx.world.entity_mut(id) as *mut gentity_t,
                            None => core::ptr::null_mut(),
                        };
                    }
                }
            }
        }
    }

    core::ptr::null_mut()
}

/// Raven `NPC_FindPlayer`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1356-1359`
pub fn NPC_FindPlayer(ctx: &mut GameContext) -> qboolean {
    NPC_TargetVisible(ctx, EntityId::from_num(0))
}

/// Raven `NPC_CheckPlayerDistance`.
///
/// Raven: the live body is a hardcoded `return qfalse; //MOOT in MP` — the
/// entire real implementation is `#if 0`-style commented out (dead in this
/// build); faithfully preserved as an always-false stub.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1367-1399`
fn NPC_CheckPlayerDistance() -> qboolean {
    qfalse
}

/// Raven `NPC_FindEnemy`.
///
/// Raven: the `SVF_IGNORE_ENEMIES` branch is hardcoded `if (0)` dead source
/// in the oracle (`//rwwFIXMEFIXME: support for flag`) — kept as the
/// always-false condition it faithfully is.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1407-1461`
pub fn NPC_FindEnemy(ctx: &mut GameContext, checkAlerts: qboolean) -> qboolean {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: gNPC_t (NPCInfo) + pool client (NPC) derefs stay raw (recipe 2c / trap 2b).
    let npc_info = ctx.world.globals.NPCInfo;

    //We're ignoring all enemies for now
    //if( NPC->svFlags & SVF_IGNORE_ENEMIES )
    if false {
        //rwwFIXMEFIXME: support for flag
        G_ClearEnemy(ctx, npc_id);
        return qfalse;
    }

    //we can't pick up any enemies for now
    if unsafe { (*npc_info).confusionTime } > ctx.world.level.time {
        return qfalse;
    }

    //Don't want a new enemy
    //rwwFIXMEFIXME: support for locked enemy

    //See if the player is closer than our current enemy
    if NPC_CheckPlayerDistance() != qfalse {
        return qtrue;
    }

    //Otherwise, turn off the flag
    //See if the player is closer than our current enemy
    let npc_client = ctx.world.entity(npc_id).client;
    if unsafe { (*npc_client).NPC_class != CLASS_RANCOR && (*npc_client).NPC_class != CLASS_WAMPA }
        && NPC_CheckPlayerDistance() != qfalse
    {
        //rancors, wampas & sand creatures don't care if player is closer, they always go with closest
        return qtrue;
    }

    //If we've gotten here alright, then our target it still valid
    let npc_enemy_id = ctx.world.entity(npc_id).enemy;
    if NPC_ValidEnemy(ctx, npc_enemy_id) != qfalse {
        return qtrue;
    }

    let newenemy = NPC_PickEnemyExt(ctx, checkAlerts);

    //if we found one, take it as the enemy
    if NPC_ValidEnemy(ctx, ctx.entity_id_of(newenemy)) != qfalse {
        G_SetEnemy(ctx, npc_id, ctx.entity_id_of(newenemy));
        return qtrue;
    }

    qfalse
}

/// Raven `NPC_CheckEnemyExt`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1469-1483`
pub fn NPC_CheckEnemyExt(ctx: &mut GameContext, checkAlerts: qboolean) -> qboolean {
    NPC_FindEnemy(ctx, checkAlerts)
}

/// Raven `NPC_FacePosition`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1491-1547`
pub fn NPC_FacePosition(ctx: &mut GameContext, position: vec3_t, doPitch: qboolean) -> qboolean {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: gNPC_t (NPCInfo) + pool client (NPC/enemy) derefs stay raw (recipe 2c / trap 2b).
    let npc_info = ctx.world.globals.NPCInfo;
    let client = ctx.world.globals.client;

    let mut muzzle: vec3_t = [0.0; 3];
    let mut angles: vec3_t = [0.0; 3];
    let mut facing = qtrue;

    let npc_client = ctx.world.entity(npc_id).client;

    //Get the positions
    if !npc_client.is_null()
        && unsafe {
            (*npc_client).NPC_class == CLASS_RANCOR || (*npc_client).NPC_class == CLASS_WAMPA
        }
    {
        CalcEntitySpot(ctx, Some(npc_id), spot_t::SPOT_ORIGIN, &mut muzzle);
        muzzle[2] += ctx.world.entity(npc_id).r.maxs[2] * 0.75;
    } else if !npc_client.is_null() && unsafe { (*npc_client).NPC_class == CLASS_GALAKMECH } {
        CalcEntitySpot(ctx, Some(npc_id), spot_t::SPOT_WEAPON, &mut muzzle);
    } else {
        CalcEntitySpot(ctx, Some(npc_id), spot_t::SPOT_HEAD_LEAN, &mut muzzle); //SPOT_HEAD
    }

    //Find the desired angles
    GetAnglesForDirection(muzzle, position, &mut angles);

    unsafe {
        (*npc_info).desiredYaw = AngleNormalize360(angles[YAW]);
        (*npc_info).desiredPitch = AngleNormalize360(angles[PITCH]);
    }

    if let Some(enemy_id) = ctx.world.entity(npc_id).enemy {
        let enemy_client = ctx.world.entity(enemy_id).client;
        if !enemy_client.is_null() {
            if unsafe { (*enemy_client).NPC_class } == CLASS_ATST {
                // FIXME: this is kind of dumb, but it was the easiest way to get it to look sort of ok
                // C's `sin` is the double libm function: the float `time*0.004f`
                // argument widens to f64, `sin(...)*7` and the `flrand` sum
                // evaluate in f64; `+=` promotes desiredYaw and narrows once.
                // Source: oracle/codemp/game/NPC_utils.c:1522
                unsafe {
                    (*npc_info).desiredYaw = ((*npc_info).desiredYaw as f64
                        + ctx.world.bg_state.rng.flrand(-5.0, 5.0) as f64
                        + (((ctx.world.level.time as f32) * 0.004) as f64).sin() * 7.0)
                        as f32;
                    (*npc_info).desiredPitch += ctx.world.bg_state.rng.flrand(-2.0, 2.0);
                }
            }
        }
    }
    //Face that yaw
    NPC_UpdateAngles(ctx, qtrue, qtrue);

    //Find the delta between our goal and our current facing
    let yaw_delta = AngleNormalize360(unsafe {
        (*npc_info).desiredYaw
            - SHORT2ANGLE(ctx.world.globals.ucmd.angles[YAW] + (*client).ps.delta_angles[YAW])
    });

    //See if we are facing properly
    if yaw_delta.abs() > VALID_ATTACK_CONE {
        facing = qfalse;
    }

    if doPitch != qfalse {
        //Find the delta between our goal and our current facing
        let current_angles = unsafe {
            SHORT2ANGLE(ctx.world.globals.ucmd.angles[PITCH] + (*client).ps.delta_angles[PITCH])
        };
        let pitch_delta = unsafe { (*npc_info).desiredPitch } - current_angles;

        //See if we are facing properly
        if pitch_delta.abs() > VALID_ATTACK_CONE {
            facing = qfalse;
        }
    }

    facing
}

/// Raven `NPC_FaceEntity`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1555-1563`
pub fn NPC_FaceEntity(ctx: &mut GameContext, ent: Option<EntityId>, doPitch: qboolean) -> qboolean {
    let mut entPos: vec3_t = [0.0; 3];
    CalcEntitySpot(ctx, ent, spot_t::SPOT_HEAD_LEAN, &mut entPos);
    NPC_FacePosition(ctx, entPos, doPitch)
}

/// Raven `NPC_FaceEnemy`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1571-1580`
pub fn NPC_FaceEnemy(ctx: &mut GameContext, doPitch: qboolean) -> qboolean {
    let npc = ctx.world.globals.NPC;

    if npc.is_null() {
        return qfalse;
    }
    let npc_id = ctx.entity_id_of(npc).unwrap();

    let enemy_id = match ctx.world.entity(npc_id).enemy {
        Some(id) => id,
        None => return qfalse,
    };

    NPC_FaceEntity(ctx, Some(enemy_id), doPitch)
}

/// Raven `NPC_CheckCanAttackExt`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1588-1603`
pub fn NPC_CheckCanAttackExt(ctx: &mut GameContext) -> qboolean {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: gNPC_t (NPCInfo) deref stays raw (recipe 2c).
    let npc_info = ctx.world.globals.NPCInfo;

    //We don't want them to shoot
    if unsafe { (*npc_info).scriptFlags & SCF_DONT_FIRE } != 0 {
        return qfalse;
    }

    //Turn to face
    if NPC_FaceEnemy(ctx, qtrue) == qfalse {
        return qfalse;
    }

    //Must have a clear line of sight to the target
    let npc_enemy_id = ctx.world.entity(npc_id).enemy;
    if NPC_ClearShot(ctx, npc_enemy_id) == qfalse {
        return qfalse;
    }

    qtrue
}

/// Raven `NPC_ClearLookTarget`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1611-1625`
pub fn NPC_ClearLookTarget(self_: &mut gentity_t) {
    // FLAG: entity's client pointer dereffed raw (trap 2b).
    let client = self_.client;
    if client.is_null() {
        return;
    }

    unsafe {
        if (*client).ps.eFlags2 & EF2_HELD_BY_MONSTER != 0 {
            //lookTarget is set by and to the monster that's holding you, no
            //other operations can change that
            return;
        }

        (*client).renderInfo.lookTarget = ENTITYNUM_NONE; //ENTITYNUM_WORLD;
        (*client).renderInfo.lookTargetClearTime = 0;
    }
}

/// Raven `NPC_SetLookTarget`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1632-1646`
pub fn NPC_SetLookTarget(self_: &mut gentity_t, entNum: c_int, clearTime: c_int) {
    // FLAG: entity's client pointer dereffed raw (trap 2b).
    let client = self_.client;
    if client.is_null() {
        return;
    }

    unsafe {
        if (*client).ps.eFlags2 & EF2_HELD_BY_MONSTER != 0 {
            //lookTarget is set by and to the monster that's holding you, no
            //other operations can change that
            return;
        }

        (*client).renderInfo.lookTarget = entNum;
        (*client).renderInfo.lookTargetClearTime = clearTime;
    }
}

/// Raven `NPC_CheckLookTarget`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1653-1679`
pub fn NPC_CheckLookTarget(ctx: &mut GameContext, self_: EntityId) -> qboolean {
    // FLAG: entity's client pointer dereffed raw (trap 2b).
    let client = ctx.world.entity(self_).client;
    if !client.is_null() {
        let lookTarget = unsafe { (*client).renderInfo.lookTarget };

        if lookTarget >= 0 && lookTarget < ENTITYNUM_WORLD {
            //within valid range
            let target_id = EntityId(lookTarget as u32);
            if ctx.world.entity(target_id).inuse == qfalse {
                //lookTarget not inuse or not valid anymore
                NPC_ClearLookTarget(ctx.entity_mut(self_));
            } else if unsafe { (*client).renderInfo.lookTargetClearTime } != 0
                && unsafe { (*client).renderInfo.lookTargetClearTime } < ctx.world.level.time
            {
                //Time to clear lookTarget
                NPC_ClearLookTarget(ctx.entity_mut(self_));
            } else if !ctx.world.entity(target_id).client.is_null()
                && !ctx.world.entity(self_).enemy.is_none()
                && ctx.world.entity(self_).enemy.map(|id| id.index()) != Some(lookTarget as usize)
            {
                //should always look at current enemy if engaged in
                //battle... FIXME: this could override certain scripted
                //lookTargets...???
                NPC_ClearLookTarget(ctx.entity_mut(self_));
            } else {
                return qtrue;
            }
        }
    }

    qfalse
}

/// Raven `NPC_CheckCharmed`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1687-1705`
pub fn NPC_CheckCharmed(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: gNPC_t (NPCInfo) + pool client (NPC) derefs stay raw (recipe 2c / trap 2b).
    let npc_info = ctx.world.globals.NPCInfo;

    let client = ctx.world.entity(npc_id).client;
    if unsafe { (*npc_info).charmedTime } != 0
        && unsafe { (*npc_info).charmedTime } < ctx.world.level.time
        && !client.is_null()
    {
        //we were charmed, set us back!
        let generic1 = ctx.world.entity(npc_id).genericValue1;
        let generic2 = ctx.world.entity(npc_id).genericValue2;
        let generic3 = ctx.world.entity(npc_id).genericValue3;
        unsafe {
            (*client).playerTeam = generic1;
            (*client).enemyTeam = generic2;
        }
        ctx.world.entity_mut(npc_id).s.teamowner = generic3;
        unsafe {
            (*client).leader = None;
            if (*npc_info).tempBehavior == bState_t::BS_FOLLOW_LEADER {
                (*npc_info).tempBehavior = bState_t::BS_DEFAULT;
            }
        }
        G_ClearEnemy(ctx, npc_id);
        unsafe {
            (*npc_info).charmedTime = 0;
        }
        let confuse_event = ctx.world.bg_state.rng.Q_irand(
            entity_event_t::EV_CONFUSE1 as c_int,
            entity_event_t::EV_CONFUSE3 as c_int,
        );
        //say something to let player know you've snapped out of it
        G_AddVoiceEvent(ctx, npc_id, confuse_event, 2000);
    }
}

/// Raven `G_GetBoltPosition`.
///
/// Out-param reshape: `pos` is guarded by `if (pos)` in the oracle (the
/// AngleVectors NULL-able idiom), so it becomes `Option<&mut vec3_t>`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1707-1740`
pub fn G_GetBoltPosition(
    ctx: &mut GameContext,
    self_: Option<EntityId>,
    boltIndex: c_int,
    pos: Option<&mut vec3_t>,
    modelIndex: c_int,
) {
    let self_id = match self_ {
        Some(i) => i,
        None => return,
    };
    if ctx.world.entity(self_id).inuse == qfalse {
        return;
    }

    let mut angles: vec3_t = [0.0; 3];
    // FLAG: entity's client pointer dereffed raw via the safe borrow (trap 2b).
    let client = ctx.world.entity(self_id).client;
    if !client.is_null() {
        angles[0] = 0.0;
        angles[1] = unsafe { (*client).ps.viewangles[YAW] };
        angles[2] = 0.0;
    } else {
        angles[0] = 0.0;
        angles[1] = ctx.world.entity(self_id).r.currentAngles[YAW];
        angles[2] = 0.0;
    }

    if ctx.world.entity(self_id).ghoul2.is_null() {
        return;
    }

    let mut boltMatrix: mdxaBone_t = unsafe { core::mem::zeroed() };
    let ghoul2 = ctx.world.entity(self_id).ghoul2;
    let currentOrigin = ctx.world.entity(self_id).r.currentOrigin;
    let modelScale = ctx.world.entity(self_id).modelScale;
    let level_time = ctx.world.level.time;
    trap::G2API_GetBoltMatrix(
        ctx.engine,
        GG2GetboltArgs::new(
            ghoul2,
            modelIndex,
            boltIndex,
            &mut boltMatrix as *mut mdxaBone_t,
            &angles as *const vec3_t,
            &currentOrigin as *const vec3_t,
            level_time,
            core::ptr::null_mut(),
            &modelScale as *const vec3_t,
        ),
    );

    if let Some(pos_ref) = pos {
        let mut result: vec3_t = [0.0; 3];
        BG_GiveMeVectorFromMatrix(&boltMatrix as *const mdxaBone_t, ORIGIN, &mut result);
        _VectorCopy(result, pos_ref);
    }
}

/// Raven `NPC_EntRangeFromBolt`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1742-1754`
pub fn NPC_EntRangeFromBolt(
    ctx: &mut GameContext,
    targEnt: Option<EntityId>,
    boltIndex: c_int,
) -> f32 {
    let targEnt_id = match targEnt {
        Some(i) => i,
        None => return Q3_INFINITE,
    };
    let npc = ctx.world.globals.NPC;

    let mut org: vec3_t = [0.0; 3];
    G_GetBoltPosition(ctx, ctx.entity_id_of(npc), boltIndex, Some(&mut org), 0);

    let targ_origin = ctx.world.entity(targEnt_id).r.currentOrigin;
    Distance(targ_origin, org)
}

/// Raven `NPC_EnemyRangeFromBolt`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1756-1759`
pub fn NPC_EnemyRangeFromBolt(ctx: &mut GameContext, boltIndex: c_int) -> f32 {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    let enemy_id = ctx.world.entity(npc_id).enemy;
    NPC_EntRangeFromBolt(ctx, enemy_id, boltIndex)
}

/// Raven `NPC_GetEntsNearBolt`.
///
/// Out-param reshape: `boltOrg` is written unconditionally
/// (`VectorCopy(org, boltOrg)`), so it becomes the non-nullable
/// `&mut vec3_t`.
///
/// Source: `oracle/codemp/game/NPC_utils.c:1761-1782`
pub fn NPC_GetEntsNearBolt(
    ctx: &mut GameContext,
    radiusEnts: *mut c_int,
    radius: f32,
    boltIndex: c_int,
    boltOrg: &mut vec3_t,
) -> c_int {
    let npc = ctx.world.globals.NPC;

    //get my handRBolt's position
    let mut org: vec3_t = [0.0; 3];

    G_GetBoltPosition(ctx, ctx.entity_id_of(npc), boltIndex, Some(&mut org), 0);

    *boltOrg = org;

    //Setup the bbox to search in
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    for i in 0..3 {
        mins[i] = boltOrg[i] - radius;
        maxs[i] = boltOrg[i] + radius;
    }

    //Get the number of entities in a given space
    trap::EntitiesInBox(
        ctx.engine,
        GEntitiesInBoxArgs::new(
            &mins as *const vec3_t,
            &maxs as *const vec3_t,
            radiusEnts,
            128,
        ),
    )
}
