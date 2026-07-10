#![allow(non_snake_case, non_camel_case_types, clippy::too_many_arguments)]
//! `cm_trace.cpp` — the box/capsule sweep tracer through the collision model
//! (brush/patch/terrain/tree walk + capsule-vs-capsule primitives).
//!
//! Source: `oracle/codemp/qcommon/cm_trace.cpp`
//!
//! PORT-NOTE(vector-math): Raven's `DotProduct`/`VectorAdd`/`VectorSubtract`/
//! `VectorCopy`/`VectorScale`/`VectorMA`/`VectorSet`/`VectorClear`/`VectorAdvance`/
//! `Square` (macros) and `VectorLength`/`VectorLengthSquared`/`VectorNormalize`/
//! `VectorInverse`/`AngleVectors` (free fns) are q_math primitives with no
//! reachable home in this crate's dependency graph yet (their only Rust port
//! lives in `mp_game`, a tier above the engine). Called here by their exact
//! Raven names/shapes (out-params as `&mut vec3_t`) per the no-stub rule;
//! reported as missing symbols for the finisher to wire to a q_math home
//! reachable from `mp_engine_qcommon` (e.g. relocated into `mp_qshared`).

use core::ffi::c_int;

use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::collision::cplane_t;
use mp_qshared::shared::{
    clipHandle_t, qfalse, qtrue, vec3_t, vec3pair_t, CONTENTS_BODY, CONTENTS_TERRAIN,
    CONTENTS_WATER,
};
// PORT-NOTE(vector-math): the file-level PORT-NOTE's q_math primitives now have a
// reachable home — the NAV-D3/RULING 39d migration relocated Raven's `q_math.c`
// vec3 helpers into `mp_qshared::shared::q_math` (the single engine-reachable,
// referee-compared definition). Raven's `Vector*`/`DotProduct`/`Square` MACRO
// names bind here to the reshaped `_`-prefixed / plain fns (inputs by value,
// outputs `&mut`) per the rosetta vec3 stanza.
use mp_qshared::shared::q_math::{
    AngleVectors, Square, VectorAdvance, VectorClear, VectorInverse, VectorLength,
    VectorLengthSquared, VectorNormalize, VectorSet, _DotProduct as DotProduct,
    _VectorAdd as VectorAdd, _VectorCopy as VectorCopy, _VectorMA as VectorMA,
    _VectorScale as VectorScale, _VectorSubtract as VectorSubtract,
};

use crate::cm::c_leaf_t::cLeaf_t;
use crate::cm::c_node_t::cNode_t;
use crate::cm::c_patch_t::cPatch_t;
use crate::cm::cbrush_s::cbrush_t;
use crate::cm::cbrushside_s::cbrushside_t;
use crate::cm::clip_map_t::clipMap_t;
use crate::cm::cm_landscape_consts::TERRAIN_STEP_MAGIC;
use crate::cm::cm_local_consts::{BOX_MODEL_HANDLE, CAPSULE_MODEL_HANDLE, SURFACE_CLIP_EPSILON};
use crate::cm::cm_trace_consts::{MAX_POSITION_LEAFS, RADIUS_EPSILON};
use crate::cm::cmodel_s::cmodel_t;
use crate::cm::leaf_list_s::leafList_t;
use crate::cm::sphere_t::sphere_t;
use crate::cm::trace_work_s::{traceWork_s, traceWork_t};
use crate::cm_load::{
    CCMLandScape, RenderModels, RmManager, CM_ClipHandleToModel, CM_ModelBounds, CM_TempBoxModel,
};
use crate::cm_test::{CM_BoxLeafnums_r, CM_StoreLeafs};
use crate::collision_world::CollisionWorld;
use crate::common::Common;
use crate::common_fns::Com_Memset;
use mp_host_interface::engine_host::EngineHost;

// PORT-NOTE(rm-types): `RmManager`/`RenderModels` are the state-receiver types
// pinned by the engine-fork-discovery preamble's receiver order (rmg-terrain.md
// / tr-model.md own their shape); neither has landed in the tree yet. Referenced
// by name only, per the no-stub rule — reported as missing symbols.
// PORT-NOTE(landscape): `CCMLandScape` is the rmg-terrain.md §F type owning
// `cmg.landScape` (currently a `*mut c_void` placeholder on `clipMap_t`);
// referenced by name only, reported as a missing symbol.

/// Raven `RotatePoint` — rotate `point` in place by `matrix` (row-major 3x3).
///
/// Raven: bk: FIXME.
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:43-50`
///
/// PORT-NOTE(signature-shape): the resolved signature takes `point: vec3_t` BY
/// VALUE (array), but Raven's C mutates the caller's array in place through the
/// decayed pointer — a real out-param the printed signature doesn't carry.
/// Transcribed literally against the LAW signature (mutates the local copy
/// only); flagged as a shape mismatch for the finisher (retype to `&mut vec3_t`
/// or `*mut vec3_t` to restore write-through).
pub fn RotatePoint(mut point: vec3_t, matrix: *mut vec3_t) {
    let mut tvec: vec3_t = [0.0; 3];
    VectorCopy(point, &mut tvec);
    unsafe {
        point[0] = DotProduct(*matrix.add(0), tvec);
        point[1] = DotProduct(*matrix.add(1), tvec);
        point[2] = DotProduct(*matrix.add(2), tvec);
    }
}

/// Raven `TransposeMatrix` — transpose a 3x3 matrix.
///
/// Raven: bk: FIXME.
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:57-64`
pub fn TransposeMatrix(matrix: *mut vec3_t, transpose: *mut vec3_t) {
    unsafe {
        for i in 0..3 {
            for j in 0..3 {
                (*transpose.add(i))[j] = (*matrix.add(j))[i];
            }
        }
    }
}

/// Raven `CreateRotationMatrix`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:71-74`
pub fn CreateRotationMatrix(angles: vec3_t, matrix: *mut vec3_t) {
    unsafe {
        AngleVectors(
            angles,
            Some(&mut *matrix.add(0)),
            Some(&mut *matrix.add(1)),
            Some(&mut *matrix.add(2)),
        );
        VectorInverse(&mut *matrix.add(1));
    }
}

/// Raven `CM_ProjectPointOntoVector`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:81-88`
pub fn CM_ProjectPointOntoVector(point: vec3_t, vStart: vec3_t, vDir: vec3_t, mut vProj: vec3_t) {
    let mut pVec: vec3_t = [0.0; 3];
    VectorSubtract(point, vStart, &mut pVec);
    // project onto the directional vector for this segment
    VectorMA(vStart, DotProduct(pVec, vDir), vDir, &mut vProj);
}

/// Raven `CM_VectorDistanceSquared`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:120-125`
pub fn CM_VectorDistanceSquared(p1: vec3_t, p2: vec3_t) -> f32 {
    let mut dir: vec3_t = [0.0; 3];
    VectorSubtract(p2, p1, &mut dir);
    VectorLengthSquared(dir)
}

/// Raven `SquareRootFloat` — the fast inverse-sqrt Newton-iteration hack.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:132-145`
pub fn SquareRootFloat(number: f32) -> f32 {
    let f: f32 = 1.5;
    let x = number * 0.5;
    let mut y = number;
    let mut i: i32 = y.to_bits() as i32;
    i = 0x5f3759df - (i >> 1);
    y = f32::from_bits(i as u32);
    y = y * (f - (x * y * y));
    y = y * (f - (x * y * y));
    number * y
}

/// Raven `CM_TestBoxInBrush`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:161-243`
pub fn CM_TestBoxInBrush(tw: *mut traceWork_t, trace: &mut trace_t, brush: *mut cbrush_t) {
    unsafe {
        if (*brush).numsides == 0 {
            return;
        }

        // special test for axial
        if (*tw).bounds[0][0] > (*brush).bounds[1][0]
            || (*tw).bounds[0][1] > (*brush).bounds[1][1]
            || (*tw).bounds[0][2] > (*brush).bounds[1][2]
            || (*tw).bounds[1][0] < (*brush).bounds[0][0]
            || (*tw).bounds[1][1] < (*brush).bounds[0][1]
            || (*tw).bounds[1][2] < (*brush).bounds[0][2]
        {
            return;
        }

        if (*tw).sphere.r#use != qfalse {
            // the first six planes are the axial planes, so we only
            // need to test the remainder
            for i in 6..(*brush).numsides as isize {
                let side: *mut cbrushside_t = (*brush).sides.offset(i);
                let plane: *mut cplane_t = (*side).plane;

                // adjust the plane distance apropriately for radius
                let dist = (*plane).dist + (*tw).sphere.radius;
                // find the closest point on the capsule to the plane
                let t = DotProduct((*plane).normal, (*tw).sphere.offset);
                let mut startp: vec3_t = [0.0; 3];
                if t > 0.0 {
                    VectorSubtract((*tw).start, (*tw).sphere.offset, &mut startp);
                } else {
                    VectorAdd((*tw).start, (*tw).sphere.offset, &mut startp);
                }
                let d1 = DotProduct(startp, (*plane).normal) - dist;
                // if completely in front of face, no intersection
                if d1 > 0.0 {
                    return;
                }
            }
        } else {
            // the first six planes are the axial planes, so we only
            // need to test the remainder
            for i in 6..(*brush).numsides as isize {
                let side: *mut cbrushside_t = (*brush).sides.offset(i);
                let plane: *mut cplane_t = (*side).plane;

                // adjust the plane distance apropriately for mins/maxs
                let dist = (*plane).dist
                    - DotProduct((*tw).offsets[(*plane).signbits as usize], (*plane).normal);

                let d1 = DotProduct((*tw).start, (*plane).normal) - dist;

                // if completely in front of face, no intersection
                if d1 > 0.0 {
                    return;
                }
            }
        }

        // inside this brush
        trace.startsolid = qtrue as u8;
        trace.allsolid = qtrue as u8;
        trace.fraction = 0.0;
        trace.contents = (*brush).contents;
    }
}

/// Raven `CM_PlaneCollision`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:523-600`
pub fn CM_PlaneCollision(tw: *mut traceWork_t, side: *mut cbrushside_t) -> bool {
    unsafe {
        let plane: *mut cplane_t = (*side).plane;

        // adjust the plane distance apropriately for mins/maxs
        let dist =
            (*plane).dist - DotProduct((*tw).offsets[(*plane).signbits as usize], (*plane).normal);

        let d1 = DotProduct((*tw).start, (*plane).normal) - dist;
        let d2 = DotProduct((*tw).end, (*plane).normal) - dist;

        if d2 > 0.0 {
            // endpoint is not in solid
            (*tw).getout = true;
        }
        if d1 > 0.0 {
            // startpoint is not in solid
            (*tw).startout = true;
        }

        // if completely in front of face, no intersection with the entire brush
        if d1 > 0.0 && (d2 >= SURFACE_CLIP_EPSILON || d2 >= d1) {
            return false;
        }

        // if it doesn't cross the plane, the plane isn't relevent
        if d1 <= 0.0 && d2 <= 0.0 {
            return true;
        }
        // crosses face
        if d1 > d2 {
            // enter
            let mut f = d1 - SURFACE_CLIP_EPSILON;
            if f < 0.0 {
                f = 0.0;
                if f > (*tw).enterFrac {
                    (*tw).enterFrac = f;
                    (*tw).clipplane = plane;
                    (*tw).leadside = side;
                }
            } else if f > (*tw).enterFrac * (d1 - d2) {
                (*tw).enterFrac = f / (d1 - d2);
                (*tw).clipplane = plane;
                (*tw).leadside = side;
            }
        } else {
            // leave
            let mut f = d1 + SURFACE_CLIP_EPSILON;
            if f < d1 - d2 {
                f = 1.0;
                if f < (*tw).leaveFrac {
                    (*tw).leaveFrac = f;
                }
            } else if f > (*tw).leaveFrac * (d1 - d2) {
                (*tw).leaveFrac = f / (d1 - d2);
            }
        }
        true
    }
}

/// Raven `CM_GenericBoxCollide`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:952-969`
pub fn CM_GenericBoxCollide(abounds: vec3pair_t, bbounds: vec3pair_t) -> bool {
    // Check for completely no intersection
    for i in 0..3 {
        if abounds[1][i] < bbounds[0][i] {
            return false;
        }
        if abounds[0][i] > bbounds[1][i] {
            return false;
        }
    }
    true
}

/// Raven `CM_CalcExtents`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:1550-1567`
pub fn CM_CalcExtents(start: vec3_t, end: vec3_t, tw: *const traceWork_t, mut bounds: vec3pair_t) {
    unsafe {
        for i in 0..3 {
            if start[i] < end[i] {
                bounds[0][i] = start[i] + (*tw).size[0][i];
                bounds[1][i] = end[i] + (*tw).size[1][i];
            } else {
                bounds[0][i] = end[i] + (*tw).size[0][i];
                bounds[1][i] = start[i] + (*tw).size[1][i];
            }
        }
    }
}

/// Raven `CM_CullBox`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:1947-1969`
pub fn CM_CullBox(frustum: *const cplane_t, transformed: *const vec3_t) -> bool {
    unsafe {
        // check against frustum planes
        for i in 0..4isize {
            let frust = frustum.offset(i);
            let mut j = 0;
            while j < 8 {
                if DotProduct(*transformed.offset(j), (*frust).normal) > (*frust).dist {
                    // a point is in front
                    break;
                }
                j += 1;
            }

            if j == 8 {
                // all points were behind one of the planes
                return true;
            }
        }
        false
    }
}

/// Raven `CM_DistanceFromLineSquared`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:95-113`
pub fn CM_DistanceFromLineSquared(p: vec3_t, lp1: vec3_t, lp2: vec3_t, dir: vec3_t) -> f32 {
    let mut proj: vec3_t = [0.0; 3];
    let mut t: vec3_t = [0.0; 3];

    CM_ProjectPointOntoVector(p, lp1, dir, proj);
    let mut j = 0usize;
    while j < 3 {
        if (proj[j] > lp1[j] && proj[j] > lp2[j]) || (proj[j] < lp1[j] && proj[j] < lp2[j]) {
            break;
        }
        j += 1;
    }
    if j < 3 {
        if (proj[j] - lp1[j]).abs() < (proj[j] - lp2[j]).abs() {
            VectorSubtract(p, lp1, &mut t);
        } else {
            VectorSubtract(p, lp2, &mut t);
        }
        return VectorLengthSquared(t);
    }
    VectorSubtract(p, proj, &mut t);
    VectorLengthSquared(t)
}

/// Raven `CM_TraceThroughBrush`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:607-690`
pub fn CM_TraceThroughBrush(
    cm: &mut CollisionWorld,
    tw: *mut traceWork_t,
    trace: &mut trace_t,
    brush: *mut cbrush_t,
    infoOnly: bool,
) {
    unsafe {
        (*tw).enterFrac = -1.0;
        (*tw).leaveFrac = 1.0;
        (*tw).clipplane = core::ptr::null_mut();

        if (*brush).numsides == 0 {
            return;
        }

        // I'm not sure if test is strictly correct.  Are all
        // bboxes axis aligned?  Do I care?  It seems to work
        // good enough...
        if (*tw).bounds[0][0] > (*brush).bounds[1][0]
            || (*tw).bounds[0][1] > (*brush).bounds[1][1]
            || (*tw).bounds[0][2] > (*brush).bounds[1][2]
            || (*tw).bounds[1][0] < (*brush).bounds[0][0]
            || (*tw).bounds[1][1] < (*brush).bounds[0][1]
            || (*tw).bounds[1][2] < (*brush).bounds[0][2]
        {
            return;
        }

        (*tw).getout = false;
        (*tw).startout = false;
        (*tw).leadside = core::ptr::null_mut();

        //
        // compare the trace against all planes of the brush
        // find the latest time the trace crosses a plane towards the interior
        // and the earliest time the trace crosses a plane towards the exterior
        //
        for i in 0..(*brush).numsides as isize {
            let side = (*brush).sides.offset(i);

            if !CM_PlaneCollision(tw, side) {
                return;
            }
        }

        //
        // all planes have been checked, and the trace was not
        // completely outside the brush
        //
        if !(*tw).startout {
            if !infoOnly {
                // original point was inside brush
                trace.startsolid = qtrue as u8;
                if !(*tw).getout {
                    trace.allsolid = qtrue as u8;
                    trace.fraction = 0.0;
                }
            }
            (*tw).enterFrac = 0.0;
            return;
        }

        if (*tw).enterFrac < (*tw).leaveFrac
            && (*tw).enterFrac > -1.0
            && (*tw).enterFrac < trace.fraction
        {
            if (*tw).enterFrac < 0.0 {
                (*tw).enterFrac = 0.0;
            }
            if !infoOnly {
                trace.fraction = (*tw).enterFrac;
                trace.plane = *(*tw).clipplane;
                trace.surfaceFlags = cm
                    .cmg
                    .shaders
                    .offset((*(*tw).leadside).shaderNum as isize)
                    .as_ref()
                    .map(|s| s.surfaceFlags)
                    .unwrap_or(0);
                trace.contents = (*brush).contents;
            }
        }
    }
}

/// Raven `CM_CullWorldBox`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:1979-1992`
pub fn CM_CullWorldBox(frustum: *const cplane_t, bounds: vec3pair_t) -> bool {
    let mut transformed: [vec3_t; 8] = [[0.0; 3]; 8];

    for i in 0..8 {
        transformed[i][0] = bounds[i & 1][0];
        transformed[i][1] = bounds[(i >> 1) & 1][1];
        transformed[i][2] = bounds[(i >> 2) & 1][2];
    }

    CM_CullBox(frustum, transformed.as_ptr())
}

/// Raven `CM_HandlePatchCollision`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:914-944`
pub fn CM_HandlePatchCollision(
    cm: &mut CollisionWorld,
    tw: *mut traceWork_s,
    trace: &mut trace_t,
    _tStart: vec3_t,
    _tEnd: vec3_t,
    patch: *mut CCMPatch,
    checkcount: c_int,
) {
    unsafe {
        // Get the collision data
        let mut brush = (*patch).GetCollisionData();
        let numBrushes = (*patch).GetNumBrushes();

        for _ in 0..numBrushes {
            if (*brush).checkcount == checkcount {
                return;
            }

            // Generic collision of terxel bounds to line segment bounds
            if !CM_GenericBoxCollide((*brush).bounds, (*tw).localBounds) {
                brush = brush.add(1);
                continue;
            }

            (*brush).checkcount = checkcount;

            CM_TraceThroughBrush(cm, tw, trace, brush, false);
            if trace.fraction <= 0.0 {
                break;
            }
            brush = brush.add(1);
        }
    }
}

// PORT-NOTE(traceWork_s-alias): the packet resolves `CM_HandlePatchCollision`'s
// tw param as `*mut traceWork_s` (the C struct tag), same layout as `traceWork_t`,
// imported at the top of the file.

// PORT-NOTE(rmg-type): `CCMPatch` is a §F type owned by `docs/subsystems/rmg-terrain.md`
// (ruling 16/41); not yet ported into this crate. Referenced by name only.
// PORT-NOTE(brush-checkcount-width): `cbrush_t::checkcount` is `u16`; the packet's
// `checkcount: c_int` param is compared/assigned directly against it here exactly as
// Raven's C implicit conversion would — flagged as a shape mismatch (brush.checkcount
// vs c_int) for the referee if the widths ever diverge in practice (Raven's own
// checkcount counters can exceed u16 range over a long session).

/// Raven `CM_TraceThroughSphere`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:1058-1136`
pub fn CM_TraceThroughSphere(
    tw: *mut traceWork_t,
    trace: &mut trace_t,
    origin: vec3_t,
    radius: f32,
    start: vec3_t,
    end: vec3_t,
) {
    unsafe {
        let mut dir: vec3_t = [0.0; 3];
        let mut v1: vec3_t = [0.0; 3];
        let mut intersection: vec3_t = [0.0; 3];

        // if inside the sphere
        VectorSubtract(start, origin, &mut dir);
        let mut l1 = VectorLengthSquared(dir);
        if l1 < Square(radius) {
            trace.fraction = 0.0;
            trace.startsolid = qtrue as u8;
            // test for allsolid
            VectorSubtract(end, origin, &mut dir);
            l1 = VectorLengthSquared(dir);
            if l1 < Square(radius) {
                trace.allsolid = qtrue as u8;
            }
            return;
        }
        //
        VectorSubtract(end, start, &mut dir);
        let length = VectorNormalize(&mut dir);
        //
        l1 = CM_DistanceFromLineSquared(origin, start, end, dir);
        VectorSubtract(end, origin, &mut v1);
        let l2 = VectorLengthSquared(v1);
        // if no intersection with the sphere and the end point is at least an epsilon away
        if l1 >= Square(radius) && l2 > Square(radius + SURFACE_CLIP_EPSILON) {
            return;
        }
        //
        VectorSubtract(start, origin, &mut v1);
        // dir is normalized so a = 1
        let b = 2.0 * (dir[0] * v1[0] + dir[1] * v1[1] + dir[2] * v1[2]);
        let c = v1[0] * v1[0] + v1[1] * v1[1] + v1[2] * v1[2]
            - (radius + RADIUS_EPSILON) * (radius + RADIUS_EPSILON);

        let d = b * b - 4.0 * c;
        if d > 0.0 {
            let sqrtd = SquareRootFloat(d);
            let mut fraction = (-b - sqrtd) * 0.5;
            //
            if fraction < 0.0 {
                fraction = 0.0;
            } else {
                fraction /= length;
            }
            if fraction < trace.fraction {
                trace.fraction = fraction;
                VectorSubtract(end, start, &mut dir);
                VectorMA(start, fraction, dir, &mut intersection);
                VectorSubtract(intersection, origin, &mut dir);
                let scale = 1.0 / (radius + RADIUS_EPSILON);
                VectorScale(dir, scale, &mut dir);
                VectorCopy(dir, &mut trace.plane.normal);
                VectorAdd((*tw).modelOrigin, intersection, &mut intersection);
                trace.plane.dist = DotProduct(trace.plane.normal, intersection);
                trace.contents = CONTENTS_BODY;
            }
        }
        // else if d == 0: slide along the sphere (no-op, matches Raven)
        // no intersection at all
    }
}

/// Raven `CM_TraceThroughVerticalCylinder`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:1146-1240`
pub fn CM_TraceThroughVerticalCylinder(
    tw: *mut traceWork_t,
    trace: &mut trace_t,
    origin: vec3_t,
    radius: f32,
    halfheight: f32,
    start: vec3_t,
    end: vec3_t,
) {
    unsafe {
        let mut dir: vec3_t = [0.0; 3];
        let mut v1: vec3_t = [0.0; 3];
        let mut intersection: vec3_t = [0.0; 3];

        // 2d coordinates
        let mut start2d: vec3_t = [0.0; 3];
        let mut end2d: vec3_t = [0.0; 3];
        let mut org2d: vec3_t = [0.0; 3];
        VectorSet(&mut start2d, start[0], start[1], 0.0);
        VectorSet(&mut end2d, end[0], end[1], 0.0);
        VectorSet(&mut org2d, origin[0], origin[1], 0.0);
        // if between lower and upper cylinder bounds
        if start[2] <= origin[2] + halfheight && start[2] >= origin[2] - halfheight {
            // if inside the cylinder
            VectorSubtract(start2d, org2d, &mut dir);
            let mut l1 = VectorLengthSquared(dir);
            if l1 < Square(radius) {
                trace.fraction = 0.0;
                trace.startsolid = qtrue as u8;
                VectorSubtract(end2d, org2d, &mut dir);
                l1 = VectorLengthSquared(dir);
                if l1 < Square(radius) {
                    trace.allsolid = qtrue as u8;
                }
                return;
            }
        }
        //
        VectorSubtract(end2d, start2d, &mut dir);
        let length = VectorNormalize(&mut dir);
        //
        let l1 = CM_DistanceFromLineSquared(org2d, start2d, end2d, dir);
        VectorSubtract(end2d, org2d, &mut v1);
        let l2 = VectorLengthSquared(v1);
        // if no intersection with the cylinder and the end point is at least an epsilon away
        if l1 >= Square(radius) && l2 > Square(radius + SURFACE_CLIP_EPSILON) {
            return;
        }
        //
        VectorSubtract(start, origin, &mut v1);
        // dir is normalized so we can use a = 1
        let b = 2.0 * (v1[0] * dir[0] + v1[1] * dir[1]);
        let c =
            v1[0] * v1[0] + v1[1] * v1[1] - (radius + RADIUS_EPSILON) * (radius + RADIUS_EPSILON);

        let d = b * b - 4.0 * c;
        if d > 0.0 {
            let sqrtd = SquareRootFloat(d);
            let mut fraction = (-b - sqrtd) * 0.5;
            //
            if fraction < 0.0 {
                fraction = 0.0;
            } else {
                fraction /= length;
            }
            if fraction < trace.fraction {
                VectorSubtract(end, start, &mut dir);
                VectorMA(start, fraction, dir, &mut intersection);
                // if the intersection is between the cylinder lower and upper bound
                if intersection[2] <= origin[2] + halfheight
                    && intersection[2] >= origin[2] - halfheight
                {
                    //
                    trace.fraction = fraction;
                    VectorSubtract(intersection, origin, &mut dir);
                    dir[2] = 0.0;
                    let scale = 1.0 / (radius + RADIUS_EPSILON);
                    VectorScale(dir, scale, &mut dir);
                    VectorCopy(dir, &mut trace.plane.normal);
                    VectorAdd((*tw).modelOrigin, intersection, &mut intersection);
                    trace.plane.dist = DotProduct(trace.plane.normal, intersection);
                    trace.contents = CONTENTS_BODY;
                }
            }
        }
        // else if d == 0: slide along the cylinder (no-op, matches Raven)
        // no intersection at all
    }
}

/// Raven `CM_TraceThroughTerrain`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:703-798`
pub fn CM_TraceThroughTerrain(
    cm: &mut CollisionWorld,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    tw: *mut traceWork_t,
    trace: &mut trace_t,
    brush: *mut cbrush_t,
) {
    unsafe {
        // At this point we know we may be colliding with a terrain brush (and we know we have a valid terrain structure)
        // PORT-NOTE(landscape-receiver): the packet's `cmg.landScape->Method(...)`
        // calls resolve through `CollisionWorld::terrain_*` (ruling 38's
        // receiver-shape repair — `CmLandScape` lives at `cm.land_scape`, not a
        // raw `cmg.landScape` pointer); see `cm_terrain.rs`'s `impl CollisionWorld`
        // block.

        // Check for absolutely no connection
        if !CM_GenericBoxCollide((*tw).bounds, *cm.terrain_bounds()) {
            return;
        }
        // Now we know that at least some part of the trace needs to collide with the terrain
        // The regular brush collision is handled elsewhere, so advance the ray to an edge in the terrain brush
        CM_TraceThroughBrush(cm, tw, trace, brush, true);

        // Remember the base entering and leaving fractions
        (*tw).baseEnterFrac = (*tw).enterFrac;
        (*tw).baseLeaveFrac = (*tw).leaveFrac;
        // Reset to full spread within the brush
        (*tw).enterFrac = -1.0;
        (*tw).leaveFrac = 1.0;

        // Work out the corners of the AABB when the trace first hits the terrain brush and when it leaves
        let mut tBegin: vec3_t = [0.0; 3];
        let mut tEnd: vec3_t = [0.0; 3];
        let mut tDistance: vec3_t = [0.0; 3];
        let mut tStep: vec3_t = [0.0; 3];
        VectorAdvance((*tw).start, (*tw).baseEnterFrac, (*tw).end, &mut tBegin);
        VectorAdvance((*tw).start, (*tw).baseLeaveFrac, (*tw).end, &mut tEnd);
        VectorSubtract(tEnd, tBegin, &mut tDistance);

        // Calculate number of iterations to process
        let mut count = (VectorLength(tDistance)
            / (cm.terrain_patch_scalar_size() * TERRAIN_STEP_MAGIC))
            .ceil() as i32;
        count = 1;
        let fraction = trace.fraction;
        VectorScale(tDistance, 1.0 / count as f32, &mut tStep);

        // Save the base start and end vectors
        let mut baseStart: vec3_t = [0.0; 3];
        let mut baseEnd: vec3_t = [0.0; 3];
        VectorCopy((*tw).start, &mut baseStart);
        VectorCopy((*tw).end, &mut baseEnd);

        // Use the terrain vectors.  Start both at the beginning since the
        // step will be added to the end as the first step of the loop
        VectorCopy(tBegin, &mut (*tw).start);
        VectorCopy(tBegin, &mut (*tw).end);

        // Step thru terrain patches moving on about 1 patch at a time
        for i in 0..count {
            // Add the step to the end
            let mut end_copy = (*tw).end;
            VectorAdd(end_copy, tStep, &mut end_copy);
            (*tw).end = end_copy;

            CM_CalcExtents(tBegin, (*tw).end, tw, (*tw).localBounds);

            cm.terrain_patch_collide(
                &mut *tw,
                trace,
                (*tw).start,
                (*tw).end,
                (*brush).checkcount as c_int,
            );

            // If collision with something closer than water then just stop here
            if trace.fraction < fraction {
                // Convert the fraction of this sub tract into the full trace's fraction
                trace.fraction =
                    i as f32 * (1.0 / count as f32) + (1.0 / count as f32) * trace.fraction;
                break;
            }

            // Move the end to the start so the next trace starts
            // where this one left off
            VectorCopy((*tw).end, &mut (*tw).start);
        }

        // Put the original start and end back
        VectorCopy(baseStart, &mut (*tw).start);
        VectorCopy(baseEnd, &mut (*tw).end);

        // Convert to global fraction only if something was hit along the way
        if trace.fraction != 1.0 {
            trace.fraction = (*tw).baseEnterFrac
                + (((*tw).baseLeaveFrac - (*tw).baseEnterFrac) * trace.fraction);
            trace.contents = (*brush).contents;
        }

        // Collide with any water
        if (*tw).contents & CONTENTS_WATER != 0 {
            let fraction = cm.terrain_water_collide((*tw).start, (*tw).end, trace.fraction);
            if fraction < trace.fraction {
                VectorSet(&mut trace.plane.normal, 0.0, 0.0, 1.0);
                trace.contents = cm.terrain_water_contents();
                trace.fraction = fraction;
                trace.surfaceFlags = cm.terrain_water_surface_flags();
            }
        }
    }
}

/// Raven `CM_TestInLeaf`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:262-333`
pub fn CM_TestInLeaf(
    cm: &mut CollisionWorld,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    tw: *mut traceWork_t,
    trace: &mut trace_t,
    leaf: *mut cLeaf_t,
    local: *mut clipMap_t,
) {
    unsafe {
        // test box position against all brushes in the leaf
        for k in 0..(*leaf).numLeafBrushes {
            let brushnum = *(*local)
                .leafbrushes
                .offset(((*leaf).firstLeafBrush + k) as isize);
            let b: *mut cbrush_t = (*local).brushes.offset(brushnum as isize);
            if (*b).checkcount as c_int == (*local).checkcount {
                continue; // already checked this brush in another leaf
            }
            (*b).checkcount = (*local).checkcount as u16;

            if (*b).contents & (*tw).contents == 0 {
                continue;
            }

            if cm.cm_terrainPhysics != 0
                && !cm.cmg.landScape.is_null()
                && ((*b).contents & CONTENTS_TERRAIN != 0)
            {
                // Invalidate the checkcount for terrain as the terrain brush has to be processed
                // many times.
                (*b).checkcount -= 1;

                CM_TraceThroughTerrain(cm, rmg, host, tw, trace, b);
                // If inside a terrain brush don't bother with regular brush collision
                continue;
            }
            CM_TestBoxInBrush(tw, trace, b);
            if trace.allsolid != 0 {
                return;
            }
        }

        // test against all patches
        if cm.cm_noCurves == 0 {
            for k in 0..(*leaf).numLeafSurfaces {
                let patch: *mut cPatch_t = *(*local).surfaces.offset(
                    *(*local)
                        .leafsurfaces
                        .offset(((*leaf).firstLeafSurface + k) as isize)
                        as isize,
                );
                if patch.is_null() {
                    continue;
                }
                if (*patch).checkcount == (*local).checkcount {
                    continue; // already checked this brush in another leaf
                }
                (*patch).checkcount = (*local).checkcount;

                if (*patch).contents & (*tw).contents == 0 {
                    continue;
                }

                if CM_PositionTestInPatchCollide(tw, (*patch).pc) {
                    trace.startsolid = qtrue as u8;
                    trace.allsolid = qtrue as u8;
                    trace.fraction = 0.0;
                    trace.contents = (*patch).contents;
                    return;
                }
            }
        }
    }
}

// PORT-NOTE(cvar-fields): `cm_noCurves`/`com_terrainPhysics` are `cvar_t*` globals
// (`cm_local.h:224`, `cm_landscape.h:267`); referenced here as plain integer fields
// (`cm.cm_noCurves`/`cm.cm_terrainPhysics`) on the `cm: &mut CollisionWorld` receiver
// per ruling 2 (spelled by their Raven name), standing in for `->integer` reads —
// exact field/cvar-handle shape is CollisionWorld's own (STATE-D2), not decided here.

/// Raven `CM_PositionTest`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:448-483`
pub fn CM_PositionTest(
    cm: &mut CollisionWorld,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    tw: *mut traceWork_t,
    trace: &mut trace_t,
) {
    unsafe {
        // §19: Raven leaves `leafs` uninitialized before `CM_BoxLeafnums_r` fills it;
        // zero-init here to avoid reading UB memory.
        let mut leafs: [c_int; MAX_POSITION_LEAFS] = [0; MAX_POSITION_LEAFS];
        let mut ll: leafList_t = core::mem::zeroed();

        // identify the leafs we are touching
        VectorAdd((*tw).start, (*tw).size[0], &mut ll.bounds[0]);
        VectorAdd((*tw).start, (*tw).size[1], &mut ll.bounds[1]);

        for i in 0..3 {
            ll.bounds[0][i] -= 1.0;
            ll.bounds[1][i] += 1.0;
        }

        ll.count = 0;
        ll.maxcount = MAX_POSITION_LEAFS as c_int;
        ll.list = leafs.as_mut_ptr();
        ll.storeLeafs = Some(CM_StoreLeafs);
        ll.lastLeaf = 0;
        ll.overflowed = qfalse;

        cm.cmg.checkcount += 1;

        CM_BoxLeafnums_r(cm, &mut ll, 0);

        cm.cmg.checkcount += 1;

        // test the contents of the leafs
        for i in 0..ll.count {
            CM_TestInLeaf(
                cm,
                rmg,
                host,
                tw,
                trace,
                &mut *cm.cmg.leafs.add(leafs[i as usize] as usize),
                &mut cm.cmg,
            );
            if trace.allsolid != 0 {
                break;
            }
        }
    }
}

/// Raven `CM_TestCapsuleInCapsule`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:342-402`
pub fn CM_TestCapsuleInCapsule(
    cm: &mut CollisionWorld,
    tw: *mut traceWork_t,
    trace: &mut trace_t,
    model: clipHandle_t,
) {
    unsafe {
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut top: vec3_t = [0.0; 3];
        let mut bottom: vec3_t = [0.0; 3];
        let mut p1: vec3_t = [0.0; 3];
        let mut p2: vec3_t = [0.0; 3];
        let mut tmp: vec3_t = [0.0; 3];
        let mut offset: vec3_t = [0.0; 3];
        let mut symetricSize: vec3pair_t = [[0.0; 3]; 2];

        // PORT-NOTE(shape-mismatch): `CM_ModelBounds`'s LAW signature (cm_load.rs)
        // takes `mins`/`maxs` by value (documented out-param non-propagation);
        // call bent to match — the fill does not write back here.
        CM_ModelBounds(cm, model, mins, maxs);

        VectorAdd((*tw).start, (*tw).sphere.offset, &mut top);
        VectorSubtract((*tw).start, (*tw).sphere.offset, &mut bottom);
        for i in 0..3 {
            offset[i] = (mins[i] + maxs[i]) * 0.5;
            symetricSize[0][i] = mins[i] - offset[i];
            symetricSize[1][i] = maxs[i] - offset[i];
        }
        let halfwidth = symetricSize[1][0];
        let halfheight = symetricSize[1][2];
        let radius = if halfwidth > halfheight {
            halfheight
        } else {
            halfwidth
        };
        let offs = halfheight - radius;

        let r = Square((*tw).sphere.radius + radius);
        // check if any of the spheres overlap
        VectorCopy(offset, &mut p1);
        p1[2] += offs;
        VectorSubtract(p1, top, &mut tmp);
        if VectorLengthSquared(tmp) < r {
            trace.startsolid = qtrue as u8;
            trace.allsolid = qtrue as u8;
            trace.fraction = 0.0;
        }
        VectorSubtract(p1, bottom, &mut tmp);
        if VectorLengthSquared(tmp) < r {
            trace.startsolid = qtrue as u8;
            trace.allsolid = qtrue as u8;
            trace.fraction = 0.0;
        }
        VectorCopy(offset, &mut p2);
        p2[2] -= offs;
        VectorSubtract(p2, top, &mut tmp);
        if VectorLengthSquared(tmp) < r {
            trace.startsolid = qtrue as u8;
            trace.allsolid = qtrue as u8;
            trace.fraction = 0.0;
        }
        VectorSubtract(p2, bottom, &mut tmp);
        if VectorLengthSquared(tmp) < r {
            trace.startsolid = qtrue as u8;
            trace.allsolid = qtrue as u8;
            trace.fraction = 0.0;
        }
        // if between cylinder up and lower bounds
        if (top[2] >= p1[2] && top[2] <= p2[2]) || (bottom[2] >= p1[2] && bottom[2] <= p2[2]) {
            // 2d coordinates
            top[2] = 0.0;
            p1[2] = 0.0;
            // if the cylinders overlap
            VectorSubtract(top, p1, &mut tmp);
            if VectorLengthSquared(tmp) < r {
                trace.startsolid = qtrue as u8;
                trace.allsolid = qtrue as u8;
                trace.fraction = 0.0;
            }
        }
    }
}

/// Raven `CM_TestBoundingBoxInCapsule`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:411-440`
pub fn CM_TestBoundingBoxInCapsule(
    cm: &mut CollisionWorld,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    tw: *mut traceWork_t,
    trace: &mut trace_t,
    model: clipHandle_t,
) {
    unsafe {
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut offset: vec3_t = [0.0; 3];
        let mut size: vec3pair_t = [[0.0; 3]; 2];

        // mins maxs of the capsule
        // PORT-NOTE(shape-mismatch): `CM_ModelBounds`'s LAW signature (cm_load.rs)
        // takes `mins`/`maxs` by value (documented out-param non-propagation);
        // call bent to match — the fill does not write back here.
        CM_ModelBounds(cm, model, mins, maxs);

        // offset for capsule center
        for i in 0..3 {
            offset[i] = (mins[i] + maxs[i]) * 0.5;
            size[0][i] = mins[i] - offset[i];
            size[1][i] = maxs[i] - offset[i];
            (*tw).start[i] -= offset[i];
            (*tw).end[i] -= offset[i];
        }

        // replace the bounding box with the capsule
        (*tw).sphere.r#use = qtrue;
        (*tw).sphere.radius = if size[1][0] > size[1][2] {
            size[1][2]
        } else {
            size[1][0]
        };
        (*tw).sphere.halfheight = size[1][2];
        VectorSet(
            &mut (*tw).sphere.offset,
            0.0,
            0.0,
            size[1][2] - (*tw).sphere.radius,
        );

        // replace the capsule with the bounding box
        let h = CM_TempBoxModel(cm, (*tw).size[0], (*tw).size[1], qfalse);
        // calculate collision
        let cmod: *mut cmodel_t = CM_ClipHandleToModel(cm, h, core::ptr::null_mut());
        CM_TestInLeaf(cm, rmg, host, tw, trace, &mut (*cmod).leaf, &mut cm.cmg);
    }
}

/// Raven `CM_TraceThroughPatch`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:500-513`
pub fn CM_TraceThroughPatch(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    tw: *mut traceWork_t,
    trace: &mut trace_t,
    patch: *mut cPatch_t,
) {
    unsafe {
        common.c_patch_traces += 1;

        let oldFrac = trace.fraction;

        CM_TraceThroughPatchCollide(common, cm, rm, host, tw, trace, (*patch).pc);

        if trace.fraction < oldFrac {
            trace.surfaceFlags = (*patch).surfaceFlags;
            trace.contents = (*patch).contents;
        }
    }
}

/// Raven `CM_TraceCapsuleThroughCapsule`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:1249-1302`
pub fn CM_TraceCapsuleThroughCapsule(
    cm: &mut CollisionWorld,
    tw: *mut traceWork_t,
    trace: &mut trace_t,
    model: clipHandle_t,
) {
    unsafe {
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut top: vec3_t = [0.0; 3];
        let mut bottom: vec3_t = [0.0; 3];
        let mut starttop: vec3_t = [0.0; 3];
        let mut startbottom: vec3_t = [0.0; 3];
        let mut endtop: vec3_t = [0.0; 3];
        let mut endbottom: vec3_t = [0.0; 3];
        let mut offset: vec3_t = [0.0; 3];
        let mut symetricSize: vec3pair_t = [[0.0; 3]; 2];

        // PORT-NOTE(shape-mismatch): `CM_ModelBounds`'s LAW signature (cm_load.rs)
        // takes `mins`/`maxs` by value (documented out-param non-propagation);
        // call bent to match — the fill does not write back here.
        CM_ModelBounds(cm, model, mins, maxs);
        // test trace bounds vs. capsule bounds
        if (*tw).bounds[0][0] > maxs[0] + RADIUS_EPSILON
            || (*tw).bounds[0][1] > maxs[1] + RADIUS_EPSILON
            || (*tw).bounds[0][2] > maxs[2] + RADIUS_EPSILON
            || (*tw).bounds[1][0] < mins[0] - RADIUS_EPSILON
            || (*tw).bounds[1][1] < mins[1] - RADIUS_EPSILON
            || (*tw).bounds[1][2] < mins[2] - RADIUS_EPSILON
        {
            return;
        }
        // top origin and bottom origin of each sphere at start and end of trace
        VectorAdd((*tw).start, (*tw).sphere.offset, &mut starttop);
        VectorSubtract((*tw).start, (*tw).sphere.offset, &mut startbottom);
        VectorAdd((*tw).end, (*tw).sphere.offset, &mut endtop);
        VectorSubtract((*tw).end, (*tw).sphere.offset, &mut endbottom);

        // calculate top and bottom of the capsule spheres to collide with
        for i in 0..3 {
            offset[i] = (mins[i] + maxs[i]) * 0.5;
            symetricSize[0][i] = mins[i] - offset[i];
            symetricSize[1][i] = maxs[i] - offset[i];
        }
        let halfwidth = symetricSize[1][0];
        let halfheight = symetricSize[1][2];
        let mut radius = if halfwidth > halfheight {
            halfheight
        } else {
            halfwidth
        };
        let offs = halfheight - radius;
        VectorCopy(offset, &mut top);
        top[2] += offs;
        VectorCopy(offset, &mut bottom);
        bottom[2] -= offs;
        // expand radius of spheres
        radius += (*tw).sphere.radius;
        // if there is horizontal movement
        if (*tw).start[0] != (*tw).end[0] || (*tw).start[1] != (*tw).end[1] {
            // height of the expanded cylinder is the height of both cylinders minus the radius of both spheres
            let h = halfheight + (*tw).sphere.halfheight - radius;
            // if the cylinder has a height
            if h > 0.0 {
                // test for collisions between the cylinders
                CM_TraceThroughVerticalCylinder(
                    tw,
                    trace,
                    offset,
                    radius,
                    h,
                    (*tw).start,
                    (*tw).end,
                );
            }
        }
        // test for collision between the spheres
        CM_TraceThroughSphere(tw, trace, top, radius, startbottom, endbottom);
        CM_TraceThroughSphere(tw, trace, bottom, radius, starttop, endtop);
    }
}

/// Raven `CM_TraceThroughLeaf`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:976-1047`
pub fn CM_TraceThroughLeaf(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    tw: *mut traceWork_t,
    trace: &mut trace_t,
    local: *mut clipMap_t,
    leaf: *mut cLeaf_t,
) {
    unsafe {
        // trace line against all brushes in the leaf
        for k in 0..(*leaf).numLeafBrushes {
            let brushnum = *(*local)
                .leafbrushes
                .offset(((*leaf).firstLeafBrush + k) as isize);

            let b: *mut cbrush_t = (*local).brushes.offset(brushnum as isize);
            if (*b).checkcount as c_int == (*local).checkcount {
                continue; // already checked this brush in another leaf
            }
            (*b).checkcount = (*local).checkcount as u16;

            if (*b).contents & (*tw).contents == 0 {
                continue;
            }

            if cm.cm_terrainPhysics != 0
                && !cm.cmg.landScape.is_null()
                && ((*b).contents & CONTENTS_TERRAIN != 0)
            {
                // Invalidate the checkcount for terrain as the terrain brush has to be processed
                // many times.
                (*b).checkcount -= 1;

                CM_TraceThroughTerrain(cm, rmg, host, tw, trace, b);
            } else {
                CM_TraceThroughBrush(cm, tw, trace, b, false);
            }

            if trace.fraction == 0.0 {
                return;
            }
        }

        // trace line against all patches in the leaf
        if cm.cm_noCurves == 0 {
            for k in 0..(*leaf).numLeafSurfaces {
                let patch: *mut cPatch_t = *(*local).surfaces.offset(
                    *(*local)
                        .leafsurfaces
                        .offset(((*leaf).firstLeafSurface + k) as isize)
                        as isize,
                );
                if patch.is_null() {
                    continue;
                }
                if (*patch).checkcount == (*local).checkcount {
                    continue; // already checked this patch in another leaf
                }
                (*patch).checkcount = (*local).checkcount;

                if (*patch).contents & (*tw).contents == 0 {
                    continue;
                }

                CM_TraceThroughPatch(common, cm, rm, host, tw, trace, patch);
                if trace.fraction == 0.0 {
                    return;
                }
            }
        }
    }
}

/// Raven `CM_TraceToLeaf`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:1349-1419`
pub fn CM_TraceToLeaf(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    tw: *mut traceWork_t,
    trace: &mut trace_t,
    leaf: *mut cLeaf_t,
    local: *mut clipMap_t,
) {
    unsafe {
        // trace line against all brushes in the leaf
        for k in 0..(*leaf).numLeafBrushes {
            let brushnum = *(*local)
                .leafbrushes
                .offset(((*leaf).firstLeafBrush + k) as isize);

            let b: *mut cbrush_t = (*local).brushes.offset(brushnum as isize);
            if (*b).checkcount as c_int == (*local).checkcount {
                continue; // already checked this brush in another leaf
            }
            (*b).checkcount = (*local).checkcount as u16;

            if (*b).contents & (*tw).contents == 0 {
                continue;
            }

            if cm.cm_terrainPhysics != 0
                && !cm.cmg.landScape.is_null()
                && ((*b).contents & CONTENTS_TERRAIN != 0)
            {
                // Invalidate the checkcount for terrain as the terrain brush has to be processed
                // many times.
                (*b).checkcount -= 1;

                CM_TraceThroughTerrain(cm, rmg, host, tw, trace, b);
                // If inside a terrain brush don't bother with regular brush collision
                continue;
            }

            CM_TraceThroughBrush(cm, tw, trace, b, false);
            if trace.fraction == 0.0 {
                return;
            }
        }

        // trace line against all patches in the leaf
        if cm.cm_noCurves == 0 {
            for k in 0..(*leaf).numLeafSurfaces {
                let patch: *mut cPatch_t = *(*local).surfaces.offset(
                    *(*local)
                        .leafsurfaces
                        .offset(((*leaf).firstLeafSurface + k) as isize)
                        as isize,
                );
                if patch.is_null() {
                    continue;
                }
                if (*patch).checkcount == (*local).checkcount {
                    continue; // already checked this patch in another leaf
                }
                (*patch).checkcount = (*local).checkcount;

                if (*patch).contents & (*tw).contents == 0 {
                    continue;
                }

                CM_TraceThroughPatch(common, cm, rm, host, tw, trace, patch);
                if trace.fraction == 0.0 {
                    return;
                }
            }
        }
    }
}

/// Raven `CM_TraceBoundingBoxThroughCapsule`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:1311-1340`
pub fn CM_TraceBoundingBoxThroughCapsule(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    tw: *mut traceWork_t,
    trace: &mut trace_t,
    model: clipHandle_t,
) {
    unsafe {
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut offset: vec3_t = [0.0; 3];
        let mut size: vec3pair_t = [[0.0; 3]; 2];

        // mins maxs of the capsule
        // PORT-NOTE(shape-mismatch): `CM_ModelBounds`'s LAW signature (cm_load.rs)
        // takes `mins`/`maxs` by value (documented out-param non-propagation);
        // call bent to match — the fill does not write back here.
        CM_ModelBounds(cm, model, mins, maxs);

        // offset for capsule center
        for i in 0..3 {
            offset[i] = (mins[i] + maxs[i]) * 0.5;
            size[0][i] = mins[i] - offset[i];
            size[1][i] = maxs[i] - offset[i];
            (*tw).start[i] -= offset[i];
            (*tw).end[i] -= offset[i];
        }

        // replace the bounding box with the capsule
        (*tw).sphere.r#use = qtrue;
        (*tw).sphere.radius = if size[1][0] > size[1][2] {
            size[1][2]
        } else {
            size[1][0]
        };
        (*tw).sphere.halfheight = size[1][2];
        VectorSet(
            &mut (*tw).sphere.offset,
            0.0,
            0.0,
            size[1][2] - (*tw).sphere.radius,
        );

        // replace the capsule with the bounding box
        let h = CM_TempBoxModel(cm, (*tw).size[0], (*tw).size[1], qfalse);
        // calculate collision
        let cmod: *mut cmodel_t = CM_ClipHandleToModel(cm, h, core::ptr::null_mut());
        CM_TraceThroughLeaf(
            common,
            cm,
            rm,
            rmg,
            host,
            tw,
            trace,
            &mut cm.cmg,
            &mut (*cmod).leaf,
        );
    }
}

/// Raven `CM_TraceThroughTree`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:1431-1548`
pub fn CM_TraceThroughTree(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    tw: *mut traceWork_t,
    trace: &mut trace_t,
    local: *mut clipMap_t,
    num: c_int,
    p1f: f32,
    p2f: f32,
    p1: vec3_t,
    p2: vec3_t,
) {
    unsafe {
        if trace.fraction <= p1f {
            return; // already hit something nearer
        }

        // if < 0, we are in a leaf node
        if num < 0 {
            CM_TraceThroughLeaf(
                common,
                cm,
                rm,
                rmg,
                host,
                tw,
                trace,
                local,
                &mut *(*local).leafs.add((-1 - num) as usize),
            );
            return;
        }

        //
        // find the point distances to the seperating plane
        // and the offset for the size of the box
        //
        let node: *mut cNode_t = (*local).nodes.offset(num as isize);
        let plane: *mut cplane_t = (*node).plane;

        // adjust the plane distance apropriately for mins/maxs
        let (t1, t2, offset);
        if (*plane).r#type < 3 {
            t1 = p1[(*plane).r#type as usize] - (*plane).dist;
            t2 = p2[(*plane).r#type as usize] - (*plane).dist;
            offset = (*tw).extents[(*plane).r#type as usize];
        } else {
            t1 = DotProduct((*plane).normal, p1) - (*plane).dist;
            t2 = DotProduct((*plane).normal, p2) - (*plane).dist;
            if (*tw).isPoint != 0 {
                offset = 0.0;
            } else {
                // this is silly
                offset = 2048.0;
            }
        }

        // see which sides we need to consider
        if t1 >= offset + 1.0 && t2 >= offset + 1.0 {
            CM_TraceThroughTree(
                common,
                cm,
                rm,
                rmg,
                host,
                tw,
                trace,
                local,
                (*node).children[0],
                p1f,
                p2f,
                p1,
                p2,
            );
            return;
        }
        if t1 < -offset - 1.0 && t2 < -offset - 1.0 {
            CM_TraceThroughTree(
                common,
                cm,
                rm,
                rmg,
                host,
                tw,
                trace,
                local,
                (*node).children[1],
                p1f,
                p2f,
                p1,
                p2,
            );
            return;
        }

        // put the crosspoint SURFACE_CLIP_EPSILON pixels on the near side
        let (side, mut frac, mut frac2);
        if t1 < t2 {
            let idist = 1.0 / (t1 - t2);
            side = 1;
            frac2 = (t1 + offset + SURFACE_CLIP_EPSILON) * idist;
            frac = (t1 - offset + SURFACE_CLIP_EPSILON) * idist;
        } else if t1 > t2 {
            let idist = 1.0 / (t1 - t2);
            side = 0;
            frac2 = (t1 - offset - SURFACE_CLIP_EPSILON) * idist;
            frac = (t1 + offset + SURFACE_CLIP_EPSILON) * idist;
        } else {
            side = 0;
            frac = 1.0;
            frac2 = 0.0;
        }

        // move up to the node
        if frac < 0.0 {
            frac = 0.0;
        }
        if frac > 1.0 {
            frac = 1.0;
        }

        let midf = p1f + (p2f - p1f) * frac;

        let mut mid: vec3_t = [0.0; 3];
        mid[0] = p1[0] + frac * (p2[0] - p1[0]);
        mid[1] = p1[1] + frac * (p2[1] - p1[1]);
        mid[2] = p1[2] + frac * (p2[2] - p1[2]);

        CM_TraceThroughTree(
            common,
            cm,
            rm,
            rmg,
            host,
            tw,
            trace,
            local,
            (*node).children[side],
            p1f,
            midf,
            p1,
            mid,
        );

        // go past the node
        if frac2 < 0.0 {
            frac2 = 0.0;
        }
        if frac2 > 1.0 {
            frac2 = 1.0;
        }

        let midf = p1f + (p2f - p1f) * frac2;

        mid[0] = p1[0] + frac2 * (p2[0] - p1[0]);
        mid[1] = p1[1] + frac2 * (p2[1] - p1[1]);
        mid[2] = p1[2] + frac2 * (p2[2] - p1[2]);

        CM_TraceThroughTree(
            common,
            cm,
            rm,
            rmg,
            host,
            tw,
            trace,
            local,
            (*node).children[side ^ 1],
            midf,
            p2f,
            mid,
            p2,
        );
    }
}

/// Raven `CM_Trace`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:1577-1829`
pub fn CM_Trace(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    trace: *mut trace_t,
    start: vec3_t,
    end: vec3_t,
    mut mins: vec3_t,
    mut maxs: vec3_t,
    model: clipHandle_t,
    origin: vec3_t,
    brushmask: c_int,
    capsule: c_int,
    sphere: *mut sphere_t,
) {
    unsafe {
        let mut tw: traceWork_t = core::mem::zeroed();
        let mut offset: vec3_t = [0.0; 3];
        let mut local: *mut clipMap_t = core::ptr::null_mut();

        let cmod: *mut cmodel_t = CM_ClipHandleToModel(cm, model, &mut local);

        (*local).checkcount += 1; // for multi-check avoidance

        common.c_traces += 1; // for statistics, may be zeroed

        // fill in a default trace
        Com_Memset(
            &mut tw as *mut traceWork_t as *mut (),
            0,
            core::mem::size_of::<traceWork_t>(),
        );
        core::ptr::write_bytes(trace, 0, 1);
        (*trace).fraction = 1.0; // assume it goes the entire distance until shown otherwise
        VectorCopy(origin, &mut tw.modelOrigin);

        if (*local).numNodes == 0 {
            return; // map not loaded, shouldn't happen
        }

        // allow NULL to be passed in for 0,0,0 — PORT-NOTE(null-vec3): the resolved
        // signature takes `mins`/`maxs` by value (never a raw pointer), so the "was
        // NULL" branch is unreachable here; kept as a no-op mirror of Raven's guard.
        let _ = &mut mins;
        let _ = &mut maxs;

        // set basic parms
        tw.contents = brushmask;

        // adjust so that mins and maxs are always symetric, which
        // avoids some complications with plane expanding of rotated
        // bmodels
        for i in 0..3 {
            offset[i] = (mins[i] + maxs[i]) * 0.5;
            tw.size[0][i] = mins[i] - offset[i];
            tw.size[1][i] = maxs[i] - offset[i];
            tw.start[i] = start[i] + offset[i];
            tw.end[i] = end[i] + offset[i];
        }

        // if a sphere is already specified
        if !sphere.is_null() {
            tw.sphere = *sphere;
        } else {
            tw.sphere.r#use = capsule;
            tw.sphere.radius = if tw.size[1][0] > tw.size[1][2] {
                tw.size[1][2]
            } else {
                tw.size[1][0]
            };
            tw.sphere.halfheight = tw.size[1][2];
            VectorSet(
                &mut tw.sphere.offset,
                0.0,
                0.0,
                tw.size[1][2] - tw.sphere.radius,
            );
        }

        tw.maxOffset = tw.size[1][0] + tw.size[1][1] + tw.size[1][2];

        // tw.offsets[signbits] = vector to apropriate corner from origin
        tw.offsets[0][0] = tw.size[0][0];
        tw.offsets[0][1] = tw.size[0][1];
        tw.offsets[0][2] = tw.size[0][2];

        tw.offsets[1][0] = tw.size[1][0];
        tw.offsets[1][1] = tw.size[0][1];
        tw.offsets[1][2] = tw.size[0][2];

        tw.offsets[2][0] = tw.size[0][0];
        tw.offsets[2][1] = tw.size[1][1];
        tw.offsets[2][2] = tw.size[0][2];

        tw.offsets[3][0] = tw.size[1][0];
        tw.offsets[3][1] = tw.size[1][1];
        tw.offsets[3][2] = tw.size[0][2];

        tw.offsets[4][0] = tw.size[0][0];
        tw.offsets[4][1] = tw.size[0][1];
        tw.offsets[4][2] = tw.size[1][2];

        tw.offsets[5][0] = tw.size[1][0];
        tw.offsets[5][1] = tw.size[0][1];
        tw.offsets[5][2] = tw.size[1][2];

        tw.offsets[6][0] = tw.size[0][0];
        tw.offsets[6][1] = tw.size[1][1];
        tw.offsets[6][2] = tw.size[1][2];

        tw.offsets[7][0] = tw.size[1][0];
        tw.offsets[7][1] = tw.size[1][1];
        tw.offsets[7][2] = tw.size[1][2];

        //
        // calculate bounds
        //
        if tw.sphere.r#use != qfalse {
            for i in 0..3 {
                if tw.start[i] < tw.end[i] {
                    tw.bounds[0][i] = tw.start[i] - tw.sphere.offset[i].abs() - tw.sphere.radius;
                    tw.bounds[1][i] = tw.end[i] + tw.sphere.offset[i].abs() + tw.sphere.radius;
                } else {
                    tw.bounds[0][i] = tw.end[i] - tw.sphere.offset[i].abs() - tw.sphere.radius;
                    tw.bounds[1][i] = tw.start[i] + tw.sphere.offset[i].abs() + tw.sphere.radius;
                }
            }
        } else {
            for i in 0..3 {
                if tw.start[i] < tw.end[i] {
                    tw.bounds[0][i] = tw.start[i] + tw.size[0][i];
                    tw.bounds[1][i] = tw.end[i] + tw.size[1][i];
                } else {
                    tw.bounds[0][i] = tw.end[i] + tw.size[0][i];
                    tw.bounds[1][i] = tw.start[i] + tw.size[1][i];
                }
            }
        }

        //
        // check for position test special case
        //
        if start[0] == end[0]
            && start[1] == end[1]
            && start[2] == end[2]
            && tw.size[0][0] == 0.0
            && tw.size[0][1] == 0.0
            && tw.size[0][2] == 0.0
        {
            if model != 0 && (*cmod).firstNode == -1 {
                if model == CAPSULE_MODEL_HANDLE as c_int {
                    if tw.sphere.r#use != qfalse {
                        CM_TestCapsuleInCapsule(cm, &mut tw, &mut *trace, model);
                    } else {
                        CM_TestBoundingBoxInCapsule(cm, rmg, host, &mut tw, &mut *trace, model);
                    }
                } else {
                    CM_TestInLeaf(
                        cm,
                        rmg,
                        host,
                        &mut tw,
                        &mut *trace,
                        &mut (*cmod).leaf,
                        local,
                    );
                }
            } else if (*cmod).firstNode == -1 {
                CM_PositionTest(cm, rmg, host, &mut tw, &mut *trace);
            } else {
                CM_TraceThroughTree(
                    common,
                    cm,
                    rm,
                    rmg,
                    host,
                    &mut tw,
                    &mut *trace,
                    local,
                    (*cmod).firstNode,
                    0.0,
                    1.0,
                    tw.start,
                    tw.end,
                );
            }
        } else {
            //
            // check for point special case
            //
            if tw.size[0][0] == 0.0 && tw.size[0][1] == 0.0 && tw.size[0][2] == 0.0 {
                tw.isPoint = qtrue;
                VectorClear(&mut tw.extents);
            } else {
                tw.isPoint = qfalse;
                tw.extents[0] = tw.size[1][0];
                tw.extents[1] = tw.size[1][1];
                tw.extents[2] = tw.size[1][2];
            }

            //
            // general sweeping through world
            //
            if model != 0 && (*cmod).firstNode == -1 {
                if model == CAPSULE_MODEL_HANDLE as c_int {
                    if tw.sphere.r#use != qfalse {
                        CM_TraceCapsuleThroughCapsule(cm, &mut tw, &mut *trace, model);
                    } else {
                        CM_TraceBoundingBoxThroughCapsule(
                            common,
                            cm,
                            rm,
                            rmg,
                            host,
                            &mut tw,
                            &mut *trace,
                            model,
                        );
                    }
                } else {
                    CM_TraceThroughLeaf(
                        common,
                        cm,
                        rm,
                        rmg,
                        host,
                        &mut tw,
                        &mut *trace,
                        local,
                        &mut (*cmod).leaf,
                    );
                }
            } else {
                CM_TraceThroughTree(
                    common,
                    cm,
                    rm,
                    rmg,
                    host,
                    &mut tw,
                    &mut *trace,
                    local,
                    (*cmod).firstNode,
                    0.0,
                    1.0,
                    tw.start,
                    tw.end,
                );
            }
        }

        // generate endpos from the original, unmodified start/end
        if (*trace).fraction == 1.0 {
            VectorCopy(end, &mut (*trace).endpos);
        } else {
            for i in 0..3 {
                (*trace).endpos[i] = start[i] + (*trace).fraction * (end[i] - start[i]);
            }
        }

        // If allsolid is set (was entirely inside something solid), the plane is not valid.
        // If fraction == 1.0, we never hit anything, and thus the plane is not valid.
        // Otherwise, the normal on the plane should have unit length
        debug_assert!(
            (*trace).allsolid != 0
                || (*trace).fraction == 1.0
                || VectorLengthSquared((*trace).plane.normal) > 0.9999
        );
    }
}

/// Raven `CM_BoxTrace`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:1836-1840`
pub fn CM_BoxTrace(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    results: *mut trace_t,
    start: vec3_t,
    end: vec3_t,
    mins: vec3_t,
    maxs: vec3_t,
    model: clipHandle_t,
    brushmask: c_int,
    capsule: c_int,
) {
    CM_Trace(
        common,
        cm,
        rm,
        rmg,
        host,
        results,
        start,
        end,
        mins,
        maxs,
        model,
        VEC3_ORIGIN,
        brushmask,
        capsule,
        core::ptr::null_mut(),
    )
}

// PORT-NOTE(vec3_origin): `vec3_origin` (`q_shared.h:1179`) is a qshared global
// with no reachable home in this crate yet; referenced as a local `VEC3_ORIGIN`
// const standing in for it (reported as a missing symbol for the finisher to
// retarget once the real global lands on `Common`/`mp_qshared`).
const VEC3_ORIGIN: vec3_t = [0.0, 0.0, 0.0];

/// Raven `CM_TransformedBoxTrace`.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:1850-1937`
pub fn CM_TransformedBoxTrace(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    trace: *mut trace_t,
    start: vec3_t,
    end: vec3_t,
    mut mins: vec3_t,
    mut maxs: vec3_t,
    model: clipHandle_t,
    brushmask: c_int,
    origin: vec3_t,
    angles: vec3_t,
    capsule: c_int,
) {
    unsafe {
        let mut start_l: vec3_t = [0.0; 3];
        let mut end_l: vec3_t = [0.0; 3];
        let mut offset: vec3_t = [0.0; 3];
        let mut symetricSize: vec3pair_t = [[0.0; 3]; 2];
        let mut matrix: [vec3_t; 3] = [[0.0; 3]; 3];
        let mut transpose: [vec3_t; 3] = [[0.0; 3]; 3];
        let mut sphere: sphere_t = core::mem::zeroed();

        // PORT-NOTE(null-vec3): see CM_Trace — `mins`/`maxs` are by-value params here
        // too, so the "was NULL" substitution is unreachable; kept as a no-op mirror.
        let _ = &mut mins;
        let _ = &mut maxs;

        // adjust so that mins and maxs are always symetric, which
        // avoids some complications with plane expanding of rotated
        // bmodels
        for i in 0..3 {
            offset[i] = (mins[i] + maxs[i]) * 0.5;
            symetricSize[0][i] = mins[i] - offset[i];
            symetricSize[1][i] = maxs[i] - offset[i];
            start_l[i] = start[i] + offset[i];
            end_l[i] = end[i] + offset[i];
        }

        // subtract origin offset
        VectorSubtract(start_l, origin, &mut start_l);
        VectorSubtract(end_l, origin, &mut end_l);

        // rotate start and end into the models frame of reference
        let rotated = model != BOX_MODEL_HANDLE as c_int
            && (angles[0] != 0.0 || angles[1] != 0.0 || angles[2] != 0.0);

        let halfwidth = symetricSize[1][0];
        let halfheight = symetricSize[1][2];

        sphere.r#use = capsule;
        sphere.radius = if halfwidth > halfheight {
            halfheight
        } else {
            halfwidth
        };
        sphere.halfheight = halfheight;
        let t = halfheight - sphere.radius;

        if rotated {
            // rotation on trace line (start-end) instead of rotating the bmodel
            // NOTE: This is still incorrect for bounding boxes because the actual bounding
            //		 box that is swept through the model is not rotated. We cannot rotate
            //		 the bounding box or the bmodel because that would make all the brush
            //		 bevels invalid.
            //		 However this is correct for capsules since a capsule itself is rotated too.
            CreateRotationMatrix(angles, matrix.as_mut_ptr());
            RotatePoint(start_l, matrix.as_mut_ptr());
            RotatePoint(end_l, matrix.as_mut_ptr());
            // rotated sphere offset for capsule
            sphere.offset[0] = matrix[0][2] * t;
            sphere.offset[1] = -matrix[1][2] * t;
            sphere.offset[2] = matrix[2][2] * t;
        } else {
            VectorSet(&mut sphere.offset, 0.0, 0.0, t);
        }

        // sweep the box through the model
        CM_Trace(
            common,
            cm,
            rm,
            rmg,
            host,
            trace,
            start_l,
            end_l,
            symetricSize[0],
            symetricSize[1],
            model,
            origin,
            brushmask,
            capsule,
            &mut sphere,
        );

        // if the bmodel was rotated and there was a collision
        if rotated && (*trace).fraction != 1.0 {
            // rotation of bmodel collision plane
            TransposeMatrix(matrix.as_mut_ptr(), transpose.as_mut_ptr());
            RotatePoint((*trace).plane.normal, transpose.as_mut_ptr());
        }

        // re-calculate the end position of the trace because the trace.endpos
        // calculated by CM_Trace could be rotated and have an offset
        (*trace).endpos[0] = start[0] + (*trace).fraction * (end[0] - start[0]);
        (*trace).endpos[1] = start[1] + (*trace).fraction * (end[1] - start[1]);
        (*trace).endpos[2] = start[2] + (*trace).fraction * (end[2] - start[2]);
    }
}

// PORT-NOTE(missing-symbols): the following are referenced by their exact Raven
// names/shapes per the no-stub rule but are NOT declared/imported in this file —
// none has a reachable home in `mp_engine_qcommon`'s current dependency graph
// (see the file-level PORT-NOTE and per-callsite notes above). Reported in
// `missing_symbols`, not stubbed:
//   math primitives: DotProduct, VectorAdd, VectorSubtract, VectorCopy,
//     VectorScale, VectorMA, VectorSet, VectorClear, VectorAdvance, VectorLength,
//     VectorLengthSquared, VectorNormalize, VectorInverse, AngleVectors, Square
//   in-engine callees not yet landed in this wave: CM_ModelBounds,
//     CM_ClipHandleToModel, CM_TempBoxModel, CM_BoxLeafnums_r, CM_StoreLeafs,
//     CM_PositionTestInPatchCollide, CM_TraceThroughPatchCollide, Com_Memset
//   receiver/§F types: RmManager, RenderModels, CCMLandScape, CCMPatch
