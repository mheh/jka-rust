//! `UiContext` — the ui module's threaded receiver (DEC-36 D4).

use mp_engine_select::Engine;

use super::ui_world::UiWorld;

/// The receiver every ui `vmMain` command routes through — the analog of
/// `mp_game`'s `GameContext` (DEC-36 D4).
///
/// The `vmMain` shell owns the one [`UiWorld`] and the engine transport and
/// builds a `UiContext` per call; the ported logic reads its world through
/// `ctx.world` and reaches the engine as `trap::X(ctx.engine, …)` in Raven's
/// syscall order. `Engine` is the `mp_engine_select` module-side transport
/// alias (SEAM-D13), not `mp_engine_core::Engine`.
///
/// `UiContext` is also the type that implements
/// [`DisplayContext`](mp_uishared::shared::display_context::DisplayContext):
/// its `menus`/`display` accessors hand out `ctx.world.menus` and
/// `ctx.world.uiDC`, and its behavior methods are the `ui_main.c` callbacks
/// Raven installed into `uiInfo.uiDC` (`UI_OwnerDraw`, `UI_FeederCount`,
/// `UI_RunMenuScript`, the `trap_*` forwarders).
///
/// Source: `docs/decisions.md` DEC-36 (ruling D4)
pub struct UiContext<'e> {
    /// The one owned [`UiWorld`] island, borrowed for the duration of the
    /// `vmMain` call — the borrow checker enforces §B4 directly.
    pub world: &'e mut UiWorld,
    pub engine: &'e Engine,
}
