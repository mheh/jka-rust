//! `RendererFrontend` — the DEC-42.3 carrier bundle the `RE_*` frontend takes,
//! owned by `Engine.re`.
//!
//! DEC-59.1 removed `refexport_t`, `GetRefAPI`, and `REF_API_VERSION`: an
//! engine-interior renderer call names the `RE_*` function and hands it the
//! receivers its own signature declares. Those receivers were scattered locals
//! before, seated once per process by the R5 harness (`UiHost`). This struct is
//! the one home for them in the live engine, so a client function reaches the
//! whole set through a single slot cast.
//!
//! Field names match the receiver names the `RE_*` signatures use, so a call
//! site reads `RE_SetColor(&mut re.frame_data, color)` with no renaming.
//!
//! State-partition law (DEC-55.2): a synchronous path reads `assets` (the CPU
//! registry) and appends to `frame_data`. `gpu_res` belongs to the render
//! thread, and no trap arm touches it.
//!
//! Source: `docs/decisions.md` DEC-42.3, DEC-55.2, DEC-59.1;
//! `crates/mp/renderer-gpu/src/ui_host/state.rs` (the harness's seated twin).

use mp_qshared::shared::com_parse::QSharedScratch;
use native_math::rng::Rng;

use crate::render_state::frame_data::FrameData;
use crate::render_state::frame_state::FrameState;
use crate::render_state::gpu_resources::GpuResources;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::render_assets_sim::RenderAssetsSim;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::tr_font::FontState;
use crate::tr_image::TrImageState;
use crate::tr_local::view_parms_t::viewParms_t;
use crate::tr_noise::NoiseState;
use crate::tr_scene::SceneState;
use crate::tr_sky::SkyState;
use crate::tr_worldeffects::world_effects::WorldEffectsState;

/// Every `RE_*` receiver except the model registry, which stays the one
/// `Engine.render_models` the server and the client share (`view.rm`).
///
/// `Engine.re` holds this as an `Option`, `Some` on a client build and `None`
/// on dedicated — the same shape `Engine.cl` and `Engine.snd` already use. The
/// seating constructor lands with the platform shell (the winit boot, DEC-56),
/// which is the first code that has a device to seat `gpu_res` against.
pub struct RendererFrontend {
    /// The registered `r_*` cvar handles.
    pub cvars: RendererCvars,
    /// The CPU-side registry root: images, shaders, skins, the world asset.
    /// A synchronous trap arm reads this and nothing else on the asset side.
    pub assets: RenderAssets,
    /// The mutation half of the registry, which publishes new `RenderAssets`
    /// generations through `Arc::make_mut`.
    pub sim: RenderAssetsSim,
    /// `tr_image.cpp`'s file-scope state (the scratch buffers and the load
    /// counters).
    pub img_state: TrImageState,
    /// The render thread's GPU objects. The state-partition law puts this off
    /// limits to every synchronous path.
    pub gpu_res: GpuResources,
    /// The frontend's per-frame scratch — the oracle's `tr` fields that are
    /// neither registry nor GPU.
    pub frame: FrameState,
    /// The ordered event stream this frame appends to, which replaces the
    /// oracle's `backEndData_t` command list.
    pub frame_data: FrameData,
    /// `tr_scene.cpp`'s per-scene accumulation state.
    pub scene: SceneState,
    /// The Perlin noise tables `R_NoiseInit` fills once.
    pub noise: NoiseState,
    /// The renderer's own LCG (ruling 21 keeps it off the engine island's).
    pub rng: Rng,
    /// The registered font set `RE_RegisterFont` fills.
    pub font: FontState,
    /// `tr_worldeffects`'s weather and wind state.
    pub world_effects: WorldEffectsState,
    /// The shared `COM_Parse` scratch the shader and font parsers thread.
    pub qs: QSharedScratch,
    /// The sky-portal view parms `RE_RegisterShader`'s sky path writes.
    pub sky_view: viewParms_t,
    /// `tr_sky.cpp`'s cloud and sky-box state.
    pub sky: SkyState,
}
