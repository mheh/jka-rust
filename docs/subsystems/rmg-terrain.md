# RMG + qcommon terrain classes (§F idiomatic reimplementation) Design

> **As-built freeze record.** The signature blocks in this doc are the
> frozen shapes the port was transcribed INTO, quoted as of their freeze
> commit. The idiom-era campaigns (2026-07: #13/#20 `&str`/`String`/`bool`
> waves, DEC-35 typed model views, #19 ctx threading) have since evolved
> some of them — the live crate source is authoritative for CURRENT
> signatures; divergences from today's code are not errors in this record.

Status: FROZEN (user sign-off 2026-07-09)     Supersedes: none
Amended: ruling 49 (2026-07-09) — CDraw32 §20-drop addendum (see Divergences)
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
  **rulings 31/32/33** (2026-07-09: the Stage-0 `mp_host_interface` crate is
  first built and green; goldens run on the fixture-backed
  `MockHost`, ruling 32 — RMG-D3), **ruling 36** (2026-07-09, sixth session:
  `EngineHost` **extended and rebuilt** — commit `a9820853`, **15 methods**:
  `+cvar_integer`, `+sv_time`, `+fs_write_file`, `+model_mdxm`/`model_mdxa` — the
  trait is re-quoted verbatim, RMG-D3), **ruling 38** (2026-07-09, sixth session:
  the terrain-collision entry points become methods **on `CollisionWorld`**
  resolving `self.land_scape` internally — the E0502-proven seam repair, closing
  RMG-Q11 — and `spawn_mission` is **dropped from Seam-A** with the dead
  syscall-arm if-body collapsed per §C10, RMG-D1), **ruling 39d** (2026-07-09,
  sixth session: mechanical roster completions — `CCMPatch`/`CCMHeightDetails` get
  their **own files** per §21, `TerrainHandle` gets its roster row, RMG-D2),
  **ruling 40** (2026-07-09, seventh session: the campaign-wide §F naming rule —
  bare `C`/`CCM`/`CRM` prefixes drop for §F internal types we own; `CmLandScape`/
  `RmArea` already conform, RMG-D6), and **ruling 41** (2026-07-09, seventh session,
  the pass-7 headline: closes **RMG-Q10** — `CM_GetShaderInfo` is a SETTLED extern
  `CollisionWorld` binding landing with the `cm` C-track waves — **and** the
  wave-order type-existence gap, by ruling the campaign-wide "struct DEFINITION
  lands at the earliest wave any method/consumer occupies; bodies follow the tool
  order" principle, RMG-D5), **ruling 46** (2026-07-09, pass 8, the brush-data
  ownership hole: `CmLandScape` owns ONE shared `Vec`-backed brush arena and each
  `CmPatch` range-indexes it — closes RMG-Q13, RMG-D7), and **ruling 47** (2026-07-09,
  pass 8: golden #4 exercises the real `.terrain` parse against a hand-authored
  fixture with a stubbed `CM_GetShaderInfo` extern — closes RMG-Q12 — `load_mission`
  keeps its full faithful signature, and the pass-8 mechanical flags resolve, RMG-D8).
  Ruling 24 is now discharged: this doc **quotes the real, current 15-method
  `EngineHost` trait verbatim** from
  `crates/mp/host-interface/src/engine_host.rs` (commit `a9820853`, RMG-D3), it no
  longer merely cites an unbuilt prerequisite.
- `docs/architecture/state-ownership.md` — the STATE-* ledger: STATE-D5 (the one
  `Engine` island lives in `mp_engine_core`, `crates/mp/engine/core/src/engine.rs:20`),
  STATE-D2 (`Engine.cm: mp_engine_qcommon::CollisionWorld` owns Raven's `cmg`
  clipmap — `state-ownership.md:418`, `collision_world.rs:10`). STATE-Q2's
  **placement half** is resolved by ruling 12 (direct `Engine.rmg` field); its
  service half is ruling 11 (`EngineHost`).
- `docs/decisions.md` — DEC-01 (renderer deferred), DEC-04 (strict per-mode),
  DEC-09 (engine verification: TU harnesses + live peers).
- GP2 is the §F exemplar (`crates/mp/engine/qcommon/src/gp2/`,
  `tools/gp2-oracle/`) and **is a live dependency of the ported surface** — the
  live `CCMLandScape::LoadTerrainDef` (In-scope item 2, `cm_terrain.cpp:39-110`,
  unconditional) GP2-parses the terrain-def file before reading any shader flag.
  The *mission/instance-file* GP2 paths (`CRMInstanceFile`, `CRMMission`) are
  §20-dropped (RMG-D1); the terrain-def parse is **not** — it reaches the already-
  ported `mp_engine_qcommon::gp2` **intra-crate** (both live in `mp_engine_qcommon`),
  so it adds **no new pub seam and no new crate edge** (Seam definition). GP2 also
  remains the design exemplar for the §F shapes recorded in the divergences.

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
   generation, under DEDICATED — with the ctor's `else` branch fatalling when the
   config's `heightMap` key is empty: `Com_Error(ERR_FATAL, "Terrain has no
   heightmap specified\n")`, `cm_terrain.cpp:190-193`, a **live** `EngineHost::error`
   use — the fourth live host method, Seam definition),
   `LoadTerrainDef` (`:208`, unconditional — no `#ifdef DEDICATED` guard). **`LoadTerrainDef`
   first GP2-parses the terrain-def file** (`cm_terrain.cpp:48-58`): `Com_ParseTextFile`
   opens `ext_data/RMG/<terrainDef>.terrain`, **falls back to
   `ext_data/arioche/<terrainDef>.terrain`**, and on a **double miss prints
   `Could not open %s` and returns non-fatally** (`:53-54`) before any shader read;
   the parse reuses the ported `mp_engine_qcommon::gp2` intra-crate (Seam definition).
   Only after a successful parse do its `altitudetexture`/`water` cases read shader flags
   via `CM_GetShaderInfo`, the wider-clipmap machinery — Non-goals; the Rust binding
   it calls is the SETTLED extern `CollisionWorld::cm_get_shader_info` method,
   ruling 41 / RMG-D5, closing the former RMG-Q10.
   Then the patch build
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
   (`cm_trace.cpp:283,760,789`, gated
   `com_terrainPhysics->integer && cmg.landScape && CONTENTS_TERRAIN`; the
   `cm_test.cpp:285-289` water-contents read gates on `cmg.landScape && CONTENTS_TERRAIN`
   only — it is **not** behind `com_terrainPhysics`). Ruling 28
   folded this fourth item into RMG-D1's live enumeration; by RMG-D4a (the `cm`
   C-track packets exclude `CCMLandScape`) these methods are owned by *this*
   subsystem — no other doc. **Per ruling 38** (the E0502-proven seam repair) they
   port as methods **on `CollisionWorld`** (`cm.terrain_patch_collide(…)`,
   `cm.terrain_water_collide(…)`, the water/bounds accessors) that resolve
   `self.land_scape` internally — *not* `CmLandScape` methods taking
   `&mut CollisionWorld` (that shape is struck). Faithful signatures, **frozen now**
   in `## Seam definition` §C (RMG-D1). Per `engine-port-order.tsv` their
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
  the Rust binding it calls is now the **SETTLED** cm-C-track `CollisionWorld`
  method `cm_get_shader_info` (ruling 41 / RMG-D5 — decl `cm_shader.cpp:498`,
  `&mut self` because the `name` overload registers shader text on a miss;
  `Option<&CCMShader>`), owned by the `cm` C-track packet, **not** ported here. This
  closes the former RMG-Q10.
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
   (`:157`) — **or `Com_Error(ERR_FATAL, …)`s if the config's `heightMap` key is
   empty** (`:190-193`, the `else` branch, a live fatal), parses the terrain def
   (`LoadTerrainDef`, `:208`), and builds the
   patch/collision arrays (`mPatches`/`UpdatePatches`, `:211-218`). This is the
   full live construction under DEDICATED; only the heightmap-image / random-terrain
   population is skipped. **`LoadTerrainDef` (`:39-110`) GP2-parses the terrain-def
   file** — `Com_ParseTextFile(ext_data/RMG/<terrainDef>.terrain)` then a
   `ext_data/arioche/…` fallback, non-fatal `Com_Printf` + `return` on a double miss
   (`:48-56`) — via `CGenericParser2` (`common.cpp:2179-2202`: `FS_FOpenFileByMode`
   + `FS_Read` + `parser.Parse`), then reads shader flags through `CM_GetShaderInfo`
   only on a successful parse (`:80,98`).
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
`cm_trace`/`cm_test` is out of scope (a `cm` C-track packet), but the terrain
collision **entry points** it invokes (Raven's `landscape->PatchCollide`/
`WaterCollide` + the `GetBounds`/water accessors — the `cm_trace.cpp:760,789` calls
gated `com_terrainPhysics->integer`, the `cm_test.cpp:285-289` water read gated on
`cmg.landScape && CONTENTS_TERRAIN` only) are owned by *this* subsystem
(RMG-D4a) and ported here — LIVE per ruling 28 (RMG-D1). **Per ruling 38 they port
as methods on `CollisionWorld`** (`terrain_patch_collide` etc.) that resolve the
owned `self.land_scape` internally, with signatures **frozen** in
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
| `CCMLandScape*` (`cmg.landScape`) | `cm_landscape.h:135`; `cm_local.h:155`; `sv_game.cpp:1631` | `mp_engine_qcommon::CollisionWorld.land_scape: Option<CmLandScape>` (a field on the existing STATE-D2 `cmg` owner — `collision_world.rs:10`). `Option` is Raven-faithful: `cmg.landScape` is a nullable pointer set only on a terrain map. **LIVE** — constructed under DEDICATED. **The field (and the `CmLandScape` struct it names) is declared at waves 0–4** with the collision surface — the RMG-D5 / ruling-41 struct-definition-lands-early principle applied to rulings 28+38 (Slice hooks); the value is *populated* later by `register_terrain` / assigned at the Wave-20 arm | `CM_RegisterTerrain` — `cm_load.cpp:1036,1055` | `TerrainHandle` (wrapping `thandle_t`) across the seam; borrow inward |
| `CRandomTerrain*` (`mRandomTerrain` / the `random_terrain` field) | `cm_landscape.h:153`, `cm_randomterrain.h:52` | **dropped** — §20 generation path (RMG-D1): `CreateRandomTerrain` is in the `#else` of `#ifdef DEDICATED` (`cm_terrain.cpp:170-188`), so `mRandomTerrain` stays `0`. No `random_terrain` field is added; `GetRandomTerrain()` is modeled as always-`None`. Shape-map entry in Divergences | (never, under DEDICATED) | — |
| `static CTerrainMap* TerrainMap` | `cm_terrainmap.cpp:14` | **dropped** — §20 (RMG-D4c/ruling 17): only writer `CM_TM_Create` is `#ifndef DEDICATED` (`RM_Mission.cpp:1503-1504`) | — | — |
| `noiseTable` / `noisePerm` | `cm_randomterrain.cpp:14-15` | **dropped** — §20 generation path (RMG-D1/RMG-D4c): the Perlin path is dead code and unreachable under DEDICATED | — | — |
| `Consonants[]`, `CRMPathManager::neighbor_x/y`, `CreateInstance::instanceID` | `cm_randomterrain.cpp:847+`; `RM_Path.h:172-173`; `RM_InstanceFile.cpp:140` | **dropped** — §20 generation path (RMG-D1): const/scratch on the never-constructed mission/path/instance objects | — | — |
| free `flrand`/`irand` global `holdrand` | `q_math.c:1432` | `mp_engine_core::Engine.common.rng: mp_qshared::QRand` — the engine's own q_math LCG instance (RMG-D4f/ruling 21). Exposed via `EngineHost::flrand`/`irand`. **No live RMG draw** under DEDICATED; only the golden-only `RMG_CreateSeed` uses it | `Rand_Init` (`q_math.c:1434`) | `&mut impl EngineHost` |
| `CRMArea*` — `mAreas` arena + `CRMInstance::mArea` | `RM_Area.h:74,80`; `RM_Instance.h:33` | **dropped** — §20 generation path (RMG-D1): `CRMAreaManager`/`CRMArea` are only constructed during mission spawn (never reached). The `AreaId` arena shape (RMG-D4g/ruling 21) is retained as a Divergences shape-map entry | — | — |
| `com_RMG`, `com_terrainPhysics` | `common.cpp:72`; `cm_landscape.h:267` | `EngineCvars` handles (fork-2). `com_RMG` is LIVE (gates `G_RMG_INIT`) | `Cvar_Get` at init | read via cvar accessor |
| `CCMLandScape::holdrand` | `cm_landscape.h:160` | `CmLandScape.holdrand: c_ulong` — an inline per-instance LCG field; seeded `0x89abcdef` in the live ctor (`cm_terrain.cpp:122`) and read by `get_rand_seed` (streamed, `sv_client.cpp:806`). `flrand`/`irand`/`rand_seed` (`cm_terrain.cpp:1548-1580`) are §20 within the class (no live caller). **Not** an external `Rng` type | `CCMLandScape` ctor (`cm_terrain.cpp:122`) | field; see RNG threading |
| `CCMLandScape::mPatchBrushData` (the single shared brush buffer) + `CCMPatch::mPatchBrushData` (`cbrush_s*` per-patch slice) | `cm_landscape.h:151` (CmLandScape `byte*` shared buffer), `:100` (CmPatch `cbrush_s*` slice); `cm_terrain.cpp:213-215` | **RMG-D7 / ruling 46:** `CmLandScape` owns **one** shared `Vec`-backed brush arena mirroring Raven's single `Z_Malloc(size * GetBlockCount())` buffer; each `CmPatch` stores an **offset/length range** into it (no raw pointer, §B5). Raven's per-patch pointer-arithmetic carving (`cm_terrain.cpp:319-321,524,588,925`) → range arithmetic over the shared `Vec`. **LIVE** — the patch-collision build + `terrain_patch_collide`/`GetAdjacentBrushX/Y` (`cm_terrain.cpp:246-256,282`) read it | `CCMLandScape` ctor (`cm_terrain.cpp:213-215`) → `UpdatePatches` slices per patch | owned `Vec` on `CmLandScape`; `CmPatch` holds ranges; sibling reads thread the owning `CmLandScape` (RMG-D4h) |
| `CCMShader`/`cmg.shaders`/`numShaders`; `shaderText`/`shaderTextTable`/`cmShaderTable` | `cm_local.h:77,110-111`; `cm_shader.cpp:28-30` | **not owned here** — wider-clipmap shader machinery on the `cm` C-track qcommon packet (Non-goals). `cmg`-resident, so it lives under `CollisionWorld` (STATE-D2). `LoadTerrainDef` reads it via the SETTLED cm-C-track binding `CollisionWorld::cm_get_shader_info` (ruling 41 / RMG-D5; former RMG-Q10) | `CM_LoadMap` (`cm_load.cpp:733,737`) — NOT terrain | `&mut CollisionWorld` (§B4) |

## Seam definition

RMG crosses **two** boundaries; nothing here crosses the *module* ABI (no
`#[repr(C)]` layout constraint — §F), so all types below are idiomatic.

**The host seam (ruling 11; the crate is BUILT + EXTENDED — RMG-D3).** Every §F
engine service Raven reached through a file-scope global or `gi.`/`Com_` call —
`Com_Printf`/`Com_Error`, cvar reads, FS — is threaded as the one `EngineHost`
services trait. **Rulings 31/33 first built it green; ruling 36 extended and
rebuilt it (commit `a9820853`, now 15 methods: `+cvar_integer`, `+sv_time`,
`+fs_write_file`, `+model_mdxm`/`model_mdxa`).** It lives in the Stage-0 crate
`mp_host_interface` (`crates/mp/host-interface`, package `mp_host_interface`).
Consumers store `&mut dyn EngineHost` (ruling 24), so it is dyn-compatible (no
generic methods, no by-value `Self`). Its full, current trait, **quoted verbatim**
from `crates/mp/host-interface/src/engine_host.rs` so this doc is self-contained
(RMG-D3) — every method's `Source:` cite is preserved:

```rust
/// Raven's host service surface for the server-side game subsystems.
///
/// The method set is fixed by ruling 24 (trace, FS read/free, print/error,
/// `VM_Call`, shared-memory window, `flrand`/`irand`, gentity), extended by
/// ruling 36 (cvar read, `svs.time`, FS write, loader model memory).
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
    /// 19/23/30, transcription-first): the entity-taking icarus/NPCNav arms
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

`Engine` implements the trait via a split-borrow view struct; goldens/referee
inject the fixture-backed `MockHost` (`crates/mp/host-interface/src/mock.rs`,
ruling 32 — DEC-09), whose `flrand`/`irand` replicate Raven's `q_math.c`
`holdrand` LCG. §F methods that touch a service take `&mut impl EngineHost`; the
`CollisionWorld` state is *not* a service and stays a separate threaded param
(§B4). **Under RMG-D1 the live host use is exactly four methods:** `EngineHost::print`
(`Com_Printf` — the RMG banner in `LoadMission`, `RM_Manager.cpp:106-107`, and
`LoadTerrainDef`'s `Could not open %s` double-miss, `cm_terrain.cpp:53`),
`EngineHost::fs_read_file` (the FS reads inside `CM_RegisterTerrain`
construction — including `LoadTerrainDef`'s two `.terrain` lookups, which port
`Com_ParseTextFile`'s `FS_FOpenFileByMode`+`FS_Read`, `common.cpp:2185-2192`),
`EngineHost::flrand`/`irand` (the golden-only `RMG_CreateSeed`'s RNG draws), and
`EngineHost::error` (the ctor's empty-`heightMap` `Com_Error(ERR_FATAL, "Terrain
has no heightmap specified\n")`, `cm_terrain.cpp:190-193` — mapped to
`host.error(errorParm_t::ERR_FATAL, msg) -> !` via the established
`Com_Error`→`error` fork-1 panic/`catch_unwind` model, no new decision). The
ruling-36 additions (`cvar_integer`/`sv_time`/`fs_write_file`/
`model_mdxm`/`model_mdxa`) have **no** RMG live-surface caller — their cites are
nav/ghoul2 sites; the terrain path's `com_RMG`/`com_terrainPhysics` reads live at
the wave-20 syscall arm and the `cm-trace` gate, not inside an RMG method.

**`LoadTerrainDef`'s GP2 parse — reuse, not a new seam.** Raven's stack-local
`CGenericParser2 parse` (`cm_terrain.cpp:42`) ports as a **function-local**
`mp_engine_qcommon::gp2::GenericParser2` (`crates/mp/engine/qcommon/src/gp2/
generic_parser2.rs:21`) — the already-ported §F exemplar in the **same crate** as
`cm_terrain.rs`, so the call is **intra-crate** and introduces **no new pub seam
signature and no new crate edge**. `Com_ParseTextFile(file, parse)`
(`common.cpp:2179-2202`) ports as `host.fs_read_file(file)` (Raven's
`FS_FOpenFileByMode`+`FS_Read`; `None`/missing → `false`) followed by
`parse.parse(&text, clean_first)` (`generic_parser2.rs:100`); the RMG→arioche
fallback and the non-fatal `print`+return on a double miss port as straightforward
control flow (§C10). The subsequent group walk uses the existing GP2 accessors
(`subgroups`/`find_pair_value`, `gp_group.rs:88,152`).

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
    // `CRMManager::SpawnMission` (RM_Manager.cpp:391) is NOT in this impl — ruling 38
    // DROPS it from Seam-A. It is unreachable under DEDICATED (`load_mission` always
    // returns false, sv_game.cpp:1632-1634), and its body drives the §20-dropped
    // `CRMMission::Spawn` (RMG-D1). Rather than a dead stub, the Wave-20 syscall arm
    // collapses the provably-dead `if load_mission { spawn_mission }` to just
    // `load_mission(…)` per porting-rules §C10 (Slice hooks, Wave 20), with a §20
    // zero-execution note. No stub anywhere (GOAL-engine.md no-stub rule).
    /// `CRMManager::GetAutomapSymbolCount` — RM_Manager.cpp:413. **No backing
    /// storage — hardcoded `0`.** Raven's `mAutomapSymbols[MAX_AUTOMAP_SYMBOLS]` /
    /// `mAutomapSymbolCount` pair (`RM_Manager.h:20-21`) is NOT mirrored as an
    /// `RmManager` field: its sole writer `AddAutomapSymbol` (`RM_Manager.cpp:400`,
    /// the only site that increments the count, `:410`) is §20-dropped (never
    /// reached under DEDICATED — it runs only on the dropped mission-spawn path), so
    /// the count stays at its ctor value `0` (`RM_Manager.cpp:41`) for the process
    /// lifetime and a permanently-empty array would be pure dead surface (§20). This
    /// extends the frozen two-field set below (`initialized`/`land`) — no third
    /// field is added; the accessor returns the constant `0`.
    pub fn automap_symbol_count(&self) -> i32;
    /// `CRMManager::GetAutomapSymbol` — RM_Manager.cpp:418. **Returns `None`
    /// unconditionally.** With no backing array (above) and `automap_symbol_count`
    /// always `0`, a well-behaved caller (`SV_WriteRMGAutomapSymbols` walks
    /// `0..count`, `sv_client.cpp:668-684`) never calls this. Raven's body is the
    /// unchecked `&mAutomapSymbols[index]` (`RM_Manager.cpp:420`, UB on any
    /// out-of-range/negative `index`). **§19 (RMG-D8 / ruling 47):** the unchecked C
    /// index becomes a `.get()` → `Option` — the port would express Raven's
    /// `&mAutomapSymbols[index]` as `self.automap_symbols.get(index as usize)` were
    /// there an array, but since there is no backing array (§20) every `index` yields
    /// `None`. Returning `None` for every `index` is the defined-behavior divergence
    /// the frozen `Option` return type already implies (no index is ever valid), so
    /// no array read and no bounds/sign check on real storage occurs; the `.get()`
    /// shape is the ruled §19 rendering, noted here at the site.
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
`CRMManager::CRMManager` null/zero-inits the members the kept Rust field set
mirrors (`mLandScape`/`mTerrain`/`mMission` NULL, `RM_Manager.cpp:34-42`; the only
non-default member, `mCurPriority = 1` at `:39`, is §20-dropped), it is
Default-equivalent: `RmManager::default()` and Raven's lazy `new CRMManager`
collapse to one construction, and the lazy step is only the flag flip. Frozen:
porters add the field and flip it at the Wave-20 syscall arm. `RmManager` also
carries the private `land: Option<TerrainHandle>` (Raven's `mLandScape` cache,
`RM_Manager.h:14`; `None` by Default matching the `NULL`-zeroing ctor,
`RM_Manager.cpp:34-42`), set by `set_landscape` and read by `land()` (ruling 28 /
RMG-D1). **These two are the *complete* `RmManager` field set** — Raven's
`mAutomapSymbols[MAX_AUTOMAP_SYMBOLS]` / `mAutomapSymbolCount` pair
(`RM_Manager.h:20-21`) is **not** carried as a third field: its only writer
`AddAutomapSymbol` is §20-dropped (never reached under DEDICATED,
`RM_Manager.cpp:400,410`), so the array is permanently unwritten and the count is
frozen at `0` (ctor, `RM_Manager.cpp:41`); the `automap_symbol_count`/
`automap_symbol` accessors are the hardcoded `0`/`None` constants above (§20 dead
surface — a stored empty array would never differ from the constant).

**Seam deviation — the added `cm: &mut CollisionWorld` parameter (not a design
change).** Raven's `LoadMission` takes only `qboolean IsServer`
(`RM_Manager.cpp:96`) and reaches the landscape through the `RmManager` members
`mLandScape`/`mTerrain` (set by `SetLandScape`, `RM_Manager.cpp:81-82`) — **all past
the `if (!mTerrain) return false` early-out** (`:110-113`), so under DEDICATED no
landscape is reached at all (RMG-D1). Per §B (no hidden globals), `RmManager` owns
**only** a `land: Option<TerrainHandle>`;
the `CCMLandScape` data lives in `CollisionWorld` (STATE-D2, `collision_world.rs:10`).
So `load_mission` threads the owning `CollisionWorld` as the §B4 substitute for those
dropped members (the dead mission-construction path — `new CRMMission(mTerrain)` /
`assert(mLandScape)`, `RM_Manager.cpp:135,168`, all past the early-out — would resolve
the handle → `CmLandScape` through it). On the live DEDICATED path both `cm` and
`is_server` are **unused** — the body is the `#ifndef FINAL_BUILD` banner print +
`false`. **`load_mission` KEEPS its full faithful signature — RESOLVED by RMG-D8 /
ruling 47:** the `cm`/`is_server` params are retained even though both are dead on
the live path, because `load_mission` is a **live call transcribed 1:1** at the
Wave-20 arm (§A2). Ruling 38 collapsed a dead *call* (`spawn_mission`, whose whole
if-body is unreachable), **not** a signature — so no param-trimming applies to the
live `load_mission`; the full-signature Seam-A form above is frozen. (This is why
`mp_engine_rmg` needs the `mp_engine_qcommon` edge — see "Crate dependencies".)
(Raven's `SpawnMission(qboolean)` is dropped from this seam entirely — ruling 38;
its would-be `cm` parameter is moot.)

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
settled its Rust form). **`SpawnMission` (`RM_Manager.cpp:391`) is likewise NOT in
the impl — ruling 38 dropped it from Seam-A** (it was an earlier draft's dead stub):
`load_mission` always returns false under DEDICATED (RMG-D1), so the Wave-20 syscall
arm collapses the provably-dead `if load_mission { spawn_mission }` to `load_mission(…)`
per porting-rules §C10, with a §20 zero-execution note — no stub (GOAL-engine.md's
no-stub rule). Six live methods remain: `new`/`set_landscape`/`load_mission`/
`automap_symbol_count`/`automap_symbol`/`land`.

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
not in another doc). Ruling 28 settled both readers' scope (LIVE) and their
idiomatic Rust signatures (RMG-D1); **ruling 38 then settled the *receiver* shape
for the second reader (the terrain-collision surface), so this subsection now
FREEZES completely** (RMG-Q11 closed).

**The two readers split by receiver (ruling 38 — the E0502 seam repair):**

1. **Snapshot/download read (In-scope item 3)** — `height_map`/`flatten_map`/
   `real_area`/`get_rand_seed` stay `&self` methods **on `CmLandScape`**. They take
   no `cm`; the snapshot caller resolves the handle from `RmManager::land()` and
   reads the landscape via the immutable split-borrow
   `if let Some(land) = &cm.land_scape { land.height_map() }` — which compiles
   (immutable borrow only).
2. **Per-frame terrain collision (In-scope item 4)** — the collision entry points
   move **onto `CollisionWorld`** as `terrain_*` methods (`cm.terrain_patch_collide(…)`,
   `cm.terrain_water_collide(…)`, the water/bounds accessors) that **resolve
   `self.land_scape` internally**. This is ruling 38's repair of the E0502-proven
   uncompilable seam: an earlier draft froze `patch_collide` as a `CmLandScape`
   method taking `(&self, cm: &mut CollisionWorld, …)`, so the C-track caller had to
   hold `&cm.land_scape` (to name `land`) while passing `&mut cm` into the same call
   — E0502 ("cannot borrow `*cm` as mutable because it is also borrowed as
   immutable"), inexpressible in safe Rust. **That `CmLandScape`-method shape is
   struck.** As a `CollisionWorld` method there is no double borrow: `self` is the
   whole owning struct and `self.land_scape` is resolved inside the method. The
   `patch_collide` checkcount writes into landscape-owned brush data are legal `&mut
   self` mutation (ruling 38); Raven's `WaterCollide`/accessors are `const`
   (`cm_landscape.h:178,199,207,232-234`) → faithful `&self` reads.

Frozen pub API on `mp_engine_qcommon` (two impl blocks — ruling 38):

```rust
impl CmLandScape {
    // --- 1. Snapshot/download read (server, sv_client.cpp:779-806, reached via
    //        TheRandomMissionManager->GetLandScape()->…; In-scope item 3). These
    //        stay CmLandScape &self methods — the caller reads them through the
    //        immutable split-borrow `if let Some(land) = &cm.land_scape { … }`. ---
    /// `CCMLandScape::GetHeightMap` — cm_landscape.h:218. `byte*` → `&[u8]`
    /// (ruling 28). §F.19-UB: unpopulated under DEDICATED — excluded from goldens.
    pub fn height_map(&self) -> &[u8];
    /// `CCMLandScape::GetFlattenMap` — cm_landscape.h:219. `byte*` → `&[u8]`.
    pub fn flatten_map(&self) -> &[u8];
    /// `CCMLandScape::GetRealArea` — cm_landscape.h:211.
    pub fn real_area(&self) -> i32;
    /// `CCMLandScape::get_rand_seed` — cm_landscape.h:239. Raven `unsigned long`.
    pub fn get_rand_seed(&self) -> c_ulong;
}

impl CollisionWorld {
    // --- 2. Per-frame terrain collision (cm-trace/cm-test C-track packets,
    //        cm_trace.cpp:760,789,859,888 (gated com_terrainPhysics->integer &&
    //        cmg.landScape && CONTENTS_TERRAIN) + cm_test.cpp:285-289 (gated
    //        cmg.landScape && CONTENTS_TERRAIN only — NOT com_terrainPhysics), non-BSPC;
    //        In-scope item 4). RULING 38: these are methods ON CollisionWorld that
    //        resolve self.land_scape internally (no double borrow) — NOT CmLandScape
    //        methods taking &mut CollisionWorld (struck). Signatures stay
    //        Raven-faithful (return types unchanged); the caller preserves Raven's
    //        `cmg.landScape != NULL` gate as a `self.land_scape.is_some()` bool
    //        check (holds no borrow across the call), so land_scape is Some at entry. ---
    /// Raven `cmg.landScape->PatchCollide(tw, trace, start, end, checkcount)` —
    /// decl cm_landscape.h:175, def cm_terrain.cpp:600. `&mut self`: the checkcount
    /// writes into landscape-owned brush data are legal mutation (ruling 38);
    /// `trace_t &trace` out-param → `&mut trace_t`. `tw`/`trace_t` are C-track types.
    pub fn terrain_patch_collide(&mut self, tw: &mut traceWork_s, trace: &mut trace_t, start: vec3_t, end: vec3_t, checkcount: i32);
    /// Raven `cmg.landScape->WaterCollide(begin, end, fraction)` — cm_landscape.h:178
    /// / cm_terrain.cpp:836. `const` in Raven → `&self` read (resolves self.land_scape).
    pub fn terrain_water_collide(&self, begin: vec3_t, end: vec3_t, fraction: f32) -> f32;
    /// `CCMLandScape::GetBounds` — cm_landscape.h:199 (`const` → `&self`).
    pub fn terrain_bounds(&self) -> &vec3pair_t;
    /// `CCMLandScape::GetPatchScalarSize` — cm_landscape.h:207.
    pub fn terrain_patch_scalar_size(&self) -> f32;
    /// `CCMLandScape::GetWaterHeight` — cm_landscape.h:232.
    pub fn terrain_water_height(&self) -> f32;
    /// `CCMLandScape::GetWaterContents` — cm_landscape.h:233.
    pub fn terrain_water_contents(&self) -> i32;
    /// `CCMLandScape::GetWaterSurfaceFlags` — cm_landscape.h:234.
    pub fn terrain_water_surface_flags(&self) -> i32;
}
```

The `terrain_*` methods on `CollisionWorld` are thin forwarders: each resolves
`self.land_scape` (`Option<CmLandScape>`) and delegates to the corresponding
private `CmLandScape` method (the `CmLandScape` methods stay private-internal §A1,
reached only through these `CollisionWorld` forwarders — no pub `CmLandScape`
collision seam). Return types are Raven-faithful (§C7): `GetBounds → &vec3pair_t`,
the numeric getters → their scalar type. The caller preserves Raven's
`cmg.landScape != NULL` gate as a `self.land_scape.is_some()` boolean check (which
holds no borrow across the call), so the field is `Some` at entry exactly as
Raven's callers guarantee `cmg.landScape` non-null — the internal `.as_ref()`
resolution mirrors that contract (the None branch is unreachable-by-caller-gate,
matching Raven's NULL-deref-avoidance, §19).

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

**`LoadTerrainDef`'s live shader reads — the SETTLED `cm_get_shader_info` binding
(ruling 41 / RMG-D5, closing RMG-Q10).**
The LIVE `SetShaders` above takes a `CCMShader*` produced by
`CM_GetShaderInfo(shaderName)` (`cm_terrain.cpp:80→83`); `LoadTerrainDef`'s `water`
case likewise calls `CM_GetShaderInfo` and reads `shader->contentFlags`/
`shader->surfaceFlags` into `mWaterContents`/`mWaterSurfaceFlags` (`:98-103`) — the
exact fields Seam §C freezes as `water_contents()`/`water_surface_flags()`.
`CCMShader`/`CM_GetShaderInfo` are **wider-clipmap machinery, not terrain-owned**
(Non-goals / State ownership: `cm_local.h:77,303-304`, `cm_shader.cpp`, `cmg`-resident
per STATE-D2), so this doc does not port them. **Ruling 41 settles the binding
`LoadTerrainDef`'s port calls: an extern method on `CollisionWorld` owned by the
`cm` C-track packet, cited here as frozen** — the port reaches it through the
threaded `CollisionWorld` (§B4):

```rust
impl CollisionWorld {
    /// `CCMShader *CM_GetShaderInfo( const char *name )` — decl `cm_local.h:303`,
    /// def `cm_shader.cpp:498`. Transcribed faithfully from the `name` overload:
    /// `&mut self` because a miss registers a fresh `CCMShader` into `cmShaderTable`
    /// (`Hunk_Alloc` + `CM_ParseShader`, `cm_shader.cpp:512-522`), so the call
    /// mutates the clipmap; `Option<&CCMShader>` mirrors the returned pointer
    /// (the `name` overload never returns NULL, but `Option` unifies with the
    /// `int shaderNum` overload's NULL and keeps the reader gate honest).
    /// **Owned by the `cm` C-track qcommon packet, landing with the `cm` waves —
    /// NOT ported by this doc; cited as a SETTLED extern binding** (ruling 41).
    pub fn cm_get_shader_info(&mut self, name: &str) -> Option<&CCMShader>;
}
```

`CCMShader` is the cm-C-track struct (`cm_local.h:77`); it keeps its Raven name
(the ruling-40 §F naming rule renames only the §F internal types *this* doc owns,
not cm-packet-owned types — RMG-D6).

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

*Numbering note (pass 8).* RMG-D1…RMG-D4 are the established records (rulings ≤39),
threaded through the whole body; RMG-D5 (ruling 41) and RMG-D6 (ruling 40) were the
pass-7 appends. The pass-8 session's two new rulings **append without renumber** —
RMG-D8 (ruling 47) explicitly blesses the RMG-D5/D6 append convention as standing
(no renumber) — as **RMG-D7 (ruling 46)** and **RMG-D8 (ruling 47)**, so every
existing cross-reference stays valid. Read in settlement order they are RMG-D7 (46)
→ RMG-D8 (47) → RMG-D5 (41) → RMG-D6 (40) → the standing RMG-D1…RMG-D4 (whose
"rulings 11-45 stand" RMG-D4 now reaffirms, extended from the pass-7 "11-39").

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
`cm` C-track packets exclude `CCMLandScape`). Ruling 28 froze these as `CmLandScape`
methods threaded through `&`/`&mut CollisionWorld` — **that receiver shape is
superseded by the ruling-38 amendment below** (they are now `CollisionWorld`
methods); the ruling-28 *scope* (LIVE, four items) stands. This doc keeps their
ownership; because `engine-port-order.tsv` demands
their signature-pinning callers at waves 0–4 (before the wave-15/17 constructor),
they **land with the early clipmap-trace waves**, not the wave-16 §F unit (the
wave split, noted explicitly in Slice hooks). Ruling 28 also settles: the snapshot
accessors `GetHeightMap`/`GetFlattenMap` → `&[u8]`; the `RmManager` landscape
accessor `land() -> Option<TerrainHandle>` with `RmManager.land: Option<TerrainHandle>`
(callers resolve through `CollisionWorld`); and the two `Area` class names —
`CRMArea` → **`RmArea`** and the qcommon `CArea` → **`CmArea`** (both remain
§20-dropped shape-map entries, RMG-D4a/RMG-D4g).

*Amendment (ruling 38, 2026-07-09, sixth session — the E0502-proven seam repair;
closes RMG-Q11).* Ruling 28 froze the collision surface as `CmLandScape` methods
taking `&`/`&mut CollisionWorld`; **that shape is struck**. The terrain-collision
entry points become methods **on `CollisionWorld`** — `cm.terrain_patch_collide(…)`,
`cm.terrain_water_collide(…)`, and the water/bounds accessors — that resolve
`self.land_scape` **internally**. Because a `CmLandScape` method that takes
`&mut CollisionWorld` forces the caller to hold `&cm.land_scape` (to name the
receiver) while passing `&mut cm` into the same call — E0502, inexpressible in safe
Rust (verified). As `CollisionWorld` methods there is no double borrow: `self` owns
`land_scape`, resolved inside the method; the `patch_collide` checkcount writes into
landscape-owned brush data are legal `&mut self` mutation. The snapshot-read
accessors (item 3: `height_map`/`flatten_map`/`real_area`/`get_rand_seed`) are
**unaffected** — they take no `cm`, stay `&self` `CmLandScape` methods, and read
through the immutable split-borrow. Seam §C is rewritten to this shape and now
FREEZES. **And:** `spawn_mission` is **DROPPED from Seam-A entirely** — the
`G_RMG_INIT` syscall arm ports with the provably-dead `if load_mission { spawn_mission }`
if-body collapsed to `load_mission(…)` per porting-rules §C10 (`load_mission`
always returns false under DEDICATED, ruling 25), plus a §20 zero-execution note.
**No stub anywhere** (GOAL-engine.md no-stub rule).

**RMG-D2 — Round-4 mechanical resolutions + ruling-39d roster completions.**
Recorded in the ledger, applied here. **(a)** STRIKE the earlier "`//TODO: Port
CArea` marker" fallback: the
engine-wide no-`TODO`/no-`FIXME` rule (`GOAL-engine.md:24-28`) wins
**unconditionally**, so §20-dropped items get a zero-caller §20 note in Divergences,
**never** a marker or stub. **(b)** `rmAutomapSymbol_t` (RMG-D4d) relocates to the
exact path `crates/mp/qshared/src/common/mp/rmg/rm_automap_symbol_t.rs` — a **new
`rmg/` folder** under `mp_qshared` mirroring `oracle/codemp/RMG/RM_Manager.h`
ownership — with the `mp_engine_client` import updated in the **same commit**.
Because the type is RMG-owned (its Raven home is `RM_Manager`'s automap seam), so
its qshared home mirrors that subsystem, not the flat `src/shared/` bucket.
Rejected the `src/shared/` placement an earlier draft named. **(c)** Per **ruling
39d**: `CCMPatch` and `CCMHeightDetails` get their **own files** —
`crates/mp/engine/qcommon/src/cm_patch.rs` and `.../cm_height_details.rs`, beside
`cm_terrain.rs` — **per porting-rules §21 (one Raven class per file)**. Each is a
distinct Raven class, so §21's one-class-per-file governs; **this overrides an
earlier draft's "private helpers colocate into `cm_terrain.rs`" call** — they get
Files-roster rows of their own (Files roster). **(d)** Per **ruling 39d**:
`TerrainHandle`'s prose placement (`crates/mp/engine/qcommon/src/terrain_handle.rs`,
beside `collision_world.rs`) is **confirmed authoritative and gets its own roster
row** — no longer prose-only. (Its cross-crate sibling `rmAutomapSymbol_t` remains
prose-relocated, RMG-D2(b).)

**RMG-D3 — Rulings 31/32/33 built it; ruling 36 extended + rebuilt it (15
methods).** Ruling 24 is discharged. The Stage-0 crate `crates/mp/host-interface`
(package `mp_host_interface`) was first built green by rulings 31/32/33; **ruling
36 (2026-07-09, sixth session) extended and rebuilt it to 15 methods — commit
`a9820853`** (`+cvar_integer` [missing name reads 0], `+sv_time`, `+fs_write_file`,
`+model_mdxm`/`model_mdxa`). This doc **quotes the real, current 15-method
`EngineHost` trait verbatim** from `crates/mp/host-interface/src/engine_host.rs`
(Seam definition) — the doc is self-contained, no unbuilt-prerequisite framing
remains. Goldens/referee run on the fixture-backed `MockHost`
(`crates/mp/host-interface/src/mock.rs`, ruling 32), whose `flrand`/`irand`
replicate Raven's `q_math.c` LCG — so **no paper-spec citations remain** in
Verification strategy. None of the five ruling-36 additions has an RMG live-surface
caller (their cites are nav/ghoul2 sites, `engine_host.rs:112-154`); RMG's live host
use stays the three of Seam definition. Rejected re-deriving the seam or a
subsystem-local test double. Because a built, green Stage-0 crate is the ground truth
ruling 24 promised.

**RMG-D4 — All rulings 11-45 stand, applied to the reduced live surface;
dropped-path classes keep their shape-map entries marked §20.** The prior §F
decisions carry forward verbatim; where RMG-D1 makes their subject dead code, the
decision still governs the shape recorded in Divergences (the §20 shape-map),
not a ported implementation. Ruling 25's generation §20-drop is RMG-D1's headline;
ruling 28's live-collision scope and its `Option<TerrainHandle>` `RmManager` accessor
both stand unchanged — **ruling 38 changed only the method *receiver* shape** (the
collision entry points move onto `CollisionWorld`, RMG-D1 amendment), not the
collision-surface scope or the accessor. Recorded with stable sub-IDs so the body's
cites resolve:

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

**RMG-D5 — `CM_GetShaderInfo` is a SETTLED extern `CollisionWorld` binding, and a
§F struct definition lands at its earliest consumer's wave.** Per **ruling 41**
(user, 2026-07-09, seventh session), which closes both RMG-Q10 and the wave-order
type-existence gap. **(1) The shader binding.** `LoadTerrainDef`'s live
`altitudetexture`/`water` cases reach shader flags through `CM_GetShaderInfo`
(`cm_terrain.cpp:80,98`); that accessor ports as a method **on `CollisionWorld`** —
`fn cm_get_shader_info(&mut self, name: &str) -> Option<&CCMShader>` — transcribed
faithfully from the oracle `name`-overload decl (`cm_local.h:303`, def
`cm_shader.cpp:498`). `&mut self` because a cache miss registers a fresh `CCMShader`
into `cmShaderTable` (`Hunk_Alloc` + `CM_ParseShader` + `insert`,
`cm_shader.cpp:512-522`); `Option<&CCMShader>` mirrors the returned pointer. It is
**owned by the `cm` C-track qcommon packet and lands with the `cm` waves** — this
doc does **not** port it, it **cites it as a frozen extern binding** the ported
`LoadTerrainDef` calls through the threaded `CollisionWorld` (§B4). This closed
RMG-Q10 (the open list was empty at ruling-41 time; a later dry-run reopened the
distinct verification-fixture item RMG-Q12, since closed by ruling 47 / RMG-D8 —
Resolved questions). `CCMShader` keeps its Raven name (the
ruling-40 §F rename touches only §F internal types this doc owns — RMG-D6).
**(2) The struct-definition-lands-early principle (ruled campaign-wide).** A §F/C++
struct **DEFINITION** (all fields, house cites, no method bodies) lands at the
**earliest wave any of its methods or consumers occupies**; the method **bodies**
follow the tool (`engine-port-order.tsv`) order. Applied here: the `CmLandScape`
struct definition — and the `CollisionWorld.land_scape: Option<CmLandScape>` field
that names it — land with the **wave-0–4 collision consumers** (so `land_scape`
type-checks from wave 0), while the ctor / `LoadTerrainDef` / `UpdatePatches`
**bodies** land at their tool waves (15-17). This is a *ruled principle*, not a
mechanical consequence the doc derived: it **replaces** the earlier
draft's ad-hoc "struct-existence prerequisite" framing wherever the Files roster and
Slice hooks describe the early struct landing. Rejected an empty forward-declared
shell or a stub: `GOAL-engine.md:24-28` forbids stubs, and the struct's frozen field
set (Divergences RMG-D4a/RMG-D4h) is complete at first landing — only its populating
logic defers.

**RMG-D6 — The campaign-wide §F naming rule applies; the doc has no stragglers.**
Per **ruling 40** (user, 2026-07-09, seventh session). Bare `C`/`CCM`/`CRM` Raven
prefixes drop for the §F **internal** types this doc owns and reshapes:
`CCMLandScape → CmLandScape`, `CCMPatch → CmPatch`, `CCMHeightDetails →
CmHeightDetails`, `CArea → CmArea`, `CRMManager → RmManager`, `CRMArea → RmArea`,
`CRMInstance → RmInstance`, `rmAutomapSymbol_t → RmAutomapSymbol`. An audit of this
doc finds every owned type already conforms (as ruling 40 notes for
`CmLandScape`/`RmArea`) — **no stragglers**. Types this doc does **not** own keep
their Raven names: `CCMShader` and `CM_GetShaderInfo` are cm-C-track-owned (RMG-D5),
`TerrainHandle`/`AreaId`/`EntityId` are §B5 idiomatic newtypes already, and the
`class:` roster/divergence keys deliberately cite the **Raven** class name (the
port's subject), not the Rust rename. Rejected renaming cm-packet types or the
roster keys: the naming rule scopes to §F-owned internal types, and the roster keys
must name the oracle class being ported.

**RMG-D7 — the per-patch collision-brush data is ONE shared `CmLandScape`-owned
`Vec` arena that each `CmPatch` indexes by range (closes RMG-Q13).** Per **ruling
46** (user, 2026-07-09, pass 8). `CmLandScape` owns **one** shared `Vec`-backed
brush arena, mirroring Raven's single `Z_Malloc(size * GetBlockCount())` buffer
`mPatchBrushData` (`cm_terrain.cpp:213-215`); each `CmPatch` stores an **offset/length
range** into it (an offset + len, §B5 — **not** a raw `struct cbrush_s*`,
`cm_landscape.h:100`), so no `CmPatch` holds a raw pointer. Raven's per-patch
pointer-arithmetic carving transcribes as **range arithmetic** over the shared `Vec`:
`UpdatePatches` slicing `mPatchBrushData + (size * (ix + iy*mBlockWidth))` per patch
(`cm_terrain.cpp:925`), `CCMPatch::Init` taking that slice base (the `patchBrushData`
arg, `cm_terrain.cpp:524,588`), and `CreatePatchPlaneData` (`:302`) sub-dividing it
into `cbrush_t`/`cbrushside_t`/`cplane_t` regions by pointer arithmetic (`brush =
mPatchBrushData; side = (cbrushside_t *)(mPatchBrushData + mNumBrushes); plane =
(cplane_t *)(side + …)`, `cm_terrain.cpp:319-321`, then `InitPlane` `:223` fills each
side/plane pair) all become index/range math over
the arena. This resolves the option (a) topology of the former RMG-Q13 — Raven's
single-allocation layout, chosen over independent per-patch `Vec`s — so the frozen
`CmLandScape`/`CmPatch` field sets (Divergences RMG-D4a/RMG-D4h) are now **complete**
and the RMG-D5 struct-definition-lands-early landing (waves 0–4) carries them with no
open field. **Load-bearing** for `UpdatePatches`, `CmPatch::Init`/`InitPlane`/
`CreatePatchPlaneData`, and the live `terrain_patch_collide`/`GetAdjacentBrushX/Y`
bodies (`cm_terrain.cpp:246-256,282`, which read a sibling patch's brushes) — none
transcribable without it. Rejected independent per-`CmPatch` `Vec<cbrush_s>`/
`Vec<cbrushside_s>` (option (b)): ruling 46 fixed the shared-arena topology (Raven's
one-allocation shape) because the sibling `GetAdjacentBrushX/Y` reads walk across
patches (§B5 — arena on sibling-walking consumers).

**RMG-D8 — golden #4 exercises the real `.terrain` parse against a hand-authored
fixture; `load_mission` keeps its full faithful signature; the pass-8 mechanical
flags resolve (closes RMG-Q12, empties Open questions).** Per **ruling 47** (user,
2026-07-09, pass 8). **(1) Golden #4 / RMG-Q12.** Golden #4 exercises the **real**
`LoadTerrainDef` `.terrain` parse (`cm_terrain.cpp:39-110`) against a **hand-authored
`.terrain` fixture**, driven through `MockHost::fs_read_file` — option (b) of the
former RMG-Q12. The standalone oracle TU **stubs the un-ported `CM_GetShaderInfo`
extern** (cm-C-track-owned, RMG-D5 — the harness runs no `CM_LoadMap`, so
`cmShaderTable` is unpopulated) and **defines the `water_contents()`/
`water_surface_flags()` expectations** the golden pins (the `water` case reads
`shader->contentFlags`/`surfaceFlags` into `mWaterContents`/`mWaterSurfaceFlags`,
`cm_terrain.cpp:98-103`). This closes RMG-Q12 and **empties Open questions**. Rejected
option (a) (both `.terrain` lookups miss → the non-fatal double-miss, fixture-free,
`cm_terrain.cpp:48-56`): ruling 47 chose the fixture that exercises the live
`altitudetexture`/`water` parse. **(2) `load_mission` signature.** `load_mission`
**keeps its full faithful signature** — `load_mission(&mut self, cm: &mut
CollisionWorld, host: &mut impl EngineHost, is_server: bool) -> bool` (Seam-A) — the
`cm`/`is_server` params retained though both are dead on the live DEDICATED path.
It is a **live** call whose Wave-20 arm transcribes 1:1 (porting-rules §A2,
faithful-first); **ruling 38 collapsed a dead *call* (`spawn_mission`), not a
signature**, so no trimming applies here.
Rejected trimming the dead params: a live call is transcribed faithfully, and the
faithful full signature is the frozen Seam-A form. **(3) Mechanical flags, blessed
as-resolved:** the RMG-D5/D6 append numbering **stands** (no renumber — RMG-D7/D8
append likewise, Numbering note); the automap-symbol accessors **hardcode `0`/`None`
with no backing array** — Raven's `mAutomapSymbols[MAX_AUTOMAP_SYMBOLS]`/
`mAutomapSymbolCount` pair (`RM_Manager.h:20-21`) is **not** an `RmManager` field
(its sole writer `AddAutomapSymbol` is §20-dropped, `RM_Manager.cpp:400,410`), the
doc's own dropped-field precedent, now stated as settled (Seam-A); and
`GetAutomapSymbol`'s unchecked C index (`&mAutomapSymbols[index]`,
`RM_Manager.cpp:420`, UB on any out-of-range/negative `index`) becomes **`.get()` →
`Option`** with a §19 divergence note **at the site** (Seam-A) — returning `None` for
every `index` is the defined-behavior divergence the frozen `Option` return already
implies (no index is ever valid, count is always `0`). Rejected any real backing
array or unchecked index: a permanently-empty array is pure §20 dead surface and the
unchecked C read is UB.

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
  test double. **The live `LoadTerrainDef` GP2 parse** (`cm_terrain.cpp:39-110`) drives
  its two `.terrain` path lookups (`ext_data/RMG/…`, `ext_data/arioche/…`) through
  `MockHost::fs_read_file`; **per RMG-D8 / ruling 47 golden #4 pins the real parse
  against a hand-authored `.terrain` fixture** returned for one of those lookups
  (option (b), RMG-Q12 closed), exercising the `altitudetexture`/`water` cases — which
  requires the standalone TU to **stub the un-ported `CM_GetShaderInfo` extern**
  (cm-C-track-owned, RMG-D5 — the harness runs no `CM_LoadMap`, so `cmShaderTable` is
  unpopulated) and to **define the `water_contents()`/`water_surface_flags()`
  expectations** the golden pins (`cm_terrain.cpp:98-103`).
- **Goldens** (committed, so `cargo test` needs no C++):
  1. `RMG_CreateSeed` seed-string + hash streams for a fixed `Engine.common.rng`
     (`QRand`) seed — the golden-only helper (zero live callers) pinning the engine
     LCG via `EngineHost::flrand`/`irand`. **Kept** (ruling 25 drops #2/#3 only).
  2. **Dropped** (RMG-D1) — the post-`Generate` heightmap/flatten bytes: no
     generation runs under DEDICATED.
  3. **Dropped** (RMG-D1) — the automap-symbol list after a full mission spawn:
     no spawn runs; the count is `0`.
  4. **The dedicated-server outcome golden** (RMG-D1, "pin exactly that";
     **exercises the real `.terrain` parse against a hand-authored fixture** per
     RMG-D8 / ruling 47): `register_terrain` builds the `CmLandScape` under DEDICATED,
     its `LoadTerrainDef` **GP2-parsing the committed `.terrain` fixture** (option (b),
     RMG-Q12) so the `altitudetexture`/`water` cases run — the standalone oracle TU
     **stubs the un-ported `CM_GetShaderInfo` extern** (cm-C-track-owned, RMG-D5; no
     `CM_LoadMap` populates `cmShaderTable`) and the golden **defines the
     `water_contents()`/`water_surface_flags()` expectations** the `water` case
     produces (`cm_terrain.cpp:98-103`). `G_RMG_INIT` then runs `SetLandScape` +
     `LoadMission → false` (no `SpawnMission`); and the snapshot stream reproduces
     `GetFlattenMap()` (memset-0, `cm_terrain.cpp:161`), `get_rand_seed()`
     (`0x89abcdef`), and `count = 0` automap symbols — byte-for-byte matching the
     DEDICATED-built oracle.
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
  Ruling 28 lands the frozen §C collision entry points with these **early
  clipmap-trace waves**, not the wave-16 §F unit — the wave split this doc owns and
  notes explicitly. Per **ruling 38** these are the `CollisionWorld` methods
  `terrain_patch_collide`/`terrain_water_collide` + the `terrain_bounds`/water
  accessors (resolving `self.land_scape` internally), with the private `CmLandScape`
  collision helpers they forward to colocating in `cm_terrain.rs`. **Producible
  now:** their Seam §C signatures are frozen (rulings 28 + 38, RMG-Q11 closed).

  **Struct definition lands at its earliest consumer's wave (the RMG-D5 / ruling-41
  campaign-wide principle).** Ruling 41(2) rules, campaign-wide, that a §F/C++ struct
  **DEFINITION** (all fields, house cites, no method bodies) lands at the earliest
  wave any of its methods or consumers occupies, with the **bodies** following the
  tool (`engine-port-order.tsv`) order. Applied here: because ruling 38 makes the
  collision entry points `CollisionWorld` methods that resolve `self.land_scape:
  Option<CmLandScape>` (STATE-D2, the `cmg.landScape` owner row) and ruling 28 lands
  those methods at waves 0–4, the `CmLandScape` **struct definition** and the
  `CollisionWorld.land_scape` field land **at the wave-0–4 landing** — otherwise the
  field type and the field-reading `terrain_*` methods (water
  height/contents/surface-flags, bounds, patch-scalar, and the `PatchCollide`
  checkcount/brush reads) cannot type-check. The struct's Rust field set is now
  **fully frozen in Divergences** (RMG-D4a/RMG-D4h: `mHeightMap`/`mFlattenMap`
  → `Vec<u8>`, `mPatches` → `Vec<CmPatch>`, the inline `holdrand` `c_ulong`, the
  dropped `owner` back-pointer, the water/bounds/patch fields the accessors read
  — `cm_landscape.h`); the former last gap — the per-patch collision-brush buffer
  `mPatchBrushData` (`cm_landscape.h:100`, `cm_terrain.cpp:213-215`) — is now
  **settled by RMG-D7 / ruling 46**: `CmLandScape` owns ONE shared `Vec`-backed brush
  arena (Raven's single `Z_Malloc` topology) and each `CmPatch` holds an offset/length
  **range** into it, so this wave-0–4 landing carries a *complete* definition. The
  early landing carries the **real definition**, not a forward-declared shell (that
  ad-hoc "shell"/"struct-existence prerequisite" framing is retired by RMG-D5). The wave-16 `cm_terrain.rs` roster row (Files roster)
  therefore fills only the **construction logic** (the ctor / `LoadTerrainDef` /
  `UpdatePatches` build + the snapshot-read accessors `height_map`/`flatten_map`/
  `real_area`/`get_rand_seed`), it is **not** the struct's first declaration.
  `CmPatch` / `CmHeightDetails` (their own files per RMG-D2(c) / ruling 39d) land with
  the `CmLandScape` struct they compose, at this same wave-0–4 point, for the same
  reason. `engine-port-order.tsv` orders at *function* granularity
  (`CCMLandScape::WaterCollide` wave 0, the ctor `CCMLandScape::CCMLandScape` wave
  15), so the tool's function order legitimately *predates* the type's construction
  wave; per RMG-D5 the struct **definition** is introduced with its first use (the
  wave-0–4 collision methods), the ctor **body** fills at wave 16 — and no stub
  (`GOAL-engine.md:24-28`), because the early definition carries its real frozen
  fields; only its populating logic is deferred.
- **Wave 16** (`plan §"RMG (113 fns, wave 16)"` — the subsystem-*completion* wave,
  i.e. the max wave over RMG fns, not a per-fn wave). **Mostly producible now**
  (frozen seams): the reduced live tree as one §F subsystem — `RmManager`
  (lifecycle through the early-out + automap seam + `land()` accessor, Seam-A),
  `CmLandScape`/`CmPatch`/`CmHeightDetails` construction, the snapshot-read
  accessors (`height_map`/`flatten_map`/`real_area`/`get_rand_seed`, Seam §C,
  frozen), `TerrainHandle`, `register_terrain`, `rmg_create_seed` (Seam-B). The
  generation subtree lands only as §20 Divergences shape-map entries (RMG-D1), not
  porter code. **The former open dependency RMG-Q10 is now settled (ruling 41 /
  RMG-D5):** `CmLandScape` construction includes the live `LoadTerrainDef`
  (`cm_terrain.cpp:208`), whose `altitudetexture`/`water` cases call
  `CM_GetShaderInfo` (wider-clipmap machinery, not ported here); the binding the port
  calls through the threaded `CollisionWorld` (§B4) is the frozen extern
  `CollisionWorld::cm_get_shader_info` (Seam §C, RMG-D5), landing with the `cm`
  C-track waves. With that pinned, this construction path is fully transcribable — a
  non-droppable live path under the no-stub rule (`GOAL-engine.md:24-28`) with no
  remaining hole. Everything in the wave is producible now.
  **Hard prerequisites (now satisfied):** the type-rosetta entries for
  `rmAutomapSymbol_t` (relocated to `crates/mp/qshared/src/common/mp/rmg/`,
  RMG-D2(b)/RMG-D4d) / `thandle_t`, and the `EngineHost` trait — **built and green**
  in the Stage-0 crate `mp_host_interface` (`crates/mp/host-interface`, extended to
  15 methods at commit `a9820853`, RMG-D3), with its `flrand`/`irand` RNG services (the live host call
  sites `register_terrain`'s FS/print + `rmg_create_seed`'s RNG can now be written
  as non-stub bodies, `GOAL-engine.md:24-28`).
- **Wave 20** (`SV_GameSystemCalls`): the RMG syscall arms wire to the frozen
  seams — `G_RMG_INIT` → Seam-A `RmManager` methods (`sv_game.cpp:1624-1638`),
  `G_CM_REGISTER_TERRAIN` → Seam-B `register_terrain` (`sv_game.cpp:1640-1641`).
  `G_SET_ACTIVE_SUBBSP` → `SV_SetActiveSubBSP` (`sv_game.cpp:1620-1622`) is
  out-of-scope clipmap wiring (Non-goals), not a seam edge. **This wave adds the
  `Engine.rmg` field** to the `mp_engine_core` `Engine` struct (ruling 12 / STATE-D5;
  it does not exist in `engine.rs` yet — STATE-Q2 attachment point, `engine.rs:35-36`),
  and **consumes** `CollisionWorld.land_scape` (this arm assigns it via
  `register_terrain`). That field is **not** introduced here — the waves-0–4 collision
  landing above already declares `CollisionWorld.land_scape: Option<CmLandScape>` (its
  struct-existence prerequisite), so by Wave 20 it exists and this arm only writes it.
  The `Engine.rmg` field, by contrast, **is** introduced here — Engine-assembly work,
  **not** in this doc's Wave-16 Files roster (State ownership). The `G_RMG_INIT` arm checks
  `!rmg.initialized`, sets it `true` in place of Raven's `new`, then calls
  `set_landscape` / `load_mission`. Per **ruling 38** the provably-dead
  `if load_mission(qtrue) { spawn_mission(qtrue) }` (Raven `sv_game.cpp:1632-1634`)
  **collapses to `load_mission(…)` alone** per porting-rules §C10 — `load_mission`
  always returns false under DEDICATED (RMG-D1), so the if-body is dead; it ports
  with a §20 zero-execution note and **no `spawn_mission` call or stub anywhere**
  (there is no `spawn_mission` method — RMG-D1 amendment). The `initialized` flip
  is here, not in any method.
- **Wave 22** (`SV_SpawnServer`): `CM_RegisterTerrain` on the map-load path; needs
  Seam-B frozen.

## Open questions

**Empty.** RMG-Q1…RMG-Q11 were closed by the 2026-07-09 §F rulings ≤41; the pass-8
session (rulings 46, 47) closes the last two — **RMG-Q13** (the §17 brush-arena
ownership shape → RMG-D7 / ruling 46) and **RMG-Q12** (the golden-#4 `.terrain`
fixture + `CM_GetShaderInfo` extern-stub → RMG-D8 / ruling 47). All are in Resolved
questions. No transcription-blocking hole and no verification-design hole remains, so
the doc is REVIEWED-eligible; the header advances DRAFT → REVIEWED on the adversarial
gate per doc-standards rule 7 (an author cannot self-attest that gate).

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
  RMG-D1 (scope, LIVE) and **ruling 38** → RMG-D1 (receiver shape): the per-frame
  terrain-collision surface is the 4th LIVE item; its entry points are frozen in
  Seam §C as **`CollisionWorld` methods** (`terrain_patch_collide` etc., resolving
  `self.land_scape` internally — ruling 38 struck the earlier `CmLandScape`-method
  shape) and land with the early clipmap-trace waves 0–4 (the wave split, Slice hooks).
- **RMG-Q9 — Read-surface signatures + `RmManager` accessor + stored-handle
  field.** RESOLVED by ruling 28 → RMG-D1: `GetHeightMap`/`GetFlattenMap` → `&[u8]`,
  `get_rand_seed` → `c_ulong` (Seam §C, frozen, stay `CmLandScape` `&self` methods);
  the accessor is `RmManager::land() -> Option<TerrainHandle>` backed by the private
  `land: Option<TerrainHandle>` field, callers resolving through `CollisionWorld`.
- **RMG-Q10 — the Rust binding `LoadTerrainDef` uses to reach
  `CM_GetShaderInfo`/`CCMShader`.** RESOLVED by **ruling 41** → RMG-D5: it is the
  frozen extern method `CollisionWorld::cm_get_shader_info(&mut self, name: &str) ->
  Option<&CCMShader>`, transcribed faithfully from the `name`-overload decl
  (`cm_local.h:303`, def `cm_shader.cpp:498`; `&mut self` because a miss registers
  shader text). It is **owned by the `cm` C-track qcommon packet** and lands with the
  `cm` waves; this doc cites it as settled and does not port `cm_shader.cpp`. The
  ownership half was already answered from ground truth (Non-goals / State ownership:
  `CCMShader`/`CM_GetShaderInfo` are `cmg`-resident wider-clipmap machinery driven by
  `CM_LoadMap`, `cm_load.cpp:733,737`, STATE-D2); ruling 41 settled the remaining
  binding half. RMG-Q10 was the last open item at ruling-41 time; a later dry-run
  then reopened RMG-Q12 (a distinct verification-fixture item, since closed by
  ruling 47 / RMG-D8 — see the RMG-Q12 entry above).
- **RMG-Q11 — the E0502 collision-seam borrow.** RESOLVED by **ruling 38** → RMG-D1:
  the frozen `patch_collide(&self, cm: &mut CollisionWorld, …)` `CmLandScape`-method
  shape was inexpressible in safe Rust (holding `&cm.land_scape` while passing
  `&mut cm`). Ruling 38 moves the collision entry points **onto `CollisionWorld`**
  (`terrain_patch_collide` etc.) resolving `self.land_scape` internally — no double
  borrow; the `patch_collide` checkcount writes are legal `&mut self` mutation. Seam
  §C is rewritten to this shape and now freezes.
- **The `EngineHost` signatures Stage-0 prerequisite.** RESOLVED by rulings 31/33 →
  RMG-D3 (first built) and **ruling 36** → RMG-D3 (extended + rebuilt): the
  `mp_host_interface` crate is built and green, now **15 methods at commit
  `a9820853`**, the trait quoted verbatim; goldens run on the fixture-backed
  `MockHost` (ruling 32).
- **RMG-Q12 — the golden-#4 `.terrain` fixture + `CM_GetShaderInfo` extern-stub.**
  RESOLVED by **ruling 47** → RMG-D8: golden #4 exercises the **real** `LoadTerrainDef`
  parse against a **hand-authored `.terrain` fixture** (option (b)) through
  `MockHost::fs_read_file`; the standalone oracle TU **stubs the un-ported
  `CM_GetShaderInfo` extern** (cm-C-track-owned, RMG-D5 — no `CM_LoadMap` runs) and
  **defines the `water_contents()`/`water_surface_flags()` expectations** the golden
  pins (`cm_terrain.cpp:98-103`). This was the last verification-design hole; closing
  it empties Open questions.
- **RMG-Q13 — the §17 ownership shape of the per-patch collision-brush data.**
  RESOLVED by **ruling 46** → RMG-D7: `CmLandScape` owns **one** shared `Vec`-backed
  brush arena mirroring Raven's single `Z_Malloc(size * GetBlockCount())` buffer
  (`cm_terrain.cpp:213-215`); each `CmPatch` stores an offset/length **range** into it
  (not a raw `cbrush_s*`, `cm_landscape.h:100`) — option (a), Raven's single-allocation
  topology, chosen over independent per-patch `Vec`s. Raven's pointer-arithmetic
  carving (`cm_terrain.cpp:319-321,524,588,925`) transcribes as range arithmetic. This was
  the last transcription-blocking hole; with it settled the `CmLandScape`/`CmPatch`
  frozen field sets are complete (Divergences RMG-D4a/RMG-D4h).

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
(15-method trait at commit `a9820853`, RMG-D3) supplies the `EngineHost` trait the
seam bodies call.

**File placement — RULING 39d (mechanical roster completions).** Ruling 39d settles
the three non-`CRMManager` files as their own roster rows (below), overriding an
earlier draft's colocate/prose-only calls:

- **`TerrainHandle`** is defined at
  `crates/mp/engine/qcommon/src/terrain_handle.rs` — a small newtype file beside
  `collision_world.rs` (`crates/mp/engine/qcommon/src/collision_world.rs`), forced by
  the settled `rmg → qcommon` dependency direction (the shared handle must live in
  `qcommon` or lower). **Ruling 39d confirms this prose placement as authoritative
  and gives it its own `files:` row** (no longer prose-only). `port-cpp-subsystem`
  porters name `TerrainHandle` and import it from this path.
- **`CCMPatch`/`CCMHeightDetails`** (idiomatic renames `CmPatch`/`CmHeightDetails`,
  Divergences) are private construction/collision helpers of `CmLandScape` —
  `mPatches: Vec<CmPatch>` is an owned field, `CmHeightDetails` a detail struct;
  **neither appears in any pub Seam signature** (`GetPatch` is private-internal §A1).
  **Ruling 39d gives each its OWN file — `cm_patch.rs` and `cm_height_details.rs`,
  beside `cm_terrain.rs` — per porting-rules §21 (one Raven class per file).** Each
  is a distinct Raven class, so §21's one-class-per-file governs; this **overrides
  the earlier "private helpers colocate into `cm_terrain.rs`" call**. They get
  Files-roster rows of their own (below).

**Struct definitions land at their earliest consumer's wave (RMG-D5 / ruling 41).**
The `cm_terrain.rs` / `cm_patch.rs` / `cm_height_details.rs` roster rows below are
each **one file** that lands in two tool passes, per the RMG-D5 campaign-wide
principle: the `CmLandScape` / `CmPatch` / `CmHeightDetails` **struct DEFINITIONS**
(their frozen field sets, Divergences RMG-D4a/RMG-D4h — no method bodies; the former
last-open field, the per-patch collision-brush buffer `mPatchBrushData`, is now
settled by RMG-D7 / ruling 46 — a shared `CmLandScape` `Vec` arena each `CmPatch`
range-indexes, so the field set is complete) land with the **wave-0–4 collision
consumers** so `CollisionWorld.land_scape:
Option<CmLandScape>` and the field-reading `terrain_*` methods type-check; the ctor
/ `LoadTerrainDef` / `UpdatePatches` **bodies** fill at the wave-15–17 tool order.
This is the ruled principle, **not** a "shell" — the early landing carries the real
definition, so no stub exists at any point (`GOAL-engine.md:24-28`). The row summaries
below describe the whole file (both passes); the porter splits by the tool wave.

```yaml
files:
  # --- mp_engine_rmg (oracle/codemp/RMG/) — LIVE surface only ---
  - { path: crates/mp/engine/rmg/src/rm_manager.rs,  crate: mp_engine_rmg,      mode: mp, class: CRMManager,   summary: "RmManager LIVE lifecycle (RMG-D1): SIX live methods new/SetLandScape/LoadMission (early-outs false — mTerrain always NULL under DEDICATED, RM_Manager.cpp:110-113)/GetLandScape→land()→Option<TerrainHandle> + the automap-symbol seam pair GetAutomapSymbolCount/GetAutomapSymbol (return 0/None). SpawnMission is DROPPED from Seam-A entirely (ruling 38) — NOT a stub: the Wave-20 G_RMG_INIT arm collapses the provably-dead `if load_mission { spawn_mission }` to load_mission(…) per porting-rules §C10 + a §20 zero-execution note. mCurObjective/WriteAutomapSymbols/ProcessAutomapSymbols and the 12 zero-caller CRMManager methods §20-dropped (RMG-D4c, divergences). Private initialized:bool flag flipped at the G_RMG_INIT arm (ruling 12); private land:Option<TerrainHandle> (RM_Manager.h:14 mLandScape cache, None by Default)" }
  # --- mp_engine_qcommon (oracle/codemp/qcommon/) — terrain twins, RMG-D4a ---
  - { path: crates/mp/engine/qcommon/src/cm_terrain.rs,       crate: mp_engine_qcommon, mode: mp, class: CCMLandScape,  summary: "CmLandScape LIVE construction under DEDICATED (RMG-D1, cm_terrain.cpp:116-219): config parse, bounds, heightmap(unpopulated)/flatten(memset-0) alloc — with the ctor else-branch empty-heightMap fatal host.error(errorParm_t::ERR_FATAL, \"Terrain has no heightmap specified\\n\") (cm_terrain.cpp:190-193, the 4th live EngineHost method), LoadTerrainDef (cm_terrain.cpp:39-110: GP2-parses ext_data/RMG/<terrainDef>.terrain, falls back to ext_data/arioche/…, non-fatal print+return on double miss :48-56 — Com_ParseTextFile ports as host.fs_read_file + a function-local mp_engine_qcommon::gp2::GenericParser2.parse, INTRA-CRATE, no new pub seam/edge; §C10 fallback control flow), mPatches/UpdatePatches + CmPatch collision build over the shared brush arena (RMG-D7/ruling 46: the single Z_Malloc(size*GetBlockCount()) buffer mPatchBrushData cm_terrain.cpp:213-215 becomes ONE CmLandScape-owned Vec-backed arena; each CmPatch holds an offset/length RANGE into it, §B5, no raw pointer; UpdatePatches' per-patch slice cm_terrain.cpp:925 + Init slice base :524,588 + CreatePatchPlaneData carve :302,:319-321 → range arithmetic), seeded holdrand=0x89abcdef; register_terrain (Seam-B) get-or-create. CmPatch (CCMPatch) and CmHeightDetails (CCMHeightDetails) get their OWN files cm_patch.rs / cm_height_details.rs per ruling 39d (§21 one class per file) — SEPARATE roster rows below; mPatches: Vec<CmPatch> is an owned field here, neither type is in any pub Seam signature. RUNTIME SURFACE split by receiver (RMG-D4a — the cm C-track packets exclude CCMLandScape): (1) snapshot-read accessors GetHeightMap/GetFlattenMap/GetRealArea/get_rand_seed stay CmLandScape &self methods (streamed sv_client.cpp:779-806; GetHeightMap/GetFlattenMap→&[u8], land with the wave-16 unit). (2) per-frame collision — PER RULING 38 these are methods ON CollisionWorld (terrain_patch_collide/terrain_water_collide + terrain_bounds/terrain_water_height/terrain_water_contents/terrain_water_surface_flags/terrain_patch_scalar_size) that resolve self.land_scape internally (NOT CmLandScape methods taking &mut CollisionWorld — struck; E0502 repair, RMG-Q11 closed); they forward to the private CmLandScape PatchCollide (cm_terrain.cpp:600)/WaterCollide (:836)+getters and land with the early clipmap-trace waves 0–4 (wave split, cm_trace.cpp:760,789,859,888 + cm_test.cpp:285-289). FROZEN in Seam §C (rulings 28+38). Private-internal §A1 helpers transcribed here (NOT pub seam): SetShaders (LIVE via LoadTerrainDef cm_terrain.cpp:83), CalcRealCoords (LIVE via UpdatePatches :914), GetPatch (LIVE via PatchCollide/WaterCollide :681,768,823 + patch-collision build :256,282). §20/dead (no live caller): GetTerxelLocalCoords (callers commented-out :948-950 in /*…*/ :929-969), CarveLine (caller the §20-dropped CarveBezierCurve :1303). LoadTerrainDef's altitudetexture/water cases read CM_GetShaderInfo/CCMShader — wider-clipmap shader machinery (Non-goals, cm_local.h:77/303-304, cm_shader.cpp; NOT terrain-owned), reached through the threaded CollisionWorld; Rust binding = SETTLED extern CollisionWorld::cm_get_shader_info(&mut self, name:&str)->Option<&CCMShader> (ruling 41/RMG-D5, decl cm_shader.cpp:498; owned by the cm C-track packet, landing with the cm waves, NOT ported here — former RMG-Q10, now closed). §20-dropped: mRefCount (renderer-only, DEC-01); the twelve cm_landscape.h:247-258 area CM_* wrappers (NOT CM_InitTerrain :246, LIVE — folded into register_terrain); ALL area/carve methods FlattenArea/SaveArea/GetWorldHeight/AreaCollision/GetFirst|NextArea/FractionBelowLevel/CarveBezierCurve/GetFirst|Player|NextObjectiveArea + CArea (Rust name CmArea, ruling 28; only callers were the generation path, now §20); CM_TerrainPatchIterate; inline flrand/irand/rand_seed (no live caller)" }
  - { path: crates/mp/engine/qcommon/src/cm_patch.rs,         crate: mp_engine_qcommon, mode: mp, class: CCMPatch,        summary: "CmPatch (CCMPatch, cm_landscape.h) — its OWN file per ruling 39d (§21 one class per file), beside cm_terrain.rs. The per-instance collision patch built by CmLandScape::UpdatePatches; owned as mPatches: Vec<CmPatch>. Carries the patch collision data + the adjacency helpers GetAdjacentBrushX/Y (cm_terrain.cpp:246-256,282) reached from the LIVE patch-collision build and terrain_patch_collide. §B3/RMG-D4h: the owner:CCMLandScape* back-pointer (cm_landscape.h:93) is a LIVE back-pointer — DROPPED; the owning CmLandScape is threaded into the patch-build methods (§B4). RMG-D7/ruling 46: the raw struct cbrush_s* mPatchBrushData (cm_landscape.h:100) that Raven points into a per-patch slice of CmLandScape's single shared Z_Malloc buffer (cm_terrain.cpp:213-215) becomes an OFFSET/LENGTH RANGE into the shared Vec-backed brush arena OWNED BY CmLandScape (§B5, no raw pointer) — Init's slice base (cm_terrain.cpp:524,588) + CreatePatchPlaneData's pointer-arithmetic carve (:302, :319-321) transcribe as range arithmetic. Not in any pub Seam signature (private helper of CmLandScape)" }
  - { path: crates/mp/engine/qcommon/src/cm_height_details.rs, crate: mp_engine_qcommon, mode: mp, class: CCMHeightDetails, summary: "CmHeightDetails (CCMHeightDetails, cm_landscape.h) — its OWN file per ruling 39d (§21 one class per file), beside cm_terrain.rs. A per-terxel height-detail struct read during CmLandScape construction/collision; not in any pub Seam signature (private detail struct of CmLandScape)" }
  - { path: crates/mp/engine/qcommon/src/terrain_handle.rs,   crate: mp_engine_qcommon, mode: mp, class: TerrainHandle,    summary: "TerrainHandle newtype over thandle_t (type thandle_t = c_int; the ABI id GetTerrainId()/mTerrainHandle returns, cm_landscape.h:139,220). Beside collision_world.rs per the settled rmg→qcommon dependency direction (the shared handle must live in qcommon or lower). Ruling 39d gives it its OWN roster row (was prose-only). register_terrain (Seam-B) constructs it; RmManager::set_landscape consumes it; land() returns Option<TerrainHandle>" }
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
  - { class: CRMManager,   kind: reshape, rule: "§B/RMG-D4b", note: "mMission:CRMMission* (RM_Manager.h:13) is §20-DROPPED — no Rust field: CRMMission has no Rust type (§20-dropped, RMG-D1) and mMission is dead under DEDICATED (ctor sets it NULL, RM_Manager.cpp:38; the only reassignment `new CRMMission` at RM_Manager.cpp:135 sits AFTER LoadMission's early-out RM_Manager.cpp:110-113 and is never reached; every read at :194/317/333/351/374/394 is inside the §20-dead methods). Corrects the earlier `owned field` framing — the field is never constructed on the live path (mirrors the mLandScape/mTerrain drops below). Cached CCMLandScape* mLandScape (RM_Manager.h:14) and CRandomTerrain* mTerrain (RM_Manager.h:15) likewise NOT stored — RmManager owns only the TerrainHandle; mTerrain resolves as always-None (RMG-D1, GetRandomTerrain()==0). TheRandomMissionManager → direct Engine.rmg field (ruling 12), no Option; lazy-init via a private initialized:bool flipped at the G_RMG_INIT arm (ctor RM_Manager.cpp:34-42 null/zero-inits the kept-field equivalents mLandScape/mTerrain/mMission=NULL — the only non-default member mCurPriority=1 at :39 is §20-dropped — so new()/Default and the lazy new collapse to one construction)" }
  - { class: CCMLandScape, kind: reshape, rule: "§B5/RMG-D4h", note: "byte* mHeightMap/mFlattenMap, CCMPatch* mPatches → owned Vec<u8>/Vec<CmPatch>; holdrand LCG stays an inline c_ulong field (Raven `unsigned long`, cm_landscape.h:160; seeded 0x89abcdef cm_terrain.cpp:122; get_rand_seed live-streamed). CCMPatch::owner:CCMLandScape* (cm_landscape.h:93) — a LIVE back-pointer on the patch-collision build (GetAdjacentBrushY, cm_terrain.cpp:246-256) — is DROPPED per §B3; the owning CmLandScape is threaded into the patch-build methods (§B4). PER-PATCH COLLISION-BRUSH DATA — SETTLED by RULING 46/RMG-D7 (former RMG-Q13, now closed): Raven's single shared Z_Malloc buffer CCMLandScape::mPatchBrushData sized size*GetBlockCount() (cm_terrain.cpp:213-215) becomes ONE shared Vec-backed brush arena OWNED BY CmLandScape (Raven's single-allocation topology, option (a)); each CmPatch stores an OFFSET/LENGTH RANGE into it (an offset+len, §B5 — NOT the raw struct cbrush_s* mPatchBrushData of cm_landscape.h:100). Raven's per-patch pointer-arithmetic carving transcribes as RANGE ARITHMETIC over the shared Vec: UpdatePatches slicing mPatchBrushData + size*(ix + iy*mBlockWidth) per patch (cm_terrain.cpp:925), CCMPatch::Init taking that slice base (the patchBrushData arg, cm_terrain.cpp:524,588), and CreatePatchPlaneData (:302) sub-dividing into cbrush_t/cbrushside_t/cplane_t regions (brush = mPatchBrushData; side = (cbrushside_t*)(mPatchBrushData + mNumBrushes); plane = (cplane_t*)(side + …), cm_terrain.cpp:319-321; InitPlane :223 fills each side/plane pair) all become index/range math. Read by the live patch-collision build incl. sibling GetAdjacentBrushX/Y (cm_terrain.cpp:246-256,282), for which §B5 (arena on sibling-walking consumers) picked the shared-arena shape. Rejected independent per-CmPatch Vec<cbrush_s>/Vec<cbrushside_s> (option (b)). With this the CmLandScape/CmPatch frozen field set is COMPLETE — no open field. Load-bearing for UpdatePatches, CmPatch::Init/InitPlane/CreatePatchPlaneData, terrain_patch_collide, GetAdjacentBrushX/Y (ruling 46 fixed the shared-arena shape: the faithful single-allocation topology (Raven's one Z_Malloc buffer) per fork-discovery ruling 4, reinforced by §B5 (arena on sibling-walking consumers))" }
  - { class: CCMLandScape, kind: reshape, rule: "§A1/RMG-D4a", note: "LIVE RUNTIME SURFACE — the constructed landscape has two live external readers, both owned by THIS subsystem (RMG-D4a: the cm C-track packets exclude CCMLandScape, so its methods port here, not in a cm packet nor another doc). (1) Snapshot/download read from the server (sv_client.cpp:779-806, via TheRandomMissionManager->GetLandScape()): GetHeightMap (cm_landscape.h:218 — §F.19 UB bytes, excluded from golden compare), GetFlattenMap (:219), GetRealArea (:211), get_rand_seed (:239). (2) Per-frame terrain collision from the cm-trace/cm-test C-track packets (cm_trace.cpp:283,760,789,997,1374 gated com_terrainPhysics->integer && cmg.landScape && CONTENTS_TERRAIN + cm_test.cpp:285-289 gated cmg.landScape && CONTENTS_TERRAIN only — NOT com_terrainPhysics, non-BSPC): PatchCollide (decl cm_landscape.h:175, def cm_terrain.cpp:600), WaterCollide (:178 / :836), GetWaterContents (:233), GetWaterSurfaceFlags (:234), GetWaterHeight (:232), GetBounds (:112/:199), GetPatchScalarSize (:207). Idiomatic Rust signatures for all of the above are FROZEN in Seam §C per ruling 28 (scope) + ruling 38 (receiver shape): read accessors stay CmLandScape &self methods GetHeightMap/GetFlattenMap→&[u8], GetRealArea→i32, get_rand_seed→c_ulong; the collision entry points are RULING-38 methods ON CollisionWorld (terrain_patch_collide is &mut self — checkcount writes into landscape-owned brushes are legal mutation — + &mut trace_t out-param; terrain_water_collide + terrain_bounds/water accessors are &self; all resolve self.land_scape internally). The earlier CmLandScape-method-taking-&mut-CollisionWorld shape is STRUCK (E0502, RMG-Q11 closed by ruling 38). The RmManager accessor is land()→Option<TerrainHandle> (callers resolve through CollisionWorld). Collision methods land with the early clipmap-trace waves 0–4 (wave split); read accessors with the wave-16 unit. SetShaders (LIVE via LoadTerrainDef cm_terrain.cpp:83), CalcRealCoords (LIVE via UpdatePatches :914) and GetPatch (LIVE via PatchCollide/WaterCollide :681,768,823 + the patch-collision build :256,282) are private-internal (§A1) helpers transcribed here — NOT pub seam; GetTerxelLocalCoords (sole callers commented-out :948-950 in the /*…*/ block :929-969) and CarveLine (sole caller the §20-dropped CarveBezierCurve :1303) have NO live caller — §20/dead. NONE of the five is a pub seam method (corrects the prior 'no live caller / renderer-only' claim for SetShaders/CalcRealCoords/GetPatch). LoadTerrainDef (cm_terrain.cpp:39-110) first GP2-parses the terrain-def file (Com_ParseTextFile opens ext_data/RMG/<terrainDef>.terrain, falls back to ext_data/arioche/…, non-fatal Com_Printf+return on a double miss, :48-56) via a function-local mp_engine_qcommon::gp2::GenericParser2 reading through the already-threaded host.fs_read_file (Com_ParseTextFile = FS_FOpenFileByMode+FS_Read+Parse, common.cpp:2179-2202) — INTRA-CRATE reuse of the ported §F GP2, NO new pub seam or crate edge (Seam definition), §C10 fallback control flow. Only on a successful parse do its altitudetexture/water cases read CM_GetShaderInfo/CCMShader (SetShaders' CCMShader* arg :80-83; the water contentFlags/surfaceFlags reads :98-103 populating mWaterContents/mWaterSurfaceFlags, i.e. water_contents()/water_surface_flags()) — the WIDER-CLIPMAP shader machinery (cm_local.h:77/303-304, cm_shader.cpp; NOT terrain-owned, cmg-resident STATE-D2, Non-goals), reached through the threaded CollisionWorld (§B4); this doc does not port cm_shader.cpp, and the Rust binding LoadTerrainDef calls is the SETTLED extern CollisionWorld::cm_get_shader_info(&mut self, name:&str)->Option<&CCMShader> (ruling 41/RMG-D5, decl cm_shader.cpp:498, &mut self because a miss registers shader text; owned by the cm C-track packet, landing with the cm waves, NOT ported here — former RMG-Q10, now closed)" }
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
  - { class: CRMManager, kind: drop, rule: "§20/RMG-D4c/ruling-38", note: "mCurObjective (RM_Manager.cpp:16) zero-init, never read/written; WriteAutomapSymbols (:424) commented-out; ProcessAutomapSymbols (:442) is a client-side static, dead under DEDICATED. SpawnMission (:391) unreachable under DEDICATED (LoadMission returns false) — RULING 38 DROPS it from Seam-A entirely (NOT a dead stub): the Wave-20 G_RMG_INIT arm collapses the provably-dead `if load_mission { spawn_mission }` to load_mission(…) per porting-rules §C10 + a §20 zero-execution note; there is no spawn_mission method/stub anywhere (GOAL-engine.md no-stub rule). Its CRMMission::Spawn-driving body is dropped with the class. All zero-caller notes" }
  - { class: CRMManager, kind: drop, rule: "§20/RMG-D1", note: "the twelve declared public/private CRMManager methods with NO live caller (of the 13 the review flagged; the 13th, GetLandScape, is LIVE — see tail) are §20 zero-caller drops (grep of `TheRandomMissionManager->` finds no invocation of any): SetCurPriority (RM_Manager.h:36), GetTerrain (:38), GetCurPriority (:41), Preview (:48), IsMissionComplete (:50), HasTimeExpired (:51), CompleteObjective (:52), CompleteMission (:53), FailedMission (:54), UpdateStatisticCvars (:23) have zero callers anywhere in codemp; GetMission (:40) and AddAutomapSymbol (:43) are called ONLY from the §20-dropped generation path (RM_Instance*/RM_Path.cpp; RM_Manager.cpp:400-410 SpawnMission body). Not ported — same §20 reasoning as the sibling drop above; only the SIX live methods new/SetLandScape/LoadMission/GetAutomapSymbolCount/GetAutomapSymbol/GetLandScape survive as the live Seam-A surface (SpawnMission is dropped, ruling 38 — no stub). The landscape accessor GetLandScape (:39) IS live (snapshot path) — Rust form land()→Option<TerrainHandle> (ruling 28/RMG-D1), not a drop" }
  - { class: CTerrainMap, kind: drop, rule: "§20/RMG-D4c", note: "whole automap-image builder dead under DEDICATED: its only ctor CM_TM_Create is #ifndef DEDICATED (RM_Mission.cpp:1503-1504); Upload/SaveImageToDisk named by ruling 17. Recorded, not ported (returns to scope if the renderer is un-deferred, DEC-01)" }
  - { class: CRMLandScape, kind: drop, rule: "§20/RMG-D4c", note: "RM_Terrain.cpp client-model chain (CRMLandScape/CCGHeightDetails/CRandomModel/CCGPatch, RM_CreateRandomModels, SpawnPatchModelsWrapper) — graph-confirmed zero engine callers under DEDICATED (ruling 17); reached only from the client (RM_CreateRandomModels ← cl_cgame.cpp:1707). Not ported" }
```

## Addendum (ruling 49, 2026-07-09 — amendment signed off in the ruling)

- **`CDraw32` (cm_draw.cpp, 22 methods) is §20-dropped**, transitively on the
  already-dropped `CTerrainMap` (its sole caller; header-only in
  WinDed.vcproj). The only instantiation chain (`CM_TM_Create`,
  RM_Mission.cpp:1501-1504) is `#ifndef DEDICATED` + `!com_dedicated->integer`
  double-guarded. Zero-caller note per porting-rules §20; no roster rows.
  Source: `docs/handoffs/engine-fork-discovery.md` ruling 49.
