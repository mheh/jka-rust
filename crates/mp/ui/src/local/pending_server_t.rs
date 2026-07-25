//! `PendingServer` — Raven `pendingServer_t`.

use core::ffi::c_int;

/// Raven `pendingServer_t` — one in-flight server-status request slot of the
/// find-player poller.
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:690-696`
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[doc(alias = "pendingServer_t")]
#[allow(non_snake_case)]
pub struct PendingServer {
    pub adrstr: String,
    pub name: String,
    pub startTime: c_int,
    pub serverNum: c_int,
    pub valid: bool,
}
