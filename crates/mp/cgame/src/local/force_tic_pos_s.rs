#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;
use mp_qshared::shared::qhandle_t;

/// Raven `forceTicPos_t` — screen position/size of a force-power icon, plus its shader.
///
/// Type definition source: `oracle/oracle/codemp/cgame/cg_local.h:1018-1026`
#[repr(C)]
pub struct forceTicPos_t {
	pub x: i32,
	pub y: i32,
	pub width: i32,
	pub height: i32,
	pub file: *mut c_char,
	pub tic: qhandle_t,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<forceTicPos_t>() == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(forceTicPos_t, x) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(forceTicPos_t, y) == 4);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(forceTicPos_t, width) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(forceTicPos_t, height) == 12);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(forceTicPos_t, file) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(forceTicPos_t, tic) == 24);
