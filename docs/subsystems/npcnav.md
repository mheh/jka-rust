# CNavigator (server/NPCNav) — engine-side nav graph (§F idiomatic reimplementation) Design
Status: DRAFT     Supersedes: none
Decision prefix: NAV     Ledger deps: engine-fork-discovery rulings 11 (one `EngineHost` seam), 12 (`Engine.nav` field), 14 (fixtures), 18 (faithful priority queue), 22 (shared const/vec3 home in `mp_qshared`), 24 (Stage-0 crate PINNED `mp_host_interface` / `crates/mp/host-interface`), 26 (nav tie-order pinned to the oracle-harness libstdc++), 30 (the ent-taking arms carry `*mut sharedEntity_t`), 31/33 (`mp_host_interface` BUILT and green), 32 (MockHost-driven goldens, no test-only ctor), 36 (`EngineHost` EXTENDED — `cvar_integer`/`sv_time`/`fs_write_file` — BUILT commit `a9820853`), 39c (engine-side nav RNG routes through `host.irand`, not the qshared free fn), 39d (the shared-const/vec3 migration is the full cross-crate edit at verified file counts; moved vec3 fns keep Raven's `_`-prefixed names); forks 2/3 (state placement, fn-scope statics), 7 (§F doc list). All rulings 11–35 stand (NAV-D4).

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
  and the §F doc-session rulings 11–39 this revision renders.
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
  pub surface those arms call; NAV-D4 keeps that boundary byte-identical. The
  `SETCHECKEDNODE`/`FLAGALLNODES` switch fall-through is that port's obligation,
  not this one — NAV-D4.
- The game-module twin of this API (the `trap_Nav_*` wrappers and the
  `GAME_NAV_*` handlers `oracle/codemp/game/g_public.h:788-796`) is already
  ported in `mp_game` — see NAV-D4. This doc does not re-port it.
- The `Sys_*`/FS/trace/cvar/time engine services `CNavigator` calls back into
  (`SV_Trace`, `SV_inPVS`, `SV_GentityNum`, `FS_*`, `Cvar_*`, `svs.time`,
  `Com_Error`, `Com_Printf`) are reached through the one shared `EngineHost`
  trait (NAV-D1, RULING 11), which is **already BUILT and green** in the pinned
  `mp_host_interface` crate (`crates/mp/host-interface`, RULING 24), **EXTENDED**
  per RULING 36 at commit `a9820853`. This doc quotes that trait's frozen
  signatures verbatim (NAV-D1, Seam) but does not define it.

**Every npcnav seam and state decision is settled; two live holes remain, both
escalating to design sessions: (1) a verification-mechanism question — how the
binary `.nav` 3a golden fixtures are generated (NAV-Q14), which blocks committing
the public golden corpus but not writing/structure-testing the first-slice code;
and (2) a cross-cutting Stage-0 gap shared by all five §F docs — the concrete Rust
shape of the `Engine` split-borrow view struct that excludes `nav` (NAV-Q15;
RULING 11 pins the pattern, not the shape), which does NOT block npcnav's first
slice (the methods drive through `MockHost` generically) but does block wiring the
`G_NAV_*` arms into the real `SV_GameSystemCalls` at wave 20. No npcnav seam or
state decision is open.** The three
services nav needs that the earlier 10-method trait lacked — the cvar reads
`d_altRoutes`/`d_patched`→`integer`, the `svs.time` server-frame clock, and
`Save`'s `FS_Write` path — are now the frozen trait's `cvar_integer`, `sv_time`,
and `fs_write_file` (NAV-D1 / RULING 36; the former NAV-Q9/Q10/Q11). Engine-side
nav RNG is settled to `host.irand`, not the qshared free fn (NAV-D2 / RULING 39c;
the former Q_irand fork). The shared-const/vec3 migration's execution scope and
the moved-fn names are settled (NAV-D3 / RULING 39d; the former NAV-Q12/Q13): it
is the full cross-crate edit at the tree-verified file counts, and the vec3 fns
keep Raven's `_`-prefixed names. `SV_inPVS` remains the one service nav uses that
is **not** on the frozen trait — but it is a **cross-doc deferral to the
server-spine work, not an open question** (added to the trait, or reached through
`trace`, when the server spine lands; the PVS-dependent nav methods are all on
the 3c referee surface). All former open questions NAV-Q1–Q13 are resolved and
retained in `## Open questions` as resolved-in-place notes for cross-doc ID
stability; the live items there are NAV-Q14 (the binary `.nav` fixture-generation
mechanism, a verification-strategy decision) and NAV-Q15 (the cross-cutting
`Engine` split-borrow view-struct shape), both for design sessions.

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
rank table — the tie-break is parity-visible (NAV-D4, RULING 26).

### The failed-edge / checked-node bookkeeping

- Per-entity failed **nodes** live on `sharedEntity_t`
  (`oracle/codemp/game/g_public.h:706-712`: `waypoint`, `failedWaypoints[8]`
  (`MAX_FAILED_NODES = 8`, g_public.h:673), `failedWaypointCheckTime`), written
  by `AddFailedNode` (:1768-1799) / re-tested by `CheckFailedNodes`
  (:1724-1766) / read by `NodeFailed` (:1801-1811). The `CNavigator` methods
  reach these fields by **dereferencing the `sharedEntity_t *ent` the trap
  hands them** (`(sharedEntity_t *)VMA(1)`, sv_game.cpp:885/888/891), exactly as
  Raven does — see NAV-D4. `AddFailedNode`'s recheck stamp is
  `svs.time + CHECK_FAILED_EDGE_INTERVAL + Q_irand(0,1000)` (:1763) — `svs.time`
  via `sv_time` (NAV-D1), the jitter via `host.irand` (NAV-D2).
- Failed **edges** live in the engine's `failedEdges[]` + `m_edgeLookupMap`;
  `AddFailedEdge` (:1925-2055), `ClearFailedEdge` (:1835-1865),
  `ClearAllFailedEdges` (:1867-1874), `CheckFailedEdge` (:2057-2142),
  `CheckAllFailedEdges` (:2144-2168). The `checkTime` stamps at :1987 and :2137
  are the same `svs.time + CHECK_FAILED_EDGE_INTERVAL + Q_irand(0,1000)` shape
  (NAV-D1 `sv_time` + NAV-D2 `host.irand`).
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
| `navigator` | navigator.cpp:32 | `mp_engine_core::Engine.nav: Navigator` (type in `mp_engine_server::npcnav`) | `Default`-init direct field; RULING 12 | `(&mut self, &mut impl EngineHost)`; NAV-D1 |
| `Navigator.m_nodes` | navigator.h:247 | `Navigator.nodes: Vec<Node>` | `AddRawPoint`/`Load` | owned arena, node id = index; NAV-D4 |
| `Navigator.m_edgeLookupMap` | navigator.h:248 | `Navigator.edge_lookup: BTreeMap<i32, Vec<usize>>` | `AddFailedEdge`/`Load` | owned; NAV-D4 |
| `Navigator.failedEdges[32]` | navigator.h:245 | `Navigator.failed_edges: [failedEdge_t; MAX_FAILED_EDGES]` | ctor/`ClearAllFailedEdges` | owned array |
| `Navigator.pathsCalculated` | navigator.h:215 | `Navigator.paths_calculated: qboolean` | `CalculatePaths` | pub field (NAV-D4 seam get/set) |
| `d_altRoutes`, `d_patched` | navigator.cpp:36-37 | engine cvar handles in `EngineCvars` (fork-2); **read through `EngineHost::cvar_integer`** | `NAV_CvarInit` | the `->integer` read path (navigator.cpp:480,1403,1418,1433,1498,1933,2278,2323,2346) resolves via `cvar_integer(name)` (NAV-D1 / RULING 36); an unregistered name reads 0 |
| `CheckedNodes` static | navigator.cpp:1687 | `Navigator.checked_nodes: BTreeMap<i32, u8>` | first `SetCheckedNode` | owned; fork-3 kind-3. **`BTreeMap` not `HashMap`** — iteration/lookup determinism (plan §3d), NAV-D4 |
| `wpMaxs`/`wpMins` | navigator.cpp:50-51 | module `const WP_MAXS/WP_MINS: [f32;3]` | — | fork-3 kind-1; `WP_MINS`'s `-24+STEPSIZE` reads `STEPSIZE` from `mp_qshared` (NAV-D3) |
| `CHECKED_NO/FAILED/PASSED` | navigator.cpp:54-56 | module `const` (`u8`) | — | fork-3 kind-1 |
| `svs.time` (server-frame clock) | server/server.h:211 | not nav-owned — `serverStatic_t` frame clock, **read through `EngineHost::sv_time`** | server | the recheck-timer reads (navigator.cpp:1733,1763,1778,1797,1987,2010,2065,2137) resolve via `sv_time()` (NAV-D1 / RULING 36) |
| `GetTime` statics | navigator.cpp:63-64 | not ported (`AI_TIMERS` off) | — | §20 dead-surface note |

**Per-entity failed-node fields are NOT engine-owned, and are reached by
dereferencing the trap-marshaled pointer, not by re-fetching through
`SV_GentityNum`.** `sharedEntity_t.{waypoint, failedWaypoints,
failedWaypointCheckTime}` (g_public.h:706-712) live in the game module's entity
array. Under **NAV-D4 (RULING 30)** the five ent-taking arms receive the entity
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
in `mp_qshared`, migrated in this doc's first slice (NAV-D3).** `Q3_INFINITE`
(`oracle/codemp/game/g_public.h:9`, `16777216`), `WORLD_SIZE`
(`oracle/codemp/game/q_shared.h:20`), `STEPSIZE` (`oracle/codemp/game/bg_public.h:22`,
`18` — used by `WP_MINS`'s `-24+STEPSIZE`, navigator.cpp:51), `WAYPOINT_NONE`
(`oracle/codemp/game/g_nav.h:7`, `-1`), and the vec3 primitives — `VectorNormalize`
(bare, `crates/mp/game/src/q_math.rs:916`) and the `_`-prefixed Raven fns
`_DotProduct` (:961), `_VectorSubtract` (:968), `_VectorCopy` (:986) that
`DotProduct`/`VectorSubtract`/`VectorCopy` are Raven `#define` macros over — are
**not nav-owned** and are **not** re-declared in npcnav. They live today only in
`mp_game` (consts: `g_public_consts.rs:14`, `NPC_combat.rs:2736`,
`bg_slidemove.rs:37`, `g_nav_consts.rs:13`), which `mp_engine_server` cannot reach
(its deps are `mp_qshared`, `mp_engine_qcommon`, `mp_abi` — never `mp_game`).
**NAV-D3 (RULING 22 + round-4 resolution + RULING 39d) MOVES all eight into
`mp_qshared`** — the single definition the referee compares — the vec3 fns to a
new `crates/mp/qshared/src/shared/q_math.rs` **keeping the `_`-prefixed names**
(RULING 39d, the former NAV-Q13), each const to the folder mirroring its owning
Raven header, with the `mp_game` copies **deleted and re-imported in the same
commit** (no re-export shims) across the full cross-crate footprint (RULING 39d,
the former NAV-Q12). This matches the precedent already in that crate: `Q_irand`
(`crates/mp/qshared/src/shared/q_math_rand.rs`) and `failedEdge_t`
(`crates/mp/qshared/src/common/mp/qcommon/failed_edge.rs`) already live in
`mp_qshared`. These four constants are therefore **absent from the nav-owned
globals table above by design** (they are not nav globals); the nav-owned consts
(`NF_*`, `EFLAG_*`, `NODE_NONE`, header IDs, `MAX_FAILED_EDGES`, `WP_MINS`/`WP_MAXS`,
`CHECKED_*` — all from navigator.h) remain module consts per fork-3 kind-1, and
`WP_MINS`/`WP_MAXS` build their `-24+STEPSIZE`/`24` bounds (navigator.cpp:50-51)
from the `mp_qshared`-homed `STEPSIZE` (NAV-D3).

## Seam definition

Two seam directions, both preserved exactly (NAV-D4). The host-taking receiver is
frozen by NAV-D1 (RULING 11/24/36): every method that reaches a service takes
`(&mut self, host: &mut impl EngineHost)`; the pure-graph queries take no host.

### The `EngineHost` trait (already built + EXTENDED — quoted verbatim, NAV-D1)

Per NAV-D1 (RULINGS 31/33 built the trait, RULING 36 EXTENDED it) `mp_host_interface`
is BUILT and green (commit `a9820853`); npcnav imports `EngineHost` from
`crates/mp/host-interface`, no other path. The frozen **15-method** trait npcnav
consumes, transcribed **verbatim** from
`crates/mp/host-interface/src/engine_host.rs:24-155` so this doc is
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

    fn cvar_integer(&mut self, name: &str) -> i32;

    fn sv_time(&mut self) -> i32;

    fn fs_write_file(&mut self, qpath: &str, data: &[u8]) -> bool;

    fn model_mdxm(&mut self, model: qhandle_t) -> *mut c_void;

    fn model_mdxa(&mut self, model: qhandle_t) -> *mut c_void;
}
```

Notes on the methods nav uses:

- `gentity` returns the raw `*mut sharedEntity_t` exactly as the trap marshals it
  (engine_host.rs cites rulings 19/23/30) — so the entity-taking nav arms
  (NAV-D4) and this index-based service carry the pointer in the same shape.
- `cvar_integer(name)` (RULING 36) is nav's `d_altRoutes`/`d_patched`→`integer`
  read path; an unregistered name reads 0, as `Cvar_VariableIntegerValue` does
  (the former NAV-Q9). The frozen trait has **no cvar-registration method** (it is
  read-only on a name), so `NAV_CvarInit`'s `Cvar_Get(...)` **registration** half
  (navigator.cpp:41-42) has **no Rust counterpart** — it is elided; only the read
  side ports, and the unregistered-name-reads-0 fallback is exactly what makes the
  elision behavior-preserving (the ctor's `NAV_CvarInit` init flag still gates the
  lazy-once semantics, but registers nothing).
- `sv_time()` (RULING 36) is `svs.time`, the `serverStatic_t` frame clock the
  failed-node/edge recheck timers read; **not** `PlatformHost::milliseconds`
  (`Sys_Milliseconds`, a different clock nav never receives) (the former NAV-Q10).
- `fs_write_file(qpath, data)` (RULING 36) collapses `Save`'s
  `FS_FOpenFileByMode(...,FS_WRITE)` + `FS_Write` + `FS_FCloseFile` sequence;
  `false` mirrors the NULL-handle open failure (the former NAV-Q11).
- `model_mdxm`/`model_mdxa` are ghoul2 loader-memory accessors (G2SV-D5) with no
  nav caller — quoted only because nav shares the one trait.

`SV_inPVS` is the one service nav uses that is **not** yet a method on the trait;
the trace/PVS-dependent nav methods (Verification 3c) reach it when the server
spine lands — either a method added to the trait then, or reached through
`trace` — **not by npcnav** (a cross-doc deferral, not an open question).

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
Rust seam carries those exactly, as `*mut sharedEntity_t` (NAV-D4). The pub Rust
surface these arms need (`EngineHost` is the one Stage-0 services trait, NAV-D1):

```rust
// Lifecycle / build
fn init(&mut self);                                             // G_NAV_INIT
fn free(&mut self);                                             // G_NAV_FREE
fn load(&mut self, host: &mut impl EngineHost, filename: &str, checksum: i32) -> bool;   // G_NAV_LOAD
fn save(&mut self, host: &mut impl EngineHost, filename: &str, checksum: i32) -> bool;   // G_NAV_SAVE (FS_Write via host.fs_write_file, NAV-D1)
fn add_raw_point(&mut self, host: &mut impl EngineHost, point: [f32;3], flags: i32, radius: i32) -> i32; // G_NAV_ADDRAWPOINT
fn calculate_paths(&mut self, host: &mut impl EngineHost, recalc: qboolean);             // G_NAV_CALCULATEPATHS
fn hard_connect(&mut self, host: &mut impl EngineHost, first: i32, second: i32);          // G_NAV_HARDCONNECT
fn show_nodes(&mut self, host: &mut impl EngineHost);          // G_NAV_SHOWNODES
fn show_edges(&mut self, host: &mut impl EngineHost);          // G_NAV_SHOWEDGES
fn show_path(&mut self, host: &mut impl EngineHost, start: i32, end: i32);  // G_NAV_SHOWPATH (Com_Printf :1661,:1681)
// Queries — host-free pure graph (golden surface; see NAV first-slice) EXCEPT
// get_nearest_node and get_edge_cost, which take host (trace-dependent, 3c)
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
// Failed-node bookkeeping (deref the *mut sharedEntity_t arg from VMA(1), NAV-D4)
fn check_failed_nodes(&mut self, host: &mut impl EngineHost, ent: *mut sharedEntity_t);              // G_NAV_CHECKFAILEDNODES ((sharedEntity_t*)VMA(1), :885)
fn add_failed_node(&mut self, host: &mut impl EngineHost, ent: *mut sharedEntity_t, node_id: i32);   // G_NAV_ADDFAILEDNODE ((sharedEntity_t*)VMA(1), :888; svs.time via host.sv_time + host.irand jitter, :1763)
fn node_failed(&self, ent: *mut sharedEntity_t, node_id: i32) -> qboolean;                           // G_NAV_NODEFAILED ((sharedEntity_t*)VMA(1), :891)
fn nodes_are_neighbors(&self, start_id: i32, end_id: i32) -> qboolean;                 // G_NAV_NODESARENEIGHBORS
// Failed-edge bookkeeping (failedEdge_t crosses by pointer via VMA)
fn clear_failed_edge(&mut self, host: &mut impl EngineHost, e: &mut failedEdge_t);        // G_NAV_CLEARFAILEDEDGE
fn clear_all_failed_edges(&mut self);                        // G_NAV_CLEARALLFAILEDEDGES
fn edge_failed(&self, start_id: i32, end_id: i32) -> i32;   // G_NAV_EDGEFAILED
fn add_failed_edge(&mut self, host: &mut impl EngineHost, ent_id: i32, start_id: i32, end_id: i32); // G_NAV_ADDFAILEDEDGE (d_patched :1933 via host.cvar_integer NAV-D1; Com_Printf :1945-2053; svs.time :1987/2010 via host.sv_time + host.irand jitter :1987 NAV-D2)
fn check_failed_edge(&mut self, host: &mut impl EngineHost, e: &mut failedEdge_t) -> qboolean; // G_NAV_CHECKFAILEDEDGE (svs.time :2065/2137 via host.sv_time + host.irand jitter :2137)
fn check_all_failed_edges(&mut self, host: &mut impl EngineHost);                         // G_NAV_CHECKALLFAILEDEDGES
fn route_blocked(&self, start_id: i32, test_edge_id: i32, end_id: i32, reject_rank: i32) -> qboolean; // G_NAV_ROUTEBLOCKED
fn get_best_node_alt_route(&mut self, host: &mut impl EngineHost, start_id: i32, end_id: i32, path_cost: &mut i32, reject_id: i32) -> i32; // G_NAV_GETBESTNODEALTROUTE (d_altRoutes :2278/2323/2346 via host.cvar_integer NAV-D1)
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
per NAV-D4 its §20 preservation obligation travels with the wave-20
`SV_GameSystemCalls` port, and NAV-D4 holds this boundary byte-identical. This
doc's `CNavigator` surface neither emits nor asserts the fall-through.

### Outbound: engine → game (`gameCallbacks.cpp`) and engine services

`CNavigator` reaches back into the game module and the rest of the engine
through the one shared `EngineHost` trait (NAV-D1 — designed once at Stage-0 in
the pinned `mp_host_interface` crate / `crates/mp/host-interface`, RULING 24;
BUILT + EXTENDED, RULING 31/33/36; not defined by this doc). The services it
consumes:

- **Nine game out-calls** (`oracle/codemp/server/NPCNav/gameCallbacks.cpp:6-49`),
  each a thin `VM_Call(gvm, GAME_NAV_*, ...)` (`GAME_NAV_*` enum
  `g_public.h:788-796`; already handled in `mp_game`, NAV-D4) — reached via the
  `EngineHost::vm_call(VmSlot::Gvm, ...)` service:
  `NAV_ClearPathToPoint`, `NPC_ClearLOS`, `NAVNEW_ClearPathBetweenPoints`,
  `NAV_CheckNodeFailedForEnt`, `G_EntIsUnlockedDoor`, `G_EntIsDoor`,
  `G_EntIsBreakable`, `G_EntIsRemovableUsable`, `CP_FindCombatPointWaypoints`.
  The `intptr_t`-slot widening for pointer args is mandatory (plan §5.4 — the
  historical `GAME_NAV_CLEARPATHTOPOINT` truncation bug). The nine free-fn
  signatures (`callbacks.rs`), transcribed from the `gameCallbacks.cpp` bodies —
  each `host: &mut impl EngineHost` first (it wraps `host.vm_call`), the `vec3_t`
  args crossing as pointers, and the `sharedEntity_t *` params **passing only
  `self->s.number`** (the callback derefs the pointer and widens the int; matches
  the `mp_game` `GAME_NAV_*` decoders' `entity_num: c_int`, e.g.
  `crates/mp/abi/src/game/vmcalls/GAME_NAV_CLEARPATHTOPOINT.rs`), return types
  exactly as the C:

  ```rust
  fn GNavCallback_NAV_ClearPathToPoint(host: &mut impl EngineHost, self_ent: *mut sharedEntity_t, pmins: &vec3_t, pmaxs: &vec3_t, point: &vec3_t, clipmask: i32, ok_to_hit_ent_num: i32) -> qboolean; // GAME_NAV_CLEARPATHTOPOINT — passes self_ent->s.number, pmins, pmaxs, point, clipmask, okToHitEntNum; gameCallbacks.cpp:6-9
  fn GNavCallback_NPC_ClearLOS(host: &mut impl EngineHost, ent: *mut sharedEntity_t, end: &vec3_t) -> qboolean; // GAME_NAV_CLEARLOS — ent->s.number, end; :11-14
  fn GNavCallback_NAVNEW_ClearPathBetweenPoints(host: &mut impl EngineHost, start: &vec3_t, end: &vec3_t, mins: &vec3_t, maxs: &vec3_t, ignore: i32, clipmask: i32) -> i32; // GAME_NAV_CLEARPATHBETWEENPOINTS; :16-19
  fn GNavCallback_NAV_CheckNodeFailedForEnt(host: &mut impl EngineHost, ent: *mut sharedEntity_t, node_num: i32) -> qboolean; // GAME_NAV_CHECKNODEFAILEDFORENT — ent->s.number, nodeNum; :21-24
  fn GNavCallback_G_EntIsUnlockedDoor(host: &mut impl EngineHost, entity_num: i32) -> qboolean; // GAME_NAV_ENTISUNLOCKEDDOOR; :26-29
  fn GNavCallback_G_EntIsDoor(host: &mut impl EngineHost, entity_num: i32) -> qboolean; // GAME_NAV_ENTISDOOR; :31-34
  fn GNavCallback_G_EntIsBreakable(host: &mut impl EngineHost, entity_num: i32) -> qboolean; // GAME_NAV_ENTISBREAKABLE; :36-39
  fn GNavCallback_G_EntIsRemovableUsable(host: &mut impl EngineHost, ent_num: i32) -> qboolean; // GAME_NAV_ENTISREMOVABLEUSABLE; :41-44
  fn GNavCallback_CP_FindCombatPointWaypoints(host: &mut impl EngineHost); // GAME_NAV_FINDCOMBATPOINTWAYPOINTS (void); :46-49
  ```
- **Engine services**, all now on the frozen trait except `SV_inPVS`:
  `SV_Trace` (`server/server.h:416` → `EngineHost::trace`),
  `SV_GentityNum` (server.h:349 → `EngineHost::gentity`, index-based access only,
  State-ownership), the **read** side of `FS_*` for `Load` —
  `FS_FOpenFileByMode(...,FS_READ)`/`FS_Read`/`FS_FCloseFile` → **one**
  `fs_read_file` whole-file read parsed from an in-memory cursor; the **write**
  side of `FS_*` for `Save` → **one** `fs_write_file` over a byte buffer (NAV-D1 /
  RULING 36, the former NAV-Q11). The oracle's `CNode::Save`/`Load` take a
  **shared** `fileHandle_t` (navigator.cpp:385,426) and the `CNavigator` loop drives
  them against that one handle (`(*ni)->Save(numNodes, file)` :693, `node->Load(
  numNodes, file)` :637); the Rust port preserves that shape — `Node::save`/`load`
  take the shared cursor/byte-buffer, the single host call is at `Navigator` level
  only, never per-node (a per-node `fs_write_file` is a whole-file overwrite — no
  append mode — so N node calls would clobber all but the last); `Cvar_Get`'s `->integer` reads → `cvar_integer` (NAV-D1 / RULING 36,
  the former NAV-Q9); `svs.time` → `sv_time` (NAV-D1 / RULING 36, the former
  NAV-Q10); `Com_Error`(ERR_DROP) → `error`; `Com_Printf` → `print`. **Not on the
  frozen trait but deferred to the server-spine work, NOT an npcnav escalation:**
  `SV_inPVS` (server.h:356 — added to the trait, or reached through `trace`, when
  the server spine lands; the PVS-dependent nav methods are 3c-surface, Seam note
  above).
- **RNG.** The three nav `Q_irand( 0, 1000 )` sites (navigator.cpp:1763,1987,2137,
  each `svs.time + CHECK_FAILED_EDGE_INTERVAL + Q_irand(0,1000)`) draw from
  `EngineHost::irand` — ruling 21's engine-owned LCG — **not** the qshared
  `Q_irand` free fn (NAV-D2 / RULING 39c; the two LCG instances differ and the
  jitter is parity-visible). `Q_flrand` nav has none. `va` **is** a pure
  `q_shared` helper (already ported in `mp_qshared`), not a host service; the
  vec3 primitives (`VectorNormalize`/`_DotProduct`/`_VectorSubtract`/`_VectorCopy`)
  and the shared constants `Q3_INFINITE`/`WORLD_SIZE`/`STEPSIZE`/`WAYPOINT_NONE`
  are the same class of shared import (State-ownership "Shared constants" note,
  NAV-D3) — imported from `mp_qshared`, never re-declared in npcnav, never host
  services.

### `#[repr(C)]` types touched

`failedEdge_t` (g_public.h:52-58) crosses the seam by pointer (`VMA` →
`&mut failedEdge_t`); it is a **shared** struct (game + engine) and keeps exact
layout — imported from the ported type (`mp_qshared`,
`crates/mp/qshared/src/common/mp/qcommon/failed_edge.rs`; the rosetta-registered
Rust name is `failedEdge_t`, **not** `FailedEdge` — there is no `FailedEdge`
alias in the tree), never re-declared (type-rosetta rule).
`sharedEntity_t` (g_public.h:679-715) crosses by pointer as `*mut sharedEntity_t`
on the five ent-taking arms (NAV-D4) and is returned by `EngineHost::gentity`
for index access; `trace_t`, `vec3_t`, `cvar_t` likewise imported.

## Decisions

**NAV-D1** — Services reach nav through the **one shared `EngineHost` trait,
which is now BUILT and EXTENDED**, and nav state is a direct field on `Engine`.
Per RULING 11 the trace/PVS/FS/print/error/`VM_Call`/shared-memory services are
the single Stage-0 `EngineHost` trait; per **RULING 24** its home crate is pinned
to package **`mp_host_interface`** at **`crates/mp/host-interface`**; per RULINGS
31/33 it was built and green, and per **RULING 36** (2026-07-09, closing the
former NAV-Q9/NAV-Q10/NAV-Q11) it is **EXTENDED** at commit **`a9820853`** with
the three services nav provably needs that the earlier 10-method trait lacked:
`cvar_integer(&mut self, name: &str) -> i32` (the `d_altRoutes`/`d_patched`
`->integer` reads at navigator.cpp:480,1403,1418,1433,1498,1933,2278,2323,2346;
an unregistered name reads 0, as `Cvar_VariableIntegerValue` does — the former
NAV-Q9), `sv_time(&mut self) -> i32` (`svs.time`, the `serverStatic_t` frame
clock the recheck timers read at navigator.cpp:1733,1763,1778,1797,1987,2010,
2065,2137 — **not** `PlatformHost::milliseconds`/`Sys_Milliseconds`, a different
clock nav never receives — the former NAV-Q10), and
`fs_write_file(&mut self, qpath: &str, data: &[u8]) -> bool`
(`CNavigator::Save`/`CNode::Save` port through it; `Save` **stays in the first
slice** — the former NAV-Q11). This doc **quotes the real 15-method frozen trait
verbatim** (Seam, `crates/mp/host-interface/src/engine_host.rs:24-155`). Every
host-taking nav method takes `(&mut self, host: &mut impl EngineHost)`, and
`Engine` supplies the impl through a **split-borrow view struct that excludes
`nav`** — that is what lets `engine.nav.method(&mut view, …)` borrow `nav` and the
rest of `Engine` disjointly (RULING 11 settles this *approach*; the view struct's
concrete Rust shape — type name, home module, field list, how the disjoint borrow
is expressed — is a cross-cutting Stage-0 gap not yet defined anywhere, **NAV-Q15**,
which does not block npcnav's first slice: those methods take `&mut impl EngineHost`
generically and drive through `MockHost`, never through `Engine`'s own impl). Per RULING 12 the state is a plain `Default`-init
`nav: Navigator` field directly on `mp_engine_core::Engine` (no `Option`/`Box`/
nesting); the ctor's lazy `NAV_CvarInit` (navigator.cpp:39-43,478-484) is modeled
with Raven's own init flag. Because reading the built crate (permitted and
required) makes the doc self-contained and pins the exact method set porters call,
with no seam gap remaining. Rejected a nav-private `NavHost` trait, a
`Server.navigator` sub-struct (RULING 11/12/24 supersede both), and — for the
three extension methods — storing resolved cvar values on `Navigator` or threading
`svs.time` another way (RULING 36 put all three on the one shared trait).
(RULINGS 11/12/24/31/33/36.)

**NAV-D2** — Engine-side nav RNG routes through **`EngineHost::irand`** — ruling
21's engine-owned `holdrand` LCG reached through the host — **never** the qshared
`Q_irand` free function. Per **RULING 39c** (2026-07-09, closing the `Q_irand`
LCG fork) the three nav `Q_irand( 0, 1000 )` call sites (navigator.cpp:1763,1987,
2137, each `svs.time + CHECK_FAILED_EDGE_INTERVAL + Q_irand(0,1000)` seeding a
failed-node/edge `checkTime`) draw from the engine's own LCG instance, which is a
*different* `holdrand` from the qshared free-fn instance and whose draw is
**parity-visible through the jitter** these `checkTime` values carry. This
**CORRECTS** the earlier Seam/outbound classification that listed `Q_irand` as "a
pure `q_shared` helper" alongside `va`: at these three sites nav calls
`host.irand(0, 1000)`, a host service — not a free-fn call. `va` remains a pure
`q_shared` helper; the qshared `Q_irand`/`Q_flrand` free fns stay for game-tier
callers, untouched. Because a second `holdrand` instance would desync the timer
jitter from the oracle. Rejected the qshared free fn at these sites (the withdrawn
classification): it draws from the wrong LCG. (RULING 39c, 2026-07-09.)

**NAV-D3** — The four shared constants and four vec3 primitives the nav code
consumes but does not own **MOVE** into `mp_qshared`, and that migration is the
**full cross-crate edit at its verified file counts**, executed as
delete-and-re-import in a **single commit with no re-export shims**; the moved
vec3 fns **keep Raven's `_`-prefixed names**. Per **RULING 39d** (2026-07-09,
closing the former NAV-Q12 and NAV-Q13): NAV-D6/D3's migration **IS** the full
cross-crate edit — the `mp_game` copies are **deleted and re-imported in the SAME
commit, no shims** — at the tree-verified footprint: in `crates/mp/game`
`_DotProduct` is referenced in **17** files, `_VectorSubtract` in **34**,
`_VectorCopy` in **43**, `VectorNormalize` in **45**, `Q3_INFINITE` in **15**,
`WAYPOINT_NONE` in **13**, `STEPSIZE` in **5**, `WORLD_SIZE` in **2** — the
migration commit edits every one of those call sites to import from `mp_qshared`
(the former NAV-Q12: the mass edit IS the intended scope, not a narrow "leave
`mp_game` untouched" reading, which the no-shims/same-commit text forbids). The
moved vec3 fns **keep Raven's underscore-prefixed function names exactly as
`mp_game`'s `q_math.rs` has them today** — `_DotProduct`
(`crates/mp/game/src/q_math.rs:961`), `_VectorSubtract` (:968), `_VectorCopy`
(:986); `VectorNormalize` stays bare (:916) (the former NAV-Q13: no rename; the
tree uses `_`-prefixed for the three the `DotProduct`/`VectorSubtract`/`VectorCopy`
Raven macros wrap). Destinations: the vec3 fns to a **new**
`crates/mp/qshared/src/shared/q_math.rs` (sibling of `q_math_rand.rs`, mirroring
`oracle/codemp/game/q_math.c`); each const to the folder mirroring its owning
Raven header — `Q3_INFINITE` (g_public.h) / `WAYPOINT_NONE` (g_nav.h) under
`crates/mp/qshared/src/common/mp/game/`, `STEPSIZE` (bg_public.h) under
`.../common/mp/bg/`, `WORLD_SIZE` (q_shared.h) under `.../shared/`. The migration
is **in-scope for this doc's first slice** and listed in the `files` roster.
Because RULING 22 pinned the destination crate and no-duplication, the round-4
resolution assigned owner/paths/move-vs-re-export, and RULING 39d fixed the two
remaining execution parameters (footprint scope + fn names) so the first slice's
host-free code (`WP_MINS`/`WP_MAXS` need `STEPSIZE`, `CNode::GetPosition` needs
`_VectorCopy`) is fully portable. Rejected re-export shims (two homes trip the
referee's single-definition compare), a narrow "add to `mp_qshared` but leave
`mp_game` untouched" scope (NAV-Q12 — forbidden by the no-shims/same-commit text),
and bare macro names for the moved fns (NAV-Q13 — the tree uses `_`-prefixed).
(RULING 22 + round-4 resolution + RULING 39d, 2026-07-09.)

**NAV-D4** — All prior settled nav decisions and **rulings 11–35 stand**; the
three the body leans on most are called out here. **(RULING 30 — pointer-carrying
ent arms.)** The five ent-taking nav arms carry `*mut sharedEntity_t` **exactly as
the trap marshals it** — `(sharedEntity_t *)VMA(1)` on `G_NAV_GETNEARESTNODE`
(sv_game.cpp:865), `G_NAV_CHECKFAILEDNODES` (:885), `G_NAV_ADDFAILEDNODE` (:888),
`G_NAV_NODEFAILED` (:891), and `(sharedEntity_t *)VMA(1)`+`VMA(2)` on
`G_NAV_GETBESTPATHBETWEENENTS` (:917) — not `EntityId`; the methods deref the
pointer like Raven (`ent->s.number`, `ent->r.currentOrigin`, `ent->waypoint`,
`ent->failedWaypoints`, navigator.cpp:1159,1202,1217,1223,1334,1347,1493,
:1724-1811), writing back through the same borrow. The `gentity()`/`SV_GentityNum`
service stays **only** for the genuinely index-based `SV_GentityNum(0)` player
access (navigator.cpp:933,943,947,975,980,1006,1011). **(RULING 32 — MockHost-driven
Load goldens.)** The 3a goldens build the graph **through the front door**: the
harness seeds the fixture-backed `MockHost` (`crates/mp/host-interface/src/mock.rs`)
whose `fs_read_file` serves the committed `.nav` bytes, calls the real
`load(&mut self, host, filename, checksum)`, then `CalculatePath` — there is **no**
test-only `Navigator` constructor. **(RULING 26 — libstdc++ tie-order.)** The
priority queue is transcribed faithfully (**not** `std::BinaryHeap`) with the
`<bits/stl_heap.h>` `push_heap`/`pop_heap` sift hand-transcribed onto an owned
`Vec<Edge>` under `NodeTotalGreater` (`first->m_cost > second->m_cost`, min-heap
on cost, navigator.cpp:2693-2699); equal-cost tie order is pinned to the
oracle-harness libstdc++ (Homebrew g++-16) — the **one** source read outside
`oracle/`, authoritative because `tools/npcnav-oracle/` compiles the unmodified
oracle TU against it and dumps the 3a rank goldens from it (retail-MSVC may tie
differently — accepted exactly as FP parity is). Because `CalculatePath` assigns
`curRank++` in pop order (:853), the tie-break is baked into every rank table and
is parity-visible (RULING 18 + 26). The node/edge graph is owned `Vec` arenas
indexed by id (node id == index, `m_nodes.size()` assignment :712), never a
pointer graph (§B5): `CNode.m_edges` → `Vec<NodeEdge>` (`NodeEdge` = Raven's
CNode-nested `edge_t` `{ID,cost,flags}`, navigator.h:72-77, defined in `node.rs`
per §21 colocation — **not** in `edge.rs`, which is `CEdge`/`Edge`), `m_ranks`
(heap `int*`) → `Vec<i32>` (`-1` fill), `m_edgeLookupMap` (`multimap<int,int>`) →
`BTreeMap<i32, Vec<usize>>` (per-key insertion order preserved so `EdgeFailed`'s
`equal_range` first-match :1876-1898 is reproduced), `CheckedNodes`/`ShowEdges`
maps → `BTreeMap` (iteration/lookup determinism). The `GAME_NAV_*`/`G_NAV_*`
boundary is kept **exactly** as the syscall switch presents it — numbers, arg
order, `VMA` marshaling (including the `sharedEntity_t*` arms), `intptr_t`-slot
widening — and the `GAME_NAV_*` handlers already in `mp_game` are not re-ported;
the `SETCHECKEDNODE`→`FLAGALLNODES`→`GETPATHSCALCULATED` switch fall-through (a
real Raven bug, no `return`/`break`, sv_game.cpp:928-933) is owned by the wave-20
`SV_GameSystemCalls` transcription, not any `CNavigator` method (build-out plan
§0.4). Golden fixtures are path queries over committed hand-authored minimal nav
graphs (public, CI-reproducible) **plus** an uncommitted, ignored-by-default local
retail `.nav` corpus (RULING 14 / ICARUS pattern). Because these were settled
before / across the §F sessions and the later rulings (36/39c/39d) **extend** —
they do not overturn — the seam, heap, and migration decisions. Rejected
`EntityId` on the ent arms (invents a re-fetch the trap did not marshal, diverges
from ruling 23), a test-only in-Rust constructor (bypasses the real `Load` the
goldens must exercise), `HashMap` (nondeterministic iteration), `std::BinaryHeap`
(diverges tie order), collapsing the two `GetBestNodeAltRoute` overloads (the game
issues both arm numbers), and committing retail `.nav` blobs (licensing).
(RULINGS 11–35.)

## Verification strategy

C++ track → porting-rules §F / §18: differential goldens from the unmodified
oracle TU, committed so `cargo test` needs no C++ toolchain. Harness home:
`tools/npcnav-oracle/` (GP2 pattern — stub headers under it, oracle never
edited).

**Fixture sources (NAV-D4, ICARUS ruling-14 pattern):** committed hand-authored
minimal nav graphs are the public, CI-reproducible corpus; the retail `.nav`
data read from the local `jka_server` assets is an **uncommitted,
ignored-by-default** extra corpus that may run locally. Goldens are dumped from
the oracle over both and committed only for the hand-authored set. **How the
binary `.nav` bytes of the hand-authored corpus are *produced* — there is no
human-writable `.nav` source form — is NOT settled (NAV-Q14, live): RULING 14
pins the corpus policy, not the byte-generation mechanism (the ICARUS pattern
uses a dedicated `tools/ibi-gen`; npcnav has no equivalent yet). A porter must
NOT invent one; it escalates to a design session.**

**Golden surface (3a, primary — path-query goldens, MockHost-driven):** the Rust
side builds its graph **through the front door** (NAV-D4): the harness seeds a
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
priority-queue tie order (NAV-D4) is exercised transitively through
`CalculatePath`'s rank output — the primary reason the faithful heap is testable
without a bespoke probe; these rank goldens are the binding check on the
`<bits/stl_heap.h>` sift transcription. The oracle side stubs `FS_Read` against
the same fixture bytes and stubs `Com_Printf`/`Cvar_Get` (`d_altRoutes`/
`d_patched` forced to fixed values — mirrored on the Rust side by the `MockHost`'s
`cvar_integer` (NAV-D1) — so both `d_altRoutes` branches are covered).

**RNG determinism (NAV-D2):** the `checkTime` jitter (`Q_irand(0,1000)` at
:1763/1987/2137) is driven by `host.irand` — the `MockHost`'s deterministic
engine-LCG replica (mock.rs) — **not** the qshared free fn, so the stamped
`checkTime`/`failedWaypointCheckTime` values reproduce the oracle's engine-LCG
draw sequence. The oracle side seeds the same engine `holdrand` instance.

**Trace/callback-dependent surface (3c, referee swap-in):** `GetNearestNode`,
`GetBestPathBetweenEnts`, `CheckBlockedEdges`, `HardConnect`, `GetEdgeCost`
(both the public `int,int` form — which validates ids then delegates to the
trace form unconditionally, navigator.cpp:2634 — and the `CNode*,CNode*` trace
form :734-755), `CheckFailedNodes`, `CheckFailedEdge`,
`CheckAllFailedEdges` reach `SV_Trace`/`SV_inPVS`/`SV_GentityNum` and the nine
game callbacks (all `EngineHost` services except `SV_inPVS`, deferred to the
server spine), so they need live engine + game state — the ent-taking ones also a
populated `*mut sharedEntity_t` (NAV-D4), which the `MockHost` supplies via
`gentity_mut` (mock.rs). They verify under the plan's §3c A/B referee
(`crates/jampgame/tests/referee.rs` / the external `sv_referee` rig) once the
server spine is real, or via captured-trace replay (§3b), the deterministic
`MockHost` injected per NAV-D1/D4.

Governing clause: porting-rules §F (§18 differential goldens; §19 UB
divergence; §20 emergent-quirk preservation; §21 one class per file).

## Slice hooks

- Build-out plan §0.4 / wave 20: `SV_GameSystemCalls` — must have this doc's
  pub surface (Seam definition) frozen before its `G_NAV_*` arms are filled; it
  also owns the SETCHECKEDNODE/FLAGALLNODES fall-through (NAV-D4).
- Build-out plan wave 25 (server complete) / M4: the full nav subsystem must be
  green under the 3c referee swap-in.
- The `EngineHost` trait (Stage-0 `mp_host_interface` crate /
  `crates/mp/host-interface`, RULING 11/24/36) is **already built, EXTENDED, and
  green** (commit `a9820853`, NAV-D1); the `Engine` split-borrow view struct that
  excludes `nav` (RULING 11) must exist before the host-taking methods are wired
  into the real `SV_GameSystemCalls` (wave 20), but its concrete Rust shape is
  **not yet defined anywhere** — a cross-cutting Stage-0 design gap, **NAV-Q15**.
  It does **not** block npcnav's first slice, which drives the host-taking methods
  directly through `MockHost` (generic `&mut impl EngineHost`), never through
  `Engine`'s own trait impl.
- **First slice (Load-anchored, MockHost-verified).** Per NAV-D4 the slice ports
  `Load` (`load`, navigator.cpp:602-657 + the `Get*` byte readers :512-564)
  against its real frozen `EngineHost` signature and verifies it with a
  `MockHost` serving the fixture `.nav` bytes — no test-only constructor. On top
  of `Load` it ports **`CalculatePath`** (the host-free inner flood-fill,
  navigator.cpp:814-877, on `navigator.rs`) and the pure-graph queries
  (`GetBestNode`, `GetNodePosition`, `GetNodeNumEdges`, `GetNodeEdge`,
  `GetNumNodes`, `Connected`, `GetPathCost`, `GetProjectedNode`, `GetNodeRadius`),
  plus the type skeletons — `mod.rs` consts (State-ownership table), `edge.rs`
  `Edge` (D-1), `node.rs` `Node` including its `Load` (host `fs_read_file`) and
  `Save` (host `fs_write_file`, NAV-D1 / RULING 36 — a frozen-trait method, so
  `Save` is fully portable in the first slice) and host-free members (accessors
  navigator.h:94-110, edge/rank queries incl. `InitRanks`/`AddRank`/`GetRank`,
  `Create`, `AddEdge`), and `priority_queue.rs`'s faithful `Vec<Edge>` heap
  (NAV-D4/D-7). Together these verify against the 3a MockHost-driven Load +
  rank/query goldens: `Load` fills `nodes`/`edges` through the front door,
  `CalculatePath` produces pop-order ranks, and the queries dump against the
  oracle. The host-taking `CalculatePaths` wrapper (:884-908, its
  `GNavCallback_CP_FindCombatPointWaypoints` at :904) is exercisable here too —
  the `MockHost` records the `vm_call` and it does not perturb ranks.
- **The NAV-D3 shared-home migration is part of this first slice** (RULING 22 +
  round-4 resolution + RULING 39d): the four consts and four vec3 primitives MOVE
  into `mp_qshared` (new `crates/mp/qshared/src/shared/q_math.rs` for the vec3 fns,
  keeping the `_`-prefixed names; each const to the folder mirroring its owning
  header), with the `mp_game` copies deleted and re-imported in the same commit
  (no shims) across the full cross-crate footprint (17/34/43/45 fn files +
  15/13/5/2 const files, NAV-D3). It must land so the host-free code that consumes
  `STEPSIZE` (`WP_MINS`/`WP_MAXS`) and `_VectorCopy` (`GetPosition`) compiles. Its
  execution scope and the moved fn names are settled (NAV-D3, the former
  NAV-Q12/Q13), so the migration is fully transcribable.
- Every remaining host-taking method (`AddRawPoint`, `HardConnect`,
  `GetNearestNode`, `GetBestPathBetweenEnts`, `CheckBlockedEdges`, the failed-edge
  checks) and the whole of `callbacks.rs` land against the built `EngineHost`
  trait — under GOAL-engine no-stub discipline a porter writes them against the
  frozen trait, never a stub — and verify under the 3c referee.

## Method transcription table

81 functions (per plan §0.4); inline accessors fold into their owning struct's
impl. Grouped by Raven class; Rust shape per NAV-D1/D4.

| Raven method | oracle cite | Rust shape |
| --- | --- | --- |
| `CEdge::CEdge()` / `(int,int,int)` / `~CEdge` | :82-96 | `Edge { first, second, cost }`; 0-arg ctor is a Raven no-op (divergence D-1) |
| `CNode::CNode`/`~CNode`/`Create(...)`/`Create()` | :104-147 | `Node::new` / `Node::create(pos,flags,radius,id)`; `Vec`-owned (no `new`/`delete`); `GetPosition`'s vec3 helpers imported from `mp_qshared` (NAV-D3, `_`-prefixed names) |
| `CNode::AddEdge` | :155-183 | dedup-or-push into `edges: Vec<NodeEdge>`; `assert(<9)` → `debug_assert!` (D-6) |
| `CNode::GetEdgeNumToNode`/`GetEdge`/`GetEdgeCost`/`GetEdgeFlags`/`SetEdgeFlags` | :191-344 | index/scan `edges`; keep `edgeNum > m_numEdges` bound verbatim (D-2) |
| `CNode::AddRank`/`InitRanks`/`GetRank` | :214-376 | `ranks: Vec<i32>` (`-1` fill) |
| `CNode::Draw` | :227-236 | empty (renderer stripped) — port as no-op with §20 note |
| `CNode::Save`/`Load` | :385-470 | Raven signatures `Save(int numNodes, fileHandle_t file)` / `Load(int numNodes, fileHandle_t file)` take a **shared** open handle, not their own FS call → the Rust methods take a **shared in-memory cursor/byte-buffer** param (the analogue of that shared `fileHandle_t`), **not** `&mut impl EngineHost`; they read/write header/position/flags/id/radius/edges/ranks against that cursor. `NODE_HEADER_ID` check. The single whole-file host call lives at `Navigator::load`/`save` (see below), never per-node — a per-node `fs_write_file(qpath, …)` would whole-file-overwrite and clobber all but the last node (no append mode in the frozen trait) |
| `CNode` inline accessors (`GetID`,`GetPosition`,`GetNumEdges`,`GetRadius`,`GetFlags`,`AddFlag`,`RemoveFlag`) | navigator.h:94-110 | trivial methods |
| `CNavigator::CNavigator`/`~CNavigator` | :478-488 | `Navigator::default`; ctor's lazy `NAV_CvarInit` runs once behind Raven's own init flag, but its `Cvar_Get` **registration** side has **no frozen-trait counterpart** and is elided (Seam cvar note) — only the `->integer` read-back ports, via `EngineHost::cvar_integer` (NAV-D1) |
| `CNavigator::Init`/`Free` | :572-594 | clear `nodes`/`edge_lookup` |
| `CNavigator::Load`/`Save` | :602-702 | the **one** whole-file host call lives here: `Load` does a single `EngineHost::fs_read_file` (first slice, NAV-D4) into a cursor, then loops `CNode::load(numNodes, &mut cursor)` (:637) over it, reads `failedEdges`, rebuilds `edge_lookup`; `Save` builds one byte buffer — writes header/checksum/`numNodes`, loops `CNode::save(numNodes, &mut buf)` (:693), appends `failedEdges` — then a single `EngineHost::fs_write_file` (NAV-D1 / RULING 36), collapsing the oracle's shared-`fileHandle_t` `FS_FOpenFileByMode(FS_WRITE)`+`FS_Write`×N+`FS_FCloseFile` (:670,678,681,686,697,699). Mirrors the oracle's single-shared-handle loop exactly |
| `CNavigator::AddRawPoint` | :710-726 | push `Node`; `Com_Error` branch dead (D-3) |
| `CNavigator::GetEdgeCost(int,int)` / `GetEdgeCost(CNode*,CNode*)` | :2621-2635,:734-755 | public `int,int` form validates ids then delegates to the trace form (:2634); `SV_Trace` via host — trace-dependent (3c), host-taking |
| `CNavigator::SetEdgeCost`/`AddNodeEdges` | :757-806 | id-indexed; bidirectional add |
| `CNavigator::CalculatePath`/`CalculatePaths` | :814-908 | faithful `Vec<Edge>` heap flood fill (D-7 raw-ptr ownership → owned values; pop-order ranks NAV-D4); `CalculatePath` in first slice |
| `CNavigator::ShowNodes`/`ShowEdges`/`ShowPath` | :916-1027,:1632-1685 | draw calls stripped (renderer); keep PVS/`Com_Printf` control flow, §20 notes; `SV_GentityNum(0)` index access via `gentity()` (NAV-D4) |
| `CNavigator::GetNodeRadius` | :1029-1034 | pure query — `m_nodes[id].radius` with the §19 range guard (D-8), host-free (golden surface) |
| `CNavigator::CheckBlockedEdges`/`HardConnect` | :1036-1140 | host trace + door/breakable callbacks |
| `CNavigator::TestNodePath`/`TestNodeLOS`/`TestBestFirst` | :1150-1237 | protected; host callbacks; deref the `*mut sharedEntity_t` ent (NAV-D4) |
| `CNavigator::CollectNearestNodes` | :1249-1318 | `nodeChain_l` → `Vec`/`VecDeque` insert-sorted (NAV-D4) |
| `CNavigator::GetBestPathBetweenEnts`/`GetNearestNode` | :1320-1624 | host trace/PVS; `ent`/`goal` are `*mut sharedEntity_t` from VMA (NAV-D4), written back through the pointer (`ent->waypoint`); `SV_GentityNum(0)` via `gentity()`; `AddFailedNode` stamp `svs.time`+jitter via `sv_time`/`irand` (:1763, NAV-D1/D2) |
| `CNavigator::ClearCheckedNodes`/`CheckedNode`/`SetCheckedNode` | :1687-1719 | `checked_nodes: BTreeMap<i32,u8>` |
| `CNavigator::CheckFailedNodes`/`AddFailedNode`/`NodeFailed` | :1724-1811 | deref the `*mut sharedEntity_t` arg (VMA(1), NAV-D4) — read/write `ent->waypoint`/`failedWaypoints`; `svs.time`+`Q_irand` stamp via `sv_time`/`irand` (:1763, NAV-D1/D2) |
| `CNavigator::NodesAreNeighbors` | :1813-1833 | scan node edges |
| `CNavigator::ClearFailedEdge`/`ClearAllFailedEdges` | :1835-1874 | `failed_edges[..]`; `memset(WAYPOINT_NONE)` → explicit fill |
| `CNavigator::EdgeFailed`/`AddFailedEdge` | :1876-2055 | `edge_lookup` `equal_range` first-match (NAV-D4); `d_patched` via `cvar_integer` (:1933, NAV-D1); `svs.time`+`Q_irand` stamp via `sv_time`/`irand` (:1987, NAV-D1/D2) |
| `CNavigator::CheckFailedEdge`/`CheckAllFailedEdges` | :2057-2168 | host trace/PVS; `svs.time`+`Q_irand` stamp via `sv_time`/`irand` (:2137, NAV-D1/D2); `#if 0` NAVNEW branch not taken (D-4) |
| `CNavigator::RouteBlocked` | :2170-2253 | rank-guided walk; `while(1)` loop |
| `CNavigator::GetBestNodeAltRoute` (both overloads) | :2261-2370 | 3-arg delegates to 4-arg; `d_altRoutes` via `cvar_integer` (:2278/2323/2346, NAV-D1) |
| `CNavigator::GetBestNode`/`GetNodePosition`/`GetNodeNumEdges`/`GetNodeEdge`/`Connected`/`GetPathCost`/`GetProjectedNode` | :2377-2686 | pure graph queries (golden surface); `Q3_INFINITE`/`WORLD_SIZE`/`WAYPOINT_NONE` + vec3 primitives imported from `mp_qshared` (NAV-D3) |
| `CNavigator::FlagAllNodes`/`GetChar`/`GetInt`/`GetFloat`/`GetLong`/`GetNumNodes` | :496-564,navigator.h:184 | helpers; `Get*` read via host `FS_Read` |
| `NodeTotalGreater::operator()` | :2693-2699 | the `first.cost > second.cost` comparator for the faithful heap sift (NAV-D4) |
| `CPriorityQueue::~/Find/Pop/Push/Update/Empty` | :2705-2782 | owned `Vec<Edge>` with hand-transcribed `push_heap`/`pop_heap` (NAV-D4/D-7); `Find`/`Update` have no live caller — §20 |
| `NAV_CvarInit`/`NAV_Free` | :39-48 | the `Cvar_Get` **registration** of `d_altRoutes`/`d_patched` (:41-42) has **no frozen-trait counterpart** → elided/no-op (Seam cvar note); only the `->integer` read-back ports, via `EngineHost::cvar_integer` (NAV-D1); `NAV_Free`→`Navigator::free` |
| `GetTime` (`#if AI_TIMERS`) | :59-74 | not ported (`AI_TIMERS` off) — §20 |
| `CNavigator::GetNodeLeadDistance` | navigator.h:182 | declared-only, **no definition** in navigator.cpp and no caller/trap arm — dropped as dead surface (§20 zero-caller note), not stubbed |
| `GNavCallback_*` ×9 | gameCallbacks.cpp:6-49 | `EngineHost::vm_call(VmSlot::Gvm, GAME_NAV_*)` (NAV-D1/D4); the four ent-taking callbacks deref `self->s.number` and pass **that int** (not the pointer) widened to an `isize` slot. Exact nine free-fn signatures frozen in the Seam Outbound block |

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
  hand-transcribed (NAV-D4). Ownership/layout is free (§F); pop order under
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
    summary: Nav module root — NF_*/EFLAG_* flags, NODE_NONE, NAV/NODE header IDs, MAX_FAILED_EDGES, WP_MINS/WP_MAXS, CHECKED_* consts (all navigator.h, nav-owned); re-exports. Q3_INFINITE/WORLD_SIZE/STEPSIZE/WAYPOINT_NONE and the vec3 primitives are NOT defined here — imported from mp_qshared (NAV-D3/RULING 22/39d), never local copies. Navigator becomes the Engine.nav field (RULING 12).
  - path: crates/mp/engine/server/src/npcnav/edge.rs
    crate: mp_engine_server
    mode: mp
    class: CEdge
    summary: Edge {first,second,cost} generic triple used by the priority queue (D-1 dead 0-arg ctor).
  - path: crates/mp/engine/server/src/npcnav/node.rs
    crate: mp_engine_server
    mode: mp
    class: CNode
    summary: Node — position/flags/radius/id, edges Vec<NodeEdge>, ranks Vec<i32>; Save/Load take a SHARED in-memory cursor/byte-buffer param (the analogue of the oracle's shared fileHandle_t; Raven Save(numNodes,file)/Load(numNodes,file), navigator.cpp:385/426) — NOT &mut impl EngineHost. Node methods never call the host; the single whole-file fs_read_file/fs_write_file (NAV-D1) lives only at Navigator::load/save, which loops these node methods over the one cursor. (A per-node fs_write_file would whole-file-overwrite — no append mode — and clobber all but the last node.) Plus accessors, edge queries. NodeEdge (Raven CNode-nested edge_t {ID,cost,flags}, navigator.h:72-77) is defined HERE alongside Node as CNode's private member type (porting-rules §21 colocation) — NOT in edge.rs, which is CEdge/Edge (the priority-queue triple). GetPosition uses the mp_qshared vec3 primitives (_-prefixed names, NAV-D3).
  - path: crates/mp/engine/server/src/npcnav/navigator.rs
    crate: mp_engine_server
    mode: mp
    class: CNavigator
    summary: CNavigator — node/edge arenas, failed-edge/checked-node bookkeeping, priority-queue pathfinding, Load/Save (first slice, NAV-D4), the G_NAV_* pub surface; host-taking methods take (&mut self, &mut impl EngineHost) (NAV-D1); the five ent-taking arms take *mut sharedEntity_t (NAV-D4); cvar reads via cvar_integer, svs.time via sv_time, Save via fs_write_file, timer jitter via host.irand (NAV-D1/D2).
  - path: crates/mp/engine/server/src/npcnav/priority_queue.rs
    crate: mp_engine_server
    mode: mp
    class: CPriorityQueue
    summary: Faithful Vec<Edge> min-heap on cost — hand-transcribed push_heap/pop_heap under NodeTotalGreater so equal-cost tie order matches the oracle-harness libstdc++ (NAV-D4/D-7); NOT std::BinaryHeap; Find/Update dropped as zero-caller.
  - path: crates/mp/engine/server/src/npcnav/callbacks.rs
    crate: mp_engine_server
    mode: mp
    class: (GNavCallback free fns)
    summary: The nine GNavCallback_* outbound calls as EngineHost vm_call(VmSlot::Gvm, GAME_NAV_*) shims (gameCallbacks.cpp); NAV_CvarInit/NAV_Free (cvar read-back via cvar_integer, NAV-D1).
  - path: crates/mp/qshared/src/shared/q_math.rs
    crate: mp_qshared
    mode: mp
    class: (q_math.c vec3 fns)
    summary: NAV-D3 migration (NEW file, sibling of q_math_rand.rs, mirrors oracle/codemp/game/q_math.c) — the vec3 fns MOVED here from crates/mp/game/src/q_math.rs KEEPING Raven's _-prefixed names (VectorNormalize bare at :916; _DotProduct at :961, _VectorSubtract at :968, _VectorCopy at :986 — RULING 39d, no rename). The mp_game copies are deleted and re-imported in the SAME commit with no shims across the full cross-crate footprint (_DotProduct 17 files, _VectorSubtract 34, _VectorCopy 43, VectorNormalize 45; RULING 39d). The single engine-reachable definition the referee compares.
  - path: crates/mp/qshared/src/common/mp/game/q3_infinite.rs
    crate: mp_qshared
    mode: mp
    class: (g_public.h const)
    summary: NAV-D3 migration — Q3_INFINITE (g_public.h:9) MOVED into the game-mirroring folder (one-const-per-file convention, snake_case leaf); the mp_game copy (g_public_consts.rs:14) deleted and re-imported in the same commit, no shims, across its 15-file footprint (RULING 39d).
  - path: crates/mp/qshared/src/common/mp/game/waypoint_none.rs
    crate: mp_qshared
    mode: mp
    class: (g_nav.h const)
    summary: NAV-D3 migration — WAYPOINT_NONE (g_nav.h:7) MOVED into the game-mirroring folder (one-const-per-file convention, snake_case leaf); the mp_game copy (g_nav_consts.rs:13) deleted and re-imported in the same commit, no shims, across its 13-file footprint (RULING 39d).
  - path: crates/mp/qshared/src/common/mp/bg/stepsize.rs
    crate: mp_qshared
    mode: mp
    class: (bg_public.h const)
    summary: NAV-D3 migration — STEPSIZE (bg_public.h:22, 18.0) MOVED into the bg-mirroring folder; the mp_game copy (bg_slidemove.rs:37) deleted and re-imported in the same commit, no shims, across its 5-file footprint (RULING 39d). Consumed by WP_MINS/WP_MAXS.
  - path: crates/mp/qshared/src/shared/world_size.rs
    crate: mp_qshared
    mode: mp
    class: (q_shared.h const)
    summary: NAV-D3 migration — WORLD_SIZE (q_shared.h:20, 131072.0) MOVED into the shared tier; the mp_game copy (NPC_combat.rs:2736) deleted and re-imported in the same commit, no shims, across its 2-file footprint (RULING 39d).
```

## Open questions

MUST be empty at FROZEN. Former open questions NAV-Q1–Q13 are all resolved
— Q1–Q5 by the §F doc-session rulings, Q6 by RULING 22, Q7 by RULING 32 (NAV-D4),
Q8 by the round-4 mechanical resolution (NAV-D3), **Q9/Q10/Q11 by RULING 36
(NAV-D1)**, and **Q12/Q13 by RULING 39d (NAV-D3)** — retained here as
resolved-in-place notes for cross-doc ID stability (never re-litigate). Every
**npcnav seam point** is settled; **two live holes** remain — **NAV-Q14** (how the
binary `.nav` 3a fixtures are generated, a verification-mechanism question) and
**NAV-Q15** (the concrete Rust shape of the cross-cutting `Engine` split-borrow
view struct — RULING 11 pins the pattern, not the shape) — both surfaced by the
dry-run and both requiring design sessions; the doc stays **DRAFT** until they are
settled and the review + dry-run gates pass.

- **NAV-Q14** *(LIVE — escalates to a design session; do NOT self-resolve)* —
  **What mechanism generates the committed binary `.nav` 3a golden fixtures?** The
  Verification strategy calls the public corpus "committed hand-authored minimal
  nav graphs," but `.nav` is a **binary** layout (`NAV_HEADER_ID` magic +
  checksum + per-node header/position/flags/id/radius/edge-vector/rank-array +
  the fixed `failedEdges[32]` array — navigator.cpp:602-702, :385-470) with **no
  human-writable source form**, so "hand-authored" cannot be taken literally. The
  sibling ICARUS doc names a dedicated generator for exactly this class (ruling
  14, `tools/ibi-gen` compiling hand-authored `.IBI` scripts); npcnav names no
  equivalent (`tools/npcnav-oracle/` does not yet exist in the tree), and at least
  three mechanisms are consistent with the settled rulings without one being
  pinned: (a) bootstrap by building a small graph programmatically and dumping it
  through the newly-ported `Navigator::save` against a `MockHost`, committing the
  captured bytes (consistent with `Save` being deliberately in-scope for the first
  slice, NAV-Q11 — but the doc never states this is `Save`'s purpose); (b) a
  throwaway generator/byte-emitter script (the ICARUS `tools/ibi-gen` analogue,
  e.g. a `tools/npcnav-oracle` helper); (c) raw byte-literal fixtures authored by
  hand. RULING 14 pins the fixture *corpus policy* (committed hand-authored public
  set + uncommitted retail set) but **not** the byte-generation *mechanism*, and
  the choice affects what the first slice must build and commit — so it is a
  design decision, escalated, not resolved here.

- **NAV-Q15** *(LIVE — cross-cutting Stage-0 design gap; does NOT block npcnav's
  first slice; escalates to a design session; do NOT self-resolve)* — **What is
  the concrete Rust shape of the `Engine` split-borrow view struct that lets
  `Engine` implement `EngineHost` while excluding the `nav` field?** RULING 11
  (`docs/handoffs/engine-fork-discovery.md`) settled the *approach* — `Engine`
  supplies the `EngineHost` impl through a split-borrow view struct that excludes
  `nav`, so `engine.nav.method(&mut view, …)` can borrow `nav` and the rest of
  `Engine` disjointly (NAV-D1, NAV-Q5) — but the *shape* (the view struct's type
  name, home module, field list, and how the disjoint mutable borrow is expressed
  in code) is defined in **none** of the five §F docs, RULING 11, or the tree:
  `crates/mp/engine` and `crates/mp/host-interface` carry no `view struct`/
  `EngineView`/`ExcludingNav` definition (only a one-line pattern mention,
  `crates/mp/host-interface/src/lib.rs:13`), and `mp_engine_core::Engine`
  (`crates/mp/engine/core/src/engine.rs:20-37`) has no `nav` field yet (its comment
  leaves the §F attachment point open, STATE-Q2). This is a **cross-cutting Stage-0
  concern shared by all five §F docs**, not nav-specific. It does **not** block
  npcnav's first slice — `load`/`save`/`calculate_path`/the pure queries take
  `&mut impl EngineHost` generically and are exercised directly against `MockHost`
  (NAV-D4 / RULING 32), never through `Engine`'s own trait impl — but it **does**
  block wiring the `G_NAV_*` arms into the real `SV_GameSystemCalls` (Slice hooks,
  wave 20). RULING 11 pins the pattern, not the shape, so the shape is a design
  decision — escalated to the shared Stage-0 view-struct session, not resolved here.

- **NAV-Q1** — *(Resolved: NAV-D1 / RULING 11/24/31/33/36.)* Host-threading
  mechanism for the trace/FS/cvar/time/callback services = the one shared Stage-0
  `EngineHost` trait, BUILT + EXTENDED and green (`crates/mp/host-interface`,
  commit `a9820853`); every host-taking method takes
  `(&mut self, host: &mut impl EngineHost)`.
- **NAV-Q2** — *(Resolved: NAV-D4 / RULING 14 pattern.)* Fixtures = committed
  hand-authored minimal nav graphs + an uncommitted local retail `.nav` corpus.
  The exact per-fixture probe list is a mechanical Verification-plan detail the
  harness enumerates, not a design point.
- **NAV-Q3** — *(Resolved: NAV-D4 / EVIDENCE.)* The SETCHECKEDNODE/FLAGALLNODES
  switch fall-through is owned by the wave-20 `SV_GameSystemCalls` port; no
  `CNavigator` artifact is responsible for it.
- **NAV-Q4** — *(Resolved: NAV-D1 / RULING 11.)* There is no nav-private host
  trait; the required services are methods on the shared `EngineHost` trait,
  quoted verbatim in the Seam from the built crate. Their Rust signatures live
  with the `EngineHost` design.
- **NAV-Q5** — *(Resolved *approach*: NAV-D1 / RULING 11; concrete shape → NAV-Q15.)*
  The `Navigator`-vs-rest self-borrow is resolved *in principle* by `Engine`'s
  split-borrow view struct that excludes `nav`, so `engine.nav.method(&mut view, …)`
  borrows disjointly. RULING 11 settles the pattern; the view struct's concrete
  Rust shape (type name, home module, field list) is the still-open cross-cutting
  NAV-Q15.
- **NAV-Q6** — *(Resolved: NAV-D3 / RULING 22.)* Canonical engine-reachable home
  for the shared constants/helpers the nav code consumes but does not own =
  `mp_qshared`, one definition the referee compares, no duplication.
- **NAV-Q7** — *(Resolved: NAV-D4 / RULING 32, 2026-07-09.)* How the first slice
  populates `Navigator{nodes, edges}` for its 3a goldens: **through the front
  door** — `Load` ports in the first slice with its real frozen signature and the
  fixture-backed `MockHost` (`crates/mp/host-interface/src/mock.rs`) serves the
  `.nav` bytes via `fs_read_file`; `CalculatePath` joins behind it. No test-only
  constructor is added.
- **NAV-Q8** — *(Resolved: NAV-D3 / round-4 mechanical resolution, 2026-07-09.)*
  The RULING-22 migration mechanics: the four consts and four vec3 primitives
  **MOVE** into `mp_qshared` (vec3 fns → new `crates/mp/qshared/src/shared/q_math.rs`;
  each const → the folder mirroring its owning Raven header), the `mp_game` copies
  **deleted and re-imported in the same commit, no re-export shims**, and the
  migration is **in-scope for this doc's first slice** and listed in the `files`
  roster. Owner (this slice), paths (pinned), and move-vs-re-export (move, no
  shims) settled.
- **NAV-Q9** — *(Resolved: NAV-D1 / RULING 36, 2026-07-09.)* **How does a nav
  method read `d_altRoutes`/`d_patched`?** Through `EngineHost::cvar_integer(name)`
  — the trait's new per-call integer cvar read (the `->integer` reads at
  navigator.cpp:480,1403,1418,1433,1498,1933,2278,2323,2346); an unregistered name
  reads 0, as `Cvar_VariableIntegerValue` does. RULING 36 put it on the one shared
  trait rather than storing resolved values on `Navigator`.
- **NAV-Q10** — *(Resolved: NAV-D1 / RULING 36, 2026-07-09.)* **How does a nav
  method read `svs.time`?** Through `EngineHost::sv_time()` — the trait's new
  server-frame-clock accessor (the recheck-timer reads at
  navigator.cpp:1733,1763,1778,1797,1987,2010,2065,2137). It is the
  `serverStatic_t` frame clock, **not** `PlatformHost::milliseconds`
  (`Sys_Milliseconds`, a different clock nav never receives).
- **NAV-Q11** — *(Resolved: NAV-D1 / RULING 36, 2026-07-09.)* **How is `Save`
  written?** Through `EngineHost::fs_write_file(qpath, data)` — the trait's new
  whole-file write that collapses `CNavigator::Save`/`CNode::Save`'s
  `FS_FOpenFileByMode(...,FS_WRITE)` + `FS_Write` + `FS_FCloseFile`
  (navigator.cpp:670,678,681,686,697,699); `false` mirrors the NULL-handle open
  failure. `Save` **stays in the first slice** and is fully portable against the
  frozen trait (no stub).
- **NAV-Q12** — *(Resolved: NAV-D3 / RULING 39d, 2026-07-09.)* **Does NAV-D3's
  "delete the `mp_game` copies in the SAME commit, no re-export shims" intend the
  full cross-crate call-site edit?** **Yes** — it is the full cross-crate edit at
  the tree-verified footprint (`_DotProduct` 17 files, `_VectorSubtract` 34,
  `_VectorCopy` 43, `VectorNormalize` 45, `Q3_INFINITE` 15, `WAYPOINT_NONE` 13,
  `STEPSIZE` 5, `WORLD_SIZE` 2); the migration commit edits every one to import
  from `mp_qshared`. The narrow "leave `mp_game` untouched" reading is forbidden by
  the no-shims/same-commit text.
- **NAV-Q13** — *(Resolved: NAV-D3 / RULING 39d, 2026-07-09.)* **What are the
  moved vec3 functions named in `mp_qshared`?** They **keep Raven's underscore-prefixed
  names** exactly as `mp_game`'s `q_math.rs` has them: `_DotProduct`
  (`crates/mp/game/src/q_math.rs:961`), `_VectorSubtract` (:968), `_VectorCopy`
  (:986); `VectorNormalize` stays bare (:916). No rename — the bare
  `DotProduct`/`VectorSubtract`/`VectorCopy` remain Raven `#define` macros over the
  `_`-prefixed fns.
