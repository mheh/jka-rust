//! `CPriorityQueue` (navigator.h:254-276) — the min-heap-on-cost priority
//! queue `CalculatePath`/`CalculatePaths` (navigator.rs) drive their frontier
//! flood-fill with.
//!
//! Raven's `mHeap` is `vector<CEdge*>` of `new`-allocated, `Pop`-caller-`delete`d
//! pointers; `CPriorityQueue`'s dtor drains and deletes whatever remains. Per
//! porting-rules §F17/D-7 (idiomatic ownership, faithful order) this becomes an
//! owning `Vec<Edge>` of values — construction/destruction fall out of Rust's
//! own `Vec` lifecycle (see the notes on the elided ctor/dtor below), while
//! `Push`/`Pop`'s libstdc++ `push_heap`/`pop_heap` sift under the
//! `NodeTotalGreater` comparator (`first->m_cost > second->m_cost`,
//! navigator.cpp:2693-2699 — a **min-heap on cost**) is hand-transcribed
//! faithfully so equal-cost tie order matches the oracle-harness libstdc++
//! (NAV-D2 / RULING 45 states the two-phase Floyd sift behaviorally; the
//! committed rank goldens are the enforcing gate — NOT `std::BinaryHeap`,
//! which would diverge tie order).
//!
//! **Elided members (not stubbed):**
//! - `CPriorityQueue()` (navigator.h:259, inline `{}` — a genuine no-op, no
//!   Raven logic to defer) ports as [`Default`] directly: an empty `Vec`.
//! - `~CPriorityQueue()` (navigator.cpp:2705-2711, loops `delete Pop()` over
//!   whatever remains) has no Rust counterpart to write: owning `Vec<Edge>`
//!   values (D-7) means the compiler-derived `Drop` already frees every
//!   remaining element when a `PriorityQueue` goes out of scope — there is no
//!   manual free logic left to port.
//! - `Find`/`Update` (navigator.cpp:2716-2726,2763-2774) linear-scan `mHeap`
//!   by `m_first`; **dropped as zero-caller dead surface** (porting-rules §20)
//!   — `CalculatePath` never calls them (only `Push`/`Pop`/`Empty` are
//!   exercised, navigator.cpp:814-877), matching the doc's D-7 note.
//!
//! Type definition source: `oracle/codemp/server/NPCNav/navigator.h:254-276`
//! (methods: `oracle/codemp/server/NPCNav/navigator.cpp:2705-2782`)

use super::edge::Edge;

/// Raven `CPriorityQueue`.
///
/// Raven: "class Priority Queue" (navigator.h:252 banner comment).
/// Type definition source: `oracle/codemp/server/NPCNav/navigator.h:254-276`
pub struct PriorityQueue {
    /// Raven `vector<CEdge*> mHeap` (navigator.h:275) — owned `Vec<Edge>` of
    /// values, not raw `CEdge*` (D-7); heap-ordered by
    /// [`node_total_greater`] via `Push`/`Pop`.
    pub heap: Vec<Edge>,
}

impl Default for PriorityQueue {
    /// Raven `CPriorityQueue() {};` — a genuine no-op inline ctor; the empty
    /// `Vec` is the direct analogue.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.h:259`
    fn default() -> Self {
        PriorityQueue { heap: Vec::new() }
    }
}

/// Raven `NodeTotalGreater::operator()(CEdge *first, CEdge *second)` — the
/// `push_heap`/`pop_heap` comparator: `first->m_cost > second->m_cost`, a
/// **min-heap on cost** (the largest-cost element compares "greater" so it
/// sifts toward the leaves, leaving the smallest cost at the root/front).
///
/// Colocated here (porting-rules §21) rather than filed separately: it has no
/// state of its own and exists solely to drive this queue's sift, exactly as
/// Raven's `NodeTotalGreater` is a helper class declared immediately above
/// `CPriorityQueue`'s method bodies for that reason.
///
/// Source: `oracle/codemp/server/NPCNav/navigator.cpp:2693-2699`
fn node_total_greater(first: &Edge, second: &Edge) -> bool {
    first.cost > second.cost
}

impl PriorityQueue {
    /// Raven `CEdge* CPriorityQueue::Pop()` — reads `mHeap.front()`, sifts
    /// with `std::pop_heap`/`NodeTotalGreater`, `pop_back()`s the vacated
    /// slot, and returns the popped (min-cost) edge. The libstdc++
    /// `pop_heap`/`__adjust_heap` two-phase Floyd sift is spelled out
    /// behaviorally in NAV-D2 (RULING 45), Verification strategy → "The
    /// libstdc++ heap sift", and hand-transcribed here: read the root as the
    /// result, take the last element out as `saved`, sift the vacated hole
    /// down to a leaf comparing only the two children at each level under
    /// [`node_total_greater`]'s child-comparison order (never against
    /// `saved`), then sift `saved` up from that leaf exactly as `Push` would.
    /// `len` and the hole/child indices are signed to mirror the algorithm's
    /// own truncating division on negative intermediates (`(len - 1) / 2` at
    /// `len == 0`), which the doc's pseudocode relies on to skip phase 1 for
    /// a one-element remainder.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:2731-2746`
    pub fn pop(&mut self) -> Edge {
        let result = self.heap[0];
        let len = self.heap.len() as isize - 1;
        let saved = self.heap[len as usize];

        // Phase 1 — sift the hole down to a leaf under NodeTotalGreater's
        // child-comparison order (visit the right child, step to the left
        // only when the right child is greater-under-comp, i.e. costlier).
        let mut hole: isize = 0;
        let mut second_child: isize = 0;
        while second_child < (len - 1) / 2 {
            second_child = 2 * (second_child + 1);
            if node_total_greater(
                &self.heap[second_child as usize],
                &self.heap[(second_child - 1) as usize],
            ) {
                second_child -= 1;
            }
            self.heap[hole as usize] = self.heap[second_child as usize];
            hole = second_child;
        }
        // Single-child fix-up for the even-length last-parent case.
        if len % 2 == 0 && second_child == (len - 2) / 2 {
            second_child = 2 * (second_child + 1);
            self.heap[hole as usize] = self.heap[(second_child - 1) as usize];
            hole = second_child - 1;
        }

        // Phase 2 — sift `saved` up from the leaf, same shape as `Push`.
        while hole > 0 {
            let parent = (hole - 1) / 2;
            if node_total_greater(&self.heap[parent as usize], &saved) {
                self.heap[hole as usize] = self.heap[parent as usize];
                hole = parent;
            } else {
                break;
            }
        }
        self.heap[hole as usize] = saved;

        self.heap.pop();
        result
    }

    /// Raven `void CPriorityQueue::Push(CEdge* theEdge)` — `mHeap.push_back`
    /// then `std::push_heap`/`NodeTotalGreater` sifts the new element up.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:2751-2758`
    ///
    /// libstdc++ `__push_heap` sift-up (NAV-D2, RULING 45): append, then
    /// while the hole is above the root and the parent is greater-under-comp
    /// than the value at the hole (equivalently: greater than the pushed
    /// value, since the hole always holds it), swap parent and hole up.
    pub fn push(&mut self, edge: Edge) {
        self.heap.push(edge);
        let mut hole = self.heap.len() - 1;
        while hole > 0 {
            let parent = (hole - 1) / 2;
            if node_total_greater(&self.heap[parent], &self.heap[hole]) {
                self.heap.swap(parent, hole);
                hole = parent;
            } else {
                break;
            }
        }
    }

    /// Raven `bool CPriorityQueue::Empty()` — "Just a wrapper for stl empty
    /// function": `mHeap.empty()`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:2779-2782`
    pub fn empty(&self) -> bool {
        self.heap.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Empty()` on a fresh queue and after draining every pushed edge.
    #[test]
    fn empty_tracks_heap_contents() {
        let mut pq = PriorityQueue::default();
        assert!(pq.empty());
        pq.push(Edge::new(0, 1, 5));
        assert!(!pq.empty());
        pq.pop();
        assert!(pq.empty());
    }

    /// Distinct costs must always pop in ascending-cost (min-heap) order,
    /// regardless of push order.
    #[test]
    fn pops_in_ascending_cost_order() {
        let mut pq = PriorityQueue::default();
        for (first, cost) in [(0, 30), (1, 10), (2, 50), (3, 20), (4, 40)] {
            pq.push(Edge::new(first, 0, cost));
        }
        let mut popped = Vec::new();
        while !pq.empty() {
            popped.push(pq.pop().cost);
        }
        assert_eq!(popped, vec![10, 20, 30, 40, 50]);
    }

    /// Equal-cost entries must pop in the exact order the two-phase Floyd
    /// sift (NAV-D2 / RULING 45) produces, not merely "some" stable order —
    /// this pins the child-comparison direction (right child visited first,
    /// step to left only when the right child is greater-under-comp) against
    /// a regression to e.g. `std::BinaryHeap`'s differing tie order.
    #[test]
    fn equal_cost_tie_order_matches_floyd_sift() {
        let mut pq = PriorityQueue::default();
        for id in 0..7 {
            pq.push(Edge::new(id, 0, 10));
        }
        let mut popped = Vec::new();
        while !pq.empty() {
            popped.push(pq.pop().first);
        }
        // Ground truth: the same 7 equal-cost push/pop sequence run through
        // real `std::push_heap`/`std::pop_heap` under an equivalent
        // `NodeTotalGreater`-shaped comparator on the doc's pinned toolchain
        // (`/opt/homebrew/Cellar/gcc/16.1.0/.../bits/stl_heap.h`, g++-16)
        // pops `0 2 5 6 4 1 3` — libstdc++'s tie order is implementation-
        // specific (Apple libc++'s `std::push_heap`/`pop_heap` on the same
        // input instead pops `0 1 3 6 5 4 2`), which is exactly why this test
        // pins the libstdc++ sequence rather than "any" stable order.
        assert_eq!(popped, vec![0, 2, 5, 6, 4, 1, 3]);
    }

    /// `Push` never sifts an element past an equal-cost parent (`comp` is a
    /// strict `>`, so ties never trigger the swap) — this is the property
    /// the tie-order test above depends on.
    #[test]
    fn push_leaves_equal_cost_insertion_order_untouched() {
        let mut pq = PriorityQueue::default();
        for id in 0..7 {
            pq.push(Edge::new(id, 0, 10));
        }
        let ids: Vec<i32> = pq.heap.iter().map(|e| e.first).collect();
        assert_eq!(ids, vec![0, 1, 2, 3, 4, 5, 6]);
    }
}
