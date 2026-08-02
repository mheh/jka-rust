#![allow(non_camel_case_types, non_snake_case)]

//! `cm_draw.cpp` — the file-scope helpers of the `CDraw32` raster.
//!
//! The class itself lives in `cm/cdraw32.rs` (one Raven class per file,
//! porting-rules §F21). This module holds the four static helpers and the
//! shell sort the polygon scan converter calls.
//!
//! Raven reaches the drawing context and the scan-converter scratch through
//! file statics. This port threads both as parameters, because the codebase
//! allows no globals (porting-rules §B3).
//!
//! Source: `oracle/codemp/qcommon/cm_draw.cpp`

use core::ffi::{c_int, c_long};

use crate::cm::cdraw32::CDraw32;
use crate::cm::cm_draw_cpp_consts::{BOTTOM, INT_SHIFT, LEFT, RIGHT, TOP};
use crate::cm::point::POINT;
use crate::cm::poly_scan::PolyScan;
use crate::cm::polyedge::POLYEDGE;

/// Raven `code` determines where a point sits relative to the clip box.
///
/// Raven reads the clip bounds from `CDraw32` class statics.
/// This port takes the drawing context as a parameter, because the codebase
/// allows no globals.
///
/// Source: `oracle/codemp/qcommon/cm_draw.cpp:195-209`
pub fn code(draw: &CDraw32, x: c_long, y: c_long) -> c_long {
    let mut c: c_long = 0;

    if x < draw.clip_min_x {
        c |= LEFT;
    }
    if x > draw.clip_max_x {
        c |= RIGHT;
    }
    if y < draw.clip_min_y {
        c |= BOTTOM;
    }
    if y > draw.clip_max_y {
        c |= TOP;
    }

    c
}

/// Raven `del_edge` removes edge `i` from the active edge list.
///
/// Raven's `memcpy` over the overlapping tail is UB. The port shifts the tail
/// down one slot, which is the forward-copy result every shipping compiler
/// produced (porting-rules §F19).
///
/// Source: `oracle/codemp/qcommon/cm_draw.cpp:1089-1105`
pub fn del_edge(scan: &mut PolyScan, i: c_long) {
    let mut j: c_long = 0;

    while j < scan.nact && scan.active[j as usize].i != i {
        j += 1;
    }

    // The edge is not in the active list. This happens at cliprect->top.
    if j >= scan.nact {
        return;
    }

    scan.nact -= 1;
    let count = (scan.nact - j) as usize;
    for k in 0..count {
        scan.active[j as usize + k] = scan.active[j as usize + k + 1];
    }
}

/// Raven `ins_edge` appends edge `i` to the end of the active edge list.
///
/// Source: `oracle/codemp/qcommon/cm_draw.cpp:1107-1143`
pub fn ins_edge(scan: &mut PolyScan, i: c_long, y: c_long) {
    let j: c_long = if i < scan.n - 1 { i + 1 } else { 0 };

    let (p, q) = if scan.pt[i as usize].y < scan.pt[j as usize].y {
        (scan.pt[i as usize], scan.pt[j as usize])
    } else {
        (scan.pt[j as usize], scan.pt[i as usize])
    };

    // Initialize the x position at the intersection of the edge with scanline y.
    let dx: c_long = if (q.y - p.y) != 0 {
        ((q.x - p.x) * (1 << INT_SHIFT)) / (q.y - p.y)
    } else {
        // horizontal line
        0
    };

    let nact = scan.nact as usize;
    scan.active[nact].dx = dx;
    scan.active[nact].x = (dx * (y - p.y)) + (p.x << INT_SHIFT);
    scan.active[nact].i = i;
    scan.nact += 1;
}

/// Raven `compare_ind` orders two vertex indices by their y coordinate.
///
/// Raven reads the vertex array from the `pt` file static; the port takes it as
/// the leading parameter.
///
/// Source: `oracle/codemp/qcommon/cm_draw.cpp:1146-1149`
pub fn compare_ind(pt: &[POINT], u: &c_long, v: &c_long) -> c_int {
    if pt[*u as usize].y <= pt[*v as usize].y {
        -1
    } else {
        1
    }
}

/// Raven `compare_active` orders two active edges by their x position.
///
/// Source: `oracle/codemp/qcommon/cm_draw.cpp:1151-1154`
pub fn compare_active(u: &POLYEDGE, v: &POLYEDGE) -> c_int {
    if u.x <= v.x {
        -1
    } else {
        1
    }
}

/// Raven `shell_sort` sorts an array in place, using the best algorithm for an
/// almost-sorted list.
///
/// Raven walks raw bytes through a `void*` and a `memcpy` temporary. The port
/// takes a typed slice and keeps the gap sequence, the comparison order, and
/// the tie order exactly, because the polygon fill reads the sorted order.
///
/// Source: `oracle/codemp/qcommon/cm_draw.cpp:1156-1189`
pub fn shell_sort<T: Copy>(a: &mut [T], n: c_long, compare: impl Fn(&T, &T) -> c_int) {
    // choose size of "heap"
    let mut h: c_long = 1;
    while h <= n / 9 {
        h = 3 * h + 1;
    }

    // divide and conq.
    while h > 0 {
        let mut i = h;
        while i < n {
            let v = a[i as usize];
            let mut j = i;
            while j >= h && compare(&a[(j - h) as usize], &v) > 0 {
                a[j as usize] = a[(j - h) as usize];
                j -= h;
            }
            a[j as usize] = v;
            i += 1;
        }
        h /= 3;
    }
}
