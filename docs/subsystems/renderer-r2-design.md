# Renderer R2 Root-Type Design

Status: FROZEN (user sign-off 2026-07-26)     Supersedes: none
Decision prefix: R2     Ledger deps: DEC-01, DEC-37 (charter + rulings 1-17 +
addenda A1-A12, including the A5 per-registry-failure amendment; A1-A4
ratified 2026-07-25, A5-A9 + the A5 amendment ratified 2026-07-26, A10-A12
(Gate-2 re-review round) ratified 2026-07-26)

> R2 targets DEC-37 ruling 14's root-type sit-down stage. Every decision below
> transcribes a ratified DEC-37 addendum (A1-A4, 2026-07-25; A5-A8, 2026-07-26;
> the A5 amendment and A9, doc-review fix round, 2026-07-26; A10-A12, Gate-2
> re-review round, 2026-07-26) — this doc is the addenda's landing site per
> A8. **Gate record**: Gate 1 (mechanical) and Gate 2 (adversarial review)
> passed across three fix rounds — an initial pass (B1-B13/N1-N12, all
> fixed), a re-review (NB-1-4/NN-1-3, all fixed), and the doc-review/Gate-2
> re-review addenda (A5 amendment, A9-A12) folded in as they ratified. Gate 3
> (dry-run) ran, found 7 BLOCKER-for-R3 gaps, all resolved in-doc against
> fresh oracle reads (no new user ruling needed); a final delta check on
> those resolutions found 2 MEDIUM + 5 LOW findings (F1-F7), all applied.
> **FROZEN by user sign-off 2026-07-26** after a personal read of the full
> doc including the tier-2 transition audit — every mechanical/adversarial/
> dry-run gate this doc's own process defines had already passed. The
> underlying DEC-37 rulings are ratified independently. Signatures freeze
> at the level ruling 14 scopes to R2 (struct/enum shapes, no method
> bodies) — R3 fills the `RenderAssets` registration algorithms, R4 the GPU
> crate internals.

## Standing context

`docs/decisions.md` DEC-37 (charter + rulings 1-17 + addenda A1-A12);
`docs/plans/2026-07-24-client-port/renderer-plan.md` stage R2;
`docs/subsystems/tr-model.md` (FROZEN, the standing design for the
already-ported headless model/skin subset, extended here rather than
restated);
`docs/porting-rules.md` §B (state threaded not reached, entities by handle),
§D12 (ABI-crossing types keep exact Raven layout/field names), §F (C++-track
idiomatic reimplementation — the renderer is a §F subsystem per DEC-37
ruling 1); `docs/architecture/state-ownership.md` (the owned-instance
pattern this doc's `RenderAssets`/`GpuResources`/`RenderWorld` split
instantiates).

## Scope & non-goals

Decides: the owned-renderer-world struct sketch (`RenderAssets` /
`GpuResources` / `RenderWorld` / `FrameState`), the `FrameData` event-stream
shape and its `backEndData_t`/`RC_*` disposition table (A1), the registry
arena count and capacity semantics including per-registry failure values,
slot-0 default-entry reservation, and the name/param lookup-key structures
each registry's oracle lookup actually needs (A2, A5, A5 amendment, A12,
`R2-D4`), the `tr.lightmaps` positional-index field beside the folded-in
image arena (`R2-D4`), the `RenderAssets` mutation/publish path (A9), the
event-append bounds and validation-state placement for scene-composition
traps — which state a trap-time append reads and where that state lives
(`### FrameData`'s append-validation principle, `R2-D9`), the renderer's
oracle-fatal error-path shape onto the
engine's existing `ComError` machinery (`R2-D11`), light-style state
classification, ownership, entry points, and the `RenderScene` snapshot
carrier's producer *and* consumer landing (A6, A9, A11, `R2-D5`), the
automap init classification as a sim-side A9 mutation (A10), whether
`trRefEntity_t`/`trRefdef_t` compose qshared seam types by value (A3, as
amended: `trRefEntity_t` only), and the SP/MP mode-divergence adapter shape
(ruling 17).

Punts, each with an owner and timing fixed by A7:

- Pipeline-key field enumeration beyond ruling 6's list, shader-backend WGSL
  details (ruling 5) — backend work, R4.
- The `RenderAssets` registration algorithms (method bodies) — R3 frontend
  waves.
- GPU crate internals — R4.
- `FrameData` buffer-recycling mechanics (double/triple free-list vs. fixed
  pool) — R4; the lean default (fixed 2-3 buffer pool + explicit return
  channel) is recorded now (`R2-D8`), but only the event-stream shape
  freezes at R2.
- `RC_AUTO_MAP`'s full oracle command struct — targeted read at the first
  automap wave (R3/R4).
- `subImageCommand_t` dead-vs-reachable status — grep `RE_SubImage`/
  `subImageCommand_t` call sites before R3 scope-freezes.

None of the sketches below have method bodies — signatures only, per DEC-37
ruling 14's R2 scope.

## Raven ground truth

### Part 1 findings (DEC-37 A4 housekeeping)

Both DEC-37 A4 housekeeping reads are done:

- **Image registry backing store**: NOT a `MAX_DRAWIMAGES`-bounded array.
  The real store is a `typedef map<LPCSTR, image_t*, CStringComparator>
  AllocatedImages_t` with a file-scope `AllocatedImages_t AllocatedImages`
  object (`oracle/codemp/renderer/tr_image.cpp:527-535`), walked by
  `R_Images_StartIteration`/`GetNextIteration` via a saved `std::map`
  iterator (`oracle/codemp/renderer/tr_image.cpp:541-556`). The
  `hashTable[FILE_HASH_SIZE]` declaration
  (`oracle/codemp/renderer/tr_image.cpp:41-42`) is dead/commented code —
  `FILE_HASH_SIZE` itself is still used by the *shader* hash table
  elsewhere (`oracle/codemp/renderer/tr_shader.cpp:105-106`), not here. The
  `MAX_DRAWIMAGES` overflow check is also dead/commented
  (`oracle/codemp/renderer/tr_image.cpp:1137-1140`) — retail enforces no
  hard image-count cap at runtime. Individual `image_t` instances are
  `Z_Malloc`'d (`TAG_IMAGE_T`) in `R_CreateImage`
  (`oracle/codemp/renderer/tr_image.cpp:1236`) and keyed by a lower-cased,
  extension-stripped name (`GenerateImageMappingName`, called at
  `oracle/codemp/renderer/tr_image.cpp:1287-1289`) — the concrete mechanism
  behind ruling 11's "lower-cased extension-stripped image cache key" and
  "mismatched-params cache hit only warns" (the map is looked up by name
  only in `R_FindImageFile_NoLoad`, so two calls with the same key and
  different params return the same `image_t`, generating a warning rather
  than a new entry). Consequence: the image arena cannot mirror
  shader/skin/model's fixed-`MAX_*` shape at the storage level — it is the
  one of the four registries whose oracle backing store is already
  unbounded and ordered by string key, not index (settled by A5, `R2-D4`).
- **`crates/mp/renderer/src/tr_local/mnode_s.rs` spot-check verdict:
  CORRECT.** It transcribes the non-XBOX merged node+leaf branch
  (`oracle/codemp/renderer/tr_local.h:917-934`), not the XBOX split branch
  (`oracle/codemp/renderer/tr_local.h:886-913`). Field-by-field:

  | Rust field | Raven field | Raven type | Rust type | Match |
  |---|---|---|---|---|
  | `contents` | `contents` | `int` | `c_int` | yes |
  | `visframe` | `visframe` | `int` | `c_int` | yes |
  | `mins` | `mins` | `vec3_t` | `vec3_t` | yes |
  | `maxs` | `maxs` | `vec3_t` | `vec3_t` | yes |
  | `parent` | `parent` | `struct mnode_s *` | `*mut mnode_t` | yes |
  | `plane` | `plane` | `cplane_t *` | `*mut cplane_t` | yes |
  | `children` | `children[2]` | `struct mnode_s *[2]` | `[*mut mnode_t; 2]` | yes |
  | `cluster` | `cluster` | `int` | `c_int` | yes |
  | `area` | `area` | `int` | `c_int` | yes |
  | `firstmarksurface` | `firstmarksurface` | `msurface_t **` | `*mut *mut msurface_t` | yes |
  | `nummarksurfaces` | `nummarksurfaces` | `int` | `c_int` | yes |

  10/10 fields match the non-XBOX branch name-for-name and type-for-type, in
  the same order; none of the XBOX branch's divergent types (`signed char
  contents`, `short mins[3]`, `unsigned int planeNum` instead of a `plane`
  pointer, `short cluster`, `unsigned short firstMarkSurfNum`) leaked in. No
  fix needed.
- **SP `stereoFrame_t` spot-check (landing-time verification, this doc):
  CORRECT, both modes.** SP's `oracle/code/renderer/tr_types.h:179-183` is a
  true `typedef enum { STEREO_CENTER, STEREO_LEFT, STEREO_RIGHT }
  stereoFrame_t;` —
  `crates/sp/qshared/src/common/sp/renderer/stereo_frame_t.rs` ports it as a
  `#[repr(i32)] enum` with a `size_of == 4` assert, matching the CLAUDE.md
  enum-vs-alias rule. MP's `oracle/codemp/cgame/tr_types.h:278-283` is an
  anonymous enum (278-282) plus `typedef int stereoFrame_t;` (283) —
  `crates/mp/qshared/src/common/mp/cgame/stereo_frame_t.rs` ports it as a
  `c_int` alias + consts, also matching the rule. Both sides already
  correctly forked; no fix needed to the Rust ports. The SP Rust file's own
  doc-comment cited `tr_types.h:183-187` (wrong); corrected in the same
  commit as this doc to `179-183`.

### Root-type vocabulary

The frozen-vocabulary source for `## Seam definition`'s owned-world split
and disposition table:

- `trGlobals_t` (`oracle/codemp/renderer/tr_local.h:1309-1423`) — the single
  `extern trGlobals_t tr;` global. Holds both **registries**
  (`models[MAX_MOD_KNOWN=1024]`, `shaders[MAX_SHADERS=16384]`/
  `sortedShaders[MAX_SHADERS]`, `skins[MAX_SKINS=1024]`,
  `lightmaps[MAX_LIGHTMAPS=256]` — `image_t*`, folded into `RenderAssets
  ::images` rather than a fifth arena, `:1364` — and
  `bspModels[MAX_SUB_BSP=32]` — sub-BSP worlds, homed at `RenderAssets
  ::bsp_models`, `:1399`) and **frontend scratch/singleton** fields
  (`viewParms`, `refdef`, `ori`, `pc` counters, function tables
  (`sinTable`…`fogTable`, `:1412-1417`), sun/fog fields, `landScape`,
  `distanceCull`/`distanceCullSquared` (`:1420`)).
- `backEndState_t` (`oracle/codemp/renderer/tr_local.h:1279-1292`) —
  `extern backEndState_t backEnd;`. Not gutted in MP: 11 fields — `refdef`,
  `viewParms`, `ori`, `pc`, `isHyperspace`, `currentEntity`,
  `skyRenderedThisView`, `projection2D`, `color2D`, `vertexes2D`,
  `entity2D`.
- `backEndData_t` (`oracle/codemp/renderer/tr_local.h:2263-2273`) —
  `extern backEndData_t *backEndData;`, heap-allocated, explicitly commented
  "duplicated so front/back end can run in parallel on SMP"
  (`:2261-2262`). Fields: `drawSurfs[MAX_DRAWSURFS=0x10000]` (`:2264`),
  `dlights[MAX_DLIGHTS=32]` (`:2266`), `entities[MAX_ENTITIES=2048]`
  (`:2268`), `miniEntities[MAX_MINI_ENTITIES=1024]` (`:2269`), unsized
  `polys`/`polyVerts` pointers (`:2270-2271`), `commands:
  renderCommandList_t` (`:2272`).
- `renderCommandList_t`/`renderCommand_t`
  (`oracle/codemp/renderer/tr_local.h:2180-2250`) — a byte-packed command
  buffer (`cmds[MAX_RENDER_COMMANDS=0x40000]`) the frontend writes and the
  backend walks; `RC_*` tags at `:2240-2249`: `RC_END_OF_LIST` (2240),
  `RC_SET_COLOR` (2241), `RC_STRETCH_PIC` (2242), `RC_ROTATE_PIC`/
  `RC_ROTATE_PIC2` (2243-2244), `RC_DRAW_SURFS` (2245), `RC_DRAW_BUFFER`
  (2246), `RC_SWAP_BUFFERS` (2247), `RC_WORLD_EFFECTS` (2248),
  `RC_AUTO_MAP` (2249).
- `glstate_t` (`oracle/codemp/renderer/tr_local.h:1253-1260`) — `extern
  glstate_t glState;`, explicitly "outside of TR since it shouldn't be
  cleared during ref re-init"; GL binding cache (`currenttextures`,
  `currenttmu`, `texEnv`, `faceCulling`, `glStateBits`).
- `glConfig` (`extern glconfig_t glConfig;`,
  `oracle/codemp/renderer/tr_local.h:1435`) and `styleColors[MAX_LIGHT_STYLES]`
  (`extern color4ub_t styleColors[MAX_LIGHT_STYLES];`,
  `oracle/codemp/renderer/tr_local.h:1888`) — both declared "outside of TR"
  alongside `glState`, but unlike it both are read from a sim-thread trap
  (below), so ruling 3 places them in `RenderAssets`, not render-thread-only
  state (B11 finding).
- The 57 `CG_R_*`/`UI_R_*` trap `Args` types under
  `crates/mp/abi/src/{cgame,ui}/syscalls/`, and their C-side dispatch at
  `oracle/codemp/client/cl_cgame.cpp:644-1720` (table-routed calls through
  `re.<Method>`, plus **eight** cases that bypass the `re` table entirely —
  the DEC-37 ruling 4 instance, listing every one closes it explicitly):

  | Trap | Cite | Disposition |
  |---|---|---|
  | `CG_R_SETRANGEFOG` | `oracle/codemp/client/cl_cgame.cpp:943-945` | writes `tr.rangedFog`; crosses as `FrameEvent::SetRangeFog` |
  | `CG_R_SETREFRACTIONPROP` | `oracle/codemp/client/cl_cgame.cpp:947-952` | writes `tr_distortionAlpha`/`Stretch`/`PrePost`/`Negate` (externs at `cl_cgame.cpp:638-641`); crosses as `FrameEvent::SetRefractionProp` |
  | `CG_R_GETDISTANCECULL` | `oracle/codemp/client/cl_cgame.cpp:1058-1064` | reads `tr.distanceCull`; synchronous `RenderAssets::distance_cull` read (B11) |
  | `CG_R_GETREALRES` | `oracle/codemp/client/cl_cgame.cpp:1066-1073` | reads `glConfig.vidWidth`/`vidHeight`; synchronous `RenderAssets::glconfig` read (B11) |
  | `CG_R_AUTOMAPELEVADJ` | `oracle/codemp/client/cl_cgame.cpp:1075-1077` | crosses as `FrameEvent::AutomapElevAdj` |
  | `CG_R_INITWIREFRAMEAUTO` | `oracle/codemp/client/cl_cgame.cpp:1079-1080` | sim-side A9 mutation — rebuilds `RenderAssets::automap_wireframe` (`oracle/codemp/renderer/tr_world.cpp:1205-1231`, A10), not a read |
  | `CG_R_WEATHER_CONTENTS_OVERRIDE` | `oracle/codemp/client/cl_cgame.cpp:1716-1718` | retail body is `//contentOverride = args[1]; return 0;` — a live no-op; not a `FrameEvent` (B9) |
  | `CG_R_WORLDEFFECTCOMMAND` | `oracle/codemp/client/cl_cgame.cpp:1720-1722` | crosses as `FrameEvent::WorldEffectCommand` |

- Light styles — three names for one mechanism, reconciled here since A6's
  own shorthand ("R_Set/GetLightStyle") is not a literal oracle identifier:
  the renderer-side functions are `RE_GetLightStyle`/`RE_SetLightStyle`
  (`oracle/codemp/renderer/tr_init.cpp:1427-1450`), the cgame trap wrappers
  are `trap_R_GetLightStyle`/`trap_R_SetLightStyle`
  (`oracle/codemp/cgame/cg_syscalls.c:411,416`, declared
  `oracle/codemp/cgame/cg_local.h:2296-2297`), and the VM syscall IDs are
  `CG_R_SET_LIGHT_STYLE`/`CG_R_GET_LIGHT_STYLE`. All three mutate/read the
  same backing store, `styleColors[MAX_LIGHT_STYLES]`
  (`extern` at `oracle/codemp/renderer/tr_local.h:1888`, defined
  `oracle/codemp/renderer/tr_shade.cpp:26`). Style colors are written from
  `trap_R_SetLightStyle` call sites (cgame's per-frame light-style update,
  `oracle/codemp/cgame/cg_light.c:64`) and read back synchronously at the
  same table by the render-side consumers
  (`oracle/codemp/renderer/tr_surface.cpp:279,324`,
  `oracle/codemp/renderer/tr_shade.cpp:1401,1685`,
  `oracle/codemp/renderer/tr_light.cpp:234-274`); classified by A6/A9/A11
  (`R2-D5`/`R2-D9`).

## State ownership

| Raven global | Oracle cite | Rust owner | Constructed by | Threaded via |
|---|---|---|---|---|
| `tr` (`trGlobals_t`) registries | `oracle/codemp/renderer/tr_local.h:1309-1423,1434` | `RenderAssets` fields (`images`, `shaders`, `skins`, `models`, `bsp_models`, `world`) | sim-thread init / level load | mutated in place via `Arc::make_mut(&mut RenderAssetsSim::published)`, visible at the next frame boundary (A9, NB-1) |
| `tr` (`trGlobals_t`) frontend scratch/counters | `oracle/codemp/renderer/tr_local.h:1309-1423,1434` | `RenderWorld::frame: FrameState` | render-thread init | owned field, render-thread-only |
| `tr.distanceCull`/`distanceCullSquared` | `oracle/codemp/renderer/tr_local.h:1420` | `RenderAssets::distance_cull`/`distance_cull_squared` (sim-readable — `CG_R_GETDISTANCECULL` reads `distanceCull` synchronously, B11) | render-thread init / RMG load | field on `Arc<RenderAssets>`, republished via A9 |
| `glConfig` (`glconfig_t`) | `oracle/codemp/renderer/tr_local.h:1435` | `RenderAssets::glconfig` (sim-readable — `CG_R_GETREALRES` reads `vidWidth`/`vidHeight` synchronously, B11) | render-thread init (mirrors `CL_InitRef`) | field on `Arc<RenderAssets>`, republished via A9 |
| `tr.registered` | `oracle/codemp/renderer/tr_local.h:1310` | `RenderAssets::registered` (sim-readable — every `RE_Add*ToScene` reads it first as an append guard, e.g. `RE_AddRefEntityToScene`, `oracle/codemp/renderer/tr_scene.cpp:195-197`) | render-thread init | field on `Arc<RenderAssets>`, republished via A9 at registration begin/end |
| `styleColors[MAX_LIGHT_STYLES]` | `oracle/codemp/renderer/tr_local.h:1888` | `LightStyleTable::colors` — sim-owned, **A6-adjacent to `RenderAssets`, not inside its `Arc`** (A6/A9) | sim-thread init | owned field, mutated in place at trap time (no COW — A9) |
| `tr.lightmaps[MAX_LIGHTMAPS]` | `oracle/codemp/renderer/tr_local.h:1364` | split: storage folds into `RenderAssets::images` (lightmaps are `image_t*`, not a fifth registry); the positional index `R_FindShader` reads by small integer is `RenderAssets::lightmaps: Vec<ImageHandle>` (`R2-D4`) | level load | field on `Arc<RenderAssets>`, republished via A9 |
| `r_numentities`/`r_numdlights`/`r_numpolys`/`r_numpolyverts` | `oracle/codemp/renderer/tr_scene.cpp:21-33` (file-scope statics, not `trGlobals_t` fields) | **derived, no dedicated field** — properties of the `FrameData` currently under construction (`### FrameData`'s append-validation principle); never `FrameState` | reset by `R_ToggleSmpFrame` at frame start (`tr_scene.cpp:44-65`) | scoped to whichever thread builds this frame's `FrameData` |
| `backEnd` (`backEndState_t`) | `oracle/codemp/renderer/tr_local.h:1279-1292,1433` | `RenderWorld::frame: FrameState` (all 11 fields — B5) | render-thread init | owned field, render-thread-only |
| `backEndData` (`backEndData_t`) | `oracle/codemp/renderer/tr_local.h:2261-2273,2278` | **dissolved** — see `## Seam definition`'s A1 disposition table; its field list is the reference vocabulary for `FrameData`'s event payloads, not a struct that survives | n/a | n/a |
| `glState` (`glstate_t`) | `oracle/codemp/renderer/tr_local.h:1253-1260,1436` | `GpuResources::gl_state` (a named placeholder — the GL binding cache has no meaning under wgpu — until R4 defines the real pipeline/bind-group cache) | render-thread init | owned field, render-thread-only |
| `tess` (`shaderCommands_t`) | `oracle/codemp/renderer/tr_local.h:1844-1887` | dissolved into R4's tessellation/vertex-building pipeline (frontend produces geometry batches, no single global scratch buffer under the new topology) | n/a (R4 concern) | n/a |
| `tr_distortionAlpha/Stretch/PrePost/Negate` | `oracle/codemp/client/cl_cgame.cpp:638-641` | payload of `FrameEvent::SetRefractionProp` | n/a | crosses in `FrameData` (ordered event) |
| `g_autoMapFrame`/`g_autoMapValid` | `oracle/codemp/renderer/tr_world.cpp:782,784` | `RenderAssets::automap_wireframe` — sim-side A9 mutation (A10), not a synchronous read | rebuilt by `RenderAssetsSim::rebuild_automap_wireframe` at `CG_R_INITWIREFRAMEAUTO` | field on `Arc<RenderAssets>`, republished via A9 |
| `g_playerHeight` | `oracle/codemp/renderer/tr_world.cpp:398` | payload of `FrameEvent::AutomapElevAdj` (unaffected by A10 — `CG_R_AUTOMAPELEVADJ` needs no synchronous answer) | n/a | crosses in `FrameData` (ordered event) |
| `re` (`refexport_t re;`) | declared `oracle/codemp/client/cl_main.cpp:111`, filled by `CL_InitRef` (`:2480-2495`) | **deleted** (DEC-37 ruling 4) | n/a | direct calls / trait, no table |

### Type tiers and the interior-safety law

The renderer's Rust type surface has three tiers with different lifetimes,
and only one keeps raw C shapes permanently:

- **Tier 1 — the frozen ABI seam set** (`refEntity_t` with its
  `*mut c_void` ghoul2 tail, `refdef_t`, `polyVert_t`, `glconfig_t` — the
  `mp_qshared` Class-B types the cgame/ui traps carry). These stay
  `#[repr(C)]` with Raven field names and layout asserts forever;
  `unsafe` at their crossings is confined per porting-rules §D11/§D12.
- **Tier 2 — the pre-DEC-37 type-pass files under
  `crates/mp/renderer/src/tr_local/`** (raw pointer fields such as
  `mnode_s`'s `cplane_t*`/parent/children links, `shader_t`'s stage
  pointers, `image_t`-family pointer graphs, `char`-array names). These
  are **transitional scaffolding** from the types-only campaign that
  predates this design. They are not the R3 target shape: as each
  subsystem's logic lands, R3 replaces their pointer fields with the
  owned forms this document assigns — world node/leaf links become
  index-linked owned structures when `tr_bsp`/`tr_world` land; bundle
  and registry pointers become `Handle`s (`Vec<ImageHandle>` for the
  animMap family, arena handles for shader/skin/model references);
  `char`-array names become `String`s under the established Latin-1
  discipline. A tier-2 file survives only until the wave that ports its
  owning subsystem; the per-type mapping and change-safety validation is
  the `### Tier-2 transition audit` below.
- **Tier 3 — this document's architecture types** (`Arena<T>`,
  `Handle<K>`, `RenderAssets`, `RenderAssetsSim`, `FrameData`/`FrameEvent`,
  `FrameState`, `LightStyleTable`) — fully owned, zero raw pointers.

**Law (binding on every R3/R4 wave and reviewer): no new interior type may
adopt raw pointers, `c_char` buffers, or `qboolean`-style ints — handles,
indices, owned `String`/`Vec`, and `bool` only.** `#[repr(C)]` and raw
pointers are permitted solely in tier 1. A transcriber who needs to
reference another asset stores its `Handle`; one who needs a name stores a
`String`. Tier-2 fields may be *read* through their existing shapes until
their owning wave replaces them, but no new field or type may extend the
tier-2 pattern.

### Tier-2 transition audit

The survey named above: every `crates/mp/renderer/src/{tr_local,tr_public,
mdx_format,tr_model}` type carrying a raw-pointer field, a `c_char`
array/pointer, or a `qboolean`/C-int boolean (`mp_qshared::shared::qboolean =
c_int`, per §C7/the interior-safety law). 39 types qualify — 34 of
`tr_local`'s 73, plus `tr_public::refexport_t` and 4 `mdx_format` on-disk
headers touched by the live `tr_model` loader. Every row's **proposed
replacement is subordinate to this document's own rulings** (the `##
Decisions`/`## Seam definition`/`## State ownership` sections above) — a row
never invents a shape those sections don't already license, and is
re-validated by whichever wave actually executes it, not frozen here. Rows
marked **coupled** require a coordinated slice with the named consumer, not a
unilateral R3 rewrite; rows marked **free** have zero consumer evidence
outside the renderer's own future port as of this sweep (grep summarized per
row) and may move on the R3 wave's own schedule. Every `#[repr(C)]` row also
carries `size_of`/`offset_of!` asserts against clang ground truth; unless a
row says otherwise, those **retire with the type** at its owning wave and are
not restated per row.

**Verified against the prompted claim** ("8 of 73 `tr_local` types have any
consumer outside `tr_local` itself"): this sweep found **2**, not 8 —
`model_t`/`model_s` (`model_s.rs`, real consumer: `tr_model/`) and
`shader_t`/`shader_s` (`shader_s.rs`, real consumer: `tr_landscape/`). Several
types that look coupled on a plain grep (`skin_t`, `skinSurface_t`,
`mdxaHeader_t`, `mdxmHeader_t` from outside the renderer crate) turn out to be
**doc-comment-only** mentions once the hit is read in context: the live
`tr_model` skin path already reshaped away from `skin_t`/`skinSurface_t` into
idiomatic `ServerSkin`/`ServerSkinSurface` (see row below), and
`mp_host_interface`/`mp_engine_ghoul2` deliberately never import
`mp_renderer`'s mdx structs (DEC-35 parse-once sidecar re-derives its own
byte-offset views — `crates/mp/engine/ghoul2/src/api_ragdoll.rs:435` states
the non-dependency outright). This discrepancy is **not resolved by this
audit** — flagged for the user rather than silently reconciled.

**Group 1 — World/BSP surface geometry (`tr_bsp`/`tr_world`, `tr_world.cpp`)**

| Type (file) | Raw fields | Owning subsystem / replacing wave | Proposed replacement shape | Can change? (validation) |
|---|---|---|---|---|
| `mnode_t` (`tr_local/mnode_s.rs`) | `parent: *mut mnode_t`, `plane: *mut cplane_t`, `children: [*mut mnode_t; 2]`, `firstmarksurface: *mut *mut msurface_t` | `tr_bsp`/`tr_world` (`oracle/codemp/renderer/tr_local.h:917-934`) | Index-linked node arena: `parent`/`children` become `Option<Handle<Node>>`, `plane` an index into a shared plane pool, `firstmarksurface`/`nummarksurfaces` a `(start, len)` range into a flat mark-surface index `Vec<u32>` (§B5 arena+id pattern) | Free — grep: only `crates/{mp,sp}/renderer/src/tr_local/{world_t,mnode_s}.rs` reference `mnode_t`/`mnode_s`; no consumer anywhere else |
| `msurface_t` (`tr_local/msurface_s.rs`) | `shader: *mut shader_s`, `data: *mut surfaceType_t` | `tr_bsp`/`tr_world` (`oracle/codemp/renderer/tr_local.h:872-878`) | `shader` → `Handle<Shader>`; `data`'s tagged-union pointer → an owned `Surface` enum (or `Handle` into a per-kind surface arena) replacing the `surfaceType_t` dispatch pointer | Free — only `bmodel_t`/`mnode_s`/`world_t`/`msurface_s` (same dir) reference it |
| `bmodel_t` (`tr_local/bmodel_t.rs`) | `firstSurface: *mut msurface_t` | `tr_bsp`/`tr_world` (`oracle/codemp/renderer/tr_local.h:938-942`) | `(start, len)` range into `world_t`'s owned flat `Vec<Surface>` | Free — only `tr_local/{model_s,world_t}.rs` reference it; not touched by `tr_model` (`server_load.rs` never assigns `.bmodel`, only `.mdxa`/`.mdxm`/`.md3`) |
| `world_t` (`tr_local/world_t.rs`) | `name`/`baseName: [c_char; 64]`, `shaders: *mut dshader_t`, `bmodels: *mut bmodel_t`, `planes: *mut cplane_t`, `nodes: *mut mnode_t`, `surfaces: *mut msurface_t`, `marksurfaces: *mut *mut msurface_t`, `fogs: *mut fog_t`, `lightGridData: *mut mgrid_t`, `lightGridArray: *mut c_ushort`, `vis: *const c_uchar`, `novis: *mut c_uchar`, `entityString`/`entityParsePoint: *mut c_char` | `tr_bsp`/`tr_world` (`oracle/codemp/renderer/tr_local.h:1039-1090`) | `String` ×2 for the names; owned `Vec<T>` for every array (`Vec<DShader>`, `Vec<BModel>`, `Vec<CPlane>`, `Vec<Node>`, `Vec<Surface>`, `Vec<u32>` mark-index table, `Vec<Fog>`, `Vec<LightGridSample>`); `vis`/`novis` → `Vec<u8>`; `entityString` → owned `String`, `entityParsePoint` → a `usize` cursor into it | Free — only `tr_local/{mod,tr_globals_t}.rs` (self) reference it |
| `fog_t` (`tr_local/fog_t.rs`) | `hasSurface: qboolean` | `tr_bsp`/`tr_world` (`oracle/codemp/renderer/tr_local.h:616-627`) | `bool` | Free — only `tr_local/{mod,world_t}.rs` |
| `srfGridMesh_t` (`tr_local/srf_grid_mesh_s.rs`) | `widthLodError`/`heightLodError: *mut f32`, `verts: [drawVert_t; 1]` (C flexible-array idiom) | `tr_bsp` bezier-patch tessellation (`oracle/codemp/renderer/tr_local.h:750-774`) | `widthLodError`/`heightLodError` → owned `Vec<f32>`; `verts` → owned `Vec<drawVert_t>` sized by `(width, height)`, replacing the variable-length trailing-array trick | Free — only its own file, `tr_local/mod.rs` |
| `srfTerrain_t` (`tr_local/srf_terrain_s.rs`) | `landscape: *mut CTRLandScape` | `tr_bsp`/RMG terrain link (`oracle/codemp/renderer/tr_local.h:744-748`) | `Handle<CTRLandScape>` (or the arena `tr_landscape`'s own R3/R4 wave settles on) once `tr_landscape` lands | Free — only `tr_local/tr_globals_t.rs`; `crates/mp/engine/rmg` (the live jampded RMG/terrain subsystem) is a **different, already-Rust-native** terrain implementation and does not reference this renderer-side type |
| `srfTriangles_t` (`tr_local/srf_triangles_t.rs`) | `indexes: *mut i32`, `verts: *mut drawVert_t` | `tr_bsp` (BSP/MD3 triangle soup, `oracle/codemp/renderer/tr_local.h:818-836`) | `Vec<i32>`, `Vec<drawVert_t>` | Free — only its own file |

**Group 2 — Shader/stage/bundle/image (`tr_shader`, `tr_shader.cpp`; `tr_image`, `tr_image.cpp`)**

| Type (file) | Raw fields | Owning subsystem / replacing wave | Proposed replacement shape | Can change? (validation) |
|---|---|---|---|---|
| `shader_t` (`tr_local/shader_s.rs`) | `name: [c_char; 64]`, `sky: *mut skyParms_t`, `fogParms: *mut fogParms_t`, `deforms: [*mut deformStage_t; 3]`, `stages: *mut shaderStage_t`, `remappedShader`/`next: *mut shader_t` | `tr_shader` (`oracle/codemp/renderer/tr_local.h:459-530`) | `String` name; `Option<SkyParms>`/`Option<FogParms>` owned inline; `Vec<DeformStage>`; `Vec<ShaderStage>`; `remappedShader` → `Handle<Shader>`; `next` **dissolves** — an intrusive registry-chain link the owning `Vec<Shader>`'s own iteration replaces, no field needed | **Coordinate (soft)**: real consumer `tr_landscape/{ctrland_scape,spatch_info,ctrpatch}.rs` (`mShader`/`mWaterShader`/`mTLShader`/`mBRShader` fields) — dormant future client-rendering code in the same crate; jampded is not live there, so its wave must land in the same slice but does not gate the change. `tr_model/server_skin_surface.rs` mentions `shader_s` only in a doc comment — the live path already reshaped to `ServerSkin`'s `server_shaders: Vec<String>` pool, no real dependency |
| `shaderStage_t` (`tr_local/shader_stage_t.rs`) | `bundle: [textureBundle_t; 2]` (nested pointers, see below), `ss: *mut surfaceSprite_t` | `tr_shader` (`oracle/codemp/renderer/tr_local.h:394-427`) | `ss` → `Option<Box<SurfaceSprite>>` (small, no arena needed) | Free — only `shader_s.rs`/`shader_commands_s.rs`; `tr_model/server_load.rs` imports only the `SHADER_MAX_INDEXES` const, not this type |
| `textureBundle_t` (`tr_local/texture_bundle_t.rs`) | `image: *mut image_t`, `tcGenVectors: *mut vec3_t`, `texMods: *mut texModInfo_t` | `tr_shader`/`tr_image` (`oracle/codemp/renderer/tr_local.h:372-389`) | `image` → `Handle<Image>`; `tcGenVectors` → owned `[vec3_t; 2]` (Raven's pointer addresses a fixed 2-element array); `texMods` → `Vec<TexModInfo>` | Free — only `shader_stage_t.rs` |
| `skyParms_t` (`tr_local/sky_parms_t.rs`) | `outerbox: [*mut image_t; 6]` | `tr_shader` (`oracle/codemp/renderer/tr_local.h:449-452`) | `[Handle<Image>; 6]` (or `Option<Handle<Image>>` ×6 if unset boxes are legal) | Free — only `shader_s.rs` |
| `shaderState_t` (`tr_local/shader_state_s.rs`) | `shaderName`/`name`/`stateShader: [c_char; *]`, `shader: *mut shader_s` | `tr_shader` (`RE_SetActiveShaderName`, `oracle/codemp/renderer/tr_local.h:532-538`) | `String` ×3, `Handle<Shader>` | Free — only its own file |
| `image_t` (`tr_local/image_s.rs`) | `imgName: [c_char; 64]` | `tr_image` (`oracle/codemp/renderer/tr_local.h:136-151`) | `String` | Free — the one non-renderer hit (`mp_qshared::common::mp::qcommon::tags.rs`) is a `TAG_IMAGE_T` enum-variant **comment**, not a type reference |
| `skin_t` (`tr_local/skin_s.rs`) | `name: [u8; 64]` (Latin-1 name buffer, `c_char`-equivalent), `surfaces: [*mut skinSurface_t; 128]` | `tr_image` (skin registration, `oracle/codemp/renderer/tr_local.h:609-613`) | `String` name, `Vec<SkinSurface>` | Free — `tr_model/{server_skin,server_skins}.rs` mention `skin_t` only in doc comments; the live server-skins slice (user ruling 2026-07-12) already reshaped to the idiomatic `ServerSkin { name: String, surfaces: Vec<ServerSkinSurface> }`, which never touches this struct |
| `skinSurface_t` (`tr_local/skin_surface_t.rs`) | `name: [c_char; 64]`, `shader: *mut shader_s` | `tr_image` (`oracle/codemp/renderer/tr_local.h:604-607`) | `String`, `Handle<Shader>` | Free — same finding as `skin_t`: `tr_model/server_skin_surface.rs`'s `ServerSkinSurface` cites it only in a doc comment, already reshaped to `{ name: String, shader: usize }` |
| `hitMatReg_t` (`tr_local/hit_mat_reg_t.rs`) | `loc: *mut u8`, `name: [c_char; 64]` | Ghoul2 hit-material registry (`oracle/codemp/renderer/tr_local.h:544-550`; loaded by `oracle/codemp/ghoul2/G2_misc.cpp`, not yet ported — `crates/mp/engine/ghoul2` has no `hitMatReg`/`HitMaterial` reference today) | `Vec<u8>` owned blob, `String` name | Free — zero consumers anywhere in the repo outside its own file; dormant until the ghoul2 hit-material load path is ported |

**Group 3 — Scene/backend per-frame carriers (`tr_scene`, `tr_scene.cpp`; `tr_backend`/`tr_shade`, `tr_backend.cpp`/`tr_shade.cpp`) — mostly A1 territory, see `## Seam definition`'s A1 disposition table**

| Type (file) | Raw fields | Owning subsystem / replacing wave | Proposed replacement shape | Can change? (validation) |
|---|---|---|---|---|
| `drawSurf_t` (`tr_local/draw_surf_s.rs`) | `surface: *mut surfaceType_t` | `tr_backend` cull/sort output (`oracle/codemp/renderer/tr_local.h:680-683`) | Per A1 ("stays render-side"): `surface` → a `Handle`/index into the surface arena (world or model) rather than a raw tagged pointer | Free — only its own file plus `back_end_data_t.rs`/`draw_surfs_command_t.rs`/`tr_refdef_t.rs` |
| `drawSurfsCommand_t` (`tr_local/draw_surfs_command_t.rs`) | `refdef: trRefdef_t` (embeds pointers, see `trRefdef_t` row), `viewParms: viewParms_t`, `drawSurfs: *mut drawSurf_t` | `tr_backend` `RC_DRAW_SURFS` (`oracle/codemp/renderer/tr_local.h:2231-2237`) | Per A1: the command struct **dissolves** — `refdef`/`viewParms` inputs cross as `FrameEvent::RenderScene` (A11 folds in the light-style snapshot), `drawSurfs` stays a render-thread-local `Vec<DrawSurf>` computed from that event, never a channel payload | Free — only its own file |
| `backEndState_t` (`tr_local/back_end_state_t.rs`) | `currentEntity: *mut trRefEntity_t`, `isHyperspace`/`skyRenderedThisView`/`projection2D`/`vertexes2D: qboolean` | `tr_backend` (`oracle/codemp/renderer/tr_local.h:1279-1292`) | Per `## State ownership`: → `RenderWorld::frame: FrameState` (owned, render-thread-only); `currentEntity` → `Option<Handle<RefEntity>>` (or an owned copy) into the frame's entity list; the four `qboolean`s → `bool` | Free — only its own file |
| `backEndData_t` (`tr_local/back_end_data_t.rs`) | `polys: *mut srfPoly_t`, `polyVerts: *mut polyVert_t` | `tr_scene` (`oracle/codemp/renderer/tr_local.h:2263-2273`) | Per `## State ownership` (already ruled): **dissolves entirely** — "its field list is the reference vocabulary for `FrameData`'s event payloads, not a struct that survives"; the poly/polyVert data moves into `FrameEvent::AddPolyToScene`/`AddDecalToScene` payloads as owned `Vec<PolyVert>` | Free — only its own file. Carries the campaign's largest layout-assert block (~2 MB `#[repr(C)]` struct, `drawSurfs`/`dlights`/`entities`/`miniEntities` fixed arrays) — asserts retire with the type |
| `trRefdef_t` (`tr_local/tr_refdef_t.rs`) | `text: [[c_char; N]; M]`, `entities: *mut trRefEntity_t`, `miniEntities: *mut trMiniRefEntity_t`, `dlights: *mut dlight_t`, `polys: *mut srfPoly_t`, `drawSurfs: *mut drawSurf_t`, `areamaskModified: qboolean` | `tr_scene` (`oracle/codemp/renderer/tr_local.h:563-598`) | Per A1, the array fields' *content* crosses as `FrameData`/`FrameEvent` payloads (owned `Vec<T>` per array on the event); `areamaskModified` → `bool`; `text` → `Vec<String>`. A render-thread-local "current refdef" carrier still exists for `RC_DRAW_SURFS`'s stays-render-side leg (not a full erasure) | Free — only its own file plus `draw_surfs_command_t.rs`/`back_end_state_t.rs`/`tr_globals_t.rs` |
| `trRefEntity_t` (`tr_local/tr_ref_entity_t.rs`) | `needDlights`/`lightingCalculated: qboolean` | `tr_scene` (`oracle/codemp/renderer/tr_local.h:94-106`) | `bool` ×2 | Free — only `tr_local/{tr_refdef_t,back_end_state_t,back_end_data_t,tr_globals_t}.rs` |
| `viewParms_t` (`tr_local/view_parms_t.rs`) | `isPortal`/`isMirror: qboolean` | `tr_scene`/`tr_backend` (`oracle/codemp/renderer/tr_local.h:629-644`) | `bool` ×2 | Free — only `tr_local/{draw_surfs_command_t,back_end_state_t,tr_globals_t}.rs` |
| `rotatePicCommand_t` (`tr_local/rotate_pic_command_t.rs`) | `shader: *mut shader_s` | `tr_backend` 2D draw (`oracle/codemp/renderer/tr_local.h:2221-2229`) | Per A1: **dissolves** into `FrameEvent::DrawRotatePic`/`DrawRotatePic2`; `shader` becomes a `Handle<Shader>` payload field on the variant | Free — only its own file |
| `stretchPicCommand_t` (`tr_local/stretch_pic_command_t.rs`) | `shader: *mut shader_s` | `tr_backend` 2D draw (`oracle/codemp/renderer/tr_local.h:2212-2219`) | Per A1: **dissolves** into `FrameEvent::DrawStretchPic`; `shader` → `Handle<Shader>` | Free — only its own file |
| `subImageCommand_t` (`tr_local/sub_image_command_t.rs`) | `image: *mut image_t`, `data: *mut c_void` | `tr_backend`/`tr_image` (`oracle/codemp/renderer/tr_local.h:2195-2201`) | Per A1: "provisionally dead (no MP trap found)" — **dissolves**, pending the A7 `RE_SubImage` call-site grep before R3 freezes this family; if confirmed dead, dropped with no Rust replacement | Free — only its own file |
| `shaderCommands_s` (`tr_local/shader_commands_s.rs`) | `shader: *mut shader_t`, `xstages: *mut shaderStage_t`, `SSInitializedWind: qboolean` | `tr_shade` tess buffer (`oracle/codemp/renderer/tr_local.h:1844-1883`) | Per `## State ownership` (already ruled): **dissolves** into R4's tessellation/vertex-building pipeline — no single global scratch buffer survives the new topology | Free — only its own file. Second-largest layout-assert block (~128 KB fixed-size tess arrays) — asserts retire with the type |
| `srfPoly_t` (`tr_local/srf_poly_s.rs`) | `verts: *mut polyVert_t` | `tr_scene` overlay polys (`oracle/codemp/renderer/tr_local.h:692-698`) | Owned `Vec<polyVert_t>` — the payload shape `FrameEvent::AddPolyToScene`/`AddDecalToScene` already carries | Free — only `tr_local/{back_end_data_t,tr_refdef_t}.rs` |

**Group 4 — Ghoul2 model-render surface**

| Type (file) | Raw fields | Owning subsystem / replacing wave | Proposed replacement shape | Can change? (validation) |
|---|---|---|---|---|
| `CRenderableSurface` (`tr_local/crenderable_surface.rs`) | `boneCache: *mut c_void`, `surfaceData: *mut mdxmSurface_t`, `alternateTex: *mut f32`, `goreChain: *mut c_void` | Ghoul2 render-side surface (`oracle/codemp/renderer/tr_local.h:2047-2101`) — the render-side counterpart of the already-DEC-35'd server-side ghoul2 ownership | `boneCache` → `Handle` into a bone-cache arena (mirrors the settled DEC-35 pattern); `surfaceData` → a `Handle`/view into the mdx parsed-once sidecar (mirroring `MdxmParsed`/`MdxmView`); `alternateTex` → owned `Vec<f32>`; `goreChain` → `Option<Handle<GoreChain>>` mirroring the existing gore-ownership shape | Free today — grep shows only `tr_local/{mod,mp,sp}` self-references; `crates/mp/engine/ghoul2`'s own `CBoneCache`-family types are server-side and distinct from this renderer-side struct. **Re-verify when the ghoul2 render-side integration wave lands** — it is the type most likely to pick up a real consumer between now and R3 |

**Group 5 — The registry root + platform**

| Type (file) | Raw fields | Owning subsystem / replacing wave | Proposed replacement shape | Can change? (validation) |
|---|---|---|---|---|
| `trGlobals_t` (`tr_local/tr_globals_t.rs`) | `registered`/`worldMapLoaded: qboolean`, `world: *mut world_t`, `externalVisData: *const u8`, 8 `*mut image_t` singleton fields + `scratchImage`/`lightmaps: [*mut image_t; N]`, 5 `*mut shader_t` singleton fields + `shaders`/`sortedShaders: [*mut shader_t; 16384]`, `currentEntity: *mut trRefEntity_t`, `currentModel: *mut model_t`, `models: [*mut model_t; 1024]`, `bspModels: [world_t; 32]`, `skins: [*mut skin_t; 1024]` | The struct this entire document targets (`oracle/codemp/renderer/tr_local.h:1309-1423`) | **Already dispositioned by `## State ownership`** — restated here, not re-derived: registries (`models`/`shaders`/`skins`/`bspModels`/`world`/`lightmaps`) → `RenderAssets` fields behind `Arc`+A9 COW publish (`Vec`/arena-backed, `Handle`-indexed); `registered` → `RenderAssets::registered` (`bool`); `worldMapLoaded` → similar `bool`; frontend scratch/singletons (`currentEntity`, `currentModel`, the named single-image/-shader pointers) → `RenderWorld::frame: FrameState` fields, `Option<Handle<T>>` | Free — grep: only `tr_local/{mod,tr_globals_t}.rs` (self); zero external consumers. Highest-stakes row to get right, but not blocked by anything |
| `glstate_t` (`tr_local/glstate_t.rs`) | `finishCalled: qboolean` | `tr_backend` GL-state cache (`oracle/codemp/renderer/tr_local.h:1253-1260`) | Per `## State ownership` (already ruled): → `GpuResources::gl_state`, "a named placeholder ... until R4 defines the real pipeline/bind-group cache" — **dissolves** into that cache; `finishCalled` drops unless R4's cache needs an equivalent flag | Free — only its own file |
| `CPBUFFER` (`tr_local/cpbuffer.rs`) | `m_hRC`/`m_hDC`/`m_hOldRC`/`m_hOldDC`/`m_hBuffer: *mut c_void` | Win32 WGL pixel-buffer context (`oracle/codemp/renderer/tr_local.h:1156-1197`) | **Dissolves** — Win32-specific GL context plumbing has no counterpart under the wgpu-based R4 architecture (DEC-01 wgpu lean); dropped, not reimplemented | Free — only its own file |
| `refexport_t` (`tr_public/refexport_t.rs`) | Every field `Option<unsafe extern "C" fn(...)>`; signatures carry `*const c_char`, `*mut/*const vec3_t`, `qboolean` params | Renderer export vtable (`oracle/codemp/renderer/tr_public.h:14-110`) | Per `## State ownership`'s `re` row (already ruled, DEC-37 ruling 4): **deleted** — replaced by direct calls / a Rust trait (porting-rules §C8: function-pointer tables → traits) | Free — only `tr_public/mod.rs` re-exports it; zero call-site consumers found anywhere |

**Group 6 — Model family: already-live `tr_model` + on-disk `.glm`/`.gla` headers**

| Type (file) | Raw fields | Owning subsystem / replacing wave | Proposed replacement shape | Can change? (validation) |
|---|---|---|---|---|
| `model_t` (`tr_local/model_s.rs`) | `name: [c_char; 64]`, `bmodel: *mut bmodel_t`, `md3: [*mut md3Header_t; 3]`, `mdxm: *mut mdxmHeader_t`, `mdxa: *mut mdxaHeader_t`, `bspInstance: qboolean` | **`tr_model` — already live, FROZEN `docs/subsystems/tr-model.md`** (`oracle/codemp/renderer/tr_local.h:1117-1135`); this row falls outside the usual R3-scaffolding framing | The live design already answers this for the dedicated-server path (zero-init `Box<ModelData>` + raw-cast writes during endian-swap load); a client-rendering R3 wave would need `bmodel` → `Handle<BModel>` (once `tr_bsp` lands), `name` → `String`, `bspInstance` → `bool` — but `md3`/`mdxm`/`mdxa` stay raw-pointer-shaped **on the live path**, where `server_load.rs`'s in-place endian-swap-and-cast strategy is the working design | **Coupled: `crates/mp/renderer/src/tr_model` (FROZEN, live jampded headless model/skin subset per `CLAUDE.md`'s jampDed link-set list)** — `RenderModels::r_alloc_model`/`register_server_model_mdxa`(`server_load.rs:322`)/`register_server_model_mdxm`(`:430`) directly read/write `.mdxa`/`.mdxm`/`.md3`/`.index`/`.dataSize`/`.r#type` today. `mp_host_interface`/`mp_engine_ghoul2` mention `model_t` only in doc comments (DEC-35 deliberately never imports it) |
| `mdxaHeader_t` (`mdx_format/mdxa_header_t.rs`) | `name: [c_char; 64]` (all other fields are file offsets, not pointers) | On-disk `.gla` file-format header (`oracle/codemp/renderer/../ghoul2/../renderer/mdx_format.h:351-371`) | **Special case, outside the R3 scaffolding pattern**: a frozen on-disk binary layout dictated by the real `.gla` file format — `server_load.rs` casts raw file bytes to `*mut mdxaHeader_t` in place for endian-swapping; converting the struct to owned Rust fields would break that zero-copy strategy. No replacement shape — stays `#[repr(C)]` permanently (same category as tier 1, frozen by file-format necessity rather than ABI necessity) | **Coupled: `crates/mp/renderer/src/tr_model/server_load.rs`** (live jampded subset) — real `as *mut mdxaHeader_t` cast + field reads/writes (`:321-322`) during the load endian-swap |
| `mdxmHeader_t` (`mdx_format/mdxm_header_t.rs`) | `name`/`animName: [c_char; 64]` ×2 (rest are offsets) | On-disk `.glm` file-format header (`oracle/codemp/renderer/../ghoul2/../renderer/mdx_format.h:153-172`) | Same special case as `mdxaHeader_t` — stays `#[repr(C)]` permanently | **Coupled: `crates/mp/renderer/src/tr_model/server_load.rs`** — real `as *mut mdxmHeader_t` casts (`:429,520`) |
| `mdxaSkel_t` (`mdx_format/mdxa_skel_t.rs`) | `name: [c_char; 64]` | On-disk `.gla` bone entry (`oracle/codemp/renderer/../ghoul2/../renderer/mdx_format.h:388-396`) | Same special-case framing, if/when it gains a consumer | Free — **genuinely dormant**: zero real consumers anywhere in the repo today (no cast, no construction, nothing beyond its own `size_of` assert); `mp_host_interface`'s `mdx/mdxa.rs` re-derives its own byte-offset view instead of importing this struct. Re-verify once ghoul2 skeleton-parsing is ported |
| `mdxmSurfHierarchy_t` (`mdx_format/mdxm_surf_hierarchy_t.rs`) | `name`/`shader: [c_char; 64]` ×2 | On-disk `.glm` surface-hierarchy entry (`oracle/codemp/renderer/../ghoul2/../renderer/mdx_format.h:187-195`) | Same special-case framing as the headers | **Coupled: `crates/mp/renderer/src/tr_model/server_load.rs`** — real `as *mut mdxmSurfHierarchy_t` cast + `offset_of!` use (`:493,514,516`) during the hierarchy walk |

**Excluded from this table**: `crates/mp/renderer/src/tr_model/{aligned_bytes.rs,cached_model_binary.rs}` define `AlignedBytes`/`CachedEndianedModelBinary` — bespoke Rust infrastructure built to support the live `mdxaHeader_t`/`mdxmHeader_t` cast sites, not ported Raven-named tier-2 types, so they carry no Raven fidelity obligation and are out of this audit's scope.

## Seam definition

Signatures only, per DEC-37 ruling 3's state-partition law (`RenderAssets`
CPU/Arc-shared/sim-readable vs `GpuResources` render-thread-only), A2 (four
typed arenas with `(index: u32, generation: u32)` handles), and A9 (the
sim-owned `Arc<RenderAssets>` mutation/publish path).

```rust
// --- generic handle infra (new construct, no Raven counterpart) ---

/// A generation-counted index into one of `RenderAssets`'s four arenas.
/// Per-kind typing (`K`) catches cross-kind handle mixups at compile time
/// (A2) — e.g. an `ImageHandle` cannot be passed where a `ShaderHandle` is
/// expected, even though both are `Handle<u32, u32>` underneath.
///
/// No oracle citation: new Rust-side infrastructure implementing ruling 11's
/// generation-counted-handle requirement. Second instance of the
/// `AlignedBytes` justified-exception precedent (`docs/subsystems/
/// tr-model.md` `TRM-D4`/ruling 58) — approved by A7, no `Source:` line.
pub struct Handle<K> {
    index: u32,
    generation: u32,
    _kind: core::marker::PhantomData<fn() -> K>,
}

// Hand-written, not `#[derive(Clone, Copy, PartialEq, Eq, Hash)]` (NB-4): a
// derive adds a `K: Trait` bound, but `K` is only ever a marker here — the
// asset structs it's instantiated with (`ImageAsset`/`ShaderAsset`/
// `SkinAsset`/`ModelAsset`) are not `Copy`, so a derived `Copy` would make
// every `*Handle` type non-`Copy`, breaking every `Arena<T>::get`/`get_mut`/
// `remove` call site that takes a handle by value (R2-D3).
impl<K> Clone for Handle<K> {
    fn clone(&self) -> Self { *self }
}
impl<K> Copy for Handle<K> {}
impl<K> PartialEq for Handle<K> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}
impl<K> Eq for Handle<K> {}
impl<K> core::hash::Hash for Handle<K> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.generation.hash(state);
    }
}

pub struct ImageAsset { /* image_t fields, oracle/codemp/renderer/tr_local.h:136-151 */ }
pub struct ShaderAsset { /* shader_t fields, oracle/codemp/renderer/tr_local.h:459-530 */ }
pub struct SkinAsset { /* skin_t fields, oracle/codemp/renderer/tr_local.h:609-613 */ }
pub struct ModelAsset { /* model_t fields, oracle/codemp/renderer/tr_local.h:1117-1135 */ }

pub type ImageHandle = Handle<ImageAsset>;
pub type ShaderHandle = Handle<ShaderAsset>;
pub type SkinHandle = Handle<SkinAsset>;
pub type ModelHandle = Handle<ModelAsset>;

/// A generic generation-counted arena backing one `RenderAssets` registry.
/// `#[derive(Clone)]` — required by `Arc::make_mut` on `RenderAssets`
/// (A9/NB-1); adds the ordinary `T: Clone` bound (no `PhantomData` issue —
/// `Arena<T>` stores `T` directly, unlike `Handle<K>` above).
///
/// Shader/skin/model arenas soft-cap at their oracle `MAX_*` constant; the
/// image arena (`RenderAssets::images` below) is the one exception — it
/// stays unbounded, matching its real oracle backing store (A5).
///
/// **Slot 0 reservation (A12).** Every capped arena is constructed with
/// slot 0 pre-populated with the registry's oracle default entry — models
/// index 0 is `MOD_BAD` (`R_ModelInit`,
/// `oracle/codemp/renderer/tr_model.cpp:1665-1680`), skins index 0 is
/// `"<default skin>"` (`R_InitSkins`,
/// `oracle/codemp/renderer/tr_image.cpp:3324-3332`), shaders index 0 is
/// `tr.defaultShader` (`CreateInternalShaders`,
/// `oracle/codemp/renderer/tr_shader.cpp:4137-4155`). `Handle { index: 0,
/// generation: 0 }` IS that live default, not a null/invalid sentinel — the
/// image arena has no reserved slot (uncapped, A5; a failed lookup returns
/// `Option::None` from `image_names`, never a handle).
#[derive(Clone)]
pub struct Arena<T> {
    slots: Vec<Option<(u32 /* generation */, T)>>,
    free_list: Vec<u32>,
    /// `None` for the unbounded image arena; `Some(MAX_*)` for
    /// shader/skin/model (A5). `Some` also implies slot 0 is reserved
    /// (A12) and never enters `free_list`.
    soft_cap: Option<u32>,
}

impl<T> Arena<T> {
    /// On overflow: warns (shaders/skins reproduce retail's `Com_Printf`;
    /// models are SILENT in retail — this port adds a clearly-marked
    /// warning, charter interior freedom / A5 amendment) and returns
    /// `Handle { index: 0, generation: 0 }` — the pre-populated default
    /// entry (A12), matching every oracle overflow path exactly: shaders'
    /// `return tr.defaultShader` and skins'/models' `return 0` all resolve
    /// to the same live slot-0 object. Never `Result`.
    pub fn insert(&mut self, value: T) -> Handle<T>;
    pub fn get(&self, handle: Handle<T>) -> Option<&T>;
    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T>;
    pub fn remove(&mut self, handle: Handle<T>) -> Option<T>;
    pub fn iter(&self) -> impl Iterator<Item = (Handle<T>, &T)>;
}

// --- RenderAssets: CPU, immutable-after-publish, Arc-shared, sim-readable (ruling 3) ---

/// `#[derive(Clone)]` — required by `Arc::make_mut(&mut RenderAssetsSim
/// ::published)` (A9/NB-1). Every field type must derive `Clone` in turn;
/// the four `Arena<T>` fields already do, and the R3/R4-owned placeholder
/// types below (`WorldAsset`, `FunctionTables`, `GlConfig`,
/// `AutomapWireframe`) pick up the derive when their crates land.
#[derive(Clone)]
pub struct RenderAssets {
    /// Unbounded (A5) — mirrors `tr_image.cpp`'s `AllocatedImages` std::map
    /// backing store (Part 1 finding); no `MAX_DRAWIMAGES` soft-cap, no
    /// slot-0 reservation (A12 applies only to the three capped arenas
    /// below).
    pub images: Arena<ImageAsset>,
    /// Mirrors the oracle's `AllocatedImages` name→ptr map (Part 1 finding),
    /// keyed by the lower-cased, extension-stripped name
    /// (`GenerateImageMappingName`,
    /// `oracle/codemp/renderer/tr_image.cpp:1287-1289`) — lookup-by-name is
    /// a first-class index in this arena, matching ruling 11's cache-key
    /// scheme (A5).
    pub image_names: std::collections::HashMap<String, ImageHandle>,
    /// `tr.lightmaps[MAX_LIGHTMAPS]` (`image_t*` in the oracle, folded into
    /// `images` rather than a fifth arena — not a name-keyed lookup, a
    /// **positional, index-addressable** list: `R_FindShader` reads it by
    /// small integer (`stage->bundle[0].image = tr.lightmaps
    /// [shader.lightmapIndex[0]]`, e.g.
    /// `oracle/codemp/renderer/tr_shader.cpp:3543`), so the fold needs an
    /// index alongside the name map, not instead of it.
    /// `oracle/codemp/renderer/tr_local.h:1364`.
    pub lightmaps: Vec<ImageHandle>,
    /// Soft-capped at `MAX_SHADERS = 16384`; slot 0 pre-populated with
    /// `tr.defaultShader`; overflow warns and returns `Handle{0,0}` — that
    /// same default (A5, A5 amendment, A12).
    pub shaders: Arena<ShaderAsset>,
    /// Shader lookup key (`R2-D4`): the oracle's `IsShader`
    /// (`oracle/codemp/renderer/tr_shader.cpp:3373-3398`) matches a
    /// stripped name **plus** the full `lightmapIndex[MAXLIGHTMAPS]`/
    /// `styles[MAXLIGHTMAPS]` arrays (`MAXLIGHTMAPS = 4`,
    /// `oracle/codemp/qcommon/qfiles.h:310`) — multiple shaders share one
    /// stripped name, differentiated by those arrays, so a plain
    /// `HashMap<String, ShaderHandle>` (the `image_names` shape) cannot
    /// represent it. The array compare is itself guarded —
    /// `if (!sh->defaultShader)` (`oracle/codemp/renderer/
    /// tr_shader.cpp:3382`) — so a *default* shader (created when no real
    /// shader/image was found for a name) matches on the stripped name
    /// alone, skipping the array walk; a real porter's per-candidate
    /// comparison reproduces that short-circuit, not just the array
    /// compare. Candidates are compared per-entry at lookup time,
    /// mirroring `IsShader`'s walk exactly (method body is R3 scope).
    pub shader_lookup: std::collections::HashMap<String, Vec<ShaderHandle>>,
    /// Soft-capped at `MAX_SKINS = 1024`; slot 0 pre-populated with
    /// `"<default skin>"`; overflow warns and returns `Handle{0,0}` (A5, A5
    /// amendment, A12).
    pub skins: Arena<SkinAsset>,
    /// Skin lookup key (`R2-D4`): `RE_RegisterSkin`'s name walk
    /// (`oracle/codemp/renderer/tr_image.cpp:3128-3136`) compares the full
    /// name only (`Q_stricmp`, no stripping, no per-entry array) — plain
    /// name→handle, unlike shaders.
    pub skin_lookup: std::collections::HashMap<String, SkinHandle>,
    /// Soft-capped at `MAX_MOD_KNOWN = 1024`; slot 0 pre-populated with
    /// `MOD_BAD`; overflow is silent in retail, this port adds a marked
    /// warning, returns `Handle{0,0}` (A5, A5 amendment, A12).
    pub models: Arena<ModelAsset>,
    /// Model lookup key (`R2-D4`): `RE_RegisterModel`'s `mhHashTable` walk
    /// (`oracle/codemp/renderer/tr_model.cpp:1211-1215`) is also plain
    /// name→handle (`Q_stricmp` against the full name, no stripping) —
    /// same shape as `skin_lookup`, kept as its own field for per-kind
    /// handle typing (A2).
    pub model_lookup: std::collections::HashMap<String, ModelHandle>,
    pub world: Option<WorldAsset>,       // tr.world — replaced wholesale on level load
    /// `tr.bspModels[MAX_SUB_BSP]` — sub-BSP worlds, homed beside `world`
    /// rather than a fifth arena (`oracle/codemp/renderer/tr_local.h:1399`).
    pub bsp_models: Vec<WorldAsset>,
    pub function_tables: FunctionTables, // sin/square/triangle/sawtooth/inverseSawtooth/fog, oracle/codemp/renderer/tr_local.h:1412-1417
    /// `tr.distanceCull`/`distanceCullSquared` — sim-readable because
    /// `CG_R_GETDISTANCECULL` reads `distanceCull` synchronously (B11).
    /// `oracle/codemp/renderer/tr_local.h:1420`.
    pub distance_cull: f32,
    pub distance_cull_squared: f32,
    /// `glconfig_t` — sim-readable because `CG_R_GETREALRES` reads
    /// `vidWidth`/`vidHeight` synchronously (B11).
    /// `oracle/codemp/renderer/tr_local.h:1435`.
    pub glconfig: GlConfig,
    /// `tr.registered` (`oracle/codemp/renderer/tr_local.h:1310`, "cleared
    /// at shutdown, set at beginRegistration") — the guard every
    /// scene-composition trap reads first (`RE_AddRefEntityToScene`,
    /// `oracle/codemp/renderer/tr_scene.cpp:195-197`; `RE_AddPolyToScene`,
    /// `:124-126`; same pattern on every `RE_Add*ToScene`). A session flag,
    /// not per-frame scratch — sim-readable/`RenderAssets`-owned by the
    /// same reasoning as `distance_cull`/`glconfig` (B11, `R2-D2`): a
    /// trap-time validation read must reach it without touching
    /// render-thread state. Mutated rarely (registration begin/end) via
    /// the same `Arc::make_mut` path as everything else in `RenderAssets`
    /// (A9) — no dedicated field-specific mutation mechanism needed.
    pub registered: bool,
    /// Wireframe automap data (A10). `CG_R_INITWIREFRAMEAUTO` cannot be an
    /// ordered `FrameEvent` — it must answer synchronously with the
    /// oracle's `qboolean` validity result — so
    /// `R_InitializeWireframeAutomap`'s live rebuild
    /// (`oracle/codemp/renderer/tr_world.cpp:1205-1231`) becomes a sim-side
    /// A9 mutation instead: pure CPU work walking `world.nodes`, ruling 3
    /// intact. Replaces `g_autoMapFrame`/`g_autoMapValid`
    /// (`oracle/codemp/renderer/tr_world.cpp:782,784`).
    pub automap_wireframe: AutomapWireframe,
}

/// `R_SetLightStyle`/`R_GetLightStyle` backing table (A6, extended by A9) —
/// sim-owned, **`RenderAssets`-ADJACENT, not inside its `Arc`**: mutated in
/// place at trap time via ordinary `&mut` access, not `Arc::make_mut`
/// copy-on-write, because it snapshots at scene-render marks rather than
/// publishing per registration event. Mirrors `styleColors
/// [MAX_LIGHT_STYLES]` (`oracle/codemp/renderer/tr_local.h:1888`).
pub struct LightStyleTable {
    pub colors: [[u8; 4]; MAX_LIGHT_STYLES],
}

// --- sim-thread-owned: the mutation/publish path (A9) ---

/// Sim-thread-owned. `published` IS the master — there is no separate
/// mutable-then-copied staging struct (NB-1: an earlier draft of this doc
/// sketched a two-field `{ master, published }` shape, but mutating a
/// `master` field and then calling `Arc::make_mut` on a *different*
/// `published` field propagates nothing; that shape cannot implement A9 as
/// ratified). Registration calls `Arc::make_mut(&mut self.published)`,
/// which mutates the existing allocation in place when the render thread
/// holds no other reference, or clones once and mutates the clone when it
/// does — ordinary copy-on-write, no locks. The fresh/mutated `Arc` becomes
/// visible to the render thread (`RenderWorld::assets`) at the next frame
/// boundary. New construct, no single Raven counterpart: the oracle's `tr`
/// registries are globals mutated in place with no publish step, because
/// Raven has no cross-thread Arc-sharing to protect (ruling 1 — no
/// coherent Raven interior here). `LightStyleTable` sits adjacent, not
/// behind the `Arc` (A6/A9).
pub struct RenderAssetsSim {
    pub published: std::sync::Arc<RenderAssets>,
    pub light_styles: LightStyleTable,
}

impl RenderAssetsSim {
    /// `Arc::make_mut(&mut self.published)` then mutates in place (A5
    /// soft-cap + slot-0 fallback, A12) — visible to the render thread at
    /// the next frame boundary (A9). Bodies are R3 scope.
    pub fn register_shader(&mut self, /* … */) -> ShaderHandle;
    pub fn register_skin(&mut self, /* … */) -> SkinHandle;
    pub fn register_model(&mut self, /* … */) -> ModelHandle;
    pub fn register_image(&mut self, /* … */) -> ImageHandle;
    pub fn remap_shader(&mut self, /* … */);
    /// `CG_R_INITWIREFRAMEAUTO` (A10) — rebuilds `published.automap_wireframe`
    /// from `published.world` via the same `Arc::make_mut` path as
    /// registration, then returns the oracle's `qboolean` validity result
    /// synchronously. Body is R3 scope.
    pub fn rebuild_automap_wireframe(&mut self) -> bool;
    /// `RE_SetLightStyle` (`oracle/codemp/renderer/tr_init.cpp:1438-1450`)
    /// — mutates `self.light_styles.colors[style]` in place, **not** via
    /// `Arc::make_mut` (A6/A9: `LightStyleTable` is adjacent, not inside
    /// the `Arc`). `[u8; 4]` replaces the oracle's packed `int color`,
    /// matching `LightStyleTable::colors`' element type and the
    /// `*(DWORD *)styleColors[…]`-style reinterpretation its render-side
    /// consumers already do (`oracle/codemp/renderer/tr_shade.cpp:1401`) —
    /// out-param → return value + typed color per §C7, not a bare `int`.
    /// Validates `style < MAX_LIGHT_STYLES` before mutating and diverges
    /// through `com_error(ERR_FATAL, …)` on failure otherwise, matching the
    /// oracle's own `Com_Error(ERR_FATAL, "RE_SetLightStyle: %d is out of
    /// range", style)` (`R2-D11`); `style: usize` closes the oracle's
    /// missing `style < 0` check by construction (§19 — no oracle-fidelity
    /// loss, retail has no defined behavior for a negative style either).
    /// Body is R3 scope.
    pub fn set_light_style(&mut self, style: usize, color: [u8; 4]);
    /// `RE_GetLightStyle` (`oracle/codemp/renderer/tr_init.cpp:1427-1436`)
    /// — reads `self.light_styles.colors[style]`; same bounds contract and
    /// `style: usize`/return-value shape as `set_light_style`. Body is R3
    /// scope.
    pub fn get_light_style(&self, style: usize) -> [u8; 4];
}

// --- render-thread-local world: GpuResources + frame state ---

/// Render-thread-only. Never touched by a trap query (ruling 3 invariant).
pub struct GpuResources {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    /// GPU-side twin of `RenderAssets.images`, keyed by the same handle.
    pub gpu_images: SecondaryArena<ImageHandle, GpuImage>,
    pub pipelines: std::collections::HashMap<PipelineKey, wgpu::RenderPipeline>, // ruling 6
    /// `glstate_t` equivalent (B6) — the GL binding cache has no wgpu
    /// meaning; placeholder until R4 defines the real pipeline/bind-group
    /// cache. `oracle/codemp/renderer/tr_local.h:1253-1260`.
    pub gl_state: GlStatePlaceholder,
}

/// Render-thread-local scratch, replacing `backEnd`'s role in full — all 11
/// `backEndState_t` fields accounted for (B5).
/// `backEndData_t`'s double-buffer is NOT reproduced here — see the A1
/// disposition table and `R2-D8` (buffer-recycling mechanics defer to R4).
pub struct FrameState {
    pub refdef: TrRefdef,
    pub view: ViewParms,
    pub ori: OrientationR,
    pub counters: BackEndCounters,
    pub is_hyperspace: bool,
    /// `trRefEntity_t *currentEntity` — points at whichever entity the
    /// backend is currently drawing. Represented by value, not a pointer:
    /// the renderer interior is oracle-match-free (DEC-37 ruling 1).
    pub current_entity: Option<RefEntity>,
    pub sky_rendered_this_view: bool,
    pub projection_2d: bool,
    pub color_2d: [u8; 4],
    pub vertexes_2d: bool,
    /// `trRefEntity_t entity2D` — a value field in the oracle (`current
    /// Entity` points here during 2D rendering), not a pointer.
    pub entity_2d: RefEntity,
    /// The A11 snapshot carrier's consumer-side landing field: filled from
    /// `FrameEvent::RenderScene.light_styles` when the render thread
    /// processes that event, then read by the R4 tessellation/vertex-
    /// building consumers for the rest of that scene's surfaces
    /// (`oracle/codemp/renderer/tr_surface.cpp:279,324`,
    /// `oracle/codemp/renderer/tr_shade.cpp:1401,1685`,
    /// `oracle/codemp/renderer/tr_light.cpp:234-274` — same sites `R2-D5`
    /// cites for the producer side). Not sim-owned `LightStyleTable`
    /// itself — a per-frame copy, so
    /// backend surface processing never reaches across to sim-owned state.
    pub scene_light_styles: [[u8; 4]; MAX_LIGHT_STYLES],
}

pub struct RenderWorld {
    pub assets: std::sync::Arc<RenderAssets>, // the sim's `published` Arc, picked up at frame boundary (A9)
    pub gpu: GpuResources,
    pub frame: FrameState,
}
```

`SecondaryArena<K, V>`, `PipelineKey`, `GpuImage`, `GlStatePlaceholder`,
`GlConfig`, `WorldAsset`, `FunctionTables`, `ViewParms`, `OrientationR`,
`BackEndCounters`, `TrRefdef`, `RefEntity`, `Poly`, `PolyVert`,
`AutomapWireframe`, `Vec3`, and the `MAX_LIGHT_STYLES` constant (`= 64`,
`oracle/codemp/game/q_shared.h:423-424`) are named but not defined here —
they land with whichever crate owns them: the CPU/frontend names
(`WorldAsset`, `FunctionTables`, `ViewParms`, `TrRefdef`, `RefEntity`,
`Poly`, `PolyVert`, `GlConfig`, `AutomapWireframe`, `Vec3`,
`MAX_LIGHT_STYLES`) at R3, the GPU-facing names (`SecondaryArena`,
`PipelineKey`, `GpuImage`, `GlStatePlaceholder`) at R4. This is root-type
surface only, per DEC-37 ruling 14's R2 scope.

The sketches above spell fully-qualified paths inline (`std::sync::Arc`,
`std::collections::HashMap`, `core::marker::PhantomData`, `core::hash
::Hash`) so each code block reads standalone; `porting-rules.md`'s
no-inline-fully-qualified-paths rule applies at transcription, where these
become ordinary file-top `use` imports.

### `FrameData` — the ordered event stream (A1)

Enumerated from the 57 `CG_R_*`/`UI_R_*` `Args` types
(`crates/mp/abi/src/{cgame,ui}/syscalls/`) plus the two table-bypassing
direct writes that carry per-frame draw/scene data (`SetRangeFog`,
`SetRefractionProp` — see the eight-trap table above for the full bypass
list). Variants are the traps that **mutate ordered per-frame draw/scene
state**; traps that are synchronous, non-event calls (registration, bounds
queries, glconfig, PVS, lighting queries, font metrics, light styles,
`distanceCull`, the automap wireframe rebuild, the weather-contents-override
no-op) are **not** events — per ruling 3 they stay direct calls against
`Arc<RenderAssets>` (or `RenderAssetsSim`/`LightStyleTable` for mutations —
registration and the A10 automap rebuild both go through the same A9
publish path) from whichever thread issues the trap.

**Event-append validation state (general principle).** Scene-composition
traps validate *before* pushing their `FrameEvent` — e.g.
`RE_AddRefEntityToScene`'s `tr.registered`/bound checks, `## Seam
definition`'s `RenderAssets::registered` field above. Two different kinds
of state feed that validation, and they live in two different places:

- **Session/asset-registry state** (`tr.registered`, capacity bounds) is
  sim-side, `RenderAssets`-owned, reached the same way every other
  synchronous trap reaches it — never render-thread `FrameState`.
- **Per-frame append counters** (`r_numentities`, `r_numdlights`,
  `r_numpolys`, `r_numpolyverts` — none of them `trGlobals_t` fields in the
  oracle; all four are file-scope statics, `oracle/codemp/renderer/
  tr_scene.cpp:21-33`) are not renderer state at all under this design —
  they are properties of **the `FrameData` currently under construction**
  on whichever thread issues the traps for this frame. The reset point is
  `R_ToggleSmpFrame` (`oracle/codemp/renderer/tr_scene.cpp:44-65`), called
  once at frame start, zeroing all four; `RE_ClearScene`
  (`oracle/codemp/renderer/tr_scene.cpp:74-80`) does **not** reset them —
  it only records the `r_firstScene*` per-scene *offsets* (and never
  touches `r_numpolyverts` at all) — `RE_RenderScene` later subtracts them
  from the current counts to fill `trRefdef_t.num_entities`/`num_dlights`/
  `numPolys` for just that scene
  (`oracle/codemp/renderer/tr_scene.cpp:796-813`), which is exactly the
  arithmetic `FrameEvent::RenderScene`'s payload needs to reproduce. The
  bound itself is explicitly per-**frame**, not
  per-scene: `oracle/codemp/renderer/tr_local.h:2254-2255`, "the limits
  apply to the sum of all scenes in a frame — the main view, all the 3D
  icons, etc." So the derivation is entity count = events pushed **since
  the start of this `FrameData`** (i.e. since the last `R_ToggleSmpFrame`-
  equivalent frame boundary), not since the last `ClearScene` — `ClearScene`
  only marks a sub-range within that same, still-accumulating count. They
  never touch `RenderWorld::frame: FrameState` (render-thread-only,
  ruling 3) because nothing about them is render-thread state to begin
  with — exact bookkeeping (a running counter incremented per push vs. a
  scan) is a method-body detail, R3 scope.

This closes the append-time question left implicit by the disposition
table: a trap-time append reads only sim-side `RenderAssets` state plus the
`FrameData` it is itself building, never `FrameState`.

```rust
pub struct FrameData {
    /// Recycled buffer (ruling 2) — one 2D/scene event stream per frame,
    /// in trap-call order. Recycling protocol (fixed pool vs. free-list)
    /// settles at R4 — `R2-D8`.
    pub events: Vec<FrameEvent>,
}

pub enum FrameEvent {
    // --- scene composition (CG_R_CLEARSCENE / UI_R_CLEARSCENE, etc.) ---
    ClearScene,
    ClearDecals,                                   // CG_R_CLEARDECALS — cgame-only, no UI trap
    AddRefEntityToScene(RefEntity),                // CG_R_/UI_R_ADDREFENTITYTOSCENE
    AddPolyToScene { shader: ShaderHandle, verts: Vec<PolyVert> },       // CG_R_/UI_R_ADDPOLYTOSCENE
    AddPolysToScene { shader: ShaderHandle, polys: Vec<Poly> },          // CG_R_ADDPOLYSTOSCENE — cgame-only
    AddLightToScene { org: Vec3, intensity: f32, r: f32, g: f32, b: f32 },       // CG_R_/UI_R_ADDLIGHTTOSCENE
    AddAdditiveLightToScene { org: Vec3, intensity: f32, r: f32, g: f32, b: f32 }, // CG_R_ADDADDITIVELIGHTTOSCENE — cgame-only
    // CG_R_ADDDECALTOSCENE — cgame-only. Exact trap signature
    // (`oracle/codemp/renderer/tr_public.h:56`, dispatched
    // `oracle/codemp/client/cl_cgame.cpp:903-904`); no invented fields (B9).
    AddDecalToScene {
        shader: ShaderHandle, origin: Vec3, dir: Vec3, orientation: f32,
        r: f32, g: f32, b: f32, a: f32, alpha_fade: bool, radius: f32,
        temporary: bool,
    },
    SetRangeFog(f32),                              // CG_R_SETRANGEFOG — table-bypass write to tr.rangedFog
    SetRefractionProp { alpha: f32, stretch: f32, pre_post: bool, negate: bool }, // CG_R_SETREFRACTIONPROP — table-bypass
    // CG_R_/UI_R_RENDERSCENE — seals the accumulated scene. `light_styles`
    // (A11) is the operational form of A6's snapshot-at-scene-marks: the
    // sim thread copies `LightStyleTable::colors` into the event so the
    // render-side consumers (`oracle/codemp/renderer/tr_surface.cpp:279,324`,
    // `oracle/codemp/renderer/tr_shade.cpp:1401,1685`,
    // `oracle/codemp/renderer/tr_light.cpp:234-274`) read the frame's
    // snapshot, not the live sim-owned table — 256 bytes
    // ([[u8; 4]; MAX_LIGHT_STYLES], MAX_LIGHT_STYLES = 64). R3 caveat
    // (R2-D5) unchanged: snapshot-vs-live timing verifies against the
    // oracle when the backend consumer lands.
    RenderScene { refdef: TrRefdef, light_styles: [[u8; 4]; MAX_LIGHT_STYLES] },

    // --- 2D draw commands (RC_SET_COLOR / RC_STRETCH_PIC / RC_ROTATE_PIC family) ---
    SetColor([f32; 4]),                             // CG_R_/UI_R_SETCOLOR
    DrawStretchPic { x: f32, y: f32, w: f32, h: f32, s1: f32, t1: f32, s2: f32, t2: f32, shader: ShaderHandle }, // CG_R_/UI_R_DRAWSTRETCHPIC
    DrawRotatePic { x: f32, y: f32, w: f32, h: f32, s1: f32, t1: f32, s2: f32, t2: f32, angle: f32, shader: ShaderHandle },  // CG_R_DRAWROTATEPIC — cgame-only
    DrawRotatePic2 { /* same fields as DrawRotatePic */ },                                                                   // CG_R_DRAWROTATEPIC2 — cgame-only
    DrawString { /* font handle, position, string, style */ },                 // CG_R_/UI_R_FONT_DRAWSTRING

    // --- world-effects / automap tail (RC_WORLD_EFFECTS / RC_AUTO_MAP) ---
    WorldEffectCommand(String),                      // CG_R_WORLDEFFECTCOMMAND — cgame-only
    AutomapElevAdj(f32),                              // CG_R_AUTOMAPELEVADJ — cgame-only

    // Deliberately absent: CG_R_WEATHER_CONTENTS_OVERRIDE. Retail's handler
    // is `//contentOverride = args[1]; return 0;` — a live no-op
    // (`oracle/codemp/client/cl_cgame.cpp:1716-1718`) — not queued (B9).
}
```

Deliberately absent: `AddMiniRefEntityToScene`. Zero call sites for
`CG_R_ADDMINIREFENTITYTOSCENE`/`MiniRefEntity` in `cl_cgame.cpp` —
consistent with DEC-37 ruling 13 ("mini-refentity chain ports the live
pad-and-forward shim only, real chain is `#if 0`"). No `FrameEvent` variant
needed; `trMiniRefEntity_t`/`backEndData_t.miniEntities` are dead per ruling
13, disposed accordingly in the table below. The shim itself is not dead
code to drop, though: it has a live **client-side** caller,
`re.AddMiniRefEntityToScene(ent)` at `oracle/codemp/client/FxSystem.h:189`,
reaching the `#if 1` pad-and-forward arm of
`RE_AddMiniRefEntityToScene` (`oracle/codemp/renderer/tr_scene.cpp:271-317`)
— an R4/cl_* wave ports that shim, it just never becomes a `FrameEvent`.

### A1 disposition table

Every `backEndData_t` field and every `RC_*` render command, classified
**crosses in `FrameData`** (data ends up inside a `FrameEvent` payload) /
**stays render-side** (backend-owned scratch, never crosses the channel) /
**dead** (no live MP consumer per existing DEC-37 rulings).

| Oracle field/command | Cite | Disposition | Justification |
|---|---|---|---|
| `backEndData_t.drawSurfs[MAX_DRAWSURFS]` | `oracle/codemp/renderer/tr_local.h:2264` | **stays render-side** | cull/sort output (a frontend *output*, A1's framing) — the render thread computes this itself from `FrameEvent`s + `RenderAssets`, never a channel payload (ruling 2: cull→sort run render-side) |
| `backEndData_t.dlights[MAX_DLIGHTS]` | `oracle/codemp/renderer/tr_local.h:2266` | **crosses in `FrameData`** | payload of `AddLightToScene`/`AddAdditiveLightToScene` events; `MAX_DLIGHTS=32` becomes the per-draw light-list bound (ruling 7). Append-time overflow (`r_numdlights >= MAX_DLIGHTS`, `oracle/codemp/renderer/tr_scene.cpp:332-334`) is **silently dropped, no warning** — unlike entities/polys below, `RE_AddDynamicLightToScene` prints nothing on overflow; the append reproduces that silence exactly, not a uniform warn-on-drop policy |
| `backEndData_t.entities[MAX_ENTITIES]` | `oracle/codemp/renderer/tr_local.h:2268` | **crosses in `FrameData`** | payload of `AddRefEntityToScene` events. `MAX_ENTITIES=2048` (`TR_WORLDENT = MAX_ENTITIES-1`, `oracle/codemp/cgame/tr_types.h:15`) is the append-time bound: `RE_AddRefEntityToScene` warns (non-`FINAL_BUILD` `Com_Printf`, `oracle/codemp/renderer/tr_scene.cpp:213-219`) and silently drops the entity past the bound — no append, no crash. The `FrameData` append reproduces warn-then-drop, matching `R2-D2`'s general append-validation principle |
| `backEndData_t.miniEntities[MAX_MINI_ENTITIES]` | `oracle/codemp/renderer/tr_local.h:2269` | **dead** | ruling 13 — real mini-refentity chain is `#if 0`; only the pad-and-forward shim is in scope, and it has no live trap (its live caller, `FxSystem.h:189`, is an R4/cl_* concern, not this struct) |
| `backEndData_t.polys`/`polyVerts` (unsized) | `oracle/codemp/renderer/tr_local.h:2270-2271` | **crosses in `FrameData`** | payload of `AddPolyToScene`/`AddPolysToScene`/`AddDecalToScene` events. Bounded at `MAX_POLYS=600`/`MAX_POLYVERTS=3000` (`oracle/codemp/renderer/tr_local.h:2256-2257`, runtime vars `max_polys`/`max_polyverts` default to those); `RE_AddPolyToScene` warns and drops the **remainder** of the call past either bound — the check sits inside its per-poly loop (`for (j = 0; j < numPolys; j++)`, `oracle/codemp/renderer/tr_scene.cpp:133-136`: `r_numpolyverts + numVerts > max_polyverts \|\| r_numpolys >= max_polys`), so a multi-poly `CG_R_ADDPOLYSTOSCENE` call that trips mid-loop has already appended the earlier polys in that same call — same warn-then-drop shape as entities, but per-poly rather than per-call |
| `backEndData_t.commands: renderCommandList_t` | `oracle/codemp/renderer/tr_local.h:2272` | **crosses in `FrameData`** (as the `FrameEvent` enum itself) | the byte-packed command buffer *is* what `FrameData.events: Vec<FrameEvent>` replaces — a typed enum instead of a byte stream, per ruling 2/A1's "ordered event stream, not a field mirror" |
| `RC_END_OF_LIST` | `oracle/codemp/renderer/tr_local.h:2240` | **dead** | an artifact of the byte-packed buffer's termination sentinel; `Vec<FrameEvent>`'s length is the terminator, no enum variant needed |
| `RC_SET_COLOR` | `oracle/codemp/renderer/tr_local.h:2241` | **crosses** | → `FrameEvent::SetColor` |
| `RC_STRETCH_PIC` | `oracle/codemp/renderer/tr_local.h:2242` | **crosses** | → `FrameEvent::DrawStretchPic` |
| `RC_ROTATE_PIC` / `RC_ROTATE_PIC2` | `oracle/codemp/renderer/tr_local.h:2243-2244` | **crosses** | → `FrameEvent::DrawRotatePic`/`DrawRotatePic2` |
| `RC_DRAW_SURFS` | `oracle/codemp/renderer/tr_local.h:2245` | **stays render-side** | this command carries `drawSurfsCommand_t { refdef, viewParms, drawSurfs, numDrawSurfs }` — the refdef/viewParms *inputs* cross as `FrameEvent::RenderScene` (A11 also folds the light-style snapshot into this event), but `drawSurfs` itself is the cull/sort output computed render-side (same reasoning as the field above) |
| `RC_DRAW_BUFFER` | `oracle/codemp/renderer/tr_local.h:2246` | **stays render-side** | `drawBufferCommand_t.buffer` selects a GL draw buffer (front/back) — pure backend/GPU concern, no sim-thread input |
| `RC_SWAP_BUFFERS` | `oracle/codemp/renderer/tr_local.h:2247` | **stays render-side** | presentation timing is a render-thread/GpuResources concern (frame end, not a scene input) |
| `RC_WORLD_EFFECTS` | `oracle/codemp/renderer/tr_local.h:2248` | **crosses** | → `FrameEvent::WorldEffectCommand` |
| `RC_AUTO_MAP` | `oracle/codemp/renderer/tr_local.h:2249` | **crosses (split)** | `AutomapElevAdj` crosses as a `FrameEvent` (drives `g_playerHeight`, unaffected by A10); `InitWireframeAuto` is a sim-side A9 mutation, not a read — it rebuilds `RenderAssets::automap_wireframe` synchronously so it can answer the oracle's `qboolean` validity result (A10); the render *command*'s full struct beyond the bare enum tag (how the backend is told to draw the rebuilt wireframe) gets its targeted oracle read at the first automap wave (A7, `R2-D8`) |
| `subImageCommand_t` | `oracle/codemp/renderer/tr_local.h:2195-2201` | **provisionally dead (no MP trap found)** | no `CG_R_*`/`UI_R_*` trap maps to `RE_SubImage`-style incremental texture upload in the 57-trap census; a `RE_SubImage`/`subImageCommand_t` call-site grep runs before R3 scope-freezes this family (A7, `R2-D8`) |
| `drawBufferCommand_t` | `oracle/codemp/renderer/tr_local.h:2190-2193` | **stays render-side** | frame lifecycle / GL buffer selection, no sim-thread payload |
| `swapBuffersCommand_t` | `oracle/codemp/renderer/tr_local.h:2203-2205` | **stays render-side** | frame lifecycle, no sim-thread payload |
| `endFrameCommand_t` | `oracle/codemp/renderer/tr_local.h:2207-2210` | **stays render-side** | frame lifecycle, no sim-thread payload |

### Seam composition plan (A3)

**`trRefEntity_t` composition is already landed; the sketch below records it,
not a migration plan.**
`crates/mp/renderer/src/tr_local/tr_ref_entity_t.rs` already wraps
`mp_qshared`'s `refEntity_t` by value:

```rust
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::shared::{qboolean, vec3_t};

#[repr(C)]
pub struct trRefEntity_t {
    pub e: refEntity_t,
    pub axisLength: f32,
    pub needDlights: qboolean,
    pub lightingCalculated: qboolean,
    pub lightDir: vec3_t,
    pub ambientLight: vec3_t,
    pub ambientLightInt: i32,
    pub directedLight: vec3_t,
    pub dlightBits: i32,
}

const _: () = assert!(core::mem::offset_of!(trRefEntity_t, e) == 0);
// … per-target-width tail asserts checking only the wrapper's additional
// fields (axisLength .. dlightBits), not refEntity_t's internal layout.
```

matching the oracle's own `refEntity_t e;` composition
(`oracle/codemp/renderer/tr_local.h:94-106`) field-for-field, Raven names
(§D12) — no snake_case renaming, and both `lightingCalculated` and
`dlightBits` present (an earlier draft of this doc's sketch renamed the
tail fields and dropped these two; the landed file above is authoritative,
B3). `mp_renderer` already depends on `mp_qshared`'s `cgame` module — at
least 10 files under `crates/mp/renderer/src/` import
`mp_qshared::common::mp::cgame::*` today (e.g. `tr_ref_entity_t.rs`,
`back_end_data_t.rs`, `tr_mini_ref_entity_t.rs`, `srf_poly_s.rs`), so there
is no Cargo-edge migration left to record (B1). **R2-D6 narrows to
`trRefEntity_t` alone** — see below.

**`trRefdef_t` does NOT compose `refdef_t`, and cannot.**
`oracle/codemp/renderer/tr_local.h:563-598` (`trRefdef_t`) and
`oracle/codemp/cgame/tr_types.h:257-275` (`refdef_t`) diverge in both field
set and order: `refdef_t` carries `viewangles`
(`oracle/codemp/cgame/tr_types.h:261`) and `viewContents`
(`oracle/codemp/cgame/tr_types.h:263`) that `trRefdef_t` lacks entirely;
`trRefdef_t` inserts `frametime` after `time`
(`oracle/codemp/renderer/tr_local.h:570`), `areamaskModified` after
`areamask` (`:575`), and `floatTime` before `text` (`:577`) that `refdef_t`
does not have. A `#[repr(C)] pub base: refdef_t` field could not reproduce
`trRefdef_t`'s actual byte layout — the two structs are not
superset/subset, they are independently shaped. `crates/mp/renderer/src/
tr_local/tr_refdef_t.rs` correctly re-declares the full field list with its
own byte-exact offset asserts (`text` @124, `num_entities` @380, size
448/416 per target width) and stays exactly as landed; no composition
applies here (B2). A3's ratified text originally named both
`trRefEntity_t`/`trRefdef_t`; the field-level proof above showed the
`trRefdef_t` half is layout-impossible, and A3 carries a ledger amendment
(same-day) narrowing it to `trRefEntity_t` alone.

**Single assert layer** (for the `trRefEntity_t` case): `refEntity_t`'s
`size_of`/`offset_of!` block lives once in `mp_qshared` (already there,
proven live at the trap-arg layer); `trRefEntity_t`'s own asserts only
check the wrapper's *additional* fields' offsets (the `e` field's offset
plus the tail), not re-derive `refEntity_t`'s internal layout a second
time — landed exactly this way in `tr_ref_entity_t.rs:29-54`.

## Decisions

**R2-D1** (ruling 3 — state-partition law). `RenderAssets` (CPU,
immutable-after-publish, `Arc`-shared, sim-readable) splits from
`GpuResources` (render-thread-only); `RenderWorld` composes both plus
render-thread-local `FrameState`. `trGlobals_t`'s registries move to
`RenderAssets`; its frontend scratch/counters move to `FrameState`.
`backEndState_t` becomes `FrameState` in full — all 11 fields (`refdef`,
`viewParms`, `ori`, `pc`, `isHyperspace`, `currentEntity`,
`skyRenderedThisView`, `projection2D`, `color2D`, `vertexes2D`,
`entity2D`), not the 5-field partial sketch an earlier draft of this doc
carried (B5); `glstate_t` becomes `GpuResources::gl_state`, a named
placeholder until R4 defines the real wgpu pipeline/bind-group cache — GL
binding concepts have no wgpu equivalent yet (B6). `distanceCull`/
`distanceCullSquared` and `glConfig`, despite living "outside of TR" beside
`glState` in the oracle, are sim-readable, not render-thread-local: two
traps read them synchronously (`CG_R_GETDISTANCECULL`, `CG_R_GETREALRES`),
so they home in `RenderAssets` (B11). Rejected keeping one merged struct
(closer to `tr`'s literal shape): would violate ruling 3's invariant that no
trap query can reach GPU state, since a merged struct has no type-level
separation between the two.

**R2-D2** (A1 — `FrameData` design + disposition table). `FrameData` is an
**ordered event stream** (`Vec<FrameEvent>`), not a field mirror of
`backEndData_t`: `backEndData_t` mixes frontend inputs (dlights, entities,
polys) with `drawSurfs`, a frontend *output* that never crosses the channel
(cull/sort run render-side per ruling 2). The disposition table above is the
per-field/per-command classification (crosses / stays render-side / dead),
now covering every `backEndData_t` field, every `RC_*` tag, and the three
named-but-unlisted command structs (`drawBufferCommand_t`,
`swapBuffersCommand_t`, `endFrameCommand_t`). Rejected a byte-packed buffer
mirroring `renderCommandList_t` literally: would reproduce the oracle's
untyped command-tag mechanism inside safe Rust for no benefit — a typed
enum is both a faithful behavioral mirror (interleaving preserved) and
idiomatic.

**R2-D3** (A2 — four typed arenas + generic handle infra). image/shader/
skin/model each get their own `Arena<T>` (matching the oracle's four
independent `MAX_*` arrays) behind `Handle<K>`, a generation-counted
`(index: u32, generation: u32)` pair with per-kind phantom typing so
cross-kind handle mixups fail to compile. `Handle<K>` implements `Clone`/
`Copy`/`PartialEq`/`Eq`/`Hash` by hand, not `#[derive(...)]` (NB-4): a
derive adds a `K: Trait` bound, but `K` is only ever a marker
(`PhantomData<fn() -> K>`) — the asset structs it's instantiated with
aren't `Copy`, so a derived `Copy` would silently make every `*Handle` type
non-`Copy`, breaking every by-value handle call site. The hand-written impls
have no `K` bound, so `Handle<K>` behaves as an ordinary value key
regardless of what `K` is. `Handle<K>`/`Arena<T>` carry no oracle citation —
new Rust-side infrastructure implementing ruling 11's generation-counted-
handle requirement, the second instance of the `AlignedBytes` justified-
exception precedent (`tr-model.md` `TRM-D4`/ruling 58: a doc-comment states
the justification, no `Source:` line). Rejected one untyped `Handle<u32>`
shared across kinds: defeats the compile-time mixup-catching that is A2's
stated purpose.

**R2-D4** (A5 + A5 amendment + A12 — arena capacity, per-registry failure
semantics, and slot-0 reservation). Shader/skin/model arenas soft-cap at
their oracle `MAX_*` constants (`MAX_SHADERS=16384`, `MAX_SKINS=1024`,
`MAX_MOD_KNOWN=1024`); the image arena is unbounded, matching its real
oracle backing store (`tr_image.cpp`'s `AllocatedImages` std::map — the
`MAX_DRAWIMAGES` check is commented out in retail, Part 1 finding), with
`RenderAssets::image_names` mirroring the lower-cased extension-stripped
key scheme beside it. Registration never returns `Result` — always a
handle. Retail's overflow behavior is per-registry, not uniform (A5
amendment, measured):
- **Shaders** warn (`Com_Printf`) and return the live
  `tr.defaultShader` fallback (`oracle/codemp/renderer/tr_shader.cpp:2758-2761`).
- **Skins** warn and return handle 0 (`oracle/codemp/renderer/tr_image.cpp:3139-3141`).
- **Models** are SILENT in retail — `R_AllocModel` returns `NULL` with no
  print (`oracle/codemp/renderer/tr_model.cpp:614-616`), `RE_RegisterModel`
  returns 0 (`:1044-1045`).

**A12 closes the gap the A5 amendment left open**: what does "handle 0"
*mean* under a generation-counted arena, when the oracle's own index 0 is a
reserved slot, not a null? Each capped arena pre-populates slot 0 at
construction with the registry's oracle default entry — models[0]
`MOD_BAD` (`R_ModelInit`, `oracle/codemp/renderer/tr_model.cpp:1665-1680`),
skins[0] `"<default skin>"` (`R_InitSkins`,
`oracle/codemp/renderer/tr_image.cpp:3324-3332`), shader 0
`tr.defaultShader` (`CreateInternalShaders`,
`oracle/codemp/renderer/tr_shader.cpp:4137-4155`). `Handle { index: 0,
generation: 0 }` IS that live default, and every oracle overflow path
(shaders' `tr.defaultShader`, skins'/models' `return 0`) resolves to it —
`qhandle_t` 0 maps to slot 0 as the identity at the seam, so retail's
render-the-default-on-failure behavior falls out exactly, with no separate
"invalid handle" representation needed for these three arenas. `Arena<T>`
therefore needs no per-registry `fallback` field (an earlier draft of this
doc's `Option<Handle<T>>` field is dropped): overflow on any capped arena
uniformly returns `Handle{0,0}`, because slot 0 already **is** the correct
object in every case. The image arena, uncapped, has no reserved slot — a
failed lookup returns `Option::None`, never a handle.

**Lookup-key structures, per registry** (the fourth piece R2's "registry
arena count and capacity semantics" scope always covered, elaborated here
against the oracle's actual lookup functions rather than left silent). The
arena is *storage*; the lookup key is a separate structure beside it, and
the three non-image registries don't share one shape:
- **Shaders**: `IsShader` (`oracle/codemp/renderer/tr_shader.cpp:3373-3398`)
  matches a stripped name **and** the full `lightmapIndex[MAXLIGHTMAPS]`/
  `styles[MAXLIGHTMAPS]` arrays (`MAXLIGHTMAPS=4`,
  `oracle/codemp/qcommon/qfiles.h:310`) — multiple shaders legitimately
  share one stripped name, differentiated only by those arrays (the
  oracle's own comment at the `R_FindShader` hash walk notes this:
  `oracle/codemp/renderer/tr_shader.cpp:3456-3459`) — **except** default
  shaders, which skip the array compare entirely: `if (!sh->defaultShader)`
  guards it (`oracle/codemp/renderer/tr_shader.cpp:3382`), so a default
  shader matches on the stripped name alone. `RenderAssets
  ::shader_lookup: HashMap<String, Vec<ShaderHandle>>` reproduces that
  shape exactly: a name maps to every candidate sharing it, compared
  per-entry the way `IsShader` walks its hash bucket, guard included.
- **Skins**: `RE_RegisterSkin`'s name walk
  (`oracle/codemp/renderer/tr_image.cpp:3128-3136`) compares the full name
  only (`Q_stricmp`, no stripping, no per-entry array) — plain
  `RenderAssets::skin_lookup: HashMap<String, SkinHandle>`.
- **Models**: `RE_RegisterModel`'s `mhHashTable` walk
  (`oracle/codemp/renderer/tr_model.cpp:1211-1215`) is the same shape as
  skins — plain `RenderAssets::model_lookup: HashMap<String, ModelHandle>`.

Lookup maps live beside their arenas in `RenderAssets` (sim-readable, same
`Arc`), not in a separate structure, and are maintained by the matching
`RenderAssetsSim` registration mutator in the same `Arc::make_mut` call
that inserts into the arena — one publish per registration, not two.

**`tr.lightmaps[MAX_LIGHTMAPS]`'s positional index** (the same scope,
resolved separately from the name maps above because it isn't a name
lookup at all). Folding `tr.lightmaps` into `images` (`R2-D1`'s state
table) only accounted for the name-keyed half of image storage;
`R_FindShader` also reads lightmaps by small integer —
`stage->bundle[0].image = tr.lightmaps[shader.lightmapIndex[0]]`
(`oracle/codemp/renderer/tr_shader.cpp:3543`, three more call sites of the
same shape). `RenderAssets::lightmaps: Vec<ImageHandle>` adds that
positional index beside `images`/`image_names` — populated at level load in
lightmap order, matching `tr.lightmaps[MAX_LIGHTMAPS]`
(`oracle/codemp/renderer/tr_local.h:1364`) index-for-index; the underlying
`ImageAsset` storage still lives once, in `images`.

**NOTE, folded (not a design gap): the `DEDICATED` branch in
`R_FindShader`'s stage-image resolution.** On a dedicated-server build the
oracle skips image I/O entirely — `stage->bundle[0].image = NULL; return
qfalse;` under `#ifdef DEDICATED`, no `R_FindImageFile` call at all
(`oracle/codemp/renderer/tr_shader.cpp:1335-1345`). `crates/mp/renderer` is
already the CPU-only crate ruling 16 scopes into `jampded`'s link (assets,
parse, cull/sort — GPU-free goldens); R3's shader-stage port reproduces the
same short-circuit on that build target rather than attempting texture I/O,
matching the oracle branch exactly. No new decision needed — ruling 16
already settled the crate boundary this branch lives inside.

Fallback *values* are reproduced exactly (rendering-observable — keeps the
differential rigs free of phantom diffs); warnings keep retail's
shader/skin prints plus a port-added, clearly-marked warning on the silent
model overflow (charter interior freedom, a debugging aid with no oracle
counterpart). Rejected uniform `Result`-returning registration, and
rejected a uniform warning-or-silent policy across all three bounded
arenas: both would diverge from retail's actual, per-registry,
individually-observable overflow behavior. Rejected a distinct sentinel
"invalid handle" value separate from slot 0: the oracle has no such value
for these registries — index 0 always resolves to a live object — and
inventing one would be speculative behavior porting-rules §A2 forbids.

**R2-D5** (A6 + A9 + A11 — light styles are synchronous, `RenderAssets`-
adjacent state, snapshotted onto a carrier at scene marks). `R_SetLightStyle`/
`R_GetLightStyle` (oracle names `RE_Set/GetLightStyle`, trap wrappers
`trap_R_Set/GetLightStyle` — `## Raven ground truth`) mutate/read
`LightStyleTable` directly at trap time — not a `FrameData` event, and
**not a field inside `RenderAssets`'s `Arc`** (an earlier draft of this doc
placed `light_styles` inside `RenderAssets`, contradicting `RenderAssets`'s
own immutable-after-publish/Arc-shared framing — B7). A9 settles the
placement: `LightStyleTable` is a separate sim-owned table beside
`RenderAssetsSim`, mutated in place with ordinary `&mut` access (no
`Arc::make_mut`, no publish step) because style writes are frequent, small,
and — unlike registration — never need cross-thread publish coordination
beyond a snapshot at scene-render marks. Style colors are written from
`trap_R_SetLightStyle` call sites (cgame's per-frame light-style update,
`oracle/codemp/cgame/cg_light.c:64`) and read back synchronously at the
same table by the render-side consumers
(`oracle/codemp/renderer/tr_surface.cpp:279,324`,
`oracle/codemp/renderer/tr_shade.cpp:1401,1685`,
`oracle/codemp/renderer/tr_light.cpp:234-274`); unlike scene-composition
traps they mutate a small lookup table rather than accumulating scene
geometry, so nothing about them requires draw-order interleaving with
2D/scene commands.

An earlier draft of this doc annotated `FrameEvent::RenderScene` as the
"snapshot mark" without giving the snapshot an actual carrier field — NB-3
found the render-side consumers had no defined path from the sim-owned
table to the backend. A11 closes it: `RenderScene` gains `light_styles:
[[u8; 4]; MAX_LIGHT_STYLES]` (256 bytes) — the sim thread copies
`LightStyleTable::colors` into the event at scene-render time, so the
backend reads the frame's own snapshot rather than reaching back into
sim-owned state. The Gate-3 dry-run found the carrier still had no
*consumer-side* landing spot — `FrameState` had no field to hold the
snapshot across per-surface backend processing
(`oracle/codemp/renderer/tr_surface.cpp`/`tr_shade.cpp`/`tr_light.cpp` all
read `styleColors` many times per scene, not once).
`FrameState::scene_light_styles: [[u8; 4]; MAX_LIGHT_STYLES]`
(`## Seam definition`) is that landing field: filled once, when the render
thread processes the frame's `RenderScene` event, then read by every R4
tessellation/vertex-building consumer for the rest of that scene — never a
second reach into `LightStyleTable` from render-thread code. R3 caveat
unchanged: the wave porting the backend style consumer verifies
snapshot-vs-live timing against the oracle — this doc fixes the carrier's
*shape* and its consumer-side home, not the precise instant within trap
processing the copy is taken, which is unverified until a real backend
consumer exists. Rejected an event-carried style *update* (as opposed to a snapshot
*read*): would force per-style-change event allocation for a value that
behaves as ambient state everywhere else in the trap surface.

**R2-D6** (A3 — seam composition by value, `trRefEntity_t` only). `trRefEntity_t`
already composes `mp_qshared`'s `refEntity_t` by value
(`crates/mp/renderer/src/tr_local/tr_ref_entity_t.rs`), matching the
oracle's own `refEntity_t e;` composition (`oracle/codemp/renderer/
tr_local.h:94-106`) — landed, not a migration (B1). `trRefdef_t` is
excluded from A3's scope: it is not a superset of `refdef_t` (field set and
order both diverge, `## Seam definition`'s Seam composition plan above), so
composing it by value is layout-impossible; `crates/mp/renderer/src/
tr_local/tr_refdef_t.rs`'s existing independent re-declaration stays as the
correct shape (B2). Rejected extending A3's ratified text to cover
`trRefdef_t`: the two oracle structs are not related by containment, only
by naming convention.

**R2-D7** (ruling 17 — SP/MP mode design). Interior scene/asset types stay
mode-agnostic by construction; SP divergences become edge adapters + quirk
flags, never a second renderer. Concrete divergences and their handling:

- **(a) `viewParms_t.frustum[4]` (MP) vs `frustum[5]` (SP)**
  (`oracle/code/renderer/tr_local.h:556-571` SP vs
  `oracle/codemp/renderer/tr_local.h:629-644` MP). The interior `ViewParms`
  type carries `frustum: [cplane_t; N]` with `N` a mode-associated const
  (`FRUSTUM_PLANES: usize = 4` MP, `5` SP) via a const generic or per-mode
  type alias (`type ViewParms = ViewParmsN<4>;` in `mp_renderer`, `<5>` in
  `sp_renderer`), not a runtime-sized `Vec` — the oracle's count is
  compile-time-fixed per mode. Because: a dynamic-length field would be
  unfaithful to a value the oracle never varies at runtime. The 5th SP
  plane's purpose is not traced this pass; a fresh oracle read is due
  whichever slice first touches SP frustum culling.
- **(b) `or`/`ori` naming** — cosmetic (C++ keyword clash in SP's C source,
  already dodged by MP's own header). Rust uses `ori` on both sides, full
  stop; no adapter needed.
- **(c) RMG/terrain fields** (`distanceCullSquared`, `landScape`,
  `sunSurfaceLight` on MP; `worldDir`, `iNumDeniedShaders`, `saveGameImage`
  on SP) — accretion, not restructuring. `distanceCullSquared` (with
  `distanceCull`) lives in `RenderAssets` — sim-readable, since
  `CG_R_GETDISTANCECULL` reads `distanceCull` synchronously (B11, `R2-D1`).
  `landScape`/`sunSurfaceLight` have no trap reading them in the 57-trap
  census, so they stay frontend-only fields on `FrameState`/`GpuResources`
  at whichever mode's crate owns them — exact home decided at R3 when the
  terrain/RMG slice lands. In every case: separate fields present only on
  the mode's own struct (each mode already gets its own crate per DEC-04) —
  no shared struct needs a quirk flag, the fields simply don't exist on the
  other mode's type.
- **(d) `refEntity_t`/renderfx variants** — the largest single divergence:
  `miniRefEntity_t` MP-only, `refEntityType_t` member-set/order differs,
  renderfx bits (`RF_*`) reassigned at different bit positions between
  modes, `ghoul2` field typed (SP) vs `c_void`-erased (MP). Already
  correctly forked: `crates/sp/qshared/.../ref_entity_t.rs` and
  `crates/mp/qshared/.../ref_entity_t.rs` are independently-declared per
  DEC-04 (duplicate, don't unify), and `R2-D6`'s compose-by-value composes
  `trRefEntity_t` against MP's own qshared `refEntity_t`, not a shared one.
  No new adapter needed — DEC-04's existing per-mode duplication working as
  designed.
- **(e) `stereoFrame_t`: enum (SP) vs int-alias (MP)** — both sides verified
  correct at landing time (`## Raven ground truth` above): SP's
  `stereo_frame_t.rs` is a `#[repr(i32)] enum` against SP's true `typedef
  enum` source; MP's is a `c_int` alias against MP's `typedef int` +
  anonymous-enum source. No fix needed.
- **(f) `refexport_t` deletion applies to SP too, for the same reason**
  (both trees statically link the renderer at the oracle source level,
  neither goes through `Sys_LoadDll`) — consistent application of DEC-37
  ruling 4's own stated rationale, not a new ruling. `crates/sp/renderer/
  src/tr_public/` follows the same deletion `R2-D1`'s MP `refexport_t`
  deletion already applies.
- **(g) `tr_quicksprite/`** — SP has it (2 files); MP's port has no sibling
  directory yet. Resolved by A4: `tr_quicksprite.cpp` is in the retail MP
  compile set and R1's srcglob, simply un-ported (R3 scope), not SP-only.
  Ruling 17's mode-agnostic-interior claim stands unconditionally once R3
  ports MP's copy; the gap today is sequencing — R3 hasn't reached it yet.

**R2-D8** (A7 — standing defers, owners and timing fixed by the ledger).
Three items don't resolve at R2:

- **`FrameData` buffer-recycling mechanics** — owner: R4 backend port.
  Lean default recorded now: a fixed 2-3 `FrameData` buffer pool with an
  explicit return channel, sized once real frame-pipeline depth is known.
  R2 freezes only the event-stream shape (`FrameData { events:
  Vec<FrameEvent> }` above), not the recycling protocol.
- **`RC_AUTO_MAP`'s full oracle command shape** — owner: the first R3/R4
  automap wave. The disposition table splits automap across
  `CG_R_AUTOMAPELEVADJ`/`CG_R_INITWIREFRAMEAUTO`; whether `RC_AUTO_MAP`
  carries a command struct beyond the bare enum tag is confirmed by a
  targeted read of `tr_local.h`/`tr_world_effects`-family backend source at
  that wave.
- **`subImageCommand_t` dead-vs-reachable** — owner: R3 scope-freeze. No
  trap in the 57-census maps to it; before R3 finalizes whether this command
  family is in scope, `RE_SubImage`/`subImageCommand_t` call sites get
  grepped in the oracle to rule out an internal renderer self-call (e.g.
  lightmap sub-uploads at map load) this pass didn't trace.

`Handle<K>`/`Arena<T>`'s no-cite status is resolved immediately, not
deferred: folded into `R2-D3` above (doc-comment citing the `AlignedBytes`
precedent, landed in `## Seam definition`'s code block).

**R2-D9** (A9 — `RenderAssets` mutation path). The sim/registration side
owns `RenderAssetsSim::published: Arc<RenderAssets>` directly — there is no
separate "master" struct (NB-1: an earlier draft of this doc sketched a
two-field `{ master, published }` shape, but mutating a `master` and then
calling `Arc::make_mut` on a *different* `published` field propagates
nothing; `Arc::make_mut` only ever operates on the `Arc` it's handed).
Synchronous mutations (register shader/model/skin, remap_shader, the A10
automap rebuild, the matching `shader_lookup`/`skin_lookup`/`model_lookup`
map update that rides along with each registration, and the rare
`registered` flag flip at registration begin/end) all call
`Arc::make_mut(&mut self.published)` — mutating the existing allocation in
place when the render thread holds no other reference, cloning once and
mutating the clone when it does — and the result becomes visible to the
render thread (`RenderWorld::assets`) at the next frame boundary. No locks:
the render thread's view is immutable within a frame, matching ruling 3's
invariant that render-thread reads never race sim-thread writes.
`RenderAssets: Clone` (and therefore `Arena<T>: Clone`, `T: Clone` for
every asset struct) is required for `Arc::make_mut` to be able to clone —
added to the sketch in `## Seam definition`. `LightStyleTable`
stays A6-adjacent — a separate sim-owned table snapshotted onto
`FrameEvent::RenderScene` at scene marks (A11), not inside the `Arc`
(`R2-D5`). This closes the hole an earlier draft of this doc left open:
`RenderAssets` was labeled immutable-after-publish while also hosting an
`Arena::insert` mutation path and (mistakenly) `light_styles`, with no
decision covering how a synchronous trap reaches published, Arc-shared
state (B7). Rejected interior mutability (`Mutex`/`RwLock` fields inside
`RenderAssets`): ruling 2's "no locks" threading topology already rules
this out, and copy-on-write matches retail's own synchronous-registration
contract (ruling 11) without introducing blocking.

**R2-D10** (A10 — automap init is a sim-side mutation, not a read). The
disposition table and eight-trap table both classified
`CG_R_INITWIREFRAMEAUTO` as a synchronous `RenderAssets` *read* — wrong:
its live arm, `R_InitializeWireframeAutomap`
(`oracle/codemp/renderer/tr_world.cpp:1205-1231`), destroys and rebuilds
the wireframe automap surface list by walking `tr.world->nodes`
(`R_DestroyWireframeMap`/`R_GenerateWireframeMap`) and sets `g_autoMapValid`
— a *mutation* of file-scope renderer state
(`oracle/codemp/renderer/tr_world.cpp:782,784`), not a query, that must
still answer synchronously with the oracle's `qboolean` validity result
(so it cannot become an ordered `FrameEvent` the way other automap traps
do). This is exactly the ruling-3 counterexample class B11 found for
`CG_R_GETDISTANCECULL`/`CG_R_GETREALRES`, but on the mutation side rather
than the read side: NB-2 found it during Gate-2 re-review because the
prior fix round never re-derived the disposition table's automap row
against the trap's actual implementation, only its dispatch site. The fix
routes it through the same A9 publish path as registration:
`RenderAssetsSim::rebuild_automap_wireframe` calls
`Arc::make_mut(&mut self.published)`, rebuilds `automap_wireframe` from
`published.world`, and returns the oracle's `bool` — pure CPU work over
already-sim-readable data (`world`), so ruling 3 stays intact: no trap
query or mutation ever touches `GpuResources`. Rejected leaving it a
`FrameEvent`: an ordered/deferred event cannot produce the synchronous
return value the oracle's own `qboolean` contract requires. Rejected
leaving it classified as a read: `g_autoMapFrame`/`g_autoMapValid` are
genuinely rebuilt, not merely inspected, on every call.

**R2-D11** (error-path shape — no oracle addendum; the renderer is
engine-interior, so it inherits an already-settled convention rather than
inventing one). The renderer is not a VM module — it never crosses the
trap/syscall ABI as a callee the way `jampgame`/`cgame`/`ui` do (ruling 4:
`refexport_t` is deleted, direct calls only). Its oracle `Com_Error`
call sites (`RE_AddRefEntityToScene`'s bad-`reType`
`Com_Error(ERR_DROP, …)`, `oracle/codemp/renderer/tr_scene.cpp:220-222`;
`RE_Set`/`GetLightStyle`'s out-of-range `Com_Error(ERR_FATAL, …)`,
`oracle/codemp/renderer/tr_init.cpp:1431,1442`) therefore route through the
**same** error machinery every other already-ported engine subsystem uses,
not a renderer-specific `Result`/panic scheme: `mp_engine_qcommon::common
::com_error(level: ErrorLevel, msg: String) -> !`
(`crates/mp/engine/qcommon/src/common/error.rs:63-68`) — receiverless,
`panic_any(ComError { level, msg })`, exactly Raven's own `Com_Error` shape
(format + throw, no local recovery). The catch boundary — `catch_unwind` +
`payload.downcast::<ComError>()` — physically lives beside the transcribed
`Com_Frame`/`Com_Init` bodies in `crates/mp/engine/qcommon/src/
common_fns.rs` (`:1082-1083,1241` and `:1354,1573`); `mp_engine_core`'s
`lifecycle.rs` only re-exports thin `&mut Engine` wrappers over those two
fns and says so explicitly ("the `catch_unwind` error boundary … live[s]
WITH the transcribed bodies in `mp_engine_qcommon::common_fns`", `lifecycle
.rs:7-9`) — a renderer porter following this cite should land in
`mp_engine_qcommon`, not `mp_engine_core`. `mp_renderer` is a leaf crate
relative to `mp_engine_qcommon` (no crate-internal `com_error` of its own),
so its call form is the **cross-crate** one — `mp_engine_qcommon::common
::com_error(level, msg)`, exactly `crates/mp/engine/server/src/
sv_init.rs:286-289`'s precedent, not `common_fns.rs`'s own in-crate
`crate::common::com_error(...)` shorthand.

Two tiers, matching what the oracle itself does at each site:
- **Where the oracle validates-then-warns-and-drops** (entity/poly/dlight
  bounds, `## Seam definition`'s append-validation principle above): the
  port validates *before* appending/mutating and skips the operation on
  failure, printing through whatever this crate's print convention turns
  out to be (a body detail, R3 scope) — never `com_error`, because the
  oracle itself never escalates these to `Com_Error`.
- **Where the oracle is fatal** (`Com_Error(ERR_DROP, …)`/`(ERR_FATAL, …)`):
  the port calls `com_error` with the same `ErrorLevel` and an equivalent
  message, diverging exactly as the oracle does — not a `Result` a caller
  could recover from, since the oracle gives none.

Rejected inventing a `Result`-returning renderer error type: every other
ported engine subsystem already resolved this exact question (STATE-Q4 /
`ComError`), and the renderer crossing no VM ABI means it has no seam-level
reason to diverge from that resolution — reusing it is the DEC-37 ruling-1
"interior is free" charter applied to error *plumbing*, not just data
shapes.

## Verification strategy

Unchanged from DEC-37 ruling 15 (layered: TU golden harnesses + draw-list
goldens + perceptual image comparison + shader zoo; CPU/draw-list goldens
gate CI). This doc adds no new verification surface — the `FrameData`/
`RenderAssets` split is validated the same way once R3 wires real call
sites; per A4, DEC-37 bucket assignments (this doc's disposition table
included) are provisional until R3 waves exercise real call sites, with the
specific re-checks tracked as `R2-D8`'s standing defers.

## Slice hooks

R3 (frontend port) consumes this doc's `RenderAssets`/`RenderAssetsSim`
shape directly — the four arenas plus `shader_lookup`/`skin_lookup`/
`model_lookup`/`lightmaps` are what `R_RegisterModel`/`R_RegisterShader`/
`R_RegisterSkin`/image loading populate, under `R2-D4`'s capacity/fallback/
lookup-key semantics, `R2-D9`'s publish path, and `R2-D11`'s error-path
shape for the oracle-fatal cases. R4 (backend port) consumes
`GpuResources`/`FrameEvent` — the event stream is what the render thread
walks to build draw calls, `FrameState::scene_light_styles` is what its
tessellation/vertex-building consumers read per surface (`R2-D5`'s A11
carrier landing), and R4 settles `R2-D8`'s buffer-recycling mechanics.
cgame's plan (after ui) consumes the frozen `tr_types.h` seam set
(`R2-D6`'s composed `trRefEntity_t`) — the "land the assert layer once"
coordination note in `renderer-plan.md:113-114`.

## Open questions

None outstanding.

- **R2-Q2 (per-registry capacity semantics)** — RESOLVED by A5 + the A5
  amendment: `R2-D4`.
- **R2-Q3 (`R_Set/GetLightStyle`: event or synchronous CPU state)** —
  RESOLVED by A6 + A9: `R2-D5`, ownership fixed by `R2-D9`.
- **R2-Q6 (`Handle<K>`/`Arena<T>` no-cite precedent)** — RESOLVED by A7:
  approved as the second `AlignedBytes`/`TRM-D4` justified-exception
  instance, landed in `R2-D3`.
- **R2-Q1 (`FrameData` buffer-recycling mechanics), R2-Q4 (`RC_AUTO_MAP`
  full command shape), R2-Q5 (`subImageCommand_t` dead-vs-reachable)** —
  RESOLVED by A7 as standing defers with owner and timing fixed: `R2-D8`.

No item above is outstanding — each was already resolved before ratification;
the bullets exist for traceability, not as a queue. Zero outstanding
questions is the FROZEN-gate condition (doc-standards), and it holds today,
confirmed across every fix round and the Gate-3 dry-run + delta check (see
the header's gate record) — the remaining step to FROZEN is user sign-off,
not a re-review of this section.
