#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr::null_mut;

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
/// Engine-internal (never crosses the module ABI — modules reach collision only
/// through trap calls), so §D12 internal-only shape applies: `name` owns a
/// `String`, `vised` is `bool`, and the old `repr(C)` layout asserts went with
/// the migration (2026-07-19). The geometry pointers stay raw exactly as
/// transcribed — they index hunk allocations.
///
/// Type definition source: `oracle/codemp/qcommon/cm_local.h:161-211`
pub struct clipMap_t {
    pub name: String,

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
    pub vised: bool,

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
    // Source: oracle/codemp/qcommon/cm_local.h:210
    pub landScape: *mut c_void,
}

/// The `Com_Memset(&cmg, 0, sizeof(cmg))` replacement: every field at its C
/// zero value, the owned `name` empty. Assigning it drops the old `name`; the
/// geometry pointers are hunk-managed and simply overwritten, as Raven's
/// memset did.
impl Default for clipMap_t {
    fn default() -> Self {
        clipMap_t {
            name: String::new(),
            numShaders: 0,
            shaders: null_mut(),
            numBrushSides: 0,
            brushsides: null_mut(),
            numPlanes: 0,
            planes: null_mut(),
            numNodes: 0,
            nodes: null_mut(),
            numLeafs: 0,
            leafs: null_mut(),
            numLeafBrushes: 0,
            leafbrushes: null_mut(),
            numLeafSurfaces: 0,
            leafsurfaces: null_mut(),
            numSubModels: 0,
            cmodels: null_mut(),
            numBrushes: 0,
            brushes: null_mut(),
            numClusters: 0,
            clusterBytes: 0,
            visibility: null_mut(),
            vised: false,
            numEntityChars: 0,
            entityString: null_mut(),
            numAreas: 0,
            areas: null_mut(),
            areaPortals: null_mut(),
            numSurfaces: 0,
            surfaces: null_mut(),
            floodvalid: 0,
            checkcount: 0,
            landScape: null_mut(),
        }
    }
}
