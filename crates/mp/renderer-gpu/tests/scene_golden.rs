//! The fixed-dt image-golden gate for the DEC-54 census surface.
//!
//! Each scene here is synthetic. The host boots against a temp game tree this
//! file writes, so the only assets in play are the images `R_Init` builds
//! procedurally and one generated shader script that binds `$whiteimage`. No
//! retail content is read, and no golden here derives from retail content. That
//! is what separates this gate from `world_golden.rs`, which loads a real map
//! and stays `#[ignore]`d.
//!
//! **Determinism.** Every scene fixes the three inputs a frame varies on: the
//! viewport is a constant, the shader clock is a constant
//! ([`FROZEN_TIME_MS`], the fixed-dt seam DEC-58.1 names), and the camera is a
//! literal. `RE_RenderScene` derives `floatTime` from the refdef time alone, so
//! two runs of the same scene submit identical geometry.
//!
//! **Backend caveat.** Rasterisation is the GPU's. A golden blessed on one
//! adapter can differ by a channel step on another, so [`CHANNEL_TOLERANCE`]
//! exists as the knob to widen; it is zero today because every run so far is on
//! one adapter. A machine with no adapter at all skips its scenes rather than
//! failing them, so `cargo test --workspace` stays green on a headless box.
//!
//! Bless flow: `JKA_GOLDEN_BLESS=1` writes the golden and passes. A mismatch
//! writes `<stem>.actual.png` beside the golden and fails.

use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use mp_engine_core::Engine;
use mp_engine_qcommon::common::error::ComError;
use mp_engine_qcommon::files_common::FS_ProductIdFile;
use mp_engine_server::Server;
use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use mp_qshared::common::mp::cgame::tr_types::{RF_DEPTHHACK, RF_FORCE_ENT_ALPHA, RF_RGB_TINT};
use mp_qshared::shared::qhandle_t;
use mp_renderer::render_state::frame_data::FrameData;
use mp_renderer::render_state::render_cvar_snapshot::RenderCvarSnapshot;
use mp_renderer::renderer_frontend::RendererFrontend;
use mp_renderer::tr_model::render_models::RenderModels;
use mp_renderer::tr_public::ref_flags::{RDF_DRAWSKYBOX, RDF_NOWORLDMODEL};
use mp_renderer::tr_scene::{
    RE_AddDynamicLightToScene, RE_AddPolyToScene, RE_AddRefEntityToScene, RE_RenderScene,
};
use mp_renderer::tr_shader::RE_RegisterShader;
use mp_renderer_gpu::ui_host::boot;
use mp_renderer_gpu::ui_host::{BootConfig, UiHost};
use mp_renderer_gpu::{read_target_rgba, FrameExecutor, Gpu, GpuImages};
use native_math::qmath::AnglesToAxis;

/// The golden viewport in physical pixels. Fixed so the projection and the
/// read-back image never depend on a window size.
const GOLDEN_WIDTH: u32 = 320;
const GOLDEN_HEIGHT: u32 = 240;

/// The frozen scene clock in milliseconds - the fixed-dt seam on the com clock
/// (DEC-58.1). `RE_RenderScene` derives `floatTime = time * 0.001`, so this
/// gives `floatTime = 12.345` on every run.
const FROZEN_TIME_MS: i32 = 12345;

/// The per-channel match tolerance. Zero means an exact match. Widen this if
/// the same scene ever renders a step apart on a second adapter.
const CHANNEL_TOLERANCE: u8 = 0;

/// The horizontal field of view in degrees.
const FOV_X: f64 = 90.0;

/// One synthetic scene: a name for its golden, the eye point, and the recorder
/// that submits its contents through the trap seam.
struct Scene {
    /// The golden's file stem under `tests/goldens/`.
    stem: &'static str,
    /// The eye point, looking down +x.
    eye: [f32; 3],
    /// Submits the scene's contents against the registered shader handle. Every
    /// scene records through the real `RE_Add*ToScene` traps, so the gate
    /// exercises the same seam a module frame does.
    record: fn(host: &mut UiHost, frame_data: &mut FrameData, shader: qhandle_t),
    /// How many dynamic lights the scene expects to reach `tr.refdef.dlights`.
    /// A no-world scene has no dlight-receiving surface, so this counter is what
    /// holds the replay chain in place.
    expect_dlights: u32,
}

/// Records `entities` through `RE_AddRefEntityToScene`, the shape most scenes
/// here use.
fn record_entities(host: &mut UiHost, frame_data: &mut FrameData, entities: &[refEntity_t]) {
    for ent in entities {
        RE_AddRefEntityToScene(frame_data, &host.re.sim.published, &mut host.re.scene, ent, None);
    }
}

/// A zeroed `refEntity_t` with the identity axis, the shape a caller fills in.
///
/// The oracle's callers `memset` the struct and set the fields their `reType`
/// reads, so an all-zero start matches the submission shape the census counted.
fn base_ref_entity() -> refEntity_t {
    // SAFETY: `refEntity_t` is a frozen `#[repr(C)]` POD of scalars, fixed
    // arrays, `vec3_t`, and a union of the same, so an all-zero value is valid.
    let mut re: refEntity_t = unsafe { core::mem::zeroed() };
    re.axis = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    re.shaderRGBA = [255, 255, 255, 255];
    re
}

/// Builds the frozen scene refdef at `eye`, looking down +x through the fixed
/// golden viewport.
///
/// `RDF_NOWORLDMODEL` is what makes the scene synthetic: `R_AddWorldSurfaces`
/// returns on that flag before it touches a BSP, so the frame draws entity
/// surfaces only. `RDF_DRAWSKYBOX` rides along because every scene the census
/// counted carried it.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:1936-1940`
fn build_refdef(eye: [f32; 3]) -> refdef_t {
    // SAFETY: `refdef_t` is a frozen `#[repr(C)]` POD of scalars, fixed arrays,
    // and `vec3_t`, so an all-zero value is valid.
    let mut rd: refdef_t = unsafe { core::mem::zeroed() };
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
    rd.rdflags = RDF_NOWORLDMODEL | RDF_DRAWSKYBOX;
    rd
}

/// The shader script the synthetic game tree carries. `$whiteimage` binds
/// `tr.whiteImage`, one of the images `R_CreateBuiltinImages` generates, so no
/// texture file is needed anywhere in this gate.
///
/// `rgbGen vertex`/`alphaGen vertex` is what makes an entity's `shaderRGBA`
/// visible: every generated entity surface writes that colour into its
/// vertices, the same way the census's saber line and glow shaders read it.
const SYNTHETIC_SHADER_SCRIPT: &str = "\
gfx/golden/vertex
{
\t{
\t\tmap $whiteimage
\t\tblendFunc GL_SRC_ALPHA GL_ONE_MINUS_SRC_ALPHA
\t\trgbGen vertex
\t\talphaGen vertex
\t}
}

gfx/golden/constant
{
\t{
\t\tmap $whiteimage
\t\tblendFunc GL_SRC_ALPHA GL_ONE_MINUS_SRC_ALPHA
\t\trgbGen const ( 0.2 0.35 0.9 )
\t\talphaGen const 1.0
\t}
}

gfx/golden/opaque
{
\t{
\t\tmap $whiteimage
\t\trgbGen vertex
\t}
}
";

/// The per-call counter [`write_atomic`] puts in its temporary file names.
static TEMP_SERIAL: AtomicU64 = AtomicU64::new(0);

/// Writes `data` to `path` through a uniquely named temporary file and a
/// rename, so a concurrent reader never sees a half-written file.
fn write_atomic(path: &Path, data: &[u8]) {
    // The tests share one process, so the temporary name needs a per-call
    // counter, not just the pid.
    let serial = TEMP_SERIAL.fetch_add(1, Ordering::Relaxed);
    let temp = path.with_extension(format!("{}.{serial}.tmp", std::process::id()));
    std::fs::write(&temp, data).expect("write: synthetic fixture");
    std::fs::rename(&temp, path).expect("rename: synthetic fixture");
}

/// Builds the synthetic game tree in a temp directory and boots a
/// renderer-only host against it. Nothing retail is reachable: `fs_basepath`
/// and `fs_homepath` both point here, and the only files present are the three
/// this function writes.
fn boot_synthetic() -> UiHost {
    // The directory name carries the fixture content's hash. A tree left by an
    // older build of this file is then simply unused, instead of feeding a
    // stale shader script to a fresh run - which renders the wrong image and
    // blesses it.
    let mut hasher = DefaultHasher::new();
    SYNTHETIC_SHADER_SCRIPT.hash(&mut hasher);
    let basepath =
        std::env::temp_dir().join(format!("jka-rust-scene-golden-{:016x}", hasher.finish()));
    let base = basepath.join("base");
    std::fs::create_dir_all(base.join("shaders")).expect("create_dir_all: synthetic basepath");

    // Every file lands atomically: the tests run in parallel and all build the
    // same tree, so a plain write lets one test read another's half-written
    // file. A rename is atomic, and the content is identical either way.
    write_atomic(&base.join("mpdefault.cfg"), b"// synthetic scene golden\n");
    // Without `productid.txt`, `FS_SetRestrictions` drops the filesystem to
    // demo mode, where loose directories are never scanned and the shader
    // script below would be invisible.
    // Source: oracle/codemp/qcommon/files_pc.cpp:2587-2637
    write_atomic(&base.join("productid.txt"), &FS_ProductIdFile());
    // `ScanAndLoadShaderFiles` is a fatal error when it finds no `.shader`
    // file, so the tree needs at least this one.
    // Source: oracle/codemp/renderer/tr_shader.cpp:3895-3900
    write_atomic(
        &base.join("shaders/synthetic.shader"),
        SYNTHETIC_SHADER_SCRIPT.as_bytes(),
    );

    let cfg = BootConfig {
        basepath: basepath.to_string_lossy().into_owned(),
        homepath: basepath.to_string_lossy().into_owned(),
        fs_game: String::new(),
        menu_file: String::new(),
        start_menu: String::new(),
    };
    // `Com_Error` panics with a typed `ComError` payload the default hook
    // prints as `Box<dyn Any>`. Report the real message, or a boot fault here
    // reads as a bare panic.
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Some(err) = info.payload().downcast_ref::<ComError>() {
            eprintln!("Com_Error: {}", err.msg);
        }
        previous(info);
    }));

    let mut host = boot::boot_renderer(&cfg);
    // The ui boot path is what normally sets this; every `RE_Add*ToScene` trap
    // drops its submission while it is false.
    Arc::make_mut(&mut host.re.sim.published).registered = true;
    host
}

/// Registers `name` and returns its handle. With no shader scripts and no
/// images on disk, every name resolves to the procedurally built default
/// shader, which is exactly the asset-free surface this gate wants.
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

/// How many pixels differ from the image's top-left one. Every scene here
/// leaves its corners at the clear colour, so a zero here means a blank render.
/// That is the trap a golden must clear before it may bless: without it an
/// inert draw path blesses an empty frame and the gate passes forever.
fn coverage(rgba: &[u8]) -> usize {
    let Some(clear) = rgba.get(0..4) else {
        return 0;
    };
    rgba.chunks_exact(4).filter(|p| *p != clear).count()
}

/// Renders `scene` at the frozen clock and compares the pixels to its committed
/// golden.
fn run_scene(scene: &Scene) {
    let Some(mut gpu) = Gpu::try_new_headless(GOLDEN_WIDTH, GOLDEN_HEIGHT) else {
        println!("{}: no GPU adapter, scene skipped", scene.stem);
        return;
    };

    let mut host = boot_synthetic();
    let shader = register_shader(&mut host, "gfx/golden/vertex");

    // ---- record the scene through the trap seam ------------------------
    let refdef = build_refdef(scene.eye);
    let mut frame_data = FrameData { events: Vec::new() };
    (scene.record)(&mut host, &mut frame_data, shader);
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
    let mut images = GpuImages::new(&gpu);
    let mut executor = FrameExecutor::new(&gpu, &images);

    // W2-F6 homes the null-landscape seed on the executor, so this test only
    // runs the terrain init for its cvar registrations.
    let _land_scape = boot::init_terrain(&mut host);

    let target = gpu.headless_view();
    gpu.clear_headless(&target);
    let float_time = FROZEN_TIME_MS as f32 * 0.001;

    let _uploaded = images.upload_pending(&mut gpu, &mut host.re.img_state, &host.re.sim.published);

    let stats = {
        // `RE_EndFrame` drains the registered model blocks into the published registry, and no test reaches it.
        // The drain therefore runs here, and it must land before the pin below.
        // A drain after the pin publishes into a generation the frame does not read, and the frame then draws nothing.
        // Source: crates/mp/renderer/src/tr_cmds.rs:354-358
        if let Some(blocks) = host.models.publish_blocks() {
            host.re.sim.publish_models(blocks);
        }
        // The frame pins the published registry, so a mid-frame `Arc::make_mut` through the seated `re` slot copies on write.
        // This scene draws no ghoul2 entity, so no register hook fires here, and the pin keeps every entity-walk site one shape.
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

        // No scene here carries a Ghoul2 token, so every entity crosses with no payload.
        executor.execute_frame(
            &mut gpu,
            &target,
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

    // ---- read the pixels back ------------------------------------------
    let (width, height, actual) = read_target_rgba(&gpu);
    assert_eq!(width, GOLDEN_WIDTH);
    assert_eq!(height, GOLDEN_HEIGHT);

    // A blank render must never bless as a golden, so both the draw count and
    // the pixels have to show work.
    assert!(
        stats.world.draw_calls > 0,
        "{}: no world draw call issued - stats = {:?}",
        scene.stem,
        stats,
    );
    let covered = coverage(&actual);
    assert!(
        covered > 0,
        "{}: nothing drawn - stats = {:?}",
        scene.stem,
        stats,
    );
    assert_eq!(
        stats.dlights, scene.expect_dlights,
        "{}: dynamic lights did not reach tr.refdef.dlights",
        scene.stem,
    );

    let golden = golden_path(scene.stem);
    if std::env::var("JKA_GOLDEN_BLESS").as_deref() == Ok("1") {
        write_png(&golden, width, height, &actual);
        println!(
            "{}: blessed {} ({} covered pixels, stats {:?})",
            scene.stem,
            golden.display(),
            covered,
            stats,
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
        let actual_out = actual_path(scene.stem);
        write_png(&actual_out, width, height, &actual);
        panic!(
            "{} golden mismatch: {} pixels differ, max channel delta {}; wrote actual image to {}",
            scene.stem,
            differing_pixels,
            max_delta,
            actual_out.display(),
        );
    }
}

/// Census row `refent/RT_SPRITE` (28,749 submissions): three view-oriented
/// quads, one unrotated and two at fixed rotations, so the rotation branch of
/// `RB_SurfaceSprite` is covered alongside the plain one.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:141-169`
fn scene_sprites(host: &mut UiHost, frame_data: &mut FrameData, shader: qhandle_t) {
    let mut out = Vec::new();
    for (index, (y, rotation, rgba)) in [
        (-40.0f32, 0.0f32, [255u8, 255, 255, 255]),
        (0.0, 30.0, [255, 64, 64, 255]),
        (40.0, 45.0, [64, 128, 255, 255]),
    ]
    .into_iter()
    .enumerate()
    {
        let mut re = base_ref_entity();
        re.reType = refEntityType_t::RT_SPRITE;
        re.customShader = shader;
        re.origin = [200.0, y, index as f32 * 4.0];
        re.radius = 16.0;
        re.rotation = rotation;
        re.shaderRGBA = rgba;
        out.push(re);
    }
    record_entities(host, frame_data, &out);
}

/// Census row `refent/RT_LINE` (94,346 submissions): three camera-facing
/// quads spanning `origin` to `oldorigin`, at three widths and three angles, so
/// the cross-product side vector is exercised off-axis as well as on.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:667-690`
fn scene_lines(host: &mut UiHost, frame_data: &mut FrameData, shader: qhandle_t) {
    let mut out = Vec::new();
    for (start, end, radius, rgba) in [
        (
            [200.0f32, -60.0, -40.0],
            [200.0f32, -60.0, 40.0],
            2.0f32,
            [255u8, 255, 255, 255],
        ),
        (
            [200.0, 0.0, -40.0],
            [220.0, 20.0, 40.0],
            4.0,
            [64, 255, 96, 255],
        ),
        (
            [180.0, 40.0, 30.0],
            [220.0, 70.0, -30.0],
            6.0,
            [255, 160, 32, 255],
        ),
    ] {
        let mut re = base_ref_entity();
        re.reType = refEntityType_t::RT_LINE;
        re.customShader = shader;
        re.origin = start;
        re.oldorigin = end;
        re.radius = radius;
        re.shaderRGBA = rgba;
        out.push(re);
    }
    record_entities(host, frame_data, &out);
}

/// Census row `refent/RT_SABER_GLOW` (94,346 submissions): two blades of
/// different lengths and start radii, so the widening sprite run and the hilt
/// blob both draw.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:560-580`
fn scene_saber_glow(host: &mut UiHost, frame_data: &mut FrameData, shader: qhandle_t) {
    let mut out = Vec::new();
    for (origin, blade_axis, length, radius, rgba) in [
        (
            [220.0f32, -40.0, -30.0],
            [0.0f32, 0.0, 1.0],
            60.0f32,
            3.0f32,
            [64u8, 96, 255, 255],
        ),
        (
            [200.0, 40.0, -20.0],
            [0.0, 0.4472136, 0.8944272],
            40.0,
            5.0,
            [255, 48, 48, 255],
        ),
    ] {
        let mut re = base_ref_entity();
        re.reType = refEntityType_t::RT_SABER_GLOW;
        re.customShader = shader;
        re.origin = origin;
        // `axis[0]` is the blade direction the sprite run steps along.
        re.axis[0] = blade_axis;
        re.saberLength = length;
        re.radius = radius;
        re.shaderRGBA = rgba;
        out.push(re);
    }
    record_entities(host, frame_data, &out);
}

/// Census rows `poly/calls` (443,515) and `marks/MarkFragments` (92,322): four
/// scene polygons through `RE_AddPolyToScene`, a triangle, a quad, a pentagon,
/// and a hexagon, so the fan triangulation is covered past its three-vertex
/// base. Per-vertex `modulate` differs across the corners, which is what a
/// faded mark carries.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:220-254`
/// (`RB_SurfacePolychain`)
fn scene_polys(host: &mut UiHost, frame_data: &mut FrameData, shader: qhandle_t) {
    for (index, corner_count) in [3usize, 4, 5, 6].into_iter().enumerate() {
        // A regular polygon in the plane x = 210, centred on its own row.
        let center_z = 45.0 - index as f32 * 30.0;
        let radius = 13.0f32;
        let verts: Vec<polyVert_t> = (0..corner_count)
            .map(|corner| {
                let angle = std::f32::consts::TAU * corner as f32 / corner_count as f32;
                // The corners fade from opaque to a quarter alpha around the
                // ring, so the per-vertex modulate is visible in the image.
                let fade = 255 - (corner * 192 / corner_count) as u8;
                polyVert_t {
                    xyz: [210.0, radius * angle.cos(), center_z + radius * angle.sin()],
                    st: [
                        0.5 + 0.5 * angle.cos(),
                        0.5 + 0.5 * angle.sin(),
                    ],
                    modulate: [255, fade, 64, 255],
                }
            })
            .collect();

        RE_AddPolyToScene(
            frame_data,
            &host.re.sim.published,
            &mut host.engine.common,
            shader,
            &verts,
            corner_count,
            1,
        );
    }
}

/// Census row `dlight/calls` (112,514 submissions): three dynamic lights beside
/// the sprite trio, one of them additive.
///
/// The dlight passes are live, and they light world and brush surfaces only.
/// This scene is `RDF_NOWORLDMODEL`, so no surface here carries a dlight mask
/// and the golden stays the sprite image. That makes this fixture the leak
/// guard: a moved pixel means the pass reached an entity surface.
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:326-345`
fn scene_dlights(host: &mut UiHost, frame_data: &mut FrameData, shader: qhandle_t) {
    scene_sprites(host, frame_data, shader);
    for (org, intensity, color, additive) in [
        ([200.0f32, -40.0, 0.0], 60.0f32, [1.0f32, 0.9, 0.7], false),
        ([200.0, 0.0, 0.0], 40.0, [0.4, 0.6, 1.0], false),
        ([200.0, 40.0, 0.0], 25.0, [1.0, 0.3, 0.2], true),
    ] {
        RE_AddDynamicLightToScene(
            frame_data,
            &host.re.sim.published,
            org,
            intensity,
            color[0],
            color[1],
            color[2],
            additive,
        );
    }
}

/// Census renderfx rows `RF_RGB_TINT` (8,679) and `RF_FORCE_ENT_ALPHA` (6,586):
/// four sprites drawn through a shader whose rgbGen is a fixed constant, so the
/// only way a corner shows the entity colour is the tint override, and the only
/// way it shows a partial alpha is the force override.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:2049-2053,2190-2202`
fn scene_renderfx_tint(host: &mut UiHost, frame_data: &mut FrameData, _shader: qhandle_t) {
    let constant = register_shader(host, "gfx/golden/constant");
    let mut out = Vec::new();
    for (index, (z, renderfx, rgba)) in [
        (45.0f32, 0i32, [255u8, 64, 64, 255]),
        (15.0, RF_RGB_TINT, [255, 64, 64, 255]),
        (-15.0, RF_FORCE_ENT_ALPHA, [255, 64, 64, 96]),
        (-45.0, RF_RGB_TINT | RF_FORCE_ENT_ALPHA, [64, 255, 96, 128]),
    ]
    .into_iter()
    .enumerate()
    {
        let mut re = base_ref_entity();
        re.reType = refEntityType_t::RT_SPRITE;
        re.customShader = constant;
        re.renderfx = renderfx;
        re.origin = [200.0, 0.0, z];
        re.radius = 14.0;
        re.shaderRGBA = rgba;
        let _ = index;
        out.push(re);
    }
    record_entities(host, frame_data, &out);
}

/// Census renderfx row `RF_DEPTHHACK` (188): a wall quad with a sprite behind
/// it, drawn twice. The plain sprite fails the depth test and stays hidden; the
/// hacked one is squeezed into the front 30 per cent of the depth window, so it
/// passes and draws over the wall. That is exactly what the flag is for, to keep
/// a view model out of the geometry it would otherwise poke into.
///
/// The scene uses the opaque shader: an alpha-blended stage writes no depth, so
/// nothing would occlude anything and the flag would leave no trace.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:930-938,957-973`
fn scene_depthhack(host: &mut UiHost, frame_data: &mut FrameData, _shader: qhandle_t) {
    let opaque = register_shader(host, "gfx/golden/opaque");
    let mut out = Vec::new();
    for (index, renderfx) in [0i32, RF_DEPTHHACK].into_iter().enumerate() {
        let y = 30.0 - index as f32 * 60.0;

        // The wall draws first and writes depth.
        let mut wall = base_ref_entity();
        wall.reType = refEntityType_t::RT_SPRITE;
        wall.customShader = opaque;
        wall.origin = [200.0, y, 0.0];
        wall.radius = 26.0;
        wall.shaderRGBA = [64, 200, 255, 255];
        out.push(wall);

        // The sprite sits behind it, so only the depth hack can show it.
        let mut behind = base_ref_entity();
        behind.reType = refEntityType_t::RT_SPRITE;
        behind.customShader = opaque;
        behind.renderfx = renderfx;
        behind.origin = [400.0, y, 0.0];
        behind.radius = 30.0;
        behind.shaderRGBA = [255, 128, 32, 255];
        out.push(behind);
    }
    record_entities(host, frame_data, &out);
}

/// The FX module's `RT_ORIENTED_QUAD` submissions, which the trap census never saw because `COrientedParticle::Draw` builds the entity inside the engine.
/// Three quads at one radius, each on its own orthonormal `axis[1]`/`axis[2]` pair and its own rotation, so the arm reads the entity's axis rather than the view's.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:177-220`
fn scene_fx_oriented_quad(host: &mut UiHost, frame_data: &mut FrameData, shader: qhandle_t) {
    let mut out = Vec::new();
    for (y, left, up, rotation, rgba) in [
        (
            -40.0f32,
            [0.0f32, 1.0, 0.0],
            [0.0f32, 0.0, 1.0],
            0.0f32,
            [255u8, 255, 255, 255],
        ),
        (
            0.0,
            [0.0, 0.70710678, 0.70710678],
            [0.0, -0.70710678, 0.70710678],
            30.0,
            [255, 64, 64, 255],
        ),
        (
            40.0,
            [0.4472136, 0.8944272, 0.0],
            [0.0, 0.0, 1.0],
            45.0,
            [64, 128, 255, 255],
        ),
    ] {
        let mut re = base_ref_entity();
        re.reType = refEntityType_t::RT_ORIENTED_QUAD;
        re.customShader = shader;
        re.origin = [200.0, y, 0.0];
        // The arm spans the quad from these two rows, so each entity carries its own orientation.
        re.axis[1] = left;
        re.axis[2] = up;
        re.radius = 16.0;
        re.rotation = rotation;
        re.shaderRGBA = rgba;
        out.push(re);
    }
    record_entities(host, frame_data, &out);
}

/// The FX module's `RT_CYLINDER` submissions, which the trap census never saw because `CCylinder::Draw` builds the entity inside the engine.
/// One straight tube and one cone tilted off the view axis, so the two ring radii and the `RotatePointAroundVector` step are both visible.
///
/// `radius` scales the ring at `oldorigin` and `rotation` scales the ring at `origin`, so the cone's wide end is its `oldorigin`.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:853-953`
fn scene_fx_cylinder(host: &mut UiHost, frame_data: &mut FrameData, shader: qhandle_t) {
    let mut out = Vec::new();
    for (origin, oldorigin, axis, radius, rotation, rgba) in [
        (
            [110.0f32, -35.0, -30.0],
            [110.0f32, -35.0, 30.0],
            [0.0f32, 0.0, 1.0],
            8.0f32,
            8.0f32,
            [255u8, 255, 255, 255],
        ),
        (
            [110.0, 35.0, -30.0],
            [130.0, 20.0, 30.0],
            [0.30769232, -0.23076923, 0.9230769],
            12.0,
            2.0,
            [64, 255, 96, 255],
        ),
    ] {
        let mut re = base_ref_entity();
        re.reType = refEntityType_t::RT_CYLINDER;
        re.customShader = shader;
        re.origin = origin;
        re.oldorigin = oldorigin;
        // `axis[0]` is the cylinder axis the rings turn around, the direction the FX submitter fills.
        re.axis[0] = axis;
        re.radius = radius;
        re.rotation = rotation;
        re.shaderRGBA = rgba;
        out.push(re);
    }
    record_entities(host, frame_data, &out);
}

#[test]
fn golden_scene_sprites() {
    run_scene(&Scene {
        stem: "scene_sprites",
        eye: [0.0, 0.0, 0.0],
        record: scene_sprites,
        expect_dlights: 0,
    });
}

#[test]
fn golden_scene_lines() {
    run_scene(&Scene {
        stem: "scene_lines",
        eye: [0.0, 0.0, 0.0],
        record: scene_lines,
        expect_dlights: 0,
    });
}

#[test]
fn golden_scene_saber_glow() {
    run_scene(&Scene {
        stem: "scene_saber_glow",
        eye: [0.0, 0.0, 0.0],
        record: scene_saber_glow,
        expect_dlights: 0,
    });
}

#[test]
fn golden_scene_polys() {
    run_scene(&Scene {
        stem: "scene_polys",
        eye: [0.0, 0.0, 0.0],
        record: scene_polys,
        expect_dlights: 0,
    });
}

#[test]
fn golden_scene_renderfx_tint() {
    run_scene(&Scene {
        stem: "scene_renderfx_tint",
        eye: [0.0, 0.0, 0.0],
        record: scene_renderfx_tint,
        expect_dlights: 0,
    });
}

#[test]
fn golden_scene_depthhack() {
    run_scene(&Scene {
        stem: "scene_depthhack",
        eye: [0.0, 0.0, 0.0],
        record: scene_depthhack,
        expect_dlights: 0,
    });
}

#[test]
fn golden_scene_dlights() {
    run_scene(&Scene {
        stem: "scene_dlights",
        eye: [0.0, 0.0, 0.0],
        record: scene_dlights,
        expect_dlights: 3,
    });
}

#[test]
fn golden_scene_fx_oriented_quad() {
    run_scene(&Scene {
        stem: "scene_fx_oriented_quad",
        eye: [0.0, 0.0, 0.0],
        record: scene_fx_oriented_quad,
        expect_dlights: 0,
    });
}

#[test]
fn golden_scene_fx_cylinder() {
    run_scene(&Scene {
        stem: "scene_fx_cylinder",
        eye: [0.0, 0.0, 0.0],
        record: scene_fx_cylinder,
        expect_dlights: 0,
    });
}
