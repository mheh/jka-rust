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

use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;
use std::path::PathBuf;

use mp_engine_core::Engine;
use mp_engine_ghoul2::ghoul2_system::Ghoul2System;
use mp_engine_qcommon::cm_terrain::CmLandScape;
use mp_engine_qcommon::common::error::ComError;
use mp_engine_qcommon::files_common::FS_ProductIdFile;
use mp_engine_server::Server;
use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use mp_qshared::shared::qhandle_t;
use mp_renderer::render_state::frame_data::FrameData;
use mp_renderer::render_state::render_cvar_snapshot::RenderCvarSnapshot;
use mp_renderer::tr_local::dlight_s::dlight_t;
use mp_renderer::tr_main::TrMainScratch;
use mp_renderer::tr_model::render_models::RenderModels;
use mp_renderer::tr_public::ref_flags::{RDF_DRAWSKYBOX, RDF_NOWORLDMODEL};
use mp_renderer::tr_scene::{RE_AddPolyToScene, RE_AddRefEntityToScene, RE_RenderScene};
use mp_renderer::tr_shader::RE_RegisterShader;
use mp_renderer_gpu::ui_host::boot;
use mp_renderer_gpu::ui_host::{BootConfig, UiHost};
use mp_renderer_gpu::{read_target_rgba, FrameExecutor, Gpu, GpuImages, WorldFrame};
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
}

/// Records `entities` through `RE_AddRefEntityToScene`, the shape most scenes
/// here use.
fn record_entities(host: &mut UiHost, frame_data: &mut FrameData, entities: &[refEntity_t]) {
    for ent in entities {
        RE_AddRefEntityToScene(frame_data, &host.assets, &mut host.scene, ent);
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
";

/// Builds the synthetic game tree in a temp directory and boots a
/// renderer-only host against it. Nothing retail is reachable: `fs_basepath`
/// and `fs_homepath` both point here, and the only files present are the three
/// this function writes.
fn boot_synthetic() -> UiHost {
    let basepath = std::env::temp_dir().join("jka-rust-scene-golden-base");
    let base = basepath.join("base");
    std::fs::create_dir_all(base.join("shaders")).expect("create_dir_all: synthetic basepath");

    // `FS_InitFilesystem` errors out when `mpdefault.cfg` reads as zero bytes.
    // The renderer never executes the file, so one comment line is enough.
    // Source: oracle/codemp/qcommon/files_pc.cpp:2700-2712
    std::fs::write(base.join("mpdefault.cfg"), b"// synthetic scene golden\n")
        .expect("write: synthetic mpdefault.cfg");
    // Without this, `FS_SetRestrictions` drops the filesystem to demo mode,
    // where loose directories are never scanned and the shader script below
    // would be invisible.
    std::fs::write(base.join("productid.txt"), FS_ProductIdFile())
        .expect("write: synthetic productid.txt");
    // `ScanAndLoadShaderFiles` is a fatal error when it finds no `.shader`
    // file, so the tree needs at least this one.
    // Source: oracle/codemp/renderer/tr_shader.cpp:3895-3900
    std::fs::write(
        base.join("shaders/synthetic.shader"),
        SYNTHETIC_SHADER_SCRIPT,
    )
    .expect("write: synthetic shader script");

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
    host.assets.registered = true;
    host
}

/// Registers `name` and returns its handle. With no shader scripts and no
/// images on disk, every name resolves to the procedurally built default
/// shader, which is exactly the asset-free surface this gate wants.
fn register_shader(host: &mut UiHost, name: &str) -> qhandle_t {
    let UiHost {
        engine,
        models,
        cvars,
        assets,
        sim,
        img_state,
        gpu_res,
        frame,
        qs,
        sky_view,
        sky,
        ..
    } = host;
    let models_ptr: *mut RenderModels = &mut *models;
    let Engine { common, cm, sv, .. } = &mut **engine;
    let sv_ptr: *mut () = sv as *mut Server as *mut ();
    let mut view = boot::host_view(common, cm, sv_ptr, models_ptr);
    RE_RegisterShader(
        name, qs, frame, assets, &mut view, cvars, sim, models, img_state, gpu_res, sky_view, sky,
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
        &host.assets,
        &host.cvars,
        &mut host.scene,
        &mut host.engine.common,
        &host.sim.light_styles,
    );

    // ---- headless GPU and the render resources -------------------------
    let mut images = GpuImages::new(&gpu);
    let mut executor = FrameExecutor::new(&gpu, &images);

    let dummy_assets = boot::empty_assets();
    let land = CmLandScape::empty();
    let land_scape = boot::init_terrain(&mut host);
    let mut dlights: Vec<dlight_t> = Vec::new();
    let mut scratch = TrMainScratch {
        pre_trans_ent_matrix: [0.0; 16],
    };

    let target = gpu.headless_view();
    gpu.clear_headless(&target);
    let float_time = FROZEN_TIME_MS as f32 * 0.001;

    let _uploaded = images.upload_pending(&mut gpu, &mut host.img_state, &host.sim.published);

    let stats = {
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
        } = &mut host;
        let models_ptr: *mut RenderModels = &mut *models;
        let Engine { common, cm, sv, .. } = &mut **engine;
        let sv_ptr: *mut () = sv as *mut Server as *mut ();
        let mut engine_view = boot::host_view(common, cm, sv_ptr, models_ptr);

        let mut g2_system = Ghoul2System::default();
        let mut world = WorldFrame {
            engine_view: &mut engine_view,
            assets,
            cvars,
            frame: fstate,
            g2: &mut g2_system,
            gpu_res,
            sky,
            models: &*models,
            land_scape: &land_scape,
            land: &land,
            dlights: dlights.as_mut_slice(),
            scratch: &mut scratch,
        };

        executor.execute_frame(
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
            RenderCvarSnapshot::default(),
            Some(&mut world),
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
            &host.assets,
            &mut host.engine.common,
            shader,
            &verts,
            corner_count,
            1,
        );
    }
}

#[test]
fn golden_scene_sprites() {
    run_scene(&Scene {
        stem: "scene_sprites",
        eye: [0.0, 0.0, 0.0],
        record: scene_sprites,
    });
}

#[test]
fn golden_scene_lines() {
    run_scene(&Scene {
        stem: "scene_lines",
        eye: [0.0, 0.0, 0.0],
        record: scene_lines,
    });
}

#[test]
fn golden_scene_saber_glow() {
    run_scene(&Scene {
        stem: "scene_saber_glow",
        eye: [0.0, 0.0, 0.0],
        record: scene_saber_glow,
    });
}

#[test]
fn golden_scene_polys() {
    run_scene(&Scene {
        stem: "scene_polys",
        eye: [0.0, 0.0, 0.0],
        record: scene_polys,
    });
}
