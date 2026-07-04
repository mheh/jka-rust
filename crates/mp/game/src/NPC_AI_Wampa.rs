// PORT-COMPLETE: NPC_AI_Wampa.c 1/10
//! Port of `oracle/oracle/codemp/game/NPC_AI_Wampa.c` (jampgame mega-pass).
//!
//! SPINE (fork rulings 1/4): NPC AI think-loop helper functions. Most functions
//! in this file read the implicit NPC/NPCInfo/ucmd bot-AI actor globals that
//! Raven's `ai_main.c` think-loop sets per NPC frame. The faithful skeleton
//! signatures carry no channel to reach these implicit globals (no `GameWorld`/
//! `GameContext` field for "current NPC" and no entity parameter in most cases).
//! This matches the `ai-context` precedent in `NPC_utils.rs`, `NPC_combat.rs`,
//! `NPC_AI_Jedi.rs` — parked pending resolution of how NPC-frame state is
//! threaded to these helpers (topic: `ai-context-threading`).
//!
//! PARKED (see PORT-ESCALATION markers): 10 functions. Only `NPC_Wampa_Precache`
//! is ported (accesses no implicit globals, only calls G_SoundIndex with a
//! string literal).
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::g_utils::G_SoundIndex;

// These define the working combat range for these suckers
// Source: `oracle/oracle/codemp/game/NPC_AI_Wampa.c:5-9`
const MIN_DISTANCE: c_int = 48;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;
const MAX_DISTANCE: c_int = 1024;
const MAX_DISTANCE_SQR: c_int = MAX_DISTANCE * MAX_DISTANCE;

const LSTATE_CLEAR: c_int = 0;
const LSTATE_WAITING: c_int = 1;

// PORT-ESCALATION(ai-context-threading): Reads/writes implicit
// NPC/NPCInfo/ucmd bot-AI frame globals; no signature channel to reach them.
/// Raven `Wampa_SetBolts`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Wampa.c:16-36`
pub fn Wampa_SetBolts(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    todo!("Port Wampa_SetBolts — parked: ai-context-threading")
}

/// Raven `NPC_Wampa_Precache`.
///
/// Precaches the swipe-hit sound. All growl/snort variants are commented out
/// in the oracle source (oracle/oracle/codemp/game/NPC_AI_Wampa.c:45-55).
/// Source: `oracle/oracle/codemp/game/NPC_AI_Wampa.c:43-58`
pub fn NPC_Wampa_Precache(ctx: GameContext<'_>) {
    // Only the swipe sound is live; growl/snort loops are commented out
    G_SoundIndex(b"sound/chars/rancor/swipehit.wav\0".as_ptr() as *const c_char);
}

// PORT-ESCALATION(ai-context-threading): Reads/writes implicit
// NPC/NPCInfo/ucmd bot-AI frame globals; no signature channel to reach them.
/// Raven `Wampa_Idle`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Wampa.c:66-76`
pub fn Wampa_Idle(ctx: GameContext<'_>) {
    todo!("Port Wampa_Idle — parked: ai-context-threading")
}

// PORT-ESCALATION(ai-context-threading): Reads implicit level.time global;
// no signature channel to reach it.
/// Raven `Wampa_CheckRoar`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Wampa.c:78-88`
pub fn Wampa_CheckRoar(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) -> qboolean {
    todo!("Port Wampa_CheckRoar — parked: ai-context-threading")
}

// PORT-ESCALATION(ai-context-threading): Reads/writes implicit
// NPC/NPCInfo/ucmd bot-AI frame globals; no signature channel to reach them.
/// Raven `Wampa_Patrol`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Wampa.c:94-119`
pub fn Wampa_Patrol(ctx: GameContext<'_>) {
    todo!("Port Wampa_Patrol — parked: ai-context-threading")
}

// PORT-ESCALATION(ai-context-threading): Reads/writes implicit
// NPC/NPCInfo/ucmd/enemyDist bot-AI frame globals; no signature channel to reach them.
/// Raven `Wampa_Move`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Wampa.c:126-169`
pub fn Wampa_Move(
    ctx: GameContext<'_>,
    visible: qboolean,
) {
    todo!("Port Wampa_Move — parked: ai-context-threading")
}

// PORT-ESCALATION(ai-context-threading): Reads implicit NPC/g_entities/level/vec3_origin
// bot-AI frame globals and entity arena; no signature channel to reach them.
/// Raven `Wampa_Slash`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Wampa.c:177-264`
pub fn Wampa_Slash(
    ctx: GameContext<'_>,
    boltIndex: c_int,
    backhand: qboolean,
) {
    todo!("Port Wampa_Slash — parked: ai-context-threading")
}

// PORT-ESCALATION(ai-context-threading): Reads/writes implicit
// NPC/ucmd bot-AI frame globals; no signature channel to reach them.
/// Raven `Wampa_Attack`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Wampa.c:267-341`
pub fn Wampa_Attack(
    ctx: GameContext<'_>,
    distance: f32,
    doCharge: qboolean,
) {
    todo!("Port Wampa_Attack — parked: ai-context-threading")
}

// PORT-ESCALATION(ai-context-threading): Reads/writes implicit
// NPC/NPCInfo/level/enemyDist bot-AI frame globals; no signature channel to reach them.
/// Raven `Wampa_Combat`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Wampa.c:344-425`
pub fn Wampa_Combat(ctx: GameContext<'_>) {
    todo!("Port Wampa_Combat — parked: ai-context-threading")
}

// PORT-ESCALATION(ai-context-threading): Reads implicit level.time global
// and NPC-frame state through entity pointers; no signature channel to reach level.
/// Raven `NPC_Wampa_Pain`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Wampa.c:433-499`
pub fn NPC_Wampa_Pain(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    todo!("Port NPC_Wampa_Pain — parked: ai-context-threading")
}

// PORT-ESCALATION(ai-context-threading): Reads/writes implicit
// NPC/NPCInfo/level/ucmd/enemyDist bot-AI frame globals; no signature channel to reach them.
/// Raven `NPC_BSWampa_Default`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Wampa.c:506-654`
pub fn NPC_BSWampa_Default(ctx: GameContext<'_>) {
    todo!("Port NPC_BSWampa_Default — parked: ai-context-threading")
}
