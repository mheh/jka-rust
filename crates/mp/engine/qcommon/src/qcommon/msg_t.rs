#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::qboolean;

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

const _: () = assert!(core::mem::size_of::<msg_t>() == 40);
const _: () = assert!(core::mem::offset_of!(msg_t, allowoverflow) == 0);
const _: () = assert!(core::mem::offset_of!(msg_t, overflowed) == 4);
const _: () = assert!(core::mem::offset_of!(msg_t, oob) == 8);
const _: () = assert!(core::mem::offset_of!(msg_t, data) == 16);
const _: () = assert!(core::mem::offset_of!(msg_t, maxsize) == 24);
const _: () = assert!(core::mem::offset_of!(msg_t, cursize) == 28);
const _: () = assert!(core::mem::offset_of!(msg_t, readcount) == 32);
const _: () = assert!(core::mem::offset_of!(msg_t, bit) == 36);
