//! `EntityId` — the entity handle (porting-rules §B5).

/// Raven's `gentity_t*` become an index into `GameWorld.entities`
/// (`mp_qshared::common::mp::gentity_t`, oracle home
/// `oracle/oracle/codemp/game/g_shared.h`). Module logic passes `(world, id)`
/// and re-indexes per access — GP2's `GpGroupId` precedent; no aliasing raw
/// pointers in safe code (§B5).
///
/// Source: `docs/architecture/state-ownership.md` § `EntityId` — the entity
/// handle (§B5).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EntityId(pub u32);

impl EntityId {
    /// The raw entity-array index.
    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// The `ent - g_entities` seam helper (ruling 22): recover an [`EntityId`] from a
/// live `gentity_t*` given the `g_entities` array base. This is the pointer→index
/// half of the fork-4 retrofit — the `Option<EntityId>` stored fields are filled
/// via `Some(ent_id(base, ent))` at pointer-assignment sites, and reloaded via
/// `&world.g_entities[id.index()]` at deref sites.
///
/// Unsafe seam (§D11): the caller guarantees `ent` points into the contiguous
/// `g_entities` array whose first element is `base` (Raven's exact
/// `ent - g_entities` arithmetic; both are `#[repr(C)]` and the array is one
/// allocation). Confined here so safe module logic never does pointer math.
///
/// Source: `oracle/oracle/codemp/game/g_utils.c` (the `ent - g_entities` idiom,
/// e.g. `G_FreeEntity`/`ENTITYNUM`).
#[inline]
pub unsafe fn ent_id(
    base: *const mp_qshared::common::mp::gentity::gentity_t,
    ent: *const mp_qshared::common::mp::gentity::gentity_t,
) -> EntityId {
    // `offset_from` yields the element delta (Raven's `ent - g_entities`).
    EntityId(unsafe { ent.offset_from(base) } as u32)
}

/// NULL-aware [`ent_id`]: Raven's nullable `gentity_t*` → `Option<EntityId>`
/// (ruling 22 — NULL becomes `None`; entity 0 is valid so there is no sentinel).
///
/// # Safety
/// Same contract as [`ent_id`] when `ent` is non-null.
#[inline]
pub unsafe fn ent_id_opt(
    base: *const mp_qshared::common::mp::gentity::gentity_t,
    ent: *const mp_qshared::common::mp::gentity::gentity_t,
) -> Option<EntityId> {
    if ent.is_null() {
        None
    } else {
        Some(unsafe { ent_id(base, ent) })
    }
}
