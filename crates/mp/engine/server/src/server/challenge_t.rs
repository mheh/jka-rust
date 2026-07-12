#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::qcommon::netadr_t::netadr_t;
use mp_qshared::shared::qboolean;

/// Raven `challenge_t`.
///
/// Type definition source: `oracle/codemp/qcommon/../server/server.h:194-201`
#[repr(C)]
pub struct challenge_t {
    pub adr: netadr_t,
    pub challenge: i32,
    /// time the last packet was sent to the autherize server
    pub time: i32,
    /// time the challenge response was sent to client
    pub pingTime: i32,
    /// time the adr was first used, for authorize timeout checks
    pub firstTime: i32,
    pub connected: qboolean,
}

const _: () = assert!(core::mem::size_of::<challenge_t>() == 40);
const _: () = assert!(core::mem::offset_of!(challenge_t, adr) == 0);
const _: () = assert!(core::mem::offset_of!(challenge_t, challenge) == 20);
const _: () = assert!(core::mem::offset_of!(challenge_t, time) == 24);
const _: () = assert!(core::mem::offset_of!(challenge_t, pingTime) == 28);
const _: () = assert!(core::mem::offset_of!(challenge_t, firstTime) == 32);
const _: () = assert!(core::mem::offset_of!(challenge_t, connected) == 36);
