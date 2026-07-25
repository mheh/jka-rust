//! `ColumnInfo` — Raven `columnInfo_s`/`columnInfo_t`.

use core::ffi::c_int;

/// Raven `columnInfo_s` (typedef `columnInfo_t`) — a single column layout
/// within a list box.
///
/// Type definition source: `oracle/codemp/ui/ui_shared.h:166-170`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[doc(alias = "columnInfo_s")]
#[doc(alias = "columnInfo_t")]
#[allow(non_snake_case)]
pub struct ColumnInfo {
    pub pos: c_int,
    pub width: c_int,
    pub maxChars: c_int,
}
