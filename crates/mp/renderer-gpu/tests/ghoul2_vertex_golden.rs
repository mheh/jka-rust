//! Ghoul2 vertex-stream differential golden: decode one stormtrooper `.glm` in
//! its default skeleton pose and compare the bone-deformed vertex streams to a
//! committed binary fixture.
//!
//! This locks the vertex stream of `decode_ghoul2_surface`
//! (`crates/mp/renderer-gpu/src/pipeline3d.rs`), which runs Raven's shipped
//! weight arm. The shipped arm special-cases one and two weights and closes
//! the last weight outside the loop. The fixture caught the swap from the dead
//! `#if 0` arm at last-bit drift (954 slots, max delta `1.9e-6`), and the
//! delta report names the exact slot count any future change moves.
//! Source: `oracle/codemp/renderer/tr_ghoul2.cpp:4313-4374`.
//!
//! The scene boots exactly like `tests/world_golden.rs`: the same `BootConfig`,
//! the `JKA_BASEPATH` override, an offscreen `Gpu`, and `maps/mp/duel1.bsp`. It
//! then inits one Ghoul2 stormtrooper the way `bin/world_harness.rs` does, adds
//! it at a fixed origin with an identity axis, and records one frame at the
//! frozen clock. The capture sink on `FrameExecutor` records each decoded ghoul2
//! surface stream in draw-surf order.
//!
//! Fixture format (`tests/goldens/ghoul2_verts_stormtrooper.bin`, all
//! little-endian):
//! - header: surface count, one `u32`.
//! - per surface, in draw-surf order:
//!   - vertex count, one `u32`.
//!   - index count, one `u32`.
//!   - positions: vertex-count `f32` triples (`x`, `y`, `z`).
//!   - normals: vertex-count `f32` triples (`x`, `y`, `z`). The parallel
//!     slice's unused `w` is dropped.
//!   - indices: index-count `u32` values.
//!
//! The test is `#[ignore]`d, matching the image goldens: it needs the retail
//! assets and a GPU, so it runs locally, not in CI. Run it with `cargo test -p
//! mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`.
//! Serial only: two engine boots in parallel threads crash in the GPU init.
//!
//! Bless flow: set `JKA_GOLDEN_BLESS=1` to write the golden and pass. On a
//! mismatch without that env var, the test writes the actual bytes next to the
//! golden as `ghoul2_verts_stormtrooper.actual.bin` and fails with a REPORT: the
//! total `f32` slots compared, the count of differing slots, the largest
//! absolute per-component delta, and the first differing surface index.

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

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
use mp_renderer::render_state::render_cvar_snapshot::RenderCvarSnapshot;
use mp_renderer::tr_local::srf_terrain_s::srfTerrain_t;
use mp_renderer::tr_main::TrMainScratch;
use mp_renderer::tr_model::render_models::RenderModels;
use mp_renderer::tr_scene::{
    ghoul2_token_encode, RE_AddRefEntityToScene, RE_ClearScene, RE_RenderScene,
};
use mp_renderer_gpu::ui_host::boot;
use mp_renderer_gpu::ui_host::{BootConfig, UiHost};
use mp_renderer_gpu::{FrameExecutor, Ghoul2SurfaceCapture, Gpu, GpuImages, WorldFrame};
use native_math::qmath::AnglesToAxis;

/// The offscreen viewport in physical pixels. Fixed so the projection and the
/// entity frustum test never depend on a window size.
const GOLDEN_WIDTH: u32 = 800;
const GOLDEN_HEIGHT: u32 = 600;

/// The frozen scene clock in milliseconds, the same value `world_golden` uses. A
/// constant clock keeps every animated stage deterministic.
const FROZEN_TIME_MS: i32 = 12345;

/// The eye-height bump added to a spawn origin, matching `world_golden`.
const EYE_HEIGHT: f32 = 40.0;

/// The horizontal field of view in degrees, matching `world_golden`.
const FOV_X: f64 = 90.0;

/// The stormtrooper stands this far in front of the eye along the view forward
/// axis. Yaw and pitch are zero, so forward is `+X`. The distance keeps the
/// model inside the frustum, so its surfaces reach the draw-surf list.
const GHOUL2_FORWARD_DIST: f32 = 160.0;

/// The model origin drops this far below the eye, so the model feet sit near the
/// floor and the body centers in the view.
const GHOUL2_DROP: f32 = 40.0;

/// The shipped player model the scene draws in its base skeleton pose.
const GHOUL2_MODEL_NAME: &str = "models/players/stormtrooper/model.glm";

/// Builds the frozen scene refdef at `eye`, looking straight ahead (yaw 0,
/// pitch 0), through the fixed viewport. This mirrors `world_golden::build_refdef`.
fn build_refdef(eye: [f32; 3]) -> refdef_t {
    // SAFETY: `refdef_t` is a frozen `#[repr(C)]` POD of scalars, fixed arrays,
    // and `vec3_t`, so an all-zero value is valid.
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

/// Inits one Ghoul2 model instance through the real `mp_engine_ghoul2` init path,
/// the same call `world_harness::init_ghoul2` makes. It allocates the handle,
/// loads the `.glm`, and reads the instance model handle. Returns `None`
/// when the model file is absent (a negative model index).
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

/// The absolute path of the committed golden.
fn golden_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/goldens/ghoul2_verts_stormtrooper.bin")
}

/// The absolute path the actual bytes land at on a mismatch.
fn actual_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/goldens/ghoul2_verts_stormtrooper.actual.bin")
}

/// Serializes the capture to the committed binary layout the module doc states.
fn serialize(capture: &[Ghoul2SurfaceCapture]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    out.extend_from_slice(&(capture.len() as u32).to_le_bytes());
    for (vertices, indices, normals) in capture {
        // The header states one count for both triples, so the two must agree.
        assert_eq!(
            vertices.len(),
            normals.len(),
            "surface vertex and normal counts must agree",
        );
        out.extend_from_slice(&(vertices.len() as u32).to_le_bytes());
        out.extend_from_slice(&(indices.len() as u32).to_le_bytes());
        for v in vertices {
            for component in v.position() {
                out.extend_from_slice(&component.to_le_bytes());
            }
        }
        for n in normals {
            // The `w` slot is unused, so only the `xyz` triple is written.
            for component in &n[0..3] {
                out.extend_from_slice(&component.to_le_bytes());
            }
        }
        for index in indices {
            out.extend_from_slice(&index.to_le_bytes());
        }
    }
    out
}

/// One parsed surface: the flat `f32` position and normal slots, and the index
/// values. The delta report compares both, so the parser keeps them per surface
/// to name the first differing surface.
struct ParsedSurface {
    f32_slots: Vec<f32>,
    indices: Vec<u32>,
}

/// A small little-endian byte cursor over a fixture blob.
struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Cursor<'a> {
        Cursor { bytes, pos: 0 }
    }

    /// Reads one little-endian `u32`, or `None` at the end of the blob.
    fn read_u32(&mut self) -> Option<u32> {
        let end = self.pos + 4;
        if end > self.bytes.len() {
            return None;
        }
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&self.bytes[self.pos..end]);
        self.pos = end;
        Some(u32::from_le_bytes(buf))
    }

    /// Reads one little-endian `f32`, or `None` at the end of the blob.
    fn read_f32(&mut self) -> Option<f32> {
        let end = self.pos + 4;
        if end > self.bytes.len() {
            return None;
        }
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&self.bytes[self.pos..end]);
        self.pos = end;
        Some(f32::from_le_bytes(buf))
    }
}

/// Parses a fixture blob back into per-surface `f32` slots and index counts. The
/// parser fails when the blob is short or ragged, so a structural fault surfaces
/// as a message, not a truncated compare.
fn parse(bytes: &[u8]) -> Result<Vec<ParsedSurface>, String> {
    let mut cursor = Cursor::new(bytes);
    let surface_count = cursor
        .read_u32()
        .ok_or_else(|| String::from("fixture is shorter than its header"))?;
    let mut surfaces: Vec<ParsedSurface> = Vec::with_capacity(surface_count as usize);
    for s in 0..surface_count {
        let vert_count = cursor
            .read_u32()
            .ok_or_else(|| format!("surface {s} is missing its vertex count"))?
            as usize;
        let index_count = cursor
            .read_u32()
            .ok_or_else(|| format!("surface {s} is missing its index count"))?
            as usize;
        // Positions and normals are both `vert_count` triples.
        let slot_count = vert_count * 3 * 2;
        let mut f32_slots: Vec<f32> = Vec::with_capacity(slot_count);
        for _ in 0..slot_count {
            let value = cursor
                .read_f32()
                .ok_or_else(|| format!("surface {s} is short on float slots"))?;
            f32_slots.push(value);
        }
        let mut indices: Vec<u32> = Vec::with_capacity(index_count);
        for _ in 0..index_count {
            let index = cursor
                .read_u32()
                .ok_or_else(|| format!("surface {s} is short on indices"))?;
            indices.push(index);
        }
        surfaces.push(ParsedSurface { f32_slots, indices });
    }
    Ok(surfaces)
}

/// The delta report between the golden and the actual capture: the total `f32`
/// slots compared, the counts of differing float slots and index values, the
/// largest absolute per-component delta, and the first differing surface index.
struct Report {
    total_slots: usize,
    differing_slots: usize,
    differing_indices: usize,
    max_delta: f32,
    first_surface: Option<usize>,
}

/// Compares two parsed fixtures slot by slot. A structural difference (surface
/// count or per-surface counts) returns an error, so the caller reports the
/// shape fault instead of a bogus delta. A float slot differs when its raw bits
/// differ, so a `-0.0` against `+0.0` counts even though its delta is zero.
fn compare(golden: &[ParsedSurface], actual: &[ParsedSurface]) -> Result<Report, String> {
    if golden.len() != actual.len() {
        return Err(format!(
            "surface count differs: golden {}, actual {}",
            golden.len(),
            actual.len(),
        ));
    }
    let mut total_slots = 0usize;
    let mut differing_slots = 0usize;
    let mut differing_indices = 0usize;
    let mut max_delta = 0.0f32;
    let mut first_surface: Option<usize> = None;
    for (s, (g, a)) in golden.iter().zip(actual.iter()).enumerate() {
        if g.f32_slots.len() != a.f32_slots.len() || g.indices.len() != a.indices.len() {
            return Err(format!(
                "surface {s} shape differs: golden {} floats {} indices, actual {} floats {} indices",
                g.f32_slots.len(),
                g.indices.len(),
                a.f32_slots.len(),
                a.indices.len(),
            ));
        }
        let mut surface_differs = false;
        for (gv, av) in g.f32_slots.iter().zip(a.f32_slots.iter()) {
            total_slots += 1;
            let delta = (gv - av).abs();
            if delta > max_delta {
                max_delta = delta;
            }
            if gv.to_bits() != av.to_bits() {
                differing_slots += 1;
                surface_differs = true;
            }
        }
        for (gi, ai) in g.indices.iter().zip(a.indices.iter()) {
            if gi != ai {
                differing_indices += 1;
                surface_differs = true;
            }
        }
        if surface_differs && first_surface.is_none() {
            first_surface = Some(s);
        }
    }
    Ok(Report {
        total_slots,
        differing_slots,
        differing_indices,
        max_delta,
        first_surface,
    })
}

/// Boots duel1, inits one stormtrooper, records one frame with the capture sink
/// armed, and compares the ghoul2 vertex streams to the committed fixture.
#[test]
#[ignore = "needs retail assets and a GPU; run locally with --ignored"]
fn golden_ghoul2_verts_stormtrooper() {
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
    assert!(loaded, "maps/mp/duel1.bsp did not load");

    // Force the first `R_MarkLeaves` to re-mark, and set the registered flag the
    // ui boot path never sets, the same two settings `world_golden` makes.
    host.frame.view_cluster = -1;
    Arc::make_mut(&mut host.sim.published).registered = true;

    // Init one stormtrooper in its default skeleton pose. No animation call runs,
    // so the pose is deterministic.
    let (mut g2, ghoul2_handle, ghoul2_model) =
        init_ghoul2(&mut host, GHOUL2_MODEL_NAME).expect("stormtrooper .glm did not init");

    // The camera sits at a spawn origin, bumped to eye height.
    let eye = host
        .sim
        .published
        .world
        .as_ref()
        .and_then(|w| boot::find_spawn_origin(&w.entity_string))
        .map(|o| [o[0], o[1], o[2] + EYE_HEIGHT])
        .unwrap_or([0.0, 0.0, 0.0]);

    let refdef = build_refdef(eye);

    // ---- record the scene ----------------------------------------------
    // The model stands a fixed distance in front of the eye with an identity
    // axis, so its surfaces reach the draw-surf list at the frozen clock.
    let mut frame_data = FrameData { events: Vec::new() };
    RE_ClearScene(&mut frame_data, &mut host.scene);

    let mut ent = refEntity_t::zeroed();
    ent.reType = refEntityType_t::RT_MODEL;
    // The zeroed `radius` pins the LOD to 0: `g2_compute_lod` projects
    // `0.75 * scale * radius`, and a zero radius projects to 0. The shape
    // asserts below lock that LOD-0 capture.
    ent.hModel = ghoul2_model;
    ent.ghoul2 = ghoul2_token_encode(Some(ghoul2_handle));
    ent.origin = [
        eye[0] + GHOUL2_FORWARD_DIST,
        eye[1],
        eye[2] - GHOUL2_DROP,
    ];
    ent.oldorigin = ent.origin;
    ent.frame = 0;
    ent.oldframe = 0;
    ent.shaderRGBA = [255, 255, 255, 255];
    AnglesToAxis([0.0, 0.0, 0.0], ent.axis.as_mut_ptr());
    RE_AddRefEntityToScene(&mut frame_data, &host.sim.published, &mut host.scene, &ent);

    RE_RenderScene(
        &refdef,
        &mut frame_data,
        &host.sim.published,
        &host.cvars,
        &mut host.scene,
        &mut host.engine.common,
        &host.sim.light_styles,
    );

    // ---- headless GPU and the render resources -------------------------
    let mut gpu = Gpu::new_headless(GOLDEN_WIDTH, GOLDEN_HEIGHT);
    let mut images = GpuImages::new(&gpu);
    let mut executor = FrameExecutor::new(&gpu, &images);
    if let Some(world) = host.sim.published.world.as_ref() {
        executor.set_world(&gpu, world);
    }

    let dummy_assets = boot::empty_assets();
    let land = CmLandScape::empty();
    let mut scratch = TrMainScratch {
        pre_trans_ent_matrix: [0.0; 16],
    };

    // ---- draw the frame with the capture sink armed --------------------
    let target = gpu.headless_view();
    gpu.clear_headless(&target);
    let float_time = FROZEN_TIME_MS as f32 * 0.001;

    // Drain the staged image uploads against the sim-published master before the
    // split borrow, the same pre-drain `world_golden` does.
    let _uploaded = images.upload_pending(&mut gpu, &mut host.img_state, &host.sim.published);

    executor.set_ghoul2_capture(true);

    {
        // Split the host and engine into disjoint borrows, the shape
        // `world_golden` builds.
        let UiHost {
            engine,
            models,
            cvars,
            sim,
            frame: fstate,
            img_state,
            noise,
            sky,
            ..
        } = &mut host;
        let models_ptr: *mut RenderModels = &mut *models;
        let Engine { common, cm, sv, .. } = &mut **engine;
        let sv_ptr: *mut () = sv as *mut Server as *mut ();
        let mut engine_view = boot::host_view(common, cm, sv_ptr, models_ptr);

        // The live Ghoul2 state threads into the frame, so the render path builds
        // the stormtrooper skeleton and deforms its surfaces.
        let mut world = WorldFrame {
            engine_view: &mut engine_view,
            assets: Arc::make_mut(&mut sim.published),
            cvars,
            frame: fstate,
            g2: &mut g2,
            sky,
            models: &*models,
            land_scape: &land_scape,
            land: &land,
            scratch: &mut scratch,
        };

        executor.execute_frame(
            &mut gpu,
            &target,
            &frame_data,
            &dummy_assets,
            img_state.pending_uploads.drain().collect(),
            &mut images,
            noise,
            float_time,
            RenderCvarSnapshot::default(),
            Some(&mut world),
        );
    }

    let capture = executor.take_ghoul2_capture();

    // The stormtrooper draws roughly 22 surfaces in the harness, so an empty
    // capture means the model never decoded and a blank fixture would bless.
    assert!(
        !capture.is_empty(),
        "no ghoul2 surface captured: the stormtrooper never decoded",
    );

    // Lock the LOD-0 shape. A count change means the capture moved to another
    // LOD or another surface set, which is not the drift this golden gates.
    assert_eq!(capture.len(), 22, "the LOD-0 stormtrooper draws 22 surfaces");
    let vert_total: usize = capture.iter().map(|(v, _, _)| v.len()).sum();
    assert_eq!(vert_total, 2583, "the LOD-0 stormtrooper decodes 2583 verts");

    let actual_bytes = serialize(&capture);
    let golden = golden_path();

    // Bless: write the golden and pass.
    if std::env::var("JKA_GOLDEN_BLESS").as_deref() == Ok("1") {
        if let Some(dir) = golden.parent() {
            fs::create_dir_all(dir).expect("create_dir_all: golden directory");
        }
        fs::write(&golden, &actual_bytes).expect("write: golden fixture");
        println!(
            "ghoul2_verts_stormtrooper: blessed {} ({} bytes, {} surfaces, {} verts)",
            golden.display(),
            actual_bytes.len(),
            capture.len(),
            vert_total,
        );
        return;
    }

    // Compare against the committed golden.
    assert!(
        golden.exists(),
        "golden missing at {}; run once with JKA_GOLDEN_BLESS=1 to write it",
        golden.display(),
    );
    let golden_bytes = fs::read(&golden).expect("read: golden fixture");
    if golden_bytes == actual_bytes {
        return;
    }

    // Mismatch: write the actual bytes and report the float-slot deltas.
    let actual_out = actual_path();
    fs::write(&actual_out, &actual_bytes).expect("write: actual fixture");

    let golden_parsed = parse(&golden_bytes).expect("parse: golden fixture");
    let actual_parsed = parse(&actual_bytes).expect("parse: actual capture");
    match compare(&golden_parsed, &actual_parsed) {
        Ok(report) => {
            panic!(
                "ghoul2 vertex golden mismatch: {} of {} float slots differ, \
                 {} index values differ, max component delta {:e}, \
                 first differing surface {:?}. Wrote actual bytes to {}.",
                report.differing_slots,
                report.total_slots,
                report.differing_indices,
                report.max_delta,
                report.first_surface,
                actual_out.display(),
            );
        }
        Err(shape) => {
            panic!(
                "ghoul2 vertex golden structural mismatch: {}. Wrote actual bytes to {}.",
                shape,
                actual_out.display(),
            );
        }
    }
}
