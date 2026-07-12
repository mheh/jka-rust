#![allow(non_camel_case_types, non_snake_case)]

use sp_engine_qcommon::qcommon::netadr_t::netadr_t;

/// Raven `challenge_t` — a pending/verified client connection challenge.
///
/// Type definition source: `oracle/code/server/server.h:135-139`
#[repr(C)]
pub struct challenge_t {
    pub adr: netadr_t,
    pub challenge: i32,
    pub time: i32,
}

const _: () = assert!(core::mem::size_of::<challenge_t>() == 16);
const _: () = assert!(core::mem::offset_of!(challenge_t, adr) == 0);
const _: () = assert!(core::mem::offset_of!(challenge_t, challenge) == 8);
const _: () = assert!(core::mem::offset_of!(challenge_t, time) == 12);
