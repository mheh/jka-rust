#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_int, c_long, c_void};

use native_types::byte;

use crate::cm::cdraw32::CDraw32;
use crate::cm::cm_draw_cpp_consts::{BOTTOM, INT_SHIFT, LEFT, RIGHT, TOP};
use crate::cm::polyedge::POLYEDGE;
use crate::collision_world::CollisionWorld;

/// Raven `code` determines where a point sits relative to the debug clip box.
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
/// Source: `oracle/codemp/qcommon/cm_draw.cpp:1089-1105`
pub fn del_edge(cm: &mut CollisionWorld, i: c_long) {
    let mut j: c_long = 0;

    while j < cm.nact && cm.active[j as usize].i != i {
        j += 1;
    }

    // The edge is not in the active list. This happens at cliprect->top.
    if j >= cm.nact {
        return;
    }

    cm.nact -= 1;
    let count = (cm.nact - j) as usize;
    for k in 0..count {
        cm.active[j as usize + k] = cm.active[j as usize + k + 1];
    }
}

/// Raven `ins_edge` appends edge `i` to the end of the active edge list.
///
/// Source: `oracle/codemp/qcommon/cm_draw.cpp:1107-1143`
pub fn ins_edge(cm: &mut CollisionWorld, i: c_long, y: c_long) {
    let j: c_long = if i < cm.n - 1 { i + 1 } else { 0 };

    let (p_x, p_y, q_x, q_y): (c_long, c_long, c_long, c_long);
    unsafe {
        let pi = cm.pt.offset(i as isize);
        let pj = cm.pt.offset(j as isize);
        if (*pi).y < (*pj).y {
            p_x = (*pi).x;
            p_y = (*pi).y;
            q_x = (*pj).x;
            q_y = (*pj).y;
        } else {
            p_x = (*pj).x;
            p_y = (*pj).y;
            q_x = (*pi).x;
            q_y = (*pi).y;
        }
    }

    // Initialize the x position at the intersection of the edge with scanline y.
    let dx: c_long;
    if (q_y - p_y) != 0 {
        let mut d = (q_x - p_x) * (1 << INT_SHIFT);
        d /= q_y - p_y;
        dx = d;
    } else {
        // Horizontal line.
        dx = 0;
    }

    let nact = cm.nact as usize;
    cm.active[nact].dx = dx;
    cm.active[nact].x = (dx * (y - p_y)) + (p_x << INT_SHIFT);
    cm.active[nact].i = i;
    cm.nact += 1;
}

/// Raven `compare_ind` orders two vertex indices by their y coordinate.
///
/// Source: `oracle/codemp/qcommon/cm_draw.cpp:1146-1149`
pub fn compare_ind(cm: &mut CollisionWorld, u: *mut c_long, v: *mut c_long) -> c_int {
    unsafe {
        let pu = cm.pt.offset(*u as isize);
        let pv = cm.pt.offset(*v as isize);
        if (*pu).y <= (*pv).y {
            -1
        } else {
            1
        }
    }
}

/// Raven `compare_active` orders two active edges by their x position.
///
/// Source: `oracle/codemp/qcommon/cm_draw.cpp:1151-1154`
pub fn compare_active(u: *mut POLYEDGE, v: *mut POLYEDGE) -> c_int {
    unsafe {
        if (*u).x <= (*v).x {
            -1
        } else {
            1
        }
    }
}

/// Raven `shell_sort` sorts an array in place, using the best algorithm for an
/// almost-sorted list.
///
/// Source: `oracle/codemp/qcommon/cm_draw.cpp:1156-1189`
pub fn shell_sort(
    vec: *mut c_void,
    n: c_long,
    siz: c_long,
    compare: extern "C" fn(*mut c_void, *mut c_void) -> c_int,
) {
    let a = vec as *mut byte;
    let mut v = [0u8; 128];

    let mut h: c_long = 1;
    while h <= n / 9 {
        h = 3 * h + 1;
    }

    while h > 0 {
        let mut i = h;
        while i < n {
            unsafe {
                let src = a.offset((i * siz) as isize);
                core::ptr::copy_nonoverlapping(src, v.as_mut_ptr(), siz as usize);
            }

            let mut j = i;
            unsafe {
                while j >= h
                    && compare(
                        a.offset(((j - h) * siz) as isize) as *mut c_void,
                        v.as_mut_ptr() as *mut c_void,
                    ) > 0
                {
                    let dst = a.offset((j * siz) as isize);
                    let src = a.offset(((j - h) * siz) as isize);
                    core::ptr::copy_nonoverlapping(src, dst, siz as usize);
                    j -= h;
                }
                let dst = a.offset((j * siz) as isize);
                core::ptr::copy_nonoverlapping(v.as_ptr(), dst, siz as usize);
            }

            i += 1;
        }
        h /= 3;
    }
}
