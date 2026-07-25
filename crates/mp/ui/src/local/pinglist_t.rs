//! `PingList` — Raven `pinglist_t`.

use core::ffi::c_int;

/// Raven `#define MAX_ADDRESSLENGTH 64`.
///
/// Source: `oracle/codemp/ui/ui_local.h:571`
pub const MAX_ADDRESSLENGTH: usize = 64;

/// Raven `pinglist_t` — one outstanding server ping (address + issue time).
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:654-657`
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[doc(alias = "pinglist_t")]
#[allow(non_snake_case)]
pub struct PingList {
    pub adrstr: String,
    pub start: c_int,
}
