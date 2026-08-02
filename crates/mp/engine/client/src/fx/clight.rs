//! Raven `CLight`, the one primitive that adds a dynamic light instead of geometry.
//!
//! Raven's `~CLight` calls `mGhoul2.kill()`, which drops the handle without
//! freeing the arena slot. Dropping a `CLight` does exactly that already, so no
//! teardown ports.
//!
//! Class definition source: `oracle/codemp/client/FxPrimitives.h:215-264`
//! Method source: `oracle/codemp/client/FxPrimitives.cpp:1494-1689`

#![allow(non_camel_case_types, non_snake_case)]

use native_math::vector::vec3_t;

use crate::fx::ceffect::EffectCore;
use crate::fx::cparticle::{fade_curve, vector_ma, vector_ma_in_place, vector_scale};
use crate::fx::fx_flags::{
    FX_RELATIVE, FX_RGB_CLAMP, FX_RGB_LINEAR, FX_RGB_NONLINEAR, FX_RGB_PARM_MASK, FX_RGB_RAND,
    FX_RGB_WAVE, FX_SIZE_CLAMP, FX_SIZE_LINEAR, FX_SIZE_NONLINEAR, FX_SIZE_PARM_MASK, FX_SIZE_RAND,
    FX_SIZE_WAVE,
};
use crate::fx::fx_host::FxHost;
use crate::fx::fx_system::FxSystem;

/// The `CLight` fields, plus the `CEffect` core it inherited.
///
/// `CLight` derives straight from `CEffect`, not from `CParticle`, so it carries
/// its own size and RGB curves and its own bolt block.
///
/// Class definition source: `oracle/codemp/client/FxPrimitives.h:215-264`
#[derive(Clone, Copy, Debug)]
pub struct CLight {
    pub e: EffectCore,

    pub mSizeStart: f32,
    pub mSizeEnd: f32,
    pub mSizeParm: f32,

    pub mOrgOffset: vec3_t,
    pub mRGBStart: vec3_t,
    pub mRGBEnd: vec3_t,
    pub mRGBParm: f32,

    /// Raven holds a `CGhoul2Info_v`, which is one `int` handle. The port keeps
    /// the handle itself, so the primitive stays `Copy` and needs no destructor.
    /// Raven's `mGhoul2.kill()` clears the handle without freeing the arena slot,
    /// which is what dropping this field already does.
    pub mGhoul2: i32,
    pub mEntNum: i16,
    pub mModelNum: i8,
    pub mBoltNum: i8,
}

impl Default for CLight {
    /// Raven `CLight::CLight`, a light with no bolt.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.h:239-242`
    fn default() -> Self {
        CLight {
            e: EffectCore::default(),
            mSizeStart: 0.0,
            mSizeEnd: 0.0,
            mSizeParm: 0.0,
            mOrgOffset: [0.0; 3],
            mRGBStart: [0.0; 3],
            mRGBEnd: [0.0; 3],
            mRGBParm: 0.0,
            mGhoul2: 0,
            mEntNum: -1,
            mModelNum: -1,
            mBoltNum: -1,
        }
    }
}

impl CLight {
    /// Source: `oracle/codemp/client/FxPrimitives.h:249-252`
    pub fn SetBoltinfo(&mut self, i_ghoul2: i32, ent_num: i32, model_num: i32, bolt_num: i32) {
        self.mGhoul2 = i_ghoul2;
        self.mEntNum = ent_num as i16;
        self.mModelNum = model_num as i8;
        self.mBoltNum = bolt_num as i8;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:256`
    pub fn SetSizeStart(&mut self, sz: f32) {
        self.mSizeStart = sz;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:257`
    pub fn SetSizeEnd(&mut self, sz: f32) {
        self.mSizeEnd = sz;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:258`
    pub fn SetSizeParm(&mut self, parm: f32) {
        self.mSizeParm = parm;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:260`
    pub fn SetOrgOffset(&mut self, o: Option<vec3_t>) {
        self.mOrgOffset = o.unwrap_or([0.0; 3]);
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:261`
    pub fn SetRGBStart(&mut self, rgb: Option<vec3_t>) {
        self.mRGBStart = rgb.unwrap_or([0.0; 3]);
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:262`
    pub fn SetRGBEnd(&mut self, rgb: Option<vec3_t>) {
        self.mRGBEnd = rgb.unwrap_or([0.0; 3]);
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:263`
    pub fn SetRGBParm(&mut self, parm: f32) {
        self.mRGBParm = parm;
    }

    /// Raven `CLight::Draw`.
    ///
    /// The color comes out of `mRefEnt.origin`, which `UpdateRGB` writes. The
    /// `VV_LIGHTING` branch is not in the retail MP build, so the `#else` arm ports.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:1494-1502`
    pub fn Draw(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) {
        host.AddLightToScene(
            self.e.mOrigin1,
            self.e.mRefEnt.radius,
            self.e.mRefEnt.origin[0],
            self.e.mRefEnt.origin[1],
            self.e.mRefEnt.origin[2],
        );
        fx.drawnFx += 1;
    }

    /// Raven `CLight::Update`. A light never culls and never runs physics.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:1507-1540`
    pub fn Update(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) -> bool {
        // Game pausing can cause dumb time things to happen, so kill the effect in this instance
        if self.e.mTimeStart > fx.clock.mTime {
            return false;
        }

        if self.e.mFlags & FX_RELATIVE != 0 {
            if !host.Ghoul2IsValid(self.mGhoul2) {
                // the thing we are bolted to is no longer valid, so we may as well just die.
                return false;
            }

            let ent = self.mEntNum as i32;
            let model = self.mModelNum as i32;
            let bolt = self.mBoltNum as i32;
            let old_time = fx.clock.mOldTime;
            // Get our current position and direction
            let Some((org, ax)) =
                host.GetOriginAxisFromBolt(self.mGhoul2, ent, model, bolt, old_time)
            else {
                // could not get bolt
                return false;
            };
            self.e.mOrigin1 = org;

            vector_ma_in_place(&mut self.e.mOrigin1, self.mOrgOffset[0], &ax[0]);
            vector_ma_in_place(&mut self.e.mOrigin1, self.mOrgOffset[1], &ax[1]);
            vector_ma_in_place(&mut self.e.mOrigin1, self.mOrgOffset[2], &ax[2]);
        }

        self.UpdateSize(host, fx);
        self.UpdateRGB(host, fx);

        self.Draw(host, fx);

        true
    }

    /// Raven `CLight::UpdateSize`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:1545-1613`
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

    /// Raven `CLight::UpdateRGB`.
    ///
    /// Raven writes the blended color into `mRefEnt.origin`, not into
    /// `shaderRGBA`, and `Draw` reads it back from there as the light color.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:1618-1689`
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
        let res = vector_scale(&self.mRGBStart, perc1);
        self.e.mRefEnt.origin = vector_ma(&res, 1.0 - perc1, &self.mRGBEnd);
    }
}
