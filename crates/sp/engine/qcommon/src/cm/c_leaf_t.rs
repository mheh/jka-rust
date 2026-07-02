#![allow(non_camel_case_types, non_snake_case)]

/// Raven `cLeaf_t` — a BSP leaf's brush/surface ranges and cluster/area membership.
///
/// Type definition source: `oracle/oracle/code/qcommon/cm_local.h:31-40`
#[repr(C)]
pub struct cLeaf_t {
    pub cluster: i32,
    pub area: i32,

    pub firstLeafBrush: i32,
    pub numLeafBrushes: i32,

    pub firstLeafSurface: i32,
    pub numLeafSurfaces: i32,
}

const _: () = assert!(core::mem::size_of::<cLeaf_t>() == 24);
const _: () = assert!(core::mem::offset_of!(cLeaf_t, cluster) == 0);
const _: () = assert!(core::mem::offset_of!(cLeaf_t, area) == 4);
const _: () = assert!(core::mem::offset_of!(cLeaf_t, firstLeafBrush) == 8);
const _: () = assert!(core::mem::offset_of!(cLeaf_t, numLeafBrushes) == 12);
const _: () = assert!(core::mem::offset_of!(cLeaf_t, firstLeafSurface) == 16);
const _: () = assert!(core::mem::offset_of!(cLeaf_t, numLeafSurfaces) == 20);
