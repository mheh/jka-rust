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
//! `RenderModels` below is still a unit placeholder — its real shape is owned
//! by the renderer-model wave (tr-model.md); `RmManager` threads as the
//! opaque-slot re-export, cast back at the server boundary.

use core::ffi::{c_char, c_int, c_uint};

use mp_qshared::common::mp::qcommon::tags::memtag_t;
use mp_qshared::shared::collision::cplane_t;
use mp_qshared::shared::cvar::{CVAR_ARCHIVE, CVAR_CHEAT};
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::limits::MAX_SUB_BSP;
use mp_qshared::shared::{qboolean, vec3_t, MAX_QPATH};
use native_types::{clipHandle_t, fileHandle_t, thandle_t};

use crate::common::engine_host_view::EngineHostView;

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

/// `RenderModels` — the type-erased opaque slot for `mp_renderer`'s real
/// `RenderModels` (the FROZEN `tr-model.md` model registry, owned by
/// `Engine.render_models`), threaded (never dereferenced) by cm_load/server.
/// The owning server crate casts it back at its boundary (`rm_from_slot`).
/// Named here because the cm_load/server threading refers to
/// `cm_load::RenderModels`. Same treatment as the sibling `RmManager` slot.
///
/// Ruling: opaque-slot (user, 2026-07-12, option A).
pub use crate::common::opaque_slots::RenderModels;

/// `RmManager` — the type-erased opaque slot for `mp_engine_rmg`'s real
/// `RmManager`, threaded (never dereferenced) by cm_load/server. The owning
/// server crate casts it back at its boundary (`rmg_from_slot`). Named here
/// because the cm_load/server threading refers to `cm_load::RmManager`.
///
/// Ruling: opaque-slot (user, 2026-07-12, option A).
pub use crate::common::opaque_slots::RmManager;

// Unused opaque placeholder; the real port of Raven's `CCMLandScape` is
// `crate::cm_terrain::CmLandScape`, used directly elsewhere in this crate.
#[allow(dead_code)]
pub struct CCMLandScape;

// Opaque placeholder for Raven's `CRMManager`/`TheRandomMissionManager`; the
// real port is `mp_engine_rmg::rm_manager::RmManager`, not yet wired into qcommon.
#[allow(dead_code)]
pub struct CRMManager;

// Sweep: extern forward-declares eliminated. Real in-crate/qshared callees
// imported. Genuinely-unported callees (this crate's own not-yet-ported
// cm/files/zone functions, cvar/platform/q_math gaps, the §F rmg-terrain
// C++ shims) referenced at their canonical homes or left bare; reported.
use crate::cm_shader::{CM_LoadShaderText, CM_SetupShaderProperties, CM_ShutdownShaderProperties};
use crate::cm_test::CM_FloodAreaConnections;
use crate::md4_fns::Com_BlockChecksum;
use crate::z_memman_pc::Hunk_Alloc;
use mp_qshared::shared::ha_pref;
use native_string::q_strncpyz::Q_strncpyzBytes;

use crate::cm_patch_fns::{CM_ClearLevelPatches, CM_GeneratePatchCollide};
use crate::common_fns::Com_DPrintf;
use crate::cvar_fns::Cvar_Get;
use crate::files_common::{FS_FCloseFile, FS_FOpenFileRead, FS_Read};
use crate::z_memman_pc::{Z_Free, Z_Malloc};
use mp_qshared::shared::q_math::{PlaneTypeForNormal, SetPlaneSignbits};
use native_platform::Sys_LowPhysicalMemory;

/// Raven `CM_BoundBrush`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:211-220`
pub fn CM_BoundBrush(b: *mut cbrush_t) {
    unsafe {
        (*b).bounds[0][0] = -(*(*b).sides).plane.as_ref().unwrap().dist;
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

/// Raven `CM_LeafCluster`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:907-912`
pub fn CM_LeafCluster(cm: &mut CollisionWorld, leafnum: c_int) -> c_int {
    unsafe {
        if leafnum < 0 || leafnum >= cm.cmg.numLeafs {
            com_error(
                errorParm_t::ERR_DROP,
                "CM_LeafCluster: bad number".to_string(),
            );
        }
        (*cm.cmg.leafs.offset(leafnum as isize)).cluster
    }
}

/// Raven `CM_LeafArea`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:914-919`
pub fn CM_LeafArea(cm: &mut CollisionWorld, leafnum: c_int) -> c_int {
    unsafe {
        if leafnum < 0 || leafnum >= cm.cmg.numLeafs {
            com_error(errorParm_t::ERR_DROP, "CM_LeafArea: bad number".to_string());
        }
        (*cm.cmg.leafs.offset(leafnum as isize)).area
    }
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
            (*p).r#type = (i >> 1) as u8;
            (*p).signbits = 0;
            (*p).normal = [0.0; 3];
            (*p).normal[(i >> 1) as usize] = 1.0;

            let p = cm.box_planes.offset((i * 2 + 1) as isize);
            (*p).r#type = (3 + (i >> 1)) as u8;
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
    // Raven ignores `terrainId` (the landscape is the singleton `cmg.landScape`).
    let _ = terrainId;
    // Raven `DecreaseRefCount()` then free at `GetRefCount() <= 0`. `mRefCount` is
    // §20-dropped (renderer-only reader, its sole caller is DEC-01-deferred —
    // rmg-terrain.md RMG-D4c), so the single owner drops unconditionally. The owner
    // is `cm.land_scape: Option<CmLandScape>` (rmg-terrain.md state table), not the
    // raw `cmg.landScape` pointer.
    if cm.land_scape.is_some() {
        cm.land_scape = None;
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
pub fn CM_GetWorldBounds(cm: &mut CollisionWorld, mins: &mut vec3_t, maxs: &mut vec3_t) {
    unsafe {
        *mins = (*cm.cmg.cmodels.offset(0)).mins;
        *maxs = (*cm.cmg.cmodels.offset(0)).maxs;
    }
}

/// Raven `CM_ClearMap`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:791-821`
pub fn CM_ClearMap(cm: &mut CollisionWorld, rmg: &mut RmManager) {
    unsafe {
        CM_ShutdownShaderProperties(cm);

        if !cm.TheRandomMissionManager.is_null() {
            // Raven `delete TheRandomMissionManager; = 0`. The RMG manager's owned
            // Rust home is `Engine.rmg: RmManager` (wave-20 — rmg-terrain.md state
            // table); this raw `CollisionWorld` slot is never allocated, so the drop
            // is a plain null.
            cm.TheRandomMissionManager = core::ptr::null_mut();
        }

        if cm.land_scape.is_some() {
            // Raven `delete cmg.landScape; = NULL` — the unconditional teardown free
            // (RMG-D4c). Owner is `cm.land_scape: Option<CmLandScape>` (rmg-terrain.md
            // state table); the raw `cmg.landScape` pointer is zeroed by the memset
            // below.
            cm.land_scape = None;
        }

        cm.cmg = clipMap_t::default();
        CM_ClearLevelPatches(cm);

        for i in 0..cm.NumSubBSP {
            cm.SubBSP[i as usize] = clipMap_t::default();
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
            unsafe {
                Z_Free(common, cm.gpvCachedMapDiskImage);
            }
            cm.gpvCachedMapDiskImage = core::ptr::null_mut();

            bActuallyFreedSomething = mp_qshared::shared::qtrue;
        }
        cm.gsCachedMapDiskImage[0] = 0;

        // force map loader to ignore cached internal BSP structures for next
        // level CM_LoadMap() call...
        cm.cmg.name.clear();
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
pub fn CMod_LoadShaders(view: &mut EngineHostView, l: *mut lump_t, cmap: &mut clipMap_t) {
    unsafe {
        let mut r#in = view.cm.cmod_base.offset((*l).fileofs as isize) as *mut dshader_t;
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
            view,
            ((1 + count) * core::mem::size_of::<CCMShader>()) as c_int,
            ha_pref::h_high,
        ) as *mut CCMShader;
        cmap.numShaders = count as c_int;

        let mut out = cmap.shaders;
        for _ in 0..count {
            // C-data site (raw BSP bytes): the byte-exact Bytes form.
            Q_strncpyzBytes(
                &mut (*out).shader,
                core::slice::from_raw_parts((*r#in).shader.as_ptr() as *const u8, MAX_QPATH),
                MAX_QPATH,
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
pub fn CMod_LoadSubmodels(view: &mut EngineHostView, l: *mut lump_t, cmap: &mut clipMap_t) {
    unsafe {
        let mut r#in = view.cm.cmod_base.offset((*l).fileofs as isize) as *mut dmodel_t;
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
            view,
            (count * core::mem::size_of::<cmodel_s>()) as c_int,
            ha_pref::h_high,
        ) as *mut cmodel_s;
        cmap.numSubModels = count as c_int;

        if count > MAX_SUBMODELS {
            com_error(errorParm_t::ERR_DROP, "MAX_SUBMODELS exceeded".into());
        }

        // §19: Raven stores `indexes - cm.leafbrushes` — a pointer difference
        // between two SEPARATE Hunk_Alloc blocks — into the i32
        // `leaf.firstLeafBrush`, relying on its contiguous hunk to keep the
        // diff small. Our hunk is malloc-backed (blocks arbitrarily far
        // apart), so the diff truncates and the reconstruction reads wild
        // memory (live SIGBUS on mp/duel1's func_bobbing). The one defined
        // behavior with identical semantics: grow the leafbrushes/leafsurfaces
        // arrays and APPEND each submodel's index block, so the stored value
        // is a genuine index. `numLeafBrushes`/`numLeafSurfaces` keep the map
        // lump's counts, exactly as Raven's do.
        let mut extra_brushes: usize = 0;
        let mut extra_surfaces: usize = 0;
        {
            let mut probe = r#in;
            for i in 0..count {
                if !(i == 0
                    && core::ptr::eq(cmap as *const clipMap_t, &view.cm.cmg as *const clipMap_t))
                {
                    extra_brushes += (*probe).numBrushes.max(0) as usize;
                    extra_surfaces += (*probe).numSurfaces.max(0) as usize;
                }
                probe = probe.offset(1);
            }
        }
        let old_lb = cmap.numLeafBrushes.max(0) as usize;
        let new_lb = Hunk_Alloc(
            view,
            ((old_lb + extra_brushes) * 4) as c_int,
            ha_pref::h_high,
        ) as *mut c_int;
        core::ptr::copy_nonoverlapping(cmap.leafbrushes, new_lb, old_lb);
        cmap.leafbrushes = new_lb;
        let mut next_lb = old_lb;
        let old_ls = cmap.numLeafSurfaces.max(0) as usize;
        let new_ls = Hunk_Alloc(
            view,
            ((old_ls + extra_surfaces) * 4) as c_int,
            ha_pref::h_high,
        ) as *mut c_int;
        core::ptr::copy_nonoverlapping(cmap.leafsurfaces, new_ls, old_ls);
        cmap.leafsurfaces = new_ls;
        let mut next_ls = old_ls;

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
            if i == 0 && core::ptr::eq(cmap as *const clipMap_t, &view.cm.cmg as *const clipMap_t) {
                (*out).firstNode = 0;
                r#in = r#in.offset(1);
                continue; // world model doesn't need other info
            }

            // make a "leaf" just to hold the model's brushes and surfaces
            (*out).firstNode = -1;

            (*out).leaf.numLeafBrushes = (*r#in).numBrushes;
            let indexes = cmap.leafbrushes.add(next_lb);
            (*out).leaf.firstLeafBrush = next_lb as c_int;
            for j in 0..(*out).leaf.numLeafBrushes {
                *indexes.offset(j as isize) = (*r#in).firstBrush + j;
            }
            next_lb += (*out).leaf.numLeafBrushes.max(0) as usize;

            (*out).leaf.numLeafSurfaces = (*r#in).numSurfaces;
            let indexes = cmap.leafsurfaces.add(next_ls);
            (*out).leaf.firstLeafSurface = next_ls as c_int;
            for j in 0..(*out).leaf.numLeafSurfaces {
                *indexes.offset(j as isize) = (*r#in).firstSurface + j;
            }
            next_ls += (*out).leaf.numLeafSurfaces.max(0) as usize;

            r#in = r#in.offset(1);
        }
    }
}

/// Raven `CMod_LoadNodes`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:175-203`
pub fn CMod_LoadNodes(view: &mut EngineHostView, l: *mut lump_t, cmap: &mut clipMap_t) {
    unsafe {
        let mut r#in = view.cm.cmod_base.offset((*l).fileofs as isize) as *mut dnode_t;
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
            view,
            (count * core::mem::size_of::<cNode_t>()) as c_int,
            ha_pref::h_high,
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
pub fn CMod_LoadBrushes(view: &mut EngineHostView, l: *mut lump_t, cmap: &mut clipMap_t) {
    unsafe {
        let mut r#in = view.cm.cmod_base.offset((*l).fileofs as isize) as *mut dbrush_t;
        if (*l).filelen as usize % core::mem::size_of::<dbrush_t>() != 0 {
            com_error(
                errorParm_t::ERR_DROP,
                "MOD_LoadBmodel: funny lump size".into(),
            );
        }
        let count = (*l).filelen as usize / core::mem::size_of::<dbrush_t>();

        cmap.brushes = Hunk_Alloc(
            view,
            ((BOX_BRUSHES + count) * core::mem::size_of::<cbrush_t>()) as c_int,
            ha_pref::h_high,
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
pub fn CMod_LoadLeafs(view: &mut EngineHostView, l: *mut lump_t, cmap: &mut clipMap_t) {
    unsafe {
        let mut r#in = view.cm.cmod_base.offset((*l).fileofs as isize) as *mut dleaf_t;
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
            view,
            ((BOX_LEAFS + count) * core::mem::size_of::<cLeaf_t>()) as c_int,
            ha_pref::h_high,
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
            view,
            (cmap.numAreas as usize * core::mem::size_of::<cArea_t>()) as c_int,
            ha_pref::h_high,
        ) as *mut cArea_t;
        cmap.areaPortals = Hunk_Alloc(
            view,
            (cmap.numAreas as usize * cmap.numAreas as usize * core::mem::size_of::<c_int>())
                as c_int,
            ha_pref::h_high,
        ) as *mut c_int;
    }
}

/// Raven `CMod_LoadPlanes`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:312-346`
pub fn CMod_LoadPlanes(view: &mut EngineHostView, l: *mut lump_t, cmap: &mut clipMap_t) {
    unsafe {
        let mut r#in = view.cm.cmod_base.offset((*l).fileofs as isize) as *mut dplane_t;
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
            view,
            ((BOX_PLANES + count) * core::mem::size_of::<cplane_t>()) as c_int,
            ha_pref::h_high,
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
            (*out).r#type = PlaneTypeForNormal((*out).normal) as u8;
            (*out).signbits = bits;

            out = out.offset(1);
            r#in = r#in.offset(1);
        }
    }
}

/// Raven `CMod_LoadLeafBrushes`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:353-373`
pub fn CMod_LoadLeafBrushes(view: &mut EngineHostView, l: *mut lump_t, cmap: &mut clipMap_t) {
    unsafe {
        let mut r#in = view.cm.cmod_base.offset((*l).fileofs as isize) as *mut c_int;
        if (*l).filelen as usize % core::mem::size_of::<c_int>() != 0 {
            com_error(
                errorParm_t::ERR_DROP,
                "MOD_LoadBmodel: funny lump size".into(),
            );
        }
        let count = (*l).filelen as usize / core::mem::size_of::<c_int>();

        cmap.leafbrushes = Hunk_Alloc(
            view,
            ((count + BOX_BRUSHES) * core::mem::size_of::<c_int>()) as c_int,
            ha_pref::h_high,
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
pub fn CMod_LoadLeafSurfaces(view: &mut EngineHostView, l: *mut lump_t, cmap: &mut clipMap_t) {
    unsafe {
        let mut r#in = view.cm.cmod_base.offset((*l).fileofs as isize) as *mut c_int;
        if (*l).filelen as usize % core::mem::size_of::<c_int>() != 0 {
            com_error(
                errorParm_t::ERR_DROP,
                "MOD_LoadBmodel: funny lump size".into(),
            );
        }
        let count = (*l).filelen as usize / core::mem::size_of::<c_int>();

        cmap.leafsurfaces = Hunk_Alloc(
            view,
            (count * core::mem::size_of::<c_int>()) as c_int,
            ha_pref::h_high,
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
pub fn CMod_LoadBrushSides(view: &mut EngineHostView, l: *mut lump_t, cmap: &mut clipMap_t) {
    unsafe {
        let mut r#in = view.cm.cmod_base.offset((*l).fileofs as isize) as *mut dbrushside_t;
        if (*l).filelen as usize % core::mem::size_of::<dbrushside_t>() != 0 {
            com_error(
                errorParm_t::ERR_DROP,
                "MOD_LoadBmodel: funny lump size".into(),
            );
        }
        let count = (*l).filelen as usize / core::mem::size_of::<dbrushside_t>();

        cmap.brushsides = Hunk_Alloc(
            view,
            ((BOX_SIDES + count) * core::mem::size_of::<cbrushside_t>()) as c_int,
            ha_pref::h_high,
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
pub fn CMod_LoadEntityString(view: &mut EngineHostView, l: *mut lump_t, cmap: &mut clipMap_t) {
    unsafe {
        cmap.entityString =
            Hunk_Alloc(view, ((*l).filelen as usize) as c_int, ha_pref::h_high) as *mut c_char;
        cmap.numEntityChars = (*l).filelen;
        Com_Memcpy(
            cmap.entityString as *mut (),
            view.cm.cmod_base.offset((*l).fileofs as isize) as *const (),
            (*l).filelen as usize,
        );
    }
}

/// Raven `CMod_LoadVisibility`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:454-472`
pub fn CMod_LoadVisibility(view: &mut EngineHostView, l: *mut lump_t, cmap: &mut clipMap_t) {
    unsafe {
        let len = (*l).filelen;
        if len == 0 {
            cmap.clusterBytes = (cmap.numClusters + 31) & !31;
            cmap.visibility =
                Hunk_Alloc(view, (cmap.clusterBytes as usize) as c_int, ha_pref::h_high) as *mut u8;
            Com_Memset(cmap.visibility as *mut (), 255, cmap.clusterBytes as usize);
            return;
        }
        let buf = view.cm.cmod_base.offset((*l).fileofs as isize);

        cmap.vised = true;
        cmap.visibility = Hunk_Alloc(view, (len as usize) as c_int, ha_pref::h_high) as *mut u8;
        cmap.numClusters = i32::from_le(*(buf as *const c_int));
        cmap.clusterBytes = i32::from_le(*(buf.offset(4) as *const c_int));
        Com_Memcpy(
            cmap.visibility as *mut (),
            buf.offset(VIS_HEADER as isize) as *const (),
            len as usize - VIS_HEADER,
        );
    }
}

/// Raven `CM_ModelBounds` — real out-params: the mechanically-resolved
/// by-value shape silently dropped the bounds, zeroing every brush entity's
/// `r.mins`/`r.maxs` and the world-sector tree (found live: dead
/// `trigger_hurt` volumes, 2026-07-13).
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:1020-1026`
pub fn CM_ModelBounds(
    cm: &mut CollisionWorld,
    model: clipHandle_t,
    mins: &mut vec3_t,
    maxs: &mut vec3_t,
) {
    unsafe {
        let cmod = CM_ClipHandleToModel(cm, model, core::ptr::null_mut());
        *mins = (*cmod).mins;
        *maxs = (*cmod).maxs;
    }
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
    view: &mut EngineHostView,
    surfs: *mut lump_t,
    verts: *mut lump_t,
    cmap: &mut clipMap_t,
) {
    unsafe {
        let mut points: [vec3_t; MAX_PATCH_VERTS] = [[0.0; 3]; MAX_PATCH_VERTS];

        let mut r#in = view.cm.cmod_base.offset((*surfs).fileofs as isize) as *mut dsurface_t;
        if (*surfs).filelen as usize % core::mem::size_of::<dsurface_t>() != 0 {
            com_error(
                errorParm_t::ERR_DROP,
                "MOD_LoadBmodel: funny lump size".into(),
            );
        }
        let count = (*surfs).filelen as usize / core::mem::size_of::<dsurface_t>();
        cmap.numSurfaces = count as c_int;
        cmap.surfaces = Hunk_Alloc(
            view,
            (cmap.numSurfaces as usize * core::mem::size_of::<*mut cPatch_t>()) as c_int,
            ha_pref::h_high,
        ) as *mut *mut cPatch_t;

        let dv = view.cm.cmod_base.offset((*verts).fileofs as isize) as *mut drawVert_t;
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
                view,
                (core::mem::size_of::<cPatch_t>()) as c_int,
                ha_pref::h_high,
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
            (*patch).pc = CM_GeneratePatchCollide(view, width, height, points.as_mut_ptr());
        }
    }
}

/// Raven `CM_LoadMap_Actual`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:605-770`
// rwwRMG - function needs heavy modification
pub fn CM_LoadMap_Actual(
    view: &mut EngineHostView,
    name: &str,
    clientload: qboolean,
    checksum: *mut c_int,
    cmap: &mut clipMap_t,
) {
    unsafe {
        if name.is_empty() {
            com_error(errorParm_t::ERR_DROP, "CM_LoadMap: NULL name".into());
        }

        view.cm.cm_noAreas = Some(Cvar_Get(view, "cm_noAreas", "0", CVAR_CHEAT));
        view.cm.cm_noCurves = Some(Cvar_Get(view, "cm_noCurves", "0", CVAR_CHEAT));
        view.cm.cm_playerCurveClip = Some(Cvar_Get(
            view,
            "cm_playerCurveClip",
            "1",
            CVAR_ARCHIVE | CVAR_CHEAT,
        ));

        Com_DPrintf(
            view.common,
            &format!("CM_LoadMap( {name}, {clientload} )\n"),
        );

        if cmap.name == name && clientload != 0 {
            *checksum = view.cm.last_checksum as c_int;
            return;
        }

        // Raven snapshots `origName` before the map data is torn down.
        let orig_name = name.to_string();

        if core::ptr::eq(cmap as *const clipMap_t, &view.cm.cmg as *const clipMap_t) {
            // free old stuff
            CM_ClearMap(view.cm, &mut view.rmg);
            CM_ClearLevelPatches(view.cm);
        }

        // free old stuff
        *cmap = clipMap_t::default();

        if name.is_empty() {
            cmap.numLeafs = 1;
            cmap.numClusters = 1;
            cmap.numAreas = 1;
            cmap.cmodels = Hunk_Alloc(
                view,
                (core::mem::size_of::<cmodel_s>()) as c_int,
                ha_pref::h_high,
            ) as *mut cmodel_s;
            *checksum = 0;
            return;
        }

        //
        // load the file
        //
        let mut buf: *mut c_int = core::ptr::null_mut();
        let new_buff: *mut ();
        let mut h: fileHandle_t = 0;
        let bsp_len = FS_FOpenFileRead(view, name, &mut h, false);
        if h != 0 {
            new_buff = Z_Malloc(
                view,
                bsp_len,
                memtag_t::TAG_BSP_DISKIMAGE,
                mp_qshared::shared::qfalse,
                0,
            );
            FS_Read(view.common, new_buff, bsp_len, h);
            FS_FCloseFile(view.common, h);

            buf = new_buff as *mut c_int;
            if core::ptr::eq(cmap as *const clipMap_t, &view.cm.cmg as *const clipMap_t) {
                view.cm.gpvCachedMapDiskImage = new_buff;
            }
        }

        if buf.is_null() {
            com_error(errorParm_t::ERR_DROP, format!("Couldn't load {name}"));
        }

        view.cm.last_checksum =
            i32::from_le(Com_BlockChecksum(view.common, buf as *const (), bsp_len) as i32)
                as c_uint;
        *checksum = view.cm.last_checksum as c_int;

        let mut header: dheader_t = core::ptr::read(buf as *const dheader_t);
        {
            let header_words =
                core::slice::from_raw_parts_mut(&mut header as *mut dheader_t as *mut i32, 38);
            for w in header_words.iter_mut() {
                *w = i32::from_le(*w);
            }
        }

        if header.version != BSP_VERSION {
            unsafe {
                Z_Free(view.common, view.cm.gpvCachedMapDiskImage);
            }
            view.cm.gpvCachedMapDiskImage = core::ptr::null_mut();

            com_error(
                errorParm_t::ERR_DROP,
                format!(
                    "CM_LoadMap: {name} has wrong version number ({} should be {})",
                    header.version, BSP_VERSION
                ),
            );
        }

        view.cm.cmod_base = buf as *mut u8;

        // load into heap
        CMod_LoadShaders(view, &mut header.lumps[LUMP_SHADERS], cmap);
        CMod_LoadLeafs(view, &mut header.lumps[LUMP_LEAFS], cmap);
        CMod_LoadLeafBrushes(view, &mut header.lumps[LUMP_LEAFBRUSHES], cmap);
        CMod_LoadLeafSurfaces(view, &mut header.lumps[LUMP_LEAFSURFACES], cmap);
        CMod_LoadPlanes(view, &mut header.lumps[LUMP_PLANES], cmap);
        CMod_LoadBrushSides(view, &mut header.lumps[LUMP_BRUSHSIDES], cmap);
        CMod_LoadBrushes(view, &mut header.lumps[LUMP_BRUSHES], cmap);
        CMod_LoadSubmodels(view, &mut header.lumps[LUMP_MODELS], cmap);
        CMod_LoadNodes(view, &mut header.lumps[LUMP_NODES], cmap);
        CMod_LoadEntityString(view, &mut header.lumps[LUMP_ENTITIES], cmap);
        CMod_LoadVisibility(view, &mut header.lumps[LUMP_VISIBILITY], cmap);
        CMod_LoadPatches(
            view,
            &mut header.lumps[LUMP_SURFACES],
            &mut header.lumps[LUMP_DRAWVERTS],
            cmap,
        );

        view.cm.TotalSubModels += cmap.numSubModels;

        if core::ptr::eq(cmap as *const clipMap_t, &view.cm.cmg as *const clipMap_t) {
            // Load in the shader text - return instantly if already loaded
            CM_LoadShaderText(view, mp_qshared::shared::qfalse);
            CM_InitBoxHull(view.cm);
            CM_SetupShaderProperties(view);
        }

        //
        // if we've got enough memory, and it's not a dedicated-server, then
        // keep the loaded map binary around for the renderer to chew on...
        // (but not if this gets ported to a big-endian machine, because some
        // of the map data will have been Little-Long'd, but some hasn't).
        //
        if Sys_LowPhysicalMemory() != 0 || view.common.cvar(view.common.com_dedicated).integer != 0
        {
            unsafe {
                Z_Free(view.common, view.cm.gpvCachedMapDiskImage);
            }
            view.cm.gpvCachedMapDiskImage = core::ptr::null_mut();
        } else {
            // ... do nothing, and let the renderer free it after it's finished
            // playing with it...
        }

        CM_FloodAreaConnections(cmap);

        // allow this to be cached if it is loaded by the server
        if clientload == 0 {
            cmap.name = orig_name;
        }
    }
}

/// Raven `CM_LoadMap`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:775-782`
pub fn CM_LoadMap(
    view: &mut EngineHostView,
    name: &str,
    clientload: qboolean,
    checksum: *mut c_int,
) {
    view.cm.gbUsingCachedMapDataRightNow = mp_qshared::shared::qtrue; // !!!!!!!!!!!!!!!!!!

    let cmg_ptr = &mut view.cm.cmg as *mut clipMap_t;
    unsafe {
        CM_LoadMap_Actual(view, name, clientload, checksum, &mut *cmg_ptr);
    }

    view.cm.gbUsingCachedMapDataRightNow = mp_qshared::shared::qfalse; // !!!!!!!!!!!!!!!!!!
}

/// Raven `CM_LoadSubBSP`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:1083-1108`
pub fn CM_LoadSubBSP(view: &mut EngineHostView, name: &str, clientload: qboolean) -> c_int {
    unsafe {
        let mut count = view.cm.cmg.numSubModels;
        for i in 0..view.cm.NumSubBSP {
            // Raven strcasecmp.
            if view.cm.SubBSP[i as usize].name.eq_ignore_ascii_case(name) {
                return count;
            }
            count += view.cm.SubBSP[i as usize].numSubModels;
        }

        if view.cm.NumSubBSP == MAX_SUB_BSP {
            com_error(
                errorParm_t::ERR_DROP,
                "CM_LoadSubBSP: too many unique sub BSPs".into(),
            );
        }

        let idx = view.cm.NumSubBSP;
        let sub_ptr = &mut view.cm.SubBSP[idx as usize] as *mut clipMap_t;
        let mut dummy_checksum: c_int = 0;
        CM_LoadMap_Actual(view, name, clientload, &mut dummy_checksum, &mut *sub_ptr);
        view.cm.NumSubBSP += 1;

        count
    }
}

// Raven `CM_RegisterTerrain` (`cm_load.cpp:1036-1057`) + `CM_InitTerrain`
// (`cm_terrain.cpp:1618-1626`) are the §F design's `register_terrain` — the
// get-or-create over `cm.land_scape: Option<CmLandScape>` that folds the
// `CCMLandScape` construction and `SetTerrainId(0)` and returns a `TerrainHandle`
// (`crate::cm_terrain::register_terrain`, rmg-terrain.md Seam-B / RMG-D4c). The
// wave-20 `G_CM_REGISTER_TERRAIN` syscall arm calls that one; this C-track
// duplicate is dropped (porting-rules §20 — zero engine callers, superseded).
