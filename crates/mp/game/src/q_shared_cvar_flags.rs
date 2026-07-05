//! `q_shared.h` `CVAR_*` registration bit flags (`trap_Cvar_Register` flags
//! arg).
//!
//! Plain `#define` bit flags (not an enum), so §C8 makes them `const`s
//! directly.
//!
//! Source: `oracle/oracle/codemp/game/q_shared.h:1782-1799`

use core::ffi::c_int;

/// Raven `CVAR_ARCHIVE` — set to cause it to be saved to vars.rc.
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:1782`
pub const CVAR_ARCHIVE: c_int = 0x0000_0001;

/// Raven `CVAR_USERINFO` — sent to server on connect or change.
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:1785`
pub const CVAR_USERINFO: c_int = 0x0000_0002;

/// Raven `CVAR_SERVERINFO` — sent in response to front end requests.
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:1786`
pub const CVAR_SERVERINFO: c_int = 0x0000_0004;

/// Raven `CVAR_SYSTEMINFO` — these cvars will be duplicated on all clients.
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:1787`
pub const CVAR_SYSTEMINFO: c_int = 0x0000_0008;

/// Raven `CVAR_INIT` — don't allow change from console at all, but can be
/// set from the command line.
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:1788`
pub const CVAR_INIT: c_int = 0x0000_0010;

/// Raven `CVAR_LATCH` — will only change when C code next does a `Cvar_Get()`.
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:1790`
pub const CVAR_LATCH: c_int = 0x0000_0020;

/// Raven `CVAR_ROM` — display only, cannot be set by user at all (can be set
/// by code).
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:1795`
pub const CVAR_ROM: c_int = 0x0000_0040;

/// Raven `CVAR_USER_CREATED` — created by a set command.
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:1796`
pub const CVAR_USER_CREATED: c_int = 0x0000_0080;

/// Raven `CVAR_TEMP` — can be set even when cheats are disabled, but is not
/// archived.
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:1797`
pub const CVAR_TEMP: c_int = 0x0000_0100;

/// Raven `CVAR_CHEAT` — can not be changed if cheats are disabled.
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:1798`
pub const CVAR_CHEAT: c_int = 0x0000_0200;

/// Raven `CVAR_NORESTART` — do not clear when a cvar_restart is issued.
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:1799`
pub const CVAR_NORESTART: c_int = 0x0000_0400;
