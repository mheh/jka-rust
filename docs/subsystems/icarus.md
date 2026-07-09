# ICARUS sequencer — MP engine (§F idiomatic reimplementation) Design
Status: DRAFT     Supersedes: none
Decision prefix: ICARUS     Ledger deps: DEC-09, DEC-10; engine-fork-discovery forks 2, 4, 5, 7; GOAL-engine M2 gate

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
  sub-structs, no `static mut`), fork-4 (faithful owned arenas), fork-5
  (dispatch tables → plain Rust structs of `fn` items), fork-7 (the 5-doc
  §F list, ICARUS named).
- `docs/GOAL-engine.md` — M2 gate ("icarus differential goldens").
- GP2 exemplar: `crates/mp/engine/qcommon/src/gp2/`, `tools/gp2-oracle/`.

## Scope & non-goals

**In scope.** The 10 `WinDed.vcproj`-Release-linked ICARUS sources
(`oracle/codemp/icarus/`): `BlockStream.cpp`, `GameInterface.cpp`,
`Instance.cpp`, `Interface.cpp`, `Memory.cpp`, `Q3_Interface.cpp`,
`Q3_Registers.cpp`, `Sequence.cpp`, `Sequencer.cpp`, `TaskManager.cpp` —
253 fns / 6,749 LOC (plan §graph row). Every method ports (§F: only zero-caller
API is droppable per §20). Target crate: `mp_engine_icarus` (MP only).

**Out of scope.** `Interpreter.cpp` and `Tokenizer.cpp` are **not** in the link
set. The dedicated server never compiles scripts: `ICARUS_RegisterScript`
appends `.IBI` and `FS_ReadFile`s a **precompiled** block-instruction blob
(`oracle/codemp/icarus/GameInterface.cpp:346-395`); the `.txt`→`.IBI` compiler
(interpreter/tokenizer) is an offline/SP tool. The type-port skeletons under
`crates/mp/engine/icarus/src/interpreter/` and `.../tokenizer/` stay untouched
by this port. SP (`oracle/code/icarus/`) is punted per DEC-04/§20 — it is a
later SP-diff pass, not a unification (see ICARUS-D5).

**Punts.** The server-side syscall dispatch that routes `G_ICARUS_*` into this
crate's public fns lives in `SV_GameSystemCalls`
(`oracle/codemp/server/sv_game.cpp:739-832`), owned by the server crate, not
here — this doc pins the *callee* signatures (§ Seam definition), the server
doc pins the switch. Frame driving (per-entity `CTaskManager::Update`) is the
game module's `G_ICARUS_MAINTAINTASKMANAGER` trap each frame
(`sv_game.cpp:763-773`); this doc owns `Update`, the server owns the trap arm.

## Raven ground truth

CITE OR OMIT. All cites are `oracle/codemp/icarus/…` unless noted.

**Two-directional seam.** ICARUS is engine-side but talks to the game module
both ways:

- **Inbound** (game→engine syscalls): 19 `G_ICARUS_*` arms in
  `SV_GameSystemCalls` call this crate's public C-linkage fns
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
  `game_export_t.rs`); the game-side `GAME_ICARUS_*` handlers are already wired
  in `mp_game` (`crates/mp/game/src/g_main.rs`).

**Init order.** `ICARUS_Init` (`GameInterface.cpp:143-149`): `Interface_Init(
&interface_export )` populates the `I_*` table, then
`iICARUS = ICARUS_Instance::Create( &interface_export )`. Per-entity setup is
`ICARUS_InitEnt` (`GameInterface.cpp:646-677`): allocates that entity's
`CSequencer`/`CTaskManager` into `gSequencers[ent->s.number]` /
`gTaskManagers[ent->s.number]`. Teardown: `ICARUS_FreeEnt`
(`GameInterface.cpp:220-256`) → `iICARUS->DeleteSequencer(...)` and nulls both
tables; `ICARUS_Shutdown` (`GameInterface.cpp:166-…`) walks and frees.

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
per-class modules below as fields. It is defined at
`crates/mp/engine/icarus/src/icarus.rs` and re-exported from the crate root
(Files roster), mirroring the `mp_engine_core::Engine` aggregate's
`engine.rs` placement (`crates/mp/engine/core/src/engine.rs:20`); its per-field
composition is the State-ownership table here. Its **attachment as a field of
the `mp_engine_core::Engine` island is not settled by this doc**: `Engine`
carries no §F-subcrate field yet (`crates/mp/engine/core/src/engine.rs:35-36`),
and the `Engine`-field attachment point for all four §F engine subcrates
(botlib/ghoul2/icarus/rmg) is tracked by **STATE-Q2**
(`docs/architecture/state-ownership.md:470-476`) — see ICARUS-Q6. ICARUS-D3
fixes only that the state lives on `Icarus`, not where `Icarus` hangs.

| Raven global | oracle cite | Rust owner (`Icarus.field`) | constructed by | threaded via |
| --- | --- | --- | --- | --- |
| `ICARUS_Instance *iICARUS` | `GameInterface.cpp:16` | `Icarus.instance: Option<IcarusInstance>` | `ICARUS_Init` | `&mut Icarus` into ICARUS_* fns |
| `CSequencer *gSequencers[MAX_GENTITIES]` | `Instance.cpp:19`, `g_public.h:723` | `Icarus.sequencers: Box<[Option<CSequencer>; MAX_GENTITIES]>` | `ICARUS_InitEnt` | index by `ent.s.number` |
| `CTaskManager *gTaskManagers[MAX_GENTITIES]` | `Instance.cpp:20` | `Icarus.task_managers: Box<[Option<CTaskManager>; MAX_GENTITIES]>` | `ICARUS_InitEnt` | index by `ent.s.number` |
| `bufferlist_t ICARUS_BufferList` | `GameInterface.cpp:17` | `Icarus.buffer_list: HashMap<String, Pscript>` | `ICARUS_RegisterScript` | `&mut Icarus` |
| `entlist_t ICARUS_EntList` | `GameInterface.cpp:18` | `Icarus.ent_list: HashMap<String, i32>` | `ICARUS_Init`/interrogate | `&mut Icarus` |
| `int ICARUS_entFilter = -1` | `GameInterface.cpp:23` | `Icarus.ent_filter: i32` | init `= -1` | `&Icarus` |
| `interface_export_t interface_export` | `Q3_Interface.cpp:32` | `Icarus.interface_export: InterfaceExport` | `Interface_Init` | stored, borrowed by sequencers |
| `varString_m varStrings` | `Q3_Registers.cpp:9` | `Icarus.var_strings: HashMap<String, String>` | `Q3_InitVariables` | `&mut Icarus` |
| `varFloat_m varFloats` | `Q3_Registers.cpp:10` | `Icarus.var_floats: HashMap<String, f32>` | `Q3_InitVariables` | `&mut Icarus` |
| `varString_m varVectors` | `Q3_Registers.cpp:11` | `Icarus.var_vectors: HashMap<String, String>` | `Q3_InitVariables` | `&mut Icarus` |
| `int numVariables = 0` | `Q3_Registers.cpp:13` | `Icarus.num_variables: i32` | `Q3_InitVariables` (reset), inc/dec in `Q3_Declare/FreeVariable` | `&mut Icarus` |
| ICARUS Zone allocations (`TAG_ICARUS2-5`) | `Memory.cpp:12`, `blockstream.h:66` etc. | subsumed by owned Rust objects / `Icarus.arena` (ICARUS-D4) | object construction | ownership |

Container-internal counters (`m_GUID`, `m_count`, `m_numCommands`, group
completion maps) stay fields of their owning Rust type — not globals.

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

**Inbound — this crate's public API** (callees of `SV_GameSystemCalls`
`G_ICARUS_*` arms, `sv_game.cpp:739-832`). Signatures thread the ICARUS
subsystem + the server host explicitly (fork-2, no globals); the exact host
parameter type is **ICARUS-Q1** (unresolved). Shown below with a placeholder
`host: &mut IcarusHost` standing for that context:

```rust
// game_interface — the G_ICARUS_* callees
pub fn icarus_init(host: &mut IcarusHost);
pub fn icarus_shutdown(host: &mut IcarusHost);
pub fn icarus_run_script(host: &mut IcarusHost, ent_num: i32, name: &str) -> bool;
pub fn icarus_register_script(host: &mut IcarusHost, name: &str, called_during_interrogate: bool) -> bool;
pub fn icarus_valid_ent(host: &mut IcarusHost, ent_num: i32) -> bool;
pub fn icarus_init_ent(host: &mut IcarusHost, ent_num: i32);
pub fn icarus_free_ent(host: &mut IcarusHost, ent_num: i32);
pub fn icarus_associate_ent(host: &mut IcarusHost, ent_num: i32);
pub fn icarus_is_initialized(host: &IcarusHost, ent_num: i32) -> bool;   // gSequencers/gTaskManagers presence
pub fn icarus_maintain_task_manager(host: &mut IcarusHost, ent_num: i32) -> bool; // -> CTaskManager::Update
pub fn icarus_is_running(host: &IcarusHost, ent_num: i32) -> bool;
// NOTE: `Svcmd_ICARUS_f` (GameInterface.h:32, GameInterface.cpp:700) is NOT a
// G_ICARUS_* arm callee — no such arm exists (sv_game.cpp:739-832) and it has
// zero callers/command-table registrations anywhere in codemp. Its drop-vs-keep
// is unsettled — see ICARUS-Q5. It is deliberately absent from this seam list.

// q3_registers — variable store. q3_variable_declared / q3_set_var / q3_get_*
// are the G_ICARUS_VARIABLEDECLARED / SETVAR / GET*VARIABLE arm callees.
// q3_declare_variable / q3_free_variable have NO G_ICARUS_* arm — they are the
// OUTBOUND I_DeclareVariable / I_FreeVariable interface_export targets
// (Interface_Init, Q3_Interface.cpp:1002-1003), reached only through the
// InterfaceExport table, not the server syscall switch.
pub fn q3_declare_variable(host: &mut IcarusHost, var_type: i32, name: &str);  // I_DeclareVariable target (not a G_ICARUS arm)
pub fn q3_free_variable(host: &mut IcarusHost, name: &str);                    // I_FreeVariable target (not a G_ICARUS arm)
pub fn q3_variable_declared(host: &IcarusHost, name: &str) -> i32;
pub fn q3_set_var(host: &mut IcarusHost, task_id: i32, ent_num: i32, type_name: &str, data: &str);
pub fn q3_get_float_variable(host: &IcarusHost, name: &str) -> Option<f32>;   // out-param -> Option (§C7)
pub fn q3_get_string_variable(host: &IcarusHost, name: &str) -> Option<String>;
pub fn q3_get_vector_variable(host: &IcarusHost, name: &str) -> Option<[f32; 3]>;

// q3_interface — task-id helpers (G_ICARUS_TASKID*)
pub fn q3_task_id_set(host: &mut IcarusHost, ent_num: i32, task_type: taskID_t, task_id: i32);
pub fn q3_task_id_complete(host: &mut IcarusHost, ent_num: i32, task_type: taskID_t);
pub fn q3_task_id_pending(host: &IcarusHost, ent_num: i32, task_type: taskID_t) -> bool;
```

**Outbound — `InterfaceExport`** (ICARUS-D2: plain Rust struct of `fn` items,
fork-5; **not** `#[repr(C)]` — it never crosses the module ABI). The `I_*`
signatures mirror `interface.h:17-70`; each impl writes a `T_G_ICARUS_*` struct
into `sv.mSharedMemory` and issues `VM_Call(gvm, GAME_ICARUS_*)`. The
host-threading of these `fn` items is the other half of **ICARUS-Q1**. The
`T_G_ICARUS_*` payloads and `GAME_ICARUS_*` ids are reused from `mp_qshared`
verbatim (already ported); this crate does not redefine them.

**Traps / cross-crate calls used by this crate** (rendered by
`abi-traps.md` / qcommon): `FS_ReadFile`/`FS_FreeFile`
(`GameInterface.cpp:373-379`), `Z_Malloc`/`Z_Free` (`Memory.cpp`),
`Com_Printf`, `Q_flrand` (`I_Random`, `Q3_Interface.cpp:978`),
`VM_Call(gvm, …)` and `sv.mSharedMemory` (the outbound path). These are
qcommon/server surfaces this crate calls, not new exports.

**Layout types (unchanged, already ported).** `T_G_ICARUS_*` structs +
`game_export_t` + `taskID_t` (the `q3_task_id_*` enum arg,
`oracle/codemp/game/g_public.h:625-639`, ported at
`crates/mp/qshared/src/common/mp/qcommon/task_id_t.rs`) — all in `mp_qshared`;
`setType_e`, `playType_e`, `pscript_t`, `vector_t` skeletons already in
`crates/mp/engine/icarus/src/`. The existing
`#[repr(C)] interface_export_s` skeleton
(`crates/mp/engine/icarus/src/interface/interface_export_s.rs`) is a type-port
artifact **superseded** by the plain `fn`-item `InterfaceExport` under
ICARUS-D2 (ICARUS internal types are not ABI-crossing, so layout is free — §F).

## Decisions

**ICARUS-D1.** Closed C++ class hierarchies → Rust structs (no virtuals to
model here beyond `ICARUS_Instance`'s `Save*/Load*`, which are inert per the
ground truth); intrusive `std::list`/`vector`/`map` members and raw
child/parent pointers → owned `Vec`/`VecDeque`/`HashMap`/`String`, entities by
index. Because §F makes ICARUS internals non-ABI (they cross only behind the
engine boundary), layout is free (porting-rules §17, GP2 exemplar
`crates/mp/engine/qcommon/src/gp2/`). Rejected byte-faithful `#[repr(C)]`
transcription of the class tree: it would freeze `Z_Malloc`-pointer layout the
seam never observes.

**ICARUS-D2.** `interface_export_t` → a plain Rust `InterfaceExport` struct of
`fn` items, populated at `Interface_Init`, per fork-5. Because the table never
crosses the module ABI and no code compares member addresses, a `fn`-item
struct is 1:1 with zero indirection. Rejected `#[repr(C)]`
`Option<extern "C" fn>` (the type-port skeleton) — needless ABI shape for an
engine-internal table; rejected a trait object — fork-5 ruled bare `fn` items.

**ICARUS-D3.** All ICARUS state (the singleton, per-entity sequencer/taskmanager
tables, script/variable stores, block pools) lives on a `mp_engine_icarus::Icarus`
sub-struct of the engine aggregate, per fork-2; no `static mut`. Because fork-2
relocated every engine global to its owning subsystem struct. Rejected retained
globals and thread-locals — fork-2 forbids `static mut` on parity paths. This
decision fixes only that ICARUS state lives on `Icarus`; **where `Icarus`
attaches to the `mp_engine_core::Engine` island is out of scope here**, tracked
by STATE-Q2 (`docs/architecture/state-ownership.md:470-476`) — see ICARUS-Q6.

**ICARUS-D4.** `ICARUS_Malloc`/`ICARUS_Free` → a faithful owned arena, per
fork-4 (faithful `Vec<u8>`-backed arenas, idiomatization deferred to safe-state
migration). Ground truth: in MP these route to the engine Zone
(`Z_Malloc(TAG_ICARUS5, qfalse)` / `Z_Free`), not a private pool; the faithful
model is the engine Zone reached under the ICARUS tags, plus `Icarus.arena` for
any residual raw block-data blobs. ICARUS-D1's owned objects remove the
`Z_Malloc(TAG_ICARUS2/3/4)` `operator new` allocations, leaving only the
`ICARUS_Malloc` (`TAG_ICARUS5`) raw-blob path; per ICARUS-D6 (parity-first,
refactor behind the passing diff) the faithful `Icarus.arena` is still built for
the parity port, and folding that residual path into plain `Vec<u8>` is a
post-parity refactor deferred to the safe-state migration (this decision's own
"idiomatization deferred" clause). It changes no referee-diffed state: the arena
never crosses the game seam and MP save/load is inert (Divergences,
`Q3_Interface.cpp:695-704`), so allocation order is unobservable.

**ICARUS-D5.** Verification = differential goldens against the unmodified oracle
TUs under `tools/icarus-oracle/` (porting-rules §18, DEC-09 layer 1, GP2/
`tools/gp2-oracle` pattern), MP-only; the SP tree (`oracle/code/icarus/`) is a
later SP-diff pass (§20), not unified now. Rejected building the full Raven
engine (DEC-09) and rejected live-only acceptance (goldens must be committed so
`cargo test` needs no C++ toolchain, §18).

**ICARUS-D6.** Transcription-first: method bodies transcribe into the ICARUS-D1
shape with no safety refactoring; parity green first, refactor behind the
passing diff (porting-rules §A2, GOAL-engine ground rules). The safe-state
migration is a separate post-parity phase.

## Files roster

C++-track roster for `port-cpp-subsystem` (`designPath` consumer). All
`crate: mp_engine_icarus`, `mode: mp`. Existing type-port files are reshaped in
place under ICARUS-D1/D2; new files mirror the owning Raven header subsystem
(one class per file, porting-rules §21).

files:
- path: `crates/mp/engine/icarus/src/icarus.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `Icarus`, summary: the cross-cutting ICARUS subsystem aggregate `pub struct Icarus` — the type the State-ownership table composes (`Icarus.instance`, `Icarus.sequencers`, `Icarus.task_managers`, `Icarus.buffer_list`, `Icarus.ent_list`, `Icarus.ent_filter`, `Icarus.interface_export`, `Icarus.var_strings/floats/vectors`, `Icarus.num_variables`, `Icarus.arena`; ICARUS-D3/D4). Not a Raven class — the fork-2 synthesized owner of every ICARUS file-scope global; placed in its own file re-exported from the crate root, mirroring `mp_engine_core::Engine` in `engine.rs`. Its `Engine`-field attachment is STATE-Q2/ICARUS-Q6, not wired here.
- path: `crates/mp/engine/icarus/src/lib.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `(crate root)`, summary: add the module declarations the roster below requires — new `pub mod icarus; pub mod sequence; pub mod taskmanager; pub mod instance; pub mod memory; pub mod q3_registers;` alongside the existing `blockstream`/`game_interface`/`interface`/`q3_interface`/`sequencer` (and the out-of-scope `interpreter`/`tokenizer` skeletons, kept untouched per Scope), plus `pub use icarus::Icarus;`. Mechanical wiring only, no logic.
- path: `crates/mp/engine/icarus/src/blockstream/cblock_member.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CBlockMember`, summary: one ID/size/data record; owned `Vec<u8>` data replacing `void* m_data`; `WriteMember`/`ReadMember` IBI serialization, `WriteData`/`WriteDataPointer`, `Duplicate` (`blockstream.h:38-105`, `BlockStream.cpp`).
- path: `crates/mp/engine/icarus/src/blockstream/cblock.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CBlock`, summary: owns `Vec<CBlockMember>`; block id + flags; `Write` overloads, `AddMember`/`GetMember`, `Duplicate` (`blockstream.h:109-154`).
- path: `crates/mp/engine/icarus/src/blockstream/cblock_stream.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CBlockStream`, summary: `.IBI` reader/writer over an owned byte buffer; `Open`/`ReadBlock`/`WriteBlock`/`BlockAvailable`, `IBI` header+version check (`blockstream.h:158-196`).
- path: `crates/mp/engine/icarus/src/blockstream/vector_t.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `vector_t`, summary: `[f32; 3]` alias used by block writes (existing skeleton, kept).
- path: `crates/mp/engine/icarus/src/blockstream/file.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CBlockStream::file`, summary: owned file-handle helper for `.IBI` I/O replacing `FILE*` (existing skeleton).
- path: `crates/mp/engine/icarus/src/sequence/csequence.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CSequence`, summary: command/child tree node; owned `Vec` children + command list, parent/return by handle; flags/iterations, `Save/Load` (inert) (`sequence.h:12-96`, `Sequence.cpp`).
- path: `crates/mp/engine/icarus/src/sequencer/csequencer.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CSequencer`, summary: per-entity script driver; parses IBI blocks into sequences; `Run`/`Callback`/`Parse*`/`Check*`/`Push/PopCommand`; holds interface + taskmanager handles (`sequencer.h:68-187`, `Sequencer.cpp`, ~43 fns).
- path: `crates/mp/engine/icarus/src/sequencer/bstream_s.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `bstream_t`, summary: internal stream-stack node; intrusive `last` pointer folds into the sequencer's owned `Vec` (existing skeleton, `sequencer.h:42-46`).
- path: `crates/mp/engine/icarus/src/taskmanager/ctask.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CTask`, summary: a scheduled task (GUID, timestamp, owned `CBlock`) (`taskmanager.h:33-58`).
- path: `crates/mp/engine/icarus/src/taskmanager/ctask_group.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CTaskGroup`, summary: completion-tracking group; `HashMap<i32,bool>` completed set, parent handle, `MarkTaskComplete`/`Complete` (`taskmanager.h:62-93`).
- path: `crates/mp/engine/icarus/src/taskmanager/ctask_manager.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CTaskManager`, summary: per-entity scheduler; `Update` heartbeat (`Go`, `RUNAWAY_LIMIT`), the `Rotate/Camera/Print/Sound/Move/Set/Use/…/Wait/WaitSignal` handlers, owned task maps/lists (`taskmanager.h:97-189`, `TaskManager.cpp`, ~52 fns).
- path: `crates/mp/engine/icarus/src/instance/icarus_instance.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `ICARUS_Instance`, summary: top singleton; owns its own sequence/sequencer pools (`m_sequences`/`m_sequencers`) + signal map (`m_signals`); `Create`/`Delete`, `Signal`/`CheckSignal`/`ClearSignal`, inert `Save*/Load*` (`instance.h:12-79`, `Instance.cpp`, ~24 fns). The per-entity `gSequencers`/`gTaskManagers` arrays are file-scope globals (`Instance.cpp:19-20`), not `ICARUS_Instance` members — they are fields of the `Icarus` subsystem struct per the State-ownership table (ICARUS-D3), not owned here.
- path: `crates/mp/engine/icarus/src/interface/interface_export_s.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `interface_export_t`, summary: reshape the `#[repr(C)]` type-port skeleton into the plain `fn`-item `InterfaceExport` table (ICARUS-D2) (`interface.h:17-70`).
- path: `crates/mp/engine/icarus/src/memory/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `IcarusArena`, summary: `ICARUS_Malloc`/`ICARUS_Free` faithful arena over the engine Zone tags, built for the parity port per ICARUS-D4/D6 (the `Vec<u8>` fold is a deferred post-parity refactor, not skipped here) (`Memory.cpp:8-20`, `icarus.h:29-30`).
- path: `crates/mp/engine/icarus/src/q3_interface/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `Q3Interface`, summary: the `Q3_*` `I_*` implementations (shared-memory write + `VM_Call`) and `Interface_Init` wiring; `Q3_Evaluate`, camera stubs, task-id helpers (`Q3_Interface.cpp`, ~49 fns).
- path: `crates/mp/engine/icarus/src/q3_interface/set_type_t.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `setType_e`, summary: SET_* enum (existing skeleton, kept).
- path: `crates/mp/engine/icarus/src/q3_interface/play_type_t.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `playType_e`, summary: PLAY_* enum (existing skeleton, kept).
- path: `crates/mp/engine/icarus/src/q3_registers/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `Q3Registers`, summary: `varStrings`/`varFloats`/`varVectors` stores + `Q3_InitVariables`/`Q3_Declare/Free/Get/Set*Variable`, `MAX_VARIABLES`, `VTYPE_*` (`Q3_Registers.cpp`, `.h:4-32`, ~16 fns).
- path: `crates/mp/engine/icarus/src/game_interface/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `GameInterface`, summary: `ICARUS_RunScript`/`RegisterScript`/`GetScript`/`Init`/`InitEnt`/`FreeEnt`/`ValidEnt`/`AssociateEnt`/`Shutdown`/`LinkEntity`/`SoundPrecache`/`InterrogateScript` + the buffer/ent-list state; `Svcmd_ICARUS_f` (zero-caller stub) is pending the drop-vs-keep fork **ICARUS-Q5** — not ported until resolved (`GameInterface.cpp`, ~14 live fns).
- path: `crates/mp/engine/icarus/src/game_interface/pscript_s.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `pscript_t`, summary: cached script record; owned buffer replacing `char* buffer` (existing skeleton, `GameInterface.h:4-8`).

`Interface.cpp` produces **no** Rust file — its `Interface_Init` is commented
out (dead surface, see Divergences); the live `Interface_Init` is in
`q3_interface/mod.rs`.

## Divergences

Raven-UB / dead-surface points where the port picks one defined behavior
(porting-rules §19; kept out of / normalized in the shared golden corpus).
`port-cpp-subsystem` `divergences`:

- MP save/load is inert: `AppendToSaveGame`/`ReadFromSaveGame` (`I_WriteSaveData`/`I_ReadSaveData` targets) both `return 1;` with no I/O (`Q3_Interface.cpp:695-704`); the `ICARUS_Instance`/`CSequencer`/`CSequence`/`CTaskManager` `Save`/`Load` methods port as structurally-present but effect-free, and are excluded from the golden corpus.
- `Interface.cpp`'s `Interface_Init` is commented out (`Interface.cpp:14-24`); it links no code. The port emits no Rust unit for it — the live table wiring lives in `q3_interface/mod.rs` (`Q3_Interface.cpp:956`).
- `gSequencers`/`gTaskManagers` are indexed by entnum with **no bounds check** on the hot paths: the `sv_game` seam arms `G_ICARUS_ISINITIALIZED`/`MAINTAINTASKMANAGER`/`ISRUNNING` index `gSequencers[entID]`/`gTaskManagers[entID]` with `entID = args[1]` taken straight from the syscall args, unchecked (`oracle/codemp/server/sv_game.cpp:752-784`), and `ICARUS_RunScript`/`ICARUS_InitEnt` index `gSequencers[ent->s.number]` unchecked (`GameInterface.cpp:75`, `:650-661`) — only `ICARUS_FreeEnt` guards `s.number >= MAX_GENTITIES || < 0` (`GameInterface.cpp:224-229`). In C a negative or `>= MAX_GENTITIES` entnum is an out-of-bounds pointer read (UB); on the Rust `[Option<_>; MAX_GENTITIES]` fixed arrays the same index panics. This is **not** a resolved divergence: the §19 defined-behavior choice for an OOB entnum (guard-and-return like `FreeEnt`, clamp, or accept the panic as the chosen behavior) is unsettled — see **ICARUS-Q4**.
- `ICARUS_Malloc` is non-zeroed (`Z_Malloc(...,qfalse)`, `Memory.cpp:12`) while class `operator new` is zeroed (`Z_Malloc(...,qtrue)`, `blockstream.h:66`); under ICARUS-D1 owned objects are always value-initialized, so the zero/non-zero distinction collapses to defined initialization — `WriteDataPointer` still `memcpy`s exact bytes, preserving content parity.

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
  bytes)` record stream. Cleanest TU (only `ICARUS_Malloc`/`Z_Malloc` deps).
- **Q3_Registers** — script variable operations: dump `varStrings/varFloats/
  varVectors` state after a scripted `Declare/Set/Get/Free` sequence.
- **Sequencer + TaskManager + Instance (end-to-end)** — drive
  `ICARUS_RunScript` on a committed `.IBI` fixture with a **mock**
  `interface_export`/`VM_Call` that records the ordered `I_*` / `GAME_ICARUS_*`
  callback trace and scripted return values; the golden is that ordered
  callback stream plus final variable/signal state. This exercises
  `Parse*`/`Check*`, task scheduling, `Wait`/`WaitSignal`, and `Callback`.

Fixture provenance (committed `.IBI` blobs) is **ICARUS-Q3** — the compiler is
out of scope, so blobs are supplied, not generated here.

Live-peer acceptance (DEC-09 layer 2) is the server-crate/full-engine gate
(M4-M5), not this subsystem's unit gate.

## Slice hooks

- **M2 (waves 7-12), GOAL-engine** — "icarus complete … icarus differential
  goldens." Needs frozen first: this doc's § Seam definition (ICARUS-Q1
  resolved), the `tools/icarus-oracle` harness, and the `.IBI` fixture corpus
  (ICARUS-Q3).
- **Server crate `SV_GameSystemCalls`** — the `G_ICARUS_*` dispatch arms
  (`sv_game.cpp:739-832`) call this crate's inbound public API; that switch
  ports in the server slice and depends on the inbound signatures here (and on
  ICARUS-Q1's host type). The outbound `GAME_ICARUS_*` handlers already exist
  in `mp_game`.
- **Memory/Zone** — ICARUS-D4's arena rests on the ported engine Zone
  (qcommon wave-0/1), which lands before ICARUS in the port order.

## Open questions

MUST be empty at FROZEN. These are unresolved by the settled inputs + oracle
ground truth and escalate to an interactive session (never self-resolved).

- **ICARUS-Q1 — host/context threading shape.** ICARUS-D2 makes
  `interface_export` bare `fn` items and ICARUS-D3 removes the `sv`/`gvm`/`svs`
  globals the `I_*` bodies (`sv.mSharedMemory` + `VM_Call`) and the inbound
  `ICARUS_*` fns reach. A bare `fn` cannot capture the relocated state, so each
  fn item / public fn must receive the engine context — but neither D2 nor D3
  pins whether that is a `&mut Engine` first parameter, a narrower
  `&mut IcarusHost` handle, or a trait the sequencer calls through. The reach is
  wider than the sequencer/taskmanager files: the otherwise-pure variable store
  also needs the host/print capability — `Q3_DeclareVariable` and
  `Q3_InitVariables` call `Q3_DebugPrint` (`Q3_Registers.cpp:58,77,199`), which
  reads the `com_developer` cvar and calls `Com_Printf`
  (`Q3_Interface.cpp:638`), both host surfaces. So the host parameter threads
  through `q3_registers` too, not just the sequencer path. The §Seam signatures
  use a placeholder `IcarusHost`; the real type must be settled before the seam
  freezes. NEEDS SESSION.
- **ICARUS-Q6 — `Icarus` attachment to the `Engine` island (cross-doc,
  STATE-Q2).** ICARUS-D3 fixes that all ICARUS state lives on
  `mp_engine_icarus::Icarus`, but not where that struct hangs on the engine
  aggregate: `mp_engine_core::Engine` carries no §F-subcrate field today
  (`crates/mp/engine/core/src/engine.rs:35-36`), and the `Engine`-field
  attachment point for all four §F engine subcrates (botlib/ghoul2/icarus/rmg)
  is an owned open question of the state-ownership doc — **STATE-Q2**
  (`docs/architecture/state-ownership.md:470-476`), which explicitly leaves
  "whether/how their engine-side state attaches to the `Engine` island"
  unsettled. Until STATE-Q2 resolves, `Icarus` cannot be wired into `Engine` as
  a field, and the ICARUS-Q1 host type (which threads that same engine context
  inward) cannot be pinned either. This doc does not resolve STATE-Q2; it is
  logged here so the fork is visible from the ICARUS side. Tracked by STATE-Q2;
  NEEDS SESSION.
- **ICARUS-Q3 — golden-fixture provenance.** The MP engine consumes
  precompiled `.IBI` (interpreter/tokenizer out of scope), so the differential
  harness needs committed `.IBI` blobs. Are retail `.IBI` assets available to
  commit as fixtures, or must minimal `.IBI` be produced by the out-of-scope
  SP/tool compiler (or hand-authored)? D5 sets the method but not the fixture
  source. NEEDS SESSION (may resolve at harness build).
- **ICARUS-Q4 — out-of-range entnum behavior (§19 fork).** The inbound
  presence-check arms take entnum straight from `args[1]` with no bounds check
  (`oracle/codemp/server/sv_game.cpp:752-784`), and `ICARUS_RunScript`/
  `ICARUS_InitEnt` index `gSequencers[ent->s.number]` unchecked
  (`GameInterface.cpp:75`, `:650-661`); only `ICARUS_FreeEnt` guards
  `s.number >= MAX_GENTITIES || < 0` (`GameInterface.cpp:224-229`). In C an
  out-of-range entnum is an out-of-bounds pointer read (UB, may be non-NULL and
  pass the `!= NULL` gate); on the Rust `[Option<_>; MAX_GENTITIES]` owner
  (`icarus_is_initialized`/`icarus_maintain_task_manager`/`icarus_is_running`/
  `icarus_run_script`/`icarus_init_ent`, § Seam definition) the same index
  panics. porting-rules §19 requires picking one defined behavior, but the
  inputs do not settle which: guard-and-return the C `FreeEnt` idiom (the only
  path that already bounds-checks), clamp/mask the index, or accept the panic as
  the chosen defined behavior. The settled ICARUS-D* decisions do not cover this
  fork and it cannot be resolved from oracle ground truth alone. NEEDS SESSION.
- **ICARUS-Q5 — `Svcmd_ICARUS_f` drop-vs-keep (§20 fork).** `Svcmd_ICARUS_f`
  (decl `GameInterface.h:32`, def `GameInterface.cpp:700`) has an
  entirely-commented-out body (a `rwwFIXMEFIXME` debug stub,
  `GameInterface.cpp:702-730`) and **zero** callers or command-table
  registrations anywhere in the codemp tree — a grep across `server/`, `game/`,
  `qcommon/`, `cgame/` finds it only in its own decl + def. It is **not** a
  `G_ICARUS_*` arm callee: there is no such arm in `SV_GameSystemCalls`
  (`sv_game.cpp:739-832`). porting-rules §20 makes zero-caller API droppable
  with a module-doc note, but the alternative — keep it structurally-present —
  requires identifying where MP registers the console command, and oracle ground
  truth shows that registration **nowhere**. The settled ICARUS-D* decisions do
  not cover it, so drop-vs-keep cannot be settled mechanically. NEEDS SESSION.
