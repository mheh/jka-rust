#![allow(non_camel_case_types, non_snake_case)]

use super::acff_t::acff_t;
use super::alpha_gen_t::alphaGen_t;
use super::color_gen_t::colorGen_t;
use super::eglfog_override::EGLFogOverride;
use super::surface_sprite_s::surfaceSprite_t;
use super::texture_bundle_t::textureBundle_t;
use super::wave_form_t::waveForm_t;

/// Number of texture bundles per shader stage.
///
/// Source: `oracle/codemp/renderer/tr_local.h:107` (`NUM_TEXTURE_BUNDLES`)
pub const NUM_TEXTURE_BUNDLES: usize = 2;

/// Raven `shaderStage_t`.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:394-427`
#[repr(C)]
pub struct shaderStage_t {
    pub active: bool,
    pub isDetail: bool,

    /// index of stage
    pub index: u8,
    pub lightmapStyle: u8,

    pub bundle: [textureBundle_t; NUM_TEXTURE_BUNDLES],

    pub rgbWave: waveForm_t,
    pub rgbGen: colorGen_t,

    pub alphaWave: waveForm_t,
    pub alphaGen: alphaGen_t,

    /// for CGEN_CONST and AGEN_CONST
    pub constantColor: [u8; 4],

    /// GLS_xxxx mask
    pub stateBits: u32,

    pub adjustColorsForFog: acff_t,

    pub mGLFogColorOverride: EGLFogOverride,

    pub ss: *mut surfaceSprite_t,

    /// Whether this object emits a glow or not.
    pub glow: bool,
}

const _: () = assert!(core::mem::offset_of!(shaderStage_t, active) == 0);
const _: () = assert!(core::mem::offset_of!(shaderStage_t, isDetail) == 1);
const _: () = assert!(core::mem::offset_of!(shaderStage_t, index) == 2);
const _: () = assert!(core::mem::offset_of!(shaderStage_t, lightmapStyle) == 3);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<shaderStage_t>() == 184);
    assert!(core::mem::offset_of!(shaderStage_t, bundle) == 8);
    assert!(core::mem::offset_of!(shaderStage_t, rgbWave) == 104);
    assert!(core::mem::offset_of!(shaderStage_t, rgbGen) == 124);
    assert!(core::mem::offset_of!(shaderStage_t, alphaWave) == 128);
    assert!(core::mem::offset_of!(shaderStage_t, alphaGen) == 148);
    assert!(core::mem::offset_of!(shaderStage_t, constantColor) == 152);
    assert!(core::mem::offset_of!(shaderStage_t, stateBits) == 156);
    assert!(core::mem::offset_of!(shaderStage_t, adjustColorsForFog) == 160);
    assert!(core::mem::offset_of!(shaderStage_t, mGLFogColorOverride) == 164);
    assert!(core::mem::offset_of!(shaderStage_t, ss) == 168);
    assert!(core::mem::offset_of!(shaderStage_t, glow) == 176);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<shaderStage_t>() == 140);
    assert!(core::mem::offset_of!(shaderStage_t, bundle) == 4);
    assert!(core::mem::offset_of!(shaderStage_t, rgbWave) == 68);
    assert!(core::mem::offset_of!(shaderStage_t, rgbGen) == 88);
    assert!(core::mem::offset_of!(shaderStage_t, alphaWave) == 92);
    assert!(core::mem::offset_of!(shaderStage_t, alphaGen) == 112);
    assert!(core::mem::offset_of!(shaderStage_t, constantColor) == 116);
    assert!(core::mem::offset_of!(shaderStage_t, stateBits) == 120);
    assert!(core::mem::offset_of!(shaderStage_t, adjustColorsForFog) == 124);
    assert!(core::mem::offset_of!(shaderStage_t, mGLFogColorOverride) == 128);
    assert!(core::mem::offset_of!(shaderStage_t, ss) == 132);
    assert!(core::mem::offset_of!(shaderStage_t, glow) == 136);
};
