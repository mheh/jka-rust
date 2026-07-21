#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `portable_samplepair_t` — a mixed stereo sample pair.
///
/// Type definition source: `oracle/codemp/client/snd_local.h:30-33`
/// Type definition source: `oracle/code/client/snd_local.h:30-33`
#[repr(C)]
pub struct portable_samplepair_t {
    /// the final values will be clamped to +/- 0x00ffff00 and shifted down
    pub left: c_int,
    pub right: c_int,
}

const _: () = assert!(core::mem::size_of::<portable_samplepair_t>() == 8);
const _: () = assert!(core::mem::offset_of!(portable_samplepair_t, left) == 0);
const _: () = assert!(core::mem::offset_of!(portable_samplepair_t, right) == 4);
