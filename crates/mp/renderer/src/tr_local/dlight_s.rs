#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec3_t;

use super::e_dlight_types::eDLightTypes;

/// Raven `dlight_s` (typedef `dlight_t`) — dynamic light.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:59-82`
#[repr(C)]
pub struct dlight_t {
    pub mType: eDLightTypes,

    pub origin: vec3_t,
    /// projected light's origin
    pub mProjOrigin: vec3_t,

    /// range from 0.0 to 1.0, should be color normalized
    pub color: vec3_t,

    pub radius: f32,
    /// desired radius of light
    pub mProjRadius: f32,

    /// texture detail is lost tho when the lightmap is dark
    pub additive: i32,

    /// origin in local coordinate system
    pub transformed: vec3_t,
    /// projected light's origin in local coordinate system
    pub mProjTransformed: vec3_t,

    pub mDirection: vec3_t,
    pub mBasis2: vec3_t,
    pub mBasis3: vec3_t,

    pub mTransDirection: vec3_t,
    pub mTransBasis2: vec3_t,
    pub mTransBasis3: vec3_t,
}

/// Raven manifest tag name; the typedef is `dlight_t`.
pub type dlight_s = dlight_t;

const _: () = assert!(core::mem::size_of::<dlight_t>() == 148);
const _: () = assert!(core::mem::offset_of!(dlight_t, mType) == 0);
const _: () = assert!(core::mem::offset_of!(dlight_t, origin) == 4);
const _: () = assert!(core::mem::offset_of!(dlight_t, mProjOrigin) == 16);
const _: () = assert!(core::mem::offset_of!(dlight_t, color) == 28);
const _: () = assert!(core::mem::offset_of!(dlight_t, radius) == 40);
const _: () = assert!(core::mem::offset_of!(dlight_t, mProjRadius) == 44);
const _: () = assert!(core::mem::offset_of!(dlight_t, additive) == 48);
const _: () = assert!(core::mem::offset_of!(dlight_t, transformed) == 52);
const _: () = assert!(core::mem::offset_of!(dlight_t, mProjTransformed) == 64);
const _: () = assert!(core::mem::offset_of!(dlight_t, mDirection) == 76);
const _: () = assert!(core::mem::offset_of!(dlight_t, mBasis2) == 88);
const _: () = assert!(core::mem::offset_of!(dlight_t, mBasis3) == 100);
const _: () = assert!(core::mem::offset_of!(dlight_t, mTransDirection) == 112);
const _: () = assert!(core::mem::offset_of!(dlight_t, mTransBasis2) == 124);
const _: () = assert!(core::mem::offset_of!(dlight_t, mTransBasis3) == 136);
