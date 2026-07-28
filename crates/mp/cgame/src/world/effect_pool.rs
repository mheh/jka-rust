//! `EffectPool` — the gen-counted slab behind cgame's two fixed effect pools
//! (DEC-46.3).

use std::collections::VecDeque;

use super::effect_handle::EffectHandle;

/// A fixed-capacity slab of effect records plus an explicit age-ordered active
/// queue — the replacement for Raven's `cg_localEntities` / `cg_markPolys`
/// arrays and their intrusive `prev`/`next` chains (DEC-46.3).
///
/// Raven kept three things in the links: which slots are free (a singly linked
/// free list), which are live (a doubly linked circular active list), and the
/// order they were allocated in (position in that list, newest at the head).
/// Here the free list is a `Vec` of indices, liveness is a flag beside the
/// slot, and the order is `active`, held oldest-at-front. `alloc` never fails:
/// at capacity it frees the oldest active entry first, which is exactly
/// `CG_AllocLocalEntity`'s steal (`oracle/codemp/cgame/cg_localents.c:59-79`).
///
/// The pool deliberately does NOT know about `cg.time`. `CG_AllocMark` frees
/// *every* mark sharing the oldest one's `time`, not just one
/// (`oracle/codemp/cgame/cg_marks.c:68-90`) — that stays in the mark code,
/// which walks [`Self::active_oldest_first`] and calls [`Self::free`] itself.
///
/// `free` is a linear scan of `active` to find the entry. The walks that free
/// (expiry sweeps) run oldest-first and hit position 0, and capacity is 512, so
/// it never matters.
pub struct EffectPool<T> {
    slots: Box<[T]>,
    /// Bumped every time a slot is freed, so handles into it stop resolving.
    generations: Box<[u32]>,
    live: Box<[bool]>,
    /// Free slot indices; `pop` hands back the most recently freed, which is
    /// Raven's LIFO free list (`le->next = cg_freeLocalEntities`).
    free: Vec<u32>,
    /// Live handles in allocation order — front is the oldest.
    active: VecDeque<EffectHandle>,
    /// Builds a fresh zeroed record. Raven `memset( le, 0, sizeof( *le ) )` on
    /// every alloc.
    seed: fn() -> T,
}

impl<T> EffectPool<T> {
    /// Builds the pool at its Raven capacity with every slot free, matching
    /// `CG_InitLocalEntities` / `CG_InitMarkPolys`' post-`memset` state.
    ///
    /// Source: `oracle/codemp/cgame/cg_localents.c:21-31`,
    /// `oracle/codemp/cgame/cg_marks.c:28-39`
    pub fn new(capacity: usize, seed: fn() -> T) -> Self {
        let slots = (0..capacity).map(|_| seed()).collect::<Vec<T>>();
        EffectPool {
            slots: slots.into_boxed_slice(),
            generations: vec![0u32; capacity].into_boxed_slice(),
            live: vec![false; capacity].into_boxed_slice(),
            // reversed so the first `pop` hands out slot 0, Raven's initial
            // free-list order
            free: (0..capacity as u32).rev().collect(),
            active: VecDeque::with_capacity(capacity),
            seed,
        }
    }

    /// Back to the freshly-`memset` state — every slot free, nothing active.
    /// Called at startup and on tournament restart, same as Raven's `CG_Init*`.
    pub fn clear(&mut self) {
        let capacity = self.slots.len();
        for i in 0..capacity {
            self.slots[i] = (self.seed)();
            self.live[i] = false;
            // outstanding handles must not resolve across a restart
            self.generations[i] = self.generations[i].wrapping_add(1);
        }
        self.free.clear();
        self.free.extend((0..capacity as u32).rev());
        self.active.clear();
    }

    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    /// How many entries are live right now.
    pub fn len(&self) -> usize {
        self.active.len()
    }

    pub fn is_empty(&self) -> bool {
        self.active.is_empty()
    }

    /// The oldest live entry, the one [`Self::alloc`] steals at capacity.
    pub fn oldest(&self) -> Option<EffectHandle> {
        self.active.front().copied()
    }

    /// Always succeeds, even if it requires freeing an old active entry.
    ///
    /// Source: `oracle/codemp/cgame/cg_localents.c:53-79` (`CG_AllocLocalEntity`)
    pub fn alloc(&mut self) -> EffectHandle {
        if self.free.is_empty() {
            // no free entries, so free the one at the end of the chain
            if let Some(oldest) = self.active.front().copied() {
                self.free(oldest);
            }
        }
        // capacity is never 0, so the steal above always yields a slot
        let index = self
            .free
            .pop()
            .expect("EffectPool::alloc on a 0-capacity pool");
        let i = index as usize;
        self.slots[i] = (self.seed)();
        self.live[i] = true;
        let handle = EffectHandle {
            index,
            generation: self.generations[i],
        };
        self.active.push_back(handle);
        handle
    }

    /// Drops `handle` out of the active queue. Returns false if it was already
    /// gone — Raven errored out on that (`CG_FreeLocalEntity: not active`), so
    /// a false here is a caller bug, not a normal path.
    ///
    /// Source: `oracle/codemp/cgame/cg_localents.c:38-50` (`CG_FreeLocalEntity`)
    pub fn free(&mut self, handle: EffectHandle) -> bool {
        if !self.resolves(handle) {
            return false;
        }
        let i = handle.index();
        self.live[i] = false;
        self.generations[i] = self.generations[i].wrapping_add(1);
        if let Some(pos) = self.active.iter().position(|&h| h == handle) {
            self.active.remove(pos);
        }
        self.free.push(handle.index);
        true
    }

    pub fn get(&self, handle: EffectHandle) -> Option<&T> {
        if self.resolves(handle) {
            Some(&self.slots[handle.index()])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, handle: EffectHandle) -> Option<&mut T> {
        if self.resolves(handle) {
            Some(&mut self.slots[handle.index()])
        } else {
            None
        }
    }

    /// Live handles oldest first — `CG_AddLocalEntities` walks this way, so any
    /// effects it spawns are present the same frame
    /// (`oracle/codemp/cgame/cg_localents.c:789-800`).
    ///
    /// Freeing while walking needs the handles copied out first
    /// (`.collect::<Vec<_>>()`), which is Raven grabbing `next` before the free.
    pub fn active_oldest_first(&self) -> impl DoubleEndedIterator<Item = EffectHandle> + '_ {
        self.active.iter().copied()
    }

    /// Live handles newest first — `CG_AddMarks` walks this way
    /// (`oracle/codemp/cgame/cg_marks.c:221-240`).
    pub fn active_newest_first(&self) -> impl DoubleEndedIterator<Item = EffectHandle> + '_ {
        self.active.iter().rev().copied()
    }

    /// Live, and the slot has not been recycled since the handle was issued.
    fn resolves(&self, handle: EffectHandle) -> bool {
        let i = handle.index();
        i < self.slots.len() && self.live[i] && self.generations[i] == handle.generation
    }
}
