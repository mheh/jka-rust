// PORT-COMPLETE: bg_slidemove.c 0/5
//! FAITHFUL signature skeleton for `oracle/oracle/codemp/game/bg_slidemove.c`.
//!
//! Every function in this file is built on the file-static pmove working set
//! (`pmove_t *pm`, `pml_t pml`, `bgEntity_t *pm_entSelf`) exactly like
//! `bg_pmove.c` (see `bg_pmove.rs`'s module doc). Porting-rules §B3 forbids
//! `static mut`/hidden globals, but the faithful C signatures here thread no
//! `pm`/engine context, so the representation of that working set is the same
//! genuine unsettled design fork already parked across `bg_pmove.rs` — every
//! function below reads and/or writes `pm`/`pml`/`pm_entSelf` and is parked
//! the same way, for consistency with that precedent.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

/// Raven `PM_VehicleImpact`.
///
/// Source: `oracle/oracle/codemp/game/bg_slidemove.c:49-557`
// PORT-ESCALATION(pmove-working-state): reads/writes `pm`/`pml`/`g_entities`; needs
// the pmove working-set threading decision (see bg_pmove.rs). Also has large
// QAGAME-only vehicle-collision/damage logic depending on unported g_combat.c /
// g_vehicles.c / FighterNPC.c bodies (G_Damage, G_DamageFromKiller,
// G_FlyVehicleSurfaceDestruction, FighterIsLanded, G_CanBeEnemy) — a second,
// independent reason to park (bg-dep).
pub fn PM_VehicleImpact(
    pEnt: *mut bgEntity_t,
    trace: *mut trace_t,
) {
    todo!("Port PM_VehicleImpact — parked: pmove-working-state")
}

/// Raven `PM_GroundSlideOkay`.
///
/// Source: `oracle/oracle/codemp/game/bg_slidemove.c:559-580`
// PORT-ESCALATION(pmove-working-state): reads `pm->ps->velocity`/`pm->ps->legsAnim`;
// needs the pmove working-set threading decision (see bg_pmove.rs).
pub fn PM_GroundSlideOkay(
    zNormal: f32,
) -> qboolean {
    todo!("Port PM_GroundSlideOkay — parked: pmove-working-state")
}

/// Raven `PM_ClientImpact`.
///
/// Source: `oracle/oracle/codemp/game/bg_slidemove.c:590-623`
// PORT-ESCALATION(pmove-working-state): reads `pm`/`pm_entSelf`/`g_entities`/`level`;
// needs the pmove working-set threading decision (see bg_pmove.rs).
pub fn PM_ClientImpact(
    trace: *mut trace_t,
) -> qboolean {
    todo!("Port PM_ClientImpact — parked: pmove-working-state")
}

/// Raven `PM_SlideMove`.
///
/// Source: `oracle/oracle/codemp/game/bg_slidemove.c:634-853`
// PORT-ESCALATION(pmove-working-state): reads/writes `pm`/`pml`/`pm_entSelf`; needs
// the pmove working-set threading decision (see bg_pmove.rs).
pub fn PM_SlideMove(
    gravity: qboolean,
) -> qboolean {
    todo!("Port PM_SlideMove — parked: pmove-working-state")
}

/// Raven `PM_StepSlideMove`.
///
/// Source: `oracle/oracle/codemp/game/bg_slidemove.c:861-1073`
// PORT-ESCALATION(pmove-working-state): reads/writes `pm`/`pm_entSelf`/`c_pmove`;
// needs the pmove working-set threading decision (see bg_pmove.rs).
pub fn PM_StepSlideMove(
    gravity: qboolean,
) {
    todo!("Port PM_StepSlideMove — parked: pmove-working-state")
}
