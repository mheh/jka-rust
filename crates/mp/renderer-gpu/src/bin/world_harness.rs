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
//! down, Escape quits. F9 toggles the shader backend between the faithful path
//! and the PBR path (DEC-37 ruling 5) and prints the new mode.
//!
//! Recorder controls: F5 drops a waypoint at the current camera pose. F6 saves
//! the recorded waypoints to the map's path file. F7 toggles replay when a path
//! file exists for the current map. F8 clears the recording.
//!
//! Path file: the harness reads and writes one file per map at
//! `crates/mp/renderer-gpu/flythroughs/<mapstem>.fly`, where `<mapstem>` is the
//! bsp base name (`duel1` for `maps/mp/duel1.bsp`). The format is plain text:
//! - line 1: `fly 1 <speed>` - the format version and the replay speed in world
//!   units per second (default 300).
//! - each later line: `x y z pitch yaw` - one waypoint as five floats, space
//!   separated. `pitch`/`yaw` are Raven view angles in degrees.
//!
//! The loader rejects a malformed file with a clear message and stays in
//! free-fly. The replay follows a closed-loop Catmull-Rom spline through the
//! waypoints at constant parameter speed. It falls back to linear interpolation
//! for fewer than four waypoints. Replay time advances from the per-frame delta
//! the harness tracks. In wall-clock mode that delta is real elapsed time, so
//! poses vary across runs with GPU load. The `--fixed-dt` mode replaces the
//! whole timeline (camera delta, scene time, shader time) with a frame counter,
//! so a recorded path replays identically every run. An image gate uses that
//! mode.
//!
//! Usage: `cargo run --release -p mp_renderer_gpu --bin world_harness
//! [-- [--flythrough] [--pbr] [--fixed-dt[=<ms>]] <basepath> [map]]`. The
//! `--flythrough` flag starts replay at boot when the map has a path file. The
//! `--pbr` flag boots on the PBR backend instead of the faithful one. The
//! `--fixed-dt` flag steps every frame by a fixed delta, 60 frames per second
//! unless `=<ms>` gives the delta in milliseconds.

use std::collections::HashSet;
use std::fs;
use std::mem;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use mp_engine_core::Engine;
use mp_engine_ghoul2::api_models::g2api_init_ghoul2_model;
use mp_engine_ghoul2::ghoul2_system::Ghoul2System;
use mp_engine_ghoul2::info_array::Ghoul2Handle;
use mp_engine_ghoul2::shared::cghoul2_info_v::CGhoul2Info_v;
use mp_engine_server::Server;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use mp_qshared::shared::qhandle_t;
use mp_renderer::render_state::frame_data::FrameData;
use mp_renderer::render_state::render_assets::RenderAssets;
use mp_renderer::render_state::bmodel_table::BModelTable;
use mp_renderer::render_state::render_cvar_snapshot::RenderCvarSnapshot;
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

/// The default replay speed in world units per second. The recorder writes this
/// into a saved path file's header.
const DEFAULT_REPLAY_SPEED: f32 = 300.0;

/// The frame delta a bare `--fixed-dt` selects, 60 frames per second.
const DEFAULT_FIXED_DT: f32 = 1.0 / 60.0;

/// Returns the simulated clock for a fixed-dt frame, in milliseconds and in
/// seconds. The frame time is the product `index * dt`, not an accumulated sum,
/// so a frame's clock depends on its index alone.
fn fixed_timeline(index: u64, dt: f32) -> (i32, f32) {
    let t = index as f64 * dt as f64;
    ((t * 1000.0) as i32, t as f32)
}

/// One recorded camera pose. `pitch`/`yaw` are Raven view angles in degrees.
/// The recorder captures these while free-flying, and the replay walks a spline
/// through them.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Waypoint {
    pos: [f32; 3],
    pitch: f32,
    yaw: f32,
}

/// Returns the signed shortest-arc angle from `a` to `b` in degrees. The result
/// stays in `[-180, 180]`, and 359 to 1 gives +2. Both signs are a valid
/// shortest arc at exactly a half turn.
fn shortest_arc_delta(a: f32, b: f32) -> f32 {
    let mut d = (b - a) % 360.0;
    if d > 180.0 {
        d -= 360.0;
    } else if d < -180.0 {
        d += 360.0;
    }
    d
}

/// Returns the copy of `y` that sits within a half turn of `prev`. This unwraps
/// a yaw sequence so the spline interpolates over the shortest arc.
fn unwrap_yaw(prev: f32, y: f32) -> f32 {
    prev + shortest_arc_delta(prev, y)
}

/// Evaluates one uniform Catmull-Rom segment between `p1` and `p2` at `u` in
/// `[0, 1]`. The curve passes through `p1` at `u = 0` and `p2` at `u = 1`.
fn catmull(p0: f32, p1: f32, p2: f32, p3: f32, u: f32) -> f32 {
    let u2 = u * u;
    let u3 = u2 * u;
    0.5 * ((2.0 * p1)
        + (-p0 + p2) * u
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * u2
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * u3)
}

/// Applies `catmull` to each component of a `vec3`.
fn catmull3(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3], p3: [f32; 3], u: f32) -> [f32; 3] {
    let mut out = [0.0f32; 3];
    for k in 0..3 {
        out[k] = catmull(p0[k], p1[k], p2[k], p3[k], u);
    }
    out
}

/// Returns the distance between two points.
fn point_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    let mut sum = 0.0f32;
    for k in 0..3 {
        let d = a[k] - b[k];
        sum += d * d;
    }
    sum.sqrt()
}

/// Serializes a recording into the `.fly` path-file text. Line 1 is the header
/// `fly 1 <speed>`. Each later line is `x y z pitch yaw`. The default float
/// format round-trips, so a parse of this text gives back the same waypoints.
fn serialize_flythrough(speed: f32, waypoints: &[Waypoint]) -> String {
    let mut out = format!("fly 1 {speed}\n");
    for w in waypoints {
        out.push_str(&format!(
            "{} {} {} {} {}\n",
            w.pos[0], w.pos[1], w.pos[2], w.pitch, w.yaw
        ));
    }
    out
}

/// Parses `.fly` path-file text into the replay speed and the waypoints. Blank
/// lines are skipped. The first content line must be the header `fly 1 <speed>`.
/// Each later line must hold five floats. A bad line returns an error with the
/// line number, so the caller can print it and stay in free-fly.
fn parse_flythrough(text: &str) -> Result<(f32, Vec<Waypoint>), String> {
    let mut lines = text
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty());

    let (hdr_no, header) = lines.next().ok_or_else(|| String::from("empty file"))?;
    let mut htok = header.split_whitespace();
    if htok.next() != Some("fly") {
        return Err(format!("line {}: expected the \"fly\" header", hdr_no + 1));
    }
    let version = htok
        .next()
        .ok_or_else(|| format!("line {}: missing version", hdr_no + 1))?;
    if version != "1" {
        return Err(format!(
            "line {}: unsupported version {version}",
            hdr_no + 1
        ));
    }
    let speed_tok = htok
        .next()
        .ok_or_else(|| format!("line {}: missing speed", hdr_no + 1))?;
    let speed: f32 = speed_tok
        .parse()
        .map_err(|_| format!("line {}: bad speed {speed_tok}", hdr_no + 1))?;
    if !speed.is_finite() || speed <= 0.0 {
        return Err(format!(
            "line {}: speed must be a positive finite number, found {speed_tok}",
            hdr_no + 1
        ));
    }

    let mut waypoints = Vec::new();
    for (no, line) in lines {
        let vals: Vec<&str> = line.split_whitespace().collect();
        if vals.len() != 5 {
            return Err(format!(
                "line {}: expected 5 floats, found {}",
                no + 1,
                vals.len()
            ));
        }
        let mut f = [0.0f32; 5];
        for (k, v) in vals.iter().enumerate() {
            f[k] = v
                .parse()
                .map_err(|_| format!("line {}: bad float {v}", no + 1))?;
            if !f[k].is_finite() {
                return Err(format!("line {}: non-finite float {v}", no + 1));
            }
        }
        waypoints.push(Waypoint {
            pos: [f[0], f[1], f[2]],
            pitch: f[3],
            yaw: f[4],
        });
    }
    Ok((speed, waypoints))
}

/// A loaded flythrough path. The replay samples a closed-loop spline through the
/// waypoints at constant parameter speed, derived from `speed` and the segment
/// lengths.
struct Flythrough {
    waypoints: Vec<Waypoint>,
    /// The replay speed in world units per second.
    speed: f32,
    /// The chord length of each closed-loop segment. Segment `i` runs from
    /// waypoint `i` to waypoint `(i + 1) % n`, so this holds `n` entries.
    seg_len: Vec<f32>,
    /// The sum of the segment lengths, the full loop distance.
    total: f32,
}

impl Flythrough {
    /// Builds a flythrough from a parsed speed and waypoints. The constructor
    /// computes the closed-loop segment lengths once.
    fn new(speed: f32, waypoints: Vec<Waypoint>) -> Flythrough {
        let n = waypoints.len();
        let mut seg_len = Vec::with_capacity(n);
        if n >= 2 {
            for i in 0..n {
                seg_len.push(point_distance(waypoints[i].pos, waypoints[(i + 1) % n].pos));
            }
        }
        let total = seg_len.iter().sum();
        Flythrough {
            waypoints,
            speed,
            seg_len,
            total,
        }
    }

    /// Samples the camera pose at a replay time in seconds. The time maps to a
    /// distance along the loop by `speed`, wrapped over the loop length.
    fn sample(&self, time: f32) -> Waypoint {
        if self.waypoints.is_empty() {
            return Waypoint {
                pos: [0.0; 3],
                pitch: 0.0,
                yaw: 0.0,
            };
        }
        if self.total <= 0.0 {
            return self.waypoints[0];
        }
        let d = (time * self.speed).rem_euclid(self.total);
        self.eval_at(d)
    }

    /// Evaluates the pose at a distance `d` along the loop, `d` in `[0, total)`.
    fn eval_at(&self, d: f32) -> Waypoint {
        let n = self.waypoints.len();

        // Locate the segment that holds this distance.
        let mut i = 0;
        let mut acc = 0.0f32;
        while i < n {
            if d < acc + self.seg_len[i] || i == n - 1 {
                break;
            }
            acc += self.seg_len[i];
            i += 1;
        }
        let seg = self.seg_len[i];
        let u = if seg > 0.0 { (d - acc) / seg } else { 0.0 };
        let next = (i + 1) % n;

        // A short recording cannot support the four-point spline.
        if n < 4 {
            return self.eval_linear(i, next, u);
        }
        let prev = (i + n - 1) % n;
        let after = (i + 2) % n;
        self.eval_catmull(prev, i, next, after, u)
    }

    /// Interpolates linearly between two waypoints. Yaw follows the shortest arc.
    fn eval_linear(&self, i: usize, next: usize, u: f32) -> Waypoint {
        let a = &self.waypoints[i];
        let b = &self.waypoints[next];
        let mut pos = [0.0f32; 3];
        for k in 0..3 {
            pos[k] = a.pos[k] + (b.pos[k] - a.pos[k]) * u;
        }
        let pitch = a.pitch + (b.pitch - a.pitch) * u;
        let yaw = a.yaw + shortest_arc_delta(a.yaw, b.yaw) * u;
        Waypoint { pos, pitch, yaw }
    }

    /// Evaluates the Catmull-Rom spline for one segment. Yaw uses the same
    /// spline weights on the four yaw values, unwrapped over the shortest arc.
    fn eval_catmull(&self, p0: usize, p1: usize, p2: usize, p3: usize, u: f32) -> Waypoint {
        let w0 = &self.waypoints[p0];
        let w1 = &self.waypoints[p1];
        let w2 = &self.waypoints[p2];
        let w3 = &self.waypoints[p3];

        let pos = catmull3(w0.pos, w1.pos, w2.pos, w3.pos, u);
        let pitch = catmull(w0.pitch, w1.pitch, w2.pitch, w3.pitch, u);

        // Unwrap the yaw run around the anchor `w1`, so the spline never turns
        // the long way. 359 to 1 crosses 0.
        let y1 = w1.yaw;
        let y0 = unwrap_yaw(y1, w0.yaw);
        let y2 = unwrap_yaw(y1, w2.yaw);
        let y3 = unwrap_yaw(y2, w3.yaw);
        let yaw = catmull(y0, y1, y2, y3, u);

        Waypoint { pos, pitch, yaw }
    }
}

/// Returns the path-file location for a map stem. The path is fixed under the
/// crate directory, so the harness finds it from any working directory.
fn flythrough_file_path(stem: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("flythroughs")
        .join(format!("{stem}.fly"))
}

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
    /// The 2D command surface reads this, and this harness draws no 2D, so it
    /// stands in for the `assets` parameter. The real registry
    /// (`host.sim.published`) is borrowed by the `WorldFrame`, so it cannot
    /// also fill that parameter. The harness therefore drains the staged image
    /// uploads against the real registry itself, before the split borrow (see
    /// `draw_world_frame`).
    dummy_assets: RenderAssets,
    // W2-F5/F6 moved the null-landscape terrain seed, its collision twin, and
    // the `tr_main` matrix scratch onto `FrameExecutor`, which owns them for
    // the process lifetime.
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
    /// The bsp base name of the loaded map. The recorder saves to this map's
    /// path file.
    map_stem: String,
    /// The waypoints the recorder has dropped this session.
    recorded: Vec<Waypoint>,
    /// The loaded path file for the current map, `None` when no valid file
    /// exists. F7 replay needs this.
    flythrough: Option<Flythrough>,
    /// The camera follows the flythrough spline while this is true. Free-fly
    /// input is ignored then, except F7 and Escape.
    replaying: bool,
    /// The replay clock in seconds. It advances by the per-frame delta, so a
    /// fixed frame sequence gives a fixed camera path.
    replay_time: f32,
    /// The fixed frame delta in seconds, `None` in wall-clock mode.
    /// `--fixed-dt` sets it, and the whole timeline then steps by it.
    fixed_dt: Option<f32>,
    /// The count of frames drawn so far.
    /// Fixed-dt mode derives its clock from this count.
    frame_index: u64,
    /// The render cvar snapshot the executor reads each frame. F9 flips its
    /// `pbr` field, and `--pbr` sets it at boot. Every other field keeps the
    /// retail default, so the faithful path stays byte-exact.
    cvars: RenderCvarSnapshot,
}

impl App {
    #[allow(clippy::too_many_arguments)]
    fn new(
        host: UiHost,
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
            map_stem: String::new(),
            recorded: Vec::new(),
            flythrough: None,
            replaying: false,
            replay_time: 0.0,
            fixed_dt: None,
            frame_index: 0,
            cvars: RenderCvarSnapshot::default(),
        }
    }

    /// Flips the shader backend and prints the new mode. F9 calls it.
    fn toggle_backend(&mut self) {
        self.cvars.pbr = if self.cvars.pbr != 0 { 0 } else { 1 };
        println!(
            "world_harness: backend now {}",
            backend_name(self.cvars.pbr)
        );
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
        // Replay drives the camera, so it ignores the mouse.
        if self.replaying {
            return;
        }
        self.camera.yaw -= dx * MOUSE_SENS;
        self.camera.pitch += dy * MOUSE_SENS;
        self.camera.pitch = self.camera.pitch.clamp(-89.0, 89.0);
    }

    /// Drops one waypoint at the current camera pose and prints the count.
    fn drop_waypoint(&mut self) {
        // The recorder captures hand-flown poses only, so replay ignores F5.
        if self.replaying {
            println!("world_harness: replay active, the recorder ignores F5");
            return;
        }
        self.recorded.push(Waypoint {
            pos: self.camera.pos,
            pitch: self.camera.pitch,
            yaw: self.camera.yaw,
        });
        println!("world_harness: waypoint {} dropped", self.recorded.len());
    }

    /// Saves the recorded waypoints to the map's path file. An empty recording
    /// prints a note and writes nothing.
    fn save_recording(&mut self) {
        if self.recorded.is_empty() {
            println!("world_harness: no waypoints recorded, nothing saved");
            return;
        }
        let path = flythrough_file_path(&self.map_stem);
        if let Some(dir) = path.parent() {
            if let Err(error) = fs::create_dir_all(dir) {
                println!(
                    "world_harness: could not create {} ({error})",
                    dir.display()
                );
                return;
            }
        }
        let text = serialize_flythrough(DEFAULT_REPLAY_SPEED, &self.recorded);
        if path.exists() {
            println!("world_harness: overwriting {}", path.display());
        }
        match fs::write(&path, text) {
            Ok(()) => {
                println!(
                    "world_harness: saved {} waypoints to {}",
                    self.recorded.len(),
                    path.display()
                );
                // Install the saved path so F7 replays it without a restart.
                self.flythrough =
                    Some(Flythrough::new(DEFAULT_REPLAY_SPEED, self.recorded.clone()));
                println!("world_harness: F7 now replays the saved path");
            }
            Err(error) => {
                println!(
                    "world_harness: could not write {} ({error})",
                    path.display()
                )
            }
        }
    }

    /// Clears the recording and prints a note.
    fn clear_recording(&mut self) {
        self.recorded.clear();
        println!("world_harness: recording cleared");
    }

    /// Toggles replay when a path file exists for the current map. The camera
    /// starts the loop from the top each time replay begins.
    fn toggle_replay(&mut self) {
        if self.flythrough.is_none() {
            println!("world_harness: no flythrough for this map, staying in free-fly");
            return;
        }
        if self.replaying {
            self.replaying = false;
            println!("world_harness: replay stopped, free-fly resumed");
        } else {
            self.replaying = true;
            self.replay_time = 0.0;
            println!("world_harness: replay started");
        }
    }

    /// Advances the replay clock and moves the camera onto the spline. The clock
    /// grows by the per-frame delta, so a fixed frame sequence is deterministic.
    fn advance_replay(&mut self, dt: f32) {
        let Some(fly) = self.flythrough.as_ref() else {
            self.replaying = false;
            return;
        };
        self.replay_time += dt;
        let wp = fly.sample(self.replay_time);
        self.camera.pos = wp.pos;
        // The spline can overshoot past a control point, so clamp pitch to the
        // same bound free-fly uses, or the view rolls over past 90 degrees.
        self.camera.pitch = wp.pitch.clamp(-89.0, 89.0);
        self.camera.yaw = wp.yaw;
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
            &self.host.sim.published,
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

        RE_AddRefEntityToScene(frame_data, &self.host.sim.published, &mut self.host.scene, &ent);
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

        RE_AddRefEntityToScene(frame_data, &self.host.sim.published, &mut self.host.scene, &ent);
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

        RE_AddRefEntityToScene(frame_data, &self.host.sim.published, &mut self.host.scene, &ent);
    }

    /// One frame: advance the camera, record the scene, draw it.
    /// Fixed-dt mode takes the whole clock from the frame count, so no wall
    /// time reaches the camera, the scene, or the shaders.
    fn frame(&mut self) {
        let (dt, time_ms, float_time) = match self.fixed_dt {
            Some(fdt) => {
                let (ms, s) = fixed_timeline(self.frame_index, fdt);
                (fdt, ms, s)
            }
            None => {
                let now = Instant::now();
                let dt = (now - self.last_frame).as_secs_f32();
                self.last_frame = now;
                let time_ms = self.start.elapsed().as_millis() as i32;
                let float_time = self.start.elapsed().as_secs_f32();
                (dt, time_ms, float_time)
            }
        };
        self.frame_index += 1;

        if self.replaying {
            self.advance_replay(dt);
        } else {
            self.update_camera(dt);
        }
        let refdef = self.build_refdef(time_ms);
        let frame_data = self.record_scene(&refdef);
        self.draw_world_frame(&frame_data, float_time);
    }

    /// Acquires the frame target, builds the world context, and drives the
    /// executor. The executor runs the whole world chain and presents.
    fn draw_world_frame(&mut self, frame_data: &FrameData, float_time: f32) {
        // Read the render cvar snapshot before the split borrow below takes
        // `self`. It is `Copy`, so the F9-driven `pbr` field rides into the
        // executor by value. The name stays clear of the `UiHost::cvars` the
        // inner block destructures.
        let cvar_snapshot = self.cvars;
        let App {
            host,
            gpu,
            executor,
            images,
            window,
            dummy_assets,
            reported,
            surface_warned,
            md3_model,
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
                // below. `execute_frame` drains with its `assets` parameter,
                // which this harness fills with `dummy_assets`, and a drain
                // against an empty registry drops every staged world texture
                // and lightmap for good. Every registration writes the one
                // published registry (A9), so the drain resolves the staged
                // handles there.
                let uploaded = images.upload_pending(gpu, &mut host.img_state, &host.sim.published);

                let mut stats = {
                    // Split the host and engine into disjoint borrows, the same
                    // shape `load_world_and_render` builds.
                    let UiHost {
                        engine,
                        models,
                        sim,
                        frame: fstate,
                        world_load,
                        img_state,
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
                        assets: Arc::make_mut(&mut sim.published),
                        world_load,
                        frame: fstate,
                        sky,
                        models: &*models,
                    };

                    executor.execute_frame(
                        gpu,
                        &target,
                        frame_data,
                        &*dummy_assets,
                        img_state.pending_uploads.drain().collect(),
                        images,
                        noise,
                        float_time,
                        // The harness rides the F9-driven snapshot. Every field
                        // but `pbr` keeps the retail default.
                        cvar_snapshot,
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

        // The executor owns the Ghoul2 instances since W2-F5, so the set this
        // harness built before the GPU came up moves in here.
        executor.set_ghoul2(mem::take(&mut self.g2));

        // Upload the loaded world's geometry once, before the first frame. The
        // brush-submodel rows the same map load registered go with it (W2-F8).
        let bmodel_table = BModelTable::build(&self.host.models);
        if let Some(world) = self.host.sim.published.world.as_ref() {
            executor.set_world(&gpu, world, bmodel_table);
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
                if state == ElementState::Pressed {
                    match code {
                        KeyCode::Escape => {
                            event_loop.exit();
                            return;
                        }
                        KeyCode::F5 => {
                            self.drop_waypoint();
                            return;
                        }
                        KeyCode::F6 => {
                            self.save_recording();
                            return;
                        }
                        KeyCode::F7 => {
                            self.toggle_replay();
                            return;
                        }
                        KeyCode::F8 => {
                            self.clear_recording();
                            return;
                        }
                        KeyCode::F9 => {
                            self.toggle_backend();
                            return;
                        }
                        _ => {}
                    }
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

/// The human name of the shader backend a `pbr` snapshot value selects. Zero is
/// the faithful reference backend, non-zero is the PBR backend.
fn backend_name(pbr: i32) -> &'static str {
    if pbr != 0 {
        "PBR"
    } else {
        "faithful"
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
        let UiHost { engine, models, .. } = &mut *host;
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
    // The `--flythrough` flag may sit anywhere. The two positionals stay
    // `[basepath] [map]`, in order. An unknown flag or a third positional
    // exits with a usage line, so a typo never lands in the wrong slot.
    let mut basepath: Option<String> = None;
    let mut map: Option<String> = None;
    let mut flythrough_flag = false;
    let mut pbr_flag = false;
    let mut fixed_dt: Option<f32> = None;
    for arg in std::env::args().skip(1) {
        if arg == "--flythrough" {
            flythrough_flag = true;
        } else if arg == "--pbr" {
            pbr_flag = true;
        } else if arg == "--fixed-dt" {
            fixed_dt = Some(DEFAULT_FIXED_DT);
        } else if let Some(ms_tok) = arg.strip_prefix("--fixed-dt=") {
            let ms: f32 = match ms_tok.parse() {
                Ok(ms) => ms,
                Err(_) => {
                    eprintln!("world_harness: bad fixed-dt value {ms_tok}");
                    std::process::exit(2);
                }
            };
            if !ms.is_finite() || ms <= 0.0 {
                eprintln!("world_harness: fixed-dt must be a positive finite number of milliseconds, found {ms_tok}");
                std::process::exit(2);
            }
            fixed_dt = Some(ms / 1000.0);
        } else if arg.starts_with("--") {
            eprintln!("world_harness: unknown flag {arg}");
            eprintln!(
                "usage: world_harness [--flythrough] [--pbr] [--fixed-dt[=<ms>]] [basepath] [map]"
            );
            std::process::exit(2);
        } else if basepath.is_none() {
            basepath = Some(arg);
        } else if map.is_none() {
            map = Some(arg);
        } else {
            eprintln!("world_harness: unexpected argument {arg}");
            eprintln!(
                "usage: world_harness [--flythrough] [--pbr] [--fixed-dt[=<ms>]] [basepath] [map]"
            );
            std::process::exit(2);
        }
    }

    let mut cfg = BootConfig::default();
    if let Some(bp) = basepath {
        cfg.basepath = bp;
    }
    let map = map.unwrap_or_else(|| String::from("maps/mp/duel1.bsp"));

    let mut host = boot::boot(&cfg);
    // The terrain surface `load_world` returns is the null-landscape seed. The
    // executor owns its own copy since W2-F6, so this one is dropped.
    let (loaded, _land_scape) = boot::load_world(&mut host, &map);
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
    Arc::make_mut(&mut host.sim.published).registered = true;

    // Start the camera at a spawn origin, bumped to eye height.
    let eye = host
        .sim
        .published
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
        .sim
        .published
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

    // Load the map's path file if one exists. A malformed file prints a note
    // and leaves the harness in free-fly.
    let map_stem = Path::new(&map)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("world")
        .to_string();
    let fly_path = flythrough_file_path(&map_stem);
    let flythrough = match fs::read_to_string(&fly_path) {
        Ok(text) => match parse_flythrough(&text) {
            Ok((speed, waypoints)) if !waypoints.is_empty() => {
                let count = waypoints.len();
                if count == 1 {
                    println!(
                        "world_harness: flythrough {} has 1 waypoint, replay holds a single pose",
                        fly_path.display()
                    );
                } else if count < 4 {
                    println!(
                        "world_harness: flythrough {} has {count} waypoints, replay uses linear interpolation",
                        fly_path.display()
                    );
                } else {
                    println!(
                        "world_harness: loaded flythrough {} ({count} waypoints, speed {speed})",
                        fly_path.display()
                    );
                }
                Some(Flythrough::new(speed, waypoints))
            }
            Ok(_) => {
                println!(
                    "world_harness: flythrough {} has no waypoints, staying in free-fly",
                    fly_path.display()
                );
                None
            }
            Err(error) => {
                println!(
                    "world_harness: flythrough {} is malformed ({error}), staying in free-fly",
                    fly_path.display()
                );
                None
            }
        },
        Err(_) => None,
    };

    app.map_stem = map_stem;
    app.cvars.pbr = if pbr_flag { 1 } else { 0 };
    println!("world_harness: backend {}", backend_name(app.cvars.pbr));
    app.fixed_dt = fixed_dt;
    if let Some(fdt) = fixed_dt {
        println!("world_harness: fixed dt {} ms", fdt * 1000.0);
    }
    app.replaying = flythrough_flag && flythrough.is_some();
    if app.replaying {
        println!("world_harness: replay started");
    } else if flythrough_flag {
        println!("world_harness: --flythrough given but no path file loaded, staying in free-fly");
    }
    app.flythrough = flythrough;

    let event_loop = EventLoop::new().expect("EventLoop::new: failed to create the event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop
        .run_app(&mut app)
        .expect("run_app: world harness event loop exited with an error");
}

#[cfg(test)]
mod tests {
    use super::{
        fixed_timeline, parse_flythrough, serialize_flythrough, shortest_arc_delta, Flythrough,
        Waypoint,
    };

    fn sample_waypoints() -> Vec<Waypoint> {
        vec![
            Waypoint {
                pos: [0.0, 0.0, 0.0],
                pitch: -10.0,
                yaw: 0.0,
            },
            Waypoint {
                pos: [100.0, 0.0, 20.0],
                pitch: -5.0,
                yaw: 90.0,
            },
            Waypoint {
                pos: [100.0, 100.0, 40.0],
                pitch: 0.0,
                yaw: 180.0,
            },
            Waypoint {
                pos: [0.0, 100.0, 20.0],
                pitch: 5.0,
                yaw: 270.0,
            },
            Waypoint {
                pos: [-50.0, 50.0, 10.0],
                pitch: 12.5,
                yaw: 350.0,
            },
        ]
    }

    #[test]
    fn path_file_round_trips() {
        let waypoints = sample_waypoints();
        let text = serialize_flythrough(300.0, &waypoints);
        let (speed, parsed) = parse_flythrough(&text).expect("valid text must parse");
        assert_eq!(speed, 300.0);
        assert_eq!(parsed, waypoints);
    }

    #[test]
    fn malformed_file_is_rejected() {
        // No header.
        assert!(parse_flythrough("garbage line here\n").is_err());
        // Wrong version.
        assert!(parse_flythrough("fly 2 300\n0 0 0 0 0\n").is_err());
        // A waypoint line with the wrong float count.
        assert!(parse_flythrough("fly 1 300\n1 2 3\n").is_err());
        // A waypoint line with a non-numeric field.
        assert!(parse_flythrough("fly 1 300\n0 0 0 0 north\n").is_err());
        // The empty string.
        assert!(parse_flythrough("").is_err());
    }

    #[test]
    fn yaw_wraps_over_the_shortest_arc() {
        // 359 to 1 is +2.
        assert!((shortest_arc_delta(359.0, 1.0) - 2.0).abs() < 1e-4);
        // 1 to 359 is -2.
        assert!((shortest_arc_delta(1.0, 359.0) + 2.0).abs() < 1e-4);
        // 350 to 10 is +20.
        assert!((shortest_arc_delta(350.0, 10.0) - 20.0).abs() < 1e-4);
        // 10 to 350 is -20.
        assert!((shortest_arc_delta(10.0, 350.0) + 20.0).abs() < 1e-4);
    }

    #[test]
    fn fixed_timeline_depends_on_the_index_alone() {
        // Frame zero starts the clock at zero.
        assert_eq!(fixed_timeline(0, 0.25), (0, 0.0));
        // Four frames of 250 ms land exactly on one second.
        assert_eq!(fixed_timeline(4, 0.25), (1000, 1.0));
        // The same index always gives the same clock.
        assert_eq!(
            fixed_timeline(12345, 1.0 / 60.0),
            fixed_timeline(12345, 1.0 / 60.0)
        );
        // A product clock never drifts: frame 3200 of a binary-exact 31.25 ms
        // delta lands exactly on 100 seconds.
        assert_eq!(fixed_timeline(3200, 0.03125), (100_000, 100.0));
    }

    #[test]
    fn closed_loop_spline_passes_through_each_waypoint() {
        let waypoints = sample_waypoints();
        let n = waypoints.len();
        let fly = Flythrough::new(300.0, waypoints.clone());

        // The cumulative distance to waypoint `i` is the sum of the segments
        // before it. The spline must return that waypoint's pose there.
        let mut acc = 0.0f32;
        for i in 0..n {
            let wp = fly.eval_at(acc);
            for k in 0..3 {
                assert!(
                    (wp.pos[k] - waypoints[i].pos[k]).abs() < 1e-3,
                    "waypoint {i} axis {k}: got {}, want {}",
                    wp.pos[k],
                    waypoints[i].pos[k]
                );
            }
            acc += fly.seg_len[i];
        }
    }
}
