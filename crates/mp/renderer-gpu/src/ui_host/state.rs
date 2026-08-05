//! `UiHost` holds everything one harness process owns.
//! It also holds the two small bookkeeping types the [`super::display::HarnessDc`] slots need.

use std::collections::BTreeMap;

use mp_engine_core::Engine;
use mp_renderer::renderer_frontend::RendererFrontend;
use mp_renderer::tr_model::render_models::RenderModels;
use mp_ui::world::ui_state::UiState;

/// The whole harness process state: the engine island booted to its FS/cvar/
/// cmd subset, the renderer's DEC-42.3 carrier bundle, and the ui module's
/// own `UiState`.
///
/// Every frame re-splits the host into disjoint `&mut` field borrows to build a [`super::display::HarnessDc`].
/// The split is two-level: `re`'s own fields borrow disjointly inside it, and `models` stays outside `re`, so one caller can hold both.
pub struct UiHost {
    // ---- engine island -------------------------------------------------
    /// Booted through the ordered FS/cvar/cmd prefix of `Com_Init` only —
    /// `sv`/`cl`/`bot` state exists but no server, client or network
    /// subsystem was started. `bot` is used solely for its precompiler
    /// (the menu-file tokenizer).
    pub engine: Box<Engine>,

    // ---- renderer carrier bundle (DEC-42.3) ----------------------------
    /// The model registry the bundle excludes, which the live engine holds at `Engine.render_models` behind the view's `rm` slot.
    pub models: RenderModels,
    /// The renderer's DEC-42.3 carrier bundle, the same struct the live client seats at `Engine.re`.
    pub re: RendererFrontend,
    // W2-F3: the sky scratch is render-thread-resident on `FrameExecutor`.

    // ---- the ui module -------------------------------------------------
    /// The real [`mp_ui::world::ui_state::UiState`]; the harness drives its
    /// `menus`/`uiDC` halves through `mp_uishared` (see the module doc for
    /// why `world` stays idle).
    pub ui: UiState,

    // ---- harness bookkeeping -------------------------------------------
    pub input: InputState,
    pub stubs: StubLog,
    /// Wall-clock milliseconds at boot, so `Milliseconds()`/`realTime` start
    /// near zero exactly as a fresh engine's `Sys_Milliseconds` does.
    pub start: std::time::Instant,
}

/// The key/cursor state Raven's engine owned on the ui module's behalf
/// (`Key_IsDown`/`Key_GetCatcher`/`Key_SetCatcher`, the overstrike flag).
#[derive(Default)]
pub struct InputState {
    /// Sparse `keys[]` — only keys the harness has seen are present.
    pub down: BTreeMap<i32, bool>,
    pub catcher: i32,
    pub overstrike: bool,
}

/// One counter per stubbed [`mp_uishared::shared::display_context::DisplayContext`]
/// slot, so a run's log can state exactly what the menu asked for and did not
/// get — honest degradation, per the wave plan, instead of a panic.
#[derive(Default)]
pub struct StubLog {
    pub counts: BTreeMap<&'static str, u32>,
}

impl StubLog {
    /// Records one call to a stubbed slot, printing the first hit so a live
    /// run shows *when* the menu first reached for the missing service.
    pub fn hit(&mut self, subject: &'static str) {
        let n = self.counts.entry(subject).or_insert(0);
        *n += 1;
        if *n == 1 {
            println!("ui_harness: stub reached — DisplayContext::{subject}");
        }
    }

    /// `subject: count` for every stub reached, in name order.
    pub fn report(&self) -> String {
        self.counts
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(" ")
    }
}
