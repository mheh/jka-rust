#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `cLeaf_t` — a leaf node's collision-model payload (BSP leaf brush/surface ranges).
///
/// Type definition source: `oracle/codemp/qcommon/cm_local.h:34-43`
#[repr(C)]
pub struct cLeaf_t {
    pub cluster: c_int,
    pub area: c_int,

    pub firstLeafBrush: c_int,
    pub numLeafBrushes: c_int,

    pub firstLeafSurface: c_int,
    pub numLeafSurfaces: c_int,
}

const _: () = assert!(core::mem::size_of::<cLeaf_t>() == 24);
const _: () = assert!(core::mem::offset_of!(cLeaf_t, cluster) == 0);
const _: () = assert!(core::mem::offset_of!(cLeaf_t, area) == 4);
const _: () = assert!(core::mem::offset_of!(cLeaf_t, firstLeafBrush) == 8);
const _: () = assert!(core::mem::offset_of!(cLeaf_t, numLeafBrushes) == 12);
const _: () = assert!(core::mem::offset_of!(cLeaf_t, firstLeafSurface) == 16);
const _: () = assert!(core::mem::offset_of!(cLeaf_t, numLeafSurfaces) == 20);
