//! `EntityId` — the entity handle (porting-rules §B5), hoisted to `mp_qshared`
//! so `gentity_t`'s stored entity fields can name it.
//!
//! `gentity_t` lives in this crate (the abi seam names `*mut gentity_t`), and its
//! 38 stored `gentity_t*` struct fields are `Option<EntityId>`. That field type must
//! therefore be visible in `mp_qshared`. `mp_game` re-exports it (via
//! `crate::world::EntityId` + the prelude) so every existing use keeps compiling.

use core::ffi::c_int;

use crate::common::mp::gentity::gentity_t;
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

/// The `ent - g_entities` seam helper: recover an [`EntityId`] from a
/// live `gentity_t*` given the `g_entities` array base. This is the pointer→index
/// half of the retrofit — the `Option<EntityId>` stored fields are filled
/// via `Some(ent_id(base, ent))` at pointer-assignment sites, and reloaded via
/// `&world.g_entities[id.index()]` at deref sites.
///
/// Unsafe seam (§D11): the caller guarantees `ent` points into the contiguous
/// `g_entities` array whose first element is `base` (Raven's exact
/// `ent - g_entities` arithmetic; both are `#[repr(C)]` and the array is one
/// allocation). Confined here so safe module logic never does pointer math.
///
/// Source: `oracle/codemp/game/g_utils.c` (the `ent - g_entities` idiom,
/// e.g. `G_FreeEntity`/`ENTITYNUM`).
#[inline]
pub unsafe fn ent_id(base: *const gentity_t, ent: *const gentity_t) -> EntityId {
    // `offset_from` yields the element delta (Raven's `ent - g_entities`).
    EntityId(unsafe { ent.offset_from(base) } as u32)
}

/// NULL-aware [`ent_id`]: Raven's nullable `gentity_t*` → `Option<EntityId>`
/// (NULL becomes `None`; entity 0 is valid so there is no sentinel).
///
/// # Safety
/// Same contract as [`ent_id`] when `ent` is non-null.
#[inline]
pub unsafe fn ent_id_opt(base: *const gentity_t, ent: *const gentity_t) -> Option<EntityId> {
    if ent.is_null() {
        None
    } else {
        Some(unsafe { ent_id(base, ent) })
    }
}
