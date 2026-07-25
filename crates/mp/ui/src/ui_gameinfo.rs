//! `ui_gameinfo.c` — arena/bot info loading.
//!
//! Source: `oracle/codemp/ui/ui_gameinfo.c`

#![allow(non_snake_case)]

use core::ffi::c_char;
use core::ffi::c_int;

use mp_qshared::shared::com_parse::{COM_Parse, COM_ParseExt};
use mp_qshared::shared::q_color::S_COLOR_RED;
use mp_qshared::shared::q_string::COM_Compress;
use mp_qshared::shared::{fileHandle_t, FS_READ};
use native_string::info::{Info_SetValueForKey, Info_ValueForKey};
use native_string::latin1_to_string;
use native_string::q_string::Q_stricmp;

use crate::trap;
use crate::ui_atoms::Com_Printf;
use crate::world::ui_context::UiContext;
use crate::world::ui_gameinfo_state::{MAX_ARENAS, MAX_BOTS};
use crate::world::ui_world::UiWorld;

// Raven `ui_gameinfo.c` file-scope `#define`s (mirrored from `bg_public.h`,
// per the g_bot.c analog — not centrally homed).
// Source: `oracle/codemp/game/bg_public.h:1674,1677`
const MAX_ARENAS_TEXT: usize = 8192;
const MAX_BOTS_TEXT: usize = 8192;

/// Raven `UI_GetBotInfoByNumber` — retrieve bot info string by numeric index.
///
/// Source: `oracle/codemp/ui/ui_gameinfo.c:297-303`
pub fn UI_GetBotInfoByNumber<'a>(ctx: &'a mut UiContext, num: c_int) -> Option<&'a str> {
    let num_bots = ctx.world.gameinfo.ui_botInfos.len() as c_int;
    if num < 0 || num >= num_bots {
        trap::Print(ctx.engine, &format!("^1Invalid bot number: {}\n", num));
        return None;
    }
    Some(ctx.world.gameinfo.ui_botInfos[num as usize].as_str())
}

/// Raven `UI_GetBotInfoByName` — retrieve bot info string by name.
///
/// Source: `oracle/codemp/ui/ui_gameinfo.c:311-323`
pub fn UI_GetBotInfoByName<'a>(world: &'a UiWorld, name: &str) -> Option<&'a str> {
    for bot_info in &world.gameinfo.ui_botInfos {
        let value = Info_ValueForKey(bot_info, "name");
        if Q_stricmp(&value, name) == 0 {
            return Some(bot_info.as_str());
        }
    }
    None
}

/// Raven `UI_GetNumBots` — return count of loaded bot info strings.
///
/// Source: `oracle/codemp/ui/ui_gameinfo.c:325-327`
pub fn UI_GetNumBots(world: &UiWorld) -> c_int {
    world.gameinfo.ui_botInfos.len() as c_int
}

/// Raven `UI_ParseInfos` — parse an arena/bot info file into a list of
/// info strings (one per `{ ... }` block), bounded by `max`.
///
/// PORT-NOTE: the C out-param `char *infos[]` + returned `count` collapse
/// into the returned `Vec<String>` (dictionary: out-params -> returns);
/// `infos.len()` is the count. `UI_Alloc`'s null-check guarding the push
/// (Raven's OOM path) is dropped — `UI_Alloc`'s Rust shape (`Vec<u8>`)
/// cannot return null; Rust `Vec` allocation panics on OOM instead.
///
/// Source: `oracle/codemp/ui/ui_gameinfo.c:26-90`
pub fn UI_ParseInfos(ctx: &mut UiContext, buf: &str, max: c_int) -> Vec<String> {
    let mut infos: Vec<String> = Vec::new();
    let mut cursor: Option<&[u8]> = Some(buf.as_bytes());

    loop {
        let (token, rest) = COM_Parse(&mut ctx.world.bg_state.qs, cursor);
        cursor = rest;
        if token.is_empty() {
            break;
        }
        if token != "{" {
            Com_Printf(ctx, "Missing { in info file\n");
            break;
        }

        if infos.len() as c_int == max {
            Com_Printf(ctx, "Max infos exceeded\n");
            break;
        }

        let mut info = String::new();
        loop {
            let (token, rest) = COM_ParseExt(&mut ctx.world.bg_state.qs, cursor, true);
            cursor = rest;
            if token.is_empty() {
                Com_Printf(ctx, "Unexpected end of info file\n");
                break;
            }
            if token == "}" {
                break;
            }
            let key = token;

            let (mut value, rest2) = COM_ParseExt(&mut ctx.world.bg_state.qs, cursor, false);
            cursor = rest2;
            if value.is_empty() {
                value = "<NULL>".to_string();
            }
            Info_SetValueForKey(&mut info, &key, &value);
        }

        // NOTE: extra space for arena number
        // PORT-NOTE: the arena-number space reservation was a C alloc-size
        // computation (`UI_Alloc(strlen(info) + strlen("\\num\\") + ...)`
        // — Source: `oracle/codemp/ui/ui_shared.c:209-234`); `String` grows
        // as needed, so no reservation is transcribed.
        //
        // PORT-NOTE: `#ifndef FINAL_BUILD` build-script bot-file validation
        // preserved unconditionally (no `FINAL_BUILD` cfg gate exists yet
        // in this port).
        if trap::Cvar_VariableValue(ctx.engine, "com_buildScript") != 0.0 {
            let botFile = Info_ValueForKey(&info, "personality");
            if !botFile.is_empty() {
                let mut fh: fileHandle_t = 0;
                trap::FS_FOpenFile(ctx.engine, &botFile, &mut fh, FS_READ);
                if fh != 0 {
                    trap::FS_FCloseFile(ctx.engine, fh);
                }
            }
        }

        infos.push(info);
    }

    infos
}

/// Raven `UI_GetBotNameByNumber` — return the bot's display name by index,
/// falling back to `"Kyle"` if the bot has no info entry.
///
/// Source: `oracle/codemp/ui/ui_gameinfo.c:330-336`
pub fn UI_GetBotNameByNumber(ctx: &mut UiContext, num: c_int) -> String {
    if let Some(info) = UI_GetBotInfoByNumber(ctx, num) {
        Info_ValueForKey(info, "name")
    } else {
        "Kyle".to_string()
    }
}

/// Raven `UI_LoadArenasFromFile` — read one arena-defs file and append its
/// parsed entries to `ui_arenaInfos` (capped by `MAX_ARENAS`).
///
/// Source: `oracle/codemp/ui/ui_gameinfo.c:97-118`
pub fn UI_LoadArenasFromFile(ctx: &mut UiContext, filename: &str) {
    let mut f: fileHandle_t = 0;
    let len = trap::FS_FOpenFile(ctx.engine, filename, &mut f, FS_READ);
    if f == 0 {
        trap::Print(
            ctx.engine,
            &format!(
                "{}file not found: {}\n",
                S_COLOR_RED.to_str().unwrap(),
                filename
            ),
        );
        return;
    }
    if len >= MAX_ARENAS_TEXT as c_int {
        trap::Print(
            ctx.engine,
            &format!(
                "{}file too large: {} is {}, max allowed is {}",
                S_COLOR_RED.to_str().unwrap(),
                filename,
                len,
                MAX_ARENAS_TEXT
            ),
        );
        trap::FS_FCloseFile(ctx.engine, f);
        return;
    }

    let mut buf = vec![0u8; len as usize];
    trap::FS_Read(ctx.engine, &mut buf, f);
    trap::FS_FCloseFile(ctx.engine, f);

    let text = latin1_to_string(&buf);
    let max = MAX_ARENAS as c_int - ctx.world.gameinfo.ui_arenaInfos.len() as c_int;
    let mut added = UI_ParseInfos(ctx, &text, max);
    ctx.world.gameinfo.ui_arenaInfos.append(&mut added);
}

/// Raven `UI_LoadBotsFromFile` — read one bot-defs file and append its
/// parsed entries to `ui_botInfos` (capped by `MAX_BOTS`).
///
/// Source: `oracle/codemp/ui/ui_gameinfo.c:210-253`
pub fn UI_LoadBotsFromFile(ctx: &mut UiContext, filename: &str) {
    let mut f: fileHandle_t = 0;
    let len = trap::FS_FOpenFile(ctx.engine, filename, &mut f, FS_READ);
    if f == 0 {
        trap::Print(
            ctx.engine,
            &format!(
                "{}file not found: {}\n",
                S_COLOR_RED.to_str().unwrap(),
                filename
            ),
        );
        return;
    }
    if len >= MAX_BOTS_TEXT as c_int {
        trap::Print(
            ctx.engine,
            &format!(
                "{}file too large: {} is {}, max allowed is {}",
                S_COLOR_RED.to_str().unwrap(),
                filename,
                len,
                MAX_BOTS_TEXT
            ),
        );
        trap::FS_FCloseFile(ctx.engine, f);
        return;
    }

    // PORT-NOTE: Raven writes `buf[len] = 0` one past the read region (the
    // fixed `buf[MAX_BOTS_TEXT]` has room); the owned buffer here carries the
    // extra zero byte so the `@STOPHERE` scan below matches Raven's
    // null-terminated search.
    let mut buf = vec![0u8; len as usize + 1];
    trap::FS_Read(ctx.engine, &mut buf[..len as usize], f);

    // This bot is in place as a mark for modview's bot viewer. If we hit it
    // just stop and trace back to the beginning of the bot define and cut
    // the string off. This is only done in the UI and not the game so that
    // "test" bots can be added manually and still not show up in the menu.
    if let Some(stop_idx) = buf.windows(9).position(|w| w == b"@STOPHERE") {
        let mut startPoint = stop_idx;
        // §19: a `@STOPHERE` with no preceding `{` walks Raven off the front of
        // the buffer; the walk stops at index 0 here.
        while startPoint > 0 && buf[startPoint] != b'{' {
            startPoint -= 1;
        }
        buf[startPoint] = 0;
    }

    trap::FS_FCloseFile(ctx.engine, f);

    let compressed_len = COM_Compress(buf.as_mut_ptr() as *mut c_char);
    let text = latin1_to_string(&buf[..compressed_len as usize]);

    let max = MAX_BOTS as c_int - ctx.world.gameinfo.ui_botInfos.len() as c_int;
    let mut added = UI_ParseInfos(ctx, &text, max);
    ctx.world.gameinfo.ui_botInfos.append(&mut added);
}
