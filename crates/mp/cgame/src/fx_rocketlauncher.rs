//! Port of `oracle/codemp/cgame/fx_rocketlauncher.c` — rocket launcher projectile and impact effects. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use mp_qshared::shared::q_math::VectorNormalize2;
use mp_qshared::shared::vec3_t;

use crate::local::centity_s::centity_t;
use crate::local::weapon_info_s::weaponInfo_t;
use crate::trap;
use crate::world::cg_context::CgContext;

/// Raven `FX_RocketProjectileThink` — plays the rocket flight effect at the
/// entity's lerped origin, oriented along its velocity (straight up if the
/// entity isn't moving).
///
/// `weapon` goes unread in Raven's body; kept for signature parity.
/// Source: `oracle/codemp/cgame/fx_rocketlauncher.c:11-21`
pub fn FX_RocketProjectileThink(ctx: &mut CgContext, cent: &centity_t, _weapon: &weaponInfo_t) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(cent.currentState.pos.trDelta, &mut forward) == 0.0 {
        forward[2] = 1.0;
    }

    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.rocketShotEffect,
        &cent.lerpOrigin,
        &forward,
        -1,
        -1,
    );
}

/// Raven `FX_RocketHitWall` — plays the rocket explosion effect at the hit
/// point, oriented along the surface normal.
/// Source: `oracle/codemp/cgame/fx_rocketlauncher.c:29-32`
pub fn FX_RocketHitWall(ctx: &mut CgContext, origin: &vec3_t, normal: &vec3_t) {
    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.rocketExplosionEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/// Raven `FX_RocketHitPlayer` — plays the rocket explosion effect against a
/// target.
///
/// PORT-NOTE: `humanoid` goes unread in Raven's body — the same explosion
/// effect plays regardless of target type. Kept for signature parity.
/// Source: `oracle/codemp/cgame/fx_rocketlauncher.c:40-43`
pub fn FX_RocketHitPlayer(ctx: &mut CgContext, origin: &vec3_t, normal: &vec3_t, _humanoid: bool) {
    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.rocketExplosionEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/// Raven `FX_RocketAltProjectileThink` — same flight effect as
/// [`FX_RocketProjectileThink`], oriented along the entity's velocity.
///
/// `weapon` goes unread in Raven's body; kept for signature parity.
///
/// PORT-NOTE: Raven plays `cgs.effects.rocketShotEffect` here too, not a
/// distinct alt-fire effect, despite the fn name — preserved as-is.
/// Source: `oracle/codemp/cgame/fx_rocketlauncher.c:51-61`
pub fn FX_RocketAltProjectileThink(ctx: &mut CgContext, cent: &centity_t, _weapon: &weaponInfo_t) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(cent.currentState.pos.trDelta, &mut forward) == 0.0 {
        forward[2] = 1.0;
    }

    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.rocketShotEffect,
        &cent.lerpOrigin,
        &forward,
        -1,
        -1,
    );
}
