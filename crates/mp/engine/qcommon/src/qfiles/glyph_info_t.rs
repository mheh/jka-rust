#![allow(non_camel_case_types, non_snake_case)]

/// Raven `glyphInfo_t` — font glyph metrics and texture coordinates.
///
/// Type definition source: `oracle/codemp/qcommon/qfiles.h:574-585`
#[repr(C)]
pub struct glyphInfo_t {
	/// number of pixels wide
	pub width: i16,
	/// number of scan lines
	pub height: i16,
	/// number of pixels to advance to the next char
	pub horizAdvance: i16,
	/// x offset into space to render glyph
	pub horizOffset: i16,
	/// y offset
	pub baseline: i32,
	/// x start tex coord
	pub s: f32,
	/// y start tex coord
	pub t: f32,
	/// x end tex coord
	pub s2: f32,
	/// y end tex coord
	pub t2: f32,
}

const _: () = assert!(core::mem::size_of::<glyphInfo_t>() == 28);
const _: () = assert!(core::mem::offset_of!(glyphInfo_t, width) == 0);
const _: () = assert!(core::mem::offset_of!(glyphInfo_t, height) == 2);
const _: () = assert!(core::mem::offset_of!(glyphInfo_t, horizAdvance) == 4);
const _: () = assert!(core::mem::offset_of!(glyphInfo_t, horizOffset) == 6);
const _: () = assert!(core::mem::offset_of!(glyphInfo_t, baseline) == 8);
const _: () = assert!(core::mem::offset_of!(glyphInfo_t, s) == 12);
const _: () = assert!(core::mem::offset_of!(glyphInfo_t, t) == 16);
const _: () = assert!(core::mem::offset_of!(glyphInfo_t, s2) == 20);
const _: () = assert!(core::mem::offset_of!(glyphInfo_t, t2) == 24);
