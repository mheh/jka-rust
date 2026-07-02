# Lifecycle Design
Status: DRAFT     Supersedes: none
Decision prefix: LIFE     Ledger deps: DEC-01, DEC-02, DEC-04, DEC-07, DEC-08, DEC-09

## Standing context

Links only — never restated here. MP tree = `oracle/oracle/codemp/`, SP tree =
`oracle/oracle/code/`; file:line short forms below resolve against those roots.

- `docs/workspace-architecture.md` — crate graph and tiers (`native/*`,
  `abi-transport`, `crates/{mp,sp}/{qshared,engine/*,app}`).
- `docs/porting-rules.md` — §B3 (no `static mut`/ambient globals), §B4 (state
  threaded not reached), §B6 (one owned instance per singleton), §C7
  (out-params → returns; `qboolean` → `bool`), §C8 (`#define` → `const`/`enum`),
  §D11 (unsafe confined to the seam), §E (green-per-commit, slice-driven).
- `docs/decisions.md` — DEC-01 (renderer port deferred; headless boot), DEC-02
  (winit windowing/input), DEC-04 (per-mode duplication), DEC-07 (SP cgame/ui
  statically linked; SP has no live `VM_Init`), DEC-08 (`Com_Error` = panic +
  `catch_unwind`), DEC-09 (verification layers).
- `docs/architecture/two-island-model.md` — STATE-D1 island model; seam
  entrypoints/dispatchers are `extern "C-unwind"`.
- `docs/architecture/engine-seam.md` — SEAM-D1 (one `OnceLock<CEngine>` module
  seam global; `currentVM` eliminated), SEAM-D10 (`extern "C-unwind"` exports;
  the `catch_unwind` boundary sits at engine `Com_Frame`, outside the exports).
  This doc supplies the boot order that reaches those dispatchers.
- `docs/architecture/module-loading.md` — **LOAD-D5** (the per-slot
  `ModuleRegistry` the `Com_Init` step-30 `VM_Init` constructs — default-constructed
  empty — and **LOAD-Q3**, its `load_module` fill-signature, open). This doc names
  the empty-registry construct as a boot step; that doc owns its shape + crate home.
- `docs/architecture/state-ownership.md` — STATE-D1/D2 (islands, `GameWorld`),
  **STATE-D3** (recovery runs synchronously *before* `panic_any`; the same
  contract this doc renders as flow — cross-ref, not restated), the `Common`
  field shapes this doc times, the `ComError` payload, and **STATE-Q1** (the
  aggregate `Engine` type's defining crate, still open).
- `docs/dossiers/A3-lifecycle.md` — the survey this doc renders.

## Scope & non-goals

**This doc decides:** the per-executable boot / frame / shutdown contracts for
the three host binaries — `jamp` (MP client), `jampded` (MP dedicated), `jasp`
(SP): loop ownership, `com_init` order, `com_frame` anatomy, `com_shutdown`
order, the error-recovery *flow* (per-level sequences and the panic boundary),
the event pump and its winit/console adapters, timing (`Sys_Milliseconds`,
FPS-cap spin, `Com_ModifyMsec` clamps), journaling, and binary packaging. It
**freezes** the per-mode `ErrorLevel` (LIFE-D3, = the existing `errorParm_t`),
the `com_*` seam signatures, and the `SysEventQueue` surface.

**Non-goals** (each punted to its owning doc):

- **Seam mechanics** — how a trap/`vmMain` crosses, dispatcher routing, transport
  (`NativeDll|Static|Wasm`) → `docs/architecture/engine-seam.md`.
- **Module load / restart mechanics** — `SV_InitGameProgs`, DLL/`GetGameAPI`
  loading, `vm_restart`, VM table → `docs/architecture/module-loading.md`.
- **Subsystem internals** — cvar parse tables, filesystem search paths, sound
  mixer, StringEd, netchan framing, collision → `docs/subsystems/*`.
- **State-ownership spine beyond the lifecycle-owned rows** →
  `state-ownership.md`. In particular the **recovery-before-panic ordering
  contract is STATE-D3** — this doc renders the *per-level flow* Raven runs, but
  the "recover synchronously, then `panic_any`, guards must be `Drop`-safe" rule
  is STATE-D3's and is cross-referenced, never restated.
- **Module `GAME_INIT` internal order** (e.g. `level.gentities`/`level.clients`
  self-referential wiring, `g_main.c:979,984`) — that runs inside the game
  module at map spawn (`SV_SpawnServer` → `SV_InitGameProgs`), not at engine
  boot; its ordering is the game-module subsystem doc's. This doc owns only the
  engine-side trigger points.
- **Renderer bring-up internals** — DEC-01 defers the renderer; the seams where a
  null-`refexport_t` stub attaches (`CL_InitRef`/`CL_InitRenderer`) are named
  here as boot steps, their bodies are DEC-01/renderer-doc territory.

## Raven ground truth

Three headline corrections from the survey shape every contract below:

1. **`Com_Error` is C++ `throw`/`catch`, not `setjmp`/`longjmp`** — string
   literals thrown from `Com_Error`, caught in `Com_Frame` (MP `common.cpp:1762`,
   SP `common.cpp:1450`) and `Com_Init` (MP `:1439`, SP `:1119`). Recovery runs
   *before* the throw; the catch only prints and returns. Maps 1:1 onto
   panic + `catch_unwind` (DEC-08).
2. **`NET_Init` is called from the entry point, not `Com_Init`** — MP
   `win_main.cpp:1561` (client) / `null/win_main.cpp:1459` (dedicated). SP calls
   it *nowhere* (it does `Netchan_Init` inside `Com_Init`, step 24).
3. **The FPS cap is a busy-spin on `Com_EventLoop`, not a sleep** — MP
   `common.cpp:1647-1653`, SP `common.cpp:1295-1306`. The only sleeps are
   `Sleep(5)` in the OS entry loop and `NET_Sleep` inside dedicated `SV_Frame`.
   SP's `#ifdef _XBOX` early-renderer block (`common.cpp:965-981`) is dead code;
   PC SP inits the renderer inside `CL_Init` like MP (DEC-01 amendment).

### Entry points and OS loop

- **`jamp`** — `WinMain` (`win32/win_main.cpp:1524`): checksum, `Sys_CreateConsole`
  (`:1539`), `SetErrorMode` (`:1542`), `Sys_Milliseconds()` warm-up captures the
  time base (`:1545`), `Sys_InitStreamThread` (`:1553`), **`Com_Init`** (`:1555`),
  `QuickMemTest` (`:1557`), **`NET_Init`** (`:1561`), `Sys_ShowConsole(0)` unless
  dedicated/viewlog (`:1565`), then `while(1)`: `Sleep(5)` if minimized/dedicated
  (`:1578`), `IN_Frame` (`:1596`), **`Com_Frame`** (`:1599`). Never returns; exit
  via `Sys_Quit`/`Sys_Error`.
- **`jampded`** — `main` (`null/win_main.cpp:1410`): merge argv (`:1425`),
  `SetErrorMode` (`:1444`), `Sys_Milliseconds()` (`:1447`), `Sys_InitStreamThread`
  (`:1455`), **`Com_Init`** (`:1457`), **`NET_Init`** (`:1459`), then `while(1)`:
  `Sleep(5)` every iteration (`:1478`), `IN_Frame` (null stub, `:1490`),
  **`Com_Frame`** (`:1493`).
- **`jasp`** — `WinMain` (`code/win32/win_main.cpp:1166`): `Sys_CreateConsole`
  (`:1182`), `SetErrorMode` (`:1185`), `Sys_Milliseconds()` (`:1188`),
  `Sys_InitStreamThread` (`:1195`), **`Com_Init`** (`:1197`, single call, no
  launcher), `QuickMemTest` (`:1199`), hide console if `com_viewlog==0` (`:1206`),
  then `while(1)`: `IN_Frame` + **`Com_Frame`** (`:1211`), `Sleep(5)`/`Sleep(50)`
  when minimized/inactive. **No `NET_Init` in SP's entry point.**

### MP `Com_Init` — 42-step boot contract

`common.cpp:1216-1442`, body wrapped `try { … } catch (const char* reason)
{ Sys_Error(…); }` (`:1221`, `:1439`) — init-time errors are always fatal.
Dedicated (`jampded`) runs this *same* sequence; the `com_dedicated`-gated steps
are marked **[ded]**.

1. Version banner `Com_Printf` — `:1219`.
2. `Com_InitPushEvent()` — `:1224` (clears `com_pushedEvents` ring).
3. `Cvar_Init()` — `:1226`.
4. `Com_ParseCommandLine(cmdline)` — `:1230` (splits `+`/`\n` into `com_consoleLines[]`).
5. `Cbuf_Init()` — `:1233`.
6. `Com_InitZoneMemory()` — `:1235` (Rust: owned-arena, not `TheZone`; §C9).
7. `Cmd_Init()` — `:1242`.
8. `Com_StartupVariable(NULL)` — `:1245` (apply `+set` cvars early).
9. `Rand_Init(Sys_Milliseconds(true))` — `:1248`.
10. `Com_StartupVariable("developer")` — `:1251`.
11. `CL_InitKeyCommands()` — `:1254`. **[ded]** links `null_client.cpp:57` no-op.
12. `FS_InitFilesystem()` — `:1266` (`Com_Error(ERR_FATAL)` if `mpdefault.cfg` unreadable).
13. `Com_InitJournaling()` — `:1268` (LIFE-D4a: ported with slice 0).
14. `Cbuf_AddText("exec mpdefault.cfg\n")` — `:1270`.
15. Unless `Com_SafeMode()`: `exec jampserver.cfg` **[ded]** / `exec jampconfig.cfg` — `:1273-1277`.
16. `Cbuf_AddText("exec autoexec.cfg\n")` — `:1281`.
17. `Cbuf_Execute()` — `:1283`.
18. `Com_StartupVariable(NULL)` again — `:1286` (cmdline re-overrides configs).
19. **`com_dedicated` registration** — `:1288-1293`: `#ifdef DEDICATED` →
    `"2"`+`CVAR_ROM` (no runtime toggle) else `"0"`+`CVAR_LATCH`. Gates 33/35.
20. `Com_InitHunkMemory()` — `:1295` (Rust: owned-arena drops; §C9).
21. `cvar_modifiedFlags &= ~CVAR_ARCHIVE` — `:1299`.
22. Bulk `Cvar_Get` block (`com_maxfps`, `com_blood`, `logfile`, `timescale`,
    `fixedtime`, `viewlog`, `com_speeds`, `sv_running`, `cl_running`, RMG_*,
    `com_noErrorInterrupt`, …) — `:1304-1360`.
23. `if (com_dedicated->integer)` → force `viewlog=1` — `:1362`. **[ded]**
24. `if (com_developer->integer)` → register `error/crash/freeze` — `:1368`.
25. `Cmd_AddCommand("quit"/"changeVectors"/"writeconfig")` — `:1373`.
26. `com_version` cvar (ROM/serverinfo) — `:1377`.
27. `SE_Init()` (StringEd localization) — `:1380`.
28. `Sys_Init()` — `:1382` (OS/CPU detect; ends with `IN_Init()`, `win_main.cpp:1472`).
29. `Netchan_Init(Com_Milliseconds() & 0xffff)` — `:1383`.
30. **`VM_Init()`** — `:1384` (MP-only; registers `vm_game/vm_cgame/vm_ui`,
    zeroes `vmTable`). There is **no standalone `VM_Init` fn to freeze**: the
    `vmTable` becomes the empty **`ModuleRegistry`** owned by the engine
    module-host state (module-loading.md **LOAD-D5**), which is simply
    default-constructed empty (read module-loading.md — now Standing context — for
    its `{ slots: [Option<LoadedModule>; MAX_VM] }` shape); the `vm_*` cvars become
    the module transport-select cvars (module-loading territory). "Registration
    only" for slice 0 = construct that empty registry + register those cvars — a
    `ModuleRegistry::default()`-shaped empty build. Its **crate home is the engine
    module-host state, a field of the engine island gated by SEAM-Q8/STATE-Q1
    exactly like `Engine` itself**, so the `use` path (not the empty-construct
    shape) rides that cross-doc blocker (§ Slice hooks), not a lifecycle-inventable
    item. The `load_module` signature that later fills the registry is
    module-loading **LOAD-Q3** (open) and is not exercised until `SV_SpawnServer`
    loads the module (post-slice-0).
31. `SV_Init()` — `:1385` (serverinfo/systeminfo cvars, operator commands).
32. `com_dedicated->modified = qfalse` — `:1393`.
33. **`if (!com_dedicated->integer) { CL_Init(); Sys_ShowConsole(…); }`** —
    `:1394`. `CL_Init` (`cl_main.cpp:2549`): `Con_Init`, `CL_ClearState`,
    `CL_InitInput`, cl_* cvars, **`CL_InitRef()`** (`:2693` — wires `re` export
    table only; DEC-01 null-`refexport_t` stub), `SCR_Init`, `cl_running=1`.
    **[ded]** skipped entirely.
34. `com_frameTime = Com_Milliseconds()` — `:1402`.
35. `Com_AddStartupCommands()` — `:1406`; if false & not dedicated, queue intro
    cinematic (`:1409`, `#ifndef _DEBUG`). **[ded]** no cinematic.
36. `Cvar_Set("r_uiFullScreen","1")` — `:1423`.
37. **`CL_StartHunkUsers()`** — `:1425` (the *real* renderer/sound/UI start,
    gated on `com_cl_running`): `CL_InitRenderer` (`re.BeginRegistration`),
    `S_Init`, `S_BeginRegistration`, `CL_InitUI`. **[ded]** links
    `null_client.cpp:66` no-op (never starts renderer/sound).
38. `Cvar_Set("ui_singlePlayerActive","0")` — `:1428`.
39. `SH_Register()` under `#ifdef MEM_DEBUG` — `:1431` (dropped).
40. `com_fullyInitialized = qtrue` — `:1434`.
41. `Com_Printf("--- Common Initialization Complete ---\n")` — `:1435`.
42. `catch (const char* reason) → Sys_Error` — `:1439`.

### SP `Com_Init` — 34-step boot contract

`code/qcommon/common.cpp:950-1130`, `try { … } catch { Sys_Error(…); }`
(`:955`, `:1119`). Divergences from MP flagged inline.

1. Version banner — `:953`.
2. `Com_ParseCommandLine(cmdline)` — `:958`.
3. `Swap_Init()` — `:960` (**SP-only**).
4. `Cbuf_Init()` — `:961`.
5. `Com_InitZoneMemory()` — `:963`.
6. **`#ifdef _XBOX` early-renderer block — `:965-981`, DEAD on Win32** (DEC-01
   amendment; the SP renderer stub sits at `CL_Init`/`CL_InitRef`, step 27, same
   seam as MP — *no* early-boot renderer path is needed).
7. `Cmd_Init()` — `:983`.
8. `Cvar_Init()` — `:984` (**note: after `Cmd_Init`, unlike MP which does
   `Cvar_Init` before `Cmd_Init`**).
9. `Com_StartupVariable(NULL)` — `:987`.
10. `CL_InitKeyCommands()` — `:990`.
11. `#ifdef _XBOX` file-codes block — `:992-999` (dead on Win32).
12. `FS_InitFilesystem()` — `:1001`.
13. `R_InitWorldEffects()` — `:1002` (**SP-only** early var init).
14. Config execs: `exec default.cfg` (`:1004`); `exec jaconfig.cfg` unless
    `Com_SafeMode()` (`:1006`); `exec autoexec.cfg` (`:1011`); `Cbuf_Execute()` (`:1013`).
15. `Com_StartupVariable(NULL)` again — `:1016`.
16. `Com_InitHunkMemory()` — `:1019`.
17. `cvar_modifiedFlags &= ~CVAR_ARCHIVE` — `:1023`.
18. `Cmd_AddCommand("quit"/"writeconfig")` — `:1028`.
19. Core cvar block (`com_maxfps`, `logfile`, `speedslog`, `timescale`,
    `fixedtime`, `viewlog`, `com_speeds`, `sv_running`, `cl_running`,
    `skippingCinematic`, …) — `:1031-1053`. **No `dedicated` cvar.**
20. Dev commands `error/crash/freeze` if developer — `:1055`.
21. `com_version` — `:1061`.
22. `SE_Init()` — `:1064`.
23. `Sys_Init()` — `:1066` (ends with `IN_Init()`, `win_main.cpp:1112`).
24. `Netchan_Init(Com_Milliseconds() & 0xffff)` — `:1068`.
25. **`// VM_Init();`** — `:1069`, **fully commented out** (SP game is statically
    linked; DEC-07).
26. `SV_Init()` — `:1070`.
27. **`CL_Init()`** — `:1072` (unconditional, no dedicated gate):
    `Con_Init`, `CL_ClearState`, `CL_InitInput`, `RM_InitTerrain`, cvars,
    **`CL_InitRef()`** (`cl_main.cpp:1294`; DEC-01 null stub), **`CL_StartHunkUsers()`**
    (`:1296` — *inside* `CL_Init` in SP, vs a separate `Com_Init` step in MP),
    `SCR_Init` (`:1298`), `Cbuf_Execute`, `cl_running=1`.
28. `#ifdef _XBOX CL_StartSound()` — `:1078` (dead).
29. `Sys_ShowConsole(com_viewlog->integer, qfalse)` — `:1081`.
30. `com_frameTime = Com_Milliseconds()` — `:1086`.
31. `Com_AddStartupCommands()` — `:1090`; if none & `NDEBUG`, queue intro
    cinematic (`:1095`). The `//if (!com_dedicated->integer)` guard at `:1093` is
    **commented out**.
32. `com_fullyInitialized = qtrue` — `:1104`; banner — `:1105`.
33. `catch → Sys_Error("Error during initialization %s", reason)` — `:1119`.
34. `#ifdef _XBOX SE_CheckForLanguageUpdates()` — `:1127` (dead).

SP has **no journaling init** (§ Event system), **no `VM_Init`**, **no
`dedicated` cvar**, and folds `CL_StartHunkUsers` inside `CL_Init`.

### `Com_Frame` anatomy — MP (SP-diff inline)

MP `common.cpp:1593-1777`, `try { … } catch (const char* reason)
{ Com_Printf(reason); return; }` (`:1596`, `:1762`) — this catch is the
`ERR_DROP` recovery point. SP `common.cpp:1269-1463` (`:1270`, `:1449`).

1. `Com_WriteConfiguration()` — MP `:1624` / SP `:1278` (writes config if archive
   cvars changed).
2. `com_viewlog->modified` → `Sys_ShowConsole` toggle — MP `:1627` / SP `:1281`.
   The `modified` check and its `qfalse` reset run **unconditionally** (MP
   `:1627,:1632` / SP `:1281,:1283`); only the inner `Sys_ShowConsole` is
   `!com_dedicated`-gated (MP `:1628`; SP has no `dedicated` gate, `:1282`).
3. com_speeds: `timeBeforeFirstEvents` — MP `:1637` / SP `:1291`.
4. `minMsec = 1000/com_maxfps` — MP `:1642` (`1` if dedicated *or* timedemo) / SP
   `:1295` (`1` only if `com_maxfps<=0`; **no dedicated/timedemo guards**).
5. **Event pump + FPS-cap busy-spin** — MP `:1647` / SP `:1295`:
   `do { com_frameTime = Com_EventLoop(); msec = com_frameTime - lastTime; }
   while (msec < minMsec)`. Pure spin, no sleep.
6. `Cbuf_Execute()` — MP `:1654` / SP `:1307`; `lastTime = com_frameTime`.
7. `com_frameMsec = msec; msec = Com_ModifyMsec(msec[, &fraction])` — MP `:1659` /
   SP `:1314` (SP adds the `float&` fraction out-param; § Timing).
8. **`SV_Frame(msec[, fraction])`** — MP `:1669` / SP `:1323`.
9. **`com_dedicated->modified` hot-toggle** — MP `:1675-1687` only: →non-dedicated
   `CL_Init`+`CL_StartHunkUsers`; →dedicated `CL_Shutdown`. Dead on the ROM
   dedicated build; **lives in the `jamp` client binary** (LIFE-D2). Absent in SP.
10. `if (!com_dedicated->integer)` client block — MP `:1692-1716` (**[ded]**
    skipped) / SP `:1342` (**guard commented out → unconditional**): second
    `Com_EventLoop()`+`Cbuf_Execute()` ("server→client packets without a frame of
    latency"), then **`CL_Frame(msec[, fraction])`** — MP `:1711` / SP `:1362`.
11. com_speeds report — MP `:1721` / SP `:1373`; com_showtrace counters — MP
    `:1738` / SP `:1430`.
12. `com_frameNumber++` — MP `:1754` / SP `:1448`.

### Shutdown paths

- **`Com_Quit_f`** (clean exit) — MP `common.cpp:356` / SP `:332`: guard
  `!com_errorEntered` → `SV_Shutdown("Server quit\n")` → `CL_Shutdown()` →
  `Com_Shutdown()` → **MP-only `FS_Shutdown(qtrue)`** (`:362`; SP omits) →
  `Sys_Quit()` (unconditional, runs even in the recursive-error case).
- **`Com_Shutdown`** — MP `common.cpp:1785` / SP `:1495`: `CM_ClearMap`, close
  `logfile`, close journal handles, MP `MSG_shutdownHuffman`.
- **`Sys_Quit`** — MP `win_main.cpp:389`: `timeEndPeriod(1)`, `IN_Shutdown`,
  `Sys_DestroyConsole`, `Com_ShutdownZoneMemory`, `Com_ShutdownHunkMemory`,
  `exit(0)`.
- **`CL_Shutdown`** — MP `cl_main.cpp:2719`: recursion guard, `CL_Disconnect`,
  `CL_ShutdownRef()` **before** `CL_ShutdownAll()` ("so images get dumped"),
  `S_Shutdown`, cvar cleanup, `cl_running=0`, `memset(&cls,0)`.
- **Fatal path** (`Com_Error(ERR_FATAL)`) — MP `common.cpp:337` / SP `:313`:
  `CL_Shutdown` → `SV_Shutdown` → `Com_Shutdown` → `Sys_Error` — **no
  `FS_Shutdown`**, guard left set, no throw.
- **`NET_Shutdown`** — declared (`qcommon.h:133`), **zero call sites** in either
  tree; Winsock teardown relies on process exit (LIFE-D4c).

### Error recovery — per-level flow

Recursive guard `com_errorEntered` (MP `common.cpp:83` / SP `:68`): if already
set on `Com_Error` entry → `Sys_Error("recursive error after: …")` (MP `:288` /
SP `:265`). Set on entry, cleared per-branch after successful cleanup, left set
on the fatal path. Recovery runs **before** the throw (STATE-D3). `com_buildScript`
forces `ERR_FATAL` (MP `:270` / SP `:261`). **MP-only:** `FS_PureServerSetLoadedPaks("","")`
at entry (`:275`); rapid-error escalation — >3 errors within 100ms → force
`ERR_FATAL` (statics `:251-252`, logic `:277-286`). **SP-only:** unconditional
`SG_Shutdown()` before dispatch (`:283`).

**MP `errorParm_t` — 5 levels** (`codemp/game/q_shared.h:451-457`; recovery
`common.cpp:249-345`):

| Level | Recovery sequence (throws after) |
|---|---|
| `ERR_SERVERDISCONNECT` (`:302`) | `CL_Disconnect(qtrue)` → `CL_FlushMemory` → clear guard → throw `"DISCONNECTED\n"`. **Server NOT shut down.** |
| `ERR_DROP` / `ERR_DISCONNECT` (`:313`) | print ERROR banner → `SV_Shutdown` → `CL_Disconnect(qtrue)` → `CL_FlushMemory` → clear guard → throw `"DROPPED\n"` |
| `ERR_NEED_CD` (`:327`) | `SV_Shutdown` → if `com_cl_running`: `CL_Disconnect`/`CL_FlushMemory`/clear → throw `"NEED CD\n"` |
| `ERR_FATAL` (`:337`) | `CL_Shutdown` → `SV_Shutdown` → `Com_Shutdown` → `Sys_Error`. No throw, no recovery, guard stays set. |

**SP `errorParm_t` — 4 levels** (`code/game/q_shared.h:251-256`; recovery
`common.cpp:245-320`). No `ERR_SERVERDISCONNECT`; **SP re-runs `CL_StartHunkUsers()`
in every recoverable branch** (MP never does inside `Com_Error`); `CL_Disconnect()`
takes no arg:

| Level | Recovery sequence |
|---|---|
| `ERR_DISCONNECT` (`:284`) | `SV_Shutdown` → `CL_Disconnect` → `CL_FlushMemory` → `CL_StartHunkUsers` → clear → throw `"DISCONNECTED\n"`. (Despite the "don't kill server" comment, SP *does* shut it down.) |
| `ERR_DROP` (`:291`) | `SG_WipeSavegame("current")` → `SV_Shutdown` → `CL_Disconnect` → `CL_FlushMemory` → `CL_StartHunkUsers` → print ERROR banner (*after* cleanup) → clear → throw `"DROPPED\n"` |
| `ERR_NEED_CD` (`:302`) | `SV_Shutdown` → if `cl_running`: `CL_Disconnect`/`CL_FlushMemory`/`CL_StartHunkUsers`/clear → throw `"NEED CD\n"` |
| `ERR_FATAL` (`:313`) | `CL_Shutdown` → `SV_Shutdown` → `Com_Shutdown` → `Sys_Error` |

### Timing

- **`Sys_Milliseconds`** — MP `win_shared.cpp:22-34`: `timeGetTime()` minus a
  static `sys_timeBase` (captured at first call); `baseTime=true` skips the
  subtraction (used at `Rand_Init(Sys_Milliseconds(true))`, `common.cpp:1248`).
  SP `code/win32/win_shared.cpp:17-25`: `int Sys_Milliseconds(void)`, no raw
  variant.
- **`Com_ModifyMsec` clamps** — MP `common.cpp:1534-1578`: `com_fixedtime`
  overrides; else `msec *= com_timescale` (floor `1` when timescale nonzero);
  ceiling = dedicated **5000** (+"Hitch warning" if msec>500) / remote-server
  client (`!com_sv_running`) **5000** / local-hosted **200**. SP
  `common.cpp:1197-1247`: adds a `float& fraction` out-param accumulating the
  truncated fractional msec; ceiling = `com_skippingcin ? 500 : 200` (**no 5000
  paths** — no dedicated/remote concept).
- **`sv_fps` decoupled tick** — `SV_Frame` (`sv_main.cpp:847`): `frameMsec =
  1000/sv_fps`; **dedicated only**: `timeResidual < frameMsec` → `NET_Sleep(…)`
  and return (`:856`) — the real dedicated throttle. SP mirrors `:500`.

### Event system

- **256-entry `sysEvent_t` ring** (`Sys_QueEvent`/`Sys_GetEvent`) —
  `win_main.cpp:1162-1166`: `MAX_QUED_EVENTS 256`, `eventQue[256]`, monotonic
  `eventHead`/`eventTail`; overflow drops the oldest (freeing its `evPtr`);
  `time==0` → stamped `Sys_Milliseconds()`. This is **distinct** from the
  1024-entry `com_pushedEvents` `Com_PushEvent` ring (`common.cpp:749-752`,
  owned as `Common.event_queue` in state-ownership).
- **`Com_EventLoop`** (`common.cpp:921`) pulls `Com_GetEvent()` (drains
  `com_pushedEvents` first, then `Com_GetRealEvent()`); on `SE_NONE` drains
  loopback packets and returns `ev.evTime` (→ `com_frameTime`); dispatches
  `SE_KEY`→`CL_KeyEvent`, `SE_CHAR`→`CL_CharEvent`, `SE_MOUSE`→`CL_MouseEvent`,
  `SE_JOYSTICK_AXIS`→`CL_JoystickEvent`, `SE_CONSOLE`→strip `\`/`/` + `Cbuf_AddText`
  (`:969`), `SE_PACKET`→dispatch; unknown type → `Com_Error(ERR_FATAL)`.
- **`Sys_GetEvent`** (Raven `win_main.cpp:1211`) drains the ring, *then* pumps
  `PeekMessage`/console/net inside the getter — the poll-style model DEC-02
  inverts (below).
- **Journaling (MP only)** — `Com_InitJournaling` (`common.cpp:759`): `journal`
  cvar (`CVAR_INIT`); `1`→write `journal.dat`, `2`→read/replay. Tap in
  `Com_GetRealEvent` (`:789`): `journal==2` `FS_Read`s the next `sysEvent_t`
  (+`evPtrLength` payload) *instead of* `Sys_GetEvent`; `journal==1` `FS_Write`s
  every real event. **SP: stripped** — handles declared/closed but no cvar, no
  init, `Com_GetRealEvent` (`:699`) is unconditionally `Sys_GetEvent()`.

### Dedicated split (`jampded`)

Raven splits `jampded` **both** at link time (the `null_*` stubs) and compile
time (`DEDICATED` macro → ROM `com_dedicated="2"`, `WinDed.vcproj:32`); either
alone would suffice (§2.3). The dedicated target links `null_client.cpp`,
`null_glimp.cpp`, `null_input.cpp`, `null_renderer.cpp`, `null_snddma.cpp`,
`null/win_main.cpp` **plus real** `win32/win_net.cpp`+`win_shared.cpp`
(`WinDed.vcproj:379-394`) — networking and OS glue are *not* nulled.
`null_client.cpp` stubs the client tier symbols `common.cpp` calls
unconditionally (`CL_Init`/`CL_Shutdown`/`CL_Frame`/`CL_StartHunkUsers`/… →
no-ops; `null_client.cpp:9-66`). **SP has no such stubs** (`code/null/` lacks
`null_client/input/renderer`) — no client-less SP target exists.

- **`Sys_ConsoleInput`** (`null/win_main.cpp:200-302`): non-blocking
  `kbhit`/`getch` poll with a hand-rolled line editor (history, backspace, Tab
  completion via `Cmd_/Cvar_CommandCompletion`, Esc, Enter, Ctrl-V). Returns
  NULL until a full line is ready; a completed line is queued as
  `Sys_QueEvent(0, SE_CONSOLE, …)` (`:1195`) and consumed by `Com_EventLoop`'s
  `SE_CONSOLE` case that frame.

## State ownership

Per DEC-04 the tables are per-mode; MP shown, SP mirror under `sp/engine/*`
(deltas: no journaling, no `dedicated`, no rapid-error escalation). The
**lifecycle-owned** rows below extend `Common` (`mp/engine/qcommon`).
`state-ownership.md`'s master table fixes only their **owner** (`Common.field`);
the internal field *types* are qcommon subsystem detail (state-ownership treats
each owned struct's field list as a non-goal), i.e. mechanical §C ports of the
cited Raven globals — **not** independently "frozen" elsewhere, so they are stated
inline here so a slice-0 skeleton can build:

- `frame_{time,msec,number}: i32` — Raven `int` (`common.cpp:79-81`).
- `error` — `entered: bool` (`qboolean`) + `message: [u8; MAXPRINTMSG]`
  (`char[MAXPRINTMSG]`, `MAXPRINTMSG=4096`, `common.cpp:18,83,86`) + **MP-only**
  `last_error_time: i32`/`error_count: i32` (the rapid-error statics
  `common.cpp:251-252`).
- `journal` (**MP only**) — two `fileHandle_t` (= `int` → `i32`,
  `q_shared.h:362`) handles `file`/`data_file` (`common.cpp:34-35`) + the
  `com_journal` mode `i32` (the `journal` cvar, `common.cpp:761`).
- `sys_events: SysEventQueue` — frozen in this doc's § Seam (below).

This doc fixes their **construction order** (the `Com_Init` step that builds them)
and **mutation points** (the `Com_Frame`/`Com_Error` sites that write them). Owner =
`crate::Type.field`; threaded via `&mut Engine` (STATE-D1 reborrow) unless noted.

| Raven global | oracle cite | Rust owner | constructed by | mutated at |
|---|---|---|---|---|
| `com_frameTime`/`com_frameMsec`/`com_frameNumber` | MP `common.cpp:79-81` | `Common.frame_{time,msec,number}: i32` (types: preamble) | `Com_Init` step 34 (MP `:1402`) | `Com_Frame` 5/7/12 |
| `com_errorEntered`/`com_errorMessage[4096]` | MP `common.cpp:83,86` | `Common.error` (`entered: bool`/`message: [u8;4096]`, preamble) | `Com_Init` | `Com_Error` entry/branch clears — STATE-D3 |
| MP rapid-error `lastErrorTime`/`errorCount` (100ms/3 escalation) | MP `common.cpp:251-252` | `Common.error` **MP-only fields** (rule §B3 — no statics) | `Com_Init` | `Com_Error` `:277-286` |
| `com_journalFile`/`com_journalDataFile` | MP `common.cpp:34-35` | `Common.journal` (**MP only**) | `Com_Init` step 13 `Com_InitJournaling` | `Com_GetRealEvent` tap |
| `eventQue[256]`/`eventHead`/`eventTail` (`Sys_QueEvent` ring) | MP `win_main.cpp:1162-1166` | `Common.sys_events: SysEventQueue` (**new; distinct from the 1024 `com_pushedEvents`**) | `Com_Init` (empty ring) | winit/console adapters `queue()`; `Sys_GetEvent` drains |
| `sys_timeBase` (Sys_Milliseconds base) | MP `win_shared.cpp:22-34` / SP `:17-25` | `native/platform` monotonic clock (Instant base) — **constructed at the warm-up point (MP `win_main.cpp:1545`), before `Com_Init` and before `Engine`, so it is NOT an `Engine` field and needs no pre-existing Engine** (Raven captures the static base lazily on first call, `win_shared.cpp:22-34`); "threaded via `Engine`" = Engine holds a handle to the already-constructed platform clock. Exact field placement is a platform-doc detail (LIFE-D4b); Engine's handle to it is part of the STATE-Q1 construction (§ Slice hooks). No ordering paradox. | entry-point warm-up (MP `win_main.cpp:1545`) | read-only after capture |

Everything else the A3 survey touched (`sv`/`svs`, `cl`/`clc`/`cls`, cvar/cmd/fs
tables, sound, module VM handles) is owned by **state-ownership.md** — see its
master table; not duplicated here.

## Seam definition

FROZEN — porters fill bodies without changing these shapes. Per LIFE-D2 the three
`com_*` functions are ported **per-mode** into `mp/engine/qcommon` /
`sp/engine/qcommon`; there is **no shared `Lifecycle` trait**. `Engine` is the
aggregate engine-island struct (state-ownership STATE-D1; its defining crate is
STATE-Q1, open — these signatures hold under either home).

**What STATE-Q1 blocks vs. what is pinned.** These signatures freeze regardless of
STATE-Q1. What STATE-Q1 (cross-doc, open) still gates is the *concrete* `use
Engine` import path, hence `mp/app`'s `[dependencies]`/`[features]` edges and the
`dedicated`-feature client-tier exclusion (§ Binary packaging) — none derivable
from oracle (Raven is C, no crates), all resolving when STATE-Q1 does. **Pinned
now** (settled elsewhere, not blocked): the `com_*` free functions port from
`common.cpp` into the **new `common` module** of `mp_engine_qcommon` that
state-ownership.md adds (alongside `cm`/`files`/`gp2`/…); CLAUDE.md's
one-type-per-file rule governs *types* — `Common`, `SysEventQueue`, `Journal` each
get their own file — while the free `com_*`/`Com_*` lifecycle functions ported from
one `.cpp` colocate in that module (a `common/lifecycle.rs` submodule is the
mechanical layout, not an architectural choice). Only **these three** types get a
file: the `error` and `frame_{time,msec,number}` field groups in the § State
ownership table are **`Common`'s own fields, not separate types** — there is **no
`ErrorState` type**; grouping `error`'s subfields into an inline sub-struct is
itself a mechanical, not file-owning, choice (state-ownership.md fixes their owner
as `Common.error` and treats the field list as subsystem detail). The **exact
submodule filenames are the porter's mechanical choice, not frozen**: one-type-per-file
yields a file each for `Common`/`SysEventQueue`/`Journal` and the free
`com_*`/`Com_*` fns colocate in a `lifecycle.rs`-shaped submodule; a dry-run need
not treat any specific filename set (e.g. an `error.rs`) as a doc-frozen list.

`com_printf` is the qcommon `Com_Printf` port (`mp/engine/qcommon`,
`common.cpp:128`); `sys_error` is `Sys_Error` (declared `qcommon.h:966`,
implemented in the platform entry shell — `win32/win_main.cpp:350`, dedicated
`null/win_main.cpp:324` → `mp/app`'s `sys` layer). Both are **slice-0
deliverables** (unported today), like `errorParm_t`/`sysEvent_t` are pre-existing
— see § Slice hooks. Their **signature shapes are pinned by the frozen bodies that
call them** (style rule 5): `com_frame`'s catch calls `com_printf(engine, &e.msg)`
and `com_init`'s init-catch calls `sys_error(engine, &e.msg)`, so:

```rust
/// Raven `Com_Printf` (`common.cpp:128`). Threads `&mut Engine` — mutates the
/// redirect buffer (`rd_buffer`), console, and the lazily-opened `logfile`
/// (`common.cpp:137-171`), all `Common` state.
pub fn com_printf(engine: &mut Engine, msg: &str);

/// Raven `Sys_Error` (`win32/win_main.cpp:350`; dedicated `null/win_main.cpp:324`).
/// Noreturn (`-> !`) — Raven ends in `exit(1)` after console teardown + `IN_Shutdown`;
/// the fatal escalation point for `com_init`'s init-catch.
pub fn sys_error(engine: &mut Engine, msg: &str) -> !;
```

Like the four `com_*` above, these shapes hold under either STATE-Q1 resolution;
what stays open is the concrete crate wiring — for `sys_error` specifically the
qcommon→app-shell **reverse dependency** (it is *declared* in `qcommon.h` but
*implemented* in `mp/app`'s platform shell, so `com_init` in `mp/engine/qcommon`
reaching it is entangled with STATE-Q1's crate shape, § Slice hooks) — not the
signatures.

### The `com_*` entry surface (per mode)

```rust
// mp/engine/qcommon (SP mirror: sp/engine/qcommon). NOT a trait — per-mode fns.
// `errorParm_t` is the EXISTING ported enum (mp_qshared::errorParm_t, 5 variants;
// sp_qshared::errorParm_t, 4 variants) — this is state-ownership's `ErrorLevel`.

/// Raven `Com_Init` (MP `common.cpp:1216` / SP `:950`). Runs the boot contract;
/// a ComError panic during init is caught here and escalated to fatal
/// (mirrors the `catch → Sys_Error` at MP `:1439` / SP `:1119`, LIFE-D3).
pub fn com_init(engine: &mut Engine, command_line: &str);

/// Raven `Com_Frame` (MP `common.cpp:1593` / SP `:1269`). One frame; the
/// `catch_unwind` boundary (DEC-08 / SEAM-D10) wraps the body — a ComError is
/// printed and the frame returns (the ERR_DROP recovery point, MP `:1762`);
/// any non-ComError panic (a genuine Rust bug) is re-raised as fatal (LIFE-D3).
pub fn com_frame(engine: &mut Engine);

/// Raven `Com_Shutdown` + `Com_Quit_f` orchestration (MP `common.cpp:356,1785`).
pub fn com_shutdown(engine: &mut Engine);

/// Raven `Com_Error` (MP `common.cpp:249` / SP `:245`). Runs the per-level
/// recovery SYNCHRONOUSLY, then panics — recovery-before-panic is STATE-D3
/// (cross-ref, not re-decided here). Diverges: ERR_FATAL / init-time never
/// return (they abort); recoverable levels never return normally either.
pub fn com_error(engine: &mut Engine, level: errorParm_t, msg: String) -> !;
```

`com_frame`'s catch boundary (the only new-code control structure):

```rust
pub fn com_frame(engine: &mut Engine) {
    use std::panic::{catch_unwind, AssertUnwindSafe, resume_unwind};
    match catch_unwind(AssertUnwindSafe(|| com_frame_body(engine))) {
        Ok(()) => {}
        Err(p) => match p.downcast::<ComError>() {
            Ok(e) => com_printf(engine, &e.msg),   // ERR_DROP recovery point; return
            Err(other) => resume_unwind(other),    // real Rust bug → fatal (LIFE-D3)
        }
    }
}
```

`com_init` wraps `com_init_body` identically but routes a caught `ComError` to
`sys_error` (fatal), matching Raven's init catch. `ComError` and its recovery
ordering are frozen in `state-ownership.md` (§ Seam, STATE-D3); this doc only
names `errorParm_t` as its `level` type. Concretely, that frozen block is
`pub struct ComError { level: ErrorLevel, msg: String }` in **`mp/engine/qcommon`**
— the same crate as `com_frame`, so the `downcast::<ComError>()` above resolves the
type locally with no cross-crate `use`. Those two fields are **exhaustive** (the
`{level, msg}` the recovery snippet reads is the whole payload), and it needs **no
derive**: a `panic_any`/`downcast` payload only requires `Any + Send + 'static`,
which the enum + `String` satisfy automatically. `ErrorLevel` is per-mode
`errorParm_t` (LIFE-D3, below).

**Helpers inside `com_frame_body` are NOT part of this frozen entry surface** — only
the four `com_*` functions above freeze (LIFE-D2). The event/journal helpers the
body calls are private, their signatures mechanical §C ports of the cited Raven
functions (all thread `&mut Engine` per STATE-D1):

- `fn com_event_loop(engine: &mut Engine) -> i32` — Raven `int Com_EventLoop(void)`
  (`common.cpp:921`); returns the last event time (→ `com_frameTime`), the value
  the FPS-cap spin (`Com_Frame` step 5) reads.
- `fn com_init_journaling(engine: &mut Engine)` — Raven `Com_InitJournaling(void)`
  (`common.cpp:759`; MP only); reads the `com_journal` `CVAR_INIT` cvar and, per
  its `1`/`2` mode, opens `journal.dat`/`journaldata.dat`, writing the two
  `fileHandle_t` handles + mode into `Common.journal` (types: § State ownership).
- `fn com_get_real_event(engine: &mut Engine) -> sysEvent_t` — Raven
  `Com_GetRealEvent(void)` (`common.cpp:789`); the journaling tap. On disk each
  record is a raw `sysEvent_t` followed by its `evPtrLength` payload
  (`common.cpp:789-800`); that byte format is golden-diffed, not a typed API
  (§ Verification 1).

### `SysEventQueue` — the 256-entry `Sys_QueEvent` ring

```rust
// mp/engine/qcommon (SP mirror). Faithful queue semantics of eventQue[256]
// (win_main.cpp:1162-1203). NOT the 1024-entry com_pushedEvents ring.
pub const MAX_QUED_EVENTS: usize = 256;

pub struct SysEventQueue {
    que:  [sysEvent_t; MAX_QUED_EVENTS],   // sysEvent_t: mp_engine_qcommon (already ported)
    head: usize,                            // monotonic; & (MAX_QUED_EVENTS-1) to index
    tail: usize,
}

impl SysEventQueue {
    /// `Sys_QueEvent` (win_main.cpp:1178). `time==0` → stamp Sys_Milliseconds();
    /// overflow drops the oldest (frees its evPtr). Called by the platform adapters.
    pub fn queue(&mut self, time: i32, ty: sysEventType_t, value: i32, value2: i32,
                 ptr: Option<Box<[u8]>>);
    /// `Sys_GetEvent` reduced to a PURE ring-drain — NO OS pump inside (DEC-02
    /// inversion). Returns a synthesized `SE_NONE` stamped `Sys_Milliseconds()`
    /// when empty (win_main.cpp:1270).
    pub fn get(&mut self, now_ms: i32) -> sysEvent_t;
}
```

### winit-adapter boundary (`jamp`/`jasp`, LIFE-D1)

DEC-02 inverts Raven's poll model: winit **owns** the loop; the app does not.
The boundary contract (designed now, first exercised when the client slice lands
— DEC-01 keeps both binaries headless until then):

- The `ApplicationHandler` runs with `ControlFlow::Poll`.
- `about_to_wait` calls `com_frame(&mut engine)` — reproducing Raven's
  `while(1){ …Com_Frame(); }` cadence.
- `window_event`/`device_event` callbacks **translate** each winit event into a
  `sysEvent_t` and call `SysEventQueue::queue(...)` (the `Sys_QueEvent`
  equivalent) — key→`SE_KEY`, text→`SE_CHAR`, pointer motion→`SE_MOUSE`,
  axis→`SE_JOYSTICK_AXIS`. The producer-side `sysEventType_t`/`evValue`
  conventions are Raven's (`qcommon.h:923-932`); the winit-keycode→`keynum_t`
  map is new code owned by the platform/input layer, **not frozen here** (LIFE-Q1).
- `SysEventQueue::get` (hence `Com_EventLoop`) is a pure drain — no
  `PeekMessage`/pump inside the getter (the inversion; dossier §7).
- The FPS-cap busy-spin (`Com_Frame` step 5) stays a **spin** that re-drains the
  ring (LIFE-D1; `WaitUntil` rejected as timing-divergent).

### `jampded` console adapter (LIFE-D4d)

No winit. The OS loop is `loop { com_frame(&mut engine) }` with the `Sleep(5)`
entry pacing (`null/win_main.cpp:1478`). A faithful minimal polled line editor
over raw stdin (the `Sys_ConsoleInput` behavior, `null/win_main.cpp:200-302`)
feeds completed lines as `SE_CONSOLE` events through `SysEventQueue::queue`; net
poll feeds `SE_PACKET`. Same ring, same `Com_EventLoop` `SE_CONSOLE` handling.

### Binary packaging (LIFE-D2)

- **`crates/mp/app`** — two `[[bin]]` targets: `jamp` (default: client tier;
  conventional path `src/bin/jamp.rs`) and `jampded` (`src/bin/jampded.rs`). The
  `cl`/`snd` **field types are not feature-dependent**: state-ownership.md freezes
  `Engine { cl: Option<Client>, snd: Option<SoundSystem>, … }` (STATE-D1); `jampded`
  simply constructs them **`None`**. Because *every* client/renderer/sound call in
  `com_*` is gated on `com_dedicated` (Raven-faithful — `Com_Init` step 33,
  `Com_Frame` step 10, `common.cpp:1394,1692`), a `None` client is **never
  dereferenced at runtime**, so **no null-stub client objects are needed** — the
  `Option` + the runtime gate together reproduce `null_client.cpp`'s "the calls are
  no-ops" behavior. The `dedicated` cargo **feature** therefore serves the
  *build-level* purpose of Raven's `null_*` link swap — a **client-less binary**:
  it compile-excludes the client-tier crates (`mp/engine/client` + renderer +
  sound) from `jampded`, not substitutes stub field types. **The exact feature
  wiring** — how the client tier is `cfg`-excluded while the `Engine` type still
  names `Option<Client>` — depends on where `Engine` is defined and is entangled
  with **STATE-Q1** (cross-doc, open; § Slice hooks), so the concrete
  `[features] dedicated = […]` + dependency edges are not pinned here. The
  **runtime** `com_dedicated` hot-toggle (`common.cpp:1675-1687`) stays compiled
  **only in the `jamp` binary** (dead on Raven's ROM dedicated build).
- **`crates/sp/app`** — one `[[bin]]`: `jasp` (`src/bin/jasp.rs`). No dedicated
  variant (SP has none).

## Decisions

**LIFE-D1 — Loop ownership, per target.** `jamp`/`jasp`: **winit (DEC-02) owns
the loop** — an `ApplicationHandler` with `ControlFlow::Poll`, `com_frame()`
called from `about_to_wait`, winit events translated into the ported 256-entry
`sysEvent_t` ring (faithful queue semantics; `Sys_GetEvent` becomes a pure
ring-drain). The FPS-cap busy-spin is preserved as a spin that pumps/re-drains
events (parity-faithful timing). `jampded`: plain `loop { com_frame() }` with the
`Sleep(5)` entry pacing and polled console input. Renderer deferral (DEC-01)
means the windowed path is *designed now, first exercised later* — early client
slices boot headless. *Because* Raven's architecture fixes the adapter direction
(callbacks enqueue; frame work runs from `about_to_wait`). *Rejected:*
`ControlFlow::WaitUntil` for the FPS cap (diverges in event-arrival timing);
`pump_app_events` loop-shape preservation (platform-limited/second-class, dossier
§7).

**LIFE-D2 — Skeleton + packaging.** `com_init`/`com_frame`/`com_shutdown` are
ported **per-mode** into the `{mp,sp}` engine `qcommon` crates — **no shared
`Lifecycle` trait** — because they are genuinely divergent (NET_Init location,
CL_Init gating, `CL_StartHunkUsers` position, journaling, msec clamps). Binary
shells are thin new-code mains: `mp/app` gets two `[[bin]]` targets (`jamp`,
`jampded`) with a `dedicated` cargo feature swapping the client tier for null
stubs (mirrors `DEDICATED` + `null_*` substitution); the runtime dedicated-toggle
(`common.cpp:1675-1687`) stays in the client binary. `sp/app` is one bin.
*Because* DEC-04 mandates per-mode duplication during porting and the divergences
are real. *Rejected:* a shared generic `Lifecycle`/host trait (would force the
three shapes to converge against oracle); a single binary with runtime `+set
dedicated` only (Raven's dedicated is compile-time ROM — parity).

**LIFE-D3 — Error payload taxonomy.** `com_error(engine, level, msg)` runs the
per-level recovery **synchronously**, then `panic_any(ComError{level, msg})` —
recovery-before-panic is STATE-D3 (cross-ref). `com_frame`'s `catch_unwind`
prints the message and continues (the `ERR_DROP` recovery point); a `catch` also
guards `com_init`, escalating to fatal (mirrors MP `:1439` / SP `:1119`).
`ErrorLevel` is the **existing per-mode `errorParm_t`** (MP 5 variants incl.
`ERR_SERVERDISCONNECT`/`ERR_NEED_CD`; SP 4 — DEC-04). `com_errorEntered` and MP's
3-errors-in-100ms escalation are owned `Engine.common` fields (§B3 — no statics).
Panics **without** our `ComError` payload (genuine Rust bugs) are re-raised as
fatal via `resume_unwind` — **never** swallowed into the `ERR_DROP` path.
Requires `panic="unwind"`; seam exports are `extern "C-unwind"` (SEAM-D10).
*Because* Raven's `throw`/`catch` maps 1:1 (dossier §4.1). *Rejected:* moving
recovery into the catch site (would let the handler observe half-torn state,
STATE-D3); one shared enum (loses the MP/SP structural difference).

**LIFE-D4 — Small-fork bundle.** (a) **Journaling** (MP-only `Com_InitJournaling`
+ the `Com_GetRealEvent` tap) is ported **with slice 0** — the cheapest
determinism/replay lever for DEC-09 differential runs. (b) **`Sys_Milliseconds`**
is backed by `std::time::Instant` behind the same base-relative `i32` API
(internal monotonic `u64`; the `timeGetTime` 49.7-day wrap documented at the
seam); MP keeps the `baseTime` bool parameter, SP the `void` signature (DEC-04).
(c) **`NET_Shutdown`**: Raven declares it and never calls it — we do idiomatic
`Drop` teardown with a 1-line divergence note at the site. (d) **`jampded`
console input**: a faithful minimal polled line editor over raw stdin feeding
`SE_CONSOLE` events into the same queue. *Rejected:* stubbing journaling to
`journal=0` (loses the replay lever early); a modern line editor for `jampded`
(the seam is `SE_CONSOLE` either way — stay faithful).

## Verification strategy

Per DEC-09, this doc's live surface is boot/frame/shutdown *observable behavior*:

1. **Journaled event streams (DEC-09.1 + LIFE-D4a).** With `journal=1` the ring
   is captured; `journal=2` replays it, driving `Com_Frame` deterministically —
   the reproducible-event-stream lever for differential runs against the oracle.
   The `sysEvent_t` write/read format (`+evPtrLength` payload, `common.cpp:789`)
   is golden-diffed against the oracle TU.
2. **Boot-transcript diffing (DEC-09.2).** Each target's console/banner output
   (version banner, `Com_Init` step prints, `--- Common Initialization Complete
   ---`) is diffed against the reference binary (`jampded`/OpenJK for MP, retail
   `jasp` for SP) — the cheapest end-to-end boot-order check.
3. **TU goldens (DEC-09.1).** `Com_ModifyMsec` clamp logic (MP 5000/5000/200 vs
   SP 500/200, incl. the fraction accumulator) and `SysEventQueue`
   push/drain/overflow semantics compile standalone and golden-diff against the
   oracle (`tools/*-oracle` pattern).

Native-track (§E): green at every commit, one fn/struct/file per commit,
slice-driven. `SysEventQueue`/`ErrorLevel` layout parity rides the existing
`#[repr(C)]` `size_of`/`offset_of!` asserts on `sysEvent_t` (§D12, green on build).

## Slice hooks

**Slice 0 — `jampded` bin.** `main` → construct the one `Engine` (dedicated: a
value with `cl: None, snd: None`; its *constructor* is STATE-Q1-gated — see
cross-doc blockers below) → `com_init(engine)` → `SV_Init` path → `loop { com_frame }`
idle with polled console input. Depends on: engine-seam.md (dispatchers — none exercised at
idle until a `map` command loads the module), state-ownership.md (`Engine`/`Common`
shapes, `ComError`), and this doc's `com_*` seam + `SysEventQueue`.

**Cross-doc blockers (tracked elsewhere, not lifecycle decisions).** The settled
signatures here still depend on external open items before slice 0 can write its
final crate wiring — surfaced so the porter treats them as known dependencies, not
doc defects: **STATE-Q1** (state-ownership.md — the crate that *defines* `pub struct
Engine`) gates the `use Engine` path, `mp/app`'s dependency/`[features]` edges, and
the `dedicated` client-tier exclusion (§ Binary packaging). **STATE-Q1 also gates
`main()`'s Engine-construction call** — the `let mut engine = …;` a slice-0 skeleton
needs *before* `com_init(engine, …)`: state-ownership.md freezes `pub struct
Engine`'s field list but **no constructor** (`Default`/`new`, and the initial build
of each field sub-struct — `Common`, `sv`/`cl`/`snd: None`, `cm`), so how the first
Engine is built is a sub-item of STATE-Q1's home decision, **not lifecycle-inventable**
(Raven is C, static zero-init — no oracle constructor exists); it resolves when
STATE-Q1 picks Engine's defining crate. The empty step-30 `ModuleRegistry` (§
`Com_Init` step 30) rides the same blocker. **LOAD-Q3**
(module-loading.md — the `load_module` signature) gates the registry-fill path,
which slice 0 does not exercise at idle. Both are owned and escalated in their home
docs; the frozen `com_*`/`SysEventQueue`/`ComError` shapes hold under either
resolution. Which of the 42 MP `Com_Init` steps slice 0 **implements** vs
**stubs**:

- **Implements:** 1-10 (banner, push-event, cvar/cmd/cbuf, cmdline, zone→arena,
  rand, startup vars), 12 (`FS_InitFilesystem`), **13 `Com_InitJournaling`**
  (LIFE-D4a), 14-18 (config execs incl. `jampserver.cfg`, `Cbuf_Execute`,
  re-override), **19 `com_dedicated="2"` ROM**, 20 (hunk→arena), 21-26 (cvar
  block, viewlog force, quit/writeconfig, version), 28 (`Sys_Init` — CPU detect;
  `IN_Init` = null stub), 29 (`Netchan_Init`), **30 `VM_Init`** (= empty
  `ModuleRegistry` + transport-select cvars, step 30 above; module *load* is
  post-slice-0, at `SV_SpawnServer`), 31 (`SV_Init`), 32,
  34 (`com_frameTime`), 35 (`Com_AddStartupCommands`; no cinematic), 40-41
  (`fullyInitialized`, banner).
- **Stubs / no-op (dedicated null tier):** 11 `CL_InitKeyCommands`
  (`null_client.cpp:57`), **33 `CL_Init` skipped** (dedicated gate), **37
  `CL_StartHunkUsers`** (`null_client.cpp:66` no-op — no renderer/sound), 36/38
  (client-only cvars, harmless), 39 (`SH_Register`, dropped).
- **Needs a real subsystem, minimal for boot:** 27 `SE_Init` (StringEd) — a
  **no-op stub is behavior-faithful for slice 0**: `SV_Init` has **zero `SE_` call
  sites** (`sv_init.cpp`, grep-confirmed — its one `SE_` hit is a `USE_CD_KEY`
  block, not compiled), and no dedicated-boot path consumes a localized string, so
  nothing at slice-0 idle boot exercises StringEd; the full port is a subsystem
  non-goal (`docs/subsystems/*`). 6/20 zone/hunk (`Com_InitZoneMemory`/
  `Com_InitHunkMemory`) construct **no arena** for slice 0: per **STATE-D4** + §C9,
  `TheZone`/hunk are *not ported* — `Z_Malloc`/hunk callers become ordinary owned
  `Vec`/`Box`, and slice-0 dedicated idle boot has no observable hunk clear-point
  (map load is post-slice-0), so both steps are no-ops. A concrete arena
  type/API is a §C9 discussion-time item **only if** a later slice surfaces an
  observable clear-point (STATE-D4 "where observable") — not a slice-0 blocker.
- **`Com_Frame` at idle** (per-step, mirroring the `Com_Init` classification):
  step **1 `Com_WriteConfiguration`** is **implemented** — unconditional core
  (`common.cpp:1624`), its deps (FS, cvars) are slice-0 steps and its
  `USE_CD_KEY`/UI client block is `#ifndef DEDICATED` (so absent on the dedicated
  build); guards on `com_fullyInitialized` + `cvar_modifiedFlags & CVAR_ARCHIVE`,
  then dedicated writes `jampserver.cfg` (`common.cpp:1471-1495`). Step **2 viewlog
  toggle** — the `modified` check + its `qfalse` reset are **implemented**
  (unconditional, `:1627,:1632`); the inner `Sys_ShowConsole` is `[ded]`-skipped
  (`:1628`). Steps **3-8** run (event spin, `Cbuf_Execute`, `Com_ModifyMsec`,
  `SV_Frame` — which `NET_Sleep`s when `timeResidual < frameMsec`). Step **9** (the
  `com_dedicated` hot-toggle) is **absent from `jampded`** (jamp-binary-only,
  LIFE-D2). Step **10** (client block) is `[ded]`-skipped; step **11**
  (com_speeds/showtrace report) is a no-op at idle (`com_speeds=0`, harmless).
  Step **12 `com_frameNumber++`** runs.

**Later slices.** `jamp` adds `cl: Some(Client)` (+ the winit adapter, the
`com_dedicated` hot-toggle) and `snd`; `jasp` adds its unconditional `CL_Init`
(no dedicated gate) and the SP `Com_ModifyMsec` fraction accumulator. The
winit-keycode map (LIFE-Q1) is needed by the first `jamp` slice.

## Open questions

- **LIFE-Q1 — the winit event → `sysEvent_t` translation table.** LIFE-D1 fixes
  the *direction and semantics* (callbacks enqueue via `Sys_QueEvent`;
  `about_to_wait` runs `com_frame`; the getter is a pure drain) and the
  producer-side `sysEventType_t`/`evValue` conventions are Raven's
  (`qcommon.h:923-932`). But the concrete map from winit's key/pointer/text event
  vocabulary to Raven's `keynum_t` and to `evValue`/`evValue2` is **new code with
  no oracle ground truth** (Raven's producers are Win32 DirectInput,
  `win_input.cpp:413-994`) and is not enumerated in the settled inputs. It is
  first exercised by the `jamp` client slice, not slice 0; its natural home is
  the platform/input subsystem doc (pending) in concert with this boundary
  contract. Escalated — not decided here.
