#![allow(non_camel_case_types, non_snake_case)]

/// Raven `md3XyzNormal_t` — MD3 model per-frame vertex position/normal.
///
/// Type definition source: `oracle/oracle/code/qcommon/../qcommon/qfiles.h:163-166`
#[repr(C)]
pub struct md3XyzNormal_t {
	pub xyz: [i16; 3],
	pub normal: i16,
}

const _: () = assert!(core::mem::size_of::<md3XyzNormal_t>() == 8);
const _: () = assert!(core::mem::offset_of!(md3XyzNormal_t, xyz) == 0);
const _: () = assert!(core::mem::offset_of!(md3XyzNormal_t, normal) == 6);
