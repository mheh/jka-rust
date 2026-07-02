#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int, c_uchar, c_ushort};

use mp_engine_qcommon::qfiles::dshader_t::dshader_t;
use mp_qshared::shared::{cplane_t, vec3_t, MAX_QPATH};

use super::bmodel_t::bmodel_t;
use super::fog_t::fog_t;
use super::mgrid_t::mgrid_t;
use super::mnode_s::mnode_t;
use super::msurface_s::msurface_t;

/// Raven `world_t` — the loaded BSP world: geometry, planes, fogs, and light grid.
///
/// Type definition source: `oracle/oracle/codemp/renderer/tr_local.h:1039-1090`
#[repr(C)]
pub struct world_t {
    /// ie: maps/tim_dm2.bsp
    pub name: [c_char; MAX_QPATH],
    /// ie: tim_dm2
    pub baseName: [c_char; MAX_QPATH],

    pub dataSize: c_int,

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

    pub lightGridOrigin: vec3_t,
    pub lightGridSize: vec3_t,
    pub lightGridInverseSize: vec3_t,
    pub lightGridBounds: [c_int; 3],

    pub lightGridOffsets: [c_int; 8],

    pub lightGridStep: vec3_t,

    pub lightGridData: *mut mgrid_t,
    pub lightGridArray: *mut c_ushort,
    pub numGridArrayElements: c_int,

    pub numClusters: c_int,
    pub clusterBytes: c_int,
    /// may be passed in by CM_LoadMap to save space
    pub vis: *const c_uchar,

    /// clusterBytes of 0xff
    pub novis: *mut c_uchar,

    pub entityString: *mut c_char,
    pub entityParsePoint: *mut c_char,
}

const _: () = assert!(core::mem::size_of::<world_t>() == 392);
const _: () = assert!(core::mem::offset_of!(world_t, name) == 0);
const _: () = assert!(core::mem::offset_of!(world_t, baseName) == 64);
const _: () = assert!(core::mem::offset_of!(world_t, dataSize) == 128);
const _: () = assert!(core::mem::offset_of!(world_t, numShaders) == 132);
const _: () = assert!(core::mem::offset_of!(world_t, shaders) == 136);
const _: () = assert!(core::mem::offset_of!(world_t, bmodels) == 144);
const _: () = assert!(core::mem::offset_of!(world_t, numplanes) == 152);
const _: () = assert!(core::mem::offset_of!(world_t, planes) == 160);
const _: () = assert!(core::mem::offset_of!(world_t, numnodes) == 168);
const _: () = assert!(core::mem::offset_of!(world_t, numDecisionNodes) == 172);
const _: () = assert!(core::mem::offset_of!(world_t, nodes) == 176);
const _: () = assert!(core::mem::offset_of!(world_t, numsurfaces) == 184);
const _: () = assert!(core::mem::offset_of!(world_t, surfaces) == 192);
const _: () = assert!(core::mem::offset_of!(world_t, nummarksurfaces) == 200);
const _: () = assert!(core::mem::offset_of!(world_t, marksurfaces) == 208);
const _: () = assert!(core::mem::offset_of!(world_t, numfogs) == 216);
const _: () = assert!(core::mem::offset_of!(world_t, fogs) == 224);
const _: () = assert!(core::mem::offset_of!(world_t, globalFog) == 232);
const _: () = assert!(core::mem::offset_of!(world_t, lightGridOrigin) == 236);
const _: () = assert!(core::mem::offset_of!(world_t, lightGridSize) == 248);
const _: () = assert!(core::mem::offset_of!(world_t, lightGridInverseSize) == 260);
const _: () = assert!(core::mem::offset_of!(world_t, lightGridBounds) == 272);
const _: () = assert!(core::mem::offset_of!(world_t, lightGridOffsets) == 284);
const _: () = assert!(core::mem::offset_of!(world_t, lightGridStep) == 316);
const _: () = assert!(core::mem::offset_of!(world_t, lightGridData) == 328);
const _: () = assert!(core::mem::offset_of!(world_t, lightGridArray) == 336);
const _: () = assert!(core::mem::offset_of!(world_t, numGridArrayElements) == 344);
const _: () = assert!(core::mem::offset_of!(world_t, numClusters) == 348);
const _: () = assert!(core::mem::offset_of!(world_t, clusterBytes) == 352);
const _: () = assert!(core::mem::offset_of!(world_t, vis) == 360);
const _: () = assert!(core::mem::offset_of!(world_t, novis) == 368);
const _: () = assert!(core::mem::offset_of!(world_t, entityString) == 376);
const _: () = assert!(core::mem::offset_of!(world_t, entityParsePoint) == 384);
