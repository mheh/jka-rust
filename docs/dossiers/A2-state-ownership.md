# A2 — State Ownership: Ground-Truth Dossier

Scope: raw material for the design session on porting-rules §B (no `static
mut`; state threaded not reached; one owned instance per singleton; entities
by index/handle). Every claim cites `oracle/<path>:<line>`. MP tree =
`oracle/codemp/`, SP tree = `oracle/code/`.

Status: complete.

---

## 1. The global census — engine tier

Build-canonical caveat (qcommon): the real PC build (per
`oracle/codemp/unix/makefile:302-307`) links `cmd_common.cpp`+
`cmd_pc.cpp`, `files_common.cpp`+`files_pc.cpp`, and `z_memman_pc.cpp`.
`files.cpp`, `cmd_console.cpp`, `files_console.cpp`,
`z_memman_console.cpp` are dead/console-platform duplicates (same globals
under `static` linkage) and are excluded below.

### 1a. qcommon — common.cpp

| Name | Type | File:Line | Size | Mutators | Cross-boundary? |
|---|---|---|---|---|---|
| `com_frameTime` | `int` | `oracle/codemp/qcommon/common.cpp:79` | 4B | `Com_Frame` (via `Com_EventLoop`/`Com_Milliseconds`, L1402,1648) | Yes — `client/cl_input.cpp`, `server/sv_ccmds.cpp`, `server/sv_init.cpp` |
| `com_frameMsec` | `int` | `common.cpp:80` | 4B | `Com_Frame` L1659 | No |
| `com_frameNumber` | `int` | `common.cpp:81` | 4B | `Com_Frame` L1754 (`++`) | No |
| `com_errorEntered` | `qboolean` | `common.cpp:83` | 4B | `Com_Error` L291,305,318,332 | Yes — `server/sv_init.cpp` |
| `com_fullyInitialized` | `qboolean` | `common.cpp:84` | 4B | `Com_Init` L1434 | Only alt-platform dup (`unix/files_linux.cpp`) |
| `com_errorMessage[MAXPRINTMSG]` | `char[4096]` | `common.cpp:86` | 4KB | `Com_Error` | Error path only |
| `rd_buffer`/`rd_buffersize`/`rd_flush` | `char*`/`int`/fn-ptr | `common.cpp:92-94` | ptr+4B+ptr | `Com_BeginRedirect`, `Com_EndRedirect`, `Com_Printf` | No (rcon print-redirect internals) |
| `com_journalFile`/`com_journalDataFile` | `fileHandle_t` | `common.cpp:34-35` | 4B each | `Com_InitJournaling`, `Com_GetRealEvent` | Extern'd in `qcommon.h`; `unix/files_linux.cpp` |
| `com_pushedEventsHead`/`Tail`, `com_pushedEvents[1024]` | `static int`×2, `static sysEvent_t[1024]` | `common.cpp:749-752` | ~1024×sizeof(sysEvent_t) | `Com_InitJournaling` L840-841, `Com_PushEvent` L867-873, `Com_GetEvent` L883 | No (file-static) |
| `com_numConsoleLines`/`com_consoleLines[32]` | `int`/`char*[32]` | `common.cpp:387-388` | ~260B | `Com_ParseCommandLine` L399,409 | No |
| `com_argc`/`com_argv[51]` | `int`/`char*[]` | `common.cpp:22-23` | ~412B | Set once at startup | Extern'd in `qcommon.h` |
| ~30 `cvar_t*` statics (`com_dedicated`, `com_developer`, `com_timescale`, `com_sv_running`, `com_cl_running`, `com_speeds`, `com_maxfps`, `cl_paused`, `sv_paused`, `com_RMG`, …) | `cvar_t*` | `common.cpp:37-72` | 8B each (pointees owned by cvar.cpp) | `Cvar_Get` at init; values via `Cvar_Set2` | Yes, widely — `com_dedicated` → renderer (`tr_ghoul2.cpp`, `tr_image.cpp`), server, RMG, win32; `com_sv_running`/`com_cl_running` → client, server, renderer, ghoul2 (`G2_API.cpp`) |

### 1b. qcommon — cmd (cmd_common.cpp + cmd_pc.cpp)

| Name | Type | File:Line | Size | Mutators | Cross-boundary? |
|---|---|---|---|---|---|
| `cmd_text` + `cmd_text_buf[MAX_CMD_BUFFER]` | `cmd_t` (data/maxsize/cursize) + `byte[]` | `oracle/codemp/qcommon/cmd_common.cpp:17-18` | ~20B header + buffer | `Cbuf_Init`, `Cbuf_AddText`, `Cbuf_InsertText`, `Cbuf_Execute` | Extern'd `qcommon.h`; fed by client/server init and console paths |
| `cmd_wait` | `int` | `cmd_common.cpp:16` | 4B | `Cmd_Wait_f` / frame decrement | No |
| `cmd_argc`/`cmd_argv[MAX_STRING_TOKENS]`/`cmd_tokenized[]` | `static int`/`char*[]`/`char[]` | `cmd_common.cpp:290-292` | large tokenize buffer | `Cmd_TokenizeString` | No (file-static; shared tokenizer scratch — a reentrancy hazard in its own right) |
| `cmd_functions` | `static cmd_function_t*` (linked-list head, heap nodes) | `oracle/codemp/qcommon/cmd_pc.cpp:11` | 8B head, ~48B/node | `Cmd_AddCommand` L37-38, `Cmd_RemoveCommand` L49+ | No (all access via `Cmd_*` API) |

### 1c. qcommon — cvar.cpp

| Name | Type | File:Line | Size | Mutators | Cross-boundary? |
|---|---|---|---|---|---|
| `cvar_vars` | `cvar_t*` (list head) | `oracle/codemp/qcommon/cvar.cpp:6` | 8B | `Cvar_Get` L271 | No direct (API-only) |
| `cvar_cheats` | `cvar_t*` | `cvar.cpp:7` | 8B | Set once (`sv_cheats` handle) | No |
| `cvar_modifiedFlags` | `int` bitmask | `cvar.cpp:8` | 4B | `Cvar_Get` L220, `Cvar_Set2` L329 (`\|=`) | Yes — `client/cl_ui.cpp`, `cl_main.cpp`, `cl_keys.cpp`, `server/sv_main.cpp`, `sv_init.cpp` |
| `cvar_indexes[MAX_CVARS=1224]`/`cvar_numIndexes` | `cvar_t[]`/`int` | `cvar.cpp:11-12` | ~78KB | `Cvar_Get` (slot fill, L260 `++`) | No direct |
| `hashTable[256]` | `static cvar_t*[]` | `cvar.cpp:15` | 2KB | `Cvar_Get` L276-277 | No |
| `lastMemPool`/`memPoolSize` | `static char*`/`static int` | `cvar.cpp:20-21` | 12B | `Cvar_Realloc` pool path L1016-1017 | No |

### 1d. qcommon — files (files_common.cpp + files_pc.cpp)

| Name | Type | File:Line | Size | Mutators | Cross-boundary? |
|---|---|---|---|---|---|
| `fs_searchpaths` | `searchpath_t*` (list head) | `oracle/codemp/qcommon/files_common.cpp:193` | 8B head + heap nodes | `FS_AddGameDirectory` (`files_pc.cpp:2241-2288`), `FS_Shutdown` (`files_common.cpp:458`) | Extern'd `files.h`; unix dup only |
| `fs_gamedir[MAX_OSPATH]` | `char[]` | `files_common.cpp:183` | MAX_OSPATH | gamedir-set code in `files_pc.cpp` | Extern'd `files.h` |
| `fsh[MAX_FILE_HANDLES=16]` | `fileHandleData_t[16]` | `files_common.cpp:202` | 16 × mid struct | `FS_HandleForFile`, `FS_FOpenFileRead/Write`, `FS_FCloseFile` (`files_pc.cpp:284-322` etc.) | Engine-wide via handle API; symbol itself qcommon-only |
| `fs_readCount`/`fs_loadCount`/`fs_loadStack`/`fs_packFiles`/`fs_fakeChkSum`/`fs_checksumFeed` | `int` each | `files_common.cpp:194-200` | 4B each | read/load accounting, `FS_LoadZipFile`, `FS_ConditionalRestart` (`files_pc.cpp:~3048`) | Extern'd `files.h` |
| `fs_numServerPaks`/`fs_serverPaks[4096]`/`fs_serverPakNames[]`, `fs_numServerReferencedPaks`/`fs_serverReferencedPaks[]`/`fs_serverReferencedPakNames[]` | `int`/`int[]`/`char*[]` | `files_common.cpp:206-214` | ~48KB+ | `FS_PureServerSetLoadedPaks`-family (`files_pc.cpp`) | **Oracle bug surface — flag it**: `files_pc.cpp:2328-2330` re-declares the referenced-paks trio as file-local `static`, shadowing the `files_common.cpp` globals |
| `lastValidBase`/`lastValidGame` | `char[MAX_OSPATH]`×2 | `files_common.cpp:217-218` | 2×MAX_OSPATH | pak-validation fallback | No |
| `initialized` | `qboolean` | `files_common.cpp:224` | 4B | `FS_InitFilesystem`/`FS_Shutdown` | Extern'd `files.h` |
| `fs_debug`,`fs_homepath`,`fs_basepath`,`fs_basegame`,`fs_cdpath`,`fs_copyfiles`,`fs_gamedirvar`,`fs_restrict`,`fs_dirbeforepak` | `cvar_t*` | `files_common.cpp:184-192` | 8B each | `Cvar_Get` at `FS_InitFilesystem` | No (win32 references by cvar-name string only) |
| `fs_reordered` | `static qboolean` | `files_pc.cpp:21` | 4B | `FS_ReorderPurePaks` | No |

### 1e. qcommon — zone/hunk allocator (z_memman_pc.cpp)

MP's allocator is not vanilla Q3 mainzone/hunk_low/hunk_high — it's a
malloc-backed tagged zone plus a hunk emulation layered on tags.

| Name | Type | File:Line | Size | Mutators | Cross-boundary? |
|---|---|---|---|---|---|
| `TheZone` | `zone_t` (`zoneStats_t` per-tag stats + `zoneHeader_t` list head) | `oracle/codemp/qcommon/z_memman_pc.cpp:77` | few hundred bytes + heap blocks | `Z_Malloc`, `Zone_FreeBlock`, `Z_Free`, `Z_TagFree` (L276-351) | No (engine-wide via `Z_*` API only) |
| `hunk_tag` | `static memtag_t` | `z_memman_pc.cpp:626` | 4B | `Com_InitHunkMemory` L679, `Hunk_SetMark` L707, `Hunk_Clear` L764 | No (drives `Hunk_Alloc` semantics engine-wide) |
| `gbMemFreeupOccured` | `qboolean` | `z_memman_pc.cpp:156` | 4B | `Z_Malloc` L159 + freeup-retry path L198-256 | No |
| `com_validateZone` | `cvar_t*` | `z_memman_pc.cpp:75` | 8B | `Cvar_Get` at init | No |

### 1f. qcommon — net_chan.cpp

| Name | Type | File:Line | Size | Mutators | Cross-boundary? |
|---|---|---|---|---|---|
| `showpackets`/`showdrop`/`qport`/`net_killdroppedfragments` | `cvar_t*`×4 | `oracle/codemp/qcommon/net_chan.cpp:40-43` | 8B each | `Netchan_Init` (L58) | `qport` → `client/cl_input.cpp`, `cl_main.cpp`, `server/sv_client.cpp`, `sv_ccmds.cpp`, `sv_main.cpp` |
| `loopbacks[2]` | `loopback_t[2]` (msg ring buffers + get/send cursors) | `net_chan.cpp:486` | 2 × small ring | `NET_SendLoopPacket` L494, `NET_GetLoopPacket` L517 | Yes — `client/cl_input.cpp` |

Per-connection state is otherwise in `netchan_t`, owned by `clc`/`client_t` (as expected).

### 1g. qcommon — cm_load.cpp (collision model)

| Name | Type | File:Line | Size | Mutators | Cross-boundary? |
|---|---|---|---|---|---|
| `cmg` | `clipMap_t` (~30 members: shaders/planes/nodes/leafs/brushes/submodels/vis/entityString/areas/surfaces arrays + `landScape` ptr) | def `oracle/codemp/qcommon/cm_load.cpp:37`, extern `cm_local.h:220` | large struct, ~30 members, heap-owned arrays | `CM_LoadMap`→`CM_LoadMap_Actual`, `CM_ClearMap` L809-813 | Yes — `client/cl_cgame.cpp`, `server/sv_game.cpp`, `RMG/RM_Terrain.cpp` |
| `SubBSP[MAX_SUB_BSP=32]`/`NumSubBSP`/`TotalSubModels` | `clipMap_t[32]`/`int`/`int` | `cm_load.cpp:60-61` | 32 × clipMap_t | `CM_LoadSubBSP` L1105, `CM_ClearMap` L816-820 | No |
| `c_pointcontents`,`c_traces`,`c_brush_traces`,`c_patch_traces` | `int`×4 | `cm_load.cpp:38-39` | 4B each | trace counters in `cm_trace.cpp:503,1590` | No |
| `gpvCachedMapDiskImage`/`gbUsingCachedMapDataRightNow` | `void*`/`qboolean` | `cm_load.cpp:568,570` | 8B/4B | `CM_LoadMap_Actual` L586-753, `CM_LoadMap` L777-781 | Yes — `renderer/tr_bsp.cpp`, `tr_bsp_xbox.cpp` (CM↔renderer shared map-disk-image cache) |
| `cm_noAreas`/`cm_noCurves`/`cm_playerCurveClip` | `cvar_t*` | `cm_load.cpp:45-47` | 8B each | `Cvar_Get` at `CM_LoadMap` | No |

### 1h. Server (`oracle/codemp/server/`)

| Name | Type | File:Line | Size | Mutators | Cross-boundary? |
|---|---|---|---|---|---|
| `sv` | `server_t` (~17 fields; `server.h:53-88`) | def `oracle/codemp/server/sv_main.cpp:11`, extern `server.h:233` | Large — embeds `svEntities[MAX_GENTITIES]`, `configstrings[MAX_CONFIGSTRINGS]`, `models[MAX_MODELS]`; also `gentities`/`gameClients` raw pointers into the game module (§4.1), `snapshotCounter`, `checksumFeed`, `mSharedMemory` | `sv_init.cpp:649-777` (`SV_SpawnServer`), `sv_main.cpp:852-911` (`SV_Frame`: timeResidual), `sv_snapshot.cpp:525` (`snapshotCounter++`), `sv_ccmds.cpp:283-305` (map_restart), `sv_game.cpp:329-334` (`SV_LocateGameData`, §4.1) | **Yes** — `icarus/GameInterface.cpp:129,402,411`, `icarus/Q3_Interface.cpp` (casts `sv.mSharedMemory`), `qcommon/msg.cpp:1268` (reads `sv.state`), `RMG/RM_Instance_BSP.cpp:262-265` (saves/restores `sv.entityParsePoint`) |
| `svs` | `serverStatic_t` (~12 fields; `server.h:94,208-228`; persists across maps) | def `sv_main.cpp:10`, extern `server.h:232` | Large — embeds `challenges[MAX_CHALLENGES=1024]`, heap `clients[sv_maxclients]`, `snapshotEntities` ring buffer | `sv_init.cpp:264,338-356,594,965` (client/snapshot alloc, full memset on shutdown), `sv_main.cpp` (`svs.time` per frame), `sv_client.cpp:167-306` (challenge handshake), `sv_snapshot.cpp:604` | **Yes** — `icarus/*` and `qcommon/RoffSystem.cpp:612,828,864-886` read `svs.time`; `server/NPCNav/navigator.cpp:1409-1778` reads `svs.time` |
| `sv_worldSectors[AREA_NODES=64]`/`sv_numworldSectors` | `worldSector_t[64]`/`int` (file-scope in sv_world.cpp, **not** a field of `sv`) | `oracle/codemp/server/sv_world.cpp:58-59` (type decl :48-57) | Small fixed array, but each node's `entities` is a mutable intrusive linked list of `svEntity_t` | **`SV_LinkEntity`** `sv_world.cpp:189`, **`SV_UnlinkEntity`** `sv_world.cpp:151` (raw in-place pointer surgery on `nextEntityInWorldSector` chains — the Chain-A reentrancy hot spot, §3), `SV_CreateworldSector` :90, memset reset :135 | No — server-only, reached via `sv.svEntities[n].worldSector` back-pointers |
| `debugpolygons`/`gWPNum`/`gWPArray[MAX_WPARRAY_SIZE]` | `bot_debugpoly_t*`/`int`/`wpobject_t*[]` | `oracle/codemp/server/sv_bot.cpp:16-23` | small | bot waypoint/debug functions in `sv_bot.cpp` | No |
| `g_lastResolveTime[MAX_MASTER_SERVERS]` | `static int[]` | `sv_main.cpp:192` | tiny | master-server heartbeat DNS throttle | No |

### 1i. Client (`oracle/codemp/client/`)

| Name | Type | File:Line | Size | Mutators | Cross-boundary? |
|---|---|---|---|---|---|
| `cl` | `clientActive_t` (~30+ fields; `client.h:58-139`) | def `oracle/codemp/client/cl_main.cpp:105`, extern `client.h:139` | Very large — `snapshots[PACKET_BACKUP]`, `entityBaselines[MAX_GENTITIES]`, `parseEntities[MAX_PARSE_ENTITIES]`, `cmds[CMD_BACKUP]`, `gameState`, `mSharedMemory` (hundreds of KB) | `CL_ClearState` `cl_main.cpp:820`, `CL_ParseGamestate` `cl_parse.cpp:533`, `CL_ParseServerMessage` `cl_parse.cpp:854`, `CL_CreateNewCommands` `cl_input.cpp:1492`, `CL_WritePacket` `cl_input.cpp:1608` | No — confined to `client/` (incl. `snd_dma.cpp`) |
| `clc` | `clientConnection_t` (`client.h:~140-236`) | def `cl_main.cpp:106`, extern `client.h:236` | Very large — `netchan_t`, reliable-command rings (`MAX_RELIABLE_COMMANDS × MAX_STRING_CHARS`), download state, demo state, MP-only RMG heightmaps (2×16000B) | `CL_Disconnect` `cl_main.cpp:837`, `CL_Connect_f` `cl_main.cpp:1141`, `CL_ConnectionlessPacket` `cl_main.cpp:2028`, `CL_CheckTimeout` `cl_main.cpp:2212` | No — confined to `client/` |
| `cls` | `clientStatic_t` (`client.h:~240-382`) | def `cl_main.cpp:107`, extern `client.h:382` | Moderate — `state` (connstate_t), `keyCatchers`, server-browser lists, `glconfig_t` copy, frame timing | `CL_MapLoading` `cl_main.cpp:778`, `CL_Disconnect` :837, `CL_Connect_f` :1141; read/written across `cl_scrn.cpp`, `cl_console.cpp`, `cl_cgame.cpp`, `cl_ui.cpp`, `cl_cin*.cpp` | **Yes** — `qcommon/files.cpp:1247,1375` + `files_pc.cpp:848,977` read `cls.state`/`cls.keyCatchers` (pure-pak restart gating); `win32/win_input.cpp:722`, `win_input_console.cpp:209-505`; `unix/linux_glimp.c:479,1499` |
| `kg` | `keyGlobals_t` (bundles `keys[MAX_KEYS]`, edit-line history, `g_consoleField`, `anykeydown`, `key_overstrikeMode`, `keyDownCount`) | def `oracle/codemp/client/cl_keys.cpp:22`, decl `keys.h:19,45` | moderate | `Key_SetBinding`, `Key_SetOverstrikeMode`, `Key_ClearStates`, key-event handlers (all `cl_keys.cpp`) | No — already pre-grouped into one struct by Raven (a natural Rust `KeyState`) |
| `chatField`/`chat_team`/`chat_playerNum` | `field_t`/`qboolean`/`int` | `cl_keys.cpp:12-15`, decl `keys.h:55-57` | small | chat input handlers `cl_keys.cpp` | No |
| `con` | `console_t` (`text[CON_TEXTSIZE=32768]` ring) | def `oracle/codemp/client/cl_console.cpp:13`, type `client.h:358-380` | ~32KB | console print/scroll in `cl_console.cpp`; read by `SCR_UpdateScreen` `cl_scrn.cpp:479` | No (extern'd into `cl_scrn.cpp:8`, same dir) |
| `scr_initialized` | `qboolean` | `oracle/codemp/client/cl_scrn.cpp:9` | 4B | `SCR_Init` `cl_scrn.cpp:376` | No |
| debug graph: `current`/`values[1024]` | `static int`/`static graphsamp_t[]` | `cl_scrn.cpp:318-319` | small | `SCR_DebugGraph` :326, `SCR_DrawDebugGraph` :338 | No |
| centerprint: `scr_centertime_off`,`scr_center_y`,`scr_centerstring[1024]`,`scr_center_lines`,`scr_center_widths[]` | `float`/`int`/`char[]` | `cl_scrn.cpp:510-515` | small | `SCR_CenterPrint` :519 | No |

### 1j. Sound (`oracle/codemp/client/snd_dma.cpp` + `snd_mix.cpp`)

| Name | Type | File:Line | Size | Mutators | Cross-boundary? |
|---|---|---|---|---|---|
| `s_channels[MAX_CHANNELS]` | `channel_t[]` | `oracle/codemp/client/snd_dma.cpp:127` | fixed array | `S_StartSound` :1541, `S_StopAllSounds` :1839, `S_PickChannel`/`S_OpenALPickChannel` | No — game/cgame trigger sounds via `trap_S_StartSound`, never touch the array |
| `s_soundStarted`/`s_soundMuted` | `int`/`qboolean` | `snd_dma.cpp:129-130` | 4B each | `S_Init` :419, `S_Shutdown` :650, `S_Update` :2700 | No |
| `dma` | `dma_t` (device buffer descriptor) | `snd_dma.cpp:132` | small struct | `SNDDMA_*` init/shutdown, `S_Update_` :2787 | No |
| `listener_number`/`listener_origin`/`listener_axis[3]` | `int`/`vec3_t`/`vec3_t[3]` | `snd_dma.cpp:134-136` | small | `S_Update` :2700 spatialization | No |
| `s_soundtime`/`s_paintedtime` | `int` sample counters | `snd_dma.cpp:138-139` | 4B each | mixer paint loop (`snd_mix.cpp`), `S_Update` | No |
| `s_knownSfx[MAX_SFX=10000]`/`s_numSfx` | `sfx_t[]`/`int` | `snd_dma.cpp:144-145` | large registry | `S_RegisterSound` family | No — handles cross the VM seam as opaque `sfxHandle_t` ints |
| `sfxHash[LOOP_HASH=128]` | `static sfx_t*[]` | `snd_dma.cpp:148` | 1KB | sfx registration lookup | No |
| `numLoopSounds`/`loopSounds[MAX_LOOP_SOUNDS=32]` | `int`/`loopSound_t[]` | `snd_dma.cpp:186-187` | small | `S_AddLoopingSound` :1912 | No |
| `s_rawend`/`s_rawsamples[MAX_RAW_SAMPLES]` | `int`/`portable_samplepair_t[]` | `snd_dma.cpp:190-191` | raw PCM stream buffer | music/cinematic raw feed (`S_GetRawSamplePointer` :2090) | No |
| `s_entityPosition[MAX_GENTITIES]`/`s_entityWavVol[]`/`s_entityWavVol_back[]` | `vec3_t[]`/`int[]`×2 | `snd_dma.cpp:192-194` | entity-count arrays | `S_UpdateEntityPosition` :2284, `S_Update` :2700 | No — positions pushed in via syscall |
| OpenAL/EAX block (`s_UseOpenAL`, `listener_pos/ori`, `s_numChannels`, `s_bEAX`, `s_bInWater`, `s_EnvironmentID`, `s_lpEAXManager`, …) | mixed | `snd_dma.cpp:213-268` | moderate | EAX/OpenAL init + environment update in same file | No — MP-only feature block |
| dynamic music: `tMusic_Info[]`,`bMusic_IsDynamic`,`eMusic_StateActual/Request`,`sMusic_BackgroundLoop`,… | `static` mixed | `snd_dma.cpp:104-109` | small | `S_SetDynamicMusicState` :100 + music update | No |

### 1k. VM layer (`oracle/codemp/qcommon/vm.cpp`)

| Name | Type | File:Line | Size | Mutators | Cross-boundary? |
|---|---|---|---|---|---|
| `vmTable[MAX_VM=3]` | `vm_t[3]` (`struct vm_s`, `vm_local.h:111-146`, ~19 fields: ASM-fixed-offset `programStack`/`systemCall` header, `dllHandle`/`entryPoint`, `codeBase`/`dataBase`/`dataMask` sandbox, symbols, debug counters) | `oracle/codemp/qcommon/vm.cpp:28-29` | 3 × ~19-field struct | `VM_Init` `vm.cpp:60`, `VM_Create` :471 (slot claim :493-503), `VM_Free` :605, `VM_Clear` :628, `VM_Restart` :391-458 | **No** — zero hits outside `qcommon/`. What crosses the boundary is opaque `vm_t*` handles + `VM_Call` |
| `currentVM` | `vm_t*` | `vm.cpp:24`, extern `vm_local.h:169` | ptr | `VM_Call` save/restore :799-800,826-827 (Chain B, §3); nulled in `VM_Free`/`VM_Clear`; consumed by `VM_DllSyscall` :377-379, `VM_ArgPtr` :640-654 | No |
| `lastVM` | `vm_t*` | `vm.cpp:25` | ptr | same as currentVM; read by `VM_VmProfile_f` :860-864 | No (debug only) |
| `vm_debugLevel` | `int` | `vm.cpp:26`, extern `vm_local.h:170` | 4B | `VM_Debug` :41-43 | No |
| `gvm` (game handle) | `vm_t*` | def `server/sv_main.cpp:12`, extern `server.h:234` | ptr | `SV_InitGameProgs`/`SV_ShutdownGameProgs` `sv_game.cpp:1672,1715,1750` | Yes — server-owned handle into qcommon's VM layer; `VM_Call(gvm, GAME_INIT, …)` `sv_game.cpp:1690` |
| `cgvm`/`uivm` (cgame/ui handles) | `vm_t*`×2 | def `client/cl_main.cpp:108` and `client/cl_ui.cpp:28`, extern `client.h:386-387` | ptr each | `cl_cgame.cpp:603,1771` (`VM_Create("cgame")`), `cl_ui.cpp:1453,1478` | Yes — client-owned handles; `VM_Call(cgvm, CG_DRAW_ACTIVE_FRAME, …)` `cl_cgame.cpp:1840-1842`, `VM_Call(uivm, UI_INIT, …)` `cl_ui.cpp:1484-1518` |

**Takeaway (MP):** VM internals are private to `vm.cpp`; the cross-module
contract is exactly "per-module handle + numbered call" — the natural Rust
shape is `Engine` holding `Option<VmHandle>` per module with calls threaded
explicitly. Consistent with the `Args`/`Output` typed-call model already
proposed in `docs/engine-plan.md` (§5).

**SP has no VM at all** (`oracle/code/qcommon/` has no
`vm.cpp`/`vm_local.h`). Three different mechanisms, one per module:
1. **jagame** — classic `GetGameAPI` import/export structs (§4.2), no VM:
   `ge = (game_export_t *)Sys_GetGameAPI(&import);`
   `oracle/code/server/sv_game.cpp:669`, then `ge->Init(...)` :690.
2. **cgame** — vestigial VM shell: SP's `vm_t` is gutted to one field,
   `struct vm_s { int (*entryPoint)(int callNum, ...); }`
   (`oracle/code/client/vmachine.h:48-52`); single global `vm_t cgvm;`
   (`cl_cgame.cpp:24`); `VM_Call(callnum, ...)`
   (`vmachine.cpp:12-24`) takes no vm parameter, always calls
   `cgvm.entryPoint`. Wiring is a direct function-pointer assignment to the
   statically-linked `vmMain` (`cgame/cg_main.cpp:94-115`):
   `*entryPoint = (int(*)(int,...))vmMain;` at
   `oracle/code/win32/win_main_console.cpp:564` (mac:
   `mac_main.c:70`). No `LoadLibrary` in the shipping path.
3. **ui** — no indirection at all: direct calls
   `UI_Init(UI_API_VERSION, &uii, ...)` `oracle/code/client/cl_ui.cpp:297`,
   `UI_ConsoleCommand()` :316, `UI_SetActiveMenu(...)` :325,338. The `uivm`
   global (`cl_ui.cpp:362`) is dead.

**Reintroduced-globals hazard**: because SP's cgame/ui are compiled into the
engine binary, their formerly VM-private state is now plain process-wide
globals in the same static segment as `sv`/`svs`/`cl`: `cg_t cg; cgs_t cgs;
centity_t cg_entities[MAX_GENTITIES];`
(`oracle/code/cgame/cg_main.cpp:210-212`, extern
`cg_local.h:542-543`) and `uiInfo_t uiInfo;`
(`oracle/code/ui/ui_main.cpp:315`, extern `ui_local.h:172`). MP
declares the identical globals (`codemp/cgame/cg_main.c:691`,
`codemp/ui/ui_main.c:875`, §2.2-2.3) but they live inside `vm_t.dataBase`
(QVM) or a separate DLL image — the difference is isolation, not presence.
**The Rust port should not inherit SP's regression**: SP cgame/ui state
should be an owned struct threaded in, per porting-rules §B, even though
oracle's SP happens to make it a plain global.

### 1l. Renderer (headline only — port deferred, DEC-01)

| Name | Type | Declared | Defined | Size |
|---|---|---|---|---|
| `tr` | `trGlobals_t` | `oracle/codemp/renderer/tr_local.h:1434` | `oracle/codemp/renderer/tr_main.cpp:15` | Large, ~50 fields: frame/scene/view counters, `world_t*`, model/image/shader/skin pointer tables, `viewParms_t`, `trRefdef_t`, trig tables, `world_t bspModels[MAX_SUB_BSP]`, RMG `srfTerrain_t landScape`. "Most renderer globals are defined here." |
| `backEnd` | `backEndState_t` | `tr_local.h:1433` | `oracle/codemp/renderer/tr_backend.cpp:21` | Medium: `refdef`/`viewParms`/`ori` copies, perf counters, render-mode flags, `currentEntity` — deliberately separated from front-end `tr` state |
| `glConfig` | `glconfig_t` | `tr_local.h:1435` (type in shared `cgame/tr_types.h:298-325`) | `oracle/codemp/renderer/tr_init.cpp:33` | Small flat struct, effectively write-once at GL init; kept outside `tr` "so it shouldn't be cleared during ref re-init" |

Other file-scope `static` declarations across
`oracle/codemp/renderer/*.cpp`: **292 total** (mix of static data/caches
and static helper functions). Top files: `tr_shader.cpp` (32), `tr_world.cpp`
(28), `tr_surface.cpp` (25), `tr_font.cpp` (25), `tr_image.cpp` (21),
`tr_ghoul2.cpp` (21), `tr_shade.cpp` (19), `tr_sky.cpp` (15). Full census
deliberately out of scope per DEC-01.

### 1m. SP deltas (`oracle/code/`)

- **common.cpp** — same core globals; `com_pushedEventsHead/Tail` are
  non-`static` (`code/qcommon/common.cpp:691`). Drops MP-only cvars
  (`com_dedicated`, `com_dropsim`, `com_timedemo`, `com_RMG`, `com_blood`,
  `com_cameraMode`, …); adds SP-only mutable state: `speedslog`/`camerafile`
  file handles, `timeInTrace`/`timeInPVSCheck`/`numTraces` profiling
  counters, `com_skippingcin`/`com_speedslog` cvars, cinematic-camera
  statics `corg`/`cangles`/`bComma`.
- **cmd.cpp** — single file (no common/pc split); cbuf is `msg_t cmd_text`
  over `cmd_text_buf`; **`cmd_functions` is a fixed array
  `cmd_function_t[CMD_MAX_NUM=256]`** — matches MP's *dead*
  `cmd_console.cpp` variant, not MP-canonical `cmd_pc.cpp`'s heap linked
  list. A real cross-branch implementation divergence to preserve knowingly.
- **cvar.cpp** — identical globals, same names/shapes.
- **files** — same shape, narrower: no `fs_basegame`/`fs_homepath`/
  `fs_dirbeforepak` cvars; **no pure-server pak-validation globals at all**
  (`fs_serverPaks*`, `fs_serverReferencedPaks*`, `lastValidBase/Game`,
  `fs_fakeChkSum`, `fs_checksumFeed`, `fs_loadStack` absent) — no pure-pak
  handshake, single-player has nothing to validate against.
- **z_memman_pc.cpp** — identical (`TheZone`, `hunk_tag`,
  `gbMemFreeupOccured`).
- **net_chan.cpp** — same, minus `net_killdroppedfragments`.
- **cm_load.cpp** — same core, plus two SP-only globals:
  `CM_OrOfAllContentsFlagsInMap` (`code/qcommon/cm_load.cpp:50`) and
  `gsCachedMapDiskImage[MAX_QPATH]` (`:581` — SP checks a cached-map name
  string, MP only checks pointer identity).
- **server** — same `sv`/`svs` names (def `code/server/sv_main.cpp:18-19`)
  but leaner: `server_t` drops `restarting`/`restartedServerId`/
  `checksumFeed` and all VM-registration fields (`gentities`/`gameClients`/
  `mSharedMemory` — statically-linked game needs no handoff, consistent with
  §4.2); `serverStatic_t` drops the entire `challenges[1024]` table and
  `redirectAddress`/`authorizeAddress`/`snapFlagServerBit`; `client_t` drops
  chunked-download and challenge/rate-limit fields. SP **adds save-game
  global state**: `sv_savegame.cpp` (whole file, no MP counterpart) plus
  `qboolean qbLoadTransition` and `SavedGameJustLoaded_e
  eSavedGameJustLoaded` (extern'd `server.h`, defined `sv_ccmds.cpp:22`/
  `sv_client.cpp`, mutated `sv_ccmds.cpp:288,311`, consumed
  `sv_client.cpp:483-500`, `sv_game.cpp:690`).
- **client** — same three globals `cl`/`clc`/`cls`
  (`code/client/client.h:112,149,262`, def `cl_main.cpp:82-84`) but much
  leaner: `clientStatic_t` drops all server-browser lists and master/auth
  fields; `clientConnection_t` drops the entire download subsystem, demo
  record/playback, and RMG heightmaps; `clientActive_t` drops
  `entityBaselines`/`vps` (vehicle playerstate) and adds one dead SP field
  `cinematictime` (`client.h:99`, kept for struct-size parity). No new
  client-struct save-game fields.
- **sound** — near-verbatim fork: same global names/order
  (`code/client/snd_dma.cpp:125-188`); same flat file-scope-global
  architecture; minor structural difference in channel-picking
  (`S_PickChannel` :1081 / `S_OpenALPickChannel` :1156 both plainly defined
  vs MP's forward-declared EAX wiring). Needs the same restructuring
  treatment as MP.
- **renderer** — no rd-common/rd-vanilla split; SP renderer is a flat
  `code/renderer/` mirroring MP, same three globals
  (`code/renderer/tr_local.h:1257-1259`; `tr` `tr_main.cpp:16`, `backEnd`
  `tr_backend.cpp:17`, `glConfig` `tr_init.cpp:15`), statically linked into
  the SP binary.

### 1n. Cross-cutting observations (engine tier)

1. **Dirtiest cross-boundary offenders**: `sv`/`svs` (reached from
   `icarus/`, `qcommon/msg.cpp`, `qcommon/RoffSystem.cpp`, `RMG/`), `cls`
   (reached from `qcommon/files` for pure-pak gating and win32/unix input
   code), `cmg` + the map-disk-image cache pair
   (`gpvCachedMapDiskImage`/`gbUsingCachedMapDataRightNow`, shared
   CM↔renderer), and the `com_*`/`cvar_modifiedFlags` cvar plumbing (touched
   from nearly every tier).
2. **Cleanly encapsulated already**: the VM table (qcommon-private,
   handle-based contract), `cmd_functions`/`cvar_vars` (API-only access),
   the entire sound mixer (syscall-fed), and `kg` (keys already pre-bundled
   into one struct by Raven — a template for what a Rust `KeyState` should
   look like).
3. **Known oracle quirks to preserve/flag**: the `files_pc.cpp:2328-2330`
   static shadowing of the referenced-paks globals; the shared
   `Cmd_TokenizeString` scratch buffers (`cmd_common.cpp:290-292`) as a
   reentrancy hazard in their own right (independent of Chain A); the
   `sv_worldSectors` intrusive-list pointer surgery in
   `SV_LinkEntity`/`SV_UnlinkEntity` (`sv_world.cpp:151,189`, Chain A, §3);
   and SP's static-linking of cgame/ui reintroducing process-wide `cg`/
   `cgs`/`uiInfo` globals that MP's VM/DLL model isolated (§1k).

---

## 2. Module-side globals

### 2.1 MP game module (`oracle/codemp/game/`)

| Global | Type | Decl (extern) | Def | Size/shape | Mutators | Cross-file ref |
|---|---|---|---|---|---|---|
| `level` | `level_locals_t` | `g_local.h:1508` | `g_main.c:9` | struct at `g_local.h:810-930` (~120 lines, ~78 members: `sortedClients[MAX_CLIENTS]`, `num_entities`, `gentities`/`clients` pointers, timers, intermission/vote state) | mutated throughout nearly every `g_*.c` file via direct field access | Singleton ambient state; every game-module TU includes `g_local.h` and touches `level.*` directly, no accessor indirection |
| `g_entities` | `gentity_t[MAX_GENTITIES]` | `g_local.h:1509` | `g_main.c:27` | `MAX_GENTITIES = (1024+(MAX_CLIENTS-1))` (`q_shared.h:2004`; `MAX_CLIENTS=32`, `q_shared.h:1985`; dead alt-def `q_shared.h:1996`, `1<<GENTITYNUM_BITS`). `gentity_t` (`struct gentity_s`, `g_local.h:133-360`, ~227 lines/~130 members) embeds `entityState_t s` + `entityShared_t r` (the ABI-shared prefix) plus private game fields | `G_InitGame` memsets it (`g_main.c:978`); individual slots mutated by spawn/free logic across `g_*.c` | Singleton array, raw-indexed everywhere (`g_entities[i]`) |
| `g_clients` (reached as `level.clients`) | `gclient_t[MAX_CLIENTS]` | not separately `extern`'d — only `level.clients` pointer used externally | `g_main.c:28` | `gclient_t` (`struct gclient_s`, `g_local.h:536-748`, ~212 lines). **Layout comment at `g_local.h:537`: `// ps MUST be the first element, because the server expects it`** — `playerState_t ps` is field 0 | `G_InitGame` memsets and wires `level.clients = g_clients` (`g_main.c:983-984`); per-client fields mutated in `g_client.c` etc. | Reached only via `level.clients[i]`; effectively private to `g_main.c`, aliased through `level` |

**`trap_LocateGameData` call site** — `g_main.c:997-998` (inside `G_InitGame`, once per level start; re-issued after entity-count changes at `g_utils.c:848`):

```c
trap_LocateGameData( level.gentities, level.num_entities, sizeof( gentity_t ),
    &level.clients[0].ps, sizeof( level.clients[0] ) );
```

Prototype `g_local.h:1697`; syscall wrapper `g_syscalls.c:105-107` marshals to
`syscall(G_LOCATE_GAME_DATA, gEnts, numGEntities, sizeofGEntity_t, clients,
sizeofGClient)`. Note it passes `&level.clients[0].ps` — the address of the
**`playerState_t` sub-field**, not the client array base — plus
`sizeof(gclient_t)` as the stride: the server only ever reads the leading
`playerState_t` of each client slot, at `gclient_t`-sized strides.

### 2.2 MP cgame module (`oracle/codemp/cgame/`)

| Global | Type | Decl | Def | Size/shape |
|---|---|---|---|---|
| `cg` | `cg_t` | `cg_local.h:1626` | `cg_main.c:691` | `cg_local.h:755-1014` (~260 lines, ~157 members) — per-frame/session state (clientNum, snapshots, viewangles, …) |
| `cgs` | `cgs_t` | `cg_local.h:1625` | `cg_main.c:692` | `cg_local.h:1067-1609` (~540 lines, ~367 members) — gamestate-derived config (media handles, client infos, map info) |
| `cg_entities` | `centity_t[MAX_GENTITIES]` | `cg_local.h:1627` | `cg_main.c:693` | `centity_t` (`cg_local.h:333-462`, ~130 lines/~79 members) — wraps `entityState_t` + interpolation/lerp state |

All three are genuine file-scope singletons (classic idTech3 cgame pattern):
every cgame TU includes `cg_local.h` and touches them directly.

### 2.3 MP ui module (`oracle/codemp/ui/`, shared widget engine in `oracle/ui/`)

| Global | Type | Decl | Def | Notes |
|---|---|---|---|---|
| `uiInfo` | `uiInfo_t` | `ui_local.h:843` | `ui_main.c:875` | `ui_local.h:729-841` (~112 lines) — menu/UI transient state |
| `DC` | `displayContextDef_t *` | `oracle/ui/ui_shared.h` | `oracle/ui/ui_shared.c:103` | Pointer to the import-function-table struct that all of `ui_shared.c`'s reusable widget code calls through — the mechanism letting the **same** `ui_shared.c` compile into both the MP `ui` module and (statically) SP |

`oracle/ui/` (top-level, no `codemp`/`code` prefix) holds the shared
widget/menu engine compiled into both MP and SP UI; `oracle/codemp/ui/`
and `oracle/code/ui/` hold the per-game specialization.

### 2.4 SP game module (`oracle/code/game/`)

| Global | Type | Decl | Def | Notes |
|---|---|---|---|---|
| `level` | `level_locals_t` | `g_local.h:222` | `g_main.cpp:46` | Same pattern as MP; SP-specific fields (e.g. `mBSPInstanceDepth`, `mOriginAdjust` for RMG/terrain) shown just above the extern at `g_local.h:215-220` |
| `g_entities` | `gentity_t[MAX_GENTITIES]` | (same pattern) | `g_main.cpp:49` | `MAX_GENTITIES = 1<<GENTITYNUM_BITS` (`code/game/q_shared.h:1451`) |
| `g_clients` | **does not exist** | — | — | SP has no free-standing `g_clients` array; `MAX_CLIENTS = 1` (`code/game/q_shared.h:1447`, note commented-out `// 128`) — single-player, no multi-client array |
| `globals` | `game_export_t` | `g_local.h:223` | `g_main.cpp:48` | SP's entire analogue of `trap_LocateGameData` — see §4.2 |

SP has **zero** `LocateGameData`/`G_LOCATE_GAME_DATA` call sites (confirmed by
grep across `oracle/code/game/`) — the single most important MP/SP
divergence for this doc, detailed in §4.2.

### 2.5 SP cgame/ui (`oracle/code/cgame/`, `oracle/code/ui/`)

Same `cg`/`cgs`/`cg_entities` and `uiInfo`/`DC` pattern, declared in
`code/cgame/cg_local.h` / `code/ui/ui_local.h`, defined in
`code/cgame/cg_main.cpp` / `code/ui/ui_main.cpp`. Structurally parallel to MP,
but the **calling convention underneath differs completely** — see §4.3.

---

## 3. Reentrancy traces

### Chain A — `SV_Frame` → game module → back into server world-linking state

1. **`SV_Frame`** — `oracle/codemp/server/sv_main.cpp:826`. Frame loop,
   `sv_main.cpp:909-915`:
   ```c
   while ( sv.timeResidual >= frameMsec ) {
       sv.timeResidual -= frameMsec;
       svs.time += frameMsec;
       VM_Call( gvm, GAME_RUN_FRAME, svs.time );   // sv_main.cpp:914
   }
   ```
2. **`VM_Call(gvm, GAME_RUN_FRAME, ...)`** dispatches into `G_RunFrame`,
   `oracle/codemp/game/g_main.c:3582`. The outer entity loop holds a
   live raw pointer across nested calls, `g_main.c:3741-3742`:
   ```c
   ent = &g_entities[0];
   for (i=0 ; i<level.num_entities ; i++, ent++) {
       ...
       if ( ent->s.eType == ET_MOVER ) { G_RunMover( ent ); continue; }  // g_main.c:3813-3814
   ```
   `G_RunMover` (`g_mover.c`) calls `trap_LinkEntity` on *other* pushed
   entities mid-iteration (`g_mover.c:402`, `g_mover.c:449`). A second hazard:
   `ClientThink_real` (`g_active.c:1939`) calls `trap_LinkEntity(ent)` at
   `g_active.c:3470` right after processing movement, reached via the
   `GAME_CLIENT_THINK` VM entry (`sv_client.cpp:1659`, `SV_ClientThink`) or
   during `G_RunFrame`'s per-client pass. Wrapper declared
   `g_local.h:1714`.
3. **Game→engine syscall dispatch**: `SV_GameSystemCalls`,
   `oracle/codemp/server/sv_game.cpp:458`, case `G_LINKENTITY` at
   `sv_game.cpp:575-577`:
   ```c
   case G_LINKENTITY:
       SV_LinkEntity( (sharedEntity_t *)VMA(1) );
       return 0;
   ```
4. **`SV_LinkEntity`** — `oracle/codemp/server/sv_world.cpp:189`.
   Resolves the server-side shadow struct via `SV_SvEntityForGentity`
   (`sv_world.cpp:200`, impl `sv_game.cpp:70-75`), then mutates the
   **world-sector linked list** — a structure entirely separate from
   `g_entities`, `sv_world.cpp:342-344`:
   ```c
   ent->worldSector = node;
   ent->nextEntityInWorldSector = node->entities;
   node->entities = ent;
   ```
   (`SV_UnlinkEntity`, `sv_world.cpp:151`, does the matching splice.)
5. **The aliasing hazard**: `SV_AreaEntities_r` (`sv_world.cpp:373-414`)
   walks that same `node->entities` list, defensively pre-fetching
   `next = check->nextEntityInWorldSector` at `sv_world.cpp:381` *before*
   evaluating `check` — a tell Raven's authors already knew the list could be
   mutated out from under a walk. `SV_AreaEntities` is itself reachable as a
   trap (`G_ENTITIES_IN_BOX`, `sv_game.cpp:581-582`) that game code can call
   **while already inside** `G_RunFrame`/`ClientThink_real` (e.g.
   `G_TouchTriggers`, radius-damage scans).

   Full picture: `SV_Frame` (holding `svs.time`/`sv.timeResidual`) →
   `G_RunFrame`'s `for (...ent++)` loop (holding a raw `gentity_t*` across the
   iteration) → `G_RunMover`/`ClientThink_real` → `trap_LinkEntity` →
   `SV_LinkEntity` splices the global world-sector tree — a structure other
   code reachable from the *same* nested call graph
   (`G_TouchTriggers` → `trap_EntitiesInBox` → `SV_AreaEntities_r`) is
   simultaneously walking.

**Why this matters for Rust**: none of `SV_Frame`, `G_RunFrame`'s entity
loop, or `SV_AreaEntities_r` can compile as safe Rust if `g_entities`,
`sv_worldSectors`, and `sv.svEntities` are fields of one struct borrowed
`&mut` at the outer scope — the reentrant `trap_LinkEntity`/
`trap_EntitiesInBox` calls need `&mut` access to a *different* field of that
same struct while the outer borrow is live. C survives this only via raw
pointers + single-threaded sequencing; Rust's borrow checker cannot see
across the indirect `VM_Call`/trait-object boundary that the outer loop's
borrow (of `g_entities`) and the inner call's borrow (of `worldSectors`) are
disjoint fields, because the call is indirect, not a sibling-field access in
one function body. Candidates: split-borrow accessors passed explicitly into
the trap layer, interior mutability (`RefCell`/`Cell`) at the seam, or
queued effects applied after the outer borrow ends.

### Chain B — `Com_Error` unwinding through a reentrant VM call

1. **`Com_Error`** — `oracle/codemp/qcommon/common.cpp:249`. **Not**
   `setjmp`/`longjmp` in this tree (no `jmp_buf`/`abortframe` exists in
   codemp's `common.cpp`) — it uses **C++ exceptions**:
   - `ERR_SERVERDISCONNECT` (`common.cpp:302-312`): `CL_Disconnect` +
     `CL_FlushMemory`, then `throw ("DISCONNECTED\n");` (line 312).
   - `ERR_DROP`/`ERR_DISCONNECT` (`common.cpp:313-326`): prints, calls
     **`SV_Shutdown(...)`** (line 315), `CL_Disconnect`/`CL_FlushMemory`
     (316-317), then `throw ("DROPPED\n");` (line 326).
   - `ERR_NEED_CD` (`common.cpp:327-336`): `SV_Shutdown`, then
     `throw ("NEED CD\n");`.
   - default/`ERR_FATAL` (`common.cpp:337-344`): `CL_Shutdown()`,
     `SV_Shutdown(...)`, `Com_Shutdown()`, then `Sys_Error(...)` — terminates
     the process directly, no throw.
   - Recursive-error guard (`common.cpp:288-291`): if `com_errorEntered` is
     already set, calls `Sys_Error("recursive error after: %s", ...)` instead
     of throwing again.
   - Vestigial comment at `common.cpp:1612` (*"might be clobbered by
     `longjmp' or `vfork'"*) is leftover from the original id Software
     `setjmp`/`longjmp` `Com_Frame`, confirming a conversion to
     `throw`/`catch` happened without updating the comment.
2. **The catch site**: `Com_Frame`, `oracle/codemp/qcommon/common.cpp:1593`,
   wraps the per-frame body (event loop, `SV_Frame` at line 1669, client
   frame, …) in `try { ... }` (opens 1595) with
   `catch (const char* reason) { Com_Printf(reason); return; }` at
   `common.cpp:1761-1765`. A second `catch` guards `Com_Init`
   (`common.cpp:1439`).
3. **Concrete deep-nested throw site**: `SV_SvEntityForGentity`,
   `oracle/codemp/server/sv_game.cpp:70-75`:
   ```c
   svEntity_t *SV_SvEntityForGentity( sharedEntity_t *gEnt ) {
       if ( !gEnt || gEnt->s.number < 0 || gEnt->s.number >= MAX_GENTITIES ) {
           Com_Error( ERR_DROP, "SV_SvEntityForGentity: bad gEnt" );   // sv_game.cpp:72
       }
       return &sv.svEntities[ gEnt->s.number ];
   }
   ```
   Called directly from **`SV_LinkEntity`** (`sv_world.cpp:200`) — i.e. the
   exact reentrant call from Chain A can throw `ERR_DROP` from several native
   stack frames deep inside a live `VM_Call(gvm, GAME_RUN_FRAME, ...)`.
4. **What breaks on unwind**: `VM_Call`,
   `oracle/codemp/qcommon/vm.cpp:787-829`, does a manual (non-RAII)
   global save/restore:
   ```c
   oldVM = currentVM;                              // vm.cpp:799
   currentVM = vm;                                 // vm.cpp:800
   ... r = VM_CallInterpreted(vm, &callnum); ...   // vm.cpp:823 — may throw
   if ( oldVM != NULL ) currentVM = oldVM;         // vm.cpp:826-827 — SKIPPED on unwind
   ```
   `currentVM` is a bare global (`vm.cpp:24`), not scoped by a guard object,
   so line 827 never runs on the throw path. The damage is partially masked
   because `Com_Error(ERR_DROP, ...)` calls `SV_Shutdown` **before**
   throwing (`common.cpp:315`), and `SV_Shutdown` → `SV_ShutdownGameProgs`
   (`sv_game.cpp:1666`) does:
   ```c
   VM_Call( gvm, GAME_SHUTDOWN, qfalse );   // sv_game.cpp:1670 — a NEW reentrant call into the SAME vm
   VM_Free( gvm );                          // sv_game.cpp:1671 — zeroes vm_t (vm.cpp:622), currentVM=NULL (vm.cpp:624)
   gvm = NULL;                              // sv_game.cpp:1672
   ```
   So *before* the `throw` at `common.cpp:326` executes, the engine
   synchronously re-enters the same game module (`GAME_SHUTDOWN`) while the
   original `G_RunFrame → trap_LinkEntity → SV_LinkEntity →
   SV_SvEntityForGentity` native frames are still live/suspended on the C++
   stack, then frees/zeroes the `vm_t` those suspended frames' `vm`/
   `currentVM` pointers refer to. Only then does the `throw` unwind past
   those frames, past `VM_Call`'s un-run restore line, up to `Com_Frame`'s
   `catch`. Every intervening `VM_Call` invocation's `oldVM` local is
   discarded — harmless only because the whole VM was torn down anyway, not
   because anything guaranteed correctness.

**Why this matters for Rust**: Raven's C++ code "solves" exception safety not
with scoped guards but with a sledgehammer — any `ERR_DROP` from anywhere
nukes the entire server/game VM synchronously, *inside* `Com_Error`, before
unwinding even starts, so there's no partially-consistent `sv`/`level`/`gvm`
state left to worry about (everything reachable is zeroed or discarded). A
Rust `panic!`/unwind translation needs the same contract made explicit: (a)
accept "abort the whole subsystem, no partial recovery" — an `Err`/panic that
forces a hard teardown of the owning struct, not a value bubbled up for
graceful per-field recovery — or (b) if finer-grained recovery is wanted,
every borrow/lock/`RefCell` guard held across a reentrant call boundary must
be `Drop`-safe under unwind (no `mem::forget`, no manual save/restore of
shared state without a guard type), because Rust unwinding runs destructors
and nothing else — exactly like the swallowed `currentVM` restore here.
(Cross-ref: `docs/decisions.md` DEC-08 already settles the headline choice —
panic + `catch_unwind` at the frame boundary — this chain is the ground truth
for *why*, and for the "no partial recovery" corollary DEC-08 doesn't spell
out.)

### Chain C — cgame draw calling back into the renderer (designed-in accumulation)

1. **`CL_CGameRendering`** — `oracle/codemp/client/cl_cgame.cpp:1830`:
   ```c
   VM_Call( cgvm, CG_DRAW_ACTIVE_FRAME, cl.serverTime, stereo, clc.demoplaying );  // cl_cgame.cpp:1842
   ```
   into `CG_DrawActiveFrame`, `oracle/codemp/cgame/cg_view.c:2447`.
2. **cgame-side reentrant trap calls**: cgame invokes `trap_R_AddRefEntityToScene`
   dozens of times across `cg_ents.c` (e.g. `cg_ents.c:672,677,795,1475,1583,
   1634,2011,2225`) via `CG_AddPacketEntities` (called `cg_view.c:1913` and
   `:2677`), then `trap_R_RenderScene(&cg.refdef)` once at `cg_view.c:1932`
   (portal/skybox scene) and `cg_view.c:2424` (main scene).
3. **Engine-side dispatch**: `CL_CgameSystemCalls`,
   `oracle/codemp/client/cl_cgame.cpp:644`:
   - `CG_R_ADDREFENTITYTOSCENE` → `re.AddRefEntityToScene(...)`
     (`cl_cgame.cpp:894-896`) → `RE_AddRefEntityToScene`,
     `oracle/codemp/renderer/tr_scene.cpp:194`.
   - `CG_R_RENDERSCENE` → `re.RenderScene(...)` (`cl_cgame.cpp:922-924`) →
     `RE_RenderScene`, `tr_scene.cpp:706`.
   Both touch the renderer's `tr`/`backEndData` globals (full census
   out of scope here — renderer port is deferred per DEC-01).
4. **The contrast with Chain A**: `RE_AddRefEntityToScene` **accumulates**
   into a module-static counter + shared backing array on every call:
   ```c
   backEndData->entities[r_numentities].e = *ent;                    // tr_scene.cpp:224
   backEndData->entities[r_numentities].lightingCalculated = qfalse; // tr_scene.cpp:225
   ...
   r_numentities++;                                                  // tr_scene.cpp:254
   ```
   `r_numentities` declared `tr_scene.cpp:24` (`static int r_numentities;`),
   reset once per scene (`tr_scene.cpp:55`/`:859`). `RE_RenderScene`
   snapshots the accumulated count into `tr.refdef` at `tr_scene.cpp:796`:
   `tr.refdef.num_entities = r_numentities - r_firstSceneEntity;`. Dozens of
   reentrant trap calls from a single outer `VM_Call(cgvm,
   CG_DRAW_ACTIVE_FRAME, ...)` each mutate a small, well-defined slice of
   shared renderer state (append/bump counter) before one terminal call
   consumes the result — a builder/accumulator pattern, not an aliasing
   hazard. `CL_CGameRendering` holds no live reference into
   `backEndData->entities[]` while this happens — it's a single opaque
   `VM_Call` with no state of its own visible across the boundary.

**Why this matters for Rust**: this is the easy case, a useful foil for Chain
A. Because the outer caller holds nothing live across the `VM_Call`, the
natural Rust shape is an explicit scene-builder object (`SceneBuilder` with
`add_ref_entity`/`render`) owned by the render subsystem and reached via
`&mut` through the syscall dispatch, with no other borrow alive
concurrently. This validates that not every trap-reentrancy case needs the
heavy machinery (split borrows / message queues) Chain A requires — only
chains where an *outer* loop/handler demonstrably holds a live
reference/borrow into the same state the reentrant call needs to mutate
require that; pure accumulate-then-flush patterns can just use one
exclusively-owned builder for the outer call's duration.

---

## 4. Shared memory

### 4.1 MP: syscall-based `G_LOCATE_GAME_DATA`

- **Enum/doc**: `oracle/codemp/game/g_public.h:145-148`:
  ```c
  G_LOCATE_GAME_DATA,   // ( gentity_t *gEnts, int numGEntities, int sizeofGEntity_t,
                         //   playerState_t *clients, int sizeofGameClient );
  // the game needs to let the server system know where and how big the gentities
  // are, so it can look at them directly without going through an interface
  ```
- **Engine-side receiver**: `SV_LocateGameData`,
  `oracle/codemp/server/sv_game.cpp:327-335`:
  ```c
  void SV_LocateGameData( sharedEntity_t *gEnts, int numGEntities, int sizeofGEntity_t,
                         playerState_t *clients, int sizeofGameClient ) {
      sv.gentities = gEnts;
      sv.gentitySize = sizeofGEntity_t;
      sv.num_entities = numGEntities;
      sv.gameClients = clients;
      sv.gameClientSize = sizeofGameClient;
  }
  ```
  Dispatched from the syscall switch, `sv_game.cpp:566-567`:
  `case G_LOCATE_GAME_DATA: SV_LocateGameData((sharedEntity_t*)VMA(1), args[2], args[3], (struct playerState_s*)VMA(4), args[5]);`
- **Storage**: the `server_t sv` global (`oracle/codemp/server/server.h:73-78`),
  fields `sharedEntity_t *gentities`, `int gentitySize`, `int num_entities`,
  `playerState_t *gameClients`, `int gameClientSize`. Comment above
  (`server.h:72`): `// the game virtual machine will update these on init and
  changes`. The engine also owns a **parallel, server-private** array
  `svEntity_t svEntities[MAX_GENTITIES]` (`server.h:68`) — *not* shared with
  the game module, holds server-only bookkeeping (world-sector linkage,
  area). `sharedEntity_t` (`g_public.h:715`, comment history
  `g_public.h:700-714`) is the type-punned view the engine uses onto the
  game's real `gentity_t` — layout-compatible only for the leading fields.
- **Downstream dereferences** proving genuine pointer aliasing into
  game-owned memory:
  - `SV_NumForGentity`/`SV_GentityNum`/`SV_GameClientNum`,
    `sv_game.cpp:44-68` — explicit comment: *"these functions must be used
    instead of pointer arithmetic, because the game allocates gentities with
    private information after the server shared part"*. Compute
    `((byte*)ent - (byte*)sv.gentities) / sv.gentitySize` and inverse —
    `sv.gentities` treated as base of a foreign, stride-`sv.gentitySize`
    array.
  - `SV_SvEntityForGentity`, `sv_game.cpp:70-75` — indexes the engine's own
    `sv.svEntities[]` by `gEnt->s.number`, read off the shared struct.
  - `SV_LinkEntity`/`SV_UnlinkEntity`, `oracle/codemp/server/sv_world.cpp:151,189` —
    take `sharedEntity_t *gEnt`, read/write `gEnt->r.bmodel`,
    `gEnt->s.solid`, etc. (`sv_world.cpp:189-206ff`) — engine mutates fields
    inside memory it doesn't own, through the aliased pointer.
  - Iteration bounds: `sv.num_entities` used as a loop bound,
    `sv_init.cpp:213`, `sv_snapshot.cpp:338` (snapshot building).

**Native-DLL vs QVM — and a latent inconsistency.** `VM_Create`
(`oracle/codemp/qcommon/vm.cpp:471+`) supports both `VMI_NATIVE`
(`Sys_LoadDll`, sets `vm->entryPoint`) and `VMI_COMPILED` (QVM bytecode,
`dataBase`/`dataMask`-relative interpreter address space). Game VM created
`sv_game.cpp:1750`: `gvm = VM_Create("jampgame", SV_GameSystemCalls,
(vmInterpret_t)(int)Cvar_VariableValue("vm_game"));` — a QVM game module is in
principle still selectable via `vm_game`. The translation function for
genuine QVM pointers is `VM_ArgPtr`, `oracle/codemp/qcommon/vm.cpp:640-654`:
```c
void *VM_ArgPtr( int intValue ) {
    if ( !intValue ) return NULL;
    if ( currentVM==NULL ) return NULL;
    if ( currentVM->entryPoint ) {
        return (void *)(currentVM->dataBase + intValue);                       // native DLL: dataBase==0 in practice
    } else {
        return (void *)(currentVM->dataBase + (intValue & currentVM->dataMask)); // QVM: masked offset
    }
}
```
**However**, `sv_game.cpp:401` defines `#define VMA(x) ((void *) args[x])` —
a direct pointer cast, **no translation**, used for every `G_*` syscall
including `G_LOCATE_GAME_DATA`. The MP `game` boundary in `sv_game.cpp` is
hard-wired to a real native DLL sharing the engine's address space; it never
calls `VM_ArgPtr`. By contrast, MP's client-side cgame/ui dispatch
(`oracle/codemp/client/cl_cgame.cpp:624`) uses
`#define VMA(x) VM_ArgPtr(args[x])` — properly translating. **For the Rust
port**: model MP `game` as always-native (real shared memory, pointer-cast
ABI per porting-rules §D); the QVM path is a legacy branch `jampgame` doesn't
actually support even in the oracle (cross-ref DEC-05 item 4: classic QVM
interpreter is out of scope anyway, which conveniently sidesteps this
inconsistency rather than needing to faithfully reproduce it).

### 4.2 SP: `GetGameAPI` (classic idTech "hard export" table, no syscall marshaling)

- `Sys_GetGameAPI` (engine side, platform-specific), called
  `oracle/code/server/sv_game.cpp:669`:
  `ge = (game_export_t *)Sys_GetGameAPI(&import);`
- Game module's `GetGameAPI(game_import_t *import)`,
  `oracle/code/game/g_main.cpp:875-905`, populates a struct of real
  function pointers and data pointers directly:
  ```c
  globals.apiversion = GAME_API_VERSION;
  globals.Init = InitGame; ... globals.RunFrame = G_RunFrame; ...
  globals.gentitySize = sizeof(gentity_t);
  ```
  and inside `InitGame` (`g_main.cpp:736,749`):
  `globals.gentities = g_entities; ... globals.num_entities = MAX_CLIENTS;`
- Engine dereferences `ge->gentities`/`ge->gentitySize` exactly like MP's
  `sv.gentities` (`oracle/code/server/sv_game.cpp:46,55`:
  `num = ((byte*)ent - (byte*)ge->gentities) / ge->gentitySize;`).
- Still real pointer aliasing (SP `jagame` is a genuinely separate loadable
  module), but the handshake is one upfront struct-of-pointers return, not an
  ongoing syscall-enum dispatch. **No `sizeofGameClient`/`gameClients`
  equivalent at all** in SP's `game_export_t` — consistent with SP having no
  `g_clients` array (§2.4), since there's exactly one local player.

### 4.3 Other cross-boundary aliasing

**`cl.snap` — copy-based even for native DLLs.** MP client state: `cl.snap`
(`clSnapshot_t`, `oracle/codemp/client/client.h:79`) and
`cl.gameState` (`gameState_t`, `client.h:90`) live in `clientActive_t`; cgame
never touches these directly. Trap: `CG_GETSNAPSHOT` (`cg_public.h:182`) →
`trap_GetSnapshot` (`cg_syscalls.c:473-475`) → engine handler
`CL_GetSnapshot` (`cl_cgame.cpp:157-208`, dispatched `:963-964`). This
**field-by-field copies** out of `cl.snapshots[]`/`cl.parseEntities[]` into
the caller-supplied `snapshot_t *snapshot`: `Com_Memcpy(snapshot->areamask,
...)`, `snapshot->ps = clSnap->ps;`, and a per-entity
`memcpy(&snapshot->entities[i], &cl.parseEntities[entNum],
sizeof(entityState_t))` loop (`cl_cgame.cpp:187-202`). Even though MP cgame
is (in practice) a native DLL sharing the process address space, the
snapshot handoff is a **deliberate copy** — cgame owns its storage,
decoupled from the client's circular snapshot buffer lifetime. SP is
structurally identical: `CL_GetSnapshot`,
`oracle/code/client/cl_cgame.cpp:135-181`
(`snapshot->entities[i] = cl.parseEntities[entNum];`, line 181), same
enum-dispatch shape (`cl_cgame.cpp:759`).

**MP vs SP calling convention: trap syscalls all the way down vs a "fake
VM".** SP cgame/ui still go through the **identical** enum + `int args[]`
marshaling convention as MP (`cgi_GetSnapshot`,
`oracle/code/cgame/cg_syscalls.cpp:454-455`: `return
syscall(CG_GETSNAPSHOT, snapshotNumber, snapshot);` — same shape as MP's
`trap_GetSnapshot`). What differs is what `syscall`/`VMA` resolve to:
- SP's `vm_t` (`oracle/code/client/vmachine.h:47-49`) is reduced to
  just `int (*entryPoint)(int callNum, ...)` — no `dataBase`/`dataMask`,
  because QVM bytecode was never an option for SP.
- SP's `VM_Call` (`oracle/code/client/vmachine.cpp:1-24`) — file
  header literally: **"wrapper to fake virtual machine for client"** —
  forwards varargs straight into `cgvm.entryPoint(...)`, a plain compiled-in
  C function call.
- SP's `VMA` macro (`oracle/code/client/cl_cgame.cpp:429`):
  `#define VMA(x) ((void*)args[x])` — direct cast, never `VM_ArgPtr` — no
  address-space translation ever happens (matches MP's `jampgame`, not MP's
  cgame/ui).
- `CGAME_HARD_LINKED` (`oracle/code/game/q_shared.h:128,144`) guards
  out DLL-only bootstrap shims (`oracle/code/ui/ui_atoms.cpp:305`,
  `#ifndef UI_HARD_LINKED`) — code that would call `ui.Printf`/`ui.Error`
  through an import table is compiled out; the hard-linked build lets
  `Com_Printf`/`Com_Error` resolve directly at link time (same binary, no
  indirection needed).
- SP's legacy `Sys_LoadCgame`
  (`oracle/code/win32/win_main.cpp:557-570`, comment: *"Used to hook
  up a development dll"*) is the only place SP could dlopen a real cgame
  DLL, for hot-reload dev builds — production SP wires `cgvm.entryPoint`
  directly to the statically-linked `vmMain` symbol, no
  `LoadLibrary`/`GetProcAddress`.

**Net for the design doc**: MP's three modules are uniformly reached through
a syscall-enum/args-array trap layer that *could* target either a real
shared-address-space native DLL (in practice, always, for all three modules)
or a masked-offset QVM sandbox (`VM_ArgPtr`, unused by `jampgame`
specifically). SP retains the same trap-enum call shape for cgame/ui purely
for source-compatibility with the shared `ui_shared.c`/cgame codebase, with
zero indirection underneath — a compile-time no-op VM — while `jagame` (the
one real SP module) skips the trap convention entirely for a single
`GetGameAPI` struct-of-pointers handshake. Suggests **one shared-memory
pointer-cast ABI seam type** for MP `game` + SP `jagame`, a **second,
translated-VM-capable seam** for MP `cgame`/`ui` only if QVM support is ever
actually in scope (DEC-05 says it isn't), and SP `cgame`/`ui` can be modeled
as **not crossing a real ABI boundary at all** — ordinary in-process Rust
calls/state, no `unsafe`/layout-asserts needed (matches DEC-07: SP
statically links cgame/ui via the vmachine shim already).

---

## 5. Current Rust precedents

| Precedent | Where | What it shows |
|---|---|---|
| GP2 arena | `crates/mp/engine/qcommon/src/gp2/generic_parser2.rs:1-95` | `GenericParser2 { groups: Vec<GpGroupNode> }` — one owned `Vec` arena, `GpGroupId(u32)` index instead of Raven's `CGPGroup*` parent pointers (doc comment: *"This type owns every group node in an arena ([`GpGroupId`] indices; [`GpGroup`] is a borrow of one node), so Raven's parent pointers need no aliasing"*, line 11-13). Concrete precedent for entities-by-index (porting-rules §B5) already applied to one C++-track subsystem, not yet to `g_entities`. |
| `EntityId` | mentioned only in `docs/porting-rules.md` §B5 (*"Raven's `gentity_t*` become `EntityId(u32)` into an owned arena"*) | **Aspirational, not yet implemented** — `grep -rn "EntityId" crates/` returns zero hits. No arena-of-entities type exists yet in `crates/mp/game` or `crates/sp/game`. |
| `crates/mp/game/src/level/level_locals.rs`, `crates/mp/game/src/entity/*` | ported | Raven's `level_locals_t` and per-entity flag/state types (`mover_state.rs`, `flags.rs`, `hit_location.rs`) are ported as plain structs/enums, but **not yet wired into an owned `level`/`g_entities`-equivalent instance** — no `static mut level` was ever introduced (`grep -rn "static mut" crates/mp/game crates/sp/game` → zero hits), consistent with §B3, but also no replacement owner exists yet. This is greenfield for the design session, not a retrofit. |
| `docs/engine-plan.md` ("Typed Boundary Over Swappable Engine Backends") | already-written design doc | Directly on-topic prior art for engine-tier ownership: proposes **one owned engine instance** ("`RustEngine` — owns engine state; ... Scattered engine-side `static mut`/atomics collapse into a single owned state object reached through one controlled accessor", engine-plan.md lines ~24-26), explicitly scopes **out** game-module globals (`level`, `g_entities`, gclients) as "stay faithful for now — not folded into this work" (engine-plan.md, Principles). Also explicitly dropped a prior "WorldState / multi-world routing + queues" design in favor of "single-instance and synchronous" (engine-plan.md, "Dropped from the prior plan") — i.e. a queued-effects reentrancy model was considered and rejected for the *engine* tier already; whether it should still be considered for the game-module tier (Chain A's problem) is open. |
| `docs/decisions.md` DEC-08 | ledger | Already settles Com_Error recovery as "typed panic payload caught by `catch_unwind` at the frame boundary" + explicit rejection of Result-threading ("reshapes thousands of faithful signatures away from oracle control flow"). Chain B (§3) is the ground truth this decision is resting on. |
| `docs/decisions.md` DEC-05 item 5 | ledger | Module transport is pluggable `NativeDll \| Static \| Wasm`, WASM is first-class from the start, requiring "wasm32 linear-memory pointer translation à la `VM_ArgPtr`, handle-only trap surface, 32-bit in-module layouts" — i.e. the shared-entity-memory design (§4.1) must already account for a non-native-pointer transport, not just the native case oracle uses today. |
| `docs/decisions.md` DEC-07 | ledger | SP cgame/ui: `sp/app` statically links `sp/cgame` + `sp/ui`, Raven's fake-VM shim survives as a thin dispatch layer. Confirms §4.3's SP finding is already the adopted plan, not just an observation. |
| `crates/abi-transport/src/generic/table.rs` | ported | `FunctionTableImport`/`FunctionTableExport` traits already model the two function-table ABI shapes (`GetGameAPI`-style) — the seam-type split recommended in §4.1/§4.3 (pointer-cast native seam vs no-real-boundary SP cgame/ui) has a structural home to land in already. |

No crate currently has anything resembling Raven's `sv`/`svs`/`cls`/`cl`/
`level`/`g_entities` wired up as owned Rust state yet — this dossier's
subsystems are still at the "ported the leaf types, haven't decided the
owning struct" stage across the board. The design session is greenfield for
all of §B except the *principle* (already stated in engine-plan.md and
porting-rules) — no code exists yet that the session would need to unwind or
reconcile with.

---

## Design forks

Already-settled (ledger, not re-litigated — cited so the session doesn't
reopen them):
- **DEC-08**: Com_Error → panic + `catch_unwind` at the frame boundary, no
  Result-threading.
- **DEC-05**: module transport pluggable `NativeDll | Static | Wasm`, WASM
  first-class from the start (classic QVM bytecode explicitly out of scope —
  sidesteps the `jampgame`/`VM_ArgPtr` inconsistency in §4.1).
- **DEC-07**: SP cgame/ui statically linked via the vmachine shim (no real
  ABI boundary to model there — confirmed independently by §4.3).
- engine-plan.md: one owned `RustEngine` instance for the *engine* tier,
  reached through one controlled accessor; game-module globals explicitly
  out of that plan's scope.

Open forks this session must settle:

1. **Mega `Engine` struct vs per-subsystem structs threaded individually.**
   engine-plan.md already commits to "one owned engine instance" for
   qcommon/server/client/renderer collectively — but does "one instance"
   mean one flat struct (`Engine { sv: ServerState, cl: ClientState, cm:
   CollisionModel, ... }`) passed as `&mut Engine` everywhere, or one
   *handle* type that internally holds several independently-borrowable
   sub-structs (so `SV_Frame` can hold `&mut ServerState` while a nested
   call borrows `&mut Renderer` disjointly)? Chain A (§3) is the forcing
   function: if `Engine` is flat, `SV_LinkEntity`'s reentrant mutation of
   `sv.worldSectors` while `G_RunFrame`'s loop borrows `g_entities` needs
   either split-borrow methods (`fn link_entity(&mut self.world, ent)`
   shaped APIs) or the mega-struct doesn't work without `RefCell`.

2. **How server↔game reentrancy (Chain A) is modeled.** Three candidates
   surfaced by the trace: (a) split borrows — the trap layer takes
   `&mut WorldSectors` + `&GEntities` as separate parameters, never
   `&mut Engine`; (b) queued effects — `trap_LinkEntity` during
   `G_RunFrame` enqueues a link operation applied after the frame's entity
   loop completes (changes ordering semantics vs oracle — needs parity
   check against e.g. `G_TouchTriggers` calling `EntitiesInBox` and
   expecting to see already-linked-this-frame entities); (c) interior
   mutability at the seam (`RefCell`/`Cell` wrapping `worldSectors`) —
   cheapest to implement, defers the aliasing question to runtime borrow
   panics, arguably against porting-rules §B's spirit but not explicitly
   forbidden. Note engine-plan.md already rejected a *queue-based* design
   for the engine tier generally ("Dropped from the prior plan") — does that
   rejection extend to this specific game-module reentrancy problem, or was
   it scoped to the engine-tier-only proposal it replaced?

3. **Whether module state (`level`/`g_entities`/`g_clients`) lives in the
   module crate as one `GameWorld` struct**, mirroring engine-plan.md's
   explicit non-scoping of this ("game module's own globals ... stay
   faithful for now — not folded into this work"). If yes: does `GameWorld`
   own `Vec<Entity>` with `EntityId(u32)` per porting-rules §B5 (GP2
   precedent, §5), and does `level` become a field of `GameWorld` or a
   sibling passed alongside it? SP's `g_clients`-doesn't-exist asymmetry
   (§2.4) means `GameWorld` likely can't be fully unified MP/SP (consistent
   with DEC-04: strict per-mode duplication during porting).

4. **How shared entity memory works per transport** (§4.1, §4.3, DEC-05
   item 5). Native: real shared pointer (`sv.gentities` aliasing
   `g_entities` — direct unsafe pointer cast is the closest faithful
   translation, confined to the ABI seam per porting-rules §D11). WASM:
   must become `VM_ArgPtr`-style offset translation into wasm linear memory
   — does `GameWorld`'s entity storage need to be laid out as a single
   contiguous `#[repr(C)]` buffer *regardless* of transport (so the
   native-pointer-cast path and the wasm-offset-translation path are just
   two different `unsafe` readers over the same underlying byte layout), or
   does the native path get to use an idiomatic `Vec<Entity>` while only the
   wasm path pays for a `repr(C)` buffer? This determines whether
   `GameWorld` has one shape or two.

5. **Whether SP's copy-based `CL_GetSnapshot` pattern (§4.3) generalizes**
   as the default cross-boundary data-sharing idiom even where oracle uses
   raw pointer aliasing (MP `game`↔server), i.e.: should the Rust port
   *narrow* the aliasing surface versus oracle (copy at every module
   boundary, even the native MP `game` one) as a deliberate improvement, or
   must it preserve oracle's true-aliasing behavior for behavioral parity
   (porting-rules §A1) since a copy vs alias distinction is observable if
   the game module mutates a `gentity_t` field the engine reads back same-frame
   without a round trip through a trap? (This one likely resolves in favor
   of preserving aliasing, since porting-rules §A2 forbids "guessing a
   cleaner behavior" — but the session should state it, since it fixes design
   fork 4 above: real aliasing rules out a copy-based `GameWorld` API for the
   native path.)

6. **Where `Com_Error`'s "no partial recovery" corollary (Chain B, §3) is
   enforced.** DEC-08 settles panic + `catch_unwind` at the frame boundary,
   but Chain B shows oracle's actual invariant is stronger: teardown of the
   entire owning subsystem happens *before* the throw, so the catch handler
   never sees inconsistent state. Does the Rust `Engine`/`GameWorld` need an
   explicit `fn teardown(&mut self)` called from inside the panic path
   (mirroring `SV_Shutdown`'s pre-throw call), or is running teardown logic
   *after* `catch_unwind` catches sufficient? The order matters if teardown
   itself would panic on the borrowed-but-not-yet-restored state fork 1/2
   leave behind.

7. **How much of qcommon's flat-global surface collapses into `Engine` vs
   stays behind small owning types.** §1n's "cleanly encapsulated already"
   list (VM table, `cmd_functions`/`cvar_vars`, sound mixer, `kg`) is
   evidence some subsystems are already effectively single-owner behind an
   API and can become an owned Rust struct with near-zero redesign; the
   "dirtiest cross-boundary offenders" list (`sv`/`svs`, `cls`, `cmg` +
   render cache pair, `com_*`/`cvar_modifiedFlags`) is where fork 1's
   flat-vs-nested question actually bites. Also decide whether to
   *faithfully reproduce* two flagged oracle divergences (the
   `files_pc.cpp:2328-2330` static-shadowing of the referenced-paks
   globals; SP's fixed-array vs MP's linked-list `cmd_functions`) or
   quietly fix them — porting-rules §A2 says port faithfully first, so the
   default is reproduce-then-flag, but a shadowed-global bug is a case
   where "faithful" and "correct" state ownership genuinely diverge, and
   the session should say which wins here.
