#![allow(non_camel_case_types, non_snake_case)]

/// Raven `MD3_IDENT`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:92`
pub const MD3_IDENT: i32 = ('3' as i32) << 24 | ('P' as i32) << 16 | ('D' as i32) << 8 | 'I' as i32;

/// Raven `MD3_VERSION`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:93`
pub const MD3_VERSION: i32 = 15;

/// Raven `MD3_MAX_LODS`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:96`
pub const MD3_MAX_LODS: usize = 3;

/// Raven `MD3_MAX_TRIANGLES` — per surface.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:97`
pub const MD3_MAX_TRIANGLES: usize = 8192;

/// Raven `MD3_MAX_VERTS` — per surface.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:98`
pub const MD3_MAX_VERTS: usize = 4096;

/// Raven `MD3_MAX_SHADERS` — per surface.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:99`
pub const MD3_MAX_SHADERS: usize = 256;

/// Raven `MD3_MAX_FRAMES` — per model.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:100`
pub const MD3_MAX_FRAMES: usize = 1024;

/// Raven `MD3_MAX_SURFACES` — per model.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:101`
pub const MD3_MAX_SURFACES: usize = 32 + 32;

/// Raven `MD3_MAX_TAGS` — per frame.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:102`
pub const MD3_MAX_TAGS: usize = 16;

/// Raven `MD3_XYZ_SCALE`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:105`
pub const MD3_XYZ_SCALE: f32 = 1.0 / 64.0;
