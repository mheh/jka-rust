//! Raven `q_math.c` — the SP/MP-shared core (see `tools/qmath-census.py`).
//!
//! - PerpendicularVectorMP/SP
//! - ClearBoundsMP/SP
//!
//! Sources:
//! - `oracle/codemp/game/q_math.c` / `oracle/code/game/q_math.cpp`
//! - `oracle/codemp/game/q_shared.h`
#![allow(non_snake_case, unused, clippy::all)]

use crate::vector::{vec3_t, vec4_t, vec_t};
use core::ffi::{c_int, c_schar, c_short, c_uint};

pub type qboolean = c_int;
type byte = u8;

pub const PITCH: usize = 0;
pub const YAW: usize = 1;
pub const ROLL: usize = 2;
pub const VEC3_ORIGIN: vec3_t = [0.0, 0.0, 0.0];

// kept as an alias rather
#[allow(non_upper_case_globals)]
pub const vec3_origin: vec3_t = VEC3_ORIGIN;

/// Raven `bytedirs[NUMVERTEXNORMALS]`
/// precomputed icosahedron-subdivision unit normals used by `DirToByte`/`ByteToDir`.
/// Source: `oracle/codemp/game/q_math.c:39-122`
pub const BYTEDIRS: [vec3_t; 162] = [
    [-0.525731f32, 0.000000f32, 0.850651f32],
    [-0.442863f32, 0.238856f32, 0.864188f32],
    [-0.295242f32, 0.000000f32, 0.955423f32],
    [-0.309017f32, 0.500000f32, 0.809017f32],
    [-0.162460f32, 0.262866f32, 0.951056f32],
    [0.000000f32, 0.000000f32, 1.000000f32],
    [0.000000f32, 0.850651f32, 0.525731f32],
    [-0.147621f32, 0.716567f32, 0.681718f32],
    [0.147621f32, 0.716567f32, 0.681718f32],
    [0.000000f32, 0.525731f32, 0.850651f32],
    [0.309017f32, 0.500000f32, 0.809017f32],
    [0.525731f32, 0.000000f32, 0.850651f32],
    [0.295242f32, 0.000000f32, 0.955423f32],
    [0.442863f32, 0.238856f32, 0.864188f32],
    [0.162460f32, 0.262866f32, 0.951056f32],
    [-0.681718f32, 0.147621f32, 0.716567f32],
    [-0.809017f32, 0.309017f32, 0.500000f32],
    [-0.587785f32, 0.425325f32, 0.688191f32],
    [-0.850651f32, 0.525731f32, 0.000000f32],
    [-0.864188f32, 0.442863f32, 0.238856f32],
    [-0.716567f32, 0.681718f32, 0.147621f32],
    [-0.688191f32, 0.587785f32, 0.425325f32],
    [-0.500000f32, 0.809017f32, 0.309017f32],
    [-0.238856f32, 0.864188f32, 0.442863f32],
    [-0.425325f32, 0.688191f32, 0.587785f32],
    [-0.716567f32, 0.681718f32, -0.147621f32],
    [-0.500000f32, 0.809017f32, -0.309017f32],
    [-0.525731f32, 0.850651f32, 0.000000f32],
    [0.000000f32, 0.850651f32, -0.525731f32],
    [-0.238856f32, 0.864188f32, -0.442863f32],
    [0.000000f32, 0.955423f32, -0.295242f32],
    [-0.262866f32, 0.951056f32, -0.162460f32],
    [0.000000f32, 1.000000f32, 0.000000f32],
    [0.000000f32, 0.955423f32, 0.295242f32],
    [-0.262866f32, 0.951056f32, 0.162460f32],
    [0.238856f32, 0.864188f32, 0.442863f32],
    [0.262866f32, 0.951056f32, 0.162460f32],
    [0.500000f32, 0.809017f32, 0.309017f32],
    [0.238856f32, 0.864188f32, -0.442863f32],
    [0.262866f32, 0.951056f32, -0.162460f32],
    [0.500000f32, 0.809017f32, -0.309017f32],
    [0.850651f32, 0.525731f32, 0.000000f32],
    [0.716567f32, 0.681718f32, 0.147621f32],
    [0.716567f32, 0.681718f32, -0.147621f32],
    [0.525731f32, 0.850651f32, 0.000000f32],
    [0.425325f32, 0.688191f32, 0.587785f32],
    [0.864188f32, 0.442863f32, 0.238856f32],
    [0.688191f32, 0.587785f32, 0.425325f32],
    [0.809017f32, 0.309017f32, 0.500000f32],
    [0.681718f32, 0.147621f32, 0.716567f32],
    [0.587785f32, 0.425325f32, 0.688191f32],
    [0.955423f32, 0.295242f32, 0.000000f32],
    [1.000000f32, 0.000000f32, 0.000000f32],
    [0.951056f32, 0.162460f32, 0.262866f32],
    [0.850651f32, -0.525731f32, 0.000000f32],
    [0.955423f32, -0.295242f32, 0.000000f32],
    [0.864188f32, -0.442863f32, 0.238856f32],
    [0.951056f32, -0.162460f32, 0.262866f32],
    [0.809017f32, -0.309017f32, 0.500000f32],
    [0.681718f32, -0.147621f32, 0.716567f32],
    [0.850651f32, 0.000000f32, 0.525731f32],
    [0.864188f32, 0.442863f32, -0.238856f32],
    [0.809017f32, 0.309017f32, -0.500000f32],
    [0.951056f32, 0.162460f32, -0.262866f32],
    [0.525731f32, 0.000000f32, -0.850651f32],
    [0.681718f32, 0.147621f32, -0.716567f32],
    [0.681718f32, -0.147621f32, -0.716567f32],
    [0.850651f32, 0.000000f32, -0.525731f32],
    [0.809017f32, -0.309017f32, -0.500000f32],
    [0.864188f32, -0.442863f32, -0.238856f32],
    [0.951056f32, -0.162460f32, -0.262866f32],
    [0.147621f32, 0.716567f32, -0.681718f32],
    [0.309017f32, 0.500000f32, -0.809017f32],
    [0.425325f32, 0.688191f32, -0.587785f32],
    [0.442863f32, 0.238856f32, -0.864188f32],
    [0.587785f32, 0.425325f32, -0.688191f32],
    [0.688191f32, 0.587785f32, -0.425325f32],
    [-0.147621f32, 0.716567f32, -0.681718f32],
    [-0.309017f32, 0.500000f32, -0.809017f32],
    [0.000000f32, 0.525731f32, -0.850651f32],
    [-0.525731f32, 0.000000f32, -0.850651f32],
    [-0.442863f32, 0.238856f32, -0.864188f32],
    [-0.295242f32, 0.000000f32, -0.955423f32],
    [-0.162460f32, 0.262866f32, -0.951056f32],
    [0.000000f32, 0.000000f32, -1.000000f32],
    [0.295242f32, 0.000000f32, -0.955423f32],
    [0.162460f32, 0.262866f32, -0.951056f32],
    [-0.442863f32, -0.238856f32, -0.864188f32],
    [-0.309017f32, -0.500000f32, -0.809017f32],
    [-0.162460f32, -0.262866f32, -0.951056f32],
    [0.000000f32, -0.850651f32, -0.525731f32],
    [-0.147621f32, -0.716567f32, -0.681718f32],
    [0.147621f32, -0.716567f32, -0.681718f32],
    [0.000000f32, -0.525731f32, -0.850651f32],
    [0.309017f32, -0.500000f32, -0.809017f32],
    [0.442863f32, -0.238856f32, -0.864188f32],
    [0.162460f32, -0.262866f32, -0.951056f32],
    [0.238856f32, -0.864188f32, -0.442863f32],
    [0.500000f32, -0.809017f32, -0.309017f32],
    [0.425325f32, -0.688191f32, -0.587785f32],
    [0.716567f32, -0.681718f32, -0.147621f32],
    [0.688191f32, -0.587785f32, -0.425325f32],
    [0.587785f32, -0.425325f32, -0.688191f32],
    [0.000000f32, -0.955423f32, -0.295242f32],
    [0.000000f32, -1.000000f32, 0.000000f32],
    [0.262866f32, -0.951056f32, -0.162460f32],
    [0.000000f32, -0.850651f32, 0.525731f32],
    [0.000000f32, -0.955423f32, 0.295242f32],
    [0.238856f32, -0.864188f32, 0.442863f32],
    [0.262866f32, -0.951056f32, 0.162460f32],
    [0.500000f32, -0.809017f32, 0.309017f32],
    [0.716567f32, -0.681718f32, 0.147621f32],
    [0.525731f32, -0.850651f32, 0.000000f32],
    [-0.238856f32, -0.864188f32, -0.442863f32],
    [-0.500000f32, -0.809017f32, -0.309017f32],
    [-0.262866f32, -0.951056f32, -0.162460f32],
    [-0.850651f32, -0.525731f32, 0.000000f32],
    [-0.716567f32, -0.681718f32, -0.147621f32],
    [-0.716567f32, -0.681718f32, 0.147621f32],
    [-0.525731f32, -0.850651f32, 0.000000f32],
    [-0.500000f32, -0.809017f32, 0.309017f32],
    [-0.238856f32, -0.864188f32, 0.442863f32],
    [-0.262866f32, -0.951056f32, 0.162460f32],
    [-0.864188f32, -0.442863f32, 0.238856f32],
    [-0.809017f32, -0.309017f32, 0.500000f32],
    [-0.688191f32, -0.587785f32, 0.425325f32],
    [-0.681718f32, -0.147621f32, 0.716567f32],
    [-0.442863f32, -0.238856f32, 0.864188f32],
    [-0.587785f32, -0.425325f32, 0.688191f32],
    [-0.309017f32, -0.500000f32, 0.809017f32],
    [-0.147621f32, -0.716567f32, 0.681718f32],
    [-0.425325f32, -0.688191f32, 0.587785f32],
    [-0.162460f32, -0.262866f32, 0.951056f32],
    [0.442863f32, -0.238856f32, 0.864188f32],
    [0.162460f32, -0.262866f32, 0.951056f32],
    [0.309017f32, -0.500000f32, 0.809017f32],
    [0.147621f32, -0.716567f32, 0.681718f32],
    [0.000000f32, -0.525731f32, 0.850651f32],
    [0.425325f32, -0.688191f32, 0.587785f32],
    [0.587785f32, -0.425325f32, 0.688191f32],
    [0.688191f32, -0.587785f32, 0.425325f32],
    [-0.955423f32, 0.295242f32, 0.000000f32],
    [-0.951056f32, 0.162460f32, 0.262866f32],
    [-1.000000f32, 0.000000f32, 0.000000f32],
    [-0.850651f32, 0.000000f32, 0.525731f32],
    [-0.955423f32, -0.295242f32, 0.000000f32],
    [-0.951056f32, -0.162460f32, 0.262866f32],
    [-0.864188f32, 0.442863f32, -0.238856f32],
    [-0.951056f32, 0.162460f32, -0.262866f32],
    [-0.809017f32, 0.309017f32, -0.500000f32],
    [-0.864188f32, -0.442863f32, -0.238856f32],
    [-0.951056f32, -0.162460f32, -0.262866f32],
    [-0.809017f32, -0.309017f32, -0.500000f32],
    [-0.681718f32, 0.147621f32, -0.716567f32],
    [-0.681718f32, -0.147621f32, -0.716567f32],
    [-0.850651f32, 0.000000f32, -0.525731f32],
    [-0.688191f32, 0.587785f32, -0.425325f32],
    [-0.587785f32, 0.425325f32, -0.688191f32],
    [-0.425325f32, 0.688191f32, -0.587785f32],
    [-0.425325f32, -0.688191f32, -0.587785f32],
    [-0.587785f32, -0.425325f32, -0.688191f32],
    [-0.688191f32, -0.587785f32, -0.425325f32],
];

/// Raven `qboolean` true/false values.
/// Source: `oracle/codemp/game/q_shared.h` (`qtrue`/`qfalse`).
const Q_TRUE: qboolean = 1;
const Q_FALSE: qboolean = 0;

/// Raven `CrossProduct` (header-inline helper).
/// Source: `oracle/codemp/game/q_shared.h:1553-1557`
pub fn CrossProduct(v1: vec3_t, v2: vec3_t, cross: &mut vec3_t) {
    cross[0] = v1[1] * v2[2] - v1[2] * v2[1];
    cross[1] = v1[2] * v2[0] - v1[0] * v2[2];
    cross[2] = v1[0] * v2[1] - v1[1] * v2[0];
}

/// Raven `VectorLength` (header-inline helper; `_XBOX` asm branch dropped, the
/// plain-C branch is the compiled one).
/// Source: `oracle/codemp/game/q_shared.h:1460-1489`
pub fn VectorLength(v: vec3_t) -> vec_t {
    // Raven: `(vec_t)sqrt(..)`. The sum is float; `sqrt` is the double libm call
    // rounded back to float. An f32 sqrt double-rounds and diverges from the
    // oracle, so compute the sqrt in f64.
    ((v[0] * v[0] + v[1] * v[1] + v[2] * v[2]) as f64).sqrt() as f32
}

/// Raven `VectorLengthSquared` (header-inline helper; `_XBOX` asm branch dropped).
/// Source: `oracle/codemp/game/q_shared.h:1491-1518`
pub fn VectorLengthSquared(v: vec3_t) -> vec_t {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

/// Raven `Distance` (header-inline helper).
/// Source: `oracle/codemp/game/q_shared.h:1520-1525`
pub fn Distance(p1: vec3_t, p2: vec3_t) -> vec_t {
    let mut v: vec3_t = [0.0; 3];
    _VectorSubtract(p2, p1, &mut v);
    VectorLength(v)
}

/// Raven `VectorCompare` (header-inline helper).
/// Returns 1 if vectors are equal, 0 otherwise.
/// Source: `oracle/codemp/game/q_shared.h:1527-1532`
// Raven returns qboolean; C truthiness call shape maps to a Rust bool (§C7).
pub fn VectorCompare(v1: vec3_t, v2: vec3_t) -> bool {
    !(v1[0] != v2[0] || v1[1] != v2[1] || v1[2] != v2[2])
}

/// Raven `VectorClear` (`q_shared.h` macro; canonical home for the per-file
/// transcriptions that porters had been re-deriving).
/// Source: `oracle/codemp/game/q_shared.h:1397`
#[inline]
pub fn VectorClear(a: &mut vec3_t) {
    a[0] = 0.0;
    a[1] = 0.0;
    a[2] = 0.0;
}

/// Raven `VectorSet` (`q_shared.h` macro). Out-param, matching the macro's
/// `(v)[0]=(x),…` write-through shape.
/// Source: `oracle/codemp/game/q_shared.h:1399`
#[inline]
pub fn VectorSet(v: &mut vec3_t, x: f32, y: f32, z: f32) {
    v[0] = x;
    v[1] = y;
    v[2] = z;
}

/// Raven `DistanceSquared` (header-inline helper).
/// Source: `oracle/codemp/game/q_shared.h:1527-1532`
#[inline]
pub fn DistanceSquared(p1: vec3_t, p2: vec3_t) -> vec_t {
    let mut v: vec3_t = [0.0; 3];
    _VectorSubtract(p2, p1, &mut v);
    VectorLengthSquared(v)
}

/// Raven `Q_rand`.
///
/// Source: `oracle/codemp/game/q_math.c:126-129`
pub fn Q_rand(seed: *mut c_int) -> c_int {
    unsafe {
        *seed = (69069i32).wrapping_mul(*seed).wrapping_add(1);
        *seed
    }
}

/// Raven `Q_random`.
///
/// Source: `oracle/codemp/game/q_math.c:131-133`
pub fn Q_random(seed: *mut c_int) -> f32 {
    (Q_rand(seed) & 0xffff) as f32 / 0x10000 as f32
}

/// Raven `Q_crandom`.
///
/// Source: `oracle/codemp/game/q_math.c:135-137`
pub fn Q_crandom(seed: *mut c_int) -> f32 {
    2.0 * (Q_random(seed) - 0.5)
}

// `random()`/`crandom()` (`q_shared.h:1591-1592`) are ported as methods on
// `bg_channel::rng::Rng` (BgState) — `Rng::random`/`Rng::crandom`.

/// Raven `ClampChar`.
///
/// Source: `oracle/codemp/game/q_math.c:279-287`
pub fn ClampChar(i: c_int) -> c_schar {
    if i < -128 {
        return -128;
    }
    if i > 127 {
        return 127;
    }
    i as c_schar
}

/// Raven `ClampShort`.
///
/// Source: `oracle/codemp/game/q_math.c:289-297`
pub fn ClampShort(i: c_int) -> c_short {
    if i < -32768 {
        return -32768;
    }
    if i > 0x7fff {
        return 0x7fff;
    }
    i as c_short
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

/// Raven `DirToByte`.
///
/// Raven checks `!dir` for a null pointer; a Rust `vec3_t` value can't be
/// null, so that branch is unreachable here (S19: diverges only on Raven UB).
/// Source: `oracle/codemp/game/q_math.c:301-322`
pub fn DirToByte(dir: vec3_t) -> c_int {
    let mut bestd = 0.0f32;
    let mut best = 0i32;
    for (i, bd) in BYTEDIRS.iter().enumerate() {
        let d = _DotProduct(dir, *bd);
        if d > bestd {
            bestd = d;
            best = i as i32;
        }
    }
    best
}

/// Raven `ByteToDir`.
///
/// Source: `oracle/codemp/game/q_math.c:324-330`
pub fn ByteToDir(b: c_int, dir: &mut vec3_t) {
    if b < 0 || b as usize >= BYTEDIRS.len() {
        *dir = VEC3_ORIGIN;
        return;
    }
    *dir = BYTEDIRS[b as usize];
}

/// Raven `ColorBytes3`.
///
/// Source: `oracle/codemp/game/q_math.c:333-341`
pub fn ColorBytes3(r: f32, g: f32, b: f32) -> c_uint {
    // §19: oracle leaves the 4th packed byte uninitialized (UB); we zero it.
    let bytes = [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, 0u8];
    u32::from_ne_bytes(bytes) as c_uint
}

/// Raven `ColorBytes4`.
///
/// Source: `oracle/codemp/game/q_math.c:343-352`
pub fn ColorBytes4(r: f32, g: f32, b: f32, a: f32) -> c_uint {
    let bytes = [
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
        (a * 255.0) as u8,
    ];
    u32::from_ne_bytes(bytes) as c_uint
}

/// Raven `NormalizeColor`.
///
/// Source: `oracle/codemp/game/q_math.c:354-373`
pub fn NormalizeColor(r#in: vec3_t, out: &mut vec3_t) -> f32 {
    let mut max = r#in[0];
    if r#in[1] > max {
        max = r#in[1];
    }
    if r#in[2] > max {
        max = r#in[2];
    }

    if max == 0.0 {
        *out = VEC3_ORIGIN;
    } else {
        out[0] = r#in[0] / max;
        out[1] = r#in[1] / max;
        out[2] = r#in[2] / max;
    }
    max
}

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

/// Raven `PlaneFromPoints`.
///
/// Returns false if the triangle is degenerate. The normal points out of the
/// clock for clockwise ordered points.
/// Source: `oracle/codemp/game/q_math.c:384-396`
// Raven returns qboolean; C truthiness call shape maps to a Rust bool (§C7).
pub fn PlaneFromPoints(plane: &mut vec4_t, a: vec3_t, b: vec3_t, c: vec3_t) -> bool {
    let mut d1: vec3_t = [0.0; 3];
    _VectorSubtract(b, a, &mut d1);
    let mut d2: vec3_t = [0.0; 3];
    _VectorSubtract(c, a, &mut d2);
    let mut n: vec3_t = [0.0; 3];
    CrossProduct(d2, d1, &mut n);
    let len = VectorNormalize(&mut n);
    plane[0] = n[0];
    plane[1] = n[1];
    plane[2] = n[2];
    if len == 0.0 {
        return false;
    }
    plane[3] = _DotProduct(a, n);
    true
}

/// Raven `RotatePointAroundVector`.
///
/// This is not implemented very well...
/// Source: `oracle/codemp/game/q_math.c:405-459`
pub fn RotatePointAroundVector(dst: &mut vec3_t, dir: vec3_t, point: vec3_t, degrees: f32) {
    let mut m = [[0f32; 3]; 3];
    let mut zrot = [[0f32; 3]; 3];
    let mut tmpmat;
    let mut rot;
    let mut vr: vec3_t = [0.0; 3];
    let mut vup: vec3_t = [0.0; 3];
    let vf = dir;

    PerpendicularVectorMP(&mut vr, dir);
    CrossProduct(vr, vf, &mut vup);

    m[0][0] = vr[0];
    m[1][0] = vr[1];
    m[2][0] = vr[2];

    m[0][1] = vup[0];
    m[1][1] = vup[1];
    m[2][1] = vup[2];

    m[0][2] = vf[0];
    m[1][2] = vf[1];
    m[2][2] = vf[2];

    let mut im = m; // memcpy( im, m, sizeof( im ) );

    im[0][1] = m[1][0];
    im[0][2] = m[2][0];
    im[1][0] = m[0][1];
    im[1][2] = m[2][1];
    im[2][0] = m[0][2];
    im[2][1] = m[1][2];

    zrot[0][0] = 1.0;
    zrot[1][1] = 1.0;
    zrot[2][2] = 1.0;

    // Raven: `rad = DEG2RAD(degrees)` = `(degrees*M_PI)/180.0F` with M_PI the
    // double from math.h on the native build; cos/sin are double libm rounded
    // to float. f32 trig here diverges from the oracle.
    let rad = ((degrees as f64 * std::f64::consts::PI) / 180.0) as f32;
    zrot[0][0] = (rad as f64).cos() as f32;
    zrot[0][1] = (rad as f64).sin() as f32;
    zrot[1][0] = -((rad as f64).sin() as f32);
    zrot[1][1] = (rad as f64).cos() as f32;

    tmpmat = [[0f32; 3]; 3];
    MatrixMultiply(&m, &zrot, &mut tmpmat);
    rot = [[0f32; 3]; 3];
    MatrixMultiply(&tmpmat, &im, &mut rot);

    for i in 0..3 {
        dst[i] = rot[i][0] * point[0] + rot[i][1] * point[1] + rot[i][2] * point[2];
    }
}

/// Raven `RotateAroundDirection`.
///
/// Source: `oracle/codemp/game/q_math.c:466-481`
pub fn RotateAroundDirection(axis: *mut vec3_t, yaw: f32) {
    // `axis` points at a 3-row `vec3_t axis[3]` matrix; reinterpret as a
    // fixed-size array so the rows can be indexed/borrowed safely below.
    let axis: &mut [vec3_t; 3] = unsafe { &mut *(axis as *mut [vec3_t; 3]) };

    // create an arbitrary axis[1]
    let a0 = axis[0];
    PerpendicularVectorMP(&mut axis[1], a0);

    // rotate it around axis[0] by yaw
    if yaw != 0.0 {
        let temp = axis[1];
        let a0 = axis[0];
        RotatePointAroundVector(&mut axis[1], a0, temp, yaw);
    }

    // cross to get axis[2]
    let a0 = axis[0];
    let a1 = axis[1];
    CrossProduct(a0, a1, &mut axis[2]);
}

/// Raven `vectoangles`.
///
/// Source: `oracle/codemp/game/q_math.c:485-522`
pub fn vectoangles(value1: vec3_t, angles: &mut vec3_t) {
    let yaw;
    let mut pitch;

    if value1[1] == 0.0 && value1[0] == 0.0 {
        yaw = 0.0;
        pitch = if value1[2] > 0.0 { 90.0 } else { 270.0 };
    } else {
        // Raven's atan2/sqrt are double libm and M_PI is math.h's double; the
        // `*180/M_PI` chain evaluates in f64 then rounds to the float result.
        let mut y = if value1[0] != 0.0 {
            ((value1[1] as f64).atan2(value1[0] as f64) * 180.0 / std::f64::consts::PI) as f32
        } else if value1[1] > 0.0 {
            90.0
        } else {
            270.0
        };
        if y < 0.0 {
            y += 360.0;
        }
        yaw = y;

        let forward = ((value1[0] * value1[0] + value1[1] * value1[1]) as f64).sqrt() as f32;
        pitch = ((value1[2] as f64).atan2(forward as f64) * 180.0 / std::f64::consts::PI) as f32;
        if pitch < 0.0 {
            pitch += 360.0;
        }
    }

    angles[PITCH] = -pitch;
    angles[YAW] = yaw;
    angles[ROLL] = 0.0;
}

/// Raven `AnglesToAxis`.
///
/// Source: `oracle/codemp/game/q_math.c:530-536`
pub fn AnglesToAxis(angles: vec3_t, axis: *mut vec3_t) {
    let axis: &mut [vec3_t; 3] = unsafe { &mut *(axis as *mut [vec3_t; 3]) };
    let mut right: vec3_t = [0.0; 3];
    {
        // angle vectors returns "right" instead of "y axis"
        let (first, rest) = axis.split_at_mut(1);
        let (_, third) = rest.split_at_mut(1);
        AngleVectors(
            angles,
            Some(&mut first[0]),
            Some(&mut right),
            Some(&mut third[0]),
        );
    }
    _VectorSubtract(VEC3_ORIGIN, right, &mut axis[1]);
}

/// Raven `AxisClear`.
///
/// Source: `oracle/codemp/game/q_math.c:538-548`
pub fn AxisClear(axis: *mut vec3_t) {
    let axis: &mut [vec3_t; 3] = unsafe { &mut *(axis as *mut [vec3_t; 3]) };
    axis[0] = [1.0, 0.0, 0.0];
    axis[1] = [0.0, 1.0, 0.0];
    axis[2] = [0.0, 0.0, 1.0];
}

/// Raven `AxisCopy`.
///
/// Source: `oracle/codemp/game/q_math.c:550-554`
pub fn AxisCopy(r#in: *mut vec3_t, out: *mut vec3_t) {
    let r#in: &[vec3_t; 3] = unsafe { &*(r#in as *const [vec3_t; 3]) };
    let out: &mut [vec3_t; 3] = unsafe { &mut *(out as *mut [vec3_t; 3]) };
    out[0] = r#in[0];
    out[1] = r#in[1];
    out[2] = r#in[2];
}

/// Raven `ProjectPointOnPlane`.
///
/// Source: `oracle/codemp/game/q_math.c:556-577`
pub fn ProjectPointOnPlane(dst: &mut vec3_t, p: vec3_t, normal: vec3_t) {
    let mut inv_denom = _DotProduct(normal, normal);
    // Raven's debug assert (`Q_fabs(inv_denom) != 0.0f`) catches a zero
    // normal; that's a caller bug (division by zero), not something to
    // silently normalize away.
    debug_assert!(Q_fabs(inv_denom) != 0.0);
    inv_denom = 1.0 / inv_denom;

    let d = _DotProduct(normal, p) * inv_denom;

    let n = [
        normal[0] * inv_denom,
        normal[1] * inv_denom,
        normal[2] * inv_denom,
    ];

    dst[0] = p[0] - d * n[0];
    dst[1] = p[1] - d * n[1];
    dst[2] = p[2] - d * n[2];
}

/// Raven `MakeNormalVectors`.
///
/// Given a normalized forward vector, create two other perpendicular vectors.
/// Source: `oracle/codemp/game/q_math.c:587-600`
pub fn MakeNormalVectors(forward: vec3_t, right: &mut vec3_t, up: &mut vec3_t) {
    // this rotate and negate guarantees a vector not colinear with the original
    right[1] = -forward[0];
    right[2] = forward[1];
    right[0] = forward[2];

    let d = _DotProduct(*right, forward);
    let r = *right;
    _VectorMA(r, -d, forward, right);
    VectorNormalize(right);
    CrossProduct(*right, forward, up);
}

/// Raven `VectorRotate`.
///
/// Source: `oracle/codemp/game/q_math.c:603-608`
pub fn VectorRotate(r#in: vec3_t, matrix: *mut vec3_t, out: &mut vec3_t) {
    let matrix: &[vec3_t; 3] = unsafe { &*(matrix as *const [vec3_t; 3]) };
    out[0] = _DotProduct(r#in, matrix[0]);
    out[1] = _DotProduct(r#in, matrix[1]);
    out[2] = _DotProduct(r#in, matrix[2]);
}

/// Raven `Q_rsqrt` ("fast inverse square root"; evil floating point bit level
/// hacking).
///
/// Source: `oracle/codemp/game/q_math.c:616-636`
pub fn Q_rsqrt(number: f32) -> f32 {
    let threehalfs = 1.5f32;
    let x2 = number * 0.5;
    let mut i = number.to_bits() as i32;
    // C's `int i = 0x5f3759df - (i>>1)` wraps; Rust's `-` panics in debug for
    // inputs whose shifted bits exceed the constant. `wrapping_sub` matches.
    i = 0x5f3759df_i32.wrapping_sub(i >> 1); // what the fuck?
    let mut y = f32::from_bits(i as u32);
    y = y * (threehalfs - (x2 * y * y)); // 1st iteration
    y
}

/// Raven `Q_fabs`.
///
/// Source: `oracle/codemp/game/q_math.c:638-642`
pub fn Q_fabs(f: f32) -> f32 {
    let tmp = (f.to_bits() as i32) & 0x7FFFFFFF;
    f32::from_bits(tmp as u32)
}

/// Raven `LerpAngle`.
///
/// Source: `oracle/codemp/game/q_math.c:653-665`
pub fn LerpAngle(from: f32, to: f32, frac: f32) -> f32 {
    let mut to = to;
    if to - from > 180.0 {
        to -= 360.0;
    }
    if to - from < -180.0 {
        to += 360.0;
    }
    from + frac * (to - from)
}

/// Raven `AngleDifference` — signed shortest-arc difference between two
/// angles; duplicated verbatim in botlib as `AngleDiff`.
///
/// Source: `oracle/codemp/game/ai_main.c:425-436`
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:203-217` (`AngleDiff`)
pub fn AngleDifference(ang1: f32, ang2: f32) -> f32 {
    let mut diff = ang1 - ang2;
    if ang1 > ang2 {
        if diff > 180.0 {
            diff -= 360.0;
        }
    } else if diff < -180.0 {
        diff += 360.0;
    }
    diff
}

/// Raven `static float AngleNormZero(float theta)` — `fmodf` into
/// `[-180, 180]` (distinct from the quantizing `AngleNormalize180`).
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:3936-3949`
pub fn AngleNormZero(theta: f32) -> f32 {
    let mut ret = theta % 360.0;
    if ret < -180.0 {
        ret += 360.0;
    } else if ret > 180.0 {
        ret -= 360.0;
    }
    ret
}

/// Raven `ANGLE2SHORT(x)` — `((int)((x)*65536/360) & 65535)`.
/// Source: `oracle/codemp/game/q_shared.h:1972`
pub fn ANGLE2SHORT(x: f32) -> c_int {
    ((x * 65536.0 / 360.0) as c_int) & 65535
}

/// Raven `SHORT2ANGLE(x)` — `((x)*(360.0/65536))`.
/// Source: `oracle/codemp/game/q_shared.h:1973`
pub fn SHORT2ANGLE(x: c_int) -> f32 {
    (x as f32) * (360.0 / 65536.0)
}

/// Raven `AngleSubtract`.
///
/// Always returns a value from -180 to 180.
/// Source: `oracle/codemp/game/q_math.c:675-687`
pub fn AngleSubtract(a1: f32, a2: f32) -> f32 {
    let mut a = a1 - a2;
    a %= 360.0; // chop it down quickly, then level it out (Rust `%` matches C `fmod`)
    while a > 180.0 {
        a -= 360.0;
    }
    while a < -180.0 {
        a += 360.0;
    }
    a
}

/// Raven `AnglesSubtract`.
///
/// Source: `oracle/codemp/game/q_math.c:690-694`
pub fn AnglesSubtract(v1: vec3_t, v2: vec3_t, v3: &mut vec3_t) {
    v3[0] = AngleSubtract(v1[0], v2[0]);
    v3[1] = AngleSubtract(v1[1], v2[1]);
    v3[2] = AngleSubtract(v1[2], v2[2]);
}

/// Raven `AngleMod`.
///
/// Source: `oracle/codemp/game/q_math.c:697-700`
pub fn AngleMod(a: f32) -> f32 {
    // Raven's `65536/360.0` and `360.0/65536` are double literals, so the
    // scale and product evaluate in f64 (rounded to the float result); an
    // all-f32 form diverges from the oracle at the int-truncation boundary.
    ((360.0f64 / 65536.0) * (((a as f64 * (65536.0 / 360.0)) as i32) & 65535) as f64) as f32
}

/// Raven `AngleNormalize360`.
///
/// Returns angle normalized to the range [0 <= angle < 360].
/// Source: `oracle/codemp/game/q_math.c:710-712`
pub fn AngleNormalize360(angle: f32) -> f32 {
    // f64 constant math, matching Raven's double literals (see `AngleMod`).
    ((360.0f64 / 65536.0) * (((angle as f64 * (65536.0 / 360.0)) as i32) & 65535) as f64) as f32
}

/// Raven `AngleNormalize180`.
///
/// Returns angle normalized to the range [-180 < angle <= 180].
/// Source: `oracle/codemp/game/q_math.c:722-728`
pub fn AngleNormalize180(angle: f32) -> f32 {
    let mut angle = AngleNormalize360(angle);
    if angle > 180.0 {
        angle -= 360.0;
    }
    angle
}

/// Raven `AngleDelta`.
///
/// Returns the normalized delta from angle1 to angle2.
/// Source: `oracle/codemp/game/q_math.c:738-740`
pub fn AngleDelta(angle1: f32, angle2: f32) -> f32 {
    AngleNormalize180(angle1 - angle2)
}

/// Raven `RadiusFromBounds`.
///
/// Source: `oracle/codemp/game/q_math.c:1114-1126`
pub fn RadiusFromBounds(mins: vec3_t, maxs: vec3_t) -> f32 {
    let mut corner: vec3_t = [0.0; 3];
    for i in 0..3 {
        let a = mins[i].abs();
        let b = maxs[i].abs();
        corner[i] = if a > b { a } else { b };
    }
    VectorLength(corner)
}

/// Raven `DistanceHorizontal`.
///
/// Source: `oracle/codemp/game/q_math.c:1134-1139`
pub fn DistanceHorizontal(p1: vec3_t, p2: vec3_t) -> vec_t {
    let mut v: vec3_t = [0.0; 3];
    _VectorSubtract(p2, p1, &mut v);
    ((v[0] * v[0] + v[1] * v[1]) as f64).sqrt() as f32 // z left off; sqrt in f64 (Raven double libm)
}

/// Raven `DistanceHorizontalSquared`.
///
/// Source: `oracle/codemp/game/q_math.c:1141-1146`
pub fn DistanceHorizontalSquared(p1: vec3_t, p2: vec3_t) -> vec_t {
    let mut v: vec3_t = [0.0; 3];
    _VectorSubtract(p2, p1, &mut v);
    v[0] * v[0] + v[1] * v[1] // Leave off the z component
}

/// Raven `AddPointToBounds`.
///
/// Source: `oracle/codemp/game/q_math.c:1148-1169`
pub fn AddPointToBounds(v: vec3_t, mins: &mut vec3_t, maxs: &mut vec3_t) {
    if v[0] < mins[0] {
        mins[0] = v[0];
    }
    if v[0] > maxs[0] {
        maxs[0] = v[0];
    }

    if v[1] < mins[1] {
        mins[1] = v[1];
    }
    if v[1] > maxs[1] {
        maxs[1] = v[1];
    }

    if v[2] < mins[2] {
        mins[2] = v[2];
    }
    if v[2] > maxs[2] {
        maxs[2] = v[2];
    }
}

/// Raven `VectorNormalize`.
///
/// Source: `oracle/codemp/game/q_math.c:1172-1186`
pub fn VectorNormalize(v: &mut vec3_t) -> vec_t {
    let mut length = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    length = (length as f64).sqrt() as f32; // Raven `sqrt` is double libm rounded to float

    if length != 0.0 {
        let ilength = 1.0 / length;
        v[0] *= ilength;
        v[1] *= ilength;
        v[2] *= ilength;
    }

    length
}

/// Raven `VectorNormalize2`.
///
/// Source: `oracle/codemp/game/q_math.c:1188-1212`
pub fn VectorNormalize2(v: vec3_t, out: &mut vec3_t) -> vec_t {
    let mut length = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    length = (length as f64).sqrt() as f32; // Raven `sqrt` is double libm rounded to float

    if length != 0.0 {
        let ilength = 1.0 / length;
        out[0] = v[0] * ilength;
        out[1] = v[1] * ilength;
        out[2] = v[2] * ilength;
    } else {
        *out = VEC3_ORIGIN;
    }

    length
}

/// Raven call shape `VectorNormalize((float*)matrix[i])` — ghoul2 casts a
/// 4-wide `mdxaBone_t` row to `vec3_t` and normalizes its first three elements
/// in place. Named once here; not a distinct Raven function.
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp` bolt/skeleton call sites
pub fn VectorNormalizeRow(row: &mut [f32; 4]) -> vec_t {
    let v3: &mut vec3_t = (&mut row[..3]).try_into().unwrap();
    VectorNormalize(v3)
}

/// Raven call shape `DotProduct((float*)row, v)` — ghoul2 dots a 4-wide
/// `mdxaBone_t` matrix row against a `vec3_t`; the macro reads `[0..3]` only.
/// Named once here; not a distinct Raven function.
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp` skinning call sites
pub fn DotProductRow(row: &[f32; 4], v: vec3_t) -> vec_t {
    row[0] * v[0] + row[1] * v[1] + row[2] * v[2]
}

/// Raven `vectoyaw`.
///
/// Source: `oracle/codemp/game/bg_misc.c:1773-1792`
pub fn vectoyaw(vec: vec3_t) -> f32 {
    let mut yaw: f32;

    if vec[YAW] == 0.0 && vec[PITCH] == 0.0 {
        yaw = 0.0;
    } else {
        if vec[PITCH] != 0.0 {
            // Raven's atan2 is the double libm call and M_PI is math.h's double;
            // the `*180/M_PI` chain evaluates in f64 then rounds to the float result.
            yaw =
                ((vec[YAW] as f64).atan2(vec[PITCH] as f64) * 180.0 / std::f64::consts::PI) as f32;
        } else if vec[YAW] > 0.0 {
            yaw = 90.0;
        } else {
            yaw = 270.0;
        }
        if yaw < 0.0 {
            yaw += 360.0;
        }
    }

    yaw
}

/// Raven `VectorBetweenVectors`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:1607-1614`
// Raven returns int; C truthiness call shape maps to a Rust bool (§C7).
pub fn VectorBetweenVectors(v: vec3_t, v1: vec3_t, v2: vec3_t) -> bool {
    let dir1: vec3_t = [v[0] - v1[0], v[1] - v1[1], v[2] - v1[2]];
    let dir2: vec3_t = [v[0] - v2[0], v[1] - v2[1], v[2] - v2[2]];
    dir1[0] * dir2[0] + dir1[1] * dir2[1] + dir1[2] * dir2[2] <= 0.0
}

/// Raven `VectorNPos` — component-wise absolute value.
///
/// Source: `oracle/codemp/game/g_weapon.c:2636-2641`
pub fn VectorNPos(r#in: vec3_t, out: &mut vec3_t) {
    out[0] = if r#in[0] < 0.0 { -r#in[0] } else { r#in[0] };
    out[1] = if r#in[1] < 0.0 { -r#in[1] } else { r#in[1] };
    out[2] = if r#in[2] < 0.0 { -r#in[2] } else { r#in[2] };
}

/// Raven `VectorCompare2` — epsilon (0.0001) component compare.
///
/// Source: `oracle/codemp/game/w_saber.c:5275-5282`
// Raven returns int; C truthiness call shape maps to a Rust bool (§C7).
pub fn VectorCompare2(v1: vec3_t, v2: vec3_t) -> bool {
    !(v1[0] > v2[0] + 0.0001f32
        || v1[0] < v2[0] - 0.0001f32
        || v1[1] > v2[1] + 0.0001f32
        || v1[1] < v2[1] - 0.0001f32
        || v1[2] > v2[2] + 0.0001f32
        || v1[2] < v2[2] - 0.0001f32)
}

/// Raven `VectorAdvance` (`q_shared.h` macro) — lerp `a`→`b` by `s` into `c`.
///
/// Source: `oracle/codemp/game/q_shared.h:1370`
pub fn VectorAdvance(a: vec3_t, s: vec_t, b: vec3_t, c: &mut vec3_t) {
    c[0] = a[0] + s * (b[0] - a[0]);
    c[1] = a[1] + s * (b[1] - a[1]);
    c[2] = a[2] + s * (b[2] - a[2]);
}

/// Raven `_VectorMA`.
///
/// Source: `oracle/codemp/game/q_math.c:1214-1218`
pub fn _VectorMA(veca: vec3_t, scale: f32, vecb: vec3_t, vecc: &mut vec3_t) {
    vecc[0] = veca[0] + scale * vecb[0];
    vecc[1] = veca[1] + scale * vecb[1];
    vecc[2] = veca[2] + scale * vecb[2];
}

/// Raven `_DotProduct`.
///
/// Source: `oracle/codemp/game/q_math.c:1221-1223`
pub fn _DotProduct(v1: vec3_t, v2: vec3_t) -> vec_t {
    v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2]
}

/// Raven `_VectorSubtract`.
///
/// Source: `oracle/codemp/game/q_math.c:1225-1229`
pub fn _VectorSubtract(veca: vec3_t, vecb: vec3_t, out: &mut vec3_t) {
    out[0] = veca[0] - vecb[0];
    out[1] = veca[1] - vecb[1];
    out[2] = veca[2] - vecb[2];
}

/// Raven `_VectorAdd`.
///
/// Source: `oracle/codemp/game/q_math.c:1231-1235`
pub fn _VectorAdd(veca: vec3_t, vecb: vec3_t, out: &mut vec3_t) {
    out[0] = veca[0] + vecb[0];
    out[1] = veca[1] + vecb[1];
    out[2] = veca[2] + vecb[2];
}

/// Raven `_VectorCopy`.
///
/// Source: `oracle/codemp/game/q_math.c:1237-1241`
pub fn _VectorCopy(r#in: vec3_t, out: &mut vec3_t) {
    out[0] = r#in[0];
    out[1] = r#in[1];
    out[2] = r#in[2];
}

/// Raven `_VectorScale`.
///
/// Source: `oracle/codemp/game/q_math.c:1243-1247`
pub fn _VectorScale(r#in: vec3_t, scale: vec_t, out: &mut vec3_t) {
    out[0] = r#in[0] * scale;
    out[1] = r#in[1] * scale;
    out[2] = r#in[2] * scale;
}

/// Raven `VectorInverse`.
///
/// Source: `oracle/codemp/game/q_shared.h:1547-1550`
pub fn VectorInverse(v: &mut vec3_t) {
    v[0] = -v[0];
    v[1] = -v[1];
    v[2] = -v[2];
}

/// Raven `Vector4Scale`.
///
/// Source: `oracle/codemp/game/q_math.c:1249-1254`
pub fn Vector4Scale(r#in: vec4_t, scale: vec_t, out: &mut vec4_t) {
    out[0] = r#in[0] * scale;
    out[1] = r#in[1] * scale;
    out[2] = r#in[2] * scale;
    out[3] = r#in[3] * scale;
}

/// Raven `Q_log2`.
///
/// Source: `oracle/codemp/game/q_math.c:1257-1265`
pub fn Q_log2(val: c_int) -> c_int {
    let mut val = val;
    let mut answer = 0;
    loop {
        val >>= 1;
        if val == 0 {
            break;
        }
        answer += 1;
    }
    answer
}

/// Raven `MatrixMultiply`.
///
/// Source: `oracle/codemp/game/q_math.c:1293-1312`
pub fn MatrixMultiply(in1: &[[f32; 3]; 3], in2: &[[f32; 3]; 3], out: &mut [[f32; 3]; 3]) {
    out[0][0] = in1[0][0] * in2[0][0] + in1[0][1] * in2[1][0] + in1[0][2] * in2[2][0];
    out[0][1] = in1[0][0] * in2[0][1] + in1[0][1] * in2[1][1] + in1[0][2] * in2[2][1];
    out[0][2] = in1[0][0] * in2[0][2] + in1[0][1] * in2[1][2] + in1[0][2] * in2[2][2];
    out[1][0] = in1[1][0] * in2[0][0] + in1[1][1] * in2[1][0] + in1[1][2] * in2[2][0];
    out[1][1] = in1[1][0] * in2[0][1] + in1[1][1] * in2[1][1] + in1[1][2] * in2[2][1];
    out[1][2] = in1[1][0] * in2[0][2] + in1[1][1] * in2[1][2] + in1[1][2] * in2[2][2];
    out[2][0] = in1[2][0] * in2[0][0] + in1[2][1] * in2[1][0] + in1[2][2] * in2[2][0];
    out[2][1] = in1[2][0] * in2[0][1] + in1[2][1] * in2[1][1] + in1[2][2] * in2[2][1];
    out[2][2] = in1[2][0] * in2[0][2] + in1[2][1] * in2[1][2] + in1[2][2] * in2[2][2];
}

/// Raven `AngleVectors`.
///
/// Raven's `forward`/`right`/`up` are checked for null (`if (forward)`
/// pointer checks) — an optional out-param in Rust terms.
/// Raven's `static float sr, sp, sy, cr, cp, cy` are recomputed unconditionally
/// on every call before use (kept only for an old MS-compiler FP bug per the
/// oracle comment), so they carry no cross-call state; plain locals suffice.
/// Source: `oracle/codemp/game/q_math.c:1315-1348`
pub fn AngleVectors(
    angles: vec3_t,
    forward: Option<&mut vec3_t>,
    right: Option<&mut vec3_t>,
    up: Option<&mut vec3_t>,
) {
    // Raven: `angle = angles[..] * (M_PI*2 / 360)` with M_PI the double from
    // math.h; the constant and the sin/cos evaluate in f64, rounded to the
    // float `angle`/`s*`/`c*` locals. f32 trig diverges from the oracle.
    let angle = (angles[YAW] as f64 * (std::f64::consts::PI * 2.0 / 360.0)) as f32;
    let sy = (angle as f64).sin() as f32;
    let cy = (angle as f64).cos() as f32;
    let angle = (angles[PITCH] as f64 * (std::f64::consts::PI * 2.0 / 360.0)) as f32;
    let sp = (angle as f64).sin() as f32;
    let cp = (angle as f64).cos() as f32;
    let angle = (angles[ROLL] as f64 * (std::f64::consts::PI * 2.0 / 360.0)) as f32;
    let sr = (angle as f64).sin() as f32;
    let cr = (angle as f64).cos() as f32;

    if let Some(forward) = forward {
        forward[0] = cp * cy;
        forward[1] = cp * sy;
        forward[2] = -sp;
    }
    if let Some(right) = right {
        right[0] = -1.0 * sr * sp * cy + -1.0 * cr * -sy;
        right[1] = -1.0 * sr * sp * sy + -1.0 * cr * cy;
        right[2] = -1.0 * sr * cp;
    }
    if let Some(up) = up {
        up[0] = cr * sp * cy + -sr * -sy;
        up[1] = cr * sp * sy + -sr * cy;
        up[2] = cr * cp;
    }
}

/// Raven `NormalToLatLong`.
///
/// We use two byte encoded normals in some space critical applications. Lat =
/// 0 at (1,0,0) to 360 (-1,0,0), encoded in 8-bit sine table format. Lng = 0
/// at (0,0,1) to 180 (0,0,-1), encoded in 8-bit sine table format.
/// Source: `oracle/codemp/game/q_math.c:1394-1423`
pub fn NormalToLatLong(normal: vec3_t, bytes: *mut byte) {
    let bytes = unsafe { std::slice::from_raw_parts_mut(bytes, 2) };
    // check for singularities
    if normal[0] == 0.0 && normal[1] == 0.0 {
        if normal[2] > 0.0 {
            bytes[0] = 0;
            bytes[1] = 0; // lat = 0, long = 0
        } else {
            bytes[0] = 128;
            bytes[1] = 0; // lat = 0, long = 128
        }
    } else {
        // Raven: `a = (int)(RAD2DEG((vec_t)atan2(..)) * (255.0f/360.0f))`. atan2
        // is double, cast to float; RAD2DEG = `(a*180.0f)/M_PI` promotes to f64
        // (M_PI is math.h's double); `* (255/360 in f32)` stays f64.
        let scale = (255.0f32 / 360.0f32) as f64;
        let atan_f = (normal[1] as f64).atan2(normal[0] as f64) as f32;
        let a = (((atan_f * 180.0f32) as f64 / std::f64::consts::PI) * scale) as i32 & 0xff;
        let acos_f = (normal[2] as f64).acos() as f32;
        let b = (((acos_f * 180.0f32) as f64 / std::f64::consts::PI) * scale) as i32 & 0xff;

        bytes[0] = b as u8; // longitude
        bytes[1] = a as u8; // lattitude
    }
}

// RNG functions (Rand_Init/flrand/Q_flrand/irand/Q_irand) are ported as methods
// on bg_channel::rng::Rng (BgState.rng).
// Source: `oracle/codemp/game/q_math.c:1434-1474` → `bg_channel/rng.rs`

/// Raven `powf`.
///
/// Source: `oracle/codemp/game/q_math.c:1476-1482`
pub fn powf(x: f32, y: c_int) -> f32 {
    let mut r = x;
    let mut y = y - 1;
    while y > 0 {
        r *= r;
        y -= 1;
    }
    r
}

/// Raven `DotProductNormalize`.
///
/// Source: `oracle/codemp/game/q_math.c:1508-1516`
pub fn DotProductNormalize(inVec1: vec3_t, inVec2: vec3_t) -> f32 {
    let mut v1: vec3_t = [0.0; 3];
    let mut v2: vec3_t = [0.0; 3];
    VectorNormalize2(inVec1, &mut v1);
    VectorNormalize2(inVec2, &mut v2);
    _DotProduct(v1, v2)
}

/// Raven `G_FindClosestPointOnLineSegment`.
///
/// Source: `oracle/codemp/game/q_math.c:1524-1604`
pub fn G_FindClosestPointOnLineSegment(
    start: vec3_t,
    end: vec3_t,
    from: vec3_t,
    result: &mut vec3_t,
) -> qboolean {
    // Find the perpendicular vector to vec from start to end
    let mut vec_start2from: vec3_t = [0.0; 3];
    _VectorSubtract(from, start, &mut vec_start2from);
    let mut vec_start2end: vec3_t = [0.0; 3];
    _VectorSubtract(end, start, &mut vec_start2end);

    let mut dot = DotProductNormalize(vec_start2from, vec_start2end);

    if dot <= 0.0 {
        // The perpendicular would be beyond or through the start point
        *result = start;
        return Q_FALSE;
    }

    if dot == 1.0 {
        // parallel, closer of 2 points will be the target
        if VectorLengthSquared(vec_start2from) < VectorLengthSquared(vec_start2end) {
            *result = from;
        } else {
            *result = end;
        }
        return Q_FALSE;
    }

    // Try other end
    let mut vec_end2from: vec3_t = [0.0; 3];
    _VectorSubtract(from, end, &mut vec_end2from);
    let mut vec_end2start: vec3_t = [0.0; 3];
    _VectorSubtract(start, end, &mut vec_end2start);

    dot = DotProductNormalize(vec_end2from, vec_end2start);

    if dot <= 0.0 {
        // The perpendicular would be beyond or through the start point
        *result = end;
        return Q_FALSE;
    }

    if dot == 1.0 {
        // parallel, closer of 2 points will be the target
        if VectorLengthSquared(vec_end2from) < VectorLengthSquared(vec_end2start) {
            *result = from;
        } else {
            *result = end;
        }
        return Q_FALSE;
    }

    // cos(theta) = b / c; solve for b: b = cos(theta) * c
    // angle between vecs end2from and end2start, should be between 0 and 90
    let theta = 90.0 * (1.0 - dot);

    // Get length of side from End2Result using sine of theta
    let dist_end2from = VectorLength(vec_end2from); // c
    let cos_theta = ((theta as f64 * std::f64::consts::PI / 180.0).cos()) as f32; // cos(DEG2RAD(theta)); Raven double libm
    let dist_end2result = cos_theta * dist_end2from; // b

    // Extrapolate to find result
    VectorNormalize(&mut vec_end2start);
    _VectorMA(end, dist_end2result, vec_end2start, result);

    // perpendicular intersection is between the 2 endpoints
    Q_TRUE
}

/// Raven `G_PointDistFromLineSegment`.
///
/// Source: `oracle/codemp/game/q_math.c:1606-1670`
pub fn G_PointDistFromLineSegment(start: vec3_t, end: vec3_t, from: vec3_t) -> f32 {
    // Find the perpendicular vector to vec from start to end
    let mut vec_start2from: vec3_t = [0.0; 3];
    _VectorSubtract(from, start, &mut vec_start2from);
    let mut vec_start2end: vec3_t = [0.0; 3];
    _VectorSubtract(end, start, &mut vec_start2end);
    let mut vec_end2from: vec3_t = [0.0; 3];
    _VectorSubtract(from, end, &mut vec_end2from);
    let mut vec_end2start: vec3_t = [0.0; 3];
    _VectorSubtract(start, end, &mut vec_end2start);

    let mut dot = DotProductNormalize(vec_start2from, vec_start2end);

    let dist_start2from = Distance(start, from);
    let dist_end2from = Distance(end, from);

    if dot <= 0.0 {
        // The perpendicular would be beyond or through the start point
        return dist_start2from;
    }

    if dot == 1.0 {
        // parallel, closer of 2 points will be the target
        return if dist_start2from < dist_end2from {
            dist_start2from
        } else {
            dist_end2from
        };
    }

    // Try other end
    dot = DotProductNormalize(vec_end2from, vec_end2start);

    if dot <= 0.0 {
        // The perpendicular would be beyond or through the end point
        return dist_end2from;
    }

    if dot == 1.0 {
        // parallel, closer of 2 points will be the target
        return if dist_start2from < dist_end2from {
            dist_start2from
        } else {
            dist_end2from
        };
    }

    // angle between vecs end2from and end2start, should be between 0 and 90
    let theta = 90.0 * (1.0 - dot);

    // Get length of side from End2Result using sine of theta
    let cos_theta = ((theta as f64 * std::f64::consts::PI / 180.0).cos()) as f32; // cos(DEG2RAD(theta)); Raven double libm
    let dist_end2result = cos_theta * dist_end2from; // b

    // Extrapolate to find result
    VectorNormalize(&mut vec_end2start);
    let mut intersection: vec3_t = [0.0; 3];
    _VectorMA(end, dist_end2result, vec_end2start, &mut intersection);

    // perpendicular intersection is between the 2 endpoints, return dist to it from `from`
    Distance(intersection, from)
}
