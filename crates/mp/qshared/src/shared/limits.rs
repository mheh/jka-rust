//! MP `q_shared.h` per-level limit constants.

#![allow(non_camel_case_types)]

/// Raven `MAX_CLIENTS` — absolute client limit (non-Xbox build).
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:1985`
pub const MAX_CLIENTS: usize = 32;

/// `c_int`-typed dual of [`MAX_CLIENTS`] (ruling-21 rider). Raven compares
/// `int` client numbers against `MAX_CLIENTS` in hundreds of `< MAX_CLIENTS`
/// sites; this spelling drops the `as c_int` noise at those comparisons.
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:1985`
pub const MAX_CLIENTS_I32: core::ffi::c_int = MAX_CLIENTS as core::ffi::c_int;

/// Raven `MAX_GENTITIES` — the entity-array size.
///
/// Relocated to `mp_qshared` per its oracle home + workspace-architecture Tier-0
/// (was mis-placed in `mp_engine_server` by the mechanical type-port; slice-0
/// wiring task). `mp_engine_server` still carries its own copy pending dedupe.
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:1996,2004`
pub const MAX_GENTITIES: usize = 1024;

/// Raven `MAX_STRING_CHARS` — max length of a string passed to `Cmd_TokenizeString`.
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:380`
pub const MAX_STRING_CHARS: usize = 1024;

use core::ffi::c_int;

/// Raven `ENTITYNUM_NONE`/`ENTITYNUM_WORLD`/`ENTITYNUM_MAX_NORMAL`.
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:2014-2016`
pub const ENTITYNUM_NONE: c_int = MAX_GENTITIES as c_int - 1;
pub const ENTITYNUM_WORLD: c_int = MAX_GENTITIES as c_int - 2;
pub const ENTITYNUM_MAX_NORMAL: c_int = MAX_GENTITIES as c_int - 2;

/// Raven `MAX_VEH_WEAPONS` — max distinct vehicle-weapon types loadable
/// (`g_vehWeaponInfo` table size). Placed alongside the other per-level
/// limits (rather than `mp_bg::vehicles`) because existing call sites
/// (`g_weapon.rs`, `bg_vehicleLoad.rs`) already spell it `crate::shared::…`.
///
/// Source: `oracle/oracle/codemp/game/bg_vehicles.h:70`
pub const MAX_VEH_WEAPONS: usize = 16;
/// Raven `VEH_WEAPON_BASE` — first real vehicle-weapon index (0 is null/default).
///
/// Source: `oracle/oracle/codemp/game/bg_vehicles.h:71`
pub const VEH_WEAPON_BASE: c_int = 0;
/// Raven `VEH_WEAPON_NONE` — sentinel "no vehicle weapon" index.
///
/// Source: `oracle/oracle/codemp/game/bg_vehicles.h:72`
pub const VEH_WEAPON_NONE: c_int = -1;
