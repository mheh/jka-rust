//! `EntityId` — the entity handle (porting-rules §B5), hoisted to `mp_qshared`
//! so `gentity_t`'s stored entity fields can name it.
//!
//! `gentity_t`'s 38 stored `gentity_t*` struct fields are `Option<EntityId>`, so
//! that field type must be visible in `mp_qshared`. `mp_game` re-exports it (via
//! `crate::world::EntityId` + the prelude) so every existing use keeps compiling.
//! The pointer↔index seam helpers (`ent_id`/`ent_id_opt`) do `offset_from` over
//! the concrete `gentity_t`, which now lives in `mp_game` (DEC-26), so they moved
//! there (`crate::ent_id`); only the number↔handle edges (`from_num`/`to_num`)
//! stay here.

use core::ffi::c_int;

use crate::shared::{ENTITYNUM_NONE, MAX_GENTITIES};

/// Raven's `gentity_t*` become an index into `GameWorld.entities`. Module logic
/// passes `(world, id)` and re-indexes per access — GP2's `GpGroupId` precedent;
/// no aliasing raw pointers in safe code (§B5).
///
/// Wire-shaped `u32` newtype (entity numbers cross the wire as indices). Entity 0
/// is valid, so nullability is carried by `Option<EntityId>`, not a
/// sentinel niche.
///
/// Source: `docs/architecture/state-ownership.md` § `EntityId` — the entity
/// handle (§B5).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EntityId(pub u32);

impl EntityId {
    /// The raw entity-array index.
    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    /// Seam edge: a raw Raven entity number → `Option<EntityId>`, the
    /// number-half companion of the pointer-half [`ent_id_opt`]. `None` for the
    /// `ENTITYNUM_NONE` "no entity" sentinel and for anything outside the valid
    /// arena range `[0, MAX_GENTITIES)` (negatives included) — the same set of
    /// values Raven treats as a NULL `gentity_t*`, so a NULL-pointer check
    /// becomes an `is_none()`.
    ///
    /// Source: `oracle/codemp/game/q_shared.h:2014-2016` (`ENTITYNUM_NONE`) and
    /// the `entityNum >= 0` guards throughout `oracle/codemp/game`.
    #[inline]
    pub fn from_num(n: c_int) -> Option<EntityId> {
        if n < 0 || n >= MAX_GENTITIES as c_int || n == ENTITYNUM_NONE {
            None
        } else {
            Some(EntityId(n as u32))
        }
    }
}

/// Seam edge: `Option<EntityId>` → a raw Raven entity number, the inverse of
/// [`EntityId::from_num`]. `None` (no entity) becomes `ENTITYNUM_NONE`, matching
/// what Raven writes into `entityNum`/`otherEntityNum` fields for "no entity".
///
/// Source: `oracle/codemp/game/q_shared.h:2014-2016` (`ENTITYNUM_NONE`).
#[inline]
pub fn to_num(id: Option<EntityId>) -> c_int {
    match id {
        Some(e) => e.0 as c_int,
        None => ENTITYNUM_NONE,
    }
}
