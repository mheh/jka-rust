//! `EditFieldDef` — Raven `editFieldDef_s`/`editFieldDef_t`.

use core::ffi::c_int;

/// Raven `#define MAX_EDITFIELD 256`.
///
/// Source: `oracle/codemp/ui/ui_shared.h:77`
pub const MAX_EDITFIELD: usize = 256;

/// Raven `editFieldDef_s` (typedef `editFieldDef_t`) — edit-field limits, one
/// of the `itemDef_t::typeData` payloads.
///
/// Type definition source: `oracle/codemp/ui/ui_shared.h:188-196`
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[doc(alias = "editFieldDef_s")]
#[doc(alias = "editFieldDef_t")]
#[allow(non_snake_case)]
pub struct EditFieldDef {
    /// edit field limits
    pub minVal: f32,
    pub maxVal: f32,
    pub defVal: f32,
    pub range: f32,
    /// for edit fields
    pub maxChars: c_int,
    /// for edit fields
    pub maxPaintChars: c_int,
    pub paintOffset: c_int,
}
