//! The image-golden gate for the census 2D group (DEC-54).
//!
//! Every other golden in this crate draws 3D content. This file draws the 2D
//! screen alone, so the `SetColor`, `DrawStretchPic`, `DrawRotatePic` and
//! `DrawRotatePic2` rows all reach a committed image. No test calls
//! `RE_RenderScene`, so nothing here depends on a world, a camera, or the 3D
//! pipeline.
//!
//! [`golden_hud_2d`] is synthetic and asset-free. It boots against a temp game
//! tree this file writes, the same recipe `scene_golden.rs` uses, and draws
//! three quads: one axis-aligned stretch pic as the reference, one
//! `RE_RotatePic` at 45 degrees, and one `RE_RotatePic2` at 30 degrees. The
//! two rotate pics resolve to the default shader, whose bordered box makes the
//! rotation visible; a flat square would not.
//!
//! **Determinism.** The test fixes the viewport and the shader clock
//! ([`FROZEN_TIME_MS`], the fixed-dt seam DEC-58.1 names), so two runs submit
//! identical geometry.
//!
//! **Backend caveat.** Rasterisation is the GPU's. A golden blessed on one
//! adapter can differ by a channel step on another, so [`CHANNEL_TOLERANCE`]
//! exists as the knob to widen. It is zero today.
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
use mp_qshared::shared::qhandle_t;
use mp_renderer::render_state::frame_data::FrameData;
use mp_renderer::render_state::render_cvar_snapshot::RenderCvarSnapshot;
use mp_renderer::renderer_frontend::RendererFrontend;
use mp_renderer::tr_cmds::{RE_RotatePic, RE_RotatePic2, RE_SetColor, RE_StretchPic};
use mp_renderer::tr_model::render_models::RenderModels;
use mp_renderer::tr_shader::RE_RegisterShader;
use mp_renderer_gpu::ui_host::boot;
use mp_renderer_gpu::ui_host::{BootConfig, UiHost};
use mp_renderer_gpu::{read_target_rgba, FrameExecutor, FrameStats, Gpu, GpuImages};

/// The golden viewport in physical pixels. The 640x480 virtual 2D screen maps
/// to the whole viewport, so this only fixes the read-back resolution.
const GOLDEN_WIDTH: u32 = 320;
const GOLDEN_HEIGHT: u32 = 240;

/// The frozen clock in milliseconds - the fixed-dt seam on the com clock
/// (DEC-58.1). It gives `floatTime = 12.345` on every run.
const FROZEN_TIME_MS: i32 = 12345;

/// The per-channel match tolerance. Zero means an exact match. Widen this if
/// the same frame ever renders a step apart on a second adapter.
const CHANNEL_TOLERANCE: u8 = 0;

/// The shader the synthetic reference quad draws through. `$whiteimage` binds
/// `tr.whiteImage`, which `R_CreateBuiltinImages` generates, so no texture file
/// is needed. `rgbGen vertex` makes the `RE_SetColor` register visible in the
/// quad's color.
const SYNTHETIC_SHADER_SCRIPT: &str = "\
gfx/hud/reference
{
\t{
\t\tmap $whiteimage
\t\tblendFunc GL_SRC_ALPHA GL_ONE_MINUS_SRC_ALPHA
\t\trgbGen vertex
\t\talphaGen vertex
\t}
}
";

/// The name the reference quad registers, which the script above defines.
const REFERENCE_SHADER: &str = "gfx/hud/reference";

/// The name both rotate pics register. The script does not define it and no
/// image file exists, so it resolves to `tr.defaultShader`, whose stage 0 binds
/// the procedural checkerboard `tr.defaultImage`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:3705-3717` (`CreateInternalShaders`)
const ROTATE_SHADER: &str = "gfx/hud/rotate";

/// The per-call counter [`write_atomic`] puts in its temporary file names.
static TEMP_SERIAL: AtomicU64 = AtomicU64::new(0);

/// Writes `data` to `path` through a uniquely named temporary file and a
/// rename, so a concurrent reader never sees a half-written file.
fn write_atomic(path: &Path, data: &[u8]) {
    let serial = TEMP_SERIAL.fetch_add(1, Ordering::Relaxed);
    let temp = path.with_extension(format!("{}.{serial}.tmp", std::process::id()));
    std::fs::write(&temp, data).expect("write: synthetic fixture");
    std::fs::rename(&temp, path).expect("rename: synthetic fixture");
}

/// Installs a panic hook that prints the real `Com_Error` message. Without it a
/// boot fault reads as a bare `Box<dyn Any>` panic.
fn report_com_error() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Some(err) = info.payload().downcast_ref::<ComError>() {
            eprintln!("Com_Error: {}", err.msg);
        }
        previous(info);
    }));
}

/// Builds the synthetic game tree in a temp directory and boots a
/// renderer-only host against it. Nothing retail is reachable: `fs_basepath`
/// and `fs_homepath` both point here, and the only files present are the three
/// this function writes.
fn boot_synthetic() -> UiHost {
    // The directory name carries the fixture content's hash, so a tree left by
    // an older build of this file is unused instead of feeding a stale shader
    // script to a fresh run.
    let mut hasher = DefaultHasher::new();
    SYNTHETIC_SHADER_SCRIPT.hash(&mut hasher);
    let basepath =
        std::env::temp_dir().join(format!("jka-rust-hud-golden-{:016x}", hasher.finish()));
    let base = basepath.join("base");
    std::fs::create_dir_all(base.join("shaders")).expect("create_dir_all: synthetic basepath");

    write_atomic(&base.join("mpdefault.cfg"), b"// synthetic hud golden\n");
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
    report_com_error();

    let mut host = boot::boot_renderer(&cfg);
    // The ui boot path is what normally sets this; every draw trap drops its
    // submission while it is false.
    Arc::make_mut(&mut host.re.sim.published).registered = true;
    host
}

/// Registers `name` and returns its handle.
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

/// How many pixels differ from the image's top-left one. The frame here leaves
/// its corners at the clear color, so a zero means a blank render. That is
/// the trap a golden must clear before it may bless: without it an inert draw
/// path blesses an empty frame and the gate passes forever.
fn coverage(rgba: &[u8]) -> usize {
    let Some(clear) = rgba.get(0..4) else {
        return 0;
    };
    rgba.chunks_exact(4).filter(|p| *p != clear).count()
}

/// Executes `frame_data` into an offscreen target at the frozen clock and reads
/// the pixels back. No `RenderScene` event reaches the executor, so this runs
/// the 2D path alone.
fn execute_2d_frame(
    gpu: &mut Gpu,
    host: &mut UiHost,
    frame_data: &FrameData,
) -> (FrameStats, u32, u32, Vec<u8>) {
    let mut images = GpuImages::new(gpu);
    let mut executor = FrameExecutor::new(gpu, &images);

    let target = gpu.headless_view();
    // The 2D pass loads the color target, so clear it first. Otherwise the
    // golden captures wgpu zero-init in every uncovered pixel.
    gpu.clear_headless(&target);
    let float_time = FROZEN_TIME_MS as f32 * 0.001;

    // Drain the staged uploads against the sim-published master before the
    // split borrow. A drain inside `execute_frame` resolves against the dummy
    // registry and drops every staged texture.
    let _uploaded = images.upload_pending(gpu, &mut host.re.img_state, &host.re.sim.published);

    let stats = {
        // `RE_EndFrame` drains the registered model blocks into the published registry, and no test reaches it.
        // The drain therefore runs here, and it must land before the pin below.
        // Source: crates/mp/renderer/src/tr_cmds.rs:354-358
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
        } = host;

        executor.execute_frame(
            gpu,
            &target,
            frame_data,
            &pinned,
            world_load,
            img_state.pending_uploads.drain().collect(),
            &mut images,
            noise,
            float_time,
            RenderCvarSnapshot::default(),
        )
    };

    let (width, height, actual) = read_target_rgba(gpu);
    (stats, width, height, actual)
}

/// Blesses or compares `actual` against the committed golden named `stem`.
fn bless_or_compare(stem: &str, width: u32, height: u32, actual: &[u8], covered: usize) {
    let golden = golden_path(stem);
    if std::env::var("JKA_GOLDEN_BLESS").as_deref() == Ok("1") {
        write_png(&golden, width, height, actual);
        println!(
            "{}: blessed {} ({} covered pixels)",
            stem,
            golden.display(),
            covered,
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

    let (differing_pixels, max_delta) = compare(&golden_bytes, actual);
    if differing_pixels > 0 {
        let actual_out = actual_path(stem);
        write_png(&actual_out, width, height, actual);
        panic!(
            "{stem} golden mismatch: {} pixels differ, max channel delta {}; wrote actual image to {}",
            differing_pixels,
            max_delta,
            actual_out.display(),
        );
    }
}

/// Census rows `2d/SetColor`, `2d/DrawStretchPic` and `2d/DrawRotatePic`: one
/// axis-aligned reference quad and two rotate pics at different angles and
/// different pivots.
///
/// `RE_RotatePic` pivots on the rectangle's top-right corner and `RE_RotatePic2`
/// pivots on its center, so the two arms cannot pass this gate with one
/// geometry.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1498-1541,1547-1602`
#[test]
fn golden_hud_2d() {
    let Some(mut gpu) = Gpu::try_new_headless(GOLDEN_WIDTH, GOLDEN_HEIGHT) else {
        println!("hud_2d: no GPU adapter, test skipped");
        return;
    };

    let mut host = boot_synthetic();
    let pic_shader = register_shader(&mut host, REFERENCE_SHADER);
    let rot_shader = register_shader(&mut host, ROTATE_SHADER);

    let mut frame_data = FrameData { events: Vec::new() };
    {
        let assets = Arc::clone(&host.re.sim.published);
        let common = &mut host.engine.common;

        RE_SetColor(&mut frame_data, Some([1.0, 1.0, 1.0, 1.0]));
        RE_StretchPic(
            &mut frame_data,
            &assets,
            common,
            64.0,
            64.0,
            128.0,
            128.0,
            0.0,
            0.0,
            1.0,
            1.0,
            pic_shader,
        );

        RE_SetColor(&mut frame_data, Some([1.0, 0.5, 0.25, 1.0]));
        RE_RotatePic(
            &mut frame_data,
            &assets,
            common,
            256.0,
            64.0,
            128.0,
            128.0,
            0.0,
            0.0,
            1.0,
            1.0,
            45.0,
            rot_shader,
        );

        RE_SetColor(&mut frame_data, Some([0.25, 0.5, 1.0, 1.0]));
        RE_RotatePic2(
            &mut frame_data,
            &assets,
            common,
            160.0,
            320.0,
            128.0,
            128.0,
            0.0,
            0.0,
            1.0,
            1.0,
            30.0,
            rot_shader,
        );
    }

    let (stats, width, height, actual) = execute_2d_frame(&mut gpu, &mut host, &frame_data);
    assert_eq!(width, GOLDEN_WIDTH);
    assert_eq!(height, GOLDEN_HEIGHT);

    assert_eq!(
        stats.rotate_pics, 2,
        "both rotate pics must batch a quad - stats = {stats:?}",
    );
    assert!(stats.quads > 0, "no stretch-pic quad - stats = {stats:?}");
    assert!(stats.draw_calls > 0, "no draw call - stats = {stats:?}");
    let covered = coverage(&actual);
    assert!(covered > 0, "nothing drawn - stats = {stats:?}");

    bless_or_compare("hud_2d", width, height, &actual, covered);
}
