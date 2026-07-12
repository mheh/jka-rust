#![allow(non_camel_case_types, non_snake_case)]

/// Raven `STREAMINGBUFFER` — OpenAL streaming audio buffer handle.
///
/// Type definition source: `oracle/codemp/client/snd_local.h:80-85`
#[repr(C)]
pub struct STREAMINGBUFFER {
    pub BufferID: u32, // ALuint
    pub Status: u32,   // ALuint
    pub Data: *mut i8,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<STREAMINGBUFFER>() == 16);
const _: () = assert!(core::mem::offset_of!(STREAMINGBUFFER, BufferID) == 0);
const _: () = assert!(core::mem::offset_of!(STREAMINGBUFFER, Status) == 4);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(STREAMINGBUFFER, Data) == 8);
