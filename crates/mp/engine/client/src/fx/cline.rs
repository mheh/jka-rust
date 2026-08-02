//! Raven `CLine`, the two-point beam, and the base `CBezier` and `CElectricity` share.
//!
//! Raven overrides `Die` back to an empty body here, so a line never runs a
//! death effect. `FxPrimitive::Die` carries that in its no-op arm.
//!
//! Class definition source: `oracle/codemp/client/FxPrimitives.h:377-394`
//! Method source: `oracle/codemp/client/FxPrimitives.cpp:784-846`

#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::cgame::tr_types::RF_DEPTHHACK;
use native_math::vector::vec3_t;

use crate::fx::cparticle::{vector_add, vector_ma, vector_ma_in_place, ParticleCore};
use crate::fx::fx_flags::{FX_DEPTH_HACK, FX_RELATIVE};
use crate::fx::fx_host::FxHost;
use crate::fx::fx_system::FxSystem;

/// The `CLine` fields, plus the `CParticle` core it inherited.
///
/// Class definition source: `oracle/codemp/client/FxPrimitives.h:377-394`
#[derive(Clone, Copy, Debug)]
pub struct CLine {
    pub p: ParticleCore,

    pub mOrigin2: vec3_t,
}

impl Default for CLine {
    /// Raven `CLine::CLine`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:784-787`
    fn default() -> Self {
        let mut p = ParticleCore::default();
        p.e.mRefEnt.reType = refEntityType_t::RT_LINE;
        CLine {
            p,
            mOrigin2: [0.0; 3],
        }
    }
}

impl CLine {
    /// Source: `oracle/codemp/client/FxPrimitives.h:393`
    pub fn SetOrigin2(&mut self, org2: vec3_t) {
        self.mOrigin2 = org2;
    }

    /// Raven `CLine::Draw`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:790-803`
    pub fn Draw(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) {
        if self.p.e.mFlags & FX_DEPTH_HACK != 0 {
            // Not sure if first person needs to be set, but it can't hurt?
            self.p.e.mRefEnt.renderfx |= RF_DEPTHHACK;
        }

        self.p.e.mRefEnt.origin = self.p.e.mOrigin1;
        self.p.e.mRefEnt.oldorigin = self.mOrigin2;

        host.AddFxToScene(Some(&self.p.e.mRefEnt));
        fx.drawnFx += 1;
    }

    /// Raven `CLine::Update`. A bolted line hangs both endpoints off the bolt axis.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:806-846`
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
            let Some((org, ax)) = self.p.fetch_bolt(host, fx) else {
                // could not get bolt
                return false;
            };
            self.p.e.mOrigin1 = org;

            //add the offset to the bolt point
            self.p.e.mOrigin1 = vector_add(&self.p.e.mOrigin1, &self.p.mOrgOffset);

            self.mOrigin2 = vector_ma(&self.p.e.mOrigin1, self.p.mVel[0], &ax[0]);
            vector_ma_in_place(&mut self.mOrigin2, self.p.mVel[1], &ax[1]);
            vector_ma_in_place(&mut self.mOrigin2, self.p.mVel[2], &ax[2]);
        }

        if !self.p.Cull(fx) {
            // Only update these if the thing is visible.
            self.p.UpdateSize(host, fx);
            self.p.UpdateRGB(host, fx);
            self.p.UpdateAlpha(host, fx);

            self.Draw(host, fx);
        }

        true
    }
}
