//! `WorldCell` — the shell's `UiWorld` static cell (STATE-D6, mirroring
//! `jampgame`'s `WorldCell`, `crates/jampgame/src/world_cell.rs`).

use std::cell::UnsafeCell;

use mp_ui::world::ui_world::UiWorld;

/// `UnsafeCell<Option<Box<UiWorld>>>` — the second sanctioned static
/// exemption (STATE-D6), holding the module island's one owned `UiWorld`
/// across `vmMain` calls. Reentrancy is handled by raw-pointer threading in
/// `vmMain` (each entry derives its own `*mut UiWorld`), not by this wrapper
/// — the `jampgame` `WorldCell` precedent exactly.
///
/// Source: `docs/architecture/state-ownership.md` § `WorldCell` (STATE-D6).
pub(crate) struct WorldCell(pub(crate) UnsafeCell<Option<Box<UiWorld>>>);

impl WorldCell {
    pub(crate) const fn new() -> Self {
        WorldCell(UnsafeCell::new(None))
    }
}

// SAFETY (Sync only): the module runs single-threaded per Raven's contract, so
// the static is never touched from a second thread. Single-threaded *reentrant*
// aliasing is handled by the raw-pointer threading in `vmMain` (STATE-D6).
unsafe impl Sync for WorldCell {}
