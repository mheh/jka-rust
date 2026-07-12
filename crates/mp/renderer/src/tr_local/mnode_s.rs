#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_qshared::shared::{cplane_t, vec3_t};

use super::msurface_s::msurface_t;

/// Raven `mnode_s` (typedef `mnode_t`) — a BSP tree node/leaf.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:917-934`
#[repr(C)]
pub struct mnode_t {
    // common with leaf and node
    /// -1 for nodes, to differentiate from leafs
    pub contents: c_int,
    /// node needs to be traversed if current
    pub visframe: c_int,
    /// for bounding box culling
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub parent: *mut mnode_t,

    // node specific
    pub plane: *mut cplane_t,
    pub children: [*mut mnode_t; 2],

    // leaf specific
    pub cluster: c_int,
    pub area: c_int,

    pub firstmarksurface: *mut *mut msurface_t,
    pub nummarksurfaces: c_int,
}

pub type mnode_s = mnode_t;

const _: () = assert!(core::mem::offset_of!(mnode_t, contents) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<mnode_t>() == 88);
    assert!(core::mem::offset_of!(mnode_t, visframe) == 4);
    assert!(core::mem::offset_of!(mnode_t, mins) == 8);
    assert!(core::mem::offset_of!(mnode_t, maxs) == 20);
    assert!(core::mem::offset_of!(mnode_t, parent) == 32);
    assert!(core::mem::offset_of!(mnode_t, plane) == 40);
    assert!(core::mem::offset_of!(mnode_t, children) == 48);
    assert!(core::mem::offset_of!(mnode_t, cluster) == 64);
    assert!(core::mem::offset_of!(mnode_t, area) == 68);
    assert!(core::mem::offset_of!(mnode_t, firstmarksurface) == 72);
    assert!(core::mem::offset_of!(mnode_t, nummarksurfaces) == 80);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<mnode_t>() == 64);
    assert!(core::mem::offset_of!(mnode_t, visframe) == 4);
    assert!(core::mem::offset_of!(mnode_t, mins) == 8);
    assert!(core::mem::offset_of!(mnode_t, maxs) == 20);
    assert!(core::mem::offset_of!(mnode_t, parent) == 32);
    assert!(core::mem::offset_of!(mnode_t, plane) == 36);
    assert!(core::mem::offset_of!(mnode_t, children) == 40);
    assert!(core::mem::offset_of!(mnode_t, cluster) == 48);
    assert!(core::mem::offset_of!(mnode_t, area) == 52);
    assert!(core::mem::offset_of!(mnode_t, firstmarksurface) == 56);
    assert!(core::mem::offset_of!(mnode_t, nummarksurfaces) == 60);
};
