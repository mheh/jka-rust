//! Port of `oracle/codemp/cgame/fx_blaster.c` — blaster projectile and impact effects. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use mp_qshared::shared::q_math::VectorNormalize2;
use mp_qshared::shared::vec3_t;

use crate::local::centity_s::centity_t;
use crate::local::weapon_info_s::weaponInfo_t;
use crate::trap;
use crate::world::cg_context::CgContext;

/// Raven `FX_BlasterProjectileThink` — plays the blaster bolt flight effect at the
/// entity's lerped origin, oriented along its velocity (straight up if the
/// entity isn't moving).
///
/// `weapon` goes unread in Raven's body; kept for signature parity.
/// Source: `oracle/codemp/cgame/fx_blaster.c:11-21`
pub fn FX_BlasterProjectileThink(ctx: &mut CgContext, cent: &centity_t, _weapon: &weaponInfo_t) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(cent.currentState.pos.trDelta, &mut forward) == 0.0 {
        forward[2] = 1.0;
    }

    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.blasterShotEffect,
        &cent.lerpOrigin,
        &forward,
        -1,
        -1,
    );
}

/// Raven `FX_BlasterAltFireThink` — same flight effect as
/// [`FX_BlasterProjectileThink`], oriented along the entity's velocity.
///
/// `weapon` goes unread in Raven's body; kept for signature parity.
///
/// PORT-NOTE: Raven plays `cgs.effects.blasterShotEffect` here too, not a
/// distinct alt-fire effect, despite the fn name — preserved as-is.
/// Source: `oracle/codemp/cgame/fx_blaster.c:28-38`
pub fn FX_BlasterAltFireThink(ctx: &mut CgContext, cent: &centity_t, _weapon: &weaponInfo_t) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(cent.currentState.pos.trDelta, &mut forward) == 0.0 {
        forward[2] = 1.0;
    }

    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.blasterShotEffect,
        &cent.lerpOrigin,
        &forward,
        -1,
        -1,
    );
}

/// Raven `FX_BlasterWeaponHitWall` — plays the blaster wall-impact effect at
/// the hit point, oriented along the surface normal.
/// Source: `oracle/codemp/cgame/fx_blaster.c:45-48`
pub fn FX_BlasterWeaponHitWall(ctx: &mut CgContext, origin: &vec3_t, normal: &vec3_t) {
    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.blasterWallImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/// Raven `FX_BlasterWeaponHitPlayer` — plays the blaster flesh-impact effect
/// against a humanoid target, or the droid-impact effect otherwise.
/// Source: `oracle/codemp/cgame/fx_blaster.c:55-65`
pub fn FX_BlasterWeaponHitPlayer(
    ctx: &mut CgContext,
    origin: &vec3_t,
    normal: &vec3_t,
    humanoid: bool,
) {
    if humanoid {
        trap::FX_PlayEffectID(
            ctx.engine,
            ctx.world.cgs.effects.blasterFleshImpactEffect,
            origin,
            normal,
            -1,
            -1,
        );
    } else {
        trap::FX_PlayEffectID(
            ctx.engine,
            ctx.world.cgs.effects.blasterDroidImpactEffect,
            origin,
            normal,
            -1,
            -1,
        );
    }
}
