// PORT-COMPLETE: NPC_behavior.c 21/21
//! FAITHFUL port of `oracle/oracle/codemp/game/NPC_behavior.c`.
//!
//! Landed from the `fnskel.py` signature skeleton; the pass-3 mega-pass fills
//! every remaining body against the settled fork rulings (ctx threading,
//! `Option<EntityId>` stored fields, bg/game state split). File-scope AI
//! globals (`NPC`, `NPCInfo`, `ucmd`, `level`, `g_entities`, `enemyVisibility`,
//! `showBBoxes`) reach through `ctx.world`/`ctx.world.globals` per ruling
//! 8/12.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use crate::ent_fn_enums::EntThink;
use crate::g_nav::NPC_SetMoveGoal;
use crate::NPC_combat::{
    G_ClearEnemy, G_SetEnemy, NPC_AimAdjust, NPC_CheckAttack, NPC_CheckEnemy,
    NPC_CheckGetNewWeapon, NPC_EnemyTooFar, NPC_FindCombatPoint, NPC_SetCombatPoint,
    NPC_ShotEntity, ValidEnemy, WeaponThink,
};
use crate::NPC_goal::{NPC_ClearGoal, UpdateGoal};
use crate::NPC_move::{NPC_MoveToGoal, NPC_SlideMoveToGoal};
use crate::NPC_utils::{
    CalcEntitySpot, G_ActivateBehavior, NPC_AimWiggle, NPC_CheckEnemyExt, NPC_ClearLOS4,
    NPC_FaceEnemy, NPC_SomeoneLookingAtMe, NPC_UpdateAngles, NPC_UpdateFiringAngles,
    NPC_UpdateShootAngles,
};
use crate::NPC_senses::{InFOV, NPC_CheckAlertEvents, NPC_CheckVisibility, NPC_GetHFOVPercentage};
use crate::NPC_sounds::G_AddVoiceEvent;
use crate::g_nav::{NAV_FindClosestWaypointForEnt, NAV_GetNearestNode};
use crate::g_timer::{TIMER_Done, TIMER_Set};
use crate::g_utils::{G_FreeEntity, G_UseTargets2};
use crate::npc::jump_state_t::jumpState_t::{self, JS_CROUCHING, JS_FACING, JS_JUMPING, JS_LANDING, JS_WAITING};
use crate::npc::spot_t::spot_t;
use crate::npc::visibility_t::visibility_t;
use crate::npc_c::{NPC_SetAnim, RestoreNPCGlobals, SaveNPCGlobals, SetNPCGlobals};
use crate::q_math::{
    _DotProduct, _VectorAdd, _VectorCopy, _VectorMA, _VectorScale, _VectorSubtract, AngleDelta,
    AngleNormalize360, AngleVectors, VectorCompare, VectorLength, VectorLengthSquared,
    VectorNormalize, vec3_origin, vectoangles,
};
use crate::bg_misc::vectoyaw;
use crate::bg_panimate::PM_InKnockDown;
use crate::teams::npcteam::NPCTEAM_ENEMY;
use mp_qshared::common::mp::entity_id::ent_id_opt;
use mp_qshared::shared::MASK_SHOT;

// Combat point search flags (per-file-scope-copy convention, matching the
// `NPC_AI_Stormtrooper.rs`/`NPC_combat.rs` precedent).
// Source: `oracle/oracle/codemp/game/b_local.h:244-260`
const CP_COVER: c_int = 0x0000_0001;
const CP_AVOID: c_int = 0x0000_0100;
const CP_HAS_ROUTE: c_int = 0x0000_1000;
const CP_NO_PVS: c_int = 0x0001_0000;

// Raven `MIN_ANGLE_ERROR`/`APEX_HEIGHT` (`NPC_behavior.c` file-scope consts
// used by `NPC_BSAdvanceFight`/`NPC_BSJump`).
// Source: `oracle/oracle/codemp/game/NPC_behavior.c` (top-of-file consts)
pub const MIN_ANGLE_ERROR: f32 = 4.0;
pub const APEX_HEIGHT: f32 = 30.0;

/// Raven `NPC_BSAdvanceFight`.
///
/// Source: `oracle/oracle/codemp/game/NPC_behavior.c:29-183`
pub fn NPC_BSAdvanceFight(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;
        let base = world.g_entities.as_ptr();

        // Make sure we're still headed where we want to capture.
        if let Some(captureGoal) = (*NPCInfo).captureGoal {
            let cap = &mut world.g_entities[captureGoal.index()] as *mut gentity_t;
            NPC_SetMoveGoal(ctx, NPC, (*cap).r.currentOrigin, 16, QTRUE, -1, core::ptr::null_mut());
            (*NPCInfo).goalTime = world.level.time + 100000;
        }

        NPC_CheckEnemy(ctx, QTRUE, QFALSE, QTRUE);

        if let Some(enemy_id) = (*NPC).enemy {
            let enemy = &mut world.g_entities[enemy_id.index()] as *mut gentity_t;
            let mut delta = [0.0f32; 3];
            let mut forward = [0.0f32; 3];
            let mut angleToEnemy = [0.0f32; 3];
            let mut hitspot = [0.0f32; 3];
            let mut muzzle = [0.0f32; 3];
            let mut diff = [0.0f32; 3];
            let mut enemy_org = [0.0f32; 3];
            let mut enemy_head = [0.0f32; 3];
            let distanceToEnemy: f32;
            let mut attack_ok = QFALSE;
            let mut dead_on = QFALSE;
            let mut attack_scale: f32 = 1.0;
            let max_aim_off: f32 = 64.0;

            _VectorMA((*enemy).r.absmin, 0.5, (*enemy).r.maxs, &mut enemy_org);
            CalcEntitySpot(ctx, NPC, spot_t::SPOT_WEAPON, &mut muzzle);

            _VectorSubtract(enemy_org, muzzle, &mut delta);
            vectoangles(delta, &mut angleToEnemy);
            distanceToEnemy = VectorNormalize(&mut delta);

            if NPC_EnemyTooFar(ctx, enemy, distanceToEnemy * distanceToEnemy, QTRUE) == QFALSE {
                attack_ok = QTRUE;
            }

            if attack_ok != QFALSE {
                NPC_UpdateShootAngles(ctx, angleToEnemy, QFALSE, QTRUE);

                (*NPCInfo).enemyLastVisibility = world.globals.enemyVisibility;
                let vis = NPC_CheckVisibility(ctx, enemy, CHECK_FOV);
                world.globals.enemyVisibility = vis;

                if vis == visibility_t::VIS_FOV {
                    attack_ok = QTRUE;
                    CalcEntitySpot(ctx, enemy, spot_t::SPOT_HEAD, &mut enemy_head);

                    if attack_ok != QFALSE {
                        let mut tr: trace_t = unsafe { core::mem::zeroed() };
                        trap::Trace(
    ctx.engine,
    GTraceArgs::new(
&mut tr as *mut trace_t,
                            &muzzle as *const vec3_t,
                            core::ptr::null(),
                            core::ptr::null(),
                            &enemy_org as *const vec3_t,
                            (*NPC).s.number,
                            MASK_SHOT,
    ),
);
                        let mut traceEnt = &mut world.g_entities[tr.entityNum as usize] as *mut gentity_t;
                        let npc_client = (*NPC).client as *mut gclient_t;
                        let enemy_client_id = ent_id_opt(base, traceEnt);
                        let trace_is_enemy = enemy_client_id == Some(enemy_id);
                        let trace_client = (*traceEnt).client as *mut gclient_t;
                        if !trace_is_enemy
                            && (traceEnt.is_null()
                                || trace_client.is_null()
                                || (*npc_client).enemyTeam == 0
                                || (*npc_client).enemyTeam != (*trace_client).playerTeam)
                        {
                            // No, so shoot for the head.
                            attack_scale *= 0.75;
                            trap::Trace(
    ctx.engine,
    GTraceArgs::new(
&mut tr as *mut trace_t,
                                &muzzle as *const vec3_t,
                                core::ptr::null(),
                                core::ptr::null(),
                                &enemy_head as *const vec3_t,
                                (*NPC).s.number,
                                MASK_SHOT,
    ),
);
                            traceEnt = &mut world.g_entities[tr.entityNum as usize] as *mut gentity_t;
                        }

                        _VectorCopy(tr.endpos, &mut hitspot);

                        let trace_client = (*traceEnt).client as *mut gclient_t;
                        let trace_is_enemy = ent_id_opt(base, traceEnt) == Some(enemy_id);
                        if trace_is_enemy
                            || (!trace_client.is_null()
                                && (*npc_client).enemyTeam != 0
                                && (*npc_client).enemyTeam == (*trace_client).playerTeam)
                        {
                            dead_on = QTRUE;
                        } else {
                            attack_scale *= 0.5;
                            if (*npc_client).playerTeam != 0 {
                                if !trace_client.is_null() && (*trace_client).playerTeam != 0 {
                                    if (*npc_client).playerTeam == (*trace_client).playerTeam {
                                        // Don't shoot our own team.
                                        attack_ok = QFALSE;
                                    }
                                }
                            }
                        }

                        if attack_ok != QFALSE {
                            // Adjust pitch aim.
                            _VectorSubtract(hitspot, muzzle, &mut delta);
                            vectoangles(delta, &mut angleToEnemy);
                            (*NPCInfo).desiredPitch = angleToEnemy[2];
                            NPC_UpdateShootAngles(ctx, angleToEnemy, QTRUE, QFALSE);

                            if dead_on == QFALSE {
                                // Suppressing fire.
                                AngleVectors((*NPCInfo).shootAngles, Some(&mut forward), None, None);
                                _VectorMA(muzzle, distanceToEnemy, forward, &mut hitspot);
                                _VectorSubtract(hitspot, enemy_org, &mut diff);
                                let mut aim_off = VectorLength(diff);
                                if aim_off > world.bg_state.rng.flrand(0.0, 1.0) * max_aim_off {
                                    attack_scale *= 0.75;
                                    _VectorSubtract(hitspot, enemy_head, &mut diff);
                                    aim_off = VectorLength(diff);
                                    if aim_off > world.bg_state.rng.flrand(0.0, 1.0) * max_aim_off {
                                        attack_ok = QFALSE;
                                    }
                                }
                                attack_scale *= (max_aim_off - aim_off + 1.0) / max_aim_off;
                            }
                        }
                    }
                }
            }

            if attack_ok != QFALSE {
                if NPC_CheckAttack(ctx, attack_scale) != QFALSE {
                    world.globals.enemyVisibility = visibility_t::VIS_SHOOT;
                    WeaponThink(ctx, QTRUE);
                } else {
                    attack_ok = QFALSE;
                }
            }
        } else {
            let client = (*NPC).client as *mut gclient_t;
            NPC_UpdateShootAngles(ctx, (*client).ps.viewangles, QTRUE, QTRUE);
        }

        if world.globals.ucmd.forwardmove == 0 && world.globals.ucmd.rightmove == 0 {
            // We reached our captureGoal.
            if trap::ICARUS_IsInitialized(ctx.engine, (*NPC).s.number) != 0 {
                trap::ICARUS_TaskIDComplete(ctx.engine, NPC, taskID_t::TID_BSTATE as c_int);
            }
        }
    }
}

/// Raven `Disappear`.
///
/// Source: `oracle/oracle/codemp/game/NPC_behavior.c:185-191`
pub fn Disappear(
    self_: *mut gentity_t,
) {
    unsafe {
        // ClientDisconnect(self); (Raven: commented out)
        (*self_).s.eFlags |= EF_NODRAW;
        (*self_).think = None;
        (*self_).nextthink = -1;
    }
}

/// Raven `BeamOut`.
///
/// Source: `oracle/oracle/codemp/game/NPC_behavior.c:194-211`
pub fn BeamOut(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    unsafe {
        let world = &mut *ctx.world;
        // fixme: doesn't actually go away!
        (*self_).nextthink = world.level.time + 1500;
        // fork ruling 2: fn-ptr field -> fn-ID enum (shape_mismatch: gentity_t.think's
        // declared type is still the raw `unsafe extern "C" fn` pointer in this
        // worktree, not `Option<EntThink>` — writing the ruling-2 assignment anyway
        // per LAW; see shape_mismatches in the port report).
        (*self_).think = Some(EntThink::Disappear);
        let client = (*self_).client as *mut gclient_t;
        (*client).squadname = core::ptr::null_mut();
        (*client).playerTeam = TEAM_FREE;
        (*self_).s.teamowner = TEAM_FREE as c_int;
        //self->r.svFlags |= SVF_BEAMING; //this appears unused in SP as well
    }
}

/// Raven `NPC_BSCinematic`.
///
/// Source: `oracle/oracle/codemp/game/NPC_behavior.c:213-244`
pub fn NPC_BSCinematic(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        if (*NPCInfo).scriptFlags & SCF_FIRE_WEAPON != 0 {
            WeaponThink(ctx, QTRUE);
        }

        if !UpdateGoal(ctx).is_null() {
            // Have a goalEntity.
            NPC_MoveToGoal(ctx, QTRUE);
        }

        if let Some(watch_id) = (*NPCInfo).watchTarget {
            // Have an entity which we want to keep facing.
            let watch = &mut world.g_entities[watch_id.index()] as *mut gentity_t;
            let mut eyes = [0.0f32; 3];
            let mut viewSpot = [0.0f32; 3];
            let mut viewvec = [0.0f32; 3];
            let mut viewangles = [0.0f32; 3];

            CalcEntitySpot(ctx, NPC, spot_t::SPOT_HEAD_LEAN, &mut eyes);
            CalcEntitySpot(ctx, watch, spot_t::SPOT_HEAD_LEAN, &mut viewSpot);

            _VectorSubtract(viewSpot, eyes, &mut viewvec);

            vectoangles(viewvec, &mut viewangles);

            (*NPCInfo).lockedDesiredYaw = viewangles[1];
            (*NPCInfo).desiredYaw = viewangles[1];
            (*NPCInfo).lockedDesiredPitch = viewangles[0];
            (*NPCInfo).desiredPitch = viewangles[0];
        }

        NPC_UpdateAngles(ctx, QTRUE, QTRUE);
    }
}

/// Raven `NPC_BSWait`.
///
/// Source: `oracle/oracle/codemp/game/NPC_behavior.c:246-249`
pub fn NPC_BSWait(ctx: GameContext<'_>) {
    NPC_UpdateAngles(ctx, QTRUE, QTRUE);
}

/// Raven `NPC_BSInvestigate`.
///
/// Raven: entire body is `/* ... */`-commented dead code (kept for reference)
/// — the live function is a no-op. Ported faithfully as a no-op.
/// Source: `oracle/oracle/codemp/game/NPC_behavior.c:252-407`
pub fn NPC_BSInvestigate() {
    // Raven's body is entirely commented out; this is a genuine no-op.
}

/// Raven `NPC_CheckInvestigate`.
///
/// Source: `oracle/oracle/codemp/game/NPC_behavior.c:409-494`
pub fn NPC_CheckInvestigate(
    ctx: GameContext<'_>,
    alertEventNum: c_int,
) -> qboolean {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;
        let base = world.g_entities.as_ptr();

        let owner = world.level.alertEvents[alertEventNum as usize].owner;
        let invAdd = world.level.alertEvents[alertEventNum as usize].level as c_int;
        let soundRad = world.level.alertEvents[alertEventNum as usize].radius;
        let earshot = (*NPCInfo).stats.earshot;

        let mut soundPos = [0.0f32; 3];
        _VectorCopy(world.level.alertEvents[alertEventNum as usize].position, &mut soundPos);

        // NOTE: Trying to preserve previous investigation behavior.
        if owner.is_null() {
            return QFALSE;
        }

        let owner_id = ent_id_opt(base, owner);
        if (*owner).s.eType != entityType_t::ET_PLAYER as c_int
            && (*owner).s.eType != entityType_t::ET_NPC as c_int
            && owner_id == (*NPCInfo).goalEntity
        {
            return QFALSE;
        }

        if (*owner).s.eFlags & EF_NODRAW != 0 {
            return QFALSE;
        }

        if (*owner).flags & FL_NOTARGET != 0 {
            return QFALSE;
        }

        if soundRad < earshot {
            return QFALSE;
        }

        if trap::InPVS(ctx.engine, soundPos, (*NPC).r.currentOrigin) == 0 {
            // Can hear through doors?
            return QFALSE;
        }

        let owner_client = (*owner).client as *mut gclient_t;
        let npc_client = (*NPC).client as *mut gclient_t;
        if !owner_client.is_null()
            && (*owner_client).playerTeam != 0
            && (*npc_client).playerTeam != 0
            && (*owner_client).playerTeam != (*npc_client).playerTeam
        {
            if (*NPCInfo).investigateCount as f32 >= ((*NPCInfo).stats.vigilance * 200.0) {
                // If investigateCount == 10, just take it as enemy and go.
                if ValidEnemy(ctx, owner) != QFALSE {
                    G_SetEnemy(ctx, NPC, owner);
                    (*NPCInfo).goalEntity = (*NPC).enemy;
                    (*NPCInfo).goalRadius = 12;
                    (*NPCInfo).behaviorState = BS_HUNT_AND_KILL;
                    return QTRUE;
                }
            } else {
                (*NPCInfo).investigateCount += invAdd;
            }
            // Run awakescript.
            G_ActivateBehavior(ctx, NPC, BSET_AWAKE as c_int);

            (*NPCInfo).eventOwner = owner_id;
            _VectorCopy(soundPos, &mut (*NPCInfo).investigateGoal);
            if (*NPCInfo).investigateCount > 20 {
                (*NPCInfo).investigateDebounceTime = world.level.time + 10000;
            } else {
                (*NPCInfo).investigateDebounceTime =
                    world.level.time + (*NPCInfo).investigateCount * 500;
            }
            (*NPCInfo).tempBehavior = BS_INVESTIGATE;
            return QTRUE;
        }

        QFALSE
    }
}

/// Raven `NPC_BSSleep`.
///
/// Source: `oracle/oracle/codemp/game/NPC_behavior.c:500-521`
pub fn NPC_BSSleep(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;

        let alertEvent = NPC_CheckAlertEvents(ctx, QTRUE, QFALSE, -1, QFALSE, AEL_MINOR as c_int);

        // There is an event to look at.
        if alertEvent >= 0 {
            G_ActivateBehavior(ctx, NPC, BSET_AWAKE as c_int);
            return;
        }
    }
}

/// Raven `NPC_BSFollowLeader`.
///
/// Source: `oracle/oracle/codemp/game/NPC_behavior.c:524-729`
pub fn NPC_BSFollowLeader(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;
        let base = world.g_entities.as_ptr();
        let npc_client = (*NPC).client as *mut gclient_t;

        let leader_id = (*npc_client).leader;
        if leader_id.is_none() {
            // Stand guard until we find an enemy.
            if (*NPCInfo).tempBehavior == BS_HUNT_AND_KILL {
                (*NPCInfo).tempBehavior = BS_DEFAULT;
            } else {
                (*NPCInfo).tempBehavior = BS_STAND_GUARD;
                crate::NPC_AI_Default::NPC_BSStandGuard(ctx);
            }
            return;
        }
        let leader = &mut world.g_entities[leader_id.unwrap().index()] as *mut gentity_t;

        if (*NPC).enemy.is_none() {
            // No enemy, find one.
            NPC_CheckEnemy(
                ctx,
                if (*NPCInfo).confusionTime < world.level.time { QTRUE } else { QFALSE },
                QFALSE,
                QTRUE,
            );
            if (*NPC).enemy.is_some() {
                (*NPCInfo).enemyCheckDebounceTime = world.level.time + world.bg_state.rng.Q_irand(3000, 10000);
            } else {
                if (*NPCInfo).scriptFlags & SCF_IGNORE_ALERTS == 0 {
                    let eventID = NPC_CheckAlertEvents(ctx, QTRUE, QTRUE, -1, QFALSE, AEL_MINOR as c_int);
                    if world.level.alertEvents[eventID as usize].level as c_int >= AEL_SUSPICIOUS as c_int
                        && ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0
                    {
                        (*NPCInfo).lastAlertID = world.level.alertEvents[eventID as usize].ID;
                        let ev_owner = world.level.alertEvents[eventID as usize].owner;
                        let ev_owner_client = if ev_owner.is_null() { core::ptr::null_mut() } else { (*ev_owner).client as *mut gclient_t };
                        if ev_owner.is_null()
                            || ev_owner_client.is_null()
                            || (*ev_owner).health <= 0
                            || (*ev_owner_client).playerTeam != (*npc_client).enemyTeam
                        {
                            // Not an enemy.
                        } else {
                            G_SetEnemy(ctx, NPC, ev_owner);
                            (*NPCInfo).enemyCheckDebounceTime = world.level.time + world.bg_state.rng.Q_irand(3000, 10000);
                            (*NPCInfo).enemyLastSeenTime = world.level.time;
                            let s = cstr("attackDelay");
                            TIMER_Set(ctx, NPC, s.as_ptr(), world.bg_state.rng.Q_irand(500, 1000));
                        }
                    }
                }
            }
            if (*NPC).enemy.is_none() {
                let l_client = (*leader).client as *mut gclient_t;
                if !leader_id.is_none()
                    && (*leader).enemy.is_some()
                    && (*leader).enemy != ent_id_opt(base, NPC)
                {
                    let l_enemy_id = (*leader).enemy.unwrap();
                    let l_enemy = &mut world.g_entities[l_enemy_id.index()] as *mut gentity_t;
                    let l_enemy_client = (*l_enemy).client as *mut gclient_t;
                    let allied_ok = !l_enemy_client.is_null() && (*l_enemy_client).playerTeam == (*npc_client).enemyTeam;
                    if allied_ok && (*l_enemy).health > 0 {
                        G_SetEnemy(ctx, NPC, l_enemy);
                        (*NPCInfo).enemyCheckDebounceTime = world.level.time + world.bg_state.rng.Q_irand(3000, 10000);
                        (*NPCInfo).enemyLastSeenTime = world.level.time;
                    }
                }
            }
        } else {
            let enemy_id = (*NPC).enemy.unwrap();
            let enemy = &mut world.g_entities[enemy_id.index()] as *mut gentity_t;
            if (*enemy).health <= 0 || ((*enemy).flags & FL_NOTARGET) != 0 {
                G_ClearEnemy(ctx, NPC);
                if (*NPCInfo).enemyCheckDebounceTime > world.level.time + 1000 {
                    (*NPCInfo).enemyCheckDebounceTime = world.level.time + world.bg_state.rng.Q_irand(1000, 2000);
                }
            } else if (*npc_client).ps.weapon != 0 && (*NPCInfo).enemyCheckDebounceTime < world.level.time {
                NPC_CheckEnemy(
                    ctx,
                    if (*NPCInfo).confusionTime < world.level.time || (*NPCInfo).tempBehavior != BS_FOLLOW_LEADER { QTRUE } else { QFALSE },
                    QFALSE,
                    QTRUE,
                );
            }
        }

        if (*NPC).enemy.is_some() && (*npc_client).ps.weapon != 0 {
            // If have an enemy, face him and fire.
            let enemy_id = (*NPC).enemy.unwrap();
            let enemy = &mut world.g_entities[enemy_id.index()] as *mut gentity_t;
            if (*npc_client).ps.weapon == WP_SABER as c_int {
                if (*NPCInfo).tempBehavior != BS_FOLLOW_LEADER {
                    (*NPCInfo).tempBehavior = BS_HUNT_AND_KILL;
                    NPC_UpdateAngles(ctx, QTRUE, QTRUE);
                    return;
                }
            }

            let vis = NPC_CheckVisibility(ctx, enemy, CHECK_FOV | CHECK_SHOOT);
            world.globals.enemyVisibility = vis;
            if (vis as c_int) > (visibility_t::VIS_PVS as c_int) {
                // Face.
                let mut enemy_org = [0.0f32; 3];
                let mut muzzle = [0.0f32; 3];
                let mut delta = [0.0f32; 3];
                let mut angleToEnemy = [0.0f32; 3];

                CalcEntitySpot(ctx, enemy, spot_t::SPOT_HEAD, &mut enemy_org);
                NPC_AimWiggle(ctx, &mut enemy_org);

                CalcEntitySpot(ctx, NPC, spot_t::SPOT_WEAPON, &mut muzzle);

                _VectorSubtract(enemy_org, muzzle, &mut delta);
                vectoangles(delta, &mut angleToEnemy);
                let distanceToEnemy = VectorNormalize(&mut delta);

                (*NPCInfo).desiredYaw = angleToEnemy[1];
                (*NPCInfo).desiredPitch = angleToEnemy[0];
                NPC_UpdateFiringAngles(ctx, QTRUE, QTRUE);

                if (vis as c_int) >= (visibility_t::VIS_SHOOT as c_int) {
                    NPC_AimAdjust(ctx, 2);
                    if NPC_GetHFOVPercentage(
                        (*enemy).r.currentOrigin,
                        (*NPC).r.currentOrigin,
                        (*npc_client).ps.viewangles,
                        (*NPCInfo).stats.hfov,
                    ) > 0.6
                        && NPC_GetHFOVPercentage(
                            (*enemy).r.currentOrigin,
                            (*NPC).r.currentOrigin,
                            (*npc_client).ps.viewangles,
                            (*NPCInfo).stats.vfov,
                        ) > 0.5
                    {
                        WeaponThink(ctx, QTRUE);
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

            CalcEntitySpot(ctx, leader, spot_t::SPOT_HEAD, &mut leaderHead);
            CalcEntitySpot(ctx, NPC, spot_t::SPOT_HEAD, &mut head);
            _VectorSubtract(leaderHead, head, &mut delta);
            vectoangles(delta, &mut angleToLeader);
            VectorNormalize(&mut delta);
            (*NPCInfo).desiredYaw = angleToLeader[1];
            (*NPCInfo).desiredPitch = angleToLeader[0];

            NPC_UpdateAngles(ctx, QTRUE, QTRUE);
        }

        // Leader visible?
        let leaderVis = NPC_CheckVisibility(ctx, leader, CHECK_PVS | CHECK_360 | CHECK_SHOOT);

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
            let walkdist = followDist * 0.83;
            let minrundist = followDist * 1.33;

            let mut vec = [0.0f32; 3];
            _VectorSubtract((*leader).r.currentOrigin, (*NPC).r.currentOrigin, &mut vec);
            let leaderDist = VectorLength(vec);
            vec[2] = 0.0;
            let leaderHDist = VectorLength(vec);
            if leaderHDist > backupdist
                && ((leaderVis as c_int) != (visibility_t::VIS_SHOOT as c_int) || leaderDist > walkdist)
            {
                (*NPCInfo).goalEntity = ent_id_opt(base, leader);
                NPC_SlideMoveToGoal(ctx);
                if (leaderVis as c_int) == (visibility_t::VIS_SHOOT as c_int) && leaderDist < minrundist {
                    world.globals.ucmd.buttons |= BUTTON_WALKING as c_int;
                }
            } else if leaderDist < backupdist {
                (*NPCInfo).goalEntity = ent_id_opt(base, leader);
                NPC_SlideMoveToGoal(ctx);

                world.globals.ucmd.forwardmove = -world.globals.ucmd.forwardmove;
                world.globals.ucmd.rightmove = -world.globals.ucmd.rightmove;
                _VectorScale((*npc_client).ps.moveDir, -1.0, &mut (*npc_client).ps.moveDir);
            }
            if world.globals.ucmd.forwardmove != 0
                || world.globals.ucmd.rightmove != 0
                || VectorCompare(vec3_origin, (*npc_client).ps.moveDir) != 0
            {
                crate::NPC_AI_Jedi::NPC_MoveDirClear(
                    ctx,
                    world.globals.ucmd.forwardmove as c_int,
                    world.globals.ucmd.rightmove as c_int,
                    QTRUE,
                );
            }
        }
    }
}

/// Raven `NPC_BSJump`.
///
/// Source: `oracle/oracle/codemp/game/NPC_behavior.c:733-919`
pub fn NPC_BSJump(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;
        let base = world.g_entities.as_ptr();
        let npc_client = (*NPC).client as *mut gclient_t;

        let Some(goal_id) = (*NPCInfo).goalEntity else {
            return;
        };
        let goal = &mut world.g_entities[goal_id.index()] as *mut gentity_t;

        if (*NPCInfo).jumpState != JS_JUMPING && (*NPCInfo).jumpState != JS_LANDING {
            // Face navgoal.
            let mut dir = [0.0f32; 3];
            let mut angles = [0.0f32; 3];
            _VectorSubtract((*goal).r.currentOrigin, (*NPC).r.currentOrigin, &mut dir);
            vectoangles(dir, &mut angles);
            (*NPCInfo).desiredPitch = AngleNormalize360(angles[0]);
            (*NPCInfo).lockedDesiredPitch = (*NPCInfo).desiredPitch;
            (*NPCInfo).desiredYaw = AngleNormalize360(angles[1]);
            (*NPCInfo).lockedDesiredYaw = (*NPCInfo).desiredYaw;
        }

        NPC_UpdateAngles(ctx, QTRUE, QTRUE);
        let yawError = AngleDelta((*npc_client).ps.viewangles[1], (*NPCInfo).desiredYaw);

        match (*NPCInfo).jumpState {
            jumpState_t::JS_FACING => {
                if yawError < MIN_ANGLE_ERROR {
                    NPC_SetAnim(NPC, SETANIM_LEGS, BOTH_CROUCH1 as c_int, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                    (*NPCInfo).jumpState = JS_CROUCHING;
                }
            }
            jumpState_t::JS_CROUCHING => {
                if (*npc_client).ps.legsTimer > 0 {
                    return;
                }

                let mut p1 = [0.0f32; 3];
                let mut p2 = [0.0f32; 3];
                if (*NPC).r.currentOrigin[2] > (*goal).r.currentOrigin[2] {
                    _VectorCopy((*NPC).r.currentOrigin, &mut p1);
                    _VectorCopy((*goal).r.currentOrigin, &mut p2);
                } else if (*NPC).r.currentOrigin[2] < (*goal).r.currentOrigin[2] {
                    _VectorCopy((*goal).r.currentOrigin, &mut p1);
                    _VectorCopy((*NPC).r.currentOrigin, &mut p2);
                } else {
                    _VectorCopy((*NPC).r.currentOrigin, &mut p1);
                    _VectorCopy((*goal).r.currentOrigin, &mut p2);
                }

                let mut dir = [0.0f32; 3];
                _VectorSubtract(p2, p1, &mut dir);
                dir[2] = 0.0;

                let mut xy = VectorNormalize(&mut dir);
                let mut z = p1[2] - p2[2];

                let apexHeight: f32 = APEX_HEIGHT / 2.0;

                z = (apexHeight + z).sqrt() - apexHeight.sqrt();
                debug_assert!(z >= 0.0);

                xy -= z;
                xy *= 0.5;
                debug_assert!(xy > 0.0);

                let mut apex = [0.0f32; 3];
                _VectorMA(p1, xy, dir, &mut apex);
                apex[2] += apexHeight;

                _VectorCopy(apex, &mut (*NPC).pos1);

                let height = apex[2] - (*NPC).r.currentOrigin[2];
                let time = (height / (0.5 * (*npc_client).ps.gravity as f32)).sqrt();
                if time == 0.0 {
                    return;
                }

                _VectorSubtract(apex, (*NPC).r.currentOrigin, &mut (*npc_client).ps.velocity);
                (*npc_client).ps.velocity[2] = 0.0;
                let dist = VectorNormalize(&mut (*npc_client).ps.velocity);

                let forward = dist / time;
                _VectorScale((*npc_client).ps.velocity, forward, &mut (*npc_client).ps.velocity);

                (*npc_client).ps.velocity[2] = time * (*npc_client).ps.gravity as f32;

                (*NPC).flags |= FL_NO_KNOCKBACK;
                (*NPCInfo).jumpState = JS_JUMPING;
            }
            jumpState_t::JS_JUMPING => {
                if world.globals.showBBoxes != 0 {
                    let mut p1 = [0.0f32; 3];
                    let mut p2 = [0.0f32; 3];
                    _VectorAdd((*NPC).r.mins, (*NPC).pos1, &mut p1);
                    _VectorAdd((*NPC).r.maxs, (*NPC).pos1, &mut p2);
                    crate::g_nav::G_Cube(p1, p2, [0.0, 0.0, 1.0], 0.5);
                }

                if (*NPC).s.groundEntityNum != ENTITYNUM_NONE {
                    // Landed, start landing anim.
                    (*npc_client).ps.velocity = [0.0; 3];
                    NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_LAND1 as c_int, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                    (*NPCInfo).jumpState = JS_LANDING;
                } else if (*npc_client).ps.legsTimer > 0 {
                    return;
                } else {
                    NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_INAIR1 as c_int, SETANIM_FLAG_OVERRIDE);
                }
            }
            jumpState_t::JS_LANDING => {
                if (*npc_client).ps.legsTimer > 0 {
                    return;
                } else {
                    (*NPCInfo).jumpState = JS_WAITING;

                    NPC_ClearGoal(ctx);
                    (*NPCInfo).goalTime = world.level.time;
                    (*NPCInfo).aiFlags &= !NPCAI_MOVING;
                    world.globals.ucmd.forwardmove = 0;
                    (*NPC).flags &= !FL_NO_KNOCKBACK;
                    trap::ICARUS_TaskIDComplete(ctx.engine, NPC, taskID_t::TID_MOVE_NAV as c_int);
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
/// Source: `oracle/oracle/codemp/game/NPC_behavior.c:921-937`
pub fn NPC_BSRemove(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;

        NPC_UpdateAngles(ctx, QTRUE, QTRUE);
        if trap::InPVS(ctx.engine, (*NPC).r.currentOrigin, world.g_entities[0].r.currentOrigin) == 0 {
            let target3 = (*NPC).target3;
            G_UseTargets2(ctx, NPC, NPC, target3);
            (*NPC).s.eFlags |= EF_NODRAW;
            (*NPC).s.eType = entityType_t::ET_INVISIBLE as c_int;
            (*NPC).r.contents = 0;
            (*NPC).health = 0;
            (*NPC).targetname = core::ptr::null_mut();

            // Disappear in half a second.
            // (shape mismatch, see BeamOut note above.)
            (*NPC).think = Some(EntThink::G_FreeEntity);
            (*NPC).nextthink = world.level.time + FRAMETIME;
        }
    }
}

/// Raven `NPC_BSSearch`.
///
/// Source: `oracle/oracle/codemp/game/NPC_behavior.c:939-1124`
pub fn NPC_BSSearch(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;
        let base = world.g_entities.as_ptr();

        NPC_CheckEnemy(ctx, QTRUE, QFALSE, QTRUE);
        if (*NPC).enemy.is_some() {
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
            let Some(tempGoal_id) = (*NPCInfo).tempGoal else { return };
            let tempGoal = &mut world.g_entities[tempGoal_id.index()] as *mut gentity_t;

            (*NPCInfo).goalEntity = (*NPCInfo).tempGoal;

            let mut vec = [0.0f32; 3];
            _VectorSubtract((*tempGoal).r.currentOrigin, (*NPC).r.currentOrigin, &mut vec);
            if vec[2] < 24.0 {
                vec[2] = 0.0;
            }

            if VectorLengthSquared(vec) < minGoalReachedDistSquared {
                // Close enough, just got there.
                (*NPC).waypoint = NAV_FindClosestWaypointForEnt(ctx, NPC, WAYPOINT_NONE);

                if (*NPCInfo).homeWp == WAYPOINT_NONE || (*NPC).waypoint == WAYPOINT_NONE {
                    if (*NPCInfo).tempBehavior == BS_SEARCH {
                        (*NPCInfo).tempBehavior = BS_DEFAULT;
                    } else {
                        (*NPCInfo).behaviorState = BS_STAND_GUARD;
                        crate::NPC_AI_Default::NPC_BSRunAndShoot(ctx);
                    }
                    return;
                }

                if (*NPC).waypoint == (*NPCInfo).homeWp {
                    if (*NPCInfo).aiFlags & NPCAI_ENROUTE_TO_HOMEWP != 0 {
                        (*NPCInfo).aiFlags &= !NPCAI_ENROUTE_TO_HOMEWP;
                        G_ActivateBehavior(ctx, NPC, BSET_LOSTENEMY as c_int);
                    }
                }

                if world.bg_state.rng.Q_irand(0, 1) == 0 {
                    NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_GUARD_LOOKAROUND1 as c_int, SETANIM_FLAG_NORMAL);
                } else {
                    NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_GUARD_IDLE1 as c_int, SETANIM_FLAG_NORMAL);
                }
                (*NPCInfo).investigateDebounceTime = world.level.time + world.bg_state.rng.Q_irand(3000, 10000);
            } else {
                NPC_MoveToGoal(ctx, QTRUE);
            }
        } else {
            if (*NPCInfo).investigateDebounceTime > world.level.time {
                if let Some(tempGoal_id) = (*NPCInfo).tempGoal {
                    let tempGoal = &mut world.g_entities[tempGoal_id.index()] as *mut gentity_t;
                    if (*tempGoal).waypoint != WAYPOINT_NONE {
                        if world.bg_state.rng.Q_irand(0, 30) == 0 {
                            let numEdges = trap::Nav_GetNodeNumEdges(ctx.engine, (*tempGoal).waypoint);
                            if numEdges != WAYPOINT_NONE {
                                let branchNum = world.bg_state.rng.Q_irand(0, numEdges - 1);
                                let mut branchPos = [0.0f32; 3];
                                let mut lookDir = [0.0f32; 3];
                                let nextWp = trap::Nav_GetNodeEdge(ctx.engine, (*tempGoal).waypoint, branchNum);
                                trap::Nav_GetNodePosition(ctx.engine, nextWp, &mut branchPos);
                                _VectorSubtract(branchPos, (*tempGoal).r.currentOrigin, &mut lookDir);
                                (*NPCInfo).desiredYaw = AngleNormalize360(vectoyaw(lookDir) + world.bg_state.rng.flrand(-45.0, 45.0));
                            }
                        }
                    }
                }
            } else {
                // Just finished waiting.
                (*NPC).waypoint = NAV_FindClosestWaypointForEnt(ctx, NPC, WAYPOINT_NONE);

                if let Some(tempGoal_id) = (*NPCInfo).tempGoal {
                    let tempGoal = &mut world.g_entities[tempGoal_id.index()] as *mut gentity_t;
                    if (*NPC).waypoint == (*NPCInfo).homeWp {
                        let numEdges = trap::Nav_GetNodeNumEdges(ctx.engine, (*tempGoal).waypoint);
                        if numEdges != WAYPOINT_NONE {
                            let branchNum = world.bg_state.rng.Q_irand(0, numEdges - 1);
                            let nextWp = trap::Nav_GetNodeEdge(ctx.engine, (*NPCInfo).homeWp, branchNum);
                            trap::Nav_GetNodePosition(ctx.engine, nextWp, &mut (*tempGoal).r.currentOrigin);
                            (*tempGoal).waypoint = nextWp;
                        }
                    } else {
                        trap::Nav_GetNodePosition(ctx.engine, (*NPCInfo).homeWp, &mut (*tempGoal).r.currentOrigin);
                        (*tempGoal).waypoint = (*NPCInfo).homeWp;
                    }

                    (*NPCInfo).investigateDebounceTime = 0;
                    (*NPCInfo).goalEntity = (*NPCInfo).tempGoal;
                    NPC_MoveToGoal(ctx, QTRUE);
                }
            }
        }

        NPC_UpdateAngles(ctx, QTRUE, QTRUE);
    }
}

/// Raven `NPC_BSSearchStart`.
///
/// Source: `oracle/oracle/codemp/game/NPC_behavior.c:1132-1150`
pub fn NPC_BSSearchStart(
    ctx: GameContext<'_>,
    homeWp: c_int,
    bState: bState_t,
) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;
        let mut homeWp = homeWp;

        if homeWp == WAYPOINT_NONE {
            homeWp = NAV_FindClosestWaypointForEnt(ctx, NPC, WAYPOINT_NONE);
            if (*NPC).waypoint == WAYPOINT_NONE {
                (*NPC).waypoint = homeWp;
            }
        }
        (*NPCInfo).homeWp = homeWp;
        (*NPCInfo).tempBehavior = bState;
        (*NPCInfo).aiFlags |= NPCAI_ENROUTE_TO_HOMEWP;
        (*NPCInfo).investigateDebounceTime = 0;
        if let Some(tempGoal_id) = (*NPCInfo).tempGoal {
            let tempGoal = &mut world.g_entities[tempGoal_id.index()] as *mut gentity_t;
            trap::Nav_GetNodePosition(ctx.engine, homeWp, &mut (*tempGoal).r.currentOrigin);
            (*tempGoal).waypoint = homeWp;
        }
    }
}

/// Raven `NPC_BSNoClip`.
///
/// Source: `oracle/oracle/codemp/game/NPC_behavior.c:1160-1191`
pub fn NPC_BSNoClip(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        if !UpdateGoal(ctx).is_null() {
            // PORT-NOTE(goal-invariant): Raven's `if (UpdateGoal())` implies
            // `NPCInfo->goalEntity` is non-NULL on this branch; `NPC_UpdateAngles`
            // still runs at the tail either way, matching Raven's fall-through.
            let goal_id = (*NPCInfo).goalEntity.expect("UpdateGoal() set goalEntity");
            let goal = &mut world.g_entities[goal_id.index()] as *mut gentity_t;
            let mut dir = [0.0f32; 3];
            let mut forward = [0.0f32; 3];
            let mut right = [0.0f32; 3];
            let mut angles = [0.0f32; 3];
            let up = [0.0f32, 0.0, 1.0];

            _VectorSubtract((*goal).r.currentOrigin, (*NPC).r.currentOrigin, &mut dir);

            vectoangles(dir, &mut angles);
            (*NPCInfo).desiredYaw = angles[1];

            AngleVectors((*NPC).r.currentAngles, Some(&mut forward), Some(&mut right), None);

            VectorNormalize(&mut dir);

            let fDot = _DotProduct(forward, dir) * 127.0;
            let rDot = _DotProduct(right, dir) * 127.0;
            let uDot = _DotProduct(up, dir) * 127.0;

            world.globals.ucmd.forwardmove = fDot.floor() as i8;
            world.globals.ucmd.rightmove = rDot.floor() as i8;
            world.globals.ucmd.upmove = uDot.floor() as i8;
        } else {
            let npc_client = (*NPC).client as *mut gclient_t;
            (*npc_client).ps.velocity = [0.0; 3];
        }

        NPC_UpdateAngles(ctx, QTRUE, QTRUE);
    }
}

/// Raven `NPC_BSWander`.
///
/// Source: `oracle/oracle/codemp/game/NPC_behavior.c:1193-1286`
pub fn NPC_BSWander(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        if (*NPCInfo).investigateDebounceTime == 0 {
            let mut minGoalReachedDistSquared: f32 = 64.0;
            let Some(tempGoal_id) = (*NPCInfo).tempGoal else { return };
            let tempGoal = &mut world.g_entities[tempGoal_id.index()] as *mut gentity_t;

            (*NPCInfo).goalEntity = (*NPCInfo).tempGoal;

            let mut vec = [0.0f32; 3];
            _VectorSubtract((*tempGoal).r.currentOrigin, (*NPC).r.currentOrigin, &mut vec);

            if (*tempGoal).waypoint != WAYPOINT_NONE {
                minGoalReachedDistSquared = 64.0;
            }

            if VectorLengthSquared(vec) < minGoalReachedDistSquared {
                (*NPC).waypoint = NAV_FindClosestWaypointForEnt(ctx, NPC, WAYPOINT_NONE);

                if world.bg_state.rng.Q_irand(0, 1) == 0 {
                    NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_GUARD_LOOKAROUND1 as c_int, SETANIM_FLAG_NORMAL);
                } else {
                    NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_GUARD_IDLE1 as c_int, SETANIM_FLAG_NORMAL);
                }
                (*NPCInfo).investigateDebounceTime = world.level.time + world.bg_state.rng.Q_irand(3000, 10000);
            } else {
                NPC_MoveToGoal(ctx, QTRUE);
            }
        } else {
            if (*NPCInfo).investigateDebounceTime > world.level.time {
                if let Some(tempGoal_id) = (*NPCInfo).tempGoal {
                    let tempGoal = &mut world.g_entities[tempGoal_id.index()] as *mut gentity_t;
                    if (*tempGoal).waypoint != WAYPOINT_NONE {
                        if world.bg_state.rng.Q_irand(0, 30) == 0 {
                            let numEdges = trap::Nav_GetNodeNumEdges(ctx.engine, (*tempGoal).waypoint);
                            if numEdges != WAYPOINT_NONE {
                                let branchNum = world.bg_state.rng.Q_irand(0, numEdges - 1);
                                let mut branchPos = [0.0f32; 3];
                                let mut lookDir = [0.0f32; 3];
                                let nextWp = trap::Nav_GetNodeEdge(ctx.engine, (*tempGoal).waypoint, branchNum);
                                trap::Nav_GetNodePosition(ctx.engine, nextWp, &mut branchPos);
                                _VectorSubtract(branchPos, (*tempGoal).r.currentOrigin, &mut lookDir);
                                (*NPCInfo).desiredYaw = AngleNormalize360(vectoyaw(lookDir) + world.bg_state.rng.flrand(-45.0, 45.0));
                            }
                        }
                    }
                }
            } else {
                (*NPC).waypoint = NAV_FindClosestWaypointForEnt(ctx, NPC, WAYPOINT_NONE);

                if (*NPC).waypoint != WAYPOINT_NONE {
                    let numEdges = trap::Nav_GetNodeNumEdges(ctx.engine, (*NPC).waypoint);
                    if numEdges != WAYPOINT_NONE {
                        if let Some(tempGoal_id) = (*NPCInfo).tempGoal {
                            let tempGoal = &mut world.g_entities[tempGoal_id.index()] as *mut gentity_t;
                            let branchNum = world.bg_state.rng.Q_irand(0, numEdges - 1);
                            let nextWp = trap::Nav_GetNodeEdge(ctx.engine, (*NPC).waypoint, branchNum);
                            trap::Nav_GetNodePosition(ctx.engine, nextWp, &mut (*tempGoal).r.currentOrigin);
                            (*tempGoal).waypoint = nextWp;
                        }

                        (*NPCInfo).investigateDebounceTime = 0;
                        (*NPCInfo).goalEntity = (*NPCInfo).tempGoal;
                        NPC_MoveToGoal(ctx, QTRUE);
                    }
                }
            }
        }

        NPC_UpdateAngles(ctx, QTRUE, QTRUE);
    }
}

/// Raven `NPC_Surrender`.
///
/// Source: `oracle/oracle/codemp/game/NPC_behavior.c:1317-1339`
pub fn NPC_Surrender(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;
        let npc_client = (*NPC).client as *mut gclient_t;

        if (*npc_client).ps.weaponTime != 0 || PM_InKnockDown(&mut (*npc_client).ps) != 0 {
            return;
        }
        if (*NPC).s.weapon != WP_NONE as c_int
            && (*NPC).s.weapon != WP_STUN_BATON as c_int
            && (*NPC).s.weapon != WP_SABER as c_int
        {
            //WP_DropWeapon( NPC, NULL ); //rwwFIXMEFIXME
        }
        if (*NPCInfo).surrenderTime < world.level.time - 5000 {
            (*NPCInfo).blockedSpeechDebounceTime = 0;
            G_AddVoiceEvent(ctx, NPC, world.bg_state.rng.Q_irand(EV_PUSHED1 as c_int, EV_PUSHED3 as c_int), 3000);
        }
        (*NPCInfo).surrenderTime = world.level.time + 1000;
    }
}

/// Raven `NPC_CheckSurrender`.
///
/// Source: `oracle/oracle/codemp/game/NPC_behavior.c:1341-1442`
pub fn NPC_CheckSurrender(ctx: GameContext<'_>) -> qboolean {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let npc_client = (*NPC).client as *mut gclient_t;

        let Some(enemy_id) = (*NPC).enemy else { return QFALSE };
        let enemy = &mut world.g_entities[enemy_id.index()] as *mut gentity_t;
        let enemy_client = (*enemy).client as *mut gclient_t;

        if trap::ICARUS_TaskIDPending(ctx.engine, NPC, taskID_t::TID_MOVE_NAV as c_int) == 0
            && (*npc_client).ps.groundEntityNum != ENTITYNUM_NONE
            && (*npc_client).ps.weaponTime == 0
            && PM_InKnockDown(&mut (*npc_client).ps) == 0
            && !enemy_client.is_null()
            && (*enemy).enemy == ent_id_opt(world.g_entities.as_ptr(), NPC)
            && (*enemy).s.weapon != WP_NONE as c_int
            && (*enemy).s.weapon != WP_STUN_BATON as c_int
            && (*enemy).health > 20
            && (*enemy).painDebounceTime < world.level.time - 3000
            && (*enemy_client).ps.fd.forcePowerDebounce[FP_SABER_DEFENSE as usize] < world.level.time - 1000
        {
            if (*NPC).s.weapon != WP_ROCKET_LAUNCHER as c_int
                && (*NPC).s.weapon != WP_REPEATER as c_int
                && (*NPC).s.weapon != WP_FLECHETTE as c_int
                && (*NPC).s.weapon != WP_SABER as c_int
            {
                if (*NPC).s.weapon != WP_NONE as c_int {
                    if (*NPC).health > 25 {
                        return QFALSE;
                    }
                    if NPC_SomeoneLookingAtMe(ctx, NPC) != QFALSE && (*NPC).painDebounceTime > world.level.time {
                        // Fall through.
                    } else {
                        if InFOV(ctx, enemy, NPC, 60, 30) == QFALSE {
                            return QFALSE;
                        } else if crate::q_math::DistanceSquared((*NPC).r.currentOrigin, (*enemy).r.currentOrigin) < 65536.0 {
                            return QFALSE;
                        } else if trap::InPVS(ctx.engine, (*NPC).r.currentOrigin, (*enemy).r.currentOrigin) == 0 {
                            return QFALSE;
                        }
                    }
                }
            }
        }
        QFALSE
    }
}

/// Raven `NPC_BSFlee`.
///
/// Source: `oracle/oracle/codemp/game/NPC_behavior.c:1444-1558`
pub fn NPC_BSFlee(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;
        let base = world.g_entities.as_ptr();

        let flee_s = cstr("flee");
        if TIMER_Done(ctx, NPC, flee_s.as_ptr()) != 0 && (*NPCInfo).tempBehavior == BS_FLEE {
            (*NPCInfo).tempBehavior = BS_DEFAULT;
            (*NPCInfo).squadState = SQUAD_IDLE;
        }
        if NPC_CheckSurrender(ctx) != QFALSE {
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
            let goal = &mut world.g_entities[goal_id.index()] as *mut gentity_t;
            let mut moved;
            let mut reverseCourse = QTRUE;

            if (*NPC).waypoint == WAYPOINT_NONE {
                (*NPC).waypoint = NAV_GetNearestNode(ctx, NPC, (*NPC).lastWaypoint);
            }
            if (*NPC).waypoint != WAYPOINT_NONE {
                let numEdges = trap::Nav_GetNodeNumEdges(ctx.engine, (*NPC).waypoint);

                if numEdges != WAYPOINT_NONE {
                    let mut dangerDir = [0.0f32; 3];
                    _VectorSubtract((*NPCInfo).investigateGoal, (*NPC).r.currentOrigin, &mut dangerDir);
                    VectorNormalize(&mut dangerDir);

                    for branchNum in 0..numEdges {
                        let mut branchPos = [0.0f32; 3];
                        let mut runDir = [0.0f32; 3];

                        let nextWp = trap::Nav_GetNodeEdge(ctx.engine, (*NPC).waypoint, branchNum);
                        trap::Nav_GetNodePosition(ctx.engine, nextWp, &mut branchPos);

                        _VectorSubtract(branchPos, (*NPC).r.currentOrigin, &mut runDir);
                        VectorNormalize(&mut runDir);
                        if _DotProduct(runDir, dangerDir) > world.bg_state.rng.flrand(0.0, 0.5) {
                            continue;
                        }
                        NPC_SetMoveGoal(ctx, NPC, branchPos, 0, QTRUE, -1, core::ptr::null_mut());
                        reverseCourse = QFALSE;
                        break;
                    }
                }
            }

            moved = NPC_MoveToGoal(ctx, QFALSE);

            if (*NPC).s.weapon == WP_NONE as c_int && (moved == QFALSE || reverseCourse != QFALSE) {
                NPC_Surrender(ctx);
                NPC_UpdateAngles(ctx, QTRUE, QTRUE);
                return;
            }
            if moved == QFALSE {
                let mut dir = [0.0f32; 3];
                if reverseCourse != QFALSE {
                    _VectorSubtract((*NPC).r.currentOrigin, (*goal).r.currentOrigin, &mut dir);
                } else {
                    _VectorSubtract((*goal).r.currentOrigin, (*NPC).r.currentOrigin, &mut dir);
                }
                let dist = VectorNormalize(&mut dir);
                (*NPCInfo).distToGoal = dist;
                (*NPCInfo).desiredYaw = vectoyaw(dir);
                (*NPCInfo).desiredPitch = 0.0;
                world.globals.ucmd.forwardmove = 127;
            } else if reverseCourse != QFALSE {
                (*NPCInfo).desiredYaw *= -1.0;
            }
            world.globals.ucmd.buttons &= !(BUTTON_WALKING as c_int);
        }
        NPC_UpdateAngles(ctx, QTRUE, QTRUE);

        NPC_CheckGetNewWeapon(ctx);
    }
}

/// Raven `NPC_StartFlee`.
///
/// Source: `oracle/oracle/codemp/game/NPC_behavior.c:1560-1634`
pub fn NPC_StartFlee(
    ctx: GameContext<'_>,
    enemy: *mut gentity_t,
    dangerPoint: vec3_t,
    dangerLevel: c_int,
    fleeTimeMin: c_int,
    fleeTimeMax: c_int,
) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;
        let npc_client = (*NPC).client as *mut gclient_t;
        let mut cp: c_int = -1;

        if trap::ICARUS_TaskIDPending(ctx.engine, NPC, taskID_t::TID_MOVE_NAV as c_int) != 0 {
            return;
        }

        if G_ActivateBehavior(ctx, NPC, BSET_FLEE as c_int) != 0 {
            return;
        }
        if !enemy.is_null() {
            G_SetEnemy(ctx, NPC, enemy);
        }

        if dangerLevel > AEL_DANGER as c_int
            || (*NPC).s.weapon == WP_NONE as c_int
            || ((*NPCInfo).group.is_null() && (*NPC).health <= 10)
        {
            cp = NPC_FindCombatPoint(
                ctx,
                (*NPC).r.currentOrigin,
                (*NPC).r.currentOrigin,
                dangerPoint,
                CP_COVER | CP_AVOID | CP_HAS_ROUTE | CP_NO_PVS,
                128.0,
                -1,
            );
        }
        if cp == -1 {
            cp = NPC_FindCombatPoint(
                ctx,
                (*NPC).r.currentOrigin,
                (*NPC).r.currentOrigin,
                dangerPoint,
                CP_COVER | CP_AVOID | CP_HAS_ROUTE,
                128.0,
                -1,
            );
            if cp == -1 {
                cp = NPC_FindCombatPoint(
                    ctx,
                    (*NPC).r.currentOrigin,
                    (*NPC).r.currentOrigin,
                    dangerPoint,
                    CP_COVER | CP_HAS_ROUTE,
                    128.0,
                    -1,
                );
                if cp == -1 {
                    cp = NPC_FindCombatPoint(
                        ctx,
                        (*NPC).r.currentOrigin,
                        (*NPC).r.currentOrigin,
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
            NPC_SetMoveGoal(ctx, NPC, world.level.combatPoints[cp as usize].origin, 8, QTRUE, cp, core::ptr::null_mut());
            (*NPCInfo).behaviorState = BS_HUNT_AND_KILL;
            (*NPCInfo).tempBehavior = BS_DEFAULT;
        } else {
            if (*NPC).s.weapon != WP_NONE as c_int {
                return;
            } else {
                (*NPCInfo).tempBehavior = BS_FLEE;
                NPC_SetMoveGoal(ctx, NPC, dangerPoint, 0, QTRUE, -1, core::ptr::null_mut());
                _VectorCopy(dangerPoint, &mut (*NPCInfo).investigateGoal);
            }
        }
        let s = cstr("attackDelay");
        TIMER_Set(ctx, NPC, s.as_ptr(), world.bg_state.rng.Q_irand(500, 2500));
        (*NPCInfo).squadState = SQUAD_RETREAT;
        let s2 = cstr("flee");
        TIMER_Set(ctx, NPC, s2.as_ptr(), world.bg_state.rng.Q_irand(fleeTimeMin, fleeTimeMax));
        let s3 = cstr("panic");
        TIMER_Set(ctx, NPC, s3.as_ptr(), world.bg_state.rng.Q_irand(1000, 4000));

        if (*npc_client).NPC_class != class_t::CLASS_PROTOCOL {
            let s4 = cstr("duck");
            TIMER_Set(ctx, NPC, s4.as_ptr(), 0);
        }
    }
}

/// Raven `G_StartFlee`.
///
/// Source: `oracle/oracle/codemp/game/NPC_behavior.c:1636-1648`
pub fn G_StartFlee(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    enemy: *mut gentity_t,
    dangerPoint: vec3_t,
    dangerLevel: c_int,
    fleeTimeMin: c_int,
    fleeTimeMax: c_int,
) {
    unsafe {
        if (*self_).NPC.is_null() {
            // Player.
            return;
        }
        SaveNPCGlobals(ctx);
        SetNPCGlobals(ctx, self_);

        NPC_StartFlee(ctx, enemy, dangerPoint, dangerLevel, fleeTimeMin, fleeTimeMax);

        RestoreNPCGlobals(ctx);
    }
}

/// Raven `NPC_BSEmplaced`.
///
/// Source: `oracle/oracle/codemp/game/NPC_behavior.c:1650-1748`
pub fn NPC_BSEmplaced(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        let mut enemyLOS = QFALSE;
        let mut enemyCS = QFALSE;
        let mut faceEnemy = QFALSE;
        let mut shoot = QFALSE;
        let mut impactPos = [0.0f32; 3];

        if (*NPC).painDebounceTime > world.level.time {
            NPC_UpdateAngles(ctx, QTRUE, QTRUE);
            return;
        }

        if (*NPCInfo).scriptFlags & SCF_FIRE_WEAPON != 0 {
            WeaponThink(ctx, QTRUE);
        }

        if NPC_CheckEnemyExt(ctx, QFALSE) == QFALSE {
            if world.bg_state.rng.Q_irand(0, 30) == 0 {
                (*NPCInfo).desiredYaw = (*NPC).s.angles[1] as f32 + world.bg_state.rng.Q_irand(-90, 90) as f32;
            }
            if world.bg_state.rng.Q_irand(0, 30) == 0 {
                (*NPCInfo).desiredPitch = world.bg_state.rng.Q_irand(-20, 20) as f32;
            }
            NPC_UpdateAngles(ctx, QTRUE, QTRUE);
            return;
        }

        if let Some(enemy_id) = (*NPC).enemy {
            let enemy = &mut world.g_entities[enemy_id.index()] as *mut gentity_t;
            if NPC_ClearLOS4(ctx, enemy) != QFALSE {
                enemyLOS = QTRUE;

                let hit = NPC_ShotEntity(ctx, enemy, &mut impactPos);
                let hitEnt = &mut world.g_entities[hit as usize] as *mut gentity_t;

                if hit == (*enemy).s.number || (!hitEnt.is_null() && (*hitEnt).takedamage != 0) {
                    enemyCS = QTRUE;
                    NPC_AimAdjust(ctx, 2);
                    _VectorCopy((*enemy).r.currentOrigin, &mut (*NPCInfo).enemyLastSeenLocation);
                }
            }

            if enemyLOS != QFALSE {
                faceEnemy = QTRUE;
            }
            if enemyCS != QFALSE {
                shoot = QTRUE;
            }

            if faceEnemy != QFALSE {
                NPC_FaceEnemy(ctx, QTRUE);
            } else {
                NPC_UpdateAngles(ctx, QTRUE, QTRUE);
            }

            if (*NPCInfo).scriptFlags & SCF_DONT_FIRE != 0 {
                shoot = QFALSE;
            }

            if (*enemy).enemy.is_some() {
                let enemy_enemy_id = (*enemy).enemy.unwrap();
                let enemy_enemy = &mut world.g_entities[enemy_enemy_id.index()] as *mut gentity_t;
                if (*enemy).s.weapon == WP_SABER as c_int && (*enemy_enemy).s.weapon == WP_SABER as c_int {
                    shoot = QFALSE;
                }
            }
            if shoot != QFALSE {
                if (*NPCInfo).scriptFlags & SCF_FIRE_WEAPON == 0 {
                    WeaponThink(ctx, QTRUE);
                }
            }
        } else {
            NPC_UpdateAngles(ctx, QTRUE, QTRUE);
        }
    }
}
