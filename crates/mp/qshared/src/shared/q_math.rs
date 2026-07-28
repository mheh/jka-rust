//! MP `q_math.c` vec3 primitives shared across the MP tree.
//!
//! NAV-D3 / RULING 39d migration (sibling of `q_math_rand.rs`, mirrors
//! `oracle/codemp/game/q_math.c`): the vec3 helpers the engine-side nav code
//! consumes moved here from `mp_game` (`crates/mp/game/src/q_math.rs`),
//! **keeping Raven's `_`-prefixed names** (the tree's `DotProduct` /
//! `VectorSubtract` / `VectorCopy` `#define` macros expand to these). This is
//! the single engine-reachable definition the referee compares.
//!
//! Source: `oracle/codemp/game/q_math.c`

use core::ffi::c_int;

use crate::shared::collision::{cplane_t, PLANE_X, PLANE_Y, PLANE_Z};
use crate::shared::{vec3_t, vec_t};

// 2026-07-17 centralization ruling: the SP/MP-shared bodies moved to
// `native_math::qmath` (membership per tools/qmath-census.py); this module
// re-exports them under their established paths and keeps only the
// ABI-tier (`cplane_t`) and engine-lineage functions defined locally.
pub use native_math::qmath::PerpendicularVectorMP as PerpendicularVector;
pub use native_math::qmath::{
    _DotProduct, _VectorAdd, _VectorCopy, _VectorMA, _VectorScale, _VectorSubtract, vec3_origin,
    vectoangles, vectoyaw, AngleDelta, AngleDifference, AngleMod, AngleNormZero, AngleNormalize180,
    AngleNormalize360, AngleSubtract, AngleVectors, AnglesSubtract, AnglesToAxis, AxisClear,
    AxisCopy, ByteToDir, Create_Matrix, CrossProduct, DirToByte, Distance, DistanceSquared,
    DotProductRow, Inverse_Matrix, LerpAngle, MatrixMultiply, ProjectPointOnPlane, Q_fabs, Q_rand,
    Q_random, RadiusFromBounds, TransformAndTranslatePoint, TransformPoint, VectorBetweenVectors,
    VectorClear, VectorCompare, VectorCompare2, VectorInverse, VectorLength, VectorLengthSquared,
    VectorNPos, VectorNormalize, VectorNormalize2, VectorNormalizeRow, VectorSet, ANGLE2SHORT,
    PITCH, ROLL, SHORT2ANGLE, VEC3_ORIGIN, YAW,
};

/// Raven `SetPlaneSignbits`.
///
/// Source: `oracle/codemp/game/q_math.c:751-762`
pub fn SetPlaneSignbits(out: *mut cplane_t) {
    let out = unsafe { &mut *out };
    // for fast box on planeside test
    let mut bits: u8 = 0;
    for j in 0..3 {
        if out.normal[j] < 0.0 {
            bits |= 1 << j;
        }
    }
    out.signbits = bits;
}

/// Raven `BoxOnPlaneSide`.
///
/// Returns 1, 2, or 1 + 2. This is the fast axial/general-case version (the
/// naked-asm variant is dropped per the frozen fork ruling; the plain-C
/// fallback path is the one that ships).
/// Source: `oracle/codemp/game/q_math.c:809-871`
pub fn BoxOnPlaneSide(emins: vec3_t, emaxs: vec3_t, p: *mut cplane_t) -> c_int {
    let p = unsafe { &*p };

    // fast axial cases
    if (p.r#type as i32) < 3 {
        let t = p.r#type as usize;
        if p.dist <= emins[t] {
            return 1;
        }
        if p.dist >= emaxs[t] {
            return 2;
        }
        return 3;
    }

    // general case
    let (dist1, dist2) = match p.signbits {
        0 => (
            p.normal[0] * emaxs[0] + p.normal[1] * emaxs[1] + p.normal[2] * emaxs[2],
            p.normal[0] * emins[0] + p.normal[1] * emins[1] + p.normal[2] * emins[2],
        ),
        1 => (
            p.normal[0] * emins[0] + p.normal[1] * emaxs[1] + p.normal[2] * emaxs[2],
            p.normal[0] * emaxs[0] + p.normal[1] * emins[1] + p.normal[2] * emins[2],
        ),
        2 => (
            p.normal[0] * emaxs[0] + p.normal[1] * emins[1] + p.normal[2] * emaxs[2],
            p.normal[0] * emins[0] + p.normal[1] * emaxs[1] + p.normal[2] * emins[2],
        ),
        3 => (
            p.normal[0] * emins[0] + p.normal[1] * emins[1] + p.normal[2] * emaxs[2],
            p.normal[0] * emaxs[0] + p.normal[1] * emaxs[1] + p.normal[2] * emins[2],
        ),
        4 => (
            p.normal[0] * emaxs[0] + p.normal[1] * emaxs[1] + p.normal[2] * emins[2],
            p.normal[0] * emins[0] + p.normal[1] * emins[1] + p.normal[2] * emaxs[2],
        ),
        5 => (
            p.normal[0] * emins[0] + p.normal[1] * emaxs[1] + p.normal[2] * emins[2],
            p.normal[0] * emaxs[0] + p.normal[1] * emins[1] + p.normal[2] * emaxs[2],
        ),
        6 => (
            p.normal[0] * emaxs[0] + p.normal[1] * emins[1] + p.normal[2] * emins[2],
            p.normal[0] * emins[0] + p.normal[1] * emaxs[1] + p.normal[2] * emaxs[2],
        ),
        7 => (
            p.normal[0] * emins[0] + p.normal[1] * emins[1] + p.normal[2] * emins[2],
            p.normal[0] * emaxs[0] + p.normal[1] * emaxs[1] + p.normal[2] * emaxs[2],
        ),
        _ => (0.0, 0.0), // shut up compiler
    };

    let mut sides = 0;
    if dist1 >= p.dist {
        sides = 1;
    }
    if dist2 < p.dist {
        sides |= 2;
    }
    sides
}

/// Safe twin of [`BoxOnPlaneSide`] for callers that hold the plane by
/// reference. `&mut` mirrors Raven's non-const `struct cplane_s *p`, so a
/// caller passes the plane it stores — keeping that plane's `signbits` cache —
/// rather than a copy.
///
/// Source: `oracle/codemp/game/q_math.c:809-871`
pub fn BoxOnPlaneSideRef(emins: vec3_t, emaxs: vec3_t, p: &mut cplane_t) -> c_int {
    BoxOnPlaneSide(emins, emaxs, p as *mut cplane_t)
}

/// Raven `PlaneTypeForNormal`. The `q_math.c` function is `#if 0`'d out; the
/// live definition is the `q_shared.h` macro (`PLANE_NON_AXIAL` = 3).
///
/// Source: `oracle/codemp/game/q_shared.h:1856`
pub fn PlaneTypeForNormal(x: vec3_t) -> c_int {
    if x[0] == 1.0 {
        PLANE_X
    } else if x[1] == 1.0 {
        PLANE_Y
    } else if x[2] == 1.0 {
        PLANE_Z
    } else {
        3 // PLANE_NON_AXIAL
    }
}

/// Raven `Square` (`q_shared.h` macro).
///
/// Source: `oracle/codemp/game/q_shared.h:3005`
pub fn Square(x: vec_t) -> vec_t {
    x * x
}

/// Raven `Sys_SnapVector`. The unix build's C fallback (`Sys_SnapVector3`)
/// rounds each component with `rint` (round half-to-even in the default FPU
/// mode) — matching the win32 `fld`/`fistp` asm path; `round_ties_even`
/// reproduces that rounding exactly.
///
/// Source: `oracle/codemp/unix/unix_shared.cpp:76-81`
pub fn Sys_SnapVector(v: *mut f32) {
    unsafe {
        *v.add(0) = (*v.add(0) as f64).round_ties_even() as f32;
        *v.add(1) = (*v.add(1) as f64).round_ties_even() as f32;
        *v.add(2) = (*v.add(2) as f64).round_ties_even() as f32;
    }
}

pub use native_math::qmath::VectorAdvance;
