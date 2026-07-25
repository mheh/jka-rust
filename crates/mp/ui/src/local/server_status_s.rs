//! `ServerStatus` — Raven `serverStatus_t`.

use core::ffi::c_int;

use mp_qshared::shared::qhandle_t;

use super::pinglist_t::PingList;

/// Raven `#define MAX_PINGREQUESTS 32`.
///
/// Source: `oracle/codemp/ui/ui_local.h:570`
pub const MAX_PINGREQUESTS: usize = 32;

/// Raven `#define MAX_DISPLAY_SERVERS 2048`.
///
/// Source: `oracle/codemp/ui/ui_local.h:577`
pub const MAX_DISPLAY_SERVERS: usize = 2048;

/// Raven `serverStatus_s` (typedef `serverStatus_t`) — the server browser's
/// refresh/ping/sort state plus the motd ticker.
///
/// PORT-NOTE: `pingList` stays a fixed `[PingList; MAX_PINGREQUESTS]` — the
/// poller scans every slot and marks free ones by clearing `adrstr`, so the
/// slots are addressed, not appended. `displayServers[2048]` + `numDisplayServers`
/// is a built-then-walked list and becomes `Vec<c_int>` (`numDisplayServers` is
/// its `len()`).
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:660-687`
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[doc(alias = "serverStatus_s")]
#[doc(alias = "serverStatus_t")]
#[allow(non_snake_case)]
pub struct ServerStatus {
    pub pingList: [PingList; MAX_PINGREQUESTS],
    pub numqueriedservers: c_int,
    pub currentping: c_int,
    pub nextpingtime: c_int,
    pub maxservers: c_int,
    pub refreshtime: c_int,
    pub numServers: c_int,
    pub sortKey: c_int,
    pub sortDir: c_int,
    pub lastCount: c_int,
    pub refreshActive: bool,
    pub currentServer: c_int,
    pub displayServers: Vec<c_int>,
    pub numPlayersOnServers: c_int,
    pub nextDisplayRefresh: c_int,
    pub nextSortTime: c_int,
    pub currentServerPreview: qhandle_t,
    pub currentServerCinematic: c_int,
    pub motdLen: c_int,
    pub motdWidth: c_int,
    pub motdPaintX: c_int,
    pub motdPaintX2: c_int,
    pub motdOffset: c_int,
    pub motdTime: c_int,
    pub motd: String,
}
