//! SP `usercmd_t` copied from Raven `code/game/q_shared.h`.
//!
//! Source: `oracle/oracle/code/game/q_shared.h:2406-2415`

#![allow(non_camel_case_types)]

use core::ffi::{c_int, c_schar};

use crate::shared::platform::BYTE;

/// Raven `usercmd_t`.
///
/// Raven comment: `usercmd_t is sent to the server each client frame`
/// Raven comment: `!!!!!!!!!! LOADSAVE-affecting structure !!!!!!!!!!`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct usercmd_t {
    pub server_time: c_int,
    pub buttons: c_int,
    pub weapon: BYTE,
    pub angles: [c_int; 3],
    pub generic_cmd: BYTE,
    pub forwardmove: c_schar,
    pub rightmove: c_schar,
    pub upmove: c_schar,
}
