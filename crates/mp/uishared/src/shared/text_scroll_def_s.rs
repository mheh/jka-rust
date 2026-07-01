#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_float, c_int};

// Raven `#define MAX_TEXTSCROLL_LINES 256`.
// Source: `oracle/oracle/codemp/ui/ui_shared.h:20`
const MAX_TEXTSCROLL_LINES: usize = 256;

/// Raven `textScrollDef_s` (typedef `textScrollDef_t`) — a scrolling text box's
/// line buffer and layout state.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_shared.h:226-240`
#[repr(C)]
pub struct textScrollDef_s {
    pub startPos: c_int,
    pub endPos: c_int,

    pub lineHeight: c_float,
    pub maxLineChars: c_int,
    pub drawPadding: c_int,

    // changed spelling to make them fall out during compile while I made them asian-aware -Ste
    pub iLineCount: c_int,
    /// can contain NULL ptrs that you should skip over during paint.
    pub pLines: [*const c_char; MAX_TEXTSCROLL_LINES],
}

/// Raven `textScrollDef_t` — `typedef struct textScrollDef_s textScrollDef_t`.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_shared.h:226-240`
pub type textScrollDef_t = textScrollDef_s;

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<textScrollDef_t>() == 2072);
const _: () = assert!(core::mem::offset_of!(textScrollDef_t, startPos) == 0);
const _: () = assert!(core::mem::offset_of!(textScrollDef_t, endPos) == 4);
const _: () = assert!(core::mem::offset_of!(textScrollDef_t, lineHeight) == 8);
const _: () = assert!(core::mem::offset_of!(textScrollDef_t, maxLineChars) == 12);
const _: () = assert!(core::mem::offset_of!(textScrollDef_t, drawPadding) == 16);
const _: () = assert!(core::mem::offset_of!(textScrollDef_t, iLineCount) == 20);
const _: () = assert!(core::mem::offset_of!(textScrollDef_t, pLines) == 24);
