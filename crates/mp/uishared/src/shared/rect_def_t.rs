#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_float;

/// Raven `rectDef_t` — a screen-space rectangle (position + size).
///
/// Type definition source: `oracle/codemp/ui/ui_shared.h:112-117`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct rectDef_t {
    /// horiz position
    pub x: c_float,
    /// vert position
    pub y: c_float,
    /// width
    pub w: c_float,
    /// height;
    pub h: c_float,
}

const _: () = assert!(core::mem::size_of::<rectDef_t>() == 16);
const _: () = assert!(core::mem::offset_of!(rectDef_t, x) == 0);
const _: () = assert!(core::mem::offset_of!(rectDef_t, y) == 4);
const _: () = assert!(core::mem::offset_of!(rectDef_t, w) == 8);
const _: () = assert!(core::mem::offset_of!(rectDef_t, h) == 12);
