#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

// `MAXLIGHTMAPS` (`qfiles.h`) imported from its canonical home on `drawVert_t`
// in `mp_engine_qcommon`.
use mp_engine_qcommon::qfiles::draw_vert_t::MAXLIGHTMAPS;
use mp_qshared::shared::{vec3_t, MAX_QPATH};

use super::cull_type_t::cullType_t;
use super::deform_stage_t::deformStage_t;
use super::fog_parms_t::fogParms_t;
use super::fog_pass_t::fogPass_t;
use super::shader_stage_t::shaderStage_t;
use super::sky_parms_t::skyParms_t;

/// Raven `MAX_SHADER_DEFORMS`.
///
/// Source: `oracle/codemp/renderer/tr_local.h:309`
const MAX_SHADER_DEFORMS: usize = 3;

/// Raven `shader_t` — compiled shader definition.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:459-530`
#[repr(C)]
pub struct shader_t {
    /// game path, including extension
    pub name: [c_char; MAX_QPATH as usize],
    /// for a shader to match, both name and lightmapIndex must match
    pub lightmapIndex: [i32; MAXLIGHTMAPS],
    pub styles: [u8; MAXLIGHTMAPS],

    /// this shader == tr.shaders[index]
    pub index: i32,
    /// this shader == tr.sortedShaders[sortedIndex]
    pub sortedIndex: i32,

    /// lower numbered shaders draw before higher numbered
    pub sort: f32,

    /// if explicitlyDefined, this will have SURF_* flags
    pub surfaceFlags: i32,
    pub contentFlags: i32,

    /// we want to return index 0 if the shader failed to
    /// load for some reason, but R_FindShader should
    /// still keep a name allocated for it, so if
    /// something calls RE_RegisterShader again with
    /// the same name, we don't try looking for it again
    pub defaultShader: bool,
    /// found in a .shader file
    pub explicitlyDefined: bool,
    /// merge across entites optimizable (smoke, blood)
    pub entityMergable: bool,

    pub isBumpMap: bool,

    pub sky: *mut skyParms_t,
    pub fogParms: *mut fogParms_t,

    /// distance to fog out at
    pub portalRange: f32,

    /// 0, GL_MODULATE, GL_ADD (FIXME: put in stage)
    pub multitextureEnv: i32,

    /// CT_FRONT_SIDED, CT_BACK_SIDED, or CT_TWO_SIDED
    pub cullType: cullType_t,
    /// set for decals and other items that must be offset
    pub polygonOffset: bool,
    /// for console fonts, 2D elements, etc.
    pub noMipMaps: bool,
    /// for images that must always be full resolution
    pub noPicMip: bool,
    /// for images that don't want to be texture compressed (eg skies)
    pub noTC: bool,

    /// draw a blended pass, possibly with depth test equals
    pub fogPass: fogPass_t,

    /// The given light vector for bump-mapping
    pub bumpVector: vec3_t,

    pub deforms: [*mut deformStage_t; MAX_SHADER_DEFORMS],
    pub numDeforms: i16,

    pub numUnfoggedPasses: i16,
    pub stages: *mut shaderStage_t,

    /// time this shader is clamped to
    pub clampTime: f32,
    /// current time offset for this shader
    pub timeOffset: f32,

    // GLOWXXX
    /// True if this shader has a stage with glow in it (just an optimization).
    pub hasGlow: bool,

    /// current shader this one is remapped too
    pub remappedShader: *mut shader_t,
    pub next: *mut shader_t,
}

/// Manifest alias: oracle tags this struct `shader_s`.
pub type shader_s = shader_t;

const _: () = assert!(core::mem::size_of::<shader_t>() == 232);
const _: () = assert!(core::mem::offset_of!(shader_t, name) == 0);
const _: () = assert!(core::mem::offset_of!(shader_t, lightmapIndex) == 64);
const _: () = assert!(core::mem::offset_of!(shader_t, styles) == 80);
const _: () = assert!(core::mem::offset_of!(shader_t, index) == 84);
const _: () = assert!(core::mem::offset_of!(shader_t, sortedIndex) == 88);
const _: () = assert!(core::mem::offset_of!(shader_t, sort) == 92);
const _: () = assert!(core::mem::offset_of!(shader_t, surfaceFlags) == 96);
const _: () = assert!(core::mem::offset_of!(shader_t, contentFlags) == 100);
const _: () = assert!(core::mem::offset_of!(shader_t, defaultShader) == 104);
const _: () = assert!(core::mem::offset_of!(shader_t, explicitlyDefined) == 105);
const _: () = assert!(core::mem::offset_of!(shader_t, entityMergable) == 106);
const _: () = assert!(core::mem::offset_of!(shader_t, isBumpMap) == 107);
const _: () = assert!(core::mem::offset_of!(shader_t, sky) == 112);
const _: () = assert!(core::mem::offset_of!(shader_t, fogParms) == 120);
const _: () = assert!(core::mem::offset_of!(shader_t, portalRange) == 128);
const _: () = assert!(core::mem::offset_of!(shader_t, multitextureEnv) == 132);
const _: () = assert!(core::mem::offset_of!(shader_t, cullType) == 136);
const _: () = assert!(core::mem::offset_of!(shader_t, polygonOffset) == 140);
const _: () = assert!(core::mem::offset_of!(shader_t, noMipMaps) == 141);
const _: () = assert!(core::mem::offset_of!(shader_t, noPicMip) == 142);
const _: () = assert!(core::mem::offset_of!(shader_t, noTC) == 143);
const _: () = assert!(core::mem::offset_of!(shader_t, fogPass) == 144);
const _: () = assert!(core::mem::offset_of!(shader_t, bumpVector) == 148);
const _: () = assert!(core::mem::offset_of!(shader_t, deforms) == 160);
const _: () = assert!(core::mem::offset_of!(shader_t, numDeforms) == 184);
const _: () = assert!(core::mem::offset_of!(shader_t, numUnfoggedPasses) == 186);
const _: () = assert!(core::mem::offset_of!(shader_t, stages) == 192);
const _: () = assert!(core::mem::offset_of!(shader_t, clampTime) == 200);
const _: () = assert!(core::mem::offset_of!(shader_t, timeOffset) == 204);
const _: () = assert!(core::mem::offset_of!(shader_t, hasGlow) == 208);
const _: () = assert!(core::mem::offset_of!(shader_t, remappedShader) == 216);
const _: () = assert!(core::mem::offset_of!(shader_t, next) == 224);
