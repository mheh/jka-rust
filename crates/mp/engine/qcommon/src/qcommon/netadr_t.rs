#![allow(non_camel_case_types, non_snake_case)]

use super::netadrtype_t::netadrtype_t;

/// Raven `netadr_t` — a network address (IP or IPX) with its associated port.
///
/// Type definition source: `oracle/oracle/codemp/qcommon/qcommon.h:123-130`
#[repr(C)]
pub struct netadr_t {
	pub r#type: netadrtype_t,

	pub ip: [u8; 4],
	pub ipx: [u8; 10],

	pub port: u16,
}

const _: () = assert!(core::mem::size_of::<netadr_t>() == 20);
const _: () = assert!(core::mem::offset_of!(netadr_t, r#type) == 0);
const _: () = assert!(core::mem::offset_of!(netadr_t, ip) == 4);
const _: () = assert!(core::mem::offset_of!(netadr_t, ipx) == 8);
const _: () = assert!(core::mem::offset_of!(netadr_t, port) == 18);
