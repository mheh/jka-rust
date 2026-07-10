# ICARUS sequencer — MP engine (§F idiomatic reimplementation) Design
Status: DRAFT     Supersedes: none
Decision prefix: ICARUS     Ledger deps: DEC-09, DEC-10; engine-fork-discovery forks 2, 4, 5, 7 + §F rulings 11–15, 17, 19, 20, 23, 24, 27, 31, 32, 33; GOAL-engine M2 gate

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
  named); and the §F doc-session **rulings 11–15, 17, 19, 20, 23, 24, 27, 31, 32,
  33** (user, 2026-07-09) that settle this doc's open questions. **Ruling 27**
  (ledger `:250-262`) closes the last two open items — **ICARUS-Q10/Q11** — with
  faithful `Vec` arenas + id newtypes (ICARUS-D11): `IcarusInstance` owns
  `Vec<Sequence>`/`Vec<Sequencer>`, `SequenceId(i32)`/`SequencerId(i32)` carry
  Raven's monotonic never-reused `m_GUID`, and `TaskManager` collapses its three
  parallel `CTaskGroup*` indexes to one `Vec<TaskGroup>` owner + `BTreeMap`
  side-indexes of ids (rulings 21/22 precedent). **Rulings 31 + 33** (ledger
  `:296-347`) build the Stage-0 crate as real compiled code *before* this
  relaunch: `crates/mp/host-interface` (package `mp_host_interface`, commit
  `4b7f01b0`) is green, so § Seam definition now quotes the crate's **actual**
  `EngineHost` signatures verbatim from
  `crates/mp/host-interface/src/engine_host.rs`, not a paper spec (ICARUS-D12);
  ruling 32's fixture-backed `MockHost`
  (`crates/mp/host-interface/src/mock.rs`) is the goldens vehicle
  (§ Verification strategy). **Ruling 23 corrects
  ruling 19's premise**: the entity-field `G_ICARUS_*` arms pass a
  `sharedEntity_t *` pointer, not an entnum (ICARUS-D1, rewritten); the
  `gentity()` EngineHost service survives only for the genuinely index-based
  `SV_GentityNum(ent->s.number)` access inside `ICARUS_ValidEnt` (its behaviorSet
write-back, `GameInterface.cpp:288`/`:291`). **Ruling 24**
  settles the two dry-run holes: the ~194 internal `I_*` dispatch sites and the
  `InterfaceExport` slots resolve on `&mut dyn EngineHost` free fns (ICARUS-Q8/Q9
  → ICARUS-D9/D10), and pins the Stage-0 crate as `crates/mp/host-interface`
  (package `mp_host_interface`). Ruling 20 (Icarus arena dropped entirely,
  ICARUS-D3) stands (ledger `:114-176`).
- `docs/GOAL-engine.md` — Stage-0 game-host interface crate checklist (`:43-55`,
  an unchecked prerequisite bullet — it names the crate's *role*), M2 gate
  ("icarus differential goldens"). Ruling 24 **pins** the crate's importable path
  as `crates/mp/host-interface`, package `mp_host_interface` (`use
  mp_host_interface::EngineHost;`); the full `EngineHost` method roster remains
  owned by the Stage-0 interface-crate design
  (`docs/plans/2026-07-08-mp-engine-build-out.md:250`, handoff ruling 11), but the
  method surface ICARUS consumes is now listed as Rust signatures in § Seam
  definition (ruling 24) — see also ICARUS-D1/D10.
- GP2 exemplar: `crates/mp/engine/qcommon/src/gp2/`, `tools/gp2-oracle/`.

## Scope & non-goals

**In scope.** The 10 `WinDed.vcproj`-Release-linked ICARUS sources
(`oracle/codemp/icarus/`): `BlockStream.cpp`, `GameInterface.cpp`,
`Instance.cpp`, `Interface.cpp`, `Memory.cpp`, `Q3_Interface.cpp`,
`Q3_Registers.cpp`, `Sequence.cpp`, `Sequencer.cpp`, `TaskManager.cpp` —
253 fns / 6,749 LOC (plan §graph row). Every method ports except the
zero-caller fns §20-drops (§F: only zero-caller API is droppable per §20) —
`Svcmd_ICARUS_f` (ICARUS-D6), `ICARUS_Malloc`/`ICARUS_Free` (ICARUS-D3 as
amended by ruling 20, the arena being dropped), and the **two BlockStream
duplicators** `CBlock::Duplicate` (`blockstream.h:138`, `BlockStream.cpp:359`) +
`CBlockMember::Duplicate` (`blockstream.h:74`, `BlockStream.cpp:148`) —
zero callers **anywhere** in `oracle/codemp/icarus/` (grep: only `CBlock::Duplicate`'s
own internal member-`Duplicate` self-call at `BlockStream.cpp:374` + a `"Duplicate
symbol"` string literal in `Tokenizer.cpp:46`; not in `Interpreter.cpp`), and not
exercised by the BlockStream verification round-trip (§ Verification, which uses
`ReadBlock`→`WriteBlock`, not `Duplicate`) — §20-dropped with a module-doc note
(ICARUS-D13). Target crate: `mp_engine_icarus` (MP only).

**BlockStream writer surface is retained, not dead (ICARUS-D13).** The remaining
`.IBI`-writer methods — `CBlockStream::Create(char *)` (the `fopen("wb")` file
writer, `blockstream.h:167`), `CBlockStream::WriteBlock` (`:174`),
`CBlockMember::WriteMember` (`:47`), `CBlockMember::WriteData`/`WriteDataPointer`
(`:76`/`:88`), and `CBlock::Write` overloads (`:125-129`) — have their only
**production** callers in the excluded compiler TUs (`Interpreter.cpp`/
`Tokenizer.cpp`, the offline `.txt`→`.IBI` path), yet are **ported, not dropped**:
settled decisions reference them as in-scope surface — the § Verification round-trip
exercises `ReadBlock`→re-`WriteBlock`→`WriteMember` byte-identical, and ICARUS-D3
preserves `WriteDataPointer`'s exact-byte `memcpy` for content parity. So a porter
ports them per the Files roster; only the two caller-less `Duplicate` methods drop.
(`CBlockStream::Init`, `blockstream.h:165`, is **live** — `CBlockStream::Open` calls
it at `BlockStream.cpp:670` — and is not a drop candidate.)

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
doc pins the switch. **Flag for the server-dispatch doc:** the three `G_ICARUS_TASKID*`
arms pass `(taskID_t)args[2]` — an unchecked int→enum cast of an arbitrary
`intptr_t` (`sv_game.cpp:786`, `:808`, `:813`). Constructing a `taskID_t` (repr
enum) from an out-of-range `args[2]` is Raven UB the server crate must resolve as a
porting-rules §19 checked conversion **before** it calls this crate's `task_type:
taskID_t` callees; Raven's own callee guard (`taskType < TID_CHAN_VOICE || taskType
>= NUM_TIDS` → no-op/`qfalse`, `Q3_Interface.cpp:116-118`/`:136-138`/`:169-171`) pins
the out-of-range outcome, so the server-side pick is not open-ended (see § Seam
definition task-id note, Divergences). Frame driving (per-entity
`CTaskManager::Update`) is the game module's `G_ICARUS_MAINTAINTASKMANAGER` trap each
frame (`sv_game.cpp:763-773`); this doc owns `Update`, the server owns the trap arm.

**The Stage-0 `EngineHost` crate now EXISTS as compiled code (rulings 31 + 33,
ICARUS-D12); its full method roster and the surface ICARUS consumes are pinned
here.** The trait lives in the built, green Stage-0 crate `crates/mp/host-interface`,
package `mp_host_interface` (commit `4b7f01b0`; `use mp_host_interface::EngineHost;`;
ruling 24 pinned the path, ruling 31 built it before this relaunch;
`docs/plans/2026-07-08-mp-engine-build-out.md:250-251`, handoff ruling 11) — a
crate of Rust traits transcribing the C seam (syscall surface, vmcall driver,
shared-memory contract). Because the crate is real code, § Seam definition quotes
its **actual** `EngineHost` signatures verbatim from
`crates/mp/host-interface/src/engine_host.rs` (ruling 33's "no deferrals" surface:
`trace`, `fs_read_file`/`fs_free_file`, `print`/`error`, `vm_call(VmSlot, …)`,
`shared_memory() -> *mut c_char`, `flrand`/`irand`, `gentity`); ICARUS binds a
subset of that already-frozen roster, not a paper spec. Every §F fn takes `(&mut
Icarus, &mut dyn EngineHost)` (ICARUS-D1/D9/D10 — `dyn`, not `impl`, forced by the
fn-pointer `InterfaceExport` table, ICARUS-Q9 → ICARUS-D10). Porters `use` the
crate directly (it precedes ICARUS in the port order, § Slice hooks), never a
stub. This mirrors the sibling §F docs, which reach the same crate
(`docs/subsystems/roff.md` non-goals, `docs/subsystems/ghoul2-server.md`
non-goals).

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

**Entity-field access — the entity-field arms carry the `sharedEntity_t *`
pointer; only the three presence-check arms carry an int entnum** (ground truth
for ICARUS-D1 as corrected by ruling 23; **supersedes the ruling-19 premise**,
which wrongly claimed the arms pass an entnum resolved through `SV_GentityNum`).
The C ICARUS ent-fns and task-id helpers take a `sharedEntity_t *ent` and
dereference fields on it: `ICARUS_RunScript` reads `ent->classname`/
`ent->targetname` for its verbose log (`GameInterface.cpp:129`) and indexes
`gSequencers[ent->s.number]` (`:75`); `ICARUS_InitEnt` walks `ent->s.number`/
`ent->taskID` (`GameInterface.cpp:178-181`, `:660`); `ICARUS_FreeEnt`/
`ICARUS_AssociateEnt`/`ICARUS_ValidEnt` read `ent->script_targetname`/
`ent->targetname` (`:236`, `:311`, `:273`); `Q3_TaskIDPending`/`Complete`/`Set`
read and write `ent->taskID[taskType]` and `ent->s.number`
(`Q3_Interface.cpp:111-183`). These fields live on `sharedEntity_t`
(`taskID[NUM_TIDS]` at `oracle/codemp/game/g_public.h:694`, `script_targetname`
`:697`, `classname` `:703`; ported at
`crates/mp/qshared/src/common/mp/qcommon/shared_entity_t.rs`).

**The `G_ICARUS_*` arms hand these fns the pointer directly, not an entnum**
(`sv_game.cpp:739-832`). The five ent-fns get `ConvertedEntity((sharedEntity_t
*)VMA(1))` — `ICARUS_RunScript` (`:740`), `ICARUS_ValidEnt` (`:750`),
`ICARUS_InitEnt` (`:789`), `ICARUS_FreeEnt` (`:793`), `ICARUS_AssociateEnt`
(`:797`); the three task-id helpers get a bare cast `(sharedEntity_t *)VMA(1)` —
`Q3_TaskIDPending` (`:786`), `Q3_TaskIDSet` (`:808`), `Q3_TaskIDComplete`
(`:813`). `ConvertedEntity` (`sv_game.cpp:422-451`) copies `ent->s`/`ent->r`/
`ent->taskID` into a file-static `gLocalModifier` and re-points its string/parms
fields through `VM_ArgPtr(...)` (`:444-448`), returning `&gLocalModifier` — a
VM-address shuffle whose `VM_ArgPtr` is **identity in the native-dylib model** (no
VM heap to relocate against), so `ConvertedEntity` is effectively a pass-through
of the same entity pointer; the port carries `*mut sharedEntity_t` straight
through the seam and need not replicate the shuffle (Divergences). Only the three
**presence-check** arms carry an int: `G_ICARUS_ISINITIALIZED` (`:752`),
`G_ICARUS_MAINTAINTASKMANAGER` (`:763`), `G_ICARUS_ISRUNNING` (`:774`) each read
`int entID = args[1]` and index `gSequencers[entID]`/`gTaskManagers[entID]`
inline (`:754-782`).

**The `gentity()` service survives for genuinely index-based access only.** The
sole ICARUS site that turns a number back into a pointer is `ICARUS_ValidEnt`
(`GameInterface.cpp:268-297`), which — for an entity that carries a `behaviorSet`
but no `script_targetname` (`:277-294`), and because `ConvertedEntity` handed it a
shuffled copy whose pointer fields cannot be written back into the real VM entity
(`:284-290`) — reaches the true entity through `SV_GentityNum(ent->s.number)`
(`:288`) to assign `trueEntity->script_targetname = trueEntity->targetname`
(`:291`). That `s.number → *mut sharedEntity_t` step is the `EngineHost::gentity`
service (`SV_GentityNum`, `sv_game.cpp:54-59`, over the `G_LOCATE_GAME_DATA` base
`SV_LocateGameData` installs, `:329-330`, arm `:567`). `ICARUS_AssociateEnt`
(`:307-318`) is **not** such a site: it only reads `ent->s.number` into
`ICARUS_EntList[…]` (`:317`) — a read of the seam pointer, no write-through, so no
`gentity` — and neither does any other ICARUS fn; the rest receive the pointer at
the seam (ICARUS-D1).

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
| `ICARUS_Instance *iICARUS` | `GameInterface.cpp:16` | `Icarus.instance: Option<IcarusInstance>` (the `Option` mirrors Raven's own `iICARUS != NULL` flag, not a subsystem wrapper). Per **ICARUS-D11** (ruling 27) `IcarusInstance` **owns** `sequences: Vec<Sequence>` + `sequencers: Vec<Sequencer>` — its Raven `m_sequences`/`m_sequencers` lists (`instance.h:62-63`) — keyed by `SequenceId`/`SequencerId` newtypes carrying `m_GUID` | `ICARUS_Init` | `&mut Icarus` into ICARUS_* fns |
| `CSequencer *gSequencers[MAX_GENTITIES]` | `Instance.cpp:19`, `g_public.h:723` | `Icarus.sequencers: Box<[Option<SequencerId>; MAX_GENTITIES]>` — **non-owning** per-entity index into `IcarusInstance.sequencers` (ICARUS-D11, ruling 27; the object is owned by the instance arena, `GetSequencer` `STL_INSERT( m_sequencers, … )` `Instance.cpp:174`; `Option`/`None` = Raven's per-entity NULL flag) | `ICARUS_InitEnt` | index by `ent.s.number` → `SequencerId` |
| `CTaskManager *gTaskManagers[MAX_GENTITIES]` | `Instance.cpp:20` | `Icarus.task_managers: Box<[Option<CTaskManager>; MAX_GENTITIES]>` — **owning** (ruling 27 leaves task-manager objects here; Raven inserts them in no instance list — `GetSequencer` creates one per sequencer, `Instance.cpp:166-182` — and ICARUS-D9 stores no sequencer→taskmanager handle, so they are reached by ent index) | `ICARUS_InitEnt` | index by `ent.s.number` |
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

**Host services — `EngineHost`** (ICARUS-D1, rulings 11 + 23 + 24 + 31 + 33). Every
engine service ICARUS needs is reached **only** through the single `EngineHost`
trait in the built Stage-0 crate `crates/mp/host-interface`, package
`mp_host_interface` (ruling 24 pinned the path, ruling 31 built it, commit
`4b7f01b0`; `docs/plans/2026-07-08-mp-engine-build-out.md:250-251`,
`docs/GOAL-engine.md:43-55`; handoff ruling 11). A porter imports it verbatim:

```rust
use mp_host_interface::EngineHost;
```

Because the crate is real compiled code (rulings 31 + 33), the roster below is the
trait's **actual** signature set, quoted **verbatim** from
`crates/mp/host-interface/src/engine_host.rs:23-106` (ruling 33's "no deferrals"
surface — do not re-derive it). ICARUS binds the annotated subset; the rest are
listed because the trait is one shared surface across all five §F subsystems:

```rust
// VERBATIM from crates/mp/host-interface/src/engine_host.rs (rulings 31 + 33).
// `// ICARUS:` annotations mark this crate's use; unmarked methods serve sibling §F subsystems.
pub trait EngineHost {
    // SV_Trace, out-param result (sv_world.cpp:803). ICARUS binds no trace site
    // (roster continuity across the five §F subsystems).
    #[allow(clippy::too_many_arguments)]
    fn trace(
        &mut self,
        results: &mut trace_t,
        start: &vec3_t,
        mins: &vec3_t,
        maxs: &vec3_t,
        end: &vec3_t,
        pass_entity_num: i32,
        contentmask: i32,
        capsule: bool,
        trace_flags: i32,
        use_lod: i32,
    );

    // FS_ReadFile → owned buffer; None = Raven's -1/NULL (files.cpp:1670).
    // ICARUS: ICARUS_RegisterScript reads the precompiled .IBI blob (GameInterface.cpp:374).
    fn fs_read_file(&mut self, qpath: &str) -> Option<Vec<u8>>;

    // FS_FreeFile — consuming the Vec is the free (files.cpp:1798).
    // ICARUS: ICARUS_RegisterScript's FS_FreeFile (GameInterface.cpp:393).
    fn fs_free_file(&mut self, _buffer: Vec<u8>) {}

    // Com_Printf — pre-formatted text (common.cpp:128).
    // ICARUS: the verbose logs / Q3_DebugPrint (GameInterface.cpp:129, Q3_Registers Q3_DebugPrint).
    // NOTE: `print` emits text but CANNOT answer Q3_DebugPrint's `com_developer` gate
    // (Q3_Interface.cpp:642) — no cvar/developer read exists on this frozen trait. OPEN
    // per ICARUS-Q12 (needs a design session; ruling 33 forbids adding a method here).
    fn print(&mut self, msg: &str);

    // Com_Error — panic + catch_unwind, never returns; code is errorParm_t (common.cpp:249).
    // ICARUS: ICARUS_Init's NULL-instance ERR_DROP (GameInterface.cpp:151-155).
    fn error(&mut self, code: errorParm_t, msg: &str) -> !;

    // VM_Call(vm, callnum, args) → intptr_t (vm.cpp:787). vm is VmSlot::Gvm/Cgvm.
    // ICARUS: the outbound Q3_* path — VM_Call(gvm, GAME_ICARUS_*) after the shared-mem
    // write (e.g. Q3_PlaySound Q3_Interface.cpp:322). The icarus arms pass no args
    // (the request travels through shared_memory()).
    fn vm_call(&mut self, vm: VmSlot, callnum: i32, args: &[isize]) -> isize;

    // sv.mSharedMemory window — raw char* the game handed over (server.h:87).
    // ICARUS: the Q3_* outbound structs write here before vm_call (Q3_Interface.cpp:315).
    fn shared_memory(&mut self) -> *mut c_char;

    // Q_flrand — float min<=x<max off the engine holdrand LCG (q_math.c:1451).
    // ICARUS: the I_Random slot (Q3_Interface.cpp:978, called Sequencer.cpp:532 etc.).
    fn flrand(&mut self, min: f32, max: f32) -> f32;

    // Q_irand — integer min<=x<=max off the same LCG (q_math.c:1471).
    fn irand(&mut self, min: i32, max: i32) -> i32;

    // SV_GentityNum — raw *mut sharedEntity_t at ent_num (sv_game.cpp:54).
    // ICARUS: index-based access; the ONLY caller is ICARUS_ValidEnt's
    // SV_GentityNum(ent->s.number) write-back (GameInterface.cpp:288/291, ruling 23).
    fn gentity(&mut self, ent_num: i32) -> *mut sharedEntity_t;
}
```

The verbatim quote replaces the earlier paper sketch: the built methods are
`print`/`error` (not `com_printf`/`com_error`), `vm_call(VmSlot, i32, &[isize]) ->
isize` (not `vm_call_game(i32) -> i32`), and `shared_memory() -> *mut c_char` (not
`&mut [u8]`). ICARUS's outbound Q3_* bodies therefore write the `T_G_ICARUS_*`
struct through the raw `*mut c_char` window (the confined ABI-seam `unsafe`, §D11)
and issue `vm_call(VmSlot::Gvm, GAME_ICARUS_*, &[])` — no args, since the request
travels in shared memory (`engine_host.rs:71-80`).

Every §F fn takes `(&mut Icarus, &mut dyn EngineHost, …)` — `dyn`, not `impl`,
because the `InterfaceExport` slot fns are stored `fn` pointers that ICARUS-D10
fixes on `&mut dyn EngineHost` and several pub seam fns double as those slot
targets (ICARUS-Q9 → ICARUS-D10; a bare `fn` pointer cannot be generic over `impl
EngineHost`). `Engine` implements `EngineHost` via a split-borrow view struct so
the `&mut Icarus` field and the rest of `Engine` borrow disjointly; the goldens
inject ruling 32's fixture-backed `MockHost`
(`crates/mp/host-interface/src/mock.rs`) — deterministic `flrand`/`irand` off the
replicated `holdrand` LCG, a recording `vm_call`, and a strided `gentity` arena.
This crate declares no engine globals and calls no `sv`/`svs`/`gvm` singletons
directly — they arrive through `host`.

**Inbound — this crate's public API** (callees of `SV_GameSystemCalls`
`G_ICARUS_*` arms, `sv_game.cpp:739-832`). Uniform §F shape per ICARUS-D1/D10
(`&mut Icarus, &mut dyn EngineHost`, then the arm's args):

Body note (ruling 23, corrects ruling 19): the fns that read entity fields carry
the **pointer** their arm passes — `*mut sharedEntity_t`, exactly the trap
payload (`ConvertedEntity((sharedEntity_t *)VMA(1))` / `(sharedEntity_t
*)VMA(1)`, Raven-ground-truth §Entity-field access). Dereferencing that pointer is
the confined ABI-seam `unsafe` (porting-rules §D11); everything above it is safe.
They do **not** take an `ent_num: i32`. The **three presence-check** callees
(`icarus_is_initialized`/`icarus_maintain_task_manager`/`icarus_is_running`) keep
`ent_num: i32` — their arms read `int entID = args[1]` (`sv_game.cpp:752-782`).
`gentity(ent_num)` is reached from a body only inside `icarus_valid_ent`
(`SV_GentityNum(ent->s.number)`, `GameInterface.cpp:288`, for the behaviorSet
write-back `:291`); `icarus_associate_ent` does **not** call it (`:307-318`).

```rust
// game_interface — the G_ICARUS_* callees. Entity-field arms carry the pointer
// (ruling 23); presence-check arms carry int entID.
pub fn icarus_init(icarus: &mut Icarus, host: &mut dyn EngineHost);
pub fn icarus_shutdown(icarus: &mut Icarus, host: &mut dyn EngineHost);
pub fn icarus_run_script(icarus: &mut Icarus, host: &mut dyn EngineHost, ent: *mut sharedEntity_t, name: &str) -> bool;  // arm :740
pub fn icarus_register_script(icarus: &mut Icarus, host: &mut dyn EngineHost, name: &str, called_during_interrogate: bool) -> bool; // arm :743, no ent
pub fn icarus_valid_ent(icarus: &mut Icarus, host: &mut dyn EngineHost, ent: *mut sharedEntity_t) -> bool;   // arm :750; host.gentity(ent.s.number) inside (behaviorSet write-back, GameInterface.cpp:288/291)
pub fn icarus_init_ent(icarus: &mut Icarus, host: &mut dyn EngineHost, ent: *mut sharedEntity_t);            // arm :789
pub fn icarus_free_ent(icarus: &mut Icarus, host: &mut dyn EngineHost, ent: *mut sharedEntity_t);            // arm :793
pub fn icarus_associate_ent(icarus: &mut Icarus, host: &mut dyn EngineHost, ent: *mut sharedEntity_t);       // arm :797; reads ent.s.number into ent_list (:317), no host.gentity
pub fn icarus_is_initialized(icarus: &mut Icarus, host: &mut dyn EngineHost, ent_num: i32) -> bool;   // arm :752, int entID; gSequencers/gTaskManagers presence
pub fn icarus_maintain_task_manager(icarus: &mut Icarus, host: &mut dyn EngineHost, ent_num: i32) -> bool; // arm :763, int entID; -> CTaskManager::Update
pub fn icarus_is_running(icarus: &mut Icarus, host: &mut dyn EngineHost, ent_num: i32) -> bool;       // arm :774, int entID
// `Svcmd_ICARUS_f` (GameInterface.h:32, GameInterface.cpp:700) is §20-dropped
// per ICARUS-D6 — zero callers, no G_ICARUS_* arm — and is intentionally absent
// from this seam. See Divergences.

// q3_registers — variable store. q3_variable_declared / q3_get_* / q3_declare_variable
// / q3_free_variable are the Q3_Registers.cpp store fns (home q3_registers/mod.rs:
// Q3_VariableDeclared :21, Q3_GetFloatVariable :130, Q3_GetStringVariable :150,
// Q3_GetVectorVariable :170, Q3_DeclareVariable :50, Q3_FreeVariable :91). All
// carry only ints/strings — no entity pointer. q3_variable_declared / q3_get_* are
// the G_ICARUS_VARIABLEDECLARED / GET*VARIABLE arm callees; q3_declare_variable /
// q3_free_variable have NO G_ICARUS_* arm — they are the OUTBOUND I_DeclareVariable
// / I_FreeVariable interface_export targets (Interface_Init, Q3_Interface.cpp:1002-1003),
// reached only through the InterfaceExport table, not the server syscall switch.
//
// q3_set_var is NOT a Q3_Registers.cpp store fn — its source is Q3_SetVar
// (Q3_Interface.cpp:337), so its home is q3_interface/mod.rs (the Q3_Interface.cpp
// unit, Files roster), not q3_registers/mod.rs. It is a thin G_ICARUS_SETVAR arm
// dispatcher (Q3_SetVar(args[1], args[2], VMA(3), VMA(4)), sv_game.cpp:817) that
// resolves type_name and calls the Q3_Registers.cpp store (Q3_GetFloatVariable etc.,
// Q3_Interface.cpp:352); it carries only ints/strings, no entity pointer. Listed in
// this seam block with the other variable-facing arm callees for reader locality; the
// callee's compilation unit is q3_interface, per its source file.
pub fn q3_declare_variable(icarus: &mut Icarus, host: &mut dyn EngineHost, var_type: i32, name: &str);  // I_DeclareVariable target
pub fn q3_free_variable(icarus: &mut Icarus, host: &mut dyn EngineHost, name: &str);                    // I_FreeVariable target
pub fn q3_variable_declared(icarus: &mut Icarus, host: &mut dyn EngineHost, name: &str) -> i32;
pub fn q3_set_var(icarus: &mut Icarus, host: &mut dyn EngineHost, task_id: i32, ent_num: i32, type_name: &str, data: &str);
pub fn q3_get_float_variable(icarus: &mut Icarus, host: &mut dyn EngineHost, name: &str) -> Option<f32>;   // out-param -> Option (§C7)
pub fn q3_get_string_variable(icarus: &mut Icarus, host: &mut dyn EngineHost, name: &str) -> Option<String>;
pub fn q3_get_vector_variable(icarus: &mut Icarus, host: &mut dyn EngineHost, name: &str) -> Option<[f32; 3]>;

// q3_interface — task-id helpers (G_ICARUS_TASKID*). Arms carry the pointer
// (bare (sharedEntity_t *)VMA(1)): PENDING :786, SET :808, COMPLETE :813.
//
// task_type: the arm passes `(taskID_t)args[2]` — an UNCHECKED int→enum cast of an
// arbitrary intptr_t (sv_game.cpp:786/808/813). Each callee body already reproduces
// Raven's own defined guard: `if ( taskType < TID_CHAN_VOICE || taskType >= NUM_TIDS )
// return`/`return qfalse` BEFORE indexing ent->taskID[taskType] (Q3_TaskIDPending
// Q3_Interface.cpp:116-118, Q3_TaskIDComplete :136-138, Q3_TaskIDSet :169-171), so
// Raven's behavior for out-of-range task_type is pinned (no-op / qfalse), not an
// undecided §19 fork — the port transcribes that guard. The residual §19 point is the
// int→enum CONVERSION itself: Rust cannot construct an invalid `taskID_t` (repr enum)
// from an out-of-range `args[2]` the way C's cast does. That checked conversion lives
// at the server-dispatch boundary that owns the arm (§ Scope Punts; Divergences), and
// Raven's callee guard pins its out-of-range outcome (no-op / false). This crate's
// callees take `task_type: taskID_t` already in-range (contrast q3_declare_variable's
// deliberately raw i32 var_type, runtime-validated by Q3_VariableDeclared's own check).
pub fn q3_task_id_set(icarus: &mut Icarus, host: &mut dyn EngineHost, ent: *mut sharedEntity_t, task_type: taskID_t, task_id: i32);
pub fn q3_task_id_complete(icarus: &mut Icarus, host: &mut dyn EngineHost, ent: *mut sharedEntity_t, task_type: taskID_t);
pub fn q3_task_id_pending(icarus: &mut Icarus, host: &mut dyn EngineHost, ent: *mut sharedEntity_t, task_type: taskID_t) -> bool;
```

**Outbound — `InterfaceExport`** (ICARUS-D8 + D10: plain Rust struct of `fn`
items, fork-5; **not** `#[repr(C)]` — it never crosses the module ABI). The `I_*`
signatures mirror `interface.h:17-70`; each impl writes a `T_G_ICARUS_*` struct
into the host's shared-memory window and issues `VM_Call(gvm, GAME_ICARUS_*)`,
both reached through the `EngineHost` passed alongside `&mut Icarus`
(ICARUS-D1). The `T_G_ICARUS_*` payloads and `GAME_ICARUS_*` ids are reused
from `mp_qshared` verbatim (already ported); this crate does not redefine them.
Per **ICARUS-D10** (ruling 24) each stored slot is a
`fn(&mut Icarus, &mut dyn EngineHost, …)` pointer — `dyn` because a bare `fn`
pointer is a concrete type and cannot be generic over `impl EngineHost`:

```rust
// interface_export_s.rs — the I_* table (fork-5 bare-fn slots, host param &mut dyn, ICARUS-D10)
pub struct InterfaceExport {
    pub i_play_sound: fn(&mut Icarus, &mut dyn EngineHost, /* T_G_ICARUS_PLAYSOUND args */),
    pub i_random:     fn(&mut Icarus, &mut dyn EngineHost, min: f32, max: f32) -> f32, // → host.flrand
    pub i_declare_variable: fn(&mut Icarus, &mut dyn EngineHost, var_type: i32, name: &str),
    // … one slot per interface.h:17-70 I_* entry, all fn(&mut Icarus, &mut dyn EngineHost, …)
}
```

That `dyn` slot type propagates to every pub seam fn that doubles as a slot target
(e.g. `q3_declare_variable`/`q3_free_variable` are the `I_DeclareVariable`/
`I_FreeVariable` targets), which is why § inbound signatures above all take
`&mut dyn EngineHost` (ICARUS-D10).

**Traps / cross-crate calls used by this crate** — all reached through
`EngineHost` (ICARUS-D1), never as direct globals; the Raven-name → real
`EngineHost` method map (ICARUS-D12): `FS_ReadFile`/`FS_FreeFile`
(`GameInterface.cpp:374`, `:393`) → `fs_read_file`/`fs_free_file`,
`Com_Printf`/`Com_Error` → `print`/`error`, `Q_flrand`
(`I_Random`, `Q3_Interface.cpp:978`) → `flrand`/`irand`, `VM_Call(gvm, …)` →
`vm_call(VmSlot::Gvm, …)` plus the `shared_memory() -> *mut c_char`
window (the outbound path), and `gentity(ent_num)` — reached from a body **only**
by `icarus_valid_ent`'s `SV_GentityNum(ent->s.number)` behaviorSet write-back
(`GameInterface.cpp:288`/`:291`; `SV_GentityNum`, `sv_game.cpp:54-59`, over the
`G_LOCATE_GAME_DATA` base `:329-330`, ruling 23). The other entity-field fns need
**no** `gentity` — including `icarus_associate_ent`, which only reads `ent.s.number`
into the ent-list (`:317`): they carry the `*mut sharedEntity_t` at the seam (ruling 23).
`Z_Malloc`/`Z_Free` (`Memory.cpp`) are subsumed by owned Rust objects — no arena
(ICARUS-D3 as amended by ruling 20; `ICARUS_Malloc`/`ICARUS_Free` §20-dropped) —
and are not a trap surface.

**Layout types (unchanged, already ported).** `T_G_ICARUS_*` structs +
`game_export_t` + `taskID_t` (the `q3_task_id_*` enum arg,
`oracle/codemp/game/g_public.h:625-639`, ported at
`crates/mp/qshared/src/common/mp/qcommon/task_id_t.rs`) + `sharedEntity_t` (the
entity-field seam arg `*mut sharedEntity_t` and the `gentity` service's return
target, ruling 23,
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
trait** in the Stage-0 interface crate `crates/mp/host-interface` / package
`mp_host_interface` (ruling 24; trace, FS read/free, print/error, `VM_Call`,
shared-memory window, `flrand`/`irand`, `gentity`); every §F fn takes `(&mut
Icarus, &mut dyn EngineHost)`, and `Engine` implements the trait via a
split-borrow view struct (handoff ruling 11). Because fork-2 removes the
`sv`/`svs`/`gvm` globals the `I_*` bodies and inbound `ICARUS_*` fns reach, and
a bare `fn`-item table (ICARUS-D8) cannot capture relocated state, the context
must be threaded — one shared trait keeps every subsystem's seam uniform and
lets the referee inject a deterministic impl. The host param is `&mut dyn` (not
`impl`) per ICARUS-D10. Rejected a per-subsystem `IcarusHost` handle and a `&mut
Engine` first parameter — ruling 11 unified all five §F subsystems on the single
`EngineHost` trait. Settles former ICARUS-Q1.
**Ruling 23 correction (2026-07-09) — supersedes the ruling-19 amendment.**
Ruling 19 wrongly premised that the `G_ICARUS_*` arms hand these fns an entnum
resolved through `SV_GentityNum`. The oracle shows the opposite: the entity-field
arms **carry the `sharedEntity_t *` pointer** — `ICARUS_RunScript` (`:740`),
`ICARUS_ValidEnt` (`:750`), `ICARUS_InitEnt` (`:789`), `ICARUS_FreeEnt` (`:793`),
`ICARUS_AssociateEnt` (`:797`) via `ConvertedEntity((sharedEntity_t *)VMA(1))`,
and `Q3_TaskIDPending` (`:786`), `Q3_TaskIDSet` (`:808`), `Q3_TaskIDComplete`
(`:813`) via a bare `(sharedEntity_t *)VMA(1)` cast (Raven-ground-truth
§Entity-field access). So the inbound seam **carries the pointer**: those fns take
`ent: *mut sharedEntity_t` (the trap payload verbatim; the deref is the confined
seam `unsafe`, §D11), **not** `ent_num: i32`. `ConvertedEntity`'s VM-address
shuffle (`sv_game.cpp:422-451`) is a **no-op in the native-dylib model** —
`VM_ArgPtr` is identity — so the port carries the pointer straight and skips the
shuffle. The three **presence-check** arms (`ISINITIALIZED` `:752`,
`MAINTAINTASKMANAGER` `:763`, `ISRUNNING` `:774`) keep `int entID = args[1]`, so
those callees keep `ent_num: i32`. The `gentity(ent_num) -> *mut sharedEntity_t`
service (`SV_GentityNum`, `sv_game.cpp:54-59`, over the `G_LOCATE_GAME_DATA` base
`:329-330`) **survives** for the one genuinely index-based access:
`ICARUS_ValidEnt`'s `SV_GentityNum(ent->s.number)` behaviorSet write-back
(`GameInterface.cpp:288`/`:291`), which the shuffled `ConvertedEntity` copy cannot
serve; `ICARUS_AssociateEnt` needs **no** `gentity` — it only reads `ent->s.number`
into `ICARUS_EntList` (`:317`). Rejected keeping the
ruling-19 `ent_num`+`host.gentity` shape for every fn — it contradicts the arms,
which pass pointers. Settles the entity-field seam gap.

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
not `#[repr(C)]`) — the concrete host-param type of those bare `fn` items is
`&mut dyn EngineHost` (**ICARUS-D10**, ruling 24, formerly ICARUS-Q9);
verification = differential goldens against the unmodified
oracle TUs under `tools/icarus-oracle/` (§18, DEC-09 layer 1), MP-only with the
SP tree (`oracle/code/icarus/`) a later SP-diff pass (DEC-04/§20), not unified
now; and transcription-first — method bodies transcribe with no safety
refactoring, parity green first, refactor behind the passing diff (§A2). Because
these were settled in the first §F design session and the second session's
rulings 11–17 amend, not overturn, them. Rejected re-litigating any of them —
they are the standing §F baseline this doc builds on.

**ICARUS-D9.** The ~194 internal interface-dispatch sites — `m_ie->I_*` in
`CSequencer` (**73×**, `Sequencer.cpp`) and `(m_owner->GetInterface())->I_*` in
`CTaskManager` (**121×**, `TaskManager.cpp`) — become **free functions taking
`(&mut Icarus, &mut dyn EngineHost, …)` with no stored back-refs** (handoff
ruling 24). `CSequencer`/`CTaskManager` hold **no** `m_ie`/`m_owner`/`m_taskManager`
handle; each method and each `I_*` target is a free fn that re-derives the
disjoint field borrows it needs from `&mut Icarus` on every call (Q8 candidate
(a)). Because ICARUS-D2 stores `sequencers`, `task_managers`, and
`interface_export` as sibling fields on one `Icarus`, a method that held `&mut
icarus.sequencers[n]` across an `I_*` call needing `&mut icarus.var_strings` (e.g.
`I_DeclareVariable`) would be two live `&mut Icarus` borrows — porting-rules §17
requires this sibling-pointer shape be settled before transcription. The free-fn /
re-index-per-call shape never holds a persistent `&mut CSequencer`/`&mut
CTaskManager` across a dispatch, so the borrows stay disjoint. Rejected
take-and-restore (move the slot out of the `[Option<_>; N]` array and back) and
`Rc<RefCell<…>>` handles (ruling 24 chose the borrow-free free-fn shape). The
`csequencer.rs` roster phrase "holds interface + taskmanager handles" is
superseded — no handles are stored. Settles former ICARUS-Q8.

**ICARUS-D10.** The `InterfaceExport` table's stored slots are
`fn(&mut Icarus, &mut dyn EngineHost, …)` **pointers — `&mut dyn EngineHost`, not
`impl`** (handoff ruling 24), and the Stage-0 crate is **pinned** as
`crates/mp/host-interface`, package `mp_host_interface` (`use
mp_host_interface::EngineHost;`). A bare `fn`-item slot (ICARUS-D8/fork-5) is a
concrete type and cannot be generic over `impl EngineHost`, so the host param must
be monomorphic; `&mut dyn EngineHost` keeps one uniform host type across the table
and every pub seam fn (several pub `Q3_*` fns double as `I_*` slot targets, so the
`dyn` propagates to them — hence § Seam definition types every fn `&mut dyn
EngineHost`). Rejected a generic `InterfaceExport<H: EngineHost>` (monomorphising
the whole table per host type — needless, one host exists) and a concrete
monomorphic host handle (loses the referee's injectable impl). Settles former
ICARUS-Q9 and the Stage-0 crate-path hole.

**ICARUS-D11.** The two ICARUS ownership graphs are **faithful `Vec` arenas +
id newtypes** (handoff ruling 27; matches the RMG `AreaId` (ruling 21) and Ghoul2
`SlotMap` (ruling 22) precedents). Concretely: **(a)** `IcarusInstance` **owns**
`sequences: Vec<Sequence>` and `sequencers: Vec<Sequencer>` — Raven's
`list<CSequence*> m_sequences` / `list<CSequencer*> m_sequencers`
(`instance.h:62-63`, owned via `STL_INSERT`, `Instance.cpp:174`, `:231`);
`SequenceId(i32)` / `SequencerId(i32)` newtypes carry Raven's **monotonic,
never-reused** `m_GUID` (`m_GUID = 0` `Instance.cpp:26`, `SetID( m_GUID++ )`
`:228`); `GetSequence(id)` stays a **linear scan** (`STL_ITERATE( … m_sequences )`
`Instance.cpp:248-258`) — faithful `O(n)`, insertion-ordered iteration, **not**
upgraded to a keyed map (an §A2-permissible change the doc declines, to keep
iteration order and GUID semantics parity-exact). `CSequence`'s
`m_parent`/`m_return`/`m_children` (raw pointers reconstructed on `Load` via
`m_owner->GetSequence(id)`, `sequence.h:86-90`, `Sequence.cpp:427-450`) become
`Option<SequenceId>` / `Option<SequenceId>` / `Vec<SequenceId>`; its
`block_l m_commands` has no cross-object aliasing → owned `Vec<CBlock>` (literal
transcription, standing ICARUS-D8). **(b)** `CSequencer`'s **own non-owning**
`sequence_l m_sequences` membership subset (`sequencer.h:176`) becomes
`Vec<SequenceId>`; its `map<CTaskGroup*,CSequence*> m_taskSequences` cross-index
(`sequencer.h:177`) becomes `BTreeMap<TaskGroupId, SequenceId>`. **(c)**
`CTaskManager` has **ONE owning `Vec<TaskGroup>` arena** (Raven's
`vector<CTaskGroup*> m_taskGroups`, `taskmanager.h:177`) + `TaskGroupId`; its two
other parallel `CTaskGroup*` indexes — `map<string,CTaskGroup*> m_taskGroupNameMap`
and `map<int,CTaskGroup*> m_taskGroupIDMap` (`taskmanager.h:183-184`) — become
`BTreeMap<String, TaskGroupId>` / `BTreeMap<i32, TaskGroupId>` **side-indexes of
ids** (three parallel owners collapse to owner + side-indexes). `CTaskManager`'s
`list<CTask*> m_tasks` (`taskmanager.h:178`) has no pointer cross-reference —
`CTaskGroup` tracks completion by **int GUID**, not pointer (`map<int,bool>
m_completedTasks`, `taskmanager.h:87`) — so it transcribes literally to an owned
`Vec<Task>` with `m_completedTasks` a `BTreeMap<i32, bool>` (standing ICARUS-D8, no
new decision). **Definition sites** (roster-hole fix, following the RMG `AreaId`
§B5 precedent of co-locating the index newtype with its arena owner —
RMG-D4g/ruling 21: `AreaId` is defined with `CRMAreaManager`, the arena that owns
it): `SequenceId(i32)` and `SequencerId(i32)` are declared in
`instance/icarus_instance.rs` beside the `IcarusInstance` `sequences`/`sequencers`
arenas they index (ICARUS-D11(a)); `TaskGroupId(i32)` is declared in
`taskmanager/ctask_manager.rs` beside its `m_taskGroups: Vec<TaskGroup>` arena
(ICARUS-D11(c)). No newtype is a standalone module — this is the same placement
the Files roster's `icarus_instance.rs`/`ctask_manager.rs` entries already describe;
pinning the declaring file makes it explicit (mirrors ICARUS-D7's roster-hole fix),
not a new decision. Because a porter cannot invent the §17/§B5 arena-owner type,
id-newtype, and lookup semantics — the sibling §F docs pinned theirs, ICARUS had
not. Rejected keying `GetSequence` by a `HashMap` (loses Raven's linear-scan
iteration order / GUID monotonicity, §A2 change the doc declines) and transcribing
the three `CTaskGroup*` indexes as three literal owning containers (double-owns the
group). Settles former **ICARUS-Q10** and **ICARUS-Q11**.

**ICARUS-D12.** The Stage-0 `EngineHost` crate is **built, green compiled code** —
`crates/mp/host-interface`, package `mp_host_interface`, commit `4b7f01b0` (handoff
rulings 31 + 33) — so § Seam definition quotes its **actual** trait signatures
**verbatim** from `crates/mp/host-interface/src/engine_host.rs:23-106`, and the
goldens run against ruling 32's fixture-backed `MockHost`
(`crates/mp/host-interface/src/mock.rs`), not a paper spec. Ruling 33 forbids
deferrals in the seam, so the real names differ from this doc's earlier sketch and
the port binds them exactly: `print`/`error(errorParm_t, &str) -> !` (not
`com_printf`/`com_error`), `vm_call(VmSlot, i32, &[isize]) -> isize` (not
`vm_call_game(i32) -> i32` — the `VmSlot::Gvm` selector mirrors Raven's `VM_Call(vm,
…)` first param, ruling 33b; icarus arms pass no args, the request travels through
`shared_memory`), and `shared_memory() -> *mut c_char` (a raw `char*` window, not a
`&mut [u8]` — the Q3_* bodies write the `T_G_ICARUS_*` struct through it under the
confined ABI-seam `unsafe`, §D11). Because a real, layout-frozen crate exists, the
doc must be self-contained against it rather than against a provisional spec that
would drift. Rejected keeping the paper method names (they never existed in the
built trait) and adding a test-only constructor to ICARUS (ruling 32 makes
`MockHost` the reusable goldens front door for every host-taking §F subsystem —
`Load`/first-slice ports run with the real frozen signature). Settles the Stage-0
signature gap.

**ICARUS-D13.** BlockStream's writer/duplicator surface splits by ground-truth
caller count (scope-hole fix; no new decision — applies settled §20 + the §
Verification round-trip + ICARUS-D3). The `.IBI` write path exists only to serve
the offline compiler, whose TUs (`Interpreter.cpp`/`Tokenizer.cpp`) are out of the
WinDed link set (Scope), so its methods have **no production callers** in-scope. But
they are **not** uniformly droppable: (a) `CBlock::Duplicate` (`blockstream.h:138`,
`BlockStream.cpp:359`) and `CBlockMember::Duplicate` (`:74`, `:148`) have **zero
callers anywhere** in `oracle/codemp/icarus/` — grep finds only `CBlock::Duplicate`'s
internal member-`Duplicate` self-call (`BlockStream.cpp:374`) and a `"Duplicate
symbol"` string literal (`Tokenizer.cpp:46`), nothing in `Interpreter.cpp` — and
**no** settled decision or verification path references them, so they are
**§20-dropped** with a `blockstream/` module-doc zero-caller note (mirrors the
`Svcmd_ICARUS_f`/`ICARUS_Malloc` drops, ICARUS-D6/D3); (b) `CBlockStream::Create(char
*)` (`:167`), `WriteBlock` (`:174`), `CBlockMember::WriteMember` (`:47`),
`WriteData`/`WriteDataPointer` (`:76`/`:88`), and `CBlock::Write` overloads
(`:125-129`) are **retained** — the § Verification BlockStream round-trip exercises
`ReadBlock`→re-`WriteBlock`→`WriteMember` byte-identical and ICARUS-D3 preserves
`WriteDataPointer`'s exact-byte `memcpy` for content parity, both settled, making the
in-scope §F harness their caller. `CBlockStream::Init` (`:165`) is live (`Open` calls
it, `:670`) and never a drop candidate. Because §20 forbids porting genuinely
dead surface speculatively **and** forbids dropping surface a settled decision still
exercises — the caller counts decide, not the module. Rejected dropping the whole
writer half (verification and ICARUS-D3 reference it) and porting the two
`Duplicate` methods (zero callers, §20). Settles the writer/duplicator scope gap
the dry-run flagged; the "three §20-drops" count in the prior draft was a survey
undercount (two `Duplicate` fns were missed), corrected here.

## Files roster

C++-track roster for `port-cpp-subsystem` (`designPath` consumer). All
`crate: mp_engine_icarus`, `mode: mp`. Existing type-port files are reshaped in
place under ICARUS-D3/D8; new files mirror the owning Raven header subsystem
(one class per file, porting-rules §21).

files:
- path: `crates/mp/engine/icarus/src/lib.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `Icarus`, summary: crate root — defines the fork-2 subsystem aggregate `pub struct Icarus` (fields per the State-ownership table: `instance`, `sequencers`, `task_managers`, `buffer_list`, `ent_list`, `ent_filter`, `interface_export`, `var_strings/floats/vectors`, `num_variables` — **no `arena` field**, dropped per ICARUS-D3/ruling 20) with a **hand-written `impl Default`, not `#[derive(Default)]`** (ICARUS-D2 requires `Icarus` be `Default`-constructible; derive is unavailable/wrong on three fields — see the note under State ownership): the `Box<[Option<SequencerId>; MAX_GENTITIES]>` (non-owning index into `IcarusInstance.sequencers`, ICARUS-D11) / `Box<[Option<CTaskManager>; MAX_GENTITIES]>` slot arrays have no blanket `[T; N]: Default` impl and are built explicitly (e.g. `Box::new(std::array::from_fn(|_| None))`); `ent_filter` seeds `-1`, not derive's `0` (`ICARUS_entFilter = -1`, `GameInterface.cpp:23`; see State ownership); and `interface_export` seeds the real `Q3_*`/`I_*` fns per ICARUS-D8/ruling 20 (see `interface/interface_export_s.rs`). Adds the module declarations the roster requires — new `pub mod sequence; pub mod taskmanager; pub mod instance; pub mod memory; pub mod q3_registers;` alongside existing `blockstream`/`game_interface`/`interface`/`q3_interface`/`sequencer` (and untouched `interpreter`/`tokenizer` skeletons per Scope). `Icarus` is not a Raven class — it is the synthesized owner of every ICARUS file-scope global; it attaches to `mp_engine_core::Engine` as a plain `icarus` field per ICARUS-D2/D7.
- path: `crates/mp/engine/icarus/src/blockstream/cblock_member.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CBlockMember`, summary: one ID/size/data record; owned `Vec<u8>` data replacing `void* m_data`; `WriteMember`/`ReadMember` IBI serialization, `WriteData`/`WriteDataPointer` (all retained per ICARUS-D13/ICARUS-D3 — the § Verification round-trip and ICARUS-D3's content-parity `memcpy` reference them, though their only production callers are the excluded compiler TUs) (`blockstream.h:38-105`, `BlockStream.cpp`). `CBlockMember::Duplicate` (`:74`) is **§20-dropped** (ICARUS-D13) — its only caller is `CBlock::Duplicate`, itself caller-less — with a module-doc zero-caller note, not ported. `m_data` is an owned `Vec<u8>` (settled — ICARUS-D3/ruling 20 drops the arena, so this `TAG_ICARUS5` blob is owned here); `WriteDataPointer` `memcpy`s exact bytes into it (content parity, Divergences).
- path: `crates/mp/engine/icarus/src/blockstream/cblock.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CBlock`, summary: owns `Vec<CBlockMember>`; block id + flags; `Create(int)` (live — reader/`ReadBlock` path, `BlockStream.cpp:635`), `Write` overloads (retained per ICARUS-D13/ICARUS-D3), `AddMember`/`GetMember` (`blockstream.h:109-154`). `CBlock::Duplicate` (`:138`) is **§20-dropped** (ICARUS-D13) — zero callers anywhere — with a module-doc zero-caller note, not ported.
- path: `crates/mp/engine/icarus/src/blockstream/cblock_stream.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CBlockStream`, summary: `.IBI` reader/writer over an owned byte buffer; `Open`/`ReadBlock`/`BlockAvailable`/`Init` (reader path; `Init` is live — `Open` calls it, `BlockStream.cpp:670`), `IBI` header+version check (`blockstream.h:158-196`). The writer methods `Create(char *)` (`fopen("wb")`, `:167`) and `WriteBlock` (`:174`) are **retained, not dropped** (ICARUS-D13): their only production callers are the excluded compiler TUs, but the § Verification round-trip exercises `ReadBlock`→re-`WriteBlock` byte-identical, so they port per the roster.
- path: `crates/mp/engine/icarus/src/blockstream/vector_t.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `vector_t`, summary: `[f32; 3]` alias used by block writes (existing skeleton, kept).
- path: `crates/mp/engine/icarus/src/blockstream/file.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CBlockStream::file`, summary: owned file-handle helper for `.IBI` I/O replacing `FILE*` (existing skeleton).
- path: `crates/mp/engine/icarus/src/sequence/csequence.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CSequence`, summary: command/child tree node; flags/iterations, `Save/Load` (inert) (`sequence.h:12-96`, `Sequence.cpp`). **Shape settled by ICARUS-D11 (ruling 27):** `m_children: Vec<SequenceId>`, `m_parent: Option<SequenceId>`, `m_return: Option<SequenceId>` (pointer graph → id newtypes into the `IcarusInstance.sequences` arena, reconstructed on `Load` via a `GetSequence(id)` linear scan); `m_commands: Vec<CBlock>` owned (no cross-object aliasing, literal transcription per ICARUS-D8).
- path: `crates/mp/engine/icarus/src/sequencer/csequencer.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CSequencer`, summary: per-entity script driver; parses IBI blocks into sequences; `Run`/`Callback`/`Parse*`/`Check*`/`Push/PopCommand` (`sequencer.h:68-187`, `Sequencer.cpp`, ~43 fns). Per **ICARUS-D9** (ruling 24) it holds **no** `m_ie`/`m_owner`/taskmanager handles — the 73 `m_ie->I_*` dispatch sites become free fns `(&mut Icarus, &mut dyn EngineHost, …)` that re-index disjoint `Icarus` field borrows per call (the former "holds handles" is superseded). Dispatch shape settled (ICARUS-D9); **member-storage shape settled by ICARUS-D11 (ruling 27):** `CSequencer`'s own non-owning `m_sequences` membership subset (`sequencer.h:176`) is `Vec<SequenceId>` and its `map<CTaskGroup*,CSequence*> m_taskSequences` (`sequencer.h:177`) is `BTreeMap<TaskGroupId, SequenceId>` — ids into the `IcarusInstance` arenas, not owning containers.
- path: `crates/mp/engine/icarus/src/sequencer/bstream_s.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `bstream_t`, summary: internal stream-stack node; intrusive `last` pointer folds into the sequencer's owned `Vec` (existing skeleton, `sequencer.h:42-46`).
- path: `crates/mp/engine/icarus/src/taskmanager/ctask.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CTask`, summary: a scheduled task (GUID, timestamp, owned `CBlock`) (`taskmanager.h:33-58`). **Ownership shape settled by ICARUS-D11 (ruling 27):** `CTask` is owned in `CTaskManager`'s `m_tasks: Vec<Task>` (Raven's `list<CTask*> m_tasks`, `taskmanager.h:178`); no cross-object pointer aliasing (groups track completion by int GUID), so this is literal transcription per ICARUS-D8, no id newtype needed.
- path: `crates/mp/engine/icarus/src/taskmanager/ctask_group.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CTaskGroup`, summary: completion-tracking group; `MarkTaskComplete`/`Complete` (`taskmanager.h:62-93`). **Shape settled by ICARUS-D11 (ruling 27):** owned in `CTaskManager`'s `Vec<TaskGroup>` arena keyed by `TaskGroupId`; `m_completedTasks` (Raven's already-int-keyed `map<int,bool>`, `taskmanager.h:87`) → `BTreeMap<i32, bool>`; `m_parent: Option<TaskGroupId>` (raw `CTaskGroup*` back-pointer → id).
- path: `crates/mp/engine/icarus/src/taskmanager/ctask_manager.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CTaskManager`, summary: per-entity scheduler; `Update` heartbeat (`Go`, `RUNAWAY_LIMIT`), the `Rotate/Camera/Print/Sound/Move/Set/Use/…/Wait/WaitSignal` handlers, owned task maps/lists; `Get` scratch out-param folds to an owned return (`taskmanager.h:97-189`, `TaskManager.cpp`, ~52 fns). Per **ICARUS-D9** (ruling 24) it holds **no** `m_owner` back-ref — the 121 `(m_owner->GetInterface())->I_*` dispatch sites become free fns `(&mut Icarus, &mut dyn EngineHost, …)` that re-index disjoint `Icarus` field borrows per call. Dispatch shape settled (ICARUS-D9); **member-storage shape settled by ICARUS-D11 (ruling 27):** the three parallel `CTaskGroup*` indexes collapse to **one owner** `m_taskGroups: Vec<TaskGroup>` (Raven's `vector<CTaskGroup*>`, `taskmanager.h:177`) + `TaskGroupId` — **this file also declares `pub struct TaskGroupId(i32)`**, co-located with the `Vec<TaskGroup>` arena it indexes per ICARUS-D11's definition-site pin (RMG `AreaId` §B5 precedent) — with `m_taskGroupNameMap` / `m_taskGroupIDMap` (`taskmanager.h:183-184`) as `BTreeMap<String, TaskGroupId>` / `BTreeMap<i32, TaskGroupId>` **side-indexes of ids**; `m_tasks: Vec<Task>` owns the tasks (literal, ICARUS-D8).
- path: `crates/mp/engine/icarus/src/instance/icarus_instance.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `ICARUS_Instance`, summary: top singleton; **owns** its sequence/sequencer arenas + signal map; `Create`/`Delete`, `Signal`/`CheckSignal`/`ClearSignal`, inert `Save*/Load*` (`instance.h:12-79`, `Instance.cpp`, ~24 fns). **Pool container + lookup shape settled by ICARUS-D11 (ruling 27):** `sequences: Vec<Sequence>` + `sequencers: Vec<Sequencer>` owning arenas (Raven's `m_sequences`/`m_sequencers`, `instance.h:62-63`); **this file also declares `pub struct SequenceId(i32)` and `pub struct SequencerId(i32)`** — the newtypes carrying the monotonic never-reused `m_GUID` (`Instance.cpp:26,228`), co-located with the arenas they index per ICARUS-D11's definition-site pin (RMG `AreaId` §B5 precedent); `GetSequence(id)` stays a **faithful linear scan** (`Instance.cpp:248-258`), insertion-ordered, **not** upgraded to a keyed map (§A2 change the doc declines); `m_signals: BTreeMap<String, u8>`. The per-entity `gSequencers`/`gTaskManagers` arrays are file-scope globals (`Instance.cpp:19-20`), not `ICARUS_Instance` members — `Icarus.sequencers` is a non-owning `SequencerId` index into this arena, `Icarus.task_managers` owns the task managers (State-ownership table, ICARUS-D2/D11).
- path: `crates/mp/engine/icarus/src/interface/interface_export_s.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `interface_export_t`, summary: reshape the `#[repr(C)]` type-port skeleton into the plain `fn`-item `InterfaceExport` table (ICARUS-D8) (`interface.h:17-70`). Because ICARUS-D8/fork-5 make the fields **bare `fn` items (not `Option<fn>`)**, the struct has no null/None state, yet ICARUS-D2 requires `Icarus` (hence this field) be `Default`-constructible — so its `impl Default` seeds every slot with the real crate `Q3_*`/`I_*` fn, identical to `Interface_Init`'s 1:1 assignment (`Q3_Interface.cpp:956-1008`). This is faithful, not an invented pre-init state: init order runs `Interface_Init(&interface_export)` to (re-)populate the table before `ICARUS_Instance::Create` and before any `I_*` call (`GameInterface.cpp:143-156`), so the seed is overwritten with the same fns before it is ever observed. `Interface_Init` in `q3_interface/mod.rs` remains the live wiring; the seed only satisfies Rust's construct-before-use requirement for the bare-`fn` table. Per **ICARUS-D10** (ruling 24) each stored slot is a `fn(&mut Icarus, &mut dyn EngineHost, …)` pointer (`&mut dyn`, not `impl` — a bare `fn` cannot be generic); the `impl Default` seed and 1:1 `Interface_Init` wiring hold under it.
- path: `crates/mp/engine/icarus/src/memory/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `(none — §20 note)`, summary: **no `IcarusArena` type** — ICARUS-D3/ruling 20 drops the arena entirely (all three `TAG_ICARUS5` families are owned buffers: `CBlockMember::m_data`, `pscript_t::buffer` owned `Vec<u8>`, and `bData` `Save`/`Load` scratch folded to a local). `ICARUS_Malloc`/`ICARUS_Free` (`Memory.cpp:8-20`, `icarus.h:29-30`) have zero live callers under the owned-buffer shape and are **§20-dropped, not ported**; this file carries only the module-doc zero-caller note recording that (porting-rules §20). ICARUS-Q7 settled.
- path: `crates/mp/engine/icarus/src/q3_interface/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `Q3Interface`, summary: the `Q3_*` `I_*` implementations (shared-memory write + `VM_Call`, both via `EngineHost` per ICARUS-D1) and `Interface_Init` wiring; `Q3_Evaluate`, camera stubs, and the `Q3_TaskIDSet`/`Complete`/`Pending` task-id helpers (which read/write `ent->taskID[]`/`s.number` on the `ent: *mut sharedEntity_t` the seam carries, per ICARUS-D1/ruling 23 — no `host.gentity`) (`Q3_Interface.cpp`, ~49 fns). The `I_*` slot fns' host-param type is `&mut dyn EngineHost` (ICARUS-D10) and internal dispatch is via free fns re-indexing `&mut Icarus` (ICARUS-D9); both settled.
- path: `crates/mp/engine/icarus/src/q3_interface/set_type_t.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `setType_e`, summary: SET_* enum (existing skeleton, kept).
- path: `crates/mp/engine/icarus/src/q3_interface/play_type_t.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `playType_e`, summary: PLAY_* enum (existing skeleton, kept).
- path: `crates/mp/engine/icarus/src/q3_registers/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `Q3Registers`, summary: `varStrings`/`varFloats`/`varVectors` stores + `Q3_InitVariables`/`Q3_Declare/Free/Get/Set*Variable`, `MAX_VARIABLES`, `VTYPE_*`; `Q3_DebugPrint` reaches `Com_Printf` via `EngineHost::print` (ICARUS-D1) — **but its `com_developer` gate (`Q3_Interface.cpp:642`) has NO `EngineHost` method; OPEN per ICARUS-Q12**, blocking these ~16 fns' debug-print sites until resolved (`Q3_Registers.cpp`, `.h:4-32`, ~16 fns).
- path: `crates/mp/engine/icarus/src/game_interface/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `GameInterface`, summary: `ICARUS_RunScript`/`RegisterScript`/`GetScript`/`Init`/`InitEnt`/`FreeEnt`/`ValidEnt`/`AssociateEnt`/`Shutdown`/`LinkEntity`/`SoundPrecache`/`InterrogateScript` + the buffer/ent-list state; the 5 unchecked entnum paths guard-and-return per ICARUS-D5; `Svcmd_ICARUS_f` is §20-dropped per ICARUS-D6 (zero-caller module-doc note, not ported); `RunScript`/`InitEnt`/`FreeEnt`/`ValidEnt`/`AssociateEnt` read `ent->classname`/`taskID`/`script_targetname`/`targetname` on the `ent: *mut sharedEntity_t` the seam carries (ICARUS-D1/ruling 23); only `ValidEnt` additionally calls `host.gentity(ent.s.number)` for the true entity, its behaviorSet write-back `trueEntity->script_targetname = trueEntity->targetname` (`GameInterface.cpp:288`/`:291`) — `AssociateEnt` does **not** (it only reads `ent.s.number` into the ent-list, `:317`) (`GameInterface.cpp`, ~14 live fns).
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
- `ConvertedEntity`'s VM-address shuffle (`sv_game.cpp:422-451`) is **not** replicated (ruling 23). It copies `ent->s`/`r`/`taskID` into a file-static `gLocalModifier` and re-points string/`parms` fields through `VM_ArgPtr` (`:444-448`), returning `&gLocalModifier`; `VM_ArgPtr` is **identity in the native-dylib model** (no VM heap to relocate against), so the shuffle is a no-op and the seam carries the original `*mut sharedEntity_t` straight to the ICARUS fns. This lives in the server crate's `G_ICARUS_*` dispatch, not this crate — noted here because it is why the entity-field seam args are `*mut sharedEntity_t` (ICARUS-D1). The `ICARUS_ValidEnt` write-back hack (`GameInterface.cpp:288-291`, the behaviorSet branch) — which in C needs the *true* entity because `gLocalModifier`'s pointer fields cannot be stored back — resolves through the `host.gentity(ent.s.number)` service (ruling 23); under the identity `VM_ArgPtr`, that true entity is the same pointer the seam carried. (`ICARUS_AssociateEnt` has no such hack — it only reads `ent.s.number` into `ICARUS_EntList`, `:317`, so it needs no `gentity`.)
- Out-of-range entnum on the 5 unchecked `gSequencers`/`gTaskManagers` paths → **guard-and-return** (resolved per ICARUS-D5, ruling 15). The `ISINITIALIZED`/`MAINTAINTASKMANAGER`/`ISRUNNING` arms index `gSequencers[entID]`/`gTaskManagers[entID]` with `entID = args[1]` unchecked (`sv_game.cpp:752-782`), and `ICARUS_RunScript`/`ICARUS_InitEnt` index `gSequencers[ent->s.number]` unchecked (`GameInterface.cpp:75`, `:660`); only `ICARUS_FreeEnt` guards `s.number >= MAX_GENTITIES || < 0` (`GameInterface.cpp:224-229`). In C a negative/`>= MAX_GENTITIES` entnum is an OOB pointer read (UB); on the Rust `[Option<_>; MAX_GENTITIES]` arrays each of the five ports the `FreeEnt` bounds-check and returns the "absent" result (`false`/no-op), with a ≤2-line §19 note per site. Excluded from / normalized in the shared corpus.
- `ICARUS_Malloc` is non-zeroed (`Z_Malloc(...,qfalse)`, `Memory.cpp:12`) while class `operator new` is zeroed (`Z_Malloc(...,qtrue)`, `blockstream.h:66`); under ICARUS-D3/ruling 20 both class instances and the `TAG_ICARUS5` blobs (`CBlockMember::m_data`, `pscript_t::buffer`) are owned Rust objects/`Vec<u8>`, always value-initialized, so the zero/non-zero distinction collapses to defined initialization — `WriteDataPointer` still `memcpy`s exact bytes, preserving content parity.
- `ICARUS_Malloc`/`ICARUS_Free` (`Memory.cpp:8-20`, `icarus.h:29-30`) are §20-dropped per ICARUS-D3/ruling 20: with every `TAG_ICARUS5` user owned, the arena is dropped entirely and these two fns have zero live callers. Recorded with a module-doc zero-caller note in `memory/mod.rs`, not ported (`Icarus.arena` does not exist).
- `Svcmd_ICARUS_f` (`GameInterface.h:32`, `GameInterface.cpp:700-730`) is §20-dropped per ICARUS-D6: commented-out body, zero callers/registrations, no `G_ICARUS_*` arm. Recorded with a module-doc zero-caller note, not ported.
- BlockStream duplicators `CBlock::Duplicate` (`blockstream.h:138`, `BlockStream.cpp:359`) and `CBlockMember::Duplicate` (`:74`, `:148`) are §20-dropped per ICARUS-D13: zero callers anywhere in `oracle/codemp/icarus/` (only `CBlock::Duplicate`'s internal member-`Duplicate` self-call `:374` + a `Tokenizer.cpp:46` string literal; nothing in `Interpreter.cpp`), unused by the § Verification round-trip. Recorded with a `blockstream/` module-doc zero-caller note, not ported. The rest of the writer surface (`Create(char *)`/`WriteBlock`/`WriteMember`/`WriteData`/`WriteDataPointer`/`CBlock::Write`) is **retained** — production-dead but exercised by the settled § Verification round-trip and ICARUS-D3 content-parity (ICARUS-D13), so it is not a divergence, just non-production-reached in-scope surface.
- Out-of-range `task_type` on the three `G_ICARUS_TASKID*` arms → **guard-and-return** is Raven's own defined behavior, not an undecided fork. The arms cast `(taskID_t)args[2]` unchecked (`sv_game.cpp:786`, `:808`, `:813`), but each callee guards `taskType < TID_CHAN_VOICE || taskType >= NUM_TIDS` and returns before indexing `ent->taskID[taskType]` (`Q3_TaskIDPending` `Q3_Interface.cpp:116-118`, `Q3_TaskIDComplete` `:136-138`, `Q3_TaskIDSet` `:169-171`), so the port transcribes that guard. The only genuine §19 point is the int→enum conversion (Rust cannot build an invalid `taskID_t` from an arbitrary int); that checked conversion lives at the server-dispatch boundary that owns the arm (Punts, § Seam definition task-id note), with Raven's guard pinning the out-of-range outcome. This crate's callees receive an in-range `taskID_t`.

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
  `ICARUS_RunScript` on a committed `.IBI` fixture through ruling 32's
  fixture-backed `MockHost` (`crates/mp/host-interface/src/mock.rs`, ICARUS-D12):
  `.IBI` bytes served from `MockHost.files`, deterministic `flrand`/`irand` off the
  replicated `holdrand` LCG, and `MockHost.vm_calls` recording the ordered
  `vm_call(VmSlot::Gvm, GAME_ICARUS_*, …)` trace (`vm_call_return` scripts the
  reply). No test-only ICARUS constructor is added — `ICARUS_RunScript` ports with
  its real frozen signature and reaches the world through the mock's front door
  (ruling 32). The golden is that ordered `vm_call`/`I_*` callback stream plus final
  variable/signal state, exercising `Parse*`/`Check*`, task scheduling,
  `Wait`/`WaitSignal`, and `Callback`.

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
  (ICARUS-D1, `&mut dyn EngineHost`) and the `tools/icarus-oracle` harness /
  `tools/ibi-gen` fixture corpus (ICARUS-D4) are the remaining build items.
  The `CSequencer`/`CTaskManager`/`InterfaceExport` **dispatch** internals are
  **unblocked** — ICARUS-Q8/Q9 settled by ICARUS-D9/D10 (ruling 24: free-fn
  dispatch, `&mut dyn EngineHost` slots). Their **member-storage** shape is now
  **also settled** — **ICARUS-Q10/Q11 are closed by ICARUS-D11** (ruling 27:
  `Vec` arenas + `SequenceId`/`SequencerId`/`TaskGroupId` newtypes) — so
  `csequence.rs`/`csequencer.rs`/`icarus_instance.rs`/`ctask*.rs` are transcribable
  with no remaining escalation.
- **Stage-0 interface crate** (`crates/mp/host-interface`, package
  `mp_host_interface`; **built, commit `4b7f01b0`**, rulings 24 + 31 + 33;
  `docs/plans/2026-07-08-mp-engine-build-out.md:250-251`) — the `EngineHost` trait
  is already compiled code (`src/engine_host.rs`) with its **full** method roster
  frozen; this doc `use`s the crate directly and quotes the ICARUS-consumed subset
  verbatim (§ Seam definition, ICARUS-D12). Already landed — precedes ICARUS in the
  port order, no longer a pending dependency.
- **Server crate `SV_GameSystemCalls`** — the `G_ICARUS_*` dispatch arms
  (`sv_game.cpp:739-832`) call this crate's inbound public API; that switch
  ports in the server slice and depends on the inbound signatures + the
  `EngineHost` shape here. The outbound `GAME_ICARUS_*` handlers already exist
  in `mp_game`.
- **Engine aggregate** — `mp_engine_core::Engine` gains the plain `icarus`
  field (ICARUS-D2) and the split-borrow view struct that implements
  `EngineHost` (ICARUS-D1) — including its `gentity(ent_num)` service
  (ruling 23, used only by `icarus_valid_ent`'s behaviorSet write-back);
  `crates/mp/engine/core/src/engine.rs:35-36`.
- **Memory/Zone** — no dependency: ICARUS-D3/ruling 20 drops the arena and
  §20-drops `ICARUS_Malloc`/`ICARUS_Free`, so ICARUS owns all its storage and
  needs no engine-Zone hook.
- **`G_LOCATE_GAME_DATA`/`SV_GentityNum`** (server crate) — installs the
  `sv.gentities`/`sv.gentitySize` base (`sv_game.cpp:329-330`) that the
  `EngineHost::gentity` service reads (ICARUS-D1/ruling 23) and that
  `ConvertedEntity`/the entity-field arms build their `*mut sharedEntity_t` over;
  lands with the server slice, before ICARUS's entity-field fns run live.

## Open questions

MUST be empty at FROZEN. **ICARUS-Q1–Q11 are all resolved; one new item —
ICARUS-Q12 (the `com_developer` debug-print gate has no `EngineHost` seam method) —
was surfaced by the dry-run and remains OPEN, blocking FROZEN until an interactive
design session settles it.**
Q1–Q6 by the second §F design-session rulings (handoff ledger `:114-158`); Q7 by
ruling 20; the entity-field **seam gap**, **ICARUS-Q8**, and **ICARUS-Q9** by the
rulings 23 + 24 (2026-07-09) — ICARUS-D1, ICARUS-D9, ICARUS-D10 respectively; and
the last two items, **ICARUS-Q10 and ICARUS-Q11**, by the final-revision
**ruling 27** (2026-07-09, ICARUS-D11) — faithful `Vec` arenas + id newtypes
(`SequenceId`/`SequencerId`/`TaskGroupId`), matching the RMG (ruling 21) and
Ghoul2 (ruling 22) precedents that these two questions had flagged as unmatched.
Ruling 27 pins the specific §17/§B5 arena-owner types, id newtypes, and lookup
semantics the porter could not invent, so `csequence.rs`/`csequencer.rs`/
`icarus_instance.rs`/`ctask.rs`/`ctask_group.rs`/`ctask_manager.rs` are
transcribable. Separately, **rulings 31 + 33** (2026-07-09, ICARUS-D12) built the
Stage-0 `EngineHost` crate as real compiled code (`crates/mp/host-interface`,
commit `4b7f01b0`), so § Seam definition quotes its actual signatures verbatim
rather than a paper spec. The Q1–Q11 closures leave the port transcribable, but
**ICARUS-Q12** (dry-run-surfaced: `Q3_DebugPrint`'s `com_developer` gate has no
`EngineHost` seam method) is **OPEN and must be settled in an interactive session
before this doc can reach FROZEN** — it cannot self-resolve (ruling 33 freezes the
seam).

- **ICARUS-Q1** (host/context threading) → resolved by **ICARUS-D1** (ruling 11:
  one `EngineHost` trait, `(&mut Icarus, &mut dyn EngineHost)`; ruling 23 corrects
  the entity-field seam to carry `*mut sharedEntity_t`, keeping `gentity` only for
  `icarus_valid_ent`'s behaviorSet write-back).
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
- **Entity-field seam gap** (do the entity-field fns take an entnum or a pointer?)
  → **resolved by ICARUS-D1 as corrected by ruling 23** (2026-07-09). The
  `G_ICARUS_*` entity-field arms carry the `sharedEntity_t *` pointer directly
  (`sv_game.cpp:740/750/786/789/793/797/808/813`), not an entnum; those fns take
  `ent: *mut sharedEntity_t`. Only the three presence-check arms (`:752/:763/:774`)
  carry `int entID`. `gentity` survives for `icarus_valid_ent`'s
  `SV_GentityNum(ent->s.number)` behaviorSet write-back alone (`GameInterface.cpp:288`/`:291`);
  `icarus_associate_ent` needs none (it only reads `ent.s.number` into the ent-list, `:317`).
  This supersedes the ruling-19 premise that every entity-field fn took an entnum + `host.gentity`.
- **ICARUS-Q8** (internal interface/host dispatch convention for `CSequencer`/
  `CTaskManager`) → **resolved by ICARUS-D9** (ruling 24, 2026-07-09). The ~194
  `m_ie->I_*` (73×, `Sequencer.cpp`) / `(m_owner->GetInterface())->I_*` (121×,
  `TaskManager.cpp`) dispatch sites become **free fns `(&mut Icarus, &mut dyn
  EngineHost, …)` with no stored back-refs** — candidate (a), re-index-per-call,
  which keeps the sibling-field borrows disjoint (no two live `&mut Icarus`).
  Rejected take-and-restore and `Rc<RefCell<…>>`.
- **ICARUS-Q9** (host-param type of the stored `InterfaceExport` `fn` items) →
  **resolved by ICARUS-D10** (ruling 24, 2026-07-09). The bare `fn`-pointer slots
  take **`&mut dyn EngineHost`** (candidate (a), trait-object dispatch); a bare
  `fn` cannot be generic over `impl EngineHost`, and the `dyn` slot type propagates
  to the pub seam fns that double as `I_*` targets — so § Seam definition types
  every fn `&mut dyn EngineHost` (superseding ICARUS-D1's original `impl`).
  Ruling 24 also pins the Stage-0 crate as `crates/mp/host-interface` /
  `mp_host_interface`, closing the crate-path hole.
- **ICARUS-Q10** (the `ICARUS_Instance` → `CSequencer`/`CSequence`
  ownership-graph shape) → **resolved by ICARUS-D11** (ruling 27, 2026-07-09).
  Ground truth the ruling settles against: `ICARUS_Instance` truly **owns** every
  `CSequence`/`CSequencer` via `list<CSequence*> m_sequences` / `list<CSequencer*>
  m_sequencers` (`instance.h:16-31,59-65`), hands out fresh objects via `GetSequence()`
  with a monotonic **never-reused** `m_GUID++` (`Instance.cpp:223-258`,
  `Sequence.cpp:60-78`), and looks them up by id through a **linear scan**
  `GetSequence(int id)`; `CSequence`'s `m_parent`/`m_return`/`m_children` are raw
  pointers reconstructed from int ids on Load via `m_owner->GetSequence(id)`
  (`sequence.h:14-90`, `Sequence.cpp:427-450`); and `CSequencer` keeps its **own
  separate non-owning** `sequence_l m_sequences` membership subset plus a
  `map<CTaskGroup*,CSequence*> m_taskSequences` cross-index (`sequencer.h:70-187`).
  ICARUS-D11 pins the three points a porter could not invent: (a) `IcarusInstance`
  owns `sequences: Vec<Sequence>` + `sequencers: Vec<Sequencer>` keyed by
  `SequenceId(i32)`/`SequencerId(i32)` newtypes carrying the monotonic never-reused
  `m_GUID`; (b) `GetSequence(id)` stays a **faithful linear scan** (insertion-ordered,
  **not** upgraded to a keyed map — an §A2 change the doc explicitly declines to keep
  GUID/iteration parity); (c) `CSequencer`'s non-owning `m_sequences` subset →
  `Vec<SequenceId>` and `m_taskSequences` → `BTreeMap<TaskGroupId, SequenceId>`; and
  `CSequence`'s `m_parent`/`m_return`/`m_children` → `Option<SequenceId>` /
  `Option<SequenceId>` / `Vec<SequenceId>`. Unblocks `csequence.rs`, `csequencer.rs`,
  `icarus_instance.rs`.
- **ICARUS-Q11** (the `CTaskManager` fan-out shape) → **resolved by ICARUS-D11**
  (ruling 27, 2026-07-09). Ground truth: `CTaskManager` fans one `CTaskGroup*` out to
  **three parallel indexes** — `vector<CTaskGroup*> m_taskGroups` (`taskmanager.h:177`),
  `map<string,CTaskGroup*> m_taskGroupNameMap` and `map<int,CTaskGroup*>
  m_taskGroupIDMap` (`taskmanager.h:183-184`) — plus a `list<CTask*> m_tasks` task list
  and per-group `map<int,bool> m_completedTasks` completion tracking
  (`taskmanager.h:87,97-189`). ICARUS-D11 collapses the three parallel `CTaskGroup*`
  indexes to **one owner** `m_taskGroups: Vec<TaskGroup>` + `TaskGroupId`, with
  `m_taskGroupNameMap`/`m_taskGroupIDMap` → `BTreeMap<String, TaskGroupId>` /
  `BTreeMap<i32, TaskGroupId>` **side-indexes of ids**; `m_tasks` → owned `Vec<Task>`
  (literal transcription — groups track completion by int GUID, no pointer aliasing —
  per standing ICARUS-D8) and `m_completedTasks` → `BTreeMap<i32, bool>`. Unblocks
  `ctask.rs`, `ctask_group.rs`, `ctask_manager.rs`.
- **ICARUS-Q12** (the `com_developer` debug-print gate has no `EngineHost` seam
  method) — **OPEN, needs an interactive design session; not self-resolvable.**
  `Q3_DebugPrint` gates **all** its output on the engine cvar `com_developer`
  before touching `Com_Printf`: `if (!com_developer || !com_developer->integer)
  return;` (`Q3_Interface.cpp:638-643`, esp. `:642`). `com_developer` is an engine
  cvar (`extern cvar_t *com_developer`, `oracle/codemp/qcommon/qcommon.h:688`;
  defined `common.cpp:39`, registered `Cvar_Get("developer","0",CVAR_TEMP)`
  `common.cpp:1307`) — settable at runtime, so it is **not** safely assumable as
  `0` in the dedicated build. `Q3_DebugPrint` is **live and reached in-scope** —
  called from `Q3_Registers.cpp:58,77,199`, `GameInterface.cpp:129`, and ~15
  `Q3_Interface.cpp` verbose/warning/error sites — so this gate is on the path of
  essentially every `q3_registers/mod.rs`/`q3_interface/mod.rs` diagnostic call.
  But the **frozen** `EngineHost` trait (`crates/mp/host-interface/src/engine_host.rs:23-106`,
  ICARUS-D12) has **no cvar/developer read** — its 9 methods are
  `trace`/`fs_read_file`/`fs_free_file`/`print`/`error`/`vm_call`/`shared_memory`/
  `flrand`/`irand`/`gentity`; `print` takes only pre-formatted text and cannot
  answer the gate. Ruling 33 ("no deferrals", frozen seam) forbids a porter adding
  a method speculatively. **This contradicts** the § Seam definition `print`
  annotation and the Files-roster `q3_registers/mod.rs` entry, both of which assert
  "`Q3_DebugPrint` reaches `Com_Printf`/`com_developer` via `EngineHost`
  (ICARUS-D1)" — there is no such seam method. Escalate: how does `Q3_DebugPrint`
  read `com_developer` through the seam? A porter cannot pick among the candidate
  resolutions (add a `developer()`/cvar-read method to the Stage-0 `EngineHost` crate
  — a change to the interface-crate design owned outside this doc, handoff ruling 11;
  route the developer state through the split-borrow `Engine` view another way; or
  gate the debug print elsewhere), because each changes the frozen seam. Until
  settled, the § Seam definition `print` note and the `q3_registers/mod.rs` roster
  entry overclaim; they are flagged inline pending ICARUS-Q12.
