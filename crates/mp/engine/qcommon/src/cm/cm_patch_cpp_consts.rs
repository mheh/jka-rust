#![allow(non_camel_case_types, non_snake_case)]

//! `cm_patch.cpp`-local constants (patch-collision epsilons).
//!
//! Source: `oracle/codemp/qcommon/cm_patch.cpp:369,437-438`

/// Raven `POINT_EPSILON` — used by `CM_ComparePoints`.
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:369`
pub const POINT_EPSILON: f32 = 0.1;

/// Raven `NORMAL_EPSILON`.
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:437`
pub const NORMAL_EPSILON: f32 = 0.0001;

/// Raven `DIST_EPSILON`.
/// Source: `oracle/codemp/qcommon/cm_patch.cpp:438`
pub const DIST_EPSILON: f32 = 0.02;
