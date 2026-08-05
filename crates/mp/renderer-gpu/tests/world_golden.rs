//! R4 image-golden gate: render a fixed world scene headless and compare the
//! pixels to a committed PNG.
//!
//! The world draw runs the whole render chain (`RE_RenderScene` record ->
//! `FrameExecutor::execute_frame` -> `R_RenderView` -> sorted world surfaces),
//! the same chain `bin/world_harness` drives in a window, but into an offscreen
//! target instead of a surface. Each test reads the pixels back and compares
//! them to its committed PNG under `tests/goldens/`. Two fixtures run: the
//! enclosed duel1 room, and the fogged open-sky ffa2 courtyard.
//!
//! The tests are `#[ignore]`d, matching the demo-replay rig: they need the
//! retail assets and a GPU, so they run locally, not in CI. Run them with
//! `cargo test -p mp_renderer_gpu --test world_golden -- --ignored
//! --test-threads=1`. Serial only: two engine boots in parallel threads crash
//! in the GPU init.
//!
//! Bless flow: set `JKA_GOLDEN_BLESS=1` to write the golden and pass. On a
//! mismatch without that env var, the test writes the actual image next to the
//! golden as `world_duel1.actual.png` and fails.

use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::io::BufWriter;
use std::path::Path;
use std::path::PathBuf;

use mp_engine_core::Engine;
use mp_engine_server::Server;
use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use mp_renderer::render_state::frame_data::FrameData;
use mp_renderer::render_state::bmodel_table::BModelTable;
use mp_renderer::render_state::render_cvar_snapshot::RenderCvarSnapshot;
use mp_renderer::renderer_frontend::RendererFrontend;
use mp_renderer::tr_local::srf_terrain_s::srfTerrain_t;
use mp_renderer::tr_model::render_models::RenderModels;
use mp_renderer::tr_scene::RE_RenderScene;
use mp_renderer_gpu::ui_host::boot;
use mp_renderer_gpu::ui_host::{BootConfig, UiHost};
use mp_renderer_gpu::{read_target_rgba, FrameExecutor, Gpu, GpuImages};
use native_math::qmath::AnglesToAxis;

/// The golden viewport in physical pixels. Fixed so the projection and the
/// read-back image never depend on a window size.
const GOLDEN_WIDTH: u32 = 800;
const GOLDEN_HEIGHT: u32 = 600;

/// The frozen scene clock in milliseconds. Every animated stage reads the
/// shader clock, so the clock must be a constant for the image to be
/// deterministic. `RE_RenderScene` derives `floatTime = time * 0.001`, so this
/// gives `floatTime = 12.345`.
const FROZEN_TIME_MS: i32 = 12345;

/// The eye-height bump added to a spawn origin, matching `world_harness`.
const EYE_HEIGHT: f32 = 40.0;

/// The per-channel match tolerance. Zero means an exact match. A future
/// cross-driver run can widen this if the same scene renders one bit apart on
/// another GPU.
const CHANNEL_TOLERANCE: u8 = 0;

/// The horizontal field of view in degrees, matching `world_harness`.
const FOV_X: f64 = 90.0;

/// Builds the frozen scene refdef at `eye`, looking straight ahead (yaw 0,
/// pitch 0), through the fixed golden viewport.
fn build_refdef(eye: [f32; 3]) -> refdef_t {
    // SAFETY: `refdef_t` is a frozen `#[repr(C)]` POD of scalars, fixed arrays,
    // and `vec3_t`, so an all-zero value is valid.
    let mut rd: refdef_t = unsafe { core::mem::zeroed() };
    rd.x = 0;
    rd.y = 0;
    rd.width = GOLDEN_WIDTH as i32;
    rd.height = GOLDEN_HEIGHT as i32;

    rd.fov_x = FOV_X as f32;
    // `x = width / tan(fov_x / 360 * PI); fov_y = atan2(height, x) * 360 / PI`.
    let x = (GOLDEN_WIDTH as f64) / (FOV_X / 360.0 * std::f64::consts::PI).tan();
    let fov_y = (GOLDEN_HEIGHT as f64).atan2(x) * 360.0 / std::f64::consts::PI;
    rd.fov_y = fov_y as f32;

    rd.vieworg = eye;
    let angles = [0.0f32, 0.0, 0.0];
    rd.viewangles = angles;
    AnglesToAxis(angles, rd.viewaxis.as_mut_ptr());

    rd.time = FROZEN_TIME_MS;
    rd.rdflags = 0;
    rd
}

/// Records the frozen scene through the trap-side `RE_RenderScene`.
fn record_scene(host: &mut UiHost, refdef: &refdef_t) -> FrameData {
    let mut frame_data = FrameData { events: Vec::new() };
    RE_RenderScene(
        refdef,
        &mut frame_data,
        &host.re.sim.published,
        &host.re.cvars,
        &mut host.re.scene,
        &mut host.engine.common,
        &host.re.sim.light_styles,
    );
    frame_data
}

/// The absolute path of the committed golden for `stem`.
fn golden_path(stem: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("tests/goldens/{stem}.png"))
}

/// The absolute path the actual image lands at on a mismatch.
fn actual_path(stem: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("tests/goldens/{stem}.actual.png"))
}

/// Writes RGBA8 pixels to `path` as an 8-bit PNG.
fn write_png(path: &Path, width: u32, height: u32, rgba: &[u8]) {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).expect("create_dir_all: golden directory");
    }
    let file = File::create(path).expect("create: golden PNG file");
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut png_writer = encoder.write_header().expect("write_header: golden PNG");
    png_writer
        .write_image_data(rgba)
        .expect("write_image_data: golden PNG");
}

/// Reads an 8-bit RGBA PNG back into width, height, and packed bytes.
fn read_png(path: &Path) -> (u32, u32, Vec<u8>) {
    let file = File::open(path).expect("open: golden PNG file");
    let decoder = png::Decoder::new(BufReader::new(file));
    let mut reader = decoder.read_info().expect("read_info: golden PNG");
    let mut buf = vec![0u8; reader.output_buffer_size().expect("output_buffer_size")];
    let info = reader.next_frame(&mut buf).expect("next_frame: golden PNG");
    buf.truncate(info.buffer_size());
    (info.width, info.height, buf)
}

/// The differing-pixel count and the largest single-channel delta between two
/// equally sized RGBA buffers.
fn compare(golden: &[u8], actual: &[u8]) -> (usize, u8) {
    let mut differing_pixels = 0usize;
    let mut max_delta = 0u8;
    for (g_pixel, a_pixel) in golden.chunks_exact(4).zip(actual.chunks_exact(4)) {
        let mut pixel_differs = false;
        for channel in 0..4 {
            let delta = g_pixel[channel].abs_diff(a_pixel[channel]);
            if delta > max_delta {
                max_delta = delta;
            }
            if delta > CHANNEL_TOLERANCE {
                pixel_differs = true;
            }
        }
        if pixel_differs {
            differing_pixels += 1;
        }
    }
    (differing_pixels, max_delta)
}

/// Renders `map` through the whole chain at the frozen clock and compares the
/// pixels to the committed golden named `stem`. `require_sky_and_fog` adds the
/// two stat gates a fogged open-sky fixture must clear, so an inert sky or fog
/// chain cannot silently bless.
fn run_golden(map: &str, stem: &str, require_sky_and_fog: bool) {
    // ---- boot and load the world ---------------------------------------
    // The default basepath points at one user's home. Read `JKA_BASEPATH` so
    // another machine can re-bless the golden without editing the default.
    let mut cfg = BootConfig::default();
    if let Ok(basepath) = std::env::var("JKA_BASEPATH") {
        cfg.basepath = basepath;
    }
    let mut host = boot::boot(&cfg);
    // The terrain surface `load_world` returns is the null-landscape seed. The
    // executor owns its own copy since W2-F6, so this one is dropped.
    let (loaded, _land_scape): (bool, srfTerrain_t) = boot::load_world(&mut host, map);
    assert!(loaded, "{map} did not load");

    // Force the first `R_MarkLeaves` to re-mark, and set the registered flag
    // the ui boot path never sets, the same two settings `world_harness` makes.
    host.re.frame.view_cluster = -1;
    Arc::make_mut(&mut host.re.sim.published).registered = true;

    // The camera sits at a spawn origin, bumped to eye height.
    let eye = host
        .re
        .sim
        .published
        .world
        .as_ref()
        .and_then(|w| boot::find_spawn_origin(&w.entity_string))
        .map(|o| [o[0], o[1], o[2] + EYE_HEIGHT])
        .unwrap_or([0.0, 0.0, 0.0]);

    let refdef = build_refdef(eye);
    let frame_data = record_scene(&mut host, &refdef);

    // ---- headless GPU and the render resources -------------------------
    let mut gpu = Gpu::new_headless(GOLDEN_WIDTH, GOLDEN_HEIGHT);
    let mut images = GpuImages::new(&gpu);
    let mut executor = FrameExecutor::new(&gpu, &images);
    let bmodel_table = BModelTable::build(&host.models);
    if let Some(world) = host.re.sim.published.world.as_ref() {
        executor.set_world(&gpu, world, bmodel_table);
    }


    // ---- draw the frame into the offscreen target ----------------------
    let target = gpu.headless_view();
    // The world pass loads the color target, so clear it first. Otherwise the
    // golden captures wgpu zero-init in every uncovered pixel, not CLEAR_COLOR.
    gpu.clear_headless(&target);
    let float_time = FROZEN_TIME_MS as f32 * 0.001;

    // Drain the staged image uploads against the sim-published master before
    // the split borrow, the same pre-drain `world_harness` does. A drain inside
    // `execute_frame` would resolve against the dummy registry and drop every
    // staged world texture.
    let _uploaded = images.upload_pending(&mut gpu, &mut host.re.img_state, &host.re.sim.published);

    {
        // `RE_EndFrame` drains the registered model blocks into the published registry, and no test reaches it.
        // The drain therefore runs here, and it must land before the pin below.
        // A drain after the pin publishes into a generation the frame does not read, and the frame then draws nothing.
        // Source: crates/mp/renderer/src/tr_cmds.rs:354-358
        if let Some(blocks) = host.models.publish_blocks() {
            host.re.sim.publish_models(blocks);
        }
        // The frame pins the published registry, so a mid-frame `Arc::make_mut` through the seated `re` slot copies on write.
        // This map draws no ghoul2 entity, so no register hook fires here, and the pin keeps every entity-walk site one shape.
        let pinned = Arc::clone(&host.re.sim.published);
        // Split the host and engine into disjoint borrows, the shape
        // `world_harness::draw_world_frame` builds.
        let re_ptr: *mut RendererFrontend = &mut host.re;
        let UiHost {
            engine,
            models,
            re:
                RendererFrontend {
                    world_load,
                    img_state,
                    noise,
                    ..
                },
            ..
        } = &mut host;
        let models_ptr: *mut RenderModels = &mut *models;
        let Engine { common, cm, sv, .. } = &mut **engine;
        let sv_ptr: *mut () = sv as *mut Server as *mut ();
        let mut engine_view = boot::host_view(common, cm, sv_ptr, models_ptr, re_ptr);

        // The golden test has no live Ghoul2 state, and the executor's own empty system is what the world pass uses (W2-F5).
        // A sim-side caller hands the entity walk the engine host, so the Ghoul2 arms run too.
        let stats = executor.execute_frame(
            &mut gpu,
            &target,
            &frame_data,
            &pinned,
            world_load,
            Some(&mut engine_view),
            img_state.pending_uploads.drain().collect(),
            &mut images,
            noise,
            float_time,
            // No live cvar table in the test, so the retail defaults keep the
            // golden byte-exact.
            RenderCvarSnapshot::default(),
        );

        // The chain must draw the world, or a blank render blesses as the
        // golden. `world_spike` reports 57 drawSurfs for the duel1 scene.
        assert!(
            stats.world.surfaces_drawn > 0,
            "no world surface drawn: stats.world = {:?}",
            stats.world,
        );
        assert!(
            stats.world.draw_calls > 0,
            "no world draw call issued: stats.world = {:?}",
            stats.world,
        );
        if require_sky_and_fog {
            assert!(
                stats.world.sky_surfaces_drawn > 0,
                "no sky surface drawn: stats.world = {:?}",
                stats.world,
            );
            assert!(
                stats.world.fog_passes_drawn > 0,
                "no fog pass drawn: stats.world = {:?}",
                stats.world,
            );
        }
    }

    // ---- read the pixels back ------------------------------------------
    let (width, height, actual) = read_target_rgba(&gpu);
    assert_eq!(width, GOLDEN_WIDTH);
    assert_eq!(height, GOLDEN_HEIGHT);

    let golden = golden_path(stem);

    // Bless: write the golden and pass.
    if std::env::var("JKA_GOLDEN_BLESS").as_deref() == Ok("1") {
        write_png(&golden, width, height, &actual);
        println!(
            "{stem}: blessed {} ({} bytes)",
            golden.display(),
            std::fs::metadata(&golden).map(|m| m.len()).unwrap_or(0),
        );
        return;
    }

    // Compare against the committed golden.
    assert!(
        golden.exists(),
        "golden missing at {}; run once with JKA_GOLDEN_BLESS=1 to write it",
        golden.display(),
    );
    let (gw, gh, golden_bytes) = read_png(&golden);
    assert_eq!(
        (gw, gh),
        (width, height),
        "golden size does not match the rendered size",
    );
    // `read_png` returns the file's own color type. A golden that is not 8-bit
    // RGBA has a different byte count, so `compare` would truncate to the short
    // side. Assert equal lengths so a format fault fails loud.
    assert_eq!(
        golden_bytes.len(),
        actual.len(),
        "golden byte count does not match the rendered byte count",
    );

    let (differing_pixels, max_delta) = compare(&golden_bytes, &actual);
    if differing_pixels > 0 {
        let actual_out = actual_path(stem);
        write_png(&actual_out, width, height, &actual);
        panic!(
            "world golden mismatch: {} pixels differ, max channel delta {}; \
             wrote actual image to {}",
            differing_pixels,
            max_delta,
            actual_out.display(),
        );
    }
}

#[test]
#[ignore = "needs retail assets and a GPU; run locally with --ignored"]
fn golden_world_duel1() {
    run_golden("maps/mp/duel1.bsp", "world_duel1", false);
}

/// The fogged open-sky fixture: ffa2 carries two brush fogs, one global fog,
/// and a visible sky from the spawn view, so this golden covers the sky and
/// fog chains the enclosed duel1 view never exercises.
#[test]
#[ignore = "needs retail assets and a GPU; run locally with --ignored"]
fn golden_world_ffa2() {
    run_golden("maps/mp/ffa2.bsp", "world_ffa2", true);
}
