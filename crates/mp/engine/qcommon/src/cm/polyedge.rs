#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_long;

/// Raven `POLYEDGE` — a polygon edge in the debug-surface scan converter (fixed
/// point long ints for accuracy & speed).
///
/// Type definition source: `oracle/codemp/qcommon/cm_draw.cpp:1070-1076`
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct POLYEDGE {
    /// x coordinate of edge's intersection with current scanline
    pub x: c_long,
    /// change in x with respect to y
    pub dx: c_long,
    /// edge number: edge i goes from pt\[i\] to pt\[i+1\]
    pub i: c_long,
}

const _: () = assert!(core::mem::offset_of!(POLYEDGE, x) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<POLYEDGE>() == 24);
    assert!(core::mem::offset_of!(POLYEDGE, dx) == 8);
    assert!(core::mem::offset_of!(POLYEDGE, i) == 16);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<POLYEDGE>() == 12);
    assert!(core::mem::offset_of!(POLYEDGE, dx) == 4);
    assert!(core::mem::offset_of!(POLYEDGE, i) == 8);
};
