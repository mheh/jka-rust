#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `editFieldDef_s` — edit field limits for a text/numeric field.
///
/// Type definition source: `oracle/codemp/ui/ui_shared.h:188-196`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct editFieldDef_s {
    /// Raven comment: edit field limits
    pub minVal: f32,
    pub maxVal: f32,
    pub defVal: f32,
    pub range: f32,
    /// Raven comment: for edit fields
    pub maxChars: c_int,
    /// Raven comment: for edit fields
    pub maxPaintChars: c_int,
    pub paintOffset: c_int,
}

/// Raven `editFieldDef_t` — `typedef struct editFieldDef_s editFieldDef_t`.
///
/// Type definition source: `oracle/codemp/ui/ui_shared.h:188-196`
pub type editFieldDef_t = editFieldDef_s;

const _: () = assert!(core::mem::size_of::<editFieldDef_t>() == 28);
const _: () = assert!(core::mem::offset_of!(editFieldDef_t, minVal) == 0);
const _: () = assert!(core::mem::offset_of!(editFieldDef_t, maxVal) == 4);
const _: () = assert!(core::mem::offset_of!(editFieldDef_t, defVal) == 8);
const _: () = assert!(core::mem::offset_of!(editFieldDef_t, range) == 12);
const _: () = assert!(core::mem::offset_of!(editFieldDef_t, maxChars) == 16);
const _: () = assert!(core::mem::offset_of!(editFieldDef_t, maxPaintChars) == 20);
const _: () = assert!(core::mem::offset_of!(editFieldDef_t, paintOffset) == 24);
