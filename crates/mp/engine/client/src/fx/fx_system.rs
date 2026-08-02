//! `FxSystem` — the one owned instance of Raven's FX globals (DEC-61.2).
//!
//! Raven spread this state over `theFxHelper`, `theFxScheduler`, the
//! `effectList` array, and four file-scope counters in `FxUtil.cpp`.
//! `Engine.fx` owns all of it, and the trap arms thread it in.
//!
//! Source: `oracle/codemp/client/FxUtil.cpp:11-28`,
//! `oracle/codemp/client/FxSystem.h:49-221`

#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use mp_qshared::shared::cvar::CvarHandle;
use native_math::vector::vec3_t;

use crate::fx::fx_clock::FxClock;
use crate::fx::fx_host::FxHost;
use crate::fx::fx_scheduler::CFxScheduler;
use crate::fx::fx_util::{SEffectList, MAX_EFFECTS};

/// The refdef fields the FX system reads, copied out of the module's `cg.refdef`.
///
/// Raven keeps a live `refdef_t*` into cgame memory and reads it every frame.
/// Nothing inside one FX trap writes `cg.refdef`, so a snapshot taken at the trap
/// entry gives the same values.
#[derive(Clone, Copy, Debug, Default)]
pub struct FxRefdef {
    pub vieworg: vec3_t,
    pub viewangles: vec3_t,
    pub viewaxis: [vec3_t; 3],
    pub fov_x: f32,
    pub fov_y: f32,
}

/// The client effect engine: templates, the schedule, the live pool, and the clock.
///
/// `Engine.fx` holds the one instance and it is `None` on a dedicated server.
/// The FX trap arms cast the view slot back with `fx_from_view` and thread it in.
pub struct FxSystem {
    /// Raven `theFxHelper`'s time block.
    pub clock: FxClock,

    /// The module's `cg.refdef`, handed over by `CG_FX_INIT_SYSTEM`.
    /// Null in the parity rig, which fills `refdef` directly.
    pub refdef_ptr: *mut refdef_t,
    /// The snapshot every read inside a trap uses.
    pub refdef: FxRefdef,

    /// `fx_debug` value for this trap.
    pub fx_debug: i32,
    /// `fx_countScale` value for this trap.
    pub fx_countScale: f32,
    /// `fx_nearCull` value for this trap.
    pub fx_nearCull: f32,
    /// `com_RMG` value for this trap, and whether the cvar exists at all.
    pub com_RMG: i32,
    pub com_RMG_present: bool,

    /// The registered cvar handles, filled by `FX_Init`.
    pub cvar_fx_debug: Option<CvarHandle>,
    pub cvar_fx_countScale: Option<CvarHandle>,
    pub cvar_fx_nearCull: Option<CvarHandle>,

    /// Raven `theFxScheduler`.
    pub scheduler: CFxScheduler,

    /// Raven `effectList` — `MAX_EFFECTS` slots, walked in index order every frame.
    pub effects: Vec<SEffectList>,
    /// Raven `nextValidEffect`, as a slot index instead of a pointer.
    pub next_valid_effect: usize,
    /// Raven `activeFx`.
    pub activeFx: i32,
    /// Raven `drawnFx`.
    pub drawnFx: i32,
    /// Raven `fxInitialized`.
    pub fxInitialized: bool,
    /// Raven `gEffectsInPortal`, set for the duration of the portal pass.
    pub gEffectsInPortal: bool,

    /// Raven `totalPrimitives` and `totalEffects`, the two spawn counters.
    pub totalPrimitives: i32,
    pub totalEffects: i32,
}

impl Default for FxSystem {
    fn default() -> Self {
        FxSystem {
            clock: FxClock::default(),
            refdef_ptr: core::ptr::null_mut(),
            refdef: FxRefdef::default(),
            fx_debug: 0,
            fx_countScale: 1.0,
            fx_nearCull: 16.0,
            com_RMG: 0,
            com_RMG_present: false,
            cvar_fx_debug: None,
            cvar_fx_countScale: None,
            cvar_fx_nearCull: None,
            scheduler: CFxScheduler::default(),
            effects: vec![SEffectList::default(); MAX_EFFECTS],
            next_valid_effect: 0,
            activeFx: 0,
            drawnFx: 0,
            fxInitialized: false,
            gEffectsInPortal: false,
            totalPrimitives: 0,
            totalEffects: 0,
        }
    }
}

impl FxSystem {
    /// Copy the module's live `cg.refdef` and the three FX cvars into the snapshot.
    ///
    /// Every FX trap arm calls this first, so the interior reads plain fields.
    pub fn refresh(&mut self, host: &mut FxHost<'_, '_>) {
        if !self.refdef_ptr.is_null() {
            // SAFETY: the module handed this pointer over at `CG_FX_INIT_SYSTEM`
            // and it addresses its own `cg.refdef` for the module's lifetime.
            let r = unsafe { &*self.refdef_ptr };
            self.refdef = FxRefdef {
                vieworg: r.vieworg,
                viewangles: r.viewangles,
                viewaxis: r.viewaxis,
                fov_x: r.fov_x,
                fov_y: r.fov_y,
            };
        }

        if let FxHost::Engine { view, .. } = host {
            if let Some(h) = self.cvar_fx_debug {
                self.fx_debug = view.common.cvar(h).integer;
            }
            if let Some(h) = self.cvar_fx_countScale {
                self.fx_countScale = view.common.cvar(h).value;
            }
            if let Some(h) = self.cvar_fx_nearCull {
                self.fx_nearCull = view.common.cvar(h).value;
            }
        }
    }
}
