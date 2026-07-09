#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use sp_engine_qcommon::qcommon::netadr_t::netadr_t;

/// Raven `MAX_INFO_STRING`.
///
/// Source: `oracle/code/client/client.h:52`
const MAX_INFO_STRING: usize = 1024;

/// Raven `serverInfoResponse_t`.
///
/// Type definition source: `oracle/code/client/client.h:183-186`
#[repr(C)]
pub struct serverInfoResponse_t {
	pub netadr: netadr_t,

	pub info: [c_char; MAX_INFO_STRING],
}

const _: () = assert!(core::mem::size_of::<serverInfoResponse_t>() == 1032);
const _: () = assert!(core::mem::offset_of!(serverInfoResponse_t, netadr) == 0);
const _: () = assert!(core::mem::offset_of!(serverInfoResponse_t, info) == 8);
