# ICARUS sequencer — MP engine (§F idiomatic reimplementation) Design
Status: FROZEN (user sign-off 2026-07-09)     Supersedes: none
Decision prefix: ICARUS     Ledger deps: DEC-09, DEC-10; engine-fork-discovery forks 2, 4, 5, 7 + §F rulings 11–39, **40, 43**; STATE-Q2 (CLOSED 2026-07-09); GOAL-engine M2 gate

## Standing context

Links only — never restate:

- `docs/workspace-architecture.md` — crate graph; ICARUS lives in the
  near-isolate crate `mp_engine_icarus` (436 internal edges, 25 into qcommon,
  6 into server; `docs/plans/2026-07-08-mp-engine-build-out.md` §graph rows).
- `docs/porting-rules.md` — §F (C++-track idiomatic reimplementation, GP2
  exemplar), §17–21 (design-before-transcription, differential parity, UB
  divergence, one-class-per-file), §12 (internal-only types get idiomatic Rust
  shape/naming), comment/source-ref rules.
- `docs/decisions.md` — DEC-09 (engine verification: TU golden harnesses +
  live peers), DEC-10 (checkpoint/reset skeleton build).
- `docs/handoffs/engine-fork-discovery.md` — fork-2 (global placement → Engine
  fields, no `static mut`), fork-4 (faithful owned arenas), fork-5 (dispatch
  tables → plain Rust structs of `fn` items), fork-7 (the 5-doc §F list, ICARUS
  named); and the §F doc-session **rulings 11–39** plus the **final-revision
  rulings 40 + 43** (user, 2026-07-09) that settle this doc. This pass-7 revision
  folds in the two final rulings and consolidates the decision records to three:
  **ICARUS-D1** = ruling 40 (BlockStream `Init` is live — closes ICARUS-Q13 — and
  the project-wide §F naming rule); **ICARUS-D2** = ruling 43 + the STATE-Q2
  closure (the concrete `EngineHostView<'a>` split-borrow); **ICARUS-D3** = all
  rulings 11–39 stand (37: `ConvertedEntity` faithful copy semantics; 36:
  15-method `EngineHost` re-quoted; 27: faithful `Vec` arenas; 39a's *genuine*
  zero-caller drops; 39d id-newtype placement; etc.).
- `docs/architecture/state-ownership.md` — **STATE-Q2 CLOSED 2026-07-09**
  (rulings 12/13 + 43, `:1860-1876`): each §F subsystem's engine-side state is a
  direct `Engine` field reached through the `EngineHostView` split-borrow. This
  doc and `crates/mp/engine/core/src/engine.rs:35-38` are **reconciled** (the
  `engine.rs` comment now records STATE-Q2 CLOSED and the `EngineHostView`
  constructors — commit `163952dc`); both are cited as authoritative here, not
  as stale placeholders.
- `docs/GOAL-engine.md` — Stage-0 game-host interface crate checklist (`:43-55`,
  the crate's *role* bullet), M2 gate ("icarus differential goldens"). Ruling 24
  pinned the crate path `crates/mp/host-interface` / package `mp_host_interface`
  (`use mp_host_interface::EngineHost;`); ruling 36 extended and rebuilt it to the
  15-method surface (commit `a9820853`), which § Seam definition quotes verbatim.
- GP2 exemplar: `crates/mp/engine/qcommon/src/gp2/`, `tools/gp2-oracle/`.

## Scope & non-goals

**In scope.** The 10 `WinDed.vcproj`-Release-linked ICARUS sources
(`oracle/codemp/icarus/`): `BlockStream.cpp`, `GameInterface.cpp`,
`Instance.cpp`, `Interface.cpp`, `Memory.cpp`, `Q3_Interface.cpp`,
`Q3_Registers.cpp`, `Sequence.cpp`, `Sequencer.cpp`, `TaskManager.cpp` —
253 fns / 6,749 LOC (plan §graph row). Every method ports except the
zero-caller / genuinely-dead fns that porting-rules §20 makes droppable (§20's
criterion is zero callers — or transitively caller-less). Target crate:
`mp_engine_icarus` (MP only).

**§20-dropped surface (settled by ICARUS-D1, ruling 40).** Not ported, each
recorded with a module-doc zero-caller note:
- `Svcmd_ICARUS_f` (`GameInterface.h:32`, `GameInterface.cpp:700-730`) — ICARUS-D3
  (ruling 17): commented-out body, zero callers/registrations, no `G_ICARUS_*` arm.
- `ICARUS_Malloc` / `ICARUS_Free` (`Memory.cpp:8-20`, `icarus.h:29-30`) — ICARUS-D3
  (ruling 20): the arena is dropped entirely, so these have zero live callers under
  the owned-buffer shape.
- **BlockStream's writer/duplicator half — exactly FIVE methods (ICARUS-D1,
  ruling 40, corrects ruling 39a's over-broad list):**
  1. `CBlockStream::Create(char *)` (`blockstream.h:167`, def `BlockStream.cpp:525`,
     the `fopen`-style file writer) — **zero callers** in `oracle/codemp/icarus/`
     (every `Create(` site resolves to `CBlock::Create(int)`, `CSequence::Create`,
     `ICARUS_Instance::Create`, `CTaskManager::Create`, or `CTask::Create`; none
     reaches `CBlockStream::Create`).
  2. `CBlockStream::WriteBlock` (`blockstream.h:174`, def `:577`) — only callers in
     the out-of-set `Interpreter.cpp`.
  3. `CBlockMember::WriteMember` (`blockstream.h:47`, def `:133`) — its only caller is
     the dropped `WriteBlock` (`BlockStream.cpp:597`).
  4. `CBlock::Duplicate` (`blockstream.h:138`, def `:359`) — **zero callers** (grep finds
     only its internal member-`Duplicate` self-call at `:374`).
  5. `CBlockMember::Duplicate` (`blockstream.h:74`, def `:148`) — its only caller is the
     dead `CBlock::Duplicate` (`:374`), so transitively caller-less.
  All five §20-dropped with a `blockstream/` module-doc note.

**These BlockStream writer-named methods PORT — they are live (ICARUS-D1
corrects pass-6's over-drop).** They are *not* in ruling 40's drop set:
- `CBlockStream::Init` (`blockstream.h:165`, def `BlockStream.cpp:558`) — the live reader
  `CBlockStream::Open` (`:665`, called `GameInterface.cpp:465`, `Sequencer.cpp:318`,`:370`)
  calls `Init()` at `BlockStream.cpp:670`; the writer `Create(char *)` (`:525`) does not.
  So `Init` has a live caller and fails §20's zero-caller test. **This closes
  ICARUS-Q13** (ruling 39a had mis-listed `Init` among the dead methods; the cited
  oracle contradicts that, so ruling 40 rules it live).
- `CBlock::Write` overloads (`blockstream.h:125-129`, def `BlockStream.cpp:244-300`) —
  **live**: they are pure in-memory member builders (`new CBlockMember` +
  `SetData`/`WriteData` + `AddMember`, `:244-300`, no file I/O) called ~20× on the
  parse path (`Sequence.cpp:503-538`, `TaskManager.cpp:1863-1887`,
  `Sequencer.cpp:396,429,479,541,714`) to construct the blocks a sequence/task pushes.
- `CBlockMember::WriteData` (`blockstream.h:76`) / `WriteDataPointer` (`:88`) /
  `SetData` (`:56-58`) — **live**: `CBlock::Write` calls `WriteData`
  (`BlockStream.cpp:278`,`:291`) and `SetData` (`:250`,`:265`), and `SetData` calls
  `WriteDataPointer` (`:76`,`:81`); `SetData` is also reached from `TaskManager.cpp:1111`,`:1132`.
  These are the in-memory member-data copies, not the dropped file writer.

The reader/build half — `CBlockStream::Open` (`:665`), `Init` (`:558`, live),
`ReadBlock` (`:619`), `BlockAvailable` (`:605`), the `Get*` stream primitives,
`CBlock::Create(int)` (`:201`), `CBlock::Init`/`Free`/`AddMember`/`GetMember`/`Write`,
`CBlockMember::ReadMember` (`:102`)/`SetData`/`WriteData`/`WriteDataPointer` — is
**live** (drives `ICARUS_InterrogateScript` `GameInterface.cpp:465-476` and
`CSequencer::Route`/parse `Sequencer.cpp:318,799-802`) and ports in full.

**Out of scope.** `Interpreter.cpp` and `Tokenizer.cpp` are **not** in the link
set. The dedicated server never compiles scripts: `ICARUS_RegisterScript`
appends `.IBI` and `FS_ReadFile`s a **precompiled** block-instruction blob
(`oracle/codemp/icarus/GameInterface.cpp:346-395`); the `.txt`→`.IBI` compiler
(interpreter/tokenizer) is an offline/SP tool — reused only once, out-of-tree,
to build the golden corpus (ICARUS-D3, ruling 14). The type-port skeletons under
`crates/mp/engine/icarus/src/interpreter/` and `.../tokenizer/` stay untouched
by this port. SP (`oracle/code/icarus/`) is a later SP-diff pass (DEC-04/§20),
not a unification now — see § Verification strategy.

**Punts.** The server-side syscall dispatch that routes `G_ICARUS_*` into this
crate's public fns lives in `SV_GameSystemCalls`
(`oracle/codemp/server/sv_game.cpp:739-832`), owned by the server crate, not
here — this doc pins the *callee* signatures (§ Seam definition), the server
doc pins the switch. **`ConvertedEntity` (`sv_game.cpp:420-451`) is ported by
the server crate, faithfully (ICARUS-D3, ruling 37).** It is in the 2,481-fn
engine-port-order list (sv_game.cpp wave-20 territory); the server crate ports it
1:1 and routes the five entity-field `G_ICARUS_*` arms through it exactly as Raven.
This doc owns only the ICARUS callee signatures (which take the pointer
`ConvertedEntity` returns — a pointer to the copy). **Flag for the
server-dispatch doc:** the three `G_ICARUS_TASKID*` arms pass `(taskID_t)args[2]`
— an unchecked int→enum cast of an arbitrary `intptr_t` (`sv_game.cpp:786`,
`:808`, `:813`). Constructing a `taskID_t` (repr enum) from an out-of-range
`args[2]` is Raven UB the server crate must resolve as a porting-rules §19 checked
conversion **before** it calls this crate's `task_type: taskID_t` callees; Raven's
own callee guard (`taskType < TID_CHAN_VOICE || taskType >= NUM_TIDS` → no-op /
`qfalse`, `Q3_Interface.cpp:116-118` / `:136-138` / `:169-171`) pins the
out-of-range outcome (see § Seam definition task-id note, Divergences). Frame
driving (per-entity `CTaskManager::Update`) is the game module's
`G_ICARUS_MAINTAINTASKMANAGER` trap each frame (`sv_game.cpp:763-773`); this doc
owns `Update`, the server owns the trap arm.

**The Stage-0 `EngineHost` crate is built, extended compiled code (ICARUS-D3,
rulings 31 + 33 + 36).** The trait lives in the green Stage-0 crate
`crates/mp/host-interface`, package `mp_host_interface` (commit `a9820853`; `use
mp_host_interface::EngineHost;`) — a crate of Rust traits transcribing the C seam
(syscall surface, vmcall driver, shared-memory contract). Ruling 36 extended it
from 10 to **15** methods; § Seam definition quotes the **actual** 15-method trait
**verbatim** from `crates/mp/host-interface/src/engine_host.rs`. ICARUS binds a
subset of that frozen roster, not a paper spec. Every §F fn takes `(&mut Icarus,
&mut dyn EngineHost)` (ICARUS-D3, ruling 24 — `dyn`, not `impl`, forced by the
fn-pointer `InterfaceExport` table). Porters `use` the crate directly (it
precedes ICARUS in the port order, § Slice hooks), never a stub. This mirrors the
sibling §F docs, which reach the same crate (`docs/subsystems/roff.md`,
`docs/subsystems/ghoul2-server.md` non-goals).

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

**Init flags Raven itself uses** (ground truth for ICARUS-D3's "Raven's own
initialized flags", ruling 12): the instance is live iff `iICARUS != NULL` — the
NULL guard gates `ICARUS_Init`'s error path (`:151`), `ICARUS_Shutdown` (`:202`),
and `assert( iICARUS )` in `InitEnt`/`FreeEnt` (`:649`, `:222`). A per-entity
sequencer/taskmanager is live iff `gSequencers[n]`/`gTaskManagers[n] != NULL` —
the exact flag the `ISINITIALIZED`/`MAINTAINTASKMANAGER`/`ISRUNNING` arms read
(`sv_game.cpp:756`, `:767`, `:778`) and the `RunScript`/`InitEnt`/`FreeEnt`
NULL checks use (`GameInterface.cpp:75`, `:653`, `:232`). These pointer-NULL
flags are Raven's initialized state; the port models them, not a redundant
Rust wrapper.

**Entity-field access — the five entity-field arms carry a `sharedEntity_t *`
that points to `ConvertedEntity`'s by-value copy; the three task-id arms carry a
bare real-entity pointer; only the three presence-check arms carry an int
entnum** (ground truth for ICARUS-D3, ruling 37, which corrects ruling 23's
rationale). The C ICARUS ent-fns and task-id helpers take a `sharedEntity_t *ent`
and dereference fields on it: `ICARUS_RunScript` reads `ent->classname`/
`ent->targetname` for its verbose log (`GameInterface.cpp:129`) and indexes
`gSequencers[ent->s.number]` (`:75`); `ICARUS_InitEnt` walks `ent->s.number`/
`ent->taskID` and `memset( &ent->taskID, -1, sizeof( ent->taskID ) )`
(`GameInterface.cpp:660`, `:664`); `ICARUS_FreeEnt`/`ICARUS_AssociateEnt`/
`ICARUS_ValidEnt` read `ent->script_targetname`/`ent->targetname` (`:236`, `:311`,
`:273`); `Q3_TaskIDPending`/`Complete`/`Set` read and write `ent->taskID[taskType]`
and `ent->s.number` (`Q3_Interface.cpp:111-183`). These fields live on
`sharedEntity_t` (`taskID[NUM_TIDS]` at `oracle/codemp/game/g_public.h:694`,
`script_targetname` `:697`, `classname` `:703`; ported at
`crates/mp/qshared/src/common/mp/qcommon/shared_entity_t.rs`).

**`ConvertedEntity` returns a pointer to a by-value copy — and its writes are
dropped** (`sv_game.cpp:420-451`, ground truth for ICARUS-D3 / ruling 37). The
function copies `ent->s`, `ent->r`, and the whole `taskID[NUM_TIDS]` array **by
value** into a file-static `sharedEntity_t gLocalModifier` (`:426-433`), re-points
its pointer fields (`parms`, `behaviorSet[]`, `script_targetname`, `fullName`,
`targetname`, `classname`) through `VM_ArgPtr(...)` (`:435-448`), copies `ghoul2`,
and **returns `&gLocalModifier`** (`:450`) — a pointer to the copy, **not** to the
caller's entity. Consequently, any field a callee **writes** through that pointer
lands in `gLocalModifier` and is **discarded** when the next `ConvertedEntity`
call overwrites it — including `ICARUS_InitEnt`'s `memset( &ent->taskID, -1, … )`
(`GameInterface.cpp:664`), which in retail therefore never zeroes the *real*
entity's task IDs. The port reproduces this exactly: the server crate ports
`ConvertedEntity` faithfully and hands the copy pointer to the five entity-field
callees, whose bodies write through it as Raven does, so the drop is emergent, not
special-cased. (This is why the "shuffle is a no-op / pass-through" rationale of
ruling 23 is **struck** — the copy is a distinct object whose writes do not reach
the real entity.)

**Which arm carries what** (`sv_game.cpp:739-832`). The **five** ent-fns get
`ConvertedEntity((sharedEntity_t *)VMA(1))` — `ICARUS_RunScript` (`:740`),
`ICARUS_ValidEnt` (`:750`), `ICARUS_InitEnt` (`:789`), `ICARUS_FreeEnt` (`:793`),
`ICARUS_AssociateEnt` (`:797`) — so their `ent` is the **copy** pointer. The
**three** task-id helpers get a **bare** cast `(sharedEntity_t *)VMA(1)` —
`Q3_TaskIDPending` (`:786`), `Q3_TaskIDSet` (`:808`), `Q3_TaskIDComplete` (`:813`)
— so their `ent` is the **real** entity (their `ent->taskID[]` writes therefore
persist, unlike `InitEnt`'s dropped `memset`). The **three** presence-check arms
carry an int: `G_ICARUS_ISINITIALIZED` (`:752`), `G_ICARUS_MAINTAINTASKMANAGER`
(`:763`), `G_ICARUS_ISRUNNING` (`:774`) each read `int entID = args[1]` and index
`gSequencers[entID]`/`gTaskManagers[entID]` inline (`:754-782`).

**The `gentity()` service is reached from six ICARUS sites — one write-back, one
teardown sweep, two outbound `I_*` slots, one per-frame freeze gate, and one
debug-log read** (oracle re-grep of `SV_GentityNum` in `oracle/codemp/icarus/*.cpp`
finds `GameInterface.cpp:169`,`:174`,`:288`,`:681`; `Q3_Interface.cpp:238`,`:679`;
`TaskManager.cpp:324`). The `s.number → *mut sharedEntity_t` step is the
`EngineHost::gentity` service (`SV_GentityNum`, `sv_game.cpp:54-59`, over the
`G_LOCATE_GAME_DATA` base `SV_LocateGameData` installs, `:329-330`, arm `:567`).
Every site below reaches that one service; **none changes a public seam signature**
(all keep the shapes § Seam definition freezes — the point is which fn bodies bind
`gentity`):

1. **`ICARUS_ValidEnt`** (`GameInterface.cpp:268-297`, `SV_GentityNum(ent->s.number)`
   `:288`) — for an entity carrying a `behaviorSet` but no `script_targetname`
   (`:277-294`) it must write `trueEntity->script_targetname = trueEntity->targetname`
   (`:291`); because `ConvertedEntity` handed it the **copy**, that write must reach
   the *true* entity through `gentity`, not through `ent`.
2. **`ICARUS_Shutdown`** (`GameInterface.cpp:166-186`) — carries **no** entity arg,
   so it walks `SV_GentityNum(i)` over all `MAX_GENTITIES` (`:169`,`:174`) and feeds
   the **real** pointer into `ICARUS_FreeEnt(ent)` (`:184`) wherever `gSequencers[i]`
   is live; genuinely index-based (`icarus_shutdown` calls `host.gentity(i)` per slot,
   unlike the arm-fed copy `FreeEnt` gets inbound).
3. **`ICARUS_LinkEntity`** (`GameInterface.cpp:679-692`, `SV_GentityNum(entID)` `:681`)
   — the **outbound** `I_LinkEntity` slot (wired `Q3_Interface.cpp:1008`, called live
   from `Sequencer.cpp:2413`); resolves `entID`, wires `gSequencers`/`gTaskManagers`,
   then `ICARUS_AssociateEnt(ent)` on the real pointer (`:686-689`).
4. **`Q3_GetEntityByName`** (`Q3_Interface.cpp:221-241`, `SV_GentityNum((*ei).second)`
   `:238`) — the **outbound** `I_GetEntityByName` slot (wired `:964`, called live from
   `Sequencer.cpp:605`,`:673`,`:1733`,`:1795` — core script entity-targeting): name →
   `ICARUS_EntList` entnum → real entity pointer returned to the sequencer.
5. **`CTaskManager::Update`** (`TaskManager.cpp:322-338`, `SV_GentityNum(m_ownerID)`
   `:324`) — a **per-frame** gate: reads `owner->r.svFlags & SVF_ICARUS_FREEZE`
   (`g_public.h:35`) and short-circuits to `TASK_FAILED` before `Go()` (`:326-329`);
   reached each frame via `icarus_maintain_task_manager` (`m_ownerID` is the int
   entnum, `taskmanager.h:173`).
6. **`Q3_DebugPrint`** (`Q3_Interface.cpp:679`, `SV_GentityNum(entNum)->script_targetname`)
   — only in the `WL_DEBUG` branch and only past the `com_developer` gate (`:642`,
   ICARUS-D3): reads the owner's `script_targetname` for the log line.

`ICARUS_AssociateEnt` (`:307-318`) is **not** a `gentity` site: it only **reads**
`ent->s.number` into `ICARUS_EntList[…]` (`:317`) — present in the copy (copied by
value), so the read is faithful and no re-fetch is needed. Under ruling 37 this
aligns: reads off the copy are correct, writes off the copy are dropped, and only
`ValidEnt` writes back to the true entity.

**Debug-print developer gate** (ground truth for ICARUS-D3, ruling 36).
`Q3_DebugPrint` gates **all** output on the engine cvar `com_developer` before
touching `Com_Printf`: `if (!com_developer || !com_developer->integer) return;`
(`Q3_Interface.cpp:638-643`, esp. `:642`). `com_developer` is an engine cvar
(`extern cvar_t *com_developer`, `oracle/codemp/qcommon/qcommon.h:688`; defined
`common.cpp:39`, registered `Cvar_Get("developer","0",CVAR_TEMP)` `common.cpp:1307`)
— settable at runtime, so it is **not** safely assumable `0` in the dedicated
build. `Q3_DebugPrint` is live and reached in-scope (called from
`Q3_Registers.cpp:58,77,199`, `GameInterface.cpp:129`, and ~15 `Q3_Interface.cpp`
verbose/warning/error sites). Ruling 36 added `EngineHost::cvar_integer`, which
collapses Raven's `com_developer->integer` read to `Cvar_VariableIntegerValue`
(`cvar.cpp:118-124`, unregistered name → 0); the gate ports as
`host.cvar_integer("developer") != 0` before any `host.print(...)`.

**Class tree.** Closed hierarchies, intrusive `std::` containers. (Rust names per
the §F naming rule, ICARUS-D1; the roster records the Raven class name in its
`class:` column.)
- `CBlockMember`→**`BlockMember`** (`blockstream.h:38-105`): `{int m_id; int m_size;
  void* m_data}` — one ID/size/data record; `SetData`/`WriteData`/`WriteDataPointer`
  build the member data in memory; `operator new` = `Z_Malloc(TAG_ICARUS4, qtrue)`.
  `ReadMember` (reader), `SetData`/`WriteData`/`WriteDataPointer` (in-memory builders)
  are live; only `WriteMember`/`Duplicate` §20-dropped (ICARUS-D1).
- `CBlock`→**`Block`** (`blockstream.h:109-154`): owns `vector<CBlockMember*> m_members`,
  `int m_id`, `unsigned char m_flags`. `Create(int)`/`Init`/`Free`/`AddMember`/
  `GetMember` **and the `Write` overloads** (`:125-129`, def `BlockStream.cpp:244-300`,
  live in-memory member builders) all port; only `Duplicate` §20-dropped (ICARUS-D1).
- `CBlockStream`→**`BlockStream`** (`blockstream.h:158-196`): `.IBI` reader over
  `char* m_stream`; `IBI_HEADER_ID "IBI"`, `IBI_VERSION 1.57f`. Reader
  `Open`/`ReadBlock`/`BlockAvailable`/`Init` (live — `Open` calls `Init` at `:670`)
  port; only the file writer `Create(char *)`/`WriteBlock` §20-dropped (ICARUS-D1).
- `CSequence`→**`Sequence`** (`sequence.h:12-96`): tree node — `list<CSequence*> m_children`,
  `m_parent`, `m_return`, `list<CBlock*> m_commands`, `m_flags`, `m_iterations`,
  `m_id`; `operator new` = `Z_Malloc(TAG_ICARUS3)`.
- `CSequencer`→**`Sequencer`** (`sequencer.h:68-187`): per-entity driver — `list<CSequence*>
  m_sequences`, `map<CTaskGroup*,CSequence*> m_taskSequences`,
  `vector<bstream_t*> m_streamsCreated`, holds `interface_export_t* m_ie`,
  `CTaskManager* m_taskManager`, `ICARUS_Instance* m_owner`; `Run`, `Callback`,
  the `Parse*`/`Check*`/`Push/PopCommand` machinery; `operator new` =
  `Z_Malloc(TAG_ICARUS2)`. `bstream_t` (`sequencer.h:42-46`) is an intrusive
  stream-stack node.
- `CTask`→**`Task`** (`taskmanager.h:33-58`), `CTaskGroup`→**`TaskGroup`**
  (`taskmanager.h:62-93`, `map<int,bool> m_completedTasks`), `CTaskManager`→**`TaskManager`**
  (`taskmanager.h:97-189`): `map<int,CTask*>`, `map<string,CTaskGroup*>`, `map<int,CTaskGroup*>`,
  `vector<CTaskGroup*>`, `list<CTask*>`; `Update` is the per-frame heartbeat
  (`Go`), the `Rotate/Camera/Print/Sound/Move/Set/Use/…/Wait/WaitSignal`
  handlers, `RUNAWAY_LIMIT=256` (`taskmanager.h:15`).
- `ICARUS_Instance`→**`IcarusInstance`** (`instance.h:12-79`): the singleton —
  `list<CSequence*>`, `list<CSequencer*>`, `map<string,unsigned char> m_signals`,
  `m_interface`, `m_GUID`; `Signal`/`CheckSignal`/`ClearSignal` and the `virtual`
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
per-class modules below as fields. Per ICARUS-D3 the `pub struct Icarus` is
defined in `crates/mp/engine/icarus/src/lib.rs` (the crate root), which also
declares the per-class module dirs, and attaches to the engine as a **plain,
`Default`-initialized `icarus` field directly on `mp_engine_core::Engine`** —
no `Option`, no `Box`, no nesting (ruling 12,
`docs/handoffs/engine-fork-discovery.md:126-130`; STATE-Q2 CLOSED,
`state-ownership.md:1860-1876`). The field lands with this subsystem's port waves
and is reached through the `EngineHostView` split-borrow (ICARUS-D2, ruling 43);
`engine.rs:35-38` records exactly this (reconciled, commit `163952dc`).
"Is ICARUS initialized?" is answered by Raven's own NULL-flags
(Raven-ground-truth §Init flags), not by wrapping the subsystem in `Option`.

| Raven global | oracle cite | Rust owner (`Icarus.field`) | constructed by | threaded via |
| --- | --- | --- | --- | --- |
| `ICARUS_Instance *iICARUS` | `GameInterface.cpp:16` | `Icarus.instance: Option<IcarusInstance>` (the `Option` mirrors Raven's own `iICARUS != NULL` flag, not a subsystem wrapper). Per ICARUS-D3 (ruling 27) `IcarusInstance` **owns** `sequences: Vec<Sequence>` + `sequencers: Vec<Sequencer>` — its Raven `m_sequences`/`m_sequencers` lists (`instance.h:62-63`) — keyed by `SequenceId`/`SequencerId` newtypes carrying `m_GUID` | `ICARUS_Init` | `&mut Icarus` into ICARUS_* fns |
| `CSequencer *gSequencers[MAX_GENTITIES]` | `Instance.cpp:19`, `g_public.h:723` | `Icarus.sequencers: Box<[Option<SequencerId>; MAX_GENTITIES]>` — **non-owning** per-entity index into `IcarusInstance.sequencers` (ICARUS-D3, ruling 27; the object is owned by the instance arena, `GetSequencer` `STL_INSERT( m_sequencers, … )` `Instance.cpp:174`; `Option`/`None` = Raven's per-entity NULL flag) | `ICARUS_InitEnt` | index by `ent.s.number` → `SequencerId` |
| `CTaskManager *gTaskManagers[MAX_GENTITIES]` | `Instance.cpp:20` | `Icarus.task_managers: Box<[Option<TaskManager>; MAX_GENTITIES]>` — **owning** (ruling 27 leaves task-manager objects here; Raven inserts them in no instance list — `GetSequencer` creates one per sequencer, `Instance.cpp:166-182` — and no sequencer→taskmanager handle is stored, so they are reached by ent index) | `ICARUS_InitEnt` | index by `ent.s.number` |
| `bufferlist_t ICARUS_BufferList` | `GameInterface.cpp:17` | `Icarus.buffer_list: HashMap<String, Pscript>` | `ICARUS_RegisterScript` | `&mut Icarus` |
| `entlist_t ICARUS_EntList` | `GameInterface.cpp:18` | `Icarus.ent_list: HashMap<String, i32>` | `ICARUS_Init`/interrogate | `&mut Icarus` |
| `int ICARUS_entFilter = -1` | `GameInterface.cpp:23` | `Icarus.ent_filter: i32` | init `= -1` | `&Icarus` |
| `interface_export_t interface_export` | `Q3_Interface.cpp:32` | `Icarus.interface_export: InterfaceExport` | `Interface_Init` | stored, borrowed by sequencers |
| `varString_m varStrings` | `Q3_Registers.cpp:9` | `Icarus.var_strings: HashMap<String, String>` | `Q3_InitVariables` | `&mut Icarus` |
| `varFloat_m varFloats` | `Q3_Registers.cpp:10` | `Icarus.var_floats: HashMap<String, f32>` | `Q3_InitVariables` | `&mut Icarus` |
| `varString_m varVectors` | `Q3_Registers.cpp:11` | `Icarus.var_vectors: HashMap<String, String>` | `Q3_InitVariables` | `&mut Icarus` |
| `int numVariables = 0` | `Q3_Registers.cpp:13` | `Icarus.num_variables: i32` | `Q3_InitVariables` (reset), inc/dec in `Q3_Declare/FreeVariable` | `&mut Icarus` |
| ICARUS Zone allocations (`TAG_ICARUS2-5`) | `Memory.cpp:12`, `blockstream.h:66` etc. | `TAG_ICARUS2/3/4` class allocs → owned Rust objects; `TAG_ICARUS5` raw blobs → owned `Vec<u8>` on their owning types (`BlockMember::m_data`, `Pscript::buffer`) — **no arena** (ICARUS-D3, ruling 20; `ICARUS_Malloc`/`ICARUS_Free` §20-dropped) | object construction | ownership |

Container-internal counters (`m_GUID`, `m_count`, `m_numCommands`, group
completion maps) stay fields of their owning Rust type — not globals.

**`Icarus::default()` is hand-written, not derived.** ICARUS-D3 (ruling 12) fixes
`Icarus` as a `Default`-initialized field on `Engine`, but `#[derive(Default)]` on
`Icarus` is both unavailable and wrong: (1) the two `Box<[Option<_>;
MAX_GENTITIES]>` slot arrays have no blanket `[T; N]: Default` impl (N>0) and
must be constructed explicitly (e.g. `Box::new(std::array::from_fn(|_| None))`),
and (2) `ent_filter` must seed `-1` — derive yields `0`, diverging from Raven's
`int ICARUS_entFilter = -1` (`GameInterface.cpp:23`). This is parity-visible,
not cosmetic: `ICARUS_entFilter` is read at two `Com_Printf`/`Q3_DebugPrint`
verbose-gate sites (`GameInterface.cpp:127`, `Q3_Interface.cpp:670`) and its
**only** writer is `Svcmd_ICARUS_f` (`GameInterface.cpp:722`) — the fn ICARUS-D3
(ruling 17) §20-drops. So in this MP-dedicated build `ent_filter` stays `-1` for
the process lifetime; a `0` seed would flip the debug-print gating, which the
engine plan's console routing feeds into the referee syscall digest. The
hand-written `impl Default` therefore seeds `ent_filter = -1` and
`interface_export` with the real `Q3_*`/`I_*` fns (see
`interface/interface_export_s.rs`). Seeding `interface_export` at `Default` time
is not an invented pre-init state (ruling 20): Raven's own initialized flag is
`iICARUS != NULL` (`Icarus.instance: Some(_)`), which flips **only after**
`ICARUS_Init` runs `Interface_Init(&interface_export)` and then
`ICARUS_Instance::Create` (`GameInterface.cpp:143-156`) — so no `I_*` slot is ever
observed before `Interface_Init` (re-)assigns the identical fn; the seed just
satisfies Rust's construct-before-use for a bare-`fn` table (fork-5) and preserves
Raven's Interface-Init timing.

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

**Host services — `EngineHost`** (ICARUS-D3, rulings 11 + 24 + 36). Every engine
service ICARUS needs is reached **only** through the single `EngineHost` trait in
the built Stage-0 crate `crates/mp/host-interface`, package `mp_host_interface`
(commit `a9820853`; `docs/plans/2026-07-08-mp-engine-build-out.md:250-251`,
`docs/GOAL-engine.md:43-55`). A porter imports it verbatim:

```rust
use mp_host_interface::EngineHost;
```

Ruling 36 **extended** the trait from 10 to **15** methods (adding `cvar_integer`,
`sv_time`, `fs_write_file`, `model_mdxm`, `model_mdxa`) and rebuilt it. The roster
below is the trait's **actual** signature set, quoted **verbatim** from
`crates/mp/host-interface/src/engine_host.rs`. ICARUS binds the subset noted after
the quote; the rest are listed because the trait is one shared surface across all
five §F subsystems.

```rust
// VERBATIM from crates/mp/host-interface/src/engine_host.rs (rulings 24 + 33 + 36).
pub trait EngineHost {
    /// Raven `SV_Trace` — sweep a box through the collision world, writing the
    /// result into `results` (kept as an out-param to transcribe the NPCNav
    /// call sites `SV_Trace( &trace, ... )` 1:1; `capsule` is Raven's
    /// `qboolean`, idiomatic `bool` per porting-rules §C7).
    /// Source: `oracle/codemp/server/sv_world.cpp:803`
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

    /// Raven `FS_ReadFile` — read a file whole; `None` mirrors Raven's `-1`
    /// length / `NULL` buffer (missing file). The returned `Vec` is the file
    /// bytes (its `len()` is Raven's returned length); FS_ReadFile's trailing
    /// NUL is an FS-impl detail, not part of the contract.
    /// Source: `oracle/codemp/qcommon/files.cpp:1670`
    fn fs_read_file(&mut self, qpath: &str) -> Option<Vec<u8>>;

    /// Raven `FS_FreeFile` — release a buffer from [`fs_read_file`]. Consuming
    /// the `Vec` keeps the read/free pairing at the call site; ownership makes
    /// the free itself a drop (default no-op).
    /// Source: `oracle/codemp/qcommon/files.cpp:1798`
    ///
    /// [`fs_read_file`]: EngineHost::fs_read_file
    fn fs_free_file(&mut self, _buffer: Vec<u8>) {}

    /// Raven `Com_Printf` — print pre-formatted text. Raven's varargs collapse
    /// to a formatted `&str` at the call site (porting-rules §C).
    /// Source: `oracle/codemp/qcommon/common.cpp:128`
    fn print(&mut self, msg: &str);

    /// Raven `Com_Error` — diverts through the panic + `catch_unwind` model
    /// (ruling fork-1): the payload carries `code` + `msg`, so this never
    /// returns. `code` is `errorParm_t` (enum fidelity over Raven's `int`).
    /// Source: `oracle/codemp/qcommon/common.cpp:249`
    fn error(&mut self, code: errorParm_t, msg: &str) -> !;

    /// Raven `VM_Call( vm, callnum, ... )` — invoke a loaded module. `vm`
    /// mirrors Raven's first parameter ([`VmSlot::Gvm`]/[`VmSlot::Cgvm`],
    /// ruling 33b); args are `intptr_t`-width slots (ruling 6); the return is
    /// `intptr_t` too, since ROFF casts it straight to a pointer
    /// (`RoffSystem.cpp:837`). The icarus arms pass no args (their request
    /// travels through [`shared_memory`]); NPCNav's `gameCallbacks` pass up
    /// to seven.
    /// Source: `oracle/codemp/qcommon/vm.cpp:787`
    ///
    /// [`shared_memory`]: EngineHost::shared_memory
    fn vm_call(&mut self, vm: VmSlot, callnum: i32, args: &[isize]) -> isize;

    /// Raven `sv.mSharedMemory` — the `char *` window the game handed over via
    /// `G_SET_SHARED_BUFFER`. A subsystem writes its `T_G_ICARUS_*` request
    /// struct here, then [`vm_call`]s the matching game export.
    /// Source: `oracle/codemp/server/server.h:87` (`sv_game.cpp:940` arms it)
    ///
    /// [`vm_call`]: EngineHost::vm_call
    fn shared_memory(&mut self) -> *mut c_char;

    /// Raven `Q_flrand` — a float `min <= x < max` off the engine's own
    /// `q_math.c` `holdrand` LCG instance (ruling 21: a qshared `QRand`-type
    /// field on `Engine.common`, reached through this method).
    /// Source: `oracle/codemp/game/q_math.c:1451`
    fn flrand(&mut self, min: f32, max: f32) -> f32;

    /// Raven `Q_irand` — an integer `min <= x <= max` off the same LCG.
    /// Source: `oracle/codemp/game/q_math.c:1471`
    fn irand(&mut self, min: i32, max: i32) -> i32;

    /// Raven `SV_GentityNum` — the game entity at slot `ent_num`. Returns the
    /// raw `*mut sharedEntity_t` exactly as the trap marshals it (rulings
    /// 19/23/30/37, transcription-first): the entity-taking icarus/NPCNav arms
    /// already carry the pointer, this serves genuinely index-based access.
    /// Source: `oracle/codemp/server/sv_game.cpp:54`
    fn gentity(&mut self, ent_num: i32) -> *mut sharedEntity_t;

    /// Per-call integer cvar read (ruling 36) — collapses Raven's cached
    /// `cvar_t->integer` pattern (a `Cvar_Get`-seeded file-static read at each
    /// gate): `com_developer` (`Q3_Interface.cpp:638-643`),
    /// `cg_g2MarksAllModels` (`G2_misc.cpp:40`, read `:1524`), nav's
    /// `d_altRoutes`/`d_patched` (`navigator.cpp:480,1403,1933`). An
    /// unregistered name reads 0, as `Cvar_VariableIntegerValue` does.
    /// Source: `oracle/codemp/qcommon/cvar.cpp:118-124`
    fn cvar_integer(&mut self, name: &str) -> i32;

    /// Raven `svs.time` — the `serverStatic_t` frame clock ("strictly
    /// increasing across level changes"), consumed by nav's failed-node/edge
    /// recheck timers (`navigator.cpp:1733,1763,1778,1797,1987,2010,2065,
    /// 2137`). NOT the same clock as `PlatformHost::milliseconds`
    /// (`Sys_Milliseconds`, the wall/profiling clock): `svs.time` advances in
    /// fixed frame steps and only while the server runs frames.
    /// Source: `oracle/codemp/server/server.h:211` (`extern svs`: `:232`)
    fn sv_time(&mut self) -> i32;

    /// Whole-file write (ruling 36) — Raven's
    /// `FS_FOpenFileByMode(qpath, &f, FS_WRITE)` + `FS_Write` calls +
    /// `FS_FCloseFile` sequence collapsed; `false` mirrors the NULL-handle
    /// open failure (`CNavigator::Save` returns false there, the live
    /// `G_NAV_SAVE` arm).
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:670-699`
    /// (`FS_FOpenFileByMode`: `files.cpp:3547`, `FS_Write`: `files.cpp:1477`)
    fn fs_write_file(&mut self, qpath: &str, data: &[u8]) -> bool;

    /// Loader model memory, mesh half (ruling 36 / G2SV-D5) — Raven
    /// `R_GetModelByHandle( model )->mdxm`: the raw pointer to the parsed
    /// `.glm` block (`mdxmHeader_t` at offset 0). `c_void` because the mdx
    /// header types are `mp_renderer`-owned and never named at this seam
    /// (G2SV-D5); NULL exactly where Raven's pointer is NULL (not a GL2M
    /// model). No re-parsing — this is the loader's live block.
    /// Source: `oracle/codemp/renderer/tr_local.h:1128` (`model_t.mdxm`);
    /// chain: `oracle/codemp/ghoul2/G2_API.cpp:2716-2721`
    /// (`R_GetModelByHandle`: `tr_model.cpp:593`)
    fn model_mdxm(&mut self, model: qhandle_t) -> *mut c_void;

    /// Loader model memory, animation half (ruling 36 / G2SV-D5) — Raven
    /// `R_GetModelByHandle( model )->mdxa`: the raw pointer to the parsed
    /// `.gla` block (`mdxaHeader_t` at offset 0; `CBoneCache` parent seeding,
    /// skeleton build, and ragdoll basepose resolve do byte arithmetic off
    /// it, `tr_ghoul2.cpp:416-421,614-615`). Callers reach the anim handle
    /// via the mesh header's `animIndex`, as `G2_SetupModelPointers` does.
    /// Source: `oracle/codemp/renderer/tr_local.h:1129` (`model_t.mdxa`);
    /// chain: `oracle/codemp/ghoul2/G2_API.cpp:2735-2739`
    fn model_mdxa(&mut self, model: qhandle_t) -> *mut c_void;
}
```

**ICARUS binds** (annotations, not edits to the verbatim quote): `fs_read_file`/
`fs_free_file` — `ICARUS_RegisterScript` reads/frees the precompiled `.IBI` blob
(`GameInterface.cpp:374`, `:393`); `print` — the verbose logs and `Q3_DebugPrint`
output (`GameInterface.cpp:129`, `Q3_Interface.cpp` sites); **`cvar_integer` —
`Q3_DebugPrint`'s `com_developer` gate** (`Q3_Interface.cpp:642`), ported as
`host.cvar_integer("developer") != 0` (ICARUS-D3, ruling 36); `error` —
`ICARUS_Init`'s NULL-instance `ERR_DROP` (`GameInterface.cpp:151-155`);
`vm_call(VmSlot::Gvm, GAME_ICARUS_*, &[])` + `shared_memory()` — the outbound
`Q3_*` path (write the `T_G_ICARUS_*` struct through the raw `*mut c_char` window,
then `VM_Call` with no args since the request travels in shared memory, e.g.
`Q3_PlaySound` `Q3_Interface.cpp:315-322`); `flrand`/`irand` — the `I_Random`
slot (`Q3_Interface.cpp:978`, called `Sequencer.cpp:532` etc.); `gentity(ent_num)`
— reached from **six** bodies (§ Raven ground truth, the gentity-site inventory):
`icarus_valid_ent`'s behaviorSet write-back (`GameInterface.cpp:288`),
`icarus_shutdown`'s per-slot teardown sweep (`:169`/`:174`),
`icarus_maintain_task_manager` → `CTaskManager::Update`'s `SVF_ICARUS_FREEZE` gate
(`TaskManager.cpp:324`), `Q3_DebugPrint`'s `WL_DEBUG` log line
(`Q3_Interface.cpp:679`), and the two **outbound** `InterfaceExport` slots
`I_LinkEntity` (`GameInterface.cpp:681`) and `I_GetEntityByName`
(`Q3_Interface.cpp:238`). The remaining methods (`trace`, `sv_time`,
`fs_write_file`, `model_mdxm`, `model_mdxa`) serve sibling §F subsystems (NPCNav,
ghoul2); ICARUS binds none of them (roster continuity across the five §F
subsystems).

Every §F fn takes `(&mut Icarus, &mut dyn EngineHost, …)` — `dyn`, not `impl`,
because the `InterfaceExport` slot fns are stored `fn` pointers that ICARUS-D3
(ruling 24) fixes on `&mut dyn EngineHost` and several pub seam fns double as those
slot targets (a bare `fn` pointer cannot be generic over `impl EngineHost`).
**`Engine` implements `EngineHost` through the concrete split-borrow view
(ICARUS-D2, ruling 43):** `pub struct EngineHostView<'a>` in `mp_engine_core`
holds `&mut` borrows of the `Common`/`Server`/`CollisionWorld`/loader fields the
trait needs, and `Engine::icarus_call(&mut self) -> (EngineHostView<'_>, &mut
Icarus)` splits the `icarus` field off the rest by plain field-level reborrowing
(no unsafe); the caller passes the view as `&mut dyn EngineHost` alongside `&mut
Icarus`. The trait impl lands at wave 20 with the `SV_GameSystemCalls` arms that
consume it (STATE-Q2 CLOSED, `state-ownership.md:1860-1876`). The goldens inject
ruling 32's fixture-backed `MockHost` (`crates/mp/host-interface/src/mock.rs`) in
place of the view — deterministic `flrand`/`irand` off the replicated `holdrand`
LCG, a recording `vm_call`, a strided `gentity` arena, and (ruling 36) a `cvars:
BTreeMap<String, i32>` fixture that answers `cvar_integer` (`mock.rs:121`, `:260`;
missing name → 0). This crate declares no engine globals and calls no
`sv`/`svs`/`gvm` singletons directly — they arrive through `host`.

**Inbound — this crate's public API** (callees of `SV_GameSystemCalls`
`G_ICARUS_*` arms, `sv_game.cpp:739-832`). Uniform §F shape (`&mut Icarus, &mut
dyn EngineHost`, then the arm's args):

Body note (ICARUS-D3, ruling 37): the five entity-field fns carry the **pointer**
their arm passes — `*mut sharedEntity_t`, exactly the trap payload
(`ConvertedEntity((sharedEntity_t *)VMA(1))`), which **points to the by-value copy**
`gLocalModifier`. Reads off it are faithful (fields copied by value); **writes off
it are dropped** exactly as retail (`ICARUS_InitEnt`'s `memset(&ent->taskID,-1)`).
The three task-id helpers carry the **bare** `(sharedEntity_t *)VMA(1)` cast — the
**real** entity, so their `ent->taskID[]` writes persist. Dereferencing any of
these pointers is the confined ABI-seam `unsafe` (porting-rules §D11). They do
**not** take an `ent_num: i32`. The **three presence-check** callees keep `ent_num:
i32` — their arms read `int entID = args[1]` (`sv_game.cpp:752-782`).
Among the inbound entity-field callees, `gentity(ent_num)` is reached only inside
`icarus_valid_ent` (`SV_GentityNum(ent->s.number)` for the behaviorSet write-back to
the *true* entity, since the copy's write would be dropped, `GameInterface.cpp:288`/
`:291`); `icarus_associate_ent` does **not** call it — it only reads `ent->s.number`
off the copy (`:317`). The other five `gentity` bodies are the no-arg
`icarus_shutdown` sweep, the per-frame `icarus_maintain_task_manager`
(`CTaskManager::Update`), `Q3_DebugPrint`, and the two outbound `I_*` slots
(`I_LinkEntity`, `I_GetEntityByName`) — see the § Raven ground truth gentity-site
inventory; none takes an `ent: *mut sharedEntity_t` seam arg, so this touches no
signature here.

```rust
// game_interface — the G_ICARUS_* callees. Entity-field arms carry the copy
// pointer (ICARUS-D3/ruling 37); presence-check arms carry int entID.
pub fn icarus_init(icarus: &mut Icarus, host: &mut dyn EngineHost);
pub fn icarus_shutdown(icarus: &mut Icarus, host: &mut dyn EngineHost);
pub fn icarus_run_script(icarus: &mut Icarus, host: &mut dyn EngineHost, ent: *mut sharedEntity_t, name: &str) -> bool;  // arm :740 (ConvertedEntity copy)
pub fn icarus_register_script(icarus: &mut Icarus, host: &mut dyn EngineHost, name: &str, called_during_interrogate: bool) -> bool; // arm :743, no ent
pub fn icarus_valid_ent(icarus: &mut Icarus, host: &mut dyn EngineHost, ent: *mut sharedEntity_t) -> bool;   // arm :750; host.gentity(ent.s.number) inside for behaviorSet write-back to the TRUE entity (:288/:291)
pub fn icarus_init_ent(icarus: &mut Icarus, host: &mut dyn EngineHost, ent: *mut sharedEntity_t);            // arm :789; memset(&ent.taskID,-1) writes the copy -> dropped, faithfully (:664)
pub fn icarus_free_ent(icarus: &mut Icarus, host: &mut dyn EngineHost, ent: *mut sharedEntity_t);            // arm :793
pub fn icarus_associate_ent(icarus: &mut Icarus, host: &mut dyn EngineHost, ent: *mut sharedEntity_t);       // arm :797; reads ent.s.number off the copy into ent_list (:317), no host.gentity
pub fn icarus_is_initialized(icarus: &mut Icarus, host: &mut dyn EngineHost, ent_num: i32) -> bool;   // arm :752, int entID; gSequencers/gTaskManagers presence
pub fn icarus_maintain_task_manager(icarus: &mut Icarus, host: &mut dyn EngineHost, ent_num: i32) -> bool; // arm :763, int entID; -> CTaskManager::Update
pub fn icarus_is_running(icarus: &mut Icarus, host: &mut dyn EngineHost, ent_num: i32) -> bool;       // arm :774, int entID
// `Svcmd_ICARUS_f` (GameInterface.h:32, GameInterface.cpp:700) is §20-dropped
// per ICARUS-D3 (ruling 17) — zero callers, no G_ICARUS_* arm — and is
// intentionally absent from this seam. See Divergences.

// q3_registers — variable store. q3_variable_declared / q3_get_* / q3_declare_variable
// / q3_free_variable are the Q3_Registers.cpp store fns (home q3_registers/mod.rs:
// Q3_VariableDeclared :21, Q3_GetFloatVariable :130, Q3_GetStringVariable :150,
// Q3_GetVectorVariable :170, Q3_DeclareVariable :50, Q3_FreeVariable :91). All
// carry only ints/strings — no entity pointer. q3_variable_declared / q3_get_* are
// the G_ICARUS_VARIABLEDECLARED / GET*VARIABLE arm callees; q3_declare_variable /
// q3_free_variable have NO G_ICARUS_* arm — they are the OUTBOUND I_DeclareVariable
// / I_FreeVariable interface_export targets (Interface_Init, Q3_Interface.cpp:1002-1003).
//
// q3_set_var is NOT a Q3_Registers.cpp store fn — its source is Q3_SetVar
// (Q3_Interface.cpp:337), so its home is q3_interface/mod.rs. It is a thin
// G_ICARUS_SETVAR arm dispatcher (Q3_SetVar(args[1], args[2], VMA(3), VMA(4)),
// sv_game.cpp:817); it carries only ints/strings, no entity pointer. Listed here
// with the other variable-facing arm callees for reader locality; its compilation
// unit is q3_interface, per its source file.
pub fn q3_declare_variable(icarus: &mut Icarus, host: &mut dyn EngineHost, var_type: i32, name: &str);  // I_DeclareVariable target
pub fn q3_free_variable(icarus: &mut Icarus, host: &mut dyn EngineHost, name: &str);                    // I_FreeVariable target
pub fn q3_variable_declared(icarus: &mut Icarus, host: &mut dyn EngineHost, name: &str) -> i32;
pub fn q3_set_var(icarus: &mut Icarus, host: &mut dyn EngineHost, task_id: i32, ent_num: i32, type_name: &str, data: &str);
pub fn q3_get_float_variable(icarus: &mut Icarus, host: &mut dyn EngineHost, name: &str) -> Option<f32>;   // out-param -> Option (§C7)
pub fn q3_get_string_variable(icarus: &mut Icarus, host: &mut dyn EngineHost, name: &str) -> Option<String>;
pub fn q3_get_vector_variable(icarus: &mut Icarus, host: &mut dyn EngineHost, name: &str) -> Option<[f32; 3]>;

// q3_interface — task-id helpers (G_ICARUS_TASKID*). Arms carry the BARE real
// pointer (sharedEntity_t *)VMA(1): PENDING :786, SET :808, COMPLETE :813.
//
// task_type: the arm passes `(taskID_t)args[2]` — an UNCHECKED int→enum cast
// (sv_game.cpp:786/808/813). Each callee body reproduces Raven's own defined
// guard: `if ( taskType < TID_CHAN_VOICE || taskType >= NUM_TIDS ) return`/
// `return qfalse` BEFORE indexing ent->taskID[taskType] (Q3_TaskIDPending
// Q3_Interface.cpp:116-118, Q3_TaskIDComplete :136-138, Q3_TaskIDSet :169-171).
// The residual §19 point is the int→enum CONVERSION itself, which lives at the
// server-dispatch boundary (§ Scope Punts; Divergences); this crate's callees take
// an already-in-range `task_type: taskID_t`.
pub fn q3_task_id_set(icarus: &mut Icarus, host: &mut dyn EngineHost, ent: *mut sharedEntity_t, task_type: taskID_t, task_id: i32);
pub fn q3_task_id_complete(icarus: &mut Icarus, host: &mut dyn EngineHost, ent: *mut sharedEntity_t, task_type: taskID_t);
pub fn q3_task_id_pending(icarus: &mut Icarus, host: &mut dyn EngineHost, ent: *mut sharedEntity_t, task_type: taskID_t) -> bool;
```

**Outbound — `InterfaceExport`** (ICARUS-D3, rulings 24 + fork-5: plain Rust
struct of `fn` items; **not** `#[repr(C)]` — it never crosses the module ABI).
The `I_*` signatures mirror `interface.h:17-70`; each impl writes a `T_G_ICARUS_*`
struct into the host's shared-memory window and issues `VM_Call(gvm, GAME_ICARUS_*)`,
both reached through the `EngineHost` passed alongside `&mut Icarus`. The
`T_G_ICARUS_*` payloads and `GAME_ICARUS_*` ids are reused from `mp_qshared`
verbatim (already ported). Each stored slot is a
`fn(&mut Icarus, &mut dyn EngineHost, …)` pointer — `dyn` because a bare `fn`
pointer is a concrete type and cannot be generic over `impl EngineHost`:

```rust
// interface_export_s.rs — the I_* table (fork-5 bare-fn slots, host param &mut dyn)
pub struct InterfaceExport {
    pub i_play_sound: fn(&mut Icarus, &mut dyn EngineHost, /* T_G_ICARUS_PLAYSOUND args */),
    pub i_random:     fn(&mut Icarus, &mut dyn EngineHost, min: f32, max: f32) -> f32, // → host.flrand
    pub i_declare_variable: fn(&mut Icarus, &mut dyn EngineHost, var_type: i32, name: &str),
    // … one slot per interface.h:17-70 I_* entry, all fn(&mut Icarus, &mut dyn EngineHost, …)
}
```

That `dyn` slot type propagates to every pub seam fn that doubles as a slot target
(e.g. `q3_declare_variable`/`q3_free_variable` are the `I_DeclareVariable`/
`I_FreeVariable` targets), which is why the inbound signatures above all take
`&mut dyn EngineHost`.

**Traps / cross-crate calls used by this crate** — all reached through
`EngineHost`, never as direct globals; the Raven-name → real `EngineHost` method
map: `FS_ReadFile`/`FS_FreeFile` (`GameInterface.cpp:374`, `:393`) →
`fs_read_file`/`fs_free_file`, `Com_Printf` → `print`, `com_developer->integer`
(`Q3_Interface.cpp:642`) → `cvar_integer("developer")` (ICARUS-D3, ruling 36),
`Com_Error` → `error`, `Q_flrand` (`I_Random`, `Q3_Interface.cpp:978`) →
`flrand`/`irand`, `VM_Call(gvm, …)` → `vm_call(VmSlot::Gvm, …)` plus the
`shared_memory() -> *mut c_char` window (the outbound path), and `gentity(ent_num)`
— reached from **six** bodies (§ Raven ground truth inventory): `icarus_valid_ent`'s
behaviorSet write-back (`GameInterface.cpp:288`/`:291`), `icarus_shutdown`'s teardown
sweep (`:169`/`:174`), `ICARUS_LinkEntity`/`I_LinkEntity` (`:681`),
`Q3_GetEntityByName`/`I_GetEntityByName` (`Q3_Interface.cpp:238`), `Q3_DebugPrint`'s
`WL_DEBUG` line (`:679`), and `CTaskManager::Update`'s `SVF_ICARUS_FREEZE` gate
(`TaskManager.cpp:324`) — all through the one service (`SV_GentityNum`,
`sv_game.cpp:54-59`, over the `G_LOCATE_GAME_DATA` base `:329-330`, ruling 23/37).
Among the entity-field fns, `icarus_associate_ent` still needs **no** `gentity` — it
only reads `ent.s.number` off the copy into the ent-list (`:317`).
`Z_Malloc`/`Z_Free` (`Memory.cpp`) are subsumed by owned Rust objects — no arena
(ICARUS-D3, ruling 20; `ICARUS_Malloc`/`ICARUS_Free` §20-dropped) — and are not a
trap surface.

**Layout types (unchanged, already ported).** `T_G_ICARUS_*` structs +
`game_export_t` + `taskID_t` (the `q3_task_id_*` enum arg,
`oracle/codemp/game/g_public.h:625-639`, ported at
`crates/mp/qshared/src/common/mp/qcommon/task_id_t.rs`) + `sharedEntity_t` (the
entity-field seam arg `*mut sharedEntity_t` — pointing to `ConvertedEntity`'s copy
for the five ent-fns, to the real entity for the task-id helpers — and the
`gentity` service's return target, ruling 37,
`crates/mp/qshared/src/common/mp/qcommon/shared_entity_t.rs`) — all in
`mp_qshared`; `setType_e`, `playType_e`, `pscript_t`, `vector_t` skeletons already
in `crates/mp/engine/icarus/src/`. The existing `#[repr(C)] interface_export_s`
skeleton (`crates/mp/engine/icarus/src/interface/interface_export_s.rs`) is a
type-port artifact **superseded** by the plain `fn`-item `InterfaceExport`
(ICARUS-D3, ruling 24; ICARUS internal types are not ABI-crossing, so layout is
free — §F).

## Decisions

Numbered records. This pass-7 revision consolidates the settled work into three
decisions: ICARUS-D1 (ruling 40), ICARUS-D2 (ruling 43 + STATE-Q2 closure), and
ICARUS-D3 (all rulings 11–39 stand).

**ICARUS-D1 (ruling 40, 2026-07-09 — closes ICARUS-Q13 + settles the §F naming
rule; supersedes ruling 39a's drop-list).** Two parts.

*(1) BlockStream `Init` is LIVE; the §20 drop-list is exactly FIVE methods.* The
live reader `CBlockStream::Open` (`blockstream.h:177`, def `BlockStream.cpp:665`,
called `GameInterface.cpp:465`, `Sequencer.cpp:318`/`:370`) calls `Init()` at
`BlockStream.cpp:670`, so `Init` (`blockstream.h:165`, def `:558`) has a live caller
and fails §20's zero-caller test — it ports (a factual set-membership correction to
ruling 39a, which had mis-listed it, **closing ICARUS-Q13**). The genuinely-dead
writer/duplicator half is exactly **five** methods, each independently
verified zero-caller: `CBlockStream::Create(char *)` (def `:525`),
`CBlockStream::WriteBlock` (def `:577`), `CBlockMember::WriteMember` (def `:133`),
`CBlock::Duplicate` (def `:359`), `CBlockMember::Duplicate` (def `:148`). **The
`CBlock::Write` overloads (`:125-129`, def `:244-300`) and
`CBlockMember::SetData`/`WriteData`/`WriteDataPointer` (`:56-58`/`:76`/`:88`) are
LIVE and PORT** — they are in-memory member builders on the parse path
(`Sequence.cpp:503-538`, `TaskManager.cpp:1863-1887`, `Sequencer.cpp:396…714`;
`CBlock::Write` calls `WriteData`/`SetData` at `:278`/`:291`/`:250`/`:265`), *not*
the file writer; this corrects pass-6's over-broad drop that grouped them with the
dead half. Scope, roster, and Divergences render exactly these five drops. Because
§20 forbids porting genuinely dead surface into a dedicated build that never writes
`.IBI`, and requires porting everything with a live caller. Rejected retaining the
file writer for a re-write round-trip (a ReadBlock record-stream dump verifies parse
parity — § Verification), and rejected dropping `Init`/`CBlock::Write` (the oracle
shows live callers).

*(2) §F internal-type naming rule (project-wide).* Every §F internal-only type
(porting-rules §12 — none crosses the ABI seam, layout free) takes an idiomatic
UpperCamelCase Rust name with Raven's Hungarian affixes stripped: the bare `C`
class-prefix **drops** (`CSequence`→`Sequence`, `CSequencer`→`Sequencer`,
`CTaskManager`→`TaskManager`, `CTask`→`Task`, `CTaskGroup`→`TaskGroup`,
`CBlock`→`Block`, `CBlockMember`→`BlockMember`, `CBlockStream`→`BlockStream`), as
do the `_t`/`_e` suffixes and the `ICARUS_`/`pscript_` prefixes
(`ICARUS_Instance`→`IcarusInstance`, `pscript_t`→`Pscript`). Applied **consistently**
across every table, prose usage, and roster summary. Two carve-outs: **subsystem
acronym prefixes stay Pascal** (the `CmLandScape`, `RmArea` precedents — a
multi-letter subsystem tag is kept and Pascal-cased, not dropped), and
**ABI-frozen types keep their exact Raven names** (§ Seam layout types —
`sharedEntity_t`, `taskID_t`, `T_G_ICARUS_*`). Two things keep the Raven spelling
and are **not** the Rust type name: the Files-roster `class:` column (it records
the Raven source class for `port-cpp-subsystem`) and the `.rs` filename
(Raven-derived snake_case — `cblock.rs` holds `pub struct Block`); prose that cites
a Raven class or method (`CTaskManager::Update`, "the class `CBlockStream`")
likewise keeps the `C` because it names Raven, not the Rust type. Rejected keeping
the `C` prefix on the Rust type (non-idiomatic, and inconsistent with the
`CmLandScape`/`RmArea` sibling precedents).

**ICARUS-D2 (ruling 43, 2026-07-09 + STATE-Q2 closure — the concrete
split-borrow).** `Engine` implements `EngineHost` through a **concrete** view
struct, not the vague "split-borrow view" pass-6 gestured at: `pub struct
EngineHostView<'a>` in `mp_engine_core` holds `&mut` borrows of the
`Common`/`Server`/`CollisionWorld`/loader fields `EngineHost`'s 15 methods need,
and `Engine` gains **per-subsystem split constructors** — for ICARUS,
`fn icarus_call(&mut self) -> (EngineHostView<'_>, &mut Icarus)` (the sibling
pattern is `nav_call`/`g2_call`/… ) — that split-borrow the subsystem field off
the shared engine fields by plain field-level reborrowing, **no unsafe**. The
`EngineHost` trait impl lands **at wave 20** with the `SV_GameSystemCalls` arms
that consume it. STATE-Q2 is **CLOSED** (`state-ownership.md:1860-1876`, rulings
12/13/43) and `crates/mp/engine/core/src/engine.rs:35-38` is **reconciled** with it
(commit `163952dc`): the `engine.rs` comment now records STATE-Q2 CLOSED and the
`EngineHostView` constructors, so this doc cites both as authoritative — the
pass-6 "engine.rs lines 34-36 are a stale pre-ruling-12 placeholder" language is
struck everywhere. Because a single owned `Engine` cannot hand a whole-`&mut self`
`EngineHost` to a subsystem fn that also holds `&mut Icarus` (aliasing); the
per-field view is the borrow-checker-clean dual of Raven's ambient-global access.
Rejected an `unsafe` whole-`&mut self` reborrow, and rejected threading the
subsystem state through the view (the subsystem field stays split off, reached as
`&mut Icarus`).

**ICARUS-D3 (all rulings 11–39 stand).** The prior settled decisions are
unchanged; consolidated here by ruling number so the body can cite them:
- **ruling 11** — one `EngineHost` services trait; every §F fn takes `(&mut Icarus,
  &mut dyn EngineHost)`; `Engine` implements it via the ICARUS-D2 split-borrow view.
- **ruling 12** — `Icarus` is a plain, `Default`-initialized `icarus` field
  directly on `mp_engine_core::Engine`
  (`docs/handoffs/engine-fork-discovery.md:126-130`); no `Option`/`Box` wrapper.
  The field lands with this subsystem's port waves (STATE-Q2 CLOSED); lazy-init is
  modeled by Raven's own NULL-flags.
- **ruling 13 + 20** — owned Rust objects replace all `TAG_ICARUS2/3/4` class
  allocs and the `TAG_ICARUS5` blobs (`BlockMember::m_data`, `Pscript::buffer`
  → owned `Vec<u8>`; `bData` scratch → function-local `Vec<u8>`); the ICARUS **arena
  is dropped entirely** and `ICARUS_Malloc`/`ICARUS_Free` are §20-dropped.
- **ruling 14** — golden `.IBI` fixtures are hand-authored scripts compiled once by
  a `tools/ibi-gen` harness (built from the out-of-set `Interpreter.cpp`/
  `Tokenizer.cpp`); compiled blobs committed, no retail `.IBI` assets committed.
- **ruling 15** — the unchecked `gSequencers`/`gTaskManagers[ent_num]` paths
  **guard-and-return** per §19 (adopt the `ICARUS_FreeEnt` bounds-check idiom,
  `s.number < 0 || >= MAX_GENTITIES` → absent), ≤2-line note per site.
- **ruling 17** — `Svcmd_ICARUS_f` (`GameInterface.cpp:700-730`) is §20-dropped
  (commented-out body, zero callers/registrations, no `G_ICARUS_*` arm).
- **ruling 19** — superseded on the entity-field seam by rulings 23 + 37.
- **ruling 23** — the entity-field arms carry a `*mut sharedEntity_t` pointer (not
  an entnum); `gentity` survives as the index→pointer service. (Its "shuffle no-op"
  rationale is corrected by ruling 37; its scope claim is corrected — `gentity` is
  reached from six bodies, not `icarus_valid_ent` alone — see the § Raven ground
  truth gentity-site inventory.)
- **ruling 24** — the ~194 internal `I_*` dispatch sites (73× `m_ie->I_*` in
  `CSequencer`, 121× `(m_owner->GetInterface())->I_*` in `CTaskManager`) become
  **free fns `(&mut Icarus, &mut dyn EngineHost, …)` with no stored back-refs**
  (re-index disjoint field borrows per call); the `InterfaceExport` slots and every
  pub seam fn take `&mut dyn EngineHost` (a bare `fn` cannot be generic over `impl`);
  the Stage-0 crate is pinned as `crates/mp/host-interface` / `mp_host_interface`.
- **ruling 27** — the two ICARUS ownership graphs are **faithful `Vec` arenas + id
  newtypes**: `IcarusInstance` owns `sequences: Vec<Sequence>` + `sequencers:
  Vec<Sequencer>` keyed by `SequenceId`/`SequencerId` carrying the monotonic
  never-reused `m_GUID` (`Instance.cpp:26,228`); `GetSequence(id)` stays a faithful
  linear scan (`Instance.cpp:248-258`, insertion-ordered, not a keyed map — an §A2
  change the doc declines); `Sequence`'s `m_parent`/`m_return`/`m_children` →
  `Option<SequenceId>`/`Option<SequenceId>`/`Vec<SequenceId>`; `Sequencer`'s
  non-owning `m_sequences` → `Vec<SequenceId>` and `m_taskSequences` →
  `BTreeMap<TaskGroupId, SequenceId>`; `TaskManager` has **one** owning
  `m_taskGroups: Vec<TaskGroup>` + `TaskGroupId` with `m_taskGroupNameMap`/
  `m_taskGroupIDMap` → `BTreeMap<String, TaskGroupId>`/`BTreeMap<i32, TaskGroupId>`
  side-indexes; `m_tasks: Vec<Task>` and `m_completedTasks: BTreeMap<i32, bool>`
  transcribe literally (groups track completion by int GUID, no pointer aliasing).
- **rulings 31 + 33** — the Stage-0 `EngineHost` crate was built as real compiled
  code before this relaunch (ruling 36 later extended it to 15 methods);
  § Seam definition quotes its actual signatures verbatim, no paper spec.
- **ruling 32** — the fixture-backed `MockHost` (`crates/mp/host-interface/src/mock.rs`)
  is the goldens vehicle; it also carries the `cvars` map fixture for `cvar_integer`
  (ruling 36).
- **ruling 36** — the `EngineHost` trait is extended to **15** methods (commit
  `a9820853`), adding `cvar_integer`/`sv_time`/`fs_write_file`/`model_mdxm`/
  `model_mdxa`. ICARUS binds `cvar_integer` to port `Q3_DebugPrint`'s `com_developer`
  gate — `if (!com_developer || !com_developer->integer) return;`
  (`Q3_Interface.cpp:638-643`) → `host.cvar_integer("developer") != 0` (unregistered
  name → 0, `Cvar_VariableIntegerValue`, `cvar.cpp:118-124`). § Seam re-quotes the
  full 15-method trait verbatim. The other four new methods serve sibling §F
  subsystems; ICARUS binds none.
- **ruling 37** — `ConvertedEntity` (`sv_game.cpp:420-451`) ports **faithfully**
  (the server crate, wave-20): it copies `ent->s`/`r`/`taskID` **by value** into the
  file-static `gLocalModifier` and returns `&gLocalModifier` (the copy), so retail
  **drops** writes to it — including `ICARUS_InitEnt`'s `memset(&ent->taskID,-1)`
  (`GameInterface.cpp:664`); reproducing the copy reproduces the drop (byte-faithful
  at M4/M5). The five entity-field ent-fns keep `ent: *mut sharedEntity_t` (the trap
  payload, now understood to point at the copy); the three task-id helpers use the
  **bare** `(sharedEntity_t *)VMA(1)` cast (the real entity, so their writes persist);
  `ICARUS_ValidEnt` re-fetches the true entity via `gentity` precisely because the
  copy's write would be dropped; `ICARUS_AssociateEnt` needs no `gentity` (reads
  `ent->s.number` off the copy, `:317`). This **strikes** ruling 23's "shuffle is a
  no-op" rationale.
- **ruling 39a (genuine drops)** — the BlockStream writer/duplicator half is
  §20-dropped. Its *genuine* zero-caller drops stand (the five in ICARUS-D1); its
  Init mis-listing and the `CBlock::Write` over-drop are corrected by ICARUS-D1
  (ruling 40).
- **ruling 39d** — the id newtypes are **declared beside their owning arena**
  (§B5 / RMG `AreaId` precedent): `pub struct SequenceId(i32)` and `pub struct
  SequencerId(i32)` in `instance/icarus_instance.rs`, beside the `IcarusInstance`
  `sequences`/`sequencers` arenas; `pub struct TaskGroupId(i32)` in
  `taskmanager/ctask_manager.rs`, beside its `m_taskGroups: Vec<TaskGroup>` arena.
  No newtype is a standalone module.
- **standing §F conventions** — closed C++ hierarchies → Rust structs/enums with
  owned `Vec`/`VecDeque`/`HashMap`/`String` (entities by index, layout free);
  `interface_export_t` → plain `InterfaceExport` struct of `fn` items at
  `Interface_Init` (fork-5); MP-only, SP a later diff (DEC-04/§20);
  transcription-first (§A2). `Icarus` lives in `lib.rs` (crate root) with the
  per-class module dirs (`instance/`, `taskmanager/`, `memory/`, `q3_registers/`,
  `sequence/` new; `blockstream`/`game_interface`/`interface`/`q3_interface`/
  `sequencer` existing).

## Files roster

C++-track roster for `port-cpp-subsystem` (`designPath` consumer). All
`crate: mp_engine_icarus`, `mode: mp`. Existing type-port files are reshaped in
place under ICARUS-D3 (rulings 13/24/27); new files mirror the owning Raven header
subsystem (one class per file, porting-rules §21). The `class:` column keeps the
Raven class name; the Rust type in each summary drops the `C` per ICARUS-D1.

files:
- path: `crates/mp/engine/icarus/src/lib.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `Icarus`, summary: crate root — defines the fork-2 subsystem aggregate `pub struct Icarus` (fields per the State-ownership table: `instance`, `sequencers`, `task_managers`, `buffer_list`, `ent_list`, `ent_filter`, `interface_export`, `var_strings/floats/vectors`, `num_variables` — **no `arena` field**, dropped per ICARUS-D3/ruling 20) with a **hand-written `impl Default`, not `#[derive(Default)]`** (see the note under State ownership): the `Box<[Option<SequencerId>; MAX_GENTITIES]>` (non-owning index into `IcarusInstance.sequencers`) / `Box<[Option<TaskManager>; MAX_GENTITIES]>` slot arrays are built explicitly (e.g. `Box::new(std::array::from_fn(|_| None))`); `ent_filter` seeds `-1`, not derive's `0` (`ICARUS_entFilter = -1`, `GameInterface.cpp:23`); and `interface_export` seeds the real `Q3_*`/`I_*` fns (see `interface/interface_export_s.rs`). Adds the module declarations the roster requires — new `pub mod sequence; pub mod taskmanager; pub mod instance; pub mod memory; pub mod q3_registers;` alongside existing `blockstream`/`game_interface`/`interface`/`q3_interface`/`sequencer` (and untouched `interpreter`/`tokenizer` skeletons per Scope). `Icarus` is not a Raven class — it is the synthesized owner of every ICARUS file-scope global; it attaches to `mp_engine_core::Engine` as a plain `icarus` field per ICARUS-D3 (rulings 11/12), reached through the ICARUS-D2 `EngineHostView`/`icarus_call` split-borrow.
- path: `crates/mp/engine/icarus/src/blockstream/cblock_member.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CBlockMember`, summary: `pub struct BlockMember` — one ID/size/data record; owned `Vec<u8>` data replacing `void* m_data` (ICARUS-D3/ruling 20 drops the arena, so this `TAG_ICARUS5` blob is owned here). `ReadMember` (reader, live) and the **in-memory builders `SetData` (`:56-58`), `WriteData` (`:76`), `WriteDataPointer` (`:88`) all PORT — they are LIVE** (called by the live `CBlock::Write` overloads and `TaskManager.cpp:1111`/`:1132`, ICARUS-D1). Only `WriteMember` (`:47`, def `BlockStream.cpp:133`) and `Duplicate` (`:74`, def `:148`) are **§20-dropped per ICARUS-D1 (ruling 40)** — dead writer/duplicator half (`WriteMember`'s only caller is the dropped `WriteBlock`; `Duplicate`'s only caller is the dead `CBlock::Duplicate`) — recorded with a `blockstream/` module-doc zero-caller note (`blockstream.h:38-105`, `BlockStream.cpp`).
- path: `crates/mp/engine/icarus/src/blockstream/cblock.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CBlock`, summary: `pub struct Block` — owns `Vec<BlockMember>`; block id + flags; `Create(int)` (live — reader/`ReadBlock` path, `BlockStream.cpp:201`), `Init`/`Free`/`AddMember`/`GetMember` (live, `blockstream.h:109-154`), **and the `Write` overloads (`:125-129`, def `BlockStream.cpp:244-300`) all PORT — they are LIVE in-memory member builders** on the parse path (`Sequence.cpp:503-538`, `TaskManager.cpp:1863-1887`, `Sequencer.cpp:396…714`), *not* file I/O (ICARUS-D1, ruling 40). Only `Duplicate` (`:138`, def `:359`, zero callers) is **§20-dropped per ICARUS-D1** with a module-doc zero-caller note.
- path: `crates/mp/engine/icarus/src/blockstream/cblock_stream.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CBlockStream`, summary: `pub struct BlockStream` — `.IBI` **reader** over an owned byte buffer; `Open` (`:665`, live — `GameInterface.cpp:465`, `Sequencer.cpp:318`/`:370`), `ReadBlock` (`:619`), `BlockAvailable` (`:605`), the `IBI` header+version check, the `Get*` stream primitives (`blockstream.h:158-196`), and **`Init` (`:165`, def `:558`) all PORT — `Init` is LIVE (`Open` calls it at `:670`), not in ruling 40's drop set (ICARUS-D1, closing ICARUS-Q13)**. Only the **file writer methods `Create(char *)` (`:167`, def `:525`, zero callers) and `WriteBlock` (`:174`, def `:577`, excluded-TU callers) are §20-dropped per ICARUS-D1 (ruling 40)** — with a module-doc zero-caller note.
- path: `crates/mp/engine/icarus/src/blockstream/vector_t.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `vector_t`, summary: `[f32; 3]` alias used by block reads (existing skeleton, kept).
- path: `crates/mp/engine/icarus/src/blockstream/file.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CBlockStream::file`, summary: owned file-handle helper replacing `FILE*` — reader-only under ICARUS-D1 (the writer `Create(char*)` path drops); kept for the reader's owned-buffer I/O (existing skeleton).
- path: `crates/mp/engine/icarus/src/sequence/csequence.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CSequence`, summary: `pub struct Sequence` — command/child tree node; flags/iterations, `Save/Load` (inert) (`sequence.h:12-96`, `Sequence.cpp`). **Shape settled by ICARUS-D3 (ruling 27):** `m_children: Vec<SequenceId>`, `m_parent: Option<SequenceId>`, `m_return: Option<SequenceId>` (pointer graph → id newtypes into the `IcarusInstance.sequences` arena, reconstructed on `Load` via a `GetSequence(id)` linear scan); `m_commands: Vec<Block>` owned (no cross-object aliasing, literal transcription).
- path: `crates/mp/engine/icarus/src/sequencer/csequencer.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CSequencer`, summary: `pub struct Sequencer` — per-entity script driver; parses IBI blocks into sequences; `Run`/`Callback`/`Parse*`/`Check*`/`Push/PopCommand` (`sequencer.h:68-187`, `Sequencer.cpp`, ~43 fns). Per ICARUS-D3 (ruling 24) it holds **no** `m_ie`/`m_owner`/taskmanager handles — the 73 `m_ie->I_*` dispatch sites become free fns `(&mut Icarus, &mut dyn EngineHost, …)` that re-index disjoint `Icarus` field borrows per call. **Member-storage shape settled by ICARUS-D3 (ruling 27):** the own non-owning `m_sequences` subset (`sequencer.h:176`) is `Vec<SequenceId>`; its `map<CTaskGroup*,CSequence*> m_taskSequences` (`:177`) is `BTreeMap<TaskGroupId, SequenceId>`.
- path: `crates/mp/engine/icarus/src/sequencer/bstream_s.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `bstream_t`, summary: internal stream-stack node; owned `stream` + Raven's intrusive `last` pointer folded to an `Option<usize>` **index** into the sequencer's owned `Vec<Bstream>` (`None` == top-level `last == NULL`; `sequencer.h:42-46`).
- path: `crates/mp/engine/icarus/src/taskmanager/ctask.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CTask`, summary: `pub struct Task` — a scheduled task (GUID, timestamp, owned `Block`) (`taskmanager.h:33-58`). **Ownership shape settled by ICARUS-D3 (ruling 27):** owned in `TaskManager`'s `m_tasks: Vec<Task>` (Raven's `list<CTask*> m_tasks`, `taskmanager.h:178`); no cross-object pointer aliasing (groups track completion by int GUID), so literal transcription, no id newtype needed.
- path: `crates/mp/engine/icarus/src/taskmanager/ctask_group.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CTaskGroup`, summary: `pub struct TaskGroup` — completion-tracking group; `MarkTaskComplete`/`Complete` (`taskmanager.h:62-93`). **Shape settled by ICARUS-D3 (ruling 27):** owned in `TaskManager`'s `Vec<TaskGroup>` arena keyed by `TaskGroupId`; `m_completedTasks` (Raven's already-int-keyed `map<int,bool>`, `:87`) → `BTreeMap<i32, bool>`; `m_parent: Option<TaskGroupId>` (raw `CTaskGroup*` back-pointer → id).
- path: `crates/mp/engine/icarus/src/taskmanager/ctask_manager.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `CTaskManager`, summary: `pub struct TaskManager` — per-entity scheduler; `Update` heartbeat (`Go`, `RUNAWAY_LIMIT`) — which **opens with a per-frame `host.gentity(m_ownerID)` freeze gate**: `SV_GentityNum(m_ownerID)` then `owner->r.svFlags & SVF_ICARUS_FREEZE` → return `TASK_FAILED` before `Go()` (`TaskManager.cpp:322-329`, `m_ownerID: i32` at `taskmanager.h:173`; § Raven ground truth gentity-site inventory) — the `Rotate/Camera/Print/Sound/Move/Set/Use/…/Wait/WaitSignal` handlers, owned task maps/lists; `Get` scratch out-param folds to an owned return (`taskmanager.h:97-189`, `TaskManager.cpp`, ~52 fns). Per ICARUS-D3 (ruling 24) it holds **no** `m_owner` back-ref — the 121 `(m_owner->GetInterface())->I_*` dispatch sites become free fns `(&mut Icarus, &mut dyn EngineHost, …)` re-indexing disjoint `Icarus` field borrows per call. **Member-storage shape settled by ICARUS-D3 (ruling 27):** the three parallel `CTaskGroup*` indexes collapse to **one owner** `m_taskGroups: Vec<TaskGroup>` (`:177`) + `TaskGroupId` — **this file also declares `pub struct TaskGroupId(i32)`**, co-located with the `Vec<TaskGroup>` arena it indexes per **ICARUS-D3 (ruling 39d)** (RMG `AreaId` §B5 precedent) — with `m_taskGroupNameMap`/`m_taskGroupIDMap` (`:183-184`) as `BTreeMap<String, TaskGroupId>`/`BTreeMap<i32, TaskGroupId>` side-indexes; `m_tasks: Vec<Task>` owns the tasks (literal).
- path: `crates/mp/engine/icarus/src/instance/icarus_instance.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `ICARUS_Instance`, summary: `pub struct IcarusInstance` — top singleton; **owns** its sequence/sequencer arenas + signal map; `Create`/`Delete`, `Signal`/`CheckSignal`/`ClearSignal`, inert `Save*/Load*` (`instance.h:12-79`, `Instance.cpp`, ~24 fns). **Pool container + lookup shape settled by ICARUS-D3 (ruling 27):** `sequences: Vec<Sequence>` + `sequencers: Vec<Sequencer>` owning arenas (`instance.h:62-63`); **this file also declares `pub struct SequenceId(i32)` and `pub struct SequencerId(i32)`** — the newtypes carrying the monotonic never-reused `m_GUID` (`Instance.cpp:26,228`), co-located with the arenas they index per **ICARUS-D3 (ruling 39d)** (RMG `AreaId` §B5 precedent); `GetSequence(id)` stays a **faithful linear scan** (`Instance.cpp:248-258`), insertion-ordered, **not** a keyed map (§A2 change declined); `m_signals: BTreeMap<String, u8>`. The per-entity `gSequencers`/`gTaskManagers` arrays are file-scope globals (`Instance.cpp:19-20`), not `ICARUS_Instance` members — `Icarus.sequencers` is a non-owning `SequencerId` index into this arena, `Icarus.task_managers` owns the task managers (State-ownership table).
- path: `crates/mp/engine/icarus/src/interface/interface_export_s.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `interface_export_t`, summary: reshape the `#[repr(C)]` type-port skeleton into the plain `fn`-item `InterfaceExport` table (ICARUS-D3, rulings 24/fork-5) (`interface.h:17-70`). The fields are **bare `fn` items (not `Option<fn>`)**, so the struct has no null/None state, yet `Icarus` (hence this field) must be `Default`-constructible — so its `impl Default` seeds every slot with the real crate `Q3_*`/`I_*` fn, identical to `Interface_Init`'s 1:1 assignment (`Q3_Interface.cpp:956-1008`), which (re-)populates the table before any `I_*` call (`GameInterface.cpp:143-156`), so the seed is overwritten with the same fns before it is ever observed. Per ICARUS-D3 (ruling 24) each stored slot is a `fn(&mut Icarus, &mut dyn EngineHost, …)` pointer (`&mut dyn`, not `impl`).
- path: `crates/mp/engine/icarus/src/memory/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `(none — §20 note)`, summary: **no `IcarusArena` type** — ICARUS-D3 (ruling 20) drops the arena entirely (all three `TAG_ICARUS5` families are owned buffers: `BlockMember::m_data`, `Pscript::buffer` owned `Vec<u8>`, and `bData` `Save`/`Load` scratch folded to a local). `ICARUS_Malloc`/`ICARUS_Free` (`Memory.cpp:8-20`, `icarus.h:29-30`) have zero live callers under the owned-buffer shape and are **§20-dropped, not ported**; this file carries only the module-doc zero-caller note recording that (porting-rules §20).
- path: `crates/mp/engine/icarus/src/q3_interface/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `Q3Interface`, summary: the `Q3_*` `I_*` implementations (shared-memory write + `VM_Call`, both via `EngineHost`) and `Interface_Init` wiring; `Q3_Evaluate`, `Q3_DebugPrint` (its `com_developer` gate ports through `host.cvar_integer("developer")` per **ICARUS-D3 (ruling 36)**; its `WL_DEBUG` branch then reaches `host.gentity(entNum)` for the log line, `Q3_Interface.cpp:679`), `Q3_GetEntityByName` (the outbound `I_GetEntityByName` slot, `:238`, wired `:964`) which resolves a script name to a real entity via `host.gentity` — both per the § Raven ground truth gentity-site inventory — camera stubs, and the `Q3_TaskIDSet`/`Complete`/`Pending` task-id helpers (which read/write `ent->taskID[]`/`s.number` on the **bare real-entity** `ent: *mut sharedEntity_t` the seam carries — writes persist, unlike the ConvertedEntity-copy arms — per ICARUS-D3/ruling 37; no `host.gentity`) (`Q3_Interface.cpp`, ~49 fns). The `I_*` slot fns' host-param type is `&mut dyn EngineHost` and internal dispatch is via free fns re-indexing `&mut Icarus` (ICARUS-D3, ruling 24).
- path: `crates/mp/engine/icarus/src/q3_interface/set_type_t.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `setType_e`, summary: SET_* enum (existing skeleton, kept).
- path: `crates/mp/engine/icarus/src/q3_interface/play_type_t.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `playType_e`, summary: PLAY_* enum (existing skeleton, kept).
- path: `crates/mp/engine/icarus/src/q3_registers/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `Q3Registers`, summary: `varStrings`/`varFloats`/`varVectors` stores + `Q3_InitVariables`/`Q3_Declare/Free/Get/Set*Variable`, `MAX_VARIABLES`, `VTYPE_*`; `Q3_DebugPrint`-gated diagnostics reach `Com_Printf` via `EngineHost::print` and gate on `EngineHost::cvar_integer("developer")` (**ICARUS-D3 (ruling 36)**) (`Q3_Registers.cpp`, `.h:4-32`, ~16 fns).
- path: `crates/mp/engine/icarus/src/game_interface/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `GameInterface`, summary: `ICARUS_RunScript`/`RegisterScript`/`GetScript`/`Init`/`InitEnt`/`FreeEnt`/`ValidEnt`/`AssociateEnt`/`Shutdown`/`LinkEntity`/`SoundPrecache`/`InterrogateScript` + the buffer/ent-list state; the 5 unchecked entnum paths guard-and-return per ICARUS-D3 (ruling 15); `Svcmd_ICARUS_f` §20-dropped per ICARUS-D3 (ruling 17). `RunScript`/`InitEnt`/`FreeEnt`/`ValidEnt`/`AssociateEnt` read `ent->classname`/`taskID`/`script_targetname`/`targetname` on the `ent: *mut sharedEntity_t` the seam carries — which **points to `ConvertedEntity`'s by-value copy** (ICARUS-D3/ruling 37), so `InitEnt`'s `memset(&ent.taskID,-1)` writes the copy and is dropped, faithfully (`GameInterface.cpp:664`). Three fns in **this file** call `host.gentity` (§ Raven ground truth inventory): `ValidEnt` (`:288`/`:291`, behaviorSet write-back `trueEntity->script_targetname = trueEntity->targetname` to the *true* entity), the no-arg `Shutdown` sweep (`:169`/`:174`, feeding the real pointer into `FreeEnt` `:184`), and `LinkEntity` (`:681`, the outbound `I_LinkEntity` slot resolving `entID`). `AssociateEnt` does **not** (it only reads `ent.s.number` off the copy into the ent-list, `:317`) (`GameInterface.cpp`, ~14 live fns).
- path: `crates/mp/engine/icarus/src/game_interface/pscript_s.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `pscript_t`, summary: `pub struct Pscript` — cached script record; `buffer` is an owned `Vec<u8>` replacing `char* buffer` (ICARUS-D3/ruling 20 drops the arena, so this `TAG_ICARUS5` blob is owned here) (existing skeleton, `GameInterface.h:4-8`).

**Module-declaration files (boilerplate, not classes).** Each one-class-per-file
directory (porting-rules §21) needs a `mod.rs` that only `pub mod`s its per-class
files; these carry no logic and make no decision. Listed for roster completeness so
the skeleton compiles (`blockstream/mod.rs`, `sequencer/mod.rs`, `interface/mod.rs`
already exist from the type port; `sequence/mod.rs`, `taskmanager/mod.rs`,
`instance/mod.rs` are new dirs per ICARUS-D3). `game_interface/mod.rs`,
`q3_interface/mod.rs`, `q3_registers/mod.rs`, and `memory/mod.rs` are **not** here —
they are the class-bearing units already rostered above.
- path: `crates/mp/engine/icarus/src/blockstream/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `(mod decl)`, summary: `pub mod cblock_member; pub mod cblock; pub mod cblock_stream; pub mod vector_t; pub mod file;` — declaration-only (existing skeleton, extend for the reshaped roster).
- path: `crates/mp/engine/icarus/src/sequence/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `(mod decl)`, summary: `pub mod csequence;` — declaration-only (new dir, ICARUS-D3).
- path: `crates/mp/engine/icarus/src/sequencer/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `(mod decl)`, summary: `pub mod csequencer; pub mod bstream_s;` — declaration-only (existing skeleton, extend for the reshaped roster).
- path: `crates/mp/engine/icarus/src/taskmanager/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `(mod decl)`, summary: `pub mod ctask; pub mod ctask_group; pub mod ctask_manager;` — declaration-only (new dir, ICARUS-D3).
- path: `crates/mp/engine/icarus/src/instance/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `(mod decl)`, summary: `pub mod icarus_instance;` — declaration-only (new dir, ICARUS-D3).
- path: `crates/mp/engine/icarus/src/interface/mod.rs`, crate: `mp_engine_icarus`, mode: `mp`, class: `(mod decl)`, summary: `pub mod interface_export_s;` — declaration-only (existing skeleton, kept).

`Interface.cpp` produces **no** Rust file — its `Interface_Init` is commented
out (dead surface, see Divergences); the live `Interface_Init` is in
`q3_interface/mod.rs`.

## Divergences

Raven-UB / dead-surface points where the port picks one defined behavior
(porting-rules §19; kept out of / normalized in the shared golden corpus).
`port-cpp-subsystem` `divergences`:

- MP save/load is inert: `AppendToSaveGame`/`ReadFromSaveGame` (`I_WriteSaveData`/`I_ReadSaveData` targets) both `return 1;` with no I/O (`Q3_Interface.cpp:695-704`); the `IcarusInstance`/`Sequencer`/`Sequence`/`TaskManager` `Save`/`Load` methods port as structurally-present but effect-free, and are excluded from the golden corpus.
- `Interface.cpp`'s `Interface_Init` is commented out (`Interface.cpp:14-24`); it links no code. The port emits no Rust unit for it — the live table wiring lives in `q3_interface/mod.rs` (`Q3_Interface.cpp:956`).
- **`ConvertedEntity` is ported FAITHFULLY, NOT diverged (ICARUS-D3, ruling 37).** `ConvertedEntity` (`sv_game.cpp:420-451`) copies `ent->s`/`r`/`taskID` **by value** into the file-static `gLocalModifier` and returns `&gLocalModifier` (the copy); the server crate ports it 1:1 and routes the five entity-field ICARUS arms through it. Writes through the copy are **dropped** exactly as retail — including `ICARUS_InitEnt`'s `memset(&ent->taskID,-1)` (`GameInterface.cpp:664`), which never reaches the real entity. This is **zero divergence vs retail** at M4/M5 (listed here only because it is why the entity-field seam args are `*mut sharedEntity_t` and why `ICARUS_ValidEnt` needs `host.gentity`). The `ICARUS_ValidEnt` behaviorSet write-back (`:288`/`:291`) reaches the *true* entity via `host.gentity(ent.s.number)` precisely because the copy's write would be dropped; `ICARUS_AssociateEnt` reads `ent.s.number` off the copy (`:317`), faithful, no `gentity`. (The three task-id helpers take the bare real pointer, so their writes persist — also faithful.)
- Out-of-range entnum on the 5 unchecked `gSequencers`/`gTaskManagers` paths → **guard-and-return** (resolved per ICARUS-D3, ruling 15). The `ISINITIALIZED`/`MAINTAINTASKMANAGER`/`ISRUNNING` arms index `gSequencers[entID]`/`gTaskManagers[entID]` with `entID = args[1]` unchecked (`sv_game.cpp:752-782`), and `ICARUS_RunScript`/`ICARUS_InitEnt` index `gSequencers[ent->s.number]` unchecked (`GameInterface.cpp:75`, `:660`); only `ICARUS_FreeEnt` guards `s.number >= MAX_GENTITIES || < 0` (`:224-229`). In C a negative/`>= MAX_GENTITIES` entnum is an OOB pointer read (UB); on the Rust `[Option<_>; MAX_GENTITIES]` arrays each of the five ports the `FreeEnt` bounds-check and returns the "absent" result (`false`/no-op), with a ≤2-line §19 note per site. Excluded from / normalized in the shared corpus.
- `ICARUS_Malloc` is non-zeroed (`Z_Malloc(...,qfalse)`, `Memory.cpp:12`) while class `operator new` is zeroed (`Z_Malloc(...,qtrue)`, `blockstream.h:66`); under ICARUS-D3 (rulings 13/20) both class instances and the `TAG_ICARUS5` blobs (`BlockMember::m_data`, `Pscript::buffer`) are owned Rust objects/`Vec<u8>`, always value-initialized, so the zero/non-zero distinction collapses to defined initialization. `WriteDataPointer`'s exact-byte member copy **is preserved** (it ports live per ICARUS-D1) — the in-memory block members carry the same bytes as retail; content parity rides on that plus `ReadMember` reading the committed golden bytes.
- `ICARUS_Malloc`/`ICARUS_Free` (`Memory.cpp:8-20`, `icarus.h:29-30`) are §20-dropped per ICARUS-D3 (ruling 20): with every `TAG_ICARUS5` user owned, the arena is dropped entirely and these two fns have zero live callers. Recorded with a module-doc zero-caller note in `memory/mod.rs`, not ported (`Icarus.arena` does not exist).
- `Svcmd_ICARUS_f` (`GameInterface.h:32`, `GameInterface.cpp:700-730`) is §20-dropped per ICARUS-D3 (ruling 17): commented-out body, zero callers/registrations, no `G_ICARUS_*` arm. Recorded with a module-doc zero-caller note, not ported.
- **BlockStream writer/duplicator half §20-dropped — exactly FIVE methods (ICARUS-D1, ruling 40):** `CBlockStream::Create(char *)` (`blockstream.h:167`, def `BlockStream.cpp:525`, zero callers), `CBlockStream::WriteBlock` (`:174`, def `:577`, callers only in excluded `Interpreter.cpp`), `CBlockMember::WriteMember` (`:47`, def `:133`, only caller the dropped `WriteBlock`), `CBlock::Duplicate` (`:138`, def `:359`, zero callers), and `CBlockMember::Duplicate` (`:74`, def `:148`, only caller the dead `CBlock::Duplicate`). Recorded with a `blockstream/` module-doc zero-caller note, not ported. The dedicated server never writes `.IBI`. **NOT dropped (they port, ICARUS-D1):** `CBlockStream::Init` (`:165`, def `:558`, live — `Open` calls it at `:670`, closing ICARUS-Q13), the `CBlock::Write` overloads (`:125-129`, def `:244-300`, live in-memory builders), and `CBlockMember::SetData`/`WriteData`/`WriteDataPointer` (`:56-58`/`:76`/`:88`, live) — ruling 39a's original list over-dropped these; ruling 40's oracle re-check confirms their live callers (this is a §20 set-membership correction, not a §19 divergence — they are fully faithful).
- Out-of-range `task_type` on the three `G_ICARUS_TASKID*` arms → **guard-and-return** is Raven's own defined behavior, not an undecided fork. The arms cast `(taskID_t)args[2]` unchecked (`sv_game.cpp:786`, `:808`, `:813`), but each callee guards `taskType < TID_CHAN_VOICE || taskType >= NUM_TIDS` and returns before indexing `ent->taskID[taskType]` (`Q3_TaskIDPending` `Q3_Interface.cpp:116-118`, `Q3_TaskIDComplete` `:136-138`, `Q3_TaskIDSet` `:169-171`), so the port transcribes that guard. The only genuine §19 point is the int→enum conversion (Rust cannot build an invalid `taskID_t`); that checked conversion lives at the server-dispatch boundary that owns the arm (Punts, § Seam definition task-id note). This crate's callees receive an in-range `taskID_t`.
- **Sequencer/TaskManager driver — internal signature refinements (ruling 24's `(&mut Icarus, &mut dyn EngineHost, …)` open-ended free-fn spec, resolved during transcription).** The seam entries `run`/`update` keep their pinned shapes; the *internal* fns they drive are threaded, not reached: a drive **detaches** the owner's `Sequencer` (from `IcarusInstance.sequencers`, via `take_sequencer`/`restore_sequencer`) and its `TaskManager` (`Option::take`) into locals, so a task handler can hold `&mut Icarus` for `I_*` dispatch while the scheduler pair is a local (the only way to satisfy Rust's disjoint-borrow rule for the `Go → CallbackCommand → Callback → Go` recursion). Consequently `CSequencer::Callback`/`Recall` are internal fns taking `(&mut Sequencer, &mut TaskManager, …)` — and `Callback` takes its block **by value** (`block: Block`), not `&Block`, so both Raven branches (`PushCommand(block,…)` transfer / `delete block` drop) are expressible. `bstream_t`'s `last` folds to an `Option<usize>` **index field** on the node (not eliminable from Vec order once `DeleteStream` removes an interior node). Named-completion `wait("group")` resolves against the running manager's own group state (threaded in). The developer-gated `WL_DEBUG` per-command trace lines are dropped (diagnostic-only, unobservable at `developer == 0`). `CSequence::SetParent`'s `SQ_RETAIN`/`SQ_PENDING` inheritance and `HasChild`'s recursive descendant walk are done by arena-aware sequencer helpers (`seq_set_parent_inherit`/`seq_has_child`) since the arena-blind `Sequence` node holds only ids. Cross-entity `affect()` (a different `gSequencers[]` target) is a golden-unreachable limitation of the single-detached-sequencer model (self-affect is faithful).
- **`Q3_InitVariables` gains `&mut dyn EngineHost`** (`Q3_Registers.cpp:192-202`): the frozen `(&mut Icarus)`-only shape could not emit Raven's `"%d residual variables found!\n"` warning, which is `Q3_DebugPrint`/`developer`-gated (ruling 36, roster row for `q3_registers`). The state reset is unchanged; only the (developer-gated, golden-unobservable) diagnostic is restored. The lone caller (`tests/icarus_parity.rs`) is updated in lockstep.

Scout/harness phases may surface further §19 sites; each gets a ≤2-line
site note at the port, per porting-rules §19.

## Verification strategy

DEC-09 layer 1, porting-rules §F/§18. `tools/icarus-oracle/` compiles the
unmodified oracle TUs standalone under stub headers (GP2/`tools/gp2-oracle`
pattern) and golden-diffs canonical dumps against the Rust port; goldens are
committed so `cargo test` needs no C++ toolchain. Gate: GOAL-engine M2
("icarus differential goldens"). Skeleton build/checkpoint semantics per DEC-10.

Standalone-diffable units and their canonical dumps:
- **BlockStream (read-only, ICARUS-D1/ruling 40)** — parse a corpus of `.IBI`
  blobs and dump the parsed `(id, member-type, size, bytes)` record stream via
  `Open`→`BlockAvailable`→`ReadBlock`→`ReadMember`. No re-`WriteBlock` round-trip
  — the file writer is §20-dropped, so parse parity is verified by the ReadBlock
  record-stream dump against the oracle's own parse (the live `CBlock::Write`/
  `WriteData` in-memory builders are exercised transitively by the parse, so their
  member bytes appear in the dump). Cleanest TU (its only allocation is the
  `BlockMember::m_data` owned `Vec<u8>` that replaces the `TAG_ICARUS5` blob —
  ICARUS-D3/ruling 20; no arena/`Z_Malloc` dep).
- **Q3_Registers** — script variable operations: dump `varStrings/varFloats/
  varVectors` state after a scripted `Declare/Set/Get/Free` sequence.
- **Sequencer + TaskManager + Instance (end-to-end)** — drive
  `ICARUS_RunScript` on a committed `.IBI` fixture through ruling 32's
  fixture-backed `MockHost` (`crates/mp/host-interface/src/mock.rs`): `.IBI`
  bytes served from `MockHost.files`, deterministic `flrand`/`irand` off the
  replicated `holdrand` LCG, `MockHost.cvars` answering `cvar_integer("developer")`
  (ruling 36), and `MockHost.vm_calls` recording the ordered
  `vm_call(VmSlot::Gvm, GAME_ICARUS_*, …)` trace. The end-to-end run exercises
  `gentity` beyond `icarus_valid_ent` (§ Raven ground truth inventory), so the
  `MockHost.gentity` arena must supply the owner entity with the right
  `r.svFlags` for `CTaskManager::Update`'s `SVF_ICARUS_FREEZE` gate and any
  `script_targetname`-named entities the fixture's `I_GetEntityByName` /
  `I_LinkEntity` calls resolve — provision the strided arena accordingly. No
  test-only ICARUS constructor is added — `ICARUS_RunScript` ports with its real
  frozen signature and reaches the world through the mock's front door (ruling 32);
  the mock stands in for the ICARUS-D2 `EngineHostView`. The golden is that ordered
  `vm_call`/`I_*` callback stream plus final variable/signal state, exercising
  `Parse*`/`Check*`, task scheduling, `Wait`/`WaitSignal`, and `Callback`.

Fixture provenance (ICARUS-D3, ruling 14): the committed `.IBI` goldens are
hand-authored scripts compiled **once** by a `tools/ibi-gen` harness built from
the oracle's out-of-set `Interpreter.cpp`/`Tokenizer.cpp`; the compiled blobs
are committed, **no** retail `.IBI` assets are committed, and a retail corpus
may run locally, uncommitted.

Live-peer acceptance (DEC-09 layer 2) is the server-crate/full-engine gate
(M4-M5), not this subsystem's unit gate — where ruling 37's faithful
`ConvertedEntity` routing and the ICARUS-D2 `EngineHostView` split-borrow are
proven zero-divergence vs retail against the live oracle dylib.

## Slice hooks

- **M2 (waves 7-12), GOAL-engine** — "icarus complete … icarus differential
  goldens." The § Seam definition pub API is frozen on the `EngineHost` shape
  (ICARUS-D3, `&mut dyn EngineHost`; ruling 36's 15-method trait) and the
  `tools/icarus-oracle` harness / `tools/ibi-gen` fixture corpus (ICARUS-D3, ruling
  14) are the remaining build items. Dispatch (ruling 24) and member-storage (ruling
  27) internals are settled, so `csequence.rs`/`csequencer.rs`/`icarus_instance.rs`/
  `ctask*.rs` are transcribable. The `com_developer` gate is unblocked by
  `cvar_integer` (ruling 36), the BlockStream drop-list is settled at five with `Init`
  and `CBlock::Write` live (ICARUS-D1), and the naming rule is fixed (ICARUS-D1). No
  ICARUS open questions remain.
- **Stage-0 interface crate** (`crates/mp/host-interface`, package
  `mp_host_interface`; **built + extended to 15 methods, commit `a9820853`**,
  rulings 24 + 31 + 33 + 36) — the `EngineHost` trait is compiled code
  (`src/engine_host.rs`); this doc `use`s it directly and quotes it verbatim
  (§ Seam definition). Already landed — precedes ICARUS in the port order, no longer
  a pending dependency.
- **Server crate `SV_GameSystemCalls` + `ConvertedEntity`** — the `G_ICARUS_*`
  dispatch arms (`sv_game.cpp:739-832`) call this crate's inbound public API; that
  switch, **and the faithful `ConvertedEntity` port (`sv_game.cpp:420-451`, ruling
  37, sv_game.cpp wave-20)** the five entity-field arms route through, port in the
  server slice and depend on the inbound signatures + the `EngineHost` shape here.
  The `EngineHost` trait impl for `Engine` (via the ICARUS-D2 `EngineHostView`) lands
  at this wave-20 point with the arms. The outbound `GAME_ICARUS_*` handlers already
  exist in `mp_game`.
- **Engine aggregate** — `mp_engine_core::Engine` gains the plain `icarus`
  field (ICARUS-D3, ruling 12) and the ICARUS-D2 `EngineHostView`/`icarus_call`
  split-borrow that implements the 15-method `EngineHost` — including its
  `gentity(ent_num)` service (ruling 23/37, reached from six ICARUS bodies —
  § Raven ground truth gentity-site inventory — not `icarus_valid_ent` alone) and
  `cvar_integer` (ruling 36). STATE-Q2 is CLOSED and `engine.rs:35-38` is reconciled
  (commit `163952dc`), so adding the `icarus` field and the `icarus_call` constructor
  is this slice's remaining work item — not a stale-comment cleanup.
- **Memory/Zone** — no dependency: ICARUS-D3 (ruling 20) drops the arena and
  §20-drops `ICARUS_Malloc`/`ICARUS_Free`, so ICARUS owns all its storage and
  needs no engine-Zone hook.
- **`G_LOCATE_GAME_DATA`/`SV_GentityNum`** (server crate) — installs the
  `sv.gentities`/`sv.gentitySize` base (`sv_game.cpp:329-330`) that the
  `EngineHost::gentity` service reads (ruling 23/37) and that
  `ConvertedEntity`/the entity-field arms build their `*mut sharedEntity_t` over;
  lands with the server slice, before ICARUS's entity-field fns run live.

## Open questions

MUST be empty at FROZEN. **ICARUS-Q1–Q13 are all resolved** — none remains open.

- **ICARUS-Q1–Q11** — resolved by rulings 11–27 (ICARUS-D3): host/context
  threading (ruling 11), arena vs. class allocs (rulings 13/20), golden-fixture
  provenance (ruling 14), out-of-range entnum §19 (ruling 15), `Svcmd_ICARUS_f`
  drop (ruling 17), `Icarus` attachment (ruling 12), residual `TAG_ICARUS5` blob
  home + `ICARUS_Malloc`/`Free` fate (ruling 20), the entity-field seam gap
  (rulings 23 + 37), internal dispatch convention (ruling 24), the `InterfaceExport`
  host-param type (ruling 24), and the two ownership-graph shapes (ruling 27).
- **ICARUS-Q12** (the `com_developer` debug-print gate had no `EngineHost` seam
  method) — **RESOLVED by ruling 36 (ICARUS-D3).** The trait was extended with
  `cvar_integer(&mut self, name: &str) -> i32` (`engine_host.rs`, commit `a9820853`),
  so `Q3_DebugPrint`'s gate `if (!com_developer || !com_developer->integer) return;`
  (`Q3_Interface.cpp:638-643`) ports as `host.cvar_integer("developer") != 0`;
  `MockHost` carries a `cvars` fixture answering it (`mock.rs:121`, `:260`).
- **ICARUS-Q13** (BlockStream `Init` — ruling 39a's drop-list named a live-called
  function) — **RESOLVED by ruling 40 (ICARUS-D1).** The oracle shows the live reader
  `CBlockStream::Open` (`:665`, called from `ICARUS_InterrogateScript`
  `GameInterface.cpp:465` and `CSequencer::Route`/parse `Sequencer.cpp:318`,`:370`)
  calls `Init()` at `BlockStream.cpp:670`, and the writer `Create(char *)` (def
  `:525`) does not — so `Init` has a live caller, fails §20's zero-caller test, and
  ports. Ruling 40 corrects ruling 39a's drop-list to exactly five zero-caller
  methods (`Create(char *)`, `WriteBlock`, `WriteMember`, `CBlock::Duplicate`,
  `CBlockMember::Duplicate`) and confirms `Init`, the `CBlock::Write` overloads, and
  `CBlockMember::SetData`/`WriteData`/`WriteDataPointer` are all live and port. No
  interactive session needed — the entry is closed by cited oracle ground truth +
  porting-rules §20.
