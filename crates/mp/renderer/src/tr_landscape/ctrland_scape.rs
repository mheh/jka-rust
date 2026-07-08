#![allow(non_camel_case_types, non_snake_case)]
use core::ffi::{c_float, c_int, c_void};

use mp_qshared::shared::qhandle_t;

use crate::tr_local::shader_s::shader_t;

use super::cter_vert::CTerVert;
use super::ctrheight_details::CTRHeightDetails;
use super::ctrpatch::CTRPatch;
use super::spatch_info::TPatchInfo;

// Raven's `#if _DEBUG` `mCycleCount` field is not present in a release build; the
// asserted layout below matches the non-debug shape.

// Raven `#define HEIGHT_RESOLUTION 256`.
// Source: oracle/oracle/codemp/qcommon/cm_landscape.h:13
const HEIGHT_RESOLUTION: usize = 256;

/// Raven `CTRLandScape` — the renderer-side landscape instance: patch storage,
/// sort order, terrain shaders, and per-height detail shaders.
///
/// Type definition source: `oracle/oracle/codemp/renderer/tr_landscape.h:119-186`
#[repr(C)]
pub struct CTRLandScape {
    //TODO: Port CCMLandScape
    // Source: oracle/oracle/codemp/qcommon/cm_landscape.h:135
    pub common: *const c_void,
    /// Local patch info
    pub mTRPatches: *mut CTRPatch,
    pub mSortedPatches: *mut TPatchInfo,

    pub mPatchMinx: c_int,
    pub mPatchMaxx: c_int,
    pub mPatchMiny: c_int,
    pub mPatchMaxy: c_int,
    /// terxels * terxels = exit condition for splitting
    pub mMaxNode: c_int,
    pub mSortedCount: c_int,

    pub mPatchSize: c_float,

    /// shader the terrain got its contents from
    pub mShader: *mut shader_t,

    /// modulation value and texture coords per vertex
    pub mRenderMap: *mut CTerVert,
    /// Scale of texture mapped to terrain
    pub mTextureScale: c_float,

    pub mScalarSize: c_float,

    /// Water shader
    pub mWaterShader: *mut shader_t,
    /// Flat ground shader
    pub mFlatShader: qhandle_t,

    /// Array of info specific to height
    pub mHeightDetails: [CTRHeightDetails; HEIGHT_RESOLUTION],
}

const _: () = assert!(core::mem::size_of::<CTRLandScape>() == 1120);
const _: () = assert!(core::mem::offset_of!(CTRLandScape, common) == 0);
const _: () = assert!(core::mem::offset_of!(CTRLandScape, mTRPatches) == 8);
const _: () = assert!(core::mem::offset_of!(CTRLandScape, mSortedPatches) == 16);
const _: () = assert!(core::mem::offset_of!(CTRLandScape, mPatchMinx) == 24);
const _: () = assert!(core::mem::offset_of!(CTRLandScape, mPatchMaxx) == 28);
const _: () = assert!(core::mem::offset_of!(CTRLandScape, mPatchMiny) == 32);
const _: () = assert!(core::mem::offset_of!(CTRLandScape, mPatchMaxy) == 36);
const _: () = assert!(core::mem::offset_of!(CTRLandScape, mMaxNode) == 40);
const _: () = assert!(core::mem::offset_of!(CTRLandScape, mSortedCount) == 44);
const _: () = assert!(core::mem::offset_of!(CTRLandScape, mPatchSize) == 48);
const _: () = assert!(core::mem::offset_of!(CTRLandScape, mShader) == 56);
const _: () = assert!(core::mem::offset_of!(CTRLandScape, mRenderMap) == 64);
const _: () = assert!(core::mem::offset_of!(CTRLandScape, mTextureScale) == 72);
const _: () = assert!(core::mem::offset_of!(CTRLandScape, mScalarSize) == 76);
const _: () = assert!(core::mem::offset_of!(CTRLandScape, mWaterShader) == 80);
const _: () = assert!(core::mem::offset_of!(CTRLandScape, mFlatShader) == 88);
const _: () = assert!(core::mem::offset_of!(CTRLandScape, mHeightDetails) == 92);
