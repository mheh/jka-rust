// PORT-COMPLETE: NPC_AI_Stormtrooper.c 26/26
//! FAITHFUL port of `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c`.
//!
//! Landed from the `fnskel.py` signature skeleton; the pass-3 mega-pass fills
//! every remaining body against the settled fork rulings (ctx threading,
//! `Option<EntityId>` stored fields, bg/game state split). File-scope AI
//! globals (`NPC`, `NPCInfo`, `ucmd`, `level`, `g_entities`) and this file's
//! own file-statics (`enemyLOS`/`enemyCS`/`enemyInFOV`/`faceEnemy`/`hitAlly`/
//! `move`/`shoot`/`enemyDist` — genuine cross-frame state)
//! reach through `ctx.world`/`ctx.world.globals`.
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_nav::{
    FlyingCreature, NAV_FindClosestWaypointForEnt, NAV_HitNavGoal, NPC_SetMoveGoal,
};
use crate::g_timer::{TIMER_Done, TIMER_Get, TIMER_Set};
use crate::g_utils::{G_ExpandPointToBBox, GetAnglesForDirection};
use crate::level::alert_event::alertEventType_e;
use crate::npc::spot_t::spot_t::SPOT_HEAD;
use crate::npc_c::{NPC_SetAnim, RestoreNPCGlobals, SaveNPCGlobals, SetNPCGlobals};
use crate::prelude::*;
use crate::q_math::{
    _VectorCopy, vec3_origin, vectoangles, AngleVectors, Distance, VectorCompare, VectorLength,
    VectorNormalize,
};
use crate::teams::NPCTEAM_PLAYER;
use crate::trap;
use crate::NPC_AI_Default::NPC_BSPatrol;
use crate::NPC_AI_Utils::{
    AI_GetGroup, AI_GroupContainsEntNum, AI_GroupUpdateClearShotTime, AI_GroupUpdateEnemyLastSeen,
    AI_GroupUpdateSquadstates,
};
use crate::NPC_behavior::{G_StartFlee, NPC_BSSearchStart, NPC_StartFlee};
use crate::NPC_combat::{
    ChangeWeapon, G_ClearEnemy, G_SetEnemy, NPC_AimAdjust, NPC_ChangeWeapon, NPC_CheckGetNewWeapon,
    NPC_FindCombatPoint, NPC_FreeCombatPoint, NPC_SetCombatPoint, NPC_ShotEntity, WeaponThink,
};
use crate::NPC_goal::{NPC_ReachedGoal, UpdateGoal};
use crate::NPC_move::NAV_GetLastMove;
use crate::NPC_move::NPC_MoveToGoal;
use crate::NPC_reactions::{NPC_Pain, NPC_TempLookTarget};
use crate::NPC_senses::InFOV;
use crate::NPC_senses::{
    NPC_CheckAlertEvents, NPC_CheckForDanger, NPC_GetHFOVPercentage, NPC_GetVFOVPercentage,
};
use crate::NPC_sounds::G_AddVoiceEvent;
use crate::NPC_utils::{
    CalcEntitySpot, G_ActivateBehavior, NPC_CheckEnemyExt, NPC_ClearLOS4, NPC_FaceEnemy,
    NPC_FacePosition, NPC_UpdateAngles, NPC_ValidEnemy,
};
use mp_abi::game::syscalls::G_ICARUS_TASKIDPENDING::GIcarusTaskidpendingArgs;
use mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs;
use mp_abi::game::syscalls::G_NAV_GETPATHCOST::GNavGetpathcostArgs;
use mp_abi::game::syscalls::G_POINT_CONTENTS::GPointContentsArgs;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use mp_bg::public::entity_event::entity_event_t::{
    EV_ANGER1, EV_ANGER3, EV_CHASE1, EV_CHASE3, EV_CONFUSE1, EV_CONFUSE3, EV_COVER1, EV_COVER5,
    EV_DETECTED1, EV_DETECTED5, EV_ESCAPING1, EV_ESCAPING3, EV_GIVEUP1, EV_GIVEUP4, EV_LOOK1,
    EV_LOOK2, EV_LOST1, EV_OUTFLANK1, EV_OUTFLANK2, EV_PUSHED1, EV_PUSHED3, EV_SIGHT1, EV_SIGHT3,
    EV_SOUND1, EV_SOUND3, EV_SUSPICIOUS1, EV_SUSPICIOUS5,
};
use mp_bg::public::weaponstate::weaponstate_t::WEAPON_READY;

// Combat point search flags: `crate::npc::combat_point_flags`
// (`b_local.h:243-264`).

// File-scope constants (`#define`).
// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:19-34`
pub const MAX_VIEW_DIST: f32 = 1024.0;
pub const MAX_VIEW_SPEED: f32 = 250.0;
pub const MAX_LIGHT_INTENSITY: f32 = 255.0;
pub const MIN_LIGHT_THRESHOLD: f32 = 0.1;
pub const ST_MIN_LIGHT_THRESHOLD: c_int = 30;
pub const ST_MAX_LIGHT_THRESHOLD: c_int = 180;
pub const DISTANCE_THRESHOLD: f32 = 0.075;
pub const DISTANCE_SCALE: f32 = 0.35;
pub const FOV_SCALE: f32 = 0.40;
pub const LIGHT_SCALE: f32 = 0.25;
pub const SPEED_SCALE: f32 = 0.25;
pub const TURNING_SCALE: f32 = 0.25;
pub const REALIZE_THRESHOLD: f32 = 0.6;
pub const CAUTIOUS_THRESHOLD: f32 = REALIZE_THRESHOLD * 0.75;
// `MIN_ROCKET_DIST_SQUARED` (`b_local.h`) — 128*128.
pub const MIN_ROCKET_DIST_SQUARED: f32 = 16384.0;

// EntityId seam helpers (local to this file, mirroring the `g_missile.rs`
// precedent): `gentity_t*` stored fields (`enemy`/`goalEntity`/`tempGoal`/…)
// are `Option<EntityId>`; these resolve an id back to the live pointer at
// call sites that still take raw `*mut gentity_t`, and build the id back at
// assignment sites.
#[inline]
unsafe fn ent_base(ctx: GameContext<'_>) -> *const gentity_t {
    unsafe { (*ctx.world).g_entities.as_ptr() }
}
#[inline]
unsafe fn ent_resolve(ctx: GameContext<'_>, id: EntityId) -> *mut gentity_t {
    unsafe { &mut (*ctx.world).g_entities[id.index()] as *mut gentity_t }
}
#[inline]
unsafe fn ent_resolve_opt(ctx: GameContext<'_>, id: Option<EntityId>) -> *mut gentity_t {
    match id {
        Some(i) => unsafe { ent_resolve(ctx, i) },
        None => core::ptr::null_mut(),
    }
}

// Raven's anonymous `enum { LSTATE_NONE, LSTATE_UNDERFIRE, LSTATE_INVESTIGATE }`
// (file-scope local state, `gNPC_t::localState`) — not a central type, ported
// as file-local consts matching the C values.
// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:53-58`
const LSTATE_NONE: i32 = 0;
const LSTATE_UNDERFIRE: i32 = 1;
const LSTATE_INVESTIGATE: i32 = 2;

// Raven's anonymous `enum { SPEECH_CHASE, ... SPEECH_PUSHED }` (file-scope
// speech-type selector for `ST_Speech`) — not a central type, ported as
// file-local consts matching the C values.
// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:106-122`
pub const SPEECH_CHASE: i32 = 0;
pub const SPEECH_CONFUSED: i32 = 1;
pub const SPEECH_COVER: i32 = 2;
pub const SPEECH_DETECTED: i32 = 3;
pub const SPEECH_GIVEUP: i32 = 4;
pub const SPEECH_LOOK: i32 = 5;
pub const SPEECH_LOST: i32 = 6;
pub const SPEECH_OUTFLANK: i32 = 7;
pub const SPEECH_ESCAPING: i32 = 8;
pub const SPEECH_SIGHT: i32 = 9;
pub const SPEECH_SOUND: i32 = 10;
pub const SPEECH_SUSPICIOUS: i32 = 11;
pub const SPEECH_YELL: i32 = 12;
pub const SPEECH_PUSHED: i32 = 13;

/// Raven `ST_AggressionAdjust`.
///
/// Raven: good guys (`NPCTEAM_PLAYER`) are less aggressive (clamp 1-7); bad
/// guys are more aggressive (clamp 3-10). //FIXME: base this on initial NPC
/// stats (Raven comment).
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:60-86`
pub fn ST_AggressionAdjust(self_: *mut gentity_t, change: c_int) {
    unsafe {
        let npc = (*self_).NPC as *mut gNPC_t;
        (*npc).stats.aggression += change;

        let client = (*self_).client as *mut gclient_t;
        let (upper_threshold, lower_threshold) = if (*client).playerTeam == NPCTEAM_PLAYER {
            (7, 1)
        } else {
            (10, 3)
        };

        if (*npc).stats.aggression > upper_threshold {
            (*npc).stats.aggression = upper_threshold;
        } else if (*npc).stats.aggression < lower_threshold {
            (*npc).stats.aggression = lower_threshold;
        }
    }
}

/// Raven `ST_ClearTimers`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:88-104`
pub fn ST_ClearTimers(ctx: GameContext<'_>, ent: *mut gentity_t) {
    TIMER_Set(ctx, ent, c"chatter".as_ptr() as *const c_char, 0);
    TIMER_Set(ctx, ent, c"duck".as_ptr() as *const c_char, 0);
    TIMER_Set(ctx, ent, c"stand".as_ptr() as *const c_char, 0);
    TIMER_Set(ctx, ent, c"shuffleTime".as_ptr() as *const c_char, 0);
    TIMER_Set(ctx, ent, c"sleepTime".as_ptr() as *const c_char, 0);
    TIMER_Set(ctx, ent, c"enemyLastVisible".as_ptr() as *const c_char, 0);
    TIMER_Set(ctx, ent, c"roamTime".as_ptr() as *const c_char, 0);
    TIMER_Set(ctx, ent, c"hideTime".as_ptr() as *const c_char, 0);
    // FIXME: Slant for difficulty levels (Raven comment).
    TIMER_Set(ctx, ent, c"attackDelay".as_ptr() as *const c_char, 0);
    TIMER_Set(ctx, ent, c"stick".as_ptr() as *const c_char, 0);
    TIMER_Set(ctx, ent, c"scoutTime".as_ptr() as *const c_char, 0);
    TIMER_Set(ctx, ent, c"flee".as_ptr() as *const c_char, 0);
    TIMER_Set(ctx, ent, c"interrogating".as_ptr() as *const c_char, 0);
    TIMER_Set(ctx, ent, c"verifyCP".as_ptr() as *const c_char, 0);
}

/// Raven `ST_Speech`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:124-225`
pub fn ST_Speech(ctx: GameContext<'_>, self_: *mut gentity_t, speechType: c_int, failChance: f32) {
    unsafe {
        let world = &mut *ctx.world;
        // Raven's `random()` macro: `(rand() & 0x7fff) / (float)0x7fff`.
        // Source: `oracle/oracle/codemp/game/q_shared.h:1591`
        let random_val = world.bg_state.rng.random();
        if random_val < failChance {
            return;
        }

        let npc = (*self_).NPC as *mut gNPC_t;
        let client = (*self_).client as *mut gclient_t;

        if failChance >= 0.0 {
            // a negative failChance makes it always talk
            if !(*npc).group.is_null() {
                // group AI speech debounce timer
                if (*(*npc).group).speechDebounceTime > world.level.time {
                    return;
                }
                /*
                else if ( !self->NPC->group->enemy )
                {
                    if ( groupSpeechDebounceTime[self->client->playerTeam] > level.time )
                    {
                        return;
                    }
                }
                */
            } else if TIMER_Done(ctx, self_, c"chatter".as_ptr()) == 0 {
                // personal timer
                return;
            } else if world.globals.groupSpeechDebounceTime[(*client).playerTeam as usize]
                > world.level.time
            {
                // for those not in group AI
                // FIXME: let certain speech types interrupt others? Let closer NPCs
                // interrupt farther away ones? (Raven comment).
                return;
            }
        }

        if !(*npc).group.is_null() {
            // So they don't all speak at once...
            // FIXME: if they're not yet mad, they have no group, so distracting a
            // group of them makes them all speak! (Raven comment).
            (*(*npc).group).speechDebounceTime =
                world.level.time + (*ctx.world).bg_state.rng.Q_irand(2000, 4000);
        } else {
            TIMER_Set(
                ctx,
                self_,
                c"chatter".as_ptr(),
                (*ctx.world).bg_state.rng.Q_irand(2000, 4000),
            );
        }
        world.globals.groupSpeechDebounceTime[(*client).playerTeam as usize] =
            world.level.time + (*ctx.world).bg_state.rng.Q_irand(2000, 4000);

        if (*npc).blockedSpeechDebounceTime > world.level.time {
            return;
        }

        match speechType {
            SPEECH_CHASE => G_AddVoiceEvent(
                ctx,
                self_,
                (*ctx.world)
                    .bg_state
                    .rng
                    .Q_irand(EV_CHASE1 as c_int, EV_CHASE3 as c_int),
                2000,
            ),
            SPEECH_CONFUSED => G_AddVoiceEvent(
                ctx,
                self_,
                (*ctx.world)
                    .bg_state
                    .rng
                    .Q_irand(EV_CONFUSE1 as c_int, EV_CONFUSE3 as c_int),
                2000,
            ),
            SPEECH_COVER => G_AddVoiceEvent(
                ctx,
                self_,
                (*ctx.world)
                    .bg_state
                    .rng
                    .Q_irand(EV_COVER1 as c_int, EV_COVER5 as c_int),
                2000,
            ),
            SPEECH_DETECTED => G_AddVoiceEvent(
                ctx,
                self_,
                (*ctx.world)
                    .bg_state
                    .rng
                    .Q_irand(EV_DETECTED1 as c_int, EV_DETECTED5 as c_int),
                2000,
            ),
            SPEECH_GIVEUP => G_AddVoiceEvent(
                ctx,
                self_,
                (*ctx.world)
                    .bg_state
                    .rng
                    .Q_irand(EV_GIVEUP1 as c_int, EV_GIVEUP4 as c_int),
                2000,
            ),
            SPEECH_LOOK => G_AddVoiceEvent(
                ctx,
                self_,
                (*ctx.world)
                    .bg_state
                    .rng
                    .Q_irand(EV_LOOK1 as c_int, EV_LOOK2 as c_int),
                2000,
            ),
            SPEECH_LOST => G_AddVoiceEvent(ctx, self_, EV_LOST1 as c_int, 2000),
            SPEECH_OUTFLANK => G_AddVoiceEvent(
                ctx,
                self_,
                (*ctx.world)
                    .bg_state
                    .rng
                    .Q_irand(EV_OUTFLANK1 as c_int, EV_OUTFLANK2 as c_int),
                2000,
            ),
            SPEECH_ESCAPING => G_AddVoiceEvent(
                ctx,
                self_,
                (*ctx.world)
                    .bg_state
                    .rng
                    .Q_irand(EV_ESCAPING1 as c_int, EV_ESCAPING3 as c_int),
                2000,
            ),
            SPEECH_SIGHT => G_AddVoiceEvent(
                ctx,
                self_,
                (*ctx.world)
                    .bg_state
                    .rng
                    .Q_irand(EV_SIGHT1 as c_int, EV_SIGHT3 as c_int),
                2000,
            ),
            SPEECH_SOUND => G_AddVoiceEvent(
                ctx,
                self_,
                (*ctx.world)
                    .bg_state
                    .rng
                    .Q_irand(EV_SOUND1 as c_int, EV_SOUND3 as c_int),
                2000,
            ),
            SPEECH_SUSPICIOUS => G_AddVoiceEvent(
                ctx,
                self_,
                (*ctx.world)
                    .bg_state
                    .rng
                    .Q_irand(EV_SUSPICIOUS1 as c_int, EV_SUSPICIOUS5 as c_int),
                2000,
            ),
            SPEECH_YELL => G_AddVoiceEvent(
                ctx,
                self_,
                (*ctx.world)
                    .bg_state
                    .rng
                    .Q_irand(EV_ANGER1 as c_int, EV_ANGER3 as c_int),
                2000,
            ),
            SPEECH_PUSHED => G_AddVoiceEvent(
                ctx,
                self_,
                (*ctx.world)
                    .bg_state
                    .rng
                    .Q_irand(EV_PUSHED1 as c_int, EV_PUSHED3 as c_int),
                2000,
            ),
            _ => {}
        }

        (*npc).blockedSpeechDebounceTime = world.level.time + 2000;
    }
}

/// Raven `ST_MarkToCover`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:227-240`
pub fn ST_MarkToCover(ctx: GameContext<'_>, self_: *mut gentity_t) {
    unsafe {
        if self_.is_null() || (*self_).NPC.is_null() {
            return;
        }
        let npc = (*self_).NPC as *mut gNPC_t;
        (*npc).localState = LSTATE_UNDERFIRE;
        TIMER_Set(
            ctx,
            self_,
            c"attackDelay".as_ptr() as *const c_char,
            (*ctx.world).bg_state.rng.Q_irand(500, 2500),
        );
        ST_AggressionAdjust(self_, -3);
        if !(*npc).group.is_null() && (*(*npc).group).numGroup > 1 {
            // FIXME: flee sound? (Raven comment).
            ST_Speech(ctx, self_, SPEECH_COVER, 0.0);
        }
    }
}

/// Raven `ST_StartFlee`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:242-253`
pub fn ST_StartFlee(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    enemy: *mut gentity_t,
    dangerPoint: vec3_t,
    dangerLevel: c_int,
    minTime: c_int,
    maxTime: c_int,
) {
    unsafe {
        if self_.is_null() || (*self_).NPC.is_null() {
            return;
        }
        G_StartFlee(
            ctx,
            self_,
            enemy,
            dangerPoint,
            dangerLevel,
            minTime,
            maxTime,
        );
        let npc = (*self_).NPC as *mut gNPC_t;
        if !(*npc).group.is_null() && (*(*npc).group).numGroup > 1 {
            // FIXME: flee sound? (Raven comment).
            ST_Speech(ctx, self_, SPEECH_COVER, 0.0);
        }
    }
}

/// Raven `NPC_ST_Pain`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:260-274`
pub fn NPC_ST_Pain(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    unsafe {
        let npc = (*self_).NPC as *mut gNPC_t;
        (*npc).localState = LSTATE_UNDERFIRE;

        TIMER_Set(ctx, self_, c"duck".as_ptr() as *const c_char, -1);
        TIMER_Set(ctx, self_, c"hideTime".as_ptr() as *const c_char, -1);
        TIMER_Set(ctx, self_, c"stand".as_ptr() as *const c_char, 2000);

        NPC_Pain(ctx, self_, attacker, damage);

        if damage == 0 && (*self_).health > 0 {
            // FIXME: better way to know I was pushed (Raven comment).
            G_AddVoiceEvent(
                ctx,
                self_,
                (*ctx.world)
                    .bg_state
                    .rng
                    .Q_irand(EV_PUSHED1 as c_int, EV_PUSHED3 as c_int),
                2000,
            );
        }
    }
}

/// Raven `ST_HoldPosition`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:282-302`
pub fn ST_HoldPosition(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        if (*NPCInfo).squadState == SQUAD_RETREAT {
            TIMER_Set(ctx, NPC, c"flee".as_ptr(), -world.level.time);
        }
        // don't look for another one for a few seconds
        TIMER_Set(
            ctx,
            NPC,
            c"verifyCP".as_ptr(),
            (*ctx.world).bg_state.rng.Q_irand(1000, 3000),
        );
        NPC_FreeCombatPoint(ctx, (*NPCInfo).combatPoint, qtrue);
        // NPCInfo->combatPoint = -1;//??? (Raven comment).
        if trap::ICARUS_TaskIDPending(
            ctx.engine,
            GIcarusTaskidpendingArgs::new(NPC, TID_MOVE_NAV as c_int),
        ) == 0
        {
            // don't have a script waiting for me to get to my point, okay to stop
            // trying and stand
            AI_GroupUpdateSquadstates((*NPCInfo).group, NPC, SQUAD_STAND_AND_SHOOT);
            (*NPCInfo).goalEntity = None;
        }

        /*if ( TIMER_Done( NPC, "stand" ) )
        {//FIXME: what if can't shoot from this pos?
            TIMER_Set( NPC, "duck", (*ctx.world).bg_state.rng.Q_irand( 2000, 4000 ) );
        }
        */
    }
}

/// Raven `NPC_ST_SayMovementSpeech`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:304-325`
pub fn NPC_ST_SayMovementSpeech(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        if (*NPCInfo).movementSpeech == 0 {
            return;
        }
        let group = (*NPCInfo).group;
        if !group.is_null()
            && !(*group).commander.is_null()
            && !(*(*group).commander).client.is_null()
            && (*((*(*group).commander).client as *mut gclient_t)).NPC_class == CLASS_IMPERIAL
            && (*ctx.world).bg_state.rng.Q_irand(0, 3) == 0
        {
            // imperial (commander) gives the order
            ST_Speech(
                ctx,
                (*group).commander,
                (*NPCInfo).movementSpeech,
                (*NPCInfo).movementSpeechChance,
            );
        } else {
            // really don't want to say this unless we can actually get there...
            ST_Speech(
                ctx,
                NPC,
                (*NPCInfo).movementSpeech,
                (*NPCInfo).movementSpeechChance,
            );
        }

        (*NPCInfo).movementSpeech = 0;
        (*NPCInfo).movementSpeechChance = 0.0;
    }
}

/// Raven `NPC_ST_StoreMovementSpeech`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:327-331`
pub fn NPC_ST_StoreMovementSpeech(ctx: GameContext<'_>, speech: c_int, chance: f32) {
    unsafe {
        let world = &mut *ctx.world;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;
        (*NPCInfo).movementSpeech = speech;
        (*NPCInfo).movementSpeechChance = chance;
    }
}

/// Raven `ST_Move`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:338-390`
pub fn ST_Move(ctx: GameContext<'_>) -> qboolean {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        // always move straight toward our goal
        (*NPCInfo).combatMove = qtrue;

        let moved = NPC_MoveToGoal(ctx, qtrue);

        // Get the move info
        let mut info: navInfo_t = core::mem::zeroed();
        NAV_GetLastMove(ctx, &mut info);

        // FIXME: if we bump into another one of our guys and can't get around him,
        // just stop! (Raven comment).
        // If we hit our target, then stop and fire!
        if (info.flags & 0x00000004) != 0 {
            // NIF_COLLISION = 0x00000004 (b_local.h:305); was wrongly 0x1 (NIF_FAILED).
            if ent_id_opt(ent_base(ctx), info.blocker) == (*NPC).enemy {
                ST_HoldPosition(ctx);
            }
        }

        // If our move failed, then reset
        if moved == qfalse {
            // FIXME: if we're going to a combat point, need to pick a different one
            // (Raven comment).
            if trap::ICARUS_TaskIDPending(
                ctx.engine,
                GIcarusTaskidpendingArgs::new(NPC, TID_MOVE_NAV as c_int),
            ) == 0
            {
                // can't transfer movegoal or stop when a script we're running is
                // waiting to complete
                if !info.blocker.is_null()
                    && !(*info.blocker).NPC.is_null()
                    && !(*NPCInfo).group.is_null()
                    && (*((*info.blocker).NPC as *mut gNPC_t)).group == (*NPCInfo).group
                {
                    // dammit, something is in our way, see if it's one of ours
                    let group = (*NPCInfo).group;
                    for j in 0..(*group).numGroup {
                        if (*group).member[j as usize].number == (*NPCInfo).blockingEntNum {
                            // we're being blocked by one of our own, pass our goal
                            // onto them and I'll stand still
                            let member = &mut world.g_entities
                                [(*group).member[j as usize].number as usize]
                                as *mut gentity_t;
                            ST_TransferMoveGoal(ctx, NPC, member);
                            break;
                        }
                    }
                }

                ST_HoldPosition(ctx);
            }
        } else {
            // First time you successfully move, say what it is you're doing
            NPC_ST_SayMovementSpeech(ctx);
        }

        moved
    }
}

/// Raven `NPC_ST_SleepShuffle`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:399-439`
pub fn NPC_ST_SleepShuffle(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;

        // Play an awake script if we have one
        if G_ActivateBehavior(ctx, NPC, bSet_t::BSET_AWAKE as c_int) != 0 {
            return;
        }

        // Automate some movement and noise
        if TIMER_Done(ctx, NPC, c"shuffleTime".as_ptr()) != 0 {
            // TODO: Play sleeping shuffle animation (Raven comment).
            //int soundIndex = (*ctx.world).bg_state.rng.Q_irand( 0, 1 );
            /*
            switch ( soundIndex )
            {
            case 0:
                G_Sound( NPC, G_SoundIndex("sound/chars/imperialsleeper1/scav4/hunh.mp3") );
                break;
            case 1:
                G_Sound( NPC, G_SoundIndex("sound/chars/imperialsleeper3/scav4/tryingtosleep.wav") );
                break;
            }
            */
            TIMER_Set(ctx, NPC, c"shuffleTime".as_ptr(), 4000);
            TIMER_Set(ctx, NPC, c"sleepTime".as_ptr(), 2000);
            return;
        }

        // They made another noise while we were stirring, see if we can see them
        if TIMER_Done(ctx, NPC, c"sleepTime".as_ptr()) != 0 {
            NPC_CheckPlayerTeamStealth(ctx);
            TIMER_Set(ctx, NPC, c"sleepTime".as_ptr(), 2000);
        }
    }
}

/// Raven `NPC_BSST_Sleep`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:447-468`
pub fn NPC_BSST_Sleep(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        // only check sounds since we're asleep!
        let alertEvent = NPC_CheckAlertEvents(ctx, qfalse, qtrue, -1, qfalse, AEL_MINOR as c_int);

        // There is an event we heard
        if alertEvent >= 0 {
            // See if it was enough to wake us up
            if world.level.alertEvents[alertEvent as usize].level == AEL_DISCOVERED
                && ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0
            {
                // rwwFIXMEFIXME: Care about all clients not just 0 (Raven comment).
                if world.g_entities[0].health > 0 {
                    let target = &mut world.g_entities[0] as *mut gentity_t;
                    G_SetEnemy(ctx, NPC, target);
                    return;
                }
            }

            // Otherwise just stir a bit
            NPC_ST_SleepShuffle(ctx);
            return;
        }
    }
}

/// Raven `NPC_CheckEnemyStealth`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:476-725`
pub fn NPC_CheckEnemyStealth(ctx: GameContext<'_>, target: *mut gentity_t) -> qboolean {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        // any closer than 40 and we definitely notice
        let mut minDist: f32 = 40.0;

        // In case we aquired one some other way
        if (*NPC).enemy != None {
            return qtrue;
        }

        // Ignore notarget
        if ((*target).flags & FL_NOTARGET) != 0 {
            return qfalse;
        }

        if (*target).health <= 0 {
            return qfalse;
        }

        let tclient = (*target).client as *mut gclient_t;
        if (*tclient).ps.weapon == WP_SABER
            && (*tclient).ps.saberHolstered == 0
            && (*tclient).ps.saberInFlight == qfalse
        {
            // if target has saber in hand and activated, we wake up even sooner
            // even if not facing him
            minDist = 100.0;
        }

        let mut target_dist = DistanceSquared((*target).r.currentOrigin, (*NPC).r.currentOrigin);

        // If the target is this close, then wake up regardless
        if ((*tclient).ps.pm_flags & PMF_DUCKED) == 0
            && ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0
            && target_dist < (minDist * minDist)
        {
            G_SetEnemy(ctx, NPC, target);
            (*NPCInfo).enemyLastSeenTime = world.level.time;
            TIMER_Set(
                ctx,
                NPC,
                c"attackDelay".as_ptr(),
                (*ctx.world).bg_state.rng.Q_irand(500, 2500),
            );
            return qtrue;
        }

        let mut maxViewDist = MAX_VIEW_DIST;

        if (*NPCInfo).stats.visrange > maxViewDist {
            // FIXME: should we always just set maxViewDist to this? (Raven comment).
            maxViewDist = (*NPCInfo).stats.visrange;
        }

        if target_dist > (maxViewDist * maxViewDist) {
            // out of possible visRange
            return qfalse;
        }

        // Check FOV first
        if InFOV(
            ctx,
            target,
            NPC,
            (*NPCInfo).stats.hfov,
            (*NPCInfo).stats.vfov,
        ) == qfalse
        {
            return qfalse;
        }

        // clearLOS = ( target->client->ps.leanofs ) ? NPC_ClearLOS5( ... ) : NPC_ClearLOS4( target );
        let clearLOS = NPC_ClearLOS4(ctx, target);

        // Now check for clear line of vision
        if clearLOS != qfalse {
            if (*tclient).NPC_class == CLASS_ATST {
                // can't miss 'em!
                G_SetEnemy(ctx, NPC, target);
                TIMER_Set(
                    ctx,
                    NPC,
                    c"attackDelay".as_ptr(),
                    (*ctx.world).bg_state.rng.Q_irand(500, 2500),
                );
                return qtrue;
            }
            let targ_org: vec3_t = [
                (*target).r.currentOrigin[0],
                (*target).r.currentOrigin[1],
                (*target).r.currentOrigin[2] + (*target).r.maxs[2] - 4.0,
            ];
            let mut hAngle_perc = NPC_GetHFOVPercentage(
                targ_org,
                (*((*NPC).client as *mut gclient_t)).renderInfo.eyePoint,
                (*((*NPC).client as *mut gclient_t)).renderInfo.eyeAngles,
                (*NPCInfo).stats.hfov as f32,
            );
            let mut vAngle_perc = NPC_GetVFOVPercentage(
                targ_org,
                (*((*NPC).client as *mut gclient_t)).renderInfo.eyePoint,
                (*((*NPC).client as *mut gclient_t)).renderInfo.eyeAngles,
                (*NPCInfo).stats.vfov as f32,
            );

            // Scale them vertically some, and horizontally pretty harshly
            vAngle_perc *= vAngle_perc;
            hAngle_perc *= hAngle_perc * hAngle_perc;

            // Assess the player's current status
            target_dist = Distance((*target).r.currentOrigin, (*NPC).r.currentOrigin);

            let target_speed = VectorLength((*tclient).ps.velocity);
            let target_crouching = (*tclient).pers.cmd.upmove < 0;
            let dist_rating = target_dist / maxViewDist;
            let mut speed_rating = target_speed / MAX_VIEW_SPEED;
            // AngleDelta(...)/180.0 + AngleDelta(...)/180.0 (Raven, commented out).
            let turning_rating: f32 = 5.0;
            let light_level: f32 = 255.0 / MAX_LIGHT_INTENSITY;
            let FOV_perc = 1.0 - (hAngle_perc + vAngle_perc) * 0.5; // FIXME: Dunno about the average... (Raven comment).
            let mut vis_rating: f32 = 0.0;

            // Too dark
            if light_level < MIN_LIGHT_THRESHOLD {
                return qfalse;
            }

            // Too close?
            if dist_rating < DISTANCE_THRESHOLD {
                G_SetEnemy(ctx, NPC, target);
                TIMER_Set(
                    ctx,
                    NPC,
                    c"attackDelay".as_ptr(),
                    (*ctx.world).bg_state.rng.Q_irand(500, 2500),
                );
                return qtrue;
            }

            // Out of range
            if dist_rating > 1.0 {
                return qfalse;
            }

            // Cap our speed checks
            if speed_rating > 1.0 {
                speed_rating = 1.0;
            }

            // Calculate the distance, fov and light influences
            // ...Visibilty linearly wanes over distance
            let dist_influence = DISTANCE_SCALE * (1.0 - dist_rating);
            // ...As the percentage out of the FOV increases, straight perception
            // suffers on an exponential scale
            let fov_influence = FOV_SCALE * (1.0 - FOV_perc);
            // ...Lack of light hides, abundance of light exposes
            let light_influence = (light_level - 0.5) * LIGHT_SCALE;

            // Calculate our base rating
            let mut target_rating = dist_influence + fov_influence + light_influence;

            // Now award any final bonuses to this number
            let contents = trap::PointContents(
                ctx.engine,
                GPointContentsArgs::new(&targ_org as *const vec3_t, (*target).s.number),
            );
            if (contents & CONTENTS_WATER) != 0 {
                let myContents = trap::PointContents(
                    ctx.engine,
                    GPointContentsArgs::new(
                        &(*((*NPC).client as *mut gclient_t)).renderInfo.eyePoint as *const vec3_t,
                        (*NPC).s.number,
                    ),
                );
                if (myContents & CONTENTS_WATER) == 0 {
                    // I'm not in water
                    if (*((*NPC).client as *mut gclient_t)).NPC_class == CLASS_SWAMPTROOPER {
                        // these guys can see in in/through water pretty well
                        vis_rating = 0.10;
                    } else {
                        vis_rating = 0.35;
                    }
                } else {
                    // else, if we're both in water
                    if (*((*NPC).client as *mut gclient_t)).NPC_class == CLASS_SWAMPTROOPER {
                        // I can see him just fine
                    } else {
                        vis_rating = 0.15;
                    }
                }
            } else if (contents & CONTENTS_FOG) != 0 {
                vis_rating = 0.15;
            }

            target_rating *= 1.0 - vis_rating;

            // ...Motion draws the eye quickly
            target_rating += speed_rating * SPEED_SCALE;
            target_rating += turning_rating * TURNING_SCALE;
            // FIXME: check to see if they're animating, too? (Raven comment).

            // ...Smaller targets are harder to indentify
            if target_crouching {
                target_rating *= 0.9;
            }

            // If he's violated the threshold, then realize him
            let (realize, cautious) =
                if (*((*NPC).client as *mut gclient_t)).NPC_class == CLASS_SWAMPTROOPER {
                    // swamptroopers can see much better
                    (CAUTIOUS_THRESHOLD, CAUTIOUS_THRESHOLD * 0.75)
                } else {
                    (REALIZE_THRESHOLD, CAUTIOUS_THRESHOLD * 0.75)
                };

            if target_rating > realize && ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
                G_SetEnemy(ctx, NPC, target);
                (*NPCInfo).enemyLastSeenTime = world.level.time;
                TIMER_Set(
                    ctx,
                    NPC,
                    c"attackDelay".as_ptr(),
                    (*ctx.world).bg_state.rng.Q_irand(500, 2500),
                );
                return qtrue;
            }

            // If he's above the caution threshold, then realize him in a few
            // seconds unless he moves to cover
            if target_rating > cautious && ((*NPCInfo).scriptFlags & SCF_IGNORE_ALERTS) == 0 {
                // FIXME: ambushing guys should never talk (Raven comment).
                if TIMER_Done(ctx, NPC, c"enemyLastVisible".as_ptr()) != 0 {
                    // If we haven't already, start the counter
                    let lookTime = (*ctx.world).bg_state.rng.Q_irand(4500, 8500);
                    TIMER_Set(ctx, NPC, c"enemyLastVisible".as_ptr(), lookTime);
                    // TODO: Play a sound along the lines of, "Huh? What was that?" (Raven comment).
                    ST_Speech(ctx, NPC, SPEECH_SIGHT, 0.0);
                    NPC_TempLookTarget(ctx, NPC, (*target).s.number, lookTime, lookTime);
                    // FIXME: set desired yaw and pitch towards this guy? (Raven comment).
                } else if TIMER_Get(ctx, NPC, c"enemyLastVisible".as_ptr())
                    <= world.level.time + 500
                    && ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0
                {
                    // FIXME: Is this reliable? (Raven comment).
                    if (*NPCInfo).rank < RANK_LT && (*ctx.world).bg_state.rng.Q_irand(0, 2) == 0 {
                        let interrogateTime = (*ctx.world).bg_state.rng.Q_irand(2000, 4000);
                        ST_Speech(ctx, NPC, SPEECH_SUSPICIOUS, 0.0);
                        TIMER_Set(ctx, NPC, c"interrogating".as_ptr(), interrogateTime);
                        G_SetEnemy(ctx, NPC, target);
                        (*NPCInfo).enemyLastSeenTime = world.level.time;
                        TIMER_Set(ctx, NPC, c"attackDelay".as_ptr(), interrogateTime);
                        TIMER_Set(ctx, NPC, c"stand".as_ptr(), interrogateTime);
                    } else {
                        G_SetEnemy(ctx, NPC, target);
                        (*NPCInfo).enemyLastSeenTime = world.level.time;
                        // FIXME: ambush guys (like those popping out of water)
                        // shouldn't delay... (Raven comment).
                        TIMER_Set(
                            ctx,
                            NPC,
                            c"attackDelay".as_ptr(),
                            (*ctx.world).bg_state.rng.Q_irand(500, 2500),
                        );
                        TIMER_Set(
                            ctx,
                            NPC,
                            c"stand".as_ptr(),
                            (*ctx.world).bg_state.rng.Q_irand(500, 2500),
                        );
                    }
                    return qtrue;
                } else {
                    return qfalse;
                }
            }
        }

        qfalse
    }
}

/// Raven `NPC_CheckPlayerTeamStealth`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:727-757`
pub fn NPC_CheckPlayerTeamStealth(ctx: GameContext<'_>) -> qboolean {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;

        /*
        NPC_CheckEnemyStealth( &g_entities[0] );	//Change this pointer to assess other entities
        */
        for i in 0..ENTITYNUM_WORLD {
            let enemy = &mut world.g_entities[i as usize] as *mut gentity_t;

            if (*enemy).inuse == qfalse {
                continue;
            }

            if !enemy.is_null()
                && !(*enemy).client.is_null()
                && NPC_ValidEnemy(ctx, enemy) != qfalse
                && (*((*enemy).client as *mut gclient_t)).playerTeam
                    == (*((*NPC).client as *mut gclient_t)).enemyTeam
            {
                // Change this pointer to assess other entities
                if NPC_CheckEnemyStealth(ctx, enemy) != qfalse {
                    return qtrue;
                }
            }
        }
        qfalse
    }
}

/// Raven `NPC_ST_InvestigateEvent`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:766-919`
pub fn NPC_ST_InvestigateEvent(
    ctx: GameContext<'_>,
    eventID: c_int,
    extraSuspicious: qboolean,
) -> qboolean {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        // If they've given themselves away, just take them as an enemy
        if (*NPCInfo).confusionTime < world.level.time {
            if world.level.alertEvents[eventID as usize].level == AEL_DISCOVERED
                && ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0
            {
                (*NPCInfo).lastAlertID = world.level.alertEvents[eventID as usize].ID;
                let owner = world.level.alertEvents[eventID as usize].owner;
                if owner.is_null()
                    || (*owner).client.is_null()
                    || (*owner).health <= 0
                    || (*((*owner).client as *mut gclient_t)).playerTeam
                        != (*((*NPC).client as *mut gclient_t)).enemyTeam
                {
                    // not an enemy
                    return qfalse;
                }
                // FIXME: what if can't actually see enemy... (Raven comment).
                // ST_Speech( NPC, SPEECH_CHARGE, 0 ); (Raven, commented out).
                G_SetEnemy(ctx, NPC, owner);
                (*NPCInfo).enemyLastSeenTime = world.level.time;
                TIMER_Set(
                    ctx,
                    NPC,
                    c"attackDelay".as_ptr(),
                    (*ctx.world).bg_state.rng.Q_irand(500, 2500),
                );
                if world.level.alertEvents[eventID as usize].r#type == alertEventType_e::AET_SOUND {
                    // heard him, didn't see him, stick for a bit
                    TIMER_Set(
                        ctx,
                        NPC,
                        c"roamTime".as_ptr(),
                        (*ctx.world).bg_state.rng.Q_irand(500, 2500),
                    );
                }
                return qtrue;
            }
        }

        // don't look at the same alert twice
        if world.level.alertEvents[eventID as usize].ID == (*NPCInfo).lastAlertID {
            return qfalse;
        }
        (*NPCInfo).lastAlertID = world.level.alertEvents[eventID as usize].ID;

        // Must be ready to take another sound event
        /*
        if ( NPCInfo->investigateSoundDebounceTime > level.time )
        {
            return qfalse;
        }
        */

        if world.level.alertEvents[eventID as usize].r#type == alertEventType_e::AET_SIGHT {
            // sight alert, check the light level
            if (world.level.alertEvents[eventID as usize].light as c_int)
                < (*ctx.world)
                    .bg_state
                    .rng
                    .Q_irand(ST_MIN_LIGHT_THRESHOLD, ST_MAX_LIGHT_THRESHOLD)
            {
                // below my threshhold of potentially seeing
                return qfalse;
            }
        }

        // Save the position for movement (if necessary)
        (*NPCInfo).investigateGoal = world.level.alertEvents[eventID as usize].position;

        // First awareness of it
        (*NPCInfo).investigateCount += if extraSuspicious != qfalse { 2 } else { 1 };

        // Clamp the value
        if (*NPCInfo).investigateCount > 4 {
            (*NPCInfo).investigateCount = 4;
        }

        // See if we should walk over and investigate
        if world.level.alertEvents[eventID as usize].level as i32 > AEL_MINOR as i32
            && (*NPCInfo).investigateCount > 1
            && ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) != 0
        {
            // make it so they can walk right to this point and look at it rather
            // than having to use combatPoints.
            // Oracle passes `NPCInfo->investigateGoal` itself by reference, so the
            // bbox-expanded value persists in the field in every branch (including
            // the `trace.fraction >= 1.0` "too high to bother" branch). Write
            // directly into the field, not a local copy.
            if G_ExpandPointToBBox(
                ctx,
                &mut (*NPCInfo).investigateGoal,
                (*NPC).r.mins,
                (*NPC).r.maxs,
                (*NPC).s.number,
                ((*NPC).clipmask & !CONTENTS_BODY) | CONTENTS_BOTCLIP,
            ) != qfalse
            {
                // we were able to move the investigateGoal to a point in which our
                // bbox would fit — drop the goal to the ground so we can get at it
                let mut end = (*NPCInfo).investigateGoal;
                end[2] -= 512.0; // FIXME: not always right? (Raven comment).
                let mut trace: trace_t = core::mem::zeroed();
                trap::Trace(
                    ctx.engine,
                    GTraceArgs::new(
                        &mut trace as *mut trace_t,
                        &(*NPCInfo).investigateGoal as *const vec3_t,
                        &(*NPC).r.mins as *const vec3_t,
                        &(*NPC).r.maxs as *const vec3_t,
                        &end as *const vec3_t,
                        ENTITYNUM_NONE,
                        ((*NPC).clipmask & !CONTENTS_BODY) | CONTENTS_BOTCLIP,
                    ),
                );
                if trace.fraction >= 1.0 {
                    // too high to even bother
                    // FIXME: look at them??? (Raven comment).
                } else {
                    (*NPCInfo).investigateGoal = trace.endpos;
                    NPC_SetMoveGoal(
                        ctx,
                        NPC,
                        (*NPCInfo).investigateGoal,
                        16,
                        qtrue,
                        -1,
                        core::ptr::null_mut(),
                    );
                    (*NPCInfo).localState = LSTATE_INVESTIGATE;
                }
            } else {
                let id = NPC_FindCombatPoint(
                    ctx,
                    (*NPCInfo).investigateGoal,
                    (*NPCInfo).investigateGoal,
                    (*NPCInfo).investigateGoal,
                    CP_INVESTIGATE | CP_HAS_ROUTE,
                    0.0,
                    -1,
                );

                if id != -1 {
                    NPC_SetMoveGoal(
                        ctx,
                        NPC,
                        world.level.combatPoints[id as usize].origin,
                        16,
                        qtrue,
                        id,
                        core::ptr::null_mut(),
                    );
                    (*NPCInfo).localState = LSTATE_INVESTIGATE;
                }
            }
            // Say something
            // FIXME: only if have others in group... (Raven comment).
            if (*NPCInfo).investigateDebounceTime + (*NPCInfo).pauseTime > world.level.time {
                // was already investigating
                let group = (*NPCInfo).group;
                if !group.is_null()
                    && !(*group).commander.is_null()
                    && !(*(*group).commander).client.is_null()
                    && (*((*(*group).commander).client as *mut gclient_t)).NPC_class
                        == CLASS_IMPERIAL
                    && (*ctx.world).bg_state.rng.Q_irand(0, 3) == 0
                {
                    ST_Speech(ctx, (*group).commander, SPEECH_LOOK, 0.0);
                } else {
                    ST_Speech(ctx, NPC, SPEECH_LOOK, 0.0);
                }
            } else {
                if world.level.alertEvents[eventID as usize].r#type == alertEventType_e::AET_SIGHT {
                    ST_Speech(ctx, NPC, SPEECH_SIGHT, 0.0);
                } else if world.level.alertEvents[eventID as usize].r#type
                    == alertEventType_e::AET_SOUND
                {
                    ST_Speech(ctx, NPC, SPEECH_SOUND, 0.0);
                }
            }
            // Setup the debounce info
            (*NPCInfo).investigateDebounceTime = (*NPCInfo).investigateCount * 5000;
            (*NPCInfo).investigateSoundDebounceTime = world.level.time + 2000;
            (*NPCInfo).pauseTime = world.level.time;
        } else {
            // just look?
            if world.level.alertEvents[eventID as usize].r#type == alertEventType_e::AET_SIGHT {
                ST_Speech(ctx, NPC, SPEECH_SIGHT, 0.0);
            } else if world.level.alertEvents[eventID as usize].r#type
                == alertEventType_e::AET_SOUND
            {
                ST_Speech(ctx, NPC, SPEECH_SOUND, 0.0);
            }
            (*NPCInfo).investigateDebounceTime = (*NPCInfo).investigateCount * 1000;
            (*NPCInfo).investigateSoundDebounceTime = world.level.time + 1000;
            (*NPCInfo).pauseTime = world.level.time;
            (*NPCInfo).investigateGoal = world.level.alertEvents[eventID as usize].position;
        }

        if world.level.alertEvents[eventID as usize].level as i32 >= AEL_DANGER as i32 {
            (*NPCInfo).investigateDebounceTime = (*ctx.world).bg_state.rng.Q_irand(500, 2500);
        }

        // Start investigating
        (*NPCInfo).tempBehavior = BS_INVESTIGATE;
        qtrue
    }
}

/// Raven `ST_OffsetLook`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:927-938`
pub fn ST_OffsetLook(ctx: GameContext<'_>, offset: f32, out: &mut vec3_t) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        let mut angles: vec3_t = [0.0; 3];
        GetAnglesForDirection(
            (*NPC).r.currentOrigin,
            (*NPCInfo).investigateGoal,
            &mut angles,
        );
        angles[1] += offset; // YAW
        let mut forward: vec3_t = [0.0; 3];
        AngleVectors(angles, Some(&mut forward), None, None);
        // VectorMA( NPC->r.currentOrigin, 64, forward, out );
        out[0] = (*NPC).r.currentOrigin[0] + 64.0 * forward[0];
        out[1] = (*NPC).r.currentOrigin[1] + 64.0 * forward[1];
        out[2] = (*NPC).r.currentOrigin[2] + 64.0 * forward[2];

        let mut temp: vec3_t = [0.0; 3];
        CalcEntitySpot(ctx, NPC, SPOT_HEAD, &mut temp);
        out[2] = temp[2];
    }
}

/// Raven `ST_LookAround`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:946-970`
pub fn ST_LookAround(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        let mut lookPos: vec3_t = [0.0; 3];
        let perc = (world.level.time - (*NPCInfo).pauseTime) as f32
            / (*NPCInfo).investigateDebounceTime as f32;

        // Keep looking at the spot
        if perc < 0.25 {
            lookPos = (*NPCInfo).investigateGoal;
        } else if perc < 0.5 {
            // Look up but straight ahead
            ST_OffsetLook(ctx, 0.0, &mut lookPos);
        } else if perc < 0.75 {
            // Look right
            ST_OffsetLook(ctx, 45.0, &mut lookPos);
        } else {
            // Look left
            ST_OffsetLook(ctx, -45.0, &mut lookPos);
        }

        NPC_FacePosition(ctx, lookPos, qtrue);
    }
}

/// Raven `NPC_BSST_Investigate`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:978-1069`
pub fn NPC_BSST_Investigate(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        // get group- mainly for group speech debouncing, but may use for group
        // scouting/investigating AI, too
        AI_GetGroup(ctx, NPC);

        if ((*NPCInfo).scriptFlags & SCF_FIRE_WEAPON) != 0 {
            WeaponThink(ctx, qtrue);
        }

        if (*NPCInfo).confusionTime < world.level.time {
            if ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
                // Look for an enemy
                if NPC_CheckPlayerTeamStealth(ctx) != qfalse {
                    // NPCInfo->behaviorState = BS_HUNT_AND_KILL; // should be auto now (Raven comment).
                    ST_Speech(ctx, NPC, SPEECH_DETECTED, 0.0);
                    (*NPCInfo).tempBehavior = BS_DEFAULT;
                    NPC_UpdateAngles(ctx, qtrue, qtrue);
                    return;
                }
            }
        }

        if ((*NPCInfo).scriptFlags & SCF_IGNORE_ALERTS) == 0 {
            let alertEvent = NPC_CheckAlertEvents(
                ctx,
                qtrue,
                qtrue,
                (*NPCInfo).lastAlertID,
                qfalse,
                AEL_MINOR as c_int,
            );

            // There is an event to look at
            if alertEvent >= 0 {
                if (*NPCInfo).confusionTime < world.level.time {
                    if NPC_CheckForDanger(ctx, alertEvent) != qfalse {
                        // running like hell
                        ST_Speech(ctx, NPC, SPEECH_COVER, 0.0); // FIXME: flee sound? (Raven comment).
                        return;
                    }
                }

                if world.level.alertEvents[alertEvent as usize].ID != (*NPCInfo).lastAlertID {
                    NPC_ST_InvestigateEvent(ctx, alertEvent, qtrue);
                }
            }
        }

        // If we're done looking, then just return to what we were doing
        if ((*NPCInfo).investigateDebounceTime + (*NPCInfo).pauseTime) < world.level.time {
            (*NPCInfo).tempBehavior = BS_DEFAULT;
            (*NPCInfo).goalEntity = ent_id_opt(ent_base(ctx), UpdateGoal(ctx));

            NPC_UpdateAngles(ctx, qtrue, qtrue);
            // Say something
            ST_Speech(ctx, NPC, SPEECH_GIVEUP, 0.0);
            return;
        }

        // FIXME: else, look for new alerts (Raven comment).

        // See if we're searching for the noise's origin
        if (*NPCInfo).localState == LSTATE_INVESTIGATE && (*NPCInfo).goalEntity != None {
            let goalEnt = ent_resolve_opt(ctx, (*NPCInfo).goalEntity);
            // See if we're there
            let flying = FlyingCreature(NPC);
            if NAV_HitNavGoal(
                (*NPC).r.currentOrigin,
                (*NPC).r.mins,
                (*NPC).r.maxs,
                (*goalEnt).r.currentOrigin,
                32,
                flying,
            ) == qfalse
            {
                world.globals.ucmd.buttons |= BUTTON_WALKING;

                // Try and move there
                if NPC_MoveToGoal(ctx, qtrue) != qfalse {
                    // Bump our times
                    (*NPCInfo).investigateDebounceTime = (*NPCInfo).investigateCount * 5000;
                    (*NPCInfo).pauseTime = world.level.time;

                    NPC_UpdateAngles(ctx, qtrue, qtrue);
                    return;
                }
            }

            // Otherwise we're done or have given up
            // Say something
            // ST_Speech( NPC, SPEECH_LOOK, 0.33f32 ); (Raven, commented out).
            (*NPCInfo).localState = LSTATE_NONE;
        }

        // Look around
        ST_LookAround(ctx);
    }
}

/// Raven `NPC_BSST_Patrol`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:1077-1181`
pub fn NPC_BSST_Patrol(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;
        let client = (*NPC).client as *mut gclient_t;

        // FIXME: pick up on bodies of dead buddies? (Raven comment).

        // get group- mainly for group speech debouncing, but may use for group
        // scouting/investigating AI, too
        AI_GetGroup(ctx, NPC);

        if (*NPCInfo).confusionTime < world.level.time {
            // Look for any enemies
            if ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
                if NPC_CheckPlayerTeamStealth(ctx) != qfalse {
                    // NPCInfo->behaviorState = BS_HUNT_AND_KILL; // should be auto now (Raven comment).
                    // NPC_AngerSound(); (Raven, commented out).
                    NPC_UpdateAngles(ctx, qtrue, qtrue);
                    return;
                }
            }
        }

        if ((*NPCInfo).scriptFlags & SCF_IGNORE_ALERTS) == 0 {
            let alertEvent =
                NPC_CheckAlertEvents(ctx, qtrue, qtrue, -1, qfalse, AEL_MINOR as c_int);

            // There is an event to look at
            if alertEvent >= 0 {
                if NPC_ST_InvestigateEvent(ctx, alertEvent, qfalse) != qfalse {
                    // actually going to investigate it
                    NPC_UpdateAngles(ctx, qtrue, qtrue);
                    return;
                }
            }
        }

        // If we have somewhere to go, then do that
        let goal = UpdateGoal(ctx);
        if goal != core::ptr::null_mut() {
            world.globals.ucmd.buttons |= BUTTON_WALKING;
            // ST_Move( NPCInfo->goalEntity ); (Raven, commented out).
            NPC_MoveToGoal(ctx, qtrue);
        } else {
            // if ( !(NPCInfo->scriptFlags&SCF_IGNORE_ALERTS) ) (Raven, commented out).
            if (*client).NPC_class != CLASS_IMPERIAL && (*client).NPC_class != CLASS_IMPWORKER {
                // imperials do not look around
                if TIMER_Done(ctx, NPC, c"enemyLastVisible".as_ptr()) != 0 {
                    // nothing suspicious, look around
                    if (*ctx.world).bg_state.rng.Q_irand(0, 30) == 0 {
                        (*NPCInfo).desiredYaw = (*NPC).s.angles[1] as f32
                            + (*ctx.world).bg_state.rng.Q_irand(-90, 90) as f32;
                    }
                    if (*ctx.world).bg_state.rng.Q_irand(0, 30) == 0 {
                        (*NPCInfo).desiredPitch = (*ctx.world).bg_state.rng.Q_irand(-20, 20) as f32;
                    }
                }
            }
        }

        NPC_UpdateAngles(ctx, qtrue, qtrue);
        // TEMP hack for Imperial stand anim
        if (*client).NPC_class == CLASS_IMPERIAL || (*client).NPC_class == CLASS_IMPWORKER {
            // hack
            if world.globals.ucmd.forwardmove != 0
                || world.globals.ucmd.rightmove != 0
                || world.globals.ucmd.upmove != 0
            {
                // moving
                if (*client).ps.torsoTimer <= 0 || (*client).ps.torsoAnim == BOTH_STAND4 as c_int {
                    if (world.globals.ucmd.buttons & BUTTON_WALKING) != 0
                        && ((*NPCInfo).scriptFlags & SCF_RUNNING) == 0
                    {
                        // not running, only set upper anim
                        // No longer overrides scripted anims
                        NPC_SetAnim(
                            ctx,
                            NPC,
                            SETANIM_TORSO,
                            BOTH_STAND4 as c_int,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        );
                        (*client).ps.torsoTimer = 200;
                    }
                }
            } else {
                // standing still, set both torso and legs anim
                // No longer overrides scripted anims
                if ((*client).ps.torsoTimer <= 0 || (*client).ps.torsoAnim == BOTH_STAND4 as c_int)
                    && ((*client).ps.legsTimer <= 0
                        || (*client).ps.legsAnim == BOTH_STAND4 as c_int)
                {
                    NPC_SetAnim(
                        ctx,
                        NPC,
                        SETANIM_BOTH,
                        BOTH_STAND4 as c_int,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                    (*client).ps.torsoTimer = 200;
                    (*client).ps.legsTimer = 200;
                }
            }
            // FIXME: this is a disgusting hack... (Raven comment).
            if (*client).ps.weapon != WP_NONE {
                ChangeWeapon(ctx, NPC, WP_NONE);
                (*client).ps.weapon = WP_NONE;
                (*client).ps.weaponstate = WEAPON_READY as c_int;
                /*
                if ( NPC->weaponModel[0] > 0 )
                {
                    gi.G2API_RemoveGhoul2Model( NPC->ghoul2, NPC->weaponModel[0] );
                    NPC->weaponModel[0] = -1;
                }
                */
                // rwwFIXMEFIXME: Do this? (Raven comment).
            }
        }
    }
}

/// Raven `ST_CheckMoveState`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:1212-1358`
pub fn ST_CheckMoveState(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        if trap::ICARUS_TaskIDPending(
            ctx.engine,
            GIcarusTaskidpendingArgs::new(NPC, TID_MOVE_NAV as c_int),
        ) != 0
        {
            // moving toward a goal that a script is waiting on, so don't stop for
            // anything!
            world.globals.r#move = qtrue;
        } else if (*NPCInfo).squadState == SQUAD_SCOUT {
            // See if we're a scout
            // If we're supposed to stay put, then stand there and fire
            if TIMER_Done(ctx, NPC, c"stick".as_ptr()) == qfalse {
                world.globals.r#move = qfalse;
                return;
            }

            // Otherwise, if we can see our target, just shoot
            if world.globals.enemyLOS != qfalse {
                if world.globals.enemyCS != qfalse {
                    // if we're going after our enemy, we can stop now
                    if (*NPCInfo).goalEntity == (*NPC).enemy {
                        AI_GroupUpdateSquadstates((*NPCInfo).group, NPC, SQUAD_STAND_AND_SHOOT);
                        world.globals.r#move = qfalse;
                        return;
                    }
                }
            } else {
                // Move to find our target
                world.globals.faceEnemy = qfalse;
            }

            /*
            if ( TIMER_Done( NPC, "scoutTime" ) )
            {
                AI_GroupUpdateSquadstates( NPCInfo->group, NPC, SQUAD_STAND_AND_SHOOT );
                TIMER_Set( NPC, "roamTime", (*ctx.world).bg_state.rng.Q_irand( 1000, 2000 ) );
                move = qfalse;
                return;
            }
            */
            // ucmd.buttons |= BUTTON_CAREFUL; (Raven, commented out).
        } else if (*NPCInfo).squadState == SQUAD_RETREAT {
            // See if we're running away
            if (*NPCInfo).goalEntity != None {
                world.globals.faceEnemy = qfalse;
            } else {
                // um, lost our goal? Just stand and shoot, then
                (*NPCInfo).squadState = SQUAD_STAND_AND_SHOOT;
            }
        } else if (*NPCInfo).squadState == SQUAD_TRANSITION {
            // see if we're heading to some other combatPoint
            // ucmd.buttons |= BUTTON_CAREFUL; (Raven, commented out).
            if (*NPCInfo).goalEntity == None {
                // um, lost our goal? Just stand and shoot, then
                (*NPCInfo).squadState = SQUAD_STAND_AND_SHOOT;
            }
        } else if (*NPCInfo).squadState == SQUAD_POINT {
            // see if we're at point, duck and fire
            if TIMER_Done(ctx, NPC, c"stick".as_ptr()) != 0 {
                AI_GroupUpdateSquadstates((*NPCInfo).group, NPC, SQUAD_STAND_AND_SHOOT);
                return;
            }

            world.globals.r#move = qfalse;
            return;
        } else if (*NPCInfo).squadState == SQUAD_STAND_AND_SHOOT {
            // see if we're just standing around
            // from this squadState we can transition to others?
            world.globals.r#move = qfalse;
            return;
        } else if (*NPCInfo).squadState == SQUAD_COVER {
            // see if we're hiding
            // Should we duck?
            world.globals.r#move = qfalse;
            return;
        } else if (*NPCInfo).squadState == SQUAD_IDLE {
            // see if we're just standing around
            if (*NPCInfo).goalEntity == None {
                world.globals.r#move = qfalse;
                return;
            }
        } else {
            // ?? invalid squadState! (Raven comment).
        }

        // See if we're moving towards a goal, not the enemy
        if ((*NPCInfo).goalEntity != (*NPC).enemy) && ((*NPCInfo).goalEntity != None) {
            let goalEnt = ent_resolve_opt(ctx, (*NPCInfo).goalEntity);
            // Did we make it?
            let flying = FlyingCreature(NPC);
            if NAV_HitNavGoal(
                (*NPC).r.currentOrigin,
                (*NPC).r.mins,
                (*NPC).r.maxs,
                (*goalEnt).r.currentOrigin,
                16,
                flying,
            ) != qfalse
                || (trap::ICARUS_TaskIDPending(
                    ctx.engine,
                    GIcarusTaskidpendingArgs::new(NPC, TID_MOVE_NAV as c_int),
                ) == 0
                    && (*NPCInfo).squadState == SQUAD_SCOUT
                    && world.globals.enemyLOS != qfalse
                    && world.globals.enemyDist <= 10000.0)
            {
                // either hit our navgoal or our navgoal was not a crucial
                // (scripted) one (maybe a combat point) and we're scouting and
                // found our enemy
                let mut newSquadState = SQUAD_STAND_AND_SHOOT;
                // we got where we wanted to go, set timers based on why we were
                // running
                match (*NPCInfo).squadState {
                    x if x == SQUAD_RETREAT => {
                        // was running away — done fleeing, obviously
                        let client = (*NPC).client as *mut gclient_t;
                        TIMER_Set(
                            ctx,
                            NPC,
                            c"duck".as_ptr(),
                            ((*client).pers.maxHealth - (*NPC).health) * 100,
                        );
                        TIMER_Set(
                            ctx,
                            NPC,
                            c"hideTime".as_ptr(),
                            (*ctx.world).bg_state.rng.Q_irand(3000, 7000),
                        );
                        TIMER_Set(ctx, NPC, c"flee".as_ptr(), -world.level.time);
                        newSquadState = SQUAD_COVER;
                    }
                    x if x == SQUAD_TRANSITION => {
                        // was heading for a combat point
                        TIMER_Set(
                            ctx,
                            NPC,
                            c"hideTime".as_ptr(),
                            (*ctx.world).bg_state.rng.Q_irand(2000, 4000),
                        );
                    }
                    x if x == SQUAD_SCOUT => {
                        // was running after player
                    }
                    _ => {}
                }
                AI_GroupUpdateSquadstates((*NPCInfo).group, NPC, newSquadState);
                NPC_ReachedGoal(ctx);
                // don't attack right away
                TIMER_Set(
                    ctx,
                    NPC,
                    c"attackDelay".as_ptr(),
                    (*ctx.world).bg_state.rng.Q_irand(250, 500),
                ); // FIXME: Slant for difficulty levels (Raven comment).
                   // don't do something else just yet
                TIMER_Set(
                    ctx,
                    NPC,
                    c"roamTime".as_ptr(),
                    (*ctx.world).bg_state.rng.Q_irand(1000, 4000),
                );
                return;
            }

            // keep going, hold of roamTimer until we get there
            TIMER_Set(
                ctx,
                NPC,
                c"roamTime".as_ptr(),
                (*ctx.world).bg_state.rng.Q_irand(4000, 8000),
            );
        }
    }
}

/// Raven `ST_ResolveBlockedShot`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:1360-1403`
pub fn ST_ResolveBlockedShot(ctx: GameContext<'_>, hit: c_int) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        // figure out how long we intend to stand here, max
        let stuckTime =
            if TIMER_Get(ctx, NPC, c"roamTime".as_ptr()) > TIMER_Get(ctx, NPC, c"stick".as_ptr()) {
                TIMER_Get(ctx, NPC, c"roamTime".as_ptr()) - world.level.time
            } else {
                TIMER_Get(ctx, NPC, c"stick".as_ptr()) - world.level.time
            };

        if TIMER_Done(ctx, NPC, c"duck".as_ptr()) != 0 {
            // we're not ducking
            if AI_GroupContainsEntNum((*NPCInfo).group, hit) != qfalse {
                let member = &mut world.g_entities[hit as usize] as *mut gentity_t;
                if TIMER_Done(ctx, member, c"duck".as_ptr()) != 0 {
                    // they aren't ducking
                    if TIMER_Done(ctx, member, c"stand".as_ptr()) != 0 {
                        // they're not being forced to stand
                        // tell them to duck at least as long as I'm not moving
                        TIMER_Set(ctx, member, c"duck".as_ptr(), stuckTime);
                        return;
                    }
                }
            }
        } else {
            // maybe we should stand
            if TIMER_Done(ctx, NPC, c"stand".as_ptr()) != 0 {
                // stand for as long as we'll be here
                TIMER_Set(ctx, NPC, c"stand".as_ptr(), stuckTime);
                return;
            }
        }
        // Hmm, can't resolve this by telling them to duck or telling me to stand
        // We need to move!
        TIMER_Set(ctx, NPC, c"roamTime".as_ptr(), -1);
        TIMER_Set(ctx, NPC, c"stick".as_ptr(), -1);
        TIMER_Set(ctx, NPC, c"duck".as_ptr(), -1);
        // Raven typo `"attakDelay"` preserved verbatim (parity: a distinct timer
        // key from "attackDelay", never read elsewhere).
        TIMER_Set(
            ctx,
            NPC,
            c"attakDelay".as_ptr(),
            (*ctx.world).bg_state.rng.Q_irand(1000, 3000),
        );
    }
}

/// Raven `ST_CheckFireState`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:1411-1534`
pub fn ST_CheckFireState(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;
        let client = (*NPC).client as *mut gclient_t;

        if world.globals.enemyCS != qfalse {
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

        if VectorCompare((*client).ps.velocity, vec3_origin) == qfalse {
            // if moving at all, don't do this
            return;
        }

        // See if we should continue to fire on their last position
        // !TIMER_Done( NPC, "stick" ) || (Raven, commented out).
        let group = (*NPCInfo).group;
        if world.globals.hitAlly == qfalse // we're not going to hit an ally
            && world.globals.enemyInFOV != qfalse // enemy is in our FOV // FIXME: or we don't have a clear LOS? (Raven comment).
            && (*NPCInfo).enemyLastSeenTime > 0 // we've seen the enemy
            && !group.is_null() // have a group
            && ((*group).numState[SQUAD_RETREAT as usize] > 0
                || (*group).numState[SQUAD_TRANSITION as usize] > 0
                || (*group).numState[SQUAD_SCOUT as usize] > 0)
        // laying down covering fire
        {
            if world.level.time - (*NPCInfo).enemyLastSeenTime < 10000 // we have seen the enemy in the last 10 seconds
                && (group.is_null() || world.level.time - (*group).lastSeenEnemyTime < 10000)
            // we are not in a group or the group has seen the enemy in the last 10 seconds
            {
                if (*ctx.world).bg_state.rng.Q_irand(0, 10) == 0 {
                    // Fire on the last known position
                    let mut muzzle: vec3_t = [0.0; 3];
                    let mut tooClose = qfalse;
                    let mut tooFar = qfalse;

                    CalcEntitySpot(ctx, NPC, SPOT_HEAD, &mut muzzle);
                    if VectorCompare(world.globals.impactPos, vec3_origin) != qfalse {
                        // never checked ShotEntity this frame, so must do a trace...
                        let mut forward: vec3_t = [0.0; 3];
                        AngleVectors((*client).ps.viewangles, Some(&mut forward), None, None);
                        let end: vec3_t = [
                            muzzle[0] + 8192.0 * forward[0],
                            muzzle[1] + 8192.0 * forward[1],
                            muzzle[2] + 8192.0 * forward[2],
                        ];
                        let mut tr: trace_t = core::mem::zeroed();
                        trap::Trace(
                            ctx.engine,
                            GTraceArgs::new(
                                &mut tr as *mut trace_t,
                                &muzzle as *const vec3_t,
                                &vec3_origin as *const vec3_t,
                                &vec3_origin as *const vec3_t,
                                &end as *const vec3_t,
                                (*NPC).s.number,
                                MASK_SHOT,
                            ),
                        );
                        world.globals.impactPos = tr.endpos;
                    }

                    // see if impact would be too close to me
                    let mut distThreshold: f32 = 16384.0; // default 128*128
                    match (*NPC).s.weapon {
                        WP_ROCKET_LAUNCHER | WP_FLECHETTE | WP_THERMAL | WP_TRIP_MINE
                        | WP_DET_PACK => {
                            distThreshold = 65536.0; // 256*256
                        }
                        WP_REPEATER => {
                            if ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) != 0 {
                                distThreshold = 65536.0;
                            }
                        }
                        _ => {}
                    }

                    let mut dist = DistanceSquared(world.globals.impactPos, muzzle);

                    if dist < distThreshold {
                        // impact would be too close to me
                        tooClose = qtrue;
                    } else if world.level.time - (*NPCInfo).enemyLastSeenTime > 5000
                        || (!group.is_null()
                            && world.level.time - (*group).lastSeenEnemyTime > 5000)
                    {
                        // we've haven't seen them in the last 5 seconds — see if
                        // it's too far from where he is
                        distThreshold = 65536.0; // default 256*256
                        match (*NPC).s.weapon {
                            WP_ROCKET_LAUNCHER | WP_FLECHETTE | WP_THERMAL | WP_TRIP_MINE
                            | WP_DET_PACK => {
                                distThreshold = 262144.0; // 512*512
                            }
                            WP_REPEATER => {
                                if ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) != 0 {
                                    distThreshold = 262144.0;
                                }
                            }
                            _ => {}
                        }
                        dist = DistanceSquared(
                            world.globals.impactPos,
                            (*NPCInfo).enemyLastSeenLocation,
                        );
                        if dist > distThreshold {
                            // impact would be too far from enemy
                            tooFar = qtrue;
                        }
                    }

                    if tooClose == qfalse && tooFar == qfalse {
                        // okay too shoot at last pos
                        let mut dir: vec3_t = [
                            (*NPCInfo).enemyLastSeenLocation[0] - muzzle[0],
                            (*NPCInfo).enemyLastSeenLocation[1] - muzzle[1],
                            (*NPCInfo).enemyLastSeenLocation[2] - muzzle[2],
                        ];
                        VectorNormalize(&mut dir);
                        let mut angles: vec3_t = [0.0; 3];
                        vectoangles(dir, &mut angles);

                        (*NPCInfo).desiredYaw = angles[1]; // YAW
                        (*NPCInfo).desiredPitch = angles[0]; // PITCH

                        world.globals.shoot = qtrue;
                        world.globals.faceEnemy = qfalse;
                        // AI_GroupUpdateSquadstates( NPCInfo->group, NPC, SQUAD_STAND_AND_SHOOT ); (Raven, commented out).
                        return;
                    }
                }
            }
        }
    }
}

/// Raven `ST_TrackEnemy`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:1536-1548`
pub fn ST_TrackEnemy(ctx: GameContext<'_>, self_: *mut gentity_t, enemyPos: vec3_t) {
    unsafe {
        let world = &mut *ctx.world;
        // clear timers
        TIMER_Set(
            ctx,
            self_,
            c"attackDelay".as_ptr(),
            (*ctx.world).bg_state.rng.Q_irand(1000, 2000),
        );
        // TIMER_Set( self, "duck", -1 ); (Raven, commented out).
        TIMER_Set(
            ctx,
            self_,
            c"stick".as_ptr(),
            (*ctx.world).bg_state.rng.Q_irand(500, 1500),
        );
        TIMER_Set(ctx, self_, c"stand".as_ptr(), -1);
        TIMER_Set(
            ctx,
            self_,
            c"scoutTime".as_ptr(),
            TIMER_Get(ctx, self_, c"stick".as_ptr()) - world.level.time
                + (*ctx.world).bg_state.rng.Q_irand(5000, 10000),
        );
        // leave my combat point
        let npc = (*self_).NPC as *mut gNPC_t;
        NPC_FreeCombatPoint(ctx, (*npc).combatPoint, qfalse);
        // go after his last seen pos
        NPC_SetMoveGoal(ctx, self_, enemyPos, 16, qfalse, -1, core::ptr::null_mut());
    }
}

/// Raven `ST_ApproachEnemy`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:1550-1561`
pub fn ST_ApproachEnemy(ctx: GameContext<'_>, self_: *mut gentity_t) -> c_int {
    unsafe {
        let world = &mut *ctx.world;
        TIMER_Set(
            ctx,
            self_,
            c"attackDelay".as_ptr(),
            (*ctx.world).bg_state.rng.Q_irand(250, 500),
        );
        // TIMER_Set( self, "duck", -1 ); (Raven, commented out).
        TIMER_Set(
            ctx,
            self_,
            c"stick".as_ptr(),
            (*ctx.world).bg_state.rng.Q_irand(1000, 2000),
        );
        TIMER_Set(ctx, self_, c"stand".as_ptr(), -1);
        TIMER_Set(
            ctx,
            self_,
            c"scoutTime".as_ptr(),
            TIMER_Get(ctx, self_, c"stick".as_ptr()) - world.level.time
                + (*ctx.world).bg_state.rng.Q_irand(5000, 10000),
        );
        // leave my combat point
        let npc = (*self_).NPC as *mut gNPC_t;
        NPC_FreeCombatPoint(ctx, (*npc).combatPoint, qfalse);
        // return the relevant combat point flags
        CP_CLEAR | CP_CLOSEST
    }
}

/// Raven `ST_HuntEnemy`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:1563-1577`
pub fn ST_HuntEnemy(ctx: GameContext<'_>, self_: *mut gentity_t) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        // TIMER_Set( NPC, "attackDelay", (*ctx.world).bg_state.rng.Q_irand( 250, 500 ) ); // Disabled this
        // for now, guys who couldn't hunt would never attack (Raven comment).
        // TIMER_Set( NPC, "duck", -1 ); (Raven, commented out).
        TIMER_Set(
            ctx,
            NPC,
            c"stick".as_ptr(),
            (*ctx.world).bg_state.rng.Q_irand(250, 1000),
        );
        TIMER_Set(ctx, NPC, c"stand".as_ptr(), -1);
        TIMER_Set(
            ctx,
            NPC,
            c"scoutTime".as_ptr(),
            TIMER_Get(ctx, NPC, c"stick".as_ptr()) - world.level.time
                + (*ctx.world).bg_state.rng.Q_irand(5000, 10000),
        );
        // leave my combat point
        NPC_FreeCombatPoint(ctx, (*NPCInfo).combatPoint, qfalse);
        // go directly after the enemy
        if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) != 0 {
            let self_npc = (*self_).NPC as *mut gNPC_t;
            (*self_npc).goalEntity = (*NPC).enemy;
        }
    }
}

/// Raven `ST_TransferTimers`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:1579-1593`
pub fn ST_TransferTimers(ctx: GameContext<'_>, self_: *mut gentity_t, other: *mut gentity_t) {
    unsafe {
        let world = &mut *ctx.world;
        TIMER_Set(
            ctx,
            other,
            c"attackDelay".as_ptr(),
            TIMER_Get(ctx, self_, c"attackDelay".as_ptr()) - world.level.time,
        );
        TIMER_Set(
            ctx,
            other,
            c"duck".as_ptr(),
            TIMER_Get(ctx, self_, c"duck".as_ptr()) - world.level.time,
        );
        TIMER_Set(
            ctx,
            other,
            c"stick".as_ptr(),
            TIMER_Get(ctx, self_, c"stick".as_ptr()) - world.level.time,
        );
        // Raven reads timer key `"scout"`, not `"scoutTime"` — likely a bug, kept
        // verbatim for parity (S19: preserve the faithful reading).
        TIMER_Set(
            ctx,
            other,
            c"scoutTime".as_ptr(),
            TIMER_Get(ctx, self_, c"scout".as_ptr()) - world.level.time,
        );
        TIMER_Set(
            ctx,
            other,
            c"roamTime".as_ptr(),
            TIMER_Get(ctx, self_, c"roamTime".as_ptr()) - world.level.time,
        );
        TIMER_Set(
            ctx,
            other,
            c"stand".as_ptr(),
            TIMER_Get(ctx, self_, c"stand".as_ptr()) - world.level.time,
        );
        TIMER_Set(ctx, self_, c"attackDelay".as_ptr(), -1);
        TIMER_Set(ctx, self_, c"duck".as_ptr(), -1);
        TIMER_Set(ctx, self_, c"stick".as_ptr(), -1);
        TIMER_Set(ctx, self_, c"scoutTime".as_ptr(), -1);
        TIMER_Set(ctx, self_, c"roamTime".as_ptr(), -1);
        TIMER_Set(ctx, self_, c"stand".as_ptr(), -1);
    }
}

/// Raven `ST_TransferMoveGoal`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:1595-1626`
pub fn ST_TransferMoveGoal(ctx: GameContext<'_>, self_: *mut gentity_t, other: *mut gentity_t) {
    unsafe {
        let world = &mut *ctx.world;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;
        let selfNpc = (*self_).NPC as *mut gNPC_t;
        let otherNpc = (*other).NPC as *mut gNPC_t;

        if trap::ICARUS_TaskIDPending(
            ctx.engine,
            GIcarusTaskidpendingArgs::new(self_, TID_MOVE_NAV as c_int),
        ) != 0
        {
            // can't transfer movegoal when a script we're running is waiting to
            // complete
            return;
        }
        if (*selfNpc).combatPoint != -1 {
            // I've got a combatPoint I'm going to, give it to him
            (*otherNpc).combatPoint = (*selfNpc).combatPoint;
            (*selfNpc).lastFailedCombatPoint = (*selfNpc).combatPoint;
            (*selfNpc).combatPoint = -1;
        } else {
            // I must be going for a goal, give that to him instead
            if (*selfNpc).goalEntity == (*selfNpc).tempGoal {
                let tempGoalEnt = ent_resolve_opt(ctx, (*selfNpc).tempGoal);
                let isNavGoal = if ((*tempGoalEnt).flags & FL_NAVGOAL) != 0 {
                    qtrue
                } else {
                    qfalse
                };
                NPC_SetMoveGoal(
                    ctx,
                    other,
                    (*tempGoalEnt).r.currentOrigin,
                    (*selfNpc).goalRadius,
                    isNavGoal,
                    -1,
                    core::ptr::null_mut(),
                );
            } else {
                (*otherNpc).goalEntity = (*selfNpc).goalEntity;
            }
        }
        // give him my squadstate
        AI_GroupUpdateSquadstates((*selfNpc).group, other, (*NPCInfo).squadState);

        // give him my timers and clear mine
        ST_TransferTimers(ctx, self_, other);

        // now make me stand around for a second or two at least
        AI_GroupUpdateSquadstates((*selfNpc).group, self_, SQUAD_STAND_AND_SHOOT);
        TIMER_Set(
            ctx,
            self_,
            c"stand".as_ptr(),
            (*ctx.world).bg_state.rng.Q_irand(1000, 3000),
        );
    }
}

/// Raven `ST_GetCPFlags`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:1628-1710`
pub fn ST_GetCPFlags(ctx: GameContext<'_>) -> c_int {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;
        let mut cpFlags: c_int = 0;

        if !NPC.is_null() && !(*NPCInfo).group.is_null() {
            let group = (*NPCInfo).group;
            let client = (*NPC).client as *mut gclient_t;
            if NPC == (*group).commander && (*client).NPC_class == CLASS_IMPERIAL {
                // imperials hang back and give orders
                if (*group).numGroup > 1
                    && (*ctx.world).bg_state.rng.Q_irand(-3, (*group).numGroup) > 1
                {
                    // FIXME: make sure he's giving orders with these lines (Raven comment).
                    if (*ctx.world).bg_state.rng.Q_irand(0, 1) != 0 {
                        ST_Speech(ctx, NPC, SPEECH_CHASE, 0.5);
                    } else {
                        ST_Speech(ctx, NPC, SPEECH_YELL, 0.5);
                    }
                }
                cpFlags = CP_CLEAR | CP_COVER | CP_AVOID | CP_SAFE | CP_RETREAT;
            } else if (*group).morale < 0 {
                // hide
                cpFlags = CP_COVER | CP_AVOID | CP_SAFE | CP_RETREAT;
            } else if (*group).morale < (*group).numGroup {
                // morale is low for our size
                let moraleDrop = (*group).numGroup - (*group).morale;
                if moraleDrop < -6 {
                    // flee (no clear shot needed)
                    cpFlags = CP_FLEE | CP_RETREAT | CP_COVER | CP_AVOID | CP_SAFE;
                } else if moraleDrop < -3 {
                    // retreat (no clear shot needed)
                    cpFlags = CP_RETREAT | CP_COVER | CP_AVOID | CP_SAFE;
                } else if moraleDrop < 0 {
                    // cover (no clear shot needed)
                    cpFlags = CP_COVER | CP_AVOID | CP_SAFE;
                }
            } else {
                let moraleBoost = (*group).morale - (*group).numGroup;
                if moraleBoost > 20 {
                    // charge to any one and outflank (no cover needed)
                    cpFlags = CP_CLEAR | CP_FLANK | CP_APPROACH_ENEMY;
                } else if moraleBoost > 15 {
                    // charge to closest one (no cover needed)
                    cpFlags = CP_CLEAR | CP_CLOSEST | CP_APPROACH_ENEMY;
                } else if moraleBoost > 10 {
                    // charge closer (no cover needed)
                    cpFlags = CP_CLEAR | CP_APPROACH_ENEMY;
                }
            }
        }
        if cpFlags == 0 {
            // at some medium level of morale
            match (*ctx.world).bg_state.rng.Q_irand(0, 3) {
                0 => cpFlags = CP_CLEAR | CP_COVER | CP_NEAREST, // just take the nearest one
                1 => cpFlags = CP_CLEAR | CP_COVER | CP_APPROACH_ENEMY, // take one closer to the enemy
                2 => cpFlags = CP_CLEAR | CP_COVER | CP_CLOSEST | CP_APPROACH_ENEMY, // take the one closest to the enemy
                3 => cpFlags = CP_CLEAR | CP_COVER | CP_FLANK | CP_APPROACH_ENEMY, // take the one on the other side of the enemy
                _ => {}
            }
        }
        if !NPC.is_null() && ((*NPCInfo).scriptFlags & SCF_USE_CP_NEAREST) != 0 {
            cpFlags &= !(CP_FLANK | CP_APPROACH_ENEMY | CP_CLOSEST);
            cpFlags |= CP_NEAREST;
        }
        cpFlags
    }
}

/// Raven `ST_Commander`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:1724-2401`
pub fn ST_Commander(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;
        let group = (*NPCInfo).group;
        let mut runner = qfalse;
        let mut enemyLost = qfalse;
        let mut enemyProtected = qfalse;

        (*group).processed = qtrue;

        if (*group).enemy.is_null() || (*(*group).enemy).client.is_null() {
            // hmm, no enemy...?!
            return;
        }

        // FIXME: have this group commander check the enemy group... (Raven comment).
        // FIXME: find the group commander and have him occasionally give orders... (Raven comment).
        // FIXME: start fleeing when only a couple of you vs. a lightsaber... (Raven comment).

        SaveNPCGlobals(ctx);

        if (*group).lastSeenEnemyTime < world.level.time - 180000 {
            // dissolve the group
            ST_Speech(ctx, NPC, SPEECH_LOST, 0.0);
            (*(*group).enemy).waypoint =
                NAV_FindClosestWaypointForEnt(ctx, (*group).enemy, WAYPOINT_NONE);
            for i in 0..(*group).numGroup {
                let member = &mut world.g_entities[(*group).member[i as usize].number as usize]
                    as *mut gentity_t;
                SetNPCGlobals(ctx, member);
                if trap::ICARUS_TaskIDPending(
                    ctx.engine,
                    GIcarusTaskidpendingArgs::new(NPC, TID_MOVE_NAV as c_int),
                ) != 0
                {
                    // running somewhere that a script requires us to go, don't break
                    continue;
                }
                if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) == 0 {
                    // not allowed to move on my own
                    continue;
                }
                // Lost enemy for three minutes? go into search mode?
                G_ClearEnemy(ctx, NPC);
                (*NPC).waypoint =
                    NAV_FindClosestWaypointForEnt(ctx, NPC, (*(*group).enemy).waypoint);
                if (*NPC).waypoint == WAYPOINT_NONE {
                    (*NPCInfo).behaviorState = BS_DEFAULT; // BS_PATROL;
                } else if (*(*group).enemy).waypoint == WAYPOINT_NONE
                    || trap::Nav_GetPathCost(
                        ctx.engine,
                        GNavGetpathcostArgs::new((*NPC).waypoint, (*(*group).enemy).waypoint),
                    ) >= Q3_INFINITE
                {
                    NPC_BSSearchStart(ctx, (*NPC).waypoint, BS_SEARCH);
                } else {
                    NPC_BSSearchStart(ctx, (*(*group).enemy).waypoint, BS_SEARCH);
                }
            }
            (*group).enemy = core::ptr::null_mut();
            RestoreNPCGlobals(ctx);
            return;
        }

        // See if anyone in our group is not alerted and alert them
        /*
        for ( i = 0; i < group->numGroup; i++ )
        {
            member = &g_entities[group->member[i].number];
            if ( !member->enemy )
            {
                if ( group->member[i].closestBuddy != ENTITYNUM_NONE )
                {
                    buddy = &g_entities[group->member[i].closestBuddy];
                    if ( buddy->enemy == group->enemy )
                    {
                        SetNPCGlobals( buddy );
                        ST_Speech( NPC, SPEECH_CHARGE, 0.7f32 );
                    }
                }
                SetNPCGlobals( member );
                G_SetEnemy( member, group->enemy );
            }
        }
        */
        // Okay, everyone is mad

        // see if anyone is running
        if (*group).numState[SQUAD_SCOUT as usize] > 0
            || (*group).numState[SQUAD_TRANSITION as usize] > 0
            || (*group).numState[SQUAD_RETREAT as usize] > 0
        {
            // someone is running
            runner = qtrue;
        }

        if
        /* !runner && */
        (*group).lastSeenEnemyTime > world.level.time - 32000
            && (*group).lastSeenEnemyTime < world.level.time - 30000
        {
            // no-one has seen the enemy for 30 seconds// and no-one is running after him
            if !(*group).commander.is_null() && (*ctx.world).bg_state.rng.Q_irand(0, 1) == 0 {
                ST_Speech(ctx, (*group).commander, SPEECH_ESCAPING, 0.0);
            } else {
                ST_Speech(ctx, NPC, SPEECH_ESCAPING, 0.0);
            }
            // don't say this again
            (*NPCInfo).blockedSpeechDebounceTime = world.level.time + 3000;
        }

        if (*group).lastSeenEnemyTime < world.level.time - 10000 {
            // no-one has seen the enemy for at least 10 seconds! Should send a scout
            enemyLost = qtrue;
        }

        if (*group).lastClearShotTime < world.level.time - 5000 {
            // no-one has had a clear shot for 5 seconds!
            enemyProtected = qtrue;
        }

        // Go through the list:

        // Everyone should try to get to a combat point if possible
        let (curMemberNum, lastMemberNum): (c_int, c_int);
        if world.cvars.d_asynchronousGroupAI.integer != 0 {
            // do one member a turn
            (*group).activeMemberNum += 1;
            if (*group).activeMemberNum >= (*group).numGroup {
                (*group).activeMemberNum = 0;
            }
            curMemberNum = (*group).activeMemberNum;
            lastMemberNum = curMemberNum + 1;
        } else {
            curMemberNum = 0;
            lastMemberNum = (*group).numGroup;
        }
        for i in curMemberNum..lastMemberNum {
            // reset combat point flags
            let mut cp: c_int = -1;
            let mut cpFlags: c_int = 0;
            let mut squadState: c_int = SQUAD_IDLE;
            let mut avoidDist: f32 = 0.0;

            // get the next guy
            let member = &mut world.g_entities[(*group).member[i as usize].number as usize]
                as *mut gentity_t;
            if (*member).enemy == None {
                // don't include guys that aren't angry
                continue;
            }
            SetNPCGlobals(ctx, member);
            // re-fetch NPC/NPCInfo after SetNPCGlobals swaps the ambient pointers
            let NPC = world.globals.NPC as *mut gentity_t;
            let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

            if TIMER_Done(ctx, NPC, c"flee".as_ptr()) == 0 {
                // running away
                continue;
            }

            if trap::ICARUS_TaskIDPending(
                ctx.engine,
                GIcarusTaskidpendingArgs::new(NPC, TID_MOVE_NAV as c_int),
            ) != 0
            {
                // running somewhere that a script requires us to go
                continue;
            }

            let goalEnt = ent_resolve_opt(ctx, (*NPCInfo).goalEntity);
            if (*NPC).s.weapon == WP_NONE
                && (*NPCInfo).goalEntity != None
                && (*NPCInfo).goalEntity == (*NPCInfo).tempGoal
                && !goalEnt.is_null()
                && (*goalEnt).enemy != None
                && (*ent_resolve_opt(ctx, (*goalEnt).enemy)).s.eType == ET_ITEM as c_int
            {
                // running to pick up a gun, don't do other logic
                continue;
            }

            // see if this member should start running (only if have no officer...
            // FIXME: should always run from AEL_DANGER_GREAT? (Raven comment).
            if (*group).commander.is_null()
                || (*((*(*group).commander).NPC as *mut gNPC_t)).rank < RANK_ENSIGN
            {
                let alert =
                    NPC_CheckAlertEvents(ctx, qtrue, qtrue, -1, qfalse, AEL_DANGER as c_int);
                if NPC_CheckForDanger(ctx, alert) != qfalse {
                    // going to run
                    ST_Speech(ctx, NPC, SPEECH_COVER, 0.0);
                    continue;
                }
            }

            if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) == 0 {
                // not allowed to do combat-movement
                continue;
            }

            // check the local state
            if (*NPCInfo).squadState != SQUAD_RETREAT {
                // not already retreating
                let client = (*NPC).client as *mut gclient_t;
                if (*client).ps.weapon == WP_NONE {
                    // weaponless, should be hiding
                    let goalEnt = ent_resolve_opt(ctx, (*NPCInfo).goalEntity);
                    if goalEnt.is_null()
                        || (*goalEnt).enemy == None
                        || (*ent_resolve_opt(ctx, (*goalEnt).enemy)).s.eType != ET_ITEM as c_int
                    {
                        // not running after a pickup
                        let enemyEnt = ent_resolve_opt(ctx, (*NPC).enemy);
                        if TIMER_Done(ctx, NPC, c"hideTime".as_ptr()) != 0
                            || (DistanceSquared(
                                (*(*group).enemy).r.currentOrigin,
                                (*NPC).r.currentOrigin,
                            ) < 65536.0
                                && NPC_ClearLOS4(ctx, enemyEnt) != qfalse)
                        {
                            // done hiding or enemy near and can see us — er, start
                            // another flee I guess?
                            NPC_StartFlee(
                                ctx,
                                enemyEnt,
                                (*enemyEnt).r.currentOrigin,
                                AEL_DANGER_GREAT as c_int,
                                5000,
                                10000,
                            );
                        } // else, just hang here
                    }
                    continue;
                }
                if TIMER_Done(ctx, NPC, c"roamTime".as_ptr()) != 0
                    && TIMER_Done(ctx, NPC, c"hideTime".as_ptr()) != 0
                    && (*NPC).health > 10
                    && trap::InPVS(
                        ctx.engine,
                        GInPvsArgs::new(
                            &(*(*group).enemy).r.currentOrigin as *const vec3_t,
                            &(*NPC).r.currentOrigin as *const vec3_t,
                        ),
                    ) == 0
                {
                    // can't even see enemy — better go after him
                    cpFlags |= CP_CLEAR | CP_COVER;
                } else if (*NPCInfo).localState == LSTATE_UNDERFIRE {
                    // we've been shot
                    let enemyClient = (*(*group).enemy).client as *mut gclient_t;
                    match (*enemyClient).ps.weapon {
                        WP_SABER => {
                            if DistanceSquared((*(*group).enemy).r.currentOrigin, (*NPC).r.currentOrigin) < 65536.0 {
                                cpFlags |= CP_AVOID_ENEMY | CP_COVER | CP_AVOID | CP_RETREAT;
                                if (*group).commander.is_null()
                                    || (*((*(*group).commander).NPC as *mut gNPC_t)).rank < RANK_ENSIGN
                                {
                                    squadState = SQUAD_RETREAT;
                                }
                                avoidDist = 256.0;
                            }
                        }
                        _ /* default, WP_BLASTER */ => {
                            cpFlags |= CP_COVER;
                        }
                    }
                    if (*NPC).health <= 10 {
                        if (*group).commander.is_null()
                            || (*((*(*group).commander).NPC as *mut gNPC_t)).rank < RANK_ENSIGN
                        {
                            cpFlags |= CP_FLEE | CP_AVOID | CP_RETREAT;
                            squadState = SQUAD_RETREAT;
                        }
                    }
                } else {
                    // not hit, see if there are other reasons we should run
                    if trap::InPVS(
                        ctx.engine,
                        GInPvsArgs::new(
                            &(*NPC).r.currentOrigin as *const vec3_t,
                            &(*(*group).enemy).r.currentOrigin as *const vec3_t,
                        ),
                    ) != 0
                    {
                        // in the same room as enemy
                        if (*client).ps.weapon == WP_ROCKET_LAUNCHER
                            && DistanceSquared(
                                (*(*group).enemy).r.currentOrigin,
                                (*NPC).r.currentOrigin,
                            ) < MIN_ROCKET_DIST_SQUARED
                            && (*NPCInfo).squadState != SQUAD_TRANSITION
                        {
                            // too close for me to fire my weapon and I'm not already
                            // on the move
                            cpFlags |= CP_AVOID_ENEMY | CP_CLEAR | CP_AVOID;
                            avoidDist = 256.0;
                        } else {
                            let enemyClient = (*(*group).enemy).client as *mut gclient_t;
                            if (*enemyClient).ps.weapon == WP_SABER {
                                // if ( group->enemy->client->ps.SaberLength() > 0 ) (Raven, commented out).
                                if (*enemyClient).ps.saberHolstered == 0 {
                                    if DistanceSquared(
                                        (*(*group).enemy).r.currentOrigin,
                                        (*NPC).r.currentOrigin,
                                    ) < 65536.0
                                    {
                                        if TIMER_Done(ctx, NPC, c"hideTime".as_ptr()) != 0 {
                                            if (*NPCInfo).squadState != SQUAD_TRANSITION {
                                                // not already moving: FIXME: we need
                                                // to see if where we're going is
                                                // good now? (Raven comment).
                                                cpFlags |= CP_AVOID_ENEMY | CP_CLEAR | CP_AVOID;
                                                avoidDist = 256.0;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if cpFlags == 0 {
                // okay, we have no new enemy-driven reason to run... let's use
                // tactics now
                if runner != qfalse && (*NPCInfo).combatPoint != -1 {
                    // someone is running and we have a combat point already
                    if (*NPCInfo).squadState != SQUAD_SCOUT
                        && (*NPCInfo).squadState != SQUAD_TRANSITION
                        && (*NPCInfo).squadState != SQUAD_RETREAT
                    {
                        // it's not us
                        if TIMER_Done(ctx, NPC, c"verifyCP".as_ptr()) != 0
                            && DistanceSquared(
                                (*NPC).r.currentOrigin,
                                world.level.combatPoints[(*NPCInfo).combatPoint as usize].origin,
                            ) > 64.0 * 64.0
                        {
                            // 1 - 3 seconds have passed since you chose a CP, see
                            // if you're there since, for some reason, you've
                            // stopped running... — uh, WTF, we're not on our
                            // combat point? er, try again, I guess?
                            cp = (*NPCInfo).combatPoint;
                            cpFlags |= ST_GetCPFlags(ctx);
                        } else {
                            // cover them — stop ducking
                            TIMER_Set(ctx, NPC, c"duck".as_ptr(), -1);
                            // start shooting
                            TIMER_Set(ctx, NPC, c"attackDelay".as_ptr(), -1);
                            // AI should take care of the rest - fire at enemy
                        }
                    } else {
                        // we're running — see if we're blocked
                        if ((*NPCInfo).aiFlags & NPCAI_BLOCKED) != 0 {
                            // dammit, something is in our way — see if it's one of ours
                            for j in 0..(*group).numGroup {
                                if (*group).member[j as usize].number == (*NPCInfo).blockingEntNum {
                                    // we're being blocked by one of our own, pass our
                                    // goal onto them and I'll stand still
                                    let blocker = &mut world.g_entities
                                        [(*group).member[j as usize].number as usize]
                                        as *mut gentity_t;
                                    ST_TransferMoveGoal(ctx, NPC, blocker);
                                    break;
                                }
                            }
                        }
                        // we don't need to do anything else
                        continue;
                    }
                } else {
                    // okay no-one is running, use some tactics
                    if (*NPCInfo).combatPoint != -1 {
                        // we have a combat point we're supposed to be running to
                        if (*NPCInfo).squadState != SQUAD_SCOUT
                            && (*NPCInfo).squadState != SQUAD_TRANSITION
                            && (*NPCInfo).squadState != SQUAD_RETREAT
                        {
                            // but we're not running
                            if TIMER_Done(ctx, NPC, c"verifyCP".as_ptr()) != 0 {
                                // 1 - 3 seconds have passed since you chose a CP,
                                // see if you're there since, for some reason,
                                // you've stopped running...
                                if DistanceSquared(
                                    (*NPC).r.currentOrigin,
                                    world.level.combatPoints[(*NPCInfo).combatPoint as usize]
                                        .origin,
                                ) > 64.0 * 64.0
                                {
                                    // uh, WTF, we're not on our combat point? er,
                                    // try again, I guess?
                                    cp = (*NPCInfo).combatPoint;
                                    cpFlags |= ST_GetCPFlags(ctx);
                                }
                            }
                        }
                    }
                    if enemyLost != qfalse {
                        // if no-one has seen the enemy for a while, send a scout —
                        // ask where he went
                        if (*group).numState[SQUAD_SCOUT as usize] <= 0 {
                            NPC_ST_StoreMovementSpeech(ctx, SPEECH_CHASE, 0.0);
                        }
                        // Since no-one else has done this, I should be the closest
                        // one, so go after him...
                        ST_TrackEnemy(ctx, NPC, (*group).enemyLastSeenPos);
                        // set me into scout mode
                        AI_GroupUpdateSquadstates(group, NPC, SQUAD_SCOUT);
                        // we're not using a cp, so we need to set runner to true
                        // right here
                        runner = qtrue;
                    } else if enemyProtected != qfalse {
                        // if no-one has a clear shot at the enemy, someone should
                        // go after him. FIXME: if I'm in an area where no safe
                        // combat points have a clear shot at me, they don't come
                        // after me... (Raven comment). ALSO: seem to give up when
                        // behind an area portal? (Raven comment). since no-one
                        // else here has done this, I should be the closest one
                        if TIMER_Done(ctx, NPC, c"roamTime".as_ptr()) != 0
                            && (*ctx.world).bg_state.rng.Q_irand(0, (*group).numGroup) == 0
                        {
                            // only do this if we're ready to move again and we
                            // feel like it
                            cpFlags |= ST_ApproachEnemy(ctx, NPC);
                            // set me into scout mode
                            AI_GroupUpdateSquadstates(group, NPC, SQUAD_SCOUT);
                        }
                    } else {
                        // group can see and has been shooting at the enemy — see
                        // if we should do something fancy?
                        {
                            // we're ready to move
                            if (*NPCInfo).combatPoint == -1 {
                                // we're not on a combat point
                                // if ( 1 )//!(*ctx.world).bg_state.rng.Q_irand( 0, 2 ) ) (Raven, always true).
                                {
                                    // we should go for a combat point
                                    cpFlags |= ST_GetCPFlags(ctx);
                                }
                            } else if TIMER_Done(ctx, NPC, c"roamTime".as_ptr()) != 0 {
                                // we are already on a combat point
                                if i == 0 {
                                    // we're the closest
                                    if ((*group).morale - (*group).numGroup > 0)
                                        && (*ctx.world).bg_state.rng.Q_irand(0, 4) == 0
                                    {
                                        // try to outflank him
                                        cpFlags |=
                                            CP_CLEAR | CP_COVER | CP_FLANK | CP_APPROACH_ENEMY;
                                    } else if (*group).morale - (*group).numGroup < 0 {
                                        // better move!
                                        cpFlags |= ST_GetCPFlags(ctx);
                                    } else {
                                        // If we're point, then get down
                                        TIMER_Set(
                                            ctx,
                                            NPC,
                                            c"roamTime".as_ptr(),
                                            (*ctx.world).bg_state.rng.Q_irand(2000, 5000),
                                        );
                                        TIMER_Set(
                                            ctx,
                                            NPC,
                                            c"stick".as_ptr(),
                                            (*ctx.world).bg_state.rng.Q_irand(2000, 5000),
                                        );
                                        // FIXME: what if we can't shoot from a
                                        // ducked pos? (Raven comment).
                                        TIMER_Set(
                                            ctx,
                                            NPC,
                                            c"duck".as_ptr(),
                                            (*ctx.world).bg_state.rng.Q_irand(3000, 4000),
                                        );
                                        AI_GroupUpdateSquadstates(group, NPC, SQUAD_POINT);
                                    }
                                } else if i == (*group).numGroup - 1 {
                                    // farthest from the enemy
                                    if (*group).morale - (*group).numGroup < 0 {
                                        // low morale, just hang here
                                        TIMER_Set(
                                            ctx,
                                            NPC,
                                            c"roamTime".as_ptr(),
                                            (*ctx.world).bg_state.rng.Q_irand(2000, 5000),
                                        );
                                        TIMER_Set(
                                            ctx,
                                            NPC,
                                            c"stick".as_ptr(),
                                            (*ctx.world).bg_state.rng.Q_irand(2000, 5000),
                                        );
                                    } else if (*group).morale - (*group).numGroup > 0 {
                                        // try to move in on the enemy
                                        cpFlags |= ST_ApproachEnemy(ctx, NPC);
                                        // set me into scout mode
                                        AI_GroupUpdateSquadstates(group, NPC, SQUAD_SCOUT);
                                    } else {
                                        // use normal decision making process
                                        cpFlags |= ST_GetCPFlags(ctx);
                                    }
                                } else {
                                    // someone in-between
                                    if ((*group).morale - (*group).numGroup < 0)
                                        || (*ctx.world).bg_state.rng.Q_irand(0, 4) == 0
                                    {
                                        // do something
                                        cpFlags |= ST_GetCPFlags(ctx);
                                    } else {
                                        TIMER_Set(
                                            ctx,
                                            NPC,
                                            c"stick".as_ptr(),
                                            (*ctx.world).bg_state.rng.Q_irand(2000, 4000),
                                        );
                                        TIMER_Set(
                                            ctx,
                                            NPC,
                                            c"roamTime".as_ptr(),
                                            (*ctx.world).bg_state.rng.Q_irand(2000, 4000),
                                        );
                                    }
                                }
                            }
                        }
                        if cpFlags == 0 {
                            // still not moving — see if we should say something?
                            /*
                            if ( NPC->attackDebounceTime < level.time - 2000 )
                            {
                                ST_Speech( NPC, SPEECH_CHARGE, 0.9f32 );
                            }
                            */
                            // see if we should do other fun stuff — toy with ducking
                            if TIMER_Done(ctx, NPC, c"duck".as_ptr()) != 0 {
                                // not ducking
                                if TIMER_Done(ctx, NPC, c"stand".as_ptr()) != 0 {
                                    // don't have to keep standing
                                    if (*NPCInfo).combatPoint == -1
                                        || (world.level.combatPoints
                                            [(*NPCInfo).combatPoint as usize]
                                            .flags
                                            & CPF_DUCK)
                                            != 0
                                    {
                                        // okay to duck here
                                        if (*ctx.world).bg_state.rng.Q_irand(0, 3) == 0 {
                                            TIMER_Set(
                                                ctx,
                                                NPC,
                                                c"duck".as_ptr(),
                                                (*ctx.world).bg_state.rng.Q_irand(1000, 3000),
                                            );
                                        }
                                    }
                                }
                            }
                            // FIXME: what about CPF_LEAN? (Raven comment).
                        }
                    }
                }
            }

            // clear the local state
            (*NPCInfo).localState = LSTATE_NONE;

            if ((*NPCInfo).scriptFlags & SCF_USE_CP_NEAREST) != 0 {
                cpFlags &= !(CP_FLANK | CP_APPROACH_ENEMY | CP_CLOSEST);
                cpFlags |= CP_NEAREST;
            }
            // Assign combat points
            if cpFlags != 0 {
                // we want to run to a combat point
                /*
                if ( NPCInfo->combatPoint != -1 )
                {
                    cpFlags |= CP_AVOID;
                }
                */
                let enemyClient = (*(*group).enemy).client as *mut gclient_t;
                if (*enemyClient).ps.weapon == WP_SABER && (*enemyClient).ps.saberHolstered == 0 {
                    // we obviously want to avoid the enemy if he has a saber
                    cpFlags |= CP_AVOID_ENEMY;
                    avoidDist = 256.0;
                }

                // remember what we *wanted* to do...
                let cpFlags_org = cpFlags;

                // now get a combat point
                if cp == -1 {
                    // may have had some set above
                    cp = NPC_FindCombatPoint(
                        ctx,
                        (*NPC).r.currentOrigin,
                        (*NPC).r.currentOrigin,
                        (*(*group).enemy).r.currentOrigin,
                        cpFlags | CP_HAS_ROUTE,
                        avoidDist,
                        (*NPCInfo).lastFailedCombatPoint,
                    );
                }
                while cp == -1 && cpFlags != CP_ANY {
                    // start "OR"ing out certain flags to see if we can find *any*
                    // point
                    if (cpFlags & CP_INVESTIGATE) != 0 {
                        cpFlags &= !CP_INVESTIGATE;
                    } else if (cpFlags & CP_SQUAD) != 0 {
                        cpFlags &= !CP_SQUAD;
                    } else if (cpFlags & CP_DUCK) != 0 {
                        cpFlags &= !CP_DUCK;
                    } else if (cpFlags & CP_NEAREST) != 0 {
                        cpFlags &= !CP_NEAREST;
                    } else if (cpFlags & CP_FLANK) != 0 {
                        cpFlags &= !CP_FLANK;
                    } else if (cpFlags & CP_SAFE) != 0 {
                        cpFlags &= !CP_SAFE;
                    } else if (cpFlags & CP_CLOSEST) != 0 {
                        // don't need closest one to me — but let's try to
                        // approach at least
                        cpFlags &= !CP_CLOSEST;
                        cpFlags |= CP_APPROACH_ENEMY;
                    } else if (cpFlags & CP_APPROACH_ENEMY) != 0 {
                        cpFlags &= !CP_APPROACH_ENEMY;
                    } else if (cpFlags & CP_COVER) != 0 {
                        // don't need cover — but let's pick one that makes us duck
                        cpFlags &= !CP_COVER;
                        cpFlags |= CP_DUCK;
                    } else if (cpFlags & CP_CLEAR) != 0 {
                        cpFlags &= !CP_CLEAR;
                    } else if (cpFlags & CP_AVOID_ENEMY) != 0 {
                        cpFlags &= !CP_AVOID_ENEMY;
                    } else if (cpFlags & CP_RETREAT) != 0 {
                        cpFlags &= !CP_RETREAT;
                    } else if (cpFlags & CP_FLEE) != 0 {
                        // don't need to flee — but at least avoid enemy and pick
                        // one that gives cover
                        cpFlags &= !CP_FLEE;
                        cpFlags |= CP_COVER | CP_AVOID_ENEMY;
                    } else if (cpFlags & CP_AVOID) != 0 {
                        // okay, even pick one right by me
                        cpFlags &= !CP_AVOID;
                    } else {
                        cpFlags = CP_ANY;
                    }
                    // now try again
                    cp = NPC_FindCombatPoint(
                        ctx,
                        (*NPC).r.currentOrigin,
                        (*NPC).r.currentOrigin,
                        (*(*group).enemy).r.currentOrigin,
                        cpFlags | CP_HAS_ROUTE,
                        avoidDist,
                        -1,
                    );
                }
                // see if we got a valid one
                if cp != -1 {
                    // found a combat point — let others know that someone is now
                    // running
                    runner = qtrue;
                    // don't change course again until we get to where we're going
                    TIMER_Set(ctx, NPC, c"roamTime".as_ptr(), Q3_INFINITE);
                    TIMER_Set(
                        ctx,
                        NPC,
                        c"verifyCP".as_ptr(),
                        (*ctx.world).bg_state.rng.Q_irand(1000, 3000),
                    );
                    NPC_SetCombatPoint(ctx, cp);
                    NPC_SetMoveGoal(
                        ctx,
                        NPC,
                        world.level.combatPoints[cp as usize].origin,
                        8,
                        qtrue,
                        cp,
                        core::ptr::null_mut(),
                    );
                    // okay, try a move right now to see if we can even get there
                    // if ( ST_Move() ) (Raven, commented out).
                    {
                        // we actually can get to it, so okay to say you're going
                        // there. FIXME: Hmm... any way we can store this move
                        // info... (Raven comment). set us up so others know
                        // we're on the move
                        if squadState != SQUAD_IDLE {
                            AI_GroupUpdateSquadstates(group, NPC, squadState);
                        } else if (cpFlags & CP_FLEE) != 0 {
                            // outright running for your life
                            AI_GroupUpdateSquadstates(group, NPC, SQUAD_RETREAT);
                        } else {
                            // any other kind of transition between combat points
                            AI_GroupUpdateSquadstates(group, NPC, SQUAD_TRANSITION);
                        }

                        // unless we're trying to flee, walk slowly
                        if (cpFlags_org & CP_FLEE) == 0 {
                            // ucmd.buttons |= BUTTON_CAREFUL; (Raven, commented out).
                        }

                        /*
                        if ( scouting )
                        {
                            ST_Speech( NPC, SPEECH_CHASE, 0.0f32 );
                        }
                        //flanking:
                        else */
                        if (cpFlags & CP_FLANK) != 0 {
                            if (*group).numGroup > 1 {
                                NPC_ST_StoreMovementSpeech(ctx, SPEECH_OUTFLANK, -1.0);
                            }
                        } else {
                            // okay, let's cheat
                            if (*group).numGroup > 1 {
                                let mut dot: f32 = 1.0;
                                if (*ctx.world).bg_state.rng.Q_irand(0, 3) == 0 {
                                    // 25% of the time, see if we're flanking the enemy
                                    let mut eDir2Me: vec3_t = [
                                        (*NPC).r.currentOrigin[0]
                                            - (*(*group).enemy).r.currentOrigin[0],
                                        (*NPC).r.currentOrigin[1]
                                            - (*(*group).enemy).r.currentOrigin[1],
                                        (*NPC).r.currentOrigin[2]
                                            - (*(*group).enemy).r.currentOrigin[2],
                                    ];
                                    VectorNormalize(&mut eDir2Me);

                                    let mut eDir2CP: vec3_t = [
                                        world.level.combatPoints[(*NPCInfo).combatPoint as usize]
                                            .origin[0]
                                            - (*(*group).enemy).r.currentOrigin[0],
                                        world.level.combatPoints[(*NPCInfo).combatPoint as usize]
                                            .origin[1]
                                            - (*(*group).enemy).r.currentOrigin[1],
                                        world.level.combatPoints[(*NPCInfo).combatPoint as usize]
                                            .origin[2]
                                            - (*(*group).enemy).r.currentOrigin[2],
                                    ];
                                    VectorNormalize(&mut eDir2CP);

                                    dot = eDir2Me[0] * eDir2CP[0]
                                        + eDir2Me[1] * eDir2CP[1]
                                        + eDir2Me[2] * eDir2CP[2];
                                }

                                if dot < 0.4 {
                                    // flanking!
                                    NPC_ST_StoreMovementSpeech(ctx, SPEECH_OUTFLANK, -1.0);
                                } else if (*ctx.world).bg_state.rng.Q_irand(0, 10) == 0 {
                                    // regular movement
                                    NPC_ST_StoreMovementSpeech(ctx, SPEECH_YELL, 0.2);
                                    // was SPEECH_COVER (Raven comment).
                                }
                            }
                        }
                        /*
                        else if ( cpFlags & CP_CLOSEST || cpFlags & CP_APPROACH_ENEMY )
                        {
                            if ( group->numGroup > 1 )
                            {
                                NPC_ST_StoreMovementSpeech( SPEECH_CHASE, 0.4f32 );
                            }
                        }
                        */
                    } // else: nothing, a failed move should clear the combatPoint and you can try again next frame
                } else if (*NPCInfo).squadState == SQUAD_SCOUT {
                    // we couldn't find a combatPoint by the player, so just go
                    // after him directly
                    ST_HuntEnemy(ctx, NPC);
                    // set me into scout mode
                    AI_GroupUpdateSquadstates(group, NPC, SQUAD_SCOUT);
                    // AI should take care of rest
                }
            }
        }

        RestoreNPCGlobals(ctx);
    }
}

/// Raven `NPC_BSST_Attack`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:2409-2724`
pub fn NPC_BSST_Attack(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;
        let client = (*NPC).client as *mut gclient_t;

        // Don't do anything if we're hurt
        if (*NPC).painDebounceTime > world.level.time {
            NPC_UpdateAngles(ctx, qtrue, qtrue);
            return;
        }

        // NPC_CheckEnemy( qtrue, qfalse ); (Raven, commented out).
        // If we don't have an enemy, just idle
        if NPC_CheckEnemyExt(ctx, qfalse) == qfalse {
            (*NPC).enemy = None;
            if (*client).playerTeam == NPCTEAM_PLAYER {
                NPC_BSPatrol(ctx);
            } else {
                NPC_BSST_Patrol(ctx); // FIXME: or patrol? (Raven comment).
            }
            return;
        }

        // FIXME: put some sort of delay into the guys depending on how they saw
        // you...? (Raven comment).

        // Get our group info
        if TIMER_Done(ctx, NPC, c"interrogating".as_ptr()) != 0 {
            AI_GetGroup(ctx, NPC); // , 45, 512, NPC->enemy ); (Raven, commented out).
        } else {
            // FIXME: when done interrogating, I should send out a team alert! (Raven comment).
        }

        if !(*NPCInfo).group.is_null() {
            // I belong to a squad of guys - we should *always* have a group
            if (*(*NPCInfo).group).processed == qfalse {
                // I'm the first ent in my group, I'll make the command decisions
                ST_Commander(ctx);
            }
        } else if TIMER_Done(ctx, NPC, c"flee".as_ptr()) != 0 {
            let alert = NPC_CheckAlertEvents(ctx, qtrue, qtrue, -1, qfalse, AEL_DANGER as c_int);
            if NPC_CheckForDanger(ctx, alert) != qfalse {
                // not already fleeing, and going to run
                ST_Speech(ctx, NPC, SPEECH_COVER, 0.0);
                NPC_UpdateAngles(ctx, qtrue, qtrue);
                return;
            }
        }

        if (*NPC).enemy == None {
            // WTF? somehow we lost our enemy?
            NPC_BSST_Patrol(ctx); // FIXME: or patrol? (Raven comment).
            return;
        }
        let enemy = ent_resolve_opt(ctx, (*NPC).enemy);

        world.globals.enemyLOS = qfalse;
        world.globals.enemyCS = qfalse;
        world.globals.enemyInFOV = qfalse;
        world.globals.r#move = qtrue;
        world.globals.faceEnemy = qfalse;
        world.globals.shoot = qfalse;
        world.globals.hitAlly = qfalse;
        world.globals.impactPos = [0.0, 0.0, 0.0];
        world.globals.enemyDist = DistanceSquared((*NPC).r.currentOrigin, (*enemy).r.currentOrigin);

        let mut enemyDir: vec3_t = [
            (*enemy).r.currentOrigin[0] - (*NPC).r.currentOrigin[0],
            (*enemy).r.currentOrigin[1] - (*NPC).r.currentOrigin[1],
            (*enemy).r.currentOrigin[2] - (*NPC).r.currentOrigin[2],
        ];
        VectorNormalize(&mut enemyDir);
        let mut shootDir: vec3_t = [0.0; 3];
        AngleVectors((*client).ps.viewangles, Some(&mut shootDir), None, None);
        let dot = enemyDir[0] * shootDir[0] + enemyDir[1] * shootDir[1] + enemyDir[2] * shootDir[2];
        if dot > 0.5 || (world.globals.enemyDist * (1.0 - dot)) < 10000.0 {
            // enemy is in front of me or they're very close and not behind me
            world.globals.enemyInFOV = qtrue;
        }

        if world.globals.enemyDist < MIN_ROCKET_DIST_SQUARED {
            // enemy within 128
            if ((*client).ps.weapon == WP_FLECHETTE || (*client).ps.weapon == WP_REPEATER)
                && ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) != 0
            {
                // shooting an explosive, but enemy too close, switch to primary fire
                (*NPCInfo).scriptFlags &= !SCF_ALT_FIRE;
                // FIXME: we can never go back to alt-fire this way... (Raven comment).
            }
        } else if world.globals.enemyDist > 65536.0 {
            // 256 squared
            if (*client).ps.weapon == WP_DISRUPTOR {
                // sniping... should be assumed
                if ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) == 0 {
                    // use primary fire
                    (*NPCInfo).scriptFlags |= SCF_ALT_FIRE;
                    // reset fire-timing variables
                    NPC_ChangeWeapon(WP_DISRUPTOR);
                    NPC_UpdateAngles(ctx, qtrue, qtrue);
                    return;
                }
            }
        }

        // can we see our target?
        if NPC_ClearLOS4(ctx, enemy) != qfalse {
            AI_GroupUpdateEnemyLastSeen(ctx, (*NPCInfo).group, (*enemy).r.currentOrigin);
            (*NPCInfo).enemyLastSeenTime = world.level.time;
            world.globals.enemyLOS = qtrue;

            if (*client).ps.weapon == WP_NONE {
                world.globals.enemyCS = qfalse; // not true, but should stop us from firing
                NPC_AimAdjust(ctx, -1); // adjust aim worse longer we have no weapon
            } else {
                // can we shoot our target?
                if ((*client).ps.weapon == WP_ROCKET_LAUNCHER
                    || ((*client).ps.weapon == WP_FLECHETTE
                        && ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) != 0))
                    && world.globals.enemyDist < MIN_ROCKET_DIST_SQUARED
                {
                    world.globals.hitAlly = qtrue; // us! // FIXME: if too close, run away! (Raven comment).
                } else if world.globals.enemyInFOV != qfalse {
                    // if enemy is FOV, go ahead and check for shooting
                    let mut impactPos = world.globals.impactPos;
                    let hit = NPC_ShotEntity(ctx, enemy, Some(&mut impactPos));
                    world.globals.impactPos = impactPos;
                    let hitEnt = &mut world.g_entities[hit as usize] as *mut gentity_t;
                    let hitClient = (*hitEnt).client as *mut gclient_t;

                    if hit == (*enemy).s.number
                        || (!hitEnt.is_null()
                            && !(*hitEnt).client.is_null()
                            && (*hitClient).playerTeam == (*client).enemyTeam)
                        || (!hitEnt.is_null()
                            && (*hitEnt).takedamage != qfalse
                            && (((*hitEnt).r.svFlags & SVF_GLASS_BRUSH) != 0
                                || (*hitEnt).health < 40
                                || (*NPC).s.weapon == WP_EMPLACED_GUN))
                    {
                        // can hit enemy or enemy ally or will hit glass or other
                        // minor breakable (or in emplaced gun), so shoot anyway
                        AI_GroupUpdateClearShotTime(ctx, (*NPCInfo).group);
                        world.globals.enemyCS = qtrue;
                        NPC_AimAdjust(ctx, 2); // adjust aim better longer we have clear shot at enemy
                        (*NPCInfo).enemyLastSeenLocation = (*enemy).r.currentOrigin;
                    } else {
                        // Hmm, have to get around this bastard
                        NPC_AimAdjust(ctx, 1); // adjust aim better longer we can see enemy
                        ST_ResolveBlockedShot(ctx, hit);
                        if !hitEnt.is_null()
                            && !(*hitEnt).client.is_null()
                            && (*hitClient).playerTeam == (*client).playerTeam
                        {
                            // would hit an ally, don't fire!!!
                            world.globals.hitAlly = qtrue;
                        } else {
                            // Check and see where our shot *would* hit... (Raven comment).
                        }
                    }
                } else {
                    world.globals.enemyCS = qfalse; // not true, but should stop us from firing
                }
            }
        } else if trap::InPVS(
            ctx.engine,
            GInPvsArgs::new(
                &(*enemy).r.currentOrigin as *const vec3_t,
                &(*NPC).r.currentOrigin as *const vec3_t,
            ),
        ) != 0
        {
            (*NPCInfo).enemyLastSeenTime = world.level.time;
            world.globals.faceEnemy = qtrue;
            NPC_AimAdjust(ctx, -1); // adjust aim worse longer we cannot see enemy
        }

        if (*client).ps.weapon == WP_NONE {
            world.globals.faceEnemy = qfalse;
            world.globals.shoot = qfalse;
        } else {
            if world.globals.enemyLOS != qfalse {
                // FIXME: no need to face enemy if we're moving to some other
                // goal... (Raven comment).
                world.globals.faceEnemy = qtrue;
            }
            if world.globals.enemyCS != qfalse {
                world.globals.shoot = qtrue;
            }
        }

        // Check for movement to take care of
        ST_CheckMoveState(ctx);

        // See if we should override shooting decision with any special considerations
        ST_CheckFireState(ctx);

        if world.globals.faceEnemy != qfalse {
            // face the enemy
            NPC_FaceEnemy(ctx, qtrue);
        }

        if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) == 0 {
            // not supposed to chase my enemies
            if (*NPCInfo).goalEntity == (*NPC).enemy {
                // goal is my entity, so don't move
                world.globals.r#move = qfalse;
            }
        }

        if (*client).ps.weaponTime > 0 && (*NPC).s.weapon == WP_ROCKET_LAUNCHER {
            world.globals.r#move = qfalse;
        }

        if world.globals.r#move != qfalse {
            // move toward goal
            if (*NPCInfo).goalEntity != None {
                world.globals.r#move = ST_Move(ctx);
            } else {
                world.globals.r#move = qfalse;
            }
        }

        if world.globals.r#move == qfalse {
            if TIMER_Done(ctx, NPC, c"duck".as_ptr()) == 0 {
                world.globals.ucmd.upmove = -127;
            }
            // FIXME: what about leaning? (Raven comment).
        } else {
            // stop ducking!
            TIMER_Set(ctx, NPC, c"duck".as_ptr(), -1);
        }

        if TIMER_Done(ctx, NPC, c"flee".as_ptr()) == 0 {
            // running away
            world.globals.faceEnemy = qfalse;
        }

        // FIXME: check scf_face_move_dir here? (Raven comment).

        if world.globals.faceEnemy == qfalse {
            // we want to face in the dir we're running
            if world.globals.r#move == qfalse {
                // if we haven't moved, we should look in the direction we last
                // looked?
                (*NPCInfo).lastPathAngles = (*client).ps.viewangles;
            }
            (*NPCInfo).desiredYaw = (*NPCInfo).lastPathAngles[1]; // YAW
            (*NPCInfo).desiredPitch = 0.0;
            NPC_UpdateAngles(ctx, qtrue, qtrue);
            if world.globals.r#move != qfalse {
                // don't run away and shoot
                world.globals.shoot = qfalse;
            }
        }

        if ((*NPCInfo).scriptFlags & SCF_DONT_FIRE) != 0 {
            world.globals.shoot = qfalse;
        }

        if (*NPC).enemy != None && (*enemy).enemy != None {
            if (*enemy).s.weapon == WP_SABER
                && (*ent_resolve_opt(ctx, (*enemy).enemy)).s.weapon == WP_SABER
            {
                // don't shoot at an enemy jedi who is fighting another jedi, for
                // fear of injuring one or causing rogue blaster deflections
                // (a la Obi Wan/Vader duel at end of ANH)
                world.globals.shoot = qfalse;
            }
        }
        // FIXME: don't shoot right away! (Raven comment).
        if (*client).ps.weaponTime > 0 {
            if (*NPC).s.weapon == WP_ROCKET_LAUNCHER {
                if world.globals.enemyLOS == qfalse || world.globals.enemyCS == qfalse {
                    // cancel it
                    (*client).ps.weaponTime = 0;
                } else {
                    // delay our next attempt
                    TIMER_Set(
                        ctx,
                        NPC,
                        c"attackDelay".as_ptr(),
                        (*ctx.world).bg_state.rng.Q_irand(3000, 5000),
                    );
                }
            }
        } else if world.globals.shoot != qfalse {
            // try to shoot if it's time
            if TIMER_Done(ctx, NPC, c"attackDelay".as_ptr()) != 0 {
                if ((*NPCInfo).scriptFlags & SCF_FIRE_WEAPON) == 0 {
                    // we've already fired, no need to do it again here
                    WeaponThink(ctx, qtrue);
                }
                // NASTY
                if (*NPC).s.weapon == WP_ROCKET_LAUNCHER
                    && (world.globals.ucmd.buttons & BUTTON_ATTACK) != 0
                    && world.globals.r#move == qfalse
                    && world.cvars.g_spskill.integer > 1
                    && (*ctx.world).bg_state.rng.Q_irand(0, 3) == 0
                {
                    // every now and then, shoot a homing rocket
                    world.globals.ucmd.buttons &= !BUTTON_ATTACK;
                    world.globals.ucmd.buttons |= BUTTON_ALT_ATTACK;
                    (*client).ps.weaponTime = (*ctx.world).bg_state.rng.Q_irand(1000, 2500);
                }
            }
        }
    }
}

/// Raven `NPC_BSST_Default`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:2726-2742`
pub fn NPC_BSST_Default(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC as *mut gentity_t;
        let NPCInfo = world.globals.NPCInfo as *mut gNPC_t;

        if ((*NPCInfo).scriptFlags & SCF_FIRE_WEAPON) != 0 {
            WeaponThink(ctx, qtrue);
        }

        if (*NPC).enemy == None {
            // don't have an enemy, look for one
            NPC_BSST_Patrol(ctx);
        } else {
            // have an enemy
            NPC_CheckGetNewWeapon(ctx);
            NPC_BSST_Attack(ctx);
        }
    }
}
