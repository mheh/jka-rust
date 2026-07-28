//! Port of `oracle/codemp/cgame/fx_disruptor.c` — disruptor beam, sniper trail and impact effects. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_qshared::shared::q_math::{_VectorAdd, _VectorCopy, _VectorMA, vec3_origin};
use mp_qshared::shared::{addbezierArgStruct_t, vec3_t};

use crate::trap;
use crate::world::cg_context::CgContext;

// FILE-SCOPE CONSTANTS
// Source: `oracle/codemp/cgame/fx_disruptor.c:11,68`

/// Raven `WHITE` — flat white color used by both disruptor beam layers and the
/// alt-fire miss trail.
/// Source: `oracle/codemp/cgame/fx_disruptor.c:11`
const WHITE: vec3_t = [1.0, 1.0, 1.0];

/// Raven `FX_ALPHA_WAVE` — `addbezierArgStruct_t.flags` bit: modulate alpha on
/// a wave/pulse rather than linearly.
/// Source: `oracle/codemp/cgame/fx_disruptor.c:68`
const FX_ALPHA_WAVE: c_int = 0x00000008;

// PORT-NOTE: `FX_ALPHA_LINEAR`/`FX_SIZE_LINEAR` are `fx_local.h` flags this TU
// includes but doesn't define itself, so they're absent from the packet's
// file-scope constants list; the packet's own oracle slice for
// `FX_DisruptorMainShot`/`FX_DisruptorAltShot` uses them by name, so they're
// transcribed here rather than deferred (mirrors `fx_bryarpistol.rs`).
/// Raven `FX_ALPHA_LINEAR` — `trap_FX_AddLine` flag: interpolate alpha linearly
/// between `alpha1`/`alpha2` over the line's lifetime.
/// Source: `oracle/codemp/cgame/fx_local.h:6`
const FX_ALPHA_LINEAR: c_int = 0x00000001;

/// Raven `FX_SIZE_LINEAR` — `trap_FX_AddLine` flag: interpolate width linearly
/// between `size1`/`size2` over the line's lifetime.
/// Source: `oracle/codemp/cgame/fx_local.h:7`
const FX_SIZE_LINEAR: c_int = 0x00000100;

/// Raven `FX_DisruptorMainShot` — draws the primary-fire beam as a single red
/// line; the cylinder spiral layer underneath is dead code, commented out in
/// the oracle, and stays commented out here.
/// Source: `oracle/codemp/cgame/fx_disruptor.c:13-33`
pub fn FX_DisruptorMainShot(ctx: &mut CgContext, start: &vec3_t, end: &vec3_t) {
    // vec3_t	dir;
    // float	len;

    let shader = trap::R_RegisterShader(ctx.engine, "gfx/effects/redLine");
    trap::FX_AddLine(
        ctx.engine,
        start,
        end,
        0.1,
        6.0,
        0.0,
        1.0,
        0.0,
        0.0,
        &WHITE,
        &WHITE,
        0.0,
        150,
        shader,
        FX_SIZE_LINEAR | FX_ALPHA_LINEAR,
    );

    // VectorSubtract( end, start, dir );
    // len = VectorNormalize( dir );

    // FX_AddCylinder( start, dir, 5.0f, 5.0f, 0.0f,
    // 								5.0f, 5.0f, 0.0f,
    // 								len, len, 0.0f,
    // 								1.0f, 1.0f, 0.0f,
    // 								WHITE, WHITE, 0.0f,
    // 								400, cgi_R_RegisterShader( "gfx/effects/spiral" ), 0 );
}

/// Raven `FX_DisruptorAltShot` — draws the alt-fire beam as a red line, adding
/// a second thicker yellow-tinted layer underneath when the shot was a full
/// charge.
/// Source: `oracle/codemp/cgame/fx_disruptor.c:41-60`
pub fn FX_DisruptorAltShot(ctx: &mut CgContext, start: &vec3_t, end: &vec3_t, fullCharge: bool) {
    let shader = trap::R_RegisterShader(ctx.engine, "gfx/effects/redLine");
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
        shader,
        FX_SIZE_LINEAR | FX_ALPHA_LINEAR,
    );

    if fullCharge {
        let yeller: vec3_t = [0.8, 0.7, 0.0];

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
            &yeller,
            &yeller,
            0.0,
            150,
            beef_shader,
            FX_SIZE_LINEAR | FX_ALPHA_LINEAR,
        );
    }
}

/// Raven `FX_DisruptorAltMiss` — draws the alt-fire miss trail as a bezier arc
/// climbing from the miss point, then plays the miss-whiff effect. The direct
/// `FX_AddBezier` call is commented out in the oracle in favor of filling an
/// `addbezierArgStruct_t` and going through the VM boundary (`trap_FX_AddBezier`);
/// both stay here for fidelity.
/// Source: `oracle/codemp/cgame/fx_disruptor.c:70-113`
pub fn FX_DisruptorAltMiss(ctx: &mut CgContext, origin: &vec3_t, normal: &vec3_t) {
    let mut pos: vec3_t = [0.0; 3];
    let mut c1: vec3_t = [0.0; 3];
    let mut c2: vec3_t = [0.0; 3];

    _VectorMA(*origin, 4.0, *normal, &mut c1);
    _VectorCopy(c1, &mut c2);
    c1[2] += 4.0;
    c2[2] += 12.0;

    _VectorAdd(*origin, *normal, &mut pos);
    pos[2] += 28.0;

    // FX_AddBezier( origin, pos, c1, vec3_origin, c2, vec3_origin, 6.0f, 6.0f, 0.0f, 0.0f, 0.2f, 0.5f,
    // WHITE, WHITE, 0.0f, 4000, trap_R_RegisterShader( "gfx/effects/smokeTrail" ), FX_ALPHA_WAVE );

    let mut b = addbezierArgStruct_t {
        start: *origin,
        end: pos,
        control1: c1,
        control1Vel: vec3_origin,
        control2: c2,
        control2Vel: vec3_origin,
        size1: 6.0,
        size2: 6.0,
        sizeParm: 0.0,
        alpha1: 0.0,
        alpha2: 0.2,
        alphaParm: 0.5,
        sRGB: WHITE,
        eRGB: WHITE,
        rgbParm: 0.0,
        killTime: 4000,
        shader: trap::R_RegisterShader(ctx.engine, "gfx/effects/smokeTrail"),
        flags: FX_ALPHA_WAVE,
    };

    trap::FX_AddBezier(ctx.engine, &mut b);

    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.disruptorAltMissEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/// Raven `FX_DisruptorAltHit` — plays the disruptor alt-fire hit effect.
/// Source: `oracle/codemp/cgame/fx_disruptor.c:121-124`
pub fn FX_DisruptorAltHit(ctx: &mut CgContext, origin: &vec3_t, normal: &vec3_t) {
    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.disruptorAltHitEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/// Raven `FX_DisruptorHitWall` — plays the disruptor wall-impact effect.
/// Source: `oracle/codemp/cgame/fx_disruptor.c:134-137`
pub fn FX_DisruptorHitWall(ctx: &mut CgContext, origin: &vec3_t, normal: &vec3_t) {
    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.disruptorWallImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/// Raven `FX_DisruptorHitPlayer` — plays the disruptor flesh-impact effect;
/// `humanoid` goes unread in Raven's body, kept for signature parity.
/// Source: `oracle/codemp/cgame/fx_disruptor.c:145-148`
pub fn FX_DisruptorHitPlayer(
    ctx: &mut CgContext,
    origin: &vec3_t,
    normal: &vec3_t,
    _humanoid: bool,
) {
    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.disruptorFleshImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}
