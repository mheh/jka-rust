// PORT-COMPLETE: g_bot.c
//! FAITHFUL port of `oracle/codemp/game/g_bot.c`.
#![allow(non_snake_case, unused, clippy::all)]

use crate::client::client_connected::CON_CONNECTED;
use crate::prelude::*;
use crate::trap;
use mp_bg::public::gametype::{
    GT_CTF, GT_CTY, GT_DUEL, GT_FFA, GT_HOLOCRON, GT_JEDIMASTER, GT_POWERDUEL, GT_SIEGE, GT_TEAM,
};
use native_string::atof::atof;
use native_string::atoi::atoi;

use crate::ai_main::BotAISetupClient;
use crate::ai_wpnav::LoadPath_ThisLevel;
use crate::g_client::{ClientBegin, ClientConnect, ClientUserinfoChanged, PickTeam};
use crate::g_cmds::SetTeam;
use crate::g_main::{Com_Printf, G_GetStringEdString, G_PowerDuelCount, G_Printf};
use crate::g_mem::G_Alloc;
use crate::g_session::G_ReadSessionData;
use crate::g_team::{S_COLOR_RED, S_COLOR_YELLOW};
use crate::level::bot_settings::bot_settings_t;
use crate::q_shared::{COM_Parse, COM_ParseExt, Info_SetValueForKey, Q_CleanStr};
use native_string::cstr::buf_to_string;
use native_string::info::Info_ValueForKey;
use native_string::q_string::Q_stricmp;
use mp_bg::public::duel_team::duelTeam_t::{DUELTEAM_DOUBLE, DUELTEAM_LONE};

use mp_abi::game::syscalls::G_BOT_ALLOCATE_CLIENT::GBotAllocateClientArgs;

/// Raven `trap_Cvar_VariableValue`.
///
/// Source: `oracle/codemp/game/g_bot.c:36-41`
pub fn trap_Cvar_VariableValue(ctx: &mut GameContext, var_name: &str) -> f32 {
    let buf = trap::Cvar_VariableStringBuffer(ctx.engine, var_name, 128);
    atof(&buf) as f32
}

/// Raven `G_ParseInfos`.
///
/// Source: `oracle/codemp/game/g_bot.c:50-99`
pub fn G_ParseInfos(ctx: &mut GameContext, buf: *const c_char, max: c_int) -> Vec<String> {
    unsafe {
        let mut infos: Vec<String> = Vec::new();
        let mut bufp = buf;

        loop {
            let token = cstr_to_str(COM_Parse(
                &mut ctx.world.bg_state.qs,
                &mut bufp as *mut *const c_char,
            ));
            if token.is_empty() {
                break;
            }
            if token != "{" {
                Com_Printf("Missing { in info file\n");
                break;
            }
            if infos.len() as c_int == max {
                Com_Printf("Max infos exceeded\n");
                break;
            }

            let mut info = String::new();
            loop {
                let token = cstr_to_str(COM_ParseExt(
                    &mut ctx.world.bg_state.qs,
                    &mut bufp as *mut *const c_char,
                    qtrue,
                ));
                if token.is_empty() {
                    Com_Printf("Unexpected end of info file\n");
                    break;
                }
                if token == "}" {
                    break;
                }
                let key = token;

                let token2 = cstr_to_str(COM_ParseExt(
                    &mut ctx.world.bg_state.qs,
                    &mut bufp as *mut *const c_char,
                    qfalse,
                ));
                let value = if token2.is_empty() {
                    "<NULL>".to_string()
                } else {
                    token2
                };
                Info_SetValueForKey(&mut info, &key, &value);
            }

            infos.push(info);
        }
        infos
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
pub fn G_LoadArenasFromFile(ctx: &mut GameContext, filename: &str) {
    let mut f: fileHandle_t = 0;
    let len = trap::FS_FOpenFile(ctx.engine, filename, &mut f, FS_READ);
    if f == 0 {
        let s = format!(
            "{}file not found: {}\n",
            S_COLOR_RED.to_string_lossy(),
            filename
        );
        trap::Printf(ctx.engine, &s);
        return;
    }
    if len >= MAX_ARENAS_TEXT as c_int {
        let s = format!(
            "{}file too large: {} is {}, max allowed is {}",
            S_COLOR_RED.to_string_lossy(),
            filename,
            len,
            MAX_ARENAS_TEXT
        );
        trap::Printf(ctx.engine, &s);
        trap::FS_FCloseFile(ctx.engine, f);
        return;
    }

    let mut buf = [0u8; MAX_ARENAS_TEXT];
    trap::FS_Read(ctx.engine, &mut buf[..len as usize], f);
    trap::FS_FCloseFile(ctx.engine, f);

    let g_numArenas = ctx.world.globals.g_numArenas;
    let mut added = G_ParseInfos(
        ctx,
        buf.as_ptr() as *const c_char,
        MAX_ARENAS - g_numArenas,
    );
    ctx.world.globals.g_numArenas += added.len() as c_int;
    ctx.world.globals.g_arenaInfos.append(&mut added);
}

/// Raven `G_GetMapTypeBits`.
///
/// Source: `oracle/codemp/game/g_bot.c:129-169`
pub fn G_GetMapTypeBits(r#type: &str) -> c_int {
    let mut typeBits: c_int = 0;

    if !r#type.is_empty() {
        let t = r#type;
        if t.contains("ffa") {
            typeBits |= 1 << GT_FFA;
            typeBits |= 1 << GT_TEAM;
        }
        if t.contains("team") {
            typeBits |= 1 << GT_TEAM;
        }
        if t.contains("holocron") {
            typeBits |= 1 << GT_HOLOCRON;
        }
        if t.contains("jedimaster") {
            typeBits |= 1 << GT_JEDIMASTER;
        }
        if t.contains("duel") {
            typeBits |= 1 << GT_DUEL;
            typeBits |= 1 << GT_POWERDUEL;
        }
        if t.contains("powerduel") {
            typeBits |= 1 << GT_DUEL;
            typeBits |= 1 << GT_POWERDUEL;
        }
        if t.contains("siege") {
            typeBits |= 1 << GT_SIEGE;
        }
        if t.contains("ctf") {
            typeBits |= 1 << GT_CTF;
        }
        if t.contains("cty") {
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
pub fn G_DoesMapSupportGametype(ctx: &mut GameContext, mapname: &str, gametype: c_int) -> bool {
    if ctx.world.globals.g_arenaInfos.is_empty() {
        return false;
    }
    if mapname.is_empty() {
        return false;
    }

    let mut thisLevel: c_int = -1;
    for n in 0..ctx.world.globals.g_numArenas {
        let r#type = Info_ValueForKey(&ctx.world.globals.g_arenaInfos[n as usize], "map");
        if Q_stricmp(mapname, &r#type) == 0 {
            thisLevel = n;
            break;
        }
    }

    if thisLevel == -1 {
        return false;
    }

    let r#type = Info_ValueForKey(&ctx.world.globals.g_arenaInfos[thisLevel as usize], "type");
    let typeBits = G_GetMapTypeBits(&r#type);
    if typeBits & (1 << gametype) != 0 {
        return true;
    }

    false
}

/// Raven `G_RefreshNextMap`.
///
/// Source: `oracle/codemp/game/g_bot.c:216-288`
pub fn G_RefreshNextMap(ctx: &mut GameContext, gametype: c_int, forced: qboolean) -> Option<String> {
    if ctx.world.cvars.g_autoMapCycle.integer == 0 && forced == 0 {
        return None;
    }
    if ctx.world.globals.g_arenaInfos.is_empty() {
        return None;
    }

    let mut mapname = vmCvar_t::zeroed();
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut mapname),
        "mapname",
        "",
        CVAR_SERVERINFO | CVAR_ROM,
    );
    let mapname_s = unsafe { cstr_to_str(mapname.string.as_ptr()) };

    let mut thisLevel: c_int = 0;
    for n in 0..ctx.world.globals.g_numArenas {
        let r#type = Info_ValueForKey(&ctx.world.globals.g_arenaInfos[n as usize], "map");
        if Q_stricmp(&mapname_s, &r#type) == 0 {
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
            || n as usize >= ctx.world.globals.g_arenaInfos.len()
            || n >= ctx.world.globals.g_numArenas
        {
            if loopingUp != 0 {
                break;
            }
            n = 0;
            loopingUp = qtrue;
        }

        let r#type = Info_ValueForKey(&ctx.world.globals.g_arenaInfos[n as usize], "type");
        let typeBits = G_GetMapTypeBits(&r#type);
        if typeBits & (1 << gametype) != 0 {
            desiredMap = n;
            break;
        }

        n += 1;
    }

    if desiredMap == thisLevel {
        trap::Cvar_Set(ctx.engine, "nextmap", "map_restart 0");
    } else {
        let r#type = Info_ValueForKey(&ctx.world.globals.g_arenaInfos[desiredMap as usize], "map");
        let cmd = format!("map {}", r#type);
        trap::Cvar_Set(ctx.engine, "nextmap", &cmd);
    }

    Some(Info_ValueForKey(
        &ctx.world.globals.g_arenaInfos[desiredMap as usize],
        "map",
    ))
}

/// Raven `G_LoadArenas`.
///
/// Source: `oracle/codemp/game/g_bot.c:295-321`
pub fn G_LoadArenas(ctx: &mut GameContext) {
    ctx.world.globals.g_numArenas = 0;
    ctx.world.globals.g_arenaInfos.clear();

    let mut dirlist = [0u8; 1024];
    let numdirs = trap::FS_GetFileList(ctx.engine, "scripts", ".arena", &mut dirlist);
    let mut dirptr = 0usize;
    let gametype = ctx.world.cvars.g_gametype.integer;
    for _ in 0..numdirs {
        let dirname = buf_to_string(&dirlist[dirptr..]);
        dirptr += dirname.len() + 1;
        G_LoadArenasFromFile(ctx, &format!("scripts/{}", dirname));
    }

    for n in 0..ctx.world.globals.g_numArenas {
        Info_SetValueForKey(
            &mut ctx.world.globals.g_arenaInfos[n as usize],
            "num",
            &format!("{}", n),
        );
    }

    G_RefreshNextMap(ctx, gametype, qfalse);
}

/// Raven `G_GetArenaInfoByMap`.
///
/// Source: `oracle/codemp/game/g_bot.c:329-339`
pub fn G_GetArenaInfoByMap(ctx: &mut GameContext, map: &str) -> Option<String> {
    for n in 0..ctx.world.globals.g_numArenas {
        if Q_stricmp(
            &Info_ValueForKey(&ctx.world.globals.g_arenaInfos[n as usize], "map"),
            map,
        ) == 0
        {
            return Some(ctx.world.globals.g_arenaInfos[n as usize].clone());
        }
    }
    None
}

/// Raven `G_AddRandomBot`.
///
/// Source: `oracle/codemp/game/g_bot.c:373-454`
pub fn G_AddRandomBot(ctx: &mut GameContext, team: c_int) {
    unsafe {
        let mut num: c_int = 0;
        for n in 0..ctx.world.globals.g_numBots {
            let value = Info_ValueForKey(&ctx.world.globals.g_botInfos[n as usize], "name");
            let mut i: c_int = 0;
            while i < ctx.world.cvars.g_maxclients.integer {
                let cl = &ctx.world.clients[i as usize];
                if cl.pers.connected != CON_CONNECTED {
                    i += 1;
                    continue;
                }
                if ctx.world.g_entities[cl.ps.clientNum as usize].r.svFlags & SVF_BOT == 0 {
                    i += 1;
                    continue;
                }
                if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
                    if team >= 0 && cl.sess.siegeDesiredTeam != team {
                        i += 1;
                        continue;
                    }
                } else if team >= 0 && cl.sess.sessionTeam != team {
                    i += 1;
                    continue;
                }
                if Q_stricmp(&value, &cl.pers.netname) == 0 {
                    break;
                }
                i += 1;
            }
            if i >= ctx.world.cvars.g_maxclients.integer {
                num += 1;
            }
        }

        num = (ctx.world.bg_state.rng.random() * num as f32) as c_int;

        for n in 0..ctx.world.globals.g_numBots {
            let value = Info_ValueForKey(&ctx.world.globals.g_botInfos[n as usize], "name");
            let mut i: c_int = 0;
            while i < ctx.world.cvars.g_maxclients.integer {
                let cl = &ctx.world.clients[i as usize];
                if cl.pers.connected != CON_CONNECTED {
                    i += 1;
                    continue;
                }
                if ctx.world.g_entities[cl.ps.clientNum as usize].r.svFlags & SVF_BOT == 0 {
                    i += 1;
                    continue;
                }
                if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
                    if team >= 0 && cl.sess.siegeDesiredTeam != team {
                        i += 1;
                        continue;
                    }
                } else if team >= 0 && cl.sess.sessionTeam != team {
                    i += 1;
                    continue;
                }
                if Q_stricmp(&value, &cl.pers.netname) == 0 {
                    break;
                }
                i += 1;
            }
            if i >= ctx.world.cvars.g_maxclients.integer {
                num -= 1;
                if num <= 0 {
                    let skill = trap_Cvar_VariableValue(ctx, "g_spSkill");
                    let teamstr = if team == TEAM_RED {
                        "red"
                    } else if team == TEAM_BLUE {
                        "blue"
                    } else {
                        ""
                    };
                    let mut netname: [c_char; 36] = [0; 36];
                    write_cstr_field(&mut netname, &value);
                    Q_CleanStr(netname.as_mut_ptr());
                    let cmd = format!(
                        "addbot \"{}\" {} {} {}\n",
                        cstr_to_str(netname.as_ptr()),
                        skill,
                        teamstr,
                        0
                    );
                    trap::SendConsoleCommand(ctx.engine, cbufExec_t::EXEC_INSERT as c_int, &cmd);
                    return;
                }
            }
        }
    }
}

/// Raven `G_RemoveRandomBot`.
///
/// Source: `oracle/codemp/game/g_bot.c:461-492`
pub fn G_RemoveRandomBot(ctx: &mut GameContext, team: c_int) -> bool {
    unsafe {
        for i in 0..ctx.world.cvars.g_maxclients.integer {
            let cl = &ctx.world.clients[i as usize];
            if cl.pers.connected != CON_CONNECTED {
                continue;
            }
            if ctx.world.g_entities[cl.ps.clientNum as usize].r.svFlags & SVF_BOT == 0 {
                continue;
            }
            if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
                if team >= 0 && cl.sess.siegeDesiredTeam != team {
                    continue;
                }
            } else if team >= 0 && cl.sess.sessionTeam != team {
                continue;
            }

            let mut netname: [c_char; 36] = [0; 36];
            write_cstr_field(&mut netname, &cl.pers.netname);
            Q_CleanStr(netname.as_mut_ptr());
            let cmd = format!("kick \"{}\"\n", cstr_to_str(netname.as_ptr()));
            trap::SendConsoleCommand(ctx.engine, cbufExec_t::EXEC_INSERT as c_int, &cmd);
            return true;
        }
        false
    }
}

/// Raven `G_CountHumanPlayers`.
///
/// Source: `oracle/codemp/game/g_bot.c:499-518`
pub fn G_CountHumanPlayers(ctx: &mut GameContext, team: c_int) -> c_int {
    let mut num: c_int = 0;
    for i in 0..ctx.world.cvars.g_maxclients.integer {
        let cl = &ctx.world.clients[i as usize];
        if cl.pers.connected != CON_CONNECTED {
            continue;
        }
        if ctx.world.g_entities[cl.ps.clientNum as usize].r.svFlags & SVF_BOT != 0 {
            continue;
        }
        if team >= 0 && cl.sess.sessionTeam as c_int != team {
            continue;
        }
        num += 1;
    }
    num
}

/// Raven `G_CountBotPlayers`.
///
/// Source: `oracle/codemp/game/g_bot.c:525-562`
pub fn G_CountBotPlayers(ctx: &mut GameContext, team: c_int) -> c_int {
    let mut num: c_int = 0;
    for i in 0..ctx.world.cvars.g_maxclients.integer {
        let cl = &ctx.world.clients[i as usize];
        if cl.pers.connected != CON_CONNECTED {
            continue;
        }
        if ctx.world.g_entities[cl.ps.clientNum as usize].r.svFlags & SVF_BOT == 0 {
            continue;
        }
        if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
            if team >= 0 && cl.sess.siegeDesiredTeam != team {
                continue;
            }
        } else if team >= 0 && cl.sess.sessionTeam != team {
            continue;
        }
        num += 1;
    }
    for n in 0..BOT_SPAWN_QUEUE_DEPTH {
        if ctx.world.globals.botSpawnQueue[n].spawnTime == 0 {
            continue;
        }
        if ctx.world.globals.botSpawnQueue[n].spawnTime > ctx.world.level.time {
            continue;
        }
        num += 1;
    }
    num
}

/// Raven `G_CheckMinimumPlayers`.
///
/// Source: `oracle/codemp/game/g_bot.c:569-690`
///
/// The `#if 0`-guarded team-balance tail (g_bot.c:611-688) is Raven-dead
/// code (never compiled); not transcribed, matching the active-code-only
/// faithful-port convention.
pub fn G_CheckMinimumPlayers(ctx: &mut GameContext) {
    if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
        return;
    }
    if ctx.world.level.intermissiontime != 0 {
        return;
    }
    // only check once each 10 seconds
    if ctx.world.globals.checkminimumplayers_time > ctx.world.level.time - 10000 {
        return;
    }
    ctx.world.globals.checkminimumplayers_time = ctx.world.level.time;
    trap::Cvar_Update(ctx.engine, &mut ctx.world.globals.bot_minplayers);
    let mut minplayers = ctx.world.globals.bot_minplayers.integer;
    if minplayers <= 0 {
        return;
    }
    if minplayers > ctx.world.cvars.g_maxclients.integer {
        minplayers = ctx.world.cvars.g_maxclients.integer;
    }

    let humanplayers = G_CountHumanPlayers(ctx, -1);
    let botplayers = G_CountBotPlayers(ctx, -1);

    if humanplayers + botplayers < minplayers {
        G_AddRandomBot(ctx, -1);
    } else if humanplayers + botplayers > minplayers && botplayers != 0 {
        // try to remove spectators first
        if !G_RemoveRandomBot(ctx, TEAM_SPECTATOR) {
            // just remove the bot that is playing
            G_RemoveRandomBot(ctx, -1);
        }
    }
}

/// Raven `G_CheckBotSpawn`.
///
/// Source: `oracle/codemp/game/g_bot.c:697-719`
pub fn G_CheckBotSpawn(ctx: &mut GameContext) {
    G_CheckMinimumPlayers(ctx);

    for n in 0..BOT_SPAWN_QUEUE_DEPTH {
        if ctx.world.globals.botSpawnQueue[n].spawnTime == 0 {
            continue;
        }
        if ctx.world.globals.botSpawnQueue[n].spawnTime > ctx.world.level.time {
            continue;
        }
        let clientNum = ctx.world.globals.botSpawnQueue[n].clientNum;
        ClientBegin(ctx, clientNum, qfalse);
        ctx.world.globals.botSpawnQueue[n].spawnTime = 0;
    }
}

/// Raven `AddBotToSpawnQueue`.
///
/// Source: `oracle/codemp/game/g_bot.c:727-740`
pub fn AddBotToSpawnQueue(ctx: &mut GameContext, clientNum: c_int, delay: c_int) {
    for n in 0..BOT_SPAWN_QUEUE_DEPTH {
        if ctx.world.globals.botSpawnQueue[n].spawnTime == 0 {
            ctx.world.globals.botSpawnQueue[n].spawnTime = ctx.world.level.time + delay;
            ctx.world.globals.botSpawnQueue[n].clientNum = clientNum;
            return;
        }
    }

    G_Printf(
        ctx,
        &format!(
            "{}Unable to delay spawn\n",
            S_COLOR_YELLOW.to_string_lossy()
        ),
    );
    ClientBegin(ctx, clientNum, qfalse);
}

/// Raven `G_RemoveQueuedBotBegin`.
///
/// Source: `oracle/codemp/game/g_bot.c:751-760`
pub fn G_RemoveQueuedBotBegin(ctx: &mut GameContext, clientNum: c_int) {
    for n in 0..BOT_SPAWN_QUEUE_DEPTH {
        if ctx.world.globals.botSpawnQueue[n].clientNum == clientNum {
            ctx.world.globals.botSpawnQueue[n].spawnTime = 0;
            return;
        }
    }
}

/// Raven `G_BotConnect`.
///
/// Source: `oracle/codemp/game/g_bot.c:768-784`
pub fn G_BotConnect(ctx: &mut GameContext, clientNum: c_int, restart: bool) -> bool {
    // `MAX_INFO_STRING` resolves via the crate prelude glob.
    unsafe {
        let mut settings: bot_settings_t = core::mem::zeroed();
        let userinfo = trap::GetUserinfo(ctx.engine, clientNum, MAX_INFO_STRING);

        write_cstr_field(
            &mut settings.personalityfile,
            &Info_ValueForKey(&userinfo, "personality"),
        );
        settings.skill = atof(&Info_ValueForKey(&userinfo, "skill")) as f32;
        write_cstr_field(&mut settings.team, &Info_ValueForKey(&userinfo, "team"));

        let ok = BotAISetupClient(
            ctx,
            clientNum,
            &mut settings as *mut bot_settings_t,
            if restart { qtrue } else { qfalse },
        );
        if ok == 0 {
            trap::DropClient(ctx.engine, clientNum, "BotAISetupClient failed");
            return false;
        }

        true
    }
}

/// Raven `G_AddBot`.
///
/// Source: `oracle/codemp/game/g_bot.c:792-1033`
pub fn G_AddBot(
    ctx: &mut GameContext,
    name: &str,
    skill: f32,
    team: &str,
    delay: c_int,
    altname: &str,
) {
    // `MAX_INFO_STRING` resolves via the crate prelude glob.
    unsafe {
        // get the botinfo from bots.txt
        let Some(botinfo) = G_GetBotInfoByName(ctx, name) else {
            G_Printf(
                ctx,
                &format!(
                    "{}Error: Bot '{}' not defined\n",
                    S_COLOR_RED.to_string_lossy(),
                    name
                ),
            );
            return;
        };

        // create the bot's userinfo
        let mut userinfo = String::new();

        let mut botname = Info_ValueForKey(&botinfo, "funname");
        if botname.is_empty() {
            botname = Info_ValueForKey(&botinfo, "name");
        }
        // check for an alternative name
        if !altname.is_empty() {
            botname = altname.to_string();
        }
        Info_SetValueForKey(&mut userinfo, "name", &botname);
        Info_SetValueForKey(&mut userinfo, "rate", "25000");
        Info_SetValueForKey(&mut userinfo, "snaps", "20");
        Info_SetValueForKey(&mut userinfo, "skill", &format!("{:.2}", skill));

        if skill >= 1.0 && skill < 2.0 {
            Info_SetValueForKey(&mut userinfo, "handicap", "50");
        } else if skill >= 2.0 && skill < 3.0 {
            Info_SetValueForKey(&mut userinfo, "handicap", "70");
        } else if skill >= 3.0 && skill < 4.0 {
            Info_SetValueForKey(&mut userinfo, "handicap", "90");
        }

        let mut model = Info_ValueForKey(&botinfo, "model");
        if model.is_empty() {
            model = "kyle/default".to_string();
        }
        Info_SetValueForKey(&mut userinfo, "model", &model);

        let mut gender = Info_ValueForKey(&botinfo, "gender");
        if gender.is_empty() {
            gender = "male".to_string();
        }
        Info_SetValueForKey(&mut userinfo, "sex", &gender);

        let mut color1 = Info_ValueForKey(&botinfo, "color1");
        if color1.is_empty() {
            color1 = "4".to_string();
        }
        Info_SetValueForKey(&mut userinfo, "color1", &color1);

        let mut color2 = Info_ValueForKey(&botinfo, "color2");
        if color2.is_empty() {
            color2 = "4".to_string();
        }
        Info_SetValueForKey(&mut userinfo, "color2", &color2);

        let mut saber1 = Info_ValueForKey(&botinfo, "saber1");
        if saber1.is_empty() {
            saber1 = "single_1".to_string();
        }
        Info_SetValueForKey(&mut userinfo, "saber1", &saber1);

        let mut saber2 = Info_ValueForKey(&botinfo, "saber2");
        if saber2.is_empty() {
            saber2 = "none".to_string();
        }
        Info_SetValueForKey(&mut userinfo, "saber2", &saber2);

        let personality = Info_ValueForKey(&botinfo, "personality");
        if personality.is_empty() {
            Info_SetValueForKey(&mut userinfo, "personality", "botfiles/default.jkb");
        } else {
            Info_SetValueForKey(&mut userinfo, "personality", &personality);
        }

        // have the server allocate a client slot
        let clientNum = trap::BotAllocateClient(ctx.engine, GBotAllocateClientArgs::new());
        if clientNum == -1 {
            let msg = G_GetStringEdString(ctx, "MP_SVGAME", "UNABLE_TO_ADD_BOT");
            let s = format!("print \"{}\n\"", msg);
            trap::SendServerCommand(ctx.engine, -1, &s);
            return;
        }

        // initialize the bot settings
        let team_owned: String;
        if team.is_empty() {
            if ctx.world.cvars.g_gametype.integer >= GT_TEAM {
                if PickTeam(ctx, clientNum) == TEAM_RED {
                    team_owned = "red".to_string();
                } else {
                    team_owned = "blue".to_string();
                }
            } else {
                team_owned = "red".to_string();
            }
        } else {
            team_owned = team.to_string();
        }
        Info_SetValueForKey(&mut userinfo, "skill", &format!("{:5.2}", skill));
        Info_SetValueForKey(&mut userinfo, "team", &team_owned);

        // The bot entity lives at `g_entities[clientNum]`; its `.client`
        // back-pointer aliases `clients[clientNum]` (Raven wires this at
        // `G_InitGame`), so client access re-indexes by `ci = clientNum`.
        let bot_id = EntityId::from_num(clientNum).unwrap();
        let ci = clientNum as usize;
        {
            let bot = ctx.world.entity_mut(bot_id);
            bot.r.svFlags |= SVF_BOT;
            bot.inuse = qtrue;
        }

        // register the userinfo
        trap::SetUserinfo(ctx.engine, clientNum, &userinfo);

        if ctx.world.cvars.g_gametype.integer >= GT_TEAM {
            if Q_stricmp(&team_owned, "red") == 0 {
                ctx.world.client_mut(ci).sess.sessionTeam = TEAM_RED;
            } else if Q_stricmp(&team_owned, "blue") == 0 {
                ctx.world.client_mut(ci).sess.sessionTeam = TEAM_BLUE;
            } else {
                let t = PickTeam(ctx, -1);
                ctx.world.client_mut(ci).sess.sessionTeam = t;
            }
        }

        if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
            let cl = ctx.world.client_mut(ci);
            cl.sess.siegeDesiredTeam = cl.sess.sessionTeam;
            cl.sess.sessionTeam = TEAM_SPECTATOR;
        }

        let preTeam = ctx.world.client(ci).sess.sessionTeam;

        // have it connect to the game as a normal client
        if !ClientConnect(ctx, clientNum, qtrue, qtrue).is_null() {
            return;
        }

        if ctx.world.client(ci).sess.sessionTeam != preTeam {
            let mut userinfo = trap::GetUserinfo(ctx.engine, clientNum, MAX_INFO_STRING);

            if ctx.world.client(ci).sess.sessionTeam == TEAM_SPECTATOR {
                ctx.world.client_mut(ci).sess.sessionTeam = preTeam;
            }

            let team_final = if ctx.world.client(ci).sess.sessionTeam == TEAM_RED {
                "Red".to_string()
            } else if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
                if ctx.world.client(ci).sess.sessionTeam == TEAM_BLUE {
                    "Blue".to_string()
                } else {
                    "s".to_string()
                }
            } else {
                "Blue".to_string()
            };

            Info_SetValueForKey(&mut userinfo, "team", &team_final);
            trap::SetUserinfo(ctx.engine, clientNum, &userinfo);

            let st = ctx.world.client(ci).sess.sessionTeam;
            ctx.world.client_mut(ci).ps.persistant[PERS_TEAM as usize] = st;

            G_ReadSessionData(ctx, clientNum as usize);
            ClientUserinfoChanged(ctx, clientNum);
        }

        if ctx.world.cvars.g_gametype.integer == GT_DUEL
            || ctx.world.cvars.g_gametype.integer == GT_POWERDUEL
        {
            let mut loners: c_int = 0;
            let mut doubles: c_int = 0;

            ctx.world.client_mut(ci).sess.duelTeam = 0;
            G_PowerDuelCount(
                ctx,
                &mut loners as *mut c_int,
                &mut doubles as *mut c_int,
                qtrue,
            );

            if doubles == 0 || loners > doubles / 2 {
                ctx.world.client_mut(ci).sess.duelTeam = DUELTEAM_DOUBLE as c_int;
            } else {
                ctx.world.client_mut(ci).sess.duelTeam = DUELTEAM_LONE as c_int;
            }

            ctx.world.client_mut(ci).sess.sessionTeam = TEAM_SPECTATOR;
            SetTeam(ctx, bot_id, "s");
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
pub fn Svcmd_AddBot_f(ctx: &mut GameContext) {
    // are bots enabled?
    if trap::Cvar_VariableIntegerValue(ctx.engine, "bot_enable") == 0 {
        return;
    }

    // name
    let name = trap::Argv(ctx.engine, 1, MAX_TOKEN_CHARS);
    if name.is_empty() {
        trap::Printf(
            ctx.engine,
            "Usage: Addbot <botname> [skill 1-5] [team] [msec delay] [altname]\n",
        );
        return;
    }

    // skill
    let string = trap::Argv(ctx.engine, 2, MAX_TOKEN_CHARS);
    let skill: f32 = if string.is_empty() {
        4.0
    } else {
        atof(&string) as f32
    };

    // team
    let team = trap::Argv(ctx.engine, 3, MAX_TOKEN_CHARS);

    // delay
    let string = trap::Argv(ctx.engine, 4, MAX_TOKEN_CHARS);
    let delay: c_int = if string.is_empty() { 0 } else { atoi(&string) };

    // alternative name
    let altname = trap::Argv(ctx.engine, 5, MAX_TOKEN_CHARS);

    G_AddBot(ctx, &name, skill, &team, delay, &altname);

    // if this was issued during gameplay and we are playing locally,
    // go ahead and load the bot's media immediately
    if ctx.world.level.time - ctx.world.level.startTime > 1000
        && trap::Cvar_VariableIntegerValue(ctx.engine, "cl_running") != 0
    {
        // FIXME: spelled wrong, but not changing for demo
        trap::SendServerCommand(ctx.engine, -1, "loaddefered\n");
    }
}

/// Raven `Svcmd_BotList_f`.
///
/// Source: `oracle/codemp/game/g_bot.c:1100-1127`
pub fn Svcmd_BotList_f(ctx: &mut GameContext) {
    trap::Printf(
        ctx.engine,
        "^1name             model            personality              funname\n",
    );

    for i in 0..ctx.world.globals.g_numBots {
        let mut name = Info_ValueForKey(&ctx.world.globals.g_botInfos[i as usize], "name");
        if name.is_empty() {
            name = "Padawan".to_string();
        }
        let mut funname = Info_ValueForKey(&ctx.world.globals.g_botInfos[i as usize], "funname");
        if funname.is_empty() {
            funname = "".to_string();
        }
        let mut model = Info_ValueForKey(&ctx.world.globals.g_botInfos[i as usize], "model");
        if model.is_empty() {
            model = "kyle/default".to_string();
        }
        let mut personality =
            Info_ValueForKey(&ctx.world.globals.g_botInfos[i as usize], "personality");
        if personality.is_empty() {
            personality = "botfiles/kyle.jkb".to_string();
        }
        let line = format!(
            "{:<16} {:<16} {:<20} {:<20}\n",
            name, model, personality, funname
        );
        trap::Printf(ctx.engine, &line);
    }
}

/// Raven `G_LoadBotsFromFile`.
///
/// Source: `oracle/codemp/game/g_bot.c:1194-1215`
pub fn G_LoadBotsFromFile(ctx: &mut GameContext, filename: &str) {
    let mut f: fileHandle_t = 0;
    let len = trap::FS_FOpenFile(ctx.engine, filename, &mut f, FS_READ);
    if f == 0 {
        let s = format!(
            "{}file not found: {}\n",
            S_COLOR_RED.to_string_lossy(),
            filename
        );
        trap::Printf(ctx.engine, &s);
        return;
    }
    if len >= MAX_BOTS_TEXT as c_int {
        let s = format!(
            "{}file too large: {} is {}, max allowed is {}",
            S_COLOR_RED.to_string_lossy(),
            filename,
            len,
            MAX_BOTS_TEXT
        );
        trap::Printf(ctx.engine, &s);
        trap::FS_FCloseFile(ctx.engine, f);
        return;
    }

    let mut buf = [0u8; MAX_BOTS_TEXT];
    trap::FS_Read(ctx.engine, &mut buf[..len as usize], f);
    trap::FS_FCloseFile(ctx.engine, f);

    let g_numBots = ctx.world.globals.g_numBots;
    let mut added = G_ParseInfos(ctx, buf.as_ptr() as *const c_char, MAX_BOTS - g_numBots);
    ctx.world.globals.g_numBots += added.len() as c_int;
    ctx.world.globals.g_botInfos.append(&mut added);
}

/// Raven `G_LoadBots`.
///
/// Source: `oracle/codemp/game/g_bot.c:1222-1256`
pub fn G_LoadBots(ctx: &mut GameContext) {
    if trap::Cvar_VariableIntegerValue(ctx.engine, "bot_enable") == 0 {
        return;
    }

    ctx.world.globals.g_numBots = 0;
    ctx.world.globals.g_botInfos.clear();

    let mut botsFile = vmCvar_t::zeroed();
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut botsFile),
        "g_botsFile",
        "",
        CVAR_INIT | CVAR_ROM,
    );
    if botsFile.string[0] != 0 {
        let bots_file = unsafe { cstr_to_str(botsFile.string.as_ptr()) };
        G_LoadBotsFromFile(ctx, &bots_file);
    } else {
        //G_LoadBotsFromFile("scripts/bots.txt");
        G_LoadBotsFromFile(ctx, "botfiles/bots.txt");
    }

    // get all bots from .bot files
    let mut dirlist = [0u8; 1024];
    let numdirs = trap::FS_GetFileList(ctx.engine, "scripts", ".bot", &mut dirlist);
    let mut dirptr = 0usize;
    for _ in 0..numdirs {
        let dirname = buf_to_string(&dirlist[dirptr..]);
        dirptr += dirname.len() + 1;
        G_LoadBotsFromFile(ctx, &format!("scripts/{}", dirname));
    }
}

/// Raven `G_GetBotInfoByNumber`.
///
/// Source: `oracle/codemp/game/g_bot.c:1265-1271`
pub fn G_GetBotInfoByNumber(ctx: &mut GameContext, num: c_int) -> Option<String> {
    if num < 0 || num >= ctx.world.globals.g_numBots {
        let s = format!(
            "{}Invalid bot number: {}\n",
            S_COLOR_RED.to_string_lossy(),
            num
        );
        trap::Printf(ctx.engine, &s);
        return None;
    }
    Some(ctx.world.globals.g_botInfos[num as usize].clone())
}

/// Raven `G_GetBotInfoByName`.
///
/// Source: `oracle/codemp/game/g_bot.c:1279-1291`
pub fn G_GetBotInfoByName(ctx: &mut GameContext, name: &str) -> Option<String> {
    for n in 0..ctx.world.globals.g_numBots {
        let value = Info_ValueForKey(&ctx.world.globals.g_botInfos[n as usize], "name");
        if Q_stricmp(&value, name) == 0 {
            return Some(ctx.world.globals.g_botInfos[n as usize].clone());
        }
    }
    None
}

/// Raven `G_InitBots`.
///
/// Source: `oracle/codemp/game/g_bot.c:1302-1311`
pub fn G_InitBots(ctx: &mut GameContext, restart: qboolean) {
    G_LoadBots(ctx);
    G_LoadArenas(ctx);

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.globals.bot_minplayers),
        "bot_minplayers",
        "0",
        CVAR_SERVERINFO,
    );

    //rww - new bot route stuff
    LoadPath_ThisLevel(ctx);
    //end rww
}
