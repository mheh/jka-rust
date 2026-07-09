# Lifecycle Design
Status: FROZEN (user sign-off 2026-07-03)     Supersedes: none
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
  the `catch_unwind` boundary sits at engine `Com_Frame`, outside the exports),
  and **SEAM-D7** (the sibling-set definition of **"Slice 0 (MP dedicated boot)"**
  = our `jampgame` cdylib hosted **inside a real/OpenJK C engine** on the
  `NativeDll` transport; that slice does **not** construct our `Engine`/`Server`
  or run `com_init`/`com_frame` — those are the *later our-engine-hosting slice*).
  This doc supplies the boot order that reaches those dispatchers **and renders
  that later our-engine-hosting slice** (the `jampded` Rust binary — `Engine::new`
  + `com_init` + the `com_frame` loop), which lands **after** SEAM-D7's Slice 0;
  see § Slice hooks (Terminology reconciliation).
- `docs/architecture/module-loading.md` — **LOAD-D5** (the per-slot
  `ModuleRegistry` the `Com_Init` step-30 `VM_Init` constructs — default-constructed
  empty — and **LOAD-D8**, which froze its `load_module` fill-signature —
  `ModuleRegistry::load_module(&mut self, policy, name, syscall) -> SlotId`,
  resolving the former LOAD-Q3, 2026-07-02). This doc names
  the empty-registry construct as a boot step; that doc owns its shape. Its **crate
  home is `mp_engine_qcommon`** (fixed by the engine-seam session resolution, not
  `Engine`'s crate — the two were decoupled 2026-07-02).
- `docs/architecture/state-ownership.md` — STATE-D1/D2 (islands, `GameWorld`),
  **STATE-D3** and **STATE-Q4** (the receiver/reachability of the error path,
  resolved 2026-07-02: `com_error` is **receiverless** and the per-level recovery
  runs **catch-side** in `com_frame`/`com_init`, not before `panic_any`;
  state-ownership.md's STATE-D3 is being amended to match — cross-ref, not
  restated), the `Common` field shapes this doc uses, the **`ComError` payload
  (home `mp_engine_qcommon`**, STATE-Q4), and **STATE-D5** (the
  aggregate `Engine` type — and the per-mode facade crate that defines it,
  `crates/{mp,sp}/engine/core` = `mp_engine_core`/`sp_engine_core`; the former
  STATE-Q1, resolved 2026-07-02).
- `docs/dossiers/A3-lifecycle.md` — the survey this doc renders (removed 2026-07-08; see git history).

## Scope & non-goals

**This doc decides:** the per-executable boot / frame / shutdown contracts for
the three host binaries — `jamp` (MP client), `jampded` (MP dedicated), `jasp`
(SP): loop ownership, `com_init` order, `com_frame` anatomy, `com_shutdown`
order, the error-recovery *flow* (per-level sequences and the panic boundary),
the event pump and its winit/console adapters, timing (`Sys_Milliseconds`,
FPS-cap spin, `Com_ModifyMsec` clamps), journaling, and binary packaging. It
**freezes** the per-mode `ErrorLevel` (LIFE-D3, = the existing `errorParm_t`),
the `com_*` seam signatures, the `sys_milliseconds` base-relative clock-read
signature (LIFE-D4b), and the `SysEventQueue` surface.

**Non-goals** (each punted to its owning doc):

- **Seam mechanics** — how a trap/`vmMain` crosses, dispatcher routing, transport
  (`NativeDll|Static|Wasm`) → `docs/architecture/engine-seam.md`.
- **Module load / restart mechanics** — `SV_InitGameProgs`, DLL/`GetGameAPI`
  loading, `vm_restart`, VM table → `docs/architecture/module-loading.md`.
- **Subsystem internals** — cvar parse tables, filesystem search paths, sound
  mixer, StringEd, netchan framing, collision → `docs/subsystems/*`.
- **State-ownership spine beyond the lifecycle-owned rows** →
  `state-ownership.md`. The **error-path reachability contract is STATE-Q4**
  (resolved 2026-07-02): `com_error` is receiverless and the per-level recovery
  runs **catch-side** (LIFE-D3, amended). This doc renders that catch-side
  per-level flow (§ Seam, `com_frame`); the "guards must be `Drop`-safe / no
  half-torn state observed" invariant is STATE-D3's (being amended to match) and
  is cross-referenced, never restated.
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

1. **`Com_Error` is C++ `throw`/`catch`, not `setjmp`/`longjmp`** — fixed string
   **literals** thrown from `Com_Error` — `"DISCONNECTED\n"` (MP `common.cpp:312`),
   `"DROPPED\n"` (`:326`), `"NEED CD\n"` (`:336`) — caught in `Com_Frame` (MP
   `common.cpp:1762`, SP `common.cpp:1450`) and `Com_Init` (MP `:1439`, SP
   `:1119`). In **Raven** the per-level recovery (SV_Shutdown / CL_Disconnect /
   ERR_DROP banner) runs *before* the throw and the `Com_Frame` catch merely
   `Com_Printf`s the thrown level literal (`reason`, MP `:1763`) and returns. Maps
   onto panic + `catch_unwind` (DEC-08) — **but the Rust port relocates the
   pre-throw recovery to the catch** because STATE-Q4 makes `com_error`
   receiverless (it cannot touch `Engine`): `com_error` only formats + `panic_any`,
   and `com_frame`/`com_init`'s catch runs the full per-level sequence in
   oracle-matching print order, printing the same level literal last (LIFE-D3,
   amended 2026-07-02). Same observable output, recovery moved throw-site → catch.
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

**Rust construction ordering (all three mains).** Raven's `WinMain`/`main` reads
`Sys_Milliseconds()` for a warm-up *before* `Com_Init`, relying on the C static
`sys_timeBase` being captured lazily on that first call (`win_shared.cpp:22-34`).
The Rust mains **construct `Engine` first** — `Engine::new()` captures the
`std::time::Instant` base into `Engine.common` (LIFE-D4b) — **then** perform the
warm-up `Sys_Milliseconds` read, **then** call `com_init(&mut engine, cmdline)`.
This is behaviorally equivalent to Raven **for the base-relative reads**
(`baseTime=false`): the warm-up read and every later frame-time read yield the same
series regardless of *when* the base is captured, and this is the ordering the
slice-0 `main()` skeleton renders (§ Slice hooks). The **raw variant**
(`baseTime=true`, Raven's `Rand_Init` seed) is *not* base-relative — it reads
`SystemTime::now()` (unix-epoch ms truncated to `i32`) instead of the `Instant`
base, its own absolute seed source (LIFE-Q3 resolved 2026-07-02; § Timing). This
diverges from Raven's absolute origin (OS-boot ms vs unix-epoch ms) but is below the
differential seam — RNG seeding from the clock was never cross-run deterministic.
Construction before `com_init` is forced anyway: `com_init` threads `&mut Engine`.

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
29. `Netchan_Init(Com_Milliseconds() & 0xffff)` — `:1383` (Rust: `& 0xffff` of
    `com_milliseconds(engine)`, the event-draining journaled reader — § Seam, § Timing;
    **not** `sys_milliseconds`).
30. **`VM_Init()`** — `:1384` (MP-only; registers `vm_game/vm_cgame/vm_ui`,
    zeroes `vmTable`). There is **no standalone `VM_Init` fn to freeze**: the
    `vmTable` becomes the empty **`ModuleRegistry`** owned by the engine
    module-host state (module-loading.md **LOAD-D5**), which is simply
    default-constructed empty (read module-loading.md — now Standing context — for
    its `{ slots: [Option<ModuleSlot>; MAX_VM] }` shape, LOAD-D8); the `vm_*` cvars become
    the module transport-select cvars (module-loading territory). "Registration
    only" for slice 0 = construct that empty registry + register those cvars — a
    `ModuleRegistry::default()`-shaped empty build. Its **crate home is
    `mp_engine_qcommon`** (fixed by the engine-seam session resolution; decoupled
    from `Engine`'s crate — the registry lives a tier below the `mp_engine_core`
    facade that owns `Engine`), so the empty-construct shape is a settled boot step.
    **Where the registry hangs off `Engine`: `Engine.common.modules`** (LIFE-Q5
    RESOLVED 2026-07-02, shared with state-ownership.md) — a field of `Common`, the
    qcommon-owned state struct, mirroring how Raven's `vmTable` is a qcommon-subsystem
    static (`vm.cpp:29`). `Engine`'s frozen five-field shape (`{ common, sv, cl, cm,
    snd }`, STATE-D5) is **unchanged** — the registry nests inside the existing
    `common` field, not a sixth `Engine` field, and the crate homes agree (`Common`
    and `ModuleRegistry` are both `mp_engine_qcommon`). It persists across frames as
    part of `Common` (§B3 — no global), reached as `engine.common.modules`. The
    single `vmTable` it replaces (`vm_t vmTable[MAX_VM]`, `vm.cpp:29`, zeroed by
    `VM_Init`, `vm.cpp:50-61`) covers all three module kinds together
    (`vm_game`/`vm_cgame`/`vm_ui`), matching one `Common.modules` registry for all
    three (it does not nest under `sv` or `cl`). state-ownership.md records the
    `Common.modules` field row (**STATE-D10**, the sibling's ID for this attachment,
    paired with LIFE-Q5); its `ModuleRegistry` type/shape stay module-loading.md's
    LOAD-D5. The `load_module` signature that later fills the registry is
    module-loading **LOAD-D8** (`ModuleRegistry::load_module -> SlotId`, frozen
    2026-07-02, resolving the former LOAD-Q3) and is not exercised until
    `SV_SpawnServer` loads the module (post-slice-0).
31. `SV_Init()` — `:1385` (serverinfo/systeminfo cvars, operator commands).
32. `com_dedicated->modified = qfalse` — `:1393`.
33. **`if (!com_dedicated->integer) { CL_Init(); Sys_ShowConsole(…); }`** —
    `:1394`. `CL_Init` (`cl_main.cpp:2549`): `Con_Init`, `CL_ClearState`,
    `CL_InitInput`, cl_* cvars, **`CL_InitRef()`** (`:2693` — wires `re` export
    table only; DEC-01 null-`refexport_t` stub), `SCR_Init`, `cl_running=1`.
    **[ded]** skipped entirely.
34. `com_frameTime = Com_Milliseconds()` — `:1402` (Rust: `com_milliseconds(engine)`,
    the journaled reader — § Seam; **not** `sys_milliseconds`).
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
24. `Netchan_Init(Com_Milliseconds() & 0xffff)` — `:1068` (Rust:
    `com_milliseconds(engine)`, the journaled reader — § Seam; **not** `sys_milliseconds`).
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
30. `com_frameTime = Com_Milliseconds()` — `:1086` (Rust: `com_milliseconds(engine)`,
    the journaled reader — § Seam; **not** `sys_milliseconds`).
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
`ERR_DROP` recovery point. SP `common.cpp:1269-1463` (`:1270`, `:1450`).

1. `Com_WriteConfiguration()` — MP `:1624` / SP `:1278` (writes config if archive
   cvars changed).
2. `com_viewlog->modified` → `Sys_ShowConsole` toggle — MP `:1627` / SP `:1281`.
   The `modified` check and its `qfalse` reset run **unconditionally** (MP
   `:1627,:1632` / SP `:1281,:1283`); only the inner `Sys_ShowConsole` is
   `!com_dedicated`-gated (MP `:1628`; SP has no `dedicated` gate, `:1282`).
3. com_speeds: `timeBeforeFirstEvents` — MP `:1637` / SP `:1291`.
4. `minMsec = 1000/com_maxfps` — MP `:1642` (`1` if dedicated, `com_maxfps<=0`, *or* timedemo) / SP
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
on the fatal path. In **Raven** this whole sequence runs *before* the throw;
**in the Rust port it runs catch-side** in `com_frame`/`com_init` (STATE-Q4:
`com_error` is receiverless, LIFE-D3 amended) — the tables below are Raven's
pre-throw order, which the catch reproduces verbatim (guard/bookkeeping,
per-level shutdowns, banner, then the level literal). A `com_error` raised
*during* catch-side recovery is Raven's recursive-error path: `com_error_recover`
runs inside its **own** `catch_unwind`, and a second `ComError` caught there while
the `errorEntered` guard is still set routes to `sys_error("recursive error after:
{saved message}")` — reproducing Raven's recursive-error banner + controlled exit
(MP `common.cpp:288` / SP `:265`), where the saved message is the *first* error's
text (it was never overwritten — `com_error` is receiverless; LIFE-D3 amendment
2026-07-03, § Seam). `com_buildScript`
forces `ERR_FATAL` (MP `:270` / SP `:261`). **MP-only:** `FS_PureServerSetLoadedPaks("","")`
at entry (`:275`); rapid-error escalation — >3 errors within 100ms → force
`ERR_FATAL` (statics `:251-252`, logic `:277-286`). **SP-only:** unconditional
`SG_Shutdown()` before dispatch (`:283`). **Both modes:** unless the level is
`ERR_DISCONNECT`, the formatted message is published as a `CVAR_ROM`
`com_errorMessage` cvar (`Cvar_Get` default + `Cvar_Set`, MP `:296-300` / SP
`:277-281`) — part of the catch-side `ErrorState` bookkeeping `com_error_recover`
runs (`com_error` is receiverless, so this cvar write, like every other side
effect, moves catch-side).

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
  subtraction, returning the raw **absolute** `timeGetTime()` value (its sole use is
  `Rand_Init(Sys_Milliseconds(true))`, `common.cpp:1248`, seeding the RNG from that
  large run-varying value). The Instant-backed clock (LIFE-D4b) exposes no absolute
  epoch, so the raw variant instead reads **`SystemTime::now()` — unix-epoch ms
  truncated to `i32`** (LIFE-Q3 RESOLVED 2026-07-02): same *role* (an absolute,
  run-varying seed for `Rand_Init`), different absolute *origin* (unix-epoch ms vs
  OS-boot ms). One-line divergence: below the differential seam — RNG-from-clock
  seeding was never cross-run deterministic, and Verification never golden-diffs it.
  SP `code/win32/win_shared.cpp:17-25`: `int Sys_Milliseconds(void)`, no raw variant.
- **`Com_Milliseconds` — a DISTINCT function, not a clock read** — MP
  `common.cpp:1028` / SP `code/qcommon/common.cpp:870`. It does **not** read the
  clock: it drains real/journaled events via `Com_GetRealEvent`, pushing each back
  onto the `com_pushedEvents` ring with `Com_PushEvent` (`:850`) in a loop until it
  sees `SE_NONE`, then returns that event's `evTime` ("will be journaled properly",
  `qcommon.h:673` / `code/qcommon/qcommon.h:528`). Its result is therefore
  replay-deterministic through the journal tap (DEC-09.1) — substituting
  `Sys_Milliseconds` would break that. Called at `Com_Init` steps 29/34 (`Netchan_Init`
  seed / `com_frameTime` init, MP `:1383,:1402`; SP steps 24/30, `:1068,:1086`) and by
  `SV_Frame`/`CL_Frame`. Ported as the private helper `com_milliseconds` (§ Seam),
  **not** `sys_milliseconds`.
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
  1024-entry `com_pushedEvents` `Com_PushEvent` ring (`common.cpp:747-752`,
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
**lifecycle-owned** rows below extend `Common` (`mp_engine_qcommon`'s `common`
module — `Common` moved down from `core` with the com_printf resolution, LIFE-D2
amendment). `state-ownership.md`'s master table fixes only their
**owner** (`Common.field`); the internal field *types* are `common`-module
subsystem detail (state-ownership treats each owned struct's field list as a
non-goal), i.e. mechanical §C ports of the cited Raven globals — their file/struct
layout is pinned mechanically in § Seam (the `common/` submodule tree), not
independently "frozen" elsewhere, and stated inline here so a slice-0 skeleton can
build:

- `frame_{time,msec,number}: i32` — Raven `int` (`common.cpp:79-81`).
- `frame_last_time: i32` — the `Com_Frame` `static int lastTime` (MP `common.cpp:1601`
  / SP `:1274`), the previous-frame timestamp the FPS-cap spin subtracts (`msec =
  com_frameTime - lastTime`, § `Com_Frame` step 5) and rewrites at step 6; a §B3 hoist
  of a function static into `Common` (no statics).
- `fully_initialized: bool` — Raven `qboolean com_fullyInitialized` (MP `common.cpp:84`
  / SP `:69`), set true at `Com_Init` step 40 (MP `:1434` / SP `:1104`) and read by
  `Com_WriteConfiguration` (§ `Com_Frame` step 1) to suppress config writes on a
  partial-init quit (MP `:1479` / SP `:1162`).
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
| `Com_Frame` `static int lastTime` (FPS-spin delta base) | MP `common.cpp:1601` / SP `:1274` | `Common.frame_last_time: i32` (§B3 hoist of a fn static) | `Com_Init` (0) | `Com_Frame` 5 reads / 6 writes (MP `:1656`) |
| `com_fullyInitialized` | MP `common.cpp:84` / SP `:69` | `Common.fully_initialized: bool` (`qboolean`) | `Com_Init` (false) | set true `Com_Init` step 40 (MP `:1434`); read `Com_WriteConfiguration` (`Com_Frame` 1, MP `:1479`) |
| `com_errorEntered`/`com_errorMessage[4096]` | MP `common.cpp:83,86` | `Common.error` (`entered: bool`/`message: [u8;4096]`, preamble) | `Com_Init` | catch-side in `com_frame`/`com_init` (set/clear/store during recovery — STATE-Q4; `com_error` is receiverless, cannot touch it) |
| MP rapid-error `lastErrorTime`/`errorCount` (100ms/3 escalation) | MP `common.cpp:251-252` | `Common.error` **MP-only fields** (rule §B3 — no statics) | `Com_Init` | catch-side in `com_frame`/`com_init` (the escalation is part of the pre-throw work relocated catch-side per LIFE-D3; `com_error` is receiverless — Raven logic `common.cpp:277-286`) |
| `com_journalFile`/`com_journalDataFile` | MP `common.cpp:34-35` | `Common.journal` (**MP only**) | `Com_Init` step 13 `Com_InitJournaling` | `Com_GetRealEvent` tap |
| `eventQue[256]`/`eventHead`/`eventTail` (`Sys_QueEvent` ring) | MP `win_main.cpp:1162-1166` | `Common.sys_events: SysEventQueue` (**new; distinct from the 1024 `com_pushedEvents`**) | `Com_Init` (empty ring) | winit/console adapters `queue()`; `Sys_GetEvent` drains |
| `sys_timeBase` (Sys_Milliseconds base) | MP `win_shared.cpp:22-34` / SP `:17-25` | `Common` field holding the `std::time::Instant` base — **captured inside `Engine::new()`** (LIFE-D4b), which runs first in `main()`, so the base exists before the warm-up read and before `com_init`. Behaviorally equivalent to Raven's lazy static capture on the first `Sys_Milliseconds` call (`win_shared.cpp:22-34`) **for the base-relative reads** (`baseTime=false`), which yield the same series regardless of capture time; the raw `baseTime=true` variant is *not* base-relative — it reads `SystemTime::now()` (unix-epoch ms → `i32`) instead of this base (LIFE-Q3 resolved 2026-07-02, § Timing). Exact field placement/name is a `common`-module mechanical detail (LIFE-D4b). | `Engine::new()` (before warm-up read + `com_init`) | read-only after capture |

Everything else the A3 survey touched (`sv`/`svs`, `cl`/`clc`/`cls`, cvar/cmd/fs
tables, sound, module VM handles — the step-30 `ModuleRegistry` lives at
`Engine.common.modules`, LIFE-Q5, its `Common.modules` row owned by
state-ownership.md) is owned by **state-ownership.md** — see its master table; not
duplicated here. The `Com_Printf` print state (`rd_buffer` redirect, `logfile`,
console buffer — `common.cpp:128,137-171`) is likewise `Common`-owned (its field
rows are state-ownership.md's, per the com_printf resolution, LIFE-D2 amendment
2026-07-03).

## Seam definition

FROZEN — porters fill bodies without changing these shapes. Per LIFE-D2 (amended
2026-07-02/07-03) the `Engine`-threading lifecycle functions
(`com_init`/`com_frame`/`com_shutdown`) are ported **per-mode** into
`crates/{mp,sp}/engine/core` (`mp_engine_core` / `sp_engine_core`) — **not** the
qcommon crates, because `com_frame` must call `SV_Frame`/`CL_Frame`, which qcommon
cannot reach; there is **no shared `Lifecycle` trait**. Two `com_*` are the
**exception** and live one tier below in `mp_engine_qcommon` so leaf callers reach
them: `com_error`/`ComError` (STATE-Q4) and `com_printf` (com_printf resolution,
LIFE-D2 amendment) — both detailed below. `Engine` is the aggregate engine-island
struct (state-ownership STATE-D1); its defining crate is **`mp_engine_core`**
(STATE-D5, the former STATE-Q1 resolved 2026-07-02).

**What STATE-D5 settles vs. what stays app-crate mechanical.** `Engine`'s crate is
now fixed (`mp_engine_core`), so the `use Engine` import path is pinned; the
`&mut Engine`-threading `com_*` lifecycle surface (`com_init`/`com_frame`/
`com_shutdown` + their private helpers) lives in `mp_engine_core` with it — nothing
below `core` can name `Engine`. **Exceptions living one tier below in
`mp_engine_qcommon`:** the `Common` state struct itself (and its owned field types),
`com_printf` (com_printf resolution, LIFE-D2 amendment 2026-07-03), and
`com_error`/`ComError` (STATE-Q4) — each because leaf callers below `core` must
reach it. What is **not** oracle-derivable and stays an `mp/app`-crate mechanical
detail (Raven is C, no crates) is the exact `[features] dedicated = […]` +
dependency edges that compile-exclude the client tier from `jampded`
(§ Binary packaging) — an app-crate wiring choice, not a lifecycle or STATE-D5
blocker. **File tree — pinned mechanically (not architectural):** the
lifecycle-owned state types + `com_printf` port from `common.cpp` into a **`common/`
module of `mp_engine_qcommon`** (so every engine crate, all of which depend on
qcommon, reaches them), and the `com_*` lifecycle functions into a **`lifecycle.rs`
of `mp_engine_core`** (they call `SV_Frame`/`CL_Frame`) —

- `common.rs` (**`mp_engine_qcommon`**) — the `Common` struct, which owns the
  `Com_Printf` print state (`rd_buffer` redirect, `logfile`, console buffer,
  `common.cpp:128,137-171`), plus the colocated `com_printf` free function that
  mutates that print state (`common.cpp:128`);
- `sys_event_queue.rs` (**`mp_engine_qcommon`**) — `SysEventQueue` (below);
- `journal.rs` (**`mp_engine_qcommon`**) — `Journal` (MP only);
- `error.rs` (**`mp_engine_qcommon`**) — `struct ErrorState { entered, message,
  last_error_time, error_count }` (the `Common.error` field group, § State ownership;
  `last_error_time`/`error_count` are MP-only), plus `com_error`/`ComError`
  (STATE-Q4);
- `engine.rs` (**`mp_engine_core`**) — the `pub struct Engine` aggregate + its
  `impl Engine { fn new() -> Box<Engine> … }` constructor. STATE-D5 *defines* the
  type in `mp_engine_core` but names no file; this pins only its **file**, and
  mechanically: `mp_engine_core` is a brand-new crate (confirmed absent from
  `crates/mp/engine/`, which today holds `botlib/client/ghoul2/icarus/qcommon/rmg/
  server` and no `core`), so it has no crate-local submodule convention to mirror
  the way qcommon's `common/` tree does; CLAUDE.md's **global one-type-per-file**
  rule therefore governs — the `Engine` type gets its own `engine.rs` (snake_case
  of the type), sibling to `lifecycle.rs`. Not an architectural choice — the same
  mechanical rule this whole tree applies, rendered verbatim rather than left to a
  porter to infer.
- `lifecycle.rs` (**`mp_engine_core`**) — the `com_init`/`com_frame`/`com_shutdown`
  functions + private `com_error_recover` helper, ported from the one `.cpp`, **plus
  the colocated `&[mut ]Engine`-threading lifecycle free functions `sys_error`
  (LIFE-D3) and `sys_milliseconds` (LIFE-D4b)**. Both are functions, not types, so
  CLAUDE.md one-type-per-file does not split them out — they colocate here (one
  colocated fns file, the same mechanical convention), with the `com_*` surface they
  serve. `sys_milliseconds`'s placement here was already stated in § `sys_milliseconds`;
  `sys_error` is stated explicitly now (it is **not** one of the two qcommon-tier
  exceptions — `com_error`/`com_printf` live one tier below — so it stays in `core`,
  LIFE-D3, and lands in `lifecycle.rs` beside the `com_*` functions that call it).

This tree is a **mechanical** layout (CLAUDE.md one-type-per-file for the structs +
one colocated fns file for the free functions), **not** an architectural decision —
a dry-run renders it verbatim rather than inventing filenames. The `common/` module follows
`mp_engine_qcommon`'s **universal `mod.rs`-root convention** (observed: every
existing submodule — `cm/`, `files/`, `qcommon/`, `vm/`, … — has a declaration-only
`mod.rs` root plus one snake_case file per type, e.g. `qcommon/sys_event_t.rs`; no
namesake-in-`mod.rs`). So the four `mp_engine_qcommon` `common/`-module bullets above
render as: **`common/mod.rs`**
(module root — `pub mod` declarations + re-exports, no types), **`common/common.rs`**
(the `Common` struct + colocated `com_printf`), **`common/sys_event_queue.rs`**,
**`common/journal.rs`**, **`common/error.rs`**; the `common::common::Common` path is
smoothed by a `pub use common::Common;` re-export in `mod.rs`, the crate's standard
idiom. `engine.rs` and `lifecycle.rs` are flat modules of `mp_engine_core` (the
`Engine` struct in the former, the `com_*`/`sys_error`/`sys_milliseconds` free-function
surface in the latter). This is the crate's
existing mechanical convention, cited not invented (CLAUDE.md one-type-per-file).

`com_printf` is the `Com_Printf` port. Per the com_printf resolution (LIFE-D2
amendment 2026-07-03) it takes **`&mut Common`** and lives in **`mp_engine_qcommon`**
(`common.cpp:128`), **not** `core` — `Common` owns its print target (`rd_buffer`,
`logfile`, console), and Raven's most-called primitive has ~427 lower-tier call
sites (qcommon/server) that cannot name a `core`-located function; every engine
crate depends on qcommon and so reaches it there. `mp_engine_core` keeps at most a
convenience re-export. This mirrors the STATE-Q4 reachability fix for `com_error`.
`sys_error` is `Sys_Error` (declared `qcommon.h:966`, Raven body in the platform
entry shell — `win32/win_main.cpp:350`, dedicated `null/win_main.cpp:324`); the Rust
port ports it **into `mp_engine_core`** delegating the OS work to `native/platform`
(LIFE-D3, LIFE-Q2 closed — detailed below). Both are **slice-0
deliverables** (unported today), like `errorParm_t`/`sysEvent_t` are pre-existing
— see § Slice hooks. Their **signature shapes are pinned by the frozen bodies that
call them** (style rule 5): `com_printf` is called (via `&mut engine.common`) with the
`ERR_DROP` ERROR banner inside `com_error_recover`, and with the **bare** level literal
as `com_frame`'s catch-arm terminal (Raven `Com_Printf(reason)`, MP `:1763`);
`sys_error(engine, &str)` is called two ways — `com_error_recover`'s `ERR_FATAL`
escalation passes the **formatted** payload `&e.msg` (Raven `Sys_Error("%s",
com_errorMessage)`, MP `:344`), while `com_init`'s init-catch escalates a *recoverable*
level with the **wrapped** literal `&format!("Error during initialization: {literal}")`
(Raven `Sys_Error("Error during initialization: %s", reason)`, MP `:1439` / SP `:1119`
— SP omits the colon, § SP `Com_Init` step 33; `reason` = the thrown literal). Both are
`sys_error(&mut Engine, &str) -> !`, so the signature freezes either way. So:

```rust
/// Raven `Com_Printf` (`common.cpp:128`). Threads `&mut Common` and lives in
/// `mp_engine_qcommon` (com_printf resolution, LIFE-D2 amendment) — mutates the
/// redirect buffer (`rd_buffer`), console, and the lazily-opened `logfile`
/// (`common.cpp:137-171`), all `Common` state. Reachable from every engine crate
/// (all depend on qcommon); `core` callers pass `&mut engine.common`.
pub fn com_printf(common: &mut Common, msg: &str);

/// Raven `Sys_Error` (`win32/win_main.cpp:350`; dedicated `null/win_main.cpp:324`).
/// Noreturn (`-> !`) — Raven ends in `exit(1)` after console teardown + `IN_Shutdown`;
/// the fatal escalation point for `com_init`'s init-catch. Ported INTO
/// `mp_engine_core` (LIFE-D3, LIFE-Q2 closed), delegating print+exit to
/// `native/platform` — a downhill call, no `core`→`mp/app` edge.
pub fn sys_error(engine: &mut Engine, msg: &str) -> !;
```

These signatures freeze as written. `com_printf` lives in **`mp_engine_qcommon`**
(the com_printf resolution, LIFE-D2 amendment — reachable by its ~427 lower-tier
callers; `mp_engine_core` keeps at most a convenience re-export). **`sys_error`
relocates into `mp_engine_core`**
(LIFE-D3, resolving LIFE-Q2 2026-07-02): its Raven body is the platform entry
shell (`win32/win_main.cpp:350`, dedicated `null/win_main.cpp:324`), but the Rust
port ports it into `core` with its frozen signature unchanged and has it **delegate
to `native/platform` fatal primitives** (stderr/console print + process `exit`) —
`core` already depends downhill on `native/platform`, so this is a normal downhill
call, **not a `core`→`mp/app` reverse edge**. `mp/app` keeps nothing error-related
and no injection machinery hangs off `Engine`. The Win32 message-box / console-show
surface of Raven's `Sys_Error` is deferred to the client-shell slice (headless
dedicated boot needs only print + exit; DEC-01). This closes the former LIFE-Q2.

### The `com_*` entry surface (per mode)

```rust
// mp/engine/core (mp_engine_core; SP mirror: sp/engine/core). NOT a trait — per-mode fns.
// EXCEPT `com_error`/`ComError`/`ErrorLevel`, which live one tier below in
// `mp_engine_qcommon` (STATE-Q4) so leaf throw sites can reach them (§ Seam prose).
// `errorParm_t` is the EXISTING ported enum (mp_qshared::errorParm_t, 5 variants;
// sp_qshared::errorParm_t, 4 variants) — this is state-ownership's `ErrorLevel`.

// state-ownership.md names the `ComError` level-field type `ErrorLevel` (its
// `ComError { level: ErrorLevel, .. }`); lifecycle.md is where that name freezes
// (LIFE-D3) as the per-mode `errorParm_t`. The alias makes the two spellings below
// (`com_error(level: errorParm_t, ..)` and `ComError.level: ErrorLevel`) one type.
// `com_error` + `ComError` live in `mp_engine_qcommon` (STATE-Q4, 2026-07-02) so
// leaf throw sites (e.g. `mp_engine_server`, which does NOT depend on the `core`
// facade) can raise them; the alias lives beside them in qcommon:
pub type ErrorLevel = errorParm_t;   // per-mode: MP 5-variant / SP 4-variant errorParm_t

/// Raven `Com_Init` (MP `common.cpp:1216` / SP `:950`). Runs the boot contract;
/// a ComError panic during init is caught here and escalated to fatal
/// (mirrors the `catch → Sys_Error` at MP `:1439` / SP `:1119`, LIFE-D3).
pub fn com_init(engine: &mut Engine, command_line: &str);

/// Raven `Com_Frame` (MP `common.cpp:1593` / SP `:1269`). One frame; the
/// `catch_unwind` boundary (DEC-08 / SEAM-D10) wraps the body — a caught ComError
/// runs the full per-level recovery catch-side (the ERR_DROP recovery point, MP
/// `:1762`) then prints the level literal; any non-ComError panic (a genuine Rust
/// bug) is re-raised as fatal (LIFE-D3).
pub fn com_frame(engine: &mut Engine);

/// Raven `Com_Shutdown` + `Com_Quit_f` orchestration (MP `common.cpp:356,1785`).
pub fn com_shutdown(engine: &mut Engine);

/// Raven `Com_Error` (MP `common.cpp:249` / SP `:245`). RECEIVERLESS and pure
/// (STATE-Q4, LIFE-D3 amended): formats the message into the payload and
/// `panic_any(ComError { level, msg })` — NO recovery, NO `Engine`. The per-level
/// recovery Raven ran before its throw is relocated CATCH-SIDE into
/// `com_frame`/`com_init` (in `core`). Lives in `mp_engine_qcommon` so leaf throw
/// sites can raise it. `-> !`: never returns (always panics).
pub fn com_error(level: errorParm_t, msg: String) -> !;   // in mp_engine_qcommon
```

`com_frame`'s catch boundary (the only new-code control structure):

```rust
pub fn com_frame(engine: &mut Engine) {
    use std::panic::{catch_unwind, AssertUnwindSafe, resume_unwind};
    use mp_engine_qcommon::ComError;   // com_error/ComError home is qcommon (STATE-Q4)
    match catch_unwind(AssertUnwindSafe(|| com_frame_body(engine))) {
        Ok(()) => {}
        Err(p) => match p.downcast::<ComError>() {
            // ERR_DROP recovery point. com_error() only panicked; the catch runs
            // ALL of Raven's PRE-THROW work via com_error_recover (errorEntered guard
            // + ErrorState bookkeeping, per-level SV_Shutdown/CL_Disconnect/
            // CL_FlushMemory, the ERR_DROP ERROR banner) in oracle print order, and
            // this arm then supplies Raven's CATCH BODY — the BARE per-level LITERAL
            // via com_printf (Raven `Com_Printf(reason)`, MP :1763). com_error_recover
            // stops at the banner; the terminal literal is the catch arm's, mirroring
            // Raven's throw/catch split (LIFE-D2 catch-print). com_init's arm differs
            // ONLY here: it wraps the same literal as sys_error("Error during
            // initialization: {literal}") instead of printing it bare (Raven :1439 /
            // SP :1119), never a double print.
            Ok(e) => {
                let level = e.level;   // Copy; e is moved into com_error_recover below
                // com_error_recover runs inside its OWN catch_unwind (LIFE-D3
                // amendment 2026-07-03). A com_error raised DURING recovery, while
                // errorEntered is still set, is Raven's recursive-error path — route
                // it to sys_error("recursive error after: {saved}"), where `saved` is
                // the FIRST error's message: com_error is receiverless (STATE-Q4), so
                // the nested throw never overwrote engine.common.error.message,
                // matching Raven's guard-before-vsprintf read of com_errorMessage
                // (MP common.cpp:288 / SP :265). ERR_FATAL/escalated never returns
                // here — com_error_recover calls sys_error(&e.msg) itself (Raven :344).
                if let Err(p2) =
                    catch_unwind(AssertUnwindSafe(|| com_error_recover(engine, *e)))
                {
                    if p2.is::<ComError>() && engine.common.error.entered {
                        // `saved` = the FIRST error's text: reading the NUL-terminated
                        // ErrorState.message ([u8; MAXPRINTMSG], § State ownership) as a
                        // &str is the standard mechanical C-string read (as at every
                        // com_printf boundary), not a new decision.
                        let saved = c_str(&engine.common.error.message).to_owned();
                        sys_error(engine, &format!("recursive error after: {saved}"));
                    }
                    resume_unwind(p2); // non-recursive re-panic / Rust bug → fatal
                }
                // Recovery returned ⇒ a recoverable level. Raven's com_frame catch
                // body: Com_Printf(reason) — the BARE level literal — then the frame
                // returns and the loop continues (MP :1763). (com_init's arm wraps this
                // same literal in sys_error("Error during initialization: …") instead.)
                com_printf(&mut engine.common, error_level_literal(level));
            }
            Err(other) => resume_unwind(other),   // real Rust bug → fatal (LIFE-D3)
        }
    }
}
```

`com_init` wraps `com_init_body` in `catch_unwind` identically. Its caught-`ComError`
arm runs the init-time recovery through the **same** `com_error_recover` helper —
itself wrapped in its own inner `catch_unwind`, exactly as `com_frame` above. The two
arms share every step **except the catch-arm terminal**: where `com_frame`'s arm prints
the bare level literal and lets the frame return, `com_init`'s arm — because init-time
errors are always fatal — escalates the returning recoverable level to `sys_error` with
the **wrapped level literal**, `sys_error(engine, &format!("Error during initialization:
{literal}"))`, reproducing Raven's `catch → Sys_Error("Error during initialization: %s",
reason)` (MP `:1439` / SP `:1119`, `reason` = the thrown literal `"DROPPED\n"` etc.) —
**not** `sys_error(engine, &e.msg)`, which is the *formatted* message and is used only
for a direct/escalated `ERR_FATAL` (Raven `Sys_Error("%s", com_errorMessage)`, MP
`:344`, run inside `com_error_recover`). It does **not** print a bare literal first (no
double-print): the same `error_level_literal(level)` string `com_frame` prints bare is
folded, here, into that one `sys_error` message. Because `ERR_FATAL`/escalated is
handled inside `com_error_recover` (which calls `sys_error(&e.msg)` and never returns),
the "Error during initialization:" wrapper reaches **only** a recoverable level, never a
fatal one — matching Raven, where `ERR_FATAL` calls `Sys_Error` directly and bypasses
the init `catch` entirely. **The recursive treatment is identical:** a `com_error` raised *during* init recovery while
`errorEntered` is still set routes to `sys_error("recursive error after: {saved}")`
from that inner catch (LIFE-D3 amendment 2026-07-03), the same mechanism `com_frame`
uses. (Were a future refactor to make `com_init`'s catch escalate to fatal *without*
running `com_error_recover`, its recovery could not recurse and the inner catch would
be unnecessary — but as written it runs recovery, so it carries the wrapping.)
`com_error_recover` (in `core`, `&mut Engine`) is a private catch-side helper,
not part of the frozen surface — its body is Raven's **pre-throw** work: the per-level
sequence of § Error recovery run in Raven's pre-throw order (per-level
`SV_Shutdown`/`CL_Disconnect`/`CL_FlushMemory` + the `ERR_DROP` ERROR banner + guard
clear for a recoverable level; the fatal shutdown chain + `sys_error(engine, &e.msg)`,
Raven `:344`, for `ERR_FATAL`/escalated, which never returns). It stops at the ERROR
banner and does **not** print the terminal level literal — that print is the **catch
arm's**, mirroring Raven's throw/catch split: `com_error_recover` = Raven's pre-throw
body, the catch-arm terminal = Raven's `catch` body (`Com_Printf(reason)` in
`com_frame`, MP `:1763`; the wrapped `Sys_Error("Error during initialization: %s",
reason)` in `com_init`, MP `:1439` / SP `:1119`). `error_level_literal` is the
mechanical per-mode `errorParm_t` → thrown-string map — the exact literals the § Error
recovery tables list per level (MP `common.cpp:312,326,336` / SP mirror), not a frozen
decision. `ComError`'s payload shape is
frozen in `state-ownership.md` (§ Seam); this doc names `errorParm_t` as its `level`
type (LIFE-D3) and the **catch-print behavior** (LIFE-D2). Concretely, that block is
`pub struct ComError { pub level: ErrorLevel, pub msg: String }` in
**`mp_engine_qcommon`** (STATE-Q4) — a tier below the `core` facade, so leaf throw
sites in `mp_engine_server` (which does not depend on `core`) can raise it. `core`
already depends downhill on qcommon, so the `downcast::<ComError>()` above resolves it
with a plain `use mp_engine_qcommon::ComError`. **The fields are `pub`, and that is
load-bearing, not stylistic** (2026-07-03 sync, closing the STATE-Q7 self-inconsistency
this doc previously carried): `com_error_recover` runs catch-side in `mp_engine_core`
and reads `e.level`/`e.msg` after the `downcast::<ComError>()` above (§ `com_frame`
snippet) — a **cross-crate** field read (`ComError` lives in `mp_engine_qcommon`) that
compiles **only** with `pub` fields. state-ownership.md's `ComError` block owns and
freezes this payload shape with the same `pub level`/`pub msg` fields; this spelling now
matches it (the earlier non-`pub` spelling here was the STATE-Q7 self-inconsistency —
its own `com_error_recover` read `e.level`/`e.msg` cross-crate yet the fields were
private). Those two fields are **exhaustive** (the
`{level, msg}` the recovery reads is the whole payload), and it needs **no derive**:
a `panic_any`/`downcast` payload only requires `Any + Send + 'static`, which the enum
+ `String` satisfy automatically. `ErrorLevel` is per-mode `errorParm_t` (LIFE-D3,
below).

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
- `fn com_milliseconds(engine: &mut Engine) -> i32` — Raven `int Com_Milliseconds(void)`
  (MP `common.cpp:1028` / SP `code/qcommon/common.cpp:870`). **Distinct from
  `com_event_loop`**: it does *not* dispatch events — it drains real/journaled events
  via `com_get_real_event` and pushes each back onto the `com_pushedEvents` ring
  (`Com_PushEvent`, `:850`) until `SE_NONE`, returning that event's `evTime`. This is
  the journaled reader `Com_Init` steps 29/34 (SP 24/30) call — the one private helper
  reached from `com_init` rather than `com_frame_body`. Faithfully porting it (rather
  than substituting the frozen `sys_milliseconds`) is **forced**, not a free choice:
  §A2 (no speculative behavior) + DEC-09.1 (only the journal-tapped read is
  replay-deterministic; substituting the clock read would silently break replay). Not
  frozen — a mechanical §C port like the helpers above.

### `sys_milliseconds` — the base-relative clock read (LIFE-D4b)

FROZEN. Backs Raven's **`Sys_Milliseconds`** — the raw clock read: every `main()`
warm-up read (§ Slice hooks), event stamping, and `Com_ModifyMsec`. It is **not**
Raven's `Com_Milliseconds` — that distinct event-draining/journaled reader (§ Timing)
is ported as the `com_milliseconds` helper (§ Seam), so `Com_Init` steps 29/34 do
**not** call this function. It threads `Engine` and reads the `std::time::Instant`
base held in `Common` (§ State ownership, `sys_timeBase` row); the base-relative read
computes `now − base` as a `u64` elapsed-milliseconds value truncated with `as i32`,
and that truncation reproduces `timeGetTime`'s practical 49.7-day wraparound
(LIFE-D4b) — matching Raven's wrapping `int` with no special-casing. Pure
`std` (no `timeGetTime`, no platform shell), so unlike `Sys_Init`/`Sys_Quit` it
needs no `native/platform` delegation (LIFE-Q2/LIFE-D3). It threads `&Engine`, so
it lives in `mp_engine_core`'s `lifecycle.rs` with the `com_*` surface (STATE-D5) —
*not* the qcommon `common/` module the `Common` state types moved to (com_printf
resolution); the base it reads is `engine.common`'s field. Receiver is `&Engine`
(shared): the base is captured once in `Engine::new()` and never mutated afterward
(§ State ownership). MP keeps Raven's `baseTime` bool, SP is `void` (LIFE-D4b,
DEC-04). Because Rust has no default arguments, MP call sites pass the value
explicitly — Raven's base-relative reads (the warm-up and every frame-time read) are
`baseTime=false` (Raven's C++ default, `qcommon.h:978`); the `baseTime=true`
raw-absolute path reads `SystemTime::now()` (unix-epoch ms → `i32`), its own
absolute seed source (LIFE-Q3 resolved, § Timing).

```rust
// mp_engine_core `lifecycle.rs` (SP mirror: sp_engine_core). Backs Sys_Milliseconds
// with the Common-owned Instant base (LIFE-D4b); base-relative i32 API. The
// baseTime=true raw variant returns SystemTime::now() (unix-epoch ms → i32; LIFE-Q3).
// MP — Raven `int Sys_Milliseconds(bool baseTime = false)` (win_shared.cpp:22-34,
// decl qcommon.h:978):
pub fn sys_milliseconds(engine: &Engine, base_time: bool) -> i32;

// SP mirror — Raven `int Sys_Milliseconds(void)` (win_shared.cpp:17-25,
// decl qcommon.h:770); no raw variant (DEC-04):
//   pub fn sys_milliseconds(engine: &Engine) -> i32;
```

### `SysEventQueue` — the 256-entry `Sys_QueEvent` ring

```rust
// mp_engine_qcommon `common` module (SP mirror: sp_engine_qcommon). Faithful queue
// semantics of eventQue[256] (win_main.cpp:1162-1203). NOT the 1024-entry
// com_pushedEvents ring.
pub const MAX_QUED_EVENTS: usize = 256;

pub struct SysEventQueue {
    que:  [sysEvent_t; MAX_QUED_EVENTS],   // sysEvent_t: mp_engine_qcommon (already ported)
    head: usize,                            // monotonic; & (MAX_QUED_EVENTS-1) to index
    tail: usize,
}

impl SysEventQueue {
    /// `Sys_QueEvent` (win_main.cpp:1178-1203). `time==0` → stamp the
    /// caller-threaded `now_ms` (Raven's internal `Sys_Milliseconds()`); a nonzero
    /// `time` is used as-is. On `ptr = Some(box)`, `Box::into_raw` stores the raw
    /// pointer in `sysEvent_t.evPtr` (LIFE-Q4 resolved — the Box round-trip below).
    /// Overflow drops the oldest, reconstituting its `Box` from `evPtr` and dropping
    /// it (the free). Called by the platform adapters.
    ///
    /// `now_ms` is threaded in — mirroring `get` — because the `Sys_Milliseconds`
    /// Instant base lives in `Common`, not this queue (LIFE-D4b, § State ownership):
    /// the queue is a pure ring that reaches no clock (§B3/§B4), so faithful
    /// `time==0` stamping (Raven's `Sys_QueEvent` computes it internally) is fed the
    /// value rather than reaching for it. Symmetric with `get`'s `now_ms`.
    pub fn queue(&mut self, time: i32, now_ms: i32, ty: sysEventType_t, value: i32,
                 value2: i32, ptr: Option<Box<[u8]>>);
    /// `Sys_GetEvent` reduced to a PURE ring-drain — NO OS pump inside (DEC-02
    /// inversion). Returns a synthesized `SE_NONE` stamped with the caller-threaded
    /// `now_ms` when empty (`win_main.cpp:1270-1273`). NB Raven stamps this empty
    /// event with the raw **absolute** `timeGetTime()` (`:1273`) — NOT base-relative
    /// `Sys_Milliseconds()`, unlike `queue`/`Sys_QueEvent` — and `Com_EventLoop`
    /// returns it as `com_frameTime` (`common.cpp:946`). Threading the base-relative
    /// `now_ms` (LIFE-D4b) makes `com_frameTime` base-relative: identical for every
    /// delta use (msec/FPS-cap) and differing only in the absolute origin of the
    /// run-varying serverid seed (`Com_Init` step 34), below the differential seam
    /// (same rationale as LIFE-Q3/LIFE-D4b).
    pub fn get(&mut self, now_ms: i32) -> sysEvent_t;
}
```

**Payload ownership / `evPtr` marshaling — RESOLVED 2026-07-02 (LIFE-Q4).** The
internal representation **is the `Box` round-trip**: `sys_event_queue.rs` owns the
codebase's **one** `Box::into_raw`/`Box::from_raw` pair. `queue`'s
`ptr: Option<Box<[u8]>>` is consumed with `Box::into_raw`, and the raw pointer is
stored directly in the faithful `sysEvent_t.evPtr` (`*mut c_void`); `get` re-wraps
`evPtr` into an owned `Box` for the consumer; overflow eviction reconstitutes the
evicted entry's `Box` and drops it (Raven's "free the oldest evPtr"). This is
**§D11-confined**, not a §D11 violation: `sysEvent_t` is the Raven ABI `#[repr(C)]`
shape (layout-asserted, § Verification), so a ring of `sysEvent_t` **is** a seam
surface — the queue is exactly where the raw pointer lives, and the `unsafe` is the
seam confinement §D11 licenses, carried with a ≤2-line site note. **Invariant (stated
once, at the type):** every non-null `evPtr` in the ring came from `Box::into_raw` on
an `Option<Box<[u8]>>` payload, so every `from_raw` re-wrap (on `get`/eviction) is
sound. The public `queue`/`get` signatures stay frozen (LIFE-D4 / above); only this
internal representation was open, and it is now fixed. Closes LIFE-Q4.

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

**Divergence note — one-frame input latency during the spin (inherent to DEC-02).**
Raven's spin *pumps the OS* each iteration: `Com_EventLoop → … → Sys_GetEvent`
runs `PeekMessage` inside the getter (`common.cpp:1647-1653` → `win_main.cpp:1211`),
so OS input arriving mid-spin is picked up that same frame. Under DEC-02 winit owns
the loop with `ControlFlow::Poll`: the spin can only **re-drain the ring**, and the
ring is only refilled by winit callbacks that fire *between* `about_to_wait`
invocations — so OS input physically arriving during the spin window (≤ `1000 /
com_maxfps` ms, ~11 ms at the default 85) is deferred to the next frame rather than
consumed in the current one. This is a documented divergence inherent to DEC-02,
**below the differential seam**: DEC-09 verification is unaffected, because
journaling replay (LIFE-D4a) feeds the ring directly from the recorded stream, not
from live OS timing (§ Verification 1).

### `jampded` console adapter (LIFE-D4d)

No winit. The OS loop is `loop { com_frame(&mut engine) }` with the `Sleep(5)`
entry pacing (`null/win_main.cpp:1478`). A faithful minimal polled line editor
over raw stdin (the `Sys_ConsoleInput` behavior, `null/win_main.cpp:200-302`)
feeds completed lines as `SE_CONSOLE` events through `SysEventQueue::queue`
(faithful `Sys_QueEvent(0, SE_CONSOLE, …)` — `time==0`, `null/win_main.cpp:1195`;
the adapter threads the current clock as `now_ms` and the queue stamps it); net
poll feeds `SE_PACKET`. Same ring, same `Com_EventLoop` `SE_CONSOLE` handling.

The completed line rides `evPtr`/`evPtrLength` as the event payload (Raven
`Sys_QueEvent(0, SE_CONSOLE, 0, 0, len, buf)`, `null/win_main.cpp:1195`), read back
by `Com_EventLoop`'s `SE_CONSOLE` case (strip `\`/`/` + `Cbuf_AddText`,
`common.cpp:969`). The adapter passes the line bytes as `Some(Box<[u8]>)`; `queue`
stores it via `Box::into_raw` and the `SE_CONSOLE` drain re-wraps it through the
`Box` round-trip resolved at **LIFE-Q4** (§ `SysEventQueue`) — so the end-to-end
drain now works, not just the `queue` call.

Because `SysEventQueue::get` is a pure drain with no OS pump (DEC-02 inversion),
this poll runs in the **`jampded` main loop, outside `com_frame`** — the § Slice
hooks skeleton renders it verbatim: `loop { sleep_ms(5); console_poll(…);
net_poll(…); com_frame(…); }`, feeding the ring before `com_frame` drains it. The
poll adapter's entry point (`console_poll`/`net_poll`) is **app/platform-bin
new-code glue** whose exact signature is app-crate mechanical — **not** a frozen
lifecycle seam — exactly like `command_line()` (§ Slice hooks) and the winit
adapter (LIFE-Q1); only its behavior (a faithful `Sys_ConsoleInput` line editor,
`null/win_main.cpp:200-302`, emitting `SE_CONSOLE`) freezes here. Its inverted
shape (feeding `SysEventQueue::queue` instead of returning Raven's `char*`) is a
mechanical translation of the same producer inversion already fixed for
`queue`/`get`, so no signature needs freezing to make this loop step real.

### Binary packaging (LIFE-D2)

- **`crates/mp/app`** — two `[[bin]]` targets: `jamp` (default: client tier;
  conventional path `src/bin/jamp.rs`) and `jampded` (`src/bin/jampded.rs`). The
  `cl`/`snd` **field types are not feature-dependent**: state-ownership.md freezes
  `Engine { sv: Server, cl: Option<Client>, snd: Option<SoundSystem>, … }`
  (STATE-D5, amended item 20 2026-07-03) — **`sv` is a bare `Server`, NOT an `Option`**:
  server liveness is `sv.state == SS_DEAD` (`server.h:47-54`), the dual of Raven's
  zero-filled statics, so a dedicated build's server is always present (state-gated), not
  presence-gated. The `jampded` build's zeroed `sv` starts `SS_DEAD` and `SV_Init` brings
  it live; `cl`/`snd` are **`None`** (genuine client-presence Option, owner of the
  client-side pass: the client slice). Because *every* client/renderer/sound call in
  `com_*` is gated on `com_dedicated` (Raven-faithful — `Com_Init` step 33,
  `Com_Frame` step 10, `common.cpp:1394,1692`), a `None` client is **never
  dereferenced at runtime**, so **no null-stub client objects are needed** — the
  `Option` + the runtime gate together reproduce `null_client.cpp`'s "the calls are
  no-ops" behavior. The `dedicated` cargo **feature** therefore serves the
  *build-level* purpose of Raven's `null_*` link swap — a **client-less binary**:
  it compile-excludes the client-tier crates (`mp/engine/client` + renderer +
  sound) from `jampded`, not substitutes stub field types. `Engine`'s defining
  crate is now settled — `mp_engine_core` (STATE-D5) — so `mp/app`'s `use Engine`
  path is fixed; **the exact feature wiring** (how the client tier is `cfg`-excluded
  while the `Engine` type still names `Option<Client>`, i.e. the concrete
  `[features] dedicated = […]` + dependency edges) is an `mp/app`-crate mechanical
  detail, not oracle-derivable (Raven is C, no crates), and is not spelled out here.
  The **runtime** `com_dedicated` hot-toggle (`common.cpp:1675-1687`) stays compiled
  **only in the `jamp` binary** (dead on Raven's ROM dedicated build).
- **`crates/sp/app`** — one `[[bin]]`: `jasp` (`src/bin/jasp.rs`). No dedicated
  variant (SP has none).

## Decisions

**LIFE-D1 — Loop ownership, per target.** `jamp`/`jasp`: **winit (DEC-02) owns
the loop** — an `ApplicationHandler` with `ControlFlow::Poll`, `com_frame()`
called from `about_to_wait`, winit events translated into the ported 256-entry
`sysEvent_t` ring (faithful queue semantics; `Sys_GetEvent` becomes a pure
ring-drain). The FPS-cap busy-spin is preserved as a spin that **re-drains the
ring** (the accurate wording is § winit-adapter boundary). `jampded`: plain
`loop { com_frame() }` with the `Sleep(5)` entry pacing and polled console input.
Renderer deferral (DEC-01) means the windowed path is *designed now, first
exercised later* — early client slices boot headless. *Because* Raven's
architecture fixes the adapter direction (callbacks enqueue; frame work runs from
`about_to_wait`). *Rejected:* `ControlFlow::WaitUntil` for the FPS cap (diverges in
event-arrival timing); `pump_app_events` loop-shape preservation
(platform-limited/second-class, dossier §7).

*Amended 2026-07-02:* softened the timing claim — the spin does **not** pump the OS
(winit owns the loop under DEC-02); it only re-drains the ring. Dropped the
"pumps"/"parity-faithful timing" wording. Consequence: OS input arriving during the
spin window is deferred one frame — a documented DEC-02 divergence, below the
differential seam (cites and rationale in § winit-adapter boundary, "Divergence
note").

**LIFE-D2 — Skeleton + packaging.** `com_init`/`com_frame`/`com_shutdown`/`com_error`
(and `pub struct Engine`) are ported **per-mode** into the per-mode facade crates
`crates/{mp,sp}/engine/core` (`mp_engine_core` / `sp_engine_core`) — **no shared
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

*Amended 2026-07-02 (per STATE-Q1 resolution = STATE-D5):* the `com_init`/`com_frame`/
`com_shutdown` functions and `pub struct Engine` live in the per-mode facade crate
`crates/{mp,sp}/engine/core`, **not** the `qcommon` crates — `com_frame` must call
`SV_Frame`/`CL_Frame`, which `qcommon` (a tier below server/client) cannot reach,
and `Engine` aggregates `sv`/`cl`/`snd`. The `&mut Engine`-threading lifecycle items
(that `com_*` surface, its private helpers, `sys_error`) live in `core` (§ Seam).
(*Superseded in part below:* `com_printf` and the `Common` state module move down to
`qcommon` — the com_printf amendment.)

*Amended 2026-07-02 (per STATE-Q4):* **`com_error` and `ComError` are the exception —
they live one tier below, in `mp_engine_qcommon`, NOT `core`.** STATE-Q4 made
`com_error` **receiverless** (`com_error(level, msg) -> !`, pure format +
`panic_any`), precisely so leaf throw sites in `mp_engine_server` (which does not
depend on the `core` facade) can raise it; putting `ComError` in `core` would make
it unnameable from those leaf sites. `core` depends downhill on `qcommon`, so
`com_frame`'s catch names the payload with a plain `use mp_engine_qcommon::ComError`
(the earlier "moves to `core` … no cross-crate `use`" wording is retracted). The
rest of this record — thin app bins, two `[[bin]]` targets, the `dedicated` feature
— stands unchanged.

*Amended 2026-07-03 (the com_printf resolution, fixing the C4 layering
contradiction):* **`com_printf` and the `Common` state struct also live one tier
below, in `mp_engine_qcommon`, NOT `core`** — the same reachability fix STATE-Q4
applied to `com_error`, applied here far more pervasively. `com_printf` becomes
`com_printf(common: &mut Common, msg: &str)` (receiver swept from `&mut Engine` to
`&mut Common`): `Common` owns the print state (`rd_buffer` redirect, `logfile`,
console buffer — `common.cpp:128,137-171`), and Raven's most-called primitive has
~427 lower-tier call sites (qcommon + server) that cannot name a `core`-located
function. Because every engine crate depends on qcommon, all of them reach
`com_printf` there; `core` callers pass `&mut engine.common`, and `mp_engine_core`
keeps at most a convenience re-export. `Common` and its owned field types
(`SysEventQueue`, `Journal`, `ErrorState`) therefore live in a `common/` module of
`mp_engine_qcommon` (§ Seam file tree); only `com_init`/`com_frame`/`com_shutdown`
(which name `Engine`/call `SV_Frame`/`CL_Frame`) and `sys_error` stay in `core`.
state-ownership.md records the `Common` field rows. *Because* the print target must
be reachable from the leaf callers that print — identical to the `com_error` logic.
*Rejected:* keeping `com_printf` in `core` (the 427 lower-tier callers cannot reach
it — a design that cannot compile).

**LIFE-D3 — Error payload taxonomy.** `com_error(engine, level, msg)` runs the
per-level recovery **synchronously**, then `panic_any(ComError{level, msg})`;
`com_frame`'s `catch_unwind` continues (the `ERR_DROP` recovery point); a `catch`
also guards `com_init`, escalating to fatal (mirrors MP `:1439` / SP `:1119`).
`ErrorLevel` is the **existing per-mode `errorParm_t`** (MP 5 variants incl.
`ERR_SERVERDISCONNECT`/`ERR_NEED_CD`; SP 4 — DEC-04). `com_errorEntered` and MP's
3-errors-in-100ms escalation are owned `Engine.common` fields (§B3 — no statics).
Panics **without** our `ComError` payload (genuine Rust bugs) are re-raised as
fatal via `resume_unwind` — **never** swallowed into the `ERR_DROP` path.
Requires `panic="unwind"`; seam exports are `extern "C-unwind"` (SEAM-D10).
*Because* Raven's `throw`/`catch` maps onto panic/`catch_unwind` (dossier §4.1).
*Rejected:* one shared enum (loses the MP/SP structural difference).

*Amended 2026-07-02 (per STATE-Q4 + LIFE-D2 catch-print + LIFE-Q2 resolutions):*
three changes.
(1) **Receiverless, catch-side recovery.** `com_error` drops its `&mut Engine`
receiver — its signature is now `com_error(level: errorParm_t, msg: String) -> !`,
and it is **pure**: format the message into the payload, `panic_any(ComError{level,
msg})`, nothing else. Raven's pre-throw recovery (the `errorEntered` guard +
`ErrorState` bookkeeping, MP rapid-error escalation / `FS_PureServerSetLoadedPaks`,
SP `SG_Shutdown`, the per-level `SV_Shutdown`/`CL_Disconnect`/`CL_FlushMemory`, the
`ERR_DROP` banner) is relocated **catch-side** into `com_frame`/`com_init` (in
`core`), run in oracle-matching print order by the private `com_error_recover`
helper (§ Seam). A `com_error` raised *during* that catch-side recovery is Raven's
recursive-error → `Sys_Error` fatal path; the catch topology that renders it as
`sys_error("recursive error after: …")` is settled by the 2026-07-03 amendment
below (`com_error_recover` wrapped in its own `catch_unwind`). `com_error`
and `ComError` therefore live in `mp_engine_qcommon` (LIFE-D2 amendment), reachable
from leaf throw sites.
(2) **Catch prints the per-level literal, not the formatted message.** Faithful to
Raven, the catch's console line is the thrown level **literal** derived from
`ComError.level` — `"DISCONNECTED\n"` (`common.cpp:312`), `"DROPPED\n"` (`:326`),
`"NEED CD\n"` (`:336`) — via `Com_Printf(reason)` at `:1763`. The *formatted*
`com_errorMessage` is printed only where Raven printed it: the `ERR_DROP` ERROR
banner, now emitted catch-side as part of `com_error_recover` (before the literal),
matching `common.cpp:314`. (Corrects the earlier snippet, which printed the
formatted `e.msg` in place of the literal.)
(3) **`sys_error` home (closes LIFE-Q2).** `sys_error` keeps its frozen
`(engine: &mut Engine, msg) -> !` signature but is **ported into `mp_engine_core`**,
delegating the actual print + process-`exit` to `native/platform` fatal primitives
— a downhill call, **no `core`→`mp/app` reverse edge**, no `Engine`-injection
machinery; `mp/app` keeps nothing error-related. The Win32 message-box/console-show
surface of Raven's `Sys_Error` is deferred to the client-shell slice.
*Rejected (for (3)):* injecting `sys_error`'s body through `Engine` (fn-pointer /
trait object) — the downhill `native/platform` delegation needs no injection.

*Amended 2026-07-03 (recursive-error catch topology, resolving the former LIFE-Q6).*
The 2026-07-02 amendment left one thing imprecise: it claimed a `com_error` raised
*during* catch-side recovery simply "re-panics past the same catch = Raven's recursive
`Sys_Error` fatal path." That was wrong in observable output — the frozen `com_frame`
skeleton wrapped only `com_frame_body` in `catch_unwind`, so a second `ComError` inside
`com_error_recover` (which ran in the `Err` arm, *outside* that catch) would escape
`com_frame` to the outer `loop { com_frame }`, which has no catch — a bare unhandled
panic, **not** Raven's recursive-error banner + graceful exit. **Resolution:**
`com_frame`'s (and `com_init`'s) catch arm wraps the `com_error_recover` call in its
**own** `catch_unwind(AssertUnwindSafe(…))`. If recovery itself panics with a `ComError`
while the `errorEntered` guard is still set, that is the recursive-error case → route to
`sys_error("recursive error after: {saved}")`, where `saved` is the *first* error's
message (`com_error` is receiverless, STATE-Q4, so the nested throw never overwrote
`engine.common.error.message` — matching Raven's guard-check-before-`vsprintf` read of
`com_errorMessage`). This reproduces Raven's exact recursive-error banner and controlled
`exit` path (MP `common.cpp:288-289` `Sys_Error("recursive error after: %s",
com_errorMessage)` / SP `:265-266`). Non-recursive re-panics and genuine Rust bugs inside
recovery `resume_unwind` to fatal. The frozen `com_frame` snippet (§ Seam) is updated to
show this inner catch; `com_init` carries the identical wrapping (its recovery runs the
same helper). `sys_error`'s `(engine, msg) -> !` seam is unchanged — only the caller
structure gained the inner catch. *Because* the receiverless/catch-side model (STATE-Q4)
moved Raven's entry-guard check catch-side, and only an inner catch around
`com_error_recover` can convert a recursive throw into the controlled `sys_error` exit
rather than an escaping panic. *Rejected:* an outer catch in each `main()` around
`loop { com_frame }` (spreads error-path structure into the thin app bins, which LIFE-D3
(3) deliberately keeps error-free) — the inner catch keeps it in `core`.

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

*Amended 2026-07-02 (LIFE-Q3 resolution, shared with LIFE-D4b):* the `baseTime=true`
raw variant reads **`SystemTime::now()` — unix-epoch ms truncated to `i32`** (its
sole use is the `Rand_Init` seed at `Com_Init` step 9). The base-relative variant
stays `Instant`-backed (above). One-line divergence: same *role* (an absolute,
run-varying RNG seed) as Raven's raw `timeGetTime()`, different absolute *origin*
(unix-epoch ms vs OS-boot ms); below the differential seam — RNG-from-clock seeding
was never cross-run deterministic, and Verification golden-diffs journaled streams
and `Com_ModifyMsec`, never the clock seed. Closes LIFE-Q3; step 9 fully implements.

**LIFE-D5 — Engine-island large-value construction reuses `zeroed_box` (2026-07-03;
resolves the LIFE-Q7 mechanism fork; state-ownership STATE-D13 dual).** `Engine::new`
builds its large embedded-by-value members — `sv: Some(Server)` (`server_t` embeds
`svEntity_t svEntities[1024]` by value, ≈ 650 KB, `server.h:53-88`) and
`cm: CollisionWorld` (cmg + SubBSP[32]) — via the **STATE-D9 heap-zeroed
`zeroed_box<T: ZeroValid>` path**, by direct analogy to `GameWorld::zeroed`; `server_t`
and `clipMap_t` each gain a one-line `unsafe impl ZeroValid` beside their existing
layout static-asserts. This sanctions the new **`mp_engine_core -> native_platform`
Cargo edge** the reuse needs (SP dual `sp_engine_core -> native_platform`; `native/*`
is cross-mode tier-legal, so only a new direct edge). *Because* the engine island hits
the identical large-by-value-on-the-heap need the module island already solved — one
idiom serves both. *Rejected:* stack-then-move `Default` for `Server`/`CollisionWorld`
(the same constrained-stack overflow class STATE-D9 was adopted to prevent); a separate
engine-only zeroing helper (duplicates `zeroed_box`). The construction *mechanism* is
what freezes here; the residual field-by-field wrapper assembly (non-headline
large-by-value fields, stack-safe field order) is **not** settled — it stays LIFE-Q7
(§ Open questions), which STATE-D13 punts here and this round's inputs do not close.

*Amendment (2026-07-03, user ruling item 20 — superseded by whole-Engine zeroing).*
LIFE-D5's *two-headline-members* framing (give `sv: Some(Server)` and `cm` each a
`zeroed_box` call, then assemble the wrapper) is **superseded**. `Engine::new() ->
Box<Engine>` now allocates the **whole aggregate** as one boxed ZEROED heap buffer,
initializing its non-zero-valid fields **in place** before exposure — the `MaybeUninit`
pattern, **not** an `unsafe impl ZeroValid for Engine` (unsound; the aggregate is not
all-zeroes-valid — checkpoint-5 finding 21). *Amendment (2026-07-03, LIFE-Q9 closed —
mechanical generalization):* the in-place-init list is **every non-`ZeroValid` field**,
not just `Common.time_base` — the zeroed bytes legally cover only the `ZeroValid`-audited
`#[repr(C)]` mass, and Rust does not guarantee all-zeros = `None` for the niche-bearing
fields (`Common.modules`' `String`/`Library`-bearing `[Option<ModuleSlot>; MAX_VM]`,
module-loading LOAD-D5/D8; `cl: Option<Client>` / `snd: Option<SoundSystem>`). So
`Engine::new()` explicitly writes **`Common.time_base`**, **`Common.modules`** (the empty
`ModuleRegistry`, § `Com_Init` step 30), **`cl = None`**, and **`snd = None`** in place
before `assume_init` (§ Open questions, LIFE-Q9 — hazard class recorded there). `sv` becomes
a bare `Server`
(liveness `sv.state == SS_DEAD`, `server.h:47-54`), not `Some(Server)`. The per-`#[repr(C)]`
`unsafe impl ZeroValid` on `server_t`/`clipMap_t` still stands (it makes those members'
zero sound); non-zero init runs in `com_init` where Raven runs it. This **dissolves the
residual wrapper-assembly / stack-ordering question**, so **LIFE-Q7 is CLOSED** (there is
no field-by-field assembly left — one zeroed allocation covers every member). The
`mp_engine_core -> native_platform` edge (SP dual) is still required and stands.

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

**Terminology reconciliation (2026-07-03) — which milestone builds first.** The
sibling reading set uses **"Slice 0 (MP dedicated boot)"** for a milestone this
doc does **not** render: engine-seam.md **SEAM-D7**, state-ownership.md
§ Slice hooks, and module-loading.md § Slice hooks all agree that *their* "Slice 0"
is the **module-island** boot — our `jampgame` cdylib hosted **inside a real/OpenJK
C engine** on the `NativeDll` transport — where **the engine side is the host's C
code**, so our `Engine`/`Server`/native `SharedGameData` impl are **not built**,
and `com_init`/`com_frame`/`Engine::new` are **explicitly deferred** to "the later
our-engine-hosting slice" (state-ownership.md § Slice hooks states this verbatim,
listing that slice's `Engine` field build; module-loading.md LOAD-Q12 defers the
`SV_InitGameProgs`-equiv call site there too). This section renders **that later
our-engine-hosting slice**: the `jampded` Rust binary that constructs the full
owned `Engine` (`Engine::new`), runs the 42-step `com_init`, and drives
`loop { com_frame }`. **Build order is settled by SEAM-D7 + those two sibling
§ Slice hooks: SEAM-D7's module-island Slice 0 first, then this doc's
our-engine-hosting milestone.** They are never the same milestone; where prose
below says "slice 0"/"this slice" it means *this* our-engine-hosting milestone
(this doc's local shorthand), **not** SEAM-D7's Slice 0. (This reconciles the
term collision without a new decision — the mapping and ordering are read straight
from SEAM-D7 and the sibling § Slice hooks.)

**The our-engine-hosting slice — `jampded` bin** (the later slice above, *not*
SEAM-D7's Slice 0). The `jampded` `main()` skeleton is:

```rust
fn main() {
    // Engine::new() -> Box<Engine> (mp_engine_core, defined in engine.rs, STATE-D5)
    // allocates the WHOLE Engine as one boxed ZEROED heap buffer, then initializes
    // EVERY non-ZeroValid field IN PLACE before exposing the Box (the MaybeUninit
    // pattern; LIFE-Q9 closed 2026-07-03): Common.time_base (the Instant capture,
    // LIFE-D4b), Common.modules (the empty ModuleRegistry, step 30), cl = None,
    // snd = None — the zeroed bytes legally cover ONLY the ZeroValid-audited #[repr(C)]
    // mass (server_t/clipMap_t/…), because Rust does not guarantee all-zeros = None for
    // the niche-bearing Option fields (String/Library-bearing ModuleSlot, Client,
    // SoundSystem). NOT an `unsafe impl ZeroValid for Engine` (unsound — the aggregate
    // is not all-zeroes-valid; checkpoint-5 finding 21). The
    // dedicated Engine value builds as: common: Common; sv: Server — a bare Server
    // (NOT Option), zeroed, whose sv.state == SS_DEAD ("no map loaded", server.h:47-54)
    // IS the liveness flag — the direct dual of Raven's zero-filled file-scope statics
    // (sv_main.cpp:10,11); SV_Init/SV_SpawnServer populate it in place, exactly the cm
    // pattern below. cl: None, cm: a zeroed CollisionWorld (empty pre-CM_LoadMap state),
    // snd: None. Runs FIRST — before the warm-up read and com_init; non-zero init runs
    // in com_init where Raven runs it. The whole-aggregate zeroing (user ruling item 20;
    // LIFE-D5 amended / state-ownership STATE-D13 amended) DISSOLVES the former
    // field-by-field wrapper-assembly / stack-ordering residual: LIFE-Q7 is CLOSED, so
    // the Engine::new() CONSTRUCTOR BODY is now portable, not blocked. Pulls the
    // sanctioned `mp_engine_core -> native_platform` edge.
    let mut engine: Box<Engine> = Engine::new();
    // Raven's warm-up read (base already captured). MP keeps the `baseTime` bool
    // (LIFE-D4b); base-relative reads pass `false` (Raven's C++ default at the
    // jampded `Sys_Milliseconds()` warm-up, null/win_main.cpp:1447 → qcommon.h:978).
    // SP's mirror warm-up is the void `sys_milliseconds(&engine)`.
    let _ = sys_milliseconds(&engine, false);
    // `command_line()` is the app-bin helper that captures/joins process argv into
    // the single command string `Com_ParseCommandLine` splits (step 4) — mirroring
    // Raven's merge-argv step (jampded `null/win_main.cpp:1425`; § Entry points).
    // New-code glue in `mp/app`, not a lifecycle seam; the join detail is app-crate
    // mechanical (Raven is C, no argv helper to freeze).
    com_init(&mut engine, &command_line());     // SV_Init path; no CL_Init (dedicated)
    // Dedicated OS loop (LIFE-D4d, § jampded console adapter). `SysEventQueue::get` is a
    // PURE ring-drain with no OS pump inside (DEC-02 inversion, § SysEventQueue), so the
    // ring can only be refilled from OUTSIDE com_frame — here, in this loop. Each
    // iteration: Sleep(5) entry pacing (null/win_main.cpp:1478), then the console/net
    // poll adapters FEED the ring before com_frame drains it. `console_poll` is the
    // Sys_ConsoleInput-equivalent minimal line editor (null/win_main.cpp:200-302) that
    // queues each completed line as an SE_CONSOLE event via SysEventQueue::queue
    // (faithful Sys_QueEvent(0, SE_CONSOLE, …), null/win_main.cpp:1195); `net_poll`
    // queues SE_PACKET. Both are app/platform-bin new-code glue — like `command_line()`
    // above and the winit adapter (LIFE-Q1) — so their exact signatures are app-crate
    // MECHANICAL, NOT frozen lifecycle seams; only their BEHAVIOR is fixed (LIFE-D4d +
    // the cited oracle). Input physically arriving during com_frame's FPS-cap spin waits
    // for the next iteration's console_poll — the same one-frame DEC-02 deferral
    // documented for the winit path (§ winit-adapter boundary, "Divergence note").
    loop {
        sleep_ms(5);                 // Sleep(5) entry pacing (null/win_main.cpp:1478)
        console_poll(&mut engine);   // SE_CONSOLE (Sys_ConsoleInput inversion, LIFE-D4d)
        net_poll(&mut engine);       // SE_PACKET (LIFE-D4d)
        com_frame(&mut engine);      // drains the ring (§ Com_Frame step 5), runs SV_Frame
    }
}
```

Depends on: engine-seam.md (dispatchers — none exercised at idle until a `map`
command loads the module) **and its SEAM-D7 ordering — this our-engine-hosting
milestone follows SEAM-D7's module-island Slice 0** (Terminology reconciliation
above); state-ownership.md (`Engine`/`Common` field shapes, `ComError`, and its
§ Slice hooks "our-engine-hosting slice" `Engine` build — the same milestone this
skeleton renders); and this doc's `com_*` seam + `SysEventQueue`. Because this is
the our-engine-hosting slice, `Engine`/`Server`/`com_init`/`com_frame` **exist and
compile here** (deferred out of SEAM-D7's Slice 0, built here), so the skeleton's
`Engine::new()` and `com_frame` loop are in-scope, not a contradiction with the
sibling docs that keep them out of the *earlier* module-island slice.

**Cross-doc blockers (tracked elsewhere, not lifecycle decisions).** With STATE-Q1
resolved (= STATE-D5), the earlier crate-wiring unknowns are largely closed;
what remains external is surfaced so the porter treats it as a known dependency,
not a doc defect:

- **STATE-D5 (resolved 2026-07-02)** fixes `pub struct Engine`'s defining crate as
  `mp_engine_core`, so the `use Engine` path is pinned and `Engine::new()` (its
  constructor, capturing the `Instant` base — LIFE-D4b) is named by the skeleton
  above. Its five frozen fields build as (types owned by state-ownership.md STATE-D5,
  glossed here only so `Engine::new()` type-checks for slice 0): `common: Common`;
  **`sv: Server`** — a **bare `Server`, NOT an `Option`** (user ruling item 20,
  2026-07-03). `Server` wraps the ported `server_t`/`serverStatic_t` state, **both
  file-scope zero-init statics** (`sv_main.cpp:10,11`; state-ownership.md master table
  rows, "constructed by `SV_SpawnServer`"/`SV_Init` = *populate-in-place*). Server
  liveness is the embedded `sv.state == SS_DEAD` (`serverState_t`, `server.h:47-54`) —
  the direct dual of Raven's zero-filled `server_t sv` static, where "no map loaded" is
  `SS_DEAD` (=0); the zeroed `Server` starts `SS_DEAD` and step 31 `SV_Init` (later
  `SV_SpawnServer`) brings it live in place — the **same pattern as `cm` below**
  (`cm_load.cpp:37`; "constructed by" means populate, not first existence). This is
  **not** an `Option`-gated presence and **not** `sv: None`-until-`SV_Init` — the mixed
  presence idiom (`sv` state-gated vs `cl`/`snd` Option-gated) is deliberate. `cl: None`,
  `snd: None` on the dedicated build; **`cm: CollisionWorld`** — the collision model
  (Raven's `CM_*`/`cmodel`, e.g. the `CM_ClearMap` at § Shutdown paths), also **not** an
  `Option`: a zeroed empty state (mirroring Raven's C static zero-init of `cmg`) exists
  from `Engine::new` before any `CM_LoadMap`, so the pre-map-load `cm` value is well-defined
  and slice-0 boot type-checks. **Construction *mechanism* — SETTLED (user ruling item 20,
  2026-07-03; supersedes the round-6 per-member framing).** `Engine::new() -> Box<Engine>`
  allocates the **whole aggregate** as one boxed ZEROED heap buffer, then initializes
  **every non-`ZeroValid` field in place** before exposing the `Box` — the `MaybeUninit`
  pattern (list per the LIFE-Q9 closure: `Common.time_base`, `Common.modules`, `cl = None`,
  `snd = None`). There is deliberately **no `unsafe impl
  ZeroValid for Engine`** (the aggregate is not all-zeroes-valid — checkpoint-5 finding 21);
  `ZeroValid` covers only the `#[repr(C)]` constituents. The large embedded members —
  `server_t` = 664960 B ≈ 650 KB (`svEntity_t svEntities[MAX_GENTITIES]` by value,
  `server.h:53-88` / `crates/mp/engine/server/src/server/server_t.rs:69`) and
  `clipMap_t` (cmg + SubBSP[32]) — are zeroed by that single allocation (each still gets a
  one-line `unsafe impl ZeroValid` beside its layout asserts), so none transits the stack;
  ordinary stack-then-move `Default` is rejected (same overflow class STATE-D9 prevents).
  This **sanctions the `mp_engine_core -> native_platform` Cargo edge** (SP dual
  `sp_engine_core -> native_platform`) — `native/*` is cross-mode tier-legal, so only a new
  direct edge (STATE-D13). Because the whole `Engine` is one zeroed allocation, there is
  **no field-by-field wrapper assembly and no `Some(Server)` stack-ordering question**: the
  non-headline members (`serverStatic_t.challenges[1024]`, the CM cache pair, …) are zeroed
  by the same allocation. **LIFE-Q7 is CLOSED** — the field-by-field-assembly residual it
  owned is dissolved. The soundness point the ruling left — the non-`#[repr(C)]`,
  niche-bearing `Option` fields' zeroed-`None` — was split out as **LIFE-Q9 and is now
  also CLOSED** (2026-07-03, § Open questions): those fields (`Common.modules`, `cl`,
  `snd`) are on the explicit in-place-init list, so no zeroed-`Option` read exists and
  the constructor body is fully portable. Non-zero
  init runs in `com_init` where Raven runs it.
  The `engine.common.*` accessor this doc uses throughout (e.g. the `sys_timeBase`
  Instant base, § State ownership) is STATE-D5's frozen field: state-ownership.md
  fixes `pub struct Engine { pub common: Common, .. }` — the field name is a
  cross-ref, not assumed here.
- **`sys_error` / platform-shell seam (LIFE-Q2, RESOLVED 2026-07-02).** `com_init`'s
  init-catch and `com_error_recover`'s fatal path call `sys_error`; step 28 calls
  `Sys_Init`; `com_shutdown`'s `Sys_Quit` (`win_main.cpp:389`) round out the
  platform-shell surface `core` reaches. LIFE-D3 settles the shape: these port
  **into `core`** with their frozen signatures and delegate the actual OS work
  (print, `exit`, console/CPU detect) downhill to `native/platform` primitives — a
  normal downhill edge, **no `core`→`mp/app` reverse dependency**, no
  `Engine`-injection. (`com_printf`'s dedicated `Sys_Print` fall-through,
  `common.cpp:167`, is the same downhill delegation but from **`qcommon`**, where
  `com_printf` now lives — the com_printf amendment; qcommon depends downhill on
  `native/platform` too.) No longer a slice-0 compile gate.
- **LOAD-D8 (resolved 2026-07-02)** — module-loading.md froze the `load_module`
  signature (`ModuleRegistry::load_module -> SlotId`, resolving the former LOAD-Q3)
  that gates the registry-fill path; slice 0 does not exercise it at idle (the empty
  step-30 `ModuleRegistry` lives in `mp_engine_qcommon`, its home settled — §
  `Com_Init` step 30). Owned in module-loading.md.
- **`ModuleRegistry` → `Engine` attachment — LIFE-Q5 (RESOLVED 2026-07-02).** The
  step-30 registry hangs off **`Engine.common.modules`** — a field of `Common` (the
  qcommon-owned state struct), mirroring how Raven's `vmTable` is a qcommon-subsystem
  static (`vm.cpp:29`). `Engine`'s frozen five-field shape (`{ common, sv, cl, cm,
  snd }`, STATE-D5) is **unchanged** — the registry nests inside the existing
  `common`, reached as `engine.common.modules` (§B3 — no global; persists across
  frames as part of `Common`). state-ownership.md records the `Common.modules` row;
  `ModuleRegistry`'s type/shape stay module-loading.md's LOAD-D5. Slice 0 constructs
  it empty at step 30 and stores it there. No longer a slice-0 compile gate.

Which of the 42 MP `Com_Init` steps slice 0 **implements**, **stubs**, or leaves
**blocked** on an already-open question / externally-owned slot:

- **Implements** (the call is *wired in boot order* at each step; for the four
  subsystem-init steps `Cvar_Init` (3), `Cbuf_Init` (5), `Cmd_Init` (7) and
  `FS_InitFilesystem` (12) the slice-0 **body** is a **deliberately-callable boot-success
  no-op** carrying the `//TODO: Port <subject>` + `// Source:` + one-line-justification
  markers — LIFE-Q8 CLOSED, user item 26, 2026-07-03; real bodies land with B1/B2, where
  DEC-09.2's boot-transcript diff activates)**: 1-8, 10 (banner, push-event, cvar/cmd/cbuf
  **[boot-success stubs, LIFE-Q8 closed]**, cmdline, zone→arena,
  startup vars), **9** (the `Rand_Init` call is wired and its `Sys_Milliseconds(true)`
  raw seed value reads `SystemTime::now()` — LIFE-Q3 resolved, § Timing), 12
  (`FS_InitFilesystem` **[boot-success stub, LIFE-Q8 closed]**), **13 `Com_InitJournaling`**
  (LIFE-D4a), 14-18 (config execs incl. `jampserver.cfg`, `Cbuf_Execute`,
  re-override), **19 `com_dedicated="2"` ROM**, 20 (hunk→arena), 21-26 (cvar
  block, viewlog force, quit/writeconfig, version), **28** (`Sys_Init` CPU-detect
  logic is oracle-derivable, `IN_Init` = null stub; its `core`→platform call is the
  LIFE-Q2-resolved downhill `native/platform` delegation — no longer blocked),
  29 (`Netchan_Init`, its port seed = `com_milliseconds`, the journaled reader,
  § Seam), **30** (the empty `ModuleRegistry` + transport-select cvars are
  settled — step 30 above; module *load* is post-slice-0 at `SV_SpawnServer` — and the
  registry is stored at **`Engine.common.modules`**, LIFE-Q5 resolved), 31
  (`SV_Init`), 32, 34 (`com_frameTime`, from `com_milliseconds` — § Seam, § Timing),
  35 (`Com_AddStartupCommands`; no cinematic),
  40-41 (`fullyInitialized`, banner).
- **Not blocked (LIFE-Q7 CLOSED 2026-07-03, user ruling item 20).** Every
  `Com_Init`/`Com_Frame` *step* above is implementable or stubbable, **and** the
  `Engine::new()` constructor body — previously the one blocker — is now portable: the
  whole-Engine boxed-zeroed ruling (`Engine::new() -> Box<Engine>` = one zeroed allocation
  + `MaybeUninit` in-place init of `Common.time_base`; `sv: Server`, liveness
  `sv.state == SS_DEAD`) **dissolves** the former residual wrapper-assembly / stack-ordering
  question (LIFE-D5 amended / STATE-D13 amended), so there is nothing left to invent. The
  round-3 session had already closed the last step-level forks — step 9's raw RNG seed
  (LIFE-Q3, now `SystemTime::now()`), step 30's `Engine` attachment slot (LIFE-Q5, now
  `Engine.common.modules`), and the `SE_CONSOLE` payload drain's `SysEventQueue`
  representation (LIFE-Q4, now the confined `Box` round-trip); the earlier
  `sys_error`/platform-shell up-tier blocker (LIFE-Q2) was resolved — `sys_error`/`Sys_Init`
  (step 28)/`com_shutdown`'s `Sys_Quit` port into `core` and delegate downhill to
  `native/platform` (LIFE-D3), and `com_printf`'s dedicated `Sys_Print` fall-through does
  the same from `qcommon` (the com_printf amendment). The step-level forks that once
  blocked the boot order are thus closed. **LIFE-Q1** (the winit-keycode → `keynum_t` map)
  does **not** touch this `jampded` milestone (no winit) — it is first exercised by the
  `jamp` client slice. **The two round-7-gate questions that touched this milestone are
  both CLOSED (2026-07-03):** **LIFE-Q9** — the `MaybeUninit` in-place-init list is
  every non-`ZeroValid` field (`Common.time_base`, `Common.modules`, `cl = None`,
  `snd = None` written explicitly before `assume_init`; zeroed bytes cover only the
  `ZeroValid`-audited `#[repr(C)]` mass), so the constructor body is sound and portable;
  and **LIFE-Q8** (user, item 26) — steps 3/5/7/12 are deliberately-callable boot-success
  no-ops with the mandated `//TODO: Port <subject>` markers; real bodies land with B1/B2,
  where the DEC-09.2 boot-transcript diff activates. The milestone is buildable
  end-to-end (§ Open questions).
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
- **~~LIFE-Q8~~ — slice-0 body depth of the FS/Cvar/Cmd/Cbuf subsystem-init steps —
  CLOSED (user, item 26, 2026-07-03).** **Resolution: boot-success stubs.** The slice-0
  `com_init` steps 3/5/7/12 (`Cvar_Init`, `Cbuf_Init`, `Cmd_Init`, `FS_InitFilesystem`)
  are **deliberately-callable boot-success no-ops** carrying the porting-rules markers —
  the sanctioned rare deliberate-no-op form: each stub site carries
  `//TODO: Port <subject>` + `// Source:` + a one-line justification (porting-rules
  § Unported-work markers). The **real** subsystem ports land with **B1** (cvar-cmd — steps
  3/5/7) and **B2** (filesystem — step 12); **at that point DEC-09.2's boot-transcript diff
  activates for this path** — until then the transcript comparison for the FS-dependent
  steps (14-18 config execs, the step-22 `Cvar_Get` block) is out of scope for slice 0, by
  this ruling rather than silent divergence. The entry's original facts stand as the
  rationale for *why* this needed a user ruling (`FS_InitFilesystem`'s real pk3 work +
  `mpdefault.cfg` `ERR_FATAL`, MP `common.cpp:1266` / SP `:1001`; steps 8/14/22 exercising
  cvar/cbuf during the same boot) — the ruling scopes those behaviors to B1/B2, not
  slice 0. *Rejected:* blocking slice 0 on real (minimal) FS/Cvar/Cmd/Cbuf ports (defers
  the boot milestone for subsystems owned by their own docs); porting them inline
  (subsystem internals are a § Scope non-goal).
- **~~LIFE-Q9~~ — soundness of the whole-Engine zeroed construction for the
  non-`#[repr(C)]`, niche-bearing fields — CLOSED (mechanical generalization,
  2026-07-03; round-7 gate extension to `cl`/`snd` folded in).** **Resolution: the
  `MaybeUninit` in-place-init list is EVERY non-`ZeroValid` field, not just
  `Common.time_base`.** The zeroed bytes of the `Box<Engine>` buffer legally cover
  **only the `ZeroValid`-audited `#[repr(C)]` mass** (`server_t`/`clipMap_t`/…);
  everything else is written explicitly before `assume_init`: `Engine::new()` in-place
  writes **`Common.time_base`** (the `Instant` capture), **`Common.modules`** (the
  already-settled empty `ModuleRegistry` build, § `Com_Init` step 30), **`cl = None`**,
  and **`snd = None`**. **Hazard class (recorded — what the round-7 gate found):** a
  zeroed `String` has a null data pointer (violates its `NonNull` invariant — the exact
  `zeroed_box::<String>()` unsoundness STATE-Q10 flagged), and Rust does **not**
  guarantee all-zero bytes = `None` for an arbitrary `Option<T>` (niche guarantees cover
  only specific types like `Option<NonNull>`/`Option<Box>`/`Option<&T>`) — so
  `Common.modules`' `Option<ModuleSlot>` slots (`ModuleSlot` carries `String` +
  `libloading::Library`, LOAD-D5/LOAD-D8) and equally `cl: Option<Client>` /
  `snd: Option<SoundSystem>` (idiomatic non-`#[repr(C)]` wrappers, no `ZeroValid` impl)
  could not soundly be read out of zeroed bytes. Writing them in place removes every
  such read; no per-type niche assertion is ever made. This is the mechanical
  generalization of the settled item-20 `MaybeUninit` mechanics, not a new decision.
  LIFE-D5's amendment and the § Slice hooks `Engine::new` mechanics render this list.
- **~~LIFE-Q7~~ — engine-island `Engine::new` construction — CLOSED (user ruling
  item 20, 2026-07-03).** The residual this question owned — the field-by-field
  wrapper assembly and stack-safe field order of `Engine::new`'s `Server`/`CollisionWorld`
  members — is **dissolved** by the user's whole-aggregate ruling: `Engine::new() ->
  Box<Engine>` allocates the **entire** `Engine` as one boxed ZEROED heap buffer, then
  initializes its few non-zero-valid fields (currently `Common.time_base:
  std::time::Instant` — unspecified layout, no all-zero validity) **in place** before the
  `Box` is exposed (the `MaybeUninit` pattern). Because the whole value is one zeroed
  allocation, there is **no field-by-field assembly and no `Some(Server)` stack-ordering
  question left** — every member, headline or not (`serverStatic_t.challenges[1024]`,
  `bot`, `master_heartbeat`, SP `savegame`, the CM cache pair), is zeroed by that single
  allocation; `sv` is a bare `Server` (liveness `sv.state == SS_DEAD`), not `Some(Server)`.
  There is deliberately **no `unsafe impl ZeroValid for Engine`** (the aggregate is not
  all-zeroes-valid — checkpoint-5 finding 21); the `ZeroValid` impls cover only the
  `#[repr(C)]` constituents (`server_t`/`clipMap_t`/…). Non-zero init (`SV_Init`/
  `CM_LoadMap`/…) runs later in `com_init`, exactly where Raven runs it. So the
  **field-by-field wrapper-assembly / stack-ordering** residual LIFE-Q7 owned is dissolved.
  (The soundness question the whole-aggregate ruling left — whether the all-zero buffer is
  a sound value for the non-`#[repr(C)]`, niche-bearing `Option` fields — was split out as
  **LIFE-Q9 and is now also CLOSED** (2026-07-03): every non-`ZeroValid` field —
  `Common.time_base`, `Common.modules`, `cl`, `snd` — is written in place before
  `assume_init`, so no zeroed-`Option` read exists and the skeleton's first line is fully
  unblocked.) See LIFE-D5 (amended)
  and state-ownership STATE-D13 (amended) / § Engine amendment. *Rejected (superseded):* the
  round-6 per-member `zeroed_box` + stack-safe-wrapper-assembly framing (LIFE-D5 / STATE-D13
  original), which the whole-aggregate ruling makes moot.
- **LIFE-Q6 — RESOLVED 2026-07-03 (→ LIFE-D3 amendment 2026-07-03, § Seam).** The
  recursive-error catch topology is settled: `com_frame`'s (and `com_init`'s) catch arm
  wraps the `com_error_recover` call in its **own** `catch_unwind`; a second `ComError`
  caught there while `errorEntered` is still set routes to `sys_error("recursive error
  after: {saved}")`, where `saved` is the *first* error's still-intact message
  (`com_error` is receiverless — STATE-Q4). This reproduces Raven's recursive-error
  banner + controlled `exit` (MP `common.cpp:288` / SP `:265`) that DEC-09.2
  boot-transcript diffing matches. The frozen `com_frame` snippet (§ Seam) now shows the
  inner catch. `sys_error`'s `(engine, msg) -> !` seam is unaffected — only the caller
  structure gained the inner catch. Retained here as a breadcrumb; the record is the
  LIFE-D3 amendment.
- **LIFE-Q2 — RESOLVED 2026-07-02 (→ LIFE-D3, § Seam).** How `core` reaches
  `sys_error` and the other platform-shell `Sys_*` calls (`Sys_Init`,
  `Sys_ShowConsole`, `Sys_Quit`) is settled: they port **into `mp_engine_core`** with
  their frozen signatures and delegate the OS work (print, `exit`, console/CPU detect)
  downhill to `native/platform` primitives — no `core`→`mp/app` reverse edge, no
  `Engine` injection. (`com_printf`'s dedicated `Sys_Print` fall-through is the same
  downhill delegation but from `qcommon`, where `com_printf` now lives — the com_printf
  amendment.) The Win32 message-box/console-show surface of `Sys_Error` is deferred to
  the client-shell slice. Retained here as a breadcrumb; the record is LIFE-D3.
- **LIFE-Q3 — RESOLVED 2026-07-02 (→ LIFE-D4b amendment, § Timing).** The
  `Sys_Milliseconds(true)` raw variant reads **`SystemTime::now()` — unix-epoch ms
  truncated to `i32`** as its own absolute seed source for `Rand_Init` at `Com_Init`
  step 9 (Raven used `timeGetTime()`, ms-since-OS-boot; `common.cpp:1248`,
  `win_shared.cpp:22-34`). The base-relative variant stays `Instant`-backed (LIFE-D4b).
  One-line divergence: same *role*, different absolute *origin* — below the
  differential seam, RNG-from-clock seeding was never cross-run deterministic. Step 9
  now fully implements. Retained here as a breadcrumb; the record is the LIFE-D4
  amendment.
- **LIFE-Q4 — RESOLVED 2026-07-02 (→ § `SysEventQueue`).** `SysEventQueue`'s internal
  representation is the **`Box` round-trip**: the pinned `sys_event_queue.rs` owns the
  codebase's one `Box::into_raw`/`from_raw` pair — `queue` stores the raw pointer in
  the faithful `sysEvent_t.evPtr`, `get` re-wraps to an owned `Box`, overflow eviction
  reconstitutes-and-drops. Invariant (stated once at the type): every non-null `evPtr`
  in the ring came from `Box::into_raw`. This is a §D11-confined seam surface, not a
  violation — `sysEvent_t` is the Raven ABI `#[repr(C)]` shape, so the ring **is** the
  seam where the raw pointer lives. Public `queue`/`get` signatures unchanged.
  Retained as a breadcrumb; the record is § `SysEventQueue`.
- **LIFE-Q5 — RESOLVED 2026-07-02 (shared with state-ownership.md; → § `Com_Init`
  step 30).** The step-30 `ModuleRegistry` hangs off **`Engine.common.modules`** — a
  field of `Common` (the qcommon-owned state struct), mirroring Raven's `vmTable` as a
  qcommon-subsystem static (`vm.cpp:29`). `Engine`'s frozen five-field shape
  (`{ common, sv, cl, cm, snd }`, STATE-D5) is **unchanged** — the registry nests
  inside the existing `common`, reached as `engine.common.modules` (§B3 — no global;
  persists as part of `Common`). state-ownership.md records the `Common.modules` row;
  `ModuleRegistry`'s type/shape stay module-loading.md's LOAD-D5. Retained as a
  breadcrumb; the record is § `Com_Init` step 30.
