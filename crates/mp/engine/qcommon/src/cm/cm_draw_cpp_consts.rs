#![allow(non_camel_case_types, non_snake_case)]

//! `cm_draw.cpp`-local constants (clip codes and the scan-converter fixed-point shift).
//!
//! Source: `oracle/codemp/qcommon/cm_draw.cpp:190-193,1078`

use core::ffi::c_long;

/// Raven `LEFT` — the point sits left of the clip box.
/// Source: `oracle/codemp/qcommon/cm_draw.cpp:190`
pub const LEFT: c_long = 1;

/// Raven `RIGHT` — the point sits right of the clip box.
/// Source: `oracle/codemp/qcommon/cm_draw.cpp:191`
pub const RIGHT: c_long = 2;

/// Raven `TOP` — the point sits above the clip box.
/// Source: `oracle/codemp/qcommon/cm_draw.cpp:192`
pub const TOP: c_long = 4;

/// Raven `BOTTOM` — the point sits below the clip box.
/// Source: `oracle/codemp/qcommon/cm_draw.cpp:193`
pub const BOTTOM: c_long = 8;

/// Raven `INT_SHIFT` — the fixed-point fraction width of the active edge list.
/// Source: `oracle/codemp/qcommon/cm_draw.cpp:1078`
pub const INT_SHIFT: c_long = 13;
