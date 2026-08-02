//! Raven `CPoly`, a three-to-five vertex polygon that rotates while it moves.
//!
//! Raven's own warning applies: this is a lot of overhead for a single triangle
//! or quad. `mOrg` holds the verts as offsets from the midpoint, so physics can
//! move one origin and the shape follows.
//!
//! Class definition source: `oracle/codemp/client/FxPrimitives.h:574-608`
//! Method source: `oracle/codemp/client/FxPrimitives.cpp:1819-1998`

#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use native_math::qmath::{VectorRotate, PITCH, YAW};
use native_math::vector::{vec2_t, vec3_t};

use crate::fx::cparticle::{
    dot_product, vector_add, vector_compare, vector_length_squared, vector_scale, vector_subtract,
    ParticleCore,
};
use crate::fx::fx_host::FxHost;
use crate::fx::fx_system::FxSystem;

/// Source: `oracle/codemp/client/FxPrimitives.h:574`
pub const MAX_CPOLY_VERTS: usize = 5;

/// The `CPoly` fields, plus the `CParticle` core it inherited.
///
/// `mTimeStamp` is an absolute time, not a duration: motion stays frozen until
/// the clock passes it.
///
/// Class definition source: `oracle/codemp/client/FxPrimitives.h:576-608`
#[derive(Clone, Copy, Debug)]
pub struct CPoly {
    pub p: ParticleCore,

    pub mCount: i32,
    pub mRotDelta: vec3_t,
    pub mTimeStamp: i32,

    pub mOrg: [vec3_t; MAX_CPOLY_VERTS],
    pub mST: [vec2_t; MAX_CPOLY_VERTS],

    pub mRot: [[f32; 3]; 3],
    pub mLastFrameTime: i32,
}

impl Default for CPoly {
    /// Raven `CPoly::CPoly`, an empty body over the `CParticle` constructor.
    ///
    /// Raven leaves every `CPoly` field uninitialized. The spawn path fills the
    /// vert arrays and `PolyInit` fills the rest, so zero is the one defined
    /// starting value (porting-rules §19).
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.h:593`
    fn default() -> Self {
        CPoly {
            p: ParticleCore::default(),
            mCount: 0,
            mRotDelta: [0.0; 3],
            mTimeStamp: 0,
            mOrg: [[0.0; 3]; MAX_CPOLY_VERTS],
            mST: [[0.0; 2]; MAX_CPOLY_VERTS],
            mRot: [[0.0; 3]; 3],
            mLastFrameTime: 0,
        }
    }
}

impl CPoly {
    /// Source: `oracle/codemp/client/FxPrimitives.h:604`
    pub fn SetNumVerts(&mut self, c: i32) {
        self.mCount = c;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:605`
    pub fn SetRot(&mut self, r: Option<vec3_t>) {
        self.mRotDelta = r.unwrap_or([0.0; 3]);
    }

    /// Raven reads `theFxHelper.GetTime()` inside the setter. The caller passes
    /// it in as `time` instead.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.h:606`
    pub fn SetMotionTimeStamp(&mut self, time: i32, t: i32) {
        self.mTimeStamp = time + t;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:607`
    pub fn GetMotionTimeStamp(&self) -> i32 {
        self.mTimeStamp
    }

    /// Raven `CPoly::Cull`. The near-cull compares against the squared cvar.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:1819-1841`
    pub fn Cull(&self, fx: &FxSystem) -> bool {
        // Get the direction to the view
        let dir = vector_subtract(&self.p.e.mOrigin1, &fx.refdef.vieworg);

        // Check if it's behind the viewer
        if dot_product(&fx.refdef.viewaxis[0], &dir) < 0.0 {
            return true;
        }

        let len = vector_length_squared(&dir);

        // Can't be too close
        if len < fx.fx_nearCull * fx.fx_nearCull {
            return true;
        }

        false
    }

    /// Raven `CPoly::Draw`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:1844-1863`
    pub fn Draw(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) {
        let mut verts = [polyVert_t {
            xyz: [0.0; 3],
            st: [0.0; 2],
            modulate: [0; 4],
        }; MAX_CPOLY_VERTS];

        let count = self.mCount as usize;
        for i in 0..count {
            // Add our midpoint and vert offset to get the actual vertex
            verts[i].xyz = vector_add(&self.p.e.mOrigin1, &self.mOrg[i]);

            // Assign the same color to each vert
            verts[i].modulate = self.p.e.mRefEnt.shaderRGBA;

            // Copy the ST coords
            verts[i].st = self.mST[i];
        }

        // Add this poly
        host.AddPolyToScene(self.p.e.mRefEnt.customShader, &verts[..count]);
        fx.drawnFx += 1;
    }

    /// Raven `CPoly::CalcRotateMatrix`. Roll is not supported.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:1866-1904`
    pub fn CalcRotateMatrix(&mut self, fx: &FxSystem) {
        // rotate around Z
        // Raven's `DEG2RAD` multiplies by the double `M_PI`, so the product
        // rounds to float only on the store.
        let rad = ((self.mRotDelta[YAW] * fx.clock.mFrameTime as f32 * 0.01) as f64
            * core::f64::consts::PI
            / 180.0) as f32;
        let cos_z = rad.cos();
        let sin_z = rad.sin();
        // rotate around X
        let rad = ((self.mRotDelta[PITCH] * fx.clock.mFrameTime as f32 * 0.01) as f64
            * core::f64::consts::PI
            / 180.0) as f32;
        let cos_x = rad.cos();
        let sin_x = rad.sin();

        self.mRot[0][0] = cos_z;
        self.mRot[1][0] = -sin_z;
        self.mRot[2][0] = 0.0;
        self.mRot[0][1] = cos_x * sin_z;
        self.mRot[1][1] = cos_x * cos_z;
        self.mRot[2][1] = -sin_x;
        self.mRot[0][2] = sin_x * sin_z;
        self.mRot[1][2] = sin_x * cos_z;
        self.mRot[2][2] = cos_x;

        self.mLastFrameTime = fx.clock.mFrameTime;
    }

    /// Raven `CPoly::Rotate`, applied to every vert offset.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:1907-1923`
    pub fn Rotate(&mut self, fx: &FxSystem) {
        let mut temp: [vec3_t; MAX_CPOLY_VERTS] = [[0.0; 3]; MAX_CPOLY_VERTS];
        let dif = ((self.mLastFrameTime - fx.clock.mFrameTime) as f32).abs();

        if dif > 0.1 * self.mLastFrameTime as f32 {
            self.CalcRotateMatrix(fx);
        }

        // Multiply our rotation matrix by each of the offset verts to get their new position
        let mut rot = self.mRot;
        for i in 0..self.mCount as usize {
            // SAFETY: `VectorRotate` takes Raven's `vec3_t*` and reads the three
            // rows this local matrix owns.
            VectorRotate(self.mOrg[i], rot.as_mut_ptr(), &mut temp[i]);
            self.mOrg[i] = temp[i];
        }
    }

    /// Raven `CPoly::Update`. Rotation runs only while the poly moves.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:1928-1966`
    pub fn Update(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) -> bool {
        // Game pausing can cause dumb time things to happen, so kill the effect in this instance
        if self.p.e.mTimeStart > fx.clock.mTime {
            return false;
        }

        // If our timestamp hasn't exired yet, we won't even consider doing any kind of motion
        if fx.clock.mTime > self.mTimeStamp {
            let old_origin = self.p.e.mOrigin1;

            if self.p.e.mTimeStart < fx.clock.mTime && !self.p.UpdateOrigin(host, fx) {
                // we are marked for death
                return false;
            }

            // Only rotate whilst moving
            if !vector_compare(&old_origin, &self.p.e.mOrigin1) {
                self.Rotate(fx);
            }
        }

        if !self.Cull(fx) {
            // Only update these if the thing is visible.
            self.p.UpdateRGB(host, fx);
            self.p.UpdateAlpha(host, fx);

            self.Draw(host, fx);
        }

        true
    }

    /// Raven `CPoly::PolyInit`, which turns the passed points into midpoint offsets.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:1969-1998`
    pub fn PolyInit(&mut self, fx: &FxSystem) {
        if self.mCount < 3 {
            return;
        }

        let count = self.mCount as usize;
        let mut org: vec3_t = [0.0, 0.0, 0.0];

        // Find our midpoint
        for i in 0..count {
            org = vector_add(&org, &self.mOrg[i]);
        }

        org = vector_scale(&org, 1.0 / self.mCount as f32);

        // now store our midpoint for physics purposes
        self.p.e.mOrigin1 = org;

        // Now we process the passed in points and make it so that they aren't actually the point...
        //	rather, they are the offset from mOrigin1.
        for i in 0..count {
            self.mOrg[i] = vector_subtract(&self.mOrg[i], &self.p.e.mOrigin1);
        }

        self.CalcRotateMatrix(fx);
    }
}
