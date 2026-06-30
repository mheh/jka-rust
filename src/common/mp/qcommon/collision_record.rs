//! MP Ghoul2 collision record types copied from Raven `codemp/game/q_shared.h`.
//!
//! Source: `oracle/oracle/codemp/game/q_shared.h:1871-1888`

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_float, c_int};

use crate::shared::vec3_t;

/*
Ghoul2 Insert Start
*/
/// Ghoul2 model collision hit record.
///
/// Raven uses this as an entry in `G2Trace_t`, described as the map of Ghoul2
/// model parts hit by a trace. Usage in Ghoul2 collision code treats
/// `mEntityNum == -1` as an unused record; populated records carry hit
/// distance, entity/model/surface indexes, collision position/normal, flags,
/// material, location, and barycentric hit coordinates.
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

pub const MAX_G2_COLLISIONS: usize = 16;

pub type G2Trace_t = [CollisionRecord_t; MAX_G2_COLLISIONS]; // map that describes all of the parts of ghoul2 models that got hit
