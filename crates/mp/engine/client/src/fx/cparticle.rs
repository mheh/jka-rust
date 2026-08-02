//! Raven `CParticle`, the sprite primitive and the base every moving primitive shares.
//!
//! DEC-61.1 composes instead of inheriting, so `ParticleCore` holds the
//! `CParticle` fields and the update helpers that `CLine`, `CTail`, `CCylinder`,
//! `CEmitter`, `CPoly`, `CFlash`, and `COrientedParticle` all call.
//! `FxPrimitive::Particle` is a bare `ParticleCore`, because `CParticle` adds no
//! fields of its own beyond the two cores.
//!
//! Raven's `~CParticle` calls `mGhoul2.kill()`, which drops the handle without
//! freeing the arena slot. Dropping a `ParticleCore` does exactly that already,
//! so no teardown ports.
//!
//! Class definition source: `oracle/codemp/client/FxPrimitives.h:267-348`
//! Method source: `oracle/codemp/client/FxPrimitives.cpp:44-632`

#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::cgame::tr_types::RF_DEPTHHACK;
use mp_qshared::shared::surface_flags::{MASK_PLAYERSOLID, MASK_SOLID, SURF_NOIMPACT};
use mp_qshared::shared::ENTITYNUM_WORLD;
use native_math::vector::vec3_t;

use crate::fx::ceffect::EffectCore;
use crate::fx::fx_flags::{
    FX_ALPHA_CLAMP, FX_ALPHA_LINEAR, FX_ALPHA_NONLINEAR, FX_ALPHA_PARM_MASK, FX_ALPHA_RAND,
    FX_ALPHA_WAVE, FX_APPLY_PHYSICS, FX_DEATH_RUNS_FX, FX_DEPTH_HACK, FX_EXPENSIVE_PHYSICS,
    FX_GHOUL2_TRACE, FX_IMPACT_RUNS_FX, FX_KILL_ON_IMPACT, FX_PLAYER_VIEW, FX_RELATIVE,
    FX_RGB_CLAMP, FX_RGB_LINEAR, FX_RGB_NONLINEAR, FX_RGB_PARM_MASK, FX_RGB_RAND, FX_RGB_WAVE,
    FX_SIZE_CLAMP, FX_SIZE_LINEAR, FX_SIZE_NONLINEAR, FX_SIZE_PARM_MASK, FX_SIZE_RAND,
    FX_SIZE_WAVE, FX_USE_ALPHA, FX_USE_BBOX,
};
use crate::fx::fx_host::FxHost;
use crate::fx::fx_scheduler::{fx_add_2d_effect, fx_play_effect_fwd};
use crate::fx::fx_system::FxSystem;

/// The `CParticle` fields, plus the `CEffect` core it inherited.
///
/// The five `Update*` helpers below are the shared fade machinery: every derived
/// primitive calls the same size, RGB, alpha, and rotation curves.
#[derive(Clone, Copy, Debug)]
pub struct ParticleCore {
    pub e: EffectCore,

    pub mOrgOffset: vec3_t,

    pub mVel: vec3_t,
    pub mAccel: vec3_t,

    pub mSizeStart: f32,
    pub mSizeEnd: f32,
    pub mSizeParm: f32,

    pub mRGBStart: vec3_t,
    pub mRGBEnd: vec3_t,
    pub mRGBParm: f32,

    pub mAlphaStart: f32,
    pub mAlphaEnd: f32,
    pub mAlphaParm: f32,

    pub mRotationDelta: f32,
    pub mElasticity: f32,

    /// Raven holds a `CGhoul2Info_v`, which is one `int` handle. The port keeps
    /// the handle itself, so the primitive stays `Copy` and needs no destructor.
    /// Raven's `mGhoul2.kill()` clears the handle without freeing the arena slot,
    /// which is what dropping this field already does.
    pub mGhoul2: i32,
    pub mEntNum: i16,
    pub mModelNum: i8,
    pub mBoltNum: i8,
}

impl Default for ParticleCore {
    /// Raven `CParticle::CParticle` — a sprite with no bolt.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.h:310-313`
    fn default() -> Self {
        let mut e = EffectCore::default();
        e.mRefEnt.reType = refEntityType_t::RT_SPRITE;
        ParticleCore {
            e,
            mOrgOffset: [0.0; 3],
            mVel: [0.0; 3],
            mAccel: [0.0; 3],
            mSizeStart: 0.0,
            mSizeEnd: 0.0,
            mSizeParm: 0.0,
            mRGBStart: [0.0; 3],
            mRGBEnd: [0.0; 3],
            mRGBParm: 0.0,
            mAlphaStart: 0.0,
            mAlphaEnd: 0.0,
            mAlphaParm: 0.0,
            mRotationDelta: 0.0,
            mElasticity: 0.0,
            mGhoul2: 0,
            mEntNum: -1,
            mModelNum: -1,
            mBoltNum: -1,
        }
    }
}

impl ParticleCore {
    /// Source: `oracle/codemp/client/FxPrimitives.h:305-308`
    pub fn SetBoltinfo(&mut self, i_ghoul2: i32, ent_num: i32, model_num: i32, bolt_num: i32) {
        self.mGhoul2 = i_ghoul2;
        self.mEntNum = ent_num as i16;
        self.mModelNum = model_num as i8;
        self.mBoltNum = bolt_num as i8;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:327`
    pub fn SetShader(&mut self, sh: i32) {
        self.e.mRefEnt.customShader = sh;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:329`
    pub fn SetOrgOffset(&mut self, o: Option<vec3_t>) {
        self.mOrgOffset = o.unwrap_or([0.0; 3]);
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:330`
    pub fn SetVel(&mut self, vel: Option<vec3_t>) {
        self.mVel = vel.unwrap_or([0.0; 3]);
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:331`
    pub fn SetAccel(&mut self, ac: Option<vec3_t>) {
        self.mAccel = ac.unwrap_or([0.0; 3]);
    }

    /// Also seeds the render radius, which the size curve overwrites each frame.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.h:333`
    pub fn SetSizeStart(&mut self, sz: f32) {
        self.mSizeStart = sz;
        self.e.mRefEnt.radius = sz;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:334`
    pub fn SetSizeEnd(&mut self, sz: f32) {
        self.mSizeEnd = sz;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:335`
    pub fn SetSizeParm(&mut self, parm: f32) {
        self.mSizeParm = parm;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:337`
    pub fn SetRGBStart(&mut self, rgb: Option<vec3_t>) {
        self.mRGBStart = rgb.unwrap_or([0.0; 3]);
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:338`
    pub fn SetRGBEnd(&mut self, rgb: Option<vec3_t>) {
        self.mRGBEnd = rgb.unwrap_or([0.0; 3]);
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:339`
    pub fn SetRGBParm(&mut self, parm: f32) {
        self.mRGBParm = parm;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:341`
    pub fn SetAlphaStart(&mut self, al: f32) {
        self.mAlphaStart = al;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:342`
    pub fn SetAlphaEnd(&mut self, al: f32) {
        self.mAlphaEnd = al;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:343`
    pub fn SetAlphaParm(&mut self, parm: f32) {
        self.mAlphaParm = parm;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:345`
    pub fn SetRotation(&mut self, rot: f32) {
        self.e.mRefEnt.rotation = rot;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:346`
    pub fn SetRotationDelta(&mut self, rot: f32) {
        self.mRotationDelta = rot;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:347`
    pub fn SetElasticity(&mut self, el: f32) {
        self.mElasticity = el;
    }

    /// Raven `CParticle::Init` — a player-view particle starts at a random screen point.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:44-52`
    pub fn Init(&mut self, host: &mut FxHost<'_, '_>) {
        self.e.mRefEnt.radius = 0.0;
        if self.e.mFlags & FX_PLAYER_VIEW != 0 {
            self.e.mOrigin1[0] = host.rng().irand(0, 639) as f32;
            self.e.mOrigin1[1] = host.rng().irand(0, 479) as f32;
        }
    }

    /// Raven `CParticle::Die` — fire the death effect along a random normal.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:55-67`
    pub fn Die(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) {
        if self.e.mFlags & FX_DEATH_RUNS_FX != 0 && self.e.mFlags & FX_KILL_ON_IMPACT == 0 {
            // Man, this just seems so, like, uncool and stuff...
            let mut norm: vec3_t = [
                host.rng().flrand(-1.0, 1.0),
                host.rng().flrand(-1.0, 1.0),
                host.rng().flrand(-1.0, 1.0),
            ];
            vector_normalize(&mut norm);

            let origin = self.e.mOrigin1;
            fx_play_effect_fwd(fx, host, self.e.mDeathFxID, origin, norm, -1, -1, false);
        }
    }

    /// Raven `CParticle::Cull`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:70-102`
    pub fn Cull(&self, fx: &FxSystem) -> bool {
        if self.e.mFlags & FX_PLAYER_VIEW != 0 {
            // this will be drawn as a 2D effect so don't cull it
            return false;
        }

        // Get the direction to the view
        let dir = vector_subtract(&self.e.mOrigin1, &fx.refdef.vieworg);

        // Check if it's behind the viewer
        if dot_product(&fx.refdef.viewaxis[0], &dir) < 0.0 {
            return true;
        }

        // don't cull if this is hacked to show up close to the inview wpn
        if self.e.mFlags & FX_DEPTH_HACK != 0 {
            return false;
        }
        // Can't be too close
        let len = vector_length_squared(&dir);
        if len < fx.fx_nearCull {
            return true;
        }

        false
    }

    /// Raven `CParticle::Draw`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:105-133`
    pub fn Draw(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) {
        if self.e.mFlags & FX_DEPTH_HACK != 0 {
            self.e.mRefEnt.renderfx |= RF_DEPTHHACK;
        }

        if self.e.mFlags & FX_PLAYER_VIEW != 0 {
            let color = [
                self.e.mRefEnt.shaderRGBA[0] as f32 / 255.0,
                self.e.mRefEnt.shaderRGBA[1] as f32 / 255.0,
                self.e.mRefEnt.shaderRGBA[2] as f32 / 255.0,
                self.e.mRefEnt.shaderRGBA[3] as f32 / 255.0,
            ];

            // add this 2D effect to the proper list. it will get drawn after the RenderScene call
            fx_add_2d_effect(
                fx,
                self.e.mOrigin1[0],
                self.e.mOrigin1[1],
                self.e.mRefEnt.radius,
                self.e.mRefEnt.radius,
                color,
                self.e.mRefEnt.customShader,
            );
        } else {
            // Add our refEntity to the scene
            self.e.mRefEnt.origin = self.e.mOrigin1;
            host.AddFxToScene(Some(&self.e.mRefEnt));
        }
        fx.drawnFx += 1;
    }

    /// Raven `CParticle::Update`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:138-204`
    pub fn Update(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) -> bool {
        // Game pausing can cause dumb time things to happen, so kill the effect in this instance
        if self.e.mTimeStart > fx.clock.mTime {
            return false;
        }

        if self.e.mFlags & FX_RELATIVE != 0 {
            if !self.bolt_still_valid(host) {
                // the thing we are bolted to is no longer valid, so we may as well just die.
                return false;
            }

            let Some((mut org, ax)) = self.fetch_bolt(host, fx) else {
                return false;
            };

            vector_ma_in_place(&mut org, self.mOrgOffset[0], &ax[0]);
            vector_ma_in_place(&mut org, self.mOrgOffset[1], &ax[1]);
            vector_ma_in_place(&mut org, self.mOrgOffset[2], &ax[2]);

            let time = (fx.clock.mTime - self.e.mTimeStart) as f32 * 0.001;
            // calc the real velocity and accel vectors
            let mut real_vel = vector_scale(&ax[0], self.mVel[0]);
            vector_ma_in_place(&mut real_vel, self.mVel[1], &ax[1]);
            vector_ma_in_place(&mut real_vel, self.mVel[2], &ax[2]);

            let mut real_accel = vector_scale(&ax[0], self.mAccel[0]);
            vector_ma_in_place(&mut real_accel, self.mAccel[1], &ax[1]);
            vector_ma_in_place(&mut real_accel, self.mAccel[2], &ax[2]);

            // Take acceleration into account for the velocity at this time.
            vector_ma_in_place(&mut real_vel, time, &real_accel);

            // Now move us to where we should be at the given time
            self.e.mOrigin1 = vector_ma(&org, time, &real_vel);
        } else if self.e.mTimeStart < fx.clock.mTime && !self.UpdateOrigin(host, fx) {
            // we are marked for death
            return false;
        }

        if !self.Cull(fx) {
            // Only update these if the thing is visible.
            self.UpdateSize(host, fx);
            self.UpdateRGB(host, fx);
            self.UpdateAlpha(host, fx);
            self.UpdateRotation(fx);

            self.Draw(host, fx);
        }

        true
    }

    /// Whether the ghoul2 instance this primitive bolts to still exists.
    ///
    /// Raven writes `mGhoul2.IsValid()`.
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:148`
    pub fn bolt_still_valid(&self, host: &mut FxHost<'_, '_>) -> bool {
        host.Ghoul2IsValid(self.mGhoul2)
    }

    /// Ask the bolt where it is this frame.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:157`
    pub fn fetch_bolt(
        &mut self,
        host: &mut FxHost<'_, '_>,
        fx: &FxSystem,
    ) -> Option<(vec3_t, [vec3_t; 3])> {
        let ent = self.mEntNum as i32;
        let model = self.mModelNum as i32;
        let bolt = self.mBoltNum as i32;
        let old_time = fx.clock.mOldTime;
        host.GetOriginAxisFromBolt(self.mGhoul2, ent, model, bolt, old_time)
    }

    /// Raven `CParticle::UpdateOrigin` — move, then optionally bounce off the world.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:209-348`
    pub fn UpdateOrigin(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) -> bool {
        vector_ma_in_place(&mut self.mVel, fx.clock.mRealTime, &self.mAccel.clone());

        // Predict the new position
        let new_origin: vec3_t = [
            self.e.mOrigin1[0] + (fx.clock.mRealTime * self.mVel[0]),
            self.e.mOrigin1[1] + (fx.clock.mRealTime * self.mVel[1]),
            self.e.mOrigin1[2] + (fx.clock.mRealTime * self.mVel[2]),
        ];

        // Only perform physics if this object is tagged to do so
        if (self.e.mFlags & FX_APPLY_PHYSICS) != 0 && (self.e.mFlags & FX_PLAYER_VIEW) == 0 {
            let solid;

            if self.e.mFlags & FX_EXPENSIVE_PHYSICS != 0 {
                // by setting this to true, we force a real trace to happen
                solid = true;
            } else if !fx.com_RMG_present || fx.com_RMG != 0 {
                // don't do this call for RMG maps
                solid = (host.PointContents(new_origin, ENTITYNUM_WORLD) & MASK_SOLID) != 0;
            } else {
                solid = false;
            }

            if solid {
                let trace = if self.e.mFlags & FX_USE_BBOX != 0 {
                    host.Trace(
                        self.e.mOrigin1,
                        Some(self.e.mMin),
                        Some(self.e.mMax),
                        new_origin,
                        -1,
                        MASK_SOLID,
                        self.e.mFlags & FX_GHOUL2_TRACE != 0,
                    )
                } else if self.e.mFlags & FX_GHOUL2_TRACE != 0 {
                    host.Trace(
                        self.e.mOrigin1,
                        None,
                        None,
                        new_origin,
                        -1,
                        MASK_PLAYERSOLID,
                        true,
                    )
                } else {
                    host.Trace(
                        self.e.mOrigin1,
                        None,
                        None,
                        new_origin,
                        -1,
                        MASK_SOLID,
                        false,
                    )
                };

                // Hit something
                if trace.startsolid != 0 || trace.allsolid != 0 {
                    self.mVel = [0.0; 3];
                    self.mAccel = [0.0; 3];

                    if (self.e.mFlags & FX_GHOUL2_TRACE) != 0
                        && (self.e.mFlags & FX_IMPACT_RUNS_FX) != 0
                    {
                        let bs_normal: vec3_t = [0.0, 1.0, 0.0];
                        fx_play_effect_fwd(
                            fx,
                            host,
                            self.e.mImpactFxID,
                            trace.endpos,
                            bs_normal,
                            -1,
                            -1,
                            false,
                        );
                    }

                    self.e.mFlags &= !(FX_APPLY_PHYSICS | FX_IMPACT_RUNS_FX);

                    return true;
                } else if trace.fraction < 1.0 {
                    if self.e.mFlags & FX_IMPACT_RUNS_FX != 0
                        && (trace.surfaceFlags & SURF_NOIMPACT) == 0
                    {
                        fx_play_effect_fwd(
                            fx,
                            host,
                            self.e.mImpactFxID,
                            trace.endpos,
                            trace.plane.normal,
                            -1,
                            -1,
                            false,
                        );
                    }

                    // Raven's `MaterialImpact` body is entirely commented out, so the
                    // port drops the call rather than transcribing an empty function.
                    // Source: oracle/codemp/client/FxScheduler.cpp:660-682

                    if self.e.mFlags & FX_KILL_ON_IMPACT != 0 {
                        // time to die
                        return false;
                    }

                    vector_ma_in_place(
                        &mut self.mVel,
                        fx.clock.mRealTime * trace.fraction,
                        &self.mAccel.clone(),
                    );

                    let dot = dot_product(&self.mVel, &trace.plane.normal);

                    vector_ma_in_place(&mut self.mVel, -2.0 * dot, &trace.plane.normal);

                    self.mVel = vector_scale(&self.mVel, self.mElasticity);
                    self.mElasticity *= 0.5;

                    // If the velocity is too low, make it stop moving, rotating, and turn off
                    // physics to avoid doing expensive operations when they aren't needed
                    if vector_length_squared(&self.mVel) < 100.0 {
                        self.mVel = [0.0; 3];
                        self.mAccel = [0.0; 3];

                        self.e.mFlags &= !(FX_APPLY_PHYSICS | FX_IMPACT_RUNS_FX);
                    }

                    // Set the origin to the exact impact point
                    self.e.mOrigin1 = vector_ma(&trace.endpos, 1.0, &trace.plane.normal);
                    return true;
                }
            }
        }

        // No physics were done to this object, move it
        self.e.mOrigin1 = new_origin;

        if self.e.mFlags & FX_PLAYER_VIEW != 0
            && (self.e.mOrigin1[0] < 0.0
                || self.e.mOrigin1[0] > 639.0
                || self.e.mOrigin1[1] < 0.0
                || self.e.mOrigin1[1] > 479.0)
        {
            return false;
        }

        true
    }

    /// Raven `CParticle::UpdateSize`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:353-421`
    pub fn UpdateSize(&mut self, host: &mut FxHost<'_, '_>, fx: &FxSystem) {
        let mut perc1 = fade_curve(
            self.e.mFlags,
            fx.clock.mTime,
            self.e.mTimeStart,
            self.e.mTimeEnd,
            self.mSizeParm,
            FX_SIZE_LINEAR,
            FX_SIZE_PARM_MASK,
            FX_SIZE_NONLINEAR,
            FX_SIZE_WAVE,
            FX_SIZE_CLAMP,
        );

        // If needed, RAND can coexist with linear and either non-linear or wave.
        if self.e.mFlags & FX_SIZE_RAND != 0 {
            perc1 = host.rng().flrand(0.0, perc1);
        }

        self.e.mRefEnt.radius = (self.mSizeStart * perc1) + (self.mSizeEnd * (1.0 - perc1));
    }

    /// Raven `CParticle::UpdateRGB`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:456-533`
    pub fn UpdateRGB(&mut self, host: &mut FxHost<'_, '_>, fx: &FxSystem) {
        let mut perc1 = fade_curve(
            self.e.mFlags,
            fx.clock.mTime,
            self.e.mTimeStart,
            self.e.mTimeEnd,
            self.mRGBParm,
            FX_RGB_LINEAR,
            FX_RGB_PARM_MASK,
            FX_RGB_NONLINEAR,
            FX_RGB_WAVE,
            FX_RGB_CLAMP,
        );

        // If needed, RAND can coexist with linear and either non-linear or wave.
        if self.e.mFlags & FX_RGB_RAND != 0 {
            perc1 = host.rng().flrand(0.0, perc1);
        }

        // Now get the correct color
        let mut res = vector_scale(&self.mRGBStart, perc1);
        vector_ma_in_place(&mut res, 1.0 - perc1, &self.mRGBEnd);

        res[0] = com_clamp(0.0, 1.0, res[0]) * 255.0;
        res[1] = com_clamp(0.0, 1.0, res[1]) * 255.0;
        res[2] = com_clamp(0.0, 1.0, res[2]) * 255.0;

        self.e.mRefEnt.shaderRGBA = vector_to_int(&res);
    }

    /// Raven `CParticle::UpdateAlpha`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:538-625`
    pub fn UpdateAlpha(&mut self, host: &mut FxHost<'_, '_>, fx: &FxSystem) {
        let mut perc1 = fade_curve(
            self.e.mFlags,
            fx.clock.mTime,
            self.e.mTimeStart,
            self.e.mTimeEnd,
            self.mAlphaParm,
            FX_ALPHA_LINEAR,
            FX_ALPHA_PARM_MASK,
            FX_ALPHA_NONLINEAR,
            FX_ALPHA_WAVE,
            FX_ALPHA_CLAMP,
        );

        perc1 = (self.mAlphaStart * perc1) + (self.mAlphaEnd * (1.0 - perc1));

        // We should be in the right range, but clamp to ensure
        perc1 = com_clamp(0.0, 1.0, perc1);

        // If needed, RAND can coexist with linear and either non-linear or wave.
        if self.e.mFlags & FX_ALPHA_RAND != 0 {
            perc1 = host.rng().flrand(0.0, perc1);
        }

        let alpha = com_clamp(0.0, 255.0, perc1 * 255.0) as i32;
        if self.e.mFlags & FX_USE_ALPHA != 0 {
            // should use this when using art that has an alpha channel
            self.e.mRefEnt.shaderRGBA[3] = alpha as u8;
        } else {
            // Modulate the rgb fields by the alpha value to do the fade, works fine for
            // additive blending
            self.e.mRefEnt.shaderRGBA[0] =
                ((self.e.mRefEnt.shaderRGBA[0] as i32 * alpha) >> 8) as u8;
            self.e.mRefEnt.shaderRGBA[1] =
                ((self.e.mRefEnt.shaderRGBA[1] as i32 * alpha) >> 8) as u8;
            self.e.mRefEnt.shaderRGBA[2] =
                ((self.e.mRefEnt.shaderRGBA[2] as i32 * alpha) >> 8) as u8;
        }
    }

    /// Raven `CParticle::UpdateRotation`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:628-632`
    pub fn UpdateRotation(&mut self, fx: &FxSystem) {
        self.e.mRefEnt.rotation += fx.clock.mFrameTime as f32 * 0.01 * self.mRotationDelta;
        // decay rotationDelta
        self.mRotationDelta *= 1.0 - (fx.clock.mFrameTime as f32 * 0.0007);
    }
}

/// The size, RGB, alpha, length, and size2 curves all share this shape.
///
/// Raven writes the same block five times with different flag constants and a
/// different parm field. The `RAND` step stays at the call site, because only
/// three of the five apply it and every draw consumes one `flrand` when they do.
///
/// Source: `oracle/codemp/client/FxPrimitives.cpp:353-421`
#[allow(clippy::too_many_arguments)]
pub fn fade_curve(
    flags: i32,
    time: i32,
    time_start: i32,
    time_end: i32,
    parm: f32,
    linear: i32,
    parm_mask: i32,
    nonlinear: i32,
    wave: i32,
    clamp: i32,
) -> f32 {
    // completely biased towards start if it doesn't get overridden
    let mut perc1: f32 = 1.0;
    let mut perc2: f32 = 1.0;

    if flags & linear != 0 {
        // calculate element biasing
        perc1 = 1.0 - (time - time_start) as f32 / (time_end - time_start) as f32;
    }

    // We can combine FX_LINEAR with _either_ FX_NONLINEAR, FX_WAVE, or FX_CLAMP
    if (flags & parm_mask) == nonlinear {
        if (time as f32) > parm {
            // get percent done, using parm as the start of the non-linear fade
            perc2 = 1.0 - (time as f32 - parm) / (time_end as f32 - parm);
        }

        if flags & linear != 0 {
            // do an even blend
            perc1 = perc1 * 0.5 + perc2 * 0.5;
        } else {
            perc1 = perc2;
        }
    } else if (flags & parm_mask) == wave {
        // wave gen, with parm being the frequency multiplier
        perc1 *= ((time - time_start) as f32 * parm).cos();
    } else if (flags & parm_mask) == clamp {
        if (time as f32) < parm {
            perc2 = (parm - time as f32) / (parm - time_start as f32);
        } else {
            perc2 = 0.0; // make it full size??
        }

        if flags & linear != 0 {
            perc1 = perc1 * 0.5 + perc2 * 0.5;
        } else {
            perc1 = perc2;
        }
    }

    perc1
}

/// Raven's `VectorToInt`, an MSVC x87 assembly block.
///
/// `fistp` rounds to nearest with ties to even, which C truncation would not
/// match, and the packed word lands in `shaderRGBA` as red, green, blue, 255.
/// Source: `oracle/codemp/client/FxPrimitives.cpp:423-451`
pub fn vector_to_int(res: &vec3_t) -> [u8; 4] {
    [
        (res[0].round_ties_even() as i32) as u8,
        (res[1].round_ties_even() as i32) as u8,
        (res[2].round_ties_even() as i32) as u8,
        0xff,
    ]
}

/// Raven `Com_Clamp`.
///
/// Source: `oracle/codemp/game/q_math.c`
pub fn com_clamp(min: f32, max: f32, value: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// `VectorSubtract` as a value.
pub fn vector_subtract(a: &vec3_t, b: &vec3_t) -> vec3_t {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// `DotProduct`.
pub fn dot_product(a: &vec3_t, b: &vec3_t) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// `VectorLengthSquared`.
pub fn vector_length_squared(v: &vec3_t) -> f32 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

/// `VectorLength`.
pub fn vector_length(v: &vec3_t) -> f32 {
    vector_length_squared(v).sqrt()
}

/// `VectorScale` as a value.
pub fn vector_scale(v: &vec3_t, s: f32) -> vec3_t {
    [v[0] * s, v[1] * s, v[2] * s]
}

/// `VectorMA` as a value.
pub fn vector_ma(base: &vec3_t, scale: f32, dir: &vec3_t) -> vec3_t {
    [
        base[0] + scale * dir[0],
        base[1] + scale * dir[1],
        base[2] + scale * dir[2],
    ]
}

/// `VectorMA` writing back into the base.
pub fn vector_ma_in_place(base: &mut vec3_t, scale: f32, dir: &vec3_t) {
    base[0] += scale * dir[0];
    base[1] += scale * dir[1];
    base[2] += scale * dir[2];
}

/// `VectorAdd` as a value.
pub fn vector_add(a: &vec3_t, b: &vec3_t) -> vec3_t {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// `VectorNormalize`, returning the old length.
pub fn vector_normalize(v: &mut vec3_t) -> f32 {
    let length = vector_length(v);
    if length != 0.0 {
        let ilength = 1.0 / length;
        v[0] *= ilength;
        v[1] *= ilength;
        v[2] *= ilength;
    }
    length
}

/// `VectorCompare`.
pub fn vector_compare(a: &vec3_t, b: &vec3_t) -> bool {
    a[0] == b[0] && a[1] == b[1] && a[2] == b[2]
}

/// `DistanceSquared`.
pub fn distance_squared(a: &vec3_t, b: &vec3_t) -> f32 {
    vector_length_squared(&vector_subtract(b, a))
}
