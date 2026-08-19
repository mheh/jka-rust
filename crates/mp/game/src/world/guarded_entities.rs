//! `GuardedEntities` - the `g_entities` arena with one guard slot before index 0.

use core::ops::{Deref, DerefMut};

use mp_qshared::shared::{ENTITYNUM_NONE, MAX_GENTITIES};

use crate::entity::gentity_t;

/// The `g_entities` storage, with one extra entity placed one stride before element 0.
///
/// Raven's engine dereferences `SV_GentityNum(-1)` when the bot AI traces with `passEntityNum = -1`
/// (`oracle/codemp/server/sv_world.cpp:534-543`, reached from `OrgVisible`, `oracle/codemp/game/ai_main.c:986`).
/// With Raven's static C array that out-of-bounds read lands in adjacent module data and is harmless.
/// With a heap allocation it can land on an unmapped page, and a stock engine then segfaults inside `SV_Trace`.
/// The guard slot provides owned memory for that read, so the module stays a drop-in replacement under engines we do not control.
///
/// The guard's `r.ownerNum` is `ENTITYNUM_NONE` and its `r.svFlags` is 0, so a stock engine computes `passOwnerNum = -1` and "owner shared"
/// - the same defined behavior our own engine picked for the -1 case (`crates/mp/engine/server/src/sv_world.rs:601-615`).
///
/// `Deref` targets the real array, so `world.g_entities[i]` and `.as_mut_ptr()` keep their meaning.
/// Element 0 is the first real entity, and the guard is reachable only by the engine's own negative index.
#[repr(C)]
pub struct GuardedEntities {
    /// The engine's `gentities[-1]` target. Never a live entity.
    pub guard: gentity_t,
    /// `g_entities[MAX_GENTITIES]` (`oracle/codemp/game/g_main.c:27`).
    pub entities: [gentity_t; MAX_GENTITIES],
}

// The engine reaches the guard as `entities_base - gentitySize`, so the guard must sit exactly one stride before element 0 with no padding between.
const _: () = {
    assert!(
        core::mem::offset_of!(GuardedEntities, entities) == core::mem::size_of::<gentity_t>()
    );
};

impl GuardedEntities {
    /// Sets the guard's required fields after zero-initialization.
    /// The caller already sets the owned strings.
    pub fn seat_guard(&mut self) {
        self.guard.r.ownerNum = ENTITYNUM_NONE;
    }
}

impl Deref for GuardedEntities {
    type Target = [gentity_t; MAX_GENTITIES];

    fn deref(&self) -> &Self::Target {
        &self.entities
    }
}

impl DerefMut for GuardedEntities {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entities
    }
}
