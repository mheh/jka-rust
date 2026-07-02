#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_float, c_int, c_uint};

use sp_qshared::shared::{qboolean, vec3_t};

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

/// `MAX_QPATH`.
/// Source: `oracle/oracle/code/game/q_shared.h:215`
const MAX_QPATH: usize = 64;

/// `NUM_SCRATCH_IMAGES` (non-`_XBOX` branch).
/// Source: `oracle/oracle/code/renderer/tr_local.h:1121-1123`
const NUM_SCRATCH_IMAGES: usize = 16;

/// `MAX_LIGHTMAPS`.
/// Source: `oracle/oracle/code/renderer/tr_local.h:1003`
const MAX_LIGHTMAPS: usize = 256;

/// `MAX_MOD_KNOWN`.
/// Source: `oracle/oracle/code/renderer/tr_local.h:991`
const MAX_MOD_KNOWN: usize = 1024;

/// `MAX_SUB_BSP`.
/// Source: `oracle/oracle/code/game/q_shared.h:1464`
const MAX_SUB_BSP: usize = 32;

/// `MAX_SHADERS` (non-`_XBOX` branch; 14 bits, see `QSORT_SHADERNUM_SHIFT`).
/// Source: `oracle/oracle/code/renderer/tr_local.h:33-39`
const MAX_SHADERS: usize = 8192;

/// `MAX_SKINS`.
/// Source: `oracle/oracle/code/renderer/tr_local.h:1004`
const MAX_SKINS: usize = 512;

/// `FUNCTABLE_SIZE`.
/// Source: `oracle/oracle/code/renderer/tr_local.h:1059`
const FUNCTABLE_SIZE: usize = 1024;

/// `FOG_TABLE_SIZE`.
/// Source: `oracle/oracle/code/renderer/tr_local.h:1058`
const FOG_TABLE_SIZE: usize = 256;

/// Raven `trGlobals_t` — the renderer's single big global state struct: current
/// scene/view/world, image/shader/model/skin registries, and lookup tables.
///
/// Raven: Most renderer globals are defined here. Backend functions should
/// never modify any of these fields, but may read fields that aren't
/// dynamically modified by the frontend. Large tables are placed at the end
/// so most elements stay within the +/-32K indexed range on RISC processors.
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:1126-1248`
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
    /// ie: maps/tim_dm2
    pub worldDir: [u8; MAX_QPATH],

    /// from RE_SetWorldVisData, shared with CM_Load
    pub externalVisData: *const u8,

    pub defaultImage: *mut image_t,
    pub scratchImage: [*mut image_t; NUM_SCRATCH_IMAGES],
    pub fogImage: *mut image_t,
    /// inverse-quare highlight for projective adding
    pub dlightImage: *mut image_t,
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
    /// currentEntityNum << QSORT_ENTITYNUM_SHIFT (possible with high bit set for RF_ALPHA_FADE)
    pub shiftedEntityNum: c_uint,
    pub currentModel: *mut model_t,

    pub viewParms: viewParms_t,

    /// 1.0 / ( 1 << overbrightBits )
    pub identityLight: c_float,
    /// identityLight * 255
    pub identityLightByte: c_int,
    /// r_overbrightBits->integer, but set to 0 if no hw gamma
    pub overbrightBits: c_int,

    /// for current entity
    pub or: orientationr_t,

    pub refdef: trRefdef_t,

    pub viewCluster: c_int,

    /// from the sky shader for this level
    pub sunLight: vec3_t,
    pub sunDirection: vec3_t,
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
    /// used for error-messages only
    pub iNumDeniedShaders: c_int,

    pub numSkins: c_int,
    pub skins: [*mut skin_t; MAX_SKINS],

    pub sinTable: [c_float; FUNCTABLE_SIZE],

    pub squareTable: [c_float; FUNCTABLE_SIZE],
    pub triangleTable: [c_float; FUNCTABLE_SIZE],
    pub sawToothTable: [c_float; FUNCTABLE_SIZE],
    pub inverseSawToothTable: [c_float; FUNCTABLE_SIZE],
    pub fogTable: [c_float; FOG_TABLE_SIZE],

    pub rangedFog: c_float,

    pub distanceCull: c_float,
    pub landScape: srfTerrain_t,
}

const _: () = assert!(core::mem::size_of::<trGlobals_t>() == 175176);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, registered) == 0);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, visCount) == 4);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, frameCount) == 8);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, sceneCount) == 12);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, viewCount) == 16);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, frameSceneNum) == 20);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, worldMapLoaded) == 24);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, world) == 32);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, worldDir) == 40);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, externalVisData) == 104);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, defaultImage) == 112);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, scratchImage) == 120);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, fogImage) == 248);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, dlightImage) == 256);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, whiteImage) == 264);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, identityLightImage) == 272);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, screenImage) == 280);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, glowVShader) == 288);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, glowPShader) == 292);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, screenGlow) == 296);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, sceneImage) == 300);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, blurImage) == 304);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, defaultShader) == 312);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, shadowShader) == 320);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, distortionShader) == 328);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, projectionShadowShader) == 336);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, sunShader) == 344);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, numLightmaps) == 352);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, lightmaps) == 360);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, currentEntity) == 2408);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, worldEntity) == 2416);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, currentEntityNum) == 2648);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, shiftedEntityNum) == 2652);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, currentModel) == 2656);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, viewParms) == 2664);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, identityLight) == 3176);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, identityLightByte) == 3180);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, overbrightBits) == 3184);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, or) == 3188);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, refdef) == 3312);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, viewCluster) == 3504);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, sunLight) == 3508);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, sunDirection) == 3520);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, sunAmbient) == 3532);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, pc) == 3544);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, frontEndMsec) == 3604);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, models) == 3608);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, numModels) == 11800);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, bspModels) == 11808);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, numBSPModels) == 18464);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, numShaders) == 18468);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, shaders) == 18472);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, sortedShaders) == 84008);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, iNumDeniedShaders) == 149544);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, numSkins) == 149548);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, skins) == 149552);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, sinTable) == 153648);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, squareTable) == 157744);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, triangleTable) == 161840);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, sawToothTable) == 165936);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, inverseSawToothTable) == 170032);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, fogTable) == 174128);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, rangedFog) == 175152);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, distanceCull) == 175156);
const _: () = assert!(core::mem::offset_of!(trGlobals_t, landScape) == 175160);
