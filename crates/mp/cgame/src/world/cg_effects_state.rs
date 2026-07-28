//! `CgEffectsState` — `cg_effects.c`'s mutable file-scope globals as one `CgWorld`
//! sub-struct.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_qshared::shared::{qhandle_t, vec3_t};

use crate::cg_effects::{
    NUM_DEBRIS_MODELS_CHUNKS, NUM_DEBRIS_MODELS_GLASS, NUM_DEBRIS_MODELS_ROCKS,
    NUM_DEBRIS_MODELS_WOOD,
};

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

    /// Raven `int dbModels_Glass[NUM_DEBRIS_MODELS_GLASS]` — `CG_CreateDebris`'s
    /// once-registered debris models for the glass special-case (repurposed
    /// for metal chunks per Raven's own comment: "glass no longer exists,
    /// using it for metal").
    /// Source: `oracle/codemp/cgame/cg_effects.c:766`
    pub dbModels_Glass: [qhandle_t; NUM_DEBRIS_MODELS_GLASS],

    /// Raven `int dbModels_Wood[NUM_DEBRIS_MODELS_WOOD]` — `CG_CreateDebris`'s
    /// once-registered debris models for the wood/crate special-case.
    /// Source: `oracle/codemp/cgame/cg_effects.c:767`
    pub dbModels_Wood: [qhandle_t; NUM_DEBRIS_MODELS_WOOD],

    /// Raven `int dbModels_Chunks[NUM_DEBRIS_MODELS_CHUNKS]` —
    /// `CG_CreateDebris`'s once-registered debris models for the generic-chunk
    /// special-case. Only the first two of the three slots are ever written
    /// (Raven registers just `chunks_1`/`chunks_2`); the BSS-zero third slot
    /// stays a null model handle if `Q_irand` ever rolls it.
    /// Source: `oracle/codemp/cgame/cg_effects.c:768`
    pub dbModels_Chunks: [qhandle_t; NUM_DEBRIS_MODELS_CHUNKS],

    /// Raven `int dbModels_Rocks[NUM_DEBRIS_MODELS_ROCKS]` —
    /// `CG_CreateDebris`'s once-registered debris models for the rock
    /// special-case.
    /// Source: `oracle/codemp/cgame/cg_effects.c:769`
    pub dbModels_Rocks: [qhandle_t; NUM_DEBRIS_MODELS_ROCKS],
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
            dbModels_Glass: [0; NUM_DEBRIS_MODELS_GLASS],
            dbModels_Wood: [0; NUM_DEBRIS_MODELS_WOOD],
            dbModels_Chunks: [0; NUM_DEBRIS_MODELS_CHUNKS],
            dbModels_Rocks: [0; NUM_DEBRIS_MODELS_ROCKS],
        }
    }
}
