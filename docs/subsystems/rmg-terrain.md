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
  Stage-0 crate `mp_host_interface`, `crates/mp/host-interface`, `:229-230`),
  **ruling 12** (the five §F states are plain
  Default-initialized direct `Engine` fields — `rmg` among them, `:127-131`),
  **ruling 16** (the qcommon terrain twins fold into *this* doc, `:143-146`),
  **ruling 17** (the four §20 dead-surface drops, `:147-152`), **ruling 21** (the
  five holes closed, RMG-D4d…h), **ruling 25** (2026-07-09: RMG generation is
  dead under DEDICATED — the headline of RMG-D1), **ruling 28** (2026-07-09: the
  per-frame terrain-collision surface IS LIVE, closing RMG-Q7/Q8/Q9 — RMG-D1),
  and **rulings 31/32/33** (2026-07-09: the Stage-0 `mp_host_interface` crate is
  built and green — commit `4b7f01b0`; goldens run on the fixture-backed
  `MockHost`, ruling 32 — RMG-D3). Ruling 24 is now discharged: this doc **quotes
  the real `EngineHost` signatures verbatim** from
  `crates/mp/host-interface/src/engine_host.rs` (RMG-D3), it no longer merely
  cites an unbuilt prerequisite.
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
early-out + collision surface).** Ruling 28 amended RMG-D1's live enumeration to
**four** items reachable under DEDICATED and ported here:

1. **The reachable RMG syscall arms** — `G_CM_REGISTER_TERRAIN`
   (`sv_game.cpp:1640-1641`) and `G_RMG_INIT` (`sv_game.cpp:1624-1638`).
   (`G_SET_ACTIVE_SUBBSP` is out-of-scope clipmap wiring, Non-goals.)
2. **`CCMLandScape` construction under DEDICATED** (`cm_terrain.cpp:116-219`):
   config parse, bounds, heightmap/flatten allocation (`mFlattenMap` memset-0 at
   `:161`; `mHeightMap` allocated but **unpopulated** — no image load, no
   generation, under DEDICATED), `LoadTerrainDef` (`:208`, unconditional — no
   `#ifdef DEDICATED` guard; its `altitudetexture`/`water` cases read shader flags
   via `CM_GetShaderInfo`, the wider-clipmap machinery — Non-goals / RMG-Q10),
   patch build
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
4. **The per-frame terrain-collision surface (LIVE, ruling 28 / RMG-D1).** The
   `CmLandScape` built in item 2 has live readers beyond construction/snapshot:
   the per-frame terrain-collision methods `PatchCollide`/`WaterCollide` + the
   `GetBounds`/water accessors the `cm-trace`/`cm-test` C-track packets call
   (`cm_trace.cpp:283,760,789`; `cm_test.cpp:285-289`, gated
   `com_terrainPhysics->integer && cmg.landScape && CONTENTS_TERRAIN`). Ruling 28
   folded this fourth item into RMG-D1's live enumeration; by RMG-D4a (the `cm`
   C-track packets exclude `CCMLandScape`) these methods are owned by *this*
   subsystem — no other doc. They port as `CmLandScape` methods with faithful
   signatures threaded through `&`/`&mut CollisionWorld`, **frozen now** in
   `## Seam definition` §C (RMG-D1). Per `engine-port-order.tsv` their
   signature-pinning C-track callers land at waves 0–4, *before* the wave-15/17
   `CmLandScape` constructor / `CM_RegisterTerrain` — so ruling 28 lands the
   collision methods with those **early clipmap-trace waves**, not with the
   wave-16 §F unit (Slice hooks).

**In scope — the qcommon terrain twins** (`oracle/codemp/qcommon/`), folded here
by **ruling 16** (RMG-D4a), reduced to their live members: `CCMLandScape` +
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
- **The wider clipmap** (`cm_load.cpp`, `cm_patch.cpp`, `cm_trace.cpp`, and the
  shader machinery `cm_shader.cpp`). Only the terrain-owned members of
  `CCMLandScape` are here; `CM_RegisterTerrain`'s clipmap wiring is a C-track
  qcommon packet. **The shader machinery `CCMShader`/`CCMShaderText`/
  `CM_GetShaderInfo` is clipmap surface, not terrain-owned**, so this doc does not
  port `cm_shader.cpp`: `CCMShader` is declared in the clipmap header
  (`cm_local.h:77`; `cmg.shaders`/`numShaders`, `:110-111`), `CM_GetShaderInfo` is
  a clipmap free function (`cm_local.h:303-304`, def `cm_shader.cpp:498,526`), and
  the shader-text/property machinery (`CM_LoadShaderText`/`CM_SetupShaderProperties`/
  `CM_CreateShaderTextHash`/`CM_ShutdownShaderProperties`, globals `shaderText`/
  `shaderTextTable`/`cmShaderTable`, `Hunk_Alloc`-backed `CCMShader`) is driven by
  map load (`cm_load.cpp:733,737,796`), not by terrain. `LoadTerrainDef` (In-scope
  item 2) only *reads* through the existing `CM_GetShaderInfo` accessor, reached via
  the `CollisionWorld` the ctor/`register_terrain` already thread (§B4, STATE-D2);
  the Rust binding it calls is the open item **RMG-Q10**.
- **Renderer-side terrain draw** (`tr_terrain*`) and everything gated on it.
  Deferred with the renderer (DEC-01); not in the dedicated link set.
- **The four ruling-17 §20 drops** (RMG-D4c): `mCurObjective`, the dead Perlin
  scratch (`noiseTable`/`noisePerm`), the `RM_Terrain.cpp` client-model chain,
  and the `CTerrainMap` automap-image builder. Recorded in Divergences.

## Raven ground truth

### Data flow (server boot → terrain → mission), corrected for DEDICATED

1. The game module vmcalls `trap_CM_RegisterTerrain(config)`
   (`oracle/codemp/game/g_syscalls.c:1473-1476`, `g_misc.c:582`). The syscall case
   `G_CM_REGISTER_TERRAIN` calls `CM_RegisterTerrain((const char *)VMA(1), true)` and
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
the `GetBounds`/water accessors, gated `com_terrainPhysics->integer`,
`cm_trace.cpp:283,760,789`) are owned by *this* subsystem (RMG-D4a) and ported
here — LIVE per ruling 28 (RMG-D1), with signatures **frozen** in
`## Seam definition` §C. Automap symbols are read once at client connect and are
empty.

### Class tree (closed hierarchy — recorded for the §20 shape-map)

`CRMInstance` is an abstract base with four concrete subclasses and pure-ish
virtuals `PreSpawn`/`Spawn`/`PostSpawn`/`SetArea`/… (`oracle/codemp/RMG/
RM_Instance.h:25-117`). The factory `CRMInstanceFile::CreateInstance`
string-dispatches `"bsp"|"group"|"random"|"void"` to `new CRM{BSP,Group,Random,
Void}Instance` (`RM_InstanceFile.cpp:138-193`); no subclass is created anywhere
else — the hierarchy is **closed**. This shape (base+4 → one `RmInstance` enum) is
recorded in Divergences (RMG-D4i) because the dropped-path classes keep their
shape-map entries (RMG-D4), even though nothing here constructs them under
DEDICATED.

### Globals (see State ownership for owners)

- `CRMManager* TheRandomMissionManager` — the one live singleton
  (`oracle/codemp/RMG/RM_Manager.cpp:23`; extern `RM_Manager.h:60`). LIVE (it is
  `new`d and runs through the early-out).
- `CRMManager::mCurObjective` — static member, zero-init only (`RM_Manager.cpp:16`)
  and never read/written in codemp. §20-dropped (ruling 17 / RMG-D4c).
- `static CTerrainMap* TerrainMap` (`cm_terrainmap.cpp:14`), `static float
  noiseTable[256]` / `static int noisePerm[256]` (`cm_randomterrain.cpp:14-15`),
  the seed-name tables (`Consonants[]`, `cm_randomterrain.cpp:847+`), and
  `CRMPathManager::neighbor_x/y` (`RM_Path.h:172-173`) — all on the §20-dropped
  generation/renderer path (RMG-D1/RMG-D4c); recorded in Divergences, not ported.
- `static int instanceID` in `CreateInstance` — assigned-never-read scratch on the
  dropped path (`RM_InstanceFile.cpp:140`).
- The free-function `flrand`/`irand` LCG over the file-scope global
  `holdrand = 0x89abcdef` (`oracle/codemp/game/q_math.c:1432,1441-1470`), seeded by
  `Rand_Init` (`:1434`). Its only RMG consumer is `RMG_CreateSeed`
  (`cm_randomterrain.cpp:1008,1016-1018`), which has **zero live callers** and is
  kept **golden-only** (RMG-D4f) — no live RMG path draws it.
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
| `TheRandomMissionManager` | `RM_Manager.cpp:23` | `mp_engine_core::Engine.rmg: RmManager` (plain direct field mandated by ruling 12; STATE-D5). **The field is not present in `engine.rs` yet** — `engine.rs:35-36` currently marks the STATE-Q2 attachment point ("rmg engine-side state is NOT yet a field here"); ruling 12 settled its placement (direct field, no `Option`/`Box`). It is added to the `mp_engine_core` `Engine` struct at the **Wave-20 `SV_GameSystemCalls`/Engine-assembly wiring** (Slice hooks, Wave 20), **not** by this Wave-16 §F roster — a cross-doc split (Gate 3). Raven lazily `new`s it under `com_RMG`; modeled with the private `RmManager.initialized: bool` (Default `false`, flipped at the `G_RMG_INIT` arm — Seam-A owned-state note), not `Option`. Its `mLandScape` cache → `RmManager.land: Option<TerrainHandle>` (RMG-D1/ruling 28: `None` until `set_landscape`, Raven's ctor leaves `mLandScape` `NULL` — `RM_Manager.cpp:34-42`) | `G_RMG_INIT` case (lazy) — `sv_game.cpp:1627-1629` | `&mut self` + `&mut impl EngineHost` from the syscall switch inward |
| `CRMManager::mCurObjective` | `RM_Manager.cpp:16` | **dropped** — §20 dead surface (RMG-D4c/ruling 17): zero-init, never read/written | — | — |
| `CCMLandScape*` (`cmg.landScape`) | `cm_landscape.h:135`; `cm_local.h:155`; `sv_game.cpp:1631` | `mp_engine_qcommon::CollisionWorld.land_scape: Option<CmLandScape>` (a field on the existing STATE-D2 `cmg` owner — `collision_world.rs:10`). `Option` is Raven-faithful: `cmg.landScape` is a nullable pointer set only on a terrain map. **LIVE** — constructed under DEDICATED | `CM_RegisterTerrain` — `cm_load.cpp:1036,1055` | `TerrainHandle` (wrapping `thandle_t`) across the seam; borrow inward |
| `CRandomTerrain*` (`mRandomTerrain` / the `random_terrain` field) | `cm_landscape.h:153`, `cm_randomterrain.h:52` | **dropped** — §20 generation path (RMG-D1): `CreateRandomTerrain` is in the `#else` of `#ifdef DEDICATED` (`cm_terrain.cpp:170-188`), so `mRandomTerrain` stays `0`. No `random_terrain` field is added; `GetRandomTerrain()` is modeled as always-`None`. Shape-map entry in Divergences | (never, under DEDICATED) | — |
| `static CTerrainMap* TerrainMap` | `cm_terrainmap.cpp:14` | **dropped** — §20 (RMG-D4c/ruling 17): only writer `CM_TM_Create` is `#ifndef DEDICATED` (`RM_Mission.cpp:1503-1504`) | — | — |
| `noiseTable` / `noisePerm` | `cm_randomterrain.cpp:14-15` | **dropped** — §20 generation path (RMG-D1/RMG-D4c): the Perlin path is dead code and unreachable under DEDICATED | — | — |
| `Consonants[]`, `CRMPathManager::neighbor_x/y`, `CreateInstance::instanceID` | `cm_randomterrain.cpp:847+`; `RM_Path.h:172-173`; `RM_InstanceFile.cpp:140` | **dropped** — §20 generation path (RMG-D1): const/scratch on the never-constructed mission/path/instance objects | — | — |
| free `flrand`/`irand` global `holdrand` | `q_math.c:1432` | `mp_engine_core::Engine.common.rng: mp_qshared::QRand` — the engine's own q_math LCG instance (RMG-D4f/ruling 21). Exposed via `EngineHost::flrand`/`irand`. **No live RMG draw** under DEDICATED; only the golden-only `RMG_CreateSeed` uses it | `Rand_Init` (`q_math.c:1434`) | `&mut impl EngineHost` |
| `CRMArea*` — `mAreas` arena + `CRMInstance::mArea` | `RM_Area.h:74,80`; `RM_Instance.h:33` | **dropped** — §20 generation path (RMG-D1): `CRMAreaManager`/`CRMArea` are only constructed during mission spawn (never reached). The `AreaId` arena shape (RMG-D4g/ruling 21) is retained as a Divergences shape-map entry | — | — |
| `com_RMG`, `com_terrainPhysics` | `common.cpp:72`; `cm_landscape.h:267` | `EngineCvars` handles (fork-2). `com_RMG` is LIVE (gates `G_RMG_INIT`) | `Cvar_Get` at init | read via cvar accessor |
| `CCMLandScape::holdrand` | `cm_landscape.h:160` | `CmLandScape.holdrand: c_ulong` — an inline per-instance LCG field; seeded `0x89abcdef` in the live ctor (`cm_terrain.cpp:122`) and read by `get_rand_seed` (streamed, `sv_client.cpp:806`). `flrand`/`irand`/`rand_seed` (`cm_terrain.cpp:1548-1580`) are §20 within the class (no live caller). **Not** an external `Rng` type | `CCMLandScape` ctor (`cm_terrain.cpp:122`) | field; see RNG threading |
| `CCMShader`/`cmg.shaders`/`numShaders`; `shaderText`/`shaderTextTable`/`cmShaderTable` | `cm_local.h:77,110-111`; `cm_shader.cpp:28-30` | **not owned here** — wider-clipmap shader machinery on the `cm` C-track qcommon packet (Non-goals). `cmg`-resident, so it lives under `CollisionWorld` (STATE-D2). `LoadTerrainDef` reads it via `CM_GetShaderInfo` — the Rust binding is **RMG-Q10** | `CM_LoadMap` (`cm_load.cpp:733,737`) — NOT terrain | `&mut CollisionWorld` (§B4) |

## Seam definition

RMG crosses **two** boundaries; nothing here crosses the *module* ABI (no
`#[repr(C)]` layout constraint — §F), so all types below are idiomatic.

**The host seam (ruling 11; the crate is BUILT — RMG-D3).** Every §F engine
service Raven reached through a file-scope global or `gi.`/`Com_` call —
`Com_Printf`/`Com_Error`, cvar reads, FS — is threaded as the one `EngineHost`
services trait (trace, FS, print/error, VM_Call, shared memory — plus the
`flrand`/`irand` RNG services backed by `Engine.common.rng`, RMG-D4f). **Rulings
31/33 discharged ruling 24: the trait now exists, green, in the Stage-0 crate
`mp_host_interface` (`crates/mp/host-interface`, package `mp_host_interface`,
commit `4b7f01b0`).** Consumers store `&mut dyn EngineHost` (ruling 24), so it is
dyn-compatible (no generic methods, no by-value `Self`). Its exact signatures,
**quoted verbatim** from `crates/mp/host-interface/src/engine_host.rs` so this doc
is self-contained (RMG-D3):

```rust
// oracle cites are on each method in engine_host.rs; abridged to signatures here.
pub trait EngineHost {
    fn trace(&mut self, results: &mut trace_t, start: &vec3_t, mins: &vec3_t,
             maxs: &vec3_t, end: &vec3_t, pass_entity_num: i32, contentmask: i32,
             capsule: bool, trace_flags: i32, use_lod: i32);
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

`Engine` implements the trait via a split-borrow view struct; goldens/referee
inject the fixture-backed `MockHost` (`crates/mp/host-interface/src/mock.rs`,
ruling 32 — DEC-09), whose `flrand`/`irand` replicate Raven's `q_math.c`
`holdrand` LCG. §F methods that touch a service take `&mut impl EngineHost`; the
`CollisionWorld` state is *not* a service and stays a separate threaded param
(§B4). Under RMG-D1 the live host use is exactly three methods: `EngineHost::print`
(`Com_Printf` — the RMG banner in `LoadMission`, `RM_Manager.cpp:106-107`),
`EngineHost::fs_read_file` (the config/FS reads inside `CM_RegisterTerrain`
construction), and `EngineHost::flrand`/`irand` (the golden-only `RMG_CreateSeed`'s
RNG draws).

**Handle types (§B5, layout-free).**

- `TerrainHandle` — a newtype over the rosetta's `thandle_t`
  (`type thandle_t = c_int`, `crates/native/types/src/lib.rs:65`); the ABI-crossing
  id the syscall returns (`GetTerrainId()`, `cm_landscape.h:220`; `mTerrainHandle`,
  `:139`). **Defined in `mp_engine_qcommon`** (a small `terrain_handle.rs` beside
  `collision_world.rs`): `register_terrain` constructs it and
  `RmManager::set_landscape` consumes it; the crate edge runs `rmg → qcommon` only
  (never the reverse), so the shared handle must live in `qcommon` (or lower) —
  the mechanical consequence of the settled dependency direction. **LIVE.**
- **No random-terrain handle** (RMG-D4e/ruling 21). Moot under RMG-D1 — the
  `CmLandScape.random_terrain` field is §20-dropped (never constructed), so no
  handle and no field exist; `GetRandomTerrain()` models as always-`None`.
- `AreaId` — the §B5 index newtype for the `CRMAreaManager` arena (RMG-D4g).
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
    /// `CRMManager::SetLandScape` — RM_Manager.cpp:79. Stores the handle into
    /// `self.land: Option<TerrainHandle>` (`Some(land)`); mTerrain =
    /// GetRandomTerrain() is always None under DEDICATED, RMG-D1.
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
    /// `CRMManager::GetLandScape` — RM_Manager.h:39. Returns the stored handle
    /// (`self.land`); the snapshot read (Seam-C) resolves it against the owning
    /// `CollisionWorld`. Frozen `Option<TerrainHandle>` per ruling 28 (RMG-D1):
    /// `None` before the `G_RMG_INIT` arm's `set_landscape`.
    pub fn land(&self) -> Option<TerrainHandle>;
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
porters add the field and flip it at the Wave-20 syscall arm. `RmManager` also
carries the private `land: Option<TerrainHandle>` (Raven's `mLandScape` cache,
`RM_Manager.h:14`; `None` by Default matching the `NULL`-zeroing ctor,
`RM_Manager.cpp:34-42`), set by `set_landscape` and read by `land()` (ruling 28 /
RMG-D1).

**Seam deviation — the added `cm: &mut CollisionWorld` parameter (not a design
change).** Raven's `LoadMission`/`SpawnMission` take only `qboolean IsServer`
(`RM_Manager.cpp:96,391`) and reach the landscape through the `cmg.landScape` file
global. Per §B (no hidden globals), `RmManager` owns **only** a
`land: Option<TerrainHandle>`;
the `CCMLandScape` data lives in `CollisionWorld` (STATE-D2, `collision_world.rs:10`).
So both methods take the owning `CollisionWorld` explicitly to resolve that handle
— the state-threading form (§B4) of Raven's global reach. (This is why
`mp_engine_rmg` needs the `mp_engine_qcommon` edge — see "Crate dependencies".)

`rmAutomapSymbol_t` is an existing ABI type (`oracle/codemp/client/client.h:149`,
`MAX_AUTOMAP_SYMBOLS = 512` `:151`); the rosetta ported it in
`mp_engine_client` (`crates/mp/engine/client/src/client/rm_automap_symbol_t.rs:9`).
Per **RMG-D4d** (ruling 21) it **relocates to `mp_qshared`** — which
`mp_engine_rmg` already depends on — so `RmManager::automap_symbol` returns
`Option<&RmAutomapSymbol>` directly, with **no** `rmg → mp_engine_client` edge. The
exact destination is pinned by **RMG-D2(b)** (round-4 mechanical resolution):
`crates/mp/qshared/src/common/mp/rmg/rm_automap_symbol_t.rs` (a new `rmg/` folder
mirroring `oracle/codemp/RMG/RM_Manager.h` ownership), with the
`mp_engine_client` import updated in the same commit.
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
`GetLandScape` (`RM_Manager.h:39`), which **is** live (the snapshot read) and is
now **in** the impl block as `land() -> Option<TerrainHandle>` (ruling 28 / RMG-D1
settled its Rust form).

### B. RMG → qcommon terrain (the free-function entry points)

`cm_landscape.h:245-265` declares the C entry points the server/clipmap call. The
frozen `mp_engine_qcommon` surface under RMG-D1:

```rust
/// `CM_RegisterTerrain` — cm_load.cpp:1036. Constructs (or, on repeat
/// registration, get-or-creates — cm_load.cpp:1040-1044) the CmLandScape under
/// DEDICATED. The random-terrain arm (cm_terrain.cpp:178) is never taken (RMG-D1).
pub fn register_terrain(cm: &mut CollisionWorld, host: &mut impl EngineHost, config: &str, server: bool) -> TerrainHandle;
/// `RMG_CreateSeed` — cm_randomterrain.cpp:1008 (draws the engine's q_math LCG via
/// EngineHost::flrand/irand — RMG-D4f; **zero live callers** in codemp, kept as a
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
(its Rust name is `CmArea`, settled by ruling 28 / RMG-D1 — the qcommon `CCM*→Cm*`
family — but it names no live type; emitted as neither a marker nor a stub per
RMG-D2(a)). Recorded in Divergences.

**Repeat-registration / refcount (RMG-D4c/DEC-01).** Raven's `CM_RegisterTerrain`
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
external readers**, both owned by *this* subsystem (RMG-D4a: the `cm` C-track
packets exclude `CCMLandScape`, so its methods port here — not in a `cm` packet,
not in another doc). **Ruling 28 settled both readers' scope (LIVE) and their
idiomatic Rust signatures (RMG-D1), so this subsection FREEZES.** Faithful
signatures threaded through `&`/`&mut CollisionWorld` per ruling 28; the C-track
`cm-trace`/`cm-test` caller resolves the landscape from `CollisionWorld.land_scape`
and calls the method (split-borrow: `if let Some(land) = &cm.land_scape { … }`).
**That split-borrow call form compiles for `water_collide`, `bounds`, and the pure
`&self` water/size accessors below — none takes `cm` — but it does *not* compile
for `patch_collide`, which is frozen as `(&self, cm: &mut CollisionWorld, …)`:
holding `&cm.land_scape` (to name `land`) while passing `&mut cm` (the whole owning
struct, of which `land_scape` is a field) into the same call is E0502 ("cannot
borrow `*cm` as mutable because it is also borrowed as immutable"), inexpressible in
safe Rust. Ruling 28 froze the signature but not the call form; the resolution is
an open item — see RMG-Q11 (it interacts with fork-1's panic+`catch_unwind`
`Com_Error` model, so it is escalated, not self-resolved).**
Frozen pub API on `mp_engine_qcommon`:

```rust
impl CmLandScape {
    // --- 1. Snapshot/download read (server, sv_client.cpp:779-806, reached via
    //        TheRandomMissionManager->GetLandScape()->…; In-scope item 3) ---
    /// `CCMLandScape::GetHeightMap` — cm_landscape.h:218. `byte*` → `&[u8]`
    /// (ruling 28). §F.19-UB: unpopulated under DEDICATED — excluded from goldens.
    pub fn height_map(&self) -> &[u8];
    /// `CCMLandScape::GetFlattenMap` — cm_landscape.h:219. `byte*` → `&[u8]`.
    pub fn flatten_map(&self) -> &[u8];
    /// `CCMLandScape::GetRealArea` — cm_landscape.h:211.
    pub fn real_area(&self) -> i32;
    /// `CCMLandScape::get_rand_seed` — cm_landscape.h:239. Raven `unsigned long`.
    pub fn get_rand_seed(&self) -> c_ulong;

    // --- 2. Per-frame terrain collision (cm-trace/cm-test C-track packets,
    //        cm_trace.cpp:283,760,789,997,1374 + cm_test.cpp:285-289, non-BSPC,
    //        gated com_terrainPhysics->integer && cmg.landScape && CONTENTS_TERRAIN;
    //        In-scope item 4, ruling 28) ---
    /// `CCMLandScape::PatchCollide` — decl cm_landscape.h:175, def cm_terrain.cpp:600.
    /// Faithful: `trace_t &trace` out-param → `&mut trace_t`; `cm: &mut CollisionWorld`
    /// threaded for the `CM_CalcExtents`/collision globals Raven reached ambiently
    /// (§B4, ruling 28). `tw`/`trace_t` are C-track types.
    /// NOTE: the prescribed `if let Some(land) = &cm.land_scape` call site cannot
    /// pass `&mut cm` alongside (E0502); resolving the borrow is open — RMG-Q11.
    pub fn patch_collide(&self, cm: &mut CollisionWorld, tw: &mut traceWork_s, trace: &mut trace_t, start: vec3_t, end: vec3_t, checkcount: i32);
    /// `CCMLandScape::WaterCollide` — cm_landscape.h:178 / cm_terrain.cpp:836.
    /// `const` + pure (reads only `mWaterHeight`) — no `CollisionWorld` needed.
    pub fn water_collide(&self, begin: vec3_t, end: vec3_t, fraction: f32) -> f32;
    /// `CCMLandScape::GetBounds` — cm_landscape.h:199.
    pub fn bounds(&self) -> &vec3pair_t;
    /// `CCMLandScape::GetPatchScalarSize` — cm_landscape.h:207.
    pub fn patch_scalar_size(&self) -> f32;
    /// `CCMLandScape::GetWaterHeight` — cm_landscape.h:232.
    pub fn water_height(&self) -> f32;
    /// `CCMLandScape::GetWaterContents` — cm_landscape.h:233.
    pub fn water_contents(&self) -> i32;
    /// `CCMLandScape::GetWaterSurfaceFlags` — cm_landscape.h:234.
    pub fn water_surface_flags(&self) -> i32;
}
```

Of the five internal `CCMLandScape` methods an earlier draft grouped as "no live
engine caller", **three are in fact LIVE private-internal (§A1) helpers** on the
in-scope construction/collision path — transcribed here, but **not** pub seam
(each is reached only from another ported method, never across a boundary):
`SetShaders` (`CCMLandScape` method, def `cm_terrain.cpp:26`) is called from
`LoadTerrainDef` at `cm_terrain.cpp:83` (In-scope item 2, no `#ifdef` guard);
`CalcRealCoords` (def `:975`) from `UpdatePatches` at `:914`; and `GetPatch`
(def `:593`) from `PatchCollide`/`WaterCollide` (`:681,768,823`, ruling-28 LIVE)
and the patch-collision build `CCMPatch::GetAdjacentBrushX/Y` (`:256,282`). The
remaining **two genuinely have no live caller** and stay §20/dead-surface, not
seam: `GetTerxelLocalCoords` (its only call sites, `:948-950`, are inside the
commented-out block `:929-969`) and `CarveLine` (its only caller is the
§20-dropped area/carve method `CarveBezierCurve`, `:1303`). None of the five is a
pub seam method. (Corrects the prior "renderer-only / no live caller" claim for
`SetShaders`/`CalcRealCoords`/`GetPatch`.)

**`LoadTerrainDef`'s live shader reads — a cross-subsystem dependency (RMG-Q10).**
The LIVE `SetShaders` above takes a `CCMShader*` produced by
`CM_GetShaderInfo(shaderName)` (`cm_terrain.cpp:80→83`); `LoadTerrainDef`'s `water`
case likewise calls `CM_GetShaderInfo` and reads `shader->contentFlags`/
`shader->surfaceFlags` into `mWaterContents`/`mWaterSurfaceFlags` (`:98-103`) — the
exact fields Seam §C freezes as `water_contents()`/`water_surface_flags()`.
`CCMShader`/`CM_GetShaderInfo` are **wider-clipmap machinery, not terrain-owned**
(Non-goals / State ownership: `cm_local.h:77,303-304`, `cm_shader.cpp`, `cmg`-resident
per STATE-D2), so this doc does not port them; the Rust binding `LoadTerrainDef`
calls through the threaded `CollisionWorld` (§B4) is the open item **RMG-Q10**.

**The `RmManager` landscape accessor is `land() -> Option<TerrainHandle>`
(ruling 28 / RMG-D1, frozen in Seam-A).** The snapshot read above reaches the
landscape through `TheRandomMissionManager->GetLandScape()`; Raven returns the
cached `mLandScape` member, but per the Divergences the port stores **only** the
`TerrainHandle` (`mLandScape` NOT stored), so `land()` returns the handle and the
snapshot caller resolves it against the owning `CollisionWorld` (which owns
`land_scape: Option<CmLandScape>`). Ruling 28 chose "return the `TerrainHandle`
for the caller to resolve" over "borrow `&CmLandScape` via a threaded
`&CollisionWorld`".

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

*Amendment (ruling 28, 2026-07-09 — closes RMG-Q7, RMG-Q8 and RMG-Q9).* The live
enumeration is amended from three items to **four**: the per-frame
terrain-collision surface — `CmLandScape::PatchCollide`/`WaterCollide`,
`GetBounds`, and the water accessors (`GetWaterHeight`/`GetWaterContents`/
`GetWaterSurfaceFlags`/`GetPatchScalarSize`) — is **LIVE** under DEDICATED (the
`cm-trace`/`cm-test` server collision path, `cm_trace.cpp:283,760,789`;
`cm_test.cpp:285-289`), not generation-path code, and is ported here (RMG-D4a: the
`cm` C-track packets exclude `CCMLandScape`). These port as `CmLandScape` methods
with **faithful signatures threaded through `&`/`&mut CollisionWorld`, FROZEN now**
in Seam §C. This doc keeps their ownership; because `engine-port-order.tsv` demands
their signature-pinning callers at waves 0–4 (before the wave-15/17 constructor),
they **land with the early clipmap-trace waves**, not the wave-16 §F unit (the
wave split, noted explicitly in Slice hooks). Ruling 28 also settles: the snapshot
accessors `GetHeightMap`/`GetFlattenMap` → `&[u8]`; the `RmManager` landscape
accessor `land() -> Option<TerrainHandle>` with `RmManager.land: Option<TerrainHandle>`
(callers resolve through `CollisionWorld`); and the two `Area` class names —
`CRMArea` → **`RmArea`** and the qcommon `CArea` → **`CmArea`** (both remain
§20-dropped shape-map entries, RMG-D4a/RMG-D4g).

**RMG-D2 — Round-4 mechanical resolutions.** Recorded in the ledger, applied
here. **(a)** STRIKE the earlier "`//TODO: Port CArea` marker" fallback: the
engine-wide no-`TODO`/no-`FIXME` rule (`GOAL-engine.md:24-28`) wins
**unconditionally**, so §20-dropped items get a zero-caller §20 note in Divergences,
**never** a marker or stub. **(b)** `rmAutomapSymbol_t` (RMG-D4d) relocates to the
exact path `crates/mp/qshared/src/common/mp/rmg/rm_automap_symbol_t.rs` — a **new
`rmg/` folder** under `mp_qshared` mirroring `oracle/codemp/RMG/RM_Manager.h`
ownership — with the `mp_engine_client` import updated in the **same commit**.
Because the type is RMG-owned (its Raven home is `RM_Manager`'s automap seam), so
its qshared home mirrors that subsystem, not the flat `src/shared/` bucket.
Rejected the `src/shared/` placement an earlier draft named.

**RMG-D3 — Rulings 31/32/33: the Stage-0 host-interface crate is BUILT.** Ruling 24
is discharged. The crate is green at `crates/mp/host-interface` (package
`mp_host_interface`, commit `4b7f01b0`), so this doc **quotes the real `EngineHost`
signatures verbatim** from `crates/mp/host-interface/src/engine_host.rs` (Seam
definition) — the doc is self-contained, no unbuilt-prerequisite framing remains.
Goldens/referee run on the fixture-backed `MockHost`
(`crates/mp/host-interface/src/mock.rs`, ruling 32), whose `flrand`/`irand`
replicate Raven's `q_math.c` LCG — so **no paper-spec citations remain** in
Verification strategy. Rejected re-deriving the seam or a subsystem-local test
double. Because a built, green Stage-0 crate is the ground truth ruling 24
promised.

**RMG-D4 — All rulings 11-26 stand, applied to the reduced live surface;
dropped-path classes keep their shape-map entries marked §20.** The prior §F
decisions carry forward verbatim; where RMG-D1 makes their subject dead code, the
decision still governs the shape recorded in Divergences (the §20 shape-map),
not a ported implementation. Recorded with stable sub-IDs so the body's cites
resolve:

- **RMG-D4a — Fold the qcommon terrain twins into this doc** (ruling 16,
  `:143-146`). `CCMLandScape`, `CCMPatch`, `CCMHeightDetails` (live construction),
  and — as dropped shape-map — `CRandomTerrain`, `CTerrainMap`, `CPathInfo`,
  `CArea` are owned by *this* subsystem; the `cm` C-track packets exclude them.
  Because the tree cannot be designed apart from them. Rejected a separate qcommon
  doc.
- **RMG-D4b — State on direct `Engine` fields; services via the one `EngineHost`
  trait** (rulings 12 `:127-131`, 11 `:121-126`). `TheRandomMissionManager` →
  `mp_engine_core::Engine.rmg: RmManager` (no `Option`/`Box`; lazy-init via
  Raven's own flag); const tables → `const`; cvars → `EngineCvars`; every engine
  service (FS, print/error, cvar, trace, `flrand`/`irand`) → `&mut impl
  EngineHost`. Rejected globals/sub-structs (§B3).
- **RMG-D4c — §20-drop the four ruling-17 dead-surface items** (ruling 17
  `:147-152`): (a) `mCurObjective` (`RM_Manager.cpp:16`); (b) `noiseTable`/
  `noisePerm` — dead Perlin path (`CM_NoiseInit` `#if 0`, `cm_randomterrain.cpp:17-28`);
  (c) the `RM_Terrain.cpp` client-model chain; (d) `CTerrainMap` (its only ctor
  `CM_TM_Create` is `#ifndef DEDICATED`, `RM_Mission.cpp:1503-1504`). All recorded
  in Divergences. Rejected porting them: no live DEDICATED caller.
- **RMG-D4d — `rmAutomapSymbol_t` relocates to `mp_qshared`** (ruling 21 part 1).
  The rosetta ported it in `mp_engine_client`
  (`crates/mp/engine/client/src/client/rm_automap_symbol_t.rs:9`); it moves to
  `mp_qshared` (already a dependency), so `RmManager::automap_symbol` names it
  directly — no `rmg → mp_engine_client` edge. **LIVE** (the automap seam pair,
  returning count 0, survives under RMG-D1). Rejected the reverse edge.
  **Concrete destination path — pinned by RMG-D2(b).** The round-4 mechanical
  resolution places it at
  `crates/mp/qshared/src/common/mp/rmg/rm_automap_symbol_t.rs` (a new `rmg/` folder
  mirroring `oracle/codemp/RMG/RM_Manager.h` ownership), imported as
  `mp_qshared::RmAutomapSymbol`, with the `mp_engine_client` import updated in the
  same commit. Per the type-rosetta discipline
  (`engine-fork-discovery.md:96-107`) the relocation regenerates
  `out/engine/type-rosetta.tsv`; the porter imports by name from the rosetta, not
  by hand-picking a path.
- **RMG-D4e — No `RandomTerrainHandle` newtype** (ruling 21 part 2). `CRandomTerrain`
  was to be a single owned `CmLandScape.random_terrain: Option<RandomTerrain>` with
  no handle. Under RMG-D1 that field is §20-dropped (never constructed);
  `GetRandomTerrain()` models as always-`None`. The decision governs the dropped
  shape-map entry. Rejected a marker/unit handle.
- **RMG-D4f — The engine owns its own q_math LCG as `Engine.common.rng:
  mp_qshared::QRand`, exposed via `EngineHost::flrand`/`irand`** (ruling 21 part 3).
  `mp_qshared` gains a `QRand` type (the stateful LCG the game tier models as
  `bg_channel::rng::Rng`, `crates/mp/game/src/bg_channel/rng.rs`); the engine holds
  a distinct instance (`engine.rs:22`, `common/common.rs:20`). **Under RMG-D1 no
  live RMG path draws it** — its only RMG consumer is the golden-only
  `RMG_CreateSeed` (`cm_randomterrain.cpp:1008`, zero live callers), pinned by
  golden #1. The engine service still exists (it stands for the wider engine).
  Rejected reaching `mp_game`'s LCG: `mp_engine_qcommon` must not depend on `mp_game`.
- **RMG-D4g — `CRMArea*` → `AreaId` + arena owned by `CRMAreaManager`, per §B5**
  (ruling 21 part 4). `AreaId` (a `u32` index newtype rendered like `EntityId`),
  `mAreas` → owned `Vec<RmArea>` (the element's Rust name is `RmArea`, settled by
  ruling 28 / RMG-D1 — the `CRM*→Rm*` family), `mArea` → an `AreaId`, `GetArea` →
  arena lookup. **Under RMG-D1 the area classes are generation-path dead** (only
  constructed during mission spawn); the arena shape is a Divergences shape-map
  entry, never a live seam type. Rejected raw `CRMArea*`/`Rc` (§B5).
- **RMG-D4h — Stored pointers into state owned elsewhere are dropped; the owner is
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
- **RMG-D4i — Prior §F shape (closed-hierarchy enum) + verification stand.**
  `CRMInstance` base+four-subclass tree → one `RmInstance` enum (factory
  `CreateInstance` → `match` on the GP2 group name), per §17 — the hierarchy is
  provably closed (`RM_InstanceFile.cpp:158-178`); the dead `"npc"` branch
  (`:162-166`) is §20-dropped. `CRMPathManager` vectors → `Vec`, `CRMInstanceFile`
  GP2 members → arena borrows. **All generation-path** under RMG-D1 → recorded in
  Divergences as §20 shape-map, not ported. Rejected a `dyn` arena. Verification is
  the §18/DEC-09 TU-harness track (Verification strategy).

RMG-D4 also carries forward the ledger deps unchanged — DEC-01 (renderer
deferred) / DEC-04 (strict per-mode) / DEC-09 (engine verification) — and ruling
25 (generation-path §20-drop, RMG-D1's headline) and the DEDICATED-built oracle
TUs (Verification strategy). No prior decision is re-litigated here; where RMG-D1
reduces a decision's live scope, its subject becomes a §20 shape-map entry rather
than ported work.

## Verification strategy

§F / DEC-09 TU-harness track (RMG-D4i), scoped to the RMG-D1 live surface:

- **Harness** `tools/rmg-oracle/` — compile the unmodified oracle TUs
  (`cm_terrain.cpp`, `cm_randomterrain.cpp`, `RM_Manager.cpp`) standalone against
  stub headers (oracle never edited, §18), **with `DEDICATED` defined** (RMG-D1) so
  the compiled behavior matches the shipped engine: `CreateRandomTerrain` is
  compiled out of the ctor's reachable path, `LoadMission` early-outs, no mission
  spawns. The dumper registers terrain with a fixed config, runs `SetLandScape` +
  `LoadMission` (observing the `false` return + the `#ifndef FINAL_BUILD` banner,
  `RM_Manager.cpp:105-108` — the harness compiles the non-FINAL_BUILD TU, so the
  banner is present in both the oracle and the port), and streams the resulting
  landscape as `SV_SendClientGameState` would. The harness/referee inject the
  fixture-backed `MockHost` (`crates/mp/host-interface/src/mock.rs`, ruling 32 /
  RMG-D3) for FS/print and the `flrand`/`irand` RNG services — `MockHost` replicates
  Raven's `q_math.c` `holdrand` LCG inline (`mock.rs:53-89`), the bit-exact stand-in
  for `Engine.common.rng` (RMG-D4f) until the real `QRand` field lands; no paper-spec
  test double.
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
- **No OpenJK peer** (RMG-D4i) — OpenJK dropped RMG entirely
  (`docs/plans/2026-07-08-mp-engine-build-out.md:425-428`), so the engine-vs-engine
  A/B square cannot exercise these paths. A hard constraint, not a choice.

## Slice hooks

- **Waves 0–4 (early clipmap-trace) — the collision surface (ruling 28 wave
  split).** Per `engine-port-order.tsv` the collision methods and their
  signature-pinning callers land *before* the wave-16 RMG unit:
  `CCMLandScape::WaterCollide` is wave 0, `PatchCollide` wave 3, and their C-track
  callers `CM_TraceThroughTerrain` (`cm_trace.cpp:703`) wave 4, `CM_TestInLeaf`
  (`cm_trace.cpp:262`) wave 5, `CM_PointContents` (`cm_test.cpp:224`) wave 7 —
  all before the wave-15 `CCMLandScape` constructor and wave-17 `CM_RegisterTerrain`.
  Ruling 28 lands the frozen §C collision methods (`patch_collide`/`water_collide`
  + `bounds`/water accessors) with these **early clipmap-trace waves**, not the
  wave-16 §F unit — the wave split this doc owns and notes explicitly. **Producible
  now:** their Seam §C signatures are frozen (ruling 28).
- **Wave 16** (`plan §"RMG (113 fns, wave 16)"` — the subsystem-*completion* wave,
  i.e. the max wave over RMG fns, not a per-fn wave). **Mostly producible now**
  (frozen seams): the reduced live tree as one §F subsystem — `RmManager`
  (lifecycle through the early-out + automap seam + `land()` accessor, Seam-A),
  `CmLandScape`/`CmPatch`/`CmHeightDetails` construction, the snapshot-read
  accessors (`height_map`/`flatten_map`/`real_area`/`get_rand_seed`, Seam §C,
  frozen), `TerrainHandle`, `register_terrain`, `rmg_create_seed` (Seam-B). The
  generation subtree lands only as §20 Divergences shape-map entries (RMG-D1), not
  porter code. **One open dependency (RMG-Q10):** `CmLandScape` construction includes
  the live `LoadTerrainDef` (`cm_terrain.cpp:208`), whose `altitudetexture`/`water`
  cases call `CM_GetShaderInfo` (wider-clipmap machinery, not ported here). The Rust
  binding the port calls through the threaded `CollisionWorld` (§B4) must be pinned
  before this construction path can be transcribed — a non-droppable live path under
  the no-stub rule (`GOAL-engine.md:24-28`). Everything else in the wave is producible
  now. **Hard prerequisites (now satisfied):** the type-rosetta entries for
  `rmAutomapSymbol_t` (relocated to `crates/mp/qshared/src/common/mp/rmg/`,
  RMG-D2(b)/RMG-D4d) / `thandle_t`, and the `EngineHost` trait — **built and green**
  in the Stage-0 crate `mp_host_interface` (`crates/mp/host-interface`, commit
  `4b7f01b0`, RMG-D3), with its `flrand`/`irand` RNG services (the live host call
  sites `register_terrain`'s FS/print + `rmg_create_seed`'s RNG can now be written
  as non-stub bodies, `GOAL-engine.md:24-28`).
- **Wave 20** (`SV_GameSystemCalls`): the RMG syscall arms wire to the frozen
  seams — `G_RMG_INIT` → Seam-A `RmManager` methods (`sv_game.cpp:1624-1638`),
  `G_CM_REGISTER_TERRAIN` → Seam-B `register_terrain` (`sv_game.cpp:1640-1641`).
  `G_SET_ACTIVE_SUBBSP` → `SV_SetActiveSubBSP` (`sv_game.cpp:1620-1622`) is
  out-of-scope clipmap wiring (Non-goals), not a seam edge. **This wave adds the
  `Engine.rmg` field** to the `mp_engine_core` `Engine` struct (ruling 12 / STATE-D5;
  it does not exist in `engine.rs` yet — STATE-Q2 attachment point, `engine.rs:35-36`),
  and needs `CollisionWorld.land_scape`. The field is Engine-assembly work here, **not**
  in this doc's Wave-16 Files roster (State ownership). The `G_RMG_INIT` arm checks
  `!rmg.initialized`, sets it `true` in place of Raven's `new`, then calls
  `set_landscape` / `load_mission` / (the guarded, never-reached) `spawn_mission` —
  the flip is here, not in any method. `load_mission` returns false (RMG-D1), so
  `spawn_mission` is never entered.
- **Wave 22** (`SV_SpawnServer`): `CM_RegisterTerrain` on the map-load path; needs
  Seam-B frozen.

## Open questions

**RMG-Q10 — the Rust binding `LoadTerrainDef` uses to reach `CM_GetShaderInfo`/
`CCMShader`.** `LoadTerrainDef` is LIVE under DEDICATED (In-scope item 2,
`cm_terrain.cpp:208`, no `#ifdef` guard) and calls `CM_GetShaderInfo(shaderName)`
in both its `altitudetexture` case (→ the LIVE `SetShaders`, `:80-83`) and its
`water` case (→ `mWaterContents`/`mWaterSurfaceFlags`, the fields Seam §C freezes as
`water_contents()`/`water_surface_flags()`, `:98-103`). The **ownership half is
answered** from ground truth (Non-goals / State ownership): `CCMShader`/
`CCMShaderText`/`CM_GetShaderInfo` and the shader hash tables are wider-clipmap
machinery (`cm_local.h:77,303-304`; `cm_shader.cpp`; driven by `CM_LoadMap`,
`cm_load.cpp:733,737`), `cmg`-resident (STATE-D2), **not** terrain-owned — so this
doc does **not** port `cm_shader.cpp`. What is **not settled** and needs a design
session: the exact Rust binding `LoadTerrainDef`'s port calls — whether the `cm`
C-track qcommon packet already exposes `CM_GetShaderInfo`/`CCMShader`, where the
Rust type lives / how to import it, and the call form through the threaded
`CollisionWorld` (§B4). Because `LoadTerrainDef` is in-scope construction on a
**non-droppable live path** (no stub/skip allowed — `GOAL-engine.md:24-28`), this
must be pinned before the Wave-16 `CmLandScape` construction can be transcribed.
Escalated (not self-resolved): confirm cm-packet ownership of the shader accessors
and freeze the `CM_GetShaderInfo` binding this doc calls.

**RMG-Q11 — how the frozen `patch_collide(&self, cm: &mut CollisionWorld, …)` is
called from the C-track `cm-trace`/`cm-test` caller.** Ruling 28 froze both the
signature (Seam §C) and the intended call form (`if let Some(land) =
&cm.land_scape { land.patch_collide(cm, …) }`), but the two are mutually
incompatible: `land_scape` is a **field of** `CollisionWorld`, so borrowing
`&cm.land_scape` to name `land` while passing `&mut cm` (the whole struct) into the
same call is a borrow-checker conflict (E0502, `cannot borrow *cm as mutable because
it is also borrowed as immutable`) — verified compiling the exact pattern. This is
**not** a free internal-shape choice: every candidate resolution changes observable
state under **fork-1** (`Com_Error` = Rust panic + `catch_unwind`,
`engine-fork-discovery.md:12-19`). `Option::take`-then-put-back opens an
invalidation window — if `patch_collide` unwinds (any `Com_Error` on its call tree)
before the put-back, `catch_unwind` recovery leaves `CollisionWorld.land_scape`
permanently `None`, a divergence from Raven (a live `cmg.landScape` pointer, no such
window). The alternatives — restructuring `patch_collide` as a `CollisionWorld`
method that reads `self.land_scape` internally, or moving `land_scape` out from under
`CollisionWorld` behind an index/handle — each change the frozen Seam §C signature
and/or the STATE-D2 / Seam §C ownership shape. The doc gives no guidance on which to
use, and the choice interacts with a settled fork ruling (fork-1); per Gate 2 this is
a contested point that **escalates to a design session, not self-resolved**. Scope:
`patch_collide` is the *only* Seam §C method affected — `water_collide`, `bounds`,
and the pure `&self` water/size accessors take no `cm` parameter and compile under
the prescribed split-borrow. Blocks the wave-0–4 `patch_collide` transcription (In
scope item 4 / Slice hooks); the rest of Seam §C is unaffected.

The dry-run gate's three earlier questions remain closed: ruling 28 (2026-07-09,
RMG-D1) closed RMG-Q7 (Area naming), RMG-Q8 (collision-surface scope + signatures)
and RMG-Q9 (read-surface signatures + `RmManager` accessor); rulings 31/32/33
(RMG-D3) discharged the `EngineHost`-signatures Stage-0 prerequisite (the crate is
built, quoted verbatim). All are recorded in Resolved questions.

## Resolved questions

Closed by the 2026-07-09 §F rulings (recorded so a re-reader sees why they left
the open list):

- **RMG-Q1 — Fold the qcommon terrain twins in?** RESOLVED by ruling 16 → RMG-D4a.
- **RMG-Q2 — Are the `RM_Terrain.cpp` client-model classes in the dedicated link
  set?** RESOLVED by ruling 17 → RMG-D4c (no; §20-dropped).
- **RMG-Q3 — Classify the dead Perlin-noise scratch.** RESOLVED by ruling 17 →
  RMG-D4c (§20-drop). Mooted further by RMG-D1 (the whole generation path is dead).
- **RMG-Q4 — Crate placement for `rmAutomapSymbol_t`.** RESOLVED by ruling 21 →
  RMG-D4d (relocate to `mp_qshared`; no `rmg → client` edge).
- **RMG-Q5 — Concrete Rust form of `RandomTerrainHandle`.** RESOLVED by ruling 21 →
  RMG-D4e (no handle); the field itself is now §20-dropped by RMG-D1.
- **RMG-Q6 — Engine-tier owner for the free `flrand`/`irand` LCG.** RESOLVED by
  ruling 21 → RMG-D4f (`Engine.common.rng: QRand`, exposed via `EngineHost`); under
  RMG-D1 used only by the golden-only `RMG_CreateSeed`.
- **The generation-path scope.** RESOLVED by ruling 25 → RMG-D1 (§20-dropped;
  live surface = syscall arms + landscape construction + `LoadMission` early-out).
- **`RmManager.mCurObjective`.** RESOLVED by ruling 17 → RMG-D4c.
- **STATE-Q2 (placement + service halves) for `rmg`.** RESOLVED by ruling 12
  (direct `Engine.rmg` field) + ruling 11 (`EngineHost`) → RMG-D4b.
- **RMG-Q7 — Rust names for the two `Area` classes.** RESOLVED by ruling 28 →
  RMG-D1: `CRMArea` → `RmArea` (`CRM*→Rm*`), the qcommon `CArea` → `CmArea`
  (`CCM*→Cm*`); both remain §20 dead-surface shape-map labels, neither emits a Rust
  type, marker, or stub (RMG-D2(a)).
- **RMG-Q8 — Collision-surface scope + signatures.** RESOLVED by ruling 28 →
  RMG-D1: the per-frame terrain-collision surface is the 4th LIVE item; its
  `CmLandScape` methods are frozen in Seam §C (faithful signatures threaded through
  `&`/`&mut CollisionWorld`) and land with the early clipmap-trace waves 0–4 (the
  wave split, Slice hooks).
- **RMG-Q9 — Read-surface signatures + `RmManager` accessor + stored-handle
  field.** RESOLVED by ruling 28 → RMG-D1: `GetHeightMap`/`GetFlattenMap` → `&[u8]`,
  `get_rand_seed` → `c_ulong` (Seam §C, frozen); the accessor is
  `RmManager::land() -> Option<TerrainHandle>` backed by the private
  `land: Option<TerrainHandle>` field, callers resolving through `CollisionWorld`.
- **The `EngineHost` signatures Stage-0 prerequisite.** RESOLVED by rulings 31/33 →
  RMG-D3: the `mp_host_interface` crate is built and green (commit `4b7f01b0`),
  signatures quoted verbatim; goldens run on the fixture-backed `MockHost` (ruling
  32).

## Files roster

C++-track roster for `.claude/workflows/port-cpp-subsystem.js` (`designPath`).
`mode: mp` throughout (dedicated MP engine; SP twin out of scope, DEC-04). Under
**RMG-D1** the roster is the **reduced live surface only** — the generation-path
classes are NOT porter work orders; they appear as §20 `drop` entries in
`divergences` below (RMG-D4: "dropped-path classes keep their shape-map entries
marked §20").

**Crate dependencies (mechanical).** `mp_engine_rmg`'s `Cargo.toml` gains an
`mp_engine_qcommon` path dependency (RMG-D4a) so `RmManager` can name
`CmLandScape`/`CollisionWorld`/`TerrainHandle` in its frozen pub API. Because every
frozen Seam-A/Seam-B/Seam-C signature takes `&mut impl EngineHost`, **both
`mp_engine_rmg` and `mp_engine_qcommon` also gain an `mp_host_interface` path
dependency** — neither declares it today (their current `Cargo.toml`s depend only on
`mp_qshared` [+ `abi_transport`/`native_platform` for qcommon]); adding it is the
mechanical consequence of the frozen `EngineHost` signatures, not a new edge
decision (the crate is Stage-0/built, RMG-D3). Per RMG-D4d /
RMG-D2(b) `rmAutomapSymbol_t` relocates to
`crates/mp/qshared/src/common/mp/rmg/rm_automap_symbol_t.rs` (a new `rmg/` folder
under `mp_qshared`, already a dependency; `mp_engine_client` import updated in the
same commit), so **no** `mp_engine_client` edge is added. `mp_host_interface`
(commit `4b7f01b0`, RMG-D3) supplies the `EngineHost` trait the seam bodies call.

**File placement (mechanical — no new decision).** Two non-class files the roster
rows below don't spell out, resolved from settled content:

- **`TerrainHandle`** is defined at
  `crates/mp/engine/qcommon/src/terrain_handle.rs` — the exact path the Seam
  definition already pins (a small newtype file beside `collision_world.rs`,
  `crates/mp/engine/qcommon/src/collision_world.rs`), forced by the settled
  `rmg → qcommon` dependency direction (the shared handle must live in `qcommon` or
  lower). Like the `rmAutomapSymbol_t` relocation above, it is a trivial newtype
  pinned in prose, **not** a class-port work order, so it carries no `files:` row —
  the same treatment as its cross-crate sibling. `port-cpp-subsystem` porters that
  name `TerrainHandle` import it from this path.
- **`CmPatch`/`CmHeightDetails`** (the idiomatic renames of `CCMPatch`/
  `CCMHeightDetails`, Divergences) are private construction/collision helpers of
  `CmLandScape` — `mPatches: Vec<CmPatch>` is an owned field, `CmHeightDetails` a
  detail struct; **neither appears in any pub Seam signature** (only `CmLandScape`
  methods are pub; `GetPatch` is private-internal §A1). Per porting-rules §21
  ("one Raven class per file — **private helpers colocate**") they colocate into the
  same TU-named file as `CmLandScape`, `cm_terrain.rs` (matching this subsystem's
  choice of a TU-named top-level file over the per-type `src/cm/` convention). They
  are therefore covered by the existing `cm_terrain.rs` row's porter, **not** a
  separate `files:` row.

```yaml
files:
  # --- mp_engine_rmg (oracle/codemp/RMG/) — LIVE surface only ---
  - { path: crates/mp/engine/rmg/src/rm_manager.rs,  crate: mp_engine_rmg,      mode: mp, class: CRMManager,   summary: "RmManager LIVE lifecycle (RMG-D1): new/SetLandScape/LoadMission (early-outs false — mTerrain always NULL under DEDICATED, RM_Manager.cpp:110-113) + the automap-symbol seam pair GetAutomapSymbolCount/GetAutomapSymbol (return 0/None). SpawnMission is a dead-under-DEDICATED stub kept so the guarded syscall arm compiles; it drives the §20-dropped CRMMission::Spawn. mCurObjective/WriteAutomapSymbols/ProcessAutomapSymbols §20-dropped (RMG-D4c). Private initialized:bool flag flipped at the G_RMG_INIT arm (ruling 12)" }
  # --- mp_engine_qcommon (oracle/codemp/qcommon/) — terrain twins, RMG-D4a ---
  - { path: crates/mp/engine/qcommon/src/cm_terrain.rs,       crate: mp_engine_qcommon, mode: mp, class: CCMLandScape,  summary: "CmLandScape LIVE construction under DEDICATED (RMG-D1, cm_terrain.cpp:116-219): config parse, bounds, heightmap(unpopulated)/flatten(memset-0) alloc, LoadTerrainDef, mPatches/UpdatePatches + CCMPatch collision build, seeded holdrand=0x89abcdef; register_terrain (Seam-B) get-or-create. CCMHeightDetails. CCMPatch→CmPatch and CCMHeightDetails→CmHeightDetails COLOCATE in this file as private construction/collision helpers of CmLandScape (§21 'private helpers colocate'; neither is in any pub Seam signature) — no separate roster row/file. LIVE RUNTIME SURFACE (Seam-B §C, owned here per RMG-D4a — the cm C-track packets exclude CCMLandScape): the snapshot-read accessors GetHeightMap/GetFlattenMap/GetRealArea/get_rand_seed (streamed sv_client.cpp:779-806) AND the per-frame collision methods PatchCollide (cm_terrain.cpp:600)/WaterCollide (:836) + GetWaterContents/GetWaterSurfaceFlags/GetWaterHeight/GetBounds/GetPatchScalarSize the cm-trace/cm-test packets call (cm_trace.cpp:283,760,789). Their idiomatic Rust signatures are FROZEN per ruling 28 (RMG-D1) in Seam §C — faithful, threaded through &/&mut CollisionWorld; the collision methods land with the early clipmap-trace waves 0–4 (wave split), the snapshot accessors (GetHeightMap/GetFlattenMap→&[u8]) with the wave-16 unit. Of GetPatch/GetTerxelLocalCoords/SetShaders/CarveLine/CalcRealCoords: SetShaders (LIVE via LoadTerrainDef cm_terrain.cpp:83), CalcRealCoords (LIVE via UpdatePatches :914) and GetPatch (LIVE via PatchCollide/WaterCollide :681,768,823 + the patch-collision build :256,282) are private-internal (§A1) helpers transcribed here — NOT pub seam; GetTerxelLocalCoords (sole callers commented-out :948-950 in the /*…*/ block :929-969) and CarveLine (sole caller the §20-dropped CarveBezierCurve :1303) have no live caller — §20/dead. LoadTerrainDef's altitudetexture/water cases read CM_GetShaderInfo/CCMShader — the wider-clipmap shader machinery (Non-goals, cm_local.h:77/303-304, cm_shader.cpp; NOT terrain-owned), reached through the threaded CollisionWorld; Rust binding = RMG-Q10. §20-dropped: mRefCount (renderer-only, DEC-01); the twelve cm_landscape.h:247-258 area CM_* wrappers (NOT CM_InitTerrain :246, which is LIVE — folded into register_terrain); and ALL area/carve methods FlattenArea/SaveArea/GetWorldHeight/AreaCollision/GetFirst|NextArea/FractionBelowLevel/CarveBezierCurve/GetFirst|Player|NextObjectiveArea + CArea (Rust name CmArea, ruling 28; their only callers were the generation path, now §20 per RMG-D1); CM_TerrainPatchIterate; the inline flrand/irand/rand_seed (no live caller)" }
  - { path: crates/mp/engine/qcommon/src/cm_randomterrain.rs, crate: mp_engine_qcommon, mode: mp, class: RMG_CreateSeed, summary: "GOLDEN-ONLY: only RMG_CreateSeed (cm_randomterrain.cpp:1008, zero live callers) is ported — it pins Engine.common.rng (QRand) via EngineHost::flrand/irand (RMG-D4f, golden #1). The entire CRandomTerrain/CPathInfo generation class (Generate/Smooth/ParseGenerate) and the dead Perlin path (noiseTable/noisePerm) are §20-dropped (RMG-D1/RMG-D4c) — see divergences" }
```

Existing skeleton already present: `crates/mp/engine/rmg/src/rm_headers/symmetry_t.rs`
(`symmetry_t`, `RM_Headers.h:29-35`) and `rm_path/ermdir.rs` (`ERMDir`,
`RM_Path.h:24-37`) — faithful C enums the generation-path shape-map entries
reference; they are not exercised by the live surface.

## Divergences

Idiomatic §F reshapings (layout-free — these types never cross the module ABI) and
the §20 drops (RMG-D1/RMG-D4c) a transcriber records rather than ports. Under
RMG-D1 the entire generation subtree is `drop`, its §F shape retained here as the
shape-map (RMG-D4).

```yaml
divergences:
  # --- LIVE reshapings ---
  - { class: CRMManager,   kind: reshape, rule: "§B/RMG-D4b", note: "mMission:CRMMission* (RM_Manager.h:13) is §20-DROPPED — no Rust field: CRMMission has no Rust type (§20-dropped, RMG-D1) and mMission is dead under DEDICATED (ctor sets it NULL, RM_Manager.cpp:38; the only reassignment `new CRMMission` at RM_Manager.cpp:135 sits AFTER LoadMission's early-out RM_Manager.cpp:110-113 and is never reached; every read at :194/317/333/351/374/394 is inside the §20-dead methods). Corrects the earlier `owned field` framing — the field is never constructed on the live path (mirrors the mLandScape/mTerrain drops below). Cached CCMLandScape* mLandScape (RM_Manager.h:14) and CRandomTerrain* mTerrain (RM_Manager.h:15) likewise NOT stored — RmManager owns only the TerrainHandle; mTerrain resolves as always-None (RMG-D1, GetRandomTerrain()==0). TheRandomMissionManager → direct Engine.rmg field (ruling 12), no Option; lazy-init via a private initialized:bool flipped at the G_RMG_INIT arm (ctor RM_Manager.cpp:34-42 only zeroes members, so new()/Default and the lazy new collapse to one construction)" }
  - { class: CCMLandScape, kind: reshape, rule: "§B5/RMG-D4h", note: "byte* mHeightMap/mFlattenMap, CCMPatch* mPatches → owned Vec<u8>/Vec<CmPatch>; holdrand LCG stays an inline c_ulong field (Raven `unsigned long`, cm_landscape.h:160; seeded 0x89abcdef cm_terrain.cpp:122; get_rand_seed live-streamed). CCMPatch::owner:CCMLandScape* (cm_landscape.h:93) — a LIVE back-pointer on the patch-collision build (GetAdjacentBrushY, cm_terrain.cpp:246-256) — is DROPPED per §B3; the owning CmLandScape is threaded into the patch-build methods (§B4)" }
  - { class: CCMLandScape, kind: reshape, rule: "§A1/RMG-D4a", note: "LIVE RUNTIME SURFACE — the constructed landscape has two live external readers, both owned by THIS subsystem (RMG-D4a: the cm C-track packets exclude CCMLandScape, so its methods port here, not in a cm packet nor another doc). (1) Snapshot/download read from the server (sv_client.cpp:779-806, via TheRandomMissionManager->GetLandScape()): GetHeightMap (cm_landscape.h:218 — §F.19 UB bytes, excluded from golden compare), GetFlattenMap (:219), GetRealArea (:211), get_rand_seed (:239). (2) Per-frame terrain collision from the cm-trace/cm-test C-track packets (cm_trace.cpp:283,760,789,997,1374 + cm_test.cpp:285-289, non-BSPC, gated com_terrainPhysics->integer && cmg.landScape && CONTENTS_TERRAIN): PatchCollide (decl cm_landscape.h:175, def cm_terrain.cpp:600), WaterCollide (:178 / :836), GetWaterContents (:233), GetWaterSurfaceFlags (:234), GetWaterHeight (:232), GetBounds (:112/:199), GetPatchScalarSize (:207). Idiomatic Rust signatures for all of the above are FROZEN per ruling 28 (RMG-D1) in Seam §C: read accessors GetHeightMap/GetFlattenMap→&[u8], GetRealArea→i32, get_rand_seed→c_ulong; collision methods faithful, threaded through &/&mut CollisionWorld (patch_collide takes &mut CollisionWorld + &mut trace_t out-param; water_collide + accessors are pure &self). The RmManager accessor is land()→Option<TerrainHandle> (callers resolve through CollisionWorld). Collision methods land with the early clipmap-trace waves 0–4 (wave split); read accessors with the wave-16 unit. SetShaders (LIVE via LoadTerrainDef cm_terrain.cpp:83), CalcRealCoords (LIVE via UpdatePatches :914) and GetPatch (LIVE via PatchCollide/WaterCollide :681,768,823 + the patch-collision build :256,282) are private-internal (§A1) helpers transcribed here — NOT pub seam; GetTerxelLocalCoords (sole callers commented-out :948-950 in the /*…*/ block :929-969) and CarveLine (sole caller the §20-dropped CarveBezierCurve :1303) have NO live caller — §20/dead. NONE of the five is a pub seam method (corrects the prior 'no live caller / renderer-only' claim for SetShaders/CalcRealCoords/GetPatch). LoadTerrainDef's altitudetexture/water cases read CM_GetShaderInfo/CCMShader (SetShaders' CCMShader* arg :80-83; the water contentFlags/surfaceFlags reads :98-103 populating mWaterContents/mWaterSurfaceFlags, i.e. water_contents()/water_surface_flags()) — the WIDER-CLIPMAP shader machinery (cm_local.h:77/303-304, cm_shader.cpp; NOT terrain-owned, cmg-resident STATE-D2, Non-goals), reached through the threaded CollisionWorld (§B4); this doc does not port cm_shader.cpp, and the Rust binding LoadTerrainDef calls is the open item RMG-Q10" }
  - { class: rmAutomapSymbol_t, kind: relocate, rule: "RMG-D4d/RMG-D2(b)", note: "ABI type (client.h:149) the rosetta ported in mp_engine_client (crates/mp/engine/client/src/client/rm_automap_symbol_t.rs:9) RELOCATES to crates/mp/qshared/src/common/mp/rmg/rm_automap_symbol_t.rs — a NEW rmg/ folder under mp_qshared mirroring oracle/codemp/RMG/RM_Manager.h ownership (RMG-D2(b)) — so mp_engine_rmg (already depends on mp_qshared) names it; RmManager::automap_symbol returns Option<&RmAutomapSymbol>. mp_engine_client import updated in the SAME commit. No rmg→mp_engine_client edge (client→rmg is the allowed direction). Seam pair is LIVE, returning count 0 under RMG-D1" }
  - { class: "flrand/irand (q_math.c:1432)", kind: reshape, rule: "§B3/RMG-D4f", note: "the free q_math LCG over file-scope holdrand → the engine's OWN mp_qshared::QRand instance Engine.common.rng (engine.rs:22, common/common.rs:20), exposed via EngineHost::flrand/irand (rulings 11+21). Only RMG consumer is the golden-only RMG_CreateSeed (cm_randomterrain.cpp:1008, zero live callers) — no live RMG path draws it (RMG-D1). The game-tier bg_channel::rng::Rng (crates/mp/game/src/bg_channel/rng.rs) is a distinct instance mp_engine_qcommon must NOT reach" }
  # --- §20 GENERATION-PATH DROPS (RMG-D1) — shape-map retained, not ported ---
  - { class: CreateRandomTerrain, kind: drop, rule: "§20/RMG-D1", note: "only call site is in the #else of #ifdef DEDICATED (cm_terrain.cpp:170-188, call :178), so mRandomTerrain stays 0 (:169) — dead code on the DEDICATED build. Was Seam-B create_random_terrain; now dropped, no seam entry" }
  - { class: CRandomTerrain, kind: drop, rule: "§20/RMG-D1/RMG-D4e", note: "whole class (Generate/Smooth/ParseGenerate/CPathInfo) unreachable: constructed only by the dead CreateRandomTerrain. No CmLandScape.random_terrain field (RMG-D4e moot under RMG-D1); GetRandomTerrain() models as always-None. Also §20 the dead Perlin path (noiseTable/noisePerm never written — CM_NoiseInit #if 0 at cm_randomterrain.cpp:17-28). mLandScape:CCMLandScape* back-pointer (cm_randomterrain.h:56) dropped with the class (§B3/RMG-D4h)" }
  - { class: CRMMission, kind: drop, rule: "§20/RMG-D1", note: "never constructed under DEDICATED — LoadMission early-outs at RM_Manager.cpp:110-113 before `new CRMMission` (:135). Load/Spawn/PreSpawn/Smooth/PlaceBridges/ParsePaths/ParseRivers and the objective/node placement all dead. Shape-map (per §17): mLandScape:CRandomTerrain* (RM_Mission.h:64) is a §B3 back-pointer that would be dropped + owner threaded (RMG-D4h); the #ifndef DEDICATED CTerrainMap block (RM_Mission.cpp:1503-1504) is §20 (RMG-D4c)" }
  - { class: RmInstance, kind: drop, rule: "§20/RMG-D1/RMG-D4i", note: "the closed CRMInstance base+4-subclass hierarchy (Bsp/Group/Random/Void, RM_Instance*.cpp) is only built during mission spawn (dead). Shape-map (§17): base+4 → one RmInstance enum, CreateInstance factory → match on GP2 group name; CRMRandomInstance's CRMInstance* mInstance → Box<RmInstance>, CRMGroupInstance's rmInstanceList_t → Vec<RmInstance>; CRMInstance::mArea → AreaId (RMG-D4g); dead \"npc\" branch (RM_InstanceFile.cpp:162-166) §20. Not ported" }
  - { class: CRMAreaManager, kind: drop, rule: "§20/RMG-D1/RMG-D4g", note: "CRMArea/CRMAreaManager only constructed during mission spawn (dead). Shape-map (§B5): mAreas (rmAreaVector_t, RM_Area.h:74,80) → owned Vec<RmArea>, raw CRMArea* handed out by CreateArea/EnumArea → AreaId index newtype (rendered like EntityId), threaded through SetArea/GetArea (RM_Instance.h:72,107). CRMArea's Rust name is RmArea (CRM*→Rm*, settled by ruling 28/RMG-D1). Not ported" }
  - { class: CRMPathManager, kind: drop, rule: "§20/RMG-D1/RMG-D4h", note: "GeneratePaths/GenerateRivers over CRMNode/CRMLoc/CRMCell — only driven by CRMMission (dead). Shape-map (§F): rmNode/Loc/CellVector_t → Vec<Node>/Vec<Loc>/Vec<Cell>, Node(x,y)=mNodes[x+y*mXNodes] index math preserved (RM_Path.h:185); neighbor_x/y → const slices (fork-5). mTerrain:CRandomTerrain* (RM_Path.h:175, set at ctor RM_Path.cpp:56,60) is a §B3 back-pointer dropped + owner threaded (RMG-D4h). Not ported" }
  - { class: CRMInstanceFile, kind: drop, rule: "§20/RMG-D1", note: "GP2-backed instance-file open/close + CreateInstance string factory — only used by the dead mission load. Shape-map: CGenericParser2/CGPGroup* members → borrows into the ported GP2 arena. Not ported" }
  - { class: CRMObjective, kind: drop, rule: "§20/RMG-D1", note: "objective parse + Link — only used by the dead mission load (RM_Objective.cpp). Not ported" }
  - { class: CCMLandScape, kind: drop, rule: "§20/RMG-D1", note: "AREA/CARVE METHODS now dead: FlattenArea (cm_terrain.cpp:1312), SaveArea (:1128), GetWorldHeight (:1011) — previously live only via the generation path (CRMPathManager/CRMInstance/CRMMission::Spawn), all §20 under RMG-D1 — plus AreaCollision/GetFirst|NextArea/FractionBelowLevel/CarveBezierCurve/GetFirst|Player|NextObjectiveArea (cm_terrain.cpp:1488,1412,1462,1379,1245,1422,1442,1472). The twelve cm_landscape.h:247-258 area CM_* free-fn wrappers (defined cm_terrain.cpp:1633-1685; CM_InitTerrain :246 is LIVE, not in this set) and SV_LoadMissionDef (:262, declared-never-defined) are zero-caller drops. CArea (cm_landscape.h:42) appears only as these methods' arg — dead-surface (Rust name CmArea, CCM*→Cm* per ruling 28/RMG-D1; emitted as neither marker nor stub, RMG-D2(a)). Recorded, not ported" }
  - { class: CCMLandScape, kind: drop, rule: "§20/DEC-01/RMG-D4c", note: "mRefCount (cm_landscape.h:138) dropped: its only reader is CM_ShutdownTerrain's count-gated free (cm_load.cpp:1073-1077), whose only caller is the renderer (tr_terrain.cpp:1050, DEC-01); the server frees unconditionally at teardown (cm_load.cpp:800-809). register_terrain still returns the existing TerrainHandle on repeat registration (get-or-create on Option<CmLandScape>, cm_load.cpp:1040-1044). CM_TerrainPatchIterate (free fn :1628 + method :997) dropped — its only callers were the renderer (tr_terrain.cpp:923, DEC-01) and the §20 RM_Terrain.cpp chain" }
  - { class: CRMManager, kind: drop, rule: "§20/RMG-D4c", note: "mCurObjective (RM_Manager.cpp:16) zero-init, never read/written; WriteAutomapSymbols (:424) commented-out; ProcessAutomapSymbols (:442) is a client-side static, dead under DEDICATED. SpawnMission (:391) unreachable under DEDICATED (LoadMission returns false) — kept as a dead stub so the guarded syscall arm compiles, its CRMMission::Spawn-driving body dropped. All zero-caller notes" }
  - { class: CRMManager, kind: drop, rule: "§20/RMG-D1", note: "the twelve declared public/private CRMManager methods with NO live caller (of the 13 the review flagged; the 13th, GetLandScape, is LIVE — see tail) are §20 zero-caller drops (grep of `TheRandomMissionManager->` finds no invocation of any): SetCurPriority (RM_Manager.h:36), GetTerrain (:38), GetCurPriority (:41), Preview (:48), IsMissionComplete (:50), HasTimeExpired (:51), CompleteObjective (:52), CompleteMission (:53), FailedMission (:54), UpdateStatisticCvars (:23) have zero callers anywhere in codemp; GetMission (:39) and AddAutomapSymbol (:43) are called ONLY from the §20-dropped generation path (RM_Instance*/RM_Path.cpp; RM_Manager.cpp:400-410 SpawnMission body). Not ported — same §20 reasoning as the sibling drop above; only new/SetLandScape/LoadMission/GetAutomapSymbolCount/GetAutomapSymbol/GetLandScape (+ the dead SpawnMission stub) survive as the live Seam-A surface. The landscape accessor GetLandScape (:39) IS live (snapshot path) — Rust form land()→Option<TerrainHandle> (ruling 28/RMG-D1), not a drop" }
  - { class: CTerrainMap, kind: drop, rule: "§20/RMG-D4c", note: "whole automap-image builder dead under DEDICATED: its only ctor CM_TM_Create is #ifndef DEDICATED (RM_Mission.cpp:1503-1504); Upload/SaveImageToDisk named by ruling 17. Recorded, not ported (returns to scope if the renderer is un-deferred, DEC-01)" }
  - { class: CRMLandScape, kind: drop, rule: "§20/RMG-D4c", note: "RM_Terrain.cpp client-model chain (CRMLandScape/CCGHeightDetails/CRandomModel/CCGPatch, RM_CreateRandomModels, SpawnPatchModelsWrapper) — graph-confirmed zero engine callers under DEDICATED (ruling 17); reached only from the client (RM_CreateRandomModels ← cl_cgame.cpp:1707). Not ported" }
```
