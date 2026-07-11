#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_long};

/// Raven `roff_hdr2_t` — v2 ROFF file header.
///
/// Raven: `mHeader` should match roff_string defined above. `mVersion`
/// version num, supported version defined above. `mCount` I think this is a
/// float because of a limitation of the roff exporter. `mFrameRate` frame
/// rate the roff should be played at. `mNumNotes` number of notes (null
/// terminated strings) after the roff data.
/// Type definition source: `oracle/code/game/g_roff.h:38-47`
#[repr(C)]
pub struct roff_hdr2_t {
    pub mHeader: [c_char; 4],
    pub mVersion: c_long,
    pub mCount: i32,
    pub mFrameRate: i32,
    pub mNumNotes: i32,
}

const _: () = assert!(core::mem::size_of::<roff_hdr2_t>() == 32);
const _: () = assert!(core::mem::offset_of!(roff_hdr2_t, mHeader) == 0);
const _: () = assert!(core::mem::offset_of!(roff_hdr2_t, mVersion) == 8);
const _: () = assert!(core::mem::offset_of!(roff_hdr2_t, mCount) == 16);
const _: () = assert!(core::mem::offset_of!(roff_hdr2_t, mFrameRate) == 20);
const _: () = assert!(core::mem::offset_of!(roff_hdr2_t, mNumNotes) == 24);
