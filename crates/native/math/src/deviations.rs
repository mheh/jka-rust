//! Behaviorally divergent SP/MP q_math functions (see `tools/qmath-census.py`).
//!
//! Both modes' bodies are housed here side by side, suffixed `MP`/`SP`; each
//! mode's tree re-exports its own variant under the plain Raven name
//! (`pub use ... PerpendicularVectorMP as PerpendicularVector`), so call
//! sites never see the suffix and the other mode's variant stays unexported.
#![allow(non_snake_case, unused, clippy::all)]

use crate::qmath::{ProjectPointOnPlane, Q_fabs, VectorNormalize};
use crate::vector::{vec3_t, vec_t};

/// Raven `PerpendicularVector`.
///
/// Assumes "src" is normalized.
/// Source: `oracle/codemp/game/q_math.c:1353-1383`
pub fn PerpendicularVectorMP(dst: &mut vec3_t, src: vec3_t) {
    // find the smallest magnitude axially aligned vector
    let mut pos = 0usize;
    let mut minelem = 1.0f32;
    for (i, comp) in src.iter().enumerate() {
        if comp.abs() < minelem {
            pos = i;
            minelem = comp.abs();
        }
    }
    let mut tempvec: vec3_t = [0.0, 0.0, 0.0];
    tempvec[pos] = 1.0;

    // project the point onto the plane defined by src
    ProjectPointOnPlane(dst, tempvec, src);

    // normalize the result
    VectorNormalize(dst);
}

/// Raven SP `PerpendicularVector` — scans axes z->x ("bias towards using z
/// instead of x or y"), where MP scans x->z: same-magnitude components can
/// select a different axis, so the two modes genuinely diverge.
/// Source: `oracle/code/game/q_math.cpp` (`PerpendicularVector`)
pub fn PerpendicularVectorSP(dst: &mut vec3_t, src: vec3_t) {
    let mut pos: usize = 0;
    let mut minelem: f32 = 1.0;
    let mut tempvec: vec3_t = [0.0; 3];

    // find the smallest magnitude axially aligned vector, z->x with z bias
    let mut i: i32 = 2;
    while i >= 0 {
        if Q_fabs(src[i as usize]) < minelem {
            pos = i as usize;
            minelem = Q_fabs(src[i as usize]);
        }
        i -= 1;
    }
    tempvec[pos] = 1.0;

    // project the point onto the plane defined by src
    ProjectPointOnPlane(dst, tempvec, src);

    // normalize the result
    VectorNormalize(dst);
}

/// Raven `ClearBounds`.
///
/// Source: `oracle/codemp/game/q_math.c:1129-1132`
pub fn ClearBoundsMP(mins: &mut vec3_t, maxs: &mut vec3_t) {
    mins[0] = 99999.0;
    mins[1] = 99999.0;
    mins[2] = 99999.0;
    maxs[0] = -99999.0;
    maxs[1] = -99999.0;
    maxs[2] = -99999.0;
}

/// Raven SP `ClearBounds` — seeds with `WORLD_SIZE` (`MAX_WORLD_COORD -
/// MIN_WORLD_COORD` = 131072), where MP seeds with `99999`: bounds that never
/// accumulate a point read back differently per mode.
/// Source: `oracle/code/game/q_math.cpp` (`ClearBounds`);
/// `oracle/code/game/q_shared.h:1599-1601` (`WORLD_SIZE`)
pub fn ClearBoundsSP(mins: &mut vec3_t, maxs: &mut vec3_t) {
    const WORLD_SIZE: vec_t = 131072.0;
    mins[0] = WORLD_SIZE;
    mins[1] = WORLD_SIZE;
    mins[2] = WORLD_SIZE;
    maxs[0] = -WORLD_SIZE;
    maxs[1] = -WORLD_SIZE;
    maxs[2] = -WORLD_SIZE;
}
