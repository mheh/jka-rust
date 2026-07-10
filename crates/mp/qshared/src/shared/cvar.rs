//! Shared Raven cvar mirror types.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_char, c_float, c_int};
use native_types::qboolean;

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

/// Raven `cvar_t` — the engine-side cvar registry node.
///
/// Raven comment: "nothing outside the Cvar_*() functions should modify these
/// fields!".
///
/// Type definition source: `oracle/codemp/game/q_shared.h:1804-1816`
#[repr(C)]
pub struct cvar_t {
    pub name: *mut c_char,
    pub string: *mut c_char,
    /// cvar_restart will reset to this value.
    pub resetString: *mut c_char,
    /// for CVAR_LATCH vars.
    pub latchedString: *mut c_char,
    pub flags: c_int,
    /// set each time the cvar is changed.
    pub modified: qboolean,
    /// incremented each time the cvar is changed.
    pub modificationCount: c_int,
    /// atof( string ).
    pub value: c_float,
    /// atoi( string ).
    pub integer: c_int,
    pub next: *mut cvar_t,
    pub hashNext: *mut cvar_t,
}

/// Raven `cvar_s` tag alias (`cvar_t`'s C struct tag).
pub type cvar_s = cvar_t;

const _: () = assert!(core::mem::offset_of!(cvar_t, name) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cvar_t, string) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cvar_t, resetString) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cvar_t, latchedString) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cvar_t, flags) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cvar_t, modified) == 36);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cvar_t, modificationCount) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cvar_t, value) == 44);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cvar_t, integer) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cvar_t, next) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cvar_t, hashNext) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<cvar_t>() == 72);

/// Raven `vmCvar_t`.
///
/// Raven comment: "the modules that run in the virtual machine can't access
/// the cvar_t directly, so they must ask for structured updates".
///
/// Type definition source: `oracle/code/game/q_shared.h:1323-1335`
/// Type definition source: `oracle/codemp/game/q_shared.h:1818-1830`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct vmCvar_t {
    pub handle: cvarHandle_t,
    pub modificationCount: c_int,
    pub value: c_float,
    pub integer: c_int,
    pub string: [c_char; MAX_CVAR_VALUE_STRING],
}

impl vmCvar_t {
    pub const fn zeroed() -> Self {
        vmCvar_t {
            handle: 0,
            modificationCount: 0,
            value: 0.0,
            integer: 0,
            string: [0; MAX_CVAR_VALUE_STRING],
        }
    }
}

impl Default for vmCvar_t {
    fn default() -> Self {
        Self::zeroed()
    }
}

const _: () = assert!(core::mem::size_of::<vmCvar_t>() == 272);
const _: () = assert!(core::mem::offset_of!(vmCvar_t, handle) == 0);
const _: () = assert!(core::mem::offset_of!(vmCvar_t, modificationCount) == 4);
const _: () = assert!(core::mem::offset_of!(vmCvar_t, value) == 8);
const _: () = assert!(core::mem::offset_of!(vmCvar_t, integer) == 12);
const _: () = assert!(core::mem::offset_of!(vmCvar_t, string) == 16);
