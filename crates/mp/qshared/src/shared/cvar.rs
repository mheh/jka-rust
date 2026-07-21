//! Shared Raven cvar mirror types.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_float, c_int};
use core::num::NonZeroU32;

/// Raven `cvarHandle_t`.
///
/// Type definition source: `oracle/code/game/q_shared.h:1325`
/// Type definition source: `oracle/codemp/game/q_shared.h:1820`
pub type cvarHandle_t = c_int;

pub const MAX_CVAR_VALUE_STRING: usize = 256;

// `q_shared.h` `CVAR_*` registration bit flags (the `Cvar_Get`/`Cvar_Register`
// flags arg). Plain `#define` bit flags (not an enum), so §C8 makes them
// `const`s directly. Canonical qshared home so the engine crates (qcommon,
// server, client) can reach them without depending on `mp_game`.
// Source: `oracle/codemp/game/q_shared.h:1782-1799`

/// Raven `CVAR_ARCHIVE` — set to cause it to be saved to vars.rc.
pub const CVAR_ARCHIVE: c_int = 0x0000_0001;
/// Raven `CVAR_USERINFO` — sent to server on connect or change.
pub const CVAR_USERINFO: c_int = 0x0000_0002;
/// Raven `CVAR_SERVERINFO` — sent in response to front end requests.
pub const CVAR_SERVERINFO: c_int = 0x0000_0004;
/// Raven `CVAR_SYSTEMINFO` — these cvars will be duplicated on all clients.
pub const CVAR_SYSTEMINFO: c_int = 0x0000_0008;
/// Raven `CVAR_INIT` — no console change, but settable from the command line.
pub const CVAR_INIT: c_int = 0x0000_0010;
/// Raven `CVAR_LATCH` — changes only when C code next does a `Cvar_Get()`.
pub const CVAR_LATCH: c_int = 0x0000_0020;
/// Raven `CVAR_ROM` — display only, cannot be set by user (settable by code).
pub const CVAR_ROM: c_int = 0x0000_0040;
/// Raven `CVAR_USER_CREATED` — created by a set command.
pub const CVAR_USER_CREATED: c_int = 0x0000_0080;
/// Raven `CVAR_TEMP` — settable even when cheats are disabled, not archived.
pub const CVAR_TEMP: c_int = 0x0000_0100;
/// Raven `CVAR_CHEAT` — can not be changed if cheats are disabled.
pub const CVAR_CHEAT: c_int = 0x0000_0200;
/// Raven `CVAR_NORESTART` — do not clear when a cvar_restart is issued.
pub const CVAR_NORESTART: c_int = 0x0000_0400;
/// Raven `CVAR_INTERNAL` — cvar won't be displayed, ever (for passwords and such).
pub const CVAR_INTERNAL: c_int = 0x0000_0800;

/// Raven `cvar_t` — the engine-side cvar registry node.
///
/// Raven comment: "nothing outside the Cvar_*() functions should modify these
/// fields!".
///
/// Engine-internal only — modules receive `vmCvar_t` copies, never this struct
/// — so the C layout is dropped (string-data migration, DEC-32): strings are
/// owned, and the `next`/`hashNext` intrusive chains live as the
/// `Common.cvar_vars` order list over the `Common.cvar_indexes` slot arena.
/// Type definition source: `oracle/codemp/game/q_shared.h:1804-1816`
pub struct cvar_t {
    pub name: String,
    pub string: String,
    /// cvar_restart will reset to this value.
    pub resetString: String,
    /// for CVAR_LATCH vars. `None` = Raven's null (no latched value pending).
    pub latchedString: Option<String>,
    pub flags: c_int,
    /// set each time the cvar is changed.
    pub modified: bool,
    /// incremented each time the cvar is changed.
    pub modificationCount: c_int,
    /// atof( string ).
    pub value: c_float,
    /// atoi( string ).
    pub integer: c_int,
}

/// Engine-internal handle to a `Common.cvar_indexes` slot — replaces Raven's
/// cached file-scope `cvar_t*` globals (§B5 index-not-pointer). The slot index
/// is Raven's `cvarHandle_t` (minted by `Cvar_Register`), so numbering stays
/// oracle-identical.
///
/// Stored as slot+1 in a `NonZeroU32` so `Option<CvarHandle>`'s all-zero bytes
/// are `None` (Raven's null pointer) inside the zero-allocated `Engine`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CvarHandle(NonZeroU32);

impl CvarHandle {
    pub fn from_slot(slot: usize) -> CvarHandle {
        CvarHandle(NonZeroU32::new(slot as u32 + 1).expect("cvar slot overflow"))
    }

    /// The `Common.cvar_indexes` index — Raven's `cvarHandle_t` value.
    pub fn slot(self) -> usize {
        (self.0.get() - 1) as usize
    }
}

/// Raven `vmCvar_t` (canonical: `native_types`).
pub use native_types::vmCvar_t;
