# State Ownership Design
Status: DRAFT     Supersedes: none
Decision prefix: STATE     Ledger deps: DEC-01, DEC-03, DEC-04, DEC-05, DEC-07, DEC-08, DEC-09

## Standing context

Links only — never restated here:

- `docs/workspace-architecture.md` — crate graph and tiers (`native/*`,
  `abi-transport`, `crates/{mp,sp}/{qshared,bg,uishared,abi,game,cgame,ui}`,
  `crates/{mp,sp}/engine/*`, `crates/{mp,sp}/app`).
- `docs/porting-rules.md` — §B is the spine this doc makes concrete (no
  `static mut`/no ambient globals §B3, state threaded not reached §B4, entities
  by index/handle §B5, one owned instance per singleton §B6); §C9
  (alloc/free → ownership); §D11 (unsafe confined to the seam); §D12
  (`#[repr(C)]` layout parity).
- `docs/decisions.md` — DEC-01 (renderer port deferred), DEC-03 (audio: faithful
  mixer via cpal, EAX/force-feedback dropped), DEC-04 (per-mode
  duplication), DEC-05 (module transport `NativeDll | Static | Wasm`, WASM
  first-class), DEC-07 (SP cgame/ui statically linked via the vmachine shim),
  DEC-08 (`Com_Error` = panic + `catch_unwind` at the frame boundary), DEC-09
  (verification layers).
- `docs/architecture/two-island-model.md` — the STATE-D1 visualization,
  incorporated as § State ownership → The two-island model below.
- `docs/architecture/engine-seam.md` — the typed ABI seam and its executors
  (SEAM-D1..D10). It **defers `SharedGameData`'s method set to this doc**
  (engine-seam § `SharedGameData` (informative)); this doc freezes it. Seam-type
  names are kept consistent with that doc, and their **concrete shapes are
  defined there, not restated here** (read engine-seam.md for them): `CEngine`
  (engine-seam § `CEngine`, SEAM-D9), `Static` (engine-seam § `Static`),
  `ModuleTransport` = `enum { NativeDll, Static, Wasm }` (engine-seam
  § Engine-side dispatchers), the `ServerGame` dispatcher arg (engine-seam
  § dispatchers — the reborrowed `Engine.sv` host state), and `SharedGameData`
  (whose *method set* this doc freezes below).
- `docs/abi-traps.md` — the generated `trap_*` signature reference; the seam
  below stores the payloads of rows 18 (`trap_LocateGameData`),
  19 (`trap_DropClient`), and 121 (`trap_SV_RegisterSharedMemory`).
- `docs/dossiers/A2-state-ownership.md` — the survey this doc renders.

## Scope & non-goals

**This doc decides:** the state-ownership spine (porting-rules §B made
concrete) — the master table mapping every Raven engine- and module-tier global
**the A2 survey censused** (qcommon/server/client/sound/renderer + the module
tiers; the four §F engine subcrates botlib/ghoul2/icarus/rmg were outside that
survey and are placeholdered pending their own §F docs, STATE-Q2) to its owning
Rust struct/crate/threading; the two-island model (one owned
`Engine`, one owned `GameWorld`); `GameWorld`'s shape **and** its physical
storage location across `vmMain` calls — the `WorldCell` static in the module
cdylib shell (STATE-D6, resolving the former STATE-Q3);
the reentrancy contract
(reborrow-threading + `EntityId` discipline, no `RefCell`, no effect queues);
and the error-recovery ordering contract (`com_error` recovers synchronously
before panicking). It **freezes `SharedGameData`** (deferred here by engine-seam).

**Non-goals** (each punted to its owning doc):

- **Seam executor mechanics** — how a trap crosses (`Execute<C>`/`Dispatch<C>`,
  the syscall/table wires, WASM marshalling) → `docs/architecture/engine-seam.md`.
- **Per-subsystem internals** — what each owned struct's fields are and what each
  handler *does* (cvar parse tables, filesystem search-path logic, sound mixer,
  collision internals) → `docs/subsystems/*` (pending).
- **Lifecycle / boot & shutdown sequences** — construction *order*, headless
  boot, the `ErrorLevel` enum variants (LIFE-D3) → `docs/architecture/lifecycle.md`
  (pending). This doc references `ErrorLevel`; lifecycle.md freezes it.
- **Renderer globals** (`tr`/`backEnd`/`glConfig`) — deferred wholesale per
  DEC-01; the master table lists them pointing at that deferral, nothing more.

## Raven ground truth

The full ~60-row census lives in the dossier; § State ownership's master table is
this doc's rendering of it. This section covers only the *architecture* the table
rests on. MP tree = `oracle/oracle/codemp/`, SP tree = `oracle/oracle/code/`.

### Build-canonical file variants

The real PC build links `cmd_common.cpp`+`cmd_pc.cpp`,
`files_common.cpp`+`files_pc.cpp`, `z_memman_pc.cpp`
(`oracle/oracle/codemp/unix/makefile:302-307`). `files.cpp`, `cmd_console.cpp`,
`files_console.cpp`, `z_memman_console.cpp` are dead console-platform duplicates
(same globals under `static` linkage) and are **not** owners of anything in the
table.

### The dirtiest cross-boundary offenders

Four clusters are reached from far outside their home subsystem and force the
"one struct threaded" discipline hardest:

1. **`sv`/`svs`** (`server_t`/`serverStatic_t`, def `sv_main.cpp:11,10`) — read
   from `icarus/GameInterface.cpp:129,402,411`, `qcommon/msg.cpp:1268`
   (`sv.state`), `qcommon/RoffSystem.cpp:612,828`, `RMG/RM_Instance_BSP.cpp:262`
   (`sv.entityParsePoint`), `server/NPCNav/navigator.cpp:1409` (`svs.time`).
2. **`cls`** (`clientStatic_t`, `cl_main.cpp:107`) — `qcommon/files*.cpp` read
   `cls.state`/`cls.keyCatchers` for pure-pak restart gating
   (`files.cpp:1247,1375`, `files_pc.cpp:848,977`), plus win32/unix input code.
3. **`cmg` + the map-disk-image cache pair** (`gpvCachedMapDiskImage`/
   `gbUsingCachedMapDataRightNow`, `cm_load.cpp:568,570`) — shared
   CM↔renderer (`renderer/tr_bsp.cpp`); renderer half is DEC-01-deferred.
4. **`com_*` cvar handles + `cvar_modifiedFlags`** (`common.cpp:37-72`,
   `cvar.cpp:8`) — touched from nearly every tier (`com_sv_running`/
   `com_cl_running` reach client, server, renderer, ghoul2).

Cleanly encapsulated *already* (near-zero redesign): the VM table (qcommon-
private, handle-based — engine-seam owns it), `cmd_functions`/`cvar_vars`
(API-only access), the sound mixer (syscall-fed), and `kg` (`keyGlobals_t`,
`cl_keys.cpp:17` — Raven already pre-bundled the key state into one struct: the
template for a Rust `KeyState`).

### SP deltas

Per DEC-04 the tables are per-mode. The load-bearing SP divergences:

- **`cmd_functions` is a fixed array** `cmd_function_t[CMD_MAX_NUM=256]`
  (`code/qcommon/cmd.cpp`, single un-split file) — matching MP's *dead*
  `cmd_console.cpp` variant, **not** MP-canonical `cmd_pc.cpp`'s heap linked
  list. A real cross-branch divergence to preserve knowingly (STATE-D4).
- **No pure-server pak-validation globals at all** — `fs_serverPaks*`,
  `fs_serverReferencedPaks*`, `lastValidBase/Game`, `fs_fakeChkSum`,
  `fs_checksumFeed`, `fs_loadStack` are absent (single-player has nothing to
  validate against).
- **`server_t`/`serverStatic_t` leaner** (`code/server/sv_main.cpp:18-19`):
  drops `restarting`/`checksumFeed`, the whole `challenges[1024]` table, and
  all VM-registration fields (`gentities`/`gameClients`/`mSharedMemory` — a
  statically-linked game needs no handoff). SP **adds** save-game state:
  `qbLoadTransition` and `SavedGameJustLoaded_e eSavedGameJustLoaded`
  (`sv_ccmds.cpp:22`, mutated `:288,311`, consumed `sv_client.cpp:483`).
- **`MAX_FILE_HANDLES` = 16 on SP** vs **64 on PC MP** (16 is `#ifdef _XBOX`-only)
  (MP define `codemp/qcommon/qcommon.h:508-510`; SP define `code/qcommon/files.h:58`;
  array decl `files_common.cpp:202`) — a per-mode array-size divergence, STATE-D4.
- **`FS_ReadFile`'s live path uses `Z_Malloc`** (its hunk calls are commented
  out) — so the read buffer is an ordinary owned allocation, not a hunk
  lifetime, in both trees (STATE-D4).

### The SP static-link reintroduced-globals hazard

Because SP's cgame/ui are compiled into the engine binary, their formerly
VM-private state is plain process-wide globals in the same static segment as
`sv`/`cl`: `cg_t cg; cgs_t cgs; centity_t cg_entities[MAX_GENTITIES];`
(`code/cgame/cg_main.cpp:210-212`) and `uiInfo_t uiInfo;`
(`code/ui/ui_main.cpp:315`). MP declares the identical globals
(`codemp/cgame/cg_main.c:691`, `codemp/ui/ui_main.c:875`) but they live inside a
separate DLL image — **the difference is isolation, not presence.** The Rust
port does **not** inherit SP's regression: SP cgame/ui state becomes an owned
struct threaded in (STATE-D4), MP's isolation as the behavioral model.

### Reentrancy chains (the forcing functions for STATE-D1/D3)

**Chain A — server frame re-enters engine world-linking state.** `SV_Frame`
(`sv_main.cpp:826`) holds `sv.timeResidual`/`svs.time` locals across
`VM_Call(gvm, GAME_RUN_FRAME, ...)` (`sv_main.cpp:909-915`) → `G_RunFrame`
(`g_main.c:3582`) whose outer loop holds a live `gentity_t*` across nested calls
(`g_main.c:3741`) → `G_RunMover`/`ClientThink_real` call `trap_LinkEntity`
mid-iteration (`g_mover.c:402`, `g_active.c:3470`) → `SV_LinkEntity`
(`sv_world.cpp:189`) splices the **world-sector linked list** (`sv_world.cpp:342`)
— a structure *disjoint* from `g_entities` — while `SV_AreaEntities_r`
(`sv_world.cpp:373`, reachable as `G_ENTITIES_IN_BOX` from the same nested graph
via `G_TouchTriggers`) walks that same list, defensively pre-fetching `next`
before evaluating each node (`sv_world.cpp:381`) — a tell the authors knew the
list mutates under the walk. This cannot compile as safe Rust if `g_entities`,
`sv_worldSectors`, and `sv.svEntities` are fields of one `&mut`-borrowed struct:
the borrow checker cannot see across the indirect `VM_Call` that the outer
borrow (entities) and inner call (world sectors) touch disjoint fields. **This is
why the two islands exist** (STATE-D1).

**Chain B — `Com_Error` unwinds through a reentrant VM call.** `Com_Error`
(`common.cpp:249`) is **C++ exceptions**, not setjmp/longjmp (no `jmp_buf` in
codemp). `ERR_DROP` prints, calls `SV_Shutdown` (`common.cpp:315`) →
`SV_ShutdownGameProgs` → `VM_Call(gvm, GAME_SHUTDOWN)` + `VM_Free(gvm)`
(`sv_game.cpp:1670-1672`, a *new reentrant call* into the same VM that frees it),
`CL_Disconnect`/`CL_FlushMemory` (`common.cpp:316-317`), **then**
`throw ("DROPPED\n")` (`common.cpp:326`), caught at `Com_Frame`
(`common.cpp:1762`). Recovery runs entirely **before** the throw. A concrete
deep throw site: `SV_SvEntityForGentity` (`sv_game.cpp:70-75`) throws `ERR_DROP`
from several native frames deep inside a live `GAME_RUN_FRAME`. Raven's
`VM_Call` manual save/restore of `currentVM` (`vm.cpp:799-827`) is *skipped* on
the unwind path — harmless only because the whole VM was already torn down. **This
is why STATE-D3 orders recovery before panic and forbids un-guarded save/restore.**

**Chain C — cgame draw accumulates into the renderer (the easy foil).**
`CL_CGameRendering` (`cl_cgame.cpp:1830`) issues one opaque `VM_Call(cgvm,
CG_DRAW_ACTIVE_FRAME, ...)` (`cl_cgame.cpp:1842`); cgame calls
`trap_R_AddRefEntityToScene` dozens of times, each appending to
`backEndData->entities[r_numentities++]` (`tr_scene.cpp:224-254`), then one
`trap_R_RenderScene` consumes the count (`tr_scene.cpp:796`). The outer caller
holds **nothing live** across the `VM_Call` — a pure accumulate-then-flush
builder, not an aliasing hazard. It confirms not every reentrancy needs Chain A's
machinery; only chains where an outer loop holds a live borrow into the state the
reentrant call mutates do.

### Shared memory (the entity aliasing that must not fight the borrow checker)

- **MP `G_LOCATE_GAME_DATA`** (`g_public.h:145`). `G_InitGame` calls once
  (re-issued on entity-count change, `g_utils.c:848`):
  `trap_LocateGameData(level.gentities, level.num_entities, sizeof(gentity_t),
  &level.clients[0].ps, sizeof(level.clients[0]))` (`g_main.c:997-998`). Note it
  passes **`&level.clients[0].ps`** (the leading `playerState_t`, not the client
  base) with **`sizeof(gclient_t)`** stride: the server only ever reads each
  client's leading `playerState_t` at `gclient_t`-sized strides
  (`gclient_s`'s `ps` is field 0 by contract, `g_local.h:537`). `SV_LocateGameData`
  stores the raw pointers/strides into `sv` (`sv_game.cpp:327-335`); the engine
  then dereferences them dozens of times per frame **outside any trap** by stride
  arithmetic (`SV_GentityNum`/`SV_NumForGentity`, `sv_game.cpp:46-65`, whose
  comment states these must be used *instead of* pointer arithmetic because the
  game allocates private data after the shared part). This is **genuine pointer
  aliasing into game-owned memory**, preserved exactly (§A1/§A2 — no
  copy-narrowing).
- **`sv.mSharedMemory`** — `G_InitGame` unconditionally calls
  `trap_SV_RegisterSharedMemory(gSharedBuffer)` (`g_main.c:920`); the
  `G_SET_SHARED_BUFFER` handler stores `VMA(1)` into `sv.mSharedMemory`
  (`sv_game.cpp:940`; field `server.h:87`). This is **live, not dead**: the buffer
  is the high-arity `vmMain` escape (read by `C_Trace`/`GAME_ICARUS_*`) and is
  consumed by icarus/RMG — a *second* register-once/read-later registration in
  the same family as `LocateGameData`.
- **No address translation on the MP game boundary (on the real native-DLL
  target).** `VMA(x)` — the macro every `G_*` syscall uses to read a pointer
  argument — is conditional (`sv_game.cpp:401-404`): the direct-cast form
  `((void*)args[x])` is gated `#if __linux__ && __powerpc__`, and the **default/PC
  branch is `VM_ArgPtr(args[x])`**. But `VM_ArgPtr` **degenerates to an identity
  cast for a native-DLL VM**: for an `entryPoint`-set VM it returns
  `dataBase + intValue` (`vm.cpp:648-649`), and `dataBase` **stays 0 for native
  DLLs** — it is assigned only in the QVM bytecode-load paths (`vm.cpp:444,560`),
  whereas the DLL load path sets only `entryPoint` (`Sys_LoadDll`, `vm.cpp:518`)
  and `vmTable` is a zero-initialized file-scope static (`vm.cpp:29`). So on the
  real PC native-DLL target `VMA(x) == args[x]`, no translation. The masking
  branch (`& dataMask`, `vm.cpp:652`) applies only to classic QVM bytecode, which
  is out of scope (DEC-05.4). The MP `game` boundary is therefore hard-wired to a
  real native DLL sharing the engine's address space; model MP `game` as
  always-native real shared memory, and `SharedGameData`'s `NativeDll`/`Static`
  impl does raw base+stride arithmetic with zero translation.
- **SP `GetGameAPI` table handoff.** SP has **zero** `LocateGameData` call sites
  (grep-confirmed). `GetGameAPI` (`code/game/g_main.cpp:875`) returns a struct of
  real fn/data pointers; `InitGame` sets `globals.gentities = g_entities;
  globals.num_entities = MAX_CLIENTS; globals.gentitySize = sizeof(gentity_t)`
  (`g_main.cpp:736,749`). The engine dereferences `ge->gentities`/`ge->gentitySize`
  exactly like MP's `sv.gentities` (`code/server/sv_game.cpp:46,55`) — still real
  aliasing, but a one-shot struct-of-pointers, and with **no `gameClients`
  equivalent** (SP has no `g_clients`, `MAX_CLIENTS = 1`).
- **`CL_GetSnapshot` copies even for native DLLs** (`cl_cgame.cpp:187-202`, SP
  `code/client/cl_cgame.cpp:181`): a deliberate field-by-field copy out of
  `cl.snapshots[]`/`cl.parseEntities[]` into the caller's buffer. cgame owns its
  storage; the copy is preserved as-is (§A2) — it does **not** generalize to the
  `game`↔server boundary, which stays true aliasing.

## State ownership

### The two-island model (STATE-D1)

One owned per-mode **`Engine`** struct (engine island) and one owned per-mode
**`GameWorld`** struct (module island) interact across the raw ABI seam without
fighting the borrow checker. Reproduced from `two-island-model.md`:

```
     ENGINE SIDE (one owned Engine)         MODULE SIDE (one owned GameWorld)
main()
 +-> com_frame(&mut Engine)
      +-> sv_frame(&mut Engine)
           |  loop {
           |    engine.sv.time += msec;   <-- fields used BETWEEN calls only;
           |                                  no borrow held ACROSS the call
           |    vm_call(GAME_RUN_FRAME) ----------.
           |  }                                    v
           |                        g_run_frame(&mut GameWorld, svc)
           |                          for id in 0..num_entities {
           |                            // EntityId re-borrow: index world.entities[id]
           |                            // briefly, release before any nested call
           |                            g_run_mover(&mut world, id, svc)
           |                          }            |
           |                                       v
           |                        trap LinkEntity(raw *mut ent)
           |     ===================== THE RAW SEAM =====================
           |     =  LocateGameData registered base+stride with engine   =
           |     =  at init. Engine reads/writes entity memory ONLY     =
           |     =  through these raw pointers (unsafe, rules §D11).    =
           |     =  Borrow checker never sees the aliasing, by design.  =
           |     =  Identical contract in NativeDll AND Static builds.  =
           v     ========================================================
      sv_link_entity(&mut engine.sv.world_sectors, ent_view)
           v
      mutates world-sector lists (engine-owned, DISJOINT from GameWorld)
```

Three load-bearing tricks: (1) **reborrow-threading** — `&mut Engine` flows
*down*; callers keep only locals across nested calls, matching oracle's `SV_Frame`
(`sv_main.cpp:909-915`); (2) **`EntityId` re-borrow discipline** — module logic
passes `(world, id)` and re-indexes per access (§B5, the GP2 arena precedent);
(3) **the raw seam** — engine↔module entity aliasing is `unsafe` pointer
arithmetic behind `SharedGameData`, confined per §D11, identical in `NativeDll`
and `Static` transports so the borrow checker never sees the aliasing. **No
`RefCell`, no effect queues** — queues would break same-frame link-then-query
parity (`G_TouchTriggers` → `trap_EntitiesInBox` observing entities linked
earlier in the same frame). Seam dispatchers/entrypoints are `extern "C-unwind"`
(engine-seam SEAM-D10) so `com_error` panics traverse live C frames when hosting
real DLLs.

**Where each island physically lives (STATE-D5/D6).** The engine island is a
`pub struct Engine` **defined** in the per-mode facade crate `mp_engine_core`
(SP mirror `sp_engine_core`) — the crate that depends on all the engine subcrates
— and **instantiated** by `Engine::new` called from the thin `mp/app` bin shell
(STATE-D5); `&mut Engine` threads down from there, never a `static`. The module
island's single owned `GameWorld` is stored in a `WorldCell` **static** in the
module cdylib shell crate (`crates/jampgame`/`crates/cgame`/`crates/ui`, SP
`jagame`), the second sanctioned static exemption (STATE-D6, §D11) — its shape and
access discipline are frozen in § Seam definition. Both statics are
module-island / seam-shell state (`WorldCell` beside engine-seam's
`ENGINE: OnceLock<CEngine>`, SEAM-D1); no logic crate holds either.

### Master table

Owner = `crate::Type.field`. "threaded via" is the reborrow discipline above
unless noted. Engine-island structs live under `mp/engine/*` (SP mirror under
`sp/engine/*`, DEC-04) and are constructed by their Raven `*_Init` and owned by
the single `Engine` created in `main()` (`mp/app`); module-island structs live in
the module crate and are constructed by `G_InitGame`. Grouped by subsystem.

**qcommon — Common (`mp/engine/qcommon::Common`, a field of `Engine`)**

`Common` is a **new** top-level module added to `mp_engine_qcommon`'s `lib.rs`
alongside its existing `cm`/`files`/`gp2`/`miniheap`/`qcommon`/`qfiles`/`timing`/
`vm` modules (no `common` module exists there yet); it owns the `common.cpp`
globals below plus the `cvars`/`cmd`/`cbuf`/`fs`/`net` sub-structs.

| Raven global | oracle cite | Rust owner | constructed by | threaded via |
|---|---|---|---|---|
| `com_frameTime`/`com_frameMsec`/`com_frameNumber` | `common.cpp:79-81` | `Common.frame_{time,msec,number}` | `Com_Init` | `&mut Engine` |
| `com_errorEntered`/`com_errorMessage[4096]` | `common.cpp:83,86` | `Common.error` (re-entry guard + last msg) | `Com_Init` | `&mut Engine`; see STATE-D3 |
| `rd_buffer`/`rd_buffersize`/`rd_flush` | `common.cpp:92-94` | `Common.redirect` | `Com_BeginRedirect` | `&mut Engine` |
| `com_journalFile`/`com_journalDataFile` | `common.cpp:34-35` | `Common.journal` | `Com_InitJournaling` | `&mut Engine` |
| `com_pushedEvents[1024]`/`Head`/`Tail` | `common.cpp:749-752` | `Common.event_queue` | `Com_InitJournaling` | `&mut Engine` |
| `com_argc`/`com_argv[]`/`com_consoleLines[32]`/`com_numConsoleLines` | `common.cpp:22-23,387-388` | `Common.cmdline` (startup) | `Com_ParseCommandLine` | `&mut Engine` |
| ~30 `cvar_t*` handles (`com_dedicated`, `com_sv_running`, `com_cl_running`, …) | `common.cpp:37-72` | `Cvar` handles resolved from `Common.cvars` | `Cvar_Get` at init | handle, not raw ptr |

**qcommon — Cvar / Cmd / Cbuf (fields of `Common`)**

| Raven global | oracle cite | Rust owner | constructed by | threaded via |
|---|---|---|---|---|
| `cvar_vars`/`cvar_indexes[1224]`/`cvar_numIndexes`/`hashTable[256]`/`cvar_cheats`/`cvar_modifiedFlags`/pool | `cvar.cpp:6-21` | `Common.cvars: CvarSystem` | `Cvar_Init` | `&mut Engine` |
| `cmd_text`+`cmd_text_buf`/`cmd_wait` | `cmd_common.cpp:16-18` | `Common.cbuf: Cbuf` | `Cbuf_Init` | `&mut Engine` |
| `cmd_argc`/`cmd_argv[]`/`cmd_tokenized[]` (shared tokenizer scratch) | `cmd_common.cpp:290-292` | `Common.cmd: CmdSystem` (**owned** scratch, STATE-D4) | `Cmd_Init` | `&mut Engine` |
| `cmd_functions` | MP `cmd_pc.cpp:11`; SP `code/qcommon/cmd.cpp` | `Common.cmd: CmdSystem` — **MP heap list / SP fixed `[_;256]`, per-mode divergence preserved** (STATE-D4) | `Cmd_AddCommand` | `&mut Engine` |

**qcommon — FileSystem / Net / Collision (fields of `Engine`)**

| Raven global | oracle cite | Rust owner | constructed by | threaded via |
|---|---|---|---|---|
| `fs_searchpaths`/`fs_gamedir`/`fsh[MAX_FILE_HANDLES]`/counters/`initialized`/fs cvars | `files_common.cpp:183-224` (`fsh` :202) | `Common.fs: FileSystem` (**`MAX_FILE_HANDLES` = 64 PC MP / 16 SP**, STATE-D4) | `FS_InitFilesystem` | `&mut Engine` |
| `fs_serverPaks*`/`fs_serverReferencedPaks*`/`lastValidBase/Game`/`fs_checksumFeed` | `files_common.cpp:206-218` | `Common.fs: FileSystem` (**MP only — SP has no pure-server globals**) | `FS_PureServerSetLoadedPaks` | `&mut Engine` |
| `showpackets`/`showdrop`/`qport`/`net_killdroppedfragments` cvars | `net_chan.cpp:40-43` | `Cvar` handles (via `Common.cvars`) | `Netchan_Init` | handle |
| `loopbacks[2]` | `net_chan.cpp:486` | `Common.net: mp/engine/qcommon::NetLoopbacks` (`net_chan.cpp` is a qcommon TU) | `Netchan_Init` | `&mut Engine` |
| `cmg`/`SubBSP[32]`/`NumSubBSP`/trace counters/cm cvars | `cm_load.cpp:37,60-61` | `Engine.cm: mp/engine/qcommon::CollisionWorld` (`cm_load.cpp` is a qcommon TU; **instance-shaped value**, STATE-D2) | `CM_LoadMap` **populates**; zero/Default from `Engine::new` (mirrors static zero-init of `cmg`, `cm_load.cpp:37`) | `&mut Engine` |
| `gpvCachedMapDiskImage`/`gbUsingCachedMapDataRightNow` (CM↔renderer) | `cm_load.cpp:568,570` | `Engine.cm: mp/engine/qcommon::CollisionWorld` (renderer half → DEC-01) | `CM_LoadMap_Actual` | `&mut Engine` |
| `sv_worldSectors[64]`/`sv_numworldSectors` | `sv_world.cpp:58-59` | `Server.world_sectors` (**instance-shaped value**, STATE-D2) | `SV_ClearWorld` | `&mut Engine.sv`; the Chain-A disjoint field |

**qcommon — zone/hunk allocator (NOT PORTED, STATE-D4 / §C9)**

| Raven global | oracle cite | Rust owner | constructed by | threaded via |
|---|---|---|---|---|
| `TheZone`/`hunk_tag`/`gbMemFreeupOccured`/`com_validateZone` | `z_memman_pc.cpp:77,626,156,75` | **— (dropped)**: Rust ownership replaces `TheZone`; hunk clear-points (map-load lifetime) become explicit owned-arena **drops** where observable | n/a | n/a |

**Server (`mp/engine/server::Server`, `Engine.sv: Option<Server>`)**

| Raven global | oracle cite | Rust owner | constructed by | threaded via |
|---|---|---|---|---|
| `sv` (`server_t`) | `sv_main.cpp:11` | `Server.sv: server_t` — **reuse the existing mechanically-ported type** (`crates/mp/engine/server/src/server/server_t.rs:28`), not a new `ServerT`; embeds `svEntities`, `configstrings`, `models`; **holds the `SharedGameData` registration**, §Seam | `SV_SpawnServer` | `&mut Engine.sv` |
| `svs` (`serverStatic_t`, persists across maps) | `sv_main.cpp:10` | `Server.svs: serverStatic_t` — **reuse the existing ported type** (`crates/mp/engine/server/src/server/server_static_t.rs:20`); challenges, heap `clients[]`, snapshot ring | `SV_Init`/`SV_BootPersistant` | `&mut Engine.sv` |
| `debugpolygons`/`gWPArray[]`/`gWPNum` | `sv_bot.cpp:16-23` | `Server.bot` | `SV_BotInitBotLib` | `&mut Engine.sv` |
| SP `qbLoadTransition`/`eSavedGameJustLoaded` | `code/server/sv_ccmds.cpp:22` | `Server.savegame` (**SP only**) | `SV_Init` (SP) | `&mut Engine.sv` |

**Client (`mp/engine/client::Client`, `Engine.cl: Option<Client>`) — confined to `client/`**

| Raven global | oracle cite | Rust owner | constructed by | threaded via |
|---|---|---|---|---|
| `cl` (`clientActive_t`) | `cl_main.cpp:105` | `Client.cl: clientActive_t` — **reuse the existing mechanically-ported type** (`crates/mp/engine/client/src/client/client_active_t.rs:16`), not a new type; snapshots, baselines, parseEntities, cmds, gameState, `mSharedMemory` | `CL_ClearState` | `&mut Engine.cl` |
| `clc` (`clientConnection_t`) | `cl_main.cpp:106` | `Client.clc: clientConnection_t` — **reuse the ported type** (`client_connection_t.rs:46`); netchan, reliable rings, download/demo, RMG heightmaps | `CL_Connect_f` | `&mut Engine.cl` |
| `cls` (`clientStatic_t`) | `cl_main.cpp:107` | `Client.cls: clientStatic_t` — **reuse the ported type** (`client_static_t.rs:45`); connstate, keyCatchers, browser lists, glconfig copy | `CL_Init` | `&mut Engine.cl` |
| `kg` (`keyGlobals_t`) | `cl_keys.cpp:17` | `Client.keys: KeyState` (Raven already pre-bundled — the model shape) | `CL_Init` | `&mut Engine.cl` |
| `chatField`/`chat_team`/`chat_playerNum` | `cl_keys.cpp:12-15` | `Client.chat` | `CL_Init` | `&mut Engine.cl` |
| `con` (`console_t`) / `scr_*` / debug graph | `cl_console.cpp:13`, `cl_scrn.cpp:9,318,510` | `Client.console` / `Client.screen` | `CL_Init`/`SCR_Init` | `&mut Engine.cl` |

**Sound (`mp/engine/client::SoundSystem`, `Engine.snd` — MP client only, DEC-03)**

| Raven global | oracle cite | Rust owner | constructed by | threaded via |
|---|---|---|---|---|
| `s_channels[]`/`dma`/`listener_*`/`s_knownSfx[]`/loop+raw buffers/OpenAL block/dynamic music | `snd_dma.cpp:127-268` | `Engine.snd: Option<SoundSystem>` (faithful mixer; EAX/force-feedback dropped, DEC-03; **`None` on dedicated** — `S_Init` gated `!com_dedicated`, `common.cpp:1394,1680`, same `Option` shape as `cl`) | `S_Init` (client-init path only) | `&mut Engine.snd` |

**VM layer / module handles → engine-seam.md (SEAM), not owned here**

| Raven global | oracle cite | Rust owner | constructed by | threaded via |
|---|---|---|---|---|
| `vmTable[3]`/`gvm`/`cgvm`/`uivm`/`vm_debugLevel` | `vm.cpp:28`, `sv_main.cpp:12`, `cl_main.cpp:108`, `cl_ui.cpp:28` | engine module-host state per loaded module (`ModuleTransport`), engine-seam § State ownership | module loader | dispatcher `engine` arg |
| `currentVM`/`lastVM` | `vm.cpp:24-25` | **eliminated** — each dispatch explicitly parameterized (engine-seam SEAM-D1; Raven's `VM_Free` global-clobber / skipped-restore bug **not** reproduced, STATE-D3) | — | — |
| per-slot `*mut Engine` trampoline stash cell (supplants `currentVM`'s bridging role) | (new; supplants `vm.cpp:800`) | `mp_engine_qcommon::ModuleRegistry`'s per-slot `EngineSlot.engine` cell — **owned by engine-seam.md (SEAM-D11), not here**; one cell per module slot (never one global, STATE-D2) | `EngineSlotGuard::enter` at each engine→module call | read only by that slot's raw `extern "C-unwind"` syscall trampoline (engine-seam § Inbound raw syscall trampoline, SEAM-D11) |

**Renderer (DEC-01 — port deferred; listed for completeness only)**

| Raven global | oracle cite | Rust owner | constructed by | threaded via |
|---|---|---|---|---|
| `tr`/`backEnd`/`glConfig` (+292 renderer file-statics) | `tr_main.cpp:15`, `tr_backend.cpp:21`, `tr_init.cpp:33` | **deferred, DEC-01** — null-`refexport_t` stub behind the seam until wgpu port | n/a | n/a |

**botlib / ghoul2 / icarus / rmg (§F C++-track engine subcrates — NOT censused by the A2 survey)**

These four engine subcrates exist in the crate graph (`mp/engine/{botlib,ghoul2,icarus,rmg}`) and have real Raven global state, but the A2 dossier's engine-tier census (dossier §1) enumerated only qcommon/server/client/sound/renderer — these appear there solely as cross-boundary *readers* of `sv`/`svs` (e.g. `icarus/GameInterface.cpp:129`, `RMG/RM_Instance_BSP.cpp:262`), never as surveyed owners. Per porting-rules §F they are idiomatic C++-track reimplementations designed in their own subsystem docs (GP2 pilot precedent); their internal state ownership belongs there, not here. What is *not* settled is whether/how their engine-side state attaches to the `Engine` island (ghoul2 in particular is shared engine↔cgame) — see STATE-Q2. The one already-surfaced fragment (botlib's `debugpolygons`/`gWPArray`) is placed under `Server.bot` in the Server block above.

| Raven global | oracle cite | Rust owner | constructed by | threaded via |
|---|---|---|---|---|
| botlib / ghoul2 / icarus / rmg engine-side globals | dossier §1 (not censused; readers only, e.g. `icarus/GameInterface.cpp:129`) | **pending §F subsystem docs** (`docs/subsystems/{botlib,ghoul2,icarus,rmg}.md`); `Engine`-field attachment is STATE-Q2 | their `*_Init` (per §F doc) | n/a here |

**Module island — MP game (`mp/game::GameWorld`)**

| Raven global | oracle cite | Rust owner | constructed by | threaded via |
|---|---|---|---|---|
| `level` (`level_locals_t`) | `g_main.c:9` | `GameWorld.level: mp_game::level::level_locals_t` | `G_InitGame` | `(world, id)` re-borrow |
| `g_entities[MAX_GENTITIES]` | `g_main.c:27` | `GameWorld.entities: Box<[mp_qshared::common::mp::gentity_t; MAX_GENTITIES]>` (contiguous `#[repr(C)]`, size-asserted 1832B — reuse the existing type, §D12) | `G_InitGame` | `EntityId(u32)` index |
| `MAX_GENTITIES` (sizes the array above) | `q_shared.h:1996` | **`mp_qshared`** const per its oracle home + workspace-architecture tier table (Tier 0 qshared) — currently mis-placed in `mp_engine_server` (`crates/mp/engine/server/src/server/server_t.rs:21`) by the mechanical type-port; **relocated to `mp_qshared`** as a slice-0 wiring task (`mp_engine_server` re-imports it), no behavioral change | (compile-time const) | referenced, not threaded |
| `g_clients[MAX_CLIENTS]` (reached as `level.clients`) | `g_main.c:28` | `GameWorld.clients: Box<[mp_game::client::gclient_t; MAX_CLIENTS]>` (`gclient_t` = `gclient_s`, asserted 7344B; **MP only**) | `G_InitGame` | index |
| `trap_LocateGameData` registration | `g_main.c:997` | registers `GameWorld` base+stride into the engine's `SharedGameData` (§Seam) | `G_InitGame` | raw seam (§D11) |

**Module island — SP game (`sp/game::GameWorld`)**

| Raven global | oracle cite | Rust owner | constructed by | threaded via |
|---|---|---|---|---|
| `level`/`g_entities` | `code/game/g_main.cpp:46,49` | `GameWorld.level` / `GameWorld.entities` | `InitGame` | `EntityId` index |
| `g_clients` | — | **does not exist** (`MAX_CLIENTS = 1`) — divergence preserved (STATE-D2) | — | — |
| `globals` (`game_export_t`) | `code/game/g_main.cpp:48` | SP seam handle (engine-seam SEAM-D2); `globals.gentities` = base register | `GetGameAPI`/`InitGame` | raw seam (§D11) |

**Module island — cgame / ui (both modes)**

| Raven global | oracle cite | Rust owner | constructed by | threaded via |
|---|---|---|---|---|
| `cg`/`cgs`/`cg_entities[]` | `cg_main.c:691-693`; SP `code/cgame/cg_main.cpp:210` | `CgState` (per mode) — **SP owned struct threaded in, not a process global** (STATE-D4) | `CG_Init` | `&mut CgState` + `EntityId` |
| `uiInfo` | `ui_main.c:875`; SP `code/ui/ui_main.cpp:315` | `UiState` (per mode) — SP owned struct threaded in (STATE-D4) | `UI_Init` | `&mut UiState` |
| `DC` (`displayContextDef_t*`) | `codemp/ui/ui_shared.c:103`; SP `code/ui/ui_shared.cpp:136` | shared-widget context threaded into `ui_shared` calls | `UI_Init` | `&DisplayContext` |

## Seam definition

The types below are FROZEN — porters fill fields/bodies without changing the
shapes or signatures. Field lists internal to each owned struct are §subsystem
detail (non-goal); the *island boundaries*, the entity handle, `SharedGameData`,
and the error payload are what freeze here.

### `Engine` — the one owned engine-island instance (per mode)

```rust
// DEFINED in the per-mode facade crate `mp_engine_core` (package mp_engine_core;
// SP mirror crate `sp_engine_core`) — the one crate that depends on all engine
// subcrates, so it can name Server/Client/etc. as fields (qcommon cannot reach
// server+client). One value, INSTANTIATED by `Engine::new` called from the thin
// `mp/app` bin shell (STATE-D5; LIFE-D2 amended in lifecycle.md), threaded `&mut`
// DOWN call chains (STATE-D1). No field is ever a `static`/global. `mp_engine_core`
// also hosts com_init/com_frame/com_shutdown/com_error (they must reach the
// server+client crates). Field *sub*-structs keep their own subcrate homes below.
pub struct Engine {
    pub common: Common,          // type from mp_engine_qcommon (cvars, cmd, cbuf, fs, net)
    pub sv: Option<Server>,      // type from mp_engine_server — Some when a server is running
    pub cl: Option<Client>,      // type from mp_engine_client — Some on client builds; None on dedicated
    pub cm: CollisionWorld,      // type from mp_engine_qcommon — cmg + SubBSP, instance-shaped value.
                                 //   NOT an Option: `Engine::new` builds it zero/Default-initialized
                                 //   (mirroring Raven's static zero-init of file-scope `clipMap_t cmg`,
                                 //   `cm_load.cpp:37`; §A2 faithful), present-but-empty before any map;
                                 //   CM_LoadMap then POPULATES it in place. The master table's
                                 //   "constructed by CM_LoadMap" means populate-in-place, not first
                                 //   existence — a zeroed CollisionWorld exists from `Engine::new`.
    pub snd: Option<SoundSystem>,// type from mp_engine_client — client only; None on dedicated,
                                 //   the same `Option`-for-client-presence shape as `cl`.
                                 //   S_Init is reached only through the client-init path, gated
                                 //   `!com_dedicated` (`common.cpp:1394,1680`; S_Init
                                 //   `cl_main.cpp:1380,2461`), so a dedicated server never
                                 //   constructs it — Slice 0 (dedicated) builds `snd: None`
                                 //   (grounded in the cited gating + DEC-03; same Sound
                                 //   master-table row).
    // botlib/ghoul2/icarus/rmg engine-side state is NOT yet a field here — those
    // four §F subcrates were outside the A2 survey; attachment point is STATE-Q2.
}
```

The game dispatcher's `engine: &mut ServerGame` parameter (engine-seam,
`sv_game_system_calls`) is the **server-island reborrow** — `&mut Engine.sv`'s
`Server`, carrying its `SharedGameData` registration. `ServerGame` is
engine-seam's name for exactly that reborrowed host state; the two names denote
the same value (kept consistent per that doc).

### `com_init` / `com_frame` / `com_shutdown` / `com_error` — the `mp_engine_core` entry points (FROZEN HERE)

The engine island's headline entry points are **defined in `mp_engine_core`** —
the aggregate facade, the only engine crate that can reach both `Server` and
`Client` (workspace-architecture § Dependency edges; STATE-D5). A leaf subcrate
**cannot** host them: `mp_engine_server`'s deps are `mp_qshared`/
`mp_engine_qcommon`/`mp_abi` only (no facade edge). `&mut Engine` threads **down**
from here (STATE-D1); an orchestrator takes the whole `Engine` and hands each
subsystem its narrower reborrow (`&mut Engine.sv` / `&mut Engine.cl`, master
table) as it calls down.

```rust
// mp/engine/core (SP: sp/engine/core). Free functions on the facade.

/// `void Com_Frame( void )` (`common.cpp:1593`). One host frame. Takes **no**
/// frame-delta argument — `msec` is computed internally (`Com_ModifyMsec`,
/// `common.cpp:1660`) before dispatching `sv_frame`/`cl_frame`
/// (`common.cpp:1669,1711`). Hosts the DEC-08 `catch_unwind` that turns a
/// `com_error` panic into print-and-continue (throw caught at `common.cpp:1762`).
pub fn com_frame(engine: &mut Engine);

/// `void Com_Shutdown( void )` (`common.cpp:1785`). Teardown of the owned
/// subsystems is their `Drop` (§C9), not an explicit free.
pub fn com_shutdown(engine: &mut Engine);
```

**`sv_frame`/`cl_frame` live in `mp_engine_core` too, not the leaf subcrates.**
The two-island diagram's `sv_frame(&mut Engine)` takes the *whole* `Engine`
because it both advances `Engine.sv` **and** issues `VM_Call(GAME_RUN_FRAME)`
through the module-host dispatch (engine-seam) and, on non-dedicated builds, calls
`cl_frame` — reaching server + client + seam, exactly the facade's remit. It is
**not** definable in `mp_engine_server` (that needs `server → core`, the reverse
of the real edge). `sv_frame` is the orchestrator; the narrower `&mut Engine.sv`
the master table documents is what it hands to `Server`-internal functions as it
calls down. (Mirrors oracle's `SV_Frame( int msec )`, `sv_main.cpp:826`, itself
called only from `Com_Frame`.)

**`com_init` and `Engine::new`.** `Engine` is instantiated by `Engine::new`
(returns the one owned `Engine` value, STATE-D5); `com_init` runs Raven
`Com_Init`'s boot body (the subsystem `*_Init` sequence, master table
"constructed by"). Both live in `mp_engine_core`. Raven `Com_Init( char
*commandLine )` (`common.cpp:1216`) parses that raw command line
(`Com_ParseCommandLine`, `common.cpp:1230,397`) into `Common.cmdline`, so the raw
command line enters this init pair. **Not fixed here — deferred to lifecycle.md
(§ Non-goals, construction order):** the division of labor between `Engine::new`
and `com_init`, and hence which of the two carries the command-line parameter.
Slice 0 already depends on lifecycle.md for construction order (§ Slice hooks), so
this rides along; STATE freezes only that `Engine::new -> Engine` yields the owned
island and that all four `com_*` live in the facade.

**`com_error`'s signature is partially frozen.** Its payload is fixed —
`level: ErrorLevel` + a formatted `msg: String` (the `ComError` struct below;
Raven `void QDECL Com_Error( int code, const char *fmt, ... )`, `common.cpp:249`,
whose varargs are formatted caller-side into `msg`) — and it **diverges**, `-> !`
via `panic_any(ComError)` after recovery (STATE-D3). Its **receiver is *not*
frozen**: whether `com_error` takes `&mut Engine` — or reaches the cross-subsystem
recovery some other way — is the unresolved **STATE-Q4** reachability question
(§ Open questions), because the concrete deep throw sites are leaf-crate code that
cannot reach the facade. Written against the settled parts only:

```rust
// mp/engine/core. Receiver PENDING STATE-Q4 (see Open questions).
pub fn com_error(/* receiver: STATE-Q4 */ level: ErrorLevel, msg: String) -> !;
```

### `GameWorld` — the one owned module-island instance (per mode)

```rust
// mp/game (SP: sp/game). A value type owned by the module crate. NOT a global.
// Field types are the EXISTING Raven-faithful, already-offset-asserted structs
// (§D12 keeps Raven names on ABI-crossing types — these are exactly the structs
// the raw LocateGameData seam aliases into). Do NOT mint new PascalCase types.
pub struct GameWorld {
    pub level: level_locals_t,                        // mp_game::level::level_locals_t
                                                      //   (game/src/level/level_locals.rs:29)
    pub entities: Box<[gentity_t; MAX_GENTITIES]>,    // mp_qshared::common::mp::gentity_t
                                                      //   (qshared/.../gentity.rs:50; size-asserted 1832B :456).
                                                      //   MAX_GENTITIES is referenced from mp_qshared (below).
    pub clients: Box<[gclient_t; MAX_CLIENTS]>,       // mp_game::client::gclient_t (= gclient_s;
                                                      //   client/gclient.rs:231/:26; asserted 7344B). MP only;
                                                      //   SP omits this field (MAX_CLIENTS = 1, no g_clients).
}
```

- **Aliasing preserved exactly** (§A1/§A2): entity/client storage is the real
  memory the engine reads through `SharedGameData`; no copy-narrowing. The
  copy-based `CL_GetSnapshot` stays a copy only because the oracle copies there.
- **`#[repr(C)]` contiguous entity array** (not `Vec<Enum>`): the same byte
  layout serves both the native pointer-cast reader and the future WASM
  offset-translation reader (DEC-05.5) — one shape, two `unsafe` readers.
- **Multi-world constraint** (STATE-D2, user requirement): `GameWorld` is a
  **value type** and the engine holds *a* (registration-keyed) `SharedGameData`,
  not a singleton. Multiple concurrent worlds with cross-world messaging must be
  structurally retrofittable — **no ambient statics, no singleton assumptions**.
  Single-world is the built configuration; nothing here forecloses many.
- **`MAX_GENTITIES` is an `mp_qshared` const, not `mp_game`'s.** It is a
  `q_shared.h` define (`GENTITYNUM_BITS`=10 → 1024, `q_shared.h:1992-1996`), so
  per the workspace-architecture tier table (Tier 0 qshared = `q_shared.{h,c}`,
  used by engine+game+cgame+ui) its home is `mp_qshared` — the same tier as
  `gentity_t`, which it sizes, and a crate `mp_game` already depends on. The
  earlier mechanical type-port placed the const in `mp_engine_server`
  (`crates/mp/engine/server/src/server/server_t.rs:21`), a crate `mp_game`
  cannot reach; the frozen struct references `mp_qshared::…::MAX_GENTITIES`, and
  relocating the const to its tier-correct home is a mechanical porting task for
  this slice (no behavioral change — the value is identical).
- **`level`'s self-referencing pointers — intra-`GameWorld` construction order.**
  (Grounded in the oracle sequence cited below, `g_main.c:978-984`; not a separate
  decision record.) `level_locals_t` carries the raw back-pointers Raven wires with
  `level.gentities = g_entities` / `level.clients = g_clients` (`g_main.c:979,984`,
  each immediately after its array is `memset`, `g_main.c:978,983`). The one
  data-dependency this doc fixes: **heap-allocate the `entities`/`clients` boxes
  first, then set `level.gentities`/`level.clients` to alias them** — the boxes
  must exist before the pointers can name them (mirroring `g_main.c:978-984`). The
  *broader* boot sequence (when `G_InitGame` runs relative to `SV_SpawnServer`,
  the `ErrorLevel` teardown order) stays lifecycle.md's (§ Non-goals). De-risking
  note: `level.gentities` is write-once/registration-only — never indexed by game
  logic (0 hits for `level.gentities[` across `codemp/`), so it is inert once
  wired (`level.clients`, by contrast, *is* the client access path).

### `WorldCell` — where the one `GameWorld` lives across `vmMain` calls (STATE-D6, FROZEN HERE)

The C module entrypoint `vmMain(command, args…)` takes **no context argument**,
yet the single owned `GameWorld` must persist **and** be mutated across successive
opaque calls (`GAME_INIT` builds it, `g_main.c:515,979`; `GAME_RUN_FRAME` mutates
it, `g_main.c:3582`). It is reached through a `WorldCell` static in the module
cdylib shell — the second sanctioned static exemption (STATE-D6), beside
engine-seam's outbound `ENGINE: OnceLock<CEngine>` (SEAM-D1). Frozen shape:

```rust
// Module cdylib shell crate (crates/jampgame / crates/cgame / crates/ui; SP jagame).
// Holds the module island's one owned GameWorld — a per-mode logical singleton,
// exactly Raven's `level` (§B6). Identical shape in NativeDll, Static, and Wasm builds.
// No logic crate names it; only the shell's vmMain reads/writes it (access confined below).
static WORLD: WorldCell = WorldCell::new();

struct WorldCell(UnsafeCell<Option<GameWorld>>);
// SAFETY: the module runs single-threaded per Raven's contract; the ONLY reader/writer
// is the vmMain dispatch below, so no cross-thread aliasing is possible.
unsafe impl Sync for WorldCell {}
```

Access discipline — the only `unsafe` in the shell, confined per §D11:

```rust
// crates/jampgame shell. C entrypoint takes NO context arg, so the world is reached via
// WORLD. GAME_INIT constructs it into the cell; every later command re-derives `&mut GameWorld`
// from the cell and threads it inward — reborrow-threading (STATE-D1) from the seam down.
#[no_mangle]
pub extern "C-unwind" fn vmMain(command: c_int, /* args… */) -> c_int {
    let world: &mut GameWorld = /* unsafe: &mut *WORLD.0.get(), the Option built at GAME_INIT */;
    dispatch(command, world, /* args… */)
}
```

- **Reentrancy is by design, not a hazard** (STATE-D6). The engine re-enters
  `vmMain` from inside a syscall handler — `G_DROP_CLIENT` (`sv_game.cpp:570`) →
  `SV_GameDropClient` (`sv_game.cpp:110-114`) → `SV_DropClient`
  (`sv_client.cpp:580`) → `VM_Call(gvm, GAME_CLIENT_DISCONNECT)`
  (`sv_client.cpp:640`), a fresh `vmMain` entry while the outer one is suspended
  — so the re-derived `&mut GameWorld` aliases the world exactly as Raven's plain
  `level`/`g_entities` globals did under the same chains. This is why the cell is a
  raw `UnsafeCell` and **not** a `Mutex` (deadlocks on those chains) or `RefCell`
  (panics on them): the safety argument is Raven's single-threaded sequencing, made
  explicit at this one seam.
- **`extern "C-unwind"`** (engine-seam SEAM-D10) so a `com_error` panic unwinds
  through the live C frames of a re-entrant chain (STATE-D3).
- The `EntityId` re-borrow discipline (below) applies to every access *inside*
  `dispatch`, so no raw `gentity_t*` alias survives above the seam.

### `EntityId` — the entity handle (§B5)

```rust
// mp/game (SP mirror). Raven's `gentity_t*` become an index into GameWorld.entities.
// Module logic passes (world, id) and re-indexes per access — GP2's GpGroupId precedent.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EntityId(pub u32);
```

### `SharedGameData` — register-once / read-later (FROZEN HERE)

engine-seam defers this method set to this doc. Engine-internal; the
raw-pointer-returning accessors sit at the ABI seam and are the §D11 exemption —
engine subsystems wrap them into safe index/handle access immediately, so no raw
pointer aliases survive in the safe tier above the seam.

```rust
// Engine-internal (mp/engine/server). Two register-once/read-later families:
//   (1) the LocateGameData entity+client arrays, (2) the SET_SHARED_BUFFER command buffer.
// Method set mirrors SV_GentityNum / SV_GameClientNum / SV_NumForGentity
// (sv_game.cpp:46-65) and the sv.mSharedMemory registration (sv_game.cpp:940).
// Stores the payloads of abi-traps.md row 18 (trap_LocateGameData) and
// row 121 (trap_SV_RegisterSharedMemory); the trap *crossing* is engine-seam's.
pub trait SharedGameData {
    // --- family 1: entity/client arrays (G_LOCATE_GAME_DATA) ---
    /// Register base+stride for the game's entity and client arrays. Called once
    /// at GAME_INIT (re-issued on entity-count change). `clients_base` is the
    /// leading-playerState_t pointer at gclient_t stride (SP passes none).
    fn locate_game_data(
        &mut self,
        ents_base: SeamPtr, num_entities: usize, gentity_size: usize,
        clients_base: Option<SeamPtr>, gclient_size: usize,
    );
    fn gentity(&self, num: usize) -> *mut u8;            // ents_base + gentity_size*num
    fn game_client(&self, num: usize) -> *mut u8;        // clients_base + gclient_size*num
    fn num_for_gentity(&self, ent: *const u8) -> usize;  // inverse mapping

    // --- family 2: shared command buffer (G_SET_SHARED_BUFFER) ---
    /// Register the module's high-arity scratch buffer (gSharedBuffer). Store, return.
    fn register_shared_memory(&mut self, buf: SeamPtr);
    fn shared_memory(&self) -> *mut u8;
}
```

`SeamPtr` is the transport's raw address word (a host pointer for
`NativeDll`/`Static`, a linear-memory offset for `Wasm`) — that semantic
contract is what this doc fixes. As "the transport's" address word its **type is
defined in `abi-transport`** (the transport crate every `mp/engine/*` crate
depends on — workspace-architecture § Dependency edges), so this engine-internal
trait names it unqualified without a new edge. Its *concrete* Rust representation
(a `usize` alias vs. a newtype, and any WASM `Memory`-handle pairing) is a
seam-executor mechanic and a non-goal here (§ Non-goals →
`docs/architecture/engine-seam.md`); the trait above is written against the
semantic contract only. Two impls, same
signature, different contract (engine-seam SEAM-D4):

- **`NativeDll` / `Static`**: cache the base pointer + stride once at
  registration; accessors do raw arithmetic — faithful to `sv.gentities`/
  `sv.mSharedMemory`, zero cost.
- **`Wasm`**: store the module `Memory` handle + offset/stride and **re-resolve
  `(base+offset, len)` per access with bounds checks**, never caching a base
  pointer, so it survives `memory.grow`.

The registration is held in `Server.sv` (per-world value), **keyed** — not a
process singleton — so the multi-world constraint (STATE-D2) holds structurally.

### `ComError` — the `com_error` panic payload

```rust
// mp/engine/qcommon. Raven's Com_Error(level, msg) → recover synchronously, then
// panic_any(ComError). Caught by catch_unwind at Com_Frame (DEC-08).
pub struct ComError {
    pub level: ErrorLevel,   // frozen in lifecycle.md (LIFE-D3); variants trace to
    pub msg: String,         //   ERR_FATAL/DROP/DISCONNECT/SERVERDISCONNECT/NEED_CD
}                            //   (common.cpp:302-344)
```

`ErrorLevel` is per-mode (MP `errorParm_t` vs SP's set) and is **frozen in
`docs/architecture/lifecycle.md` (LIFE-D3, pending)**, not here; this doc only
fixes the payload shape and the recovery-before-panic ordering (STATE-D3). The
`com_error` *function* signature (this payload as the panic value, `-> !`, and its
still-open receiver) is in § `com_init`/`com_frame`/`com_shutdown`/`com_error`
above — the receiver awaits STATE-Q4.

## Decisions

**STATE-D1 — The two-island model.** One owned per-mode `Engine` struct (fields
= owned subsystem structs: `Common{cvars,cmd,cbuf,fs,net}`, `Option<Server>`,
`Option<Client>`, `snd`, …), created in `main()`, threaded `&mut` **down** call
chains via reborrows; callers never hold a borrow **across** a nested call
(matching oracle's `SV_Frame` shape, `sv_main.cpp:909-915`). Module state is a
separate owned `GameWorld` island. Engine↔module entity aliasing goes exclusively
through the raw `LocateGameData`-family seam (`unsafe`, §D11) in **both** the
`NativeDll` and `Static` transports, so the borrow checker never sees it. Module
logic uses `EntityId(u32)` re-borrow discipline (pass `(world, id)`, re-index per
access — GP2 precedent). **No `RefCell`, no effect queues** (queues would break
same-frame link-then-query parity, e.g. `G_TouchTriggers` observing entities
linked earlier in the frame, Chain A). Seam dispatchers/entrypoints are
`extern "C-unwind"` so `com_error` panics traverse live C frames when hosting real
DLLs (backtraces resolve as module+offset). *Because* Chain A (`sv_world.cpp:189`
splicing world-sectors while `SV_AreaEntities_r` walks them, reachable from the
same nested `G_RunFrame` graph) cannot compile as safe Rust inside one
`&mut`-borrowed mega-struct. *Rejected:* one flat mega-struct (borrow checker
cannot prove disjoint fields across the indirect `VM_Call`); `RefCell`/effect
queues (defer aliasing to runtime panics / break same-frame ordering parity).

**STATE-D2 — GameWorld shape.** Per-mode `GameWorld` owned by the module crate:
`level_locals` + contiguous `#[repr(C)]` boxed `MAX_GENTITIES` array of the
already-asserted `gentity_t` + MP `clients` array (**SP has no `g_clients` —
divergence preserved**, `MAX_CLIENTS = 1`). Aliasing is preserved exactly (§A2 —
no copy-narrowing; copy-based `CL_GetSnapshot` stays a copy because the oracle
copies there). **Multi-world constraint (user requirement):** `GameWorld` is a
value type and the engine's server state holds *a* registration-keyed
`SharedGameData` — multiple concurrent worlds with cross-world messaging must be
structurally retrofittable (no singleton assumptions, no ambient statics);
single-world remains the built configuration. `clipMap`/`CollisionWorld` and
`svEntity`/`worldSector` state are likewise instance-shaped values. *Rejected:* a
`Vec<Enum>` entity store (would fork the native pointer-cast and WASM offset paths
into two layouts); a process-global `GameWorld` (forecloses multi-world, §B3).

**STATE-D3 — Error recovery ordering.** `com_error(level, msg)` runs oracle's
per-level recovery **synchronously BEFORE panicking** (mirroring `SV_Shutdown` →
`GAME_SHUTDOWN` → free happening pre-throw, `common.cpp:313-326`), then
`panic_any(ComError)`; `catch_unwind` at `com_frame` prints and continues
(DEC-08). Guards held across seam calls must be `Drop`-safe (**no `mem::forget`,
no manual save/restore without a guard type**); Raven's skipped `currentVM`
restore on unwind (`vm.cpp:826-827`) is a bug we do **not** reproduce — there is
no current-module global at all (engine-seam SEAM-D1 eliminated `currentVM`).
*Because* Chain B shows the catch handler never sees inconsistent state — the
subsystem is already torn down when the throw starts; unwinding runs destructors
and nothing else, so any across-call guard must clean up in `Drop`. *Rejected:*
recovering *after* `catch_unwind` (would let the handler observe half-torn state);
`Result`-threading (DEC-08 — reshapes thousands of faithful signatures). *Open:*
the crate-graph mechanism by which a deep **leaf-crate** throw site (e.g.
`SV_SvEntityForGentity`, `sv_game.cpp:70-75`, in `mp_engine_server`) reaches this
facade-level recovery — `com_error` lives in `mp_engine_core`, which the leaf
crate cannot depend on — is unresolved (**STATE-Q4**); it also fixes `com_error`'s
receiver.

**STATE-D4 — Globals collapse policy.** Cleanly-bounded subsystems become owned
structs within the Engine island (`CvarSystem`, `CmdSystem`+`Cbuf` incl. owned
tokenizer scratch, `FileSystem`, `NetLoopbacks`, `CollisionWorld`, `KeyState`
(`kg`), `SoundSystem`, …). The zone/hunk allocator is **NOT ported** (§C9): Rust
ownership replaces `TheZone`; hunk clear-points (map-load lifetime) become
explicit owned-arena **drops** where behavior is observable. `cmd_functions`
divergence preserved per-mode (SP fixed `[_;256]` with its overflow behavior, MP
heap linked list). SP cgame/ui state (`cg`/`cgs`/`uiInfo`) does **not** become
process globals — owned structs threaded in, fixing SP's static-link regression
(MP's isolation is the behavioral model). `MAX_FILE_HANDLES` = **64 on PC MP**
(16 was Xbox-only; SP = 16), and `FS_ReadFile`'s live path uses `Z_Malloc` (hunk
calls are commented out) — both flow into the state table. *Note:* the previously
flagged `files_pc.cpp:2328` "static shadowing bug" is **dead comment text, not
real code** — no shadowing behavior exists to model; the clause is dropped.
*Rejected:* porting `TheZone` faithfully (§C9 — Rust ownership is the
replacement); unifying MP/SP `cmd_functions` (DEC-04 — duplicate, don't unify).

**STATE-D5 — `Engine`'s defining crate + instantiation (resolves STATE-Q1;
session 2026-07-02).** `pub struct Engine` is **defined** in a new per-mode facade
crate `crates/mp/engine/core` (package `mp_engine_core`; SP mirror
`crates/sp/engine/core`, `sp_engine_core`) — the one crate depending on every
engine subcrate, so it can name `Common`/`Server`/`Client`/… as fields
(`mp_engine_qcommon` cannot reach the server+client crates). That facade crate
also hosts `com_init`/`com_frame`/`com_shutdown`/`com_error` (they must reach
server+client). `Engine` is **instantiated** by `Engine::new` from the thin
`mp/app` (`sp/app`) bin shell; `&mut Engine` threads **down**, never a `static`.
LIFE-D2 is amended accordingly in lifecycle.md; field sub-structs keep their own
subcrate homes (§Seam Engine block). *Because* `mp/engine/` is seven independent
subcrates with no aggregator, and a dedicated facade keeps `mp/app` a thin shell.
*Rejected:* defining `Engine` in `mp/app` (the bin shell must stay thin, and
`com_*` belong in a lib crate the modules can link, not the app binary).

**STATE-D6 — `GameWorld` storage across `vmMain` calls (resolves STATE-Q3;
session 2026-07-02).** The single owned `GameWorld` lives in a `WorldCell`
**static** in the module cdylib shell crate (`crates/jampgame`/`crates/cgame`/
`crates/ui`, SP `jagame`), beside engine-seam's outbound `ENGINE` cell — the
**second** sanctioned static exemption to §D11/§B3, widening engine-seam SEAM-D1's
"one static" to two module-shell cells. It is a per-mode logical singleton exactly
like Raven's `level` (§B6). Unsafe access is confined to the shell's `vmMain`
dispatch, which re-derives `&mut GameWorld` from the cell once per entry and
threads it inward; same cell shape in all three transports (module-island state).
**Reentrancy contract:** the engine genuinely re-enters `vmMain` from inside a
syscall handler — `G_DROP_CLIENT` (`sv_game.cpp:570`) → `SV_GameDropClient`
(`sv_game.cpp:110-114`) → `SV_DropClient` (`sv_client.cpp:580`) →
`VM_Call(gvm, GAME_CLIENT_DISCONNECT)` (`sv_client.cpp:640`), a fresh `vmMain`
entry while the outer one is live — so the re-derived `&mut GameWorld` aliases the
world exactly as Raven's plain `level`/`g_entities` globals did under those same
chains. *Because* the C `vmMain(command, args…)` entrypoint takes no context
argument, so a mutable-across-calls owned value must be reachable without one, and
those live re-entry chains alias it by design. *Rejected:* `Mutex` (deadlocks on
those chains); `RefCell` (panics on them); a context-threaded entrypoint (the ABI
is fixed — no context slot in `vmMain`).

## Verification strategy

Per DEC-09, this doc owns **no wire behavior of its own** — it fixes ownership
and threading, whose correctness is a compile-time + review property, and defers
behavioral parity to the per-subsystem docs' harnesses:

1. **No `static mut` / no ambient globals** — enforced by review **and a standing
   CI grep gate**: `grep -rn "static mut" crates/{mp,sp}/{game,cgame,ui,engine}`
   must stay empty (it is today, dossier §5); a companion gate rejects new
   non-`const` `static` in those trees outside the one engine-seam `OnceLock`
   exception (SEAM-D1). This is the mechanical proof of §B3.
2. **Drop-safety audit** — every guard type held across a seam call
   (`extern "C-unwind"` boundary) carries a review checklist item: it must clean
   up in `Drop`, no `mem::forget`, no manual save/restore (STATE-D3). Auditing
   the unwind path is DEC-08's per-subsystem obligation, tracked per owned struct.
3. **Behavioral parity via the owning subsystem docs** — the `SharedGameData`
   native-impl stride arithmetic is checked against `SV_GentityNum`/
   `SV_NumForGentity` goldens in the seam/server subsystem harness (DEC-09.1,
   `tools/gp2-oracle` pattern); the entity-layout asserts are the existing
   `#[repr(C)]` `size_of`/`offset_of!` static-asserts (§D12), green on build.
   This doc introduces no new golden harness.

Native-track (§E): green at every commit, one struct/file per commit,
slice-driven.

## Slice hooks

- **Slice 0 (MP dedicated boot).** `Engine::new` (in `mp_engine_core`, called
  from the thin `mp/app` bin shell, STATE-D5) builds `Engine` with all five
  fields: `common: Common`, `cm: CollisionWorld` (zero/Default — no map loaded
  yet in the dedicated-boot skeleton; the first `CM_LoadMap` populates it later,
  mirroring `cmg`'s static zero-init, `cm_load.cpp:37`), `sv: Some(Server)`,
  `cl: None`, `snd: None` (dedicated — no client, no sound); `&mut Engine` threads
  down. `jampgame`'s `GAME_INIT` builds
  the `GameWorld` **into the `WORLD` cell** (STATE-D6) and issues **both**
  registrations — `trap_LocateGameData` (`g_main.c:997`) and
  `trap_SV_RegisterSharedMemory` (`g_main.c:920`) — dispatched into the `NativeDll`
  `SharedGameData` impl held in `Server.sv` (native contract: cache base+stride /
  store buffer, return `0`). The subsequent `GAME_RUN_FRAME` re-derives
  `&mut GameWorld` from that same cell (STATE-D6 access discipline).
  Needs frozen here (and are): the `Engine`/`Server`/`GameWorld`/`WorldCell`
  shapes, `EntityId`, the `SharedGameData` trait, the `ComError` payload, and the
  `com_frame`/`com_shutdown` entry-point signatures (§ Seam). Depends on
  engine-seam.md for the dispatchers exercising these registrations (SEAM Slice 0)
  and on lifecycle.md for construction *order*, `ErrorLevel` variants, **and the
  `Engine::new`/`com_init` command-line-parameter split** (§ Seam). `com_error`'s
  receiver is not needed for the dedicated first-frame skeleton, which raises no
  error — it is STATE-Q4.
- **Later slices.** MP client adds `cl: Some(Client)` (+ `KeyState`, console) and
  `snd`; SP adds its `GameWorld` (no `clients`) and the `GetGameAPI` table
  register (SEAM-D2); the WASM `SharedGameData` re-resolving impl lands after
  native parity (DEC-05.5). Multi-world retrofit (STATE-D2) is a post-parity
  slice — the value-typed, registration-keyed design makes it non-structural.

## Open questions

The survey's own forks (dossier § Design forks 1–7) are all settled by
STATE-D1..D6 or the cited ledger entries (DEC-04/05/07/08). The two former
structural gaps — the defining crate for `Engine` (was STATE-Q1) and where the
owned `GameWorld` lives across `vmMain` calls (was STATE-Q3) — are resolved by
STATE-D5 and STATE-D6 (escalation session 2026-07-02). The two cross-doc
dependencies — the seam dispatchers (engine-seam.md) and the `ErrorLevel` enum
plus construction order (lifecycle.md, LIFE-D3) — are scoped non-goals owned
elsewhere, not unresolved decisions. Two structural questions remain genuinely
undecided and go back to a design session:

- **STATE-Q4 — `com_error`'s recovery reachability across the crate graph (and
  hence its receiver).** *Owner: escalate to a design session (touches STATE-D3,
  STATE-D5, DEC-08).* STATE-D3 requires `com_error` to run the full cross-subsystem
  recovery (`SV_Shutdown` → `GAME_SHUTDOWN` dispatch + `VM_Free`,
  `CL_Disconnect`/`CL_FlushMemory`, `common.cpp:313-326`) **synchronously before**
  panicking, and rejects recovering after `catch_unwind`. `com_error` is hosted in
  the `mp_engine_core` facade (STATE-D5), which can reach `Server` + `Client`. But
  the cited concrete deep throw sites — e.g. `SV_SvEntityForGentity` firing
  `ERR_DROP` several native frames deep inside a live `GAME_RUN_FRAME`
  (`sv_game.cpp:70-75`) — are **leaf-crate** code in `mp_engine_server`, which does
  **not** depend on the facade (its deps are `mp_qshared`/`mp_engine_qcommon`/
  `mp_abi`; workspace-architecture § Dependency edges), so such a site **cannot
  call the recovery-running `com_error`**, and threading a full `&mut Engine` down
  to every potential error site contradicts the master table's `&mut Engine.sv`/
  `&mut Engine.cl` per-subsystem scoping. The mechanism is undecided: (a) thread
  the full `&mut Engine` to error-capable sites; (b) inject a recovery
  callback/trait handle into `Server`/`Client`; (c) have leaf sites raise a
  **narrow** panic that a facade-level guard converts into recovery-then-rethrow
  (which pushes recovery *after* the first unwind — in tension with STATE-D3); or
  (d) amend STATE-D3. Its resolution also fixes `com_error`'s receiver signature
  (§ Seam, left `PENDING STATE-Q4`). This is **not** a subsystem-internals
  non-goal — it is exactly the STATE-D1/D3 threading contract, surfaced by the
  doc's own Chain-B worked example.

- **STATE-Q2 — `Engine`-island attachment for the four §F subcrates.**
  *Owner: the per-subsystem C++-track design docs
  (`docs/subsystems/{botlib,ghoul2,icarus,rmg}.md`).* botlib, ghoul2, icarus, and
  rmg have real engine-side Raven globals but were outside the A2 survey (dossier
  §1 censused only qcommon/server/client/sound/renderer; these appear there as
  readers, not owners). Their internal ownership is designed in their own §F
  subsystem docs (porting-rules §F, GP2 precedent), but whether/how their
  engine-side state becomes fields of `Engine` — especially ghoul2, shared
  engine↔cgame — is undecided. Placeholdered in the master table; resolve
  alongside those §F docs.
