#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::qboolean;

/// Raven `msg_t` — a growable read/write bit-stream buffer used for network
/// messages and demo/save serialization.
///
/// Type definition source: `oracle/oracle/code/qcommon/qcommon.h:26-34`
#[repr(C)]
pub struct msg_t {
	pub allowoverflow: qboolean, // if false, do a Com_Error
	pub overflowed: qboolean,    // set to true if the buffer size failed (with allowoverflow set)
	pub data: *mut u8,
	pub maxsize: i32,
	pub cursize: i32,
	pub readcount: i32,
	pub bit: i32, // for bitwise reads and writes
}

const _: () = assert!(core::mem::size_of::<msg_t>() == 32);
const _: () = assert!(core::mem::offset_of!(msg_t, allowoverflow) == 0);
const _: () = assert!(core::mem::offset_of!(msg_t, overflowed) == 4);
const _: () = assert!(core::mem::offset_of!(msg_t, data) == 8);
const _: () = assert!(core::mem::offset_of!(msg_t, maxsize) == 16);
const _: () = assert!(core::mem::offset_of!(msg_t, cursize) == 20);
const _: () = assert!(core::mem::offset_of!(msg_t, readcount) == 24);
const _: () = assert!(core::mem::offset_of!(msg_t, bit) == 28);
