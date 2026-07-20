//! MP `q_shared.h` per-level limit constants.

#![allow(non_camel_case_types)]

/// Raven `MAX_CLIENTS` — absolute client limit (non-Xbox build).
///
/// Source: `oracle/codemp/game/q_shared.h:1985`
pub const MAX_CLIENTS: usize = 32;

/// `c_int`-typed dual of [`MAX_CLIENTS`]. Raven compares
/// `int` client numbers against `MAX_CLIENTS` in hundreds of `< MAX_CLIENTS`
/// sites; this spelling drops the `as c_int` noise at those comparisons.
///
/// Source: `oracle/codemp/game/q_shared.h:1985`
pub const MAX_CLIENTS_I32: core::ffi::c_int = MAX_CLIENTS as core::ffi::c_int;

/// Raven `MAX_GENTITIES` — the entity-array size.
///
/// Relocated to `mp_qshared` per its oracle home + workspace-architecture Tier-0
/// (was mis-placed in `mp_engine_server` by the mechanical type-port; slice-0
/// wiring task). `mp_engine_server` still carries its own copy pending dedupe.
///
/// Source: `oracle/codemp/game/q_shared.h:1996,2004`
pub const MAX_GENTITIES: usize = 1024;

/// Raven `MAX_STRING_CHARS` — max length of a string passed to `Cmd_TokenizeString`.
///
/// Source: `oracle/codemp/game/q_shared.h:380`
pub const MAX_STRING_CHARS: usize = 1024;

/// Raven `MAX_INFO_STRING`.
///
/// Source: `oracle/codemp/game/q_shared.h:384`
pub const MAX_INFO_STRING: usize = 1024;

/// Raven `BIG_INFO_STRING` — used for system info key only.
///
/// Source: `oracle/codemp/game/q_shared.h:388`
pub const BIG_INFO_STRING: usize = 8192;

use core::ffi::c_int;

/// Raven `ENTITYNUM_NONE`/`ENTITYNUM_WORLD`/`ENTITYNUM_MAX_NORMAL`.
///
/// Source: `oracle/codemp/game/q_shared.h:2014-2016`
pub const ENTITYNUM_NONE: c_int = MAX_GENTITIES as c_int - 1;
pub const ENTITYNUM_WORLD: c_int = MAX_GENTITIES as c_int - 2;
pub const ENTITYNUM_MAX_NORMAL: c_int = MAX_GENTITIES as c_int - 2;

/// Raven `MAX_NAME_LENGTH`.
///
/// Source: `oracle/codemp/game/q_shared.h:400`
pub const MAX_NAME_LENGTH: usize = 32;

/// Raven `MAX_STRING_TOKENS` — max tokens resulting from
/// `Cmd_TokenizeString`.
///
/// Source: `oracle/codemp/game/q_shared.h:381`
pub const MAX_STRING_TOKENS: usize = 1024;

/// Raven `MAX_TOKEN_CHARS` — max length of an individual token.
///
/// Source: `oracle/codemp/game/q_shared.h:382`
pub const MAX_TOKEN_CHARS: usize = 1024;

/// Raven `MAX_MODELS` — these are sent over the net as -12 bits.
///
/// Source: `oracle/codemp/game/q_shared.h:2020`
pub const MAX_MODELS: c_int = 512;

/// Raven `MAX_SOUNDS` — so they cannot be blindly increased.
///
/// Source: `oracle/codemp/game/q_shared.h:2021`
pub const MAX_SOUNDS: c_int = 256;

/// Raven `MAX_ICONS` — max registered icons you can have per map.
///
/// Source: `oracle/codemp/game/q_shared.h:2022`
pub const MAX_ICONS: c_int = 64;

/// Raven `MAX_FX` — max effects strings.
///
/// Source: `oracle/codemp/game/q_shared.h:2023`
pub const MAX_FX: c_int = 64;

/// Raven `MAX_WPARRAY_SIZE`.
///
/// Source: `oracle/codemp/game/q_shared.h:993`
pub const MAX_WPARRAY_SIZE: c_int = 4096;

/// Raven `MAX_SUB_BSP`.
///
/// Source: `oracle/codemp/game/q_shared.h:2025`
pub const MAX_SUB_BSP: c_int = 32;

/// Raven `MAX_SAY_TEXT`.
///
/// Source: `oracle/codemp/game/q_shared.h:402`
pub const MAX_SAY_TEXT: usize = 150;

/// Raven `MAX_VEH_WEAPONS` — max distinct vehicle-weapon types loadable
/// (`g_vehWeaponInfo` table size). Placed alongside the other per-level
/// limits (rather than `mp_bg::vehicles`) because existing call sites
/// (`g_weapon.rs`, `bg_vehicleLoad.rs`) already spell it `crate::shared::…`.
///
/// Source: `oracle/codemp/game/bg_vehicles.h:70`
pub const MAX_VEH_WEAPONS: usize = 16;
/// Raven `VEH_WEAPON_BASE` — first real vehicle-weapon index (0 is null/default).
///
/// Source: `oracle/codemp/game/bg_vehicles.h:71`
pub const VEH_WEAPON_BASE: c_int = 0;
/// Raven `VEH_WEAPON_NONE` — sentinel "no vehicle weapon" index.
///
/// Source: `oracle/codemp/game/bg_vehicles.h:72`
pub const VEH_WEAPON_NONE: c_int = -1;

/// Raven `GENTITYNUM_BITS` — bits needed to send an entity number over the
/// wire. Non-Xbox value (`#ifdef _XBOX` gives `9`; this project only builds
/// the non-Xbox branch, matching [`MAX_CLIENTS`]'s existing precedent).
///
/// Source: `oracle/codemp/game/q_shared.h:1992-1994`
pub const GENTITYNUM_BITS: c_int = 10;

/// Raven `SNAPFLAG_RATE_DELAYED`.
///
/// Source: `oracle/codemp/game/q_shared.h:1975`
pub const SNAPFLAG_RATE_DELAYED: c_int = 1;

/// Raven `SNAPFLAG_NOT_ACTIVE` — snapshot used during connection and for
/// zombies.
///
/// Source: `oracle/codemp/game/q_shared.h:1976`
pub const SNAPFLAG_NOT_ACTIVE: c_int = 2;

/// Raven `SNAPFLAG_SERVERCOUNT` — toggled every `map_restart` so transitions
/// can be detected.
///
/// Source: `oracle/codemp/game/q_shared.h:1977`
pub const SNAPFLAG_SERVERCOUNT: c_int = 4;

/// Raven `MAX_OSPATH` (`PATH_MAX`, 1024 on this target) — max length of a
/// filesystem pathname. The former FS `char[MAX_OSPATH]` buffers are owned
/// `String`s now; this survives as their write-site truncation bound.
///
/// Source: `oracle/codemp/game/q_shared.h:395`
pub const MAX_OSPATH: usize = 1024;
