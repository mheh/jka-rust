#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use sp_qshared::shared::vec3_t;

/// Raven `interestPoint_t` — a point of interest an AI may look toward.
///
/// Type definition source: `oracle/code/game/g_local.h:84-88`
#[repr(C)]
pub struct interestPoint_t {
    pub origin: vec3_t,
    pub target: *mut c_char,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<interestPoint_t>() == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interestPoint_t, origin) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interestPoint_t, target) == 16);
