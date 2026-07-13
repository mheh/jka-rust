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

// EntityId seam helper: resolve `Option<EntityId>` back to the raw pointer the
// verbatim body still expects (`None` -> null), per the `NPC_AI_Stormtrooper.rs`
// precedent.
#[inline]
unsafe fn ent_resolve_opt(ctx: &mut GameContext, id: Option<EntityId>) -> *mut gentity_t {
    match id {
        Some(i) => unsafe { &mut (*ctx.world_raw()).g_entities[i.index()] as *mut gentity_t },
        None => core::ptr::null_mut(),
    }
}

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
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);
    TIMER_Set(
        ctx,
        ctx.entity_id_of(ent),
        c"chatter".as_ptr() as *const c_char,
        0,
    );
    TIMER_Set(
        ctx,
        ctx.entity_id_of(ent),
        c"duck".as_ptr() as *const c_char,
        0,
    );
    TIMER_Set(
        ctx,
        ctx.entity_id_of(ent),
        c"stand".as_ptr() as *const c_char,
        0,
    );
    TIMER_Set(
        ctx,
        ctx.entity_id_of(ent),
        c"shuffleTime".as_ptr() as *const c_char,
        0,
    );
    TIMER_Set(
        ctx,
        ctx.entity_id_of(ent),
        c"sleepTime".as_ptr() as *const c_char,
        0,
    );
    TIMER_Set(
        ctx,
        ctx.entity_id_of(ent),
        c"enemyLastVisible".as_ptr() as *const c_char,
        0,
    );
    TIMER_Set(
        ctx,
        ctx.entity_id_of(ent),
        c"roamTime".as_ptr() as *const c_char,
        0,
    );
    TIMER_Set(
        ctx,
        ctx.entity_id_of(ent),
        c"hideTime".as_ptr() as *const c_char,
        0,
    );
    // FIXME: Slant for difficulty levels (Raven comment).
    TIMER_Set(
        ctx,
        ctx.entity_id_of(ent),
        c"attackDelay".as_ptr() as *const c_char,
        0,
    );
    TIMER_Set(
        ctx,
        ctx.entity_id_of(ent),
        c"stick".as_ptr() as *const c_char,
        0,
    );
    TIMER_Set(
        ctx,
        ctx.entity_id_of(ent),
        c"scoutTime".as_ptr() as *const c_char,
        0,
    );
    TIMER_Set(
        ctx,
        ctx.entity_id_of(ent),
        c"flee".as_ptr() as *const c_char,
        0,
    );
}

/// Raven `NPC_Grenadier_PlayConfusionSound`.
///
/// Source: `oracle/codemp/game/NPC_AI_Grenadier.c:65-81`
pub fn NPC_Grenadier_PlayConfusionSound(ctx: &mut GameContext, self_: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    unsafe {
        // FIXME: make this a custom sound in sound set (Raven comment).
        if (*self_).health > 0 {
            let __h66 = ctx.entity_id_of(self_).unwrap();
            let __h67 = (*ctx.world_raw())
                .bg_state
                .rng
                .Q_irand(EV_CONFUSE1 as c_int, EV_CONFUSE3 as c_int);
            G_AddVoiceEvent(ctx, __h66, __h67, 2000);
        }
        // reset him to be totally unaware again
        TIMER_Set(
            ctx,
            ctx.entity_id_of(self_),
            c"enemyLastVisible".as_ptr() as *const c_char,
            0,
        );
        TIMER_Set(
            ctx,
            ctx.entity_id_of(self_),
            c"flee".as_ptr() as *const c_char,
            0,
        );
        let npc = (*self_).NPC as *mut gNPC_t;
        (*npc).squadState = SQUAD_IDLE;
        (*npc).tempBehavior = bState_t::BS_DEFAULT;
        G_ClearEnemy(ctx, ctx.entity_id_of(self_).unwrap()); // FIXME: or just self->enemy = NULL;? (Raven comment).
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
    // STAGE-1: EntityId params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    let attacker: *mut gentity_t = unsafe { ent_resolve_opt(ctx, attacker) };
    unsafe {
        let npc = (*self_).NPC as *mut gNPC_t;
        (*npc).localState = LSTATE_UNDERFIRE;

        TIMER_Set(
            ctx,
            ctx.entity_id_of(self_),
            c"duck".as_ptr() as *const c_char,
            -1,
        );
        TIMER_Set(
            ctx,
            ctx.entity_id_of(self_),
            c"stand".as_ptr() as *const c_char,
            2000,
        );

        NPC_Pain(
            ctx,
            ctx.entity_id_of(self_).unwrap(),
            ctx.entity_id_of(attacker),
            damage,
        );

        if damage == 0 && (*self_).health > 0 {
            let __h68 = ctx.entity_id_of(self_).unwrap();
            let __h69 = (*ctx.world_raw())
                .bg_state
                .rng
                .Q_irand(EV_PUSHED1 as c_int, EV_PUSHED3 as c_int);
            // FIXME: better way to know I was pushed (Raven comment).
            G_AddVoiceEvent(ctx, __h68, __h69, 2000);
        }
    }
}

/// Raven `Grenadier_HoldPosition`.
///
/// Source: `oracle/codemp/game/NPC_AI_Grenadier.c:111-121`
pub fn Grenadier_HoldPosition(ctx: &mut GameContext) {
    unsafe {
        let world = &*ctx.world_raw();
        let npc_ptr = world.globals.NPC;
        let npc_info_ptr = world.globals.NPCInfo;

        if !npc_info_ptr.is_null() {
            NPC_FreeCombatPoint(ctx, (*npc_info_ptr).combatPoint, qtrue);
            (*npc_info_ptr).goalEntity = None;
        }
    }
}

/// Raven `Grenadier_Move`.
///
/// Source: `oracle/codemp/game/NPC_AI_Grenadier.c:129-182`
pub fn Grenadier_Move(ctx: &mut GameContext) -> qboolean {
    unsafe {
        let world = &*ctx.world_raw();
        let npc_ptr = world.globals.NPC;
        let npc_info_ptr = world.globals.NPCInfo;

        if npc_info_ptr.is_null() || npc_ptr.is_null() {
            return qfalse;
        }

        (*npc_info_ptr).combatMove = qtrue;
        let moved = NPC_MoveToGoal(ctx, qtrue);

        // Get the move info
        let mut info: navInfo_t = core::mem::zeroed();
        NAV_GetLastMove(ctx, &mut info);

        // If we hit our target, then stop and fire!
        if (info.flags & NIF_COLLISION) != 0 {
            if ent_id_opt(world.g_entities.as_ptr(), info.blocker) == (*npc_ptr).enemy {
                Grenadier_HoldPosition(ctx);
            }
        }

        // If our move failed, then reset
        if moved == qfalse {
            // couldn't get to enemy
            if ((*npc_info_ptr).scriptFlags & SCF_CHASE_ENEMIES) != 0
                && (*((*npc_ptr).client as *mut gclient_t)).ps.weapon == WP_THERMAL
                && !(*npc_info_ptr).goalEntity.is_none()
                && (*npc_info_ptr).goalEntity == (*npc_ptr).enemy
            {
                // we were running after enemy
                // Try to find a combat point that can hit the enemy
                let mut cpFlags = CP_CLEAR | CP_HAS_ROUTE;

                if ((*npc_info_ptr).scriptFlags & SCF_USE_CP_NEAREST) != 0 {
                    cpFlags &= !(CP_FLANK | CP_APPROACH_ENEMY | CP_CLOSEST);
                    cpFlags |= CP_NEAREST;
                }

                let mut cp = NPC_FindCombatPoint(
                    ctx,
                    (*npc_ptr).r.currentOrigin,
                    (*npc_ptr).r.currentOrigin,
                    (*npc_ptr).r.currentOrigin,
                    cpFlags,
                    32.0,
                    -1,
                );

                if cp == -1 && ((*npc_info_ptr).scriptFlags & SCF_USE_CP_NEAREST) == 0 {
                    // okay, try one by the enemy
                    // `goalEntity == enemy` (checked above) and `goalEntity` is
                    // `Some` here, so `enemy` is guaranteed `Some` too.
                    cp = NPC_FindCombatPoint(
                        ctx,
                        (*npc_ptr).r.currentOrigin,
                        (*npc_ptr).r.currentOrigin,
                        world.g_entities[(*npc_ptr).enemy.unwrap().index()]
                            .r
                            .currentOrigin,
                        CP_CLEAR | CP_HAS_ROUTE | CP_HORZ_DIST_COLL,
                        32.0,
                        -1,
                    );
                }

                // NOTE: there may be a perfectly valid one, just not one within CP_COLLECT_RADIUS
                if cp != -1 {
                    // found a combat point that has a clear shot to enemy
                    NPC_SetCombatPoint(ctx, cp);
                    NPC_SetMoveGoal(
                        ctx,
                        ctx.entity_id_of(npc_ptr).unwrap(),
                        world.level.combatPoints[cp as usize].origin,
                        8,
                        qtrue,
                        cp,
                        None,
                    );
                    return moved;
                }
            }
            // just hang here
            Grenadier_HoldPosition(ctx);
        }

        moved
    }
}

/// Raven `NPC_BSGrenadier_Patrol`.
///
/// Source: `oracle/codemp/game/NPC_AI_Grenadier.c:190-277`
pub fn NPC_BSGrenadier_Patrol(ctx: &mut GameContext) {
    unsafe {
        let world = &mut *ctx.world_raw();
        let npc_ptr = world.globals.NPC;
        let npc_info_ptr = world.globals.NPCInfo;

        if npc_info_ptr.is_null() || npc_ptr.is_null() {
            return;
        }

        if (*npc_info_ptr).confusionTime < world.level.time {
            // Look for any enemies
            if ((*npc_info_ptr).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
                if NPC_CheckPlayerTeamStealth(ctx) != qfalse {
                    NPC_UpdateAngles(ctx, qtrue, qtrue);
                    return;
                }
            }

            if ((*npc_info_ptr).scriptFlags & SCF_IGNORE_ALERTS) == 0 {
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
                        && world.level.alertEvents[alertEvent as usize].ID
                            != (*npc_info_ptr).lastAlertID
                    {
                        (*npc_info_ptr).lastAlertID =
                            world.level.alertEvents[alertEvent as usize].ID;
                        if world.level.alertEvents[alertEvent as usize].level
                            == alertEventLevel_e::AEL_DISCOVERED
                        {
                            if !world.level.alertEvents[alertEvent as usize].owner.is_null()
                                && !(*world.level.alertEvents[alertEvent as usize].owner)
                                    .client
                                    .is_null()
                                && (*world.level.alertEvents[alertEvent as usize].owner).health >= 0
                                && (*((*world.level.alertEvents[alertEvent as usize].owner).client
                                    as *mut gclient_t))
                                    .playerTeam
                                    == (*((*npc_ptr).client as *mut gclient_t)).enemyTeam
                            {
                                let __h70 = ctx.entity_id_of(npc_ptr).unwrap();
                                let __h71 = ctx.entity_id_of(
                                    world.level.alertEvents[alertEvent as usize].owner,
                                );
                                // an enemy
                                G_SetEnemy(ctx, __h70, __h71);
                                let __h72 = ctx.entity_id_of(npc_ptr);
                                let __h73 = (*ctx.world_raw()).bg_state.rng.Q_irand(500, 2500);
                                TIMER_Set(
                                    ctx,
                                    __h72,
                                    c"attackDelay".as_ptr() as *const c_char,
                                    __h73,
                                );
                            }
                        } else {
                            // Save the position for movement (if necessary)
                            crate::q_math::_VectorCopy(
                                world.level.alertEvents[alertEvent as usize].position,
                                &mut (*npc_info_ptr).investigateGoal,
                            );
                            (*npc_info_ptr).investigateDebounceTime = world.level.time
                                + (*ctx.world_raw()).bg_state.rng.Q_irand(500, 1000);
                            if world.level.alertEvents[alertEvent as usize].level
                                == alertEventLevel_e::AEL_SUSPICIOUS
                            {
                                // suspicious looks longer
                                (*npc_info_ptr).investigateDebounceTime +=
                                    (*ctx.world_raw()).bg_state.rng.Q_irand(500, 2500);
                            }
                        }
                    }
                }

                if (*npc_info_ptr).investigateDebounceTime > world.level.time {
                    // FIXME: walk over to it, maybe?  Not if not chase enemies
                    // NOTE: stops walking or doing anything else below
                    let mut dir: vec3_t = [0.0; 3];
                    let mut angles: vec3_t = [0.0; 3];
                    let o_yaw = (*npc_info_ptr).desiredYaw;
                    let o_pitch = (*npc_info_ptr).desiredPitch;

                    crate::q_math::_VectorSubtract(
                        (*npc_info_ptr).investigateGoal,
                        (*((*npc_ptr).client as *mut gclient_t)).renderInfo.eyePoint,
                        &mut dir,
                    );
                    vectoangles(dir, &mut angles);

                    (*npc_info_ptr).desiredYaw = angles[YAW];
                    (*npc_info_ptr).desiredPitch = angles[PITCH];

                    NPC_UpdateAngles(ctx, qtrue, qtrue);

                    (*npc_info_ptr).desiredYaw = o_yaw;
                    (*npc_info_ptr).desiredPitch = o_pitch;
                    return;
                }
            }
        }

        // If we have somewhere to go, then do that
        if !UpdateGoal(ctx).is_null() {
            world.globals.ucmd.buttons |=
                mp_qshared::common::mp::qcommon::usercmd_button::BUTTON_WALKING as c_int;
            NPC_MoveToGoal(ctx, qtrue);
        }

        NPC_UpdateAngles(ctx, qtrue, qtrue);
    }
}

/// Raven `Grenadier_CheckMoveState`.
///
/// Source: `oracle/codemp/game/NPC_AI_Grenadier.c:307-391`
pub fn Grenadier_CheckMoveState(ctx: &mut GameContext) {
    unsafe {
        let world = &mut *ctx.world_raw();
        let npc_ptr = world.globals.NPC;
        let npc_info_ptr = world.globals.NPCInfo;

        if npc_info_ptr.is_null() || npc_ptr.is_null() {
            return;
        }

        // See if we're a scout
        if ((*npc_info_ptr).scriptFlags & SCF_CHASE_ENEMIES) == 0 {
            if (*npc_info_ptr).goalEntity == (*npc_ptr).enemy {
                world.globals.move3 = qfalse;
                return;
            }
        }
        // See if we're running away
        else if (*npc_info_ptr).squadState == SQUAD_RETREAT {
            if TIMER_Done(
                ctx,
                ctx.entity_id_of(npc_ptr),
                c"flee".as_ptr() as *const c_char,
            ) != qfalse
            {
                (*npc_info_ptr).squadState = SQUAD_IDLE;
            } else {
                world.globals.faceEnemy3 = qfalse;
            }
        }

        // See if we're moving towards a goal, not the enemy
        if (*npc_info_ptr).goalEntity != (*npc_ptr).enemy && !(*npc_info_ptr).goalEntity.is_none() {
            let __h84 = ctx.entity_id_of(npc_ptr);
            let __h85 = (*ctx.world_raw()).bg_state.rng.Q_irand(4000, 8000);
            // Did we make it?
            if NAV_HitNavGoal(
                (*npc_ptr).r.currentOrigin,
                (*npc_ptr).r.mins,
                (*npc_ptr).r.maxs,
                // guarded by `!goalEntity.is_none()` above.
                world.g_entities[(*npc_info_ptr).goalEntity.unwrap().index()]
                    .r
                    .currentOrigin,
                16,
                FlyingCreature(&*npc_ptr),
            ) != qfalse
                || ((*npc_info_ptr).squadState == SQUAD_SCOUT
                    && world.globals.enemyLOS3 != qfalse
                    && world.globals.enemyDist3 <= 10000.0)
            {
                // Oracle assigns the dead local `newSquadState` here (never written back
                // to NPCInfo->squadState), so squadState stays SQUAD_RETREAT and the later
                // `== SQUAD_RETREAT` flee/IDLE reset fires. Preserve that quirk (§20).
                let mut newSquadState = SQUAD_STAND_AND_SHOOT;
                // we got where we wanted to go, set timers based on why we were running
                match (*npc_info_ptr).squadState {
                    SQUAD_RETREAT => {
                        let __h74 = ctx.entity_id_of(npc_ptr);
                        // was running away
                        TIMER_Set(
                            ctx,
                            __h74,
                            c"duck".as_ptr() as *const c_char,
                            ((*npc_ptr).client as *mut gclient_t)
                                .as_ref()
                                .map(|c| ((c.pers.maxHealth - (*npc_ptr).health) * 100) as c_int)
                                .unwrap_or(0),
                        );
                        let __h75 = ctx.entity_id_of(npc_ptr);
                        let __h76 = (*ctx.world_raw()).bg_state.rng.Q_irand(3000, 7000);
                        TIMER_Set(ctx, __h75, c"hideTime".as_ptr() as *const c_char, __h76);
                        newSquadState = SQUAD_COVER;
                    }
                    SQUAD_TRANSITION => {
                        let __h77 = ctx.entity_id_of(npc_ptr);
                        let __h78 = (*ctx.world_raw()).bg_state.rng.Q_irand(2000, 4000);
                        // was heading for a combat point
                        TIMER_Set(ctx, __h77, c"hideTime".as_ptr() as *const c_char, __h78);
                    }
                    SQUAD_SCOUT => {
                        // was running after player
                    }
                    _ => {}
                }
                NPC_ReachedGoal(ctx);
                let __h79 = ctx.entity_id_of(npc_ptr);
                let __h80 = (*ctx.world_raw()).bg_state.rng.Q_irand(250, 500);
                // don't attack right away
                TIMER_Set(ctx, __h79, c"attackDelay".as_ptr() as *const c_char, __h80);
                let __h81 = ctx.entity_id_of(npc_ptr);
                let __h82 = (*ctx.world_raw()).bg_state.rng.Q_irand(1000, 4000);
                // don't do something else just yet
                TIMER_Set(ctx, __h81, c"roamTime".as_ptr() as *const c_char, __h82);
                // stop fleeing
                if (*npc_info_ptr).squadState == SQUAD_RETREAT {
                    let __h83 = ctx.entity_id_of(npc_ptr);
                    TIMER_Set(
                        ctx,
                        __h83,
                        c"flee".as_ptr() as *const c_char,
                        -world.level.time,
                    );
                    (*npc_info_ptr).squadState = SQUAD_IDLE;
                }
                return;
            }

            // keep going, hold of roamTimer until we get there
            let __h84 = ctx.entity_id_of(npc_ptr);
            let __h85 = (*ctx.world_raw()).bg_state.rng.Q_irand(4000, 8000);
            TIMER_Set(ctx, __h84, c"roamTime".as_ptr() as *const c_char, __h85);
        }

        if (*npc_info_ptr).goalEntity.is_none() {
            if ((*npc_info_ptr).scriptFlags & SCF_CHASE_ENEMIES) != 0 {
                (*npc_info_ptr).goalEntity = (*npc_ptr).enemy;
            }
        }
    }
}

/// Raven `Grenadier_CheckFireState`.
///
/// Source: `oracle/codemp/game/NPC_AI_Grenadier.c:399-439`
pub fn Grenadier_CheckFireState(ctx: &mut GameContext) {
    unsafe {
        let world = &*ctx.world_raw();
        let npc_ptr = world.globals.NPC;

        if world.globals.enemyCS3 != qfalse {
            // if have a clear shot, always try
            return;
        }

        let npc_info_ptr = world.globals.NPCInfo;
        if npc_info_ptr.is_null() || npc_ptr.is_null() {
            return;
        }

        if (*npc_info_ptr).squadState == SQUAD_RETREAT
            || (*npc_info_ptr).squadState == SQUAD_TRANSITION
            || (*npc_info_ptr).squadState == SQUAD_SCOUT
        {
            // runners never try to fire at the last pos
            return;
        }

        if crate::q_math::VectorCompare(
            (*((*npc_ptr).client as *mut gclient_t)).ps.velocity,
            crate::q_math::vec3_origin,
        ) == qfalse
        {
            // if moving at all, don't do this
            return;
        }
    }
}

/// Raven `Grenadier_EvaluateShot`.
///
/// Source: `oracle/codemp/game/NPC_AI_Grenadier.c:441-453`
pub fn Grenadier_EvaluateShot(ctx: &mut GameContext, hit: c_int) -> qboolean {
    unsafe {
        let world = &*ctx.world_raw();
        let npc_ptr = world.globals.NPC;

        if npc_ptr.is_null() || (*npc_ptr).enemy.is_none() {
            return qfalse;
        }

        if hit == world.g_entities[(*npc_ptr).enemy.unwrap().index()].s.number {
            // can hit enemy
            return qtrue;
        }

        if hit >= 0 && (hit as usize) < mp_qshared::shared::MAX_GENTITIES {
            let hit_ent = &world.g_entities[hit as usize];
            if (hit_ent.r.svFlags & SVF_GLASS_BRUSH as i32) != 0 {
                // will hit glass, so shoot anyway
                return qtrue;
            }
        }

        qfalse
    }
}

/// Raven `NPC_BSGrenadier_Attack`.
///
/// Source: `oracle/codemp/game/NPC_AI_Grenadier.c:461-662`
pub fn NPC_BSGrenadier_Attack(ctx: &mut GameContext) {
    unsafe {
        let world = &mut *ctx.world_raw();
        let npc_ptr = world.globals.NPC;
        let npc_info_ptr = world.globals.NPCInfo;

        let __h86 = ctx.entity_id_of(npc_ptr);
        let __h767 = NPC_CheckAlertEvents(
            ctx,
            qtrue,
            qtrue,
            -1,
            qfalse,
            alertEventLevel_e::AEL_DANGER as c_int,
        );
        if npc_info_ptr.is_null() || npc_ptr.is_null() {
            return;
        }

        // Don't do anything if we're hurt
        if (*npc_ptr).painDebounceTime > world.level.time {
            NPC_UpdateAngles(ctx, qtrue, qtrue);
            return;
        }

        // If we don't have an enemy, just idle
        if NPC_CheckEnemyExt(ctx, qfalse) == qfalse {
            (*npc_ptr).enemy = None;
            NPC_BSGrenadier_Patrol(ctx);
            return;
        }

        if TIMER_Done(ctx, __h86, c"flee".as_ptr() as *const c_char) != qfalse
            && NPC_CheckForDanger(ctx, __h767) != qfalse
        {
            // going to run
            NPC_UpdateAngles(ctx, qtrue, qtrue);
            return;
        }

        if (*npc_ptr).enemy.is_none() {
            // WTF?  somehow we lost our enemy?
            NPC_BSGrenadier_Patrol(ctx);
            return;
        }

        // Guaranteed `Some` from here to the end of the function by the guard above.
        let enemy_ent =
            &mut (*world).g_entities[(*npc_ptr).enemy.unwrap().index()] as *mut gentity_t;

        world.globals.enemyLOS3 = qfalse;
        world.globals.enemyCS3 = qfalse;
        world.globals.move3 = qtrue;
        world.globals.faceEnemy3 = qfalse;
        world.globals.shoot3 = qfalse;
        world.globals.enemyDist3 =
            DistanceSquared((*enemy_ent).r.currentOrigin, (*npc_ptr).r.currentOrigin);

        let __h87 = ctx.entity_id_of(enemy_ent);
        // See if we should switch to melee attack
        if world.globals.enemyDist3 < 16384.0
            && ((*enemy_ent).client.is_null()
                || (*((*enemy_ent).client as *mut gclient_t)).ps.weapon
                    != mp_bg::weapons::weapon_t::WP_SABER
                || BG_SabersOff(&mut (*((*enemy_ent).client as *mut gclient_t)).ps) != qfalse)
        {
            // enemy is close and not using saber
            if (*((*npc_ptr).client as *mut gclient_t)).ps.weapon == WP_THERMAL {
                // grenadier
                let mut trace: trace_t = core::mem::zeroed();
                trap::Trace(
                    ctx.engine,
                    mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                        &mut trace,
                        &(*npc_ptr).r.currentOrigin,
                        &(*enemy_ent).r.mins,
                        &(*enemy_ent).r.maxs,
                        &(*enemy_ent).r.currentOrigin,
                        (*npc_ptr).s.number,
                        (*enemy_ent).clipmask,
                    ),
                );
                if trace.allsolid == 0
                    && trace.startsolid == 0
                    && (trace.fraction == 1.0 || trace.entityNum as c_int == (*enemy_ent).s.number)
                {
                    // I can get right to him
                    // reset fire-timing variables
                    NPC_ChangeWeapon(WP_STUN_BATON);
                    if ((*npc_info_ptr).scriptFlags & SCF_CHASE_ENEMIES) == 0 {
                        (*npc_info_ptr).scriptFlags |= SCF_CHASE_ENEMIES;
                    }
                }
            }
        } else if world.globals.enemyDist3 > 65536.0
            || (!(*enemy_ent).client.is_null()
                && (*((*enemy_ent).client as *mut gclient_t)).ps.weapon
                    == mp_bg::weapons::weapon_t::WP_SABER
                && (*((*enemy_ent).client as *mut gclient_t)).ps.saberHolstered == 0)
        {
            // enemy is far or using saber
            if (*((*npc_ptr).client as *mut gclient_t)).ps.weapon == WP_STUN_BATON
                && (((*((*npc_ptr).client as *mut gclient_t)).ps.stats[STAT_WEAPONS as usize]
                    & (1 << WP_THERMAL))
                    != 0)
            {
                // fisticuffs, make switch to thermal if have it
                // reset fire-timing variables
                NPC_ChangeWeapon(WP_THERMAL);
            }
        }

        // can we see our target?
        if NPC_ClearLOS4(ctx, __h87) != qfalse {
            (*npc_info_ptr).enemyLastSeenTime = world.level.time;
            world.globals.enemyLOS3 = qtrue;

            if (*((*npc_ptr).client as *mut gclient_t)).ps.weapon == WP_STUN_BATON {
                if world.globals.enemyDist3 <= 4096.0
                    && InFOV3(
                        (*enemy_ent).r.currentOrigin,
                        (*npc_ptr).r.currentOrigin,
                        (*((*npc_ptr).client as *mut gclient_t)).ps.viewangles,
                        90,
                        45,
                    ) != qfalse
                {
                    // within 64 & infront
                    crate::q_math::_VectorCopy(
                        (*enemy_ent).r.currentOrigin,
                        &mut (*npc_info_ptr).enemyLastSeenLocation,
                    );
                    world.globals.enemyCS3 = qtrue;
                }
            } else if InFOV3(
                (*enemy_ent).r.currentOrigin,
                (*npc_ptr).r.currentOrigin,
                (*((*npc_ptr).client as *mut gclient_t)).ps.viewangles,
                45,
                90,
            ) != qfalse
            {
                let __h88 = ctx.entity_id_of(enemy_ent);
                // in front of me
                // can we shoot our target?
                let hit = NPC_ShotEntity(ctx, __h88, None);
                let hit_ent = &world.g_entities[hit as usize];
                if hit == (*enemy_ent).s.number
                    || (!hit_ent.client.is_null()
                        && (*(hit_ent.client as *mut gclient_t)).playerTeam
                            == (*((*npc_ptr).client as *mut gclient_t)).enemyTeam)
                {
                    let enemyHorzDist = DistanceHorizontalSquared(
                        (*enemy_ent).r.currentOrigin,
                        (*npc_ptr).r.currentOrigin,
                    );
                    crate::q_math::_VectorCopy(
                        (*enemy_ent).r.currentOrigin,
                        &mut (*npc_info_ptr).enemyLastSeenLocation,
                    );

                    if enemyHorzDist < 1048576.0 {
                        // within 1024
                        world.globals.enemyCS3 = qtrue;
                        NPC_AimAdjust(ctx, 2); // adjust aim better longer we have clear shot at enemy
                    } else {
                        NPC_AimAdjust(ctx, 1); // adjust aim better longer we can see enemy
                    }
                }
            }
        } else {
            NPC_AimAdjust(ctx, -1); // adjust aim worse longer we cannot see enemy
        }

        if world.globals.enemyLOS3 != qfalse {
            // FIXME: no need to face enemy if we're moving to some other goal and he's too far away to shoot?
            world.globals.faceEnemy3 = qtrue;
        }

        if world.globals.enemyCS3 != qfalse {
            world.globals.shoot3 = qtrue;
            if (*((*npc_ptr).client as *mut gclient_t)).ps.weapon == WP_THERMAL {
                // don't chase and throw
                world.globals.move3 = qfalse;
            } else if (*((*npc_ptr).client as *mut gclient_t)).ps.weapon == WP_STUN_BATON
                && world.globals.enemyDist3
                    < (((*npc_ptr).r.maxs[0] + (*enemy_ent).r.maxs[0] + 16.0)
                        * ((*npc_ptr).r.maxs[0] + (*enemy_ent).r.maxs[0] + 16.0))
            {
                // close enough
                world.globals.move3 = qfalse;
            }
        }

        // Check for movement to take care of
        Grenadier_CheckMoveState(ctx);

        // See if we should override shooting decision with any special considerations
        Grenadier_CheckFireState(ctx);

        if world.globals.move3 != qfalse {
            // move toward goal
            if !(*npc_info_ptr).goalEntity.is_none() {
                world.globals.move3 = Grenadier_Move(ctx);
            } else {
                world.globals.move3 = qfalse;
            }
        }

        if world.globals.move3 == qfalse {
            if TIMER_Done(
                ctx,
                ctx.entity_id_of(npc_ptr),
                c"duck".as_ptr() as *const c_char,
            ) == qfalse
            {
                world.globals.ucmd.upmove = -127;
            }
        } else {
            let __h89 = ctx.entity_id_of(npc_ptr);
            // stop ducking!
            TIMER_Set(ctx, __h89, c"duck".as_ptr() as *const c_char, -1);
        }

        if world.globals.faceEnemy3 == qfalse {
            // we want to face in the dir we're running
            if world.globals.move3 != qfalse {
                // don't run away and shoot
                (*npc_info_ptr).desiredYaw = (*npc_info_ptr).lastPathAngles[YAW];
                (*npc_info_ptr).desiredPitch = 0.0;
                world.globals.shoot3 = qfalse;
            }
            NPC_UpdateAngles(ctx, qtrue, qtrue);
        } else {
            // face the enemy
            NPC_FaceEnemy(ctx, qtrue);
        }

        if ((*npc_info_ptr).scriptFlags & SCF_DONT_FIRE) != 0 {
            world.globals.shoot3 = qfalse;
        }

        // FIXME: don't shoot right away!
        if world.globals.shoot3 != qfalse {
            // try to shoot if it's time
            if TIMER_Done(
                ctx,
                ctx.entity_id_of(npc_ptr),
                c"attackDelay".as_ptr() as *const c_char,
            ) != qfalse
            {
                if ((*npc_info_ptr).scriptFlags & SCF_FIRE_WEAPON) == 0 {
                    // we've already fired, no need to do it again here
                    WeaponThink(ctx, qtrue);
                    let __h90 = ctx.entity_id_of(npc_ptr);
                    TIMER_Set(
                        ctx,
                        __h90,
                        c"attackDelay".as_ptr() as *const c_char,
                        (*npc_info_ptr).shotTime - world.level.time,
                    );
                }
            }
        }
    }
}

/// Raven `NPC_BSGrenadier_Default`.
///
/// Source: `oracle/codemp/game/NPC_AI_Grenadier.c:664-679`
pub fn NPC_BSGrenadier_Default(ctx: &mut GameContext) {
    unsafe {
        let world = &*ctx.world_raw();
        let npc_ptr = world.globals.NPC;
        let npc_info_ptr = world.globals.NPCInfo;

        if npc_info_ptr.is_null() || npc_ptr.is_null() {
            return;
        }

        if ((*npc_info_ptr).scriptFlags & SCF_FIRE_WEAPON) != 0 {
            WeaponThink(ctx, qtrue);
        }

        if (*npc_ptr).enemy.is_none() {
            // don't have an enemy, look for one
            NPC_BSGrenadier_Patrol(ctx);
        } else {
            // have an enemy
            NPC_BSGrenadier_Attack(ctx);
        }
    }
}
