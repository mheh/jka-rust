#![allow(non_camel_case_types, non_snake_case)]

/// Raven `dgrid_t` — light grid sample.
///
/// Type definition source: `oracle/codemp/qcommon/../qcommon/qfiles.h:522-528`
#[repr(C)]
pub struct dgrid_t {
	pub ambientLight: [[u8; 3]; 4],
	pub directLight: [[u8; 3]; 4],
	pub styles: [u8; 4],
	pub latLong: [u8; 2],
}

const _: () = assert!(core::mem::size_of::<dgrid_t>() == 30);
const _: () = assert!(core::mem::offset_of!(dgrid_t, ambientLight) == 0);
const _: () = assert!(core::mem::offset_of!(dgrid_t, directLight) == 12);
const _: () = assert!(core::mem::offset_of!(dgrid_t, styles) == 24);
const _: () = assert!(core::mem::offset_of!(dgrid_t, latLong) == 28);
