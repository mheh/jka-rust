//! `ServerStatusInfo` — Raven `serverStatusInfo_t`.

/// Raven `#define MAX_SERVERSTATUS_LINES 128`.
///
/// Source: `oracle/codemp/ui/ui_local.h:578`
pub const MAX_SERVERSTATUS_LINES: usize = 128;

/// Raven `#define MAX_SERVERSTATUS_TEXT 1024`.
///
/// Source: `oracle/codemp/ui/ui_local.h:579`
pub const MAX_SERVERSTATUS_TEXT: usize = 1024;

/// Raven `serverStatusInfo_t` — the parsed `serverStatus` response the
/// server-info menu feeds from.
///
/// PORT-NOTE: Raven's `char *lines[MAX_SERVERSTATUS_LINES][4]` pointed into the
/// `text` and `pings` backing buffers that the same parse filled; owning the
/// four cells per row makes the rows self-contained, so the backing buffers
/// drop out (porting-rules §C9). `numLines` is `lines.len()`.
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:703-709`
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[doc(alias = "serverStatusInfo_t")]
pub struct ServerStatusInfo {
    pub address: String,
    pub lines: Vec<[String; 4]>,
}
