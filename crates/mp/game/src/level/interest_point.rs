//! MP `interestPoint_t`.
//!
//! Type definition source: `oracle/codemp/game/g_local.h:752-758`

#![allow(non_camel_case_types)]

use core::ffi::c_char;

use mp_qshared::shared::vec3_t;

/// Raven `MAX_INTEREST_POINTS`. Source: `oracle/codemp/game/g_local.h:752`
pub const MAX_INTEREST_POINTS: usize = 64;

/// Raven `interestPoint_t`. Pointer-bearing => arch-dependent.
///
/// Type definition source: `oracle/codemp/game/g_local.h:754-758`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct interestPoint_t {
    pub origin: vec3_t,
    pub target: *mut c_char,
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<interestPoint_t>() == 24);
