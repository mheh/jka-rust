//! Raven `CCylinder`, a tapered tube along one axis.
//!
//! `mRefEnt.radius` is the start radius and `mRefEnt.rotation` carries the end
//! radius, which is why the class runs a second size curve. A `mTraceEnd`
//! cylinder traces the world every frame to find its own length.
//!
//! Class definition source: `oracle/codemp/client/FxPrimitives.h:499-526`
//! Method source: `oracle/codemp/client/FxPrimitives.cpp:1139-1309`

#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::cgame::tr_types::RF_DEPTHHACK;
use mp_qshared::shared::surface_flags::MASK_SOLID;
use native_math::vector::vec3_t;

use crate::fx::cparticle::{fade_curve, vector_add, vector_length, vector_ma, vector_subtract};
use crate::fx::ctail::CTail;
use crate::fx::fx_flags::{
    FX_DEPTH_HACK, FX_MAX_TRACE_DIST, FX_RELATIVE, FX_SIZE2_CLAMP, FX_SIZE2_LINEAR,
    FX_SIZE2_NONLINEAR, FX_SIZE2_PARM_MASK, FX_SIZE2_RAND, FX_SIZE2_WAVE,
};
use crate::fx::fx_host::FxHost;
use crate::fx::fx_system::FxSystem;

/// The `CCylinder` fields, plus the `CTail` core it inherited.
///
/// Class definition source: `oracle/codemp/client/FxPrimitives.h:499-526`
#[derive(Clone, Copy, Debug)]
pub struct CCylinder {
    pub t: CTail,

    pub mSize2Start: f32,
    pub mSize2End: f32,
    pub mSize2Parm: f32,
    pub mTraceEnd: bool,
}

impl Default for CCylinder {
    /// Raven `CCylinder::CCylinder`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:1139-1143`
    fn default() -> Self {
        let mut t = CTail::default();
        t.p.e.mRefEnt.reType = refEntityType_t::RT_CYLINDER;
        CCylinder {
            t,
            mSize2Start: 0.0,
            mSize2End: 0.0,
            mSize2Parm: 0.0,
            mTraceEnd: false,
        }
    }
}

impl CCylinder {
    /// Source: `oracle/codemp/client/FxPrimitives.h:520`
    pub fn SetSize2Start(&mut self, sz: f32) {
        self.mSize2Start = sz;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:521`
    pub fn SetSize2End(&mut self, sz: f32) {
        self.mSize2End = sz;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:522`
    pub fn SetSize2Parm(&mut self, parm: f32) {
        self.mSize2Parm = parm;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:523`
    pub fn SetTraceEnd(&mut self, trace_end: bool) {
        self.mTraceEnd = trace_end;
    }

    /// The cylinder's own axis, which the length and the draw both point along.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.h:525`
    pub fn SetNormal(&mut self, norm: vec3_t) {
        self.t.p.e.mRefEnt.axis[0] = norm;
    }

    /// Raven `CCylinder::Cull`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:1145-1153`
    pub fn Cull(&self, fx: &FxSystem) -> bool {
        if self.mTraceEnd {
            //eh, don't cull variable-length cylinders
            return false;
        }

        self.t.p.Cull(fx)
    }

    /// Raven `CCylinder::UpdateLength`, the trace-terminated override.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:1155-1171`
    pub fn UpdateLength(&mut self, host: &mut FxHost<'_, '_>, fx: &FxSystem) {
        if self.mTraceEnd {
            let temp = vector_ma(
                &self.t.p.e.mOrigin1,
                FX_MAX_TRACE_DIST,
                &self.t.p.e.mRefEnt.axis[0],
            );
            let tr = host.Trace(self.t.p.e.mOrigin1, None, None, temp, -1, MASK_SOLID, false);
            let temp = vector_subtract(&tr.endpos, &self.t.p.e.mOrigin1);
            self.t.mLength = vector_length(&temp);
        } else {
            self.t.UpdateLength(host, fx);
        }
    }

    /// Raven `CCylinder::Draw`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:1174-1187`
    pub fn Draw(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) {
        if self.t.p.e.mFlags & FX_DEPTH_HACK != 0 {
            // Not sure if first person needs to be set, but it can't hurt?
            self.t.p.e.mRefEnt.renderfx |= RF_DEPTHHACK;
        }

        self.t.p.e.mRefEnt.origin = self.t.p.e.mOrigin1;
        self.t.p.e.mRefEnt.oldorigin = vector_ma(
            &self.t.p.e.mOrigin1,
            self.t.mLength,
            &self.t.p.e.mRefEnt.axis[0],
        );

        host.AddFxToScene(Some(&self.t.p.e.mRefEnt));
        fx.drawnFx += 1;
    }

    /// Raven `CCylinder::UpdateSize2`, the end-radius curve.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:1192-1260`
    pub fn UpdateSize2(&mut self, host: &mut FxHost<'_, '_>, fx: &FxSystem) {
        let mut perc1 = fade_curve(
            self.t.p.e.mFlags,
            fx.clock.mTime,
            self.t.p.e.mTimeStart,
            self.t.p.e.mTimeEnd,
            self.mSize2Parm,
            FX_SIZE2_LINEAR,
            FX_SIZE2_PARM_MASK,
            FX_SIZE2_NONLINEAR,
            FX_SIZE2_WAVE,
            FX_SIZE2_CLAMP,
        );

        // If needed, RAND can coexist with linear and either non-linear or wave.
        if self.t.p.e.mFlags & FX_SIZE2_RAND != 0 {
            perc1 = host.rng().flrand(0.0, perc1);
        }

        self.t.p.e.mRefEnt.rotation = (self.mSize2Start * perc1) + (self.mSize2End * (1.0 - perc1));
    }

    /// Raven `CCylinder::Update`. A bolted cylinder takes its axis from the bolt.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:1263-1309`
    pub fn Update(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) -> bool {
        // Game pausing can cause dumb time things to happen, so kill the effect in this instance
        if self.t.p.e.mTimeStart > fx.clock.mTime {
            return false;
        }

        if self.t.p.e.mFlags & FX_RELATIVE != 0 {
            if !self.t.p.bolt_still_valid(host) {
                // the thing we are bolted to is no longer valid, so we may as well just die.
                return false;
            }

            // Get our current position and direction
            let Some((org, ax)) = self.t.p.fetch_bolt(host, fx) else {
                // could not get bolt
                return false;
            };
            self.t.p.e.mOrigin1 = org;

            //add the offset to the bolt point
            self.t.p.e.mOrigin1 = vector_add(&self.t.p.e.mOrigin1, &self.t.p.mOrgOffset);

            self.t.p.e.mRefEnt.axis[0] = ax[0];
        }

        if !self.Cull(fx) {
            // Only update these if the thing is visible.
            self.t.p.UpdateSize(host, fx);
            self.UpdateSize2(host, fx);
            self.UpdateLength(host, fx);
            self.t.p.UpdateRGB(host, fx);
            self.t.p.UpdateAlpha(host, fx);

            self.Draw(host, fx);
        }

        true
    }
}
