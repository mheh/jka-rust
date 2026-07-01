//! MP `clientConnected_t`.
//!
//! Source: `oracle/oracle/codemp/game/g_local.h:366`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

/// Raven `clientConnected_t` (anonymous enum + `typedef int`).
///
/// Source: `oracle/oracle/codemp/game/g_local.h:366`
pub type clientConnected_t = c_int;
pub const CON_DISCONNECTED: clientConnected_t = 0;
pub const CON_CONNECTING: clientConnected_t = 1;
pub const CON_CONNECTED: clientConnected_t = 2;
