//! `TextScrollDef` — Raven `textScrollDef_s`/`textScrollDef_t`.

use core::ffi::c_int;

/// Raven `#define MAX_TEXTSCROLL_LINES 256`.
///
/// Source: `oracle/codemp/ui/ui_shared.h:20`
pub const MAX_TEXTSCROLL_LINES: usize = 256;

/// Raven `textScrollDef_s` (typedef `textScrollDef_t`) — a scrolling text
/// box's line buffer and layout state, one of the `itemDef_t::typeData`
/// payloads.
///
/// PORT-NOTE: Raven's `pLines[MAX_TEXTSCROLL_LINES]` held `String_Alloc`
/// pointers with NULL holes the painter skipped; the owned `Vec<String>`
/// carries the built lines and `iLineCount` is `pLines.len()`.
///
/// Type definition source: `oracle/codemp/ui/ui_shared.h:226-240`
#[derive(Debug, Clone, PartialEq, Default)]
#[doc(alias = "textScrollDef_s")]
#[doc(alias = "textScrollDef_t")]
#[allow(non_snake_case)]
pub struct TextScrollDef {
    pub startPos: c_int,
    pub endPos: c_int,

    pub lineHeight: f32,
    pub maxLineChars: c_int,
    pub drawPadding: c_int,

    // changed spelling to make them fall out during compile while I made them
    // asian-aware -Ste
    pub pLines: Vec<String>,
}
