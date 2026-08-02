//! Raven `CTrail`, the saber slash trail.
//!
//! Raven calls this class "an exception to the rule": cgame builds the four
//! verts and feeds them straight in through `FX_FeedTrail`, so the class runs no
//! spawn curves of its own.
//!
//! Class definition source: `oracle/codemp/client/FxPrimitives.h:174-212`
//! Method source: `oracle/codemp/client/FxPrimitives.cpp:1696-1810`

#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::shared::effect_trail_arg::effectTrailArgStruct_t;
use native_math::vector::vec3_t;
use native_types::qhandle_t;

use crate::fx::ceffect::EffectCore;
use crate::fx::fx_host::FxHost;
use crate::fx::fx_primitive::FxPrimitive;
use crate::fx::fx_system::FxSystem;
use crate::fx::fx_util::FX_AddPrimitive;

/// Raven's index names for the four trail corners.
///
/// Source: `oracle/codemp/client/FxPrimitives.cpp:1696-1699`
const NEW_MUZZLE: usize = 0;
const NEW_TIP: usize = 1;
const OLD_TIP: usize = 2;
const OLD_MUZZLE: usize = 3;

/// Raven `CTrail::TVert`, one corner of the trail quad.
///
/// The `cur*` fields are the interpolated values the draw reads. Raven left the
/// color and alpha interpolation commented out, so only `curST` moves.
///
/// Type definition source: `oracle/codemp/client/FxPrimitives.h:183-202`
#[derive(Clone, Copy, Debug, Default)]
pub struct TVert {
    pub origin: vec3_t,

    pub rgb: vec3_t,
    pub destrgb: vec3_t,
    pub curRGB: vec3_t,

    pub alpha: f32,
    pub destAlpha: f32,
    pub curAlpha: f32,

    pub ST: [f32; 2],
    pub destST: [f32; 2],
    pub curST: [f32; 2],
}

/// The `CTrail` fields, plus the `CEffect` core it inherited.
///
/// Class definition source: `oracle/codemp/client/FxPrimitives.h:174-212`
#[derive(Clone, Copy, Debug)]
pub struct CTrail {
    pub e: EffectCore,

    pub mVerts: [TVert; 4],
    pub mShader: qhandle_t,
}

impl Default for CTrail {
    /// Raven `CTrail::CTrail`, an empty body over the `CEffect` constructor.
    ///
    /// Raven leaves `mVerts` and `mShader` uninitialized. `FX_FeedTrail` writes
    /// every one of them before the primitive is scheduled, so zero is the one
    /// defined starting value (porting-rules §19).
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.h:208`
    fn default() -> Self {
        CTrail {
            e: EffectCore::default(),
            mVerts: [TVert::default(); 4],
            mShader: 0,
        }
    }
}

impl CTrail {
    /// Raven `CTrail::Draw`, two triangles sharing one vert buffer.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:1702-1774`
    pub fn Draw(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) {
        let mut verts = [polyVert_t {
            xyz: [0.0; 3],
            st: [0.0; 2],
            modulate: [0; 4],
        }; 3];

        // build the first tri out of the new muzzle...new tip...old muzzle
        verts[0].xyz = self.mVerts[NEW_MUZZLE].origin;
        verts[1].xyz = self.mVerts[NEW_TIP].origin;
        verts[2].xyz = self.mVerts[OLD_MUZZLE].origin;

        // Raven assigns a float to a byte field. A C conversion out of range is
        // undefined, so the port takes Rust's saturating cast (porting-rules §19).
        verts[0].modulate[0] = self.mVerts[NEW_MUZZLE].rgb[0] as u8;
        verts[0].modulate[1] = self.mVerts[NEW_MUZZLE].rgb[1] as u8;
        verts[0].modulate[2] = self.mVerts[NEW_MUZZLE].rgb[2] as u8;
        verts[0].modulate[3] = self.mVerts[NEW_MUZZLE].alpha as u8;

        verts[1].modulate[0] = self.mVerts[NEW_TIP].rgb[0] as u8;
        verts[1].modulate[1] = self.mVerts[NEW_TIP].rgb[1] as u8;
        verts[1].modulate[2] = self.mVerts[NEW_TIP].rgb[2] as u8;
        verts[1].modulate[3] = self.mVerts[NEW_TIP].alpha as u8;

        verts[2].modulate[0] = self.mVerts[OLD_MUZZLE].rgb[0] as u8;
        verts[2].modulate[1] = self.mVerts[OLD_MUZZLE].rgb[1] as u8;
        verts[2].modulate[2] = self.mVerts[OLD_MUZZLE].rgb[2] as u8;
        verts[2].modulate[3] = self.mVerts[OLD_MUZZLE].alpha as u8;

        verts[0].st[0] = self.mVerts[NEW_MUZZLE].curST[0];
        verts[0].st[1] = self.mVerts[NEW_MUZZLE].curST[1];
        verts[1].st[0] = self.mVerts[NEW_TIP].curST[0];
        verts[1].st[1] = self.mVerts[NEW_TIP].curST[1];
        verts[2].st[0] = self.mVerts[OLD_MUZZLE].curST[0];
        verts[2].st[1] = self.mVerts[OLD_MUZZLE].curST[1];

        // Add this tri
        host.AddPolyToScene(self.mShader, &verts);

        // build the second tri out of the old muzzle...old tip...new tip
        verts[0].xyz = self.mVerts[OLD_MUZZLE].origin;
        verts[1].xyz = self.mVerts[OLD_TIP].origin;
        verts[2].xyz = self.mVerts[NEW_TIP].origin;

        verts[0].modulate[0] = self.mVerts[OLD_MUZZLE].rgb[0] as u8;
        verts[0].modulate[1] = self.mVerts[OLD_MUZZLE].rgb[1] as u8;
        verts[0].modulate[2] = self.mVerts[OLD_MUZZLE].rgb[2] as u8;
        verts[0].modulate[3] = self.mVerts[OLD_MUZZLE].alpha as u8;

        verts[1].modulate[0] = self.mVerts[OLD_TIP].rgb[0] as u8;
        verts[1].modulate[1] = self.mVerts[OLD_TIP].rgb[1] as u8;
        verts[1].modulate[2] = self.mVerts[OLD_TIP].rgb[2] as u8;
        // Raven's own copy-paste bug: this alpha and the one below land on
        // `verts[0]`, so the second tri keeps the first tri's vert 1 and 2 alpha.
        // Source: `oracle/codemp/client/FxPrimitives.cpp:1755`
        verts[0].modulate[3] = self.mVerts[OLD_TIP].alpha as u8;

        verts[2].modulate[0] = self.mVerts[NEW_TIP].rgb[0] as u8;
        verts[2].modulate[1] = self.mVerts[NEW_TIP].rgb[1] as u8;
        verts[2].modulate[2] = self.mVerts[NEW_TIP].rgb[2] as u8;
        // Source: `oracle/codemp/client/FxPrimitives.cpp:1761`
        verts[0].modulate[3] = self.mVerts[NEW_TIP].alpha as u8;

        verts[0].st[0] = self.mVerts[OLD_MUZZLE].curST[0];
        verts[0].st[1] = self.mVerts[OLD_MUZZLE].curST[1];
        verts[1].st[0] = self.mVerts[OLD_TIP].curST[0];
        verts[1].st[1] = self.mVerts[OLD_TIP].curST[1];
        verts[2].st[0] = self.mVerts[NEW_TIP].curST[0];
        verts[2].st[1] = self.mVerts[NEW_TIP].curST[1];

        // Add this tri
        host.AddPolyToScene(self.mShader, &verts);

        fx.drawnFx += 1;
    }

    /// Raven `CTrail::Update`, an ST slide over the trail life.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:1779-1810`
    pub fn Update(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) -> bool {
        // Game pausing can cause dumb time things to happen, so kill the effect in this instance
        if self.e.mTimeStart > fx.clock.mTime {
            return false;
        }

        let perc = (self.e.mTimeEnd - fx.clock.mTime) as f32
            / (self.e.mTimeEnd - self.e.mTimeStart) as f32;

        for t in 0..4 {
            self.mVerts[t].curST[0] =
                self.mVerts[t].ST[0] * perc + self.mVerts[t].destST[0] * (1.0 - perc);
            if self.mVerts[t].curST[0] > 1.0 {
                self.mVerts[t].curST[0] = 1.0;
            }
            self.mVerts[t].curST[1] =
                self.mVerts[t].ST[1] * perc + self.mVerts[t].destST[1] * (1.0 - perc);
        }

        self.Draw(host, fx);

        true
    }
}

/// Raven `FX_FeedTrail` — copy the module's four verts in and schedule the trail.
///
/// This is the whole of `CG_FX_ADDPRIMITIVE`. The trail carries no spawn curves,
/// so the caller owns every value.
///
/// Source: `oracle/codemp/client/FxPrimitives.cpp:2315-2343`
pub fn FX_FeedTrail(fx: &mut FxSystem, host: &mut FxHost<'_, '_>, a: &effectTrailArgStruct_t) {
    let mut trail = CTrail::default();

    for i in 0..4 {
        trail.mVerts[i].origin = a.mVerts[i].origin;
        trail.mVerts[i].rgb = a.mVerts[i].rgb;
        trail.mVerts[i].destrgb = a.mVerts[i].destrgb;
        trail.mVerts[i].curRGB = a.mVerts[i].curRGB;
        trail.mVerts[i].alpha = a.mVerts[i].alpha;
        trail.mVerts[i].destAlpha = a.mVerts[i].destAlpha;
        trail.mVerts[i].curAlpha = a.mVerts[i].curAlpha;
        trail.mVerts[i].ST[0] = a.mVerts[i].ST[0];
        trail.mVerts[i].ST[1] = a.mVerts[i].ST[1];
        trail.mVerts[i].destST[0] = a.mVerts[i].destST[0];
        trail.mVerts[i].destST[1] = a.mVerts[i].destST[1];
        trail.mVerts[i].curST[0] = a.mVerts[i].curST[0];
        trail.mVerts[i].curST[1] = a.mVerts[i].curST[1];
    }

    trail.e.SetFlags(a.mSetFlags);

    trail.mShader = a.mShader;

    FX_AddPrimitive(fx, host, FxPrimitive::Trail(trail), a.mKillTime);
}
