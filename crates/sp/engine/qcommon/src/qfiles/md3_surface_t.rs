#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use sp_qshared::shared::MAX_QPATH;

/// Raven `md3Surface_t` — MD3 model surface header.
///
/// Type definition source: `oracle/code/qcommon/../qcommon/qfiles.h:129-148`
#[repr(C)]
pub struct md3Surface_t {
	pub ident: i32,

	/// polyset name
	pub name: [c_char; MAX_QPATH],

	pub flags: i32,
	/// all surfaces in a model should have the same
	pub numFrames: i32,

	/// all surfaces in a model should have the same
	pub numShaders: i32,
	pub numVerts: i32,

	pub numTriangles: i32,
	pub ofsTriangles: i32,

	/// offset from start of md3Surface_t
	pub ofsShaders: i32,
	/// texture coords are common for all frames
	pub ofsSt: i32,
	/// numVerts * numFrames
	pub ofsXyzNormals: i32,

	/// next surface follows
	pub ofsEnd: i32,
}

const _: () = assert!(core::mem::size_of::<md3Surface_t>() == 108);
const _: () = assert!(core::mem::offset_of!(md3Surface_t, ident) == 0);
const _: () = assert!(core::mem::offset_of!(md3Surface_t, name) == 4);
const _: () = assert!(core::mem::offset_of!(md3Surface_t, flags) == 68);
const _: () = assert!(core::mem::offset_of!(md3Surface_t, numFrames) == 72);
const _: () = assert!(core::mem::offset_of!(md3Surface_t, numShaders) == 76);
const _: () = assert!(core::mem::offset_of!(md3Surface_t, numVerts) == 80);
const _: () = assert!(core::mem::offset_of!(md3Surface_t, numTriangles) == 84);
const _: () = assert!(core::mem::offset_of!(md3Surface_t, ofsTriangles) == 88);
const _: () = assert!(core::mem::offset_of!(md3Surface_t, ofsShaders) == 92);
const _: () = assert!(core::mem::offset_of!(md3Surface_t, ofsSt) == 96);
const _: () = assert!(core::mem::offset_of!(md3Surface_t, ofsXyzNormals) == 100);
const _: () = assert!(core::mem::offset_of!(md3Surface_t, ofsEnd) == 104);
