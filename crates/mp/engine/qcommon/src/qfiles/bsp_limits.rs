#![allow(non_camel_case_types, non_snake_case)]

/// Raven `BSP_IDENT`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:201`
pub const BSP_IDENT: i32 = ('P' as i32) << 24 | ('S' as i32) << 16 | ('B' as i32) << 8 | 'R' as i32;

/// Raven `BSP_VERSION`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:203`
pub const BSP_VERSION: i32 = 1;

/// Raven `MAX_MAP_MODELS`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:208`
pub const MAX_MAP_MODELS: usize = 0x400;

/// Raven `MAX_MAP_BRUSHES`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:209`
pub const MAX_MAP_BRUSHES: usize = 0x8000;

/// Raven `MAX_MAP_ENTITIES`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:210`
pub const MAX_MAP_ENTITIES: usize = 0x800;

/// Raven `MAX_MAP_ENTSTRING`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:211`
pub const MAX_MAP_ENTSTRING: usize = 0x40000;

/// Raven `MAX_MAP_SHADERS`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:212`
pub const MAX_MAP_SHADERS: usize = 0x400;

/// Raven `MAX_MAP_AREAS` — must match `MAX_MAP_AREA_BYTES` in `q_shared.h`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:214`
pub const MAX_MAP_AREAS: usize = 0x100;

/// Raven `MAX_MAP_FOGS`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:215`
pub const MAX_MAP_FOGS: usize = 0x100;

/// Raven `MAX_MAP_PLANES`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:216`
pub const MAX_MAP_PLANES: usize = 0x20000;

/// Raven `MAX_MAP_NODES`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:217`
pub const MAX_MAP_NODES: usize = 0x20000;

/// Raven `MAX_MAP_BRUSHSIDES`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:218`
pub const MAX_MAP_BRUSHSIDES: usize = 0x20000;

/// Raven `MAX_MAP_LEAFS`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:219`
pub const MAX_MAP_LEAFS: usize = 0x20000;

/// Raven `MAX_MAP_LEAFFACES`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:220`
pub const MAX_MAP_LEAFFACES: usize = 0x20000;

/// Raven `MAX_MAP_LEAFBRUSHES`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:221`
pub const MAX_MAP_LEAFBRUSHES: usize = 0x40000;

/// Raven `MAX_MAP_PORTALS`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:222`
pub const MAX_MAP_PORTALS: usize = 0x20000;

/// Raven `MAX_MAP_LIGHTING`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:223`
pub const MAX_MAP_LIGHTING: usize = 0x800000;

/// Raven `MAX_MAP_LIGHTGRID`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:224`
pub const MAX_MAP_LIGHTGRID: usize = 65535;

/// Raven `MAX_MAP_LIGHTGRID_ARRAY`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:225`
pub const MAX_MAP_LIGHTGRID_ARRAY: usize = 0x100000;

/// Raven `MAX_MAP_VISIBILITY`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:226`
pub const MAX_MAP_VISIBILITY: usize = 0x600000;

/// Raven `MAX_MAP_DRAW_SURFS`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:228`
pub const MAX_MAP_DRAW_SURFS: usize = 0x20000;

/// Raven `MAX_MAP_DRAW_VERTS`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:229`
pub const MAX_MAP_DRAW_VERTS: usize = 0x80000;

/// Raven `MAX_MAP_DRAW_INDEXES`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:230`
pub const MAX_MAP_DRAW_INDEXES: usize = 0x80000;

/// Raven `MAX_KEY`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:234`
pub const MAX_KEY: usize = 32;

/// Raven `MAX_VALUE`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:235`
pub const MAX_VALUE: usize = 1024;

// Raven: the editor uses these predefined yaw angles to orient entities up
// or down.

/// Raven `ANGLE_UP`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:238`
pub const ANGLE_UP: i32 = -1;

/// Raven `ANGLE_DOWN`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:239`
pub const ANGLE_DOWN: i32 = -2;

/// Raven `LIGHTMAP_WIDTH`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:241`
pub const LIGHTMAP_WIDTH: usize = 128;

/// Raven `LIGHTMAP_HEIGHT`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:242`
pub const LIGHTMAP_HEIGHT: usize = 128;
