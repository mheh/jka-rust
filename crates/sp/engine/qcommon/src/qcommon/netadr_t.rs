#![allow(non_camel_case_types, non_snake_case)]

use super::netadrtype_t::netadrtype_t;

/// Raven `netadr_t` — a network address with its associated port.
///
/// Type definition source: `oracle/oracle/code/qcommon/qcommon.h:137-141`
#[repr(C)]
pub struct netadr_t {
	pub r#type: netadrtype_t,

	pub port: u16,
}

const _: () = assert!(core::mem::size_of::<netadr_t>() == 8);
const _: () = assert!(core::mem::offset_of!(netadr_t, r#type) == 0);
const _: () = assert!(core::mem::offset_of!(netadr_t, port) == 4);
