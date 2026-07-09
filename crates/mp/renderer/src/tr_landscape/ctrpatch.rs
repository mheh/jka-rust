#![allow(non_camel_case_types, non_snake_case)]
use core::ffi::c_void;

use mp_qshared::shared::vec3_t;

use crate::tr_local::shader_s::shader_t;

use super::cter_vert::CTerVert;
use super::ctrland_scape::CTRLandScape;

/// Raven `CTRPatch` — one renderer-side terrain patch: its bounds, blended shaders
/// for the two triangles, and visibility state for the current frame.
///
/// Type definition source: `oracle/codemp/renderer/tr_landscape.h:52-101`
#[repr(C)]
pub struct CTRPatch {
    //TODO: Port CCMLandScape
    // Source: oracle/codemp/qcommon/cm_landscape.h:135
    pub owner: *mut c_void,
    pub localowner: *mut CTRLandScape,
    //TODO: Port CCMPatch
    // Source: oracle/codemp/qcommon/cm_landscape.h:90
    pub common: *mut c_void,
    /// Real world center of the patch
    pub mCenter: vec3_t,
    /// Modulation value and texture coords per vertex
    pub mRenderMap: *mut CTerVert,
    /// Dynamically created blended shader for the top left triangle
    pub mTLShader: *mut shader_t,
    /// Dynamically created blended shader for the bottom right triangle
    pub mBRShader: *mut shader_t,
    /// Is this patch visible in the current frame?
    pub misVisible: bool,
}

const _: () = assert!(core::mem::size_of::<CTRPatch>() == 72);
const _: () = assert!(core::mem::offset_of!(CTRPatch, owner) == 0);
const _: () = assert!(core::mem::offset_of!(CTRPatch, localowner) == 8);
const _: () = assert!(core::mem::offset_of!(CTRPatch, common) == 16);
const _: () = assert!(core::mem::offset_of!(CTRPatch, mCenter) == 24);
const _: () = assert!(core::mem::offset_of!(CTRPatch, mRenderMap) == 40);
const _: () = assert!(core::mem::offset_of!(CTRPatch, mTLShader) == 48);
const _: () = assert!(core::mem::offset_of!(CTRPatch, mBRShader) == 56);
const _: () = assert!(core::mem::offset_of!(CTRPatch, misVisible) == 64);
