//! MP `bg_public.h` saber move animation data.
//!
//! Type definition source: `oracle/codemp/game/bg_public.h:1496-1508`

#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int, c_uint};

use mp_qshared::shared::qboolean;

use super::saber_move_name::saberMoveName_t;

/// Raven `saberMoveData_t` — per-move animation and chaining data for a saber move.
///
/// Type definition source: `oracle/codemp/game/bg_public.h:1496-1508`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct saberMoveData_t {
    pub name: *mut c_char,
    pub animToUse: c_int,
    pub startQuad: c_int,
    pub endQuad: c_int,
    pub animSetFlags: c_uint,
    pub blendTime: c_int,
    pub blocking: c_int,
    /// What move to call if the attack button is not pressed at the end of this anim
    pub chain_idle: saberMoveName_t,
    /// What move to call if the attack button (and nothing else) is pressed
    pub chain_attack: saberMoveName_t,
    pub trailLength: qboolean,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<saberMoveData_t>() == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(saberMoveData_t, name) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(saberMoveData_t, animToUse) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(saberMoveData_t, startQuad) == 12);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(saberMoveData_t, endQuad) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(saberMoveData_t, animSetFlags) == 20);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(saberMoveData_t, blendTime) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(saberMoveData_t, blocking) == 28);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(saberMoveData_t, chain_idle) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(saberMoveData_t, chain_attack) == 36);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(saberMoveData_t, trailLength) == 40);
