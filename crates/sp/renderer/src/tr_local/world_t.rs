#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_int, c_uchar, c_ushort};

use sp_engine_qcommon::qfiles::dshader_s::dshader_t;
use sp_qshared::shared::collision::cplane_t;
use sp_qshared::shared::vec3_t;

use super::bmodel_t::bmodel_t;
use super::fog_t::fog_t;
use super::mgrid_t::mgrid_t;
use super::mnode_s::mnode_t;
use super::msurface_s::msurface_t;

/// Raven `world_t` — the loaded BSP world: geometry, planes, fogs, and light grid.
///
/// Type definition source: `oracle/code/renderer/tr_local.h:896-951`
#[repr(C)]
pub struct world_t {
    pub numShaders: c_int,
    pub shaders: *mut dshader_t,

    pub bmodels: *mut bmodel_t,

    pub numplanes: c_int,
    pub planes: *mut cplane_t,

    /// includes leafs
    pub numnodes: c_int,
    pub numDecisionNodes: c_int,
    pub nodes: *mut mnode_t,

    pub numsurfaces: c_int,
    pub surfaces: *mut msurface_t,

    pub nummarksurfaces: c_int,
    pub marksurfaces: *mut *mut msurface_t,

    pub numfogs: c_int,
    pub fogs: *mut fog_t,
    pub globalFog: c_int,

    pub startLightMapIndex: c_int,

    pub lightGridOrigin: vec3_t,
    pub lightGridSize: vec3_t,
    pub lightGridInverseSize: vec3_t,
    pub lightGridBounds: [c_int; 3],
    pub lightGridData: *mut mgrid_t,
    pub lightGridArray: *mut c_ushort,
    pub numGridArrayElements: c_int,

    pub numClusters: c_int,
    pub clusterBytes: c_int,

    /// may be passed in by CM_LoadMap to save space
    pub vis: *const c_uchar,

    /// clusterBytes of 0xff
    pub novis: *mut c_uchar,
}

const _: () = assert!(core::mem::size_of::<world_t>() == 208);
const _: () = assert!(core::mem::offset_of!(world_t, numShaders) == 0);
const _: () = assert!(core::mem::offset_of!(world_t, shaders) == 8);
const _: () = assert!(core::mem::offset_of!(world_t, bmodels) == 16);
const _: () = assert!(core::mem::offset_of!(world_t, numplanes) == 24);
const _: () = assert!(core::mem::offset_of!(world_t, planes) == 32);
const _: () = assert!(core::mem::offset_of!(world_t, numnodes) == 40);
const _: () = assert!(core::mem::offset_of!(world_t, numDecisionNodes) == 44);
const _: () = assert!(core::mem::offset_of!(world_t, nodes) == 48);
const _: () = assert!(core::mem::offset_of!(world_t, numsurfaces) == 56);
const _: () = assert!(core::mem::offset_of!(world_t, surfaces) == 64);
const _: () = assert!(core::mem::offset_of!(world_t, nummarksurfaces) == 72);
const _: () = assert!(core::mem::offset_of!(world_t, marksurfaces) == 80);
const _: () = assert!(core::mem::offset_of!(world_t, numfogs) == 88);
const _: () = assert!(core::mem::offset_of!(world_t, fogs) == 96);
const _: () = assert!(core::mem::offset_of!(world_t, globalFog) == 104);
const _: () = assert!(core::mem::offset_of!(world_t, startLightMapIndex) == 108);
const _: () = assert!(core::mem::offset_of!(world_t, lightGridOrigin) == 112);
const _: () = assert!(core::mem::offset_of!(world_t, lightGridSize) == 124);
const _: () = assert!(core::mem::offset_of!(world_t, lightGridInverseSize) == 136);
const _: () = assert!(core::mem::offset_of!(world_t, lightGridBounds) == 148);
const _: () = assert!(core::mem::offset_of!(world_t, lightGridData) == 160);
const _: () = assert!(core::mem::offset_of!(world_t, lightGridArray) == 168);
const _: () = assert!(core::mem::offset_of!(world_t, numGridArrayElements) == 176);
const _: () = assert!(core::mem::offset_of!(world_t, numClusters) == 180);
const _: () = assert!(core::mem::offset_of!(world_t, clusterBytes) == 184);
const _: () = assert!(core::mem::offset_of!(world_t, vis) == 192);
const _: () = assert!(core::mem::offset_of!(world_t, novis) == 200);
