//! `world_harness` — R4 world wave: a window that flies a free camera through a
//! real BSP and draws it through the whole render chain.
//!
//! This bin proves the world path end to end, the same way `ui_harness` proves
//! the 2D path. It boots the engine subset and renderer CPU frontend, loads
//! `maps/mp/duel1.bsp`, opens a window, and every frame builds a `refdef_t`
//! from the free-fly camera, records it with the trap-side `RE_RenderScene`,
//! and drives the executor. The executor replays the event, runs `R_RenderView`
//! (DEC-50), and draws the sorted world surfaces. Nothing is hand-built past
//! the camera: the harness never touches `viewParms_t` directly.
//!
//! Controls: WASD moves, the mouse looks, Space and Left-Control move up and
//! down, Escape quits.
//!
//! Usage: `cargo run --release -p mp_renderer_gpu --bin world_harness
//! [-- <basepath> [map]]`.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use mp_engine_core::Engine;
use mp_engine_qcommon::cm_terrain::CmLandScape;
use mp_engine_server::Server;
use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use mp_renderer::render_state::frame_data::FrameData;
use mp_renderer::render_state::render_assets::RenderAssets;
use mp_renderer::tr_local::dlight_s::dlight_t;
use mp_renderer::tr_local::fog_t::fog_t;
use mp_renderer::tr_local::srf_terrain_s::srfTerrain_t;
use mp_renderer::tr_local::tr_ref_entity_t::trRefEntity_t;
use mp_renderer::tr_main::TrMainScratch;
use mp_renderer::tr_model::render_models::RenderModels;
use mp_renderer::tr_scene::RE_RenderScene;
use mp_renderer_gpu::ui_host::boot;
use mp_renderer_gpu::ui_host::{BootConfig, UiHost};
use mp_renderer_gpu::{FrameExecutor, FrameStats, Gpu, GpuImages, WorldFrame};
use native_math::qmath::{AngleVectors, AnglesToAxis};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

/// Camera move speed in world units per second. Raven maps use roughly 320
/// units per two meters, so this covers a map in a few seconds.
const MOVE_SPEED: f32 = 500.0;

/// Mouse look sensitivity in degrees per pixel of raw motion.
const MOUSE_SENS: f32 = 0.12;

/// Eye height added to a spawn origin, matching `world_spike`'s bump.
const EYE_HEIGHT: f32 = 40.0;

/// The free-fly camera. `pitch`/`yaw` are Raven view angles in degrees.
struct Camera {
    pos: [f32; 3],
    pitch: f32,
    yaw: f32,
}

struct App {
    window: Option<Arc<Window>>,
    gpu: Option<Gpu>,
    images: Option<GpuImages>,
    executor: Option<FrameExecutor>,
    host: UiHost,
    /// The 2D command surface reads these, and this harness draws no 2D, so
    /// they stand in for the `assets`/`image_assets` parameters. The world's
    /// own registry (`host.assets`) is borrowed by the `WorldFrame`, so it
    /// cannot also fill those parameters. The harness therefore drains the
    /// staged image uploads against the real registry itself, before the
    /// split borrow (see `draw_world_frame`).
    dummy_assets: RenderAssets,
    /// The null-landscape terrain surface, initialized once and reused every
    /// frame.
    land_scape: srfTerrain_t,
    land: CmLandScape,
    /// Empty per-frame scratch buffers. Dlights, fogs, and entities are later
    /// waves.
    dlights: Vec<dlight_t>,
    fogs: Vec<fog_t>,
    entities: Vec<trRefEntity_t>,
    scratch: TrMainScratch,
    camera: Camera,
    /// The movement keys currently held down.
    keys: HashSet<KeyCode>,
    start: Instant,
    /// The previous frame's instant, so movement scales by real elapsed time.
    last_frame: Instant,
    /// The last window size, so the refdef viewport tracks the window.
    surface: (f32, f32),
    reported: bool,
}

impl App {
    fn new(
        host: UiHost,
        land_scape: srfTerrain_t,
        dummy_assets: RenderAssets,
        eye: [f32; 3],
    ) -> App {
        App {
            window: None,
            gpu: None,
            images: None,
            executor: None,
            host,
            dummy_assets,
            land_scape,
            land: CmLandScape::empty(),
            dlights: Vec::new(),
            fogs: Vec::new(),
            entities: Vec::new(),
            scratch: TrMainScratch {
                pre_trans_ent_matrix: [0.0; 16],
            },
            camera: Camera {
                pos: eye,
                pitch: 0.0,
                yaw: 0.0,
            },
            keys: HashSet::new(),
            start: Instant::now(),
            last_frame: Instant::now(),
            surface: (1280.0, 720.0),
            reported: false,
        }
    }

    /// Moves the camera along its forward and right vectors from the held keys.
    fn update_camera(&mut self, dt: f32) {
        let angles = [self.camera.pitch, self.camera.yaw, 0.0];
        let mut forward = [0.0f32; 3];
        let mut right = [0.0f32; 3];
        AngleVectors(angles, Some(&mut forward), Some(&mut right), None);

        let mut delta = [0.0f32; 3];
        if self.keys.contains(&KeyCode::KeyW) {
            for i in 0..3 {
                delta[i] += forward[i];
            }
        }
        if self.keys.contains(&KeyCode::KeyS) {
            for i in 0..3 {
                delta[i] -= forward[i];
            }
        }
        if self.keys.contains(&KeyCode::KeyD) {
            for i in 0..3 {
                delta[i] += right[i];
            }
        }
        if self.keys.contains(&KeyCode::KeyA) {
            for i in 0..3 {
                delta[i] -= right[i];
            }
        }
        if self.keys.contains(&KeyCode::Space) {
            delta[2] += 1.0;
        }
        if self.keys.contains(&KeyCode::ControlLeft) {
            delta[2] -= 1.0;
        }

        let speed = MOVE_SPEED * dt;
        for i in 0..3 {
            self.camera.pos[i] += delta[i] * speed;
        }
    }

    /// Turns the raw mouse motion into a yaw/pitch change, clamping pitch so the
    /// camera never flips over.
    fn look(&mut self, dx: f32, dy: f32) {
        self.camera.yaw -= dx * MOUSE_SENS;
        self.camera.pitch += dy * MOUSE_SENS;
        self.camera.pitch = self.camera.pitch.clamp(-89.0, 89.0);
    }

    /// Builds this frame's scene definition from the camera and window size.
    /// `fov_y` follows Raven's `CalcFov`: a fixed horizontal fov, the vertical
    /// derived from the window aspect.
    fn build_refdef(&self, time_ms: i32) -> refdef_t {
        let (w, h) = self.surface;

        // SAFETY: `refdef_t` is a frozen `#[repr(C)]` POD of scalars, fixed
        // arrays, and `vec3_t`, so an all-zero value is valid.
        let mut rd: refdef_t = unsafe { core::mem::zeroed() };
        rd.x = 0;
        rd.y = 0;
        rd.width = w as i32;
        rd.height = h as i32;

        let fov_x = 90.0f64;
        rd.fov_x = fov_x as f32;
        // `x = width / tan(fov_x / 360 * PI); fov_y = atan2(height, x) * 360 / PI`.
        let x = (w as f64) / (fov_x / 360.0 * std::f64::consts::PI).tan();
        let fov_y = (h as f64).atan2(x) * 360.0 / std::f64::consts::PI;
        rd.fov_y = fov_y as f32;

        rd.vieworg = self.camera.pos;
        let angles = [self.camera.pitch, self.camera.yaw, 0.0];
        rd.viewangles = angles;
        AnglesToAxis(angles, rd.viewaxis.as_mut_ptr());

        rd.time = time_ms;
        rd.rdflags = 0;
        rd
    }

    /// Records this frame's scene through the trap-side `RE_RenderScene`, which
    /// pushes a `FrameEvent::RenderScene` the executor replays.
    fn record_scene(&mut self, refdef: &refdef_t) -> FrameData {
        let mut frame_data = FrameData { events: Vec::new() };
        RE_RenderScene(
            refdef,
            &mut frame_data,
            &self.host.assets,
            &self.host.cvars,
            &mut self.host.scene,
            &mut self.host.engine.common,
            &self.host.sim.light_styles,
        );
        frame_data
    }

    /// One frame: advance the camera, record the scene, draw it.
    fn frame(&mut self) {
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;

        self.update_camera(dt);
        let time_ms = self.start.elapsed().as_millis() as i32;
        let float_time = self.start.elapsed().as_secs_f32();
        let refdef = self.build_refdef(time_ms);
        let frame_data = self.record_scene(&refdef);
        self.draw_world_frame(&frame_data, float_time);
    }

    /// Acquires the frame target, builds the world context, and drives the
    /// executor. The executor runs the whole world chain and presents.
    fn draw_world_frame(&mut self, frame_data: &FrameData, float_time: f32) {
        let App {
            host,
            gpu,
            executor,
            images,
            window,
            dummy_assets,
            land_scape,
            land,
            dlights,
            fogs,
            entities,
            scratch,
            reported,
            ..
        } = self;
        let (Some(gpu), Some(executor), Some(images), Some(window)) = (
            gpu.as_mut(),
            executor.as_mut(),
            images.as_mut(),
            window.as_ref(),
        ) else {
            return;
        };

        match gpu.begin_frame() {
            Ok(frame) => {
                let target = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                // Drain the staged image uploads before the split borrow
                // below. `execute_frame` drains with its `image_assets`
                // parameter, which this harness fills with `dummy_assets`,
                // and a drain against an empty registry drops every staged
                // world texture and lightmap for good.
                // Image registration writes the sim-published master (A9),
                // so the drain resolves the staged handles there, not in
                // `host.assets`.
                let uploaded = images.upload_pending(gpu, &mut host.img_state, &host.sim.published);

                let mut stats = {
                    // Split the host and engine into disjoint borrows, the same
                    // shape `load_world_and_render` builds.
                    let UiHost {
                        engine,
                        models,
                        cvars,
                        assets,
                        frame: fstate,
                        gpu_res,
                        img_state,
                        font,
                        noise,
                        ..
                    } = host;
                    let models_ptr: *mut RenderModels = &mut *models;
                    let Engine { common, cm, sv, .. } = &mut **engine;
                    let sv_ptr: *mut () = sv as *mut Server as *mut ();
                    let mut engine_view = boot::host_view(common, cm, sv_ptr, models_ptr);

                    let mut world = WorldFrame {
                        engine_view: &mut engine_view,
                        assets,
                        cvars,
                        frame: fstate,
                        gpu_res,
                        models: &*models,
                        land_scape: &*land_scape,
                        land: &*land,
                        dlights: dlights.as_mut_slice(),
                        fogs: fogs.as_slice(),
                        entities: entities.as_mut_slice(),
                        scratch,
                    };

                    executor.execute_frame(
                        gpu,
                        &target,
                        frame_data,
                        &*dummy_assets,
                        &*dummy_assets,
                        img_state,
                        images,
                        font,
                        noise,
                        float_time,
                        Some(&mut world),
                    )
                };

                stats.images_uploaded += uploaded as u32;

                if !*reported {
                    *reported = true;
                    report(&stats);
                }
                gpu.present(frame);
            }
            Err(_) => {
                // The surface reconfigure must resize the executor too. The
                // world pass needs a depth texture that matches the color
                // target size, or wgpu rejects the pass.
                let size = window.inner_size();
                gpu.resize(size.width, size.height);
                executor.resize(gpu, size.width, size.height);
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes().with_title("jka-rust world harness");
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("create_window: failed to open the world harness window"),
        );
        let gpu = Gpu::new(window.clone());
        let images = GpuImages::new(&gpu);
        let mut executor = FrameExecutor::new(&gpu, &images);

        // Upload the loaded world's geometry once, before the first frame.
        if let Some(world) = self.host.assets.world.as_ref() {
            executor.set_world(&gpu, world);
        }

        let size = window.inner_size();
        self.surface = (size.width.max(1) as f32, size.height.max(1) as f32);

        // Lock the pointer for mouse look. A platform that refuses lock falls
        // back to confine, then to nothing.
        window.set_cursor_visible(false);
        let _ = window
            .set_cursor_grab(CursorGrabMode::Locked)
            .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));

        window.request_redraw();

        self.window = Some(window);
        self.gpu = Some(gpu);
        self.images = Some(images);
        self.executor = Some(executor);
        self.last_frame = Instant::now();
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _id: DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.look(delta.0 as f32, delta.1 as f32);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                self.surface = (size.width.max(1) as f32, size.height.max(1) as f32);
                if let (Some(gpu), Some(executor)) = (self.gpu.as_mut(), self.executor.as_mut()) {
                    gpu.resize(size.width, size.height);
                    executor.resize(gpu, size.width, size.height);
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        ..
                    },
                ..
            } => {
                if code == KeyCode::Escape && state == ElementState::Pressed {
                    event_loop.exit();
                    return;
                }
                match state {
                    ElementState::Pressed => {
                        self.keys.insert(code);
                    }
                    ElementState::Released => {
                        self.keys.remove(&code);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                self.frame();
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn report(stats: &FrameStats) {
    println!(
        "world_harness: first frame — {} images uploaded, {} world surfaces drawn \
         ({} lightmapped, {} draw calls), {} non-world skipped, {} empty surfaces",
        stats.images_uploaded,
        stats.world.surfaces_drawn,
        stats.world.lightmapped,
        stats.world.draw_calls,
        stats.world.skipped_non_world,
        stats.world.empty_surfaces,
    );
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mut cfg = BootConfig::default();
    if let Some(basepath) = args.next() {
        cfg.basepath = basepath;
    }
    let map = args
        .next()
        .unwrap_or_else(|| String::from("maps/mp/duel1.bsp"));

    let mut host = boot::boot(&cfg);
    let (loaded, land_scape) = boot::load_world(&mut host, &map);
    if !loaded {
        eprintln!("world_harness: {map} did not load, exiting");
        return;
    }

    // Force the first frame's `R_MarkLeaves` to re-mark regardless of the
    // leftover view cluster, the same first-mark guarantee `load_world_and_render`
    // gets from forcing `areamask_modified`.
    host.frame.view_cluster = -1;

    // `RE_RenderScene` returns before it pushes the scene event while the
    // renderer is unregistered. Only `RE_BeginRegistration` sets the flag
    // (`tr_model/frontend.rs:791`), and this harness boots through the ui
    // path without it, so we set the flag here.
    host.assets.registered = true;

    // Start the camera at a spawn origin, bumped to eye height.
    let eye = host
        .assets
        .world
        .as_ref()
        .and_then(|w| boot::find_spawn_origin(&w.entity_string))
        .map(|o| [o[0], o[1], o[2] + EYE_HEIGHT])
        .unwrap_or([0.0, 0.0, 0.0]);
    println!("world_harness: camera at {eye:?}");

    let dummy_assets = boot::empty_assets();
    let mut app = App::new(host, land_scape, dummy_assets, eye);

    let event_loop = EventLoop::new().expect("EventLoop::new: failed to create the event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop
        .run_app(&mut app)
        .expect("run_app: world harness event loop exited with an error");
}
