//! Dev harness for `mp_renderer_gpu` — opens a window, builds one frame's
//! `FrameData` per redraw, executes it through [`FrameExecutor`], and presents.
//! Exits on Escape or window close.
//!
//! R4a wave 1 (2D first light): this is the end-to-end proof that event
//! *production* -> *execution* -> *pixels* works. The harness plays the sim
//! side of the seam — it appends `SetColor`/`DrawStretchPic` events the way
//! `ui`/`cgame` traps will — and the render side replays them.
//!
//! R4a wave 2 (real textures): the harness now also plays the *registration*
//! side. [`dev_registries`] builds a checkerboard image in code, files it as
//! an `ImageAsset` with its pixels staged in `TrImageState::pending_uploads`
//! exactly as `R_CreateImage` does, and registers a one-stage `ShaderAsset`
//! pointing at it. Nothing is faked past that point: the executor drains the
//! staging table, resolves the quad's shader handle through the real registry
//! walk, and binds the real texture.
//!
//! No filesystem yet — `R_CreateImage` itself needs the whole engine carrier
//! list (`EngineHostView`, cvars, `RenderModels`), which arrives with the
//! ui-host wave. Until then the registry entries are built through the same
//! public arena/asset API that function writes through.
//!
//! **Single-threaded staging.** The frame stream is built and executed inline,
//! the same frame. DEC-37 ruling 2's sim/render thread split is a later R4
//! slice; when it lands, only this file changes — the `FrameData` arrives over
//! a channel instead of from `test_pattern()`, and `execute_frame` is called
//! unmodified.

use std::collections::HashMap;
use std::sync::Arc;

use mp_engine_qcommon::qfiles::draw_vert_t::MAXLIGHTMAPS;
use mp_renderer::gl_constants::{GL_REPEAT, GL_RGBA};
use mp_renderer::render_state::arena::Arena;
use mp_renderer::render_state::frame_data::FrameData;
use mp_renderer::render_state::frame_event::FrameEvent;
use mp_renderer::render_state::image_asset::ImageAsset;
use mp_renderer::render_state::placeholders::{AutomapWireframe, FunctionTables, GlConfig};
use mp_renderer::render_state::render_assets::RenderAssets;
use mp_renderer::render_state::sky_parse::SkyParse;
use mp_renderer::render_state::world_load_state::WorldLoadState;
use mp_renderer::render_state::render_cvar_snapshot::RenderCvarSnapshot;
use mp_renderer::render_state::shader_asset::{ShaderAsset, ShaderHandle};
use mp_renderer::render_state::shader_stage::ShaderStage;
use mp_renderer::tr_shader::{CullType, FogPass};
use mp_renderer::tr_image::{PendingUpload, TrImageState};
use mp_renderer::tr_noise::NoiseState;
use mp_renderer::tr_shader::ShaderStageParse;
use mp_renderer_gpu::{FrameExecutor, FrameStats, Gpu, GpuImages, GLS_2D_DEFAULT};
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

/// Side of the synthetic checkerboard, in texels. A power of two because
/// `R_CreateImage` rejects anything else.
const CHECKER_SIZE: i32 = 64;

/// Side of one checker cell, in texels.
const CHECKER_CELL: i32 = 8;

/// The registries and their sim-side staging table, standing in for the
/// `RenderAssetsSim`/`TrImageState` pair the engine will own.
struct DevRegistries {
    assets: RenderAssets,
    img_state: TrImageState,
    /// The shader whose stage 0 points at the checkerboard.
    checker: ShaderHandle,
}

struct App {
    window: Option<Arc<Window>>,
    gpu: Option<Gpu>,
    images: Option<GpuImages>,
    executor: Option<FrameExecutor>,
    registries: DevRegistries,
    /// Wall clock since boot, standing in for `ri.Milliseconds()` — the 2D
    /// shader clock `RB_SetGL2D` installs.
    start: std::time::Instant,
    /// The first frame's stats, printed once so a headless run leaves proof in
    /// the log that the events reached the GPU.
    reported: bool,
}

impl App {
    fn new() -> App {
        App {
            window: None,
            gpu: None,
            images: None,
            executor: None,
            registries: dev_registries(),
            start: std::time::Instant::now(),
            reported: false,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = Window::default_attributes().with_title("jka-rust dev harness");
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("create_window: failed to open the dev harness window"),
        );
        let gpu = Gpu::new(window.clone());
        let images = GpuImages::new(&gpu);
        let executor = FrameExecutor::new(&gpu, &images);
        window.request_redraw();

        self.window = Some(window);
        self.gpu = Some(gpu);
        self.images = Some(images);
        self.executor = Some(executor);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let (Some(window), Some(gpu), Some(images), Some(executor)) = (
            self.window.as_ref(),
            self.gpu.as_mut(),
            self.images.as_mut(),
            self.executor.as_mut(),
        ) else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => event_loop.exit(),
            WindowEvent::Resized(size) => {
                gpu.resize(size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                match gpu.begin_frame() {
                    Ok(frame) => {
                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        let stats = executor.execute_frame(
                            gpu,
                            &view,
                            &test_pattern(self.registries.checker),
                            &self.registries.assets,
                            &WorldLoadState::default(),
                            // 2D-only harness: no entity walk to host.
                            None,
                            self.registries.img_state.pending_uploads.drain().collect(),
                            images,
                            &NoiseState::default(),
                            self.start.elapsed().as_secs_f32(),
                            // No live cvar table in the harness, so the retail
                            // defaults apply.
                            RenderCvarSnapshot::default(),
                        );
                        if !self.reported {
                            self.reported = true;
                            report(&stats);
                        }
                        gpu.present(frame);
                    }
                    Err(_) => {
                        // Reconfigure at the current window size and retry next frame.
                        let size = window.inner_size();
                        gpu.resize(size.width, size.height);
                    }
                }
                window.request_redraw();
            }
            _ => {}
        }
    }
}

/// Builds the frame stream: four overlapping quads in the 640x480 virtual
/// screen — opaque red, green and blue staggered down the diagonal, then a
/// half-transparent white band across their overlap so the alpha blend is
/// visible — the checkerboard drawn twice beside them (once at 1:1 UVs,
/// once tiled 3x3 to exercise the wrap sampler).
///
/// The colour quads carry `ShaderHandle::slot_zero()`, the registries' default
/// entry (A12), which has no stages: they resolve to the white texel and
/// render as their flat vertex colour, exactly as in wave 1.
fn test_pattern(checker: ShaderHandle) -> FrameData {
    let mut events = Vec::new();

    let mut quad = |rgba: [f32; 4], shader: ShaderHandle, x, y, w, h, s2, t2| {
        events.push(FrameEvent::SetColor(rgba));
        events.push(FrameEvent::DrawStretchPic {
            x,
            y,
            w,
            h,
            s1: 0.0,
            t1: 0.0,
            s2,
            t2,
            shader,
        });
    };

    let white = [1.0, 1.0, 1.0, 1.0];
    let flat = ShaderHandle::slot_zero();

    quad(
        [1.0, 0.0, 0.0, 1.0],
        flat,
        40.0,
        80.0,
        180.0,
        150.0,
        1.0,
        1.0,
    );
    quad(
        [0.0, 1.0, 0.0, 1.0],
        flat,
        130.0,
        140.0,
        180.0,
        150.0,
        1.0,
        1.0,
    );
    quad(
        [0.0, 0.0, 1.0, 1.0],
        flat,
        220.0,
        200.0,
        180.0,
        150.0,
        1.0,
        1.0,
    );
    quad(
        [1.0, 1.0, 1.0, 0.5],
        flat,
        90.0,
        160.0,
        260.0,
        140.0,
        1.0,
        1.0,
    );

    // The real texture: 1:1, then tiled, then tinted to prove the fragment
    // shader still multiplies by the vertex colour.
    quad(white, checker, 400.0, 60.0, 128.0, 128.0, 1.0, 1.0);
    quad(white, checker, 400.0, 200.0, 128.0, 128.0, 3.0, 3.0);
    quad(
        [1.0, 0.8, 0.2, 1.0],
        checker,
        400.0,
        340.0,
        128.0,
        96.0,
        1.0,
        1.0,
    );

    FrameData { events }
}

/// Builds the checkerboard image and the shader that binds it, filed through
/// the same registry API `R_CreateImage`/`GeneratePermanentShader` write
/// through: an `ImageAsset` in the image arena, its RGBA8 pixels staged in
/// `pending_uploads` under that handle, and a one-stage `ShaderAsset` whose
/// `bundle[0].image` names it.
fn dev_registries() -> DevRegistries {
    let mut images = Arena::new_unbounded();
    let checker_image = images.insert(ImageAsset {
        img_name: String::from("*devchecker"),
        width: CHECKER_SIZE,
        height: CHECKER_SIZE,
        internal_format: GL_RGBA,
        // Repeat, so the tiled quad's 0..3 UVs wrap instead of clamping.
        wrap_clamp_mode: GL_REPEAT,
        ..Default::default()
    });
    let mut image_names = HashMap::new();
    image_names.insert(String::from("*devchecker"), checker_image);

    let mut img_state = TrImageState::default();
    img_state.pending_uploads.insert(
        checker_image,
        PendingUpload {
            pixels: checkerboard(),
            width: CHECKER_SIZE,
            height: CHECKER_SIZE,
        },
    );

    // Slot 0 is the registries' default entry (A12), shaped like
    // `CreateInternalShaders`' `<default>`: one active stage whose image is
    // `tr.defaultImage`. This harness has no builtin images, so that stage
    // binds nothing and resolves to the white texel.
    let mut shaders = Arena::new_unbounded();
    let default_stage = ShaderStage::from(&ShaderStageParse {
        active: true,
        state_bits: GLS_2D_DEFAULT,
        ..Default::default()
    });
    shaders.insert(shader_asset("<default>", vec![default_stage]));

    let mut stage = ShaderStage::from(&ShaderStageParse {
        active: true,
        state_bits: GLS_2D_DEFAULT,
        ..Default::default()
    });
    stage.bundle[0].image = Some(checker_image);
    let checker = shaders.insert(shader_asset("dev/checker", vec![stage]));

    let mut shader_lookup = HashMap::new();
    shader_lookup.insert(String::from("dev/checker"), vec![checker]);

    let assets = RenderAssets {
        images,
        image_names,
        default_image: None,
        fog_image: None,
        dlight_image: None,
        white_image: None,
        scratch_images: Vec::new(),
        lightmaps: Vec::new(),
        shaders,
        shader_lookup,
        sorted_shaders: Vec::new(),
        shader_text: String::new(),
        shader_text_hash_table: Vec::new(),
        defer_load: false,
        skins: Arena::new_unbounded(),
        skin_lookup: HashMap::new(),
        projection_shadow_shader: ShaderHandle::slot_zero(),
        sun_shader: ShaderHandle::slot_zero(),
        world: None,
        external_vis_data: None,
        sky_parse: SkyParse::default(),
        bsp_models: Vec::new(),
        function_tables: FunctionTables::default(),
        distance_cull: 0.0,
        distance_cull_squared: 0.0,
        glconfig: GlConfig::default(),
        registered: true,
        world_map_loaded: false,
        max_polys: 0,
        max_polyverts: 0,
        automap_wireframe: AutomapWireframe {},
    };

    DevRegistries {
        assets,
        img_state,
        checker,
    }
}

/// A `CHECKER_SIZE`-square RGBA8 checkerboard. The top-left cell is red so
/// the texture's orientation — and therefore the UV convention — is readable
/// on screen; the rest alternates white and dark teal.
fn checkerboard() -> Vec<u8> {
    let mut pixels = Vec::with_capacity((CHECKER_SIZE * CHECKER_SIZE * 4) as usize);
    for y in 0..CHECKER_SIZE {
        for x in 0..CHECKER_SIZE {
            let cell = (x / CHECKER_CELL + y / CHECKER_CELL) % 2 == 0;
            let texel = match (x < CHECKER_CELL && y < CHECKER_CELL, cell) {
                (true, _) => [0xe0, 0x20, 0x20, 0xff],
                (false, true) => [0xf0, 0xf0, 0xf0, 0xff],
                (false, false) => [0x10, 0x50, 0x60, 0xff],
            };
            pixels.extend_from_slice(&texel);
        }
    }
    pixels
}

/// A `ShaderAsset` with every field at the value `GeneratePermanentShader`
/// leaves it for a plain, explicitly-defined 2D shader.
fn shader_asset(name: &str, stages: Vec<ShaderStage>) -> ShaderAsset {
    ShaderAsset {
        name: String::from(name),
        lightmap_index: [0; MAXLIGHTMAPS],
        styles: [0; MAXLIGHTMAPS],
        sort: 0.0,
        sorted_index: 0,
        cull_type: CullType::FrontSided,
        surface_flags: 0,
        content_flags: 0,
        multitexture_env: 0,
        default_shader: false,
        explicitly_defined: true,
        num_unfogged_passes: stages.len() as i32,
        sky: None,
        fog_parms: None,
        fog_pass: FogPass::None,
        stages,
        time_offset: 0.0,
        remapped_shader: None,
    }
}

fn report(stats: &FrameStats) {
    println!(
        "dev_harness: first frame executed — {} images uploaded, {} quads, \
         {} color changes, {} draw calls, {} events skipped",
        stats.images_uploaded,
        stats.quads,
        stats.color_changes,
        stats.draw_calls,
        stats.skipped_events()
    );
}

fn main() {
    let event_loop = EventLoop::new().expect("EventLoop::new: failed to create the event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop
        .run_app(&mut app)
        .expect("run_app: dev harness event loop exited with an error");
}
