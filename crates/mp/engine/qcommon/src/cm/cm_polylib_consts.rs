#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `MAX_POINTS_ON_WINDING`.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.h:10`
pub const MAX_POINTS_ON_WINDING: c_int = 64;

/// Raven `SIDE_FRONT`.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.h:12`
pub const SIDE_FRONT: c_int = 0;

/// Raven `SIDE_BACK`.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.h:13`
pub const SIDE_BACK: c_int = 1;

/// Raven `SIDE_ON`.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.h:14`
pub const SIDE_ON: c_int = 2;

/// Raven `SIDE_CROSS`.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.h:15`
pub const SIDE_CROSS: c_int = 3;

/// Raven `CLIP_EPSILON`.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.h:17`
pub const CLIP_EPSILON: f32 = 0.1;

/// Raven `MAX_MAP_BOUNDS`.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.h:19`
pub const MAX_MAP_BOUNDS: c_int = 65535;

/// Raven `ON_EPSILON` — guarded by `#ifndef ON_EPSILON` in the oracle; no
/// build in this tree defines it externally, so this is the effective value.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.h:23`
pub const ON_EPSILON: f32 = 0.1;

/// Raven `MAX_HULL_POINTS` — max points scanned per convex-hull merge step in
/// `AddWindingToConvexHull`.
///
/// Source: `oracle/codemp/qcommon/cm_polylib.cpp:625`
pub const MAX_HULL_POINTS: c_int = 128;
