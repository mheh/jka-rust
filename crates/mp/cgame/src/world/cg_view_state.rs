//! `CgViewState` — `cg_view.c`'s mutable file-scope globals as one `CgWorld`
//! sub-struct.

#![allow(non_snake_case)]

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
/// Source: `oracle/codemp/cgame/cg_view.c:223-232,1396-1397,1476,2009,2277-2281,2445`
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

    /// Raven `cgscreffects_t cgScreenEffects` — the screen-shake and
    /// music-ducking state the `CG_SE_*` / `CGCam_*` fns drive.
    /// Source: `oracle/codemp/cgame/cg_view.c:2009`
    pub cgScreenEffects: cgscreffects_t,
}

impl Default for CgViewState {
    /// Raven's zeroed BSS — every one of these is an uninitialized file-scope
    /// global.
    fn default() -> Self {
        CgViewState {
            camerafwd: [0.0; 3],
            cameraFocusLoc: [0.0; 3],
            cameraIdealTarget: [0.0; 3],
            cameraIdealLoc: [0.0; 3],
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
        }
    }
}
