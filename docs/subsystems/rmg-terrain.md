# RMG + qcommon terrain classes (§F idiomatic reimplementation) Design
Status: DRAFT     Supersedes: none
Decision prefix: RMG     Ledger deps: DEC-01, DEC-04, DEC-09

## Standing context

Links only — never restated here:

- `docs/workspace-architecture.md` — crate graph; the two crates this doc
  targets are `mp_engine_rmg` (`crates/mp/engine/rmg/`) and `mp_engine_qcommon`
  (`crates/mp/engine/qcommon/`).
- `docs/porting-rules.md` — §B (state spine), §F (C++ track: §17 design-before-
  transcription, §18 differential goldens, §19 UB divergence, §20 dead-surface
  drop, §21 one-class-per-file). This subsystem is pure §F.
- `docs/GOAL-engine.md` — the dedicated MP engine goal; RMG is in the WinDed
  Release link set, done at wave 16. **The whole engine is built `DEDICATED`**,
  which is what makes RMG-D1 (ruling 25) apply.
- `docs/plans/2026-07-08-mp-engine-build-out.md` — §"RMG (113 fns, wave 16)",
  the cross-subsystem matrix (6 server→RMG edges), §3c (OpenJK dropped RMG).
- `docs/handoffs/engine-fork-discovery.md` — settled forks and the §F rulings
  this doc consumes: **fork-2** (global state placement, `:21-29`), **fork-3**
  (function-scope statics, `:31-36`), **fork-5** (internal dispatch tables → plain
  fn-item structs / const slices, `:46-53`), **ruling 7** (the blessed 5-doc §F
  list — RMG is one, `:61-68`), **ruling 11** (the one `EngineHost` services trait
  + view-struct impl, `:121-126`) and **ruling 24** (that trait is pinned to the
  Stage-0 crate `mp_host_interface`, `crates/mp/host-interface`, `:229-230` — this doc
  cites it, does not define its signatures), **ruling 12** (the five §F states are plain
  Default-initialized direct `Engine` fields — `rmg` among them, `:127-131`),
  **ruling 16** (the qcommon terrain twins fold into *this* doc, `:143-146`),
  **ruling 17** (the four §20 dead-surface drops, `:147-152`), **ruling 21** (the
  five holes closed, RMG-D2d…h) and **ruling 25** (2026-07-09: RMG generation is
  dead under DEDICATED — the headline of RMG-D1).
- `docs/architecture/state-ownership.md` — the STATE-* ledger: STATE-D5 (the one
  `Engine` island lives in `mp_engine_core`, `crates/mp/engine/core/src/engine.rs:20`),
  STATE-D2 (`Engine.cm: mp_engine_qcommon::CollisionWorld` owns Raven's `cmg`
  clipmap — `state-ownership.md:418`, `collision_world.rs:10`). STATE-Q2's
  **placement half** is resolved by ruling 12 (direct `Engine.rmg` field); its
  service half is ruling 11 (`EngineHost`).
- `docs/decisions.md` — DEC-01 (renderer deferred), DEC-04 (strict per-mode),
  DEC-09 (engine verification: TU harnesses + live peers).
- GP2 is the §F exemplar (`crates/mp/engine/qcommon/src/gp2/`,
  `tools/gp2-oracle/`). Under RMG-D1 it is **no longer a live dependency** of the
  ported surface — every GP2 parse path runs through the mission/instance-file
  load, which is §20-dropped (RMG-D1). It remains the design exemplar for the
  §F shapes recorded in the divergences.

## Scope & non-goals

**RMG-D1 (ruling 25) is the governing fact for this whole doc: the RMG
generation path is unreachable code under DEDICATED, and the engine is built
DEDICATED.** `CreateRandomTerrain`'s only call site is inside the `#else` of
`#ifdef DEDICATED` (`oracle/codemp/qcommon/cm_terrain.cpp:170-188`, call at
`:178`), so `CCMLandScape::mRandomTerrain` stays `0` (`cm_terrain.cpp:169`;
default `GetRandomTerrain()` returns it, `cm_landscape.h:236`). `SetLandScape`
therefore sets `mTerrain = NULL` (`RM_Manager.cpp:82`), and `LoadMission`
early-outs at `if (!mTerrain) return false` (`RM_Manager.cpp:110-113`) **before**
it constructs the `CRMMission` (`:135`). Because `G_RMG_INIT` only spawns on a
truthy `LoadMission` (`if (LoadMission(qtrue)) SpawnMission(qtrue)`,
`sv_game.cpp:1632-1634`), `SpawnMission` is never called. The entire generation
subtree downstream of that early-out is dead code on the dedicated server.

**In scope — the LIVE surface (RMG-D1: syscall arms + landscape construction +
early-out).** Only three things are reachable under DEDICATED and are ported:

1. **The reachable RMG syscall arms** — `G_CM_REGISTER_TERRAIN`
   (`sv_game.cpp:1640-1641`) and `G_RMG_INIT` (`sv_game.cpp:1624-1638`).
   (`G_SET_ACTIVE_SUBBSP` is out-of-scope clipmap wiring, Non-goals.)
2. **`CCMLandScape` construction under DEDICATED** (`cm_terrain.cpp:116-219`):
   config parse, bounds, heightmap/flatten allocation (`mFlattenMap` memset-0 at
   `:161`; `mHeightMap` allocated but **unpopulated** — no image load, no
   generation, under DEDICATED), `LoadTerrainDef` (`:208`), patch build
   (`mPatches`, `UpdatePatches` — `:211-218`) with the `CCMPatch` collision data,
   and the seeded per-instance LCG (`holdrand = 0x89abcdef`, `:122`). Plus the
   `RmManager` lifecycle through the early-out (`new`, `SetLandScape`,
   `LoadMission → false`) and the automap-symbol seam pair (which return count `0`
   / nothing under DEDICATED, `RM_Manager.cpp:41,413-421`).
3. **The snapshot/download read of the constructed landscape** —
   `SV_SendClientGameState` streams `GetHeightMap()`/`GetFlattenMap()`/
   `get_rand_seed()` and the automap symbols (`sv_client.cpp:768-809`), and
   `SV_WriteRMGAutomapSymbols` writes the (zero) symbol count
   (`sv_client.cpp:668-684`). Under DEDICATED this streams the default,
   un-generated terrain — the observable "an RMG mission on the dedicated server
   fails identically to C" outcome the goldens pin (RMG-D1).

**Live runtime surface of the constructed landscape (owned here, RMG-D2a).** The
`CmLandScape` built in item 2 has live external readers beyond construction: the
item-3 snapshot accessors, **and** the per-frame terrain-collision methods
`PatchCollide`/`WaterCollide` (+ the water/bounds accessors) the `cm-trace`/
`cm-test` C-track packets call (`cm_trace.cpp:283,760,789`; `cm_test.cpp:285-289`).
By RMG-D2a (the `cm` C-track packets exclude `CCMLandScape`) these methods are
owned by *this* subsystem and land with `CmLandScape` in Wave 16 — there is no
other doc. The collision path was **not** in RMG-D1's three-item enumeration
(a design-session omission, not a decision to drop it); its scope + Rust
signatures are **RMG-Q8/Q9**. See `## Seam definition` §C.

**In scope — the qcommon terrain twins** (`oracle/codemp/qcommon/`), folded here
by **ruling 16** (RMG-D2a), reduced to their live members: `CCMLandScape` +
`CCMPatch`/`CCMHeightDetails` (`cm_terrain.cpp`, `cm_landscape.h`) and the
golden-only `RMG_CreateSeed` free function (`cm_randomterrain.cpp:1008`). The `cm`
C-track packets exclude these classes (ruling 16). Everything else in these TUs —
`CRandomTerrain`, `CPathInfo`, `CTerrainMap`, `CArea`, the area/carve methods — is
generation-path or renderer-only and is §20-dropped (below).

**Non-goals** (punted / dropped, each with its owner):
- **The entire RMG generation path** (RMG-D1, §20-dropped, shape-map entries kept
  in Divergences): `CreateRandomTerrain`, `CRandomTerrain`/`CPathInfo`
  (`Generate`/`Smooth`/`ParseGenerate`), `CRMMission` (`Load`/`Spawn`/`PreSpawn`/
  `Smooth`/`PlaceBridges`/`ParsePaths`/`ParseRivers`), the `CRMInstance` hierarchy,
  `CRMInstanceFile`, `CRMObjective`, `CRMPathManager` (with `CRMNode`/`CRMLoc`/
  `CRMCell`), `CRMArea`/`CRMAreaManager` (the `AreaId` arena), and the
  `CmLandScape.random_terrain` field. None is reachable past `LoadMission`'s
  early-out (`RM_Manager.cpp:110-113`). Recorded per §20 with dead-under-DEDICATED
  notes, not ported.
- **SP RMG** (`oracle/code/RMG/`, a near-duplicate tree). Per-mode discipline
  (DEC-04) forbids unifying it; SP engine is a later campaign.
- **The wider clipmap** (`cm_load.cpp`, `cm_patch.cpp`, `cm_trace.cpp`). Only the
  terrain-owned members of `CCMLandScape` are here; `CM_RegisterTerrain`'s clipmap
  wiring is a C-track qcommon packet.
- **Renderer-side terrain draw** (`tr_terrain*`) and everything gated on it.
  Deferred with the renderer (DEC-01); not in the dedicated link set.
- **The four ruling-17 §20 drops** (RMG-D2c): `mCurObjective`, the dead Perlin
  scratch (`noiseTable`/`noisePerm`), the `RM_Terrain.cpp` client-model chain,
  and the `CTerrainMap` automap-image builder. Recorded in Divergences.

## Raven ground truth

### Data flow (server boot → terrain → mission), corrected for DEDICATED

1. The game module vmcalls `trap_CM_RegisterTerrain(config)`
   (`oracle/codemp/game/g_syscalls.c:1473-1476`, `g_misc.c:582`). The syscall case
   `G_CM_REGISTER_TERRAIN` calls `CM_RegisterTerrain((char*)VMA(1), true)` and
   returns `->GetTerrainId()` (`oracle/codemp/server/sv_game.cpp:1640-1641`).
   `CM_RegisterTerrain` (`oracle/codemp/qcommon/cm_load.cpp:1036`) constructs the
   `CCMLandScape` (or, on repeat registration, `IncreaseRefCount()`s the existing
   one and returns it — `cm_load.cpp:1040-1044`). **The `CreateRandomTerrain` arm
   at `cm_terrain.cpp:178` never runs under DEDICATED** — it is in the `#else` of
   `#ifdef DEDICATED` (`:170-188`), where `imageData` is forced `NULL` (`:171`),
   so the `if (imageData)` body is skipped; `mRandomTerrain` stays `0` (`:169`).
2. `CCMLandScape::CCMLandScape` seeds `holdrand = 0x89abcdef` (`cm_terrain.cpp:122`),
   memsets `mFlattenMap` to 0 (`:161`), allocates the (unpopulated) `mHeightMap`
   (`:157`), parses the terrain def (`LoadTerrainDef`, `:208`), and builds the
   patch/collision arrays (`mPatches`/`UpdatePatches`, `:211-218`). This is the
   full live construction under DEDICATED; only the heightmap-image / random-terrain
   population is skipped.
3. The game vmcalls `trap_RMG_Init(terrainID)` (`g_syscalls.c:1478-1481`,
   `g_misc.c:608`). Case `G_RMG_INIT` (`sv_game.cpp:1624-1638`), gated on
   `com_RMG->integer`: lazily `new CRMManager`, `SetLandScape(cmg.landScape)`, then
   `if (LoadMission(qtrue)) SpawnMission(qtrue)`.
4. **`SetLandScape` sets `mTerrain = landscape->GetRandomTerrain()`
   (`RM_Manager.cpp:79-83`), which is `NULL` under DEDICATED** (step 1:
   `mRandomTerrain == 0`). `LoadMission` prints the RMG banner — **guarded by
   `#ifndef FINAL_BUILD`** (`RM_Manager.cpp:105-108`) — and then hits
   `if (!mTerrain) return false`
   (`:110-113`) — it returns **before** `new CRMMission (mTerrain)` at `:135`.
   No mission, path grid, instance, area, or objective object is ever constructed.
5. **`SpawnMission` is never reached** — `G_RMG_INIT` only calls it on a truthy
   `LoadMission` (`sv_game.cpp:1632-1634`), and `LoadMission` returns false. So
   `CRMMission::Spawn`/`PreSpawn`/`Smooth`/`PlaceBridges`, `CRandomTerrain::Generate`,
   `CRMPathManager::GeneratePaths`/`GenerateRivers`, all `CRMInstance` placement,
   and every automap-symbol `AddAutomapSymbol` (`RM_Manager.cpp:400-410`, the only
   writer of `mAutomapSymbolCount`) never execute. `mAutomapSymbolCount` stays `0`
   (ctor `RM_Manager.cpp:41`).
6. Snapshot/download path (LIVE, un-generated): `SV_SendClientGameState` guards on
   `if (TheRandomMissionManager)` (non-NULL once `G_RMG_INIT` `new`d it) and streams
   `GetLandScape()->GetHeightMap()` (`sv_client.cpp:779`), `GetFlattenMap()`
   (`:795`, the memset-0 map), `get_rand_seed()` (`:806`, the `0x89abcdef` seed —
   never re-seeded, since `rand_seed` is only called from the dead
   `CreateRandomTerrain`), and `SV_WriteRMGAutomapSymbols` (`:808`), which writes
   `count = 0` (`sv_client.cpp:670-673`). `sv_snapshot.cpp:394` gates on `com_RMG`.

### Frame role

RMG is **generation-time, not per-frame** — and under DEDICATED the generation
step is a no-op that returns false. The whole live *generation* tree runs once at
`SV_SpawnServer` time (through the two syscall arms); afterward the produced
`CCMLandScape` collision/height data is touched per frame — the **caller**
`cm_trace`/`cm_test` is out of scope (a `cm` C-track packet), but the
`CCMLandScape` collision **methods** it invokes (`PatchCollide`/`WaterCollide` +
the water/bounds accessors, gated `com_terrainPhysics->integer`,
`cm_trace.cpp:283,760,789`) are owned by *this* subsystem (RMG-D2a) and ported
here — see `## Seam definition` §C / RMG-Q8. Automap symbols are read once at
client connect and are empty.

### Class tree (closed hierarchy — recorded for the §20 shape-map)

`CRMInstance` is an abstract base with four concrete subclasses and pure-ish
virtuals `PreSpawn`/`Spawn`/`PostSpawn`/`SetArea`/… (`oracle/codemp/RMG/
RM_Instance.h:25-117`). The factory `CRMInstanceFile::CreateInstance`
string-dispatches `"bsp"|"group"|"random"|"void"` to `new CRM{BSP,Group,Random,
Void}Instance` (`RM_InstanceFile.cpp:138-193`); no subclass is created anywhere
else — the hierarchy is **closed**. This shape (base+4 → one `RmInstance` enum) is
recorded in Divergences (RMG-D2i) because the dropped-path classes keep their
shape-map entries (RMG-D2), even though nothing here constructs them under
DEDICATED.

### Globals (see State ownership for owners)

- `CRMManager* TheRandomMissionManager` — the one live singleton
  (`oracle/codemp/RMG/RM_Manager.cpp:23`; extern `RM_Manager.h:60`). LIVE (it is
  `new`d and runs through the early-out).
- `CRMManager::mCurObjective` — static member, zero-init only (`RM_Manager.cpp:16`)
  and never read/written in codemp. §20-dropped (ruling 17 / RMG-D2c).
- `static CTerrainMap* TerrainMap` (`cm_terrainmap.cpp:14`), `static float
  noiseTable[256]` / `static int noisePerm[256]` (`cm_randomterrain.cpp:14-15`),
  the seed-name tables (`Consonants[]`, `cm_randomterrain.cpp:847+`), and
  `CRMPathManager::neighbor_x/y` (`RM_Path.h:172-173`) — all on the §20-dropped
  generation/renderer path (RMG-D1/RMG-D2c); recorded in Divergences, not ported.
- `static int instanceID` in `CreateInstance` — assigned-never-read scratch on the
  dropped path (`RM_InstanceFile.cpp:140`).
- The free-function `flrand`/`irand` LCG over the file-scope global
  `holdrand = 0x89abcdef` (`oracle/codemp/game/q_math.c:1432,1441-1470`), seeded by
  `Rand_Init` (`:1434`). Its only RMG consumer is `RMG_CreateSeed`
  (`cm_randomterrain.cpp:1008,1016-1018`), which has **zero live callers** and is
  kept **golden-only** (RMG-D2f) — no live RMG path draws it.
- cvars `com_RMG` (`oracle/codemp/qcommon/common.cpp:72,1335`) — LIVE, gates the
  `G_RMG_INIT` arm; `com_terrainPhysics` (`cm_landscape.h:267`).
- Per-instance RNG state: `CCMLandScape::holdrand` (member `cm_landscape.h:160`,
  seeded `cm_terrain.cpp:122`). LIVE — the field is seeded in the live ctor and
  `get_rand_seed()` is streamed (`sv_client.cpp:806`). Its `flrand`/`irand`/
  `rand_seed` methods (`cm_terrain.cpp:1548-1580`) have no live caller under
  DEDICATED (all draws are on the dropped generation path); §20 within the ported
  class. See RNG threading.

## State ownership

Per **ruling 12** (the five §F states are plain Default-initialized direct
`Engine` fields — no Option/Box/nesting; lazy-init timing modeled with Raven's own
initialized flags) and §B. RMG-D1 marks the generation-path owners **dropped**
(shape-map entry retained in Divergences).

| Raven global | oracle cite | Rust owner (crate::Type.field) | constructed by | threaded via |
| --- | --- | --- | --- | --- |
| `TheRandomMissionManager` | `RM_Manager.cpp:23` | `mp_engine_core::Engine.rmg: RmManager` (plain direct field, ruling 12; STATE-D5, `engine.rs:20`). Raven lazily `new`s it under `com_RMG`; modeled with the private `RmManager.initialized: bool` (Default `false`, flipped at the `G_RMG_INIT` arm — Seam-A owned-state note), not `Option` | `G_RMG_INIT` case (lazy) — `sv_game.cpp:1627-1629` | `&mut self` + `&mut impl EngineHost` from the syscall switch inward |
| `CRMManager::mCurObjective` | `RM_Manager.cpp:16` | **dropped** — §20 dead surface (RMG-D2c/ruling 17): zero-init, never read/written | — | — |
| `CCMLandScape*` (`cmg.landScape`) | `cm_landscape.h:135`; `cm_local.h:155`; `sv_game.cpp:1631` | `mp_engine_qcommon::CollisionWorld.land_scape: Option<CmLandScape>` (a field on the existing STATE-D2 `cmg` owner — `collision_world.rs:10`). `Option` is Raven-faithful: `cmg.landScape` is a nullable pointer set only on a terrain map. **LIVE** — constructed under DEDICATED | `CM_RegisterTerrain` — `cm_load.cpp:1036,1055` | `TerrainHandle` (wrapping `thandle_t`) across the seam; borrow inward |
| `CRandomTerrain*` (`mRandomTerrain` / the `random_terrain` field) | `cm_landscape.h:153`, `cm_randomterrain.h:52` | **dropped** — §20 generation path (RMG-D1): `CreateRandomTerrain` is in the `#else` of `#ifdef DEDICATED` (`cm_terrain.cpp:170-188`), so `mRandomTerrain` stays `0`. No `random_terrain` field is added; `GetRandomTerrain()` is modeled as always-`None`. Shape-map entry in Divergences | (never, under DEDICATED) | — |
| `static CTerrainMap* TerrainMap` | `cm_terrainmap.cpp:14` | **dropped** — §20 (RMG-D2c/ruling 17): only writer `CM_TM_Create` is `#ifndef DEDICATED` (`RM_Mission.cpp:1503-1504`) | — | — |
| `noiseTable` / `noisePerm` | `cm_randomterrain.cpp:14-15` | **dropped** — §20 generation path (RMG-D1/RMG-D2c): the Perlin path is dead code and unreachable under DEDICATED | — | — |
| `Consonants[]`, `CRMPathManager::neighbor_x/y`, `CreateInstance::instanceID` | `cm_randomterrain.cpp:847+`; `RM_Path.h:172-173`; `RM_InstanceFile.cpp:140` | **dropped** — §20 generation path (RMG-D1): const/scratch on the never-constructed mission/path/instance objects | — | — |
| free `flrand`/`irand` global `holdrand` | `q_math.c:1432` | `mp_engine_core::Engine.common.rng: mp_qshared::QRand` — the engine's own q_math LCG instance (RMG-D2f/ruling 21). Exposed via `EngineHost::flrand`/`irand`. **No live RMG draw** under DEDICATED; only the golden-only `RMG_CreateSeed` uses it | `Rand_Init` (`q_math.c:1434`) | `&mut impl EngineHost` |
| `CRMArea*` — `mAreas` arena + `CRMInstance::mArea` | `RM_Area.h:74,80`; `RM_Instance.h:33` | **dropped** — §20 generation path (RMG-D1): `CRMAreaManager`/`CRMArea` are only constructed during mission spawn (never reached). The `AreaId` arena shape (RMG-D2g/ruling 21) is retained as a Divergences shape-map entry | — | — |
| `com_RMG`, `com_terrainPhysics` | `common.cpp:72`; `cm_landscape.h:267` | `EngineCvars` handles (fork-2). `com_RMG` is LIVE (gates `G_RMG_INIT`) | `Cvar_Get` at init | read via cvar accessor |
| `CCMLandScape::holdrand` | `cm_landscape.h:160` | `CmLandScape.holdrand: c_ulong` — an inline per-instance LCG field; seeded `0x89abcdef` in the live ctor (`cm_terrain.cpp:122`) and read by `get_rand_seed` (streamed, `sv_client.cpp:806`). `flrand`/`irand`/`rand_seed` (`cm_terrain.cpp:1548-1580`) are §20 within the class (no live caller). **Not** an external `Rng` type | `CCMLandScape` ctor (`cm_terrain.cpp:122`) | field; see RNG threading |

## Seam definition

RMG crosses **two** boundaries; nothing here crosses the *module* ABI (no
`#[repr(C)]` layout constraint — §F), so all types below are idiomatic.

**The host seam (ruling 11).** Every §F engine service Raven reached through a
file-scope global or `gi.`/`Com_` call — `Com_Printf`/`Com_Error`, cvar reads,
FS — is threaded as the one `EngineHost` services trait (trace, FS, print/error,
VM_Call, shared memory — plus the `flrand`/`irand` RNG services backed by
`Engine.common.rng`, RMG-D2f). **The trait lives in the pinned Stage-0 interface
crate `mp_host_interface` (`crates/mp/host-interface`, ruling 24 — docs cite real
paths); its exact method signatures are owned and frozen by that crate, not by
this doc** (a declared, still-unchecked Stage-0 prerequisite, `GOAL-engine.md`
Stage 0). This doc names only *which* `EngineHost` methods the RMG live surface
calls; freezing their signatures is that crate's Stage-0 work, and it is a hard
prerequisite of Wave 16 (Slice hooks). `Engine` implements the trait via a
split-borrow view struct; the referee injects a deterministic impl (DEC-09). §F
methods that touch a service take `&mut impl EngineHost`; the `CollisionWorld`
state is *not* a service and stays a separate threaded param (§B4). Under RMG-D1
the live host use is exactly three methods: `EngineHost` print (`Com_Printf` — the
RMG banner in `LoadMission`, `RM_Manager.cpp:106-107`), `EngineHost` FS/cvar reads
(the config/FS reads inside `CM_RegisterTerrain` construction), and
`EngineHost::flrand`/`irand` (the golden-only `RMG_CreateSeed`'s RNG draws).

**Handle types (§B5, layout-free).**

- `TerrainHandle` — a newtype over the rosetta's `thandle_t`
  (`type thandle_t = c_int`, `crates/native/types/src/lib.rs:65`); the ABI-crossing
  id the syscall returns (`GetTerrainId()`, `cm_landscape.h:220`; `mTerrainHandle`,
  `:139`). **Defined in `mp_engine_qcommon`** (a small `terrain_handle.rs` beside
  `collision_world.rs`): `register_terrain` constructs it and
  `RmManager::set_landscape` consumes it; the crate edge runs `rmg → qcommon` only
  (never the reverse), so the shared handle must live in `qcommon` (or lower) —
  the mechanical consequence of the settled dependency direction. **LIVE.**
- **No random-terrain handle** (RMG-D2e/ruling 21). Moot under RMG-D1 — the
  `CmLandScape.random_terrain` field is §20-dropped (never constructed), so no
  handle and no field exist; `GetRandomTerrain()` models as always-`None`.
- `AreaId` — the §B5 index newtype for the `CRMAreaManager` arena (RMG-D2g).
  Generation-path only under RMG-D1; retained as a Divergences shape-map entry,
  never in the live seam.

The clipmap the terrain hangs off is the existing STATE-D2 `CollisionWorld`
(`Engine.cm`, `collision_world.rs:10`) — there is no separate `ClipMap` type.

### A. Server → RMG (the reachable arms)

The game module reaches RMG through the vmcalls hitting the syscall switch
(`oracle/codemp/game/g_public.h:571-573`, `g_syscalls.c:1468-1481`). Inside the
switch the server calls the `RmManager` methods below; under RMG-D1 the
mission-driving methods run only up to `LoadMission`'s early-out. Frozen pub API on
`mp_engine_rmg`:

```rust
impl RmManager {
    /// `CRMManager::CRMManager` — RM_Manager.cpp:34
    pub fn new() -> Self;
    /// `CRMManager::SetLandScape` — RM_Manager.cpp:79 (stores the handle;
    /// mTerrain = GetRandomTerrain() is always None under DEDICATED, RMG-D1)
    pub fn set_landscape(&mut self, land: TerrainHandle);
    /// `CRMManager::LoadMission` — RM_Manager.cpp:96. Prints the RMG banner
    /// (guarded by `#ifndef FINAL_BUILD`, RM_Manager.cpp:105-108) then early-outs
    /// `false`: mTerrain is always NULL
    /// under DEDICATED (RMG-D1, RM_Manager.cpp:110-113) — never constructs a mission.
    pub fn load_mission(&mut self, cm: &mut CollisionWorld, host: &mut impl EngineHost, is_server: bool) -> bool;
    /// `CRMManager::SpawnMission` — RM_Manager.cpp:391. **Unreachable under
    /// DEDICATED** (load_mission returns false, sv_game.cpp:1632-1634): the body
    /// drives the §20-dropped CRMMission::Spawn (RMG-D1). Kept as a
    /// dead-under-DEDICATED stub so the ported `if load_mission { spawn_mission }`
    /// syscall arm compiles; it must never execute.
    pub fn spawn_mission(&mut self, cm: &mut CollisionWorld, host: &mut impl EngineHost, is_server: bool) -> bool;
    /// `CRMManager::GetAutomapSymbolCount` — RM_Manager.cpp:413 (returns 0 under
    /// DEDICATED — nothing calls AddAutomapSymbol, RM_Manager.cpp:41)
    pub fn automap_symbol_count(&self) -> i32;
    /// `CRMManager::GetAutomapSymbol` — RM_Manager.cpp:418
    pub fn automap_symbol(&self, index: i32) -> Option<&RmAutomapSymbol>;
}
```

**Owned-state field — the lazy-init flag (rendering of ruling 12).** `RmManager`
carries one private field `initialized: bool` (Default `false`), the concrete
rendering of ruling 12's "Raven's own initialized flag" for
`TheRandomMissionManager`. Raven's flag *is* the `!TheRandomMissionManager` null
check at the `G_RMG_INIT` arm (`sv_game.cpp:1627-1629`); the field flips to `true`
**at that syscall arm**, not inside any `RmManager` method. Because
`CRMManager::CRMManager` only zeroes members (`RM_Manager.cpp:34-42`), it is
Default-equivalent: `RmManager::default()` and Raven's lazy `new CRMManager`
collapse to one construction, and the lazy step is only the flag flip. Frozen:
porters add the field and flip it at the Wave-20 syscall arm.

**Seam deviation — the added `cm: &mut CollisionWorld` parameter (not a design
change).** Raven's `LoadMission`/`SpawnMission` take only `qboolean IsServer`
(`RM_Manager.cpp:96,391`) and reach the landscape through the `cmg.landScape` file
global. Per §B (no hidden globals), `RmManager` owns **only** a `TerrainHandle`;
the `CCMLandScape` data lives in `CollisionWorld` (STATE-D2, `collision_world.rs:10`).
So both methods take the owning `CollisionWorld` explicitly to resolve that handle
— the state-threading form (§B4) of Raven's global reach. (This is why
`mp_engine_rmg` needs the `mp_engine_qcommon` edge — see "Crate dependencies".)

`rmAutomapSymbol_t` is an existing ABI type (`oracle/codemp/client/client.h:149`,
`MAX_AUTOMAP_SYMBOLS = 512` `:151`); the rosetta ported it in
`mp_engine_client` (`crates/mp/engine/client/src/client/rm_automap_symbol_t.rs:9`).
Per **RMG-D2d** (ruling 21) it **relocates to `mp_qshared`** — which
`mp_engine_rmg` already depends on — so `RmManager::automap_symbol` returns
`Option<&RmAutomapSymbol>` directly, with **no** `rmg → mp_engine_client` edge.
The live automap serializer is `SV_WriteRMGAutomapSymbols` (`sv_client.cpp:668`),
which walks the count/get pair (count is `0`).
`CRMManager::WriteAutomapSymbols` (`RM_Manager.cpp:424`) is commented-out dead
code (§20); `CRMManager::ProcessAutomapSymbols` (`RM_Manager.cpp:442`) is a
`static` client-side reader, dead under DEDICATED (§20).

**The frozen `impl RmManager` above is the *complete* live surface — the twelve
other declared `CRMManager` methods (of the 13 the review flagged) are §20
zero-caller drops, not omissions; the 13th, `GetLandScape`, is live (see below).**
A grep of
`TheRandomMissionManager->` finds no invocation of `SetCurPriority`, `GetTerrain`,
`GetCurPriority`, `Preview`, `IsMissionComplete`, `HasTimeExpired`,
`CompleteObjective`, `CompleteMission`, `FailedMission`, or `UpdateStatisticCvars`
anywhere in codemp (zero callers), and `GetMission`/`AddAutomapSymbol` are called
only from the §20-dropped generation path (`RM_Instance*`/`RM_Path.cpp`;
`RM_Manager.cpp:400-410`). All are §20-dropped (Divergences), the same reasoning as
`WriteAutomapSymbols`/`ProcessAutomapSymbols`. The one exception is
`GetLandScape` (`RM_Manager.h:39`), which **is** live (the snapshot read) — it is
absent from the impl block only because its Rust signature is unsettled (**RMG-Q9**,
§C), not because it is dropped.

### B. RMG → qcommon terrain (the free-function entry points)

`cm_landscape.h:245-265` declares the C entry points the server/clipmap call. The
frozen `mp_engine_qcommon` surface under RMG-D1:

```rust
/// `CM_RegisterTerrain` — cm_load.cpp:1036. Constructs (or, on repeat
/// registration, get-or-creates — cm_load.cpp:1040-1044) the CmLandScape under
/// DEDICATED. The random-terrain arm (cm_terrain.cpp:178) is never taken (RMG-D1).
pub fn register_terrain(cm: &mut CollisionWorld, host: &mut impl EngineHost, config: &str, server: bool) -> TerrainHandle;
/// `RMG_CreateSeed` — cm_randomterrain.cpp:1008 (draws the engine's q_math LCG via
/// EngineHost::flrand/irand — RMG-D2f; **zero live callers** in codemp, kept as a
/// golden-only helper the harness pins against Engine.common.rng)
pub fn rmg_create_seed(host: &mut impl EngineHost) -> (String, u32);
```

**`create_random_terrain` is §20-dropped, not a seam entry (RMG-D1).** Raven
`CreateRandomTerrain` (`cm_terrain.cpp:1688`) is only called from the `#else` of
`#ifdef DEDICATED` (`cm_terrain.cpp:178`), so it is dead code on the dedicated
engine. It is recorded in Divergences, not ported; there is no
`create_random_terrain` in the frozen surface.

**`CM_TerrainPatchIterate`, the twelve `cm_landscape.h:247-258` area `CM_*`
wrappers, and the `CCMLandScape` area methods are all §20-dropped under RMG-D1.**
(`CM_InitTerrain`, `cm_landscape.h:246`, is **LIVE** — the constructor
`CM_RegisterTerrain` calls at `cm_load.cpp:1048`, folded into `register_terrain`
below — and is **not** in this dropped set.) The
free-function wrappers (`CM_GetWorldHeight`/`CM_FlattenArea`/`CM_CarveBezierCurve`/
`CM_SaveArea`/`CM_FractionBelowLevel`/`CM_AreaCollision`/the `CArea`-cursor family/
`CM_CircularIterate`, `cm_landscape.h:247-258`, defined `cm_terrain.cpp:1633-1685`)
each grep-resolve to
their declaration + wrapper with **no call site**, so they are zero-caller drops
(`SV_LoadMissionDef`, `:262`, is declared-never-defined — also dropped). The
`CCMLandScape` **methods** they forward to — `FlattenArea` (`cm_terrain.cpp:1312`),
`SaveArea` (`:1128`), `GetWorldHeight` (`:1011`), `AreaCollision`/`GetFirstArea`/
`GetNextArea`/`FractionBelowLevel`/`CarveBezierCurve`/`GetFirst|Player|NextObjectiveArea`
— were previously live only through the generation path (`CRMPathManager`,
`CRMInstance`, `CRMMission::Spawn`, the `RM_Terrain.cpp` chain). **Under RMG-D1
every one of those callers is §20-dropped**, so all of these methods lose their
last live caller and are §20-dropped too. `CM_TerrainPatchIterate`
(`cm_terrain.cpp:1628,997`) likewise had only the renderer (DEC-01) and the
dropped `RM_Terrain.cpp` chain as callers. The `CArea` class (`cm_landscape.h:42`)
appears only as the argument of these now-dead methods; it is dead-surface too
(its Rust name is RMG-Q7, Open questions — but it names no live type). Recorded in
Divergences.

**Repeat-registration / refcount (RMG-D2c/DEC-01).** Raven's `CM_RegisterTerrain`
refcounts: a second call with `cmg.landScape` already set `IncreaseRefCount()`s and
returns the existing landscape (`cm_load.cpp:1040-1044`; `mRefCount = 1` at ctor,
`cm_terrain.cpp:130`). The only consumer of that count is `CM_ShutdownTerrain`
(`cm_load.cpp:1073-1077`), whose only caller is the renderer (`tr_terrain.cpp:1050`,
DEC-01); the dedicated server frees unconditionally at teardown (`delete
cmg.landScape`, `cm_load.cpp:800-809`). So `mRefCount` has no live reader here and
is §20-dropped (renderer-only); `register_terrain` reproduces only the observable
seam behavior — **return the existing `TerrainHandle` on repeat registration**
(a get-or-create on the owned `Option<CmLandScape>`, matching
`cm_load.cpp:1040-1044`).

### C. Landscape runtime surface (live readers of the constructed landscape)

Beyond construction (Seam-B) the constructed `CmLandScape` has **two live
external readers**, both owned by *this* subsystem (RMG-D2a: the `cm` C-track
packets exclude `CCMLandScape`, so its methods port here — not in a `cm` packet,
not in another doc). The C++ signatures + oracle cites are given as ground truth;
their **exact idiomatic Rust signatures are deferred to Open questions**
(porting-rules §C — a per-method idiom choice this doc's design session has not
settled), so this subsection does not yet freeze:

1. **Snapshot/download read** (server, `sv_client.cpp:779-806`, reached as
   `TheRandomMissionManager->GetLandScape()->…`) — scope **settled** (In-scope
   item 3); only the Rust signatures are open (**RMG-Q9**):
   - `byte *GetHeightMap(void) const` (`cm_landscape.h:218`; §F.19-UB bytes)
   - `byte *GetFlattenMap(void) const` (`cm_landscape.h:219`)
   - `const int GetRealArea(void) const` (`cm_landscape.h:211`)
   - `unsigned long get_rand_seed(void)` (`cm_landscape.h:239`)
2. **Per-frame terrain collision** (the `cm-trace`/`cm-test` C-track packets,
   `cm_trace.cpp:283,760,789,997,1374` + `cm_test.cpp:285-289`, non-`BSPC`, gated
   `com_terrainPhysics->integer && cmg.landScape && CONTENTS_TERRAIN`) — this path
   is LIVE but was **not** in RMG-D1's three-item live enumeration, so its
   inclusion-in-scope (with signatures) is **RMG-Q8**:
   - `void PatchCollide(traceWork_s *tw, trace_t &trace, const vec3_t start, const vec3_t end, int checkcount)` (decl `cm_landscape.h:175`, def `cm_terrain.cpp:600`)
   - `float WaterCollide(const vec3_t begin, const vec3_t end, float fraction) const` (`cm_landscape.h:178` / `cm_terrain.cpp:836`)
   - `const vec3pair_t &GetBounds(void) const` (`cm_landscape.h:112/199`),
     `const float GetPatchScalarSize(void) const` (`:207`),
     `float GetWaterHeight(void) const` (`:232`),
     `int GetWaterContents(void) const` (`:233`),
     `int GetWaterSurfaceFlags(void) const` (`:234`)

`GetPatch`/`GetTerxelLocalCoords`/`SetShaders`/`CarveLine`/`CalcRealCoords` have
**no live engine caller** (grep-resolved to the renderer `tr_terrain.cpp`, DEC-01,
and the §20 `RM_Terrain.cpp` chain), so they are §20/renderer-only or
private-internal (§A1), never seam.

**The `RmManager` landscape accessor (`GetLandScape`, `RM_Manager.h:39`) is
required but its Rust form is open (RMG-Q9).** The snapshot read above reaches the
landscape through `TheRandomMissionManager->GetLandScape()`; Raven returns the
cached `mLandScape` member, but per the Divergences the port stores **only** the
`TerrainHandle` (`mLandScape` NOT stored), so the accessor must resolve that handle
against the owning `CollisionWorld`. The exact form — return the `TerrainHandle`
for the caller to resolve, vs. borrow `&CmLandScape` via a threaded
`&CollisionWorld` — is a §C idiom choice not settled by any decision; the frozen
`impl RmManager` block (Seam-A) omits it pending **RMG-Q9**.

## Decisions

**RMG-D1 — RMG generation is dead under DEDICATED; §20-drop the generation
path.** Per **ruling 25** (user, 2026-07-09, `engine-fork-discovery.md`). We port
only the reachable surface — the two RMG syscall arms, `CCMLandScape` construction
under DEDICATED, and `LoadMission`'s early-out — and §20-drop the entire
generation subtree. Because on the DEDICATED build `CreateRandomTerrain`'s only
call site is inside the `#else` of `#ifdef DEDICATED` (`cm_terrain.cpp:170-188`,
call `:178`), so `mRandomTerrain` stays `0` (`:169`), `SetLandScape` sets
`mTerrain = NULL` (`RM_Manager.cpp:82`), `LoadMission` early-outs at
`RM_Manager.cpp:110-113` before constructing the mission (`:135`), and
`G_RMG_INIT` never reaches `SpawnMission` (`sv_game.cpp:1632-1634`). §20-dropped
with dead-under-DEDICATED notes: `CreateRandomTerrain`, `CRandomTerrain::Generate`
(and the whole `CRandomTerrain`/`CPathInfo` class), `CRMMission::Spawn`/`PreSpawn`/
`Smooth`/`PlaceBridges` (and `Load`/`ParsePaths`/`ParseRivers` — all downstream of
the early-out), the `CRMInstance` hierarchy, `CRMInstanceFile`, `CRMObjective`,
`CRMPathManager`, `CRMArea`/`CRMAreaManager`, the `CmLandScape.random_terrain`
field, and the Seam-B `create_random_terrain` entry. **The live surface** = the
reachable syscall arms + `CCMLandScape` construction under DEDICATED + `LoadMission`'s
early-out behavior; the goldens pin exactly that (an RMG mission on the dedicated
server fails identically to C — no mission spawns, the default un-generated terrain
is streamed). Rejected porting the generation path: it is unreachable dead code on
the shipped DEDICATED build (§20). `tools/rmg-oracle` compiles the oracle TUs
**with `DEDICATED` defined** to match (Verification strategy).

**RMG-D2 — All rulings 11-22 stand, applied to the reduced live surface;
dropped-path classes keep their shape-map entries marked §20.** The prior §F
decisions carry forward verbatim; where RMG-D1 makes their subject dead code, the
decision still governs the shape recorded in Divergences (the §20 shape-map),
not a ported implementation. Recorded with stable sub-IDs so the body's cites
resolve:

- **RMG-D2a — Fold the qcommon terrain twins into this doc** (ruling 16,
  `:143-146`). `CCMLandScape`, `CCMPatch`, `CCMHeightDetails` (live construction),
  and — as dropped shape-map — `CRandomTerrain`, `CTerrainMap`, `CPathInfo`,
  `CArea` are owned by *this* subsystem; the `cm` C-track packets exclude them.
  Because the tree cannot be designed apart from them. Rejected a separate qcommon
  doc.
- **RMG-D2b — State on direct `Engine` fields; services via the one `EngineHost`
  trait** (rulings 12 `:127-131`, 11 `:121-126`). `TheRandomMissionManager` →
  `mp_engine_core::Engine.rmg: RmManager` (no `Option`/`Box`; lazy-init via
  Raven's own flag); const tables → `const`; cvars → `EngineCvars`; every engine
  service (FS, print/error, cvar, trace, `flrand`/`irand`) → `&mut impl
  EngineHost`. Rejected globals/sub-structs (§B3).
- **RMG-D2c — §20-drop the four ruling-17 dead-surface items** (ruling 17
  `:147-152`): (a) `mCurObjective` (`RM_Manager.cpp:16`); (b) `noiseTable`/
  `noisePerm` — dead Perlin path (`CM_NoiseInit` `#if 0`, `cm_randomterrain.cpp:17-28`);
  (c) the `RM_Terrain.cpp` client-model chain; (d) `CTerrainMap` (its only ctor
  `CM_TM_Create` is `#ifndef DEDICATED`, `RM_Mission.cpp:1503-1504`). All recorded
  in Divergences. Rejected porting them: no live DEDICATED caller.
- **RMG-D2d — `rmAutomapSymbol_t` relocates to `mp_qshared`** (ruling 21 part 1).
  The rosetta ported it in `mp_engine_client`
  (`crates/mp/engine/client/src/client/rm_automap_symbol_t.rs:9`); it moves to
  `mp_qshared` (already a dependency), so `RmManager::automap_symbol` names it
  directly — no `rmg → mp_engine_client` edge. **LIVE** (the automap seam pair,
  returning count 0, survives under RMG-D1). Rejected the reverse edge.
  **Concrete destination path — a type-rosetta datum, not a design choice.** Per
  the type-rosetta discipline (`engine-fork-discovery.md:96-107`: porters import a
  type from its rosetta path and never declare it; a relocation regenerates
  `out/engine/type-rosetta.tsv`), the authoritative post-move path is the rosetta
  row for this type, updated when it lands in `mp_qshared`. Following `mp_qshared`'s
  existing one-type-per-file `src/shared/` layout (e.g. `collision.rs`,
  `connstate.rs`), that mechanical placement is
  `crates/mp/qshared/src/shared/rm_automap_symbol_t.rs`, imported as
  `mp_qshared::RmAutomapSymbol`. The porter imports by name from the rosetta, not
  by hand-picking a path.
- **RMG-D2e — No `RandomTerrainHandle` newtype** (ruling 21 part 2). `CRandomTerrain`
  was to be a single owned `CmLandScape.random_terrain: Option<RandomTerrain>` with
  no handle. Under RMG-D1 that field is §20-dropped (never constructed);
  `GetRandomTerrain()` models as always-`None`. The decision governs the dropped
  shape-map entry. Rejected a marker/unit handle.
- **RMG-D2f — The engine owns its own q_math LCG as `Engine.common.rng:
  mp_qshared::QRand`, exposed via `EngineHost::flrand`/`irand`** (ruling 21 part 3).
  `mp_qshared` gains a `QRand` type (the stateful LCG the game tier models as
  `bg_channel::rng::Rng`, `crates/mp/game/src/bg_channel/rng.rs`); the engine holds
  a distinct instance (`engine.rs:22`, `common/common.rs:20`). **Under RMG-D1 no
  live RMG path draws it** — its only RMG consumer is the golden-only
  `RMG_CreateSeed` (`cm_randomterrain.cpp:1008`, zero live callers), pinned by
  golden #1. The engine service still exists (it stands for the wider engine).
  Rejected reaching `mp_game`'s LCG: `mp_engine_qcommon` must not depend on `mp_game`.
- **RMG-D2g — `CRMArea*` → `AreaId` + arena owned by `CRMAreaManager`, per §B5**
  (ruling 21 part 4). `AreaId` (a `u32` index newtype rendered like `EntityId`),
  `mAreas` → owned `Vec`, `mArea` → an `AreaId`, `GetArea` → arena lookup. **Under
  RMG-D1 the area classes are generation-path dead** (only constructed during
  mission spawn); the arena shape is a Divergences shape-map entry, never a live
  seam type. Rejected raw `CRMArea*`/`Rc` (§B5).
- **RMG-D2h — Stored pointers into state owned elsewhere are dropped; the owner is
  threaded** (ruling 21 part 5, §B3). One recurring shape, applied to every
  occurrence the survey found. **Four** fields matched: `CCMPatch::owner:
  CCMLandScape*` (`cm_landscape.h:93`) — **LIVE**, on the live patch-collision
  build (`GetAdjacentBrushY`, `cm_terrain.cpp:246-256`); and three generation-path
  fields now §20-dropped with their owners — `CRandomTerrain::mLandScape`
  (`cm_randomterrain.h:56`), `CRMMission::mLandScape:CRandomTerrain*`
  (`RM_Mission.h:64`), `CRMPathManager::mTerrain:CRandomTerrain*` (`RM_Path.h:175`).
  Per §B3 none is a safe Rust field; each is dropped and the owner threaded (§B4).
  For the live `CCMPatch::owner`, the owning `CmLandScape` is threaded into the
  patch-build methods. Rejected `Rc`/raw back-pointers (§B3).
- **RMG-D2i — Prior §F shape (closed-hierarchy enum) + verification stand.**
  `CRMInstance` base+four-subclass tree → one `RmInstance` enum (factory
  `CreateInstance` → `match` on the GP2 group name), per §17 — the hierarchy is
  provably closed (`RM_InstanceFile.cpp:158-178`); the dead `"npc"` branch
  (`:162-166`) is §20-dropped. `CRMPathManager` vectors → `Vec`, `CRMInstanceFile`
  GP2 members → arena borrows. **All generation-path** under RMG-D1 → recorded in
  Divergences as §20 shape-map, not ported. Rejected a `dyn` arena. Verification is
  the §18/DEC-09 TU-harness track (Verification strategy).

**RMG-D3 — All prior settled decisions stand.** Every earlier settled decision —
the first design-session §F choices and the DEC-01 (renderer deferred) / DEC-04
(strict per-mode) / DEC-09 (engine verification) ledger deps — carries forward
unchanged, except where RMG-D1 reduces its live scope (in which case its subject
becomes a §20 shape-map entry rather than ported work). No prior decision is
re-litigated here.

## Verification strategy

§F / DEC-09 TU-harness track (RMG-D2i), scoped to the RMG-D1 live surface:

- **Harness** `tools/rmg-oracle/` — compile the unmodified oracle TUs
  (`cm_terrain.cpp`, `cm_randomterrain.cpp`, `RM_Manager.cpp`) standalone against
  stub headers (oracle never edited, §18), **with `DEDICATED` defined** (RMG-D1) so
  the compiled behavior matches the shipped engine: `CreateRandomTerrain` is
  compiled out of the ctor's reachable path, `LoadMission` early-outs, no mission
  spawns. The dumper registers terrain with a fixed config, runs `SetLandScape` +
  `LoadMission` (observing the `false` return + the `#ifndef FINAL_BUILD` banner,
  `RM_Manager.cpp:105-108` — the harness compiles the non-FINAL_BUILD TU, so the
  banner is present in both the oracle and the port), and streams the resulting
  landscape as `SV_SendClientGameState` would. The referee injects a deterministic
  `EngineHost` impl (ruling 11) for FS/print and the `flrand`/`irand` RNG services
  (seeding a fixed `Engine.common.rng`, RMG-D2f).
- **Goldens** (committed, so `cargo test` needs no C++):
  1. `RMG_CreateSeed` seed-string + hash streams for a fixed `Engine.common.rng`
     (`QRand`) seed — the golden-only helper (zero live callers) pinning the engine
     LCG via `EngineHost::flrand`/`irand`. **Kept** (ruling 25 drops #2/#3 only).
  2. **Dropped** (RMG-D1) — the post-`Generate` heightmap/flatten bytes: no
     generation runs under DEDICATED.
  3. **Dropped** (RMG-D1) — the automap-symbol list after a full mission spawn:
     no spawn runs; the count is `0`.
  4. **The dedicated-server outcome golden** (RMG-D1, "pin exactly that"):
     `register_terrain` builds the `CmLandScape` under DEDICATED; `G_RMG_INIT` runs
     `SetLandScape` + `LoadMission → false` (no `SpawnMission`); and the snapshot
     stream reproduces `GetFlattenMap()` (memset-0, `cm_terrain.cpp:161`),
     `get_rand_seed()` (`0x89abcdef`), and `count = 0` automap symbols —
     byte-for-byte matching the DEDICATED-built oracle.
     **§F.19 (UB) — the streamed `GetHeightMap()` bytes are excluded from the
     byte-compare.** `mHeightMap` is allocated by the **non-zeroing** `Z_Malloc`
     overload (`bZeroit` defaults `qfalse`, `qcommon.h:787`) at `cm_terrain.cpp:157`
     and is **never written** under DEDICATED (no image load, no generation), so the
     oracle streams *uninitialized heap* — reading it is UB and its bytes are
     non-deterministic. Per §F.19 the heightmap region is kept **out of** the
     committed byte-compare (the dumper elides it, with a comment at the site); only
     the deterministic flatten/seed/count streams are pinned. The observable the
     golden proves — no mission spawns, the default un-generated terrain is streamed —
     does not depend on the UB bytes.
- **Determinism anchor**: the streamed seed is the ctor-seeded `holdrand`
  (`cm_terrain.cpp:122`, never re-seeded under DEDICATED); `RMG_CreateSeed`'s LCG is
  bit-exact (`holdrand*214013 + 2531011`, `result = holdrand >> 17`,
  `q_math.c:1445-1466`). Any drift shows as a first-diverging RNG draw or (in the
  deterministic flatten/seed/count streams, not the §F.19-excluded heightmap) stream
  byte.
- **No OpenJK peer** (RMG-D2i) — OpenJK dropped RMG entirely
  (`docs/plans/2026-07-08-mp-engine-build-out.md:425-428`), so the engine-vs-engine
  A/B square cannot exercise these paths. A hard constraint, not a choice.

## Slice hooks

- **Wave 16** (`plan §"RMG (113 fns, wave 16)"` — the subsystem-*completion* wave,
  i.e. the max wave over RMG fns, not a per-fn wave). **Producible now** (frozen
  seams, no open question): the reduced live tree as one §F subsystem — `RmManager`
  (lifecycle through the early-out + automap seam, Seam-A), `CmLandScape`/`CmPatch`/
  `CmHeightDetails` construction, `TerrainHandle`, `register_terrain`,
  `rmg_create_seed` (Seam-B). A dry-run skeleton legitimately covers exactly these.
  **NOT yet producible — deferred to RMG-Q8/Q9, do not transcribe until they
  resolve:** the constructed landscape's live runtime surface (Seam-B §C) — the
  snapshot-read accessors (RMG-Q9), the `RmManager` landscape accessor + its
  stored-handle field type (RMG-Q9), and the per-frame collision methods
  `PatchCollide`/`WaterCollide` + water/bounds accessors (RMG-Q8). Both their Rust
  signatures **and** their wave placement are open: per
  `engine-port-order.tsv` those collision methods are demanded at waves 0–4 by the
  C-track clipmap-trace callers, *before* the wave-15 `CmLandScape` constructor and
  wave-17 `CM_RegisterTerrain` (RMG-Q8) — so "land with `CmLandScape` in Wave 16"
  is one candidate the design session weighs, not a settled placement. The
  generation subtree lands only as §20 Divergences shape-map entries (RMG-D1), not
  porter code. **Hard prerequisites frozen first:** the type-rosetta entries for
  `rmAutomapSymbol_t` (relocated to `mp_qshared`, RMG-D2d) / `thandle_t`, and the
  `EngineHost` trait — defined by the Stage-0 crate `mp_host_interface`
  (`crates/mp/host-interface`, ruling 24; still an unchecked `GOAL-engine.md` Stage-0
  box), with its `flrand`/`irand` RNG services backed by `Engine.common.rng: QRand`
  (RMG-D2f). Without the frozen `EngineHost` signatures the live host call sites
  (`register_terrain`'s FS/print, `rmg_create_seed`'s RNG) cannot be written as
  non-stub bodies (`GOAL-engine.md:24-28`).
- **Wave 20** (`SV_GameSystemCalls`): the RMG syscall arms wire to the frozen
  seams — `G_RMG_INIT` → Seam-A `RmManager` methods (`sv_game.cpp:1624-1638`),
  `G_CM_REGISTER_TERRAIN` → Seam-B `register_terrain` (`sv_game.cpp:1640-1641`).
  `G_SET_ACTIVE_SUBBSP` → `SV_SetActiveSubBSP` (`sv_game.cpp:1620-1622`) is
  out-of-scope clipmap wiring (Non-goals), not a seam edge. Needs the `Engine.rmg`
  field (ruling 12) and `CollisionWorld.land_scape`. The `G_RMG_INIT` arm checks
  `!rmg.initialized`, sets it `true` in place of Raven's `new`, then calls
  `set_landscape` / `load_mission` / (the guarded, never-reached) `spawn_mission` —
  the flip is here, not in any method. `load_mission` returns false (RMG-D1), so
  `spawn_mission` is never entered.
- **Wave 22** (`SV_SpawnServer`): `CM_RegisterTerrain` on the map-load path; needs
  Seam-B frozen.

## Open questions

Ruling 25 (RMG-D1) shrank the live surface but left three questions for a design
session before FROZEN: one naming (RMG-Q7) and two on the constructed landscape's
live runtime surface (RMG-Q8, RMG-Q9), which the dry-run gate surfaced. The
ownership/scope of that surface is **resolved in place** (RMG-D2a: owned here, not
another doc — `## Seam definition` §C); what remains on the runtime pair is a scope
reconciliation with RMG-D1's enumeration plus the wave-placement inversion (RMG-Q8)
and the exact idiomatic Rust signatures + stored-handle field type (RMG-Q9)
(porting-rules §C: per-method/per-member idiom choices, never agent-invented). The
`EngineHost` trait signatures the live host call sites need are **not** an RMG open
question — they are owned by the pinned Stage-0 crate `mp_host_interface` (ruling
24) and tracked as a `GOAL-engine.md` Stage-0 prerequisite (Slice hooks / Seam
definition), a cross-crate dependency this doc points at rather than decides.

- **RMG-Q7 — Rust names for the two distinct `Area` classes.** The doc's naming
  convention gives every `RM_*.h` class an `Rm` prefix (`RmManager`, `RmMission`,
  `RmInstance`) and reserves `Cm` for the qcommon/collision classes (`CmLandScape`,
  `CmPatch`). Two *separate* oracle classes named around "Area" need Rust names:
  1. `CRMArea` (`oracle/codemp/RMG/RM_Area.h:17`) — the RMG arena element
     (RMG-D2g). The Divergences shape-map calls its Rust type `CmArea`, which
     contradicts the `Rm`-prefix convention (by that rule it should read `RmArea`).
     Whether `CmArea` is a typo for `RmArea` or deliberate is unsettled.
  2. `CArea` (`oracle/codemp/qcommon/cm_landscape.h:42`) — a distinct qcommon class
     (NOT the same as `CRMArea`), the `area` argument of the now-dead
     `FlattenArea`/`SaveArea`/`GetWorldHeight` methods. Never named in the doc.
  Under RMG-D1 **both area classes are §20 dead-surface** (generation-path only):
  their names appear solely as labels in Divergences shape-map entries, never in a
  live seam type. That lowers the urgency but does not settle the question — a
  rename is a design decision (porting-rules §C: specific renames are decided in
  discussion), and the collision (a qcommon `CArea` fitting neither `CCM*→Cm*` nor
  `CRM*→Rm*`) cannot be resolved from the convention alone. Escalate to a design
  session. **Freezes NO `## Seam definition` pub API** — the frozen surface
  (`register_terrain`, `rmg_create_seed`, the `RmManager` methods) names no area
  type, so a skeleton is producible without it. Provisional handling: treat every
  `CmArea` in Divergences as a non-final placeholder for `CRMArea`'s element.
  **The qcommon `CArea` is emitted as neither a marker nor a stub.** An earlier
  draft said to "leave a `//TODO: Port CArea` marker"; that is withdrawn — it
  contradicts `GOAL-engine.md`'s hard, user-directed no-marker ground rule
  (`GOAL-engine.md:24-28`: no `TODO`/`FIXME` markers at any commit; `grep` stays
  empty) and the identical rule in `docs/plans/2026-07-08-mp-engine-build-out.md`.
  The contradiction is resolved by that standing rule plus the doc's own settled
  fact that `CArea` is §20 dead-surface (it appears only as the argument of the
  now-dead `FlattenArea`/`SaveArea`/`GetWorldHeight` methods, Seam-B; it names no
  live seam type). Dead surface is simply **dropped, not ported** (§20) — so no
  `CArea` Rust type, marker, or stub is emitted at all, and RMG-Q7 leaves nothing
  for a porter to name on the live path. `CArea`'s Rust name is decided only if
  the generation path is later revived; until then the question is moot in
  practice (it produces no code).

- **RMG-Q8 — Is the per-frame terrain-collision surface in the ported live scope,
  and with what signatures?** The `CmLandScape` collision methods `PatchCollide`
  (`cm_landscape.h:175`, def `cm_terrain.cpp:600`) and `WaterCollide` (`:178` /
  `:836`) — plus the water/bounds accessors `GetWaterContents`/`GetWaterSurfaceFlags`/
  `GetWaterHeight`/`GetBounds`/`GetPatchScalarSize` (`:233`/`:234`/`:232`/`:112`/`:207`)
  — are called **live, per frame**, from the `cm-trace`/`cm-test` C-track packets
  (`cm_trace.cpp:283,760,789,997,1374`; `cm_test.cpp:285-289`, non-`BSPC`, gated
  `com_terrainPhysics->integer && cmg.landScape && CONTENTS_TERRAIN`) — this is the
  server collision path, **not** generation-path code. Ownership is **settled**
  (RMG-D2a: the `cm` C-track packets exclude `CCMLandScape`, so these port here, in
  Wave 16). What is **not** settled: RMG-D1's live-surface enumeration lists only
  three items (syscall arms + construction + the snapshot read) and **omits this
  collision path** — expanding that enumeration amends a user-settled decision
  (ruling 25). And the exact idiomatic Rust signatures (does `PatchCollide` take a
  threaded `&mut CollisionWorld` for the `traceWork_s`/`trace_t` C-track types? does
  `trace_t` become a return value per §C7?) are a per-method §C idiom choice.
  Escalate both. Almost certainly the resolution is "yes, port here alongside
  construction" — but the count reconciliation and signatures are the user's, not an
  agent's. **Freezes the `## Seam definition` §C collision list** once settled.
  **Port-order ground truth the design session must weigh (the tool is the sole
  order authority, `GOAL-engine.md:29-31`).** Per
  `tools/closure-prototype/out/engine/engine-port-order.tsv` the collision methods
  and their signature-pinning callers land *far earlier* than the RMG unit's
  wave-16 completion, not "with `CmLandScape` in Wave 16": `CCMLandScape::WaterCollide`
  is wave 0, `CCMLandScape::PatchCollide` wave 3; their C-track callers
  `CM_TraceThroughTerrain` (`cm_trace.cpp:703`) wave 4, `CM_TestInLeaf`
  (`cm_trace.cpp:262`) wave 5, and `CM_PointContents` (`cm_test.cpp:224`, which
  reads `GetWaterContents`/`GetWaterHeight`) wave 7 — **all before** the
  `CCMLandScape` constructor (`cm_terrain.cpp:116`, wave 15), `CM_RegisterTerrain`
  (`cm_load.cpp:1036`, wave 17), and the RMG subsystem's completion at wave 16.
  So the raw dependency order demands these methods (and thus their frozen Rust
  signatures) at waves 0–4, an inversion against landing the whole terrain twin as
  one wave-16 §F unit. This resolves hole "before/after Wave 16" as ground truth —
  the callers are **before** — but the ordering inversion is precisely part of the
  scope reconciliation the design session owns (it may split the collision methods
  into the early clipmap-trace waves rather than the wave-16 §F unit); an agent
  does not choose the split.

- **RMG-Q9 — Exact Rust signatures for the landscape read surface + the `RmManager`
  accessor that reaches it.** Scope is **settled** (In-scope item 3 blesses the
  snapshot read `GetHeightMap`/`GetFlattenMap`/`GetRealArea`/`get_rand_seed`,
  `cm_landscape.h:218/219/211/239`, streamed at `sv_client.cpp:779-806`); only the
  idiomatic Rust forms are open (`byte*` → `&[u8]`? `get_rand_seed`'s
  `unsigned long` return width?), a §C per-type choice. Coupled to it: Raven reaches
  those accessors via `TheRandomMissionManager->GetLandScape()` (`RM_Manager.h:39`,
  returning cached `mLandScape`), but the port stores **only** the `TerrainHandle`
  (`mLandScape` NOT stored, Divergences), so the accessor must resolve the handle
  against the owning `CollisionWorld`. The exact form — return the `TerrainHandle`
  vs. borrow `&CmLandScape` through a threaded `&CollisionWorld` — is undecided; the
  frozen `impl RmManager` (Seam-A) omits it pending this. **Freezes the `RmManager`
  accessor row + the `## Seam definition` §C read list** once settled. The accessor
  is required (the snapshot path cannot compile without it) — this is a signature
  gap, not a scope gap.
  **Includes the stored-handle field's name and type.** `set_landscape` "stores the
  handle" (Seam-A) but the doc never names or types that member. Its type is the
  same undecided §C idiom as the accessor: `Option<TerrainHandle>` (`None` until
  set) vs. a sentinel-defaulted `TerrainHandle` vs. relying on the existing
  `RmManager.initialized: bool` flag with a plain `TerrainHandle` field. Ruling 12's
  "no `Option`/`Box`" guidance is stated for the **top-level `Engine.rmg` field**
  only, not for `RmManager`'s internal members, so it does not settle this by
  itself — this is a per-member idiom choice, not an agent's call. Raven ground
  truth that bears on it (not a resolution): the `G_RMG_INIT` arm always calls
  `new` → `SetLandScape(cmg.landScape)` → `LoadMission` in sequence
  (`sv_game.cpp:1624-1634`), and the snapshot read only runs once `G_RMG_INIT` has
  `new`d the manager (`SV_SendClientGameState` guards on `if (TheRandomMissionManager)`
  — Raven ground truth Data-flow step 6, `sv_client.cpp:779`) — so the handle is
  always set before any read; Raven's ctor
  leaves `mLandScape` `NULL` (`RM_Manager.cpp:34-42`). The field name/type freezes
  with the accessor row.

## Resolved questions

Closed by the 2026-07-09 §F rulings (recorded so a re-reader sees why they left
the open list):

- **RMG-Q1 — Fold the qcommon terrain twins in?** RESOLVED by ruling 16 → RMG-D2a.
- **RMG-Q2 — Are the `RM_Terrain.cpp` client-model classes in the dedicated link
  set?** RESOLVED by ruling 17 → RMG-D2c (no; §20-dropped).
- **RMG-Q3 — Classify the dead Perlin-noise scratch.** RESOLVED by ruling 17 →
  RMG-D2c (§20-drop). Mooted further by RMG-D1 (the whole generation path is dead).
- **RMG-Q4 — Crate placement for `rmAutomapSymbol_t`.** RESOLVED by ruling 21 →
  RMG-D2d (relocate to `mp_qshared`; no `rmg → client` edge).
- **RMG-Q5 — Concrete Rust form of `RandomTerrainHandle`.** RESOLVED by ruling 21 →
  RMG-D2e (no handle); the field itself is now §20-dropped by RMG-D1.
- **RMG-Q6 — Engine-tier owner for the free `flrand`/`irand` LCG.** RESOLVED by
  ruling 21 → RMG-D2f (`Engine.common.rng: QRand`, exposed via `EngineHost`); under
  RMG-D1 used only by the golden-only `RMG_CreateSeed`.
- **The generation-path scope.** RESOLVED by ruling 25 → RMG-D1 (§20-dropped;
  live surface = syscall arms + landscape construction + `LoadMission` early-out).
- **`RmManager.mCurObjective`.** RESOLVED by ruling 17 → RMG-D2c.
- **STATE-Q2 (placement + service halves) for `rmg`.** RESOLVED by ruling 12
  (direct `Engine.rmg` field) + ruling 11 (`EngineHost`) → RMG-D2b.

## Files roster

C++-track roster for `.claude/workflows/port-cpp-subsystem.js` (`designPath`).
`mode: mp` throughout (dedicated MP engine; SP twin out of scope, DEC-04). Under
**RMG-D1** the roster is the **reduced live surface only** — the generation-path
classes are NOT porter work orders; they appear as §20 `drop` entries in
`divergences` below (RMG-D2: "dropped-path classes keep their shape-map entries
marked §20").

**Crate dependencies (mechanical).** `mp_engine_rmg`'s `Cargo.toml` gains an
`mp_engine_qcommon` path dependency (RMG-D2a) so `RmManager` can name
`CmLandScape`/`CollisionWorld`/`TerrainHandle` in its frozen pub API. Per RMG-D2d
`rmAutomapSymbol_t` relocates to `mp_qshared` (already a dependency), so **no**
`mp_engine_client` edge is added.

```yaml
files:
  # --- mp_engine_rmg (oracle/codemp/RMG/) — LIVE surface only ---
  - { path: crates/mp/engine/rmg/src/rm_manager.rs,  crate: mp_engine_rmg,      mode: mp, class: CRMManager,   summary: "RmManager LIVE lifecycle (RMG-D1): new/SetLandScape/LoadMission (early-outs false — mTerrain always NULL under DEDICATED, RM_Manager.cpp:110-113) + the automap-symbol seam pair GetAutomapSymbolCount/GetAutomapSymbol (return 0/None). SpawnMission is a dead-under-DEDICATED stub kept so the guarded syscall arm compiles; it drives the §20-dropped CRMMission::Spawn. mCurObjective/WriteAutomapSymbols/ProcessAutomapSymbols §20-dropped (RMG-D2c). Private initialized:bool flag flipped at the G_RMG_INIT arm (ruling 12)" }
  # --- mp_engine_qcommon (oracle/codemp/qcommon/) — terrain twins, RMG-D2a ---
  - { path: crates/mp/engine/qcommon/src/cm_terrain.rs,       crate: mp_engine_qcommon, mode: mp, class: CCMLandScape,  summary: "CmLandScape LIVE construction under DEDICATED (RMG-D1, cm_terrain.cpp:116-219): config parse, bounds, heightmap(unpopulated)/flatten(memset-0) alloc, LoadTerrainDef, mPatches/UpdatePatches + CCMPatch collision build, seeded holdrand=0x89abcdef; register_terrain (Seam-B) get-or-create. CCMHeightDetails. LIVE RUNTIME SURFACE (Seam-B §C, owned here per RMG-D2a — the cm C-track packets exclude CCMLandScape): the snapshot-read accessors GetHeightMap/GetFlattenMap/GetRealArea/get_rand_seed (streamed sv_client.cpp:779-806) AND the per-frame collision methods PatchCollide (cm_terrain.cpp:600)/WaterCollide (:836) + GetWaterContents/GetWaterSurfaceFlags/GetWaterHeight/GetBounds/GetPatchScalarSize the cm-trace/cm-test packets call (cm_trace.cpp:283,760,789). Their idiomatic Rust signatures freeze via RMG-Q8/Q9 (not yet frozen). GetPatch/GetTerxelLocalCoords/SetShaders/CarveLine/CalcRealCoords are renderer(tr_terrain.cpp, DEC-01)/generation-only — §20 or private-internal, not seam. §20-dropped: mRefCount (renderer-only, DEC-01); the twelve cm_landscape.h:247-258 area CM_* wrappers (NOT CM_InitTerrain :246, which is LIVE — folded into register_terrain); and ALL area/carve methods FlattenArea/SaveArea/GetWorldHeight/AreaCollision/GetFirst|NextArea/FractionBelowLevel/CarveBezierCurve/GetFirst|Player|NextObjectiveArea + CArea (their only callers were the generation path, now §20 per RMG-D1); CM_TerrainPatchIterate; the inline flrand/irand/rand_seed (no live caller)" }
  - { path: crates/mp/engine/qcommon/src/cm_randomterrain.rs, crate: mp_engine_qcommon, mode: mp, class: RMG_CreateSeed, summary: "GOLDEN-ONLY: only RMG_CreateSeed (cm_randomterrain.cpp:1008, zero live callers) is ported — it pins Engine.common.rng (QRand) via EngineHost::flrand/irand (RMG-D2f, golden #1). The entire CRandomTerrain/CPathInfo generation class (Generate/Smooth/ParseGenerate) and the dead Perlin path (noiseTable/noisePerm) are §20-dropped (RMG-D1/RMG-D2c) — see divergences" }
```

Existing skeleton already present: `crates/mp/engine/rmg/src/rm_headers/symmetry_t.rs`
(`symmetry_t`, `RM_Headers.h:29-35`) and `rm_path/ermdir.rs` (`ERMDir`,
`RM_Path.h:24-37`) — faithful C enums the generation-path shape-map entries
reference; they are not exercised by the live surface.

## Divergences

Idiomatic §F reshapings (layout-free — these types never cross the module ABI) and
the §20 drops (RMG-D1/RMG-D2c) a transcriber records rather than ports. Under
RMG-D1 the entire generation subtree is `drop`, its §F shape retained here as the
shape-map (RMG-D2).

```yaml
divergences:
  # --- LIVE reshapings ---
  - { class: CRMManager,   kind: reshape, rule: "§B/RMG-D2b", note: "mMission:CRMMission* (RM_Manager.h:13) is §20-DROPPED — no Rust field: CRMMission has no Rust type (§20-dropped, RMG-D1) and mMission is dead under DEDICATED (ctor sets it NULL, RM_Manager.cpp:38; the only reassignment `new CRMMission` at RM_Manager.cpp:135 sits AFTER LoadMission's early-out RM_Manager.cpp:110-113 and is never reached; every read at :194/317/333/351/374/394 is inside the §20-dead methods). Corrects the earlier `owned field` framing — the field is never constructed on the live path (mirrors the mLandScape/mTerrain drops below). Cached CCMLandScape* mLandScape (RM_Manager.h:14) and CRandomTerrain* mTerrain (RM_Manager.h:15) likewise NOT stored — RmManager owns only the TerrainHandle; mTerrain resolves as always-None (RMG-D1, GetRandomTerrain()==0). TheRandomMissionManager → direct Engine.rmg field (ruling 12), no Option; lazy-init via a private initialized:bool flipped at the G_RMG_INIT arm (ctor RM_Manager.cpp:34-42 only zeroes members, so new()/Default and the lazy new collapse to one construction)" }
  - { class: CCMLandScape, kind: reshape, rule: "§B5/RMG-D2h", note: "byte* mHeightMap/mFlattenMap, CCMPatch* mPatches → owned Vec<u8>/Vec<CmPatch>; holdrand LCG stays an inline c_ulong field (Raven `unsigned long`, cm_landscape.h:160; seeded 0x89abcdef cm_terrain.cpp:122; get_rand_seed live-streamed). CCMPatch::owner:CCMLandScape* (cm_landscape.h:93) — a LIVE back-pointer on the patch-collision build (GetAdjacentBrushY, cm_terrain.cpp:246-256) — is DROPPED per §B3; the owning CmLandScape is threaded into the patch-build methods (§B4)" }
  - { class: CCMLandScape, kind: reshape, rule: "§A1/RMG-D2a", note: "LIVE RUNTIME SURFACE — the constructed landscape has two live external readers, both owned by THIS subsystem (RMG-D2a: the cm C-track packets exclude CCMLandScape, so its methods port here, not in a cm packet nor another doc). (1) Snapshot/download read from the server (sv_client.cpp:779-806, via TheRandomMissionManager->GetLandScape()): GetHeightMap (cm_landscape.h:218 — §F.19 UB bytes, excluded from golden compare), GetFlattenMap (:219), GetRealArea (:211), get_rand_seed (:239). (2) Per-frame terrain collision from the cm-trace/cm-test C-track packets (cm_trace.cpp:283,760,789,997,1374 + cm_test.cpp:285-289, non-BSPC, gated com_terrainPhysics->integer && cmg.landScape && CONTENTS_TERRAIN): PatchCollide (decl cm_landscape.h:175, def cm_terrain.cpp:600), WaterCollide (:178 / :836), GetWaterContents (:233), GetWaterSurfaceFlags (:234), GetWaterHeight (:232), GetBounds (:112/:199), GetPatchScalarSize (:207). Exact idiomatic Rust signatures for all of the above are RMG-Q8 (collision path — scope reconciliation + sigs) / RMG-Q9 (read path + RmManager accessor sigs), NOT invented here. GetPatch/GetTerxelLocalCoords/SetShaders/CarveLine/CalcRealCoords have NO live engine caller — only the renderer (tr_terrain.cpp, DEC-01) and the §20 RM_Terrain.cpp chain — so they are §20/renderer-only or private-internal (§A1), not seam" }
  - { class: rmAutomapSymbol_t, kind: relocate, rule: "RMG-D2d", note: "ABI type (client.h:149) the rosetta ported in mp_engine_client (crates/mp/engine/client/src/client/rm_automap_symbol_t.rs:9) RELOCATES to mp_qshared so mp_engine_rmg (already depends on mp_qshared) names it; RmManager::automap_symbol returns Option<&RmAutomapSymbol>. No rmg→mp_engine_client edge (client→rmg is the allowed direction). Seam pair is LIVE, returning count 0 under RMG-D1" }
  - { class: "flrand/irand (q_math.c:1432)", kind: reshape, rule: "§B3/RMG-D2f", note: "the free q_math LCG over file-scope holdrand → the engine's OWN mp_qshared::QRand instance Engine.common.rng (engine.rs:22, common/common.rs:20), exposed via EngineHost::flrand/irand (rulings 11+21). Only RMG consumer is the golden-only RMG_CreateSeed (cm_randomterrain.cpp:1008, zero live callers) — no live RMG path draws it (RMG-D1). The game-tier bg_channel::rng::Rng (crates/mp/game/src/bg_channel/rng.rs) is a distinct instance mp_engine_qcommon must NOT reach" }
  # --- §20 GENERATION-PATH DROPS (RMG-D1) — shape-map retained, not ported ---
  - { class: CreateRandomTerrain, kind: drop, rule: "§20/RMG-D1", note: "only call site is in the #else of #ifdef DEDICATED (cm_terrain.cpp:170-188, call :178), so mRandomTerrain stays 0 (:169) — dead code on the DEDICATED build. Was Seam-B create_random_terrain; now dropped, no seam entry" }
  - { class: CRandomTerrain, kind: drop, rule: "§20/RMG-D1/RMG-D2e", note: "whole class (Generate/Smooth/ParseGenerate/CPathInfo) unreachable: constructed only by the dead CreateRandomTerrain. No CmLandScape.random_terrain field (RMG-D2e moot under RMG-D1); GetRandomTerrain() models as always-None. Also §20 the dead Perlin path (noiseTable/noisePerm never written — CM_NoiseInit #if 0 at cm_randomterrain.cpp:17-28). mLandScape:CCMLandScape* back-pointer (cm_randomterrain.h:56) dropped with the class (§B3/RMG-D2h)" }
  - { class: CRMMission, kind: drop, rule: "§20/RMG-D1", note: "never constructed under DEDICATED — LoadMission early-outs at RM_Manager.cpp:110-113 before `new CRMMission` (:135). Load/Spawn/PreSpawn/Smooth/PlaceBridges/ParsePaths/ParseRivers and the objective/node placement all dead. Shape-map (per §17): mLandScape:CRandomTerrain* (RM_Mission.h:64) is a §B3 back-pointer that would be dropped + owner threaded (RMG-D2h); the #ifndef DEDICATED CTerrainMap block (RM_Mission.cpp:1503-1504) is §20 (RMG-D2c)" }
  - { class: RmInstance, kind: drop, rule: "§20/RMG-D1/RMG-D2i", note: "the closed CRMInstance base+4-subclass hierarchy (Bsp/Group/Random/Void, RM_Instance*.cpp) is only built during mission spawn (dead). Shape-map (§17): base+4 → one RmInstance enum, CreateInstance factory → match on GP2 group name; CRMRandomInstance's CRMInstance* mInstance → Box<RmInstance>, CRMGroupInstance's rmInstanceList_t → Vec<RmInstance>; CRMInstance::mArea → AreaId (RMG-D2g); dead \"npc\" branch (RM_InstanceFile.cpp:162-166) §20. Not ported" }
  - { class: CRMAreaManager, kind: drop, rule: "§20/RMG-D1/RMG-D2g", note: "CRMArea/CRMAreaManager only constructed during mission spawn (dead). Shape-map (§B5): mAreas (rmAreaVector_t, RM_Area.h:74,80) → owned Vec<CmArea>, raw CRMArea* handed out by CreateArea/EnumArea → AreaId index newtype (rendered like EntityId), threaded through SetArea/GetArea (RM_Instance.h:72,107). Rust name of CRMArea's element is RMG-Q7. Not ported" }
  - { class: CRMPathManager, kind: drop, rule: "§20/RMG-D1/RMG-D2h", note: "GeneratePaths/GenerateRivers over CRMNode/CRMLoc/CRMCell — only driven by CRMMission (dead). Shape-map (§F): rmNode/Loc/CellVector_t → Vec<Node>/Vec<Loc>/Vec<Cell>, Node(x,y)=mNodes[x+y*mXNodes] index math preserved (RM_Path.h:185); neighbor_x/y → const slices (fork-5). mTerrain:CRandomTerrain* (RM_Path.h:175, set at ctor RM_Path.cpp:56,60) is a §B3 back-pointer dropped + owner threaded (RMG-D2h). Not ported" }
  - { class: CRMInstanceFile, kind: drop, rule: "§20/RMG-D1", note: "GP2-backed instance-file open/close + CreateInstance string factory — only used by the dead mission load. Shape-map: CGenericParser2/CGPGroup* members → borrows into the ported GP2 arena. Not ported" }
  - { class: CRMObjective, kind: drop, rule: "§20/RMG-D1", note: "objective parse + Link — only used by the dead mission load (RM_Objective.cpp). Not ported" }
  - { class: CCMLandScape, kind: drop, rule: "§20/RMG-D1", note: "AREA/CARVE METHODS now dead: FlattenArea (cm_terrain.cpp:1312), SaveArea (:1128), GetWorldHeight (:1011) — previously live only via the generation path (CRMPathManager/CRMInstance/CRMMission::Spawn), all §20 under RMG-D1 — plus AreaCollision/GetFirst|NextArea/FractionBelowLevel/CarveBezierCurve/GetFirst|Player|NextObjectiveArea (cm_terrain.cpp:1488,1412,1462,1379,1245,1422,1442,1472). The twelve cm_landscape.h:247-258 area CM_* free-fn wrappers (defined cm_terrain.cpp:1633-1685; CM_InitTerrain :246 is LIVE, not in this set) and SV_LoadMissionDef (:262, declared-never-defined) are zero-caller drops. CArea (cm_landscape.h:42) appears only as these methods' arg — dead-surface (Rust name RMG-Q7). Recorded, not ported" }
  - { class: CCMLandScape, kind: drop, rule: "§20/DEC-01/RMG-D2c", note: "mRefCount (cm_landscape.h:138) dropped: its only reader is CM_ShutdownTerrain's count-gated free (cm_load.cpp:1073-1077), whose only caller is the renderer (tr_terrain.cpp:1050, DEC-01); the server frees unconditionally at teardown (cm_load.cpp:800-809). register_terrain still returns the existing TerrainHandle on repeat registration (get-or-create on Option<CmLandScape>, cm_load.cpp:1040-1044). CM_TerrainPatchIterate (free fn :1628 + method :997) dropped — its only callers were the renderer (tr_terrain.cpp:923, DEC-01) and the §20 RM_Terrain.cpp chain" }
  - { class: CRMManager, kind: drop, rule: "§20/RMG-D2c", note: "mCurObjective (RM_Manager.cpp:16) zero-init, never read/written; WriteAutomapSymbols (:424) commented-out; ProcessAutomapSymbols (:442) is a client-side static, dead under DEDICATED. SpawnMission (:391) unreachable under DEDICATED (LoadMission returns false) — kept as a dead stub so the guarded syscall arm compiles, its CRMMission::Spawn-driving body dropped. All zero-caller notes" }
  - { class: CRMManager, kind: drop, rule: "§20/RMG-D1", note: "the twelve declared public/private CRMManager methods with NO live caller (of the 13 the review flagged; the 13th, GetLandScape, is LIVE — see tail) are §20 zero-caller drops (grep of `TheRandomMissionManager->` finds no invocation of any): SetCurPriority (RM_Manager.h:36), GetTerrain (:38), GetCurPriority (:41), Preview (:48), IsMissionComplete (:50), HasTimeExpired (:51), CompleteObjective (:52), CompleteMission (:53), FailedMission (:54), UpdateStatisticCvars (:23) have zero callers anywhere in codemp; GetMission (:39) and AddAutomapSymbol (:43) are called ONLY from the §20-dropped generation path (RM_Instance*/RM_Path.cpp; RM_Manager.cpp:400-410 SpawnMission body). Not ported — same §20 reasoning as the sibling drop above; only new/SetLandScape/LoadMission/GetAutomapSymbolCount/GetAutomapSymbol (+ the dead SpawnMission stub) survive as the live Seam-A surface. The landscape accessor GetLandScape (:39) IS live (snapshot path) — its Rust form is RMG-Q9, not a drop" }
  - { class: CTerrainMap, kind: drop, rule: "§20/RMG-D2c", note: "whole automap-image builder dead under DEDICATED: its only ctor CM_TM_Create is #ifndef DEDICATED (RM_Mission.cpp:1503-1504); Upload/SaveImageToDisk named by ruling 17. Recorded, not ported (returns to scope if the renderer is un-deferred, DEC-01)" }
  - { class: CRMLandScape, kind: drop, rule: "§20/RMG-D2c", note: "RM_Terrain.cpp client-model chain (CRMLandScape/CCGHeightDetails/CRandomModel/CCGPatch, RM_CreateRandomModels, SpawnPatchModelsWrapper) — graph-confirmed zero engine callers under DEDICATED (ruling 17); reached only from the client (RM_CreateRandomModels ← cl_cgame.cpp:1707). Not ported" }
```
