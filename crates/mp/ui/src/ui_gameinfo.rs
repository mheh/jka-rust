//! `ui_gameinfo.c` — arena/bot info loading.
//!
//! Source: `oracle/codemp/ui/ui_gameinfo.c`

#![allow(non_snake_case)]

use core::ffi::c_char;
use core::ffi::c_int;

use mp_bg::public::gametype::{
    GT_CTF, GT_CTY, GT_DUEL, GT_FFA, GT_HOLOCRON, GT_JEDIMASTER, GT_POWERDUEL, GT_SIEGE, GT_TEAM,
};
use mp_bg::public::{MAX_ARENAS_TEXT, MAX_BOTS_TEXT};
use mp_qshared::shared::com_parse::{COM_Parse, COM_ParseExt};
use mp_qshared::shared::cvar::{vmCvar_t, CVAR_INIT, CVAR_ROM};
use mp_qshared::shared::q_color::{S_COLOR_RED, S_COLOR_YELLOW};
use mp_qshared::shared::q_string::COM_Compress;
use mp_qshared::shared::{fileHandle_t, FS_READ};
use mp_uishared::ui_shared::UI_OutOfMemory;
use native_string::info::{Info_SetValueForKey, Info_ValueForKey};
use native_string::q_string::Q_stricmp;
use native_string::{buf_to_string, latin1_to_string};

use crate::local::map_info::MapInfo;
use crate::trap;
use crate::ui_atoms::Com_Printf;
use crate::world::ui_context::UiContext;
use crate::world::ui_gameinfo_state::{MAX_ARENAS, MAX_BOTS};
use crate::world::ui_world::UiWorld;

/// Raven `#define MAX_MAPS 128`.
///
/// Source: `oracle/codemp/ui/ui_local.h:567`
pub const MAX_MAPS: usize = 128;

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

/// Raven `UI_LoadArenas` — parse every `scripts/*.arena` file into
/// `ui_arenaInfos`, then build `uiInfo.mapList` from the parsed entries
/// (map name/load name/levelshot path and game-type bits).
///
/// Source: `oracle/codemp/ui/ui_gameinfo.c:125-202`
pub fn UI_LoadArenas(ctx: &mut UiContext) {
    // PORT-NOTE: Raven resets `ui_numArenas = 0; uiInfo.mapCount = 0;` — the
    // owned `Vec`s carry the count as `len()`, so clearing is the equivalent.
    ctx.world.gameinfo.ui_arenaInfos.clear();
    ctx.world.mapList.clear();

    // get all arenas from .arena files
    let mut dirlist = vec![0u8; 1024];
    let numdirs = trap::FS_GetFileList(ctx.engine, "scripts", ".arena", &mut dirlist);
    let dirnames = latin1_to_string(&dirlist);
    let mut dirptrs = dirnames.split('\0');
    for _ in 0..numdirs {
        let dirptr = match dirptrs.next() {
            Some(d) => d,
            None => break,
        };
        let filename = format!("scripts/{}", dirptr);
        UI_LoadArenasFromFile(ctx, &filename);
    }

    if UI_OutOfMemory() {
        trap::Print(
            ctx.engine,
            &format!(
                "{}WARNING: not anough memory in pool to load all arenas\n",
                S_COLOR_YELLOW.to_str().unwrap()
            ),
        );
    }

    for n in 0..ctx.world.gameinfo.ui_arenaInfos.len() {
        // determine type
        let arena_info = ctx.world.gameinfo.ui_arenaInfos[n].clone();

        let mapLoadName = Info_ValueForKey(&arena_info, "map");
        let mapName = Info_ValueForKey(&arena_info, "longname");
        let imageName = format!("levelshots/{}", mapLoadName);

        let mut typeBits: c_int = 0;
        let gtype = Info_ValueForKey(&arena_info, "type");
        // if no type specified, it will be treated as "ffa"
        if !gtype.is_empty() {
            if gtype.contains("ffa") {
                typeBits |= 1 << GT_FFA;
            }
            if gtype.contains("team") {
                typeBits |= 1 << GT_TEAM;
            }
            if gtype.contains("holocron") {
                typeBits |= 1 << GT_HOLOCRON;
            }
            if gtype.contains("jedimaster") {
                typeBits |= 1 << GT_JEDIMASTER;
            }
            if gtype.contains("duel") {
                typeBits |= 1 << GT_DUEL;
                typeBits |= 1 << GT_POWERDUEL;
            }
            if gtype.contains("powerduel") {
                typeBits |= 1 << GT_DUEL;
                typeBits |= 1 << GT_POWERDUEL;
            }
            if gtype.contains("siege") {
                typeBits |= 1 << GT_SIEGE;
            }
            if gtype.contains("ctf") {
                typeBits |= 1 << GT_CTF;
            }
            if gtype.contains("cty") {
                typeBits |= 1 << GT_CTY;
            }
        } else {
            typeBits |= 1 << GT_FFA;
        }

        ctx.world.mapList.push(MapInfo {
            cinematic: -1,
            mapLoadName,
            mapName,
            levelShot: -1,
            imageName,
            typeBits,
            ..Default::default()
        });

        if ctx.world.mapList.len() >= MAX_MAPS {
            break;
        }
    }
}

/// Raven `UI_LoadBots` — register `g_botsFile`, load its bot defs (falling
/// back to `botfiles/bots.txt`), then parse every `scripts/*.bot` file into
/// `ui_botInfos`.
///
/// Source: `oracle/codemp/ui/ui_gameinfo.c:260-289`
pub fn UI_LoadBots(ctx: &mut UiContext) {
    // PORT-NOTE: Raven resets `ui_numBots = 0` — the owned `Vec` carries the
    // count as `len()`, so clearing is the equivalent.
    ctx.world.gameinfo.ui_botInfos.clear();

    let mut botsFile = vmCvar_t::zeroed();
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut botsFile),
        "g_botsFile",
        "",
        CVAR_INIT | CVAR_ROM,
    );
    let botsFile_string = buf_to_string(
        &botsFile
            .string
            .iter()
            .map(|&c| c as u8)
            .collect::<Vec<u8>>(),
    );
    if !botsFile_string.is_empty() {
        UI_LoadBotsFromFile(ctx, &botsFile_string);
    } else {
        UI_LoadBotsFromFile(ctx, "botfiles/bots.txt");
    }

    // get all bots from .bot files
    let mut dirlist = vec![0u8; 1024];
    let numdirs = trap::FS_GetFileList(ctx.engine, "scripts", ".bot", &mut dirlist);
    let dirnames = latin1_to_string(&dirlist);
    let mut dirptrs = dirnames.split('\0');
    for _ in 0..numdirs {
        let dirptr = match dirptrs.next() {
            Some(d) => d,
            None => break,
        };
        let filename = format!("scripts/{}", dirptr);
        UI_LoadBotsFromFile(ctx, &filename);
    }
}
