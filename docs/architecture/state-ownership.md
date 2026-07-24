# State Ownership Design
Status: FROZEN (user sign-off 2026-07-03)     Supersedes: none

> **Amendment note (2026-07-24):** the frozen design below remains the
> baseline; the 2026-07 idiom-era campaigns refined its concrete string/
> ownership details on top of it — gentity strings are owned `String`s with
> the level-lifetime `prefixStrings` arena for engine-visible slots (#13),
> `native/*` holds the canonical C-runtime homes (DEC-32/34), model blocks
> cross the host seam as typed views (DEC-35), and the index-fn strap cells
> are gone (#19). Current state: `docs/decisions.md` + `docs/plans/`.
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
  duplication), DEC-05 (module transport `NativeDll | Static`), DEC-07 (SP
  cgame/ui statically linked via the vmachine shim),
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
  `ModuleTransport` = `enum { NativeDll, Static }` (engine-seam
  § Engine-side dispatchers), the `ServerGame` dispatcher arg (engine-seam
  § dispatchers — the reborrowed `Engine.sv` host state), and `SharedGameData`
  (whose *method set* this doc freezes below). The frozen blocks below
  additionally rest on these engine-seam IDs — **cited, not restated** (rule 4):
  **SEAM-D8** (the `Dispatch<C: InboundVmCall>` trait `fn dispatch(&self, args:
  C::Args) -> C::Output`, `InboundVmCall`'s `Args`/`Output` associated surface,
  and the per-command marker structs — the CamelCase `InboundVmCall` markers in `mp/abi`
  (e.g. `GameInit`), each carrying its SCREAMING_SNAKE `MpGameExport` variant (e.g.
  `GAME_INIT`) as `COMMAND`; distinct types, **not** the enum variants themselves); the
  `impl Dispatch<GameRunFrame> for GameContext` block in § `GameContext` is written
  against that trait, defined there. **SEAM-D6** (the `c_int`↔enum decoding the
  `dispatch(command: c_int, …)` router uses, and the `GAME_INIT`/`GAME_SHUTDOWN`/
  `GAME_RUN_FRAME` command constants the § `WorldCell` skeleton branches on).
  **SEAM-D13** (the module-side `pub type Engine` alias in `mp_engine_select` —
  a **different** type from this doc's engine-island `pub struct Engine`; the two
  are disambiguated in § Engine and STATE-Q8). **SEAM-Q12** (the `&Engine` channel
  by which a `Dispatch<C>` body issues *outbound* traps mid-dispatch — **resolved
  this round**: the `GameContext` receiver carries `engine: &Engine`, STATE-D8
  amendment 2026-07-03, § `GameContext`; engine-seam.md absorbs the receiver-name
  change as a cross-doc amendment, STATE-Q7). The `ServerGame`
  dispatcher-arg's **concrete shape** (type alias vs. wrapper struct) is only
  *forward-declared* in engine-seam.md, not yet pinned there (§ Engine).
  engine-seam.md is **Status: FROZEN** (user sign-off 2026-07-03, whole-set freeze);
  the STATE-Q7 freeze-ordering gate is satisfied.
- `docs/abi-traps.md` — the generated `trap_*` signature reference; the seam
  below stores the register-once payloads of rows 18 (`trap_LocateGameData`)
  and 121 (`trap_SV_RegisterSharedMemory`), and references row 19
  (`trap_DropClient`) only as the STATE-D6 reentrancy trigger (not stored).
  Rows 6/18/121 are the exact signatures the Slice-0 `GAME_INIT` outbound-call
  arm emits; they live in abi-traps.md (a signature reference, linked not
  restated per rule 4).
- `docs/architecture/lifecycle.md` — **Status: FROZEN** (user sign-off 2026-07-03).
  Load-bearing co-reading: it
  owns the `ErrorLevel` enum (LIFE-D3 = the per-mode `errorParm_t` — MP
  `errorParm_t`, SP's 4-variant set — this doc's `ComError.level` type), the
  `Engine::new` / `com_init` command-line split (LIFE-D4b; `com_init(&mut engine,
  cmdline)` carries the raw command line, `Engine::new()` does not), the boot
  construction *order*, `com_printf` (LIFE-Q2 — STATE-D11's `&mut Common` /
  `mp_engine_qcommon` relocation is confirmed the winner this round and lifecycle.md
  is being amended to match, STATE-D3/STATE-Q7 2026-07-03), and the module-registry
  step-30 attachment shared as LIFE-Q5/STATE-D10. With the 2026-07-03 whole-set
  freeze, the variants/order/split it owns are reachable as frozen facts.
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
the module-island `Dispatch<C>` receiver (`GameContext`, STATE-D8, superseding the
prior `WorldPtr`); the **SP access discipline** (STATE-D12 — `GetGameAPI`
fn-pointer table, no `vmMain`/`Dispatch<C>`; `ge->Init`/`ge->Shutdown` world
lifetime; per-export `GameContext` construction); the reentrancy
contract (reborrow-threading + `EntityId` discipline, no `RefCell`, no effect
queues); and the error-recovery contract (`com_error` is a receiverless
format+panic in `mp_engine_qcommon`; all engine recovery runs catch-side in
`mp_engine_core`, STATE-D7). It **freezes `SharedGameData`** (deferred here by
engine-seam) and records the `Common.modules` module-table attachment (STATE-D10)
and `com_printf`'s `Common`-owned print state (STATE-D11).

**Non-goals** (each punted to its owning doc):

- **Seam executor mechanics** — how a trap crosses (`Execute<C>`/`Dispatch<C>`,
  the syscall/table wires) → `docs/architecture/engine-seam.md`.
- **Per-subsystem internals** — what each owned struct's fields are and what each
  handler *does* (cvar parse tables, filesystem search-path logic, sound mixer,
  collision internals) → `docs/subsystems/*` (pending).
- **Lifecycle / boot & shutdown sequences** — construction *order*, headless
  boot, the `ErrorLevel` enum variants (LIFE-D3), and the `Engine::new`/`com_init`
  command-line split → `docs/architecture/lifecycle.md` (**Status: FROZEN**,
  2026-07-03 — it freezes them, not this doc). This doc references `ErrorLevel` and the
  split by ID only.
- **Renderer globals** (`tr`/`backEnd`/`glConfig`) — deferred wholesale per
  DEC-01; the master table lists them pointing at that deferral, nothing more.

## Raven ground truth

The full ~60-row census lives in the dossier; § State ownership's master table is
this doc's rendering of it. This section covers only the *architecture* the table
rests on. MP tree = `oracle/codemp/`, SP tree = `oracle/code/`.

### Build-canonical file variants

The real PC build links `cmd_common.cpp`+`cmd_pc.cpp`,
`files_common.cpp`+`files_pc.cpp`, `z_memman_pc.cpp`
(`oracle/codemp/unix/makefile:302-307`). `files.cpp`, `cmd_console.cpp`,
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
access discipline are frozen in § Seam definition. On MP the shell's `vmMain`
reads/writes the cell; **SP `jagame` has no `vmMain`** — its `game_export_t` fns
(`ge->Init`/`ge->Shutdown`) write/take the cell and each export re-derives its own
pointer (STATE-D12). Both statics are
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

**Server (`mp/engine/server::Server`, `Engine.sv: Server`)** — not an `Option`;
liveness is `sv.state == SS_DEAD` (`server.h:47-54`; user ruling item 20, 2026-07-03)

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
| botlib / ghoul2 / icarus / rmg engine-side globals | dossier §1 (not censused; readers only, e.g. `icarus/GameInterface.cpp:129`) | direct `Engine` fields (`engine.icarus`/`nav`/`g2`/`rmg`/`roff` — STATE-Q2 CLOSED per rulings 12/13/43; internal shapes per the §F docs `docs/subsystems/{icarus,rmg-terrain,ghoul2-server,npcnav,roff}.md`) | their `*_Init` (per §F doc), fields land with each subsystem's waves | `EngineHostView` split-borrow (ruling 43) |

**Module island — MP game (`mp/game::GameWorld`)**

| Raven global | oracle cite | Rust owner | constructed by | threaded via |
|---|---|---|---|---|
| `level` (`level_locals_t`) | `g_main.c:9` | `GameWorld.level: mp_game::level::level_locals_t` | `G_InitGame` | `(world, id)` re-borrow |
| `g_entities[MAX_GENTITIES]` | `g_main.c:27` | `GameWorld.g_entities: Box<[mp_qshared::common::mp::gentity_t; MAX_GENTITIES]>` (contiguous `#[repr(C)]`, size-asserted 1832B — reuse the existing type, §D12) | `G_InitGame` | `EntityId(u32)` index |
| `MAX_GENTITIES` (sizes the array above) | `q_shared.h:1996` | **`mp_qshared`** const per its oracle home + workspace-architecture tier table (Tier 0 qshared) — GameWorld needs it, so it **now also lives in `mp_qshared`**; the existing `mp_engine_server` copy (`crates/mp/engine/server/src/server/server_t.rs:21`) **and its size-asserts stand**. The dedupe (server re-importing the `mp_qshared` const) is a **deferred mechanical sweep** (skeleton finding 5, 2026-07-03), no behavioral change — the value is identical | (compile-time const) | referenced, not threaded |
| `g_clients[MAX_CLIENTS]` (reached as `level.clients`) | `g_main.c:28` | `GameWorld.clients: Box<[mp_game::client::gclient_t; MAX_CLIENTS]>` (`gclient_t` = `gclient_s`, asserted 7344B; **MP only**) | `G_InitGame` | index |
| `trap_LocateGameData` registration | `g_main.c:997` | registers `GameWorld` base+stride into the engine's `SharedGameData` (§Seam) | `G_InitGame` | raw seam (§D11) |

**Module island — SP game (`sp/game::GameWorld`)**

| Raven global | oracle cite | Rust owner | constructed by | threaded via |
|---|---|---|---|---|
| `level`/`g_entities` | `code/game/g_main.cpp:46,49` | `GameWorld.level` / `GameWorld.entities` — SP `WORLD` cell written by `ge->Init`, taken by `ge->Shutdown` (STATE-D12) | `InitGame` | `EntityId` index; per-export `GameContext` (STATE-D12) |
| `g_clients` | — | **does not exist** (`MAX_CLIENTS = 1`) — divergence preserved (STATE-D2) | — | — |
| `globals` (`game_export_t`) | `code/game/g_main.cpp:48` | SP seam handle (engine-seam SEAM-D2); `globals.gentities` = base register | `GetGameAPI`/`InitGame` | raw seam (§D11) |
| `gi` (`game_import_t`, the stored engine handle) | `code/game/g_main.cpp:47`, extern `g_shared.h:830` | SP module-side engine handle = the `game_import_t` `gi = *import` from `GetGameAPI` (`g_main.cpp:878`) — SP mirror of `mp_engine_select::Engine` (STATE-D12; Rust alias name/crate = **STATE-Q9**, SP slice) | `GetGameAPI` | paired into the per-export SP `GameContext` mirror; outbound calls `gi.X(...)` |

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
// server+client). File: `engine.rs` in mp_engine_core (one-type-per-file rule;
// pinned 2026-07-03, skeleton checkpoint 6 conforms; lifecycle.md § Seam dual).
// One value, allocated as `Box<Engine>` by
// `Engine::new() -> Box<Engine>` called from the thin `mp/app` bin shell (STATE-D5;
// LIFE-D2 amended in lifecycle.md): a boxed ZEROED heap buffer whose few non-zero-valid
// fields (currently `Common.time_base: std::time::Instant` — unspecified layout, no
// all-zero validity) are then initialized IN PLACE before the `Box` is exposed (the
// `MaybeUninit` pattern). There is deliberately NO `unsafe impl ZeroValid for Engine`
// (the aggregate is not all-zeroes-valid — checkpoint-5 finding 21); the `ZeroValid`
// impls cover only the `#[repr(C)]` constituents (`server_t`/`clipMap_t`/…, STATE-D9/D13).
// Threaded `&mut` DOWN call chains (STATE-D1). No field is ever a `static`/global.
// `mp_engine_core`
// also hosts com_init/com_frame/com_shutdown (they must reach the server+client
// crates); com_error itself lives one tier lower in mp_engine_qcommon (STATE-D7),
// its recovery run by com_frame/com_init's catch. Field *sub*-structs keep their
// own subcrate homes below.
//
// NAMING (STATE-D2 disambiguation, STATE-Q8; session 2026-07-03, no renames): THIS is
// `mp_engine_core::Engine`, the engine-island aggregate — a DIFFERENT type from
// `mp_engine_select::Engine` (the module-side transport alias the `GameContext.engine`
// receiver field holds, SEAM-D13). Opposite islands, never co-scoped; crate-qualify
// every ambiguous `Engine`. Canonical disambiguation block: workspace-architecture.md
// (§ crate tiers, 2026-07-03). STATE-Q8 is CLOSED (round 6, 2026-07-03): keep both
// names, crate-qualify, NO renames — the disambiguation is by qualification, not rename.
pub struct Engine {
    pub common: Common,          // type from mp_engine_qcommon (cvars, cmd, cbuf, fs, net)
    pub sv: Server,              // type from mp_engine_server — NOT an `Option` (user ruling
                                 //   2026-07-03, item 20). `server_t.state: serverState_t`
                                 //   (`server.h:54`) IS the liveness flag: a zeroed `Server` has
                                 //   `sv.state == SS_DEAD` (the first, =0 enumerator "no map loaded",
                                 //   `server.h:47-51`) — the direct dual of Raven's file-scope
                                 //   `server_t sv` that the loader zero-fills, so "no server running"
                                 //   is `SS_DEAD`, not a missing `Option`. `Server` embeds
                                 //   `server_t.svEntities[1024]` (svEntity_t) + more large-by-value
                                 //   state; it is zeroed as part of the WHOLE-Engine boxed-zeroed
                                 //   allocation (`Engine::new() -> Box<Engine>`; `server_t: ZeroValid`
                                 //   covers this member), never stack-then-move `Default`. Non-zero
                                 //   init (`SV_Init`/`SV_SpawnServer`) runs in `com_init` exactly where
                                 //   Raven runs it (STATE-D13 amendment; construction: lifecycle.md).
                                 //   PRESENCE IDIOM (checkpoint-5 finding 22): `sv` is always-present /
                                 //   state-gated (`sv.state`, Raven's zero-filled-static dual), whereas
                                 //   `cl`/`snd` below stay `Option`-gated until the client-side pass
                                 //   (owner: client slice) — the mixed idiom is deliberate, not unified.
    pub cl: Option<Client>,      // type from mp_engine_client — Some on client builds; None on dedicated
    pub cm: CollisionWorld,      // type from mp_engine_qcommon — cmg + SubBSP, instance-shaped value.
                                 //   NOT an Option: it is built zeroed as part of the same whole-Engine
                                 //   ZeroValid allocation (mirroring Raven's static zero-init of
                                 //   file-scope `clipMap_t cmg`, `cm_load.cpp:37`; §A2 faithful),
                                 //   present-but-empty before any map; CM_LoadMap then POPULATES it in
                                 //   place (in `com_init`/map load, not a second construction). The
                                 //   master table's "constructed by CM_LoadMap" means populate-in-place,
                                 //   not first existence — a zeroed CollisionWorld exists from
                                 //   `Engine::new`.
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

*Amendment (2026-07-03, user ruling item 20 — `sv` de-`Option`ed; whole-Engine zeroed
allocation).* The frozen `sv` field was `Option<Server>` and `Engine::new` built the
`Some(Server)`/`cm` members via *per-member* `zeroed_box` calls (the round-6 STATE-D13
/ LIFE-D5 mechanics). Both are **superseded** by the user ruling: (1) `sv: Server` —
no `Option`; server liveness is `sv.state == SS_DEAD` (`serverState_t`, `server.h:47-51`;
field `server.h:54`), the direct dual of Raven's loader-zero-filled file-scope `server_t
sv` static, where "no map loaded" is `SS_DEAD` (=0), not an absent value. (2)
`Engine::new() -> Box<Engine>` allocates the **whole aggregate** as a boxed ZEROED heap
buffer and initializes its few non-zero-valid fields (currently `Common.time_base:
std::time::Instant`) **in place** before exposing the `Box` — the `MaybeUninit` pattern.
There is deliberately **no `unsafe impl ZeroValid for Engine`** (the aggregate is not
all-zeroes-valid — checkpoint-5 finding 21); `ZeroValid` covers only the `#[repr(C)]`
constituents (`server_t`/`clipMap_t`/… — STATE-D9/D13). No member is built stack-then-move
and none transits the stack; non-zero init (`SV_Init`/`CM_LoadMap`/…) happens later in
`com_init`, exactly where Raven runs it. This **dissolves** STATE-D13's per-member
`zeroed_box`/stack-ordering residual and LIFE-Q7's wrapper-assembly residual entirely (one
zeroed allocation, no field-by-field ordering question) — recorded at STATE-D13 and
lifecycle.md LIFE-Q7 (now CLOSED). `cl`/`snd` stay `Option` (genuine dedicated-vs-client
build presence, `None` = the zero value; presence-idiom rule, checkpoint-5 finding 22 —
owner of the client-side pass: the client slice).

The game dispatcher's `engine: &mut ServerGame` parameter (engine-seam,
`sv_game_system_calls`) is the **server-island reborrow** — `&mut Engine.sv`'s
`Server`, carrying its `SharedGameData` registration. `ServerGame` is
engine-seam's name for exactly that reborrowed host state; the two names denote
the same value (kept consistent per that doc). Its **concrete shape** (a
`type ServerGame = Server` alias vs. a wrapper struct) is a seam-executor
mechanic (§ Non-goals) that engine-seam.md only *forward-declares*; it is not
pinned in either doc yet, and rides the same not-yet-FROZEN-sibling gate as the
other SEAM IDs (engine-seam.md is REVIEWED, not FROZEN; STATE-Q7). This doc fixes
only *which value* `ServerGame` denotes.

*Amendment (2026-07-05, user ruling): concrete shape pinned —
`pub type ServerGame = Server;` (plain alias, not a wrapper struct;
`crates/mp/engine/server/src/server_host.rs`). STATE-Q7 residual CLOSED.*

**Two distinct types are both named `Engine` — disambiguation (STATE-Q8).** This
doc's `pub struct Engine` (crate `mp_engine_core`) is the **engine-island
aggregate** (`common`/`sv`/`cl`/`cm`/`snd`). It is a **different type** from
engine-seam SEAM-D13's `pub type Engine` (crate `mp_engine_select`), which is the
**module-side outbound-transport alias** (`CEngine`/`Static`) that logic
crates thread as `&Engine` to *issue* traps. Same bare name, different crates,
different purpose. Consequently, the `&Engine` a `Dispatch<C>` body needs to issue
outbound traps mid-dispatch (the former **SEAM-Q12**) is the **module-side
`mp_engine_select::Engine`**, carried by the `GameContext` receiver's `engine`
field (STATE-D8 amendment 2026-07-03, § `GameContext`) — **not** this struct:
`mp_game` (where the `Dispatch<C>` impls live, STATE-D8) depends on
`mp_qshared`/`mp_bg`/`mp_abi`/**`mp_engine_select`** (workspace-architecture
§ Dependency edges) — the `mp_engine_select` edge is exactly what lets `GameContext`
name `&mp_engine_select::Engine` — but it can never gain an edge to the
`mp_engine_core` facade (an illegal upward edge), so `mp_engine_core::Engine` is
structurally unreachable from a `Dispatch<C>` body. SEAM-Q12 is thereby **resolved**
(the channel is `GameContext.engine`); the two `Engine` names are **kept, not renamed**
(STATE-Q8 CLOSED round 6, 2026-07-03 — disambiguate by crate-qualification).

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
lifecycle.md (**REVIEWED**) now *renders* that split — `Engine::new()` takes no
command line (it captures the timer base, LIFE-D4b), and `com_init(&mut engine,
cmdline)` carries the raw command line and runs `Com_ParseCommandLine`
(lifecycle.md § Com_Init) — but it freezes there, not here, and its not-yet-FROZEN status
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
    pub g_entities: Box<[gentity_t; MAX_GENTITIES]>,  // mp_qshared::common::mp::gentity_t
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
- **`#[repr(C)]` contiguous entity array** (not `Vec<Enum>`): the byte layout
  is exactly what the native pointer-cast reader crosses the seam through.
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
  cannot reach; the frozen struct references `mp_qshared::…::MAX_GENTITIES`, so the
  const **now also lives in `mp_qshared`** while the existing `mp_engine_server`
  copy **and its size-asserts stand** — deduping them (server re-importing the
  `mp_qshared` const) is a **deferred mechanical sweep** (skeleton finding 5,
  2026-07-03), no behavioral change since the value is identical.
- **`level`'s self-referencing pointers + client back-pointers — intra-`GameWorld`
  construction order.** (Grounded in the oracle sequence cited below,
  `g_main.c:978-994`; not a separate decision record.) The full `G_InitGame` entity
  block is a five-step faithful sequence, all in the `GAME_INIT` dispatched arm
  (§ `GameContext`), *after* `G_RegisterCvars` has run (`g_main.c:931`; provenance in
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
// mp_qshared/mp_bg/mp_abi/mp_engine_select; mp_qshared depends on native_platform but does not re-export
// it, so the edge is not transitive), so a new direct Cargo dependency edge
// mp_game -> native_platform must be added as a slice-0 wiring task (SP dual:
// sp_game -> native_platform, checkpoint-4 finding 16 — both rows land in
// workspace-architecture's dep table).
//
// `ZeroValid`: a hand-rolled UNSAFE MARKER TRAIT (bytemuck-`Zeroable` style, NO new
// dependency) — "the all-zero bit pattern is a valid value of Self" (STATE-Q10
// resolution 2026-07-03). It makes zeroed_box a SAFE fn: the per-type all-zero audit
// lives in each type's `unsafe impl`, colocated with that type's layout static-asserts
// (the same file the `offset_of!` evidence lives in), so the unsafety stays confined
// per §D11 and every call site stays safe.
// SOUNDNESS, NOT RUSTC, IS THE GATE (checkpoint-5 finding 21): a bare marker `unsafe impl
// ZeroValid for X {}` ALWAYS compiles — rustc checks nothing — so the whole audit burden
// sits on that one `unsafe impl` line (is `X` genuinely `#[repr(C)]` and all-zeroes-valid?).
// This is exactly why the aggregate `Engine` gets NO such impl: `Common.time_base:
// std::time::Instant` is not all-zero-valid, so `Engine` is zeroed via the `MaybeUninit`
// in-place-init pattern (§ Engine), never a marker impl that would silently compile.
pub unsafe trait ZeroValid {}
// Blanket for the entity/client boxes, which are ARRAYS — array types have no owning
// file for a colocated per-type impl (checkpoint-4 finding 17), so native_platform
// carries this one blanket:
unsafe impl<T: ZeroValid, const N: usize> ZeroValid for [T; N] {}
// Each #[repr(C)] all-zeroes-valid ABI type opts in with a one-line
// `unsafe impl ZeroValid for X {}` beside its layout asserts — e.g. `gentity_t`,
// `gclient_t`, `level_locals_t` (and the engine-island `server_t`/`clipMap_t`, STATE-D13).
//
// THE sanctioned construction idiom for large #[repr(C)] all-zeroes-valid types:
// alloc_zeroed the storage and Box::from_raw it, so the ~1.83 MB entity array is
// built directly on the heap and never transits the stack (naive stack-build-then-box
// risks overflow on constrained-stack targets, e.g. 1 MB default Windows threads).
// SAFE fn: the `ZeroValid` bound discharges the all-zero-valid precondition at the type
// system, so the `unsafe` is in the audited `impl`s above, never the call site.
pub fn zeroed_box<T: ZeroValid>() -> Box<T>;
```

```rust
// mp/game (SP mirror). Builds the zeroed island, then wires level's self-referencing
// back-pointers in the allocate-first order (§ GameWorld construction-order bullet).
// The gentity_t / gclient_t / level_locals_t types satisfy zeroed_box's `ZeroValid`
// bound (each a one-line `unsafe impl ZeroValid` beside its layout asserts; the arrays
// via the blanket) — they are the same all-zeroes-valid ABI structs Raven memsets.
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

The `unsafe` lives in the per-type `unsafe impl ZeroValid` audits (each beside its
type's layout asserts), **not** the call site: `zeroed_box`, `GameWorld::zeroed`, and
the `GAME_INIT` arm that calls it are all safe. This resolves the former STATE-Q6
(construction idiom, its §D11 licensing, and whether the ABI types gain a zeroing
constructor — they do, via this shared helper rather than per-type `Default`) **and**,
via the `ZeroValid` bound, the former **STATE-Q10** (the safe-generic soundness): a
`zeroed_box::<String>()` no longer compiles, because `String: !ZeroValid` — the
bytemuck-`Zeroable`-style marker moves the all-zero precondition into the type system
while keeping every call site safe (round-6 resolution 2026-07-03; STATE-D9 amendment
below, STATE-Q10 closed).

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
// exactly Raven's `level` (§B6). Identical shape in NativeDll and Static builds.
// No logic crate names it; MP's shell vmMain reads/writes it (access confined below). SP
// jagame has no vmMain — ge->Init/ge->Shutdown write/take the cell, each export re-derives
// its own pointer (STATE-D12); the cell shape and the confined-unsafe rule are identical.
static WORLD: WorldCell = WorldCell::new();

struct WorldCell(pub(crate) UnsafeCell<Option<GameWorld>>); // `pub(crate)` field: `WORLD.0.get()`
                                    // is reached from `lib.rs`, a different module from the
                                    // `world_cell.rs` this struct lives in (§ Per-file placement) —
                                    // so the tuple field must out-scope its defining module. Exactly
                                    // one visibility satisfies that cross-module same-crate access, the
                                    // identical mechanical convention module-loading.md pins `pub(crate)`
                                    // (LOAD-D12f). Settles nothing new (mechanical amendment 2026-07-03).
impl WorldCell {
    // `const fn` is forced — `WORLD` is a `static` initializer — and the body is the only
    // possible one (`UnsafeCell::new` is const, the cell starts `None`). Mechanical, no
    // design freedom; spelled here so the frozen block is self-contained.
    const fn new() -> Self { WorldCell(UnsafeCell::new(None)) }
}
// SAFETY (Sync only): the module runs single-threaded per Raven's contract, so the static is
// never touched from a second thread. Single-threaded *reentrant* aliasing is a separate hazard,
// handled by the raw-pointer threading in vmMain below — not by this impl.
unsafe impl Sync for WorldCell {}
```

Access discipline — the only `unsafe` in the shell, confined per §D11 (STATE-D6
amendment 2026-07-02, receiver updated by STATE-D8 amendment 2026-07-03: thread a
**raw `*mut GameWorld`** paired with the module-side `&Engine` into the `Copy`
`GameContext` the `Dispatch<C>` impls take by value, never a dispatch-spanning `&mut`):

```rust
// crates/jampgame shell (MP). C entrypoint takes NO context arg, so the world is reached via WORLD.
// SP jagame has NO vmMain (STATE-D12): ge->Init/ge->Shutdown write/take the cell and EACH
// game_export_t fn re-derives its own GameContext (per-export, not this once-per-vmMain funnel).
// GAME_INIT constructs the GameWorld INTO the cell and falls through to the generic dispatch
// (whose GAME_INIT arm runs G_InitGame's init logic against it); GAME_SHUTDOWN takes it back
// OUT after its dispatch returns; every other command re-derives a raw `*mut GameWorld` from
// the cell and threads it (paired with `&Engine` as `GameContext`) through dispatch. A `&mut GameWorld` is reborrowed
// only at non-overlapping leaf accesses inside the Dispatch bodies (the STATE-D1 EntityId
// discipline), always dropped before any `trap::` call that can re-enter vmMain — so no `&mut`
// ever spans a reentrant entry.
// EXPORT SIGNATURE OWNER: the exact param/return ABI words are engine-seam.md's, NOT this
// doc's (§ Non-goals — seam executor mechanics). This skeleton spells engine-seam SEAM-D9/D10's
// frozen export verbatim — `vmMain(command: AbiCommand, arg0..arg11: AbiWord) -> AbiWord`,
// where `AbiCommand = c_int` and `AbiWord = isize` (module-loading LOAD-D12e / SEAM-D4) — so a
// porter transcribes ONE return type across both docs. (Corrected 2026-07-03 from a stale
// `c_int -> c_int` here, which contradicted the `-> AbiWord` this doc already cites in the
// dispatch comment below and in engine-seam's SEAM-D9/D10 export; this doc freezes only the
// WorldCell access, not the seam signature.)
// TIE-BREAK (2026-07-03): oracle `g_main.c:515` is `int vmMain( int command, int arg0 …
// int arg11 )` — the all-`int`, 32-bit original. LOAD-D12e's `AbiWord = isize` is the settled
// 64-bit-widened dual (pointer-width trap word). The two are reconciled by making
// engine-seam.md's SEAM-D9/D10 block AUTHORITATIVE for the export signature
// (`command: AbiCommand, arg0..arg11: AbiWord -> AbiWord`); this doc renders it verbatim.
#[no_mangle]
pub extern "C-unwind" fn vmMain(command: AbiCommand, /* arg0..arg11: AbiWord */) -> AbiWord {
    // BOOTSTRAP (STATE-D6 amendment 2026-07-02). GAME_INIT is the ONE command that WRITES the
    // cell before reading it: it stores a zeroed GameWorld (GameWorld::zeroed, STATE-D9),
    // THEN falls through so the dispatched GAME_INIT arm does G_InitGame's init against it
    // (g_main.c:515,979). WORLD starts `None`, so this write-then-fallthrough is what keeps the
    // shared read below from panicking on the very first call. map_restart's fresh GAME_INIT
    // overwrites the cell with a fresh world.
    // PRE-DECODE spelling: `command` is still the raw AbiCommand (= c_int, entrypoints.rs:3)
    // — it has NOT yet gone through `MpGameExport::try_from` — so the bootstrap branch
    // compares against the `#[repr(i32)]` discriminant cast `MpGameExport::GAME_INIT as c_int`.
    // `MpGameExport` (exports.rs:10) and its wire values are engine-seam SEAM-D6's; its variant
    // is Raven's `GAME_INIT` enumerator transcribed verbatim in SCREAMING_SNAKE_CASE (g_public.h:735,
    // gameExport_t enum, first variant), kept 1:1 per the CLAUDE.md enum-fidelity rule
    // (`#![allow(non_camel_case_types)]`, exports.rs:6 — NOT renamed to CamelCase). The CamelCase
    // `GameInit` is a SEPARATE zero-sized `InboundVmCall` marker struct (`vmcalls/GAME_INIT.rs:46`,
    // `const COMMAND: MpGameExport = MpGameExport::GAME_INIT`) used as a `Dispatch<C>` type
    // parameter — it is NOT an `MpGameExport` variant, so `MpGameExport::GameInit` names nothing and
    // would not compile; only the SCREAMING_SNAKE variant is castable to `c_int`. There is NO
    // standalone c_int const to name instead — the enum discriminant IS the constant. (This
    // bare-int compare must precede the fallible decode because it is what makes the world exist;
    // the shared read below would panic otherwise.)
    // CROSS-DOC (STATE-Q7): RECONCILED 2026-07-03 — engine-seam § SEAM-D6/SEAM-D10 now carries
    // the identical `MpGameExport::GAME_INIT as c_int` spelling (the round-6 pinning (b) had
    // synced both docs to the non-compiling CamelCase; corrected in the reconciliation pass).
    // CRATE-VISIBILITY (STATE-Q13 CLOSED, user item 25, 2026-07-03): the shell reaches
    // `MpGameExport` (an `mp_abi`-defined enum, exports.rs:10, SEAM-D6) through `mp_game`'s
    // CRATE-ROOT re-export — `pub use mp_abi::game::exports::MpGameExport;` in
    // crates/mp/game/src/lib.rs — so SEAM-D10's frozen exactly-two-edges shell property
    // (`abi_transport` + `mp_game`) stays intact: the shell sees the seam THROUGH the logic
    // crate it wraps. See § Open questions STATE-Q13 (closed).
    if command == MpGameExport::GAME_INIT as c_int {
        // SAFETY: single-threaded init; no reentrancy is possible before the world exists.
        unsafe { *WORLD.0.get() = Some(GameWorld::zeroed()); } // STATE-D9 zeroed-construction idiom
    }
    // SAFETY: single-threaded per Raven's contract, and — the real hazard — single-threaded
    // REENTRANCY: the engine re-enters vmMain from inside a syscall handler (bullet below) while
    // this frame is suspended. We derive a RAW *mut (not a live &mut) and pair it with the
    // module-side &Engine (ENGINE.get(), engine-seam SEAM-D1's outbound ENGINE cell) into the Copy
    // GameContext the Dispatch<C> impls take by value, so each reentrant entry derives its OWN raw
    // pointer; aliasing raw pointers are sound, a dispatch-spanning &mut would be UB even
    // single-threaded. &mut GameWorld is reborrowed only at leaf accesses that never span a
    // trap:: re-entry (STATE-D8, amended 2026-07-03: WorldPtr -> GameContext, which also carries
    // the outbound-trap &Engine, closing the former SEAM-Q12 punt).
    let ctx = GameContext {
        // world: the raw *mut derived from WORLD once per entry.
        world: unsafe { (*WORLD.0.get()).as_mut().expect("GAME_INIT built the world") } as *mut GameWorld,
        // engine: the module-side mp_engine_select::Engine transport alias (SEAM-D13), NOT
        // mp_engine_core::Engine; registered into ENGINE (OnceLock<CEngine>, SEAM-D1) at load.
        engine: ENGINE.get().expect("ENGINE registered before first vmMain call"),
    }; // GameContext has PUB fields — the shell struct-literals it (STATE-D8, round-6 resolution 2026-07-03)
    // `dispatch` routes each command to its per-call `Dispatch<C> for GameContext` impl (engine-seam
    // SEAM-D8; receiver settled as GameContext by STATE-D8, amended 2026-07-03). The impl bodies do
    // the STATE-D8 leaf `&mut *ctx.world` reborrows, so a MUTATING command (GAME_RUN_FRAME,
    // GAME_CLIENT_*) mutates level/entities through those leaf borrows — no RefCell, no effect queue
    // (STATE-D1) — and issue outbound traps as `trap::X(ctx.engine, …)` in oracle call order.
    // `dispatch(...)` is NOT a free helper this doc freezes — it is a placeholder for
    // engine-seam.md's inbound `vmMain` export-enum `match` (SEAM-D10, the inbound dual of
    // the outbound `sv_game_system_calls` match; § Non-goals → seam executor mechanics). That
    // match is an EXHAUSTIVE INLINE match in this same shell `vmMain` — NOT a helper fn
    // (round-6 pinning (b), mirroring the outbound `sv_game_system_calls` match)
    // (engine-seam § "Concrete shell — crates/jampgame"),
    // takes the individual `arg0..arg11: AbiWord` words engine-seam's frozen export signature
    // spells (`vmMain(command: AbiCommand, arg0..arg11: AbiWord) -> AbiWord`, SEAM-D10 — NOT a
    // packed word slice), decodes each arm via `DecodeVmMain`/re-encodes via `EncodeVmMainReturn`
    // (SEAM), and routes each decoded command to its `Dispatch<C> for GameContext` impl. This doc
    // fixes ONLY the `ctx: GameContext` receiver threaded into it (STATE-D8); the match's own
    // shape/signature is engine-seam.md's, not re-frozen here.
    let result = dispatch(command, ctx, /* arg0..arg11: AbiWord, per SEAM-D10 */);
    // GAME_SHUTDOWN takes the world OUT of the cell AFTER its dispatch returns — module-unload
    // lifetime; dropping the Some(GameWorld) runs the owned island's Drop (§C9). STATE-D6 amend.
    if command == MpGameExport::GAME_SHUTDOWN as c_int { // same pre-decode c_int spelling as GAME_INIT above
        // SAFETY: single-threaded; the just-returned GAME_SHUTDOWN dispatch holds no live borrow.
        unsafe { *WORLD.0.get() = None; }
    }
    result
}
```

- **Reentrancy is by design, not a hazard** (STATE-D6, amended 2026-07-02). The
  engine re-enters `vmMain` from inside a syscall handler — `G_DROP_CLIENT`
  (`sv_game.cpp:569`) → `SV_GameDropClient` (`sv_game.cpp:110-114`) →
  `SV_DropClient` (`sv_client.cpp:580`) → `VM_Call(gvm, GAME_CLIENT_DISCONNECT)`
  (`sv_client.cpp:640`), a fresh `vmMain` entry while the outer one is suspended.
  Each entry derives its **own** `*mut GameWorld` (a fresh `GameContext`) from the cell;
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
  receiver (was STATE-Q5 → STATE-D8, `GameContext` below — `WorldPtr` superseded 2026-07-03).

### `GameContext` — the `Dispatch<C>` receiver (STATE-D8, FROZEN HERE)

*Amendment (2026-07-03 — supersedes `WorldPtr`, resolves SEAM-Q12): the receiver is
`GameContext<'e>`, superseding the single-field `WorldPtr(*mut GameWorld)` of the
2026-07-02 amendment. It adds one field — `engine: &'e Engine` — so the receiver
itself carries the outbound-trap channel a `Dispatch<C>` body needs, closing the
former **SEAM-Q12** punt (below). `Copy`-ness, the `&self`-satisfies-SEAM-D8 property,
and the STATE-D6 leaf-reborrow discipline are all unchanged — only the field set grew.
The fields are **`pub` and the shell struct-literals the value** (round-6 resolution
2026-07-03 — the interim private-fields/`pub fn new` detour is struck; engine-seam.md's
matching 'private + new()' text is superseded and removed the same round, so both docs
read identically). engine-seam.md absorbs the receiver-name change as a cross-doc
amendment (§ SEAM-D8; tracked by STATE-Q7).*

engine-seam froze the `Dispatch<C: InboundVmCall>` trait and its
`fn dispatch(&self, args: C::Args) -> C::Output` (SEAM-D8) and punted **which
`Self` each `Dispatch<C>` impl reads/writes** to this doc. The trait, the
`InboundVmCall` bound (its `Args`/`Output` associated types), and the concrete
per-command marker structs (`GameRunFrame`/`GameInit`/`GameShutdown` — the CamelCase
`InboundVmCall` markers in `mp/abi`, each carrying its SCREAMING_SNAKE `MpGameExport`
variant as `COMMAND`, **not** themselves enum variants, SEAM-D6) are engine-seam's, read
there and not restated here (rule 4); the impl block below is written against them.
**Visibility path (STATE-Q12 CLOSED, user item 24, 2026-07-03):** `mp_game` names
`Dispatch<C>`/`InboundVmCall` (and `trap.rs` names `Execute<C>`/`OutboundSysCall`) through
**`mp_abi`'s re-exports** — `mp_abi` already depends on `abi-transport` and is the seam crate
by definition, so it re-exports the four seam traits; module logic crates keep their frozen
dep sets, no new Cargo edges (§ Open questions, STATE-Q12). The receiver
this doc settles is a copyable context carrying the raw world pointer `vmMain`
threads **plus** the module-side `&Engine` it pairs with — **not** `GameWorld` itself:

```rust
// mp_game (the crate the orphan rule forces the `impl Dispatch<C>` blocks into,
// engine-seam § SEAM-D8). A Copy context pairing the raw `*mut GameWorld` derived from
// WORLD once per vmMain entry (§ WorldCell) with the module-side `&Engine` outbound-trap
// channel. porting-rules §17's arena/copyable-borrow-wrapper idiom (the GP2 precedent),
// NOT a field of GameWorld. `engine` is the `mp_engine_select::Engine` transport alias
// (SEAM-D13) — reachable because mp_game depends on mp_engine_select (workspace-architecture
// § Dependency edges) — NOT `mp_engine_core::Engine` (§ Engine disambiguation, STATE-Q8).
#[derive(Clone, Copy)]
pub struct GameContext<'e> {
    // PUB fields, constructed by struct literal (STATE-D8; round-6 resolution
    // 2026-07-03 — the WorldPtr `pub`-field precedent restored; the interim
    // private-fields + `pub fn new` detour is STRUCK). A `Copy` struct of a raw
    // pointer + a shared reference has no invariant a constructor would protect, so
    // the jampgame *shell* crate — though a separate crate wrapping mp_game — builds
    // it directly `GameContext { world, engine }`; **no `pub fn new` is exposed**.
    // engine-seam.md § SEAM-D8 now reads identically (its 'private + new()' text is
    // superseded and removed this same round — both docs read the same).
    pub world: *mut GameWorld,
    pub engine: &'e Engine,   // mp_engine_select::Engine (SEAM-D13); crate-qualify per STATE-Q8
}

// Each per-call InboundVmCall command C gets one impl; `&self` on the Copy context
// satisfies SEAM-D8's frozen signature WITHOUT changing the trait. The body reborrows
// `&mut *self.world` only at non-overlapping LEAF accesses and drops that `&mut` before any
// `trap::` call that can re-enter vmMain (STATE-D1 EntityId discipline) — so a mutating
// command mutates level/entities without any RefCell/effect queue, and no `&mut` ever
// spans a reentrant entry. Outbound traps go through `self.engine` as `trap::X(self.engine, …)`
// threaded into the logic fns in oracle call order.
impl<'e> Dispatch<GameRunFrame> for GameContext<'e> {
    fn dispatch(&self, args: <GameRunFrame as InboundVmCall>::Args)
        -> <GameRunFrame as InboundVmCall>::Output
    {
        // e.g. g_run_frame(unsafe { &mut *self.world }, self.engine, args) at a leaf that
        // holds no borrow across a trap:: re-entry; nested reborrows re-derive from self.world;
        // outbound traps issue as trap::X(self.engine, …).
        todo!("Port G_RunFrame — oracle/codemp/game/g_main.c:3582")
    }
}
```

The impl pattern is **frozen**: `Dispatch<C>` is implemented on `GameContext<'e>` (by
value, `Copy`), never on `GameWorld`; bodies reborrow `&mut GameWorld` at leaves and drop
it before re-entrant `trap::` calls, and issue outbound traps through `self.engine`. This
resolves the former STATE-Q5 (mutating-command receiver) **and** the former SEAM-Q12
(outbound-trap `&Engine` access): a `Dispatch<C>` body that must issue *outbound* traps
mid-dispatch (e.g. `GAME_INIT` → `trap_LocateGameData`/`trap_SV_RegisterSharedMemory`)
reaches them through `self.engine`, which `vmMain` supplied from `ENGINE.get()`
(§ `WorldCell`). That `engine` is the **module-side `mp_engine_select::Engine`** transport
alias (SEAM-D13), *not* this doc's engine-island `Engine` struct — `mp_game` cannot reach
`mp_engine_core` (§ Engine disambiguation, STATE-Q8), so no illegal upward crate edge is
introduced.

**SP mirror (STATE-D12).** SP `sp/game` carries the same `GameContext` shape
(`*mut GameWorld` + engine handle, **`pub` fields, struct-literal-constructed** — the
same round-6 shape as MP), but it is
**constructed per `game_export_t` export**, not once per `vmMain` (SP has no
`vmMain`/`Dispatch<C>` funnel): each SP export (`InitGame`/`G_RunFrame`/… reached
via `ge->…`) derives its own `*mut GameWorld` from the SP `WORLD` cell and pairs it
with the stored `game_import_t` (`gi`) rather than `&mp_engine_select::Engine`.
Outbound calls go `gi.X(...)` (the SP mirror of MP's `trap::X(self.engine, …)`). The
SP engine-handle Rust alias name/crate is **STATE-Q9** (owner: the SP slice).

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
    /// leading-playerState_t pointer at gclient_t stride; `None` = no client
    /// array. (MP always passes clients; SP never reaches this trap — it has
    /// zero LocateGameData call sites and registers via the GetGameAPI/SEAM-D2
    /// handoff, which has no gameClients equivalent, § Raven ground truth.)
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

`SeamPtr` is the transport's raw address word (a host pointer for `NativeDll` /
`Static`) — that semantic contract is what this doc fixes. As "the transport's"
address word its **type is defined in `abi-transport`** (the transport crate
every `mp/engine/*` crate depends on — workspace-architecture § Dependency
edges), so this engine-internal trait names it unqualified without a new edge.
Its *concrete* Rust representation (a `usize` alias vs. a newtype) is a
seam-executor mechanic and a non-goal here (§ Non-goals →
`docs/architecture/engine-seam.md`); the trait above is written against the
semantic contract only. For both `NativeDll` and `Static` the impl is the same
(engine-seam SEAM-D4): cache the base pointer + stride once at registration;
accessors do raw arithmetic — faithful to `sv.gentities` / `sv.mSharedMemory`,
zero cost.

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
`docs/architecture/lifecycle.md` (LIFE-D3; FROZEN 2026-07-03, so its variants are
frozen facts)**, not here; this doc fixes the
payload shape, the recovery-ordering contract (STATE-D3, recovery relocated
catch-side by STATE-D7), and — via STATE-D7 — the crate home (`mp_engine_qcommon`)
and the receiverless `com_error` signature. The
`com_error` *function* signature (this payload as the panic value, `-> !`, no
receiver) is in § `com_init`/`com_frame`/`com_shutdown`/`com_error` above.

**The fields are `pub`, and that is load-bearing, not stylistic.** STATE-D7 relocates all
per-level recovery to the **catch side in `mp_engine_core`** (`com_frame`/`com_init`) — a
**different crate** from `mp_engine_qcommon`, where `ComError` lives. That catch reads
`e.level`/`e.msg` after `downcast::<ComError>()` to drive the per-level recovery table, a
cross-crate field read that compiles **only** with `pub` fields. `ComError` is this doc's to
freeze (§ Scope — "the error payload"), so the `pub`-field shape above is authoritative.
**Cross-doc sync (STATE-Q7):** lifecycle.md's parallel block currently spells the fields
**non-`pub`** (`{ level: ErrorLevel, msg: String }`, lifecycle.md § Seam) even though its own
`com_error_recover` helper reads `e.level`/`e.msg` cross-crate — self-inconsistent there; it
must adopt the `pub` fields to match this freeze (tracked among STATE-Q7's pending cross-doc
amendments).

### Per-file placement (mechanical, not architectural)

The synthesized types this doc introduces — `GameWorld`/`GameContext`/`EntityId`
(`mp_game`), `WorldCell` (the module cdylib shell), and `zeroed_box`
(`native_platform`) — have **no owning Raven-header folder** (they render `g_main.c`'s
`level`/`g_entities`/`g_clients` globals + the `vmMain` seam, not a single header),
the identical situation module-loading.md pinned mechanically under
`crates/native/platform/src/module_loader/` (LOAD-D6) and lifecycle.md under
`mp_engine_qcommon`'s `common/` + `mp_engine_core`'s `lifecycle.rs` (§ Seam file
tree). Applying the **same** CLAUDE.md one-type-per-file / folder-mirrors-subsystem
convention (snake_case of the Rust type name; `impl` blocks colocate in their type's
file, mirroring `com_printf` beside `Common`) — **mechanical, settles nothing new**, a
dry-run renders it verbatim rather than inventing filenames, exactly as those two
sibling trees are pinned:

```
crates/mp/game/src/world/            // NEW module (`pub mod world;` in lib.rs) — the
  mod.rs                             //   module-island world + its Dispatch<C> receiver,
                                     //   mirroring module_loader/'s "one synthesized subsystem, one folder"
  game_world.rs                      // GameWorld + `impl GameWorld { fn zeroed }` (STATE-D2/D9)
  game_context.rs                    // GameContext<'e> (STATE-D8; `pub` fields, NO `new` — struct-literal)
                                     //   + ALL `impl Dispatch<C> for GameContext` blocks (every
                                     //   MpGameExport variant: GameInit/GameShutdown/GameRunFrame/…):
                                     //   impls colocate in their type's file per the convention above
                                     //   ("impl blocks colocate in their type's file"), so ONE
                                     //   game_context.rs, NOT one file per command — derived from the
                                     //   stated convention, settles nothing new
  entity_id.rs                       // EntityId (§B5)

crates/native/platform/src/
  zeroed_box.rs                      // `pub fn zeroed_box<T: ZeroValid>` + `unsafe trait ZeroValid`
                                     //   + the `[T; N]` blanket impl (STATE-D9); `pub mod zeroed_box;` in
                                     //   lib.rs — a sibling file beside platform.rs (other native primitives)

crates/jampgame/src/                 // the MP module cdylib SHELL (a separate crate wrapping mp_game)
  world_cell.rs                      // WorldCell (STATE-D6)
  lib.rs                             // `static WORLD: WorldCell` + `#[no_mangle] extern "C-unwind" fn vmMain`
                                     //   (the cdylib entry surface, § WorldCell)
```

**Not pinned here — `mod trap` is engine-seam.md's.** The outbound `trap::X(ctx.engine, …)`
wrappers the `GameContext`/`WorldCell` skeletons call (§ `GameContext`, § `WorldCell`) are
**not** among the synthesized types this doc introduces: they are engine-seam.md's
§ Call-site conventions module (`mod trap`, the per-call `pub fn X(engine: &Engine, …)`
wrappers, SEAM-D10/SEAM-D13), which that doc homes in the **logic crate `mp_game`** (SP:
`mod gi`, binding `game_import_t` directly). Its exact file is engine-seam.md's /
module-loading.md's to pin, not this doc's; by the same one-type/one-file-per-Raven-subsystem
convention this section applies, the mechanical default is `crates/mp/game/src/trap.rs`
(SP `gi.rs`), mirroring Raven's single `g_syscalls.c` (the one file all `trap_*` wrappers
live in) — a reproducible name, not an architectural choice (round-6 pinning (a),
2026-07-03).

SP mirrors the same layout under `crates/sp/game/src/world/`; the SP shell (`jagame`)
has **no** `vmMain` (STATE-D12), so its `WORLD: WorldCell` static + the
`ge->Init`/`ge->Shutdown` cell writes live in `lib.rs` beside the `game_export_t`
table (`world_cell.rs` unchanged). This pins **only** the reproducible file/folder
names, not an architectural choice (LOAD-D6 / lifecycle § Seam file tree precedent);
the module cdylib shells for the other modules (`crates/cgame`/`crates/ui`, STATE-D6)
carry their own island types (`CgState`/`UiState`) and are out of scope of this
game-module tree.

**Layout freeze (STATE-Q11 CLOSED, user ruling item 19, 2026-07-03).** The `world/` folder
**wins**: the synthesized module-island types live under `crates/{mp,sp}/game/src/world/`
(`mod.rs`, `game_world.rs`, `game_context.rs`, `entity_id.rs`), and the § Per-file placement
freeze above **stands as frozen**. The earlier drift — the skeleton seed had rendered
`GameWorld`/`GameContext` flat at `crates/mp/game/src/game_world.rs` / `game_context.rs` with
no `world/` subfolder and no `entity_id.rs` (`skeleton` branch, commit `497fff4`) — is
reconciled **toward the doc**: skeleton checkpoint 5 relocates the flat files into `world/`
and seeds `entity_id.rs` (`EntityId(u32)`), conforming to this freeze. No open question
remains; the doc layout is authoritative.

## Decisions

**STATE-D1 — The two-island model.** One owned per-mode `Engine` struct (fields
= owned subsystem structs: `Common{cvars,cmd,cbuf,fs,net}`, `Server` (liveness
`sv.state == SS_DEAD`, not an `Option` — item 20), `Option<Client>`, `snd`, …), allocated
as one zeroed `Box<Engine>` in `main()`, threaded `&mut` **down** call
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
`Vec<Enum>` entity store (would break the native pointer-cast layout the seam
crosses through); a process-global `GameWorld` (forecloses multi-world, §B3).

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
syscall handler — `G_DROP_CLIENT` (`sv_game.cpp:569`) → `SV_GameDropClient`
(`sv_game.cpp:110-114`) → `SV_DropClient` (`sv_client.cpp:580`) →
`VM_Call(gvm, GAME_CLIENT_DISCONNECT)` (`sv_client.cpp:640`), a fresh `vmMain`
entry while the outer one is live — each entry derives its **own** `*mut GameWorld`
from the cell, so the two live raw pointers alias soundly (Raven's C `level`/
`g_entities` globals aliased freely under those chains because C has no `&mut`).
*Because* the C `vmMain(command, args…)` entrypoint takes no context argument, so a
mutable-across-calls owned value must be reachable without one, and those live
re-entry chains alias it by design. *Rejected:* `Mutex` (deadlocks on those chains);
`RefCell` (panics on them); a context-threaded entrypoint (the ABI is fixed — no
context slot in `vmMain`). *(SP mapping: STATE-D12 — SP `jagame` has no `vmMain`;
`ge->Init`/`ge->Shutdown` write/take the cell and each `game_export_t` export
re-derives its own `*mut GameWorld`, so this reentrancy contract is MP-specific to
the `vmMain` funnel while the cell-lifetime and raw-pointer-per-entry parts carry over.)*

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
raw-pointer receiver of the earlier amendment is now the `Copy` `GameContext` context the
`Dispatch<C>` impls take by value (STATE-D8, amended 2026-07-03: `WorldPtr` → `GameContext`,
which also pairs in the outbound-trap `&Engine`). Skeleton in § `WorldCell`.

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

**STATE-D8 — The `Dispatch<C>` receiver is a `Copy` `*mut GameWorld`-carrying
context (resolves STATE-Q5; session 2026-07-02; receiver superseded `WorldPtr` →
`GameContext` by the 2026-07-03 amendment below, which also resolves SEAM-Q12).**
engine-seam froze
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
§ `GameContext`. *Because* an immutable `&self` on `GameWorld` cannot express per-call
mutation and STATE-D1 forbids the interior-mutability escapes, while a `Copy` pointer
wrapper carries the mutation capability through `&self` and matches the single
`*mut GameWorld` the shell already threads. *Rejected:* implementing
`Dispatch<C>` on `GameWorld` (immutable `&self` can't mutate); a mutating-only router
that bypasses `Dispatch<C>` (forks the seam into two dispatch shapes for no gain).

*Amendment (2026-07-03 — supersedes `WorldPtr`, resolves SEAM-Q12).* The receiver is
now **`pub struct GameContext<'e> { pub world: *mut GameWorld, pub engine: &'e Engine }`**
in `mp_game`, adding the `engine` field to the 2026-07-02 `WorldPtr`. Its fields are
**`pub`** and the jampgame shell constructs it **by struct literal**
`GameContext { world, engine }` — a `Copy` struct of a raw pointer + a shared reference
has no invariant a constructor would protect (the `WorldPtr` `pub`-field precedent), so
**no `pub fn new` is exposed**. The extra field is
the **module-side `mp_engine_select::Engine`** transport alias (SEAM-D13, *not*
`mp_engine_core::Engine`), reachable via `mp_game`'s existing `mp_engine_select` edge
(workspace-architecture § Dependency edges). So a `Dispatch<C>` body issuing outbound
traps mid-dispatch reaches them as `trap::X(self.engine, …)` in oracle order — closing
the former "out of scope (SEAM-Q12)" clause this record originally carried. `vmMain`
constructs `GameContext` per call from `WORLD` + `ENGINE.get()` (§ `WorldCell`).
`Copy`-ness, the `&self`-satisfies-SEAM-D8 property, and the STATE-D6 leaf-reborrow
discipline (`&mut *self.world`) are unchanged; engine-seam.md's cross-ref is amended to
name `GameContext` (tracked by STATE-Q7). Frozen shape in § `GameContext`.*

*Amendment (2026-07-03, round 6 — GameContext construction surface, resolves the round-5
cross-doc conflict, "all recommended").* An interim round-5 skeleton draft had briefly
given `GameContext` **private fields + `pub fn new`** (a compile-forced reading of
"foreign crate can't struct-literal private fields"). That detour is **struck**: the
fields are `pub` and the shell struct-literals the value, as above and in § `GameContext`.
engine-seam.md's matching 'private + new()' text is **superseded and removed** this same
round, so both docs read identically. STATE-D8's original `pub`-field/struct-literal
intent (the `WorldPtr` precedent) therefore **stands as written**.

**STATE-D9 — Zeroed heap construction via one `zeroed_box` helper (resolves STATE-Q6;
session 2026-07-02).** The `GameWorld` boxes (`Box<[gentity_t; MAX_GENTITIES]>` ≈1.83
MB, `Box<[gclient_t; MAX_CLIENTS]>`, plus `level_locals_t`) are all `#[repr(C)]`,
non-`Default`, all-zeroes-valid ABI structs Raven `memset`s in place; Rust heap-boxes
them, so the zeroed behavior needs a heap idiom C never had. The sanctioned one is a
single helper **`pub fn zeroed_box<T: ZeroValid>() -> Box<T>`** (`alloc_zeroed` +
`Box::from_raw`; the `ZeroValid` bound added by the round-6 amendment below)
in **`crates/native/platform`** (package `native_platform`) — Raven-free native-tier
vocabulary, the Rust mirror of C static zero-initialization, allocating **directly on
the heap** so the ~1.83 MB array never transits the stack (naive stack-build-then-box
risks overflow on constrained-stack targets, e.g. 1 MB default Windows threads). It
needs **no tier-rule change and no new crate** (`native/*` is usable by everything
cross-mode, workspace-architecture § tier definitions), but it **does require a new
direct Cargo dependency edge `mp_game -> native_platform`** — currently absent
(`mp_game` depends on `mp_qshared`/`mp_bg`/`mp_abi`/`mp_engine_select` only, and
`mp_qshared` does not re-export `native_platform`, so the edge is not transitive);
adding it is a slice-0
wiring task (SP dual `sp_game -> native_platform`, added by the amendment below). The
`unsafe` and its safety contract (`T` is `#[repr(C)]`, all-zero bit pattern valid) — in
the round-6 form (amendment below) — live in each type's `unsafe impl ZeroValid` beside
its layout asserts, not at the helper; `GameWorld::zeroed`
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

*Amendment (2026-07-03, round 6 — resolves STATE-Q10, "all recommended").* The frozen
signature becomes **`pub fn zeroed_box<T: ZeroValid>() -> Box<T>`**, still a **safe**
fn, where **`unsafe trait ZeroValid`** is a hand-rolled marker trait in
`native_platform` (bytemuck-`Zeroable` style, **no new dependency**) meaning "the
all-zero bit pattern is a valid value of `Self`". Each `#[repr(C)]` all-zeroes-valid ABI
type opts in with a one-line **`unsafe impl ZeroValid for X {}` colocated with its layout
static-asserts** — so the per-type audit lives exactly where the offset/size evidence
lives, call sites stay safe, and unsafety stays confined (§D11). Arrays (the entity/client
boxes) have no owning file for a per-type impl, so `native_platform` also carries the
blanket **`unsafe impl<T: ZeroValid, const N: usize> ZeroValid for [T; N] {}`**
(checkpoint-4 finding 17). The `mp_game -> native_platform` edge above gains its **SP dual
`sp_game -> native_platform`** (checkpoint-4 finding 16 — SP `level_locals_t`'s `ZeroValid`
impl lives in `sp_game`); both rows land in workspace-architecture's dep table. This closes
STATE-Q10 (the earlier "safe fn whose precondition Rust cannot check" unsoundness).
*Because* the marker moves the all-zero precondition into the type system without a new
dependency, keeping the frozen `GameWorld::zeroed` body genuinely safe. *Rejected:*
`unsafe fn` (would reintroduce `unsafe {}` into `GameWorld::zeroed`); a documented-only
precondition (unsound — `zeroed_box::<String>()` would compile).

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

**STATE-D12 — SP access discipline: `GetGameAPI` fn-pointer table, no
`vmMain`/`Dispatch<C>` (resolves the round-4 escalation; session 2026-07-03).**
SP `jagame` has **no `vmMain`, no command decode, no `Dispatch<C>` routing** —
`GetGameAPI(game_import_t *import)` (`code/game/g_main.cpp:875`) returns a
`game_export_t` (`globals`, `g_main.cpp:48`) of **direct fn pointers**
(`globals.Init = InitGame` `g_main.cpp:881`, `globals.RunFrame = G_RunFrame` `:895`, …);
the engine calls `ge->Init(...)`/`ge->RunFrame(...)` directly
(`code/server/sv_game.cpp:669,690`), never a numbered `VM_Call`. So the MP-side
`WorldCell`/`vmMain`/`GameContext`-per-`vmMain` mechanisms below **do not apply
verbatim** to SP; they map as follows:
- **(a) World lifetime — `ge->Init` writes the SP `WORLD` cell, `ge->Shutdown`
  takes it** (the direct analog of MP's `GAME_INIT`-write / `GAME_SHUTDOWN`-take,
  STATE-D6): `InitGame` constructs the zeroed SP `GameWorld` (`GameWorld::zeroed`,
  STATE-D9) **into** the cell; `ShutdownGame` takes it out (drop = teardown).
- **(b) Per-export `GameContext` construction — each `game_export_t` fn derives
  its own `*mut GameWorld` from the SP `WORLD` cell in its prologue and constructs
  the SP `GameContext` mirror (in `sp/game`) itself.** This **per-export**
  construction replaces MP's **once-per-`vmMain`** construction — SP has no single
  dispatch funnel, so each entry point re-derives the raw pointer, pairs it with the
  SP engine handle, then reborrows `&mut GameWorld` only at leaf accesses (the
  STATE-D1 discipline, unchanged).
- **(c) SP module-side engine handle = the stored `game_import_t`** the engine
  passes into `GetGameAPI` (`gi = *import`, `g_main.cpp:878`; `game_import_t gi`,
  `g_main.cpp:47`, extern `g_shared.h:830`; type `g_public.h:168-471`) — the SP
  mirror of MP's `mp_engine_select::Engine` outbound-transport alias (SEAM-D13). SP
  outbound calls are `gi.EntitiesInBox(...)`/`gi.SetConfigstring(...)`
  (`g_main.cpp:298,262`), the method-on-`gi` mirror of MP's `trap::X(self.engine, …)`.
  The SP `GameContext` mirror pairs `*mut GameWorld` with this handle exactly as MP's
  pairs it with `&Engine`.
*Because* SP statically links its game and hands over a struct of real fn pointers
(dossier §1k, §2.4), so none of the QVM-shaped `vmMain`/numbered-dispatch machinery
exists to mirror; only the world-cell lifetime and per-entry raw-pointer threading
carry over. *Rejected:* forcing a synthetic `vmMain`/`Dispatch<C>` shim onto SP
(invents a dispatch layer Raven never had); a single once-per-frame `GameContext`
construction (SP has no single entry funnel — each export is called independently).
**The precise Rust name/crate of the SP engine-handle alias is not derivable from
settled decisions** (no `sp_engine_select` alias is established; MP's SEAM-D13 alias
also encodes a transport selection SP's static link lacks) — **STATE-Q9**, owner:
the SP slice; do not invent.

**STATE-D13 — Engine-island large values reuse the `zeroed_box` path (session
2026-07-03, "all recommended").** `Engine::new`'s construction of its large
embedded-by-value members — `sv: Some(Server)` (whose `server_t` embeds the 1024-entry
`svEntities` table plus `configstrings`/`models`) and `cm: CollisionWorld` (the `cmg`
`clipMap_t`) — **reuses the STATE-D9 `zeroed_box<T: ZeroValid>` heap path**, by direct
analogy to `GameWorld`: `stack-then-move Default` is rejected for the same
stack-overflow class (`server_t` is large enough to overflow a constrained thread stack
if built on the stack). `server_t` and `clipMap_t` therefore each gain a one-line
`unsafe impl ZeroValid` beside their existing layout static-asserts (STATE-D9 amendment),
exactly like the module-island ABI types. This **sanctions the `mp_engine_core ->
native_platform` Cargo edge** that reuse requires (SP dual `sp_engine_core ->
native_platform`); `native/*` is cross-mode tier-legal for any crate
(workspace-architecture § tier definitions), so no tier-rule change — only the new direct
edge. The **construction mechanics** (the `Engine::new` body, field order, the
`Some(Server)` wrap) live in **lifecycle.md** (§ Non-goals — construction order); this
doc keeps the single frozen home for the *zeroing idiom* (STATE-D9) and records only that
the engine island reuses it, the Engine block pointing to lifecycle.md for the rest.

*Scope + cross-doc gap (noted 2026-07-03 — SUPERSEDED by the amendment below; retained as
history. Its "LIFE-Q7 is OPEN" and "STATE-Q10 open" claims are both stale — LIFE-Q7 and
STATE-Q10 are now CLOSED, and the whole-Engine zeroed allocation dissolves the residual.)*
STATE-D13 names only the two **large-by-value
headline members** — `server_t` (664960 B via `svEntities[1024]`, `server_t.rs:69`) and
`clipMap_t` (`cmg` + `SubBSP[32]`) — as the stack-overflow-class values that require the
`zeroed_box` path, and gives those two the `unsafe impl ZeroValid`. It does **not** settle how
the **wrapping** structs are assembled: `Server`'s remaining fields (`svs: serverStatic_t`,
itself carrying `challenges[1024]`; `bot`; `master_heartbeat`; SP `savegame`) and
`CollisionWorld`'s non-headline `gpvCachedMapDiskImage`/`gbUsingCachedMapDataRightNow` cache
pair — whether each also warrants `zeroed_box`/`ZeroValid` or an ordinary construction, and the
field order of the `Some(Server)` wrap — are **construction mechanics** this doc punts to
lifecycle.md (§ Non-goals — construction order; Engine block). That cross-ref does **not yet
land**: lifecycle.md's owning **LIFE-Q7 is OPEN**, and its text is **stale** — it predates
STATE-D13 (still asserting "no cited record sanctions [engine-island] reuse", which STATE-D13
now does) and still calls STATE-Q10 open (STATE-Q10 was closed by the STATE-D9 amendment). So a
porter writing the full `Engine::new` body cannot complete these non-headline fields from the
current doc set. The residual assembly decision stays **LIFE-Q7's, not settled here**; the
stale-LIFE-Q7 sync is tracked under STATE-Q7.

*Because* the engine island hits the identical large-by-value-on-the-heap need the module
island already solved, so one idiom should serve both rather than a second mechanism.
*Rejected:* a stack-built `Default` for `Server`/`CollisionWorld` (same overflow risk
STATE-D9 rejected); a separate engine-only zeroing helper (duplicates `zeroed_box`).

*Amendment (2026-07-03, user ruling item 20 — per-member mechanics superseded by
whole-Engine zeroing).* STATE-D13's *per-member* framing (give `server_t`/`clipMap_t` each
a `zeroed_box` call, then assemble the `Some(Server)` wrapper) is **superseded**:
`Engine::new() -> Box<Engine>` now allocates the **whole aggregate** as a boxed ZEROED heap
buffer with in-place init of the non-zero-valid fields (`Common.time_base`) before exposure —
the `MaybeUninit` pattern, **not** an `unsafe impl ZeroValid for Engine` (unsound — the
aggregate is not all-zeroes-valid, checkpoint-5 finding 21; § Engine amendment). `sv` is a
bare `Server` (liveness `sv.state == SS_DEAD`, `server.h:47-54`), not `Some(Server)`. The
one-line `unsafe impl ZeroValid` on `server_t`/`clipMap_t` beside their layout asserts
**still stands** — it is what makes those `#[repr(C)]` members' zero sound. What **dissolves** is the
*Scope + cross-doc gap* note **above** and its residual: because the entire `Engine` is one zeroed
heap value, there is no field-by-field wrapper assembly and no `Some(Server)` stack-ordering
question left to settle — the non-headline fields (`serverStatic_t.challenges[1024]`, `bot`,
`master_heartbeat`, SP `savegame`, the CM cache pair) are zeroed by the same single allocation.
lifecycle.md's **LIFE-Q7 is thereby CLOSED** (it owned exactly that dissolved residual); non-zero
init runs in `com_init` where Raven runs it. The *Scope + cross-doc gap* note above is retained
only as superseded history — its "LIFE-Q7 is OPEN" / "STATE-Q10 open" claims are both stale
(LIFE-Q7 and STATE-Q10 are now CLOSED).

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
  `trap_SV_RegisterSharedMemory` (`g_main.c:920`), each issued as `trap::X(ctx.engine,
  …)` through the `GameContext` receiver's `&Engine` (STATE-D8 amendment 2026-07-03 —
  the channel the former SEAM-Q12 blocker lacked) — but their **engine-side
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
  a raw `*mut GameWorld` (as `GameContext`, paired with `ENGINE.get()`) from that same
  cell and threads it through dispatch, reborrowing `&mut GameWorld` only at leaf
  accesses (STATE-D6 access discipline, amended 2026-07-02; receiver 2026-07-03). The
  two module-island skeleton mechanics Slice 0
  exercises are now **unblocked**: GAME_INIT's zeroed-`GameWorld` construction
  (STATE-D9, `GameWorld::zeroed`) and GAME_RUN_FRAME's mutating dispatch via
  `impl Dispatch<C> for GameContext` (STATE-D8) — both frozen here, so Slice 0's
  `vmMain` skeleton (§ `WorldCell`) compiles as written.
  **Needs frozen here (and are):** the `GameWorld`/`WorldCell`/`GameContext` shapes,
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
  `GetGameAPI` table register (SEAM-D2). Multi-world retrofit (STATE-D2) is a
  post-parity slice — the value-typed, registration-keyed design makes it
  non-structural.

## Open questions

The survey's own forks (dossier § Design forks 1–7) are all settled by
STATE-D1..D13 or the cited ledger entries (DEC-04/05/07/08). The former
structural gaps — the defining crate for `Engine` (was STATE-Q1), where the
owned `GameWorld` lives across `vmMain` calls (was STATE-Q3), `com_error`'s
recovery reachability + receiver (was STATE-Q4), the mutating-command `Dispatch<C>`
receiver (was STATE-Q5), and the zeroed-`GameWorld` construction idiom (was STATE-Q6)
— are resolved by STATE-D5, STATE-D6, STATE-D7, **STATE-D8, and STATE-D9**
respectively (escalation sessions 2026-07-02). The former outbound-trap `&Engine`
access (engine-seam SEAM-Q12) is **resolved this round (2026-07-03)** by the
`GameContext` receiver, which pairs in `engine: &Engine` (STATE-D8 amendment).
**Two more closed this round (2026-07-03, "all recommended"):** STATE-Q8 (the `Engine`
name collision — keep both, crate-qualify, no renames, citing the round-4 record) and
STATE-Q10 (the `zeroed_box` safe-generic soundness — the `ZeroValid` marker bound,
STATE-D9 amendment). The
remaining cross-doc dependencies — the seam dispatchers (engine-seam.md) and the
`ErrorLevel` enum plus construction order (lifecycle.md, LIFE-D3) — are scoped
non-goals owned elsewhere, not unresolved decisions *of this doc*; but their owning
docs are not yet all FROZEN (engine-seam.md, lifecycle.md, and module-loading.md
are all now REVIEWED, none yet FROZEN), which is itself an open item (STATE-Q7). **Three**
questions remain (plus one tracked note), each with a named owner and going back to a
design session — none is self-resolvable here; all three (STATE-Q2/Q7/Q9) are the
sanctioned, owner-named **non**-blockers of the prior rounds (per the gate policy). The
round-7 gate's two Slice-0 stamping blockers are **CLOSED** (user, 2026-07-03, items
24-25): **STATE-Q12** — `mp_abi` re-exports the four `abi-transport` seam traits; and
**STATE-Q13** — `mp_game` re-exports `MpGameExport` at its crate root. STATE-Q11 also
**closed** this round (user ruling item 19 — `world/` folder wins; § Layout freeze):

- **STATE-Q2** (below) — the four §F subcrates' `Engine`-island attachment;
- **STATE-Q7** (below) — the sequencing gate: **CLOSED 2026-07-03** by the
  whole-set freeze (user sign-off; all four docs FROZEN together);
- **STATE-Q9** (below) — the SP engine-handle Rust alias name/crate (STATE-D12(c));
- **Tracked note (finding 15)** (below STATE-Q9) — SP *engine-side* signatures
  (`sv_init_game_progs`, the `ge` handle placement) have no doc home yet.

- **STATE-Q2 — `Engine`-island attachment for the four §F subcrates —
  CLOSED 2026-07-09 (engine-fork-discovery rulings 12/13 + 43).** The
  placement half is ruled: each §F subsystem's engine-side state becomes a
  **direct `Engine` field** (`engine.icarus`, `engine.nav`, `engine.g2`,
  `engine.rmg`, `engine.roff` — no `Option`/`Box` wrapping, ruling 12);
  internal shapes stay owned by their §F docs
  (`docs/subsystems/{icarus,rmg-terrain,ghoul2-server,npcnav,roff}.md`).
  The fields land with each subsystem's port waves (the struct types do
  not exist before their §F work). `Engine` implements `EngineHost` for
  subsystem calls through the pinned split-borrow view (ruling 43):
  `pub struct EngineHostView<'a>` — AMENDED by DEC-23 (host-seam
  restructure, 2026-07-11): the view lives in **`mp_engine_qcommon`**
  (`common/engine_host_view.rs`), holds `&mut Common` + `&mut
  CollisionWorld` plus the six type-erased slots (`sv`/`cl`/`bot`/`rm`/
  `rmg`/`g2`), and is the single world parameter of every host-consuming
  C-track function (the receiver-list convention collapsed into it). The
  split constructor is `mp_engine_core::engine_host_view(&mut Engine)`
  (plain field reborrows + slot wraps); Server/RenderModels-touching
  trait methods route through `Common.hooks` accessor fields installed
  at boot. See `docs/decisions.md` DEC-23.

- **STATE-Q7 — Freeze-ordering — CLOSED 2026-07-03 (whole-set freeze, user
  sign-off).** All four architecture docs advanced REVIEWED → FROZEN together,
  which was the resolution the entry anticipated: every cross-doc ID this doc's
  "FROZEN HERE" blocks lean on (engine-seam's SEAM-D6/D8/D13 + `ServerGame`
  shape; lifecycle's LIFE-D3 `ErrorLevel` + `Engine::new`/`com_init` split +
  construction order; module-loading's `ModuleTransport`/`ModuleSlot`
  internals) is now a frozen fact. The reconciliation ledger below records the
  final synced state of the formerly-contested pairs.
  **Cross-doc reconciliation ledger (2026-07-03 reconciliation pass — the single editor
  that lands all four docs' syncs).** The round-5/round-6 cross-doc pairs are now
  **RECONCILED**, all rendered identically across the affected docs by this pass:
  - **GameContext pair — RECONCILED.** `pub` fields + struct-literal construction, no
    `::new` (STATE-D8; user round-5 fork 1). engine-seam.md § SEAM-D8 now frames it as
    "reconciled 2026-07-03; STATE-D8 is the frozen home", dropping the "state-ownership
    lags / this doc governs" tie-break prose. Both docs read identically.
  - **vmMain pair — RECONCILED.** `vmMain(command: AbiCommand, arg0..arg11: AbiWord) ->
    AbiWord` (STATE-D6 § WorldCell tie-break; engine-seam SEAM-D9/D10 authoritative for the
    export signature; oracle `g_main.c:515` is the all-`int` 32-bit original, `AbiWord = isize`
    the settled 64-bit-widened dual).
  - **ComError pair — RECONCILED.** `pub level`/`pub msg` fields in **both** docs
    (state-ownership § `ComError` freezes it; lifecycle.md's own cross-crate
    `com_error_recover` read requires it — lifecycle.md now spells the fields `pub`).
  - **com_printf pair — RECONCILED.** `com_printf(&mut Common, …)` in `mp_engine_qcommon`
    (STATE-D11; lifecycle.md renders it identically — the narrowest owner, no uphill callers).
  - **LIFE-Q7 — CLOSED** (user ruling item 20): the whole-Engine boxed-zeroed
    `Engine::new() -> Box<Engine>` (§ Engine amendment / STATE-D13 amendment) dissolves the
    residual wrapper-assembly / stack-ordering question LIFE-Q7 owned; lifecycle.md's LIFE-Q7
    is closed the same round. No longer stale-open.

  **Legitimately still open / owned outside these four docs** (not reconcilable within this
  pass): STATE-D7 amends **workspace-architecture.md**'s engine-tier line; STATE-D10 amends
  lifecycle.md's LIFE-Q5 (registry attachment, already rendered). One **mechanical
  ground-truth correction remains pending in engine-seam.md** (beyond this reconciliation
  pass's settled list, flagged for the next engine-seam revision): the `GAME_INIT`/
  `GAME_SHUTDOWN` pre-decode spelling. This doc's § `WorldCell` bootstrap casts the
  SCREAMING_SNAKE `MpGameExport::GAME_INIT`/`GAME_SHUTDOWN` variants (the only spelling that
  compiles — the CamelCase `GameInit`/`GameShutdown` are distinct `InboundVmCall` marker
  structs, not enum variants, per the enum-fidelity rule / `exports.rs:6`); engine-seam.md
  § SEAM-D6/SEAM-D10 absorbed the identical correction 2026-07-03 (reconciliation pass) —
  both docs now spell `MpGameExport::GAME_INIT as c_int`. Named here only so the sync
  stays greppable — the variant names are fixed by cited code, not a design choice.

- **~~STATE-Q8~~ — Two distinct types are both named `Engine` — CLOSED (session
  2026-07-03, citing the settled round-4 decision).** `mp_engine_core::Engine`
  (this doc — the owned engine-island aggregate) and `mp_engine_select::Engine`
  (engine-seam SEAM-D13 — the cfg'd module-side outbound-transport alias
  `CEngine`/`Static`) share the bare name in different crates for different
  purposes. **Resolution: keep BOTH names, no renames; always crate-qualify.**
  workspace-architecture.md carries the canonical disambiguation block (§ crate tiers,
  2026-07-03); the distinction is also documented inline (§ Engine, and the naming note
  at the frozen `Engine` struct). The former SEAM-Q12 `&Engine` ambiguity is resolved to
  the module-side alias — concretely carried by `GameContext.engine` (STATE-D8 amendment
  2026-07-03), crate-graph-derived. The question of whether to *rename* one outright is
  settled **no** (the round-4 keep-both + disambiguate record). *Rejected:* renaming
  either type (the round-4 decision keeps both; disambiguation is by crate-qualification,
  not renaming).

- **STATE-Q9 — The SP engine-handle Rust alias name/crate (STATE-D12(c)).**
  *Owner: the SP slice.* STATE-D12 settles that SP's module-side engine handle is
  the stored `game_import_t` (`gi`, `code/game/g_main.cpp:47,878`, extern
  `g_shared.h:830`) — the SP mirror of MP's `mp_engine_select::Engine`
  outbound-transport alias (SEAM-D13). But **no SP alias name/crate is derivable
  from settled decisions**: no `sp_engine_select` alias exists yet, and MP's
  SEAM-D13 alias additionally encodes a `NativeDll`/`Static` transport
  selection that SP's static-link/`GetGameAPI` handoff has no analog for — so the
  MP naming does not transpose cleanly. Per the round-4 mandate, this is flagged,
  not invented: the SP slice settles the concrete Rust name/crate when SP mirror
  work begins (the handoff already defers the whole SP surface —
  `sp_engine_core`/`select`, SP `GameWorld`, jagame live exports — to that point).
  Not a stamping blocker (sanctioned, owner-named).

- **Tracked note (checkpoint-3 finding 15) — SP *engine-side* signatures have no
  doc home yet.** *Owner: the SP engine-island pass (a future doc round).* The
  skeleton seed marked `sv_init_game_progs` and the `ge` (`game_export_t`) handle
  placement in-source as placeholders (cites `code/server/sv_game.cpp:478,403,669-691`);
  no Group-A doc freezes the SP engine tier yet (this doc's SP content, STATE-D12, is
  the *module* side). It is recorded here so the open work is greppable, and handed to
  the SP engine-island pass. **Not a stamping blocker** (tracked, owner-named; round-6
  pinning (d) 2026-07-03) — this doc invents no SP engine-side signature.

- **~~STATE-Q12~~ — `mp_game`'s path to the `abi-transport` seam traits — CLOSED (user,
  item 24, 2026-07-03).** **Resolution: candidate (b) — `mp_abi` re-exports the four seam
  traits** (`Dispatch`/`InboundVmCall`/`Execute`/`OutboundSysCall`, defined in
  `crates/abi-transport/src/generic/{inbound,outbound}.rs`, engine-seam § Seam definition).
  `mp_abi` already depends on `abi-transport` and is the seam crate by definition — it
  defines the `InboundVmCall`/`OutboundSysCall` command markers (`GameInit`, `MpGameImport`),
  so it is the standing Raven↔transport bridge; `mp_game` (and its peers) reach the traits as
  `use mp_abi::…` through their **existing** `mp_abi` edge. Module logic crates keep their
  frozen dep sets — **no new Cargo edges** — and the SEAM-D10/SEAM-D13 no-cfg/no-feature
  invariant is untouched. The § `GameContext` `Dispatch<C>` impls and the
  `crates/mp/game/src/trap.rs` `mod trap` wrappers now have a doc-sanctioned in-scope path.
  Rendered in workspace-architecture.md (dep-table/ergonomics line, 2026-07-03). *Rejected:*
  (a) a direct `mp_game -> abi-transport` edge (an edge the frozen dep table omits, which the
  `mp_engine_select` indirection was built to avoid); (c) `mp_engine_select` re-exports
  (SEAM-D13 scopes that leaf to the *outbound* `type Engine` alias only).

- **~~STATE-Q13~~ — the `jampgame` shell's path to `mp_abi`'s `MpGameExport` — CLOSED (user,
  item 25, 2026-07-03).** **Resolution: candidate (b) — `mp_game` re-exports `MpGameExport`
  at its crate root** (`pub use mp_abi::game::exports::MpGameExport;` in
  `crates/mp/game/src/lib.rs`), reached via the shell's **existing** `mp_game` edge. SEAM-D10's
  frozen exactly-two-edges shell property (`abi_transport` + `mp_game`) **stays intact** — the
  shell sees the seam through the logic crate it wraps; no `jampgame -> mp_abi` edge is added.
  The frozen § `WorldCell` `vmMain` bootstrap (`command == MpGameExport::GAME_INIT as c_int`)
  now compiles through this path. Rendered in workspace-architecture.md (dep-table/ergonomics
  line, 2026-07-03). *Rejected:* (a) a third `jampgame -> mp_abi` Cargo edge (widens the frozen
  two-edge shell); (c) an `mp_engine_select` re-export (that leaf is the outbound transport
  alias, not the inbound command vocabulary).

- **~~STATE-Q11~~ — § Per-file placement folder freeze vs the committed skeleton's flat
  layout — CLOSED (user ruling item 19, 2026-07-03).** The `world/` folder **wins**: the
  synthesized module-island types stay frozen under `crates/{mp,sp}/game/src/world/`
  (`mod.rs`, `game_world.rs`, `game_context.rs`, `entity_id.rs`), and § Per-file placement
  stands as written. The skeleton conforms: checkpoint 5 relocates the earlier flat
  `game_world.rs`/`game_context.rs` into `world/` and seeds `entity_id.rs` (`EntityId(u32)`).
  *Rejected:* amending the freeze to the committed flat layout (the user ruled the folder
  form wins). Entry retained struck-through; § Layout freeze carries the live text.

*Resolved this round (2026-07-03, "all recommended"):*
- **STATE-Q11** (the per-file placement folder freeze vs the flat skeleton) — `world/` folder
  wins (user ruling item 19); § Per-file placement stands frozen, skeleton checkpoint 5
  conforms. Entry above, struck through.
- **STATE-Q8** (the two `Engine` types' name collision) — keep both names, always
  crate-qualify, no renames (citing the round-4 keep-both + disambiguate record;
  workspace-architecture.md carries the canonical disambiguation block). Entry above,
  struck through.
- **STATE-Q10** (the `zeroed_box<T>` safe-generic soundness) — closed by the
  **STATE-D9 amendment**: `zeroed_box<T: ZeroValid>()` stays a SAFE fn, `ZeroValid`
  a hand-rolled `unsafe` marker trait in `native_platform`, per-type `unsafe impl`
  colocated with layout asserts (§ `zeroed_box`, STATE-D9 amendment).
- **SEAM-Q12** (the outbound-trap `&Engine` channel a `Dispatch<C>` body needs) — closed
  by the **STATE-D8 amendment**: the receiver is `GameContext<'e>`, pairing
  `world: *mut GameWorld` with `engine: &'e Engine` (mp_engine_select alias), so a body
  issues outbound traps as `trap::X(self.engine, …)` (§ `GameContext`); `WorldPtr` is
  superseded.
- **GameContext construction surface** (round-5 fork 1) — `GameContext` fields are
  `pub` and the shell struct-literals the value; the interim private-fields/`pub fn new`
  detour is struck, engine-seam.md's matching text superseded and removed the same round
  (STATE-D8 round-6 amendment).

*Resolved 2026-07-02:* **STATE-Q5** (the mutating-command `Dispatch<C>` receiver) is
closed by **STATE-D8** — `Self` is a `Copy` `*mut GameWorld`-carrying context (originally
`WorldPtr`, now `GameContext`), bodies leaf-reborrow `&mut` (§ `GameContext`). **STATE-Q6**
(the zeroed-heap construction idiom + §D11 licensing) is closed by **STATE-D9** — the one
`zeroed_box` helper in `native_platform`, used by `GameWorld::zeroed`
(§ `zeroed_box` / `GameWorld::zeroed`).
