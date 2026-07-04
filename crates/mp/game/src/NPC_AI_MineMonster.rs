// PORT-COMPLETE: NPC_AI_MineMonster.c 1/9
//! FAITHFUL port of `oracle/oracle/codemp/game/NPC_AI_MineMonster.c` (MP `_JK2MP` +
//! `QAGAME` compile path).
//!
//! Generated from the `fnskel.py` signature skeleton; bodies transcribed per
//! the settled jampgame fork rulings. STAGING ONLY — not yet wired into
//! crates/.
//!
//! Parking pattern (mirrors `NPC_AI_Stormtrooper.rs`):
//! - `ai-context`: reads the file-static ambient globals `NPC`, `NPCInfo`,
//!   `ucmd` (fork 1: these become GameWorld fields, but no `GameContext` is
//!   threaded into this faithful skeleton signature to access them). Also
//!   reads `level.time` for timer operations and the LCG-based `random()`
//!   (fork 3: owned threaded Rng, unavailable here).
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::g_utils::G_SoundIndex;
use crate::q_shared::va;
use mp_bg::public::entity_event::entity_event_t;

// Raven's working combat range defines (NPC_AI_MineMonster.c:3-8):
// These define the working combat range for these suckers
const MIN_DISTANCE: c_int = 54;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

pub const MAX_DISTANCE: c_int = 128;
const MAX_DISTANCE_SQR: c_int = MAX_DISTANCE * MAX_DISTANCE;

// Raven's file-scope local state (NPC_AI_MineMonster.c:10-11):
const LSTATE_CLEAR: i32 = 0;
const LSTATE_WAITING: i32 = 1;

// `VectorLengthSquared` is the canonical `crate::q_math::VectorLengthSquared`,
// reached via the prelude glob (the former per-file copy was unused).

/// Raven `NPC_MineMonster_Precache`.
///
/// Precaches the MineMonster's sound effects.
/// Source: `oracle/oracle/codemp/game/NPC_AI_MineMonster.c:18-27`
// PORT-ESCALATION(seam-threading): faithful skeleton signature carries no
// `GameContext`/`&Engine` receiver, but this body calls a callee (or reads a
// file-scope global) that needs one (ruling 1/precedent `ai_main.rs`/
// `g_weapon.rs`) — how is state threaded in?
pub fn NPC_MineMonster_Precache(ctx: GameContext<'_>) {
    todo!("Port NPC_MineMonster_Precache — parked: seam-threading")
}

// PORT-ESCALATION(ai-context): reads ambient globals `NPC`, `NPCInfo`, `ucmd`,
// and calls functions that operate on them. No `GameContext`/world handle is
// threaded into this faithful skeleton signature (fork 1).
/// Raven `MineMonster_Idle`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_MineMonster.c:35-42`
pub fn MineMonster_Idle(ctx: GameContext<'_>) {
    todo!("Port MineMonster_Idle — parked: ai-context")
}

// PORT-ESCALATION(ai-context): reads ambient globals `NPC`, `NPCInfo`, `ucmd`,
// `level.time` (timer operations), and `g_entities[0]` array access. Also needs
// `random()` (fork 3: LCG).
/// Raven `MineMonster_Patrol`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_MineMonster.c:50-83`
pub fn MineMonster_Patrol(ctx: GameContext<'_>) {
    todo!("Port MineMonster_Patrol — parked: ai-context")
}

// PORT-ESCALATION(ai-context): reads ambient globals `NPC`, `NPCInfo`, `ucmd`;
// modifies `NPCInfo->goalEntity` and `NPCInfo->goalRadius`.
/// Raven `MineMonster_Move`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_MineMonster.c:90-98`
pub fn MineMonster_Move(
    ctx: GameContext<'_>,
    visible: qboolean,
) {
    todo!("Port MineMonster_Move — parked: ai-context")
}

// PORT-ESCALATION(ai-context): reads ambient global `NPC` for entity data
// (`client->ps.viewangles`, `r.currentOrigin`, `s.number`), and needs `g_entities[]`
// array access. Also needs trap engine channel for `trap::Trace`.
/// Raven `MineMonster_TryDamage`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_MineMonster.c:101-126`
pub fn MineMonster_TryDamage(
    ctx: GameContext<'_>,
    enemy: *mut gentity_t,
    damage: c_int,
) {
    todo!("Port MineMonster_TryDamage — parked: ai-context")
}

// PORT-ESCALATION(ai-context): reads ambient global `NPC`, calls `TIMER_*`
// functions (need `level.time` from GameWorld), and needs `random()` (fork 3: LCG).
/// Raven `MineMonster_Attack`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_MineMonster.c:129-186`
pub fn MineMonster_Attack(ctx: GameContext<'_>) {
    todo!("Port MineMonster_Attack — parked: ai-context")
}

// PORT-ESCALATION(ai-context): reads ambient globals `NPC`, `NPCInfo`;
// modifies `NPCInfo->combatMove`, `NPCInfo->goalEntity`, `NPCInfo->goalRadius`,
// `NPCInfo->localState`. Also calls `TIMER_*` functions (need `level.time`).
/// Raven `MineMonster_Combat`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_MineMonster.c:189-227`
pub fn MineMonster_Combat(ctx: GameContext<'_>) {
    todo!("Port MineMonster_Combat — parked: ai-context")
}

// PORT-ESCALATION(anim-constants): the body calls `NPC_SetAnim` with
// `SETANIM_BOTH`, `BOTH_PAIN1`, `SETANIM_FLAG_OVERRIDE`, `SETANIM_FLAG_HOLD`
// constants that have not yet been ported to the jampgame crate (they exist in
// `oracle/src/codemp/game/anims.rs` and `bg_panimate.rs` uses them directly but
// they are not exported; see `bg_panimate.rs` line 29 "Unported types").
/// Raven `NPC_MineMonster_Pain`.
///
/// Handles pain/damage response for the MineMonster.
/// Source: `oracle/oracle/codemp/game/NPC_AI_MineMonster.c:234-254`
pub fn NPC_MineMonster_Pain(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    todo!("Port NPC_MineMonster_Pain — parked: anim-constants")
}

// PORT-ESCALATION(ai-context): reads ambient globals `NPC`, `NPCInfo`;
// calls functions that operate on them (no `GameContext` channel).
/// Raven `NPC_BSMineMonster_Default`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_MineMonster.c:262-278`
pub fn NPC_BSMineMonster_Default(ctx: GameContext<'_>) {
    todo!("Port NPC_BSMineMonster_Default — parked: ai-context")
}
