#![allow(non_camel_case_types, non_snake_case)]

/// Raven `netadrtype_t` — network address types.
///
/// Type definition source: `oracle/codemp/qcommon/qcommon.h:108-116`
#[repr(i32)]
pub enum netadrtype_t {
    NA_BOT = 0,
    NA_BAD = 1,              // an address lookup failed
    NA_LOOPBACK = 2,
    NA_BROADCAST = 3,
    NA_IP = 4,
    NA_IPX = 5,
    NA_BROADCAST_IPX = 6,
}
