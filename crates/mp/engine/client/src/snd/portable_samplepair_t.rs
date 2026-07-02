#![allow(non_camel_case_types, non_snake_case)]

/// Raven `portable_samplepair_t` — a mixed stereo sample pair.
///
/// Raven: the final values will be clamped to +/- 0x00ffff00 and shifted down.
/// Type definition source: `oracle/oracle/codemp/client/snd_local.h:30-33`
#[repr(C)]
pub struct portable_samplepair_t {
	pub left: i32,
	pub right: i32,
}

const _: () = assert!(core::mem::size_of::<portable_samplepair_t>() == 8);
const _: () = assert!(core::mem::offset_of!(portable_samplepair_t, left) == 0);
const _: () = assert!(core::mem::offset_of!(portable_samplepair_t, right) == 4);
