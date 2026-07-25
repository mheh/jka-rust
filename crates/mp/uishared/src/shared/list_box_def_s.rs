//! `ListBoxDef` — Raven `listBoxDef_s`/`listBoxDef_t`.

use core::ffi::c_int;

use super::column_info_s::ColumnInfo;

/// Raven `#define MAX_LB_COLUMNS 16`.
///
/// Source: `oracle/codemp/ui/ui_shared.h:164`
pub const MAX_LB_COLUMNS: usize = 16;

/// Raven `listBoxDef_s` (typedef `listBoxDef_t`) — list box layout/state, one
/// of the `itemDef_t::typeData` payloads.
///
/// Type definition source: `oracle/codemp/ui/ui_shared.h:172-186`
#[derive(Debug, Clone, PartialEq, Default)]
#[doc(alias = "listBoxDef_s")]
#[doc(alias = "listBoxDef_t")]
#[allow(non_snake_case)]
pub struct ListBoxDef {
    pub startPos: c_int,
    pub endPos: c_int,
    pub drawPadding: c_int,
    pub cursorPos: c_int,
    pub elementWidth: f32,
    pub elementHeight: f32,
    pub elementStyle: c_int,
    pub numColumns: c_int,
    pub columnInfo: [ColumnInfo; MAX_LB_COLUMNS],
    pub doubleClick: String,
    pub notselectable: bool,
    // JLF MPMOVED
    pub scrollhidden: bool,
}
