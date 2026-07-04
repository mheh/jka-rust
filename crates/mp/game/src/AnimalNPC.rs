// PORT-COMPLETE: AnimalNPC.c 2/7
//! FAITHFUL port of `oracle/oracle/codemp/game/AnimalNPC.c` (MP `_JK2MP` +
//! `QAGAME` compile path).
//!
//! Generated from the `fnskel.py` signature skeleton; bodies transcribed per
//! the settled jampgame fork rulings. STAGING ONLY — not yet wired into
//! crates/.
//!
//! Parking pattern in this file (mirrors `SpeederNPC.rs`/`g_vehicles.rs`):
//! - `raw-ptr-skeleton-no-world-handle` / `ambient-global (level.time)`: reads
//!   `level.time`, unreachable from the raw-pointer skeleton signature
//!   (rulings item 1: `level` lives on the world).
//! - `ambient-global (g_vehicleInfo)`: reads the file-static `g_vehicleInfo`
//!   table to dispatch the base vehicle-type vtable.
//! - `bg-dep (vehicleInfo_t)`: dereferences `Vehicle_t::m_pVehicleInfo`, which
//!   is still a `*mut c_void` placeholder (`//TODO: Port vehicleInfo_t`,
//!   `bg_vehicles.h:586`) pending that type's pointer-field port.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

// Raven angle-vector index (`q_shared.h`): YAW=1.
const YAW: usize = 1;

/// Raven `DeathUpdate` — update death sequence.
///
/// Source: `oracle/oracle/codemp/game/AnimalNPC.c:97-148`
// PORT-ESCALATION(raw-ptr-skeleton-no-world-handle): reads `level.time`, not
// reachable from the faithful raw-pointer skeleton signature (fork 1:
// `level` lives on `GameWorld`). Also dereferences
// `pVeh->m_pVehicleInfo->Inhabited`/`EjectAll`, still a `*mut c_void`
// placeholder (bg-dep: vehicleInfo_t).
pub extern "C" fn DeathUpdate(pVeh: *mut Vehicle_t) {
    todo!("Port DeathUpdate — parked: raw-ptr-skeleton-no-world-handle (level.time) + bg-dep (vehicleInfo_t) — oracle/oracle/codemp/game/AnimalNPC.c:97")
}

/// Raven `Update` — like a think or move command, this updates various
/// vehicle properties.
///
/// Source: `oracle/oracle/codemp/game/AnimalNPC.c:151-154`
// PORT-ESCALATION(ambient-global): reads the file-static `g_vehicleInfo`
// table (`g_vehicleInfo[VEHICLE_BASE].Update`) to dispatch the base
// vehicle-type vtable — same ambient global already parked (unresolved) in
// `bg_vehicleLoad.rs`/`SpeederNPC.rs`. Needs that global placed on
// `GameWorld` (fork 1) before this can thread it.
pub extern "C" fn Update(pVeh: *mut Vehicle_t, pUcmd: *const usercmd_t) -> qboolean {
    todo!("Port Update — parked: ambient-global (g_vehicleInfo) — oracle/oracle/codemp/game/AnimalNPC.c:151")
}

/// `ProcessMoveCommands` the Vehicle.
///
/// Raven: MP RULE - ALL PROCESSMOVECOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE!!!
/// If you really need to violate this rule for SP, then use ifdefs.
/// By BG-compatible, I mean no use of game-specific data - ONLY use
/// stuff available in the MP bgEntity.
/// Source: `oracle/oracle/codemp/game/AnimalNPC.c:168-329`
// PORT-ESCALATION(ambient-global): reads the file-static `level.time` (via
// `curTime`) with no `GameContext`/world receiver on the faithful skeleton
// signature (fork 1). Also reads `pVeh->m_pVehicleInfo->decelIdle`/`speedMax`/
// `speedIdle`/`accelIdle`/`speedMin`/`turboSpeed`/`turboRecharge`/
// `turboDuration`/`acceleration`, all through the still-unported
// `vehicleInfo_t` (bg-dep). The `#ifndef _JK2MP`/SP-only bucking-flag early
// return (lines 195-205) is dead in the `_JK2MP` build and is dropped, not
// parked (unreachable per porting-rules §10).
pub extern "C" fn ProcessMoveCommands(pVeh: *mut Vehicle_t) {
    todo!("Port ProcessMoveCommands — parked: ambient-global (level.time) + bg-dep (vehicleInfo_t) — oracle/oracle/codemp/game/AnimalNPC.c:168")
}

/// `ProcessOrientCommands` the Vehicle.
///
/// Raven: MP RULE - ALL PROCESSORIENTCOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE!!!
/// If you really need to violate this rule for SP, then use ifdefs.
/// By BG-compatible, I mean no use of game-specific data - ONLY use
/// stuff available in the MP bgEntity.
/// Source: `oracle/oracle/codemp/game/AnimalNPC.c:338-464`
// PORT-ESCALATION(bg-dep): the `_JK2MP` branch (lines 346-409, the only one
// compiled here — the `#ifndef _JK2MP` bucking-flag early return at 357-362
// is dead in the `_JK2MP` build and is dropped per porting-rules §10) reads
// `pVeh->m_pVehicleInfo->turningSpeed`/`speedMax` through the still-unported
// `vehicleInfo_t` (`Vehicle_t::m_pVehicleInfo` is a `*mut c_void`
// placeholder, `bg_vehicles.h:586`). Needs that type's pointer-field port
// settled before transcription.
pub extern "C" fn ProcessOrientCommands(pVeh: *mut Vehicle_t) {
    todo!("Port ProcessOrientCommands — parked: bg-dep (vehicleInfo_t) — oracle/oracle/codemp/game/AnimalNPC.c:338")
}

/// Raven `AnimalProcessOri` — temp hack til mp speeder controls are sorted
/// (`_JK2MP` only).
///
/// Source: `oracle/oracle/codemp/game/AnimalNPC.c:467-470`
pub fn AnimalProcessOri(pVeh: *mut Vehicle_t) {
    ProcessOrientCommands(pVeh);
}

/// Raven `AnimateVehicle`.
///
/// Source: `oracle/oracle/codemp/game/AnimalNPC.c:474-615`
// PORT-ESCALATION(raw-ptr-skeleton-no-world-handle): reads `level.time`
// (boarding-timer arithmetic), not reachable from the faithful raw-pointer
// skeleton signature (fork 1). Also dereferences
// `pVeh->m_pVehicleInfo->speedMax` through the still-unported `vehicleInfo_t`
// (bg-dep), and casts `pVeh->m_pParentEntity`/`m_pPilot` to the game-side
// `gentity_t` (client/health/legsAnim fields) rather than the MP-restricted
// `bgEntity_t` this skeleton carries.
pub extern "C" fn AnimateVehicle(pVeh: *mut Vehicle_t) {
    todo!("Port AnimateVehicle — parked: raw-ptr-skeleton-no-world-handle (level.time) + bg-dep (vehicleInfo_t) — oracle/oracle/codemp/game/AnimalNPC.c:474")
}

/// Raven `AnimateRiders` — makes sure the riders in this vehicle are
/// properly animated.
///
/// Raven: rwwFIXMEFIXME: This is all going to have to be predicted I think,
/// or it will feel awful and lagged.
/// Source: `oracle/oracle/codemp/game/AnimalNPC.c:620-849`
// PORT-ESCALATION(raw-ptr-skeleton-no-world-handle): reads `level.time`
// (turbo-timer/boarding checks), not reachable from the faithful raw-pointer
// skeleton signature (fork 1). Also dereferences
// `pVeh->m_pVehicleInfo->speedMax` through the still-unported `vehicleInfo_t`
// (bg-dep), and casts `pVeh->m_pPilot`/`m_pParentEntity` to the game-side
// `gentity_t` (client/ghoul2/enemy fields) rather than the MP-restricted
// `bgEntity_t` this skeleton carries.
pub extern "C" fn AnimateRiders(pVeh: *mut Vehicle_t) {
    todo!("Port AnimateRiders — parked: raw-ptr-skeleton-no-world-handle (level.time) + bg-dep (vehicleInfo_t) — oracle/oracle/codemp/game/AnimalNPC.c:620")
}

/// Raven `G_SetAnimalVehicleFunctions` — on the client this function will
/// only set up the process command funcs.
///
/// Raven: installs this file's vehicle-vtable functions onto a
/// `vehicleInfo_t` (ruling 7: enum-over-vehicle-type dispatch lives in the
/// caller — `bg_vehicleLoad.rs` matches on `vehicleType_t::VH_ANIMAL` — this
/// fn just fills the already-ported `Option<unsafe extern "C" fn(...)>`
/// vtable fields directly). Only the `#ifdef QAGAME` (game-side) and shared
/// assignments are live for jampgame; the `#ifndef QAGAME` (cgame
/// `AttachRidersGeneric`) arm is dead here.
/// Source: `oracle/oracle/codemp/game/AnimalNPC.c:857-887`
pub fn G_SetAnimalVehicleFunctions(pVehInfo: *mut vehicleInfo_t) {
    unsafe {
        (*pVehInfo).AnimateVehicle = Some(AnimateVehicle);
        (*pVehInfo).AnimateRiders = Some(AnimateRiders);
        (*pVehInfo).DeathUpdate = Some(DeathUpdate);
        (*pVehInfo).Update = Some(Update);

        // shared
        (*pVehInfo).ProcessMoveCommands = Some(ProcessMoveCommands);
        (*pVehInfo).ProcessOrientCommands = Some(ProcessOrientCommands);
    }
}

/// Raven `G_CreateAnimalNPC` — create/allocate a new Animal Vehicle
/// (initializing it as well).
///
/// Raven: this is a BG function too in MP so don't un-bg-compatibilify it.
/// Source: `oracle/oracle/codemp/game/AnimalNPC.c:904-925`
// PORT-ESCALATION(ambient-global): reads the file-static `g_vehicleInfo`
// table (`&g_vehicleInfo[BG_VehicleGetIndex(strAnimalType)]`) to populate
// `m_pVehicleInfo` — same ambient global parked (unresolved) elsewhere
// (`bg_vehicleLoad.rs`, `SpeederNPC.rs`); also `Vehicle_t::m_pVehicleInfo` is
// itself still a `*mut c_void` placeholder (`//TODO: Port vehicleInfo_t`,
// `bg_vehicles.h:586`) pending that type's pointer-field port. Needs both
// settled before transcription.
pub fn G_CreateAnimalNPC(pVeh: *mut *mut Vehicle_t, strAnimalType: *const c_char) {
    todo!("Port G_CreateAnimalNPC — parked: ambient-global (g_vehicleInfo) + bg-dep (vehicleInfo_t) — oracle/oracle/codemp/game/AnimalNPC.c:904")
}
