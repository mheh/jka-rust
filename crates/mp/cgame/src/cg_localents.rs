//! Port of `oracle/codemp/cgame/cg_localents.c` — the local-entity effect pool and its per-type add functions. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::f64::consts::PI;
use core::ffi::c_int;

use mp_bg::bg_misc::{BG_EvaluateTrajectory, BG_EvaluateTrajectoryDelta};
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::cgame::tr_types::RF_FORCE_ENT_ALPHA;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::q_math::{
    _DotProduct, _VectorCopy, _VectorMA, _VectorScale, _VectorSubtract, vec3_origin, AnglesToAxis,
    CrossProduct, VectorLength, VectorNormalize,
};
use mp_qshared::shared::surface_flags::{CONTENTS_NODROP, CONTENTS_SOLID};
use mp_qshared::shared::{
    qtrue, sfxHandle_t, trType_t, trajectory_t, vec3_t, CHAN_AUTO, ENTITYNUM_WORLD,
};

use crate::cg_effects::CG_SmokePuff;
use crate::cg_ents::ScaleModelAxis;
use crate::cg_main::CG_Error;
use crate::cg_predict::CG_Trace;
use crate::local::le_bounce_sound_type_t::leBounceSoundType_t;
use crate::local::le_flag_t::leFlag_t;
use crate::local::le_mark_type_t::leMarkType_t;
use crate::local::le_type_t::leType_t;
use crate::local::local_entity_s::localEntity_t;
use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::cg_world::CgWorld;
use crate::world::effect_handle::EffectHandle;

// FILE-SCOPE CONSTANTS
// Source: `oracle/codemp/cgame/cg_localents.c:9,637`

// Raven `#define MAX_LOCAL_ENTITIES 512` already lives on `CgWorld` as
// `world::cg_world::MAX_LOCAL_ENTITIES`, sizing `CgWorld::cg_localEntities`
// (DEC-46.3) — not redeclared here.

/// Raven `#define NUMBER_SIZE 8` — digit-glyph count `CG_AddScorePlum` sizes its
/// per-digit refEntity array with.
/// Source: `oracle/codemp/cgame/cg_localents.c:637`
pub const NUMBER_SIZE: usize = 8;

/// Raven `LEF_PUFF_DONT_SCALE` — `localEntity_t::leFlags` bit meaning "do not
/// scale size over time", tested by the puff/move-scale-fade add fns.
///
/// Lives in the anonymous `leFlags` enum in `cg_local.h`, not in
/// `cg_localents.c` itself, but every add fn below reads it straight out of
/// `leFlags`, so it's pulled in here rather than guessed.
/// Source: `oracle/codemp/cgame/cg_local.h:499`
pub const LEF_PUFF_DONT_SCALE: c_int = 0x0001;

/// Raven `SINK_TIME` — time for fragments to sink into the ground before
/// going away, in ms. Lives in `cg_local.h`, not `cg_localents.c` itself, but
/// `CG_AddFragment` reads it straight out, so it's pulled in here rather than
/// guessed.
/// Source: `oracle/codemp/cgame/cg_local.h:48`
const SINK_TIME: c_int = 1000;

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

/// Raven `CG_FreeLocalEntity` — hands one local entity back to the pool.
///
/// The doubly-linked unlink plus the free-list push IS
/// [`EffectPool::free`](crate::world::effect_pool::EffectPool::free) — the
/// links dissolved into the slab under DEC-46.3, and its `false` return is
/// Raven's `!le->prev` "not active" case.
///
/// Source: `oracle/codemp/cgame/cg_localents.c:39-51`
pub fn CG_FreeLocalEntity(ctx: &mut CgContext, le: EffectHandle) {
    if !ctx.world.cg_localEntities.free(le) {
        CG_Error(ctx, "CG_FreeLocalEntity: not active");
    }
}

/// Raven `CG_AllocLocalEntity` — grabs a free local-entity slot, stealing the
/// oldest active one when the pool is full, and zeroes it.
///
/// DEC-46.3 folds the steal-and-`memset` dance into
/// [`EffectPool::alloc`](crate::world::effect_pool::EffectPool::alloc); this
/// just forwards to it.
/// Source: `oracle/codemp/cgame/cg_localents.c:60-80`
pub fn CG_AllocLocalEntity(world: &mut CgWorld) -> EffectHandle {
    world.cg_localEntities.alloc()
}

/// Raven `CG_AddMoveScaleFade` — grows or fades a local entity toward its
/// trajectory position and adds it to the scene, killing it early if the view
/// origin winds up inside the sprite.
///
/// Takes the pool handle rather than a resolved `localEntity_t` (unlike the
/// earlier-wave add fns in this file) so it can hand the same handle to
/// [`CG_FreeLocalEntity`] on the early-out.
/// Source: `oracle/codemp/cgame/cg_localents.c:397-432`
pub fn CG_AddMoveScaleFade(ctx: &mut CgContext, handle: EffectHandle) {
    let le = ctx
        .world
        .cg_localEntities
        .get_mut(handle)
        .expect("CG_AddMoveScaleFade: not active");

    let c = if le.fadeInTime > le.startTime && ctx.world.cg.time < le.fadeInTime {
        // fade / grow time
        1.0 - (le.fadeInTime - ctx.world.cg.time) as f32 / (le.fadeInTime - le.startTime) as f32
    } else {
        // fade / grow time
        (le.endTime - ctx.world.cg.time) as f32 * le.lifeRate
    };

    // see CG_AddFadeRGB for the truncate-through-i32 note.
    le.refEntity.shaderRGBA[3] = (0xff as f32 * c * le.color[3]) as i32 as u8;

    if (le.leFlags & LEF_PUFF_DONT_SCALE) == 0 {
        le.refEntity.radius = le.radius * (1.0 - c) + 8.0;
    }

    BG_EvaluateTrajectory(
        &le.pos as *const trajectory_t,
        ctx.world.cg.time,
        &mut le.refEntity.origin,
    );

    // if the view would be "inside" the sprite, kill the sprite
    // so it doesn't add too much overdraw
    let mut delta: vec3_t = [0.0; 3];
    _VectorSubtract(le.refEntity.origin, ctx.world.cg.refdef.vieworg, &mut delta);
    let len = VectorLength(delta);
    if len < le.radius {
        CG_FreeLocalEntity(ctx, handle);
        return;
    }

    trap::R_AddRefEntityToScene(ctx.engine, &le.refEntity);
}

/// Raven `CG_AddPuff` — fades a puff local entity's color and radius toward
/// its trajectory position and adds it to the scene, killing it early if the
/// view origin winds up inside the sprite.
///
/// See [`CG_AddMoveScaleFade`] for why this takes the pool handle.
/// Source: `oracle/codemp/cgame/cg_localents.c:439-470`
pub fn CG_AddPuff(ctx: &mut CgContext, handle: EffectHandle) {
    let le = ctx
        .world
        .cg_localEntities
        .get_mut(handle)
        .expect("CG_AddPuff: not active");

    // fade / grow time
    let c = (le.endTime - ctx.world.cg.time) as f32 / (le.endTime - le.startTime) as f32;

    // see CG_AddFadeRGB for the truncate-through-i32 note.
    le.refEntity.shaderRGBA[0] = (le.color[0] * c) as i32 as u8;
    le.refEntity.shaderRGBA[1] = (le.color[1] * c) as i32 as u8;
    le.refEntity.shaderRGBA[2] = (le.color[2] * c) as i32 as u8;

    if (le.leFlags & LEF_PUFF_DONT_SCALE) == 0 {
        le.refEntity.radius = le.radius * (1.0 - c) + 8.0;
    }

    BG_EvaluateTrajectory(
        &le.pos as *const trajectory_t,
        ctx.world.cg.time,
        &mut le.refEntity.origin,
    );

    // if the view would be "inside" the sprite, kill the sprite
    // so it doesn't add too much overdraw
    let mut delta: vec3_t = [0.0; 3];
    _VectorSubtract(le.refEntity.origin, ctx.world.cg.refdef.vieworg, &mut delta);
    let len = VectorLength(delta);
    if len < le.radius {
        CG_FreeLocalEntity(ctx, handle);
        return;
    }

    trap::R_AddRefEntityToScene(ctx.engine, &le.refEntity);
}

/// Raven `CG_AddScaleFade` — fades a local entity's alpha and radius over its
/// remaining lifetime, leaving whatever origin it already carries, and adds
/// it to the scene, killing it early if the view origin winds up inside the
/// sprite.
///
/// See [`CG_AddMoveScaleFade`] for why this takes the pool handle.
/// Source: `oracle/codemp/cgame/cg_localents.c:481-505`
pub fn CG_AddScaleFade(ctx: &mut CgContext, handle: EffectHandle) {
    let le = ctx
        .world
        .cg_localEntities
        .get_mut(handle)
        .expect("CG_AddScaleFade: not active");

    // fade / grow time
    let c = (le.endTime - ctx.world.cg.time) as f32 * le.lifeRate;

    // see CG_AddFadeRGB for the truncate-through-i32 note.
    le.refEntity.shaderRGBA[3] = (0xff as f32 * c * le.color[3]) as i32 as u8;
    le.refEntity.radius = le.radius * (1.0 - c) + 8.0;

    // if the view would be "inside" the sprite, kill the sprite
    // so it doesn't add too much overdraw
    let mut delta: vec3_t = [0.0; 3];
    _VectorSubtract(le.refEntity.origin, ctx.world.cg.refdef.vieworg, &mut delta);
    let len = VectorLength(delta);
    if len < le.radius {
        CG_FreeLocalEntity(ctx, handle);
        return;
    }

    trap::R_AddRefEntityToScene(ctx.engine, &le.refEntity);
}

/// Raven `CG_AddFallScaleFade` — fades a falling local entity's alpha and
/// radius over its remaining lifetime, tracking its Z drop directly off
/// `pos.trBase`/`trDelta` rather than a trajectory eval, and adds it to the
/// scene, killing it early if the view origin winds up inside the sprite.
///
/// See [`CG_AddMoveScaleFade`] for why this takes the pool handle.
/// Source: `oracle/codemp/cgame/cg_localents.c:518-545`
pub fn CG_AddFallScaleFade(ctx: &mut CgContext, handle: EffectHandle) {
    let le = ctx
        .world
        .cg_localEntities
        .get_mut(handle)
        .expect("CG_AddFallScaleFade: not active");

    // fade time
    let c = (le.endTime - ctx.world.cg.time) as f32 * le.lifeRate;

    // see CG_AddFadeRGB for the truncate-through-i32 note.
    le.refEntity.shaderRGBA[3] = (0xff as f32 * c * le.color[3]) as i32 as u8;

    le.refEntity.origin[2] = le.pos.trBase[2] - (1.0 - c) * le.pos.trDelta[2];

    le.refEntity.radius = le.radius * (1.0 - c) + 16.0;

    // if the view would be "inside" the sprite, kill the sprite
    // so it doesn't add too much overdraw
    let mut delta: vec3_t = [0.0; 3];
    _VectorSubtract(le.refEntity.origin, ctx.world.cg.refdef.vieworg, &mut delta);
    let len = VectorLength(delta);
    if len < le.radius {
        CG_FreeLocalEntity(ctx, handle);
        return;
    }

    trap::R_AddRefEntityToScene(ctx.engine, &le.refEntity);
}

/// Raven `CG_AddRefEntity` — adds a local entity's refEntity to the scene
/// as-is, killing it once its lifetime is up.
///
/// See [`CG_AddMoveScaleFade`] for why this takes the pool handle.
/// Source: `oracle/codemp/cgame/cg_localents.c:624-630`
pub fn CG_AddRefEntity(ctx: &mut CgContext, handle: EffectHandle) {
    let le = ctx
        .world
        .cg_localEntities
        .get_mut(handle)
        .expect("CG_AddRefEntity: not active");

    if le.endTime < ctx.world.cg.time {
        CG_FreeLocalEntity(ctx, handle);
        return;
    }

    trap::R_AddRefEntityToScene(ctx.engine, &le.refEntity);
}

/// Raven `CG_AddScorePlum` — draws a floating score number that billboards
/// toward the view, colored by score magnitude/sign, drawing one digit
/// refEntity at a time, killing it early if the view origin winds up inside
/// it.
///
/// See [`CG_AddMoveScaleFade`] for why this takes the pool handle.
/// Source: `oracle/codemp/cgame/cg_localents.c:639-716`
pub fn CG_AddScorePlum(ctx: &mut CgContext, handle: EffectHandle) {
    let le = ctx
        .world
        .cg_localEntities
        .get_mut(handle)
        .expect("CG_AddScorePlum: not active");

    let c = (le.endTime - ctx.world.cg.time) as f32 * le.lifeRate;

    let mut score = le.radius as i32;
    if score < 0 {
        le.refEntity.shaderRGBA[0] = 0xff;
        le.refEntity.shaderRGBA[1] = 0x11;
        le.refEntity.shaderRGBA[2] = 0x11;
    } else {
        le.refEntity.shaderRGBA[0] = 0xff;
        le.refEntity.shaderRGBA[1] = 0xff;
        le.refEntity.shaderRGBA[2] = 0xff;
        if score >= 50 {
            le.refEntity.shaderRGBA[1] = 0;
        } else if score >= 20 {
            le.refEntity.shaderRGBA[0] = 0;
            le.refEntity.shaderRGBA[1] = 0;
        } else if score >= 10 {
            le.refEntity.shaderRGBA[2] = 0;
        } else if score >= 2 {
            le.refEntity.shaderRGBA[0] = 0;
            le.refEntity.shaderRGBA[2] = 0;
        }
    }

    if c < 0.25 {
        // see CG_AddFadeRGB for the truncate-through-i32 note.
        le.refEntity.shaderRGBA[3] = (0xff as f32 * 4.0 * c) as i32 as u8;
    } else {
        le.refEntity.shaderRGBA[3] = 0xff;
    }

    le.refEntity.radius = (NUMBER_SIZE / 2) as f32;

    let mut origin: vec3_t = [0.0; 3];
    _VectorCopy(le.pos.trBase, &mut origin);
    origin[2] += 110.0 - c * 100.0;

    let mut dir: vec3_t = [0.0; 3];
    _VectorSubtract(ctx.world.cg.refdef.vieworg, origin, &mut dir);
    let up: vec3_t = [0.0, 0.0, 1.0];
    let mut vec: vec3_t = [0.0; 3];
    CrossProduct(dir, up, &mut vec);
    VectorNormalize(&mut vec);

    // Raven's `sin` is the double libm call, and `c * 2` computes in float before widening to
    // double for the `* M_PI` (see CG_CalcFOVFromX in cg_view.rs for the same promotion note);
    // the whole `-10 + 20 * sin(...)` stays double until it narrows into VectorMA's float scale.
    let sinArg = (c * 2.0) as f64 * PI;
    let moveScale = (-10.0 + 20.0 * sinArg.sin()) as f32;
    _VectorMA(origin, moveScale, vec, &mut origin);

    // if the view would be "inside" the sprite, kill the sprite
    // so it doesn't add too much overdraw
    let mut delta: vec3_t = [0.0; 3];
    _VectorSubtract(origin, ctx.world.cg.refdef.vieworg, &mut delta);
    let len = VectorLength(delta);
    if len < 20.0 {
        CG_FreeLocalEntity(ctx, handle);
        return;
    }

    let mut negative = false;
    if score < 0 {
        negative = true;
        score = -score;
    }

    let mut digits: [i32; 10] = [0; 10];
    let mut numdigits: usize = 0;
    while !(numdigits != 0 && score == 0) {
        digits[numdigits] = score % 10;
        score /= 10;
        numdigits += 1;
    }

    if negative {
        digits[numdigits] = 10;
        numdigits += 1;
    }

    for i in 0..numdigits {
        let digitScale = (numdigits as f32 / 2.0 - i as f32) * NUMBER_SIZE as f32;
        _VectorMA(origin, digitScale, vec, &mut le.refEntity.origin);
        le.refEntity.customShader =
            ctx.world.cgs.media.numberShaders[digits[numdigits - 1 - i] as usize];
        trap::R_AddRefEntityToScene(ctx.engine, &le.refEntity);
    }
}

/// Raven `CG_AddOLine` — scales an oriented-line local entity's width/alpha
/// over its lifetime and adds it to the scene, killing it early once its
/// width fades to nothing.
///
/// See [`CG_AddMoveScaleFade`] for why this takes the pool handle. The
/// `data.line` reads/writes below match Raven's raw struct access through the
/// same anonymous union on both `localEntity_t` and `refEntity_t` — both
/// variants are POD f32 structs, so it's always defined.
/// Source: `oracle/codemp/cgame/cg_localents.c:725-761`
pub fn CG_AddOLine(ctx: &mut CgContext, handle: EffectHandle) {
    let le = ctx
        .world
        .cg_localEntities
        .get_mut(handle)
        .expect("CG_AddOLine: not active");

    let mut frac = (ctx.world.cg.time - le.startTime) as f32 / (le.endTime - le.startTime) as f32;
    if frac > 1.0 {
        frac = 1.0; // can happen during connection problems
    } else if frac < 0.0 {
        frac = 0.0;
    }

    // Use the liferate to set the scale over time.
    // SAFETY: `data.line` mirrors Raven's raw access through the same
    // anonymous union — every variant is a POD f32 struct, always defined.
    let width = unsafe { le.data.line.width + le.data.line.dwidth * frac };
    le.refEntity.data.line.width = width;
    if width <= 0.0 {
        CG_FreeLocalEntity(ctx, handle);
        return;
    }

    // We will assume here that we want additive transparency effects.
    let alpha = le.alpha + le.dalpha * frac;
    // see CG_AddFadeRGB for the truncate-through-i32 note.
    le.refEntity.shaderRGBA[0] = (0xff as f32 * alpha) as i32 as u8;
    le.refEntity.shaderRGBA[1] = (0xff as f32 * alpha) as i32 as u8;
    le.refEntity.shaderRGBA[2] = (0xff as f32 * alpha) as i32 as u8;
    // Yes, we could apply c to this too, but fading the color is better for lines.
    le.refEntity.shaderRGBA[3] = (0xff as f32 * alpha) as i32 as u8;

    le.refEntity.shaderTexCoord[0] = 1.0;
    le.refEntity.shaderTexCoord[1] = 1.0;

    le.refEntity.rotation = 90.0;

    le.refEntity.reType = refEntityType_t::RT_ORIENTEDLINE;

    trap::R_AddRefEntityToScene(ctx.engine, &le.refEntity);
}

/// Raven `CG_BloodTrail` — walks a fragment's trajectory in fixed 150ms steps
/// over the current frame and drops a smoke puff at each step, turning each
/// puff into a falling blood splat.
///
/// `cgs.media.bloodTrailShader` is commented out in the oracle (never
/// registered) — the literal `0` hShader below matches that as-shipped
/// behavior, not a guess.
/// Source: `oracle/codemp/cgame/cg_localents.c:101-128`
pub fn CG_BloodTrail(world: &mut CgWorld, le: &localEntity_t) {
    let step: c_int = 150;
    let mut t = step * ((world.cg.time - world.cg.frametime + step) / step);
    let t2 = step * (world.cg.time / step);

    while t <= t2 {
        let mut newOrigin: vec3_t = [0.0; 3];
        BG_EvaluateTrajectory(&le.pos as *const trajectory_t, t, &mut newOrigin);

        let handle = CG_SmokePuff(
            world,
            &newOrigin,
            &vec3_origin,
            20.0, // radius
            1.0,
            1.0,
            1.0,
            1.0,    // color
            2000.0, // trailTime
            t,      // startTime
            0,      // fadeInTime
            0,      // flags
            0,      // cgs.media.bloodTrailShader
        );

        let blood = world
            .cg_localEntities
            .get_mut(handle)
            .expect("CG_BloodTrail: fresh slot");
        // use the optimized version
        blood.leType = leType_t::LE_FALL_SCALE_FADE;
        // drop a total of 40 units over its lifetime
        blood.pos.trDelta[2] = 40.0;

        t += step;
    }
}

/// Raven `CG_AddFragment` — the per-frame think for a bouncing gib/fragment:
/// sinks stationary fragments into the ground near removal time, otherwise
/// traces its trajectory and either keeps falling, gets discarded in a nodrop
/// volume, or bounces (leaving a mark, playing a sound, reflecting velocity).
///
/// Takes the pool handle rather than a resolved `localEntity_t` like the
/// other Add fns in this file: several callees below
/// ([`CG_FragmentBounceMark`], [`CG_FragmentBounceSound`],
/// [`CG_ReflectVelocity`], [`CG_BloodTrail`]) take `world`/`ctx` *and*
/// `le: &mut localEntity_t` as separate params, which would fight the pool's
/// mutable borrow — so the entity is taken out of the slab up front
/// (`localEntity_t::zeroed()` left in its place) and put back before every
/// return that doesn't free it.
/// Source: `oracle/codemp/cgame/cg_localents.c:234-332`
pub fn CG_AddFragment(ctx: &mut CgContext, handle: EffectHandle) {
    let mut le = core::mem::replace(
        ctx.world
            .cg_localEntities
            .get_mut(handle)
            .expect("CG_AddFragment: not active"),
        localEntity_t::zeroed(),
    );

    if le.forceAlpha != 0 {
        le.refEntity.renderfx |= RF_FORCE_ENT_ALPHA;
        le.refEntity.shaderRGBA[3] = le.forceAlpha as u8;
    }

    if le.pos.trType == trType_t::TR_STATIONARY {
        // sink into the ground if near the removal time
        let t = le.endTime - ctx.world.cg.time;
        if t < SINK_TIME * 2 {
            le.refEntity.renderfx |= RF_FORCE_ENT_ALPHA;
            // narrows through `int` before landing back in `float t_e` — see CG_AddFadeRGB
            // for the same C truncate-through-i32 shape, mirrored here going the other way.
            let mut t_e = ((le.endTime - ctx.world.cg.time) as f32 / (SINK_TIME * 2) as f32 * 255.0)
                as i32 as f32;

            if t_e > 255.0 {
                t_e = 255.0;
            }
            if t_e < 1.0 {
                t_e = 1.0;
            }

            if le.refEntity.shaderRGBA[3] != 0 && t_e > le.refEntity.shaderRGBA[3] as f32 {
                t_e = le.refEntity.shaderRGBA[3] as f32;
            }

            le.refEntity.shaderRGBA[3] = t_e as i32 as u8;

            trap::R_AddRefEntityToScene(ctx.engine, &le.refEntity);
        } else {
            trap::R_AddRefEntityToScene(ctx.engine, &le.refEntity);
        }

        *ctx.world
            .cg_localEntities
            .get_mut(handle)
            .expect("CG_AddFragment: not active") = le;
        return;
    }

    // calculate new position
    let mut newOrigin: vec3_t = [0.0; 3];
    BG_EvaluateTrajectory(
        &le.pos as *const trajectory_t,
        ctx.world.cg.time,
        &mut newOrigin,
    );

    // trace a line from previous position to new position
    let mut trace = trace_t::zeroed();
    CG_Trace(
        ctx,
        &mut trace,
        &le.refEntity.origin,
        None,
        None,
        &newOrigin,
        -1,
        CONTENTS_SOLID,
    );
    if trace.fraction == 1.0 {
        // still in free fall
        _VectorCopy(newOrigin, &mut le.refEntity.origin);

        if le.leFlags & leFlag_t::LEF_TUMBLE as c_int != 0 {
            let mut angles: vec3_t = [0.0; 3];
            BG_EvaluateTrajectory(
                &le.angles as *const trajectory_t,
                ctx.world.cg.time,
                &mut angles,
            );
            AnglesToAxis(angles, le.refEntity.axis.as_mut_ptr());
            ScaleModelAxis(&mut le.refEntity);
        }

        trap::R_AddRefEntityToScene(ctx.engine, &le.refEntity);

        // add a blood trail
        if le.leBounceSoundType == leBounceSoundType_t::LEBS_BLOOD {
            CG_BloodTrail(ctx.world, &le);
        }

        *ctx.world
            .cg_localEntities
            .get_mut(handle)
            .expect("CG_AddFragment: not active") = le;
        return;
    }

    // if it is in a nodrop zone, remove it
    // this keeps gibs from waiting at the bottom of pits of death
    // and floating levels
    if trap::CM_PointContents(ctx.engine, &trace.endpos, 0) & CONTENTS_NODROP != 0 {
        CG_FreeLocalEntity(ctx, handle);
        return;
    }

    if trace.startsolid == 0 {
        // leave a mark
        CG_FragmentBounceMark(ctx.world, &mut le, &trace);

        // do a bouncy sound
        CG_FragmentBounceSound(ctx, &mut le, &trace);

        if le.bounceSound != 0 {
            // specified bounce sound (debris)
            trap::S_StartSound(
                ctx.engine,
                Some(&le.pos.trBase),
                ENTITYNUM_WORLD,
                CHAN_AUTO,
                le.bounceSound as sfxHandle_t,
            );
        }

        // reflect the velocity on the trace plane
        CG_ReflectVelocity(ctx.world, &mut le, &trace);

        trap::R_AddRefEntityToScene(ctx.engine, &le.refEntity);
    }

    *ctx.world
        .cg_localEntities
        .get_mut(handle)
        .expect("CG_AddFragment: not active") = le;
}

/// Raven `CG_AddLocalEntities` — walks every active local entity, frees the
/// ones whose `endTime` has passed, and dispatches the rest to their
/// per-`leType` add fn.
///
/// Raven walks the intrusive active list backwards (`cg_activeLocalEntities.prev`)
/// so anything a `CG_Add*` spawns this frame (trails, marks, etc) is still
/// present for this same walk — new entries link at the head, so the cursor
/// reaches them before it finishes. The port collects a batch of handles, and
/// after processing re-collects for anything spawned mid-walk, until a pass
/// spawns nothing new (same entities visited, same oldest-first order; frees
/// mid-walk are safe because a stale handle just resolves to `None`).
///
/// `LE_FADE_SCALE_MODEL`/`LE_FADE_RGB`/`LE_LINE`/`LE_EXPLOSION`/
/// `LE_SPRITE_EXPLOSION` take a resolved `localEntity_t` reference rather than
/// a handle (earlier-wave shape) — those five go through the take/put-back
/// dance so the borrowed record doesn't alias the `ctx: &mut CgContext` those
/// fns also need.
/// Source: `oracle/codemp/cgame/cg_localents.c:789-865`
pub fn CG_AddLocalEntities(ctx: &mut CgContext) {
    // grab next now, so if the local entity is freed we still have it
    let mut visited: Vec<EffectHandle> = Vec::new();
    loop {
        let batch: Vec<EffectHandle> = ctx
            .world
            .cg_localEntities
            .active_oldest_first()
            .filter(|h| !visited.contains(h))
            .collect();
        if batch.is_empty() {
            // nothing new spawned mid-walk - Raven's cursor would be done too
            break;
        }
        for handle in batch {
            visited.push(handle);
            add_local_entity(ctx, handle);
        }
    }
}

/// One iteration of [`CG_AddLocalEntities`]'s walk - the body of Raven's
/// `for` loop over the active list, split out so the spawn-during-walk
/// re-collect above stays readable.
/// Source: `oracle/codemp/cgame/cg_localents.c:798-864`
fn add_local_entity(ctx: &mut CgContext, handle: EffectHandle) {
    {
        let le = match ctx.world.cg_localEntities.get(handle) {
            Some(le) => le,
            // already freed earlier this same walk
            None => return,
        };

        if ctx.world.cg.time >= le.endTime {
            CG_FreeLocalEntity(ctx, handle);
            return;
        }

        let leType = le.leType;
        match leType {
            leType_t::LE_MARK => {}

            leType_t::LE_SPRITE_EXPLOSION => {
                let le = core::mem::replace(
                    ctx.world
                        .cg_localEntities
                        .get_mut(handle)
                        .expect("CG_AddLocalEntities: not active"),
                    localEntity_t::zeroed(),
                );
                CG_AddSpriteExplosion(ctx, &le);
                *ctx.world
                    .cg_localEntities
                    .get_mut(handle)
                    .expect("CG_AddLocalEntities: not active") = le;
            }

            leType_t::LE_EXPLOSION => {
                let le = core::mem::replace(
                    ctx.world
                        .cg_localEntities
                        .get_mut(handle)
                        .expect("CG_AddLocalEntities: not active"),
                    localEntity_t::zeroed(),
                );
                CG_AddExplosion(ctx, &le);
                *ctx.world
                    .cg_localEntities
                    .get_mut(handle)
                    .expect("CG_AddLocalEntities: not active") = le;
            }

            leType_t::LE_FADE_SCALE_MODEL => {
                let mut le = core::mem::replace(
                    ctx.world
                        .cg_localEntities
                        .get_mut(handle)
                        .expect("CG_AddLocalEntities: not active"),
                    localEntity_t::zeroed(),
                );
                CG_AddFadeScaleModel(ctx, &mut le);
                *ctx.world
                    .cg_localEntities
                    .get_mut(handle)
                    .expect("CG_AddLocalEntities: not active") = le;
            }

            leType_t::LE_FRAGMENT => CG_AddFragment(ctx, handle), // gibs and brass

            leType_t::LE_PUFF => CG_AddPuff(ctx, handle),

            leType_t::LE_MOVE_SCALE_FADE => CG_AddMoveScaleFade(ctx, handle), // water bubbles

            leType_t::LE_FADE_RGB => {
                // teleporters, railtrails
                let mut le = core::mem::replace(
                    ctx.world
                        .cg_localEntities
                        .get_mut(handle)
                        .expect("CG_AddLocalEntities: not active"),
                    localEntity_t::zeroed(),
                );
                CG_AddFadeRGB(ctx, &mut le);
                *ctx.world
                    .cg_localEntities
                    .get_mut(handle)
                    .expect("CG_AddLocalEntities: not active") = le;
            }

            leType_t::LE_FALL_SCALE_FADE => CG_AddFallScaleFade(ctx, handle), // gib blood trails

            leType_t::LE_SCALE_FADE => CG_AddScaleFade(ctx, handle), // rocket trails

            leType_t::LE_SCOREPLUM => CG_AddScorePlum(ctx, handle),

            leType_t::LE_OLINE => CG_AddOLine(ctx, handle),

            leType_t::LE_SHOWREFENTITY => CG_AddRefEntity(ctx, handle),

            leType_t::LE_LINE => {
                // oriented lines for FX
                let mut le = core::mem::replace(
                    ctx.world
                        .cg_localEntities
                        .get_mut(handle)
                        .expect("CG_AddLocalEntities: not active"),
                    localEntity_t::zeroed(),
                );
                CG_AddLine(ctx, &mut le);
                *ctx.world
                    .cg_localEntities
                    .get_mut(handle)
                    .expect("CG_AddLocalEntities: not active") = le;
            } // no default: Raven's `default` arm (`CG_Error("Bad leType: %i", ...)`) is
              // unreachable here — `leType_t` is an exhaustive Rust enum, so every value
              // that can land in the field is already one of the arms above.
        }
    }
}
