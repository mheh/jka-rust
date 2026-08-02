//! Raven `CTail`, a comet-like streak drawn as a line from the origin backwards.
//!
//! The tail keeps last frame's origin so it can point the streak along the real
//! direction of travel, and `mLength` runs its own fade curve.
//!
//! Class definition source: `oracle/codemp/client/FxPrimitives.h:469-495`
//! Method source: `oracle/codemp/client/FxPrimitives.cpp:945-1131`

#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::cgame::tr_types::RF_DEPTHHACK;
use native_math::vector::vec3_t;

use crate::fx::cparticle::{
    fade_curve, vector_ma, vector_ma_in_place, vector_normalize, vector_scale, vector_subtract,
    ParticleCore,
};
use crate::fx::fx_flags::{
    FX_DEPTH_HACK, FX_LENGTH_CLAMP, FX_LENGTH_LINEAR, FX_LENGTH_NONLINEAR, FX_LENGTH_PARM_MASK,
    FX_LENGTH_RAND, FX_LENGTH_WAVE, FX_RELATIVE,
};
use crate::fx::fx_host::FxHost;
use crate::fx::fx_system::FxSystem;

/// The `CTail` fields, plus the `CParticle` core it inherited.
///
/// Class definition source: `oracle/codemp/client/FxPrimitives.h:469-495`
#[derive(Clone, Copy, Debug)]
pub struct CTail {
    pub p: ParticleCore,

    pub mOldOrigin: vec3_t,

    pub mLengthStart: f32,
    pub mLengthEnd: f32,
    pub mLengthParm: f32,

    pub mLength: f32,
}

impl Default for CTail {
    /// Raven `CTail::CTail`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:945-948`
    fn default() -> Self {
        let mut p = ParticleCore::default();
        p.e.mRefEnt.reType = refEntityType_t::RT_LINE;
        CTail {
            p,
            mOldOrigin: [0.0; 3],
            mLengthStart: 0.0,
            mLengthEnd: 0.0,
            mLengthParm: 0.0,
            mLength: 0.0,
        }
    }
}

impl CTail {
    /// Source: `oracle/codemp/client/FxPrimitives.h:492`
    pub fn SetLengthStart(&mut self, len: f32) {
        self.mLengthStart = len;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:493`
    pub fn SetLengthEnd(&mut self, len: f32) {
        self.mLengthEnd = len;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:494`
    pub fn SetLengthParm(&mut self, len: f32) {
        self.mLengthParm = len;
    }

    /// Raven `CTail::Draw`. The endpoint comes from `CalcNewEndpoint`, not from here.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:951-963`
    pub fn Draw(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) {
        if self.p.e.mFlags & FX_DEPTH_HACK != 0 {
            // Not sure if first person needs to be set
            self.p.e.mRefEnt.renderfx |= RF_DEPTHHACK;
        }

        self.p.e.mRefEnt.origin = self.p.e.mOrigin1;

        host.AddFxToScene(Some(&self.p.e.mRefEnt));
        fx.drawnFx += 1;
    }

    /// Raven `CTail::Update`.
    ///
    /// Raven reads `org` and `ax` uninitialized when `FX_RELATIVE` is set but the
    /// model or bolt index is negative. The port pins both to zero, which is the
    /// one defined behavior for that path (porting-rules §19).
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:966-1045`
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

            let mut org: vec3_t = [0.0; 3];
            let mut ax: [vec3_t; 3] = [[0.0; 3]; 3];
            if self.p.mModelNum >= 0 && self.p.mBoltNum >= 0 {
                //bolt style
                let Some((bolt_org, bolt_ax)) = self.p.fetch_bolt(host, fx) else {
                    // could not get bolt
                    return false;
                };
                org = bolt_org;
                ax = bolt_ax;
            }

            vector_ma_in_place(&mut org, self.p.mOrgOffset[0], &ax[0]);
            vector_ma_in_place(&mut org, self.p.mOrgOffset[1], &ax[1]);
            vector_ma_in_place(&mut org, self.p.mOrgOffset[2], &ax[2]);

            // calc the real velocity and accel vectors
            // FIXME: if you want right and up movement in addition to the forward movement, you'll have to convert dir into a set of perp. axes and do some extra work
            let mut real_vel = vector_scale(&ax[0], self.p.mVel[0]);
            vector_ma_in_place(&mut real_vel, self.p.mVel[1], &ax[1]);
            vector_ma_in_place(&mut real_vel, self.p.mVel[2], &ax[2]);

            let mut real_accel = vector_scale(&ax[0], self.p.mAccel[0]);
            vector_ma_in_place(&mut real_accel, self.p.mAccel[1], &ax[1]);
            vector_ma_in_place(&mut real_accel, self.p.mAccel[2], &ax[2]);

            let time = (fx.clock.mTime - self.p.e.mTimeStart) as f32 * 0.001;

            // Take acceleration into account for the velocity at this time.
            vector_ma_in_place(&mut real_vel, time, &real_accel);

            // Now move us to where we should be at the given time
            self.p.e.mOrigin1 = vector_ma(&org, time, &real_vel);

            // Just calc an old point some time in the past, doesn't really matter when
            self.mOldOrigin = vector_ma(&org, time - 0.003, &real_vel);
        } else {
            // The `_SOF2DEV_` freeze check is not in the retail MP build.
            self.mOldOrigin = self.p.e.mOrigin1;
        }

        if self.p.e.mTimeStart < fx.clock.mTime && !self.p.UpdateOrigin(host, fx) {
            // we are marked for death
            return false;
        }

        if !self.p.Cull(fx) {
            // Only update these if the thing is visible.
            self.p.UpdateSize(host, fx);
            self.UpdateLength(host, fx);
            self.p.UpdateRGB(host, fx);
            self.p.UpdateAlpha(host, fx);

            self.CalcNewEndpoint();
            self.Draw(host, fx);
        }

        true
    }

    /// Raven `CTail::UpdateLength`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:1048-1116`
    pub fn UpdateLength(&mut self, host: &mut FxHost<'_, '_>, fx: &FxSystem) {
        let mut perc1 = fade_curve(
            self.p.e.mFlags,
            fx.clock.mTime,
            self.p.e.mTimeStart,
            self.p.e.mTimeEnd,
            self.mLengthParm,
            FX_LENGTH_LINEAR,
            FX_LENGTH_PARM_MASK,
            FX_LENGTH_NONLINEAR,
            FX_LENGTH_WAVE,
            FX_LENGTH_CLAMP,
        );

        // If needed, RAND can coexist with linear and either non-linear or wave.
        if self.p.e.mFlags & FX_LENGTH_RAND != 0 {
            perc1 = host.rng().flrand(0.0, perc1);
        }

        self.mLength = (self.mLengthStart * perc1) + (self.mLengthEnd * (1.0 - perc1));
    }

    /// Raven `CTail::CalcNewEndpoint`, the streak tail point.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:1120-1131`
    pub fn CalcNewEndpoint(&mut self) {
        // FIXME:  Hmmm, this looks dumb when physics are on and a bounce happens
        let mut temp = vector_subtract(&self.mOldOrigin, &self.p.e.mOrigin1);

        // I wish we didn't have to do a VectorNormalize every frame.....
        vector_normalize(&mut temp);

        self.p.e.mRefEnt.oldorigin = vector_ma(&self.p.e.mOrigin1, self.mLength, &temp);
    }
}
