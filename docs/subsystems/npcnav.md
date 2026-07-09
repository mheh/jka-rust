# CNavigator (server/NPCNav) Design
Status: DRAFT     Supersedes: none
Decision prefix: NAV     Ledger deps: engine-fork-discovery fork-2 (global placement), fork-3 (fn-scope statics), fork-7 (§F doc list)

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
- `docs/handoffs/engine-fork-discovery.md` — settled fork rulings (fork-2/3/7).
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
  pub surface those arms call; NAV-D4 keeps that boundary byte-identical.
- The game-module twin of this API (the `trap_Nav_*` wrappers and the
  `GAME_NAV_*` handlers `oracle/codemp/game/g_public.h:788-796`) is already
  ported in `mp_game` — see NAV-D4. This doc does not re-port it.
- The `Sys_*`/FS/trace/cvar engine services `CNavigator` calls back into
  (`SV_Trace`, `SV_inPVS`, `SV_GentityNum`, `FS_*`, `Cvar_Get`, `Com_Error`,
  `Com_Printf`) are ported by their own subsystems in the wave order; this doc
  only records that they are the nav seam's inbound dependencies.
- The exact host-threading mechanism for those callbacks and the exact golden
  fixture set are **not settled by the inputs** — see NAV-Q1, NAV-Q2.

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
rest (:2705-2711).

### The failed-edge / checked-node bookkeeping

- Per-entity failed **nodes** live on `sharedEntity_t`
  (`oracle/codemp/game/g_public.h:706-712`: `waypoint`, `failedWaypoints[8]`
  (`MAX_FAILED_NODES = 8`, g_public.h:673), `failedWaypointCheckTime`), written
  by `AddFailedNode` (:1768-1799) / re-tested by `CheckFailedNodes`
  (:1724-1766) / read by `NodeFailed` (:1801-1811). These fields are in the
  **game-owned** entity array the engine sees through `SV_GentityNum`.
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

Per engine-fork-discovery **fork-2** (file-scope globals → fields on the owning
subsystem struct under `Engine`, grouped by owning `.c` file; cvar *handles* in
an `EngineCvars` sub-struct; no `static mut`) and **fork-3** (fn-scope statics:
const tables → `const`; genuine cross-frame state → host field).

| Raven global | oracle cite | Rust owner (crate::Type.field) | constructed by | threaded via |
| --- | --- | --- | --- | --- |
| `navigator` | navigator.cpp:32 | `mp_engine_server::Server.navigator: Navigator` | `Server::new` / `SV_Init` path | `&mut Server` in the syscall dispatcher; NAV-D2 |
| `Navigator.m_nodes` | navigator.h:247 | `Navigator.nodes: Vec<Node>` | `AddRawPoint`/`Load` | owned arena, node id = index; NAV-D1 |
| `Navigator.m_edgeLookupMap` | navigator.h:248 | `Navigator.edge_lookup: BTreeMap<i32, Vec<usize>>` | `AddFailedEdge`/`Load` | owned; NAV-D1/D3 |
| `Navigator.failedEdges[32]` | navigator.h:245 | `Navigator.failed_edges: [FailedEdge; MAX_FAILED_EDGES]` | ctor/`ClearAllFailedEdges` | owned array |
| `Navigator.pathsCalculated` | navigator.h:215 | `Navigator.paths_calculated: qboolean` | `CalculatePaths` | pub field (NAV-D4 seam get/set) |
| `d_altRoutes`, `d_patched` | navigator.cpp:36-37 | `mp_engine_server::EngineCvars` (nav cvar handles) | `NAV_CvarInit` | handle read at method entry; fork-2 |
| `CheckedNodes` static | navigator.cpp:1687 | `Navigator.checked_nodes: BTreeMap<i32, u8>` | first `SetCheckedNode` | owned; fork-3 kind-3. **`BTreeMap` not `HashMap`** — iteration/lookup determinism (plan §3d) |
| `wpMaxs`/`wpMins` | navigator.cpp:50-51 | module `const WP_MAXS/WP_MINS: [f32;3]` | — | fork-3 kind-1 |
| `CHECKED_NO/FAILED/PASSED` | navigator.cpp:54-56 | module `const` (`u8`) | — | fork-3 kind-1 |
| `GetTime` statics | navigator.cpp:63-64 | not ported (`AI_TIMERS` off) | — | §20 dead-surface note |

`sharedEntity_t.{waypoint, failedWaypoints, failedWaypointCheckTime}`
(g_public.h:706-712) are **not** engine-owned — they live in the game module's
entity array reached through `SV_GentityNum`; the nav methods read/write them
through that borrow, exactly as Raven does through the raw `sharedEntity_t*`.

## Seam definition

Two seam directions, both preserved exactly (NAV-D4):

### Inbound: game → engine (the `G_NAV_*` arms)

The `SV_GameSystemCalls` switch dispatches 42 `G_NAV_*` arms
(`oracle/codemp/game/g_public.h:298-339`) at
`oracle/codemp/server/sv_game.cpp:837-936` — 40 are `CNavigator` **method**
calls (the plan's "39 direct callees" figure, §0.4; two arms are the same
overloaded `GetBestNodeAltRoute`), and two (`G_NAV_GETPATHSCALCULATED`/
`G_NAV_SETPATHSCALCULATED`) read/write the public `pathsCalculated` **field**.
Args arrive as `intptr_t` slots; pointer args use `VMA(n)` (shared-memory base
offset). The pub Rust surface these arms need (host-threading param is
NAV-Q1; shown here as `&mut self, host: &mut impl NavHost` — the C signatures
are load-bearing, the receiver is not yet frozen):

```rust
// Lifecycle / build
fn init(&mut self);                                             // G_NAV_INIT
fn free(&mut self);                                             // G_NAV_FREE
fn load(&mut self, host: &mut impl NavHost, filename: &str, checksum: i32) -> bool;   // G_NAV_LOAD
fn save(&mut self, host: &mut impl NavHost, filename: &str, checksum: i32) -> bool;   // G_NAV_SAVE
fn add_raw_point(&mut self, host: &mut impl NavHost, point: [f32;3], flags: i32, radius: i32) -> i32; // G_NAV_ADDRAWPOINT
fn calculate_paths(&mut self, host: &mut impl NavHost, recalc: qboolean);             // G_NAV_CALCULATEPATHS
fn hard_connect(&mut self, host: &mut impl NavHost, first: i32, second: i32);          // G_NAV_HARDCONNECT
fn show_nodes(&mut self, host: &mut impl NavHost);             // G_NAV_SHOWNODES
fn show_edges(&mut self, host: &mut impl NavHost);             // G_NAV_SHOWEDGES
fn show_path(&mut self, start: i32, end: i32);                 // G_NAV_SHOWPATH
// Queries
fn get_nearest_node(&mut self, host: &mut impl NavHost, ent: EntityId, last_id: i32, flags: i32, target_id: i32) -> i32; // G_NAV_GETNEARESTNODE
fn get_best_node(&mut self, start_id: i32, end_id: i32, reject_id: i32) -> i32;        // G_NAV_GETBESTNODE
fn get_node_position(&self, node_id: i32, out: &mut [f32;3]) -> i32;                   // G_NAV_GETNODEPOSITION
fn get_node_num_edges(&self, node_id: i32) -> i32;            // G_NAV_GETNODENUMEDGES
fn get_node_edge(&self, node_id: i32, edge: i32) -> i32;     // G_NAV_GETNODEEDGE
fn get_num_nodes(&self) -> i32;                              // G_NAV_GETNUMNODES
fn connected(&self, start_id: i32, end_id: i32) -> bool;    // G_NAV_CONNECTED
fn get_path_cost(&self, start_id: i32, end_id: i32) -> u32; // G_NAV_GETPATHCOST
fn get_edge_cost(&mut self, host: &mut impl NavHost, start_id: i32, end_id: i32) -> u32; // G_NAV_GETEDGECOST
fn get_projected_node(&self, origin: [f32;3], node_id: i32) -> i32;                    // G_NAV_GETPROJECTEDNODE
fn get_node_radius(&self, node_id: i32) -> i32;             // G_NAV_GETNODERADIUS
// Failed-node bookkeeping (writes into the game entity via host)
fn check_failed_nodes(&mut self, host: &mut impl NavHost, ent: EntityId);              // G_NAV_CHECKFAILEDNODES
fn add_failed_node(&mut self, host: &mut impl NavHost, ent: EntityId, node_id: i32);   // G_NAV_ADDFAILEDNODE
fn node_failed(&self, host: &impl NavHost, ent: EntityId, node_id: i32) -> qboolean;   // G_NAV_NODEFAILED
fn nodes_are_neighbors(&self, start_id: i32, end_id: i32) -> qboolean;                 // G_NAV_NODESARENEIGHBORS
// Failed-edge bookkeeping (failedEdge_t crosses by pointer via VMA)
fn clear_failed_edge(&mut self, host: &mut impl NavHost, e: &mut FailedEdge);          // G_NAV_CLEARFAILEDEDGE
fn clear_all_failed_edges(&mut self);                        // G_NAV_CLEARALLFAILEDEDGES
fn edge_failed(&self, start_id: i32, end_id: i32) -> i32;   // G_NAV_EDGEFAILED
fn add_failed_edge(&mut self, ent_id: i32, start_id: i32, end_id: i32);               // G_NAV_ADDFAILEDEDGE
fn check_failed_edge(&mut self, host: &mut impl NavHost, e: &mut FailedEdge) -> qboolean; // G_NAV_CHECKFAILEDEDGE
fn check_all_failed_edges(&mut self, host: &mut impl NavHost);                         // G_NAV_CHECKALLFAILEDEDGES
fn route_blocked(&self, start_id: i32, test_edge_id: i32, end_id: i32, reject_rank: i32) -> qboolean; // G_NAV_ROUTEBLOCKED
fn get_best_node_alt_route(&mut self, host: &mut impl NavHost, start_id: i32, end_id: i32, path_cost: &mut i32, reject_id: i32) -> i32; // G_NAV_GETBESTNODEALTROUTE
fn get_best_node_alt_route2(&mut self, host: &mut impl NavHost, start_id: i32, end_id: i32, reject_id: i32) -> i32; // G_NAV_GETBESTNODEALT2 (overload)
fn get_best_path_between_ents(&mut self, host: &mut impl NavHost, ent: EntityId, goal: EntityId, flags: i32) -> i32; // G_NAV_GETBESTPATHBETWEENENTS
fn check_blocked_edges(&mut self, host: &mut impl NavHost);  // G_NAV_CHECKBLOCKEDEDGES
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
property of the `SV_GameSystemCalls` switch, **not** of any `CNavigator` method,
so under §20 its preservation obligation travels with that switch: the wave-20
`SV_GameSystemCalls` port owns asserting it (per the Non-goals scope boundary
above and build-out plan §0.4, which put that 1,200-LOC function at wave 20),
and NAV-D4 holds that boundary byte-identical. Recorded here only so the wave-20
porter preserves it — this doc's `CNavigator` surface neither emits nor asserts
the fall-through, so no `CNavigator` port artifact is responsible for it.

### Outbound: engine → game (`gameCallbacks.cpp`) and engine services

`CNavigator` reaches back into the game module and the rest of the engine.
These are the `NavHost` seam the methods above take:

- **Nine game out-calls** (`oracle/codemp/server/NPCNav/gameCallbacks.cpp:6-49`),
  each a thin `VM_Call(gvm, GAME_NAV_*, ...)` (`GAME_NAV_*` enum
  `g_public.h:788-796`; already handled in `mp_game`, NAV-D4):
  `NAV_ClearPathToPoint`, `NPC_ClearLOS`, `NAVNEW_ClearPathBetweenPoints`,
  `NAV_CheckNodeFailedForEnt`, `G_EntIsUnlockedDoor`, `G_EntIsDoor`,
  `G_EntIsBreakable`, `G_EntIsRemovableUsable`, `CP_FindCombatPointWaypoints`.
  The `intptr_t`-slot widening for pointer args is mandatory (plan §5.4 — the
  historical `GAME_NAV_CLEARPATHTOPOINT` truncation bug).
- **Engine services** (dependencies ported by their own subsystems): `SV_Trace`
  (`server/server.h:416`), `SV_inPVS` (server.h:356), `SV_GentityNum`
  (server.h:349), `FS_Read`/`FS_Write`/`FS_FOpenFileByMode`/`FS_FCloseFile`,
  `Cvar_Get`, `Com_Error`(ERR_DROP), `Com_Printf`, `va`, `Q_irand`, and
  `svs.time`.

### `#[repr(C)]` types touched

`failedEdge_t` (g_public.h:52-58) crosses the seam by pointer (`VMA` →
`&mut FailedEdge`); it is a **shared** struct (game + engine) and keeps exact
layout — imported from the ported type, never re-declared (fork type-rosetta
rule). `sharedEntity_t` (g_public.h:676-715) is reached through `SV_GentityNum`;
`trace_t`, `vec3_t`, `cvar_t` likewise imported.

## Decisions

**NAV-D1** — The node/edge graph is owned `Vec` arenas indexed by ids, not a
pointer graph. `m_nodes: vector<CNode*>` → `nodes: Vec<Node>` with node id ==
index (matches `AddRawPoint`'s `m_nodes.size()` id assignment,
navigator.cpp:712); `CNode.m_edges` → `Vec<NodeEdge>`; `m_ranks` (heap `int*`)
→ `Vec<i32>`; `m_edgeLookupMap` → `BTreeMap<i32, Vec<usize>>`. Because the graph
is walked by id everywhere (`m_nodes[id]`) and §B5 forbids aliasing raw
pointers in safe code. Rejected keeping `Box<Node>` + raw pointers because
nothing needs identity beyond the index and it would drag `unsafe` out of the
seam.

**NAV-D2** — `CNavigator` state lives on an `Engine` sub-struct (fork-2): a
`Navigator` field on the server host (`mp_engine_server::Server.navigator`),
constructed on the server-init path, threaded as `&mut` through the syscall
dispatcher. Because the single Raven `navigator` global is engine-owned server
state (navigator.cpp:32-34). Rejected a `static`/`OnceCell` singleton because
fork-2 bans `static mut`/hidden globals.

**NAV-D3** — STL members become std containers; parity is proven over goldens,
not container identity (§17). `CPriorityQueue`'s `vector<CEdge*>` +
`push_heap`/`pop_heap`/`NodeTotalGreater` → `BinaryHeap<Edge>` (min-heap on
`cost`; use `Reverse`/custom `Ord`); `m_edgeLookupMap` `multimap<int,int>` →
`BTreeMap<i32, Vec<usize>>` (per-key insertion order preserved so
`EdgeFailed`'s `equal_range` first-match, :1876-1898, is reproduced);
`ShowEdges`'s `map<int,bool>` and `CheckedNodes`'s `map<int,byte>` → `BTreeMap`.
Because §F frees internal layout and the observable is the path/query result,
not the heap's internal array order. Rejected transcribing `push_heap`/`pop_heap`
over a `Vec` verbatim — idiomatic `BinaryHeap` gives the same pop order under
the same comparator.

**NAV-D4** — The game-module nav API boundary is kept exactly as the syscall
switch presents it. The 42 `G_NAV_*` inbound arms (g_public.h:298-339,
sv_game.cpp:837-936) and the nine `GAME_NAV_*` outbound calls (g_public.h:788-796,
gameCallbacks.cpp) keep their numbers, arg order, `VMA` pointer marshaling, and
`intptr_t`-slot widening unchanged; the `GAME_NAV_*` handlers already live in
`mp_game` and are not re-ported. Because this is the frozen ABI seam between the
already-ported game module and this engine subsystem (§D). Rejected renumbering
or collapsing the two `GetBestNodeAltRoute` overloads — the game module issues
both arm numbers.

## Verification strategy

C++ track → porting-rules §F / §18: differential goldens from the unmodified
oracle TU, committed so `cargo test` needs no C++ toolchain. Harness home:
`tools/npcnav-oracle/` (GP2 pattern — stub headers under it, oracle never
edited).

**Golden surface (3a, primary — "path-query goldens over retail nav data"):**
after `Load` of a committed retail `.nav` fixture, the pure-graph query surface
is fully deterministic with **no trace/PVS/callback** dependency — the ranks are
baked into the file. Dump-and-compare `GetBestNode`, `GetBestNodeAltRoute`,
`GetPathCost`, `GetEdgeCost(int,int)` [graph portion], `Connected`,
`NodesAreNeighbors`, `GetProjectedNode`, `GetNodeNumEdges`/`GetNodeEdge`/
`GetNodePosition`/`GetNodeRadius`/`GetNumNodes`, plus `CalculatePaths` →
`GetPathCost` (regenerate ranks and re-query). `NodeTotalGreater`/priority-queue
order is exercised transitively through `CalculatePath`'s rank output. The
oracle side stubs `FS_Read` against the fixture bytes and stubs
`Com_Printf`/`Cvar_Get` (`d_altRoutes`/`d_patched` forced to fixed values so
both `d_altRoutes` branches are covered).

**Trace/callback-dependent surface (3c, referee swap-in):** `GetNearestNode`,
`GetBestPathBetweenEnts`, `CheckBlockedEdges`, `HardConnect`, `GetEdgeCost`
(the `CNode*,CNode*` trace form), `CheckFailedNodes`, `CheckFailedEdge`,
`CheckAllFailedEdges` reach `SV_Trace`/`SV_inPVS`/`SV_GentityNum` and the nine
game callbacks, so they need live engine + game state. These verify under the
plan's §3c A/B referee (`crates/jampgame/tests/referee.rs` / the external
`sv_referee` rig) once the server spine is real, or via captured-trace replay
(§3b). The precise fixture map(s), the query probe list, and whether any
trace-dependent method is additionally covered by a stubbed-trace golden are
**NAV-Q2** (not settled by the inputs).

Governing clause: porting-rules §F (§18 differential goldens; §19 UB
divergence; §20 emergent-quirk preservation; §21 one class per file).

## Slice hooks

- Build-out plan §0.4 / wave 20: `SV_GameSystemCalls` — must have this doc's
  pub surface (Seam definition) frozen before its `G_NAV_*` arms are filled.
- Build-out plan wave 25 (server complete) / M4: the full nav subsystem must be
  green under the 3c referee swap-in.
- `NavHost` composition (NAV-Q1, with its dependent NAV-Q4 trait contents and
  NAV-Q5 self-borrow mechanics) must be settled before the first porter opens
  `navigator.rs`/`callbacks.rs`, because it fixes every host-taking method
  receiver. The ~9 host-free pure-graph query methods (NAV-Q1) are unblocked and
  may be ported ahead of this ruling.
- **First slice (portable today; no NAV-Q1/Q4/Q5 needed).** Under the
  GOAL-engine no-stub / no-`todo!` discipline the only bodies a porter can
  legally produce before that session are the host-free ones, and they form a
  self-contained slice: the type skeletons — `mod.rs` consts (State-ownership
  table), `edge.rs` `Edge` (D-1), `node.rs` `Node`'s host-free members
  (accessors navigator.h:94-110, edge/rank queries, `Create`, `AddEdge`; `Save`/
  `Load` are **deferred**, they take `FS_*` via host — transcription table :385-470),
  and `priority_queue.rs`'s `BinaryHeap<Edge>` (NAV-D3/D-7) — plus the nine
  host-free `Navigator` queries enumerated in NAV-Q1 (`GetBestNode`,
  `GetNodePosition`, `GetNodeNumEdges`, `GetNodeEdge`, `GetNumNodes`,
  `Connected`, `GetPathCost`, `GetProjectedNode`, `GetNodeRadius`). Every other
  `Navigator` method and the whole of `callbacks.rs` are a **hard stop** here:
  the porter escalates to the NAV-Q1/Q4/Q5 session rather than guessing a
  receiver or stubbing a body (GOAL-engine). This first slice is what verifies
  against the 3a path-query goldens (Verification strategy); the NAV-Q2 fixture
  set is only needed to *run* those goldens, not to write these bodies.

## Method transcription table

81 functions (per plan §0.4); inline accessors fold into their owning struct's
impl. Grouped by Raven class; Rust shape per NAV-D1/D3.

| Raven method | oracle cite | Rust shape |
| --- | --- | --- |
| `CEdge::CEdge()` / `(int,int,int)` / `~CEdge` | :82-96 | `Edge { first, second, cost }`; 0-arg ctor is a Raven no-op (divergence D-1) |
| `CNode::CNode`/`~CNode`/`Create(...)`/`Create()` | :104-147 | `Node::new` / `Node::create(pos,flags,radius,id)`; `Vec`-owned (no `new`/`delete`) |
| `CNode::AddEdge` | :155-183 | dedup-or-push into `edges: Vec<NodeEdge>`; `assert(<9)` → `debug_assert!` (D-6) |
| `CNode::GetEdgeNumToNode`/`GetEdge`/`GetEdgeCost`/`GetEdgeFlags`/`SetEdgeFlags` | :191-344 | index/scan `edges`; keep `edgeNum > m_numEdges` bound verbatim (D-2) |
| `CNode::AddRank`/`InitRanks`/`GetRank` | :214-376 | `ranks: Vec<i32>` (`-1` fill) |
| `CNode::Draw` | :227-236 | empty (renderer stripped) — port as no-op with §20 note |
| `CNode::Save`/`Load` | :385-470 | `FS_*` via `NavHost`; `NODE_HEADER_ID` check |
| `CNode` inline accessors (`GetID`,`GetPosition`,`GetNumEdges`,`GetRadius`,`GetFlags`,`AddFlag`,`RemoveFlag`) | navigator.h:94-110 | trivial methods |
| `CNavigator::CNavigator`/`~CNavigator` | :478-488 | `Navigator::new`; ctor's lazy `NAV_CvarInit` → cvar handles resolved on host |
| `CNavigator::Init`/`Free` | :572-594 | clear `nodes`/`edge_lookup` |
| `CNavigator::Load`/`Save` | :602-702 | `FS_*` via host; rebuild `edge_lookup` |
| `CNavigator::AddRawPoint` | :710-726 | push `Node`; `Com_Error` branch dead (D-3) |
| `CNavigator::GetEdgeCost(CNode*,CNode*)` | :734-755 | `SV_Trace` via host |
| `CNavigator::SetEdgeCost`/`AddNodeEdges` | :757-806 | id-indexed; bidirectional add |
| `CNavigator::CalculatePath`/`CalculatePaths` | :814-908 | `BinaryHeap<Edge>` flood fill (D-7 raw-ptr ownership → owned values) |
| `CNavigator::ShowNodes`/`ShowEdges`/`ShowPath` | :916-1027,:1632-1685 | draw calls stripped (renderer); keep PVS/`Com_Printf` control flow, §20 notes |
| `CNavigator::GetNodeRadius`/`CheckBlockedEdges`/`HardConnect` | :1029-1140 | host trace + door/breakable callbacks |
| `CNavigator::TestNodePath`/`TestNodeLOS`/`TestBestFirst` | :1150-1237 | protected; host callbacks |
| `CNavigator::CollectNearestNodes` | :1249-1318 | `nodeChain_l` → `Vec`/`VecDeque` insert-sorted (NAV-D3) |
| `CNavigator::GetBestPathBetweenEnts`/`GetNearestNode` | :1320-1624 | host trace/PVS; writes `ent->waypoint` via host |
| `CNavigator::ClearCheckedNodes`/`CheckedNode`/`SetCheckedNode` | :1687-1719 | `checked_nodes: BTreeMap<i32,u8>` |
| `CNavigator::CheckFailedNodes`/`AddFailedNode`/`NodeFailed` | :1724-1811 | read/write `sharedEntity_t` via `SV_GentityNum` |
| `CNavigator::NodesAreNeighbors` | :1813-1833 | scan node edges |
| `CNavigator::ClearFailedEdge`/`ClearAllFailedEdges` | :1835-1874 | `failed_edges[..]`; `memset(WAYPOINT_NONE)` → explicit fill |
| `CNavigator::EdgeFailed`/`AddFailedEdge` | :1876-2055 | `edge_lookup` `equal_range` first-match (NAV-D3) |
| `CNavigator::CheckFailedEdge`/`CheckAllFailedEdges` | :2057-2168 | host trace/PVS; `#if 0` NAVNEW branch not taken (D-4) |
| `CNavigator::RouteBlocked` | :2170-2253 | rank-guided walk; `while(1)` loop |
| `CNavigator::GetBestNodeAltRoute` (both overloads) | :2261-2370 | 3-arg delegates to 4-arg |
| `CNavigator::GetBestNode`/`GetNodePosition`/`GetNodeNumEdges`/`GetNodeEdge`/`Connected`/`GetPathCost`/`GetEdgeCost(int,int)`/`GetProjectedNode` | :2377-2686 | pure graph queries (golden surface) |
| `CNavigator::FlagAllNodes`/`GetChar`/`GetInt`/`GetFloat`/`GetLong`/`GetNumNodes` | :496-564,navigator.h:184 | helpers; `Get*` read via host `FS_Read` |
| `NodeTotalGreater::operator()` | :2693-2699 | `Ord`/`cmp` for the `BinaryHeap` |
| `CPriorityQueue::~/Find/Pop/Push/Update/Empty` | :2705-2782 | subsumed by `BinaryHeap<Edge>` (D-7); `Find`/`Update` have no live caller — §20 |
| `NAV_CvarInit`/`NAV_Free` | :39-48 | host cvar registration / `Navigator::free` |
| `GetTime` (`#if AI_TIMERS`) | :59-74 | not ported (`AI_TIMERS` off) — §20 |
| `CNavigator::GetNodeLeadDistance` | navigator.h:182 | declared-only, **no definition** in navigator.cpp and no caller/trap arm — dropped as dead surface (§20 zero-caller note), not stubbed |
| `GNavCallback_*` ×9 | gameCallbacks.cpp:6-49 | `NavHost` methods = `VM_Call(GAME_NAV_*)` (NAV-D4) |

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
- **D-7 (idiomatic ownership):** `CPriorityQueue`'s raw `CEdge*` `new`/`delete`
  lifecycle (:818,838,865,869,2705-2758) becomes an owning `BinaryHeap<Edge>`
  of values (NAV-D3). Layout is free (§F); pop order under `NodeTotalGreater` is
  preserved. `Find`/`Update` (:2716-2774) have no live caller — drop with §20
  zero-caller notes.

## files

```
files:
  - path: crates/mp/engine/server/src/npcnav/mod.rs
    crate: mp_engine_server
    mode: mp
    class: (module)
    summary: Nav module root — NF_*/EFLAG_* flags, NODE_NONE, NAV/NODE header IDs, MAX_FAILED_EDGES, WP_MINS/WP_MAXS, CHECKED_* consts; re-exports.
  - path: crates/mp/engine/server/src/npcnav/edge.rs
    crate: mp_engine_server
    mode: mp
    class: CEdge
    summary: Edge {first,second,cost} generic triple used by the priority queue (D-1 dead 0-arg ctor).
  - path: crates/mp/engine/server/src/npcnav/node.rs
    crate: mp_engine_server
    mode: mp
    class: CNode
    summary: Node — position/flags/radius/id, edges Vec<NodeEdge>, ranks Vec<i32>; Save/Load, accessors, edge queries.
  - path: crates/mp/engine/server/src/npcnav/navigator.rs
    crate: mp_engine_server
    mode: mp
    class: CNavigator
    summary: CNavigator — node/edge arenas, failed-edge/checked-node bookkeeping, priority-queue pathfinding, Load/Save, the G_NAV_* pub surface.
  - path: crates/mp/engine/server/src/npcnav/priority_queue.rs
    crate: mp_engine_server
    mode: mp
    class: CPriorityQueue
    summary: BinaryHeap<Edge> min-heap on cost (NodeTotalGreater); subsumes CPriorityQueue (D-7); Find/Update dropped as zero-caller.
  - path: crates/mp/engine/server/src/npcnav/callbacks.rs
    crate: mp_engine_server
    mode: mp
    class: (GNavCallback free fns)
    summary: NavHost outbound calls — the nine GNavCallback_* VM_Call(GAME_NAV_*) shims (gameCallbacks.cpp); NAV_CvarInit/NAV_Free.
```

## Open questions

MUST be empty at FROZEN. Each escalates to an interactive session (not
self-resolved).

- **NAV-Q1** — The host-threading mechanism for the trace/FS/callback services
  (`SV_Trace`/`SV_inPVS`/`SV_GentityNum`/`FS_*`/the nine `GAME_NAV_*` out-calls/
  `svs.time`/cvars): a dedicated `NavHost` seam trait taken as `&mut impl
  NavHost` (shown provisionally in Seam definition), **or** nav methods as
  `impl Server` fns that borrow `self.navigator` + sibling engine fields
  directly. Not settled by NAV-D1..D4; the oracle uses a global + direct
  `extern` calls, so it cannot be resolved from ground truth. Fixes every
  method receiver — must be ruled before porting. Its resolution must
  simultaneously settle the trait's own contents (NAV-Q4) and the field-split /
  reborrow mechanics the chosen receiver forces (NAV-Q5); resolve all three in
  one session. The ~9 pure-graph query methods that take no host param
  (`GetBestNode`, `GetNodePosition`, `GetNodeNumEdges`, `GetNodeEdge`,
  `GetNumNodes`, `Connected`, `GetPathCost`, `GetProjectedNode`,
  `GetNodeRadius`) are unaffected and portable ahead of this ruling; every other
  method is blocked on it.
- **NAV-Q2** — Verification fixture specifics: which retail map `.nav`
  file(s) become the committed golden fixture, the exact query probe list, and
  whether any trace/callback-dependent method (`GetNearestNode`,
  `GetBestPathBetweenEnts`, `CheckBlockedEdges`, `HardConnect`,
  `GetEdgeCost` trace form, `CheckFailedNodes`, `CheckFailedEdge`) is covered by
  a stubbed-trace golden vs. left to the §3c referee swap-in. The inputs settle
  the *approach* (path-query goldens over retail nav data) but not the set.
- **NAV-Q3** — *(Resolved in place, not a design act — kept for ID stability.)*
  Ownership of the `G_NAV_SETCHECKEDNODE`/`G_NAV_FLAGALLNODES` switch
  fall-through (`oracle/codemp/server/sv_game.cpp:928-933`) is settled by the
  already-fixed scope, not a new decision: the fall-through lives in the
  `SV_GameSystemCalls` switch, which the Non-goals section punts to wave 20
  (build-out plan §0.4); §20 makes its preservation the wave-20 port's
  obligation and NAV-D4 keeps that boundary byte-identical. Answered in the
  Seam-definition switch-fallthrough note; no `CNavigator` port artifact is
  responsible for it. Escalates nothing.
- **NAV-Q4** — The `NavHost` seam trait's *own contents* are undesigned (and the
  type does not yet exist in the repo). The Seam definition names the required
  services only as Raven identifiers; their C signatures are the ground truth —
  `SV_Trace` (server.h:416), `SV_inPVS` (server.h:356), `SV_GentityNum`
  (server.h:349), `FS_Read`/`FS_Write`/`FS_FOpenFileByMode`/`FS_FCloseFile`,
  `Cvar_Get`, `Com_Error`(ERR_DROP), `Com_Printf`, `va`, `Q_irand`, `svs.time`,
  and the nine `GNavCallback_*` out-calls (gameCallbacks.cpp:6-49) — but
  translating them into a trait (the method set, each method's Rust
  argument/return types and borrow shape, the error/`Result` mapping, and the
  crate/module home for `callbacks.rs`) is a design act, not a transcription:
  it presupposes NAV-Q1 chose the `impl NavHost` form over methods-on-`impl
  Server`. No decision covers the trait's contents. `callbacks.rs` (files
  roster) cannot be written until this is settled — resolve it in the NAV-Q1
  session.
- **NAV-Q5** — Self-borrow mechanics at the nav call site, forced by NAV-D2 but
  not resolved by it. NAV-D2 makes `Navigator` a field on `Server`
  (`mp_engine_server::Server.navigator`). If NAV-Q1 resolves to `host: &mut
  impl NavHost` with `Server` implementing `NavHost` (the natural reading —
  `Server` is the one owner of `sv`/`svs`/the entity view), the call
  `self.navigator.get_nearest_node(/* host = */ self, ...)` double-borrows
  `Server`: once through the `.navigator` field access, once through the whole
  `NavHost`-implementing struct that contains that same field. Neither
  porting-rules §B nor the existing whole-struct reborrow (`type ServerGame =
  Server`, `crates/mp/engine/server/src/server_host.rs:118`, which reborrows the
  *entire* `Server`, never a split of `navigator` vs. the rest) covers this. The
  fix — a field-split/reborrow wrapper, a `NavHost` view that excludes
  `navigator`, or hosting the methods on `impl Server` rather than `impl
  Navigator` — is a design act bound to NAV-Q1's receiver choice; settle it in
  the same session.
