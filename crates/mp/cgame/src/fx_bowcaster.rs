//! Port of `oracle/codemp/cgame/fx_bowcaster.c` — bowcaster projectile and impact effects. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use mp_qshared::shared::q_math::VectorNormalize2;
use mp_qshared::shared::vec3_t;

use crate::local::centity_s::centity_t;
use crate::local::weapon_info_s::weaponInfo_t;
use crate::trap;
use crate::world::cg_context::CgContext;

/// Raven `FX_BowcasterProjectileThink` — plays the bowcaster bolt flight
/// effect at the entity's lerped origin, oriented along its velocity
/// (straight up if the entity isn't moving).
///
/// `weapon` goes unread in Raven's body; kept for signature parity.
/// Source: `oracle/codemp/cgame/fx_bowcaster.c:11-21`
pub fn FX_BowcasterProjectileThink(ctx: &mut CgContext, cent: &centity_t, _weapon: &weaponInfo_t) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(cent.currentState.pos.trDelta, &mut forward) == 0.0 {
        forward[2] = 1.0;
    }

    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.bowcasterShotEffect,
        &cent.lerpOrigin,
        &forward,
        -1,
        -1,
    );
}

/// Raven `FX_BowcasterHitWall` — plays the bowcaster wall-impact effect at
/// the hit point, oriented along the surface normal.
/// Source: `oracle/codemp/cgame/fx_bowcaster.c:29-32`
pub fn FX_BowcasterHitWall(ctx: &mut CgContext, origin: &vec3_t, normal: &vec3_t) {
    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.bowcasterImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/// Raven `FX_BowcasterHitPlayer` — plays the bowcaster impact effect against
/// a target.
///
/// PORT-NOTE: `humanoid` goes unread in Raven's body — unlike the blaster's
/// hit-player fn, the bowcaster plays the same impact effect regardless of
/// target type. Kept for signature parity.
/// Source: `oracle/codemp/cgame/fx_bowcaster.c:40-43`
pub fn FX_BowcasterHitPlayer(
    ctx: &mut CgContext,
    origin: &vec3_t,
    normal: &vec3_t,
    _humanoid: bool,
) {
    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.bowcasterImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/// Raven `FX_BowcasterAltProjectileThink` — same flight effect as
/// [`FX_BowcasterProjectileThink`], oriented along the entity's velocity.
///
/// `weapon` goes unread in Raven's body; kept for signature parity.
///
/// PORT-NOTE: Raven plays `cgs.effects.bowcasterShotEffect` here too, not a
/// distinct alt-fire effect, despite the fn name — preserved as-is.
/// Source: `oracle/codemp/cgame/fx_bowcaster.c:51-61`
pub fn FX_BowcasterAltProjectileThink(
    ctx: &mut CgContext,
    cent: &centity_t,
    _weapon: &weaponInfo_t,
) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(cent.currentState.pos.trDelta, &mut forward) == 0.0 {
        forward[2] = 1.0;
    }

    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.bowcasterShotEffect,
        &cent.lerpOrigin,
        &forward,
        -1,
        -1,
    );
}
