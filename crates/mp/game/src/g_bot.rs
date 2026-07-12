// PORT-COMPLETE: g_bot.c 25/25 (pass-3 zero-park fill — every fn below has a
// real body per the pass-3 packet; genuinely-unported globals/types are
// referenced verbatim per porting-rules zero-park policy and surfaced in the
// packet's missing_symbols report, not stubbed).
//! FAITHFUL port of `oracle/codemp/game/g_bot.c`.
#![allow(non_snake_case, unused, clippy::all)]

use crate::client::client_connected::CON_CONNECTED;
use crate::prelude::*;
use crate::trap;
use core::ffi::CStr;
use mp_bg::public::gametype::{
    GT_CTF, GT_CTY, GT_DUEL, GT_FFA, GT_HOLOCRON, GT_JEDIMASTER, GT_POWERDUEL, GT_SIEGE, GT_TEAM,
};
use std::ffi::CString;

use crate::ai_main::BotAISetupClient;
use crate::ai_wpnav::LoadPath_ThisLevel;
use crate::g_client::{ClientBegin, ClientConnect, ClientUserinfoChanged, PickTeam};
use crate::g_cmds::SetTeam;
use crate::g_main::{Com_Printf, G_GetStringEdString, G_PowerDuelCount, G_Printf};
use crate::g_mem::G_Alloc;
use crate::g_session::G_ReadSessionData;
use crate::g_team::{S_COLOR_RED, S_COLOR_YELLOW};
use crate::level::bot_settings::bot_settings_t;
use crate::q_shared::{COM_Parse, COM_ParseExt, Info_SetValueForKey, Info_ValueForKey, Q_CleanStr};
use mp_bg::public::duel_team::duelTeam_t::{DUELTEAM_DOUBLE, DUELTEAM_LONE};

use mp_abi::game::syscalls::G_ARGV::GArgvArgs;
use mp_abi::game::syscalls::G_BOT_ALLOCATE_CLIENT::GBotAllocateClientArgs;
use mp_abi::game::syscalls::G_CVAR_REGISTER::GCvarRegisterArgs;
use mp_abi::game::syscalls::G_CVAR_SET::GCvarSetArgs;
use mp_abi::game::syscalls::G_CVAR_UPDATE::GCvarUpdateArgs;
use mp_abi::game::syscalls::G_CVAR_VARIABLE_INTEGER_VALUE::GCvarVariableIntegerValueArgs;
use mp_abi::game::syscalls::G_DROP_CLIENT::GDropClientArgs;
use mp_abi::game::syscalls::G_FS_FCLOSE_FILE::GFsFcloseFileArgs;
use mp_abi::game::syscalls::G_FS_FOPEN_FILE::GFsFopenFileArgs;
use mp_abi::game::syscalls::G_FS_GETFILELIST::GFsGetfilelistArgs;
use mp_abi::game::syscalls::G_FS_READ::GFsReadArgs;
use mp_abi::game::syscalls::G_GET_USERINFO::GGetUserinfoArgs;
use mp_abi::game::syscalls::G_PRINT::GPrintArgs;
use mp_abi::game::syscalls::G_SEND_CONSOLE_COMMAND::GSendConsoleCommandArgs;
use mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs;
use mp_abi::game::syscalls::G_SET_USERINFO::GSetUserinfoArgs;

// No libc dependency in this crate: thin `CStr`-based wrapper over the one
// libc call `G_GetMapTypeBits` needs (`strstr`).
//
// # Safety
// `haystack`/`needle` must be valid NUL-terminated C strings.
unsafe fn libc_strstr(haystack: *const c_char, needle: *const c_char) -> *const c_char {
    let hay = CStr::from_ptr(haystack).to_bytes();
    let ndl = CStr::from_ptr(needle).to_bytes();
    if ndl.is_empty() {
        return haystack;
    }
    if let Some(pos) = hay.windows(ndl.len()).position(|w| w == ndl) {
        haystack.add(pos)
    } else {
        core::ptr::null()
    }
}

/// Raven `trap_Cvar_VariableValue`.
///
/// Source: `oracle/codemp/game/g_bot.c:36-41`
pub fn trap_Cvar_VariableValue(ctx: GameContext<'_>, var_name: *const c_char) -> f32 {
    unsafe {
        let mut buf = [0 as c_char; 128];
        trap::Cvar_VariableStringBuffer(
            ctx.engine,
            mp_abi::game::syscalls::G_CVAR_VARIABLE_STRING_BUFFER::GCvarVariableStringBufferArgs::new(
                CStr::from_ptr(var_name).to_owned(),
                buf.as_mut_ptr(),
                buf.len() as c_int,
            ),
        );
        crate::bg_lib::atof(buf.as_ptr()) as f32
    }
}

// MISSING-SYMBOL: `g_arenaInfos`/`g_botInfos` (Raven `static char *g_arenaInfos[MAX_ARENAS]`/
// `g_botInfos[MAX_BOTS]`, g_bot.c:9/13) have no `GameGlobals` field yet — only
// `g_numArenas`/`g_numBots` (the counters) were promoted. Every reference below
// is written as `(*ctx.world).globals.g_arenaInfos`/`g_botInfos` exactly as
// Raven names them; a fixer must add
// `pub g_arenaInfos: [*mut c_char; MAX_ARENAS as usize]` /
// `pub g_botInfos: [*mut c_char; MAX_BOTS as usize]` to `GameGlobals`.
// MISSING-SYMBOL: `botSpawnQueue_t` (Raven `struct { int spawnTime; int
// clientNum; } botSpawnQueue_t`, g_bot.c:19-24) is unported — `GameGlobals`
// carries only a `()` placeholder for `botSpawnQueue`. Every reference below
// indexes `.spawnTime`/`.clientNum` as if the array were typed; a fixer must
// port `botSpawnQueue_t` and retype the field
// `[botSpawnQueue_t; BOT_SPAWN_QUEUE_DEPTH]`.
// MISSING-SYMBOL: `bot_minplayers` (Raven file-static `vmCvar_t bot_minplayers`,
// g_bot.c:1226) and `checkminimumplayers_time` (Raven fn-static `int`,
// g_bot.c:572) have no `GameGlobals` home yet; referenced below as
// `(*ctx.world).globals.bot_minplayers` / `.checkminimumplayers_time`.

/// Raven `G_ParseInfos`.
///
/// Source: `oracle/codemp/game/g_bot.c:50-99`
pub fn G_ParseInfos(
    ctx: GameContext<'_>,
    buf: *mut c_char,
    max: c_int,
    infos: *mut *mut c_char,
) -> c_int {
    unsafe {
        let mut count: c_int = 0;
        let mut bufp: *const c_char = buf as *const c_char;

        loop {
            let token = COM_Parse(&mut bufp as *mut *const c_char);
            if *token == 0 {
                break;
            }
            if Q_stricmp(token, cstr("{").as_ptr()) != 0 {
                Com_Printf(cstr("Missing { in info file\n").as_ptr());
                break;
            }
            if count == max {
                Com_Printf(cstr("Max infos exceeded\n").as_ptr());
                break;
            }

            let mut info: [c_char; MAX_INFO_STRING] = [0; MAX_INFO_STRING];
            info[0] = 0;
            loop {
                let token = COM_ParseExt(&mut bufp as *mut *const c_char, qtrue);
                if *token == 0 {
                    Com_Printf(cstr("Unexpected end of info file\n").as_ptr());
                    break;
                }
                if Q_stricmp(token, cstr("}").as_ptr()) == 0 {
                    break;
                }
                let mut key: [c_char; MAX_TOKEN_CHARS] = [0; MAX_TOKEN_CHARS];
                Q_strncpyz(key.as_mut_ptr(), token, key.len() as c_int);

                let token2 = COM_ParseExt(&mut bufp as *mut *const c_char, qfalse);
                let value_ptr = if *token2 == 0 {
                    c"<NULL>".as_ptr()
                } else {
                    token2 as *const c_char
                };
                Info_SetValueForKey(info.as_mut_ptr(), key.as_ptr(), value_ptr);
            }

            // NOTE: extra space for arena number.
            let info_s = cstr_to_str(info.as_ptr());
            let alloc_size = info_s.len() + "\\num\\".len() + format!("{}", MAX_ARENAS).len() + 1;
            let dest = G_Alloc(ctx, alloc_size as c_int) as *mut c_char;
            if !dest.is_null() {
                let bytes = info_s.as_bytes();
                for i in 0..bytes.len() {
                    *dest.add(i) = bytes[i] as c_char;
                }
                *dest.add(bytes.len()) = 0;
                *infos.add(count as usize) = dest;
                count += 1;
            }
        }
        count
    }
}

// Raven `g_bot.c` file-scope `#define`s (verified against the owning TU).
// Source: `oracle/codemp/game/g_bot.c:9,13,19`
const MAX_ARENAS: c_int = 1024;
const MAX_ARENAS_TEXT: usize = 8192;
const MAX_BOTS: c_int = 1024;
const MAX_BOTS_TEXT: usize = 8192;
const BOT_SPAWN_QUEUE_DEPTH: usize = 16;

/// Raven `G_LoadArenasFromFile`.
///
/// Source: `oracle/codemp/game/g_bot.c:106-127`
pub fn G_LoadArenasFromFile(ctx: GameContext<'_>, filename: *mut c_char) {
    unsafe {
        let mut f: fileHandle_t = 0;
        let filename_s = cstr_to_str(filename);
        let len = trap::FS_FOpenFile(
            ctx.engine,
            GFsFopenFileArgs::new(CString::new(filename_s.clone()).unwrap(), &mut f, FS_READ),
        );
        if f == 0 {
            let s = format!(
                "{}file not found: {}\n",
                S_COLOR_RED.to_string_lossy(),
                filename_s
            );
            trap::Printf(ctx.engine, GPrintArgs::new(CString::new(s).unwrap()));
            return;
        }
        if len >= MAX_ARENAS_TEXT as c_int {
            let s = format!(
                "{}file too large: {} is {}, max allowed is {}",
                S_COLOR_RED.to_string_lossy(),
                filename_s,
                len,
                MAX_ARENAS_TEXT
            );
            trap::Printf(ctx.engine, GPrintArgs::new(CString::new(s).unwrap()));
            trap::FS_FCloseFile(ctx.engine, GFsFcloseFileArgs::new(f));
            return;
        }

        let mut buf: [c_char; MAX_ARENAS_TEXT] = [0; MAX_ARENAS_TEXT];
        trap::FS_Read(
            ctx.engine,
            GFsReadArgs::new(buf.as_mut_ptr() as *mut u8, len, f),
        );
        buf[len as usize] = 0;
        trap::FS_FCloseFile(ctx.engine, GFsFcloseFileArgs::new(f));

        let g_numArenas = (*ctx.world).globals.g_numArenas;
        let added = G_ParseInfos(
            ctx,
            buf.as_mut_ptr(),
            MAX_ARENAS - g_numArenas,
            &mut (*ctx.world).globals.g_arenaInfos[g_numArenas as usize] as *mut *mut c_char,
        );
        (*ctx.world).globals.g_numArenas += added;
    }
}

/// Raven `G_GetMapTypeBits`.
///
/// Source: `oracle/codemp/game/g_bot.c:129-169`
///
/// # Safety
/// `r#type` must be a valid NUL-terminated C string.
pub unsafe fn G_GetMapTypeBits(r#type: *mut c_char) -> c_int {
    let mut typeBits: c_int = 0;

    if *r#type != 0 {
        let t = r#type as *const c_char;
        if !libc_strstr(t, c"ffa".as_ptr()).is_null() {
            typeBits |= 1 << GT_FFA;
            typeBits |= 1 << GT_TEAM;
        }
        if !libc_strstr(t, c"team".as_ptr()).is_null() {
            typeBits |= 1 << GT_TEAM;
        }
        if !libc_strstr(t, c"holocron".as_ptr()).is_null() {
            typeBits |= 1 << GT_HOLOCRON;
        }
        if !libc_strstr(t, c"jedimaster".as_ptr()).is_null() {
            typeBits |= 1 << GT_JEDIMASTER;
        }
        if !libc_strstr(t, c"duel".as_ptr()).is_null() {
            typeBits |= 1 << GT_DUEL;
            typeBits |= 1 << GT_POWERDUEL;
        }
        if !libc_strstr(t, c"powerduel".as_ptr()).is_null() {
            typeBits |= 1 << GT_DUEL;
            typeBits |= 1 << GT_POWERDUEL;
        }
        if !libc_strstr(t, c"siege".as_ptr()).is_null() {
            typeBits |= 1 << GT_SIEGE;
        }
        if !libc_strstr(t, c"ctf".as_ptr()).is_null() {
            typeBits |= 1 << GT_CTF;
        }
        if !libc_strstr(t, c"cty".as_ptr()).is_null() {
            typeBits |= 1 << GT_CTY;
        }
    } else {
        typeBits |= 1 << GT_FFA;
    }

    typeBits
}

/// Raven `G_DoesMapSupportGametype`.
///
/// Source: `oracle/codemp/game/g_bot.c:171-213`
pub fn G_DoesMapSupportGametype(
    ctx: GameContext<'_>,
    mapname: *const c_char,
    gametype: c_int,
) -> qboolean {
    unsafe {
        let world = &*ctx.world;
        if world.globals.g_arenaInfos[0].is_null() {
            return qfalse;
        }
        if mapname.is_null() || *mapname == 0 {
            return qfalse;
        }

        let mut thisLevel: c_int = -1;
        for n in 0..world.globals.g_numArenas {
            let r#type =
                Info_ValueForKey(world.globals.g_arenaInfos[n as usize], cstr("map").as_ptr());
            if Q_stricmp(mapname, r#type) == 0 {
                thisLevel = n;
                break;
            }
        }

        if thisLevel == -1 {
            return qfalse;
        }

        let r#type = Info_ValueForKey(
            world.globals.g_arenaInfos[thisLevel as usize],
            cstr("type").as_ptr(),
        );
        let typeBits = G_GetMapTypeBits(r#type);
        if typeBits & (1 << gametype) != 0 {
            return qtrue;
        }

        qfalse
    }
}

/// Raven `G_RefreshNextMap`.
///
/// Source: `oracle/codemp/game/g_bot.c:216-288`
pub fn G_RefreshNextMap(ctx: GameContext<'_>, gametype: c_int, forced: qboolean) -> *const c_char {
    unsafe {
        let world = &mut *ctx.world;
        if world.cvars.g_autoMapCycle.integer == 0 && forced == 0 {
            return core::ptr::null();
        }
        if world.globals.g_arenaInfos[0].is_null() {
            return core::ptr::null();
        }

        let mut mapname = vmCvar_t::zeroed();
        trap::Cvar_Register(
            ctx.engine,
            GCvarRegisterArgs::new(
                &mut mapname as *mut vmCvar_t,
                CString::new("mapname").unwrap(),
                CString::new("").unwrap(),
                CVAR_SERVERINFO | CVAR_ROM,
            ),
        );

        let mut thisLevel: c_int = 0;
        for n in 0..world.globals.g_numArenas {
            let r#type =
                Info_ValueForKey(world.globals.g_arenaInfos[n as usize], cstr("map").as_ptr());
            if Q_stricmp(mapname.string.as_ptr(), r#type) == 0 {
                thisLevel = n;
                break;
            }
        }

        let mut desiredMap = thisLevel;
        let mut n = thisLevel + 1;
        let mut loopingUp = qfalse;
        while n != thisLevel {
            // Oracle indexes one past the array (real, silent UB reading adjacent static
            // storage) when n reaches MAX_ARENAS on entry; we choose the defined behavior
            // of treating out-of-range n as null/wrap immediately (porting-rules §19).
            if n >= MAX_ARENAS
                || world.globals.g_arenaInfos[n as usize].is_null()
                || n >= world.globals.g_numArenas
            {
                if loopingUp != 0 {
                    break;
                }
                n = 0;
                loopingUp = qtrue;
            }

            let r#type = Info_ValueForKey(
                world.globals.g_arenaInfos[n as usize],
                cstr("type").as_ptr(),
            );
            let typeBits = G_GetMapTypeBits(r#type);
            if typeBits & (1 << gametype) != 0 {
                desiredMap = n;
                break;
            }

            n += 1;
        }

        if desiredMap == thisLevel {
            trap::Cvar_Set(
                ctx.engine,
                GCvarSetArgs::new(
                    CString::new("nextmap").unwrap(),
                    CString::new("map_restart 0").unwrap(),
                ),
            );
        } else {
            let r#type = Info_ValueForKey(
                world.globals.g_arenaInfos[desiredMap as usize],
                cstr("map").as_ptr(),
            );
            let cmd = format!("map {}", cstr_to_str(r#type));
            trap::Cvar_Set(
                ctx.engine,
                GCvarSetArgs::new(CString::new("nextmap").unwrap(), CString::new(cmd).unwrap()),
            );
        }

        Info_ValueForKey(
            world.globals.g_arenaInfos[desiredMap as usize],
            cstr("map").as_ptr(),
        ) as *const c_char
    }
}

/// Raven `G_LoadArenas`.
///
/// Source: `oracle/codemp/game/g_bot.c:295-321`
pub fn G_LoadArenas(ctx: GameContext<'_>) {
    unsafe {
        (*ctx.world).globals.g_numArenas = 0;

        let mut dirlist: [c_char; 1024] = [0; 1024];
        let numdirs = trap::FS_GetFileList(
            ctx.engine,
            GFsGetfilelistArgs::new(
                CString::new("scripts").unwrap(),
                CString::new(".arena").unwrap(),
                dirlist.as_mut_ptr() as *mut u8,
                1024,
            ),
        );
        let mut dirptr = dirlist.as_ptr();
        for _ in 0..numdirs {
            let dirlen = CStr::from_ptr(dirptr).to_bytes().len();
            let mut filename: [c_char; 128] = [0; 128];
            write_cstr_field(&mut filename, &format!("scripts/{}", cstr_to_str(dirptr)));
            G_LoadArenasFromFile(ctx, filename.as_mut_ptr());
            dirptr = dirptr.add(dirlen + 1);
        }

        for n in 0..(*ctx.world).globals.g_numArenas {
            Info_SetValueForKey(
                (*ctx.world).globals.g_arenaInfos[n as usize],
                cstr("num").as_ptr(),
                cstr(&format!("{}", n)).as_ptr(),
            );
        }

        G_RefreshNextMap(ctx, (*ctx.world).cvars.g_gametype.integer, qfalse);
    }
}

/// Raven `G_GetArenaInfoByMap`.
///
/// Source: `oracle/codemp/game/g_bot.c:329-339`
pub fn G_GetArenaInfoByMap(ctx: GameContext<'_>, map: *const c_char) -> *const c_char {
    unsafe {
        let world = &*ctx.world;
        for n in 0..world.globals.g_numArenas {
            if Q_stricmp(
                Info_ValueForKey(world.globals.g_arenaInfos[n as usize], cstr("map").as_ptr()),
                map,
            ) == 0
            {
                return world.globals.g_arenaInfos[n as usize] as *const c_char;
            }
        }
        core::ptr::null()
    }
}

/// Raven `G_AddRandomBot`.
///
/// Source: `oracle/codemp/game/g_bot.c:373-454`
pub fn G_AddRandomBot(ctx: GameContext<'_>, team: c_int) {
    unsafe {
        let world = &mut *ctx.world;
        let mut num: c_int = 0;
        for n in 0..world.globals.g_numBots {
            let value =
                Info_ValueForKey(world.globals.g_botInfos[n as usize], cstr("name").as_ptr());
            let mut i: c_int = 0;
            while i < world.cvars.g_maxclients.integer {
                let cl = &world.clients[i as usize];
                if cl.pers.connected != CON_CONNECTED {
                    i += 1;
                    continue;
                }
                if world.g_entities[cl.ps.clientNum as usize].r.svFlags & SVF_BOT == 0 {
                    i += 1;
                    continue;
                }
                if world.cvars.g_gametype.integer == GT_SIEGE {
                    if team >= 0 && cl.sess.siegeDesiredTeam != team {
                        i += 1;
                        continue;
                    }
                } else if team >= 0 && cl.sess.sessionTeam != team {
                    i += 1;
                    continue;
                }
                if Q_stricmp(value, cl.pers.netname.as_ptr()) == 0 {
                    break;
                }
                i += 1;
            }
            if i >= world.cvars.g_maxclients.integer {
                num += 1;
            }
        }

        num = (world.bg_state.rng.random() * num as f32) as c_int;

        for n in 0..world.globals.g_numBots {
            let value =
                Info_ValueForKey(world.globals.g_botInfos[n as usize], cstr("name").as_ptr());
            let mut i: c_int = 0;
            while i < world.cvars.g_maxclients.integer {
                let cl = &world.clients[i as usize];
                if cl.pers.connected != CON_CONNECTED {
                    i += 1;
                    continue;
                }
                if world.g_entities[cl.ps.clientNum as usize].r.svFlags & SVF_BOT == 0 {
                    i += 1;
                    continue;
                }
                if world.cvars.g_gametype.integer == GT_SIEGE {
                    if team >= 0 && cl.sess.siegeDesiredTeam != team {
                        i += 1;
                        continue;
                    }
                } else if team >= 0 && cl.sess.sessionTeam != team {
                    i += 1;
                    continue;
                }
                if Q_stricmp(value, cl.pers.netname.as_ptr()) == 0 {
                    break;
                }
                i += 1;
            }
            if i >= world.cvars.g_maxclients.integer {
                num -= 1;
                if num <= 0 {
                    let skill = trap_Cvar_VariableValue(ctx, cstr("g_spSkill").as_ptr());
                    let teamstr = if team == TEAM_RED {
                        "red"
                    } else if team == TEAM_BLUE {
                        "blue"
                    } else {
                        ""
                    };
                    let mut netname: [c_char; 36] = [0; 36];
                    write_cstr_field(&mut netname, &cstr_to_str(value));
                    Q_CleanStr(netname.as_mut_ptr());
                    let cmd = format!(
                        "addbot \"{}\" {} {} {}\n",
                        cstr_to_str(netname.as_ptr()),
                        skill,
                        teamstr,
                        0
                    );
                    trap::SendConsoleCommand(
                        ctx.engine,
                        GSendConsoleCommandArgs::new(cbufExec_t::EXEC_INSERT as c_int, cstr(&cmd)),
                    );
                    return;
                }
            }
        }
    }
}

/// Raven `G_RemoveRandomBot`.
///
/// Source: `oracle/codemp/game/g_bot.c:461-492`
pub fn G_RemoveRandomBot(ctx: GameContext<'_>, team: c_int) -> c_int {
    unsafe {
        let world = &mut *ctx.world;
        for i in 0..world.cvars.g_maxclients.integer {
            let cl = &world.clients[i as usize];
            if cl.pers.connected != CON_CONNECTED {
                continue;
            }
            if world.g_entities[cl.ps.clientNum as usize].r.svFlags & SVF_BOT == 0 {
                continue;
            }
            if world.cvars.g_gametype.integer == GT_SIEGE {
                if team >= 0 && cl.sess.siegeDesiredTeam != team {
                    continue;
                }
            } else if team >= 0 && cl.sess.sessionTeam != team {
                continue;
            }

            let mut netname: [c_char; 36] = [0; 36];
            write_cstr_field(&mut netname, &cstr_to_str(cl.pers.netname.as_ptr()));
            Q_CleanStr(netname.as_mut_ptr());
            let cmd = format!("kick \"{}\"\n", cstr_to_str(netname.as_ptr()));
            trap::SendConsoleCommand(
                ctx.engine,
                GSendConsoleCommandArgs::new(cbufExec_t::EXEC_INSERT as c_int, cstr(&cmd)),
            );
            return qtrue;
        }
        qfalse
    }
}

/// Raven `G_CountHumanPlayers`.
///
/// Source: `oracle/codemp/game/g_bot.c:499-518`
pub fn G_CountHumanPlayers(ctx: GameContext<'_>, team: c_int) -> c_int {
    unsafe {
        let world = &*ctx.world;
        let mut num: c_int = 0;
        for i in 0..world.cvars.g_maxclients.integer {
            let cl = &world.clients[i as usize];
            if cl.pers.connected != CON_CONNECTED {
                continue;
            }
            if world.g_entities[cl.ps.clientNum as usize].r.svFlags & SVF_BOT != 0 {
                continue;
            }
            if team >= 0 && cl.sess.sessionTeam as c_int != team {
                continue;
            }
            num += 1;
        }
        num
    }
}

/// Raven `G_CountBotPlayers`.
///
/// Source: `oracle/codemp/game/g_bot.c:525-562`
pub fn G_CountBotPlayers(ctx: GameContext<'_>, team: c_int) -> c_int {
    unsafe {
        let world = &*ctx.world;
        let mut num: c_int = 0;
        for i in 0..world.cvars.g_maxclients.integer {
            let cl = &world.clients[i as usize];
            if cl.pers.connected != CON_CONNECTED {
                continue;
            }
            if world.g_entities[cl.ps.clientNum as usize].r.svFlags & SVF_BOT == 0 {
                continue;
            }
            if world.cvars.g_gametype.integer == GT_SIEGE {
                if team >= 0 && cl.sess.siegeDesiredTeam != team {
                    continue;
                }
            } else if team >= 0 && cl.sess.sessionTeam != team {
                continue;
            }
            num += 1;
        }
        for n in 0..BOT_SPAWN_QUEUE_DEPTH {
            if world.globals.botSpawnQueue[n].spawnTime == 0 {
                continue;
            }
            if world.globals.botSpawnQueue[n].spawnTime > world.level.time {
                continue;
            }
            num += 1;
        }
        num
    }
}

/// Raven `G_CheckMinimumPlayers`.
///
/// Source: `oracle/codemp/game/g_bot.c:569-690`
///
/// The `#if 0`-guarded team-balance tail (g_bot.c:611-688) is Raven-dead
/// code (never compiled); not transcribed, matching the active-code-only
/// faithful-port convention.
pub fn G_CheckMinimumPlayers(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        if world.cvars.g_gametype.integer == GT_SIEGE {
            return;
        }
        if world.level.intermissiontime != 0 {
            return;
        }
        // only check once each 10 seconds
        if world.globals.checkminimumplayers_time > world.level.time - 10000 {
            return;
        }
        world.globals.checkminimumplayers_time = world.level.time;
        trap::Cvar_Update(
            ctx.engine,
            GCvarUpdateArgs::new(&mut world.globals.bot_minplayers as *mut vmCvar_t),
        );
        let mut minplayers = world.globals.bot_minplayers.integer;
        if minplayers <= 0 {
            return;
        }
        if minplayers > world.cvars.g_maxclients.integer {
            minplayers = world.cvars.g_maxclients.integer;
        }

        let humanplayers = G_CountHumanPlayers(ctx, -1);
        let botplayers = G_CountBotPlayers(ctx, -1);

        if humanplayers + botplayers < minplayers {
            G_AddRandomBot(ctx, -1);
        } else if humanplayers + botplayers > minplayers && botplayers != 0 {
            // try to remove spectators first
            if G_RemoveRandomBot(ctx, TEAM_SPECTATOR) == 0 {
                // just remove the bot that is playing
                G_RemoveRandomBot(ctx, -1);
            }
        }
    }
}

/// Raven `G_CheckBotSpawn`.
///
/// Source: `oracle/codemp/game/g_bot.c:697-719`
pub fn G_CheckBotSpawn(ctx: GameContext<'_>) {
    unsafe {
        G_CheckMinimumPlayers(ctx);

        let world = &mut *ctx.world;
        for n in 0..BOT_SPAWN_QUEUE_DEPTH {
            if world.globals.botSpawnQueue[n].spawnTime == 0 {
                continue;
            }
            if world.globals.botSpawnQueue[n].spawnTime > world.level.time {
                continue;
            }
            let clientNum = world.globals.botSpawnQueue[n].clientNum;
            ClientBegin(ctx, clientNum, qfalse);
            world.globals.botSpawnQueue[n].spawnTime = 0;
        }
    }
}

/// Raven `AddBotToSpawnQueue`.
///
/// Source: `oracle/codemp/game/g_bot.c:727-740`
pub fn AddBotToSpawnQueue(ctx: GameContext<'_>, clientNum: c_int, delay: c_int) {
    unsafe {
        let world = &mut *ctx.world;
        for n in 0..BOT_SPAWN_QUEUE_DEPTH {
            if world.globals.botSpawnQueue[n].spawnTime == 0 {
                world.globals.botSpawnQueue[n].spawnTime = world.level.time + delay;
                world.globals.botSpawnQueue[n].clientNum = clientNum;
                return;
            }
        }

        G_Printf(
            ctx,
            cstr(&format!(
                "{}Unable to delay spawn\n",
                S_COLOR_YELLOW.to_string_lossy()
            ))
            .as_ptr(),
        );
        ClientBegin(ctx, clientNum, qfalse);
    }
}

/// Raven `G_RemoveQueuedBotBegin`.
///
/// Source: `oracle/codemp/game/g_bot.c:751-760`
pub fn G_RemoveQueuedBotBegin(ctx: GameContext<'_>, clientNum: c_int) {
    unsafe {
        let world = &mut *ctx.world;
        for n in 0..BOT_SPAWN_QUEUE_DEPTH {
            if world.globals.botSpawnQueue[n].clientNum == clientNum {
                world.globals.botSpawnQueue[n].spawnTime = 0;
                return;
            }
        }
    }
}

/// Raven `G_BotConnect`.
///
/// Source: `oracle/codemp/game/g_bot.c:768-784`
pub fn G_BotConnect(ctx: GameContext<'_>, clientNum: c_int, restart: qboolean) -> qboolean {
    // `MAX_INFO_STRING` resolves via the crate prelude glob.
    unsafe {
        let mut settings: bot_settings_t = core::mem::zeroed();
        let mut userinfo: [c_char; MAX_INFO_STRING] = [0; MAX_INFO_STRING];
        trap::GetUserinfo(
            ctx.engine,
            GGetUserinfoArgs::new(clientNum, userinfo.as_mut_ptr(), userinfo.len() as c_int),
        );

        write_cstr_field(
            &mut settings.personalityfile,
            &cstr_to_str(Info_ValueForKey(
                userinfo.as_ptr(),
                cstr("personality").as_ptr(),
            )),
        );
        settings.skill =
            crate::bg_lib::atof(Info_ValueForKey(userinfo.as_ptr(), cstr("skill").as_ptr())) as f32;
        write_cstr_field(
            &mut settings.team,
            &cstr_to_str(Info_ValueForKey(userinfo.as_ptr(), cstr("team").as_ptr())),
        );

        let ok = BotAISetupClient(
            ctx,
            clientNum,
            &mut settings as *mut bot_settings_t,
            restart,
        );
        if ok == 0 {
            trap::DropClient(
                ctx.engine,
                GDropClientArgs::new(clientNum, CString::new("BotAISetupClient failed").unwrap()),
            );
            return qfalse;
        }

        qtrue
    }
}

/// Raven `G_AddBot`.
///
/// Source: `oracle/codemp/game/g_bot.c:792-1033`
pub fn G_AddBot(
    ctx: GameContext<'_>,
    name: *const c_char,
    skill: f32,
    team: *const c_char,
    delay: c_int,
    altname: *mut c_char,
) {
    // `MAX_INFO_STRING` resolves via the crate prelude glob.
    unsafe {
        // get the botinfo from bots.txt
        let botinfo = G_GetBotInfoByName(ctx, name);
        if botinfo.is_null() {
            G_Printf(
                ctx,
                cstr(&format!(
                    "{}Error: Bot '{}' not defined\n",
                    S_COLOR_RED.to_string_lossy(),
                    cstr_to_str(name)
                ))
                .as_ptr(),
            );
            return;
        }

        // create the bot's userinfo
        let mut userinfo: [c_char; MAX_INFO_STRING] = [0; MAX_INFO_STRING];
        userinfo[0] = 0;

        let mut botname = Info_ValueForKey(botinfo, cstr("funname").as_ptr());
        if *botname == 0 {
            botname = Info_ValueForKey(botinfo, cstr("name").as_ptr());
        }
        // check for an alternative name
        if !altname.is_null() && *altname != 0 {
            botname = altname;
        }
        Info_SetValueForKey(userinfo.as_mut_ptr(), cstr("name").as_ptr(), botname);
        Info_SetValueForKey(
            userinfo.as_mut_ptr(),
            cstr("rate").as_ptr(),
            cstr("25000").as_ptr(),
        );
        Info_SetValueForKey(
            userinfo.as_mut_ptr(),
            cstr("snaps").as_ptr(),
            cstr("20").as_ptr(),
        );
        Info_SetValueForKey(
            userinfo.as_mut_ptr(),
            cstr("skill").as_ptr(),
            cstr(&format!("{:.2}", skill)).as_ptr(),
        );

        if skill >= 1.0 && skill < 2.0 {
            Info_SetValueForKey(
                userinfo.as_mut_ptr(),
                cstr("handicap").as_ptr(),
                cstr("50").as_ptr(),
            );
        } else if skill >= 2.0 && skill < 3.0 {
            Info_SetValueForKey(
                userinfo.as_mut_ptr(),
                cstr("handicap").as_ptr(),
                cstr("70").as_ptr(),
            );
        } else if skill >= 3.0 && skill < 4.0 {
            Info_SetValueForKey(
                userinfo.as_mut_ptr(),
                cstr("handicap").as_ptr(),
                cstr("90").as_ptr(),
            );
        }

        let mut model = Info_ValueForKey(botinfo, cstr("model").as_ptr());
        if *model == 0 {
            model = cstr("kyle/default").into_raw();
        }
        Info_SetValueForKey(userinfo.as_mut_ptr(), cstr("model").as_ptr(), model);

        let mut gender = Info_ValueForKey(botinfo, cstr("gender").as_ptr());
        if *gender == 0 {
            gender = cstr("male").into_raw();
        }
        Info_SetValueForKey(userinfo.as_mut_ptr(), cstr("sex").as_ptr(), gender);

        let mut color1 = Info_ValueForKey(botinfo, cstr("color1").as_ptr());
        if *color1 == 0 {
            color1 = cstr("4").into_raw();
        }
        Info_SetValueForKey(userinfo.as_mut_ptr(), cstr("color1").as_ptr(), color1);

        let mut color2 = Info_ValueForKey(botinfo, cstr("color2").as_ptr());
        if *color2 == 0 {
            color2 = cstr("4").into_raw();
        }
        Info_SetValueForKey(userinfo.as_mut_ptr(), cstr("color2").as_ptr(), color2);

        let mut saber1 = Info_ValueForKey(botinfo, cstr("saber1").as_ptr());
        if *saber1 == 0 {
            saber1 = cstr("single_1").into_raw();
        }
        Info_SetValueForKey(userinfo.as_mut_ptr(), cstr("saber1").as_ptr(), saber1);

        let mut saber2 = Info_ValueForKey(botinfo, cstr("saber2").as_ptr());
        if *saber2 == 0 {
            saber2 = cstr("none").into_raw();
        }
        Info_SetValueForKey(userinfo.as_mut_ptr(), cstr("saber2").as_ptr(), saber2);

        let personality = Info_ValueForKey(botinfo, cstr("personality").as_ptr());
        if *personality == 0 {
            Info_SetValueForKey(
                userinfo.as_mut_ptr(),
                cstr("personality").as_ptr(),
                cstr("botfiles/default.jkb").as_ptr(),
            );
        } else {
            Info_SetValueForKey(
                userinfo.as_mut_ptr(),
                cstr("personality").as_ptr(),
                personality,
            );
        }

        // have the server allocate a client slot
        let clientNum = trap::BotAllocateClient(ctx.engine, GBotAllocateClientArgs::new());
        if clientNum == -1 {
            let msg = G_GetStringEdString(
                ctx,
                cstr("MP_SVGAME").into_raw(),
                cstr("UNABLE_TO_ADD_BOT").into_raw(),
            );
            let s = format!("print \"{}\n\"", cstr_to_str(msg));
            trap::SendServerCommand(ctx.engine, GSendServerCommandArgs::new(-1, cstr(&s)));
            return;
        }

        // initialize the bot settings
        let mut team_owned: String;
        if team.is_null() || *team == 0 {
            if (*ctx.world).cvars.g_gametype.integer >= GT_TEAM {
                if PickTeam(ctx, clientNum) == TEAM_RED {
                    team_owned = "red".to_string();
                } else {
                    team_owned = "blue".to_string();
                }
            } else {
                team_owned = "red".to_string();
            }
        } else {
            team_owned = cstr_to_str(team);
        }
        Info_SetValueForKey(
            userinfo.as_mut_ptr(),
            cstr("skill").as_ptr(),
            cstr(&format!("{:5.2}", skill)).as_ptr(),
        );
        Info_SetValueForKey(
            userinfo.as_mut_ptr(),
            cstr("team").as_ptr(),
            cstr(&team_owned).as_ptr(),
        );

        let bot = &mut (*ctx.world).g_entities[clientNum as usize] as *mut gentity_t;
        (*bot).r.svFlags |= SVF_BOT;
        (*bot).inuse = qtrue;

        // register the userinfo
        trap::SetUserinfo(
            ctx.engine,
            GSetUserinfoArgs::new(
                clientNum,
                CString::new(cstr_to_str(userinfo.as_ptr())).unwrap(),
            ),
        );

        if (*ctx.world).cvars.g_gametype.integer >= GT_TEAM {
            let cl = (*bot).client as *mut gclient_t;
            if Q_stricmp(cstr(&team_owned).as_ptr(), cstr("red").as_ptr()) == 0 {
                (*cl).sess.sessionTeam = TEAM_RED;
            } else if Q_stricmp(cstr(&team_owned).as_ptr(), cstr("blue").as_ptr()) == 0 {
                (*cl).sess.sessionTeam = TEAM_BLUE;
            } else {
                (*cl).sess.sessionTeam = PickTeam(ctx, -1);
            }
        }

        if (*ctx.world).cvars.g_gametype.integer == GT_SIEGE {
            let cl = (*bot).client as *mut gclient_t;
            (*cl).sess.siegeDesiredTeam = (*cl).sess.sessionTeam;
            (*cl).sess.sessionTeam = TEAM_SPECTATOR;
        }

        let cl = (*bot).client as *mut gclient_t;
        let preTeam = (*cl).sess.sessionTeam;

        // have it connect to the game as a normal client
        if !ClientConnect(ctx, clientNum, qtrue, qtrue).is_null() {
            return;
        }

        if (*cl).sess.sessionTeam != preTeam {
            trap::GetUserinfo(
                ctx.engine,
                GGetUserinfoArgs::new(clientNum, userinfo.as_mut_ptr(), MAX_INFO_STRING as c_int),
            );

            if (*cl).sess.sessionTeam == TEAM_SPECTATOR {
                (*cl).sess.sessionTeam = preTeam;
            }

            let team_final = if (*cl).sess.sessionTeam == TEAM_RED {
                "Red".to_string()
            } else if (*ctx.world).cvars.g_gametype.integer == GT_SIEGE {
                if (*cl).sess.sessionTeam == TEAM_BLUE {
                    "Blue".to_string()
                } else {
                    "s".to_string()
                }
            } else {
                "Blue".to_string()
            };

            Info_SetValueForKey(
                userinfo.as_mut_ptr(),
                cstr("team").as_ptr(),
                cstr(&team_final).as_ptr(),
            );
            trap::SetUserinfo(
                ctx.engine,
                GSetUserinfoArgs::new(
                    clientNum,
                    CString::new(cstr_to_str(userinfo.as_ptr())).unwrap(),
                ),
            );

            (*cl).ps.persistant[PERS_TEAM as usize] = (*cl).sess.sessionTeam;

            G_ReadSessionData(ctx, cl);
            ClientUserinfoChanged(ctx, clientNum);
        }

        if (*ctx.world).cvars.g_gametype.integer == GT_DUEL
            || (*ctx.world).cvars.g_gametype.integer == GT_POWERDUEL
        {
            let mut loners: c_int = 0;
            let mut doubles: c_int = 0;

            (*cl).sess.duelTeam = 0;
            G_PowerDuelCount(
                ctx,
                &mut loners as *mut c_int,
                &mut doubles as *mut c_int,
                qtrue,
            );

            if doubles == 0 || loners > doubles / 2 {
                (*cl).sess.duelTeam = DUELTEAM_DOUBLE as c_int;
            } else {
                (*cl).sess.duelTeam = DUELTEAM_LONE as c_int;
            }

            (*cl).sess.sessionTeam = TEAM_SPECTATOR;
            SetTeam(ctx, bot, cstr("s").into_raw());
        } else {
            if delay == 0 {
                ClientBegin(ctx, clientNum, qfalse);
                return;
            }

            AddBotToSpawnQueue(ctx, clientNum, delay);
        }
    }
}

/// Raven `Svcmd_AddBot_f`.
///
/// Source: `oracle/codemp/game/g_bot.c:1041-1093`
pub fn Svcmd_AddBot_f(ctx: GameContext<'_>) {
    unsafe {
        // are bots enabled?
        if trap::Cvar_VariableIntegerValue(
            ctx.engine,
            GCvarVariableIntegerValueArgs::new(CString::new("bot_enable").unwrap()),
        ) == 0
        {
            return;
        }

        // name
        let mut name: [c_char; MAX_TOKEN_CHARS] = [0; MAX_TOKEN_CHARS];
        trap::Argv(
            ctx.engine,
            GArgvArgs::new(1, name.as_mut_ptr(), name.len() as c_int),
        );
        if name[0] == 0 {
            trap::Printf(
                ctx.engine,
                GPrintArgs::new(
                    CString::new(
                        "Usage: Addbot <botname> [skill 1-5] [team] [msec delay] [altname]\n",
                    )
                    .unwrap(),
                ),
            );
            return;
        }

        // skill
        let mut string: [c_char; MAX_TOKEN_CHARS] = [0; MAX_TOKEN_CHARS];
        trap::Argv(
            ctx.engine,
            GArgvArgs::new(2, string.as_mut_ptr(), string.len() as c_int),
        );
        let skill: f32 = if string[0] == 0 {
            4.0
        } else {
            crate::bg_lib::atof(string.as_ptr()) as f32
        };

        // team
        let mut team: [c_char; MAX_TOKEN_CHARS] = [0; MAX_TOKEN_CHARS];
        trap::Argv(
            ctx.engine,
            GArgvArgs::new(3, team.as_mut_ptr(), team.len() as c_int),
        );

        // delay
        trap::Argv(
            ctx.engine,
            GArgvArgs::new(4, string.as_mut_ptr(), string.len() as c_int),
        );
        let delay: c_int = if string[0] == 0 {
            0
        } else {
            atoi(string.as_ptr())
        };

        // alternative name
        let mut altname: [c_char; MAX_TOKEN_CHARS] = [0; MAX_TOKEN_CHARS];
        trap::Argv(
            ctx.engine,
            GArgvArgs::new(5, altname.as_mut_ptr(), altname.len() as c_int),
        );

        G_AddBot(
            ctx,
            name.as_ptr(),
            skill,
            team.as_ptr(),
            delay,
            altname.as_mut_ptr(),
        );

        // if this was issued during gameplay and we are playing locally,
        // go ahead and load the bot's media immediately
        if (*ctx.world).level.time - (*ctx.world).level.startTime > 1000
            && trap::Cvar_VariableIntegerValue(
                ctx.engine,
                GCvarVariableIntegerValueArgs::new(CString::new("cl_running").unwrap()),
            ) != 0
        {
            // FIXME: spelled wrong, but not changing for demo
            trap::SendServerCommand(
                ctx.engine,
                GSendServerCommandArgs::new(-1, CString::new("loaddefered\n").unwrap()),
            );
        }
    }
}

/// Raven `Svcmd_BotList_f`.
///
/// Source: `oracle/codemp/game/g_bot.c:1100-1127`
pub fn Svcmd_BotList_f(ctx: GameContext<'_>) {
    unsafe {
        trap::Printf(
            ctx.engine,
            GPrintArgs::new(
                CString::new(
                    "^1name             model            personality              funname\n",
                )
                .unwrap(),
            ),
        );

        let world = &*ctx.world;
        for i in 0..world.globals.g_numBots {
            let mut name = cstr_to_str(Info_ValueForKey(
                world.globals.g_botInfos[i as usize],
                cstr("name").as_ptr(),
            ));
            if name.is_empty() {
                name = "Padawan".to_string();
            }
            let mut funname = cstr_to_str(Info_ValueForKey(
                world.globals.g_botInfos[i as usize],
                cstr("funname").as_ptr(),
            ));
            if funname.is_empty() {
                funname = "".to_string();
            }
            let mut model = cstr_to_str(Info_ValueForKey(
                world.globals.g_botInfos[i as usize],
                cstr("model").as_ptr(),
            ));
            if model.is_empty() {
                model = "kyle/default".to_string();
            }
            let mut personality = cstr_to_str(Info_ValueForKey(
                world.globals.g_botInfos[i as usize],
                cstr("personality").as_ptr(),
            ));
            if personality.is_empty() {
                personality = "botfiles/kyle.jkb".to_string();
            }
            let line = format!(
                "{:<16} {:<16} {:<20} {:<20}\n",
                name, model, personality, funname
            );
            trap::Printf(ctx.engine, GPrintArgs::new(CString::new(line).unwrap()));
        }
    }
}

/// Raven `G_LoadBotsFromFile`.
///
/// Source: `oracle/codemp/game/g_bot.c:1194-1215`
pub fn G_LoadBotsFromFile(ctx: GameContext<'_>, filename: *mut c_char) {
    unsafe {
        let mut f: fileHandle_t = 0;
        let filename_s = cstr_to_str(filename);
        let len = trap::FS_FOpenFile(
            ctx.engine,
            GFsFopenFileArgs::new(CString::new(filename_s.clone()).unwrap(), &mut f, FS_READ),
        );
        if f == 0 {
            let s = format!(
                "{}file not found: {}\n",
                S_COLOR_RED.to_string_lossy(),
                filename_s
            );
            trap::Printf(ctx.engine, GPrintArgs::new(CString::new(s).unwrap()));
            return;
        }
        if len >= MAX_BOTS_TEXT as c_int {
            let s = format!(
                "{}file too large: {} is {}, max allowed is {}",
                S_COLOR_RED.to_string_lossy(),
                filename_s,
                len,
                MAX_BOTS_TEXT
            );
            trap::Printf(ctx.engine, GPrintArgs::new(CString::new(s).unwrap()));
            trap::FS_FCloseFile(ctx.engine, GFsFcloseFileArgs::new(f));
            return;
        }

        let mut buf: [c_char; MAX_BOTS_TEXT] = [0; MAX_BOTS_TEXT];
        trap::FS_Read(
            ctx.engine,
            GFsReadArgs::new(buf.as_mut_ptr() as *mut u8, len, f),
        );
        buf[len as usize] = 0;
        trap::FS_FCloseFile(ctx.engine, GFsFcloseFileArgs::new(f));

        let g_numBots = (*ctx.world).globals.g_numBots;
        let added = G_ParseInfos(
            ctx,
            buf.as_mut_ptr(),
            MAX_BOTS - g_numBots,
            &mut (*ctx.world).globals.g_botInfos[g_numBots as usize] as *mut *mut c_char,
        );
        (*ctx.world).globals.g_numBots += added;
    }
}

/// Raven `G_LoadBots`.
///
/// Source: `oracle/codemp/game/g_bot.c:1222-1256`
pub fn G_LoadBots(ctx: GameContext<'_>) {
    unsafe {
        if trap::Cvar_VariableIntegerValue(
            ctx.engine,
            GCvarVariableIntegerValueArgs::new(CString::new("bot_enable").unwrap()),
        ) == 0
        {
            return;
        }

        (*ctx.world).globals.g_numBots = 0;

        let mut botsFile = vmCvar_t::zeroed();
        trap::Cvar_Register(
            ctx.engine,
            GCvarRegisterArgs::new(
                &mut botsFile as *mut vmCvar_t,
                CString::new("g_botsFile").unwrap(),
                CString::new("").unwrap(),
                CVAR_INIT | CVAR_ROM,
            ),
        );
        if botsFile.string[0] != 0 {
            G_LoadBotsFromFile(ctx, botsFile.string.as_mut_ptr());
        } else {
            //G_LoadBotsFromFile("scripts/bots.txt");
            let mut default_path: [c_char; 128] = [0; 128];
            write_cstr_field(&mut default_path, "botfiles/bots.txt");
            G_LoadBotsFromFile(ctx, default_path.as_mut_ptr());
        }

        // get all bots from .bot files
        let mut dirlist: [c_char; 1024] = [0; 1024];
        let numdirs = trap::FS_GetFileList(
            ctx.engine,
            GFsGetfilelistArgs::new(
                CString::new("scripts").unwrap(),
                CString::new(".bot").unwrap(),
                dirlist.as_mut_ptr() as *mut u8,
                1024,
            ),
        );
        let mut dirptr = dirlist.as_ptr();
        for _ in 0..numdirs {
            let dirlen = CStr::from_ptr(dirptr).to_bytes().len();
            let mut filename: [c_char; 128] = [0; 128];
            write_cstr_field(&mut filename, &format!("scripts/{}", cstr_to_str(dirptr)));
            G_LoadBotsFromFile(ctx, filename.as_mut_ptr());
            dirptr = dirptr.add(dirlen + 1);
        }
    }
}

/// Raven `G_GetBotInfoByNumber`.
///
/// Source: `oracle/codemp/game/g_bot.c:1265-1271`
pub fn G_GetBotInfoByNumber(ctx: GameContext<'_>, num: c_int) -> *mut c_char {
    unsafe {
        let world = &*ctx.world;
        if num < 0 || num >= world.globals.g_numBots {
            let s = format!(
                "{}Invalid bot number: {}\n",
                S_COLOR_RED.to_string_lossy(),
                num
            );
            trap::Printf(ctx.engine, GPrintArgs::new(CString::new(s).unwrap()));
            return core::ptr::null_mut();
        }
        world.globals.g_botInfos[num as usize]
    }
}

/// Raven `G_GetBotInfoByName`.
///
/// Source: `oracle/codemp/game/g_bot.c:1279-1291`
pub fn G_GetBotInfoByName(ctx: GameContext<'_>, name: *const c_char) -> *mut c_char {
    unsafe {
        let world = &*ctx.world;
        for n in 0..world.globals.g_numBots {
            let value =
                Info_ValueForKey(world.globals.g_botInfos[n as usize], cstr("name").as_ptr());
            if Q_stricmp(value, name) == 0 {
                return world.globals.g_botInfos[n as usize];
            }
        }
        core::ptr::null_mut()
    }
}

/// Raven `G_InitBots`.
///
/// Source: `oracle/codemp/game/g_bot.c:1302-1311`
pub fn G_InitBots(ctx: GameContext<'_>, restart: qboolean) {
    unsafe {
        G_LoadBots(ctx);
        G_LoadArenas(ctx);

        trap::Cvar_Register(
            ctx.engine,
            GCvarRegisterArgs::new(
                &mut (*ctx.world).globals.bot_minplayers as *mut vmCvar_t,
                CString::new("bot_minplayers").unwrap(),
                CString::new("0").unwrap(),
                CVAR_SERVERINFO,
            ),
        );

        //rww - new bot route stuff
        LoadPath_ThisLevel(ctx);
        //end rww
    }
}
