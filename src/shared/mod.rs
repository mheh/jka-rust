//! Globally-shared wire & math types (Raven `q_shared.h` scope): vec3, entityState, playerState, trace, usercmd.

#![allow(non_camel_case_types)]

use core::ffi::c_int;

pub mod collision;
pub mod platform;
pub mod vector;

pub use collision::{cplane_t, CollisionRecord_t};
pub use vector::{vec2_t, vec3_t, vec3pair_t, vec4_t, vec5_t, vec_t};

/// Raven `qboolean`.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h`
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h`
pub type qboolean = c_int;

pub const QFALSE: qboolean = 0;
pub const QTRUE: qboolean = 1;
