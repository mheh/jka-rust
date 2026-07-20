#![allow(non_snake_case, non_upper_case_globals, clippy::too_many_arguments)]

//! `cm_patch.cpp` — patch (bezier curve) collision generation and trace (the
//! C-track TU). Its `_fns` basename is the DESTINATION-rule collision escape:
//! `cm_patch.rs` holds the §F `CCMPatch` terrain class (ruling 40), so the
//! Raven `cm_patch.cpp` free functions land here.
//!
//! State ownership: the file-scope generation scratch (`static int numPlanes`,
//! `static patchPlane_t planes[MAX_PATCH_PLANES]`, the running `c_totalPatchBlocks`
//! statistic) is threaded on `CollisionWorld` per ruling 2; the per-generation
//! `facets` workspace (`Z_Malloc`/`Z_Free`) and the `cGrid_t`/`gridPlanes`
//! scratch are heap-owned locals of `CM_GeneratePatchCollide`/
//! `CM_PatchCollideFromGrid` (Raven's `MAC_STATIC`/`Z_Malloc` transients).
//!
//! Renderer-debug surface dropped (§20 dead surface): Raven's `debugPatchCollide`
//! /`debugFacet`/`debugBlock`/`debugBlockPoints` file statics and `r_debugSurfaceUpdate`
//! cached-cvar writes feed only `CM_DrawDebugSurface`, whose `drawPoly` callback
//! seam has no home in this tree; both `CM_ClearLevelPatches` and the trace
//! debug-capture branches are faithful no-ops in the ported subset, and
//! `CM_DrawDebugSurface` itself is not ported (it has no callers here).

use core::ffi::c_int;
use std::alloc::{alloc_zeroed, Layout};

use mp_qshared::common::mp::qcommon::tags::memtag_t;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::errorParm_t;
use mp_qshared::shared::ha_pref;
use mp_qshared::shared::q_math::{
    _DotProduct, _VectorAdd, _VectorMA, _VectorSubtract, CrossProduct, DotProductRow,
    VectorClear, VectorLengthSquared, VectorNormalize,
};
use mp_qshared::shared::{qboolean, qfalse, qtrue, vec3_t, vec4_t};

use crate::cm::c_grid_t::{cGrid_t, MAX_GRID_SIZE};
use crate::cm::cm_local_consts::SURFACE_CLIP_EPSILON;
use crate::cm::cm_patch_cpp_consts::{DIST_EPSILON, NORMAL_EPSILON, POINT_EPSILON};
use crate::cm::cm_patch_h_consts::{
    MAX_FACETS, MAX_PATCH_PLANES, PLANE_TRI_EPSILON, SUBDIVIDE_DISTANCE, WRAP_POINT_EPSILON,
};
use crate::cm::cm_polylib_consts::{MAX_MAP_BOUNDS, SIDE_BACK, SIDE_FRONT, SIDE_ON};
use crate::cm::facet_t::facet_t;
use crate::cm::patch_collide_s::{patchCollide_s, patchCollide_t};
use crate::cm::patch_plane_t::patchPlane_t;
use crate::cm::trace_work_s::traceWork_t;
use crate::cm_polylib::{
    winding_p, BaseWindingForPlane, ChopWindingInPlace, CopyWinding, FreeWinding,
};
use crate::collision_world::CollisionWorld;
use crate::common::engine_host_view::EngineHostView;
use crate::common::{com_error, com_printf, Common};
use crate::common_fns::{Com_DPrintf, Com_Memcpy, Com_Memset};
use crate::z_memman_pc::{Hunk_Alloc, Z_Free, Z_Malloc};

/// Raven `CM_ClearLevelPatches` — clears the `debugPatchCollide`/`debugFacet`
/// debug pointers.
///
/// These two globals are renderer-debug-only (written under `r_debugSurfaceUpdate`,
/// read only by the deferred `CM_DrawDebugSurface`); both write sites were dropped
/// as dead surface (§20), so this is a faithful no-op in the ported subset.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:108-111`
pub fn CM_ClearLevelPatches(cm: &mut CollisionWorld) {
    let _ = cm;
}

/// Raven `CM_SignbitsForNormal`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:118-128`
fn CM_SignbitsForNormal(normal: vec3_t) -> c_int {
    let mut bits = 0;
    for j in 0..3 {
        if normal[j] < 0.0 {
            bits |= 1 << j;
        }
    }
    bits
}

/// Raven `CM_PlaneFromPoints` — returns false if the triangle is degenerate. The
/// normal points out of the clock for clockwise ordered points.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:138-150`
fn CM_PlaneFromPoints(plane: &mut vec4_t, a: vec3_t, b: vec3_t, c: vec3_t) -> qboolean {
    let mut d1: vec3_t = [0.0; 3];
    let mut d2: vec3_t = [0.0; 3];
    _VectorSubtract(b, a, &mut d1);
    _VectorSubtract(c, a, &mut d2);
    let mut n: vec3_t = [0.0; 3];
    CrossProduct(d2, d1, &mut n);
    if VectorNormalize(&mut n) == 0.0 {
        return qfalse;
    }
    plane[0] = n[0];
    plane[1] = n[1];
    plane[2] = n[2];
    plane[3] = _DotProduct(a, n);
    qtrue
}

/*
================================================================================
GRID SUBDIVISION
================================================================================
*/

/// Raven `CM_NeedsSubdivision` — true if the quadratic curve is not flat enough
/// for collision purposes.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:169-191`
fn CM_NeedsSubdivision(a: vec3_t, b: vec3_t, c: vec3_t) -> qboolean {
    let mut lmid: vec3_t = [0.0; 3];
    let mut cmid: vec3_t = [0.0; 3];

    // calculate the linear midpoint
    for i in 0..3 {
        lmid[i] = 0.5 * (a[i] + c[i]);
    }

    // calculate the exact curve midpoint
    for i in 0..3 {
        cmid[i] = 0.5 * (0.5 * (a[i] + b[i]) + 0.5 * (b[i] + c[i]));
    }

    // see if the curve is far enough away from the linear mid
    let mut delta: vec3_t = [0.0; 3];
    _VectorSubtract(cmid, lmid, &mut delta);
    let dist = VectorLengthSquared(delta);

    (dist >= SUBDIVIDE_DISTANCE * SUBDIVIDE_DISTANCE) as qboolean
}

/// Raven `CM_Subdivide` — a, b, c are control points; the subdivided sequence is
/// a, out1, out2, out3, c.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:201-209`
fn CM_Subdivide(
    a: vec3_t,
    b: vec3_t,
    c: vec3_t,
    out1: &mut vec3_t,
    out2: &mut vec3_t,
    out3: &mut vec3_t,
) {
    for i in 0..3 {
        out1[i] = 0.5 * (a[i] + b[i]);
        out3[i] = 0.5 * (b[i] + c[i]);
        out2[i] = 0.5 * (out1[i] + out3[i]);
    }
}

/// Raven `CM_TransposeGrid` — swaps rows and columns in place.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:218-260`
fn CM_TransposeGrid(grid: &mut cGrid_t) {
    if grid.width > grid.height {
        for i in 0..grid.height as usize {
            for j in (i + 1)..grid.width as usize {
                if (j as c_int) < grid.height {
                    // swap the value
                    let temp = grid.points[i][j];
                    grid.points[i][j] = grid.points[j][i];
                    grid.points[j][i] = temp;
                } else {
                    // just copy
                    grid.points[i][j] = grid.points[j][i];
                }
            }
        }
    } else {
        for i in 0..grid.width as usize {
            for j in (i + 1)..grid.height as usize {
                if (j as c_int) < grid.width {
                    // swap the value
                    let temp = grid.points[j][i];
                    grid.points[j][i] = grid.points[i][j];
                    grid.points[i][j] = temp;
                } else {
                    // just copy
                    grid.points[j][i] = grid.points[i][j];
                }
            }
        }
    }

    let l = grid.width;
    grid.width = grid.height;
    grid.height = l;

    let temp_wrap = grid.wrapWidth;
    grid.wrapWidth = grid.wrapHeight;
    grid.wrapHeight = temp_wrap;
}

/// Raven `CM_SetGridWrapWidth` — if the left and right columns are exactly equal,
/// set `grid->wrapWidth` qtrue.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:269-289`
fn CM_SetGridWrapWidth(grid: &mut cGrid_t) {
    let mut i = 0;
    let mut j;
    while i < grid.height {
        j = 0;
        while j < 3 {
            let d = grid.points[0][i as usize][j as usize]
                - grid.points[(grid.width - 1) as usize][i as usize][j as usize];
            if d < -WRAP_POINT_EPSILON || d > WRAP_POINT_EPSILON {
                break;
            }
            j += 1;
        }
        if j != 3 {
            break;
        }
        i += 1;
    }
    if i == grid.height {
        grid.wrapWidth = qtrue;
    } else {
        grid.wrapWidth = qfalse;
    }
}

/// Raven `CM_SubdivideGridColumns` — adds columns until all approximating points
/// are within `SUBDIVIDE_DISTANCE` of the true curve.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:301-361`
fn CM_SubdivideGridColumns(grid: &mut cGrid_t) {
    let mut i = 0;
    while i < grid.width - 2 {
        // grid->points[i][x] is an interpolating control point
        // grid->points[i+1][x] is an aproximating control point
        // grid->points[i+2][x] is an interpolating control point

        // first see if we can collapse the aproximating collumn away
        let mut j = 0;
        while j < grid.height {
            if CM_NeedsSubdivision(
                grid.points[i as usize][j as usize],
                grid.points[(i + 1) as usize][j as usize],
                grid.points[(i + 2) as usize][j as usize],
            ) != qfalse
            {
                break;
            }
            j += 1;
        }
        if j == grid.height {
            // all of the points were close enough to the linear midpoints
            // that we can collapse the entire column away
            for j in 0..grid.height {
                // remove the column
                for k in (i + 2)..grid.width {
                    grid.points[(k - 1) as usize][j as usize] = grid.points[k as usize][j as usize];
                }
            }

            grid.width -= 1;

            // go to the next curve segment
            i += 1;
            continue;
        }

        // we need to subdivide the curve
        for j in 0..grid.height as usize {
            // save the control points now
            let prev = grid.points[i as usize][j];
            let mid = grid.points[(i + 1) as usize][j];
            let next = grid.points[(i + 2) as usize][j];

            // make room for two additional columns in the grid
            // columns i+1 will be replaced, column i+2 will become i+4
            // i+1, i+2, and i+3 will be generated
            let mut k = grid.width - 1;
            while k > i + 1 {
                grid.points[(k + 2) as usize][j] = grid.points[k as usize][j];
                k -= 1;
            }

            // generate the subdivided points
            let mut out1: vec3_t = [0.0; 3];
            let mut out2: vec3_t = [0.0; 3];
            let mut out3: vec3_t = [0.0; 3];
            CM_Subdivide(prev, mid, next, &mut out1, &mut out2, &mut out3);
            grid.points[(i + 1) as usize][j] = out1;
            grid.points[(i + 2) as usize][j] = out2;
            grid.points[(i + 3) as usize][j] = out3;
        }

        grid.width += 2;

        // the new aproximating point at i+1 may need to be removed
        // or subdivided farther, so don't advance i
    }
}

/// Raven `CM_ComparePoints`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:370-386`
fn CM_ComparePoints(a: vec3_t, b: vec3_t) -> qboolean {
    let mut d = a[0] - b[0];
    if d < -POINT_EPSILON || d > POINT_EPSILON {
        return qfalse;
    }
    d = a[1] - b[1];
    if d < -POINT_EPSILON || d > POINT_EPSILON {
        return qfalse;
    }
    d = a[2] - b[2];
    if d < -POINT_EPSILON || d > POINT_EPSILON {
        return qfalse;
    }
    qtrue
}

/// Raven `CM_RemoveDegenerateColumns` — remove any identical columns.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:395-420`
fn CM_RemoveDegenerateColumns(grid: &mut cGrid_t) {
    let mut i = 0;
    while i < grid.width - 1 {
        let mut j = 0;
        while j < grid.height {
            if CM_ComparePoints(
                grid.points[i as usize][j as usize],
                grid.points[(i + 1) as usize][j as usize],
            ) == qfalse
            {
                break;
            }
            j += 1;
        }

        if j != grid.height {
            i += 1;
            continue; // not degenerate
        }

        for j in 0..grid.height {
            // remove the column
            for k in (i + 2)..grid.width {
                grid.points[(k - 1) as usize][j as usize] = grid.points[k as usize][j as usize];
            }
        }
        grid.width -= 1;

        // check against the next column
        i -= 1;
        i += 1;
    }
}

/*
================================================================================
PATCH COLLIDE GENERATION
================================================================================
*/

/// Raven `CM_PlaneEqual`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:440-467`
fn CM_PlaneEqual(p: &patchPlane_t, plane: &[f32; 4], flipped: &mut c_int) -> c_int {
    if (p.plane[0] - plane[0]).abs() < NORMAL_EPSILON
        && (p.plane[1] - plane[1]).abs() < NORMAL_EPSILON
        && (p.plane[2] - plane[2]).abs() < NORMAL_EPSILON
        && (p.plane[3] - plane[3]).abs() < DIST_EPSILON
    {
        *flipped = qfalse;
        return qtrue;
    }

    let invplane: [f32; 4] = [-plane[0], -plane[1], -plane[2], -plane[3]];

    if (p.plane[0] - invplane[0]).abs() < NORMAL_EPSILON
        && (p.plane[1] - invplane[1]).abs() < NORMAL_EPSILON
        && (p.plane[2] - invplane[2]).abs() < NORMAL_EPSILON
        && (p.plane[3] - invplane[3]).abs() < DIST_EPSILON
    {
        *flipped = qtrue;
        return qtrue;
    }

    qfalse
}

/// Raven `CM_SnapVector`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:469-487`
fn CM_SnapVector(normal: &mut vec3_t) {
    for i in 0..3 {
        if (normal[i] - 1.0).abs() < NORMAL_EPSILON {
            VectorClear(normal);
            normal[i] = 1.0;
            break;
        }
        if (normal[i] - -1.0).abs() < NORMAL_EPSILON {
            VectorClear(normal);
            normal[i] = -1.0;
            break;
        }
    }
}

/// Raven `CM_FindPlane2`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:489-510`
fn CM_FindPlane2(cm: &mut CollisionWorld, plane: &[f32; 4], flipped: &mut c_int) -> c_int {
    // see if the points are close enough to an existing plane
    for i in 0..cm.numPlanes {
        if CM_PlaneEqual(&cm.planes[i as usize], plane, flipped) != 0 {
            return i;
        }
    }

    // add a new plane
    if cm.numPlanes == MAX_PATCH_PLANES as c_int {
        com_error(errorParm_t::ERR_DROP, "MAX_PATCH_PLANES".to_string());
    }

    let n = cm.numPlanes as usize;
    cm.planes[n].plane = *plane;
    cm.planes[n].signbits = CM_SignbitsForNormal([plane[0], plane[1], plane[2]]);

    cm.numPlanes += 1;

    *flipped = qfalse;

    cm.numPlanes - 1
}

/// Raven `CM_FindPlane`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:517-562`
fn CM_FindPlane(cm: &mut CollisionWorld, p1: vec3_t, p2: vec3_t, p3: vec3_t) -> c_int {
    let mut plane: vec4_t = [0.0; 4];
    if CM_PlaneFromPoints(&mut plane, p1, p2, p3) == qfalse {
        return -1;
    }

    // see if the points are close enough to an existing plane
    for i in 0..cm.numPlanes {
        if _DotProduct(
            [plane[0], plane[1], plane[2]],
            [
                cm.planes[i as usize].plane[0],
                cm.planes[i as usize].plane[1],
                cm.planes[i as usize].plane[2],
            ],
        ) < 0.0
        {
            continue; // allow backwards planes?
        }

        let mut d =
            DotProductRow(&cm.planes[i as usize].plane, p1) - cm.planes[i as usize].plane[3];
        if d < -PLANE_TRI_EPSILON || d > PLANE_TRI_EPSILON {
            continue;
        }

        d = DotProductRow(&cm.planes[i as usize].plane, p2) - cm.planes[i as usize].plane[3];
        if d < -PLANE_TRI_EPSILON || d > PLANE_TRI_EPSILON {
            continue;
        }

        d = DotProductRow(&cm.planes[i as usize].plane, p3) - cm.planes[i as usize].plane[3];
        if d < -PLANE_TRI_EPSILON || d > PLANE_TRI_EPSILON {
            continue;
        }

        // found it
        return i;
    }

    // add a new plane
    if cm.numPlanes == MAX_PATCH_PLANES as c_int {
        com_error(errorParm_t::ERR_DROP, "MAX_PATCH_PLANES".to_string());
    }

    let n = cm.numPlanes as usize;
    cm.planes[n].plane = plane;
    cm.planes[n].signbits = CM_SignbitsForNormal([plane[0], plane[1], plane[2]]);

    cm.numPlanes += 1;

    cm.numPlanes - 1
}

/// Raven `CM_PointOnPlaneSide`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:570-590`
fn CM_PointOnPlaneSide(cm: &CollisionWorld, p: vec3_t, planeNum: c_int) -> c_int {
    if planeNum == -1 {
        return SIDE_ON;
    }
    let plane = &cm.planes[planeNum as usize].plane;

    let d = DotProductRow(plane, p) - plane[3];

    if d > PLANE_TRI_EPSILON {
        return SIDE_FRONT;
    }

    if d < -PLANE_TRI_EPSILON {
        return SIDE_BACK;
    }

    SIDE_ON
}

/// Raven `CM_GridPlane`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:592-607`
fn CM_GridPlane(
    common: &mut Common,
    gridPlanes: &[[[c_int; 2]; MAX_GRID_SIZE]; MAX_GRID_SIZE],
    i: c_int,
    j: c_int,
    tri: c_int,
) -> c_int {
    let mut p = gridPlanes[i as usize][j as usize][tri as usize];
    if p != -1 {
        return p;
    }
    p = gridPlanes[i as usize][j as usize][(tri == 0) as usize];
    if p != -1 {
        return p;
    }

    // should never happen
    com_printf(common, "WARNING: CM_GridPlane unresolvable\n");
    -1
}

/// Raven `CM_EdgePlaneNum`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:615-667`
fn CM_EdgePlaneNum(
    common: &mut Common,
    cm: &mut CollisionWorld,
    grid: &cGrid_t,
    gridPlanes: &[[[c_int; 2]; MAX_GRID_SIZE]; MAX_GRID_SIZE],
    i: c_int,
    j: c_int,
    k: c_int,
) -> c_int {
    let mut up: vec3_t = [0.0; 3];

    match k {
        0 => {
            // top border
            let p1 = grid.points[i as usize][j as usize];
            let p2 = grid.points[(i + 1) as usize][j as usize];
            let p = CM_GridPlane(common, gridPlanes, i, j, 0);
            let pl = cm.planes[p as usize].plane;
            _VectorMA(p1, 4.0, [pl[0], pl[1], pl[2]], &mut up);
            CM_FindPlane(cm, p1, p2, up)
        }
        2 => {
            // bottom border
            let p1 = grid.points[i as usize][(j + 1) as usize];
            let p2 = grid.points[(i + 1) as usize][(j + 1) as usize];
            let p = CM_GridPlane(common, gridPlanes, i, j, 1);
            let pl = cm.planes[p as usize].plane;
            _VectorMA(p1, 4.0, [pl[0], pl[1], pl[2]], &mut up);
            CM_FindPlane(cm, p2, p1, up)
        }
        3 => {
            // left border
            let p1 = grid.points[i as usize][j as usize];
            let p2 = grid.points[i as usize][(j + 1) as usize];
            let p = CM_GridPlane(common, gridPlanes, i, j, 1);
            let pl = cm.planes[p as usize].plane;
            _VectorMA(p1, 4.0, [pl[0], pl[1], pl[2]], &mut up);
            CM_FindPlane(cm, p2, p1, up)
        }
        1 => {
            // right border
            let p1 = grid.points[(i + 1) as usize][j as usize];
            let p2 = grid.points[(i + 1) as usize][(j + 1) as usize];
            let p = CM_GridPlane(common, gridPlanes, i, j, 0);
            let pl = cm.planes[p as usize].plane;
            _VectorMA(p1, 4.0, [pl[0], pl[1], pl[2]], &mut up);
            CM_FindPlane(cm, p1, p2, up)
        }
        4 => {
            // diagonal out of triangle 0
            let p1 = grid.points[(i + 1) as usize][(j + 1) as usize];
            let p2 = grid.points[i as usize][j as usize];
            let p = CM_GridPlane(common, gridPlanes, i, j, 0);
            let pl = cm.planes[p as usize].plane;
            _VectorMA(p1, 4.0, [pl[0], pl[1], pl[2]], &mut up);
            CM_FindPlane(cm, p1, p2, up)
        }
        5 => {
            // diagonal out of triangle 1
            let p1 = grid.points[i as usize][j as usize];
            let p2 = grid.points[(i + 1) as usize][(j + 1) as usize];
            let p = CM_GridPlane(common, gridPlanes, i, j, 1);
            let pl = cm.planes[p as usize].plane;
            _VectorMA(p1, 4.0, [pl[0], pl[1], pl[2]], &mut up);
            CM_FindPlane(cm, p1, p2, up)
        }
        _ => {
            com_error(errorParm_t::ERR_DROP, "CM_EdgePlaneNum: bad k".to_string());
        }
    }
}

/// Raven `CM_SetBorderInward`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:675-746`
fn CM_SetBorderInward(
    common: &mut Common,
    cm: &CollisionWorld,
    facet: *mut facet_t,
    grid: &cGrid_t,
    i: c_int,
    j: c_int,
    which: c_int,
) {
    unsafe {
        let mut points: [vec3_t; 4] = [[0.0; 3]; 4];
        let num_points: c_int;

        match which {
            -1 => {
                points[0] = grid.points[i as usize][j as usize];
                points[1] = grid.points[(i + 1) as usize][j as usize];
                points[2] = grid.points[(i + 1) as usize][(j + 1) as usize];
                points[3] = grid.points[i as usize][(j + 1) as usize];
                num_points = 4;
            }
            0 => {
                points[0] = grid.points[i as usize][j as usize];
                points[1] = grid.points[(i + 1) as usize][j as usize];
                points[2] = grid.points[(i + 1) as usize][(j + 1) as usize];
                num_points = 3;
            }
            1 => {
                points[0] = grid.points[(i + 1) as usize][(j + 1) as usize];
                points[1] = grid.points[i as usize][(j + 1) as usize];
                points[2] = grid.points[i as usize][j as usize];
                num_points = 3;
            }
            _ => {
                com_error(
                    errorParm_t::ERR_FATAL,
                    "CM_SetBorderInward: bad parameter".to_string(),
                );
            }
        }

        for k in 0..(*facet).numBorders {
            let mut front = 0;
            let mut back = 0;

            for l in 0..num_points {
                let side =
                    CM_PointOnPlaneSide(cm, points[l as usize], (*facet).borderPlanes[k as usize]);
                if side == SIDE_FRONT {
                    front += 1;
                }
                if side == SIDE_BACK {
                    back += 1;
                }
            }

            if front != 0 && back == 0 {
                (*facet).borderInward[k as usize] = qtrue;
            } else if back != 0 && front == 0 {
                (*facet).borderInward[k as usize] = qfalse;
            } else if front == 0 && back == 0 {
                // flat side border
                (*facet).borderPlanes[k as usize] = -1;
            } else {
                // bisecting side border
                Com_DPrintf(common, "WARNING: CM_SetBorderInward: mixed plane sides\n");
                (*facet).borderInward[k as usize] = qfalse;
                // §20: Raven's `debugBlock`/`debugBlockPoints` capture here feeds
                // only the dropped `CM_DrawDebugSurface`; omitted as dead surface.
            }
        }
    }
}

/// Raven `CM_ValidateFacet` — if the facet isn't bounded by its borders, we
/// screwed up.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:755-800`
fn CM_ValidateFacet(view: &mut EngineHostView, facet: *mut facet_t) -> qboolean {
    unsafe {
        if (*facet).surfacePlane == -1 {
            return qfalse;
        }

        let mut plane = view.cm.planes[(*facet).surfacePlane as usize].plane;
        let mut w = BaseWindingForPlane(view, [plane[0], plane[1], plane[2]], plane[3]);
        let mut j = 0;
        while j < (*facet).numBorders && !w.is_null() {
            if (*facet).borderPlanes[j as usize] == -1 {
                FreeWinding(view.common, view.cm, w);
                return qfalse;
            }
            plane = view.cm.planes[(*facet).borderPlanes[j as usize] as usize].plane;
            if (*facet).borderInward[j as usize] == 0 {
                plane = [-plane[0], -plane[1], -plane[2], -plane[3]];
            }
            ChopWindingInPlace(view, &mut w, [plane[0], plane[1], plane[2]], plane[3], 0.1);
            j += 1;
        }

        if w.is_null() {
            return qfalse; // winding was completely chopped away
        }

        // see if the facet is unreasonably large
        // WindingBounds inlined (min/max over the winding points): the crate's
        // `cm_polylib::WindingBounds` takes by-value out-params (a flagged shape
        // mismatch), so its result cannot return here.
        let mut mins: vec3_t = [MAX_MAP_BOUNDS as f32; 3];
        let mut maxs: vec3_t = [-(MAX_MAP_BOUNDS as f32); 3];
        for p in 0..(*w).numpoints {
            for c in 0..3usize {
                let v = (*winding_p(w, p as usize))[c];
                if v < mins[c] {
                    mins[c] = v;
                }
                if v > maxs[c] {
                    maxs[c] = v;
                }
            }
        }
        FreeWinding(view.common, view.cm, w);

        for j in 0..3usize {
            if maxs[j] - mins[j] > MAX_MAP_BOUNDS as f32 {
                return qfalse; // we must be missing a plane
            }
            if mins[j] >= MAX_MAP_BOUNDS as f32 {
                return qfalse;
            }
            if maxs[j] <= -(MAX_MAP_BOUNDS as f32) {
                return qfalse;
            }
        }
        qtrue // winding is fine
    }
}

/// Raven `CM_AddFacetBevels`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:807-968`
fn CM_AddFacetBevels(view: &mut EngineHostView, facet: *mut facet_t) {
    unsafe {
        let mut plane = view.cm.planes[(*facet).surfacePlane as usize].plane;

        let mut w = BaseWindingForPlane(view, [plane[0], plane[1], plane[2]], plane[3]);
        let mut j = 0;
        while j < (*facet).numBorders && !w.is_null() {
            if (*facet).borderPlanes[j as usize] == (*facet).surfacePlane {
                j += 1;
                continue;
            }
            plane = view.cm.planes[(*facet).borderPlanes[j as usize] as usize].plane;

            if (*facet).borderInward[j as usize] == 0 {
                plane = [-plane[0], -plane[1], -plane[2], -plane[3]];
            }

            ChopWindingInPlace(view, &mut w, [plane[0], plane[1], plane[2]], plane[3], 0.1);
            j += 1;
        }
        if w.is_null() {
            return;
        }

        // WindingBounds inlined (see CM_ValidateFacet).
        let mut mins: vec3_t = [MAX_MAP_BOUNDS as f32; 3];
        let mut maxs: vec3_t = [-(MAX_MAP_BOUNDS as f32); 3];
        for p in 0..(*w).numpoints {
            for c in 0..3usize {
                let v = (*winding_p(w, p as usize))[c];
                if v < mins[c] {
                    mins[c] = v;
                }
                if v > maxs[c] {
                    maxs[c] = v;
                }
            }
        }

        // add the axial planes
        let mut flipped: c_int = 0;
        let mut order = 0;
        for axis in 0..3usize {
            let mut dir = -1;
            while dir <= 1 {
                let mut aplane: vec4_t = [0.0; 4];
                aplane[axis] = dir as f32;
                if dir == 1 {
                    aplane[3] = maxs[axis];
                } else {
                    aplane[3] = -mins[axis];
                }
                // if it's the surface plane
                if CM_PlaneEqual(
                    &view.cm.planes[(*facet).surfacePlane as usize],
                    &aplane,
                    &mut flipped,
                ) != 0
                {
                    dir += 2;
                    order += 1;
                    continue;
                }
                // see if the plane is allready present
                let mut i = 0;
                while i < (*facet).numBorders {
                    if CM_PlaneEqual(
                        &view.cm.planes[(*facet).borderPlanes[i as usize] as usize],
                        &aplane,
                        &mut flipped,
                    ) != 0
                    {
                        break;
                    }
                    i += 1;
                }

                if i == (*facet).numBorders {
                    if (*facet).numBorders > 4 + 6 + 16 {
                        com_printf(view.common, "ERROR: too many bevels\n");
                    }
                    (*facet).borderPlanes[(*facet).numBorders as usize] =
                        CM_FindPlane2(view.cm, &aplane, &mut flipped);
                    (*facet).borderNoAdjust[(*facet).numBorders as usize] = qfalse;
                    (*facet).borderInward[(*facet).numBorders as usize] = flipped;
                    (*facet).numBorders += 1;
                }
                dir += 2;
                order += 1;
            }
        }
        let _ = order;

        // add the edge bevels
        // test the non-axial plane edges
        let mut j = 0;
        while j < (*w).numpoints {
            let k = (j + 1) % (*w).numpoints;
            let mut vec: vec3_t = [0.0; 3];
            _VectorSubtract(
                *winding_p(w, j as usize),
                *winding_p(w, k as usize),
                &mut vec,
            );
            // if it's a degenerate edge
            if VectorNormalize(&mut vec) < 0.5 {
                j += 1;
                continue;
            }
            CM_SnapVector(&mut vec);
            let mut kk = 0;
            while kk < 3 {
                if vec[kk] == -1.0 || vec[kk] == 1.0 {
                    break; // axial
                }
                kk += 1;
            }
            if kk < 3 {
                j += 1;
                continue; // only test non-axial edges
            }

            // try the six possible slanted axials from this edge
            for axis in 0..3usize {
                let mut dir = -1;
                while dir <= 1 {
                    // construct a plane
                    let mut vec2: vec3_t = [0.0; 3];
                    vec2[axis] = dir as f32;
                    let mut eplane: vec4_t = [0.0; 4];
                    let mut n: vec3_t = [0.0; 3];
                    CrossProduct(vec, vec2, &mut n);
                    if VectorNormalize(&mut n) < 0.5 {
                        dir += 2;
                        continue;
                    }
                    eplane[0] = n[0];
                    eplane[1] = n[1];
                    eplane[2] = n[2];
                    eplane[3] = _DotProduct(*winding_p(w, j as usize), n);

                    // if all the points of the facet winding are
                    // behind this plane, it is a proper edge bevel
                    let mut l = 0;
                    while l < (*w).numpoints {
                        let d = _DotProduct(*winding_p(w, l as usize), n) - eplane[3];
                        if d > 0.1 {
                            break; // point in front
                        }
                        l += 1;
                    }
                    if l < (*w).numpoints {
                        dir += 2;
                        continue;
                    }

                    // if it's the surface plane
                    if CM_PlaneEqual(
                        &view.cm.planes[(*facet).surfacePlane as usize],
                        &eplane,
                        &mut flipped,
                    ) != 0
                    {
                        dir += 2;
                        continue;
                    }
                    // see if the plane is allready present
                    let mut i = 0;
                    while i < (*facet).numBorders {
                        if CM_PlaneEqual(
                            &view.cm.planes[(*facet).borderPlanes[i as usize] as usize],
                            &eplane,
                            &mut flipped,
                        ) != 0
                        {
                            break;
                        }
                        i += 1;
                    }

                    if i == (*facet).numBorders {
                        if (*facet).numBorders > 4 + 6 + 16 {
                            com_printf(view.common, "ERROR: too many bevels\n");
                        }
                        (*facet).borderPlanes[(*facet).numBorders as usize] =
                            CM_FindPlane2(view.cm, &eplane, &mut flipped);

                        let mut kchk = 0;
                        while kchk < (*facet).numBorders {
                            if (*facet).borderPlanes[(*facet).numBorders as usize]
                                == (*facet).borderPlanes[kchk as usize]
                            {
                                com_printf(view.common, "WARNING: bevel plane already used\n");
                            }
                            kchk += 1;
                        }

                        (*facet).borderNoAdjust[(*facet).numBorders as usize] = qfalse;
                        (*facet).borderInward[(*facet).numBorders as usize] = flipped;

                        let mut w2 = CopyWinding(view, w);
                        let mut newplane = view.cm.planes
                            [(*facet).borderPlanes[(*facet).numBorders as usize] as usize]
                            .plane;
                        if (*facet).borderInward[(*facet).numBorders as usize] == 0 {
                            newplane = [-newplane[0], -newplane[1], -newplane[2], -newplane[3]];
                        }
                        ChopWindingInPlace(
                            view,
                            &mut w2,
                            [newplane[0], newplane[1], newplane[2]],
                            newplane[3],
                            0.1,
                        );
                        if w2.is_null() {
                            Com_DPrintf(
                                view.common,
                                "WARNING: CM_AddFacetBevels... invalid bevel\n",
                            );
                            dir += 2;
                            continue;
                        } else {
                            FreeWinding(view.common, view.cm, w2);
                        }

                        (*facet).numBorders += 1;
                        // already got a bevel
                        // break;
                    }
                    dir += 2;
                }
            }
            j += 1;
        }
        FreeWinding(view.common, view.cm, w);

        // add opposite plane
        (*facet).borderPlanes[(*facet).numBorders as usize] = (*facet).surfacePlane;
        (*facet).borderNoAdjust[(*facet).numBorders as usize] = qfalse;
        (*facet).borderInward[(*facet).numBorders as usize] = qtrue;
        (*facet).numBorders += 1;
    }
}

/// Raven `edgeName_t`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:971-976`
const EN_TOP: usize = 0;
const EN_RIGHT: usize = 1;
const EN_BOTTOM: usize = 2;
const EN_LEFT: usize = 3;

/// Raven `CM_PatchCollideFromGrid`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:983-1150`
fn CM_PatchCollideFromGrid(view: &mut EngineHostView, grid: &cGrid_t, pf: *mut patchCollide_t) {
    unsafe {
        let mut borders = [0 as c_int; 4];
        let mut no_adjust = [0 as c_int; 4];

        // gridPlanes[MAX_GRID_SIZE][MAX_GRID_SIZE][2] — heap-owned (Raven's
        // MAC_STATIC/stack array), zeroed (unwritten cells are never read).
        let grid_planes: Box<[[[c_int; 2]; MAX_GRID_SIZE]; MAX_GRID_SIZE]> = {
            let raw = alloc_zeroed(Layout::new::<[[[c_int; 2]; MAX_GRID_SIZE]; MAX_GRID_SIZE]>())
                as *mut [[[c_int; 2]; MAX_GRID_SIZE]; MAX_GRID_SIZE];
            Box::from_raw(raw)
        };
        let mut grid_planes = grid_planes;

        let facets = Z_Malloc(
            view,
            (MAX_FACETS * core::mem::size_of::<facet_t>()) as c_int,
            memtag_t::TAG_TEMP_WORKSPACE,
            qfalse,
            4,
        ) as *mut facet_t;

        view.cm.numPlanes = 0;
        let mut num_facets: c_int = 0;

        // find the planes for each triangle of the grid
        for i in 0..grid.width - 1 {
            for j in 0..grid.height - 1 {
                let p1 = grid.points[i as usize][j as usize];
                let p2 = grid.points[(i + 1) as usize][j as usize];
                let p3 = grid.points[(i + 1) as usize][(j + 1) as usize];
                grid_planes[i as usize][j as usize][0] = CM_FindPlane(view.cm, p1, p2, p3);

                let p1 = grid.points[(i + 1) as usize][(j + 1) as usize];
                let p2 = grid.points[i as usize][(j + 1) as usize];
                let p3 = grid.points[i as usize][j as usize];
                grid_planes[i as usize][j as usize][1] = CM_FindPlane(view.cm, p1, p2, p3);
            }
        }

        // create the borders for each facet
        for i in 0..grid.width - 1 {
            for j in 0..grid.height - 1 {
                borders[EN_TOP] = -1;
                if j > 0 {
                    borders[EN_TOP] = grid_planes[i as usize][(j - 1) as usize][1];
                } else if grid.wrapHeight != qfalse {
                    borders[EN_TOP] = grid_planes[i as usize][(grid.height - 2) as usize][1];
                }
                no_adjust[EN_TOP] =
                    (borders[EN_TOP] == grid_planes[i as usize][j as usize][0]) as c_int;
                if borders[EN_TOP] == -1 || no_adjust[EN_TOP] != 0 {
                    borders[EN_TOP] =
                        CM_EdgePlaneNum(view.common, view.cm, grid, &grid_planes, i, j, 0);
                }

                borders[EN_BOTTOM] = -1;
                if j < grid.height - 2 {
                    borders[EN_BOTTOM] = grid_planes[i as usize][(j + 1) as usize][0];
                } else if grid.wrapHeight != qfalse {
                    borders[EN_BOTTOM] = grid_planes[i as usize][0][0];
                }
                no_adjust[EN_BOTTOM] =
                    (borders[EN_BOTTOM] == grid_planes[i as usize][j as usize][1]) as c_int;
                if borders[EN_BOTTOM] == -1 || no_adjust[EN_BOTTOM] != 0 {
                    borders[EN_BOTTOM] =
                        CM_EdgePlaneNum(view.common, view.cm, grid, &grid_planes, i, j, 2);
                }

                borders[EN_LEFT] = -1;
                if i > 0 {
                    borders[EN_LEFT] = grid_planes[(i - 1) as usize][j as usize][0];
                } else if grid.wrapWidth != qfalse {
                    borders[EN_LEFT] = grid_planes[(grid.width - 2) as usize][j as usize][0];
                }
                no_adjust[EN_LEFT] =
                    (borders[EN_LEFT] == grid_planes[i as usize][j as usize][1]) as c_int;
                if borders[EN_LEFT] == -1 || no_adjust[EN_LEFT] != 0 {
                    borders[EN_LEFT] =
                        CM_EdgePlaneNum(view.common, view.cm, grid, &grid_planes, i, j, 3);
                }

                borders[EN_RIGHT] = -1;
                if i < grid.width - 2 {
                    borders[EN_RIGHT] = grid_planes[(i + 1) as usize][j as usize][1];
                } else if grid.wrapWidth != qfalse {
                    borders[EN_RIGHT] = grid_planes[0][j as usize][1];
                }
                no_adjust[EN_RIGHT] =
                    (borders[EN_RIGHT] == grid_planes[i as usize][j as usize][0]) as c_int;
                if borders[EN_RIGHT] == -1 || no_adjust[EN_RIGHT] != 0 {
                    borders[EN_RIGHT] =
                        CM_EdgePlaneNum(view.common, view.cm, grid, &grid_planes, i, j, 1);
                }

                if num_facets == MAX_FACETS as c_int {
                    com_error(errorParm_t::ERR_DROP, "MAX_FACETS".to_string());
                }
                let facet = facets.add(num_facets as usize);
                Com_Memset(facet as *mut (), 0, core::mem::size_of::<facet_t>());

                if grid_planes[i as usize][j as usize][0] == grid_planes[i as usize][j as usize][1]
                {
                    if grid_planes[i as usize][j as usize][0] == -1 {
                        continue; // degenrate
                    }
                    (*facet).surfacePlane = grid_planes[i as usize][j as usize][0];
                    (*facet).numBorders = 4;
                    (*facet).borderPlanes[0] = borders[EN_TOP];
                    (*facet).borderNoAdjust[0] = no_adjust[EN_TOP];
                    (*facet).borderPlanes[1] = borders[EN_RIGHT];
                    (*facet).borderNoAdjust[1] = no_adjust[EN_RIGHT];
                    (*facet).borderPlanes[2] = borders[EN_BOTTOM];
                    (*facet).borderNoAdjust[2] = no_adjust[EN_BOTTOM];
                    (*facet).borderPlanes[3] = borders[EN_LEFT];
                    (*facet).borderNoAdjust[3] = no_adjust[EN_LEFT];
                    CM_SetBorderInward(view.common, view.cm, facet, grid, i, j, -1);
                    if CM_ValidateFacet(view, facet) != qfalse {
                        CM_AddFacetBevels(view, facet);
                        num_facets += 1;
                    }
                } else {
                    // two seperate triangles
                    (*facet).surfacePlane = grid_planes[i as usize][j as usize][0];
                    (*facet).numBorders = 3;
                    (*facet).borderPlanes[0] = borders[EN_TOP];
                    (*facet).borderNoAdjust[0] = no_adjust[EN_TOP];
                    (*facet).borderPlanes[1] = borders[EN_RIGHT];
                    (*facet).borderNoAdjust[1] = no_adjust[EN_RIGHT];
                    (*facet).borderPlanes[2] = grid_planes[i as usize][j as usize][1];
                    if (*facet).borderPlanes[2] == -1 {
                        (*facet).borderPlanes[2] = borders[EN_BOTTOM];
                        if (*facet).borderPlanes[2] == -1 {
                            (*facet).borderPlanes[2] =
                                CM_EdgePlaneNum(view.common, view.cm, grid, &grid_planes, i, j, 4);
                        }
                    }
                    CM_SetBorderInward(view.common, view.cm, facet, grid, i, j, 0);
                    if CM_ValidateFacet(view, facet) != qfalse {
                        CM_AddFacetBevels(view, facet);
                        num_facets += 1;
                    }

                    if num_facets == MAX_FACETS as c_int {
                        com_error(errorParm_t::ERR_DROP, "MAX_FACETS".to_string());
                    }
                    let facet = facets.add(num_facets as usize);
                    Com_Memset(facet as *mut (), 0, core::mem::size_of::<facet_t>());

                    (*facet).surfacePlane = grid_planes[i as usize][j as usize][1];
                    (*facet).numBorders = 3;
                    (*facet).borderPlanes[0] = borders[EN_BOTTOM];
                    (*facet).borderNoAdjust[0] = no_adjust[EN_BOTTOM];
                    (*facet).borderPlanes[1] = borders[EN_LEFT];
                    (*facet).borderNoAdjust[1] = no_adjust[EN_LEFT];
                    (*facet).borderPlanes[2] = grid_planes[i as usize][j as usize][0];
                    if (*facet).borderPlanes[2] == -1 {
                        (*facet).borderPlanes[2] = borders[EN_TOP];
                        if (*facet).borderPlanes[2] == -1 {
                            (*facet).borderPlanes[2] =
                                CM_EdgePlaneNum(view.common, view.cm, grid, &grid_planes, i, j, 5);
                        }
                    }
                    CM_SetBorderInward(view.common, view.cm, facet, grid, i, j, 1);
                    if CM_ValidateFacet(view, facet) != qfalse {
                        CM_AddFacetBevels(view, facet);
                        num_facets += 1;
                    }
                }
            }
        }

        // copy the results out
        (*pf).numPlanes = view.cm.numPlanes;
        (*pf).numFacets = num_facets;
        if num_facets != 0 {
            (*pf).facets = Hunk_Alloc(
                view,
                (num_facets as usize * core::mem::size_of::<facet_t>()) as c_int,
                ha_pref::h_high,
            ) as *mut facet_t;
            Com_Memcpy(
                (*pf).facets as *mut (),
                facets as *const (),
                num_facets as usize * core::mem::size_of::<facet_t>(),
            );
        } else {
            (*pf).facets = core::ptr::null_mut();
        }
        (*pf).planes = Hunk_Alloc(
            view,
            (view.cm.numPlanes as usize * core::mem::size_of::<patchPlane_t>()) as c_int,
            ha_pref::h_high,
        ) as *mut patchPlane_t;
        Com_Memcpy(
            (*pf).planes as *mut (),
            view.cm.planes.as_ptr() as *const (),
            view.cm.numPlanes as usize * core::mem::size_of::<patchPlane_t>(),
        );

        Z_Free(view.common, facets as *mut ());
        let _ = &mut grid_planes;
    }
}

/// Raven `CM_GeneratePatchCollide` — creates the internal structure used to
/// perform collision detection with a patch mesh. `points` is packed as
/// concatenated rows.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:1163-1229`
pub fn CM_GeneratePatchCollide(
    view: &mut EngineHostView,
    width: c_int,
    height: c_int,
    points: *mut vec3_t,
) -> *mut patchCollide_s {
    unsafe {
        if width <= 2 || height <= 2 || points.is_null() {
            com_error(
                errorParm_t::ERR_DROP,
                format!("CM_GeneratePatchFacets: bad parameters: ({width}, {height}, {points:p})"),
            );
        }

        if (width & 1) == 0 || (height & 1) == 0 {
            com_error(
                errorParm_t::ERR_DROP,
                "CM_GeneratePatchFacets: even sizes are invalid for quadratic meshes".to_string(),
            );
        }

        if width > MAX_GRID_SIZE as c_int || height > MAX_GRID_SIZE as c_int {
            com_error(
                errorParm_t::ERR_DROP,
                "CM_GeneratePatchFacets: source is > MAX_GRID_SIZE".to_string(),
            );
        }

        // build a grid — heap-owned (Raven's MAC_STATIC cGrid_t on the stack).
        let mut grid: Box<cGrid_t> = {
            let raw = alloc_zeroed(Layout::new::<cGrid_t>()) as *mut cGrid_t;
            Box::from_raw(raw)
        };
        grid.width = width;
        grid.height = height;
        grid.wrapWidth = qfalse;
        grid.wrapHeight = qfalse;
        for i in 0..width {
            for j in 0..height {
                grid.points[i as usize][j as usize] = *points.add((j * width + i) as usize);
            }
        }

        // subdivide the grid
        CM_SetGridWrapWidth(&mut grid);
        CM_SubdivideGridColumns(&mut grid);
        CM_RemoveDegenerateColumns(&mut grid);

        CM_TransposeGrid(&mut grid);

        CM_SetGridWrapWidth(&mut grid);
        CM_SubdivideGridColumns(&mut grid);
        CM_RemoveDegenerateColumns(&mut grid);

        // we now have a grid of points exactly on the curve
        // the aproximate surface defined by these points will be
        // collided against
        let pf = Hunk_Alloc(
            view,
            core::mem::size_of::<patchCollide_s>() as c_int,
            ha_pref::h_high,
        ) as *mut patchCollide_s;

        // ClearBounds
        (*pf).bounds[0] = [99999.0; 3];
        (*pf).bounds[1] = [-99999.0; 3];
        for i in 0..grid.width {
            for j in 0..grid.height {
                // AddPointToBounds
                let v = grid.points[i as usize][j as usize];
                for c in 0..3usize {
                    if v[c] < (*pf).bounds[0][c] {
                        (*pf).bounds[0][c] = v[c];
                    }
                    if v[c] > (*pf).bounds[1][c] {
                        (*pf).bounds[1][c] = v[c];
                    }
                }
            }
        }

        view.cm.c_totalPatchBlocks += (grid.width - 1) * (grid.height - 1);

        // generate a bsp tree for the surface
        CM_PatchCollideFromGrid(view, &grid, pf);

        // expand by one unit for epsilon purposes
        (*pf).bounds[0][0] -= 1.0;
        (*pf).bounds[0][1] -= 1.0;
        (*pf).bounds[0][2] -= 1.0;

        (*pf).bounds[1][0] += 1.0;
        (*pf).bounds[1][1] += 1.0;
        (*pf).bounds[1][2] += 1.0;

        pf
    }
}

/*
================================================================================
TRACE TESTING
================================================================================
*/

/// Raven `CM_TracePointThroughPatchCollide` — special case for point traces
/// because the patch collide "brushes" have no volume.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:1246-1339`
fn CM_TracePointThroughPatchCollide(
    common: &Common,
    cm: &CollisionWorld,
    tw: *mut traceWork_t,
    trace: &mut trace_t,
    pc: *const patchCollide_s,
) {
    unsafe {
        // §19: guard the not-yet-registered `cm_playerCurveClip` cvar (Raven
        // dereferences it unconditionally) — a `None` handle reads as 0 here.
        if cm.cm_playerCurveClip.is_none()
            || common.cvar(cm.cm_playerCurveClip).integer == 0
            || (*tw).isPoint == qfalse
        {
            return;
        }

        let mut front_facing = [qfalse; MAX_PATCH_PLANES];
        let mut intersection = [0.0f32; MAX_PATCH_PLANES];

        // determine the trace's relationship to all planes
        let mut planes = (*pc).planes;
        for i in 0..(*pc).numPlanes {
            let offset =
                DotProductRow(&(*planes).plane, (*tw).offsets[(*planes).signbits as usize]);
            let d1 = DotProductRow(&(*planes).plane, (*tw).start) - (*planes).plane[3] + offset;
            let d2 = DotProductRow(&(*planes).plane, (*tw).end) - (*planes).plane[3] + offset;
            if d1 <= 0.0 {
                front_facing[i as usize] = qfalse;
            } else {
                front_facing[i as usize] = qtrue;
            }
            if d1 == d2 {
                intersection[i as usize] = 99999.0;
            } else {
                intersection[i as usize] = d1 / (d1 - d2);
                if intersection[i as usize] <= 0.0 {
                    intersection[i as usize] = 99999.0;
                }
            }
            planes = planes.add(1);
        }

        // see if any of the surface planes are intersected
        let mut facet = (*pc).facets;
        for _i in 0..(*pc).numFacets {
            if front_facing[(*facet).surfacePlane as usize] == qfalse {
                facet = facet.add(1);
                continue;
            }
            let intersect = intersection[(*facet).surfacePlane as usize];
            if intersect < 0.0 {
                facet = facet.add(1);
                continue; // surface is behind the starting point
            }
            if intersect > trace.fraction {
                facet = facet.add(1);
                continue; // already hit something closer
            }
            let mut j = 0;
            while j < (*facet).numBorders {
                let k = (*facet).borderPlanes[j as usize];
                if (front_facing[k as usize] != qfalse) ^ ((*facet).borderInward[j as usize] != 0) {
                    if intersection[k as usize] > intersect {
                        break;
                    }
                } else if intersection[k as usize] < intersect {
                    break;
                }
                j += 1;
            }
            if j == (*facet).numBorders {
                // we hit this facet
                // §20: Raven's `debugPatchCollide`/`debugFacet` capture (gated on
                // the cached `r_debugSurfaceUpdate` cvar) is dropped dead surface.
                let planes = (*pc).planes.add((*facet).surfacePlane as usize);

                // calculate intersection with a slight pushoff
                let offset =
                    DotProductRow(&(*planes).plane, (*tw).offsets[(*planes).signbits as usize]);
                let d1 = DotProductRow(&(*planes).plane, (*tw).start) - (*planes).plane[3] + offset;
                let d2 = DotProductRow(&(*planes).plane, (*tw).end) - (*planes).plane[3] + offset;
                trace.fraction = (d1 - SURFACE_CLIP_EPSILON) / (d1 - d2);

                if trace.fraction < 0.0 {
                    trace.fraction = 0.0;
                }

                trace.plane.normal = [(*planes).plane[0], (*planes).plane[1], (*planes).plane[2]];
                trace.plane.dist = (*planes).plane[3];
            }
            facet = facet.add(1);
        }
    }
}

/// Raven `CM_CheckFacetPlane`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:1346-1385`
fn CM_CheckFacetPlane(
    plane: &[f32; 4],
    start: vec3_t,
    end: vec3_t,
    enterFrac: &mut f32,
    leaveFrac: &mut f32,
    hit: &mut c_int,
) -> c_int {
    *hit = qfalse;

    let d1 = DotProductRow(plane, start) - plane[3];
    let d2 = DotProductRow(plane, end) - plane[3];

    // if completely in front of face, no intersection with the entire facet
    if d1 > 0.0 && (d2 >= SURFACE_CLIP_EPSILON || d2 >= d1) {
        return qfalse;
    }

    // if it doesn't cross the plane, the plane isn't relevent
    if d1 <= 0.0 && d2 <= 0.0 {
        return qtrue;
    }

    // crosses face
    if d1 > d2 {
        // enter
        let mut f = (d1 - SURFACE_CLIP_EPSILON) / (d1 - d2);
        if f < 0.0 {
            f = 0.0;
        }
        // always favor previous plane hits and thus also the surface plane hit
        if f > *enterFrac {
            *enterFrac = f;
            *hit = qtrue;
        }
    } else {
        // leave
        let mut f = (d1 + SURFACE_CLIP_EPSILON) / (d1 - d2);
        if f > 1.0 {
            f = 1.0;
        }
        if f < *leaveFrac {
            *leaveFrac = f;
        }
    }
    qtrue
}

/// Raven `CM_TraceThroughPatchCollide` — sweep a trace through a patch's facets.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:1392-1527`
pub fn CM_TraceThroughPatchCollide(
    view: &mut EngineHostView,
    tw: *mut traceWork_t,
    trace: &mut trace_t,
    pc: *const patchCollide_s,
) {
    // §20: only `cm` (for `cm_playerCurveClip`) is live in the ported trace
    // path; the dropped debug-capture removed the other receivers' uses.
    unsafe {
        // I'm not sure if test is strictly correct.  Are all
        // bboxes axis aligned?  Do I care?  It seems to work
        // good enough...
        for i in 0..3usize {
            if (*tw).bounds[0][i] > (*pc).bounds[1][i] || (*tw).bounds[1][i] < (*pc).bounds[0][i] {
                return;
            }
        }

        if (*tw).isPoint != qfalse {
            CM_TracePointThroughPatchCollide(view.common, view.cm, tw, trace, pc);
            return;
        }

        let mut facet = (*pc).facets;
        for _i in 0..(*pc).numFacets {
            let mut enter_frac = -1.0f32;
            let mut leave_frac = 1.0f32;
            let mut hitnum: c_int = -1;

            let mut planes = (*pc).planes.add((*facet).surfacePlane as usize);
            let mut plane: vec4_t = [
                (*planes).plane[0],
                (*planes).plane[1],
                (*planes).plane[2],
                (*planes).plane[3],
            ];
            let mut startp: vec3_t;
            let mut endp: vec3_t;
            if (*tw).sphere.r#use != qfalse {
                // adjust the plane distance apropriately for radius
                plane[3] += (*tw).sphere.radius;

                // find the closest point on the capsule to the plane
                let t = DotProductRow(&plane, (*tw).sphere.offset);
                if t > 0.0 {
                    startp = [0.0; 3];
                    endp = [0.0; 3];
                    _VectorSubtract((*tw).start, (*tw).sphere.offset, &mut startp);
                    _VectorSubtract((*tw).end, (*tw).sphere.offset, &mut endp);
                } else {
                    startp = [0.0; 3];
                    endp = [0.0; 3];
                    _VectorAdd((*tw).start, (*tw).sphere.offset, &mut startp);
                    _VectorAdd((*tw).end, (*tw).sphere.offset, &mut endp);
                }
            } else {
                let offset = DotProductRow(&plane, (*tw).offsets[(*planes).signbits as usize]);
                plane[3] -= offset;
                startp = (*tw).start;
                endp = (*tw).end;
            }

            let mut hit: c_int = 0;
            if CM_CheckFacetPlane(
                &plane,
                startp,
                endp,
                &mut enter_frac,
                &mut leave_frac,
                &mut hit,
            ) == qfalse
            {
                facet = facet.add(1);
                continue;
            }
            // §19: Raven leaves `bestplane` uninitialized until the first hit;
            // it is only read below when `enterFrac >= 0`, which requires a hit.
            let mut bestplane: vec4_t = [0.0; 4];
            if hit != 0 {
                bestplane = plane;
            }

            let mut j = 0;
            while j < (*facet).numBorders {
                planes = (*pc).planes.add((*facet).borderPlanes[j as usize] as usize);
                if (*facet).borderInward[j as usize] != 0 {
                    plane = [
                        -(*planes).plane[0],
                        -(*planes).plane[1],
                        -(*planes).plane[2],
                        -(*planes).plane[3],
                    ];
                } else {
                    plane = [
                        (*planes).plane[0],
                        (*planes).plane[1],
                        (*planes).plane[2],
                        (*planes).plane[3],
                    ];
                }
                if (*tw).sphere.r#use != qfalse {
                    // adjust the plane distance apropriately for radius
                    plane[3] += (*tw).sphere.radius;

                    // find the closest point on the capsule to the plane
                    let t = DotProductRow(&plane, (*tw).sphere.offset);
                    if t > 0.0 {
                        startp = [0.0; 3];
                        endp = [0.0; 3];
                        _VectorSubtract((*tw).start, (*tw).sphere.offset, &mut startp);
                        _VectorSubtract((*tw).end, (*tw).sphere.offset, &mut endp);
                    } else {
                        startp = [0.0; 3];
                        endp = [0.0; 3];
                        _VectorAdd((*tw).start, (*tw).sphere.offset, &mut startp);
                        _VectorAdd((*tw).end, (*tw).sphere.offset, &mut endp);
                    }
                } else {
                    // NOTE: this works even though the plane might be flipped because the bbox is centered
                    let offset = DotProductRow(&plane, (*tw).offsets[(*planes).signbits as usize]);
                    plane[3] += offset.abs();
                    startp = (*tw).start;
                    endp = (*tw).end;
                }

                if CM_CheckFacetPlane(
                    &plane,
                    startp,
                    endp,
                    &mut enter_frac,
                    &mut leave_frac,
                    &mut hit,
                ) == qfalse
                {
                    break;
                }
                if hit != 0 {
                    hitnum = j;
                    bestplane = plane;
                }
                j += 1;
            }
            if j < (*facet).numBorders {
                facet = facet.add(1);
                continue;
            }
            // never clip against the back side
            if hitnum == (*facet).numBorders - 1 {
                facet = facet.add(1);
                continue;
            }

            if enter_frac < leave_frac && enter_frac >= 0.0 {
                if enter_frac < trace.fraction {
                    if enter_frac < 0.0 {
                        enter_frac = 0.0;
                    }
                    // §20: `debugPatchCollide`/`debugFacet` capture dropped.

                    trace.fraction = enter_frac;
                    trace.plane.normal = [bestplane[0], bestplane[1], bestplane[2]];
                    trace.plane.dist = bestplane[3];
                }
            }
            facet = facet.add(1);
        }
    }
}

/*
=======================================================================
POSITION DETECTION
=======================================================================
*/

/// Raven `CM_PositionTestInPatchCollide` — modifies the trace if any facet
/// affects it.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:1545-1628`
pub fn CM_PositionTestInPatchCollide(tw: *mut traceWork_t, pc: *const patchCollide_s) -> qboolean {
    unsafe {
        if (*tw).isPoint != qfalse {
            return qfalse;
        }

        let mut facet = (*pc).facets;
        for _i in 0..(*pc).numFacets {
            let mut planes = (*pc).planes.add((*facet).surfacePlane as usize);
            let mut plane: vec4_t = [
                (*planes).plane[0],
                (*planes).plane[1],
                (*planes).plane[2],
                (*planes).plane[3],
            ];
            let mut startp: vec3_t;
            if (*tw).sphere.r#use != qfalse {
                // adjust the plane distance apropriately for radius
                plane[3] += (*tw).sphere.radius;

                // find the closest point on the capsule to the plane
                let t = DotProductRow(&plane, (*tw).sphere.offset);
                if t > 0.0 {
                    startp = [0.0; 3];
                    _VectorSubtract((*tw).start, (*tw).sphere.offset, &mut startp);
                } else {
                    startp = [0.0; 3];
                    _VectorAdd((*tw).start, (*tw).sphere.offset, &mut startp);
                }
            } else {
                let offset = DotProductRow(&plane, (*tw).offsets[(*planes).signbits as usize]);
                plane[3] -= offset;
                startp = (*tw).start;
            }

            if DotProductRow(&plane, startp) - plane[3] > 0.0 {
                facet = facet.add(1);
                continue;
            }

            let mut j = 0;
            while j < (*facet).numBorders {
                planes = (*pc).planes.add((*facet).borderPlanes[j as usize] as usize);
                if (*facet).borderInward[j as usize] != 0 {
                    plane = [
                        -(*planes).plane[0],
                        -(*planes).plane[1],
                        -(*planes).plane[2],
                        -(*planes).plane[3],
                    ];
                } else {
                    plane = [
                        (*planes).plane[0],
                        (*planes).plane[1],
                        (*planes).plane[2],
                        (*planes).plane[3],
                    ];
                }
                if (*tw).sphere.r#use != qfalse {
                    // adjust the plane distance apropriately for radius
                    plane[3] += (*tw).sphere.radius;

                    // find the closest point on the capsule to the plane
                    let t = DotProductRow(&plane, (*tw).sphere.offset);
                    if t > 0.0 {
                        startp = [0.0; 3];
                        _VectorSubtract((*tw).start, (*tw).sphere.offset, &mut startp);
                    } else {
                        startp = [0.0; 3];
                        _VectorAdd((*tw).start, (*tw).sphere.offset, &mut startp);
                    }
                } else {
                    // NOTE: this works even though the plane might be flipped because the bbox is centered
                    let offset = DotProductRow(&plane, (*tw).offsets[(*planes).signbits as usize]);
                    plane[3] += offset.abs();
                    startp = (*tw).start;
                }

                if DotProductRow(&plane, startp) - plane[3] > 0.0 {
                    break;
                }
                j += 1;
            }
            if j < (*facet).numBorders {
                facet = facet.add(1);
                continue;
            }
            // inside this patch facet
            return qtrue;
        }

        qfalse
    }
}
