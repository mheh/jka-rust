//! `FxPrimitive` — the closed `CEffect` hierarchy as one enum (DEC-61.1).
//!
//! Raven dispatched `Update` and `Draw` through virtuals on twelve concrete
//! classes. The enum matches instead, and each variant embeds the core blocks
//! its class inherited.
//!
//! Source: `oracle/codemp/client/FxPrimitives.h:108-608`

#![allow(non_camel_case_types, non_snake_case)]

use crate::fx::cbezier::CBezier;
use crate::fx::ccylinder::CCylinder;
use crate::fx::ceffect::EffectCore;
use crate::fx::celectricity::CElectricity;
use crate::fx::cemitter::CEmitter;
use crate::fx::cflash::CFlash;
use crate::fx::clight::CLight;
use crate::fx::cline::CLine;
use crate::fx::coriented_particle::COrientedParticle;
use crate::fx::cparticle::ParticleCore;
use crate::fx::cpoly::CPoly;
use crate::fx::ctail::CTail;
use crate::fx::ctrail::CTrail;
use crate::fx::fx_host::FxHost;
use crate::fx::fx_system::FxSystem;

/// One live effect primitive.
///
/// The variant order follows Raven's declaration order in `FxPrimitives.h`.
///
/// - `Trail`: the saber slash trail, fed straight from cgame.
/// - `Light`: a dynamic light, the one variant that draws no geometry.
/// - `Particle`: the plain sprite, and the base every moving variant embeds.
/// - `Flash`: a full screen or localized flash.
/// - `Line`: a two-point beam.
/// - `Bezier`: a four-point curve drawn as a quad strip.
/// - `Electricity`: a lightning beam with chaos, branching, and taper.
/// - `OrientedParticle`: a quad with its own normal.
/// - `Tail`: a comet-like streak.
/// - `Cylinder`: a tapered tube, optionally trace-terminated.
/// - `Emitter`: a moving spawner that can carry an attached model.
/// - `Poly`: a rotating three-to-five-vertex polygon.
#[derive(Clone, Debug)]
pub enum FxPrimitive {
    Trail(CTrail),
    Light(CLight),
    Particle(ParticleCore),
    Flash(CFlash),
    Line(CLine),
    Bezier(CBezier),
    Electricity(CElectricity),
    OrientedParticle(COrientedParticle),
    Tail(CTail),
    Cylinder(CCylinder),
    Emitter(CEmitter),
    Poly(CPoly),
}

impl FxPrimitive {
    /// The `CEffect` block every variant carries.
    pub fn core(&self) -> &EffectCore {
        match self {
            FxPrimitive::Trail(p) => &p.e,
            FxPrimitive::Light(p) => &p.e,
            FxPrimitive::Particle(p) => &p.e,
            FxPrimitive::Flash(p) => &p.p.e,
            FxPrimitive::Line(p) => &p.p.e,
            FxPrimitive::Bezier(p) => &p.l.p.e,
            FxPrimitive::Electricity(p) => &p.l.p.e,
            FxPrimitive::OrientedParticle(p) => &p.p.e,
            FxPrimitive::Tail(p) => &p.p.e,
            FxPrimitive::Cylinder(p) => &p.t.p.e,
            FxPrimitive::Emitter(p) => &p.p.e,
            FxPrimitive::Poly(p) => &p.p.e,
        }
    }

    /// The `CEffect` block every variant carries, for writing.
    pub fn core_mut(&mut self) -> &mut EffectCore {
        match self {
            FxPrimitive::Trail(p) => &mut p.e,
            FxPrimitive::Light(p) => &mut p.e,
            FxPrimitive::Particle(p) => &mut p.e,
            FxPrimitive::Flash(p) => &mut p.p.e,
            FxPrimitive::Line(p) => &mut p.p.e,
            FxPrimitive::Bezier(p) => &mut p.l.p.e,
            FxPrimitive::Electricity(p) => &mut p.l.p.e,
            FxPrimitive::OrientedParticle(p) => &mut p.p.e,
            FxPrimitive::Tail(p) => &mut p.p.e,
            FxPrimitive::Cylinder(p) => &mut p.t.p.e,
            FxPrimitive::Emitter(p) => &mut p.p.e,
            FxPrimitive::Poly(p) => &mut p.p.e,
        }
    }

    /// Raven's `virtual bool Update()`.
    ///
    /// A `false` return marks the primitive for death this frame.
    pub fn Update(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) -> bool {
        match self {
            FxPrimitive::Trail(p) => p.Update(host, fx),
            FxPrimitive::Light(p) => p.Update(host, fx),
            FxPrimitive::Particle(p) => p.Update(host, fx),
            FxPrimitive::Flash(p) => p.Update(host, fx),
            FxPrimitive::Line(p) => p.Update(host, fx),
            FxPrimitive::Bezier(p) => p.Update(host, fx),
            FxPrimitive::Electricity(p) => p.Update(host, fx),
            FxPrimitive::OrientedParticle(p) => p.Update(host, fx),
            FxPrimitive::Tail(p) => p.Update(host, fx),
            FxPrimitive::Cylinder(p) => p.Update(host, fx),
            FxPrimitive::Emitter(p) => p.Update(host, fx),
            FxPrimitive::Poly(p) => p.Update(host, fx),
        }
    }

    /// Raven's `virtual void Die()`.
    ///
    /// Only the particle family runs a death effect. `CLine`, `CElectricity`, and
    /// `CBezier` override it back to nothing, and `CEffect`'s own body is empty.
    pub fn Die(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) {
        match self {
            FxPrimitive::Particle(p) => p.Die(host, fx),
            FxPrimitive::Flash(p) => p.p.Die(host, fx),
            FxPrimitive::OrientedParticle(p) => p.p.Die(host, fx),
            FxPrimitive::Tail(p) => p.p.Die(host, fx),
            FxPrimitive::Cylinder(p) => p.t.p.Die(host, fx),
            FxPrimitive::Emitter(p) => p.p.Die(host, fx),
            FxPrimitive::Poly(p) => p.p.Die(host, fx),
            FxPrimitive::Trail(_)
            | FxPrimitive::Light(_)
            | FxPrimitive::Line(_)
            | FxPrimitive::Bezier(_)
            | FxPrimitive::Electricity(_) => {}
        }
    }
}
