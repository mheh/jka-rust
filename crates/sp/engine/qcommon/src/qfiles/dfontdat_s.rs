#![allow(non_camel_case_types, non_snake_case)]

use super::glyph_info_t::glyphInfo_t;

/// Number of glyphs in a `dfontdat_t`.
pub const GLYPH_COUNT: usize = 256;

/// Raven `dfontdat_t` — on-disk font data (glyph table + metrics).
///
/// Type definition source: `oracle/code/qcommon/qfiles.h:617-627`
#[repr(C)]
pub struct dfontdat_t {
	pub mGlyphs: [glyphInfo_t; GLYPH_COUNT],

	pub mPointSize: i16,
	/// max height of font
	pub mHeight: i16,
	pub mAscender: i16,
	pub mDescender: i16,

	/// unused field, written out by John's fontgen program but we have to leave it there for disk structs <sigh>
	pub mKoreanHack: i16,
}

const _: () = assert!(core::mem::size_of::<dfontdat_t>() == 7180);
const _: () = assert!(core::mem::offset_of!(dfontdat_t, mGlyphs) == 0);
const _: () = assert!(core::mem::offset_of!(dfontdat_t, mPointSize) == 7168);
const _: () = assert!(core::mem::offset_of!(dfontdat_t, mHeight) == 7170);
const _: () = assert!(core::mem::offset_of!(dfontdat_t, mAscender) == 7172);
const _: () = assert!(core::mem::offset_of!(dfontdat_t, mDescender) == 7174);
const _: () = assert!(core::mem::offset_of!(dfontdat_t, mKoreanHack) == 7176);

pub type dfontdat_s = dfontdat_t;
