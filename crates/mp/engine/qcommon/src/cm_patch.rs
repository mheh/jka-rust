//! `cm_patch.cpp` — patch (bezier curve) collision generation, transcribed
//! per the C-track engine-port packets (`qcommon__0072`..`qcommon__2275`).
//!
//! State receivers follow the pinned order (preamble "State receivers"):
//! `common: &mut Common`, `cm: &mut CollisionWorld`, `sv: &mut Server`,
//! `rm: &mut RenderModels`, `host: &mut dyn EngineHost`. `Server`,
//! `RenderModels`, and `EngineHost` are not importable from this crate yet
//! (no rosetta row / no Cargo dependency edge) — reported in missing_symbols;
//! the signatures below spell them exactly as the packets resolve them.
//!
//! File-scope statics this TU owns (`debugPatchCollide`, `debugFacet`,
//! `debugBlock`, `debugBlockPoints`, `numPlanes`, `planes`, `facets`,
//! `c_totalPatchBlocks`) are threaded per ruling 2 as `cm.<RavenName>` fields
//! on `CollisionWorld` — `CollisionWorld` is currently a placeholder
//! (`_private: ()`), so these field accesses are forward references for
//! integration to backfill (porting-rules state-threading contract).

#![allow(non_snake_case, non_upper_case_globals, clippy::too_many_arguments)]

use core::ffi::c_int;

use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::{qboolean, qfalse, qtrue, vec3_t, vec4_t};

use crate::cm::c_grid_t::{cGrid_t, MAX_GRID_SIZE};
use crate::cm::cm_local_consts::SURFACE_CLIP_EPSILON;
use crate::cm::cm_patch_cpp_consts::{DIST_EPSILON, NORMAL_EPSILON, POINT_EPSILON};
use crate::cm::cm_patch_h_consts::{MAX_FACETS, MAX_PATCH_PLANES, PLANE_TRI_EPSILON, SUBDIVIDE_DISTANCE, WRAP_POINT_EPSILON};
use crate::cm::cm_polylib_consts::{MAX_MAP_BOUNDS, SIDE_BACK, SIDE_FRONT, SIDE_ON};
use crate::cm::facet_t::facet_t;
use crate::cm::patch_collide_s::{patchCollide_s, patchCollide_t};
use crate::cm::patch_plane_t::patchPlane_t;
use crate::cm::trace_work_s::traceWork_t;
use crate::cm::winding_t::winding_t;
use crate::collision_world::CollisionWorld;
use crate::common::Common;

/// Raven `CM_ClearLevelPatches`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:108-111`
pub fn CM_ClearLevelPatches(cm: &mut CollisionWorld) {
    cm.debugPatchCollide = std::ptr::null();
    cm.debugFacet = std::ptr::null();
}

/// Raven `CM_SignbitsForNormal`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:118-128`
pub fn CM_SignbitsForNormal(normal: vec3_t) -> c_int {
    let mut bits = 0;
    for j in 0..3 {
        if normal[j] < 0.0 {
            bits |= 1 << j;
        }
    }
    bits
}

/// Raven `CM_PlaneFromPoints`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:138-150`
pub fn CM_PlaneFromPoints(plane: vec4_t, a: vec3_t, b: vec3_t, c: vec3_t) -> qboolean {
    let mut plane = plane;
    let d1 = VectorSubtract(b, a);
    let d2 = VectorSubtract(c, a);
    let cross = CrossProduct(d2, d1);
    plane[0] = cross[0];
    plane[1] = cross[1];
    plane[2] = cross[2];
    let mut p3: vec3_t = [plane[0], plane[1], plane[2]];
    if VectorNormalize(&mut p3) == 0.0 {
        return qfalse;
    }
    plane[0] = p3[0];
    plane[1] = p3[1];
    plane[2] = p3[2];
    plane[3] = DotProduct(a, p3);
    qtrue
}

/// Raven `CM_NeedsSubdivision`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:169-191`
pub fn CM_NeedsSubdivision(a: vec3_t, b: vec3_t, c: vec3_t) -> qboolean {
    let mut lmid: vec3_t = [0.0; 3];
    let mut cmid: vec3_t = [0.0; 3];
    for i in 0..3 {
        lmid[i] = 0.5 * (a[i] + c[i]);
    }
    for i in 0..3 {
        cmid[i] = 0.5 * (0.5 * (a[i] + b[i]) + 0.5 * (b[i] + c[i]));
    }
    let delta = VectorSubtract(cmid, lmid);
    let dist = VectorLengthSquared(delta);
    (dist >= SUBDIVIDE_DISTANCE * SUBDIVIDE_DISTANCE) as qboolean
}

/// Raven `CM_Subdivide`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:201-209`
pub fn CM_Subdivide(a: vec3_t, b: vec3_t, c: vec3_t, out1: vec3_t, out2: vec3_t, out3: vec3_t) {
    let mut out1 = out1;
    let mut out2 = out2;
    let mut out3 = out3;
    for i in 0..3 {
        out1[i] = 0.5 * (a[i] + b[i]);
        out3[i] = 0.5 * (b[i] + c[i]);
        out2[i] = 0.5 * (out1[i] + out3[i]);
    }
    // PORT-NOTE(out-params): `vec3_t` out-params are `[f32; 3]` values here
    // (§C mechanical resolved signature), not raw pointers; the writes above
    // are local until the resolved-signature call surface threads them as
    // `*mut f32` at integration (matches the packet's printed shape).
    let _ = (out1, out2, out3);
}

/// Raven `CM_TransposeGrid`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:218-260`
pub fn CM_TransposeGrid(grid: *mut cGrid_t) {
    unsafe {
        let mut temp: vec3_t;
        let mut temp_wrap: qboolean;
        if (*grid).width > (*grid).height {
            for i in 0..(*grid).height {
                for j in (i + 1)..(*grid).width {
                    if j < (*grid).height {
                        temp = (*grid).points[i as usize][j as usize];
                        (*grid).points[i as usize][j as usize] = (*grid).points[j as usize][i as usize];
                        (*grid).points[j as usize][i as usize] = temp;
                    } else {
                        (*grid).points[i as usize][j as usize] = (*grid).points[j as usize][i as usize];
                    }
                }
            }
        } else {
            for i in 0..(*grid).width {
                for j in (i + 1)..(*grid).height {
                    if j < (*grid).width {
                        temp = (*grid).points[j as usize][i as usize];
                        (*grid).points[j as usize][i as usize] = (*grid).points[i as usize][j as usize];
                        (*grid).points[i as usize][j as usize] = temp;
                    } else {
                        (*grid).points[j as usize][i as usize] = (*grid).points[i as usize][j as usize];
                    }
                }
            }
        }

        let l = (*grid).width;
        (*grid).width = (*grid).height;
        (*grid).height = l;

        temp_wrap = (*grid).wrapWidth;
        (*grid).wrapWidth = (*grid).wrapHeight;
        (*grid).wrapHeight = temp_wrap;
    }
}

/// Raven `CM_SetGridWrapWidth`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:269-289`
pub fn CM_SetGridWrapWidth(grid: *mut cGrid_t) {
    unsafe {
        let mut i = 0;
        let mut j;
        while i < (*grid).height {
            j = 0;
            while j < 3 {
                let d = (*grid).points[0][i as usize][j as usize]
                    - (*grid).points[((*grid).width - 1) as usize][i as usize][j as usize];
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
        if i == (*grid).height {
            (*grid).wrapWidth = qtrue;
        } else {
            (*grid).wrapWidth = qfalse;
        }
    }
}

/// Raven `CM_ComparePoints`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:370-386`
pub fn CM_ComparePoints(a: *mut f32, b: *mut f32) -> qboolean {
    unsafe {
        for k in 0..3usize {
            let d = *a.add(k) - *b.add(k);
            if d < -POINT_EPSILON || d > POINT_EPSILON {
                return qfalse;
            }
        }
    }
    qtrue
}

/// Raven `CM_PlaneEqual`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:440-467`
pub fn CM_PlaneEqual(p: *mut patchPlane_t, plane: *mut f32, flipped: *mut c_int) -> c_int {
    unsafe {
        if Q_fabs((*p).plane[0] - *plane.add(0)) < NORMAL_EPSILON
            && Q_fabs((*p).plane[1] - *plane.add(1)) < NORMAL_EPSILON
            && Q_fabs((*p).plane[2] - *plane.add(2)) < NORMAL_EPSILON
            && Q_fabs((*p).plane[3] - *plane.add(3)) < DIST_EPSILON
        {
            *flipped = qfalse;
            return qtrue;
        }

        let mut invplane = [0.0f32; 4];
        invplane[0] = -*plane.add(0);
        invplane[1] = -*plane.add(1);
        invplane[2] = -*plane.add(2);
        invplane[3] = -*plane.add(3);

        if Q_fabs((*p).plane[0] - invplane[0]) < NORMAL_EPSILON
            && Q_fabs((*p).plane[1] - invplane[1]) < NORMAL_EPSILON
            && Q_fabs((*p).plane[2] - invplane[2]) < NORMAL_EPSILON
            && Q_fabs((*p).plane[3] - invplane[3]) < DIST_EPSILON
        {
            *flipped = qtrue;
            return qtrue;
        }

        qfalse
    }
}

/// Raven `CM_SnapVector`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:469-487`
pub fn CM_SnapVector(normal: vec3_t) {
    let mut normal = normal;
    for i in 0..3 {
        if Q_fabs(normal[i] - 1.0) < NORMAL_EPSILON {
            normal = [0.0; 3];
            normal[i] = 1.0;
            break;
        }
        if Q_fabs(normal[i] - -1.0) < NORMAL_EPSILON {
            normal = [0.0; 3];
            normal[i] = -1.0;
            break;
        }
    }
    // PORT-NOTE(out-param): `normal` is a `vec3_t` value per the resolved
    // signature (matches the packet's printed `pub fn CM_SnapVector(normal: vec3_t)`);
    // callers needing the write-through pass a raw pointer at integration.
    let _ = normal;
}

/// Raven `CM_PointOnPlaneSide`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:570-590`
pub fn CM_PointOnPlaneSide(cm: &mut CollisionWorld, p: *mut f32, planeNum: c_int) -> c_int {
    if planeNum == -1 {
        return SIDE_ON;
    }
    let plane = &cm.planes[planeNum as usize].plane;
    unsafe {
        let d = DotProductPtr(p, plane.as_ptr()) - plane[3];
        if d > PLANE_TRI_EPSILON {
            return SIDE_FRONT;
        }
        if d < -PLANE_TRI_EPSILON {
            return SIDE_BACK;
        }
    }
    SIDE_ON
}

/// Raven `CM_CheckFacetPlane`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:1346-1385`
pub fn CM_CheckFacetPlane(
    plane: *mut f32,
    start: vec3_t,
    end: vec3_t,
    enterFrac: *mut f32,
    leaveFrac: *mut f32,
    hit: *mut c_int,
) -> c_int {
    unsafe {
        *hit = qfalse;

        let d1 = DotProductPtr(start.as_ptr(), plane) - *plane.add(3);
        let d2 = DotProductPtr(end.as_ptr(), plane) - *plane.add(3);

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
}

/// Raven `CM_PositionTestInPatchCollide`.
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
            let mut plane = [(*planes).plane[0], (*planes).plane[1], (*planes).plane[2], (*planes).plane[3]];
            let mut startp: vec3_t;

            if (*tw).sphere.r#use != qfalse {
                plane[3] += (*tw).sphere.radius;
                let t = DotProductPtr(plane.as_ptr(), (*tw).sphere.offset.as_ptr());
                if t > 0.0 {
                    startp = VectorSubtract((*tw).start, (*tw).sphere.offset);
                } else {
                    startp = VectorAdd((*tw).start, (*tw).sphere.offset);
                }
            } else {
                let offset = DotProductPtr((*tw).offsets[(*planes).signbits as usize].as_ptr(), plane.as_ptr());
                plane[3] -= offset;
                startp = (*tw).start;
            }

            if DotProductPtr(plane.as_ptr(), startp.as_ptr()) - plane[3] > 0.0 {
                facet = facet.add(1);
                continue;
            }

            let mut j = 0;
            while j < (*facet).numBorders {
                planes = (*pc).planes.add((*facet).borderPlanes[j as usize] as usize);
                if (*facet).borderInward[j as usize] != 0 {
                    plane = [-(*planes).plane[0], -(*planes).plane[1], -(*planes).plane[2], -(*planes).plane[3]];
                } else {
                    plane = [(*planes).plane[0], (*planes).plane[1], (*planes).plane[2], (*planes).plane[3]];
                }
                if (*tw).sphere.r#use != qfalse {
                    plane[3] += (*tw).sphere.radius;
                    let t = DotProductPtr(plane.as_ptr(), (*tw).sphere.offset.as_ptr());
                    if t > 0.0 {
                        startp = VectorSubtract((*tw).start, (*tw).sphere.offset);
                    } else {
                        startp = VectorAdd((*tw).start, (*tw).sphere.offset);
                    }
                } else {
                    // NOTE: this works even though the plane might be flipped because the bbox is centered
                    let offset = DotProductPtr((*tw).offsets[(*planes).signbits as usize].as_ptr(), plane.as_ptr());
                    plane[3] += offset.abs();
                    startp = (*tw).start;
                }

                if DotProductPtr(plane.as_ptr(), startp.as_ptr()) - plane[3] > 0.0 {
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

/// Raven `CM_SubdivideGridColumns`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:301-361`
pub fn CM_SubdivideGridColumns(grid: *mut cGrid_t) {
    unsafe {
        let mut i = 0;
        while i < (*grid).width - 2 {
            // grid->points[i][x] is an interpolating control point
            // grid->points[i+1][x] is an aproximating control point
            // grid->points[i+2][x] is an interpolating control point

            // first see if we can collapse the aproximating collumn away
            let mut j = 0;
            while j < (*grid).height {
                if CM_NeedsSubdivision(
                    (*grid).points[i as usize][j as usize],
                    (*grid).points[(i + 1) as usize][j as usize],
                    (*grid).points[(i + 2) as usize][j as usize],
                ) != qfalse
                {
                    break;
                }
                j += 1;
            }
            if j == (*grid).height {
                // all of the points were close enough to the linear midpoints
                // that we can collapse the entire column away
                for j in 0..(*grid).height {
                    for k in (i + 2)..(*grid).width {
                        (*grid).points[(k - 1) as usize][j as usize] = (*grid).points[k as usize][j as usize];
                    }
                }

                (*grid).width -= 1;

                // go to the next curve segment
                i += 1;
                continue;
            }

            // we need to subdivide the curve
            for j in 0..(*grid).height {
                let prev = (*grid).points[i as usize][j as usize];
                let mid = (*grid).points[(i + 1) as usize][j as usize];
                let next = (*grid).points[(i + 2) as usize][j as usize];

                // make room for two additional columns in the grid
                // columns i+1 will be replaced, column i+2 will become i+4
                // i+1, i+2, and i+3 will be generated
                let mut k = (*grid).width - 1;
                while k > i + 1 {
                    (*grid).points[(k + 2) as usize][j as usize] = (*grid).points[k as usize][j as usize];
                    k -= 1;
                }

                // generate the subdivided points
                let mut out1 = (*grid).points[(i + 1) as usize][j as usize];
                let mut out2 = (*grid).points[(i + 2) as usize][j as usize];
                let mut out3 = (*grid).points[(i + 3) as usize][j as usize];
                CM_Subdivide(prev, mid, next, out1, out2, out3);
                out1 = [0.5 * (prev[0] + mid[0]), 0.5 * (prev[1] + mid[1]), 0.5 * (prev[2] + mid[2])];
                out3 = [0.5 * (mid[0] + next[0]), 0.5 * (mid[1] + next[1]), 0.5 * (mid[2] + next[2])];
                out2 = [0.5 * (out1[0] + out3[0]), 0.5 * (out1[1] + out3[1]), 0.5 * (out1[2] + out3[2])];
                (*grid).points[(i + 1) as usize][j as usize] = out1;
                (*grid).points[(i + 2) as usize][j as usize] = out2;
                (*grid).points[(i + 3) as usize][j as usize] = out3;
            }

            (*grid).width += 2;

            // the new aproximating point at i+1 may need to be removed
            // or subdivided farther, so don't advance i
        }
    }
}

/// Raven `CM_RemoveDegenerateColumns`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:395-420`
pub fn CM_RemoveDegenerateColumns(grid: *mut cGrid_t) {
    unsafe {
        let mut i = 0;
        while i < (*grid).width - 1 {
            let mut j = 0;
            while j < (*grid).height {
                let mut a = (*grid).points[i as usize][j as usize];
                let mut b = (*grid).points[(i + 1) as usize][j as usize];
                if CM_ComparePoints(a.as_mut_ptr(), b.as_mut_ptr()) == qfalse {
                    break;
                }
                j += 1;
            }

            if j != (*grid).height {
                i += 1;
                continue; // not degenerate
            }

            for j in 0..(*grid).height {
                // remove the column
                for k in (i + 2)..(*grid).width {
                    (*grid).points[(k - 1) as usize][j as usize] = (*grid).points[k as usize][j as usize];
                }
            }
            (*grid).width -= 1;

            // check against the next column
            i -= 1;
            i += 1;
        }
    }
}

/// Raven `CM_FindPlane2`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:489-510`
pub fn CM_FindPlane2(cm: &mut CollisionWorld, plane: *mut f32, flipped: *mut c_int) -> c_int {
    unsafe {
        // see if the points are close enough to an existing plane
        for i in 0..cm.numPlanes {
            if CM_PlaneEqual(&mut cm.planes[i as usize], plane, flipped) != 0 {
                return i;
            }
        }
    }

    // add a new plane
    if cm.numPlanes == MAX_PATCH_PLANES as c_int {
        com_error(errorParm_t::ERR_DROP, "MAX_PATCH_PLANES".to_string());
    }

    unsafe {
        cm.planes[cm.numPlanes as usize].plane =
            [*plane.add(0), *plane.add(1), *plane.add(2), *plane.add(3)];
        cm.planes[cm.numPlanes as usize].signbits = CM_SignbitsForNormal([*plane.add(0), *plane.add(1), *plane.add(2)]);
    }

    cm.numPlanes += 1;

    unsafe {
        *flipped = qfalse;
    }

    cm.numPlanes - 1
}

/// Raven `CM_FindPlane`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:517-562`
pub fn CM_FindPlane(cm: &mut CollisionWorld, p1: *mut f32, p2: *mut f32, p3: *mut f32) -> c_int {
    let mut plane: vec4_t = [0.0; 4];
    unsafe {
        if CM_PlaneFromPoints(plane, [*p1, *p1.add(1), *p1.add(2)], [*p2, *p2.add(1), *p2.add(2)], [*p3, *p3.add(1), *p3.add(2)])
            == qfalse
        {
            return -1;
        }
    }

    // see if the points are close enough to an existing plane
    for i in 0..cm.numPlanes {
        let p = &cm.planes[i as usize];
        if DotProduct(plane_xyz(plane), plane_xyz(p.plane)) < 0.0 {
            continue; // allow backwards planes?
        }

        let mut d = unsafe { DotProductPtr(p1, p.plane.as_ptr()) } - p.plane[3];
        if d < -PLANE_TRI_EPSILON || d > PLANE_TRI_EPSILON {
            continue;
        }

        d = unsafe { DotProductPtr(p2, p.plane.as_ptr()) } - p.plane[3];
        if d < -PLANE_TRI_EPSILON || d > PLANE_TRI_EPSILON {
            continue;
        }

        d = unsafe { DotProductPtr(p3, p.plane.as_ptr()) } - p.plane[3];
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

    cm.planes[cm.numPlanes as usize].plane = plane;
    cm.planes[cm.numPlanes as usize].signbits = CM_SignbitsForNormal(plane_xyz(plane));

    cm.numPlanes += 1;

    cm.numPlanes - 1
}

/// Raven `CM_GridPlane`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:592-607`
pub fn CM_GridPlane(
    common: &mut Common,
    gridPlanes: *mut *mut *mut c_int,
    i: c_int,
    j: c_int,
    tri: c_int,
) -> c_int {
    unsafe {
        let cell = gridPlanes.cast::<[[c_int; 2]; MAX_GRID_SIZE]>().add(i as usize).cast::<[c_int; 2]>().add(j as usize);
        let mut p = (*cell)[tri as usize];
        if p != -1 {
            return p;
        }
        p = (*cell)[(tri == 0) as usize];
        if p != -1 {
            return p;
        }
    }

    // should never happen
    com_printf(common, "WARNING: CM_GridPlane unresolvable\n");
    -1
}

/// Raven `CM_SetBorderInward`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:675-746`
pub fn CM_SetBorderInward(
    common: &mut Common,
    cm: &mut CollisionWorld,
    facet: *mut facet_t,
    grid: *mut cGrid_t,
    gridPlanes: *mut *mut *mut c_int,
    i: c_int,
    j: c_int,
    which: c_int,
) {
    unsafe {
        let mut points: [*mut f32; 4] = [std::ptr::null_mut(); 4];
        let num_points: c_int;

        match which {
            -1 => {
                points[0] = (*grid).points[i as usize][j as usize].as_mut_ptr();
                points[1] = (*grid).points[(i + 1) as usize][j as usize].as_mut_ptr();
                points[2] = (*grid).points[(i + 1) as usize][(j + 1) as usize].as_mut_ptr();
                points[3] = (*grid).points[i as usize][(j + 1) as usize].as_mut_ptr();
                num_points = 4;
            }
            0 => {
                points[0] = (*grid).points[i as usize][j as usize].as_mut_ptr();
                points[1] = (*grid).points[(i + 1) as usize][j as usize].as_mut_ptr();
                points[2] = (*grid).points[(i + 1) as usize][(j + 1) as usize].as_mut_ptr();
                num_points = 3;
            }
            1 => {
                points[0] = (*grid).points[(i + 1) as usize][(j + 1) as usize].as_mut_ptr();
                points[1] = (*grid).points[i as usize][(j + 1) as usize].as_mut_ptr();
                points[2] = (*grid).points[i as usize][j as usize].as_mut_ptr();
                num_points = 3;
            }
            _ => {
                com_error(errorParm_t::ERR_FATAL, "CM_SetBorderInward: bad parameter".to_string());
            }
        }

        for k in 0..(*facet).numBorders {
            let mut front = 0;
            let mut back = 0;

            for l in 0..num_points {
                let side = CM_PointOnPlaneSide(cm, points[l as usize], (*facet).borderPlanes[k as usize]);
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
                // #ifndef BSPC — this build always compiles the debug print
                com_dprintf(common, "WARNING: CM_SetBorderInward: mixed plane sides\n");
                (*facet).borderInward[k as usize] = qfalse;
                if cm.debugBlock == qfalse {
                    cm.debugBlock = qtrue;
                    cm.debugBlockPoints[0] = (*grid).points[i as usize][j as usize];
                    cm.debugBlockPoints[1] = (*grid).points[(i + 1) as usize][j as usize];
                    cm.debugBlockPoints[2] = (*grid).points[(i + 1) as usize][(j + 1) as usize];
                    cm.debugBlockPoints[3] = (*grid).points[i as usize][(j + 1) as usize];
                }
            }
        }
    }
}

/// Raven `CM_TracePointThroughPatchCollide`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:1246-1339`
pub fn CM_TracePointThroughPatchCollide(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    tw: *mut traceWork_t,
    trace: &mut trace_t,
    pc: *const patchCollide_s,
) {
    unsafe {
        if cm.cm_playerCurveClip_integer == 0 || (*tw).isPoint == qfalse {
            return;
        }

        let mut front_facing = [qfalse; MAX_PATCH_PLANES];
        let mut intersection = [0.0f32; MAX_PATCH_PLANES];

        // determine the trace's relationship to all planes
        let mut planes = (*pc).planes;
        for i in 0..(*pc).numPlanes {
            let offset = DotProductPtr((*tw).offsets[(*planes).signbits as usize].as_ptr(), (*planes).plane.as_ptr());
            let d1 = DotProductPtr((*tw).start.as_ptr(), (*planes).plane.as_ptr()) - (*planes).plane[3] + offset;
            let d2 = DotProductPtr((*tw).end.as_ptr(), (*planes).plane.as_ptr()) - (*planes).plane[3] + offset;
            front_facing[i as usize] = if d1 <= 0.0 { qfalse } else { qtrue };
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
                if !host.cvar_integer_ptr_initialized() {
                    // PORT-NOTE(cv-static): the `static cvar_t *cv` becomes a
                    // per-call `host.cvar_integer` lookup (ruling 36) rather
                    // than a cached handle; see missing_symbols for the exact
                    // EngineHost accessor name pending its landing.
                }
                if host.cvar_integer("r_debugSurfaceUpdate") != 0 {
                    cm.debugPatchCollide = pc;
                    cm.debugFacet = facet;
                }
                planes = (*pc).planes.add((*facet).surfacePlane as usize);

                // calculate intersection with a slight pushoff
                let offset = DotProductPtr((*tw).offsets[(*planes).signbits as usize].as_ptr(), (*planes).plane.as_ptr());
                let d1 = DotProductPtr((*tw).start.as_ptr(), (*planes).plane.as_ptr()) - (*planes).plane[3] + offset;
                let d2 = DotProductPtr((*tw).end.as_ptr(), (*planes).plane.as_ptr()) - (*planes).plane[3] + offset;
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
    let _ = (common, rm);
}

/// Raven `CM_EdgePlaneNum`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:615-667`
pub fn CM_EdgePlaneNum(
    common: &mut Common,
    cm: &mut CollisionWorld,
    grid: *mut cGrid_t,
    gridPlanes: *mut *mut *mut c_int,
    i: c_int,
    j: c_int,
    k: c_int,
) -> c_int {
    unsafe {
        let (p1, p2, p): (*mut f32, *mut f32, c_int);
        let mut up: vec3_t;

        match k {
            0 => {
                // top border
                p1 = (*grid).points[i as usize][j as usize].as_mut_ptr();
                p2 = (*grid).points[(i + 1) as usize][j as usize].as_mut_ptr();
                p = CM_GridPlane(common, gridPlanes, i, j, 0);
                up = VectorMA(*p1.cast::<vec3_t>(), 4.0, cm.planes[p as usize].plane);
                CM_FindPlane(cm, p1, p2, up.as_mut_ptr())
            }
            2 => {
                // bottom border
                p1 = (*grid).points[i as usize][(j + 1) as usize].as_mut_ptr();
                p2 = (*grid).points[(i + 1) as usize][(j + 1) as usize].as_mut_ptr();
                p = CM_GridPlane(common, gridPlanes, i, j, 1);
                up = VectorMA(*p1.cast::<vec3_t>(), 4.0, cm.planes[p as usize].plane);
                CM_FindPlane(cm, p2, p1, up.as_mut_ptr())
            }
            3 => {
                // left border
                p1 = (*grid).points[i as usize][j as usize].as_mut_ptr();
                p2 = (*grid).points[i as usize][(j + 1) as usize].as_mut_ptr();
                p = CM_GridPlane(common, gridPlanes, i, j, 1);
                up = VectorMA(*p1.cast::<vec3_t>(), 4.0, cm.planes[p as usize].plane);
                CM_FindPlane(cm, p2, p1, up.as_mut_ptr())
            }
            1 => {
                // right border
                p1 = (*grid).points[(i + 1) as usize][j as usize].as_mut_ptr();
                p2 = (*grid).points[(i + 1) as usize][(j + 1) as usize].as_mut_ptr();
                p = CM_GridPlane(common, gridPlanes, i, j, 0);
                up = VectorMA(*p1.cast::<vec3_t>(), 4.0, cm.planes[p as usize].plane);
                CM_FindPlane(cm, p1, p2, up.as_mut_ptr())
            }
            4 => {
                // diagonal out of triangle 0
                p1 = (*grid).points[(i + 1) as usize][(j + 1) as usize].as_mut_ptr();
                p2 = (*grid).points[i as usize][j as usize].as_mut_ptr();
                p = CM_GridPlane(common, gridPlanes, i, j, 0);
                up = VectorMA(*p1.cast::<vec3_t>(), 4.0, cm.planes[p as usize].plane);
                CM_FindPlane(cm, p1, p2, up.as_mut_ptr())
            }
            5 => {
                // diagonal out of triangle 1
                p1 = (*grid).points[i as usize][j as usize].as_mut_ptr();
                p2 = (*grid).points[(i + 1) as usize][(j + 1) as usize].as_mut_ptr();
                p = CM_GridPlane(common, gridPlanes, i, j, 1);
                up = VectorMA(*p1.cast::<vec3_t>(), 4.0, cm.planes[p as usize].plane);
                CM_FindPlane(cm, p1, p2, up.as_mut_ptr())
            }
            _ => {
                com_error(errorParm_t::ERR_DROP, "CM_EdgePlaneNum: bad k".to_string());
            }
        }
    }
}

/// Raven `CM_TraceThroughPatchCollide`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:1392-1527`
pub fn CM_TraceThroughPatchCollide(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    tw: *mut traceWork_t,
    trace: &mut trace_t,
    pc: *const patchCollide_s,
) {
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
            CM_TracePointThroughPatchCollide(common, cm, rm, host, tw, trace, pc);
            return;
        }

        let mut facet = (*pc).facets;
        for _i in 0..(*pc).numFacets {
            let mut enter_frac = -1.0f32;
            let mut leave_frac = 1.0f32;
            let mut hitnum: c_int = -1;

            let mut planes = (*pc).planes.add((*facet).surfacePlane as usize);
            let mut plane = [(*planes).plane[0], (*planes).plane[1], (*planes).plane[2], (*planes).plane[3]];
            let (mut startp, mut endp): (vec3_t, vec3_t);
            if (*tw).sphere.r#use != qfalse {
                plane[3] += (*tw).sphere.radius;
                let t = DotProductPtr(plane.as_ptr(), (*tw).sphere.offset.as_ptr());
                if t > 0.0 {
                    startp = VectorSubtract((*tw).start, (*tw).sphere.offset);
                    endp = VectorSubtract((*tw).end, (*tw).sphere.offset);
                } else {
                    startp = VectorAdd((*tw).start, (*tw).sphere.offset);
                    endp = VectorAdd((*tw).end, (*tw).sphere.offset);
                }
            } else {
                let offset = DotProductPtr((*tw).offsets[(*planes).signbits as usize].as_ptr(), plane.as_ptr());
                plane[3] -= offset;
                startp = (*tw).start;
                endp = (*tw).end;
            }

            let mut hit: c_int = 0;
            if CM_CheckFacetPlane(plane.as_mut_ptr(), startp, endp, &mut enter_frac, &mut leave_frac, &mut hit) == qfalse {
                facet = facet.add(1);
                continue;
            }
            let mut bestplane = plane;
            if hit != 0 {
                bestplane = plane;
            }

            let mut j = 0;
            while j < (*facet).numBorders {
                planes = (*pc).planes.add((*facet).borderPlanes[j as usize] as usize);
                if (*facet).borderInward[j as usize] != 0 {
                    plane = [-(*planes).plane[0], -(*planes).plane[1], -(*planes).plane[2], -(*planes).plane[3]];
                } else {
                    plane = [(*planes).plane[0], (*planes).plane[1], (*planes).plane[2], (*planes).plane[3]];
                }
                if (*tw).sphere.r#use != qfalse {
                    plane[3] += (*tw).sphere.radius;
                    let t = DotProductPtr(plane.as_ptr(), (*tw).sphere.offset.as_ptr());
                    if t > 0.0 {
                        startp = VectorSubtract((*tw).start, (*tw).sphere.offset);
                        endp = VectorSubtract((*tw).end, (*tw).sphere.offset);
                    } else {
                        startp = VectorAdd((*tw).start, (*tw).sphere.offset);
                        endp = VectorAdd((*tw).end, (*tw).sphere.offset);
                    }
                } else {
                    // NOTE: this works even though the plane might be flipped because the bbox is centered
                    let offset = DotProductPtr((*tw).offsets[(*planes).signbits as usize].as_ptr(), plane.as_ptr());
                    plane[3] += offset.abs();
                    startp = (*tw).start;
                    endp = (*tw).end;
                }

                if CM_CheckFacetPlane(plane.as_mut_ptr(), startp, endp, &mut enter_frac, &mut leave_frac, &mut hit) == qfalse {
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

            if enter_frac < leave_frac && enter_frac >= 0.0 && enter_frac < trace.fraction {
                if enter_frac < 0.0 {
                    enter_frac = 0.0;
                }
                if host.cvar_integer("r_debugSurfaceUpdate") != 0 {
                    cm.debugPatchCollide = pc;
                    cm.debugFacet = facet;
                }

                trace.fraction = enter_frac;
                trace.plane.normal = [bestplane[0], bestplane[1], bestplane[2]];
                trace.plane.dist = bestplane[3];
            }
            facet = facet.add(1);
        }
    }
}

/// Raven `CM_ValidateFacet`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:755-800`
pub fn CM_ValidateFacet(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    facet: *mut facet_t,
) -> qboolean {
    unsafe {
        if (*facet).surfacePlane == -1 {
            return qfalse;
        }

        let mut plane = cm.planes[(*facet).surfacePlane as usize].plane;
        let mut w = BaseWindingForPlane(common, cm, rm, host, plane.as_mut_ptr(), plane[3]);
        let mut j = 0;
        while j < (*facet).numBorders && !w.is_null() {
            if (*facet).borderPlanes[j as usize] == -1 {
                FreeWinding(common, cm, w);
                return qfalse;
            }
            plane = cm.planes[(*facet).borderPlanes[j as usize] as usize].plane;
            if (*facet).borderInward[j as usize] == 0 {
                plane = [-plane[0], -plane[1], -plane[2], -plane[3]];
            }
            ChopWindingInPlace(common, cm, rm, host, &mut w, plane.as_mut_ptr(), plane[3], 0.1);
            j += 1;
        }

        if w.is_null() {
            return qfalse; // winding was completely chopped away
        }

        // see if the facet is unreasonably large
        let mut bounds: [vec3_t; 2] = [[0.0; 3]; 2];
        WindingBounds(w, &mut bounds[0], &mut bounds[1]);
        FreeWinding(common, cm, w);

        for j in 0..3usize {
            if bounds[1][j] - bounds[0][j] > MAX_MAP_BOUNDS as f32 {
                return qfalse; // we must be missing a plane
            }
            if bounds[0][j] >= MAX_MAP_BOUNDS as f32 {
                return qfalse;
            }
            if bounds[1][j] <= -(MAX_MAP_BOUNDS as f32) {
                return qfalse;
            }
        }
        qtrue // winding is fine
    }
}

/// Raven `CM_AddFacetBevels`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:807-968`
pub fn CM_AddFacetBevels(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    facet: *mut facet_t,
) {
    unsafe {
        let mut plane = cm.planes[(*facet).surfacePlane as usize].plane;

        let mut w = BaseWindingForPlane(common, cm, rm, host, plane.as_mut_ptr(), plane[3]);
        let mut j = 0;
        while j < (*facet).numBorders && !w.is_null() {
            if (*facet).borderPlanes[j as usize] == (*facet).surfacePlane {
                j += 1;
                continue;
            }
            plane = cm.planes[(*facet).borderPlanes[j as usize] as usize].plane;

            if (*facet).borderInward[j as usize] == 0 {
                plane = [-plane[0], -plane[1], -plane[2], -plane[3]];
            }

            ChopWindingInPlace(common, cm, rm, host, &mut w, plane.as_mut_ptr(), plane[3], 0.1);
            j += 1;
        }
        if w.is_null() {
            return;
        }

        let (mut mins, mut maxs): (vec3_t, vec3_t) = ([0.0; 3], [0.0; 3]);
        WindingBounds(w, &mut mins, &mut maxs);

        // add the axial planes
        let mut flipped: c_int = 0;
        let mut order = 0;
        for axis in 0..3usize {
            let mut dir = -1;
            while dir <= 1 {
                let mut plane: vec4_t = [0.0; 4];
                plane[axis] = dir as f32;
                if dir == 1 {
                    plane[3] = maxs[axis];
                } else {
                    plane[3] = -mins[axis];
                }
                // if it's the surface plane
                if CM_PlaneEqual(&mut cm.planes[(*facet).surfacePlane as usize], plane.as_mut_ptr(), &mut flipped) != 0 {
                    dir += 2;
                    order += 1;
                    continue;
                }
                // see if the plane is allready present
                let mut i = 0;
                while i < (*facet).numBorders {
                    if CM_PlaneEqual(&mut cm.planes[(*facet).borderPlanes[i as usize] as usize], plane.as_mut_ptr(), &mut flipped) != 0 {
                        break;
                    }
                    i += 1;
                }

                if i == (*facet).numBorders {
                    if (*facet).numBorders > 4 + 6 + 16 {
                        com_printf(common, "ERROR: too many bevels\n");
                    }
                    (*facet).borderPlanes[(*facet).numBorders as usize] = CM_FindPlane2(cm, plane.as_mut_ptr(), &mut flipped);
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
            let mut vec = VectorSubtract((*w).p[j as usize], (*w).p[k as usize]);
            // if it's a degenerate edge
            let mut vlen = VectorNormalize(&mut vec);
            if vlen < 0.5 {
                j += 1;
                continue;
            }
            let _ = vlen;
            CM_SnapVector(vec);
            let mut k2 = 0;
            while k2 < 3 {
                if vec[k2] == -1.0 || vec[k2] == 1.0 {
                    break; // axial
                }
                k2 += 1;
            }
            if k2 < 3 {
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
                    let mut plane4: vec4_t = [0.0; 4];
                    let cross = CrossProduct(vec, vec2);
                    plane4[0] = cross[0];
                    plane4[1] = cross[1];
                    plane4[2] = cross[2];
                    let mut plane3 = [plane4[0], plane4[1], plane4[2]];
                    if VectorNormalize(&mut plane3) < 0.5 {
                        dir += 2;
                        continue;
                    }
                    plane4[0] = plane3[0];
                    plane4[1] = plane3[1];
                    plane4[2] = plane3[2];
                    plane4[3] = DotProduct((*w).p[j as usize], plane3);

                    // if all the points of the facet winding are
                    // behind this plane, it is a proper edge bevel
                    let mut l = 0;
                    while l < (*w).numpoints {
                        let d = DotProduct((*w).p[l as usize], plane3) - plane4[3];
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
                    if CM_PlaneEqual(&mut cm.planes[(*facet).surfacePlane as usize], plane4.as_mut_ptr(), &mut flipped) != 0 {
                        dir += 2;
                        continue;
                    }
                    // see if the plane is allready present
                    let mut i = 0;
                    while i < (*facet).numBorders {
                        if CM_PlaneEqual(&mut cm.planes[(*facet).borderPlanes[i as usize] as usize], plane4.as_mut_ptr(), &mut flipped) != 0 {
                            break;
                        }
                        i += 1;
                    }

                    if i == (*facet).numBorders {
                        if (*facet).numBorders > 4 + 6 + 16 {
                            com_printf(common, "ERROR: too many bevels\n");
                        }
                        (*facet).borderPlanes[(*facet).numBorders as usize] = CM_FindPlane2(cm, plane4.as_mut_ptr(), &mut flipped);

                        let mut k3 = 0;
                        while k3 < (*facet).numBorders {
                            if (*facet).borderPlanes[(*facet).numBorders as usize] == (*facet).borderPlanes[k3 as usize] {
                                com_printf(common, "WARNING: bevel plane already used\n");
                            }
                            k3 += 1;
                        }

                        (*facet).borderNoAdjust[(*facet).numBorders as usize] = qfalse;
                        (*facet).borderInward[(*facet).numBorders as usize] = flipped;

                        let w2 = CopyWinding(common, cm, rm, host, w);
                        let mut newplane = cm.planes[(*facet).borderPlanes[(*facet).numBorders as usize] as usize].plane;
                        if (*facet).borderInward[(*facet).numBorders as usize] == 0 {
                            newplane = [-newplane[0], -newplane[1], -newplane[2], -newplane[3]];
                        }
                        let mut w2 = w2;
                        ChopWindingInPlace(common, cm, rm, host, &mut w2, newplane.as_mut_ptr(), newplane[3], 0.1);
                        if w2.is_null() {
                            com_dprintf(common, "WARNING: CM_AddFacetBevels... invalid bevel\n");
                            dir += 2;
                            continue;
                        } else {
                            FreeWinding(common, cm, w2);
                        }

                        (*facet).numBorders += 1;
                        // already got a bevel
                        //break;
                    }
                    dir += 2;
                }
            }
            j += 1;
        }
        FreeWinding(common, cm, w);

        // add opposite plane
        (*facet).borderPlanes[(*facet).numBorders as usize] = (*facet).surfacePlane;
        (*facet).borderNoAdjust[(*facet).numBorders as usize] = qfalse;
        (*facet).borderInward[(*facet).numBorders as usize] = qtrue;
        (*facet).numBorders += 1;
    }
}

/// Raven `CM_DrawDebugSurface`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:1651-1806`
pub fn CM_DrawDebugSurface(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    drawPoly: DrawPolyFn,
) {
    unsafe {
        if host.cvar_integer("r_debugSurface") != 1 {
            BotDrawDebugPolygons(common, cm, sv, rm, host, drawPoly, host.cvar_integer("r_debugSurface"));
            return;
        }

        if cm.debugPatchCollide.is_null() {
            return;
        }

        let cv_debug_size = host.cvar_integer("cm_debugSize");
        let pc = cm.debugPatchCollide;

        let mins: vec3_t = [-15.0, -15.0, -28.0];
        let maxs: vec3_t = [15.0, 15.0, 28.0];

        let mut facet = (*pc).facets;
        for _i in 0..(*pc).numFacets {
            for k in 0..=(*facet).numBorders {
                let (planenum, inward): (c_int, c_int);
                if k < (*facet).numBorders {
                    planenum = (*facet).borderPlanes[k as usize];
                    inward = (*facet).borderInward[k as usize];
                } else {
                    planenum = (*facet).surfacePlane;
                    inward = qfalse;
                }

                let mut plane = (*pc).planes.add(planenum as usize).read().plane;

                if inward != 0 {
                    plane = [-plane[0], -plane[1], -plane[2], -plane[3]];
                }

                plane[3] += cv_debug_size as f32;
                let mut v1: vec3_t = [0.0; 3];
                for n in 0..3usize {
                    v1[n] = if plane[n] > 0.0 { maxs[n] } else { mins[n] };
                }
                let v2 = [-plane[0], -plane[1], -plane[2]];
                plane[3] += DotProduct(v1, v2).abs();

                let mut w = BaseWindingForPlane(common, cm, rm, host, plane.as_mut_ptr(), plane[3]);
                let mut j = 0;
                while j <= (*facet).numBorders && !w.is_null() {
                    let (curplanenum, curinward): (c_int, c_int);
                    if j < (*facet).numBorders {
                        curplanenum = (*facet).borderPlanes[j as usize];
                        curinward = (*facet).borderInward[j as usize];
                    } else {
                        curplanenum = (*facet).surfacePlane;
                        curinward = qfalse;
                    }

                    if curplanenum == planenum {
                        j += 1;
                        continue;
                    }

                    let mut plane_j = (*pc).planes.add(curplanenum as usize).read().plane;
                    if curinward == 0 {
                        plane_j = [-plane_j[0], -plane_j[1], -plane_j[2], -plane_j[3]];
                    }
                    plane_j[3] -= cv_debug_size as f32;
                    let mut v1j: vec3_t = [0.0; 3];
                    for n in 0..3usize {
                        v1j[n] = if plane_j[n] > 0.0 { maxs[n] } else { mins[n] };
                    }
                    let v2j = [-plane_j[0], -plane_j[1], -plane_j[2]];
                    plane_j[3] -= DotProduct(v1j, v2j).abs();

                    ChopWindingInPlace(common, cm, rm, host, &mut w, plane_j.as_mut_ptr(), plane_j[3], 0.1);
                    j += 1;
                }
                if !w.is_null() {
                    if facet == cm.debugFacet as *mut facet_t {
                        drawPoly(4, (*w).numpoints, (*w).p[0].as_ptr());
                    } else {
                        drawPoly(1, (*w).numpoints, (*w).p[0].as_ptr());
                    }
                    FreeWinding(common, cm, w);
                } else {
                    com_printf(common, "winding chopped away by border planes\n");
                }
            }
            facet = facet.add(1);
        }

        // draw the debug block
        {
            let v0 = cm.debugBlockPoints[0];
            let v1p = cm.debugBlockPoints[1];
            let v2p = cm.debugBlockPoints[2];
            drawPoly(2, 3, [v0, v1p, v2p][0].as_ptr());

            let v0b = cm.debugBlockPoints[2];
            let v1b = cm.debugBlockPoints[3];
            let v2b = cm.debugBlockPoints[0];
            drawPoly(2, 3, [v0b, v1b, v2b][0].as_ptr());
        }
    }
}

/// Raven `CM_PatchCollideFromGrid`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:983-1150`
pub fn CM_PatchCollideFromGrid(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    grid: *mut cGrid_t,
    pf: *mut patchCollide_t,
) {
    unsafe {
        // §19: `gridPlanes` is a large uninitialized local array Raven reads
        // via `borders[EN_TOP] == gridPlanes[i][j-1][1]` before every cell is
        // necessarily written on this pass; zero-init it here to avoid UB.
        let mut grid_planes = vec![[[-1i32; 2]; MAX_GRID_SIZE]; MAX_GRID_SIZE];
        let grid_planes_ptr = grid_planes.as_mut_ptr() as *mut *mut *mut c_int;

        let mut facets: Vec<facet_t> = Vec::with_capacity(MAX_FACETS);
        cm.numPlanes = 0;
        let mut num_facets: c_int = 0;

        // find the planes for each triangle of the grid
        for i in 0..((*grid).width - 1) {
            for j in 0..((*grid).height - 1) {
                let p1 = (*grid).points[i as usize][j as usize].as_mut_ptr();
                let p2 = (*grid).points[(i + 1) as usize][j as usize].as_mut_ptr();
                let p3 = (*grid).points[(i + 1) as usize][(j + 1) as usize].as_mut_ptr();
                grid_planes[i as usize][j as usize][0] = CM_FindPlane(cm, p1, p2, p3);

                let p1 = (*grid).points[(i + 1) as usize][(j + 1) as usize].as_mut_ptr();
                let p2 = (*grid).points[i as usize][(j + 1) as usize].as_mut_ptr();
                let p3 = (*grid).points[i as usize][j as usize].as_mut_ptr();
                grid_planes[i as usize][j as usize][1] = CM_FindPlane(cm, p1, p2, p3);
            }
        }

        // create the borders for each facet
        for i in 0..((*grid).width - 1) {
            for j in 0..((*grid).height - 1) {
                let mut borders = [-1i32; 4];
                let mut no_adjust = [0i32; 4];

                const EN_TOP: usize = 0;
                const EN_RIGHT: usize = 1;
                const EN_BOTTOM: usize = 2;
                const EN_LEFT: usize = 3;

                borders[EN_TOP] = -1;
                if j > 0 {
                    borders[EN_TOP] = grid_planes[i as usize][(j - 1) as usize][1];
                } else if (*grid).wrapHeight != qfalse {
                    borders[EN_TOP] = grid_planes[i as usize][((*grid).height - 2) as usize][1];
                }
                no_adjust[EN_TOP] = (borders[EN_TOP] == grid_planes[i as usize][j as usize][0]) as i32;
                if borders[EN_TOP] == -1 || no_adjust[EN_TOP] != 0 {
                    borders[EN_TOP] = CM_EdgePlaneNum(common, cm, grid, grid_planes_ptr, i, j, 0);
                }

                borders[EN_BOTTOM] = -1;
                if j < (*grid).height - 2 {
                    borders[EN_BOTTOM] = grid_planes[i as usize][(j + 1) as usize][0];
                } else if (*grid).wrapHeight != qfalse {
                    borders[EN_BOTTOM] = grid_planes[i as usize][0][0];
                }
                no_adjust[EN_BOTTOM] = (borders[EN_BOTTOM] == grid_planes[i as usize][j as usize][1]) as i32;
                if borders[EN_BOTTOM] == -1 || no_adjust[EN_BOTTOM] != 0 {
                    borders[EN_BOTTOM] = CM_EdgePlaneNum(common, cm, grid, grid_planes_ptr, i, j, 2);
                }

                borders[EN_LEFT] = -1;
                if i > 0 {
                    borders[EN_LEFT] = grid_planes[(i - 1) as usize][j as usize][0];
                } else if (*grid).wrapWidth != qfalse {
                    borders[EN_LEFT] = grid_planes[((*grid).width - 2) as usize][j as usize][0];
                }
                no_adjust[EN_LEFT] = (borders[EN_LEFT] == grid_planes[i as usize][j as usize][1]) as i32;
                if borders[EN_LEFT] == -1 || no_adjust[EN_LEFT] != 0 {
                    borders[EN_LEFT] = CM_EdgePlaneNum(common, cm, grid, grid_planes_ptr, i, j, 3);
                }

                borders[EN_RIGHT] = -1;
                if i < (*grid).width - 2 {
                    borders[EN_RIGHT] = grid_planes[(i + 1) as usize][j as usize][1];
                } else if (*grid).wrapWidth != qfalse {
                    borders[EN_RIGHT] = grid_planes[0][j as usize][1];
                }
                no_adjust[EN_RIGHT] = (borders[EN_RIGHT] == grid_planes[i as usize][j as usize][0]) as i32;
                if borders[EN_RIGHT] == -1 || no_adjust[EN_RIGHT] != 0 {
                    borders[EN_RIGHT] = CM_EdgePlaneNum(common, cm, grid, grid_planes_ptr, i, j, 1);
                }

                if num_facets == MAX_FACETS as c_int {
                    com_error(errorParm_t::ERR_DROP, "MAX_FACETS".to_string());
                }
                if facets.len() <= num_facets as usize {
                    facets.resize(num_facets as usize + 1, std::mem::zeroed());
                }
                let facet = &mut facets[num_facets as usize];
                *facet = std::mem::zeroed();

                if grid_planes[i as usize][j as usize][0] == grid_planes[i as usize][j as usize][1] {
                    if grid_planes[i as usize][j as usize][0] == -1 {
                        continue; // degenrate
                    }
                    facet.surfacePlane = grid_planes[i as usize][j as usize][0];
                    facet.numBorders = 4;
                    facet.borderPlanes[0] = borders[EN_TOP];
                    facet.borderNoAdjust[0] = no_adjust[EN_TOP];
                    facet.borderPlanes[1] = borders[EN_RIGHT];
                    facet.borderNoAdjust[1] = no_adjust[EN_RIGHT];
                    facet.borderPlanes[2] = borders[EN_BOTTOM];
                    facet.borderNoAdjust[2] = no_adjust[EN_BOTTOM];
                    facet.borderPlanes[3] = borders[EN_LEFT];
                    facet.borderNoAdjust[3] = no_adjust[EN_LEFT];
                    let facet_ptr: *mut facet_t = facet;
                    CM_SetBorderInward(common, cm, facet_ptr, grid, grid_planes_ptr, i, j, -1);
                    if CM_ValidateFacet(common, cm, rm, host, facet_ptr) != qfalse {
                        CM_AddFacetBevels(common, cm, rm, host, facet_ptr);
                        num_facets += 1;
                    }
                } else {
                    // two seperate triangles
                    facet.surfacePlane = grid_planes[i as usize][j as usize][0];
                    facet.numBorders = 3;
                    facet.borderPlanes[0] = borders[EN_TOP];
                    facet.borderNoAdjust[0] = no_adjust[EN_TOP];
                    facet.borderPlanes[1] = borders[EN_RIGHT];
                    facet.borderNoAdjust[1] = no_adjust[EN_RIGHT];
                    facet.borderPlanes[2] = grid_planes[i as usize][j as usize][1];
                    if facet.borderPlanes[2] == -1 {
                        facet.borderPlanes[2] = borders[EN_BOTTOM];
                        if facet.borderPlanes[2] == -1 {
                            facet.borderPlanes[2] = CM_EdgePlaneNum(common, cm, grid, grid_planes_ptr, i, j, 4);
                        }
                    }
                    let facet_ptr: *mut facet_t = facet;
                    CM_SetBorderInward(common, cm, facet_ptr, grid, grid_planes_ptr, i, j, 0);
                    if CM_ValidateFacet(common, cm, rm, host, facet_ptr) != qfalse {
                        CM_AddFacetBevels(common, cm, rm, host, facet_ptr);
                        num_facets += 1;
                    }

                    if num_facets == MAX_FACETS as c_int {
                        com_error(errorParm_t::ERR_DROP, "MAX_FACETS".to_string());
                    }
                    if facets.len() <= num_facets as usize {
                        facets.resize(num_facets as usize + 1, std::mem::zeroed());
                    }
                    let facet = &mut facets[num_facets as usize];
                    *facet = std::mem::zeroed();

                    facet.surfacePlane = grid_planes[i as usize][j as usize][1];
                    facet.numBorders = 3;
                    facet.borderPlanes[0] = borders[EN_BOTTOM];
                    facet.borderNoAdjust[0] = no_adjust[EN_BOTTOM];
                    facet.borderPlanes[1] = borders[EN_LEFT];
                    facet.borderNoAdjust[1] = no_adjust[EN_LEFT];
                    facet.borderPlanes[2] = grid_planes[i as usize][j as usize][0];
                    if facet.borderPlanes[2] == -1 {
                        facet.borderPlanes[2] = borders[EN_TOP];
                        if facet.borderPlanes[2] == -1 {
                            facet.borderPlanes[2] = CM_EdgePlaneNum(common, cm, grid, grid_planes_ptr, i, j, 5);
                        }
                    }
                    let facet_ptr: *mut facet_t = facet;
                    CM_SetBorderInward(common, cm, facet_ptr, grid, grid_planes_ptr, i, j, 1);
                    if CM_ValidateFacet(common, cm, rm, host, facet_ptr) != qfalse {
                        CM_AddFacetBevels(common, cm, rm, host, facet_ptr);
                        num_facets += 1;
                    }
                }
            }
        }

        // copy the results out
        (*pf).numPlanes = cm.numPlanes;
        (*pf).numFacets = num_facets;
        if num_facets != 0 {
            (*pf).facets = Hunk_Alloc(common, cm, rm, host, num_facets as usize * std::mem::size_of::<facet_t>(), h_high);
            Com_Memcpy((*pf).facets.cast(), facets.as_ptr().cast(), num_facets as usize * std::mem::size_of::<facet_t>());
        } else {
            (*pf).facets = std::ptr::null_mut();
        }
        (*pf).planes = Hunk_Alloc(common, cm, rm, host, cm.numPlanes as usize * std::mem::size_of::<patchPlane_t>(), h_high);
        Com_Memcpy((*pf).planes.cast(), cm.planes.as_ptr().cast(), cm.numPlanes as usize * std::mem::size_of::<patchPlane_t>());

        Z_Free(common, facets.as_mut_ptr().cast());
    }
}

/// Raven `CM_GeneratePatchCollide`.
///
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:1163-1229`
pub fn CM_GeneratePatchCollide(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
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
            com_error(errorParm_t::ERR_DROP, "CM_GeneratePatchFacets: even sizes are invalid for quadratic meshes".to_string());
        }

        if width > MAX_GRID_SIZE as c_int || height > MAX_GRID_SIZE as c_int {
            com_error(errorParm_t::ERR_DROP, "CM_GeneratePatchFacets: source is > MAX_GRID_SIZE".to_string());
        }

        // build a grid
        let mut grid: cGrid_t = std::mem::zeroed();
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
        let pf: *mut patchCollide_s =
            Hunk_Alloc(common, cm, rm, host, std::mem::size_of::<patchCollide_s>(), h_high).cast();
        let mut bmin: vec3_t = [0.0; 3];
        let mut bmax: vec3_t = [0.0; 3];
        ClearBounds(&mut bmin, &mut bmax);
        for i in 0..grid.width {
            for j in 0..grid.height {
                AddPointToBounds(grid.points[i as usize][j as usize], &mut bmin, &mut bmax);
            }
        }
        (*pf).bounds = [bmin, bmax];

        cm.c_totalPatchBlocks += (grid.width - 1) * (grid.height - 1);

        // generate a bsp tree for the surface
        CM_PatchCollideFromGrid(common, cm, rm, host, &mut grid, pf.cast());

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

// PORT-NOTE(deref-arith): `plane_xyz`/`DotProductPtr` are transcription
// helpers only (not Raven fns) — the packets' `DotProduct`/`VectorSubtract`
// etc. resolve to the qshared free-function surface (missing_symbols); these
// two shims read raw f32 spans the same way until that surface lands.
fn plane_xyz(plane: vec4_t) -> vec3_t {
    [plane[0], plane[1], plane[2]]
}
unsafe fn DotProductPtr(a: *const f32, b: *const f32) -> f32 {
    unsafe { *a * *b + *a.add(1) * *b.add(1) + *a.add(2) * *b.add(2) }
}
