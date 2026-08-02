//! Raven `CEmitter`, a moving spawner that trails other effects behind it.
//!
//! Raven derives it from `CParticle` so a spawned effect can borrow the
//! emitter's current alpha and color. The emitter draws nothing of its own
//! unless `FX_ATTACHED_MODEL` gives it a model to carry.
//!
//! Class definition source: `oracle/codemp/client/FxPrimitives.h:532-569`
//! Method source: `oracle/codemp/client/FxPrimitives.cpp:1317-1484`

#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use native_math::qmath::AnglesToAxis;
use native_math::vector::vec3_t;
use native_types::qtrue;

use crate::fx::cparticle::{
    distance_squared, vector_compare, vector_ma, vector_ma_in_place, vector_scale, ParticleCore,
};
use crate::fx::fx_flags::{FX_ATTACHED_MODEL, FX_EMIT_FX, FX_PAPER_PHYSICS, FX_RELATIVE};
use crate::fx::fx_host::FxHost;
use crate::fx::fx_scheduler::fx_play_effect_axis;
use crate::fx::fx_system::FxSystem;

/// Raven: we "think" at about a 60hz rate.
///
/// Source: `oracle/codemp/client/FxPrimitives.cpp:1357`
const TRAIL_RATE: i32 = 12;

/// The `CEmitter` fields, plus the `CParticle` core it inherited.
///
/// Class definition source: `oracle/codemp/client/FxPrimitives.h:532-569`
#[derive(Clone, Copy, Debug)]
pub struct CEmitter {
    pub p: ParticleCore,

    /// we use these to do some nice
    pub mOldOrigin: vec3_t,
    /// tricks...
    pub mLastOrigin: vec3_t,
    pub mOldVelocity: vec3_t,
    pub mOldTime: i32,

    /// for a rotating thing, using a delta
    pub mAngles: vec3_t,
    /// as opposed to an end angle is probably much easier
    pub mAngleDelta: vec3_t,

    /// if we have emitter fx, this is our id
    pub mEmitterFxID: i32,

    /// controls how often emitter chucks an effect
    pub mDensity: f32,
    /// density sloppiness
    pub mVariance: f32,
}

impl Default for CEmitter {
    /// Raven `CEmitter::CEmitter`.
    ///
    /// Raven: there may or may not be a model, but if there isn't one, we just
    /// won't bother adding the refEnt in our Draw func.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:1317-1322`
    fn default() -> Self {
        let mut p = ParticleCore::default();
        p.e.mRefEnt.reType = refEntityType_t::RT_MODEL;
        CEmitter {
            p,
            mOldOrigin: [0.0; 3],
            mLastOrigin: [0.0; 3],
            mOldVelocity: [0.0; 3],
            mOldTime: 0,
            mAngles: [0.0; 3],
            mAngleDelta: [0.0; 3],
            mEmitterFxID: 0,
            mDensity: 0.0,
            mVariance: 0.0,
        }
    }
}

impl CEmitter {
    /// Source: `oracle/codemp/client/FxPrimitives.h:560`
    pub fn SetModel(&mut self, model: i32) {
        self.p.e.mRefEnt.hModel = model;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:561`
    pub fn SetAngles(&mut self, ang: Option<vec3_t>) {
        self.mAngles = ang.unwrap_or([0.0; 3]);
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:562`
    pub fn SetAngleDelta(&mut self, ang: Option<vec3_t>) {
        self.mAngleDelta = ang.unwrap_or([0.0; 3]);
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:563`
    pub fn SetEmitterFxID(&mut self, id: i32) {
        self.mEmitterFxID = id;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:564`
    pub fn SetDensity(&mut self, density: f32) {
        self.mDensity = density;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:565`
    pub fn SetVariance(&mut self, var: f32) {
        self.mVariance = var;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:566`
    pub fn SetOldTime(&mut self, time: i32) {
        self.mOldTime = time;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:567`
    pub fn SetLastOrg(&mut self, org: Option<vec3_t>) {
        self.mLastOrigin = org.unwrap_or([0.0; 3]);
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:568`
    pub fn SetLastVel(&mut self, vel: Option<vec3_t>) {
        self.mOldVelocity = vel.unwrap_or([0.0; 3]);
    }

    /// Raven `CEmitter::Cull`. An emitter never culls.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.h:557`
    pub fn Cull(&self) -> bool {
        false
    }

    /// Raven `CEmitter::Draw`, the attached model plus the trail spawn loop.
    ///
    /// The loop walks from `mOldTime` to now in fixed `TRAIL_RATE` steps and
    /// spawns one effect each time the emitter has covered the target distance.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:1332-1399`
    pub fn Draw(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) {
        // Emitters don't draw themselves, but they may need to add an attached model
        if self.p.e.mFlags & FX_ATTACHED_MODEL != 0 {
            self.p.e.mRefEnt.nonNormalizedAxes = qtrue;

            self.p.e.mRefEnt.origin = self.p.e.mOrigin1;

            let radius = self.p.e.mRefEnt.radius;
            self.p.e.mRefEnt.axis[0] = vector_scale(&self.p.e.mRefEnt.axis[0], radius);
            self.p.e.mRefEnt.axis[1] = vector_scale(&self.p.e.mRefEnt.axis[1], radius);
            self.p.e.mRefEnt.axis[2] = vector_scale(&self.p.e.mRefEnt.axis[2], radius);

            // I hate having to do this, but this needs to get added as a regular refEntity
            host.AddFxToScene(None);
            host.AddFxToScene(Some(&self.p.e.mRefEnt));
        }

        // If we are emitting effects, we had better be careful because just calling it every cgame frame could
        //	either choke up the effects system on a fast machine, or look really nasty on a low end one.
        if self.p.e.mFlags & FX_EMIT_FX != 0 {
            // Pick a target step distance and square it
            let mut step = self.mDensity + host.rng().flrand(-self.mVariance, self.mVariance);
            step *= step;

            let mut dif = 0;

            let mut t = self.mOldTime;
            while t <= fx.clock.mTime {
                dif += TRAIL_RATE;

                // ?Not sure if it's better to update this before or after updating the origin
                let v = vector_ma(&self.mOldVelocity, dif as f32 * 0.001, &self.p.mAccel);

                // Calc the time differences
                let ftime = dif as f32 * 0.001;
                let time2 = ftime * ftime * 0.5;

                // Predict the new position
                let org: vec3_t = [
                    self.mOldOrigin[0] + (ftime * v[0]) + (time2 * v[0]),
                    self.mOldOrigin[1] + (ftime * v[1]) + (time2 * v[1]),
                    self.mOldOrigin[2] + (ftime * v[2]) + (time2 * v[2]),
                ];

                // Is it time to draw an effect?
                if distance_squared(&org, &self.mOldOrigin) >= step {
                    // Pick a new target step distance and square it
                    step = self.mDensity + host.rng().flrand(-self.mVariance, self.mVariance);
                    step *= step;

                    // We met the step criteria so, we should add in the effect
                    let axis = self.p.e.mRefEnt.axis;
                    fx_play_effect_axis(
                        fx,
                        host,
                        self.mEmitterFxID,
                        Some(org),
                        axis,
                        -1,
                        0,
                        -1,
                        -1,
                        -1,
                        false,
                        0,
                        false,
                    );

                    self.mOldOrigin = org;
                    self.mOldVelocity = v;
                    dif = 0;
                    self.mOldTime = t;
                }

                t += TRAIL_RATE;
            }
        }
        fx.drawnFx += 1;
    }

    /// Raven `CEmitter::Update`.
    ///
    /// Raven's `FX_RELATIVE` arm is an `assert(0)` with the question "need
    /// this?". The retail build compiles the assert out and falls straight
    /// through, so the port carries the validity check and nothing else.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:1402-1477`
    pub fn Update(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) -> bool {
        // Game pausing can cause dumb time things to happen, so kill the effect in this instance
        if self.p.e.mTimeStart > fx.clock.mTime {
            return false;
        }

        // Use this to track if we've stopped moving
        self.mOldOrigin = self.p.e.mOrigin1;
        self.mOldVelocity = self.p.mVel;

        if self.p.e.mFlags & FX_RELATIVE != 0 && !self.p.bolt_still_valid(host) {
            // the thing we are bolted to is no longer valid, so we may as well just die.
            return false;
        }

        if self.p.e.mTimeStart < fx.clock.mTime && !self.p.UpdateOrigin(host, fx) {
            // we are marked for death
            return false;
        }

        let mut moving = false;

        // If the thing is no longer moving, kill the angle delta, but don't do it too quickly or it will
        //	look very artificial.  Don't do it too slowly or it will look like there is no friction.
        if vector_compare(&self.mOldOrigin, &self.p.e.mOrigin1) {
            self.mAngleDelta = vector_scale(&self.mAngleDelta, 0.7);
        } else {
            moving = true;
        }

        if self.p.e.mFlags & FX_PAPER_PHYSICS != 0 {
            // do this in a more framerate independant manner
            let mut sc = 20.0 / fx.clock.mFrameTime as f32;

            // bah, evil clamping
            if sc >= 1.0 {
                sc = 1.0;
            }

            if moving {
                // scale the velocity down, basically inducing drag.  Acceleration ( gravity ) should keep it pulling down, which is what we want.
                self.p.mVel = vector_scale(&self.p.mVel, (sc * 0.8 + 0.2) * 0.92);

                // add some chaotic motion based on the way we are oriented
                let axis0 = self.p.e.mRefEnt.axis[0];
                let axis1 = self.p.e.mRefEnt.axis[1];
                vector_ma_in_place(&mut self.p.mVel, (1.5 - sc) * 4.0, &axis0);
                vector_ma_in_place(&mut self.p.mVel, (1.5 - sc) * 4.0, &axis1);
            }

            // make us settle so we can lay flat
            self.mAngles[0] *= 0.97 * (sc * 0.4 + 0.6);
            self.mAngles[2] *= 0.97 * (sc * 0.4 + 0.6);

            // decay our angle delta so we don't rotate as quickly
            self.mAngleDelta = vector_scale(&self.mAngleDelta, 0.96 * (sc * 0.1 + 0.9));
        }

        self.UpdateAngles(fx);
        self.p.UpdateSize(host, fx);

        self.Draw(host, fx);

        true
    }

    /// Raven `CEmitter::UpdateAngles`.
    ///
    /// Raven: was 0.001f, but then you really have to jack up the delta to even
    /// notice anything.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:1480-1484`
    pub fn UpdateAngles(&mut self, fx: &FxSystem) {
        let delta = self.mAngleDelta;
        vector_ma_in_place(&mut self.mAngles, fx.clock.mFrameTime as f32 * 0.01, &delta);
        // SAFETY: `AnglesToAxis` takes Raven's `vec3_t*` and writes the three rows
        // the refEntity axis owns.
        AnglesToAxis(self.mAngles, self.p.e.mRefEnt.axis.as_mut_ptr());
    }
}
