#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::c_int;

/// Raven `wavinfo_t` — parsed WAV file header info.
///
/// Raven: (none).
/// Type definition source: `oracle/oracle/code/client/snd_local.h:137-144`
#[repr(C)]
pub struct wavinfo_t {
	pub format: c_int,
	pub rate: c_int,
	pub width: c_int,
	pub channels: c_int,
	pub samples: c_int,
	/// chunk starts this many bytes from file start
	pub dataofs: c_int,
}

const _: () = assert!(core::mem::size_of::<wavinfo_t>() == 24);
const _: () = assert!(core::mem::offset_of!(wavinfo_t, format) == 0);
const _: () = assert!(core::mem::offset_of!(wavinfo_t, rate) == 4);
const _: () = assert!(core::mem::offset_of!(wavinfo_t, width) == 8);
const _: () = assert!(core::mem::offset_of!(wavinfo_t, channels) == 12);
const _: () = assert!(core::mem::offset_of!(wavinfo_t, samples) == 16);
const _: () = assert!(core::mem::offset_of!(wavinfo_t, dataofs) == 20);
