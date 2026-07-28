//! `CgEffectsState` — `cg_effects.c`'s mutable file-scope globals as one `CgWorld`
//! sub-struct.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_qshared::shared::vec3_t;

/// `cg_effects.c`'s mutable file-scope globals, grouped by owning `.c` file
/// (§B3: file-scope globals become owned state, they never become Rust
/// globals).
///
/// Fields fold in as the waves transcribe `cg_effects.c`'s file-scope statics
/// (DEC-46.1), so a wave transcriber only ever touches its own TU's two files
/// — the function file and this one — and never `cg_world.rs`. Raven's
/// read-only tables beside them are compiled-in data, not state; they land as
/// `const`s beside the functions that read them (§C8).
///
/// Source: `oracle/codemp/cgame/cg_effects.c:766-769`
#[derive(Debug, Clone)]
pub struct CgEffectsState {
    /// Raven `static float offX[20][20]` — `CG_InitGlass`'s random
    /// crack-offset table, built once and read by `CG_DoGlass`'s glass
    /// tesselator.
    /// Source: `oracle/codemp/cgame/cg_effects.c:254-255`
    pub offX: [[f32; 20]; 20],

    /// Raven `static float offZ[20][20]` — `CG_InitGlass`'s random
    /// crack-offset table, built once and read by `CG_DoGlass`'s glass
    /// tesselator.
    /// Source: `oracle/codemp/cgame/cg_effects.c:254-255`
    pub offZ: [[f32; 20]; 20],

    /// Raven's `static int seed = 0x92` inside `CG_SmokePuff` — the `Q_random`
    /// stream that picks each puff's sprite rotation. Its own generator, not
    /// the shared `holdrand`/CRT one, so it stays a separate field.
    /// Source: `oracle/codemp/cgame/cg_effects.c:86`
    pub seed: c_int,

    /// Raven's `static vec3_t lastPos` inside `CG_ScorePlum` — where the last
    /// plum spawned, so two plums close in height get stacked instead of
    /// overlapping.
    /// Source: `oracle/codemp/cgame/cg_effects.c:1265`
    pub lastPos: vec3_t,
}

impl Default for CgEffectsState {
    /// Zeroed BSS everywhere except `seed`, which Raven gives a compile-time
    /// `0x92`.
    fn default() -> Self {
        CgEffectsState {
            offX: [[0.0; 20]; 20],
            offZ: [[0.0; 20]; 20],
            seed: 0x92,
            lastPos: [0.0; 3],
        }
    }
}
