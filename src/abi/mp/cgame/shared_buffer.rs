//! MP cgame shared-buffer payloads.
//!
//! Raven registers `cg.sharedBuffer` with the engine through
//! `CG_SET_SHARED_BUFFER`, then several cgame exports read or write typed
//! structs through that shared memory instead of normal `vmMain` argv slots.
//!
//! Transport source: `oracle/oracle/codemp/cgame/cg_main.c:3713`
//! Transport source: `oracle/oracle/codemp/client/cl_cgame.cpp:1179-1184`

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_char, c_float, c_int};

use crate::codemp::game::q_shared_h::qhandle_t;
use crate::common::mp::trace_t::trace_t;
use crate::shared::qboolean;
use crate::shared::vec3_t;

/// Typed view of Raven's MP cgame `cg.sharedBuffer` payload.
///
/// This is not a `vmMain` integer argument. The executable and cgame exchange a
/// single registered shared-memory region, and these vmcalls interpret that
/// region as the payload type documented on each export.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SharedBufferPayload<T> {
    ptr: *mut T,
}

impl<T> SharedBufferPayload<T> {
    /// Construct a typed shared-buffer payload pointer.
    ///
    /// # Safety
    /// `ptr` must point at the registered MP cgame shared buffer and contain a
    /// valid `T` payload for the selected vmcall.
    pub const unsafe fn new(ptr: *mut T) -> Self {
        Self { ptr }
    }

    pub const fn as_ptr(self) -> *const T {
        self.ptr
    }

    pub const fn as_mut_ptr(self) -> *mut T {
        self.ptr
    }
}

/// `CG_AUTOMAP_INPUT` shared-buffer payload.
///
/// Type definition source: `oracle/oracle/codemp/cgame/cg_public.h:442-449`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:317-319`
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct autoMapInput_t {
    pub up: c_float,
    pub down: c_float,
    pub yaw: c_float,
    pub pitch: c_float,
    pub goToDefaults: qboolean,
}

/// `CG_POINT_CONTENTS` shared-buffer payload.
///
/// Type definition source: `oracle/oracle/codemp/cgame/cg_public.h:451-456`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:362-366`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:366`
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TCGPointContents {
    pub mPoint: vec3_t,
    pub mPassEntityNum: c_int,
}

/// `CG_GET_LERP_DATA` shared-buffer payload.
///
/// Type definition source: `oracle/oracle/codemp/cgame/cg_public.h:458-466`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:377-378`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:380-406`
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TCGGetBoltData {
    pub mOrigin: vec3_t,
    pub mAngles: vec3_t,
    pub mScale: vec3_t,
    pub mEntityNum: c_int,
}

/// `CG_IMPACT_MARK` shared-buffer payload.
///
/// Raven's switch passes `qtrue` for `alphaFade` and `qfalse` for `temporary`;
/// those flags are not present in the shared-buffer struct.
///
/// Type definition source: `oracle/oracle/codemp/cgame/cg_public.h:467-479`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:570-579`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:578-579`
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TCGImpactMark {
    pub mHandle: qhandle_t,
    pub mPoint: vec3_t,
    pub mAngle: vec3_t,
    pub mRotation: c_float,
    pub mRed: c_float,
    pub mGreen: c_float,
    pub mBlue: c_float,
    pub mAlphaStart: c_float,
    pub mSizeStart: c_float,
}

/// `CG_GET_LERP_ORIGIN` shared-buffer payload.
///
/// Type definition source: `oracle/oracle/codemp/cgame/cg_public.h:484-489`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:371`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:373-374`
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TCGVectorData {
    pub mEntityNum: c_int,
    pub mPoint: vec3_t,
}

/// `CG_TRACE` and `CG_G2TRACE` shared-buffer payload.
///
/// Type definition source: `oracle/oracle/codemp/cgame/cg_public.h:490-496`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:408-417`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:412-417`
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TCGTrace {
    pub mResult: trace_t,
    pub mStart: vec3_t,
    pub mMins: vec3_t,
    pub mMaxs: vec3_t,
    pub mEnd: vec3_t,
    pub mSkipNumber: c_int,
    pub mMask: c_int,
}

/// `CG_G2MARK` shared-buffer payload.
///
/// Type definition source: `oracle/oracle/codemp/cgame/cg_public.h:498-504`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:419-424`
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TCGG2Mark {
    pub shader: c_int,
    pub size: c_float,
    pub start: vec3_t,
    pub dir: vec3_t,
}

/// `CG_FX_CAMERASHAKE` shared-buffer payload.
///
/// Type definition source: `oracle/oracle/codemp/cgame/cg_public.h:512-519`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:346-350`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:351`
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TCGCameraShake {
    pub mOrigin: vec3_t,
    pub mIntensity: c_float,
    pub mRadius: c_int,
    pub mTime: c_int,
}

/// `CG_MISC_ENT` shared-buffer payload.
///
/// Type definition source: `oracle/oracle/codemp/cgame/cg_public.h:521-526`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:582-586`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:599-621`
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TCGMiscEnt {
    /// Raven `char mModel[MAX_QPATH]`.
    ///
    /// `MAX_QPATH` definition source: `oracle/oracle/codemp/game/q_shared.h:393`
    pub mModel: [c_char; 64],
    pub mOrigin: vec3_t,
    pub mAngles: vec3_t,
    pub mScale: vec3_t,
}
