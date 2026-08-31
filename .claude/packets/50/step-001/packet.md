# Packet gh#50 step-001 - the sub-BSP instance draw side

## Scope

This step makes a sub-BSP instance's brush models render. A mod server registers other maps as instances, cgame draws their submodels as `RT_MODEL` entities, and on this client they are invisible today.

The sim-side half already landed as `e8c175d4` on master: `RenderModels::bmodel_indices` records `(world_index, submodel)` and `r_model_bounds` resolves the owning world through `bmodel_location`. This step delivers the draw side. It gives the renderer one flat surface-index space that spans the main world and every loaded instance, uploads every world's geometry into the one pair of world buffers, teaches `R_AddBrushModelSurfaces` to resolve its owning world and offset into that space, and gives an instance world a way to reach the render thread after the main world already uploaded. It also fixes `RE_GetBModelVerts`, the last main-world-only brush read left after `e8c175d4`.

The step does not add a per-world fog table, a per-world lightmap table, a per-world vertex or index buffer, or a world tag on `WorldSurfaceRef`. It ports no new oracle function, adds no cvar, no `FrameEvent` variant, and no ABI surface. It touches no file under `crates/mp/engine/`, `crates/mp/cgame/`, `crates/mp/ui/`, or `crates/mp/game/`, so the lockstep referee is not a gate here.

Two oracle behaviors in this area are quirks, not defects, and this step preserves both. Row 5 and row 6 below name them, and the lane must not correct either.

## The oracle, cited

### The world is a parameter, the surface reference is a pointer

`RE_LoadWorldMap_Actual(const char *name, world_t &worldData, int index)` (`oracle/codemp/renderer/tr_bsp.cpp:2003`) threads `world_t&` through every load step. The main world is the file static `s_worldData` (`:17`) at index 0, and instance `k` lives in `tr.bspModels[k - 1]` (`oracle/codemp/renderer/tr_local.h:1399-1400`). `MAX_SUB_BSP` is 32 (`oracle/codemp/game/q_shared.h:2025`).

Each world hunk-allocates its own surfaces (`tr_bsp.cpp:1373-1376`), bmodels (`:1431`), fogs (`:1684`), planes, marksurfaces, and nodes. `bmodel_t::firstSurface` is an `msurface_t*` into the owning world's array (`oracle/codemp/renderer/tr_local.h:938-942`, filled at `tr_bsp.cpp:1464`), never a global index. The pointer is the world resolution.

The draw walk hands the pointer straight down: `R_AddWorldSurface(bmodel->firstSurface + i, tr.currentEntity->dlightBits, qtrue)` (`oracle/codemp/renderer/tr_world.cpp:608-610`). The draw surf carries `surfaceType_t *surface` (`oracle/codemp/renderer/tr_main.cpp:1262`), and the backend dispatches `rb_surfaceTable[*drawSurf->surface](drawSurf->surface)` (`oracle/codemp/renderer/tr_backend.cpp:756`). No frontend cull, no sort, no batch break, and no backend arm ever asks which world a surface came from.

### Instance geometry never enters the node walk

`R_AddWorldSurfaces` and `R_RecursiveWorldNode` traverse `tr.world->nodes` alone (`oracle/codemp/renderer/tr_world.cpp:1957`), and so does `R_MarkLeaves`. Instance surfaces reach the draw list only through `R_AddBrushModelSurfaces` off an entity (`oracle/codemp/renderer/tr_main.cpp:1432-1433`). The instance's own nodes, leaves, and PVS are dead weight in the oracle, so the node-side scratch stays sized to the main world.

### Fog: instance worlds do carry it, and the backend still reads the main world

`R_LoadFogs` runs unguarded for every world (`oracle/codemp/renderer/tr_bsp.cpp:2077`). Only entities, the light grid, the light-grid array, the `tr.world` assignment, and RMG init sit inside the `if (!index)` block (`:2086-2098`). An instance world therefore gets a real fog table: `worldData.numfogs = count + 1` and its own `Hunk_Alloc` (`:1683-1684`).

An instance also copies the main world's global fog into its own table: `worldData.fogs[worldData.numfogs] = tr.world->fogs[tr.world->globalFog]; worldData.globalFog = worldData.numfogs; worldData.numfogs++` (`:1689-1697`). That write lands one past the `numfogs` allocation, a one-element hunk overrun.

The four surface parsers give an instance surface a fog number out of that instance numbering: `surf->fogIndex = LittleLong(ds->fogNum) + 1; if (index && !surf->fogIndex && tr.world->globalFog != -1) surf->fogIndex = worldData.globalFog;` (`tr_bsp.cpp:371-375` ParseFace, `:459-463` ParseMesh, `:529-533` ParseTriSurf, `:605-609` ParseFlare).

`R_AddBrushModelSurfaces` does not touch fog. `R_AddWorldSurface` passes `surf->fogIndex` straight into the sort key (`oracle/codemp/renderer/tr_world.cpp:553`, with Raven's own standing `// FIXME: bmodel fog?` at `:431`). The key packs it at `QSORT_FOGNUM_SHIFT` (`oracle/codemp/renderer/tr_main.cpp:1280-1281`, shifts at `oracle/codemp/renderer/tr_local.h:1226-1228`) in five bits, so a fog number saturates at 31 (`R_DecomposeSort`, `tr_main.cpp:1291-1297`).

Every backend fog read is against the main world: `fog = tr.world->fogs + tess.fogNum` at `oracle/codemp/renderer/tr_shade.cpp:1192`, `:1376`, `:1660`, `:1963`, and `oracle/codemp/renderer/tr_shade_calc.cpp:993`. There is no per-surface world pointer at the backend. So an instance surface's fog number indexes the main world's table, and Raven papers over the global-fog case with the equality tests at `tr_shade.cpp:726`, `:1078`, `:1960`, `:2381`.

This answers the issue's open question. Instance worlds do carry fog, their fog numbers do enter the sort key, and the backend still resolves them against the main world's table. The port already reproduces exactly that, so the fog-table shape needs no work. See row 5.

### Lightmaps are one global table, and the last world loaded wins it

`R_LoadLightmaps` zeroes `tr.numLightmaps` only for the main world, `if (&worldData == &s_worldData) { tr.numLightmaps = 0; }` (`oracle/codemp/renderer/tr_bsp.cpp:176-179`). It then sets `tr.numLightmaps = len / (LIGHTMAP_SIZE * LIGHTMAP_SIZE * 3)` for every world (`:191`) and writes `tr.lightmaps[i]` from index 0 (`:240`). An instance load therefore replaces the whole lightmap table.

That replacement does not relight the main world. Every read of the table sits in shader-state creation, where the stage captures the image pointer once at build time and keeps it (`oracle/codemp/renderer/tr_shader.cpp`, the four read sites). A stage built before the instance load holds the main world's lightmap for the rest of the session. The one behavior the clobber does change is a shader built after it, which reads the instance's table. See row 6.

### Dlight bits need no world discriminator

`msurface_t` has no `dlightBits` field (`oracle/codemp/renderer/tr_local.h:872-878` holds `viewCount`, `shader`, `fogIndex`, `data`). The mask lives on each payload: `srfGridMesh_t` (`:754`), `srfSurfaceFace_t` (`:804`), `srfTriangles_t` (`:822`). `R_DlightBmodel` writes it per surface (`oracle/codemp/renderer/tr_light.cpp:78-89`), and `R_AddWorldSurface` ORs more bits in on a repeat visit (`oracle/codemp/renderer/tr_world.cpp:416-427`). Each world owns its payloads, so there is no shared dlight index space in the oracle, and the port's flat `surf_dlight_bits` is the exact twin once it covers every world.

### Entity lighting for an instance reads the main world

`R_AddBrushModelSurfaces` calls `R_SetupEntityLighting` when `pModel->bspInstance` (`oracle/codemp/renderer/tr_world.cpp:585-591`), and that path reads the main world's light grid (`oracle/codemp/renderer/tr_light.cpp:154-204,380,472`). An instance world has no light grid at all, because `R_LoadLightGrid` sits inside the `if (!index)` block. The port already threads `assets` and reaches the main world there (`crates/mp/renderer/src/tr_world.rs:1550-1553`), so this leg is already faithful and needs no change.

### When an instance loads

A `misc_bsp` entity registers a `#`-prefixed name as a `CS_BSP_MODELS` configstring (`oracle/codemp/game/g_misc.c:416-418`, `oracle/codemp/game/g_utils.c:153-156`). cgame init walks those configstrings once and calls `trap_R_RegisterModel(bspName)` for each (`oracle/codemp/cgame/cg_main.c:2308-2324`). `RE_RegisterModel` branches on the `#`, bumps `tr.numBSPModels`, loads into `tr.bspModels[tr.numBSPModels - 1]`, and returns the handle hashed for `*<index>-0` (`oracle/codemp/renderer/tr_model.cpp:1227-1246`). `R_LoadSubmodels` registered those names during the load (`tr_bsp.cpp:1442-1460`).

So instances arrive in a batch at cgame init, after the main world already loaded. The `#` branch has no cache check, so a later registration of the same name would load a second copy and burn another slot. Nothing in the MP tree calls it outside `CG_Init`.

## The port as it stands

### The flat index space

`WorldSurfaceRef` is a kind-tagged `Copy` index-handle enum, five variants over the flat `u32` surface index (`crates/mp/renderer/src/tr_main.rs:273-285`), built by `WorldSurfaceRef::of` (`:291-299`). DEC-43.3 ruling 3 chose that shape, and DEC-43.1 states the flat index is the oracle's own `worldData.surfaces` subscript. This step leaves the enum exactly as it stands.

Four things are parallel to that index today, all sized to the main world alone:

- `WorldGeometry::ranges` (`crates/mp/renderer-gpu/src/pipeline3d.rs:226`), built by `build_world_mesh` from `world.surfaces` (`:334-384`) and read by `WorldGeometry::range` (`:314-319`). An out-of-range index returns `SurfaceRange::EMPTY` (`:213-218`), so the surface silently draws nothing and only bumps `stats.empty_surfaces` (`:2096-2099`). That is the gh#50 symptom.
- `WorldGeometry::cpu_vertices` and `cpu_indices` (`:230,234`), sliced by the same range.
- `WorldWalkScratch::surf_view_count` and `surf_dlight_bits` (`crates/mp/renderer/src/render_state/world_walk_scratch.rs:29`), sized by `set_world` (`:52-54`) through `resize` (`:59-66`).
- `world_surface_grid`, which re-fetches the surface from `assets.world.surfaces` for the patch LOD path (`crates/mp/renderer-gpu/src/pipeline3d.rs:4093-4098`, called at `:2177`).

The only other consumers of the index are `world_ref_index` (`:4080-4088`), the portal path that discards it (`crates/mp/renderer/src/tr_main.rs:755`), and the boot diagnostic that counts kinds (`crates/mp/renderer-gpu/src/ui_host/boot.rs:805-813`).

### The brush walk

`R_AddBrushModelSurfaces` (`crates/mp/renderer/src/tr_world.rs:1515-1646`) unwraps `assets.world` twice (`:1539-1543`, `:1610-1613`) and indexes `world.bmodels[model.bmodel_index]` (`:1543`), `world.surfaces[first..first + num]` (`:1565`), and `scratch.surf_dlight_bits[first + i]` (`:1571-1577`). Every one of those resolves against the main world. The loop passes `surf_index = (first + i) as u32` to `R_AddWorldSurface` (`:1616-1644`).

`R_AddWorldSurface` (`:1342-1425`) takes `surf: &Surface` and `surf_index: u32` as separate parameters. It uses the index only for the two scratch arrays (`:1367-1377`, `:1402-1410`) and for `WorldSurfaceRef::of` (`:1417`). So an offset applied to `surf_index` alone reaches every consumer, and this function needs no change.

### The submodel row loses the world

`BModelEntry` carries `bmodel_index` and `bsp_instance` only (`crates/mp/renderer/src/render_state/bmodel_table.rs:12-20`). `BModelTable::build` fills it from `RenderModels::bmodel_index` (`:57-71`), which discards the world index the map actually stores (`crates/mp/renderer/src/tr_model/render_models.rs:358-362`). The pair survives only in `bmodel_location` (`:364-370`), which `e8c175d4` added for `r_model_bounds` (`crates/mp/renderer/src/tr_model/frontend.rs:309-321`).

`RE_GetBModelVerts` still resolves the main world (`crates/mp/renderer/src/tr_world.rs:627-637`) and would panic on `world.bmodels[idx]` for an instance handle with a higher submodel count. That is the same defect `e8c175d4` closed in `r_model_bounds`.

### Nothing publishes an instance world

The instance `WorldAsset` does reach the render thread as CPU data. `RE_RegisterModel_Actual`'s `#` arm stores it at `assets.bsp_models[index - 1]` (`crates/mp/renderer/src/tr_model/frontend.rs:935-971`), `assets` there is `Arc::make_mut(&mut re.sim.published)` (`crates/mp/engine/client/src/cl_cgame.rs:1893`), and `RE_EndFrame` clones the whole `Arc<RenderAssets>` onto every package (`crates/mp/renderer/src/tr_cmds.rs:388`). The issue's root-cause line saying instance worlds "never cross to the render thread" is wrong about the CPU side.

What never happens is the upload. `WorldGeometry::upload` runs only from `FrameExecutor::set_world` (`crates/mp/renderer-gpu/src/frame_exec.rs:305-309`), reached only when `package.world.take()` yields `Some` (`:360-368`). The one writer of `pending_world` is the `CG_R_LOADWORLDMAP` trap arm (`crates/mp/engine/client/src/cl_cgame.rs:1881-1884`). The `#` registration path sets nothing, so the GPU never sees an instance vertex, the walk scratch stays main-world-sized, and the `BModelTable` the render thread holds was built before the instances registered.

That last point is why the symptom is silence rather than a crash. A stale table hands an instance submodel handle the default row with `bmodel_index = -1` (`crates/mp/renderer/src/render_state/bmodel_table.rs:22-31,87-95`), the brush test at `crates/mp/renderer/src/tr_main.rs:1955` fails, the handle falls to the published model registry, and it resolves `MOD_BAD`.

### Fog and lightmaps in the port

`abi_fogs` is rebuilt every frame from the main world alone (`crates/mp/renderer-gpu/src/frame_exec.rs:742-746`) and `resolve_surface_fog` indexes it by the decoded sort-key number (`crates/mp/renderer-gpu/src/pipeline3d.rs:4060-4077`). That matches the oracle's `tr.world->fogs + tess.fogNum` exactly.

`R_LoadFogs`'s instance clause is already ported, and the port lands the one-past-the-end write legally with a push and cites porting rule 19 (`crates/mp/renderer/src/tr_bsp.rs:1992-2004`). The four parser clauses are ported too (`:2593-2599`, `:2743`, `:2875`, `:3036`).

`R_LoadLightmaps` replaces `assets.lightmaps` wholesale for every world (`crates/mp/renderer/src/tr_bsp.rs:2417,2519`), which is the oracle's own last-world-wins behavior. The port's five reads of that table all sit in shader-state creation and capture the handle at build time (`crates/mp/renderer/src/tr_shader.rs:3883,4287,4506,5602,5734`), mirroring the oracle's four read sites, so a built stage keeps the lightmap it captured.

## Surface contract

### `crates/mp/renderer/src/render_state/render_assets.rs`

Two new methods on `RenderAssets`. They define the flat surface-index space by summation over the worlds it already owns, so there is no stored table to go stale and no new state home.

```rust
/// The first flat surface index world `world_index` owns: `0` for the main world, and the running sum of every
/// earlier world's surface count for `RenderAssets::bsp_models[world_index - 1]`.
/// The oracle needs no such number, because `bmodel_t::firstSurface` is a pointer into the owning world's own array.
///
/// Source: `oracle/codemp/renderer/tr_local.h:938-942`
pub fn world_surface_base(&self, world_index: usize) -> u32

/// The world that owns flat surface index `flat`, with that surface's index inside it.
/// `None` past the last loaded world's surfaces.
pub fn resolve_world_surface(&self, flat: u32) -> Option<(&WorldAsset, usize)>
```

The ordering law both share, stated once in `world_surface_base`'s doc: the main world first, then `bsp_models` in slot order. `build_world_mesh` concatenates in that same order, and the two must never diverge.

A third method that returns the total surface count is deliberately absent, per row 2. Nothing in this step needs one, because `WorldWalkScratch::set_world` takes the instance slice directly and sums it there.

### `crates/mp/renderer/src/render_state/bmodel_table.rs`

One new field on `BModelEntry`, filled from `bmodel_location`:

```rust
    /// The world that owns this submodel: `0` for the main world, `k` for `RenderAssets::bsp_models[k - 1]`.
    /// Raven's `bmodel_t::firstSurface` pointer carries this, so the oracle's row has no twin field.
    ///
    /// Source: `oracle/codemp/renderer/tr_bsp.cpp:1442-1450`
    pub world_index: i32,
```

`BModelEntry::default` sets it to `0`, which is inert because the default row already fails the brush test on `bmodel_index = -1`. `BModelTable::build` keeps its signature and reads `RenderModels::bmodel_location` instead of `bmodel_index`.

### `crates/mp/renderer/src/tr_model/render_models.rs`

`RenderModels::bmodel_index` is removed. After commit 3 its only live caller is `RE_GetBModelVerts` (`crates/mp/renderer/src/tr_world.rs:628`), which commit 5 moves, and its only other callers are the four assertions in this file's test module (`:485`, `:486`, `:493`, `:496`). Those four move to `bmodel_location`.

`model_init` and `hunk_clear` each gain one line clearing `bmodel_indices`, per amendment F2. Both already reset the pool, the hash, and the published blocks around it (`:183-189`, `:225-231`), and the map is the one survivor.

One new private field and one new method, in the `publish_blocks` shape this struct already uses for the model-block handoff:

```rust
    /// Set when a `#`-prefixed registration loaded a new sub-BSP instance, cleared when `RE_EndFrame` republishes the
    /// world generation. The render thread uploads geometry only on a generation, so a late instance needs one.
    worlds_dirty: bool,

    /// Takes the sub-BSP dirty flag, `true` once per batch of instance registrations.
    pub(crate) fn take_worlds_dirty(&mut self) -> bool
```

### `crates/mp/renderer/src/tr_model/frontend.rs`

`RE_RegisterModel_Actual`'s `#` arm sets `rm.worlds_dirty = true` after the instance lands in `assets.bsp_models[slot]` (`:964`). No signature change and no other edit in this file.

### `crates/mp/renderer/src/tr_bsp.rs`

One new statement, per amendment F1. `RE_LoadWorldMap_Actual` truncates `assets.bsp_models` to empty beside its `assets.world` write (`:3791`), on the main-world leg only. No signature change and no other edit in this file.

`assets.bsp_models` is cleared nowhere else but `R_Init` (`crates/mp/renderer/src/tr_init.rs:1606`), while a level change resets `num_bsp_models` alone (`crates/mp/renderer/src/tr_model/cached_model_binary.rs:738`). Without the truncation the previous map's instance worlds stay in the `Vec`, and this step's flat index space would then span them forever on a mod-server map rotation.

### `crates/mp/renderer/src/tr_cmds.rs`

`RE_EndFrame` keeps its signature. It gains one block above the sink match, beside the existing `publish_blocks` drain (`:354-358`): when `rm.take_worlds_dirty()` returns `true`, it overwrites `*pending_world` with a fresh `WorldGeneration` built from `sim.published.world.clone()` and `BModelTable::build(rm)`.

The overwrite is correct even on a frame that already holds a map-load generation, because that generation's `BModelTable` was built before the instance registrations and is the stale one. `pending_world` is an `Option` the package `take`s, so a batch of instance registrations between two frames coalesces into one upload.

The same-frame interleave is proven, per amendment F3. `RE_LoadWorldMap_Actual` writes `assets.world` inside the load (`crates/mp/renderer/src/tr_bsp.rs:3791`), before the trap arm builds the first generation (`crates/mp/engine/client/src/cl_cgame.rs:1881-1884`). The drain sits above the sink match, so the generation it rebuilds and the package's own `Arc::clone` of the published registry read the same post-registration state. A map load and a batch of instance registrations in one frame therefore cross together and agree.

### `crates/mp/renderer/src/render_state/world_walk_scratch.rs`

```rust
/// Sizes the surface marks to the main world and every loaded instance together, and the node marks to the main world
/// alone.
/// Only the main world's nodes are ever walked: `R_RecursiveWorldNode` and `R_MarkLeaves` traverse `tr.world->nodes`,
/// and instance surfaces reach the draw list through `R_AddBrushModelSurfaces` alone.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:1957`, `oracle/codemp/renderer/tr_main.cpp:1432-1433`
pub fn set_world(&mut self, world: &WorldAsset, instances: &[WorldAsset])
```

`resize` keeps its signature and its body.

### `crates/mp/renderer/src/tr_world.rs`

`R_AddBrushModelSurfaces` keeps its signature. Inside it:

- The two `assets.world` unwraps become an owning-world resolve on `model.world_index`, the `r_model_bounds` shape (`crates/mp/renderer/src/tr_model/frontend.rs:309-321`).
- `let base = assets.world_surface_base(model.world_index as usize);` and every flat index becomes `base + (first + i) as u32`.
- The `DlightBmodel` snapshot reads `scratch.surf_dlight_bits[base as usize + first + i]`.
- The `R_SetupEntityLighting` call under `bsp_instance` is untouched, per the oracle section above.

`RE_GetBModelVerts` keeps its signature and resolves through `models.bmodel_location(bmodel_index)`, the same two-arm world pick.

### `crates/mp/renderer-gpu/src/pipeline3d.rs`

```rust
/// Packs the main world's surfaces and then every loaded instance's, in `RenderAssets::bsp_models` slot order, into one
/// pair of buffers.
/// The concatenation order is the flat index space `RenderAssets::world_surface_base` computes, and the two must agree.
pub fn upload(gpu: &Gpu, world: &WorldAsset, instances: &[WorldAsset]) -> WorldGeometry

pub fn build_world_mesh(
    world: &WorldAsset,
    instances: &[WorldAsset],
) -> (Vec<WorldVertex>, Vec<u32>, Vec<SurfaceRange>)
```

`world_surface_grid` keeps its signature and resolves the flat index through `RenderAssets::resolve_world_surface`. Nothing else in this file changes: `WorldGeometry`'s fields, `SurfaceRange`, `range`, `surface_count`, `world_ref_index`, `collect_world_surface`, `resolve_surface_fog`, the bind-group cache, and the pass structure all stand.

### `crates/mp/renderer-gpu/src/frame_exec.rs`

```rust
pub fn set_world(
    &mut self,
    gpu: &Gpu,
    world: &WorldAsset,
    instances: &[WorldAsset],
    bmodels: BModelTable,
)
```

`execute_package` passes `&package.assets.bsp_models`. `drop_world` keeps its signature and its body. `abi_fogs` in `render_world` is untouched.

### `crates/mp/renderer-gpu/tests/world_golden.rs`

One new scene and one new test, in the shape of `golden_world_duel1` (`:424-431`):

```rust
#[test] #[ignore] fn golden_world_subbsp_ffa2()
```

It boots `mp/ffa2` as the main world, registers `#mp/duel1` through `boot::register_model`, adds the returned `*1-0` handle as an `RT_MODEL` entity, and renders one frame at the frozen clock. Row 8 holds its bless procedure and its defect conditions.

### Fixtures

One new PNG under `crates/mp/renderer-gpu/tests/goldens/`: `world_subbsp_ffa2.png`.

Anything not on this list is out of scope, and the agent must not add it. No new third-party crate, because a dependency of the DEC-49 kind is a user ruling and this packet may never grant one. No per-world fog table, no per-world lightmap table, no per-world vertex or index buffer, no world tag on `WorldSurfaceRef`, no change to `DrawSurf`, `SurfaceGeometry`, `R_AddDrawSurf`, `R_DecomposeSort`, or the sort key. No node-side or PVS work for an instance world. No port of the `tess.fogNum == tr.world->numfogs` equality tests, which sit in `crates/mp/renderer/src/tr_shade.rs` deferral notes and belong to the fog wave. No change to any file under `crates/mp/engine/`, `crates/mp/cgame/`, `crates/mp/ui/`, `crates/mp/uishared/`, or `crates/mp/game/`. No WGSL shader change. Every committed fixture except the one new PNG is read-only.

## Open rows

**Row 1 - the fix shape (user ruling, ratified as amended 2026-08-30).** **Ratified: shape (a), append with offsets.** The grounds, as proven.

The oracle's draw chain is pointer-carried and holds no world id anywhere. `bmodel_t::firstSurface` is an `msurface_t*` (`oracle/codemp/renderer/tr_local.h:938-942`), the walk hands that pointer straight down as `R_AddWorldSurface(bmodel->firstSurface + i, ...)` (`oracle/codemp/renderer/tr_world.cpp:608-610`), the sort key packs the shader, the entity, the fog, and the dlight bit and nothing else (`oracle/codemp/renderer/tr_main.cpp:1277-1283`), and the backend dispatches `rb_surfaceTable[*drawSurf->surface](drawSurf->surface)` (`oracle/codemp/renderer/tr_backend.cpp:755`). Cull, sort, batch break, and every backend arm are world-agnostic, and the only world-keyed read in the whole path is the fog lookup, which reads the main world unconditionally (`oracle/codemp/renderer/tr_shade.cpp:1192`). A world tag on `WorldSurfaceRef` would carry information the oracle deliberately does not carry.

DEC-43.3 ruling 3 chose a kind-tagged `Copy` index-handle enum over the flat `u32` surface index, and DEC-43.1 states flat-index addressing preserves the oracle's one surface-index space. Shape (a) leaves that enum untouched. Shape (b) contradicts both rulings, so it is a DEC divergence and needs its own ruling row on top of this one.

The backend binds the world vertex and index buffers once for the whole pass (`crates/mp/renderer-gpu/src/pipeline3d.rs:1601,1615`), and the item list is sorted by shader, so worlds interleave through it. Shape (a) leaves that single bind pair as it stands. Per-world buffers would force a rebind per item and break the one-pass shape.

The parallel tables extend cleanly. `WorldGeometry::ranges`, `cpu_vertices`, `cpu_indices`, `surf_view_count`, and `surf_dlight_bits` are all flat vectors, and the oracle keeps dlight bits on the per-world payload with no shared index space (`oracle/codemp/renderer/tr_local.h:754,804,822`), so a longer flat vector is the exact twin.

The cost of (a) is one number. `R_AddBrushModelSurfaces` and `world_surface_grid` each need the owning world resolved, and everything downstream stays flat.

**Row 2 - the flat-index resolver's home (mechanical, ratified as amended 2026-08-30).** **Ratified: two methods on `RenderAssets`, `world_surface_base` and `resolve_world_surface`, computing the base by summation with no stored table.** `RenderAssets` already owns both `world` and `bsp_models`, so the base is a pure read of its own fields and can never go stale. The sum runs over at most 33 worlds, and `world_surface_grid` is the only per-surface caller. The alternative is a `Vec<u32>` of bases stored on `WorldWalkScratch` beside the marks, which is one more thing that must be rebuilt in lockstep with the upload.

The amendment strikes the drafted third method, `total_world_surfaces`. It has no caller anywhere in this packet, because `WorldWalkScratch::set_world` takes `instances: &[WorldAsset]` and sums them itself. The ordering law stays stated once, in `world_surface_base`'s doc.

**Row 3 - the late-arrival republish (user ruling, ratified as amended 2026-08-30).** An instance world registers after the main world uploaded, and nothing signals the render thread. **Ratified: a `worlds_dirty` flag on `RenderModels`, set by the `#` arm of `RE_RegisterModel_Actual` and drained in `RE_EndFrame`, which then rebuilds the whole `WorldGeneration` and overwrites `pending_world`.**

The evidence. `pending_world` is an `Option` the package `take`s (`crates/mp/renderer/src/tr_cmds.rs:392`), so the batch of instance registrations cgame fires at init coalesces into exactly one geometry upload. The flag copies `RenderModels::publish_blocks`, the dirty-flag drain this same function already runs two lines earlier (`:355-358`). The whole change stays inside `mp_renderer`, so no engine crate is touched and the lockstep referee stays off the gate list. And it covers any caller of `RE_RegisterModel`, including the ui dispatcher, not just the cgame arm.

The alternative is a second `pending_world` write in the `CG_R_REGISTERMODEL` trap arm (`crates/mp/engine/client/src/cl_cgame.rs:1893`), beside the existing `CG_R_LOADWORLDMAP` write. That duplicates the writer, misses `UI_R_REGISTERMODEL` (`crates/mp/engine/client/src/cl_ui.rs:1232`), and pulls an engine crate into the write scope.

A rider on the ruling: the drain overwrites `pending_world` rather than filling it only when empty. On a frame that loaded a map and then registered instances, the map load's generation carries a `BModelTable` built before those registrations, so the pending generation is the stale one and the overwrite is the correct read.

The amendment states the gate honestly. Commit 4's battery proves inertness and nothing more. No automated test in this workspace reaches `RE_EndFrame` at all, which all five golden test files record verbatim (`crates/mp/renderer-gpu/tests/world_golden.rs:294` and its four siblings), and each one drains the published model blocks by hand instead. The row-8 golden hands the world to the executor directly, so it does not exercise the drain either. Live play on a mod server that loads instances is the verification, and the finished file records the missing automated coverage as an open gap.

**Row 4 - the surface count the walk scratch covers (mechanical).** **Proposed default: surfaces sized to the main world plus every instance, nodes sized to the main world alone.** `R_RecursiveWorldNode` and `R_MarkLeaves` traverse `tr.world->nodes` only (`oracle/codemp/renderer/tr_world.cpp:1957`), and instance surfaces reach the draw list through `R_AddBrushModelSurfaces` alone (`oracle/codemp/renderer/tr_main.cpp:1432-1433`), so an instance's nodes and PVS are dead weight in the oracle too.

**Row 5 - the fog-table shape (mechanical, this is the issue's open question).** **Proposed default: no change at all.** The oracle reads above settle it. `R_LoadFogs` runs for every world (`oracle/codemp/renderer/tr_bsp.cpp:2077`), an instance surface can carry a fog number from its own world's numbering (`:371-375` and the three sibling parsers), and every backend read is `tr.world->fogs + tess.fogNum` (`oracle/codemp/renderer/tr_shade.cpp:1192,1376,1660,1963`, `oracle/codemp/renderer/tr_shade_calc.cpp:993`). The port already reproduces that pairing: `abi_fogs` comes from the main world alone (`crates/mp/renderer-gpu/src/frame_exec.rs:742-746`) and `resolve_surface_fog` indexes it by the decoded number (`crates/mp/renderer-gpu/src/pipeline3d.rs:4060-4077`). An out-of-range number already drops to `None` there rather than reading past the list, which is the defined answer to a case C leaves open. Nothing to build.

**Row 6 - the lightmap clobber (mechanical, ratified as amended 2026-08-30).** **Ratified: no code change, with the drafted consequence corrected.** `R_LoadLightmaps` zeroes the counter only for the main world (`oracle/codemp/renderer/tr_bsp.cpp:176-179`) and then writes `tr.numLightmaps` and `tr.lightmaps[0..n]` for every world (`:191,240`), so an instance load does replace the whole table. The port matches with `assets.lightmaps = lightmaps` (`crates/mp/renderer/src/tr_bsp.rs:2519`).

The drafted claim that the main world then relights is false. Every read of the table sits in shader-state creation, where the stage captures the image handle at build time (`crates/mp/renderer/src/tr_shader.rs:3883,4287,4506,5602,5734`), mirroring the oracle's four `tr_shader.cpp` read sites. A stage built before the instance load keeps the lightmap it captured, so the main world keeps its own lighting.

The one faithful quirk that remains: a shader built after the clobber reads the instance's table, in the port as in the oracle. Porting rule 20 keeps it. The drafted pause trigger about a main-world lighting change is deleted, and row 8's defect conditions are recast on this reading.

**Row 7 - `RE_GetBModelVerts` (mechanical).** **Proposed default: in scope, as its own commit.** It is the last main-world-only brush read after `e8c175d4`, it takes the same two-arm resolve through `bmodel_location`, and an instance handle whose submodel index exceeds the main world's `bmodels` length panics there today. Leaving it open would leave the ticket half closed.

**Row 8 - the live gate (user ruling, ratified as amended 2026-08-30).** **Ratified: one new world golden, `world_subbsp_ffa2.png`, plus unit tests on the index space, with the defect conditions recast on row 6.**

The golden boots `mp/ffa2`, registers `#mp/duel1` through `boot::register_model`, and draws the returned `*1-0` handle as an `RT_MODEL` entity. Both maps already back committed goldens, so no new asset is needed. It runs through the step-007 bless procedure: run once with `JKA_GOLDEN_BLESS=1`, run again without it to confirm the byte-identical pass, then STOP before the commit that carries the PNG so the user looks at the image.

Named defect conditions. The correct image is the `world_ffa2.png` scene with the main world rendering identically, plus instance geometry that was not there before. An image identical to `world_ffa2.png`, lighting included, means the instance submodel drew nothing, which is the bug this step closes. Any main-world change at all is a defect, because a built stage keeps its captured lightmap, per row 6. A panic in `world.bmodels` or `surf_dlight_bits` means an index escaped its offset.

The golden claims no coverage of commit 4. It hands the loaded world to the executor directly through `FrameExecutor::set_world`, so the `RE_EndFrame` drain never runs in it. Row 3 holds that gap.

The known cost, stated plainly: the instance submodel draws at `mp/duel1`'s own world coordinates inside `mp/ffa2`, so the lane may need to hunt a camera and an entity origin that put the geometry in frame. That hunt is the row's real content, and it is why this row is a ruling and not mechanical.

The unit tests are cheap and run in CI with no assets: `world_surface_base` and `resolve_world_surface` over a synthetic `RenderAssets` with a main world and two instances, and a `build_world_mesh` case proving the concatenated ranges match the bases.

The alternative is unit tests alone. They prove the index arithmetic but not that a sub-BSP brush model reaches the screen, which is the whole ticket.

**Row 9 - the removal of `RenderModels::bmodel_index` (mechanical, ratified as amended 2026-08-30).** **Ratified: remove it in commit 5, which takes its last live caller (`crates/mp/renderer/src/tr_world.rs:628`), and move the four assertions in the file's test module (`crates/mp/renderer/src/tr_model/render_models.rs:485,486,493,496`) to `bmodel_location`.** After commits 3 and 5 nothing else calls it, and a second accessor that silently drops the world index is the exact shape that caused this ticket. The amendment corrects the drafted count from three to four.

## Pause triggers, named for this step

- Any committed fixture other than the new PNG moves. STOP. No commit in this bundle changes a draw for a session with no instance loaded, because the concatenation of a main world and an empty instance list is the main world.
- The lightmap table looks like it should be per world. STOP, per row 6. An instance load replaces the table in the oracle too, and a stage built before it keeps the handle it captured.
- The main world renders differently in the row-8 golden. STOP. That is a defect, not the lightmap clobber, because a built stage keeps its captured lightmap.
- `assets.bsp_models` looks like it should survive a map change. STOP, per amendment F1. A stale instance world would sit inside the flat index space for the rest of the session.
- A per-world fog table looks necessary. STOP, per row 5. The backend resolves every fog number against the main world in the oracle too.
- The instance world's nodes, leaves, or PVS look like they need marks. STOP, per row 4. The oracle never walks them.
- `WorldSurfaceRef` looks like it should carry a world. STOP, per row 1 and DEC-43.3. That is shape (b), and it is not this step.
- A trap arm under `crates/mp/engine/client/` looks like it needs an edit. STOP, per row 3. The republish lands inside `mp_renderer`.
- The golden's instance geometry cannot be framed at any camera. STOP and report before blessing an image that proves nothing.
- The one-past-the-end fog write or the `numfogs` equality tests look like open work. STOP. The first is already ported legally (`crates/mp/renderer/src/tr_bsp.rs:1992-2004`) and the second belongs to the fog wave.
- Verification is `cargo build` or `cargo check` plus the golden suites, never rust-analyzer, which is stale in this workspace.

## Commit bundle

The full gate battery, named once and referenced per commit. Every golden run is serial with `--test-threads=1`, each as one foreground command with a long timeout. Two engine boots in parallel threads crash in the GPU init path, and the world-golden pk3 inflate aborts without it.

- `cargo build --workspace`. An intermediate commit may carry warnings, and the bundle's final state must build with zero warnings.
- `cargo test --workspace -- --test-threads=1`.
- `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`, all four world goldens byte-identical.
- `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`, every scene golden byte-identical.
- `cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`, byte-identical.
- `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`, byte-identical.
- `cargo test -p mp_renderer_gpu --test hud_golden -- --test-threads=1` and the same with `--ignored`, both byte-identical.

The lockstep referee is not required, because no commit touches `mp_game`, the server, or any `jampded` link-set crate.

1. **The flat surface-index space.** `RenderAssets::world_surface_base` and `resolve_world_surface`, with their unit tests. No caller yet, so no behavior changes. Files: `crates/mp/renderer/src/render_state/render_assets.rs`. Subject: `feat(gh#50 s001): the flat world surface index space`. Gates: the full battery, all twenty committed fixtures byte-identical.

2. **Every world's geometry uploads into the one buffer pair.** `build_world_mesh` and `WorldGeometry::upload` take the instances, `WorldWalkScratch::set_world` sizes the surface marks to the total, `FrameExecutor::set_world` gains the parameter, and the six call sites pass it. The `assets.bsp_models` truncation of amendment F1 lands here too, because it defines what the concatenation may hold. With no instance registered the concatenation is the main world alone. Files: `crates/mp/renderer-gpu/src/pipeline3d.rs`, `crates/mp/renderer-gpu/src/frame_exec.rs`, `crates/mp/renderer/src/render_state/world_walk_scratch.rs`, `crates/mp/renderer/src/tr_bsp.rs`, `crates/mp/renderer-gpu/src/ui_host/boot.rs`, `crates/mp/renderer-gpu/src/bin/world_harness.rs`, `crates/mp/renderer-gpu/tests/world_golden.rs`, `entity_golden.rs`, `ghoul2_vertex_golden.rs`. Subject: `feat(gh#50 s001): the world geometry spans every loaded world`. Gates: the full battery, all twenty fixtures byte-identical, which is the proof the concatenation is inert with no instance loaded.

3. **The submodel row carries its owning world.** `BModelEntry::world_index`, `BModelTable::build` through `bmodel_location`, `R_AddBrushModelSurfaces` resolving the owning world and offsetting every flat index, and `world_surface_grid` resolving through `resolve_world_surface`. Files: `crates/mp/renderer/src/render_state/bmodel_table.rs`, `crates/mp/renderer/src/tr_world.rs`, `crates/mp/renderer-gpu/src/pipeline3d.rs`. Subject: `feat(gh#50 s001): the brush walk resolves its owning world`. Gates: the full battery, all twenty fixtures byte-identical, because a main-world submodel has base zero.

4. **The instance world reaches the render thread.** `RenderModels::worlds_dirty` and `take_worlds_dirty`, the flag set in `RE_RegisterModel_Actual`'s `#` arm, and the drain in `RE_EndFrame` that rebuilds the generation. Files: `crates/mp/renderer/src/tr_model/render_models.rs`, `crates/mp/renderer/src/tr_model/frontend.rs`, `crates/mp/renderer/src/tr_cmds.rs`. Subject: `feat(gh#50 s001): a late sub-BSP world republishes its generation`. Gates: the full battery, all twenty fixtures byte-identical. Per row 3 that battery proves inertness alone: no automated test reaches `RE_EndFrame`, so live play verifies the drain and the finished file records the gap.

5. **`RE_GetBModelVerts` resolves its owning world.** The row-7 fix, the removal of `RenderModels::bmodel_index` with the four test moves of row 9, and the two `bmodel_indices` teardown lines of amendment F2. Files: `crates/mp/renderer/src/tr_world.rs`, `crates/mp/renderer/src/tr_model/render_models.rs`. Subject: `fix(gh#50 s001): RE_GetBModelVerts reads the owning world`. Gates: the full battery, all twenty fixtures byte-identical.

6. **The sub-BSP world golden.** `golden_world_subbsp_ffa2` and its PNG, after the row-8 STOP. Files: `crates/mp/renderer-gpu/tests/world_golden.rs` and the new PNG. Subject: `test(gh#50 s001): the sub-BSP instance golden`. Gates: the full battery, with the new golden green at tolerance zero and the other twenty byte-identical.

7. **The finished file**, per the packet skill: assumptions and choices keyed to their commits, deviations or the word "none", the commit list with gate results, and open gaps. File: `.claude/packets/50/step-001/finished.md`. Subject: `process(gh#50 s001): finished file`.

Every commit uses `--no-gpg-sign`, a heading subject, an STE body, and no trailer of any kind: no `Co-Authored-By`, no generated-with footer. Gate results are written as plain sentences inside the body, and a gate paragraph opens with prose, so no line parses as a git trailer.

## Write scopes

Branch `gh50-step-001-sub-bsp-draw`, cut from `master`. A worktree builder runs `git merge master --no-gpg-sign` as its first act.

- `crates/mp/renderer/src/render_state/render_assets.rs` - the two index-space methods and their tests only.
- `crates/mp/renderer/src/render_state/bmodel_table.rs` - `world_index` and the `build` read only.
- `crates/mp/renderer/src/render_state/world_walk_scratch.rs` - `set_world` only.
- `crates/mp/renderer/src/tr_world.rs` - `R_AddBrushModelSurfaces` and `RE_GetBModelVerts` only.
- `crates/mp/renderer/src/tr_model/render_models.rs` - `worlds_dirty`, `take_worlds_dirty`, the `bmodel_index` removal, the four test moves, and the two `bmodel_indices` teardown lines in `model_init` and `hunk_clear`.
- `crates/mp/renderer/src/tr_model/frontend.rs` - the one flag write in the `#` arm only.
- `crates/mp/renderer/src/tr_bsp.rs` - the `assets.bsp_models` truncation beside the `assets.world` write (`:3791`) only.
- `crates/mp/renderer/src/tr_cmds.rs` - the `RE_EndFrame` drain block only.
- `crates/mp/renderer-gpu/src/pipeline3d.rs` - `build_world_mesh`, `WorldGeometry::upload`, `world_surface_grid`, and the imports.
- `crates/mp/renderer-gpu/src/frame_exec.rs` - `set_world` and its `execute_package` call only.
- `crates/mp/renderer-gpu/src/ui_host/boot.rs`, `crates/mp/renderer-gpu/src/bin/world_harness.rs` - edit-only, to pass the new arguments.
- `crates/mp/renderer-gpu/tests/world_golden.rs` - the new argument, plus `golden_world_subbsp_ffa2` and its scene.
- `crates/mp/renderer-gpu/tests/entity_golden.rs`, `ghoul2_vertex_golden.rs` - edit-only, to pass the new argument. Any other caller `cargo check` shows broken by the same signatures is in scope on the same edit-only terms.
- `crates/mp/renderer-gpu/tests/goldens/world_subbsp_ffa2.png` - new, blessed under the row-8 STOP.
- `.claude/packets/50/step-001/` for `finished.md`, for session-directed `packet.md` tail appends, and for the vet's `vet.md`.

Everything else is read-only, including `oracle/`, every file under `crates/mp/engine/`, `crates/mp/cgame/`, `crates/mp/ui/`, `crates/mp/uishared/`, `crates/mp/game/`, `crates/sp/`, every WGSL shader, every other committed fixture, and `~/Developer/jka/` beyond read-only asset reads. Source files change through the Edit tool only.

## Disposition

After a clean lane-review: open the pull request from `gh50-step-001-sub-bsp-draw` into `master` and merge it on GitHub with a merge commit, per DEC-67. Never squash, and never commit on master. The session never pushes or opens the pull request unprompted. It prepares the branch, asks, and the user rules on the push and on the merge.

## Amendments

**2026-08-30 - the ratification walk closed all nine open rows.** The audit is at `.claude/packets/50/step-001/audit.md` (`06cdef92`). Rows 4, 5 and 7 are ratified as proposed and keep their drafted text. Rows 1, 2, 3, 6, 8 and 9 are ratified as amended, and each row above carries its folded text.

- Row 1, the fix shape: shape (a) stands. The "eight sites against eleven" blast-radius sentence is struck as unsupported, and the proven pointer-carried facts replace it. DEC-43.3 ruling 3's handle is a kind-tagged `Copy` index-handle enum over the flat `u32` index, not a bare `u32`, and the packet's phrasing is corrected. Shape (a) leaves both that enum and the backend's single world bind pair untouched.
- Row 2, the resolver home: two methods, not three. `total_world_surfaces` is struck from the surface contract and from commit 1, because it has no caller anywhere in this packet.
- Row 3, the late-arrival republish: the flag and the drain stand, and the no-gate finding is now explicit. Commit 4's battery proves inertness alone, no automated test reaches `RE_EndFrame`, live play verifies the drain, and the finished file records it as an open gap.
- Row 6, the lightmap clobber: no code change, and the consequence claim is corrected. Every read of `assets.lightmaps` sits in shader-state creation and captures the handle at build time, so the main world keeps its own lighting after an instance load. The "will light differently" text and the pause trigger built on it are deleted. The one faithful quirk that remains is a shader built after the clobber reading the instance's table.
- Row 8, the live gate: the golden and the unit tests stand, with the defect conditions recast per row 6. Any main-world change is now a defect. The row claims no coverage of commit 4, because the golden hands the world to the executor directly. The camera-hunt cost paragraph stands.
- Row 9, the `bmodel_index` removal: the count is four test assertions, not three, and the removal lands in commit 5 with its last live caller.

**2026-08-30 - Row A, the vet's guard rail, ratified.** A post-walk vet returned four findings. Two take code and land inside the existing bundle, and two are recorded.

1. **F1, high. Stale instance worlds survive a map change.** `assets.bsp_models` is cleared only in `R_Init` (`crates/mp/renderer/src/tr_init.rs:1606`), and a level change resets `num_bsp_models` alone (`crates/mp/renderer/src/tr_model/cached_model_binary.rs:738`) while the `Vec` persists. Under this step the previous map's instance worlds would then sit inside the flat index space forever on a mod-server map rotation. The fix truncates `assets.bsp_models` when a new main world loads, beside the `assets.world` write (`crates/mp/renderer/src/tr_bsp.rs:3791`). It lands in commit 2, and the write scopes carry the site.
2. **F2, medium. A reused handle can panic after a video restart.** `bmodel_indices` is never cleared, while `model_init` and `hunk_clear` reset the pool, the hash, and the published blocks around it (`crates/mp/renderer/src/tr_model/render_models.rs:183-189,225-231`). A reused handle carrying a stale `(world, submodel)` entry would then resolve into an empty `bsp_models` and panic. The fix clears the map at both teardowns, two lines, folded into commit 5. The write scopes carry it.
3. **F3, record only. The same-frame interleave is correct.** `RE_LoadWorldMap_Actual` writes `assets.world` inside the load (`crates/mp/renderer/src/tr_bsp.rs:3791`), before the trap arm builds the generation (`crates/mp/engine/client/src/cl_cgame.rs:1881-1884`). The drain sits above the sink match, so the rebuilt generation and the package's `Arc::clone` read the same post-registration state. The proof is folded into the `tr_cmds.rs` contract section.
4. **F4, record only. The golden's registration window is safe by timing, not by construction.** cgame registers its instances at `CG_Init`, before its first draw frame, so the walk scratch is always sized before an instance surface can reach it. A future caller that registered an instance and drew in the same frame would panic in the walk scratch. The lane's finished file carries this in one sentence.

**2026-08-30 - Row 8, the bless. BLESSED, user-typed.** The user reviewed `world_subbsp_ffa2.png` beside `world_ffa2.png` and blessed the image. 96,309 of 480,000 pixels differ from the ffa2 baseline, 20.1 percent, inside one bounding box at x 391-799, y 63-339, and every pixel outside that box is byte-identical. The camera recipe: the fixture keeps the `golden_world_ffa2` eye and angles, registers `#mp/duel1`, and places the instance's own bounding-box near face 64 units in front of the eye with its centre level with the eye in y and z, because an instance world keeps its own map coordinates and the entity origin carries the whole offset. The ffa2 spawn view is enclosed, so a first placement at 2500 units drew 438 entity surfaces that ffa2's walls occluded entirely and blessed an image identical to the baseline. The four defect conditions check out: instance geometry is present in the measured box, the main world is byte-identical outside it, `world_ffa2.png` is untouched, and no run panicked in `world.bmodels` or `surf_dlight_bits`.
