#![allow(non_camel_case_types, non_snake_case)]

/// Raven `netadrtype_t` — network address type enumeration.
///
/// Raven: .
/// Type definition source: `oracle/code/qcommon/qcommon.h:127-130`
#[repr(i32)]
pub enum netadrtype_t {
	NA_BAD,		// an address lookup failed
	NA_LOOPBACK,
}
