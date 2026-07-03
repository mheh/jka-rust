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
  (whose *method set* this doc freezes below). The frozen blocks below
  additionally rest on these engine-seam IDs — **cited, not restated** (rule 4):
  **SEAM-D8** (the `Dispatch<C: InboundVmCall>` trait `fn dispatch(&self, args:
  C::Args) -> C::Output`, `InboundVmCall`'s `Args`/`Output` associated surface,
  and the per-command marker types — the `MpGameExport` variants, `mp/abi`); the
  `impl Dispatch<GameRunFrame> for WorldPtr` block in § `WorldPtr` is written
  against that trait, defined there. **SEAM-D6** (the `c_int`↔enum decoding the
  `dispatch(command: c_int, …)` router uses, and the `GAME_INIT`/`GAME_SHUTDOWN`/
  `GAME_RUN_FRAME` command constants the § `WorldCell` skeleton branches on).
  **SEAM-D13** (the module-side `pub type Engine` alias in `mp_engine_select` —
  a **different** type from this doc's engine-island `pub struct Engine`; the two
  are disambiguated in § Engine and STATE-Q8). **SEAM-Q12** (the still-open
  `&Engine` channel by which a `Dispatch<C>` body issues *outbound* traps
  mid-dispatch — engine-seam's open item, not re-opened here). The `ServerGame`
  dispatcher-arg's **concrete shape** (type alias vs. wrapper struct) is only
  *forward-declared* in engine-seam.md, not yet pinned there (§ Engine).
  engine-seam.md is **Status: DRAFT**; every "FROZEN HERE" block that cites a
  `SEAM-Dn` is gated on it freezing first — see STATE-Q7.
- `docs/abi-traps.md` — the generated `trap_*` signature reference; the seam
  below stores the register-once payloads of rows 18 (`trap_LocateGameData`)
  and 121 (`trap_SV_RegisterSharedMemory`), and references row 19
  (`trap_DropClient`) only as the STATE-D6 reentrancy trigger (not stored).
  Rows 6/18/121 are the exact signatures the Slice-0 `GAME_INIT` outbound-call
  arm emits; they live in abi-traps.md (a signature reference, linked not
  restated per rule 4).
- `docs/architecture/lifecycle.md` — **Status: DRAFT** (it now exists; the
  earlier "(pending)" annotations below are stale). Load-bearing co-reading: it
  owns the `ErrorLevel` enum (LIFE-D3 = the per-mode `errorParm_t` — MP
  `errorParm_t`, SP's 4-variant set — this doc's `ComError.level` type), the
  `Engine::new` / `com_init` command-line split (LIFE-D4b; `com_init(&mut engine,
  cmdline)` carries the raw command line, `Engine::new()` does not), the boot
  construction *order*, `com_printf` (LIFE-Q2 — but see STATE-D11's relocation,
  which lifecycle.md has not yet absorbed, STATE-Q7), and the module-registry
  step-30 attachment shared as LIFE-Q5/STATE-D10. Because it is DRAFT, the
  variants/order/split it owns are not yet reachable as frozen facts — STATE-Q7.
- `docs/dossiers/A2-state-ownership.md` — the survey this doc renders; its census
  §1a–1m (engine tier) and §2.1–2.5 (module tier) map group-for-group onto the
  master table below (§ Master table coverage note, for Gate 1).

## Scope & non-goals

**This doc decides:** the state-ownership spine (porting-rules §B made
concrete) — the master table mapping every Raven engine- and module-tier global
**the A2 survey censused** (qcommon/server/client/sound/renderer + the module
tiers; the four §F engine subcrates botlib/ghoul2/icarus/rmg were outside that
survey and are placeholdered pending their own §F docs, STATE-Q2) to its owning
Rust struct/crate/threading; the two-island model (one owned
`Engine`, one owned `GameWorld`); `GameWorld`'s shape, its zeroed heap
construction (`zeroed_box`/`GameWorld::zeroed`, STATE-D9) **and** its physical
storage location across `vmMain` calls — the `WorldCell` static in the module
cdylib shell, with `GAME_INIT`-write / `GAME_SHUTDOWN`-take lifetime (STATE-D6);
the module-island `Dispatch<C>` receiver (`WorldPtr`, STATE-D8); the reentrancy
contract (reborrow-threading + `EntityId` discipline, no `RefCell`, no effect
queues); and the error-recovery contract (`com_error` is a receiverless
format+panic in `mp_engine_qcommon`; all engine recovery runs catch-side in
`mp_engine_core`, STATE-D7). It **freezes `SharedGameData`** (deferred here by
engine-seam) and records the `Common.modules` module-table attachment (STATE-D10)
and `com_printf`'s `Common`-owned print state (STATE-D11).

**Non-goals** (each punted to its owning doc):

- **Seam executor mechanics** — how a trap crosses (`Execute<C>`/`Dispatch<C>`,
  the syscall/table wires, WASM marshalling) → `docs/architecture/engine-seam.md`.
- **Per-subsystem internals** — what each owned struct's fields are and what each
  handler *does* (cvar parse tables, filesystem search-path logic, sound mixer,
  collision internals) → `docs/subsystems/*` (pending).
- **Lifecycle / boot & shutdown sequences** — construction *order*, headless
  boot, the `ErrorLevel` enum variants (LIFE-D3), and the `Engine::new`/`com_init`
  command-line split → `docs/architecture/lifecycle.md` (**Status: DRAFT** — the
  doc exists and now renders all three; it freezes them, not this doc, and its
  DRAFT status is tracked by STATE-Q7). This doc references `ErrorLevel` and the
  split by ID only.
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

**Coverage (for Gate 1).** The subsystem groups below map group-for-group onto
the A2 dossier's census: qcommon (dossier §1a–1g) → the Common / Cvar-Cmd-Cbuf /
FileSystem-Net-Collision / zone-hunk blocks; Server (§1h), Client (§1i), Sound
(§1j), VM layer (§1k), Renderer (§1l, DEC-01-deferred), SP deltas (§1m) folded
into the per-mode rows; module tier (§2.1–2.5) → the MP/SP game and cgame/ui
blocks. The **only** censused globals *not* individually rowed are those the
dossier itself flags as deferred (renderer §1l → DEC-01; zone/hunk §1e → §C9
drop). The four §F engine subcrates (botlib/ghoul2/icarus/rmg) were **outside**
the dossier's census (readers, not owners) and are placeholdered pending their
own docs — STATE-Q2. So "state table covers the survey's global inventory" holds
by this group-for-group correspondence.

**qcommon — Common (`mp/engine/qcommon::Common`, a field of `Engine`)**

`Common` is a **new** top-level module added to `mp_engine_qcommon`'s `lib.rs`
alongside its existing `cm`/`files`/`gp2`/`miniheap`/`qcommon`/`qfiles`/`timing`/
`vm` modules (no `common` module exists there yet); it owns the `common.cpp`
globals below plus the `cvars`/`cmd`/`cbuf`/`fs`/`net`/`modules` sub-structs. It
also hosts `com_printf(&mut Common, …)` (STATE-D11, whose print-adjacent fields
`redirect`/`logfile` are rowed below) and the receiverless `com_error` (STATE-D7)
— both defined here, below the facade, so the leaf callers that print/throw reach
them.

| Raven global | oracle cite | Rust owner | constructed by | threaded via |
|---|---|---|---|---|
| `com_frameTime`/`com_frameMsec`/`com_frameNumber` | `common.cpp:79-81` | `Common.frame_{time,msec,number}` | `Com_Init` | `&mut Engine` |
| `com_errorEntered`/`com_errorMessage[4096]` | `common.cpp:83,86` | `Common.error` (re-entry guard + last msg) | `Com_Init` | `&mut Engine`; **set/cleared catch-side by the `mp_engine_core` recovery** (STATE-D3/D7) |
| `com_fullyInitialized` | `common.cpp:84` | `Common.fully_initialized` (boot-complete flag) | `Com_Init` (set `common.cpp:1434`) | `&mut Engine` |
| `rd_buffer`/`rd_buffersize`/`rd_flush` | `common.cpp:92-94` | `Common.redirect` — **`com_printf`'s rcon console-redirect target** (STATE-D11) | `Com_BeginRedirect` | `&mut Common` (`com_printf` receiver, STATE-D11) |
| `logfile`/`debuglogfile`/`com_logfile` cvar | `common.cpp:32-33,50` | `Common.logfile` — **`com_printf`'s logfile sink** (STATE-D11; full `com_printf`-state record in lifecycle.md) | `Com_Init` / lazily on first `Com_Printf` (`common.cpp:171-181`) | `&mut Common` |
| `com_journalFile`/`com_journalDataFile` | `common.cpp:34-35` | `Common.journal` | `Com_InitJournaling` | `&mut Engine` |
| `com_pushedEvents[1024]`/`Head`/`Tail` | `common.cpp:749-752` | `Common.event_queue` | `Com_InitJournaling` | `&mut Engine` |
| `com_argc`/`com_argv[]`/`com_consoleLines[32]`/`com_numConsoleLines` | `common.cpp:22-23,387-388` | `Common.cmdline` (startup) | `Com_ParseCommandLine` | `&mut Engine` |
| ~30 `cvar_t*` handles (`com_dedicated`, `com_sv_running`, `com_cl_running`, …) | `common.cpp:37-72` | `Cvar` handles resolved from `Common.cvars` | `Cvar_Get` at init | handle, not raw ptr |
| `vmTable[3]`/`gvm`/`cgvm`/`uivm`/`vm_debugLevel` (the module table) | `vm.cpp:28-29`, `sv_main.cpp:12`, `cl_main.cpp:108`, `cl_ui.cpp:28` | `Common.modules: ModuleRegistry` — Raven's `vmTable` is a **qcommon-subsystem file-scope static** (`vm.cpp:28-29`), so its Rust owner is a field of `Common` (STATE-D10; attachment shared with lifecycle.md LIFE-Q5). Per-slot host state, the `ModuleSlot.engine` trampoline cell, and the `ModuleTransport` shapes remain engine-seam's / module-loading's (§ VM-layer block below) | `Com_Init` step 30 (`VM_Init`), constructed **empty** (lifecycle.md § Com_Init) | `&mut Engine.common` |

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
| `g_lastResolveTime[MAX_MASTER_SERVERS]` | `sv_main.cpp:192` | `Server.master_heartbeat` (master-server DNS-resolve throttle; MP only, `#ifndef _XBOX`) | `SV_Init` | `&mut Engine.sv` |
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

**VM layer / module handles — container attaches at `Common.modules` (STATE-D10); per-slot internals → engine-seam.md (SEAM) / module-loading.md**

| Raven global | oracle cite | Rust owner | constructed by | threaded via |
|---|---|---|---|---|
| `vmTable[3]`/`gvm`/`cgvm`/`uivm`/`vm_debugLevel` | `vm.cpp:28-29`, `sv_main.cpp:12`, `cl_main.cpp:108`, `cl_ui.cpp:28` | **`Common.modules: ModuleRegistry`** (mp_engine_qcommon; the container is a `Common` field per STATE-D10 — Raven's `vmTable` is a qcommon-subsystem static). Per-loaded-module host state + `ModuleTransport` internals are engine-seam § State ownership; the per-slot `ModuleSlot.engine` cell is SEAM-D11 (row below) | `Com_Init` step 30 (`VM_Init`), empty | `&mut Engine.common`; dispatcher `engine` arg for calls |
| `currentVM`/`lastVM` | `vm.cpp:24-25` | **eliminated** — each dispatch explicitly parameterized (engine-seam SEAM-D1; Raven's `VM_Free` global-clobber / skipped-restore bug **not** reproduced, STATE-D3) | — | — |
| per-slot `*mut Engine` trampoline stash cell (supplants `currentVM`'s bridging role) | (new; supplants `vm.cpp:800`) | `mp_engine_qcommon::ModuleRegistry`'s per-slot `ModuleSlot.engine` cell — **owned by engine-seam.md (SEAM-D11), not here**; one cell per module slot (never one global, STATE-D2) | `EngineSlotGuard::enter` at each engine→module call | read only by that slot's raw `extern "C-unwind"` syscall trampoline (engine-seam § Inbound raw syscall trampoline, SEAM-D11) |

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
// also hosts com_init/com_frame/com_shutdown (they must reach the server+client
// crates); com_error itself lives one tier lower in mp_engine_qcommon (STATE-D7),
// its recovery run by com_frame/com_init's catch. Field *sub*-structs keep their
// own subcrate homes below.
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
                                 //   constructs it — the dedicated our-engine build sets
                                 //   `snd: None` (grounded in the cited gating + DEC-03;
                                 //   same Sound master-table row).
    // botlib/ghoul2/icarus/rmg engine-side state is NOT yet a field here — those
    // four §F subcrates were outside the A2 survey; attachment point is STATE-Q2.
}
```

The game dispatcher's `engine: &mut ServerGame` parameter (engine-seam,
`sv_game_system_calls`) is the **server-island reborrow** — `&mut Engine.sv`'s
`Server`, carrying its `SharedGameData` registration. `ServerGame` is
engine-seam's name for exactly that reborrowed host state; the two names denote
the same value (kept consistent per that doc). Its **concrete shape** (a
`type ServerGame = Server` alias vs. a wrapper struct) is a seam-executor
mechanic (§ Non-goals) that engine-seam.md only *forward-declares*; it is not
pinned in either doc yet, and rides the same DRAFT-sibling gate as the other
SEAM IDs (STATE-Q7). This doc fixes only *which value* `ServerGame` denotes.

**Two distinct types are both named `Engine` — disambiguation (STATE-Q8).** This
doc's `pub struct Engine` (crate `mp_engine_core`) is the **engine-island
aggregate** (`common`/`sv`/`cl`/`cm`/`snd`). It is a **different type** from
engine-seam SEAM-D13's `pub type Engine` (crate `mp_engine_select`), which is the
**module-side outbound-transport alias** (`CEngine`/`Static`/wasm) that logic
crates thread as `&Engine` to *issue* traps. Same bare name, different crates,
different purpose. Consequently, SEAM-Q12's phrase "a `Dispatch<C>` body needs
the `&Engine` to issue outbound traps mid-dispatch" refers to the **module-side
`mp_engine_select::Engine`** (or a still-undefined `svc` handle SEAM-Q12 may
mint) — **not** this struct: `mp_game` (where the `Dispatch<C>` impls live,
STATE-D8) depends only on `mp_qshared`/`mp_bg`/`mp_abi` (workspace-architecture
§ Dependency edges) and can never gain an edge to the `mp_engine_core` facade
(an illegal upward edge), so `mp_engine_core::Engine` is structurally
unreachable from a `Dispatch<C>` body. The resolution of *that* channel is
engine-seam's SEAM-Q12; whether the two `Engine` names should be renamed to
remove the collision is STATE-Q8 (Open questions).

### `com_init` / `com_frame` / `com_shutdown` (`mp_engine_core`) + `com_error` (`mp_engine_qcommon`) — entry points (FROZEN HERE)

The engine island's headline **recovery-holding** entry points — `com_init` /
`com_frame` / `com_shutdown` — are **defined in `mp_engine_core`**, the aggregate
facade, the only engine crate that can reach both `Server` and `Client`
(workspace-architecture § Dependency edges; STATE-D5). A leaf subcrate **cannot**
host them: `mp_engine_server`'s deps are `mp_qshared`/`mp_engine_qcommon`/`mp_abi`
only (no facade edge). `&mut Engine` threads **down** from here (STATE-D1); an
orchestrator takes the whole `Engine` and hands each subsystem its narrower reborrow
(`&mut Engine.sv` / `&mut Engine.cl`, master table) as it calls down. **`com_error`
is the exception** — it holds no `Engine` and does no recovery (STATE-D7), so it
lives *below* the facade in `mp_engine_qcommon` where the leaf throw sites reach it;
its recovery is what `com_frame`/`com_init`'s catch runs (below).

```rust
// mp/engine/core (SP: sp/engine/core). Free functions on the facade.

/// `void Com_Frame( void )` (`common.cpp:1593`). One host frame. Takes **no**
/// frame-delta argument — `msec` is computed internally (`Com_ModifyMsec`,
/// `common.cpp:1660`) before dispatching `sv_frame`/`cl_frame`
/// (`common.cpp:1669,1711`). Hosts the DEC-08 `catch_unwind` that catches a
/// `com_error` panic (throw caught at `common.cpp:1762`) and — because `com_error`
/// now only formats + panics (STATE-D7) — runs **all** of Raven's per-level
/// recovery *here*, `&mut Engine` in hand: the `errorEntered` recursion guard +
/// `ErrorState` bookkeeping (`Engine.common.error`), the `ERR_DROP` banner,
/// `SV_Shutdown`, `CL_Disconnect`, reproducing oracle's console-output order
/// (`common.cpp:288-344`). A panic *during* recovery escalates to the outer
/// `com_init`/`main` catch — Raven's recursive-error `ERR_FATAL` path
/// (`com_errorEntered` re-entry → `Sys_Error`, `common.cpp:288-289`) (STATE-D7).
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
command line enters this init pair. **Not fixed here — owned by lifecycle.md
(§ Non-goals, construction order):** the division of labor between `Engine::new`
and `com_init`, and hence which of the two carries the command-line parameter.
lifecycle.md (**DRAFT**) now *renders* that split — `Engine::new()` takes no
command line (it captures the timer base, LIFE-D4b), and `com_init(&mut engine,
cmdline)` carries the raw command line and runs `Com_ParseCommandLine`
(lifecycle.md § Com_Init) — but it freezes there, not here, and its DRAFT status
gates any downstream boot-sequence code (STATE-Q7). The our-engine-hosting slice
that first builds `Engine` already depends on lifecycle.md for construction order
(§ Slice hooks), so this rides along; STATE freezes only that `Engine::new ->
Engine` yields the owned island and that all four `com_*` live in the facade.

**`com_error` is a receiverless leaf throw (STATE-D7).** Its payload is fixed —
`level: ErrorLevel` + a formatted `msg: String` (the `ComError` struct below;
Raven `void QDECL Com_Error( int code, const char *fmt, ... )`, `common.cpp:249`,
whose varargs are formatted caller-side into `msg`, mirroring `vsprintf` into
`com_errorMessage`, `common.cpp:293-295`) — and it **diverges**, `-> !` via
`panic_any(ComError)`. It takes **no receiver**: it *only* formats and panics, so
the concrete deep leaf throw sites (`SV_SvEntityForGentity`, `sv_game.cpp:70-75`,
in `mp_engine_server`) can call it directly. It therefore lives in
**`mp_engine_qcommon`** — the lowest engine crate every other engine crate depends
on — **not** the `mp_engine_core` facade the leaf crates cannot reach (STATE-D7,
resolving the former STATE-Q4). **None** of Raven's pre-throw engine work runs here;
it all moves to the *catch* side (`com_frame`/`com_init` recovery in
`mp_engine_core`, where `&mut Engine` is in hand — see `com_frame` above and
STATE-D7).

```rust
// mp/engine/qcommon. NO receiver, NO recovery here — pure format + diverge.
// All of Raven's pre-throw work (errorEntered guard, ErrorState bookkeeping,
// the ERR_DROP banner, SV_Shutdown, CL_Disconnect) runs catch-side in
// mp_engine_core's com_frame/com_init catch (STATE-D7).
pub fn com_error(level: ErrorLevel, msg: String) -> !;
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
- **`level`'s self-referencing pointers + client back-pointers — intra-`GameWorld`
  construction order.** (Grounded in the oracle sequence cited below,
  `g_main.c:978-994`; not a separate decision record.) The full `G_InitGame` entity
  block is a five-step faithful sequence, all in the `GAME_INIT` dispatched arm
  (§ `WorldPtr`), *after* `G_RegisterCvars` has run (`g_main.c:931`; provenance in
  the next bullet): (1) zero the `entities` box (Raven `memset g_entities`,
  `g_main.c:978`) and set the `level.gentities` raw back-pointer to alias it
  (`g_main.c:979`); (2) read `level.maxclients = g_maxclients.integer`
  (`g_main.c:982`); (3) zero the `clients` box (`memset g_clients`, `g_main.c:983`)
  and set `level.clients` to alias it (`g_main.c:984`); (4) wire each valid client
  slot's entity back-pointer `entities[i].client = &mut clients[i]` for
  `i in 0..level.maxclients` (`g_main.c:987-988`) — into the real ported
  `gentity_t.client` field (`crates/mp/qshared/src/common/mp/gentity.rs:125`,
  `*mut c_void`, offset-asserted 976 at `:463`), a **second** set of
  intra-`GameWorld` self-referencing raw pointers (entities → the `clients` box);
  (5) set `level.num_entities = MAX_CLIENTS` (`g_main.c:994`), the count
  `trap_LocateGameData` then registers (`g_main.c:997`). The data-dependency this
  doc fixes: **heap-allocate the `entities`/`clients` boxes first, then set every
  back-pointer (`level.gentities`, `level.clients`, and each `entities[i].client`)
  to alias them** — the boxes must exist before any pointer can name them (mirroring
  `g_main.c:978-988`). The *broader* boot sequence (when `G_InitGame` runs relative
  to `SV_SpawnServer`, the `ErrorLevel` teardown order) stays lifecycle.md's
  (§ Non-goals). De-risking note: `level.gentities` is write-once/registration-only
  — never indexed by game logic (0 hits for `level.gentities[` across `codemp/`), so
  it is inert once wired; `level.clients` and the `entities[i].client` back-pointers,
  by contrast, *are* client access paths (the engine dereferences `ent->client`).
  The zeroed *behavior* is faithful to Raven's `memset` of the `g_entities`/
  `g_clients` statics + `level` zero-init (`g_main.c:978-984`); those are C statics,
  but the Rust boxes are built zeroed **on the heap** by `GameWorld::zeroed`
  (STATE-D9, via the `zeroed_box` helper — § `zeroed_box` / `GameWorld::zeroed`
  below), then the back-pointers alias the boxes in the allocate-first order above.
  Both the construction *order* and the zeroing *mechanism* are now fixed (STATE-D9
  closed the former STATE-Q6).
- **`level.maxclients` / `level.num_entities` provenance — an outbound
  `trap_Cvar_Register`, not an engine `Common.cvars` read.** `g_maxclients` is a
  **module-side `vmCvar_t`** (`g_main.c:111`), registered as `sv_maxclients` in the
  game's cvar table (`g_main.c:249`) by `G_RegisterCvars`'s `trap_Cvar_Register`
  (`g_main.c:809,931`; abi-traps.md row 6) — an **outbound** trap the engine services
  by populating the module's own `vmCvar_t` copy. So `level.maxclients =
  g_maxclients.integer` (`g_main.c:982`) reads module-side state the module already
  synced, **not** `Engine.common.cvars`. The `GAME_INIT` arm obtains the value by
  *emitting* `trap_Cvar_Register` for each table row (as it emits
  `trap_LocateGameData`/`trap_SV_RegisterSharedMemory`) and reading back the synced
  integer; in **Slice 0** the real/OpenJK host services that trap (§ Slice hooks),
  symmetric with the other `GAME_INIT` outbound emissions — no `Common.cvars` of ours
  is needed. The Rust owner of the module `vmCvar_t` table (`g_maxclients` et al.) is
  a game-subsystem-internal detail (§ Non-goals — per-subsystem internals), not fixed
  here; only the value's *provenance* is.

### `zeroed_box` / `GameWorld::zeroed` — zeroed heap construction (STATE-D9, FROZEN HERE)

Raven's `g_entities`/`g_clients`/`level` are C statics the loader zero-fills once
and `G_InitGame` `memset`s (`g_main.c:978-984`); the Rust `GameWorld` heap-boxes
them instead (§ `GameWorld`), so the zeroed *behavior* needs a heap idiom Raven
never had. The sanctioned one is a single native-tier helper — the Rust mirror of
C static zero-initialization for large all-zeroes-valid `#[repr(C)]` types:

```rust
// crates/native/platform (package native_platform), beside the other platform
// primitives — Raven-free native-tier vocabulary. Tier-legal for mp_game (native/* is
// "everything, cross-mode", workspace-architecture § tier definitions), so no tier-rule
// change — but mp_game does NOT yet depend on native_platform (its deps are
// mp_qshared/mp_bg/mp_abi; mp_qshared depends on native_platform but does not re-export
// it, so the edge is not transitive), so a new direct Cargo dependency edge
// mp_game -> native_platform must be added as a slice-0 wiring task.
// THE sanctioned construction idiom for large #[repr(C)] all-zeroes-valid types:
// alloc_zeroed the storage and Box::from_raw it, so the ~1.83 MB entity array is
// built directly on the heap and never transits the stack (naive stack-build-then-box
// risks overflow on constrained-stack targets, e.g. 1 MB default Windows threads).
//
// SAFETY (stated ONCE, here): the caller guarantees `T` is `#[repr(C)]` and that the
// all-zero bit pattern is a valid value of `T` (no NonNull/enum-niche/reference fields).
pub fn zeroed_box<T>() -> Box<T>;
```

```rust
// mp/game (SP mirror). Builds the zeroed island, then wires level's self-referencing
// back-pointers in the allocate-first order (§ GameWorld construction-order bullet).
// The gentity_t / gclient_t / level_locals_t types satisfy zeroed_box's #[repr(C)]
// all-zero contract — they are the same all-zeroes-valid ABI structs Raven memsets.
impl GameWorld {
    pub fn zeroed() -> Self {
        let entities = zeroed_box::<[gentity_t; MAX_GENTITIES]>();
        let clients  = zeroed_box::<[gclient_t; MAX_CLIENTS]>(); // MP only; SP omits
        let level    = *zeroed_box::<level_locals_t>();
        // level.gentities/clients + each entities[i].client back-pointer are wired to
        // alias the boxes AFTER they exist (g_main.c:978-988), in G_InitGame's dispatched
        // arm (after G_RegisterCvars sources level.maxclients, next §) — not here.
        GameWorld { level, entities, clients }
    }
}
```

The `unsafe` lives entirely inside `zeroed_box` (one native-tier helper, contract
stated once); `GameWorld::zeroed` and the `GAME_INIT` arm that calls it are safe.
This resolves the former STATE-Q6 (construction idiom, its §D11 licensing, and
whether the ABI types gain a zeroing constructor — they do, via this shared helper
rather than per-type `Default`).

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
// SAFETY (Sync only): the module runs single-threaded per Raven's contract, so the static is
// never touched from a second thread. Single-threaded *reentrant* aliasing is a separate hazard,
// handled by the raw-pointer threading in vmMain below — not by this impl.
unsafe impl Sync for WorldCell {}
```

Access discipline — the only `unsafe` in the shell, confined per §D11 (STATE-D6
amendment 2026-07-02: thread a **raw `*mut GameWorld`** wrapped in the `Copy`
`WorldPtr` the `Dispatch<C>` impls take by value, never a dispatch-spanning `&mut`):

```rust
// crates/jampgame shell. C entrypoint takes NO context arg, so the world is reached via WORLD.
// GAME_INIT constructs the GameWorld INTO the cell and falls through to the generic dispatch
// (whose GAME_INIT arm runs G_InitGame's init logic against it); GAME_SHUTDOWN takes it back
// OUT after its dispatch returns; every other command re-derives a raw `*mut GameWorld` from
// the cell and threads it (as `WorldPtr`) through dispatch. A `&mut GameWorld` is reborrowed
// only at non-overlapping leaf accesses inside the Dispatch bodies (the STATE-D1 EntityId
// discipline), always dropped before any `trap::` call that can re-enter vmMain — so no `&mut`
// ever spans a reentrant entry.
#[no_mangle]
pub extern "C-unwind" fn vmMain(command: c_int, /* args… */) -> c_int {
    // BOOTSTRAP (STATE-D6 amendment 2026-07-02). GAME_INIT is the ONE command that WRITES the
    // cell before reading it: it stores a zeroed GameWorld (GameWorld::zeroed, STATE-D9),
    // THEN falls through so the dispatched GAME_INIT arm does G_InitGame's init against it
    // (g_main.c:515,979). WORLD starts `None`, so this write-then-fallthrough is what keeps the
    // shared read below from panicking on the very first call. map_restart's fresh GAME_INIT
    // overwrites the cell with a fresh world.
    if command == GAME_INIT /* MpGameExport::GameInit, decoded per SEAM-D6 */ {
        // SAFETY: single-threaded init; no reentrancy is possible before the world exists.
        unsafe { *WORLD.0.get() = Some(GameWorld::zeroed()); } // STATE-D9 zeroed-construction idiom
    }
    // SAFETY: single-threaded per Raven's contract, and — the real hazard — single-threaded
    // REENTRANCY: the engine re-enters vmMain from inside a syscall handler (bullet below) while
    // this frame is suspended. We derive a RAW *mut (not a live &mut) and wrap it in the Copy
    // WorldPtr the Dispatch<C> impls take by value, so each reentrant entry derives its OWN raw
    // pointer; aliasing raw pointers are sound, a dispatch-spanning &mut would be UB even
    // single-threaded. &mut GameWorld is reborrowed only at leaf accesses that never span a
    // trap:: re-entry (STATE-D8).
    let world = WorldPtr(
        unsafe { (*WORLD.0.get()).as_mut().expect("GAME_INIT built the world") } as *mut GameWorld,
    );
    // `dispatch` routes each command to its per-call `Dispatch<C> for WorldPtr` impl (engine-seam
    // SEAM-D8; receiver frozen as WorldPtr by STATE-D8). The impl bodies do the STATE-D8 leaf
    // `&mut *world.0` reborrows, so a MUTATING command (GAME_RUN_FRAME, GAME_CLIENT_*) mutates
    // level/entities through those leaf borrows — no RefCell, no effect queue (STATE-D1).
    let result = dispatch(command, world, /* args… */); // dispatch(command: c_int, world: WorldPtr, …)
    // GAME_SHUTDOWN takes the world OUT of the cell AFTER its dispatch returns — module-unload
    // lifetime; dropping the Some(GameWorld) runs the owned island's Drop (§C9). STATE-D6 amend.
    if command == GAME_SHUTDOWN /* MpGameExport::GameShutdown */ {
        // SAFETY: single-threaded; the just-returned GAME_SHUTDOWN dispatch holds no live borrow.
        unsafe { *WORLD.0.get() = None; }
    }
    result
}
```

- **Reentrancy is by design, not a hazard** (STATE-D6, amended 2026-07-02). The
  engine re-enters `vmMain` from inside a syscall handler — `G_DROP_CLIENT`
  (`sv_game.cpp:570`) → `SV_GameDropClient` (`sv_game.cpp:110-114`) →
  `SV_DropClient` (`sv_client.cpp:580`) → `VM_Call(gvm, GAME_CLIENT_DISCONNECT)`
  (`sv_client.cpp:640`), a fresh `vmMain` entry while the outer one is suspended.
  Each entry derives its **own** `*mut GameWorld` (a fresh `WorldPtr`) from the cell;
  the two live raw pointers alias, which is **sound** — only a live `&mut` spanning
  the re-entry would be UB. (Raven's C `level`/`g_entities` globals aliased freely
  under these chains because C has no `&mut`; the Rust port reproduces that aliasing
  with raw pointers + leaf reborrows, **not** a dispatch-spanning `&mut`.) This is
  why the cell is a raw `UnsafeCell` and **not** a `Mutex` (deadlocks on those
  chains) or `RefCell` (panics on them): the safety argument is Raven's
  single-threaded sequencing **plus** the raw-pointer threading above, made explicit
  at this one seam.
- **`extern "C-unwind"`** (engine-seam SEAM-D10) so a `com_error` panic unwinds
  through the live C frames of a re-entrant chain (STATE-D3).
- The `EntityId` re-borrow discipline (below) applies to every access *inside* the
  `Dispatch<C>` bodies, so no raw `gentity_t*` alias survives above the seam.
- **GAME_INIT / GAME_SHUTDOWN lifetime** (STATE-D6 amended). `GAME_INIT` is the sole
  command that *writes* the cell before reading it (via `GameWorld::zeroed`, STATE-D9,
  now a concrete idiom), and `GAME_SHUTDOWN` the sole one that *takes it out* after
  dispatch (drop = teardown). The two former open sub-mechanics are now resolved: the
  zeroed-`GameWorld` construction (was STATE-Q6 → STATE-D9) and the mutating-command
  receiver (was STATE-Q5 → STATE-D8, `WorldPtr` below).

### `WorldPtr` — the `Dispatch<C>` receiver (STATE-D8, FROZEN HERE)

engine-seam froze the `Dispatch<C: InboundVmCall>` trait and its
`fn dispatch(&self, args: C::Args) -> C::Output` (SEAM-D8) and punted **which
`Self` each `Dispatch<C>` impl reads/writes** to this doc. The trait, the
`InboundVmCall` bound (its `Args`/`Output` associated types), and the concrete
per-command marker types (`GameRunFrame`/`GameInit`/`GameShutdown` — the
`MpGameExport` variants, `mp/abi`, SEAM-D6) are engine-seam's, read there and not
restated here (rule 4); the impl block below is written against them. The
receiver this doc settles is a copyable wrapper over the raw world pointer
`vmMain` threads — **not** `GameWorld` itself:

```rust
// mp_game (the crate the orphan rule forces the `impl Dispatch<C>` blocks into,
// engine-seam § SEAM-D8). A Copy wrapper over the raw `*mut GameWorld` derived from
// WORLD once per vmMain entry (§ WorldCell). This is porting-rules §17's
// arena/copyable-borrow-wrapper idiom (the GP2 precedent), NOT a field of GameWorld.
#[derive(Clone, Copy)]
pub struct WorldPtr(pub *mut GameWorld);

// Each per-call InboundVmCall command C gets one impl; `&self` on the Copy wrapper
// satisfies SEAM-D8's frozen signature WITHOUT changing the trait. The body reborrows
// `&mut *self.0` only at non-overlapping LEAF accesses and drops that `&mut` before any
// `trap::` call that can re-enter vmMain (STATE-D1 EntityId discipline) — so a mutating
// command mutates level/entities without any RefCell/effect queue, and no `&mut` ever
// spans a reentrant entry.
impl Dispatch<GameRunFrame> for WorldPtr {
    fn dispatch(&self, args: <GameRunFrame as InboundVmCall>::Args)
        -> <GameRunFrame as InboundVmCall>::Output
    {
        // e.g. g_run_frame(unsafe { &mut *self.0 }, args) at a leaf that holds no
        // borrow across a trap:: re-entry; nested reborrows re-derive from self.0.
        todo!("Port G_RunFrame — oracle/oracle/codemp/game/g_main.c:3582")
    }
}
```

The impl pattern is **frozen**: `Dispatch<C>` is implemented on `WorldPtr` (by value,
`Copy`), never on `GameWorld`; bodies reborrow `&mut GameWorld` at leaves and drop it
before re-entrant `trap::` calls. This resolves the former STATE-Q5 (mutating-command
receiver). One thing this doc does **not** settle: how a `Dispatch<C>` body that must
issue *outbound* traps mid-dispatch (e.g. `GAME_INIT` → `trap_LocateGameData`) obtains
the `&Engine`/`svc` those traps need — `WorldPtr` carries only the world. That
`&Engine`-access question is engine-seam's **SEAM-Q12** (§ Standing context →
engine-seam.md), owned there, not re-opened here. Note the `&Engine` there is the
**module-side `mp_engine_select::Engine`** transport alias (SEAM-D13), *not* this
doc's engine-island `Engine` struct — `mp_game` cannot reach `mp_engine_core` (§
Engine disambiguation, STATE-Q8).

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
// mp/engine/qcommon (beside com_error, STATE-D7). Raven's Com_Error(level, msg)
// becomes a pure format + panic_any(ComError); ALL recovery runs catch-side at
// com_frame/com_init in mp_engine_core. Caught by catch_unwind (DEC-08).
pub struct ComError {
    pub level: ErrorLevel,   // frozen in lifecycle.md (LIFE-D3); variants trace to
    pub msg: String,         //   ERR_FATAL/DROP/DISCONNECT/SERVERDISCONNECT/NEED_CD
}                            //   (common.cpp:302-344)
```

`ErrorLevel` is per-mode (MP `errorParm_t` vs SP's 4-variant set) and is **owned by
`docs/architecture/lifecycle.md` (LIFE-D3; that doc is Status: DRAFT, so its
variants are not yet a frozen fact — STATE-Q7)**, not here; this doc fixes the
payload shape, the recovery-ordering contract (STATE-D3, recovery relocated
catch-side by STATE-D7), and — via STATE-D7 — the crate home (`mp_engine_qcommon`)
and the receiverless `com_error` signature. The
`com_error` *function* signature (this payload as the panic value, `-> !`, no
receiver) is in § `com_init`/`com_frame`/`com_shutdown`/`com_error` above.

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
`Result`-threading (DEC-08 — reshapes thousands of faithful signatures).
*Amendment (2026-07-02, resolves STATE-Q4 → STATE-D7):* the recovery **location**
moves from com_error-side to the **catch side**. `com_error` becomes a receiverless
format+panic in `mp_engine_qcommon` (reachable by the leaf throw sites) and the full
per-level recovery runs in `mp_engine_core`'s `com_frame`/`com_init` catch, `&mut
Engine` in hand, reproducing oracle's console-output order. This does **not** revive
the rejected "handler observes half-torn state" hazard: the catch *is* what tears the
subsystems down (in oracle order), so nothing is half-torn before it runs. STATE-D3's
Drop-safety and no-`currentVM`-global contracts are unchanged; see STATE-D7.

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
also hosts `com_init`/`com_frame`/`com_shutdown` (they must reach server+client);
STATE-D7 later relocated `com_error` itself one tier lower to `mp_engine_qcommon`
(the facade only runs its recovery, catch-side). `Engine` is **instantiated** by `Engine::new` from the thin
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
dispatch, which derives a raw `*mut GameWorld` from the cell once per entry and
threads that pointer inward (reborrowing `&mut GameWorld` only at leaf accesses,
amendment below); same cell shape in all three transports (module-island state).
**Reentrancy contract:** the engine genuinely re-enters `vmMain` from inside a
syscall handler — `G_DROP_CLIENT` (`sv_game.cpp:570`) → `SV_GameDropClient`
(`sv_game.cpp:110-114`) → `SV_DropClient` (`sv_client.cpp:580`) →
`VM_Call(gvm, GAME_CLIENT_DISCONNECT)` (`sv_client.cpp:640`), a fresh `vmMain`
entry while the outer one is live — each entry derives its **own** `*mut GameWorld`
from the cell, so the two live raw pointers alias soundly (Raven's C `level`/
`g_entities` globals aliased freely under those chains because C has no `&mut`).
*Because* the C `vmMain(command, args…)` entrypoint takes no context argument, so a
mutable-across-calls owned value must be reachable without one, and those live
re-entry chains alias it by design. *Rejected:* `Mutex` (deadlocks on those chains);
`RefCell` (panics on them); a context-threaded entrypoint (the ABI is fixed — no
context slot in `vmMain`).

*Amendment (2026-07-02, soundness fix):* `vmMain` threads a raw `*mut GameWorld`
through `dispatch` and reborrows `&mut GameWorld` only at non-overlapping **leaf**
accesses (dropped before any re-entrant `trap::` call), because a single
dispatch-spanning `&mut` across the re-entry chain above is UB in Rust even
single-threaded (aliasing raw pointers are not). The earlier framing of a
"dispatch-spanning `&mut` aliasing the world exactly as Raven's globals did" is
corrected — that holds for C raw globals only. Mechanism in the `vmMain` SAFETY note
(§ `WorldCell`).

*Amendment (2026-07-02, round 3, bootstrap + world lifetime):* the frozen `vmMain`
sketch is fixed to remove the first-call panic. **`GAME_INIT`** is the sole command
that *writes* the cell before the shared read — it constructs a zeroed `GameWorld`
(`GameWorld::zeroed`, STATE-D9) into `WORLD`, then falls through to the generic
dispatch path so the dispatched `GAME_INIT` arm runs `G_InitGame`'s init logic against
it (mirroring C's static allocation at load time, then `memset`+init, `g_main.c:515,
978-984`). **`GAME_SHUTDOWN`** is the sole command that *takes the world out* of the
cell, after its dispatch returns — module-unload lifetime; the dropped `GameWorld`'s
`Drop` tears down the owned island (§C9). `map_restart`'s fresh `GAME_INIT` overwrites
the cell with a fresh world. No unconditional `.expect()` fires on the first call. The
raw-pointer receiver of the earlier amendment is now the `Copy` `WorldPtr` wrapper the
`Dispatch<C>` impls take by value (STATE-D8). Skeleton in § `WorldCell`.

**STATE-D7 — `com_error` is a receiverless leaf throw; recovery runs catch-side
(resolves STATE-Q4; session 2026-07-02).** `com_error` is a **free function in
`mp_engine_qcommon`** — the lowest engine crate every other engine crate depends
on — with **no receiver**: `pub fn com_error(level: ErrorLevel, msg: String) -> !`.
It *only* formats the message and diverges via `panic_any(ComError)`; the `ComError`
payload lives beside it in `mp_engine_qcommon`. This lets the concrete deep leaf
throw sites (`SV_SvEntityForGentity`, `sv_game.cpp:70-75`, in `mp_engine_server`)
call it directly — the earlier `mp_engine_core` facade home foreclosed that, since
`mp_engine_server` cannot depend on the facade (workspace-architecture § Dependency
edges). **All** of Raven's pre-throw engine work — the `errorEntered` recursion
guard and `ErrorState` bookkeeping in `Engine.common.error`, the `ERR_DROP` banner
print, `SV_Shutdown`, `CL_Disconnect` (`common.cpp:288-344`) — moves to the **catch
side** in `mp_engine_core`: `com_frame`/`com_init`'s `catch_unwind` block, where
`&mut Engine` is in hand, and the catch **reproduces oracle's console-output order**
exactly. A panic raised *during* recovery escalates to the outer `com_init`/`main`
catch — Raven's recursive-error `ERR_FATAL` path (`com_errorEntered` re-entry →
`Sys_Error`, `common.cpp:288-289`). This **amends STATE-D3's recovery *location***
(com_error-side → catch-side) while keeping its Drop-safety and no-`currentVM`-global
contracts and preserving the observable order (the catch, not the leaf, tears the
subsystems down, so no handler observes half-torn state). **Cross-doc:** this
supersedes `workspace-architecture.md`'s engine-tier line (§ crate tiers and
§ Dependency edges); that doc now annotates `mp/engine/core` as hosting only
`com_error`'s *recovery*, not its definition (which lives one tier lower in
`mp_engine_qcommon`), reconciled to this decision. *Because* the leaf throw
sites are `mp_engine_server` code that cannot reach the facade, so the divergence
point (`panic_any`) must live where every crate can name it, and the `&mut
Engine`-holding recovery must live where the facade already is. *Rejected:* threading
a full `&mut Engine` to every error-capable leaf site (contradicts the master table's
`&mut Engine.sv`/`&mut Engine.cl` per-subsystem scoping, STATE-D1); injecting a
recovery callback/trait handle into `Server`/`Client` (more machinery than a
catch-side handler that already holds `&mut Engine`); a facade guard that converts a
narrow leaf panic into recover-then-rethrow (pushes recovery after a *first* unwind
for no gain now that the catch owns teardown).

**STATE-D8 — The `Dispatch<C>` receiver is `WorldPtr`, a `Copy` `*mut GameWorld`
wrapper (resolves STATE-Q5; session 2026-07-02).** engine-seam froze
`fn dispatch(&self, args: C::Args) -> C::Output` (SEAM-D8) and deferred *which* `Self`
each impl reads/writes to this doc. That `Self` is **`pub struct WorldPtr(pub *mut
GameWorld)`** — a copyable wrapper (porting-rules §17's arena/copyable-borrow-wrapper
idiom, the GP2 precedent), defined in **`mp_game`** (the crate the orphan rule forces
`impl Dispatch<C>` into, engine-seam § SEAM-D8), **not** a method set on `GameWorld`.
`&self` on the `Copy` wrapper satisfies SEAM-D8's frozen signature with **no trait
change** (engine-seam.md is cross-ref-amended to name `WorldPtr`, § SEAM-D8). Each
per-call `Dispatch<C>` impl body does the STATE-D6 **leaf `&mut *self.0` reborrows**
internally, dropping any live `&mut` before a `trap::` call that can re-enter `vmMain`
— so a mutating command (`GAME_RUN_FRAME`, `GAME_CLIENT_*`) mutates `level`/`entities`
without `RefCell` or effect queues (STATE-D1 upheld). Frozen shape + impl pattern in
§ `WorldPtr`. *Because* an immutable `&self` on `GameWorld` cannot express per-call
mutation and STATE-D1 forbids the interior-mutability escapes, while a `Copy` pointer
wrapper carries the mutation capability through `&self` and matches the single
`*mut GameWorld` the shell already threads. *Out of scope here (engine-seam SEAM-Q12):*
how a `Dispatch<C>` body issuing *outbound* traps mid-dispatch reaches the `&Engine`
those traps need — `WorldPtr` carries only the world. *Rejected:* implementing
`Dispatch<C>` on `GameWorld` (immutable `&self` can't mutate); a mutating-only router
that bypasses `Dispatch<C>` (forks the seam into two dispatch shapes for no gain).

**STATE-D9 — Zeroed heap construction via one `zeroed_box` helper (resolves STATE-Q6;
session 2026-07-02).** The `GameWorld` boxes (`Box<[gentity_t; MAX_GENTITIES]>` ≈1.83
MB, `Box<[gclient_t; MAX_CLIENTS]>`, plus `level_locals_t`) are all `#[repr(C)]`,
non-`Default`, all-zeroes-valid ABI structs Raven `memset`s in place; Rust heap-boxes
them, so the zeroed behavior needs a heap idiom C never had. The sanctioned one is a
single helper **`pub fn zeroed_box<T>() -> Box<T>`** (`alloc_zeroed` + `Box::from_raw`)
in **`crates/native/platform`** (package `native_platform`) — Raven-free native-tier
vocabulary, the Rust mirror of C static zero-initialization, allocating **directly on
the heap** so the ~1.83 MB array never transits the stack (naive stack-build-then-box
risks overflow on constrained-stack targets, e.g. 1 MB default Windows threads). It
needs **no tier-rule change and no new crate** (`native/*` is usable by everything
cross-mode, workspace-architecture § tier definitions), but it **does require a new
direct Cargo dependency edge `mp_game -> native_platform`** — currently absent
(`mp_game` depends on `mp_qshared`/`mp_bg`/`mp_abi` only, and `mp_qshared` does not
re-export `native_platform`, so the edge is not transitive); adding it is a slice-0
wiring task. The `unsafe` and its safety contract (`T` is `#[repr(C)]`,
all-zero bit pattern valid) are stated **once**, at the helper; `GameWorld::zeroed`
(§ `zeroed_box` / `GameWorld::zeroed`) builds the three via it, then wires
`level.gentities`/`level.clients` per the existing allocate-first construction-order
note — all safe above the helper. This settles the construction idiom, its §D11
licensing (confined to the one native-tier helper, not spread to the trap seam), and
whether the ABI types gain a zeroing constructor (they do, via the shared helper, not
per-type `Default`). *Because* the port needs a zeroed heap value with no oracle idiom
to copy and no in-tree `Box<[T; N]>`/`MaybeUninit`/`alloc_zeroed` precedent, and one
contract-carrying helper keeps the `unsafe` singular. *Rejected:* stack-building the
array then boxing (stack-overflow risk); a per-type `Default`/`zeroed` method
(duplicates the same `unsafe` at every large ABI type instead of one native helper).

**STATE-D10 — `ModuleRegistry` attaches at `Common.modules` (session 2026-07-02;
shared with lifecycle.md LIFE-Q5).** The `ModuleRegistry` that supersedes Raven's
`vmTable[3]`/`gvm`/`cgvm`/`uivm` is a **field of `Common` — `Engine.common.modules`**,
because Raven's `vmTable` is a **qcommon-subsystem file-scope static** (`vm.cpp:28-29`, in
`vm.cpp`, a qcommon TU), so its Rust owner is a field of the qcommon-tier `Common`, not
a fresh top-level `Engine` field. `Engine`'s frozen five-field shape
(`common`/`sv`/`cl`/`cm`/`snd`) is **unchanged** — the registry hangs off `common`. It
is constructed **empty** at `Com_Init` step 30 (`VM_Init`; lifecycle.md § Com_Init).
The registry's internal slot shape, per-slot host state, the `ModuleSlot.engine`
trampoline cell (SEAM-D11), and the `ModuleTransport` variants remain engine-seam's /
module-loading.md's — only the *container's attachment point* is fixed here. Master
row updated in the qcommon-`Common` block. *Because* lifecycle.md's step-30 registry had
no cited home (LIFE-Q5) and `Engine`'s shape is frozen, while Raven already scopes the
table to qcommon. *Rejected:* a sixth `Engine.<field>` (widens the frozen `Engine`
shape for a subsystem Raven already nests under qcommon).

**STATE-D11 — `com_printf` state lives in `Common`; `com_printf` in `mp_engine_qcommon`
(session 2026-07-02; full record in lifecycle.md).** `com_printf(common: &mut Common,
msg)` is defined in **`mp_engine_qcommon`** — one tier below the facade, the mirror of
STATE-D7's `com_error` relocation — so the 400-plus lower-tier print call sites in
server/qcommon can reach it (a `core`-hosted `com_printf` they cannot name). `Common`
owns the print-adjacent state it mutates: the **rcon redirect** fields
(`rd_buffer`/`rd_buffersize`/`rd_flush`, `common.cpp:92-94`, `Common.redirect`) and the
**logfile** sink (`logfile`/`debuglogfile` + `com_logfile` cvar, `common.cpp:32-33,50`,
`Common.logfile`) — both rowed in the qcommon-`Common` block. The **full `com_printf`
decision record** (signature, threading, console-target enumeration) is
**lifecycle.md's** (§ Seam, `com_printf`) — this doc records only the state-ownership
side (which fields `Common` owns, and that the function's crate home is
`mp_engine_qcommon`), and cites lifecycle.md for the rest. *Because* `com_printf` is
Raven's most-pervasive lower-tier primitive and its print target is `common.cpp`-owned
state, so both must sit where the leaf callers reach them — the identical layering
constraint STATE-D7 resolved for `com_error`. *Rejected:* `com_printf` in the `core`
facade (the 400-plus lower-tier callers cannot name it — the lifecycle.md-flagged
layering contradiction this record fixes).

## Verification strategy

Per DEC-09, this doc owns **no wire behavior of its own** — it fixes ownership
and threading, whose correctness is a compile-time + review property, and defers
behavioral parity to the per-subsystem docs' harnesses:

1. **No `static mut` / no ambient globals** — enforced by review **and a standing
   CI grep gate**: `grep -rn "static mut" crates/{mp,sp}/{game,cgame,ui,engine}`
   must stay empty (it is today, dossier §5); a companion gate rejects new
   non-`const` `static` in those trees outside the **two** sanctioned seam-shell
   exemptions — engine-seam's outbound `ENGINE: OnceLock<CEngine>` (SEAM-D1) and
   the module shell's `WORLD: WorldCell` (STATE-D6, which widened SEAM-D1's "one
   static" to two module-shell cells). This is the mechanical proof of §B3.
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

- **Slice 0 (MP dedicated boot) exercises only the module island — hosted inside
  a real/OpenJK engine, our own `Engine`/`Server`/native `SharedGameData` impl are
  NOT built here.** Slice 0's transport and hosting are settled by **engine-seam
  SEAM-D7**, which state-ownership defers seam Slice 0 scope to (§ Standing context;
  engine-seam § Slice hooks): `jampgame` boots as a real `NativeDll` cdylib hosted
  by a real/OpenJK engine, so the **engine side is the host's C code**. `jampgame`'s
  `GAME_INIT` builds the `GameWorld` **into the `WORLD` cell** (STATE-D6) and *emits*
  **both** outbound registrations — `trap_LocateGameData` (`g_main.c:997`) and
  `trap_SV_RegisterSharedMemory` (`g_main.c:920`) — but their **engine-side
  receivers** (`SV_LocateGameData` / the `G_SET_SHARED_BUFFER` handler storing
  `VMA(1)` and returning `0`, `sv_game.cpp:327-335,940`) are the **real host's** C,
  **not** an `Engine`/`Server.sv`-hosted `SharedGameData` impl of ours (SEAM-D7): the
  module only emits and trusts the host handler. The same `GAME_INIT` arm also emits
  `trap_Cvar_Register` (`g_main.c:809,931`; abi-traps.md row 6) for the game cvar
  table and reads back its own `g_maxclients` `vmCvar_t` (`g_main.c:111,249`) to source
  `level.maxclients` — the count that bounds the `entities[i].client` back-pointer
  wiring and feeds `level.num_entities = MAX_CLIENTS` (§ `GameWorld` construction-order
  bullet); the host services that trap too, so no `Common.cvars` of ours exists in this
  slice. Building our own native
  `SharedGameData` engine-side impl — and therefore constructing `Engine`/`Server`
  at all — is the later our-engine-hosting slice's work (below), the same boundary
  as the SEAM-D11 engine-side trampoline. The subsequent `GAME_RUN_FRAME` re-derives
  a raw `*mut GameWorld` (as `WorldPtr`) from that same cell and threads it through
  dispatch, reborrowing `&mut GameWorld` only at leaf accesses (STATE-D6 access
  discipline, amended 2026-07-02). The two module-island skeleton mechanics Slice 0
  exercises are now **unblocked**: GAME_INIT's zeroed-`GameWorld` construction
  (STATE-D9, `GameWorld::zeroed`) and GAME_RUN_FRAME's mutating dispatch via
  `impl Dispatch<C> for WorldPtr` (STATE-D8) — both frozen here, so Slice 0's
  `vmMain` skeleton (§ `WorldCell`) compiles as written.
  **Needs frozen here (and are):** the `GameWorld`/`WorldCell`/`WorldPtr` shapes,
  `GameWorld::zeroed`/`zeroed_box`, `EntityId`,
  and the `ComError` payload. The `Engine`/`Server` shapes and the `SharedGameData`
  **trait** are also frozen here, but Slice 0 does **not** *instantiate* them (that
  is the later our-engine-hosting slice). Depends on engine-seam.md for the
  module-side `Dispatch<C>` surface emitting these registrations (SEAM Slice 0) and
  on lifecycle.md for construction *order* and `ErrorLevel` variants. `com_error` is
  receiverless (STATE-D7), so it needs nothing frozen here for the module
  first-frame skeleton, which raises no error anyway (and our engine's
  `com_frame`/`com_shutdown` loop is not run in this host-inside slice).
- **Later slices.** The **our-engine-hosting slice** (SEAM-D7, after the engine
  service traits exist) is where `Engine::new` (in `mp_engine_core`, from the thin
  `mp/app` bin shell, STATE-D5) first builds the full owned `Engine` — `common:
  Common`, `cm: CollisionWorld` (zero/Default until the first `CM_LoadMap` populates
  it, mirroring `cmg`'s static zero-init, `cm_load.cpp:37`), `sv: Some(Server)`
  **hosting the `NativeDll` `SharedGameData` engine-side impl** that now receives the
  module's registrations (native contract: cache base+stride / store buffer, return
  `0`), `cl: None`, `snd: None` on dedicated — and where `com_frame`/`com_shutdown`
  run and the SEAM-D11 engine-side syscall trampoline lands (both deferred out of
  Slice 0 by SEAM-D7); it also needs lifecycle.md's `Engine::new`/`com_init`
  command-line-parameter split (§ Seam). MP client then adds `cl: Some(Client)`
  (+ `KeyState`, console) and `snd`; SP adds its `GameWorld` (no `clients`) and the
  `GetGameAPI` table register (SEAM-D2); the WASM `SharedGameData` re-resolving impl
  lands after native parity (DEC-05.5). Multi-world retrofit (STATE-D2) is a
  post-parity slice — the value-typed, registration-keyed design makes it
  non-structural.

## Open questions

The survey's own forks (dossier § Design forks 1–7) are all settled by
STATE-D1..D11 or the cited ledger entries (DEC-04/05/07/08). The former
structural gaps — the defining crate for `Engine` (was STATE-Q1), where the
owned `GameWorld` lives across `vmMain` calls (was STATE-Q3), `com_error`'s
recovery reachability + receiver (was STATE-Q4), the mutating-command `Dispatch<C>`
receiver (was STATE-Q5), and the zeroed-`GameWorld` construction idiom (was STATE-Q6)
— are resolved by STATE-D5, STATE-D6, STATE-D7, **STATE-D8, and STATE-D9**
respectively (escalation sessions 2026-07-02). The cross-doc dependencies — the seam
dispatchers and the outbound-trap `&Engine` access (engine-seam.md, SEAM-Q12), and the
`ErrorLevel` enum plus construction order (lifecycle.md, LIFE-D3) — are scoped
non-goals owned elsewhere, not unresolved decisions *of this doc*; but their owning
docs are still DRAFT, which is itself an open item (STATE-Q7). **Three** questions
remain and go back to a design session — none is self-resolvable here:

- **STATE-Q2** (below) — the four §F subcrates' `Engine`-island attachment;
- **STATE-Q7** (below) — the sequencing gate: this doc's "FROZEN HERE" blocks
  rest on sibling docs still at DRAFT;
- **STATE-Q8** (below) — the `Engine` name collision across `mp_engine_core` and
  `mp_engine_select`.

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

- **STATE-Q7 — Freeze-ordering: this doc's "FROZEN HERE" guarantees rest on
  sibling docs still at DRAFT.** *Owner: the architecture-doc-set freeze
  ordering (a process decision, not an oracle-derivable one).* Several frozen
  blocks here cite IDs whose defining docs are not yet REVIEWED/FROZEN:
  engine-seam.md (**DRAFT**) owns `Dispatch<C>`/`InboundVmCall`/the marker types
  (SEAM-D8), the wire decoding (SEAM-D6), the `mp_engine_select::Engine` alias
  (SEAM-D13), the `&Engine` outbound channel (SEAM-Q12), and the `ServerGame`
  concrete shape (forward-declared); lifecycle.md (**DRAFT**) owns `ErrorLevel`
  (LIFE-D3), the `Engine::new`/`com_init` command-line split, and construction
  order. state-ownership.md cannot advance past REVIEWED → FROZEN until those
  siblings freeze the IDs it leans on (or the cited IDs individually freeze).
  This also surfaces **cross-doc amendments not yet absorbed by their targets**
  — e.g. STATE-D11 relocates `com_printf` to `mp_engine_qcommon` (`&mut Common`)
  while lifecycle.md still renders it in `mp_engine_core` (`&mut Engine`); STATE-D7
  amends workspace-architecture.md's engine-tier line; STATE-D10 amends
  lifecycle.md LIFE-Q5. Which spelling wins in each pair is settled when the docs
  jointly freeze, not here. Not answerable from oracle ground truth — a
  documentation-sequencing decision for the design session.

- **STATE-Q8 — Two distinct types are both named `Engine`.** *Owner: a joint
  naming decision across this doc and engine-seam.md.* `mp_engine_core::Engine`
  (this doc — the owned engine-island aggregate) and `mp_engine_select::Engine`
  (engine-seam SEAM-D13 — the cfg'd module-side outbound-transport alias
  `CEngine`/`Static`/wasm) share the bare name in different crates for different
  purposes, with no cross-reference between the two docs before this round. The
  distinction is now documented inline (§ Engine) and the SEAM-Q12 `&Engine`
  ambiguity resolved to the module-side alias (crate-graph-derived, not a new
  decision). **Left open:** whether to *rename* one of them to remove the
  collision outright — a naming choice this agent cannot make; it goes back to a
  session. Until then, every reference to `Engine` must be crate-qualified.

*Resolved this round (2026-07-02):* **STATE-Q5** (the mutating-command `Dispatch<C>`
receiver) is closed by **STATE-D8** — `Self` is the `Copy` `WorldPtr(*mut GameWorld)`
wrapper, bodies leaf-reborrow `&mut` (§ `WorldPtr`). **STATE-Q6** (the zeroed-heap
construction idiom + §D11 licensing) is closed by **STATE-D9** — the one
`zeroed_box` helper in `native_platform`, used by `GameWorld::zeroed`
(§ `zeroed_box` / `GameWorld::zeroed`).
