#![allow(non_camel_case_types, non_snake_case)]

/// Raven `IN_OUT` — byte counters for an mp3 decode call.
///
/// Type definition source: `oracle/code/client/../mp3code/small_header.h:18-22`
#[repr(C)]
pub struct IN_OUT {
    pub in_bytes: i32,
    pub out_bytes: i32,
}

const _: () = assert!(core::mem::size_of::<IN_OUT>() == 8);
const _: () = assert!(core::mem::offset_of!(IN_OUT, in_bytes) == 0);
const _: () = assert!(core::mem::offset_of!(IN_OUT, out_bytes) == 4);
