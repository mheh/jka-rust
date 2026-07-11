#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_long};

/// Raven `roff_hdr_t` — legacy (v1) ROFF file header.
///
/// Raven: `mHeader` should be "ROFF" (Rotation, Origin File Format). `mCount`
/// there isn't any reason for this to be anything other than an int, sigh...
/// Move - Rotate data follows....vec3_t delta_origin, vec3_t delta_rotation
/// Type definition source: `oracle/code/game/g_roff.h:18-26`
#[repr(C)]
pub struct roff_hdr_t {
    pub mHeader: [c_char; 4],
    pub mVersion: c_long,
    pub mCount: f32,
}

const _: () = assert!(core::mem::size_of::<roff_hdr_t>() == 24);
const _: () = assert!(core::mem::offset_of!(roff_hdr_t, mHeader) == 0);
const _: () = assert!(core::mem::offset_of!(roff_hdr_t, mVersion) == 8);
const _: () = assert!(core::mem::offset_of!(roff_hdr_t, mCount) == 16);
