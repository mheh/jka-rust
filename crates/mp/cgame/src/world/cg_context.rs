//! `CgContext` — the cgame module's threaded receiver, `mp_ui`'s `UiContext`
//! shape (DEC-46.1, DEC-36 D4).

use mp_engine_select::Engine;

use super::cg_world::CgWorld;

/// The receiver every cgame `vmMain` command routes through — the analog of
/// `mp_ui`'s `UiContext` and `mp_game`'s `GameContext`.
///
/// The `vmMain` shell owns the one [`CgState`](super::cg_state::CgState),
/// splits it into three disjoint borrows and builds a `CgContext` per call; the
/// ported logic reads its world through `ctx.world` and reaches the engine as
/// `trap::X(ctx.engine, …)` in Raven's syscall order. `Engine` is the
/// `mp_engine_select` module-side transport alias (SEAM-D13), not
/// `mp_engine_core::Engine`.
///
/// `menus` and `cgDC` thread as separate params beside `ctx`, so a framework
/// call like `Item_Paint(menus, ds, ctx)` hands out three disjoint borrows and
/// the re-entrant callbacks take the caller's `menus`/`ds` straight back —
/// mutations visible on return, zero aliasing, zero `unsafe` (DEC-38 ruling 1).
/// The cgame `DisplayContext` implementor lands with the C5 waves, on ui's
/// `ui_display_context` pattern.
///
/// Source: `docs/decisions.md` DEC-46 (ruling 1), DEC-36 (ruling D4)
pub struct CgContext<'e> {
    /// The one owned [`CgWorld`] island, borrowed for the duration of the
    /// `vmMain` call — the borrow checker enforces §B4 directly.
    pub world: &'e mut CgWorld,
    pub engine: &'e Engine,
}

impl CgContext<'_> {
    /// Raw world pointer for the pmove seam structs (`CgBgTraps` /
    /// `CgGameCallbacks`) - `mp_game`'s `GameContext::world_raw` shape
    /// (DEC-47.2). The seam methods reborrow it for the duration of one call;
    /// no borrow outlives the call.
    #[inline]
    pub fn world_raw(&mut self) -> *mut CgWorld {
        &raw mut *self.world
    }
}
