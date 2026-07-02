#![allow(non_camel_case_types, non_snake_case)]

/// Raven `SAMPLE` — a single MP3 PCM sample, reinterpretable as int or float.
///
/// Type definition source: `oracle/oracle/codemp/client/../mp3code/small_header.h:11-15`
#[repr(C)]
pub union SAMPLE {
    pub s: i32,
    pub x: f32,
}

const _: () = assert!(core::mem::size_of::<SAMPLE>() == 4);
const _: () = assert!(core::mem::offset_of!(SAMPLE, s) == 0);
const _: () = assert!(core::mem::offset_of!(SAMPLE, x) == 0);
