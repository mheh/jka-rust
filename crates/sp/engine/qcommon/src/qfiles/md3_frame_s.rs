#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use sp_qshared::shared::vec3_t;

/// Raven `md3Frame_t` — MD3 model animation frame.
///
/// Type definition source: `oracle/oracle/code/qcommon/../qcommon/qfiles.h:106-111`
#[repr(C)]
pub struct md3Frame_t {
	pub bounds: [vec3_t; 2],
	pub localOrigin: vec3_t,
	pub radius: f32,
	pub name: [c_char; 16],
}

pub type md3Frame_s = md3Frame_t;

const _: () = assert!(core::mem::size_of::<md3Frame_t>() == 56);
const _: () = assert!(core::mem::offset_of!(md3Frame_t, bounds) == 0);
const _: () = assert!(core::mem::offset_of!(md3Frame_t, localOrigin) == 24);
const _: () = assert!(core::mem::offset_of!(md3Frame_t, radius) == 36);
const _: () = assert!(core::mem::offset_of!(md3Frame_t, name) == 40);
