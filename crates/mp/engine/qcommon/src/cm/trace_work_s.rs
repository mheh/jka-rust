#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;
use mp_qshared::shared::collision::cplane_t;
use mp_qshared::shared::{qboolean, vec3_t, vec3pair_t};

use super::cbrushside_s::cbrushside_t;
use super::sphere_t::sphere_t;

/// Raven `traceWork_t` — working state for a single trace/sweep through the
/// collision model.
///
/// Raven: rwwRMG - modified.
/// Type definition source: `oracle/oracle/codemp/qcommon/cm_local.h:238-264`
#[repr(C)]
pub struct traceWork_t {
    pub start: vec3_t,
    pub end: vec3_t,
    /// size of the box being swept through the model
    pub size: [vec3_t; 2],
    /// `[signbits][x]` = either `size[0][x]` or `size[1][x]`
    pub offsets: [vec3_t; 8],
    /// longest corner length from origin
    pub maxOffset: f32,
    /// greatest of abs(size[0]) and abs(size[1])
    pub extents: vec3_t,
    /// origin of the model tracing through
    pub modelOrigin: vec3_t,
    /// ored contents of the model tracing through
    pub contents: c_int,
    /// optimized case
    pub isPoint: qboolean,
    /// sphere for oriendted capsule collision
    pub sphere: sphere_t,

    // rwwRMG - added:
    /// enclosing box of start and end surrounding by size
    pub bounds: vec3pair_t,
    /// enclosing box of start and end surrounding by size for a segment
    pub localBounds: vec3pair_t,

    /// global enter fraction (before processing subsections of the brush)
    pub baseEnterFrac: f32,
    /// global leave fraction (before processing subsections of the brush)
    pub baseLeaveFrac: f32,
    /// fraction where the ray enters the brush
    pub enterFrac: f32,
    /// fraction where the ray leaves the brush
    pub leaveFrac: f32,
    pub leadside: *mut cbrushside_t,
    pub clipplane: *mut cplane_t,
    pub startout: bool,
    pub getout: bool,
}

pub type traceWork_s = traceWork_t;

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<traceWork_t>() == 296);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(traceWork_t, start) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(traceWork_t, end) == 12);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(traceWork_t, size) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(traceWork_t, offsets) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(traceWork_t, maxOffset) == 144);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(traceWork_t, extents) == 148);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(traceWork_t, modelOrigin) == 160);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(traceWork_t, contents) == 172);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(traceWork_t, isPoint) == 176);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(traceWork_t, sphere) == 180);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(traceWork_t, bounds) == 204);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(traceWork_t, localBounds) == 228);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(traceWork_t, baseEnterFrac) == 252);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(traceWork_t, baseLeaveFrac) == 256);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(traceWork_t, enterFrac) == 260);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(traceWork_t, leaveFrac) == 264);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(traceWork_t, leadside) == 272);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(traceWork_t, clipplane) == 280);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(traceWork_t, startout) == 288);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(traceWork_t, getout) == 289);
