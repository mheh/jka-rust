#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_uint};

/// Raven `STREAMINGBUFFER` — OpenAL streaming audio buffer handle.
///
/// Type definition source: `oracle/codemp/client/snd_local.h:80-85`
/// Type definition source: `oracle/code/client/snd_local.h:80-85`
#[repr(C)]
pub struct STREAMINGBUFFER {
    pub BufferID: c_uint, // ALuint
    pub Status: c_uint,   // ALuint
    pub Data: *mut c_char,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<STREAMINGBUFFER>() == 16);
const _: () = assert!(core::mem::offset_of!(STREAMINGBUFFER, BufferID) == 0);
const _: () = assert!(core::mem::offset_of!(STREAMINGBUFFER, Status) == 4);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(STREAMINGBUFFER, Data) == 8);
