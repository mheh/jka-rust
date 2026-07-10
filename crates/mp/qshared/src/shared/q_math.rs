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
