//! `EntityId` <-> `gentity_t*` conversion helpers.
//! Both halves of the `Option<EntityId>` retrofit live here (DEC-26), because they all do pointer
//! arithmetic over the concrete `gentity_t`, which lives in `mp_game` (`crate::entity::gentity`).
//!
//! - [`resolve`] is the index->pointer half.
//!   Raven code dereferences a `gentity_t*` field directly, for example `ent->activator`.
//!   That field is ported as `Option<EntityId>`, so call sites re-derive the live pointer via
//!   `base.add(id.index())` before dereferencing, matching Raven's `g_entities + entityNum` idiom.
//! - [`ent_id`]/[`ent_id_opt`] are the pointer->index half (Raven's `ent - g_entities`).
//!   They fill the stored `Option<EntityId>` fields at pointer-assignment sites.
//!
//! Source: `oracle/codemp/game/g_utils.c` (the `g_entities + i` / `ent - g_entities` idioms,
//! for example `G_Find`/`ENTITYNUM`).

use crate::entity::gentity_t;
use crate::world::EntityId;

/// This resolves a possibly-absent [`EntityId`] back to Raven's `gentity_t*`, given the `g_entities` array base.
/// It is the NULL-aware counterpart to `ent_id_opt`.
/// `None` maps to a null pointer, matching Raven's nullable `gentity_t*` fields.
///
/// # Safety
/// The caller guarantees that `base` points at the live, contiguous `g_entities` array.
/// This function confines the pointer arithmetic here, per §D11, so no caller does aliasing pointer math directly.
#[inline]
pub unsafe fn resolve(base: *mut gentity_t, id: Option<EntityId>) -> *mut gentity_t {
    match id {
        Some(id) => unsafe { base.add(id.index()) },
        None => core::ptr::null_mut(),
    }
}

/// This recovers an [`EntityId`] from a live `gentity_t*`, given the `g_entities` array base (Raven's `ent - g_entities`).
/// It is the pointer->index half of the retrofit.
/// The `Option<EntityId>` stored fields are filled via `Some(ent_id(base, ent))` at pointer-assignment sites.
/// They are reloaded via `&world.g_entities[id.index()]` at deref sites.
///
/// # Safety
/// The caller guarantees that `ent` points into the contiguous `g_entities` array whose first element is `base`.
/// This matches Raven's exact `ent - g_entities` arithmetic, since both are `#[repr(C)]` and the array is one allocation.
/// This function confines the pointer math here, per §D11, so safe module logic never does it directly.
///
/// Source: `oracle/codemp/game/g_utils.c` (the `ent - g_entities` idiom, for example `G_FreeEntity`/`ENTITYNUM`).
#[inline]
pub unsafe fn ent_id(base: *const gentity_t, ent: *const gentity_t) -> EntityId {
    // `offset_from` yields the element delta (Raven's `ent - g_entities`).
    EntityId(unsafe { ent.offset_from(base) } as u32)
}

/// This is the NULL-aware version of [`ent_id`]: NULL becomes `None`.
/// Raven's nullable `gentity_t*` maps to `Option<EntityId>`.
/// Entity 0 is valid, so there is no sentinel value.
///
/// # Safety
/// This has the same contract as [`ent_id`] when `ent` is non-null.
#[inline]
pub unsafe fn ent_id_opt(base: *const gentity_t, ent: *const gentity_t) -> Option<EntityId> {
    if ent.is_null() {
        None
    } else {
        Some(unsafe { ent_id(base, ent) })
    }
}
