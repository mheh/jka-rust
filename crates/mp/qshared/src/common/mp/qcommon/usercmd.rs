//! MP `usercmd_t` copied from Raven `codemp/game/q_shared.h`.
//!
//! Source: `oracle/codemp/game/q_shared.h:2523-2533`

#![allow(non_camel_case_types)]

use core::ffi::{c_int, c_schar};

use crate::shared::platform::BYTE;

/// Raven `usercmd_t`.
///
/// Raven comment: `usercmd_t is sent to the server each client frame`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct usercmd_t {
    pub serverTime: c_int,
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

// All-zero is a valid `usercmd_t` (POD `#[repr(C)]`, ints/bytes only);
// needed so `GameGlobals` (mp_game) can `#[derive(Default)]` over its
// `ucmd`/`_saved_ucmd` fields (NPC.c pass-2).
impl Default for usercmd_t {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}

const _: () = assert!(core::mem::size_of::<usercmd_t>() == 28);
const _: () = assert!(core::mem::offset_of!(usercmd_t, serverTime) == 0);
const _: () = assert!(core::mem::offset_of!(usercmd_t, angles) == 4);
const _: () = assert!(core::mem::offset_of!(usercmd_t, buttons) == 16);
const _: () = assert!(core::mem::offset_of!(usercmd_t, weapon) == 20);
const _: () = assert!(core::mem::offset_of!(usercmd_t, forcesel) == 21);
const _: () = assert!(core::mem::offset_of!(usercmd_t, invensel) == 22);
const _: () = assert!(core::mem::offset_of!(usercmd_t, generic_cmd) == 23);
const _: () = assert!(core::mem::offset_of!(usercmd_t, forwardmove) == 24);
const _: () = assert!(core::mem::offset_of!(usercmd_t, rightmove) == 25);
const _: () = assert!(core::mem::offset_of!(usercmd_t, upmove) == 26);
