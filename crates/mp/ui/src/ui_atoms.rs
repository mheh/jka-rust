//! `ui_atoms.c` — ui module utility functions.
//!
//! Source: `oracle/codemp/ui/ui_atoms.c`

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_bg::public::configstring::CS_SERVERINFO;
use mp_bg::public::gametype::GT_SIEGE;
use mp_qshared::shared::cbuf_exec::cbufExec_t;
use mp_qshared::shared::fileHandle_t;
use mp_qshared::shared::keycatch::KEYCATCH_UI;
use mp_qshared::shared::qhandle_t;
use mp_qshared::shared::vec4_t;
use mp_qshared::shared::FS_READ;
use mp_qshared::shared::FS_WRITE;
use mp_qshared::shared::MAX_INFO_STRING;
use mp_qshared::shared::MAX_QPATH;
use mp_qshared::shared::{colorBlack, colorWhite};
use mp_qshared::shared::{BIGCHAR_HEIGHT, BIGCHAR_WIDTH};
use native_string::atoi;
use native_string::info::Info_ValueForKey;
use native_string::latin1_to_string;
use native_string::q_string::Q_stricmp;

use mp_uishared::shared::display_context::DisplayContext;
use mp_uishared::ui_shared::Display_CacheAll;
use mp_uishared::ui_shared::Menus_ActivateByName;
use mp_uishared::ui_shared::Menus_CloseAll;

use crate::local::post_game_info_s::postGameInfo_t;
use crate::trap;
use crate::ui_main::UI_Load;
use crate::ui_main::UI_Report;
use crate::ui_main::UI_ShowPostGame;
use crate::world::ui_context::UiContext;

/// Raven `Com_Error` — formats and forwards a fatal error to the engine.
///
/// Raven's `...`/`vsprintf` formatting collapses to `format!` (dictionary:
/// `va()`/`Com_sprintf` → `format!`); the caller already hands a fully
/// formatted `error` string.
///
/// Source: `oracle/codemp/ui/ui_atoms.c:15-24`
pub fn Com_Error(ctx: &mut UiContext, error: &str) {
    trap::Error(ctx.engine, error);
}

/// Raven `Com_Printf` — formats and forwards a print to the engine.
///
/// Source: `oracle/codemp/ui/ui_atoms.c:26-35`
pub fn Com_Printf(ctx: &mut UiContext, msg: &str) {
    trap::Print(ctx.engine, msg);
}

/// Raven `UI_ClampCvar` — clamps `value` to `[min, max]`.
///
/// Source: `oracle/codemp/ui/ui_atoms.c:47-52`
pub fn UI_ClampCvar(min: f32, max: f32, value: f32) -> f32 {
    if value < min {
        return min;
    }
    if value > max {
        return max;
    }
    value
}

/// Raven `UI_StartDemoLoop` — kicks off the `d1` demo-loop command.
///
/// Source: `oracle/codemp/ui/ui_atoms.c:59-61`
pub fn UI_StartDemoLoop(ctx: &mut UiContext) {
    trap::Cmd_ExecuteText(ctx.engine, cbufExec_t::EXEC_APPEND as c_int, "d1\n");
}

/// Raven `UI_Argv` — returns the `arg`th console-command argument.
///
/// Raven's `static char buffer[MAX_STRING_CHARS]` return-buffer is the
/// `trap::Argv` wrapper's owned `String`.
///
/// Source: `oracle/codemp/ui/ui_atoms.c:64-70`
pub fn UI_Argv(ctx: &mut UiContext, arg: c_int) -> String {
    trap::Argv(ctx.engine, arg, 1024)
}

/// Raven `UI_Cvar_VariableString` — returns a cvar's string value.
///
/// Source: `oracle/codemp/ui/ui_atoms.c:73-79`
pub fn UI_Cvar_VariableString(ctx: &mut UiContext, var_name: &str) -> String {
    trap::Cvar_VariableStringBuffer(ctx.engine, var_name, 1024)
}

/// Raven `UI_SetBestScores` — pushes a post-game score snapshot into the
/// `ui_score*` cvars, doubling into the `*2`-suffixed set when `postGame` is
/// set.
///
/// Source: `oracle/codemp/ui/ui_atoms.c:83-116`
pub fn UI_SetBestScores(ctx: &mut UiContext, newInfo: &postGameInfo_t, postGame: bool) {
    trap::Cvar_Set(
        ctx.engine,
        "ui_scoreAccuracy",
        &format!("{}%", newInfo.accuracy),
    );
    trap::Cvar_Set(
        ctx.engine,
        "ui_scoreImpressives",
        &format!("{}", newInfo.impressives),
    );
    trap::Cvar_Set(
        ctx.engine,
        "ui_scoreExcellents",
        &format!("{}", newInfo.excellents),
    );
    trap::Cvar_Set(
        ctx.engine,
        "ui_scoreDefends",
        &format!("{}", newInfo.defends),
    );
    trap::Cvar_Set(
        ctx.engine,
        "ui_scoreAssists",
        &format!("{}", newInfo.assists),
    );
    trap::Cvar_Set(
        ctx.engine,
        "ui_scoreGauntlets",
        &format!("{}", newInfo.gauntlets),
    );
    trap::Cvar_Set(ctx.engine, "ui_scoreScore", &format!("{}", newInfo.score));
    trap::Cvar_Set(
        ctx.engine,
        "ui_scorePerfect",
        &format!("{}", newInfo.perfects),
    );
    trap::Cvar_Set(
        ctx.engine,
        "ui_scoreTeam",
        &format!("{} to {}", newInfo.redScore, newInfo.blueScore),
    );
    trap::Cvar_Set(
        ctx.engine,
        "ui_scoreBase",
        &format!("{}", newInfo.baseScore),
    );
    trap::Cvar_Set(
        ctx.engine,
        "ui_scoreTimeBonus",
        &format!("{}", newInfo.timeBonus),
    );
    trap::Cvar_Set(
        ctx.engine,
        "ui_scoreSkillBonus",
        &format!("{}", newInfo.skillBonus),
    );
    trap::Cvar_Set(
        ctx.engine,
        "ui_scoreShutoutBonus",
        &format!("{}", newInfo.shutoutBonus),
    );
    trap::Cvar_Set(
        ctx.engine,
        "ui_scoreTime",
        &format!("{:02}:{:02}", newInfo.time / 60, newInfo.time % 60),
    );
    trap::Cvar_Set(
        ctx.engine,
        "ui_scoreCaptures",
        &format!("{}", newInfo.captures),
    );
    if postGame {
        trap::Cvar_Set(
            ctx.engine,
            "ui_scoreAccuracy2",
            &format!("{}%", newInfo.accuracy),
        );
        trap::Cvar_Set(
            ctx.engine,
            "ui_scoreImpressives2",
            &format!("{}", newInfo.impressives),
        );
        trap::Cvar_Set(
            ctx.engine,
            "ui_scoreExcellents2",
            &format!("{}", newInfo.excellents),
        );
        trap::Cvar_Set(
            ctx.engine,
            "ui_scoreDefends2",
            &format!("{}", newInfo.defends),
        );
        trap::Cvar_Set(
            ctx.engine,
            "ui_scoreAssists2",
            &format!("{}", newInfo.assists),
        );
        trap::Cvar_Set(
            ctx.engine,
            "ui_scoreGauntlets2",
            &format!("{}", newInfo.gauntlets),
        );
        trap::Cvar_Set(ctx.engine, "ui_scoreScore2", &format!("{}", newInfo.score));
        trap::Cvar_Set(
            ctx.engine,
            "ui_scorePerfect2",
            &format!("{}", newInfo.perfects),
        );
        trap::Cvar_Set(
            ctx.engine,
            "ui_scoreTeam2",
            &format!("{} to {}", newInfo.redScore, newInfo.blueScore),
        );
        trap::Cvar_Set(
            ctx.engine,
            "ui_scoreBase2",
            &format!("{}", newInfo.baseScore),
        );
        trap::Cvar_Set(
            ctx.engine,
            "ui_scoreTimeBonus2",
            &format!("{}", newInfo.timeBonus),
        );
        trap::Cvar_Set(
            ctx.engine,
            "ui_scoreSkillBonus2",
            &format!("{}", newInfo.skillBonus),
        );
        trap::Cvar_Set(
            ctx.engine,
            "ui_scoreShutoutBonus2",
            &format!("{}", newInfo.shutoutBonus),
        );
        trap::Cvar_Set(
            ctx.engine,
            "ui_scoreTime2",
            &format!("{:02}:{:02}", newInfo.time / 60, newInfo.time % 60),
        );
        trap::Cvar_Set(
            ctx.engine,
            "ui_scoreCaptures2",
            &format!("{}", newInfo.captures),
        );
    }
}

/// Raven `UI_Shutdown` — no-op; ui holds no engine-side resources to release.
///
/// Source: `oracle/codemp/ui/ui_atoms.c:389-390`
pub fn UI_Shutdown() {}

/// Raven `UI_DrawNamedPic` — registers `picname` as a no-mip shader and draws
/// it stretched to `(x, y, width, height)`.
///
/// Source: `oracle/codemp/ui/ui_atoms.c:393-398`
pub fn UI_DrawNamedPic(
    ctx: &mut UiContext,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    picname: &str,
) {
    let hShader: qhandle_t = trap::R_RegisterShaderNoMip(ctx.engine, picname);
    trap::R_DrawStretchPic(ctx.engine, x, y, width, height, 0.0, 0.0, 1.0, 1.0, hShader);
}

/// Raven `UI_DrawHandlePic` — draws an already-registered shader stretched to
/// `(x, y, w, h)`, flipping the s/t texture coordinates when `w`/`h` are
/// negative.
///
/// Source: `oracle/codemp/ui/ui_atoms.c:400-427`
pub fn UI_DrawHandlePic(ctx: &mut UiContext, x: f32, y: f32, w: f32, h: f32, hShader: qhandle_t) {
    let (w, s0, s1) = if w < 0.0 {
        // flip about vertical
        (-w, 1.0, 0.0)
    } else {
        (w, 0.0, 1.0)
    };
    let (h, t0, t1) = if h < 0.0 {
        // flip about horizontal
        (-h, 1.0, 0.0)
    } else {
        (h, 0.0, 1.0)
    };
    trap::R_DrawStretchPic(ctx.engine, x, y, w, h, s0, t0, s1, t1, hShader);
}

/// Raven `UI_FillRect` — draws a solid `color`-filled rect using the white
/// shader, then resets the renderer color to white.
///
/// Source: `oracle/codemp/ui/ui_atoms.c:436-442`
pub fn UI_FillRect(ctx: &mut UiContext, x: f32, y: f32, width: f32, height: f32, color: &vec4_t) {
    trap::R_SetColor(ctx.engine, Some(color));

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
        ctx.world.uiDC.whiteShader,
    );

    trap::R_SetColor(ctx.engine, None);
}

/// Raven `UI_DrawSides` — draws the left/right 1px border sides of a rect.
///
/// Source: `oracle/codemp/ui/ui_atoms.c:444-447`
pub fn UI_DrawSides(ctx: &mut UiContext, x: f32, y: f32, w: f32, h: f32) {
    let whiteShader = ctx.world.uiDC.whiteShader;
    trap::R_DrawStretchPic(ctx.engine, x, y, 1.0, h, 0.0, 0.0, 0.0, 0.0, whiteShader);
    trap::R_DrawStretchPic(
        ctx.engine,
        x + w - 1.0,
        y,
        1.0,
        h,
        0.0,
        0.0,
        0.0,
        0.0,
        whiteShader,
    );
}

/// Raven `UI_DrawTopBottom` — draws the top/bottom 1px border edges of a rect.
///
/// Source: `oracle/codemp/ui/ui_atoms.c:449-452`
pub fn UI_DrawTopBottom(ctx: &mut UiContext, x: f32, y: f32, w: f32, h: f32) {
    let whiteShader = ctx.world.uiDC.whiteShader;
    trap::R_DrawStretchPic(ctx.engine, x, y, w, 1.0, 0.0, 0.0, 0.0, 0.0, whiteShader);
    trap::R_DrawStretchPic(
        ctx.engine,
        x,
        y + h - 1.0,
        w,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        whiteShader,
    );
}

/// Raven `UI_SetColor` — forwards a renderer color set.
///
/// Source: `oracle/codemp/ui/ui_atoms.c:469-471`
pub fn UI_SetColor(ctx: &mut UiContext, rgba: Option<&vec4_t>) {
    trap::R_SetColor(ctx.engine, rgba);
}

/// Raven `UI_UpdateScreen` — forwards a screen update.
///
/// Source: `oracle/codemp/ui/ui_atoms.c:473-475`
pub fn UI_UpdateScreen(ctx: &mut UiContext) {
    trap::UpdateScreen(ctx.engine);
}

/// Raven `UI_CursorInRect` — tests whether the cursor sits inside
/// `(x, y, width, height)`.
///
/// Source: `oracle/codemp/ui/ui_atoms.c:484-493`
pub fn UI_CursorInRect(ctx: &UiContext, x: c_int, y: c_int, width: c_int, height: c_int) -> bool {
    !(ctx.world.uiDC.cursorx < x
        || ctx.world.uiDC.cursory < y
        || ctx.world.uiDC.cursorx > x + width
        || ctx.world.uiDC.cursory > y + height)
}

/// `postGameInfo_t`'s on-disk byte width — Raven's `sizeof(postGameInfo_t)`.
const POST_GAME_INFO_SIZE: usize = core::mem::size_of::<postGameInfo_t>();

// PORT-NOTE: Raven reads/writes `postGameInfo_t` as raw struct bytes
// (`trap_FS_Read(&newInfo, sizeof(postGameInfo_t), f)`). No `unsafe` cast is
// available under the dictionary, so the byte layout is reproduced field by
// field (native-endian, matching the on-disk host-native layout).
fn postGameInfo_from_bytes(buf: &[u8]) -> postGameInfo_t {
    let read = |i: usize| i32::from_ne_bytes(buf[i * 4..i * 4 + 4].try_into().unwrap());
    postGameInfo_t {
        score: read(0),
        redScore: read(1),
        blueScore: read(2),
        perfects: read(3),
        accuracy: read(4),
        impressives: read(5),
        excellents: read(6),
        defends: read(7),
        assists: read(8),
        gauntlets: read(9),
        captures: read(10),
        time: read(11),
        timeBonus: read(12),
        shutoutBonus: read(13),
        skillBonus: read(14),
        baseScore: read(15),
    }
}

fn postGameInfo_to_bytes(info: &postGameInfo_t) -> [u8; POST_GAME_INFO_SIZE] {
    let mut buf = [0u8; POST_GAME_INFO_SIZE];
    let fields = [
        info.score,
        info.redScore,
        info.blueScore,
        info.perfects,
        info.accuracy,
        info.impressives,
        info.excellents,
        info.defends,
        info.assists,
        info.gauntlets,
        info.captures,
        info.time,
        info.timeBonus,
        info.shutoutBonus,
        info.skillBonus,
        info.baseScore,
    ];
    for (i, v) in fields.iter().enumerate() {
        buf[i * 4..i * 4 + 4].copy_from_slice(&v.to_ne_bytes());
    }
    buf
}

/// Raven `UI_LoadBestScores` — loads a saved post-game score snapshot for
/// `map`/`game` off disk into the `ui_score*` cvars, and probes whether a
/// matching demo file exists.
///
/// Source: `oracle/codemp/ui/ui_atoms.c:118-140`
pub fn UI_LoadBestScores(ctx: &mut UiContext, map: &str, game: c_int) {
    let mut newInfo = postGameInfo_from_bytes(&[0u8; POST_GAME_INFO_SIZE]);

    // PORT-NOTE: Raven `Com_sprintf` into `char fileName[MAX_QPATH]`.
    let fileName: String = format!("games/{}_{}.game", map, game)
        .chars()
        .take(MAX_QPATH as usize - 1)
        .collect();
    let mut f: fileHandle_t = 0;
    if trap::FS_FOpenFile(ctx.engine, &fileName, &mut f, FS_READ) >= 0 {
        let mut sizeBuf = [0u8; 4];
        trap::FS_Read(ctx.engine, &mut sizeBuf, f);
        let size = i32::from_ne_bytes(sizeBuf);
        if size as usize == POST_GAME_INFO_SIZE {
            let mut infoBuf = [0u8; POST_GAME_INFO_SIZE];
            trap::FS_Read(ctx.engine, &mut infoBuf, f);
            newInfo = postGameInfo_from_bytes(&infoBuf);
        }
        trap::FS_FCloseFile(ctx.engine, f);
    }
    UI_SetBestScores(ctx, &newInfo, false);

    let protocol = trap::Cvar_VariableValue(ctx.engine, "protocol") as c_int;
    // PORT-NOTE: Raven `Com_sprintf` into `char fileName[MAX_QPATH]`.
    let demoName: String = format!("demos/{}_{}.dm_{}", map, game, protocol)
        .chars()
        .take(MAX_QPATH as usize - 1)
        .collect();
    ctx.world.demoAvailable = false;
    if trap::FS_FOpenFile(ctx.engine, &demoName, &mut f, FS_READ) >= 0 {
        ctx.world.demoAvailable = true;
        trap::FS_FCloseFile(ctx.engine, f);
    }
}

/// Raven `UI_ClearScores` — zeroes every saved post-game score file under
/// `games/` and resets the `ui_score*` cvars.
///
/// Source: `oracle/codemp/ui/ui_atoms.c:147-174`
pub fn UI_ClearScores(ctx: &mut UiContext) {
    let mut gameList = [0u8; 4096];
    let count = trap::FS_GetFileList(ctx.engine, "games", "game", &mut gameList);

    let newInfo = postGameInfo_from_bytes(&[0u8; POST_GAME_INFO_SIZE]);

    if count > 0 {
        let mut offset = 0usize;
        for _ in 0..count {
            // PORT-NOTE: an unterminated tail ends the walk rather than running off
            // the buffer.
            if offset >= gameList.len() {
                break;
            }
            let end = gameList[offset..]
                .iter()
                .position(|&b| b == 0)
                .map(|p| offset + p)
                .unwrap_or(gameList.len());
            let gameFile = latin1_to_string(&gameList[offset..end]);
            let mut f: fileHandle_t = 0;
            if trap::FS_FOpenFile(ctx.engine, &format!("games/{}", gameFile), &mut f, FS_WRITE) >= 0
            {
                trap::FS_Write(ctx.engine, &(POST_GAME_INFO_SIZE as i32).to_ne_bytes(), f);
                trap::FS_Write(ctx.engine, &postGameInfo_to_bytes(&newInfo), f);
                trap::FS_FCloseFile(ctx.engine, f);
            }
            offset = end + 1;
        }
    }

    UI_SetBestScores(ctx, &newInfo, false);
}

/// Raven `UI_DrawRect` — draws a 1px border rectangle around `(x, y, width,
/// height)` in `color`.
///
/// Source: `oracle/codemp/ui/ui_atoms.c:460-467`
pub fn UI_DrawRect(ctx: &mut UiContext, x: f32, y: f32, width: f32, height: f32, color: &vec4_t) {
    trap::R_SetColor(ctx.engine, Some(color));

    UI_DrawTopBottom(ctx, x, y, width, height);
    UI_DrawSides(ctx, x, y, width, height);

    trap::R_SetColor(ctx.engine, None);
}

/// Raven `UI_DrawTextBox` — draws a text-box frame around text using filled and
/// outlined rects, each offset by half a character width/height and sized by
/// (width + 1) and (lines + 1) character units.
///
/// Source: `oracle/codemp/ui/ui_atoms.c:478-482`
pub fn UI_DrawTextBox(ctx: &mut UiContext, x: c_int, y: c_int, width: c_int, lines: c_int) {
    let x_f = (x + BIGCHAR_WIDTH / 2) as f32;
    let y_f = (y + BIGCHAR_HEIGHT / 2) as f32;
    let w = ((width + 1) * BIGCHAR_WIDTH) as f32;
    let h = ((lines + 1) * BIGCHAR_HEIGHT) as f32;

    UI_FillRect(ctx, x_f, y_f, w, h, &colorBlack);
    UI_DrawRect(ctx, x_f, y_f, w, h, &colorWhite);
}

/// Raven `UI_Cache_f` — console command: cache all menu render assets; if
/// invoked with 2 args, also print the list of head model names.
///
/// PORT-NOTE (DisplayContext threading): the threading digest indicated only
/// `UiContext` and `UiWorld` state channels, but `Display_CacheAll` requires a
/// `DisplayContext` trait object to perform caching. Per DEC-36 addendum 12, ui
/// functions that call DisplayContext-using callees take `dc: &mut dyn
/// DisplayContext` as a parameter. This function records an escalation.
///
/// Source: `oracle/codemp/ui/ui_atoms.c:178-187`
pub fn UI_Cache_f(ctx: &mut UiContext, dc: &mut dyn DisplayContext) {
    Display_CacheAll(&ctx.world.menus, dc);
    if trap::Argc(ctx.engine) == 2 {
        for i in 0..ctx.world.q3HeadNames.len() {
            trap::Print(ctx.engine, &format!("model {}\n", ctx.world.q3HeadNames[i]));
        }
    }
}

/// Raven `UI_CalcPostGameStats` — reconciles the just-finished match's score
/// against the saved best (loading/writing `games/<map>_<gametype>.game`),
/// tallies bonuses, restores the ui-overridden server cvars, and pushes the
/// result into the score cvars / post-game screen.
///
/// PORT-NOTE (DisplayContext threading): calls `UI_ShowPostGame`, which takes
/// `dc: &mut dyn DisplayContext` (DEC-36 addendum 12); this fn threads `dc`
/// through as well. Recorded as an escalation per the wave10 packet note (the
/// threading digest only listed `UiContext`/`UiWorld` channels).
///
/// Source: `oracle/codemp/ui/ui_atoms.c:194-288`
pub fn UI_CalcPostGameStats(ctx: &mut UiContext, dc: &mut dyn DisplayContext) {
    let info =
        trap::GetConfigString(ctx.engine, CS_SERVERINFO, MAX_INFO_STRING).unwrap_or_default();
    // PORT-NOTE: Raven `Q_strncpyz(map, Info_ValueForKey(info, "mapname"), sizeof(map))`.
    let map: String = Info_ValueForKey(&info, "mapname")
        .chars()
        .take(MAX_QPATH as usize - 1)
        .collect();
    let game = atoi(&Info_ValueForKey(&info, "g_gametype"));

    // PORT-NOTE: Raven `Com_sprintf` into `char fileName[MAX_QPATH]`.
    let fileName: String = format!("games/{}_{}.game", map, game)
        .chars()
        .take(MAX_QPATH as usize - 1)
        .collect();

    let mut oldInfo = postGameInfo_from_bytes(&[0u8; POST_GAME_INFO_SIZE]);
    let mut f: fileHandle_t = 0;
    if trap::FS_FOpenFile(ctx.engine, &fileName, &mut f, FS_READ) >= 0 {
        let mut sizeBuf = [0u8; 4];
        trap::FS_Read(ctx.engine, &mut sizeBuf, f);
        let size = i32::from_ne_bytes(sizeBuf);
        if size as usize == POST_GAME_INFO_SIZE {
            let mut infoBuf = [0u8; POST_GAME_INFO_SIZE];
            trap::FS_Read(ctx.engine, &mut infoBuf, f);
            oldInfo = postGameInfo_from_bytes(&infoBuf);
        }
        trap::FS_FCloseFile(ctx.engine, f);
    }

    let mut newInfo = postGameInfo_from_bytes(&[0u8; POST_GAME_INFO_SIZE]);
    newInfo.accuracy = atoi(&UI_Argv(ctx, 3));
    newInfo.impressives = atoi(&UI_Argv(ctx, 4));
    newInfo.excellents = atoi(&UI_Argv(ctx, 5));
    newInfo.defends = atoi(&UI_Argv(ctx, 6));
    newInfo.assists = atoi(&UI_Argv(ctx, 7));
    newInfo.gauntlets = atoi(&UI_Argv(ctx, 8));
    newInfo.baseScore = atoi(&UI_Argv(ctx, 9));
    newInfo.perfects = atoi(&UI_Argv(ctx, 10));
    newInfo.redScore = atoi(&UI_Argv(ctx, 11));
    newInfo.blueScore = atoi(&UI_Argv(ctx, 12));
    let time = atoi(&UI_Argv(ctx, 13));
    newInfo.captures = atoi(&UI_Argv(ctx, 14));

    newInfo.time = ((time as f32 - trap::Cvar_VariableValue(ctx.engine, "ui_matchStartTime"))
        / 1000.0) as c_int;
    let mapIdx = ctx.world.cvars.ui_currentMap.integer as usize;
    // PORT-NOTE (§19 UB pick): out-of-range `ui_currentMap`/`g_gametype` index past
    // Raven's live counts into the fixed arrays' garbage (`ui_atoms.c:236`); 0 is the pick.
    let adjustedTime = ctx
        .world
        .mapList
        .get(mapIdx)
        .and_then(|m| m.timeToBeat.get(game as usize))
        .copied()
        .unwrap_or(0);
    if newInfo.time < adjustedTime {
        newInfo.timeBonus = (adjustedTime - newInfo.time) * 10;
    } else {
        newInfo.timeBonus = 0;
    }

    if newInfo.redScore > newInfo.blueScore && newInfo.blueScore <= 0 {
        newInfo.shutoutBonus = 100;
    } else {
        newInfo.shutoutBonus = 0;
    }

    newInfo.skillBonus = trap::Cvar_VariableValue(ctx.engine, "g_spSkill") as c_int;
    if newInfo.skillBonus <= 0 {
        newInfo.skillBonus = 1;
    }
    newInfo.score = newInfo.baseScore + newInfo.shutoutBonus + newInfo.timeBonus;
    newInfo.score *= newInfo.skillBonus;

    // see if the score is higher for this one
    let newHigh = newInfo.redScore > newInfo.blueScore && newInfo.score > oldInfo.score;

    if newHigh {
        // if so write out the new one
        ctx.world.newHighScoreTime = ctx.world.uiDC.realTime + 20000;
        if trap::FS_FOpenFile(ctx.engine, &fileName, &mut f, FS_WRITE) >= 0 {
            trap::FS_Write(ctx.engine, &(POST_GAME_INFO_SIZE as i32).to_ne_bytes(), f);
            trap::FS_Write(ctx.engine, &postGameInfo_to_bytes(&newInfo), f);
            trap::FS_FCloseFile(ctx.engine, f);
        }
    }

    if newInfo.time < oldInfo.time {
        ctx.world.newBestTime = ctx.world.uiDC.realTime + 20000;
    }

    // put back all the ui overrides
    let saveCaptureLimit = UI_Cvar_VariableString(ctx, "ui_saveCaptureLimit");
    trap::Cvar_Set(ctx.engine, "capturelimit", &saveCaptureLimit);
    let saveFragLimit = UI_Cvar_VariableString(ctx, "ui_saveFragLimit");
    trap::Cvar_Set(ctx.engine, "fraglimit", &saveFragLimit);
    let saveDuelLimit = UI_Cvar_VariableString(ctx, "ui_saveDuelLimit");
    trap::Cvar_Set(ctx.engine, "duel_fraglimit", &saveDuelLimit);
    let drawTimer = UI_Cvar_VariableString(ctx, "ui_drawTimer");
    trap::Cvar_Set(ctx.engine, "cg_drawTimer", &drawTimer);
    let doWarmup = UI_Cvar_VariableString(ctx, "ui_doWarmup");
    trap::Cvar_Set(ctx.engine, "g_doWarmup", &doWarmup);
    let warmup = UI_Cvar_VariableString(ctx, "ui_Warmup");
    trap::Cvar_Set(ctx.engine, "g_Warmup", &warmup);
    let pure = UI_Cvar_VariableString(ctx, "ui_pure");
    trap::Cvar_Set(ctx.engine, "sv_pure", &pure);
    let friendlyFire = UI_Cvar_VariableString(ctx, "ui_friendlyFire");
    trap::Cvar_Set(ctx.engine, "g_friendlyFire", &friendlyFire);

    UI_SetBestScores(ctx, &newInfo, true);
    UI_ShowPostGame(ctx, dc, newHigh);
}

/// Raven `UI_ConsoleCommand` — dispatches the `ui_*` console-command family;
/// returns whether the command was consumed by the ui module.
///
/// PORT-NOTE (DisplayContext threading): `Menus_CloseAll`/`Menus_ActivateByName`
/// take a `dc: &mut dyn DisplayContext` (DEC-36 addendum 12); the threading
/// digest only listed `UiContext`/`UiWorld` channels, so this fn threads `dc`
/// through as well. Recorded as an escalation.
///
/// Source: `oracle/codemp/ui/ui_atoms.c:296-382`
pub fn UI_ConsoleCommand(
    ctx: &mut UiContext,
    dc: &mut dyn DisplayContext,
    realTime: c_int,
) -> bool {
    ctx.world.uiDC.frameTime = realTime - ctx.world.uiDC.realTime;
    ctx.world.uiDC.realTime = realTime;

    let cmd = UI_Argv(ctx, 0);

    // ensure minimum menu data is available
    //Menu_Cache();

    if Q_stricmp(&cmd, "ui_test") == 0 {
        UI_ShowPostGame(ctx, dc, true);
    }

    if Q_stricmp(&cmd, "ui_report") == 0 {
        UI_Report(dc);
        return true;
    }

    if Q_stricmp(&cmd, "ui_load") == 0 {
        UI_Load(ctx, dc);
        return true;
    }

    if Q_stricmp(&cmd, "ui_opensiegemenu") == 0 {
        if trap::Cvar_VariableValue(ctx.engine, "g_gametype") == GT_SIEGE as f32 {
            Menus_CloseAll(&mut ctx.world.menus, &ctx.world.uiDC, dc);
            let arg1 = UI_Argv(ctx, 1);
            if Menus_ActivateByName(&mut ctx.world.menus, &ctx.world.uiDC, dc, &arg1).is_some() {
                trap::Key_SetCatcher(ctx.engine, KEYCATCH_UI);
            }
        }
        return true;
    }

    if Q_stricmp(&cmd, "ui_openmenu") == 0 {
        //if ( trap_Cvar_VariableValue ( "developer" ) )
        {
            Menus_CloseAll(&mut ctx.world.menus, &ctx.world.uiDC, dc);
            let arg1 = UI_Argv(ctx, 1);
            if Menus_ActivateByName(&mut ctx.world.menus, &ctx.world.uiDC, dc, &arg1).is_some() {
                trap::Key_SetCatcher(ctx.engine, KEYCATCH_UI);
            }
            return true;
        }
    }

    /*
    if ( Q_stricmp (cmd, "remapShader") == 0 ) {
        if (trap_Argc() == 4) {
            char shader1[MAX_QPATH];
            char shader2[MAX_QPATH];
            Q_strncpyz(shader1, UI_Argv(1), sizeof(shader1));
            Q_strncpyz(shader2, UI_Argv(2), sizeof(shader2));
            trap_R_RemapShader(shader1, shader2, UI_Argv(3));
            return qtrue;
        }
    }
    */

    if Q_stricmp(&cmd, "postgame") == 0 {
        UI_CalcPostGameStats(ctx, dc);
        return true;
    }

    if Q_stricmp(&cmd, "ui_cache") == 0 {
        UI_Cache_f(ctx, dc);
        return true;
    }

    if Q_stricmp(&cmd, "ui_teamOrders") == 0 {
        //UI_TeamOrdersMenu_f();
        return true;
    }

    if Q_stricmp(&cmd, "ui_cdkey") == 0 {
        //UI_CDKeyMenu_f();
        return true;
    }

    false
}
