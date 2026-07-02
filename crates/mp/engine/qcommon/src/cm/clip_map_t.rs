#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

use mp_qshared::shared::{qboolean, MAX_QPATH};

use super::c_area_t::cArea_t;
use super::c_leaf_t::cLeaf_t;
use super::c_node_t::cNode_t;
use super::c_patch_t::cPatch_t;
use super::cbrush_s::cbrush_t;
use super::cbrushside_s::cbrushside_t;
use super::ccmshader::CCMShader;
use super::cmodel_s::cmodel_t;
use mp_qshared::shared::collision::cplane_t;

/// Raven `clipMap_t` — the collision model: parsed BSP geometry (planes, nodes,
/// leafs, brushes, patch surfaces) plus area/visibility data used for tracing.
///
/// Type definition source: `oracle/oracle/codemp/qcommon/cm_local.h:161-211`
#[repr(C)]
pub struct clipMap_t {
    pub name: [c_char; MAX_QPATH],

    pub numShaders: c_int,
    pub shaders: *mut CCMShader,

    pub numBrushSides: c_int,
    pub brushsides: *mut cbrushside_t,

    pub numPlanes: c_int,
    pub planes: *mut cplane_t,

    pub numNodes: c_int,
    pub nodes: *mut cNode_t,

    pub numLeafs: c_int,
    pub leafs: *mut cLeaf_t,

    pub numLeafBrushes: c_int,
    pub leafbrushes: *mut c_int,

    pub numLeafSurfaces: c_int,
    pub leafsurfaces: *mut c_int,

    pub numSubModels: c_int,
    pub cmodels: *mut cmodel_t,

    pub numBrushes: c_int,
    pub brushes: *mut cbrush_t,

    pub numClusters: c_int,
    pub clusterBytes: c_int,
    pub visibility: *mut u8,
    /// if false, visibility is just a single cluster of ffs
    pub vised: qboolean,

    pub numEntityChars: c_int,
    pub entityString: *mut c_char,

    pub numAreas: c_int,
    pub areas: *mut cArea_t,
    /// `[ numAreas*numAreas ]` reference counts
    pub areaPortals: *mut c_int,

    pub numSurfaces: c_int,
    /// non-patches will be NULL
    pub surfaces: *mut *mut cPatch_t,

    pub floodvalid: c_int,
    /// incremented on each trace
    pub checkcount: c_int,

    //TODO: Port CCMLandScape
    // Source: oracle/oracle/codemp/qcommon/cm_local.h:210
    pub landScape: *mut c_void,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<clipMap_t>() == 296);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, name) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, numShaders) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, shaders) == 72);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, numBrushSides) == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, brushsides) == 88);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, numPlanes) == 96);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, planes) == 104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, numNodes) == 112);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, nodes) == 120);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, numLeafs) == 128);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, leafs) == 136);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, numLeafBrushes) == 144);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, leafbrushes) == 152);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, numLeafSurfaces) == 160);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, leafsurfaces) == 168);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, numSubModels) == 176);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, cmodels) == 184);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, numBrushes) == 192);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, brushes) == 200);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, numClusters) == 208);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, clusterBytes) == 212);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, visibility) == 216);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, vised) == 224);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, numEntityChars) == 228);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, entityString) == 232);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, numAreas) == 240);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, areas) == 248);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, areaPortals) == 256);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, numSurfaces) == 264);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, surfaces) == 272);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, floodvalid) == 280);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, checkcount) == 284);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clipMap_t, landScape) == 288);
