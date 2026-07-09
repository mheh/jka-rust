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
  Release link set, done at wave 16.
- `docs/plans/2026-07-08-mp-engine-build-out.md` — §"RMG (113 fns, wave 16)",
  the cross-subsystem matrix (6 server→RMG edges), §3c (OpenJK dropped RMG).
- `docs/handoffs/engine-fork-discovery.md` — settled forks and the second-session
  §F rulings this doc consumes: **fork-2** (global state placement, `:21-29`),
  **fork-3** (function-scope statics, three-kind rule, `:31-36`), **fork-5**
  (internal dispatch tables → plain fn-item structs / const slices, no fn-ID
  enums, `:46-53`), **ruling 7** (the blessed 5-doc §F list — RMG is one,
  `:61-68`), **ruling 11** (the one `EngineHost` services trait + view-struct
  impl, `:121-126`), **ruling 12** (the five §F states are plain
  Default-initialized direct `Engine` fields — `rmg` among them, `:127-131`),
  **ruling 16** (the qcommon terrain twins fold into *this* doc, `:143-146`), and
  **ruling 17** (the four §20 dead-surface drops, `:147-152`).
- `docs/architecture/state-ownership.md` — the STATE-* ledger: STATE-D5 (the one
  `Engine` island lives in `mp_engine_core`, `crates/mp/engine/core/src/engine.rs:20`),
  STATE-D2 (`Engine.cm: mp_engine_qcommon::CollisionWorld` owns Raven's `cmg`
  clipmap — `state-ownership.md:418`, `collision_world.rs:10`). STATE-Q2's
  **placement half** is now resolved by ruling 12 (direct `Engine.rmg` field);
  its service half is ruling 11 (`EngineHost`).
- `docs/decisions.md` — DEC-01 (renderer deferred), DEC-04 (strict per-mode),
  DEC-09 (engine verification: TU harnesses + live peers).
- GP2 is the §F exemplar and a *live dependency* of this subsystem
  (`crates/mp/engine/qcommon/src/gp2/`, `tools/gp2-oracle/`): every RMG parse
  path runs through `CGenericParser2`/`CGPGroup`, already ported.

## Scope & non-goals

**In scope — the RMG class tree** (`oracle/codemp/RMG/`): `CRMManager`,
`CRMMission`, the closed `CRMInstance` hierarchy (`CRMBSPInstance`,
`CRMGroupInstance`, `CRMRandomInstance`, `CRMVoidInstance`),
`CRMAreaManager`/`CRMArea`, `CRMPathManager` (with `CRMNode`/`CRMLoc`/`CRMCell`),
`CRMInstanceFile`, `CRMObjective`.

**In scope — the qcommon terrain twins** (`oracle/codemp/qcommon/`), folded here
by **ruling 16** (RMG-D2a): `CCMLandScape` + `CCMPatch`/`CCMHeightDetails`/`CArea`
(`cm_terrain.cpp`, `cm_landscape.h`), `CRandomTerrain` + `CPathInfo`
(`cm_randomterrain.cpp/.h`), and `CTerrainMap` (`cm_terrainmap.cpp/.h`). The
`cm` C-track packets exclude these classes (ruling 16). `CTerrainMap` is folded
here for *ownership*, but it is **DEDICATED-dead** (see Non-goals) and is recorded
per §20, not ported.

**Non-goals** (punted / dropped, each with its owner):
- **SP RMG** (`oracle/code/RMG/`, a near-duplicate tree). This is the dedicated
  MP engine (`docs/GOAL-engine.md` "Explicitly after this process"); per-mode
  discipline (DEC-04) forbids unifying it. SP engine is a later campaign.
- **The wider clipmap** (`cm_load.cpp`, `cm_patch.cpp`, `cm_trace.cpp`). Only the
  terrain-owned members of `CCMLandScape` are here; `CM_RegisterTerrain`'s
  clipmap wiring is a C-track qcommon packet.
- **Renderer-side terrain draw** (`tr_terrain*`). Deferred with the renderer
  (DEC-01); not in the dedicated link set.
- **`CRMNPCInstance`** — a fifth instance kind, dead in Raven (its `new` is
  commented out at the factory, `oracle/codemp/RMG/RM_InstanceFile.cpp:162-166`).
  Dropped per §20.
- **The `RM_Terrain.cpp` client-model chain** — `CRMLandScape`,
  `CCGHeightDetails`, `CRandomModel`, `CCGPatch`, `RM_CreateRandomModels`,
  `SpawnPatchModelsWrapper`. §20-dropped by **ruling 17** (graph-confirmed zero
  engine callers under DEDICATED). Recorded as a divergence, not ported.
- **`CTerrainMap`'s automap-image builder** — `CM_TM_Create`/`Add*`/`Upload`/
  `SaveImageToDisk`. Its *only* construction site is `#ifndef DEDICATED`
  (`RM_Mission.cpp:1503-1504`), so the whole class is dead under DEDICATED;
  `CTerrainMap::Upload`/`SaveImageToDisk` are named explicitly by **ruling 17**.
  §20-dropped, recorded as a divergence.

## Raven ground truth

### Data flow (server boot → terrain → mission)

1. The game module vmcalls `trap_CM_RegisterTerrain(config)`
   (`oracle/codemp/game/g_syscalls.c:1473-1476`,
   `oracle/codemp/game/g_misc.c:582`). The syscall switch case
   `G_CM_REGISTER_TERRAIN` calls `CM_RegisterTerrain((char*)VMA(1), true)` and
   returns `->GetTerrainId()` (`oracle/codemp/server/sv_game.cpp:1640-1641`).
   `CM_RegisterTerrain` (`oracle/codemp/qcommon/cm_load.cpp:1036`) constructs the
   `CCMLandScape` and, for a random config, `CreateRandomTerrain(...)`
   (`oracle/codemp/qcommon/cm_landscape.h:260`) building the `CRandomTerrain`.
2. `CCMLandScape::CCMLandScape` seeds `holdrand = 0x89abcdef`
   (`oracle/codemp/qcommon/cm_terrain.cpp:116-122`), loads the terrain def, and
   builds the patch/height arrays.
3. The game vmcalls `trap_RMG_Init(terrainID)`
   (`oracle/codemp/game/g_syscalls.c:1478-1481`,
   `oracle/codemp/game/g_misc.c:608`). Case `G_RMG_INIT`
   (`oracle/codemp/server/sv_game.cpp:1624-1638`), gated on `com_RMG->integer`:
   lazily `new CRMManager`, `SetLandScape(cmg.landScape)`, then
   `LoadMission(qtrue)` → on success `SpawnMission(qtrue)`.
4. `CRMManager::LoadMission` (`oracle/codemp/RMG/RM_Manager.cpp:96`) builds a
   `CRMMission(mTerrain)` and `Load()`s the `.mission`/`.instances`/difficulty
   GP2 files (`CRMMission::Load`, `RM_Mission.cpp:1362`). **Generation runs inside
   `Load`, not `Spawn`**: `ParsePaths`/`ParseRivers` drive
   `CRMPathManager::GeneratePaths`/`GenerateRivers` (`RM_Mission.cpp:321,372`), and
   `Load` then carves the heightmap via `CRandomTerrain::Generate(mSymmetric)`
   (`RM_Mission.cpp:1417`) — all off the per-landscape RNG already seeded at
   `CreateRandomTerrain` time (the config `"seed"` info key → `rand_seed`,
   `cm_terrain.cpp:1688-1700`). `SpawnMission` (`RM_Manager.cpp:391`) then drives
   `CRMMission::Spawn` (`RM_Mission.cpp:1438`), which **on the server** (`IsServer`
   true, as `G_RMG_INIT` calls it) `PreSpawn`s every instance, `Smooth`s the
   landscape, and `PlaceBridges` (`RM_Mission.cpp:1446-1467`), placing the
   instances (BSP models, groups, void holes) onto the terrain. The
   `rand_seed(clc.rmgSeed)` at `RM_Mission.cpp:1473` is the **client**
   reconstruction branch (`#ifndef DEDICATED else`), *not* the server path — the
   dedicated engine never compiles it.
5. Snapshot/download path: `SV_ClientEnterWorld`/`SV_SendClientGameState`
   read `TheRandomMissionManager->GetLandScape()->GetHeightMap()` /
   `GetFlattenMap()` / `get_rand_seed()` and the automap symbols to stream the
   generated terrain to clients (`oracle/codemp/server/sv_client.cpp:668-806`);
   `sv_snapshot.cpp:394` gates on `com_RMG`.

### Frame role

RMG is **generation-time, not per-frame**: the whole tree runs once at
`SV_SpawnServer` time (through `G_RMG_INIT`) and then only the produced
`CCMLandScape` collision/height data is touched per frame (by `cm_trace`, out of
scope). Automap symbols are read once at client connect.

### Class tree (closed hierarchy)

`CRMInstance` is an abstract base with four concrete subclasses and pure-ish
virtuals `PreSpawn`/`Spawn`/`PostSpawn`/`SetArea`/`SetFilter`/`SetMirror`/
`Preview`/`GetPreviewColor`/`GetSpacingRadius`/… (`oracle/codemp/RMG/
RM_Instance.h:25-117`). The factory `CRMInstanceFile::CreateInstance`
string-dispatches `"bsp"|"group"|"random"|"void"` to
`new CRM{BSP,Group,Random,Void}Instance` (`RM_InstanceFile.cpp:138-193`); no
subclass is created anywhere else — the hierarchy is **closed**.
`CRMRandomInstance` holds a `CRMInstance* mInstance` it forwards most virtuals to
(`RM_Instance_Random.h:15,22-29`); `CRMGroupInstance` owns a
`rmInstanceList_t mInstances` (`RM_Instance_Group.h:13`).

### Globals (see State ownership for owners)

- `CRMManager* TheRandomMissionManager` — the one live singleton
  (`oracle/codemp/RMG/RM_Manager.cpp:23`; extern `RM_Manager.h:60`).
- `CRMObjective* CRMManager::mCurObjective` — static member, zero-init only
  (`RM_Manager.cpp:16`) and **never read or written** anywhere in codemp (grep:
  decl `RM_Manager.h:57` + init `:16` are its only refs). §20-dropped (ruling 17).
- `static CTerrainMap* TerrainMap` — file-scope in the terrain-map builder
  (`oracle/codemp/qcommon/cm_terrainmap.cpp:14`); its only writer `CM_TM_Create`
  is `#ifndef DEDICATED` (`RM_Mission.cpp:1503-1504`) — dead under DEDICATED,
  §20-dropped (ruling 17).
- `static float noiseTable[256]` / `static int noisePerm[256]` — zero-initialized
  file-scope statics that are **never written**: their only writer `CM_NoiseInit`
  is inside `#if 0` (`oracle/codemp/qcommon/cm_randomterrain.cpp:17-28`) and its
  sole call is inside a `/* */` comment block (`:785-795`). They are still live-
  **read** during `CRandomTerrain::Generate` (`CM_NoiseGet4f` at `:806` →
  `GetNoiseValue`/`INDEX` → `noiseTable`/`noisePerm`, `:30-40`), where the all-zero
  tables yield a deterministic **0** noise contribution. §20-dropped, the 0
  encoded directly (ruling 17 / RMG-D2c).
- `static TCharacterPiece Consonants[]…` — const seed-name tables
  (`cm_randomterrain.cpp:847+`).
- `static int CRMPathManager::neighbor_x/y[DIR_MAX]` — const step tables
  (`oracle/codemp/RMG/RM_Path.h:172-173`).
- `static int instanceID` in `CreateInstance` — assigned-never-read scratch
  (`RM_InstanceFile.cpp:140`).
- The free-function `flrand`/`irand` LCG over the file-scope global
  `holdrand = 0x89abcdef` (`oracle/codemp/game/q_math.c:1432,1441-1470`), seeded
  by `Rand_Init` (`:1434`; called nondeterministically from
  `common.cpp:1248`). This is the RNG `RMG_CreateSeed` draws
  (`cm_randomterrain.cpp:1016,1018,…`) — distinct from the per-landscape LCG below.
- cvars `com_RMG` (`oracle/codemp/qcommon/common.cpp:72,1335`),
  `com_terrainPhysics` (`oracle/codemp/qcommon/cm_landscape.h:267`).
- Per-instance (not global) RNG state: `CCMLandScape::holdrand`
  (member decl `cm_landscape.h:160`, seeded `cm_terrain.cpp:122`); its
  `flrand`/`irand`/`rand_seed` are defined **inline** on the class
  (`cm_terrain.cpp:1548-1580`) — see RNG threading.

## State ownership

Per **ruling 12** (the five §F states are plain Default-initialized direct
`Engine` fields — no Option/Box/nesting; lazy-init timing modeled with Raven's
own initialized flags) and §B. Ruling 12 supersedes fork-2's "sub-struct"
framing for the §F subcrates.

| Raven global | oracle cite | Rust owner (crate::Type.field) | constructed by | threaded via |
| --- | --- | --- | --- | --- |
| `TheRandomMissionManager` | `RM_Manager.cpp:23` | `mp_engine_core::Engine.rmg: RmManager` (plain direct field, ruling 12; STATE-D5 places the island in `mp_engine_core`, `engine.rs:20`). Raven lazily `new`s it under `com_RMG`; modeled with the private `RmManager.initialized: bool` field (Default `false`, flipped at the `G_RMG_INIT` arm — see Seam-A owned-state note) mirroring Raven's null check, not `Option` (ruling 12) | `G_RMG_INIT` case (lazy) — `sv_game.cpp:1627-1629` | `&mut self` + `&mut impl EngineHost` from the syscall switch inward |
| `CRMManager::mCurObjective` | `RM_Manager.cpp:16` | **dropped** — §20 dead surface (ruling 17): zero-init only, never read or written in codemp | — | — |
| `CCMLandScape*` (`cmg.landScape`) | `cm_landscape.h:135` (class), `cm_local.h:155` (`cmg.landScape`), `sv_game.cpp:1631` | `mp_engine_qcommon::CollisionWorld.land_scape: Option<CmLandScape>` (a new field on the **existing** STATE-D2 `cmg` owner `CollisionWorld` — `collision_world.rs:10`; there is no `ClipMap` type). `Option` here is Raven-faithful: `cmg.landScape` is a nullable pointer set only on a random map | `CM_RegisterTerrain` — `cm_load.cpp:1036,1055` | `TerrainHandle` (wrapping `thandle_t`) across the seam; borrow inward |
| `CRandomTerrain*` | `cm_randomterrain.h:52`, `RM_Manager.h:15` | owned inside `CmLandScape.random_terrain: Option<RandomTerrain>` | `CreateRandomTerrain` — `cm_landscape.h:260` | borrow from its `CmLandScape` |
| `static CTerrainMap* TerrainMap` | `cm_terrainmap.cpp:14` | **dropped** — §20 dead surface (ruling 17): only writer `CM_TM_Create` is `#ifndef DEDICATED` (`RM_Mission.cpp:1503-1504`) | — | — |
| `noiseTable` / `noisePerm` | `cm_randomterrain.cpp:14-15` | **dropped** — §20 dead surface (ruling 17): never written (`CM_NoiseInit` is `#if 0`, `:17-28`; its call is commented, `:785-795`). The live read at `:806` is reproduced by encoding the deterministic **0** contribution directly; no field, no RNG draw | — | — |
| `Consonants[]` etc. seed-piece tables | `cm_randomterrain.cpp:847+` | `const` slices (fork-3 kind-1) | — | module `const` |
| `CRMPathManager::neighbor_x/y` | `RM_Path.h:172-173` | `const NEIGHBOR_X/Y: [i32; DIR_MAX]` (fork-3 kind-1) | — | module `const` |
| `CreateInstance::instanceID` | `RM_InstanceFile.cpp:140` | dropped — assigned-never-read (fork-3, §20) | — | — |
| free `flrand`/`irand` global `holdrand` | `q_math.c:1432` | `mp_engine_core::Engine.common.rng: mp_qshared::QRand` — the engine's **own** instance of Raven's q_math LCG (RMG-D1 part 3, ruling 21). `mp_qshared` gains a `QRand` type (the stateful LCG the game tier already models as `bg_channel::rng::Rng`, `crates/mp/game/src/bg_channel/rng.rs`); the engine holds a distinct instance on `Common` (`engine.rs:22`, `common/common.rs:20`), exposed as `EngineHost::flrand`/`irand`. `RMG_CreateSeed` (golden-only, zero live callers) draws through those services | `Rand_Init` (`q_math.c:1434`) | `&mut impl EngineHost` (`flrand`/`irand`) |
| `CRMArea*` — `mAreas` arena + `CRMInstance::mArea` | `RM_Area.h:74,80`; `RM_Instance.h:33` | `AreaId` (a `u32` index newtype, rendered like `EntityId`) into `mp_engine_rmg::CrmAreaManager.areas: Vec<CmArea>` — §B5 arena (RMG-D1 part 4) | `CRMAreaManager::CreateArea` — `RM_Area.h:91` | `AreaId` threaded through `SetArea`/`GetArea` (`RM_Instance.h:72,107`) and the `RmInstance` variants |
| `com_RMG`, `com_terrainPhysics` | `common.cpp:72`, `cm_landscape.h:267` | `EngineCvars` handles (fork-2) | `Cvar_Get` at init | read via cvar accessor |
| `CCMLandScape::holdrand` | `cm_landscape.h:160` | `CmLandScape.holdrand: c_ulong` — an **inline** per-instance LCG field with `flrand`/`irand`/`rand_seed`/`get_rand_seed` methods transcribed verbatim from `cm_terrain.cpp:1548-1580`; **not** an external `Rng` type, so no cross-tier reachability problem | `CCMLandScape` ctor seeds `0x89abcdef` (`cm_terrain.cpp:122`) | field; see RNG threading |

## Seam definition

RMG crosses **two** boundaries; nothing here crosses the *module* ABI (no
`#[repr(C)]` layout constraint — §F), so all types below are idiomatic.

**The host seam (ruling 11).** Every §F engine service Raven reached through a
file-scope global or `gi.`/`Com_` call — FS reads of the `.mission`/`.terrain`/
`.landscape` GP2 files, `Com_Printf`/`Com_Error`, cvar reads — is threaded as the
one `EngineHost` services trait defined in the Stage-0 interface crate (ruling
11: trace, FS, print/error, VM_Call, shared memory — plus the `flrand`/`irand`
RNG services added by RMG-D1 part 3, backed by `Engine.common.rng`). `Engine`
implements it via a
split-borrow view struct; the referee injects a deterministic impl (DEC-09). §F
methods that touch a service take `&mut impl EngineHost`; the `CollisionWorld`
state is *not* a service and stays a separate threaded param (§B4).

**Handle types (§B5, layout-free).** The seam names two handle types (and rules
out a third), defined here so porters do not invent them:

- `TerrainHandle` — a newtype over the rosetta's `thandle_t`
  (`type thandle_t = c_int`, `crates/native/types/src/lib.rs:65`); it is the
  ABI-crossing id the syscall returns (`GetTerrainId()` returns `thandle_t`,
  `cm_landscape.h:220`; `mTerrainHandle`, `:139`). **Defined in
  `mp_engine_qcommon`** (e.g. a small `terrain_handle.rs` beside
  `collision_world.rs`): `register_terrain` there constructs it and
  `RmManager::set_landscape` in `mp_engine_rmg` consumes it, and the added crate
  edge runs `rmg → qcommon` only (RMG-D2a, "Crate dependencies" under Files
  roster), never the reverse — so the shared handle must live in `qcommon` (or
  lower). This is the mechanical consequence of the settled dependency direction,
  not a placement choice (mirrors `AreaId` living in `mp_engine_rmg` where
  `CrmAreaManager` owns the arena).
- **No random-terrain handle** (RMG-D1 part 2, RMG-Q5 SETTLED). Raven
  `CreateRandomTerrain` returns a bare `CRandomTerrain*` (`cm_landscape.h:260`)
  that its only caller assigns straight to `mRandomTerrain`
  (`cm_terrain.cpp:178`) — a single owned field, not an arena. `CRandomTerrain`
  is therefore modeled as the one `CmLandScape.random_terrain: Option<RandomTerrain>`
  field (`cm_landscape.h:153`); methods borrow it directly and no
  `RandomTerrainHandle` newtype exists. It is qcommon-internal and never crosses
  the module ABI — only `thandle_t` (`TerrainHandle`) does.
- `AreaId` — an **internal** §B5 index newtype into `CRMAreaManager`'s owned
  area arena (Raven `CRMAreaManager` owns `rmAreaVector_t mAreas`,
  `oracle/codemp/RMG/RM_Area.h:74,80`, and hands out raw `CRMArea*` via
  `CreateArea`/`EnumArea`, `:91,93`). `CRMInstance::mArea` (`RM_Instance.h:33`)
  is stored long-term by `SetArea` (`mArea = area`, `RM_Instance.h:72`; call
  sites `RM_Mission.cpp:435,467,853,942`) and dereffed unconditionally by
  `GetArea` (`return *mArea`, `RM_Instance.h:107`), with consumers walking
  `mArea->…` (`GetOrigin`/`GetAngle`, `RM_Instance.h:108-110`). That is §B5's
  "arena + id + copyable borrow wrapper when consumers walk parent/sibling
  pointers" case verbatim (porting-rules §17): `mAreas` → an owned `Vec<CmArea>`
  in `CrmAreaManager`, `mArea` → an `AreaId` index, `GetArea` → an arena lookup —
  rendered the way the codebase renders `EntityId`. It never crosses the module
  ABI. This is **determined by §B5**, not a representation choice: unlike
  `CRandomTerrain` (a single owned `Option<RandomTerrain>`, no handle — RMG-D1
  part 2), `mAreas` is a genuine multi-element shared arena, so §B5's arena+id
  form applies directly. Per RMG-D1 part 4, `AreaId` also appears as a row in the
  State-ownership table and is threaded through `SetArea`/`GetArea` and the
  `RmInstance` variants.

The clipmap the terrain hangs off is the existing STATE-D2 `CollisionWorld`
(`Engine.cm`, `collision_world.rs:10`) — there is no separate `ClipMap` type.

### A. Server → RMG (the 6 call edges; plan matrix `server → RMG = 6`)

The game module reaches RMG only through three vmcalls hitting the syscall
switch (`oracle/codemp/game/g_public.h:571-573`,
`g_syscalls.c:1468-1481`): `G_SET_ACTIVE_SUBBSP`, `G_CM_REGISTER_TERRAIN`,
`G_RMG_INIT`. Inside the switch the server calls these `RmManager` methods
(the 6 edges: ctor, `SetLandScape`, `LoadMission`, `SpawnMission`,
`GetAutomapSymbolCount`, `GetAutomapSymbol`; `sv_game.cpp:1627-1634`,
`sv_client.cpp:670-677`). Frozen pub API on `mp_engine_rmg`:

```rust
impl RmManager {
    /// `CRMManager::CRMManager` — RM_Manager.cpp:34
    pub fn new() -> Self;
    /// `CRMManager::SetLandScape` — RM_Manager.cpp:79
    pub fn set_landscape(&mut self, land: TerrainHandle);
    /// `CRMManager::LoadMission` — RM_Manager.cpp:96
    pub fn load_mission(&mut self, cm: &mut CollisionWorld, host: &mut impl EngineHost, is_server: bool) -> bool;
    /// `CRMManager::SpawnMission` — RM_Manager.cpp:391
    pub fn spawn_mission(&mut self, cm: &mut CollisionWorld, host: &mut impl EngineHost, is_server: bool) -> bool;
    /// `CRMManager::GetAutomapSymbolCount` — RM_Manager.cpp:413
    pub fn automap_symbol_count(&self) -> i32;
    /// `CRMManager::GetAutomapSymbol` — RM_Manager.cpp:418
    pub fn automap_symbol(&self, index: i32) -> Option<&RmAutomapSymbol>;
}
```

**Owned-state field — the lazy-init flag (rendering of ruling 12, not a new
decision).** Beyond the pub API above, `RmManager` carries one private field
`initialized: bool` (Default `false`), the concrete rendering of ruling 12's
"Raven's own initialized flag" for `TheRandomMissionManager`. Raven's flag *is*
the `!TheRandomMissionManager` null check at the `G_RMG_INIT` arm
(`sv_game.cpp:1627-1629`); the field flips to `true` **at that syscall arm**,
where Raven `new`s — **not** inside any `RmManager` method (`set_landscape`/etc.
do not touch it). Because `CRMManager::CRMManager` only zeroes members
(`RM_Manager.cpp:34-42`: `mLandScape/mTerrain/mMission = NULL`, `mCurPriority=1`,
`mUseTimeLimit=false`, `mAutomapSymbolCount=0`), it is Default-equivalent: the
Engine-construction `RmManager::default()` and Raven's lazy `new CRMManager`
collapse to **one** Default construction, and the lazy step is *only* the flag
flip — there is no distinct re-construction. Frozen: porters add the field and
flip it at the Wave-20 syscall arm; they do not change it.

**Seam deviation — the added `cm: &mut CollisionWorld` parameter (not a design
change).** Raven's `CRMManager::LoadMission`/`SpawnMission` take only
`qboolean IsServer` (`oracle/codemp/RMG/RM_Manager.cpp:96,391`) and reach the
landscape through the `cmg.landScape` file global. Per §B (no hidden globals) and
the State-ownership table, `RmManager` owns **only** a `TerrainHandle`; the
`CCMLandScape` data lives in `CollisionWorld` (STATE-D2, `Engine.cm` —
`collision_world.rs:10`). So both methods take the owning `CollisionWorld`
explicitly to resolve that handle — the state-threading form (§B4) of Raven's
global reach, not added behavior. (This is why `mp_engine_rmg` needs the
`mp_engine_qcommon` edge to name `CollisionWorld` — see "Crate dependencies"
under Files roster.)

`rmAutomapSymbol_t` is an existing ABI type (`oracle/codemp/client/client.h:149`,
`MAX_AUTOMAP_SYMBOLS = 512` `:151`); the rosetta ported it in crate
**`mp_engine_client`** (`crates/mp/engine/client/src/client/rm_automap_symbol_t.rs:9`).
Per **RMG-D1 part 1** (RMG-Q4 SETTLED) it **relocates to `mp_qshared`** — the crate
`mp_engine_rmg` already depends on — so `RmManager::automap_symbol` returns
`Option<&RmAutomapSymbol>` (the relocated `mp_qshared` type) directly, with **no**
`rmg → mp_engine_client` edge (client → rmg is the allowed graph direction, never
the reverse).
The live automap serializer is the server-side `SV_WriteRMGAutomapSymbols`
(`oracle/codemp/server/sv_client.cpp:670`), which walks the count/get pair (edges
#5/#6); `CRMManager::WriteAutomapSymbols` (`RM_Manager.cpp:424`) is commented-out
dead code and is dropped per §20 (not part of the seam).
`CRMManager::ProcessAutomapSymbols` (`RM_Manager.cpp:442`) is a `static`
client-side reader; dead under DEDICATED, dropped per §20.

### B. RMG → qcommon terrain (the free-function entry points)

`cm_landscape.h:245-265` and `cm_terrainmap.h:69-80` declare the C entry points
the server/clipmap call. The frozen `mp_engine_qcommon` surface (faithful
signatures, `thandle_t` handles per §B5, host threaded per ruling 11):

```rust
/// `CM_RegisterTerrain` — cm_load.cpp:1036
pub fn register_terrain(cm: &mut CollisionWorld, host: &mut impl EngineHost, config: &str, server: bool) -> TerrainHandle;
/// `CreateRandomTerrain` — cm_terrain.cpp:1688 (parses the `"seed"` info key from
/// `config` and calls `landscape->rand_seed(seed)` — cm_terrain.cpp:1688-1700).
/// Per RMG-D1 part 2 there is NO handle: it builds the `RandomTerrain` and stores
/// it into `land.random_terrain`. It is qcommon-internal — its only caller is
/// `CCMLandScape`'s terrain build (`cm_terrain.cpp:178`), never the module ABI.
pub fn create_random_terrain(land: &mut CmLandScape, host: &mut impl EngineHost, config: &str, heightmap: &mut [u8], width: i32, height: i32);
/// `RMG_CreateSeed` — cm_randomterrain.cpp:1008 (draws the engine's q_math LCG via
/// `EngineHost::flrand`/`irand` — RMG-D1 part 3; zero live callers in codemp, kept
/// as a golden-only helper the harness pins against `Engine.common.rng`)
pub fn rmg_create_seed(host: &mut impl EngineHost) -> (String, u32);
```

**`CM_TerrainPatchIterate` is §20-dropped, not a seam entry (answered from the
caller census, DEC-01 + RMG-D2c).** The free function (`cm_terrain.cpp:1628`) and
the `CCMLandScape::TerrainPatchIterate` method it forwards to
(`cm_terrain.cpp:997`) have exactly two callers in codemp
(grep `CM_TerrainPatchIterate`): the renderer (`tr_terrain.cpp:923`, DEC-01
deferred) and `RM_CreateRandomModels`/`SpawnPatchModelsWrapper`
(`RM_Terrain.cpp:493`) — the latter is the `RM_Terrain.cpp` client-model chain
**already §20-dropped by RMG-D2c(c)**. Both callers are dead on the dedicated-engine
path, so patch-iterate has zero live callers here (the same classification the
doc applies to `mRefCount`, whose only reader is renderer-only). It is dropped
with a zero-caller note, not ported — which also moots the borrow question its
`const CCMLandScape*`-callback-mutating-`CCMPatch*` shape would otherwise raise
(shallow C++ const-through-pointer has no faithful `&CmLandScape` rendering; no
§17 signature choice is needed for dropped surface). Recorded in `divergences`.

**The twelve `cm_landscape.h:245-265` `CM_*` FREE-FUNCTION WRAPPERS are §20
zero-caller drops — but that drop does NOT extend to the same-named
`CCMLandScape` methods; three of those methods are live and MUST be ported
(answered by a per-file caller census, the same check the doc already runs for
`CM_TerrainPatchIterate`'s method at `:997`).** Beyond the three frozen entries
above and the `CM_TerrainPatchIterate` drop, the remaining C wrappers declared in
that range — `CM_GetWorldHeight` (`cm_landscape.h:247`), `CM_FlattenArea`
(`:248`), `CM_CarveBezierCurve` (`:249`), `CM_SaveArea` (`:250`),
`CM_FractionBelowLevel` (`:251`), `CM_AreaCollision` (`:252`), and the
`CArea`-cursor family `CM_GetFirstArea`/`CM_GetFirstObjectiveArea`/
`CM_GetPlayerArea`/`CM_GetNextArea`/`CM_GetNextObjectiveArea` (`:253-257`) — each
grep-resolves to exactly its `cm_landscape.h` declaration plus its
`cm_terrain.cpp:1633-1685` **wrapper** definition, with no call site. So **the
wrapper free functions have zero callers and are all §20-dropped**.
`SV_LoadMissionDef` (`cm_landscape.h:262`) is **declared but never defined**
anywhere in the tree and never called — also dropped.

**Three of the `CCMLandScape` methods the dropped wrappers forward to are LIVE
and ported; nine are dead and dropped with the wrappers.** Grepping the *methods*
(not the `CM_*` wrappers): `CCMLandScape::FlattenArea` (`cm_terrain.cpp:1312`) is
called live from the doc's own central generation dataflow —
`CRMPathManager::GeneratePaths`/`GenerateRivers` path/river carving
(`RM_Path.cpp:346,583`) and `CRMInstance` placement (`RM_Instance.cpp:77`), plus
internally (`:1224`); `CCMLandScape::SaveArea` (`cm_terrain.cpp:1128`) from
`CRMMission::Spawn`'s objective-area save (`RM_Mission.cpp:1573,1581`) and
internally (`:1321`); `CCMLandScape::GetWorldHeight` (`cm_terrain.cpp:1011`) from
`CRMBSPInstance` PreSpawn/Spawn placement (`RM_Instance_BSP.cpp:122,160`) and
`CRMMission` node placement (`RM_Mission.cpp:403,412`). **All three are ported**
as methods on `CmLandScape`, even though their `CM_*` free-function wrappers are
dropped — dropping them would silently break heightmap-mutation and BSP-placement
logic. The remaining nine methods are correctly dead:
`AreaCollision`/`GetFirstArea`/`GetNextArea` (`cm_terrain.cpp:1488,1412,1462`) are
called only from the already-§20-dropped `RM_Terrain.cpp` client-model chain
(`RM_Terrain.cpp:417,251,257,273`, RMG-D2c(c)) and internally within
`AreaCollision`; `FractionBelowLevel`/`CarveBezierCurve`/`GetFirstObjectiveArea`/
`GetPlayerArea`/`GetNextObjectiveArea`
(`cm_terrain.cpp:1379,1245,1422,1442,1472`) have zero callers anywhere. Those nine
are §20-dropped with the wrappers. Recorded in `divergences`. (The `CArea` these
methods take is the qcommon class — its Rust name is RMG-Q7, Open questions.)

`CRandomTerrain` forwards `flrand`/`irand`/`rand_seed`/`get_rand_seed` to its
`CCMLandScape` (`cm_randomterrain.h:70-73`) — model that as method delegation on
the owned `CmLandScape` (whose LCG is the inline per-instance field, State
table), not a duplicated LCG.

**Repeat-registration / refcount (answered from oracle + DEC-01).** Raven's
`CM_RegisterTerrain` refcounts: on a second call with `cmg.landScape` already
set it `IncreaseRefCount()`s and returns the existing landscape
(`cm_load.cpp:1040-1044`; `mRefCount=1` at ctor, `cm_terrain.cpp:130`). The
**only** consumer of that count is `CM_ShutdownTerrain`, which frees the
landscape only when the count hits 0 (`cm_load.cpp:1073-1077`) — and its **only**
caller is the renderer (`tr_terrain.cpp:1050`), deferred with DEC-01. On the
dedicated-server path the landscape is instead freed **unconditionally** at map
teardown (`delete cmg.landScape`, `cm_load.cpp:800-809`), never through the
count. So `mRefCount` has no live reader on this subsystem's paths: `mRefCount`
is dropped as renderer-only dead surface (§20/DEC-01), and `register_terrain`
reproduces only the observable seam behavior — **return the existing
`TerrainHandle` on repeat registration** (a get-or-create on the owned
`Option<CmLandScape>`; matches `cm_load.cpp:1040-1044`). This is the §20
dead-surface classification the caller census forces; if the renderer is ever
un-deferred (DEC-01), the field returns then.

## Decisions

**RMG-D1 — Ruling 21 closes the five remaining holes.** Per **ruling 21** (user,
2026-07-09, `engine-fork-discovery.md`), five parts:

1. **`rmAutomapSymbol_t` relocates to `mp_qshared` — no `rmg → client` edge**
   (SETTLES RMG-Q4). The rosetta ported it in `mp_engine_client`
   (`crates/mp/engine/client/src/client/rm_automap_symbol_t.rs:9`,
   `oracle/codemp/client/client.h:149`); it moves to `mp_qshared`, on which
   `mp_engine_rmg` already depends, so `RmManager::automap_symbol` names it
   directly. Rejected an `rmg → mp_engine_client` dependency: client → rmg is the
   allowed graph direction, never the reverse.
2. **No `RandomTerrainHandle` newtype** (SETTLES RMG-Q5). `CRandomTerrain` is a
   single owned `CmLandScape.random_terrain: Option<RandomTerrain>`
   (`cm_landscape.h:153`); methods borrow that field directly and the seam
   converts Raven's `thandle_t` int at the module boundary — there is no
   random-terrain handle. Rejected a marker/unit handle: a lone `Option` field
   needs no index, and `CreateRandomTerrain` is qcommon-internal
   (its only caller is `cm_terrain.cpp:178`), never crossing the module ABI.
3. **The engine owns its own q_math LCG as a `mp_qshared::QRand` field on
   `Engine.common`, exposed via `EngineHost::flrand`/`irand`** (SETTLES RMG-Q6).
   `mp_qshared` gains a `QRand` type (the stateful LCG the game tier already
   models as `bg_channel::rng::Rng`, `crates/mp/game/src/bg_channel/rng.rs`); the
   engine holds a distinct instance `Engine.common.rng` (`engine.rs:22`,
   `common/common.rs:20`), and the one `EngineHost` trait (ruling 11) grows
   `flrand`/`irand` services backed by it. `RMG_CreateSeed`
   (`cm_randomterrain.cpp:1008`, zero live callers) draws through those services;
   the golden harness targets that instance. Rejected reaching `mp_game`'s
   game-tier LCG: `mp_engine_qcommon` must not depend on `mp_game`.
4. **`CRMArea*` → `AreaId` + arena owned by `CRMAreaManager`, per §B5** (SETTLES
   the `mArea` hole). `AreaId` (a `u32` index newtype) is added to the State
   table and threaded through `SetArea`/`GetArea` (`RM_Instance.h:72,107`) and
   the `RmInstance` variants; `mAreas` (`RM_Area.h:74,80`) becomes an owned
   `Vec<CmArea>`. Rejected raw `CRMArea*`/`Rc`: §B5 forbids aliasing pointers in
   safe code and `mAreas` is a genuine multi-element shared arena.
5. **Stored pointers into state owned elsewhere are DROPPED; the owner is
   threaded** (SETTLES the back-pointer hole). This is one recurring §B3 shape,
   stated once and applied to **every** occurrence the survey found — a field
   holding a raw pointer to state another object owns, whether a strict
   owner-back-pointer or a reach-to-collaborator. **Four** fields match:
   `CCMPatch::owner:CCMLandScape*` (`cm_landscape.h:93`),
   `CRandomTerrain::mLandScape:CCMLandScape*` (`cm_randomterrain.h:56`),
   `CRMMission::mLandScape:CRandomTerrain*` (`RM_Mission.h:64`), and
   `CRMPathManager::mTerrain:CRandomTerrain*` (`RM_Path.h:175`, set at
   construction — `RM_Path.cpp:56,60` — from the owning mission's own `mLandScape`,
   `RM_Mission.cpp:71`). Per §B3 none can be a safe Rust field, so each is dropped
   and the owner is threaded through the affected method signatures (§B4).
   `mTerrain` is dereffed only as `mTerrain->CreatePath(...)`
   (`CRandomTerrain::CreatePath`, `cm_randomterrain.cpp:605`) from
   `PathVisit`/`RiverVisit` under `GeneratePaths`/`GenerateRivers`
   (`RM_Path.cpp:327,416,564`), so the same `CRandomTerrain` (or its owning
   `CmLandScape`) that `RmMission` already threads is passed `&mut` where those
   methods carve; the exact borrow is internal (§A1), as for the other three.
   Rejected `Rc`/raw back-pointers (§B3).

**RMG-D2 — All rulings 11-18 and prior settled decisions stand unchanged.** The
first-session §F decisions carry forward verbatim; they are recorded here with
stable sub-IDs (RMG-D2a…d) so the body's cites resolve.

**RMG-D2a — The qcommon terrain twins fold into this doc.** Per **ruling 16**
(user, 2026-07-09, `engine-fork-discovery.md:143-146`): `CCMLandScape`,
`CRandomTerrain`, `CTerrainMap`, `CPathInfo`, `CArea`, `CCMPatch`,
`CCMHeightDetails` are owned by *this* subsystem, and the `cm` C-track packets
exclude them. Because the RMG tree cannot be designed without them — the RMG
classes hold `CRandomTerrain*`/`CCMLandScape*` members and reach the RNG and the
heightmap through them (`RM_Manager.h:14-15`, `RM_Mission.h:64`). Rejected a
separate qcommon C++-track doc: the split would fracture the RNG-threading and
generation dataflow across two docs. (SETTLES the former RMG-Q1.)

**RMG-D2b — State on direct `Engine` fields; services via the one `EngineHost`
trait.** Per **ruling 12** (`:127-131`): the §F states are plain
Default-initialized direct fields on `Engine` — `TheRandomMissionManager` →
`mp_engine_core::Engine.rmg: RmManager` (no `Option`/`Box`/nesting; lazy-init
timing modeled with Raven's own initialized flag), `TerrainMap`/noise scratch/
`mCurObjective` handled by RMG-D2c, const tables → `const`, cvar handles →
`EngineCvars`. Per **ruling 11** (`:121-126`): every engine service (FS, print/
error, cvar, trace, and the `flrand`/`irand` RNG added by RMG-D1 part 3) is
reached through the one `EngineHost` services trait
(Stage-0 interface crate), threaded as `&mut impl EngineHost`; `Engine`
implements it via a split-borrow view struct. Because ruling 12 resolves STATE-Q2's
placement half and ruling 11 its service half. Rejected globals/sub-structs:
the spine (§B3) and ruling 12 forbid them.

**RMG-D2c — §20-drop four dead-surface items with zero-caller notes.** Per
**ruling 17** (`:147-152`) plus graph evidence: (a) `RmManager.mCurObjective`
— zero-init only, never read or written in codemp (`RM_Manager.cpp:16`); (b)
`noiseTable`/`noisePerm` — the Perlin path is dead (`CM_NoiseInit` is `#if 0`,
`cm_randomterrain.cpp:17-28`; its call is commented, `:785-795`) yet live-read at
`:806`, so encode the deterministic **0** contribution directly, keep no field,
draw no RNG; (c) the `RM_Terrain.cpp` client-model chain (`RM_CreateRandomModels`,
`SpawnPatchModelsWrapper`, `CRMLandScape`/`CCGHeightDetails`/`CRandomModel`/
`CCGPatch`) — graph-confirmed zero engine callers under DEDICATED (ruling 17);
(d) `CTerrainMap::Upload`/`SaveImageToDisk` and, by extension, the whole
`CTerrainMap` builder — its only ctor `CM_TM_Create` is `#ifndef DEDICATED`
(`RM_Mission.cpp:1503-1504`). All four are recorded as divergences, not ported.
Because a §20 drop keeps a greppable zero-caller note instead of dead code.
Rejected porting them: no live DEDICATED caller. (SETTLES the former RMG-Q2 and
RMG-Q3.)

**RMG-D2d — Prior settled §F shape + verification stand.** Carried unchanged from
the first design session:
- **CRMInstance closed hierarchy → enum.** The base+four-subclass tree becomes
  one `RmInstance` enum (`Bsp`/`Group`/`Random`/`Void`, shared base fields on a
  common struct; the factory `CreateInstance` → a `match` on the GP2 group name),
  per §17. Because the hierarchy is provably closed — the only construction site
  is the string factory (`RM_InstanceFile.cpp:158-178`); no subclass is
  instantiated elsewhere. Rejected a `dyn` trait-object arena: the set never
  grows at runtime, and `dyn` blocks the by-value forwarding
  `CRMRandomInstance`/`CRMGroupInstance` need. The dead `"npc"` branch
  (`RM_InstanceFile.cpp:162-166`) is dropped, not given a variant (§20).
- **fork-2 state, fork-5 init shape.** Globals become the RMG-D2b fields
  (fork-2); the string factory (`CreateInstance`), the syscall-switch arms
  (`sv_game.cpp:1620-1641`), and the `neighbor_x/y` step tables keep their 1:1
  init shape — plain matches / const slices at the same sites, no fn-ID enums
  (fork-5, `:46-53`; grep finds no address comparison of these members).
- **Verification: differential goldens under `tools/rmg-oracle/`, seeded by the
  faithful LCG.** Per §18: compile the unmodified oracle TUs standalone against
  stub headers, dump canonical generation output over committed fixtures, require
  byte-for-byte reproduction (DEC-09 TU-harness track). The faithful LCG drives
  both `RMG_CreateSeed` (free `flrand`/`irand`, `q_math.c:1441-1470`) and the
  per-landscape `holdrand` (identical algorithm, separate instance,
  `cm_terrain.cpp:1548-1580`). Because RMG is deterministic given a seed.
- **Oracle goldens are the *only* referee; no OpenJK cross-check.** OpenJK
  dropped RMG entirely (`plan §3c`, `docs/plans/2026-07-08-mp-engine-build-out.md:425-428`),
  so the engine-vs-engine A/B square cannot exercise these paths. A hard
  constraint on the verification plan, not a choice.

## Verification strategy

§F / DEC-09 TU-harness track (RMG-D2d):

- **Harness** `tools/rmg-oracle/` — compile the unmodified oracle TUs
  (`RM_*.cpp`, `cm_terrain.cpp`, `cm_randomterrain.cpp`) standalone against stub
  headers (oracle never edited, §18), driven by a small dumper that registers
  terrain with a fixed config `"seed"` info key (→ the server-side landscape
  `rand_seed`, `cm_terrain.cpp:1696-1698`; `clc.rmgSeed` is the client-only path)
  and runs `LoadMission`→`SpawnMission`. The referee injects a deterministic
  `EngineHost` impl (ruling 11) for the FS/print services and the `flrand`/`irand`
  RNG services, the latter seeding a fixed `Engine.common.rng` (RMG-D1 part 3).
- **Goldens** (committed, so `cargo test` needs no C++): (1) `RMG_CreateSeed`
  seed-string + hash streams for a fixed `Engine.common.rng` (`QRand`) seed —
  the golden-only helper (zero live callers) that pins the engine LCG via
  `EngineHost::flrand`/`irand`; (2) the generated heightmap + flatten-map
  bytes and `get_rand_seed()` after `Generate` for a fixed landscape seed;
  (3) the automap-symbol list after a full mission spawn.
- **Determinism anchor**: both LCGs are bit-exact — `holdrand*214013 + 2531011`,
  `result = holdrand >> 17` (`cm_terrain.cpp:1554-1580`, `q_math.c:1445-1466`);
  any drift shows up as a first-diverging RNG draw.
- **No OpenJK peer** (RMG-D2d) — the 3c-external A/B square deliberately excludes
  these paths.

## Slice hooks

- **Wave 16** (`plan §"RMG (113 fns, wave 16)"`): the whole tree lands as one
  §F subsystem. Needs frozen first: the GP2 port (done — live dep), the type
  rosetta entries for `symmetry_t`/`ERMDir`/`rmAutomapSymbol_t` (relocated to
  `mp_qshared`, RMG-D1 part 1)/`thandle_t`/`vec3pair_t`, the `EngineHost` trait
  (ruling 11, Stage-0 interface crate) **with its `flrand`/`irand` RNG services
  backed by `Engine.common.rng: QRand`** (RMG-D1 part 3).
- **Wave 20** (`SV_GameSystemCalls`): the RMG syscall arms wire to the frozen
  seams — `G_RMG_INIT` → Seam-A `RmManager` methods (`sv_game.cpp:1624-1638`),
  `G_CM_REGISTER_TERRAIN` → Seam-B `register_terrain` (`sv_game.cpp:1640-1641`).
  `G_SET_ACTIVE_SUBBSP` → `SV_SetActiveSubBSP` (`sv_game.cpp:185,1621`) is
  out-of-scope clipmap/subBSP wiring (Non-goals: the wider clipmap), not a seam
  edge here. Needs the `Engine.rmg` field (present per ruling 12 — no longer
  blocked) and `CollisionWorld.land_scape`. The `G_RMG_INIT` lazy-construct call
  site (`sv_game.cpp:1627-1629`) builds against the frozen direct `Engine.rmg`
  field + its `initialized: bool` flag (Seam-A owned-state note): the arm checks
  `!rmg.initialized`, sets it `true` in place of Raven's `new`, then calls
  `set_landscape`/`load_mission`/`spawn_mission` — the flip is here, not in any
  `RmManager` method.
- **Wave 22** (`SV_SpawnServer`): `CM_RegisterTerrain` on the map-load path;
  needs Seam-B frozen.

## Open questions

Ruling 21 (RMG-D1) closed RMG-Q4/Q5/Q6 plus the `mArea` and back-pointer holes
(see Resolved questions); one naming question remains and must go back to a design
session before FROZEN.

- **RMG-Q7 — Rust names for the two distinct `Area` classes.** The doc's naming
  convention gives every `RM_*.h` class an `Rm` prefix (`RmManager`, `RmMission`,
  `RmInstance`, `RmObjective`, `RmPathManager`) and reserves `Cm` for the
  qcommon/collision classes (`CmLandScape`, `CmPatch`, `CmHeightDetails`). Two
  *separate* oracle classes named around "Area" need Rust names, and the doc is
  inconsistent about them:
  1. `CRMArea` (`oracle/codemp/RMG/RM_Area.h:17`) — the RMG arena element. The
     State-ownership table (`:198`), Seam (`:233-250`) and Divergences (`:648`)
     call its Rust type `CmArea`, which contradicts the doc's own `Rm`-prefix
     convention (by that rule it should read `RmArea`). Whether `CmArea` is a typo
     for `RmArea` or a deliberate choice is unsettled.
  2. `CArea` (`oracle/codemp/qcommon/cm_landscape.h:42`) — a genuinely distinct
     qcommon class (confirmed NOT the same as `CRMArea`), the `area` argument of
     the live `FlattenArea`/`SaveArea`/`GetWorldHeight` methods
     (`cm_terrain.cpp:1312,1128,1011`) and named `+ CArea` in the `cm_terrain.rs`
     roster (`:620`). Its Rust type is **never named anywhere in the doc**; it
     needs one, and it must not collide with whatever `CRMArea` is called.
  A rename is a design decision (porting-rules §C: specific renames are decided in
  discussion, not pre-baked), and the collision cannot be resolved from the
  convention alone — the qcommon `CArea` fits neither the `CCM*→Cm*` nor the
  `CRM*→Rm*` derivation pattern. Escalate to a design session.

  **Both names are internal (§A1), so RMG-Q7 freezes NO `## Seam definition` pub
  API.** `CmArea`/`CArea` appear only in internal types — the
  `CrmAreaManager.areas: Vec<…>` element and the `area` parameter of the internal
  `CmLandScape::FlattenArea`/`SaveArea`/`GetWorldHeight` methods — none of which is
  in the frozen module seam (the frozen pub API is `register_terrain`,
  `create_random_terrain`, `rmg_create_seed`, and the `RmManager` methods, none of
  which name an area type). So a skeleton can be produced without this question
  answered; only these two internal type names remain provisional.

  **Provisional handling until RMG-Q7 settles** (so the body is self-consistent and
  a porter invents nothing): (1) treat every `CmArea` occurrence in the
  State/Seam/Divergences rows as a **non-final placeholder** for `CRMArea`'s arena
  element — it deliberately violates the `Rm`-prefix convention and may be renamed
  (e.g. to `RmArea`) by the session; (2) the qcommon `CArea` has **no** Rust name —
  a skeleton leaves its `area` parameter as a `//TODO: Port CArea`
  (`// Source: oracle/codemp/qcommon/cm_landscape.h:42`) marker rather than
  inventing one. The session assigns both names, guarantees they do not collide,
  and reconciles every body occurrence.

## Resolved questions

Closed by the 2026-07-09 §F rulings (recorded so a re-reader sees why they left
the open list):

- **RMG-Q1 — Fold the qcommon terrain twins in?** RESOLVED by ruling 16 → RMG-D2a
  (fold them in; `cm` C-track excludes them).
- **RMG-Q2 — Are the `RM_Terrain.cpp` client-model classes in the dedicated link
  set?** RESOLVED by ruling 17 → RMG-D2c (no — graph-confirmed zero engine callers
  under DEDICATED; §20-dropped).
- **RMG-Q3 — Classify the dead Perlin-noise scratch.** RESOLVED by ruling 17 →
  RMG-D2c (§20-drop; encode the deterministic 0 directly, no field, no RNG draw).
- **RMG-Q4 — Crate placement / dependency edge for `rmAutomapSymbol_t`.** RESOLVED
  by **ruling 21 → RMG-D1 part 1**: the type relocates to `mp_qshared` (which
  `mp_engine_rmg` already depends on); `RmManager::automap_symbol` returns
  `Option<&RmAutomapSymbol>` directly, with no `rmg → mp_engine_client` edge.
- **RMG-Q5 — Concrete Rust form of `RandomTerrainHandle`.** RESOLVED by **ruling
  21 → RMG-D1 part 2**: there is no handle. `CRandomTerrain` is the single owned
  `CmLandScape.random_terrain: Option<RandomTerrain>` field; methods borrow it
  directly and the seam converts Raven's `thandle_t` int at the module boundary.
- **RMG-Q6 — Engine-tier owner for the free `flrand`/`irand` LCG.** RESOLVED by
  **ruling 21 → RMG-D1 part 3**: the engine owns its own q_math LCG as a
  `mp_qshared::QRand` field `Engine.common.rng`, exposed via `EngineHost::flrand`/
  `irand`; the golden harness targets that instance.
- **`RmManager.mCurObjective`.** RESOLVED by ruling 17 → RMG-D2c (§20-dropped,
  never read/written).
- **STATE-Q2 (placement + service halves) for `rmg`.** RESOLVED by ruling 12
  (direct `Engine.rmg` field) + ruling 11 (`EngineHost` service seam) → RMG-D2b.

## Files roster

C++-track roster for `.claude/workflows/port-cpp-subsystem.js` (`designPath`).
`mode: mp` throughout (dedicated MP engine; SP twin out of scope, DEC-04). The
four §20-dropped items (RMG-D2c) are **not** porter work orders — they appear only
in `divergences` below.

**Crate dependencies (mechanical).** `mp_engine_rmg`'s `Cargo.toml` today depends
only on `mp_qshared`; folding the twins (RMG-D2a) requires an added
`mp_engine_qcommon` path dependency so `mp_engine_rmg` can name
`CmLandScape`/`CollisionWorld` in its frozen pub API. Per **RMG-D1 part 1** the
automap-symbol type `rmAutomapSymbol_t` relocates to `mp_qshared` (already a
dependency), so **no** `mp_engine_client` edge is added.

```yaml
files:
  # --- mp_engine_rmg (oracle/codemp/RMG/) ---
  - { path: crates/mp/engine/rmg/src/rm_manager.rs,        crate: mp_engine_rmg,      mode: mp, class: CRMManager,       summary: "Random-mission manager singleton; load/spawn mission, automap symbols (17 fns, RM_Manager.cpp; mCurObjective/WriteAutomapSymbols/ProcessAutomapSymbols §20-dropped)" }
  - { path: crates/mp/engine/rmg/src/rm_mission.rs,        crate: mp_engine_rmg,      mode: mp, class: CRMMission,       summary: "Mission file parse + Spawn: origins/nodes/paths/rivers/instances/objectives/difficulty (24 fns, RM_Mission.cpp — the bulk; the #ifndef DEDICATED CTerrainMap block §20-dropped)" }
  - { path: crates/mp/engine/rmg/src/rm_instance.rs,       crate: mp_engine_rmg,      mode: mp, class: RmInstance,        summary: "Closed instance hierarchy as one enum (RMG-D2d): base CRMInstance + Bsp/Group/Random/Void variants and their PreSpawn/Spawn/PostSpawn (RM_Instance*.cpp, 24 fns across 5 files; npc branch §20-dropped)" }
  - { path: crates/mp/engine/rmg/src/rm_area.rs,           crate: mp_engine_rmg,      mode: mp, class: CRMArea,          summary: "CRMArea + CRMAreaManager: area placement, mirror, look-at, move (8 fns, RM_Area.cpp)" }
  - { path: crates/mp/engine/rmg/src/rm_path.rs,           crate: mp_engine_rmg,      mode: mp, class: CRMPathManager,   summary: "Path/river grid generation over CRMNode/CRMLoc/CRMCell; GeneratePaths/GenerateRivers (15 fns, RM_Path.cpp). The CRandomTerrain* mTerrain back-pointer (RM_Path.h:175) is DROPPED per RMG-D1 part 5 (§B3): its only use is mTerrain->CreatePath at :327,416,564, so the owning CRandomTerrain/CmLandScape is threaded &mut through PathVisit/RiverVisit instead" }
  - { path: crates/mp/engine/rmg/src/rm_instance_file.rs,  crate: mp_engine_rmg,      mode: mp, class: CRMInstanceFile,  summary: "GP2-backed instance-file open/close + CreateInstance string factory → RmInstance (5 fns, RM_InstanceFile.cpp)" }
  - { path: crates/mp/engine/rmg/src/rm_objective.rs,      crate: mp_engine_rmg,      mode: mp, class: CRMObjective,     summary: "Objective parse + Link (2 fns, RM_Objective.cpp)" }
  # --- mp_engine_qcommon (oracle/codemp/qcommon/) — terrain twins, RMG-D2a ---
  - { path: crates/mp/engine/qcommon/src/cm_terrain.rs,     crate: mp_engine_qcommon, mode: mp, class: CCMLandScape,      summary: "Common landscape: CCMLandScape + CCMPatch + CCMHeightDetails + CArea (qcommon CArea, RM_Area.h's CRMArea is a DISTINCT class — Rust names are RMG-Q7); patch collide, heightmap, and the LIVE area methods GetWorldHeight/FlattenArea/SaveArea (ported — live RMG callers, see Seam-B), inline per-instance LCG rand_seed/flrand/irand (cm_terrain.cpp, ~32 fns; §20-dropped: mRefCount [DEC-01], the twelve cm_landscape.h:245-265 CM_* free-fn wrappers, and the nine dead area methods AreaCollision/GetFirst|NextArea/FractionBelowLevel/CarveBezierCurve/GetFirst|Player|NextObjectiveArea)" }
  - { path: crates/mp/engine/qcommon/src/cm_randomterrain.rs, crate: mp_engine_qcommon, mode: mp, class: CRandomTerrain,  summary: "CRandomTerrain + CPathInfo spline paths, Generate/Smooth/ParseGenerate, RMG_CreateSeed (golden-only, zero live callers). The Perlin path is §20-DROPPED (RMG-D2c): noiseTable/noisePerm are never written (CM_NoiseInit #if 0 at :17-28, call commented :785-795); CM_NoiseGet4f's deterministic 0 at :806 is encoded directly, no field, no RNG draw" }
```

Existing skeleton already present: `crates/mp/engine/rmg/src/rm_headers/symmetry_t.rs`
(`symmetry_t`, `RM_Headers.h:29-35`) and `rm_path/ermdir.rs` (`ERMDir`,
`RM_Path.h:24-37`) — the faithful C enums; the class files above build on them.

## Divergences

Idiomatic §F reshapings (layout-free — these types never cross the module ABI)
and the §20 drops (RMG-D2c) a transcriber records rather than ports:

```yaml
divergences:
  - { class: CRMInstance,     kind: reshape, rule: "§17/RMG-D2d", note: "virtual base+4 subclasses → RmInstance enum; CreateInstance factory → match on GP2 group name" }
  - { class: CRMRandomInstance, kind: reshape, rule: "§B5",     note: "CRMInstance* mInstance forward-pointer → Box<RmInstance>; virtuals forwarded by delegation (RM_Instance_Random.h:22-29)" }
  - { class: CRMGroupInstance,  kind: reshape, rule: "§B5",     note: "rmInstanceList_t mInstances (list<CRMInstance*>) → Vec<RmInstance>; RemoveInstances → Drop/clear (RM_Instance_Group.cpp:204)" }
  - { class: CRMManager,      kind: reshape, rule: "§B/RMG-D2b",  note: "raw CRMMission* → owned field; the cached CCMLandScape* mLandScape (RM_Manager.h:14) AND CRandomTerrain* mTerrain (RM_Manager.h:15, set together by SetLandScape = landscape->GetRandomTerrain(), never diverge) are NOT stored — RmManager owns only the TerrainHandle and both resolve through it via the threaded `cm` (Seam deviation note: mTerrain == cm.land_scape[handle].random_terrain, e.g. LoadMission's `if (!mTerrain) return false` → `random_terrain.is_none()`); TheRandomMissionManager → direct Engine.rmg field (ruling 12), no Option; lazy-init via a Raven-faithful private `initialized: bool` field (Default false, flipped at the G_RMG_INIT arm, not in any method — ctor RM_Manager.cpp:34-42 only zeroes members, so new()/Default and the lazy new collapse to one construction)" }
  - { class: CRMManager,      kind: drop,    rule: "§20/ruling17", note: "mCurObjective (RM_Manager.cpp:16) zero-init, never read/written; WriteAutomapSymbols (:424) commented-out; ProcessAutomapSymbols (:442) is a client-side static, dead under DEDICATED. All dropped with zero-caller notes" }
  - { class: CRMPathManager,  kind: reshape, rule: "§F",        note: "rmNodeVector_t/rmLocVector_t/rmCellVector_t (vector<T*>) → Vec<Node>/Vec<Loc>/Vec<Cell>; Node(x,y)=mNodes[x+y*mXNodes] index math preserved verbatim (RM_Path.h:185)" }
  - { class: CRMInstanceFile, kind: reshape, rule: "§F",        note: "CGenericParser2/CGPGroup* members → borrows into the ported GP2 arena (crates/.../gp2)" }
  - { class: CCMLandScape,    kind: reshape, rule: "§B5",       note: "byte* mHeightMap/mFlattenMap, CCMPatch* mPatches, list<CArea*> → owned Vec<u8>/Vec<CmPatch>/Vec<CArea>; std::list iterator members (mAreasIt) → index cursor; holdrand LCG stays an inline c_ulong field (Raven `unsigned long`, cm_landscape.h:160; platform-width per the State table) + flrand/irand/rand_seed methods (cm_terrain.cpp:1548-1580)" }
  - { class: CCMLandScape,    kind: drop,    rule: "§20/DEC-01/RMG-D2c", note: "CM_TerrainPatchIterate (free fn cm_terrain.cpp:1628) + CCMLandScape::TerrainPatchIterate (method it forwards to, cm_terrain.cpp:997) dropped: exactly two callers — renderer (tr_terrain.cpp:923, DEC-01) and RM_CreateRandomModels/SpawnPatchModelsWrapper (RM_Terrain.cpp:493), the RM_Terrain.cpp client-model chain already §20-dropped by RMG-D2c(c) — both dead under DEDICATED, zero live callers. Not a Seam-B entry; recorded, not ported. Dropping it also moots the shallow-const `const CCMLandScape*`→non-const `CCMPatch*` callback borrow (no faithful &CmLandScape rendering; no §17 signature choice needed for dead surface)" }
  - { class: CCMLandScape,    kind: drop,    rule: "§20/DEC-01", note: "mRefCount (cm_landscape.h:138) dropped: its only reader is CM_ShutdownTerrain's count-gated free (cm_load.cpp:1073-1077), whose only caller is the renderer (tr_terrain.cpp:1050, DEC-01-deferred); the server frees unconditionally at teardown (cm_load.cpp:800-809). register_terrain still returns the existing TerrainHandle on repeat registration (get-or-create on Option<CmLandScape>, cm_load.cpp:1040-1044)" }
  - { class: CRandomTerrain,  kind: drop,    rule: "§20/RMG-D2c", note: "dead Perlin path: noiseTable/noisePerm zero-init and NEVER written (CM_NoiseInit is #if 0 at cm_randomterrain.cpp:17-28; its only call is inside a /* */ comment at :785-795), yet live-read at :806 — CM_NoiseGet4f returns a deterministic 0. Encode the 0 contribution directly; keep NO field and draw NO RNG (recomputing via CM_NoiseInit would draw 256 flrand/irand and break golden #2)" }
  - { class: CTerrainMap,     kind: drop,    rule: "§20/ruling17", note: "whole automap-image builder dead under DEDICATED: its only ctor CM_TM_Create is #ifndef DEDICATED (RM_Mission.cpp:1503-1504); Upload/SaveImageToDisk named by ruling 17. Recorded with a zero-caller note, not ported (returns to scope if the renderer is un-deferred, DEC-01)" }
  - { class: CRMLandScape,    kind: drop,    rule: "§20/ruling17", note: "RM_Terrain.cpp client-model chain (CRMLandScape/CCGHeightDetails/CRandomModel/CCGPatch, RM_CreateRandomModels, SpawnPatchModelsWrapper) — graph-confirmed zero engine callers under DEDICATED (ruling 17); reached only from the client (RM_CreateRandomModels ← cl_cgame.cpp:1707). Dropped, not ported" }
  - { class: CRMArea,         kind: reshape, rule: "§B5/§17/RMG-D1p4", note: "CRMAreaManager owns rmAreaVector_t mAreas (vector<CRMArea*>, RM_Area.h:74,80) and hands out raw CRMArea* via CreateArea/EnumArea (:91,93); CRMInstance::mArea (RM_Instance.h:33) is stored long-term by SetArea (mArea=area, :72; RM_Mission.cpp:435,467,853,942) and dereffed by GetArea (return *mArea, :107), consumers walking mArea-> (GetOrigin/GetAngle, :108-110). §B5 arena+id+borrow-wrapper: mAreas → owned Vec<CmArea> in CrmAreaManager, mArea → an AreaId index newtype (rendered like EntityId), GetArea → arena lookup; AreaId is a State-table row threaded through SetArea/GetArea and the RmInstance variants. Determined by §B5, not a representation choice — multi-element shared arena, unlike CRandomTerrain's single Option<RandomTerrain> field (RMG-D1 part 2)" }
  - { class: CRandomTerrain,  kind: reshape, rule: "§B5/RMG-D1p2", note: "Raven CreateRandomTerrain returns a bare CRandomTerrain* (cm_landscape.h:260) its only caller assigns to mRandomTerrain (cm_terrain.cpp:178) — a single owned field. Modeled as CmLandScape.random_terrain: Option<RandomTerrain> (cm_landscape.h:153); NO RandomTerrainHandle newtype, methods borrow the field directly, qcommon-internal (never crosses the module ABI)" }
  - { class: rmAutomapSymbol_t, kind: relocate, rule: "RMG-D1p1", note: "ABI type (client.h:149) the rosetta ported in mp_engine_client (crates/mp/engine/client/src/client/rm_automap_symbol_t.rs:9) RELOCATES to mp_qshared so mp_engine_rmg (already depends on mp_qshared) names it; RmManager::automap_symbol returns Option<&RmAutomapSymbol>. No rmg→mp_engine_client edge (client→rmg is the allowed direction)" }
  - { class: "flrand/irand (q_math.c:1432)", kind: reshape, rule: "§B3/RMG-D1p3", note: "the free q_math LCG over file-scope holdrand → the engine's OWN mp_qshared::QRand instance Engine.common.rng (engine.rs:22, common/common.rs:20), exposed via EngineHost::flrand/irand (ruling 11 + 21). RMG_CreateSeed (cm_randomterrain.cpp:1008, zero live callers) draws through those services; the game-tier bg_channel::rng::Rng (crates/mp/game/src/bg_channel/rng.rs) is a distinct instance mp_engine_qcommon must NOT reach" }
  - { class: CCMPatch,        kind: drop,    rule: "§B3/§B4/§17/RMG-D1p5", note: "recurring shape, stated once (§17, RMG-D1 part 5) — FOUR fields hold a raw pointer to state another object owns (owner-back-pointer or reach-to-collaborator), stored where safe Rust cannot: CCMPatch::owner:CCMLandScape* (cm_landscape.h:93), CRandomTerrain::mLandScape:CCMLandScape* (cm_randomterrain.h:56), CRMMission::mLandScape:CRandomTerrain* (RM_Mission.h:64), CRMPathManager::mTerrain:CRandomTerrain* (RM_Path.h:175, set at ctor RM_Path.cpp:56,60 from the mission's own mLandScape, RM_Mission.cpp:71) — all forbidden as hidden reach (§B3: no Rc/raw). Dropped; the owner is threaded explicitly (§B4), the same form the seam already uses (register_terrain/create_random_terrain's `land: &mut CmLandScape`, CRMManager's `cm: &mut CollisionWorld`). The RNG-forwarding methods these back-pointers served (CRandomTerrain flrand/irand/rand_seed → mLandScape, cm_randomterrain.h:70-73; CRMMission DenyPickup* → mLandScape->flrand, RM_Mission.h:93-97) resolve to CmLandScape's inline per-instance LCG (State table); mTerrain is dereffed only as mTerrain->CreatePath (CRandomTerrain::CreatePath, cm_randomterrain.cpp:605) at RM_Path.cpp:327,416,564 (PathVisit/RiverVisit under GeneratePaths/GenerateRivers), so the threaded CRandomTerrain/CmLandScape is passed &mut where the carve happens. Per-method signatures are internal (§A1); the owner is passed &/&mut where the method needs it" }
  - { class: CCMLandScape,    kind: drop,    rule: "§20",         note: "WRAPPER-vs-METHOD: do not conflate. (1) The twelve cm_landscape.h:245-265 CM_* FREE-FUNCTION WRAPPERS are all dead — CM_GetWorldHeight (:247), CM_FlattenArea (:248), CM_CarveBezierCurve (:249), CM_SaveArea (:250), CM_FractionBelowLevel (:251), CM_AreaCollision (:252), CM_GetFirstArea/CM_GetFirstObjectiveArea/CM_GetPlayerArea/CM_GetNextArea/CM_GetNextObjectiveArea (:253-257) — each grep-resolves to only its decl + cm_terrain.cpp:1633-1685 wrapper def, no caller in codemp; SV_LoadMissionDef (:262) is declared-never-defined — all dropped. (2) Of the same-named CCMLandScape METHODS the wrappers forward to (per-file review, as done for CM_TerrainPatchIterate's method at :997), NINE are also dead and dropped: AreaCollision/GetFirstArea/GetNextArea (:1488,1412,1462; only caller is the already-§20-dropped RM_Terrain.cpp chain :417,251,257,273, and internal AreaCollision use) and FractionBelowLevel/CarveBezierCurve/GetFirstObjectiveArea/GetPlayerArea/GetNextObjectiveArea (:1379,1245,1422,1442,1472; zero callers anywhere). Recorded, not ported. The remaining THREE methods are LIVE and PORTED — see next bullet" }
  - { class: CCMLandScape,    kind: reshape, rule: "§20 per-file review", note: "CORRECTION to a blanket read of the drop above — CCMLandScape::FlattenArea (cm_terrain.cpp:1312), SaveArea (:1128) and GetWorldHeight (:1011) are LIVE methods and MUST be ported (as methods on CmLandScape), despite their CM_* free-function wrappers being dropped. Live callers on the doc's own central generation/placement dataflow: FlattenArea <- RM_Path.cpp:346,583 (CRMPathManager GeneratePaths/GenerateRivers), RM_Instance.cpp:77 (CRMInstance placement); SaveArea <- RM_Mission.cpp:1573,1581 (CRMMission::Spawn objective-area save); GetWorldHeight <- RM_Instance_BSP.cpp:122,160 (CRMBSPInstance PreSpawn/Spawn placement — the RmInstance::Bsp variant), RM_Mission.cpp:403,412 (CRMMission node placement). Dropping them would silently break heightmap-mutation and BSP-placement logic. Their CArea* param is the qcommon CArea class (Rust name RMG-Q7); signatures are internal (§A1)" }
```
