#![allow(non_camel_case_types, non_snake_case)]

/// Raven `DRAWVERT_LIGHTMAP_SCALE`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:324`
pub const DRAWVERT_LIGHTMAP_SCALE: f32 = 32768.0;

/// Raven `DRAWVERT_ST_SCALE`.
///
/// Raven: change texture coordinates for TriSurfs to be even more fine
/// grain. See `GRID_DRAWVERT_ST_SCALE` for a note about keeping MIN_ST and
/// MAX_ST up to date with ST_SCALE.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:329`
pub const DRAWVERT_ST_SCALE: f32 = 2048.0;

/// Raven `GRID_DRAWVERT_ST_SCALE`.
///
/// Raven: we use a slightly different format for the fixed point texture
/// coords in Grid/Mesh drawverts: 10.6 rather than 12.4. To be sure that
/// this is ok, keep the max and min values equal to the largest and
/// smallest whole numbers that can be stored using the format (don't change
/// this without changing `DRAWVERT_ST_SCALE` and its min/max too).
///
/// Source: `oracle/codemp/qcommon/qfiles.h:337`
pub const GRID_DRAWVERT_ST_SCALE: f32 = 64.0;
