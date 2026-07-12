#![allow(non_camel_case_types, non_snake_case)]

//! `cm_patch.h` patch-collision constants.
//!
//! These five names also appear textually inside a large `/* ... */`
//! documentation block at `cm_patch.cpp:44-83` (dead — never compiled); the
//! real, active `#define`s compiled into the build live in `cm_patch.h`
//! (values identical), which is what's cited below.
//!
//! Source: `oracle/codemp/qcommon/cm_patch.h:42-43,119-121`

/// Raven `MAX_FACETS`.
/// Source: `oracle/codemp/qcommon/cm_patch.h:42`
pub const MAX_FACETS: usize = 1024;

/// Raven `MAX_PATCH_PLANES`.
/// Source: `oracle/codemp/qcommon/cm_patch.h:43`
pub const MAX_PATCH_PLANES: usize = 2048;

/// Raven `SUBDIVIDE_DISTANCE` — never more than this units away from curve.
/// Source: `oracle/codemp/qcommon/cm_patch.h:119`
pub const SUBDIVIDE_DISTANCE: f32 = 16.0;

/// Raven `PLANE_TRI_EPSILON`.
/// Source: `oracle/codemp/qcommon/cm_patch.h:120`
pub const PLANE_TRI_EPSILON: f32 = 0.1;

/// Raven `WRAP_POINT_EPSILON`.
/// Source: `oracle/codemp/qcommon/cm_patch.h:121`
pub const WRAP_POINT_EPSILON: f32 = 0.1;
