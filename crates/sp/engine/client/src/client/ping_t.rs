#![allow(non_camel_case_types, non_snake_case)]

use sp_engine_qcommon::qcommon::netadr_t::netadr_t;

/// Raven `ping_t` — a pending server ping request/response record.
///
/// Raven: (unnamed).
/// Type definition source: `oracle/code/client/client.h:177-181`
#[repr(C)]
pub struct ping_t {
    pub adr: netadr_t,
    pub start: i32,
    pub time: i32,
}

const _: () = assert!(core::mem::size_of::<ping_t>() == 16);
const _: () = assert!(core::mem::offset_of!(ping_t, adr) == 0);
const _: () = assert!(core::mem::offset_of!(ping_t, start) == 8);
const _: () = assert!(core::mem::offset_of!(ping_t, time) == 12);
