//! `ui_atoms.c` — ui module utility functions.
//!
//! Source: `oracle/codemp/ui/ui_atoms.c`

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_qshared::shared::cbuf_exec::cbufExec_t;
use mp_qshared::shared::fileHandle_t;
use mp_qshared::shared::qhandle_t;
use mp_qshared::shared::vec4_t;
use mp_qshared::shared::FS_READ;
use mp_qshared::shared::FS_WRITE;
use mp_qshared::shared::MAX_QPATH;
use native_string::latin1_to_string;

use crate::local::post_game_info_s::postGameInfo_t;
use crate::trap;
use crate::world::ui_context::UiContext;

/// Raven `BIGCHAR_WIDTH`.
///
/// Source: `oracle/codemp/game/q_shared.h:1038`
const BIGCHAR_WIDTH: c_int = 16;

/// Raven `BIGCHAR_HEIGHT`.
///
/// Source: `oracle/codemp/game/q_shared.h:1039`
const BIGCHAR_HEIGHT: c_int = 16;

/// Raven `colorBlack`.
///
/// Source: `oracle/codemp/game/q_math.c:11`
const COLOR_BLACK: vec4_t = [0.0, 0.0, 0.0, 1.0];

/// Raven `colorWhite`.
///
/// Source: `oracle/codemp/game/q_math.c:18`
const COLOR_WHITE: vec4_t = [1.0, 1.0, 1.0, 1.0];

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

    UI_FillRect(ctx, x_f, y_f, w, h, &COLOR_BLACK);
    UI_DrawRect(ctx, x_f, y_f, w, h, &COLOR_WHITE);
}
