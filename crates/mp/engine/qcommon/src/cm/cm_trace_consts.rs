#![allow(non_camel_case_types, non_snake_case)]

/// Raven `MAX_POSITION_LEAFS` — max leafs collected for a position/leaf test.
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:447`
pub const MAX_POSITION_LEAFS: usize = 1024;

/// Raven `RADIUS_EPSILON` — bias added to a sphere/capsule radius during
/// trace-bounds and clip checks.
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:1049`
pub const RADIUS_EPSILON: f32 = 1.0;
