//! Raven `CElectricity`, a lightning beam.
//!
//! The renderer does the whole bolt. This class only hands it the two endpoints
//! plus three packed parameters: chaos, duration, and end time, all smuggled
//! through `mRefEnt.axis[0]`.
//!
//! Class definition source: `oracle/codemp/client/FxPrimitives.h:426-445`
//! Method source: `oracle/codemp/client/FxPrimitives.cpp:853-938`

#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::cgame::tr_types::{RF_DEPTHHACK, RF_FORKED, RF_GROW, RF_TAPERED};

use crate::fx::cline::CLine;
use crate::fx::cparticle::{vector_add, vector_ma, vector_ma_in_place};
use crate::fx::fx_flags::{FX_BRANCH, FX_DEPTH_HACK, FX_GROW, FX_RELATIVE, FX_TAPER};
use crate::fx::fx_host::FxHost;
use crate::fx::fx_system::FxSystem;

/// The `CElectricity` fields, plus the `CLine` core it inherited.
///
/// Class definition source: `oracle/codemp/client/FxPrimitives.h:426-445`
#[derive(Clone, Copy, Debug)]
pub struct CElectricity {
    pub l: CLine,

    pub mChaos: f32,
}

impl Default for CElectricity {
    /// Raven `CElectricity::CElectricity`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:853-856`
    fn default() -> Self {
        let mut l = CLine::default();
        l.p.e.mRefEnt.reType = refEntityType_t::RT_ELECTRICITY;
        CElectricity { l, mChaos: 0.0 }
    }
}

impl CElectricity {
    /// Source: `oracle/codemp/client/FxPrimitives.h:444`
    pub fn SetChaos(&mut self, chaos: f32) {
        self.mChaos = chaos;
    }

    /// Raven `CElectricity::Initialize`, the one-time seed and renderfx setup.
    ///
    /// Raven draws `flrand(0.0, 1.0)` inside the body. The caller draws it and
    /// passes it in as `frame_draw`, which keeps the generator order Raven has.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:859-883`
    pub fn Initialize(&mut self, time: i32, frame_draw: f32) {
        self.l.p.e.mRefEnt.frame = (frame_draw * 1265536.0) as i32;
        // endtime
        self.l.p.e.mRefEnt.axis[0][2] =
            (time + (self.l.p.e.mTimeEnd - self.l.p.e.mTimeStart)) as f32;

        if self.l.p.e.mFlags & FX_DEPTH_HACK != 0 {
            self.l.p.e.mRefEnt.renderfx |= RF_DEPTHHACK;
        }

        if self.l.p.e.mFlags & FX_BRANCH != 0 {
            self.l.p.e.mRefEnt.renderfx |= RF_FORKED;
        }

        if self.l.p.e.mFlags & FX_TAPER != 0 {
            self.l.p.e.mRefEnt.renderfx |= RF_TAPERED;
        }

        if self.l.p.e.mFlags & FX_GROW != 0 {
            self.l.p.e.mRefEnt.renderfx |= RF_GROW;
        }
    }

    /// Raven `CElectricity::Draw`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:886-895`
    pub fn Draw(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) {
        self.l.p.e.mRefEnt.origin = self.l.p.e.mOrigin1;
        self.l.p.e.mRefEnt.oldorigin = self.l.mOrigin2;
        self.l.p.e.mRefEnt.axis[0][0] = self.mChaos;
        self.l.p.e.mRefEnt.axis[0][1] = (self.l.p.e.mTimeEnd - self.l.p.e.mTimeStart) as f32;

        host.AddFxToScene(Some(&self.l.p.e.mRefEnt));
        fx.drawnFx += 1;
    }

    /// Raven `CElectricity::Update`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:898-938`
    pub fn Update(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) -> bool {
        // Game pausing can cause dumb time things to happen, so kill the effect in this instance
        if self.l.p.e.mTimeStart > fx.clock.mTime {
            return false;
        }

        if self.l.p.e.mFlags & FX_RELATIVE != 0 {
            if !self.l.p.bolt_still_valid(host) {
                // the thing we are bolted to is no longer valid, so we may as well just die.
                return false;
            }

            // Get our current position and direction
            let Some((org, ax)) = self.l.p.fetch_bolt(host, fx) else {
                // could not get bolt
                return false;
            };
            self.l.p.e.mOrigin1 = org;

            //add the offset to the bolt point
            self.l.p.e.mOrigin1 = vector_add(&self.l.p.e.mOrigin1, &self.l.p.mOrgOffset);

            self.l.mOrigin2 = vector_ma(&self.l.p.e.mOrigin1, self.l.p.mVel[0], &ax[0]);
            vector_ma_in_place(&mut self.l.mOrigin2, self.l.p.mVel[1], &ax[1]);
            vector_ma_in_place(&mut self.l.mOrigin2, self.l.p.mVel[2], &ax[2]);
        }

        if !self.l.p.Cull(fx) {
            // Only update these if the thing is visible.
            self.l.p.UpdateSize(host, fx);
            self.l.p.UpdateRGB(host, fx);
            self.l.p.UpdateAlpha(host, fx);

            self.Draw(host, fx);
        }

        true
    }
}
