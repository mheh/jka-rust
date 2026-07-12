#![allow(non_camel_case_types, non_snake_case)]

// Raven allocates extra brush/side/leaf/plane slots so the box (and capsule) trace
// shapes can be treated as ordinary brush models.

/// Raven `BOX_BRUSHES` — extra brush slots reserved for the box trace shape.
/// Source: `oracle/codemp/qcommon/cm_load.cpp:29`
pub const BOX_BRUSHES: usize = 1;

/// Raven `BOX_SIDES` — extra brush-side slots reserved for the box trace shape.
/// Source: `oracle/codemp/qcommon/cm_load.cpp:30`
pub const BOX_SIDES: usize = 6;

/// Raven `BOX_LEAFS` — extra leaf slots reserved for the box trace shape.
/// Source: `oracle/codemp/qcommon/cm_load.cpp:31`
pub const BOX_LEAFS: usize = 2;

/// Raven `BOX_PLANES` — extra plane slots reserved for the box trace shape.
/// Source: `oracle/codemp/qcommon/cm_load.cpp:32`
pub const BOX_PLANES: usize = 12;

/// Raven `VIS_HEADER` — leading dword count (cluster count, bytes-per-cluster) in
/// the BSP visibility lump.
/// Source: `oracle/codemp/qcommon/cm_load.cpp:453`
pub const VIS_HEADER: usize = 8;

/// Raven `MAX_PATCH_VERTS` — max control-mesh vertices for one loaded bezier patch.
/// Source: `oracle/codemp/qcommon/cm_load.cpp:482`
pub const MAX_PATCH_VERTS: usize = 1024;
