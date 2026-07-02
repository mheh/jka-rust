//! SP `CCollisionRecord` — owning Raven header is `ghoul2_shared.h`.
//!
//! The struct layout lives in `crate::shared::collision` (shared with MP's
//! identically-shaped `CollisionRecord_t`); the SP-facing name plus Raven's
//! constructor defaults live in `crate::common::sp::qcommon::collision_record`.
//! Re-exported here so ghoul2-tier users find it at the owning header's home.
//!
//! Type definition source: `oracle/oracle/code/game/ghoul2_shared.h:461-481`

pub use crate::common::sp::qcommon::collision_record::{
    new_ccollision_record, CCollisionRecord, MAX_G2_COLLISIONS,
};
