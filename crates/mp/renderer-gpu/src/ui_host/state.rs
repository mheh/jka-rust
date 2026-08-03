//! `UiHost` — everything one harness process owns, plus the two small
//! bookkeeping types the [`super::display::HarnessDc`] slots need.

use std::collections::BTreeMap;

use mp_engine_core::Engine;
use mp_qshared::shared::com_parse::QSharedScratch;
use mp_renderer::render_state::frame_state::FrameState;
use mp_renderer::render_state::render_assets_sim::RenderAssetsSim;
use mp_renderer::render_state::renderer_cvars::RendererCvars;
use mp_renderer::render_state::world_load_state::WorldLoadState;
use mp_renderer::tr_font::FontState;
use mp_renderer::tr_image::TrImageState;
use mp_renderer::tr_local::view_parms_t::viewParms_t;
use mp_renderer::tr_model::render_models::RenderModels;
use mp_renderer::tr_noise::NoiseState;
use mp_renderer::tr_scene::SceneState;
use mp_renderer::tr_sky::SkyState;
use mp_renderer::tr_worldeffects::world_effects::WorldEffectsState;
use mp_ui::world::ui_state::UiState;
use native_math::rng::Rng;

/// The whole harness process state: the engine island booted to its FS/cvar/
/// cmd subset, the renderer's DEC-42.3 carrier bundle, and the ui module's
/// own `UiState`.
///
/// Deliberately one flat owner rather than nested structs: every frame
/// re-splits it into disjoint `&mut` field borrows to build a
/// [`super::display::HarnessDc`], and a flat struct is what makes that split
/// borrow-checkable.
pub struct UiHost {
    // ---- engine island -------------------------------------------------
    /// Booted through the ordered FS/cvar/cmd prefix of `Com_Init` only —
    /// `sv`/`cl`/`bot` state exists but no server, client or network
    /// subsystem was started. `bot` is used solely for its precompiler
    /// (the menu-file tokenizer).
    pub engine: Box<Engine>,

    // ---- renderer carrier bundle (DEC-42.3) ----------------------------
    pub models: RenderModels,
    pub cvars: RendererCvars,
    /// The one CPU registry. Every registration writes `sim.published`, so a
    /// harness draw reads the generation the registration produced.
    pub sim: RenderAssetsSim,
    pub img_state: TrImageState,
    pub frame: FrameState,
    /// The `tr` fields the sim writes at load and the render side only reads
    /// (W2-F3), the harness twin of `RendererFrontend::world_load`.
    pub world_load: WorldLoadState,
    pub scene: SceneState,
    pub noise: NoiseState,
    pub rng: Rng,
    pub font: FontState,
    pub world_effects: WorldEffectsState,
    pub qs: QSharedScratch,
    pub sky_view: viewParms_t,
    pub sky: SkyState,

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
