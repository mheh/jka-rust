//! Shared Raven vector aliases from `q_shared.h`.
//!
//! Source: `oracle/oracle/code/game/q_shared.h:314-320`
//! Source: `oracle/oracle/codemp/game/q_shared.h:530-537`

#![allow(non_camel_case_types)]

use core::ffi::c_float;

pub type vec_t = c_float;
pub type vec2_t = [vec_t; 2];
pub type vec3_t = [vec_t; 3];
pub type vec4_t = [vec_t; 4];
pub type vec5_t = [vec_t; 5];

// MP Raven comment: rwwRMG - new vec types
pub type vec3pair_t = [vec3_t; 2];
