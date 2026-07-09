//! MP `tr_types.h` full render entity — extends `miniRefEntity_t`'s layout.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_int, c_void};

use native_types::{byte, qhandle_t};

use crate::shared::{qboolean, vec2_t, vec3_t};

use super::ref_entity_type_t::refEntityType_t;

/// Anonymous struct for `refEntity_t::uRefEnt::uMini` (no Raven name — anonymous in the header).
///
/// Type definition source: `oracle/codemp/cgame/tr_types.h:191-195`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct refEntity_t_uMini {
    pub miniStart: c_int,
    pub miniCount: c_int,
}

/// Anonymous union for `refEntity_t::uRefEnt` (no Raven name — anonymous in the header).
///
/// Raven: only the `uMini` member is live; `skinNum`/`terxelCoords` alternatives are
/// commented out in the source.
/// Type definition source: `oracle/codemp/cgame/tr_types.h:187-196`
#[repr(C)]
#[derive(Clone, Copy)]
pub union refEntity_t_uRefEnt {
    pub uMini: refEntity_t_uMini,
}

/// Anonymous struct for `refEntity_t::data::sprite` (no Raven name — anonymous in the header).
///
/// Type definition source: `oracle/codemp/cgame/tr_types.h:200-205`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct refEntity_t_sprite {
    pub rotation: f32,
    pub radius: f32,
    pub vertRGBA: [[byte; 4]; 4],
}

/// Anonymous struct for `refEntity_t::data::line` (no Raven name — anonymous in the header).
///
/// Type definition source: `oracle/codemp/cgame/tr_types.h:206-211`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct refEntity_t_line {
    pub width: f32,
    pub width2: f32,
    pub stscale: f32,
}

/// Anonymous struct for `refEntity_t::data::bezier` (no Raven name — anonymous in the header).
///
/// Raven: that whole put-the-opening-brace-on-the-same-line-as-the-beginning-of-the-definition
/// coding style is fecal.
/// Type definition source: `oracle/codemp/cgame/tr_types.h:212-217`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct refEntity_t_bezier {
    pub width: f32,
    pub control1: vec3_t,
    pub control2: vec3_t,
}

/// Anonymous struct for `refEntity_t::data::cylinder` (no Raven name — anonymous in the header).
///
/// Type definition source: `oracle/codemp/cgame/tr_types.h:218-226`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct refEntity_t_cylinder {
    pub width: f32,
    pub width2: f32,
    pub stscale: f32,
    pub height: f32,
    pub bias: f32,
    pub wrap: qboolean,
}

/// Anonymous struct for `refEntity_t::data::electricity` (no Raven name — anonymous in the header).
///
/// Type definition source: `oracle/codemp/cgame/tr_types.h:227-234`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct refEntity_t_electricity {
    pub width: f32,
    pub deviation: f32,
    pub stscale: f32,
    pub wrap: qboolean,
    pub taper: qboolean,
}

/// Anonymous union for `refEntity_t::data` (no Raven name — anonymous in the header).
///
/// Type definition source: `oracle/codemp/cgame/tr_types.h:199-235`
#[repr(C)]
#[derive(Clone, Copy)]
pub union refEntity_t_data {
    pub sprite: refEntity_t_sprite,
    pub line: refEntity_t_line,
    pub bezier: refEntity_t_bezier,
    pub cylinder: refEntity_t_cylinder,
    pub electricity: refEntity_t_electricity,
}

/// Raven `refEntity_t` — full render entity handed from cgame/ui to the renderer.
///
/// Raven: this stucture must remain identical as the miniRefEntity_t (comment mirrored
/// from `miniRefEntity_t`, whose head this struct's layout matches).
/// Type definition source: `oracle/codemp/cgame/tr_types.h:133-251`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct refEntity_t {
    pub reType: refEntityType_t,
    pub renderfx: c_int,

    /// opaque type outside refresh
    pub hModel: qhandle_t,

    // most recent data
    /// rotation vectors
    pub axis: [vec3_t; 3],
    /// axis are not normalized, i.e. they have scale
    pub nonNormalizedAxes: qboolean,
    /// also used as MODEL_BEAM's "from"
    pub origin: vec3_t,

    // previous data for frame interpolation
    /// also used as MODEL_BEAM's "to"
    pub oldorigin: vec3_t,

    // texturing
    /// use one image for the entire thing
    pub customShader: qhandle_t,

    // misc
    /// colors used by rgbgen entity shaders
    pub shaderRGBA: [byte; 4],
    /// texture coordinates used by tcMod entity modifiers
    pub shaderTexCoord: vec2_t,

    // extra sprite information
    pub radius: f32,
    pub rotation: f32,

    // misc
    /// subtracted from refdef time to control effect start times
    pub shaderTime: f32,
    /// also used as MODEL_BEAM's diameter
    pub frame: c_int,

    // most recent data
    /// so multi-part models can be lit identically (RF_LIGHTING_ORIGIN)
    pub lightingOrigin: vec3_t,
    /// projection shadows go here, stencils go slightly lower
    pub shadowPlane: f32,

    // previous data for frame interpolation
    pub oldframe: c_int,
    /// 0.0 = current, 1.0 = old
    pub backlerp: f32,

    // texturing
    /// inline skin index
    pub skinNum: c_int,
    /// NULL for default skin
    pub customSkin: qhandle_t,

    // texturing
    pub uRefEnt: refEntity_t_uRefEnt,

    // extra sprite information
    pub data: refEntity_t_data,

    pub endTime: f32,
    pub saberLength: f32,

    // Ghoul2 Insert Start
    /// rotation angles - used for Ghoul2
    pub angles: vec3_t,

    /// axis scale for models
    pub modelScale: vec3_t,
    /// has to be at the end of the ref-ent in order for it to be created properly
    ///
    /// Raven: `CGhoul2Info_v *ghoul2;` is commented out in favor of the opaque `void*` below.
    pub ghoul2: *mut c_void,
    // Ghoul2 Insert End
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<refEntity_t>() == 216);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, reType) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, renderfx) == 4);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, hModel) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, axis) == 12);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, nonNormalizedAxes) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, origin) == 52);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, oldorigin) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, customShader) == 76);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, shaderRGBA) == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, shaderTexCoord) == 84);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, radius) == 92);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, rotation) == 96);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, shaderTime) == 100);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, frame) == 104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, lightingOrigin) == 108);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, shadowPlane) == 120);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, oldframe) == 124);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, backlerp) == 128);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, skinNum) == 132);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, customSkin) == 136);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, uRefEnt) == 140);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, data) == 148);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, endTime) == 176);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, saberLength) == 180);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, angles) == 184);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, modelScale) == 196);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refEntity_t, ghoul2) == 208);
