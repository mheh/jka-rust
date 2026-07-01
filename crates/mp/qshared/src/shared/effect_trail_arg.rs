#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::shared::effect_trail_vert::effectTrailVertStruct_t;
use native_types::qhandle_t;

/// Raven `effectTrailArgStruct_t` (`effectTrailArgStruct_s`) — `CG_ADDTRAIL` VM args.
///
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:2615-2620`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct effectTrailArgStruct_t {
    pub mVerts: [effectTrailVertStruct_t; 4],
    pub mShader: qhandle_t,
    pub mSetFlags: core::ffi::c_int,
    pub mKillTime: core::ffi::c_int,
}

const _: () = {
    use core::mem::{offset_of, size_of};
    assert!(size_of::<effectTrailArgStruct_t>() == 348);
    assert!(offset_of!(effectTrailArgStruct_t, mShader) == 336);
    assert!(offset_of!(effectTrailArgStruct_t, mKillTime) == 344);
};
