# B5 — Server Dossier (ground truth for design session)

Scope: MP `oracle/codemp/server/` in full (sv_main, sv_init, sv_client,
sv_snapshot, sv_world, sv_ccmds, sv_game beyond A1 §1.5, sv_bot/NPCNav
headline), SP `oracle/code/server/` as a diff (incl. sv_savegame
headline). Every claim cites `oracle/<path>:<line>`. Companion dossiers:
A2 (sv/svs census §1h, LocateGameData deep dive, Chain-A reentrancy trace),
A1 §1.5 (SV_GameSystemCalls dispatch). Slice 0 = MP dedicated boot:
`main → Com_Init(dedicated) → SV_Init → SV_Frame idle`; this doc is its core.

Status: complete.

---

## 1. SV_Init — cvars, commands, initial state

`SV_Init` — `oracle/codemp/server/sv_init.cpp:803-886`. Order: calls
`SV_AddOperatorCommands()` first (`sv_init.cpp:804`), registers cvars, then
`SV_BotInitCvars()` (`sv_init.cpp:875`) and `SV_BotInitBotLib()`
(`sv_init.cpp:877-878`).

CVAR_* flag values (`oracle/codemp/game/q_shared.h:1782-1799`):
ARCHIVE=0x1, USERINFO=0x2, SERVERINFO=0x4, SYSTEMINFO=0x8, INIT=0x10,
LATCH=0x20, ROM=0x40, TEMP=0x100, CHEAT=0x200, NORESTART=0x400.

### 1.1 Cvar registration table

All lines `oracle/codemp/server/sv_init.cpp`:

| cvar | default | flags | purpose | line |
|---|---|---|---|---|
| `dmflags` | `"0"` | SERVERINFO | deathmatch behavior flags | :807 |
| `fraglimit` | `"20"` | SERVERINFO | frag limit | :808 |
| `timelimit` | `"0"` | SERVERINFO | time limit | :809 |
| `capturelimit` | `"0"` | SERVERINFO | CTF capture limit | :810 |
| `g_maxHolocronCarry` | `"3"` | SERVERINFO | max holocrons carried | :813 |
| `g_privateDuel` | `"1"` | SERVERINFO | allow private duels | :814 |
| `g_saberLocking` | `"1"` | SERVERINFO | saber-lock toggle | :815 |
| `g_maxForceRank` | `"6"` | SERVERINFO | max force rank | :816 |
| `duel_fraglimit` | `"10"` | SERVERINFO | duel frag limit | :817 |
| `g_forceBasedTeams` | `"0"` | SERVERINFO | force-based team balance | :818 |
| `g_duelWeaponDisable` | `"1"` | SERVERINFO | weapons off in duel | :819 |
| `g_gametype` → `sv_gametype` | `"0"` | SERVERINFO\|LATCH | gametype (GT_*) | :821 |
| `g_needpass` → `sv_needpass` | `"0"` | SERVERINFO\|ROM | password required flag | :822 |
| `sv_keywords` | `""` | SERVERINFO | master keyword tags | :823 |
| `protocol` | `PROTOCOL_VERSION` | SERVERINFO\|ROM | net protocol version | :824 |
| `mapname` → `sv_mapname` | `"nomap"` | SERVERINFO\|ROM | current map | :825 |
| `sv_privateClients` | `"0"` | SERVERINFO | reserved private slots | :826 |
| `sv_hostname` | `"*Jedi*"` | SERVERINFO\|ARCHIVE | server name | :827 |
| `sv_maxclients` | `"8"` | SERVERINFO\|LATCH | max client slots | :828 |
| `sv_maxRate` | `"0"` | ARCHIVE\|SERVERINFO | per-client bandwidth cap | :829 |
| `sv_minPing` | `"0"` | ARCHIVE\|SERVERINFO | min ping to connect | :830 |
| `sv_maxPing` | `"0"` | ARCHIVE\|SERVERINFO | max ping to connect | :831 |
| `sv_floodProtect` | `"1"` | ARCHIVE\|SERVERINFO | reliable-cmd flood protect | :832 |
| `sv_allowAnonymous` | `"0"` | SERVERINFO | `#ifdef USE_CD_KEY` only | :834 |
| `sv_cheats` | `"0"` | SYSTEMINFO\|ROM | cheats flag (set by SV_Map_f) | :837 |
| `sv_serverid` | `"0"` | SYSTEMINFO\|ROM | session id | :838 |
| `sv_pure` | `"1"` (`"0"` DLL_ONLY) | SYSTEMINFO (+INIT\|ROM DLL_ONLY) | pure-pak enforcement | :840,842 |
| `sv_paks` | `""` | SYSTEMINFO\|ROM | loaded pak checksums | :844 |
| `sv_pakNames` | `""` | SYSTEMINFO\|ROM | loaded pak names | :845 |
| `sv_referencedPaks` | `""` | SYSTEMINFO\|ROM | referenced pak checksums | :846 |
| `sv_referencedPakNames` | `""` | SYSTEMINFO\|ROM | referenced pak names | :847 |
| `rconPassword` → `sv_rconPassword` | `""` | TEMP | rcon password | :850 |
| `sv_privatePassword` | `""` | TEMP | private-slot password | :851 |
| `sv_fps` | `"20"` | TEMP | server tick rate | :852 |
| `sv_timeout` | `"200"` | TEMP | client timeout (s) | :853 |
| `sv_zombietime` | `"2"` | TEMP | zombie-slot linger (s) | :854 |
| `nextmap` | `""` | TEMP | vstr'd next map | :855 |
| `sv_allowDownload` | `"0"` | SERVERINFO | UDP pak downloads (non-Xbox) | :858 |
| `sv_master1` | `MASTER_SERVER_NAME` | 0 | primary master | :859 |
| `sv_master2..5` | `""` | ARCHIVE | secondary masters | :860-863 |
| `sv_reconnectlimit` | `"3"` | 0 | reconnect throttle (s) | :865 |
| `sv_showghoultraces` | `"0"` | 0 | G2 trace debug | :866 |
| `sv_showloss` | `"0"` | 0 | packet-loss debug | :867 |
| `sv_padPackets` | `"0"` | 0 | packet padding debug | :868 |
| `sv_killserver` | `"0"` | 0 | polled shutdown flag | :869 |
| `sv_mapChecksum` | `""` | ROM | loaded BSP checksum | :870 |

No `Cvar_CheckRange`/`Cvar_SetDescription` anywhere in sv_init.cpp (grepped;
API postdates this codebase's use here).

### 1.2 Command registration

`SV_AddOperatorCommands` — `oracle/codemp/server/sv_ccmds.cpp:958-996`,
idempotent via `static qboolean initialized` (`sv_ccmds.cpp:958-964`).
`SV_RemoveOperatorCommands` is entirely `#if 0`'d — "removing these won't let
the server start again" (`sv_ccmds.cpp:1003-1018`).

| command | handler | line |
|---|---|---|
| `heartbeat` | `SV_Heartbeat_f` | sv_ccmds.cpp:966 |
| `kick` | `SV_Kick_f` | :967 |
| `banUser` / `banClient` | `SV_Ban_f` / `SV_BanNum_f` (`#ifdef USE_CD_KEY`) | :969-970 |
| `clientkick` | `SV_KickNum_f` | :973 |
| `status` | `SV_Status_f` | :974 |
| `serverinfo` | `SV_Serverinfo_f` | :975 |
| `systeminfo` | `SV_Systeminfo_f` | :976 |
| `dumpuser` | `SV_DumpUser_f` | :977 |
| `map_restart` | `SV_MapRestart_f` | :978 |
| `sectorlist` | `SV_SectorList_f` | :979 |
| `map` | `SV_Map_f` | :980 |
| `devmap`/`spmap`/`spdevmap`/`devmapmdl`/`devmapall` | `SV_Map_f` (`#ifndef PRE_RELEASE_DEMO`) | :982-987 |
| `killserver` | `SV_KillServer_f` | :989 |
| `svsay` | `SV_ConSay_f` | :992 |
| `forcetoggle` | `SV_ForceToggle_f` | :995 |

### 1.3 Initial state beyond cvars/commands

- `SV_BotInitCvars()` / `SV_BotInitBotLib()` — `sv_init.cpp:875,877-878`
  (botlib init "here because we need the pre-compiler in the UI").
- `G2VertSpaceServer = &CMiniHeap_singleton` — the one-time Ghoul2 server
  vertex-transform miniheap, `sv_init.cpp:883-885`.
- No challenge-table or `svs` init happens in `SV_Init` — `svs.initialized`
  etc. happen at first spawn via `SV_Startup` (`sv_init.cpp:258-278`, §2 step 12).

---

## 2. SV_SpawnServer end-to-end — THE slice-1 script

`void SV_SpawnServer(char *server, qboolean killBots, ForceReload_e
eForceReload)` — `oracle/codemp/server/sv_init.cpp:472-791`.

`sv.state` machine first: `serverState_t` = `SS_DEAD` (no map), `SS_LOADING`
(spawning level entities), `SS_GAME` (running) — `server.h:47-51`, field
`server.h:54`. Transitions: SS_DEAD implicit via `memset(&sv,0,...)` in
`SV_InitSV` (`sv_init.cpp:284-289`, called from `SV_ClearServer`
`sv_init.cpp:365-372` at step 21 — the *only* way state returns to SS_DEAD);
→ SS_LOADING at `sv_init.cpp:659`; → SS_GAME at `sv_init.cpp:777`. Outside
spawn, `map_restart` toggles SS_LOADING (`sv_ccmds.cpp:293`) → SS_GAME
(`sv_ccmds.cpp:304`) without a full clear.

Numbered sequence:

1. `SV_SendMapChange()` #1 — `sv_init.cpp:479`. For each connected non-bot
   client (`state >= CS_CONNECTED`, `type != NA_BOT`, `sv_init.cpp:414-430`)
   sends a one-byte `svc_mapchange` message via `SV_SendClientMapChange`
   (`sv_client.cpp:820-842`). Repeated 5× through the function to keep client
   netchans alive during the long load.
2. `RE_RegisterMedia_LevelLoadBegin(server, eForceReload)` — `sv_init.cpp:481`.
3. `SV_ShutdownGameProgs()` — `sv_init.cpp:484`: if a gvm exists,
   `VM_Call(gvm, GAME_SHUTDOWN, qfalse)`, `VM_Free`, `gvm = NULL`
   (`sv_game.cpp:1666-1673`).
4. Banner `Com_Printf("------ Server Initialization ------\n" / "Server:
   %s\n")` — `sv_init.cpp:486-487`.
5. `delete[] svs.snapshotEntities; svs.snapshotEntities = NULL` —
   `sv_init.cpp:493-497`.
6. `SV_SendMapChange()` #2 — `sv_init.cpp:502`.
7. `CL_MapLoading()` — `sv_init.cpp:511` (listen-server client hook).
8. `CL_ShutdownAll()` — `sv_init.cpp:515` (`#ifndef DEDICATED`).
9. `CM_ClearMap()` — `sv_init.cpp:518`.
10. **`Hunk_Clear()`** — `sv_init.cpp:527` — whole hunk dropped on every
    (re)load; no `Hunk_ClearToMark` here (the mark is *set* at step 46).
11. `R_InitSkins(); R_InitShaders(qtrue)` — `sv_init.cpp:533-534`.
12. Client array: if `sv_running` unset → `SV_Startup()`
    (`sv_init.cpp:258-278`: `svs.clients = Z_Malloc(sizeof(client_t) *
    sv_maxclients)`, `svs.numSnapshotEntities = sv_maxclients * PACKET_BACKUP
    * 64` dedicated / `sv_maxclients * 4 * 64` listen (`sv_init.cpp:264-274`),
    `svs.initialized = qtrue`, `Cvar_Set("sv_running","1")`); else if
    `sv_maxclients->modified` → `SV_ChangeMaxClients()`
    (`sv_init.cpp:299-358`, reallocs preserving connected clients).
    Call site `sv_init.cpp:547-554`.
13. `SV_SendMapChange()` #3 — `sv_init.cpp:556`.
14. Dedicated: `R_SVModelInit()` — `sv_init.cpp:576-579`.
15. `SV_SendMapChange()` #4 — `sv_init.cpp:581`.
16. `FS_ClearPakReferences(0)` — `sv_init.cpp:584`.
17. `svs.nextSnapshotEntities = 0` — `sv_init.cpp:591`.
18. `svs.snapshotEntities = new entityState_s[svs.numSnapshotEntities]` +
    memset — `sv_init.cpp:594-596`.
19. `svs.snapFlagServerBit ^= SNAPFLAG_SERVERCOUNT` — `sv_init.cpp:604`
    (clients detect server restart in snapshot flags; `server.h:213`).
20. `Cvar_Set("nextmap", "map_restart 0")` — `sv_init.cpp:608`.
21. `SV_ClearServer()` — `sv_init.cpp:612`: Z_Frees all `sv.configstrings[i]`,
    then `SV_InitSV()` memsets `sv` to zero (→ SS_DEAD) and sets
    `sv.mLocalSubBSPIndex = -1` (`sv_init.cpp:284-289,365-372`).
22. Configstrings reset: every slot `sv.configstrings[i] = CopyString("")` —
    `sv_init.cpp:613-615`. (CS_MODELS/CS_PLAYERS etc. are populated later by
    the game module during GAME_INIT via `G_SET_CONFIGSTRING`, not here.)
23. `G2API_SetTime(svs.time, 0)` — `sv_init.cpp:618`.
24. `Cvar_Set("cl_paused", "0")` — `sv_init.cpp:622`.
25. **checksumFeed**: `srand(Com_Milliseconds()); sv.checksumFeed =
    (((int)rand() << 16) ^ rand()) ^ Com_Milliseconds();` then
    `FS_Restart(sv.checksumFeed)` — `sv_init.cpp:625-627`. Consumers:
    gamestate send (`MSG_WriteLong(&msg, sv.checksumFeed)`,
    `sv_client.cpp:765`), pure-pak validation XOR (`SV_VerifyPaks_f`,
    `sv_client.cpp:1283,1407`), usercmd anti-tamper key (`SV_UserMove`,
    `sv_client.cpp:1700`).
26. `CM_LoadMap(va("maps/%s.bsp", server), qfalse, &checksum)` —
    `sv_init.cpp:638` (out-param checksum; `qfalse` = not clientload).
27. `SV_SendMapChange()` #5 — `sv_init.cpp:641`.
28. `Cvar_Set("mapname", server)` — `sv_init.cpp:644`.
29. `Cvar_Set("sv_mapChecksum", va("%i",checksum))` — `sv_init.cpp:646`.
30. `sv.serverId = com_frameTime; sv.restartedServerId = sv.serverId;
    Cvar_Set("sv_serverid", ...)` — `sv_init.cpp:649-651`.
31. `SV_ClearWorld()` — `sv_init.cpp:654` (resets `sv_worldSectors`,
    `sv_world.cpp:129-146`).
32. **`sv.state = SS_LOADING`** — `sv_init.cpp:659`.
33. **GVM creation + GAME_INIT**: `SV_InitGameProgs()` — `sv_init.cpp:662` →
    `gvm = VM_Create("jampgame", SV_GameSystemCalls,
    (vmInterpret_t)Cvar_VariableValue("vm_game"))` (`sv_game.cpp:1750-1753`);
    `SV_InitGameVM(qfalse)`: `sv.entityParsePoint = CM_EntityString()`, then
    `VM_Call(gvm, GAME_INIT, svs.time, Com_Milliseconds(), qfalse)`
    (`sv_game.cpp:1682-1690`); clears every `svs.clients[i].gentity = NULL`
    (`sv_game.cpp:1694-1696`).
34. `sv_gametype->modified = qfalse` — `sv_init.cpp:665`.
35. **Settle loop ×3**: `G2API_SetTime`; `VM_Call(gvm, GAME_RUN_FRAME,
    svs.time)`; `SV_BotFrame(svs.time)`; `svs.time += 100` —
    `sv_init.cpp:668-675`; final `G2API_SetTime` `sv_init.cpp:676-678`.
36. `SV_CreateBaseline()` — `sv_init.cpp:681` — copies each linked entity's
    `svent->s` into `sv.svEntities[entnum].baseline`, loop from entnum 1
    (`sv_init.cpp:209-225`).
37. **Client carry-over loop** — `sv_init.cpp:683-728`, for each
    `svs.clients[i].state >= CS_CONNECTED`:
    - bot (`netchan.remoteAddress.type == NA_BOT`): if `killBots` →
      `SV_DropClient(cl, "")`, continue (`sv_init.cpp:688-690`);
    - `denied = VM_ExplicitArgPtr(gvm, VM_Call(gvm, GAME_CLIENT_CONNECT, i,
      qfalse, isBot))` — reconnect, `firstTime = qfalse` (`sv_init.cpp:700`);
      denied → `SV_DropClient` (`sv_init.cpp:704`);
    - human: `state = CS_CONNECTED` (`sv_init.cpp:709`) — clients are *kept*,
      silently demoted; "when we get the next packet from a connected client,
      the new gamestate will be sent" (the stale-serverId path in
      `SV_ExecuteClientMessage`, §4.7, triggers the resend);
    - bot: `state = CS_ACTIVE`, rewire `gentity`, `deltaMessage = -1`,
      `nextSnapshotTime = svs.time`, `VM_Call(gvm, GAME_CLIENT_BEGIN, i)`
      (`sv_init.cpp:712-725`).
38. One more settle frame (`GAME_RUN_FRAME` + `SV_BotFrame` + `svs.time +=
    100`) — `sv_init.cpp:731-736`.
39. Pure paks: if `sv_pure` → `Cvar_Set("sv_paks", FS_LoadedPakChecksums())`,
    `sv_pakNames`, warn if empty, dedicated → `SV_TouchCGame()`; else clear —
    `sv_init.cpp:738-758`.
40. `sv_referencedPaks`/`sv_referencedPakNames` from
    `FS_ReferencedPakChecksums/Names()` — `sv_init.cpp:761-764`.
41. `SV_SetConfigstring(CS_SYSTEMINFO, Cvar_InfoString_Big(CVAR_SYSTEMINFO))`,
    modified-flag cleared first — `sv_init.cpp:766-769`.
42. `SV_SetConfigstring(CS_SERVERINFO, Cvar_InfoString(CVAR_SERVERINFO))` +
    flag clear — `sv_init.cpp:771-772`.
43. **`sv.state = SS_GAME`** — `sv_init.cpp:777` — from here configstring
    changes reliably broadcast (§8.1).
44. `SV_Heartbeat_f()` — `sv_init.cpp:780` (force next-frame master heartbeat,
    `sv_ccmds.cpp:876`).
45. `Hunk_SetMark()` — `sv_init.cpp:782` — hunk high-water mark *after* level
    load (comment `sv_init.cpp:784-790`).

There is no end-of-spawn "reconnect" reliable broadcast — carried clients get
the new gamestate lazily via the serverId-mismatch path (§4.7), and the 5
`svc_mapchange` sends during spawn keep them from timing out.

### 2.1 SV_Map_f — what happens *before* SV_SpawnServer

`SV_Map_f` — `oracle/codemp/server/sv_ccmds.cpp:138-223`:
1. `map = Cmd_Argv(1)`, bail if absent (`sv_ccmds.cpp:145-148`); reject `\` in
   name (`:152-155`).
2. Pre-flight `FS_ReadFile("maps/<map>.bsp")` existence check — bail with
   "Can't find map" (`:157-163`).
3. `Cvar_Get("g_gametype", ...)` to establish latched value (`:166`).
4. Variant dispatch on `Cmd_Argv(0)`: `sp*` prefix → `g_gametype =
   GT_SINGLE_PLAYER`, `g_doWarmup 0`, `Cvar_SetLatched("sv_maxclients","8")`,
   strip prefix, `cheat=qfalse, killBots=qtrue` (`:169-177`); `devmap*` or
   `spdevmap` → `cheat=qtrue, killBots=qtrue`; plain `map` →
   `cheat=qfalse, killBots=qfalse` (`:179-185`).
5. Defensive copy of the map name — "on a map restart we reload the
   jampconfig.cfg and thus nuke the arguments of the map command"
   (`:193-195`).
6. `eForceReload`: `devmapmdl` → MODELS, `devmapall` → ALL, else NOTHING
   (`:197-209`; `devmapbsp` commented out, "not relevant in MP").
7. `SV_SpawnServer(mapname, killBots, eForceReload)` (`:212`).
8. Post-spawn `Cvar_Set("sv_cheats", cheat ? "1" : "0")` (`:214-222`).

No killserver/FS_Restart/etc. in SV_Map_f itself — all lifecycle work is
inside SV_SpawnServer (FS_Restart at `sv_init.cpp:627`, game shutdown at
`:484`).

### 2.2 SV_MapRestart_f (light restart)

`sv_ccmds.cpp:234-343`: double-restart-same-frame guard via `com_frameTime ==
sv.serverId` (`:242-244`); delayed restart via `sv.restartTime` + CS_WARMUP
configstring, consumed by `SV_Frame` (`sv_ccmds.cpp:252-266`,
`sv_main.cpp:881-885`); if `sv_maxclients` or `sv_gametype` modified → full
`SV_SpawnServer` instead (`:270-279`); else flips `snapFlagServerBit`, new
`sv.serverId`, `sv.state = SS_LOADING` + **`sv.restarting = qtrue`** so
configstring changes still broadcast (`:290-293`; `server.h:55`),
`SV_RestartGameProgs()`, 3 settle frames, then per-client in-place
reconnect: forces `client->state = CS_ACTIVE` and calls
`SV_ClientEnterWorld(client, &client->lastUsercmd)` directly (`:296-338`,
force-active at `:335`, enter-world at `:337`).

---

## 3. SV_Frame anatomy

`void SV_Frame(int msec)` — `oracle/codemp/server/sv_main.cpp:826-937`.
Called unconditionally each `Com_Frame` iteration
(`oracle/codemp/qcommon/common.cpp:1669`; dedicated `minMsec = 1`,
`common.cpp:1645` — outer loop never paces the sim, `NET_Sleep` does).

1. Early-outs: `sv_killserver` set → `SV_Shutdown("Server was killed.\n")` +
   `Cvar_Set("sv_killserver","0")`, return (`sv_main.cpp:831-835`);
   `!com_sv_running->integer` → return (`:837-839`); `SV_CheckPaused()` →
   return (`:842-844`).
2. `sv_fps` clamp (`< 1` → reset to `"10"`); `frameMsec = 1000 /
   sv_fps->integer` (`:847-850`). Default `sv_fps` = 20 → 50 ms steps.
3. **`sv.timeResidual += msec`** (`:852`; field `server.h:64`).
4. Listen-server bot think `SV_BotFrame(svs.time + sv.timeResidual)` (`:854`).
5. **Dedicated sleep gate**: `if (com_dedicated->integer && sv.timeResidual <
   frameMsec && (!com_timescale || com_timescale->value >= 1)) {
   NET_Sleep(frameMsec - sv.timeResidual); return; }` (`:856-861`).
   `NET_Sleep` = `select()` on UDP socket + stdin with that timeout
   (`oracle/codemp/unix/unix_net.c:582-598`; win32
   `win32/win_net.cpp:1211`) — wakes early on packet/console input. This is
   the entire idle-loop story for slice 0.
6. Wrap guards: `svs.time > 0x70000000` → `SV_Shutdown("Restarting server due
   to time wrapping")` + `Cbuf_AddText("map_restart 0\n")` (`:867-872`);
   `svs.nextSnapshotEntities` near overflow → same (`:874-879`); pending
   `sv.restartTime` reached → `map_restart 0` (`:881-885`).
7. Modified-cvar configstring refresh: `cvar_modifiedFlags & CVAR_SERVERINFO`
   → re-push CS_SERVERINFO; `& CVAR_SYSTEMINFO` → CS_SYSTEMINFO (`:887-895`).
8. `SV_CalcPings()` (`:904`); dedicated `SV_BotFrame(svs.time)` (`:906`).
9. **Fixed-step sim loop**: `while (sv.timeResidual >= frameMsec) {
   sv.timeResidual -= frameMsec; svs.time += frameMsec; VM_Call(gvm,
   GAME_RUN_FRAME, svs.time); }` (`:909-915`) — the sole GAME_RUN_FRAME
   dispatch in SV_Frame; `svs.time` advances only here, in exact frameMsec
   quanta. (Matches the two-island reborrow-threading anchor,
   `docs/architecture/two-island-model.md`.)
10. `G2API_SetTime(svs.time, 0)` (`:918`).
11. **`SV_CheckTimeouts()`** (`:926`; body `:719-751`): `droppoint = svs.time -
    1000*sv_timeout`, `zombiepoint = svs.time - 1000*sv_zombietime`; CS_ZOMBIE
    older than zombiepoint → `state = CS_FREE` (`:736-738`); connected clients
    with `lastPacketTime < droppoint` accumulate `timeoutCount`, at 5
    consecutive → `SV_DropClient(cl, "timed out")` + CS_FREE (`:740-746`);
    else reset count (`:748`).
12. **`SV_SendClientMessages()`** (`:929`) — per-client snapshot send, §5.6.
13. `SV_CheckCvars()` (`:931`).
14. **`SV_MasterHeartbeat()`** (`:935`, `#ifndef _XBOX`; body `:222-279`):
    only `com_dedicated->integer == 2` (`:228-230`); interval `HEARTBEAT_MSEC
    = 300*1000` vs `svs.nextHeartbeatTime` (`:220,233-236`); resolves each
    `sv_master[i]` and sends OOB `"heartbeat %s\n"` with `HEARTBEAT_GAME =
    "QuakeArena-1"` (`:221,277`).

`sv_fps` vs `com_maxfps`: `com_maxfps` bounds only the outer non-dedicated
`Com_Frame` poll granularity (`common.cpp:1642-1653`); `sv_fps` alone controls
sim stepping via timeResidual.

---

## 4. Client lifecycle state machine

### 4.1 clientState_t

`oracle/codemp/server/server.h:114-121`:

| Value | Meaning |
|---|---|
| `CS_FREE` (0) | slot reusable | 
| `CS_ZOMBIE` (1) | disconnected; slot held so the `disconnect` reliable cmd can flush; aged to CS_FREE by `SV_CheckTimeouts` after `sv_zombietime` (`sv_main.cpp:736-738`) |
| `CS_CONNECTED` (2) | assigned a client_t, no gamestate yet |
| `CS_PRIMED` (3) | gamestate sent, no usercmd received yet |
| `CS_ACTIVE` (4) | fully in game |

Ordinal ordering is load-bearing (`state >= CS_CONNECTED`, `< CS_PRIMED`
comparisons throughout).

### 4.2 client_t lifecycle fields

`server.h:124-182` (full struct; lifecycle-germane subset): `state` (:125),
`userinfo[MAX_INFO_STRING]` (:126), `sentGamedir` (:128, svc_setgame flag),
`reliableCommands[MAX_RELIABLE_COMMANDS][MAX_STRING_CHARS]` +
`reliableSequence`/`reliableAcknowledge`/`reliableSent`/`messageAcknowledge`
(:130-134), `gamestateMessageNum` (:136), `challenge` (:137), `lastUsercmd`
(:139), `lastClientCommand`(+String) (:141-142), `gentity` (:143), `name`
(:144), `deltaMessage` (:161), `nextReliableTime` (:162), `lastPacketTime` /
`lastConnectTime` (:163-164), `nextSnapshotTime` / `rateDelayed` (:165-166),
`timeoutCount` (:167), `frames[PACKET_BACKUP]` (:168), `ping`/`rate`/
`snapshotMsec` (:169-171), `pureAuthentic` (:172), `netchan` (:173),
userinfo-flood throttles (:175-176).

### 4.3 Challenge handshake

- OOB dispatch: `SV_ConnectionlessPacket` (`sv_main.cpp:545-584`) —
  `getchallenge` → `SV_GetChallenge` (`sv_main.cpp:566-567`), `connect` →
  `SV_DirectConnect` (`:568-569`).
- `SV_GetChallenge` (`sv_client.cpp:31-130`): finds existing unconnected
  `svs.challenges[]` entry by address (`:52-61`) or evicts the oldest slot and
  fills `{challenge = rand()-based, adr, firstTime/time = svs.time, connected
  = qfalse}` (`:63-73`). LAN → immediate `challengeResponse %i` OOB reply
  (`:76-80`); CD-key builds relay through an authorize server (`:82-125`,
  `#ifdef USE_CD_KEY`); otherwise immediate reply (`:126-129`).
- Storage: `challenge_t` (`server.h:194-201`), `svs.challenges[MAX_CHALLENGES
  = 1024]` (`server.h:190,220`).

### 4.4 SV_DirectConnect

`sv_client.cpp:221-568`:
1. Userinfo copy, `protocol` check → reject `print\nServer uses protocol
   version %i.\n` (`:239-246`); reads `challenge`+`qport` keys (`:248-249`).
2. Reconnect-too-soon: scan all clients by address+qport; `svs.time -
   lastConnectTime < sv_reconnectlimit*1000` → reject (`:252-273`, the
   CS_FREE skip is deliberately commented out `:254-261`).
3. Challenge validation (non-local only): match address+challenge in
   `svs.challenges[]` (`:280-290`); force real `ip` into userinfo (`:292`);
   `ping = svs.time - pingTime`, `connected = qtrue` (`:296`);
   `sv_minPing`/`sv_maxPing` enforcement (`:299-314`; min-ping reject zeroes
   `challenge->adr.port` to defeat ping cheat `:306`). Local → `ip =
   "localhost"` (`:315-318`).
4. Reconnect detection: second scan for `state != CS_FREE` match by
   address+qport → `Com_Printf("%s:reconnect\n")`, `VM_Call(gvm,
   GAME_CLIENT_DISCONNECT, ...)` on the old slot, `goto gotnewcl` (`:324-341`).
5. Slot allocation: `startIndex = 0` if `password` == `sv_privatePassword`
   else `sv_privateClients` (`:354-364`); first CS_FREE slot (`:366-373`);
   full + local: if all remaining are bots, force-drop the last slot
   (`SV_DropClient(..., "only bots on server")`, `:376-388`), else
   `Com_Error(ERR_FATAL, "server is full on local connect")` (`:390`); full +
   remote → OOB `print\n<SERVER_IS_FULL>\n` (`:394-398`).
6. `gotnewcl:` — `*newcl = temp` (zeroed; "the only place a client_t is ever
   initialized", `:505`), `gentity = SV_GentityNum(clientNum)` (`:507-508`),
   `challenge` stored (`:511`), `Netchan_Setup(NS_SERVER, &newcl->netchan,
   from, qport)` (`:514`), userinfo copied (`:517`).
7. **GAME_CLIENT_CONNECT**: `denied = (char*)VM_Call(gvm,
   GAME_CLIENT_CONNECT, clientNum, qtrue, qfalse)`; non-NULL →
   `VM_ExplicitArgPtr` resolve, OOB `print\n%s\n`, abort (state never set;
   slot stays free) (`:520-528`).
8. `SV_UserinfoChanged(newcl)` (`:530`); OOB `connectResponse` (`:540`).
9. **CS_FREE → CS_CONNECTED** (`:544`), `nextSnapshotTime`/`lastPacketTime`/
   `lastConnectTime = svs.time` (`:545-547`), `gamestateMessageNum = -1`
   (`:552`).
10. Heartbeat if first client or server just filled (`:559-567`).

### 4.5 Gamestate send — CS_CONNECTED → CS_PRIMED

`SV_SendClientGameState` (`sv_client.cpp:697-817`): flush pending fragments
(`:705-712`); **`state = CS_PRIMED`** (`:716`), `pureAuthentic = 0` (`:717`),
`gamestateMessageNum = netchan.outgoingSequence` (`:722`); writes
`lastClientCommand` ack (`:728`), pending reliables
(`SV_UpdateServerCommandsToClient`, `:734`), `svc_gamestate` +
`reliableSequence` (`:737-738`), all non-empty configstrings (`:741-747`),
all baselines with nonzero `number` delta'd from null (`:749-758`),
`svc_EOF`, clientNum, `sv.checksumFeed` (`:760-765`), optional RMG data
(`:767-813`); `SV_SendMessageToClient` (`:816`).

### 4.6 Begin — CS_PRIMED → CS_ACTIVE

**There is no `begin` client command in JAMP** (the `ucmds[]` table
`sv_client.cpp:1542-1555` has only `userinfo`, `disconnect`, `cp`, `vdr`,
`download`, `nextdl`, `stopdl`, `donedl`). The transition is usercmd-driven:
`SV_UserMove` (`sv_client.cpp:1674-1755`) — first usercmd while
`state == CS_PRIMED` triggers `SV_ClientEnterWorld(cl, &cmds[0])` (`:1719`).
`SV_ClientEnterWorld` (`:943-970`): **`state = CS_ACTIVE`** (`:948`), rewires
`gentity`/`s.number` (`:956-959`), `deltaMessage = -1`, `nextSnapshotTime =
svs.time` (`:964-965`), `lastUsercmd = *cmd` (`:966`), then `VM_Call(gvm,
GAME_CLIENT_BEGIN, ...)` (`:969`). Post-check in SV_UserMove: `state !=
CS_ACTIVE` → `deltaMessage = -1`, return (`:1731-1734`); pure validation drop
(`sv_pure && pureAuthentic == 0` → `SV_DropClient("Cannot validate pure
client!")`, `:1724-1729`).

### 4.7 Per-state packet handling — SV_ExecuteClientMessage

`sv_client.cpp:1773-1854`:
- reads `serverId`, `messageAcknowledge` (reject `< 0`, `:1779-1787`),
  `reliableAcknowledge` (stale clamp guard, `:1789-1800`).
- **Stale serverId** (`:1808-1824`): not mid-download; `serverId ==
  sv.restartedServerId` → pre-map_restart leftover, return (`:1813-1816`);
  else if `messageAcknowledge > gamestateMessageNum` (client missed the
  gamestate) → `SV_SendClientGameState(cl)` (`:1819-1822`); return without
  usercmd processing either way. This is how carried-over clients (§2 step
  37) and fresh CS_CONNECTED clients get their gamestate.
- Reliable client-command loop until `clc_EOF`: `SV_ClientCommand` per
  `clc_clientCommand`; `qfalse` return (flood stall) → stop; `state ==
  CS_ZOMBIE` after command (client sent `disconnect`) → stop (`:1827-1841`).
- `clc_move`/`clc_moveNoDelta` → `SV_UserMove(cl, msg, qtrue/qfalse)`
  (`:1843-1850`).
- `SV_ClientCommand` (`:1590-1639`): duplicate seq ignored (`:1599-1601`);
  lost command (`seq > lastClientCommand + 1`) → `SV_DropClient("Lost
  reliable commands")` (`:1606-1611`); flood protect applies only when
  `state >= CS_ACTIVE` (`:1620-1628`, downloads must spam).

### 4.8 SV_DropClient teardown

`sv_client.cpp:580-666`:
1. `state == CS_ZOMBIE` → return (idempotent, `:584-586`).
2. Non-bot: mark matching `svs.challenges[]` entry `connected = qfalse`
   (`:588-598`).
3. `SV_CloseDownload(drop)` (`:622`).
4. Broadcast reason: `SV_SendServerCommand(NULL, "print \"%s %s\n\"", name,
   reason)` (`:626`).
5. **`state = CS_ZOMBIE`** ("become free in a few seconds", `:628-629`) — not
   CS_FREE, so step 7's queued reliable can still transmit; aging to CS_FREE
   happens in `SV_CheckTimeouts` (§3.11).
6. `VM_Call(gvm, GAME_CLIENT_DISCONNECT, ...)` (`:640`).
7. Queue `disconnect "reason"` reliable to the dropped client (`:643`).
8. Bot → `SV_BotFreeClient` (`:645-647`).
9. `SV_SetUserinfo(clientNum, "")` (`:650`).
10. If no clients remain `>= CS_CONNECTED` → `SV_Heartbeat_f()` (`:656-665`).

### 4.9 Reliable-command delivery

- Ring: `reliableCommands[MAX_RELIABLE_COMMANDS = 128][MAX_STRING_CHARS]`
  (`server.h:130`; constant `oracle/codemp/qcommon/qcommon.h:106`).
- Enqueue `SV_AddServerCommand` (`sv_main.cpp:116-141`): `++reliableSequence`,
  index `seq & (MAX_RELIABLE_COMMANDS-1)` (`:125,139-140`). Overflow check
  `reliableSequence - reliableAcknowledge == MAX_RELIABLE_COMMANDS + 1` →
  dump backlog + `SV_DropClient(client, "Server command overflow")`
  (`:130-136`; `==` not `>=` so the drop's own broadcast print can't recurse,
  comment `:128-129`).
- `SV_SendServerCommand` (`sv_main.cpp:153-180`): `cl != NULL` → single
  target; `NULL` → broadcast to all `state >= CS_PRIMED` (`:174-178`).
- Transmit `SV_UpdateServerCommandsToClient` (`sv_snapshot.cpp:225-235`):
  (re)writes `(reliableAcknowledge+1 ..= reliableSequence]` as
  `svc_serverCommand` each snapshot/gamestate; sets `reliableSent`.
- Ack: client's `reliableAcknowledge` read in `SV_ExecuteClientMessage`
  (`sv_client.cpp:1789`); no explicit free — fixed ring overwritten mod 128,
  overflow-drop guarantees unacked entries never wrap.

---

## 5. Snapshot pipeline (server half)

### 5.1 SV_AddEntitiesVisibleFromPoint

`static void SV_AddEntitiesVisibleFromPoint(vec3_t origin, clientSnapshot_t
*frame, snapshotEntityNumbers_t *eNums, qboolean portal)` —
`oracle/codemp/server/sv_snapshot.cpp:301-498`. Called from
`SV_BuildClientSnapshot` (`:582`) and recursively for portals (`:490`).
**Linear scan of `sv.num_entities`, not a worldSector walk** — the area tree
is collision-only (§6); snapshot visibility is pure PVS/area data cached on
`svEntity_t` at link time.

Flow: shutdown guard `!sv.state` (`:323-325`); `CM_PointLeafnum` →
`CM_LeafArea`/`CM_LeafCluster` (`:327-329`); `frame->areabytes =
CM_WriteAreaBits(frame->areabits, clientarea)` (`:332`); `clientpvs =
CM_ClusterPVS` (`:334`). Per entity `e < sv.num_entities` (`:338-497`):
- skip `!ent->r.linked` (`:342-344`); skip `EF_PERMANENT` (`:346-349`);
  self-heal `ent->s.number != e` with `"FIXING ENT->S.NUMBER!!!"` DPrintf
  (`:351-354`); skip `SVF_NOCLIENT` (`:357-359`);
  `SVF_SINGLECLIENT`/`SVF_NOTSINGLECLIENT` per-client filters (`:361-372`).
- Double-add guard: `svEnt->snapshotCounter == sv.snapshotCounter` → skip
  (`:374-379`).
- **Always-send**: `SVF_BROADCAST`, own entity (`e == frame->ps.clientNum`),
  or `broadcastClients[clientNum/32]` bit → add + continue (`:381-386`).
  **Portal entities** (`ent->s.isPortalEnt`) always added (`:388-392`). RMG
  distance-cull path replaces PVS when `com_RMG` (`:394-408`).
- Area check: `CM_AreasConnected(clientarea, svEnt->areanum)`, retry
  `areanum2` ("doors can legally straddle two areas"), fail → continue
  (`:413-419`).
- Cluster check: walk `svEnt->clusternums[0..numClusters)` against
  `clientpvs` bits; overflow fallback scans up to `svEnt->lastCluster`
  (`:421-460`).
- Optional `g_svCullDist` diameter cull (`:300,462-476`).
- `SV_AddEntToSnapshot(svEnt, ent, eNums)` (`:479`); `SVF_PORTAL` → recurse
  from `ent->s.origin2` with `generic1` max-range check (`:482-495`).

`SV_AddEntToSnapshot` (`:279-293`): de-dupe stamp `snapshotCounter`, append
to `eNums->snapshotEntities`, cap `MAX_SNAPSHOT_ENTITIES = 1024` (`:245,
287-289`, silent discard). `SV_BuildClientSnapshot` then qsorts the merged
list (portal recursion breaks ordering) — duplicates are `Com_Error(ERR_DROP,
"SV_QsortEntityStates: duplicated entity")` (`:256-271,588-589`; increasing
order required for delta compression, comment `:584-587`). The client's own
entity is pre-stamped so it is never double-added (`:573`).

### 5.2 svEntity_t (snapshot read side)

`server.h:27-45` (non-Xbox widths): `worldSector`/`nextEntityInWorldSector`
(collision-tree linkage, §6, write side `sv_world.cpp:342-344`); `baseline`
(`entityState_t`, filled by `SV_CreateBaseline` `sv_init.cpp:209-225`, read as
delta base for newly-visible entities `sv_snapshot.cpp:80`); `numClusters` /
`clusternums[MAX_ENT_CLUSTERS = 16]` / `lastCluster` (PVS cache written by
`SV_LinkEntity` `sv_world.cpp:309-323`, read `sv_snapshot.cpp:424-456`);
`areanum`/`areanum2` (written `sv_world.cpp:295-305`, read
`sv_snapshot.cpp:413-418`); `snapshotCounter` (vs `sv.snapshotCounter`,
bumped once per `SV_BuildClientSnapshot` `sv_snapshot.cpp:525`, field
`server.h:62`). Mapping helpers: `SV_SvEntityForGentity` /
`SV_GEntityForSvEntity` — index arithmetic on `sv.svEntities[]`
(`sv_game.cpp:70-82`; array `server.h:68`).

### 5.3 snapshotEntities ring

`svs.snapshotEntities` / `numSnapshotEntities` / `nextSnapshotEntities` —
`server.h:216-218`. Size: dedicated `sv_maxclients * PACKET_BACKUP * 64`,
listen `sv_maxclients * 4 * 64` (`sv_init.cpp:264-274,351-357`;
`PACKET_BACKUP = 32`, `qcommon.h:98`; the `64` is a literal — the
`MAX_PACKET_ENTITIES` name in the comment is not a real `#define`).
Allocated per spawn (`sv_init.cpp:591-596`). Writes in
`SV_BuildClientSnapshot`: `frame->first_entity = svs.nextSnapshotEntities`
then `state = &svs.snapshotEntities[svs.nextSnapshotEntities %
svs.numSnapshotEntities]; *state = ent->s; svs.nextSnapshotEntities++`
(`sv_snapshot.cpp:597-609`). Guard `nextSnapshotEntities >= 0x7FFFFFFE` →
ERR_FATAL (`:606-608`); SV_Frame proactively map_restarts long before
(`sv_main.cpp:874-879`). Readers index `(first_entity + i) %
numSnapshotEntities` (`sv_snapshot.cpp:57,64`).

### 5.4 Per-client delta bookkeeping

`clientSnapshot_t` (`server.h:94-112`): `areabytes`/`areabits`, `ps` + `vps`
(vehicle playerstate, `server.h:98`), `num_entities`/`first_entity`
(ring window; "MUST be in increasing state number order"),
`messageSent`/`messageAcked`/`messageSize`. Stored as
`client->frames[PACKET_BACKUP]` indexed by `netchan.outgoingSequence &
PACKET_MASK` (`server.h:168`; `sv_snapshot.cpp:110,528`).

Delta-source selection `SV_WriteSnapshotToClient` (`sv_snapshot.cpp:103-134`):
`deltaMessage <= 0 || state != CS_ACTIVE` → full snapshot (`:113-116`);
`outgoingSequence - deltaMessage >= PACKET_BACKUP - 3` → "Delta request from
out of date packet", full (`:117-122`); else `oldframe =
&frames[deltaMessage & PACKET_MASK]` (`:124-126`); ring-staleness check
`oldframe->first_entity <= nextSnapshotEntities - numSnapshotEntities` →
full (`:128-133`). `lastframe` byte tells the client the delta distance, 0 =
full (`:147`). Playerstate delta `MSG_WriteDeltaPlayerstate` with a
vehicle-vps null-base special case (`:164-204`, "if last frame didn't have
vehicle, then the old vps isn't gonna delta properly" `:172-179`).
`SV_EmitPacketEntities` (`:36-94`): parallel walk old/new frame lists —
match → delta (`:72`), new-only → delta from `sv.svEntities[n].baseline`
(`:80`), old-only → remove via delta-to-NULL (`:87`); terminator
`(MAX_GENTITIES-1)` in `GENTITYNUM_BITS` bits (`:93`).

### 5.5 Rate/snaps throttling

Userinfo intake `SV_UserinfoChanged` (`sv_client.cpp:1452-1500`): `rate` —
LAN & not dedicated-2 → 99999 (`:1463-1464`), default 3000, clamp
[1000, 90000] (`:1470-1476`); `handicap` clamp (`:1479-1485`); `snaps` →
`snapshotMsec = 1000/i`, i clamp [1,30], default 50 ms (`:1487-1499`).
`SV_RateMsec` (`sv_snapshot.cpp:622-643`): message size clamp ≤ 1500
(`:628-630`); `sv_maxRate` floor-1000/override (`:633-638`); `rateMsec =
(size + HEADER_RATE_BYTES=48) * 1000 / rate` (`:622,640`).
`SV_SendMessageToClient` (`:652-707`): record
`messageSize/Sent/Acked=-1` (`:666-668`), `SV_Netchan_Transmit` (`:671`);
LAN → `nextSnapshotTime = svs.time - 1` (`:676-679`); else max(rateMsec,
snapshotMsec) with `rateDelayed` flag (`:684-690`, surfaced as
`SNAPFLAG_RATE_DELAYED`, `:150-151`); `nextSnapshotTime = svs.time +
rateMsec` (`:692`); non-ACTIVE non-downloading floor of +1000 ms (`:695-706`).

### 5.6 Per-client send sequence

`SV_SendClientMessages` (`sv_snapshot.cpp:806-832`): skip `!state`; skip
`svs.time < nextSnapshotTime`; unsent netchan fragments → transmit next
fragment + reschedule instead of building (`:820-827`); else
`SV_SendClientSnapshot` (`:830`). `SV_SendClientSnapshot` (`:719-798`):
one-off `svc_setgame` gamedir handshake if `!sentGamedir` (`:723-761`);
`SV_BuildClientSnapshot` (`:764`); **bots short-circuit — no network send**
(`SVF_BOT`, `:766-770`); assemble msg: `lastClientCommand` ack (`:777`),
`SV_UpdateServerCommandsToClient` (`:780`), `SV_WriteSnapshotToClient`
(`:784`), `SV_WriteDownloadToClient` (`:788`); overflow → warn + MSG_Clear
(`:792-795`); `SV_SendMessageToClient` (`:797`). No compression in this file
— Huffman lives inside netchan/msg (`SV_Netchan_Transmit` downstream).

---

## 6. sv_world.cpp deep dive

Structures: `worldSector_t` (file-local, `sv_world.cpp:48-53`: `axis` (-1 =
leaf), `dist`, `children[2]`, `entities` intrusive `svEntity_t` chain);
`sv_worldSectors[AREA_NODES = 64]` + count (`:55-59`); built once per spawn by
`SV_CreateworldSector` to `AREA_DEPTH = 4` alternating x/y splits (`:90-123`,
no bounds check against AREA_NODES — hand-sized constant); reset by
`SV_ClearWorld` memset (`:129-146`). `moveclip_t` is also file-local
(`:440-461`): `boxmins/boxmaxs` (swept envelope), `mins/maxs` (`const
float*`), `start/end`, `passEntityNum`, `contentmask`, `capsule`,
`traceFlags`/`useLod` (Ghoul2), `trace` accumulator.

### 6.1 Link/Unlink (recap — A2 §3 "Chain A" has the full trace)

`SV_LinkEntity` (`:189-347`) computes `s.solid`, abs bounds, PVS
cluster/area data, then splices the `svEntity_t` onto the deepest
non-straddled node's `entities` list (`:342-344`). `SV_UnlinkEntity`
(`:151-179`) reverse-splices with linear-scan fallback + warning (`:178`).
Quirk kept: the `SOLID_BMODEL` encoding-collision dodge (`i -= 1`, "yikes,
this would make everything explode violently", `:233-236`).

### 6.2 SV_AreaEntities

`areaParms_t` (`:359-364`); entry `SV_AreaEntities` (`:421-433`) recursing
`SV_AreaEntities_r` (`:373-414`): per node, walk `entities` chain
(prefetching `next` before evaluating, `:381` — the aliasing tell), resolve
via `SV_GEntityForSvEntity` (`:383`), 6-way AABB overlap reject on
absmin/absmax (`:385-392`; intersection, not exact touch, comment
`:354-355`); **MAXCOUNT abort is global**: hitting `maxcount` DPrintfs
`"SV_AreaEntities: MAXCOUNT"` and aborts the *entire remaining walk*, not the
node (`:394-397`); append index `check - sv.svEntities` (`:399-400`); leaf →
return (`:403-405`); recurse children per box-vs-dist straddle (`:407-413`).
Callers: `G_ENTITIES_IN_BOX` trap (`sv_game.cpp:582` →
`trap_EntitiesInBox`, pervasive in game code — the Chain-A reentrant path),
`SV_ClipMoveToEntities` (`sv_world.cpp:532`), `SV_PointContents` (`:883`).
**Not** used by snapshot visibility (§5.1).

### 6.3 SV_PointContents

`:871-903`: `CM_PointContents(p, 0)` world base (`:880`); zero-extent
`SV_AreaEntities(p, p, touch, MAX_GENTITIES)` (`:883`); per touch: skip
`passEntityNum` by index equality (`:886-888`), clip handle via
`SV_ClipHandleForEntity` (`:891`), `CM_TransformedPointContents(p, handle,
hit->s.origin, hit->s.angles)` (`:897`) — note **`s.origin/s.angles`, not
`r.currentOrigin/currentAngles`** as every other clip path uses (quirk, §6.6);
OR-accumulate (`:899`). Trap `G_POINT_CONTENTS` (`sv_game.cpp:597`); also
`sv_bot.cpp:361`.

### 6.4 SV_ClipHandleForEntity / SV_ClipToEntity

`SV_ClipHandleForEntity` (`:19-31`): `r.bmodel` →
`CM_InlineModel(s.modelindex)`; `SVF_CAPSULE` → `CM_TempBoxModel(mins, maxs,
qtrue)`; else box `CM_TempBoxModel(..., qfalse)`. `SV_ClipToEntity`
(`:470-503`): zero trace; contentmask pre-reject → `fraction = 1.0` (`:481-484`);
`CM_TransformedBoxTrace` in entity local space, `r.currentOrigin`/
`currentAngles`, angles zeroed for non-bmodels (`:487-498`); hit →
`trace->entityNum = touch->s.number` (`:500-502`). External caller:
`sv_bot.cpp:337`.

### 6.5 SV_Trace / SV_ClipMoveToEntities

`SV_Trace` (`:803-862`): null mins/maxs → `vec3_origin` (`:810-815`);
**phase 1 world**: `CM_BoxTrace(..., 0, contentmask, capsule)` (`:820`),
`entityNum = ENTITYNUM_WORLD`/`NONE` (`:821`); **early-out `fraction == 0`**
(`:822-825`); fill `moveclip_t`, swept `boxmins/boxmaxs` trimmed to the
world-unclipped remainder ("significant savings for line of sight and shot
traces", `:827-856`); **phase 2** `SV_ClipMoveToEntities(&clip)` (`:859`);
copy out (`:861`).

`SV_ClipMoveToEntities` (`:522-789`, static): `SV_AreaEntities` into a
**file-static `touchlist[MAX_GENTITIES]`** (`:523,532`); `passOwnerNum`
resolution (`:534-541`); `SVF_OWNERNOTSHARED` read that unconditionally
dereferences the pass entity even for ENTITYNUM_NONE (`:543-546`, quirk
§6.6); per entity: **`allsolid` early-return at loop top** (`:549-551`);
pass/owner/missile exclusion chain incl. an `ET_MISSILE` "blah, hack"
special case (`:555-585`); contentmask + `CONTENTS_NOSHOT` rejects
(`:589-596`); `CM_TransformedBoxTrace` (`:599-611`); allsolid/startsolid
propagation with forced `entityNum` stamp (`:619-628`); nearest-fraction keep
preserving prior startsolid (`:630-639`); Ghoul2 per-poly refinement block
gated on `G2TRFLAG_DOGHOULTRACE` + `touch->ghoul2` — the `#if 0` half is dead,
the MP `#else` (`:687-784`) refines endpos/normal or reverts to `oldTrace`.
Callers: `G_TRACE`/`G_G2TRACE`/`G_TRACECAPSULE` traps
(`sv_game.cpp:588,591,594`), `sv_bot.cpp:46,50,312`,
`NPCNav/navigator.cpp:748,1065,1129,2111`.

### 6.6 Oracle quirks worth preserving/flagging

1. Global (not per-subtree) MAXCOUNT abort in `SV_AreaEntities_r`
   (`sv_world.cpp:394-397`) — callers with small maxcount (e.g. 128 in
   `g_combat.c:1790`) can miss entities by traversal order, not proximity.
2. File-static `touchlist` in `SV_ClipMoveToEntities` (`:523`) — clobbered on
   reentry; same class as the `cmd_common.cpp:290-292` tokenizer scratch.
3. Unguarded pass-entity deref for `SVF_OWNERNOTSHARED` (`:543-546`) vs the
   guarded `passOwnerNum` block (`:534-541`).
4. `SV_PointContents` uses `s.origin/s.angles` where all sibling paths use
   `r.currentOrigin/currentAngles` (`:897` vs `:489-490,601-602`).
5. `SOLID_BMODEL` collision dodge in `SV_LinkEntity` (`:233-236`).
6. `AREA_NODES = 64` headroom for depth-4 splits with no overflow check in
   `SV_CreateworldSector` (`:55-56,90-123`).

---

## 7. sv_ccmds command surface (slice relevance)

Full registration table in §1.2; per-command behavior:

| command | fn (def line, sv_ccmds.cpp) | behavior | slice |
|---|---|---|---|
| `map`/`devmap`/`spmap`/… | `SV_Map_f` :138 | §2.1 | **0-1** |
| `map_restart` | `SV_MapRestart_f` :234 | §2.2 | **1-2** |
| `killserver` | `SV_KillServer_f` :947 | `SV_Shutdown("killserver")` | **0** |
| `status` | `SV_Status_f` :669 | map + per-client score/ping/name/lastmsg/address/qport/rate table | **0-2** (transcript diffing) |
| `serverinfo` | `SV_Serverinfo_f` :888 | `Info_Print(Cvar_InfoString(CVAR_SERVERINFO))` | 0-2 (diag) |
| `systeminfo` | `SV_Systeminfo_f` :904 | `Info_Print(Cvar_InfoString(CVAR_SYSTEMINFO))` | 0-2 (diag) |
| `dumpuser` | `SV_DumpUser_f` :917 | print a client's userinfo | 2 (diag) |
| `kick` / `clientkick` | `SV_Kick_f` :455 / `SV_KickNum_f` :636 | drop by name (`all`/`allbots` special) / by slot | 2 |
| `heartbeat` | `SV_Heartbeat_f` :876 | `svs.nextHeartbeatTime = -9999999` | rare |
| `svsay` | `SV_ConSay_f` :757 | dedicated console chat broadcast | cosmetic |
| `forcetoggle` | `SV_ForceToggle_f` :817 | toggle `g_forcePowerDisable` bit | cosmetic |
| `banUser`/`banClient` | :523/:580 | CD-key authorize-server relay, `#ifdef USE_CD_KEY` — dead infra | never |

No `rehashbans`/ban-file system exists in MP (grepped). Helpers:
`SV_GetPlayerByName` :43, `SV_GetPlayerByNum` :89, `SV_KickByName` :389,
`SV_GetStringEdString` :16 (`@@@` StringEd token relay for client-side
localization).

---

## 8. Configstrings + userinfo systems

### 8.1 SV_SetConfigstring — broadcast rule

`sv_init.cpp:25-91`: bounds `Com_Error(ERR_DROP)` (`:30-32`;
`MAX_CONFIGSTRINGS = 1700`, `game/q_shared.h:2037`); NULL → `""` (`:34-36`);
no-op if unchanged (`:39-41`); `Z_Free` + `CopyString` into
`sv.configstrings[index]` (`:44-45`; array `server.h:67`). **Broadcast only
when `sv.state == SS_GAME || sv.restarting`** (`:49`) — during a fresh
SS_LOADING the strings ride the initial gamestate instead. Broadcast is an
*immediate reliable command* (not snapshot-deferred): loop clients `state >=
CS_PRIMED` (`:52-53`), skip CS_SERVERINFO for `SVF_NOSERVERINFO` entities
(`:56-59`); strings ≥ `MAX_STRING_CHARS - 24` chunked as `bcs0`/`bcs1`/`bcs2`
commands, else single `cs %i "%s"` (`:61-88`), all via
`SV_SendServerCommand`.

`SV_GetConfigstring` — `sv_init.cpp:101-114` (bounds + empty-slot handling).
`SV_AddConfigstring` — `sv_init.cpp:123-160` (dedup slot claim in
`[start+1, start+max)`, strips leading `/`/`\`). There is **no**
`SV_UpdateConfigStrings` function in MP (grepped); the per-frame refresh is
inline in SV_Frame (§3.7).

### 8.2 Userinfo

`SV_SetUserinfo` — `sv_init.cpp:168-179`: bounds vs `sv_maxclients`, stores
into `svs.clients[index].userinfo` (`MAX_INFO_STRING = 1024`,
`game/q_shared.h:384`) and extracts `name` (`:177-178`). `SV_GetUserinfo` —
`sv_init.cpp:189-197`. Both trap-exposed (`G_SET_USERINFO`/`G_GET_USERINFO`,
`sv_game.cpp:615-620`). Client-initiated changes: `userinfo` ucmd →
`SV_UpdateUserinfo_f` (`sv_client.cpp:1511`, table `:1542`) →
`SV_UserinfoChanged` (`:1452-1500`, §5.5 clamps) → `VM_Call(gvm,
GAME_CLIENT_USERINFO_CHANGED, ...)` (`:1533`).

### 8.3 systeminfo propagation / pure list

No dedicated builder — two call sites of `Cvar_InfoString(_Big)`:
1. Spawn: `Cvar_InfoString_Big(CVAR_SYSTEMINFO)` (16 KB buffer,
   `sv_init.cpp:476,767`) → `CS_SYSTEMINFO` (index 1, `q_shared.h:2042`);
   `Cvar_InfoString(CVAR_SERVERINFO)` → `CS_SERVERINFO` (index 0,
   `q_shared.h:2041`) — `sv_init.cpp:766-772`, clearing `cvar_modifiedFlags`
   bits.
2. Per frame: modified-flag re-push (`sv_main.cpp:887-895`) — this is what
   makes a live `/set fraglimit 30` propagate (via §8.1's reliable
   broadcast, since state is SS_GAME).
The pure list is just cvars: `sv_paks`/`sv_pakNames`/`sv_referencedPaks`/
`sv_referencedPakNames` are CVAR_SYSTEMINFO|ROM (`sv_init.cpp:844-847`) set
from `FS_*PakChecksums/Names()` at spawn (`sv_init.cpp:738-764`) and ride
CS_SYSTEMINFO. Client-side verification comes back through
`SV_VerifyPaks_f` (`sv_client.cpp:1283`) XORing against `sv.checksumFeed`.

---

## 9. MP/SP diffs

Builds on A2 §1m (SP server-global census); differences only.

### 9.1 server.h struct diffs

- **`server_t`**: SP (`oracle/code/server/server.h:48-72`) has no
  `restarting`/`restartedServerId`/`checksumFeed` and — critically — none of
  MP's VM-handoff fields `gentities`/`gentitySize`/`num_entities`/
  `gameClients`/`gameClientSize`/`mSharedMemory`
  (`oracle/codemp/server/server.h:72-87`): SP's game is statically
  linked, nothing to LocateGameData. SP adds `timeResidualFraction` +
  `nextFrameTime` (`code/server/server.h:57-58`) for its sub-ms clock.
- **`serverStatic_t`**: SP (`code/server/server.h:142-149`) drops the whole
  `challenges[1024]` table, `redirectAddress`/`authorizeAddress` (no rcon),
  `snapFlagServerBit` — and has **no `time` field at all**: SP clocks off
  `sv.time` (reset per map), MP off `svs.time` (persistent).
- **`client_t`**: SP (`code/server/server.h:99-130`) drops download fields,
  `challenge`, `reliableSent`, `messageAcknowledge`, `pureAuthentic`,
  `sentGamedir`, userinfo-flood throttles; SP's `reliableCommands` is
  `char*[MAX_RELIABLE_COMMANDS]` (`code/server/server.h:103`) vs MP's fixed
  2-D char array (`codemp/server/server.h:130`).
- **`clientSnapshot_t`**: SP has no `vps` vehicle playerstate
  (`codemp/server/server.h:98` vs `code/server/server.h:76-87`).
- **Ghoul2 in the header**: the only ghoul2-typed surface is SP's `SV_Trace`
  prototype taking `const EG2_Collision eG2TraceType = G2_NOCOLLIDE, const
  int useLod = 0` (C++ default args, `code/server/server.h:274`) where MP
  takes plain ints `capsule, traceFlags, useLod`
  (`codemp/server/server.h:416`). No ghoul2 struct fields in
  `server_t`/`client_t` on either side.
- **Entity pointer types**: SP's server API is raw `gentity_t*` throughout —
  `SV_AreaEntities` returns `gentity_t **elist`
  (`code/server/server.h:199-274`); MP uses `sharedEntity_t*` + entity-number
  `int*` lists (`codemp/server/server.h:349-403`) — the VM-boundary
  indirection of A2 §4.1 visible in the query surface.
- MP-only in server.h: `SV_ClipToEntity` (:428), `SV_ChangeMaxClients`/
  `SV_RestartGameProgs`, the sv_bot prototype block (:359-376), networked
  cvar externs (:238-265). SP-only: the savegame prototype block
  (`code/server/server.h:290-319`).

### 9.2 SV_SpawnServer / SV_Frame / sv_client diffs

- Signatures: SP `SV_SpawnServer(char *server, ForceReload_e, qboolean
  bAllowScreenDissolve)` (`code/server/sv_init.cpp:234`) — no `killBots`;
  MP has no screen-dissolve param.
- SP zeroes `sv` with a flat memset (`code/server/sv_init.cpp:370`) vs MP's
  `SV_ClearServer()`; SP's reconnect loop is hardcoded `for (i=0; i<1; i++)`
  (`code/server/sv_init.cpp:419`) calling `ge->ClientConnect(i, qfalse, eNO)`
  through the static `game_export_t*` (`:430`) instead of `VM_Call`. No bot
  path; no savegame hook inside SV_SpawnServer itself — save/load is driven
  from sv_ccmds + `SV_TryLoadTransition` (`code/server/sv_savegame.cpp:517`).
- Boot banner asymmetry (transcript-diff relevant): SP prints the version in
  the opening banner and a closing `"-----------------------------------\n"`
  (`code/server/sv_init.cpp:272-273,469`); MP prints only the two opening
  lines, no closing rule (`codemp/server/sv_init.cpp:486-487`; grep-verified).
- SP `SV_Frame(int msec, float fractionMsec)` (`code/server/sv_main.cpp:470`)
  accumulates `sv.timeResidualFraction` gated by `cl_newClock`
  (`:506-514`); calls `SG_TestSave()` every frame (`:559`, savegame
  stress hook). SP has **no** NET_Sleep dedicated gate, no master heartbeat,
  no SV_BotFrame, no snapshotEntities-wrap or restartTime checks; time-wrap
  just shuts down with a joke message (`code/server/sv_main.cpp:523-526`).
- **Single-client hardcoding**: no `sv_maxclients` cvar in SP at all; loops
  are `i < 1` (`code/server/sv_client.cpp:59,76`;
  `code/server/sv_ccmds.cpp:336`); `svs.clients = Z_Malloc(sizeof(client_t)
  * 1)` (`code/server/sv_init.cpp:166`); `SV_DirectConnect` rejects any
  non-local address before any challenge logic
  (`code/server/sv_client.cpp:46-49`) — SP structurally cannot accept a
  remote connection.
- sv_ccmds: SP registers 9 commands (`code/server/sv_ccmds.cpp:447-462`) —
  `status`/`serverinfo`/`systeminfo`/`dumpuser`/`sectorlist`, 6 map variants,
  `maptransition`, and SP-only `load`/`loadtransition`/`save`/`wipe`. No
  kick/ban/map_restart/killserver/svsay/forcetoggle/heartbeat.
  `SV_Status_f` header + row format strings are byte-identical MP↔SP
  (`code/server/sv_ccmds.cpp:334-335` / `codemp/server/sv_ccmds.cpp:696-697,
  724`); MP adds a `notrunc` variant (`:737`).
- sv_snapshot/sv_world: SP-only `SV_PlayerCanSeeEnt(gentity_t*, int)`
  (snapshot sight-level filter); MP-only `SV_RateMsec` (no rate governor for
  a loopback client) and `SV_ClipToEntity`.

### 9.3 sv_savegame.cpp headline (SP-only, 2002 lines)

Implements `save`/`load`/`loadtransition`/`wipe`
(`code/server/sv_ccmds.cpp:459-461`) + `SV_TryLoadTransition`
(`code/server/sv_savegame.cpp:517`). `SG_WriteSavegame` (`:1176`) writes a
JPEG screenshot, comment/map chunks, all non-CVAR_INTERNAL cvars
(`SG_WriteCvars`, chunk tags `CVCN`/`CVAR`/`VALU`, `:817-828`),
`sv.time`/`sv.timeResidual` directly (`:1245-1246`), all
`sv.configstrings[]` (`:1248,870-881`), CM portal state (`:1247`), and
delegates entity/player state to `ge->WriteLevel(qbAutosave)` (`:1250`) — it
touches `server_t` in place but reaches client/entity state only through the
game module.

### 9.4 sv_bot.cpp + NPCNav headline (MP-only)

`sv_bot.cpp` (797 lines): botlib bridge — waypoint reception/path calc
(`SV_BotWaypointReception` :63, `SV_BotCalculatePaths` :81), botlib
setup/shutdown (`:598,619`), bot client slot alloc/free (`:178,208`), debug
polygons (`:471-524`). File-scope statics per A2 §1h (`gWPNum`/`gWPArray`,
`sv_bot.cpp:16-23`). `NPCNav/navigator.cpp` (2783 lines) is a self-contained
C++ node-graph pathfinder — `CNode`/`CEdge` with STL containers
(`navigator.h:35-45`), `CNavigator::CalculatePath(s)`
(`navigator.cpp:814,884`), best-first search (`:1182`),
`GetBestPathBetweenEnts` (`:1320`) — reached from the game module only via
the `GNavCallback_*` externs (`navigator.cpp:21-29`) bridged by the 49-line
`NPCNav/gameCallbacks.cpp`. C++-track per porting-rules §F when its slice
arrives; irrelevant to slices 0-2.

---

## 10. TU-harness candidates (DEC-09) + live-peer hooks

DEC-09 (`docs/decisions.md:109-122`): layer 1 = TU golden harnesses
(`tools/gp2-oracle` pattern — compile the unmodified oracle TU standalone
with stub headers, golden-diff canonical dumps); layer 2 = live-peer scripted
sessions against retail/OpenJK, diffing observable behavior.

### 10.1 Golden-testable standalone (layer 1)

| Candidate | Why it stands alone | Anchor |
|---|---|---|
| `clientState_t` transition machine | 5 states driven by DirectConnect/DropClient/CheckTimeouts; mock `client_t[]` + fake `svs.time`, no sockets | `codemp/server/server.h:114-121`, §4 |
| `SV_SetConfigstring` bookkeeping | strcmp change-detect, Z_Free/CopyString swap, bcs0/1/2 chunking threshold — string logic over an array; only the broadcast tail needs a client fixture | `sv_init.cpp:25-91` |
| `Cvar_InfoString(bit)` filtering | pure list filter (`!(CVAR_INTERNAL) && (flags & bit)`); feeds serverinfo/systeminfo/getstatus — golden-diff the byte-exact info string from a canned cvar set | `oracle/codemp/qcommon/cvar.cpp:811-824` |
| `SV_RateMsec` + nextSnapshotTime math | pure arithmetic (size clamp, sv_maxRate override, 48-byte header, snaps-vs-rate max) | `sv_snapshot.cpp:622-643,684-692` |
| `SV_CalcPings` | arithmetic over synthetic `frames[]` (`messageAcked - messageSent`, clamp 999) | `sv_main.cpp:659-704` |
| `SV_EmitPacketEntities` parallel-walk | old/new list merge → delta/baseline/remove decisions; feed canned `clientSnapshot_t` pairs + msg goldens (pairs with the existing msg/huffman harness plan, engine-seam.md) | `sv_snapshot.cpp:36-94` |
| moveclip composition (`SV_Trace` phases, exclusion chain) | needs a CM stub or a real loaded BSP fixture, but no network/VM; the pass/owner/missile exclusion logic (`:555-585`) is a pure decision table | `sv_world.cpp:522-789,803-862` |
| `SV_AreaEntities_r` walk + MAXCOUNT abort | canned worldSector tree + svEntity chains; asserts the *global-abort* quirk (§6.6.1) faithfully | `sv_world.cpp:373-414` |
| `SV_Status_f` / status formats | static printf formats, byte-identical MP/SP — synthetic client fixtures → transcript golden | `sv_ccmds.cpp:696-747` |
| SV_SpawnServer step ordering | not a TU harness per se, but the numbered script (§2) is assertable as an event-trace golden once the Rust spawn emits a structured trace |  |

### 10.2 Needs the live-peer layer (layer 2)

- Full `SV_DirectConnect` handshake — challenge exchange, reconnect-too-soon
  timing, qport disambiguation in `SV_PacketEvent`
  (`sv_main.cpp:612-644`): real OOB packets, two endpoints.
- Netchan send/ack + fragmentation (`SV_Netchan_Transmit/Process`,
  `server.h:434-436`) — real packet sequencing over a real transport.
- Snapshot delta correctness end-to-end — entity selection is TU-able, but
  "does a real client decode it" is the live check (a retail/OpenJK client
  connecting to the Rust dedicated server is the strongest oracle).
- `SV_ConnectionlessPacket` under real UDP framing (`\xff\xff\xff\xff`
  marker + huff-decompress-on-connect, `sv_main.cpp:545-553`).
- SP savegame round-trip — delegates to `ge->WriteLevel/ReadLevel`, needs a
  real game module; only the server-owned chunks (cvars, configstrings,
  sv.time) TU-test in isolation.

### 10.3 Scripted dedicated-session transcript diff — what it captures

The slice-0/1 acceptance artifact: run `jamp dedicated +map <x>`, capture
stdout + scripted OOB queries, diff against the same script on OpenJK/retail.

- **Boot lines**: `"------ Server Initialization ------\n"` + `"Server:
  %s\n"` (`sv_init.cpp:486-487`; MP has no closing rule — SP does,
  `code/server/sv_init.cpp:272-273,469` — the harness must be
  variant-aware).
- **`status` output**: header `"num score ping name            lastmsg
  address               qport rate\n"` + dashes + rows `"%3i %5i %s %-15.15s
  %7i %21s %5i %5i\n"` (`sv_ccmds.cpp:696-724`).
- **`getstatus` OOB** → `SVC_Status` (`sv_main.cpp:320-371`):
  `"statusResponse\n%s\n%s"` = `Cvar_InfoString(CVAR_SERVERINFO)` with
  `challenge` echoed (`:341`) + one `"%i %i \"%s\"\n"` score/ping/name line
  per connected client (`:359`). Directly exercises the §1.1 SERVERINFO cvar
  table — a boot-time getstatus is a machine-checkable cvar-defaults golden.
- **`getinfo` OOB** → `SVC_Info` (`sv_main.cpp:381-469`): hand-built keys
  `challenge, protocol, hostname, mapname, clients, sv_maxclients (minus
  private), gametype, needpass, truejedi, wdisable, fdisable
  [, minPing, maxPing, game]` (`:416-448`) — *not* the SERVERINFO filter.
- **`rcon`** (`sv_main.cpp:570-576`) extends the scriptable surface to every
  §7 command without a connected client. SP's OOB surface is strictly
  smaller: `getstatus/getinfo/connect/disconnect` only
  (`code/server/sv_main.cpp:251-260`) — no getchallenge/rcon.

---

## Design forks

Framed for the design session; each fork cites the constraint that forces it.

1. **`client_t` storage: fixed array vs slotmap.** Raven allocates
   `svs.clients = Z_Malloc(sizeof(client_t) * sv_maxclients)` once per
   `SV_Startup` and identifies clients by pointer arithmetic (`cl -
   svs.clients`) everywhere — including across the seam (`GAME_CLIENT_*`
   VM_Calls take the index, `sv_client.cpp:520`, `sv_init.cpp:700`). The
   CS_FREE/CS_ZOMBIE lifecycle *is* the slot allocator (§4.1). Fork:
   (a) faithful `Vec<Client>` sized `sv_maxclients` with the state enum as
   the free-list — index identity preserved by construction, matches
   `SV_ChangeMaxClients` realloc semantics (`sv_init.cpp:299-358`); or
   (b) a generational slotmap — cleaner aliasing story, but clientNum is a
   wire/ABI-visible index (gamestate `MSG_WriteLong(clientNum)`,
   `sv_client.cpp:762`; `GAME_CLIENT_CONNECT i`), so generations would have
   to be stripped at every boundary. The oracle-parity price of (b) is high
   and constant; (a) with a `ClientId(u32)` newtype per rules §B5 looks
   strictly better. Decide: does zombie aging (`SV_CheckTimeouts`,
   `sv_main.cpp:736-738`) live in the array type or in SvFrame logic?

2. **svEntity/worldSector arena shape under multi-world (STATE-D2).** Raven
   has three coupled arrays: `sv.svEntities[MAX_GENTITIES]` (`server.h:68`),
   file-scope `sv_worldSectors[64]` + count (`sv_world.cpp:58-59`), and the
   intrusive `worldSector->entities` chains threaded by raw pointers
   (`svEntity_t.worldSector`/`nextEntityInWorldSector`, `server.h:28-29`).
   Per STATE-D2 (`docs/architecture/two-island-model.md:92-94`) the world
   must be a value: `sv_worldSectors` cannot stay file-scope — it moves into
   the server-world struct alongside `svEntities`. Fork on chain
   representation: (a) faithful intrusive links as `Option<u32>` indices
   (sector id + next-svEntity id) — preserves Chain-A link-during-iteration
   semantics exactly (A2 §3; the `next` prefetch at `sv_world.cpp:381` is
   the pattern to keep); (b) per-sector `Vec<EntityId>` — simpler but
   changes eviction order and makes the mid-walk MAXCOUNT abort (§6.6.1)
   harder to reproduce faithfully. `snapshotCounter` de-dup stamps
   (`server.h:43`) stay per-svEntity either way. Also decide where the
   file-static `touchlist` scratch (§6.6.2) goes — a stack `[i32;
   MAX_GENTITIES]` is 4 KB×? (fine) vs a per-world scratch buffer that
   documents the oracle's non-reentrancy.

3. **snapshotEntities ring ownership.** Raven: heap `entityState_t[]` in
   `svs`, sized by boot-time dedicated/listen branch (`sv_init.cpp:264-274`),
   freed/reallocated per spawn (`:493-497,594-596`), with a monotonic global
   cursor consumed modulo by per-client frames (§5.3) — clients' delta
   validity depends on the *shared* cursor (`sv_snapshot.cpp:128-133`).
   Fork: (a) keep the shared ring in the persistent server-static struct
   (faithful; the staleness check ports as-is); (b) per-client snapshot
   history (owns its entities) — kills the shared-cursor coupling but
   changes memory shape and the "Delta request from out of date entities"
   behavior. (a) is the parity choice; (b) only worth revisiting after
   live-peer green.

4. **Where GAME_* dispatch state lives (two-island model).** Every
   `VM_Call(gvm, GAME_*)` site enumerated here — GAME_INIT (§2.33),
   GAME_RUN_FRAME (§2.35, §3.9), GAME_CLIENT_CONNECT (§4.4.7, §2.37),
   GAME_CLIENT_BEGIN (§4.6, §2.37), GAME_CLIENT_DISCONNECT (§4.4.4, §4.8.6),
   GAME_CLIENT_USERINFO_CHANGED (§8.2), GAME_SHUTDOWN (§2.3) — is reached
   while `&mut` server state is live (e.g. SV_DirectConnect holds `newcl`
   across the CONNECT call). Per the two-island model
   (`docs/architecture/two-island-model.md:71-83`), the fork is: (a) the
   server holds a transport handle (`Static`/`NativeDll` enum per SEAM-D5)
   and every call site re-borrows engine state around the call (locals-only
   across VM_Call, matching `sv_main.cpp:909-915`); or (b) a mediator
   object owning both islands. (a) is what the frozen seam docs assume;
   this dossier's contribution is the *call-site inventory* above — each
   one is a point where the Rust port must drop borrows before dispatch.
   Note the GAME_CLIENT_CONNECT denial string (`VM_ExplicitArgPtr` resolve,
   `sv_client.cpp:523`) is module-memory read back by the engine — a seam
   crossing beyond the 12-word convention, same family as SEAM-Q1.

5. **Configstring storage + broadcast coupling.** `sv.configstrings` is
   `char*[1700]` of Z_Malloc'd strings with set-time broadcast side effects
   gated on `sv.state`/`sv.restarting` (§8.1). Fork: (a)
   `Vec<String>`/`Box<[String; 1700]>` with the broadcast as an explicit
   call into the client list (state threaded per rules §B4); (b) an
   observer/dirty-flag scheme. (a) is faithful — the broadcast is
   *immediate reliable enqueue*, not deferred, and same-frame ordering with
   other reliables is observable by clients; a dirty-flag batch would
   reorder.

6. **SP/MP sharing.** The state machines are near-identical but the types
   differ structurally (§9.1: SP leaner structs, `gentity_t*` vs
   `sharedEntity_t*` surfaces, sv.time vs svs.time, `i < 1` loops). Fork:
   (a) two ported server crates (mirrors the existing mp_*/sp_* crate split
   and the oracle's two trees — no risk of MP semantics bleeding into SP);
   (b) shared core generic over an entity-access trait. Given the crate
   graph already commits to per-variant engine crates
   (docs/workspace-architecture.md) and the diffs above are pervasive
   rather than parametric, (a) matches the codebase's existing convention;
   any sharing should be discovered post-parity, not designed up front.
