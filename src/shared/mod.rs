//! Globally-shared wire & math types (Raven `q_shared.h` scope): vec3, entityState, playerState, trace, usercmd.

#![allow(non_camel_case_types)]

use core::ffi::c_int;

pub mod collision;
pub mod cvar;
#[path = "fsMode_t.rs"]
pub mod file_mode;
pub mod platform;
pub mod vector;

pub use collision::{cplane_t, CollisionRecord_t};
pub use cvar::{cvarHandle_t, vmCvar_t, MAX_CVAR_VALUE_STRING};
pub use file_mode::{fsMode_t, FS_APPEND, FS_APPEND_SYNC, FS_READ, FS_WRITE};
pub use vector::{vec2_t, vec3_t, vec3pair_t, vec4_t, vec5_t, vec_t};

/// Raven `qboolean`.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h`
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h`
pub type qboolean = c_int;

/// Raven `fileHandle_t`.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:187`
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:362`
pub type fileHandle_t = c_int;

/// Raven `qhandle_t`.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:183`
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:358`
pub type qhandle_t = c_int;

/// Raven `mdxaBone_t`.
///
/// Type definition source: `oracle/oracle/code/renderer/mdx_format.h:137`
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:3078`
/// Type definition source: `oracle/oracle/codemp/renderer/mdx_format.h:137`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct mdxaBone_t {
    pub matrix: [[f32; 4]; 3],
}

pub const QFALSE: qboolean = 0;
pub const QTRUE: qboolean = 1;
