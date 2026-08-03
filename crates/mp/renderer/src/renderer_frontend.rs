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
//! State-partition law (DEC-55.2): a synchronous path reads `sim.published`
//! (the one CPU registry) and appends to `frame_data`. This bundle carries no
//! GPU state. DEC-63.4 deleted the empty `GpuResources` carrier, and
//! `mp_renderer_gpu` owns every real GPU object on the render thread.
//!
//! One registry, not two (user ruling 2026-08-02). A second direct
//! `RenderAssets` instance used to take the shader and skin registrations while
//! image registration wrote the published generation, so a draw could read a
//! registry the registration never reached.
//!
//! Source: `docs/decisions.md` DEC-42.3, DEC-55.2, DEC-59.1;
//! `crates/mp/renderer-gpu/src/ui_host/state.rs` (the harness's seated twin).

use core::ffi::c_int;
use std::sync::Arc;

use mp_engine_qcommon::qfiles::light_style_limits::MAX_LIGHT_STYLES;
use mp_qshared::shared::com_parse::QSharedScratch;
use native_math::rng::Rng;

use crate::render_state::arena::Arena;
use crate::render_state::capture_request::CaptureRequest;
use crate::render_state::frame_data::FrameData;
use crate::render_state::frame_sink::FrameSink;
use crate::render_state::frame_state::FrameState;
use crate::render_state::light_style_table::LightStyleTable;
use crate::render_state::placeholders::{
    AutomapWireframe, BackEndCounters, FunctionTables, GlConfig, OrientationR, RefEntity, TrRefdef,
    ViewParms,
};
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::render_assets_sim::RenderAssetsSim;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::render_state::shader_asset::{ShaderAsset, ShaderHandle};
use crate::render_state::world_load_state::WorldLoadState;
use crate::render_state::skin_asset::SkinAsset;
use crate::render_state::sky_parse::SkyParse;
use crate::render_state::world_generation::WorldGeneration;
use crate::tr_font::FontState;
use crate::tr_image::TrImageState;
use crate::tr_local::view_parms_t::viewParms_t;
use crate::tr_noise::NoiseState;
use crate::tr_scene::SceneState;
use crate::tr_sky::SkyState;
use crate::tr_world::WireframeAutomap;
use crate::tr_worldeffects::world_effects::WorldEffectsState;

/// Every `RE_*` receiver except the model registry, which stays the one
/// `Engine.render_models` the server and the client share (`view.rm`).
///
/// `Engine.re` holds this as an `Option`, `Some` on a client build and `None`
/// on dedicated — the same shape `Engine.cl` and `Engine.snd` already use. The
/// seating constructor lands with the platform shell (the winit boot, DEC-56).
pub struct RendererFrontend {
    /// The registered `r_*` cvar handles.
    pub cvars: RendererCvars,
    /// The one CPU-side registry: images, shaders, skins, the world asset.
    /// Every registration writes `sim.published` through `Arc::make_mut`, and
    /// draw time reads the generation that publish produced.
    pub sim: RenderAssetsSim,
    /// `tr_image.cpp`'s file-scope state (the scratch buffers and the load
    /// counters).
    pub img_state: TrImageState,
    /// The frontend's per-frame scratch — the oracle's `tr` fields that are
    /// neither registry nor GPU.
    pub frame: FrameState,
    /// The `tr` fields the sim writes at load and the render side only reads
    /// (W2-F3). A copy rides on every `FramePackage`.
    pub world_load: WorldLoadState,
    /// The ordered event stream this frame appends to, which replaces the
    /// oracle's `backEndData_t` command list.
    pub frame_data: FrameData,
    /// The render thread's end of the frame channel, `Some` only on a client
    /// build that started one. `RE_EndFrame` sends a `FramePackage` when this
    /// is installed, and clears the stream in place when it is not.
    pub frame_sink: Option<FrameSink>,
    /// A `screenshot_tga` waiting for the next frame to carry it.
    pub pending_capture: Option<CaptureRequest>,
    /// A world change waiting for the next frame to carry it (W2-F7). Set on a
    /// map load, a map drop, and a video restart. `RE_EndFrame` moves it onto
    /// the package, so the render thread sees each change exactly once.
    pub pending_world: Option<WorldGeneration>,
    /// Raven's `R_ScreenShotTGA_f`-local `static int lastNumber`, which starts
    /// at `-1` and holds the scan position across calls so a burst of shots
    /// does not rescan thousands of names. Genuine cross-call state, so it
    /// sits on the bundle (three-kind rule, kind 3).
    ///
    /// Source: `oracle/codemp/renderer/tr_init.cpp:708`
    pub screenshot_last_number: i32,
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
    // W2-F3 split `tr_sky.cpp`'s file-scope state. The parse-time cloud tables
    // ride `RenderAssets::sky_parse`, and the per-view scratch is
    // render-thread-resident on `FrameExecutor`, so this bundle carries
    // neither.
    /// `tr_world.cpp`'s wireframe-automap generator state.
    pub automap: WireframeAutomap,
}

/// `MAX_SHADERS` (non-`_XBOX`) - the shader arena's soft cap.
///
/// Source: `oracle/codemp/renderer/tr_local.h` (`MAX_SHADERS`)
const MAX_SHADERS: u32 = 16384;

/// `MAX_SKINS` - the skin arena's soft cap.
///
/// Source: `oracle/codemp/renderer/tr_local.h:1204`
const MAX_SKINS: u32 = 1024;

/// The `max_texture_size` a boot with no `qglGetIntegerv` starts from.
///
/// Raven read this from GL during `GLimp_Init`. `Upload32`'s clamp loop would
/// mip every image down to nothing against a zero, so the seat uses wgpu's
/// `downlevel_defaults()` `max_texture_dimension_2d`: the smallest bound any
/// backend guarantees, and above every retail `base/` texture.
const SEED_MAX_TEXTURE_SIZE: c_int = 2048;

/// A `RenderAssets` at the state `R_Init`'s partial clear leaves it: the arenas
/// empty and the tables empty. `R_InitImages`/`R_InitShaders` fill it.
///
/// `shaders` and `skins` are capped with slot 0 pre-populated, and `R_Init`'s
/// own `Arena::reset` re-seats the real defaults. `images` stays unbounded,
/// because its purge is `R_DeleteTextures` rather than a reset.
pub fn empty_render_assets() -> RenderAssets {
    RenderAssets {
        images: Arena::new_unbounded(),
        image_names: Default::default(),
        default_image: None,
        fog_image: None,
        dlight_image: None,
        white_image: None,
        scratch_images: Vec::new(),
        lightmaps: Vec::new(),
        shaders: Arena::new_with_slot0(MAX_SHADERS, ShaderAsset::default()),
        shader_lookup: Default::default(),
        sorted_shaders: Vec::new(),
        shader_text: String::new(),
        shader_text_hash_table: Vec::new(),
        defer_load: false,
        skins: Arena::new_with_slot0(MAX_SKINS, SkinAsset::default()),
        skin_lookup: Default::default(),
        projection_shadow_shader: ShaderHandle::slot_zero(),
        sun_shader: ShaderHandle::slot_zero(),
        sky_parse: SkyParse::default(),
        world: None,
        external_vis_data: None,
        bsp_models: Vec::new(),
        function_tables: FunctionTables::default(),
        distance_cull: 0.0,
        distance_cull_squared: 0.0,
        glconfig: GlConfig {
            max_texture_size: SEED_MAX_TEXTURE_SIZE,
            ..GlConfig::default()
        },
        registered: false,
        world_map_loaded: false,
        max_polys: 0,
        max_polyverts: 0,
        automap_wireframe: AutomapWireframe {},
    }
}

/// A zeroed `FrameState` for `R_Init` to overwrite wholesale.
pub fn zeroed_frame_state() -> FrameState {
    FrameState {
        refdef: TrRefdef::default(),
        view: ViewParms::default(),
        ori: OrientationR::default(),
        counters: BackEndCounters {},
        is_hyperspace: false,
        current_entity: None,
        sky_rendered_this_view: false,
        projection_2d: false,
        color_2d: [0; 4],
        vertexes_2d: false,
        entity_2d: RefEntity::default(),
        scene_light_styles: [[0u8; 4]; MAX_LIGHT_STYLES],
        scene_count: 0,
        frame_scene_num: 0,
        view_cluster: 0,
        skyboxportal: 0,
        drawskyboxportal: 0,
        render_glowing_objects: false,
    }
}

/// A zeroed `viewParms_t` for the sky-shader parse carrier.
///
/// `viewParms_t` is a frozen `#[repr(C)]` struct of scalars, fixed arrays and
/// `#[repr(C)]` sub-structs, so the all-zero image is a valid value, and it is
/// what `Com_Memset(&tr.viewParms, 0, ...)` gives it in the oracle.
pub fn zeroed_view_parms() -> viewParms_t {
    // SAFETY: POD `#[repr(C)]`; see the doc comment.
    unsafe { core::mem::zeroed() }
}

/// A `SkyState` before any sky shader parses.
pub fn empty_sky_state() -> SkyState {
    SkyState {
        sky_mins: [[0.0; 6]; 2],
        sky_maxs: [[0.0; 6]; 2],
        sky_min: 0.0,
        sky_max: 0.0,
        sky_clip: [[0.0; 3]; 6],
        // The two cloud tables moved to `RenderAssets::sky_parse` (W2-F3).
        sky_points: Vec::new(),
        sky_tex_coords: Vec::new(),
    }
}

impl RendererFrontend {
    /// The seat `Engine.re` takes on a client build, before `R_Init` runs.
    ///
    /// Every field starts at the value Raven's loader-zeroed `tr` had, and
    /// `R_Init` fills the rest. The platform shell calls this at client boot
    /// (DEC-56); `jampded` leaves `Engine.re` as `None`.
    pub fn new() -> RendererFrontend {
        RendererFrontend {
            cvars: RendererCvars::default(),
            sim: RenderAssetsSim {
                published: Arc::new(empty_render_assets()),
                light_styles: LightStyleTable {
                    colors: [[0u8; 4]; MAX_LIGHT_STYLES],
                },
            },
            img_state: TrImageState::default(),
            frame: zeroed_frame_state(),
            world_load: WorldLoadState::default(),
            frame_data: FrameData { events: Vec::new() },
            frame_sink: None,
            pending_capture: None,
            pending_world: None,
            screenshot_last_number: -1,
            scene: SceneState::default(),
            noise: NoiseState::default(),
            rng: Rng::new(),
            font: FontState::default(),
            world_effects: WorldEffectsState::default(),
            qs: QSharedScratch::zeroed(),
            sky_view: zeroed_view_parms(),
            automap: WireframeAutomap::default(),
        }
    }
}

impl Default for RendererFrontend {
    fn default() -> RendererFrontend {
        RendererFrontend::new()
    }
}
