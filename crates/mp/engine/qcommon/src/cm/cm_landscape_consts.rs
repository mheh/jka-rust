#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

// Raven `HEIGHT_RESOLUTION` is already ported for the renderer-side
// `CTRLandScape` at `crates/mp/renderer/src/tr_landscape/ctrland_scape.rs`.
// Source: `oracle/codemp/qcommon/cm_landscape.h:13`

/// Raven `TERRAIN_STEP_MAGIC` — average of 1 side and the diagonal presuming a
/// square patch; used as the optimal step through the patches.
///
/// Source: `oracle/codemp/qcommon/cm_landscape.h:17`
pub const TERRAIN_STEP_MAGIC: f32 = 1.0 / 1.2071;

/// Raven `MIN_TERXELS`.
///
/// Source: `oracle/codemp/qcommon/cm_landscape.h:19`
pub const MIN_TERXELS: c_int = 2;

/// Raven `MAX_TERXELS`.
///
/// Source: `oracle/codemp/qcommon/cm_landscape.h:20`
pub const MAX_TERXELS: c_int = 8;

/// Raven `MAX_VARIANCE_SIZE` — defined as `1 << (sqrt(MAX_TERXELS) + 1)`.
///
/// Source: `oracle/codemp/qcommon/cm_landscape.h:22`
pub const MAX_VARIANCE_SIZE: c_int = 16;

/// Raven `MAX_INSTANCE_TYPES` — max number of instances to pick from an
/// instance file.
///
/// Source: `oracle/codemp/qcommon/cm_landscape.h:25`
pub const MAX_INSTANCE_TYPES: c_int = 16;
