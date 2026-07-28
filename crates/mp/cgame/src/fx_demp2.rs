//! Port of `oracle/codemp/cgame/fx_demp2.c` — DEMP2 projectile, shock-sphere and impact effects. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use mp_qshared::shared::q_math::VectorNormalize2;
use mp_qshared::shared::vec3_t;

use crate::local::centity_s::centity_t;
use crate::local::weapon_info_s::weaponInfo_t;
use crate::trap;
use crate::world::cg_context::CgContext;

/// Raven `FX_DEMP2_ProjectileThink` — plays the DEMP2 bolt flight effect at
/// the entity's lerped origin, oriented along its velocity (straight up if
/// the entity isn't moving).
///
/// `weapon` goes unread in Raven's body; kept for signature parity.
/// Source: `oracle/codemp/cgame/fx_demp2.c:11-21`
pub fn FX_DEMP2_ProjectileThink(ctx: &mut CgContext, cent: &centity_t, _weapon: &weaponInfo_t) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(cent.currentState.pos.trDelta, &mut forward) == 0.0 {
        forward[2] = 1.0;
    }

    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.demp2ProjectileEffect,
        &cent.lerpOrigin,
        &forward,
        -1,
        -1,
    );
}

/// Raven `FX_DEMP2_HitWall` — plays the DEMP2 wall-impact effect at the hit
/// point, oriented along the surface normal.
/// Source: `oracle/codemp/cgame/fx_demp2.c:29-32`
pub fn FX_DEMP2_HitWall(ctx: &mut CgContext, origin: &vec3_t, normal: &vec3_t) {
    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.demp2WallImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/// Raven `FX_DEMP2_HitPlayer` — plays the DEMP2 flesh-impact effect against
/// a hit target.
///
/// `humanoid` goes unread in Raven's body (unlike the blaster's equivalent,
/// DEMP2 has no separate droid-impact effect); kept for signature parity.
/// Source: `oracle/codemp/cgame/fx_demp2.c:40-43`
pub fn FX_DEMP2_HitPlayer(ctx: &mut CgContext, origin: &vec3_t, normal: &vec3_t, humanoid: bool) {
    let _ = humanoid;

    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.demp2FleshImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/// Raven `FX_DEMP2_AltBeam` — dead: the entire body is commented out in the
/// oracle ("NOTENOTE Fix this after trap calls for all primitives are
/// created"), so this is a no-op preserved verbatim.
/// Source: `oracle/codemp/cgame/fx_demp2.c:50-237`
pub fn FX_DEMP2_AltBeam(
    _start: &vec3_t,
    _end: &vec3_t,
    _normal: &vec3_t,
    _targ1: &vec3_t,
    _targ2: &vec3_t,
) {
}
