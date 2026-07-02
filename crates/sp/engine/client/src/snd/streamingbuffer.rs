#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::{c_char, c_uint};

/// Raven `STREAMINGBUFFER` — OpenAL streaming buffer state.
///
/// Raven: (none).
/// Type definition source: `oracle/oracle/code/client/snd_local.h:80-85`
#[repr(C)]
pub struct STREAMINGBUFFER {
	pub BufferID: c_uint,
	pub Status: c_uint,
	pub Data: *mut c_char,
}

const _: () = assert!(core::mem::size_of::<STREAMINGBUFFER>() == 16);
const _: () = assert!(core::mem::offset_of!(STREAMINGBUFFER, BufferID) == 0);
const _: () = assert!(core::mem::offset_of!(STREAMINGBUFFER, Status) == 4);
const _: () = assert!(core::mem::offset_of!(STREAMINGBUFFER, Data) == 8);
