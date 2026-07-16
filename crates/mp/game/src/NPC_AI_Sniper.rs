// PORT-COMPLETE: NPC_AI_Sniper.c 15/15
//! FAITHFUL port of `oracle/codemp/game/NPC_AI_Sniper.c`.
//!
//! All functions reach file-scope game state (`level`, `g_entities`, the
//! file-statics `enemyLOS2`/`enemyCS2`/`faceEnemy2`/`move2`/`shoot2`/
//! `enemyDist2` — genuine cross-frame state, `GameWorld` fields) and engine
//! traps through the threaded `GameContext`/`GameWorld` handle.
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_nav::NPC_SetMoveGoal;
use crate::g_nav::{FlyingCreature, NAV_HitNavGoal};
use crate::g_timer::{TIMER_Done, TIMER_Get, TIMER_Set};
use crate::g_utils::{G_SoundOnEnt, GetAnglesForDirection};
use crate::g_weapon::CalcMuzzlePoint;
use crate::npc::g_npc_t::{ENEMY_POS_LAG_INTERVAL, MAX_ENEMY_POS_LAG};
use crate::prelude::*;
use crate::q_math::{
    _VectorMA, _VectorSubtract, vec3_origin, vectoangles, AngleNormalize360, AngleVectors,
    VectorNormalize, PITCH, YAW,
};
use crate::NPC_AI_Stormtrooper::NPC_CheckPlayerTeamStealth;
use crate::NPC_combat::{
    G_ClearEnemy, G_SetEnemy, NPC_ChangeWeapon, NPC_FindCombatPoint, NPC_FreeCombatPoint,
    NPC_MaxDistSquaredForWeapon, NPC_SetCombatPoint, WeaponThink,
};
use crate::NPC_goal::{NPC_ReachedGoal, UpdateGoal};
use crate::NPC_move::NAV_GetLastMove;
use crate::NPC_move::NPC_MoveToGoal;
use crate::NPC_reactions::NPC_Pain;
use crate::NPC_senses::{NPC_CheckAlertEvents, NPC_CheckForDanger};
use crate::NPC_sounds::G_AddVoiceEvent;
use crate::NPC_utils::{CalcEntitySpot, NPC_CheckEnemyExt, NPC_ClearLOS4, NPC_UpdateAngles};
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use mp_bg::public::entity_event::entity_event_t::{
    EV_CONFUSE1, EV_CONFUSE3, EV_PUSHED1, EV_PUSHED3,
};
use mp_qshared::common::mp::qcommon::usercmd_button::{
    BUTTON_ALT_ATTACK, BUTTON_ATTACK, BUTTON_WALKING,
};

// EntityId seam helper: resolve `Option<EntityId>` back to the raw pointer the
// verbatim body still expects (`None` -> null), per the `NPC_AI_Stormtrooper.rs`
// precedent.
#[inline]
unsafe fn ent_resolve_opt(ctx: &mut GameContext, id: Option<EntityId>) -> *mut gentity_t {
    match id {
        Some(i) => &mut ctx.world.g_entities[i.index()] as *mut gentity_t,
        None => core::ptr::null_mut(),
    }
}

// Raven's anonymous `enum { LSTATE_NONE, LSTATE_UNDERFIRE, LSTATE_INVESTIGATE }`
// (file-scope local state, `gNPC_t::localState`) — not a central type, ported
// as file-local consts matching the C values.
// Source: `oracle/codemp/game/NPC_AI_Sniper.c:37-42`
const LSTATE_NONE: i32 = 0;
const LSTATE_UNDERFIRE: i32 = 1;
const LSTATE_INVESTIGATE: i32 = 2;

// Squad behavior states (from oracle/codemp/game/NPC_behavior.c)
// Source: `oracle/codemp/game/NPC_behavior.c`
const SQUAD_IDLE: i32 = 0;
const SQUAD_STAND_AND_SHOOT: i32 = 1;
const SQUAD_RETREAT: i32 = 2;
const SQUAD_COVER: i32 = 3;
const SQUAD_TRANSITION: i32 = 4;
const SQUAD_POINT: i32 = 5;
const SQUAD_SCOUT: i32 = 6;

// Combat point search flags (`combatPoint_t` request bits):
// `crate::npc::combat_point_flags` (`b_local.h:244-259`).

// Enemy position lagging for sniper targeting. MAX_ENEMY_POS_LAG /
// ENEMY_POS_LAG_INTERVAL imported from `crate::npc::g_npc_t`. STEPS is kept
// local as `i32` (g_npc_t's is `usize`, for array sizing) since it is used in
// signed arithmetic here.
// Source: `oracle/codemp/game/b_public.h:113-115`
const ENEMY_POS_LAG_STEPS: i32 = MAX_ENEMY_POS_LAG / ENEMY_POS_LAG_INTERVAL; // 24

// `MASK_SHOT` (`bg_public.h:1177`) now resolves via the crate prelude
// (pass-3 symbol backfill, `mp_qshared::shared::surface_flags`).

/// Raven `Sniper_ClearTimers`.
///
/// Source: `oracle/codemp/game/NPC_AI_Sniper.c:44-58`
pub fn Sniper_ClearTimers(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);
    TIMER_Set(ctx, ctx.entity_id_of(ent), c"chatter".as_ptr(), 0);
    TIMER_Set(ctx, ctx.entity_id_of(ent), c"duck".as_ptr(), 0);
    TIMER_Set(ctx, ctx.entity_id_of(ent), c"stand".as_ptr(), 0);
    TIMER_Set(ctx, ctx.entity_id_of(ent), c"shuffleTime".as_ptr(), 0);
    TIMER_Set(ctx, ctx.entity_id_of(ent), c"sleepTime".as_ptr(), 0);
    TIMER_Set(ctx, ctx.entity_id_of(ent), c"enemyLastVisible".as_ptr(), 0);
    TIMER_Set(ctx, ctx.entity_id_of(ent), c"roamTime".as_ptr(), 0);
    TIMER_Set(ctx, ctx.entity_id_of(ent), c"hideTime".as_ptr(), 0);
    // FIXME: Slant for difficulty levels (Raven comment).
    TIMER_Set(ctx, ctx.entity_id_of(ent), c"attackDelay".as_ptr(), 0);
    TIMER_Set(ctx, ctx.entity_id_of(ent), c"stick".as_ptr(), 0);
    TIMER_Set(ctx, ctx.entity_id_of(ent), c"scoutTime".as_ptr(), 0);
    TIMER_Set(ctx, ctx.entity_id_of(ent), c"flee".as_ptr(), 0);
}

/// Raven `NPC_Sniper_PlayConfusionSound`.
///
/// Source: `oracle/codemp/game/NPC_AI_Sniper.c:60-76`
pub fn NPC_Sniper_PlayConfusionSound(ctx: &mut GameContext, self_: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    unsafe {
        if (*self_).health > 0 {
            let self_id = ctx.entity_id_of(self_).unwrap();
            let voice_event = ctx
                .world
                .bg_state
                .rng
                .Q_irand(EV_CONFUSE1 as c_int, EV_CONFUSE3 as c_int);
            G_AddVoiceEvent(ctx, self_id, voice_event, 2000);
        }
        // reset him to be totally unaware again
        TIMER_Set(
            ctx,
            ctx.entity_id_of(self_),
            c"enemyLastVisible".as_ptr(),
            0,
        );
        TIMER_Set(ctx, ctx.entity_id_of(self_), c"flee".as_ptr(), 0);

        let npc = (*self_).NPC as *mut gNPC_t;
        (*npc).squadState = SQUAD_IDLE;
        (*npc).tempBehavior = bState_t::BS_DEFAULT;

        // Clear the enemy
        G_ClearEnemy(ctx, ctx.entity_id_of(self_).unwrap()); // FIXME: or just self->enemy = NULL;? (Raven comment).

        (*npc).investigateCount = 0;
    }
}

/// Raven `NPC_Sniper_Pain`.
///
/// Source: `oracle/codemp/game/NPC_AI_Sniper.c:85-98`
pub fn NPC_Sniper_Pain(
    ctx: &mut GameContext,
    self_: EntityId,
    attacker: Option<EntityId>,
    damage: c_int,
) {
    // STAGE-1: EntityId params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    let attacker: *mut gentity_t = unsafe { ent_resolve_opt(ctx, attacker) };
    unsafe {
        let npc = (*self_).NPC as *mut gNPC_t;
        (*npc).localState = LSTATE_UNDERFIRE;

        TIMER_Set(ctx, ctx.entity_id_of(self_), c"duck".as_ptr(), -1);
        TIMER_Set(ctx, ctx.entity_id_of(self_), c"stand".as_ptr(), 2000);

        NPC_Pain(
            ctx,
            ctx.entity_id_of(self_).unwrap(),
            ctx.entity_id_of(attacker),
            damage,
        );

        if damage == 0 && (*self_).health > 0 {
            let self_id = ctx.entity_id_of(self_).unwrap();
            let voice_event = ctx
                .world
                .bg_state
                .rng
                .Q_irand(EV_PUSHED1 as c_int, EV_PUSHED3 as c_int);
            // FIXME: better way to know I was pushed (Raven comment).
            G_AddVoiceEvent(ctx, self_id, voice_event, 2000);
        }
    }
}

/// Raven `Sniper_HoldPosition`.
///
/// Source: `oracle/codemp/game/NPC_AI_Sniper.c:106-116`
pub fn Sniper_HoldPosition(ctx: &mut GameContext) {
    unsafe {
        let NPCInfo = ctx.world.globals.NPCInfo as *mut gNPC_t;

        NPC_FreeCombatPoint(ctx, (*NPCInfo).combatPoint, qtrue);
        (*NPCInfo).goalEntity = None;
    }
}

/// Raven `Sniper_Move`.
///
/// Source: `oracle/codemp/game/NPC_AI_Sniper.c:124-177`
pub fn Sniper_Move(ctx: &mut GameContext) -> qboolean {
    unsafe {
        let NPC = ctx.world.globals.NPC as *mut gentity_t;
        let NPCInfo = ctx.world.globals.NPCInfo as *mut gNPC_t;

        (*NPCInfo).combatMove = qtrue;

        let moved = NPC_MoveToGoal(ctx, qtrue);

        // Get the move info
        let mut info: navInfo_t = core::mem::zeroed();
        NAV_GetLastMove(ctx, &mut info);

        // If we hit our target, then stop and fire!
        if (info.flags & NIF_COLLISION) != 0 {
            // NIF_COLLISION = 0x4 (0x1 is NIF_FAILED)
            if ent_id_opt(ctx.world.g_entities.as_ptr(), info.blocker) == (*NPC).enemy {
                Sniper_HoldPosition(ctx);
            }
        }

        // If our move failed, then reset
        if moved == qfalse {
            // couldn't get to enemy
            if ((*NPCInfo).scriptFlags & 0x00000400) != 0
                && (*NPCInfo).goalEntity != None
                && (*NPCInfo).goalEntity == (*NPC).enemy
            {
                // SCF_CHASE_ENEMIES = 0x00000400, we were running after enemy
                // Try to find a combat point that can hit the enemy
                let mut cpFlags = CP_CLEAR | CP_HAS_ROUTE;
                let cp;
                if ((*NPCInfo).scriptFlags & 0x00100000) != 0 {
                    // SCF_USE_CP_NEAREST = 0x00100000
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
                if cp == -1 && ((*NPCInfo).scriptFlags & 0x00100000) == 0 {
                    // okay, try one by the enemy
                    let cp2 = NPC_FindCombatPoint(
                        ctx,
                        (*NPC).r.currentOrigin,
                        (*NPC).r.currentOrigin,
                        ctx.world.g_entities[(*NPC).enemy.unwrap().index()]
                            .r
                            .currentOrigin,
                        CP_CLEAR | CP_HAS_ROUTE | CP_HORZ_DIST_COLL,
                        32.0,
                        -1,
                    );
                    if cp2 != -1 {
                        // found a combat point that has a clear shot to enemy
                        NPC_SetCombatPoint(ctx, cp2);
                        let npc_id = ctx.entity_id_of(NPC).unwrap();
                        NPC_SetMoveGoal(
                            ctx,
                            npc_id,
                            ctx.world.level.combatPoints[cp2 as usize].origin,
                            8,
                            qtrue,
                            cp2,
                            None,
                        );
                        return moved;
                    }
                } else if cp != -1 {
                    // found a combat point that has a clear shot to enemy
                    NPC_SetCombatPoint(ctx, cp);
                    let npc_id = ctx.entity_id_of(NPC).unwrap();
                    NPC_SetMoveGoal(
                        ctx,
                        npc_id,
                        ctx.world.level.combatPoints[cp as usize].origin,
                        8,
                        qtrue,
                        cp,
                        None,
                    );
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
/// Source: `oracle/codemp/game/NPC_AI_Sniper.c:185-275`
pub fn NPC_BSSniper_Patrol(ctx: &mut GameContext) {
    unsafe {
        let NPC = ctx.world.globals.NPC as *mut gentity_t;
        let NPCInfo = ctx.world.globals.NPCInfo as *mut gNPC_t;

        (*NPC).count = 0;

        if (*NPCInfo).confusionTime < ctx.world.level.time {
            // Look for any enemies
            if ((*NPCInfo).scriptFlags & 0x00000800) != 0 {
                // SCF_LOOK_FOR_ENEMIES = 0x00000800
                if NPC_CheckPlayerTeamStealth(ctx) != 0 {
                    // Look for player team members with stealth
                    NPC_UpdateAngles(ctx, qtrue, qtrue);
                    return;
                }
            }

            if ((*NPCInfo).scriptFlags & 0x00002000) == 0 {
                // SCF_IGNORE_ALERTS = 0x00002000
                // Is there danger nearby
                let alertEvent = NPC_CheckAlertEvents(ctx, qtrue, qtrue, -1, qfalse, 1); // AEL_SUSPICIOUS = 1
                if NPC_CheckForDanger(ctx, alertEvent) != 0 {
                    NPC_UpdateAngles(ctx, qtrue, qtrue);
                    return;
                } else if alertEvent >= 0
                    && ctx.world.level.alertEvents[alertEvent as usize].ID != (*NPCInfo).lastAlertID
                {
                    // check for other alert events
                    // There is an event to look at
                    (*NPCInfo).lastAlertID = ctx.world.level.alertEvents[alertEvent as usize].ID;
                    if ctx.world.level.alertEvents[alertEvent as usize].level as i32 == 2 {
                        // AEL_DISCOVERED = 2
                        let owner = ctx.world.level.alertEvents[alertEvent as usize].owner;
                        if owner != core::ptr::null_mut()
                            && (*owner).client != core::ptr::null_mut()
                            && (*owner).health >= 0
                            && (*((*owner).client as *mut gclient_t)).playerTeam
                                == (*((*NPC).client as *mut gclient_t)).enemyTeam
                        {
                            // an enemy
                            let npc_id = ctx.entity_id_of(NPC).unwrap();
                            let owner_id = ctx.entity_id_of(owner);
                            G_SetEnemy(ctx, npc_id, owner_id);
                            //NPCInfo->enemyLastSeenTime = level.time;
                            let attack_npc_id = ctx.entity_id_of(NPC);
                            let delay = ctx.world.bg_state.rng.Q_irand(
                                (6 - (*NPCInfo).stats.aim) * 100,
                                (6 - (*NPCInfo).stats.aim) * 500,
                            );
                            TIMER_Set(ctx, attack_npc_id, c"attackDelay".as_ptr(), delay);
                        }
                    } else {
                        // FIXME: get more suspicious over time?
                        // Save the position for movement (if necessary)
                        (*NPCInfo).investigateGoal[0] =
                            ctx.world.level.alertEvents[alertEvent as usize].position[0];
                        (*NPCInfo).investigateGoal[1] =
                            ctx.world.level.alertEvents[alertEvent as usize].position[1];
                        (*NPCInfo).investigateGoal[2] =
                            ctx.world.level.alertEvents[alertEvent as usize].position[2];
                        (*NPCInfo).investigateDebounceTime =
                            ctx.world.level.time + ctx.world.bg_state.rng.Q_irand(500, 1000);
                        if ctx.world.level.alertEvents[alertEvent as usize].level as i32 == 1 {
                            // AEL_SUSPICIOUS = 1: suspicious looks longer
                            (*NPCInfo).investigateDebounceTime +=
                                ctx.world.bg_state.rng.Q_irand(500, 2500);
                        }
                    }
                }
            }

            if (*NPCInfo).investigateDebounceTime > ctx.world.level.time {
                // FIXME: walk over to it, maybe?  Not if not chase enemies flag
                // NOTE: stops walking or doing anything else below
                let mut dir = [0.0f32; 3];
                let mut angles = [0.0f32; 3];

                _VectorSubtract(
                    (*NPCInfo).investigateGoal,
                    (*((*NPC).client as *mut gclient_t)).renderInfo.eyePoint,
                    &mut dir,
                );
                vectoangles(dir, &mut angles);

                let o_yaw = (*NPCInfo).desiredYaw;
                let o_pitch = (*NPCInfo).desiredPitch;
                (*NPCInfo).desiredYaw = angles[YAW];
                (*NPCInfo).desiredPitch = angles[PITCH];

                NPC_UpdateAngles(ctx, qtrue, qtrue);

                (*NPCInfo).desiredYaw = o_yaw;
                (*NPCInfo).desiredPitch = o_pitch;
                return;
            }
        }

        // If we have somewhere to go, then do that
        if UpdateGoal(ctx) != core::ptr::null_mut() {
            ctx.world.globals.ucmd.buttons |= BUTTON_WALKING;
            NPC_MoveToGoal(ctx, qtrue);
        }

        NPC_UpdateAngles(ctx, qtrue, qtrue);
    }
}

/// Raven `Sniper_CheckMoveState`.
///
/// Source: `oracle/codemp/game/NPC_AI_Sniper.c:308-381`
pub fn Sniper_CheckMoveState(ctx: &mut GameContext) {
    unsafe {
        let NPC = ctx.world.globals.NPC as *mut gentity_t;
        let NPCInfo = ctx.world.globals.NPCInfo as *mut gNPC_t;

        // See if we're a scout
        if ((*NPCInfo).scriptFlags & 0x00000400) == 0 {
            // SCF_CHASE_ENEMIES = 0x00000400
            if (*NPCInfo).goalEntity == (*NPC).enemy {
                ctx.world.globals.move2 = qfalse;
                return;
            }
        } else if (*NPCInfo).squadState == SQUAD_RETREAT {
            // See if we're running away
            if TIMER_Done(ctx, ctx.entity_id_of(NPC), c"flee".as_ptr()) != 0 {
                (*NPCInfo).squadState = SQUAD_IDLE;
            } else {
                ctx.world.globals.faceEnemy2 = qfalse;
            }
        } else if (*NPCInfo).squadState == SQUAD_IDLE {
            if (*NPCInfo).goalEntity == None {
                ctx.world.globals.move2 = qfalse;
                return;
            }
        }

        // See if we're moving towards a goal, not the enemy
        if ((*NPCInfo).goalEntity != (*NPC).enemy) && ((*NPCInfo).goalEntity != None) {
            // Did we make it?
            let flying = FlyingCreature(&*NPC);
            let goal_ent =
                &mut ctx.world.g_entities[(*NPCInfo).goalEntity.unwrap().index()] as *mut gentity_t;
            let npc_id = ctx.entity_id_of(NPC);
            if NAV_HitNavGoal(
                (*NPC).r.currentOrigin,
                (*NPC).r.mins,
                (*NPC).r.maxs,
                (*goal_ent).r.currentOrigin,
                16,
                flying,
            ) != 0
                || ((*NPCInfo).squadState == SQUAD_SCOUT
                    && ctx.world.globals.enemyLOS2 != 0
                    && ctx.world.globals.enemyDist2 <= 10000.0)
            {
                // we got where we wanted to go, set timers based on why we were running
                let mut newSquadState = SQUAD_STAND_AND_SHOOT;
                match (*NPCInfo).squadState {
                    2 => {
                        let duck_npc_id = ctx.entity_id_of(NPC);
                        // SQUAD_RETREAT=2: was running away
                        TIMER_Set(
                            ctx,
                            duck_npc_id,
                            c"duck".as_ptr(),
                            ((*((*NPC).client as *mut gclient_t)).pers.maxHealth - (*NPC).health)
                                * 100,
                        );
                        let hide_npc_id = ctx.entity_id_of(NPC);
                        let hide_delay = ctx.world.bg_state.rng.Q_irand(3000, 7000);
                        TIMER_Set(ctx, hide_npc_id, c"hideTime".as_ptr(), hide_delay);
                        newSquadState = SQUAD_COVER;
                    }
                    4 => {
                        let hide_npc_id = ctx.entity_id_of(NPC);
                        let hide_delay = ctx.world.bg_state.rng.Q_irand(2000, 4000);
                        // SQUAD_TRANSITION=4: was heading for a combat point
                        TIMER_Set(ctx, hide_npc_id, c"hideTime".as_ptr(), hide_delay);
                    }
                    6 => {
                        // SQUAD_SCOUT=6: was running after player
                    }
                    _ => {}
                }
                NPC_ReachedGoal(ctx);
                let attack_npc_id = ctx.entity_id_of(NPC);
                let attack_delay = ctx.world.bg_state.rng.Q_irand(
                    (6 - (*NPCInfo).stats.aim) * 50,
                    (6 - (*NPCInfo).stats.aim) * 100,
                );
                // don't attack right away
                TIMER_Set(ctx, attack_npc_id, c"attackDelay".as_ptr(), attack_delay);
                let roam_npc_id = ctx.entity_id_of(NPC);
                let roam_delay = ctx.world.bg_state.rng.Q_irand(1000, 4000);
                // don't do something else just yet
                TIMER_Set(ctx, roam_npc_id, c"roamTime".as_ptr(), roam_delay);
                // stop fleeing
                if (*NPCInfo).squadState == SQUAD_RETREAT {
                    let flee_npc_id = ctx.entity_id_of(NPC);
                    TIMER_Set(ctx, flee_npc_id, c"flee".as_ptr(), -ctx.world.level.time);
                    (*NPCInfo).squadState = SQUAD_IDLE;
                }
                return;
            }

            // keep going, hold of roamTimer until we get there
            let delay = ctx.world.bg_state.rng.Q_irand(4000, 8000);
            TIMER_Set(ctx, npc_id, c"roamTime".as_ptr(), delay);
        }
    }
}

/// Raven `Sniper_ResolveBlockedShot`.
///
/// Source: `oracle/codemp/game/NPC_AI_Sniper.c:383-434`
pub fn Sniper_ResolveBlockedShot(ctx: &mut GameContext) {
    unsafe {
        let NPC = ctx.world.globals.NPC as *mut gentity_t;
        let NPCInfo = ctx.world.globals.NPCInfo as *mut gNPC_t;

        if TIMER_Done(ctx, ctx.entity_id_of(NPC), c"duck".as_ptr()) != 0 {
            // we're not ducking
            if TIMER_Done(ctx, ctx.entity_id_of(NPC), c"roamTime".as_ptr()) != 0 {
                // not roaming
                // FIXME: try to find another spot from which to hit the enemy
                if ((*NPCInfo).scriptFlags & 0x00000400) != 0
                    && ((*NPCInfo).goalEntity == None || (*NPCInfo).goalEntity == (*NPC).enemy)
                {
                    // SCF_CHASE_ENEMIES = 0x00000400
                    // we were running after enemy
                    // Try to find a combat point that can hit the enemy
                    let mut cpFlags = CP_CLEAR | CP_HAS_ROUTE;
                    let cp;

                    if ((*NPCInfo).scriptFlags & 0x00100000) != 0 {
                        // SCF_USE_CP_NEAREST = 0x00100000
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
                    if cp == -1 && ((*NPCInfo).scriptFlags & 0x00100000) == 0 {
                        // okay, try one by the enemy
                        let cp2 = NPC_FindCombatPoint(
                            ctx,
                            (*NPC).r.currentOrigin,
                            (*NPC).r.currentOrigin,
                            ctx.world.g_entities[(*NPC).enemy.unwrap().index()]
                                .r
                                .currentOrigin,
                            CP_CLEAR | CP_HAS_ROUTE | CP_HORZ_DIST_COLL,
                            32.0,
                            -1,
                        );
                        if cp2 != -1 {
                            // found a combat point that has a clear shot to enemy
                            NPC_SetCombatPoint(ctx, cp2);
                            let npc_id = ctx.entity_id_of(NPC).unwrap();
                            NPC_SetMoveGoal(
                                ctx,
                                npc_id,
                                ctx.world.level.combatPoints[cp2 as usize].origin,
                                8,
                                qtrue,
                                cp2,
                                None,
                            );
                            TIMER_Set(ctx, ctx.entity_id_of(NPC), c"duck".as_ptr(), -1);
                            let attack_npc_id = ctx.entity_id_of(NPC);
                            let delay = ctx.world.bg_state.rng.Q_irand(1000, 3000);
                            TIMER_Set(ctx, attack_npc_id, c"attackDelay".as_ptr(), delay);
                            return;
                        }
                    } else if cp != -1 {
                        // found a combat point that has a clear shot to enemy
                        NPC_SetCombatPoint(ctx, cp);
                        let npc_id = ctx.entity_id_of(NPC).unwrap();
                        NPC_SetMoveGoal(
                            ctx,
                            npc_id,
                            ctx.world.level.combatPoints[cp as usize].origin,
                            8,
                            qtrue,
                            cp,
                            None,
                        );
                        TIMER_Set(ctx, ctx.entity_id_of(NPC), c"duck".as_ptr(), -1);
                        let attack_npc_id = ctx.entity_id_of(NPC);
                        let delay = ctx.world.bg_state.rng.Q_irand(1000, 3000);
                        TIMER_Set(ctx, attack_npc_id, c"attackDelay".as_ptr(), delay);
                        return;
                    }
                }
            }
        }
    }
}

/// Raven `Sniper_CheckFireState`.
///
/// Source: `oracle/codemp/game/NPC_AI_Sniper.c:442-486`
pub fn Sniper_CheckFireState(ctx: &mut GameContext) {
    unsafe {
        let NPC = ctx.world.globals.NPC as *mut gentity_t;
        let NPCInfo = ctx.world.globals.NPCInfo as *mut gNPC_t;

        if ctx.world.globals.enemyCS2 != 0 {
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
        if (*((*NPC).client as *mut gclient_t)).ps.velocity[0] != 0.0
            || (*((*NPC).client as *mut gclient_t)).ps.velocity[1] != 0.0
            || (*((*NPC).client as *mut gclient_t)).ps.velocity[2] != 0.0
        {
            // if moving at all, don't do this
            return;
        }

        // continue to fire on their last position
        if ctx.world.bg_state.rng.Q_irand(0, 1) == 0
            && (*NPCInfo).enemyLastSeenTime != 0
            && ctx.world.level.time - (*NPCInfo).enemyLastSeenTime
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

                CalcEntitySpot(ctx, ctx.entity_id_of(NPC), spot_t::SPOT_WEAPON, &mut muzzle);
                _VectorSubtract((*NPCInfo).enemyLastSeenLocation, muzzle, &mut dir);

                VectorNormalize(&mut dir);

                vectoangles(dir, &mut angles);

                (*NPCInfo).desiredYaw = angles[YAW];
                (*NPCInfo).desiredPitch = angles[PITCH];

                ctx.world.globals.shoot2 = qtrue;
                //faceEnemy2 = qfalse;
            }
            return;
        } else if ctx.world.level.time - (*NPCInfo).enemyLastSeenTime > 10000 {
            // next time we see him, we'll miss few times first
            (*NPC).count = 0;
        }
    }
}

/// Raven `Sniper_EvaluateShot`.
///
/// Source: `oracle/codemp/game/NPC_AI_Sniper.c:488-506`
pub fn Sniper_EvaluateShot(ctx: &mut GameContext, hit: c_int) -> qboolean {
    unsafe {
        let NPC = ctx.world.globals.NPC as *mut gentity_t;

        if (*NPC).enemy == None {
            return qfalse;
        }

        let enemy_number = ctx.world.g_entities[(*NPC).enemy.unwrap().index()].s.number;
        let hitEnt = &mut ctx.world.g_entities[hit as usize];
        if hit == enemy_number
            || (hitEnt.client != core::ptr::null_mut()
                && (*(hitEnt.client as *mut gclient_t)).playerTeam == (*((*NPC).client as *mut gclient_t)).enemyTeam)
            || (hitEnt.takedamage != 0
                && ((hitEnt.r.svFlags & 0x08000000) != 0 // SVF_GLASS_BRUSH = 0x08000000
                    || hitEnt.health < 40
                    || (*NPC).s.weapon == 17)) // WP_EMPLACED_GUN = 17
            || (hitEnt.r.svFlags & 0x08000000) != 0
        {
            // can hit enemy or will hit glass, so shoot anyway
            return qtrue;
        }
        qfalse
    }
}

/// Raven `Sniper_FaceEnemy`.
///
/// Source: `oracle/codemp/game/NPC_AI_Sniper.c:508-603`
pub fn Sniper_FaceEnemy(ctx: &mut GameContext) {
    unsafe {
        let NPC = ctx.world.globals.NPC as *mut gentity_t;
        let NPCInfo = ctx.world.globals.NPCInfo as *mut gNPC_t;

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
        AngleVectors(
            (*((*NPC).client as *mut gclient_t)).ps.viewangles,
            Some(&mut forward),
            Some(&mut right),
            Some(&mut up),
        );
        let npc_id = ctx.entity_id_of(NPC).unwrap();
        CalcMuzzlePoint(ctx, npc_id, forward, right, up, &mut muzzle);
        //CalcEntitySpot( NPC, SPOT_WEAPON, muzzle );
        CalcEntitySpot(ctx, (*NPC).enemy, spot_t::SPOT_ORIGIN, &mut target);

        if ctx.world.globals.enemyDist2 > 65536.0 && (*NPCInfo).stats.aim < 5 {
            // is 256 squared, was 16384 (128*128)
            if (*NPC).count < (5 - (*NPCInfo).stats.aim) {
                // miss a few times first
                if ctx.world.globals.shoot2 != 0
                    && TIMER_Done(ctx, ctx.entity_id_of(NPC), c"attackDelay".as_ptr()) != 0
                    && ctx.world.level.time >= (*NPCInfo).shotTime
                {
                    // ready to fire again
                    let mut aimError = qfalse;
                    let mut hit = qtrue;
                    let mut tryMissCount = 0;
                    let mut trace: trace_t = core::mem::zeroed();

                    GetAnglesForDirection(muzzle, target, &mut angles);
                    AngleVectors(angles, Some(&mut forward), Some(&mut right), Some(&mut up));

                    while hit != 0 && tryMissCount < 10 {
                        tryMissCount += 1;
                        let enemy_maxs2 =
                            ctx.world.g_entities[(*NPC).enemy.unwrap().index()].r.maxs[2];
                        let enemy_mins2 =
                            ctx.world.g_entities[(*NPC).enemy.unwrap().index()].r.mins[2];
                        // VectorMA is the live `#if 1` macro (q_shared.h:1365): the
                        // flrand-bearing scale expression substitutes per component —
                        // THREE draws per call, distinct offsets, all f32.
                        // Source: `oracle/codemp/game/NPC_AI_Sniper.c:544-559`
                        if ctx.world.bg_state.rng.Q_irand(0, 1) == 0 {
                            aimError = qtrue;
                            if ctx.world.bg_state.rng.Q_irand(0, 1) == 0 {
                                for i in 0..3 {
                                    target[i] += right[i]
                                        * (enemy_maxs2 * ctx.world.bg_state.rng.flrand(1.5, 4.0));
                                }
                            } else {
                                for i in 0..3 {
                                    target[i] += right[i]
                                        * (enemy_mins2 * ctx.world.bg_state.rng.flrand(1.5, 4.0));
                                }
                            }
                        }
                        if aimError == qfalse || ctx.world.bg_state.rng.Q_irand(0, 1) == 0 {
                            if ctx.world.bg_state.rng.Q_irand(0, 1) == 0 {
                                for i in 0..3 {
                                    target[i] += up[i]
                                        * (enemy_maxs2 * ctx.world.bg_state.rng.flrand(1.5, 4.0));
                                }
                            } else {
                                for i in 0..3 {
                                    target[i] += up[i]
                                        * (enemy_mins2 * ctx.world.bg_state.rng.flrand(1.5, 4.0));
                                }
                            }
                        }
                        trap::Trace(
                            ctx.engine,
                            GTraceArgs::new(
                                &mut trace as *mut trace_t,
                                &muzzle as *const vec3_t,
                                &vec3_origin as *const vec3_t,
                                &vec3_origin as *const vec3_t,
                                &target as *const vec3_t,
                                (*NPC).s.number,
                                MASK_SHOT,
                            ),
                        );
                        hit = Sniper_EvaluateShot(ctx, trace.entityNum as c_int);
                    }
                    (*NPC).count += 1;
                } else if ctx.world.globals.enemyLOS2 == 0 {
                    NPC_UpdateAngles(ctx, qtrue, qtrue);
                    return;
                }
            } else {
                // based on distance, aim value, difficulty and enemy movement, miss
                // FIXME: incorporate distance as a factor?
                let missFactor = 8 - ((*NPCInfo).stats.aim + ctx.world.cvars.g_spskill.integer) * 3;
                let missFactor = if missFactor > ENEMY_POS_LAG_STEPS {
                    ENEMY_POS_LAG_STEPS
                } else if missFactor < 0 {
                    0
                } else {
                    missFactor
                };
                // §19: C clamps missFactor to ENEMY_POS_LAG_STEPS (24) then reads
                // enemyLaggedPos[24], one past the 24-elem array (UB). We guard <24 and
                // skip; unreachable in practice since missFactor = 8-(aim+skill)*3 <= 8.
                if missFactor >= 0 && (missFactor as usize) < ENEMY_POS_LAG_STEPS as usize {
                    target[0] = (*NPCInfo).enemyLaggedPos[missFactor as usize][0];
                    target[1] = (*NPCInfo).enemyLaggedPos[missFactor as usize][1];
                    target[2] = (*NPCInfo).enemyLaggedPos[missFactor as usize][2];
                }
            }
            GetAnglesForDirection(muzzle, target, &mut angles);
        } else {
            let enemy_maxs2 = ctx.world.g_entities[(*NPC).enemy.unwrap().index()].r.maxs[2];
            target[2] += ctx.world.bg_state.rng.flrand(0.0, enemy_maxs2);
            //CalcEntitySpot( NPC->enemy, SPOT_HEAD_LEAN, target );
            GetAnglesForDirection(muzzle, target, &mut angles);
        }

        (*NPCInfo).desiredYaw = AngleNormalize360(angles[YAW]);
        (*NPCInfo).desiredPitch = AngleNormalize360(angles[PITCH]);
        NPC_UpdateAngles(ctx, qtrue, qtrue);
    }
}

/// Raven `Sniper_UpdateEnemyPos`.
///
/// Source: `oracle/codemp/game/NPC_AI_Sniper.c:605-623`
pub fn Sniper_UpdateEnemyPos(ctx: &mut GameContext) {
    unsafe {
        let NPC = ctx.world.globals.NPC as *mut gentity_t;
        let NPCInfo = ctx.world.globals.NPCInfo as *mut gNPC_t;

        let mut i = MAX_ENEMY_POS_LAG - ENEMY_POS_LAG_INTERVAL;
        while i >= 0 {
            let index = (i / ENEMY_POS_LAG_INTERVAL) as usize;
            if index == 0 {
                let mut spot = [0.0f32; 3];
                let enemy_id = ctx.entity_id_of(
                    (*NPC)
                        .enemy
                        .map(|id| &ctx.world.g_entities[id.index()] as *const gentity_t)
                        .unwrap_or(core::ptr::null()),
                );
                CalcEntitySpot(ctx, enemy_id, spot_t::SPOT_HEAD_LEAN, &mut spot);
                (*NPCInfo).enemyLaggedPos[index][0] = spot[0];
                (*NPCInfo).enemyLaggedPos[index][1] = spot[1];
                (*NPCInfo).enemyLaggedPos[index][2] =
                    spot[2] - ctx.world.bg_state.rng.flrand(2.0, 16.0);
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
/// Source: `oracle/codemp/game/NPC_AI_Sniper.c:631-638`
pub fn Sniper_StartHide(ctx: &mut GameContext) {
    let NPC = ctx.world.globals.NPC as *mut gentity_t;

    let duckTime = ctx.world.bg_state.rng.Q_irand(2000, 5000);
    TIMER_Set(ctx, ctx.entity_id_of(NPC), c"duck".as_ptr(), duckTime);
    TIMER_Set(ctx, ctx.entity_id_of(NPC), c"watch".as_ptr(), 500);
    let npc_id = ctx.entity_id_of(NPC);
    let delay = duckTime + ctx.world.bg_state.rng.Q_irand(500, 2000);
    TIMER_Set(ctx, npc_id, c"attackDelay".as_ptr(), delay);
}

/// Raven `NPC_BSSniper_Attack`.
///
/// Source: `oracle/codemp/game/NPC_AI_Sniper.c:640-852`
pub fn NPC_BSSniper_Attack(ctx: &mut GameContext) {
    unsafe {
        let NPC = ctx.world.globals.NPC as *mut gentity_t;
        let NPCInfo = ctx.world.globals.NPCInfo as *mut gNPC_t;

        let npc_id = ctx.entity_id_of(NPC);
        // Don't do anything if we're hurt
        if (*NPC).painDebounceTime > ctx.world.level.time {
            NPC_UpdateAngles(ctx, qtrue, qtrue);
            return;
        }

        // If we don't have an enemy, just idle
        if NPC_CheckEnemyExt(ctx, qfalse) == qfalse {
            (*NPC).enemy = None;
            NPC_BSSniper_Patrol(ctx);
            return;
        }

        // Oracle short-circuit: NPC_CheckAlertEvents (side-effectful) runs only
        // when the flee timer is done (NPC_AI_Sniper.c:658).
        if TIMER_Done(ctx, npc_id, c"flee".as_ptr()) != 0 && {
            let alert_event = NPC_CheckAlertEvents(ctx, qtrue, qtrue, -1, qfalse, 4);
            NPC_CheckForDanger(ctx, alert_event) != 0
        } {
            // AEL_DANGER = 4, going to run
            NPC_UpdateAngles(ctx, qtrue, qtrue);
            return;
        }

        if (*NPC).enemy == None {
            // WTF? somehow we lost our enemy?
            NPC_BSSniper_Patrol(ctx);
            return;
        }

        let enemy_ptr = &mut ctx.world.g_entities[(*NPC).enemy.unwrap().index()] as *mut gentity_t;

        ctx.world.globals.enemyLOS2 = qfalse;
        ctx.world.globals.enemyCS2 = qfalse;
        ctx.world.globals.move2 = qtrue;
        ctx.world.globals.faceEnemy2 = qfalse;
        ctx.world.globals.shoot2 = qfalse;
        ctx.world.globals.enemyDist2 =
            DistanceSquared((*NPC).r.currentOrigin, (*enemy_ptr).r.currentOrigin);

        if ctx.world.globals.enemyDist2 < 16384.0 {
            // 128 squared, too close, so switch to primary fire
            if (*((*NPC).client as *mut gclient_t)).ps.weapon == 6 {
                // WP_DISRUPTOR = 6
                // sniping... should be assumed
                if ((*NPCInfo).scriptFlags & 0x00000040) != 0 {
                    // SCF_ALT_FIRE = 0x00000040
                    // use primary fire
                    let mut trace: trace_t = core::mem::zeroed();
                    trap::Trace(
                        ctx.engine,
                        GTraceArgs::new(
                            &mut trace,
                            &(*enemy_ptr).r.currentOrigin,
                            &(*enemy_ptr).r.mins,
                            &(*enemy_ptr).r.maxs,
                            &(*NPC).r.currentOrigin,
                            (*enemy_ptr).s.number,
                            (*enemy_ptr).clipmask,
                        ),
                    );
                    if trace.allsolid == 0
                        && trace.startsolid == 0
                        && (trace.fraction == 1.0 || trace.entityNum as c_int == (*NPC).s.number)
                    {
                        // he can get right to me
                        (*NPCInfo).scriptFlags &= !0x00000040; // SCF_ALT_FIRE
                                                               // reset fire-timing variables
                        NPC_ChangeWeapon(6); // WP_DISRUPTOR
                        NPC_UpdateAngles(ctx, qtrue, qtrue);
                        return;
                    }
                    // FIXME: switch back if he gets far away again?
                }
            }
        } else if ctx.world.globals.enemyDist2 > 65536.0 {
            // 256 squared
            if (*((*NPC).client as *mut gclient_t)).ps.weapon == 6 {
                // WP_DISRUPTOR = 6
                // sniping... should be assumed
                if ((*NPCInfo).scriptFlags & 0x00000040) == 0 {
                    // SCF_ALT_FIRE = 0x00000040
                    // use alt fire
                    (*NPCInfo).scriptFlags |= 0x00000040; // SCF_ALT_FIRE
                                                          // reset fire-timing variables
                    NPC_ChangeWeapon(6); // WP_DISRUPTOR
                    NPC_UpdateAngles(ctx, qtrue, qtrue);
                    return;
                }
            }
        }

        Sniper_UpdateEnemyPos(ctx);
        let enemy_ent = (*NPC)
            .enemy
            .map(|id| &raw mut ctx.world.g_entities[id.index()])
            .unwrap_or(core::ptr::null_mut());
        let enemy_id = ctx.entity_id_of(enemy_ent);
        // can we see our target?
        if NPC_ClearLOS4(ctx, enemy_id) != 0 {
            let maxShootDist;

            (*NPCInfo).enemyLastSeenTime = ctx.world.level.time;
            (*NPCInfo).enemyLastSeenLocation[0] = (*enemy_ptr).r.currentOrigin[0];
            (*NPCInfo).enemyLastSeenLocation[1] = (*enemy_ptr).r.currentOrigin[1];
            (*NPCInfo).enemyLastSeenLocation[2] = (*enemy_ptr).r.currentOrigin[2];
            ctx.world.globals.enemyLOS2 = qtrue;
            maxShootDist = NPC_MaxDistSquaredForWeapon(ctx);
            if ctx.world.globals.enemyDist2 < maxShootDist {
                let mut fwd = [0.0f32; 3];
                let mut right = [0.0f32; 3];
                let mut up = [0.0f32; 3];
                let mut muzzle = [0.0f32; 3];
                let mut end = [0.0f32; 3];
                let mut tr: trace_t = core::mem::zeroed();
                let hit;

                AngleVectors(
                    (*((*NPC).client as *mut gclient_t)).ps.viewangles,
                    Some(&mut fwd),
                    Some(&mut right),
                    Some(&mut up),
                );
                let npc_shoot_id = ctx.entity_id_of(NPC).unwrap();
                CalcMuzzlePoint(ctx, npc_shoot_id, fwd, right, up, &mut muzzle);
                _VectorMA(muzzle, 8192.0, fwd, &mut end);
                trap::Trace(
                    ctx.engine,
                    GTraceArgs::new(
                        &mut tr,
                        &muzzle,
                        core::ptr::null(),
                        core::ptr::null(),
                        &end,
                        (*NPC).s.number,
                        MASK_SHOT,
                    ),
                );

                hit = tr.entityNum;
                // can we shoot our target?
                if Sniper_EvaluateShot(ctx, hit as c_int) != 0 {
                    ctx.world.globals.enemyCS2 = qtrue;
                }
            }
        }

        if ctx.world.globals.enemyLOS2 != 0 {
            // FIXME: no need to face enemy if we're moving to some other goal and he's too far away to shoot?
            ctx.world.globals.faceEnemy2 = qtrue;
        }
        if ctx.world.globals.enemyCS2 != 0 {
            ctx.world.globals.shoot2 = qtrue;
        } else if ctx.world.level.time - (*NPCInfo).enemyLastSeenTime > 3000 {
            // Hmm, have to get around this bastard...
            Sniper_ResolveBlockedShot(ctx);
        }

        // Check for movement to take care of
        Sniper_CheckMoveState(ctx);

        // See if we should override shooting decision with any special considerations
        Sniper_CheckFireState(ctx);

        if ctx.world.globals.move2 != 0 {
            // move toward goal
            if (*NPCInfo).goalEntity != None {
                ctx.world.globals.move2 = Sniper_Move(ctx);
            } else {
                ctx.world.globals.move2 = qfalse;
            }
        }

        if ctx.world.globals.move2 == qfalse {
            if TIMER_Done(ctx, ctx.entity_id_of(NPC), c"duck".as_ptr()) == 0 {
                // not TIMER_Done
                if TIMER_Done(ctx, ctx.entity_id_of(NPC), c"watch".as_ptr()) != 0 {
                    // not while watching
                    ctx.world.globals.ucmd.upmove = -127;
                }
            }
            // FIXME: what about leaning?
            // FIXME: also, when stop ducking, start looking, if enemy can see me, chance of ducking back down again
        } else {
            let npc_id = ctx.entity_id_of(NPC);
            // stop ducking!
            TIMER_Set(ctx, npc_id, c"duck".as_ptr(), -1);
        }

        if TIMER_Done(ctx, ctx.entity_id_of(NPC), c"duck".as_ptr()) != 0
            && TIMER_Done(ctx, ctx.entity_id_of(NPC), c"watch".as_ptr()) != 0
            && (TIMER_Get(ctx, ctx.entity_id_of(NPC), c"attackDelay".as_ptr())
                - ctx.world.level.time)
                > 1000
            && (*NPC).attackDebounceTime < ctx.world.level.time
        {
            if ctx.world.globals.enemyLOS2 != 0 && ((*NPCInfo).scriptFlags & 0x00000040) != 0 {
                // SCF_ALT_FIRE = 0x00000040
                if (*NPC).fly_sound_debounce_time < ctx.world.level.time {
                    (*NPC).fly_sound_debounce_time = ctx.world.level.time + 2000;
                }
            }
        }

        if ctx.world.globals.faceEnemy2 == qfalse {
            // we want to face in the dir we're running
            if ctx.world.globals.move2 != 0 {
                // don't run away and shoot
                (*NPCInfo).desiredYaw = (*NPCInfo).lastPathAngles[YAW];
                (*NPCInfo).desiredPitch = 0.0;
                ctx.world.globals.shoot2 = qfalse;
            }
            NPC_UpdateAngles(ctx, qtrue, qtrue);
        } else {
            // face the enemy
            Sniper_FaceEnemy(ctx);
        }

        if ((*NPCInfo).scriptFlags & 0x00004000) != 0 {
            // SCF_DONT_FIRE = 0x00004000
            ctx.world.globals.shoot2 = qfalse;
        }

        // FIXME: don't shoot right away!
        if ctx.world.globals.shoot2 != 0 {
            // try to shoot if it's time
            if TIMER_Done(ctx, ctx.entity_id_of(NPC), c"attackDelay".as_ptr()) != 0 {
                WeaponThink(ctx, qtrue);
                if (ctx.world.globals.ucmd.buttons & (BUTTON_ATTACK | BUTTON_ALT_ATTACK)) != 0 {
                    let sound_npc_id = ctx.entity_id_of(NPC).unwrap();
                    G_SoundOnEnt(ctx, sound_npc_id, CHAN_WEAPON, c"sound/null.wav".as_ptr());
                }

                // took a shot, now hide
                if ((*NPC).spawnflags & 2) == 0 && ctx.world.bg_state.rng.Q_irand(0, 1) == 0 {
                    // SPF_NO_HIDE = 2 (file-local #define, NPC_AI_Sniper.c:11)
                    // FIXME: do this if in combat point and combat point has duck-type cover...
                    Sniper_StartHide(ctx);
                } else {
                    let npc_id = ctx.entity_id_of(NPC);
                    TIMER_Set(
                        ctx,
                        npc_id,
                        c"attackDelay".as_ptr(),
                        (*NPCInfo).shotTime - ctx.world.level.time,
                    );
                }
            }
        }
    }
}

/// Raven `NPC_BSSniper_Default`.
///
/// Source: `oracle/codemp/game/NPC_AI_Sniper.c:854-864`
pub fn NPC_BSSniper_Default(ctx: &mut GameContext) {
    unsafe {
        let NPC = ctx.world.globals.NPC as *mut gentity_t;

        if (*NPC).enemy == None {
            // don't have an enemy, look for one
            NPC_BSSniper_Patrol(ctx);
        } else {
            // have an enemy
            NPC_BSSniper_Attack(ctx);
        }
    }
}
