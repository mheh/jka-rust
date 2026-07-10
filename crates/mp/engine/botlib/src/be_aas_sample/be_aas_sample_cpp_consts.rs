//! `be_aas_sample.cpp`-local AAS environment-sampling constants.
//!
//! Source: `oracle/codemp/botlib/be_aas_sample.cpp:34-38`

/// Raven `BBOX_NORMAL_EPSILON`.
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:34`
pub const BBOX_NORMAL_EPSILON: f32 = 0.001;

/// Raven `ON_EPSILON` — Raven left this at `0` (originally `0.0005`, commented out).
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:36`
pub const ON_EPSILON: f32 = 0.0;

/// Raven `TRACEPLANE_EPSILON`.
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:38`
pub const TRACEPLANE_EPSILON: f32 = 0.125;
