// PORT-COMPLETE: NPC_behavior.c
//! FAITHFUL port of `oracle/codemp/game/NPC_behavior.c`.
//!
//! Landed from the `fnskel.py` signature skeleton; the pass-3 mega-pass fills
//! every remaining body against the settled fork rulings (ctx threading,
//! `Option<EntityId>` stored fields, bg/game state split). File-scope AI
//! globals (`NPC`, `NPCInfo`, `ucmd`, `level`, `g_entities`, `enemyVisibility`,
//! `showBBoxes`) reach through `ctx.world`/`ctx.world.globals` per ruling
//! 8/12.
//!
//! Safe-state migration **Stage 1**: entity-pointer params are `EntityId` /
//! `Option<EntityId>` handles (§B5), not raw `gentity_t*`; ctx-free leaf helpers
//! take `&mut`/`&gentity_t`. Bodies re-derive the raw pointers verbatim at the
//! top (`// STAGE-1:` markers) — Stage-2 debt. Callers bridge at the boundary
//! via `ctx.entity_id_of(ptr)`.
#![allow(non_snake_case, unused, clippy::all)]

use crate::ent_fn_enums::EntThink;
use crate::g_nav::NPC_SetMoveGoal;
use crate::g_nav::{NAV_FindClosestWaypointForEnt, NAV_GetNearestNode};
use crate::g_timer::{TIMER_Done, TIMER_Set};
use crate::g_utils::{G_FreeEntity, G_UseTargets2};
use crate::npc::jump_state_t::jumpState_t::{
    self, JS_CROUCHING, JS_FACING, JS_JUMPING, JS_LANDING, JS_WAITING,
};
use crate::npc::spot_t::spot_t;
use crate::npc::visibility_t::visibility_t;
use crate::npc_c::{NPC_SetAnim, RestoreNPCGlobals, SaveNPCGlobals, SetNPCGlobals};
use crate::prelude::*;
use crate::q_math::{
    _DotProduct, _VectorAdd, _VectorCopy, _VectorMA, _VectorScale, _VectorSubtract, vec3_origin,
    vectoangles, AngleDelta, AngleNormalize360, AngleVectors, VectorCompare, VectorLength,
    VectorLengthSquared, VectorNormalize,
};
use crate::teams::npcteam::NPCTEAM_ENEMY;
use crate::NPC_combat::{
    G_ClearEnemy, G_SetEnemy, NPC_AimAdjust, NPC_CheckAttack, NPC_CheckEnemy,
    NPC_CheckGetNewWeapon, NPC_EnemyTooFar, NPC_FindCombatPoint, NPC_SetCombatPoint,
    NPC_ShotEntity, ValidEnemy, WeaponThink,
};
use crate::NPC_goal::{NPC_ClearGoal, UpdateGoal};
use crate::NPC_move::{NPC_MoveToGoal, NPC_SlideMoveToGoal};
use crate::NPC_senses::{InFOV, NPC_CheckAlertEvents, NPC_CheckVisibility, NPC_GetHFOVPercentage};
use crate::NPC_sounds::G_AddVoiceEvent;
use crate::NPC_utils::{
    CalcEntitySpot, G_ActivateBehavior, NPC_AimWiggle, NPC_CheckEnemyExt, NPC_ClearLOS4,
    NPC_FaceEnemy, NPC_SomeoneLookingAtMe, NPC_UpdateAngles, NPC_UpdateFiringAngles,
    NPC_UpdateShootAngles,
};
use mp_abi::game::syscalls::G_ICARUS_ISINITIALIZED::GIcarusIsinitializedArgs;
use mp_abi::game::syscalls::G_ICARUS_TASKIDCOMPLETE::GIcarusTaskidcompleteArgs;
use mp_abi::game::syscalls::G_ICARUS_TASKIDPENDING::GIcarusTaskidpendingArgs;
use mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs;
use mp_abi::game::syscalls::G_NAV_GETNODEEDGE::GNavGetnodeedgeArgs;
use mp_abi::game::syscalls::G_NAV_GETNODENUMEDGES::GNavGetnodenumedgesArgs;
use mp_abi::game::syscalls::G_NAV_GETNODEPOSITION::GNavGetnodepositionArgs;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use mp_bg::bg_misc::vectoyaw;
use mp_bg::bg_panimate::PM_InKnockDown;
use mp_qshared::shared::MASK_SHOT;

// Combat point search flags: `crate::npc::combat_point_flags`
// (`b_local.h:244-260`).

// Raven `MIN_ANGLE_ERROR` (`b_local.h`, the facing gate in `NPC_BSJump`).
// Source: `oracle/codemp/game/b_local.h:29`
pub const MIN_ANGLE_ERROR: f32 = 0.01;
// Raven `APEX_HEIGHT` (`NPC_behavior.c` #define, the jump-parabola apex).
// Source: `oracle/codemp/game/NPC_behavior.c:730`
pub const APEX_HEIGHT: f32 = 200.0;

/// Raven `NPC_BSAdvanceFight`.
///
/// Source: `oracle/codemp/game/NPC_behavior.c:29-183`
pub fn NPC_BSAdvanceFight(ctx: &mut GameContext) {
    let NPC = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(NPC).unwrap();
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let NPCInfo = ctx.world.globals.NPCInfo;
    unsafe {
        // Make sure we're still headed where we want to capture.
        if let Some(captureGoal) = (*NPCInfo).captureGoal {
            let cap_origin = ctx.world.entity(captureGoal).r.currentOrigin;
            NPC_SetMoveGoal(ctx, npc_id, cap_origin, 16, qtrue, -1, None);
            (*NPCInfo).goalTime = ctx.world.level.time + 100000;
        }

        NPC_CheckEnemy(ctx, qtrue, qfalse, qtrue);

        if let Some(enemy_id) = ctx.world.entity(npc_id).enemy {
            let mut delta = [0.0f32; 3];
            let mut forward = [0.0f32; 3];
            let mut angleToEnemy = [0.0f32; 3];
            let mut hitspot = [0.0f32; 3];
            let mut muzzle = [0.0f32; 3];
            let mut diff = [0.0f32; 3];
            let mut enemy_org = [0.0f32; 3];
            let mut enemy_head = [0.0f32; 3];
            let distanceToEnemy: f32;
            let mut attack_ok = qfalse;
            let mut dead_on = qfalse;
            let mut attack_scale: f32 = 1.0;
            let max_aim_off: f32 = 64.0;

            let enemy_absmin = ctx.world.entity(enemy_id).r.absmin;
            let enemy_maxs = ctx.world.entity(enemy_id).r.maxs;
            _VectorMA(enemy_absmin, 0.5, enemy_maxs, &mut enemy_org);
            CalcEntitySpot(ctx, Some(npc_id), spot_t::SPOT_WEAPON, &mut muzzle);

            _VectorSubtract(enemy_org, muzzle, &mut delta);
            vectoangles(delta, &mut angleToEnemy);
            distanceToEnemy = VectorNormalize(&mut delta);

            if NPC_EnemyTooFar(
                ctx,
                Some(enemy_id),
                distanceToEnemy * distanceToEnemy,
                qtrue,
            ) == qfalse
            {
                attack_ok = qtrue;
            }

            if attack_ok != qfalse {
                NPC_UpdateShootAngles(ctx, angleToEnemy, qfalse, qtrue);

                (*NPCInfo).enemyLastVisibility = ctx.world.globals.enemyVisibility;
                let vis = NPC_CheckVisibility(ctx, Some(enemy_id), CHECK_FOV);
                ctx.world.globals.enemyVisibility = vis;

                if vis == visibility_t::VIS_FOV {
                    attack_ok = qtrue;
                    CalcEntitySpot(ctx, Some(enemy_id), spot_t::SPOT_HEAD, &mut enemy_head);

                    if attack_ok != qfalse {
                        let mut tr: trace_t = core::mem::zeroed();
                        let npc_number = ctx.world.entity(npc_id).s.number;
                        trap::Trace(
                            ctx.engine,
                            GTraceArgs::new(
                                &mut tr as *mut trace_t,
                                &muzzle as *const vec3_t,
                                core::ptr::null(),
                                core::ptr::null(),
                                &enemy_org as *const vec3_t,
                                npc_number,
                                MASK_SHOT,
                            ),
                        );
                        // Trace result index maps directly to the arena slot
                        // (ENTITYNUM_WORLD/NONE are valid slots), so construct the
                        // handle raw rather than via `from_num` (which nulls NONE).
                        let mut trace_id = EntityId(tr.entityNum as u32);
                        // FLAG: pool/world clients; deref raw via safe borrow (trap 2b).
                        let npc_client = ctx.world.entity(npc_id).client;
                        let trace_client = ctx.world.entity(trace_id).client;
                        let trace_is_enemy = trace_id == enemy_id;
                        if !trace_is_enemy
                            && (trace_client.is_null()
                                || (*npc_client).enemyTeam == 0
                                || (*npc_client).enemyTeam != (*trace_client).playerTeam)
                        {
                            // No, so shoot for the head.
                            attack_scale *= 0.75;
                            let npc_number = ctx.world.entity(npc_id).s.number;
                            trap::Trace(
                                ctx.engine,
                                GTraceArgs::new(
                                    &mut tr as *mut trace_t,
                                    &muzzle as *const vec3_t,
                                    core::ptr::null(),
                                    core::ptr::null(),
                                    &enemy_head as *const vec3_t,
                                    npc_number,
                                    MASK_SHOT,
                                ),
                            );
                            trace_id = EntityId(tr.entityNum as u32);
                        }

                        _VectorCopy(tr.endpos, &mut hitspot);

                        let trace_client = ctx.world.entity(trace_id).client;
                        let trace_is_enemy = trace_id == enemy_id;
                        if trace_is_enemy
                            || (!trace_client.is_null()
                                && (*npc_client).enemyTeam != 0
                                && (*npc_client).enemyTeam == (*trace_client).playerTeam)
                        {
                            dead_on = qtrue;
                        } else {
                            attack_scale *= 0.5;
                            if (*npc_client).playerTeam != 0 {
                                if !trace_client.is_null() && (*trace_client).playerTeam != 0 {
                                    if (*npc_client).playerTeam == (*trace_client).playerTeam {
                                        // Don't shoot our own team.
                                        attack_ok = qfalse;
                                    }
                                }
                            }
                        }

                        if attack_ok != qfalse {
                            // Adjust pitch aim.
                            _VectorSubtract(hitspot, muzzle, &mut delta);
                            vectoangles(delta, &mut angleToEnemy);
                            (*NPCInfo).desiredPitch = angleToEnemy[0]; // PITCH
                            NPC_UpdateShootAngles(ctx, angleToEnemy, qtrue, qfalse);

                            if dead_on == qfalse {
                                // Suppressing fire.
                                AngleVectors(
                                    (*NPCInfo).shootAngles,
                                    Some(&mut forward),
                                    None,
                                    None,
                                );
                                _VectorMA(muzzle, distanceToEnemy, forward, &mut hitspot);
                                _VectorSubtract(hitspot, enemy_org, &mut diff);
                                let mut aim_off = VectorLength(diff);
                                // Oracle uses the `random()` macro (randSeed stream), not flrand.
                                // Source: `oracle/codemp/game/NPC_behavior.c:140,146`
                                if aim_off > ctx.world.bg_state.rng.random() * max_aim_off {
                                    attack_scale *= 0.75;
                                    _VectorSubtract(hitspot, enemy_head, &mut diff);
                                    aim_off = VectorLength(diff);
                                    if aim_off > ctx.world.bg_state.rng.random() * max_aim_off {
                                        attack_ok = qfalse;
                                    }
                                }
                                attack_scale *= (max_aim_off - aim_off + 1.0) / max_aim_off;
                            }
                        }
                    }
                }
            }

            if attack_ok != qfalse {
                if NPC_CheckAttack(ctx, attack_scale) != qfalse {
                    ctx.world.globals.enemyVisibility = visibility_t::VIS_SHOOT;
                    WeaponThink(ctx, qtrue);
                } else {
                    attack_ok = qfalse;
                }
            }
        } else {
            // FLAG: pool client; deref raw via safe borrow (trap 2b).
            let client = ctx.world.entity(npc_id).client;
            let viewangles = (*client).ps.viewangles;
            NPC_UpdateShootAngles(ctx, viewangles, qtrue, qtrue);
        }

        if ctx.world.globals.ucmd.forwardmove == 0 && ctx.world.globals.ucmd.rightmove == 0 {
            // We reached our captureGoal.
            let npc_number = ctx.world.entity(npc_id).s.number;
            if trap::ICARUS_IsInitialized(ctx.engine, GIcarusIsinitializedArgs::new(npc_number))
                != 0
            {
                trap::ICARUS_TaskIDComplete(
                    ctx.engine,
                    GIcarusTaskidcompleteArgs::new(NPC.cast(), taskID_t::TID_BSTATE as c_int),
                );
            }
        }
    }
}

/// Raven `Disappear`.
///
/// Source: `oracle/codemp/game/NPC_behavior.c:185-191`
pub fn Disappear(self_: &mut gentity_t) {
    // ClientDisconnect(self); (Raven: commented out)
    self_.s.eFlags |= EF_NODRAW;
    self_.think = FnId::NONE;
    self_.nextthink = -1;
}

/// Raven `BeamOut`.
///
/// Source: `oracle/codemp/game/NPC_behavior.c:194-211`
pub fn BeamOut(ctx: &mut GameContext, self_: EntityId) {
    // fixme: doesn't actually go away!
    let level_time = ctx.world.level.time;
    ctx.world.entity_mut(self_).nextthink = level_time + 1500;
    // fn-ptr field -> fn-ID enum (shape_mismatch; see the settled rule).
    ctx.world.entity_mut(self_).think = Some(EntThink::Disappear).into();
    // FLAG: pool client; deref raw via safe borrow (trap 2b).
    let client = ctx.world.entity(self_).client;
    unsafe {
        (*client).squadname = core::ptr::null_mut();
        (*client).playerTeam = TEAM_FREE;
    }
    ctx.world.entity_mut(self_).s.teamowner = TEAM_FREE as c_int;
    //self->r.svFlags |= SVF_BEAMING; //this appears unused in SP as well
}

/// Raven `NPC_BSCinematic`.
///
/// Source: `oracle/codemp/game/NPC_behavior.c:213-244`
pub fn NPC_BSCinematic(ctx: &mut GameContext) {
    let NPC = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(NPC).unwrap();
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let NPCInfo = ctx.world.globals.NPCInfo;
    unsafe {
        if (*NPCInfo).scriptFlags & SCF_FIRE_WEAPON != 0 {
            WeaponThink(ctx, qtrue);
        }

        if !UpdateGoal(ctx).is_null() {
            // Have a goalEntity.
            NPC_MoveToGoal(ctx, qtrue);
        }

        if let Some(watch_id) = (*NPCInfo).watchTarget {
            // Have an entity which we want to keep facing.
            let mut eyes = [0.0f32; 3];
            let mut viewSpot = [0.0f32; 3];
            let mut viewvec = [0.0f32; 3];
            let mut viewangles = [0.0f32; 3];

            CalcEntitySpot(ctx, Some(npc_id), spot_t::SPOT_HEAD_LEAN, &mut eyes);
            CalcEntitySpot(ctx, Some(watch_id), spot_t::SPOT_HEAD_LEAN, &mut viewSpot);

            _VectorSubtract(viewSpot, eyes, &mut viewvec);

            vectoangles(viewvec, &mut viewangles);

            (*NPCInfo).lockedDesiredYaw = viewangles[1];
            (*NPCInfo).desiredYaw = viewangles[1];
            (*NPCInfo).lockedDesiredPitch = viewangles[0];
            (*NPCInfo).desiredPitch = viewangles[0];
        }

        NPC_UpdateAngles(ctx, qtrue, qtrue);
    }
}

/// Raven `NPC_BSWait`.
///
/// Source: `oracle/codemp/game/NPC_behavior.c:246-249`
pub fn NPC_BSWait(ctx: &mut GameContext) {
    NPC_UpdateAngles(ctx, qtrue, qtrue);
}

/// Raven `NPC_BSInvestigate`.
///
/// Raven: entire body is `/* ... */`-commented dead code (kept for reference)
/// — the live function is a no-op. Ported faithfully as a no-op.
/// Source: `oracle/codemp/game/NPC_behavior.c:252-407`
pub fn NPC_BSInvestigate() {
    // Raven's body is entirely commented out; this is a genuine no-op.
}

/// Raven `NPC_CheckInvestigate`.
///
/// Source: `oracle/codemp/game/NPC_behavior.c:409-494`
pub fn NPC_CheckInvestigate(ctx: &mut GameContext, alertEventNum: c_int) -> qboolean {
    let NPC = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(NPC).unwrap();
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let NPCInfo = ctx.world.globals.NPCInfo;
    unsafe {
        let owner = ctx.world.level.alertEvents[alertEventNum as usize].owner;
        let invAdd = ctx.world.level.alertEvents[alertEventNum as usize].level as c_int;
        let soundRad = ctx.world.level.alertEvents[alertEventNum as usize].radius;
        let earshot = (*NPCInfo).stats.earshot;

        let mut soundPos = [0.0f32; 3];
        _VectorCopy(
            ctx.world.level.alertEvents[alertEventNum as usize].position,
            &mut soundPos,
        );

        // NOTE: Trying to preserve previous investigation behavior.
        if owner.is_null() {
            return qfalse;
        }

        // `owner` is a raw alert-event pointer; recover its handle at the seam.
        let owner_id = ctx.entity_id_of(owner);
        let owner_ent = owner_id.unwrap();
        if ctx.world.entity(owner_ent).s.eType != entityType_t::ET_PLAYER as c_int
            && ctx.world.entity(owner_ent).s.eType != entityType_t::ET_NPC as c_int
            && owner_id == (*NPCInfo).goalEntity
        {
            return qfalse;
        }

        if ctx.world.entity(owner_ent).s.eFlags & EF_NODRAW != 0 {
            return qfalse;
        }

        if ctx.world.entity(owner_ent).flags & FL_NOTARGET != 0 {
            return qfalse;
        }

        if soundRad < earshot {
            return qfalse;
        }

        let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
        if trap::InPVS(
            ctx.engine,
            GInPvsArgs::new(&soundPos as *const _, &npc_origin as *const _),
        ) == 0
        {
            // Can hear through doors?
            return qfalse;
        }

        // FLAG: pool clients; deref raw via safe borrow (trap 2b).
        let owner_client = ctx.world.entity(owner_ent).client;
        let npc_client = ctx.world.entity(npc_id).client;
        if !owner_client.is_null()
            && (*owner_client).playerTeam != 0
            && (*npc_client).playerTeam != 0
            && (*owner_client).playerTeam != (*npc_client).playerTeam
        {
            if (*NPCInfo).investigateCount as f32 >= ((*NPCInfo).stats.vigilance * 200.0) {
                // If investigateCount == 10, just take it as enemy and go.
                if ValidEnemy(ctx, owner_id) != qfalse {
                    G_SetEnemy(ctx, npc_id, owner_id);
                    (*NPCInfo).goalEntity = ctx.world.entity(npc_id).enemy;
                    (*NPCInfo).goalRadius = 12;
                    (*NPCInfo).behaviorState = BS_HUNT_AND_KILL;
                    return qtrue;
                }
            } else {
                (*NPCInfo).investigateCount += invAdd;
            }
            // Run awakescript.
            G_ActivateBehavior(ctx, Some(npc_id), BSET_AWAKE as c_int);

            (*NPCInfo).eventOwner = owner_id;
            _VectorCopy(soundPos, &mut (*NPCInfo).investigateGoal);
            if (*NPCInfo).investigateCount > 20 {
                (*NPCInfo).investigateDebounceTime = ctx.world.level.time + 10000;
            } else {
                (*NPCInfo).investigateDebounceTime =
                    ctx.world.level.time + (*NPCInfo).investigateCount * 500;
            }
            (*NPCInfo).tempBehavior = BS_INVESTIGATE;
            return qtrue;
        }

        qfalse
    }
}

/// Raven `NPC_BSSleep`.
///
/// Source: `oracle/codemp/game/NPC_behavior.c:500-521`
pub fn NPC_BSSleep(ctx: &mut GameContext) {
    let NPC = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(NPC).unwrap();

    let alertEvent = NPC_CheckAlertEvents(ctx, qtrue, qfalse, -1, qfalse, AEL_MINOR as c_int);

    // There is an event to look at.
    if alertEvent >= 0 {
        G_ActivateBehavior(ctx, Some(npc_id), BSET_AWAKE as c_int);
        return;
    }
}

/// Raven `NPC_BSFollowLeader`.
///
/// Source: `oracle/codemp/game/NPC_behavior.c:524-729`
pub fn NPC_BSFollowLeader(ctx: &mut GameContext) {
    let NPC = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(NPC).unwrap();
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let NPCInfo = ctx.world.globals.NPCInfo;
    // FLAG: pool client; deref raw via safe borrow (trap 2b).
    let npc_client = ctx.world.entity(npc_id).client;
    unsafe {
        let leader_id = (*npc_client).leader;
        let Some(leader_ent) = leader_id else {
            // Stand guard until we find an enemy.
            if (*NPCInfo).tempBehavior == BS_HUNT_AND_KILL {
                (*NPCInfo).tempBehavior = BS_DEFAULT;
            } else {
                (*NPCInfo).tempBehavior = BS_STAND_GUARD;
                crate::NPC_AI_Default::NPC_BSStandGuard(ctx);
            }
            return;
        };

        if ctx.world.entity(npc_id).enemy.is_none() {
            // No enemy, find one.
            NPC_CheckEnemy(
                ctx,
                if (*NPCInfo).confusionTime < ctx.world.level.time {
                    qtrue
                } else {
                    qfalse
                },
                qfalse,
                qtrue,
            );
            if ctx.world.entity(npc_id).enemy.is_some() {
                (*NPCInfo).enemyCheckDebounceTime =
                    ctx.world.level.time + ctx.world.bg_state.rng.Q_irand(3000, 10000);
            } else {
                if (*NPCInfo).scriptFlags & SCF_IGNORE_ALERTS == 0 {
                    let eventID =
                        NPC_CheckAlertEvents(ctx, qtrue, qtrue, -1, qfalse, AEL_MINOR as c_int);
                    if ctx.world.level.alertEvents[eventID as usize].level as c_int
                        >= AEL_SUSPICIOUS as c_int
                        && ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0
                    {
                        (*NPCInfo).lastAlertID = ctx.world.level.alertEvents[eventID as usize].ID;
                        let ev_owner = ctx.world.level.alertEvents[eventID as usize].owner;
                        let ev_owner_id = ctx.entity_id_of(ev_owner);
                        // FLAG: pool client; deref raw via safe borrow (trap 2b).
                        let ev_owner_client = match ev_owner_id {
                            Some(id) => ctx.world.entity(id).client,
                            None => core::ptr::null_mut(),
                        };
                        if ev_owner_id.is_none()
                            || ev_owner_client.is_null()
                            || ctx.world.entity(ev_owner_id.unwrap()).health <= 0
                            || (*ev_owner_client).playerTeam != (*npc_client).enemyTeam
                        {
                            // Not an enemy.
                        } else {
                            G_SetEnemy(ctx, npc_id, ev_owner_id);
                            (*NPCInfo).enemyCheckDebounceTime =
                                ctx.world.level.time + ctx.world.bg_state.rng.Q_irand(3000, 10000);
                            (*NPCInfo).enemyLastSeenTime = ctx.world.level.time;
                            let s = cstr("attackDelay");
                            let atk_delay = ctx.world.bg_state.rng.Q_irand(500, 1000);
                            TIMER_Set(ctx, Some(npc_id), s.as_ptr(), atk_delay);
                        }
                    }
                }
            }
            if ctx.world.entity(npc_id).enemy.is_none() {
                // FLAG: pool client (unused, kept faithful); deref raw (trap 2b).
                let l_client = ctx.world.entity(leader_ent).client;
                let leader_enemy = ctx.world.entity(leader_ent).enemy;
                if !leader_id.is_none() && leader_enemy.is_some() && leader_enemy != Some(npc_id) {
                    let l_enemy_id = leader_enemy.unwrap();
                    // FLAG: pool client; deref raw via safe borrow (trap 2b).
                    let l_enemy_client = ctx.world.entity(l_enemy_id).client;
                    let allied_ok = !l_enemy_client.is_null()
                        && (*l_enemy_client).playerTeam == (*npc_client).enemyTeam;
                    if allied_ok && ctx.world.entity(l_enemy_id).health > 0 {
                        G_SetEnemy(ctx, npc_id, Some(l_enemy_id));
                        (*NPCInfo).enemyCheckDebounceTime =
                            ctx.world.level.time + ctx.world.bg_state.rng.Q_irand(3000, 10000);
                        (*NPCInfo).enemyLastSeenTime = ctx.world.level.time;
                    }
                }
            }
        } else {
            let enemy_id = ctx.world.entity(npc_id).enemy.unwrap();
            if ctx.world.entity(enemy_id).health <= 0
                || (ctx.world.entity(enemy_id).flags & FL_NOTARGET) != 0
            {
                G_ClearEnemy(ctx, npc_id);
                if (*NPCInfo).enemyCheckDebounceTime > ctx.world.level.time + 1000 {
                    (*NPCInfo).enemyCheckDebounceTime =
                        ctx.world.level.time + ctx.world.bg_state.rng.Q_irand(1000, 2000);
                }
            } else if (*npc_client).ps.weapon != 0
                && (*NPCInfo).enemyCheckDebounceTime < ctx.world.level.time
            {
                NPC_CheckEnemy(
                    ctx,
                    if (*NPCInfo).confusionTime < ctx.world.level.time
                        || (*NPCInfo).tempBehavior != BS_FOLLOW_LEADER
                    {
                        qtrue
                    } else {
                        qfalse
                    },
                    qfalse,
                    qtrue,
                );
            }
        }

        if ctx.world.entity(npc_id).enemy.is_some() && (*npc_client).ps.weapon != 0 {
            // If have an enemy, face him and fire.
            let enemy_id = ctx.world.entity(npc_id).enemy.unwrap();
            if (*npc_client).ps.weapon == WP_SABER as c_int {
                if (*NPCInfo).tempBehavior != BS_FOLLOW_LEADER {
                    (*NPCInfo).tempBehavior = BS_HUNT_AND_KILL;
                    NPC_UpdateAngles(ctx, qtrue, qtrue);
                    return;
                }
            }

            let vis = NPC_CheckVisibility(ctx, Some(enemy_id), CHECK_FOV | CHECK_SHOOT);
            ctx.world.globals.enemyVisibility = vis;
            if (vis as c_int) > (visibility_t::VIS_PVS as c_int) {
                // Face.
                let mut enemy_org = [0.0f32; 3];
                let mut muzzle = [0.0f32; 3];
                let mut delta = [0.0f32; 3];
                let mut angleToEnemy = [0.0f32; 3];

                CalcEntitySpot(ctx, Some(enemy_id), spot_t::SPOT_HEAD, &mut enemy_org);
                NPC_AimWiggle(ctx, &mut enemy_org);

                CalcEntitySpot(ctx, Some(npc_id), spot_t::SPOT_WEAPON, &mut muzzle);

                _VectorSubtract(enemy_org, muzzle, &mut delta);
                vectoangles(delta, &mut angleToEnemy);
                let distanceToEnemy = VectorNormalize(&mut delta);

                (*NPCInfo).desiredYaw = angleToEnemy[1];
                (*NPCInfo).desiredPitch = angleToEnemy[0];
                NPC_UpdateFiringAngles(ctx, qtrue, qtrue);

                if (vis as c_int) >= (visibility_t::VIS_SHOOT as c_int) {
                    NPC_AimAdjust(ctx, 2);
                    let enemy_origin = ctx.world.entity(enemy_id).r.currentOrigin;
                    let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
                    let viewangles = (*npc_client).ps.viewangles;
                    if NPC_GetHFOVPercentage(
                        enemy_origin,
                        npc_origin,
                        viewangles,
                        (*NPCInfo).stats.hfov as f32,
                    ) > 0.6
                        && NPC_GetHFOVPercentage(
                            enemy_origin,
                            npc_origin,
                            viewangles,
                            (*NPCInfo).stats.vfov as f32,
                        ) > 0.5
                    {
                        WeaponThink(ctx, qtrue);
                    }
                } else {
                    NPC_AimAdjust(ctx, 1);
                }
            } else {
                NPC_AimAdjust(ctx, -1);
            }
        } else {
            let mut head = [0.0f32; 3];
            let mut leaderHead = [0.0f32; 3];
            let mut delta = [0.0f32; 3];
            let mut angleToLeader = [0.0f32; 3];

            CalcEntitySpot(ctx, Some(leader_ent), spot_t::SPOT_HEAD, &mut leaderHead);
            CalcEntitySpot(ctx, Some(npc_id), spot_t::SPOT_HEAD, &mut head);
            _VectorSubtract(leaderHead, head, &mut delta);
            vectoangles(delta, &mut angleToLeader);
            VectorNormalize(&mut delta);
            (*NPCInfo).desiredYaw = angleToLeader[1];
            (*NPCInfo).desiredPitch = angleToLeader[0];

            NPC_UpdateAngles(ctx, qtrue, qtrue);
        }

        // Leader visible?
        let leaderVis =
            NPC_CheckVisibility(ctx, Some(leader_ent), CHECK_PVS | CHECK_360 | CHECK_SHOOT);

        let curAnim = (*npc_client).ps.legsAnim;
        if curAnim != BOTH_ATTACK1 as c_int
            && curAnim != BOTH_ATTACK2 as c_int
            && curAnim != BOTH_ATTACK3 as c_int
            && curAnim != BOTH_MELEE1 as c_int
            && curAnim != BOTH_MELEE2 as c_int
        {
            let mut followDist: f32 = 96.0;
            if (*NPCInfo).followDist != 0.0 {
                followDist = (*NPCInfo).followDist;
            }
            let backupdist = followDist / 2.0;
            // C's `0.83`/`1.33` are double literals, so each product is formed in
            // double and narrowed to the f32 local; match that width so the
            // `walkdist`/`minrundist` boundary comparisons agree.
            let walkdist = (followDist as f64 * 0.83) as f32;
            let minrundist = (followDist as f64 * 1.33) as f32;

            let mut vec = [0.0f32; 3];
            let leader_origin = ctx.world.entity(leader_ent).r.currentOrigin;
            let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
            _VectorSubtract(leader_origin, npc_origin, &mut vec);
            let leaderDist = VectorLength(vec);
            vec[2] = 0.0;
            let leaderHDist = VectorLength(vec);
            if leaderHDist > backupdist
                && ((leaderVis as c_int) != (visibility_t::VIS_SHOOT as c_int)
                    || leaderDist > walkdist)
            {
                (*NPCInfo).goalEntity = Some(leader_ent);
                NPC_SlideMoveToGoal(ctx);
                if (leaderVis as c_int) == (visibility_t::VIS_SHOOT as c_int)
                    && leaderDist < minrundist
                {
                    ctx.world.globals.ucmd.buttons |= BUTTON_WALKING as c_int;
                }
            } else if leaderDist < backupdist {
                (*NPCInfo).goalEntity = Some(leader_ent);
                NPC_SlideMoveToGoal(ctx);

                ctx.world.globals.ucmd.forwardmove = -ctx.world.globals.ucmd.forwardmove;
                ctx.world.globals.ucmd.rightmove = -ctx.world.globals.ucmd.rightmove;
                _VectorScale(
                    (*npc_client).ps.moveDir,
                    -1.0,
                    &mut (*npc_client).ps.moveDir,
                );
            }
            if ctx.world.globals.ucmd.forwardmove != 0
                || ctx.world.globals.ucmd.rightmove != 0
                || VectorCompare(vec3_origin, (*npc_client).ps.moveDir)
            {
                crate::NPC_AI_Jedi::NPC_MoveDirClear(
                    ctx,
                    ctx.world.globals.ucmd.forwardmove as c_int,
                    ctx.world.globals.ucmd.rightmove as c_int,
                    qtrue,
                );
            }
        }
    }
}

/// Raven `NPC_BSJump`.
///
/// Source: `oracle/codemp/game/NPC_behavior.c:733-919`
pub fn NPC_BSJump(ctx: &mut GameContext) {
    let NPC = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(NPC).unwrap();
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let NPCInfo = ctx.world.globals.NPCInfo;
    // FLAG: pool client; deref raw via safe borrow (trap 2b).
    let npc_client = ctx.world.entity(npc_id).client;
    unsafe {
        let Some(goal_id) = (*NPCInfo).goalEntity else {
            return;
        };

        if (*NPCInfo).jumpState != JS_JUMPING && (*NPCInfo).jumpState != JS_LANDING {
            // Face navgoal.
            let mut dir = [0.0f32; 3];
            let mut angles = [0.0f32; 3];
            let goal_origin = ctx.world.entity(goal_id).r.currentOrigin;
            let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
            _VectorSubtract(goal_origin, npc_origin, &mut dir);
            vectoangles(dir, &mut angles);
            (*NPCInfo).desiredPitch = AngleNormalize360(angles[0]);
            (*NPCInfo).lockedDesiredPitch = (*NPCInfo).desiredPitch;
            (*NPCInfo).desiredYaw = AngleNormalize360(angles[1]);
            (*NPCInfo).lockedDesiredYaw = (*NPCInfo).desiredYaw;
        }

        NPC_UpdateAngles(ctx, qtrue, qtrue);
        let yawError = AngleDelta((*npc_client).ps.viewangles[1], (*NPCInfo).desiredYaw);

        match (*NPCInfo).jumpState {
            jumpState_t::JS_FACING => {
                if yawError < MIN_ANGLE_ERROR {
                    NPC_SetAnim(
                        ctx,
                        npc_id,
                        SETANIM_LEGS,
                        BOTH_CROUCH1 as c_int,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                    (*NPCInfo).jumpState = JS_CROUCHING;
                }
            }
            jumpState_t::JS_CROUCHING => {
                if (*npc_client).ps.legsTimer > 0 {
                    return;
                }

                let mut p1 = [0.0f32; 3];
                let mut p2 = [0.0f32; 3];
                let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
                let goal_origin = ctx.world.entity(goal_id).r.currentOrigin;
                if npc_origin[2] > goal_origin[2] {
                    _VectorCopy(npc_origin, &mut p1);
                    _VectorCopy(goal_origin, &mut p2);
                } else if npc_origin[2] < goal_origin[2] {
                    _VectorCopy(goal_origin, &mut p1);
                    _VectorCopy(npc_origin, &mut p2);
                } else {
                    _VectorCopy(npc_origin, &mut p1);
                    _VectorCopy(goal_origin, &mut p2);
                }

                let mut dir = [0.0f32; 3];
                _VectorSubtract(p2, p1, &mut dir);
                dir[2] = 0.0;

                let mut xy = VectorNormalize(&mut dir);
                let mut z = p1[2] - p2[2];

                let apexHeight: f32 = APEX_HEIGHT / 2.0;

                // C's `sqrt` is the libm double routine: the f32 `apexHeight + z`
                // is promoted to double for each sqrt and the result narrowed back
                // to the f32 local; evaluate through f64 to match.
                z = (((apexHeight + z) as f64).sqrt() - (apexHeight as f64).sqrt()) as f32;
                debug_assert!(z >= 0.0);

                xy -= z;
                xy *= 0.5;
                debug_assert!(xy > 0.0);

                let mut apex = [0.0f32; 3];
                _VectorMA(p1, xy, dir, &mut apex);
                apex[2] += apexHeight;

                _VectorCopy(apex, &mut ctx.world.entity_mut(npc_id).pos1);

                let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
                let height = apex[2] - npc_origin[2];
                // C evaluates `.5 * gravity` and the `height / …` divide in double
                // (libm `sqrt`), narrowing only into the f32 `time`; match that width.
                let time =
                    ((height as f64) / (0.5 * (*npc_client).ps.gravity as f64)).sqrt() as f32;
                if time == 0.0 {
                    return;
                }

                _VectorSubtract(apex, npc_origin, &mut (*npc_client).ps.velocity);
                (*npc_client).ps.velocity[2] = 0.0;
                let dist = VectorNormalize(&mut (*npc_client).ps.velocity);

                let forward = dist / time;
                _VectorScale(
                    (*npc_client).ps.velocity,
                    forward,
                    &mut (*npc_client).ps.velocity,
                );

                (*npc_client).ps.velocity[2] = time * (*npc_client).ps.gravity as f32;

                ctx.world.entity_mut(npc_id).flags |= FL_NO_KNOCKBACK;
                (*NPCInfo).jumpState = JS_JUMPING;
            }
            jumpState_t::JS_JUMPING => {
                if ctx.world.globals.showBBoxes != 0 {
                    let mut p1 = [0.0f32; 3];
                    let mut p2 = [0.0f32; 3];
                    let mins = ctx.world.entity(npc_id).r.mins;
                    let maxs = ctx.world.entity(npc_id).r.maxs;
                    let pos1 = ctx.world.entity(npc_id).pos1;
                    _VectorAdd(mins, pos1, &mut p1);
                    _VectorAdd(maxs, pos1, &mut p2);
                    crate::g_nav::G_Cube(p1, p2, [0.0, 0.0, 1.0], 0.5);
                }

                if ctx.world.entity(npc_id).s.groundEntityNum != ENTITYNUM_NONE {
                    // Landed, start landing anim.
                    (*npc_client).ps.velocity = [0.0; 3];
                    NPC_SetAnim(
                        ctx,
                        npc_id,
                        SETANIM_BOTH,
                        BOTH_LAND1 as c_int,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                    (*NPCInfo).jumpState = JS_LANDING;
                } else if (*npc_client).ps.legsTimer > 0 {
                    return;
                } else {
                    NPC_SetAnim(
                        ctx,
                        npc_id,
                        SETANIM_BOTH,
                        BOTH_INAIR1 as c_int,
                        SETANIM_FLAG_OVERRIDE,
                    );
                }
            }
            jumpState_t::JS_LANDING => {
                if (*npc_client).ps.legsTimer > 0 {
                    return;
                } else {
                    (*NPCInfo).jumpState = JS_WAITING;

                    NPC_ClearGoal(ctx);
                    (*NPCInfo).goalTime = ctx.world.level.time;
                    (*NPCInfo).aiFlags &= !NPCAI_MOVING;
                    ctx.world.globals.ucmd.forwardmove = 0;
                    ctx.world.entity_mut(npc_id).flags &= !FL_NO_KNOCKBACK;
                    trap::ICARUS_TaskIDComplete(
                        ctx.engine,
                        GIcarusTaskidcompleteArgs::new(NPC.cast(), taskID_t::TID_MOVE_NAV as c_int),
                    );
                }
            }
            _ => {
                (*NPCInfo).jumpState = JS_FACING;
            }
        }
    }
}

/// Raven `NPC_BSRemove`.
///
/// Source: `oracle/codemp/game/NPC_behavior.c:921-937`
pub fn NPC_BSRemove(ctx: &mut GameContext) {
    let NPC = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(NPC).unwrap();

    NPC_UpdateAngles(ctx, qtrue, qtrue);
    let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
    let ent0_origin = ctx.world.entity(EntityId(0)).r.currentOrigin;
    if trap::InPVS(
        ctx.engine,
        GInPvsArgs::new(&npc_origin as *const _, &ent0_origin as *const _),
    ) == 0
    {
        let target3 = ctx.world.entity(npc_id).target3.clone();
        let target3 = if target3.is_empty() { None } else { Some(target3) };
        G_UseTargets2(ctx, Some(npc_id), Some(npc_id), target3.as_deref());
        let level_time = ctx.world.level.time;
        let ent = ctx.world.entity_mut(npc_id) as *mut gentity_t;
        unsafe {
            (*ent).s.eFlags |= EF_NODRAW;
            (*ent).s.eType = entityType_t::ET_INVISIBLE as c_int;
            (*ent).r.contents = 0;
            (*ent).health = 0;
            ctx.ent_set(npc_id, PrefixSet::Targetname(None));

            // Disappear in half a second.
            // (shape mismatch, see BeamOut note above.)
            (*ent).think = Some(EntThink::G_FreeEntity).into();
            (*ent).nextthink = level_time + FRAMETIME;
        }
    }
}

/// Raven `NPC_BSSearch`.
///
/// Source: `oracle/codemp/game/NPC_behavior.c:939-1124`
pub fn NPC_BSSearch(ctx: &mut GameContext) {
    let NPC = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(NPC).unwrap();
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let NPCInfo = ctx.world.globals.NPCInfo;
    unsafe {
        NPC_CheckEnemy(ctx, qtrue, qfalse, qtrue);
        if ctx.world.entity(npc_id).enemy.is_some() {
            if (*NPCInfo).tempBehavior == BS_SEARCH {
                (*NPCInfo).tempBehavior = BS_DEFAULT;
            } else {
                (*NPCInfo).behaviorState = BS_HUNT_AND_KILL;
                crate::NPC_AI_Default::NPC_BSRunAndShoot(ctx);
            }
            return;
        }

        if (*NPCInfo).investigateDebounceTime == 0 {
            let minGoalReachedDistSquared: f32 = 32.0 * 32.0;
            let Some(tempGoal_id) = (*NPCInfo).tempGoal else {
                return;
            };

            (*NPCInfo).goalEntity = (*NPCInfo).tempGoal;

            let mut vec = [0.0f32; 3];
            let tempGoal_origin = ctx.world.entity(tempGoal_id).r.currentOrigin;
            let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
            _VectorSubtract(tempGoal_origin, npc_origin, &mut vec);
            if vec[2] < 24.0 {
                vec[2] = 0.0;
            }

            if VectorLengthSquared(vec) < minGoalReachedDistSquared {
                // Close enough, just got there.
                let wp = NAV_FindClosestWaypointForEnt(ctx, npc_id, WAYPOINT_NONE);
                ctx.world.entity_mut(npc_id).waypoint = wp;

                if (*NPCInfo).homeWp == WAYPOINT_NONE
                    || ctx.world.entity(npc_id).waypoint == WAYPOINT_NONE
                {
                    if (*NPCInfo).tempBehavior == BS_SEARCH {
                        (*NPCInfo).tempBehavior = BS_DEFAULT;
                    } else {
                        (*NPCInfo).behaviorState = BS_STAND_GUARD;
                        crate::NPC_AI_Default::NPC_BSRunAndShoot(ctx);
                    }
                    return;
                }

                if ctx.world.entity(npc_id).waypoint == (*NPCInfo).homeWp {
                    if (*NPCInfo).aiFlags & NPCAI_ENROUTE_TO_HOMEWP != 0 {
                        (*NPCInfo).aiFlags &= !NPCAI_ENROUTE_TO_HOMEWP;
                        G_ActivateBehavior(ctx, Some(npc_id), BSET_LOSTENEMY as c_int);
                    }
                }

                if ctx.world.bg_state.rng.Q_irand(0, 1) == 0 {
                    NPC_SetAnim(
                        ctx,
                        npc_id,
                        SETANIM_BOTH,
                        BOTH_GUARD_LOOKAROUND1 as c_int,
                        SETANIM_FLAG_NORMAL,
                    );
                } else {
                    NPC_SetAnim(
                        ctx,
                        npc_id,
                        SETANIM_BOTH,
                        BOTH_GUARD_IDLE1 as c_int,
                        SETANIM_FLAG_NORMAL,
                    );
                }
                (*NPCInfo).investigateDebounceTime =
                    ctx.world.level.time + ctx.world.bg_state.rng.Q_irand(3000, 10000);
            } else {
                NPC_MoveToGoal(ctx, qtrue);
            }
        } else {
            if (*NPCInfo).investigateDebounceTime > ctx.world.level.time {
                if let Some(tempGoal_id) = (*NPCInfo).tempGoal {
                    if ctx.world.entity(tempGoal_id).waypoint != WAYPOINT_NONE {
                        if ctx.world.bg_state.rng.Q_irand(0, 30) == 0 {
                            let tempGoal_wp = ctx.world.entity(tempGoal_id).waypoint;
                            let numEdges = trap::Nav_GetNodeNumEdges(
                                ctx.engine,
                                GNavGetnodenumedgesArgs::new(tempGoal_wp),
                            );
                            if numEdges != WAYPOINT_NONE {
                                let branchNum = ctx.world.bg_state.rng.Q_irand(0, numEdges - 1);
                                let mut branchPos = [0.0f32; 3];
                                let mut lookDir = [0.0f32; 3];
                                let nextWp = trap::Nav_GetNodeEdge(
                                    ctx.engine,
                                    GNavGetnodeedgeArgs::new(tempGoal_wp, branchNum),
                                );
                                trap::Nav_GetNodePosition(
                                    ctx.engine,
                                    GNavGetnodepositionArgs::new(
                                        nextWp,
                                        &mut branchPos as *mut vec3_t,
                                    ),
                                );
                                let tempGoal_origin = ctx.world.entity(tempGoal_id).r.currentOrigin;
                                _VectorSubtract(branchPos, tempGoal_origin, &mut lookDir);
                                (*NPCInfo).desiredYaw = AngleNormalize360(
                                    vectoyaw(lookDir) + ctx.world.bg_state.rng.flrand(-45.0, 45.0),
                                );
                            }
                        }
                    }
                }
            } else {
                // Just finished waiting.
                let wp = NAV_FindClosestWaypointForEnt(ctx, npc_id, WAYPOINT_NONE);
                ctx.world.entity_mut(npc_id).waypoint = wp;

                if let Some(tempGoal_id) = (*NPCInfo).tempGoal {
                    if ctx.world.entity(npc_id).waypoint == (*NPCInfo).homeWp {
                        let tempGoal_wp = ctx.world.entity(tempGoal_id).waypoint;
                        let numEdges = trap::Nav_GetNodeNumEdges(
                            ctx.engine,
                            GNavGetnodenumedgesArgs::new(tempGoal_wp),
                        );
                        if numEdges != WAYPOINT_NONE {
                            let branchNum = ctx.world.bg_state.rng.Q_irand(0, numEdges - 1);
                            let nextWp = trap::Nav_GetNodeEdge(
                                ctx.engine,
                                GNavGetnodeedgeArgs::new((*NPCInfo).homeWp, branchNum),
                            );
                            let dst = &mut ctx.world.entity_mut(tempGoal_id).r.currentOrigin
                                as *mut vec3_t;
                            trap::Nav_GetNodePosition(
                                ctx.engine,
                                GNavGetnodepositionArgs::new(nextWp, dst),
                            );
                            ctx.world.entity_mut(tempGoal_id).waypoint = nextWp;
                        }
                    } else {
                        let dst =
                            &mut ctx.world.entity_mut(tempGoal_id).r.currentOrigin as *mut vec3_t;
                        trap::Nav_GetNodePosition(
                            ctx.engine,
                            GNavGetnodepositionArgs::new((*NPCInfo).homeWp, dst),
                        );
                        ctx.world.entity_mut(tempGoal_id).waypoint = (*NPCInfo).homeWp;
                    }

                    (*NPCInfo).investigateDebounceTime = 0;
                    (*NPCInfo).goalEntity = (*NPCInfo).tempGoal;
                    NPC_MoveToGoal(ctx, qtrue);
                }
            }
        }

        NPC_UpdateAngles(ctx, qtrue, qtrue);
    }
}

/// Raven `NPC_BSSearchStart`.
///
/// Source: `oracle/codemp/game/NPC_behavior.c:1132-1150`
pub fn NPC_BSSearchStart(ctx: &mut GameContext, homeWp: c_int, bState: bState_t) {
    let NPC = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(NPC).unwrap();
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let NPCInfo = ctx.world.globals.NPCInfo;
    unsafe {
        let mut homeWp = homeWp;

        if homeWp == WAYPOINT_NONE {
            homeWp = NAV_FindClosestWaypointForEnt(ctx, npc_id, WAYPOINT_NONE);
            if ctx.world.entity(npc_id).waypoint == WAYPOINT_NONE {
                ctx.world.entity_mut(npc_id).waypoint = homeWp;
            }
        }
        (*NPCInfo).homeWp = homeWp;
        (*NPCInfo).tempBehavior = bState;
        (*NPCInfo).aiFlags |= NPCAI_ENROUTE_TO_HOMEWP;
        (*NPCInfo).investigateDebounceTime = 0;
        if let Some(tempGoal_id) = (*NPCInfo).tempGoal {
            let dst = &mut ctx.world.entity_mut(tempGoal_id).r.currentOrigin as *mut vec3_t;
            trap::Nav_GetNodePosition(ctx.engine, GNavGetnodepositionArgs::new(homeWp, dst));
            ctx.world.entity_mut(tempGoal_id).waypoint = homeWp;
        }
    }
}

/// Raven `NPC_BSNoClip`.
///
/// Source: `oracle/codemp/game/NPC_behavior.c:1160-1191`
pub fn NPC_BSNoClip(ctx: &mut GameContext) {
    let NPC = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(NPC).unwrap();
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let NPCInfo = ctx.world.globals.NPCInfo;
    unsafe {
        if !UpdateGoal(ctx).is_null() {
            // Raven's `if (UpdateGoal())` implies `goalEntity` is non-NULL here;
            // `.expect()` encodes that invariant.
            let goal_id = (*NPCInfo).goalEntity.expect("UpdateGoal() set goalEntity");
            let mut dir = [0.0f32; 3];
            let mut forward = [0.0f32; 3];
            let mut right = [0.0f32; 3];
            let mut angles = [0.0f32; 3];
            let up = [0.0f32, 0.0, 1.0];

            let goal_origin = ctx.world.entity(goal_id).r.currentOrigin;
            let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
            _VectorSubtract(goal_origin, npc_origin, &mut dir);

            vectoangles(dir, &mut angles);
            (*NPCInfo).desiredYaw = angles[1];

            let npc_angles = ctx.world.entity(npc_id).r.currentAngles;
            AngleVectors(npc_angles, Some(&mut forward), Some(&mut right), None);

            VectorNormalize(&mut dir);

            let fDot = _DotProduct(forward, dir) * 127.0;
            let rDot = _DotProduct(right, dir) * 127.0;
            let uDot = _DotProduct(up, dir) * 127.0;

            ctx.world.globals.ucmd.forwardmove = fDot.floor() as i8;
            ctx.world.globals.ucmd.rightmove = rDot.floor() as i8;
            ctx.world.globals.ucmd.upmove = uDot.floor() as i8;
        } else {
            // FLAG: pool client; deref raw via safe borrow (trap 2b).
            let npc_client = ctx.world.entity(npc_id).client;
            (*npc_client).ps.velocity = [0.0; 3];
        }

        NPC_UpdateAngles(ctx, qtrue, qtrue);
    }
}

/// Raven `NPC_BSWander`.
///
/// Source: `oracle/codemp/game/NPC_behavior.c:1193-1286`
pub fn NPC_BSWander(ctx: &mut GameContext) {
    let NPC = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(NPC).unwrap();
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let NPCInfo = ctx.world.globals.NPCInfo;
    unsafe {
        if (*NPCInfo).investigateDebounceTime == 0 {
            let mut minGoalReachedDistSquared: f32 = 64.0;
            let Some(tempGoal_id) = (*NPCInfo).tempGoal else {
                return;
            };

            (*NPCInfo).goalEntity = (*NPCInfo).tempGoal;

            let mut vec = [0.0f32; 3];
            let tempGoal_origin = ctx.world.entity(tempGoal_id).r.currentOrigin;
            let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
            _VectorSubtract(tempGoal_origin, npc_origin, &mut vec);

            if ctx.world.entity(tempGoal_id).waypoint != WAYPOINT_NONE {
                minGoalReachedDistSquared = 64.0;
            }

            if VectorLengthSquared(vec) < minGoalReachedDistSquared {
                let wp = NAV_FindClosestWaypointForEnt(ctx, npc_id, WAYPOINT_NONE);
                ctx.world.entity_mut(npc_id).waypoint = wp;

                if ctx.world.bg_state.rng.Q_irand(0, 1) == 0 {
                    NPC_SetAnim(
                        ctx,
                        npc_id,
                        SETANIM_BOTH,
                        BOTH_GUARD_LOOKAROUND1 as c_int,
                        SETANIM_FLAG_NORMAL,
                    );
                } else {
                    NPC_SetAnim(
                        ctx,
                        npc_id,
                        SETANIM_BOTH,
                        BOTH_GUARD_IDLE1 as c_int,
                        SETANIM_FLAG_NORMAL,
                    );
                }
                (*NPCInfo).investigateDebounceTime =
                    ctx.world.level.time + ctx.world.bg_state.rng.Q_irand(3000, 10000);
            } else {
                NPC_MoveToGoal(ctx, qtrue);
            }
        } else {
            if (*NPCInfo).investigateDebounceTime > ctx.world.level.time {
                if let Some(tempGoal_id) = (*NPCInfo).tempGoal {
                    if ctx.world.entity(tempGoal_id).waypoint != WAYPOINT_NONE {
                        if ctx.world.bg_state.rng.Q_irand(0, 30) == 0 {
                            let tempGoal_wp = ctx.world.entity(tempGoal_id).waypoint;
                            let numEdges = trap::Nav_GetNodeNumEdges(
                                ctx.engine,
                                GNavGetnodenumedgesArgs::new(tempGoal_wp),
                            );
                            if numEdges != WAYPOINT_NONE {
                                let branchNum = ctx.world.bg_state.rng.Q_irand(0, numEdges - 1);
                                let mut branchPos = [0.0f32; 3];
                                let mut lookDir = [0.0f32; 3];
                                let nextWp = trap::Nav_GetNodeEdge(
                                    ctx.engine,
                                    GNavGetnodeedgeArgs::new(tempGoal_wp, branchNum),
                                );
                                trap::Nav_GetNodePosition(
                                    ctx.engine,
                                    GNavGetnodepositionArgs::new(
                                        nextWp,
                                        &mut branchPos as *mut vec3_t,
                                    ),
                                );
                                let tempGoal_origin = ctx.world.entity(tempGoal_id).r.currentOrigin;
                                _VectorSubtract(branchPos, tempGoal_origin, &mut lookDir);
                                (*NPCInfo).desiredYaw = AngleNormalize360(
                                    vectoyaw(lookDir) + ctx.world.bg_state.rng.flrand(-45.0, 45.0),
                                );
                            }
                        }
                    }
                }
            } else {
                let wp = NAV_FindClosestWaypointForEnt(ctx, npc_id, WAYPOINT_NONE);
                ctx.world.entity_mut(npc_id).waypoint = wp;

                if ctx.world.entity(npc_id).waypoint != WAYPOINT_NONE {
                    let npc_wp = ctx.world.entity(npc_id).waypoint;
                    let numEdges =
                        trap::Nav_GetNodeNumEdges(ctx.engine, GNavGetnodenumedgesArgs::new(npc_wp));
                    if numEdges != WAYPOINT_NONE {
                        if let Some(tempGoal_id) = (*NPCInfo).tempGoal {
                            let branchNum = ctx.world.bg_state.rng.Q_irand(0, numEdges - 1);
                            let nextWp = trap::Nav_GetNodeEdge(
                                ctx.engine,
                                GNavGetnodeedgeArgs::new(npc_wp, branchNum),
                            );
                            let dst = &mut ctx.world.entity_mut(tempGoal_id).r.currentOrigin
                                as *mut vec3_t;
                            trap::Nav_GetNodePosition(
                                ctx.engine,
                                GNavGetnodepositionArgs::new(nextWp, dst),
                            );
                            ctx.world.entity_mut(tempGoal_id).waypoint = nextWp;
                        }
                    }

                    // Oracle: these three run inside `if waypoint != WAYPOINT_NONE`
                    // but outside `if numEdges != WAYPOINT_NONE`.
                    // Source: oracle/codemp/game/NPC_behavior.c:1276-1280
                    (*NPCInfo).investigateDebounceTime = 0;
                    (*NPCInfo).goalEntity = (*NPCInfo).tempGoal;
                    NPC_MoveToGoal(ctx, qtrue);
                }
            }
        }

        NPC_UpdateAngles(ctx, qtrue, qtrue);
    }
}

/// Raven `NPC_Surrender`.
///
/// Source: `oracle/codemp/game/NPC_behavior.c:1317-1339`
pub fn NPC_Surrender(ctx: &mut GameContext) {
    let NPC = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(NPC).unwrap();
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let NPCInfo = ctx.world.globals.NPCInfo;
    // FLAG: pool client; deref raw via safe borrow (trap 2b).
    let npc_client = ctx.world.entity(npc_id).client;
    unsafe {
        if (*npc_client).ps.weaponTime != 0 || PM_InKnockDown(&mut (*npc_client).ps) != 0 {
            return;
        }
        if ctx.world.entity(npc_id).s.weapon != WP_NONE as c_int
            && ctx.world.entity(npc_id).s.weapon != WP_STUN_BATON as c_int
            && ctx.world.entity(npc_id).s.weapon != WP_SABER as c_int
        {
            //WP_DropWeapon( NPC, NULL ); //rwwFIXMEFIXME
        }
        if (*NPCInfo).surrenderTime < ctx.world.level.time - 5000 {
            (*NPCInfo).blockedSpeechDebounceTime = 0;
            let voice_event = ctx
                .world
                .bg_state
                .rng
                .Q_irand(EV_PUSHED1 as c_int, EV_PUSHED3 as c_int);
            G_AddVoiceEvent(ctx, npc_id, voice_event, 3000);
        }
        (*NPCInfo).surrenderTime = ctx.world.level.time + 1000;
    }
}

/// Raven `NPC_CheckSurrender`.
///
/// Source: `oracle/codemp/game/NPC_behavior.c:1341-1442`
pub fn NPC_CheckSurrender(ctx: &mut GameContext) -> qboolean {
    let NPC = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(NPC).unwrap();
    // FLAG: pool client; deref raw via safe borrow (trap 2b).
    let npc_client = ctx.world.entity(npc_id).client;
    unsafe {
        let Some(enemy_id) = ctx.world.entity(npc_id).enemy else {
            return qfalse;
        };
        // FLAG: pool client; deref raw via safe borrow (trap 2b).
        let enemy_client = ctx.world.entity(enemy_id).client;

        if trap::ICARUS_TaskIDPending(
            ctx.engine,
            GIcarusTaskidpendingArgs::new(NPC.cast(), taskID_t::TID_MOVE_NAV as c_int),
        ) == 0
            && (*npc_client).ps.groundEntityNum != ENTITYNUM_NONE
            && (*npc_client).ps.weaponTime == 0
            && PM_InKnockDown(&mut (*npc_client).ps) == 0
            && !enemy_client.is_null()
            && ctx.world.entity(enemy_id).enemy == Some(npc_id)
            && ctx.world.entity(enemy_id).s.weapon != WP_NONE as c_int
            && ctx.world.entity(enemy_id).s.weapon != WP_STUN_BATON as c_int
            && ctx.world.entity(enemy_id).health > 20
            && ctx.world.entity(enemy_id).painDebounceTime < ctx.world.level.time - 3000
            && (*enemy_client).ps.fd.forcePowerDebounce[FP_SABER_DEFENSE as usize]
                < ctx.world.level.time - 1000
        {
            if ctx.world.entity(npc_id).s.weapon != WP_ROCKET_LAUNCHER as c_int
                && ctx.world.entity(npc_id).s.weapon != WP_REPEATER as c_int
                && ctx.world.entity(npc_id).s.weapon != WP_FLECHETTE as c_int
                && ctx.world.entity(npc_id).s.weapon != WP_SABER as c_int
            {
                if ctx.world.entity(npc_id).s.weapon != WP_NONE as c_int {
                    if ctx.world.entity(npc_id).health > 25 {
                        return qfalse;
                    }
                    if NPC_SomeoneLookingAtMe(ctx, npc_id) != qfalse
                        && ctx.world.entity(npc_id).painDebounceTime > ctx.world.level.time
                    {
                        // Fall through.
                    } else {
                        if InFOV(ctx, Some(enemy_id), npc_id, 60, 30) == qfalse {
                            return qfalse;
                        }
                        let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
                        let enemy_origin = ctx.world.entity(enemy_id).r.currentOrigin;
                        if crate::q_math::DistanceSquared(npc_origin, enemy_origin) < 65536.0 {
                            return qfalse;
                        } else if trap::InPVS(
                            ctx.engine,
                            GInPvsArgs::new(&npc_origin as *const _, &enemy_origin as *const _),
                        ) == 0
                        {
                            return qfalse;
                        }
                    }
                }
            }
        }
        qfalse
    }
}

/// Raven `NPC_BSFlee`.
///
/// Source: `oracle/codemp/game/NPC_behavior.c:1444-1558`
pub fn NPC_BSFlee(ctx: &mut GameContext) {
    let NPC = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(NPC).unwrap();
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let NPCInfo = ctx.world.globals.NPCInfo;
    unsafe {
        let flee_s = cstr("flee");
        if TIMER_Done(ctx, Some(npc_id), flee_s.as_ptr()) != 0 && (*NPCInfo).tempBehavior == BS_FLEE
        {
            (*NPCInfo).tempBehavior = BS_DEFAULT;
            (*NPCInfo).squadState = SQUAD_IDLE;
        }
        if NPC_CheckSurrender(ctx) != qfalse {
            return;
        }
        let mut goal_id = (*NPCInfo).goalEntity;
        if goal_id.is_none() {
            goal_id = (*NPCInfo).lastGoalEntity;
            if goal_id.is_none() {
                goal_id = (*NPCInfo).tempGoal;
            }
        }

        if let Some(goal_id) = goal_id {
            let mut moved;
            let mut reverseCourse = qtrue;

            if ctx.world.entity(npc_id).waypoint == WAYPOINT_NONE {
                let last_wp = ctx.world.entity(npc_id).lastWaypoint;
                let wp = NAV_GetNearestNode(ctx, npc_id, last_wp);
                ctx.world.entity_mut(npc_id).waypoint = wp;
            }
            if ctx.world.entity(npc_id).waypoint != WAYPOINT_NONE {
                let npc_wp = ctx.world.entity(npc_id).waypoint;
                let numEdges =
                    trap::Nav_GetNodeNumEdges(ctx.engine, GNavGetnodenumedgesArgs::new(npc_wp));

                if numEdges != WAYPOINT_NONE {
                    let mut dangerDir = [0.0f32; 3];
                    let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
                    _VectorSubtract((*NPCInfo).investigateGoal, npc_origin, &mut dangerDir);
                    VectorNormalize(&mut dangerDir);

                    for branchNum in 0..numEdges {
                        let mut branchPos = [0.0f32; 3];
                        let mut runDir = [0.0f32; 3];

                        let npc_wp = ctx.world.entity(npc_id).waypoint;
                        let nextWp = trap::Nav_GetNodeEdge(
                            ctx.engine,
                            GNavGetnodeedgeArgs::new(npc_wp, branchNum),
                        );
                        trap::Nav_GetNodePosition(
                            ctx.engine,
                            GNavGetnodepositionArgs::new(nextWp, &mut branchPos as *mut vec3_t),
                        );

                        let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
                        _VectorSubtract(branchPos, npc_origin, &mut runDir);
                        VectorNormalize(&mut runDir);
                        if _DotProduct(runDir, dangerDir) > ctx.world.bg_state.rng.flrand(0.0, 0.5)
                        {
                            continue;
                        }
                        NPC_SetMoveGoal(ctx, npc_id, branchPos, 0, qtrue, -1, None);
                        reverseCourse = qfalse;
                        break;
                    }
                }
            }

            moved = NPC_MoveToGoal(ctx, qfalse);

            if ctx.world.entity(npc_id).s.weapon == WP_NONE as c_int
                && (moved == qfalse || reverseCourse != qfalse)
            {
                NPC_Surrender(ctx);
                NPC_UpdateAngles(ctx, qtrue, qtrue);
                return;
            }
            if moved == qfalse {
                let mut dir = [0.0f32; 3];
                let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
                let goal_origin = ctx.world.entity(goal_id).r.currentOrigin;
                if reverseCourse != qfalse {
                    _VectorSubtract(npc_origin, goal_origin, &mut dir);
                } else {
                    _VectorSubtract(goal_origin, npc_origin, &mut dir);
                }
                let dist = VectorNormalize(&mut dir);
                (*NPCInfo).distToGoal = dist;
                (*NPCInfo).desiredYaw = vectoyaw(dir);
                (*NPCInfo).desiredPitch = 0.0;
                ctx.world.globals.ucmd.forwardmove = 127;
            } else if reverseCourse != qfalse {
                (*NPCInfo).desiredYaw *= -1.0;
            }
            ctx.world.globals.ucmd.buttons &= !(BUTTON_WALKING as c_int);
        }
        NPC_UpdateAngles(ctx, qtrue, qtrue);

        NPC_CheckGetNewWeapon(ctx);
    }
}

/// Raven `NPC_StartFlee`.
///
/// Source: `oracle/codemp/game/NPC_behavior.c:1560-1634`
pub fn NPC_StartFlee(
    ctx: &mut GameContext,
    enemy: Option<EntityId>,
    dangerPoint: vec3_t,
    dangerLevel: c_int,
    fleeTimeMin: c_int,
    fleeTimeMax: c_int,
) {
    let NPC = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(NPC).unwrap();
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let NPCInfo = ctx.world.globals.NPCInfo;
    // FLAG: pool client; deref raw via safe borrow (trap 2b).
    let npc_client = ctx.world.entity(npc_id).client;
    unsafe {
        let mut cp: c_int = -1;

        if trap::ICARUS_TaskIDPending(
            ctx.engine,
            GIcarusTaskidpendingArgs::new(NPC.cast(), taskID_t::TID_MOVE_NAV as c_int),
        ) != 0
        {
            return;
        }

        if G_ActivateBehavior(ctx, Some(npc_id), BSET_FLEE as c_int) != 0 {
            return;
        }
        if enemy.is_some() {
            G_SetEnemy(ctx, npc_id, enemy);
        }

        let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
        // FLAG: gNPC_t.group is a raw pool pointer (no accessor); deref raw (recipe 2c).
        if dangerLevel > AEL_DANGER as c_int
            || ctx.world.entity(npc_id).s.weapon == WP_NONE as c_int
            || (((*NPCInfo).group.is_null() || (*(*NPCInfo).group).numGroup <= 1)
                && ctx.world.entity(npc_id).health <= 10)
        {
            cp = NPC_FindCombatPoint(
                ctx,
                npc_origin,
                npc_origin,
                dangerPoint,
                CP_COVER | CP_AVOID | CP_HAS_ROUTE | CP_NO_PVS,
                128.0,
                -1,
            );
        }
        if cp == -1 {
            cp = NPC_FindCombatPoint(
                ctx,
                npc_origin,
                npc_origin,
                dangerPoint,
                CP_COVER | CP_AVOID | CP_HAS_ROUTE,
                128.0,
                -1,
            );
            if cp == -1 {
                cp = NPC_FindCombatPoint(
                    ctx,
                    npc_origin,
                    npc_origin,
                    dangerPoint,
                    CP_COVER | CP_HAS_ROUTE,
                    128.0,
                    -1,
                );
                if cp == -1 {
                    cp = NPC_FindCombatPoint(
                        ctx,
                        npc_origin,
                        npc_origin,
                        dangerPoint,
                        CP_HAS_ROUTE,
                        128.0,
                        -1,
                    );
                }
            }
        }

        if cp != -1 {
            NPC_SetCombatPoint(ctx, cp);
            let cp_origin = ctx.world.level.combatPoints[cp as usize].origin;
            NPC_SetMoveGoal(ctx, npc_id, cp_origin, 8, qtrue, cp, None);
            (*NPCInfo).behaviorState = BS_HUNT_AND_KILL;
            (*NPCInfo).tempBehavior = BS_DEFAULT;
        } else {
            if ctx.world.entity(npc_id).s.weapon != WP_NONE as c_int {
                return;
            } else {
                (*NPCInfo).tempBehavior = BS_FLEE;
                NPC_SetMoveGoal(ctx, npc_id, dangerPoint, 0, qtrue, -1, None);
                _VectorCopy(dangerPoint, &mut (*NPCInfo).investigateGoal);
            }
        }
        let s = cstr("attackDelay");
        let atk_delay = ctx.world.bg_state.rng.Q_irand(500, 2500);
        TIMER_Set(ctx, Some(npc_id), s.as_ptr(), atk_delay);
        (*NPCInfo).squadState = SQUAD_RETREAT;
        let s2 = cstr("flee");
        let flee_delay = ctx.world.bg_state.rng.Q_irand(fleeTimeMin, fleeTimeMax);
        TIMER_Set(ctx, Some(npc_id), s2.as_ptr(), flee_delay);
        let s3 = cstr("panic");
        let panic_delay = ctx.world.bg_state.rng.Q_irand(1000, 4000);
        TIMER_Set(ctx, Some(npc_id), s3.as_ptr(), panic_delay);

        if (*npc_client).NPC_class != class_t::CLASS_PROTOCOL {
            let s4 = cstr("duck");
            TIMER_Set(ctx, Some(npc_id), s4.as_ptr(), 0);
        }
    }
}

/// Raven `G_StartFlee`.
///
/// Source: `oracle/codemp/game/NPC_behavior.c:1636-1648`
pub fn G_StartFlee(
    ctx: &mut GameContext,
    self_: EntityId,
    enemy: Option<EntityId>,
    dangerPoint: vec3_t,
    dangerLevel: c_int,
    fleeTimeMin: c_int,
    fleeTimeMax: c_int,
) {
    // `.NPC` is a raw pool pointer field; reading it and null-checking needs no
    // deref, so the whole bridge stays safe.
    if ctx.world.entity(self_).NPC.is_null() {
        // Player.
        return;
    }
    SaveNPCGlobals(ctx);
    SetNPCGlobals(ctx, self_);

    NPC_StartFlee(
        ctx,
        enemy,
        dangerPoint,
        dangerLevel,
        fleeTimeMin,
        fleeTimeMax,
    );

    RestoreNPCGlobals(ctx);
}

/// Raven `NPC_BSEmplaced`.
///
/// Source: `oracle/codemp/game/NPC_behavior.c:1650-1748`
pub fn NPC_BSEmplaced(ctx: &mut GameContext) {
    let NPC = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(NPC).unwrap();
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let NPCInfo = ctx.world.globals.NPCInfo;
    unsafe {
        let mut enemyLOS = qfalse;
        let mut enemyCS = qfalse;
        let mut faceEnemy = qfalse;
        let mut shoot = qfalse;
        let mut impactPos = [0.0f32; 3];

        if ctx.world.entity(npc_id).painDebounceTime > ctx.world.level.time {
            NPC_UpdateAngles(ctx, qtrue, qtrue);
            return;
        }

        if (*NPCInfo).scriptFlags & SCF_FIRE_WEAPON != 0 {
            WeaponThink(ctx, qtrue);
        }

        if NPC_CheckEnemyExt(ctx, qfalse) == qfalse {
            if ctx.world.bg_state.rng.Q_irand(0, 30) == 0 {
                let angle = ctx.world.entity(npc_id).s.angles[1];
                (*NPCInfo).desiredYaw =
                    angle as f32 + ctx.world.bg_state.rng.Q_irand(-90, 90) as f32;
            }
            if ctx.world.bg_state.rng.Q_irand(0, 30) == 0 {
                (*NPCInfo).desiredPitch = ctx.world.bg_state.rng.Q_irand(-20, 20) as f32;
            }
            NPC_UpdateAngles(ctx, qtrue, qtrue);
            return;
        }

        if let Some(enemy_id) = ctx.world.entity(npc_id).enemy {
            if NPC_ClearLOS4(ctx, Some(enemy_id)) != qfalse {
                enemyLOS = qtrue;

                let hit = NPC_ShotEntity(ctx, Some(enemy_id), Some(&mut impactPos));
                // Shot result index maps directly to the arena slot.
                let hit_id = EntityId(hit as u32);

                if hit == ctx.world.entity(enemy_id).s.number
                    || ctx.world.entity(hit_id).takedamage != 0
                {
                    enemyCS = qtrue;
                    NPC_AimAdjust(ctx, 2);
                    let enemy_origin = ctx.world.entity(enemy_id).r.currentOrigin;
                    _VectorCopy(enemy_origin, &mut (*NPCInfo).enemyLastSeenLocation);
                }
            }

            if enemyLOS != qfalse {
                faceEnemy = qtrue;
            }
            if enemyCS != qfalse {
                shoot = qtrue;
            }

            if faceEnemy != qfalse {
                NPC_FaceEnemy(ctx, qtrue);
            } else {
                NPC_UpdateAngles(ctx, qtrue, qtrue);
            }

            if (*NPCInfo).scriptFlags & SCF_DONT_FIRE != 0 {
                shoot = qfalse;
            }

            if let Some(enemy_enemy_id) = ctx.world.entity(enemy_id).enemy {
                if ctx.world.entity(enemy_id).s.weapon == WP_SABER as c_int
                    && ctx.world.entity(enemy_enemy_id).s.weapon == WP_SABER as c_int
                {
                    shoot = qfalse;
                }
            }
            if shoot != qfalse {
                if (*NPCInfo).scriptFlags & SCF_FIRE_WEAPON == 0 {
                    WeaponThink(ctx, qtrue);
                }
            }
        } else {
            NPC_UpdateAngles(ctx, qtrue, qtrue);
        }
    }
}
