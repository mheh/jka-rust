// PORT-COMPLETE: g_object.c 0/4
//! FAITHFUL signature skeleton for `oracle/oracle/codemp/game/g_object.c`.
//!
//! All 4 functions in this file read ambient game state (`level`, `g_gravity`,
//! `g_entities`) that is not reachable through the raw-pointer-only signatures
//! (no `GameWorld`/engine context parameter). This matches the precedent in
//! `g_main.rs`, `g_combat.rs`, and others where `raw-ptr-skeleton-no-world-handle`
//! escalations block porting. Once the seam decision is settled (how to thread
//! `GameWorld` through raw-pointer game logic functions), these will unblock.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

// PORT-ESCALATION(raw-ptr-skeleton-no-world-handle): reads `level` and `g_gravity` cvars; no world handle.
/// Raven `G_BounceObject`. Reflects velocity on trace plane.
///
/// Source: `oracle/oracle/codemp/game/g_object.c:14-59`
pub fn G_BounceObject(
    ent: *mut gentity_t,
    trace: *mut trace_t,
) {
    todo!("Port G_BounceObject — parked: raw-ptr-skeleton-no-world-handle")
}

// PORT-ESCALATION(raw-ptr-skeleton-no-world-handle): reads `level`, `g_gravity`, `g_entities`; calls traps (trap_LinkEntity, trap_Trace); stores fn-pointer (EntThink::G_RunObject); no world handle.
/// Raven `G_RunObject`. Main object physics simulation.
///
/// Source: `oracle/oracle/codemp/game/g_object.c:72-241`
pub fn G_RunObject(
    ent: *mut gentity_t,
) {
    todo!("Port G_RunObject — parked: raw-ptr-skeleton-no-world-handle")
}

// PORT-ESCALATION(raw-ptr-skeleton-no-world-handle): reads/writes `level` fields; no world handle.
/// Raven `G_StopObjectMoving`. Stops an object from moving.
///
/// Source: `oracle/oracle/codemp/game/g_object.c:244-258`
pub fn G_StopObjectMoving(
    object: *mut gentity_t,
) {
    todo!("Port G_StopObjectMoving — parked: raw-ptr-skeleton-no-world-handle")
}

// PORT-ESCALATION(raw-ptr-skeleton-no-world-handle): reads `level.time`; stores fn-pointer assignment (object->think = G_RunObject, ruling 2); no world handle.
/// Raven `G_StartObjectMoving`. Starts an object moving with direction and speed.
///
/// Source: `oracle/oracle/codemp/game/g_object.c:260-287`
pub fn G_StartObjectMoving(
    object: *mut gentity_t,
    dir: vec3_t,
    speed: f32,
    trType: trType_t,
) {
    todo!("Port G_StartObjectMoving — parked: raw-ptr-skeleton-no-world-handle")
}
