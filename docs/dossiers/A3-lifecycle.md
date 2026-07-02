# A3 — Lifecycle dossier (boot / frame / shutdown, all three executables)

Ground-truth survey for the A3 design doc. Every claim cites
`oracle/oracle/<path>:<line>`. Feeds the design session on boot/frame/shutdown
contracts for `jamp` (MP client), `jampded` (MP dedicated), `jasp` (SP).

**Headline findings**

- **Raven uses C++ exceptions, not setjmp/longjmp.** `Com_Error` throws
  `const char*` literals caught by `try/catch` in `Com_Frame` and `Com_Init`.
  No live `jmp_buf` exists in either tree (only vendored libjpeg internals).
  This maps *directly* onto DEC-08's panic + `catch_unwind`.
- **`NET_Init` is called from `WinMain`/`main`, not `Com_Init`, in MP** —
  the entry point owns network bring-up (`codemp/win32/win_main.cpp:1561`,
  `codemp/null/win_main.cpp:1459`).
- **SP's "early renderer init in Com_Init" is `#ifdef _XBOX` dead code.** On
  the shipping Win32 build SP initializes the renderer in the same late place
  MP does (inside `CL_Init` → `CL_InitRef`). See §3 step 6.
- **SP has no live `com_dedicated` cvar at all** — only commented-out guards
  (`code/qcommon/common.cpp:1093,1342`).
- The frame-rate cap is a **busy-spin on `Com_EventLoop()`**, not a sleep;
  the only sleeps are `Sleep(5)` in the OS entry loop and `NET_Sleep` inside
  dedicated `SV_Frame`.

---

## 1. MP client — `jamp`

### 1.1 Entry: WinMain

`WinMain` — `oracle/oracle/codemp/win32/win_main.cpp:1524`.

1. `Sys_CodeInMemoryChecksum`/`Sys_VerifyCodeChecksum` — `win_main.cpp:1532-1533`.
2. `Sys_CreateConsole()` — `win_main.cpp:1539` (early system console, before Com_Init so errors have output).
3. `SetErrorMode(SEM_FAILCRITICALERRORS)` — `win_main.cpp:1542`.
4. `Sys_Milliseconds()` warm-up (captures time base) — `win_main.cpp:1545`.
5. `Sys_InitStreamThread()` — `win_main.cpp:1553`.
6. `Com_Init(sys_cmdline)` — `win_main.cpp:1555` → `codemp/qcommon/common.cpp:1216`.
7. `QuickMemTest()` (non-DEDICATED) — `win_main.cpp:1557-1559`.
8. **`NET_Init()`** — `win_main.cpp:1561`, def `codemp/win32/win_net.cpp:1151`.
   Confirmed *not* called inside Com_Init (full read of common.cpp:1216-1442).
9. `Sys_ShowConsole(0, qfalse)` unless dedicated/viewlog — `win_main.cpp:1565-1567`.
10. `while(1)` main loop — `win_main.cpp:1576-1604`: `Sleep(5)` if
    minimized/dedicated (:1578-1580), `IN_Frame()` (:1596), `Com_Frame()`
    (:1599). The loop never returns; exit is via `Sys_Quit`/`Sys_Error`.

### 1.2 Com_Init — full order

`oracle/oracle/codemp/qcommon/common.cpp:1216-1442`. Body wrapped in
`try { ... } catch (const char* reason) { Sys_Error(...); }` (:1221, :1439-1441)
— init-time errors always escalate to fatal.

1. Version banner `Com_Printf` — common.cpp:1219.
2. `Com_InitPushEvent()` — :1224, def common.cpp:834 (clears `com_pushedEvents` ring "before anything else decides to push events").
3. `Cvar_Init()` — :1226, def `codemp/qcommon/cvar.cpp:951` (registers `sv_cheats`, `toggle/set/sets/setu/seta/reset/cvarlist/cvar_restart`).
4. `Com_ParseCommandLine(commandLine)` — :1230, def common.cpp:397 (splits on `+`/`\n` into `com_consoleLines[]`).
5. `Cbuf_Init()` — :1233, def `codemp/qcommon/cmd_common.cpp:54`.
6. `Com_InitZoneMemory()` — :1235, def `codemp/qcommon/z_memman_pc.cpp:577` (Z_Malloc pool: zeroes `TheZone`, sets magic, registers `zone_stats`/`zone_details`).
7. `Cmd_Init()` — :1242, def `cmd_common.cpp:501` (`cmdlist/exec/vstr/echo/wait`).
8. `Com_StartupVariable(NULL)` — :1245, def common.cpp:451 (applies `+set` cmdline cvars early).
9. `Rand_Init(Sys_Milliseconds(true))` — :1248.
10. `Com_StartupVariable("developer")` — :1251.
11. `CL_InitKeyCommands()` — :1254 ("done early so bind command exists" for config exec). Real def `codemp/client/cl_keys.cpp:1403` (registers `bind/unbind/unbindall/bindlist`); dedicated build links the no-op stub `codemp/null/null_client.cpp:57` instead. (Note: `cl_keys.cpp` contains non-UTF-8 bytes; use `grep -a`.)
12. `FS_InitFilesystem()` — :1266, def `codemp/qcommon/files.cpp:3433`. Single pass: reapplies `fs_cdpath/fs_basepath/fs_homepath/fs_game/fs_copyfiles/fs_restrict` startup vars, `FS_Startup(BASEGAME)`, `FS_SetRestrictions()`, `Com_Error(ERR_FATAL)` if `mpdefault.cfg` unreadable. No restart pass inside Com_Init — `FS_Restart` (files.cpp:3471) is invoked later (e.g. pure-server handshake).
13. `Com_InitJournaling()` — :1268, def common.cpp:759 (see §6).
14. `Cbuf_AddText("exec mpdefault.cfg\n")` — :1270.
15. Unless `Com_SafeMode()` (:1273, def common.cpp:425): `exec jampserver.cfg` under `DEDICATED` (:1275) else `exec jampconfig.cfg` (:1277).
16. `Cbuf_AddText("exec autoexec.cfg\n")` — :1281.
17. `Cbuf_Execute()` — :1283 (runs the queued config execs now).
18. `Com_StartupVariable(NULL)` again — :1286 (cmdline `+set`s re-override configs).
19. **`com_dedicated` registration branch** — :1288-1293:
    `#ifdef DEDICATED` → `Cvar_Get("dedicated","2",CVAR_ROM)` else
    `Cvar_Get("dedicated","0",CVAR_LATCH)`. Comment: "get dedicated here for
    proper hunk megs initialization". Gates steps 33/35.
20. `Com_InitHunkMemory()` — :1295, def z_memman_pc.cpp:678 (`hunk_tag = TAG_HUNK_MARK1; Hunk_Clear();`).
21. `cvar_modifiedFlags &= ~CVAR_ARCHIVE` — :1299.
22. Bulk `Cvar_Get` block — :1304-1360: `com_maxfps, com_blood, developer, vmdebug, logfile, timescale, fixedtime, com_showtrace, com_terrainPhysics, com_dropsim, viewlog, com_speeds, timedemo, com_cameraMode, com_optvehtrace, cl_paused, sv_paused, sv_running, cl_running, com_buildScript`, RMG_* block, `com_introplayed`, `com_noErrorInterrupt`.
23. `if (com_dedicated->integer)` → force `viewlog=1` — :1362-1366.
24. `if (com_developer->integer)` → register `error/crash/freeze` — :1368-1372.
25. `Cmd_AddCommand("quit", Com_Quit_f)`, `"changeVectors"`, `"writeconfig"` — :1373-1375.
26. `com_version` cvar (ROM/serverinfo) — :1377-1378.
27. `SE_Init()` (StringEd localization) — :1380, def `codemp/qcommon/stringed_ingame.cpp:1156`.
28. `Sys_Init()` — :1382, def `codemp/win32/win_main.cpp:1336` (OS version check, CPU detect → `sys_cpuid`/`sys_cpustring`, `username`, `sys_cpuspeed`, `sys_memory`; ends with `IN_Init()` at win_main.cpp:1472).
29. `Netchan_Init(Com_Milliseconds() & 0xffff)` — :1383, def `codemp/qcommon/net_chan.cpp:56`.
30. **`VM_Init()`** — :1384, def `codemp/qcommon/vm.cpp:50` (`vm_cgame/vm_game/vm_ui` cvars, `vmprofile/vminfo`, zeroes `vmTable`). MP-only; SP has this commented out (§3 step 25).
31. `SV_Init()` — :1385, def `codemp/server/sv_init.cpp:803` (serverinfo/systeminfo cvars, `SV_AddOperatorCommands()`).
32. `com_dedicated->modified = qfalse` — :1393.
33. **`if (!com_dedicated->integer) { CL_Init(); Sys_ShowConsole(...); }`** — :1394-1397. `CL_Init` def `codemp/client/cl_main.cpp:2549`: `Con_Init()` (:2552), `CL_ClearState()` (:2554), `CL_InitInput()` (:2560), cl_*/userinfo cvars (:2565-2663), command regs (:2669-2691), **`CL_InitRef()`** (:2693, def :2480 — `GetRefAPI(REF_API_VERSION)`, stores `re`, sets `cl_paused=0`; *only wires the export table, does not run R_Init/BeginRegistration*), `SCR_Init()` (:2695), `Cbuf_Execute()` (:2697), `Cvar_Set("cl_running","1")` (:2699).
34. `com_frameTime = Com_Milliseconds()` — :1402 (so a cmdline-started map gets a random-enough serverid).
35. `Com_AddStartupCommands()` — :1406, def common.cpp:484; if it returned false and not dedicated, queue `cinematic openinglogos.roq` (:1409-1419, `#ifndef _DEBUG`).
36. `Cvar_Set("r_uiFullScreen","1")` — :1423.
37. **`CL_StartHunkUsers()`** — :1425, def cl_main.cpp:2445 — **the real renderer/sound/UI start**, gated on `com_cl_running`: `CL_InitRenderer()` (:2456, def :2424 — `re.BeginRegistration`, loads charset/white/console shaders), `S_Init()` (:2461), `S_BeginRegistration()` (:2466), `CL_InitUI()` (:2471). Two-phase pattern: step 33 wires the ref API; step 37 actually starts it.
38. `Cvar_Set("ui_singlePlayerActive","0")` — :1428.
39. `SH_Register()` under `#ifdef MEM_DEBUG` — :1431.
40. `com_fullyInitialized = qtrue` — :1434.
41. `Com_Printf("--- Common Initialization Complete ---\n")` — :1435.
42. `catch (const char* reason) → Sys_Error` — :1439-1441.

### 1.3 Com_Frame — per-frame sequence

`oracle/oracle/codemp/qcommon/common.cpp:1593-1777`. Wrapped in
`try { ... } catch (const char* reason) { Com_Printf(reason); return; }`
(:1596, :1762-1765) — this catch is the ERR_DROP recovery point.

1. `Com_WriteConfiguration()` — :1624, def :1471 (writes `jampconfig.cfg`/`jampserver.cfg` if archive cvars changed).
2. `com_viewlog->modified` → `Sys_ShowConsole` toggle — :1627-1632 (skipped when dedicated).
3. com_speeds: `timeBeforeFirstEvents = Sys_Milliseconds()` — :1637-1639.
4. `minMsec = 1000/com_maxfps` unless dedicated/timedemo, else `1` — :1642-1646.
5. **Event pump + FPS-cap spin** — :1647-1653:
   `do { com_frameTime = Com_EventLoop(); ... msec = com_frameTime - lastTime; } while (msec < minMsec)`.
   Pure busy-spin; no Sleep here (dedicated relies on `Sleep(5)` in the OS loop and `NET_Sleep` in SV_Frame).
6. `Cbuf_Execute()` — :1654; `lastTime = com_frameTime` — :1656.
7. `com_frameMsec = msec; msec = Com_ModifyMsec(msec)` — :1659-1660 (see §5).
8. com_speeds: `timeBeforeServer` — :1665-1667.
9. **`SV_Frame(msec)`** — :1669, def `codemp/server/sv_main.cpp:826`.
10. **`com_dedicated->modified` hot-toggle** — :1675-1687: → non-dedicated:
    `CL_Init()` + `Sys_ShowConsole` + `CL_StartHunkUsers()`; → dedicated:
    `CL_Shutdown()` + `Sys_ShowConsole(1,qtrue)`. ("after the server may have
    started, but before the client tries to auto-connect"). Dead on the true
    DEDICATED build (cvar is ROM).
11. `if (!com_dedicated->integer)` client block — :1692-1716: second
    `Com_EventLoop()` + `Cbuf_Execute()` (:1700-1701, "get server to client
    packets without a frame of latency"); com_speeds `timeBeforeEvents`
    (:1698) / `timeBeforeClient` (:1708) / `timeAfter` (:1714);
    **`CL_Frame(msec)`** (:1711, def `codemp/client/cl_main.cpp:2268`).
12. com_speeds report — :1721-1733: `all/sv/ev/cl` deltas (subtracting
    `time_game` from sv, `time_frontend+time_backend` from cl).
13. com_showtrace: print/reset `c_traces/c_brush_traces/c_patch_traces/c_pointcontents` — :1738-1749.
14. Dead legacy `key = lastTime * 0x87243987` — :1752 (value never read).
15. `com_frameNumber++` — :1754; `#ifdef _XBOX XBL_Tick()` — :1758.
16. `catch` → print + return — :1762-1765.
17. `#ifdef G2_PERFORMANCE_ANALYSIS` timers — :1767-1776.

### 1.4 Shutdown

- **`Com_Quit_f`** — common.cpp:356-365: guard `!com_errorEntered` (:358) →
  `SV_Shutdown("Server quit\n")` (:359, def `codemp/server/sv_init.cpp:929`) →
  `CL_Shutdown()` (:360, def `codemp/client/cl_main.cpp:2719`) →
  `Com_Shutdown()` (:361, def common.cpp:1785) → `FS_Shutdown(qtrue)` (:362,
  def `codemp/qcommon/files.cpp:2868`) → `Sys_Quit()` (:364, unconditional —
  runs even in the recursive-error case, def `codemp/win32/win_main.cpp:389`).
- **`Com_Shutdown`** — common.cpp:1785-1810: `CM_ClearMap()` (:1787), close
  `logfile` (:1789-1793), close `com_journalFile` (:1795-1798),
  `MSG_shutdownHuffman()` (:1800).
- **`Sys_Quit`** — win_main.cpp:389-397: `timeEndPeriod(1)`, `IN_Shutdown()`,
  `Sys_DestroyConsole()`, `Com_ShutdownZoneMemory()`,
  `Com_ShutdownHunkMemory()`, `exit(0)`.
- **`CL_Shutdown` internals** — cl_main.cpp:2719-2774: recursion guard
  (:2720-2728), delete `G2VertSpaceClient` (:2730-2734), `CL_Disconnect(qtrue)`
  (:2736), `CL_ShutdownRef()` (:2738, def :2411 — must precede `CL_ShutdownAll`
  "so images get dumped in RE_Shutdown"), `CL_ShutdownAll()` (:2741, def :657),
  `S_Shutdown()` (:2743, def `codemp/client/snd_dma.cpp:650`),
  `Cmd_RemoveCommand` block (:2746-2764), `Cvar_Set("cl_running","0")` (:2766),
  `memset(&cls,0)` (:2770).
- **Gap:** `NET_Shutdown()` (def `codemp/win32/win_net.cpp:1193` —
  `NET_Config(qfalse)` + `WSACleanup()`) is declared (`qcommon.h:133`) but has
  **zero call sites** in the MP tree — Winsock teardown relies on process exit.
- Fatal-path shutdown (Com_Error ERR_FATAL, common.cpp:337-344):
  `CL_Shutdown()` → `SV_Shutdown(...)` → `Com_Shutdown()` →
  `Sys_Error(...)` — note **no `FS_Shutdown`** on this path, unlike Com_Quit_f.

---

## 2. MP dedicated — `jampded`

### 2.1 Entry: `main()` in null/win_main.cpp

`int main(int argc, char **argv)` — `oracle/oracle/codemp/null/win_main.cpp:1410-1499`
(the null replacement for win32's `WinMain`; `//int WINAPI WinMain(...)` at :1409).

Startup: merge argv into `cmdline` (:1425-1434); `SetErrorMode` (:1444);
`Sys_Milliseconds()` (:1447); `Sys_InitStreamThread()` (:1455);
`Com_Init(cmdline)` (:1457); **`NET_Init()`** (:1459); print working directory
(:1461-1462); hide console if `!com_dedicated && !com_viewlog` (:1466-1468 —
never taken on the true dedicated build).

Loop, `while(1)` (:1477-1498): `Sleep(5)` every iteration (:1478-1481 —
commented-out conditional guard at :1479 shows it was meant to be conditional);
`IN_Frame()` (:1490 — the no-op stub `null_input.cpp:6`); `Com_Frame()` (:1493).
No exit condition; `Sys_Quit`/`Sys_Error` (`null/win_main.cpp:363`, :324-355)
terminate via `exit()`.

### 2.2 null/ stub inventory

`ls oracle/oracle/codemp/null/`: `mac_net.c, null_client.cpp, null_glimp.cpp,
null_input.cpp, null_main.c, null_net.c, null_renderer.cpp, null_snddma.cpp,
win_main.cpp`.

**Build truth (`oracle/oracle/codemp/WinDed.vcproj`):** the dedicated target
compiles `null_client.cpp, null_glimp.cpp, null_input.cpp, null_renderer.cpp,
null_snddma.cpp, null/win_main.cpp` (WinDed.vcproj:379-394) **plus the real**
`win32/win_net.cpp` and `win32/win_shared.cpp` — networking and OS glue are
real, not nulled. `null_net.c`/`mac_net.c`/`null_main.c` are *not* in the
project (leftover generic null-platform stubs; `null_main.c` even has its own
conflicting `main()` at null_main.c:87). `WinDed.vcproj:32` defines
`DEDICATED`; the client project `jk2mp.vcproj:32` does not, and compiles the
real `win32/win_main.cpp, win_input.cpp, win_snd.cpp, win_glimp.cpp,
win_syscon.cpp, win_wndproc.cpp` + full `renderer/tr_*.cpp` set instead.

Per-file stub inventory:

- **null_client.cpp** (stubs client tier for symbols common.cpp calls
  unconditionally): `CL_Shutdown` :9, `CL_Init` :12-14 (registers only
  `cl_shownet`), `CL_MouseEvent` :16, `Key_WriteBindings` :19, `CL_Frame` :22,
  `CL_PacketEvent` :25, `CL_CharEvent` :28, `CL_Disconnect` :31,
  `CL_MapLoading` :34, `CL_GameCommand`→qfalse :37-39, `CL_KeyEvent` :41,
  `UI_GameCommand`→qfalse :44-46, `CL_ForwardCommandToServer` :48,
  `CL_ConsolePrint` :51, `CL_JoystickEvent` :54, `CL_InitKeyCommands` :57,
  `CL_CDDialog` :60, `CL_FlushMemory` :63, `CL_StartHunkUsers` :66.
- **null_snddma.cpp** (stubs DirectSound backend + a few S_ entry points):
  `SNDDMA_Init`→qfalse :9-12, `SNDDMA_GetDMAPos`→0 :14-17, `SNDDMA_Shutdown`
  :19-21, `SNDDMA_BeginPainting` :23-25, `SNDDMA_Submit` :27-29,
  `S_RegisterSound`→0 :31-33, `S_StartLocalSound` :35-36, `S_ClearSoundBuffer`
  :38-39, `SND_RegisterAudio_LevelLoadEnd`→qfalse :41-44,
  `SND_FreeOldestSound`→0 :46-48.
- **null_glimp.cpp** (stubs win_glimp/win_qgl; duplicated `#ifdef _WIN32`
  blocks :7/:41/:74): `GLimp_EndFrame` :19/:52, `GLimp_Init` :22/:55,
  `GLimp_Shutdown` :26/:59, `GLimp_EnableLogging` :29/:62, `GLimp_LogComment`
  :32/:65, `QGL_Init`→qtrue :35-37/:68-70, `QGL_Shutdown` :39/:72-73.
- **null_input.cpp**: `IN_Init` :3 (called from Sys_Init at
  null/win_main.cpp:1396, `// FIXME: not in dedicated?`), `IN_Frame` :6,
  `IN_Shutdown` :9, `Sys_SendKeyEvents` :12.
- **null_renderer.cpp** (link-resolution only): `RB_StageIteratorGeneric` :3,
  `RB_StageIteratorSky` :7, `RB_StageIteratorVertexLitTexture` :11,
  `RB_StageIteratorLightmappedMultitexture` :15, `R_SyncRenderThread` :19.

### 2.3 com_dedicated branches in common.cpp

`cvar_t *com_dedicated;` — `codemp/qcommon/common.cpp:41`.

- :162 — `Com_Printf`: `CL_ConsolePrint` only when not dedicated; dedicated
  falls through to `Sys_Print(msg)` at :167.
- :1288-1293 — registration: `DEDICATED` build → `"2"` + `CVAR_ROM`
  (no runtime toggle possible); client build → `"0"` + `CVAR_LATCH`.
- :1362-1366 — dedicated forces `viewlog=1`.
- :1393-1397 — `CL_Init()` skipped entirely when dedicated.
- :1409-1419 — intro cinematic never queued when dedicated.
- :1553-1571 (`Com_ModifyMsec`) — dedicated `clampTime = 5000` (vs 200 local),
  "Hitch warning" print when msec > 500 (:1557-1559).
- :1627-1632 — viewlog console toggle skipped when dedicated.
- :1642-1646 — dedicated `minMsec = 1` (uncapped spin; throttled only by
  `Sleep(5)` in the OS loop and `NET_Sleep` in SV_Frame).
- :1671-1687 — latched-toggle hot-swap (CL_Init/CL_Shutdown) — dead on ROM.
- :1692 — second event-loop pass + `CL_Frame` skipped when dedicated.

The split is **both** link-time (null_* stubs) and runtime (DEDICATED macro
forcing the ROM cvar); either alone would suffice, Raven did both.

### 2.4 Console input: Sys_ConsoleInput

- Dedicated impl — `codemp/null/win_main.cpp:200-302`: non-blocking
  `kbhit()`/`getch()` conio poll (:207 `if (!kbhit()) return NULL;`), static
  line-edit buffers `g_consoleField1/2` (:190-191,:204), per-key state machine
  (:237-299) with history (up-arrow :213-235), backspace (:244-249), Tab
  completion via `Cmd_CommandCompletion`/`Cvar_CommandCompletion` (:250-259),
  Esc clear (:260-264), Enter returns the line (:265-273), Ctrl-V paste
  (:274-289). Returns NULL until a full line is ready.
- Windowed-client impl (contrast) — `codemp/win32/win_syscon.cpp:456` (GUI
  edit control); Unix — `codemp/unix/unix_main.c:259`.
- Feed path: `Sys_GetEvent()` (`null/win_main.cpp:1187`) calls
  `Sys_ConsoleInput()`; a returned line is Z_Malloc-copied (:1193) and queued
  as `Sys_QueEvent(0, SE_CONSOLE, ...)` (:1195). `Com_EventLoop`'s
  `SE_CONSOLE` case (`common.cpp:969-979`) strips a leading `\`/`/` and
  `Cbuf_AddText`s the line + `"\n"`; `Cbuf_Execute()` (common.cpp:1654) runs
  it that frame.

---

## 3. SP — `jasp`

### 3.1 Entry: WinMain

`WinMain` — `oracle/oracle/code/win32/win_main.cpp:1166-1238`.

1. `Sys_CreateConsole()` — :1182.
2. `SetErrorMode(SEM_FAILCRITICALERRORS)` — :1185.
3. `Sys_Milliseconds()` — :1188.
4. `Sys_InitStreamThread()` — :1195 (def `code/win32/win_main_common.cpp:302`).
5. `Com_Init(sys_cmdline)` — :1197 → `code/qcommon/common.cpp:950`. Direct
   single call; no launcher indirection. **Note: no `NET_Init()` call in SP's
   WinMain** (unlike MP; SP does `Netchan_Init` inside Com_Init, step 24 below).
6. `QuickMemTest()` — :1199 (def :1119-1154; 128MB malloc probe, can
   `Com_Error(ERR_FATAL)`).
7. Hide console if `com_viewlog==0` — :1206-1208.
8. `while(1)`: `IN_Frame()` + `Com_Frame()` — :1211-1238, with `Sleep(5)` when
   minimized / `Sleep(50)` when inactive in `_DEBUG`.

### 3.2 Com_Init — full order

`oracle/oracle/code/qcommon/common.cpp:950-1130`; wrapped in
`try { ... } catch (const char* reason) { Sys_Error(...); }` (:955, :1119-1121).

1. Version banner — :953.
2. `Com_ParseCommandLine(commandLine)` — :958.
3. `Swap_Init()` — :960.
4. `Cbuf_Init()` — :961 (def `code/qcommon/cmd.cpp:46`).
5. `Com_InitZoneMemory()` — :963 (def `code/qcommon/z_memman_pc.cpp:866`).
6. **`#ifdef _XBOX` block — :965-981, DEAD on the shipping Win32 build:**
   `WF_Init()` (:966 — Xbox "Win File" 8-slot file-handle table, def
   `code/win32/win_file_xbox.cpp:36-46`; *not* a widget framework),
   `CL_InitRef()` (:969), `R_Register()` (:973), `GLimp_Init()` (:977),
   `SP_DoLicense()` (:980). **This is the "early renderer init at 969-977" —
   it is real code but Xbox-only.** On Win32, `CL_InitRef` is called inside
   `CL_Init` at `code/client/cl_main.cpp:1294`, i.e. the same late position as
   MP. The headless design does NOT need a special early-renderer path for SP
   on the platforms we target; it needs the stub at the same CL_Init seam as
   MP. (This corrects the note in `docs/decisions.md` DEC-01 which cites
   969-977 as live behavior.)
7. `Cmd_Init()` — :983 (def cmd.cpp:697).
8. `Cvar_Init()` — :984 (def `code/qcommon/cvar.cpp:883`).
9. `Com_StartupVariable(NULL)` — :987 (def :426).
10. `CL_InitKeyCommands()` — :990 ("done early so bind command exists").
11. `#ifdef _XBOX`: `Sys_InitFileCodes()` + `filecodes` cmd + `Sys_StreamInit()` — :992-999 (dead on Win32).
12. `FS_InitFilesystem()` — :1001 (def `code/qcommon/files_common.cpp:551`; "uses z_malloc").
13. `R_InitWorldEffects()` — :1002 ("doesn't do much but I want to be sure certain variables are initialized").
14. Config execs: `exec default.cfg` (:1004); `exec jaconfig.cfg` unless
    `Com_SafeMode()` (:1006-1009, def :400); `exec autoexec.cfg` (:1011);
    `Cbuf_Execute()` (:1013).
15. `Com_StartupVariable(NULL)` again — :1016 (CLI overrides configs).
16. `Com_InitHunkMemory()` — :1019 (def common.cpp:614).
17. `cvar_modifiedFlags &= ~CVAR_ARCHIVE` — :1023.
18. `Cmd_AddCommand("quit"/"writeconfig")` — :1028-1029.
19. Core cvar block — :1031-1053: `com_maxfps, developer, logfile, speedslog,
    timescale, fixedtime, com_showtrace, com_terrainPhysics, viewlog,
    com_speeds, (com_G2Report), cl_paused, sv_paused, sv_running, cl_running,
    skippingCinematic, com_buildScript`. **No `dedicated` cvar registered.**
20. Dev commands `error/crash/freeze` if developer — :1055-1059.
21. `com_version` — :1061-1062.
22. **`SE_Init()`** — :1064 ("Initialize StringEd" — localization string
    packages, def `code/qcommon/stringed_ingame.cpp:1156`; registers
    `se_language/se_debug/sp_leet`). Position: *before* Sys_Init, unlike MP
    where SE_Init (:1380) also precedes Sys_Init (:1382) — same relative spot.
23. `Sys_Init()` — :1066 (def `code/win32/win_main.cpp:976-1113`; CPU detect,
    `username`; ends with `IN_Init()` at win_main.cpp:1112, `// FIXME: not in
    dedicated?`).
24. `Netchan_Init(Com_Milliseconds() & 0xffff)` — :1068 (def `code/qcommon/net_chan.cpp:68`).
25. **`//	VM_Init();`** — :1069 — **fully commented out**; no live VM_Init
    anywhere in SP's Com_Init (SP game module is statically linked, no QVM
    abstraction needed).
26. `SV_Init()` — :1070 (def `code/server/sv_init.cpp:479`).
27. **`CL_Init()`** — :1072 (def `code/client/cl_main.cpp:1193-1310`):
    `Con_Init`, `CL_ClearState`, `CL_InitInput`, `RM_InitTerrain` (non-Xbox),
    cvars, **`CL_InitRef()`** at cl_main.cpp:1294, `CL_StartHunkUsers()`
    (:1296 — **inside CL_Init in SP**, vs a separate late Com_Init step in
    MP), `SCR_Init()` (:1298), `Cbuf_Execute()` (:1300),
    `Cvar_Set("cl_running","1")` (:1302). Unconditional — no dedicated gate.
28. `#ifdef _XBOX`: `CL_StartSound()` — :1078 (dead on Win32).
29. `Sys_ShowConsole(com_viewlog->integer, qfalse)` — :1081.
30. `com_frameTime = Com_Milliseconds()` — :1086.
31. `Com_AddStartupCommands()` — :1090 (def :459); if no `+`-commands and
    `NDEBUG`: `Cbuf_AddText("cinematic openinglogos\n")` (:1095). The
    commented-out `//if (!com_dedicated->integer)` guard sits at :1093.
32. `com_fullyInitialized = qtrue` — :1104; banner — :1105.
33. `catch` → `Sys_Error("Error during initialization %s", reason)` — :1119-1121.
34. `#ifdef _XBOX SE_CheckForLanguageUpdates()` — :1127 (dead on Win32).

Key MP divergences: no journaling init (§6), no VM_Init (:1069 commented), no
dedicated cvar, `CL_StartHunkUsers` inside `CL_Init` rather than a separate
Com_Init step, extra `Swap_Init`/`R_InitWorldEffects`/`speedslog`/
`skippingCinematic`, and NET_Init absent from both Com_Init and WinMain.

### 3.3 Com_Frame — per-frame sequence

`oracle/oracle/code/qcommon/common.cpp:1269-1463`; `try` (:1270) /
`catch (const char* reason) { Com_Printf(reason); return; }` (:1449-1453).

1. `Com_WriteConfiguration()` — :1278 (non-Xbox); viewlog toggle — :1281-1284.
2. com_speeds: `timeBeforeFirstEvents` — :1291.
3. Spin loop — :1295-1306: `minMsec = 1000/com_maxfps` (or 1 if <=0); loop
   `com_frameTime = Com_EventLoop()` until `msec >= minMsec`. No
   dedicated/timedemo guards (unlike MP :1642-1646); no NET_Sleep anywhere.
4. `Cbuf_Execute()` — :1307.
5. `lastTime = com_frameTime` (:1309); `com_frameMsec = msec` (:1312);
   `msec = Com_ModifyMsec(msec, fractionMsec)` (:1314) — SP variant with
   float fraction out-param, def :1197-1247 (see §5).
6. com_speeds: `timeBeforeServer` — :1320.
7. **`SV_Frame(msec, fractionMsec)`** — :1323.
8. `#ifdef _XBOX` demo-timer block — :1329-1341 (dead).
9. Client block — guard `//	if ( !com_dedicated->integer )` **commented out**
   at :1342, so unconditional: com_speeds `timeBeforeEvents` (:1349); second
   `Com_EventLoop()` (:1351, "second time to get server to client packets
   without a frame of latency") + `Cbuf_Execute()` (:1352); `timeBeforeClient`
   (:1359); **`CL_Frame(msec, fractionMsec)`** (:1362); `timeAfter` (:1365).
10. com_speeds report — :1373-1425: prints
    `fr/all/sv/ev/cl/gm/tr/pvs/rf/bk` (:1383-1384); optional `speedslog` file
    logging (:1386-1422, non-Xbox).
11. com_showtrace counters — :1430-1446.
12. `com_frameNumber++` — :1448.
13. `#ifdef G2_PERFORMANCE_ANALYSIS` `G2Time_ReportTimers`/`G2Time_ResetTimers` — :1455-1462.

### 3.4 SP dedicated handling — confirmed absent

- `grep -n com_dedicated code/qcommon/common.cpp` → only two hits, both
  commented out: `code/qcommon/common.cpp:1093` (Com_Init intro-cinematic
  guard) and `:1342` (Com_Frame client-block guard).
- No `Cvar_Get("dedicated", ...)` anywhere in SP's live code path. Live
  references exist only in non-shipping platform stubs:
  `code/mac/mac_input.c:36,161`, `code/unix/unix_net.c:432`,
  `code/unix/unix_main.c:197`. Self-documenting comment at
  `code/qcommon/cm_load.cpp:767`: "no need to check for dedicated in
  single-player codebase".
- SP's `code/null/` exists but is thin (`mac_net.c, null_glimp.c, null_main.c,
  null_net.c, null_snddma.c`) — **no** null_client/null_input/null_renderer,
  so no client-less build target exists. MP's `codemp/null/` has exactly those
  extra three files + `win_main.cpp` because jampded needs them.

### 3.5 SP shutdown

- **`Com_Quit_f`** — `code/qcommon/common.cpp:332-340`: guard
  `!com_errorEntered` (:334) → `SV_Shutdown("Server quit\n")` (:335) →
  `CL_Shutdown()` (:336) → `Com_Shutdown()` (:337) → `Sys_Quit()` (:339,
  unconditional). **No `FS_Shutdown`** — MP added it (`codemp` :362), SP never
  calls it here.
- SP `Com_Shutdown` closes logfile + journal handles —
  `code/qcommon/common.cpp:1495-1497` (journal handles exist but are never
  written; see §6).

---

## 4. Error recovery (feeds DEC-08)

### 4.1 Mechanism: C++ exceptions, NOT setjmp/longjmp

Tree-wide grep for `jmp_buf|setjmp|longjmp` in both trees finds **no live
usage** in the error path — only vendored libjpeg internals
(`codemp/jpeg-6/jerror.cpp:55-56`, `code/jpeg-6/jerror.cpp:60-61`) and stale
comments (`codemp/renderer/tr_image.cpp:1772,1906`,
`code/renderer/tr_jpeg_interface.cpp:173`, `codemp/qcommon/common.cpp:1612`).
JKA (unlike vanilla Q3) uses `throw`/`catch`:

- `Com_Error` throws string literals: `"DISCONNECTED\n"`
  (MP common.cpp:312 / SP :290), `"DROPPED\n"` (MP :326 / SP :301),
  `"NEED CD\n"` (MP :336 / SP :312).
- **Recovery point (per-frame):** `Com_Frame`'s
  `catch (const char* reason) { Com_Printf(reason); return; }` —
  MP `codemp/qcommon/common.cpp:1762-1765`, SP `code/qcommon/common.cpp:1450-1453`.
- **Init-time:** `Com_Init`'s catch escalates to `Sys_Error` — MP :1439-1441,
  SP :1119-1121. Errors during init are always fatal.

This is a 1:1 template for DEC-08: typed panic payload = the thrown level;
`catch_unwind` at the `Com_Frame` boundary = the existing catch; the recovery
work (SV_Shutdown/CL_Disconnect/...) happens *inside `Com_Error` before the
throw*, not in the catch handler — the catch only prints and returns.

### 4.2 Recursive-error guard

- Global `qboolean com_errorEntered` — MP `codemp/qcommon/common.cpp:83`
  (extern `codemp/qcommon/qcommon.h:722`); SP `code/qcommon/common.cpp:68`
  (extern `code/qcommon/qcommon.h:562`).
- Guard — MP :288-291 / SP :265-269: if already set →
  `Sys_Error("recursive error after: %s", com_errorMessage)` (fatal exit).
  Set on entry; cleared per-branch after successful cleanup (MP :305,:318,:332;
  SP :289,:300,:308); left set on the fatal path.
- **MP-only rapid-error escalation** — codemp common.cpp:277-286 (statics
  :251-252): >3 errors within 100ms of each other → force `code = ERR_FATAL`.
  Absent from SP.
- Both trees: `com_buildScript` forces `ERR_FATAL` (MP :270-272, SP :261-263).
- MP-only: `FS_PureServerSetLoadedPaks("", "")` at top of Com_Error
  (codemp :275, "make sure we can get at our local stuff").
- SP-only: unconditional `SG_Shutdown()` before dispatch
  (code :283, "close any file pointers" — savegame system).

### 4.3 ERR_ levels and per-level recovery

**Enums differ structurally:**

- MP `errorParm_t` — `codemp/game/q_shared.h:451-457`: `ERR_FATAL, ERR_DROP,
  ERR_SERVERDISCONNECT` ("don't kill server"), `ERR_DISCONNECT` ("client
  disconnected from the server"), `ERR_NEED_CD` — 5 levels.
- SP `errorParm_t` — `code/game/q_shared.h:251-256`: `ERR_FATAL, ERR_DROP,
  ERR_DISCONNECT` ("don't kill server" — comment drifted), `ERR_NEED_CD` —
  4 levels, **no ERR_SERVERDISCONNECT**. And despite the comment, SP's
  ERR_DISCONNECT *does* shut down the server.

**MP recovery sequences** (`codemp/qcommon/common.cpp:249-345`, Com_Error at :249):

| Level | Sequence (in order) |
|---|---|
| ERR_SERVERDISCONNECT (:302-312) | `CL_Disconnect(qtrue)` :303 → `CL_FlushMemory()` :304 → clear guard :305 → throw "DISCONNECTED\n" :312. Server deliberately NOT shut down. |
| ERR_DROP / ERR_DISCONNECT (combined, :313-326) | print ERROR banner :314 → `SV_Shutdown("Server crashed: ...")` :315 → `CL_Disconnect(qtrue)` :316 → `CL_FlushMemory()` :317 → clear guard :318 → throw "DROPPED\n" :326 |
| ERR_NEED_CD (:327-336) | `SV_Shutdown` :328 → if `com_cl_running`: `CL_Disconnect(qtrue)`/`CL_FlushMemory()`/clear guard :330-332 else print :334 → throw "NEED CD\n" :336 |
| ERR_FATAL (else, :337-344) | `CL_Shutdown()` :338 → `SV_Shutdown("Server fatal crashed: ...")` :339 → `Com_Shutdown()` :342 → `Sys_Error(...)` :344 — no throw, no recovery, guard stays set |

Before dispatch (all levels): message formatted into `com_errorMessage`
(:293-295) and published as a cvar unless ERR_DISCONNECT (:297-300).

**SP recovery sequences** (`code/qcommon/common.cpp:245-320`, Com_Error at :245):

| Level | Sequence |
|---|---|
| ERR_DISCONNECT (:284-290) | `SV_Shutdown("Disconnect")` :285 → `CL_Disconnect()` :286 → `CL_FlushMemory()` :287 → `CL_StartHunkUsers()` :288 → clear guard :289 → throw "DISCONNECTED\n" :290 |
| ERR_DROP (:291-301) | `SG_WipeSavegame("current")` :293 (delete temp save) → `SV_Shutdown("Server crashed: ...")` :295 → `CL_Disconnect()` :296 → `CL_FlushMemory()` :297 → `CL_StartHunkUsers()` :298 → print colorized ERROR banner :299 (*after* cleanup, unlike MP which prints first) → clear guard :300 → throw "DROPPED\n" :301 |
| ERR_NEED_CD (:302-312) | `SV_Shutdown` :303 → if cl_running: `CL_Disconnect()`/`CL_FlushMemory()`/`CL_StartHunkUsers()`/clear :305-308 else print :310 → throw "NEED CD\n" :312 |
| ERR_FATAL (:313-320) | `CL_Shutdown()` :314 → `SV_Shutdown` :315 → `Com_Shutdown()` :318 → `Sys_Error` :320 |

SP-vs-MP recovery diffs that matter for the Rust design: SP re-runs
`CL_StartHunkUsers()` in every recoverable branch (MP never does inside
Com_Error); SP `CL_Disconnect()` takes no arg (prototypes diverged); SP wipes
the temp savegame on ERR_DROP; MP has the pure-paks reset and the rapid-error
escalation.

### 4.4 Com_Quit flow

See §1.4 (MP) / §3.5 (SP). Summary: both guard the shutdown calls with
`!com_errorEntered` but call `Sys_Quit()` unconditionally; MP adds
`FS_Shutdown(qtrue)` (codemp :362) which SP omits.

---

## 5. Timing

### 5.1 Sys_Milliseconds

- MP — `codemp/win32/win_shared.cpp:22-34`: `timeGetTime()` (multimedia timer,
  NOT QueryPerformanceCounter) minus a static `sys_timeBase` captured at first
  call; `baseTime=true` skips the subtraction.
- SP — `code/win32/win_shared.cpp:17-25`: identical but old signature
  `int Sys_Milliseconds(void)` — always base-relative, no raw variant.

### 5.2 Cvar registration

- MP: `com_maxfps = Cvar_Get("com_maxfps","85",CVAR_ARCHIVE)`
  codemp common.cpp:1304; `timescale` `"1"` `CVAR_CHEAT|CVAR_SYSTEMINFO` :1311;
  `fixedtime` `"0"` `CVAR_CHEAT` :1312 (declarations :42,:43,:46).
  `sv_fps = Cvar_Get("sv_fps","20",CVAR_TEMP)` — `codemp/server/sv_init.cpp:852`
  (decl `codemp/server/sv_main.cpp:14`).
- SP: `com_maxfps` "85" — code common.cpp:1031; `timescale`/`fixedtime` —
  :1037-1038 (SP timescale lacks CVAR_SYSTEMINFO). `sv_fps` "20" —
  `code/server/sv_init.cpp:491`.

### 5.3 FPS cap / spin loop

MP `codemp/qcommon/common.cpp:1641-1653`: `minMsec = 1000/com_maxfps` unless
dedicated or `com_timedemo` (then 1); `do { com_frameTime = Com_EventLoop();
... } while (msec < minMsec)` — CPU-spin via event-loop repolling, no Sleep.
SP `code/qcommon/common.cpp:1294-1306`: same shape, no dedicated/timedemo
guards.

### 5.4 Com_ModifyMsec — clamp logic

MP — `codemp/qcommon/common.cpp:1534-1578`:
- `com_fixedtime` nonzero → msec overridden outright.
- else msec *= `com_timescale->value` (plus a redundant `com_cameraMode`
  branch multiplying by the same value).
- floor: msec >= 1 when timescale nonzero (:1549).
- ceiling `clampTime`: dedicated → **5000** (+ "Hitch warning" print if
  msec>500, :1557-1559); client of remote server (`!com_sv_running`) →
  **5000**; local single-player-hosted → **200**.
- Call site :1660, immediately before `SV_Frame(msec)` :1669.

SP — `code/qcommon/common.cpp:1197-1247` (called :1314): extra
`float &fraction` out-param accumulating the fractional msec lost to int
truncation under timescale (:1216-1222); no cameraMode branch; clamp is
`com_skippingcin->integer ? 500 : 200` (:1231-1239) — no 5000 paths (no
dedicated/remote-server concept).

### 5.5 sv_fps and the server's decoupled tick

`SV_Frame` — `codemp/server/sv_main.cpp:847-861`: `frameMsec = 1000/sv_fps`
(:850, forced to 10 if <1 :847-849); accumulates `sv.timeResidual += msec`
(:852); **dedicated only**: if `timeResidual < frameMsec` (and timescale >= 1)
→ `NET_Sleep(frameMsec - sv.timeResidual)` and return (:856-861) — the actual
dedicated-server throttle. Listen servers never take this path. SP mirrors:
`code/server/sv_main.cpp:500-503`.

---

## 6. Event system

### 6.1 Queue model

- Ring buffer — `codemp/win32/win_main.cpp:1162-1166`:
  `MAX_QUED_EVENTS 256` / `MASK_QUED_EVENTS`, `sysEvent_t eventQue[256]`,
  monotonic `eventHead/eventTail`.
- Push — `Sys_QueEvent`, win_main.cpp:1178-1203: overflow drops the oldest
  (freeing its `evPtr` via `Z_Free`); `time==0` → stamped `Sys_Milliseconds()`.
  Producers: Win32 message pump / DirectInput handlers
  (`codemp/win32/win_input.cpp:413-445`, :588-994).
- Pull — `Sys_GetEvent`, win_main.cpp:1211-1276: drain ring; else pump
  PeekMessage/DispatchMessage (:1225-1235), poll `Sys_ConsoleInput()` →
  SE_CONSOLE (:1238-1247), poll net → SE_PACKET (:~1250-1261), re-check ring
  (:1264-1268), else synthesize empty event stamped with current time
  (:1270-1275).
- Dedicated variant — `codemp/null/win_main.cpp` `Sys_GetEvent` (:1174-1211):
  drains Win32 messages, polls console + `Sys_GetPacket`. (A separate console
  build variant exists at `codemp/win32/win_main_console.cpp:130-180`, polling
  packets up to `MAX_POLL_RATE`=15 per call; the WinDed.vcproj target uses
  null/win_main.cpp.)
- SP: identical 256-entry ring + push/pull pair —
  `code/win32/win_main.cpp:872` area and `code/win32/win_main_console.cpp:130`.

### 6.2 Com_EventLoop / Com_GetEvent

`Com_EventLoop` — `codemp/qcommon/common.cpp:921-946+`: `while(1)` pulling
`Com_GetEvent()` (:881; drains the 1024-entry `com_pushedEvents` ring
:747-752 first, then `Com_GetRealEvent()` :789); on `SE_NONE` drains loopback
packets (`NET_GetLoopPacket` NS_CLIENT → `CL_PacketEvent`, NS_SERVER →
`Com_RunAndTimeServerPacket`) and returns `ev.evTime` (becomes
`com_frameTime`); dispatch: SE_KEY→`CL_KeyEvent`, SE_CHAR→`CL_CharEvent`,
SE_MOUSE→`CL_MouseEvent`, SE_JOYSTICK_AXIS→`CL_JoystickEvent`,
SE_CONSOLE→`Cbuf_AddText` (:969-979), SE_PACKET→drop-sim + dispatch (:980+);
unknown type → `Com_Error(ERR_FATAL)`. SP same shape —
`code/qcommon/common.cpp:784+`.

### 6.3 Journaling (MP only)

- `Com_InitJournaling` — codemp common.cpp:759-782: `journal` cvar
  (`CVAR_INIT`); `1` → open `journal.dat`/`journaldata.dat` for write
  (:768-769); `2` → open for read/replay (:772-773); open failure resets to 0
  (:776-781).
- Tap point — `Com_GetRealEvent`, common.cpp:789-825: `journal==2` →
  `FS_Read` the next `sysEvent_t` (+ `evPtrLength` payload) *instead of*
  calling `Sys_GetEvent`; `journal==1` → `FS_Write` every real event after
  `Sys_GetEvent`. Read/write failures are `ERR_FATAL`.
- **SP: journaling stripped.** Handles declared (`code/qcommon/common.cpp:28-29`)
  and closed at shutdown (:1495-1497), but no `com_journal` cvar, no
  Com_InitJournaling, and SP's `Com_GetRealEvent` (:699-706) is
  unconditionally `ev = Sys_GetEvent(); return ev;`.

---

## 7. Current Rust state

- `crates/mp/app/src/main.rs` — `fn main() {}` with
  `//TODO: Port module mp_app`. Deps already wired in
  `crates/mp/app/Cargo.toml`: `mp_engine_qcommon`, `mp_engine_server`,
  `mp_engine_client`, `mp_renderer`, `mp_abi`.
- `crates/sp/app/src/main.rs` — identical empty stub; deps
  `sp_engine_qcommon/server/client`, `sp_renderer`, `sp_abi`.
- `crates/native/platform/` exists but **no winit dependency anywhere in the
  workspace yet** (grep for `winit` across all Cargo.toml files → zero hits).
- **DEC-02 (winit) imposes control-flow inversion.** Raven's model is
  poll-style: the app owns `while(1) { IN_Frame(); Com_Frame(); }` and
  `Sys_GetEvent` *pulls* OS events on demand (PeekMessage inside the getter).
  Modern winit (0.29+ `ApplicationHandler` API) owns the loop and *pushes*
  events into callbacks (`window_event`, `about_to_wait`); on some platforms
  (macOS, iOS, web) `EventLoop::run` never returns and a manual pump is not
  supported (macOS strongly prefers the run-loop own the process). The
  adapter direction is fixed by Raven's architecture: winit callbacks must
  enqueue into the ported 256-entry `sysEvent_t` ring (the `Sys_QueEvent`
  equivalent), and the per-frame work (`Com_Frame`) runs from
  `about_to_wait`/redraw callbacks with `ControlFlow::Poll` — reproducing the
  busy-poll cadence — while `Sys_GetEvent` becomes a pure ring-drain with no
  OS pump inside it. `winit`'s `EventLoopExtPumpEvents::pump_app_events` is a
  fallback that preserves Raven's loop shape but is explicitly second-class /
  platform-limited.
- **DEC-01 (renderer deferred) headless paths needed:**
  - jampded: Raven's own null path already exists and is fully mapped (§2.2)
    — port the null_* stub surface (or gate the client tier out).
  - jamp client / jasp: need a null-renderer `refexport_t` stub behind the
    `CL_InitRef` seam (MP `codemp/client/cl_main.cpp:2480-2693`, SP
    `code/client/cl_main.cpp:1294`) and a null `CL_InitRenderer`
    (`re.BeginRegistration`) path in `CL_StartHunkUsers`
    (MP cl_main.cpp:2445-2471). §3.2 step 6 finding: SP does **not** need an
    early-Com_Init renderer stub on Win32-lineage platforms — the cited
    `code/qcommon/common.cpp:969-977` block is `#ifdef _XBOX` dead code, so
    the SP stub sits at the same seam as MP's. (Worth correcting in
    decisions.md DEC-01.)
  - Headless also implies no winit at all for jampded (pure console loop +
    `Sys_ConsoleInput` equivalent), and optionally a windowless mode for
    client binaries until the renderer lands.

---

## Design forks

Questions the A3 session must resolve, grounded in the findings above.

1. **Who owns the loop, per target.**
   - jampded: no window → no winit; a plain Rust `loop { com_frame(...) }`
     with `Sleep(5)`-equivalent (§2.1) and non-blocking console input (§2.4).
   - jamp/jasp: (a) winit `ApplicationHandler` owns the loop, `Com_Frame`
     called from `about_to_wait` with `ControlFlow::Poll`, winit events pushed
     into the ported sysEvent ring; or (b) manual `pump_app_events` preserving
     Raven's loop shape (platform-limited, second-class). Sub-question: does
     the FPS-cap busy-spin (§5.3) stay a spin (parity-faithful) or become
     `ControlFlow::WaitUntil` (idiomatic, diverges in event-arrival timing)?
2. **One lifecycle skeleton for three shapes.** MP client, MP dedicated, SP
   differ in: NET_Init location (entry vs absent), CL_Init gating
   (com_dedicated vs unconditional), CL_StartHunkUsers position (separate
   Com_Init step in MP vs inside CL_Init in SP), journaling (MP only),
   Com_ModifyMsec clamps (3-way vs 2-way). Fork: a shared generic
   `Lifecycle`/host trait with per-mode impls, vs strict per-mode duplication
   (DEC-04 says per-mode during porting — does the *app shell* count as a
   Raven-derived subsystem or new Rust code allowed to be shared?).
3. **jamp vs jampded packaging.** Raven uses a separate vcproj + `DEDICATED`
   define + null_* link substitution (§2.2). Rust options: (a) two `[[bin]]`
   targets in `mp_app` with a cargo feature (`dedicated`) swapping the client
   tier for stubs; (b) separate `mp_app`/`mp_dedicated_app` crates; (c) one
   binary with runtime `+set dedicated` only (rejected by parity: Raven's
   dedicated is compile-time ROM). Note the runtime hot-toggle path
   (common.cpp:1675-1687) must still exist in the *client* binary.
4. **Panic-payload taxonomy for DEC-08.** Raven throws 3 distinct strings for
   4-5 levels; recovery work happens *before* the throw inside Com_Error, and
   the catch only prints + returns (§4.1). Fork: mirror that exactly (payload
   = level + message; recovery inside the `com_error` fn before
   `panic_any`), vs moving recovery into the catch site. Must also decide:
   Rust representation of `ERR_SERVERDISCONNECT` (MP-only) vs SP's 4-level
   enum — two enums (per-mode, matches oracle) or one shared with an MP-only
   variant. Recursive-guard (`com_errorEntered`) and MP's 100ms/3-error
   escalation need a home in owned state (no statics, rule B3). ERR_FATAL and
   init-time errors bypass recovery entirely (abort path). Also mind rule:
   `panic = "unwind"` required; foreign/non-typed panics (real bugs) should
   probably be treated as ERR_FATAL, not swallowed.
5. **Journaling in/out of scope for slice 0.** MP-only, small surface
   (Com_InitJournaling + one tap in Com_GetRealEvent, §6.3), but it's the
   cheapest determinism/replay lever for differential testing against the
   oracle — arguably worth porting early precisely because A-doc verification
   wants reproducible event streams. Fork: port with slice 0 vs `//TODO: Port
   Com_InitJournaling` and stub `journal=0` behavior.
6. **Sys_Milliseconds source.** Raven uses `timeGetTime()` (ms resolution,
   wraps at 49.7 days, §5.1). Rust: `std::time::Instant` behind the same
   base-relative API. Decide the returned type width (`i32` per parity vs
   internally i64) and whether MP's `baseTime` raw variant needs preserving.
7. **NET_Shutdown is dead in Raven** (declared, never called, §1.4). Decide:
   faithful (leak, rely on process exit) vs Rust-idiomatic Drop teardown —
   invisible at the seam either way, but shutdown ordering docs should state
   it.
8. **Console input for jampded.** Raven polls conio `kbhit`/`getch` with a
   hand-rolled line editor (§2.4). Fork: faithful minimal poll (portable via
   crossterm/raw stdin) vs a modern line editor; either way it feeds
   SE_CONSOLE events into the same queue, so the seam is stable.
