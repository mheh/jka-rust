#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::c_char;

use mp_engine_qcommon::qcommon::netadr_t::netadr_t;

/// Raven `MAX_INFO_STRING`.
const MAX_INFO_STRING: usize = 1024;

/// Raven `ping_t` — a pending server ping request/response record.
///
/// Type definition source: `oracle/oracle/codemp/client/client.h:247-255`
#[repr(C)]
pub struct ping_t {
	pub adr: netadr_t,
	pub start: i32,
	pub time: i32,
	pub info: [c_char; MAX_INFO_STRING],
}

const _: () = assert!(core::mem::size_of::<ping_t>() == 1052);
const _: () = assert!(core::mem::offset_of!(ping_t, adr) == 0);
const _: () = assert!(core::mem::offset_of!(ping_t, start) == 20);
const _: () = assert!(core::mem::offset_of!(ping_t, time) == 24);
const _: () = assert!(core::mem::offset_of!(ping_t, info) == 28);
