//! `CgPlayersState` — `cg_players.c`'s mutable file-scope globals as one `CgWorld`
//! sub-struct.

#![allow(non_snake_case)]

use core::ffi::c_void;
use core::ptr::null_mut;

/// `cg_players.c`'s mutable file-scope globals, grouped by owning `.c` file
/// (§B3: file-scope globals become owned state, they never become Rust
/// globals).
///
/// Empty at C5 skeleton time by design: fields fold in as the waves transcribe
/// `cg_players.c`'s file-scope statics (DEC-46.1), so a wave transcriber only ever
/// touches its own TU's two files — the function file and this one — and never
/// `cg_world.rs`. Raven's read-only tables beside them are compiled-in data,
/// not state; they land as `const`s beside the functions that read them (§C8).
///
/// Source: `oracle/codemp/cgame/cg_players.c:1945,3362,7583,7887,7978,7980,8338`
#[derive(Debug, Clone)]
pub struct CgPlayersState {
    /// Raven `qboolean cgQueueLoad` — set by `CG_LoadDeferredPlayers`, drained
    /// at the top of `CG_Player`.
    /// Source: `oracle/codemp/cgame/cg_players.c:1945`
    pub cgQueueLoad: bool,

    /// Raven `void *cg_g2JetpackInstance` — the one shared jetpack ghoul2
    /// instance, built by `CG_InitJetpackGhoul2` and copied onto each jetpacking
    /// player. An opaque engine token, never dereferenced module-side
    /// (DEC-46.2).
    /// Source: `oracle/codemp/cgame/cg_players.c:7887`
    pub cg_g2JetpackInstance: *mut c_void,
}

impl Default for CgPlayersState {
    /// Raven's BSS start: the queue flag clear and the jetpack instance NULL.
    /// Hand-written because a raw pointer has no derivable `Default`.
    fn default() -> Self {
        CgPlayersState {
            cgQueueLoad: false,
            cg_g2JetpackInstance: null_mut(),
        }
    }
}
