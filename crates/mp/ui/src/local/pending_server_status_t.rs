//! `PendingServerStatus` — Raven `pendingServerStatus_t`.

use core::ffi::c_int;

use super::pending_server_t::PendingServer;

/// Raven `#define MAX_SERVERSTATUSREQUESTS 16`.
///
/// Source: `oracle/codemp/game/q_shared.h:3062`
pub const MAX_SERVERSTATUSREQUESTS: usize = 16;

/// Raven `pendingServerStatus_t` — the find-player poller's fixed request
/// slots. The array stays fixed-size: slots are addressed by request index and
/// individually validated by `PendingServer::valid`, not appended to.
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:698-701`
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[doc(alias = "pendingServerStatus_t")]
#[allow(non_snake_case)]
pub struct PendingServerStatus {
    pub num: c_int,
    pub server: [PendingServer; MAX_SERVERSTATUSREQUESTS],
}
