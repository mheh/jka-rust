#![allow(non_camel_case_types, non_snake_case)]

/// Raven `wavinfo_t` — parsed WAV file header info.
///
/// Type definition source: `oracle/codemp/client/snd_local.h:137-144`
#[repr(C)]
pub struct wavinfo_t {
	pub format: i32,
	pub rate: i32,
	pub width: i32,
	pub channels: i32,
	pub samples: i32,
	pub dataofs: i32, // chunk starts this many bytes from file start
}

const _: () = assert!(core::mem::size_of::<wavinfo_t>() == 24);
const _: () = assert!(core::mem::offset_of!(wavinfo_t, format) == 0);
const _: () = assert!(core::mem::offset_of!(wavinfo_t, rate) == 4);
const _: () = assert!(core::mem::offset_of!(wavinfo_t, width) == 8);
const _: () = assert!(core::mem::offset_of!(wavinfo_t, channels) == 12);
const _: () = assert!(core::mem::offset_of!(wavinfo_t, samples) == 16);
const _: () = assert!(core::mem::offset_of!(wavinfo_t, dataofs) == 20);
