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
- `docs/handoffs/engine-fork-discovery.md` — settled forks; this doc consumes
  ruling 2 (global placement → `Engine` sub-structs), ruling 3 (function-scope
  statics, three-kind rule), ruling 5 (dispatch tables 1:1 init), ruling 7 (the
  blessed 5-doc §F list — RMG is one).
- `docs/architecture/state-ownership.md` — the STATE-* ledger this doc builds on:
  STATE-D5 (the one `Engine` island is defined in `mp_engine_core`, not the
  server crate — `crates/mp/engine/core/src/engine.rs:20`), STATE-D2
  (`Engine.cm: mp_engine_qcommon::CollisionWorld` is the Rust owner of Raven's
  `cmg` clipmap — `state-ownership.md:418`, `collision_world.rs:10`), STATE-Q2
  (the four §F subcrates' — including rmg's — `Engine`-island attachment point is
  **unresolved** and owned there, not here — `state-ownership.md:476,1868`).
- `docs/decisions.md` — DEC-01 (renderer deferred), DEC-04 (strict per-mode),
  DEC-09 (engine verification: TU harnesses + live peers).
- GP2 is the §F exemplar and a *live dependency* of this subsystem
  (`crates/mp/engine/qcommon/src/gp2/`, `tools/gp2-oracle/`): every RMG parse
  path runs through `CGenericParser2`/`CGPGroup`, already ported.

## Scope & non-goals

**In scope — the RMG class tree** (`oracle/codemp/RMG/`, 12 `.cpp` sources):
`CRMManager`, `CRMMission`, the closed `CRMInstance` hierarchy
(`CRMBSPInstance`, `CRMGroupInstance`, `CRMRandomInstance`, `CRMVoidInstance`),
`CRMAreaManager`/`CRMArea`, `CRMPathManager` (with `CRMNode`/`CRMLoc`/`CRMCell`),
`CRMInstanceFile`, `CRMObjective`, and the client-model classes in
`RM_Terrain.cpp` (`CRMLandScape`/`CCGHeightDetails`/`CRandomModel`/`CCGPatch` —
see RMG-Q2).

**In scope — the qcommon terrain twins RMG drives** (`oracle/codemp/qcommon/`):
`CCMLandScape` + `CCMPatch`/`CCMHeightDetails`/`CArea` (`cm_terrain.cpp`,
`cm_landscape.h`), `CRandomTerrain` + `CPathInfo` (`cm_randomterrain.cpp/.h`),
`CTerrainMap` (`cm_terrainmap.cpp/.h`). The blessed 5-doc list (fork-discovery
ruling 7) named only "RMG (CRMManager/instance hierarchy)"; this doc **proposes**
folding the terrain twins here because they are inseparable from RMG (the RMG
class tree holds `CRandomTerrain*`/`CCMLandScape*` members and reaches the shared
RNG through them — `oracle/codemp/RMG/RM_Manager.h:14-15`,
`oracle/codemp/RMG/RM_Mission.h:64`). This scope extension is **not** settled by
the inputs — see RMG-Q1.

**Non-goals** (punted, each with its owner):
- **SP RMG** (`oracle/code/RMG/`, a near-duplicate tree). Out of scope: this is
  the dedicated MP engine (`docs/GOAL-engine.md` "Explicitly after this
  process"); per-mode discipline (DEC-04) forbids unifying it. SP engine is a
  later campaign.
- **The wider clipmap** (`cm_load.cpp`, `cm_patch.cpp`, `cm_trace.cpp`). Only the
  terrain-owned members of `CCMLandScape` are here; `CM_RegisterTerrain`'s
  clipmap wiring is a C-track qcommon packet.
- **Renderer-side terrain draw** (`tr_terrain*`). Deferred with the renderer
  (DEC-01); not in the dedicated link set.
- **`CRMNPCInstance`** — a fifth instance kind, dead in Raven (its `new` is
  commented out at the factory, `oracle/codemp/RMG/RM_InstanceFile.cpp:162-166`).
  Dropped per §20 (RMG-D1 note).

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
   `cm_terrain.cpp:1696-1698`). `SpawnMission` (`RM_Manager.cpp:391`) then drives
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
- `CRMObjective* CRMManager::mCurObjective` — static member
  (`RM_Manager.cpp:16`).
- `static CTerrainMap* TerrainMap` — file-scope in the terrain-map builder
  (`oracle/codemp/qcommon/cm_terrainmap.cpp:14`).
- `static float noiseTable[256]` / `static int noisePerm[256]` — zero-initialized
  file-scope statics that are **never written**: their only writer `CM_NoiseInit`
  is inside `#if 0` (`oracle/codemp/qcommon/cm_randomterrain.cpp:17-28`) and its
  sole call is inside a `/* */` comment block (`:785-795`). They are still live-
  **read** during `CRandomTerrain::Generate` (`CM_NoiseGet4f` at `:806` →
  `GetNoiseValue`/`INDEX` → `noiseTable`/`noisePerm`, `:30-40`), where the all-zero
  tables yield a deterministic 0 noise contribution (decl `:14-15`).
- `static TCharacterPiece Consonants[]…` — const seed-name tables
  (`cm_randomterrain.cpp:847+`).
- `static int CRMPathManager::neighbor_x/y[DIR_MAX]` — const step tables
  (`oracle/codemp/RMG/RM_Path.h:172-173`).
- `static int instanceID` in `CreateInstance` — assigned-never-read scratch
  (`RM_InstanceFile.cpp:140`).
- cvars `com_RMG` (`oracle/codemp/qcommon/common.cpp:72,1335`),
  `com_terrainPhysics` (`oracle/codemp/qcommon/cm_landscape.h:267`).
- Per-instance (not global) RNG state: `CCMLandScape::holdrand`
  (member decl `cm_landscape.h:160`, seeded `cm_terrain.cpp:122`) — see RNG
  threading.

## State ownership

Per fork-discovery ruling 2 (`Engine` sub-structs, no `static mut`) and §B. The
RMG tree is generation-scoped, so the manager and its owned graph live as an
`Option` field cleared at map load (mirroring Raven's `delete
TheRandomMissionManager` at `oracle/codemp/qcommon/cm_load.cpp:800-803`).

| Raven global | oracle cite | Rust owner (crate::Type.field) | constructed by | threaded via |
| --- | --- | --- | --- | --- |
| `TheRandomMissionManager` | `RM_Manager.cpp:23` | `mp_engine_core::Engine.rmg: Option<RmManager>` (the island crate — **not** `mp_engine_server`, STATE-D5; today's `Engine` has no such field — `engine.rs:35` comment — and *which* struct/crate the `rmg` field lands on is deferred to **STATE-Q2**, `state-ownership.md:476,1868`, not decided here) | `G_RMG_INIT` case (lazy) — `sv_game.cpp:1627-1629` | `&mut` from the syscall switch inward |
| `CRMManager::mCurObjective` | `RM_Manager.cpp:16` | `RmManager.cur_objective: Option<ObjectiveId>` | zero-init only (`RM_Manager.cpp:16`); never written or read anywhere in codemp — dead static (grep: decl `RM_Manager.h:57` + init `:16` are its only refs). Keep-field vs §20-drop: see contested note | field of the owning manager |
| `CCMLandScape*` (`cmg.landScape`) | `cm_landscape.h:135` (class), `cm_local.h:155` (`cmg.landScape`), `sv_game.cpp:1631` | `mp_engine_qcommon::CollisionWorld.land_scape: Option<CmLandScape>` (a new field on the **existing** STATE-D2 `cmg` owner `CollisionWorld` — `collision_world.rs:10`; there is no `ClipMap` type) | `CM_RegisterTerrain` — `cm_load.cpp:1036,1055` | `TerrainHandle` (wrapping `thandle_t`) across the seam; borrow inward |
| `CRandomTerrain*` | `cm_randomterrain.h:52`, `RM_Manager.h:15` | owned inside `CmLandScape.random_terrain: Option<RandomTerrain>` | `CreateRandomTerrain` — `cm_landscape.h:260` | borrow from its `CmLandScape` |
| `static CTerrainMap* TerrainMap` | `cm_terrainmap.cpp:14` | `RandomTerrainGen.terrain_map: Option<TerrainMap>` (generation-scoped) | `CM_TM_Create` — `cm_terrainmap.h:69` | owned by the generation pass; freed by `CM_TM_Free` |
| `noiseTable` / `noisePerm` | `cm_randomterrain.cpp:14-15` | owned zeroed scratch on `RandomTerrain` per RMG-D2 (kept-field vs §20-drop classification open — RMG-Q3) | **never written** — `CM_NoiseInit` is dead (`#if 0`, `cm_randomterrain.cpp:17-28`) and its sole call is inside a `/* */` comment (`:785-795`) | live-**read** by `CM_NoiseGet4f` at `:806` → deterministic **0** contribution; the tables themselves draw **no** RNG (recomputing via `CM_NoiseInit` would draw 256 `flrand`/`irand` — breaking golden #2) |
| `Consonants[]` etc. seed-piece tables | `cm_randomterrain.cpp:847+` | `const` slices (fork-3 kind-1) | — | module `const` |
| `CRMPathManager::neighbor_x/y` | `RM_Path.h:172-173` | `const NEIGHBOR_X/Y: [i32; DIR_MAX]` (fork-3 kind-1) | — | module `const` |
| `CreateInstance::instanceID` | `RM_InstanceFile.cpp:140` | dropped — assigned-never-read (fork-3, §20) | — | — |
| `com_RMG`, `com_terrainPhysics` | `common.cpp:72`, `cm_landscape.h:267` | `EngineCvars` handles (fork-2) | `Cvar_Get` at init | read via cvar accessor |
| `CCMLandScape::holdrand` | `cm_landscape.h:160` | `CmLandScape.rng: Rng` (per-instance, **not** global; the `Rng` type is game-tier at `crates/mp/game/src/bg_channel/rng.rs` and not yet engine-reachable — RMG-Q6) | `CCMLandScape` ctor seeds `0x89abcdef` (`cm_terrain.cpp:122`) | field; see RNG threading |

## Seam definition

RMG crosses **two** boundaries; nothing here crosses the *module* ABI (no
`#[repr(C)]` layout constraint — §F), so all types below are idiomatic.

**Handle types (§B5, layout-free).** The seam names two handles, defined here
so porters do not invent them:

- `TerrainHandle` — a newtype over the rosetta's `thandle_t`
  (`type thandle_t = c_int`, `crates/native/types/src/lib.rs:65`); it is the
  ABI-crossing id the syscall returns (`GetTerrainId()` returns `thandle_t`,
  `cm_landscape.h:220`; `mTerrainHandle`, `:139`).
- `RandomTerrainHandle` — an **internal** §B5 handle to the `RandomTerrain`
  owned inside its `CmLandScape` (Raven `CreateRandomTerrain` returns a bare
  `CRandomTerrain*`, `cm_landscape.h:260`, **not** a `thandle_t`); it never
  crosses the module ABI. Its concrete Rust form is **unresolved**: the
  State-ownership table owns `CRandomTerrain` as a single
  `CmLandScape.random_terrain: Option<RandomTerrain>` (no collection to index),
  which contradicts a "borrow-back id into an arena" — a §B5 id has nothing to
  index against one optional field. Whether the handle is a unit/marker type, an
  existence check, or the ownership becomes a keyed collection is not settled —
  see **RMG-Q5**.

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
    pub fn load_mission(&mut self, cm: &mut CollisionWorld, is_server: bool) -> bool;
    /// `CRMManager::SpawnMission` — RM_Manager.cpp:391
    pub fn spawn_mission(&mut self, cm: &mut CollisionWorld, is_server: bool) -> bool;
    /// `CRMManager::GetAutomapSymbolCount` — RM_Manager.cpp:413
    pub fn automap_symbol_count(&self) -> i32;
    /// `CRMManager::GetAutomapSymbol` — RM_Manager.cpp:418
    pub fn automap_symbol(&self, index: i32) -> Option<&RmAutomapSymbol>;
}
```

**Seam deviation — the added `cm: &mut CollisionWorld` parameter (not a design
change).** Raven's `CRMManager::LoadMission`/`SpawnMission` take only
`qboolean IsServer` (`oracle/codemp/RMG/RM_Manager.cpp:96,391`) and reach the
landscape through the `cmg.landScape` file global. Per §B (no hidden globals)
and the State-ownership table, `RmManager` owns **only** a `TerrainHandle`; the
`CCMLandScape` data lives in `CollisionWorld` (STATE-D2, `Engine.cm` —
`collision_world.rs:10`). So both methods take the owning `CollisionWorld`
explicitly to resolve that handle — the state-threading form (§B4) of Raven's
global reach, not added behavior. (This is why `mp_engine_rmg` needs the gated
`mp_engine_qcommon` edge to name `CollisionWorld` — see "Crate dependencies"
under Files roster, gated on RMG-Q1.)

`rmAutomapSymbol_t` is an existing ABI type (`oracle/codemp/client/client.h:149`,
`MAX_AUTOMAP_SYMBOLS = 512` `:151`); the rosetta already ported it as
`rmAutomapSymbol_t` in crate **`mp_engine_client`**
(`crates/mp/engine/client/src/client/rm_automap_symbol_t.rs:9`). But
`mp_engine_rmg`'s `Cargo.toml` depends only on `mp_qshared` and has **no** edge
to `mp_engine_client`, so `RmManager::automap_symbol`'s return type cannot name
that type as-is. Whether to add an `rmg → client` dependency edge, relocate the
type to a shared crate, or wrap it is a build-graph decision the inputs do not
settle — **RMG-Q4**, not resolved here.
The live automap serializer is the server-side `SV_WriteRMGAutomapSymbols`
(`oracle/codemp/server/sv_client.cpp:670`), which walks the count/get pair (edges
#5/#6); `CRMManager::WriteAutomapSymbols` (`RM_Manager.cpp:424`) is commented-out
dead code and is dropped per §20 (not part of the seam).
`CRMManager::ProcessAutomapSymbols` (`RM_Manager.cpp:442`) is a `static`
client-side reader; keep it colocated but gated (RMG-Q2).

### B. RMG → qcommon terrain (the free-function entry points)

`cm_landscape.h:245-265` and `cm_terrainmap.h:69-80` declare the C entry points
the server/clipmap call. The frozen `mp_engine_qcommon` surface (faithful
signatures, `thandle_t` handles per §B5):

```rust
/// `CM_RegisterTerrain` — cm_load.cpp:1036
pub fn register_terrain(cm: &mut CollisionWorld, config: &str, server: bool) -> TerrainHandle;
/// `CreateRandomTerrain` — cm_landscape.h:260 (parses the `"seed"` info key
/// from `config` and calls `landscape->rand_seed(seed)` — cm_terrain.cpp:1688-1700)
pub fn create_random_terrain(land: &mut CmLandScape, config: &str, heightmap: &mut [u8], width: i32, height: i32) -> RandomTerrainHandle;
/// `CM_TerrainPatchIterate` — cm_landscape.h:245
pub fn terrain_patch_iterate(land: &CmLandScape, f: impl FnMut(&mut CmPatch));
/// `RMG_CreateSeed` — cm_randomterrain.cpp:1008 (uses the bg-shared LCG)
pub fn rmg_create_seed(rng: &mut Rng) -> (String, u32);
```

`CRandomTerrain` forwards `flrand`/`irand`/`rand_seed`/`get_rand_seed` to its
`CCMLandScape` (`cm_randomterrain.h:70-73`) — model that as method delegation on
the owned `CmLandScape`, not a duplicated LCG. **But** the faithful `Rng` these
signatures name is not reachable from this engine crate as written — it lives on
the game tier (`crates/mp/game/src/bg_channel/rng.rs`), while `mp_engine_qcommon`
depends only on `mp_qshared` and must not reach `mp_game`. Typing
`CmLandScape.rng: Rng` / `rmg_create_seed(rng: &mut Rng)` needs an engine-tier
`Rng` path first — **RMG-Q6**, not resolved here.

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
`Option<CmLandScape>`; matches `cm_load.cpp:1040-1044`). This is not a new
design decision — it is the §20 dead-surface classification the caller census
forces; if the renderer is ever un-deferred (DEC-01), the field returns then.

## Decisions

**RMG-D1 — `CRMInstance` closed hierarchy → enum over instance kinds.** We
reimplement the base+four-subclass tree as one `RmInstance` enum with `Bsp`,
`Group`, `Random`, `Void` variants (shared base fields on a common struct, the
factory `CreateInstance` becoming a `match` on the GP2 group name), per §17
(closed virtual hierarchy → enum). Because the hierarchy is provably closed —
the only construction site is the string factory at `RM_InstanceFile.cpp:158-178`
and no subclass is instantiated elsewhere. Rejected a trait-object arena: the set
never grows at runtime, so `dyn` buys nothing and blocks the by-value forwarding
`CRMRandomInstance`/`CRMGroupInstance` need. The dead `"npc"` branch
(`RM_InstanceFile.cpp:162-166`) is dropped, not given a variant (§20).

**RMG-D2 — State on `Engine` sub-structs; no `static mut`.** Per fork-discovery
ruling 2: `TheRandomMissionManager` → `Engine.rmg: Option<RmManager>`,
`mCurObjective`/`TerrainMap`/noise scratch → owned fields, const tables →
`const`, cvar handles → `EngineCvars`. Because RMG's globals are exactly the
subsystem-struct case ruling 2 blessed. Rejected globals: the spine forbids them
(§B3).

**RMG-D3 — Dispatch/table-population sites keep their 1:1 init shape.** Per
fork-discovery ruling 5: the string factory (`CreateInstance`), the syscall-
switch arms (`sv_game.cpp:1620-1641`), and the `neighbor_x/y` step tables port
as plain matches/const slices populated at the same sites — no fn-ID enums, no
added indirection. Because grep finds no address comparison of these members;
ruling 5 governs. Rejected a registry/trait table: unjustified indirection.

**RMG-D4 — Verification: differential goldens under `tools/rmg-oracle/`.** Per
§18: compile the unmodified oracle TUs standalone against stub headers, dump
canonical generation output over committed fixtures, require byte-for-byte
reproduction (DEC-09 TU-harness track). Seed via the faithful LCG — the
`Rng` (Raven's `holdrand`, `crates/mp/game/src/bg_channel/rng.rs`) drives
`RMG_CreateSeed`, and the per-landscape `holdrand` (identical algorithm, separate
instance) drives generation. Because RMG is deterministic given a seed; goldens
make `cargo test` need no C++ toolchain. Rejected live-peer-only: no external
peer exists (RMG-D5). (The `Rng` type is game-tier and not yet reachable from the
engine crates that host this subsystem — its engine-tier crate placement is
**RMG-Q6**, unresolved.)

**RMG-D5 — Oracle goldens are the *only* referee; no OpenJK cross-check.**
OpenJK dropped RMG entirely (`plan §3c`, `docs/plans/2026-07-08-mp-engine-build-
out.md:425-428`), so the engine-vs-engine A/B square cannot exercise these paths.
Anything touching the 3 RMG syscall edges verifies only against oracle-derived
goldens/replay (3a/3b), never against OpenJK. Because the peer physically lacks
the code. No alternative — recorded as a hard constraint on the verification
plan.

## Files roster

C++-track roster for `.claude/workflows/port-cpp-subsystem.js` (`designPath`).
`mode: mp` throughout (dedicated MP engine; SP twin out of scope, DEC-04).

**Provisional — three roster entries are gated on open questions and must NOT be
handed to a porter as settled** (the roster is not FROZEN; this doc is DRAFT):

- The three `mp_engine_qcommon` terrain-twin files
  (`cm_terrain.rs`/`cm_randomterrain.rs`/`cm_terrainmap.rs`) are in-scope only if
  **RMG-Q1** folds the twins into this subsystem; if RMG-Q1 rules them a separate
  qcommon C++-track doc, they move there with half of Seam-B. Do not start them
  until RMG-Q1 resolves.
- `rm_terrain.rs` and `cm_terrainmap.rs` (the client-model / automap-image
  classes) are gated on **RMG-Q2**: a porter cannot tell whether to write live
  logic or a §20 dead-code note until the WinDed Release source list is checked.
- `cm_randomterrain.rs`'s owned-struct shape (whether `noiseTable`/`noisePerm`
  are a kept-zeroed field or dropped-with-hardcoded-0) is blocked on **RMG-Q3**;
  the rest of the file (spline paths, `Generate`/`Smooth`, `RMG_CreateSeed`) is
  not.

**Crate dependencies (mechanical, but gated).** `mp_engine_rmg`'s `Cargo.toml`
today depends only on `mp_qshared`. If RMG-Q1 folds the twins, `mp_engine_rmg`
needs an added `mp_engine_qcommon` path dependency to name `CmLandScape`/
`CollisionWorld` in its own frozen pub API (mechanically required, but do not add
it until RMG-Q1 lands). The automap-symbol return type needs **RMG-Q4** resolved
before any `mp_engine_client` edge is added.

```yaml
files:
  # --- mp_engine_rmg (oracle/codemp/RMG/) ---
  - { path: crates/mp/engine/rmg/src/rm_manager.rs,        crate: mp_engine_rmg,      mode: mp, class: CRMManager,       summary: "Random-mission manager singleton; load/spawn mission, automap symbols, objective completion (17 fns, RM_Manager.cpp)" }
  - { path: crates/mp/engine/rmg/src/rm_mission.rs,        crate: mp_engine_rmg,      mode: mp, class: CRMMission,       summary: "Mission file parse + Spawn: origins/nodes/paths/rivers/instances/objectives/difficulty (24 fns, RM_Mission.cpp — the bulk)" }
  - { path: crates/mp/engine/rmg/src/rm_instance.rs,       crate: mp_engine_rmg,      mode: mp, class: RmInstance,        summary: "Closed instance hierarchy as one enum (RMG-D1): base CRMInstance + Bsp/Group/Random/Void variants and their PreSpawn/Spawn/PostSpawn (RM_Instance*.cpp, 24 fns across 5 files)" }
  - { path: crates/mp/engine/rmg/src/rm_area.rs,           crate: mp_engine_rmg,      mode: mp, class: CRMArea,          summary: "CRMArea + CRMAreaManager: area placement, mirror, look-at, move (8 fns, RM_Area.cpp)" }
  - { path: crates/mp/engine/rmg/src/rm_path.rs,           crate: mp_engine_rmg,      mode: mp, class: CRMPathManager,   summary: "Path/river grid generation over CRMNode/CRMLoc/CRMCell; GeneratePaths/GenerateRivers (15 fns, RM_Path.cpp)" }
  - { path: crates/mp/engine/rmg/src/rm_instance_file.rs,  crate: mp_engine_rmg,      mode: mp, class: CRMInstanceFile,  summary: "GP2-backed instance-file open/close + CreateInstance string factory → RmInstance (5 fns, RM_InstanceFile.cpp)" }
  - { path: crates/mp/engine/rmg/src/rm_objective.rs,      crate: mp_engine_rmg,      mode: mp, class: CRMObjective,     summary: "Objective parse + Link (2 fns, RM_Objective.cpp)" }
  - { path: crates/mp/engine/rmg/src/rm_terrain.rs,        crate: mp_engine_rmg,      mode: mp, class: CRMLandScape,     summary: "Client-model sprinkling: CRMLandScape/CCGHeightDetails/CRandomModel/CCGPatch, density map, SpawnPatchModels (10 fns, RM_Terrain.cpp) — see RMG-Q2 (client-only, dead under DEDICATED?)" }
  # --- mp_engine_qcommon (oracle/codemp/qcommon/) — terrain twins, RMG-Q1 ---
  - { path: crates/mp/engine/qcommon/src/cm_terrain.rs,     crate: mp_engine_qcommon, mode: mp, class: CCMLandScape,      summary: "Common landscape: CCMLandScape + CCMPatch + CCMHeightDetails + CArea; patch collide, heightmap, flatten/carve, per-instance LCG rand_seed/flrand/irand (cm_terrain.cpp, ~32 fns)" }
  - { path: crates/mp/engine/qcommon/src/cm_randomterrain.rs, crate: mp_engine_qcommon, mode: mp, class: CRandomTerrain,  summary: "CRandomTerrain + CPathInfo spline paths, Generate/Smooth/ParseGenerate, RMG_CreateSeed; the Perlin-noise path is DEAD — noiseTable/noisePerm are zero-init and never written (CM_NoiseInit is #if 0 at :17-28, its call commented out at :785-795), so CM_NoiseGet4f contributes a deterministic 0 at :806 (cm_randomterrain.cpp)" }
  - { path: crates/mp/engine/qcommon/src/cm_terrainmap.rs,  crate: mp_engine_qcommon, mode: mp, class: CTerrainMap,       summary: "Automap image builder: Add{Building,Start,End,Objective,NPC,WallRect,Node,Player}, Upload, SaveImageToDisk (cm_terrainmap.cpp) — see RMG-Q2" }
```

Existing skeleton already present: `crates/mp/engine/rmg/src/rm_headers/symmetry_t.rs`
(`symmetry_t`, `RM_Headers.h:29-35`) and `rm_path/ermdir.rs` (`ERMDir`,
`RM_Path.h:24-37`) — the faithful C enums; the class files above build on them.

## Divergences

Idiomatic §F reshapings (layout-free — these types never cross the module ABI)
and the UB spots (§19) a transcriber must pin at the site:

```yaml
divergences:
  - { class: CRMInstance,     kind: reshape, rule: "§17/RMG-D1", note: "virtual base+4 subclasses → RmInstance enum; CreateInstance factory → match on GP2 group name" }
  - { class: CRMRandomInstance, kind: reshape, rule: "§B5",     note: "CRMInstance* mInstance forward-pointer → Box<RmInstance>; virtuals forwarded by delegation (RM_Instance_Random.h:22-29)" }
  - { class: CRMGroupInstance,  kind: reshape, rule: "§B5",     note: "rmInstanceList_t mInstances (list<CRMInstance*>) → Vec<RmInstance>; RemoveInstances → Drop/clear (RM_Instance_Group.cpp:204)" }
  - { class: CRMManager,      kind: reshape, rule: "§B",        note: "list/vector members + raw CRMMission*/CCMLandScape* → owned Option fields + handles; TheRandomMissionManager → Engine.rmg (RMG-D2)" }
  - { class: CRMPathManager,  kind: reshape, rule: "§F",        note: "rmNodeVector_t/rmLocVector_t/rmCellVector_t (vector<T*>) → Vec<Node>/Vec<Loc>/Vec<Cell>; Node(x,y)=mNodes[x+y*mXNodes] index math preserved verbatim (RM_Path.h:185)" }
  - { class: CRMInstanceFile, kind: reshape, rule: "§F",        note: "CGenericParser2/CGPGroup* members → borrows into the ported GP2 arena (crates/.../gp2)" }
  - { class: CCMLandScape,    kind: reshape, rule: "§B5",       note: "byte* mHeightMap/mFlattenMap, CCMPatch* mPatches, list<CArea*> → owned Vec<u8>/Vec<CmPatch>/Vec<CArea>; std::list iterator members (mAreasIt) → index cursor" }
  - { class: CCMLandScape,    kind: drop,    rule: "§20/DEC-01", note: "mRefCount (cm_landscape.h:138) dropped: its only reader is CM_ShutdownTerrain's count-gated free (cm_load.cpp:1073-1077), whose only caller is the renderer (tr_terrain.cpp:1050, DEC-01-deferred); the server frees unconditionally at teardown (cm_load.cpp:800-809). register_terrain still returns the existing TerrainHandle on repeat registration (get-or-create on Option<CmLandScape>, cm_load.cpp:1040-1044)" }
  - { class: CTerrainMap,     kind: reshape, rule: "§F",        note: "512x512x4 mImage/mBufImage stack arrays → owned boxed buffers; SaveImageToDisk over the PlatformHost FS trait" }
  - { class: CCGHeightDetails, kind: ub,     rule: "§19",       note: "GetAverageFrequency divides mTotalFrequency/mNumModels — divide-by-zero when mNumModels==0 (RM_Terrain.h:43); pick the defined guard at the site, keep out of shared fixtures" }
  - { class: CCGHeightDetails, kind: quirk,  rule: "§F",        note: "ctor memset(this,0,sizeof(*this)) on a class with no vtable — reproduce as Default/zeroed fields, not a raw memset (RM_Terrain.h:38)" }
  - { class: CRandomTerrain,   kind: quirk,  rule: "§20",       note: "dead Perlin path: noiseTable/noisePerm zero-init and NEVER written (CM_NoiseInit is #if 0 at cm_randomterrain.cpp:17-28; its only call is inside a /* */ comment at :785-795), yet live-read at :806 — CM_NoiseGet4f returns a deterministic 0. Reproduce the 0 contribution; do NOT recompute via CM_NoiseInit (would draw 256 flrand/irand and break golden #2). Kept-field vs §20-drop classification: RMG-Q3" }
```

## Verification strategy

§F / DEC-09 TU-harness track (RMG-D4, RMG-D5):

- **Harness** `tools/rmg-oracle/` — compile the unmodified oracle TUs
  (`RM_*.cpp`, `cm_terrain.cpp`, `cm_randomterrain.cpp`, `cm_terrainmap.cpp`)
  standalone against stub headers (oracle never edited, §18), driven by a small
  dumper that registers terrain with a fixed config `"seed"` info key (→ the
  server-side landscape `rand_seed`, `cm_terrain.cpp:1696-1698`; `clc.rmgSeed` is
  the client-only path) and runs `LoadMission`→`SpawnMission`.
- **Goldens** (committed, so `cargo test` needs no C++): (1) `RMG_CreateSeed`
  seed-string + hash streams for a fixed bg-`Rng` seed; (2) the generated
  heightmap + flatten-map bytes and `get_rand_seed()` after `Generate` for a
  fixed landscape seed; (3) the automap-symbol list after a full mission spawn;
  (4) the `CTerrainMap` image bytes (RMG-Q2 permitting).
- **Determinism anchor**: both LCGs are bit-exact — `holdrand*214013 + 2531011`,
  `result = holdrand >> 17` (`cm_terrain.cpp:1554-1580`); any drift shows up as a
  first-diverging RNG draw.
- **No OpenJK peer** (RMG-D5) — the 3c-external A/B square deliberately excludes
  these paths.

## Slice hooks

- **Wave 16** (`plan §"RMG (113 fns, wave 16)"`): the whole tree lands as one
  §F subsystem. Needs frozen first: the GP2 port (done — live dep), the type
  rosetta entries for `symmetry_t`/`ERMDir`/`rmAutomapSymbol_t`/`thandle_t`/
  `vec3pair_t`, and an engine-reachable `Rng` (the LCG exists on the game tier
  but is not yet reachable from the engine crates — **RMG-Q6**).
- **Wave 20** (`SV_GameSystemCalls`): the RMG syscall arms wire to the frozen
  seams — `G_RMG_INIT` → Seam-A `RmManager` methods (`sv_game.cpp:1624-1638`),
  `G_CM_REGISTER_TERRAIN` → Seam-B `register_terrain` (`sv_game.cpp:1640-1641`).
  `G_SET_ACTIVE_SUBBSP` → `SV_SetActiveSubBSP` (`sv_game.cpp:185,1621`) is
  out-of-scope clipmap/subBSP wiring (Non-goals: the wider clipmap), not a seam
  edge here. Needs the `Engine.rmg` field and `CollisionWorld.land_scape`
  present. **Blocked**: the `G_RMG_INIT` lazy-construct call site
  (`sv_game.cpp:1627-1629`) has no frozen `Engine.rmg` accessor to build against
  until STATE-Q2 freezes the field's owning struct/crate — see Open questions.
- **Wave 22** (`SV_SpawnServer`): `CM_RegisterTerrain` on the map-load path;
  needs Seam-B frozen.

## Open questions

- **RMG-Q1 — Fold the qcommon terrain twins into this doc/subsystem?** The
  blessed 5-doc list (fork-discovery ruling 7) assigned only "RMG
  (CRMManager/instance hierarchy)". This doc proposes owning `CCMLandScape`/
  `CRandomTerrain`/`CTerrainMap`/`CPathInfo`/`CArea`/`CCMPatch`/`CCMHeightDetails`
  here because the RMG tree cannot be designed without them (they carry the RNG
  and the heightmap the whole subsystem mutates). But the assignment is a scope
  decision the inputs do not settle — needs user sign-off (or a ruling that they
  stay a separate qcommon C++-track doc). **needsSession.**
- **RMG-Q2 — Are the client-model classes in the dedicated link set?**
  `RM_Terrain.cpp` (`CRMLandScape`/`CCGHeightDetails`/`CRandomModel`) and
  `CTerrainMap`'s image upload are reached only from the client
  (`RM_CreateRandomModels` ← `oracle/codemp/client/cl_cgame.cpp:1707`;
  `CTerrainMap::Upload`/`SaveImageToDisk` are client/automap). Under DEDICATED
  the client is the null stub layer. Whether `WinDed.vcproj` compiles these TUs
  (so they port as live code) or they are zero-caller dead surface to record per
  §20 is not resolvable from the RMG/qcommon sources alone — it needs the WinDed
  Release source list. **needsSession.**
- **RMG-Q3 — Classify the dead Perlin-noise scratch (`noiseTable`/`noisePerm`).**
  Oracle ground truth (not in dispute): the tables are zero-initialized file-scope
  statics that are never written — `CM_NoiseInit` is `#if 0`
  (`oracle/codemp/qcommon/cm_randomterrain.cpp:17-28`) and its only call is inside
  a `/* */` comment (`:785-795`) — yet they are live-read during `Generate`
  (`CM_NoiseGet4f` at `:806`), so they contribute a deterministic 0. RMG-D2's
  owned-field principle is **not** overturned (they never become a global). What
  the settled inputs do **not** resolve is the §19/§20 refinement now that the
  write path is known dead: keep a zeroed-owned-never-written field (fork-3 kind-2,
  as RMG-D2 assumed when it believed them live) or drop the tables as dead surface
  and encode the 0 directly (§20, effectively-const 0). Both ports must reproduce
  the 0 and draw no RNG for the tables; choosing between them is a classification
  decision a drafting/review agent must not invent. **needsSession.**
- **RMG-Q4 — Crate placement / dependency edge for `rmAutomapSymbol_t`.** The
  rosetta already ported this ABI type in crate `mp_engine_client`
  (`crates/mp/engine/client/src/client/rm_automap_symbol_t.rs:9`,
  `oracle/codemp/client/client.h:149`), but `mp_engine_rmg` depends only on
  `mp_qshared` and has no edge to `mp_engine_client`, so `RmManager::automap_symbol`'s
  frozen return type cannot name it as-is. Options — add an `rmg → client`
  dependency edge, relocate the type to a shared crate reachable by both, or have
  `RmManager` return an internal symbol type the server-side serializer converts —
  are all build-graph decisions the inputs do not settle (crate-graph direction is
  `workspace-architecture.md` territory; today client → qcommon, never into rmg).
  Raven itself has RMG include the client header, but the Rust layering is
  unresolved. **needsSession.**
- **RMG-Q5 — Concrete Rust form of `RandomTerrainHandle`.** The Seam names
  `RandomTerrainHandle` as settled §B5 vocabulary porters must not invent, but
  the State-ownership table owns `CRandomTerrain` as a single
  `CmLandScape.random_terrain: Option<RandomTerrain>` (`cm_randomterrain.h:52`,
  `RM_Manager.h:15`) — one optional field, **no** collection. A §B5
  "borrow-back id into an arena" therefore has nothing to index. Whether the
  handle is a unit/marker type, an existence check, or the ownership is instead a
  keyed collection is a representation choice the inputs do not settle; Raven's
  `CreateRandomTerrain` returning a bare `CRandomTerrain*` (`cm_landscape.h:260`)
  does not decide the Rust shape. **needsSession.**
- **RMG-Q6 — Engine-tier reachability of the faithful `Rng`.** RMG-D4 and Seam-B
  type `CmLandScape.rng`/`rmg_create_seed(rng: &mut Rng)` against the faithful
  LCG, but that `Rng` lives on the **game** tier
  (`crates/mp/game/src/bg_channel/rng.rs`, owned in `BgState`;
  `crates/mp/qshared/src/shared/q_math_rand.rs` states the LCG is game-tier, not
  qshared). Both `mp_engine_rmg` and `mp_engine_qcommon` (engine tier) depend
  only on `mp_qshared` and must not depend on `mp_game` — the engine hosts game
  as a separate dylib (confirmed via their `Cargo.toml`). So no engine-reachable
  `Rng` path exists, yet Seam-B forbids a duplicated LCG. Relocating/re-exposing
  the LCG to a crate both tiers reach (e.g. `mp_qshared`) or another layering fix
  is a build-graph decision the inputs do not settle — parallel to RMG-Q4.
  **needsSession.**
- **STATE-Q2 (inherited) — the Wave-20 construction site has no frozen
  `Engine.rmg` accessor.** The `rmg: Option<RmManager>` field's owning
  `Engine`-island struct/crate is undecided (STATE-Q2,
  `state-ownership.md:476,1868`) and is unresolved in state-ownership.md itself,
  so the cross-reference does not answer it. The Wave-20 `G_RMG_INIT`
  lazy-construct call site (`sv_game.cpp:1627-1629`) has no frozen accessor to
  build the construction call against until STATE-Q2 freezes the attachment
  point. Escalates with STATE-Q2; not independently resolvable in this doc.
  **needsSession.**
