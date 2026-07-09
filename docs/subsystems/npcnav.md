# CNavigator (server/NPCNav) Design
Status: DRAFT     Supersedes: none
Decision prefix: NAV     Ledger deps: engine-fork-discovery rulings 11 (EngineHost seam), 12 (`Engine.nav` field), 14 (fixture pattern), 18 (faithful priority queue), 22 (shared const/vec3 home in `mp_qshared` — closes NAV-Q6); forks 2/3 (state placement, fn-scope statics), 7 (§F doc list)

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
  and the §F doc-session rulings 11–18 this revision renders.
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
  not this one — NAV-D3.
- The game-module twin of this API (the `trap_Nav_*` wrappers and the
  `GAME_NAV_*` handlers `oracle/codemp/game/g_public.h:788-796`) is already
  ported in `mp_game` — see NAV-D5. This doc does not re-port it.
- The `Sys_*`/FS/trace/cvar engine services `CNavigator` calls back into
  (`SV_Trace`, `SV_inPVS`, `SV_GentityNum`, `FS_*`, `Cvar_Get`, `Com_Error`,
  `Com_Printf`) are reached through the one shared `EngineHost` trait (NAV-D2,
  RULING 11); that trait is designed once at Stage-0 (interface crate), not here.
  This doc only records that they are the nav seam's inbound dependencies.
- The host seam mechanism and the golden fixture set are **settled** by NAV-D2
  and NAV-D4 (they were open at the prior draft; the §F doc-session rulings
  closed them).

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
rank table — the tie-break is parity-visible (NAV-D1).

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

Per engine-fork-discovery **RULING 12** (the five §F states — `icarus`, `nav`,
`g2`, `roff`, `rmg` — are plain `Default`-initialized **direct fields on
`Engine`**, no `Option`/`Box`/nesting; lazy-init timing modeled with Raven's own
init flags), **fork-2** (other file-scope globals → fields on the owning
subsystem struct; cvar *handles* in an `EngineCvars` sub-struct; no `static
mut`), and **fork-3** (fn-scope statics: const tables → `const`; genuine
cross-frame state → host field).

| Raven global | oracle cite | Rust owner (crate::Type.field) | constructed by | threaded via |
| --- | --- | --- | --- | --- |
| `navigator` | navigator.cpp:32 | `mp_engine_core::Engine.nav: Navigator` (type in `mp_engine_server::npcnav`) | `Default`-init direct field; RULING 12 | `(&mut self, &mut impl EngineHost)`; NAV-D2 |
| `Navigator.m_nodes` | navigator.h:247 | `Navigator.nodes: Vec<Node>` | `AddRawPoint`/`Load` | owned arena, node id = index; NAV-D5 |
| `Navigator.m_edgeLookupMap` | navigator.h:248 | `Navigator.edge_lookup: BTreeMap<i32, Vec<usize>>` | `AddFailedEdge`/`Load` | owned; NAV-D5 |
| `Navigator.failedEdges[32]` | navigator.h:245 | `Navigator.failed_edges: [failedEdge_t; MAX_FAILED_EDGES]` | ctor/`ClearAllFailedEdges` | owned array |
| `Navigator.pathsCalculated` | navigator.h:215 | `Navigator.paths_calculated: qboolean` | `CalculatePaths` | pub field (NAV-D5 seam get/set) |
| `d_altRoutes`, `d_patched` | navigator.cpp:36-37 | engine cvar handles in `EngineCvars` (fork-2) | `NAV_CvarInit` | read via `EngineHost` at method entry; NAV-D2 |
| `CheckedNodes` static | navigator.cpp:1687 | `Navigator.checked_nodes: BTreeMap<i32, u8>` | first `SetCheckedNode` | owned; fork-3 kind-3. **`BTreeMap` not `HashMap`** — iteration/lookup determinism (plan §3d), NAV-D5 |
| `wpMaxs`/`wpMins` | navigator.cpp:50-51 | module `const WP_MAXS/WP_MINS: [f32;3]` | — | fork-3 kind-1; `WP_MINS`'s `-24+STEPSIZE` reads `STEPSIZE` from `mp_qshared` (NAV-D6) |
| `CHECKED_NO/FAILED/PASSED` | navigator.cpp:54-56 | module `const` (`u8`) | — | fork-3 kind-1 |
| `GetTime` statics | navigator.cpp:63-64 | not ported (`AI_TIMERS` off) | — | §20 dead-surface note |

`sharedEntity_t.{waypoint, failedWaypoints, failedWaypointCheckTime}`
(g_public.h:706-712) are **not** engine-owned — they live in the game module's
entity array reached through `SV_GentityNum` (an `EngineHost` service); the nav
methods read/write them through that borrow, exactly as Raven does through the
raw `sharedEntity_t*`.

**Shared constants & vec3 helpers the nav code consumes (not nav-owned).**
`Q3_INFINITE` (`oracle/codemp/game/g_public.h:9`, `16777216`), `WORLD_SIZE`
(`oracle/codemp/game/q_shared.h:20`), `STEPSIZE` (`oracle/codemp/game/bg_public.h:22`,
`18` — used by `WP_MINS`'s `-24+STEPSIZE`, navigator.cpp:51), `WAYPOINT_NONE`
(`oracle/codemp/game/g_nav.h:7`, `-1`), and the vec3 primitives
`VectorNormalize`/`DotProduct`/`VectorSubtract`/`VectorCopy`
(`q_shared.h`/`q_math.c`; used by `GetProjectedNode` and `CNode::GetPosition`)
are **not nav-owned** and are **not** re-declared in npcnav. They are shared
items imported from the engine-reachable shared tier — the same status as the
`va`/`Q_irand` q_shared helpers (Seam, outbound) and `failedEdge_t` (repr
section): imported, never a local copy (porting-rules §14/single-source). npcnav
does **not** read them from `mp_game`/`mp_bg` — `mp_engine_server` has no game/bg
source dependency (Cargo deps: `mp_qshared`, `mp_engine_qcommon`, `mp_abi`), so a
`mp_game`-only copy is unreachable. **NAV-D6 (RULING 22) settles their canonical
engine-reachable home as `mp_qshared`** — the single definition the referee
compares, moved or re-exported out of the copies that today sit only in `mp_game`
(`crates/mp/game/src/g_public_consts.rs:14`, `crates/mp/game/src/NPC_combat.rs:2736`,
`crates/mp/game/src/bg_slidemove.rs:37`, `crates/mp/game/src/g_nav_consts.rs:13`,
`crates/mp/game/src/q_math.rs:916`), with no duplication. This matches the
precedent already in that crate: `Q_irand`
(`crates/mp/qshared/src/shared/q_math_rand.rs`) and `failedEdge_t`
(`crates/mp/qshared/src/common/mp/qcommon/failed_edge.rs`) already live in
`mp_qshared`, the only shared tier the engine depends on. These four constants
are therefore **absent from the nav-owned globals table above by design** (they
are not nav globals); the nav-owned consts (`NF_*`, `EFLAG_*`, `NODE_NONE`,
header IDs, `MAX_FAILED_EDGES`, `WP_MINS`/`WP_MAXS`, `CHECKED_*` — all from
navigator.h) remain module consts per fork-3 kind-1, and `WP_MINS`/`WP_MAXS`
build their `-24+STEPSIZE`/`24` bounds (navigator.cpp:50-51) from the
`mp_qshared`-homed `STEPSIZE` (NAV-D6).

## Seam definition

Two seam directions, both preserved exactly (NAV-D5). The host-taking receiver is
frozen by NAV-D2 (RULING 11): every method that reaches a service takes
`(&mut self, host: &mut impl EngineHost)`; the ~9 pure-graph queries take no host.

### Inbound: game → engine (the `G_NAV_*` arms)

The `SV_GameSystemCalls` switch dispatches 42 `G_NAV_*` arms
(`oracle/codemp/game/g_public.h:298-339`) at
`oracle/codemp/server/sv_game.cpp:837-936` — 40 are `CNavigator` **method**
calls (the plan's "39 direct callees" figure, §0.4; two arms are the same
overloaded `GetBestNodeAltRoute`), and two (`G_NAV_GETPATHSCALCULATED`/
`G_NAV_SETPATHSCALCULATED`) read/write the public `pathsCalculated` **field**.
Args arrive as `intptr_t` slots; pointer args use `VMA(n)` (shared-memory base
offset). The pub Rust surface these arms need (`EngineHost` is the one Stage-0
services trait, NAV-D2 — trace/PVS/FS/print/`VM_Call`/shared-memory):

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
fn get_nearest_node(&mut self, host: &mut impl EngineHost, ent: EntityId, last_id: i32, flags: i32, target_id: i32) -> i32; // G_NAV_GETNEARESTNODE
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
// Failed-node bookkeeping (writes into the game entity via host)
fn check_failed_nodes(&mut self, host: &mut impl EngineHost, ent: EntityId);              // G_NAV_CHECKFAILEDNODES
fn add_failed_node(&mut self, host: &mut impl EngineHost, ent: EntityId, node_id: i32);   // G_NAV_ADDFAILEDNODE
fn node_failed(&self, host: &impl EngineHost, ent: EntityId, node_id: i32) -> qboolean;   // G_NAV_NODEFAILED
fn nodes_are_neighbors(&self, start_id: i32, end_id: i32) -> qboolean;                 // G_NAV_NODESARENEIGHBORS
// Failed-edge bookkeeping (failedEdge_t crosses by pointer via VMA)
fn clear_failed_edge(&mut self, host: &mut impl EngineHost, e: &mut failedEdge_t);        // G_NAV_CLEARFAILEDEDGE
fn clear_all_failed_edges(&mut self);                        // G_NAV_CLEARALLFAILEDEDGES
fn edge_failed(&self, start_id: i32, end_id: i32) -> i32;   // G_NAV_EDGEFAILED
fn add_failed_edge(&mut self, host: &mut impl EngineHost, ent_id: i32, start_id: i32, end_id: i32); // G_NAV_ADDFAILEDEDGE (d_patched :1933, Com_Printf :1945-2053, svs.time :1987/2010)
fn check_failed_edge(&mut self, host: &mut impl EngineHost, e: &mut failedEdge_t) -> qboolean; // G_NAV_CHECKFAILEDEDGE
fn check_all_failed_edges(&mut self, host: &mut impl EngineHost);                         // G_NAV_CHECKALLFAILEDEDGES
fn route_blocked(&self, start_id: i32, test_edge_id: i32, end_id: i32, reject_rank: i32) -> qboolean; // G_NAV_ROUTEBLOCKED
fn get_best_node_alt_route(&mut self, host: &mut impl EngineHost, start_id: i32, end_id: i32, path_cost: &mut i32, reject_id: i32) -> i32; // G_NAV_GETBESTNODEALTROUTE
fn get_best_node_alt_route2(&mut self, host: &mut impl EngineHost, start_id: i32, end_id: i32, reject_id: i32) -> i32; // G_NAV_GETBESTNODEALT2 (overload)
fn get_best_path_between_ents(&mut self, host: &mut impl EngineHost, ent: EntityId, goal: EntityId, flags: i32) -> i32; // G_NAV_GETBESTPATHBETWEENENTS
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
per NAV-D3 its §20 preservation obligation travels with the wave-20
`SV_GameSystemCalls` port, and NAV-D5 holds this boundary byte-identical. This
doc's `CNavigator` surface neither emits nor asserts the fall-through.

### Outbound: engine → game (`gameCallbacks.cpp`) and engine services

`CNavigator` reaches back into the game module and the rest of the engine
through the one shared `EngineHost` trait (NAV-D2, RULING 11 — designed once at
Stage-0, not defined by this doc). The services it consumes:

- **Nine game out-calls** (`oracle/codemp/server/NPCNav/gameCallbacks.cpp:6-49`),
  each a thin `VM_Call(gvm, GAME_NAV_*, ...)` (`GAME_NAV_*` enum
  `g_public.h:788-796`; already handled in `mp_game`, NAV-D5) — reached via the
  `EngineHost` `VM_Call` service:
  `NAV_ClearPathToPoint`, `NPC_ClearLOS`, `NAVNEW_ClearPathBetweenPoints`,
  `NAV_CheckNodeFailedForEnt`, `G_EntIsUnlockedDoor`, `G_EntIsDoor`,
  `G_EntIsBreakable`, `G_EntIsRemovableUsable`, `CP_FindCombatPointWaypoints`.
  The `intptr_t`-slot widening for pointer args is mandatory (plan §5.4 — the
  historical `GAME_NAV_CLEARPATHTOPOINT` truncation bug).
- **Engine services** (all through `EngineHost`; each ported by its own
  subsystem): `SV_Trace` (`server/server.h:416`), `SV_inPVS` (server.h:356),
  `SV_GentityNum` (server.h:349), `FS_Read`/`FS_Write`/`FS_FOpenFileByMode`/
  `FS_FCloseFile`, `Cvar_Get`, `Com_Error`(ERR_DROP), `Com_Printf`, and
  `svs.time`. `va`/`Q_irand` are pure `q_shared` helpers (already ported in
  `mp_qshared`), not host services; the vec3 primitives
  (`VectorNormalize`/`DotProduct`/`VectorSubtract`/`VectorCopy`) and the shared
  constants `Q3_INFINITE`/`WORLD_SIZE`/`STEPSIZE`/`WAYPOINT_NONE` are the same
  class of shared import (State-ownership "Shared constants" note, NAV-D6) —
  imported from `mp_qshared`, never re-declared in npcnav, never host services.

### `#[repr(C)]` types touched

`failedEdge_t` (g_public.h:52-58) crosses the seam by pointer (`VMA` →
`&mut failedEdge_t`); it is a **shared** struct (game + engine) and keeps exact
layout — imported from the ported type (`mp_qshared`,
`crates/mp/qshared/src/common/mp/qcommon/failed_edge.rs`; the rosetta-registered
Rust name is `failedEdge_t`, **not** `FailedEdge` — there is no `FailedEdge`
alias in the tree), never re-declared (type-rosetta rule).
`sharedEntity_t` (g_public.h:679-715) is reached through `SV_GentityNum`;
`trace_t`, `vec3_t`, `cvar_t` likewise imported.

## Decisions

**NAV-D1** — The priority queue is transcribed faithfully, **not** replaced with
`std::BinaryHeap`. `CPriorityQueue` (navigator.h:254-276, navigator.cpp:2705-2782)
— a `vector<CEdge*>` driven by `std::push_heap`/`std::pop_heap` under
`NodeTotalGreater` (`first->m_cost > second->m_cost`, min-heap on cost,
:2693-2699) — ports as an owned `Vec<Edge>` with the C++ `push_heap`/`pop_heap`
sift algorithm hand-transcribed, so equal-cost tie order reproduces libstdc++
byte-for-byte. The procedures to transcribe are the ones behind
`std::push_heap`/`std::pop_heap` in libstdc++'s `<bits/stl_heap.h>`
(`__push_heap` sift-up; `__adjust_heap` + `__pop_heap` sift-down) — this is the
**one** source the port reads outside `oracle/`, authoritative *because* the
parity target is the oracle toolchain's own heap; it is transcribed from the
referee toolchain's header (`tools/npcnav-oracle/` compiles the oracle TU against
that same standard library), and the 3a rank-output goldens (NAV-D4) are the
binding byte-for-byte check that catches any slip. The port does **not**
reconstruct the algorithm from memory. Because `CalculatePath` assigns `curRank++` in pop order
(navigator.cpp:853), the tie-break among equal-cost frontier nodes is baked into
every node's rank table and is parity-visible. Rejected `std::BinaryHeap<Edge>`
(the withdrawn prior claim, a settled doc defect): Rust's binary heap resolves
equal keys in a different order than libstdc++'s heap, diverging the ranks.
(RULING 18, 2026-07-09.)

**NAV-D2** — Services reach nav through the one shared `EngineHost` trait, and
nav state is a direct field on `Engine`. Per RULING 11 the trace/PVS/FS/print/
error/`VM_Call`/shared-memory services are the single Stage-0 `EngineHost` trait
(interface crate); every host-taking nav method takes `(&mut self, host: &mut
impl EngineHost)`, and `Engine` supplies the impl through a **split-borrow view
struct that excludes `nav`** — that is what lets `engine.nav.method(&mut view,
…)` borrow `nav` and the rest of `Engine` disjointly. Per RULING 12 the state is
a plain `Default`-initialized `nav: Navigator` field directly on
`mp_engine_core::Engine` (no `Option`/`Box`/nesting); the ctor's lazy
`NAV_CvarInit` (navigator.cpp:39-43,478-484) is modeled with Raven's own init
flag. Resolves NAV-Q1/Q4/Q5. Because the engine-wide fork rulings put every §F
subsystem on this one seam. Rejected a nav-private `NavHost` trait and a
`Server.navigator` sub-struct — RULING 11/12 supersede both.

**NAV-D3** — The `G_NAV_SETCHECKEDNODE`→`FLAGALLNODES`→`GETPATHSCALCULATED`
switch fall-through (a real Raven bug: no `return`/`break`, sv_game.cpp:928-933)
is owned by the wave-20 `SV_GameSystemCalls` transcription, **not** this
subsystem. Because the fall-through is a property of the dispatch switch, not of
any `CNavigator` method; §20's preservation obligation travels with the switch
(build-out plan §0.4 puts it at wave 20). Closes NAV-Q3: no `CNavigator` port
artifact emits or asserts it. Rejected asserting it from a nav method — the
methods never see the arm boundary. (EVIDENCE, engine-fork-discovery NAV-Q3.)

**NAV-D4** — Golden fixtures are path queries over two nav sources: the retail
`.nav` data read locally from the `jka_server` assets (**uncommitted,
ignored-by-default** — never in the public repo) **plus** committed
hand-authored minimal nav graphs. Mirrors the ICARUS ruling-14 pattern
(committed hand-authored goldens + an optional local retail corpus). Because
retail blobs cannot ship in the repo yet the query surface must be exercised
over both minimal and realistic graphs. Resolves NAV-Q2's fixture-source
question. Rejected committing retail `.nav` blobs (licensing) and relying on a
retail-only corpus (not reproducible in CI). (RULING 14 pattern.)

**NAV-D5** — All prior settled nav decisions stand. The node/edge graph is owned
`Vec` arenas indexed by id (node id == index, `m_nodes.size()` assignment
navigator.cpp:712), never a pointer graph (§B5): `CNode.m_edges` →
`Vec<NodeEdge>`, `m_ranks` (heap `int*`) → `Vec<i32>` (`-1` fill),
`m_edgeLookupMap` (`multimap<int,int>`) → `BTreeMap<i32, Vec<usize>>` (per-key
insertion order preserved so `EdgeFailed`'s `equal_range` first-match,
:1876-1898, is reproduced), `CheckedNodes`/`ShowEdges` maps → `BTreeMap`
(iteration/lookup determinism). The `GAME_NAV_*`/`G_NAV_*` boundary is kept
exactly as the syscall switch presents it — numbers, arg order, `VMA`
marshaling, `intptr_t`-slot widening — and the `GAME_NAV_*` handlers already in
`mp_game` are not re-ported. Because these were settled before the §F doc session
and rulings 11/12/14/18 do not disturb them. Rejected `HashMap` (nondeterministic
iteration) and collapsing the two `GetBestNodeAltRoute` overloads (the game
module issues both arm numbers).

**NAV-D6** — The shared constants and vec3 primitives the nav code consumes but
does not own — `Q3_INFINITE` (g_public.h:9, `16777216`), `WORLD_SIZE`
(q_shared.h:20), `STEPSIZE` (bg_public.h:22, `18`), `WAYPOINT_NONE` (g_nav.h:7,
`-1`), and `VectorNormalize`/`DotProduct`/`VectorSubtract`/`VectorCopy`
(q_shared.h / q_math.c) — get their **canonical engine-reachable home in
`mp_qshared`**: one definition, the single one the referee compares, moved or
re-exported out of the copies that today sit only in `mp_game`
(`crates/mp/game/src/g_public_consts.rs:14`, `.../NPC_combat.rs:2736`,
`.../bg_slidemove.rs:37`, `.../g_nav_consts.rs:13`, `.../q_math.rs:916`), with
**no duplication** — npcnav re-declares none of them and consumes them from
`mp_qshared`. `WP_MINS`/`WP_MAXS` (navigator.cpp:50-51) and the affected pure-graph
queries (`GetBestNode`/`GetPathCost`/`GetProjectedNode`, `GetEdgeCost`'s id form,
`CNode::GetPosition`) cite the `mp_qshared` source for those items. Because the
engine depends only on `mp_qshared`/`mp_engine_qcommon`/`mp_abi` — never on
`mp_game` (game and engine are separate ABI-boundary binaries) — and `mp_qshared`
is the shared tier that precedent already homes these classes in (`Q_irand`,
`crates/mp/qshared/src/shared/q_math_rand.rs`; `failedEdge_t`,
`crates/mp/qshared/src/common/mp/qcommon/failed_edge.rs`). Closes NAV-Q6 and
unblocks the first slice. Rejected a local npcnav copy (porting-rules
§14/single-source forbids it) and homing them in `native_math`/`mp_game` (the
former splits the definition across crates the referee would have to reconcile,
the latter is unreachable from the engine). (RULING 22, 2026-07-09.)

## Verification strategy

C++ track → porting-rules §F / §18: differential goldens from the unmodified
oracle TU, committed so `cargo test` needs no C++ toolchain. Harness home:
`tools/npcnav-oracle/` (GP2 pattern — stub headers under it, oracle never
edited).

**Fixture sources (NAV-D4, ICARUS ruling-14 pattern):** committed hand-authored
minimal nav graphs are the public, CI-reproducible corpus; the retail `.nav`
data read from the local `jka_server` assets is an **uncommitted,
ignored-by-default** extra corpus that may run locally. Goldens are dumped from
the oracle over both and committed only for the hand-authored set.

**Golden surface (3a, primary — path-query goldens):** after `Load` (or
`CalculatePaths` regenerating ranks), the pure-graph query surface is fully
deterministic with **no trace/PVS/callback** dependency — the ranks are baked
into the file / recomputed in-process. Dump-and-compare `GetBestNode`,
`GetBestNodeAltRoute`, `GetPathCost`,
`Connected`, `NodesAreNeighbors`, `GetProjectedNode`,
`GetNodeNumEdges`/`GetNodeEdge`/`GetNodePosition`/`GetNodeRadius`/`GetNumNodes`,
plus `CalculatePaths` → `GetPathCost`. The priority-queue tie order (NAV-D1) is
exercised transitively through `CalculatePath`'s rank output — the primary reason
the faithful heap is testable without a bespoke probe; these rank goldens are the
binding check on the `<bits/stl_heap.h>` sift transcription named in NAV-D1. The oracle side stubs
`FS_Read` against the fixture bytes and stubs `Com_Printf`/`Cvar_Get`
(`d_altRoutes`/`d_patched` forced to fixed values so both `d_altRoutes` branches
are covered).

**Trace/callback-dependent surface (3c, referee swap-in):** `GetNearestNode`,
`GetBestPathBetweenEnts`, `CheckBlockedEdges`, `HardConnect`, `GetEdgeCost`
(both the public `int,int` form — which validates ids then delegates to the
trace form unconditionally, navigator.cpp:2634 — and the `CNode*,CNode*` trace
form :734-755), `CheckFailedNodes`, `CheckFailedEdge`,
`CheckAllFailedEdges` reach `SV_Trace`/`SV_inPVS`/`SV_GentityNum` and the nine
game callbacks (all `EngineHost` services), so they need live engine + game
state. They verify under the plan's §3c A/B referee (`crates/jampgame/tests/
referee.rs` / the external `sv_referee` rig) once the server spine is real, or
via captured-trace replay (§3b), the deterministic `EngineHost` impl injected
per RULING 11.

Governing clause: porting-rules §F (§18 differential goldens; §19 UB
divergence; §20 emergent-quirk preservation; §21 one class per file).

## Slice hooks

- Build-out plan §0.4 / wave 20: `SV_GameSystemCalls` — must have this doc's
  pub surface (Seam definition) frozen before its `G_NAV_*` arms are filled; it
  also owns the SETCHECKEDNODE/FLAGALLNODES fall-through (NAV-D3).
- Build-out plan wave 25 (server complete) / M4: the full nav subsystem must be
  green under the 3c referee swap-in.
- The `EngineHost` trait (Stage-0 interface crate, RULING 11) and the `Engine`
  split-borrow view struct must exist before the host-taking methods can be
  written — this is a shared Stage-0 dependency, not a nav-specific open point
  (NAV-D2 froze the receiver `(&mut self, &mut impl EngineHost)`).
- **First slice (host-independent — portable now that NAV-D6 lands the shared
  home).** The nine host-free pure-graph queries (`GetBestNode`,
  `GetNodePosition`, `GetNodeNumEdges`, `GetNodeEdge`, `GetNumNodes`,
  `Connected`, `GetPathCost`, `GetProjectedNode`, `GetNodeRadius`) plus the type
  skeletons — `mod.rs` consts (State-ownership table), `edge.rs` `Edge` (D-1),
  `node.rs` `Node`'s host-free members (accessors navigator.h:94-110, edge/rank
  queries, `Create`, `AddEdge`; `Save`/`Load` are **deferred**, they take `FS_*`
  via host), and `priority_queue.rs`'s faithful `Vec<Edge>` heap (NAV-D1/D-7) —
  form a self-contained, **host-independent** slice that verifies against the 3a
  path-query goldens: it needs no `EngineHost` service, so the Stage-0 host seam
  is not its blocker. Every host-taking method and the whole of `callbacks.rs`
  land separately, once that Stage-0 `EngineHost` seam is present; under
  GOAL-engine no-stub discipline a porter writes them against the frozen
  `EngineHost` trait, never a stub. The slice consumes the shared
  `Q3_INFINITE`/`WORLD_SIZE`/`WAYPOINT_NONE` consts
  (`GetBestNode`/`GetPathCost`/`GetEdgeCost`, the latter delegating per
  navigator.cpp:2634), the vec3 primitives (`GetProjectedNode` — pure,
  navigator.cpp:2643-2686 — and `CNode::GetPosition`), and `mod.rs`'s
  `WP_MINS`/`WP_MAXS`'s `STEPSIZE`; npcnav owns none of these and re-declares none
  of them (porting-rules §14/single-source), importing them all from `mp_qshared`
  per **NAV-D6** — the crate `mp_engine_server` already depends on
  (`mp_qshared`/`mp_engine_qcommon`/`mp_abi`). With NAV-D6 settled there is no
  longer any open blocker on this slice.

## Method transcription table

81 functions (per plan §0.4); inline accessors fold into their owning struct's
impl. Grouped by Raven class; Rust shape per NAV-D1/D5.

| Raven method | oracle cite | Rust shape |
| --- | --- | --- |
| `CEdge::CEdge()` / `(int,int,int)` / `~CEdge` | :82-96 | `Edge { first, second, cost }`; 0-arg ctor is a Raven no-op (divergence D-1) |
| `CNode::CNode`/`~CNode`/`Create(...)`/`Create()` | :104-147 | `Node::new` / `Node::create(pos,flags,radius,id)`; `Vec`-owned (no `new`/`delete`); `GetPosition`'s vec3 helpers imported from `mp_qshared` (NAV-D6) |
| `CNode::AddEdge` | :155-183 | dedup-or-push into `edges: Vec<NodeEdge>`; `assert(<9)` → `debug_assert!` (D-6) |
| `CNode::GetEdgeNumToNode`/`GetEdge`/`GetEdgeCost`/`GetEdgeFlags`/`SetEdgeFlags` | :191-344 | index/scan `edges`; keep `edgeNum > m_numEdges` bound verbatim (D-2) |
| `CNode::AddRank`/`InitRanks`/`GetRank` | :214-376 | `ranks: Vec<i32>` (`-1` fill) |
| `CNode::Draw` | :227-236 | empty (renderer stripped) — port as no-op with §20 note |
| `CNode::Save`/`Load` | :385-470 | `FS_*` via `EngineHost`; `NODE_HEADER_ID` check |
| `CNode` inline accessors (`GetID`,`GetPosition`,`GetNumEdges`,`GetRadius`,`GetFlags`,`AddFlag`,`RemoveFlag`) | navigator.h:94-110 | trivial methods |
| `CNavigator::CNavigator`/`~CNavigator` | :478-488 | `Navigator::default`; ctor's lazy `NAV_CvarInit` → cvar handles via host, Raven init flag |
| `CNavigator::Init`/`Free` | :572-594 | clear `nodes`/`edge_lookup` |
| `CNavigator::Load`/`Save` | :602-702 | `FS_*` via host; rebuild `edge_lookup` |
| `CNavigator::AddRawPoint` | :710-726 | push `Node`; `Com_Error` branch dead (D-3) |
| `CNavigator::GetEdgeCost(int,int)` / `GetEdgeCost(CNode*,CNode*)` | :2621-2635,:734-755 | public `int,int` form validates ids then delegates to the trace form (:2634); `SV_Trace` via host — trace-dependent (3c), host-taking |
| `CNavigator::SetEdgeCost`/`AddNodeEdges` | :757-806 | id-indexed; bidirectional add |
| `CNavigator::CalculatePath`/`CalculatePaths` | :814-908 | faithful `Vec<Edge>` heap flood fill (D-7 raw-ptr ownership → owned values; pop-order ranks NAV-D1) |
| `CNavigator::ShowNodes`/`ShowEdges`/`ShowPath` | :916-1027,:1632-1685 | draw calls stripped (renderer); keep PVS/`Com_Printf` control flow, §20 notes |
| `CNavigator::GetNodeRadius` | :1029-1034 | pure query — `m_nodes[id].radius` with the §19 range guard (D-8), host-free (golden surface) |
| `CNavigator::CheckBlockedEdges`/`HardConnect` | :1036-1140 | host trace + door/breakable callbacks |
| `CNavigator::TestNodePath`/`TestNodeLOS`/`TestBestFirst` | :1150-1237 | protected; host callbacks |
| `CNavigator::CollectNearestNodes` | :1249-1318 | `nodeChain_l` → `Vec`/`VecDeque` insert-sorted (NAV-D5) |
| `CNavigator::GetBestPathBetweenEnts`/`GetNearestNode` | :1320-1624 | host trace/PVS; writes `ent->waypoint` via host |
| `CNavigator::ClearCheckedNodes`/`CheckedNode`/`SetCheckedNode` | :1687-1719 | `checked_nodes: BTreeMap<i32,u8>` |
| `CNavigator::CheckFailedNodes`/`AddFailedNode`/`NodeFailed` | :1724-1811 | read/write `sharedEntity_t` via `SV_GentityNum` |
| `CNavigator::NodesAreNeighbors` | :1813-1833 | scan node edges |
| `CNavigator::ClearFailedEdge`/`ClearAllFailedEdges` | :1835-1874 | `failed_edges[..]`; `memset(WAYPOINT_NONE)` → explicit fill |
| `CNavigator::EdgeFailed`/`AddFailedEdge` | :1876-2055 | `edge_lookup` `equal_range` first-match (NAV-D5) |
| `CNavigator::CheckFailedEdge`/`CheckAllFailedEdges` | :2057-2168 | host trace/PVS; `#if 0` NAVNEW branch not taken (D-4) |
| `CNavigator::RouteBlocked` | :2170-2253 | rank-guided walk; `while(1)` loop |
| `CNavigator::GetBestNodeAltRoute` (both overloads) | :2261-2370 | 3-arg delegates to 4-arg |
| `CNavigator::GetBestNode`/`GetNodePosition`/`GetNodeNumEdges`/`GetNodeEdge`/`Connected`/`GetPathCost`/`GetProjectedNode` | :2377-2686 | pure graph queries (golden surface); `Q3_INFINITE`/`WORLD_SIZE`/`WAYPOINT_NONE` + vec3 primitives imported from `mp_qshared` (NAV-D6) |
| `CNavigator::FlagAllNodes`/`GetChar`/`GetInt`/`GetFloat`/`GetLong`/`GetNumNodes` | :496-564,navigator.h:184 | helpers; `Get*` read via host `FS_Read` |
| `NodeTotalGreater::operator()` | :2693-2699 | the `first.cost > second.cost` comparator for the faithful heap sift (NAV-D1) |
| `CPriorityQueue::~/Find/Pop/Push/Update/Empty` | :2705-2782 | owned `Vec<Edge>` with hand-transcribed `push_heap`/`pop_heap` (NAV-D1/D-7); `Find`/`Update` have no live caller — §20 |
| `NAV_CvarInit`/`NAV_Free` | :39-48 | host cvar registration / `Navigator::free` |
| `GetTime` (`#if AI_TIMERS`) | :59-74 | not ported (`AI_TIMERS` off) — §20 |
| `CNavigator::GetNodeLeadDistance` | navigator.h:182 | declared-only, **no definition** in navigator.cpp and no caller/trap arm — dropped as dead surface (§20 zero-caller note), not stubbed |
| `GNavCallback_*` ×9 | gameCallbacks.cpp:6-49 | `EngineHost` `VM_Call(GAME_NAV_*)` (NAV-D2/D5) |

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
  hand-transcribed (NAV-D1). Ownership/layout is free (§F); pop order under
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
    summary: Nav module root — NF_*/EFLAG_* flags, NODE_NONE, NAV/NODE header IDs, MAX_FAILED_EDGES, WP_MINS/WP_MAXS, CHECKED_* consts (all navigator.h, nav-owned); re-exports. Q3_INFINITE/WORLD_SIZE/STEPSIZE/WAYPOINT_NONE and the vec3 primitives are NOT defined here — imported from mp_qshared (NAV-D6/RULING 22), never local copies. Navigator becomes the Engine.nav field (RULING 12).
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
    summary: CNavigator — node/edge arenas, failed-edge/checked-node bookkeeping, priority-queue pathfinding, Load/Save, the G_NAV_* pub surface; host-taking methods take (&mut self, &mut impl EngineHost) (NAV-D2).
  - path: crates/mp/engine/server/src/npcnav/priority_queue.rs
    crate: mp_engine_server
    mode: mp
    class: CPriorityQueue
    summary: Faithful Vec<Edge> min-heap on cost — hand-transcribed push_heap/pop_heap under NodeTotalGreater so equal-cost tie order matches libstdc++ (NAV-D1/D-7); NOT std::BinaryHeap; Find/Update dropped as zero-caller.
  - path: crates/mp/engine/server/src/npcnav/callbacks.rs
    crate: mp_engine_server
    mode: mp
    class: (GNavCallback free fns)
    summary: The nine GNavCallback_* outbound calls as EngineHost VM_Call(GAME_NAV_*) shims (gameCallbacks.cpp); NAV_CvarInit/NAV_Free.
```

## Open questions

MUST be empty at FROZEN. **All of NAV-Q1–Q6 are resolved** — Q1–Q5 by the §F
doc-session rulings, Q6 by RULING 22 (NAV-D6) at the interactive design session
that followed the 2026-07-09 dry-run escalation. Retained here as
resolved-in-place notes for cross-doc ID stability (never re-litigate). No open
question remains.

- **NAV-Q1** — *(Resolved: NAV-D2 / RULING 11.)* Host-threading mechanism for the
  trace/FS/callback services = the one shared Stage-0 `EngineHost` trait; every
  host-taking method takes `(&mut self, host: &mut impl EngineHost)`.
- **NAV-Q2** — *(Resolved: NAV-D4 / RULING 14 pattern.)* Fixtures = committed
  hand-authored minimal nav graphs + an uncommitted local retail `.nav` corpus.
  The exact per-fixture probe list is a mechanical Verification-plan detail the
  harness enumerates, not a design point.
- **NAV-Q3** — *(Resolved: NAV-D3 / EVIDENCE.)* The SETCHECKEDNODE/FLAGALLNODES
  switch fall-through is owned by the wave-20 `SV_GameSystemCalls` port; no
  `CNavigator` artifact is responsible for it.
- **NAV-Q4** — *(Resolved: NAV-D2 / RULING 11.)* There is no nav-private host
  trait; the required services are methods on the shared `EngineHost` trait,
  designed once at Stage-0. This doc names the Raven services it consumes
  (Seam definition, outbound); their Rust signatures live with the `EngineHost`
  design.
- **NAV-Q5** — *(Resolved: NAV-D2 / RULING 11.)* The `Navigator`-vs-rest
  self-borrow is resolved by `Engine`'s split-borrow view struct that excludes
  `nav`, so `engine.nav.method(&mut view, …)` borrows disjointly.
- **NAV-Q6** — *(Resolved: NAV-D6 / RULING 22, 2026-07-09.)* Canonical
  engine-reachable home for the shared constants/helpers the nav code **consumes
  but does not own** (`Q3_INFINITE`, `WORLD_SIZE`, `STEPSIZE`, `WAYPOINT_NONE`,
  and the vec3 primitives `VectorNormalize`/`DotProduct`/`VectorSubtract`/
  `VectorCopy`) = **`mp_qshared`**: one definition the referee compares, moved or
  re-exported out of the copies that today sit only in `mp_game`, with no
  duplication. Authorizes promoting the `bg_public.h` `STEPSIZE` and the un-homed
  `g_nav.h` `WAYPOINT_NONE` and relocating the vec3 math + `Q3_INFINITE` +
  `WORLD_SIZE` copies into that tier. npcnav re-declares none of them locally and
  imports them from `mp_qshared`; the first slice (Slice hooks) is unblocked.
