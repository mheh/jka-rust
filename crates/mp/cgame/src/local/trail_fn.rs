//! The missile-trail think dispatch.

#![allow(non_camel_case_types, non_snake_case)]

use crate::fx_blaster::FX_BlasterProjectileThink;
use crate::fx_bowcaster::{FX_BowcasterAltProjectileThink, FX_BowcasterProjectileThink};
use crate::fx_bryarpistol::{
    FX_BryarAltProjectileThink, FX_BryarProjectileThink, FX_ConcussionProjectileThink,
    FX_TurretProjectileThink,
};
use crate::fx_demp2::FX_DEMP2_ProjectileThink;
use crate::fx_flechette::{FX_FlechetteAltProjectileThink, FX_FlechetteProjectileThink};
use crate::fx_heavyrepeater::{FX_RepeaterAltProjectileThink, FX_RepeaterProjectileThink};
use crate::fx_rocketlauncher::{FX_RocketAltProjectileThink, FX_RocketProjectileThink};
use crate::local::centity_s::centity_t;
use crate::local::weapon_info_s::weaponInfo_t;
use crate::world::cg_context::CgContext;

/// Raven's `weaponInfo_t.missileTrailFunc`/`altMissileTrailFunc` fn pointers
/// as the DEC-47.6 closed enum (DEC-46.4, the `leType` precedent): the only
/// values Raven ever stores are the `FX_*ProjectileThink` set below and `0`,
/// so the port records *which* and [`Self::dispatch`] is the one call site.
///
/// `None = 0` is Raven's null pointer - it keeps `weaponInfo_t` all-zero-valid
/// for the `CgWorld::new_boxed` zero fill.
///
/// - `None`: Raven's `= 0` stores - nothing to play.
/// - `Concussion`: `FX_ConcussionProjectileThink`.
/// - `Bryar`: `FX_BryarProjectileThink`.
/// - `BryarAlt`: `FX_BryarAltProjectileThink`.
/// - `Blaster`: `FX_BlasterProjectileThink`.
/// - `Bowcaster`: `FX_BowcasterProjectileThink`.
/// - `BowcasterAlt`: `FX_BowcasterAltProjectileThink`.
/// - `Repeater`: `FX_RepeaterProjectileThink`.
/// - `RepeaterAlt`: `FX_RepeaterAltProjectileThink`.
/// - `Demp2`: `FX_DEMP2_ProjectileThink`.
/// - `Flechette`: `FX_FlechetteProjectileThink`.
/// - `FlechetteAlt`: `FX_FlechetteAltProjectileThink`.
/// - `Rocket`: `FX_RocketProjectileThink`.
/// - `RocketAlt`: `FX_RocketAltProjectileThink`.
/// - `Turret`: `FX_TurretProjectileThink`.
///
/// Source: `oracle/codemp/cgame/cg_local.h:206,215` (the two fn-ptr fields);
/// stores at `oracle/codemp/cgame/cg_weaponinit.c:159-581`
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
#[repr(u8)]
pub enum TrailFn {
    #[default]
    None = 0,
    Concussion,
    Bryar,
    BryarAlt,
    Blaster,
    Bowcaster,
    BowcasterAlt,
    Repeater,
    RepeaterAlt,
    Demp2,
    Flechette,
    FlechetteAlt,
    Rocket,
    RocketAlt,
    Turret,
}

impl TrailFn {
    /// Raven's `weapon->missileTrailFunc( cent, weapon )` indirect call.
    /// `cent` is `&mut` for `FX_RepeaterAltProjectileThink`'s sake (the one
    /// mutating think); the rest take the shared reborrow.
    pub fn dispatch(self, ctx: &mut CgContext, cent: &mut centity_t, weapon: &weaponInfo_t) {
        match self {
            TrailFn::None => {}
            TrailFn::Concussion => FX_ConcussionProjectileThink(ctx, cent, weapon),
            TrailFn::Bryar => FX_BryarProjectileThink(ctx, cent, weapon),
            TrailFn::BryarAlt => FX_BryarAltProjectileThink(ctx, cent, weapon),
            TrailFn::Blaster => FX_BlasterProjectileThink(ctx, cent, weapon),
            TrailFn::Bowcaster => FX_BowcasterProjectileThink(ctx, cent, weapon),
            TrailFn::BowcasterAlt => FX_BowcasterAltProjectileThink(ctx, cent, weapon),
            TrailFn::Repeater => FX_RepeaterProjectileThink(ctx, cent, weapon),
            TrailFn::RepeaterAlt => FX_RepeaterAltProjectileThink(ctx, cent, weapon),
            TrailFn::Demp2 => FX_DEMP2_ProjectileThink(ctx, cent, weapon),
            TrailFn::Flechette => FX_FlechetteProjectileThink(ctx, cent, weapon),
            TrailFn::FlechetteAlt => FX_FlechetteAltProjectileThink(ctx, cent, weapon),
            TrailFn::Rocket => FX_RocketProjectileThink(ctx, cent, weapon),
            TrailFn::RocketAlt => FX_RocketAltProjectileThink(ctx, cent, weapon),
            TrailFn::Turret => FX_TurretProjectileThink(ctx, cent, weapon),
        }
    }
}
