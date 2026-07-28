//! `CgScoreboardState` — `cg_scoreboard.c`'s mutable file-scope globals as one `CgWorld`
//! sub-struct.

#![allow(non_snake_case)]

use core::ffi::c_int;

/// `cg_scoreboard.c`'s mutable file-scope globals, grouped by owning `.c` file
/// (§B3: file-scope globals become owned state, they never become Rust
/// globals).
///
/// Fields fold in as the waves transcribe `cg_scoreboard.c`'s file-scope
/// statics (DEC-46.1), so a wave transcriber only ever touches its own TU's
/// two files — the function file and this one — and never `cg_world.rs`.
/// Raven's read-only tables beside them are compiled-in data, not state; they
/// land as `const`s beside the functions that read them (§C8).
///
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:54,344`
#[derive(Debug, Clone, Default)]
pub struct CgScoreboardState {
    /// Raven `static qboolean localClient` — set once `CG_DrawClientScore`
    /// draws the row belonging to the local client's own `clientNum`.
    ///
    /// Source: `oracle/codemp/cgame/cg_scoreboard.c:54`
    pub localClient: bool,

    /// Raven `int cg_siegeWinTeam` — which side won the siege round, off
    /// `CS_SIEGE_WINTEAM` (1 or 2); `cg_main.c` writes it.
    /// Source: `oracle/codemp/cgame/cg_scoreboard.c:344`
    pub cg_siegeWinTeam: c_int,
}
