//! Raven `CBezier`, a four-point curve drawn as a strip of view-facing quads.
//!
//! `mOrigin1` and `mOrigin2` are the two endpoints, and the two control points
//! carry their own velocity, so the curve bends over its life.
//!
//! Class definition source: `oracle/codemp/client/FxPrimitives.h:397-423`
//! Method source: `oracle/codemp/client/FxPrimitives.cpp:2007-2183`

#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use native_math::qmath::CrossProduct;
use native_math::vector::vec3_t;

use crate::fx::cline::CLine;
use crate::fx::cparticle::{dot_product, vector_ma, vector_normalize, vector_subtract};
use crate::fx::fx_host::FxHost;
use crate::fx::fx_system::FxSystem;

/// How many segments one curve draws.
///
/// Source: `oracle/codemp/client/FxPrimitives.cpp:2142`
const BEZIER_RESOLUTION: f32 = 16.0;

/// The `CBezier` fields, plus the `CLine` core it inherited.
///
/// `mInit` says whether the previous segment left a shared edge in `lastEnd`.
/// `Draw` clears it before every batch, so it never carries across frames.
///
/// Class definition source: `oracle/codemp/client/FxPrimitives.h:397-423`
#[derive(Clone, Copy, Debug)]
pub struct CBezier {
    pub l: CLine,

    pub mControl1: vec3_t,
    pub mControl1Vel: vec3_t,

    pub mControl2: vec3_t,
    pub mControl2Vel: vec3_t,

    pub mInit: bool,
}

impl Default for CBezier {
    /// Raven `CBezier::CBezier`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.h:411`
    fn default() -> Self {
        CBezier {
            l: CLine::default(),
            mControl1: [0.0; 3],
            mControl1Vel: [0.0; 3],
            mControl2: [0.0; 3],
            mControl2Vel: [0.0; 3],
            mInit: false,
        }
    }
}

impl CBezier {
    /// Source: `oracle/codemp/client/FxPrimitives.h:421`
    pub fn SetControlPoints(&mut self, ctrl1: vec3_t, ctrl2: vec3_t) {
        self.mControl1 = ctrl1;
        self.mControl2 = ctrl2;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:422`
    pub fn SetControlVel(&mut self, ctrl1v: vec3_t, ctrl2v: vec3_t) {
        self.mControl1Vel = ctrl1v;
        self.mControl2Vel = ctrl2v;
    }

    /// Raven `CBezier::Cull`. The curve survives while any of three points is in front.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:2007-2036`
    pub fn Cull(&self, fx: &FxSystem) -> bool {
        let dir = vector_subtract(&self.l.p.e.mOrigin1, &fx.refdef.vieworg);

        //Check if it's in front of the viewer
        if dot_product(&fx.refdef.viewaxis[0], &dir) >= 0.0 {
            return false; //don't cull
        }

        let dir = vector_subtract(&self.l.mOrigin2, &fx.refdef.vieworg);

        //Check if it's in front of the viewer
        if dot_product(&fx.refdef.viewaxis[0], &dir) >= 0.0 {
            return false;
        }

        let dir = vector_subtract(&self.mControl1, &fx.refdef.vieworg);

        //Check if it's in front of the viewer
        if dot_product(&fx.refdef.viewaxis[0], &dir) >= 0.0 {
            return false;
        }

        true //all points behind viewer
    }

    /// Raven `CBezier::Update`. The curve runs no time check and no physics.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:2039-2063`
    pub fn Update(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) -> bool {
        let ftime = fx.clock.mFrameTime as f32 * 0.001;
        let time2 = ftime * ftime * 0.5;

        self.mControl1[0] =
            self.mControl1[0] + (ftime * self.mControl1Vel[0]) + (time2 * self.mControl1Vel[0]);
        self.mControl2[0] =
            self.mControl2[0] + (ftime * self.mControl2Vel[0]) + (time2 * self.mControl2Vel[0]);
        self.mControl1[1] =
            self.mControl1[1] + (ftime * self.mControl1Vel[1]) + (time2 * self.mControl1Vel[1]);
        self.mControl2[1] =
            self.mControl2[1] + (ftime * self.mControl2Vel[1]) + (time2 * self.mControl2Vel[1]);
        self.mControl1[2] =
            self.mControl1[2] + (ftime * self.mControl1Vel[2]) + (time2 * self.mControl1Vel[2]);
        self.mControl2[2] =
            self.mControl2[2] + (ftime * self.mControl2Vel[2]) + (time2 * self.mControl2Vel[2]);

        if !self.Cull(fx) {
            // Only update these if the thing is visible.
            self.l.p.UpdateSize(host, fx);
            self.l.p.UpdateRGB(host, fx);
            self.l.p.UpdateAlpha(host, fx);

            self.Draw(host, fx);
        }
        true
    }

    /// Raven `CBezier::DrawSegment`, one quad of the strip.
    ///
    /// Raven holds the shared edge in a `static vec3_t lastEnd[2]`. `Draw` clears
    /// `mInit` before the first segment and the static is read only when `mInit`
    /// is true, so it never lives across calls and becomes a `Draw` local here.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:2066-2140`
    #[allow(clippy::too_many_arguments)]
    pub fn DrawSegment(
        &mut self,
        host: &mut FxHost<'_, '_>,
        fx: &FxSystem,
        last_end: &mut [vec3_t; 2],
        start: vec3_t,
        end: vec3_t,
        texcoord1: f32,
        texcoord2: f32,
        seg_percent: f32,
        last_seg_percent: f32,
    ) {
        let mut verts = [polyVert_t {
            xyz: [0.0; 3],
            st: [0.0; 2],
            modulate: [0; 4],
        }; 4];

        let line_dir = vector_subtract(&end, &start);
        let view_dir = vector_subtract(&end, &fx.refdef.vieworg);
        let mut cross: vec3_t = [0.0; 3];
        CrossProduct(line_dir, view_dir, &mut cross);
        vector_normalize(&mut cross);

        // scaleBottom is the width of the bottom edge of the quad, scaleTop is the width of the top edge
        let scale_bottom = (self.l.p.mSizeStart
            + ((self.l.p.mSizeEnd - self.l.p.mSizeStart) * last_seg_percent))
            * 0.5;
        let scale_top =
            (self.l.p.mSizeStart + ((self.l.p.mSizeEnd - self.l.p.mSizeStart) * seg_percent)) * 0.5;

        //Construct the oriented quad
        if self.mInit {
            verts[0].xyz = last_end[0];
            verts[1].xyz = last_end[1];
        } else {
            verts[0].xyz = vector_ma(&start, -scale_bottom, &cross);
            verts[1].xyz = vector_ma(&start, scale_bottom, &cross);
        }

        let rgba = self.l.p.e.mRefEnt.shaderRGBA;

        verts[0].st[0] = 0.0;
        verts[0].st[1] = texcoord1;

        verts[0].modulate[0] = (rgba[0] as f32 * (1.0 - texcoord1)) as u8;
        verts[0].modulate[1] = (rgba[1] as f32 * (1.0 - texcoord1)) as u8;
        verts[0].modulate[2] = (rgba[2] as f32 * (1.0 - texcoord1)) as u8;
        verts[0].modulate[3] = rgba[3];

        verts[1].st[0] = 1.0;
        verts[1].st[1] = texcoord1;

        verts[1].modulate[0] = (rgba[0] as f32 * (1.0 - texcoord1)) as u8;
        verts[1].modulate[1] = (rgba[1] as f32 * (1.0 - texcoord1)) as u8;
        verts[1].modulate[2] = (rgba[2] as f32 * (1.0 - texcoord1)) as u8;
        verts[1].modulate[3] = rgba[3];

        if texcoord1 == 0.0 {
            // Raven zeroes the whole four-byte word through an int cast.
            verts[0].modulate = [0; 4];
            verts[1].modulate = [0; 4];
        }

        verts[2].xyz = vector_ma(&end, scale_top, &cross);
        verts[2].st[0] = 1.0;
        verts[2].st[1] = texcoord2;

        verts[2].modulate[0] = (rgba[0] as f32 * (1.0 - texcoord2)) as u8;
        verts[2].modulate[1] = (rgba[1] as f32 * (1.0 - texcoord2)) as u8;
        verts[2].modulate[2] = (rgba[2] as f32 * (1.0 - texcoord2)) as u8;
        verts[2].modulate[3] = rgba[3];

        verts[3].xyz = vector_ma(&end, -scale_top, &cross);
        verts[3].st[0] = 0.0;
        verts[3].st[1] = texcoord2;

        verts[3].modulate[0] = (rgba[0] as f32 * (1.0 - texcoord2)) as u8;
        verts[3].modulate[1] = (rgba[1] as f32 * (1.0 - texcoord2)) as u8;
        verts[3].modulate[2] = (rgba[2] as f32 * (1.0 - texcoord2)) as u8;
        verts[3].modulate[3] = rgba[3];

        host.AddPolyToScene(self.l.p.e.mRefEnt.customShader, &verts);

        last_end[1] = verts[2].xyz;
        last_end[0] = verts[3].xyz;

        self.mInit = true;
    }

    /// Raven `CBezier::Draw`, sixteen segments along the curve.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:2145-2183`
    pub fn Draw(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) {
        let incr = 1.0 / BEZIER_RESOLUTION;
        let tex = 1.0f32;

        let mut old_pos = self.l.p.e.mOrigin1;

        self.mInit = false; //Signify a new batch for vert gluing

        // The shared-edge carrier Raven keeps in a file-scope static.
        let mut last_end: [vec3_t; 2] = [[0.0; 3]; 2];

        let mut tc1 = 0.0f32;

        let mut mu = incr;
        while mu <= 1.0 {
            //Four point curve
            let mum1 = 1.0 - mu;
            let mum13 = mum1 * mum1 * mum1;
            let mu3 = mu * mu * mu;
            let group1 = 3.0 * mu * mum1 * mum1;
            let group2 = 3.0 * mu * mu * mum1;

            let mut pos: vec3_t = [0.0; 3];
            for i in 0..3 {
                pos[i] = mum13 * self.l.p.e.mOrigin1[i]
                    + group1 * self.mControl1[i]
                    + group2 * self.mControl2[i]
                    + mu3 * self.l.mOrigin2[i];
            }

            let tc2 = mu * tex;

            //Draw it
            self.DrawSegment(
                host,
                fx,
                &mut last_end,
                old_pos,
                pos,
                tc1,
                tc2,
                mu,
                mu - incr,
            );

            old_pos = pos;
            tc1 = tc2;

            mu += incr;
        }
        fx.drawnFx += 1;
    }
}
