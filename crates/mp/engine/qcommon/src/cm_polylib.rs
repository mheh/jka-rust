#![allow(non_snake_case, non_camel_case_types, clippy::too_many_arguments)]
//! `cm_polylib.cpp` — winding (convex polygon) primitives used by the BSP
//! clipper: allocation, plane/area/bounds/center queries, clipping and the
//! convex-hull accumulator.
//!
//! Source: `oracle/codemp/qcommon/cm_polylib.cpp`
//!
//! PORT-NOTE(vector-math): Raven's `VectorSubtract`/`VectorAdd`/`VectorCopy`/
//! `VectorScale`/`VectorMA`/`CrossProduct`/`DotProduct`/`VectorNormalize2`/
//! `VectorLength` (q_math primitives) have no reachable home in this crate's
//! dependency graph yet (cm_trace.rs precedent — their only Rust port lives in
//! `mp_game`, a tier above the engine). Called here by their exact Raven names
//! per the no-stub rule; reported as missing symbols for the finisher to wire
//! to a q_math home reachable from `mp_engine_qcommon` (e.g. relocated into
//! `mp_qshared` per NAV-D6 precedent).
//!
//! PORT-NOTE(rm-types): `RenderModels`/`RmManager` are state-receiver types
//! pinned by the engine-fork-discovery preamble's receiver order
//! (rmg-terrain.md/tr-model.md own their real shape); neither has landed in
//! this crate yet. Referenced by their exact resolved-signature names per the
//! no-stub rule (common_fns.rs/vm_x86.rs precedent); reported as missing
//! symbols for the finisher to replace with the real imports once they land.

use core::ffi::{c_char, c_int};

use mp_qshared::common::mp::qcommon::tags::memtag_t;
use mp_qshared::shared::q_math::{
    vec3_origin, CrossProduct, VectorLength, VectorNormalize2, _DotProduct as DotProduct,
    _VectorAdd as VectorAdd, _VectorCopy as VectorCopy, _VectorMA as VectorMA,
    _VectorScale as VectorScale, _VectorSubtract as VectorSubtract,
};
use mp_qshared::shared::{errorParm_t, vec3_t, vec_t};

use crate::cm::cm_polylib_consts::{
    MAX_HULL_POINTS, MAX_MAP_BOUNDS, MAX_POINTS_ON_WINDING, SIDE_BACK, SIDE_CROSS, SIDE_FRONT,
    SIDE_ON,
};
use crate::cm::winding_t::winding_t;
use crate::collision_world::CollisionWorld;
use crate::common::com_error;
use crate::common::Common;
use mp_host_interface::engine_host::EngineHost;

// PORT-NOTE(rm-types): see module doc.
#[allow(dead_code)]
use crate::cm_load::RenderModels;
#[allow(dead_code)]
struct RmManager;

// PORT-NOTE(on-epsilon): the packet's rosetta row points `ON_EPSILON` at
// `mp_engine_botlib`'s `be_aas_bspq3_cpp_consts` (0.005) — but `mp_engine_botlib`
// depends on `mp_engine_qcommon` (Cargo.toml), so importing it here would be a
// crate cycle. `cm_polylib.h`'s own `#ifndef ON_EPSILON` definition (0.1) is
// already ported in this crate at `cm::cm_polylib_consts::ON_EPSILON` and is
// the value that actually governs this TU; used instead, escalated as a shape
// mismatch for the finisher/rosetta to reconcile.
use crate::cm::cm_polylib_consts::ON_EPSILON;

// Sweep: extern forward-declare eliminated — libc provides `printf` (rule 3,
// this crate already links libc for fopen-family I/O).
use libc::printf;

// First-compile wiring (sweep): real in-crate `Com_Memcpy`; `Z_Malloc`/`Z_Free`
// referenced at their canonical `z_memman_pc` home (genuinely unported; reported).
use crate::common_fns::Com_Memcpy;
use crate::z_memman_pc::{Z_Free, Z_Malloc};

/// Raven `pw` — debug-print a winding's points.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.cpp:17-22`
pub fn pw(w: *mut winding_t) {
    unsafe {
        for i in 0..(*w).numpoints {
            printf(
                c"(%5.1f, %5.1f, %5.1f)\n".as_ptr(),
                (*w).p[i as usize][0] as f64,
                (*w).p[i as usize][1] as f64,
                (*w).p[i as usize][2] as f64,
            );
        }
    }
}

/// Raven `WindingPlane` — derive the winding's plane normal/dist from its
/// first three points.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.cpp:100-110`
///
/// PORT-NOTE(signature-shape): the resolved signature takes `normal: vec3_t`
/// BY VALUE (array), but Raven's C writes through the decayed pointer — a
/// real out-param the printed signature doesn't carry (cm_trace.rs
/// `RotatePoint` precedent). Transcribed literally against the LAW signature
/// (mutates the local copy only); flagged as a shape mismatch for the
/// finisher (retype to `&mut vec3_t`).
pub fn WindingPlane(w: *mut winding_t, mut normal: vec3_t, dist: *mut vec_t) {
    unsafe {
        let mut v1: vec3_t = [0.0; 3];
        let mut v2: vec3_t = [0.0; 3];

        VectorSubtract((*w).p[1], (*w).p[0], &mut v1);
        VectorSubtract((*w).p[2], (*w).p[0], &mut v2);
        CrossProduct(v2, v1, &mut normal);
        VectorNormalize2(normal, &mut normal);
        *dist = DotProduct((*w).p[0], normal);
    }
}

/// Raven `WindingArea` — sum of the fan-triangulated area.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.cpp:117-132`
pub fn WindingArea(w: *mut winding_t) -> vec_t {
    unsafe {
        let mut total: vec_t = 0.0;
        for i in 2..(*w).numpoints {
            let mut d1: vec3_t = [0.0; 3];
            let mut d2: vec3_t = [0.0; 3];
            let mut cross: vec3_t = [0.0; 3];
            VectorSubtract((*w).p[(i - 1) as usize], (*w).p[0], &mut d1);
            VectorSubtract((*w).p[i as usize], (*w).p[0], &mut d2);
            CrossProduct(d1, d2, &mut cross);
            total += 0.5 * VectorLength(cross);
        }
        total
    }
}

/// Raven `WindingBounds`.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.cpp:134-153`
///
/// PORT-NOTE(signature-shape): `mins`/`maxs` are BY-VALUE `vec3_t` out-params
/// per the same shape mismatch as `WindingPlane`; flagged for the finisher.
pub fn WindingBounds(w: *mut winding_t, mut mins: vec3_t, mut maxs: vec3_t) {
    mins[0] = MAX_MAP_BOUNDS as vec_t;
    mins[1] = MAX_MAP_BOUNDS as vec_t;
    mins[2] = MAX_MAP_BOUNDS as vec_t;
    maxs[0] = -(MAX_MAP_BOUNDS as vec_t);
    maxs[1] = -(MAX_MAP_BOUNDS as vec_t);
    maxs[2] = -(MAX_MAP_BOUNDS as vec_t);

    unsafe {
        for i in 0..(*w).numpoints {
            for j in 0..3usize {
                let v = (*w).p[i as usize][j];
                if v < mins[j] {
                    mins[j] = v;
                }
                if v > maxs[j] {
                    maxs[j] = v;
                }
            }
        }
    }
}

/// Raven `WindingCenter`.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.cpp:160-171`
///
/// PORT-NOTE(signature-shape): `center` is a BY-VALUE `vec3_t` out-param per
/// the same shape mismatch as `WindingPlane`; flagged for the finisher.
pub fn WindingCenter(common: &mut Common, w: *mut winding_t, mut center: vec3_t) {
    VectorCopy(vec3_origin, &mut center);
    unsafe {
        for i in 0..(*w).numpoints {
            let p = (*w).p[i as usize];
            VectorAdd(p, center, &mut center);
        }
        let scale: vec_t = 1.0 / (*w).numpoints as vec_t;
        VectorScale(center, scale, &mut center);
    }
}

/// Raven `WindingOnPlaneSide`.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.cpp:583-615`
pub fn WindingOnPlaneSide(w: *mut winding_t, normal: vec3_t, dist: vec_t) -> c_int {
    let mut front = false;
    let mut back = false;

    unsafe {
        for i in 0..(*w).numpoints {
            let d = DotProduct((*w).p[i as usize], normal) - dist;
            if d < -ON_EPSILON {
                if front {
                    return SIDE_CROSS;
                }
                back = true;
                continue;
            }
            if d > ON_EPSILON {
                if back {
                    return SIDE_CROSS;
                }
                front = true;
                continue;
            }
        }
    }

    if back {
        return SIDE_BACK;
    }
    if front {
        return SIDE_FRONT;
    }
    SIDE_ON
}

/// Raven `RemoveColinearPoints`.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.cpp:64-93`
pub fn RemoveColinearPoints(cm: &mut CollisionWorld, w: *mut winding_t) {
    // PORT-NOTE(missing-field): `c_removed` (`cm_polylib.cpp:62`) is a
    // file-scope global with no home field on `CollisionWorld` yet;
    // referenced as `cm.c_removed`, reported as a missing symbol.
    let mut nump: c_int = 0;
    // §19: `p` is a local scratch array Raven writes before reading (never
    // read uninitialized) — zero-init is not load-bearing but keeps the
    // array well-defined.
    let mut p: [vec3_t; MAX_POINTS_ON_WINDING as usize] =
        [[0.0; 3]; MAX_POINTS_ON_WINDING as usize];

    unsafe {
        let numpoints = (*w).numpoints;
        for i in 0..numpoints {
            let j = (i + 1) % numpoints;
            let k = (i + numpoints - 1) % numpoints;
            let mut v1: vec3_t = [0.0; 3];
            let mut v2: vec3_t = [0.0; 3];
            VectorSubtract((*w).p[j as usize], (*w).p[i as usize], &mut v1);
            VectorSubtract((*w).p[i as usize], (*w).p[k as usize], &mut v2);
            VectorNormalize2(v1, &mut v1);
            VectorNormalize2(v2, &mut v2);
            if DotProduct(v1, v2) < 0.999 {
                VectorCopy((*w).p[i as usize], &mut p[nump as usize]);
                nump += 1;
            }
        }

        if nump == numpoints {
            return;
        }

        cm.c_removed += numpoints - nump;
        (*w).numpoints = nump;
        Com_Memcpy(
            (*w).p.as_mut_ptr() as *mut (),
            p.as_ptr() as *const (),
            nump as usize * core::mem::size_of::<vec3_t>(),
        );
    }
}

/// Raven `AllocWinding`.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.cpp:30-45`
pub fn AllocWinding(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    points: c_int,
) -> *mut winding_t {
    // PORT-NOTE(missing-field): `c_active_windings`/`c_peak_windings`/
    // `c_winding_allocs`/`c_winding_points` (`cm_polylib.cpp:12-15`) have no
    // home fields yet; referenced as `common.c_active_windings` etc. per the
    // owning-file convention, reported as missing symbols.
    common.c_winding_allocs += 1;
    common.c_winding_points += points;
    common.c_active_windings += 1;
    if common.c_active_windings > common.c_peak_windings {
        common.c_peak_windings = common.c_active_windings;
    }

    let s = core::mem::size_of::<vec_t>() as c_int * 3 * points
        + core::mem::size_of::<c_int>() as c_int;
    // PORT-NOTE(missing-callee): `Z_Malloc` is part of the still-unlanded
    // z_memman unit; called with the exact resolved receivers and reported
    // as a missing symbol (cm_trace.rs/vm_x86.rs precedent).
    let w = Z_Malloc(common, cm, rm, host, s, memtag_t::TAG_BSP, true) as *mut winding_t;
    // Raven: qtrue param in Z_Malloc does this (Com_Memset commented out).
    w
}

/// Raven `FreeWinding`.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.cpp:47-55`
pub fn FreeWinding(common: &mut Common, cm: &mut CollisionWorld, w: *mut winding_t) {
    unsafe {
        if *(w as *mut u32) == 0xdeaddead {
            com_error(
                errorParm_t::ERR_FATAL,
                "FreeWinding: freed a freed winding".to_string(),
            );
        }
        *(w as *mut u32) = 0xdeaddead;
    }

    cm.c_active_windings -= 1;
    // PORT-NOTE(missing-callee): `Z_Free` is part of the still-unlanded
    // z_memman unit; called with the exact resolved receivers and reported
    // as a missing symbol.
    Z_Free(common, w as *mut ());
}

/// Raven `CheckWinding` — sanity-checks a winding's planarity/convexity.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.cpp:520-575`
pub fn CheckWinding(w: *mut winding_t) {
    unsafe {
        if (*w).numpoints < 3 {
            com_error(
                errorParm_t::ERR_DROP,
                format!("CheckWinding: {} points", (*w).numpoints),
            );
        }

        let area = WindingArea(w);
        if area < 1.0 {
            com_error(
                errorParm_t::ERR_DROP,
                format!("CheckWinding: {} area", area),
            );
        }

        let mut facenormal: vec3_t = [0.0; 3];
        let mut facedist: vec_t = 0.0;
        WindingPlane(w, facenormal, &mut facedist);

        for i in 0..(*w).numpoints {
            let p1 = (*w).p[i as usize];

            for j in 0..3usize {
                if p1[j] > MAX_MAP_BOUNDS as vec_t || p1[j] < -(MAX_MAP_BOUNDS as vec_t) {
                    // Raven's literal string reads "BUGUS_RANGE" (a typo for
                    // "bogus range"), transcribed verbatim.
                    com_error(
                        errorParm_t::ERR_DROP,
                        format!("CheckFace: BUGUS_RANGE: {}", p1[j]),
                    );
                }
            }

            let j = if i + 1 == (*w).numpoints { 0 } else { i + 1 };

            // check the point is on the face plane
            let d = DotProduct(p1, facenormal) - facedist;
            if d < -ON_EPSILON || d > ON_EPSILON {
                com_error(
                    errorParm_t::ERR_DROP,
                    "CheckWinding: point off plane".to_string(),
                );
            }

            // check the edge isnt degenerate
            let p2 = (*w).p[j as usize];
            let mut dir: vec3_t = [0.0; 3];
            VectorSubtract(p2, p1, &mut dir);

            if VectorLength(dir) < ON_EPSILON {
                com_error(
                    errorParm_t::ERR_DROP,
                    "CheckWinding: degenerate edge".to_string(),
                );
            }

            let mut edgenormal: vec3_t = [0.0; 3];
            CrossProduct(facenormal, dir, &mut edgenormal);
            VectorNormalize2(edgenormal, &mut edgenormal);
            let mut edgedist = DotProduct(p1, edgenormal);
            edgedist += ON_EPSILON;

            // all other points must be on front side
            for jj in 0..(*w).numpoints {
                if jj == i {
                    continue;
                }
                let d = DotProduct((*w).p[jj as usize], edgenormal);
                if d > edgedist {
                    com_error(
                        errorParm_t::ERR_DROP,
                        "CheckWinding: non-convex".to_string(),
                    );
                }
            }
        }
    }
}

/// Raven `BaseWindingForPlane` — builds a huge axis-aligned quad on `normal`/
/// `dist` (the seed winding for BSP-plane clipping).
///
/// Source: `oracle/codemp/qcommon/cm_polylib.cpp:178-242`
pub fn BaseWindingForPlane(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    normal: vec3_t,
    dist: vec_t,
) -> *mut winding_t {
    // find the major axis
    let mut max: vec_t = -(MAX_MAP_BOUNDS as vec_t);
    let mut x: c_int = -1;
    for i in 0..3usize {
        let v = normal[i].abs();
        if v > max {
            x = i as c_int;
            max = v;
        }
    }
    if x == -1 {
        com_error(
            errorParm_t::ERR_DROP,
            "BaseWindingForPlane: no axis found".to_string(),
        );
    }

    // PORT-NOTE(missing-field): `vec3_origin` — see `WindingCenter`'s note.
    let mut vup: vec3_t = [0.0; 3];
    VectorCopy(vec3_origin, &mut vup);
    match x {
        0 | 1 => vup[2] = 1.0,
        2 => vup[0] = 1.0,
        _ => {}
    }

    let v = DotProduct(vup, normal);
    VectorMA(vup, -v, normal, &mut vup);
    VectorNormalize2(vup, &mut vup);

    let mut org: vec3_t = [0.0; 3];
    VectorScale(normal, dist, &mut org);

    let mut vright: vec3_t = [0.0; 3];
    CrossProduct(vup, normal, &mut vright);

    VectorScale(vup, MAX_MAP_BOUNDS as vec_t, &mut vup);
    VectorScale(vright, MAX_MAP_BOUNDS as vec_t, &mut vright);

    // project a really big axis aligned box onto the plane
    let w = AllocWinding(common, cm, rm, host, 4);

    unsafe {
        VectorSubtract(org, vright, &mut (*w).p[0]);
        let p0 = (*w).p[0];
        VectorAdd(p0, vup, &mut (*w).p[0]);

        VectorAdd(org, vright, &mut (*w).p[1]);
        let p1 = (*w).p[1];
        VectorAdd(p1, vup, &mut (*w).p[1]);

        VectorAdd(org, vright, &mut (*w).p[2]);
        let p2 = (*w).p[2];
        VectorSubtract(p2, vup, &mut (*w).p[2]);

        VectorSubtract(org, vright, &mut (*w).p[3]);
        let p3 = (*w).p[3];
        VectorSubtract(p3, vup, &mut (*w).p[3]);

        (*w).numpoints = 4;
    }

    w
}

/// Raven `CopyWinding`.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.cpp:249-258`
pub fn CopyWinding(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    w: *mut winding_t,
) -> *mut winding_t {
    unsafe {
        let c = AllocWinding(common, cm, rm, host, (*w).numpoints);
        let size = core::mem::offset_of!(winding_t, p)
            + (*w).numpoints as usize * core::mem::size_of::<vec3_t>();
        Com_Memcpy(c as *mut (), w as *const (), size);
        c
    }
}

/// Raven `ReverseWinding`.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.cpp:265-277`
pub fn ReverseWinding(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    w: *mut winding_t,
) -> *mut winding_t {
    unsafe {
        let c = AllocWinding(common, cm, rm, host, (*w).numpoints);
        for i in 0..(*w).numpoints {
            let src = (*w).p[((*w).numpoints - 1 - i) as usize];
            VectorCopy(src, &mut (*c).p[i as usize]);
        }
        (*c).numpoints = (*w).numpoints;
        c
    }
}

/// Raven `ChopWindingInPlace`.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.cpp:397-491`
pub fn ChopWindingInPlace(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    inout: *mut *mut winding_t,
    normal: vec3_t,
    dist: vec_t,
    epsilon: vec_t,
) {
    // §19: `dists`/`sides` are read only after this loop writes every index
    // (plus the wrap-around index `numpoints`) it reads; zero-init keeps the
    // arrays well-defined without changing behavior.
    let mut dists: [vec_t; (MAX_POINTS_ON_WINDING + 4) as usize] =
        [0.0; (MAX_POINTS_ON_WINDING + 4) as usize];
    let mut sides: [c_int; (MAX_POINTS_ON_WINDING + 4) as usize] =
        [0; (MAX_POINTS_ON_WINDING + 4) as usize];
    let mut counts: [c_int; 3] = [0; 3];
    // `static vec_t dot` (VC 4.2 optimizer-bug workaround, not genuine
    // cross-frame state) — a plain local reproduces the same math.
    let mut dot: vec_t;

    unsafe {
        let in_ = *inout;
        counts[0] = 0;
        counts[1] = 0;
        counts[2] = 0;

        // determine sides for each point
        for i in 0..(*in_).numpoints {
            dot = DotProduct((*in_).p[i as usize], normal);
            dot -= dist;
            dists[i as usize] = dot;
            if dot > epsilon {
                sides[i as usize] = SIDE_FRONT;
            } else if dot < -epsilon {
                sides[i as usize] = SIDE_BACK;
            } else {
                sides[i as usize] = SIDE_ON;
            }
            counts[sides[i as usize] as usize] += 1;
        }
        sides[(*in_).numpoints as usize] = sides[0];
        dists[(*in_).numpoints as usize] = dists[0];

        if counts[0] == 0 {
            FreeWinding(common, cm, in_);
            *inout = core::ptr::null_mut();
            return;
        }
        if counts[1] == 0 {
            return; // inout stays the same
        }

        let maxpts = (*in_).numpoints + 4; // cant use counts[0]+2 because
                                           // of fp grouping errors

        let f = AllocWinding(common, cm, rm, host, maxpts);

        for i in 0..(*in_).numpoints {
            let p1 = (*in_).p[i as usize];

            if sides[i as usize] == SIDE_ON {
                let idx = (*f).numpoints as usize;
                VectorCopy(p1, &mut (*f).p[idx]);
                (*f).numpoints += 1;
                continue;
            }

            if sides[i as usize] == SIDE_FRONT {
                let idx = (*f).numpoints as usize;
                VectorCopy(p1, &mut (*f).p[idx]);
                (*f).numpoints += 1;
            }

            if sides[(i + 1) as usize] == SIDE_ON || sides[(i + 1) as usize] == sides[i as usize] {
                continue;
            }

            // generate a split point
            let p2 = (*in_).p[((i + 1) % (*in_).numpoints) as usize];

            dot = dists[i as usize] / (dists[i as usize] - dists[(i + 1) as usize]);
            let mut mid: vec3_t = [0.0; 3];
            for j in 0..3usize {
                // avoid round off error when possible
                if normal[j] == 1.0 {
                    mid[j] = dist;
                } else if normal[j] == -1.0 {
                    mid[j] = -dist;
                } else {
                    mid[j] = p1[j] + dot * (p2[j] - p1[j]);
                }
            }

            let idx = (*f).numpoints as usize;
            VectorCopy(mid, &mut (*f).p[idx]);
            (*f).numpoints += 1;
        }

        if (*f).numpoints > maxpts {
            com_error(
                errorParm_t::ERR_DROP,
                "ClipWinding: points exceeded estimate".to_string(),
            );
        }
        if (*f).numpoints > MAX_POINTS_ON_WINDING {
            com_error(
                errorParm_t::ERR_DROP,
                "ClipWinding: MAX_POINTS_ON_WINDING".to_string(),
            );
        }

        FreeWinding(common, cm, in_);
        *inout = f;
    }
}

/// Raven `ClipWindingEpsilon`.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.cpp:285-389`
pub fn ClipWindingEpsilon(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    in_: *mut winding_t,
    normal: vec3_t,
    dist: vec_t,
    epsilon: vec_t,
    front: *mut *mut winding_t,
    back: *mut *mut winding_t,
) {
    let mut dists: [vec_t; (MAX_POINTS_ON_WINDING + 4) as usize] =
        [0.0; (MAX_POINTS_ON_WINDING + 4) as usize];
    let mut sides: [c_int; (MAX_POINTS_ON_WINDING + 4) as usize] =
        [0; (MAX_POINTS_ON_WINDING + 4) as usize];
    let mut counts: [c_int; 3] = [0; 3];
    // `static vec_t dot` (VC 4.2 optimizer-bug workaround) — plain local.
    let mut dot: vec_t;

    unsafe {
        counts[0] = 0;
        counts[1] = 0;
        counts[2] = 0;

        // determine sides for each point
        for i in 0..(*in_).numpoints {
            dot = DotProduct((*in_).p[i as usize], normal);
            dot -= dist;
            dists[i as usize] = dot;
            if dot > epsilon {
                sides[i as usize] = SIDE_FRONT;
            } else if dot < -epsilon {
                sides[i as usize] = SIDE_BACK;
            } else {
                sides[i as usize] = SIDE_ON;
            }
            counts[sides[i as usize] as usize] += 1;
        }
        sides[(*in_).numpoints as usize] = sides[0];
        dists[(*in_).numpoints as usize] = dists[0];

        *front = core::ptr::null_mut();
        *back = core::ptr::null_mut();

        if counts[0] == 0 {
            *back = CopyWinding(common, cm, rm, host, in_);
            return;
        }
        if counts[1] == 0 {
            *front = CopyWinding(common, cm, rm, host, in_);
            return;
        }

        let maxpts = (*in_).numpoints + 4; // cant use counts[0]+2 because
                                           // of fp grouping errors

        let f = AllocWinding(common, cm, rm, host, maxpts);
        let b = AllocWinding(common, cm, rm, host, maxpts);
        *front = f;
        *back = b;

        for i in 0..(*in_).numpoints {
            let p1 = (*in_).p[i as usize];

            if sides[i as usize] == SIDE_ON {
                let fi = (*f).numpoints as usize;
                VectorCopy(p1, &mut (*f).p[fi]);
                (*f).numpoints += 1;
                let bi = (*b).numpoints as usize;
                VectorCopy(p1, &mut (*b).p[bi]);
                (*b).numpoints += 1;
                continue;
            }

            if sides[i as usize] == SIDE_FRONT {
                let fi = (*f).numpoints as usize;
                VectorCopy(p1, &mut (*f).p[fi]);
                (*f).numpoints += 1;
            }
            if sides[i as usize] == SIDE_BACK {
                let bi = (*b).numpoints as usize;
                VectorCopy(p1, &mut (*b).p[bi]);
                (*b).numpoints += 1;
            }

            if sides[(i + 1) as usize] == SIDE_ON || sides[(i + 1) as usize] == sides[i as usize] {
                continue;
            }

            // generate a split point
            let p2 = (*in_).p[((i + 1) % (*in_).numpoints) as usize];

            dot = dists[i as usize] / (dists[i as usize] - dists[(i + 1) as usize]);
            let mut mid: vec3_t = [0.0; 3];
            for j in 0..3usize {
                // avoid round off error when possible
                if normal[j] == 1.0 {
                    mid[j] = dist;
                } else if normal[j] == -1.0 {
                    mid[j] = -dist;
                } else {
                    mid[j] = p1[j] + dot * (p2[j] - p1[j]);
                }
            }

            let fi = (*f).numpoints as usize;
            VectorCopy(mid, &mut (*f).p[fi]);
            (*f).numpoints += 1;
            let bi = (*b).numpoints as usize;
            VectorCopy(mid, &mut (*b).p[bi]);
            (*b).numpoints += 1;
        }

        if (*f).numpoints > maxpts || (*b).numpoints > maxpts {
            com_error(
                errorParm_t::ERR_DROP,
                "ClipWinding: points exceeded estimate".to_string(),
            );
        }
        if (*f).numpoints > MAX_POINTS_ON_WINDING || (*b).numpoints > MAX_POINTS_ON_WINDING {
            com_error(
                errorParm_t::ERR_DROP,
                "ClipWinding: MAX_POINTS_ON_WINDING".to_string(),
            );
        }
    }
}

/// Raven `AddWindingToConvexHull`.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.cpp:626-711`
pub fn AddWindingToConvexHull(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    mut w: *mut winding_t,
    hull: *mut *mut winding_t,
    normal: vec3_t,
) {
    unsafe {
        if (*hull).is_null() {
            *hull = CopyWinding(common, cm, rm, host, w);
            return;
        }

        let mut num_hull_points = (**hull).numpoints as usize;
        let mut hull_points: [vec3_t; MAX_HULL_POINTS as usize] =
            [[0.0; 3]; MAX_HULL_POINTS as usize];
        Com_Memcpy(
            hull_points.as_mut_ptr() as *mut (),
            (**hull).p.as_ptr() as *const (),
            num_hull_points * core::mem::size_of::<vec3_t>(),
        );

        for i in 0..(*w).numpoints as usize {
            let p = (*w).p[i];

            // calculate hull side vectors
            let mut hull_dirs: [vec3_t; MAX_HULL_POINTS as usize] =
                [[0.0; 3]; MAX_HULL_POINTS as usize];
            for j in 0..num_hull_points {
                let k = (j + 1) % num_hull_points;
                let mut dir: vec3_t = [0.0; 3];
                VectorSubtract(hull_points[k], hull_points[j], &mut dir);
                VectorNormalize2(dir, &mut dir);
                CrossProduct(normal, dir, &mut hull_dirs[j]);
            }

            let mut outside = false;
            let mut hull_side: [bool; MAX_HULL_POINTS as usize] = [false; MAX_HULL_POINTS as usize];
            for j in 0..num_hull_points {
                let mut dir: vec3_t = [0.0; 3];
                VectorSubtract(p, hull_points[j], &mut dir);
                let d = DotProduct(dir, hull_dirs[j]);
                if d >= ON_EPSILON {
                    outside = true;
                }
                hull_side[j] = d >= -ON_EPSILON;
            }

            // if the point is effectively inside, do nothing
            if !outside {
                continue;
            }

            // find the back side to front side transition
            let mut j = 0usize;
            while j < num_hull_points {
                if !hull_side[j % num_hull_points] && hull_side[(j + 1) % num_hull_points] {
                    break;
                }
                j += 1;
            }
            if j == num_hull_points {
                continue;
            }

            // insert the point here
            let mut new_hull_points: [vec3_t; MAX_HULL_POINTS as usize] =
                [[0.0; 3]; MAX_HULL_POINTS as usize];
            VectorCopy(p, &mut new_hull_points[0]);
            let mut num_new = 1usize;

            // copy over all points that aren't double fronts
            j = (j + 1) % num_hull_points;
            for k in 0..num_hull_points {
                if hull_side[(j + k) % num_hull_points] && hull_side[(j + k + 1) % num_hull_points]
                {
                    continue;
                }
                let copy = hull_points[(j + k + 1) % num_hull_points];
                VectorCopy(copy, &mut new_hull_points[num_new]);
                num_new += 1;
            }

            num_hull_points = num_new;
            Com_Memcpy(
                hull_points.as_mut_ptr() as *mut (),
                new_hull_points.as_ptr() as *const (),
                num_hull_points * core::mem::size_of::<vec3_t>(),
            );
        }

        FreeWinding(common, cm, *hull);
        w = AllocWinding(common, cm, rm, host, num_hull_points as c_int);
        (*w).numpoints = num_hull_points as c_int;
        *hull = w;
        Com_Memcpy(
            (*w).p.as_mut_ptr() as *mut (),
            hull_points.as_ptr() as *const (),
            num_hull_points * core::mem::size_of::<vec3_t>(),
        );
    }
}

/// Raven `ChopWinding`.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.cpp:502-511`
pub fn ChopWinding(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    in_: *mut winding_t,
    normal: vec3_t,
    dist: vec_t,
) -> *mut winding_t {
    let mut f: *mut winding_t = core::ptr::null_mut();
    let mut b: *mut winding_t = core::ptr::null_mut();

    ClipWindingEpsilon(
        common, cm, rm, host, in_, normal, dist, ON_EPSILON, &mut f, &mut b,
    );
    FreeWinding(common, cm, in_);
    if !b.is_null() {
        FreeWinding(common, cm, b);
    }
    f
}
