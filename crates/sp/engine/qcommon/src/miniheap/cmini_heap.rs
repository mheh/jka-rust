#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

/// Raven `CMiniHeap` — simple bump-allocator heap used by the Ghoul2 skeleton
/// system.
///
/// Raven: reset the heap back to the start / initialise the heap / free up
/// the heap / give me some space from the heap please.
/// Type definition source: `oracle/oracle/code/qcommon/MiniHeap.h:5-62`
#[repr(C)]
pub struct CMiniHeap {
    pub mHeap: *mut c_char,
    pub mCurrentHeap: *mut c_char,
    pub mSize: i32,
}

const _: () = assert!(core::mem::size_of::<CMiniHeap>() == 24);
const _: () = assert!(core::mem::offset_of!(CMiniHeap, mHeap) == 0);
const _: () = assert!(core::mem::offset_of!(CMiniHeap, mCurrentHeap) == 8);
const _: () = assert!(core::mem::offset_of!(CMiniHeap, mSize) == 16);
