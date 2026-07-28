//! Port of `oracle/codemp/cgame/cg_localents.c` — the local-entity effect pool and its per-type add functions. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_bg::bg_misc::BG_EvaluateTrajectoryDelta;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::q_math::{_DotProduct, _VectorCopy, _VectorMA, _VectorScale};
use mp_qshared::shared::{
    qtrue, sfxHandle_t, trType_t, trajectory_t, vec3_t, CHAN_AUTO, ENTITYNUM_WORLD,
};

use crate::local::le_bounce_sound_type_t::leBounceSoundType_t;
use crate::local::le_mark_type_t::leMarkType_t;
use crate::local::local_entity_s::localEntity_t;
use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::cg_world::CgWorld;

// FILE-SCOPE CONSTANTS
// Source: `oracle/codemp/cgame/cg_localents.c:9,637`

// Raven `#define MAX_LOCAL_ENTITIES 512` already lives on `CgWorld` as
// `world::cg_world::MAX_LOCAL_ENTITIES`, sizing `CgWorld::cg_localEntities`
// (DEC-46.3) — not redeclared here.

/// Raven `#define NUMBER_SIZE 8` — digit-glyph count `CG_AddScorePlum` sizes its
/// per-digit refEntity array with (a later wave's fn; not consumed here).
/// Source: `oracle/codemp/cgame/cg_localents.c:637`
pub const NUMBER_SIZE: usize = 8;

/// Raven `CG_InitLocalEntities` — resets the local-entity pool to all-free.
///
/// DEC-46.3 folds `cg_localEntities` plus its `cg_activeLocalEntities`/
/// `cg_freeLocalEntities` intrusive-list heads into `CgWorld::cg_localEntities`,
/// an `EffectPool`; `EffectPool::clear` reproduces the `memset` + free-chain
/// rebuild this did (called at startup and on tournament restart).
/// Source: `oracle/codemp/cgame/cg_localents.c:21-31`
pub fn CG_InitLocalEntities(world: &mut CgWorld) {
    world.cg_localEntities.clear();
}

/// Raven `CG_FragmentBounceMark` — was going to leave an impact mark for a
/// bouncing fragment; the actual `CG_ImpactMark` calls are commented out in the
/// oracle, so this only draws (and discards) a radius and clears the mark type.
///
/// Raven: don't allow a fragment to make multiple marks, or they pile up while
/// settling.
///
/// The dead `radius` draws are preserved (not skipped) because `rand()` still
/// advances the shared RNG stream — dropping them would desync parity with the
/// oracle's later draws.
/// Source: `oracle/codemp/cgame/cg_localents.c:136-155`
pub fn CG_FragmentBounceMark(world: &mut CgWorld, le: &mut localEntity_t, _trace: &trace_t) {
    if le.leMarkType == leMarkType_t::LEMT_BLOOD {
        let _radius = 16 + (world.bg_state.rng.rand() & 31);
    } else if le.leMarkType == leMarkType_t::LEMT_BURN {
        let _radius = 8 + (world.bg_state.rng.rand() & 15);
    }

    le.leMarkType = leMarkType_t::LEMT_NONE;
}

/// Raven `CG_FragmentBounceSound` — half of bouncing fragments play a bounce
/// sound; the other roll may still clear `leBounceSoundType` without a sound.
///
/// Raven: bouncers only make the sound once (FIXME: arbitrary...change if it
/// bugs you); a fragment whose type falls through the switch's default (none,
/// blood, brass) returns before clearing `leBounceSoundType`, so it stays
/// eligible to draw again next bounce.
/// Source: `oracle/codemp/cgame/cg_localents.c:162-195`
pub fn CG_FragmentBounceSound(ctx: &mut CgContext, le: &mut localEntity_t, trace: &trace_t) {
    // half the fragments will make a bounce sound
    if ctx.world.bg_state.rng.rand() & 1 != 0 {
        let s: sfxHandle_t = match le.leBounceSoundType {
            leBounceSoundType_t::LEBS_ROCK => {
                let idx = ctx.world.bg_state.rng.Q_irand(0, 1) as usize;
                ctx.world.cgs.media.rockBounceSound[idx]
            }
            // Raven FIXME: make sure that this sound is registered properly...might still be
            // rock bounce sound....
            leBounceSoundType_t::LEBS_METAL => {
                let idx = ctx.world.bg_state.rng.Q_irand(0, 1) as usize;
                ctx.world.cgs.media.metalBounceSound[idx]
            }
            _ => return,
        };

        if s != 0 {
            trap::S_StartSound(
                ctx.engine,
                Some(&trace.endpos),
                ENTITYNUM_WORLD,
                CHAN_AUTO,
                s,
            );
        }

        le.leBounceSoundType = leBounceSoundType_t::LEBS_NONE;
    } else if ctx.world.bg_state.rng.rand() & 1 != 0 {
        // we may end up bouncing again, but each bounce reduces the chance of playing the
        // sound again or they may make a lot of noise when they settle (Raven FIXME: maybe
        // just always do this??)
        le.leBounceSoundType = leBounceSoundType_t::LEBS_NONE;
    }
}

/// Raven `CG_ReflectVelocity` — bounces a fragment's trajectory off the trace
/// plane and decides whether it has come to rest.
///
/// Raven: check for stop, making sure that even on low FPS systems it doesn't
/// bobble. The oracle's `else` arm is empty (`cg_localents.c:224-226`) — no-op,
/// preserved rather than dropped.
/// Source: `oracle/codemp/cgame/cg_localents.c:203-227`
pub fn CG_ReflectVelocity(world: &CgWorld, le: &mut localEntity_t, trace: &trace_t) {
    // reflect the velocity on the trace plane
    let hitTime = ((world.cg.time - world.cg.frametime) as f32
        + world.cg.frametime as f32 * trace.fraction) as c_int;
    let mut velocity: vec3_t = [0.0; 3];
    BG_EvaluateTrajectoryDelta(&le.pos as *const trajectory_t, hitTime, &mut velocity);
    let dot = _DotProduct(velocity, trace.plane.normal);
    _VectorMA(
        velocity,
        -2.0 * dot,
        trace.plane.normal,
        &mut le.pos.trDelta,
    );

    let preScale = le.pos.trDelta;
    _VectorScale(preScale, le.bounceFactor, &mut le.pos.trDelta);

    _VectorCopy(trace.endpos, &mut le.pos.trBase);
    le.pos.trTime = world.cg.time;

    if trace.allsolid != 0
        || (trace.plane.normal[2] > 0.0
            && (le.pos.trDelta[2] < 40.0
                || le.pos.trDelta[2] < -world.cg.frametime as f32 * le.pos.trDelta[2]))
    {
        le.pos.trType = trType_t::TR_STATIONARY;
    }
}

/// Raven `CG_AddFadeRGB` — fades a local entity's refEntity color out linearly
/// over its remaining lifetime and adds it to the scene.
/// Source: `oracle/codemp/cgame/cg_localents.c:348-363`
pub fn CG_AddFadeRGB(ctx: &mut CgContext, le: &mut localEntity_t) {
    let mut c = (le.endTime - ctx.world.cg.time) as f32 * le.lifeRate;
    c *= 0xff as f32;

    // C narrows the float through `int` before landing in the `unsigned char` field (wraps
    // mod 256), not Rust's saturating `as u8` — truncate through `i32` first to match.
    le.refEntity.shaderRGBA[0] = (le.color[0] * c) as i32 as u8;
    le.refEntity.shaderRGBA[1] = (le.color[1] * c) as i32 as u8;
    le.refEntity.shaderRGBA[2] = (le.color[2] * c) as i32 as u8;
    le.refEntity.shaderRGBA[3] = (le.color[3] * c) as i32 as u8;

    trap::R_AddRefEntityToScene(ctx.engine, &le.refEntity);
}

/// Raven `CG_AddFadeScaleModel` — grows a local entity's model from nothing up
/// to `radius` on a cubic ease, fading it out as it grows, and adds it to the
/// scene.
///
/// Raven: yes, this is completely ridiculous...but it causes the shell to grow
/// slowly then "explode" at the end.
/// Source: `oracle/codemp/cgame/cg_localents.c:365-390`
pub fn CG_AddFadeScaleModel(ctx: &mut CgContext, le: &mut localEntity_t) {
    let mut frac = (ctx.world.cg.time - le.startTime) as f32 / (le.endTime - le.startTime) as f32;

    frac *= frac * frac;

    le.refEntity.nonNormalizedAxes = qtrue;

    // DEFERRED: axisDefault — oracle/codemp/game/q_math.c:8; AxisCopy —
    // oracle/codemp/game/q_math.c:550-554. Neither is reachable from mp_cgame's dependency
    // graph yet (axisDefault isn't ported anywhere, and AxisCopy lives in native_math but
    // isn't re-exported through mp_qshared::shared::q_math like its VectorMA/VectorScale
    // siblings). The identity-axis literal `AxisCopy(axisDefault, ...)` copies is inlined
    // here, cited directly from the oracle definition, until the shared symbols land.
    le.refEntity.axis = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    _VectorScale(
        le.refEntity.axis[0],
        le.radius * frac,
        &mut le.refEntity.axis[0],
    );
    _VectorScale(
        le.refEntity.axis[1],
        le.radius * frac,
        &mut le.refEntity.axis[1],
    );
    _VectorScale(
        le.refEntity.axis[2],
        le.radius * 0.5 * frac,
        &mut le.refEntity.axis[2],
    );

    frac = 1.0 - frac;

    // see CG_AddFadeRGB for the truncate-through-i32 note.
    le.refEntity.shaderRGBA[0] = (le.color[0] * frac) as i32 as u8;
    le.refEntity.shaderRGBA[1] = (le.color[1] * frac) as i32 as u8;
    le.refEntity.shaderRGBA[2] = (le.color[2] * frac) as i32 as u8;
    le.refEntity.shaderRGBA[3] = (le.color[3] * frac) as i32 as u8;

    // add the entity
    trap::R_AddRefEntityToScene(ctx.engine, &le.refEntity);
}

/// Raven `CG_AddExplosion` — adds an explosion's model, plus its fade-in/out
/// dlight, to the scene.
/// Source: `oracle/codemp/cgame/cg_localents.c:554-575`
pub fn CG_AddExplosion(ctx: &mut CgContext, ex: &localEntity_t) {
    // add the entity
    trap::R_AddRefEntityToScene(ctx.engine, &ex.refEntity);

    // add the dlight
    if ex.light != 0.0 {
        let mut light =
            (ctx.world.cg.time - ex.startTime) as f32 / (ex.endTime - ex.startTime) as f32;
        if light < 0.5 {
            light = 1.0;
        } else {
            light = 1.0 - (light - 0.5) * 2.0;
        }
        light = ex.light * light;
        trap::R_AddLightToScene(
            ctx.engine,
            &ex.refEntity.origin,
            light,
            ex.lightColor[0],
            ex.lightColor[1],
            ex.lightColor[2],
        );
    }
}

/// Raven `CG_AddSpriteExplosion` — a sprite-quad explosion variant of
/// [`CG_AddExplosion`]: fades a copy of the refEntity to white-out over its
/// life, shrinks its dlight the same way, and adds both to the scene.
///
/// Raven: `c > 1` can happen during connection problems.
/// Source: `oracle/codemp/cgame/cg_localents.c:582-616`
pub fn CG_AddSpriteExplosion(ctx: &mut CgContext, le: &localEntity_t) {
    // Raven copies the whole refEntity into a stack local so the sprite tweaks below never
    // touch `le` itself.
    let mut re = le.refEntity;

    let mut c = (le.endTime - ctx.world.cg.time) as f32 / (le.endTime - le.startTime) as f32;
    if c > 1.0 {
        c = 1.0;
    }

    re.shaderRGBA[0] = 0xff;
    re.shaderRGBA[1] = 0xff;
    re.shaderRGBA[2] = 0xff;
    // see CG_AddFadeRGB for the truncate-through-i32 note.
    re.shaderRGBA[3] = (0xff as f32 * c * 0.33) as i32 as u8;

    re.reType = refEntityType_t::RT_SPRITE;
    re.radius = 42.0 * (1.0 - c) + 30.0;

    trap::R_AddRefEntityToScene(ctx.engine, &re);

    // add the dlight
    if le.light != 0.0 {
        let mut light =
            (ctx.world.cg.time - le.startTime) as f32 / (le.endTime - le.startTime) as f32;
        if light < 0.5 {
            light = 1.0;
        } else {
            light = 1.0 - (light - 0.5) * 2.0;
        }
        light = le.light * light;
        trap::R_AddLightToScene(
            ctx.engine,
            &re.origin,
            light,
            le.lightColor[0],
            le.lightColor[1],
            le.lightColor[2],
        );
    }
}

/// Raven `CG_AddLine` — sets a local entity's refEntity to render as a line and
/// adds it to the scene.
/// Source: `oracle/codemp/cgame/cg_localents.c:770-779`
pub fn CG_AddLine(ctx: &mut CgContext, le: &mut localEntity_t) {
    le.refEntity.reType = refEntityType_t::RT_LINE;

    trap::R_AddRefEntityToScene(ctx.engine, &le.refEntity);
}
