//! Port of `oracle/codemp/cgame/cg_drawtools.c` — low-level 2D drawing helpers (rects, pics, strings). Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_bg::public::stat_index::statIndex_t::{STAT_ARMOR, STAT_HEALTH};
use mp_qshared::shared::q_color::{g_color_table, Q_IsColorString};
use mp_qshared::shared::{
    qhandle_t, vec4_t, BIGCHAR_HEIGHT, BIGCHAR_WIDTH, SMALLCHAR_HEIGHT, SMALLCHAR_WIDTH,
};
use mp_uishared::shared::display_state::DisplayState;
use mp_uishared::shared::menudef::{ITEM_TEXTSTYLE_BLINK, ITEM_TEXTSTYLE_SHADOWED};

use crate::cg_draw::{CG_Text_Paint, CG_Text_Width};
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

/// Raven `#define NUM_FONT_BIG 1` — number-field font style: the default chunky retail HUD digits.
///
/// Source: `oracle/codemp/cgame/cg_local.h:70`
const NUM_FONT_BIG: c_int = 1;

/// Raven `#define NUM_FONT_SMALL 2` — number-field font style: small ammo/health digits.
///
/// Source: `oracle/codemp/cgame/cg_local.h:71`
const NUM_FONT_SMALL: c_int = 2;

/// Raven `#define NUM_FONT_CHUNKY 3` — number-field font style: the wide scoreboard digits.
///
/// Source: `oracle/codemp/cgame/cg_local.h:72`
const NUM_FONT_CHUNKY: c_int = 3;

/// Raven `#define STAT_MINUS 10` — num frame for the '-' stats digit.
///
/// Source: `oracle/codemp/cgame/cg_local.h:59`
const STAT_MINUS: c_int = 10;

// PORT-NOTE: `q_shared.h`'s font enum is anonymous (`enum { FONT_NONE,
// FONT_SMALL=1, ... }`), so per the anonymous-enum convention these are
// `const`s; local, same file-local-copy story `cg_draw.rs` already carries.
/// Source: `oracle/codemp/game/q_shared.h:3176-3182`
const FONT_SMALL: c_int = 1;
/// Source: `oracle/codemp/game/q_shared.h:3176-3182`
const FONT_MEDIUM: c_int = 2;

// Raven `UI_*` text-flag `#define`s (porting-rules §C8): this is their first
// ported consumer, so they land here, file-local like `FONT_SMALL` above.
/// Source: `oracle/codemp/game/q_shared.h:487`
const UI_LEFT: c_int = 0x00000000;
/// Source: `oracle/codemp/game/q_shared.h:488`
const UI_CENTER: c_int = 0x00000001;
/// Source: `oracle/codemp/game/q_shared.h:489`
const UI_RIGHT: c_int = 0x00000002;
/// Source: `oracle/codemp/game/q_shared.h:491`
const UI_SMALLFONT: c_int = 0x00000010;
/// Source: `oracle/codemp/game/q_shared.h:494`
const UI_DROPSHADOW: c_int = 0x00000800;
/// Source: `oracle/codemp/game/q_shared.h:495`
const UI_BLINK: c_int = 0x00001000;
/// Source: `oracle/codemp/game/q_shared.h:497`
const UI_PULSE: c_int = 0x00004000;

/// Raven `ColorIndex` — a `q_shared.h` inline macro (`((c) - '0') & 0x07`),
/// no ported Rust home yet; reproduced file-local per the precedent at
/// `crates/mp/renderer/src/tr_font.rs:2435` / `crates/mp/game/src/g_client.rs:1524`.
///
/// Source: `oracle/codemp/game/q_shared.h:1158`
fn ColorIndex(c: u8) -> c_int {
    (c as c_int - '0' as c_int) & 0x07
}

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

/// Raven `CG_DrawRect` — a hollow rect border built from top/bottom + side bars.
///
/// Source: `oracle/codemp/cgame/cg_drawtools.c:24-31`
pub fn CG_DrawRect(
    ctx: &mut CgContext,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    size: f32,
    color: &vec4_t,
) {
    trap::R_SetColor(ctx.engine, Some(color));

    CG_DrawTopBottom(ctx, x, y, width, height, size);
    CG_DrawSides(ctx, x, y, width, height, size);

    // NULL resets the renderer's tint back to white.
    trap::R_SetColor(ctx.engine, None);
}

/// Raven `CG_TileClear` — repaints the letterbox margins around a scaled-down 3D view with the
/// back-tile shader; a no-op when the refdef already covers the whole screen.
///
/// Source: `oracle/codemp/cgame/cg_drawtools.c:350-378`
pub fn CG_TileClear(ctx: &mut CgContext) {
    let w = ctx.world.cgs.glconfig.vidWidth;
    let h = ctx.world.cgs.glconfig.vidHeight;

    if ctx.world.cg.refdef.x == 0
        && ctx.world.cg.refdef.y == 0
        && ctx.world.cg.refdef.width == w
        && ctx.world.cg.refdef.height == h
    {
        // full screen rendering
        return;
    }

    let top = ctx.world.cg.refdef.y;
    let bottom = top + ctx.world.cg.refdef.height - 1;
    let left = ctx.world.cg.refdef.x;
    let right = left + ctx.world.cg.refdef.width - 1;

    let backTileShader = ctx.world.cgs.media.backTileShader;

    // clear above view screen
    CG_TileClearBox(ctx, 0, 0, w, top, backTileShader);

    // clear below view screen
    CG_TileClearBox(ctx, 0, bottom, w, h - bottom, backTileShader);

    // clear left of view screen
    CG_TileClearBox(ctx, 0, top, left, bottom - top + 1, backTileShader);

    // clear right of view screen
    CG_TileClearBox(ctx, right, top, w - right, bottom - top + 1, backTileShader);
}

/// Raven `CG_ColorForHealth` — armor-folded health color for the live `cg.snap` player state,
/// delegating the green/blue ramp to `CG_ColorForGivenHealth`.
///
/// §F19: Raven dereferences `cg.snap` with no null check here - before the first snapshot that's
/// a null deref. Same hazard `CG_DamageFeedback` (cg_playerstate.c) already takes the neutral
/// early-out for; this returns the same black/opaque color the `health <= 0` branch below uses.
/// Source: `oracle/codemp/cgame/cg_drawtools.c:454-481`
pub fn CG_ColorForHealth(world: &CgWorld) -> vec4_t {
    let Some(snap) = world.cg.snap_ref() else {
        return [0.0, 0.0, 0.0, 1.0];
    };

    // calculate the total points of damage that can
    // be sustained at the current health / armor level
    let mut health = snap.ps.stats[STAT_HEALTH as usize];

    if health <= 0 {
        // black
        return [0.0, 0.0, 0.0, 1.0];
    }

    let mut count = snap.ps.stats[STAT_ARMOR as usize];
    let max = (health as f32 * ARMOR_PROTECTION / (1.0 - ARMOR_PROTECTION)) as c_int;
    if max < count {
        count = max;
    }
    health += count;

    let hcolor: vec4_t = [0.0, 0.0, 0.0, 1.0];
    CG_ColorForGivenHealth(hcolor, health)
}

/// Raven `CG_DrawNumField` — draws a fixed-width numeric field glyph-by-glyph from one of the
/// three number-shader sets (`NUM_FONT_SMALL`/`CHUNKY`/`BIG`), with optional zero-padding.
///
/// Source: `oracle/codemp/cgame/cg_drawtools.c:491-600`
pub fn CG_DrawNumField(
    ctx: &mut CgContext,
    mut x: c_int,
    y: c_int,
    mut width: c_int,
    mut value: c_int,
    charWidth: c_int,
    charHeight: c_int,
    style: c_int,
    zeroFill: bool,
) {
    if width < 1 {
        return;
    }

    // draw number string
    if width > 5 {
        width = 5;
    }

    match width {
        1 => {
            if value > 9 {
                value = 9;
            }
            if value < 0 {
                value = 0;
            }
        }
        2 => {
            if value > 99 {
                value = 99;
            }
            if value < -9 {
                value = -9;
            }
        }
        3 => {
            if value > 999 {
                value = 999;
            }
            if value < -99 {
                value = -99;
            }
        }
        4 => {
            if value > 9999 {
                value = 9999;
            }
            if value < -999 {
                value = -999;
            }
        }
        _ => {}
    }

    let num = format!("{}", value);
    let mut l = num.len() as c_int;
    if l > width {
        l = width;
    }

    // FIXME: Might need to do something different for the chunky font??
    let xWidth = match style {
        NUM_FONT_SMALL => charWidth,
        NUM_FONT_CHUNKY => (charWidth as f32 / 1.2 + 2.0) as c_int,
        // default, and NUM_FONT_BIG
        NUM_FONT_BIG | _ => (charWidth / 2) + 7,
    };

    if zeroFill {
        for _ in 0..(width - l) {
            let shader = match style {
                NUM_FONT_SMALL => ctx.world.cgs.media.smallnumberShaders[0],
                NUM_FONT_CHUNKY => ctx.world.cgs.media.chunkyNumberShaders[0],
                // default, and NUM_FONT_BIG
                NUM_FONT_BIG | _ => ctx.world.cgs.media.numberShaders[0],
            };
            CG_DrawPic(
                ctx,
                x as f32,
                y as f32,
                charWidth as f32,
                charHeight as f32,
                shader,
            );
            x += 2 + xWidth;
        }
    } else {
        x += 2 + xWidth * (width - l);
    }

    let mut chars = num.chars();
    while l > 0 {
        let Some(ch) = chars.next() else {
            break;
        };

        let frame = if ch == '-' {
            STAT_MINUS
        } else {
            ch as c_int - '0' as c_int
        };

        let shader = match style {
            NUM_FONT_SMALL => ctx.world.cgs.media.smallnumberShaders[frame as usize],
            NUM_FONT_CHUNKY => ctx.world.cgs.media.chunkyNumberShaders[frame as usize],
            // default, and NUM_FONT_BIG
            NUM_FONT_BIG | _ => ctx.world.cgs.media.numberShaders[frame as usize],
        };
        CG_DrawPic(
            ctx,
            x as f32,
            y as f32,
            charWidth as f32,
            charHeight as f32,
            shader,
        );
        if style == NUM_FONT_SMALL {
            // For a one line gap
            x += 1;
        }

        x += xWidth;
        l -= 1;
    }
}

/// Raven `CG_DrawStringExt` — the console-font string painter: drop shadow
/// pass then a colored pass, switching per glyph on embedded `^N` color codes.
///
/// PORT-NOTE: `maxChars` is a dead parameter - Raven's body never reads it (no
/// truncation happens here despite the name), preserved as an unused param.
/// Source: `oracle/codemp/cgame/cg_drawtools.c:212-274`
#[allow(clippy::too_many_arguments)]
pub fn CG_DrawStringExt(
    ctx: &mut CgContext,
    ds: &DisplayState,
    x: c_int,
    y: c_int,
    string: &str,
    setColor: &vec4_t,
    forceColor: bool,
    shadow: bool,
    charWidth: c_int,
    charHeight: c_int,
    _maxChars: c_int,
) {
    if trap::Language_IsAsian(ctx.engine) {
        // hack-a-doodle-do (post-release quick fix code)...
        let color: vec4_t = *setColor; // de-const it
        CG_Text_Paint(
            ctx,
            ds,
            x as f32,
            y as f32,
            1.0,
            color,
            string,
            0.0,
            0,
            if shadow { ITEM_TEXTSTYLE_SHADOWED } else { 0 },
            FONT_MEDIUM,
        );
    } else {
        // draw the drop shadow
        if shadow {
            let color: vec4_t = [0.0, 0.0, 0.0, setColor[3]];
            trap::R_SetColor(ctx.engine, Some(&color));
            let mut xx = x;
            let mut chars = string.chars().peekable();
            while let Some(c) = chars.next() {
                let pair = [c as u32 as u8, chars.peek().map_or(0, |&n| n as u32 as u8)];
                if Q_IsColorString(&pair) {
                    chars.next();
                    continue;
                }
                CG_DrawChar(ctx, xx + 2, y + 2, charWidth, charHeight, pair[0] as c_int);
                xx += charWidth;
            }
        }

        // draw the colored text
        let mut xx = x;
        trap::R_SetColor(ctx.engine, Some(setColor));
        let mut chars = string.chars().peekable();
        while let Some(c) = chars.next() {
            let pair = [c as u32 as u8, chars.peek().map_or(0, |&n| n as u32 as u8)];
            if Q_IsColorString(&pair) {
                if !forceColor {
                    let mut color = g_color_table[ColorIndex(pair[1]) as usize];
                    color[3] = setColor[3];
                    trap::R_SetColor(ctx.engine, Some(&color));
                }
                chars.next();
                continue;
            }
            CG_DrawChar(ctx, xx, y, charWidth, charHeight, pair[0] as c_int);
            xx += charWidth;
        }
        trap::R_SetColor(ctx.engine, None);
    }
}

/// Raven `UI_DrawProportionalString` — the shared/UI proportional-font string
/// painter: left/center/right justify, then style flags folded to
/// `CG_Text_Paint`'s `ITEM_TEXTSTYLE_*` set.
///
/// Raven comment: "having all these different style defines (1 for UI, one
/// for CG, and now one for the re->font stuff) is dumb, but for now..."
/// Source: `oracle/codemp/cgame/cg_drawtools.c:603-644`
pub fn UI_DrawProportionalString(
    ctx: &CgContext,
    ds: &DisplayState,
    x: c_int,
    y: c_int,
    str: &str,
    style: c_int,
    color: vec4_t,
) {
    let iMenuFont = if style & UI_SMALLFONT != 0 {
        FONT_SMALL
    } else {
        FONT_MEDIUM
    };

    let mut x = x;
    match style & (UI_LEFT | UI_CENTER | UI_RIGHT) {
        UI_CENTER => {
            x -= CG_Text_Width(ctx, ds, str, 1.0, iMenuFont) / 2;
        }
        UI_RIGHT => {
            x -= CG_Text_Width(ctx, ds, str, 1.0, iMenuFont) / 2;
        }
        // default, and UI_LEFT
        _ => {}
    }

    let mut iStyle: c_int = 0;
    if style & UI_DROPSHADOW != 0 {
        iStyle = ITEM_TEXTSTYLE_SHADOWED;
    } else if style & (UI_BLINK | UI_PULSE) != 0 {
        iStyle = ITEM_TEXTSTYLE_BLINK;
    }

    CG_Text_Paint(
        ctx, ds, x as f32, y as f32, 1.0, color, str, 0.0, 0, iStyle, iMenuFont,
    );
}

/// Raven `UI_DrawScaledProportionalString` — `UI_DrawProportionalString` with
/// an explicit scale and always `FONT_MEDIUM` (no `UI_SMALLFONT` read here).
///
/// Source: `oracle/codemp/cgame/cg_drawtools.c:646-686`
pub fn UI_DrawScaledProportionalString(
    ctx: &CgContext,
    ds: &DisplayState,
    x: c_int,
    y: c_int,
    str: &str,
    style: c_int,
    color: vec4_t,
    scale: f32,
) {
    let mut x = x;
    match style & (UI_LEFT | UI_CENTER | UI_RIGHT) {
        UI_CENTER => {
            x -= CG_Text_Width(ctx, ds, str, scale, FONT_MEDIUM) / 2;
        }
        UI_RIGHT => {
            x -= CG_Text_Width(ctx, ds, str, scale, FONT_MEDIUM) / 2;
        }
        // default, and UI_LEFT
        _ => {}
    }

    let mut iStyle: c_int = 0;
    if style & UI_DROPSHADOW != 0 {
        iStyle = ITEM_TEXTSTYLE_SHADOWED;
    } else if style & (UI_BLINK | UI_PULSE) != 0 {
        iStyle = ITEM_TEXTSTYLE_BLINK;
    }

    CG_Text_Paint(
        ctx,
        ds,
        x as f32,
        y as f32,
        scale,
        color,
        str,
        0.0,
        0,
        iStyle,
        FONT_MEDIUM,
    );
}

/// Raven `CG_DrawBigString` — the big console font, white with a caller-given alpha.
///
/// Source: `oracle/codemp/cgame/cg_drawtools.c:276-282`
pub fn CG_DrawBigString(
    ctx: &mut CgContext,
    ds: &DisplayState,
    x: c_int,
    y: c_int,
    s: &str,
    alpha: f32,
) {
    let mut color: vec4_t = [0.0, 0.0, 0.0, 0.0];
    color[0] = 1.0;
    color[1] = 1.0;
    color[2] = 1.0;
    color[3] = alpha;
    CG_DrawStringExt(
        ctx,
        ds,
        x,
        y,
        s,
        &color,
        false,
        true,
        BIGCHAR_WIDTH,
        BIGCHAR_HEIGHT,
        0,
    );
}

/// Raven `CG_DrawBigStringColor` — the big console font, caller-supplied color.
///
/// Source: `oracle/codemp/cgame/cg_drawtools.c:284-286`
pub fn CG_DrawBigStringColor(
    ctx: &mut CgContext,
    ds: &DisplayState,
    x: c_int,
    y: c_int,
    s: &str,
    color: &vec4_t,
) {
    CG_DrawStringExt(
        ctx,
        ds,
        x,
        y,
        s,
        color,
        true,
        true,
        BIGCHAR_WIDTH,
        BIGCHAR_HEIGHT,
        0,
    );
}

/// Raven `CG_DrawSmallString` — the small console font, white with a caller-given alpha.
///
/// Source: `oracle/codemp/cgame/cg_drawtools.c:288-294`
pub fn CG_DrawSmallString(
    ctx: &mut CgContext,
    ds: &DisplayState,
    x: c_int,
    y: c_int,
    s: &str,
    alpha: f32,
) {
    let mut color: vec4_t = [0.0, 0.0, 0.0, 0.0];
    color[0] = 1.0;
    color[1] = 1.0;
    color[2] = 1.0;
    color[3] = alpha;
    CG_DrawStringExt(
        ctx,
        ds,
        x,
        y,
        s,
        &color,
        false,
        false,
        SMALLCHAR_WIDTH,
        SMALLCHAR_HEIGHT,
        0,
    );
}

/// Raven `CG_DrawSmallStringColor` — the small console font, caller-supplied color.
///
/// Source: `oracle/codemp/cgame/cg_drawtools.c:296-298`
pub fn CG_DrawSmallStringColor(
    ctx: &mut CgContext,
    ds: &DisplayState,
    x: c_int,
    y: c_int,
    s: &str,
    color: &vec4_t,
) {
    CG_DrawStringExt(
        ctx,
        ds,
        x,
        y,
        s,
        color,
        true,
        false,
        SMALLCHAR_WIDTH,
        SMALLCHAR_HEIGHT,
        0,
    );
}
