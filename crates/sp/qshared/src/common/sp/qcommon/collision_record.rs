//! SP Ghoul2 collision record type copied from Raven `code/game/ghoul2_shared.h`.
//!
//! Source: `oracle/oracle/code/game/ghoul2_shared.h:456-486`

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::c_int;

/// Ghoul2 model collision hit record.
///
/// Raven uses this as an entry in `trace_t.G2CollisionMap`, described as the
/// map of Ghoul2 model parts hit by a trace. Usage in Ghoul2 collision code
/// treats `mEntityNum == -1` as an unused record; populated records carry hit
/// distance, entity/model/surface indexes, collision position/normal, flags,
/// material, location, and barycentric hit coordinates.
pub use crate::shared::CollisionRecord_t as CCollisionRecord;

// collision detection stuff
pub const G2_FRONTFACE: c_int = 1;
pub const G2_BACKFACE: c_int = 0;

/// SP Raven constructor defaults for `CCollisionRecord`.
///
/// Constructor source: `oracle/oracle/code/game/ghoul2_shared.h:477-481`
pub const fn new_ccollision_record() -> CCollisionRecord {
    CCollisionRecord {
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

pub const MAX_G2_COLLISIONS: usize = 16;
