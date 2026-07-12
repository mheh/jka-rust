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

const _: () = assert!(core::mem::offset_of!(CTRPatch, owner) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<CTRPatch>() == 72);
    assert!(core::mem::offset_of!(CTRPatch, localowner) == 8);
    assert!(core::mem::offset_of!(CTRPatch, common) == 16);
    assert!(core::mem::offset_of!(CTRPatch, mCenter) == 24);
    assert!(core::mem::offset_of!(CTRPatch, mRenderMap) == 40);
    assert!(core::mem::offset_of!(CTRPatch, mTLShader) == 48);
    assert!(core::mem::offset_of!(CTRPatch, mBRShader) == 56);
    assert!(core::mem::offset_of!(CTRPatch, misVisible) == 64);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<CTRPatch>() == 40);
    assert!(core::mem::offset_of!(CTRPatch, localowner) == 4);
    assert!(core::mem::offset_of!(CTRPatch, common) == 8);
    assert!(core::mem::offset_of!(CTRPatch, mCenter) == 12);
    assert!(core::mem::offset_of!(CTRPatch, mRenderMap) == 24);
    assert!(core::mem::offset_of!(CTRPatch, mTLShader) == 28);
    assert!(core::mem::offset_of!(CTRPatch, mBRShader) == 32);
    assert!(core::mem::offset_of!(CTRPatch, misVisible) == 36);
};
