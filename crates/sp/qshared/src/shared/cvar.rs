//! Shared Raven cvar mirror types.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_char, c_float, c_int};

use crate::shared::qboolean;

/// Raven `cvarHandle_t`.
///
/// Type definition source: `oracle/code/game/q_shared.h:1325`
/// Type definition source: `oracle/codemp/game/q_shared.h:1820`
pub type cvarHandle_t = c_int;

pub const MAX_CVAR_VALUE_STRING: usize = 256;

/// Raven `cvar_t` — the engine-side cvar registry node.
///
/// Raven comment: "nothing outside the Cvar_*() functions should modify these
/// fields!". Unlike MP, the SP node has only a `next` link (no `hashNext`).
///
/// Type definition source: `oracle/code/game/q_shared.h:1310-1321`
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
const _: () = assert!(core::mem::size_of::<cvar_t>() == 64);

/// Raven `vmCvar_t` (canonical: `native_types`).
pub use native_types::vmCvar_t;
