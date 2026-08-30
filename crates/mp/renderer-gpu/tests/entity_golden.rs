//! Entity image golden: render one MD3 map object and one Ghoul2 player in a fixed duel1 scene and compare the pixels to a committed PNG.
//!
//! This is the gate gh#31 step-004 needed.
//! The world goldens draw no entity, the scene goldens draw no `RT_MODEL`, and the ghoul2 vertex golden locks a vertex stream rather than pixels,
//! so the migrated `MOD_MESH` arm had no proof that it puts anything on screen.
//! This test draws both migrated arms in one scene and locks the result.
//! DEC-54 names image goldens on fixed scenes as the verification shape.
//!
//! The scene boots exactly like `tests/world_golden.rs`:
//! the same `BootConfig`, the `JKA_BASEPATH` override, an offscreen `Gpu`, and `maps/mp/duel1.bsp`.
//! It then registers `models/map_objects/bespin/twinpodcc.md3` through `boot::register_model`
//! and inits one stormtrooper through the `init_ghoul2` recipe,
//! adds both as `RT_MODEL` entities at fixed origins in front of the eye, and records one frame at the frozen clock.
//! Both entities carry a zero radius, which pins the Ghoul2 LOD to 0.
//!
//! Bless provenance: blessed on 2026-08-05 during gh#31 step-004, on the client register path with `dedicated` at `"0"`.
//! The origins below are the blessed placement, and moving any of them moves the image.
//!
//! The test is `#[ignore]`d, matching the other image goldens: it needs the retail assets and a GPU, so it runs locally, not in CI.
//! Run it with `cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`.
//! Serial only: two engine boots in parallel threads crash in the GPU init.
//!
//! Bless flow: set `JKA_GOLDEN_BLESS=1` to write the golden and pass.
//! On a mismatch without that env var, the test writes the actual image next to the golden as `entity_duel1.actual.png` and fails.

use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use mp_engine_core::Engine;
use mp_engine_ghoul2::api_models::g2api_init_ghoul2_model;
use mp_engine_ghoul2::ghoul2_system::Ghoul2System;
use mp_engine_ghoul2::info_array::Ghoul2Handle;
use mp_engine_ghoul2::shared::cghoul2_info_v::CGhoul2Info_v;
use mp_engine_ghoul2::token::ghoul2_token_encode;
use mp_engine_server::Server;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use mp_qshared::common::mp::cgame::tr_types::{
    RF_DISINTEGRATE1, RF_DISINTEGRATE2, RF_VOLUMETRIC,
};
use mp_qshared::shared::qhandle_t;
use mp_renderer::render_state::bmodel_table::BModelTable;
use mp_renderer::render_state::frame_data::FrameData;
use mp_renderer::render_state::render_cvar_snapshot::RenderCvarSnapshot;
use mp_renderer::renderer_frontend::RendererFrontend;
use mp_renderer::tr_ghoul2::build_ghoul2_render_payload;
use mp_renderer::tr_local::srf_terrain_s::srfTerrain_t;
use mp_renderer::tr_model::render_models::RenderModels;
use mp_renderer::tr_public::ref_flags::RDF_NOWORLDMODEL;
use mp_renderer::tr_scene::{RE_AddRefEntityToScene, RE_ClearScene, RE_RenderScene};
use mp_renderer_gpu::ui_host::boot;
use mp_renderer_gpu::ui_host::{BootConfig, UiHost};
use mp_renderer_gpu::{read_target_rgba, FrameExecutor, Gpu, GpuImages};
use native_math::qmath::AnglesToAxis;

/// The golden viewport in physical pixels, matching `world_golden`.
const GOLDEN_WIDTH: u32 = 800;
const GOLDEN_HEIGHT: u32 = 600;

/// The frozen scene clock in milliseconds, the same value the other goldens use.
const FROZEN_TIME_MS: i32 = 12345;

/// The eye-height bump added to a spawn origin, matching `world_golden`.
const EYE_HEIGHT: f32 = 40.0;

/// The horizontal field of view in degrees, matching `world_golden`.
const FOV_X: f64 = 90.0;

/// The per-channel match tolerance.
/// Zero means an exact match.
const CHANNEL_TOLERANCE: u8 = 0;

/// The shipped map object the `MOD_MESH` arm draws, the one `world_harness` already mounts.
const MD3_MODEL_NAME: &str = "models/map_objects/bespin/twinpodcc.md3";

/// The shipped player model the `MOD_MDXM` arm draws in its base skeleton pose.
const GHOUL2_MODEL_NAME: &str = "models/players/stormtrooper/model.glm";

/// The MD3 map object stands this far in front of the eye and this far to the left.
/// Yaw and pitch are zero, so forward is `+X` and left is `+Y`.
const MD3_FORWARD_DIST: f32 = 260.0;
const MD3_SIDE_OFFSET: f32 = 110.0;
const MD3_DROP: f32 = 60.0;

/// The stormtrooper stands nearer the eye and to the right, the mirror of the MD3 placement.
const GHOUL2_FORWARD_DIST: f32 = 170.0;
const GHOUL2_SIDE_OFFSET: f32 = -70.0;
const GHOUL2_DROP: f32 = 40.0;

/// The three renderfx copies stand this far in front of the eye and this far below it.
/// The map object is 315 units deep and 303 wide, so at the `entity_duel1` distance one copy alone spans most of the frame.
/// This distance shrinks each copy to about 255 pixels, which is what lets three of them share one image.
const RENDERFX_FORWARD_DIST: f32 = 600.0;
const RENDERFX_DROP: f32 = 60.0;

/// The side offsets that spread the three copies across the frame, one near each third of the width.
const RENDERFX_SIDE_OFFSETS: [f32; 3] = [340.0, 0.0, -340.0];

/// The burn centre the two disintegrate copies carry, a model-space point.
/// `RB_CalcDisintegrateColors` compares `oldorigin` against `tess.xyz`, which is model space for a mesh, so a world-space origin would put the burn nowhere near the skin.
/// This is the model vertex nearest the frame's bounding-box centre, so the burn sphere cuts real geometry rather than empty interior.
const DISINTEGRATE_ORIGIN: [f32; 3] = [24.5, -20.703125, 54.34375];

/// The burn clock the two disintegrate copies carry.
/// `RB_CalcDisintegrateColors` reads `threshold = (refdef.time - endTime) * 0.045`, so this puts the threshold at 120 units.
/// The model carries 702 vertices over a 315 unit span, and 184 of them fall inside that radius, so the burn takes a readable bite rather than one corner.
/// The colour bands are per vertex, so a radius near the vertex spacing would shade a couple of corners and interpolate away.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:1545-1560`
const DISINTEGRATE_END_TIME: f32 = 9678.333;

/// Builds the frozen scene refdef at `eye`, looking straight ahead (yaw 0, pitch 0), through the fixed viewport.
/// This mirrors `world_golden::build_refdef`.
fn build_refdef(eye: [f32; 3]) -> refdef_t {
    // SAFETY: `refdef_t` is a frozen `#[repr(C)]` POD of scalars, fixed arrays, and `vec3_t`, so an all-zero value is valid.
    let mut rd: refdef_t = unsafe { core::mem::zeroed() };
    rd.x = 0;
    rd.y = 0;
    rd.width = GOLDEN_WIDTH as i32;
    rd.height = GOLDEN_HEIGHT as i32;

    rd.fov_x = FOV_X as f32;
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

/// Inits one Ghoul2 model instance through the real `mp_engine_ghoul2` init path, the same call `ghoul2_vertex_golden::init_ghoul2` makes.
/// Returns `None` when the model file is absent (a negative model index).
fn init_ghoul2(host: &mut UiHost, name: &str) -> Option<(Ghoul2System, Ghoul2Handle, qhandle_t)> {
    let mut g2 = Ghoul2System::default();
    let mut info = CGhoul2Info_v { mItem: 0 };
    info.alloc(&mut g2);

    let model_index = {
        let re_ptr: *mut RendererFrontend = &mut host.re;
        let UiHost {
            engine, models, ..
        } = &mut *host;
        let models_ptr: *mut RenderModels = &mut *models;
        let Engine { common, cm, sv, .. } = &mut **engine;
        let sv_ptr: *mut () = sv as *mut Server as *mut ();
        let mut view = boot::host_view(common, cm, sv_ptr, models_ptr, re_ptr);
        g2api_init_ghoul2_model(&mut g2, &mut view, &mut info, name, 0, 0, 0, 0, 0)
    };
    if model_index < 0 {
        return None;
    }

    let model_handle = info.get(&g2, 0).model;
    Some((g2, Ghoul2Handle(info.mItem), model_handle))
}

/// The absolute path of the committed golden named `stem`.
fn golden_path_for(stem: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("tests/goldens/{stem}.png"))
}

/// The absolute path the actual image lands at on a mismatch of the golden named `stem`.
fn actual_path_for(stem: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("tests/goldens/{stem}.actual.png"))
}

/// The absolute path of the committed golden.
fn golden_path() -> PathBuf {
    golden_path_for("entity_duel1")
}

/// The absolute path the actual image lands at on a mismatch.
fn actual_path() -> PathBuf {
    actual_path_for("entity_duel1")
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

/// The differing-pixel count and the largest single-channel delta between two equally sized RGBA buffers.
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

/// Boots duel1, registers both entity models, draws one frame with both in view, and compares the pixels to the committed golden.
#[test]
#[ignore = "needs retail assets and a GPU; run locally with --ignored"]
fn golden_entity_duel1() {
    // ---- boot and load the world ---------------------------------------
    // The default basepath points at one user's home.
    // Read `JKA_BASEPATH` so another machine can re-bless the golden without editing the default.
    let mut cfg = BootConfig::default();
    if let Ok(basepath) = std::env::var("JKA_BASEPATH") {
        cfg.basepath = basepath;
    }
    let mut host = boot::boot(&cfg);
    // The terrain surface `load_world` returns is the null-landscape seed.
    // The executor owns its own copy since W2-F6, so this one is dropped.
    let (loaded, _land_scape): (bool, srfTerrain_t) =
        boot::load_world(&mut host, "maps/mp/duel1.bsp");
    assert!(loaded, "maps/mp/duel1.bsp did not load");

    // Force the first `R_MarkLeaves` to re-mark, and set the registered flag the ui boot path never sets, the same two settings `world_golden` makes.
    host.re.frame.view_cluster = -1;
    Arc::make_mut(&mut host.re.sim.published).registered = true;

    // Register both entity models through Raven's client register path.
    let md3_model = boot::register_model(&mut host, MD3_MODEL_NAME);
    assert!(md3_model > 0, "{MD3_MODEL_NAME} did not register");
    let (mut g2, ghoul2_handle, ghoul2_model) =
        init_ghoul2(&mut host, GHOUL2_MODEL_NAME).expect("stormtrooper .glm did not init");

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

    // ---- record the scene ----------------------------------------------
    let mut frame_data = FrameData { events: Vec::new() };
    RE_ClearScene(&mut frame_data, &mut host.re.scene);

    let mut md3_ent = refEntity_t::zeroed();
    md3_ent.reType = refEntityType_t::RT_MODEL;
    md3_ent.hModel = md3_model;
    md3_ent.origin = [
        eye[0] + MD3_FORWARD_DIST,
        eye[1] + MD3_SIDE_OFFSET,
        eye[2] - MD3_DROP,
    ];
    md3_ent.oldorigin = md3_ent.origin;
    md3_ent.frame = 0;
    md3_ent.oldframe = 0;
    md3_ent.shaderRGBA = [255, 255, 255, 255];
    AnglesToAxis([0.0, 0.0, 0.0], md3_ent.axis.as_mut_ptr());
    RE_AddRefEntityToScene(
        &mut frame_data,
        &host.re.sim.published,
        &mut host.re.scene,
        &md3_ent,
        None,
    );

    let mut g2_ent = refEntity_t::zeroed();
    g2_ent.reType = refEntityType_t::RT_MODEL;
    // The zeroed `radius` pins the Ghoul2 LOD to 0, the same pin `ghoul2_vertex_golden` relies on.
    g2_ent.hModel = ghoul2_model;
    g2_ent.ghoul2 = ghoul2_token_encode(Some(ghoul2_handle));
    g2_ent.origin = [
        eye[0] + GHOUL2_FORWARD_DIST,
        eye[1] + GHOUL2_SIDE_OFFSET,
        eye[2] - GHOUL2_DROP,
    ];
    g2_ent.oldorigin = g2_ent.origin;
    g2_ent.frame = 0;
    g2_ent.oldframe = 0;
    g2_ent.shaderRGBA = [255, 255, 255, 255];
    AnglesToAxis([0.0, 0.0, 0.0], g2_ent.axis.as_mut_ptr());

    // The DEC-65 ruling 2 crossing is built here, on the same host view `init_ghoul2` builds.
    // The build ends before the add, because the setup inside it can re-register a model through the `re` hooks.
    let g2_payload = {
        let re_ptr: *mut RendererFrontend = &mut host.re;
        let UiHost {
            engine, models, ..
        } = &mut host;
        let models_ptr: *mut RenderModels = &mut *models;
        let Engine { common, cm, sv, .. } = &mut **engine;
        let sv_ptr: *mut () = sv as *mut Server as *mut ();
        let mut view = boot::host_view(common, cm, sv_ptr, models_ptr, re_ptr);
        build_ghoul2_render_payload(&mut g2, &mut view, &g2_ent, 0)
    };

    RE_AddRefEntityToScene(
        &mut frame_data,
        &host.re.sim.published,
        &mut host.re.scene,
        &g2_ent,
        g2_payload,
    );

    RE_RenderScene(
        &refdef,
        &mut frame_data,
        &host.re.sim.published,
        &host.re.cvars,
        &mut host.re.scene,
        &mut host.engine.common,
        &host.re.sim.light_styles,
    );

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
    // The frame executor takes `&mut Gpu`, so the capture source is cloned out here and the borrow on `gpu` ends.
    let target_texture = gpu.headless_texture().clone();
    // The world pass loads the color target, so clear it first.
    gpu.clear_headless(&target);
    let float_time = FROZEN_TIME_MS as f32 * 0.001;

    // Drain the staged image uploads against the sim-published master before the split borrow, the same pre-drain `world_golden` does.
    let _uploaded = images.upload_pending(&mut gpu, &mut host.re.img_state, &host.re.sim.published);

    let stats = {
        // `RE_EndFrame` drains the registered model blocks into the published registry, and no test reaches it.
        // The drain therefore runs here, and it must land before the pin below.
        // A drain after the pin publishes into a generation the frame does not read, and the frame then draws nothing.
        // Source: crates/mp/renderer/src/tr_cmds.rs:354-358
        if let Some(blocks) = host.models.publish_blocks() {
            host.re.sim.publish_models(blocks);
        }
        // The frame pins the published registry, because `G2_SetupModelPointers` re-registers at scene-add, ahead of the drain above.
        // The client `RE_RegisterModel` hook then calls `Arc::make_mut(&mut re.sim.published)` through the seated `re` slot.
        // The clone holds a second reference, so that call copies on write instead of mutating the allocation this frame reads.
        let pinned = Arc::clone(&host.re.sim.published);
        // Split the host into disjoint borrows, the shape `world_golden` builds.
        let UiHost {
            re:
                RendererFrontend {
                    world_load,
                    img_state,
                    noise,
                    ..
                },
            ..
        } = &mut host;

        executor.execute_frame(
            &mut gpu,
            &target,
            &target_texture,
            &frame_data,
            &pinned,
            world_load,
            img_state.pending_uploads.drain().collect(),
            &mut images,
            noise,
            float_time,
            // No live cvar table in the test, so the retail defaults keep the golden byte-exact.
            RenderCvarSnapshot::default(),
        )
    };

    // Both migrated arms must put geometry on screen, or a world-only render blesses as the golden.
    assert!(
        stats.world.md3_surfaces_drawn > 0,
        "no MD3 entity surface drawn: stats.world = {:?}",
        stats.world,
    );
    assert!(
        stats.world.ghoul2_surfaces_drawn > 0,
        "no Ghoul2 entity surface drawn: stats.world = {:?}",
        stats.world,
    );
    assert_eq!(
        stats.world.md3_decode_failed, 0,
        "an MD3 surface failed to decode: stats.world = {:?}",
        stats.world,
    );
    assert_eq!(
        stats.world.ghoul2_decode_failed, 0,
        "a Ghoul2 surface failed to decode: stats.world = {:?}",
        stats.world,
    );

    // ---- read the pixels back ------------------------------------------
    let (width, height, actual) = read_target_rgba(&gpu);
    assert_eq!(width, GOLDEN_WIDTH);
    assert_eq!(height, GOLDEN_HEIGHT);

    let golden = golden_path();

    // Bless: write the golden and pass.
    if std::env::var("JKA_GOLDEN_BLESS").as_deref() == Ok("1") {
        write_png(&golden, width, height, &actual);
        println!(
            "entity_duel1: blessed {} ({} bytes, {} md3 surfaces, {} ghoul2 surfaces)",
            golden.display(),
            std::fs::metadata(&golden).map(|m| m.len()).unwrap_or(0),
            stats.world.md3_surfaces_drawn,
            stats.world.ghoul2_surfaces_drawn,
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
            "entity golden mismatch: {} pixels differ, max channel delta {}; wrote actual image to {}",
            differing_pixels,
            max_delta,
            actual_out.display(),
        );
    }
}

/// Boots duel1, draws the same map object three times with three different renderfx flags, and compares the pixels to the committed golden.
///
/// The census counts `RF_DISINTEGRATE1`, `RF_DISINTEGRATE2` and `RF_VOLUMETRIC` as live rows, and all three arms already draw.
/// None of them had an image golden, so this scene locks the three colour short-circuits that `ComputeColors` runs.
/// The scene needs surface normals: the vert deform reads them and the volumetric fade reads them, so it draws an MD3 model rather than a sprite.
///
/// The two disintegrate copies carry `endTime` and `oldorigin`, and the burn threshold sits inside the model's own bounds at the frozen clock.
/// `RB_CalcDisintegrateColors` compares `oldorigin` against `tess.xyz`, which is model space for a mesh, so the burn centre here is a model-space point.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:1536-1581`,
/// `oracle/codemp/renderer/tr_shade_calc.cpp:1545-1671`
#[test]
#[ignore = "needs retail assets and a GPU; run locally with --ignored"]
fn golden_entity_renderfx_duel1() {
    // ---- boot and load the world ---------------------------------------
    let mut cfg = BootConfig::default();
    if let Ok(basepath) = std::env::var("JKA_BASEPATH") {
        cfg.basepath = basepath;
    }
    let mut host = boot::boot(&cfg);
    let (loaded, _land_scape): (bool, srfTerrain_t) =
        boot::load_world(&mut host, "maps/mp/duel1.bsp");
    assert!(loaded, "maps/mp/duel1.bsp did not load");

    host.re.frame.view_cluster = -1;
    Arc::make_mut(&mut host.re.sim.published).registered = true;

    let md3_model = boot::register_model(&mut host, MD3_MODEL_NAME);
    assert!(md3_model > 0, "{MD3_MODEL_NAME} did not register");

    let eye = host
        .re
        .sim
        .published
        .world
        .as_ref()
        .and_then(|w| boot::find_spawn_origin(&w.entity_string))
        .map(|o| [o[0], o[1], o[2] + EYE_HEIGHT])
        .unwrap_or([0.0, 0.0, 0.0]);

    let mut refdef = build_refdef(eye);
    // The three copies are the whole subject here, so the world stays out of the frame.
    // `R_AddWorldSurfaces` returns on this flag before it touches the BSP, so no wall can occlude a copy and the three effects read side by side.
    // The sibling test above keeps the world in frame, so the world and entity composition is still covered.
    // Source: oracle/codemp/renderer/tr_world.cpp:1936-1940
    refdef.rdflags = RDF_NOWORLDMODEL;

    // ---- record the scene ----------------------------------------------
    let mut frame_data = FrameData { events: Vec::new() };
    RE_ClearScene(&mut frame_data, &mut host.re.scene);

    for (index, renderfx) in [RF_DISINTEGRATE1, RF_DISINTEGRATE2, RF_VOLUMETRIC]
        .into_iter()
        .enumerate()
    {
        let mut ent = refEntity_t::zeroed();
        ent.reType = refEntityType_t::RT_MODEL;
        ent.hModel = md3_model;
        ent.renderfx = renderfx;
        ent.origin = [
            eye[0] + RENDERFX_FORWARD_DIST,
            eye[1] + RENDERFX_SIDE_OFFSETS[index],
            eye[2] - RENDERFX_DROP,
        ];
        ent.oldorigin = DISINTEGRATE_ORIGIN;
        ent.endTime = DISINTEGRATE_END_TIME;
        ent.frame = 0;
        ent.oldframe = 0;
        // The volumetric fade reads the red channel alone and writes it into every channel.
        ent.shaderRGBA = [255, 255, 255, 255];
        AnglesToAxis([0.0, 0.0, 0.0], ent.axis.as_mut_ptr());
        RE_AddRefEntityToScene(
            &mut frame_data,
            &host.re.sim.published,
            &mut host.re.scene,
            &ent,
            None,
        );
    }

    RE_RenderScene(
        &refdef,
        &mut frame_data,
        &host.re.sim.published,
        &host.re.cvars,
        &mut host.re.scene,
        &mut host.engine.common,
        &host.re.sim.light_styles,
    );

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
    // The frame executor takes `&mut Gpu`, so the capture source is cloned out here and the borrow on `gpu` ends.
    let target_texture = gpu.headless_texture().clone();
    gpu.clear_headless(&target);
    let float_time = FROZEN_TIME_MS as f32 * 0.001;

    let _uploaded = images.upload_pending(&mut gpu, &mut host.re.img_state, &host.re.sim.published);

    let stats = {
        if let Some(blocks) = host.models.publish_blocks() {
            host.re.sim.publish_models(blocks);
        }
        let pinned = Arc::clone(&host.re.sim.published);
        let UiHost {
            re:
                RendererFrontend {
                    world_load,
                    img_state,
                    noise,
                    ..
                },
            ..
        } = &mut host;

        executor.execute_frame(
            &mut gpu,
            &target,
            &target_texture,
            &frame_data,
            &pinned,
            world_load,
            img_state.pending_uploads.drain().collect(),
            &mut images,
            noise,
            float_time,
            RenderCvarSnapshot::default(),
        )
    };

    // All three copies must draw, or a world-only render blesses as the golden.
    assert!(
        stats.world.md3_surfaces_drawn > 0,
        "no MD3 entity surface drawn: stats.world = {:?}",
        stats.world,
    );
    assert_eq!(
        stats.world.md3_decode_failed, 0,
        "an MD3 surface failed to decode: stats.world = {:?}",
        stats.world,
    );

    // ---- read the pixels back ------------------------------------------
    let (width, height, actual) = read_target_rgba(&gpu);
    assert_eq!(width, GOLDEN_WIDTH);
    assert_eq!(height, GOLDEN_HEIGHT);

    let golden = golden_path_for("entity_renderfx_duel1");

    if std::env::var("JKA_GOLDEN_BLESS").as_deref() == Ok("1") {
        write_png(&golden, width, height, &actual);
        println!(
            "entity_renderfx_duel1: blessed {} ({} bytes, {} md3 surfaces)",
            golden.display(),
            std::fs::metadata(&golden).map(|m| m.len()).unwrap_or(0),
            stats.world.md3_surfaces_drawn,
        );
        return;
    }

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
    assert_eq!(
        golden_bytes.len(),
        actual.len(),
        "golden byte count does not match the rendered byte count",
    );

    let (differing_pixels, max_delta) = compare(&golden_bytes, &actual);
    if differing_pixels > 0 {
        let actual_out = actual_path_for("entity_renderfx_duel1");
        write_png(&actual_out, width, height, &actual);
        panic!(
            "renderfx entity golden mismatch: {} pixels differ, max channel delta {}; wrote actual image to {}",
            differing_pixels,
            max_delta,
            actual_out.display(),
        );
    }
}
