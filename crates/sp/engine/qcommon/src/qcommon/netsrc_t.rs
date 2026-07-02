#![allow(non_camel_case_types, non_snake_case)]

/// Raven `netsrc_t` — network source enumeration.
///
/// Raven: .
/// Type definition source: `oracle/oracle/code/qcommon/qcommon.h:132-135`
#[repr(i32)]
pub enum netsrc_t {
	NS_CLIENT,
	NS_SERVER,
}
