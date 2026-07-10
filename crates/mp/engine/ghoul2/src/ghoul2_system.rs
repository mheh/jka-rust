#![allow(non_camel_case_types, non_snake_case)]

//! The `Engine.g2` subsystem owner (`G2SV-D5`, ruling 12): one plain
//! `Default`-initialized aggregate holding every server-side Ghoul2 global the
//! survey found, folded from the former `RenderG2State` (bone caches +
//! `gore_shader`) per ruling 12.

use crate::gore::gore_set::GoreState;
use crate::info_array::Ghoul2InfoArray;
use crate::ragdoll::RagDollSolver;
use crate::render::bone_cache::CBoneCache;
use mp_qshared::shared::qhandle_t;

/// Raven `#define NUM_G2T_TIME (2)` — the `G2TimeBases` clock count.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:159`
pub const NUM_G2T_TIME: usize = 2;

/// Raven `#define MAX_G2_MODELS (1024)` (non-`_XBOX` arm) — the arena slot count.
///
/// `G2_INDEX_MASK = MAX_G2_MODELS - 1` extracts the slot index from a handle;
/// the arithmetic is duplicated in `info_array.rs`, which owns the arena proper
/// (`Ghoul2System::delete` only needs the mask to reach `delete_low(idx)`).
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:304,308`
const MAX_G2_MODELS: i32 = 1024;
const G2_INDEX_MASK: i32 = MAX_G2_MODELS - 1;

/// Key into `Ghoul2System.bone_caches` — the hand-rolled in-crate generational
/// arena of `CBoneCache` (`G2SV-D9`; §B5 arena, **no** external `slotmap` crate).
///
/// Raven had a raw `CBoneCache *mBoneCache` per `CGhoul2Info`
/// (`ghoul2_shared.h:265`); the port replaces the aliasing pointer with this
/// generational handle so no raw pointer escapes the ABI seam (§B5/§D11).
/// Shape is free (`§A1`): unlike the ABI-frozen `Ghoul2InfoArray` handle, the
/// bone cache had no ABI surface, so this carries no bit-exact-vs-oracle burden.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct BoneCacheId {
    index: u32,
    generation: u32,
}

/// The hand-rolled owned generational arena of `CBoneCache` (`G2SV-D9`; §B5,
/// same kind as `Ghoul2InfoArray`, hand-rolled in-crate, not a `slotmap` crate).
///
/// Raven owned each `CBoneCache` through a raw `CBoneCache *mBoneCache`
/// (`ghoul2_shared.h:265`) `new`'d in `G2_ConstructGhoulSkeleton` and `delete`'d
/// via `RemoveBoneCache` (`tr_ghoul2.cpp:569`); here the arena owns them and
/// `BoneCacheId` handles reference them.
#[derive(Default)]
pub struct BoneCacheArena {
    slots: Vec<Option<CBoneCache>>,
    generations: Vec<u32>,
    free: Vec<u32>,
}

impl BoneCacheArena {
    /// Insert an owned cache, returning its handle (Raven `new CBoneCache(...)`
    /// in `G2_ConstructGhoulSkeleton`, `tr_ghoul2.cpp`).
    pub fn insert(&mut self, cache: CBoneCache) -> BoneCacheId {
        if let Some(index) = self.free.pop() {
            let i = index as usize;
            self.slots[i] = Some(cache);
            BoneCacheId {
                index,
                generation: self.generations[i],
            }
        } else {
            let index = self.slots.len() as u32;
            self.slots.push(Some(cache));
            self.generations.push(0);
            BoneCacheId {
                index,
                generation: 0,
            }
        }
    }

    /// Borrow the cache a live handle refers to (stale/removed handles → `None`).
    pub fn get(&self, id: BoneCacheId) -> Option<&CBoneCache> {
        let i = id.index as usize;
        if self.generations.get(i).copied() == Some(id.generation) {
            self.slots[i].as_ref()
        } else {
            None
        }
    }

    /// Mutably borrow the cache a live handle refers to.
    pub fn get_mut(&mut self, id: BoneCacheId) -> Option<&mut CBoneCache> {
        let i = id.index as usize;
        if self.generations.get(i).copied() == Some(id.generation) {
            self.slots[i].as_mut()
        } else {
            None
        }
    }

    /// Free the cache a handle refers to and bump the slot's generation (Raven
    /// `RemoveBoneCache`, `tr_ghoul2.cpp:569`; the `delete` owner is the arena,
    /// reached from `Ghoul2System::delete_low`, `G2SV-D13`(a)).
    pub fn remove(&mut self, id: BoneCacheId) {
        let i = id.index as usize;
        if self.generations.get(i).copied() == Some(id.generation) && self.slots[i].is_some() {
            self.slots[i] = None;
            self.generations[i] = self.generations[i].wrapping_add(1);
            self.free.push(id.index);
        }
    }
}

/// The `Engine.g2` field (`G2SV-D5`, ruling 12) — the one owned instance of every
/// server-side Ghoul2 global, `Default`-initialized (no `Option`/`Box`/nesting;
/// Raven's lazy `TheGhoul2InfoArray()` singleton and the render-side state both
/// fold in here). Threaded `&mut Ghoul2System` into every `G2API_*` entry.
///
/// Raven had these as scattered file-scope globals: the `Ghoul2InfoArray`
/// singleton (`G2_API.cpp:477`), `G2TimeBases` (`:160`), the `GetBoltMatrix`
/// reconstruct flags (`:1724-1725`), the gore store (`G2_misc.cpp:35,125`), the
/// ragdoll fn-statics block (`G2_bones.cpp:1214-1241`), the per-instance
/// `CBoneCache *mBoneCache` (`ghoul2_shared.h:265`), and `goreShader`
/// (`tr_ghoul2.cpp:139`).
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:160,477,1724-1725`
#[derive(Default)]
pub struct Ghoul2System {
    /// Raven `Ghoul2InfoArray *singleton` (`G2_API.cpp:477`) — the model-instance
    /// arena, lazily `new`'d in Raven, eagerly `Default`-seeded here.
    pub info_array: Ghoul2InfoArray,

    /// Raven `static int G2TimeBases[NUM_G2T_TIME]` (`G2_API.cpp:160`), driving
    /// `G2API_SetTime`/`GetTime`.
    pub time_bases: [i32; NUM_G2T_TIME],

    /// Raven `gG2_GBMNoReconstruct` (`G2_API.cpp:1724`) — `GetBoltMatrix`
    /// reconstruct-skip flag.
    pub gbm_no_reconstruct: bool,

    /// Raven `gG2_GBMUseSPMethod` (`G2_API.cpp:1725`) — `GetBoltMatrix` SP-method
    /// flag.
    pub gbm_use_sp_method: bool,

    /// Raven `GoreRecords`/`GoreSets`/`CurrentTag`/`GoreTouch`… gore store
    /// (`G2_misc.cpp:35,125`), `_G2_GORE` on (`G2SV-D5`).
    pub gore: GoreState,

    /// Raven ragdoll/IK fn-statics block (`G2_bones.cpp:1214-1241`), server-live
    /// (`G2SV-D3`; ruling 3 cross-frame kind).
    pub rag: RagDollSolver,

    /// The per-instance `CBoneCache`s (Raven `CBoneCache *mBoneCache`,
    /// `ghoul2_shared.h:265`), folded from the former `RenderG2State` per ruling
    /// 12 into this owned generational arena (`G2SV-D9`).
    pub bone_caches: BoneCacheArena,

    /// Raven `goreShader` (`tr_ghoul2.cpp:139`, `_G2_GORE`), folded from the
    /// former `RenderG2State` per ruling 12.
    pub gore_shader: qhandle_t,
}

impl Ghoul2System {
    /// Raven `Ghoul2InfoArray::Delete(int handle)` (`G2_API.cpp:413`): a
    /// no-op on the null handle, else `delete_low(handle & G2_INDEX_MASK)`.
    ///
    /// Moved UP from the arena to `Ghoul2System` (`G2SV-D13`(a) / ruling 29)
    /// because the teardown frees bone caches from the sibling `bone_caches`
    /// arena, unreachable from `Ghoul2InfoArray` alone.
    /// Source: `oracle/codemp/ghoul2/G2_API.cpp:413-427`
    pub fn delete(&mut self, handle: i32) {
        if handle == 0 {
            return;
        }
        if self.info_array.is_valid(handle) {
            self.delete_low(handle & G2_INDEX_MASK);
        }
    }

    /// Raven `Ghoul2InfoArray::DeleteLow(int idx)` (`G2_API.cpp:315`), split by
    /// `G2SV-D13`(a): (1) free every model instance's bone cache
    /// (`RemoveBoneCache`, `:319-326`) from the sibling `bone_caches` arena, then
    /// (2) clear the slot + bump generation / free-list via
    /// `Ghoul2InfoArray::clear_slot` (`:328-339`).
    /// Source: `oracle/codemp/ghoul2/G2_API.cpp:315-339`
    fn delete_low(&mut self, idx: i32) {
        // (1) free each model instance's bone cache (G2_API.cpp:319-326).
        let ids: Vec<BoneCacheId> = self
            .info_array
            .get_mut(idx)
            .iter_mut()
            .filter_map(|info| info.bone_cache.take())
            .collect();
        for id in ids {
            self.bone_caches.remove(id);
        }
        // (2) slot bookkeeping: clear mInfos[idx] + generation bump (:328-339).
        self.info_array.clear_slot(idx);
    }
}
