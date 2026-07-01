//! SP `tr_types.h` full render entity.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_int, c_void};

use native_types::byte;

use crate::shared::{qboolean, qhandle_t, vec2_t, vec3_t};

use super::ref_entity_type_t::refEntityType_t;

/// Anonymous union for `refEntity_t` (no Raven name — anonymous *and* unnamed in the
/// header, so `rotation`/`endTime`/`saberLength` are promoted directly onto `refEntity_t`
/// via GCC's anonymous-union extension; here it needs a field name to exist in Rust).
///
/// Raven: this doesn't have to be unioned, but it does make for more meaningful variable
/// names :)
/// Type definition source: `oracle/oracle/code/renderer/tr_types.h:135-140`
#[repr(C)]
#[derive(Clone, Copy)]
pub union refEntity_t_uMisc {
    pub rotation: f32,
    pub endTime: f32,
    pub saberLength: f32,
}

/// Raven `refEntity_t` — full render entity handed from cgame/ui to the renderer.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_types.h:100-153`
#[repr(C)]
pub struct refEntity_t {
    pub reType: refEntityType_t,
    pub renderfx: c_int,

    /// opaque type outside refresh
    pub hModel: qhandle_t,

    // most recent data
    /// so multi-part models can be lit identically (RF_LIGHTING_ORIGIN)
    pub lightingOrigin: vec3_t,
    /// projection shadows go here, stencils go slightly lower
    pub shadowPlane: f32,

    /// rotation vectors
    pub axis: [vec3_t; 3],
    /// axis are not normalized, i.e. they have scale
    pub nonNormalizedAxes: qboolean,
    /// also used as MODEL_BEAM's "from"
    pub origin: vec3_t,
    /// also used as MODEL_BEAM's diameter
    pub frame: c_int,

    // previous data for frame interpolation
    /// also used as MODEL_BEAM's "to"
    pub oldorigin: vec3_t,
    pub oldframe: c_int,
    /// 0.0 = current, 1.0 = old
    pub backlerp: f32,

    // texturing
    /// inline skin index
    pub skinNum: c_int,

    /// NULL for default skin
    pub customSkin: qhandle_t,
    /// use one image for the entire thing
    pub customShader: qhandle_t,

    // misc
    /// colors used by colorSrc=vertex shaders
    pub shaderRGBA: [byte; 4],
    /// texture coordinates used by tcMod=vertex modifiers
    pub shaderTexCoord: vec2_t,
    /// subtracted from refdef time to control effect start times
    pub shaderTime: f32,

    // extra sprite information
    pub radius: f32,

    // Raven: this doesn't have to be unioned, but it does make for more meaningful
    // variable names :)
    pub uMisc: refEntity_t_uMisc,

    // Ghoul2 Insert Start
    /// rotation angles - used for Ghoul2
    pub angles: vec3_t,

    /// axis scale for models
    pub modelScale: vec3_t,
    //TODO: Port CGhoul2Info_v
    // Source: oracle/oracle/code/ghoul2/G2.h
    /// has to be at the end of the ref-ent in order for it to be created properly
    pub ghoul2: *mut c_void,
    // Ghoul2 Insert End
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<refEntity_t>() == 176);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, reType) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, renderfx) == 4);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, hModel) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, lightingOrigin) == 12);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, shadowPlane) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, axis) == 28);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, nonNormalizedAxes) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, origin) == 68);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, frame) == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, oldorigin) == 84);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, oldframe) == 96);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, backlerp) == 100);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, skinNum) == 104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, customSkin) == 108);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, customShader) == 112);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, shaderRGBA) == 116);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, shaderTexCoord) == 120);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, shaderTime) == 128);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, radius) == 132);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, uMisc) == 136);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, angles) == 140);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, modelScale) == 152);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, ghoul2) == 168);
