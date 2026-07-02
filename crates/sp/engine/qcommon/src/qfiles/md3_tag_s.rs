#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use sp_qshared::shared::{vec3_t, MAX_QPATH};

/// Raven `md3Tag_s` — MD3 model attachment tag.
///
/// Type definition source: `oracle/oracle/code/qcommon/../qcommon/qfiles.h:113-117`
#[repr(C)]
pub struct md3Tag_t {
	/// tag name
	pub name: [c_char; MAX_QPATH],
	pub origin: vec3_t,
	pub axis: [vec3_t; 3],
}

/// C tag name for `md3Tag_t`.
pub type md3Tag_s = md3Tag_t;

const _: () = assert!(core::mem::size_of::<md3Tag_t>() == 112);
const _: () = assert!(core::mem::offset_of!(md3Tag_t, name) == 0);
const _: () = assert!(core::mem::offset_of!(md3Tag_t, origin) == 64);
const _: () = assert!(core::mem::offset_of!(md3Tag_t, axis) == 76);
