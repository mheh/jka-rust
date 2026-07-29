//! `WorldCell` — the shell's `CgState` static cell (STATE-D6, mirroring
//! `jampgame`'s `WorldCell`, `crates/jampgame/src/world_cell.rs`).

use std::cell::UnsafeCell;

use mp_cgame::world::cg_state::CgState;

/// `UnsafeCell<Option<Box<CgState>>>` — the second sanctioned static
/// exemption (STATE-D6), holding the module island's one owned `CgState`
/// (the `CgWorld` plus the hoisted `menus`/`cgDC`, DEC-46 ruling 1) across
/// `vmMain` calls. Reentrancy is handled by raw-pointer threading in `vmMain`
/// (each entry derives its own `*mut CgState`), not by this wrapper — the
/// `jampgame` `WorldCell` precedent exactly.
///
/// Source: `docs/architecture/state-ownership.md` § `WorldCell` (STATE-D6).
pub(crate) struct WorldCell(pub(crate) UnsafeCell<Option<Box<CgState>>>);

impl WorldCell {
    pub(crate) const fn new() -> Self {
        WorldCell(UnsafeCell::new(None))
    }
}

// SAFETY (Sync only): the module runs single-threaded per Raven's contract, so
// the static is never touched from a second thread. Single-threaded *reentrant*
// aliasing is handled by the raw-pointer threading in `vmMain` (STATE-D6).
unsafe impl Sync for WorldCell {}
