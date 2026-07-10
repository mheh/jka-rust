#![allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    clippy::too_many_arguments
)]

//! `cm_load.cpp` — the collision-model loader: BSP lump parsing (shaders,
//! planes, nodes, leafs, brushes, submodels, patches, visibility, entity
//! string), the box/capsule synthetic trace hulls, sub-BSP instancing, and
//! the cached-map-diskimage lifecycle.
//!
//! Source: `oracle/codemp/qcommon/cm_load.cpp`
//!
//! PORT-NOTE(rm-types): `RenderModels`/`RmManager` are state-receiver types
//! pinned by the engine-fork-discovery preamble's receiver order
//! (rmg-terrain.md/tr-model.md own their real shape); neither has landed in
//! this crate yet. Referenced by their exact resolved-signature names per the
//! no-stub rule (common_fns.rs/vm_x86.rs/cm_polylib.rs precedent); reported as
//! missing symbols for the finisher to replace with the real imports once
//! they land.
//!
//! PORT-NOTE(cm-fields): `CollisionWorld` (`crate::collision_world`) is still
//! a `//TODO: Port CollisionWorld fields` placeholder (`_private: ()`).
//! Bodies below reach it as `cm.cmg`/`cm.SubBSP`/`cm.NumSubBSP`/
//! `cm.TotalSubModels`/`cm.box_model`/`cm.box_planes`/`cm.box_brush`/
//! `cm.cmod_base`/`cm.cm_noAreas`/`cm.cm_noCurves`/`cm.cm_playerCurveClip`/
//! `cm.gpvCachedMapDiskImage`/`cm.gsCachedMapDiskImage`/
//! `cm.gbUsingCachedMapDataRightNow`/`cm.last_checksum` — the exact Raven
//! global names per the STATE THREADED tables (STATE FIELDS rule) — reported
//! in missing_symbols for the finisher to add once the struct lands.

use core::ffi::{c_char, c_int, c_uint};

use mp_qshared::common::mp::qcommon::tags::memtag_t;
use mp_qshared::shared::collision::cplane_t;
use mp_qshared::shared::cvar::cvar_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::limits::MAX_SUB_BSP;
use mp_qshared::shared::{qboolean, vec3_t, MAX_QPATH};
use native_types::{clipHandle_t, fileHandle_t, thandle_t};

use mp_host_interface::engine_host::EngineHost;

use crate::cm::c_area_t::cArea_t;
use crate::cm::c_leaf_t::cLeaf_t;
use crate::cm::c_node_t::cNode_t;
use crate::cm::c_patch_t::cPatch_t;
use crate::cm::cbrush_s::cbrush_t;
use crate::cm::cbrushside_s::cbrushside_t;
use crate::cm::ccmshader::CCMShader;
use crate::cm::clip_map_t::clipMap_t;
use crate::cm::cm_load_consts::{
    BOX_BRUSHES, BOX_LEAFS, BOX_PLANES, BOX_SIDES, MAX_PATCH_VERTS, VIS_HEADER,
};
use crate::cm::cm_local_consts::{BOX_MODEL_HANDLE, CAPSULE_MODEL_HANDLE, MAX_SUBMODELS};
use crate::cm::cmodel_s::{cmodel_s, cmodel_t};
use crate::collision_world::CollisionWorld;
use crate::common::com_error;
use crate::common::Common;
use crate::common_fns::{Com_Memcpy, Com_Memset};
use crate::qfiles::bsp_limits::BSP_VERSION;
use crate::qfiles::dbrush_t::dbrush_t;
use crate::qfiles::dbrushside_t::dbrushside_t;
use crate::qfiles::dheader_t::dheader_t;
use crate::qfiles::dleaf_t::dleaf_t;
use crate::qfiles::dmodel_t::dmodel_t;
use crate::qfiles::dnode_t::dnode_t;
use crate::qfiles::dplane_t::dplane_t;
use crate::qfiles::draw_vert_t::drawVert_t;
use crate::qfiles::dshader_t::dshader_t;
use crate::qfiles::dsurface_t::dsurface_t;
use crate::qfiles::lump_indices::{
    LUMP_BRUSHES, LUMP_BRUSHSIDES, LUMP_DRAWVERTS, LUMP_ENTITIES, LUMP_LEAFBRUSHES, LUMP_LEAFS,
    LUMP_LEAFSURFACES, LUMP_MODELS, LUMP_NODES, LUMP_PLANES, LUMP_SHADERS, LUMP_SURFACES,
    LUMP_VISIBILITY,
};
use crate::qfiles::lump_t::lump_t;
use crate::qfiles::map_surface_type_t::mapSurfaceType_t;

// PORT-NOTE(rm-types): see module doc.
#[allow(dead_code)]
pub struct RenderModels;
#[allow(dead_code)]
pub struct RmManager;

// PORT-NOTE(rmg-terrain): `CCMLandScape` is the rmg-terrain.md §F design's
// class (porting-rules §F) — not the type rosetta. Referenced opaquely here
// (raw pointer only, per the frozen §F seam) exactly as the packet resolves
// it; the finisher wires the real import once rmg-terrain.md's crate lands.
#[allow(dead_code)]
pub struct CCMLandScape;

// PORT-NOTE(rmg-terrain): `CRMManager` is likewise the rmg-terrain.md §F
// class (`TheRandomMissionManager` global) — opaque placeholder pending that
// crate landing.
#[allow(dead_code)]
pub struct CRMManager;

// ---------------------------------------------------------------------
// Externally-ported callees this file reaches whose bodies are not linked
// into this crate yet — forward-declared with the faithful shape inferred
// from the Raven call sites (receivers per the packets' RESOLVED CALL
// SURFACE tables), matching the established `extern "Rust"` forward-declare
// convention used elsewhere in this crate (`be_aas_move.rs`, `cm_polylib.rs`).
// PORT-NOTE(callee-signatures): reported in missing_symbols.
// ---------------------------------------------------------------------
extern "Rust" {
    fn CM_ClearLevelPatches(cm: &mut CollisionWorld);
    fn CM_ShutdownShaderProperties(cm: &mut CollisionWorld);
    fn Com_BlockChecksum(common: &mut Common, buffer: *const (), length: c_int) -> c_uint;
    fn CM_GeneratePatchCollide(
        common: &mut Common,
        cm: &mut CollisionWorld,
        rm: &mut RenderModels,
        host: &mut dyn EngineHost,
        width: c_int,
        height: c_int,
        points: *mut vec3_t,
    ) -> *mut crate::cm::patch_collide_s::patchCollide_s;
    fn CM_LoadShaderText(
        common: &mut Common,
        cm: &mut CollisionWorld,
        rm: &mut RenderModels,
        rmg: &mut RmManager,
        host: &mut dyn EngineHost,
        server: qboolean,
    );
    fn CM_SetupShaderProperties(
        common: &mut Common,
        cm: &mut CollisionWorld,
        rmg: &mut RmManager,
        host: &mut dyn EngineHost,
    );
    fn CM_FloodAreaConnections(cm: &mut clipMap_t);
    fn CM_InitTerrain(
        rmg: &mut RmManager,
        host: &mut dyn EngineHost,
        config: *const c_char,
        checksum: c_int,
        server: bool,
    ) -> *mut CCMLandScape;
    fn Hunk_Alloc(
        common: &mut Common,
        cm: &mut CollisionWorld,
        rm: &mut RenderModels,
        host: &mut dyn EngineHost,
        size: usize,
        h_high: c_int,
    ) -> *mut ();
    fn Cvar_Get(
        common: &mut Common,
        cm: &mut CollisionWorld,
        rm: &mut RenderModels,
        host: &mut dyn EngineHost,
        var_name: *const c_char,
        var_value: *const c_char,
        flags: c_int,
    ) -> *mut cvar_t;
    fn Com_DPrintf(common: &mut Common, fmt: *const c_char, ...);
    fn FS_FOpenFileRead(
        common: &mut Common,
        cm: &mut CollisionWorld,
        rm: &mut RenderModels,
        host: &mut dyn EngineHost,
        filename: *const c_char,
        file: *mut fileHandle_t,
        uniqueFILE: qboolean,
    ) -> c_int;
    fn FS_Read(common: &mut Common, buffer: *mut (), len: c_int, f: fileHandle_t) -> c_int;
    fn FS_FCloseFile(common: &mut Common, f: fileHandle_t);
    fn Z_Malloc(
        common: &mut Common,
        cm: &mut CollisionWorld,
        rm: &mut RenderModels,
        host: &mut dyn EngineHost,
        iSize: c_int,
        eTag: memtag_t,
        bZeroit: qboolean,
        iUnusedAlign: c_int,
    ) -> *mut ();
    fn Z_Free(common: &mut Common, pvAddress: *mut ());
    fn Sys_LowPhysicalMemory() -> c_int;
    // PORT-NOTE(q_math-reach): `Q_strncpyz`/`SetPlaneSignbits` (q_shared/q_math
    // primitives) are ported in `mp_game`, a tier above this crate's
    // dependency graph (cm_polylib.rs precedent) — not reachable here.
    // Referenced by their exact Raven names; reported as missing symbols.
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    fn SetPlaneSignbits(out: *mut cplane_t);
    fn PlaneTypeForNormal(normal: vec3_t) -> c_int;

    // PORT-NOTE(rmg-terrain): `CCMLandScape`/`CRMManager` methods (the
    // rmg-terrain.md §F class, porting-rules §F) referenced opaquely by their
    // exact Raven member names via C-style free-fn shims (`Class_Method`)
    // pending that crate landing; reported as missing symbols.
    fn CCMLandScape_DecreaseRefCount(ls: *mut CCMLandScape);
    fn CCMLandScape_GetRefCount(ls: *mut CCMLandScape) -> c_int;
    fn CCMLandScape_delete(ls: *mut CCMLandScape);
    fn CCMLandScape_IncreaseRefCount(ls: *mut CCMLandScape);
    fn CRMManager_delete(mgr: *mut CRMManager);
}

const H_HIGH: c_int = 1;

/// Raven `CM_BoundBrush`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:211-220`
pub fn CM_BoundBrush(b: *mut cbrush_t) {
    unsafe {
        (*b).bounds[0][0] = -(*(*b).sides).plane.as_ref().unwrap().dist;
        // PORT-NOTE(sides-indexing): Raven indexes `b->sides[N]` (an array of
        // 6 `cbrushside_t`); `cbrush_t::sides` is `*mut cbrushside_t`, so the
        // per-side accesses below use pointer offset arithmetic to match.
        let sides = (*b).sides;
        (*b).bounds[0][0] = -(*(*sides.offset(0)).plane).dist;
        (*b).bounds[1][0] = (*(*sides.offset(1)).plane).dist;

        (*b).bounds[0][1] = -(*(*sides.offset(2)).plane).dist;
        (*b).bounds[1][1] = (*(*sides.offset(3)).plane).dist;

        (*b).bounds[0][2] = -(*(*sides.offset(4)).plane).dist;
        (*b).bounds[1][2] = (*(*sides.offset(5)).plane).dist;
    }
}

/// Raven `CM_NumClusters`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:890-892`
pub fn CM_NumClusters(cm: &mut CollisionWorld) -> c_int {
    cm.cmg.numClusters
}

/// Raven `CM_NumInlineModels`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:894-896`
pub fn CM_NumInlineModels(cm: &mut CollisionWorld) -> c_int {
    cm.cmg.numSubModels
}

/// Raven `CM_EntityString`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:898-900`
pub fn CM_EntityString(cm: &mut CollisionWorld) -> *mut c_char {
    cm.cmg.entityString
}

/// Raven `CM_SubBSPEntityString`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:902-905`
pub fn CM_SubBSPEntityString(cm: &mut CollisionWorld, index: c_int) -> *mut c_char {
    cm.SubBSP[index as usize].entityString
}

/// Raven `CM_InitBoxHull`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:932-976`
pub fn CM_InitBoxHull(cm: &mut CollisionWorld) {
    unsafe {
        cm.box_planes = cm.cmg.planes.offset(cm.cmg.numPlanes as isize);

        cm.box_brush = cm.cmg.brushes.offset(cm.cmg.numBrushes as isize);
        (*cm.box_brush).numsides = 6;
        (*cm.box_brush).sides = cm.cmg.brushsides.offset(cm.cmg.numBrushSides as isize);
        (*cm.box_brush).contents = mp_qshared::shared::surface_flags::CONTENTS_BODY;

        cm.box_model.firstNode = -1;
        cm.box_model.leaf.numLeafBrushes = 1;
        cm.box_model.leaf.firstLeafBrush = cm.cmg.numLeafBrushes;
        *cm.cmg.leafbrushes.offset(cm.cmg.numLeafBrushes as isize) = cm.cmg.numBrushes;

        for i in 0..6i32 {
            let side = i & 1;

            // brush sides
            let s = cm
                .cmg
                .brushsides
                .offset((cm.cmg.numBrushSides + i) as isize);
            (*s).plane = cm
                .cmg
                .planes
                .offset((cm.cmg.numPlanes + i * 2 + side) as isize);
            (*s).shaderNum = cm.cmg.numShaders;

            // planes
            let p = cm.box_planes.offset((i * 2) as isize);
            (*p).ptype = (i >> 1) as u8;
            (*p).signbits = 0;
            (*p).normal = [0.0; 3];
            (*p).normal[(i >> 1) as usize] = 1.0;

            let p = cm.box_planes.offset((i * 2 + 1) as isize);
            (*p).ptype = (3 + (i >> 1)) as u8;
            (*p).signbits = 0;
            (*p).normal = [0.0; 3];
            (*p).normal[(i >> 1) as usize] = -1.0;

            SetPlaneSignbits(p);
        }
    }
}

/// Raven `CM_TempBoxModel`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:987-1013`
pub fn CM_TempBoxModel(
    cm: &mut CollisionWorld,
    mins: vec3_t,
    maxs: vec3_t,
    capsule: c_int,
) -> clipHandle_t {
    unsafe {
        cm.box_model.mins = mins;
        cm.box_model.maxs = maxs;

        if capsule != 0 {
            return CAPSULE_MODEL_HANDLE as clipHandle_t;
        }

        (*cm.box_planes.offset(0)).dist = maxs[0];
        (*cm.box_planes.offset(1)).dist = -maxs[0];
        (*cm.box_planes.offset(2)).dist = mins[0];
        (*cm.box_planes.offset(3)).dist = -mins[0];
        (*cm.box_planes.offset(4)).dist = maxs[1];
        (*cm.box_planes.offset(5)).dist = -maxs[1];
        (*cm.box_planes.offset(6)).dist = mins[1];
        (*cm.box_planes.offset(7)).dist = -mins[1];
        (*cm.box_planes.offset(8)).dist = maxs[2];
        (*cm.box_planes.offset(9)).dist = -maxs[2];
        (*cm.box_planes.offset(10)).dist = mins[2];
        (*cm.box_planes.offset(11)).dist = -mins[2];

        (*cm.box_brush).bounds[0] = mins;
        (*cm.box_brush).bounds[1] = maxs;

        BOX_MODEL_HANDLE as clipHandle_t
    }
}

/// Raven `CM_ShutdownTerrain`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:1065-1080`
pub fn CM_ShutdownTerrain(cm: &mut CollisionWorld, terrainId: thandle_t) {
    unsafe {
        let landscape = cm.cmg.landScape;

        if !landscape.is_null() {
            // PORT-NOTE(cpp-methods): `CCMLandScape::DecreaseRefCount`/
            // `GetRefCount`/`delete` are the rmg-terrain.md §F class's methods
            // (porting-rules §F, not the type rosetta); called opaquely per
            // their doc-stated shape, reported as missing symbols.
            CCMLandScape_DecreaseRefCount(landscape);
            if CCMLandScape_GetRefCount(landscape) <= 0 {
                CCMLandScape_delete(landscape);
                cm.cmg.landScape = core::ptr::null_mut();
            }
        }
    }
}

/// Raven `CM_FindSubBSP`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:1110-1130`
pub fn CM_FindSubBSP(cm: &mut CollisionWorld, modelIndex: c_int) -> c_int {
    let mut count = cm.cmg.numSubModels;
    if modelIndex < count {
        // belongs to the main bsp
        return -1;
    }

    for i in 0..cm.NumSubBSP {
        count += cm.SubBSP[i as usize].numSubModels;
        if modelIndex < count {
            return i;
        }
    }
    -1
}

/// Raven `CM_GetWorldBounds`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:1132-1136`
pub fn CM_GetWorldBounds(cm: &mut CollisionWorld, mins: vec3_t, maxs: vec3_t) {
    let mut mins = mins;
    let mut maxs = maxs;
    unsafe {
        mins = (*cm.cmg.cmodels.offset(0)).mins;
        maxs = (*cm.cmg.cmodels.offset(0)).maxs;
    }
    let _ = (mins, maxs);
    // PORT-NOTE(shape-mismatch): `mins`/`maxs` are plain `vec3_t` (by value)
    // per the mechanically-resolved out-param convention already established
    // elsewhere in this crate; the write does not propagate to the caller in
    // this shape (reported in shape_mismatches).
}

/// Raven `CM_ClearMap`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:791-821`
pub fn CM_ClearMap(cm: &mut CollisionWorld, rmg: &mut RmManager) {
    unsafe {
        CM_ShutdownShaderProperties(cm);

        if !cm.TheRandomMissionManager.is_null() {
            // PORT-NOTE(rmg-terrain): `CRMManager` delete is the rmg-terrain.md
            // §F class's destructor (porting-rules §F) — called opaquely,
            // reported as a missing symbol.
            CRMManager_delete(cm.TheRandomMissionManager);
            cm.TheRandomMissionManager = core::ptr::null_mut();
        }

        if !cm.cmg.landScape.is_null() {
            CCMLandScape_delete(cm.cmg.landScape);
            cm.cmg.landScape = core::ptr::null_mut();
        }

        Com_Memset(
            &mut cm.cmg as *mut clipMap_t as *mut (),
            0,
            core::mem::size_of::<clipMap_t>(),
        );
        CM_ClearLevelPatches(cm);

        for i in 0..cm.NumSubBSP {
            Com_Memset(
                &mut cm.SubBSP[i as usize] as *mut clipMap_t as *mut (),
                0,
                core::mem::size_of::<clipMap_t>(),
            );
        }
        cm.NumSubBSP = 0;
        cm.TotalSubModels = 0;
    }
}

/// Raven `CM_LumpChecksum`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:540-542`
pub fn CM_LumpChecksum(common: &mut Common, cm: &mut CollisionWorld, lump: *mut lump_t) -> c_uint {
    unsafe {
        i32::from_le(Com_BlockChecksum(
            common,
            cm.cmod_base.offset((*lump).fileofs as isize) as *const (),
            (*lump).filelen,
        ) as i32) as c_uint
    }
}

/// Raven `CM_DeleteCachedMap`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:576-599`
pub fn CM_DeleteCachedMap(
    common: &mut Common,
    cm: &mut CollisionWorld,
    bGuaranteedOkToDelete: qboolean,
) -> qboolean {
    let mut bActuallyFreedSomething: qboolean = mp_qshared::shared::qfalse;

    if bGuaranteedOkToDelete != 0 || cm.gbUsingCachedMapDataRightNow == 0 {
        // dump cached disk image...
        if !cm.gpvCachedMapDiskImage.is_null() {
            Z_Free(common, cm.gpvCachedMapDiskImage);
            cm.gpvCachedMapDiskImage = core::ptr::null_mut();

            bActuallyFreedSomething = mp_qshared::shared::qtrue;
        }
        cm.gsCachedMapDiskImage[0] = 0;

        // force map loader to ignore cached internal BSP structures for next
        // level CM_LoadMap() call...
        cm.cmg.name[0] = 0;
    }

    bActuallyFreedSomething
}

/// Raven `CM_Checksum`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:544-559`
pub fn CM_Checksum(common: &mut Common, cm: &mut CollisionWorld, header: *mut dheader_t) -> c_uint {
    unsafe {
        let mut checksums: [c_uint; 16] = [0; 16];
        checksums[0] = CM_LumpChecksum(common, cm, &mut (*header).lumps[LUMP_SHADERS]);
        checksums[1] = CM_LumpChecksum(common, cm, &mut (*header).lumps[LUMP_LEAFS]);
        checksums[2] = CM_LumpChecksum(common, cm, &mut (*header).lumps[LUMP_LEAFBRUSHES]);
        checksums[3] = CM_LumpChecksum(common, cm, &mut (*header).lumps[LUMP_LEAFSURFACES]);
        checksums[4] = CM_LumpChecksum(common, cm, &mut (*header).lumps[LUMP_PLANES]);
        checksums[5] = CM_LumpChecksum(common, cm, &mut (*header).lumps[LUMP_BRUSHSIDES]);
        checksums[6] = CM_LumpChecksum(common, cm, &mut (*header).lumps[LUMP_BRUSHES]);
        checksums[7] = CM_LumpChecksum(common, cm, &mut (*header).lumps[LUMP_MODELS]);
        checksums[8] = CM_LumpChecksum(common, cm, &mut (*header).lumps[LUMP_NODES]);
        checksums[9] = CM_LumpChecksum(common, cm, &mut (*header).lumps[LUMP_SURFACES]);
        checksums[10] = CM_LumpChecksum(common, cm, &mut (*header).lumps[LUMP_DRAWVERTS]);

        i32::from_le(Com_BlockChecksum(common, checksums.as_ptr() as *const (), 11 * 4) as i32)
            as c_uint
    }
}

/// Raven `CM_ClipHandleToModel`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:828-875`
pub fn CM_ClipHandleToModel(
    cm: &mut CollisionWorld,
    handle: clipHandle_t,
    clipMap: *mut *mut clipMap_t,
) -> *mut cmodel_t {
    unsafe {
        if handle < 0 {
            com_error(
                errorParm_t::ERR_DROP,
                format!("CM_ClipHandleToModel: bad handle {}", handle),
            );
        }
        if handle < cm.cmg.numSubModels {
            if !clipMap.is_null() {
                *clipMap = &mut cm.cmg as *mut clipMap_t;
            }
            return cm.cmg.cmodels.offset(handle as isize);
        }
        if handle == BOX_MODEL_HANDLE as clipHandle_t {
            if !clipMap.is_null() {
                *clipMap = &mut cm.cmg as *mut clipMap_t;
            }
            return &mut cm.box_model as *mut cmodel_t;
        }

        let mut count = cm.cmg.numSubModels;
        for i in 0..cm.NumSubBSP {
            if handle < count + cm.SubBSP[i as usize].numSubModels {
                if !clipMap.is_null() {
                    *clipMap = &mut cm.SubBSP[i as usize] as *mut clipMap_t;
                }
                return cm.SubBSP[i as usize]
                    .cmodels
                    .offset((handle - count) as isize);
            }
            count += cm.SubBSP[i as usize].numSubModels;
        }

        if handle < MAX_SUBMODELS as clipHandle_t {
            com_error(
                errorParm_t::ERR_DROP,
                format!(
                    "CM_ClipHandleToModel: bad handle {} < {} < {}",
                    cm.cmg.numSubModels, handle, MAX_SUBMODELS
                ),
            );
        }
        com_error(
            errorParm_t::ERR_DROP,
            format!(
                "CM_ClipHandleToModel: bad handle {}",
                handle + MAX_SUBMODELS as clipHandle_t
            ),
        );
    }
}

/// Raven `CM_InlineModel`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:882-888`
pub fn CM_InlineModel(cm: &mut CollisionWorld, index: c_int) -> clipHandle_t {
    if index < 0 || index >= cm.TotalSubModels {
        com_error(
            errorParm_t::ERR_DROP,
            format!(
                "CM_InlineModel: bad number: {} > {}",
                index, cm.TotalSubModels
            ),
        );
    }
    index
}

/// Raven `CMod_LoadShaders`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:76-101`
pub fn CMod_LoadShaders(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    l: *mut lump_t,
    cmap: &mut clipMap_t,
) {
    unsafe {
        let mut r#in = cm.cmod_base.offset((*l).fileofs as isize) as *mut dshader_t;
        if (*l).filelen as usize % core::mem::size_of::<dshader_t>() != 0 {
            com_error(
                errorParm_t::ERR_DROP,
                "CMod_LoadShaders: funny lump size".into(),
            );
        }
        let count = (*l).filelen as usize / core::mem::size_of::<dshader_t>();

        if count < 1 {
            com_error(errorParm_t::ERR_DROP, "Map with no shaders".into());
        }
        cmap.shaders = Hunk_Alloc(
            common,
            cm,
            rm,
            host,
            (1 + count) * core::mem::size_of::<CCMShader>(),
            H_HIGH,
        ) as *mut CCMShader;
        cmap.numShaders = count as c_int;

        let mut out = cmap.shaders;
        for _ in 0..count {
            Q_strncpyz(
                (*out).shader.as_mut_ptr(),
                (*r#in).shader.as_ptr(),
                MAX_QPATH as c_int,
            );
            (*out).contentFlags = i32::from_le((*r#in).contentFlags);
            (*out).surfaceFlags = i32::from_le((*r#in).surfaceFlags);
            r#in = r#in.offset(1);
            out = out.offset(1);
        }
    }
}

/// Raven `CMod_LoadSubmodels`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:109-166`
pub fn CMod_LoadSubmodels(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    l: *mut lump_t,
    cmap: &mut clipMap_t,
) {
    unsafe {
        let mut r#in = cm.cmod_base.offset((*l).fileofs as isize) as *mut dmodel_t;
        if (*l).filelen as usize % core::mem::size_of::<dmodel_t>() != 0 {
            com_error(
                errorParm_t::ERR_DROP,
                "CMod_LoadSubmodels: funny lump size".into(),
            );
        }
        let count = (*l).filelen as usize / core::mem::size_of::<dmodel_t>();

        if count < 1 {
            com_error(errorParm_t::ERR_DROP, "Map with no models".into());
        }
        cmap.cmodels = Hunk_Alloc(
            common,
            cm,
            rm,
            host,
            count * core::mem::size_of::<cmodel_s>(),
            H_HIGH,
        ) as *mut cmodel_s;
        cmap.numSubModels = count as c_int;

        if count > MAX_SUBMODELS {
            com_error(errorParm_t::ERR_DROP, "MAX_SUBMODELS exceeded".into());
        }

        for i in 0..count {
            let out = cmap.cmodels.offset(i as isize);

            for j in 0..3usize {
                // spread the mins / maxs by a pixel
                (*out).mins[j] = (*r#in).mins[j] - 1.0;
                (*out).maxs[j] = (*r#in).maxs[j] + 1.0;
            }

            // rwwRMG - sof2 doesn't have to add this &cm == &cmg check.
            // Are they getting leaf data elsewhere? (the reason this needs to
            // be done is in sub bsp instances the first brush model isn't
            // necessary a world model and might be real architecture)
            if i == 0 && core::ptr::eq(cmap as *const clipMap_t, &cm.cmg as *const clipMap_t) {
                (*out).firstNode = 0;
                r#in = r#in.offset(1);
                continue; // world model doesn't need other info
            }

            // make a "leaf" just to hold the model's brushes and surfaces
            (*out).firstNode = -1;

            (*out).leaf.numLeafBrushes = (*r#in).numBrushes;
            let indexes = Hunk_Alloc(
                common,
                cm,
                rm,
                host,
                (*out).leaf.numLeafBrushes as usize * 4,
                H_HIGH,
            ) as *mut c_int;
            (*out).leaf.firstLeafBrush = (indexes.offset_from(cmap.leafbrushes)) as c_int;
            for j in 0..(*out).leaf.numLeafBrushes {
                *indexes.offset(j as isize) = (*r#in).firstBrush + j;
            }

            (*out).leaf.numLeafSurfaces = (*r#in).numSurfaces;
            let indexes = Hunk_Alloc(
                common,
                cm,
                rm,
                host,
                (*out).leaf.numLeafSurfaces as usize * 4,
                H_HIGH,
            ) as *mut c_int;
            (*out).leaf.firstLeafSurface = (indexes.offset_from(cmap.leafsurfaces)) as c_int;
            for j in 0..(*out).leaf.numLeafSurfaces {
                *indexes.offset(j as isize) = (*r#in).firstSurface + j;
            }

            r#in = r#in.offset(1);
        }
    }
}

/// Raven `CMod_LoadNodes`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:175-203`
pub fn CMod_LoadNodes(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    l: *mut lump_t,
    cmap: &mut clipMap_t,
) {
    unsafe {
        let mut r#in = cm.cmod_base.offset((*l).fileofs as isize) as *mut dnode_t;
        if (*l).filelen as usize % core::mem::size_of::<dnode_t>() != 0 {
            com_error(
                errorParm_t::ERR_DROP,
                "MOD_LoadBmodel: funny lump size".into(),
            );
        }
        let count = (*l).filelen as usize / core::mem::size_of::<dnode_t>();

        if count < 1 {
            com_error(errorParm_t::ERR_DROP, "Map has no nodes".into());
        }
        cmap.nodes = Hunk_Alloc(
            common,
            cm,
            rm,
            host,
            count * core::mem::size_of::<cNode_t>(),
            H_HIGH,
        ) as *mut cNode_t;
        cmap.numNodes = count as c_int;

        let mut out = cmap.nodes;
        for _ in 0..count {
            (*out).plane = cmap.planes.offset(i32::from_le((*r#in).planeNum) as isize);
            for j in 0..2usize {
                let child = i32::from_le((*r#in).children[j]);
                (*out).children[j] = child;
            }
            out = out.offset(1);
            r#in = r#in.offset(1);
        }
    }
}

/// Raven `CMod_LoadBrushes`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:229-262`
pub fn CMod_LoadBrushes(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    l: *mut lump_t,
    cmap: &mut clipMap_t,
) {
    unsafe {
        let mut r#in = cm.cmod_base.offset((*l).fileofs as isize) as *mut dbrush_t;
        if (*l).filelen as usize % core::mem::size_of::<dbrush_t>() != 0 {
            com_error(
                errorParm_t::ERR_DROP,
                "MOD_LoadBmodel: funny lump size".into(),
            );
        }
        let count = (*l).filelen as usize / core::mem::size_of::<dbrush_t>();

        cmap.brushes = Hunk_Alloc(
            common,
            cm,
            rm,
            host,
            (BOX_BRUSHES + count) * core::mem::size_of::<cbrush_t>(),
            H_HIGH,
        ) as *mut cbrush_t;
        cmap.numBrushes = count as c_int;

        let mut out = cmap.brushes;
        for _ in 0..count {
            (*out).sides = cmap
                .brushsides
                .offset(i32::from_le((*r#in).firstSide) as isize);
            (*out).numsides = i32::from_le((*r#in).numSides) as u16;

            (*out).shaderNum = i32::from_le((*r#in).shaderNum);
            if (*out).shaderNum < 0 || (*out).shaderNum >= cmap.numShaders {
                com_error(
                    errorParm_t::ERR_DROP,
                    format!("CMod_LoadBrushes: bad shaderNum: {}", (*out).shaderNum),
                );
            }
            (*out).contents = (*cmap.shaders.offset((*out).shaderNum as isize)).contentFlags;

            // Landscapes are set up afterwards in the entity spawning

            CM_BoundBrush(out);

            out = out.offset(1);
            r#in = r#in.offset(1);
        }
    }
}

/// Raven `CMod_LoadLeafs`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:269-305`
pub fn CMod_LoadLeafs(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    l: *mut lump_t,
    cmap: &mut clipMap_t,
) {
    unsafe {
        let mut r#in = cm.cmod_base.offset((*l).fileofs as isize) as *mut dleaf_t;
        if (*l).filelen as usize % core::mem::size_of::<dleaf_t>() != 0 {
            com_error(
                errorParm_t::ERR_DROP,
                "MOD_LoadBmodel: funny lump size".into(),
            );
        }
        let count = (*l).filelen as usize / core::mem::size_of::<dleaf_t>();

        if count < 1 {
            com_error(errorParm_t::ERR_DROP, "Map with no leafs".into());
        }

        cmap.leafs = Hunk_Alloc(
            common,
            cm,
            rm,
            host,
            (BOX_LEAFS + count) * core::mem::size_of::<cLeaf_t>(),
            H_HIGH,
        ) as *mut cLeaf_t;
        cmap.numLeafs = count as c_int;

        let mut out = cmap.leafs;
        for _ in 0..count {
            (*out).cluster = i32::from_le((*r#in).cluster);
            (*out).area = i32::from_le((*r#in).area);
            (*out).firstLeafBrush = i32::from_le((*r#in).firstLeafBrush);
            (*out).numLeafBrushes = i32::from_le((*r#in).numLeafBrushes);
            (*out).firstLeafSurface = i32::from_le((*r#in).firstLeafSurface);
            (*out).numLeafSurfaces = i32::from_le((*r#in).numLeafSurfaces);

            if (*out).cluster >= cmap.numClusters {
                cmap.numClusters = (*out).cluster + 1;
            }
            if (*out).area >= cmap.numAreas {
                cmap.numAreas = (*out).area + 1;
            }

            out = out.offset(1);
            r#in = r#in.offset(1);
        }

        cmap.areas = Hunk_Alloc(
            common,
            cm,
            rm,
            host,
            cmap.numAreas as usize * core::mem::size_of::<cArea_t>(),
            H_HIGH,
        ) as *mut cArea_t;
        cmap.areaPortals = Hunk_Alloc(
            common,
            cm,
            rm,
            host,
            cmap.numAreas as usize * cmap.numAreas as usize * core::mem::size_of::<c_int>(),
            H_HIGH,
        ) as *mut c_int;
    }
}

/// Raven `CMod_LoadPlanes`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:312-346`
pub fn CMod_LoadPlanes(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    l: *mut lump_t,
    cmap: &mut clipMap_t,
) {
    unsafe {
        let mut r#in = cm.cmod_base.offset((*l).fileofs as isize) as *mut dplane_t;
        if (*l).filelen as usize % core::mem::size_of::<dplane_t>() != 0 {
            com_error(
                errorParm_t::ERR_DROP,
                "MOD_LoadBmodel: funny lump size".into(),
            );
        }
        let count = (*l).filelen as usize / core::mem::size_of::<dplane_t>();

        if count < 1 {
            com_error(errorParm_t::ERR_DROP, "Map with no planes".into());
        }
        cmap.planes = Hunk_Alloc(
            common,
            cm,
            rm,
            host,
            (BOX_PLANES + count) * core::mem::size_of::<cplane_t>(),
            H_HIGH,
        ) as *mut cplane_t;
        cmap.numPlanes = count as c_int;

        let mut out = cmap.planes;
        for _ in 0..count {
            let mut bits = 0u8;
            for j in 0..3usize {
                (*out).normal[j] = f32::from_le_bytes((*r#in).normal[j].to_le_bytes());
                if (*out).normal[j] < 0.0 {
                    bits |= 1 << j;
                }
            }

            (*out).dist = f32::from_le_bytes((*r#in).dist.to_le_bytes());
            (*out).ptype = PlaneTypeForNormal((*out).normal) as u8;
            (*out).signbits = bits;

            out = out.offset(1);
            r#in = r#in.offset(1);
        }
    }
}

/// Raven `CMod_LoadLeafBrushes`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:353-373`
pub fn CMod_LoadLeafBrushes(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    l: *mut lump_t,
    cmap: &mut clipMap_t,
) {
    unsafe {
        let mut r#in = cm.cmod_base.offset((*l).fileofs as isize) as *mut c_int;
        if (*l).filelen as usize % core::mem::size_of::<c_int>() != 0 {
            com_error(
                errorParm_t::ERR_DROP,
                "MOD_LoadBmodel: funny lump size".into(),
            );
        }
        let count = (*l).filelen as usize / core::mem::size_of::<c_int>();

        cmap.leafbrushes = Hunk_Alloc(
            common,
            cm,
            rm,
            host,
            (count + BOX_BRUSHES) * core::mem::size_of::<c_int>(),
            H_HIGH,
        ) as *mut c_int;
        cmap.numLeafBrushes = count as c_int;

        let mut out = cmap.leafbrushes;
        for _ in 0..count {
            *out = i32::from_le(*r#in);
            out = out.offset(1);
            r#in = r#in.offset(1);
        }
    }
}

/// Raven `CMod_LoadLeafSurfaces`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:380-400`
pub fn CMod_LoadLeafSurfaces(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    l: *mut lump_t,
    cmap: &mut clipMap_t,
) {
    unsafe {
        let mut r#in = cm.cmod_base.offset((*l).fileofs as isize) as *mut c_int;
        if (*l).filelen as usize % core::mem::size_of::<c_int>() != 0 {
            com_error(
                errorParm_t::ERR_DROP,
                "MOD_LoadBmodel: funny lump size".into(),
            );
        }
        let count = (*l).filelen as usize / core::mem::size_of::<c_int>();

        cmap.leafsurfaces = Hunk_Alloc(
            common,
            cm,
            rm,
            host,
            count * core::mem::size_of::<c_int>(),
            H_HIGH,
        ) as *mut c_int;
        cmap.numLeafSurfaces = count as c_int;

        let mut out = cmap.leafsurfaces;
        for _ in 0..count {
            *out = i32::from_le(*r#in);
            out = out.offset(1);
            r#in = r#in.offset(1);
        }
    }
}

/// Raven `CMod_LoadBrushSides`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:407-434`
pub fn CMod_LoadBrushSides(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    l: *mut lump_t,
    cmap: &mut clipMap_t,
) {
    unsafe {
        let mut r#in = cm.cmod_base.offset((*l).fileofs as isize) as *mut dbrushside_t;
        if (*l).filelen as usize % core::mem::size_of::<dbrushside_t>() != 0 {
            com_error(
                errorParm_t::ERR_DROP,
                "MOD_LoadBmodel: funny lump size".into(),
            );
        }
        let count = (*l).filelen as usize / core::mem::size_of::<dbrushside_t>();

        cmap.brushsides = Hunk_Alloc(
            common,
            cm,
            rm,
            host,
            (BOX_SIDES + count) * core::mem::size_of::<cbrushside_t>(),
            H_HIGH,
        ) as *mut cbrushside_t;
        cmap.numBrushSides = count as c_int;

        let mut out = cmap.brushsides;
        for _ in 0..count {
            let num = i32::from_le((*r#in).planeNum);
            (*out).plane = cmap.planes.offset(num as isize);
            (*out).shaderNum = i32::from_le((*r#in).shaderNum);
            if (*out).shaderNum < 0 || (*out).shaderNum >= cmap.numShaders {
                com_error(
                    errorParm_t::ERR_DROP,
                    format!("CMod_LoadBrushSides: bad shaderNum: {}", (*out).shaderNum),
                );
            }

            out = out.offset(1);
            r#in = r#in.offset(1);
        }
    }
}

/// Raven `CMod_LoadEntityString`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:442-446`
pub fn CMod_LoadEntityString(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    l: *mut lump_t,
    cmap: &mut clipMap_t,
) {
    unsafe {
        cmap.entityString =
            Hunk_Alloc(common, cm, rm, host, (*l).filelen as usize, H_HIGH) as *mut c_char;
        cmap.numEntityChars = (*l).filelen;
        Com_Memcpy(
            cmap.entityString as *mut (),
            cm.cmod_base.offset((*l).fileofs as isize) as *const (),
            (*l).filelen as usize,
        );
    }
}

/// Raven `CMod_LoadVisibility`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:454-472`
pub fn CMod_LoadVisibility(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    l: *mut lump_t,
    cmap: &mut clipMap_t,
) {
    unsafe {
        let len = (*l).filelen;
        if len == 0 {
            cmap.clusterBytes = (cmap.numClusters + 31) & !31;
            cmap.visibility =
                Hunk_Alloc(common, cm, rm, host, cmap.clusterBytes as usize, H_HIGH) as *mut u8;
            Com_Memset(cmap.visibility as *mut (), 255, cmap.clusterBytes as usize);
            return;
        }
        let buf = cm.cmod_base.offset((*l).fileofs as isize);

        cmap.vised = mp_qshared::shared::qtrue;
        cmap.visibility = Hunk_Alloc(common, cm, rm, host, len as usize, H_HIGH) as *mut u8;
        cmap.numClusters = i32::from_le(*(buf as *const c_int));
        cmap.clusterBytes = i32::from_le(*(buf.offset(4) as *const c_int));
        Com_Memcpy(
            cmap.visibility as *mut (),
            buf.offset(VIS_HEADER as isize) as *const (),
            len as usize - VIS_HEADER,
        );
    }
}

/// Raven `CM_ModelBounds`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:1020-1026`
pub fn CM_ModelBounds(cm: &mut CollisionWorld, model: clipHandle_t, mins: vec3_t, maxs: vec3_t) {
    unsafe {
        let cmod = CM_ClipHandleToModel(cm, model, core::ptr::null_mut());
        let _mins = (*cmod).mins;
        let _maxs = (*cmod).maxs;
    }
    // PORT-NOTE(shape-mismatch): `mins`/`maxs` are plain `vec3_t` (by value)
    // per the mechanically-resolved out-param convention (see
    // `CM_GetWorldBounds`); the write does not propagate to the caller in
    // this shape (reported in shape_mismatches).
}

/// Raven `CM_ModelContents_Actual`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:1138-1174`
pub fn CM_ModelContents_Actual(
    cm: &mut CollisionWorld,
    model: clipHandle_t,
    cmap: *mut clipMap_t,
) -> c_int {
    unsafe {
        let mut cmap = cmap;
        if cmap.is_null() {
            cmap = &mut cm.cmg as *mut clipMap_t;
        }

        let cmod = CM_ClipHandleToModel(cm, model, &mut cmap);

        let mut contents = 0;

        // MCG ADDED - return the contents, too
        if (*cmod).leaf.numLeafBrushes != 0 {
            // check for brush
            for i in (*cmod).leaf.firstLeafBrush
                ..((*cmod).leaf.firstLeafBrush + (*cmod).leaf.numLeafBrushes)
            {
                let brushNum = *(*cmap).leafbrushes.offset(i as isize);
                contents |= (*(*cmap).brushes.offset(brushNum as isize)).contents;
            }
        }
        if (*cmod).leaf.numLeafSurfaces != 0 {
            // if not brush, check for patch
            for i in (*cmod).leaf.firstLeafSurface
                ..((*cmod).leaf.firstLeafSurface + (*cmod).leaf.numLeafSurfaces)
            {
                let surfaceNum = *(*cmap).leafsurfaces.offset(i as isize);
                let surf = *(*cmap).surfaces.offset(surfaceNum as isize);
                if !surf.is_null() {
                    contents |= (*surf).contents;
                }
            }
        }
        contents
    }
}

/// Raven `CM_ModelContents`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:1176-1184`
pub fn CM_ModelContents(cm: &mut CollisionWorld, model: clipHandle_t, subBSPIndex: c_int) -> c_int {
    if subBSPIndex < 0 {
        return CM_ModelContents_Actual(cm, model, core::ptr::null_mut());
    }

    let sub = &mut cm.SubBSP[subBSPIndex as usize] as *mut clipMap_t;
    CM_ModelContents_Actual(cm, model, sub)
}

/// Raven `CMod_LoadPatches`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:483-536`
pub fn CMod_LoadPatches(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    surfs: *mut lump_t,
    verts: *mut lump_t,
    cmap: &mut clipMap_t,
) {
    unsafe {
        let mut points: [vec3_t; MAX_PATCH_VERTS] = [[0.0; 3]; MAX_PATCH_VERTS];

        let mut r#in = cm.cmod_base.offset((*surfs).fileofs as isize) as *mut dsurface_t;
        if (*surfs).filelen as usize % core::mem::size_of::<dsurface_t>() != 0 {
            com_error(
                errorParm_t::ERR_DROP,
                "MOD_LoadBmodel: funny lump size".into(),
            );
        }
        let count = (*surfs).filelen as usize / core::mem::size_of::<dsurface_t>();
        cmap.numSurfaces = count as c_int;
        cmap.surfaces = Hunk_Alloc(
            common,
            cm,
            rm,
            host,
            cmap.numSurfaces as usize * core::mem::size_of::<*mut cPatch_t>(),
            H_HIGH,
        ) as *mut *mut cPatch_t;

        let dv = cm.cmod_base.offset((*verts).fileofs as isize) as *mut drawVert_t;
        if (*verts).filelen as usize % core::mem::size_of::<drawVert_t>() != 0 {
            com_error(
                errorParm_t::ERR_DROP,
                "MOD_LoadBmodel: funny lump size".into(),
            );
        }

        // scan through all the surfaces, but only load patches, not planar faces
        for i in 0..count {
            let surf = r#in.add(i);
            if i32::from_le((*surf).surfaceType) != mapSurfaceType_t::MST_PATCH as i32 {
                continue; // ignore other surfaces
            }
            // FIXME: check for non-colliding patches

            let patch = Hunk_Alloc(
                common,
                cm,
                rm,
                host,
                core::mem::size_of::<cPatch_t>(),
                H_HIGH,
            ) as *mut cPatch_t;
            *cmap.surfaces.add(i) = patch;

            // load the full drawverts onto the stack
            let width = i32::from_le((*surf).patchWidth);
            let height = i32::from_le((*surf).patchHeight);
            let c = width * height;
            if c as usize > MAX_PATCH_VERTS {
                com_error(errorParm_t::ERR_DROP, "ParseMesh: MAX_PATCH_VERTS".into());
            }

            let mut dv_p = dv.offset(i32::from_le((*surf).firstVert) as isize);
            for j in 0..c as usize {
                points[j][0] = f32::from_le_bytes((*dv_p).xyz[0].to_le_bytes());
                points[j][1] = f32::from_le_bytes((*dv_p).xyz[1].to_le_bytes());
                points[j][2] = f32::from_le_bytes((*dv_p).xyz[2].to_le_bytes());
                dv_p = dv_p.offset(1);
            }

            let shaderNum = i32::from_le((*surf).shaderNum);
            (*patch).contents = (*cmap.shaders.offset(shaderNum as isize)).contentFlags;
            (*patch).surfaceFlags = (*cmap.shaders.offset(shaderNum as isize)).surfaceFlags;

            // create the internal facet structure
            (*patch).pc =
                CM_GeneratePatchCollide(common, cm, rm, host, width, height, points.as_mut_ptr());
        }
    }
}

/// Raven `CM_LoadMap_Actual`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:605-770`
// rwwRMG - function needs heavy modification
pub fn CM_LoadMap_Actual(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    name: *const c_char,
    clientload: qboolean,
    checksum: *mut c_int,
    cmap: &mut clipMap_t,
) {
    unsafe {
        if name.is_null() || *name == 0 {
            com_error(errorParm_t::ERR_DROP, "CM_LoadMap: NULL name".into());
        }

        cm.cm_noAreas = Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"cm_noAreas".as_ptr(),
            c"0".as_ptr(),
            mp_qshared::shared::cvar::CVAR_CHEAT,
        );
        cm.cm_noCurves = Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"cm_noCurves".as_ptr(),
            c"0".as_ptr(),
            mp_qshared::shared::cvar::CVAR_CHEAT,
        );
        cm.cm_playerCurveClip = Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"cm_playerCurveClip".as_ptr(),
            c"1".as_ptr(),
            mp_qshared::shared::cvar::CVAR_ARCHIVE | mp_qshared::shared::cvar::CVAR_CHEAT,
        );

        let name_cstr = std::ffi::CStr::from_ptr(name);
        Com_DPrintf(common, c"CM_LoadMap( %s, %i )\n".as_ptr(), name, clientload);

        let cmap_name = std::ffi::CStr::from_ptr(cmap.name.as_ptr());
        if cmap_name == name_cstr && clientload != 0 {
            *checksum = cm.last_checksum as c_int;
            return;
        }

        let orig_name = name_cstr.to_owned();

        if core::ptr::eq(cmap as *const clipMap_t, &cm.cmg as *const clipMap_t) {
            // free old stuff
            CM_ClearMap(cm, rmg);
            CM_ClearLevelPatches(cm);
        }

        // free old stuff
        Com_Memset(
            cmap as *mut clipMap_t as *mut (),
            0,
            core::mem::size_of::<clipMap_t>(),
        );

        if name_cstr.to_bytes().is_empty() {
            cmap.numLeafs = 1;
            cmap.numClusters = 1;
            cmap.numAreas = 1;
            cmap.cmodels = Hunk_Alloc(
                common,
                cm,
                rm,
                host,
                core::mem::size_of::<cmodel_s>(),
                H_HIGH,
            ) as *mut cmodel_s;
            *checksum = 0;
            return;
        }

        //
        // load the file
        //
        let mut buf: *mut c_int = core::ptr::null_mut();
        let mut new_buff: *mut () = core::ptr::null_mut();
        let mut h: fileHandle_t = 0;
        let bsp_len = FS_FOpenFileRead(
            common,
            cm,
            rm,
            host,
            name,
            &mut h,
            mp_qshared::shared::qfalse,
        );
        if h != 0 {
            new_buff = Z_Malloc(
                common,
                cm,
                rm,
                host,
                bsp_len,
                memtag_t::TAG_BSP_DISKIMAGE,
                mp_qshared::shared::qfalse,
                0,
            );
            FS_Read(common, new_buff, bsp_len, h);
            FS_FCloseFile(common, h);

            buf = new_buff as *mut c_int;
            if core::ptr::eq(cmap as *const clipMap_t, &cm.cmg as *const clipMap_t) {
                cm.gpvCachedMapDiskImage = new_buff;
                new_buff = core::ptr::null_mut();
            }
        }

        if buf.is_null() {
            com_error(
                errorParm_t::ERR_DROP,
                format!("Couldn't load {}", name_cstr.to_string_lossy()),
            );
        }

        cm.last_checksum =
            i32::from_le(Com_BlockChecksum(common, buf as *const (), bsp_len) as i32) as c_uint;
        *checksum = cm.last_checksum as c_int;

        let mut header: dheader_t = core::ptr::read(buf as *const dheader_t);
        {
            let header_words =
                core::slice::from_raw_parts_mut(&mut header as *mut dheader_t as *mut i32, 38);
            for w in header_words.iter_mut() {
                *w = i32::from_le(*w);
            }
        }

        if header.version != BSP_VERSION {
            Z_Free(common, cm.gpvCachedMapDiskImage);
            cm.gpvCachedMapDiskImage = core::ptr::null_mut();

            com_error(
                errorParm_t::ERR_DROP,
                format!(
                    "CM_LoadMap: {} has wrong version number ({} should be {})",
                    name_cstr.to_string_lossy(),
                    header.version,
                    BSP_VERSION
                ),
            );
        }

        cm.cmod_base = buf as *mut u8;

        // load into heap
        CMod_LoadShaders(common, cm, rm, host, &mut header.lumps[LUMP_SHADERS], cmap);
        CMod_LoadLeafs(common, cm, rm, host, &mut header.lumps[LUMP_LEAFS], cmap);
        CMod_LoadLeafBrushes(
            common,
            cm,
            rm,
            host,
            &mut header.lumps[LUMP_LEAFBRUSHES],
            cmap,
        );
        CMod_LoadLeafSurfaces(
            common,
            cm,
            rm,
            host,
            &mut header.lumps[LUMP_LEAFSURFACES],
            cmap,
        );
        CMod_LoadPlanes(common, cm, rm, host, &mut header.lumps[LUMP_PLANES], cmap);
        CMod_LoadBrushSides(
            common,
            cm,
            rm,
            host,
            &mut header.lumps[LUMP_BRUSHSIDES],
            cmap,
        );
        CMod_LoadBrushes(common, cm, rm, host, &mut header.lumps[LUMP_BRUSHES], cmap);
        CMod_LoadSubmodels(common, cm, rm, host, &mut header.lumps[LUMP_MODELS], cmap);
        CMod_LoadNodes(common, cm, rm, host, &mut header.lumps[LUMP_NODES], cmap);
        CMod_LoadEntityString(common, cm, rm, host, &mut header.lumps[LUMP_ENTITIES], cmap);
        CMod_LoadVisibility(
            common,
            cm,
            rm,
            host,
            &mut header.lumps[LUMP_VISIBILITY],
            cmap,
        );
        CMod_LoadPatches(
            common,
            cm,
            rm,
            host,
            &mut header.lumps[LUMP_SURFACES],
            &mut header.lumps[LUMP_DRAWVERTS],
            cmap,
        );

        cm.TotalSubModels += cmap.numSubModels;

        if core::ptr::eq(cmap as *const clipMap_t, &cm.cmg as *const clipMap_t) {
            // Load in the shader text - return instantly if already loaded
            CM_LoadShaderText(common, cm, rm, rmg, host, mp_qshared::shared::qfalse);
            CM_InitBoxHull(cm);
            CM_SetupShaderProperties(common, cm, rmg, host);
        }

        //
        // if we've got enough memory, and it's not a dedicated-server, then
        // keep the loaded map binary around for the renderer to chew on...
        // (but not if this gets ported to a big-endian machine, because some
        // of the map data will have been Little-Long'd, but some hasn't).
        //
        if Sys_LowPhysicalMemory() != 0 || (*common.com_dedicated).integer != 0 {
            Z_Free(common, cm.gpvCachedMapDiskImage);
            cm.gpvCachedMapDiskImage = core::ptr::null_mut();
        } else {
            // ... do nothing, and let the renderer free it after it's finished
            // playing with it...
        }

        CM_FloodAreaConnections(cmap);

        // allow this to be cached if it is loaded by the server
        if clientload == 0 {
            Q_strncpyz(
                cmap.name.as_mut_ptr(),
                orig_name.as_ptr(),
                MAX_QPATH as c_int,
            );
        }
    }
}

/// Raven `CM_LoadMap`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:775-782`
pub fn CM_LoadMap(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    name: *const c_char,
    clientload: qboolean,
    checksum: *mut c_int,
) {
    cm.gbUsingCachedMapDataRightNow = mp_qshared::shared::qtrue; // !!!!!!!!!!!!!!!!!!

    let cmg_ptr = &mut cm.cmg as *mut clipMap_t;
    unsafe {
        CM_LoadMap_Actual(
            common,
            cm,
            rm,
            rmg,
            host,
            name,
            clientload,
            checksum,
            &mut *cmg_ptr,
        );
    }

    cm.gbUsingCachedMapDataRightNow = mp_qshared::shared::qfalse; // !!!!!!!!!!!!!!!!!!
}

/// Raven `CM_LoadSubBSP`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:1083-1108`
pub fn CM_LoadSubBSP(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    name: *const c_char,
    clientload: qboolean,
) -> c_int {
    unsafe {
        let mut count = cm.cmg.numSubModels;
        for i in 0..cm.NumSubBSP {
            let sub_name = cm.SubBSP[i as usize].name.as_ptr();
            if libc::strcasecmp(name, sub_name) == 0 {
                return count;
            }
            count += cm.SubBSP[i as usize].numSubModels;
        }

        if cm.NumSubBSP == MAX_SUB_BSP {
            com_error(
                errorParm_t::ERR_DROP,
                "CM_LoadSubBSP: too many unique sub BSPs".into(),
            );
        }

        let idx = cm.NumSubBSP;
        let sub_ptr = &mut cm.SubBSP[idx as usize] as *mut clipMap_t;
        let mut dummy_checksum: c_int = 0;
        CM_LoadMap_Actual(
            common,
            cm,
            rm,
            rmg,
            host,
            name,
            clientload,
            &mut dummy_checksum,
            &mut *sub_ptr,
        );
        cm.NumSubBSP += 1;

        count
    }
}

/// Raven `CM_RegisterTerrain`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:1036-1057`
pub fn CM_RegisterTerrain(
    cm: &mut CollisionWorld,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    config: *const c_char,
    server: bool,
) -> *mut CCMLandScape {
    unsafe {
        if !cm.cmg.landScape.is_null() {
            // Already spawned so just return
            let ls = cm.cmg.landScape;
            CCMLandScape_IncreaseRefCount(ls);
            return ls;
        }
        // Doesn't exist so create and link in
        let ls = CM_InitTerrain(rmg, host, config, 0, server);

        // Increment for the next instance
        if !cm.cmg.landScape.is_null() {
            com_error(
                errorParm_t::ERR_DROP,
                "You cannot have more than one terrain brush.\n".into(),
            );
        }
        cm.cmg.landScape = ls;
        ls
    }
}
