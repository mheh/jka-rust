//! `ModelPool` — Raven's `tr.models[]`/`tr.numModels` registry carrying the R2
//! arena mechanics in place.
//!
//! Design: `docs/subsystems/tr-model.md` `## Amendment 2026-07-27 — models
//! pool: arena mechanics` (#51), with DEC-42.1 (`Arena::reset`), DEC-42.2
//! ("slot = index") and DEC-37 A12 (slot-0 pre-population) as the mechanics
//! this file reproduces.
//!
//! **Why a local pool rather than `render_state::arena::Arena<T>`.** Two
//! independent reasons, both fatal to reuse:
//!
//! 1. `Arena<T>`'s `#[derive(Clone)]` exists for `Arc::make_mut` on
//!    `RenderAssets` (A9/NB-1). A pool entry here is a `model_t` owning the
//!    `mdxm`/`mdxa` raw block pointers the DEC-35 mdx views hand out, so it is
//!    deliberately not `Clone` and never enters an `Arc`-published registry.
//!    DEC-65 ruling 1 publishes the blocks themselves instead, as
//!    `Arc<ModelBlocks>` on `RenderAssets`. The pool keeps its `Box` entries
//!    and its address-stability contract unchanged.
//! 2. `Arena::insert` hands out slots from a LIFO free list. Raven's
//!    `R_AllocModel` hands out `tr.numModels` — a strictly sequential
//!    high-water mark — and the only vacating operations
//!    (`R_ModelInit`/`R_HunkClearCrap`) reset that mark to `0` rather than
//!    freeing individual slots. Routing allocation through a free list would
//!    renumber every post-reset handle, and `qhandle_t`s are observable at the
//!    G2/server seam (DEC-42.2), so the lockstep referee would see it.
//!
//! What is adopted from `Arena<T>` verbatim: slot-0 reservation with the
//! registry's live default entry (A12 — slot 0 never vacates and never bumps
//! its generation, so `Handle::slot_zero()` is the persistent default identity
//! across lives), per-slot generation counting with a bump on vacate
//! (ruling 11), and `handle_at_slot` as the bare-int → handle resolution
//! (DEC-42.2).

use core::mem;

use mp_qshared::shared::qhandle_t;

use crate::render_state::handle::Handle;
use crate::tr_local::model_s::model_t;

/// `MAX_MOD_KNOWN` — the pool's soft cap; `R_AllocModel` returns `NULL` at it.
///
/// Source: `oracle/codemp/renderer/tr_local.h:1138`
pub const MAX_MOD_KNOWN: usize = 1024;

/// `ModelData` — the `tr.models[]` pool entry (ruling 40 reuse: the already-
/// ported `model_t`, imported never re-declared; a thin wrapper vs the bare
/// `model_t` is §D12 porter latitude, and the skeleton pins the direct reuse).
///
/// Type reuse source: `oracle/codemp/renderer/tr_local.h:1117-1135`
pub type ModelData = model_t;

/// A generation-counted handle into [`ModelPool`] — Raven's `model_t *`, the
/// pointer form `tr_ghoul2.cpp`'s render-pass data holders carry.
///
/// The oracle's own `qhandle_t` (a bare `int` = the slot number, DEC-42.2) is
/// what crosses the G2/server seam; this typed handle is the in-crate form for
/// code that held a `model_t *` rather than an index.
pub type ModelHandle = Handle<ModelData>;

/// One pool slot: the generation ruling 11 counts, plus the `Box`-pinned
/// entry. `Box` is load-bearing — `G2_API.cpp:2716` caches the registered
/// `model_t *`, and the DEC-35 mdx views read the `mdxm`/`mdxa` blocks out of
/// this entry, so the entry's address must not move when the pool grows.
/// The render thread reads the published `Arc<ModelBlocks>` copy instead, which holds byte offsets rather than
/// these pointers (DEC-65 ruling 1).
struct ModelSlot {
    generation: u32,
    data: Box<ModelData>,
}

/// Raven's `tr.models[MAX_MOD_KNOWN]` + `tr.numModels`, as one owner (§B3).
///
/// Slots below `num_models` are live; slots at or above it are the vacated
/// leftovers Raven's array keeps after a `tr.numModels = 0` reset, reachable
/// only through [`ModelPool::slot`] and re-used in order by the next
/// [`ModelPool::alloc`] run.
///
/// Source: `oracle/codemp/renderer/tr_local.h:1396-1397`;
/// `oracle/codemp/renderer/tr_model.cpp:591-624,1665-1680,1682-1690`
pub struct ModelPool {
    slots: Vec<ModelSlot>,
    /// `tr.numModels` — the high-water mark, and the bound
    /// `R_GetModelByHandle`/`R_Modellist_f` range-check against.
    num_models: i32,
}

impl Default for ModelPool {
    fn default() -> Self {
        ModelPool::new()
    }
}

impl ModelPool {
    /// A pool with slot 0 pre-populated with the reserved default entry (A12).
    ///
    /// The entry is a zeroed `model_t`, i.e. `MOD_BAD` with a null `md3`/
    /// `mdxm`/`mdxa` — the same object `R_ModelInit` re-creates at slot 0 on
    /// every life. Raven's `tr.models[0]` before `R_ModelInit` runs is an
    /// uninitialised pointer and reading it is UB; pre-populating is the one
    /// defined behavior A12 picks (§19), and it makes [`Self::by_handle`]
    /// total.
    pub fn new() -> ModelPool {
        ModelPool {
            slots: vec![ModelSlot {
                generation: 0,
                data: Self::blank_entry(0),
            }],
            num_models: 0,
        }
    }

    /// A zeroed pool entry with `->index` set, mirroring `R_AllocModel`'s
    /// `Hunk_Alloc` (which returns zeroed memory).
    fn blank_entry(index: qhandle_t) -> Box<ModelData> {
        // SAFETY: every `model_t` field has a valid all-zero bit pattern
        // (`modtype_t::MOD_BAD == 0`, null raw pointers, `qboolean::qfalse ==
        // 0`) and the struct holds no references. Confined internal `unsafe`,
        // the same quarantine the pre-migration `r_alloc_model` used.
        let mut entry: Box<ModelData> = Box::new(unsafe { mem::zeroed() });
        entry.index = index;
        entry
    }

    /// `tr.numModels`.
    pub fn num_models(&self) -> i32 {
        self.num_models
    }

    /// Raven `R_AllocModel` — `Hunk_Alloc`s the next `model_t`, sets
    /// `->index = tr.numModels`, stores it at `tr.models[tr.numModels]`, and
    /// returns `NULL` (here `None`) at the `MAX_MOD_KNOWN` cap.
    ///
    /// A slot vacated by [`Self::reset`] is re-used in place at the generation
    /// that reset assigned it, so no second bump happens here (mirroring
    /// `Arena::reset` + `Arena::insert`). Slot 0 stays at generation 0 across
    /// lives (A12).
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:611-624`
    pub fn alloc(&mut self) -> Option<qhandle_t> {
        if self.num_models == MAX_MOD_KNOWN as i32 {
            return None;
        }

        let index = self.num_models;
        let entry = Self::blank_entry(index);
        let slot = index as usize;
        if slot < self.slots.len() {
            self.slots[slot].data = entry;
        } else {
            self.slots.push(ModelSlot {
                generation: 0,
                data: entry,
            });
        }
        self.num_models += 1;

        Some(index)
    }

    /// The `tr.numModels = 0` half of `R_ModelInit`/`R_HunkClearCrap` — the
    /// registry teardown DEC-42.1 spells `Arena::reset` for the capped
    /// arenas.
    ///
    /// Raven leaves the `tr.models[]` array itself alone (the entries died
    /// with the Hunk); the mark drop is what makes every live handle
    /// out-of-range. Vacating bumps each live slot's generation so every
    /// pre-reset [`ModelHandle`] goes stale (ruling 11), except slot 0, whose
    /// generation stays 0 so `Handle::slot_zero()` remains the persistent
    /// default identity (A12).
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:1673-1674,1686-1687`
    pub fn reset(&mut self) {
        let live = (self.num_models.max(0) as usize).min(self.slots.len());
        for slot in self.slots[..live].iter_mut().skip(1) {
            slot.generation = slot.generation.wrapping_add(1);
        }
        self.num_models = 0;
    }

    /// Raven `R_GetModelByHandle` — out-of-range (`< 1` or `>= tr.numModels`)
    /// falls back to slot 0, the default/NULL model.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:591-604`
    pub fn by_handle(&self, handle: qhandle_t) -> &ModelData {
        // out of range gets the default model
        if handle < 1 || handle >= self.num_models {
            return &self.slots[0].data;
        }
        &self.slots[handle as usize].data
    }

    /// Raw slot read — the `tr.models[i]` access oracle code performs while
    /// holding the bare `int` index `R_AllocModel` just handed it (DEC-42.2
    /// "slot = index"). No range fallback: the caller owns the slot.
    pub fn slot(&self, slot: usize) -> &ModelData {
        &self.slots[slot].data
    }

    /// Raw slot write — [`Self::slot`]'s mutable twin, the form the model
    /// loaders fill a freshly allocated entry through.
    pub fn slot_mut(&mut self, slot: usize) -> &mut ModelData {
        &mut self.slots[slot].data
    }

    /// The CURRENT handle occupying `slot`, if the slot is live — the bare-int
    /// → handle resolution for oracle code that stores plain `qhandle_t`
    /// indices (DEC-42.2). A stale int after a registry reset resolves to
    /// `None` rather than to a foreign occupant, because the reset drops the
    /// high-water mark below it.
    pub fn handle_at_slot(&self, slot: u32) -> Option<ModelHandle> {
        if slot as i32 >= self.num_models {
            return None;
        }
        let entry = self.slots.get(slot as usize)?;
        Some(Handle::new(slot, entry.generation))
    }

    /// Generation-checked read; `None` for a handle from a previous life.
    pub fn get(&self, handle: ModelHandle) -> Option<&ModelData> {
        let entry = self.slots.get(handle.index() as usize)?;
        if entry.generation != handle.generation() || handle.index() as i32 >= self.num_models {
            return None;
        }
        Some(&entry.data)
    }

    /// Generation-checked write; `None` for a handle from a previous life.
    pub fn get_mut(&mut self, handle: ModelHandle) -> Option<&mut ModelData> {
        let num_models = self.num_models;
        let entry = self.slots.get_mut(handle.index() as usize)?;
        if entry.generation != handle.generation() || handle.index() as i32 >= num_models {
            return None;
        }
        Some(&mut entry.data)
    }

    /// The registered entries `R_Modellist_f` walks — `for (i = 1;
    /// i < tr.numModels; i++)`, so slot 0 (the reserved default) is skipped.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:1712`
    pub fn registered(&self) -> impl Iterator<Item = &ModelData> + '_ {
        let live = (self.num_models.max(0) as usize).min(self.slots.len());
        self.slots[..live].iter().skip(1).map(|slot| &*slot.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tr_local::modtype_t::modtype_t;

    #[test]
    fn slot_zero_is_pre_populated_with_the_default_entry() {
        let pool = ModelPool::new();
        assert_eq!(pool.num_models(), 0);
        assert_eq!(pool.slot(0).index, 0);
        assert!(matches!(pool.slot(0).r#type, modtype_t::MOD_BAD));
        // Every out-of-range handle resolves to that entry.
        assert_eq!(pool.by_handle(0).index, 0);
        assert_eq!(pool.by_handle(-5).index, 0);
        assert_eq!(pool.by_handle(99).index, 0);
    }

    #[test]
    fn alloc_hands_out_sequential_slots_and_caps() {
        let mut pool = ModelPool::new();
        assert_eq!(pool.alloc(), Some(0));
        assert_eq!(pool.alloc(), Some(1));
        assert_eq!(pool.num_models(), 2);
        assert_eq!(pool.slot(1).index, 1);
        while pool.num_models() < MAX_MOD_KNOWN as i32 {
            assert!(pool.alloc().is_some());
        }
        assert_eq!(pool.alloc(), None);
        assert_eq!(pool.num_models(), MAX_MOD_KNOWN as i32);
    }

    #[test]
    fn reset_renumbers_from_zero_and_stales_handles_above_slot_zero() {
        let mut pool = ModelPool::new();
        pool.alloc();
        pool.alloc();
        let h0 = pool.handle_at_slot(0).unwrap();
        let h1 = pool.handle_at_slot(1).unwrap();

        pool.reset();
        assert_eq!(pool.num_models(), 0);

        // Raven re-allocates from index 0, never from a free list.
        assert_eq!(pool.alloc(), Some(0));
        assert_eq!(pool.alloc(), Some(1));

        // Slot 0 keeps its generation-0 identity (A12); slot 1's pre-reset
        // handle is stale (ruling 11).
        assert_eq!(h0.generation(), 0);
        assert!(pool.get(h0).is_some());
        assert!(pool.get(h1).is_none());
        assert_eq!(pool.handle_at_slot(1).unwrap().generation(), 1);
    }

    #[test]
    fn registered_walks_slot_one_upwards() {
        let mut pool = ModelPool::new();
        pool.alloc();
        pool.alloc();
        pool.alloc();
        let indices: Vec<i32> = pool.registered().map(|m| m.index).collect();
        assert_eq!(indices, vec![1, 2]);
    }
}
