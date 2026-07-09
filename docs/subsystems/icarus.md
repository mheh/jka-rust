# ICARUS sequencer — MP engine (§F idiomatic reimplementation) Design
Status: DRAFT     Supersedes: none
Decision prefix: ICARUS     Ledger deps: DEC-09, DEC-10; engine-fork-discovery forks 2, 4, 5, 7 + §F rulings 11–15, 17, 19, 20; GOAL-engine M2 gate

## Standing context

Links only — never restate:

- `docs/workspace-architecture.md` — crate graph; ICARUS lives in the
  near-isolate crate `mp_engine_icarus` (436 internal edges, 25 into qcommon,
  6 into server; `docs/plans/2026-07-08-mp-engine-build-out.md` §graph rows).
- `docs/porting-rules.md` — §F (C++-track idiomatic reimplementation, GP2
  exemplar), §17–21 (design-before-transcription, differential parity, UB
  divergence, one-class-per-file), comment/source-ref rules.
- `docs/decisions.md` — DEC-09 (engine verification: TU golden harnesses +
  live peers), DEC-10 (checkpoint/reset skeleton build).
- `docs/handoffs/engine-fork-discovery.md` — fork-2 (global placement → Engine
  fields, no `static mut`), fork-4 (faithful owned arenas), fork-5 (dispatch
  tables → plain Rust structs of `fn` items), fork-7 (the 5-doc §F list, ICARUS
  named); and the §F doc-session **rulings 11–15, 17, 19, 20** (user,
  2026-07-09) that settle this doc's former open questions — including the
  final-revision pair ruling 19 (EngineHost `gentity` service, ICARUS-D1) and
  ruling 20 (Icarus arena dropped entirely, ICARUS-Q7 settled, ICARUS-D3)
  (ledger `:114-158`).
- `docs/GOAL-engine.md` — Stage-0 game-host interface crate checklist (`:43-55`,
  an unchecked prerequisite bullet — it names the crate's *role*, not its name or
  trait surface), M2 gate ("icarus differential goldens"). The crate's importable
  path (`crates/mp/host-interface`, **name TBD**) and the full `EngineHost` method
  roster are pinned by the Stage-0 interface-crate design
  (`docs/plans/2026-07-08-mp-engine-build-out.md:250`, handoff ruling 11), **not
  here** — see § Scope & non-goals (the `EngineHost` punt) and ICARUS-D1.
- GP2 exemplar: `crates/mp/engine/qcommon/src/gp2/`, `tools/gp2-oracle/`.

## Scope & non-goals

**In scope.** The 10 `WinDed.vcproj`-Release-linked ICARUS sources
(`oracle/codemp/icarus/`): `BlockStream.cpp`, `GameInterface.cpp`,
`Instance.cpp`, `Interface.cpp`, `Memory.cpp`, `Q3_Interface.cpp`,
`Q3_Registers.cpp`, `Sequence.cpp`, `Sequencer.cpp`, `TaskManager.cpp` —
253 fns / 6,749 LOC (plan §graph row). Every method ports except the three
zero-caller fns §20-drops (§F: only zero-caller API is droppable per §20) —
`Svcmd_ICARUS_f` (ICARUS-D6) and `ICARUS_Malloc`/`ICARUS_Free` (ICARUS-D3 as
amended by ruling 20, the arena being dropped). Target crate: `mp_engine_icarus`
(MP only).

**Out of scope.** `Interpreter.cpp` and `Tokenizer.cpp` are **not** in the link
set. The dedicated server never compiles scripts: `ICARUS_RegisterScript`
appends `.IBI` and `FS_ReadFile`s a **precompiled** block-instruction blob
(`oracle/codemp/icarus/GameInterface.cpp:346-395`); the `.txt`→`.IBI` compiler
(interpreter/tokenizer) is an offline/SP tool — reused only once, out-of-tree,
to build the golden corpus (ICARUS-D4). The type-port skeletons under
`crates/mp/engine/icarus/src/interpreter/` and `.../tokenizer/` stay untouched
by this port. SP (`oracle/code/icarus/`) is a later SP-diff pass (DEC-04/§20),
not a unification now — see § Verification strategy and ICARUS-D8.

**Punts.** The server-side syscall dispatch that routes `G_ICARUS_*` into this
crate's public fns lives in `SV_GameSystemCalls`
(`oracle/codemp/server/sv_game.cpp:739-832`), owned by the server crate, not
here — this doc pins the *callee* signatures (§ Seam definition), the server
doc pins the switch. Frame driving (per-entity `CTaskManager::Update`) is the
game module's `G_ICARUS_MAINTAINTASKMANAGER` trap each frame
(`sv_game.cpp:763-773`); this doc owns `Update`, the server owns the trap arm.

**The `EngineHost` trait itself — its importable crate path and full method
roster — is not defined here.** It is owned by the Stage-0 interface-crate design
(`docs/plans/2026-07-08-mp-engine-build-out.md:250-251`, handoff ruling 11): a
`crates/mp/host-interface` (**name TBD**) crate of Rust traits transcribing the C
seam (syscall surface, vmcall driver, shared-memory contract). This doc names the
services ICARUS consumes and types every §F fn `(&mut Icarus, &mut impl
EngineHost)` (ICARUS-D1); it does **not** fix the trait's Rust method
names/signatures or the `use` path. The oracle-side ground truth each service
wraps is cited in § Seam definition (Traps / cross-crate calls); porters bind
those host methods against the Stage-0 trait once it lands — it lands before
ICARUS in the port order (§ Slice hooks), never a stub. This mirrors the sibling
§F docs, which defer the identical dependency the same way
(`docs/subsystems/roff.md` non-goals, `docs/subsystems/ghoul2-server.md`
non-goals). Because the crate name is TBD upstream, no `use` line can be
finalized until Stage-0 lands — that is a settled cross-doc prerequisite for all
five §F subsystems, not an ICARUS decision.

## Raven ground truth

CITE OR OMIT. All cites are `oracle/codemp/icarus/…` unless noted.

**Two-directional seam.** ICARUS is engine-side but talks to the game module
both ways:

- **Inbound** (game→engine syscalls): 19 `G_ICARUS_*` arms in
  `SV_GameSystemCalls` call this crate's public fns
  (`sv_game.cpp:739-832`) — `ICARUS_RunScript`, `ICARUS_RegisterScript`,
  `ICARUS_Init`, `ICARUS_ValidEnt`, `ICARUS_InitEnt`, `ICARUS_FreeEnt`,
  `ICARUS_AssociateEnt`, `ICARUS_Shutdown`, `Q3_TaskIDSet/Complete/Pending`,
  `Q3_SetVar`, `Q3_VariableDeclared`, `Q3_Get{Float,String,Vector}Variable`,
  plus inline `gSequencers`/`gTaskManagers` presence checks
  (`ISINITIALIZED`, `MAINTAINTASKMANAGER`, `ISRUNNING`).
- **Outbound** (engine→game vmcalls): the `interface_export_t` `I_*` function
  table (`interface.h:17-70`) is wired by `Interface_Init`
  (`Q3_Interface.cpp:956-1008`). Its `Q3_*` implementations write a
  `T_G_ICARUS_*` struct into the server's shared-memory window
  (`sv.mSharedMemory`, `oracle/codemp/server/server.h:87`; alias of the game's
  `gSharedBuffer[8192]`, `oracle/codemp/game/g_local.h:85-86`,
  `trap_SV_RegisterSharedMemory`) then `VM_Call(gvm, GAME_ICARUS_*)`
  (e.g. `Q3_PlaySound`, `Q3_Interface.cpp:313-323`). The `GAME_ICARUS_*` enum
  and the 17 `T_G_ICARUS_*` structs (`oracle/codemp/game/g_public.h:770-923`)
  are already layout-ported into `mp_qshared`
  (`crates/mp/qshared/src/common/mp/qcommon/t_g_icarus_*.rs`,
  `game_export_t.rs`); the game-side `GAME_ICARUS_*` handlers (the 17 inbound
  vmcalls, oracle `g_main.c:558-668`) are already wired in `mp_game`
  (dispatch in `crates/mp/game/src/world/game_context.rs`, arg-type re-exports
  in `crates/mp/game/src/lib.rs`).

**Init order.** `ICARUS_Init` (`GameInterface.cpp:143-156`): `Interface_Init(
&interface_export )` populates the `I_*` table, then
`iICARUS = ICARUS_Instance::Create( &interface_export )`; NULL result →
`Com_Error( ERR_DROP )` (`:151-155`). Per-entity setup is `ICARUS_InitEnt`
(`GameInterface.cpp:646-677`): allocates that entity's `CSequencer`/
`CTaskManager` into `gSequencers[ent->s.number]` / `gTaskManagers[ent->s.number]`
(`:660-661`). Teardown: `ICARUS_FreeEnt` (`GameInterface.cpp:220-256`) →
`iICARUS->DeleteSequencer(...)` and nulls both tables (`:252-256`);
`ICARUS_Shutdown` (`GameInterface.cpp:166-…`) walks and frees.

**Init flags Raven itself uses** (ground truth for ICARUS-D2's "Raven's own
initialized flags"): the instance is live iff `iICARUS != NULL` — the NULL
guard gates `ICARUS_Init`'s error path (`:151`), `ICARUS_Shutdown` (`:202`),
and `assert( iICARUS )` in `InitEnt`/`FreeEnt` (`:649`, `:222`). A per-entity
sequencer/taskmanager is live iff `gSequencers[n]`/`gTaskManagers[n] != NULL` —
the exact flag the `ISINITIALIZED`/`MAINTAINTASKMANAGER`/`ISRUNNING` arms read
(`sv_game.cpp:756`, `:767`, `:778`) and the `RunScript`/`InitEnt`/`FreeEnt`
NULL checks use (`GameInterface.cpp:75`, `:653`, `:232`). These pointer-NULL
flags are Raven's initialized state; the port models them, not a redundant
Rust wrapper (ICARUS-D2).

**Entity-field access — the syscall arms pass an entnum, the fns need the
`sharedEntity_t`** (ground truth for ICARUS-D1's `gentity` service, ruling 19).
The C ICARUS ent-fns and task-id helpers take a `sharedEntity_t *ent` and
dereference fields on it: `ICARUS_RunScript` reads `ent->classname`/
`ent->targetname` for its verbose log (`GameInterface.cpp:129`) and indexes
`gSequencers[ent->s.number]` (`:75`); `ICARUS_InitEnt` `memset`s `ent->taskID`
to −1 (`GameInterface.cpp:664`); `ICARUS_FreeEnt`/`ICARUS_AssociateEnt`/
`ICARUS_ValidEnt` read `ent->script_targetname`/`ent->targetname`
(`:236`, `:311`, `:273`); `Q3_TaskIDPending`/`Complete`/`Set` read and write
`ent->taskID[taskType]` and `ent->s.number` (`Q3_Interface.cpp:111-183`). These
fields live on `sharedEntity_t` (`taskID[NUM_TIDS]` at
`oracle/codemp/game/g_public.h:694`, `script_targetname` `:697`, `classname`
`:703`; ported at `crates/mp/qshared/src/common/mp/qcommon/shared_entity_t.rs`).
But the `G_ICARUS_*` syscall arms pass only an **entnum** (`args[1]`,
`sv_game.cpp:739-832`); the engine turns it into the pointer through
`SV_GentityNum` — `ent = (sharedEntity_t *)((byte *)sv.gentities +
sv.gentitySize*num)` (`oracle/codemp/server/sv_game.cpp:54-59`) over the
`sv.gentities`/`sv.gentitySize` base that `G_LOCATE_GAME_DATA`
(`SV_LocateGameData`) installs (`sv_game.cpp:329-330`, arm `:566`). That
`entnum → *mut sharedEntity_t` step is exactly the `EngineHost::gentity` service
ruling 19 adds (ICARUS-D1); the affected Rust fns keep an `ent_num: i32` seam
arg and reach the entity fields through `host.gentity(ent_num)`.

**Class tree.** Closed hierarchies, intrusive `std::` containers:
- `CBlockMember` (`blockstream.h:38-105`): `{int m_id; int m_size; void*
  m_data}` — one ID/size/data record; `WriteData<T>`/`WriteDataPointer<T>`
  templates malloc via `ICARUS_Malloc`; `operator new` = `Z_Malloc(TAG_ICARUS4,
  qtrue)`.
- `CBlock` (`blockstream.h:109-154`): owns `vector<CBlockMember*> m_members`,
  `int m_id`, `unsigned char m_flags`.
- `CBlockStream` (`blockstream.h:158-196`): `.IBI` reader/writer over
  `FILE* m_fileHandle` + `char* m_stream`; `IBI_HEADER_ID "IBI"`,
  `IBI_VERSION 1.57f`.
- `CSequence` (`sequence.h:12-96`): tree node — `list<CSequence*> m_children`,
  `m_parent`, `m_return`, `list<CBlock*> m_commands`, `m_flags`, `m_iterations`,
  `m_id`; `operator new` = `Z_Malloc(TAG_ICARUS3)`.
- `CSequencer` (`sequencer.h:68-187`): per-entity driver — `list<CSequence*>
  m_sequences`, `map<CTaskGroup*,CSequence*> m_taskSequences`,
  `vector<bstream_t*> m_streamsCreated`, holds `interface_export_t* m_ie`,
  `CTaskManager* m_taskManager`, `ICARUS_Instance* m_owner`; `Run`, `Callback`,
  the `Parse*`/`Check*`/`Push/PopCommand` machinery; `operator new` =
  `Z_Malloc(TAG_ICARUS2)`. `bstream_t` (`sequencer.h:42-46`) is an intrusive
  stream-stack node.
- `CTask` (`taskmanager.h:33-58`), `CTaskGroup` (`taskmanager.h:62-93`,
  `map<int,bool> m_completedTasks`), `CTaskManager` (`taskmanager.h:97-189`):
  `map<int,CTask*>`, `map<string,CTaskGroup*>`, `map<int,CTaskGroup*>`,
  `vector<CTaskGroup*>`, `list<CTask*>`; `Update` is the per-frame heartbeat
  (`Go`), the `Rotate/Camera/Print/Sound/Move/Set/Use/…/Wait/WaitSignal`
  handlers, `RUNAWAY_LIMIT=256` (`taskmanager.h:15`).
- `ICARUS_Instance` (`instance.h:12-79`): the singleton — `list<CSequence*>`,
  `list<CSequencer*>`, `map<string,unsigned char> m_signals`, `m_interface`,
  `m_GUID`; `Signal`/`CheckSignal`/`ClearSignal` and the `virtual`
  `Save*`/`Load*` family.

**Allocator.** `ICARUS_Malloc(n)` = `Z_Malloc(n, TAG_ICARUS5, qfalse)`
(non-zeroed); `ICARUS_Free` = `Z_Free` (`Memory.cpp:8-20`,
`icarus.h:29-30`) — **not** a distinct pool; it tags into the engine Zone.
The class `operator new` overloads use `TAG_ICARUS2/3/4` with `qtrue` (zeroed).

**Script cache / variables.** `ICARUS_BufferList`
(`map<string,pscript_t*>`) caches loaded `.IBI` blobs; `ICARUS_EntList`
(`map<string,int>`) maps script-name→entnum (`GameInterface.cpp:17-18`,
`GameInterface.h:10-11`). `varStrings`/`varFloats`/`varVectors`
(`map<string,…>`, `Q3_Registers.cpp:9-11`) are the script variable store,
`MAX_VARIABLES 32`, types `VTYPE_{NONE,FLOAT,STRING,VECTOR}`
(`Q3_Registers.h:4-32`).

**Save/load is inert in MP dedicated.** `AppendToSaveGame`/`ReadFromSaveGame`
(the `I_WriteSaveData`/`I_ReadSaveData` targets) both `return 1;` with no I/O
(`Q3_Interface.cpp:695-704`). The `Save`/`Load` traversals on
`ICARUS_Instance`/`CSequencer`/`CSequence`/`CTaskManager` run but persist and
restore nothing in this build.

**Interface.cpp is dead surface.** Its `Interface_Init` is commented out
(`Interface.cpp:14-24`); the linked one is `Q3_Interface.cpp:956`. Interface.cpp
contributes no live code to the link.

## State ownership

Every file-scope global the survey found (fork-2: fields on the owning
subsystem struct, no `static mut`). Rust owner = the ICARUS subsystem struct
`mp_engine_icarus::Icarus` — the cross-cutting aggregate that names the
per-class modules below as fields. Per **ICARUS-D7** the `pub struct Icarus` is
defined in `crates/mp/engine/icarus/src/lib.rs` (the crate root), which also
declares the per-class module dirs. Per **ICARUS-D2** the aggregate attaches to
the engine as a **plain, `Default`-initialized `icarus` field directly on
`mp_engine_core::Engine`** — no `Option`, no `Box`, no nesting
(`crates/mp/engine/core/src/engine.rs:35-36`; ruling 12). "Is ICARUS
initialized?" is answered by Raven's own NULL-flags (Raven-ground-truth §Init
flags), not by wrapping the subsystem in `Option`.

| Raven global | oracle cite | Rust owner (`Icarus.field`) | constructed by | threaded via |
| --- | --- | --- | --- | --- |
| `ICARUS_Instance *iICARUS` | `GameInterface.cpp:16` | `Icarus.instance: Option<IcarusInstance>` (the `Option` mirrors Raven's own `iICARUS != NULL` flag, not a subsystem wrapper) | `ICARUS_Init` | `&mut Icarus` into ICARUS_* fns |
| `CSequencer *gSequencers[MAX_GENTITIES]` | `Instance.cpp:19`, `g_public.h:723` | `Icarus.sequencers: Box<[Option<CSequencer>; MAX_GENTITIES]>` (`Option` = Raven's per-entity NULL flag) | `ICARUS_InitEnt` | index by `ent.s.number` |
| `CTaskManager *gTaskManagers[MAX_GENTITIES]` | `Instance.cpp:20` | `Icarus.task_managers: Box<[Option<CTaskManager>; MAX_GENTITIES]>` | `ICARUS_InitEnt` | index by `ent.s.number` |
| `bufferlist_t ICARUS_BufferList` | `GameInterface.cpp:17` | `Icarus.buffer_list: HashMap<String, Pscript>` | `ICARUS_RegisterScript` | `&mut Icarus` |
| `entlist_t ICARUS_EntList` | `GameInterface.cpp:18` | `Icarus.ent_list: HashMap<String, i32>` | `ICARUS_Init`/interrogate | `&mut Icarus` |
| `int ICARUS_entFilter = -1` | `GameInterface.cpp:23` | `Icarus.ent_filter: i32` | init `= -1` | `&Icarus` |
| `interface_export_t interface_export` | `Q3_Interface.cpp:32` | `Icarus.interface_export: InterfaceExport` | `Interface_Init` | stored, borrowed by sequencers |
| `varString_m varStrings` | `Q3_Registers.cpp:9` | `Icarus.var_strings: HashMap<String, String>` | `Q3_InitVariables` | `&mut Icarus` |
| `varFloat_m varFloats` | `Q3_Registers.cpp:10` | `Icarus.var_floats: HashMap<String, f32>` | `Q3_InitVariables` | `&mut Icarus` |
| `varString_m varVectors` | `Q3_Registers.cpp:11` | `Icarus.var_vectors: HashMap<String, String>` | `Q3_InitVariables` | `&mut Icarus` |
| `int numVariables = 0` | `Q3_Registers.cpp:13` | `Icarus.num_variables: i32` | `Q3_InitVariables` (reset), inc/dec in `Q3_Declare/FreeVariable` | `&mut Icarus` |
| ICARUS Zone allocations (`TAG_ICARUS2-5`) | `Memory.cpp:12`, `blockstream.h:66` etc. | `TAG_ICARUS2/3/4` class allocs → owned Rust objects; `TAG_ICARUS5` raw blobs → owned `Vec<u8>` on their owning types (`CBlockMember::m_data`, `pscript_t::buffer`) — **no arena** (ICARUS-D3 as amended by ruling 20; `ICARUS_Malloc`/`ICARUS_Free` are §20-dropped, not ported) | object construction | ownership |

Container-internal counters (`m_GUID`, `m_count`, `m_numCommands`, group
completion maps) stay fields of their owning Rust type — not globals.

**`Icarus::default()` is hand-written, not derived.** ICARUS-D2 fixes `Icarus`
as a `Default`-initialized field on `Engine`, but `#[derive(Default)]` on
`Icarus` is both unavailable and wrong: (1) the two `Box<[Option<_>;
MAX_GENTITIES]>` slot arrays have no blanket `[T; N]: Default` impl (N>0) and
must be constructed explicitly (e.g. `Box::new(std::array::from_fn(|_| None))`),
and (2) `ent_filter` must seed `-1` — derive yields `0`, diverging from Raven's
`int ICARUS_entFilter = -1` (`GameInterface.cpp:23`). This is parity-visible,
not cosmetic: `ICARUS_entFilter` is read at two `Com_Printf`/`Q3_DebugPrint`
verbose-gate sites (`GameInterface.cpp:127`, `Q3_Interface.cpp:670`) and its
**only** writer is `Svcmd_ICARUS_f` (`GameInterface.cpp:722`) — the fn ICARUS-D6
§20-drops. So in this MP-dedicated build `ent_filter` stays `-1` for the process
lifetime; a `0` seed would flip the debug-print gating, which the engine plan's
console routing feeds into the referee syscall digest. The hand-written
`impl Default` therefore seeds `ent_filter = -1` and `interface_export` with the
real `Q3_*`/`I_*` fns (see `interface/interface_export_s.rs`); this reconciles
the lib.rs roster entry with this table without a new decision (both the `-1`
seed and the §20-drop of its writer are already settled). Seeding
`interface_export` at `Default` time is not an invented pre-init state (ruling
20): Raven's own initialized flag is `iICARUS != NULL`
(`Icarus.instance: Some(_)`), which flips **only after** `ICARUS_Init` runs
`Interface_Init(&interface_export)` and then `ICARUS_Instance::Create`
(`GameInterface.cpp:143-156`) — so no `I_*` slot is ever observed before
`Interface_Init` (re-)assigns the identical fn; the seed just satisfies Rust's
construct-before-use for a bare-`fn` table (fork-5) and preserves Raven's
Interface-Init timing.

The remaining two file-scope symbols the survey found are not mutable state:
`BSTable[]` (`GameInterface.cpp:592`, a `stringID_table_t` behavior-state
name→id lookup read at `GameInterface.cpp:627` via `GetIDForString`) is a
read-only parse table → Rust `const`/`static` per fork-2 ("const tables stay
`const`"); `tagsTable[]` (`Q3_Interface.cpp:22`) is commented-out dead surface
(Divergences) and ports nothing.

The one function-local `static` the survey found — `tempBuffer[128]` in
`CTaskManager::Get` (`TaskManager.cpp:533`), a scratch buffer whose address is
handed back through the `char **value` out-param — is hidden per-call storage,
not persistent state; per porting-rules §B3/§C7 it folds to a local returned
value (owned `String`/buffer), not an `Icarus` field.

## Seam definition

This is what freezes. Two seam directions; the `#[repr(C)]` layout types are
already ported (they don't change here).

**Host services — `EngineHost`** (ICARUS-D1, rulings 11 + 19). Per-frame the
engine services ICARUS needs (`FS_ReadFile`/`FS_FreeFile`, `Com_Printf`/
`Com_Error`, `Q_flrand`, `VM_Call(gvm, …)`, the `sv.mSharedMemory` window, and —
per ruling 19 — `gentity(ent_num) -> *mut sharedEntity_t`, the `SV_GentityNum`
dual over the `G_LOCATE_GAME_DATA` base) are reached **only** through the single
`EngineHost` trait defined in the Stage-0 game-host interface crate
(`crates/mp/host-interface`, **name TBD**;
`docs/plans/2026-07-08-mp-engine-build-out.md:250-251`, `docs/GOAL-engine.md:43-55`;
handoff ruling 11). This doc consumes that trait and does not define its method
roster or `use` path (§ Scope & non-goals — the `EngineHost` punt). Every §F fn takes `(&mut
SubsystemState, &mut impl EngineHost)` — here `SubsystemState = Icarus`. The
`gentity` service resolves the entnum an arm hands in into the entity pointer
whose `taskID`/`script_targetname`/`classname` the fn reads (Raven-ground-truth
§Entity-field access; `SV_GentityNum`, `sv_game.cpp:54-59`). `Engine` implements `EngineHost` via a split-borrow view struct so the
`&mut Icarus` field and the rest of `Engine` borrow disjointly; the referee
injects a deterministic impl. This crate declares no engine globals and calls no
`sv`/`svs`/`gvm` singletons directly — they arrive through `host`.

**Inbound — this crate's public API** (callees of `SV_GameSystemCalls`
`G_ICARUS_*` arms, `sv_game.cpp:739-832`). Uniform §F shape per ICARUS-D1
(`&mut Icarus, &mut impl EngineHost`, then the arm's args):

Body note (ruling 19): the entnum-taking fns that read entity fields —
`icarus_run_script` (`ent->classname`/`targetname`, `s.number`),
`icarus_init_ent` (`ent->taskID` memset), `icarus_free_ent`/
`icarus_associate_ent`/`icarus_valid_ent` (`ent->script_targetname`/
`targetname`), and `q3_task_id_set`/`_complete`/`_pending` (`ent->taskID[]`,
`s.number`) — keep the `ent_num: i32` seam arg and reach those fields through
`host.gentity(ent_num)`; they do **not** take a `sharedEntity_t` by value.

```rust
// game_interface — the G_ICARUS_* callees
pub fn icarus_init(icarus: &mut Icarus, host: &mut impl EngineHost);
pub fn icarus_shutdown(icarus: &mut Icarus, host: &mut impl EngineHost);
pub fn icarus_run_script(icarus: &mut Icarus, host: &mut impl EngineHost, ent_num: i32, name: &str) -> bool;
pub fn icarus_register_script(icarus: &mut Icarus, host: &mut impl EngineHost, name: &str, called_during_interrogate: bool) -> bool;
pub fn icarus_valid_ent(icarus: &mut Icarus, host: &mut impl EngineHost, ent_num: i32) -> bool;
pub fn icarus_init_ent(icarus: &mut Icarus, host: &mut impl EngineHost, ent_num: i32);
pub fn icarus_free_ent(icarus: &mut Icarus, host: &mut impl EngineHost, ent_num: i32);
pub fn icarus_associate_ent(icarus: &mut Icarus, host: &mut impl EngineHost, ent_num: i32);
pub fn icarus_is_initialized(icarus: &mut Icarus, host: &mut impl EngineHost, ent_num: i32) -> bool;   // gSequencers/gTaskManagers presence
pub fn icarus_maintain_task_manager(icarus: &mut Icarus, host: &mut impl EngineHost, ent_num: i32) -> bool; // -> CTaskManager::Update
pub fn icarus_is_running(icarus: &mut Icarus, host: &mut impl EngineHost, ent_num: i32) -> bool;
// `Svcmd_ICARUS_f` (GameInterface.h:32, GameInterface.cpp:700) is §20-dropped
// per ICARUS-D6 — zero callers, no G_ICARUS_* arm — and is intentionally absent
// from this seam. See Divergences.

// q3_registers — variable store. q3_variable_declared / q3_set_var / q3_get_*
// are the G_ICARUS_VARIABLEDECLARED / SETVAR / GET*VARIABLE arm callees.
// q3_declare_variable / q3_free_variable have NO G_ICARUS_* arm — they are the
// OUTBOUND I_DeclareVariable / I_FreeVariable interface_export targets
// (Interface_Init, Q3_Interface.cpp:1002-1003), reached only through the
// InterfaceExport table, not the server syscall switch.
pub fn q3_declare_variable(icarus: &mut Icarus, host: &mut impl EngineHost, var_type: i32, name: &str);  // I_DeclareVariable target
pub fn q3_free_variable(icarus: &mut Icarus, host: &mut impl EngineHost, name: &str);                    // I_FreeVariable target
pub fn q3_variable_declared(icarus: &mut Icarus, host: &mut impl EngineHost, name: &str) -> i32;
pub fn q3_set_var(icarus: &mut Icarus, host: &mut impl EngineHost, task_id: i32, ent_num: i32, type_name: &str, data: &str);
pub fn q3_get_float_variable(icarus: &mut Icarus, host: &mut impl EngineHost, name: &str) -> Option<f32>;   // out-param -> Option (§C7)
pub fn q3_get_string_variable(icarus: &mut Icarus, host: &mut impl EngineHost, name: &str) -> Option<String>;
pub fn q3_get_vector_variable(icarus: &mut Icarus, host: &mut impl EngineHost, name: &str) -> Option<[f32; 3]>;

// q3_interface — task-id helpers (G_ICARUS_TASKID*)
pub fn q3_task_id_set(icarus: &mut Icarus, host: &mut impl EngineHost, ent_num: i32, task_type: taskID_t, task_id: i32);
pub fn q3_task_id_complete(icarus: &mut Icarus, host: &mut impl EngineHost, ent_num: i32, task_type: taskID_t);
pub fn q3_task_id_pending(icarus: &mut Icarus, host: &mut impl EngineHost, ent_num: i32, task_type: taskID_t) -> bool;
```

**Outbound — `InterfaceExport`** (ICARUS-D8: plain Rust struct of `fn` items,
fork-5; **not** `#[repr(C)]` — it never crosses the module ABI). The `I_*`
signatures mirror `interface.h:17-70`; each impl writes a `T_G_ICARUS_*` struct
into the host's shared-memory window and issues `VM_Call(gvm, GAME_ICARUS_*)`,
both reached through the `EngineHost` passed alongside `&mut Icarus`
(ICARUS-D1). The `T_G_ICARUS_*` payloads and `GAME_ICARUS_*` ids are reused
from `mp_qshared` verbatim (already ported); this crate does not redefine them.
The **stored** `fn`-item slots' host-param type (bare `fn` cannot be generic over
`impl EngineHost`) is **ICARUS-Q9, open** — it does not change these pub seam
signatures, only the table's internal realization.

**Traps / cross-crate calls used by this crate** — all reached through
`EngineHost` (ICARUS-D1), never as direct globals: `FS_ReadFile`/`FS_FreeFile`
(`GameInterface.cpp:374`, `:393`), `Com_Printf`/`Com_Error`, `Q_flrand`
(`I_Random`, `Q3_Interface.cpp:978`), `VM_Call(gvm, …)` and the shared-memory
window (the outbound path), and `gentity(ent_num)` (`SV_GentityNum` over the
`G_LOCATE_GAME_DATA` base, ruling 19, `sv_game.cpp:54-59`, `:329-330`).
`Z_Malloc`/`Z_Free` (`Memory.cpp`) are subsumed by owned Rust objects — no arena
(ICARUS-D3 as amended by ruling 20; `ICARUS_Malloc`/`ICARUS_Free` §20-dropped) —
and are not a trap surface.

**Layout types (unchanged, already ported).** `T_G_ICARUS_*` structs +
`game_export_t` + `taskID_t` (the `q3_task_id_*` enum arg,
`oracle/codemp/game/g_public.h:625-639`, ported at
`crates/mp/qshared/src/common/mp/qcommon/task_id_t.rs`) + `sharedEntity_t` (the
`gentity` service's return target, ruling 19,
`crates/mp/qshared/src/common/mp/qcommon/shared_entity_t.rs`) — all in
`mp_qshared`;
`setType_e`, `playType_e`, `pscript_t`, `vector_t` skeletons already in
`crates/mp/engine/icarus/src/`. The existing
`#[repr(C)] interface_export_s` skeleton
(`crates/mp/engine/icarus/src/interface/interface_export_s.rs`) is a type-port
artifact **superseded** by the plain `fn`-item `InterfaceExport` under
ICARUS-D8 (ICARUS internal types are not ABI-crossing, so layout is free — §F).

## Decisions

**ICARUS-D1.** Host services reach ICARUS through **one `EngineHost` services
trait** in the Stage-0 interface crate (trace, FS, print/error, `VM_Call`,
shared memory); every §F fn takes `(&mut SubsystemState, &mut impl EngineHost)`
(here `SubsystemState = Icarus`), and `Engine` implements the trait via a
split-borrow view struct (handoff ruling 11). Because fork-2 removes the
`sv`/`svs`/`gvm` globals the `I_*` bodies and inbound `ICARUS_*` fns reach, and
a bare `fn`-item table (ICARUS-D8) cannot capture relocated state, the context
must be threaded — one shared trait keeps every subsystem's seam uniform and
lets the referee inject a deterministic impl. Rejected a per-subsystem
`IcarusHost` handle and a `&mut Engine` first parameter — ruling 11 unified all
five §F subsystems on the single `EngineHost` trait. Settles former ICARUS-Q1.
**Ruling 19 amendment (2026-07-09).** The `EngineHost` service list gains
`gentity(ent_num) -> *mut sharedEntity_t` — the `SV_GentityNum` dual over the
`G_LOCATE_GAME_DATA` base (`sv_game.cpp:54-59`, `:329-330`). This closes the
seam gap where the `q3_task_id_set`/`_complete`/`_pending` and `icarus_init_ent`/
`_free_ent`/`_valid_ent`/`_associate_ent`/`_run_script` fns — whose C forms take
a `sharedEntity_t *ent` and read `ent->taskID`/`script_targetname`/`classname`
(Raven-ground-truth §Entity-field access) — are given only an entnum by their
`G_ICARUS_*` arms. They keep the `ent_num: i32` seam arg and reach those fields
through `host.gentity(ent_num)`; no fn takes a `sharedEntity_t` by value.
Rejected passing a resolved `&mut sharedEntity_t` through the seam — the arms
carry an entnum, and the referee's deterministic `EngineHost` supplies the
gentity mapping. Settles the entity-field seam gap.

**ICARUS-D2.** ICARUS state is a **plain, `Default`-initialized `icarus` field
directly on `mp_engine_core::Engine`** — no `Option`, no `Box`, no nesting
(handoff ruling 12; `engine.rs:35-36`). Lazy-init timing is modeled with
Raven's own initialized flags (`iICARUS != NULL`, per-entity `gSequencers[n]`/
`gTaskManagers[n] != NULL`; Raven-ground-truth §Init flags), not by wrapping
the subsystem in `Option`. Because fork-2 relocated every engine global to its
owning subsystem struct and ruling 12 fixed the attachment point uniformly for
all five §F states. Rejected `Option<Icarus>`/`Box<Icarus>` lazy attachment and
`static mut` — ruling 12 chose direct fields; internal `Option`s that mirror
Raven's own NULL-able pointers (the instance, the per-entity slots) are the
faithful flag model and stay. Settles former ICARUS-Q6 / STATE-Q2 (placement
half).

**ICARUS-D3.** Owned Rust objects replace **all** ICARUS Zone allocations, and
the ICARUS arena is **dropped entirely** (handoff ruling 13, as amended by
ruling 20). Ground truth: `operator new` uses `Z_Malloc(TAG_ICARUS2/3/4, qtrue)`
(`blockstream.h:66`, `sequence.h`, `sequencer.h`) for the class instances, while
`ICARUS_Malloc` uses `Z_Malloc(TAG_ICARUS5, qfalse)` (`Memory.cpp:12`) for the
raw `WriteDataPointer` blobs. Ruling 13 owned the class instances (removing
`TAG_ICARUS2/3/4`) and originally kept a faithful arena for the residual
`TAG_ICARUS5` blobs. The oracle exhibits exactly three `TAG_ICARUS5` call-site
families (`grep ICARUS_Malloc oracle/codemp/icarus/`) — `CBlockMember::m_data`
(`WriteData`/`WriteDataPointer`, `blockstream.h:80-95`, `BlockStream.cpp:48,
87-89,111,119`), `pscript_t::buffer` (`GameInterface.cpp:389`, freed `:192`),
and the `bData` `Save`/`Load` scratch (`Sequence.cpp:490/548`,
`TaskManager.cpp:1850/1897`) — and all three take owned storage: the two
long-lived fields become owned `Vec<u8>` on their types, and the scratch folds
to a function-local `Vec<u8>` (§C7, and inert in MP anyway — Divergences). With
every `TAG_ICARUS5` user owned, **ruling 20 drops the arena** (`Icarus.arena`
does not exist) and `ICARUS_Malloc`/`ICARUS_Free` are **§20-dropped, not
ported** — they have zero live callers under the owned-buffer shape (recorded as
a module-doc note in `memory/mod.rs`). This changes no referee-diffed state: the
Zone allocations never crossed the game seam, MP save/load is inert
(`Q3_Interface.cpp:695-704`), and content parity is preserved because
`WriteDataPointer` still `memcpy`s the exact bytes into the owned `Vec<u8>`.
Rejected a private arena for the whole class tree (ruling 13 scoped it away) and
rejected keeping a vestigial arena that holds nothing (ruling 20's option (b)).
Settles former ICARUS-Q2 and ICARUS-Q7.

**ICARUS-D4.** Golden `.IBI` fixtures are **hand-authored scripts compiled once
by a `tools/ibi-gen` harness** built from the oracle's out-of-set
`Interpreter.cpp`/`Tokenizer.cpp`; the compiled blobs are committed, and **no
retail `.IBI` assets are committed** (a retail corpus may run locally,
uncommitted) (handoff ruling 14). Because §18 requires committed goldens so
`cargo test` needs no C++ toolchain, but the compiler is out of the link set and
retail blobs are not redistributable — reusing the oracle's own compiler once,
offline, on authored scripts yields byte-faithful, redistributable fixtures.
Rejected committing retail blobs and rejected hand-hexing `.IBI` by hand — the
former is non-redistributable, the latter drifts from the real format. Settles
former ICARUS-Q3.

**ICARUS-D5.** The **5 unchecked `gSequencers[ent_num]`/`gTaskManagers[ent_num]`
paths guard-and-return** per §19, with a ≤2-line note at each site (handoff
ruling 15). Ground truth: `icarus_is_initialized`/`icarus_maintain_task_manager`/
`icarus_is_running` take `ent_num` straight from `args[1]` unchecked
(`sv_game.cpp:752-782`), and `icarus_run_script`/`icarus_init_ent` index
`gSequencers[ent->s.number]` unchecked (`GameInterface.cpp:75`, `:660`); only
`ICARUS_FreeEnt` bounds-checks (`GameInterface.cpp:224-229`). In C an
out-of-range entnum is an OOB pointer read (UB); on the Rust
`[Option<_>; MAX_GENTITIES]` owner the same index panics, so §19 requires one
defined behavior. We adopt the `FreeEnt` idiom (`s.number < 0 ||
>= MAX_GENTITIES` → treat as absent/return) at all five sites — the only path
Raven itself already bounds-checks. Rejected clamp/mask and accept-the-panic —
ruling 15 chose guard-and-return. Settles former ICARUS-Q4.

**ICARUS-D6.** `Svcmd_ICARUS_f` (decl `GameInterface.h:32`, def
`GameInterface.cpp:700-730`) is **§20-dropped with a zero-caller note, not
ported** (handoff ruling 17). Ground truth: its body is entirely commented out
(a `rwwFIXMEFIXME` debug stub) and it has **zero** callers or command-table
registrations anywhere in codemp — no `G_ICARUS_*` arm exists for it
(`sv_game.cpp:739-832`), and grep across `server/`/`game/`/`qcommon/`/`cgame/`
finds only its own decl + def. §20 makes zero-caller API droppable with a
module-doc note. Rejected keeping it structurally-present — there is no
registration site to wire it to. Settles former ICARUS-Q5.

**ICARUS-D7.** The top-level `pub struct Icarus` is defined in
`crates/mp/engine/icarus/src/lib.rs` (the crate root), which also carries the
`pub mod` declarations for the new per-class module dirs (`instance/`,
`taskmanager/`, `memory/`, `q3_registers/`, `sequence/`) alongside the existing
`blockstream`/`game_interface`/`interface`/`q3_interface`/`sequencer` and the
out-of-scope `interpreter`/`tokenizer` skeletons. Because the roster otherwise
had no home for the fork-2 aggregate owner and the new module dirs need root
declarations to compile. Rejected a separate `icarus.rs` re-exported from root —
the crate root is the natural, single home for the subsystem aggregate and its
module tree (roster-hole fix).

**ICARUS-D8.** All prior settled §F conventions stand: closed C++ class
hierarchies → Rust structs / enums with owned `Vec`/`VecDeque`/`HashMap`/
`String` members (entities by index; layout free because §F internals are
non-ABI, GP2 exemplar, porting-rules §17); `interface_export_t` → a plain
`InterfaceExport` struct of `fn` items populated at `Interface_Init` (fork-5,
not `#[repr(C)]`) — the concrete host-param type of those bare `fn` items vs.
ICARUS-D1's `impl EngineHost` is **ICARUS-Q9, open**; verification = differential goldens against the unmodified
oracle TUs under `tools/icarus-oracle/` (§18, DEC-09 layer 1), MP-only with the
SP tree (`oracle/code/icarus/`) a later SP-diff pass (DEC-04/§20), not unified
now; and transcription-first — method bodies transcribe with no safety
refactoring, parity green first, refactor behind the passing diff (§A2). Because
these were settled in the first §F design session and the second session's
rulings 11–17 amend, not overturn, them. Rejected re-litigating any of them —
they are the standing §F baseline this doc builds on.

## Files roster

C++-track roster for `port-cpp-subsystem` (`designPath` consumer). All
`crate: mp_engine_icarus`, `mode: mp`. Existing type-port files are reshaped in
place under ICARUS-D3/D8; new files mirror the owning Raven header subsystem
(one class per file, porting-rules §21).

files:
- path: `crates/mp/engine/icarus/src/lib.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `Icarus`, summary: crate root — defines the fork-2 subsystem aggregate `pub struct Icarus` (fields per the State-ownership table: `instance`, `sequencers`, `task_managers`, `buffer_list`, `ent_list`, `ent_filter`, `interface_export`, `var_strings/floats/vectors`, `num_variables` — **no `arena` field**, dropped per ICARUS-D3/ruling 20) with a **hand-written `impl Default`, not `#[derive(Default)]`** (ICARUS-D2 requires `Icarus` be `Default`-constructible; derive is unavailable/wrong on three fields — see the note under State ownership): the `Box<[Option<CSequencer>; MAX_GENTITIES]>`/`Box<[Option<CTaskManager>; MAX_GENTITIES]>` slot arrays have no blanket `[T; N]: Default` impl and are built explicitly (e.g. `Box::new(std::array::from_fn(|_| None))`); `ent_filter` seeds `-1`, not derive's `0` (`ICARUS_entFilter = -1`, `GameInterface.cpp:23`; see State ownership); and `interface_export` seeds the real `Q3_*`/`I_*` fns per ICARUS-D8/ruling 20 (see `interface/interface_export_s.rs`). Adds the module declarations the roster requires — new `pub mod sequence; pub mod taskmanager; pub mod instance; pub mod memory; pub mod q3_registers;` alongside existing `blockstream`/`game_interface`/`interface`/`q3_interface`/`sequencer` (and untouched `interpreter`/`tokenizer` skeletons per Scope). `Icarus` is not a Raven class — it is the synthesized owner of every ICARUS file-scope global; it attaches to `mp_engine_core::Engine` as a plain `icarus` field per ICARUS-D2/D7.
- path: `crates/mp/engine/icarus/src/blockstream/cblock_member.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CBlockMember`, summary: one ID/size/data record; owned `Vec<u8>` data replacing `void* m_data`; `WriteMember`/`ReadMember` IBI serialization, `WriteData`/`WriteDataPointer`, `Duplicate` (`blockstream.h:38-105`, `BlockStream.cpp`). `m_data` is an owned `Vec<u8>` (settled — ICARUS-D3/ruling 20 drops the arena, so this `TAG_ICARUS5` blob is owned here); `WriteDataPointer` `memcpy`s exact bytes into it (content parity, Divergences).
- path: `crates/mp/engine/icarus/src/blockstream/cblock.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CBlock`, summary: owns `Vec<CBlockMember>`; block id + flags; `Write` overloads, `AddMember`/`GetMember`, `Duplicate` (`blockstream.h:109-154`).
- path: `crates/mp/engine/icarus/src/blockstream/cblock_stream.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CBlockStream`, summary: `.IBI` reader/writer over an owned byte buffer; `Open`/`ReadBlock`/`WriteBlock`/`BlockAvailable`, `IBI` header+version check (`blockstream.h:158-196`).
- path: `crates/mp/engine/icarus/src/blockstream/vector_t.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `vector_t`, summary: `[f32; 3]` alias used by block writes (existing skeleton, kept).
- path: `crates/mp/engine/icarus/src/blockstream/file.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CBlockStream::file`, summary: owned file-handle helper for `.IBI` I/O replacing `FILE*` (existing skeleton).
- path: `crates/mp/engine/icarus/src/sequence/csequence.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CSequence`, summary: command/child tree node; owned `Vec` children + command list, parent/return by handle; flags/iterations, `Save/Load` (inert) (`sequence.h:12-96`, `Sequence.cpp`).
- path: `crates/mp/engine/icarus/src/sequencer/csequencer.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CSequencer`, summary: per-entity script driver; parses IBI blocks into sequences; `Run`/`Callback`/`Parse*`/`Check*`/`Push/PopCommand`; holds interface + taskmanager handles (`sequencer.h:68-187`, `Sequencer.cpp`, ~43 fns). **The method↔`interface_export`/host dispatch convention (73 `m_ie->I_*` sites; whether "holds handles" or re-indexes `&mut Icarus` per call) is ICARUS-Q8, open** — signatures here are blocked until it settles.
- path: `crates/mp/engine/icarus/src/sequencer/bstream_s.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `bstream_t`, summary: internal stream-stack node; intrusive `last` pointer folds into the sequencer's owned `Vec` (existing skeleton, `sequencer.h:42-46`).
- path: `crates/mp/engine/icarus/src/taskmanager/ctask.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CTask`, summary: a scheduled task (GUID, timestamp, owned `CBlock`) (`taskmanager.h:33-58`).
- path: `crates/mp/engine/icarus/src/taskmanager/ctask_group.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CTaskGroup`, summary: completion-tracking group; `HashMap<i32,bool>` completed set, parent handle, `MarkTaskComplete`/`Complete` (`taskmanager.h:62-93`).
- path: `crates/mp/engine/icarus/src/taskmanager/ctask_manager.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CTaskManager`, summary: per-entity scheduler; `Update` heartbeat (`Go`, `RUNAWAY_LIMIT`), the `Rotate/Camera/Print/Sound/Move/Set/Use/…/Wait/WaitSignal` handlers, owned task maps/lists; `Get` scratch out-param folds to an owned return (`taskmanager.h:97-189`, `TaskManager.cpp`, ~52 fns). **The method↔`interface_export`/host dispatch convention (121 `(m_owner->GetInterface())->I_*` sites) is ICARUS-Q8, open** — signatures here are blocked until it settles.
- path: `crates/mp/engine/icarus/src/instance/icarus_instance.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `ICARUS_Instance`, summary: top singleton; owns its own sequence/sequencer pools (`m_sequences`/`m_sequencers`) + signal map (`m_signals`); `Create`/`Delete`, `Signal`/`CheckSignal`/`ClearSignal`, inert `Save*/Load*` (`instance.h:12-79`, `Instance.cpp`, ~24 fns). The per-entity `gSequencers`/`gTaskManagers` arrays are file-scope globals (`Instance.cpp:19-20`), not `ICARUS_Instance` members — they are fields of the `Icarus` aggregate per the State-ownership table (ICARUS-D2), not owned here.
- path: `crates/mp/engine/icarus/src/interface/interface_export_s.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `interface_export_t`, summary: reshape the `#[repr(C)]` type-port skeleton into the plain `fn`-item `InterfaceExport` table (ICARUS-D8) (`interface.h:17-70`). Because ICARUS-D8/fork-5 make the fields **bare `fn` items (not `Option<fn>`)**, the struct has no null/None state, yet ICARUS-D2 requires `Icarus` (hence this field) be `Default`-constructible — so its `impl Default` seeds every slot with the real crate `Q3_*`/`I_*` fn, identical to `Interface_Init`'s 1:1 assignment (`Q3_Interface.cpp:956-1008`). This is faithful, not an invented pre-init state: init order runs `Interface_Init(&interface_export)` to (re-)populate the table before `ICARUS_Instance::Create` and before any `I_*` call (`GameInterface.cpp:143-156`), so the seed is overwritten with the same fns before it is ever observed. `Interface_Init` in `q3_interface/mod.rs` remains the live wiring; the seed only satisfies Rust's construct-before-use requirement for the bare-`fn` table. **The stored slots' concrete host-param type — bare `fn` cannot be generic over ICARUS-D1's `impl EngineHost`, so `&mut dyn EngineHost` vs. a generic `InterfaceExport<H>` vs. a monomorphic handle — is ICARUS-Q9, open**; the `impl Default` seed and 1:1 `Interface_Init` wiring hold under whichever realization wins.
- path: `crates/mp/engine/icarus/src/memory/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `(none — §20 note)`, summary: **no `IcarusArena` type** — ICARUS-D3/ruling 20 drops the arena entirely (all three `TAG_ICARUS5` families are owned buffers: `CBlockMember::m_data`, `pscript_t::buffer` owned `Vec<u8>`, and `bData` `Save`/`Load` scratch folded to a local). `ICARUS_Malloc`/`ICARUS_Free` (`Memory.cpp:8-20`, `icarus.h:29-30`) have zero live callers under the owned-buffer shape and are **§20-dropped, not ported**; this file carries only the module-doc zero-caller note recording that (porting-rules §20). ICARUS-Q7 settled.
- path: `crates/mp/engine/icarus/src/q3_interface/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `Q3Interface`, summary: the `Q3_*` `I_*` implementations (shared-memory write + `VM_Call`, both via `EngineHost` per ICARUS-D1) and `Interface_Init` wiring; `Q3_Evaluate`, camera stubs, and the `Q3_TaskIDSet`/`Complete`/`Pending` task-id helpers (which read/write `ent->taskID[]`/`s.number` via `host.gentity(ent_num)` per ICARUS-D1/ruling 19) (`Q3_Interface.cpp`, ~49 fns). **The `I_*` slot fns' host-param type (the `InterfaceExport` table's stored `fn` items) is ICARUS-Q9, open; their reach into `Icarus` fields under an internal call is ICARUS-Q8, open.**
- path: `crates/mp/engine/icarus/src/q3_interface/set_type_t.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `setType_e`, summary: SET_* enum (existing skeleton, kept).
- path: `crates/mp/engine/icarus/src/q3_interface/play_type_t.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `playType_e`, summary: PLAY_* enum (existing skeleton, kept).
- path: `crates/mp/engine/icarus/src/q3_registers/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `Q3Registers`, summary: `varStrings`/`varFloats`/`varVectors` stores + `Q3_InitVariables`/`Q3_Declare/Free/Get/Set*Variable`, `MAX_VARIABLES`, `VTYPE_*`; `Q3_DebugPrint` reaches `Com_Printf`/`com_developer` via `EngineHost` (ICARUS-D1) (`Q3_Registers.cpp`, `.h:4-32`, ~16 fns).
- path: `crates/mp/engine/icarus/src/game_interface/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `GameInterface`, summary: `ICARUS_RunScript`/`RegisterScript`/`GetScript`/`Init`/`InitEnt`/`FreeEnt`/`ValidEnt`/`AssociateEnt`/`Shutdown`/`LinkEntity`/`SoundPrecache`/`InterrogateScript` + the buffer/ent-list state; the 5 unchecked entnum paths guard-and-return per ICARUS-D5; `Svcmd_ICARUS_f` is §20-dropped per ICARUS-D6 (zero-caller module-doc note, not ported); `RunScript`/`InitEnt`/`FreeEnt`/`ValidEnt`/`AssociateEnt` reach `ent->classname`/`taskID`/`script_targetname`/`targetname` via `host.gentity(ent_num)` per ICARUS-D1/ruling 19 (`GameInterface.cpp`, ~14 live fns).
- path: `crates/mp/engine/icarus/src/game_interface/pscript_s.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `pscript_t`, summary: cached script record; `buffer` is an owned `Vec<u8>` replacing `char* buffer` (settled — ICARUS-D3/ruling 20 drops the arena, so this `TAG_ICARUS5` blob is owned here) (existing skeleton, `GameInterface.h:4-8`).

**Module-declaration files (boilerplate, not classes).** Each one-class-per-file
directory (porting-rules §21) needs a `mod.rs` that only `pub mod`s its per-class
files; these carry no logic and make no decision — they realize the module tree
ICARUS-D7 fixes. Listed for roster completeness so the skeleton compiles
(`blockstream/mod.rs`, `sequencer/mod.rs`, and `interface/mod.rs` already exist on
disk from the type port; `sequence/mod.rs`, `taskmanager/mod.rs`, `instance/mod.rs`
are new dirs under ICARUS-D7). `game_interface/mod.rs`, `q3_interface/mod.rs`,
`q3_registers/mod.rs`, and `memory/mod.rs` are **not** here — they are the
class-bearing units already rostered above (`GameInterface`/`Q3Interface`/
`Q3Registers`/§20-note), not declaration-only files.
- path: `crates/mp/engine/icarus/src/blockstream/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `(mod decl)`, summary: `pub mod cblock_member; pub mod cblock; pub mod cblock_stream; pub mod vector_t; pub mod file;` — declaration-only (existing skeleton, extend for the reshaped roster).
- path: `crates/mp/engine/icarus/src/sequence/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `(mod decl)`, summary: `pub mod csequence;` — declaration-only (new dir, ICARUS-D7).
- path: `crates/mp/engine/icarus/src/sequencer/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `(mod decl)`, summary: `pub mod csequencer; pub mod bstream_s;` — declaration-only (existing skeleton, extend for the reshaped roster).
- path: `crates/mp/engine/icarus/src/taskmanager/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `(mod decl)`, summary: `pub mod ctask; pub mod ctask_group; pub mod ctask_manager;` — declaration-only (new dir, ICARUS-D7).
- path: `crates/mp/engine/icarus/src/instance/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `(mod decl)`, summary: `pub mod icarus_instance;` — declaration-only (new dir, ICARUS-D7).
- path: `crates/mp/engine/icarus/src/interface/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `(mod decl)`, summary: `pub mod interface_export_s;` — declaration-only (existing skeleton, kept).

`Interface.cpp` produces **no** Rust file — its `Interface_Init` is commented
out (dead surface, see Divergences); the live `Interface_Init` is in
`q3_interface/mod.rs`.

## Divergences

Raven-UB / dead-surface points where the port picks one defined behavior
(porting-rules §19; kept out of / normalized in the shared golden corpus).
`port-cpp-subsystem` `divergences`:

- MP save/load is inert: `AppendToSaveGame`/`ReadFromSaveGame` (`I_WriteSaveData`/`I_ReadSaveData` targets) both `return 1;` with no I/O (`Q3_Interface.cpp:695-704`); the `ICARUS_Instance`/`CSequencer`/`CSequence`/`CTaskManager` `Save`/`Load` methods port as structurally-present but effect-free, and are excluded from the golden corpus.
- `Interface.cpp`'s `Interface_Init` is commented out (`Interface.cpp:14-24`); it links no code. The port emits no Rust unit for it — the live table wiring lives in `q3_interface/mod.rs` (`Q3_Interface.cpp:956`).
- Out-of-range entnum on the 5 unchecked `gSequencers`/`gTaskManagers` paths → **guard-and-return** (resolved per ICARUS-D5, ruling 15). The `ISINITIALIZED`/`MAINTAINTASKMANAGER`/`ISRUNNING` arms index `gSequencers[entID]`/`gTaskManagers[entID]` with `entID = args[1]` unchecked (`sv_game.cpp:752-782`), and `ICARUS_RunScript`/`ICARUS_InitEnt` index `gSequencers[ent->s.number]` unchecked (`GameInterface.cpp:75`, `:660`); only `ICARUS_FreeEnt` guards `s.number >= MAX_GENTITIES || < 0` (`GameInterface.cpp:224-229`). In C a negative/`>= MAX_GENTITIES` entnum is an OOB pointer read (UB); on the Rust `[Option<_>; MAX_GENTITIES]` arrays each of the five ports the `FreeEnt` bounds-check and returns the "absent" result (`false`/no-op), with a ≤2-line §19 note per site. Excluded from / normalized in the shared corpus.
- `ICARUS_Malloc` is non-zeroed (`Z_Malloc(...,qfalse)`, `Memory.cpp:12`) while class `operator new` is zeroed (`Z_Malloc(...,qtrue)`, `blockstream.h:66`); under ICARUS-D3/ruling 20 both class instances and the `TAG_ICARUS5` blobs (`CBlockMember::m_data`, `pscript_t::buffer`) are owned Rust objects/`Vec<u8>`, always value-initialized, so the zero/non-zero distinction collapses to defined initialization — `WriteDataPointer` still `memcpy`s exact bytes, preserving content parity.
- `ICARUS_Malloc`/`ICARUS_Free` (`Memory.cpp:8-20`, `icarus.h:29-30`) are §20-dropped per ICARUS-D3/ruling 20: with every `TAG_ICARUS5` user owned, the arena is dropped entirely and these two fns have zero live callers. Recorded with a module-doc zero-caller note in `memory/mod.rs`, not ported (`Icarus.arena` does not exist).
- `Svcmd_ICARUS_f` (`GameInterface.h:32`, `GameInterface.cpp:700-730`) is §20-dropped per ICARUS-D6: commented-out body, zero callers/registrations, no `G_ICARUS_*` arm. Recorded with a module-doc zero-caller note, not ported.

Scout/harness phases may surface further §19 sites; each gets a ≤2-line
site note at the port, per porting-rules §19.

## Verification strategy

DEC-09 layer 1, porting-rules §F/§18. `tools/icarus-oracle/` compiles the
unmodified oracle TUs standalone under stub headers (GP2/`tools/gp2-oracle`
pattern) and golden-diffs canonical dumps against the Rust port; goldens are
committed so `cargo test` needs no C++ toolchain. Gate: GOAL-engine M2
("icarus differential goldens"). Skeleton build/checkpoint semantics per DEC-10.

Standalone-diffable units and their canonical dumps:
- **BlockStream** — round-trip a corpus of `.IBI` blobs: `ReadBlock` →
  re-`WriteBlock` → byte-identical; dump the parsed `(id, member-type, size,
  bytes)` record stream. Cleanest TU (its only allocation is the
  `CBlockMember::m_data` owned `Vec<u8>` that replaces the `TAG_ICARUS5` blob —
  ICARUS-D3/ruling 20; no arena/`Z_Malloc` dep).
- **Q3_Registers** — script variable operations: dump `varStrings/varFloats/
  varVectors` state after a scripted `Declare/Set/Get/Free` sequence.
- **Sequencer + TaskManager + Instance (end-to-end)** — drive
  `ICARUS_RunScript` on a committed `.IBI` fixture with a **mock**
  `EngineHost`/`interface_export`/`VM_Call` that records the ordered `I_*` /
  `GAME_ICARUS_*` callback trace and scripted return values; the golden is that
  ordered callback stream plus final variable/signal state. This exercises
  `Parse*`/`Check*`, task scheduling, `Wait`/`WaitSignal`, and `Callback`.

Fixture provenance (ICARUS-D4, ruling 14): the committed `.IBI` goldens are
hand-authored scripts compiled **once** by a `tools/ibi-gen` harness built from
the oracle's out-of-set `Interpreter.cpp`/`Tokenizer.cpp`; the compiled blobs
are committed, **no** retail `.IBI` assets are committed, and a retail corpus
may run locally, uncommitted.

Live-peer acceptance (DEC-09 layer 2) is the server-crate/full-engine gate
(M4-M5), not this subsystem's unit gate.

## Slice hooks

- **M2 (waves 7-12), GOAL-engine** — "icarus complete … icarus differential
  goldens." The § Seam definition pub API is frozen on the `EngineHost` shape
  (ICARUS-D1) and the `tools/icarus-oracle` harness / `tools/ibi-gen` fixture
  corpus (ICARUS-D4) are the remaining build items — but transcription of the
  `CSequencer`/`CTaskManager`/`InterfaceExport` internals is **gated on
  ICARUS-Q8/Q9** (the internal interface/host dispatch convention), which must
  settle in a design session before the skeleton compiles.
- **Stage-0 interface crate** (`crates/mp/host-interface`, **name TBD**;
  `docs/plans/2026-07-08-mp-engine-build-out.md:250-251`) — defines the
  `EngineHost` trait (crate path + full method roster) this crate's seam threads
  (ICARUS-D1); this doc consumes it, does not define it (§ Scope & non-goals).
  Lands before ICARUS in the port order, so no ICARUS `use` line for the trait is
  finalizable until it lands.
- **Server crate `SV_GameSystemCalls`** — the `G_ICARUS_*` dispatch arms
  (`sv_game.cpp:739-832`) call this crate's inbound public API; that switch
  ports in the server slice and depends on the inbound signatures + the
  `EngineHost` shape here. The outbound `GAME_ICARUS_*` handlers already exist
  in `mp_game`.
- **Engine aggregate** — `mp_engine_core::Engine` gains the plain `icarus`
  field (ICARUS-D2) and the split-borrow view struct that implements
  `EngineHost` (ICARUS-D1) — including its `gentity(ent_num)` service
  (ruling 19); `crates/mp/engine/core/src/engine.rs:35-36`.
- **Memory/Zone** — no dependency: ICARUS-D3/ruling 20 drops the arena and
  §20-drops `ICARUS_Malloc`/`ICARUS_Free`, so ICARUS owns all its storage and
  needs no engine-Zone hook.
- **`G_LOCATE_GAME_DATA`/`SV_GentityNum`** (server crate) — installs the
  `sv.gentities`/`sv.gentitySize` base (`sv_game.cpp:329-330`) that the
  `EngineHost::gentity` service reads (ICARUS-D1/ruling 19); lands with the
  server slice, before ICARUS's entnum-taking fns run live.

## Open questions

MUST be empty at FROZEN. Q1–Q7 are resolved (Q1–Q6 by the second §F
design-session rulings, handoff ledger `:114-158`; Q7 by the final-revision
rulings 19 + 20, 2026-07-09). **Two items reopen the doc**: ICARUS-Q8 and
ICARUS-Q9, both **surfaced by the second dry-run gate (2026-07-09)** as forks a
porter hits that no settled decision covers. They are **escalated to a design
session, not self-resolved** (doc-standards §Open questions); the doc stays
DRAFT until they settle. Each is an *internal-shape* fork (§F internals are
non-ABI), so neither disturbs the frozen § Seam definition pub signatures — but
each blocks writing the skeleton those signatures front.

- **ICARUS-Q1** (host/context threading) → resolved by **ICARUS-D1** (ruling 11:
  one `EngineHost` trait, `(&mut Icarus, &mut impl EngineHost)`; ruling 19 adds
  the `gentity(ent_num)` service that closes the entity-field seam gap).
- **ICARUS-Q2** (arena scope vs. class allocs) → resolved by **ICARUS-D3**
  (ruling 13: owned objects for `TAG_ICARUS2/3/4`; ruling 20 then drops the
  arena for `TAG_ICARUS5` too — see Q7).
- **ICARUS-Q3** (golden-fixture provenance) → resolved by **ICARUS-D4**
  (ruling 14: hand-authored, `tools/ibi-gen`-compiled, no retail blobs).
- **ICARUS-Q4** (out-of-range entnum §19 fork) → resolved by **ICARUS-D5**
  (ruling 15: guard-and-return at all 5 sites).
- **ICARUS-Q5** (`Svcmd_ICARUS_f` drop-vs-keep §20 fork) → resolved by
  **ICARUS-D6** (ruling 17: §20-drop, zero-caller note).
- **ICARUS-Q6 / STATE-Q2** (`Icarus` attachment to the `Engine` island) →
  resolved by **ICARUS-D2** (ruling 12: plain `Default`-initialized `icarus`
  field directly on `Engine`).
- **ICARUS-Q7** (residual `TAG_ICARUS5` blob home — arena handle vs. owned
  buffer — and the fate of `ICARUS_Malloc`/`ICARUS_Free`) → **resolved by
  ICARUS-D3 as amended by ruling 20** (2026-07-09): option (b) — the Icarus
  arena is **dropped entirely**. All three `TAG_ICARUS5` families take owned
  storage (`CBlockMember::m_data` and `pscript_t::buffer` owned `Vec<u8>`; the
  `bData` `Save`/`Load` scratch folds to a function-local `Vec<u8>`, inert in MP
  anyway), so `Icarus.arena` does not exist and `ICARUS_Malloc`/`ICARUS_Free`
  are §20-dropped with a `memory/mod.rs` zero-caller note. No handle
  representation is needed, and the BlockStream first-TU blocker is cleared:
  `cblock_member.rs`'s `m_data` is simply an owned `Vec<u8>` (State ownership,
  ICARUS-D3, Files roster).

- **ICARUS-Q8** (internal interface/host dispatch convention for `CSequencer`/
  `CTaskManager` — **new, 2026-07-09 dry-run gate; escalated**). The
  State-ownership table stores `sequencers`, `task_managers`, and
  `interface_export` as **sibling fields on the same `Icarus`**. Internally,
  `CSequencer` methods call `m_ie->I_*` **73×** (`Sequencer.cpp`, grep) and
  `CTaskManager` methods call `(m_owner->GetInterface())->I_*` **121×**
  (`TaskManager.cpp`, grep) — ~194 sites across the two largest files (of ~253
  fns). No settled decision says how such a call is realized in Rust: ICARUS-D1
  fixes only the **seam** fn shape (`(&mut Icarus, &mut impl EngineHost)`) for the
  ~19 inbound entry points, and ICARUS-D8's "transcription-first" is §A2
  (behavioral parity), which does **not** settle the borrow shape. The conflict:
  an internal method that holds `&mut icarus.sequencers[n]` (or a `&mut CSequencer`
  drawn from it) while invoking an `I_*` that itself needs `&mut Icarus` — e.g.
  `I_DeclareVariable`/`I_GetFloat` reaching `var_strings`/`var_floats`/
  `var_vectors` (State ownership) — is **two live `&mut Icarus` borrows**, which
  the storage shape creates but no decision resolves. porting-rules §17 requires
  exactly this sibling-pointer-walking shape be settled *before* transcription; it
  is not. Candidate resolutions the dry-run surfaced (**pick one in session, do
  not self-resolve**): (a) `&mut self`-free **re-index-per-call** free functions
  `fn(icarus: &mut Icarus, host: &mut …, ent_num: i32, …)` that re-derive disjoint
  field borrows on each call and never hold a persistent `&mut CSequencer`/
  `&mut CTaskManager` across an `I_*` call; (b) **take-and-restore** (move the slot
  out of the `[Option<_>; N]` array, operate, put it back); (c) `Rc<RefCell<…>>`
  handles. The roster's `csequencer.rs` phrase "holds interface + taskmanager
  handles" is in tension with (a) and must be reconciled by the winning option.
  Blocks writing the `csequencer.rs`/`ctask_manager.rs`/`q3_interface` skeletons.

- **ICARUS-Q9** (host-param type of the stored `InterfaceExport` `fn` items —
  **new, 2026-07-09 dry-run gate; escalated**). ICARUS-D8/fork-5 fix
  `InterfaceExport` as a **plain struct of bare `fn` items**; ICARUS-D1 types
  every §F fn (including the `I_*` targets these fields point at) with
  `&mut impl EngineHost`. A bare `fn`-pointer field **cannot** be generic over
  `impl EngineHost` — a stored `fn` item is a concrete type — so the table's slots
  must take a **monomorphic** host, and no settled decision picks which. Candidate
  resolutions (**pick one in session, do not self-resolve**): (a) `&mut dyn
  EngineHost` trait-object slots (dynamic dispatch); (b) a generic
  `InterfaceExport<H: EngineHost>` whose fields are `fn(&mut Icarus, &mut H, …)`;
  (c) a monomorphic concrete host handle. This is orthogonal to but interacts with
  ICARUS-Q8 (both concern how internal calls reach the host/interface). The pub
  § Seam definition signatures keep `&mut impl EngineHost` unchanged — Q9 scopes
  **only** the stored-`fn`-table realization (`interface/interface_export_s.rs`,
  the `Q3_*`/`I_*` wiring in `q3_interface/mod.rs`). Blocks writing that table's
  type and its ~49 `Q3_*` `I_*` slot fns to the letter of both ICARUS-D8 and
  ICARUS-D1 simultaneously.
