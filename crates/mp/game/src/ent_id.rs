//! `EntityId` -> `gentity_t*` seam helper (ruling 22): the index->pointer half
//! of the fork-4 retrofit, complementing `mp_qshared`'s pointer->index
//! `ent_id`/`ent_id_opt` (see `crate::world`/`crate::ent_id`).
//!
//! Raven code dereferences a `gentity_t*` field directly (e.g.
//! `ent->activator`); after ruling 22 that field is `Option<EntityId>`, so call
//! sites re-derive the live pointer via `base.add(id.index())` before
//! dereferencing — Raven's exact `g_entities + entityNum` idiom.
//!
//! Source: `oracle/oracle/codemp/game/g_utils.c` (the `g_entities + i` idiom,
//! e.g. `G_Find`/`ENT_STATE`); ruling 22, `docs/handoffs/jampgame-fork-discovery.md`.

use crate::world::EntityId;
use mp_qshared::common::mp::gentity::gentity_t;

/// Resolve a possibly-absent [`EntityId`] back to Raven's `gentity_t*`, given
/// the `g_entities` array base. NULL-aware counterpart to `ent_id_opt`: `None`
/// maps to a null pointer, matching Raven's nullable `gentity_t*` fields.
///
/// # Safety
/// Caller guarantees `base` points at the live, contiguous `g_entities` array
/// (§D11 seam confinement — no aliasing pointer math above this fn).
#[inline]
pub unsafe fn resolve(base: *mut gentity_t, id: Option<EntityId>) -> *mut gentity_t {
    match id {
        Some(id) => unsafe { base.add(id.index()) },
        None => core::ptr::null_mut(),
    }
}
