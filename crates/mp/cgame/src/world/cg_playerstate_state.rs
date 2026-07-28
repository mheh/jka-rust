//! `CgPlayerstateState` — `cg_playerstate.c`'s mutable file-scope globals as
//! one `CgWorld` sub-struct.

#![allow(non_snake_case)]

use core::ffi::c_int;

/// `cg_playerstate.c`'s mutable file-scope globals, grouped by owning `.c`
/// file (§B3: file-scope globals become owned state, they never become Rust
/// globals).
///
/// Source: `oracle/codemp/cgame/cg_playerstate.c:304`
#[derive(Debug, Clone, Default)]
pub struct CgPlayerstateState {
    /// Raven: to prevent announce sounds from playing on top of each other.
    ///
    /// Gates the timelimit/fraglimit warning blocks in `CG_CheckLocalSounds`
    /// until `cg.time` passes it; each fired warning pushes it 3s out.
    /// Source: `oracle/codemp/cgame/cg_playerstate.c:304`
    pub cgAnnouncerTime: c_int,
}
