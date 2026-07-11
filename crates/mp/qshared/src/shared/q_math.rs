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

/// Raven `_VectorCopy`.
///
/// Source: `oracle/codemp/game/q_math.c:1237-1241`
pub fn _VectorCopy(r#in: vec3_t, out: &mut vec3_t) {
    out[0] = r#in[0];
    out[1] = r#in[1];
    out[2] = r#in[2];
}

/// Raven angle-vector indices (`PITCH`/`YAW`/`ROLL`).
///
/// Source: `oracle/codemp/game/q_shared.h:521-524`
pub const PITCH: usize = 0;
pub const YAW: usize = 1;
pub const ROLL: usize = 2;

/// Raven `_VectorAdd`.
///
/// Source: `oracle/codemp/game/q_math.c:1231-1235`
pub fn _VectorAdd(veca: vec3_t, vecb: vec3_t, out: &mut vec3_t) {
    out[0] = veca[0] + vecb[0];
    out[1] = veca[1] + vecb[1];
    out[2] = veca[2] + vecb[2];
}

/// Raven `_VectorScale`.
///
/// Source: `oracle/codemp/game/q_math.c:1243-1247`
pub fn _VectorScale(r#in: vec3_t, scale: vec_t, out: &mut vec3_t) {
    out[0] = r#in[0] * scale;
    out[1] = r#in[1] * scale;
    out[2] = r#in[2] * scale;
}

/// Raven `_VectorMA`.
///
/// Source: `oracle/codemp/game/q_math.c:1214-1218`
pub fn _VectorMA(veca: vec3_t, scale: f32, vecb: vec3_t, vecc: &mut vec3_t) {
    vecc[0] = veca[0] + scale * vecb[0];
    vecc[1] = veca[1] + scale * vecb[1];
    vecc[2] = veca[2] + scale * vecb[2];
}

/// Raven `CrossProduct` (header-inline helper).
///
/// Source: `oracle/codemp/game/q_shared.h:1553-1557`
pub fn CrossProduct(v1: vec3_t, v2: vec3_t, cross: &mut vec3_t) {
    cross[0] = v1[1] * v2[2] - v1[2] * v2[1];
    cross[1] = v1[2] * v2[0] - v1[0] * v2[2];
    cross[2] = v1[0] * v2[1] - v1[1] * v2[0];
}

/// Raven `VectorLength` (header-inline helper). `sqrt` computed in f64 (double
/// libm rounded to float) to match the oracle; an f32 sqrt double-rounds.
///
/// Source: `oracle/codemp/game/q_shared.h:1460-1489`
pub fn VectorLength(v: vec3_t) -> vec_t {
    ((v[0] * v[0] + v[1] * v[1] + v[2] * v[2]) as f64).sqrt() as f32
}

/// Raven `VectorLengthSquared` (header-inline helper).
///
/// Source: `oracle/codemp/game/q_shared.h:1491-1518`
pub fn VectorLengthSquared(v: vec3_t) -> vec_t {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
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
        *out = [0.0, 0.0, 0.0]; // Raven `VEC3_ORIGIN`
    }

    length
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

/// Raven `VectorInverse`.
///
/// Source: `oracle/codemp/game/q_shared.h:1547-1550`
pub fn VectorInverse(v: &mut vec3_t) {
    v[0] = -v[0];
    v[1] = -v[1];
    v[2] = -v[2];
}

/// Raven `VectorClear` (`q_shared.h` macro).
///
/// Source: `oracle/codemp/game/q_shared.h:1397`
pub fn VectorClear(a: &mut vec3_t) {
    a[0] = 0.0;
    a[1] = 0.0;
    a[2] = 0.0;
}

/// Raven `vec3_origin`.
///
/// Source: `oracle/codemp/game/q_math.c:7`
#[allow(non_upper_case_globals)]
pub const vec3_origin: vec3_t = [0.0, 0.0, 0.0];

/// Raven `VectorCompare` (header-inline helper). Returns 1 if vectors are
/// equal, 0 otherwise.
///
/// Source: `oracle/codemp/game/q_shared.h:1527-1532`
pub fn VectorCompare(v1: vec3_t, v2: vec3_t) -> i32 {
    if v1[0] != v2[0] || v1[1] != v2[1] || v1[2] != v2[2] {
        return 0;
    }
    1
}

/// Raven `VectorSet` (`q_shared.h` macro). Out-param, matching the macro's
/// `(v)[0]=(x),…` write-through shape.
///
/// Source: `oracle/codemp/game/q_shared.h:1399`
pub fn VectorSet(v: &mut vec3_t, x: f32, y: f32, z: f32) {
    v[0] = x;
    v[1] = y;
    v[2] = z;
}

/// Raven `VectorAdvance` (`q_shared.h` macro) — lerp `a`→`b` by `s` into `c`.
///
/// Source: `oracle/codemp/game/q_shared.h:1370`
pub fn VectorAdvance(a: vec3_t, s: vec_t, b: vec3_t, c: &mut vec3_t) {
    c[0] = a[0] + s * (b[0] - a[0]);
    c[1] = a[1] + s * (b[1] - a[1]);
    c[2] = a[2] + s * (b[2] - a[2]);
}

/// Raven `Square` (`q_shared.h` macro).
///
/// Source: `oracle/codemp/game/q_shared.h:3005`
pub fn Square(x: vec_t) -> vec_t {
    x * x
}

/// Raven `AngleVectors`. Raven's `static float sr,sp,…` are recomputed every
/// call before use (kept only for an old MS-compiler FP bug per the oracle
/// comment), so they carry no cross-call state; plain locals suffice.
///
/// Source: `oracle/codemp/game/q_math.c:1315-1348`
pub fn AngleVectors(
    angles: vec3_t,
    forward: Option<&mut vec3_t>,
    right: Option<&mut vec3_t>,
    up: Option<&mut vec3_t>,
) {
    // Raven: `angle = angles[..] * (M_PI*2 / 360)` with `M_PI` the double from
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

/// Raven `ProjectPointOnPlane`.
///
/// Source: `oracle/codemp/game/q_math.c:556-577`
pub fn ProjectPointOnPlane(dst: &mut vec3_t, p: vec3_t, normal: vec3_t) {
    let mut inv_denom = _DotProduct(normal, normal);
    // Raven's debug assert (`Q_fabs(inv_denom) != 0.0f`) catches a zero
    // normal; that's a caller bug (division by zero), not something to
    // silently normalize away.
    debug_assert!(inv_denom.abs() != 0.0);
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

/// Raven `PerpendicularVector`.
///
/// Assumes "src" is normalized.
/// Source: `oracle/codemp/game/q_math.c:1353-1383`
pub fn PerpendicularVector(dst: &mut vec3_t, src: vec3_t) {
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
