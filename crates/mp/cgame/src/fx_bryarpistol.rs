//! Port of `oracle/codemp/cgame/fx_bryarpistol.c` — bryar pistol projectile and impact effects. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_qshared::shared::q_math::VectorNormalize2;
use mp_qshared::shared::vec3_t;

use crate::local::centity_s::centity_t;
use crate::local::weapon_info_s::weaponInfo_t;
use crate::trap;
use crate::world::cg_context::CgContext;

/// Raven `WHITE` — flat white color used by the concussion alt-shot beam.
///
/// Source: `oracle/codemp/cgame/fx_bryarpistol.c:219`
const WHITE: vec3_t = [1.0, 1.0, 1.0];

/// Raven `BRIGHT` — the "beef" beam layer's tint, laid under `WHITE`'s core line.
///
/// Source: `oracle/codemp/cgame/fx_bryarpistol.c:220`
const BRIGHT: vec3_t = [0.75, 0.5, 1.0];

// PORT-NOTE: `FX_ALPHA_LINEAR`/`FX_SIZE_LINEAR` are `fx_local.h` flags this TU
// includes but doesn't define itself, so they're absent from the packet's
// file-scope constants list; the packet's own oracle slice for
// `FX_ConcAltShot` uses them by name, so they're transcribed here rather than
// deferred.
/// Raven `FX_ALPHA_LINEAR` — `trap_FX_AddLine` flag: interpolate alpha linearly
/// between `alpha1`/`alpha2` over the line's lifetime.
///
/// Source: `oracle/codemp/cgame/fx_local.h:6`
const FX_ALPHA_LINEAR: c_int = 0x00000001;

/// Raven `FX_SIZE_LINEAR` — `trap_FX_AddLine` flag: interpolate width linearly
/// between `size1`/`size2` over the line's lifetime.
///
/// Source: `oracle/codemp/cgame/fx_local.h:7`
const FX_SIZE_LINEAR: c_int = 0x00000100;

/// Raven `FX_BryarProjectileThink` — plays the bryar bolt flight effect at the
/// entity's lerped origin, oriented along its velocity (straight up if the
/// entity isn't moving).
///
/// `weapon` goes unread in Raven's body; kept for signature parity.
/// Source: `oracle/codemp/cgame/fx_bryarpistol.c:15-25`
pub fn FX_BryarProjectileThink(ctx: &mut CgContext, cent: &centity_t, _weapon: &weaponInfo_t) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(cent.currentState.pos.trDelta, &mut forward) == 0.0 {
        forward[2] = 1.0;
    }

    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.bryarShotEffect,
        &cent.lerpOrigin,
        &forward,
        -1,
        -1,
    );
}

/// Raven `FX_BryarHitWall` — plays the bryar wall-impact effect at the hit
/// point, oriented along the surface normal.
/// Source: `oracle/codemp/cgame/fx_bryarpistol.c:32-35`
pub fn FX_BryarHitWall(ctx: &mut CgContext, origin: &vec3_t, normal: &vec3_t) {
    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.bryarWallImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/// Raven `FX_BryarHitPlayer` — plays the bryar flesh-impact effect against a
/// humanoid target, or the droid-impact effect otherwise.
/// Source: `oracle/codemp/cgame/fx_bryarpistol.c:42-52`
pub fn FX_BryarHitPlayer(ctx: &mut CgContext, origin: &vec3_t, normal: &vec3_t, humanoid: bool) {
    if humanoid {
        trap::FX_PlayEffectID(
            ctx.engine,
            ctx.world.cgs.effects.bryarFleshImpactEffect,
            origin,
            normal,
            -1,
            -1,
        );
    } else {
        trap::FX_PlayEffectID(
            ctx.engine,
            ctx.world.cgs.effects.bryarDroidImpactEffect,
            origin,
            normal,
            -1,
            -1,
        );
    }
}

/// Raven `FX_BryarAltProjectileThink` — plays the powerup-charge shot effect
/// once per charge level above 1 (SP's `gent->count` charge counter isn't
/// reachable client-side, so `currentState.generic1` stands in, per Raven's
/// own comment), then plays the normal bolt flight effect.
///
/// `weapon` goes unread in Raven's body; kept for signature parity.
/// Source: `oracle/codemp/cgame/fx_bryarpistol.c:64-84`
pub fn FX_BryarAltProjectileThink(ctx: &mut CgContext, cent: &centity_t, _weapon: &weaponInfo_t) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(cent.currentState.pos.trDelta, &mut forward) == 0.0 {
        forward[2] = 1.0;
    }

    // see if we have some sort of extra charge going on
    for _t in 1..cent.currentState.generic1 {
        // just add ourselves over, and over, and over when we are charged
        trap::FX_PlayEffectID(
            ctx.engine,
            ctx.world.cgs.effects.bryarPowerupShotEffect,
            &cent.lerpOrigin,
            &forward,
            -1,
            -1,
        );
    }

    // for ( int t = 1; t < cent->gent->count; t++ )	// The single player stores the charge in count, which isn't accessible on the client

    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.bryarShotEffect,
        &cent.lerpOrigin,
        &forward,
        -1,
        -1,
    );
}

/// Raven `FX_BryarAltHitWall` — plays a wall-impact effect scaled by charge
/// `power`: 4-5 the heaviest, 2-3 medium, anything else (incl. 0-1) the base
/// effect.
/// Source: `oracle/codemp/cgame/fx_bryarpistol.c:91-109`
pub fn FX_BryarAltHitWall(ctx: &mut CgContext, origin: &vec3_t, normal: &vec3_t, power: c_int) {
    match power {
        4 | 5 => trap::FX_PlayEffectID(
            ctx.engine,
            ctx.world.cgs.effects.bryarWallImpactEffect3,
            origin,
            normal,
            -1,
            -1,
        ),
        2 | 3 => trap::FX_PlayEffectID(
            ctx.engine,
            ctx.world.cgs.effects.bryarWallImpactEffect2,
            origin,
            normal,
            -1,
            -1,
        ),
        _ => trap::FX_PlayEffectID(
            ctx.engine,
            ctx.world.cgs.effects.bryarWallImpactEffect,
            origin,
            normal,
            -1,
            -1,
        ),
    }
}

/// Raven `FX_BryarAltHitPlayer` — plays the bryar flesh-impact effect against
/// a humanoid target, or the droid-impact effect otherwise.
/// Source: `oracle/codemp/cgame/fx_bryarpistol.c:116-126`
pub fn FX_BryarAltHitPlayer(ctx: &mut CgContext, origin: &vec3_t, normal: &vec3_t, humanoid: bool) {
    if humanoid {
        trap::FX_PlayEffectID(
            ctx.engine,
            ctx.world.cgs.effects.bryarFleshImpactEffect,
            origin,
            normal,
            -1,
            -1,
        );
    } else {
        trap::FX_PlayEffectID(
            ctx.engine,
            ctx.world.cgs.effects.bryarDroidImpactEffect,
            origin,
            normal,
            -1,
            -1,
        );
    }
}

/// Raven `FX_TurretProjectileThink` — plays the turret bolt flight effect at
/// the entity's lerped origin, oriented along its velocity (straight up if
/// the entity isn't moving).
///
/// `weapon` goes unread in Raven's body; kept for signature parity.
/// Source: `oracle/codemp/cgame/fx_bryarpistol.c:135-145`
pub fn FX_TurretProjectileThink(ctx: &mut CgContext, cent: &centity_t, _weapon: &weaponInfo_t) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(cent.currentState.pos.trDelta, &mut forward) == 0.0 {
        forward[2] = 1.0;
    }

    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.turretShotEffect,
        &cent.lerpOrigin,
        &forward,
        -1,
        -1,
    );
}

/// Raven `FX_TurretHitWall` — plays the bryar wall-impact effect at the hit
/// point, oriented along the surface normal.
///
/// PORT-NOTE: reuses `bryarWallImpactEffect`, not a distinct turret effect,
/// despite the fn name — preserved as-is.
/// Source: `oracle/codemp/cgame/fx_bryarpistol.c:152-155`
pub fn FX_TurretHitWall(ctx: &mut CgContext, origin: &vec3_t, normal: &vec3_t) {
    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.bryarWallImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/// Raven `FX_TurretHitPlayer` — plays the bryar flesh-impact effect against a
/// humanoid target, or the droid-impact effect otherwise.
///
/// PORT-NOTE: reuses the bryar impact effects, not distinct turret effects,
/// despite the fn name — preserved as-is.
/// Source: `oracle/codemp/cgame/fx_bryarpistol.c:162-172`
pub fn FX_TurretHitPlayer(ctx: &mut CgContext, origin: &vec3_t, normal: &vec3_t, humanoid: bool) {
    if humanoid {
        trap::FX_PlayEffectID(
            ctx.engine,
            ctx.world.cgs.effects.bryarFleshImpactEffect,
            origin,
            normal,
            -1,
            -1,
        );
    } else {
        trap::FX_PlayEffectID(
            ctx.engine,
            ctx.world.cgs.effects.bryarDroidImpactEffect,
            origin,
            normal,
            -1,
            -1,
        );
    }
}

/// Raven `FX_ConcussionHitWall` — plays the concussion impact effect at the
/// hit point, oriented along the surface normal.
/// Source: `oracle/codemp/cgame/fx_bryarpistol.c:182-185`
pub fn FX_ConcussionHitWall(ctx: &mut CgContext, origin: &vec3_t, normal: &vec3_t) {
    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.concussionImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/// Raven `FX_ConcussionHitPlayer` — plays the concussion impact effect;
/// unlike the other weapons in this file, Raven doesn't branch on `humanoid`
/// here (`humanoid` goes unread in Raven's body; kept for signature parity).
/// Source: `oracle/codemp/cgame/fx_bryarpistol.c:192-195`
pub fn FX_ConcussionHitPlayer(
    ctx: &mut CgContext,
    origin: &vec3_t,
    normal: &vec3_t,
    _humanoid: bool,
) {
    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.concussionImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/// Raven `FX_ConcussionProjectileThink` — plays the concussion bolt flight
/// effect at the entity's lerped origin, oriented along its velocity
/// (straight up if the entity isn't moving).
///
/// `weapon` goes unread in Raven's body; kept for signature parity.
/// Source: `oracle/codemp/cgame/fx_bryarpistol.c:202-212`
pub fn FX_ConcussionProjectileThink(ctx: &mut CgContext, cent: &centity_t, _weapon: &weaponInfo_t) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(cent.currentState.pos.trDelta, &mut forward) == 0.0 {
        forward[2] = 1.0;
    }

    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.concussionShotEffect,
        &cent.lerpOrigin,
        &forward,
        -1,
        -1,
    );
}

/// Raven `FX_ConcAltShot` — draws the concussion rifle's alt-fire beam as two
/// stacked lines: a thin white core (`gfx/effects/blueLine`) and a wider,
/// dimmer "beef" layer underneath (`gfx/misc/whiteline2`).
/// Source: `oracle/codemp/cgame/fx_bryarpistol.c:222-237`
pub fn FX_ConcAltShot(ctx: &mut CgContext, start: &vec3_t, end: &vec3_t) {
    // "concussion/beam"
    let core_shader = trap::R_RegisterShader(ctx.engine, "gfx/effects/blueLine");
    trap::FX_AddLine(
        ctx.engine,
        start,
        end,
        0.1,
        10.0,
        0.0,
        1.0,
        0.0,
        0.0,
        &WHITE,
        &WHITE,
        0.0,
        175,
        core_shader,
        FX_SIZE_LINEAR | FX_ALPHA_LINEAR,
    );

    // add some beef
    let beef_shader = trap::R_RegisterShader(ctx.engine, "gfx/misc/whiteline2");
    trap::FX_AddLine(
        ctx.engine,
        start,
        end,
        0.1,
        7.0,
        0.0,
        1.0,
        0.0,
        0.0,
        &BRIGHT,
        &BRIGHT,
        0.0,
        150,
        beef_shader,
        FX_SIZE_LINEAR | FX_ALPHA_LINEAR,
    );
}
