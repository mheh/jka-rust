//! Raven `CFlash`, the full screen or localized muzzle flash.
//!
//! A plain flash pins itself 12 units in front of the eye at a fixed radius. A
//! `FX_LOCALIZED_FLASH` projects the world point to screen space once at spawn
//! and draws as a 2D pic from then on.
//!
//! Class definition source: `oracle/codemp/client/FxPrimitives.h:351-374`
//! Method source: `oracle/codemp/client/FxPrimitives.cpp:2194-2312`

#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use native_math::qmath::AngleVectors;
use native_math::vector::vec3_t;

use crate::fx::cparticle::{
    dot_product, vector_ma_in_place, vector_normalize, vector_scale, vector_subtract, ParticleCore,
};
use crate::fx::fx_flags::FX_LOCALIZED_FLASH;
use crate::fx::fx_host::FxHost;
use crate::fx::fx_scheduler::fx_add_2d_effect;
use crate::fx::fx_system::FxSystem;

/// The `CFlash` fields, plus the `CParticle` core it inherited.
///
/// `mScreenX` and `mScreenY` hold the projected 2D point, and they stay at the
/// constructor zero when the projection fails.
///
/// Class definition source: `oracle/codemp/client/FxPrimitives.h:351-374`
#[derive(Clone, Copy, Debug)]
pub struct CFlash {
    pub p: ParticleCore,

    // kef -- mScreenX and mScreenY are used for flashes that are FX_LOCALIZED_FLASH
    pub mScreenX: f32,
    pub mScreenY: f32,
    pub mRadiusModifier: f32,
}

impl Default for CFlash {
    /// Raven `CFlash::CFlash`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.h:355-359`
    fn default() -> Self {
        CFlash {
            p: ParticleCore::default(),
            mScreenX: 0.0,
            mScreenY: 0.0,
            mRadiusModifier: 1.0,
        }
    }
}

impl CFlash {
    /// Raven `CFlash::Cull`. A flash is always in view, so it never culls.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.h:365`
    pub fn Cull(&self) -> bool {
        false
    }

    /// Raven `CFlash::Init`, the one-time distance fade and screen projection.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:2246-2284`
    pub fn Init(&mut self, fx: &FxSystem) {
        // 10/19/01 kef -- maybe we want to do something different here for localized flashes, but right
        //now I want to be sure that whatever RGB changes occur to a non-localized flash will also occur
        //to a localized flash (so I'll have the same initial RGBA values for both...I need them sync'd for an effect)
        let max_range: f32 = 900.0;

        let mut dif = vector_subtract(&self.p.e.mOrigin1, &fx.refdef.vieworg);
        let dis = vector_normalize(&mut dif);

        let mut modulate = dot_product(&dif, &fx.refdef.viewaxis[0]);

        if dis > max_range || (modulate < 0.5 && dis > 100.0) {
            modulate = 0.0;
        } else if modulate < 0.5 && dis <= 100.0 {
            modulate += 1.1;
        }

        modulate *= 1.0 - ((dis * dis) / (max_range * max_range));

        self.p.mRGBStart = vector_scale(&self.p.mRGBStart, modulate);
        self.p.mRGBEnd = vector_scale(&self.p.mRGBEnd, modulate);

        if self.p.e.mFlags & FX_LOCALIZED_FLASH != 0 {
            // A failed projection leaves the two screen fields at their old value,
            // which is what Raven's untouched out-params do.
            if let Some((x, y)) = FX_WorldToScreen(fx, self.p.e.mOrigin1) {
                self.mScreenX = x;
                self.mScreenY = y;
            }

            // modify size of localized flash based on distance to effect (but not orientation)
            if dis > 100.0 && dis < max_range {
                self.mRadiusModifier = 1.0 - ((dis * dis) / (max_range * max_range));
            }
        }
    }

    /// Raven `CFlash::Draw`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:2287-2312`
    pub fn Draw(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) {
        self.p.e.mRefEnt.reType = refEntityType_t::RT_SPRITE;

        if self.p.e.mFlags & FX_LOCALIZED_FLASH != 0 {
            let color = [
                self.p.e.mRefEnt.shaderRGBA[0] as f32 / 255.0,
                self.p.e.mRefEnt.shaderRGBA[1] as f32 / 255.0,
                self.p.e.mRefEnt.shaderRGBA[2] as f32 / 255.0,
                self.p.e.mRefEnt.shaderRGBA[3] as f32 / 255.0,
            ];

            // add this 2D effect to the proper list. it will get drawn after the RenderScene call
            fx_add_2d_effect(
                fx,
                self.mScreenX,
                self.mScreenY,
                self.p.e.mRefEnt.radius,
                self.p.e.mRefEnt.radius,
                color,
                self.p.e.mRefEnt.customShader,
            );
        } else {
            self.p.e.mRefEnt.origin = fx.refdef.vieworg;
            vector_ma_in_place(&mut self.p.e.mRefEnt.origin, 12.0, &fx.refdef.viewaxis[0]);
            self.p.e.mRefEnt.radius = 11.0;

            host.AddFxToScene(Some(&self.p.e.mRefEnt));
        }
        fx.drawnFx += 1;
    }

    /// Raven `CFlash::Update`. A flash runs no cull and no time check.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:2194-2209`
    pub fn Update(&mut self, host: &mut FxHost<'_, '_>, fx: &mut FxSystem) -> bool {
        if !self.p.UpdateOrigin(host, fx) {
            // we are marked for death
            return false;
        }

        self.p.UpdateSize(host, fx);
        self.p.e.mRefEnt.radius *= self.mRadiusModifier;
        self.p.UpdateRGB(host, fx);
        self.p.UpdateAlpha(host, fx);

        self.Draw(host, fx);
        true
    }
}

/// Raven `FX_WorldToScreen`, the projection `CFlash::Init` uses.
///
/// The screen point comes out in virtual 640x480 coordinates, because the 2D
/// draw path adjusts for the real resolution itself. `None` is Raven's `false`
/// return for a point behind the near plane.
///
/// Source: `oracle/codemp/client/FxPrimitives.cpp:2211-2243`
fn FX_WorldToScreen(fx: &FxSystem, world_coord: vec3_t) -> Option<(f32, f32)> {
    //NOTE: did it this way because most draw functions expect virtual 640x480 coords
    //	and adjust them for current resolution
    let xcenter: i32 = 640 / 2;
    let ycenter: i32 = 480 / 2;

    let local = vector_subtract(&world_coord, &fx.refdef.vieworg);

    let mut vfwd: vec3_t = [0.0; 3];
    let mut vright: vec3_t = [0.0; 3];
    let mut vup: vec3_t = [0.0; 3];
    AngleVectors(
        fx.refdef.viewangles,
        Some(&mut vfwd),
        Some(&mut vright),
        Some(&mut vup),
    );

    let transformed: vec3_t = [
        dot_product(&local, &vright),
        dot_product(&local, &vup),
        dot_product(&local, &vfwd),
    ];

    // Make sure Z is not negative.
    if (transformed[2] as f64) < 0.01 {
        return None;
    }
    // Simple convert to screen coords.
    // Raven's `90.0` and the divide by the fov are double, so the scale factor
    // rounds to float only on the store.
    let xzi = ((xcenter as f32 / transformed[2]) as f64 * (90.0 / fx.refdef.fov_x as f64)) as f32;
    let yzi = ((ycenter as f32 / transformed[2]) as f64 * (90.0 / fx.refdef.fov_y as f64)) as f32;

    let x = xcenter as f32 + xzi * transformed[0];
    let y = ycenter as f32 - yzi * transformed[1];

    Some((x, y))
}
