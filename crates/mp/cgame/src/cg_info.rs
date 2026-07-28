//! Port of `oracle/codemp/cgame/cg_info.c` — the loading screen and connection info display. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;

use native_string::Q_strncpyz;

use mp_bg::public::bg_itemlist::bg_itemlist;
use mp_qshared::shared::{colorWhite, MAX_STRING_CHARS};

use crate::cg_drawtools::CG_DrawPic;
use crate::cg_main::CG_GetStringEdString;
use crate::trap;
use crate::world::cg_context::CgContext;

/// Raven `#define MAX_LOADING_PLAYER_ICONS 16`.
/// Source: `oracle/codemp/cgame/cg_info.c:7`
pub const MAX_LOADING_PLAYER_ICONS: usize = 16;

/// Raven `#define MAX_LOADING_ITEM_ICONS 26`.
/// Source: `oracle/codemp/cgame/cg_info.c:8`
pub const MAX_LOADING_ITEM_ICONS: usize = 26;

// DEFERRED: UI_INFOFONT — oracle/codemp/cgame/cg_info.c:109
// `#define UI_INFOFONT (UI_BIGFONT)` resolves to q_shared.h's `UI_BIGFONT`,
// whose numeric value is not in this packet's FILE-SCOPE CONSTANTS and has no
// existing Rust binding in mp_qshared/mp_uishared to alias; not needed by
// CG_LoadingString (the only fn this wave opens), so left unported rather
// than guessed.

/// Raven `CG_LoadingString` — copies the loading-screen status text into
/// `cg.infoScreenText` and forces the frame to draw immediately so the
/// player sees load progress instead of a frozen screen.
///
/// Source: `oracle/codemp/cgame/cg_info.c:21-25`
pub fn CG_LoadingString(ctx: &mut CgContext, s: &str) {
    Q_strncpyz(&mut ctx.world.cg.infoScreenText, s, MAX_STRING_CHARS);

    trap::UpdateScreen(ctx.engine);
}

/// Raven `CG_LoadingItem` — looks up `bg_itemlist[itemNum]`'s classname in the
/// StringEd table and posts it as the loading-screen status text.
///
/// Source: `oracle/codemp/cgame/cg_info.c:32-46`
pub fn CG_LoadingItem(ctx: &mut CgContext, itemNum: c_int) {
    let item = &bg_itemlist[itemNum as usize];

    if item.classname.is_empty() {
        // Raven's `CG_LoadingString( "Unknown item" )` fallback is commented
        // out in the oracle, so an unknown/sentinel item is a silent no-op.
        return;
    }

    let upperKey = item.classname.to_ascii_uppercase();
    let text = CG_GetStringEdString(ctx, "SP_INGAME", &upperKey);
    CG_LoadingString(ctx, &text);
}

/// Raven `CG_LoadBar` — draws the LCARS-style loading progress bar: a
/// surround frame, backwards left cap, filled tick bar sized by
/// `cg.loadLCARSStage`, and a right cap.
///
/// Source: `oracle/codemp/cgame/cg_info.c:439-460`
pub fn CG_LoadBar(ctx: &mut CgContext) {
    let numticks: c_int = 9;
    let tickwidth: c_int = 40;
    let tickheight: c_int = 8;
    let tickpadx: c_int = 20;
    let tickpady: c_int = 12;
    let capwidth: c_int = 8;
    let barwidth: c_int = numticks * tickwidth + tickpadx * 2 + capwidth * 2;
    let barleft: c_int = (640 - barwidth) / 2;
    let barheight: c_int = tickheight + tickpady * 2;
    let bartop: c_int = 480 - barheight;
    let capleft: c_int = barleft + tickpadx;
    let tickleft: c_int = capleft + capwidth;
    let ticktop: c_int = bartop + tickpady;

    let loadBarLEDSurround = ctx.world.cgs.media.loadBarLEDSurround;
    let loadBarLEDCap = ctx.world.cgs.media.loadBarLEDCap;
    let loadBarLED = ctx.world.cgs.media.loadBarLED;
    let loadLCARSStage = ctx.world.cg.loadLCARSStage;

    trap::R_SetColor(ctx.engine, Some(&colorWhite));

    // Draw background
    CG_DrawPic(
        ctx,
        barleft as f32,
        bartop as f32,
        barwidth as f32,
        barheight as f32,
        loadBarLEDSurround,
    );

    // Draw left cap (backwards)
    CG_DrawPic(
        ctx,
        tickleft as f32,
        ticktop as f32,
        -capwidth as f32,
        tickheight as f32,
        loadBarLEDCap,
    );

    // Draw bar
    CG_DrawPic(
        ctx,
        tickleft as f32,
        ticktop as f32,
        (tickwidth * loadLCARSStage) as f32,
        tickheight as f32,
        loadBarLED,
    );

    // Draw right cap
    CG_DrawPic(
        ctx,
        (tickleft + tickwidth * loadLCARSStage) as f32,
        ticktop as f32,
        capwidth as f32,
        tickheight as f32,
        loadBarLEDCap,
    );
}
