#![allow(non_camel_case_types, non_snake_case)]

//! Raven's shader surface-geometry limits — "surface geometry should not
//! exceed these limits" (`qfiles.h`).

/// Raven `SHADER_MAX_VERTEXES`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:10`
pub const SHADER_MAX_VERTEXES: usize = 1000;

/// Raven `SHADER_MAX_INDEXES`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:11`
pub const SHADER_MAX_INDEXES: usize = 6 * SHADER_MAX_VERTEXES;
