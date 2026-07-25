//! `GameTypeInfo` — Raven `gameTypeInfo`.

use core::ffi::c_int;

/// Raven `gameTypeInfo` — a gametype's menu label and its `GT_*` enum value.
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:624-627`
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[doc(alias = "gameTypeInfo")]
#[allow(non_snake_case)]
pub struct GameTypeInfo {
    pub gameType: String,
    pub gtEnum: c_int,
}
