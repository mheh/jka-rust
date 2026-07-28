//! Port of `oracle/codemp/cgame/fx_flechette.c` — flechette projectile, shrapnel and impact effects. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use mp_qshared::shared::q_math::VectorNormalize2;
use mp_qshared::shared::vec3_t;

use crate::local::centity_s::centity_t;
use crate::local::weapon_info_s::weaponInfo_t;
use crate::trap;
use crate::world::cg_context::CgContext;

/// Raven `FX_FlechetteProjectileThink` — plays the flechette bolt flight
/// effect at the entity's lerped origin, oriented along its velocity
/// (straight up if the entity isn't moving).
///
/// `weapon` goes unread in Raven's body; kept for signature parity.
/// Source: `oracle/codemp/cgame/fx_flechette.c:11-21`
pub fn FX_FlechetteProjectileThink(ctx: &mut CgContext, cent: &centity_t, weapon: &weaponInfo_t) {
    let _ = weapon;
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(cent.currentState.pos.trDelta, &mut forward) == 0.0 {
        forward[2] = 1.0;
    }

    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.flechetteShotEffect,
        &cent.lerpOrigin,
        &forward,
        -1,
        -1,
    );
}

/// Raven `FX_FlechetteWeaponHitWall` — plays the flechette wall-impact
/// effect at the hit point, oriented along the surface normal.
/// Source: `oracle/codemp/cgame/fx_flechette.c:28-31`
pub fn FX_FlechetteWeaponHitWall(ctx: &mut CgContext, origin: &vec3_t, normal: &vec3_t) {
    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.flechetteWallImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/// Raven `FX_FlechetteWeaponHitPlayer` — plays the flechette flesh-impact
/// effect against a hit target.
///
// PORT-NOTE: Raven's droid-impact branch (`trap_FX_PlayEffect(
// "blaster/droid_impact", ... )`) is commented out in the oracle, so the
// flesh-impact effect always plays regardless of `humanoid`; kept for
// signature parity, unread.
/// Source: `oracle/codemp/cgame/fx_flechette.c:38-48`
pub fn FX_FlechetteWeaponHitPlayer(
    ctx: &mut CgContext,
    origin: &vec3_t,
    normal: &vec3_t,
    humanoid: bool,
) {
    let _ = humanoid;

    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.flechetteFleshImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/// Raven `FX_FlechetteAltProjectileThink` — plays the flechette alt-fire
/// bolt flight effect at the entity's lerped origin, oriented along its
/// velocity (straight up if the entity isn't moving).
///
/// `weapon` goes unread in Raven's body; kept for signature parity.
/// Source: `oracle/codemp/cgame/fx_flechette.c:57-67`
pub fn FX_FlechetteAltProjectileThink(
    ctx: &mut CgContext,
    cent: &centity_t,
    weapon: &weaponInfo_t,
) {
    let _ = weapon;
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(cent.currentState.pos.trDelta, &mut forward) == 0.0 {
        forward[2] = 1.0;
    }

    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.flechetteAltShotEffect,
        &cent.lerpOrigin,
        &forward,
        -1,
        -1,
    );
}
