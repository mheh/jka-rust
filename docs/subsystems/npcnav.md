# CNavigator (server/NPCNav) — engine-side nav graph (§F idiomatic reimplementation) Design
Status: DRAFT     Supersedes: none
Decision prefix: NAV     Ledger deps: engine-fork-discovery rulings 11 (one `EngineHost` seam), 12 (`Engine.nav` field), 14 (fixtures), 18 (faithful priority queue), 22 (shared const/vec3 home in `mp_qshared`), 24 (Stage-0 crate PINNED `mp_host_interface` / `crates/mp/host-interface`), 26 (nav tie-order pinned to the oracle-harness libstdc++), 30 (the ent-taking arms carry `*mut sharedEntity_t`), 31/33 (`mp_host_interface` BUILT and green, commit `4b7f01b0`), 32 (MockHost-driven goldens, no test-only ctor); forks 2/3 (state placement, fn-scope statics), 7 (§F doc list). All rulings 11–26 stand (NAV-D5).

C++-track subsystem (porting-rules §F). This doc carries the `files` roster and
`divergences` list so it drops into `.claude/workflows/port-cpp-subsystem.js`
`designPath` unchanged (doc-standards rule 6).

## Standing context

Links only — never restated here:

- `docs/porting-rules.md` — rules; §B (state spine), §F (C++ track), §17–21.
- `docs/plans/2026-07-08-mp-engine-build-out.md` — the MP engine build-out; §0.4
  (server is the integrator; the 39 `SV_GameSystemCalls`→`CNavigator` edges the
  pre-correction graph missed), §3 (parity methodology 3a/3b/3c), §5.1 (C++ in a
  C pipeline).
- `docs/GOAL-engine.md` — no-stub / no-`todo!` port discipline; every fn verified.
- `docs/doc-standards.md` — this template.
- `docs/handoffs/engine-fork-discovery.md` — settled fork rulings (forks 2/3/7)
  and the §F doc-session rulings 11–33 this revision renders.
- GP2 is the §F exemplar: `crates/mp/engine/qcommon/src/gp2/`, `tools/gp2-oracle/`.

## Scope & non-goals

**Decides:** the idiomatic Rust shape of the engine-side nav graph —
`oracle/codemp/server/NPCNav/navigator.cpp` (2,783 lines) +
`oracle/codemp/server/NPCNav/gameCallbacks.cpp` (49 lines): the classes
`CEdge`, `CNode`, `CNavigator`, `CPriorityQueue`, the helper `NodeTotalGreater`,
the file-scope `navigator`/cvars/statics, and the nine `GNavCallback_*`
engine→game out-calls. Header: `oracle/codemp/server/NPCNav/navigator.h`.

**Non-goals (punted, with pointers):**

- The `SV_GameSystemCalls` switch itself (`oracle/codemp/server/sv_game.cpp:837-936`,
  the `G_NAV_*` in-call arms) ports with the rest of that 1,200-LOC function at
  wave 20 — see the build-out plan §0.4. This doc freezes only the `CNavigator`
  pub surface those arms call; NAV-D5 keeps that boundary byte-identical. The
  `SETCHECKEDNODE`/`FLAGALLNODES` switch fall-through is that port's obligation,
  not this one — NAV-D5.
- The game-module twin of this API (the `trap_Nav_*` wrappers and the
  `GAME_NAV_*` handlers `oracle/codemp/game/g_public.h:788-796`) is already
  ported in `mp_game` — see NAV-D5. This doc does not re-port it.
- The `Sys_*`/FS/trace engine services `CNavigator` calls back into
  (`SV_Trace`, `SV_inPVS`, `SV_GentityNum`, `FS_*`, `Com_Error`, `Com_Printf`)
  are reached through the one shared `EngineHost` trait (NAV-D4, RULING 11),
  which is **already BUILT and green** in the pinned `mp_host_interface` crate
  (`crates/mp/host-interface`, RULING 24; commit `4b7f01b0`, RULING 31/33). This
  doc quotes that trait's frozen signatures verbatim (NAV-D4, Seam) but does not
  define it. **Three services nav also needs are NOT on that frozen trait and
  are open seam escalations, not settled here** (like `SV_inPVS`): the cvar reads
  `d_altRoutes`/`d_patched`→`integer` (**NAV-Q9**), the `svs.time` server-frame
  clock (**NAV-Q10**), and `Save`'s `FS_Write` path (**NAV-Q11**). Their
  resolution is a seam/scope decision this doc does not make.

**The Stage-0-covered seam points are settled:** the host seam is built (NAV-D4),
the graph-construction/goldens mechanism is the fixture-backed `MockHost` (NAV-D2,
closes the old NAV-Q7), the shared const/vec3 migration's destination / ownership /
move-vs-re-export are pinned (NAV-D3, closes the old NAV-Q8), and the ent-taking
arms carry `*mut sharedEntity_t` (NAV-D1). **But three services the nav code
provably calls have no method on the frozen `EngineHost` and are covered by no
ruling — cvar reads (NAV-Q9), `svs.time` (NAV-Q10), and `Save`'s FS-write
(NAV-Q11) — and two NAV-D3 execution parameters its "deleted and re-imported in
the same commit, no shims" wording leaves unspecified are, on a tree check, not
mechanically self-resolvable: the cross-crate call-site footprint (NAV-Q12) and
the moved vec3-fn names (NAV-Q13).** None can be self-resolved by an agent
(extending the frozen Stage-0 trait, or threading the value another way, is a
design decision — rulings 31/33 / NAV-D4; the NAV-D3 scope/naming likewise); per
doc-standards Gate-2 they **escalate to an interactive session**, so `## Open
questions` carries five live holes and the doc stays **DRAFT**.

## Raven ground truth

### Frame role and lifecycle

`CNavigator navigator` is a single file-scope global
(`oracle/codemp/server/NPCNav/navigator.cpp:32`) living in the **engine**, not
the game module: the comment at :33-34 states the nav code moved into the engine
and the game reaches it only through `trap_*` syscalls. Its ctor
(:478-484) lazily runs `NAV_CvarInit` (:39-43) to register `d_altRoutes` and
`d_patched` (both `CVAR_CHEAT`). `Init()` (:572-575) just calls `Free()`
(:583-594), which `delete`s every `CNode*` and clears `m_nodes` +
`m_edgeLookupMap`.

The graph is populated one of two ways:
1. **Load** from `maps/<name>.nav` (:602-657): validates `NAV_HEADER_ID`
   (`'JNV5'`, navigator.h:21) and a checksum, then reads `numNodes` `CNode`s
   (each via `CNode::Load`, :426-470, which reads position/flags/id/radius, its
   edge vector, and its per-node rank array), then reads the `failedEdges[]`
   array and rebuilds `m_edgeLookupMap`.
2. **Built live**: `AddRawPoint` (:710-726) appends a node; `HardConnect`
   (:1113-1140) or `CalculatePaths` (:884-908) wires edges; `Save` (:665-702)
   writes it back out.

`CalculatePaths` (:884-908) allocates each node's rank table
(`CNode::InitRanks`, :351-363) then runs `CalculatePath` (:814-877) — a
priority-queue flood fill from each node that fills `m_ranks[targetID] = rank`
for every reachable node. It is pure graph work (no traces). It then calls
`GNavCallback_CP_FindCombatPointWaypoints` (unless `recalc`) and sets
`pathsCalculated = qtrue`.

### Data flow — the graph

- `CNode` (navigator.h:70-126): `m_position` (vec3), `m_flags`, `m_radius`,
  `m_ID`, a `vector<edge_t>` (`edge_t = {int ID; int cost; BYTE flags}`,
  navigator.h:72-77), and a heap `int *m_ranks` (one rank per node,
  `-1`-initialised, navigator.cpp:351-363). `m_numEdges` caps at 8
  (`assert(m_numEdges < 9)`, :182). Edges are stored bidirectionally
  (`SetEdgeCost`/`HardConnect` add to both endpoints, :778-779, :1138-1139).
- `CEdge` (navigator.h:50-62): `{m_first, m_second, m_cost}` — reused by the
  priority queue as a generic `(node, root, cost)` triple, **not** a stored
  graph edge.
- `m_edgeLookupMap` (`multimap<int,int>`, navigator.h:40,248): maps a failed
  edge's `startID` → its index in `failedEdges[]`, so `EdgeFailed`
  (:1876-1923) does `equal_range` lookups instead of a linear scan.
- `failedEdges[MAX_FAILED_EDGES]` (`MAX_FAILED_EDGES = 32`, navigator.h:133,245):
  fixed array of `failedEdge_t {int startID, endID, checkTime, entID}`
  (`oracle/codemp/game/g_public.h:52-58` — the struct is **shared** with the
  game module and crosses the seam by pointer via `G_NAV_CLEARFAILEDEDGE` /
  `G_NAV_CHECKFAILEDEDGE`).

### The priority queue

`CPriorityQueue` (navigator.h:254-276) wraps a `vector<CEdge*> mHeap` driven by
`std::push_heap`/`std::pop_heap` with comparator `NodeTotalGreater`
(:2693-2699, `first->m_cost > second->m_cost` — a **min-heap on cost**). It
owns raw `CEdge*` allocated by `new` in `CalculatePath`; `Pop` returns the
pointer and the caller `delete`s it (:869), the dtor drains and deletes the
rest (:2705-2711). `Push` = `push_back` + `std::push_heap`; `Pop` = read
`front()`, `std::pop_heap`, `pop_back` (:2731-2758). `Find`/`Update`
(:2716-2774) linear-scan `mHeap` by `m_first`.

`CalculatePath` (:814-877) drives it: seed the frontier with each direct edge,
then repeatedly `Pop` the min-cost `CEdge`, assign the popped node
`node->AddRank(testNode->GetID(), curRank++)` (:853), and `Push` each unchecked
neighbour at cumulative cost. **The `curRank++` is assigned in pop order**, so
the order in which equal-cost frontier entries pop is baked into every node's
rank table — the tie-break is parity-visible (NAV-D5, RULING 26).

### The failed-edge / checked-node bookkeeping

- Per-entity failed **nodes** live on `sharedEntity_t`
  (`oracle/codemp/game/g_public.h:706-712`: `waypoint`, `failedWaypoints[8]`
  (`MAX_FAILED_NODES = 8`, g_public.h:673), `failedWaypointCheckTime`), written
  by `AddFailedNode` (:1768-1799) / re-tested by `CheckFailedNodes`
  (:1724-1766) / read by `NodeFailed` (:1801-1811). The `CNavigator` methods
  reach these fields by **dereferencing the `sharedEntity_t *ent` the trap
  hands them** (`(sharedEntity_t *)VMA(1)`, sv_game.cpp:885/888/891), exactly as
  Raven does — see NAV-D1.
- Failed **edges** live in the engine's `failedEdges[]` + `m_edgeLookupMap`;
  `AddFailedEdge` (:1925-2055), `ClearFailedEdge` (:1835-1865),
  `ClearAllFailedEdges` (:1867-1874), `CheckFailedEdge` (:2057-2142),
  `CheckAllFailedEdges` (:2144-2168).
- `CheckedNode`/`SetCheckedNode`/`ClearCheckedNodes` (:1688-1719) memoise
  per-(waypoint,ent) trace results in a file-scope `static map<int,byte>
  CheckedNodes` (:1687) keyed `wayPoint*MAX_GENTITIES+ent`, values
  `CHECKED_NO/FAILED/PASSED` (:54-56). This is genuine cross-call state (fork-3
  kind-3), reset by `ClearCheckedNodes`.

### Globals inventory (all of them — feeds the State-ownership table)

| Raven global | oracle cite |
| --- | --- |
| `CNavigator navigator` | navigator.cpp:32 |
| `cvar_t *d_altRoutes` | navigator.cpp:36 |
| `cvar_t *d_patched` | navigator.cpp:37 |
| `static map<int,byte> CheckedNodes` | navigator.cpp:1687 |
| `static vec3_t wpMaxs`, `wpMins` | navigator.cpp:50-51 |
| `static byte CHECKED_NO/FAILED/PASSED` | navigator.cpp:54-56 |
| `GetTime` statics `timeBase`,`initialized` | navigator.cpp:63-64 (`#if AI_TIMERS`) |

`AI_TIMERS` is **not** in the WinDed Release macro set (plan appendix:
`-DNDEBUG -DDEDICATED -DBOTLIB`; navigator.cpp does not include `b_local.h`
where it is `0`), so the `#if AI_TIMERS` block (`GetTime` + its two statics,
:59-74) does not compile in and does not port — record it with a §20
zero-caller note, do not port speculatively.

## State ownership

Per engine-fork-discovery **RULING 12** (the five §F states — `icarus`, `nav`,
`g2`, `roff`, `rmg` — are plain `Default`-initialized **direct fields on
`Engine`**, no `Option`/`Box`/nesting; lazy-init timing modeled with Raven's own
init flags), **fork-2** (other file-scope globals → fields on the owning
subsystem struct; cvar *handles* in an `EngineCvars` sub-struct; no `static
mut`), and **fork-3** (fn-scope statics: const tables → `const`; genuine
cross-frame state → host field).

| Raven global | oracle cite | Rust owner (crate::Type.field) | constructed by | threaded via |
| --- | --- | --- | --- | --- |
| `navigator` | navigator.cpp:32 | `mp_engine_core::Engine.nav: Navigator` (type in `mp_engine_server::npcnav`) | `Default`-init direct field; RULING 12 | `(&mut self, &mut impl EngineHost)`; NAV-D4 |
| `Navigator.m_nodes` | navigator.h:247 | `Navigator.nodes: Vec<Node>` | `AddRawPoint`/`Load` | owned arena, node id = index; NAV-D5 |
| `Navigator.m_edgeLookupMap` | navigator.h:248 | `Navigator.edge_lookup: BTreeMap<i32, Vec<usize>>` | `AddFailedEdge`/`Load` | owned; NAV-D5 |
| `Navigator.failedEdges[32]` | navigator.h:245 | `Navigator.failed_edges: [failedEdge_t; MAX_FAILED_EDGES]` | ctor/`ClearAllFailedEdges` | owned array |
| `Navigator.pathsCalculated` | navigator.h:215 | `Navigator.paths_calculated: qboolean` | `CalculatePaths` | pub field (NAV-D5 seam get/set) |
| `d_altRoutes`, `d_patched` | navigator.cpp:36-37 | engine cvar handles in `EngineCvars` (fork-2) | `NAV_CvarInit` | handle placement is fork-2; but the frozen `EngineHost` has **no cvar accessor**, so the `->integer` read path (navigator.cpp:480,1403,1418,1433,1498,1933,2278,2323,2346) is an **open seam gap — NAV-Q9**, not reachable via NAV-D4 as frozen |
| `CheckedNodes` static | navigator.cpp:1687 | `Navigator.checked_nodes: BTreeMap<i32, u8>` | first `SetCheckedNode` | owned; fork-3 kind-3. **`BTreeMap` not `HashMap`** — iteration/lookup determinism (plan §3d), NAV-D5 |
| `wpMaxs`/`wpMins` | navigator.cpp:50-51 | module `const WP_MAXS/WP_MINS: [f32;3]` | — | fork-3 kind-1; `WP_MINS`'s `-24+STEPSIZE` reads `STEPSIZE` from `mp_qshared` (NAV-D3) |
| `CHECKED_NO/FAILED/PASSED` | navigator.cpp:54-56 | module `const` (`u8`) | — | fork-3 kind-1 |
| `GetTime` statics | navigator.cpp:63-64 | not ported (`AI_TIMERS` off) | — | §20 dead-surface note |

**Per-entity failed-node fields are NOT engine-owned, and are reached by
dereferencing the trap-marshaled pointer, not by re-fetching through
`SV_GentityNum`.** `sharedEntity_t.{waypoint, failedWaypoints,
failedWaypointCheckTime}` (g_public.h:706-712) live in the game module's entity
array. Under **NAV-D1 (RULING 30)** the five ent-taking arms receive the entity
as a raw `*mut sharedEntity_t` produced by the trap's `(sharedEntity_t *)VMA(1)`
marshal (sv_game.cpp:865/885/888/891/917), and the methods deref that pointer
directly for `ent->s.number`, `ent->r.currentOrigin`, `ent->r.mins/maxs`,
`ent->waypoint`, `ent->failedWaypoints` (navigator.cpp:1159,1202,1217,1223,
1334,1347,1493 and the `AddFailedNode`/`NodeFailed`/`CheckFailedNodes` bodies
:1724-1811) — writing back through the same borrow, exactly as Raven does. The
`SV_GentityNum` service (the `gentity()` `EngineHost` method) is kept **only**
for the genuinely index-based access the nav code still makes by slot number:
`SV_GentityNum(0)` (the player entity) in `GetNearestNode`/`ShowNodes`/`ShowPath`
(navigator.cpp:933,943,947,975,980,1006,1011).

**Shared constants & vec3 helpers the nav code consumes (not nav-owned) — home
in `mp_qshared`, migrated in this doc's first slice.** `Q3_INFINITE`
(`oracle/codemp/game/g_public.h:9`, `16777216`), `WORLD_SIZE`
(`oracle/codemp/game/q_shared.h:20`), `STEPSIZE` (`oracle/codemp/game/bg_public.h:22`,
`18` — used by `WP_MINS`'s `-24+STEPSIZE`, navigator.cpp:51), `WAYPOINT_NONE`
(`oracle/codemp/game/g_nav.h:7`, `-1`), and the vec3 primitives
`VectorNormalize`/`DotProduct`/`VectorSubtract`/`VectorCopy`
(`q_shared.h`/`q_math.c`; used by `GetProjectedNode` and `CNode::GetPosition`)
are **not nav-owned** and are **not** re-declared in npcnav. They live today only
in `mp_game` — the consts at `crates/mp/game/src/g_public_consts.rs:14`
(`Q3_INFINITE`), `.../NPC_combat.rs:2736` (`WORLD_SIZE`), `.../bg_slidemove.rs:37`
(`STEPSIZE`), `.../g_nav_consts.rs:13` (`WAYPOINT_NONE`); the vec3 fns in
`.../q_math.rs`, where **only `VectorNormalize` is at :916** —
`DotProduct`/`VectorSubtract`/`VectorCopy` are Raven **macros** over the
`_`-prefixed C functions transcribed as `_DotProduct` (:961), `_VectorSubtract`
(:968), `_VectorCopy` (:986), **not** bare names at :916 (the prior "`q_math.rs:916`
for all four" cite was wrong; whether the `mp_qshared` copies keep those
`_`-prefixed mp_game/Raven-fn names or adopt the bare macro names is unresolved —
**NAV-Q13**) — none of which the engine can reach (`mp_engine_server` deps:
`mp_qshared`, `mp_engine_qcommon`, `mp_abi` — never `mp_game`). **NAV-D3
(RULING 22 + the round-4 mechanical resolution) MOVES all eight into
`mp_qshared`** — the single definition the referee compares — the vec3 fns to a
new `crates/mp/qshared/src/shared/q_math.rs`, each const to the folder mirroring
its owning Raven header, with the `mp_game` copies **deleted and re-imported in
the same commit** (no re-export shims). This matches the precedent already in
that crate: `Q_irand` (`crates/mp/qshared/src/shared/q_math_rand.rs`) and
`failedEdge_t` (`crates/mp/qshared/src/common/mp/qcommon/failed_edge.rs`) already
live in `mp_qshared`. These four constants are therefore **absent from the
nav-owned globals table above by design** (they are not nav globals); the
nav-owned consts (`NF_*`, `EFLAG_*`, `NODE_NONE`, header IDs, `MAX_FAILED_EDGES`,
`WP_MINS`/`WP_MAXS`, `CHECKED_*` — all from navigator.h) remain module consts per
fork-3 kind-1, and `WP_MINS`/`WP_MAXS` build their `-24+STEPSIZE`/`24` bounds
(navigator.cpp:50-51) from the `mp_qshared`-homed `STEPSIZE` (NAV-D3).

## Seam definition

Two seam directions, both preserved exactly (NAV-D5). The host-taking receiver is
frozen by NAV-D4 (RULING 11/24): every method that reaches a service takes
`(&mut self, host: &mut impl EngineHost)`; the pure-graph queries take no host.

### The `EngineHost` trait (already built — quoted verbatim, NAV-D4)

Per NAV-D4 (RULINGS 31/33) `mp_host_interface` is BUILT and green (commit
`4b7f01b0`); npcnav imports `EngineHost` from `crates/mp/host-interface`, no
other path. The frozen signatures npcnav consumes, transcribed **verbatim** from
`crates/mp/host-interface/src/engine_host.rs:23-106` so this doc is
self-contained (doc-comments elided; `Source:` cites are on each method there):

```rust
pub trait EngineHost {
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

    fn fs_read_file(&mut self, qpath: &str) -> Option<Vec<u8>>;

    fn fs_free_file(&mut self, _buffer: Vec<u8>) {}

    fn print(&mut self, msg: &str);

    fn error(&mut self, code: errorParm_t, msg: &str) -> !;

    fn vm_call(&mut self, vm: VmSlot, callnum: i32, args: &[isize]) -> isize;

    fn shared_memory(&mut self) -> *mut c_char;

    fn flrand(&mut self, min: f32, max: f32) -> f32;

    fn irand(&mut self, min: i32, max: i32) -> i32;

    fn gentity(&mut self, ent_num: i32) -> *mut sharedEntity_t;
}
```

Note: `gentity` returns the raw `*mut sharedEntity_t` exactly as the trap
marshals it (engine_host.rs:100-105 cites rulings 19/23/30) — so the entity-taking
nav arms and this index-based service carry the pointer in the same shape.
`SV_inPVS` is not yet a method on the trait; the trace/PVS-dependent nav methods
(Verification 3c) are added to the trait — or reached through `trace` — when the
server spine lands, not by npcnav.

**Three further services nav calls have no accessor on this frozen trait, and —
unlike `SV_inPVS` — no ruling covers how to reach them; each is an open seam
escalation this doc does not resolve:**

- **NAV-Q9 (cvar reads).** `NAV_CvarInit` registers `d_altRoutes`/`d_patched`
  via `Cvar_Get` (navigator.cpp:41-42, both `CVAR_CHEAT`) and the code reads
  `->integer` at navigator.cpp:480,1403,1418,1433,1498,1933,2278,2323,2346
  (`d_altRoutes` gates the entire alt-route pathing family — parity-visible on
  the 3c surface; `d_patched` gates patched-nav in `AddFailedEdge`). The frozen
  `EngineHost` exposes **no** `Cvar_Get`/cvar-read method, and a nav method
  receives only `(&mut self, host: &mut impl EngineHost)` — neither `self` nor
  the trait can reach the `EngineCvars`-placed handles (fork-2).
- **NAV-Q10 (`svs.time`).** Read at navigator.cpp:1733,1763,1778,1797,1987,2010,
  2065,2137 (failed-node/edge re-check timers; the resulting `checkTime`/
  `failedWaypointCheckTime` values are parity-visible). `svs.time` is the server
  frame time (`serverStatic_t`), not a `Navigator` field; the frozen `EngineHost`
  has no time accessor. (`PlatformHost::milliseconds` is `Sys_Milliseconds`, a
  different clock, and nav never receives `PlatformHost`.)
- **NAV-Q11 (`Save` FS-write).** `Save` uses `FS_FOpenFileByMode(...,FS_WRITE)` +
  `FS_Write` + `FS_FCloseFile` (navigator.cpp:670,678,681,686,697,699), and
  `CNode::Save` writes likewise; the frozen `EngineHost` exposes only
  `fs_read_file` (whole-file read → `Option<Vec<u8>>`) and `fs_free_file` — **no
  write capability**. `Load` (FS_READ) *is* mappable onto `fs_read_file`; `Save`
  is not writable through the frozen trait.

Resolving each (extend the frozen Stage-0 `EngineHost`; or store resolved cvar
values on `Navigator`; or thread `svs.time` another way; or rule `Save`
§20-dead under DEDICATED) changes a settled artifact (rulings 31/33 / NAV-D4) or
this doc's scope, so it is escalated, not decided here.

### Inbound: game → engine (the `G_NAV_*` arms)

The `SV_GameSystemCalls` switch dispatches 42 `G_NAV_*` arms
(`oracle/codemp/game/g_public.h:298-339`) at
`oracle/codemp/server/sv_game.cpp:837-936` — 40 are `CNavigator` **method**
calls (the plan's "39 direct callees" figure, §0.4; two arms are the same
overloaded `GetBestNodeAltRoute`), and two (`G_NAV_GETPATHSCALCULATED`/
`G_NAV_SETPATHSCALCULATED`) read/write the public `pathsCalculated` **field**.
Args arrive as `intptr_t` slots; pointer args use `VMA(n)` (shared-memory base
offset). **Five arms marshal a `(sharedEntity_t *)VMA(1)` and one a second
`(sharedEntity_t *)VMA(2)`** (`GETNEARESTNODE` :865, `CHECKFAILEDNODES` :885,
`ADDFAILEDNODE` :888, `NODEFAILED` :891, `GETBESTPATHBETWEENENTS` :917) — the
Rust seam carries those exactly, as `*mut sharedEntity_t` (NAV-D1). The pub Rust
surface these arms need (`EngineHost` is the one Stage-0 services trait, NAV-D4):

```rust
// Lifecycle / build
fn init(&mut self);                                             // G_NAV_INIT
fn free(&mut self);                                             // G_NAV_FREE
fn load(&mut self, host: &mut impl EngineHost, filename: &str, checksum: i32) -> bool;   // G_NAV_LOAD
fn save(&mut self, host: &mut impl EngineHost, filename: &str, checksum: i32) -> bool;   // G_NAV_SAVE
fn add_raw_point(&mut self, host: &mut impl EngineHost, point: [f32;3], flags: i32, radius: i32) -> i32; // G_NAV_ADDRAWPOINT
fn calculate_paths(&mut self, host: &mut impl EngineHost, recalc: qboolean);             // G_NAV_CALCULATEPATHS
fn hard_connect(&mut self, host: &mut impl EngineHost, first: i32, second: i32);          // G_NAV_HARDCONNECT
fn show_nodes(&mut self, host: &mut impl EngineHost);          // G_NAV_SHOWNODES
fn show_edges(&mut self, host: &mut impl EngineHost);          // G_NAV_SHOWEDGES
fn show_path(&mut self, host: &mut impl EngineHost, start: i32, end: i32);  // G_NAV_SHOWPATH (Com_Printf :1661,:1681)
// Queries (host-free = pure graph; see NAV first-slice)
fn get_nearest_node(&mut self, host: &mut impl EngineHost, ent: *mut sharedEntity_t, last_id: i32, flags: i32, target_id: i32) -> i32; // G_NAV_GETNEARESTNODE ((sharedEntity_t*)VMA(1), sv_game.cpp:865)
fn get_best_node(&mut self, start_id: i32, end_id: i32, reject_id: i32) -> i32;        // G_NAV_GETBESTNODE
fn get_node_position(&self, node_id: i32, out: &mut [f32;3]) -> i32;                   // G_NAV_GETNODEPOSITION
fn get_node_num_edges(&self, node_id: i32) -> i32;            // G_NAV_GETNODENUMEDGES
fn get_node_edge(&self, node_id: i32, edge: i32) -> i32;     // G_NAV_GETNODEEDGE
fn get_num_nodes(&self) -> i32;                              // G_NAV_GETNUMNODES
fn connected(&self, start_id: i32, end_id: i32) -> bool;    // G_NAV_CONNECTED
fn get_path_cost(&self, start_id: i32, end_id: i32) -> u32; // G_NAV_GETPATHCOST
fn get_edge_cost(&mut self, host: &mut impl EngineHost, start_id: i32, end_id: i32) -> u32; // G_NAV_GETEDGECOST
fn get_projected_node(&self, origin: [f32;3], node_id: i32) -> i32;                    // G_NAV_GETPROJECTEDNODE
fn get_node_radius(&self, node_id: i32) -> i32;             // G_NAV_GETNODERADIUS
// Failed-node bookkeeping (deref the *mut sharedEntity_t arg from VMA(1), NAV-D1)
fn check_failed_nodes(&mut self, host: &mut impl EngineHost, ent: *mut sharedEntity_t);              // G_NAV_CHECKFAILEDNODES ((sharedEntity_t*)VMA(1), :885)
fn add_failed_node(&mut self, host: &mut impl EngineHost, ent: *mut sharedEntity_t, node_id: i32);   // G_NAV_ADDFAILEDNODE ((sharedEntity_t*)VMA(1), :888)
fn node_failed(&self, ent: *mut sharedEntity_t, node_id: i32) -> qboolean;                           // G_NAV_NODEFAILED ((sharedEntity_t*)VMA(1), :891)
fn nodes_are_neighbors(&self, start_id: i32, end_id: i32) -> qboolean;                 // G_NAV_NODESARENEIGHBORS
// Failed-edge bookkeeping (failedEdge_t crosses by pointer via VMA)
fn clear_failed_edge(&mut self, host: &mut impl EngineHost, e: &mut failedEdge_t);        // G_NAV_CLEARFAILEDEDGE
fn clear_all_failed_edges(&mut self);                        // G_NAV_CLEARALLFAILEDEDGES
fn edge_failed(&self, start_id: i32, end_id: i32) -> i32;   // G_NAV_EDGEFAILED
fn add_failed_edge(&mut self, host: &mut impl EngineHost, ent_id: i32, start_id: i32, end_id: i32); // G_NAV_ADDFAILEDEDGE (d_patched :1933 = NAV-Q9, Com_Printf :1945-2053, svs.time :1987/2010 = NAV-Q10 — neither on the frozen trait)
fn check_failed_edge(&mut self, host: &mut impl EngineHost, e: &mut failedEdge_t) -> qboolean; // G_NAV_CHECKFAILEDEDGE
fn check_all_failed_edges(&mut self, host: &mut impl EngineHost);                         // G_NAV_CHECKALLFAILEDEDGES
fn route_blocked(&self, start_id: i32, test_edge_id: i32, end_id: i32, reject_rank: i32) -> qboolean; // G_NAV_ROUTEBLOCKED
fn get_best_node_alt_route(&mut self, host: &mut impl EngineHost, start_id: i32, end_id: i32, path_cost: &mut i32, reject_id: i32) -> i32; // G_NAV_GETBESTNODEALTROUTE
fn get_best_node_alt_route2(&mut self, host: &mut impl EngineHost, start_id: i32, end_id: i32, reject_id: i32) -> i32; // G_NAV_GETBESTNODEALT2 (overload)
fn get_best_path_between_ents(&mut self, host: &mut impl EngineHost, ent: *mut sharedEntity_t, goal: *mut sharedEntity_t, flags: i32) -> i32; // G_NAV_GETBESTPATHBETWEENENTS ((sharedEntity_t*)VMA(1)+VMA(2), :917)
fn check_blocked_edges(&mut self, host: &mut impl EngineHost);  // G_NAV_CHECKBLOCKEDEDGES
fn clear_checked_nodes(&mut self);                          // G_NAV_CLEARCHECKEDNODES
fn checked_node(&self, waypoint: i32, ent: i32) -> u8;      // G_NAV_CHECKEDNODE
fn set_checked_node(&mut self, waypoint: i32, ent: i32, value: u8); // G_NAV_SETCHECKEDNODE
fn flag_all_nodes(&mut self, new_flag: i32);                // G_NAV_FLAGALLNODES
// pathsCalculated is a pub field: G_NAV_GETPATHSCALCULATED / _SETPATHSCALCULATED
```

**Switch-fallthrough note (load-bearing, ported at wave 20, not here):**
`G_NAV_SETCHECKEDNODE` (:928-929) and `G_NAV_FLAGALLNODES` (:930-931) have **no
`return`/`break`** — they fall through
(`SETCHECKEDNODE`→`FLAGALLNODES`→`GETPATHSCALCULATED`, which returns
`pathsCalculated`; `oracle/codemp/server/sv_game.cpp:928-933`). This quirk is a
property of the `SV_GameSystemCalls` switch, **not** of any `CNavigator` method;
per NAV-D5 its §20 preservation obligation travels with the wave-20
`SV_GameSystemCalls` port, and NAV-D5 holds this boundary byte-identical. This
doc's `CNavigator` surface neither emits nor asserts the fall-through.

### Outbound: engine → game (`gameCallbacks.cpp`) and engine services

`CNavigator` reaches back into the game module and the rest of the engine
through the one shared `EngineHost` trait (NAV-D4 — designed once at Stage-0 in
the pinned `mp_host_interface` crate / `crates/mp/host-interface`, RULING 24;
BUILT and green, RULING 31/33; not defined by this doc). The services it consumes:

- **Nine game out-calls** (`oracle/codemp/server/NPCNav/gameCallbacks.cpp:6-49`),
  each a thin `VM_Call(gvm, GAME_NAV_*, ...)` (`GAME_NAV_*` enum
  `g_public.h:788-796`; already handled in `mp_game`, NAV-D5) — reached via the
  `EngineHost::vm_call(VmSlot::Gvm, ...)` service:
  `NAV_ClearPathToPoint`, `NPC_ClearLOS`, `NAVNEW_ClearPathBetweenPoints`,
  `NAV_CheckNodeFailedForEnt`, `G_EntIsUnlockedDoor`, `G_EntIsDoor`,
  `G_EntIsBreakable`, `G_EntIsRemovableUsable`, `CP_FindCombatPointWaypoints`.
  The `intptr_t`-slot widening for pointer args is mandatory (plan §5.4 — the
  historical `GAME_NAV_CLEARPATHTOPOINT` truncation bug).
- **Engine services** (each ported by its own subsystem). Reached on the frozen
  `EngineHost`: `SV_Trace` (`server/server.h:416` → `EngineHost::trace`),
  `SV_GentityNum` (server.h:349 → `EngineHost::gentity`, index-based access only,
  State-ownership), the **read** side of `FS_*` for `Load` —
  `FS_FOpenFileByMode(...,FS_READ)`/`FS_Read`/`FS_FCloseFile` → one
  `fs_read_file` whole-file read parsed from an in-memory cursor,
  `Com_Error`(ERR_DROP) (→ `error`), and `Com_Printf` (→ `print`). **Not on the
  frozen trait but deferred to the server-spine work, NOT an npcnav escalation:**
  `SV_inPVS` (server.h:356 — a trace/PVS service added to the trait, or reached
  through `trace`, when the server spine lands; the PVS-dependent nav methods are
  3c-surface, Seam note above). **Not on the frozen trait AND covered by no ruling
  — open seam escalations (Seam note above):** `Cvar_Get` + the
  `d_altRoutes`/`d_patched`→`integer` reads
  (**NAV-Q9**), `svs.time` (**NAV-Q10**), and `Save`'s **write** side —
  `FS_FOpenFileByMode(...,FS_WRITE)`/`FS_Write`/`FS_FCloseFile`, which
  `fs_read_file`/`fs_free_file` cannot express (**NAV-Q11**). `va`/`Q_irand` are pure `q_shared`
  helpers (already ported in `mp_qshared`), not host services; the vec3 primitives
  (`VectorNormalize`/`DotProduct`/`VectorSubtract`/`VectorCopy`) and the shared
  constants `Q3_INFINITE`/`WORLD_SIZE`/`STEPSIZE`/`WAYPOINT_NONE` are the same
  class of shared import (State-ownership "Shared constants" note, NAV-D3) —
  imported from `mp_qshared`, never re-declared in npcnav, never host services.

### `#[repr(C)]` types touched

`failedEdge_t` (g_public.h:52-58) crosses the seam by pointer (`VMA` →
`&mut failedEdge_t`); it is a **shared** struct (game + engine) and keeps exact
layout — imported from the ported type (`mp_qshared`,
`crates/mp/qshared/src/common/mp/qcommon/failed_edge.rs`; the rosetta-registered
Rust name is `failedEdge_t`, **not** `FailedEdge` — there is no `FailedEdge`
alias in the tree), never re-declared (type-rosetta rule).
`sharedEntity_t` (g_public.h:679-715) crosses by pointer as `*mut sharedEntity_t`
on the five ent-taking arms (NAV-D1) and is returned by `EngineHost::gentity`
for index access; `trace_t`, `vec3_t`, `cvar_t` likewise imported.

## Decisions

**NAV-D1** — The five ent-taking nav arms carry `*mut sharedEntity_t` **exactly
as the trap marshals it**, not `EntityId`. Per **RULING 30** (2026-07-09, closing
the contested seam point) `G_NAV_GETNEARESTNODE` (sv_game.cpp:865),
`G_NAV_CHECKFAILEDNODES` (:885), `G_NAV_ADDFAILEDNODE` (:888), `G_NAV_NODEFAILED`
(:891), and `G_NAV_GETBESTPATHBETWEENENTS` (:917) pass `(sharedEntity_t *)VMA(1)`
(the last a second `(sharedEntity_t *)VMA(2)` too), so the seam signatures take
`ent: *mut sharedEntity_t` (`get_best_path_between_ents` takes `ent` **and**
`goal`), and the methods deref the pointer like Raven — `ent->s.number`,
`ent->r.currentOrigin`, `ent->waypoint`, `ent->failedWaypoints`
(navigator.cpp:1159,1202,1217,1223,1334,1347,1493,:1724-1811). Because ruling
23's precedent applies verbatim: an arm that the trap already marshals as a
pointer keeps that pointer at the seam, transcription-first, rather than
round-tripping through an `EntityId`+`SV_GentityNum` re-fetch that Raven never
does. This **replaces every prior `ent: EntityId` seam signature** and the prior
"reached through `SV_GentityNum`" State-ownership note (rewritten this pass), and
**reconciles with NAV-D5's VMA-marshaling clause** — carrying the arm exactly as
the switch presents it *is* "kept exactly as the syscall switch presents it".
The `gentity()`/`SV_GentityNum` `EngineHost` service **stays** for the genuinely
index-based access nav still makes (`SV_GentityNum(0)`, the player entity,
navigator.cpp:933,943,947,975,980,1006,1011). Rejected `ent: EntityId` on these
arms (the withdrawn prior draft): it invents a re-fetch the trap did not marshal
and diverges the seam from ruling 23. (RULING 30, 2026-07-09.)

**NAV-D2** — The 3a goldens are driven by the fixture-backed **`MockHost`** that
now exists at `crates/mp/host-interface/src/mock.rs`; there is **no** test-only
`Navigator` constructor. Per **RULING 32** (2026-07-09, closing the old NAV-Q7)
the golden harness implements `EngineHost` via `MockHost` — its `fs_read_file`
serves the committed `.nav` fixture bytes, `print`/`error` are captured, and
`flrand`/`irand` are deterministic off the faithful `holdrand` LCG replica
(mock.rs:53-89). So **`Load` ports in the first slice with its real frozen
signature** (`load(&mut self, host: &mut impl EngineHost, filename, checksum)`)
and populates `Navigator{nodes, edges}` **through the front door** — the mock's
`fs_read_file` returns the fixture map bytes exactly as `FS_ReadFile` would — and
`CalculatePath` joins the first slice behind it. Because RULING 32 makes the mock
the reusable goldens vehicle for every host-taking subsystem (mock.rs:5-9), so no
subsystem grows a bespoke test seam. Rejected a test-only in-Rust constructor
that sets `nodes`/`edges` directly (the old NAV-Q7 candidate): it bypasses the
real `Load` path the goldens must exercise and adds a seam Raven has no analogue
for. (RULING 32, 2026-07-09.)

**NAV-D3** — The four shared constants and four vec3 primitives the nav code
consumes but does not own **MOVE** — never duplicate — into `mp_qshared`, and
that migration is **in-scope for this doc's first slice**. Per the round-4
mechanical resolution (closing the old NAV-D6/NAV-Q8 hole): the vec3 fns
(`VectorNormalize`/`DotProduct`/`VectorSubtract`/`VectorCopy`) go to a **new**
`crates/mp/qshared/src/shared/q_math.rs` (sibling of `q_math_rand.rs`, mirroring
`oracle/codemp/game/q_math.c`); each const goes to the folder mirroring its
owning Raven header per existing convention — `Q3_INFINITE` (g_public.h) and
`WAYPOINT_NONE` (g_nav.h) under `crates/mp/qshared/src/common/mp/game/`,
`STEPSIZE` (bg_public.h) under `crates/mp/qshared/src/common/mp/bg/`, `WORLD_SIZE`
(q_shared.h) under `crates/mp/qshared/src/shared/`. The existing `mp_game` copies
(`g_public_consts.rs:14`, `g_nav_consts.rs:13`, `bg_slidemove.rs:37`,
`NPC_combat.rs:2736`; and in `q_math.rs`: `VectorNormalize` at :916, `_DotProduct`
at :961, `_VectorSubtract` at :968, `_VectorCopy` at :986 — the last three are
Raven macros wrapping `_`-prefixed C functions, **not** bare names at :916) are
**deleted and re-imported from `mp_qshared` in the SAME commit** — **no re-export
shims**. **Two execution parameters this "delete + re-import, no shims" instruction
leaves open surfaced against the tree and escalate (both block the first slice,
`GetProjectedNode`/`CNode::GetPosition`): the mass cross-crate call-site footprint
the delete-with-no-shim entails — NAV-Q12 — and the `mp_qshared` function names,
`_`-prefixed vs bare — NAV-Q13.** NAV-D3 settles destination, ownership, and
move-vs-re-export; it does **not** settle these two, so a porter cannot
self-resolve them without inventing the answer. Because RULING 22
pinned the destination crate and the no-duplication constraint but left the
owner/paths/move-vs-re-export unassigned; the mechanical resolution assigns them
to this slice so the host-free code (`WP_MINS`/`WP_MAXS` need `STEPSIZE`,
`node.rs`'s `GetPosition` needs `VectorCopy`) can actually be written. The
migration files are listed in this doc's `files` roster. Rejected re-export shims
(would leave two homes, tripping the referee's single-definition compare) and a
separate later ticket (the first slice cannot compile without the moved items).
(RULING 22 + round-4 mechanical resolution, 2026-07-09.)

**NAV-D4** — Services reach nav through the one shared `EngineHost` trait, which
is **already BUILT and green**, and nav state is a direct field on `Engine`. Per
RULING 11 the trace/PVS/FS/print/error/`VM_Call`/shared-memory services are the
single Stage-0 `EngineHost` trait; per **RULING 24** its home crate is pinned to
package **`mp_host_interface`** at **`crates/mp/host-interface`**; per **RULINGS
31/33** that crate is built and green at commit **`4b7f01b0`**, so this doc
**quotes its real frozen signatures verbatim** (Seam,
`crates/mp/host-interface/src/engine_host.rs:23-106`) rather than sketching them.
Every host-taking nav method takes `(&mut self, host: &mut impl EngineHost)`, and
`Engine` supplies the impl through a **split-borrow view struct that excludes
`nav`** — that is what lets `engine.nav.method(&mut view, …)` borrow `nav` and the
rest of `Engine` disjointly. Per RULING 12 the state is a plain `Default`-init
`nav: Navigator` field directly on `mp_engine_core::Engine` (no `Option`/`Box`/
nesting); the ctor's lazy `NAV_CvarInit` (navigator.cpp:39-43,478-484) is modeled
with Raven's own init flag. Because reading the built crate (permitted and
required) makes the doc self-contained and pins the exact method set porters call.
Rejected a nav-private `NavHost` trait and a `Server.navigator` sub-struct —
RULING 11/12/24 supersede both. (RULINGS 11/12/24/31/33.)

**NAV-D5** — All prior settled nav decisions and **rulings 11–26 stand**. The
node/edge graph is owned `Vec` arenas indexed by id (node id == index,
`m_nodes.size()` assignment navigator.cpp:712), never a pointer graph (§B5):
`CNode.m_edges` → `Vec<NodeEdge>` (`NodeEdge` = Raven's CNode-nested `edge_t`
`{ID,cost,flags}`, navigator.h:72-77, defined in `node.rs` alongside `Node` as
CNode's private member type per porting-rules §21 colocation — **not** in
`edge.rs`, which is `CEdge`/`Edge`, the priority-queue triple), `m_ranks` (heap
`int*`) → `Vec<i32>` (`-1`
fill), `m_edgeLookupMap` (`multimap<int,int>`) → `BTreeMap<i32, Vec<usize>>`
(per-key insertion order preserved so `EdgeFailed`'s `equal_range` first-match,
:1876-1898, is reproduced), `CheckedNodes`/`ShowEdges` maps → `BTreeMap`
(iteration/lookup determinism). **The priority queue is transcribed faithfully,
not `std::BinaryHeap`, and its equal-cost tie order is pinned to the
oracle-harness libstdc++** (Homebrew g++-16) `push_heap`/`pop_heap` under
`NodeTotalGreater` (`first->m_cost > second->m_cost`, min-heap on cost,
navigator.cpp:2693-2699) — RULING 26 (2026-07-09): the `<bits/stl_heap.h>`
`__push_heap` sift-up and `__adjust_heap`+`__pop_heap` sift-down are
hand-transcribed onto an owned `Vec<Edge>`, the **one** source read outside
`oracle/`, authoritative because it is the reference `tools/npcnav-oracle/`
compiles the unmodified oracle TU against and the 3a rank goldens are dumped from
(retail-MSVC may tie differently — that divergence is accepted exactly as FP
parity is; the port does not reconstruct the algorithm from memory). Because
`CalculatePath` assigns `curRank++` in pop order (:853) the tie-break is baked
into every rank table and is parity-visible (RULING 18 + 26). The `GAME_NAV_*`/
`G_NAV_*` boundary is kept **exactly** as the syscall switch presents it —
numbers, arg order, **`VMA` marshaling** (including the `sharedEntity_t*` arms,
NAV-D1), `intptr_t`-slot widening — and the `GAME_NAV_*` handlers already in
`mp_game` are not re-ported; the `SETCHECKEDNODE`→`FLAGALLNODES`→
`GETPATHSCALCULATED` switch fall-through (a real Raven bug, no `return`/`break`,
sv_game.cpp:928-933) is owned by the wave-20 `SV_GameSystemCalls` transcription,
not any `CNavigator` method (build-out plan §0.4). Golden fixtures are path
queries over committed hand-authored minimal nav graphs (public, CI-reproducible)
**plus** an uncommitted, ignored-by-default local retail `.nav` corpus (RULING 14
/ ICARUS pattern). Because these were settled before the §F doc session and the
later rulings (24/26/30/31/32/33) refine — they do not overturn — the seam and
heap decisions. Rejected `HashMap` (nondeterministic iteration), `std::BinaryHeap`
(diverges tie order), collapsing the two `GetBestNodeAltRoute` overloads (the game
module issues both arm numbers), and committing retail `.nav` blobs (licensing).
(RULINGS 11–26.)

## Verification strategy

C++ track → porting-rules §F / §18: differential goldens from the unmodified
oracle TU, committed so `cargo test` needs no C++ toolchain. Harness home:
`tools/npcnav-oracle/` (GP2 pattern — stub headers under it, oracle never
edited).

**Fixture sources (NAV-D5, ICARUS ruling-14 pattern):** committed hand-authored
minimal nav graphs are the public, CI-reproducible corpus; the retail `.nav`
data read from the local `jka_server` assets is an **uncommitted,
ignored-by-default** extra corpus that may run locally. Goldens are dumped from
the oracle over both and committed only for the hand-authored set.

**Golden surface (3a, primary — path-query goldens, MockHost-driven):** the Rust
side builds its graph **through the front door** (NAV-D2): the harness seeds a
`MockHost` (`crates/mp/host-interface/src/mock.rs`) whose `fs_read_file` returns
the fixture `.nav` bytes, calls the real `load(&mut self, host, filename,
checksum)`, then `CalculatePaths`/per-node `CalculatePath` to (re)build the rank
tables — no test-only constructor. After load, the pure-graph query surface is
fully deterministic with **no trace/PVS/callback** dependency (the ranks are
baked into the file / recomputed in-process). Dump-and-compare `GetBestNode`,
`GetBestNodeAltRoute`, `GetPathCost`, `Connected`, `NodesAreNeighbors`,
`GetProjectedNode`,
`GetNodeNumEdges`/`GetNodeEdge`/`GetNodePosition`/`GetNodeRadius`/`GetNumNodes`,
plus `GetPathCost` over a rank table populated by `CNode::InitRanks` +
`CalculatePath` (navigator.cpp:351-363, :814-877). The trailing
`GNavCallback_CP_FindCombatPointWaypoints` in the `CalculatePaths` wrapper
(:904) runs after every rank is assigned and touches only combat waypoints, so it
is a `vm_call` the `MockHost` records but that does not perturb the ranks. The
priority-queue tie order (NAV-D5) is exercised transitively through
`CalculatePath`'s rank output — the primary reason the faithful heap is testable
without a bespoke probe; these rank goldens are the binding check on the
`<bits/stl_heap.h>` sift transcription. The oracle side stubs `FS_Read` against
the same fixture bytes and stubs `Com_Printf`/`Cvar_Get` (`d_altRoutes`/
`d_patched` forced to fixed values so both `d_altRoutes` branches are covered).

**Trace/callback-dependent surface (3c, referee swap-in):** `GetNearestNode`,
`GetBestPathBetweenEnts`, `CheckBlockedEdges`, `HardConnect`, `GetEdgeCost`
(both the public `int,int` form — which validates ids then delegates to the
trace form unconditionally, navigator.cpp:2634 — and the `CNode*,CNode*` trace
form :734-755), `CheckFailedNodes`, `CheckFailedEdge`,
`CheckAllFailedEdges` reach `SV_Trace`/`SV_inPVS`/`SV_GentityNum` and the nine
game callbacks (all `EngineHost` services), so they need live engine + game
state — the ent-taking ones also a populated `*mut sharedEntity_t` (NAV-D1),
which the `MockHost` supplies via `gentity_mut` (mock.rs:158-166). They verify
under the plan's §3c A/B referee (`crates/jampgame/tests/referee.rs` / the
external `sv_referee` rig) once the server spine is real, or via captured-trace
replay (§3b), the deterministic `MockHost` injected per NAV-D2/D4.

Governing clause: porting-rules §F (§18 differential goldens; §19 UB
divergence; §20 emergent-quirk preservation; §21 one class per file).

## Slice hooks

- Build-out plan §0.4 / wave 20: `SV_GameSystemCalls` — must have this doc's
  pub surface (Seam definition) frozen before its `G_NAV_*` arms are filled; it
  also owns the SETCHECKEDNODE/FLAGALLNODES fall-through (NAV-D5).
- Build-out plan wave 25 (server complete) / M4: the full nav subsystem must be
  green under the 3c referee swap-in.
- The `EngineHost` trait (Stage-0 `mp_host_interface` crate /
  `crates/mp/host-interface`, RULING 11/24) is **already built and green**
  (commit `4b7f01b0`, NAV-D4); the `Engine` split-borrow view struct that
  excludes `nav` must exist before the host-taking methods compile — a shared
  Stage-0 dependency, not a nav-specific open point.
- **First slice (Load-anchored, MockHost-verified).** Per NAV-D2 the slice ports
  `Load` (`load`, navigator.cpp:602-657 + the `Get*` byte readers :512-564)
  against its real frozen `EngineHost` signature and verifies it with a
  `MockHost` serving the fixture `.nav` bytes — no test-only constructor. On top
  of `Load` it ports **`CalculatePath`** (the host-free inner flood-fill,
  navigator.cpp:814-877, on `navigator.rs`) and the pure-graph queries
  (`GetBestNode`, `GetNodePosition`, `GetNodeNumEdges`, `GetNodeEdge`,
  `GetNumNodes`, `Connected`, `GetPathCost`, `GetProjectedNode`, `GetNodeRadius`),
  plus the type skeletons — `mod.rs` consts (State-ownership table), `edge.rs`
  `Edge` (D-1), `node.rs` `Node` including its `Load` (host `fs_read_file`) and
  `Save` (host `FS_Write` — **NAV-Q11**, no frozen-trait method, so as literally
  scoped this slice **also** blocks on NAV-Q11; see Open questions) and
  host-free members (accessors navigator.h:94-110, edge/rank queries incl.
  `InitRanks`/`AddRank`/`GetRank`, `Create`, `AddEdge`), and `priority_queue.rs`'s
  faithful `Vec<Edge>` heap (NAV-D5/D-7). Together these verify against the 3a
  MockHost-driven Load + rank/query goldens: `Load` fills `nodes`/`edges` through
  the front door, `CalculatePath` produces pop-order ranks, and the queries dump
  against the oracle. The host-taking `CalculatePaths` wrapper (:884-908, its
  `GNavCallback_CP_FindCombatPointWaypoints` at :904) is exercisable here too —
  the `MockHost` records the `vm_call` and it does not perturb ranks.
- **The NAV-D3 shared-home migration is part of this first slice** (RULING 22 +
  round-4 resolution): the four consts and four vec3 primitives MOVE into
  `mp_qshared` (new `crates/mp/qshared/src/shared/q_math.rs` for the vec3 fns;
  each const to the folder mirroring its owning header), with the five `mp_game`
  copies deleted and re-imported in the same commit (no shims). It is listed in
  the `files` roster below and must land so the host-free code that consumes
  `STEPSIZE` (`WP_MINS`/`WP_MAXS`) and `VectorCopy` (`GetPosition`) compiles.
  **Its execution scope (NAV-Q12) and the moved vec3-fn names (NAV-Q13) are
  unresolved and block this slice** — the migration cannot be transcribed until
  they settle (a tree check shows "deleted in the same commit, no shims" touches
  40+ `mp_game` files not in the roster, and only `VectorNormalize` is a bare fn;
  see Open questions).
- Every remaining host-taking method (`AddRawPoint`, `HardConnect`,
  `GetNearestNode`, `GetBestPathBetweenEnts`, `CheckBlockedEdges`, the failed-edge
  checks) and the whole of `callbacks.rs` land against the built `EngineHost`
  trait — under GOAL-engine no-stub discipline a porter writes them against the
  frozen trait, never a stub — and verify under the 3c referee.

## Method transcription table

81 functions (per plan §0.4); inline accessors fold into their owning struct's
impl. Grouped by Raven class; Rust shape per NAV-D1/D5.

| Raven method | oracle cite | Rust shape |
| --- | --- | --- |
| `CEdge::CEdge()` / `(int,int,int)` / `~CEdge` | :82-96 | `Edge { first, second, cost }`; 0-arg ctor is a Raven no-op (divergence D-1) |
| `CNode::CNode`/`~CNode`/`Create(...)`/`Create()` | :104-147 | `Node::new` / `Node::create(pos,flags,radius,id)`; `Vec`-owned (no `new`/`delete`); `GetPosition`'s vec3 helpers imported from `mp_qshared` (NAV-D3) |
| `CNode::AddEdge` | :155-183 | dedup-or-push into `edges: Vec<NodeEdge>`; `assert(<9)` → `debug_assert!` (D-6) |
| `CNode::GetEdgeNumToNode`/`GetEdge`/`GetEdgeCost`/`GetEdgeFlags`/`SetEdgeFlags` | :191-344 | index/scan `edges`; keep `edgeNum > m_numEdges` bound verbatim (D-2) |
| `CNode::AddRank`/`InitRanks`/`GetRank` | :214-376 | `ranks: Vec<i32>` (`-1` fill) |
| `CNode::Draw` | :227-236 | empty (renderer stripped) — port as no-op with §20 note |
| `CNode::Save`/`Load` | :385-470 | `Load` reads via `EngineHost::fs_read_file`; `Save`'s `FS_Write` has **no frozen-trait method — NAV-Q11**; `NODE_HEADER_ID` check |
| `CNode` inline accessors (`GetID`,`GetPosition`,`GetNumEdges`,`GetRadius`,`GetFlags`,`AddFlag`,`RemoveFlag`) | navigator.h:94-110 | trivial methods |
| `CNavigator::CNavigator`/`~CNavigator` | :478-488 | `Navigator::default`; ctor's lazy `NAV_CvarInit` registers the cvars (Raven init flag); the read-back accessor is **unresolved — NAV-Q9** (no cvar method on the frozen trait) |
| `CNavigator::Init`/`Free` | :572-594 | clear `nodes`/`edge_lookup` |
| `CNavigator::Load`/`Save` | :602-702 | `Load` reads whole-file via `EngineHost::fs_read_file` (first slice, NAV-D2), rebuilds `edge_lookup`; `Save`'s `FS_Write`+`FS_FOpenFileByMode(FS_WRITE)` path (:670,678,681,686,697,699) has **no frozen-trait method — NAV-Q11**, unwritable as the trait is frozen |
| `CNavigator::AddRawPoint` | :710-726 | push `Node`; `Com_Error` branch dead (D-3) |
| `CNavigator::GetEdgeCost(int,int)` / `GetEdgeCost(CNode*,CNode*)` | :2621-2635,:734-755 | public `int,int` form validates ids then delegates to the trace form (:2634); `SV_Trace` via host — trace-dependent (3c), host-taking |
| `CNavigator::SetEdgeCost`/`AddNodeEdges` | :757-806 | id-indexed; bidirectional add |
| `CNavigator::CalculatePath`/`CalculatePaths` | :814-908 | faithful `Vec<Edge>` heap flood fill (D-7 raw-ptr ownership → owned values; pop-order ranks NAV-D5); `CalculatePath` in first slice |
| `CNavigator::ShowNodes`/`ShowEdges`/`ShowPath` | :916-1027,:1632-1685 | draw calls stripped (renderer); keep PVS/`Com_Printf` control flow, §20 notes; `SV_GentityNum(0)` index access via `gentity()` (NAV-D1) |
| `CNavigator::GetNodeRadius` | :1029-1034 | pure query — `m_nodes[id].radius` with the §19 range guard (D-8), host-free (golden surface) |
| `CNavigator::CheckBlockedEdges`/`HardConnect` | :1036-1140 | host trace + door/breakable callbacks |
| `CNavigator::TestNodePath`/`TestNodeLOS`/`TestBestFirst` | :1150-1237 | protected; host callbacks; deref the `*mut sharedEntity_t` ent (NAV-D1) |
| `CNavigator::CollectNearestNodes` | :1249-1318 | `nodeChain_l` → `Vec`/`VecDeque` insert-sorted (NAV-D5) |
| `CNavigator::GetBestPathBetweenEnts`/`GetNearestNode` | :1320-1624 | host trace/PVS; `ent`/`goal` are `*mut sharedEntity_t` from VMA (NAV-D1), written back through the pointer (`ent->waypoint`); `SV_GentityNum(0)` via `gentity()` |
| `CNavigator::ClearCheckedNodes`/`CheckedNode`/`SetCheckedNode` | :1687-1719 | `checked_nodes: BTreeMap<i32,u8>` |
| `CNavigator::CheckFailedNodes`/`AddFailedNode`/`NodeFailed` | :1724-1811 | deref the `*mut sharedEntity_t` arg (VMA(1), NAV-D1) — read/write `ent->waypoint`/`failedWaypoints` |
| `CNavigator::NodesAreNeighbors` | :1813-1833 | scan node edges |
| `CNavigator::ClearFailedEdge`/`ClearAllFailedEdges` | :1835-1874 | `failed_edges[..]`; `memset(WAYPOINT_NONE)` → explicit fill |
| `CNavigator::EdgeFailed`/`AddFailedEdge` | :1876-2055 | `edge_lookup` `equal_range` first-match (NAV-D5) |
| `CNavigator::CheckFailedEdge`/`CheckAllFailedEdges` | :2057-2168 | host trace/PVS; `#if 0` NAVNEW branch not taken (D-4) |
| `CNavigator::RouteBlocked` | :2170-2253 | rank-guided walk; `while(1)` loop |
| `CNavigator::GetBestNodeAltRoute` (both overloads) | :2261-2370 | 3-arg delegates to 4-arg |
| `CNavigator::GetBestNode`/`GetNodePosition`/`GetNodeNumEdges`/`GetNodeEdge`/`Connected`/`GetPathCost`/`GetProjectedNode` | :2377-2686 | pure graph queries (golden surface); `Q3_INFINITE`/`WORLD_SIZE`/`WAYPOINT_NONE` + vec3 primitives imported from `mp_qshared` (NAV-D3) |
| `CNavigator::FlagAllNodes`/`GetChar`/`GetInt`/`GetFloat`/`GetLong`/`GetNumNodes` | :496-564,navigator.h:184 | helpers; `Get*` read via host `FS_Read` |
| `NodeTotalGreater::operator()` | :2693-2699 | the `first.cost > second.cost` comparator for the faithful heap sift (NAV-D5) |
| `CPriorityQueue::~/Find/Pop/Push/Update/Empty` | :2705-2782 | owned `Vec<Edge>` with hand-transcribed `push_heap`/`pop_heap` (NAV-D5/D-7); `Find`/`Update` have no live caller — §20 |
| `NAV_CvarInit`/`NAV_Free` | :39-48 | registers `d_altRoutes`/`d_patched` via `Cvar_Get` (:41-42) — the frozen `EngineHost` has **no cvar method — NAV-Q9**; `NAV_Free`→`Navigator::free` |
| `GetTime` (`#if AI_TIMERS`) | :59-74 | not ported (`AI_TIMERS` off) — §20 |
| `CNavigator::GetNodeLeadDistance` | navigator.h:182 | declared-only, **no definition** in navigator.cpp and no caller/trap arm — dropped as dead surface (§20 zero-caller note), not stubbed |
| `GNavCallback_*` ×9 | gameCallbacks.cpp:6-49 | `EngineHost::vm_call(VmSlot::Gvm, GAME_NAV_*)` (NAV-D4/D5); ent args pass the `*mut sharedEntity_t` widened to `isize` slots |

## Divergences

Per porting-rules §19 (diverge only where Raven is UB; note ≤2 lines at the
site; keep out of shared fixtures) and §20 (preserve emergent quirks; drop dead
surface). `port-cpp-subsystem` consumes this list.

- **D-1 (UB):** `CEdge::CEdge(void)` (:82-85) calls `CEdge(-1,-1,-1)` which
  constructs a *discarded temporary* — the real object's `m_first/second/cost`
  are left **uninitialised**. The 0-arg ctor has no live caller (queue nodes use
  `new CEdge(a,b,c)`; graph edges use the inner `edge_t`). Port `Edge` without a
  meaning-bearing `Default`; if one is required, use `Edge{-1,-1,-1}` (the
  author's evident intent). Keep out of fixtures.
- **D-2 (preserved quirk):** `CNode::GetEdge*`/`SetEdgeFlags` bound-check with
  `edgeNum > m_numEdges` not `>=` (:246,273,300,327). Faithful `>` kept; the
  off-by-one is harmless (the `count` scan never matches `edgeNum ==
  m_numEdges`, returns the fallback).
- **D-3 (dead path):** `AddRawPoint` (:714-718) tests `node == NULL` after
  `CNode::Create` (C++ `new` throws, never returns NULL) → the `Com_Error`
  branch is unreachable. Rust `Node::create` is infallible; port the error
  branch as unreachable/omitted with a §20 note.
- **D-4 (dead branch):** `CheckFailedEdge` (:2100-2118) `#if 0` selects the
  `SV_Trace` `#else` arm; the `NAVNEW_ClearPathBetweenPoints` arm does not
  compile. Port only the live `#else`.
- **D-5 (preserved bug):** `GetNearestNode` (:1572-1579) has `if (...==
  CHECKED_FAILED) ... else if (... == CHECKED_FAILED)` — the second branch is a
  duplicate of the first (dead; likely meant `CHECKED_PASSED`). Preserve
  bug-for-bug (§20).
- **D-6 (release-elided assert):** `AddEdge`'s `assert(m_numEdges < 9)` (:182)
  and the `assert(m_ranks)`/`assert(ent...)` sites compile out under `NDEBUG`.
  Port as `debug_assert!` (no release-path behavior change).
- **D-7 (idiomatic ownership, faithful order):** `CPriorityQueue`'s raw `CEdge*`
  `new`/`delete` lifecycle (:818,838,865,869,2705-2758) becomes an owning
  `Vec<Edge>` of values with the C++ `push_heap`/`pop_heap` sift algorithm
  hand-transcribed (NAV-D5). Ownership/layout is free (§F); pop order under
  `NodeTotalGreater` — including equal-cost ties — is preserved to match
  libstdc++. **Not** `std::BinaryHeap` (would diverge tie order). `Find`/`Update`
  (:2716-2774) have no live caller — drop with §20 zero-caller notes.
- **D-8 (UB):** `CNavigator::GetNodeRadius` (:1029-1034) guards only
  `m_nodes.size()==0`, never the `nodeID` range — an out-of-range `nodeID` on a
  **non-empty** graph is unchecked `m_nodes[nodeID]` Vec-index UB (every sibling
  accessor guards `nodeID < 0 || nodeID >= m_nodes.size()`: `GetNodePosition`
  :2447, `GetNodeNumEdges` :2465, `GetNodeEdge` :2483). Per §19 and the engine §F
  unchecked-index policy (engine-fork-discovery RULING 15 — guard-and-return),
  add the sibling range guard and return the function's **own** existing
  empty-graph sentinel `0` (:1032). ≤2-line note at the site; keep the UB path
  out of fixtures.

## files

```
files:
  - path: crates/mp/engine/server/src/npcnav/mod.rs
    crate: mp_engine_server
    mode: mp
    class: (module)
    summary: Nav module root — NF_*/EFLAG_* flags, NODE_NONE, NAV/NODE header IDs, MAX_FAILED_EDGES, WP_MINS/WP_MAXS, CHECKED_* consts (all navigator.h, nav-owned); re-exports. Q3_INFINITE/WORLD_SIZE/STEPSIZE/WAYPOINT_NONE and the vec3 primitives are NOT defined here — imported from mp_qshared (NAV-D3/RULING 22), never local copies. Navigator becomes the Engine.nav field (RULING 12).
  - path: crates/mp/engine/server/src/npcnav/edge.rs
    crate: mp_engine_server
    mode: mp
    class: CEdge
    summary: Edge {first,second,cost} generic triple used by the priority queue (D-1 dead 0-arg ctor).
  - path: crates/mp/engine/server/src/npcnav/node.rs
    crate: mp_engine_server
    mode: mp
    class: CNode
    summary: Node — position/flags/radius/id, edges Vec<NodeEdge>, ranks Vec<i32>; Save/Load (host FS_* via EngineHost), accessors, edge queries. NodeEdge (Raven CNode-nested edge_t {ID,cost,flags}, navigator.h:72-77) is defined HERE alongside Node as CNode's private member type (porting-rules §21 colocation) — NOT in edge.rs, which is CEdge/Edge (the priority-queue triple). GetPosition uses the mp_qshared vec3 primitives (NAV-D3).
  - path: crates/mp/engine/server/src/npcnav/navigator.rs
    crate: mp_engine_server
    mode: mp
    class: CNavigator
    summary: CNavigator — node/edge arenas, failed-edge/checked-node bookkeeping, priority-queue pathfinding, Load/Save (first slice, NAV-D2), the G_NAV_* pub surface; host-taking methods take (&mut self, &mut impl EngineHost) (NAV-D4); the five ent-taking arms take *mut sharedEntity_t (NAV-D1).
  - path: crates/mp/engine/server/src/npcnav/priority_queue.rs
    crate: mp_engine_server
    mode: mp
    class: CPriorityQueue
    summary: Faithful Vec<Edge> min-heap on cost — hand-transcribed push_heap/pop_heap under NodeTotalGreater so equal-cost tie order matches the oracle-harness libstdc++ (NAV-D5/D-7); NOT std::BinaryHeap; Find/Update dropped as zero-caller.
  - path: crates/mp/engine/server/src/npcnav/callbacks.rs
    crate: mp_engine_server
    mode: mp
    class: (GNavCallback free fns)
    summary: The nine GNavCallback_* outbound calls as EngineHost vm_call(VmSlot::Gvm, GAME_NAV_*) shims (gameCallbacks.cpp); NAV_CvarInit/NAV_Free.
  - path: crates/mp/qshared/src/shared/q_math.rs
    crate: mp_qshared
    mode: mp
    class: (q_math.c vec3 fns)
    summary: NAV-D3 migration (NEW file, sibling of q_math_rand.rs, mirrors oracle/codemp/game/q_math.c) — the vec3 fns MOVED here from crates/mp/game/src/q_math.rs (VectorNormalize at :916; _DotProduct at :961, _VectorSubtract at :968, _VectorCopy at :986 — the last three are Raven macros over _-prefixed C fns, NOT bare names at :916), mp_game copies deleted and re-imported in the same commit, no shims. The single engine-reachable definition the referee compares. OPEN: the cross-crate call-site footprint (NAV-Q12) and the mp_qshared fn names, _-prefixed vs bare (NAV-Q13), are escalated — this file's exact contents and edit scope are not portable until they settle.
  - path: crates/mp/qshared/src/common/mp/game/q3_infinite.rs
    crate: mp_qshared
    mode: mp
    class: (g_public.h const)
    summary: NAV-D3 migration — Q3_INFINITE (g_public.h:9) MOVED into the game-mirroring folder (one-const-per-file convention, snake_case leaf); the mp_game copy (g_public_consts.rs:14) deleted and re-imported in the same commit, no shims.
  - path: crates/mp/qshared/src/common/mp/game/waypoint_none.rs
    crate: mp_qshared
    mode: mp
    class: (g_nav.h const)
    summary: NAV-D3 migration — WAYPOINT_NONE (g_nav.h:7) MOVED into the game-mirroring folder (one-const-per-file convention, snake_case leaf); the mp_game copy (g_nav_consts.rs:13) deleted and re-imported in the same commit, no shims.
  - path: crates/mp/qshared/src/common/mp/bg/stepsize.rs
    crate: mp_qshared
    mode: mp
    class: (bg_public.h const)
    summary: NAV-D3 migration — STEPSIZE (bg_public.h:22, 18.0) MOVED into the bg-mirroring folder; the mp_game copy (bg_slidemove.rs:37) deleted and re-imported in the same commit, no shims. Consumed by WP_MINS/WP_MAXS.
  - path: crates/mp/qshared/src/shared/world_size.rs
    crate: mp_qshared
    mode: mp
    class: (q_shared.h const)
    summary: NAV-D3 migration — WORLD_SIZE (q_shared.h:20, 131072.0) MOVED into the shared tier; the mp_game copy (NPC_combat.rs:2736) deleted and re-imported in the same commit, no shims.
```

## Open questions

MUST be empty at FROZEN. **NAV-Q1–Q8 are all resolved** — Q1–Q5 by the §F
doc-session rulings, Q6 by RULING 22, Q7 by RULING 32 (NAV-D2), Q8 by the round-4
mechanical resolution (NAV-D3); retained here as resolved-in-place notes for
cross-doc ID stability (never re-litigate). **NAV-Q9–Q13 are live holes.**
NAV-Q9–Q11 are three frozen-`EngineHost` seam gaps a porter would hit that no
ruling covers — each needs a service the frozen trait
(`crates/mp/host-interface/src/engine_host.rs:23-106`) does not expose. **NAV-Q11
additionally blocks the first slice**: its `Save` (host `FS_Write`) is
first-slice-scoped (`node.rs`, Slice hooks), so under GOAL-engine no-stub
discipline the first slice **as literally scoped** cannot complete until NAV-Q11
resolves or `Save` is dropped from it — which of those two is itself part of the
NAV-Q11 escalation, not settled here. NAV-Q12–Q13
are two NAV-D3 execution parameters the settled "MOVE, delete the copies in the
same commit, no shims" instruction leaves unspecified and that a tree check shows
are not mechanically self-resolvable (both also block the first slice). Resolving any of
the five extends a settled artifact (rulings 31/33 / NAV-D4, or NAV-D3 / RULING 22)
or this doc's scope, so per doc-standards Gate-2 they **escalate to an interactive
session and are not self-resolved here**. The doc stays **DRAFT** until they are
settled.

- **NAV-Q1** — *(Resolved: NAV-D4 / RULING 11/24/31/33.)* Host-threading mechanism
  for the trace/FS/callback services = the one shared Stage-0 `EngineHost` trait,
  now BUILT and green (`crates/mp/host-interface`, commit `4b7f01b0`); every
  host-taking method takes `(&mut self, host: &mut impl EngineHost)`.
- **NAV-Q2** — *(Resolved: NAV-D5 / RULING 14 pattern.)* Fixtures = committed
  hand-authored minimal nav graphs + an uncommitted local retail `.nav` corpus.
  The exact per-fixture probe list is a mechanical Verification-plan detail the
  harness enumerates, not a design point.
- **NAV-Q3** — *(Resolved: NAV-D5 / EVIDENCE.)* The SETCHECKEDNODE/FLAGALLNODES
  switch fall-through is owned by the wave-20 `SV_GameSystemCalls` port; no
  `CNavigator` artifact is responsible for it.
- **NAV-Q4** — *(Resolved: NAV-D4 / RULING 11.)* There is no nav-private host
  trait; the required services are methods on the shared `EngineHost` trait,
  quoted verbatim in the Seam from the built crate. Their Rust signatures live
  with the `EngineHost` design.
- **NAV-Q5** — *(Resolved: NAV-D4 / RULING 11.)* The `Navigator`-vs-rest
  self-borrow is resolved by `Engine`'s split-borrow view struct that excludes
  `nav`, so `engine.nav.method(&mut view, …)` borrows disjointly.
- **NAV-Q6** — *(Resolved: NAV-D3 / RULING 22.)* Canonical engine-reachable home
  for the shared constants/helpers the nav code consumes but does not own =
  `mp_qshared`, one definition the referee compares, no duplication.
- **NAV-Q7** — *(Resolved: NAV-D2 / RULING 32, 2026-07-09.)* How the first slice
  populates `Navigator{nodes, edges}` for its 3a goldens: **through the front
  door** — `Load` ports in the first slice with its real frozen signature and the
  fixture-backed `MockHost` (`crates/mp/host-interface/src/mock.rs`) serves the
  `.nav` bytes via `fs_read_file`; `CalculatePath` joins behind it. No test-only
  constructor is added (RULING 32 makes the mock the reusable goldens vehicle).
- **NAV-Q8** — *(Resolved: NAV-D3 / round-4 mechanical resolution, 2026-07-09.)*
  The NAV-D6/RULING-22 migration mechanics: the four consts and four vec3
  primitives **MOVE** into `mp_qshared` (vec3 fns → new
  `crates/mp/qshared/src/shared/q_math.rs`; each const → the folder mirroring its
  owning Raven header), the five `mp_game` copies **deleted and re-imported in the
  same commit, no re-export shims**, and the migration is **in-scope for this
  doc's first slice** and listed in the `files` roster. Owner (this slice), paths
  (pinned above), and move-vs-re-export (move, no shims) are all settled.
- **NAV-Q9** — *(LIVE — escalate; blocks execution.)* **How does a nav method
  read `d_altRoutes`/`d_patched`?** `NAV_CvarInit` registers both (`Cvar_Get`,
  navigator.cpp:41-42, `CVAR_CHEAT`) and `->integer` is read at
  navigator.cpp:480,1403,1418,1433,1498,1933,2278,2323,2346 — `d_altRoutes` gates
  the whole alt-route pathing family (parity-visible, 3c), `d_patched` gates
  patched-nav in `AddFailedEdge`. The frozen `EngineHost`
  (`engine_host.rs:23-106`) exposes **no** cvar accessor, the handles live in
  `EngineCvars` (fork-2, not on `Navigator`), and a nav method receives only
  `(&mut self, host: &mut impl EngineHost)` — so neither `self` nor the trait can
  reach them. Candidate resolutions (a cvar-read method on the frozen Stage-0
  trait, vs. resolved cvar values stored on `Navigator`) each change a settled
  artifact (rulings 31/33 / NAV-D4); **decision owed at an interactive session,
  not made here.**
- **NAV-Q10** — *(LIVE — escalate; blocks execution.)* **How does a nav method
  read `svs.time`?** Read at navigator.cpp:1733,1763,1778,1797,1987,2010,2065,2137
  (failed-node/edge re-check timers; the stored `checkTime`/
  `failedWaypointCheckTime` are parity-visible). `svs.time` is `serverStatic_t`
  server frame time, not a `Navigator` field, and the frozen `EngineHost` has no
  time accessor (`PlatformHost::milliseconds` = `Sys_Milliseconds` is a different
  clock and nav never receives `PlatformHost`). Resolution (a `svs.time` accessor
  on the frozen trait, vs. threading it another way) is a seam decision;
  **escalate, not decided here.**
- **NAV-Q11** — *(LIVE — escalate; blocks execution AND the first slice.)* **How is `Save` written?**
  `CNavigator::Save` and `CNode::Save` use `FS_FOpenFileByMode(...,FS_WRITE)` +
  `FS_Write` + `FS_FCloseFile` (navigator.cpp:670,678,681,686,697,699), but the
  frozen `EngineHost` exposes only `fs_read_file` (whole-file read →
  `Option<Vec<u8>>`) + `fs_free_file` — **no write**. `Save` stays in scope
  (`G_NAV_SAVE`, sv_game.cpp:845-846) yet is unwritable against the trait as
  frozen; under GOAL-engine no-stub discipline a porter cannot fill its body.
  (`Load`, by contrast, IS mappable: `FS_READ`+`FS_Read` → one `fs_read_file`
  read parsed from a cursor.) Because `node.rs`'s `Save` is scoped into the
  **first slice** (Slice hooks), NAV-Q11 also blocks that slice: whether to **drop
  `Save` from the first slice** or **resolve NAV-Q11 before it starts** is part of
  this same escalation, not decided here. Resolution (add an FS-write method to the
  frozen Stage-0 trait, vs. rule `Save` §20-dead under DEDICATED) changes a settled
  artifact / this doc's scope; **decision owed at an interactive session.**
- **NAV-Q12** — *(LIVE — escalate; blocks the first slice.)* **Does NAV-D3's
  "delete the `mp_game` copies in the SAME commit, no re-export shims" intend the
  full cross-crate call-site edit its wording entails, or a narrower scope?** The
  four vec3 fns are not single-use copies: in `crates/mp/game` `_DotProduct` is
  referenced in 17 files, `_VectorSubtract` in 34, `_VectorCopy` in 43, and
  `VectorNormalize` in 45 (hundreds of call sites total); the four consts add
  `Q3_INFINITE` (15 files), `WAYPOINT_NONE` (13), `STEPSIZE` (5), `WORLD_SIZE` (2).
  Deleting each `mp_game` definition and re-importing from `mp_qshared` in the same
  commit — with **no** re-export shim — therefore edits 40+ `mp_game` files that
  appear in **no** `files:` roster entry, an order of magnitude beyond the
  five-file migration the roster discloses. Whether that mass edit is the intended
  first-slice scope, or NAV-D3 should read narrowly (add the eight items to
  `mp_qshared` for nav's own import and leave `mp_game`'s existing local
  definitions and their callers untouched — which the "no re-export shims / deleted
  in the same commit" text explicitly forbids), is a scope decision a porter cannot
  self-resolve without inventing the answer. **Blocks the first slice**
  (`GetProjectedNode` and `CNode::GetPosition` need the vec3 fns). Escalate; not
  decided here.
- **NAV-Q13** — *(LIVE — escalate; blocks the first slice.)* **What are the moved
  vec3 functions named in `mp_qshared`?** NAV-D3's prose and Seam-adjacent text use
  the bare Raven macro names `VectorNormalize`/`DotProduct`/`VectorSubtract`/
  `VectorCopy`, but only `VectorNormalize` is a bare function in the tree
  (`crates/mp/game/src/q_math.rs:916`); `DotProduct`/`VectorSubtract`/`VectorCopy`
  are Raven `#define` macros over the `_`-prefixed C functions transcribed as
  `_DotProduct` (:961), `_VectorSubtract` (:968), `_VectorCopy` (:986), matching
  Raven's own `q_math.c` function names and mp_game's existing convention (bare
  names reserved for C-macro-style call-site inlining). Whether `mp_qshared` keeps
  the `_`-prefixed names or adopts the bare macro names is a naming decision NAV-D3
  never states; it changes the moved file's contents and every re-import, so a
  porter cannot self-resolve it. Escalate; not decided here.
