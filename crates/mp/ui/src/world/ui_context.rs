//! `UiContext` — the ui module's threaded receiver (DEC-36 D4), and the one
//! `DisplayContext` implementor (DEC-38 ruling 1, revised).

use mp_engine_select::Engine;

use super::ui_world::UiWorld;

/// The receiver every ui `vmMain` command routes through — the analog of
/// `mp_game`'s `GameContext` (DEC-36 D4).
///
/// The `vmMain` shell owns the one [`UiState`](super::ui_state::UiState),
/// splits it into three disjoint borrows and builds a `UiContext` per call;
/// the ported logic reads its world through `ctx.world` and reaches the engine
/// as `trap::X(ctx.engine, …)` in Raven's syscall order. `Engine` is the
/// `mp_engine_select` module-side transport alias (SEAM-D13), not
/// `mp_engine_core::Engine`.
///
/// `UiContext` IS the module's
/// [`DisplayContext`](mp_uishared::shared::display_context::DisplayContext)
/// implementor (DEC-38 ruling 1, revised; the impl lives in
/// `crate::ui_display_context`). DEC-36 addendum 12 forbade that while the
/// context owned `menus`/`uiDC` — a framework fn holding `&mut MenuSystem`
/// could not also hold the `dc` that owned it. Hoisting both out of `UiWorld`
/// into `UiState` dissolves the objection: `menus` and `uiDC` thread as
/// separate params beside `ctx`, so `Item_ListBox_Paint(menus, ds, ctx)` hands
/// the framework three disjoint borrows, and the re-entrant callbacks
/// (`ownerDrawItem`, `runScript`, the feeders) take the caller's `menus`/`ds`
/// straight back — mutations visible on return, zero aliasing, zero `unsafe`.
///
/// Source: `docs/decisions.md` DEC-36 (ruling D4), DEC-38 (ruling 1, revised)
pub struct UiContext<'e> {
    /// The one owned [`UiWorld`] island, borrowed for the duration of the
    /// `vmMain` call — the borrow checker enforces §B4 directly.
    pub world: &'e mut UiWorld,
    pub engine: &'e Engine,
}
