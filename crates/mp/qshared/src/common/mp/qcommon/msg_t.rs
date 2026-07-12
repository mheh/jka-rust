#![allow(non_camel_case_types, non_snake_case)]

use crate::shared::qboolean;

/// Raven `msg_t` — a growable read/write bit-stream buffer used for network
/// messages and demo/save serialization.
///
/// Type definition source: `oracle/codemp/qcommon/qcommon.h:17-26`
#[repr(C)]
pub struct msg_t {
    pub allowoverflow: qboolean, // if false, do a Com_Error
    pub overflowed: qboolean,    // set to true if the buffer size failed (with allowoverflow set)
    pub oob: qboolean,           // set to true if the buffer size failed (with allowoverflow set)
    pub data: *mut u8,
    pub maxsize: i32,
    pub cursize: i32,
    pub readcount: i32,
    pub bit: i32, // for bitwise reads and writes
}

const _: () = assert!(core::mem::offset_of!(msg_t, allowoverflow) == 0);
const _: () = assert!(core::mem::offset_of!(msg_t, overflowed) == 4);
const _: () = assert!(core::mem::offset_of!(msg_t, oob) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<msg_t>() == 40);
    assert!(core::mem::offset_of!(msg_t, data) == 16);
    assert!(core::mem::offset_of!(msg_t, maxsize) == 24);
    assert!(core::mem::offset_of!(msg_t, cursize) == 28);
    assert!(core::mem::offset_of!(msg_t, readcount) == 32);
    assert!(core::mem::offset_of!(msg_t, bit) == 36);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree) — the retail
// 32-bit module ABI.
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<msg_t>() == 32);
    assert!(core::mem::offset_of!(msg_t, data) == 12);
    assert!(core::mem::offset_of!(msg_t, maxsize) == 16);
    assert!(core::mem::offset_of!(msg_t, cursize) == 20);
    assert!(core::mem::offset_of!(msg_t, readcount) == 24);
    assert!(core::mem::offset_of!(msg_t, bit) == 28);
};
