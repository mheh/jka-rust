// PORT-COMPLETE: NPC_AI_Grenadier.c 3/8
//! FAITHFUL port of `oracle/codemp/game/NPC_AI_Grenadier.c`.
//!
//! Filled by the jampgame mega-pass; functions reach file-scope game state
//! (`level`, `g_entities`, cvars) and engine traps through the threaded
//! `GameContext`/`GameWorld` handle.
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_public_consts::SVF_GLASS_BRUSH;
use crate::g_timer::TIMER_Set;
use crate::npc::nav_info_s::NIF_COLLISION;
use crate::npc::script_flags::{
    SCF_CHASE_ENEMIES, SCF_DONT_FIRE, SCF_FIRE_WEAPON, SCF_IGNORE_ALERTS, SCF_LOOK_FOR_ENEMIES,
    SCF_USE_CP_NEAREST,
};
use crate::prelude::*;
use crate::q_math::{PITCH, YAW};
use crate::NPC_combat::G_ClearEnemy;
use crate::NPC_reactions::NPC_Pain;
use crate::NPC_sounds::G_AddVoiceEvent;
use mp_bg::public::entity_event::entity_event_t::{
    EV_CONFUSE1, EV_CONFUSE3, EV_PUSHED1, EV_PUSHED3,
};
use mp_qshared::common::mp::qcommon::b_state_t::bState_t;

// Raven's anonymous `enum { LSTATE_NONE, LSTATE_UNDERFIRE, LSTATE_INVESTIGATE }`
// (file-scope local state, `gNPC_t::localState`) — not a central type, ported
// as file-local consts matching the C values.
// Source: `oracle/codemp/game/NPC_AI_Grenadier.c:42-47`
const LSTATE_NONE: i32 = 0;
pub const LSTATE_UNDERFIRE: i32 = 1;
pub const LSTATE_INVESTIGATE: i32 = 2;

// Raven's `enum { SQUAD_IDLE, SQUAD_STAND_AND_SHOOT, ... }` from `ai.h`
// (squad state selector, `gNPC_t::squadState`) — file-scope consts matching C values.
// Source: `oracle/codemp/game/ai.h:36-43`
const SQUAD_IDLE: i32 = 0;
const SQUAD_STAND_AND_SHOOT: i32 = 1;
const SQUAD_RETREAT: i32 = 2;
const SQUAD_COVER: i32 = 3;
const SQUAD_TRANSITION: i32 = 4;
const SQUAD_POINT: i32 = 5;
const SQUAD_SCOUT: i32 = 6;

// Combat point flags (`combatPoint_t` request bits):
// `crate::npc::combat_point_flags` (`b_local.h:244-259`).

/// Raven `Grenadier_ClearTimers`.
///
/// Source: `oracle/codemp/game/NPC_AI_Grenadier.c:49-63`
pub fn Grenadier_ClearTimers(ctx: &mut GameContext, ent: EntityId) {
    TIMER_Set(ctx, Some(ent), c"chatter".as_ptr() as *const c_char, 0);
    TIMER_Set(ctx, Some(ent), c"duck".as_ptr() as *const c_char, 0);
    TIMER_Set(ctx, Some(ent), c"stand".as_ptr() as *const c_char, 0);
    TIMER_Set(ctx, Some(ent), c"shuffleTime".as_ptr() as *const c_char, 0);
    TIMER_Set(ctx, Some(ent), c"sleepTime".as_ptr() as *const c_char, 0);
    TIMER_Set(
        ctx,
        Some(ent),
        c"enemyLastVisible".as_ptr() as *const c_char,
        0,
    );
    TIMER_Set(ctx, Some(ent), c"roamTime".as_ptr() as *const c_char, 0);
    TIMER_Set(ctx, Some(ent), c"hideTime".as_ptr() as *const c_char, 0);
    // FIXME: Slant for difficulty levels (Raven comment).
    TIMER_Set(ctx, Some(ent), c"attackDelay".as_ptr() as *const c_char, 0);
    TIMER_Set(ctx, Some(ent), c"stick".as_ptr() as *const c_char, 0);
    TIMER_Set(ctx, Some(ent), c"scoutTime".as_ptr() as *const c_char, 0);
    TIMER_Set(ctx, Some(ent), c"flee".as_ptr() as *const c_char, 0);
}

/// Raven `NPC_Grenadier_PlayConfusionSound`.
///
/// Source: `oracle/codemp/game/NPC_AI_Grenadier.c:65-81`
pub fn NPC_Grenadier_PlayConfusionSound(ctx: &mut GameContext, self_: EntityId) {
    // FIXME: make this a custom sound in sound set (Raven comment).
    if ctx.world.entity(self_).health > 0 {
        let confuse_event = ctx
            .world
            .bg_state
            .rng
            .Q_irand(EV_CONFUSE1 as c_int, EV_CONFUSE3 as c_int);
        G_AddVoiceEvent(ctx, self_, confuse_event, 2000);
    }
    // reset him to be totally unaware again
    TIMER_Set(
        ctx,
        Some(self_),
        c"enemyLastVisible".as_ptr() as *const c_char,
        0,
    );
    TIMER_Set(ctx, Some(self_), c"flee".as_ptr() as *const c_char, 0);
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let npc = ctx.world.entity(self_).NPC;
    unsafe {
        (*npc).squadState = SQUAD_IDLE;
        (*npc).tempBehavior = bState_t::BS_DEFAULT;
    }
    G_ClearEnemy(ctx, self_); // FIXME: or just self->enemy = NULL;? (Raven comment).
    unsafe {
        (*npc).investigateCount = 0;
    }
}

/// Raven `NPC_Grenadier_Pain`.
///
/// Source: `oracle/codemp/game/NPC_AI_Grenadier.c:90-103`
pub fn NPC_Grenadier_Pain(
    ctx: &mut GameContext,
    self_: EntityId,
    attacker: Option<EntityId>,
    damage: c_int,
) {
    // FLAG: gNPC_t (NPCInfo) has no accessor; deref stays raw (recipe 2c).
    let npc = ctx.world.entity(self_).NPC;
    unsafe {
        (*npc).localState = LSTATE_UNDERFIRE;
    }

    TIMER_Set(ctx, Some(self_), c"duck".as_ptr() as *const c_char, -1);
    TIMER_Set(ctx, Some(self_), c"stand".as_ptr() as *const c_char, 2000);

    NPC_Pain(ctx, self_, attacker, damage);

    if damage == 0 && ctx.world.entity(self_).health > 0 {
        let pushed_event = ctx
            .world
            .bg_state
            .rng
            .Q_irand(EV_PUSHED1 as c_int, EV_PUSHED3 as c_int);
        // FIXME: better way to know I was pushed (Raven comment).
        G_AddVoiceEvent(ctx, self_, pushed_event, 2000);
    }
}

/// Raven `Grenadier_HoldPosition`.
///
/// Source: `oracle/codemp/game/NPC_AI_Grenadier.c:111-121`
pub fn Grenadier_HoldPosition(ctx: &mut GameContext) {
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let npc_info_ptr = ctx.world.globals.NPCInfo;

    if !npc_info_ptr.is_null() {
        let combat_point = unsafe { (*npc_info_ptr).combatPoint };
        NPC_FreeCombatPoint(ctx, combat_point, qtrue);
        unsafe {
            (*npc_info_ptr).goalEntity = None;
        }
    }
}

/// Raven `Grenadier_Move`.
///
/// Source: `oracle/codemp/game/NPC_AI_Grenadier.c:129-182`
pub fn Grenadier_Move(ctx: &mut GameContext) -> qboolean {
    let npc_ptr = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let npc_info_ptr = ctx.world.globals.NPCInfo;

    if npc_info_ptr.is_null() || npc_ptr.is_null() {
        return qfalse;
    }
    let npc_id = ctx.entity_id_of(npc_ptr).unwrap();

    unsafe {
        (*npc_info_ptr).combatMove = qtrue;
    }
    let moved = NPC_MoveToGoal(ctx, qtrue);

    // Get the move info
    let mut info: navInfo_t = unsafe { core::mem::zeroed() };
    NAV_GetLastMove(ctx, &mut info);

    // If we hit our target, then stop and fire!
    if (info.flags & NIF_COLLISION) != 0 {
        if unsafe { ent_id_opt(ctx.world.g_entities.as_ptr(), info.blocker) }
            == ctx.world.entity(npc_id).enemy
        {
            Grenadier_HoldPosition(ctx);
        }
    }

    // If our move failed, then reset
    if moved == qfalse {
        // FLAG: NPC carries a BG_Alloc'd pool client (not level.clients); deref
        // raw via the safe entity borrow, per trap 2b.
        let client = ctx.world.entity(npc_id).client;
        let enemy = ctx.world.entity(npc_id).enemy;
        // couldn't get to enemy
        if unsafe {
            ((*npc_info_ptr).scriptFlags & SCF_CHASE_ENEMIES) != 0
                && (*client).ps.weapon == WP_THERMAL
                && !(*npc_info_ptr).goalEntity.is_none()
                && (*npc_info_ptr).goalEntity == enemy
        } {
            // we were running after enemy
            // Try to find a combat point that can hit the enemy
            let mut cpFlags = CP_CLEAR | CP_HAS_ROUTE;

            if unsafe { ((*npc_info_ptr).scriptFlags & SCF_USE_CP_NEAREST) != 0 } {
                cpFlags &= !(CP_FLANK | CP_APPROACH_ENEMY | CP_CLOSEST);
                cpFlags |= CP_NEAREST;
            }

            let origin = ctx.world.entity(npc_id).r.currentOrigin;
            let mut cp = NPC_FindCombatPoint(ctx, origin, origin, origin, cpFlags, 32.0, -1);

            if cp == -1 && unsafe { ((*npc_info_ptr).scriptFlags & SCF_USE_CP_NEAREST) == 0 } {
                // okay, try one by the enemy
                // `goalEntity == enemy` (checked above) and `goalEntity` is
                // `Some` here, so `enemy` is guaranteed `Some` too.
                let enemy_origin = ctx.world.entity(enemy.unwrap()).r.currentOrigin;
                cp = NPC_FindCombatPoint(
                    ctx,
                    origin,
                    origin,
                    enemy_origin,
                    CP_CLEAR | CP_HAS_ROUTE | CP_HORZ_DIST_COLL,
                    32.0,
                    -1,
                );
            }

            // NOTE: there may be a perfectly valid one, just not one within CP_COLLECT_RADIUS
            if cp != -1 {
                // found a combat point that has a clear shot to enemy
                NPC_SetCombatPoint(ctx, cp);
                let cp_origin = ctx.world.level.combatPoints[cp as usize].origin;
                NPC_SetMoveGoal(ctx, npc_id, cp_origin, 8, qtrue, cp, None);
                return moved;
            }
        }
        // just hang here
        Grenadier_HoldPosition(ctx);
    }

    moved
}

/// Raven `NPC_BSGrenadier_Patrol`.
///
/// Source: `oracle/codemp/game/NPC_AI_Grenadier.c:190-277`
pub fn NPC_BSGrenadier_Patrol(ctx: &mut GameContext) {
    let npc_ptr = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let npc_info_ptr = ctx.world.globals.NPCInfo;

    if npc_info_ptr.is_null() || npc_ptr.is_null() {
        return;
    }
    let npc_id = ctx.entity_id_of(npc_ptr).unwrap();

    if unsafe { (*npc_info_ptr).confusionTime } < ctx.world.level.time {
        // Look for any enemies
        if unsafe { ((*npc_info_ptr).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 } {
            if NPC_CheckPlayerTeamStealth(ctx) != qfalse {
                NPC_UpdateAngles(ctx, qtrue, qtrue);
                return;
            }
        }

        if unsafe { ((*npc_info_ptr).scriptFlags & SCF_IGNORE_ALERTS) == 0 } {
            // Is there danger nearby
            let alertEvent = NPC_CheckAlertEvents(
                ctx,
                qtrue,
                qtrue,
                -1,
                qfalse,
                alertEventLevel_e::AEL_SUSPICIOUS as c_int,
            );
            if NPC_CheckForDanger(ctx, alertEvent) != qfalse {
                NPC_UpdateAngles(ctx, qtrue, qtrue);
                return;
            } else {
                // check for other alert events
                // There is an event to look at
                if alertEvent >= 0
                    && ctx.world.level.alertEvents[alertEvent as usize].ID
                        != unsafe { (*npc_info_ptr).lastAlertID }
                {
                    let alert_id = ctx.world.level.alertEvents[alertEvent as usize].ID;
                    unsafe {
                        (*npc_info_ptr).lastAlertID = alert_id;
                    }
                    if ctx.world.level.alertEvents[alertEvent as usize].level
                        == alertEventLevel_e::AEL_DISCOVERED
                    {
                        let owner = ctx.world.level.alertEvents[alertEvent as usize].owner;
                        // FLAG: pool clients (owner may be an NPC) — deref raw via
                        // the safe entity borrow, per trap 2b.
                        let is_enemy = if owner.is_null() {
                            false
                        } else {
                            let owner_id = ctx.entity_id_of(owner).unwrap();
                            let owner_client = ctx.world.entity(owner_id).client;
                            let owner_health = ctx.world.entity(owner_id).health;
                            let npc_client = ctx.world.entity(npc_id).client;
                            !owner_client.is_null()
                                && owner_health >= 0
                                && unsafe { (*owner_client).playerTeam == (*npc_client).enemyTeam }
                        };
                        if is_enemy {
                            let owner_id = ctx.entity_id_of(owner);
                            // an enemy
                            G_SetEnemy(ctx, npc_id, owner_id);
                            let attack_delay = ctx.world.bg_state.rng.Q_irand(500, 2500);
                            TIMER_Set(
                                ctx,
                                Some(npc_id),
                                c"attackDelay".as_ptr() as *const c_char,
                                attack_delay,
                            );
                        }
                    } else {
                        // Save the position for movement (if necessary)
                        let pos = ctx.world.level.alertEvents[alertEvent as usize].position;
                        unsafe {
                            crate::q_math::_VectorCopy(pos, &mut (*npc_info_ptr).investigateGoal);
                        }
                        let dbt = ctx.world.level.time + ctx.world.bg_state.rng.Q_irand(500, 1000);
                        unsafe {
                            (*npc_info_ptr).investigateDebounceTime = dbt;
                        }
                        if ctx.world.level.alertEvents[alertEvent as usize].level
                            == alertEventLevel_e::AEL_SUSPICIOUS
                        {
                            // suspicious looks longer
                            let extra = ctx.world.bg_state.rng.Q_irand(500, 2500);
                            unsafe {
                                (*npc_info_ptr).investigateDebounceTime += extra;
                            }
                        }
                    }
                }
            }

            if unsafe { (*npc_info_ptr).investigateDebounceTime } > ctx.world.level.time {
                // FIXME: walk over to it, maybe?  Not if not chase enemies
                // NOTE: stops walking or doing anything else below
                let mut dir: vec3_t = [0.0; 3];
                let mut angles: vec3_t = [0.0; 3];
                let (o_yaw, o_pitch) =
                    unsafe { ((*npc_info_ptr).desiredYaw, (*npc_info_ptr).desiredPitch) };

                // FLAG: NPC carries a BG_Alloc'd pool client — deref raw via the
                // safe entity borrow, per trap 2b.
                let npc_client = ctx.world.entity(npc_id).client;
                let (investigate_goal, eye_point) = unsafe {
                    (
                        (*npc_info_ptr).investigateGoal,
                        (*npc_client).renderInfo.eyePoint,
                    )
                };
                crate::q_math::_VectorSubtract(investigate_goal, eye_point, &mut dir);
                vectoangles(dir, &mut angles);

                unsafe {
                    (*npc_info_ptr).desiredYaw = angles[YAW];
                    (*npc_info_ptr).desiredPitch = angles[PITCH];
                }

                NPC_UpdateAngles(ctx, qtrue, qtrue);

                unsafe {
                    (*npc_info_ptr).desiredYaw = o_yaw;
                    (*npc_info_ptr).desiredPitch = o_pitch;
                }
                return;
            }
        }
    }

    // If we have somewhere to go, then do that
    if !UpdateGoal(ctx).is_null() {
        ctx.world.globals.ucmd.buttons |=
            mp_qshared::common::mp::qcommon::usercmd_button::BUTTON_WALKING as c_int;
        NPC_MoveToGoal(ctx, qtrue);
    }

    NPC_UpdateAngles(ctx, qtrue, qtrue);
}

/// Raven `Grenadier_CheckMoveState`.
///
/// Source: `oracle/codemp/game/NPC_AI_Grenadier.c:307-391`
pub fn Grenadier_CheckMoveState(ctx: &mut GameContext) {
    let npc_ptr = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let npc_info_ptr = ctx.world.globals.NPCInfo;

    if npc_info_ptr.is_null() || npc_ptr.is_null() {
        return;
    }
    let npc_id = ctx.entity_id_of(npc_ptr).unwrap();

    // See if we're a scout
    if unsafe { ((*npc_info_ptr).scriptFlags & SCF_CHASE_ENEMIES) == 0 } {
        if unsafe { (*npc_info_ptr).goalEntity } == ctx.world.entity(npc_id).enemy {
            ctx.world.globals.move3 = qfalse;
            return;
        }
    }
    // See if we're running away
    else if unsafe { (*npc_info_ptr).squadState == SQUAD_RETREAT } {
        if TIMER_Done(ctx, Some(npc_id), c"flee".as_ptr() as *const c_char) != qfalse {
            unsafe {
                (*npc_info_ptr).squadState = SQUAD_IDLE;
            }
        } else {
            ctx.world.globals.faceEnemy3 = qfalse;
        }
    }

    // See if we're moving towards a goal, not the enemy
    let goal_entity = unsafe { (*npc_info_ptr).goalEntity };
    if goal_entity != ctx.world.entity(npc_id).enemy && !goal_entity.is_none() {
        // Did we make it?
        let origin = ctx.world.entity(npc_id).r.currentOrigin;
        let mins = ctx.world.entity(npc_id).r.mins;
        let maxs = ctx.world.entity(npc_id).r.maxs;
        // guarded by `!goalEntity.is_none()` above.
        let goal_origin = ctx.world.entity(goal_entity.unwrap()).r.currentOrigin;
        let flying = FlyingCreature(ctx.world.entity(npc_id));
        let squad_state = unsafe { (*npc_info_ptr).squadState };
        if NAV_HitNavGoal(origin, mins, maxs, goal_origin, 16, flying) != qfalse
            || (squad_state == SQUAD_SCOUT
                && ctx.world.globals.enemyLOS3 != qfalse
                && ctx.world.globals.enemyDist3 <= 10000.0)
        {
            // Oracle assigns the dead local `newSquadState` here (never written back
            // to NPCInfo->squadState), so squadState stays SQUAD_RETREAT and the later
            // `== SQUAD_RETREAT` flee/IDLE reset fires. Preserve that quirk (§20).
            let mut newSquadState = SQUAD_STAND_AND_SHOOT;
            // we got where we wanted to go, set timers based on why we were running
            match squad_state {
                SQUAD_RETREAT => {
                    // FLAG: NPC carries a BG_Alloc'd pool client — deref raw via
                    // the safe entity borrow, per trap 2b.
                    let client = ctx.world.entity(npc_id).client;
                    let health = ctx.world.entity(npc_id).health;
                    // was running away
                    let duck_val = unsafe { client.as_ref() }
                        .map(|c| ((c.pers.maxHealth - health) * 100) as c_int)
                        .unwrap_or(0);
                    TIMER_Set(
                        ctx,
                        Some(npc_id),
                        c"duck".as_ptr() as *const c_char,
                        duck_val,
                    );
                    let hide_delay = ctx.world.bg_state.rng.Q_irand(3000, 7000);
                    TIMER_Set(
                        ctx,
                        Some(npc_id),
                        c"hideTime".as_ptr() as *const c_char,
                        hide_delay,
                    );
                    newSquadState = SQUAD_COVER;
                }
                SQUAD_TRANSITION => {
                    let hide_delay = ctx.world.bg_state.rng.Q_irand(2000, 4000);
                    // was heading for a combat point
                    TIMER_Set(
                        ctx,
                        Some(npc_id),
                        c"hideTime".as_ptr() as *const c_char,
                        hide_delay,
                    );
                }
                SQUAD_SCOUT => {
                    // was running after player
                }
                _ => {}
            }
            NPC_ReachedGoal(ctx);
            let attack_delay = ctx.world.bg_state.rng.Q_irand(250, 500);
            // don't attack right away
            TIMER_Set(
                ctx,
                Some(npc_id),
                c"attackDelay".as_ptr() as *const c_char,
                attack_delay,
            );
            let roam_delay = ctx.world.bg_state.rng.Q_irand(1000, 4000);
            // don't do something else just yet
            TIMER_Set(
                ctx,
                Some(npc_id),
                c"roamTime".as_ptr() as *const c_char,
                roam_delay,
            );
            // stop fleeing
            if unsafe { (*npc_info_ptr).squadState == SQUAD_RETREAT } {
                let neg_time = -ctx.world.level.time;
                TIMER_Set(
                    ctx,
                    Some(npc_id),
                    c"flee".as_ptr() as *const c_char,
                    neg_time,
                );
                unsafe {
                    (*npc_info_ptr).squadState = SQUAD_IDLE;
                }
            }
            return;
        }

        // keep going, hold of roamTimer until we get there
        let roam_delay = ctx.world.bg_state.rng.Q_irand(4000, 8000);
        TIMER_Set(
            ctx,
            Some(npc_id),
            c"roamTime".as_ptr() as *const c_char,
            roam_delay,
        );
    }

    if unsafe { (*npc_info_ptr).goalEntity.is_none() } {
        if unsafe { ((*npc_info_ptr).scriptFlags & SCF_CHASE_ENEMIES) != 0 } {
            let enemy = ctx.world.entity(npc_id).enemy;
            unsafe {
                (*npc_info_ptr).goalEntity = enemy;
            }
        }
    }
}

/// Raven `Grenadier_CheckFireState`.
///
/// Source: `oracle/codemp/game/NPC_AI_Grenadier.c:399-439`
pub fn Grenadier_CheckFireState(ctx: &mut GameContext) {
    let npc_ptr = ctx.world.globals.NPC;

    if ctx.world.globals.enemyCS3 != qfalse {
        // if have a clear shot, always try
        return;
    }

    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let npc_info_ptr = ctx.world.globals.NPCInfo;
    if npc_info_ptr.is_null() || npc_ptr.is_null() {
        return;
    }
    let npc_id = ctx.entity_id_of(npc_ptr).unwrap();

    if unsafe {
        (*npc_info_ptr).squadState == SQUAD_RETREAT
            || (*npc_info_ptr).squadState == SQUAD_TRANSITION
            || (*npc_info_ptr).squadState == SQUAD_SCOUT
    } {
        // runners never try to fire at the last pos
        return;
    }

    // FLAG: NPC carries a BG_Alloc'd pool client — deref raw via the safe entity
    // borrow, per trap 2b.
    let client = ctx.world.entity(npc_id).client;
    if crate::q_math::VectorCompare(unsafe { (*client).ps.velocity }, crate::q_math::vec3_origin)
        == qfalse
    {
        // if moving at all, don't do this
        return;
    }
}

/// Raven `Grenadier_EvaluateShot`.
///
/// Source: `oracle/codemp/game/NPC_AI_Grenadier.c:441-453`
pub fn Grenadier_EvaluateShot(ctx: &mut GameContext, hit: c_int) -> qboolean {
    let npc_ptr = ctx.world.globals.NPC;

    if npc_ptr.is_null() {
        return qfalse;
    }
    let npc_id = ctx.entity_id_of(npc_ptr).unwrap();
    let enemy = ctx.world.entity(npc_id).enemy;
    if enemy.is_none() {
        return qfalse;
    }

    if hit == ctx.world.entity(enemy.unwrap()).s.number {
        // can hit enemy
        return qtrue;
    }

    // §19: oracle indexes `g_entities[hit]` unguarded (`&g_entities[hit] != NULL`
    // is always true); the bounds check avoids a panic on a bad index.
    // Source: oracle/codemp/game/NPC_AI_Grenadier.c:448
    if hit >= 0 && (hit as usize) < mp_qshared::shared::MAX_GENTITIES {
        let hit_ent = &ctx.world.g_entities[hit as usize];
        if (hit_ent.r.svFlags & SVF_GLASS_BRUSH as i32) != 0 {
            // will hit glass, so shoot anyway
            return qtrue;
        }
    }

    qfalse
}

/// Raven `NPC_BSGrenadier_Attack`.
///
/// Source: `oracle/codemp/game/NPC_AI_Grenadier.c:461-662`
pub fn NPC_BSGrenadier_Attack(ctx: &mut GameContext) {
    let npc_ptr = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let npc_info_ptr = ctx.world.globals.NPCInfo;

    if npc_info_ptr.is_null() || npc_ptr.is_null() {
        return;
    }
    let npc_id = ctx.entity_id_of(npc_ptr).unwrap();

    // Don't do anything if we're hurt
    if ctx.world.entity(npc_id).painDebounceTime > ctx.world.level.time {
        NPC_UpdateAngles(ctx, qtrue, qtrue);
        return;
    }

    // If we don't have an enemy, just idle
    if NPC_CheckEnemyExt(ctx, qfalse) == qfalse {
        ctx.world.entity_mut(npc_id).enemy = None;
        NPC_BSGrenadier_Patrol(ctx);
        return;
    }

    // Oracle short-circuit: NPC_CheckAlertEvents (side-effectful) runs only
    // when the flee timer is done (NPC_AI_Grenadier.c:461-478).
    if TIMER_Done(ctx, Some(npc_id), c"flee".as_ptr() as *const c_char) != qfalse && {
        let alert_event = NPC_CheckAlertEvents(
            ctx,
            qtrue,
            qtrue,
            -1,
            qfalse,
            alertEventLevel_e::AEL_DANGER as c_int,
        );
        NPC_CheckForDanger(ctx, alert_event) != qfalse
    } {
        // going to run
        NPC_UpdateAngles(ctx, qtrue, qtrue);
        return;
    }

    if ctx.world.entity(npc_id).enemy.is_none() {
        // WTF?  somehow we lost our enemy?
        NPC_BSGrenadier_Patrol(ctx);
        return;
    }

    // Guaranteed `Some` from here to the end of the function by the guard above.
    let enemy_id = ctx.world.entity(npc_id).enemy.unwrap();

    ctx.world.globals.enemyLOS3 = qfalse;
    ctx.world.globals.enemyCS3 = qfalse;
    ctx.world.globals.move3 = qtrue;
    ctx.world.globals.faceEnemy3 = qfalse;
    ctx.world.globals.shoot3 = qfalse;
    let enemy_origin = ctx.world.entity(enemy_id).r.currentOrigin;
    let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
    ctx.world.globals.enemyDist3 = DistanceSquared(enemy_origin, npc_origin);

    // FLAG: NPC/enemy carry BG_Alloc'd pool clients — deref raw via the safe
    // entity borrow, per trap 2b.
    let npc_client = ctx.world.entity(npc_id).client;
    let enemy_client = ctx.world.entity(enemy_id).client;
    // See if we should switch to melee attack
    if ctx.world.globals.enemyDist3 < 16384.0
        && (enemy_client.is_null()
            || unsafe { (*enemy_client).ps.weapon } != mp_bg::weapons::weapon_t::WP_SABER
            || BG_SabersOff(unsafe { &mut (*enemy_client).ps }) != qfalse)
    {
        // enemy is close and not using saber
        if unsafe { (*npc_client).ps.weapon } == WP_THERMAL {
            // grenadier
            let mut trace: trace_t = unsafe { core::mem::zeroed() };
            let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
            let enemy_mins = ctx.world.entity(enemy_id).r.mins;
            let enemy_maxs = ctx.world.entity(enemy_id).r.maxs;
            let enemy_origin = ctx.world.entity(enemy_id).r.currentOrigin;
            let npc_num = ctx.world.entity(npc_id).s.number;
            let enemy_clipmask = ctx.world.entity(enemy_id).clipmask;
            trap::Trace(
                ctx.engine,
                mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                    &mut trace,
                    &npc_origin,
                    &enemy_mins,
                    &enemy_maxs,
                    &enemy_origin,
                    npc_num,
                    enemy_clipmask,
                ),
            );
            let enemy_num = ctx.world.entity(enemy_id).s.number;
            if trace.allsolid == 0
                && trace.startsolid == 0
                && (trace.fraction == 1.0 || trace.entityNum as c_int == enemy_num)
            {
                // I can get right to him
                // reset fire-timing variables
                NPC_ChangeWeapon(WP_STUN_BATON);
                if unsafe { ((*npc_info_ptr).scriptFlags & SCF_CHASE_ENEMIES) == 0 } {
                    unsafe {
                        (*npc_info_ptr).scriptFlags |= SCF_CHASE_ENEMIES;
                    }
                }
            }
        }
    } else if ctx.world.globals.enemyDist3 > 65536.0
        || (!enemy_client.is_null()
            && unsafe { (*enemy_client).ps.weapon } == mp_bg::weapons::weapon_t::WP_SABER
            && unsafe { (*enemy_client).ps.saberHolstered } == 0)
    {
        // enemy is far or using saber
        if unsafe { (*npc_client).ps.weapon } == WP_STUN_BATON
            && (unsafe { (*npc_client).ps.stats[STAT_WEAPONS as usize] } & (1 << WP_THERMAL)) != 0
        {
            // fisticuffs, make switch to thermal if have it
            // reset fire-timing variables
            NPC_ChangeWeapon(WP_THERMAL);
        }
    }

    // can we see our target?
    if NPC_ClearLOS4(ctx, Some(enemy_id)) != qfalse {
        let level_time = ctx.world.level.time;
        unsafe {
            (*npc_info_ptr).enemyLastSeenTime = level_time;
        }
        ctx.world.globals.enemyLOS3 = qtrue;

        // FLAG: NPC carries a BG_Alloc'd pool client — deref raw via the safe
        // entity borrow, per trap 2b.
        let npc_client = ctx.world.entity(npc_id).client;
        if unsafe { (*npc_client).ps.weapon } == WP_STUN_BATON {
            let enemy_origin = ctx.world.entity(enemy_id).r.currentOrigin;
            let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
            let viewangles = unsafe { (*npc_client).ps.viewangles };
            if ctx.world.globals.enemyDist3 <= 4096.0
                && InFOV3(enemy_origin, npc_origin, viewangles, 90, 45) != qfalse
            {
                // within 64 & infront
                let els = ctx.world.entity(enemy_id).r.currentOrigin;
                unsafe {
                    crate::q_math::_VectorCopy(els, &mut (*npc_info_ptr).enemyLastSeenLocation);
                }
                ctx.world.globals.enemyCS3 = qtrue;
            }
        } else {
            let enemy_origin = ctx.world.entity(enemy_id).r.currentOrigin;
            let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
            let viewangles = unsafe { (*npc_client).ps.viewangles };
            if InFOV3(enemy_origin, npc_origin, viewangles, 45, 90) != qfalse {
                // in front of me
                // can we shoot our target?
                let hit = NPC_ShotEntity(ctx, Some(enemy_id), None);
                let enemy_num = ctx.world.entity(enemy_id).s.number;
                let hit_matches = if hit == enemy_num {
                    true
                } else {
                    // FLAG: hit entity may be an NPC — pool client deref raw, per trap 2b.
                    let hit_client = ctx.world.g_entities[hit as usize].client;
                    let npc_client2 = ctx.world.entity(npc_id).client;
                    !hit_client.is_null()
                        && unsafe { (*hit_client).playerTeam == (*npc_client2).enemyTeam }
                };
                if hit_matches {
                    let enemy_origin = ctx.world.entity(enemy_id).r.currentOrigin;
                    let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
                    let enemyHorzDist = DistanceHorizontalSquared(enemy_origin, npc_origin);
                    let els = ctx.world.entity(enemy_id).r.currentOrigin;
                    unsafe {
                        crate::q_math::_VectorCopy(els, &mut (*npc_info_ptr).enemyLastSeenLocation);
                    }

                    if enemyHorzDist < 1048576.0 {
                        // within 1024
                        ctx.world.globals.enemyCS3 = qtrue;
                        NPC_AimAdjust(ctx, 2); // adjust aim better longer we have clear shot at enemy
                    } else {
                        NPC_AimAdjust(ctx, 1); // adjust aim better longer we can see enemy
                    }
                }
            }
        }
    } else {
        NPC_AimAdjust(ctx, -1); // adjust aim worse longer we cannot see enemy
    }

    if ctx.world.globals.enemyLOS3 != qfalse {
        // FIXME: no need to face enemy if we're moving to some other goal and he's too far away to shoot?
        ctx.world.globals.faceEnemy3 = qtrue;
    }

    if ctx.world.globals.enemyCS3 != qfalse {
        ctx.world.globals.shoot3 = qtrue;
        // FLAG: NPC carries a BG_Alloc'd pool client — deref raw via the safe
        // entity borrow, per trap 2b.
        let npc_client = ctx.world.entity(npc_id).client;
        let weapon = unsafe { (*npc_client).ps.weapon };
        if weapon == WP_THERMAL {
            // don't chase and throw
            ctx.world.globals.move3 = qfalse;
        } else if weapon == WP_STUN_BATON && {
            let npc_max0 = ctx.world.entity(npc_id).r.maxs[0];
            let enemy_max0 = ctx.world.entity(enemy_id).r.maxs[0];
            ctx.world.globals.enemyDist3
                < ((npc_max0 + enemy_max0 + 16.0) * (npc_max0 + enemy_max0 + 16.0))
        } {
            // close enough
            ctx.world.globals.move3 = qfalse;
        }
    }

    // Check for movement to take care of
    Grenadier_CheckMoveState(ctx);

    // See if we should override shooting decision with any special considerations
    Grenadier_CheckFireState(ctx);

    if ctx.world.globals.move3 != qfalse {
        // move toward goal
        if !unsafe { (*npc_info_ptr).goalEntity.is_none() } {
            ctx.world.globals.move3 = Grenadier_Move(ctx);
        } else {
            ctx.world.globals.move3 = qfalse;
        }
    }

    if ctx.world.globals.move3 == qfalse {
        if TIMER_Done(ctx, Some(npc_id), c"duck".as_ptr() as *const c_char) == qfalse {
            ctx.world.globals.ucmd.upmove = -127;
        }
    } else {
        // stop ducking!
        TIMER_Set(ctx, Some(npc_id), c"duck".as_ptr() as *const c_char, -1);
    }

    if ctx.world.globals.faceEnemy3 == qfalse {
        // we want to face in the dir we're running
        if ctx.world.globals.move3 != qfalse {
            // don't run away and shoot
            let last_yaw = unsafe { (*npc_info_ptr).lastPathAngles[YAW] };
            unsafe {
                (*npc_info_ptr).desiredYaw = last_yaw;
                (*npc_info_ptr).desiredPitch = 0.0;
            }
            ctx.world.globals.shoot3 = qfalse;
        }
        NPC_UpdateAngles(ctx, qtrue, qtrue);
    } else {
        // face the enemy
        NPC_FaceEnemy(ctx, qtrue);
    }

    if unsafe { ((*npc_info_ptr).scriptFlags & SCF_DONT_FIRE) != 0 } {
        ctx.world.globals.shoot3 = qfalse;
    }

    // FIXME: don't shoot right away!
    if ctx.world.globals.shoot3 != qfalse {
        // try to shoot if it's time
        if TIMER_Done(ctx, Some(npc_id), c"attackDelay".as_ptr() as *const c_char) != qfalse {
            if unsafe { ((*npc_info_ptr).scriptFlags & SCF_FIRE_WEAPON) == 0 } {
                // we've already fired, no need to do it again here
                WeaponThink(ctx, qtrue);
                let delay = unsafe { (*npc_info_ptr).shotTime } - ctx.world.level.time;
                TIMER_Set(
                    ctx,
                    Some(npc_id),
                    c"attackDelay".as_ptr() as *const c_char,
                    delay,
                );
            }
        }
    }
}

/// Raven `NPC_BSGrenadier_Default`.
///
/// Source: `oracle/codemp/game/NPC_AI_Grenadier.c:664-679`
pub fn NPC_BSGrenadier_Default(ctx: &mut GameContext) {
    let npc_ptr = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let npc_info_ptr = ctx.world.globals.NPCInfo;

    if npc_info_ptr.is_null() || npc_ptr.is_null() {
        return;
    }
    let npc_id = ctx.entity_id_of(npc_ptr).unwrap();

    if unsafe { ((*npc_info_ptr).scriptFlags & SCF_FIRE_WEAPON) != 0 } {
        WeaponThink(ctx, qtrue);
    }

    if ctx.world.entity(npc_id).enemy.is_none() {
        // don't have an enemy, look for one
        NPC_BSGrenadier_Patrol(ctx);
    } else {
        // have an enemy
        NPC_BSGrenadier_Attack(ctx);
    }
}
