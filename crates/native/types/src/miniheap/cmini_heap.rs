#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};

/// Raven `CMiniHeap` — simple bump-allocator heap used by the Ghoul2 skeleton
/// system.
///
/// Raven: reset the heap back to the start / initialise the heap / free up
/// the heap / give me some space from the heap please.
/// Type definition source: `oracle/code/qcommon/MiniHeap.h:5-62`
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
