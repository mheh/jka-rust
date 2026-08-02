//! Raven `COrientedParticle`, a quad that faces its own normal instead of the view.
//!
//! When the primitive is bolted, `mNormal` stops being a normal and becomes a
//! pitch, yaw, and roll offset applied on top of the bolt axis.
//!
//! Class definition source: `oracle/codemp/client/FxPrimitives.h:450-466`
//! Method source: `oracle/codemp/client/FxPrimitives.cpp:640-776`

#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::cgame::tr_types::RF_DEPTHHACK;
use native_math::qmath::{AnglesToAxis, MakeNormalVectors, MatrixMultiply};
use native_math::vector::vec3_t;

use crate::fx::cparticle::{
    dot_product, vector_ma, vector_ma_in_place, vector_scale, vector_subtract, ParticleCore,
};
use crate::fx::fx_flags::{FX_DEPTH_HACK, FX_RELATIVE};
use crate::fx::fx_host::FxHost;
use crate::fx::fx_system::FxSystem;

/// The `COrientedParticle` fields, plus the `CParticle` core it inherited.
///
/// Class definition source: `oracle/codemp/client/FxPrimitives.h:450-466`
#[derive(Clone, Copy, Debug)]
pub struct COrientedParticle {
    pub p: ParticleCore,

    pub mNormal: vec3_t,
}

impl Default for COrientedParticle {
    /// Raven `COrientedParticle::COrientedParticle`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:640-643`
    fn default() -> Self {
        let mut p = ParticleCore::default();
        p.e.mRefEnt.reType = refEntityType_t::RT_ORIENTED_QUAD;
        COrientedParticle {
            p,
            mNormal: [0.0; 3],
        }
    }
}

impl COrientedParticle {
    /// Source: `oracle/codemp/client/FxPrimitives.h:465`
    pub fn SetNormal(&mut self, norm: vec3_t) {
        self.mNormal = norm;
    }

    /// Raven `COrientedParticle::Cull`. Only the behind-the-viewer test survives.
    ///
    /// Raven commented the near-cull out, so the depth-hack branch and the tail
    /// both answer `false`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:646-674`
    pub fn Cull(&self, fx: &FxSystem) -> bool {
        // Get the direction to the view
        let dir = vector_subtract(&self.p.e.mOrigin1, &fx.refdef.vieworg);

        // Check if it's behind the viewer
        if dot_product(&fx.refdef.viewaxis[0], &dir) < 0.0 {
            return true;
        }

        // don't cull stuff that's associated with inview wpns
        if self.p.e.mFlags & FX_DEPTH_HACK != 0 {
            return false;
        }

        false
    }

    /// Raven `COrientedParticle::Draw`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:677-695`
    pub fn Draw(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) {
        if self.p.e.mFlags & FX_DEPTH_HACK != 0 {
            // Not sure if first person needs to be set
            self.p.e.mRefEnt.renderfx |= RF_DEPTHHACK;
        }

        // Add our refEntity to the scene
        self.p.e.mRefEnt.origin = self.p.e.mOrigin1;
        if self.p.e.mFlags & FX_RELATIVE == 0 {
            self.p.e.mRefEnt.axis[0] = self.mNormal;

            let forward = self.p.e.mRefEnt.axis[0];
            let mut right: vec3_t = [0.0; 3];
            let mut up: vec3_t = [0.0; 3];
            MakeNormalVectors(forward, &mut right, &mut up);
            self.p.e.mRefEnt.axis[1] = right;
            self.p.e.mRefEnt.axis[2] = up;
        }

        host.AddFxToScene(Some(&self.p.e.mRefEnt));
        fx.drawnFx += 1;
    }

    /// Raven `COrientedParticle::Update`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:700-776`
    pub fn Update(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) -> bool {
        // Game pausing can cause dumb time things to happen, so kill the effect in this instance
        if self.p.e.mTimeStart > fx.clock.mTime {
            return false;
        }

        if self.p.e.mFlags & FX_RELATIVE != 0 {
            if !self.p.bolt_still_valid(host) {
                // the thing we are bolted to is no longer valid, so we may as well just die.
                return false;
            }

            // Get our current position and direction
            let Some((mut org, ax)) = self.p.fetch_bolt(host, fx) else {
                // could not get bolt
                return false;
            };

            vector_ma_in_place(&mut org, self.p.mOrgOffset[0], &ax[0]);
            vector_ma_in_place(&mut org, self.p.mOrgOffset[1], &ax[1]);
            vector_ma_in_place(&mut org, self.p.mOrgOffset[2], &ax[2]);

            let time = (fx.clock.mTime - self.p.e.mTimeStart) as f32 * 0.001;
            // calc the real velocity and accel vectors
            let mut real_vel = vector_scale(&ax[0], self.p.mVel[0]);
            vector_ma_in_place(&mut real_vel, self.p.mVel[1], &ax[1]);
            vector_ma_in_place(&mut real_vel, self.p.mVel[2], &ax[2]);

            let mut real_accel = vector_scale(&ax[0], self.p.mAccel[0]);
            vector_ma_in_place(&mut real_accel, self.p.mAccel[1], &ax[1]);
            vector_ma_in_place(&mut real_accel, self.p.mAccel[2], &ax[2]);

            // Take acceleration into account for the velocity at this time.
            vector_ma_in_place(&mut real_vel, time, &real_accel);

            // Now move us to where we should be at the given time
            self.p.e.mOrigin1 = vector_ma(&org, time, &real_vel);

            //use the normalOffset and add that to the actual normal of the bolt
            //NOTE: not tested!!!
            self.p.e.mRefEnt.axis[0] = ax[0];
            self.p.e.mRefEnt.axis[1] = ax[1];
            self.p.e.mRefEnt.axis[2] = ax[2];

            let mut offset_axis: [vec3_t; 3] = [[0.0; 3]; 3];
            //NOTE: mNormal is actually PITCH YAW and ROLL offsets
            // SAFETY: `AnglesToAxis` takes Raven's `vec3_t*` and writes the three
            // rows this array owns.
            AnglesToAxis(self.mNormal, offset_axis.as_mut_ptr());
            MatrixMultiply(&offset_axis, &ax, &mut self.p.e.mRefEnt.axis);
        } else if self.p.e.mTimeStart < fx.clock.mTime && !self.p.UpdateOrigin(host, fx) {
            // we are marked for death
            return false;
        }

        if !self.Cull(fx) {
            // Only update these if the thing is visible.
            self.p.UpdateSize(host, fx);
            self.p.UpdateRGB(host, fx);
            self.p.UpdateAlpha(host, fx);
            self.p.UpdateRotation(fx);

            self.Draw(host, fx);
        }

        true
    }
}
