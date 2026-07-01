//! Shared Raven vector aliases from `q_shared.h`.
//!
//! Source: `oracle/oracle/code/game/q_shared.h:314-320`
//! Source: `oracle/oracle/codemp/game/q_shared.h:530-537`

#![allow(non_camel_case_types)]

use core::ffi::{c_float, c_int};

pub type vec_t = c_float;
pub type vec2_t = [vec_t; 2];
pub type vec3_t = [vec_t; 3];
pub type vec4_t = [vec_t; 4];
pub type vec5_t = [vec_t; 5];

// MP Raven comment: rwwRMG - new vec types
pub type vec3pair_t = [vec3_t; 2];

// Integer vectors.
// Source: `oracle/oracle/code/game/q_shared.h:323-325`
// Source: `oracle/oracle/codemp/game/q_shared.h:539-541`
// Note: `ivec2_t` is SP-only (diverges) and lives per-mode in SP qshared, not here.
pub type ivec3_t = [c_int; 3];
pub type ivec4_t = [c_int; 4];
pub type ivec5_t = [c_int; 5];

// Fixed-point scalars.
// Source: `oracle/oracle/code/game/q_shared.h:327-329`
// Source: `oracle/oracle/codemp/game/q_shared.h:543-545`
pub type fixed4_t = c_int;
pub type fixed8_t = c_int;
pub type fixed16_t = c_int;
