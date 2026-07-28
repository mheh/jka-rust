//! `CgEffectsState` — `cg_effects.c`'s mutable file-scope globals as one `CgWorld`
//! sub-struct.

#![allow(non_snake_case)]

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
#[derive(Debug, Clone, Default)]
pub struct CgEffectsState {
    /// Raven `static float offX[20][20]` — `CG_InitGlass`'s random
    /// crack-offset table, built once and read by the (not-yet-ported) glass
    /// tesselator.
    /// Source: `oracle/codemp/cgame/cg_effects.c:254-255`
    pub offX: [[f32; 20]; 20],

    /// Raven `static float offZ[20][20]` — `CG_InitGlass`'s random
    /// crack-offset table, built once and read by the (not-yet-ported) glass
    /// tesselator.
    /// Source: `oracle/codemp/cgame/cg_effects.c:254-255`
    pub offZ: [[f32; 20]; 20],
}
