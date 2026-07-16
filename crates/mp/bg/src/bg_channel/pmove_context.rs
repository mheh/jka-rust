//! `PmoveContext` — the per-`Pmove`-call bg working set.
//!
//! Raven's `bg_pmove.c` kept its working state in file-scope statics
//! (`pmove_t *pm`, `pml_t pml`, `bgEntity_t *pm_entSelf`/`pm_entVeh`,
//! `pm_flying`, `gPMDoSlowFall`, `pm_cancelOutZoom`), reset at the top of every
//! `Pmove` call (`pm = pmove` in `PmoveSingle`, `bg_pmove.c:10180`). §B3 forbids
//! that; this struct is the bg-owned replacement, constructed once per `Pmove`
//! and threaded through the move pipeline. The game tier builds it (handing in
//! `&mut BgState` + the two trait objects); no game type leaks below game.
#![allow(non_snake_case)]

use core::ffi::c_int;

use crate::prelude::*;

use super::bg_state::BgState;
use super::bg_traps::BgTraps;
use super::game_callbacks::GameCallbacks;
// `pml_t` (pmove-local scratch) is not in the game prelude; import it directly.
use crate::local::pml_t::pml_t;

/// The pmove working set for one `Pmove` call plus the bg channel handles.
///
/// The raw `pm`/`pm_entSelf`/`pm_entVeh` pointers are the faithful pmove seam
/// (`bgEntity_t` access stays the `baseEnt`/`entSize` overlay); the
/// unsafe that dereferences them is confined to the pmove methods.
pub struct PmoveContext<'a> {
    /// Raven `pmove_t *pm` — the in/out move block, assigned per call.
    /// Source: `oracle/codemp/game/bg_pmove.c:30`
    pub pm: *mut pmove_t,
    /// Raven `pml_t pml` — pmove-local scratch (BSS-zeroed each call).
    /// Source: `oracle/codemp/game/bg_pmove.c:31`
    pub pml: pml_t,
    /// Raven `bgEntity_t *pm_entSelf`.
    /// Source: `oracle/codemp/game/bg_pmove.c:33`
    pub pm_entSelf: *mut bgEntity_t,
    /// Raven `bgEntity_t *pm_entVeh`.
    /// Source: `oracle/codemp/game/bg_pmove.c:34`
    pub pm_entVeh: *mut bgEntity_t,
    /// Raven `static int pm_flying` (`FLY_NONE`/`FLY_NORMAL`/`FLY_VEHICLE`/`FLY_HOVER`).
    /// Source: `oracle/codemp/game/bg_pmove.c:445`
    pub pm_flying: c_int,
    /// Raven `qboolean gPMDoSlowFall`.
    /// Source: `oracle/codemp/game/bg_pmove.c:36`
    pub gPMDoSlowFall: qboolean,
    /// Raven `qboolean pm_cancelOutZoom`.
    /// Source: `oracle/codemp/game/bg_pmove.c:38`
    pub pm_cancelOutZoom: qboolean,

    /// Session-lifetime bg state (anim/saber/vehicle tables + RNG), threaded in
    /// by the game-tier caller.
    pub bg: &'a mut BgState,
    /// The outbound engine surface.
    pub traps: &'a dyn BgTraps,
    /// The bg→game upcall surface.
    pub callbacks: &'a mut dyn GameCallbacks,
}

impl<'a> PmoveContext<'a> {
    /// Build a fresh per-call context. Mirrors Raven's file-static reset: `pm`
    /// null until `PmoveSingle` assigns it, `pml` BSS-zeroed, `pm_flying =
    /// FLY_NONE`, the slow-fall/zoom flags cleared.
    /// Source: `oracle/codemp/game/bg_pmove.c:30-38,445`
    pub fn new(
        bg: &'a mut BgState,
        traps: &'a dyn BgTraps,
        callbacks: &'a mut dyn GameCallbacks,
    ) -> Self {
        Self {
            pm: core::ptr::null_mut(),
            // Faithful BSS-zero init of the pmove-local scratch (`pml_t` is a
            // `#[repr(C)]` POD scratch struct; C zero-inits the file static).
            pml: unsafe { core::mem::zeroed() },
            pm_entSelf: core::ptr::null_mut(),
            pm_entVeh: core::ptr::null_mut(),
            pm_flying: 0, // FLY_NONE
            gPMDoSlowFall: qfalse,
            pm_cancelOutZoom: qfalse,
            bg,
            traps,
            callbacks,
        }
    }
}
