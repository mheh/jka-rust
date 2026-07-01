//! Shared Raven collision types from `q_shared.h`.
//!
//! Source: `oracle/oracle/code/game/q_shared.h:1355-1363`
//! Source: `oracle/oracle/codemp/game/q_shared.h:1858-1866`

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_float, c_int};

use crate::shared::vec3_t;

// plane_t structure
// !!! if this is changed, it must be changed in asm code too !!!
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct cplane_t {
    pub normal: vec3_t,
    pub dist: c_float,
    pub r#type: u8,   // for fast side tests: 0,1,2 = axial, 3 = nonaxial
    pub signbits: u8, // signx + (signy<<1) + (signz<<2), used as lookup during collision
    pub pad: [u8; 2],
}

/// Ghoul2 model collision hit record.
///
/// MP type definition source: `oracle/oracle/codemp/game/q_shared.h:1871-1884`
/// SP equivalent source: `oracle/oracle/code/game/ghoul2_shared.h:461-486`
///
/// Raven uses this layout for records describing Ghoul2 model parts hit by a
/// trace. Mode-specific modules may expose Raven's local naming and
/// initialization behavior on top of this shared layout.
///
/// SP callers must use `crate::common::sp::qcommon::collision_record` for the
/// SP-facing `CCollisionRecord` name and constructor defaults.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionRecord_t {
    pub mDistance: c_float,
    pub mEntityNum: c_int,
    pub mModelIndex: c_int,
    pub mPolyIndex: c_int,
    pub mSurfaceIndex: c_int,
    pub mCollisionPosition: vec3_t,
    pub mCollisionNormal: vec3_t,
    pub mFlags: c_int,
    pub mMaterial: c_int,
    pub mLocation: c_int,
    pub mBarycentricI: c_float, // two barycentic coodinates for the hit point
    pub mBarycentricJ: c_float, // K = 1-I-J
}

const _: () = assert!(core::mem::size_of::<cplane_t>() == 20);
const _: () = assert!(core::mem::offset_of!(cplane_t, normal) == 0);
const _: () = assert!(core::mem::offset_of!(cplane_t, dist) == 12);
const _: () = assert!(core::mem::offset_of!(cplane_t, r#type) == 16);
const _: () = assert!(core::mem::offset_of!(cplane_t, signbits) == 17);
const _: () = assert!(core::mem::offset_of!(cplane_t, pad) == 18);

const _: () = assert!(core::mem::size_of::<CollisionRecord_t>() == 64);
const _: () = assert!(core::mem::offset_of!(CollisionRecord_t, mDistance) == 0);
const _: () = assert!(core::mem::offset_of!(CollisionRecord_t, mCollisionPosition) == 20);
const _: () = assert!(core::mem::offset_of!(CollisionRecord_t, mCollisionNormal) == 32);
const _: () = assert!(core::mem::offset_of!(CollisionRecord_t, mFlags) == 44);
const _: () = assert!(core::mem::offset_of!(CollisionRecord_t, mBarycentricI) == 56);
const _: () = assert!(core::mem::offset_of!(CollisionRecord_t, mBarycentricJ) == 60);
