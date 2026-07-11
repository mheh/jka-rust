#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_void};

/// Raven `roff_list_t` — loaded ROFF (relative orientation-and-facing) file entry.
///
/// Type definition source: `oracle/code/game/g_roff.h:62-73`
#[repr(C)]
pub struct roff_list_t {
    pub r#type: i32,           // roff type number, 1-old, 2-new
    pub fileName: *mut c_char, // roff filename
    pub frames: i32,           // number of roff entries
    pub data: *mut c_void,     // delta move and rotate vector list
    pub mFrameTime: i32,       // frame rate
    pub mLerp: i32,            // Lerp rate (FPS)
    pub mNumNoteTracks: i32,
    pub mNoteTrackIndexes: *mut *mut c_char,
}

const _: () = assert!(core::mem::size_of::<roff_list_t>() == 56);
const _: () = assert!(core::mem::offset_of!(roff_list_t, r#type) == 0);
const _: () = assert!(core::mem::offset_of!(roff_list_t, fileName) == 8);
const _: () = assert!(core::mem::offset_of!(roff_list_t, frames) == 16);
const _: () = assert!(core::mem::offset_of!(roff_list_t, data) == 24);
const _: () = assert!(core::mem::offset_of!(roff_list_t, mFrameTime) == 32);
const _: () = assert!(core::mem::offset_of!(roff_list_t, mLerp) == 36);
const _: () = assert!(core::mem::offset_of!(roff_list_t, mNumNoteTracks) == 40);
const _: () = assert!(core::mem::offset_of!(roff_list_t, mNoteTrackIndexes) == 48);
