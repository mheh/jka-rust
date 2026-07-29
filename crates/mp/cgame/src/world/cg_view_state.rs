//! `CgViewState` — `cg_view.c`'s mutable file-scope globals as one `CgWorld`
//! sub-struct.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_abi::cgame::shared_buffer::autoMapInput_t;
use mp_qshared::shared::{qfalse, vec3_t};

use crate::local::cgscreffects_s::cgscreffects_t;

/// `cg_view.c`'s mutable file-scope globals, grouped by owning `.c` file
/// (§B3: file-scope globals become owned state, they never become Rust
/// globals).
///
/// Fields fold in as the waves transcribe `cg_view.c`'s file-scope statics
/// (DEC-46.1), so a wave transcriber only ever touches its own TU's two files —
/// the function file and this one — and never `cg_world.rs`. Raven's read-only
/// tables beside them are compiled-in data, not state; they land as `const`s
/// beside the functions that read them (§C8).
///
/// No derives: `cgscreffects_t` is a plain C1 leaf port with none of its own,
/// and it is not this wave's file to touch. `Default` is hand-written below
/// instead, which is the only one `CgWorld::new_boxed` asks for.
///
/// Source: `oracle/codemp/cgame/cg_view.c:223-232,1189,1264,1396-1397,1476,2009,2277-2281,2445`
pub struct CgViewState {
    /// Raven `vec3_t camerafwd` — the third-person camera's forward axis, the
    /// direction [`crate::cg_view::CG_CalcIdealThirdPersonViewLocation`] backs
    /// the camera off along.
    /// Source: `oracle/codemp/cgame/cg_view.c:223`
    pub camerafwd: vec3_t,

    /// Raven `vec3_t cameraFocusLoc` — the eye point the third-person camera
    /// aims at, before the vertical offset.
    /// Source: `oracle/codemp/cgame/cg_view.c:225`
    pub cameraFocusLoc: vec3_t,

    /// Raven `vec3_t cameraIdealTarget` — `cameraFocusLoc` plus the vertical
    /// offset, i.e. where the camera would look with no damping.
    /// Source: `oracle/codemp/cgame/cg_view.c:226`
    pub cameraIdealTarget: vec3_t,

    /// Raven `vec3_t cameraIdealLoc` — where the camera would sit with no
    /// damping.
    /// Source: `oracle/codemp/cgame/cg_view.c:226`
    pub cameraIdealLoc: vec3_t,

    /// Raven `vec3_t cameraup` — the third-person camera's up axis, filled
    /// beside `camerafwd` from `cameraFocusAngles`.
    /// Source: `oracle/codemp/cgame/cg_view.c:223`
    pub cameraup: vec3_t,

    /// Raven `vec3_t cameraFocusAngles` — the angles the third-person camera
    /// looks along, pitch-capped to +-89.
    /// Source: `oracle/codemp/cgame/cg_view.c:225`
    pub cameraFocusAngles: vec3_t,

    /// Raven `vec3_t cameraCurTarget` — the damped target the camera is
    /// actually looking at this frame.
    /// Source: `oracle/codemp/cgame/cg_view.c:227`
    pub cameraCurTarget: vec3_t,

    /// Raven `vec3_t cameraCurLoc` — the damped, trace-clipped spot the camera
    /// is actually sitting at this frame.
    /// Source: `oracle/codemp/cgame/cg_view.c:227`
    pub cameraCurLoc: vec3_t,

    /// Raven `int cameraLastFrame` — `cg.time` the last time the camera damp
    /// ran; the damp exponent's time base.
    /// Source: `oracle/codemp/cgame/cg_view.c:229`
    pub cameraLastFrame: c_int,

    /// Raven `float cameraLastYaw` — last frame's focus yaw, so a fast yaw
    /// change can stiffen the camera.
    /// Source: `oracle/codemp/cgame/cg_view.c:231`
    pub cameraLastYaw: f32,

    /// Raven `float cameraStiffFactor` — how much of the remaining damp gets
    /// shaved off; approaches 1 as the yaw change speeds up.
    /// Source: `oracle/codemp/cgame/cg_view.c:232`
    pub cameraStiffFactor: f32,

    /// Raven `float cg_autoMapZoom` — how far back the automap camera pulls,
    /// walked by the automap input.
    /// Source: `oracle/codemp/cgame/cg_view.c:2277`
    pub cg_autoMapZoom: f32,

    /// Raven `float cg_autoMapZoomMainOffset` — the floor the automap zoom
    /// clamps against, so the zoom range slides with it.
    /// Source: `oracle/codemp/cgame/cg_view.c:2278`
    pub cg_autoMapZoomMainOffset: f32,

    /// Raven `vec3_t cg_autoMapAngle` — the automap camera's angles; starts
    /// straight down.
    /// Source: `oracle/codemp/cgame/cg_view.c:2279`
    pub cg_autoMapAngle: vec3_t,

    /// Raven `autoMapInput_t cg_autoMapInput` — the last automap input the
    /// engine handed over through the shared buffer.
    /// Source: `oracle/codemp/cgame/cg_view.c:2280`
    pub cg_autoMapInput: autoMapInput_t,

    /// Raven `int cg_autoMapInputTime` — until when that input keeps driving
    /// the automap camera.
    /// Source: `oracle/codemp/cgame/cg_view.c:2281`
    pub cg_autoMapInputTime: c_int,

    /// Raven `cgscreffects_t cgScreenEffects` — the screen-shake and
    /// music-ducking state the `CG_SE_*` / `CGCam_*` fns drive.
    /// Source: `oracle/codemp/cgame/cg_view.c:2009`
    pub cgScreenEffects: cgscreffects_t,

    /// Raven `float zoomFov` — the live zoom fov, walked down a frametime step
    /// at a time by [`crate::cg_view::CG_CalcFov`].
    ///
    /// Raven: "this has to be global client-side".
    /// Source: `oracle/codemp/cgame/cg_view.c:1189`
    pub zoomFov: f32,

    /// Raven's `static int zoomSoundTime` inside `CG_CalcFov` — when the
    /// disruptor zoom loop is next allowed to fire.
    /// Source: `oracle/codemp/cgame/cg_view.c:1264`
    pub zoomSoundTime: c_int,

    /// Raven `float cg_linearFogOverride` — designer-specified override for
    /// linear fogging style, off the worldspawn's `fogstart`; `cg_main.c` is
    /// its one writer.
    /// Source: `oracle/codemp/cgame/cg_view.c:2435`
    pub cg_linearFogOverride: f32,

    /// Raven `vec3_t cg_actionCamLastPos` — the third-person action camera's
    /// last damped position, walked toward the desired position a frame at a
    /// time.
    /// Source: `oracle/codemp/cgame/cg_view.c:1396`
    pub cg_actionCamLastPos: vec3_t,

    /// Raven `int cg_actionCamLastTime` — `cg.time` the action camera last
    /// ran; a 300ms gap re-seeds it from a fresh third-person offset.
    /// Source: `oracle/codemp/cgame/cg_view.c:1397`
    pub cg_actionCamLastTime: c_int,

    /// Raven `vec3_t cg_lastTurretViewAngles` — the view angles
    /// [`crate::cg_view::CG_CalcViewValues`] last latched on a turret frame.
    /// Write-only in retail: Raven's sole reader sits inside a commented-out
    /// block (`cg_view.c:1541-1544`), preserved faithfully.
    /// Source: `oracle/codemp/cgame/cg_view.c:1476,1646`
    pub cg_lastTurretViewAngles: vec3_t,

    /// Raven's `static float lastfov` inside `CG_DrawSkyBoxPortal` — latches
    /// the live zoom fov on entry, for transitions back out of a zoomed-in
    /// mode; walked further while the portal sky's own fov is zooming in.
    /// Source: `oracle/codemp/cgame/cg_view.c:1750`
    pub lastfov: f32,

    /// Raven `int cg_siegeClassIndex` — the local player's last-seen siege
    /// class, so [`crate::cg_view::CG_DrawActiveFrame`] only re-fires
    /// `ui_mySiegeClass` when it changes. Starts at Raven's -2 sentinel, which
    /// no real siegeIndex (-1 = none, >= 0 = a class) can equal, so the first
    /// siege frame always fires.
    /// Source: `oracle/codemp/cgame/cg_view.c:2445`
    pub cg_siegeClassIndex: c_int,

    /// Raven `qboolean cg_rangedFogging` — so we know if we should go back to
    /// normal fog.
    /// Source: `oracle/codemp/cgame/cg_view.c:2434`
    pub cg_rangedFogging: bool,

    /// Raven's `static centity_t *veh` inside `CG_DrawActiveFrame` — the last
    /// vehicle entity latched for the fighter-alt-control check; an
    /// `Option<usize>` index into `cg_entities` per DEC-46.2, since it is
    /// never reset back to `None` except by the fighter-alt-control branch
    /// itself (Raven's own comment: "so I don't want an extra assign each
    /// frame").
    /// Source: `oracle/codemp/cgame/cg_view.c:2453`
    pub veh: Option<usize>,
}

impl Default for CgViewState {
    /// Raven's zeroed BSS, except the four automap globals he gave loaded
    /// initializers — those keep Raven's values.
    fn default() -> Self {
        CgViewState {
            camerafwd: [0.0; 3],
            cameraFocusLoc: [0.0; 3],
            cameraIdealTarget: [0.0; 3],
            cameraIdealLoc: [0.0; 3],
            cameraup: [0.0; 3],
            cameraFocusAngles: [0.0; 3],
            cameraCurTarget: [0.0; 3],
            cameraCurLoc: [0.0; 3],
            cameraLastFrame: 0,
            cameraLastYaw: 0.0,
            cameraStiffFactor: 0.0,
            cg_autoMapZoom: 512.0,
            cg_autoMapZoomMainOffset: 0.0,
            cg_autoMapAngle: [90.0, 0.0, 0.0],
            cg_autoMapInput: autoMapInput_t {
                up: 0.0,
                down: 0.0,
                yaw: 0.0,
                pitch: 0.0,
                goToDefaults: qfalse,
            },
            cg_autoMapInputTime: 0,
            cgScreenEffects: cgscreffects_t {
                FOV: 0.0,
                FOV2: 0.0,
                shake_intensity: 0.0,
                shake_duration: 0,
                shake_start: 0,
                music_volume_multiplier: 0.0,
                music_volume_time: 0,
                music_volume_set: qfalse,
            },
            zoomFov: 0.0,
            zoomSoundTime: 0,
            cg_linearFogOverride: 0.0,
            cg_actionCamLastPos: [0.0; 3],
            cg_actionCamLastTime: 0,
            cg_lastTurretViewAngles: [0.0; 3],
            lastfov: 0.0,
            // Raven's loaded initializer, not BSS zero - see the field doc.
            cg_siegeClassIndex: -2,
            cg_rangedFogging: false,
            veh: None,
        }
    }
}
