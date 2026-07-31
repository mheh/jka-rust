//! R4 image-golden gate: render a fixed world scene headless and compare the
//! pixels to a committed PNG.
//!
//! The world draw runs the whole render chain (`RE_RenderScene` record ->
//! `FrameExecutor::execute_frame` -> `R_RenderView` -> sorted world surfaces),
//! the same chain `bin/world_harness` drives in a window, but into an offscreen
//! target instead of a surface. The test then reads the pixels back and
//! compares them to `tests/goldens/world_duel1.png`.
//!
//! The test is `#[ignore]`d, matching the demo-replay rig: it needs the retail
//! assets and a GPU, so it runs locally, not in CI. Run it with
//! `cargo test -p mp_renderer_gpu --test world_golden -- --ignored`.
//!
//! Bless flow: set `JKA_GOLDEN_BLESS=1` to write the golden and pass. On a
//! mismatch without that env var, the test writes the actual image next to the
//! golden as `world_duel1.actual.png` and fails.

use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;
use std::path::PathBuf;

use mp_engine_core::Engine;
use mp_engine_ghoul2::ghoul2_system::Ghoul2System;
use mp_engine_qcommon::cm_terrain::CmLandScape;
use mp_engine_server::Server;
use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use mp_renderer::render_state::frame_data::FrameData;
use mp_renderer::tr_local::dlight_s::dlight_t;
use mp_renderer::tr_local::fog_t::fog_t;
use mp_renderer::tr_local::srf_terrain_s::srfTerrain_t;
use mp_renderer::tr_main::TrMainScratch;
use mp_renderer::tr_model::render_models::RenderModels;
use mp_renderer::tr_scene::RE_RenderScene;
use mp_renderer_gpu::ui_host::boot;
use mp_renderer_gpu::ui_host::{BootConfig, UiHost};
use mp_renderer_gpu::{read_target_rgba, FrameExecutor, Gpu, GpuImages, WorldFrame};
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
        &host.assets,
        &host.cvars,
        &mut host.scene,
        &mut host.engine.common,
        &host.sim.light_styles,
    );
    frame_data
}

/// The absolute path of the committed golden.
fn golden_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/goldens/world_duel1.png")
}

/// The absolute path the actual image lands at on a mismatch.
fn actual_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/goldens/world_duel1.actual.png")
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

#[test]
#[ignore = "needs retail assets and a GPU; run locally with --ignored"]
fn golden_world_duel1() {
    // ---- boot and load the world ---------------------------------------
    // The default basepath points at one user's home. Read `JKA_BASEPATH` so
    // another machine can re-bless the golden without editing the default.
    let mut cfg = BootConfig::default();
    if let Ok(basepath) = std::env::var("JKA_BASEPATH") {
        cfg.basepath = basepath;
    }
    let mut host = boot::boot(&cfg);
    let (loaded, land_scape): (bool, srfTerrain_t) =
        boot::load_world(&mut host, "maps/mp/duel1.bsp");
    assert!(loaded, "duel1.bsp did not load");

    // Force the first `R_MarkLeaves` to re-mark, and set the registered flag
    // the ui boot path never sets, the same two settings `world_harness` makes.
    host.frame.view_cluster = -1;
    host.assets.registered = true;

    // The camera sits at a spawn origin, bumped to eye height.
    let eye = host
        .assets
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
    if let Some(world) = host.assets.world.as_ref() {
        executor.set_world(&gpu, world);
    }

    let dummy_assets = boot::empty_assets();
    let land = CmLandScape::empty();
    let mut dlights: Vec<dlight_t> = Vec::new();
    let fogs: Vec<fog_t> = Vec::new();
    let mut scratch = TrMainScratch {
        pre_trans_ent_matrix: [0.0; 16],
    };

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
    let _uploaded = images.upload_pending(&mut gpu, &mut host.img_state, &host.sim.published);

    {
        // Split the host and engine into disjoint borrows, the shape
        // `world_harness::draw_world_frame` builds.
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
        } = &mut host;
        let models_ptr: *mut RenderModels = &mut *models;
        let Engine { common, cm, sv, .. } = &mut **engine;
        let sv_ptr: *mut () = sv as *mut Server as *mut ();
        let mut engine_view = boot::host_view(common, cm, sv_ptr, models_ptr);

        // The golden test has no live Ghoul2 state, so it threads an empty
        // owned system (design point 2).
        let mut g2_system = Ghoul2System::default();
        let mut world = WorldFrame {
            engine_view: &mut engine_view,
            assets,
            cvars,
            frame: fstate,
            g2: &mut g2_system,
            gpu_res,
            models: &*models,
            land_scape: &land_scape,
            land: &land,
            dlights: dlights.as_mut_slice(),
            fogs: fogs.as_slice(),
            scratch: &mut scratch,
        };

        let stats = executor.execute_frame(
            &mut gpu,
            &target,
            &frame_data,
            &dummy_assets,
            &dummy_assets,
            img_state,
            &mut images,
            font,
            noise,
            float_time,
            Some(&mut world),
        );

        // The chain must draw the world, or a blank render blesses as the
        // golden. `world_spike` reports 57 drawSurfs for this scene.
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
    }

    // ---- read the pixels back ------------------------------------------
    let (width, height, actual) = read_target_rgba(&gpu);
    assert_eq!(width, GOLDEN_WIDTH);
    assert_eq!(height, GOLDEN_HEIGHT);

    let golden = golden_path();

    // Bless: write the golden and pass.
    if std::env::var("JKA_GOLDEN_BLESS").as_deref() == Ok("1") {
        write_png(&golden, width, height, &actual);
        println!(
            "golden_world_duel1: blessed {} ({} bytes)",
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
        let actual_out = actual_path();
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
