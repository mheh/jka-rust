//! `CgPlayersState` — `cg_players.c`'s mutable file-scope globals as one `CgWorld`
//! sub-struct.

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};
use core::ptr::null_mut;

use mp_qshared::shared::MAX_GENTITIES;

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

    /// Raven `static int lastFlyBySound[MAX_GENTITIES]` — per-vehicle debounce
    /// on the flyby whoosh, stamped with `cg.time`.
    ///
    /// Its only reader is `CG_VehicleEffects`' flyby block, which is deferred on
    /// the `Vehicle_t` referent pool (the sound handles live on
    /// `m_pVehicle->m_pVehicleInfo`); the state lands now so the block only has
    /// to be filled in when the pool arrives.
    /// Source: `oracle/codemp/cgame/cg_players.c:7978`
    pub lastFlyBySound: [c_int; MAX_GENTITIES],

    /// Raven `int cg_lastHyperSpaceEffectTime` — when we last threw the
    /// hyperspace-stars effect, so a second jump doesn't replay it.
    /// Source: `oracle/codemp/cgame/cg_players.c:7980`
    pub cg_lastHyperSpaceEffectTime: c_int,
}

impl Default for CgPlayersState {
    /// Raven's BSS start: the queue flag clear and the jetpack instance NULL.
    /// Hand-written because a raw pointer has no derivable `Default`.
    fn default() -> Self {
        CgPlayersState {
            cgQueueLoad: false,
            cg_g2JetpackInstance: null_mut(),
            lastFlyBySound: [0; MAX_GENTITIES],
            cg_lastHyperSpaceEffectTime: 0,
        }
    }
}
