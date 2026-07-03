//! `WorldCell` — the shell's `GameWorld` static cell (STATE-D6; per-file
//! placement + `pub(crate)` field pinned by the round-7 resolutions, the
//! LOAD-D12f precedent).

use std::cell::UnsafeCell;

use mp_game::GameWorld;

/// `UnsafeCell<Option<GameWorld>>` — the second sanctioned static exemption
/// (STATE-D6), holding the module island's one owned `GameWorld` across
/// `vmMain` calls. Reentrancy is handled by raw-pointer threading in `vmMain`
/// (each entry derives its own `*mut GameWorld`), not by this wrapper.
///
/// Source: `docs/architecture/state-ownership.md` § `WorldCell` (STATE-D6).
pub(crate) struct WorldCell(pub(crate) UnsafeCell<Option<GameWorld>>);

impl WorldCell {
    pub(crate) const fn new() -> Self {
        WorldCell(UnsafeCell::new(None))
    }
}

// SAFETY (Sync only): the module runs single-threaded per Raven's contract, so
// the static is never touched from a second thread. Single-threaded *reentrant*
// aliasing is handled by the raw-pointer threading in `vmMain` (STATE-D6).
unsafe impl Sync for WorldCell {}
