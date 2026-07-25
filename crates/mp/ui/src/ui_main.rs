//! `ui_main.c` — the ui module's main logic (ownerdraws, feeders, menu
//! scripts, server browser).
//!
//! Source: `oracle/codemp/ui/ui_main.c`

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_abi::ui::public::ui_client_state_t::uiClientState_t;
use mp_bg::bg_channel::BgState;
use mp_bg::public::configstring::{CS_PLAYERS, CS_SERVERINFO};
use mp_bg::public::gametype::{
    GT_DUEL, GT_FFA, GT_HOLOCRON, GT_JEDIMASTER, GT_POWERDUEL, GT_SINGLE_PLAYER, GT_TEAM,
};
use mp_bg::public::team::{TEAM_BLUE, TEAM_RED, TEAM_SPECTATOR};
use mp_bg::saga::siege_class_t::siegeClass_t;
use mp_bg::weapons::weapon_t::{WP_NONE, WP_NUM_WEAPONS, WP_SABER};
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
    connstate_t, fileHandle_t, qhandle_t, vec4_t, FS_READ, MAX_CLIENTS, MAX_INFO_STRING, MAX_QPATH,
    MAX_STRING_CHARS, Q3_VERSION,
};
use mp_uishared::shared::display_context::DisplayContext;
use mp_uishared::shared::menu_system::MAX_MENUFILE;
use mp_uishared::shared::rect_def_t::RectDef;
use native_string::{atoi, latin1_to_string, Info_ValueForKey, Q_CleanStr, Q_stricmp};

use crate::keycodes::fake_ascii_t::fakeAscii_t;
use crate::local::player_species_info_t::PlayerSpeciesInfo;
use crate::local::server_status_info_t::ServerStatusInfo;
use crate::trap;
use crate::world::ui_context::UiContext;
use crate::world::ui_cvars::UiCvars;
use crate::world::ui_world::{UiWorld, MAX_FORCE_CONFIGS};

/// Raven `#define AS_FAVORITES 2` (with `AS_LOCAL`/`AS_GLOBAL`/`AS_MPLAYER`) —
/// the `ui_netSource` server-browser source selector. No canonical qshared
/// home ported yet, so this stays a file-local const.
///
/// Source: `oracle/codemp/game/q_shared.h:3025-3029`
const AS_FAVORITES: c_int = 2;

/// Raven `menudef.h` `UI_SHOW_*` ownerDraw visibility bitflags
/// (`UI_OwnerDrawVisible`'s `flags` argument).
///
/// Source: `oracle/ui/menudef.h:144-156`
const UI_SHOW_LEADER: c_int = 0x0000_0001;
const UI_SHOW_NOTLEADER: c_int = 0x0000_0002;
const UI_SHOW_FAVORITESERVERS: c_int = 0x0000_0004;
const UI_SHOW_ANYNONTEAMGAME: c_int = 0x0000_0008;
const UI_SHOW_ANYTEAMGAME: c_int = 0x0000_0010;
const UI_SHOW_NEWHIGHSCORE: c_int = 0x0000_0020;
const UI_SHOW_DEMOAVAILABLE: c_int = 0x0000_0040;
const UI_SHOW_NEWBESTTIME: c_int = 0x0000_0080;
const UI_SHOW_FFA: c_int = 0x0000_0100;
const UI_SHOW_NOTFFA: c_int = 0x0000_0200;
const UI_SHOW_NETANYNONTEAMGAME: c_int = 0x0000_0400;
const UI_SHOW_NETANYTEAMGAME: c_int = 0x0000_0800;
const UI_SHOW_NOTFAVORITESERVERS: c_int = 0x0000_1000;

/// Raven `#define CIN_loop 2` / `#define CIN_silent 8` (`e_status` playback
/// bits passed to `trap_CIN_PlayCinematic`). No canonical qshared home ported
/// yet, so these stay file-local consts.
///
/// Source: `oracle/codemp/game/q_shared.h:516-518`
const CIN_LOOP: c_int = 2;
const CIN_SILENT: c_int = 8;

/// Raven `#define KEYCATCH_UI 0x0002` — the key-catcher bit `trap_Key_SetCatcher`
/// sets while a ui menu owns input. No canonical qshared home ported yet, so
/// this stays a file-local const.
///
/// Source: `oracle/codemp/game/q_shared.h:1937`
const KEYCATCH_UI: c_int = 0x0002;

/// Raven `static const int numSkillLevels = sizeof(skillLevels) /
/// sizeof(const char*)` — `skillLevels[]` (`ui_main.c:902-908`) has 5 rows;
/// the table itself is compiled-in data that lands beside the fn that reads
/// it (PORT-NOTE, `UiMainState`), so only the derived count is needed here.
///
/// Source: `oracle/codemp/ui/ui_main.c:902-909`
const NUM_SKILL_LEVELS: c_int = 5;

// DEFERRED: UI_AnimsetAlloc — part of the ui_main.c hand-maintained animation
// fork (`bgAllAnims`/`uiNumAllAnims`/`UI_ParseAnimationFile`); DEC-36 D5 rules
// ui reuses mp_bg's animation module instead of Raven's manually synced copy
// (see `UiMainState`'s PORT-NOTE, which drops the same fork's state fields).
// Source: `oracle/codemp/ui/ui_main.c:645-651`

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
        let mut fileptrs = names.split('\0').filter(|s| !s.is_empty());

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
