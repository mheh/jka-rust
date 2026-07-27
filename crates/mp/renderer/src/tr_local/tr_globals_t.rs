#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_float, c_int, c_uint};

use mp_qshared::shared::limits::MAX_SUB_BSP as MAX_SUB_BSP_I32;
use mp_qshared::shared::{qboolean, vec3_t};

use super::front_end_counters_t::frontEndCounters_t;
use super::image_s::image_t;
use super::model_s::model_t;
use super::orientationr_t::orientationr_t;
use super::shader_s::shader_t;
use super::skin_s::skin_t;
use super::srf_terrain_s::srfTerrain_t;
use super::tr_ref_entity_t::trRefEntity_t;
use super::tr_refdef_t::trRefdef_t;
use super::view_parms_t::viewParms_t;
use super::world_t::world_t;

/// `NUM_SCRATCH_IMAGES` (non-`_XBOX` branch).
/// Source: `oracle/codemp/renderer/tr_local.h:1300-1307`
const NUM_SCRATCH_IMAGES: usize = 16;

/// `MAX_LIGHTMAPS`.
/// Source: `oracle/codemp/renderer/tr_local.h:1203`
const MAX_LIGHTMAPS: usize = 256;

/// `MAX_MOD_KNOWN`.
/// Source: `oracle/codemp/renderer/tr_local.h:1138`
pub(crate) const MAX_MOD_KNOWN: usize = 1024;

/// `MAX_SUB_BSP` — rwwRMG - added. `usize`-typed dual of the canonical
/// `mp_qshared::shared::limits::MAX_SUB_BSP` (`c_int`), for array sizing.
/// Source: `oracle/codemp/game/q_shared.h:2025`
const MAX_SUB_BSP: usize = MAX_SUB_BSP_I32 as usize;

/// `MAX_SHADERS` (non-`_XBOX` branch; 14 bits, see `QSORT_SHADERNUM_SHIFT`).
/// Source: `oracle/codemp/renderer/tr_local.h:40-46`
const MAX_SHADERS: usize = 16384;

/// `MAX_SKINS`.
/// Source: `oracle/codemp/renderer/tr_local.h:1204`
pub(crate) const MAX_SKINS: usize = 1024;

/// `FUNCTABLE_SIZE`.
/// Source: `oracle/codemp/renderer/tr_local.h:1247`
pub const FUNCTABLE_SIZE: usize = 1024;

/// `FOG_TABLE_SIZE`.
/// Source: `oracle/codemp/renderer/tr_local.h:1246`
pub const FOG_TABLE_SIZE: usize = 256;

/// Raven `trGlobals_t` — the renderer's single big global state struct: current
/// scene/view/world, image/shader/model/skin registries, and lookup tables.
///
/// Raven: fields for the backend functions should never be modified by the
/// frontend, and vice versa: "backend functions should never modify any of
/// these fields, but may read fields that aren't dynamically modified by the
/// frontend". Large tables are placed at the end so most elements stay within
/// the +/-32K indexed range on RISC processors.
/// Type definition source: `oracle/codemp/renderer/tr_local.h:1309-1423`
#[repr(C)]
pub struct trGlobals_t {
    /// cleared at shutdown, set at beginRegistration
    pub registered: qboolean,

    /// incremented every time a new vis cluster is entered
    pub visCount: c_int,
    /// incremented every frame
    pub frameCount: c_int,
    /// incremented every scene
    pub sceneCount: c_int,
    /// incremented every view (twice a scene if portaled)
    /// and every R_MarkFragments call
    pub viewCount: c_int,

    /// zeroed at RE_BeginFrame
    pub frameSceneNum: c_int,

    pub worldMapLoaded: qboolean,
    pub world: *mut world_t,

    /// from RE_SetWorldVisData, shared with CM_Load
    pub externalVisData: *const u8,

    pub defaultImage: *mut image_t,
    pub scratchImage: [*mut image_t; NUM_SCRATCH_IMAGES],
    pub fogImage: *mut image_t,
    /// inverse-quare highlight for projective adding
    pub dlightImage: *mut image_t,
    pub flareImage: *mut image_t,
    /// full of 0xff
    pub whiteImage: *mut image_t,
    /// full of tr.identityLightByte
    pub identityLightImage: *mut image_t,

    /// reserve us a gl texnum to use with RF_DISTORTION
    pub screenImage: *mut image_t,

    // GLOWXXX
    /// Handle to the Glow Effect Vertex Shader. - AReis
    pub glowVShader: c_uint,

    /// Handle to the Glow Effect Pixel Shader. - AReis
    pub glowPShader: c_uint,

    /// Image the glowing objects are rendered to. - AReis
    pub screenGlow: c_uint,

    /// A rectangular texture representing the normally rendered scene.
    pub sceneImage: c_uint,

    /// Image used to downsample and blur scene to. - AReis
    pub blurImage: c_uint,

    pub defaultShader: *mut shader_t,
    pub shadowShader: *mut shader_t,
    pub distortionShader: *mut shader_t,
    pub projectionShadowShader: *mut shader_t,

    pub sunShader: *mut shader_t,

    pub numLightmaps: c_int,
    pub lightmaps: [*mut image_t; MAX_LIGHTMAPS],

    pub currentEntity: *mut trRefEntity_t,
    /// point currentEntity at this when rendering world
    pub worldEntity: trRefEntity_t,
    pub currentEntityNum: c_int,
    /// currentEntityNum << QSORT_ENTITYNUM_SHIFT
    pub shiftedEntityNum: c_int,
    pub currentModel: *mut model_t,

    pub viewParms: viewParms_t,

    /// 1.0 / ( 1 << overbrightBits )
    pub identityLight: c_float,
    /// identityLight * 255
    pub identityLightByte: c_int,
    /// r_overbrightBits->integer, but set to 0 if no hw gamma
    pub overbrightBits: c_int,

    /// for current entity
    pub ori: orientationr_t,

    pub refdef: trRefdef_t,

    pub viewCluster: c_int,

    /// from the sky shader for this level
    pub sunLight: vec3_t,
    pub sunDirection: vec3_t,
    /// from the sky shader for this level
    pub sunSurfaceLight: c_int,
    /// from the sky shader (only used for John's terrain system)
    pub sunAmbient: vec3_t,

    pub pc: frontEndCounters_t,
    /// not in pc due to clearing issue
    pub frontEndMsec: c_int,

    //
    // put large tables at the end, so most elements will be
    // within the +/32K indexed range on risc processors
    //
    pub models: [*mut model_t; MAX_MOD_KNOWN],
    pub numModels: c_int,

    pub bspModels: [world_t; MAX_SUB_BSP],
    pub numBSPModels: c_int,

    // shader indexes from other modules will be looked up in tr.shaders[]
    // shader indexes from drawsurfs will be looked up in sortedShaders[]
    // lower indexed sortedShaders must be rendered first (opaque surfaces before translucent)
    pub numShaders: c_int,
    pub shaders: [*mut shader_t; MAX_SHADERS],
    pub sortedShaders: [*mut shader_t; MAX_SHADERS],

    pub numSkins: c_int,
    pub skins: [*mut skin_t; MAX_SKINS],

    pub sinTable: [c_float; FUNCTABLE_SIZE],
    pub squareTable: [c_float; FUNCTABLE_SIZE],
    pub triangleTable: [c_float; FUNCTABLE_SIZE],
    pub sawToothTable: [c_float; FUNCTABLE_SIZE],
    pub inverseSawToothTable: [c_float; FUNCTABLE_SIZE],
    pub fogTable: [c_float; FOG_TABLE_SIZE],

    pub rangedFog: c_float,
    /// rwwRMG - added
    pub distanceCull: c_float,
    /// rwwRMG - added
    pub distanceCullSquared: c_float,

    /// rwwRMG - added
    pub landScape: srfTerrain_t,
}

const _: () = assert!(core::mem::offset_of!(trGlobals_t, registered) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<trGlobals_t>() == 316472);
    assert!(core::mem::offset_of!(trGlobals_t, visCount) == 4);
    assert!(core::mem::offset_of!(trGlobals_t, frameCount) == 8);
    assert!(core::mem::offset_of!(trGlobals_t, sceneCount) == 12);
    assert!(core::mem::offset_of!(trGlobals_t, viewCount) == 16);
    assert!(core::mem::offset_of!(trGlobals_t, frameSceneNum) == 20);
    assert!(core::mem::offset_of!(trGlobals_t, worldMapLoaded) == 24);
    assert!(core::mem::offset_of!(trGlobals_t, world) == 32);
    assert!(core::mem::offset_of!(trGlobals_t, externalVisData) == 40);
    assert!(core::mem::offset_of!(trGlobals_t, defaultImage) == 48);
    assert!(core::mem::offset_of!(trGlobals_t, scratchImage) == 56);
    assert!(core::mem::offset_of!(trGlobals_t, fogImage) == 184);
    assert!(core::mem::offset_of!(trGlobals_t, dlightImage) == 192);
    assert!(core::mem::offset_of!(trGlobals_t, flareImage) == 200);
    assert!(core::mem::offset_of!(trGlobals_t, whiteImage) == 208);
    assert!(core::mem::offset_of!(trGlobals_t, identityLightImage) == 216);
    assert!(core::mem::offset_of!(trGlobals_t, screenImage) == 224);
    assert!(core::mem::offset_of!(trGlobals_t, glowVShader) == 232);
    assert!(core::mem::offset_of!(trGlobals_t, glowPShader) == 236);
    assert!(core::mem::offset_of!(trGlobals_t, screenGlow) == 240);
    assert!(core::mem::offset_of!(trGlobals_t, sceneImage) == 244);
    assert!(core::mem::offset_of!(trGlobals_t, blurImage) == 248);
    assert!(core::mem::offset_of!(trGlobals_t, defaultShader) == 256);
    assert!(core::mem::offset_of!(trGlobals_t, shadowShader) == 264);
    assert!(core::mem::offset_of!(trGlobals_t, distortionShader) == 272);
    assert!(core::mem::offset_of!(trGlobals_t, projectionShadowShader) == 280);
    assert!(core::mem::offset_of!(trGlobals_t, sunShader) == 288);
    assert!(core::mem::offset_of!(trGlobals_t, numLightmaps) == 296);
    assert!(core::mem::offset_of!(trGlobals_t, lightmaps) == 304);
    assert!(core::mem::offset_of!(trGlobals_t, currentEntity) == 2352);
    assert!(core::mem::offset_of!(trGlobals_t, worldEntity) == 2360);
    assert!(core::mem::offset_of!(trGlobals_t, currentEntityNum) == 2632);
    assert!(core::mem::offset_of!(trGlobals_t, shiftedEntityNum) == 2636);
    assert!(core::mem::offset_of!(trGlobals_t, currentModel) == 2640);
    assert!(core::mem::offset_of!(trGlobals_t, viewParms) == 2648);
    assert!(core::mem::offset_of!(trGlobals_t, identityLight) == 3140);
    assert!(core::mem::offset_of!(trGlobals_t, identityLightByte) == 3144);
    assert!(core::mem::offset_of!(trGlobals_t, overbrightBits) == 3148);
    assert!(core::mem::offset_of!(trGlobals_t, ori) == 3152);
    assert!(core::mem::offset_of!(trGlobals_t, refdef) == 3280);
    assert!(core::mem::offset_of!(trGlobals_t, viewCluster) == 3728);
    assert!(core::mem::offset_of!(trGlobals_t, sunLight) == 3732);
    assert!(core::mem::offset_of!(trGlobals_t, sunDirection) == 3744);
    assert!(core::mem::offset_of!(trGlobals_t, sunSurfaceLight) == 3756);
    assert!(core::mem::offset_of!(trGlobals_t, sunAmbient) == 3760);
    assert!(core::mem::offset_of!(trGlobals_t, pc) == 3772);
    assert!(core::mem::offset_of!(trGlobals_t, frontEndMsec) == 3832);
    assert!(core::mem::offset_of!(trGlobals_t, models) == 3840);
    assert!(core::mem::offset_of!(trGlobals_t, numModels) == 12032);
    assert!(core::mem::offset_of!(trGlobals_t, bspModels) == 12040);
    assert!(core::mem::offset_of!(trGlobals_t, numBSPModels) == 24584);
    assert!(core::mem::offset_of!(trGlobals_t, numShaders) == 24588);
    assert!(core::mem::offset_of!(trGlobals_t, shaders) == 24592);
    assert!(core::mem::offset_of!(trGlobals_t, sortedShaders) == 155664);
    assert!(core::mem::offset_of!(trGlobals_t, numSkins) == 286736);
    assert!(core::mem::offset_of!(trGlobals_t, skins) == 286744);
    assert!(core::mem::offset_of!(trGlobals_t, sinTable) == 294936);
    assert!(core::mem::offset_of!(trGlobals_t, squareTable) == 299032);
    assert!(core::mem::offset_of!(trGlobals_t, triangleTable) == 303128);
    assert!(core::mem::offset_of!(trGlobals_t, sawToothTable) == 307224);
    assert!(core::mem::offset_of!(trGlobals_t, inverseSawToothTable) == 311320);
    assert!(core::mem::offset_of!(trGlobals_t, fogTable) == 315416);
    assert!(core::mem::offset_of!(trGlobals_t, rangedFog) == 316440);
    assert!(core::mem::offset_of!(trGlobals_t, distanceCull) == 316444);
    assert!(core::mem::offset_of!(trGlobals_t, distanceCullSquared) == 316448);
    assert!(core::mem::offset_of!(trGlobals_t, landScape) == 316456);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<trGlobals_t>() == 173676);
    assert!(core::mem::offset_of!(trGlobals_t, visCount) == 4);
    assert!(core::mem::offset_of!(trGlobals_t, frameCount) == 8);
    assert!(core::mem::offset_of!(trGlobals_t, sceneCount) == 12);
    assert!(core::mem::offset_of!(trGlobals_t, viewCount) == 16);
    assert!(core::mem::offset_of!(trGlobals_t, frameSceneNum) == 20);
    assert!(core::mem::offset_of!(trGlobals_t, worldMapLoaded) == 24);
    assert!(core::mem::offset_of!(trGlobals_t, world) == 28);
    assert!(core::mem::offset_of!(trGlobals_t, externalVisData) == 32);
    assert!(core::mem::offset_of!(trGlobals_t, defaultImage) == 36);
    assert!(core::mem::offset_of!(trGlobals_t, scratchImage) == 40);
    assert!(core::mem::offset_of!(trGlobals_t, fogImage) == 104);
    assert!(core::mem::offset_of!(trGlobals_t, dlightImage) == 108);
    assert!(core::mem::offset_of!(trGlobals_t, flareImage) == 112);
    assert!(core::mem::offset_of!(trGlobals_t, whiteImage) == 116);
    assert!(core::mem::offset_of!(trGlobals_t, identityLightImage) == 120);
    assert!(core::mem::offset_of!(trGlobals_t, screenImage) == 124);
    assert!(core::mem::offset_of!(trGlobals_t, glowVShader) == 128);
    assert!(core::mem::offset_of!(trGlobals_t, glowPShader) == 132);
    assert!(core::mem::offset_of!(trGlobals_t, screenGlow) == 136);
    assert!(core::mem::offset_of!(trGlobals_t, sceneImage) == 140);
    assert!(core::mem::offset_of!(trGlobals_t, blurImage) == 144);
    assert!(core::mem::offset_of!(trGlobals_t, defaultShader) == 148);
    assert!(core::mem::offset_of!(trGlobals_t, shadowShader) == 152);
    assert!(core::mem::offset_of!(trGlobals_t, distortionShader) == 156);
    assert!(core::mem::offset_of!(trGlobals_t, projectionShadowShader) == 160);
    assert!(core::mem::offset_of!(trGlobals_t, sunShader) == 164);
    assert!(core::mem::offset_of!(trGlobals_t, numLightmaps) == 168);
    assert!(core::mem::offset_of!(trGlobals_t, lightmaps) == 172);
    assert!(core::mem::offset_of!(trGlobals_t, currentEntity) == 1196);
    assert!(core::mem::offset_of!(trGlobals_t, worldEntity) == 1200);
    assert!(core::mem::offset_of!(trGlobals_t, currentEntityNum) == 1468);
    assert!(core::mem::offset_of!(trGlobals_t, shiftedEntityNum) == 1472);
    assert!(core::mem::offset_of!(trGlobals_t, currentModel) == 1476);
    assert!(core::mem::offset_of!(trGlobals_t, viewParms) == 1480);
    assert!(core::mem::offset_of!(trGlobals_t, identityLight) == 1972);
    assert!(core::mem::offset_of!(trGlobals_t, identityLightByte) == 1976);
    assert!(core::mem::offset_of!(trGlobals_t, overbrightBits) == 1980);
    assert!(core::mem::offset_of!(trGlobals_t, ori) == 1984);
    assert!(core::mem::offset_of!(trGlobals_t, refdef) == 2108);
    assert!(core::mem::offset_of!(trGlobals_t, viewCluster) == 2524);
    assert!(core::mem::offset_of!(trGlobals_t, sunLight) == 2528);
    assert!(core::mem::offset_of!(trGlobals_t, sunDirection) == 2540);
    assert!(core::mem::offset_of!(trGlobals_t, sunSurfaceLight) == 2552);
    assert!(core::mem::offset_of!(trGlobals_t, sunAmbient) == 2556);
    assert!(core::mem::offset_of!(trGlobals_t, pc) == 2568);
    assert!(core::mem::offset_of!(trGlobals_t, frontEndMsec) == 2628);
    assert!(core::mem::offset_of!(trGlobals_t, models) == 2632);
    assert!(core::mem::offset_of!(trGlobals_t, numModels) == 6728);
    assert!(core::mem::offset_of!(trGlobals_t, bspModels) == 6732);
    assert!(core::mem::offset_of!(trGlobals_t, numBSPModels) == 16972);
    assert!(core::mem::offset_of!(trGlobals_t, numShaders) == 16976);
    assert!(core::mem::offset_of!(trGlobals_t, shaders) == 16980);
    assert!(core::mem::offset_of!(trGlobals_t, sortedShaders) == 82516);
    assert!(core::mem::offset_of!(trGlobals_t, numSkins) == 148052);
    assert!(core::mem::offset_of!(trGlobals_t, skins) == 148056);
    assert!(core::mem::offset_of!(trGlobals_t, sinTable) == 152152);
    assert!(core::mem::offset_of!(trGlobals_t, squareTable) == 156248);
    assert!(core::mem::offset_of!(trGlobals_t, triangleTable) == 160344);
    assert!(core::mem::offset_of!(trGlobals_t, sawToothTable) == 164440);
    assert!(core::mem::offset_of!(trGlobals_t, inverseSawToothTable) == 168536);
    assert!(core::mem::offset_of!(trGlobals_t, fogTable) == 172632);
    assert!(core::mem::offset_of!(trGlobals_t, rangedFog) == 173656);
    assert!(core::mem::offset_of!(trGlobals_t, distanceCull) == 173660);
    assert!(core::mem::offset_of!(trGlobals_t, distanceCullSquared) == 173664);
    assert!(core::mem::offset_of!(trGlobals_t, landScape) == 173668);
};
