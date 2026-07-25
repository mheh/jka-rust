//! `ui_saber.c` — the saber-selection screen.
//!
//! Source: `oracle/codemp/ui/ui_saber.c`

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use core::mem;

use mp_bg::bg_channel::BgState;
use mp_bg::bg_misc::BG_GiveMeVectorFromMatrix;
use mp_qshared::common::mp::cgame::ref_entity_t::{
    refEntity_t, refEntity_t_data, refEntity_t_sprite, refEntity_t_uMini, refEntity_t_uRefEnt,
};
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::qcommon::saber::saber_colors::{
    saber_colors_t, SABER_BLUE, SABER_GREEN, SABER_ORANGE, SABER_PURPLE, SABER_RED, SABER_YELLOW,
};
use mp_qshared::common::mp::qcommon::saber::saber_type::saberType_t;
use mp_qshared::shared::com_parse::{
    COM_BeginParseSession, COM_ParseExt, COM_ParseString, QSharedScratch, SkipBracedSection,
    SkipRestOfLine,
};
use mp_qshared::shared::q_color::S_COLOR_RED;
use mp_qshared::shared::q_math::{_VectorMA, vec3_origin, VectorNormalize, VectorSet};
use mp_qshared::shared::q_string::COM_Compress;
use mp_qshared::shared::{fileHandle_t, mdxaBone_t, vec3_t, Eorientations, FS_READ, MAX_QPATH};
use mp_uishared::shared::item_def_s::ItemDef;
use mp_uishared::shared::menu_system::MAX_MENUFILE;
use mp_uishared::ui_shared::{String_Alloc, ITF_ISCHARACTER, ITF_ISSABER, ITF_ISSABER2};
use native_string::{atof, atoi, latin1_to_string};
use native_types::qfalse;

use crate::trap;
use crate::ui_atoms::{Com_Error, Com_Printf};
use crate::world::ui_context::UiContext;
use crate::world::ui_main_state::MAX_SABER_HILTS;
use crate::world::ui_saber_state::MAX_SABER_DATA_SIZE;

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

/// Raven `UI_ParseLiteral` — consumes one token and reports whether it did
/// NOT match `string` (`qtrue` on EOF or a mismatch — printing a diagnostic
/// either way — `qfalse` when the token matched).
///
/// PORT-NOTE: same in/out `Option<&[u8]>` cursor shape as `UI_ParseLiteralSilent`
/// (genuinely read-modify-write); `Q_stricmp(token, string) != 0` becomes
/// `!token.eq_ignore_ascii_case(string)`. Unlike the silent twin this fn
/// forwards diagnostics through the engine, so it threads `ctx` — and with it
/// the session's `COM_ParseExt` scratch (`ctx.world.bg_state.qs`).
///
/// Source: `oracle/codemp/ui/ui_saber.c:54-72`
pub fn UI_ParseLiteral(ctx: &mut UiContext, data: &mut Option<&[u8]>, string: &str) -> bool {
    let (token, remaining) = COM_ParseExt(&mut ctx.world.bg_state.qs, *data, true);
    *data = remaining;

    if token.is_empty() {
        Com_Printf(ctx, "unexpected EOF\n");
        return true;
    }

    if !token.eq_ignore_ascii_case(string) {
        Com_Printf(ctx, &format!("required string '{}' missing\n", string));
        return true;
    }

    false
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

/// Raven `UI_SaberLoadParms` — reads every `ext_data/sabers/*.sab` extension
/// file and concatenates their (comment/whitespace-compressed) text into the
/// hilt-parse buffer every `UI_SaberLoad*` lookup reparses.
///
/// PORT-NOTE: Raven's `char saberExtensionListBuf[2048]`/`marker` pointer
/// walk becomes a byte-offset walk over a fixed `[u8; 2048]` list buffer and
/// a `String` accumulator (`world.saber.SaberParms`) — `totallen` collapses
/// to `SaberParms.chars().count()` (wire bytes, one per Latin-1 char),
/// `*(marker-1) == '}'` to `SaberParms.ends_with('}')`.
/// `trap_FS_GetFileList` NUL-separates names inside `listbuf` (per the
/// wrapper's doc); each entry is decoded with `latin1_to_string` (opaque
/// engine-filled bytes, #13 discipline) to match the filesystem-path
/// dictionary entry. Raven's two `Com_Error( ERR_FATAL, ... )` calls never
/// return (longjmp out of the ui module); the port makes that explicit with
/// an early `return` since there is no Rust analog of the longjmp.
///
/// Source: `oracle/codemp/ui/ui_saber.c:336-400`
pub fn UI_SaberLoadParms(ctx: &mut UiContext) {
    ctx.world.saber.ui_saber_parms_parsed = true;
    UI_CacheSaberGlowGraphics(ctx);

    ctx.world.saber.SaberParms.clear();

    let mut listbuf = [0u8; 2048];
    let file_cnt = trap::FS_GetFileList(ctx.engine, "ext_data/sabers", ".sab", &mut listbuf);

    let mut offset = 0usize;
    for _ in 0..file_cnt {
        // PORT-NOTE: an unterminated tail ends the walk rather than running off the
        // buffer.
        if offset >= listbuf.len() {
            break;
        }
        let end = listbuf[offset..]
            .iter()
            .position(|&b| b == 0)
            .map(|p| offset + p)
            .unwrap_or(listbuf.len());
        let hold = latin1_to_string(&listbuf[offset..end]);
        offset = end + 1;

        let mut f: fileHandle_t = 0;
        let len = trap::FS_FOpenFile(
            ctx.engine,
            &format!("ext_data/sabers/{}", hold),
            &mut f,
            FS_READ,
        );

        if f == 0 {
            continue;
        }

        if len == -1 {
            Com_Printf(ctx, &format!("UI_SaberLoadParms: error reading {}\n", hold));
            continue;
        }

        if len as usize > MAX_MENUFILE {
            Com_Error(
                ctx,
                &format!(
                    "UI_SaberLoadParms: file {} too large to read (max={})",
                    hold, MAX_MENUFILE
                ),
            );
            return;
        }

        // PORT-NOTE (§19): Raven writes `buffer[len] = 0` one past its fixed array;
        // the owned buffer carries the extra byte.
        let mut buffer = vec![0u8; len as usize + 1];
        trap::FS_Read(ctx.engine, &mut buffer[..len as usize], f);
        trap::FS_FCloseFile(ctx.engine, f);
        buffer[len as usize] = 0;

        if !ctx.world.saber.SaberParms.is_empty() && ctx.world.saber.SaberParms.ends_with('}') {
            ctx.world.saber.SaberParms.push(' ');
        }

        let compressed_len = COM_Compress(buffer.as_mut_ptr() as *mut c_char);
        let compressed = latin1_to_string(&buffer[..compressed_len as usize]);

        // Raven's `totallen` counts wire bytes; each Latin-1 char is one wire byte.
        if ctx.world.saber.SaberParms.chars().count() + compressed.chars().count()
            >= MAX_SABER_DATA_SIZE
        {
            Com_Error(
                ctx,
                &format!(
                    "UI_SaberLoadParms: ran out of space before reading {}\n(you must make the .sab files smaller)",
                    hold
                ),
            );
            return;
        }
        ctx.world.saber.SaberParms.push_str(&compressed);
    }
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

/// Raven `UI_SaberParseParm` — looks up `saberName`'s block in the cached
/// `.sab` extension text (`world.saber.SaberParms`) and returns the string
/// value bound to `parmname` inside it.
///
/// PORT-NOTE: Raven's `char *saberData` out-param collapses into a returned
/// `Option<String>` (dictionary: out-param -> return, `qboolean` success flag
/// -> `Some`/`None`). `p` walks `SaberParms`'s bytes with the same
/// `Option<&[u8]>` cursor shape `COM_ParseExt`/`UI_ParseLiteral` already use
/// elsewhere in this file; the `while ( p )` search loop becomes a
/// `while p.is_some()` loop with an early labeled `break` on a name match.
///
/// Source: `oracle/codemp/ui/ui_saber.c:92-164`
pub fn UI_SaberParseParm(ctx: &mut UiContext, saberName: &str, parmname: &str) -> Option<String> {
    if saberName.is_empty() {
        return None;
    }

    // try to parse it out. The cursor walks the cached text, so the text moves
    // out of `world.saber` for the parse and back afterwards — the rest of the
    // context stays mutably reachable without copying it.
    let saber_parms = mem::take(&mut ctx.world.saber.SaberParms);
    let result = 'parse: {
        let mut p: Option<&[u8]> = Some(saber_parms.as_bytes());
        // A bogus name is passed in
        COM_BeginParseSession(&mut ctx.world.bg_state.qs, "saberinfo");

        // look for the right saber
        'search: while p.is_some() {
            let (token, rest) = COM_ParseExt(&mut ctx.world.bg_state.qs, p, true);
            p = rest;
            if token.is_empty() {
                break 'parse None;
            }

            if token.eq_ignore_ascii_case(saberName) {
                break 'search;
            }

            p = SkipBracedSection(&mut ctx.world.bg_state.qs, p);
        }
        if p.is_none() {
            break 'parse None;
        }

        if UI_ParseLiteral(ctx, &mut p, "{") {
            break 'parse None;
        }

        // parse the saber info block
        loop {
            let (token, rest) = COM_ParseExt(&mut ctx.world.bg_state.qs, p, true);
            p = rest;
            if token.is_empty() {
                Com_Printf(
                    ctx,
                    &format!(
                        "{}ERROR: unexpected EOF while parsing '{}'\n",
                        S_COLOR_RED.to_str().unwrap(),
                        saberName
                    ),
                );
                break 'parse None;
            }

            if token.eq_ignore_ascii_case("}") {
                break;
            }

            if token.eq_ignore_ascii_case(parmname) {
                // PORT-NOTE: `COM_ParseString`'s EOF guard is dead (see its
                // doc comment) — this call never observes the `qtrue`/EOF
                // branch.
                let (value, _) = COM_ParseString(&mut ctx.world.bg_state.qs, p);
                break 'parse Some(value);
            }

            p = SkipRestOfLine(&mut ctx.world.bg_state.qs, p);
        }

        None
    };
    ctx.world.saber.SaberParms = saber_parms;
    result
}

/// Raven `UI_SaberModelForSaber` — looks up `saberModel` for `saberName`.
///
/// PORT-NOTE: `char *saberModel` out-param + `qboolean` found-flag collapses
/// into `UI_SaberParseParm`'s `Option<String>` (dictionary: out-param ->
/// return).
///
/// Source: `oracle/codemp/ui/ui_saber.c:167-170`
pub fn UI_SaberModelForSaber(ctx: &mut UiContext, saberName: &str) -> Option<String> {
    UI_SaberParseParm(ctx, saberName, "saberModel")
}

/// Raven `UI_SaberSkinForSaber` — looks up `customSkin` for `saberName`.
///
/// Source: `oracle/codemp/ui/ui_saber.c:172-175`
pub fn UI_SaberSkinForSaber(ctx: &mut UiContext, saberName: &str) -> Option<String> {
    UI_SaberParseParm(ctx, saberName, "customSkin")
}

/// Raven `UI_SaberTypeForSaber` — looks up `saberType` for `saberName`.
///
/// Source: `oracle/codemp/ui/ui_saber.c:177-180`
pub fn UI_SaberTypeForSaber(ctx: &mut UiContext, saberName: &str) -> Option<String> {
    UI_SaberParseParm(ctx, saberName, "saberType")
}

/// Raven `UI_SaberNumBladesForSaber` — looks up `numBlades` for `saberName`,
/// clamped to `[1, 8]` (defaulting to `1` when unparsed/missing).
///
/// Source: `oracle/codemp/ui/ui_saber.c:182-197`
pub fn UI_SaberNumBladesForSaber(ctx: &mut UiContext, saberName: &str) -> i32 {
    let numBladesString = UI_SaberParseParm(ctx, saberName, "numBlades").unwrap_or_default();
    let mut numBlades = atoi(&numBladesString);
    if numBlades < 1 {
        numBlades = 1;
    } else if numBlades > 8 {
        numBlades = 8;
    }
    numBlades
}

/// Raven `UI_SaberShouldDrawBlade` — whether blade `bladeNum` of `saberName`
/// should be drawn, per the (optionally two-style) `noBlade`/`noBlade2`
/// bitmask parms gated by `bladeStyle2Start`.
///
/// Source: `oracle/codemp/ui/ui_saber.c:199-230`
pub fn UI_SaberShouldDrawBlade(ctx: &mut UiContext, saberName: &str, bladeNum: i32) -> bool {
    let mut bladeStyle2Start = 0;
    let mut noBlade = 0;

    let bladeStyle2StartString =
        UI_SaberParseParm(ctx, saberName, "bladeStyle2Start").unwrap_or_default();
    if !bladeStyle2StartString.is_empty() {
        bladeStyle2Start = atoi(&bladeStyle2StartString);
    }

    if bladeStyle2Start != 0 && bladeNum >= bladeStyle2Start {
        // use second blade style
        let noBladeString = UI_SaberParseParm(ctx, saberName, "noBlade2").unwrap_or_default();
        if !noBladeString.is_empty() {
            noBlade = atoi(&noBladeString);
        }
    } else {
        // use first blade style
        let noBladeString = UI_SaberParseParm(ctx, saberName, "noBlade").unwrap_or_default();
        if !noBladeString.is_empty() {
            noBlade = atoi(&noBladeString);
        }
    }

    noBlade == 0
}

/// Raven `UI_IsSaberTwoHanded` — whether `saberName` is held two-handed
/// (`twoHanded` parm; undefined defaults to `qfalse`).
///
/// Source: `oracle/codemp/ui/ui_saber.c:233-244`
pub fn UI_IsSaberTwoHanded(ctx: &mut UiContext, saberName: &str) -> bool {
    let twoHandedString = UI_SaberParseParm(ctx, saberName, "twoHanded").unwrap_or_default();
    if twoHandedString.is_empty() {
        // not defined defaults to "no"
        return false;
    }
    atoi(&twoHandedString) != 0
}

/// Raven `UI_SaberBladeLengthForSaber` — blade length for `saberName`'s blade
/// `bladeNum`: `saberLength` (default `40.0`), overridden by the per-blade
/// `saberLength<N+1>` parm when present, each clamped to `>= 0.0`.
///
/// Source: `oracle/codemp/ui/ui_saber.c:246-271`
pub fn UI_SaberBladeLengthForSaber(ctx: &mut UiContext, saberName: &str, bladeNum: i32) -> f32 {
    let mut length = 40.0f32;

    let lengthString = UI_SaberParseParm(ctx, saberName, "saberLength").unwrap_or_default();
    if !lengthString.is_empty() {
        length = atof(&lengthString) as f32;
        if length < 0.0 {
            length = 0.0;
        }
    }

    let perBlade = UI_SaberParseParm(ctx, saberName, &format!("saberLength{}", bladeNum + 1))
        .unwrap_or_default();
    if !perBlade.is_empty() {
        length = atof(&perBlade) as f32;
        if length < 0.0 {
            length = 0.0;
        }
    }

    length
}

/// Raven `UI_SaberBladeRadiusForSaber` — blade radius for `saberName`'s blade
/// `bladeNum`: `saberRadius` (default `3.0`), overridden by the per-blade
/// `saberRadius<N+1>` parm when present, each clamped to `>= 0.0`.
///
/// Source: `oracle/codemp/ui/ui_saber.c:273-298`
pub fn UI_SaberBladeRadiusForSaber(ctx: &mut UiContext, saberName: &str, bladeNum: i32) -> f32 {
    let mut radius = 3.0f32;

    let radiusString = UI_SaberParseParm(ctx, saberName, "saberRadius").unwrap_or_default();
    if !radiusString.is_empty() {
        radius = atof(&radiusString) as f32;
        if radius < 0.0 {
            radius = 0.0;
        }
    }

    let perBlade = UI_SaberParseParm(ctx, saberName, &format!("saberRadius{}", bladeNum + 1))
        .unwrap_or_default();
    if !perBlade.is_empty() {
        radius = atof(&perBlade) as f32;
        if radius < 0.0 {
            radius = 0.0;
        }
    }

    radius
}

/// Raven `UI_SaberProperNameForSaber` — the display name for `saberName`
/// (`name` parm); a leading `@` marks a string-package reference, translated
/// through `trap_SP_GetStringTextString`.
///
/// PORT-NOTE: the `qboolean ret` return (whether `name` was found at all)
/// collapses into the `Option<String>` return per the out-param dictionary —
/// `None` on not-found, `Some(<proper name>)` (translated, or the raw
/// stringed name) when found. Raven writes into the caller's own buffer
/// (`ui_main.c:8793,8799`) and discards the `trap_SP_GetStringTextString`
/// return, so a lookup miss leaves that buffer's prior contents; this port
/// falls back to the empty string (`unwrap_or_default`).
///
/// Source: `oracle/codemp/ui/ui_saber.c:300-317`
pub fn UI_SaberProperNameForSaber(ctx: &mut UiContext, saberName: &str) -> Option<String> {
    let stringedSaberName = UI_SaberParseParm(ctx, saberName, "name")?;
    // if it's a stringed reference translate it
    if stringedSaberName.starts_with('@') {
        let translated = trap::SP_GetStringTextString(ctx.engine, &stringedSaberName[1..], 1024);
        Some(translated.unwrap_or_default())
    } else {
        // no stringed so just use it as it
        Some(stringedSaberName)
    }
}

/// Raven `UI_SaberValidForPlayerInMP` — whether `saberName` is usable in MP
/// (`notInMP` parm; undefined, empty, or `0` all default to allowed).
///
/// Source: `oracle/codemp/ui/ui_saber.c:319-334`
pub fn UI_SaberValidForPlayerInMP(ctx: &mut UiContext, saberName: &str) -> bool {
    match UI_SaberParseParm(ctx, saberName, "notInMP") {
        None => {
            // not defined, default is yes
            true
        }
        Some(allowed) => {
            if allowed.is_empty() {
                // not defined, default is yes
                true
            } else {
                atoi(&allowed) == 0
            }
        }
    }
}

/// Raven `UI_SaberDrawBlade` — computes one saber blade's world origin/axis
/// off the model's bolt tag (or a fallback `*flash` tag / hardcoded offsets
/// when the tag is missing), then draws it with `UI_DoSaber`.
///
/// PORT-NOTE: Raven's `char *tagName = va(...)` becomes a `format!`; the
/// `tagHack` `qboolean` stays a `bool`. `G2API_HasGhoul2ModelOnIndex` takes the
/// ADDRESS of the `ghoul2` slot (the engine derefs arg1 as `CGhoul2Info_v **` —
/// `cl_ui.cpp:1341`), while `G2API_AddBolt`/`G2API_GetBoltMatrix` take the
/// handle value itself.
///
/// Source: `oracle/codemp/ui/ui_saber.c:614-846`
#[allow(clippy::too_many_arguments)]
pub fn UI_SaberDrawBlade(
    ctx: &mut UiContext,
    item: &ItemDef,
    saberName: &str,
    saberModel: c_int,
    saberType: saberType_t,
    origin: vec3_t,
    angles: vec3_t,
    bladeNum: i32,
) {
    let bladeColorString: String;
    if (item.flags & ITF_ISSABER) != 0 && saberModel < 2 {
        bladeColorString = trap::Cvar_VariableStringBuffer(ctx.engine, "ui_saber_color", MAX_QPATH);
    } else {
        // if ( item->flags&ITF_ISSABER2 ) - presumed
        bladeColorString =
            trap::Cvar_VariableStringBuffer(ctx.engine, "ui_saber2_color", MAX_QPATH);
    }

    // Raven `&(item->ghoul2)`: the engine only reads the slot here, so the
    // shared borrow is cast to the seam's `*mut c_void` slot token.
    let g2slot = &item.ghoul2 as *const *mut c_void as *mut c_void;
    if !trap::G2API_HasGhoul2ModelOnIndex(ctx.engine, g2slot, saberModel) {
        // invalid index!
        return;
    }

    let bladeColor = TranslateSaberColor(&bladeColorString, &mut ctx.world.bg_state);

    let bladeLength = UI_SaberBladeLengthForSaber(ctx, saberName, bladeNum);
    let bladeRadius = UI_SaberBladeRadiusForSaber(ctx, saberName, bladeNum);

    let tagName = format!("*blade{}", bladeNum + 1);
    let mut bolt = trap::G2API_AddBolt(ctx.engine, item.ghoul2, saberModel, &tagName);

    let mut tagHack = false;
    if bolt == -1 {
        tagHack = true;
        // hmm, just fall back to the most basic tag (this will also make it work with
        // pre-JKA saber models
        bolt = trap::G2API_AddBolt(ctx.engine, item.ghoul2, saberModel, "*flash");
        if bolt == -1 {
            // no tag_flash either?!!
            bolt = 0;
        }
    }

    let mut boltMatrix = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };
    // NULL was cgs.model_draw
    trap::G2API_GetBoltMatrix(
        ctx.engine,
        item.ghoul2,
        saberModel,
        bolt,
        &mut boltMatrix,
        &angles,
        &origin,
        ctx.world.uiDC.realTime,
        None,
        &vec3_origin,
    );

    // work the matrix axis stuff into the original axis and origins used.
    let mut bladeOrigin: vec3_t = [0.0; 3];
    let mut axis: [vec3_t; 3] = [[0.0; 3]; 3];
    BG_GiveMeVectorFromMatrix(
        &boltMatrix,
        Eorientations::ORIGIN as c_int,
        &mut bladeOrigin,
    );
    // front (was NEGATIVE_Y, but the md3->glm exporter screws up this tag somethin'
    // awful) ...changed this back to NEGATIVE_Y
    BG_GiveMeVectorFromMatrix(
        &boltMatrix,
        Eorientations::NEGATIVE_Y as c_int,
        &mut axis[0],
    );
    // right ... and changed this to NEGATIVE_X
    BG_GiveMeVectorFromMatrix(
        &boltMatrix,
        Eorientations::NEGATIVE_X as c_int,
        &mut axis[1],
    );
    // up
    BG_GiveMeVectorFromMatrix(
        &boltMatrix,
        Eorientations::POSITIVE_Z as c_int,
        &mut axis[2],
    );

    // Where do I get scale from?
    // scale = DC->xscale;
    let scale = 1.0f32;

    if tagHack {
        match saberType {
            saberType_t::SABER_SINGLE => {
                let mut out = bladeOrigin;
                _VectorMA(bladeOrigin, scale, axis[0], &mut out);
                bladeOrigin = out;
            }
            saberType_t::SABER_DAGGER | saberType_t::SABER_LANCE => {}
            saberType_t::SABER_STAFF => {
                if bladeNum == 0 {
                    let mut out = bladeOrigin;
                    _VectorMA(bladeOrigin, 12.0 * scale, axis[0], &mut out);
                    bladeOrigin = out;
                }
                if bladeNum == 1 {
                    axis[0] = [-axis[0][0], -axis[0][1], -axis[0][2]];
                    let mut out = bladeOrigin;
                    _VectorMA(bladeOrigin, 12.0 * scale, axis[0], &mut out);
                    bladeOrigin = out;
                }
            }
            saberType_t::SABER_BROAD => {
                if bladeNum == 0 {
                    let mut out = bladeOrigin;
                    _VectorMA(bladeOrigin, -1.0 * scale, axis[1], &mut out);
                    bladeOrigin = out;
                } else if bladeNum == 1 {
                    let mut out = bladeOrigin;
                    _VectorMA(bladeOrigin, 1.0 * scale, axis[1], &mut out);
                    bladeOrigin = out;
                }
            }
            saberType_t::SABER_PRONG => {
                if bladeNum == 0 {
                    let mut out = bladeOrigin;
                    _VectorMA(bladeOrigin, -3.0 * scale, axis[1], &mut out);
                    bladeOrigin = out;
                } else if bladeNum == 1 {
                    let mut out = bladeOrigin;
                    _VectorMA(bladeOrigin, 3.0 * scale, axis[1], &mut out);
                    bladeOrigin = out;
                }
            }
            saberType_t::SABER_ARC => {
                axis[1] = [
                    axis[1][0] - axis[2][0],
                    axis[1][1] - axis[2][1],
                    axis[1][2] - axis[2][2],
                ];
                let mut normalized = axis[1];
                VectorNormalize(&mut normalized);
                axis[1] = normalized;
                match bladeNum {
                    0 => {
                        let mut out = bladeOrigin;
                        _VectorMA(bladeOrigin, 8.0 * scale, axis[0], &mut out);
                        bladeOrigin = out;
                        axis[0] = [axis[0][0] * 0.75, axis[0][1] * 0.75, axis[0][2] * 0.75];
                        axis[1] = [axis[1][0] * 0.25, axis[1][1] * 0.25, axis[1][2] * 0.25];
                        axis[0] = [
                            axis[0][0] + axis[1][0],
                            axis[0][1] + axis[1][1],
                            axis[0][2] + axis[1][2],
                        ];
                    }
                    1 => {
                        axis[0] = [axis[0][0] * 0.25, axis[0][1] * 0.25, axis[0][2] * 0.25];
                        axis[1] = [axis[1][0] * 0.75, axis[1][1] * 0.75, axis[1][2] * 0.75];
                        axis[0] = [
                            axis[0][0] + axis[1][0],
                            axis[0][1] + axis[1][1],
                            axis[0][2] + axis[1][2],
                        ];
                    }
                    2 => {
                        let mut out = bladeOrigin;
                        _VectorMA(bladeOrigin, -8.0 * scale, axis[0], &mut out);
                        bladeOrigin = out;
                        axis[0] = [axis[0][0] * -0.25, axis[0][1] * -0.25, axis[0][2] * -0.25];
                        axis[1] = [axis[1][0] * 0.75, axis[1][1] * 0.75, axis[1][2] * 0.75];
                        axis[0] = [
                            axis[0][0] + axis[1][0],
                            axis[0][1] + axis[1][1],
                            axis[0][2] + axis[1][2],
                        ];
                    }
                    3 => {
                        let mut out = bladeOrigin;
                        _VectorMA(bladeOrigin, -16.0 * scale, axis[0], &mut out);
                        bladeOrigin = out;
                        axis[0] = [axis[0][0] * -0.75, axis[0][1] * -0.75, axis[0][2] * -0.75];
                        axis[1] = [axis[1][0] * 0.25, axis[1][1] * 0.25, axis[1][2] * 0.25];
                        axis[0] = [
                            axis[0][0] + axis[1][0],
                            axis[0][1] + axis[1][1],
                            axis[0][2] + axis[1][2],
                        ];
                    }
                    _ => {}
                }
            }
            saberType_t::SABER_SAI => {
                if bladeNum == 1 {
                    let mut out = bladeOrigin;
                    _VectorMA(bladeOrigin, -3.0 * scale, axis[1], &mut out);
                    bladeOrigin = out;
                } else if bladeNum == 2 {
                    let mut out = bladeOrigin;
                    _VectorMA(bladeOrigin, 3.0 * scale, axis[1], &mut out);
                    bladeOrigin = out;
                }
            }
            saberType_t::SABER_CLAW => match bladeNum {
                0 => {
                    let mut out = bladeOrigin;
                    _VectorMA(bladeOrigin, 2.0 * scale, axis[0], &mut out);
                    bladeOrigin = out;
                    let mut out2 = bladeOrigin;
                    _VectorMA(bladeOrigin, 2.0 * scale, axis[2], &mut out2);
                    bladeOrigin = out2;
                }
                1 => {
                    let mut out = bladeOrigin;
                    _VectorMA(bladeOrigin, 2.0 * scale, axis[0], &mut out);
                    bladeOrigin = out;
                    let mut out2 = bladeOrigin;
                    _VectorMA(bladeOrigin, 2.0 * scale, axis[2], &mut out2);
                    bladeOrigin = out2;
                    let mut out3 = bladeOrigin;
                    _VectorMA(bladeOrigin, 2.0 * scale, axis[1], &mut out3);
                    bladeOrigin = out3;
                }
                2 => {
                    let mut out = bladeOrigin;
                    _VectorMA(bladeOrigin, 2.0 * scale, axis[0], &mut out);
                    bladeOrigin = out;
                    let mut out2 = bladeOrigin;
                    _VectorMA(bladeOrigin, 2.0 * scale, axis[2], &mut out2);
                    bladeOrigin = out2;
                    let mut out3 = bladeOrigin;
                    _VectorMA(bladeOrigin, -2.0 * scale, axis[1], &mut out3);
                    bladeOrigin = out3;
                }
                _ => {}
            },
            saberType_t::SABER_STAR => match bladeNum {
                0 => {
                    let mut out = bladeOrigin;
                    _VectorMA(bladeOrigin, 8.0 * scale, axis[0], &mut out);
                    bladeOrigin = out;
                }
                1 => {
                    axis[0] = [axis[0][0] * 0.33, axis[0][1] * 0.33, axis[0][2] * 0.33];
                    axis[2] = [axis[2][0] * 0.67, axis[2][1] * 0.67, axis[2][2] * 0.67];
                    axis[0] = [
                        axis[0][0] + axis[2][0],
                        axis[0][1] + axis[2][1],
                        axis[0][2] + axis[2][2],
                    ];
                    let mut out = bladeOrigin;
                    _VectorMA(bladeOrigin, 8.0 * scale, axis[0], &mut out);
                    bladeOrigin = out;
                }
                2 => {
                    axis[0] = [axis[0][0] * -0.33, axis[0][1] * -0.33, axis[0][2] * -0.33];
                    axis[2] = [axis[2][0] * 0.67, axis[2][1] * 0.67, axis[2][2] * 0.67];
                    axis[0] = [
                        axis[0][0] + axis[2][0],
                        axis[0][1] + axis[2][1],
                        axis[0][2] + axis[2][2],
                    ];
                    let mut out = bladeOrigin;
                    _VectorMA(bladeOrigin, 8.0 * scale, axis[0], &mut out);
                    bladeOrigin = out;
                }
                3 => {
                    axis[0] = [-axis[0][0], -axis[0][1], -axis[0][2]];
                    let mut out = bladeOrigin;
                    _VectorMA(bladeOrigin, 8.0 * scale, axis[0], &mut out);
                    bladeOrigin = out;
                }
                4 => {
                    axis[0] = [axis[0][0] * -0.33, axis[0][1] * -0.33, axis[0][2] * -0.33];
                    axis[2] = [axis[2][0] * -0.67, axis[2][1] * -0.67, axis[2][2] * -0.67];
                    axis[0] = [
                        axis[0][0] + axis[2][0],
                        axis[0][1] + axis[2][1],
                        axis[0][2] + axis[2][2],
                    ];
                    let mut out = bladeOrigin;
                    _VectorMA(bladeOrigin, 8.0 * scale, axis[0], &mut out);
                    bladeOrigin = out;
                }
                5 => {
                    axis[0] = [axis[0][0] * 0.33, axis[0][1] * 0.33, axis[0][2] * 0.33];
                    axis[2] = [axis[2][0] * -0.67, axis[2][1] * -0.67, axis[2][2] * -0.67];
                    axis[0] = [
                        axis[0][0] + axis[2][0],
                        axis[0][1] + axis[2][1],
                        axis[0][2] + axis[2][2],
                    ];
                    let mut out = bladeOrigin;
                    _VectorMA(bladeOrigin, 8.0 * scale, axis[0], &mut out);
                    bladeOrigin = out;
                }
                _ => {}
            },
            saberType_t::SABER_TRIDENT => match bladeNum {
                0 => {
                    let mut out = bladeOrigin;
                    _VectorMA(bladeOrigin, 24.0 * scale, axis[0], &mut out);
                    bladeOrigin = out;
                }
                1 => {
                    let mut out = bladeOrigin;
                    _VectorMA(bladeOrigin, -6.0 * scale, axis[1], &mut out);
                    bladeOrigin = out;
                    let mut out2 = bladeOrigin;
                    _VectorMA(bladeOrigin, 24.0 * scale, axis[0], &mut out2);
                    bladeOrigin = out2;
                }
                2 => {
                    let mut out = bladeOrigin;
                    _VectorMA(bladeOrigin, 6.0 * scale, axis[1], &mut out);
                    bladeOrigin = out;
                    let mut out2 = bladeOrigin;
                    _VectorMA(bladeOrigin, 24.0 * scale, axis[0], &mut out2);
                    bladeOrigin = out2;
                }
                3 => {
                    let mut out = bladeOrigin;
                    _VectorMA(bladeOrigin, -32.0 * scale, axis[0], &mut out);
                    bladeOrigin = out;
                    axis[0] = [-axis[0][0], -axis[0][1], -axis[0][2]];
                }
                _ => {}
            },
            saberType_t::SABER_SITH_SWORD => {
                // no blade
            }
            _ => {}
        }
    }
    if saberType == saberType_t::SABER_SITH_SWORD {
        // draw no blade
        return;
    }

    UI_DoSaber(
        ctx,
        bladeOrigin,
        axis[0],
        bladeLength,
        bladeLength,
        bladeRadius,
        bladeColor,
    );
}

/// Raven `UI_GetSaberForMenu` — resolves `ui_saber`/`ui_saber2`'s current
/// value (falling back to `"kyle"` when the cvar names a saber invalid in
/// MP), then applies the current move-style's single/staff override.
///
/// PORT-NOTE: Raven's `char *saber` out-param collapses into a returned
/// `String` (dictionary: out-param -> return).
///
/// Source: `oracle/codemp/ui/ui_saber.c:894-950`
pub fn UI_GetSaberForMenu(ctx: &mut UiContext, saberNum: i32) -> String {
    let mut saber: String;

    if saberNum == 0 {
        saber = trap::Cvar_VariableStringBuffer(ctx.engine, "ui_saber", MAX_QPATH);
        if !UI_SaberValidForPlayerInMP(ctx, &saber) {
            trap::Cvar_Set(ctx.engine, "ui_saber", "kyle");
            saber = trap::Cvar_VariableStringBuffer(ctx.engine, "ui_saber", MAX_QPATH);
        }
    } else {
        saber = trap::Cvar_VariableStringBuffer(ctx.engine, "ui_saber2", MAX_QPATH);
        if !UI_SaberValidForPlayerInMP(ctx, &saber) {
            trap::Cvar_Set(ctx.engine, "ui_saber2", "kyle");
            saber = trap::Cvar_VariableStringBuffer(ctx.engine, "ui_saber2", MAX_QPATH);
        }
    }

    // read this from the sabers.cfg
    let saberTypeString = UI_SaberTypeForSaber(ctx, &saber).unwrap_or_default();
    let mut saberType = saberType_t::SABER_NONE;
    if !saberTypeString.is_empty() {
        saberType = TranslateSaberType(&saberTypeString);
    }

    match ctx.world.movesTitleIndex {
        0 => {
            // MD_ACROBATICS
        }
        1 | 2 | 3 => {
            // MD_SINGLE_FAST / MD_SINGLE_MEDIUM / MD_SINGLE_STRONG
            if saberType != saberType_t::SABER_SINGLE {
                saber = "single_1".to_string();
            }
        }
        4 => {
            // MD_DUAL_SABERS
            if saberType != saberType_t::SABER_SINGLE {
                saber = "single_1".to_string();
            }
        }
        5 => {
            // MD_SABER_STAFF
            if saberType == saberType_t::SABER_SINGLE || saberType == saberType_t::SABER_NONE {
                saber = "dual_1".to_string();
            }
        }
        _ => {}
    }

    saber
}

/// Raven `UI_SaberGetHiltInfo` — walks the cached `.sab` block text
/// (`world.saber.SaberParms`), sorting each MP-valid saber name into the
/// single-hilt or staff-hilt list by `UI_IsSaberTwoHanded`, each capped at
/// `MAX_SABER_HILTS - 1` entries.
///
/// PORT-NOTE: Raven's two `const char *singleHilts[MAX_SABER_HILTS]`/
/// `staffHilts[MAX_SABER_HILTS]` out-params (NULL-terminated) collapse into a
/// returned `(Vec<String>, Vec<String>)` (dictionary: out-param -> return;
/// the NULL terminator has no analog on an owned `Vec`). The digest's
/// `UiWorld`-only channel omits `ctx` because it does not name the trap seam,
/// but the overflow-warning path calls the already-ported `Com_Printf(ctx,
/// ..)`, so `ctx` threads through here too.
///
/// Source: `oracle/codemp/ui/ui_saber.c:1080-1143`
pub fn UI_SaberGetHiltInfo(ctx: &mut UiContext) -> (Vec<String>, Vec<String>) {
    let mut singleHilts: Vec<String> = Vec::new();
    let mut staffHilts: Vec<String> = Vec::new();

    // go through all the loaded sabers and put the valid ones in the proper list
    let saber_parms = mem::take(&mut ctx.world.saber.SaberParms);
    let mut p: Option<&[u8]> = Some(saber_parms.as_bytes());
    COM_BeginParseSession(&mut ctx.world.bg_state.qs, "saberlist");

    // look for a saber
    while p.is_some() {
        let (token, rest) = COM_ParseExt(&mut ctx.world.bg_state.qs, p, true);
        p = rest;
        if token.is_empty() {
            // invalid name
            continue;
        }
        let saberName = String_Alloc(Some(&token)).unwrap_or_default();
        // see if there's a "{" on the next line
        p = SkipRestOfLine(&mut ctx.world.bg_state.qs, p);

        if UI_ParseLiteralSilent(&mut p, "{") {
            // nope, not a name, keep looking
            continue;
        }

        // this is a saber name
        if !UI_SaberValidForPlayerInMP(ctx, &saberName) {
            p = SkipBracedSection(&mut ctx.world.bg_state.qs, p);
            continue;
        }

        if UI_IsSaberTwoHanded(ctx, &saberName) {
            // -1 because we have to NULL terminate the list
            if staffHilts.len() < MAX_SABER_HILTS - 1 {
                staffHilts.push(saberName);
            } else {
                Com_Printf(
                    ctx,
                    &format!(
                        "WARNING: too many two-handed sabers, ignoring saber '{}'\n",
                        saberName
                    ),
                );
            }
        } else {
            // -1 because we have to NULL terminate the list
            if singleHilts.len() < MAX_SABER_HILTS - 1 {
                singleHilts.push(saberName);
            } else {
                Com_Printf(
                    ctx,
                    &format!(
                        "WARNING: too many one-handed sabers, ignoring saber '{}'\n",
                        saberName
                    ),
                );
            }
        }
        // skip the whole braced section and move on to the next entry
        p = SkipBracedSection(&mut ctx.world.bg_state.qs, p);
    }
    ctx.world.saber.SaberParms = saber_parms;

    (singleHilts, staffHilts)
}

/// Raven `UI_SaberDrawBlades` — draws every blade of `item`'s hacked
/// sabermoves-character saber(s), or of the currently-equipped `ui_saber`/
/// `ui_saber2` (falling back to `"kyle"` when invalid in MP).
///
/// Raven: `NOTE: only allows one saber type in view at a time`.
///
/// PORT-NOTE: `char saber[MAX_QPATH]`/`saber[0]` truthiness collapses to a
/// `String`/`is_empty()` check (dictionary: fixed C string buffer -> owned
/// `String`).
///
/// Source: `oracle/codemp/ui/ui_saber.c:952-1017`
pub fn UI_SaberDrawBlades(ctx: &mut UiContext, item: &ItemDef, origin: vec3_t, angles: vec3_t) {
    let mut numSabers = 1;

    if (item.flags & ITF_ISCHARACTER) != 0 && ctx.world.movesTitleIndex == 4 {
        // MD_DUAL_SABERS
        numSabers = 2;
    }

    for saberNum in 0..numSabers {
        let mut saber: String;
        let saberModel: c_int;

        if (item.flags & ITF_ISCHARACTER) != 0 {
            // hacked sabermoves sabers in character's hand
            saber = UI_GetSaberForMenu(ctx, saberNum);
            saberModel = saberNum + 1;
        } else if (item.flags & ITF_ISSABER) != 0 {
            saber = trap::Cvar_VariableStringBuffer(ctx.engine, "ui_saber", MAX_QPATH);
            if !UI_SaberValidForPlayerInMP(ctx, &saber) {
                trap::Cvar_Set(ctx.engine, "ui_saber", "kyle");
                saber = trap::Cvar_VariableStringBuffer(ctx.engine, "ui_saber", MAX_QPATH);
            }
            saberModel = 0;
        } else if (item.flags & ITF_ISSABER2) != 0 {
            saber = trap::Cvar_VariableStringBuffer(ctx.engine, "ui_saber2", MAX_QPATH);
            if !UI_SaberValidForPlayerInMP(ctx, &saber) {
                trap::Cvar_Set(ctx.engine, "ui_saber2", "kyle");
                saber = trap::Cvar_VariableStringBuffer(ctx.engine, "ui_saber2", MAX_QPATH);
            }
            saberModel = 0;
        } else {
            return;
        }

        if !saber.is_empty() {
            let numBlades = UI_SaberNumBladesForSaber(ctx, &saber);
            if numBlades != 0 {
                // okay, here we go, time to draw each blade...
                let saberTypeString = UI_SaberTypeForSaber(ctx, &saber).unwrap_or_default();
                let saberType = TranslateSaberType(&saberTypeString);
                for curBlade in 0..numBlades {
                    if UI_SaberShouldDrawBlade(ctx, &saber, curBlade) {
                        UI_SaberDrawBlade(
                            ctx, item, &saber, saberModel, saberType, origin, angles, curBlade,
                        );
                    }
                }
            }
        }
    }
}

/// Raven `UI_SaberAttachToChar` — rebuilds `item`'s bolted-on saber ghoul2
/// model(s) (single, or two for `MD_DUAL_SABERS`) from the currently-equipped
/// saber(s), attaching each to the `*r_hand`/`*l_hand` bolt.
///
/// PORT-NOTE: `G2API_HasGhoul2ModelOnIndex`/`G2API_RemoveGhoul2Model`/
/// `G2API_InitGhoul2Model` take the ADDRESS of the `ghoul2` slot (the engine
/// derefs arg1 as `CGhoul2Info_v **` — `cl_ui.cpp:1341,1348`), while
/// `G2API_SetSkin`/`G2API_AddBolt`/`G2API_AttachG2Model` take the handle value.
///
/// Source: `oracle/codemp/ui/ui_saber.c:1019-1075`
pub fn UI_SaberAttachToChar(ctx: &mut UiContext, item: &mut ItemDef) {
    let mut numSabers = 1;

    let g2slot = &mut item.ghoul2 as *mut *mut c_void as *mut c_void;
    if trap::G2API_HasGhoul2ModelOnIndex(ctx.engine, g2slot, 2) {
        // remove any extra models
        trap::G2API_RemoveGhoul2Model(ctx.engine, g2slot, 2);
    }
    if trap::G2API_HasGhoul2ModelOnIndex(ctx.engine, g2slot, 1) {
        // remove any extra models
        trap::G2API_RemoveGhoul2Model(ctx.engine, g2slot, 1);
    }

    if ctx.world.movesTitleIndex == 4 {
        // MD_DUAL_SABERS
        numSabers = 2;
    }

    for saberNum in 0..numSabers {
        // bolt sabers
        let saber = UI_GetSaberForMenu(ctx, saberNum);

        if let Some(modelPath) = UI_SaberModelForSaber(ctx, &saber) {
            // successfully found a model
            let g2Saber = trap::G2API_InitGhoul2Model(
                ctx.engine,
                &mut item.ghoul2 as *mut *mut c_void,
                &modelPath,
                0,
                0,
                0,
                0,
                0,
            );
            if g2Saber != 0 {
                // get the customSkin, if any
                if let Some(skinPath) = UI_SaberSkinForSaber(ctx, &saber) {
                    let g2skin = trap::R_RegisterSkin(ctx.engine, &skinPath);
                    // this is going to set the surfs on/off matching the skin file
                    trap::G2API_SetSkin(ctx.engine, item.ghoul2, g2Saber, 0, g2skin);
                } else {
                    // turn off custom skin
                    trap::G2API_SetSkin(ctx.engine, item.ghoul2, g2Saber, 0, 0);
                }

                let boltNum = if saberNum == 0 {
                    trap::G2API_AddBolt(ctx.engine, item.ghoul2, 0, "*r_hand")
                } else {
                    trap::G2API_AddBolt(ctx.engine, item.ghoul2, 0, "*l_hand")
                };
                trap::G2API_AttachG2Model(
                    ctx.engine,
                    item.ghoul2,
                    g2Saber,
                    item.ghoul2,
                    boltNum,
                    0,
                );
            }
        }
    }
}
