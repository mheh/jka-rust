//! MP `usercmd_t` copied from Raven `codemp/game/q_shared.h`.
//!
//! Source: `oracle/oracle/codemp/game/q_shared.h:2523-2533`

#![allow(non_camel_case_types)]

use core::ffi::{c_int, c_schar};

use crate::shared::platform::BYTE;

/// Raven `usercmd_t`.
///
/// Raven comment: `usercmd_t is sent to the server each client frame`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct usercmd_t {
    pub server_time: c_int,
    pub angles: [c_int; 3],
    pub buttons: c_int,
    /// Raven `weapon`: weapon
    pub weapon: BYTE,
    pub forcesel: BYTE,
    pub invensel: BYTE,
    pub generic_cmd: BYTE,
    pub forwardmove: c_schar,
    pub rightmove: c_schar,
    pub upmove: c_schar,
}
