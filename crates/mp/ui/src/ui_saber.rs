//! `ui_saber.c` — the saber-selection screen.
//!
//! Source: `oracle/codemp/ui/ui_saber.c`

#![allow(non_snake_case)]

use mp_bg::bg_channel::BgState;
use mp_qshared::common::mp::cgame::ref_entity_t::{
    refEntity_t, refEntity_t_data, refEntity_t_sprite, refEntity_t_uMini, refEntity_t_uRefEnt,
};
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::qcommon::saber::saber_colors::{
    saber_colors_t, SABER_BLUE, SABER_GREEN, SABER_ORANGE, SABER_PURPLE, SABER_RED, SABER_YELLOW,
};
use mp_qshared::common::mp::qcommon::saber::saber_type::saberType_t;
use mp_qshared::shared::com_parse::{COM_ParseExt, QSharedScratch};
use mp_qshared::shared::q_math::{_VectorMA, VectorSet};
use mp_qshared::shared::vec3_t;
use native_types::qfalse;

use crate::trap;
use crate::world::ui_context::UiContext;

/// Raven `UI_CacheSaberGlowGraphics` — registers the twelve saber blade
/// glow/core shaders (two per color) into `UiWorld.saber`.
///
/// Raven: `//FIXME: these get fucked by vid_restarts`.
///
/// Source: `oracle/codemp/ui/ui_saber.c:38-52`
pub fn UI_CacheSaberGlowGraphics(ctx: &mut UiContext) {
    ctx.world.saber.redSaberGlowShader =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/effects/sabers/red_glow");
    ctx.world.saber.redSaberCoreShader =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/effects/sabers/red_line");
    ctx.world.saber.orangeSaberGlowShader =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/effects/sabers/orange_glow");
    ctx.world.saber.orangeSaberCoreShader =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/effects/sabers/orange_line");
    ctx.world.saber.yellowSaberGlowShader =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/effects/sabers/yellow_glow");
    ctx.world.saber.yellowSaberCoreShader =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/effects/sabers/yellow_line");
    ctx.world.saber.greenSaberGlowShader =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/effects/sabers/green_glow");
    ctx.world.saber.greenSaberCoreShader =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/effects/sabers/green_line");
    ctx.world.saber.blueSaberGlowShader =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/effects/sabers/blue_glow");
    ctx.world.saber.blueSaberCoreShader =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/effects/sabers/blue_line");
    ctx.world.saber.purpleSaberGlowShader =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/effects/sabers/purple_glow");
    ctx.world.saber.purpleSaberCoreShader =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/effects/sabers/purple_line");
}

/// Raven `UI_ParseLiteralSilent` — consumes one token and reports whether it
/// was NOT the expected literal (`qtrue` on EOF or a mismatch, `qfalse` when
/// the token matched `string`).
///
/// PORT-NOTE: `data` is Raven's `const char **` parse cursor — kept as the
/// same in/out `Option<&[u8]>` cursor shape `COM_ParseExt` already uses,
/// rather than a returned out-param, since it is genuinely read-modify-write.
/// `Q_stricmp(token, string) != 0` becomes `!token.eq_ignore_ascii_case(string)`
/// (dictionary: case-insensitive C string compare -> `eq_ignore_ascii_case`).
/// The `COM_ParseExt` scratch (`com_lines`/error-name buffers) is genuinely
/// function-local here — a fresh `QSharedScratch` per call, since this fn
/// carries no persistent state (packet channel: pure fn).
///
/// Source: `oracle/codemp/ui/ui_saber.c:74-90`
pub fn UI_ParseLiteralSilent(data: &mut Option<&[u8]>, string: &str) -> bool {
    let mut qs = QSharedScratch::zeroed();
    let (token, remaining) = COM_ParseExt(&mut qs, *data, true);
    *data = remaining;

    if token.is_empty() {
        return true;
    }

    if !token.eq_ignore_ascii_case(string) {
        return true;
    }

    false
}

/// Raven `UI_DoSaber` — draws one saber blade's glow blob and hot core into
/// the scene as two `refEntity_t`s.
///
/// Source: `oracle/codemp/ui/ui_saber.c:402-505`
pub fn UI_DoSaber(
    ctx: &mut UiContext,
    origin: vec3_t,
    dir: vec3_t,
    length: f32,
    lengthMax: f32,
    radius: f32,
    color: saber_colors_t,
) {
    if length < 0.5 {
        // if the thing is so short, just forget even adding me.
        return;
    }

    // Find the midpoint of the saber for lighting purposes
    let mut mid: vec3_t = [0.0; 3];
    _VectorMA(origin, length * 0.5, dir, &mut mid);

    let mut rgb: vec3_t = [1.0, 1.0, 1.0];
    let mut blade = 0;
    let mut glow = 0;

    match color {
        SABER_RED => {
            glow = ctx.world.saber.redSaberGlowShader;
            blade = ctx.world.saber.redSaberCoreShader;
            VectorSet(&mut rgb, 1.0, 0.2, 0.2);
        }
        SABER_ORANGE => {
            glow = ctx.world.saber.orangeSaberGlowShader;
            blade = ctx.world.saber.orangeSaberCoreShader;
            VectorSet(&mut rgb, 1.0, 0.5, 0.1);
        }
        SABER_YELLOW => {
            glow = ctx.world.saber.yellowSaberGlowShader;
            blade = ctx.world.saber.yellowSaberCoreShader;
            VectorSet(&mut rgb, 1.0, 1.0, 0.2);
        }
        SABER_GREEN => {
            glow = ctx.world.saber.greenSaberGlowShader;
            blade = ctx.world.saber.greenSaberCoreShader;
            VectorSet(&mut rgb, 0.2, 1.0, 0.2);
        }
        SABER_BLUE => {
            glow = ctx.world.saber.blueSaberGlowShader;
            blade = ctx.world.saber.blueSaberCoreShader;
            VectorSet(&mut rgb, 0.2, 0.4, 1.0);
        }
        SABER_PURPLE => {
            glow = ctx.world.saber.purpleSaberGlowShader;
            blade = ctx.world.saber.purpleSaberCoreShader;
            VectorSet(&mut rgb, 0.9, 0.2, 1.0);
        }
        _ => {}
    }
    let _ = rgb;

    // always add a light because sabers cast a nice glow before they slice you in half!!  or something...
    // Raven: the light-add call is commented out in the source (needs an
    // averaged RGB across all active saber blades) — nothing to port.

    // `memset(&saber, 0, sizeof(refEntity_t))` — zero every field, including
    // the union members, as a full literal (porting-rules: no `unsafe`).
    let mut saber = refEntity_t {
        reType: refEntityType_t::RT_MODEL,
        renderfx: 0,
        hModel: 0,
        axis: [[0.0; 3]; 3],
        nonNormalizedAxes: qfalse,
        origin: [0.0; 3],
        oldorigin: [0.0; 3],
        customShader: 0,
        shaderRGBA: [0; 4],
        shaderTexCoord: [0.0; 2],
        radius: 0.0,
        rotation: 0.0,
        shaderTime: 0.0,
        frame: 0,
        lightingOrigin: [0.0; 3],
        shadowPlane: 0.0,
        oldframe: 0,
        backlerp: 0.0,
        skinNum: 0,
        customSkin: 0,
        uRefEnt: refEntity_t_uRefEnt {
            uMini: refEntity_t_uMini {
                miniStart: 0,
                miniCount: 0,
            },
        },
        data: refEntity_t_data {
            sprite: refEntity_t_sprite {
                rotation: 0.0,
                radius: 0.0,
                vertRGBA: [[0; 4]; 4],
            },
        },
        endTime: 0.0,
        saberLength: 0.0,
        angles: [0.0; 3],
        modelScale: [0.0; 3],
        ghoul2: core::ptr::null_mut(),
    };

    // Saber glow is it's own ref type because it uses a ton of sprites, otherwise it would eat up too many
    //	refEnts to do each glow blob individually
    saber.saberLength = length;

    // Jeff, I did this because I foolishly wished to have a bright halo as the saber is unleashed.
    // It's not quite what I'd hoped tho.  If you have any ideas, go for it!  --Pat
    let radiusmult = if length < lengthMax {
        // Note this creates a curve, and length cannot be < 0.5.
        1.0 + (2.0 / length)
    } else {
        1.0
    };

    let radiusRange = radius * 0.075;
    let mut radiusStart = radius - radiusRange;

    // PORT-NOTE: `crandom()` routes through ui's own bg state
    // (`world.bg_state.rng`, DEC-36 addendum 11) — Raven's ui link unit had
    // its own libc rand, never the game's.
    saber.radius = ((radiusStart as f64 + ctx.world.bg_state.rng.crandom() * radiusRange as f64)
        * radiusmult as f64) as f32;

    // `VectorCopy(origin, saber.origin)` / `VectorCopy(dir, saber.axis[0])` —
    // `vec3_t` (`[f32; 3]`) is `Copy`, so the macro collapses to assignment.
    saber.origin = origin;
    saber.axis[0] = dir;
    saber.reType = refEntityType_t::RT_SABER_GLOW;
    saber.customShader = glow;
    saber.shaderRGBA[0] = 0xff;
    saber.shaderRGBA[1] = 0xff;
    saber.shaderRGBA[2] = 0xff;
    saber.shaderRGBA[3] = 0xff;
    //saber.renderfx = rfx;

    trap::R_AddRefEntityToScene(ctx.engine, &saber);

    // Do the hot core
    _VectorMA(origin, length, dir, &mut saber.origin);
    _VectorMA(origin, -1.0, dir, &mut saber.oldorigin);
    saber.customShader = blade;
    saber.reType = refEntityType_t::RT_LINE;
    radiusStart = radius / 3.0;
    saber.radius = ((radiusStart as f64 + ctx.world.bg_state.rng.crandom() * radiusRange as f64)
        * radiusmult as f64) as f32;

    trap::R_AddRefEntityToScene(ctx.engine, &saber);
}

/// Raven `SaberColorToString` — the color-constant to name-string table used
/// by the saber-selection UI.
///
/// PORT-NOTE: Raven returns `NULL` for an out-of-range `color`; that becomes
/// `None` here (dictionary: `char*` -> `&str`, NULL preserved as `Option`).
///
/// Source: `oracle/codemp/ui/ui_saber.c:507-527`
pub fn SaberColorToString(color: saber_colors_t) -> Option<&'static str> {
    if color == SABER_RED {
        return Some("red");
    }
    if color == SABER_ORANGE {
        return Some("orange");
    }
    if color == SABER_YELLOW {
        return Some("yellow");
    }
    if color == SABER_GREEN {
        return Some("green");
    }
    if color == SABER_BLUE {
        return Some("blue");
    }
    if color == SABER_PURPLE {
        return Some("purple");
    }
    None
}

/// Raven `TranslateSaberColor` — the name-string to color-constant table used
/// by the saber-selection UI (the inverse of `SaberColorToString`, plus a
/// `"random"` case).
///
/// Source: `oracle/codemp/ui/ui_saber.c:528-559`
pub fn TranslateSaberColor(name: &str, bg: &mut BgState) -> saber_colors_t {
    if name.eq_ignore_ascii_case("red") {
        return SABER_RED;
    }
    if name.eq_ignore_ascii_case("orange") {
        return SABER_ORANGE;
    }
    if name.eq_ignore_ascii_case("yellow") {
        return SABER_YELLOW;
    }
    if name.eq_ignore_ascii_case("green") {
        return SABER_GREEN;
    }
    if name.eq_ignore_ascii_case("blue") {
        return SABER_BLUE;
    }
    if name.eq_ignore_ascii_case("purple") {
        return SABER_PURPLE;
    }
    if name.eq_ignore_ascii_case("random") {
        // PORT-NOTE: `Q_irand` routes through ui's own bg state (`bg.rng`,
        // DEC-36 addendum 11), mirroring the `bg_saberLoad.rs` twin's
        // `bg: &mut BgState` threading.
        return bg.rng.Q_irand(SABER_ORANGE, SABER_PURPLE);
    }
    SABER_BLUE
}

/// Raven `TranslateSaberType` — the name-string to `saberType_t` table used by
/// the saber-selection UI.
///
/// Source: `oracle/codemp/ui/ui_saber.c:561-612`
pub fn TranslateSaberType(name: &str) -> saberType_t {
    if name.eq_ignore_ascii_case("SABER_SINGLE") {
        return saberType_t::SABER_SINGLE;
    }
    if name.eq_ignore_ascii_case("SABER_STAFF") {
        return saberType_t::SABER_STAFF;
    }
    if name.eq_ignore_ascii_case("SABER_BROAD") {
        return saberType_t::SABER_BROAD;
    }
    if name.eq_ignore_ascii_case("SABER_PRONG") {
        return saberType_t::SABER_PRONG;
    }
    if name.eq_ignore_ascii_case("SABER_DAGGER") {
        return saberType_t::SABER_DAGGER;
    }
    if name.eq_ignore_ascii_case("SABER_ARC") {
        return saberType_t::SABER_ARC;
    }
    if name.eq_ignore_ascii_case("SABER_SAI") {
        return saberType_t::SABER_SAI;
    }
    if name.eq_ignore_ascii_case("SABER_CLAW") {
        return saberType_t::SABER_CLAW;
    }
    if name.eq_ignore_ascii_case("SABER_LANCE") {
        return saberType_t::SABER_LANCE;
    }
    if name.eq_ignore_ascii_case("SABER_STAR") {
        return saberType_t::SABER_STAR;
    }
    if name.eq_ignore_ascii_case("SABER_TRIDENT") {
        return saberType_t::SABER_TRIDENT;
    }
    if name.eq_ignore_ascii_case("SABER_SITH_SWORD") {
        return saberType_t::SABER_SITH_SWORD;
    }
    saberType_t::SABER_SINGLE
}
