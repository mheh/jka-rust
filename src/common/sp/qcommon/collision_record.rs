//! SP Ghoul2 collision record type copied from Raven `code/game/ghoul2_shared.h`.
//!
//! Source: `oracle/oracle/code/game/ghoul2_shared.h:456-486`

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_float, c_int};

use crate::shared::vec3_t;

// collision detection stuff
pub const G2_FRONTFACE: c_int = 1;
pub const G2_BACKFACE: c_int = 0;

/// Ghoul2 model collision hit record.
///
/// Raven uses this as an entry in `trace_t.G2CollisionMap`, described as the
/// map of Ghoul2 model parts hit by a trace. Usage in Ghoul2 collision code
/// treats `mEntityNum == -1` as an unused record; populated records carry hit
/// distance, entity/model/surface indexes, collision position/normal, flags,
/// material, location, and barycentric hit coordinates.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CCollisionRecord {
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

impl CCollisionRecord {
    /// SP Raven constructor defaults.
    ///
    /// Constructor source: `oracle/oracle/code/game/ghoul2_shared.h:477-481`
    pub const fn new() -> Self {
        Self {
            mDistance: 100000.0,
            mEntityNum: -1,
            mModelIndex: 0,
            mPolyIndex: 0,
            mSurfaceIndex: 0,
            mCollisionPosition: [0.0; 3],
            mCollisionNormal: [0.0; 3],
            mFlags: 0,
            mMaterial: 0,
            mLocation: 0,
            mBarycentricI: 0.0,
            mBarycentricJ: 0.0,
        }
    }
}

impl Default for CCollisionRecord {
    fn default() -> Self {
        Self::new()
    }
}

pub const MAX_G2_COLLISIONS: usize = 16;
