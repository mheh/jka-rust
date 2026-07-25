//! `ui_main.c` — the ui module's main logic (ownerdraws, feeders, menu
//! scripts, server browser).
//!
//! Source: `oracle/codemp/ui/ui_main.c`

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};

use mp_abi::ui::public::ui_client_state_t::uiClientState_t;
use mp_bg::bg_channel::BgState;
use mp_bg::public::configstring::{CS_PLAYERS, CS_SERVERINFO};
use mp_bg::public::gametype::{
    GT_CTF, GT_CTY, GT_DUEL, GT_FFA, GT_HOLOCRON, GT_JEDIMASTER, GT_POWERDUEL, GT_SIEGE,
    GT_SINGLE_PLAYER, GT_TEAM,
};
use mp_bg::public::team::{TEAM_BLUE, TEAM_RED, TEAM_SPECTATOR};
use mp_bg::saga::siege_class_t::siegeClass_t;
use mp_bg::weapons::weapon_t::{WP_NONE, WP_NUM_WEAPONS, WP_SABER};
use mp_qshared::common::mp::qcommon::saber::saber_colors::saber_colors_t;
use mp_qshared::shared::cbuf_exec::cbufExec_t;
use mp_qshared::shared::com_parse::{COM_BeginParseSession, COM_ParseExt, QSharedScratch};
use mp_qshared::shared::cvar::{
    vmCvar_t, CVAR_ARCHIVE, CVAR_INIT, CVAR_INTERNAL, CVAR_NORESTART, CVAR_ROM, CVAR_SERVERINFO,
    CVAR_TEMP,
};
use mp_qshared::shared::force_powers::{
    FP_LEVITATION, FP_SABER_DEFENSE, FP_SABER_OFFENSE, NUM_FORCE_POWERS,
};
use mp_qshared::shared::limits::MAX_NAME_LENGTH;
use mp_qshared::shared::q_color::S_COLOR_RED;
use mp_qshared::shared::q_string::COM_StripExtension;
use mp_qshared::shared::{
    connstate_t, fileHandle_t, qhandle_t, vec4_t, AS_FAVORITES, CIN_LOOP, CIN_SILENT, FS_READ,
    KEYCATCH_UI, MAX_CLIENTS, MAX_INFO_STRING, MAX_QPATH, MAX_STRING_CHARS, Q3_VERSION,
    SCREEN_HEIGHT, SCREEN_WIDTH,
};
use mp_uishared::shared::display_context::DisplayContext;
use mp_uishared::shared::menu_system::MAX_MENUFILE;
use mp_uishared::shared::menudef::{
    ITEM_TEXTSTYLE_BLINK, ITEM_TEXTSTYLE_NORMAL, ITEM_TEXTSTYLE_OUTLINED,
    ITEM_TEXTSTYLE_OUTLINESHADOWED, ITEM_TEXTSTYLE_PULSE, ITEM_TEXTSTYLE_SHADOWED,
    ITEM_TEXTSTYLE_SHADOWEDMORE, UI_CLANCINEMATIC, UI_MAPCINEMATIC, UI_NETMAPCINEMATIC,
    UI_SHOW_ANYNONTEAMGAME, UI_SHOW_ANYTEAMGAME, UI_SHOW_DEMOAVAILABLE, UI_SHOW_FAVORITESERVERS,
    UI_SHOW_FFA, UI_SHOW_LEADER, UI_SHOW_NETANYNONTEAMGAME, UI_SHOW_NETANYTEAMGAME,
    UI_SHOW_NEWBESTTIME, UI_SHOW_NEWHIGHSCORE, UI_SHOW_NOTFAVORITESERVERS, UI_SHOW_NOTFFA,
    UI_SHOW_NOTLEADER,
};
use mp_uishared::shared::rect_def_t::RectDef;
use mp_uishared::ui_shared::{
    Menu_FindItemByName, Menu_GetFocused, Menus_AnyFullScreenVisible, UI_CleanupGhoul2,
};
use native_string::{atoi, latin1_to_string, Info_ValueForKey, Q_CleanStr, Q_stricmp, Q_stricmpn};

use crate::keycodes::fake_ascii_t::fakeAscii_t;
use crate::local::pinglist_t::MAX_ADDRESSLENGTH;
use crate::local::player_species_info_t::{PlayerSpeciesInfo, MAX_PLAYERMODELS};
use crate::local::server_status_info_t::{
    ServerStatusInfo, MAX_SERVERSTATUS_LINES, MAX_SERVERSTATUS_TEXT,
};
use crate::local::tier_info::MAPS_PER_TIER;
use crate::trap;
use crate::ui_atoms::{Com_Printf, UI_Cvar_VariableString, UI_DrawHandlePic};
use crate::ui_gameinfo::UI_GetNumBots;
use crate::ui_saber::{SaberColorToString, TranslateSaberColor};
use crate::world::ui_context::UiContext;
use crate::world::ui_cvars::UiCvars;
use crate::world::ui_world::{UiWorld, MAX_FORCE_CONFIGS};

/// Raven `static const int numSkillLevels = sizeof(skillLevels) /
/// sizeof(const char*)` — `skillLevels[]` (`ui_main.c:902-908`) has 5 rows;
/// the table itself is compiled-in data that lands beside the fn that reads
/// it (PORT-NOTE, `UiMainState`), so only the derived count is needed here.
///
/// Source: `oracle/codemp/ui/ui_main.c:902-909`
const NUM_SKILL_LEVELS: c_int = 5;

/// Raven `qfiles.h` `STYLE_DROPSHADOW`/`STYLE_BLINK` font-render bits
/// (`Text_Paint`'s `iFontHandle` high bits). Already ported once as a
/// `mp_engine_qcommon` const (`qfiles/font_style.rs`), but `mp_ui` has no
/// dependency on that crate, so these stay file-local consts (same fidelity,
/// same values).
///
/// Source: `oracle/codemp/qcommon/qfiles.h:570-571`
const STYLE_DROPSHADOW: u32 = 0x8000_0000;
const STYLE_BLINK: u32 = 0x4000_0000;

/// Raven `#define MAX_Q3PLAYERMODELS 256`.
///
/// Source: `oracle/codemp/ui/ui_local.h:593`
const MAX_Q3PLAYERMODELS: usize = 256;

// DEFERRED: UI_AnimsetAlloc — part of the ui_main.c hand-maintained animation
// fork (`bgAllAnims`/`uiNumAllAnims`/`UI_ParseAnimationFile`); DEC-36 D5 rules
// ui reuses mp_bg's animation module instead of Raven's manually synced copy
// (see `UiMainState`'s PORT-NOTE, which drops the same fork's state fields).
// Source: `oracle/codemp/ui/ui_main.c:645-651`

// DEFERRED: UI_ParseAnimationFile — same hand-maintained animation fork as
// UI_AnimsetAlloc above; its state (`uiHumanoidAnimations`, `UIPAFtext`,
// `UIPAFtextLoaded`, `bgAllAnims`, `uiNumAllAnims`) was dropped at U2 per the
// same DEC-36 D5 ruling, so there is nothing left to thread this fn's body
// through.
// Source: `oracle/codemp/ui/ui_main.c:664-863`

/// Raven `GetCRDelineatedString`.
///
/// Raven kept the result in a function-scope `static char sTemp[256]`; the
/// idiomatic port returns the owned `String` (or `None` for Raven's `NULL`
/// out-of-range return) directly instead of reusing a shared buffer.
///
/// PORT-NOTE (§19): Raven `strcpy`s the line into `char sTemp[256]` (an overrun
/// for longer lines); the owned `String` returns it untruncated.
///
/// Source: `oracle/codemp/ui/ui_main.c:954-976`
pub fn GetCRDelineatedString(
    ctx: &mut UiContext,
    psStripFileRef: &str,
    psStripStringRef: &str,
    iIndex: c_int,
) -> Option<String> {
    let psList = UI_GetStringEdString(ctx, psStripFileRef, psStripStringRef);
    let mut rest = psList.as_str();

    // Raven's `while (iIndex--)` tests before the decrement, so a negative index
    // walks to the end of the list and falls out through the OOR return.
    let mut i = iIndex;
    while i != 0 {
        match rest.find('\n') {
            Some(pos) => rest = &rest[pos + 1..],
            None => return None, // OOR
        }
        i -= 1;
    }

    let sTemp = match rest.find('\n') {
        Some(pos) => &rest[..pos],
        None => rest,
    };

    Some(sTemp.to_string())
}

/// Raven `UI_TeamName`.
///
/// Source: `oracle/codemp/ui/ui_main.c:942-950`
pub fn UI_TeamName(team: c_int) -> &'static str {
    if team == TEAM_RED {
        "RED"
    } else if team == TEAM_BLUE {
        "BLUE"
    } else if team == TEAM_SPECTATOR {
        "SPECTATOR"
    } else {
        "FREE"
    }
}

/// Raven `AssetCache`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1009-1045`
pub fn AssetCache(ctx: &mut UiContext) {
    ctx.world.uiDC.Assets.gradientBar =
        trap::R_RegisterShaderNoMip(ctx.engine, "ui/assets/gradientbar2.tga");
    ctx.world.uiDC.Assets.fxBasePic = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/fx_base");
    ctx.world.uiDC.Assets.fxPic[0] = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/fx_red");
    ctx.world.uiDC.Assets.fxPic[1] = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/fx_orange");
    ctx.world.uiDC.Assets.fxPic[2] = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/fx_yel");
    ctx.world.uiDC.Assets.fxPic[3] = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/fx_grn");
    ctx.world.uiDC.Assets.fxPic[4] = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/fx_blue");
    ctx.world.uiDC.Assets.fxPic[5] = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/fx_purple");
    ctx.world.uiDC.Assets.fxPic[6] = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/fx_white");
    ctx.world.uiDC.Assets.scrollBar =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/scrollbar.tga");
    ctx.world.uiDC.Assets.scrollBarArrowDown =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/scrollbar_arrow_dwn_a.tga");
    ctx.world.uiDC.Assets.scrollBarArrowUp =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/scrollbar_arrow_up_a.tga");
    ctx.world.uiDC.Assets.scrollBarArrowLeft =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/scrollbar_arrow_left.tga");
    ctx.world.uiDC.Assets.scrollBarArrowRight =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/scrollbar_arrow_right.tga");
    ctx.world.uiDC.Assets.scrollBarThumb =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/scrollbar_thumb.tga");
    ctx.world.uiDC.Assets.sliderBar = trap::R_RegisterShaderNoMip(ctx.engine, "menu/new/slider");
    ctx.world.uiDC.Assets.sliderThumb =
        trap::R_RegisterShaderNoMip(ctx.engine, "menu/new/sliderthumb");

    // Icons for various server settings.
    ctx.world.uiDC.Assets.needPass = trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/needpass");
    ctx.world.uiDC.Assets.noForce = trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/noforce");
    ctx.world.uiDC.Assets.forceRestrict =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/forcerestrict");
    ctx.world.uiDC.Assets.saberOnly =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/saberonly");
    ctx.world.uiDC.Assets.trueJedi = trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/truejedi");

    for n in 0..ctx.world.uiDC.Assets.crosshairShader.len() {
        let letter = (b'a' + n as u8) as char;
        ctx.world.uiDC.Assets.crosshairShader[n] =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("gfx/2d/crosshair{}", letter));
    }

    // trap_S_RegisterSound("sound/feedback/voc_newhighscore.wav") — Raven left
    // this call commented out.
    ctx.world.newHighScoreSound = 0;
}

/// Raven `_UI_DrawSides`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1047-1051`
pub fn _UI_DrawSides(ctx: &mut UiContext, x: f32, y: f32, w: f32, h: f32, size: f32) {
    let size = size * ctx.world.uiDC.xscale;
    let white = ctx.world.uiDC.whiteShader;
    trap::R_DrawStretchPic(ctx.engine, x, y, size, h, 0.0, 0.0, 0.0, 0.0, white);
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
        white,
    );
}

/// Raven `_UI_DrawTopBottom`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1053-1057`
pub fn _UI_DrawTopBottom(ctx: &mut UiContext, x: f32, y: f32, w: f32, h: f32, size: f32) {
    let size = size * ctx.world.uiDC.yscale;
    let white = ctx.world.uiDC.whiteShader;
    trap::R_DrawStretchPic(ctx.engine, x, y, w, size, 0.0, 0.0, 0.0, 0.0, white);
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
        white,
    );
}

/// Raven `_UI_DrawRect`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1065-1072`
pub fn _UI_DrawRect(
    ctx: &mut UiContext,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    size: f32,
    color: &vec4_t,
) {
    trap::R_SetColor(ctx.engine, Some(color));

    _UI_DrawTopBottom(ctx, x, y, width, height, size);
    _UI_DrawSides(ctx, x, y, width, height, size);

    trap::R_SetColor(ctx.engine, None);
}

/// Raven `MenuFontToHandle`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1075-1086`
pub fn MenuFontToHandle(world: &UiWorld, iMenuFont: c_int) -> qhandle_t {
    match iMenuFont {
        1 => world.uiDC.Assets.qhSmallFont,
        2 => world.uiDC.Assets.qhMediumFont,
        3 => world.uiDC.Assets.qhBigFont,
        4 => world.uiDC.Assets.qhSmall2Font,
        _ => world.uiDC.Assets.qhMediumFont,
    }
}

/// Raven `Text_Width`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1089-1094`
pub fn Text_Width(ctx: &UiContext, text: &str, scale: f32, iMenuFont: c_int) -> c_int {
    let iFontIndex = MenuFontToHandle(ctx.world, iMenuFont);
    trap::R_Font_StrLenPixels(ctx.engine, text, iFontIndex, scale)
}

/// Raven `Text_Height`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1096-1101`
pub fn Text_Height(ctx: &UiContext, _text: &str, scale: f32, iMenuFont: c_int) -> c_int {
    let iFontIndex = MenuFontToHandle(ctx.world, iMenuFont);
    trap::R_Font_HeightPixels(ctx.engine, iFontIndex, scale)
}

/// Raven `Text_Paint`.
///
/// PORT-NOTE: the JK2-menu-style-to-SOF2-printstring-ctrl-code `switch`
/// (`ITEM_TEXTSTYLE_*` → `STYLE_BLINK`/`STYLE_DROPSHADOW`) is transcribed as a
/// `match`; both file-local const families are defined above (no canonical
/// qshared/qcommon home reachable from this crate).
///
/// Source: `oracle/codemp/ui/ui_main.c:1103-1130`
#[allow(clippy::too_many_arguments)]
pub fn Text_Paint(
    ctx: &UiContext,
    x: f32,
    y: f32,
    scale: f32,
    color: vec4_t,
    text: &str,
    _adjust: f32,
    limit: c_int,
    style: c_int,
    iMenuFont: c_int,
) {
    let iFontIndex = MenuFontToHandle(ctx.world, iMenuFont);
    // kludge.. convert JK2 menu styles to SOF2 printstring ctrl codes...
    let iStyleOR: c_int = match style {
        ITEM_TEXTSTYLE_NORMAL => 0,                           // JK2 normal text
        ITEM_TEXTSTYLE_BLINK => STYLE_BLINK as c_int,         // JK2 fast blinking
        ITEM_TEXTSTYLE_PULSE => STYLE_BLINK as c_int,         // JK2 slow pulsing
        ITEM_TEXTSTYLE_SHADOWED => STYLE_DROPSHADOW as c_int, // JK2 drop shadow
        ITEM_TEXTSTYLE_OUTLINED => STYLE_DROPSHADOW as c_int, // JK2 drop shadow
        ITEM_TEXTSTYLE_OUTLINESHADOWED => STYLE_DROPSHADOW as c_int, // JK2 drop shadow
        ITEM_TEXTSTYLE_SHADOWEDMORE => STYLE_DROPSHADOW as c_int, // JK2 drop shadow
        _ => 0,
    };

    trap::R_Font_DrawString(
        ctx.engine,
        x as c_int,                          // int ox
        y as c_int,                          // int oy
        text,                                // const char *text
        &color,                              // paletteRGBA_c c
        iStyleOR | iFontIndex,               // const int iFontHandle
        if limit == 0 { -1 } else { limit }, // iCharLimit (-1 = none)
        scale,                               // const float scale = 1.0f
    );
}

/// Raven `UI_GetStringEdString`.
///
/// Raven kept the result in a function-scope `static char text[1024]`; the
/// idiomatic port returns the owned `String` directly instead of reusing a
/// shared buffer.
///
/// Source: `oracle/codemp/ui/ui_main.c:1249-1255`
pub fn UI_GetStringEdString(ctx: &mut UiContext, refSection: &str, refName: &str) -> String {
    let key = format!("{}_{}", refSection, refName);
    trap::SP_GetStringTextString(ctx.engine, &key, 1024).unwrap_or_default()
}

/// Raven `GetMenuBuffer`.
///
/// Raven read into a function-scope `static char buf[MAX_MENUFILE]`; the port
/// reads into a local `Vec<u8>` sized to the file length and returns the
/// decoded `String` directly (each call fully repopulates the buffer before
/// use, so there is no cross-call state to preserve).
///
/// Source: `oracle/codemp/ui/ui_main.c:1439-1461`
pub fn GetMenuBuffer(ctx: &mut UiContext, filename: &str) -> String {
    let mut f: fileHandle_t = 0;
    let len = trap::FS_FOpenFile(ctx.engine, filename, &mut f, FS_READ);
    if f == 0 {
        trap::Print(
            ctx.engine,
            &format!(
                "{}menu file not found: {}, using default\n",
                S_COLOR_RED.to_str().unwrap(),
                filename
            ),
        );
        return ctx.world.main.defaultMenu.clone().unwrap_or_default();
    }
    if len >= MAX_MENUFILE as c_int {
        trap::Print(
            ctx.engine,
            &format!(
                "{}menu file too large: {} is {}, max allowed is {}",
                S_COLOR_RED.to_str().unwrap(),
                filename,
                len,
                MAX_MENUFILE
            ),
        );
        trap::FS_FCloseFile(ctx.engine, f);
        return ctx.world.main.defaultMenu.clone().unwrap_or_default();
    }

    let mut buf = vec![0u8; len as usize];
    trap::FS_Read(ctx.engine, &mut buf, f);
    trap::FS_FCloseFile(ctx.engine, f);
    // COM_Compress(buf) — Raven left this call commented out.
    latin1_to_string(&buf)
}

/// Raven `UI_DrawCenteredPic`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1229-1234`
pub fn UI_DrawCenteredPic(ctx: &mut UiContext, image: qhandle_t, w: c_int, h: c_int) {
    let x = (SCREEN_WIDTH - w) / 2;
    let y = (SCREEN_HEIGHT - h) / 2;
    UI_DrawHandlePic(ctx, x as f32, y as f32, w as f32, h as f32, image);
}

/// Raven `_UI_Shutdown`.
///
/// PORT-NOTE: `UI_CleanupGhoul2` calls through `DisplayContext` (its ported
/// shape takes `dc: &mut dyn DisplayContext`, DEC-36 addendum 12), so this fn
/// carries a `dc` parameter alongside `ctx` even though Raven's own body has
/// no `DC->` call (see escalations).
///
/// Source: `oracle/codemp/ui/ui_main.c:1432-1435`
pub fn _UI_Shutdown(ctx: &mut UiContext, dc: &mut dyn DisplayContext) {
    trap::LAN_SaveCachedServers(ctx.engine);
    UI_CleanupGhoul2(&mut ctx.world.menus, dc);
}

/// Raven `UI_SetCapFragLimits`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1911-1922`
pub fn UI_SetCapFragLimits(ctx: &mut UiContext, uiVars: bool) {
    let cap = 5;
    let frag = 10;

    if uiVars {
        trap::Cvar_Set(ctx.engine, "ui_captureLimit", &format!("{}", cap));
        trap::Cvar_Set(ctx.engine, "ui_fragLimit", &format!("{}", frag));
    } else {
        trap::Cvar_Set(ctx.engine, "capturelimit", &format!("{}", cap));
        trap::Cvar_Set(ctx.engine, "fraglimit", &format!("{}", frag));
    }
}

/// Raven `UI_GetGameTypeName`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1924-1950`
pub fn UI_GetGameTypeName(ctx: &mut UiContext, gtEnum: c_int) -> String {
    match gtEnum {
        GT_FFA => UI_GetStringEdString(ctx, "MENUS", "FREE_FOR_ALL"), //"Free For All";
        GT_HOLOCRON => UI_GetStringEdString(ctx, "MENUS", "HOLOCRON_FFA"), //"Holocron FFA";
        GT_JEDIMASTER => UI_GetStringEdString(ctx, "MENUS", "SAGA"),  //"Jedi Master";??
        GT_SINGLE_PLAYER => UI_GetStringEdString(ctx, "MENUS", "SAGA"), //"Team FFA";
        GT_DUEL => UI_GetStringEdString(ctx, "MENUS", "DUEL"),        //"Team FFA";
        GT_POWERDUEL => UI_GetStringEdString(ctx, "MENUS", "POWERDUEL"), //"Team FFA";
        GT_TEAM => UI_GetStringEdString(ctx, "MENUS", "TEAM_FFA"),    //"Team FFA";
        GT_SIEGE => UI_GetStringEdString(ctx, "MENUS", "SIEGE"),      //"Siege";
        GT_CTF => UI_GetStringEdString(ctx, "MENUS", "CAPTURE_THE_FLAG"), //"Capture the Flag";
        GT_CTY => UI_GetStringEdString(ctx, "MENUS", "CAPTURE_THE_YSALIMARI"), //"Capture the Ysalamiri";
        _ => UI_GetStringEdString(ctx, "MENUS", "SAGA"),                       //"Team FFA";
    }
}

/// Raven `UI_TeamIndexFromName`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2010-2023`
pub fn UI_TeamIndexFromName(world: &UiWorld, name: &str) -> c_int {
    if !name.is_empty() {
        for (i, team) in world.teamList.iter().enumerate() {
            if Q_stricmp(name, &team.teamName) == 0 {
                return i as c_int;
            }
        }
    }

    0
}

/// Raven `UI_DrawClanLogo`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2025-2040`
pub fn UI_DrawClanLogo(ctx: &mut UiContext, rect: &RectDef, _scale: f32, color: vec4_t) {
    let teamName = UI_Cvar_VariableString(ctx, "ui_teamName");
    let i = UI_TeamIndexFromName(ctx.world, &teamName);
    if i >= 0 && (i as usize) < ctx.world.teamList.len() {
        trap::R_SetColor(ctx.engine, Some(&color));

        if ctx.world.teamList[i as usize].teamIcon == -1 {
            let imageName = ctx.world.teamList[i as usize].imageName.clone();
            ctx.world.teamList[i as usize].teamIcon =
                trap::R_RegisterShaderNoMip(ctx.engine, &imageName);
            ctx.world.teamList[i as usize].teamIcon_Metal =
                trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_metal", imageName));
            ctx.world.teamList[i as usize].teamIcon_Name =
                trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_name", imageName));
        }

        let icon = ctx.world.teamList[i as usize].teamIcon;
        UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, icon);
        trap::R_SetColor(ctx.engine, None);
    }
}

/// Raven `UI_DrawClanCinematic`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2042-2068`
pub fn UI_DrawClanCinematic(ctx: &mut UiContext, rect: &RectDef, _scale: f32, color: vec4_t) {
    let teamName = UI_Cvar_VariableString(ctx, "ui_teamName");
    let i = UI_TeamIndexFromName(ctx.world, &teamName);
    if i >= 0 && (i as usize) < ctx.world.teamList.len() {
        let idx = i as usize;

        if ctx.world.teamList[idx].cinematic >= -2 {
            if ctx.world.teamList[idx].cinematic == -1 {
                let imageName = ctx.world.teamList[idx].imageName.clone();
                ctx.world.teamList[idx].cinematic = trap::CIN_PlayCinematic(
                    ctx.engine,
                    &format!("{}.roq", imageName),
                    0,
                    0,
                    0,
                    0,
                    CIN_LOOP | CIN_SILENT,
                );
            }
            if ctx.world.teamList[idx].cinematic >= 0 {
                let cinematic = ctx.world.teamList[idx].cinematic;
                trap::CIN_RunCinematic(ctx.engine, cinematic);
                trap::CIN_SetExtents(
                    ctx.engine,
                    cinematic,
                    rect.x as c_int,
                    rect.y as c_int,
                    rect.w as c_int,
                    rect.h as c_int,
                );
                trap::CIN_DrawCinematic(ctx.engine, cinematic);
            } else {
                trap::R_SetColor(ctx.engine, Some(&color));
                let icon = ctx.world.teamList[idx].teamIcon_Metal;
                UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, icon);
                trap::R_SetColor(ctx.engine, None);
                ctx.world.teamList[idx].cinematic = -2;
            }
        } else {
            trap::R_SetColor(ctx.engine, Some(&color));
            let icon = ctx.world.teamList[idx].teamIcon;
            UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, icon);
            trap::R_SetColor(ctx.engine, None);
        }
    }
}

/// Raven `UI_DrawPreviewCinematic`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2070-2082`
pub fn UI_DrawPreviewCinematic(ctx: &mut UiContext, rect: &RectDef, _scale: f32, _color: vec4_t) {
    if ctx.world.previewMovie > -2 {
        let movie = ctx.world.movieList[ctx.world.movieIndex as usize].clone();
        ctx.world.previewMovie = trap::CIN_PlayCinematic(
            ctx.engine,
            &format!("{}.roq", movie),
            0,
            0,
            0,
            0,
            CIN_LOOP | CIN_SILENT,
        );
        if ctx.world.previewMovie >= 0 {
            trap::CIN_RunCinematic(ctx.engine, ctx.world.previewMovie);
            trap::CIN_SetExtents(
                ctx.engine,
                ctx.world.previewMovie,
                rect.x as c_int,
                rect.y as c_int,
                rect.w as c_int,
                rect.h as c_int,
            );
            trap::CIN_DrawCinematic(ctx.engine, ctx.world.previewMovie);
        } else {
            ctx.world.previewMovie = -2;
        }
    }
}

/// Raven `UI_HasSetSaberOnly`.
///
/// PORT-NOTE (§19 UB pick): Raven reads `info` through `Info_ValueForKey`
/// before `trap_GetConfigString` fills it — an uninitialized-stack read
/// (`ui_main.c:2234-2239`). This port treats the pre-fill buffer as empty,
/// matching a zeroed C automatic (`Info_ValueForKey("", ...)` returns `""`,
/// so `atoi` yields 0).
///
/// Source: `oracle/codemp/ui/ui_main.c:2232-2269`
pub fn UI_HasSetSaberOnly(ctx: &mut UiContext) -> bool {
    let empty = String::new();
    let gametype = atoi(&Info_ValueForKey(&empty, "g_gametype"));

    if gametype == GT_JEDIMASTER {
        return false;
    }

    let info =
        trap::GetConfigString(ctx.engine, CS_SERVERINFO, MAX_INFO_STRING).unwrap_or_default();

    let wDisable = if gametype == GT_DUEL || gametype == GT_POWERDUEL {
        atoi(&Info_ValueForKey(&info, "g_duelWeaponDisable"))
    } else {
        atoi(&Info_ValueForKey(&info, "g_weaponDisable"))
    };

    let mut i = 0;
    while i < WP_NUM_WEAPONS {
        if (wDisable & (1 << i)) == 0 && i != WP_SABER && i != WP_NONE {
            return false;
        }
        i += 1;
    }

    true
}

/// Raven `UI_AllForceDisabled`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2271-2289`
pub fn UI_AllForceDisabled(force: c_int) -> bool {
    if force != 0 {
        for i in 0..NUM_FORCE_POWERS {
            if force & (1 << i) == 0 {
                return false;
            }
        }
        return true;
    }

    false
}

/// Raven `UI_TrueJediEnabled`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2291-2319`
pub fn UI_TrueJediEnabled(ctx: &mut UiContext) -> bool {
    let info =
        trap::GetConfigString(ctx.engine, CS_SERVERINFO, MAX_INFO_STRING).unwrap_or_default();

    // already have serverinfo at this point for stuff below. Don't bother
    // trying to use ui_forcePowerDisable.
    let disabledForce = atoi(&Info_ValueForKey(&info, "g_forcePowerDisable"));
    let allForceDisabled = UI_AllForceDisabled(disabledForce);
    let gametype = atoi(&Info_ValueForKey(&info, "g_gametype"));
    let saberOnly = UI_HasSetSaberOnly(ctx);

    let trueJedi =
        if gametype == GT_HOLOCRON || gametype == GT_JEDIMASTER || saberOnly || allForceDisabled {
            0
        } else {
            atoi(&Info_ValueForKey(&info, "g_jediVmerc"))
        };

    trueJedi != 0
}

/// Raven `UI_SetForceDisabled`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2502-2547`
pub fn UI_SetForceDisabled(world: &mut UiWorld, force: c_int) {
    if force != 0 {
        let mut i = 0;
        while i < NUM_FORCE_POWERS {
            if force & (1 << i) != 0 {
                world.force.uiForcePowersDisabled[i as usize] = true;

                if i != FP_LEVITATION && i != FP_SABER_OFFENSE && i != FP_SABER_DEFENSE {
                    world.force.uiForcePowersRank[i as usize] = 0;
                } else if i == FP_LEVITATION {
                    world.force.uiForcePowersRank[i as usize] = 1;
                } else {
                    world.force.uiForcePowersRank[i as usize] = 3;
                }
            } else {
                world.force.uiForcePowersDisabled[i as usize] = false;
            }
            i += 1;
        }
    } else {
        let mut i = 0;
        while i < NUM_FORCE_POWERS {
            world.force.uiForcePowersDisabled[i as usize] = false;
            i += 1;
        }
    }
}

/// Raven `UI_DrawEffects`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2425-2428`
pub fn UI_DrawEffects(ctx: &mut UiContext, rect: &RectDef, _scale: f32, _color: vec4_t) {
    // PORT-NOTE (§19): Raven read a stale fixed-array slot here; guarded to a
    // skipped draw (there is no fallback shader).
    let idx = ctx.world.effectsColor;
    if idx < 0 || idx as usize >= ctx.world.force.uiSaberColorShaders.len() {
        return;
    }
    let shader = ctx.world.force.uiSaberColorShaders[idx as usize];
    UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, shader);
}

/// Raven `UI_DrawMapPreview`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2430-2452`
pub fn UI_DrawMapPreview(
    ctx: &mut UiContext,
    rect: &RectDef,
    _scale: f32,
    _color: vec4_t,
    net: bool,
) {
    let mut map = if net {
        ctx.world.cvars.ui_currentNetMap.integer
    } else {
        ctx.world.cvars.ui_currentMap.integer
    };
    if map < 0 || map > ctx.world.mapList.len() as c_int {
        if net {
            ctx.world.cvars.ui_currentNetMap.integer = 0;
            trap::Cvar_Set(ctx.engine, "ui_currentNetMap", "0");
        } else {
            ctx.world.cvars.ui_currentMap.integer = 0;
            trap::Cvar_Set(ctx.engine, "ui_currentMap", "0");
        }
        map = 0;
    }

    let idx = map as usize;
    // PORT-NOTE (§19): `map == mapCount` clears Raven's guard and reads a stale
    // fixed-array slot (levelShot 0); guarded to the unknown-map fallback below.
    if idx >= ctx.world.mapList.len() {
        let shader = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/unknownmap_mp");
        UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, shader);
        return;
    }
    if ctx.world.mapList[idx].levelShot == -1 {
        let imageName = ctx.world.mapList[idx].imageName.clone();
        ctx.world.mapList[idx].levelShot = trap::R_RegisterShaderNoMip(ctx.engine, &imageName);
    }

    if ctx.world.mapList[idx].levelShot > 0 {
        let shot = ctx.world.mapList[idx].levelShot;
        UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, shot);
    } else {
        let shader = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/unknownmap_mp");
        UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, shader);
    }
}

/// Raven `UI_DrawNetMapPreview`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2739-2746`
pub fn UI_DrawNetMapPreview(ctx: &mut UiContext, rect: &RectDef, _scale: f32, _color: vec4_t) {
    if ctx.world.serverStatus.currentServerPreview > 0 {
        let preview = ctx.world.serverStatus.currentServerPreview;
        UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, preview);
    } else {
        let shader = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/unknownmap_mp");
        UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, shader);
    }
}

/// Raven `UI_DrawTierMap`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2791-2803`
pub fn UI_DrawTierMap(ctx: &mut UiContext, rect: &RectDef, index: c_int) {
    let mut i = trap::Cvar_VariableValue(ctx.engine, "ui_currentTier") as c_int;
    if i < 0 || i as usize >= ctx.world.tierList.len() {
        i = 0;
    }

    let tierIdx = i as usize;
    let mapIdx = index as usize;
    // PORT-NOTE (§19): Raven read a stale fixed-array slot here; guarded to a
    // skipped draw (there is no fallback shader).
    if tierIdx >= ctx.world.tierList.len() || mapIdx >= MAPS_PER_TIER {
        return;
    }
    if ctx.world.tierList[tierIdx].mapHandles[mapIdx] == -1 {
        let mapName = ctx.world.tierList[tierIdx].maps[mapIdx].clone();
        ctx.world.tierList[tierIdx].mapHandles[mapIdx] =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("levelshots/{}", mapName));
    }

    let handle = ctx.world.tierList[tierIdx].mapHandles[mapIdx];
    UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, handle);
}

/// Raven `UI_EnglishMapName`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2805-2813`
pub fn UI_EnglishMapName(world: &UiWorld, map: &str) -> String {
    for m in world.mapList.iter() {
        if Q_stricmp(map, &m.mapLoadName) == 0 {
            return m.mapName.clone();
        }
    }
    String::new()
}

/// Raven `UI_AIFromName`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2844-2852`
pub fn UI_AIFromName(world: &UiWorld, name: &str) -> String {
    for alias in world.aliasList.iter() {
        if Q_stricmp(&alias.name, name) == 0 {
            return alias.ai.clone();
        }
    }
    "Kyle".to_string()
}

/// Raven `UI_NextOpponent`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2886-2900`
pub fn UI_NextOpponent(ctx: &mut UiContext) {
    let opponentName = UI_Cvar_VariableString(ctx, "ui_opponentName");
    let mut i = UI_TeamIndexFromName(ctx.world, &opponentName);
    let teamName = UI_Cvar_VariableString(ctx, "ui_teamName");
    let j = UI_TeamIndexFromName(ctx.world, &teamName);

    i += 1;
    if i >= ctx.world.teamList.len() as c_int {
        i = 0;
    }
    if i == j {
        i += 1;
        if i >= ctx.world.teamList.len() as c_int {
            i = 0;
        }
    }
    let name = ctx.world.teamList[i as usize].teamName.clone();
    trap::Cvar_Set(ctx.engine, "ui_opponentName", &name);
}

/// Raven `UI_PriorOpponent`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2902-2916`
pub fn UI_PriorOpponent(ctx: &mut UiContext) {
    let opponentName = UI_Cvar_VariableString(ctx, "ui_opponentName");
    let mut i = UI_TeamIndexFromName(ctx.world, &opponentName);
    let teamName = UI_Cvar_VariableString(ctx, "ui_teamName");
    let j = UI_TeamIndexFromName(ctx.world, &teamName);

    i -= 1;
    if i < 0 {
        i = ctx.world.teamList.len() as c_int - 1;
    }
    if i == j {
        i -= 1;
        if i < 0 {
            i = ctx.world.teamList.len() as c_int - 1;
        }
    }
    let name = ctx.world.teamList[i as usize].teamName.clone();
    trap::Cvar_Set(ctx.engine, "ui_opponentName", &name);
}

/// Raven `UI_DrawPlayerLogo`.
///
/// PORT-NOTE: Raven's param is `vec3_t`, but `UI_OwnerDraw` hands it a real
/// `vec4_t` and `trap_R_SetColor` reads all four floats; the port takes
/// `vec4_t` and passes it through unchanged. Same for
/// `UI_DrawPlayerLogoMetal`/`Name` and the `UI_DrawOpponentLogo*` family below.
///
/// Source: `oracle/codemp/ui/ui_main.c:2918-2930`
pub fn UI_DrawPlayerLogo(ctx: &mut UiContext, rect: &RectDef, color: vec4_t) {
    let teamName = UI_Cvar_VariableString(ctx, "ui_teamName");
    let i = UI_TeamIndexFromName(ctx.world, &teamName) as usize;
    // PORT-NOTE (§19): Raven read a stale fixed-array slot here when no team is
    // loaded; guarded to a skipped draw (there is no fallback icon).
    if i >= ctx.world.teamList.len() {
        return;
    }

    if ctx.world.teamList[i].teamIcon == -1 {
        let imageName = ctx.world.teamList[i].imageName.clone();
        ctx.world.teamList[i].teamIcon = trap::R_RegisterShaderNoMip(ctx.engine, &imageName);
        ctx.world.teamList[i].teamIcon_Metal =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_metal", imageName));
        ctx.world.teamList[i].teamIcon_Name =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_name", imageName));
    }

    trap::R_SetColor(ctx.engine, Some(&color));
    let icon = ctx.world.teamList[i].teamIcon;
    UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, icon);
    trap::R_SetColor(ctx.engine, None);
}

/// Raven `UI_DrawPlayerLogoMetal`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2932-2943`
pub fn UI_DrawPlayerLogoMetal(ctx: &mut UiContext, rect: &RectDef, color: vec4_t) {
    let teamName = UI_Cvar_VariableString(ctx, "ui_teamName");
    let i = UI_TeamIndexFromName(ctx.world, &teamName) as usize;
    // PORT-NOTE (§19): Raven read a stale fixed-array slot here when no team is
    // loaded; guarded to a skipped draw (there is no fallback icon).
    if i >= ctx.world.teamList.len() {
        return;
    }

    if ctx.world.teamList[i].teamIcon == -1 {
        let imageName = ctx.world.teamList[i].imageName.clone();
        ctx.world.teamList[i].teamIcon = trap::R_RegisterShaderNoMip(ctx.engine, &imageName);
        ctx.world.teamList[i].teamIcon_Metal =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_metal", imageName));
        ctx.world.teamList[i].teamIcon_Name =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_name", imageName));
    }

    trap::R_SetColor(ctx.engine, Some(&color));
    let icon = ctx.world.teamList[i].teamIcon_Metal;
    UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, icon);
    trap::R_SetColor(ctx.engine, None);
}

/// Raven `UI_DrawPlayerLogoName`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2945-2956`
pub fn UI_DrawPlayerLogoName(ctx: &mut UiContext, rect: &RectDef, color: vec4_t) {
    let teamName = UI_Cvar_VariableString(ctx, "ui_teamName");
    let i = UI_TeamIndexFromName(ctx.world, &teamName) as usize;
    // PORT-NOTE (§19): Raven read a stale fixed-array slot here when no team is
    // loaded; guarded to a skipped draw (there is no fallback icon).
    if i >= ctx.world.teamList.len() {
        return;
    }

    if ctx.world.teamList[i].teamIcon == -1 {
        let imageName = ctx.world.teamList[i].imageName.clone();
        ctx.world.teamList[i].teamIcon = trap::R_RegisterShaderNoMip(ctx.engine, &imageName);
        ctx.world.teamList[i].teamIcon_Metal =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_metal", imageName));
        ctx.world.teamList[i].teamIcon_Name =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_name", imageName));
    }

    trap::R_SetColor(ctx.engine, Some(&color));
    let icon = ctx.world.teamList[i].teamIcon_Name;
    UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, icon);
    trap::R_SetColor(ctx.engine, None);
}

/// Raven `UI_DrawOpponentLogo`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2958-2969`
pub fn UI_DrawOpponentLogo(ctx: &mut UiContext, rect: &RectDef, color: vec4_t) {
    let opponentName = UI_Cvar_VariableString(ctx, "ui_opponentName");
    let i = UI_TeamIndexFromName(ctx.world, &opponentName) as usize;
    // PORT-NOTE (§19): Raven read a stale fixed-array slot here when no team is
    // loaded; guarded to a skipped draw (there is no fallback icon).
    if i >= ctx.world.teamList.len() {
        return;
    }

    if ctx.world.teamList[i].teamIcon == -1 {
        let imageName = ctx.world.teamList[i].imageName.clone();
        ctx.world.teamList[i].teamIcon = trap::R_RegisterShaderNoMip(ctx.engine, &imageName);
        ctx.world.teamList[i].teamIcon_Metal =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_metal", imageName));
        ctx.world.teamList[i].teamIcon_Name =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_name", imageName));
    }

    trap::R_SetColor(ctx.engine, Some(&color));
    let icon = ctx.world.teamList[i].teamIcon;
    UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, icon);
    trap::R_SetColor(ctx.engine, None);
}

/// Raven `UI_DrawOpponentLogoMetal`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2971-2982`
pub fn UI_DrawOpponentLogoMetal(ctx: &mut UiContext, rect: &RectDef, color: vec4_t) {
    let opponentName = UI_Cvar_VariableString(ctx, "ui_opponentName");
    let i = UI_TeamIndexFromName(ctx.world, &opponentName) as usize;
    // PORT-NOTE (§19): Raven read a stale fixed-array slot here when no team is
    // loaded; guarded to a skipped draw (there is no fallback icon).
    if i >= ctx.world.teamList.len() {
        return;
    }

    if ctx.world.teamList[i].teamIcon == -1 {
        let imageName = ctx.world.teamList[i].imageName.clone();
        ctx.world.teamList[i].teamIcon = trap::R_RegisterShaderNoMip(ctx.engine, &imageName);
        ctx.world.teamList[i].teamIcon_Metal =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_metal", imageName));
        ctx.world.teamList[i].teamIcon_Name =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_name", imageName));
    }

    trap::R_SetColor(ctx.engine, Some(&color));
    let icon = ctx.world.teamList[i].teamIcon_Metal;
    UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, icon);
    trap::R_SetColor(ctx.engine, None);
}

/// Raven `UI_DrawOpponentLogoName`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2984-2995`
pub fn UI_DrawOpponentLogoName(ctx: &mut UiContext, rect: &RectDef, color: vec4_t) {
    let opponentName = UI_Cvar_VariableString(ctx, "ui_opponentName");
    let i = UI_TeamIndexFromName(ctx.world, &opponentName) as usize;
    // PORT-NOTE (§19): Raven read a stale fixed-array slot here when no team is
    // loaded; guarded to a skipped draw (there is no fallback icon).
    if i >= ctx.world.teamList.len() {
        return;
    }

    if ctx.world.teamList[i].teamIcon == -1 {
        let imageName = ctx.world.teamList[i].imageName.clone();
        ctx.world.teamList[i].teamIcon = trap::R_RegisterShaderNoMip(ctx.engine, &imageName);
        ctx.world.teamList[i].teamIcon_Metal =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_metal", imageName));
        ctx.world.teamList[i].teamIcon_Name =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_name", imageName));
    }

    trap::R_SetColor(ctx.engine, Some(&color));
    let icon = ctx.world.teamList[i].teamIcon_Name;
    UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, icon);
    trap::R_SetColor(ctx.engine, None);
}

/// Raven `UI_BuildPlayerList`.
///
/// Source: `oracle/codemp/ui/ui_main.c:3276-3336`
pub fn UI_BuildPlayerList(ctx: &mut UiContext) {
    let mut cs = uiClientState_t {
        connState: connstate_t::CA_UNINITIALIZED,
        connectPacketCount: 0,
        clientNum: 0,
        servername: [0; MAX_STRING_CHARS],
        updateInfoString: [0; MAX_STRING_CHARS],
        messageString: [0; MAX_STRING_CHARS],
    };
    trap::GetClientState(ctx.engine, &mut cs);

    let info = trap::GetConfigString(ctx.engine, CS_PLAYERS + cs.clientNum, MAX_INFO_STRING)
        .unwrap_or_default();
    ctx.world.playerNumber = cs.clientNum;
    ctx.world.teamLeader = atoi(&Info_ValueForKey(&info, "tl")) != 0;
    let team = atoi(&Info_ValueForKey(&info, "t"));

    let info =
        trap::GetConfigString(ctx.engine, CS_SERVERINFO, MAX_INFO_STRING).unwrap_or_default();
    let count = atoi(&Info_ValueForKey(&info, "sv_maxclients"));

    let mut playerNames: Vec<String> = Vec::new();
    let mut playerIndexes: Vec<c_int> = Vec::new();
    let mut teamNames: Vec<String> = Vec::new();
    let mut teamClientNums: Vec<c_int> = Vec::new();
    let mut playerTeamNumber: c_int = 0;

    let mut n = 0;
    while n < count {
        let info =
            trap::GetConfigString(ctx.engine, CS_PLAYERS + n, MAX_INFO_STRING).unwrap_or_default();

        if !info.is_empty() {
            // Raven `Q_strncpyz(..., MAX_NAME_LENGTH)` truncates the raw name
            // to MAX_NAME_LENGTH-1 bytes BEFORE Q_CleanStr; Latin-1 decoding
            // is one char per byte, so a char-truncate is byte-faithful.
            let raw_name = |info: &str| -> String {
                Info_ValueForKey(info, "n")
                    .chars()
                    .take(MAX_NAME_LENGTH - 1)
                    .collect()
            };
            playerNames.push(Q_CleanStr(&raw_name(&info)));
            playerIndexes.push(n);
            let team2 = atoi(&Info_ValueForKey(&info, "t"));
            if team2 == team && n != ctx.world.playerNumber {
                teamNames.push(Q_CleanStr(&raw_name(&info)));
                teamClientNums.push(n);
                if ctx.world.playerNumber == n {
                    playerTeamNumber = (teamNames.len() - 1) as c_int;
                }
            }
        }
        n += 1;
    }

    ctx.world.playerNames = playerNames;
    ctx.world.playerIndexes = playerIndexes;
    ctx.world.teamNames = teamNames;
    ctx.world.teamClientNums = teamClientNums;

    if !ctx.world.teamLeader {
        trap::Cvar_Set(
            ctx.engine,
            "cg_selectedPlayer",
            &format!("{}", playerTeamNumber),
        );
    }

    let mut n = trap::Cvar_VariableValue(ctx.engine, "cg_selectedPlayer") as c_int;
    if n < 0 || n > ctx.world.teamNames.len() as c_int {
        n = 0;
    }

    if n < ctx.world.teamNames.len() as c_int {
        let name = ctx.world.teamNames[n as usize].clone();
        trap::Cvar_Set(ctx.engine, "cg_selectedPlayerName", &name);
    } else {
        trap::Cvar_Set(ctx.engine, "cg_selectedPlayerName", "Everyone");
    }

    if team == 0 || team == TEAM_SPECTATOR || !ctx.world.teamLeader {
        let n = ctx.world.teamNames.len() as c_int;
        trap::Cvar_Set(ctx.engine, "cg_selectedPlayer", &format!("{}", n));
        trap::Cvar_Set(ctx.engine, "cg_selectedPlayerName", "N/A");
    }
}

/// Raven `UI_Version`.
///
/// PORT-NOTE: `uiDC.textWidth`/`uiDC.drawText` are `DisplayContext` trait
/// methods (DEC-36 D3); the trait is imported here, but `impl DisplayContext
/// for UiContext` has not landed yet (see escalations).
///
/// Source: `oracle/codemp/ui/ui_main.c:3494-3501`
pub fn UI_Version(
    dc: &mut dyn DisplayContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    iMenuFont: c_int,
) {
    let width = dc.textWidth(Q3_VERSION, scale, iMenuFont);
    dc.drawText(
        rect.x - width as f32,
        rect.y,
        scale,
        color,
        Q3_VERSION,
        0.0,
        0,
        0,
        iMenuFont,
    );
}

/// Raven `UI_OwnerDrawVisible`.
///
/// Source: `oracle/codemp/ui/ui_main.c:3781-3891`
pub fn UI_OwnerDrawVisible(ctx: &mut UiContext, flags: c_int) -> bool {
    let mut vis = true;
    let mut flags = flags;

    while flags != 0 {
        if flags & UI_SHOW_FFA != 0 {
            let gt = trap::Cvar_VariableValue(ctx.engine, "g_gametype");
            if gt != GT_FFA as f32 && gt != GT_HOLOCRON as f32 && gt != GT_JEDIMASTER as f32 {
                vis = false;
            }
            flags &= !UI_SHOW_FFA;
        }

        if flags & UI_SHOW_NOTFFA != 0 {
            let gt = trap::Cvar_VariableValue(ctx.engine, "g_gametype");
            if gt == GT_FFA as f32 || gt == GT_HOLOCRON as f32 || gt != GT_JEDIMASTER as f32 {
                vis = false;
            }
            flags &= !UI_SHOW_NOTFFA;
        }

        if flags & UI_SHOW_LEADER != 0 {
            // these need to show when this client can give orders to a player or a group
            if !ctx.world.teamLeader {
                vis = false;
            } else {
                // if showing yourself
                let sel = ctx.world.cvars.ui_selectedPlayer.integer;
                if (sel as usize) < ctx.world.teamClientNums.len()
                    && ctx.world.teamClientNums[sel as usize] == ctx.world.playerNumber
                {
                    vis = false;
                }
            }
            flags &= !UI_SHOW_LEADER;
        }

        if flags & UI_SHOW_NOTLEADER != 0 {
            // these need to show when this client is assigning their own status or they are NOT the leader
            if ctx.world.teamLeader {
                // if not showing yourself
                let sel = ctx.world.cvars.ui_selectedPlayer.integer;
                let showing_self = (sel as usize) < ctx.world.teamClientNums.len()
                    && ctx.world.teamClientNums[sel as usize] == ctx.world.playerNumber;
                if !showing_self {
                    vis = false;
                }
            }
            flags &= !UI_SHOW_NOTLEADER;
        }

        if flags & UI_SHOW_FAVORITESERVERS != 0 {
            // this assumes you only put this type of display flag on something showing in the proper context
            if ctx.world.cvars.ui_netSource.integer != AS_FAVORITES {
                vis = false;
            }
            flags &= !UI_SHOW_FAVORITESERVERS;
        }

        if flags & UI_SHOW_NOTFAVORITESERVERS != 0 {
            // this assumes you only put this type of display flag on something showing in the proper context
            if ctx.world.cvars.ui_netSource.integer == AS_FAVORITES {
                vis = false;
            }
            flags &= !UI_SHOW_NOTFAVORITESERVERS;
        }

        if flags & UI_SHOW_ANYTEAMGAME != 0 {
            let idx = ctx.world.cvars.ui_gameType.integer as usize;
            if ctx.world.gameTypes[idx].gtEnum <= GT_TEAM {
                vis = false;
            }
            flags &= !UI_SHOW_ANYTEAMGAME;
        }

        if flags & UI_SHOW_ANYNONTEAMGAME != 0 {
            let idx = ctx.world.cvars.ui_gameType.integer as usize;
            if ctx.world.gameTypes[idx].gtEnum > GT_TEAM {
                vis = false;
            }
            flags &= !UI_SHOW_ANYNONTEAMGAME;
        }

        if flags & UI_SHOW_NETANYTEAMGAME != 0 {
            let idx = ctx.world.cvars.ui_netGameType.integer as usize;
            if ctx.world.gameTypes[idx].gtEnum <= GT_TEAM {
                vis = false;
            }
            flags &= !UI_SHOW_NETANYTEAMGAME;
        }

        if flags & UI_SHOW_NETANYNONTEAMGAME != 0 {
            let idx = ctx.world.cvars.ui_netGameType.integer as usize;
            if ctx.world.gameTypes[idx].gtEnum > GT_TEAM {
                vis = false;
            }
            flags &= !UI_SHOW_NETANYNONTEAMGAME;
        }

        if flags & UI_SHOW_NEWHIGHSCORE != 0 {
            if ctx.world.newHighScoreTime < ctx.world.uiDC.realTime {
                vis = false;
            } else if ctx.world.soundHighScore
                && trap::Cvar_VariableValue(ctx.engine, "sv_killserver") == 0.0
            {
                // wait on server to go down before playing sound
                // trap_S_StartLocalSound(uiInfo.newHighScoreSound, CHAN_ANNOUNCER);
                ctx.world.soundHighScore = false;
            }
            flags &= !UI_SHOW_NEWHIGHSCORE;
        }

        if flags & UI_SHOW_NEWBESTTIME != 0 {
            if ctx.world.newBestTime < ctx.world.uiDC.realTime {
                vis = false;
            }
            flags &= !UI_SHOW_NEWBESTTIME;
        }

        if flags & UI_SHOW_DEMOAVAILABLE != 0 {
            if !ctx.world.demoAvailable {
                vis = false;
            }
            flags &= !UI_SHOW_DEMOAVAILABLE;
        } else {
            flags = 0;
        }
    }

    vis
}

/// Raven `UI_Handicap_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:3893-3911`
pub fn UI_Handicap_HandleKey(
    ctx: &mut UiContext,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        let mut h = trap::Cvar_VariableValue(ctx.engine, "handicap").clamp(5.0, 100.0) as c_int;
        if key == fakeAscii_t::A_MOUSE2 as c_int {
            h -= 5;
        } else {
            h += 5;
        }
        if h > 100 {
            h = 5;
        } else if h < 0 {
            h = 100;
        }
        trap::Cvar_Set(ctx.engine, "handicap", &format!("{}", h));
        return true;
    }
    false
}

/// Raven `UI_AutoSwitch_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4377-4400`
pub fn UI_AutoSwitch_HandleKey(
    ctx: &mut UiContext,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        let mut switchVal = trap::Cvar_VariableValue(ctx.engine, "cg_autoswitch") as c_int;

        if key == fakeAscii_t::A_MOUSE2 as c_int {
            switchVal -= 1;
        } else {
            switchVal += 1;
        }

        if switchVal < 0 {
            switchVal = 2;
        } else if switchVal >= 3 {
            switchVal = 0;
        }

        trap::Cvar_Set(ctx.engine, "cg_autoswitch", &format!("{}", switchVal));
        return true;
    }
    false
}

/// Raven `UI_Skill_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4426-4446`
pub fn UI_Skill_HandleKey(
    ctx: &mut UiContext,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        let mut i = trap::Cvar_VariableValue(ctx.engine, "g_spSkill") as c_int;

        if key == fakeAscii_t::A_MOUSE2 as c_int {
            i -= 1;
        } else {
            i += 1;
        }

        if i < 1 {
            i = NUM_SKILL_LEVELS;
        } else if i > NUM_SKILL_LEVELS {
            i = 1;
        }

        trap::Cvar_Set(ctx.engine, "g_spSkill", &format!("{}", i));
        return true;
    }
    false
}

/// Raven `UI_BotSkill_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4615-4630`
pub fn UI_BotSkill_HandleKey(
    world: &mut UiWorld,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        if key == fakeAscii_t::A_MOUSE2 as c_int {
            world.skillIndex -= 1;
        } else {
            world.skillIndex += 1;
        }
        if world.skillIndex >= NUM_SKILL_LEVELS {
            world.skillIndex = 0;
        } else if world.skillIndex < 0 {
            world.skillIndex = NUM_SKILL_LEVELS - 1;
        }
        return true;
    }
    false
}

/// Raven `UI_RedBlue_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4632-4638`
pub fn UI_RedBlue_HandleKey(
    world: &mut UiWorld,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        world.redBlue ^= 1;
        return true;
    }
    false
}

/// Raven `#define NUM_CROSSHAIRS 9`. No canonical qshared home ported yet, so
/// this stays a file-local const (same treatment as `AS_FAVORITES`/`CIN_LOOP`
/// above).
///
/// Source: `oracle/codemp/ui/ui_shared.h:104`
const NUM_CROSSHAIRS: c_int = 9;

/// Raven `UI_DrawCrosshair`.
///
/// Source: `oracle/codemp/ui/ui_main.c:3262-3269`
pub fn UI_DrawCrosshair(ctx: &mut UiContext, rect: &RectDef, _scale: f32, color: vec4_t) {
    trap::R_SetColor(ctx.engine, Some(&color));
    if ctx.world.currentCrosshair < 0 || ctx.world.currentCrosshair >= NUM_CROSSHAIRS {
        ctx.world.currentCrosshair = 0;
    }
    let shader = ctx.world.uiDC.Assets.crosshairShader[ctx.world.currentCrosshair as usize];
    UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, shader);
    trap::R_SetColor(ctx.engine, None);
}

/// Raven `UI_Crosshair_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4640-4657`
pub fn UI_Crosshair_HandleKey(
    ctx: &mut UiContext,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        if key == fakeAscii_t::A_MOUSE2 as c_int {
            ctx.world.currentCrosshair -= 1;
        } else {
            ctx.world.currentCrosshair += 1;
        }

        if ctx.world.currentCrosshair >= NUM_CROSSHAIRS {
            ctx.world.currentCrosshair = 0;
        } else if ctx.world.currentCrosshair < 0 {
            ctx.world.currentCrosshair = NUM_CROSSHAIRS - 1;
        }
        trap::Cvar_Set(
            ctx.engine,
            "cg_drawCrosshair",
            &format!("{}", ctx.world.currentCrosshair),
        );
        return true;
    }
    false
}

/// Raven `UI_InSoloMenu`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4300-4320`
pub fn UI_InSoloMenu(world: &UiWorld) -> bool {
    // Get current menu (either video or ingame video, I would assume)
    let menu = Menu_GetFocused(&world.menus);

    if menu.is_none() {
        return false;
    }

    Menu_FindItemByName(&world.menus, menu, "solo_gametypefield").is_some()
}

/// Raven `UI_TeamName_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4449-4471`
pub fn UI_TeamName_HandleKey(
    ctx: &mut UiContext,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
    blue: bool,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        let cvarName = if blue { "ui_blueTeam" } else { "ui_redTeam" };
        let current = UI_Cvar_VariableString(ctx, cvarName);
        let mut i = UI_TeamIndexFromName(ctx.world, &current);

        if key == fakeAscii_t::A_MOUSE2 as c_int {
            i -= 1;
        } else {
            i += 1;
        }

        if i >= ctx.world.teamList.len() as c_int {
            i = 0;
        } else if i < 0 {
            i = ctx.world.teamList.len() as c_int - 1;
        }

        let name = ctx.world.teamList[i as usize].teamName.clone();
        trap::Cvar_Set(ctx.engine, cvarName, &name);

        return true;
    }
    false
}

/// Raven `UI_TeamMember_HandleKey`.
///
/// Raven's comment: 0 - None, 1 - Human, 2..NumCharacters - Bot.
///
/// Source: `oracle/codemp/ui/ui_main.c:4473-4524`
pub fn UI_TeamMember_HandleKey(
    ctx: &mut UiContext,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
    blue: bool,
    num: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        let cvar = if blue {
            format!("ui_blueteam{}", num)
        } else {
            format!("ui_redteam{}", num)
        };
        let mut value = trap::Cvar_VariableValue(ctx.engine, &cvar) as c_int;
        let maxcl = trap::Cvar_VariableValue(ctx.engine, "sv_maxClients") as c_int;
        let mut numval = num;

        numval *= 2;
        if blue {
            numval -= 1;
        }

        if numval > maxcl {
            return false;
        }

        if value < 1 {
            value = 1;
        }

        if key == fakeAscii_t::A_MOUSE2 as c_int {
            value -= 1;
        } else {
            value += 1;
        }

        if value >= UI_GetNumBots(ctx.world) + 2 {
            value = 1;
        } else if value < 1 {
            value = UI_GetNumBots(ctx.world) + 2 - 1;
        }

        trap::Cvar_Set(ctx.engine, &cvar, &format!("{}", value));
        return true;
    }
    false
}

/// Raven `UI_BotName_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4583-4613`
pub fn UI_BotName_HandleKey(
    world: &mut UiWorld,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        let mut value = world.botIndex;

        if key == fakeAscii_t::A_MOUSE2 as c_int {
            value -= 1;
        } else {
            value += 1;
        }

        if value >= UI_GetNumBots(world) {
            value = 0;
        } else if value < 0 {
            value = UI_GetNumBots(world) - 1;
        }
        world.botIndex = value;
        return true;
    }
    false
}

/// Raven `UI_SelectedPlayer_HandleKey`.
///
/// Raven's own body never returns `qtrue` from inside the key-match block —
/// it always falls through to the trailing `return qfalse`, transcribed
/// faithfully (porting-rules §2: no speculative behavior fix).
///
/// Source: `oracle/codemp/ui/ui_main.c:4661-4691`
pub fn UI_SelectedPlayer_HandleKey(
    ctx: &mut UiContext,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        UI_BuildPlayerList(ctx);
        if !ctx.world.teamLeader {
            return false;
        }

        let mut selected = trap::Cvar_VariableValue(ctx.engine, "cg_selectedPlayer") as c_int;

        if key == fakeAscii_t::A_MOUSE2 as c_int {
            selected -= 1;
        } else {
            selected += 1;
        }

        // Raven `uiInfo.myTeamCount` — folded into `teamNames.len()` (§B3).
        let myTeamCount = ctx.world.teamNames.len() as c_int;
        if selected > myTeamCount {
            selected = 0;
        } else if selected < 0 {
            selected = myTeamCount;
        }

        if selected == myTeamCount {
            trap::Cvar_Set(ctx.engine, "cg_selectedPlayerName", "Everyone");
        } else {
            let name = ctx.world.teamNames[selected as usize].clone();
            trap::Cvar_Set(ctx.engine, "cg_selectedPlayerName", &name);
        }
        trap::Cvar_Set(ctx.engine, "cg_selectedPlayer", &format!("{}", selected));
    }
    false
}

/// Raven `UI_GetValue`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4963-4965`
pub fn UI_GetValue(_ownerDraw: c_int) -> f32 {
    0.0
}

/// Raven `UI_ServersQsortCompare`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4972-4974`
pub fn UI_ServersQsortCompare(ctx: &UiContext, arg1: c_int, arg2: c_int) -> c_int {
    trap::LAN_CompareServers(
        ctx.engine,
        ctx.world.cvars.ui_netSource.integer,
        ctx.world.serverStatus.sortKey,
        ctx.world.serverStatus.sortDir,
        arg1,
        arg2,
    )
}

/// Raven `UI_ServersSort`.
///
/// PORT-NOTE: Raven sorts `displayServers` in place with `qsort` over
/// `UI_ServersQsortCompare`; the port uses `Vec::sort_by` with the same
/// trap-backed comparator (`UI_ServersQsortCompare`'s body inlined here — a
/// closure cannot re-borrow `ctx` while `displayServers` is already borrowed
/// mutably), preserving behavior without pulling in `native_sort`'s
/// libc-`qsort`-shaped API (DEC-34's canonical-qsort ruling covers
/// gameplay-visible determinism, not this UI list order).
///
/// Source: `oracle/codemp/ui/ui_main.c:4982-4992`
pub fn UI_ServersSort(ctx: &mut UiContext, column: c_int, force: bool) {
    if !force && ctx.world.serverStatus.sortKey == column {
        return;
    }

    ctx.world.serverStatus.sortKey = column;

    let engine = ctx.engine;
    let sortKey = ctx.world.serverStatus.sortKey;
    let sortDir = ctx.world.serverStatus.sortDir;
    let source = ctx.world.cvars.ui_netSource.integer;
    ctx.world
        .serverStatus
        .displayServers
        .sort_by(|a, b| trap::LAN_CompareServers(engine, source, sortKey, sortDir, *a, *b).cmp(&0));
}

/// Raven `UI_Update`.
///
/// Source: `oracle/codemp/ui/ui_main.c:5255-5406`
pub fn UI_Update(ctx: &mut UiContext, name: &str) {
    let val = trap::Cvar_VariableValue(ctx.engine, name) as c_int;

    if Q_stricmp(name, "s_khz") == 0 {
        trap::Cmd_ExecuteText(
            ctx.engine,
            cbufExec_t::EXEC_APPEND as c_int,
            "snd_restart\n",
        );
        return;
    }

    if Q_stricmp(name, "ui_SetName") == 0 {
        let uiName = UI_Cvar_VariableString(ctx, "ui_Name");
        trap::Cvar_Set(ctx.engine, "name", &uiName);
    } else if Q_stricmp(name, "ui_setRate") == 0 {
        let rate = trap::Cvar_VariableValue(ctx.engine, "rate");
        if rate >= 5000.0 {
            trap::Cvar_Set(ctx.engine, "cl_maxpackets", "30");
            trap::Cvar_Set(ctx.engine, "cl_packetdup", "1");
        } else if rate >= 4000.0 {
            trap::Cvar_Set(ctx.engine, "cl_maxpackets", "15");
            // favor less prediction errors when there's packet loss
            trap::Cvar_Set(ctx.engine, "cl_packetdup", "2");
        } else {
            trap::Cvar_Set(ctx.engine, "cl_maxpackets", "15");
            // favor lower bandwidth
            trap::Cvar_Set(ctx.engine, "cl_packetdup", "1");
        }
    } else if Q_stricmp(name, "ui_GetName") == 0 {
        let clName = UI_Cvar_VariableString(ctx, "name");
        trap::Cvar_Set(ctx.engine, "ui_Name", &clName);
    } else if Q_stricmp(name, "ui_r_colorbits") == 0 {
        match val {
            0 => trap::Cvar_SetValue(ctx.engine, "ui_r_depthbits", 0.0),
            16 => trap::Cvar_SetValue(ctx.engine, "ui_r_depthbits", 16.0),
            32 => trap::Cvar_SetValue(ctx.engine, "ui_r_depthbits", 24.0),
            _ => {}
        }
    } else if Q_stricmp(name, "ui_r_lodbias") == 0 {
        match val {
            0 => trap::Cvar_SetValue(ctx.engine, "ui_r_subdivisions", 4.0),
            1 => trap::Cvar_SetValue(ctx.engine, "ui_r_subdivisions", 12.0),
            2 => trap::Cvar_SetValue(ctx.engine, "ui_r_subdivisions", 20.0),
            _ => {}
        }
    } else if Q_stricmp(name, "ui_r_glCustom") == 0 {
        match val {
            0 => {
                // high quality
                trap::Cvar_SetValue(ctx.engine, "ui_r_fullScreen", 1.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_subdivisions", 4.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_lodbias", 0.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_colorbits", 32.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_depthbits", 24.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_picmip", 0.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_mode", 4.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_texturebits", 32.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_fastSky", 0.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_inGameVideo", 1.0);
                trap::Cvar_Set(ctx.engine, "ui_r_texturemode", "GL_LINEAR_MIPMAP_LINEAR");
            }
            1 => {
                // normal
                trap::Cvar_SetValue(ctx.engine, "ui_r_fullScreen", 1.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_subdivisions", 4.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_lodbias", 0.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_colorbits", 0.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_depthbits", 24.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_picmip", 1.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_mode", 3.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_texturebits", 0.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_fastSky", 0.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_inGameVideo", 1.0);
                trap::Cvar_Set(ctx.engine, "ui_r_texturemode", "GL_LINEAR_MIPMAP_LINEAR");
            }
            2 => {
                // fast
                trap::Cvar_SetValue(ctx.engine, "ui_r_fullScreen", 1.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_subdivisions", 12.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_lodbias", 1.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_colorbits", 0.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_depthbits", 0.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_picmip", 2.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_mode", 3.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_texturebits", 0.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_fastSky", 1.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_inGameVideo", 0.0);
                trap::Cvar_Set(ctx.engine, "ui_r_texturemode", "GL_LINEAR_MIPMAP_NEAREST");
            }
            3 => {
                // fastest
                trap::Cvar_SetValue(ctx.engine, "ui_r_fullScreen", 1.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_subdivisions", 20.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_lodbias", 2.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_colorbits", 16.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_depthbits", 16.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_mode", 3.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_picmip", 3.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_texturebits", 16.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_fastSky", 1.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_inGameVideo", 0.0);
                trap::Cvar_Set(ctx.engine, "ui_r_texturemode", "GL_LINEAR_MIPMAP_NEAREST");
            }
            _ => {}
        }
    } else if Q_stricmp(name, "ui_mousePitch") == 0 {
        if val == 0 {
            trap::Cvar_SetValue(ctx.engine, "m_pitch", 0.022);
        } else {
            trap::Cvar_SetValue(ctx.engine, "m_pitch", -0.022);
        }
    } else if Q_stricmp(name, "ui_mousePitchVeh") == 0 {
        if val == 0 {
            trap::Cvar_SetValue(ctx.engine, "m_pitchVeh", 0.022);
        } else {
            trap::Cvar_SetValue(ctx.engine, "m_pitchVeh", -0.022);
        }
    }
}

/// Raven `UI_UpdateSaberType`.
///
/// Source: `oracle/codemp/ui/ui_main.c:5951-5961`
pub fn UI_UpdateSaberType(ctx: &mut UiContext) {
    let sType = trap::Cvar_VariableStringBuffer(ctx.engine, "ui_saber_type", MAX_QPATH);

    if Q_stricmp("single", &sType) == 0 || Q_stricmp("staff", &sType) == 0 {
        trap::Cvar_Set(ctx.engine, "ui_saber2", "");
    }
}

/// Raven `UI_UpdateSaberColor`.
///
/// Source: `oracle/codemp/ui/ui_main.c:6020-6022`
pub fn UI_UpdateSaberColor(_secondSaber: bool) {}

/// Raven `UI_GetTeamColor`.
///
/// Source: `oracle/codemp/ui/ui_main.c:7509-7510`
pub fn UI_GetTeamColor(_color: &mut vec4_t) {}

/// Raven `UI_ClampMaxPlayers`.
///
/// Source: `oracle/codemp/ui/ui_main.c:7529-7557`
pub fn UI_ClampMaxPlayers(ctx: &mut UiContext) {
    let idx = ctx.world.cvars.ui_netGameType.integer as usize;

    if ctx.world.gameTypes[idx].gtEnum == GT_DUEL {
        if trap::Cvar_VariableValue(ctx.engine, "sv_maxClients") < 2.0 {
            trap::Cvar_Set(ctx.engine, "sv_maxClients", "2");
        }
    } else if ctx.world.gameTypes[idx].gtEnum == GT_POWERDUEL {
        if trap::Cvar_VariableValue(ctx.engine, "sv_maxClients") < 3.0 {
            trap::Cvar_Set(ctx.engine, "sv_maxClients", "3");
        }
    }

    // max check for all game types
    if trap::Cvar_VariableValue(ctx.engine, "sv_maxClients") > MAX_CLIENTS as f32 {
        trap::Cvar_Set(ctx.engine, "sv_maxClients", &format!("{}", MAX_CLIENTS));
    }
}

/// Raven `UI_MapCountByGameType`.
///
/// Source: `oracle/codemp/ui/ui_main.c:7599-7626`
pub fn UI_MapCountByGameType(world: &mut UiWorld, singlePlayer: bool) -> c_int {
    let mut c = 0;
    let mut game = if singlePlayer {
        world.gameTypes[world.cvars.ui_gameType.integer as usize].gtEnum
    } else {
        world.gameTypes[world.cvars.ui_netGameType.integer as usize].gtEnum
    };
    if game == GT_SINGLE_PLAYER {
        game += 1;
    }
    if game == GT_TEAM {
        game = GT_FFA;
    }
    if game == GT_HOLOCRON || game == GT_JEDIMASTER {
        game = GT_FFA;
    }

    for i in 0..world.mapList.len() {
        world.mapList[i].active = false;
        if world.mapList[i].typeBits & (1 << game) != 0 {
            if singlePlayer && (world.mapList[i].typeBits & (1 << GT_SINGLE_PLAYER)) == 0 {
                continue;
            }
            c += 1;
            world.mapList[i].active = true;
        }
    }
    c
}

/// Raven `UI_hasSkinForBase`.
///
/// Source: `oracle/codemp/ui/ui_main.c:7628-7645`
pub fn UI_hasSkinForBase(ctx: &mut UiContext, base: &str, team: &str) -> bool {
    let mut f: fileHandle_t = 0;

    let test = format!("models/players/{}/{}/lower_default.skin", base, team);
    trap::FS_FOpenFile(ctx.engine, &test, &mut f, FS_READ);
    if f != 0 {
        trap::FS_FCloseFile(ctx.engine, f);
        return true;
    }

    let test = format!(
        "models/players/characters/{}/{}/lower_default.skin",
        base, team
    );
    trap::FS_FOpenFile(ctx.engine, &test, &mut f, FS_READ);
    if f != 0 {
        trap::FS_FCloseFile(ctx.engine, f);
        return true;
    }
    false
}

/// Raven `UI_HeadCountByColor`.
///
/// Source: `oracle/codemp/ui/ui_main.c:7652-7679`
pub fn UI_HeadCountByColor(world: &UiWorld) -> c_int {
    let mut c = 0;
    let teamname = match world.main.uiSkinColor {
        TEAM_BLUE => "/blue",
        TEAM_RED => "/red",
        _ => "/default",
    };

    // Count each head with this color.
    for name in world.q3HeadNames.iter() {
        if name.contains(teamname) {
            c += 1;
        }
    }
    c
}

/// Raven `UI_InsertServerIntoDisplayList`.
///
/// PORT-NOTE: Raven's manual shift-right loop becomes `Vec::insert`
/// (identical resulting order, idiomatic shape — porting-rules §10).
///
/// Source: `oracle/codemp/ui/ui_main.c:7686-7698`
pub fn UI_InsertServerIntoDisplayList(world: &mut UiWorld, num: c_int, position: c_int) {
    if position < 0 || position as usize > world.serverStatus.displayServers.len() {
        return;
    }
    world
        .serverStatus
        .displayServers
        .insert(position as usize, num);
}

/// Raven `UI_RemoveServerFromDisplayList`.
///
/// PORT-NOTE: Raven's manual shift-left loop becomes `Vec::remove` (identical
/// resulting order, idiomatic shape — porting-rules §10).
///
/// Source: `oracle/codemp/ui/ui_main.c:7705-7717`
pub fn UI_RemoveServerFromDisplayList(world: &mut UiWorld, num: c_int) {
    if let Some(i) = world
        .serverStatus
        .displayServers
        .iter()
        .position(|&n| n == num)
    {
        world.serverStatus.displayServers.remove(i);
    }
}

/// Raven `serverStatusCvar_t` — one row of the server-status name/altName
/// rename table. Internal-only (never crosses the ABI seam), so it takes the
/// idiomatic Rust shape.
///
/// Source: `oracle/codemp/ui/ui_main.c:7878-7882`
struct ServerStatusCvar {
    name: &'static str,
    altName: &'static str,
}

/// Raven `serverStatusCvar_t serverStatusCvars[]` — compiled-in data, so it
/// lands beside the function that reads it (§C8). Raven's `{NULL, NULL}`
/// terminator row is kept as the empty-name row the loop breaks on.
///
/// Source: `oracle/codemp/ui/ui_main.c:7884-7895`
const SERVER_STATUS_CVARS: [ServerStatusCvar; 10] = [
    ServerStatusCvar {
        name: "sv_hostname",
        altName: "Name",
    },
    ServerStatusCvar {
        name: "Address",
        altName: "",
    },
    ServerStatusCvar {
        name: "gamename",
        altName: "Game name",
    },
    ServerStatusCvar {
        name: "g_gametype",
        altName: "Game type",
    },
    ServerStatusCvar {
        name: "mapname",
        altName: "Map",
    },
    ServerStatusCvar {
        name: "version",
        altName: "",
    },
    ServerStatusCvar {
        name: "protocol",
        altName: "",
    },
    ServerStatusCvar {
        name: "timelimit",
        altName: "",
    },
    ServerStatusCvar {
        name: "fraglimit",
        altName: "",
    },
    ServerStatusCvar {
        name: "",
        altName: "",
    },
];

/// Raven `UI_SortServerStatusInfo`.
///
/// PORT-NOTE: only columns 0 and 3 are swapped per Raven's original (columns
/// 1/2 are left alone), kept literal here.
///
/// Source: `oracle/codemp/ui/ui_main.c:7901-7930`
pub fn UI_SortServerStatusInfo(info: &mut ServerStatusInfo) {
    // FIXME: if "gamename" == "base" or "missionpack" then
    // replace the gametype number by FFA, CTF etc.
    let mut index = 0usize;
    for cv in SERVER_STATUS_CVARS.iter() {
        if cv.name.is_empty() {
            break;
        }
        for j in 0..info.lines.len() {
            if !info.lines[j][1].is_empty() {
                continue;
            }
            if Q_stricmp(cv.name, &info.lines[j][0]) == 0 {
                // swap lines
                let tmp1 = info.lines[index][0].clone();
                let tmp2 = info.lines[index][3].clone();
                info.lines[index][0] = info.lines[j][0].clone();
                info.lines[index][3] = info.lines[j][3].clone();
                info.lines[j][0] = tmp1;
                info.lines[j][3] = tmp2;

                if !cv.altName.is_empty() {
                    info.lines[index][0] = cv.altName.to_string();
                }
                index += 1;
            }
        }
    }
}

/// Raven `UI_JoinServer`.
///
/// PORT-NOTE: the `_XBOX` live-server/system-link branch is dead PC surface
/// (porting-rules §20) and is dropped.
///
/// Source: `oracle/codemp/ui/ui_main.c:7984-8008`
pub fn UI_JoinServer(ctx: &mut UiContext) {
    trap::Cvar_Set(ctx.engine, "cg_thirdPerson", "0");
    trap::Cvar_Set(ctx.engine, "cg_cameraOrbit", "0");
    trap::Cvar_Set(ctx.engine, "ui_singlePlayerActive", "0");

    if ctx.world.serverStatus.currentServer >= 0
        && ctx.world.serverStatus.currentServer
            < ctx.world.serverStatus.displayServers.len() as c_int
    {
        let num =
            ctx.world.serverStatus.displayServers[ctx.world.serverStatus.currentServer as usize];
        let source = ctx.world.cvars.ui_netSource.integer;
        let buff = trap::LAN_GetServerAddressString(ctx.engine, source, num, 1024);
        trap::Cmd_ExecuteText(
            ctx.engine,
            cbufExec_t::EXEC_APPEND as c_int,
            &format!("connect {}\n", buff),
        );
    }
}

/// Raven `UI_CheckServerName`.
///
/// Source: `oracle/codemp/ui/ui_main.c:8017-8040`
pub fn UI_CheckServerName(ctx: &mut UiContext) {
    let hostname =
        trap::Cvar_VariableStringBuffer(ctx.engine, "sv_hostname", MAX_INFO_STRING as usize);

    let mut changed = false;
    let cleaned: String = hostname
        .chars()
        .map(|c| {
            if c == '\\' || c == ';' || c == '"' {
                changed = true;
                '.'
            } else {
                c
            }
        })
        .collect();

    if changed {
        trap::Cvar_Set(ctx.engine, "sv_hostname", &cleaned);
    }
}

/// Raven `stristr`.
///
/// Source: `oracle/codemp/ui/ui_main.c:8146-8157`
pub fn stristr<'a>(s: &'a str, charset: &str) -> Option<&'a str> {
    let s_bytes = s.as_bytes();
    let charset_bytes = charset.as_bytes();

    for start in 0..s_bytes.len() {
        let mut i = 0;
        while i < charset_bytes.len() && start + i < s_bytes.len() {
            if charset_bytes[i].to_ascii_uppercase() != s_bytes[start + i].to_ascii_uppercase() {
                break;
            }
            i += 1;
        }
        if i == charset_bytes.len() {
            return Some(&s[start..]);
        }
    }
    None
}

/// Raven `UI_SiegeClassNum`.
///
/// PORT-NOTE: `bgNumSiegeClasses`/`bgSiegeClasses` live on ui's own
/// [`BgState`] (`world.bg_state`, DEC-36 addendum 11 — Raven's ui link unit
/// compiled `bg_saga.c` itself). `ptr::eq` keeps Raven's
/// `&bgSiegeClasses[i] == scl` pointer-identity check.
///
/// Source: `oracle/codemp/ui/ui_main.c:8342-8356`
pub fn UI_SiegeClassNum(bg: &BgState, scl: &siegeClass_t) -> c_int {
    let mut i: c_int = 0;
    while i < bg.bgNumSiegeClasses {
        if core::ptr::eq(&bg.bgSiegeClasses[i as usize], scl) {
            return i;
        }
        i += 1;
    }
    0
}

/// Raven `UI_SelectedMap`.
///
/// Source: `oracle/codemp/ui/ui_main.c:8696-8712`
pub fn UI_SelectedMap(world: &UiWorld, index: c_int, actual: &mut c_int) -> String {
    let mut c = 0;
    *actual = 0;

    for i in 0..world.mapList.len() {
        if world.mapList[i].active {
            if c == index {
                *actual = i as c_int;
                return world.mapList[i].mapName.clone();
            } else {
                c += 1;
            }
        }
    }
    String::new()
}

/// Raven `UI_SelectedTeamHead`.
///
/// Source: `oracle/codemp/ui/ui_main.c:8719-8754`
pub fn UI_SelectedTeamHead(world: &UiWorld, index: c_int, actual: &mut c_int) -> String {
    let teamname = match world.main.uiSkinColor {
        TEAM_BLUE => "/blue",
        TEAM_RED => "/red",
        _ => "/default",
    };

    // Count each head with this color.
    let mut c = 0;
    for i in 0..world.q3HeadNames.len() {
        if world.q3HeadNames[i].contains(teamname) {
            if c == index {
                *actual = i as c_int;
                return world.q3HeadNames[i].clone();
            } else {
                c += 1;
            }
        }
    }
    String::new()
}

/// Raven `UI_GetIndexFromSelection`.
///
/// Source: `oracle/codemp/ui/ui_main.c:8757-8769`
pub fn UI_GetIndexFromSelection(world: &UiWorld, actual: c_int) -> c_int {
    let mut c = 0;
    for i in 0..world.mapList.len() {
        if world.mapList[i].active {
            if i as c_int == actual {
                return c;
            }
            c += 1;
        }
    }
    0
}

/// Raven `UI_UpdatePendingPings`.
///
/// Source: `oracle/codemp/ui/ui_main.c:8771-8778`
pub fn UI_UpdatePendingPings(ctx: &mut UiContext) {
    let source = ctx.world.cvars.ui_netSource.integer;
    trap::LAN_ResetPings(ctx.engine, source);
    ctx.world.serverStatus.refreshActive = true;
    ctx.world.serverStatus.refreshtime = ctx.world.uiDC.realTime + 1000;
}

/// Raven `UI_Pause`.
///
/// Source: `oracle/codemp/ui/ui_main.c:10171-10182`
pub fn UI_Pause(ctx: &mut UiContext, b: bool) {
    if b {
        // pause the game and set the ui keycatcher
        trap::Cvar_Set(ctx.engine, "cl_paused", "1");
        trap::Key_SetCatcher(ctx.engine, KEYCATCH_UI);
    } else {
        // unpause the game and clear the ui keycatcher
        let catcher = trap::Key_GetCatcher(ctx.engine);
        trap::Key_SetCatcher(ctx.engine, catcher & !KEYCATCH_UI);
        trap::Key_ClearStates(ctx.engine);
        trap::Cvar_Set(ctx.engine, "cl_paused", "0");
    }
}

/// Raven `UI_PlayCinematic`.
///
/// Source: `oracle/codemp/ui/ui_main.c:10184-10186`
pub fn UI_PlayCinematic(ctx: &mut UiContext, name: &str, x: f32, y: f32, w: f32, h: f32) -> c_int {
    trap::CIN_PlayCinematic(
        ctx.engine,
        name,
        x as c_int,
        y as c_int,
        w as c_int,
        h as c_int,
        CIN_LOOP | CIN_SILENT,
    )
}

/// Raven `UI_DrawCinematic`.
///
/// Source: `oracle/codemp/ui/ui_main.c:10215-10218`
pub fn UI_DrawCinematic(ctx: &mut UiContext, handle: c_int, x: f32, y: f32, w: f32, h: f32) {
    trap::CIN_SetExtents(
        ctx.engine, handle, x as c_int, y as c_int, w as c_int, h as c_int,
    );
    trap::CIN_DrawCinematic(ctx.engine, handle);
}

/// Raven `UI_RunCinematicFrame`.
///
/// Source: `oracle/codemp/ui/ui_main.c:10220-10222`
pub fn UI_RunCinematicFrame(ctx: &mut UiContext, handle: c_int) {
    trap::CIN_RunCinematic(ctx.engine, handle);
}

/// Raven `UI_LoadForceConfig_List`.
///
/// PORT-NOTE: `COM_StripExtension` is qshared's `(name: &str) -> String`
/// out-param-to-return reshape. Raven's `goto nextSearch` two-pass
/// (dark then light) becomes a `bool` flag loop that runs exactly twice
/// (porting-rules §10 — behavior preserved, shape idiomatic).
///
/// Source: `oracle/codemp/ui/ui_main.c:10231-10283`
pub fn UI_LoadForceConfig_List(ctx: &mut UiContext) {
    ctx.world.forceConfigNames.clear();
    ctx.world.forceConfigSide.clear();
    // Always reserve index 0 as the "custom" config. Raven never writes
    // `forceConfigSide[0]` (zeroed static = qfalse); the placeholder keeps the
    // side table index-aligned with `forceConfigNames`.
    ctx.world.forceConfigNames.push("Custom".to_string());
    ctx.world.forceConfigSide.push(false);

    let mut lightSearch = false;
    loop {
        let mut filelist = vec![0u8; 2048];
        let numfiles = if lightSearch {
            // search light side folder
            let n = trap::FS_GetFileList(ctx.engine, "forcecfg/light", "fcf", &mut filelist);
            ctx.world.forceConfigLightIndexBegin = ctx.world.forceConfigNames.len() as c_int - 1;
            n
        } else {
            // search dark side folder
            let n = trap::FS_GetFileList(ctx.engine, "forcecfg/dark", "fcf", &mut filelist);
            ctx.world.forceConfigDarkIndexBegin = ctx.world.forceConfigNames.len() as c_int - 1;
            n
        };

        let names = latin1_to_string(&filelist);
        // Raven walks `fileptr += filelen+1` — empty entries are consumed,
        // not skipped, so no is_empty filter (entry i must stay entry i).
        let mut fileptrs = names.split('\0');

        let mut j = 0;
        while j < numfiles && ctx.world.forceConfigNames.len() < MAX_FORCE_CONFIGS {
            let fileptr = match fileptrs.next() {
                Some(f) => f,
                None => break,
            };
            let configname = COM_StripExtension(fileptr);

            ctx.world.forceConfigSide.push(lightSearch);
            ctx.world.forceConfigNames.push(configname);
            j += 1;
        }

        if !lightSearch {
            lightSearch = true;
        } else {
            break;
        }
    }
}

/// Raven `bIsImageFile`.
///
/// PORT-NOTE: the `_XBOX` `.dds`-only path (and its `_DEBUG` png/tga
/// fallback gate) is dead PC surface (porting-rules §20) and is dropped; the
/// PC build's jpg/png/tga fallback chain is transcribed unconditionally.
///
/// Source: `oracle/codemp/ui/ui_main.c:10292-10322`
pub fn bIsImageFile(ctx: &mut UiContext, dirptr: &str, skinname: &str) -> bool {
    let mut fpath = format!("models/players/{}/icon_{}.jpg", dirptr, skinname);
    let mut f: fileHandle_t = 0;
    trap::FS_FOpenFile(ctx.engine, &fpath, &mut f, FS_READ);

    if f == 0 {
        // not there, try png
        fpath = format!("models/players/{}/icon_{}.png", dirptr, skinname);
        trap::FS_FOpenFile(ctx.engine, &fpath, &mut f, FS_READ);
    }
    if f == 0 {
        // not there, try tga
        fpath = format!("models/players/{}/icon_{}.tga", dirptr, skinname);
        trap::FS_FOpenFile(ctx.engine, &fpath, &mut f, FS_READ);
    }
    let _ = &fpath;

    if f != 0 {
        trap::FS_FCloseFile(ctx.engine, f);
        return true;
    }
    false
}

/// Raven `UI_ParseColorData`.
///
/// PORT-NOTE: `COM_BeginParseSession`/`COM_ParseExt` take qshared's
/// `QSharedScratch` (Raven's `com_lines`/`com_parsename` parse-session
/// globals) and a byte-slice cursor, so `qs` is threaded in as a parameter
/// (ui has no owned scratch home yet — see escalations). `ColorCount` is the
/// `ColorActionText`/`ColorShader` `len()` (`PlayerSpeciesInfo`'s Vec model),
/// so the shader token is held until its action block closes rather than
/// written at `[ColorCount]` and left uncounted on the failure paths.
///
/// Source: `oracle/codemp/ui/ui_main.c:10468-10508`
pub fn UI_ParseColorData(
    qs: &mut QSharedScratch,
    buf: &str,
    species: &mut PlayerSpeciesInfo,
    file: &str,
) -> bool {
    let mut p: Option<&[u8]> = Some(buf.as_bytes());
    COM_BeginParseSession(qs, file);
    species.ColorShader.clear();
    species.ColorActionText.clear();

    while p.is_some() {
        // looking for the shader
        let (token, rest) = COM_ParseExt(qs, p, true);
        p = rest;
        if token.is_empty() {
            return !species.ColorActionText.is_empty();
        }
        let shader = token;

        // looking for action block {
        let (token, rest) = COM_ParseExt(qs, p, true);
        p = rest;
        // Raven tests `token[0]` only, not the whole token.
        if !token.starts_with('{') {
            return false;
        }

        // looking for action commands
        let mut actionText = String::new();
        let (mut token, rest) = COM_ParseExt(qs, p, true);
        p = rest;
        while !token.starts_with('}') {
            if token.is_empty() {
                // EOF
                return false;
            }
            actionText.push_str(&token);
            actionText.push(' ');
            // looking for action commands or final }
            let (next, rest) = COM_ParseExt(qs, p, true);
            p = rest;
            token = next;
        }
        // next color please
        species.ColorShader.push(shader);
        species.ColorActionText.push(actionText);
    }
    true // never get here
}

/// Raven `UI_ReadableSize`.
///
/// Source: `oracle/codemp/ui/ui_main.c:11039-11054`
pub fn UI_ReadableSize(value: c_int) -> String {
    if value > 1024 * 1024 * 1024 {
        // gigs
        format!(
            "{}.{:02} GB",
            value / (1024 * 1024 * 1024),
            (value % (1024 * 1024 * 1024)) * 100 / (1024 * 1024 * 1024)
        )
    } else if value > 1024 * 1024 {
        // megs
        format!(
            "{}.{:02} MB",
            value / (1024 * 1024),
            (value % (1024 * 1024)) * 100 / (1024 * 1024)
        )
    } else if value > 1024 {
        // kilos
        format!("{} KB", value / 1024)
    } else {
        // bytes
        format!("{} bytes", value)
    }
}

/// Raven `UI_PrintTime`.
///
/// Source: `oracle/codemp/ui/ui_main.c:11057-11067`
pub fn UI_PrintTime(time: c_int) -> String {
    let time = time / 1000; // change to seconds

    if time > 3600 {
        // in the hours range
        format!("{} hr {:2} min", time / 3600, (time % 3600) / 60)
    } else if time > 60 {
        // mins
        format!("{:2} min {:2} sec", time / 60, time % 60)
    } else {
        // secs
        format!("{:2} sec", time)
    }
}

/// Raven `cvarTable_t` — one `cvarTable` row. Raven's `vmCvar_t *vmCvar`
/// pointer becomes `field`, the [`UiCvars`] member name holding that cvar
/// (Rust has no runtime field reflection; the `GAME_CVAR_TABLE` precedent in
/// `g_main.rs`).
///
/// Type definition source: `oracle/codemp/ui/ui_main.c:11278-11284`
struct UiCvarTableEntry {
    field: &'static str,
    name: &'static str,
    default: &'static str,
    flags: c_int,
}

/// Raven `cvarTable` — the compiled-in name/default/flags registration table
/// (99 rows, verbatim order). The `#ifdef _XBOX` rows are dead PC surface
/// (porting-rules §20) and are dropped.
///
/// Source: `oracle/codemp/ui/ui_main.c:11399-11532`
const UI_CVAR_TABLE: [UiCvarTableEntry; 99] = [
    UiCvarTableEntry {
        field: "ui_ffa_fraglimit",
        name: "ui_ffa_fraglimit",
        default: "20",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_ffa_timelimit",
        name: "ui_ffa_timelimit",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_selectedModelIndex",
        name: "ui_selectedModelIndex",
        default: "16",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_char_model",
        name: "ui_char_model",
        default: "jedi_tf",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_char_skin_head",
        name: "ui_char_skin_head",
        default: "head_a1",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_char_skin_torso",
        name: "ui_char_skin_torso",
        default: "torso_a1",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_char_skin_legs",
        name: "ui_char_skin_legs",
        default: "lower_a1",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_char_anim",
        name: "ui_char_anim",
        default: "BOTH_WALK1",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_saber_type",
        name: "ui_saber_type",
        default: "single",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_saber",
        name: "ui_saber",
        default: "single_1",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_saber2",
        name: "ui_saber2",
        default: "none",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_saber_color",
        name: "ui_saber_color",
        default: "yellow",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_saber2_color",
        name: "ui_saber2_color",
        default: "yellow",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_char_color_red",
        name: "ui_char_color_red",
        default: "255",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_char_color_green",
        name: "ui_char_color_green",
        default: "255",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_char_color_blue",
        name: "ui_char_color_blue",
        default: "255",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_PrecacheModels",
        name: "ui_PrecacheModels",
        default: "0",
        flags: CVAR_ARCHIVE,
    },
    UiCvarTableEntry {
        field: "ui_team_fraglimit",
        name: "ui_team_fraglimit",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_team_timelimit",
        name: "ui_team_timelimit",
        default: "20",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_team_friendly",
        name: "ui_team_friendly",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_ctf_capturelimit",
        name: "ui_ctf_capturelimit",
        default: "8",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_ctf_timelimit",
        name: "ui_ctf_timelimit",
        default: "30",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_ctf_friendly",
        name: "ui_ctf_friendly",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_botsFile",
        name: "g_botsFile",
        default: "",
        flags: CVAR_INIT | CVAR_ROM,
    },
    UiCvarTableEntry {
        field: "ui_spSkill",
        name: "g_spSkill",
        default: "2",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_browserMaster",
        name: "ui_browserMaster",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_browserGameType",
        name: "ui_browserGameType",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_browserSortKey",
        name: "ui_browserSortKey",
        default: "4",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_browserShowFull",
        name: "ui_browserShowFull",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_browserShowEmpty",
        name: "ui_browserShowEmpty",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_drawCrosshair",
        name: "cg_drawCrosshair",
        default: "1",
        flags: CVAR_ARCHIVE,
    },
    UiCvarTableEntry {
        field: "ui_drawCrosshairNames",
        name: "cg_drawCrosshairNames",
        default: "1",
        flags: CVAR_ARCHIVE,
    },
    UiCvarTableEntry {
        field: "ui_marks",
        name: "cg_marks",
        default: "1",
        flags: CVAR_ARCHIVE,
    },
    UiCvarTableEntry {
        field: "ui_debug",
        name: "ui_debug",
        default: "0",
        flags: CVAR_TEMP | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_initialized",
        name: "ui_initialized",
        default: "0",
        flags: CVAR_TEMP | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_opponentName",
        name: "ui_opponentName",
        default: "Rebellion",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_rankChange",
        name: "ui_rankChange",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_freeSaber",
        name: "ui_freeSaber",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_forcePowerDisable",
        name: "ui_forcePowerDisable",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_redteam",
        name: "ui_redteam",
        default: "Empire",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_blueteam",
        name: "ui_blueteam",
        default: "Rebellion",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_dedicated",
        name: "ui_dedicated",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_gameType",
        name: "ui_gametype",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_joinGameType",
        name: "ui_joinGametype",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_netGameType",
        name: "ui_netGametype",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_actualNetGameType",
        name: "ui_actualNetGametype",
        default: "3",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_redteam1",
        name: "ui_redteam1",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_redteam2",
        name: "ui_redteam2",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_redteam3",
        name: "ui_redteam3",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_redteam4",
        name: "ui_redteam4",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_redteam5",
        name: "ui_redteam5",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_redteam6",
        name: "ui_redteam6",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_redteam7",
        name: "ui_redteam7",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_redteam8",
        name: "ui_redteam8",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_blueteam1",
        name: "ui_blueteam1",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_blueteam2",
        name: "ui_blueteam2",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_blueteam3",
        name: "ui_blueteam3",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_blueteam4",
        name: "ui_blueteam4",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_blueteam5",
        name: "ui_blueteam5",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_blueteam6",
        name: "ui_blueteam6",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_blueteam7",
        name: "ui_blueteam7",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_blueteam8",
        name: "ui_blueteam8",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_netSource",
        name: "ui_netSource",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_menuFiles",
        name: "ui_menuFilesMP",
        default: "ui/jampmenus.txt",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_currentMap",
        name: "ui_currentMap",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_currentNetMap",
        name: "ui_currentNetMap",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_mapIndex",
        name: "ui_mapIndex",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_currentOpponent",
        name: "ui_currentOpponent",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_selectedPlayer",
        name: "cg_selectedPlayer",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_selectedPlayerName",
        name: "cg_selectedPlayerName",
        default: "",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_lastServerRefresh_0",
        name: "ui_lastServerRefresh_0",
        default: "",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_lastServerRefresh_1",
        name: "ui_lastServerRefresh_1",
        default: "",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_lastServerRefresh_2",
        name: "ui_lastServerRefresh_2",
        default: "",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_lastServerRefresh_3",
        name: "ui_lastServerRefresh_3",
        default: "",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_singlePlayerActive",
        name: "ui_singlePlayerActive",
        default: "0",
        flags: CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreAccuracy",
        name: "ui_scoreAccuracy",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreImpressives",
        name: "ui_scoreImpressives",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreExcellents",
        name: "ui_scoreExcellents",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreCaptures",
        name: "ui_scoreCaptures",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreDefends",
        name: "ui_scoreDefends",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreAssists",
        name: "ui_scoreAssists",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreGauntlets",
        name: "ui_scoreGauntlets",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreScore",
        name: "ui_scoreScore",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scorePerfect",
        name: "ui_scorePerfect",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreTeam",
        name: "ui_scoreTeam",
        default: "0 to 0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreBase",
        name: "ui_scoreBase",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreTime",
        name: "ui_scoreTime",
        default: "00:00",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreTimeBonus",
        name: "ui_scoreTimeBonus",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreSkillBonus",
        name: "ui_scoreSkillBonus",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreShutoutBonus",
        name: "ui_scoreShutoutBonus",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_fragLimit",
        name: "ui_fragLimit",
        default: "10",
        flags: CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_captureLimit",
        name: "ui_captureLimit",
        default: "5",
        flags: CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_findPlayer",
        name: "ui_findPlayer",
        default: "Kyle",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_recordSPDemo",
        name: "ui_recordSPDemo",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_realWarmUp",
        name: "g_warmup",
        default: "20",
        flags: CVAR_ARCHIVE,
    },
    UiCvarTableEntry {
        field: "ui_realCaptureLimit",
        name: "capturelimit",
        default: "0",
        flags: CVAR_SERVERINFO | CVAR_ARCHIVE | CVAR_NORESTART,
    },
    UiCvarTableEntry {
        field: "ui_serverStatusTimeOut",
        name: "ui_serverStatusTimeOut",
        default: "7000",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "se_language",
        name: "se_language",
        default: "english",
        flags: CVAR_ARCHIVE | CVAR_NORESTART,
    },
    UiCvarTableEntry {
        field: "ui_bypassMainMenuLoad",
        name: "ui_bypassMainMenuLoad",
        default: "0",
        flags: CVAR_INTERNAL,
    },
];

/// Maps one `UI_CVAR_TABLE` row's `field` name to its `vmCvar_t` storage in
/// `UiCvars`, standing in for the `cv->vmCvar` pointer Raven's row carries.
impl UiCvars {
    fn field_mut(&mut self, name: &str) -> &mut vmCvar_t {
        let cvars = self;
        match name {
            "ui_ffa_fraglimit" => &mut cvars.ui_ffa_fraglimit,
            "ui_ffa_timelimit" => &mut cvars.ui_ffa_timelimit,
            "ui_selectedModelIndex" => &mut cvars.ui_selectedModelIndex,
            "ui_char_model" => &mut cvars.ui_char_model,
            "ui_char_skin_head" => &mut cvars.ui_char_skin_head,
            "ui_char_skin_torso" => &mut cvars.ui_char_skin_torso,
            "ui_char_skin_legs" => &mut cvars.ui_char_skin_legs,
            "ui_char_anim" => &mut cvars.ui_char_anim,
            "ui_saber_type" => &mut cvars.ui_saber_type,
            "ui_saber" => &mut cvars.ui_saber,
            "ui_saber2" => &mut cvars.ui_saber2,
            "ui_saber_color" => &mut cvars.ui_saber_color,
            "ui_saber2_color" => &mut cvars.ui_saber2_color,
            "ui_char_color_red" => &mut cvars.ui_char_color_red,
            "ui_char_color_green" => &mut cvars.ui_char_color_green,
            "ui_char_color_blue" => &mut cvars.ui_char_color_blue,
            "ui_PrecacheModels" => &mut cvars.ui_PrecacheModels,
            "ui_team_fraglimit" => &mut cvars.ui_team_fraglimit,
            "ui_team_timelimit" => &mut cvars.ui_team_timelimit,
            "ui_team_friendly" => &mut cvars.ui_team_friendly,
            "ui_ctf_capturelimit" => &mut cvars.ui_ctf_capturelimit,
            "ui_ctf_timelimit" => &mut cvars.ui_ctf_timelimit,
            "ui_ctf_friendly" => &mut cvars.ui_ctf_friendly,
            "ui_botsFile" => &mut cvars.ui_botsFile,
            "ui_spSkill" => &mut cvars.ui_spSkill,
            "ui_browserMaster" => &mut cvars.ui_browserMaster,
            "ui_browserGameType" => &mut cvars.ui_browserGameType,
            "ui_browserSortKey" => &mut cvars.ui_browserSortKey,
            "ui_browserShowFull" => &mut cvars.ui_browserShowFull,
            "ui_browserShowEmpty" => &mut cvars.ui_browserShowEmpty,
            "ui_drawCrosshair" => &mut cvars.ui_drawCrosshair,
            "ui_drawCrosshairNames" => &mut cvars.ui_drawCrosshairNames,
            "ui_marks" => &mut cvars.ui_marks,
            "ui_debug" => &mut cvars.ui_debug,
            "ui_initialized" => &mut cvars.ui_initialized,
            "ui_opponentName" => &mut cvars.ui_opponentName,
            "ui_rankChange" => &mut cvars.ui_rankChange,
            "ui_freeSaber" => &mut cvars.ui_freeSaber,
            "ui_forcePowerDisable" => &mut cvars.ui_forcePowerDisable,
            "ui_redteam" => &mut cvars.ui_redteam,
            "ui_blueteam" => &mut cvars.ui_blueteam,
            "ui_dedicated" => &mut cvars.ui_dedicated,
            "ui_gameType" => &mut cvars.ui_gameType,
            "ui_joinGameType" => &mut cvars.ui_joinGameType,
            "ui_netGameType" => &mut cvars.ui_netGameType,
            "ui_actualNetGameType" => &mut cvars.ui_actualNetGameType,
            "ui_redteam1" => &mut cvars.ui_redteam1,
            "ui_redteam2" => &mut cvars.ui_redteam2,
            "ui_redteam3" => &mut cvars.ui_redteam3,
            "ui_redteam4" => &mut cvars.ui_redteam4,
            "ui_redteam5" => &mut cvars.ui_redteam5,
            "ui_redteam6" => &mut cvars.ui_redteam6,
            "ui_redteam7" => &mut cvars.ui_redteam7,
            "ui_redteam8" => &mut cvars.ui_redteam8,
            "ui_blueteam1" => &mut cvars.ui_blueteam1,
            "ui_blueteam2" => &mut cvars.ui_blueteam2,
            "ui_blueteam3" => &mut cvars.ui_blueteam3,
            "ui_blueteam4" => &mut cvars.ui_blueteam4,
            "ui_blueteam5" => &mut cvars.ui_blueteam5,
            "ui_blueteam6" => &mut cvars.ui_blueteam6,
            "ui_blueteam7" => &mut cvars.ui_blueteam7,
            "ui_blueteam8" => &mut cvars.ui_blueteam8,
            "ui_netSource" => &mut cvars.ui_netSource,
            "ui_menuFiles" => &mut cvars.ui_menuFiles,
            "ui_currentMap" => &mut cvars.ui_currentMap,
            "ui_currentNetMap" => &mut cvars.ui_currentNetMap,
            "ui_mapIndex" => &mut cvars.ui_mapIndex,
            "ui_currentOpponent" => &mut cvars.ui_currentOpponent,
            "ui_selectedPlayer" => &mut cvars.ui_selectedPlayer,
            "ui_selectedPlayerName" => &mut cvars.ui_selectedPlayerName,
            "ui_lastServerRefresh_0" => &mut cvars.ui_lastServerRefresh_0,
            "ui_lastServerRefresh_1" => &mut cvars.ui_lastServerRefresh_1,
            "ui_lastServerRefresh_2" => &mut cvars.ui_lastServerRefresh_2,
            "ui_lastServerRefresh_3" => &mut cvars.ui_lastServerRefresh_3,
            "ui_singlePlayerActive" => &mut cvars.ui_singlePlayerActive,
            "ui_scoreAccuracy" => &mut cvars.ui_scoreAccuracy,
            "ui_scoreImpressives" => &mut cvars.ui_scoreImpressives,
            "ui_scoreExcellents" => &mut cvars.ui_scoreExcellents,
            "ui_scoreCaptures" => &mut cvars.ui_scoreCaptures,
            "ui_scoreDefends" => &mut cvars.ui_scoreDefends,
            "ui_scoreAssists" => &mut cvars.ui_scoreAssists,
            "ui_scoreGauntlets" => &mut cvars.ui_scoreGauntlets,
            "ui_scoreScore" => &mut cvars.ui_scoreScore,
            "ui_scorePerfect" => &mut cvars.ui_scorePerfect,
            "ui_scoreTeam" => &mut cvars.ui_scoreTeam,
            "ui_scoreBase" => &mut cvars.ui_scoreBase,
            "ui_scoreTime" => &mut cvars.ui_scoreTime,
            "ui_scoreTimeBonus" => &mut cvars.ui_scoreTimeBonus,
            "ui_scoreSkillBonus" => &mut cvars.ui_scoreSkillBonus,
            "ui_scoreShutoutBonus" => &mut cvars.ui_scoreShutoutBonus,
            "ui_fragLimit" => &mut cvars.ui_fragLimit,
            "ui_captureLimit" => &mut cvars.ui_captureLimit,
            "ui_findPlayer" => &mut cvars.ui_findPlayer,
            "ui_recordSPDemo" => &mut cvars.ui_recordSPDemo,
            "ui_realWarmUp" => &mut cvars.ui_realWarmUp,
            "ui_realCaptureLimit" => &mut cvars.ui_realCaptureLimit,
            "ui_serverStatusTimeOut" => &mut cvars.ui_serverStatusTimeOut,
            "se_language" => &mut cvars.se_language,
            "ui_bypassMainMenuLoad" => &mut cvars.ui_bypassMainMenuLoad,
            other => unreachable!("UI_CVAR_TABLE row field {other:?} has no UiCvars member"),
        }
    }
}

/// Raven `UI_RegisterCvars`.
///
/// Source: `oracle/codemp/ui/ui_main.c:11540-11547`
pub fn UI_RegisterCvars(ctx: &mut UiContext) {
    for cv in UI_CVAR_TABLE.iter() {
        trap::Cvar_Register(
            ctx.engine,
            Some(ctx.world.cvars.field_mut(cv.field)),
            cv.name,
            cv.default,
            cv.flags,
        );
    }
}

/// Raven `UI_UpdateCvars`.
///
/// Source: `oracle/codemp/ui/ui_main.c:11554-11561`
pub fn UI_UpdateCvars(ctx: &mut UiContext) {
    for cv in UI_CVAR_TABLE.iter() {
        trap::Cvar_Update(ctx.engine, ctx.world.cvars.field_mut(cv.field));
    }
}

/// Raven `UI_UpdateVideoSetup`.
///
/// Source: `oracle/codemp/ui/ui_main.c:5473-5493`
pub fn UI_UpdateVideoSetup(ctx: &mut UiContext) {
    let r_mode = UI_Cvar_VariableString(ctx, "ui_r_mode");
    trap::Cvar_Set(ctx.engine, "r_mode", &r_mode);
    let r_fullscreen = UI_Cvar_VariableString(ctx, "ui_r_fullscreen");
    trap::Cvar_Set(ctx.engine, "r_fullscreen", &r_fullscreen);
    let r_colorbits = UI_Cvar_VariableString(ctx, "ui_r_colorbits");
    trap::Cvar_Set(ctx.engine, "r_colorbits", &r_colorbits);
    let r_lodbias = UI_Cvar_VariableString(ctx, "ui_r_lodbias");
    trap::Cvar_Set(ctx.engine, "r_lodbias", &r_lodbias);
    let r_picmip = UI_Cvar_VariableString(ctx, "ui_r_picmip");
    trap::Cvar_Set(ctx.engine, "r_picmip", &r_picmip);
    let r_texturebits = UI_Cvar_VariableString(ctx, "ui_r_texturebits");
    trap::Cvar_Set(ctx.engine, "r_texturebits", &r_texturebits);
    let r_texturemode = UI_Cvar_VariableString(ctx, "ui_r_texturemode");
    trap::Cvar_Set(ctx.engine, "r_texturemode", &r_texturemode);
    let r_detailtextures = UI_Cvar_VariableString(ctx, "ui_r_detailtextures");
    trap::Cvar_Set(ctx.engine, "r_detailtextures", &r_detailtextures);
    let r_ext_compress_textures = UI_Cvar_VariableString(ctx, "ui_r_ext_compress_textures");
    trap::Cvar_Set(
        ctx.engine,
        "r_ext_compress_textures",
        &r_ext_compress_textures,
    );
    let r_depthbits = UI_Cvar_VariableString(ctx, "ui_r_depthbits");
    trap::Cvar_Set(ctx.engine, "r_depthbits", &r_depthbits);
    let r_subdivisions = UI_Cvar_VariableString(ctx, "ui_r_subdivisions");
    trap::Cvar_Set(ctx.engine, "r_subdivisions", &r_subdivisions);
    let r_fastSky = UI_Cvar_VariableString(ctx, "ui_r_fastSky");
    trap::Cvar_Set(ctx.engine, "r_fastSky", &r_fastSky);
    let r_inGameVideo = UI_Cvar_VariableString(ctx, "ui_r_inGameVideo");
    trap::Cvar_Set(ctx.engine, "r_inGameVideo", &r_inGameVideo);
    let r_allowExtensions = UI_Cvar_VariableString(ctx, "ui_r_allowExtensions");
    trap::Cvar_Set(ctx.engine, "r_allowExtensions", &r_allowExtensions);
    let cg_shadows = UI_Cvar_VariableString(ctx, "ui_cg_shadows");
    trap::Cvar_Set(ctx.engine, "cg_shadows", &cg_shadows);
    trap::Cvar_Set(ctx.engine, "ui_r_modified", "0");

    trap::Cmd_ExecuteText(ctx.engine, cbufExec_t::EXEC_APPEND as c_int, "vid_restart;");
}

/// Raven `UI_GetVideoSetup`.
///
/// Source: `oracle/codemp/ui/ui_main.c:5503-5542`
pub fn UI_GetVideoSetup(ctx: &mut UiContext) {
    // Make sure the cvars are registered as read only.
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_glCustom",
        "4",
        CVAR_ROM | CVAR_INTERNAL | CVAR_ARCHIVE,
    );

    trap::Cvar_Register(ctx.engine, None, "ui_r_mode", "0", CVAR_ROM | CVAR_INTERNAL);
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_fullscreen",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_colorbits",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_lodbias",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_picmip",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_texturebits",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_texturemode",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_detailtextures",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_ext_compress_textures",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_depthbits",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_subdivisions",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_fastSky",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_inGameVideo",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_allowExtensions",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_cg_shadows",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_modified",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );

    // Copy over the real video cvars into their temporary counterparts
    let r_mode = UI_Cvar_VariableString(ctx, "r_mode");
    trap::Cvar_Set(ctx.engine, "ui_r_mode", &r_mode);
    let r_colorbits = UI_Cvar_VariableString(ctx, "r_colorbits");
    trap::Cvar_Set(ctx.engine, "ui_r_colorbits", &r_colorbits);
    let r_fullscreen = UI_Cvar_VariableString(ctx, "r_fullscreen");
    trap::Cvar_Set(ctx.engine, "ui_r_fullscreen", &r_fullscreen);
    let r_lodbias = UI_Cvar_VariableString(ctx, "r_lodbias");
    trap::Cvar_Set(ctx.engine, "ui_r_lodbias", &r_lodbias);
    let r_picmip = UI_Cvar_VariableString(ctx, "r_picmip");
    trap::Cvar_Set(ctx.engine, "ui_r_picmip", &r_picmip);
    let r_texturebits = UI_Cvar_VariableString(ctx, "r_texturebits");
    trap::Cvar_Set(ctx.engine, "ui_r_texturebits", &r_texturebits);
    let r_texturemode = UI_Cvar_VariableString(ctx, "r_texturemode");
    trap::Cvar_Set(ctx.engine, "ui_r_texturemode", &r_texturemode);
    let r_detailtextures = UI_Cvar_VariableString(ctx, "r_detailtextures");
    trap::Cvar_Set(ctx.engine, "ui_r_detailtextures", &r_detailtextures);
    let r_ext_compress_textures = UI_Cvar_VariableString(ctx, "r_ext_compress_textures");
    trap::Cvar_Set(
        ctx.engine,
        "ui_r_ext_compress_textures",
        &r_ext_compress_textures,
    );
    let r_depthbits = UI_Cvar_VariableString(ctx, "r_depthbits");
    trap::Cvar_Set(ctx.engine, "ui_r_depthbits", &r_depthbits);
    let r_subdivisions = UI_Cvar_VariableString(ctx, "r_subdivisions");
    trap::Cvar_Set(ctx.engine, "ui_r_subdivisions", &r_subdivisions);
    let r_fastSky = UI_Cvar_VariableString(ctx, "r_fastSky");
    trap::Cvar_Set(ctx.engine, "ui_r_fastSky", &r_fastSky);
    let r_inGameVideo = UI_Cvar_VariableString(ctx, "r_inGameVideo");
    trap::Cvar_Set(ctx.engine, "ui_r_inGameVideo", &r_inGameVideo);
    let r_allowExtensions = UI_Cvar_VariableString(ctx, "r_allowExtensions");
    trap::Cvar_Set(ctx.engine, "ui_r_allowExtensions", &r_allowExtensions);
    let cg_shadows = UI_Cvar_VariableString(ctx, "cg_shadows");
    trap::Cvar_Set(ctx.engine, "ui_cg_shadows", &cg_shadows);
    trap::Cvar_Set(ctx.engine, "ui_r_modified", "0");
}

/// Raven `UI_UpdateCharacterCvars`.
///
/// Source: `oracle/codemp/ui/ui_main.c:5575-5602`
pub fn UI_UpdateCharacterCvars(ctx: &mut UiContext) {
    let model = trap::Cvar_VariableStringBuffer(ctx.engine, "ui_char_model", MAX_QPATH as usize);
    let head = trap::Cvar_VariableStringBuffer(ctx.engine, "ui_char_skin_head", MAX_QPATH as usize);
    let torso =
        trap::Cvar_VariableStringBuffer(ctx.engine, "ui_char_skin_torso", MAX_QPATH as usize);
    let legs = trap::Cvar_VariableStringBuffer(ctx.engine, "ui_char_skin_legs", MAX_QPATH as usize);

    // PORT-NOTE: Raven `Com_sprintf` into `char skin[MAX_QPATH]`.
    let skin: String = format!("{}/{}|{}|{}", model, head, torso, legs)
        .chars()
        .take(MAX_QPATH as usize - 1)
        .collect();

    trap::Cvar_Set(ctx.engine, "model", &skin);

    let char_color_red = UI_Cvar_VariableString(ctx, "ui_char_color_red");
    trap::Cvar_Set(ctx.engine, "char_color_red", &char_color_red);
    let char_color_green = UI_Cvar_VariableString(ctx, "ui_char_color_green");
    trap::Cvar_Set(ctx.engine, "char_color_green", &char_color_green);
    let char_color_blue = UI_Cvar_VariableString(ctx, "ui_char_color_blue");
    trap::Cvar_Set(ctx.engine, "char_color_blue", &char_color_blue);
    trap::Cvar_Set(ctx.engine, "ui_selectedModelIndex", "-1");
}

/// Raven `UI_GetCharacterCvars`.
///
/// PORT-NOTE: Raven's `strrchr`/`strchr` pointer walk over the mutable
/// `"model"` cvar string is transcribed as byte-offset splits over the owned
/// `String` (the delimiters `/` and `|` are single-byte ASCII, so byte offsets
/// stay char-boundary-safe under the Latin-1 discipline); the `assert(p2)`
/// guards on the second and third `|` become `.expect(..)`, matching Raven's
/// abort-on-violation behavior.
///
/// Source: `oracle/codemp/ui/ui_main.c:5604-5678`
pub fn UI_GetCharacterCvars(ctx: &mut UiContext) {
    let char_color_red = UI_Cvar_VariableString(ctx, "char_color_red");
    trap::Cvar_Set(ctx.engine, "ui_char_color_red", &char_color_red);
    let char_color_green = UI_Cvar_VariableString(ctx, "char_color_green");
    trap::Cvar_Set(ctx.engine, "ui_char_color_green", &char_color_green);
    let char_color_blue = UI_Cvar_VariableString(ctx, "char_color_blue");
    trap::Cvar_Set(ctx.engine, "ui_char_color_blue", &char_color_blue);

    let model = UI_Cvar_VariableString(ctx, "model");
    if let Some(slash) = model.rfind('/') {
        if model.contains('|') {
            // we have a multipart custom jedi
            let base = model[..slash].to_string();
            let rest = &model[slash + 1..];

            let p1 = rest
                .find('|')
                .expect("multipart custom jedi model string missing '|' separator");
            let skinhead = rest[..p1].to_string();
            let rest2 = &rest[p1 + 1..];

            let p2 = rest2
                .find('|')
                .expect("multipart custom jedi model string missing second '|' separator");
            let skintorso = rest2[..p2].to_string();
            let skinlower = rest2[p2 + 1..].to_string();

            trap::Cvar_Set(ctx.engine, "ui_char_model", &base);
            trap::Cvar_Set(ctx.engine, "ui_char_skin_head", &skinhead);
            trap::Cvar_Set(ctx.engine, "ui_char_skin_torso", &skintorso);
            trap::Cvar_Set(ctx.engine, "ui_char_skin_legs", &skinlower);

            for i in 0..ctx.world.playerSpecies.len() {
                if Q_stricmp(&base, &ctx.world.playerSpecies[i].Name) == 0 {
                    ctx.world.playerSpeciesIndex = i as c_int;
                    break;
                }
            }
            return;
        }
    }

    let model = UI_Cvar_VariableString(ctx, "ui_char_model");
    for i in 0..ctx.world.playerSpecies.len() {
        if Q_stricmp(&model, &ctx.world.playerSpecies[i].Name) == 0 {
            ctx.world.playerSpeciesIndex = i as c_int;
            return; // FOUND IT, don't fall through
        }
    }
    // nope, didn't find it.
    ctx.world.playerSpeciesIndex = 0; // jic
    let name = ctx.world.playerSpecies[ctx.world.playerSpeciesIndex as usize]
        .Name
        .clone();
    trap::Cvar_Set(ctx.engine, "ui_char_model", &name);
    trap::Cvar_Set(ctx.engine, "ui_char_skin_head", "head_a1");
    trap::Cvar_Set(ctx.engine, "ui_char_skin_torso", "torso_a1");
    trap::Cvar_Set(ctx.engine, "ui_char_skin_legs", "lower_a1");
}

/// Raven `UI_UpdateSaberCvars`.
///
/// Source: `oracle/codemp/ui/ui_main.c:5851-5865`
pub fn UI_UpdateSaberCvars(ctx: &mut UiContext) {
    let saber1 = UI_Cvar_VariableString(ctx, "ui_saber");
    trap::Cvar_Set(ctx.engine, "saber1", &saber1);
    let saber2 = UI_Cvar_VariableString(ctx, "ui_saber2");
    trap::Cvar_Set(ctx.engine, "saber2", &saber2);

    let saber_color = UI_Cvar_VariableString(ctx, "ui_saber_color");
    let colorI = TranslateSaberColor(&saber_color, &mut ctx.world.bg_state);
    trap::Cvar_Set(ctx.engine, "color1", &format!("{}", colorI));
    let g_saber_color = UI_Cvar_VariableString(ctx, "ui_saber_color");
    trap::Cvar_Set(ctx.engine, "g_saber_color", &g_saber_color);

    let saber2_color = UI_Cvar_VariableString(ctx, "ui_saber2_color");
    let colorI = TranslateSaberColor(&saber2_color, &mut ctx.world.bg_state);
    trap::Cvar_Set(ctx.engine, "color2", &format!("{}", colorI));
    let g_saber2_color = UI_Cvar_VariableString(ctx, "ui_saber2_color");
    trap::Cvar_Set(ctx.engine, "g_saber2_color", &g_saber2_color);
}

/// Raven `UI_SetSaberBoxesandHilts`.
///
/// Source: `oracle/codemp/ui/ui_main.c:5868-5942`
pub fn UI_SetSaberBoxesandHilts(ctx: &mut UiContext) {
    // Get current menu (either video or ingame video, I would assume)
    let menu = match Menu_GetFocused(&ctx.world.menus) {
        Some(m) => m,
        None => return,
    };

    let sType = trap::Cvar_VariableStringBuffer(ctx.engine, "ui_saber_type", MAX_QPATH as usize);

    let mut getBig = false;

    if Q_stricmp("dual", &sType) != 0 {
        getBig = true;
    } else if Q_stricmp("staff", &sType) != 0 {
        getBig = true;
    }

    if !getBig {
        return;
    }

    if let Some(item) = Menu_FindItemByName(&ctx.world.menus, Some(menu), "box2middle") {
        let window = &mut ctx.world.menus.item_mut(item).window;
        window.rect.x = 212.0;
        window.rect.y = 126.0;
        window.rect.w = 219.0;
        window.rect.h = 44.0;
    }

    if let Some(item) = Menu_FindItemByName(&ctx.world.menus, Some(menu), "box2bottom") {
        let window = &mut ctx.world.menus.item_mut(item).window;
        window.rect.x = 212.0;
        window.rect.y = 170.0;
        window.rect.w = 219.0;
        window.rect.h = 60.0;
    }

    if let Some(item) = Menu_FindItemByName(&ctx.world.menus, Some(menu), "box3middle") {
        let window = &mut ctx.world.menus.item_mut(item).window;
        window.rect.x = 418.0;
        window.rect.y = 126.0;
        window.rect.w = 219.0;
        window.rect.h = 44.0;
    }

    if let Some(item) = Menu_FindItemByName(&ctx.world.menus, Some(menu), "box3bottom") {
        let window = &mut ctx.world.menus.item_mut(item).window;
        window.rect.x = 418.0;
        window.rect.y = 170.0;
        window.rect.w = 219.0;
        window.rect.h = 60.0;
    }
}

/// Raven `UI_GetSaberCvars`.
///
/// Source: `oracle/codemp/ui/ui_main.c:6026-6039`
pub fn UI_GetSaberCvars(ctx: &mut UiContext) {
    let saber1 = UI_Cvar_VariableString(ctx, "saber1");
    trap::Cvar_Set(ctx.engine, "ui_saber", &saber1);
    let saber2 = UI_Cvar_VariableString(ctx, "saber2");
    trap::Cvar_Set(ctx.engine, "ui_saber2", &saber2);

    let color1 = trap::Cvar_VariableValue(ctx.engine, "color1") as saber_colors_t;
    match SaberColorToString(color1) {
        Some(s) => trap::Cvar_Set(ctx.engine, "g_saber_color", s),
        None => trap::Cvar_Reset(ctx.engine, "g_saber_color"),
    }
    let color2 = trap::Cvar_VariableValue(ctx.engine, "color2") as saber_colors_t;
    match SaberColorToString(color2) {
        Some(s) => trap::Cvar_Set(ctx.engine, "g_saber2_color", s),
        None => trap::Cvar_Reset(ctx.engine, "g_saber2_color"),
    }

    let g_saber_color = UI_Cvar_VariableString(ctx, "g_saber_color");
    trap::Cvar_Set(ctx.engine, "ui_saber_color", &g_saber_color);
    let g_saber2_color = UI_Cvar_VariableString(ctx, "g_saber2_color");
    trap::Cvar_Set(ctx.engine, "ui_saber2_color", &g_saber2_color);
}

/// Raven `UI_ResetCharacterListBoxes`.
///
/// Source: `oracle/codemp/ui/ui_main.c:6087-6142`
pub fn UI_ResetCharacterListBoxes(world: &mut UiWorld) {
    let menu = match Menu_GetFocused(&world.menus) {
        Some(m) => m,
        None => return,
    };

    for name in ["headlistbox", "torsolistbox", "lowerlistbox", "colorbox"] {
        if let Some(item) = Menu_FindItemByName(&world.menus, Some(menu), name) {
            let itemDef = world.menus.item_mut(item);
            if let Some(listPtr) = itemDef.typeData.listBox_mut() {
                listPtr.cursorPos = 0;
            }
            itemDef.cursorPos = 0;
        }
    }
}

/// Raven `UI_BinaryServerInsertion`.
///
/// Source: `oracle/codemp/ui/ui_main.c:7724-7756`
pub fn UI_BinaryServerInsertion(ctx: &mut UiContext, num: c_int) {
    // use binary search to insert server
    let mut len = ctx.world.serverStatus.displayServers.len() as c_int;
    let mut mid = len;
    let mut offset: c_int = 0;
    let mut res: c_int = 0;

    while mid > 0 {
        mid = len >> 1;

        let source = ctx.world.cvars.ui_netSource.integer;
        let sortKey = ctx.world.serverStatus.sortKey;
        let sortDir = ctx.world.serverStatus.sortDir;
        let s2 = ctx.world.serverStatus.displayServers[(offset + mid) as usize];
        res = trap::LAN_CompareServers(ctx.engine, source, sortKey, sortDir, num, s2);

        // if equal
        if res == 0 {
            UI_InsertServerIntoDisplayList(ctx.world, num, offset + mid);
            return;
        }
        // if larger
        else if res == 1 {
            offset += mid;
            len -= mid;
        }
        // if smaller
        else {
            len -= mid;
        }
    }
    if res == 1 {
        offset += 1;
    }
    UI_InsertServerIntoDisplayList(ctx.world, num, offset);
}

/// Raven `UI_GetServerStatusInfo`.
///
/// PORT-NOTE: Raven walks `info->text` in place with a `char *p`, nulling
/// delimiters as it goes and storing dangling pointers into the same buffer;
/// the port walks an owned `Vec<char>` with a byte-free character cursor
/// (`pos`) and copies each resolved substring instead of aliasing the buffer.
/// `info->pings`/the raw `lines[i][0]` pointer trick for the player index is
/// replaced by formatting the index directly into the owned cell.
///
/// Source: `oracle/codemp/ui/ui_main.c:8048-8139`
pub fn UI_GetServerStatusInfo(
    ctx: &mut UiContext,
    serverAddress: &str,
    info: Option<&mut ServerStatusInfo>,
) -> bool {
    let Some(info) = info else {
        trap::LAN_ServerStatus(ctx.engine, Some(serverAddress), 0);
        return false;
    };

    *info = ServerStatusInfo::default();
    let (status, text) =
        trap::LAN_ServerStatus(ctx.engine, Some(serverAddress), MAX_SERVERSTATUS_TEXT);
    if status == 0 {
        return false;
    }

    // PORT-NOTE: Raven `Q_strncpyz` into `char address[MAX_ADDRESSLENGTH]`.
    info.address = serverAddress.chars().take(MAX_ADDRESSLENGTH - 1).collect();
    info.lines.push([
        "Address".to_string(),
        String::new(),
        String::new(),
        info.address.clone(),
    ]);

    let buf: Vec<char> = text.chars().collect();
    let find_from = |from: usize, needle: char| -> Option<usize> {
        buf[from..]
            .iter()
            .position(|&c| c == needle)
            .map(|off| from + off)
    };

    // get the cvars
    let mut pos: usize = 0;
    loop {
        if pos >= buf.len() {
            pos = buf.len();
            break;
        }
        let bs = match find_from(pos, '\\') {
            Some(i) => i,
            None => {
                pos = buf.len();
                break;
            }
        };
        let after_bs = bs + 1;
        if after_bs < buf.len() && buf[after_bs] == '\\' {
            pos = after_bs;
            break;
        }
        if after_bs >= buf.len() {
            pos = buf.len();
            break;
        }
        let key_start = after_bs;
        let bs2 = match find_from(key_start, '\\') {
            Some(i) => i,
            None => {
                pos = buf.len();
                break;
            }
        };
        let key: String = buf[key_start..bs2].iter().collect();
        let value_start = bs2 + 1;
        let value_end = find_from(value_start, '\\').unwrap_or(buf.len());
        let value: String = buf[value_start..value_end].iter().collect();

        info.lines.push([key, String::new(), String::new(), value]);
        pos = value_start;
        if info.lines.len() >= MAX_SERVERSTATUS_LINES {
            // PORT-NOTE: Raven NUL-terminates a value only on the following
            // iteration, so the cap-break leaves the last value running to the end
            // of `info->text`.
            let tail: String = buf[value_start..].iter().collect();
            if let Some(last) = info.lines.last_mut() {
                last[3] = tail;
            }
            break;
        }
    }

    // get the player list
    if info.lines.len() < MAX_SERVERSTATUS_LINES - 3 {
        // empty line
        info.lines
            .push([String::new(), String::new(), String::new(), String::new()]);
        // header
        info.lines.push([
            "num".to_string(),
            "score".to_string(),
            "ping".to_string(),
            "name".to_string(),
        ]);
        // parse players
        let mut i: c_int = 0;
        loop {
            if pos >= buf.len() {
                break;
            }
            if buf[pos] == '\\' {
                pos += 1;
            }
            if pos >= buf.len() {
                break;
            }
            let score_start = pos;
            let sp1 = match find_from(pos, ' ') {
                Some(o) => o,
                None => break,
            };
            let score: String = buf[score_start..sp1].iter().collect();
            pos = sp1 + 1;

            let ping_start = pos;
            let sp2 = match find_from(pos, ' ') {
                Some(o) => o,
                None => break,
            };
            let ping: String = buf[ping_start..sp2].iter().collect();
            pos = sp2 + 1;

            let name_start = pos;
            let name_end = find_from(name_start, '\\').unwrap_or(buf.len());
            let name: String = buf[name_start..name_end].iter().collect();

            info.lines.push([format!("{}", i), score, ping, name]);
            if info.lines.len() >= MAX_SERVERSTATUS_LINES {
                // PORT-NOTE: Raven NUL-terminates the name only after this cap
                // check, so the cap-break leaves it running to the end of
                // `info->text`.
                let tail: String = buf[name_start..].iter().collect();
                if let Some(last) = info.lines.last_mut() {
                    last[3] = tail;
                }
                break;
            }

            if name_end >= buf.len() {
                break;
            }
            pos = name_end + 1;
            i += 1;
        }
    }

    UI_SortServerStatusInfo(info);
    true
}

/// Raven `UI_StopCinematic`.
///
/// Source: `oracle/codemp/ui/ui_main.c:10188-10213`
pub fn UI_StopCinematic(ctx: &mut UiContext, handle: c_int) {
    if handle >= 0 {
        trap::CIN_StopCinematic(ctx.engine, handle);
    } else {
        let handle = handle.abs();
        if handle == UI_MAPCINEMATIC {
            let idx = ctx.world.cvars.ui_currentMap.integer as usize;
            if ctx.world.mapList[idx].cinematic >= 0 {
                let cinematic = ctx.world.mapList[idx].cinematic;
                trap::CIN_StopCinematic(ctx.engine, cinematic);
                ctx.world.mapList[idx].cinematic = -1;
            }
        } else if handle == UI_NETMAPCINEMATIC {
            if ctx.world.serverStatus.currentServerCinematic >= 0 {
                let cinematic = ctx.world.serverStatus.currentServerCinematic;
                trap::CIN_StopCinematic(ctx.engine, cinematic);
                ctx.world.serverStatus.currentServerCinematic = -1;
            }
        } else if handle == UI_CLANCINEMATIC {
            let teamName = UI_Cvar_VariableString(ctx, "ui_teamName");
            let i = UI_TeamIndexFromName(ctx.world, &teamName);
            if i >= 0 && (i as usize) < ctx.world.teamList.len() {
                let idx = i as usize;
                if ctx.world.teamList[idx].cinematic >= 0 {
                    let cinematic = ctx.world.teamList[idx].cinematic;
                    trap::CIN_StopCinematic(ctx.engine, cinematic);
                    ctx.world.teamList[idx].cinematic = -1;
                }
            }
        }
    }
}

/// Raven `UI_BuildQ3Model_List`.
///
/// PORT-NOTE: the `/*...*/`-commented-out `fpath`/`trap_FS_FOpenFile` probe
/// (superseded by `bIsImageFile`, per Raven's own comment) is dead and is
/// dropped, matching Raven's compiled-out behavior.
///
/// Source: `oracle/codemp/ui/ui_main.c:10330-10441`
pub fn UI_BuildQ3Model_List(ctx: &mut UiContext) {
    ctx.world.q3HeadNames.clear();
    ctx.world.q3HeadIcons.clear();

    // iterate directory of all player models
    let mut dirlist = vec![0u8; 2048];
    let numdirs = trap::FS_GetFileList(ctx.engine, "models/players", "/", &mut dirlist);
    let dirnames = latin1_to_string(&dirlist);
    let mut dirptrs = dirnames.split('\0');

    let mut i = 0;
    while i < numdirs && ctx.world.q3HeadNames.len() < MAX_Q3PLAYERMODELS {
        let dirptr_raw = match dirptrs.next() {
            Some(d) => d,
            None => break,
        };
        let dirptr = dirptr_raw.strip_suffix('/').unwrap_or(dirptr_raw);

        if dirptr == "." || dirptr == ".." {
            i += 1;
            continue;
        }

        let mut filelist = vec![0u8; 2048];
        let numfiles = trap::FS_GetFileList(
            ctx.engine,
            &format!("models/players/{}", dirptr),
            "skin",
            &mut filelist,
        );
        let filenames = latin1_to_string(&filelist);
        let mut fileptrs = filenames.split('\0');

        let mut j = 0;
        while j < numfiles && ctx.world.q3HeadNames.len() < MAX_Q3PLAYERMODELS {
            let fileptr = match fileptrs.next() {
                Some(f) => f,
                None => break,
            };

            let mut skinname = COM_StripExtension(fileptr);
            if let Some(k) = skinname.find('_') {
                skinname = skinname[k..].to_string();
            }

            // PORT-NOTE (§19): Raven takes `&skinname[1]` unconditionally (past the
            // terminator when the stripped name is empty); the empty case is skipped.
            if !skinname.is_empty() {
                // Raven `check = &skinname[1]` — skip exactly one character.
                let check: String = skinname.chars().skip(1).collect();
                if bIsImageFile(ctx, dirptr, &check) {
                    // if it exists
                    if skinname.starts_with('_') {
                        // change character to append properly
                        skinname.replace_range(0..1, "/");
                    }

                    let candidate = format!("{}{}", dirptr, skinname);
                    // check for dupes
                    let iconExists = ctx
                        .world
                        .q3HeadNames
                        .iter()
                        .any(|n| Q_stricmp(&candidate, n) == 0);

                    if !iconExists {
                        // PORT-NOTE: Raven `Com_sprintf` into `q3HeadNames[i][64]`.
                        let candidate: String = candidate.chars().take(63).collect();
                        ctx.world.q3HeadNames.push(candidate);
                        // rww - we are now registering them as they are drawn like the
                        // TA feeder, so as to decrease UI load time.
                        ctx.world.q3HeadIcons.push(0);
                    }
                }
            }

            if ctx.world.q3HeadNames.len() >= MAX_Q3PLAYERMODELS {
                return;
            }
            j += 1;
        }
        i += 1;
    }
}

// DEFERRED: UI_SiegeInit — its bg calls' real ported shapes
// (`BG_SiegeLoadClasses(descBuffer: *mut siegeClassDesc_t, bg: &mut BgState,
// traps: &dyn BgTraps, callbacks: &mut dyn GameCallbacks)`,
// `BG_SiegeLoadTeams(bg: &mut BgState, traps: &dyn BgTraps)`) require a
// `&dyn BgTraps` and a `&mut dyn GameCallbacks` instance; ui has neither an
// implementor nor a wiring point for either trait yet (DEC-36 addendum 11/12
// name the traits, but `impl BgTraps for ...` / `impl GameCallbacks for ...`
// have not landed in `mp_ui`), and `g_UIClassDescriptions` is an owned
// `Vec<String>` with no `*mut siegeClassDesc_t` buffer to hand `descBuffer`.
// Source: `oracle/codemp/ui/ui_main.c:10443-10460`
// Source: `crates/mp/bg/src/bg_saga.rs:1477-1492` (real `BG_SiegeLoadClasses` shape)
// Source: `crates/mp/bg/src/bg_channel/bg_traps.rs:21` (`trait BgTraps`)
// Source: `crates/mp/bg/src/bg_channel/game_callbacks.rs:21` (`trait GameCallbacks`)

/// Raven `UI_BuildPlayerModel_List`.
///
/// PORT-NOTE: the `trap_Cvar_VariableValue("fs_copyfiles") > 0` `.skin`
/// re-open/close probe (a filesystem cache-warm side effect with no
/// observable state change) is transcribed as a fire-and-discard trap pair,
/// matching Raven's own discarded `f`.
///
/// Source: `oracle/codemp/ui/ui_main.c:10515-10654`
pub fn UI_BuildPlayerModel_List(ctx: &mut UiContext, inGameLoad: bool) {
    ctx.world.playerSpecies.clear();
    ctx.world.playerSpeciesIndex = 0;

    // iterate directory of all player models
    let mut dirlist = vec![0u8; 2048];
    let numdirs = trap::FS_GetFileList(ctx.engine, "models/players", "/", &mut dirlist);
    let dirnames = latin1_to_string(&dirlist);
    let mut dirptrs = dirnames.split('\0');

    let mut i = 0;
    while i < numdirs {
        let dirptr_raw = match dirptrs.next() {
            Some(d) => d,
            None => break,
        };
        // Raven tests `dirlen` on the raw entry, then strips one trailing '/'.
        if dirptr_raw.is_empty() {
            i += 1;
            continue;
        }
        let dirptr = dirptr_raw
            .strip_suffix('/')
            .unwrap_or(dirptr_raw)
            .to_string();

        if dirptr == "." || dirptr == ".." {
            i += 1;
            continue;
        }

        let fpath = format!("models/players/{}/PlayerChoice.txt", dirptr);
        let mut f: fileHandle_t = 0;
        let filelen = trap::FS_FOpenFile(ctx.engine, &fpath, &mut f, FS_READ);

        if f != 0 {
            // PORT-NOTE (§19): Raven freads into `char buffer[2048]` (overrun for
            // larger files, embedded NUL ends the parse); the whole-file read takes
            // the defined behavior.
            let mut buffer = vec![0u8; filelen as usize];
            trap::FS_Read(ctx.engine, &mut buffer, f);
            trap::FS_FCloseFile(ctx.engine, f);
            let buffer = latin1_to_string(&buffer);
            // Raven's `buffer[filelen] = 0` NUL-terminates; a NUL inside the file
            // ends the parse there.
            let buffer = match buffer.find('\0') {
                Some(n) => buffer[..n].to_string(),
                None => buffer,
            };

            // record this species
            let mut species = PlayerSpeciesInfo {
                // PORT-NOTE: Raven `Q_strncpyz` into `playerSpecies[].Name[64]`.
                Name: dirptr.chars().take(63).collect(),
                ..PlayerSpeciesInfo::default()
            };

            if !UI_ParseColorData(&mut ctx.world.bg_state.qs, &buffer, &mut species, &fpath) {
                Com_Printf(
                    ctx,
                    &format!(
                        "{}UI_BuildPlayerModel_List: Errors parsing '{}'\n",
                        S_COLOR_RED.to_str().unwrap(),
                        fpath
                    ),
                );
            }

            let mut filelist = vec![0u8; 2048];
            let numfiles = trap::FS_GetFileList(
                ctx.engine,
                &format!("models/players/{}", dirptr),
                ".skin",
                &mut filelist,
            );
            let filenames = latin1_to_string(&filelist);
            let mut fileptrs = filenames.split('\0');

            let mut iSkinParts: c_int = 0;
            let mut j = 0;
            while j < numfiles {
                let fileptr = match fileptrs.next() {
                    Some(f) => f,
                    None => break,
                };

                if trap::Cvar_VariableValue(ctx.engine, "fs_copyfiles") > 0.0 {
                    let mut f2: fileHandle_t = 0;
                    trap::FS_FOpenFile(
                        ctx.engine,
                        &format!("models/players/{}/{}", dirptr, fileptr),
                        &mut f2,
                        FS_READ,
                    );
                    if f2 != 0 {
                        trap::FS_FCloseFile(ctx.engine, f2);
                    }
                }

                let skinname = COM_StripExtension(fileptr);

                if bIsImageFile(ctx, &dirptr, &skinname) {
                    // if it exists.
                    // PORT-NOTE: Raven `Q_strncpyz` into `Skin*Names[][16]`.
                    let stored: String = skinname.chars().take(15).collect();
                    if Q_stricmpn(&skinname, "head_", 5) == 0 {
                        if species.SkinHeadNames.len() < MAX_PLAYERMODELS {
                            species.SkinHeadNames.push(stored);
                            iSkinParts |= 1 << 0;
                        }
                    } else if Q_stricmpn(&skinname, "torso_", 6) == 0 {
                        if species.SkinTorsoNames.len() < MAX_PLAYERMODELS {
                            species.SkinTorsoNames.push(stored);
                            iSkinParts |= 1 << 1;
                        }
                    } else if Q_stricmpn(&skinname, "lower_", 6) == 0 {
                        if species.SkinLegNames.len() < MAX_PLAYERMODELS {
                            species.SkinLegNames.push(stored);
                            iSkinParts |= 1 << 2;
                        }
                    }
                }
                j += 1;
            }

            if iSkinParts != 7 {
                // didn't get a skin for each, then skip this model.
                i += 1;
                continue;
            }

            ctx.world.playerSpecies.push(species);
            if !inGameLoad && ctx.world.cvars.ui_PrecacheModels.integer != 0 {
                let mut ghoul2: *mut c_void = core::ptr::null_mut();
                let modelPath = format!("models/players/{}/model.glm", dirptr);
                let g2Model =
                    trap::G2API_InitGhoul2Model(ctx.engine, &mut ghoul2, &modelPath, 0, 0, 0, 0, 0);
                if g2Model >= 0 {
                    trap::G2API_CleanGhoul2Models(ctx.engine, &mut ghoul2);
                }
            }

            if ctx.world.playerSpecies.len() >= MAX_PLAYERMODELS {
                return;
            }
        }
        i += 1;
    }
}

/// Raven `_UI_IsFullscreen`.
///
/// Source: `oracle/codemp/ui/ui_main.c:11030-11032`
pub fn _UI_IsFullscreen(world: &UiWorld) -> bool {
    Menus_AnyFullScreenVisible(&world.menus)
}

/// Raven `UI_StopServerRefresh`.
///
/// Source: `oracle/codemp/ui/ui_main.c:11569-11588`
pub fn UI_StopServerRefresh(ctx: &mut UiContext) {
    if !ctx.world.serverStatus.refreshActive {
        // not currently refreshing
        return;
    }
    ctx.world.serverStatus.refreshActive = false;
    Com_Printf(
        ctx,
        &format!(
            "{} servers listed in browser with {} players.\n",
            ctx.world.serverStatus.displayServers.len(),
            ctx.world.serverStatus.numPlayersOnServers
        ),
    );
    let count = trap::LAN_GetServerCount(ctx.engine, ctx.world.cvars.ui_netSource.integer);
    let numDisplayServers = ctx.world.serverStatus.displayServers.len() as c_int;
    if count - numDisplayServers > 0 {
        let maxPing = trap::Cvar_VariableValue(ctx.engine, "cl_maxPing") as c_int;
        Com_Printf(
            ctx,
            &format!(
                "{} servers not listed due to filters, packet loss, or pings higher than {}\n",
                count - numDisplayServers,
                maxPing
            ),
        );
    }
}
