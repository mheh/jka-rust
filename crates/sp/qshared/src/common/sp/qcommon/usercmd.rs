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

const _: () = assert!(core::mem::size_of::<usercmd_t>() == 28);
const _: () = assert!(core::mem::offset_of!(usercmd_t, server_time) == 0);
const _: () = assert!(core::mem::offset_of!(usercmd_t, buttons) == 4);
const _: () = assert!(core::mem::offset_of!(usercmd_t, weapon) == 8);
const _: () = assert!(core::mem::offset_of!(usercmd_t, angles) == 12);
const _: () = assert!(core::mem::offset_of!(usercmd_t, generic_cmd) == 24);
const _: () = assert!(core::mem::offset_of!(usercmd_t, forwardmove) == 25);
const _: () = assert!(core::mem::offset_of!(usercmd_t, rightmove) == 26);
const _: () = assert!(core::mem::offset_of!(usercmd_t, upmove) == 27);
