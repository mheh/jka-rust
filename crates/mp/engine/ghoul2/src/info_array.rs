#![allow(non_camel_case_types, non_snake_case)]

//! Raven `Ghoul2InfoArray` (`G2_API.cpp:310-481`) — the model-instance arena
//! behind the abstract `IGhoul2InfoArray` interface (`ghoul2_shared.h:316-322`).
//! `IGhoul2InfoArray` has exactly one implementor in this codebase
//! (`Ghoul2InfoArray` itself), so per porting-rules §F17 ("interface classes →
//! the arena/handle they hide") the interface collapses into
//! `Ghoul2InfoArray`'s own inherent `impl` — no separate Rust trait is
//! introduced.
//!
//! Colocated per the roster (`docs/subsystems/ghoul2-server.md` files roster,
//! `info_array.rs` row): the `Ghoul2Handle` newtype (the §B5 arena id) and the
//! free functions that reached the former file-scope `singleton` —
//! `TheGhoul2InfoArray`/`Ghoul2InfoArray_Free` (`G2_API.cpp:477-493`).
//!
//! **Not here** (moved/colocated elsewhere per frozen decisions):
//! - `Delete`/`DeleteLow` move UP to `Ghoul2System::delete`/`delete_low`
//!   (`ghoul2_system.rs`, `G2SV-D13`(a) / ruling 29): the teardown also frees
//!   bone caches from the sibling `Ghoul2System.bone_caches` arena, unreachable
//!   from `Ghoul2InfoArray` alone. Only `DeleteLow`'s slot-bookkeeping half
//!   survives here as [`Ghoul2InfoArray::clear_slot`].
//! - `CGhoul2Info_v`'s forwarding/lifecycle methods (`operator[]`/`resize`/
//!   `size`/`push_back`/`Alloc`/`Free`/`clear`/`DeepCopy`/`operator=`) colocate
//!   with the frozen handle struct in `shared/cghoul2_info_v.rs` (`G2SV-D10`,
//!   ruling 22, §F21) and forward into [`Ghoul2InfoArray::get`]/
//!   [`Ghoul2InfoArray::get_mut`] below — they are not transcribed here.
//!
//! Dropped, `G2API_DEBUG`-only surface (off in this build; debug alloc/leak
//! tracking has no parity surface, divergences list): `~Ghoul2InfoArray`'s
//! leak-report destructor (`:351-386`), `GetDebug` (`:447-455`), and
//! `TestAllAnims` (`:457-471`) — no roster row, no stub.

use crate::ghoul2_system::Ghoul2System;
use crate::shared::cghoul2_info::CGhoul2Info;
use std::collections::VecDeque;

/// Raven `#define MAX_G2_MODELS (1024)` (non-`_XBOX` arm) — the arena's fixed
/// slot count.
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:304`
const MAX_G2_MODELS: usize = 1024;

/// Raven `#define G2_MODEL_BITS (10)` (non-`_XBOX` arm) — width of the slot
/// index packed into a handle's low bits; the high bits are the generation
/// counter (`New`'s rollover test at `:330` reads `1<<(31-G2_MODEL_BITS)`).
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:305`
const G2_MODEL_BITS: i32 = 10;

/// Raven `#define G2_INDEX_MASK (MAX_G2_MODELS-1)` — extracts the slot index
/// from a handle (`handle & G2_INDEX_MASK`).
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:308`
const G2_INDEX_MASK: i32 = MAX_G2_MODELS as i32 - 1;

/// The §B5 arena id: packs `slot | generation` bit-exact vs the oracle
/// (`G2SV-D6`) into a single `i32`, the same value stored into the
/// already-ported `CGhoul2Info_v.mItem: i32` ABI field.
///
/// Only the `CGhoul2Info_v.mItem` ABI layout and the packed handle *value* are
/// frozen (`G2SV-D6`); this newtype wrapper's own shape is free internal
/// latitude (porting-rules §A1) — the inherent methods below take/return the
/// raw packed `i32` directly, matching the doc's frozen `## Seam definition`
/// signatures.
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:304-341` (handle arithmetic)
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct Ghoul2Handle(pub i32);

/// Raven `Ghoul2InfoArray` (`G2_API.cpp:310-481`) — the fixed
/// `MAX_G2_MODELS`-slot arena of `vector<CGhoul2Info>` behind
/// `IGhoul2InfoArray`, reached in Raven through the lazy file-scope
/// `singleton` / `TheGhoul2InfoArray()` (`:477`); here it lives as the plain
/// `Ghoul2System.info_array` field (`G2SV-D5`).
///
/// Raven: (none).
/// Type definition source: `oracle/codemp/ghoul2/G2_API.cpp:310-481`
pub struct Ghoul2InfoArray {
    /// Raven `vector<CGhoul2Info> mInfos[MAX_G2_MODELS]` (`:312`) — one model
    /// list per slot.
    mInfos: Vec<Vec<CGhoul2Info>>,
    /// Raven `int mIds[MAX_G2_MODELS]` (`:313`) — per-slot packed
    /// `slot | generation` id; `New()` returns this, `IsValid`/`Get` compare
    /// against it.
    mIds: Vec<i32>,
    /// Raven `list<int> mFreeIndecies` (`:314`) — free-slot queue. `New()`
    /// pops the front (`begin()`/`erase(begin())`); `DeleteLow` pushes front
    /// (still-valid rollover, `:339`) or back (generation exhausted, `:335`).
    /// `VecDeque` supports both ends, matching `list<int>`'s push/pop shape.
    free_indices: VecDeque<i32>,
    /// Raven `Get(int handle)`'s shared function-`static vector<CGhoul2Info>
    /// null` (`:429`), `.clear()`d on every invalid-handle hit (`:435`) —
    /// ported as an owned field instead of a hidden static (porting-rules
    /// §B3), so [`Ghoul2InfoArray::get_mut`] still has somewhere to return
    /// `&mut` into on the divergent invalid-handle path (§19, kept out of
    /// shared fixtures).
    null_slot: Vec<CGhoul2Info>,
}

impl Default for Ghoul2InfoArray {
    /// Raven `Ghoul2InfoArray::Ghoul2InfoArray()` (`G2_API.cpp:341-349`):
    /// seeds `mIds[i] = MAX_G2_MODELS + i` for every slot and pushes every
    /// index onto the free list in order (`0..MAX_G2_MODELS`).
    fn default() -> Self {
        let mut mIds = Vec::with_capacity(MAX_G2_MODELS);
        let mut free_indices = VecDeque::with_capacity(MAX_G2_MODELS);
        for i in 0..MAX_G2_MODELS as i32 {
            mIds.push(MAX_G2_MODELS as i32 + i);
            free_indices.push_back(i);
        }
        Self {
            mInfos: (0..MAX_G2_MODELS).map(|_| Vec::new()).collect(),
            mIds,
            free_indices,
            null_slot: Vec::new(),
        }
    }
}

impl Ghoul2InfoArray {
    /// Raven `Ghoul2InfoArray::New()` (`G2_API.cpp:386-398`): asserts +
    /// `Com_Error(ERR_FATAL, ...)` when the free list is empty, else pops the
    /// front free index and returns its packed `mIds[idx]`.
    pub fn new_handle(&mut self) -> i32 {
        let idx = match self.free_indices.pop_front() {
            Some(idx) => idx,
            // Com_Error(ERR_FATAL, "Out of ghoul2 info slots") -> panic (frozen Group A).
            None => panic!("Out of ghoul2 info slots"),
        };
        self.mIds[idx as usize]
    }

    /// Raven `Ghoul2InfoArray::IsValid(int handle) const` (`:399-408`): false
    /// on the null (`0`) handle; else compares `mIds[handle & G2_INDEX_MASK]`
    /// against `handle` — a stale or garbage handle reads false.
    pub fn is_valid(&self, handle: i32) -> bool {
        if handle == 0 {
            return false;
        }
        self.mIds[(handle & G2_INDEX_MASK) as usize] == handle
    }

    /// Raven `vector<CGhoul2Info> &Ghoul2InfoArray::Get(int handle)`, the
    /// `const` overload (`:427-436`): an invalid handle returns Raven's shared
    /// function-static null vector (first `.clear()`d, non-reentrant
    /// aliasing) — ported as an empty slice (divergence, kept out of shared
    /// fixtures per §F19). [`Ghoul2InfoArray::get_mut`] is the mutable
    /// counterpart `CGhoul2Info_v`'s forwarding methods
    /// (`shared/cghoul2_info_v.rs`, `G2SV-D10`) call into.
    pub fn get(&self, handle: i32) -> &[CGhoul2Info] {
        if handle <= 0 || self.mIds[(handle & G2_INDEX_MASK) as usize] != handle {
            return &[];
        }
        &self.mInfos[(handle & G2_INDEX_MASK) as usize]
    }

    /// Raven `vector<CGhoul2Info> &Ghoul2InfoArray::Get(int handle)`, the
    /// non-`const` overload (`:427-436`) — same invalid-handle empty-result
    /// divergence as [`Ghoul2InfoArray::get`].
    pub fn get_mut(&mut self, handle: i32) -> &mut Vec<CGhoul2Info> {
        if handle <= 0 || self.mIds[(handle & G2_INDEX_MASK) as usize] != handle {
            self.null_slot.clear();
            return &mut self.null_slot;
        }
        &mut self.mInfos[(handle & G2_INDEX_MASK) as usize]
    }

    /// Raven `Ghoul2InfoArray::DeleteLow(int idx)`'s **slot half only**
    /// (`:315-339`): clears `mInfos[idx]` and bumps the packed generation,
    /// pushing the slot back onto the free list — front on the ordinary path
    /// (`:339`), or back with the id reset to `MAX_G2_MODELS+idx` when the
    /// generation would roll over past `1<<(31-G2_MODEL_BITS)` (`:330-335`).
    ///
    /// The bone-cache-freeing half (`RemoveBoneCache` loop, `:319-326`) is
    /// **not** here — it moved UP to `Ghoul2System::delete_low`
    /// (`G2SV-D13`(a), ruling 29) because those caches live in the sibling
    /// `Ghoul2System.bone_caches` arena, unreachable from this struct alone.
    pub(crate) fn clear_slot(&mut self, handle: i32) {
        let idx = handle as usize;
        self.mInfos[idx].clear();

        if (self.mIds[idx] >> G2_MODEL_BITS) > (1 << (31 - G2_MODEL_BITS)) {
            self.mIds[idx] = MAX_G2_MODELS as i32 + idx as i32; // rollover reset id to minimum value
            self.free_indices.push_back(idx as i32);
        } else {
            // `wrapping_add`, not `+=`: matches Raven's own signed-overflow
            // wraparound (reachable only after ~2^21 reuses of one slot) instead of an unrequested Rust debug-mode panic.
            self.mIds[idx] = self.mIds[idx].wrapping_add(MAX_G2_MODELS as i32);
            self.free_indices.push_front(idx as i32);
        }
    }
}

/// Raven `IGhoul2InfoArray &TheGhoul2InfoArray()` (`G2_API.cpp:478-484`): the
/// lazy file-scope `singleton` accessor (`new`s it on first call).
///
/// State is threaded, not reached (porting-rules §B3/§B4):
/// `Ghoul2System.info_array` is a plain `Default`-initialized field
/// (`G2SV-D5`), so there is no lazy `new` left to guard — this free fn is the
/// mechanical stand-in for the former global accessor, reached by threading
/// `&mut Ghoul2System` through instead of a hidden static.
pub fn the_ghoul2_info_array(g2: &mut Ghoul2System) -> &mut Ghoul2InfoArray {
    &mut g2.info_array
}

/// Raven `void Ghoul2InfoArray_Free(void)` (`G2_API.cpp:487-493`): `delete`s
/// the singleton and nulls the pointer.
///
/// `Ghoul2System.info_array` is an owned field freed by ordinary `Drop` when
/// `Ghoul2System` itself drops — there is no singleton pointer left to null.
/// Kept as a callable stub per the roster's explicit `info_array.rs`
/// placement rather than dropped, matching the doc's roster row.
pub fn ghoul2_info_array_free(g2: &mut Ghoul2System) {
    g2.info_array = Ghoul2InfoArray::default();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `New()` pulls from the front of the free list in slot order, so the
    /// first handles out of a fresh array are `MAX_G2_MODELS + 0`, `+1`, ...
    /// (ctor seeding, `G2_API.cpp:341-349`).
    #[test]
    fn new_handle_seeds_from_ctor_and_pops_front() {
        let mut arr = Ghoul2InfoArray::default();
        assert_eq!(arr.new_handle(), MAX_G2_MODELS as i32);
        assert_eq!(arr.new_handle(), MAX_G2_MODELS as i32 + 1);
    }

    /// `IsValid` (`:399-408`): null handle `0` is always invalid; a handle
    /// just issued by `New()` is valid; a handle whose slot has never been
    /// issued in that generation is not.
    #[test]
    fn is_valid_rejects_null_and_stale() {
        let mut arr = Ghoul2InfoArray::default();
        assert!(!arr.is_valid(0));
        let h = arr.new_handle();
        assert!(arr.is_valid(h));
        // Same slot, next generation (not yet issued) — must read invalid.
        assert!(!arr.is_valid(h + MAX_G2_MODELS as i32));
    }

    /// `clear_slot` (`DeleteLow`'s slot half, `:328-339`): bumps the
    /// generation by `+MAX_G2_MODELS` on the ordinary path and pushes the
    /// slot back onto the *front* of the free list, so the very next
    /// `New()` reissues the same slot with the bumped generation — and the
    /// old handle is now stale.
    #[test]
    fn clear_slot_bumps_generation_and_requeues_front() {
        let mut arr = Ghoul2InfoArray::default();
        let h = arr.new_handle();
        let idx = h & G2_INDEX_MASK;
        arr.clear_slot(idx);
        assert!(
            !arr.is_valid(h),
            "old handle must go stale after clear_slot"
        );
        let reissued = arr.new_handle();
        assert_eq!(reissued, h + MAX_G2_MODELS as i32);
    }

    /// `clear_slot`'s ordinary path at the generation ceiling (`:330-339`):
    /// the rollover check reads `mIds[idx]` *before* this cycle's increment,
    /// so at the highest still-positive generation
    /// (`1<<(31-G2_MODEL_BITS) - 1`) it reads `<=` the threshold and takes
    /// the ordinary (`push_front`) branch — whose `+= MAX_G2_MODELS` then
    /// silently wraps the id negative, one bit shy of the sign bit. This is
    /// bit-exact with the oracle's own signed-overflow arithmetic (both are
    /// 32-bit `int`); `wrapping_add` (not plain `+=`) reproduces that
    /// wraparound instead of an unrequested Rust debug-mode overflow panic.
    /// A consequence neither we nor the oracle avoid: the `>` check can
    /// never actually observe a *post*-wrap value exceeding the threshold
    /// (right-shifting a negative `i32` never reads `>` a positive
    /// threshold), so the reset (`push_back`) arm is structurally
    /// unreachable via any naturally-issued handle — a preexisting oracle
    /// quirk, not a port defect.
    #[test]
    fn clear_slot_wraps_generation_without_panicking() {
        let mut arr = Ghoul2InfoArray::default();
        let idx: usize = 5;
        let max_positive_gen = (1i32 << (31 - G2_MODEL_BITS)) - 1;
        arr.mIds[idx] = (max_positive_gen << G2_MODEL_BITS) | idx as i32;
        arr.clear_slot(idx as i32);
        assert_eq!(
            *arr.free_indices.front().unwrap(),
            idx as i32,
            "boundary generation still takes the ordinary push_front path"
        );
        assert!(arr.mIds[idx] < 0, "the increment silently wraps negative");
    }

    /// `Get`'s invalid-handle divergence (`:427-436`, `F19`): out-of-range or
    /// stale handles read as empty rather than the oracle's shared aliasing
    /// `static null` vector.
    #[test]
    fn get_and_get_mut_are_empty_on_invalid_handle() {
        let mut arr = Ghoul2InfoArray::default();
        assert!(arr.get(0).is_empty());
        assert!(arr.get(-1).is_empty());
        assert!(arr.get_mut(0).is_empty());
        let h = arr.new_handle();
        arr.clear_slot(h & G2_INDEX_MASK);
        assert!(arr.get(h).is_empty(), "stale handle must read empty");
    }

    /// `get_mut`/`get` round-trip through the same slot once a handle is
    /// live.
    #[test]
    fn get_mut_writes_are_visible_through_get() {
        let mut arr = Ghoul2InfoArray::default();
        let h = arr.new_handle();
        arr.get_mut(h).push(CGhoul2Info::default());
        assert_eq!(arr.get(h).len(), 1);
    }
}
