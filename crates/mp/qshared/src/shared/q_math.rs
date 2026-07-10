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
