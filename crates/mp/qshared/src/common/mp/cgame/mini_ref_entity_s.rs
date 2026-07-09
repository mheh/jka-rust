//! MP `tr_types.h` minimal ref-entity — reduced-cost variant of `refEntity_t`.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use native_types::{byte, qhandle_t};

use super::ref_entity_type_t::refEntityType_t;
use crate::shared::{qboolean, vec2_t, vec3_t};

/// Raven `miniRefEntity_t`.
///
/// Raven: this stucture must remain identical as the miniRefEntity_t (comment
/// mirrored from `refEntity_t`, whose head this struct's layout matches).
/// Type definition source: `oracle/codemp/cgame/tr_types.h:100-130`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct miniRefEntity_t {
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
    /// size 2 for RT_CYLINDER or number of verts in RT_ELECTRICITY
    pub rotation: f32,

    // misc
    /// subtracted from refdef time to control effect start times
    pub shaderTime: f32,
    /// also used as MODEL_BEAM's diameter
    pub frame: c_int,
}

const _: () = assert!(core::mem::size_of::<miniRefEntity_t>() == 108);
const _: () = assert!(core::mem::offset_of!(miniRefEntity_t, reType) == 0);
const _: () = assert!(core::mem::offset_of!(miniRefEntity_t, renderfx) == 4);
const _: () = assert!(core::mem::offset_of!(miniRefEntity_t, hModel) == 8);
const _: () = assert!(core::mem::offset_of!(miniRefEntity_t, axis) == 12);
const _: () = assert!(core::mem::offset_of!(miniRefEntity_t, nonNormalizedAxes) == 48);
const _: () = assert!(core::mem::offset_of!(miniRefEntity_t, origin) == 52);
const _: () = assert!(core::mem::offset_of!(miniRefEntity_t, oldorigin) == 64);
const _: () = assert!(core::mem::offset_of!(miniRefEntity_t, customShader) == 76);
const _: () = assert!(core::mem::offset_of!(miniRefEntity_t, shaderRGBA) == 80);
const _: () = assert!(core::mem::offset_of!(miniRefEntity_t, shaderTexCoord) == 84);
const _: () = assert!(core::mem::offset_of!(miniRefEntity_t, radius) == 92);
const _: () = assert!(core::mem::offset_of!(miniRefEntity_t, rotation) == 96);
const _: () = assert!(core::mem::offset_of!(miniRefEntity_t, shaderTime) == 100);
const _: () = assert!(core::mem::offset_of!(miniRefEntity_t, frame) == 104);
