#![allow(non_camel_case_types, non_snake_case)]

use super::pending_server_t::pendingServer_t;

/// `MAX_SERVERSTATUSREQUESTS`.
///
/// Source: `oracle/codemp/game/q_shared.h:3062`
const MAX_SERVERSTATUSREQUESTS: usize = 16;

/// Raven `pendingServerStatus_t`.
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:698-701`
#[repr(C)]
pub struct pendingServerStatus_t {
	pub num: i32,
	pub server: [pendingServer_t; MAX_SERVERSTATUSREQUESTS],
}

const _: () = assert!(core::mem::size_of::<pendingServerStatus_t>() == 2244);
const _: () = assert!(core::mem::offset_of!(pendingServerStatus_t, num) == 0);
const _: () = assert!(core::mem::offset_of!(pendingServerStatus_t, server) == 4);
