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
use mp_engine_ghoul2::api_models::g2api_init_ghoul2_model;
use mp_engine_ghoul2::ghoul2_system::Ghoul2System;
use mp_engine_ghoul2::info_array::Ghoul2Handle;
use mp_engine_ghoul2::shared::cghoul2_info_v::CGhoul2Info_v;
use mp_engine_qcommon::cm_terrain::CmLandScape;
use mp_engine_server::Server;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use mp_qshared::shared::qhandle_t;
use mp_renderer::render_state::frame_data::FrameData;
use mp_renderer::render_state::render_assets::RenderAssets;
use mp_renderer::tr_local::dlight_s::dlight_t;
use mp_renderer::tr_local::srf_terrain_s::srfTerrain_t;
use mp_renderer::tr_main::TrMainScratch;
use mp_renderer::tr_model::render_models::RenderModels;
use mp_renderer::tr_scene::{
    ghoul2_token_encode, RE_AddRefEntityToScene, RE_ClearScene, RE_RenderScene,
};
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

/// The test entity's vertical bob amplitude in world units. Inline brush
/// geometry sits at absolute map coordinates, so the entity origin stays at
/// zero and only this offset moves, the func_plat motion shape.
const ENTITY_BOB_AMPLITUDE: f32 = 48.0;

/// The test entity's bob period in seconds.
const ENTITY_BOB_PERIOD: f32 = 3.0;

/// The MD3 test entity's height above the brush entity's geometry center.
const MD3_LIFT: f32 = 64.0;

/// The MD3 test entity's yaw spin rate in degrees per second. MD3 vertices are
/// entity-local, so a spin is correct there, unlike the brush entity.
const MD3_SPIN_RATE: f32 = 45.0;

/// The map object the MD3 test entity draws — the model duel1 mounts on its
/// func_bobbing.
const MD3_MODEL_NAME: &str = "models/map_objects/bespin/twinpodcc.md3";

/// The Ghoul2 (`.glm`) skinned model the third test entity draws — a shipped
/// player model in its base skeleton pose (no animation).
const GHOUL2_MODEL_NAME: &str = "models/players/stormtrooper/model.glm";

/// The Ghoul2 test entity's height above the brush entity's geometry center,
/// beside the MD3 entity.
const GHOUL2_LIFT: f32 = 64.0;

/// The Ghoul2 test entity's sideways offset from the MD3 entity, so the two do
/// not overlap.
const GHOUL2_SIDE_OFFSET: f32 = 96.0;

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
    /// Empty per-frame scratch buffers. Dlights and entities are later waves.
    /// The fog list is not held here: `render_world` copies it from the loaded
    /// world each frame.
    dlights: Vec<dlight_t>,
    scratch: TrMainScratch,
    camera: Camera,
    /// The brush submodel handle the one test entity draws (`*1`), computed
    /// once at boot. The entity origin is the per-frame bob, not a field.
    test_model: qhandle_t,
    /// The MD3 map-object handle the second test entity draws, 0 when the model
    /// file is absent.
    md3_model: qhandle_t,
    /// The world-space point the MD3 test entity sits above (the `*1` geometry
    /// center, or the eye when the map has no inline model).
    md3_center: [f32; 3],
    /// The live Ghoul2 state, threaded into every `WorldFrame`. It holds the
    /// bone caches the render path builds each frame, so it persists across
    /// frames rather than reset per frame.
    g2: Ghoul2System,
    /// The Ghoul2 instance handle the third test entity carries in its
    /// `refEntity_t.ghoul2` token, `None` when the `.glm` file is absent.
    ghoul2_handle: Option<Ghoul2Handle>,
    /// The `.glm` model handle the Ghoul2 test entity draws (its `hModel`), 0
    /// when the model file is absent.
    ghoul2_model: qhandle_t,
    /// The movement keys currently held down.
    keys: HashSet<KeyCode>,
    start: Instant,
    /// The previous frame's instant, so movement scales by real elapsed time.
    last_frame: Instant,
    /// The last window size, so the refdef viewport tracks the window.
    surface: (f32, f32),
    reported: bool,
    /// One log line for a failed surface acquire, so an occluded window
    /// cannot flood stderr.
    surface_warned: bool,
}

impl App {
    #[allow(clippy::too_many_arguments)]
    fn new(
        host: UiHost,
        land_scape: srfTerrain_t,
        dummy_assets: RenderAssets,
        eye: [f32; 3],
        test_model: qhandle_t,
        md3_model: qhandle_t,
        md3_center: [f32; 3],
        g2: Ghoul2System,
        ghoul2_handle: Option<Ghoul2Handle>,
        ghoul2_model: qhandle_t,
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
            scratch: TrMainScratch {
                pre_trans_ent_matrix: [0.0; 16],
            },
            camera: Camera {
                pos: eye,
                pitch: 0.0,
                yaw: 0.0,
            },
            test_model,
            md3_model,
            md3_center,
            g2,
            ghoul2_handle,
            ghoul2_model,
            keys: HashSet::new(),
            start: Instant::now(),
            last_frame: Instant::now(),
            surface: (1280.0, 720.0),
            reported: false,
            surface_warned: false,
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

    /// Records this frame's scene through the trap-side traps, which push the
    /// `FrameEvent`s the executor replays. The scene clears, adds one spinning
    /// brush-model entity, then renders. The order matches a real cgame frame.
    fn record_scene(&mut self, refdef: &refdef_t) -> FrameData {
        let mut frame_data = FrameData { events: Vec::new() };

        RE_ClearScene(&mut frame_data, &mut self.host.scene);
        self.record_test_entity(&mut frame_data, refdef.time);
        self.record_md3_entity(&mut frame_data, refdef.time);
        self.record_ghoul2_entity(&mut frame_data, refdef.time);

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

    /// Records one test brush-model entity through the trap-side
    /// `RE_AddRefEntityToScene`. Inline brush geometry lives at absolute map
    /// coordinates, so the origin carries only a vertical bob and the axis
    /// stays identity. The bob shows the per-entity transform as motion. A
    /// missing `*1` submodel handle (a map with no inline models) skips the
    /// entity.
    fn record_test_entity(&mut self, frame_data: &mut FrameData, time_ms: i32) {
        if self.test_model == 0 {
            return;
        }

        let bob_phase = (time_ms as f32) * 0.001 / ENTITY_BOB_PERIOD * std::f32::consts::TAU;
        let bob = ENTITY_BOB_AMPLITUDE * bob_phase.sin();

        let mut ent = refEntity_t::zeroed();
        ent.reType = refEntityType_t::RT_MODEL;
        ent.hModel = self.test_model;
        ent.origin = [0.0, 0.0, bob];
        ent.oldorigin = ent.origin;
        ent.shaderRGBA = [255, 255, 255, 255];
        AnglesToAxis([0.0, 0.0, 0.0], ent.axis.as_mut_ptr());

        RE_AddRefEntityToScene(frame_data, &self.host.assets, &mut self.host.scene, &ent);
    }

    /// Records the MD3 map-object entity through `RE_AddRefEntityToScene`. It
    /// sits above the brush entity's geometry center with the same vertical bob
    /// plus a slow yaw spin. MD3 vertices are entity-local, so the spin rotates
    /// the model in place. A missing model handle skips the entity.
    fn record_md3_entity(&mut self, frame_data: &mut FrameData, time_ms: i32) {
        if self.md3_model == 0 {
            return;
        }

        let seconds = time_ms as f32 * 0.001;
        let bob_phase = seconds / ENTITY_BOB_PERIOD * std::f32::consts::TAU;
        let bob = ENTITY_BOB_AMPLITUDE * bob_phase.sin();
        let yaw = (seconds * MD3_SPIN_RATE) % 360.0;

        let mut ent = refEntity_t::zeroed();
        ent.reType = refEntityType_t::RT_MODEL;
        ent.hModel = self.md3_model;
        ent.origin = [
            self.md3_center[0],
            self.md3_center[1],
            self.md3_center[2] + MD3_LIFT + bob,
        ];
        ent.oldorigin = ent.origin;
        ent.frame = 0;
        ent.oldframe = 0;
        ent.shaderRGBA = [255, 255, 255, 255];
        AnglesToAxis([0.0, yaw, 0.0], ent.axis.as_mut_ptr());

        RE_AddRefEntityToScene(frame_data, &self.host.assets, &mut self.host.scene, &ent);
    }

    /// Records the Ghoul2 skinned test entity through `RE_AddRefEntityToScene`.
    /// It sits beside the MD3 entity above the brush entity's geometry center,
    /// with the same vertical bob plus a slow yaw spin, and carries the Ghoul2
    /// instance handle in its `refEntity_t.ghoul2` token. The render path builds
    /// the skeleton and deforms the surfaces each frame. A missing model handle
    /// or instance skips the entity.
    fn record_ghoul2_entity(&mut self, frame_data: &mut FrameData, time_ms: i32) {
        let Some(handle) = self.ghoul2_handle else {
            return;
        };
        if self.ghoul2_model == 0 {
            return;
        }

        let seconds = time_ms as f32 * 0.001;
        let bob_phase = seconds / ENTITY_BOB_PERIOD * std::f32::consts::TAU;
        let bob = ENTITY_BOB_AMPLITUDE * bob_phase.sin();
        let yaw = (seconds * MD3_SPIN_RATE) % 360.0;

        let mut ent = refEntity_t::zeroed();
        ent.reType = refEntityType_t::RT_MODEL;
        ent.hModel = self.ghoul2_model;
        ent.ghoul2 = ghoul2_token_encode(Some(handle));
        ent.origin = [
            self.md3_center[0] + GHOUL2_SIDE_OFFSET,
            self.md3_center[1],
            self.md3_center[2] + GHOUL2_LIFT + bob,
        ];
        ent.oldorigin = ent.origin;
        ent.frame = 0;
        ent.oldframe = 0;
        ent.shaderRGBA = [255, 255, 255, 255];
        AnglesToAxis([0.0, yaw, 0.0], ent.axis.as_mut_ptr());

        RE_AddRefEntityToScene(frame_data, &self.host.assets, &mut self.host.scene, &ent);
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
            scratch,
            reported,
            surface_warned,
            md3_model,
            g2,
            ghoul2_handle,
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
                        sky,
                        ..
                    } = host;
                    let models_ptr: *mut RenderModels = &mut *models;
                    let Engine { common, cm, sv, .. } = &mut **engine;
                    let sv_ptr: *mut () = sv as *mut Server as *mut ();
                    let mut engine_view = boot::host_view(common, cm, sv_ptr, models_ptr);

                    // The persisted Ghoul2 state threads into the frame, so the
                    // bone caches the render path builds survive across frames
                    // (design point 2).
                    let mut world = WorldFrame {
                        engine_view: &mut engine_view,
                        assets,
                        cvars,
                        frame: fstate,
                        g2,
                        gpu_res,
                        sky,
                        models: &*models,
                        land_scape: &*land_scape,
                        land: &*land,
                        dlights: dlights.as_mut_slice(),
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
                    report(&stats, *md3_model, *ghoul2_handle);
                }
                gpu.present(frame);
            }
            Err(error) => {
                // The surface reconfigure must resize the executor too. The
                // world pass needs a depth texture that matches the color
                // target size, or wgpu rejects the pass.
                if !*surface_warned {
                    *surface_warned = true;
                    eprintln!(
                        "world_harness: begin_frame failed ({error:?}) - an occluded window skips frames until it is visible",
                    );
                }
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

fn report(stats: &FrameStats, md3_model: qhandle_t, ghoul2_handle: Option<Ghoul2Handle>) {
    let ghoul2_handle = ghoul2_handle.map(|h| h.0).unwrap_or(-1);
    println!(
        "world_harness: first frame — {} images uploaded, {} world surfaces drawn \
         ({} lightmapped, {} draw calls), {} non-world skipped, {} empty surfaces, \
         {} entities ({} entity surfaces drawn), {} sky surfaces drawn, \
         md3 handle {} ({} md3 entity surfaces, \
         {} md3 decode failed), ghoul2 handle {} ({} ghoul2 surfaces drawn, \
         {} ghoul2 decode failed), {} fog passes drawn",
        stats.images_uploaded,
        stats.world.surfaces_drawn,
        stats.world.lightmapped,
        stats.world.draw_calls,
        stats.world.skipped_non_world,
        stats.world.empty_surfaces,
        stats.entities,
        stats.world.entity_surfaces_drawn,
        stats.world.sky_surfaces_drawn,
        md3_model,
        stats.world.md3_surfaces_drawn,
        stats.world.md3_decode_failed,
        ghoul2_handle,
        stats.world.ghoul2_surfaces_drawn,
        stats.world.ghoul2_decode_failed,
        stats.world.fog_passes_drawn,
    );
}

/// Inits one Ghoul2 model instance through the real `mp_engine_ghoul2` init
/// path (`G2API_InitGhoul2Model`). It allocates a `CGhoul2Info_v` handle, loads
/// the `.glm` through the renderer model path the init helper drives, and reads
/// back the instance's model handle. Returns the live system, the instance
/// handle for the `refEntity_t.ghoul2` token, and the model handle for
/// `hModel`. Returns `None` when the model file is absent (the init returns a
/// negative model index), so the harness draws no Ghoul2 entity.
fn init_ghoul2(host: &mut UiHost, name: &str) -> Option<(Ghoul2System, Ghoul2Handle, qhandle_t)> {
    let mut g2 = Ghoul2System::default();
    let mut info = CGhoul2Info_v { mItem: 0 };
    info.alloc(&mut g2);

    let model_index = {
        let UiHost {
            engine, models, ..
        } = &mut *host;
        let models_ptr: *mut RenderModels = &mut *models;
        let Engine { common, cm, sv, .. } = &mut **engine;
        let sv_ptr: *mut () = sv as *mut Server as *mut ();
        let mut view = boot::host_view(common, cm, sv_ptr, models_ptr);
        g2api_init_ghoul2_model(&mut g2, &mut view, &mut info, name, 0, 0, 0, 0, 0)
    };
    if model_index < 0 {
        return None;
    }

    let model_handle = info.get(&g2, 0).model;
    Some((g2, Ghoul2Handle(info.mItem), model_handle))
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

    // The first inline brush submodel (`*1`) is the one test entity. A map with
    // no inline models leaves the handle at 0, and the harness draws no entity.
    let test_model = host.models.handle_for_name("*1").unwrap_or(0);
    println!("world_harness: test entity model handle *1 = {test_model}");

    // Inline brush geometry sits at absolute map coordinates, so the entity
    // shows at its compile spot. Aim the starting camera at that spot.
    let entity_center = host
        .assets
        .world
        .as_ref()
        .and_then(|w| w.bmodels.get(1))
        .map(|b| {
            [
                (b.bounds[0][0] + b.bounds[1][0]) * 0.5,
                (b.bounds[0][1] + b.bounds[1][1]) * 0.5,
                (b.bounds[0][2] + b.bounds[1][2]) * 0.5,
            ]
        });
    if let Some(c) = entity_center {
        println!("world_harness: test entity geometry center {c:?}");
    }

    // Register the MD3 map object through the real RE_RegisterModel chain, the
    // model duel1 mounts on its func_bobbing. A missing file leaves the handle
    // at 0, and the harness draws no MD3 entity.
    let md3_model = boot::register_model(&mut host, MD3_MODEL_NAME);
    if md3_model == 0 {
        println!("world_harness: MD3 model {MD3_MODEL_NAME} absent, skipping md3 entity");
    } else {
        println!("world_harness: md3 entity model {MD3_MODEL_NAME} = {md3_model}");
    }

    // The MD3 entity sits above the brush entity's geometry center, or the eye
    // when the map has no inline model.
    let md3_center = entity_center.unwrap_or(eye);

    // Init one Ghoul2 model through the real init path. A missing `.glm` file
    // leaves the state empty and the harness draws no Ghoul2 entity.
    let (g2, ghoul2_handle, ghoul2_model) = match init_ghoul2(&mut host, GHOUL2_MODEL_NAME) {
        Some((g2, handle, model)) => {
            println!(
                "world_harness: ghoul2 entity model {GHOUL2_MODEL_NAME} = {model}, \
                 instance handle {}",
                handle.0
            );
            (g2, Some(handle), model)
        }
        None => {
            println!(
                "world_harness: Ghoul2 model {GHOUL2_MODEL_NAME} absent, skipping ghoul2 entity"
            );
            (Ghoul2System::default(), None, 0)
        }
    };

    let dummy_assets = boot::empty_assets();
    let mut app = App::new(
        host,
        land_scape,
        dummy_assets,
        eye,
        test_model,
        md3_model,
        md3_center,
        g2,
        ghoul2_handle,
        ghoul2_model,
    );

    // Point the first view at the entity geometry (Raven vectoangles shape:
    // yaw from x/y, pitch negative when the target is above the eye).
    if let Some(c) = entity_center {
        let d = [c[0] - eye[0], c[1] - eye[1], c[2] - eye[2]];
        let flat = (d[0] * d[0] + d[1] * d[1]).sqrt();
        app.camera.yaw = d[1].atan2(d[0]).to_degrees();
        app.camera.pitch = (-d[2].atan2(flat)).to_degrees();
    }

    let event_loop = EventLoop::new().expect("EventLoop::new: failed to create the event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop
        .run_app(&mut app)
        .expect("run_app: world harness event loop exited with an error");
}
