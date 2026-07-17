// PORT-COMPLETE: NPC_sounds.c

//! FAITHFUL port of `oracle/codemp/game/NPC_sounds.c`.
//!
//! Filled by the jampgame mega-pass; functions reach file-scope game state
//! (`level`, `g_entities`, cvars) and engine traps through the threaded
//! `GameContext`/`GameWorld` handle.
//!
//! Safe-state migration **Stage 1**: entity-pointer params are `EntityId` /
//! `Option<EntityId>` handles (§B5), not raw `gentity_t*`; ctx-free leaf helpers
//! take `&mut`/`&gentity_t`. Bodies re-derive the raw pointers verbatim at the
//! top (`// STAGE-1:` markers) — Stage-2 debt. Callers bridge at the boundary
//! via `ctx.entity_id_of(ptr)`.
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_timer::{TIMER_Done, TIMER_Set};
use crate::g_utils::G_SpeechEvent;
use crate::prelude::*;
use crate::trap;
use crate::NPC_combat::G_ClearEnemy;

/// Raven `G_AddVoiceEvent`.
///
/// Source: `oracle/codemp/game/NPC_sounds.c:23-64`
pub fn G_AddVoiceEvent(
    ctx: &mut GameContext,
    self_: EntityId,
    event: c_int,
    speakDebounceTime: c_int,
) {
    // `NPC`/`client` are raw `gNPC_t`/`gclient_t` pointer fields read by value
    // through the entity accessor; the derefs below stay `unsafe` (the gNPC_t /
    // gclient_t deref regime is deferred — safe-state task #7).
    let npc = ctx.entity(self_).NPC;
    if npc.is_null() {
        return;
    }

    let client = ctx.entity(self_).client;
    if client.is_null() || unsafe { (*client).ps.pm_type } >= PM_DEAD as c_int {
        return;
    }

    if unsafe { (*npc).blockedSpeechDebounceTime } > ctx.world.level.time {
        return;
    }

    let self_ptr = ctx.entity_mut(self_) as *mut gentity_t;
    if trap::ICARUS_TaskIDPending(
        ctx.engine,
        mp_abi::game::syscalls::G_ICARUS_TASKIDPENDING::GIcarusTaskidpendingArgs::new(
            self_ptr.cast(),
            TID_CHAN_VOICE as c_int,
        ),
    ) != qfalse
    {
        return;
    }

    if (unsafe { (*npc).scriptFlags } & SCF_NO_COMBAT_TALK) != 0
        && ((event >= EV_ANGER1 as c_int && event <= EV_VICTORY3 as c_int)
            || (event >= EV_CHASE1 as c_int && event <= EV_SUSPICIOUS5 as c_int))
    {
        return;
    }

    if (unsafe { (*npc).scriptFlags } & SCF_NO_ALERT_TALK) != 0
        && (event >= EV_GIVEUP1 as c_int && event <= EV_SUSPICIOUS5 as c_int)
    {
        return;
    }

    G_SpeechEvent(ctx, self_, event);

    let new_time = ctx.world.level.time
        + if speakDebounceTime == 0 {
            5000
        } else {
            speakDebounceTime
        };
    unsafe {
        (*npc).blockedSpeechDebounceTime = new_time;
    }
}

/// Raven `NPC_PlayConfusionSound`.
///
/// Source: `oracle/codemp/game/NPC_sounds.c:66-93`
pub fn NPC_PlayConfusionSound(ctx: &mut GameContext, self_: EntityId) {
    // `NPC`/`client` raw pointer fields read by value through the entity
    // accessor; the derefs below stay `unsafe` (gNPC_t / gclient_t deref regime
    // deferred — safe-state task #7). The `client` deref is confined to the
    // short-circuit operand, preserving Raven's evaluation order.
    let npc = ctx.entity(self_).NPC;
    let client = ctx.entity(self_).client;

    if ctx.entity(self_).health > 0 {
        if ctx.entity(self_).enemy.is_some()
            || TIMER_Done(ctx, Some(self_), cstr("enemyLastVisible").as_ptr()) == qfalse
            || unsafe { (*client).renderInfo.lookTarget } == 0
        {
            unsafe {
                (*npc).blockedSpeechDebounceTime = 0;
            }
            let event = ctx
                .world
                .bg_state
                .rng
                .Q_irand(EV_CONFUSE2 as c_int, EV_CONFUSE3 as c_int);
            G_AddVoiceEvent(ctx, self_, event, 2000);
        } else if !npc.is_null()
            && unsafe { (*npc).investigateDebounceTime + (*npc).pauseTime } > ctx.world.level.time
        {
            unsafe {
                (*npc).blockedSpeechDebounceTime = 0;
            }
            G_AddVoiceEvent(ctx, self_, EV_CONFUSE1 as c_int, 2000);
        }
    }

    TIMER_Set(ctx, Some(self_), cstr("enemyLastVisible").as_ptr(), 0);
    unsafe {
        (*npc).tempBehavior = BS_DEFAULT;
    }

    G_ClearEnemy(ctx, self_);

    unsafe {
        (*npc).investigateCount = 0;
    }
}
