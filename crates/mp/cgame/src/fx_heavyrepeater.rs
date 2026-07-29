//! Port of `oracle/codemp/cgame/fx_heavyrepeater.c` — heavy repeater projectile, concussion orb and impact effects. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::tr_types::{RF_DISTORTION, RF_RGB_TINT};
use mp_qshared::shared::q_math::{
    _VectorCopy, _VectorScale, _VectorSubtract, vectoangles, AnglesToAxis, VectorLength,
    VectorNormalize, VectorNormalize2, ROLL,
};
use mp_qshared::shared::vec3_t;

use crate::local::centity_s::centity_t;
use crate::local::weapon_info_s::weaponInfo_t;
use crate::trap;
use crate::world::cg_context::CgContext;

/// Raven `FX_RepeaterProjectileThink` — plays the repeater bolt flight effect
/// at the entity's lerped origin, oriented along its velocity (straight up if
/// the entity isn't moving).
///
/// `weapon` goes unread in Raven's body; kept for signature parity.
/// Source: `oracle/codemp/cgame/fx_heavyrepeater.c:11-21`
pub fn FX_RepeaterProjectileThink(ctx: &mut CgContext, cent: &centity_t, _weapon: &weaponInfo_t) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(cent.currentState.pos.trDelta, &mut forward) == 0.0 {
        forward[2] = 1.0;
    }

    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.repeaterProjectileEffect,
        &cent.lerpOrigin,
        &forward,
        -1,
        -1,
    );
}

/// Raven `FX_RepeaterHitWall` — plays the repeater wall-impact effect at the
/// hit point, oriented along the surface normal.
/// Source: `oracle/codemp/cgame/fx_heavyrepeater.c:29-32`
pub fn FX_RepeaterHitWall(ctx: &mut CgContext, origin: &vec3_t, normal: &vec3_t) {
    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.repeaterWallImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/// Raven `FX_RepeaterHitPlayer` — plays the repeater flesh-impact effect.
///
/// `humanoid` goes unread in Raven's body; kept for signature parity.
/// Source: `oracle/codemp/cgame/fx_heavyrepeater.c:40-43`
pub fn FX_RepeaterHitPlayer(
    ctx: &mut CgContext,
    origin: &vec3_t,
    normal: &vec3_t,
    _humanoid: bool,
) {
    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.repeaterFleshImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/// Raven `CG_DistortionOrb` — draws the render-to-texture half-sphere
/// distortion orb over the heavy repeater alt-fire projectile, oriented to
/// face the viewer and spinning ("screwdriver" effect) via `cent->trickAlpha`.
///
/// Bails out early when `cg_renderToTextureFX` is off, or when the entity
/// sits right on the view origin (degenerate facing axis).
/// Source: `oracle/codemp/cgame/fx_heavyrepeater.c:45-111`
pub fn CG_DistortionOrb(ctx: &mut CgContext, cent: &mut centity_t) {
    let scale: f32 = 0.5;

    if ctx.world.cvars.cg_renderToTextureFX.integer == 0 {
        return;
    }

    let mut ent = refEntity_t::zeroed();

    _VectorCopy(cent.lerpOrigin, &mut ent.origin);

    _VectorSubtract(ent.origin, ctx.world.cg.refdef.vieworg, &mut ent.axis[0]);
    let vLen = VectorLength(ent.axis[0]);
    if VectorNormalize(&mut ent.axis[0]) <= 0.1 {
        // Entity is right on vieworg.  quit.
        return;
    }

    let mut ang: vec3_t = [0.0; 3];
    vectoangles(ent.axis[0], &mut ang);
    ang[ROLL] = cent.trickAlpha as f32;
    cent.trickAlpha += 16; // spin the half-sphere to give a "screwdriver" effect
    AnglesToAxis(ang, ent.axis.as_mut_ptr());

    // radius must be a power of 2, and is the actual captured texture size
    if vLen < 128.0 {
        ent.radius = 256.0;
    } else if vLen < 256.0 {
        ent.radius = 128.0;
    } else if vLen < 512.0 {
        ent.radius = 64.0;
    } else {
        ent.radius = 32.0;
    }

    let axis0 = ent.axis[0];
    _VectorScale(axis0, scale, &mut ent.axis[0]);
    let axis1 = ent.axis[1];
    _VectorScale(axis1, scale, &mut ent.axis[1]);
    let axis2 = ent.axis[2];
    _VectorScale(axis2, -scale, &mut ent.axis[2]);

    ent.hModel = ctx.world.cgs.media.halfShieldModel;
    ent.customShader = 0; // cgs.media.halfShieldShader

    // tint the whole thing a shade of blue
    ent.renderfx = RF_DISTORTION | RF_RGB_TINT;
    ent.shaderRGBA[0] = 200;
    ent.shaderRGBA[1] = 200;
    ent.shaderRGBA[2] = 255;

    trap::R_AddRefEntityToScene(ctx.engine, &ent);
}

/// Raven `FX_RepeaterAltProjectileThink` — plays the concussion-ball flight
/// effect at the entity's lerped origin, oriented along its velocity (straight
/// up if the entity isn't moving), with the distortion orb layered on top when
/// `cg_repeaterOrb` is set.
///
/// `weapon` goes unread in Raven's body; kept for signature parity. `cent` is
/// `&mut` because the orb spins itself through `cent->trickAlpha`.
/// Source: `oracle/codemp/cgame/fx_heavyrepeater.c:119-133`
pub fn FX_RepeaterAltProjectileThink(
    ctx: &mut CgContext,
    cent: &mut centity_t,
    _weapon: &weaponInfo_t,
) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(cent.currentState.pos.trDelta, &mut forward) == 0.0 {
        forward[2] = 1.0;
    }

    if ctx.world.cvars.cg_repeaterOrb.integer != 0 {
        CG_DistortionOrb(ctx, cent);
    }
    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.repeaterAltProjectileEffect,
        &cent.lerpOrigin,
        &forward,
        -1,
        -1,
    );
}

/// Raven `FX_RepeaterAltHitWall` — plays the repeater alt-fire wall-impact
/// effect at the hit point, oriented along the surface normal.
/// Source: `oracle/codemp/cgame/fx_heavyrepeater.c:141-144`
pub fn FX_RepeaterAltHitWall(ctx: &mut CgContext, origin: &vec3_t, normal: &vec3_t) {
    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.repeaterAltWallImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/// Raven `FX_RepeaterAltHitPlayer` — plays the repeater alt-fire flesh-impact
/// effect.
///
/// `humanoid` goes unread in Raven's body; kept for signature parity.
///
/// PORT-NOTE: Raven plays `cgs.effects.repeaterAltWallImpactEffect` here too,
/// not a distinct alt flesh-impact effect (there isn't one — `cgEffects_t` has
/// no `repeaterAltFleshImpactEffect` field), despite the fn name — preserved
/// as-is.
/// Source: `oracle/codemp/cgame/fx_heavyrepeater.c:152-155`
pub fn FX_RepeaterAltHitPlayer(
    ctx: &mut CgContext,
    origin: &vec3_t,
    normal: &vec3_t,
    _humanoid: bool,
) {
    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.repeaterAltWallImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}
