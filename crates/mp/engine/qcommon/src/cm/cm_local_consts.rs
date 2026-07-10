#![allow(non_camel_case_types, non_snake_case)]

/// Raven `MAX_SUBMODELS` — maximum inline (brush) submodels in a clip map.
/// Source: oracle/codemp/qcommon/cm_local.h:12
pub const MAX_SUBMODELS: usize = 512;

/// Raven `BOX_MODEL_HANDLE` — synthetic clip-handle used for the box-hull trace shape.
/// Source: oracle/codemp/qcommon/cm_local.h:13
pub const BOX_MODEL_HANDLE: usize = MAX_SUBMODELS - 1;

/// Raven `CAPSULE_MODEL_HANDLE` — synthetic clip-handle used for the capsule trace shape.
/// Source: oracle/codemp/qcommon/cm_local.h:14
pub const CAPSULE_MODEL_HANDLE: usize = MAX_SUBMODELS - 2;

/// Raven `SURFACE_CLIP_EPSILON` — bias kept off a surface after a clip to avoid re-hitting it.
/// Source: oracle/codemp/qcommon/cm_local.h:218
pub const SURFACE_CLIP_EPSILON: f32 = 0.125;
