//! Port of `oracle/codemp/cgame/cg_drawtools.c` — low-level 2D drawing helpers (rects, pics, strings). Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_qshared::shared::q_color::Q_IsColorString;
use mp_qshared::shared::{qhandle_t, vec4_t};

use crate::trap;
use crate::world::{CgContext, CgWorld};

/// Raven `#define ARMOR_PROTECTION 0.50` — shields absorb 50% of the damage before it reaches health.
///
/// Source: `oracle/codemp/game/bg_public.h:26`
const ARMOR_PROTECTION: f32 = 0.50;

/// Raven `#define FADE_TIME 200` — the fade-out tail, in milliseconds, of a HUD element's lifetime.
///
/// Source: `oracle/codemp/cgame/cg_local.h:26`
const FADE_TIME: c_int = 200;

/// Raven `CG_GetColorForHealth` — health bar color ramp, armor-adjusted.
///
/// Armor is folded into the effective health (up to the amount `ARMOR_PROTECTION` would let it
/// absorb) before the ramp runs, so a fully-armored player reads green even at low health.
/// Source: `oracle/codemp/cgame/cg_drawtools.c:40-76`
pub fn CG_GetColorForHealth(health: c_int, armor: c_int) -> vec4_t {
    // calculate the total points of damage that can
    // be sustained at the current health / armor level
    if health <= 0 {
        // black
        return [0.0, 0.0, 0.0, 1.0];
    }

    let mut count = armor;
    let max = (health as f32 * ARMOR_PROTECTION / (1.0 - ARMOR_PROTECTION)) as c_int;
    if max < count {
        count = max;
    }
    let health = health + count;

    // set the color based on health
    let mut hcolor: vec4_t = [0.0, 0.0, 0.0, 0.0];
    hcolor[0] = 1.0;
    hcolor[3] = 1.0;
    if health >= 100 {
        hcolor[2] = 1.0;
    } else if health < 66 {
        hcolor[2] = 0.0;
    } else {
        hcolor[2] = (health - 66) as f32 / 33.0;
    }

    if health > 60 {
        hcolor[1] = 1.0;
    } else if health < 30 {
        hcolor[1] = 0.0;
    } else {
        hcolor[1] = (health - 30) as f32 / 30.0;
    }

    hcolor
}

/// Raven `CG_DrawSides` — the left/right border bars of a scaled screen element.
///
/// Source: `oracle/codemp/cgame/cg_drawtools.c:85-89`
pub fn CG_DrawSides(ctx: &mut CgContext, x: f32, y: f32, w: f32, h: f32, size: f32) {
    let size = size * ctx.world.cgs.screenXScale;
    let white_shader = ctx.world.cgs.media.whiteShader;
    trap::R_DrawStretchPic(ctx.engine, x, y, size, h, 0.0, 0.0, 0.0, 0.0, white_shader);
    trap::R_DrawStretchPic(
        ctx.engine,
        x + w - size,
        y,
        size,
        h,
        0.0,
        0.0,
        0.0,
        0.0,
        white_shader,
    );
}

/// Raven `CG_DrawTopBottom` — the top/bottom border bars of a scaled screen element.
///
/// Source: `oracle/codemp/cgame/cg_drawtools.c:91-95`
pub fn CG_DrawTopBottom(ctx: &mut CgContext, x: f32, y: f32, w: f32, h: f32, size: f32) {
    let size = size * ctx.world.cgs.screenYScale;
    let white_shader = ctx.world.cgs.media.whiteShader;
    trap::R_DrawStretchPic(ctx.engine, x, y, w, size, 0.0, 0.0, 0.0, 0.0, white_shader);
    trap::R_DrawStretchPic(
        ctx.engine,
        x,
        y + h - size,
        w,
        size,
        0.0,
        0.0,
        0.0,
        0.0,
        white_shader,
    );
}

/// Raven `CG_FillRect2` — solid-color rect using the white shader, tinted via `trap_R_SetColor`.
///
/// Source: `oracle/codemp/cgame/cg_drawtools.c:103-107`
pub fn CG_FillRect2(ctx: &mut CgContext, x: f32, y: f32, width: f32, height: f32, color: &vec4_t) {
    trap::R_SetColor(ctx.engine, Some(color));
    let white_shader = ctx.world.cgs.media.whiteShader;
    trap::R_DrawStretchPic(
        ctx.engine,
        x,
        y,
        width,
        height,
        0.0,
        0.0,
        0.0,
        0.0,
        white_shader,
    );
    // NULL resets the renderer's tint back to white.
    trap::R_SetColor(ctx.engine, None);
}

/// Raven `CG_FillRect` — same as `CG_FillRect2`, Raven kept both under separate names.
///
/// Source: `oracle/codemp/cgame/cg_drawtools.c:116-122`
pub fn CG_FillRect(ctx: &mut CgContext, x: f32, y: f32, width: f32, height: f32, color: &vec4_t) {
    trap::R_SetColor(ctx.engine, Some(color));

    let white_shader = ctx.world.cgs.media.whiteShader;
    trap::R_DrawStretchPic(
        ctx.engine,
        x,
        y,
        width,
        height,
        0.0,
        0.0,
        0.0,
        0.0,
        white_shader,
    );

    // NULL resets the renderer's tint back to white.
    trap::R_SetColor(ctx.engine, None);
}

/// Raven `CG_DrawPic` — draws a shader across the full given rect, unrotated.
///
/// Source: `oracle/codemp/cgame/cg_drawtools.c:133-135`
pub fn CG_DrawPic(
    ctx: &mut CgContext,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    hShader: qhandle_t,
) {
    trap::R_DrawStretchPic(ctx.engine, x, y, width, height, 0.0, 0.0, 1.0, 1.0, hShader);
}

/// Raven `CG_DrawRotatePic` — `CG_DrawPic`, rotated about its origin.
///
/// Source: `oracle/codemp/cgame/cg_drawtools.c:146-148`
pub fn CG_DrawRotatePic(
    ctx: &mut CgContext,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    angle: f32,
    hShader: qhandle_t,
) {
    trap::R_DrawRotatePic(
        ctx.engine, x, y, width, height, 0.0, 0.0, 1.0, 1.0, angle, hShader,
    );
}

/// Raven `CG_DrawRotatePic2` — `CG_DrawRotatePic` against the renderer's second rotate-pic path.
///
/// Source: `oracle/codemp/cgame/cg_drawtools.c:159-161`
pub fn CG_DrawRotatePic2(
    ctx: &mut CgContext,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    angle: f32,
    hShader: qhandle_t,
) {
    trap::R_DrawRotatePic2(
        ctx.engine, x, y, width, height, 0.0, 0.0, 1.0, 1.0, angle, hShader,
    );
}

/// Raven `CG_DrawChar` — draws one glyph from the charset shader's 16x16 cell grid.
///
/// PORT-NOTE: the u-span (`size` = 0.03125) and v-span (`size2` = 0.0625) of the texcoord rect
/// differ - Raven quirk in the charset grid math, preserved as-is.
/// Source: `oracle/codemp/cgame/cg_drawtools.c:170-199`
pub fn CG_DrawChar(
    ctx: &mut CgContext,
    x: c_int,
    y: c_int,
    width: c_int,
    height: c_int,
    ch: c_int,
) {
    let ch = ch & 255;

    if ch == b' ' as c_int {
        return;
    }

    let ax = x as f32;
    let ay = y as f32;
    let aw = width as f32;
    let ah = height as f32;

    let row = ch >> 4;
    let col = ch & 15;

    let frow = row as f32 * 0.0625;
    let fcol = col as f32 * 0.0625;
    let size = 0.03125;
    let size2 = 0.0625;

    let charset_shader = ctx.world.cgs.media.charsetShader;
    trap::R_DrawStretchPic(
        ctx.engine,
        ax,
        ay,
        aw,
        ah,
        fcol,
        frow,
        fcol + size,
        frow + size2,
        charset_shader,
    );
}

/// Raven `CG_DrawStrlen` — printable-glyph count, skipping `^N` color codes.
///
/// `str` is already-decoded console text (native_string's Latin-1 lane): each `char` is one
/// Raven byte, so pairing adjacent chars and handing them to `Q_IsColorString` matches Raven's
/// raw-byte walk exactly.
/// Source: `oracle/codemp/cgame/cg_drawtools.c:307-321`
pub fn CG_DrawStrlen(str: &str) -> c_int {
    let mut count: c_int = 0;
    let mut chars = str.chars().peekable();
    while let Some(c) = chars.next() {
        let pair = [c as u32 as u8, chars.peek().map_or(0, |&n| n as u32 as u8)];
        if Q_IsColorString(&pair) {
            chars.next();
        } else {
            count += 1;
        }
    }
    count
}

/// Raven `CG_TileClearBox` — tiles a shader across a rect at a fixed 64-unit texel scale.
///
/// Source: `oracle/codemp/cgame/cg_drawtools.c:331-339`
pub fn CG_TileClearBox(
    ctx: &mut CgContext,
    x: c_int,
    y: c_int,
    w: c_int,
    h: c_int,
    hShader: qhandle_t,
) {
    let s1 = x as f32 / 64.0;
    let t1 = y as f32 / 64.0;
    let s2 = (x + w) as f32 / 64.0;
    let t2 = (y + h) as f32 / 64.0;
    trap::R_DrawStretchPic(
        ctx.engine, x as f32, y as f32, w as f32, h as f32, s1, t1, s2, t2, hShader,
    );
}

/// Raven `CG_FadeColor` — fade-out alpha ramp for a HUD element aging past `totalMsec`.
///
/// Raven backs the return with a file-scope `static vec4_t`; every non-null return fully
/// overwrites all four components (nothing survives from the previous call), so there's no real
/// cross-call state to fold - this returns by value instead of reaching for a `CgWorld` field.
/// Source: `oracle/codemp/cgame/cg_drawtools.c:387-410`
pub fn CG_FadeColor(world: &CgWorld, startMsec: c_int, totalMsec: c_int) -> Option<vec4_t> {
    if startMsec == 0 {
        return None;
    }

    let t = world.cg.time - startMsec;

    if t >= totalMsec {
        return None;
    }

    // fade out
    let alpha = if totalMsec - t < FADE_TIME {
        (totalMsec - t) as f32 * 1.0 / FADE_TIME as f32
    } else {
        1.0
    };

    Some([1.0, 1.0, 1.0, alpha])
}

/// Raven `CG_ColorForGivenHealth` — health-based color ramp on the green/blue channels only.
///
/// Raven never touches `hcolor[3]` (alpha) here - the caller's buffer keeps whatever alpha it
/// already had. Ported as an in/out `vec4_t` so that quirk carries over exactly.
/// Source: `oracle/codemp/cgame/cg_drawtools.c:418-447`
pub fn CG_ColorForGivenHealth(mut hcolor: vec4_t, health: c_int) -> vec4_t {
    // set the color based on health
    hcolor[0] = 1.0;
    if health >= 100 {
        hcolor[2] = 1.0;
    } else if health < 66 {
        hcolor[2] = 0.0;
    } else {
        hcolor[2] = (health - 66) as f32 / 33.0;
    }

    if health > 60 {
        hcolor[1] = 1.0;
    } else if health < 30 {
        hcolor[1] = 0.0;
    } else {
        hcolor[1] = (health - 30) as f32 / 30.0;
    }

    hcolor
}
