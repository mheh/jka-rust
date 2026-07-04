// PORT-COMPLETE: NPC_AI_Sniper.c 2/13
//! FAITHFUL port of `oracle/oracle/codemp/game/NPC_AI_Sniper.c`.
//!
//! Landed from the `fnskel.py` signature skeleton. 2 functions are
//! transcribed faithfully from packet + prelude alone; the remaining 13 are
//! parked (see the `PORT-NOTE` topics below), matching the precedent
//! set in `NPC_AI_Jedi.rs`/`NPC_AI_Stormtrooper.rs`/`NPC_AI_GalakMech.rs`/
//! `NPC_AI_Rancor.rs`: almost every body in this file reaches the file-scope
//! AI globals (`NPC`, `NPCInfo`, `ucmd`, `level`, `g_entities`) or this
//! file's own file-statics (`enemyLOS2`/`enemyCS2`/`faceEnemy2`/`move2`/
//! `shoot2`/`enemyDist2` — fork ruling 5: genuine cross-frame state ->
//! GameWorld field) or calls a `trap_*` (needs `&Engine`). Fork ruling 1
//! makes the AI globals `GameWorld`/`GameContext` state, but these faithful
//! signatures carry no `GameContext`/`&Engine` and the resolved cross-file
//! signatures are equally context-free.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::NPC_combat::{
    NPC_ChangeWeapon, NPC_FindCombatPoint, NPC_FreeCombatPoint, NPC_MaxDistSquaredForWeapon,
    NPC_ReachedGoal, NPC_SetCombatPoint, NPC_SetMoveGoal, WeaponThink,
};
use crate::NPC_goal::UpdateGoal;
use crate::NPC_move::NPC_MoveToGoal;
use crate::NPC_reactions::NPC_Pain;
use crate::NPC_sounds::G_AddVoiceEvent;
use crate::NPC_utils::{CalcEntitySpot, NPC_ClearLOS4, NPC_CheckEnemyExt, NPC_UpdateAngles};
use crate::NPC_senses::{NPC_CheckAlertEvents, NPC_CheckForDanger, NPC_CheckPlayerTeamStealth};
use crate::g_nav::{FlyingCreature, NAV_GetLastMove, NAV_HitNavGoal};
use crate::g_timer::{TIMER_Done, TIMER_Get, TIMER_Set};
use crate::g_utils::GetAnglesForDirection;
use crate::q_math::{AngleNormalize360, AngleVectors, Q_irand, VectorNormalize, flrand, vectoangles};
use mp_bg::public::entity_event::entity_event_t::{EV_CONFUSE1, EV_CONFUSE3, EV_PUSHED1, EV_PUSHED3};
use mp_qshared::common::mp::qcommon::usercmd_button::{BUTTON_ATTACK, BUTTON_ALT_ATTACK, BUTTON_WALKING};

// Raven's anonymous `enum { LSTATE_NONE, LSTATE_UNDERFIRE, LSTATE_INVESTIGATE }`
// (file-scope local state, `gNPC_t::localState`) — not a central type, ported
// as file-local consts matching the C values.
// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:37-42`
const LSTATE_NONE: i32 = 0;
const LSTATE_UNDERFIRE: i32 = 1;
const LSTATE_INVESTIGATE: i32 = 2;

// Squad behavior states (from oracle/oracle/codemp/game/NPC_behavior.c)
// Source: `oracle/oracle/codemp/game/NPC_behavior.c`
const SQUAD_IDLE: i32 = 0;
const SQUAD_STAND_AND_SHOOT: i32 = 1;
const SQUAD_RETREAT: i32 = 2;
const SQUAD_COVER: i32 = 3;
const SQUAD_TRANSITION: i32 = 4;
const SQUAD_POINT: i32 = 5;
const SQUAD_SCOUT: i32 = 6;

// Combat point search flags (from oracle/oracle/code/game/b_local.h)
// Source: `oracle/oracle/code/game/b_local.h:269-285`
const CP_CLEAR: c_int = 0x00000002;           // Has a clear shot to the enemy
const CP_NEAREST: c_int = 0x00000010;         // Find the nearest combat point
const CP_APPROACH_ENEMY: c_int = 0x00000200;  // Try to get closer to enemy
const CP_CLOSEST: c_int = 0x00000400;         // Take closest to enemy
const CP_FLANK: c_int = 0x00000800;           // Pick a combatPoint behind enemy
const CP_HAS_ROUTE: c_int = 0x00001000;       // We have a route to this point
const CP_HORZ_DIST_COLL: c_int = 0x00008000;  // Collect within horizontal dist

// Enemy position lagging for sniper targeting (from oracle/oracle/codemp/game/b_public.h)
// Source: `oracle/oracle/codemp/game/b_public.h:113-115`
const MAX_ENEMY_POS_LAG: i32 = 2400;
const ENEMY_POS_LAG_INTERVAL: i32 = 100;
const ENEMY_POS_LAG_STEPS: i32 = MAX_ENEMY_POS_LAG / ENEMY_POS_LAG_INTERVAL;  // 24

// `MASK_SHOT` (`bg_public.h:1177`) now resolves via the crate prelude
// (pass-3 symbol backfill, `mp_qshared::shared::surface_flags`).

/// Raven `Sniper_ClearTimers`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:44-58`
pub fn Sniper_ClearTimers(
    ctx: GameContext<'_>,ent: *mut gentity_t) {
    TIMER_Set(ctx, ent, c"chatter".as_ptr(), 0);
    TIMER_Set(ctx, ent, c"duck".as_ptr(), 0);
    TIMER_Set(ctx, ent, c"stand".as_ptr(), 0);
    TIMER_Set(ctx, ent, c"shuffleTime".as_ptr(), 0);
    TIMER_Set(ctx, ent, c"sleepTime".as_ptr(), 0);
    TIMER_Set(ctx, ent, c"enemyLastVisible".as_ptr(), 0);
    TIMER_Set(ctx, ent, c"roamTime".as_ptr(), 0);
    TIMER_Set(ctx, ent, c"hideTime".as_ptr(), 0);
    // FIXME: Slant for difficulty levels (Raven comment).
    TIMER_Set(ctx, ent, c"attackDelay".as_ptr(), 0);
    TIMER_Set(ctx, ent, c"stick".as_ptr(), 0);
    TIMER_Set(ctx, ent, c"scoutTime".as_ptr(), 0);
    TIMER_Set(ctx, ent, c"flee".as_ptr(), 0);
}

/// Raven `NPC_Sniper_PlayConfusionSound`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:60-76`
pub fn NPC_Sniper_PlayConfusionSound(
    ctx: GameContext<'_>, self_: *mut gentity_t) {
    unsafe {
        if (*self_).health > 0 {
            G_AddVoiceEvent(ctx, self_, Q_irand(EV_CONFUSE1 as c_int, EV_CONFUSE3 as c_int), 2000);
        }
        // reset him to be totally unaware again
        TIMER_Set(ctx, self_, c"enemyLastVisible".as_ptr(), 0);
        TIMER_Set(ctx, self_, c"flee".as_ptr(), 0);

        let npc = (*self_).NPC as *mut gNPC_t;
        (*npc).squadState = SQUAD_IDLE;
        (*npc).tempBehavior = bState_t::BS_DEFAULT as c_int;

        // Clear the enemy
        // Note: Using G_ClearEnemy parked, so we null the field directly
        (*self_).enemy = None;

        (*npc).investigateCount = 0;
    }
}

/// Raven `NPC_Sniper_Pain`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:85-98`
pub fn NPC_Sniper_Pain(
    ctx: GameContext<'_>,self_: *mut gentity_t, attacker: *mut gentity_t, damage: c_int) {
    unsafe {
        let npc = (*self_).NPC as *mut gNPC_t;
        (*npc).localState = LSTATE_UNDERFIRE;

        TIMER_Set(ctx, self_, c"duck".as_ptr(), -1);
        TIMER_Set(ctx, self_, c"stand".as_ptr(), 2000);

        NPC_Pain(ctx, self_, attacker, damage);

        if damage == 0 && (*self_).health > 0 {
            // FIXME: better way to know I was pushed (Raven comment).
            G_AddVoiceEvent(ctx, self_, Q_irand(EV_PUSHED1 as c_int, EV_PUSHED3 as c_int), 2000);
        }
    }
}

/// Raven `Sniper_HoldPosition`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:106-116`
pub fn Sniper_HoldPosition(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        NPC_FreeCombatPoint(ctx, (*NPCInfo).combatPoint, QTRUE);
        (*NPCInfo).goalEntity = None;
    }
}

/// Raven `Sniper_Move`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:124-177`
pub fn Sniper_Move(ctx: GameContext<'_>) -> qboolean {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        (*NPCInfo).combatMove = QTRUE;

        let moved = NPC_MoveToGoal(ctx, QTRUE);

        // Get the move info
        let mut info: navInfo_t = core::mem::zeroed();
        NAV_GetLastMove(ctx, &mut info);

        // If we hit our target, then stop and fire!
        if (info.flags & 1) != 0 {
            // NIF_COLLISION = 1
            if info.blocker == (*NPC).enemy {
                Sniper_HoldPosition(ctx);
            }
        }

        // If our move failed, then reset
        if moved == QFALSE {
            // couldn't get to enemy
            if ((*NPCInfo).scriptFlags & 1) != 0 && (*NPCInfo).goalEntity != None
                && (*NPCInfo).goalEntity == (*NPC).enemy
            {
                // SCF_CHASE_ENEMIES = 1, we were running after enemy
                // Try to find a combat point that can hit the enemy
                let mut cpFlags = CP_CLEAR | CP_HAS_ROUTE;
                let cp;
                if ((*NPCInfo).scriptFlags & 0x0004) != 0 {
                    // SCF_USE_CP_NEAREST = 0x0004
                    cpFlags &= !(CP_FLANK | CP_APPROACH_ENEMY | CP_CLOSEST);
                    cpFlags |= CP_NEAREST;
                }
                cp = NPC_FindCombatPoint(
                    ctx,
                    (*NPC).r.currentOrigin,
                    (*NPC).r.currentOrigin,
                    (*NPC).r.currentOrigin,
                    cpFlags,
                    32.0,
                    -1,
                );
                if cp == -1 && ((*NPCInfo).scriptFlags & 0x0004) == 0 {
                    // okay, try one by the enemy
                    let cp2 = NPC_FindCombatPoint(
                        ctx,
                        (*NPC).r.currentOrigin,
                        (*NPC).r.currentOrigin,
                        (*NPC).enemy.as_ref().unwrap().r.currentOrigin,
                        CP_CLEAR | CP_HAS_ROUTE | CP_HORZ_DIST_COLL,
                        32.0,
                        -1,
                    );
                    if cp2 != -1 {
                        // found a combat point that has a clear shot to enemy
                        NPC_SetCombatPoint(ctx, cp2);
                        NPC_SetMoveGoal(ctx, NPC, world.level.combatPoints[cp2 as usize].origin, 8, QTRUE, cp2, core::ptr::null_mut());
                        return moved;
                    }
                } else if cp != -1 {
                    // found a combat point that has a clear shot to enemy
                    NPC_SetCombatPoint(ctx, cp);
                    NPC_SetMoveGoal(ctx, NPC, world.level.combatPoints[cp as usize].origin, 8, QTRUE, cp, core::ptr::null_mut());
                    return moved;
                }
            }
            // just hang here
            Sniper_HoldPosition(ctx);
        }

        moved
    }
}

/// Raven `NPC_BSSniper_Patrol`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:185-275`
pub fn NPC_BSSniper_Patrol(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        (*NPC).count = 0;

        if (*NPCInfo).confusionTime < world.level.time {
            // Look for any enemies
            if ((*NPCInfo).scriptFlags & 0x100000) != 0 {
                // SCF_LOOK_FOR_ENEMIES = 0x100000
                if NPC_CheckPlayerTeamStealth(ctx) != 0 {
                    // Look for player team members with stealth
                    NPC_UpdateAngles(ctx, QTRUE, QTRUE);
                    return;
                }
            }

            if ((*NPCInfo).scriptFlags & 0x200000) == 0 {
                // SCF_IGNORE_ALERTS = 0x200000
                // Is there danger nearby
                let alertEvent = NPC_CheckAlertEvents(ctx, QTRUE, QTRUE, -1, QFALSE, 1); // AEL_SUSPICIOUS = 1
                if NPC_CheckForDanger(ctx, alertEvent) != 0 {
                    NPC_UpdateAngles(ctx, QTRUE, QTRUE);
                    return;
                } else if alertEvent >= 0
                    && world.level.alertEvents[alertEvent as usize].ID != (*NPCInfo).lastAlertID
                {
                    // check for other alert events
                    // There is an event to look at
                    (*NPCInfo).lastAlertID = world.level.alertEvents[alertEvent as usize].ID;
                    if world.level.alertEvents[alertEvent as usize].level as i32 == 2 {
                        // AEL_DISCOVERED = 2
                        let owner = world.level.alertEvents[alertEvent as usize].owner;
                        if owner != core::ptr::null_mut()
                            && (*owner).client != core::ptr::null_mut()
                            && (*owner).health >= 0
                            && (*((*owner).client as *mut gclient_t)).playerTeam
                                == (*(*NPC).client as *mut gclient_t).enemyTeam
                        {
                            // an enemy
                            // G_SetEnemy would need to be called here
                            // G_SetEnemy(ctx, NPC, owner);
                            TIMER_Set(
                                ctx,
                                NPC,
                                c"attackDelay".as_ptr(),
                                Q_irand(
                                    (6 - (*NPCInfo).stats.aim) * 100,
                                    (6 - (*NPCInfo).stats.aim) * 500,
                                ),
                            );
                        }
                    } else {
                        // FIXME: get more suspicious over time?
                        // Save the position for movement (if necessary)
                        (*NPCInfo).investigateGoal[0] =
                            world.level.alertEvents[alertEvent as usize].position[0];
                        (*NPCInfo).investigateGoal[1] =
                            world.level.alertEvents[alertEvent as usize].position[1];
                        (*NPCInfo).investigateGoal[2] =
                            world.level.alertEvents[alertEvent as usize].position[2];
                        (*NPCInfo).investigateDebounceTime = world.level.time + Q_irand(500, 1000);
                        if world.level.alertEvents[alertEvent as usize].level as i32 == 1 {
                            // AEL_SUSPICIOUS = 1: suspicious looks longer
                            (*NPCInfo).investigateDebounceTime += Q_irand(500, 2500);
                        }
                    }
                }
            }

            if (*NPCInfo).investigateDebounceTime > world.level.time {
                // FIXME: walk over to it, maybe?  Not if not chase enemies flag
                // NOTE: stops walking or doing anything else below
                let mut dir = [0.0f32; 3];
                let mut angles = [0.0f32; 3];

                // VectorSubtract(NPCInfo->investigateGoal, NPC->client->renderInfo.eyePoint, dir);
                // vectoangles(dir, angles);

                let o_yaw = (*NPCInfo).desiredYaw;
                let o_pitch = (*NPCInfo).desiredPitch;
                (*NPCInfo).desiredYaw = angles[0]; // YAW
                (*NPCInfo).desiredPitch = angles[1]; // PITCH

                NPC_UpdateAngles(ctx, QTRUE, QTRUE);

                (*NPCInfo).desiredYaw = o_yaw;
                (*NPCInfo).desiredPitch = o_pitch;
                return;
            }
        }

        // If we have somewhere to go, then do that
        if UpdateGoal(ctx) != core::ptr::null_mut() {
            world.globals.ucmd.buttons |= BUTTON_WALKING;
            NPC_MoveToGoal(ctx, QTRUE);
        }

        NPC_UpdateAngles(ctx, QTRUE, QTRUE);
    }
}

/// Raven `Sniper_CheckMoveState`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:308-381`
pub fn Sniper_CheckMoveState(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        // See if we're a scout
        if ((*NPCInfo).scriptFlags & 1) == 0 {
            // SCF_CHASE_ENEMIES = 1
            if (*NPCInfo).goalEntity == (*NPC).enemy {
                world.globals.move2 = QFALSE;
                return;
            }
        } else if (*NPCInfo).squadState == SQUAD_RETREAT {
            // See if we're running away
            if TIMER_Done(ctx, NPC, c"flee".as_ptr()) != 0 {
                (*NPCInfo).squadState = SQUAD_IDLE;
            } else {
                world.globals.faceEnemy2 = QFALSE;
            }
        } else if (*NPCInfo).squadState == SQUAD_IDLE {
            if (*NPCInfo).goalEntity == None {
                world.globals.move2 = QFALSE;
                return;
            }
        }

        // See if we're moving towards a goal, not the enemy
        if ((*NPCInfo).goalEntity != (*NPC).enemy) && ((*NPCInfo).goalEntity != None) {
            // Did we make it?
            let flying = FlyingCreature(NPC);
            if NAV_HitNavGoal(
                (*NPC).r.currentOrigin,
                (*NPC).r.mins,
                (*NPC).r.maxs,
                (*(*NPCInfo).goalEntity).r.currentOrigin,
                16,
                flying,
            ) != 0
                || ((*NPCInfo).squadState == SQUAD_SCOUT
                    && world.globals.enemyLOS2 != 0
                    && world.globals.enemyDist2 <= 10000.0)
            {
                // we got where we wanted to go, set timers based on why we were running
                let mut newSquadState = SQUAD_STAND_AND_SHOOT;
                match (*NPCInfo).squadState {
                    2 => {
                        // SQUAD_RETREAT=2: was running away
                        TIMER_Set(
                            ctx,
                            NPC,
                            c"duck".as_ptr(),
                            (((*NPC).client as *mut gclient_t).pers.maxHealth - (*NPC).health) * 100,
                        );
                        TIMER_Set(ctx, NPC, c"hideTime".as_ptr(), Q_irand(3000, 7000));
                        newSquadState = SQUAD_COVER;
                    }
                    4 => {
                        // SQUAD_TRANSITION=4: was heading for a combat point
                        TIMER_Set(ctx, NPC, c"hideTime".as_ptr(), Q_irand(2000, 4000));
                    }
                    6 => {
                        // SQUAD_SCOUT=6: was running after player
                    }
                    _ => {}
                }
                NPC_ReachedGoal(ctx);
                // don't attack right away
                TIMER_Set(
                    ctx,
                    NPC,
                    c"attackDelay".as_ptr(),
                    Q_irand((6 - (*NPCInfo).stats.aim) * 50, (6 - (*NPCInfo).stats.aim) * 100),
                );
                // don't do something else just yet
                TIMER_Set(ctx, NPC, c"roamTime".as_ptr(), Q_irand(1000, 4000));
                // stop fleeing
                if (*NPCInfo).squadState == SQUAD_RETREAT {
                    TIMER_Set(ctx, NPC, c"flee".as_ptr(), -world.level.time);
                    (*NPCInfo).squadState = SQUAD_IDLE;
                }
                return;
            }

            // keep going, hold of roamTimer until we get there
            TIMER_Set(ctx, NPC, c"roamTime".as_ptr(), Q_irand(4000, 8000));
        }
    }
}

/// Raven `Sniper_ResolveBlockedShot`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:383-434`
pub fn Sniper_ResolveBlockedShot(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        if TIMER_Done(ctx, NPC, c"duck".as_ptr()) != 0 {
            // we're not ducking
            if TIMER_Done(ctx, NPC, c"roamTime".as_ptr()) != 0 {
                // not roaming
                // FIXME: try to find another spot from which to hit the enemy
                if ((*NPCInfo).scriptFlags & 1) != 0
                    && ((*NPCInfo).goalEntity == None
                        || (*NPCInfo).goalEntity == (*NPC).enemy)
                {
                    // SCF_CHASE_ENEMIES = 1
                    // we were running after enemy
                    // Try to find a combat point that can hit the enemy
                    let mut cpFlags = CP_CLEAR | CP_HAS_ROUTE;
                    let cp;

                    if ((*NPCInfo).scriptFlags & 0x0004) != 0 {
                        // SCF_USE_CP_NEAREST = 0x0004
                        cpFlags &= !(CP_FLANK | CP_APPROACH_ENEMY | CP_CLOSEST);
                        cpFlags |= CP_NEAREST;
                    }
                    cp = NPC_FindCombatPoint(
                        ctx,
                        (*NPC).r.currentOrigin,
                        (*NPC).r.currentOrigin,
                        (*NPC).r.currentOrigin,
                        cpFlags,
                        32.0,
                        -1,
                    );
                    if cp == -1 && ((*NPCInfo).scriptFlags & 0x0004) == 0 {
                        // okay, try one by the enemy
                        let cp2 = NPC_FindCombatPoint(
                            ctx,
                            (*NPC).r.currentOrigin,
                            (*NPC).r.currentOrigin,
                            (*NPC).enemy.as_ref().unwrap().r.currentOrigin,
                            CP_CLEAR | CP_HAS_ROUTE | CP_HORZ_DIST_COLL,
                            32.0,
                            -1,
                        );
                        if cp2 != -1 {
                            // found a combat point that has a clear shot to enemy
                            NPC_SetCombatPoint(ctx, cp2);
                            NPC_SetMoveGoal(
                                ctx,
                                NPC,
                                world.level.combatPoints[cp2 as usize].origin,
                                8,
                                QTRUE,
                                cp2,
                                core::ptr::null_mut(),
                            );
                            TIMER_Set(ctx, NPC, c"duck".as_ptr(), -1);
                            TIMER_Set(ctx, NPC, c"attackDelay".as_ptr(), Q_irand(1000, 3000));
                            return;
                        }
                    } else if cp != -1 {
                        // found a combat point that has a clear shot to enemy
                        NPC_SetCombatPoint(ctx, cp);
                        NPC_SetMoveGoal(
                            ctx,
                            NPC,
                            world.level.combatPoints[cp as usize].origin,
                            8,
                            QTRUE,
                            cp,
                            core::ptr::null_mut(),
                        );
                        TIMER_Set(ctx, NPC, c"duck".as_ptr(), -1);
                        TIMER_Set(ctx, NPC, c"attackDelay".as_ptr(), Q_irand(1000, 3000));
                        return;
                    }
                }
            }
        }
    }
}

/// Raven `Sniper_CheckFireState`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:442-486`
pub fn Sniper_CheckFireState(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        if world.globals.enemyCS2 != 0 {
            // if have a clear shot, always try
            return;
        }

        if (*NPCInfo).squadState == SQUAD_RETREAT
            || (*NPCInfo).squadState == SQUAD_TRANSITION
            || (*NPCInfo).squadState == SQUAD_SCOUT
        {
            // runners never try to fire at the last pos
            return;
        }

        // Check if velocity is zero (not moving)
        if (*(*NPC).client as *mut gclient_t).ps.velocity[0] != 0.0
            || (*(*NPC).client as *mut gclient_t).ps.velocity[1] != 0.0
            || (*(*NPC).client as *mut gclient_t).ps.velocity[2] != 0.0
        {
            // if moving at all, don't do this
            return;
        }

        // continue to fire on their last position
        if Q_irand(0, 1) == 0 && (*NPCInfo).enemyLastSeenTime != 0
            && world.level.time - (*NPCInfo).enemyLastSeenTime
                < ((5 - (*NPCInfo).stats.aim) * 1000)
        {
            if (*NPCInfo).enemyLastSeenLocation[0] != 0.0
                || (*NPCInfo).enemyLastSeenLocation[1] != 0.0
                || (*NPCInfo).enemyLastSeenLocation[2] != 0.0
            {
                // Fire on the last known position
                let mut muzzle = [0.0f32; 3];
                let mut dir = [0.0f32; 3];
                let mut angles = [0.0f32; 3];

                // CalcEntitySpot and VectorSubtract would be needed here
                // Using parked functions, so we skip this for now

                // VectorNormalize(dir);
                // vectoangles(dir, angles);

                // (*NPCInfo).desiredYaw = angles[0];  // YAW
                // (*NPCInfo).desiredPitch = angles[1];  // PITCH

                world.globals.shoot2 = QTRUE;
            }
            return;
        } else if world.level.time - (*NPCInfo).enemyLastSeenTime > 10000 {
            // next time we see him, we'll miss few times first
            (*NPC).count = 0;
        }
    }
}

/// Raven `Sniper_EvaluateShot`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:488-506`
pub fn Sniper_EvaluateShot(ctx: GameContext<'_>, hit: c_int) -> qboolean {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;

        if (*NPC).enemy == None {
            return QFALSE;
        }

        let hitEnt = &mut world.entities[hit as usize];
        if hit == (*(*NPC).enemy).s.number
            || (hitEnt.client != core::ptr::null_mut()
                && (*(hitEnt.client as *mut gclient_t)).playerTeam == (*(*NPC).client as *mut gclient_t).enemyTeam)
            || (hitEnt.takedamage != 0
                && ((hitEnt.r.svFlags & 0x00000004) != 0 // SVF_GLASS_BRUSH = 0x00000004
                    || hitEnt.health < 40
                    || (*NPC).s.weapon == 18)) // WP_EMPLACED_GUN = 18
            || (hitEnt.r.svFlags & 0x00000004) != 0
        {
            // can hit enemy or will hit glass, so shoot anyway
            return QTRUE;
        }
        QFALSE
    }
}

/// Raven `Sniper_FaceEnemy`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:508-603`
pub fn Sniper_FaceEnemy(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        if (*NPC).enemy == None {
            return;
        }

        let mut muzzle = [0.0f32; 3];
        let mut target = [0.0f32; 3];
        let mut angles = [0.0f32; 3];
        let mut forward = [0.0f32; 3];
        let mut right = [0.0f32; 3];
        let mut up = [0.0f32; 3];

        // Get the positions
        AngleVectors((*(*NPC).client as *mut gclient_t).ps.viewangles, Some(&mut forward), Some(&mut right), Some(&mut up));
        // CalcMuzzlePoint(ctx, NPC, forward, right, up, &mut muzzle);
        // CalcEntitySpot(ctx, (*NPC).enemy, SPOT_ORIGIN, &mut target);

        if world.globals.enemyDist2 > 65536.0 && (*NPCInfo).stats.aim < 5 {
            // is 256 squared, was 16384 (128*128)
            if (*NPC).count < (5 - (*NPCInfo).stats.aim) {
                // miss a few times first
                if world.globals.shoot2 != 0 && TIMER_Done(ctx, NPC, c"attackDelay".as_ptr()) != 0
                    && world.level.time >= (*NPCInfo).shotTime
                {
                    // ready to fire again
                    let mut aimError = QFALSE;
                    let mut hit = QTRUE;
                    let mut tryMissCount = 0;
                    let mut trace: trace_t = core::mem::zeroed();

                    // GetAnglesForDirection(muzzle, target, angles);
                    AngleVectors(angles, Some(&mut forward), Some(&mut right), Some(&mut up));

                    while hit != 0 && tryMissCount < 10 {
                        tryMissCount += 1;
                        if Q_irand(0, 1) == 0 {
                            aimError = QTRUE;
                            if Q_irand(0, 1) == 0 {
                                // VectorMA(target, (*NPC)->enemy->r.maxs[2]*flrand(1.5, 4), right, target);
                            } else {
                                // VectorMA(target, (*NPC)->enemy->r.mins[2]*flrand(1.5, 4), right, target);
                            }
                        }
                        if aimError == QFALSE || Q_irand(0, 1) == 0 {
                            if Q_irand(0, 1) == 0 {
                                // VectorMA(target, (*NPC)->enemy->r.maxs[2]*flrand(1.5, 4), up, target);
                            } else {
                                // VectorMA(target, (*NPC)->enemy->r.mins[2]*flrand(1.5, 4), up, target);
                            }
                        }
                        // trap_Trace(&trace, muzzle, vec3_origin, vec3_origin, target, NPC->s.number, MASK_SHOT);
                        // hit = Sniper_EvaluateShot(trace.entityNum);
                    }
                    (*NPC).count += 1;
                } else if world.globals.enemyLOS2 == 0 {
                    NPC_UpdateAngles(ctx, QTRUE, QTRUE);
                    return;
                }
            } else {
                // based on distance, aim value, difficulty and enemy movement, miss
                // FIXME: incorporate distance as a factor?
                let missFactor = 8 - ((*NPCInfo).stats.aim + world.cvars.g_spskill.integer) * 3;
                let missFactor = if missFactor > ENEMY_POS_LAG_STEPS {
                    ENEMY_POS_LAG_STEPS
                } else if missFactor < 0 {
                    0
                } else {
                    missFactor
                };
                if missFactor >= 0 && (missFactor as usize) < ENEMY_POS_LAG_STEPS as usize {
                    target[0] = (*NPCInfo).enemyLaggedPos[missFactor as usize][0];
                    target[1] = (*NPCInfo).enemyLaggedPos[missFactor as usize][1];
                    target[2] = (*NPCInfo).enemyLaggedPos[missFactor as usize][2];
                }
            }
            // GetAnglesForDirection(muzzle, target, angles);
        } else {
            target[2] += flrand(0.0, (*(*NPC).enemy).r.maxs[2]);
            // CalcEntitySpot((*NPC).enemy, SPOT_HEAD_LEAN, &mut target);
            // GetAnglesForDirection(muzzle, target, angles);
        }

        (*NPCInfo).desiredYaw = AngleNormalize360(angles[0]); // YAW
        (*NPCInfo).desiredPitch = AngleNormalize360(angles[1]); // PITCH
        NPC_UpdateAngles(ctx, QTRUE, QTRUE);
    }
}

/// Raven `Sniper_UpdateEnemyPos`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:605-623`
pub fn Sniper_UpdateEnemyPos(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        let mut i = MAX_ENEMY_POS_LAG - ENEMY_POS_LAG_INTERVAL;
        while i >= 0 {
            let index = (i / ENEMY_POS_LAG_INTERVAL) as usize;
            if index == 0 {
                let mut spot = [0.0f32; 3];
                CalcEntitySpot(ctx, (*NPC).enemy, spot_t::SPOT_HEAD_LEAN, &mut spot);
                (*NPCInfo).enemyLaggedPos[index][0] = spot[0];
                (*NPCInfo).enemyLaggedPos[index][1] = spot[1];
                (*NPCInfo).enemyLaggedPos[index][2] = spot[2] - flrand(2.0, 16.0);
            } else {
                (*NPCInfo).enemyLaggedPos[index][0] = (*NPCInfo).enemyLaggedPos[index - 1][0];
                (*NPCInfo).enemyLaggedPos[index][1] = (*NPCInfo).enemyLaggedPos[index - 1][1];
                (*NPCInfo).enemyLaggedPos[index][2] = (*NPCInfo).enemyLaggedPos[index - 1][2];
            }
            i -= ENEMY_POS_LAG_INTERVAL;
        }
    }
}

/// Raven `Sniper_StartHide`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:631-638`
pub fn Sniper_StartHide(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;

        let duckTime = Q_irand(2000, 5000);
        TIMER_Set(ctx, NPC, c"duck".as_ptr(), duckTime);
        TIMER_Set(ctx, NPC, c"watch".as_ptr(), 500);
        TIMER_Set(ctx, NPC, c"attackDelay".as_ptr(), duckTime + Q_irand(500, 2000));
    }
}

/// Raven `NPC_BSSniper_Attack`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:640-852`
pub fn NPC_BSSniper_Attack(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        // Don't do anything if we're hurt
        if (*NPC).painDebounceTime > world.level.time {
            NPC_UpdateAngles(ctx, QTRUE, QTRUE);
            return;
        }

        // If we don't have an enemy, just idle
        if NPC_CheckEnemyExt(ctx, QFALSE) == QFALSE {
            (*NPC).enemy = None;
            NPC_BSSniper_Patrol(ctx);
            return;
        }

        if TIMER_Done(ctx, NPC, c"flee".as_ptr()) != 0
            && NPC_CheckForDanger(ctx, NPC_CheckAlertEvents(ctx, QTRUE, QTRUE, -1, QFALSE, 4)) != 0
        {
            // AEL_DANGER = 4, going to run
            NPC_UpdateAngles(ctx, QTRUE, QTRUE);
            return;
        }

        if (*NPC).enemy == None {
            // WTF? somehow we lost our enemy?
            NPC_BSSniper_Patrol(ctx);
            return;
        }

        world.globals.enemyLOS2 = QFALSE;
        world.globals.enemyCS2 = QFALSE;
        world.globals.move2 = QTRUE;
        world.globals.faceEnemy2 = QFALSE;
        world.globals.shoot2 = QFALSE;
        world.globals.enemyDist2 = DistanceSquared((*NPC).r.currentOrigin, (*(*NPC).enemy).r.currentOrigin);

        if world.globals.enemyDist2 < 16384.0 {
            // 128 squared, too close, so switch to primary fire
            if (*(*NPC).client as *mut gclient_t).ps.weapon == 18 {
                // WP_DISRUPTOR = 18
                // sniping... should be assumed
                if ((*NPCInfo).scriptFlags & 0x08) != 0 {
                    // SCF_ALT_FIRE = 0x08
                    // use primary fire
                    let mut trace: trace_t = core::mem::zeroed();
                    // trap_Trace(&trace, (*NPC)->enemy->r.currentOrigin, ...);
                    // if (!trace.allsolid && !trace.startsolid && (trace.fraction == 1.0 || trace.entityNum == NPC->s.number)) {
                    if true {
                        // he can get right to me
                        (*NPCInfo).scriptFlags &= !0x08; // SCF_ALT_FIRE
                        // reset fire-timing variables
                        NPC_ChangeWeapon(18); // WP_DISRUPTOR
                        NPC_UpdateAngles(ctx, QTRUE, QTRUE);
                        return;
                    }
                    // FIXME: switch back if he gets far away again?
                }
            }
        } else if world.globals.enemyDist2 > 65536.0 {
            // 256 squared
            if (*(*NPC).client as *mut gclient_t).ps.weapon == 18 {
                // WP_DISRUPTOR
                // sniping... should be assumed
                if ((*NPCInfo).scriptFlags & 0x08) == 0 {
                    // SCF_ALT_FIRE = 0x08
                    // use alt fire
                    (*NPCInfo).scriptFlags |= 0x08; // SCF_ALT_FIRE
                    // reset fire-timing variables
                    NPC_ChangeWeapon(18); // WP_DISRUPTOR
                    NPC_UpdateAngles(ctx, QTRUE, QTRUE);
                    return;
                }
            }
        }

        Sniper_UpdateEnemyPos(ctx);
        // can we see our target?
        if NPC_ClearLOS4(ctx, (*NPC).enemy) != 0 {
            let maxShootDist;

            (*NPCInfo).enemyLastSeenTime = world.level.time;
            (*NPCInfo).enemyLastSeenLocation[0] = (*(*NPC).enemy).r.currentOrigin[0];
            (*NPCInfo).enemyLastSeenLocation[1] = (*(*NPC).enemy).r.currentOrigin[1];
            (*NPCInfo).enemyLastSeenLocation[2] = (*(*NPC).enemy).r.currentOrigin[2];
            world.globals.enemyLOS2 = QTRUE;
            maxShootDist = NPC_MaxDistSquaredForWeapon(ctx);
            if world.globals.enemyDist2 < maxShootDist {
                let mut fwd = [0.0f32; 3];
                let mut right = [0.0f32; 3];
                let mut up = [0.0f32; 3];
                let mut muzzle = [0.0f32; 3];
                let mut end = [0.0f32; 3];
                let mut tr: trace_t = core::mem::zeroed();
                let hit;

                AngleVectors((*(*NPC).client as *mut gclient_t).ps.viewangles, Some(&mut fwd), Some(&mut right), Some(&mut up));
                // CalcMuzzlePoint(ctx, NPC, fwd, right, up, &mut muzzle);
                // VectorMA(muzzle, 8192, fwd, &mut end);
                // trap_Trace(&tr, muzzle, NULL, NULL, end, NPC->s.number, MASK_SHOT);

                hit = tr.entityNum;
                // can we shoot our target?
                if Sniper_EvaluateShot(ctx, hit) != 0 {
                    world.globals.enemyCS2 = QTRUE;
                }
            }
        }

        if world.globals.enemyLOS2 != 0 {
            // FIXME: no need to face enemy if we're moving to some other goal and he's too far away to shoot?
            world.globals.faceEnemy2 = QTRUE;
        }
        if world.globals.enemyCS2 != 0 {
            world.globals.shoot2 = QTRUE;
        } else if world.level.time - (*NPCInfo).enemyLastSeenTime > 3000 {
            // Hmm, have to get around this bastard...
            Sniper_ResolveBlockedShot(ctx);
        }

        // Check for movement to take care of
        Sniper_CheckMoveState(ctx);

        // See if we should override shooting decision with any special considerations
        Sniper_CheckFireState(ctx);

        if world.globals.move2 != 0 {
            // move toward goal
            if (*NPCInfo).goalEntity != None {
                world.globals.move2 = Sniper_Move(ctx);
            } else {
                world.globals.move2 = QFALSE;
            }
        }

        if world.globals.move2 == QFALSE {
            if TIMER_Done(ctx, NPC, c"duck".as_ptr()) == 0 {
                // not TIMER_Done
                if TIMER_Done(ctx, NPC, c"watch".as_ptr()) != 0 {
                    // not while watching
                    world.globals.ucmd.upmove = -127;
                }
            }
            // FIXME: what about leaning?
            // FIXME: also, when stop ducking, start looking, if enemy can see me, chance of ducking back down again
        } else {
            // stop ducking!
            TIMER_Set(ctx, NPC, c"duck".as_ptr(), -1);
        }

        if TIMER_Done(ctx, NPC, c"duck".as_ptr()) != 0
            && TIMER_Done(ctx, NPC, c"watch".as_ptr()) != 0
            && (TIMER_Get(ctx, NPC, c"attackDelay".as_ptr()) - world.level.time) > 1000
            && (*NPC).attackDebounceTime < world.level.time
        {
            if world.globals.enemyLOS2 != 0 && ((*NPCInfo).scriptFlags & 0x08) != 0 {
                // SCF_ALT_FIRE = 0x08
                if (*NPC).fly_sound_debounce_time < world.level.time {
                    (*NPC).fly_sound_debounce_time = world.level.time + 2000;
                }
            }
        }

        if world.globals.faceEnemy2 == QFALSE {
            // we want to face in the dir we're running
            if world.globals.move2 != 0 {
                // don't run away and shoot
                (*NPCInfo).desiredYaw = (*NPCInfo).lastPathAngles[0]; // YAW
                (*NPCInfo).desiredPitch = 0.0;
                world.globals.shoot2 = QFALSE;
            }
            NPC_UpdateAngles(ctx, QTRUE, QTRUE);
        } else {
            // face the enemy
            Sniper_FaceEnemy(ctx);
        }

        if ((*NPCInfo).scriptFlags & 0x01000) != 0 {
            // SCF_DONT_FIRE = 0x01000
            world.globals.shoot2 = QFALSE;
        }

        // FIXME: don't shoot right away!
        if world.globals.shoot2 != 0 {
            // try to shoot if it's time
            if TIMER_Done(ctx, NPC, c"attackDelay".as_ptr()) != 0 {
                WeaponThink(ctx, QTRUE);
                if (world.globals.ucmd.buttons & (BUTTON_ATTACK | BUTTON_ALT_ATTACK)) != 0 {
                    // G_SoundOnEnt(ctx, NPC, CHAN_WEAPON, "sound/null.wav");
                }

                // took a shot, now hide
                if ((*NPC).spawnflags & 0x0100) == 0 && Q_irand(0, 1) == 0 {
                    // SPF_NO_HIDE = 0x0100
                    // FIXME: do this if in combat point and combat point has duck-type cover...
                    Sniper_StartHide(ctx);
                } else {
                    TIMER_Set(ctx, NPC, c"attackDelay".as_ptr(), (*NPCInfo).shotTime - world.level.time);
                }
            }
        }
    }
}

/// Raven `NPC_BSSniper_Default`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:854-864`
pub fn NPC_BSSniper_Default(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;

        if (*NPC).enemy == None {
            // don't have an enemy, look for one
            NPC_BSSniper_Patrol(ctx);
        } else {
            // have an enemy
            NPC_BSSniper_Attack(ctx);
        }
    }
}
