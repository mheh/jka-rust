#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_long;

//TODO: Port CDraw32
// Source: oracle/codemp/qcommon/cm_draw.h:86-247
// The automap raster lane (gh#29, DEC-55.4) owns the drawing methods and the
// rest of the static context (`buffer`, `buf_width`, `buf_height`, `stride`,
// `row_off`).
// This struct holds only the clip bounds, which `cm_draw.rs::code` reads today.

/// Raven `CDraw32` — the 32-bit-per-pixel drawing class.
///
/// Raven keeps the drawing context in class statics so that a caller sets it
/// once for many draw calls.
/// This port carries that context as fields, because the codebase allows no
/// globals.
///
/// Type definition source: `oracle/codemp/qcommon/cm_draw.h:86-247`
/// Static definitions source: `oracle/codemp/qcommon/cm_draw.cpp:16-23`
#[derive(Clone, Copy, Default, Debug)]
pub struct CDraw32 {
    /// clip bounds
    pub clip_min_x: c_long,
    /// clip bounds
    pub clip_min_y: c_long,
    /// clip bounds
    pub clip_max_x: c_long,
    /// clip bounds
    pub clip_max_y: c_long,
}
