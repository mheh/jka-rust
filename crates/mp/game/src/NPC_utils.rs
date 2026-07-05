// PORT-COMPLETE: NPC_utils.c 23/24 (pass-2, packets/NPC_utils.md)
//! Port of `oracle/oracle/codemp/game/NPC_utils.c` (jampgame mega-pass).
//!
//! Generated from `tools/closure-prototype/fnskel.py`; bodies filled per the
//! jampgame mega-pass (settled fork rulings,
//! `docs/handoffs/jampgame-fork-discovery.md`), then the pass-2 sweep
//! (`packets/NPC_utils.md`) that resolved the `ai-context` park class below.
//!
//! SPINE (fork rulings 1/4 + `docs/architecture/engine-seam.md`, precedent
//! `w_force.rs`/`g_client.rs`): logic fns that reach `level`/cvars/`g_entities`/
//! traps thread the `GameContext<'_>` receiver (`.world: *mut GameWorld`,
//! `.engine`) as an ADDITIVE first parameter (the faithful C signature carries
//! none). Globals are `GameWorld` fields (fork 1): `level` →
//! `(*ctx.world).level`, `g_entities[i]` → `(*ctx.world).g_entities[i]`; this
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
//! `G_GetBoltPosition`'s `pos` got the fork-9 vec3 out-param reshape
//! (`&mut vec3_t` / `Option<&mut vec3_t>`) and same-file callers were fixed up.
//!
//! Two markers remain (see PORT-NOTE): `NPC_SetSurfaceOnOff` needs
//! `bgToggleableSurfaces` (genuinely unported bg-shared table — no Rust home
//! anywhere in the worktree) and `G_ActivateBehavior` needs `BSTable`
//! (unported ICARUS string table, same class of gap as `Q3_SetBState` in
//! `g_ICARUScb.rs`) plus a real-variadic `va(fmt, args…)` call (topic
//! `va-varargs`, same fork as `g_client.rs`/`w_force.rs`). `G_GetBoltPosition`
//! is parked on a cross-crate-visibility gap: `BG_GiveMeVectorFromMatrix`
//! (`NPC_AI_Mark2.rs`) is a private fn in its owning file.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::NPC_senses::{InFOV, G_ClearLOS, G_ClearLOS2, G_ClearLOS3, G_ClearLOS4, G_ClearLOS5, NPC_CheckAlertEvents};
use crate::NPC_combat::{G_ClearEnemy, G_SetEnemy, NPC_ClearShot};
use crate::NPC_sounds::G_AddVoiceEvent;
use crate::g_utils::{G_BoneIndex, GetAnglesForDirection};
use crate::q_shared::{Q_stricmp, GetIDForString, GetStringForID, va};
use crate::q_math::{AngleDelta, AngleNormalize360, AngleVectors, Distance, vec3_origin, PITCH, YAW, ROLL, _VectorCopy};
use crate::g_target::Q3_SCRIPT_DIR;
use crate::teams::npcteam::{NPCTEAM_PLAYER, NPCTEAM_ENEMY, NPCTEAM_NEUTRAL, NPCTEAM_FREE};
use crate::level::alert_event::{alertEvent_t, alertEventLevel_e::AEL_DISCOVERED};
use crate::bg_lib::atof;
use mp_qshared::shared::force_powers::FP_SPEED;
use crate::trap;
use crate::world::GameContext;
pub use crate::NPC_goal::UpdateGoal;

use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use mp_abi::game::syscalls::G_ENTITIES_IN_BOX::GEntitiesInBoxArgs;
use mp_abi::game::syscalls::G_CVAR_VARIABLE_STRING_BUFFER::GCvarVariableStringBufferArgs;
use mp_abi::game::syscalls::G_ICARUS_TASKIDPENDING::GIcarusTaskidpendingArgs;
use mp_abi::game::syscalls::G_ICARUS_TASKIDCOMPLETE::GIcarusTaskidcompleteArgs;
use mp_abi::game::syscalls::G_ICARUS_RUNSCRIPT::GIcarusRunscriptArgs;
use mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs;

// Raven `#define VALID_ATTACK_CONE 2.0f` (this file's own macro).
// Source: `oracle/oracle/codemp/game/NPC_utils.c:11`
pub const VALID_ATTACK_CONE: f32 = 2.0;

// Raven `#define MIN_ANGLE_ERROR 0.01f` (`b_local.h`).
// Source: `oracle/oracle/codemp/game/b_local.h:29`
const MIN_ANGLE_ERROR: f32 = 0.01;

// Raven `#define Q3_INFINITE 16777216` (`g_public.h`).
// Source: `oracle/oracle/codemp/game/g_public.h:9`
const Q3_INFINITE: f32 = 16777216.0;

// Raven `#define WORLD_SIZE ( MAX_WORLD_COORD - MIN_WORLD_COORD )` (65536 -
// (-65536) = 131072). Per-file local const, same idiom as `NPC_combat.rs`.
// Source: `oracle/oracle/codemp/game/q_shared.h`
const WORLD_SIZE: f32 = 131072.0;

// Raven `#define MASK_PLAYERSOLID (CONTENTS_SOLID|CONTENTS_PLAYERCLIP|CONTENTS_BODY|CONTENTS_TERRAIN)`.
// Source: `oracle/oracle/codemp/game/q_shared.h`
const MASK_PLAYERSOLID: c_int = CONTENTS_SOLID | CONTENTS_PLAYERCLIP | CONTENTS_BODY | CONTENTS_TERRAIN;

// Raven `#define MAX_RADIUS_ENTS 128` (per-file local const, same idiom as
// `NPC_AI_Utils.rs`).
const MAX_RADIUS_ENTS: usize = 128;

// Raven `SCF_DONT_FIRE` (`gNPC_t::scriptFlags` bit), per-file local const,
// same idiom as `NPC_combat.rs`.
// Source: `oracle/oracle/codemp/game/b_public.h:41`
const SCF_DONT_FIRE: c_int = 0x00004000;

/// Raven `ANGLE2SHORT(x)` — `((int)((x)*65536/360) & 65535)`.
/// Source: `oracle/oracle/codemp/game/q_shared.h:1972`
pub(crate) fn ANGLE2SHORT(x: f32) -> c_int {
    (((x * 65536.0 / 360.0) as c_int) & 65535) as c_int
}

/// Raven `SHORT2ANGLE(x)` — `((x)*(360.0/65536))`.
/// Source: `oracle/oracle/codemp/game/q_shared.h:1973`
pub(crate) fn SHORT2ANGLE(x: c_int) -> f32 {
    (x as f32) * (360.0 / 65536.0)
}

// `DistanceSquared` is the canonical `crate::q_math::DistanceSquared`, reached
// via the prelude glob (no per-file copy).

/// Raven `BONE_ANGLES_POSTMULT` (ghoul2 bone-angle apply mode).
/// Source: `oracle/oracle/code/game/ghoul2_shared.h:54`
pub const BONE_ANGLES_POSTMULT: c_int = 0x0002;

/// Raven `TURN_ON` flag for surface toggling.
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1022`
const TURN_ON: c_int = 0x00000000;

/// Raven `ORIGIN` — extract the origin vector from a bolt matrix.
/// Source: `oracle/oracle/codemp/game/ghoul2_shared.h` (Eorientations enum)
const ORIGIN: c_int = Eorientations::ORIGIN as c_int;

/// Raven `q_shared.h:30` `VALIDSTRING(a)` macro.
/// Source: `oracle/oracle/codemp/game/q_shared.h:30`
unsafe fn VALIDSTRING(a: *const c_char) -> bool {
    !a.is_null() && *a as c_int != 0
}

/// Raven `BG_NUM_TOGGLEABLE_SURFACES`.
/// Source: `oracle/oracle/codemp/game/bg_public.h:138`
pub const BG_NUM_TOGGLEABLE_SURFACES: c_int = 31;

/// Raven `PMF_FOLLOW` — spectate following another player.
/// Source: `oracle/oracle/codemp/game/bg_public.h:415`
const PMF_FOLLOW: c_int = 4096;

use mp_bg::public::team::{TEAM_SPECTATOR, TEAM_FREE, TEAM_BLUE, TEAM_RED};
use mp_qshared::shared::MAX_CLIENTS;

use mp_abi::game::syscalls::G_G2_ANGLEOVERRIDE::GG2AngleoverrideArgs;
use mp_abi::game::syscalls::G_G2_SETSURFACEONOFF::GG2SetsurfaceonoffArgs;
use mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs;

/// Raven `CalcEntitySpot`.
///
/// Fork-9 reshape: `point` is written unconditionally on every branch (no
/// oracle caller passes NULL), so it becomes the non-nullable `&mut vec3_t`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:20-168`
pub fn CalcEntitySpot(
    ctx: GameContext<'_>,
    ent: *const gentity_t,
    spot: spot_t,
    point: &mut vec3_t,
) {
    unsafe {
        if ent.is_null() {
            return;
        }

        match spot {
            spot_t::SPOT_ORIGIN => {
                if (*ent).r.currentOrigin == vec3_origin {
                    //brush
                    let size = [
                        (*ent).r.absmax[0] - (*ent).r.absmin[0],
                        (*ent).r.absmax[1] - (*ent).r.absmin[1],
                        (*ent).r.absmax[2] - (*ent).r.absmin[2],
                    ];
                    for i in 0..3 {
                        point[i] = (*ent).r.absmin[i] + 0.5 * size[i];
                    }
                } else {
                    *point = (*ent).r.currentOrigin;
                }
            }
            spot_t::SPOT_CHEST | spot_t::SPOT_HEAD => {
                let client = (*ent).client as *mut gclient_t;
                //Actual tag_head eyespot!
                //FIXME: Stasis aliens may have a problem here...
                if !client.is_null() && VectorLengthSquared((*client).renderInfo.eyePoint) != 0.0 {
                    *point = (*client).renderInfo.eyePoint;
                    if (*client).NPC_class == CLASS_ATST {
                        //adjust up some
                        point[2] += 28.0; //magic number :)
                    }
                    if !(*ent).NPC.is_null() {
                        //always aim from the center of my bbox, so we don't wiggle when we lean forward or backwards
                        point[0] = (*ent).r.currentOrigin[0];
                        point[1] = (*ent).r.currentOrigin[1];
                    }
                } else {
                    *point = (*ent).r.currentOrigin;
                    if !client.is_null() {
                        point[2] += (*client).ps.viewheight as f32;
                    }
                }
                if matches!(spot, spot_t::SPOT_CHEST) && !client.is_null() {
                    if (*client).NPC_class != CLASS_ATST {
                        //adjust up some
                        point[2] -= (*ent).r.maxs[2] * 0.2;
                    }
                }
            }
            spot_t::SPOT_HEAD_LEAN => {
                let client = (*ent).client as *mut gclient_t;
                if !client.is_null() && VectorLengthSquared((*client).renderInfo.eyePoint) != 0.0 {
                    //Actual tag_head eyespot!
                    *point = (*client).renderInfo.eyePoint;
                    if (*client).NPC_class == CLASS_ATST {
                        point[2] += 28.0;
                    }
                    if !(*ent).NPC.is_null() {
                        point[0] = (*ent).r.currentOrigin[0];
                        point[1] = (*ent).r.currentOrigin[1];
                    }
                    //NOTE: automatically takes leaning into account!
                } else {
                    *point = (*ent).r.currentOrigin;
                    if !client.is_null() {
                        point[2] += (*client).ps.viewheight as f32;
                    }
                }
            }
            spot_t::SPOT_LEGS => {
                *point = (*ent).r.currentOrigin;
                point[2] += (*ent).r.mins[2] * 0.5;
            }
            spot_t::SPOT_WEAPON => {
                let mut forward: vec3_t = [0.0; 3];
                let mut right: vec3_t = [0.0; 3];
                let mut up: vec3_t = [0.0; 3];
                let npc = (*ent).NPC as *mut gNPC_t;
                let client = (*ent).client as *mut gclient_t;
                let use_shoot_angles = !npc.is_null()
                    && (*npc).shootAngles != vec3_origin
                    && (*npc).shootAngles != (*client).ps.viewangles;
                if use_shoot_angles {
                    AngleVectors((*npc).shootAngles, Some(&mut forward), Some(&mut right), Some(&mut up));
                } else {
                    AngleVectors((*client).ps.viewangles, Some(&mut forward), Some(&mut right), Some(&mut up));
                }
                crate::g_weapon::CalcMuzzlePoint(ctx, ent as *mut gentity_t, forward, right, up, point);
                //NOTE: automatically takes leaning into account!
            }
            spot_t::SPOT_GROUND => {
                // if entity is on the ground, just use it's absmin
                if (*ent).s.groundEntityNum != -1 {
                    *point = (*ent).r.currentOrigin;
                    point[2] = (*ent).r.absmin[2];
                    return;
                }

                // if it is reasonably close to the ground, give the point underneath of it
                let mut start = (*ent).r.currentOrigin;
                start[2] = (*ent).r.absmin[2];
                let mut end = start;
                end[2] -= 64.0;
                let mut tr: trace_t = core::mem::zeroed();
                trap::Trace(
                    ctx.engine,
                    GTraceArgs::new(
                        &mut tr,
                        &start,
                        &(*ent).r.mins,
                        &(*ent).r.maxs,
                        &end,
                        (*ent).s.number,
                        MASK_PLAYERSOLID,
                    ),
                );
                if tr.fraction < 1.0 {
                    *point = tr.endpos;
                    return;
                }

                // otherwise just use the origin
                *point = (*ent).r.currentOrigin;
            }
        }
    }
}

/// Raven `NPC_UpdateAngles`.
///
/// Raven: the `#if 1` branch is the compiled one (the `#else` branch below it
/// is dead source, per house ruling on `#if 0`/`#if 1` branches) — only that
/// branch is transcribed.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:182-517`
pub fn NPC_UpdateAngles(
    ctx: GameContext<'_>,
    doPitch: qboolean,
    doYaw: qboolean,
) -> qboolean {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;
        let client = (*ctx.world).globals.client;

        let mut target_pitch: f32 = 0.0;
        let mut target_yaw: f32 = 0.0;
        let mut exact = QTRUE;

        // if angle changes are locked; just keep the current angles
        // aimTime isn't even set anymore... so this code was never reached, but I need a way to lock NPC's yaw, so instead of making a new SCF_ flag, just use the existing render flag... - dmv
        if (*npc).enemy.is_none() && (*ctx.world).level.time < (*npc_info).aimTime {
            if doPitch != QFALSE {
                target_pitch = (*npc_info).lockedDesiredPitch;
            }
            if doYaw != QFALSE {
                target_yaw = (*npc_info).lockedDesiredYaw;
            }
        } else {
            // we're changing the lockedDesired Pitch/Yaw below so it's lost it's original meaning, get rid of the lock flag
            if doPitch != QFALSE {
                target_pitch = (*npc_info).desiredPitch;
                (*npc_info).lockedDesiredPitch = (*npc_info).desiredPitch;
            }
            if doYaw != QFALSE {
                target_yaw = (*npc_info).desiredYaw;
                (*npc_info).lockedDesiredYaw = (*npc_info).desiredYaw;
            }
        }

        let mut yaw_speed: f32;
        if (*npc).s.weapon == WP_EMPLACED_GUN {
            // FIXME: this seems to do nothing, actually...
            yaw_speed = 20.0;
        } else {
            yaw_speed = (*npc_info).stats.yawSpeed;
        }

        let npc_client = (*npc).client as *mut gclient_t;
        if (*npc).s.weapon == WP_SABER && ((*npc_client).ps.fd.forcePowersActive & (1 << (FP_SPEED as c_int))) != 0 {
            let mut buf = [0i8; 128];
            trap::Cvar_VariableStringBuffer(
                ctx.engine,
                GCvarVariableStringBufferArgs::new(
                    std::ffi::CString::new("timescale").unwrap(),
                    buf.as_mut_ptr(),
                    buf.len() as c_int,
                ),
            );
            let t_f_val = atof(buf.as_ptr());
            yaw_speed *= 1.0 / (t_f_val as f32);
        }

        if doYaw != QFALSE {
            // decay yaw error
            let mut error = AngleDelta((*npc_client).ps.viewangles[YAW], target_yaw);
            if error.abs() > MIN_ANGLE_ERROR {
                if error != 0.0 {
                    exact = QFALSE;

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

            (*ctx.world).globals.ucmd.angles[YAW] =
                ANGLE2SHORT(target_yaw + error) - (*client).ps.delta_angles[YAW];
        }

        //FIXME: have a pitchSpeed?
        if doPitch != QFALSE {
            // decay pitch error
            let mut error = AngleDelta((*npc_client).ps.viewangles[PITCH], target_pitch);
            if error.abs() > MIN_ANGLE_ERROR {
                if error != 0.0 {
                    exact = QFALSE;

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

            (*ctx.world).globals.ucmd.angles[PITCH] =
                ANGLE2SHORT(target_pitch + error) - (*client).ps.delta_angles[PITCH];
        }

        (*ctx.world).globals.ucmd.angles[ROLL] =
            ANGLE2SHORT((*npc_client).ps.viewangles[ROLL]) - (*client).ps.delta_angles[ROLL];

        if exact != QFALSE
            && trap::ICARUS_TaskIDPending(ctx.engine, GIcarusTaskidpendingArgs::new(npc, taskID_t::TID_ANGLE_FACE as c_int)) != 0
        {
            trap::ICARUS_TaskIDComplete(ctx.engine, GIcarusTaskidcompleteArgs::new(npc, taskID_t::TID_ANGLE_FACE as c_int));
        }
        exact
    }
}

/// Raven `NPC_AimWiggle`.
///
/// Fork-9 reshape: `enemy_org` is mutated in place (`VectorAdd(enemy_org,
/// NPCInfo->aimOfs, enemy_org)`), so it becomes `&mut vec3_t`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:519-533`
pub fn NPC_AimWiggle(
    ctx: GameContext<'_>,
    enemy_org: &mut vec3_t,
) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;

        //shoot for somewhere between the head and torso
        //NOTE: yes, I know this looks weird, but it works
        if (*npc_info).aimErrorDebounceTime < (*ctx.world).level.time {
            // Raven derefs `NPC->enemy` unconditionally here (assumed non-null
            // by the caller).
            let enemy = &mut (*ctx.world).g_entities[(*npc).enemy.unwrap().index()] as *mut gentity_t;
            (*npc_info).aimOfs[0] = 0.3 * (*ctx.world).bg_state.rng.flrand((*enemy).r.mins[0], (*enemy).r.maxs[0]);
            (*npc_info).aimOfs[1] = 0.3 * (*ctx.world).bg_state.rng.flrand((*enemy).r.mins[1], (*enemy).r.maxs[1]);
            if (*enemy).r.maxs[2] > 0.0 {
                (*npc_info).aimOfs[2] = (*enemy).r.maxs[2] * (*ctx.world).bg_state.rng.flrand(0.0, -1.0);
            }
        }
        for i in 0..3 {
            enemy_org[i] += (*npc_info).aimOfs[i];
        }
    }
}

/// Raven `NPC_UpdateFiringAngles`.
///
/// Raven: the `#else` branch is the compiled one (`#if 0` above it is dead
/// source, per house ruling on `#if 0` branches) — only that branch is
/// transcribed.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:540-731`
pub fn NPC_UpdateFiringAngles(
    ctx: GameContext<'_>,
    doPitch: qboolean,
    doYaw: qboolean,
) -> qboolean {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;
        let client = (*ctx.world).globals.client;

        let mut target_pitch: f32 = 0.0;
        let mut target_yaw: f32 = 0.0;
        let mut exact = QTRUE;

        // if angle changes are locked; just keep the current angles
        if (*ctx.world).level.time < (*npc_info).aimTime {
            if doPitch != QFALSE {
                target_pitch = (*npc_info).lockedDesiredPitch;
            }
            if doYaw != QFALSE {
                target_yaw = (*npc_info).lockedDesiredYaw;
            }
        } else {
            if doPitch != QFALSE {
                target_pitch = (*npc_info).desiredPitch;
            }
            if doYaw != QFALSE {
                target_yaw = (*npc_info).desiredYaw;
            }

            if doPitch != QFALSE {
                (*npc_info).lockedDesiredPitch = (*npc_info).desiredPitch;
            }
            if doYaw != QFALSE {
                (*npc_info).lockedDesiredYaw = (*npc_info).desiredYaw;
            }
        }

        if (*npc_info).aimErrorDebounceTime < (*ctx.world).level.time {
            if (*ctx.world).bg_state.rng.Q_irand(0, 1) != 0 {
                (*npc_info).lastAimErrorYaw = ((6 - (*npc_info).stats.aim) as f32) * (*ctx.world).bg_state.rng.flrand(-1.0, 1.0);
            }
            if (*ctx.world).bg_state.rng.Q_irand(0, 1) != 0 {
                (*npc_info).lastAimErrorPitch = ((6 - (*npc_info).stats.aim) as f32) * (*ctx.world).bg_state.rng.flrand(-1.0, 1.0);
            }
            (*npc_info).aimErrorDebounceTime = (*ctx.world).level.time + (*ctx.world).bg_state.rng.Q_irand(250, 2000);
        }

        let npc_client = (*npc).client as *mut gclient_t;

        if doYaw != QFALSE {
            // decay yaw diff
            let mut diff = AngleDelta((*npc_client).ps.viewangles[YAW], target_yaw);

            if diff != 0.0 {
                exact = QFALSE;

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

            (*ctx.world).globals.ucmd.angles[YAW] =
                ANGLE2SHORT(target_yaw + diff + error) - (*client).ps.delta_angles[YAW];
        }

        if doPitch != QFALSE {
            // decay pitch diff
            let mut diff = AngleDelta((*npc_client).ps.viewangles[PITCH], target_pitch);
            if diff != 0.0 {
                exact = QFALSE;

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

            (*ctx.world).globals.ucmd.angles[PITCH] =
                ANGLE2SHORT(target_pitch + diff + error) - (*client).ps.delta_angles[PITCH];
        }

        (*ctx.world).globals.ucmd.angles[ROLL] =
            ANGLE2SHORT((*npc_client).ps.viewangles[ROLL]) - (*client).ps.delta_angles[ROLL];

        exact
    }
}

/// Raven `NPC_UpdateShootAngles`.
///
/// Raven: FIXME: shoot angles either not set right or not used! `angles` is
/// read-only here (never written), so the fork-9 out-param reshape does not
/// apply — kept by-value.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:740-808`
pub fn NPC_UpdateShootAngles(
    ctx: GameContext<'_>,
    angles: vec3_t,
    doPitch: qboolean,
    doYaw: qboolean,
) {
    unsafe {
        let npc_info = (*ctx.world).globals.NPCInfo;

        let mut target_pitch: f32 = 0.0;
        let mut target_yaw: f32 = 0.0;

        if doPitch != QFALSE {
            target_pitch = angles[PITCH];
        }
        if doYaw != QFALSE {
            target_yaw = angles[YAW];
        }

        if doYaw != QFALSE {
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

        if doPitch != QFALSE {
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
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:818-847`
pub fn SetTeamNumbers(ctx: GameContext<'_>) {
    unsafe {
        for i in 0..4usize {
            (*ctx.world).teamNumbers[i] = 0;
            (*ctx.world).teamStrength[i] = 0;
        }

        for i in 0..1usize {
            let found = &mut (*ctx.world).g_entities[i] as *mut gentity_t;
            if !(*found).client.is_null() {
                if (*found).health > 0 {
                    let client = (*found).client as *mut gclient_t;
                    let team = (*client).playerTeam as usize;
                    (*ctx.world).teamNumbers[team] += 1;
                    (*ctx.world).teamStrength[team] += (*found).health;
                }
            }
        }

        for i in 0..4usize {
            // Raven: `floor( ((float)(teamStrength[i])) / ((float)(teamNumbers[i])) )`.
            let strength = (*ctx.world).teamStrength[i] as f32;
            let count = (*ctx.world).teamNumbers[i] as f32;
            (*ctx.world).teamStrength[i] = (strength / count).floor() as c_int;
        }
    }
}

/// Raven `G_ActivateBehavior`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:851-894`
pub fn G_ActivateBehavior(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    bset: c_int,
) -> qboolean {
    unsafe {
        if self_.is_null() {
            return QFALSE;
        }

        let bs_name = (*self_).behaviorSet[bset as usize];

        if !VALIDSTRING(bs_name) {
            return QFALSE;
        }

        let mut bSID: c_int = -1;
        if !(*self_).NPC.is_null() {
            bSID = GetIDForString(BSTable.as_ptr() as *mut stringID_table_t, bs_name);
        }

        if bSID > -1 {
            (*((*self_).NPC as *mut gNPC_t)).tempBehavior = bState_t::BS_DEFAULT;
            (*((*self_).NPC as *mut gNPC_t)).behaviorState =
                core::mem::transmute::<c_int, bState_t>(bSID);
        } else {
            // if (0) branch is dead code in oracle
            let script_path = format!(
                "{}/{}",
                cstr_to_str(Q3_SCRIPT_DIR.as_ptr()),
                cstr_to_str(bs_name)
            );
            trap::ICARUS_RunScript(ctx.engine, GIcarusRunscriptArgs::new(self_, cstr(&script_path)));
        }
        QTRUE
    }
}

/// Raven `NPC_SetBoneAngles`.
///
/// Raven: rww - special system for sync'ing bone angles between client and
/// server. The `#ifdef _XBOX` byte-index branch is dead on this build; the
/// plain `int *` branch below is the compiled one (per house ruling on
/// `_XBOX` branches).
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:906-995`
pub fn NPC_SetBoneAngles(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
    bone: *mut c_char,
    angles: vec3_t,
) {
    unsafe {
        let boneIndex = G_BoneIndex(ctx, bone as *const c_char);

        // Walk the 4 fixed bone-index/bone-angle slot pairs looking for
        // `boneIndex` (or the first free slot if not already present).
        let mut thebone: *mut c_int = &mut (*ent).s.boneIndex1;
        let mut boneVector: *mut vec3_t = &mut (*ent).s.boneAngles1;
        let mut firstFree: *mut c_int = core::ptr::null_mut();
        let mut freeBoneVec: *mut vec3_t = core::ptr::null_mut();
        let mut i = 0;
        let mut found = false;

        loop {
            if thebone.is_null() {
                break;
            }
            if *thebone == 0 && firstFree.is_null() {
                firstFree = thebone;
                freeBoneVec = boneVector;
            } else if *thebone != 0 {
                if *thebone == boneIndex {
                    found = true;
                    break;
                }
            }

            match i {
                0 => {
                    thebone = &mut (*ent).s.boneIndex2;
                    boneVector = &mut (*ent).s.boneAngles2;
                }
                1 => {
                    thebone = &mut (*ent).s.boneIndex3;
                    boneVector = &mut (*ent).s.boneAngles3;
                }
                2 => {
                    thebone = &mut (*ent).s.boneIndex4;
                    boneVector = &mut (*ent).s.boneAngles4;
                }
                _ => {
                    thebone = core::ptr::null_mut();
                    boneVector = core::ptr::null_mut();
                }
            }
            i += 1;
        }

        if thebone.is_null() {
            // didn't find it, create it
            if firstFree.is_null() {
                let msg = std::ffi::CString::new("WARNING: NPC has no free bone indexes\n").unwrap();
                crate::g_main::Com_Printf(msg.as_ptr());
                return;
            }
            thebone = firstFree;
            *thebone = boneIndex;
            boneVector = freeBoneVec;
        }

        // Copy the angles over the vector in the entitystate, so we can use
        // the corresponding index to set the bone angles on the client.
        *boneVector = angles;

        // Now set the angles on our server instance if we have one.
        if (*ent).ghoul2.is_null() {
            return;
        }

        let flags = BONE_ANGLES_POSTMULT;
        let up = POSITIVE_X as c_int;
        let right = NEGATIVE_Y as c_int;
        let forward = NEGATIVE_Z as c_int;

        //first 3 bits is forward, second 3 bits is right, third 3 bits is up
        (*ent).s.boneOrient = forward | (right << 3) | (up << 6);

        let bone_name = std::ffi::CStr::from_ptr(bone as *const c_char).to_owned();
        trap::G2API_SetBoneAngles(
            ctx.engine,
            GG2AngleoverrideArgs::new(
                (*ent).ghoul2,
                0,
                bone_name,
                &angles as *const vec3_t,
                flags,
                up,
                right,
                forward,
                core::ptr::null_mut(),
                100,
                (*ctx.world).level.time,
            ),
        );
    }
}

/// Raven `NPC_SetSurfaceOnOff`.
///
/// Raven: rww - and another method of automatically managing surface status
/// for the client and server at once.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1001-1039`
pub fn NPC_SetSurfaceOnOff(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
    surfaceName: *const c_char,
    surfaceFlags: c_int,
) {
    unsafe {
        let mut i: c_int = 0;
        let mut foundIt = QFALSE;

        while i < BG_NUM_TOGGLEABLE_SURFACES {
            if let Some(surf_name) = bgToggleableSurfaces[i as usize] {
                if Q_stricmp(surfaceName, surf_name.as_ptr()) == 0 {
                    foundIt = QTRUE;
                    break;
                }
            } else {
                break;
            }
            i += 1;
        }

        if foundIt == QFALSE {
            let msg = format!(
                "WARNING: Tried to toggle NPC surface that isn't in toggleable surface list ({})\n",
                cstr_to_str(surfaceName)
            );
            crate::g_main::Com_Printf(cstr(&msg).as_ptr());
            return;
        }

        if surfaceFlags == TURN_ON {
            (*ent).s.surfacesOn |= 1 << i;
            (*ent).s.surfacesOff &= !(1 << i);
        } else {
            (*ent).s.surfacesOn &= !(1 << i);
            (*ent).s.surfacesOff |= 1 << i;
        }

        if (*ent).ghoul2.is_null() {
            return;
        }

        trap::G2API_SetSurfaceOnOff(
            ctx.engine,
            GG2SetsurfaceonoffArgs::new((*ent).ghoul2, surfaceName, surfaceFlags),
        );
    }
}

/// Raven `NPC_SomeoneLookingAtMe`.
///
/// Raven: rww - cheap check to see if an armed client is looking in our
/// general direction.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1042-1067`
pub fn NPC_SomeoneLookingAtMe(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
) -> qboolean {
    unsafe {
        let mut i: usize = 0;
        while i < MAX_CLIENTS {
            let pEnt = &mut (*ctx.world).g_entities[i] as *mut gentity_t;

            let eligible = !pEnt.is_null()
                && (*pEnt).inuse != QFALSE
                && !(*pEnt).client.is_null()
                && {
                    let cl = (*pEnt).client as *mut gclient_t;
                    (*cl).sess.sessionTeam != TEAM_SPECTATOR
                        && ((*cl).ps.pm_flags & PMF_FOLLOW) == 0
                        && (*pEnt).s.weapon != WP_NONE
                };

            if eligible
                && trap::InPVS(
                    ctx.engine,
                    GInPvsArgs::new(
                        &(*ent).r.currentOrigin as *const vec3_t,
                        &(*pEnt).r.currentOrigin as *const vec3_t,
                    ),
                ) != 0
                //I'm in a 30 fov or so cone from this player.. that's enough I guess.
                && InFOV(ctx, pEnt, ent, 30, 30) != 0
            {
                return QTRUE;
            }

            i += 1;
        }

        QFALSE
    }
}

/// Raven `NPC_ClearLOS`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1069-1072`
pub fn NPC_ClearLOS(
    ctx: GameContext<'_>,
    start: vec3_t,
    end: vec3_t,
) -> qboolean {
    unsafe { G_ClearLOS(ctx, (*ctx.world).globals.NPC, start, end) }
}

/// Raven `NPC_ClearLOS5`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1073-1076`
pub fn NPC_ClearLOS5(
    ctx: GameContext<'_>,
    end: vec3_t,
) -> qboolean {
    unsafe { G_ClearLOS5(ctx, (*ctx.world).globals.NPC, end) }
}

/// Raven `NPC_ClearLOS4`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1077-1080`
pub fn NPC_ClearLOS4(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
) -> qboolean {
    unsafe { G_ClearLOS4(ctx, (*ctx.world).globals.NPC, ent) }
}

/// Raven `NPC_ClearLOS3`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1081-1084`
pub fn NPC_ClearLOS3(
    ctx: GameContext<'_>,
    start: vec3_t,
    ent: *mut gentity_t,
) -> qboolean {
    unsafe { G_ClearLOS3(ctx, (*ctx.world).globals.NPC, start, ent) }
}

/// Raven `NPC_ClearLOS2`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1085-1088`
pub fn NPC_ClearLOS2(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
    end: vec3_t,
) -> qboolean {
    unsafe { G_ClearLOS2(ctx, (*ctx.world).globals.NPC, ent, end) }
}

/// Raven `NPC_ValidEnemy`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1096-1187`
pub fn NPC_ValidEnemy(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
) -> qboolean {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let mut ent_team: c_int = TEAM_FREE as c_int;

        //Must be a valid pointer
        if ent.is_null() {
            return QFALSE;
        }

        //Must not be me
        if ent == npc {
            return QFALSE;
        }

        //Must not be deleted
        if (*ent).inuse == QFALSE {
            return QFALSE;
        }

        //Must be alive
        if (*ent).health <= 0 {
            return QFALSE;
        }

        //In case they're in notarget mode
        if ((*ent).flags & FL_NOTARGET) != 0 {
            return QFALSE;
        }

        let npc_client = (*npc).client as *mut gclient_t;

        //Must be an NPC
        if (*ent).client.is_null() {
            //	if ( ent->svFlags&SVF_NONNPC_ENEMY )
            if (*ent).s.eType != ET_NPC {
                //still potentially valid
                if (*ent).alliedTeam == (*npc_client).playerTeam as c_int {
                    return QFALSE;
                } else {
                    return QTRUE;
                }
            } else {
                return QFALSE;
            }
        } else if (*(((*ent).client) as *mut gclient_t)).sess.sessionTeam == TEAM_SPECTATOR {
            //don't go after spectators
            return QFALSE;
        }

        let ent_client = (*ent).client as *mut gclient_t;

        if !(*ent).NPC.is_null() && !(*ent).client.is_null() {
            ent_team = (*ent_client).playerTeam as c_int;
        } else if !(*ent).client.is_null() {
            if (*ctx.world).cvars.g_gametype.integer < GT_TEAM {
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
            return QFALSE;
        }

        let ent_enemy = match (*ent).enemy {
            Some(id) => &mut (*ctx.world).g_entities[id.index()] as *mut gentity_t,
            None => core::ptr::null_mut(),
        };

        //if haven't seen him in a while, give up
        if ent_team == (*npc_client).enemyTeam as c_int //simplest case: they're on my enemy team
            || ((*npc_client).enemyTeam as c_int == NPCTEAM_FREE && (*ent_client).NPC_class != (*npc_client).NPC_class) //I get mad at anyone and this guy isn't the same class as me
            || ((*ent_client).NPC_class == CLASS_WAMPA && !ent_enemy.is_null()) //a rampaging wampa
            || ((*ent_client).NPC_class == CLASS_RANCOR && !ent_enemy.is_null()) //a rampaging rancor
            || (ent_team == NPCTEAM_FREE
                && (*ent_client).enemyTeam as c_int == NPCTEAM_FREE
                && !ent_enemy.is_null()
                && !(*ent_enemy).client.is_null()
                && ({
                    let enemy_client = (*ent_enemy).client as *mut gclient_t;
                    (*enemy_client).playerTeam == (*npc_client).playerTeam
                        || ((*enemy_client).playerTeam as c_int != NPCTEAM_ENEMY
                            && (*npc_client).playerTeam as c_int == NPCTEAM_PLAYER)
                })) //enemy is a rampaging non-aligned creature who is attacking someone on our team or a non-enemy (this last condition is used only if we're a good guy - in effect, we protect the innocent)
        {
            return QTRUE;
        }

        QFALSE
    }
}

/// Raven `NPC_TargetVisible`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1195-1210`
pub fn NPC_TargetVisible(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
) -> qboolean {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;

        //Make sure we're in a valid range
        if DistanceSquared((*ent).r.currentOrigin, (*npc).r.currentOrigin)
            > (*npc_info).stats.visrange * (*npc_info).stats.visrange
        {
            return QFALSE;
        }

        //Check our FOV
        if InFOV(ctx, ent, npc, (*npc_info).stats.hfov, (*npc_info).stats.vfov) == QFALSE {
            return QFALSE;
        }

        //Check for sight
        if NPC_ClearLOS4(ctx, ent) == QFALSE {
            return QFALSE;
        }

        QTRUE
    }
}

/// Raven `NPC_FindNearestEnemy`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1246-1294`
pub fn NPC_FindNearestEnemy(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
) -> c_int {
    unsafe {
        let npc_info = (*ctx.world).globals.NPCInfo;

        let mut nearest_ent_id: c_int = -1;
        let mut nearest_dist = WORLD_SIZE * WORLD_SIZE;

        //Setup the bbox to search in
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        for i in 0..3 {
            mins[i] = (*ent).r.currentOrigin[i] - (*npc_info).stats.visrange;
            maxs[i] = (*ent).r.currentOrigin[i] + (*npc_info).stats.visrange;
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
            let rad_ent = &mut (*ctx.world).g_entities[iradius_ents[i as usize] as usize] as *mut gentity_t;

            //Don't consider self
            if rad_ent == ent {
                i += 1;
                continue;
            }

            //Must be valid
            if NPC_ValidEnemy(ctx, rad_ent) == QFALSE {
                i += 1;
                continue;
            }

            //Must be visible
            if NPC_TargetVisible(ctx, rad_ent) == QFALSE {
                i += 1;
                continue;
            }

            let distance = DistanceSquared((*ent).r.currentOrigin, (*rad_ent).r.currentOrigin);

            //Found one closer to us
            if distance < nearest_dist {
                nearest_ent_id = (*rad_ent).s.number;
                nearest_dist = distance;
            }

            i += 1;
        }

        nearest_ent_id
    }
}

/// Raven `NPC_PickEnemyExt`.
///
/// Raven: the "Hazard Team status" `NPC_FindPlayer` shortcut above is `/*
/// */`-commented out in the oracle — dead source, not transcribed.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1302-1348`
pub fn NPC_PickEnemyExt(
    ctx: GameContext<'_>,
    checkAlerts: qboolean,
) -> *mut gentity_t {
    unsafe {
        let npc = (*ctx.world).globals.NPC;

        //If we've asked for the closest enemy
        let ent_id = NPC_FindNearestEnemy(ctx, npc);

        //If we have a valid enemy, use it
        if ent_id >= 0 {
            return &mut (*ctx.world).g_entities[ent_id as usize] as *mut gentity_t;
        }

        if checkAlerts != QFALSE {
            let alert_event = NPC_CheckAlertEvents(ctx, QTRUE, QTRUE, -1, QTRUE, AEL_DISCOVERED as c_int);

            //There is an event to look at
            if alert_event >= 0 {
                let event = &mut (*ctx.world).level.alertEvents[alert_event as usize] as *mut alertEvent_t;

                //Don't pay attention to our own alerts
                if (*event).owner == npc {
                    return core::ptr::null_mut();
                }

                if ((*event).level as c_int) >= (AEL_DISCOVERED as c_int) {
                    //If it's the player, attack him
                    if (*event).owner == &mut (*ctx.world).g_entities[0] as *mut gentity_t {
                        return (*event).owner;
                    }

                    //If it's on our team, then take its enemy as well
                    let owner = (*event).owner;
                    if !(*owner).client.is_null() {
                        let owner_client = (*owner).client as *mut gclient_t;
                        let npc_client = (*npc).client as *mut gclient_t;
                        if (*owner_client).playerTeam == (*npc_client).playerTeam {
                            return (*owner).enemy;
                        }
                    }
                }
            }
        }

        core::ptr::null_mut()
    }
}

/// Raven `NPC_FindPlayer`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1356-1359`
pub fn NPC_FindPlayer(ctx: GameContext<'_>) -> qboolean {
    unsafe { NPC_TargetVisible(ctx, &mut (*ctx.world).g_entities[0] as *mut gentity_t) }
}

/// Raven `NPC_CheckPlayerDistance`.
///
/// Raven: the live body is a hardcoded `return qfalse; //MOOT in MP` — the
/// entire real implementation is `#if 0`-style commented out (dead in this
/// build); faithfully preserved as an always-false stub.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1367-1399`
fn NPC_CheckPlayerDistance() -> qboolean {
    QFALSE
}

/// Raven `NPC_FindEnemy`.
///
/// Raven: the `SVF_IGNORE_ENEMIES` branch is hardcoded `if (0)` dead source
/// in the oracle (`//rwwFIXMEFIXME: support for flag`) — kept as the
/// always-false condition it faithfully is.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1407-1461`
pub fn NPC_FindEnemy(
    ctx: GameContext<'_>,
    checkAlerts: qboolean,
) -> qboolean {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;

        //We're ignoring all enemies for now
        //if( NPC->svFlags & SVF_IGNORE_ENEMIES )
        if false {
            //rwwFIXMEFIXME: support for flag
            G_ClearEnemy(ctx, npc);
            return QFALSE;
        }

        //we can't pick up any enemies for now
        if (*npc_info).confusionTime > (*ctx.world).level.time {
            return QFALSE;
        }

        //Don't want a new enemy
        //rwwFIXMEFIXME: support for locked enemy

        //See if the player is closer than our current enemy
        if NPC_CheckPlayerDistance() != QFALSE {
            return QTRUE;
        }

        //Otherwise, turn off the flag
        //See if the player is closer than our current enemy
        let npc_client = (*npc).client as *mut gclient_t;
        if (*npc_client).NPC_class != CLASS_RANCOR
            && (*npc_client).NPC_class != CLASS_WAMPA
            && NPC_CheckPlayerDistance() != QFALSE
        {
            //rancors, wampas & sand creatures don't care if player is closer, they always go with closest
            return QTRUE;
        }

        //If we've gotten here alright, then our target it still valid
        if NPC_ValidEnemy(ctx, (*npc).enemy) != QFALSE {
            return QTRUE;
        }

        let newenemy = NPC_PickEnemyExt(ctx, checkAlerts);

        //if we found one, take it as the enemy
        if NPC_ValidEnemy(ctx, newenemy) != QFALSE {
            G_SetEnemy(ctx, npc, newenemy);
            return QTRUE;
        }

        QFALSE
    }
}

/// Raven `NPC_CheckEnemyExt`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1469-1483`
pub fn NPC_CheckEnemyExt(
    ctx: GameContext<'_>,
    checkAlerts: qboolean,
) -> qboolean {
    NPC_FindEnemy(ctx, checkAlerts)
}

/// Raven `NPC_FacePosition`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1491-1547`
pub fn NPC_FacePosition(
    ctx: GameContext<'_>,
    position: vec3_t,
    doPitch: qboolean,
) -> qboolean {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;
        let client = (*ctx.world).globals.client;

        let mut muzzle: vec3_t = [0.0; 3];
        let mut angles: vec3_t = [0.0; 3];
        let mut facing = QTRUE;

        let npc_client = (*npc).client as *mut gclient_t;

        //Get the positions
        if !npc_client.is_null()
            && ((*npc_client).NPC_class == CLASS_RANCOR || (*npc_client).NPC_class == CLASS_WAMPA)
        {
            CalcEntitySpot(ctx, npc as *const gentity_t, spot_t::SPOT_ORIGIN, &mut muzzle);
            muzzle[2] += (*npc).r.maxs[2] * 0.75;
        } else if !npc_client.is_null() && (*npc_client).NPC_class == CLASS_GALAKMECH {
            CalcEntitySpot(ctx, npc as *const gentity_t, spot_t::SPOT_WEAPON, &mut muzzle);
        } else {
            CalcEntitySpot(ctx, npc as *const gentity_t, spot_t::SPOT_HEAD_LEAN, &mut muzzle); //SPOT_HEAD
        }

        //Find the desired angles
        GetAnglesForDirection(muzzle, position, &mut angles);

        (*npc_info).desiredYaw = AngleNormalize360(angles[YAW]);
        (*npc_info).desiredPitch = AngleNormalize360(angles[PITCH]);

        if let Some(enemy_id) = (*npc).enemy {
            let enemy = &mut (*ctx.world).g_entities[enemy_id.index()] as *mut gentity_t;
            if !(*enemy).client.is_null() {
                let enemy_client = (*enemy).client as *mut gclient_t;
                if (*enemy_client).NPC_class == CLASS_ATST {
                    // FIXME: this is kind of dumb, but it was the easiest way to get it to look sort of ok
                    (*npc_info).desiredYaw +=
                        (*ctx.world).bg_state.rng.flrand(-5.0, 5.0) + (((*ctx.world).level.time as f32) * 0.004).sin() * 7.0;
                    (*npc_info).desiredPitch += (*ctx.world).bg_state.rng.flrand(-2.0, 2.0);
                }
            }
        }
        //Face that yaw
        NPC_UpdateAngles(ctx, QTRUE, QTRUE);

        //Find the delta between our goal and our current facing
        let yaw_delta = AngleNormalize360(
            (*npc_info).desiredYaw
                - SHORT2ANGLE((*ctx.world).globals.ucmd.angles[YAW] + (*client).ps.delta_angles[YAW]),
        );

        //See if we are facing properly
        if yaw_delta.abs() > VALID_ATTACK_CONE {
            facing = QFALSE;
        }

        if doPitch != QFALSE {
            //Find the delta between our goal and our current facing
            let current_angles =
                SHORT2ANGLE((*ctx.world).globals.ucmd.angles[PITCH] + (*client).ps.delta_angles[PITCH]);
            let pitch_delta = (*npc_info).desiredPitch - current_angles;

            //See if we are facing properly
            if pitch_delta.abs() > VALID_ATTACK_CONE {
                facing = QFALSE;
            }
        }

        facing
    }
}

/// Raven `NPC_FaceEntity`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1555-1563`
pub fn NPC_FaceEntity(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
    doPitch: qboolean,
) -> qboolean {
    let mut entPos: vec3_t = [0.0; 3];
    CalcEntitySpot(ctx, ent as *const gentity_t, spot_t::SPOT_HEAD_LEAN, &mut entPos);
    NPC_FacePosition(ctx, entPos, doPitch)
}

/// Raven `NPC_FaceEnemy`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1571-1580`
pub fn NPC_FaceEnemy(
    ctx: GameContext<'_>,
    doPitch: qboolean,
) -> qboolean {
    unsafe {
        let npc = (*ctx.world).globals.NPC;

        if npc.is_null() {
            return QFALSE;
        }

        if (*npc).enemy.is_none() {
            return QFALSE;
        }

        NPC_FaceEntity(ctx, (*npc).enemy, doPitch)
    }
}

/// Raven `NPC_CheckCanAttackExt`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1588-1603`
pub fn NPC_CheckCanAttackExt(ctx: GameContext<'_>) -> qboolean {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;

        //We don't want them to shoot
        if ((*npc_info).scriptFlags & SCF_DONT_FIRE) != 0 {
            return QFALSE;
        }

        //Turn to face
        if NPC_FaceEnemy(ctx, QTRUE) == QFALSE {
            return QFALSE;
        }

        //Must have a clear line of sight to the target
        if NPC_ClearShot(ctx, (*npc).enemy) == QFALSE {
            return QFALSE;
        }

        QTRUE
    }
}

/// Raven `NPC_ClearLookTarget`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1611-1625`
pub fn NPC_ClearLookTarget(
    self_: *mut gentity_t,
) {
    unsafe {
        if (*self_).client.is_null() {
            return;
        }
        let client = (*self_).client as *mut gclient_t;

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
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1632-1646`
pub fn NPC_SetLookTarget(
    self_: *mut gentity_t,
    entNum: c_int,
    clearTime: c_int,
) {
    unsafe {
        if (*self_).client.is_null() {
            return;
        }
        let client = (*self_).client as *mut gclient_t;

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
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1653-1679`
pub fn NPC_CheckLookTarget(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) -> qboolean {
    unsafe {
        if !(*self_).client.is_null() {
            let client = (*self_).client as *mut gclient_t;
            let lookTarget = (*client).renderInfo.lookTarget;

            if lookTarget >= 0 && lookTarget < ENTITYNUM_WORLD {
                //within valid range
                let target = &mut (*ctx.world).g_entities[lookTarget as usize] as *mut gentity_t;
                if (target.is_null()) || (*target).inuse == QFALSE {
                    //lookTarget not inuse or not valid anymore
                    NPC_ClearLookTarget(self_);
                } else if (*client).renderInfo.lookTargetClearTime != 0
                    && (*client).renderInfo.lookTargetClearTime < (*ctx.world).level.time
                {
                    //Time to clear lookTarget
                    NPC_ClearLookTarget(self_);
                } else if !(*target).client.is_null()
                    && !(*self_).enemy.is_none()
                    && target != (*self_).enemy
                {
                    //should always look at current enemy if engaged in
                    //battle... FIXME: this could override certain scripted
                    //lookTargets...???
                    NPC_ClearLookTarget(self_);
                } else {
                    return QTRUE;
                }
            }
        }

        QFALSE
    }
}

/// Raven `NPC_CheckCharmed`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1687-1705`
pub fn NPC_CheckCharmed(ctx: GameContext<'_>) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;

        if (*npc_info).charmedTime != 0
            && (*npc_info).charmedTime < (*ctx.world).level.time
            && !(*npc).client.is_null()
        {
            //we were charmed, set us back!
            let client = (*npc).client as *mut gclient_t;
            (*client).playerTeam = (*npc).genericValue1;
            (*client).enemyTeam = (*npc).genericValue2;
            (*npc).s.teamowner = (*npc).genericValue3;

            (*client).leader = None;
            if (*npc_info).tempBehavior == bState_t::BS_FOLLOW_LEADER {
                (*npc_info).tempBehavior = bState_t::BS_DEFAULT;
            }
            G_ClearEnemy(ctx, npc);
            (*npc_info).charmedTime = 0;
            //say something to let player know you've snapped out of it
            G_AddVoiceEvent(
                ctx,
                npc,
                (*ctx.world).bg_state.rng.Q_irand(entity_event_t::EV_CONFUSE1 as c_int, entity_event_t::EV_CONFUSE3 as c_int),
                2000,
            );
        }
    }
}

/// Raven `G_GetBoltPosition`.
///
/// Fork-9 reshape: `pos` is guarded by `if (pos)` in the oracle (the
/// AngleVectors NULL-able idiom), so it becomes `Option<&mut vec3_t>`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1707-1740`
pub fn G_GetBoltPosition(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    boltIndex: c_int,
    pos: Option<&mut vec3_t>,
    modelIndex: c_int,
) {
    unsafe {
        if self_.is_null() || (*self_).inuse == QFALSE {
            return;
        }

        let mut angles: vec3_t = [0.0; 3];
        if !(*self_).client.is_null() {
            angles[0] = 0.0;
            angles[1] = (*(*self_).client).ps.viewangles[YAW];
            angles[2] = 0.0;
        } else {
            angles[0] = 0.0;
            angles[1] = (*self_).r.currentAngles[YAW];
            angles[2] = 0.0;
        }

        if (*self_).ghoul2.is_null() {
            return;
        }

        let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
        trap::G2API_GetBoltMatrix(
            ctx.engine,
            GG2GetboltArgs::new(
                (*self_).ghoul2,
                modelIndex,
                boltIndex,
                &mut boltMatrix as *mut mdxaBone_t,
                &angles as *const vec3_t,
                &(*self_).r.currentOrigin as *const vec3_t,
                (*ctx.world).level.time,
                core::ptr::null_mut(),
                &(*self_).modelScale as *const vec3_t,
            ),
        );

        if let Some(pos_ref) = pos {
            let mut result: vec3_t = [0.0; 3];
            BG_GiveMeVectorFromMatrix(&boltMatrix as *const mdxaBone_t, ORIGIN, &mut result);
            _VectorCopy(result, pos_ref);
        }
    }
}

/// Raven `NPC_EntRangeFromBolt`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1742-1754`
pub fn NPC_EntRangeFromBolt(
    ctx: GameContext<'_>,
    targEnt: *mut gentity_t,
    boltIndex: c_int,
) -> f32 {
    unsafe {
        let npc = (*ctx.world).globals.NPC;

        if targEnt.is_null() {
            return Q3_INFINITE;
        }

        let mut org: vec3_t = [0.0; 3];
        G_GetBoltPosition(ctx, npc, boltIndex, Some(&mut org), 0);

        Distance((*targEnt).r.currentOrigin, org)
    }
}

/// Raven `NPC_EnemyRangeFromBolt`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1756-1759`
pub fn NPC_EnemyRangeFromBolt(
    ctx: GameContext<'_>,
    boltIndex: c_int,
) -> f32 {
    unsafe { NPC_EntRangeFromBolt(ctx, (*(*ctx.world).globals.NPC).enemy, boltIndex) }
}

/// Raven `NPC_GetEntsNearBolt`.
///
/// Fork-9 reshape: `boltOrg` is written unconditionally
/// (`VectorCopy(org, boltOrg)`), so it becomes the non-nullable
/// `&mut vec3_t`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1761-1782`
pub fn NPC_GetEntsNearBolt(
    ctx: GameContext<'_>,
    radiusEnts: *mut c_int,
    radius: f32,
    boltIndex: c_int,
    boltOrg: &mut vec3_t,
) -> c_int {
    unsafe {
        let npc = (*ctx.world).globals.NPC;

        //get my handRBolt's position
        let mut org: vec3_t = [0.0; 3];

        G_GetBoltPosition(ctx, npc, boltIndex, Some(&mut org), 0);

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
            GEntitiesInBoxArgs::new(&mins as *const vec3_t, &maxs as *const vec3_t, radiusEnts, 128),
        )
    }
}
