# B3 dossier — Collision model (CM)

Survey input for design doc B3. Scope: MP `codemp/qcommon/cm_*.cpp` with SP
`code/qcommon/` contrasts. Terrain/RMG (`cm_terrain*`, `cm_randomterrain*`,
`cm_shader.cpp`, `cm_landscape.h`) flagged only — C++-track per porting-rules §F.
Xbox variants (`cm_load_xbox.cpp`, `cm_patch_xbox.cpp`) are unported platform
forks, noted but not analyzed. Cross-ref: A2-state-ownership.md §1g (CM global
census), §1m (SP deltas).

Subsystem size: MP ~15.8k lines across 20 files; SP ~14.8k across 19. The
portable native-track core (load/test/trace/patch/polylib + headers) is ~7.1k MP
lines; the rest is terrain/RMG (C++-track) and cm_draw (debug drawing).

## 1. Load path

### 1a. CM_LoadMap / CM_LoadMap_Actual

MP `CM_LoadMap` (`oracle/codemp/qcommon/cm_load.cpp:775-782`) sets
`gbUsingCachedMapDataRightNow = qtrue`, calls the static
`CM_LoadMap_Actual(name, clientload, checksum, cmg)` (:779, defined :605-770),
clears the flag. `CM_LoadMap_Actual` takes a `clipMap_t&` — the same function
loads the main map (`cmg`) and every sub-BSP (§1c).

SP `CM_LoadMap` (`oracle/code/qcommon/cm_load.cpp:814-836`) adds a
`qboolean subBSP` param: if set, it reroutes to
`CM_LoadSubBSP(va("maps/%s.bsp", name+1), qfalse)` (:818) instead of loading
into `cmg`; otherwise same pattern (:823-827). MP has no combined entrypoint —
sub-BSPs load only via `CM_LoadSubBSP` directly.

**Lump order** — identical MP/SP, from `CM_LoadMap_Actual`:

| # | Call | Lump | MP line | SP line |
|---|------|------|---------|---------|
| 1 | `CMod_LoadShaders` | `LUMP_SHADERS` | cm_load.cpp:714 | :747 |
| 2 | `CMod_LoadLeafs` | `LUMP_LEAFS` | :715 | :748 |
| 3 | `CMod_LoadLeafBrushes` | `LUMP_LEAFBRUSHES` | :716 | :749 |
| 4 | `CMod_LoadLeafSurfaces` | `LUMP_LEAFSURFACES` | :717 | :750 |
| 5 | `CMod_LoadPlanes` | `LUMP_PLANES` | :718 | :751 |
| 6 | `CMod_LoadBrushSides` | `LUMP_BRUSHSIDES` | :719 | :752 |
| 7 | `CMod_LoadBrushes` | `LUMP_BRUSHES` | :720 | :753 |
| 8 | `CMod_LoadSubmodels` | `LUMP_MODELS` | :721 | :754 |
| 9 | `CMod_LoadNodes` | `LUMP_NODES` | :722 | :755 |
| 10 | `CMod_LoadEntityString` | `LUMP_ENTITIES` | :723 | :756 |
| 11 | `CMod_LoadVisibility` | `LUMP_VISIBILITY` | :724 | :757 |
| 12 | `CMod_LoadPatches` | `LUMP_SURFACES` + `LUMP_DRAWVERTS` | :725 | :758 |

**Checksum**: the `*checksum` out-param is a whole-file
`Com_BlockChecksum(buf, iBSPLen)` over the raw BSP buffer (MP :695, SP :728).
The per-lump `CM_Checksum`/`CM_LumpChecksum` pair (MP :540-559, SP :552-571,
folding 11 lump checksums) is defined but **never called** from
`CM_LoadMap_Actual` — dead code to preserve knowingly or drop.

**Cached-map-disk-image** (`gpvCachedMapDiskImage` /
`gbUsingCachedMapDataRightNow`, MP cm_load.cpp:568-570; refines A2 §1g):

- MP: *not* a load-skip cache. Stale blob freed at entry if left over from an
  `ERR_DROP` (:656-660); the file is always re-read into a fresh `Z_Malloc`
  buffer (:668-686). Its real purpose is to hand the raw disk image to the
  renderer (`tr_bsp.cpp`) after CM parses it. The only skip-reload path is the
  `!strcmp(cm.name, name) && clientload` early return (:625-628).
- SP adds `gsCachedMapDiskImage[MAX_QPATH]` (:581) as a name tag: mismatched tag
  frees the image + `CM_ClearMap()` (:641-651); then a genuine same-map
  server-load fast path (:656-668) skips the whole re-read/re-parse, just
  re-zeroing `cm.areas`/`cm.areaPortals` and bumping `cm.checkcount`. Full
  reload (:669-797) does `CM_ClearLevelPatches()` + `Z_TagFree(TAG_BSP)`
  (:678-680) and re-tags the cache on success (:795).
- Both free the image post-load only under `Sys_LowPhysicalMemory()` (MP also
  `com_dedicated->integer`, :747-754; SP commented that check out, :766-772) —
  otherwise the renderer owns freeing it later. This is the CM↔renderer shared
  ownership from A2 §1g/§1n.

**Allocator split**: MP `Hunk_Alloc(…, h_high)` for every clipMap buffer (e.g.
:91, :122, :188); SP `Z_Malloc(…, TAG_BSP, …)` (e.g. :87, :125, :184), hence
SP's `Z_TagFree(TAG_BSP)` on reload where MP just memsets `cm` and relies on
hunk lifetime.

### 1b. clipMap_t population

`clipMap_t`: MP `cm_local.h:91-157` (non-Xbox branch), SP `cm_local.h:156-206`.
Field → filler:

| Field(s) | Type | Filled by (MP / SP) |
|---|---|---|
| `name` | `char[MAX_QPATH]` | `CM_LoadMap_Actual` tail, MP :768 / SP :807 (skipped on clientload) |
| `numShaders`/`shaders` | `CCMShader*` | `CMod_LoadShaders` MP :76-101 / SP :72-97 |
| `numBrushSides`/`brushsides` | `cbrushside_t*` | MP :407-434 / SP :401-429 |
| `numPlanes`/`planes` | `cplane_t*` | MP :312-346 / SP :306-340 |
| `numNodes`/`nodes` | `cNode_t*` | MP :175-203 / SP :171-199 |
| `numLeafs`/`leafs` | `cLeaf_t*` | MP :269-305 / SP :263-299 — also derives `numClusters`/`numAreas`, allocates `areas`/`areaPortals` (MP :303-304) |
| `numLeafBrushes`/`leafbrushes` | `int*` | MP :353-373 / SP :347-367 |
| `numLeafSurfaces`/`leafsurfaces` | `int*` | MP :380-400 / SP :374-394 |
| `numSubModels`/`cmodels` | `cmodel_t*` | MP :109-166 / SP :105-162 |
| `numBrushes`/`brushes` | `cbrush_t*` | MP :229-262 / SP :225-256 |
| `clusterBytes`/`visibility`/`vised` | | `CMod_LoadVisibility` MP :454-472 / SP :449-467 |
| `numEntityChars`/`entityString` | `char*` | `CMod_LoadEntityString` MP :442-446 / SP :437-441 |
| `numSurfaces`/`surfaces` | `cPatch_t**` | `CMod_LoadPatches` MP :483-536 / SP :478-533 |
| `floodvalid`/`checkcount` | `int` | flooded by `CM_FloodAreaConnections(cm)` MP :764 / SP :803 |
| `landScape` | `CCMLandScape*` | not in load path; `CM_RegisterTerrain` MP :1036-1057 / SP :1116-1140 (C++-track) |

The Rust struct skeletons already exist:
`crates/mp/engine/qcommon/src/cm/clip_map_t.rs` plus the full data-type set
(`c_node_t`, `c_leaf_t`, `cbrush_s`, `cbrushside_s`, `c_patch_t`, `c_grid_t`,
`patch_collide_s`, `patch_plane_t`, `facet_t`, `winding_t`, `sphere_t`,
`trace_work_s`, `leaf_list_s`, `ccmshader`, `cmodel_s`, `c_area_t`), mirrored
under `crates/sp/engine/qcommon/src/cm/`. **No function bodies are ported** —
cm_trace/cm_patch/cm_polylib/cm_load logic is entirely greenfield.

### 1c. CM_LoadSubBSP / CM_InlineModel handle math

Globals: `clipMap_t SubBSP[MAX_SUB_BSP=32]`, `int NumSubBSP, TotalSubModels`
(MP cm_load.cpp:60-61, SP :57; `MAX_SUB_BSP` at
`oracle/codemp/game/q_shared.h:2025` / `code/game/q_shared.h:1464`).

The handle scheme is **cumulative-sum offset ranges, no bit packing**:

- Handle range `[0, cmg.numSubModels)` = main-map inline models; then each
  loaded SubBSP appends its `numSubModels` contiguously.
- `CM_LoadSubBSP(name, clientload)` (MP :1083-1108, SP :1165-1190): scan
  `SubBSP[0..NumSubBSP)` by `stricmp` accumulating
  `count = cmg.numSubModels + Σ SubBSP[i].numSubModels`; on hit return the
  already-computed base offset; else error at 32 (MP :1099-1102), load into
  `SubBSP[NumSubBSP]` via `CM_LoadMap_Actual` (MP :1104), return pre-load
  `count`.
- `TotalSubModels += cm.numSubModels` inside `CM_LoadMap_Actual` (MP :727,
  SP :760) — runs for main map *and* every sub-BSP.
- Decode: `CM_ClipHandleToModel` (MP :828-875, SP :912-960) — `handle <
  cmg.numSubModels` → `&cmg.cmodels[handle]`; `handle == BOX_MODEL_HANDLE`
  special; else walk the same accumulation to find the containing range,
  return `&SubBSP[i].cmodels[handle - count]` (MP :862).
- `CM_InlineModel(index)` (MP :882-888) is a pure bounds check against
  `TotalSubModels` — the handle *is* the index.
- Inverse: `CM_FindSubBSP(modelIndex)` (MP :1110-1130, SP :1192-1212) → `-1`
  for main map, else the sub-BSP slot index.
- `BOX_MODEL_HANDLE`/`CAPSULE_MODEL_HANDLE`: MP `cm_local.h:13-14`
  (`MAX_SUBMODELS-1`/`-2`); SP has no `CAPSULE_MODEL_HANDLE` (§4).

### 1d. CM_EntityString

`CM_EntityString()` (MP cm_load.cpp:898-900, SP :981-983) returns
`cmg.entityString` — a raw pointer into the buffer copied from `LUMP_ENTITIES`
by `CMod_LoadEntityString`. `CM_SubBSPEntityString(int index)` (MP :902-905,
SP :985-988) returns `SubBSP[index].entityString`; present in both engines.

### 1e. CM_ClearMap

MP (:791-821): `CM_ShutdownShaderProperties` (:796), delete
`TheRandomMissionManager` (:800-804), delete `cmg.landScape` (:806-810),
`Com_Memset(&cmg, 0, …)` (:812), `CM_ClearLevelPatches()` (:813), zero each
live `SubBSP[i]` (:815-818), `NumSubBSP = TotalSubModels = 0` (:819-820).
SP (:868-900) identical plus `CM_OrOfAllContentsFlagsInMap = CONTENTS_BODY`
reset at :872.

## 2. Query API surface

Declared in `oracle/codemp/qcommon/cm_public.h` (74 lines) / SP
`cm_public.h` (72 lines). Semantics + definitions + primary call sites (MP
unless noted):

| Function | Semantics | Def | Primary call sites |
|---|---|---|---|
| `CM_NumClusters` | `cmg.numClusters` | cm_load.cpp:890 | renderer vis setup |
| `CM_NumInlineModels` | `cmg.numSubModels` | cm_load.cpp:894 | `CG_CM_NUMINLINEMODELS` handler `client/cl_cgame.cpp:781-782` |
| `CM_LeafCluster` | leaf → vis cluster | cm_load.cpp:907 | `SV_inPVS` chain sv_game.cpp:220-230 |
| `CM_LeafArea` | leaf → area | cm_load.cpp:914 | same chain |
| `CM_ClusterPVS` | PVS bit vector for cluster | cm_test.cpp:351 | `SV_inPVS*` sv_game.cpp:220-230, 254-261; snapshot build |
| `CM_PointLeafnum` | BSP descent to containing leaf (`CM_PointLeafnum_r` :16) | cm_test.cpp:46 | `SV_inPVS*`, `SV_AreaEntities` sector code |
| `CM_PointContents` | ORed content flags at point in model | cm_test.cpp:224 | `SV_PointContents` `server/sv_world.cpp:880`; `CG_CM_POINTCONTENTS` cl_cgame.cpp:789-790 |
| `CM_TransformedPointContents` | as above, into rotated bmodel space | cm_test.cpp:306 | per-entity clip `sv_world.cpp:897`; cl_cgame.cpp:791-792 |
| `CM_BoxLeafnums` | leafs overlapping AABB (`_r` helper :130); overflow flagged via `*lastLeaf` | cm_test.cpp:168 | `SV_AreaEntities`/linking in sv_world.cpp |
| `CM_AreasConnected` | area-portal flood connectivity (flood: `CM_FloodAreaConnections`/`CM_FloodArea_r` :427-475) | cm_test.cpp:509 | `G_AREAS_CONNECTED` direct call sv_game.cpp:628; `SV_inPVS` :230 |
| `CM_AdjustAreaPortalState` | open/close portal edge, re-flood | cm_test.cpp:476 | `SV_AdjustAreaPortalState` sv_game.cpp:275-282 |
| `CM_WriteAreaBits` | connected-area bitmask into buffer | cm_test.cpp:545 | snapshot area bits (sv_snapshot) |
| `CM_BoxTrace` | swept AABB/capsule vs model brushes+patches → `trace_t` | cm_trace.cpp:1836 (wraps `CM_Trace` :1577) | `SV_Trace` → world clip `sv_world.cpp:820`; `CG_CM_BOXTRACE`/`CAPSULETRACE` cl_cgame.cpp:793-797 |
| `CM_TransformedBoxTrace` | trace into rotated/offset model space | cm_trace.cpp:1850 | `SV_ClipMoveToEntities` `sv_world.cpp:496,609`; `SV_EntityContact` sv_game.cpp:301; cl_cgame.cpp:799-803 |
| `CM_MarkFragments` | **declared cm_public.h:65, never defined in qcommon** — `CG_CM_MARKFRAGMENTS` calls `re.MarkFragments` (renderer) directly, cl_cgame.cpp:805-806 | — | — |

Non-existent names to avoid in the doc: `CM_LeafsAlongPath`,
`CM_AreaConnected` (it's plural), `CM_SetAreaPortalState` (only `Adjust`).

SP-only public API: `CM_ModelContents`(+`_Actual`), `CM_TotalMapContents`
(`code/qcommon/cm_load.cpp:902-905`), `CM_SameMap` (:838), `CM_HasTerrain`
(:853), `CM_SubBSPEntityString`, `CM_WritePortalState`/`CM_ReadPortalState`
(:1280-1297 — savegame `SG_Append('PRTS', cmg.areaPortals, …)`/`SG_Read`).
MP-only: the `int capsule` param on `CM_BoxTrace`/`CM_TransformedBoxTrace`/
`CM_TempBoxModel` (MP cm_public.h:12,24-30 vs SP :16,32-38).

## 3. Trace internals

### 3a. trace_t contract (ported)

- MP: `crates/mp/qshared/src/common/mp/trace_t.rs` — size 48, fields
  `allsolid, startsolid, entityNum, fraction, endpos, plane, surfaceFlags,
  contents`; matches `oracle/codemp/game/q_shared.h:1894-1912` (Raven
  commented out `G2CollisionMap` in MP — "wasting space").
- SP: `crates/sp/qshared/src/common/sp/trace_t.rs` — size 1080; the
  `G2CollisionMap` field is live at offset 56
  (`oracle/code/game/q_shared.h:1395` region).
- `traceWork_t` convention: `trace_t` is deliberately the **last** field so
  Ghoul2 code can treat memory past it as the collision map
  (`crates/sp/engine/qcommon/src/cm/trace_work_s.rs:56` comment; Raven's
  original in `cm_local.h`). Preserve ordering in any refactor.

### 3b. Brush trace flow (cm_trace.cpp, MP 1992 lines)

`CM_BoxTrace` (:1836) → `CM_Trace` (:1577-1829, workhorse) →
`CM_TraceThroughTree` (:1431-1548, recursive BSP descent with
`SURFACE_CLIP_EPSILON`-biased crossings, early-out on `fraction <= p1f`) →
`CM_TraceThroughLeaf` (:976-1047) → `CM_TraceThroughBrush` (:607-690) /
`CM_TraceThroughPatch` (:500-513). `CM_TransformedBoxTrace` (:1850+) adds
rotation/offset for bmodels.

`CM_Trace` builds a `traceWork_t`: symmetric mins/maxs, 8 signbit-indexed
corner `offsets`, bounds, and a `sphere_t` driving capsule mode. Dispatch:
zero-length → position tests (`CM_TestInLeaf`, `CM_TestCapsuleInCapsule`,
`CM_TestBoundingBoxInCapsule`, `CM_PositionTest`); swept →
`CM_TraceThroughLeaf`/capsule variants/`CM_TraceThroughTree`, gated on real
bmodel vs `BOX_MODEL_HANDLE` vs `CAPSULE_MODEL_HANDLE` (:1699-1812).

`CM_TraceThroughBrush` iterates sides calling `CM_PlaneCollision` (:523-600) —
classic Q3 enter/leave-fraction clipping, plane dist adjusted by
`tw->offsets[plane->signbits]`, sets `startsolid`/`allsolid`.
`CM_TraceThroughLeaf` dedupes brushes/patches shared across leaves via a
global `checkcount` stamped on `cbrush_t`/`cPatch_t` (incremented in
`CM_Trace` :1588) — **mutable trace-scoped state living in the clipMap data**,
a key fact for the Rust ownership design (needs `&mut` clipMap or a separate
epoch table per query).

`CM_TraceThroughTerrain` (:703, :802 — two overloads) exists but is
terrain-gated; C++-track, out of a first CM wave.

### 3c. Patch-collide generation (cm_patch.cpp, MP 1809 lines)

`CM_GeneratePatchCollide(width, height, points)` (:1163-1226), called **once
per patch at map load** from cm_load.cpp:534 (`patch->pc = …`), result stored
on `cPatch_t` and `Hunk_Alloc`'d (:1204). **There is no runtime cache** — it's
generate-at-load, own-for-level-lifetime; trace time touches only the baked
`patchCollide_t` (grep: cm_trace.cpp has zero winding references).

Pipeline: control points → `cGrid_t` (:1183-1188) →
`CM_SetGridWrapWidth`/`CM_SubdivideGridColumns` (bezier flattening via
`CM_NeedsSubdivision`/`CM_Subdivide`, :169-217) → `CM_RemoveDegenerateColumns`
(:395-439), transposed (`CM_TransposeGrid` :218-268) and repeated for rows →
`CM_PatchCollideFromGrid` (:983-1162): triangle planes per cell
(`CM_GridPlane`/`CM_EdgePlaneNum`/`CM_FindPlane`), plane interning with
epsilon+sign-flip dedupe (`CM_PlaneEqual`/`CM_FindPlane2`, :440-569), one
`facet_t` per quad with inward borders (`CM_SetBorderInward` :675-754),
validation (`CM_ValidateFacet` :755-806), then bevel synthesis
(`CM_AddFacetBevels` :807-982 — winding clipping to add edge/corner bevel
planes so box sweeps don't tunnel through patch edges), final 1-unit bounds
expansion (:1220-1226).

Trace-time: `CM_TraceThroughPatchCollide` (:1392+),
`CM_TracePointThroughPatchCollide` (:1246-1345), `CM_CheckFacetPlane`
(:1346-1391) — pure plane math over baked data.

### 3d. Capsule vs box variants (MP-only; see §4)

All in MP cm_trace.cpp: `CM_TestCapsuleInCapsule` (:342-410),
`CM_TestBoundingBoxInCapsule` (:411-447), `CM_TraceCapsuleThroughCapsule`
(:1249-1302; decomposes capsules into two sphere caps + cylinder, uses
`CM_TraceThroughSphere` :1058-1136 and `CM_TraceThroughVerticalCylinder`
:1146-1240, quadratic solves with `RADIUS_EPSILON`),
`CM_TraceBoundingBoxThroughCapsule` (:1311-1340 — inverts the query: the box
becomes a swept sphere-mode trace against a temp box model of the capsule's
AABB via `CM_TempBoxModel`, recursing into `CM_TraceThroughLeaf`). cm_test.cpp
has zero capsule code.

### 3e. Winding allocation (cm_polylib.cpp, 713 lines)

Function set: `AllocWinding`, `FreeWinding`, `RemoveColinearPoints`,
`WindingPlane/Area/Bounds/Center`, `BaseWindingForPlane`, `CopyWinding`,
`ReverseWinding`, `ClipWindingEpsilon` (:285-396), `ChopWindingInPlace`
(:397-501), `ChopWinding` (:502-519), `CheckWinding`, `WindingOnPlaneSide`,
`AddWindingToConvexHull`.

Allocation is per-call heap: `AllocWinding` → `Z_Malloc(s, TAG_BSP, qtrue)`
(:42); `FreeWinding` has a `0xdeaddead` double-free sentinel (:47ff).
`ClipWindingEpsilon` allocs a front/back pair per clip (:335-336);
`ChopWindingInPlace` frees + reallocs per chop (:434, :444, :489-509). Sole
CM consumer: `CM_AddFacetBevels` (cm_patch.cpp:817 `BaseWindingForPlane`,
:826 `ChopWindingInPlace` loop; frees :769, :786, :948, :958, :1763). **All
winding churn is load-time-only and function-local** — never on the trace hot
path.

### 3f. Ghoul2/CM boundary

Zero `Ghoul2|G2_|G2API` hits in cm_trace.cpp, cm_test.cpp, cm_patch.cpp —
clean boundary. Ghoul2 collision (`G2API_CollisionDetect`) is a separate
higher-layer subsystem (ghoul2/, C++-track). CM and G2 structurally touch only
via SP's `trace_t.G2CollisionMap` field and the trace_t-last-in-traceWork_t
ordering convention (§3a). MP's `G_G2TRACE` trap reaches `SV_Trace`
(sv_game.cpp:587-595) same as `G_TRACE`; the G2 part happens above CM.

## 4. MP/SP diffs

Beyond A2 §1m (`CM_OrOfAllContentsFlagsInMap` `code/qcommon/cm_load.cpp:50`,
`gsCachedMapDiskImage` :581):

- **cm_load.cpp** (MP 1184 / SP 1298): SP-only `CM_FreeMap`, `CM_HasTerrain`
  (:853), `CM_SameMap` (:838), `CM_LoadShaderText`, `CM_TotalMapContents`
  (:902-905), and savegame portal persistence `CM_WritePortalState` /
  `CM_ReadPortalState` (:1277-1297, `SG_Append('PRTS', …)`). SP same-map
  fast path (§1a). Allocator: MP Hunk vs SP Z_Malloc/TAG_BSP (§1a).
  `cmodel_t.firstNode` exists in MP `cm_local.h:48` only (set at
  cm_load.cpp:144,149); SP's `cmodel_t` (`cm_local.h:42-45`) lacks it.
- **cm_trace.cpp** (MP 1992 / SP 1244, MP +748): the delta is MP's **entire
  capsule subsystem** — 52 case-insensitive `capsule` hits MP vs **0** SP.
  MP-only: `CM_TestCapsuleInCapsule` (:342), `CM_TestBoundingBoxInCapsule`
  (:411), `CM_TraceCapsuleThroughCapsule` (:1249),
  `CM_TraceBoundingBoxThroughCapsule` (:1311), `CM_TraceThroughSphere`
  (:1058), `CM_TraceThroughVerticalCylinder` (:1146),
  `CreateRotationMatrix`/`RotatePoint` (:71/:43) and helpers. SP has only
  classic AABB brush/patch tracing, no `CAPSULE_MODEL_HANDLE` (`cm_local.h`),
  no capsule params (SP cm_public.h:32-38). ioquake3-lineage feature never
  backported to SP.
- **cm_patch.cpp** (MP 1809 / SP 2930, SP +1121): almost entirely **dead Xbox
  code** — 28 `_XBOX` occurrences SP vs 0 MP, duplicating every major function
  (`CM_AddFacetBevels` SP :1028/:1200, `CM_PatchCollideFromGrid`
  :1405/:1628, `CM_GeneratePatchCollide` :1829/:1913,
  `CM_TraceThroughPatchCollide` ×3, etc.) plus SP-only Xbox pool-alloc helpers
  (`CM_GridAlloc`/`CM_PatchCollideFromGridTempAlloc`/… SP ~:427-455,
  :1377-1400, :1800-1828). Port target = the non-`_XBOX` branch; MP is the
  clean single-path reference.
- **cm_test.cpp** (MP 573 / SP 793): SP-only `CM_CleanLeafCache`
  (`code/qcommon/cm_test.cpp:54`) and `CM_SnapPVS(vec3_t, byte*)` (:775) —
  SP leaf-cache/PVS-snapshot helpers.
- **cm_polylib.cpp** (MP 713 / SP 711): effectively identical; port once,
  share or mirror trivially.

## 5. Module seam

**MP server** (trap → `sv_game.cpp` dispatch → wrapper → CM):

| Trap | Handler chain |
|---|---|
| `G_TRACE`/`G_TRACECAPSULE`/`G_G2TRACE` | dispatch `server/sv_game.cpp:587-595` → `SV_Trace` (`sv_world.cpp:803`) → `CM_BoxTrace` world clip (`sv_world.cpp:820`) + `SV_ClipMoveToEntities` → `CM_TransformedBoxTrace` per entity (`sv_world.cpp:496,609`) |
| `G_POINT_CONTENTS` | :596 → `SV_PointContents` (`sv_world.cpp:871`) → `CM_PointContents` (:880) + `CM_TransformedPointContents` per touched entity (:897) |
| `G_ENTITY_CONTACT`(`CAPSULE`) | :583-586 → `SV_EntityContact` (sv_game.cpp:291) → `CM_TransformedBoxTrace` (:301) |
| `G_IN_PVS`/`G_IN_PVS_IGNORE_PORTALS` | :604-607 → `SV_inPVS`/`SV_inPVSIgnorePortals` (sv_game.cpp:209/:243) → `CM_PointLeafnum`→`CM_LeafCluster`→`CM_LeafArea`→`CM_ClusterPVS`→`CM_AreasConnected` (:220-230, :254-261) |
| `G_ADJUST_AREA_PORTAL_STATE` | :624 → `SV_AdjustAreaPortalState` (:275) → `CM_AdjustAreaPortalState` (:282) |
| `G_AREAS_CONNECTED` | :627-628 → `CM_AreasConnected` direct, no wrapper |

**MP client-cgame** (`cl_cgame.cpp` dispatcher calls CM directly, in-process;
trap enums `oracle/codemp/cgame/cg_public.h:83-94`): `CG_CM_LOADMAP` →
`CL_CM_LoadMap` (:583) → `CM_LoadMap`, plus `CM_LoadSubBSP` (:771-778);
`CG_CM_NUMINLINEMODELS` :781-782; `CG_CM_POINTCONTENTS` :789-790;
`CG_CM_TRANSFORMEDPOINTCONTENTS` :791-792; `CG_CM_BOXTRACE`/`CAPSULETRACE` →
`CM_BoxTrace(capsule=qfalse/qtrue)` :793-797;
`CG_CM_TRANSFORMEDBOXTRACE`/`TRANSFORMEDCAPSULETRACE` :799-803;
`CG_CM_MARKFRAGMENTS` → `re.MarkFragments` (renderer, **not** CM) :805-806.

**SP seam** (per DEC-07, statically linked): `SV_InitGameProgs`
(`oracle/code/server/sv_game.cpp:477`) fills a `game_import_t` function
pointer table handed over via `Sys_GetGameAPI` (:669): `import.trace =
SV_Trace` (:507), `import.pointcontents = SV_PointContents` (:508),
`import.totalMapContents = CM_TotalMapContents` (:509, SP-only),
`import.inPVS/inPVSIgnorePortals = SV_inPVS/...` (:512-513),
`import.AdjustAreaPortalState = SV_AdjustAreaPortalState` (:546),
`import.AreasConnected = CM_AreasConnected` (:547 — CM wired directly, no SV
wrapper). Game code calls through `gi.trace(...)` (e.g.
`oracle/code/game/g_vehicles.c:102`). One indirection vs MP's
opcode+VMA marshalling — matches the existing `game_import_t` port at
`crates/sp/abi/src/game/public/game_import_t.rs`.

## 6. TU-harness candidates (DEC-09.1)

Header deps and engine-service surface per TU (MP):

| TU | Includes | Engine calls to stub | Verdict |
|---|---|---|---|
| cm_trace.cpp | `exe_headers.h`, `cm_local.h`, `cm_landscape.h`, `../renderer/tr_local.h` (vestigial — zero real `re.*`/engine calls; apparent hits are `tw->sphere.use` substrings) | none | **ideal** — pure math over `traceWork_t`/`clipMap_t` |
| cm_test.cpp | same | 5 `Com_Error` (`code` side cites :381,:405,:482,:492,:521) | ideal |
| cm_polylib.cpp | light | 12 `Com_Error`, `Z_Malloc`/`Z_Free` (:42, :54) | light stubs |
| cm_patch.cpp | + `cm_patch.h` | 4 `Cvar_Get`, 15 `Com_Printf/Error`, 6 `Hunk_Alloc`, 2 `Z_Malloc/Z_Free` | moderate stubs |
| cm_load.cpp | + `../RMG/RM_Headers.h`, `cm_landscape.h` | 31 `Com_Printf/Error`, 19 `Hunk_Alloc`, 5 `Z_Malloc`, 3 `Cvar_Get`, 4 `FS_*` | heavy — needs FS stub feeding a fixture buffer |

Harness shape (gp2-oracle pattern: unmodified TU + `stubs/` headers +
committed `golden/` dumps):

1. **Patch harness (no BSP needed).** Feed `CM_GeneratePatchCollide` synthetic
   control grids (flat quad, cylinder-wrap, degenerate-column cases); dump the
   `patchCollide_t` (interned planes, facets, borders, bevels) canonically;
   then run `CM_TraceThroughPatchCollide` sweeps over the baked data and dump
   `trace_t`. This exercises cm_patch + cm_polylib + half of cm_trace with
   zero file I/O.
2. **Brush harness.** Hand-construct `clipMap_t` contents in the driver
   (a few axial + bevel-plane brushes, tiny node tree — or reuse
   `CM_InitBoxHull`'s own 6-plane box model, cm_load.cpp:~930ff) and golden
   `CM_BoxTrace`/`CM_TransformedBoxTrace`/`CM_PointContents` across a sweep
   matrix (box/point/capsule × start-solid/graze/tunnel cases).
3. **Load harness (optional, later).** A synthetic minimal `.bsp` built by the
   harness itself (write `dheader_t` + tiny lumps into a buffer, stub
   `FS_ReadFile` to return it) → golden-dump the populated `clipMap_t` +
   `Com_BlockChecksum`.

**Fixture licensing**: no `.bsp`, `.map`, or `.pk3` anywhere in the repo
(verified by find over repo + oracle/; `tools/` holds only `abi/`,
`closure-prototype/`, `gp2-oracle/`). Shipped JKA maps are proprietary and
must not be committed — fixtures must be synthetic (as above) or a
user-supplied local asset outside the repo, like gp2-oracle's hand-authored
fixtures.

**Terrain/RMG flag**: `cm_terrain.cpp` (1720/1714 ln), `cm_terrainmap.cpp`,
`cm_randomterrain.cpp` (1091/1086 ln — `RMG_CreateSeed`
`codemp/qcommon/cm_randomterrain.cpp:1008`) are C++ classes over GP2 parsing
(`"ext_data/RMG/%s.terrain"`, `cm_terrain.cpp:45`) — C++-track per
porting-rules §F, `port-cpp-subsystem` workflow, out of B3's native-track
scope. B3 must only define the *seam*: `clipMap_t.landScape:
CCMLandScape*` and `cbrush_t` terrain gating in `CM_TraceThroughLeaf`
remain `//TODO: Port` markers.

## Design forks

**FORK-1 — clipMap arena shape (instance vs global).** Raven: `cmg` +
`SubBSP[32]` + `NumSubBSP`/`TotalSubModels` as file-scope globals
(cm_load.cpp:37,60-61). Per rules §B3/§B6 and STATE-D2's multi-world corollary
(GameWorld is a value; engine holds *a* seam registration), the natural shape
is one owned `CollisionWorld { main: ClipMap, sub_bsps: Vec<ClipMap> /* cap 32
for parity */, total_sub_models: i32 }` value — **not** a process global —
threaded to SV/CL query paths. Tension to resolve in the doc: (a) server and
client in a listen server share one CM instance in Raven (same process
globals) — who owns `CollisionWorld` in our lifecycle (com-level, shared by
SV+CL borrows, per A3)? (b) the handle math (cumulative offsets, §1c) already
makes handles instance-relative, so multi-instance is structurally free; (c)
`BOX_MODEL_HANDLE`/`CAPSULE_MODEL_HANDLE` + `CM_TempBoxModel`'s box_brush
scratch state must live in the instance too, not statics.

**FORK-2 — checkcount mutation inside queries.** `CM_Trace` bumps a
`checkcount` stamped onto `cbrush_t`/`cPatch_t` to dedupe across leaves
(cm_trace.cpp:1588; also `CM_PointContents` paths in cm_test.cpp). So a
"read-only" trace mutates the clipMap. Options: (a) faithful — queries take
`&mut CollisionWorld` (matches Raven, forbids concurrent traces, fine for
parity phase); (b) epoch table outside the brush/patch structs (`Vec<u32>`
indexed by brush/patch id) owned by a per-query or per-thread scratch —
enables `&self` queries later. Recommend (a) first, (b) as a
behind-green-diff refactor. Note ABI: `cbrush_s`/`c_patch_t` Rust structs
already carry `checkcount` fields for layout parity — dropping the field
later interacts with offset asserts.

**FORK-3 — patch-collide storage/caching.** Resolved by survey: Raven has
**no runtime cache** — `CM_GeneratePatchCollide` runs once per patch at load
(cm_load.cpp:534) into hunk memory. Rust: bake `PatchCollide` at load, own it
in the `ClipMap` (Box/Vec per patch), no invalidation logic. The only real
choice is arena-vs-individual-Box for the ~thousands of facets/planes;
per-map arena matches hunk lifetime semantics exactly.

**FORK-4 — winding allocation.** All winding churn is load-time-local to
`CM_AddFacetBevels` (§3e); trace path never allocates. Faithful per-call
`Vec`/`Box` allocation is acceptable (load-time only); a small scratch
arena/pool scoped to `generate_patch_collide` is a free idiom win with zero
cross-call lifetime risk. Do **not** port the `0xdeaddead` sentinel — Rust
ownership subsumes it (note it as intentionally dropped).

**FORK-5 — capsule support asymmetry.** MP has the full capsule subsystem, SP
none (§4). Options: (a) two faithful trace cores (MP with capsule, SP
without) mirroring the crate split; (b) one shared core with capsule
feature-gated, SP entry points never passing capsule. (b) risks silent
divergence from SP's exact 1244-line control flow; recommend (a) for parity
phase given the files genuinely differ, with cm_polylib (§4) as the one truly
shared piece.

**FORK-6 — CM↔renderer disk-image handoff.** `gpvCachedMapDiskImage` /
`gbUsingCachedMapDataRightNow` (+SP name tag) couple CM and renderer through
raw globals (A2 §1g/§1n). Rust shape: CM load returns/holds an owned
`MapDiskImage` (the raw buffer) that the renderer *takes* (ownership
transfer) or borrows during `RE_LoadWorldMap`, with the low-memory early-free
becoming an explicit drop. Needs a decision on which side owns it between
`CM_LoadMap` and `RE_LoadWorldMap` in the boot sequence (A3 lifecycle).

**FORK-7 — SubBSP fixed cap.** `MAX_SUB_BSP=32` with a hard `Com_Error` on
overflow (cm_load.cpp:1099-1102). Keep the cap + error for parity (mods may
rely on the error, and handle math is stable regardless since offsets are
computed from live counts, not the cap).
