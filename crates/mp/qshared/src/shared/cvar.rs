//! Shared Raven cvar mirror types.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_char, c_float, c_int};

/// Raven `cvarHandle_t`.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:1325`
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:1820`
pub type cvarHandle_t = c_int;

pub const MAX_CVAR_VALUE_STRING: usize = 256;

/// Raven `vmCvar_t`.
///
/// Raven comment: "the modules that run in the virtual machine can't access
/// the cvar_t directly, so they must ask for structured updates".
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:1323-1335`
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:1818-1830`
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
