//! Port of `oracle/codemp/cgame/cg_info.c` — the loading screen and connection info display. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;

use native_string::{
    atoi, buf_to_string, strncpyz_string, Info_ValueForKey, Q_CleanStr, Q_strncpyz,
};

use mp_bg::public::bg_itemlist::bg_itemlist;
use mp_bg::public::configstring::{CS_MESSAGE, CS_MOTD, CS_PLAYERS, CS_SERVERINFO, CS_SYSTEMINFO};
use mp_bg::public::force_mastery::NUM_FORCE_MASTERY_LEVELS;
use mp_bg::public::gametype::{
    GT_CTF, GT_CTY, GT_DUEL, GT_FFA, GT_HOLOCRON, GT_JEDIMASTER, GT_POWERDUEL, GT_SIEGE,
    GT_SINGLE_PLAYER, GT_TEAM,
};
use mp_qshared::shared::screen::{SCREEN_HEIGHT, SCREEN_WIDTH};
use mp_qshared::shared::{colorWhite, qhandle_t, MAX_QPATH, MAX_STRING_CHARS};
use mp_uishared::shared::display_state::DisplayState;

use crate::cg_drawtools::{CG_DrawPic, UI_DrawProportionalString};
use crate::cg_main::{CG_ConfigString, CG_GetStringEdString};
use crate::trap;
use crate::world::cg_context::CgContext;

/// Raven `#define MAX_LOADING_PLAYER_ICONS 16`.
/// Source: `oracle/codemp/cgame/cg_info.c:7`
pub const MAX_LOADING_PLAYER_ICONS: usize = 16;

/// Raven `#define MAX_LOADING_ITEM_ICONS 26`.
/// Source: `oracle/codemp/cgame/cg_info.c:8`
pub const MAX_LOADING_ITEM_ICONS: usize = 26;

// UI_* style flags for `UI_DrawProportionalString`. q_shared.h `#define`s land
// per-TU (same pattern cg_draw.rs/cg_drawtools.rs already follow) since they're
// C macros, not a shared Rust binding.
// Source: `oracle/codemp/game/q_shared.h:489,493,495`
const UI_CENTER: c_int = 0x0000_0001;
const UI_BIGFONT: c_int = 0x0000_0020;
const UI_DROPSHADOW: c_int = 0x0000_0800;

/// Raven `#define UI_INFOFONT (UI_BIGFONT)`.
/// Source: `oracle/codemp/cgame/cg_info.c:109`
const UI_INFOFONT: c_int = UI_BIGFONT;

/// Raven `char *forceMasteryLevels[NUM_FORCE_MASTERY_LEVELS]` — `bg_misc.c`
/// compiled-in data. cgame compiles `bg_misc.c` into its own link unit and
/// this table has no Rust port reachable from `mp_bg` yet (same situation as
/// `mp_ui`'s copy in `ui_main.rs`), so it lands beside the one cgame fn that
/// reads it.
///
/// Source: `oracle/codemp/game/bg_misc.c:150-160`
const FORCE_MASTERY_LEVELS: [&str; 8] = [
    "MASTERY0", "MASTERY1", "MASTERY2", "MASTERY3", "MASTERY4", "MASTERY5", "MASTERY6", "MASTERY7",
];

/// Runtime `va()`-style substitution for a format string that is data rather
/// than a Rust `format!` literal — StringEd templates fetched at runtime that
/// carry their own `%s`. Walks the template once, replacing each `%s`/`%d`/`%i`
/// conversion in the order it appears with the next argument.
///
/// Port-local helper — no Raven counterpart. Mirrors `mp_ui`'s `ui_main.rs`
/// `va_runtime` (same problem, different crate). Bare `%i`/`%d`/`%s` only.
fn va_runtime(template: &str, args: &[&str]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut chars = template.chars().peekable();
    let mut arg_iter = args.iter();
    while let Some(c) = chars.next() {
        if c == '%' {
            if let Some(&next) = chars.peek() {
                if next == 'd' || next == 'i' || next == 's' {
                    chars.next();
                    if let Some(a) = arg_iter.next() {
                        out.push_str(a);
                        continue;
                    }
                }
            }
        }
        out.push(c);
    }
    out
}

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

/// Raven `CG_LoadingClient` — pulls the connecting client's personality name
/// out of its configstring and posts it as the loading-screen status text.
///
/// Raven's per-client loading icon block (registering `icon_<model>_<skin>.tga`
/// into `loadingPlayerIcons`) and the singleplayer announce-sound block are
/// both `/* */`-commented out in the oracle; left unported to match, per
/// porting-rules §A2 (transcribe what Raven actually runs).
///
/// Source: `oracle/codemp/cgame/cg_info.c:53-98`
pub fn CG_LoadingClient(ctx: &mut CgContext, clientNum: c_int) {
    let info = CG_ConfigString(ctx, CS_PLAYERS + clientNum);

    let personality = strncpyz_string(Info_ValueForKey(&info, "n").as_bytes(), MAX_QPATH);
    let personality = Q_CleanStr(&personality);

    CG_LoadingString(ctx, &personality);
}

/// Raven `CG_DrawInformation` — the pre-map loading screen: levelshot
/// background, load bar, connecting/awaiting-snapshot status line, then the
/// server hostname/motd/gametype/rules block info panel underneath.
///
/// Source: `oracle/codemp/cgame/cg_info.c:110-432`
pub fn CG_DrawInformation(ctx: &mut CgContext, ds: &DisplayState) {
    // I know, this is total crap, but as a post release asian-hack....  -Ste
    let iPropHeight: c_int = 18;

    let info = CG_ConfigString(ctx, CS_SERVERINFO);
    let sysInfo = CG_ConfigString(ctx, CS_SYSTEMINFO);

    let s = Info_ValueForKey(&info, "mapname");
    let mut levelshot: qhandle_t =
        trap::R_RegisterShaderNoMip(ctx.engine, &format!("levelshots/{s}"));
    if levelshot == 0 {
        levelshot = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/unknownmap_mp");
    }
    trap::R_SetColor(ctx.engine, None);
    CG_DrawPic(
        ctx,
        0.0,
        0.0,
        SCREEN_WIDTH as f32,
        SCREEN_HEIGHT as f32,
        levelshot,
    );

    CG_LoadBar(ctx);

    // Raven's CG_DrawLoadingIcons() call for the icons of things as they are
    // loaded is `//`-commented out in the oracle; left unported to match.

    // the first 150 rows are reserved for the client connection screen to
    // write into
    if ctx.world.cg.infoScreenText[0] != 0 {
        let psLoading = CG_GetStringEdString(ctx, "MENUS", "LOADING_MAPNAME");
        let infoScreenText = buf_to_string(&ctx.world.cg.infoScreenText.map(|c| c as u8));
        let text = va_runtime(&psLoading, &[&infoScreenText]);
        UI_DrawProportionalString(
            ctx,
            ds,
            320,
            128 - 32,
            &text,
            UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
            colorWhite,
        );
    } else {
        let psAwaitingSnapshot = CG_GetStringEdString(ctx, "MENUS", "AWAITING_SNAPSHOT");
        UI_DrawProportionalString(
            ctx,
            ds,
            320,
            128 - 32,
            &psAwaitingSnapshot,
            UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
            colorWhite,
        );
    }

    // draw info string information
    let mut y: c_int = 180 - 32;

    // don't print server lines if playing a local game
    let buf = trap::Cvar_VariableStringBuffer(ctx.engine, "sv_running", 1024);
    if atoi(&buf) == 0 {
        // server hostname
        let hostname = strncpyz_string(Info_ValueForKey(&info, "sv_hostname").as_bytes(), 1024);
        let hostname = Q_CleanStr(&hostname);
        UI_DrawProportionalString(
            ctx,
            ds,
            320,
            y,
            &hostname,
            UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
            colorWhite,
        );
        y += iPropHeight;

        // pure server
        let s = Info_ValueForKey(&sysInfo, "sv_pure");
        if s.starts_with('1') {
            let psPure = CG_GetStringEdString(ctx, "MP_INGAME", "PURE_SERVER");
            UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y,
                &psPure,
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite,
            );
            y += iPropHeight;
        }

        // server-specific message of the day
        let s = CG_ConfigString(ctx, CS_MOTD);
        if !s.is_empty() {
            UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y,
                &s,
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite,
            );
            y += iPropHeight;
        }

        // display global MOTD at bottom (mirrors ui_main UI_DrawConnectScreen
        {
            let motdString = trap::Cvar_VariableStringBuffer(ctx.engine, "cl_motdString", 1024);
            if !motdString.is_empty() {
                UI_DrawProportionalString(
                    ctx,
                    ds,
                    320,
                    425,
                    &motdString,
                    UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                    colorWhite,
                );
            }
        }

        // some extra space after hostname and motd
        y += 10;
    }

    // map-specific message (long map name)
    let s = CG_ConfigString(ctx, CS_MESSAGE);
    if !s.is_empty() {
        UI_DrawProportionalString(
            ctx,
            ds,
            320,
            y,
            &s,
            UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
            colorWhite,
        );
        y += iPropHeight;
    }

    // cheats warning
    let s = Info_ValueForKey(&sysInfo, "sv_cheats");
    if s.starts_with('1') {
        let cheatsAreEnabled = CG_GetStringEdString(ctx, "MP_INGAME", "CHEATSAREENABLED");
        UI_DrawProportionalString(
            ctx,
            ds,
            320,
            y,
            &cheatsAreEnabled,
            UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
            colorWhite,
        );
        y += iPropHeight;
    }

    // game type
    let s = match ctx.world.cgs.gametype {
        GT_FFA => CG_GetStringEdString(ctx, "MENUS", "FREE_FOR_ALL"),
        GT_HOLOCRON => CG_GetStringEdString(ctx, "MENUS", "HOLOCRON_FFA"),
        GT_JEDIMASTER => CG_GetStringEdString(ctx, "MENUS", "SAGA"), //"Jedi Master";??
        GT_SINGLE_PLAYER => CG_GetStringEdString(ctx, "MENUS", "SAGA"), //"Team FFA";
        GT_DUEL => CG_GetStringEdString(ctx, "MENUS", "DUEL"),
        GT_POWERDUEL => CG_GetStringEdString(ctx, "MENUS", "POWERDUEL"),
        GT_TEAM => CG_GetStringEdString(ctx, "MENUS", "TEAM_FFA"),
        GT_SIEGE => CG_GetStringEdString(ctx, "MENUS", "SIEGE"),
        GT_CTF => CG_GetStringEdString(ctx, "MENUS", "CAPTURE_THE_FLAG"),
        GT_CTY => CG_GetStringEdString(ctx, "MENUS", "CAPTURE_THE_YSALIMARI"),
        _ => CG_GetStringEdString(ctx, "MENUS", "SAGA"), //"Team FFA";
    };
    UI_DrawProportionalString(
        ctx,
        ds,
        320,
        y,
        &s,
        UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
        colorWhite,
    );
    y += iPropHeight;

    if ctx.world.cgs.gametype != GT_SIEGE {
        let value = atoi(&Info_ValueForKey(&info, "timelimit"));
        if value != 0 {
            let label = CG_GetStringEdString(ctx, "MP_INGAME", "TIMELIMIT");
            UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y,
                &format!("{label} {value}"),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite,
            );
            y += iPropHeight;
        }

        if ctx.world.cgs.gametype < GT_CTF {
            let value = atoi(&Info_ValueForKey(&info, "fraglimit"));
            if value != 0 {
                let label = CG_GetStringEdString(ctx, "MP_INGAME", "FRAGLIMIT");
                UI_DrawProportionalString(
                    ctx,
                    ds,
                    320,
                    y,
                    &format!("{label} {value}"),
                    UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                    colorWhite,
                );
                y += iPropHeight;
            }

            if ctx.world.cgs.gametype == GT_DUEL || ctx.world.cgs.gametype == GT_POWERDUEL {
                let value = atoi(&Info_ValueForKey(&info, "duel_fraglimit"));
                if value != 0 {
                    let label = CG_GetStringEdString(ctx, "MP_INGAME", "WINLIMIT");
                    UI_DrawProportionalString(
                        ctx,
                        ds,
                        320,
                        y,
                        &format!("{label} {value}"),
                        UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                        colorWhite,
                    );
                    y += iPropHeight;
                }
            }
        }
    }

    if ctx.world.cgs.gametype >= GT_CTF {
        let value = atoi(&Info_ValueForKey(&info, "capturelimit"));
        if value != 0 {
            let label = CG_GetStringEdString(ctx, "MP_INGAME", "CAPTURELIMIT");
            UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y,
                &format!("{label} {value}"),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite,
            );
            y += iPropHeight;
        }
    }

    if ctx.world.cgs.gametype >= GT_TEAM {
        let value = atoi(&Info_ValueForKey(&info, "g_forceBasedTeams"));
        if value != 0 {
            let label = CG_GetStringEdString(ctx, "MP_INGAME", "FORCEBASEDTEAMS");
            UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y,
                &label,
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite,
            );
            y += iPropHeight;
        }
    }

    if ctx.world.cgs.gametype != GT_SIEGE {
        let valueNOFP = atoi(&Info_ValueForKey(&info, "g_forcePowerDisable"));
        let value = atoi(&Info_ValueForKey(&info, "g_maxForceRank"));

        if value > 0 && valueNOFP == 0 && value < NUM_FORCE_MASTERY_LEVELS {
            let fmStr = trap::SP_GetStringTextString(ctx.engine, "MP_INGAME_MAXFORCERANK", 1024)
                .unwrap_or_else(|| "??MP_INGAME_MAXFORCERANK".to_string());
            // §F19: Raven indexes with the raw, server-derived `g_maxForceRank`
            // int with no lower-bound check — a negative value is a C OOB read
            // (UB). The `> 0` guard above skips the line instead: the value is
            // server-supplied, so a panic would be a remote crash trigger.
            let level =
                CG_GetStringEdString(ctx, "MP_INGAME", FORCE_MASTERY_LEVELS[value as usize]);
            UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y,
                &format!("{fmStr} {level}"),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite,
            );
            y += iPropHeight;
        } else if valueNOFP == 0 {
            let fmStr = trap::SP_GetStringTextString(ctx.engine, "MP_INGAME_MAXFORCERANK", 1024)
                .unwrap_or_else(|| "??MP_INGAME_MAXFORCERANK".to_string());
            let level = CG_GetStringEdString(ctx, "MP_INGAME", FORCE_MASTERY_LEVELS[7]);
            UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y,
                &format!("{fmStr} {level}"),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite,
            );
            y += iPropHeight;
        }

        let value = if ctx.world.cgs.gametype == GT_DUEL || ctx.world.cgs.gametype == GT_POWERDUEL {
            atoi(&Info_ValueForKey(&info, "g_duelWeaponDisable"))
        } else {
            atoi(&Info_ValueForKey(&info, "g_weaponDisable"))
        };
        if ctx.world.cgs.gametype != GT_JEDIMASTER && value != 0 {
            let saberOnly = CG_GetStringEdString(ctx, "MP_INGAME", "SABERONLYSET");
            UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y,
                &saberOnly,
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite,
            );
            y += iPropHeight;
        }

        if valueNOFP != 0 {
            let noFpSet = CG_GetStringEdString(ctx, "MP_INGAME", "NOFPSET");
            UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y,
                &noFpSet,
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite,
            );
            y += iPropHeight;
        }
    }

    // Display the rules based on type.
    // Raven writes `y += iPropHeight` after the last draw in every arm below;
    // since the fn returns right after the switch, that final write is dead —
    // dropped in each arm (the mid-arm increments between two draws are real).
    y += iPropHeight;
    match ctx.world.cgs.gametype {
        GT_FFA => {
            let l1 = CG_GetStringEdString(ctx, "MP_INGAME", "RULES_FFA_1");
            UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y,
                &l1,
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite,
            );
        }
        GT_HOLOCRON => {
            let l1 = CG_GetStringEdString(ctx, "MP_INGAME", "RULES_HOLO_1");
            UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y,
                &l1,
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite,
            );
            y += iPropHeight;
            let l2 = CG_GetStringEdString(ctx, "MP_INGAME", "RULES_HOLO_2");
            UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y,
                &l2,
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite,
            );
        }
        GT_JEDIMASTER => {
            let l1 = CG_GetStringEdString(ctx, "MP_INGAME", "RULES_JEDI_1");
            UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y,
                &l1,
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite,
            );
            y += iPropHeight;
            let l2 = CG_GetStringEdString(ctx, "MP_INGAME", "RULES_JEDI_2");
            UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y,
                &l2,
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite,
            );
        }
        GT_SINGLE_PLAYER => {}
        GT_DUEL => {
            let l1 = CG_GetStringEdString(ctx, "MP_INGAME", "RULES_DUEL_1");
            UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y,
                &l1,
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite,
            );
            y += iPropHeight;
            let l2 = CG_GetStringEdString(ctx, "MP_INGAME", "RULES_DUEL_2");
            UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y,
                &l2,
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite,
            );
        }
        GT_POWERDUEL => {
            let l1 = CG_GetStringEdString(ctx, "MP_INGAME", "RULES_POWERDUEL_1");
            UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y,
                &l1,
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite,
            );
            y += iPropHeight;
            let l2 = CG_GetStringEdString(ctx, "MP_INGAME", "RULES_POWERDUEL_2");
            UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y,
                &l2,
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite,
            );
        }
        GT_TEAM => {
            let l1 = CG_GetStringEdString(ctx, "MP_INGAME", "RULES_TEAM_1");
            UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y,
                &l1,
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite,
            );
            y += iPropHeight;
            let l2 = CG_GetStringEdString(ctx, "MP_INGAME", "RULES_TEAM_2");
            UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y,
                &l2,
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite,
            );
        }
        GT_SIEGE => {}
        GT_CTF => {
            let l1 = CG_GetStringEdString(ctx, "MP_INGAME", "RULES_CTF_1");
            UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y,
                &l1,
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite,
            );
            y += iPropHeight;
            let l2 = CG_GetStringEdString(ctx, "MP_INGAME", "RULES_CTF_2");
            UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y,
                &l2,
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite,
            );
        }
        GT_CTY => {
            let l1 = CG_GetStringEdString(ctx, "MP_INGAME", "RULES_CTY_1");
            UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y,
                &l1,
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite,
            );
            y += iPropHeight;
            let l2 = CG_GetStringEdString(ctx, "MP_INGAME", "RULES_CTY_2");
            UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y,
                &l2,
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite,
            );
        }
        _ => {}
    }
}
