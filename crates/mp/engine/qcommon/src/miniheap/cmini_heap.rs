#![allow(non_camel_case_types, non_snake_case)]
use std::os::raw::{c_char, c_int};

/// Raven `CMiniHeap` — bump-pointer heap allocator over a fixed malloc'd block.
///
/// Raven: no class-level comment.
/// Type definition source: `oracle/codemp/qcommon/MiniHeap.h:5-51`
#[repr(C)]
pub struct CMiniHeap {
    mHeap: *mut c_char,
    mCurrentHeap: *mut c_char,
    mSize: c_int,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<CMiniHeap>() == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(CMiniHeap, mHeap) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(CMiniHeap, mCurrentHeap) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(CMiniHeap, mSize) == 16);
