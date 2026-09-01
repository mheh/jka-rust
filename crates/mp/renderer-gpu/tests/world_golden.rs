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
use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use mp_qshared::shared::mark_fragment::markFragment_t;
use mp_qshared::shared::qhandle_t;
use mp_renderer::render_state::frame_data::FrameData;
use mp_renderer::render_state::frame_event::FrameEvent;
use mp_renderer::render_state::bmodel_table::BModelTable;
use mp_renderer::render_state::render_cvar_snapshot::RenderCvarSnapshot;
use mp_renderer::renderer_frontend::RendererFrontend;
use mp_renderer::tr_cmds::RE_RenderWorldEffects;
use mp_renderer::tr_local::srf_terrain_s::srfTerrain_t;
use mp_renderer::tr_marks::R_MarkFragments;
use mp_renderer::tr_model::render_models::RenderModels;
use mp_renderer::tr_scene::{
    RE_AddDynamicLightToScene, RE_AddPolyToScene, RE_AddRefEntityToScene, RE_RenderScene,
};
use mp_renderer::tr_shader::RE_RegisterShader;
use mp_renderer_gpu::ui_host::boot;
use mp_renderer_gpu::ui_host::{BootConfig, UiHost};
use mp_renderer_gpu::{read_target_rgba, FrameExecutor, Gpu, GpuImages};
use native_math::qmath::{
    AnglesToAxis, CrossProduct, PerpendicularVectorMP, RotatePointAroundVector, VectorNormalize2,
    _DotProduct,
};
use native_math::vector::vec3_t;

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

/// Builds the frozen scene refdef at `eye`, looking along `angles`, through the fixed golden viewport.
fn build_refdef(eye: [f32; 3], angles: [f32; 3]) -> refdef_t {
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
    rd.viewangles = angles;
    AnglesToAxis(angles, rd.viewaxis.as_mut_ptr());

    rd.time = FROZEN_TIME_MS;
    rd.rdflags = 0;
    rd
}

/// Records the frozen scene through the trap-side `RE_RenderScene`.
/// The caller appends its own scene primitives to `frame_data` first, because the render command must sit after them.
fn record_scene(host: &mut UiHost, refdef: &refdef_t, frame_data: &mut FrameData) {
    RE_RenderScene(
        refdef,
        frame_data,
        &host.re.sim.published,
        &host.re.cvars,
        &mut host.re.scene,
        &mut host.engine.common,
        &host.re.sim.light_styles,
    );
}

/// Registers `name` against the loaded world and returns its handle.
///
/// The destructuring matches `boot::load_world`'s own split, which is what lets one `host` hand `RE_RegisterShader` every receiver it takes.
fn register_shader(host: &mut UiHost, name: &str) -> qhandle_t {
    let re_ptr: *mut RendererFrontend = &mut host.re;
    let UiHost {
        engine,
        models,
        re:
            RendererFrontend {
                cvars,
                sim,
                img_state,
                world_load,
                qs,
                sky_view,
                ..
            },
        ..
    } = host;
    let models_ptr: *mut RenderModels = &mut *models;
    let Engine { common, cm, sv, .. } = &mut **engine;
    let sv_ptr: *mut () = sv as *mut Server as *mut ();
    let mut view = boot::host_view(common, cm, sv_ptr, models_ptr, re_ptr);
    RE_RegisterShader(
        name,
        qs,
        world_load,
        Arc::make_mut(&mut sim.published),
        &mut view,
        cvars,
        models,
        img_state,
        sky_view,
    )
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

/// A scene step that appends its own primitives to the frame before the render command.
/// The two plain world fixtures pass `None`, and the marks fixture passes its projection step.
type SceneStep = fn(&mut UiHost, &mut FrameData, [f32; 3]);

/// Renders `map` through the whole chain at the frozen clock and compares the
/// pixels to the committed golden named `stem`. `require_sky_and_fog` adds the
/// two stat gates a fogged open-sky fixture must clear, so an inert sky or fog
/// chain cannot silently bless.
fn run_golden(map: &str, stem: &str, require_sky_and_fog: bool) {
    run_golden_scene(map, stem, require_sky_and_fog, [0.0, 0.0, 0.0], None, false);
}

/// The body behind [`run_golden`], with the three knobs the later fixtures need: the view angles, one scene step, and the dlight stat gate.
#[allow(clippy::too_many_arguments)]
fn run_golden_scene(
    map: &str,
    stem: &str,
    require_sky_and_fog: bool,
    angles: [f32; 3],
    step: Option<SceneStep>,
    require_dlights: bool,
) {
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

    let refdef = build_refdef(eye, angles);
    let mut frame_data = FrameData { events: Vec::new() };
    if let Some(step) = step {
        step(&mut host, &mut frame_data, eye);
    }
    record_scene(&mut host, &refdef, &mut frame_data);

    // ---- headless GPU and the render resources -------------------------
    let mut gpu = Gpu::new_headless(GOLDEN_WIDTH, GOLDEN_HEIGHT);
    let mut images = GpuImages::new(&gpu);
    let mut executor = FrameExecutor::new(&gpu, &images);
    let bmodel_table = BModelTable::build(&host.models);
    let assets = &host.re.sim.published;
    if let Some(world) = assets.world.as_ref() {
        executor.set_world(&gpu, world, &assets.bsp_models, bmodel_table);
    }


    // ---- draw the frame into the offscreen target ----------------------
    let target = gpu.headless_view();
    // The frame executor takes `&mut Gpu`, so the capture source is cloned out here and the borrow on `gpu` ends.
    let target_texture = gpu.headless_texture().clone();
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

        // The world scenes add no entity, so every entity arm sits idle here.
        let stats = executor.execute_frame(
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
        if require_dlights {
            // An inert pass would bless the unlit image, so the counter gates the golden.
            println!("{stem}: {} dlight passes", stats.world.dlight_passes);
            assert!(
                stats.world.dlight_passes > 0,
                "no dlight pass drawn: stats.world = {:?}",
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

/// Raven `MAX_MARK_FRAGMENTS` and `MAX_MARK_POINTS`, the two caps `CG_ImpactMark` passes to the trap.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:107-108`
const MAX_MARK_FRAGMENTS: usize = 128;
const MAX_MARK_POINTS: usize = 384;

/// Raven cgame's `MAX_VERTS_ON_POLY` - the per-fragment vertex cap `CG_ImpactMark` clamps to.
/// This is the cgame value, not the renderer-local one of the same name.
///
/// Source: `oracle/codemp/cgame/cg_local.h:56`
const MAX_VERTS_ON_POLY: usize = 10;

/// The mark's radius in world units, and the drop from the eye to the mark plane.
/// The duel1 spawn eye sits at z 192 and the floor under it at z 128, so a 64-unit drop puts the mark on the floor.
/// `CG_ImpactMark` wants the origin within a unit of the surface it marks, and this is that placement.
const MARK_RADIUS: f32 = 16.0;
const MARK_DROP: f32 = 64.0;

/// The census's top mark shader, at 292,999 poly submissions across the four traces.
const MARK_SHADER: &str = "gfx/damage/rivetmark";

/// Projects one mark straight down under the eye and submits its fragments as scene polygons.
///
/// This is Raven `CG_ImpactMark`'s `temporary` path: build the texture axis, build the four-corner quad, get the fragments, then draw each one.
/// The texture math is Raven's own, `st = 0.5 + DotProduct(delta, axis) * 0.5 / radius`.
/// The step panics when the walk returns no fragment, so an inert walk can never bless an empty image.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:110-211`
fn duel1_floor_mark(host: &mut UiHost, frame_data: &mut FrameData, eye: [f32; 3]) {
    let shader = register_shader(host, MARK_SHADER);
    assert!(shader != 0, "{MARK_SHADER} did not register");

    let origin: vec3_t = [eye[0], eye[1], eye[2] - MARK_DROP];
    let dir: vec3_t = [0.0, 0.0, 1.0];

    // create the texture axis
    let mut axis: [vec3_t; 3] = [[0.0; 3]; 3];
    VectorNormalize2(dir, &mut axis[0]);
    let axis0 = axis[0];
    PerpendicularVectorMP(&mut axis[1], axis0);
    let axis1 = axis[1];
    RotatePointAroundVector(&mut axis[2], axis0, axis1, 0.0);
    let axis2 = axis[2];
    CrossProduct(axis0, axis2, &mut axis[1]);
    let axis1 = axis[1];

    let tex_coord_scale = 0.5 * 1.0 / MARK_RADIUS;

    // create the full polygon
    let mut original_points: [vec3_t; 4] = [[0.0; 3]; 4];
    for i in 0..3usize {
        original_points[0][i] = origin[i] - MARK_RADIUS * axis1[i] - MARK_RADIUS * axis2[i];
        original_points[1][i] = origin[i] + MARK_RADIUS * axis1[i] - MARK_RADIUS * axis2[i];
        original_points[2][i] = origin[i] + MARK_RADIUS * axis1[i] + MARK_RADIUS * axis2[i];
        original_points[3][i] = origin[i] - MARK_RADIUS * axis1[i] + MARK_RADIUS * axis2[i];
    }

    // get the fragments
    let projection: vec3_t = [dir[0] * -20.0, dir[1] * -20.0, dir[2] * -20.0];
    let mut mark_points: Vec<vec3_t> = Vec::new();
    let mut mark_fragments: Vec<markFragment_t> = Vec::new();
    let num_fragments = R_MarkFragments(
        &host.re.sim.published,
        &mut host.re.mark_state,
        &original_points,
        projection,
        MAX_MARK_POINTS,
        &mut mark_points,
        MAX_MARK_FRAGMENTS,
        &mut mark_fragments,
    );
    println!("world_marks_duel1: {num_fragments} fragments under the eye at {origin:?}");
    assert!(
        num_fragments > 0,
        "the mark walk returned no fragment, so the golden would bless an empty floor",
    );

    let colors: [u8; 4] = [255, 255, 255, 255];
    for mf in mark_fragments.iter().take(num_fragments as usize) {
        // we have an upper limit on the complexity of polygons that we store persistantly
        let num_points = (mf.numPoints as usize).min(MAX_VERTS_ON_POLY);
        let verts: Vec<polyVert_t> = (0..num_points)
            .map(|j| {
                let xyz = mark_points[mf.firstPoint as usize + j];
                let delta: vec3_t = [
                    xyz[0] - origin[0],
                    xyz[1] - origin[1],
                    xyz[2] - origin[2],
                ];
                polyVert_t {
                    xyz,
                    st: [
                        0.5 + _DotProduct(delta, axis1) * tex_coord_scale,
                        0.5 + _DotProduct(delta, axis2) * tex_coord_scale,
                    ],
                    modulate: colors,
                }
            })
            .collect();
        RE_AddPolyToScene(
            frame_data,
            &host.re.sim.published,
            &mut host.engine.common,
            shader,
            &verts,
            num_points,
            1,
        );
    }
}

/// The marks fixture: the duel1 room again, with one `gfx/damage/rivetmark` decal on the floor under the eye.
/// The camera looks straight down, so the whole mark sits in frame.
/// This is the gate on the census's `marks/MarkFragments` group, 92,322 trap calls across the four traces.
#[test]
#[ignore = "needs retail assets and a GPU; run locally with --ignored"]
fn golden_world_marks_duel1() {
    run_golden_scene(
        "maps/mp/duel1.bsp",
        "world_marks_duel1",
        false,
        [90.0, 0.0, 0.0],
        Some(duel1_floor_mark),
        false,
    );
}

/// The `#`-prefixed name that registers a second map as a sub-BSP instance world.
/// `RE_RegisterModel` branches on the `#`, loads `maps/mp/duel1.bsp` into `tr.bspModels[0]`, and returns the handle
/// hashed for `*1-0`, that instance's whole submodel 0.
///
/// Source: `oracle/codemp/renderer/tr_model.cpp:1227-1246`
const SUBBSP_INSTANCE_NAME: &str = "#mp/duel1";

/// Where the instance's own bounding box lands, relative to the eye: its near face this far along `+x`, and its
/// centre level with the eye plus this drop.
/// An instance world keeps its own map coordinates, and `mp/duel1` sits nowhere near the `mp/ffa2` spawn, so the
/// entity origin carries the whole offset.
const INSTANCE_NEAR_DIST: f32 = 64.0;
const INSTANCE_DROP: f32 = 0.0;

/// Registers `mp/duel1` as a sub-BSP instance and draws its submodel 0 as one `RT_MODEL` entity in front of the eye.
///
/// This is the `misc_bsp` path a mod server drives. A `misc_bsp` entity registers a `#`-prefixed name as a
/// `CS_BSP_MODELS` configstring, cgame registers every such configstring at init, and the returned `*<k>-0` handle
/// draws as a brush entity.
///
/// Source: `oracle/codemp/game/g_misc.c:416-418`, `oracle/codemp/cgame/cg_main.c:2308-2324`
fn ffa2_subbsp_instance(host: &mut UiHost, frame_data: &mut FrameData, eye: [f32; 3]) {
    let model = boot::register_model(host, SUBBSP_INSTANCE_NAME);
    assert!(
        model > 0,
        "{SUBBSP_INSTANCE_NAME} did not register as a sub-BSP instance",
    );

    // An empty instance world would draw nothing and bless the plain ffa2 image, so the load is gated here.
    let bounds = {
        let instance = host
            .re
            .sim
            .published
            .bsp_models
            .first()
            .expect("the instance world must reach the published registry");
        assert!(
            !instance.surfaces.is_empty(),
            "the instance world loaded no surface",
        );
        println!(
            "world_subbsp_ffa2: instance carries {} surfaces and {} submodels",
            instance.surfaces.len(),
            instance.bmodels.len(),
        );
        instance.bmodels[0].bounds
    };
    println!("world_subbsp_ffa2: instance submodel 0 bounds {bounds:?}");

    let centre = [
        (bounds[0][0] + bounds[1][0]) * 0.5,
        (bounds[0][1] + bounds[1][1]) * 0.5,
        (bounds[0][2] + bounds[1][2]) * 0.5,
    ];

    let mut ent = refEntity_t::zeroed();
    ent.reType = refEntityType_t::RT_MODEL;
    ent.hModel = model;
    ent.origin = [
        eye[0] + INSTANCE_NEAR_DIST - bounds[0][0],
        eye[1] - centre[1],
        eye[2] - INSTANCE_DROP - centre[2],
    ];
    ent.oldorigin = ent.origin;
    ent.shaderRGBA = [255, 255, 255, 255];
    AnglesToAxis([0.0, 0.0, 0.0], ent.axis.as_mut_ptr());
    RE_AddRefEntityToScene(
        frame_data,
        &host.re.sim.published,
        &mut host.re.scene,
        &ent,
        None,
    );
}

/// The sub-BSP fixture: the `mp/ffa2` courtyard with the whole `mp/duel1` map drawn inside it as one instance
/// brush entity.
/// This is the gh#50 gate. The instance's surfaces live past the main world's in the flat index space, so a lost
/// offset draws nothing and blesses the plain ffa2 image instead.
#[test]
#[ignore = "needs retail assets and a GPU; run locally with --ignored"]
fn golden_world_subbsp_ffa2() {
    run_golden_scene(
        "maps/mp/ffa2.bsp",
        "world_subbsp_ffa2",
        true,
        [0.0, 0.0, 0.0],
        Some(ffa2_subbsp_instance),
        false,
    );
}

/// The three lights the dlight fixture adds, each relative to the eye:
/// the offset from the eye, the radius, the color, and whether the light is additive.
/// The camera looks along +x from the duel1 spawn, so every light sits in front of it.
/// The first light drops to the floor the marks fixture already proved is 64 units under the eye.
const DUEL1_DLIGHTS: [([f32; 3], f32, [f32; 3], bool); 3] = [
    ([80.0, 0.0, -56.0], 250.0, [1.0, 0.85, 0.6], false),
    ([200.0, 0.0, 20.0], 250.0, [0.4, 0.6, 1.0], false),
    ([140.0, -80.0, -40.0], 200.0, [1.0, 0.4, 0.2], true),
];

/// Adds the three dynamic lights through the real `RE_AddDynamicLightToScene` trap, before the render command records.
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:326-345`
fn duel1_dlights(host: &mut UiHost, frame_data: &mut FrameData, eye: [f32; 3]) {
    for (offset, radius, color, additive) in DUEL1_DLIGHTS {
        let org: vec3_t = [
            eye[0] + offset[0],
            eye[1] + offset[1],
            eye[2] + offset[2],
        ];
        RE_AddDynamicLightToScene(
            frame_data,
            &host.re.sim.published,
            org,
            radius,
            color[0],
            color[1],
            color[2],
            additive,
        );
    }
}

/// The three effect strings `SP_CreateSnow` registers, in its own order.
/// `ctf2` carries two `fx_snow` entities, and the doubling is inert because a string already registered returns its existing index.
/// Without `constantwind` the global wind velocity is zero and the snow falls straight down, which is not the retail picture.
///
/// Source: `oracle/codemp/game/g_misc.c:2522-2527`
const CTF2_WEATHER_COMMANDS: [&str; 3] = ["snow", "fog", "constantwind (100 100 -100)"];

/// The two fixed generator seeds the fixture pins.
/// `Rng::srand` seeds the C runtime state alone and `Rng::Rand_Init` seeds `holdrand`, and weather draws from both.
/// The `holdrand` stream is live on this path, because the `snow` preset sets `mRotationChangeNext` to zero.
/// The live path keeps Raven's wall-clock seed, `srand(Com_Milliseconds())`, because the reseed is fixture-only.
///
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1491,1811`
const WEATHER_SEED_CRT: u32 = 12345;
const WEATHER_SEED_HOLDRAND: i32 = 6789;

/// The weather fixture's step count and step length in milliseconds.
/// The rig renders one frame, and the particle system needs many updates before the fade reaches its ceiling and the flakes spread out.
/// `RE_RenderScene` derives `frametime` from the delta between calls, so the clock advances by one step per call.
///
/// Source: `crates/mp/renderer/src/tr_scene.rs:1248-1249`
const WEATHER_STEPS: i32 = 60;
const WEATHER_STEP_MS: i32 = 33;

/// Issues one `R_WorldEffectCommand` against the booted host.
///
/// The rig runs no game and no cgame, so it calls the parser directly, the way the marks fixture calls `RE_RegisterShader` directly.
/// The destructuring matches `boot::load_world`'s own split, which is what lets one `host` hand the call every receiver it takes.
fn weather_command(host: &mut UiHost, command: &str) {
    let re_ptr: *mut RendererFrontend = &mut host.re;
    let UiHost {
        engine,
        models,
        re:
            RendererFrontend {
                cvars,
                sim,
                img_state,
                qs,
                world_effects,
                ..
            },
        ..
    } = host;
    let models_ptr: *mut RenderModels = &mut *models;
    let Engine { common, cm, sv, .. } = &mut **engine;
    let sv_ptr: *mut () = sv as *mut Server as *mut ();
    let mut view = boot::host_view(common, cm, sv_ptr, models_ptr, re_ptr);
    world_effects.R_WorldEffectCommand(
        qs,
        &mut view,
        cvars,
        Arc::make_mut(&mut sim.published),
        models,
        img_state,
        Some(command.as_bytes()),
    );
}

/// Steps the weather once for the scene `frame_data` just recorded, and appends the batch event.
///
/// This is the trap arm's own shape: take the refdef `RE_RenderScene` sealed off the last event, then call `RE_RenderWorldEffects`.
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:868`
fn step_weather(host: &mut UiHost, frame_data: &mut FrameData) {
    let scene_refdef = match frame_data.events.last() {
        Some(FrameEvent::RenderScene { refdef, .. }) => Some(refdef.clone()),
        _ => None,
    };
    let Some(refdef) = scene_refdef else {
        panic!("the weather step must follow a recorded scene");
    };

    let re_ptr: *mut RendererFrontend = &mut host.re;
    let UiHost {
        engine,
        models,
        re: RendererFrontend {
            sim, world_effects, ..
        },
        ..
    } = host;
    let models_ptr: *mut RenderModels = &mut *models;
    let Engine { common, cm, sv, .. } = &mut **engine;
    let sv_ptr: *mut () = sv as *mut Server as *mut ();
    let mut view = boot::host_view(common, cm, sv_ptr, models_ptr, re_ptr);
    RE_RenderWorldEffects(
        frame_data,
        world_effects,
        &sim.published,
        &refdef,
        &mut view,
    );
}

/// The weather fixture: the `ctf2` spawn view under the snow and fog `SP_CreateSnow` builds.
///
/// `ctf2` is the one stock MP map that ships weather. Its entity lump carries two `fx_snow` entities and three `misc_weather_zone` brushes.
/// The rig loads no collision world and runs no cgame, so the point cache reads every cell as outside and the zone list falls back to the
/// whole map. Both are rig properties. This golden proves the draw path and byte stability, and it proves nothing about zone or cache behavior.
///
/// Source: `oracle/codemp/game/g_misc.c:2522-2527`, `oracle/codemp/renderer/tr_WorldEffects.cpp:1798,1879`
#[test]
#[ignore = "needs retail assets and a GPU; run locally with --ignored"]
fn golden_world_weather_ctf2() {
    let stem = "world_weather_ctf2";

    // ---- boot and load the world ---------------------------------------
    let mut cfg = BootConfig::default();
    if let Ok(basepath) = std::env::var("JKA_BASEPATH") {
        cfg.basepath = basepath;
    }
    let mut host = boot::boot(&cfg);
    let (loaded, _land_scape): (bool, srfTerrain_t) =
        boot::load_world(&mut host, "maps/mp/ctf2.bsp");
    assert!(loaded, "maps/mp/ctf2.bsp did not load");

    host.re.frame.view_cluster = -1;
    Arc::make_mut(&mut host.re.sim.published).registered = true;

    let eye = host
        .re
        .sim
        .published
        .world
        .as_ref()
        .and_then(|w| boot::find_spawn_origin(&w.entity_string))
        .map(|o| [o[0], o[1], o[2] + EYE_HEIGHT])
        .unwrap_or([0.0, 0.0, 0.0]);

    // ---- pin both generator streams, build the weather, pin them again ---
    // The first pin covers the commands themselves. `CWeatherParticleCloud::Initialize` picks every particle's `mMass` off the
    // C runtime stream, and it runs inside `R_WorldEffectCommand`, so the snow and fog commands take 1060 draws before the
    // second pin. Mass divides the force, so an unpinned draw here gives each particle its own fall rate and the image moves.
    // Source: oracle/codemp/renderer/tr_WorldEffects.cpp:928-935
    host.re.world_effects.rng.srand(WEATHER_SEED_CRT);
    host.re.world_effects.rng.Rand_Init(WEATHER_SEED_HOLDRAND);

    for command in CTF2_WEATHER_COMMANDS {
        weather_command(&mut host, command);
    }
    assert_eq!(
        host.re.world_effects.mParticleClouds.len(),
        2,
        "the snow and fog commands must each build a particle cloud",
    );
    assert_eq!(
        host.re.world_effects.mWindZones.len(),
        1,
        "the constantwind command must build one global wind zone",
    );
    // The second pin keeps the stepped stream independent of how many draws the command path took.
    host.re.world_effects.rng.srand(WEATHER_SEED_CRT);
    host.re.world_effects.rng.Rand_Init(WEATHER_SEED_HOLDRAND);

    // ---- step the weather, keeping the last frame for the draw ----------
    // The first step builds the point cache and draws nothing, which the oracle does too.
    // Source: oracle/codemp/renderer/tr_WorldEffects.cpp:1544-1547
    let mut frame_data = FrameData { events: Vec::new() };
    for step in 0..WEATHER_STEPS {
        let mut refdef = build_refdef(eye, [0.0, 0.0, 0.0]);
        refdef.time = FROZEN_TIME_MS + step * WEATHER_STEP_MS;
        frame_data = FrameData { events: Vec::new() };
        record_scene(&mut host, &refdef, &mut frame_data);
        step_weather(&mut host, &mut frame_data);
    }

    // ---- headless GPU and the render resources -------------------------
    let mut gpu = Gpu::new_headless(GOLDEN_WIDTH, GOLDEN_HEIGHT);
    let mut images = GpuImages::new(&gpu);
    let mut executor = FrameExecutor::new(&gpu, &images);
    let bmodel_table = BModelTable::build(&host.models);
    let assets = &host.re.sim.published;
    if let Some(world) = assets.world.as_ref() {
        executor.set_world(&gpu, world, &assets.bsp_models, bmodel_table);
    }

    let target = gpu.headless_view();
    let target_texture = gpu.headless_texture().clone();
    gpu.clear_headless(&target);
    let float_time = FROZEN_TIME_MS as f32 * 0.001;

    let _uploaded = images.upload_pending(&mut gpu, &mut host.re.img_state, &host.re.sim.published);

    {
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

        let stats = executor.execute_frame(
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
        );

        println!(
            "{stem}: {} weather vertices over {} world draw calls",
            stats.world.weather_vertices, stats.world.draw_calls,
        );
        assert!(
            stats.world.surfaces_drawn > 0,
            "no world surface drawn: stats.world = {:?}",
            stats.world,
        );
        // An empty batch would bless the plain ctf2 room, so the counter gates the golden.
        assert!(
            stats.world.weather_vertices > 0,
            "no weather vertex drawn: stats.world = {:?}",
            stats.world,
        );
    }

    // ---- read the pixels back ------------------------------------------
    let (width, height, actual) = read_target_rgba(&gpu);
    assert_eq!(width, GOLDEN_WIDTH);
    assert_eq!(height, GOLDEN_HEIGHT);

    let golden = golden_path(stem);
    if std::env::var("JKA_GOLDEN_BLESS").as_deref() == Ok("1") {
        write_png(&golden, width, height, &actual);
        println!(
            "{stem}: blessed {} ({} bytes)",
            golden.display(),
            std::fs::metadata(&golden).map(|m| m.len()).unwrap_or(0),
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

/// The dlight fixture: the duel1 room again, with three dynamic lights in front of the camera.
/// This is the gate on the census's `dlight/calls` group, 112,514 submissions across the four traces.
/// The `dlight_passes` counter must be nonzero, so an inert pass can never bless an unlit image.
#[test]
#[ignore = "needs retail assets and a GPU; run locally with --ignored"]
fn golden_world_dlights_duel1() {
    run_golden_scene(
        "maps/mp/duel1.bsp",
        "world_dlights_duel1",
        false,
        [0.0, 0.0, 0.0],
        Some(duel1_dlights),
        true,
    );
}
