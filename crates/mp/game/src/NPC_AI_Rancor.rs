// PORT-COMPLETE: NPC_AI_Rancor.c 4/12
//! FAITHFUL port of `oracle/oracle/codemp/game/NPC_AI_Rancor.c`.
//!
//! Landed from the `fnskel.py` signature skeleton. 4 functions are transcribed
//! faithfully from packet + prelude alone; the remaining 12 are parked (see
//! the `PORT-ESCALATION` topics below), matching the precedent set in
//! `NPC_AI_Jedi.rs`/`NPC_AI_Stormtrooper.rs`/`NPC_AI_GalakMech.rs`: almost
//! every body in this file reaches the file-scope AI globals (`NPC`,
//! `NPCInfo`, `ucmd`, `level`, `g_entities`) or a `trap_*` seam call, and the
//! faithful context-free signatures have no channel to reach either (fork
//! ruling 1 makes the AI globals `GameWorld`/`GameContext` state, but these
//! resolved cross-file signatures are equally context-free).
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

// These define the working combat range for these suckers (`NPC_AI_Rancor.c:10-17`).
const MIN_DISTANCE: c_int = 128;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;
const MAX_DISTANCE: c_int = 1024;
const MAX_DISTANCE_SQR: c_int = MAX_DISTANCE * MAX_DISTANCE;
const LSTATE_CLEAR: c_int = 0;
const LSTATE_WAITING: c_int = 1;

/// Raven `DistanceSquared` (`static ID_INLINE`, header-inline helper; ported
/// inline here per the ruling — plain-C branch only, `_XBOX` asm branch
/// skipped).
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:1527-1532`
fn DistanceSquared(p1: vec3_t, p2: vec3_t) -> f32 {
    let v = [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]];
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

// PORT-ESCALATION(ambient-state): calls `trap_G2API_AddBolt` (needs &Engine);
// the faithful context-free signature carries no engine handle to reach the
// trap seam through.
/// Raven `Rancor_SetBolts`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:19-29`
pub fn Rancor_SetBolts(
    ctx: GameContext<'_>,self_: *mut gentity_t) {
    todo!("Port Rancor_SetBolts — parked: ambient-state")
}

/// Raven `NPC_Rancor_Precache`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:36-45`
pub fn NPC_Rancor_Precache(ctx: GameContext<'_>) {
    for i in 1..3 {
        crate::g_utils::G_SoundIndex(
            std::ffi::CString::new(format!("sound/chars/rancor/snort_{}.wav", i))
                .unwrap()
                .as_ptr(),
        );
    }
    crate::g_utils::G_SoundIndex(c"sound/chars/rancor/swipehit.wav".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/rancor/chomp.wav".as_ptr());
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC`/`NPCInfo`/`ucmd`
// ambient globals; no channel from this context-free faithful signature.
/// Raven `Rancor_Idle`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:53-63`
pub fn Rancor_Idle(ctx: GameContext<'_>) {
    todo!("Port Rancor_Idle — parked: ambient-state")
}

/// Raven `Rancor_CheckRoar`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:66-77`
// PORT-ESCALATION(unported-type): reads/returns Raven `animNumber_t`
// (`BOTH_*`/`TORSO_*`/`LEGS_*`) enumerator(s) — this ~1500-entry enum is a
// documented deferred type-port item (`docs/type-port-todo.md`), not a
// missing `use`. Left as unresolved bare identifiers, these silently
// type-check as irrefutable match-pattern bindings (always-true), which is
// a behavioral bug, not just a compile gap — parked instead.
pub fn Rancor_CheckRoar(
    ctx: GameContext<'_>,self_: *mut gentity_t) -> qboolean {
    todo!("Port Rancor_CheckRoar — parked: unported-type (animNumber_t)")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC`/`NPCInfo`/`ucmd`
// ambient globals and calls `crandom`/timer helpers keyed off the ambient
// `NPC`; no channel from this context-free faithful signature.
/// Raven `Rancor_Patrol`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:83-108`
pub fn Rancor_Patrol(ctx: GameContext<'_>) {
    todo!("Port Rancor_Patrol — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC`/`NPCInfo` ambient
// globals; no channel from this context-free faithful signature.
/// Raven `Rancor_Move`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:115-130`
pub fn Rancor_Move(
    ctx: GameContext<'_>,visible: qboolean) {
    todo!("Port Rancor_Move — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the `level.time` ambient global; no
// channel from this context-free faithful signature.
/// Raven `Rancor_DropVictim`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:140-194`
pub fn Rancor_DropVictim(
    ctx: GameContext<'_>,self_: *mut gentity_t) {
    todo!("Port Rancor_DropVictim — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC` ambient global
// (`Rancor_Swing` operates on the file-scope `NPC` pointer, not a parameter);
// no channel from this context-free faithful signature.
/// Raven `Rancor_Swing`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:196-306`
pub fn Rancor_Swing(
    ctx: GameContext<'_>,tryGrab: qboolean) {
    todo!("Port Rancor_Swing — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the `NPC` ambient global; no channel
// from this context-free faithful signature.
/// Raven `Rancor_Smash`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:308-367`
pub fn Rancor_Smash(ctx: GameContext<'_>) {
    todo!("Port Rancor_Smash — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the `NPC` ambient global; no channel
// from this context-free faithful signature.
/// Raven `Rancor_Bite`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:369-428`
pub fn Rancor_Bite(ctx: GameContext<'_>) {
    todo!("Port Rancor_Bite — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC` ambient global; no
// channel from this context-free faithful signature.
/// Raven `Rancor_Attack`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:431-614`
pub fn Rancor_Attack(
    ctx: GameContext<'_>,distance: f32, doCharge: qboolean) {
    todo!("Port Rancor_Attack — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC`/`NPCInfo` ambient
// globals; no channel from this context-free faithful signature.
/// Raven `Rancor_Combat`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:617-695`
pub fn Rancor_Combat(ctx: GameContext<'_>) {
    todo!("Port Rancor_Combat — parked: ambient-state")
}

/// Raven `NPC_Rancor_Pain`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:703-782`
// PORT-ESCALATION(unported-type): reads/returns Raven `animNumber_t`
// (`BOTH_*`/`TORSO_*`/`LEGS_*`) enumerator(s) — this ~1500-entry enum is a
// documented deferred type-port item (`docs/type-port-todo.md`), not a
// missing `use`. Left as unresolved bare identifiers, these silently
// type-check as irrefutable match-pattern bindings (always-true), which is
// a behavioral bug, not just a compile gap — parked instead.
pub fn NPC_Rancor_Pain(
    ctx: GameContext<'_>,self_: *mut gentity_t, attacker: *mut gentity_t, damage: c_int) {
    todo!("Port NPC_Rancor_Pain — parked: unported-type (animNumber_t)")
}

// PORT-ESCALATION(ambient-state): reads the `NPC` ambient global and calls
// `trap_Trace` (needs &Engine); no channel from this context-free faithful
// signature.
/// Raven `Rancor_CheckDropVictim`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:784-802`
pub fn Rancor_CheckDropVictim(ctx: GameContext<'_>) {
    todo!("Port Rancor_CheckDropVictim — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the `NPC`/`g_entities` ambient
// globals; no channel from this context-free faithful signature.
/// Raven `Rancor_Crush`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:805-821`
pub fn Rancor_Crush(ctx: GameContext<'_>) {
    todo!("Port Rancor_Crush — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC`/`NPCInfo`/`level`
// ambient globals; no channel from this context-free faithful signature.
/// Raven `NPC_BSRancor_Default`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:828-955`
pub fn NPC_BSRancor_Default(ctx: GameContext<'_>) {
    todo!("Port NPC_BSRancor_Default — parked: ambient-state")
}
