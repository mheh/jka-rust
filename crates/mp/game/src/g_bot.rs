// PORT-COMPLETE: g_bot.c 4/25
//! FAITHFUL port of `oracle/oracle/codemp/game/g_bot.c`.
//!
//! Pass-2 (`ctx: GameContext<'_>` threaded per fork 8): `G_GetMapTypeBits`,
//! `trap_Cvar_VariableValue`, and `G_CountHumanPlayers` are implemented —
//! the rest stay parked on state this crate doesn't yet own: `g_arenaInfos`/
//! `g_botInfos` (char* info-string tables), `botSpawnQueue`/`bot_minplayers`/
//! `checkminimumplayers_time` (no `GameGlobals` field yet — porters may not
//! add one), the still-`todo!()` `va`/`Com_Printf` C-varargs seam, the
//! unported `bot_settings_s`, and missing `MAX_TOKEN_CHARS`/`atoi`/
//! `CVAR_INIT`/`CVAR_ROM`. See each fn's PORT-ESCALATION marker.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::trap;
use crate::client::client_connected::CON_CONNECTED;
use mp_bg::public::gametype::{
    GT_CTF, GT_CTY, GT_DUEL, GT_FFA, GT_HOLOCRON, GT_JEDIMASTER, GT_POWERDUEL, GT_SIEGE, GT_TEAM,
};
use core::ffi::CStr;

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
/// Source: `oracle/oracle/codemp/game/g_bot.c:36-41`
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

// PORT-ESCALATION(variadic-c-abi): the tail allocation-size calc calls
// `va("%d", MAX_ARENAS)` — the staged `va` signature is fixed single-arg
// (`format: *const c_char`, no real C varargs), so this can't be threaded
// through faithfully without the variadic-seam decision other porters
// parked the same way (see `Com_Printf`/`COM_ParseError` in this crate).
/// Raven `G_ParseInfos`.
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:50-99`
pub fn G_ParseInfos(
    ctx: GameContext<'_>,
    buf: *mut c_char,
    max: c_int,
    infos: *mut *mut c_char,
) -> c_int {
    todo!("Port G_ParseInfos — parked: variadic-c-abi")
}

// PORT-ESCALATION(missing-global): reads/writes `g_arenaInfos`/`g_numArenas`
// (g_bot.c-owned globals, fork ruling 1). `g_numArenas` is a `GameGlobals`
// field, but `g_arenaInfos` (the `char *[MAX_ARENAS]` info-string table) has
// no home yet — not a `GameGlobals` placeholder, and porters may not add
// fields. Also calls the still-`todo!()` `va`/`trap_Printf` variadic seam.
/// Raven `G_LoadArenasFromFile`.
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:106-127`
pub fn G_LoadArenasFromFile(
    ctx: GameContext<'_>,
    filename: *mut c_char,
) {
    todo!("Port G_LoadArenasFromFile — parked: missing-global: g_arenaInfos")
}

/// Raven `G_GetMapTypeBits`.
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:129-169`
///
/// # Safety
/// `r#type` must be a valid NUL-terminated C string.
pub unsafe fn G_GetMapTypeBits(
    r#type: *mut c_char,
) -> c_int {
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

// PORT-ESCALATION(missing-global): reads `g_arenaInfos`/`g_numArenas`;
// `g_arenaInfos` has no `GameGlobals` field yet (see `G_LoadArenasFromFile`).
/// Raven `G_DoesMapSupportGametype`.
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:171-213`
pub fn G_DoesMapSupportGametype(
    ctx: GameContext<'_>,
    mapname: *const c_char,
    gametype: c_int,
) -> qboolean {
    todo!("Port G_DoesMapSupportGametype — parked: missing-global: g_arenaInfos")
}

// PORT-ESCALATION(missing-global): reads `g_arenaInfos` (no `GameGlobals`
// field yet) and calls the still-`todo!()` `va` variadic seam (via
// `trap_Cvar_Set`'s format string).
/// Raven `G_RefreshNextMap`.
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:216-288`
pub fn G_RefreshNextMap(
    ctx: GameContext<'_>,
    gametype: c_int,
    forced: qboolean,
) -> *const c_char {
    todo!("Port G_RefreshNextMap — parked: missing-global: g_arenaInfos")
}

// PORT-ESCALATION(missing-global): writes `g_numArenas`/reads `g_arenaInfos`
// (no `GameGlobals` field yet, see `G_LoadArenasFromFile`) via
// `Info_SetValueForKey`/`G_RefreshNextMap`.
/// Raven `G_LoadArenas`.
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:295-321`
pub fn G_LoadArenas(ctx: GameContext<'_>) {
    todo!("Port G_LoadArenas — parked: missing-global: g_arenaInfos")
}

// PORT-ESCALATION(missing-global): reads `g_arenaInfos` — no `GameGlobals`
// field yet (see `G_LoadArenasFromFile`).
/// Raven `G_GetArenaInfoByMap`.
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:329-339`
pub fn G_GetArenaInfoByMap(
    ctx: GameContext<'_>,
    map: *const c_char,
) -> *const c_char {
    todo!("Port G_GetArenaInfoByMap — parked: missing-global: g_arenaInfos")
}

// PORT-ESCALATION(missing-global): reads `g_botInfos` (no `GameGlobals` field
// yet, char*[MAX_BOTS] info-string table) and calls the ruling-3 shared LCG
// (`random()`) which is not yet threaded into `GameContext`; also needs the
// still-`todo!()` `va` variadic seam.
/// Raven `G_AddRandomBot`.
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:373-454`
pub fn G_AddRandomBot(
    ctx: GameContext<'_>,
    team: c_int,
) {
    todo!("Port G_AddRandomBot — parked: missing-global: g_botInfos")
}

// PORT-ESCALATION(variadic-abi): the kick command uses `va("kick \"%s\"\n", netname)`;
// `va` is itself parked on the C-varargs seam decision (still `todo!()`).
/// Raven `G_RemoveRandomBot`.
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:461-492`
pub fn G_RemoveRandomBot(
    ctx: GameContext<'_>,
    team: c_int,
) -> c_int {
    todo!("Port G_RemoveRandomBot — parked: variadic-abi")
}

/// Raven `G_CountHumanPlayers`.
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:499-518`
pub fn G_CountHumanPlayers(ctx: GameContext<'_>, team: c_int) -> c_int {
    unsafe {
        let world = &*ctx.world;
        let mut num: c_int = 0;
        for i in 0..world.cvars.g_maxclients.integer {
            let cl = &world.clients[i as usize];
            if cl.pers.connected != CON_CONNECTED {
                continue;
            }
            if world.entities[cl.ps.clientNum as usize].r.svFlags & SVF_BOT != 0 {
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

// PORT-ESCALATION(missing-global): reads `botSpawnQueue`, whose `GameGlobals`
// field is a `()` placeholder (the `botSpawnQueue_t` array type is not yet
// ported) — cannot index/read `.spawnTime` without that type existing.
/// Raven `G_CountBotPlayers`.
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:525-562`
pub fn G_CountBotPlayers(
    ctx: GameContext<'_>,
    team: c_int,
) -> c_int {
    todo!("Port G_CountBotPlayers — parked: missing-global: botSpawnQueue")
}

// PORT-ESCALATION(missing-global): reads `bot_minplayers` (cvar handle, not
// yet a `GameCvars` field) and the fn-scope static `checkminimumplayers_time`
// (ruling 5: promotes to a `GameWorld`/`GameGlobals` field) — neither exists
// yet; porters may not add fields.
/// Raven `G_CheckMinimumPlayers`.
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:569-690`
pub fn G_CheckMinimumPlayers(ctx: GameContext<'_>) {
    todo!("Port G_CheckMinimumPlayers — parked: missing-global: bot_minplayers/checkminimumplayers_time")
}

// PORT-ESCALATION(missing-global): reads/writes `botSpawnQueue`, whose
// `GameGlobals` field is a `()` placeholder (see `G_CountBotPlayers`).
/// Raven `G_CheckBotSpawn`.
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:697-719`
pub fn G_CheckBotSpawn(ctx: GameContext<'_>) {
    todo!("Port G_CheckBotSpawn — parked: missing-global: botSpawnQueue")
}

// PORT-ESCALATION(missing-global): writes `botSpawnQueue`, whose `GameGlobals`
// field is a `()` placeholder (see `G_CountBotPlayers`).
/// Raven `AddBotToSpawnQueue`.
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:727-740`
pub fn AddBotToSpawnQueue(
    ctx: GameContext<'_>,
    clientNum: c_int,
    delay: c_int,
) {
    todo!("Port AddBotToSpawnQueue — parked: missing-global: botSpawnQueue")
}

// PORT-ESCALATION(missing-global): writes `botSpawnQueue`, whose `GameGlobals`
// field is a `()` placeholder (see `G_CountBotPlayers`).
/// Raven `G_RemoveQueuedBotBegin`.
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:751-760`
pub fn G_RemoveQueuedBotBegin(
    ctx: GameContext<'_>,
    clientNum: c_int,
) {
    todo!("Port G_RemoveQueuedBotBegin — parked: missing-global: botSpawnQueue")
}

// PORT-ESCALATION(missing-type): needs a `bot_settings_t` value (Raven
// `bot_settings_s`) to pass to `BotAISetupClient`; the type is unported —
// the callee's staged signature only carries a `*mut c_void`, so there is
// nowhere to write `.personalityfile`/`.skill`/`.team` faithfully.
/// Raven `G_BotConnect`.
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:768-784`
pub fn G_BotConnect(
    ctx: GameContext<'_>,
    clientNum: c_int,
    restart: qboolean,
) -> qboolean {
    todo!("Port G_BotConnect — parked: missing-type: bot_settings_s")
}

// PORT-ESCALATION(missing-global): calls `G_GetBotInfoByName`, which is
// itself parked on the missing `g_botInfos` `GameGlobals` field (see
// `G_CountBotPlayers`/`G_GetBotInfoByName`); also needs the still-`todo!()`
// `va` variadic seam for several format strings.
/// Raven `G_AddBot`.
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:792-1033`
pub fn G_AddBot(
    ctx: GameContext<'_>,
    name: *const c_char,
    skill: f32,
    team: *const c_char,
    delay: c_int,
    altname: *mut c_char,
) {
    todo!("Port G_AddBot — parked: missing-global: g_botInfos")
}

// PORT-ESCALATION(missing-const): needs `MAX_TOKEN_CHARS` and an `atoi`
// helper, neither of which exist yet in this crate's ported surface.
/// Raven `Svcmd_AddBot_f`.
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:1041-1093`
pub fn Svcmd_AddBot_f(ctx: GameContext<'_>) {
    todo!("Port Svcmd_AddBot_f — parked: missing-const: MAX_TOKEN_CHARS/atoi")
}

// PORT-ESCALATION(missing-global): reads `g_botInfos`, whose `GameGlobals`
// field does not exist yet (see `G_CountBotPlayers`).
/// Raven `Svcmd_BotList_f`.
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:1100-1127`
pub fn Svcmd_BotList_f(ctx: GameContext<'_>) {
    todo!("Port Svcmd_BotList_f — parked: missing-global: g_botInfos")
}

// PORT-ESCALATION(missing-global): writes `g_botInfos` (no `GameGlobals`
// field yet, see `G_CountBotPlayers`) and calls the still-`todo!()` `va`
// variadic seam (via `trap_Printf`'s format strings).
/// Raven `G_LoadBotsFromFile`.
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:1194-1215`
pub fn G_LoadBotsFromFile(
    ctx: GameContext<'_>,
    filename: *mut c_char,
) {
    todo!("Port G_LoadBotsFromFile — parked: missing-global: g_botInfos")
}

// PORT-ESCALATION(missing-const): registers a cvar with `CVAR_INIT|CVAR_ROM`
// flag constants that don't exist yet anywhere in the ported surface (no
// cvar-flags module has landed).
/// Raven `G_LoadBots`.
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:1222-1256`
pub fn G_LoadBots(ctx: GameContext<'_>) {
    todo!("Port G_LoadBots — parked: missing-const: CVAR_INIT/CVAR_ROM")
}

// PORT-ESCALATION(missing-global): reads `g_botInfos`, whose `GameGlobals`
// field does not exist yet (see `G_CountBotPlayers`).
/// Raven `G_GetBotInfoByNumber`.
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:1265-1271`
pub fn G_GetBotInfoByNumber(
    ctx: GameContext<'_>,
    num: c_int,
) -> *mut c_char {
    todo!("Port G_GetBotInfoByNumber — parked: missing-global: g_botInfos")
}

// PORT-ESCALATION(missing-global): reads `g_botInfos`, whose `GameGlobals`
// field does not exist yet (see `G_CountBotPlayers`).
/// Raven `G_GetBotInfoByName`.
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:1279-1291`
pub fn G_GetBotInfoByName(
    ctx: GameContext<'_>,
    name: *const c_char,
) -> *mut c_char {
    todo!("Port G_GetBotInfoByName — parked: missing-global: g_botInfos")
}

// PORT-ESCALATION(missing-global): reads `bot_minplayers` (cvar handle, not
// yet a `GameCvars` field, see `G_CheckMinimumPlayers`); also calls
// `G_LoadBots`/`G_LoadArenas`, themselves parked on missing globals.
/// Raven `G_InitBots`.
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:1302-1311`
pub fn G_InitBots(
    ctx: GameContext<'_>,
    restart: qboolean,
) {
    todo!("Port G_InitBots — parked: missing-global: bot_minplayers")
}
