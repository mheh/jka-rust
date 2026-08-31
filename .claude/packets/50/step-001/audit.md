# Audit gh#50 step-001 - the sub-BSP instance draw packet

Draft under audit: `.claude/packets/50/step-001/packet.md` at commit `bccf68ae`. Method: every `oracle/` cite was opened and read in the source before the draft, then every Rust claim was checked against the real neighbors in `crates/mp/renderer/` and `crates/mp/renderer-gpu/`, then the draft was read against both.

Cite quality is high. Nearly every line reference in the draft lands on the exact line it names, in both trees. The two exceptions are named at the end of this file and neither changes a verdict.

## Mechanical open rows

### Row 2 - the flat-index resolver's home - CHALLENGED (scope only)

The design is correct. `RenderAssets` owns both fields the sum needs.

```
crates/mp/renderer/src/render_state/render_assets.rs:157:    pub world: Option<Arc<WorldAsset>>,
crates/mp/renderer/src/render_state/render_assets.rs:170:    pub bsp_models: Vec<WorldAsset>,
```

The per-surface cost claim also holds. `world_surface_grid` is the only per-surface re-fetch of a flat index in the whole backend, and a grep for `.surfaces` across `crates/mp/renderer-gpu/src/` returns exactly one indexed read outside `build_world_mesh`:

```
crates/mp/renderer-gpu/src/pipeline3d.rs:4094:    match &assets.world.as_ref()?.surfaces.get(index as usize)?.data {
```

The challenge is narrow. `total_world_surfaces` has no caller anywhere in the packet. The surface contract does not name one, `WorldWalkScratch::set_world` takes `world: &WorldAsset, instances: &[WorldAsset]` and never sees a `RenderAssets`, and commit 1 states plainly "No caller yet, so no behavior changes". Porting rule 20 drops API with zero callers rather than adding it speculatively. Either name the caller or cut the method to two.

### Row 4 - the walk-scratch sizing - CLEARED

The oracle evidence is stronger than the packet states. `tr.bspModels` appears in exactly two lines of the whole MP tree, the declaration and the one load call:

```
oracle/codemp/renderer/tr_local.h:1399:	world_t					bspModels[MAX_SUB_BSP];
oracle/codemp/renderer/tr_model.cpp:1233:		RE_LoadWorldMap_Actual(va("maps/%s.bsp", name + 1), tr.bspModels[tr.numBSPModels - 1], tr.numBSPModels);
```

Nothing else in the renderer ever names an instance world. The only reach into an instance's data is the `bmodel_t::firstSurface` pointer the submodel row carries. The node walk is main world only, as cited:

```
oracle/codemp/renderer/tr_world.cpp:1957:	R_RecursiveWorldNode( tr.world->nodes, 15, ( 1 << tr.refdef.num_dlights ) - 1 );
```

`R_MarkLeaves` reads `tr.world->nodes` at `:1867` and `:1876` as well. An instance's nodes, leaves and PVS are dead weight in the oracle, so node marks stay main world sized. The default is right.

One consequence the packet leaves unstated and the worker should know: with the main world first in the flat space, its base is zero, so the node walk needs no offset and `R_RecursiveWorldNode` keeps passing raw `WorldAsset::surfaces` subscripts.

### Row 5 - the fog-table shape - CLEARED

Both legs check out, and the answer to the issue's open question is the packet's answer.

Every `tess.fogNum` indexed read in the backend resolves the main world. The list is complete, not a sample:

```
oracle/codemp/renderer/tr_shade.cpp:1192:	fog = tr.world->fogs + tess.fogNum;
oracle/codemp/renderer/tr_shade.cpp:1376:			fog = tr.world->fogs + tess.fogNum;
oracle/codemp/renderer/tr_shade.cpp:1660:				fog = tr.world->fogs + tess.fogNum;
oracle/codemp/renderer/tr_shade.cpp:1963:		fog = tr.world->fogs + tess.fogNum;
oracle/codemp/renderer/tr_shade_calc.cpp:993:	fog = tr.world->fogs + tess.fogNum;
```

Every other `->fogs` read in the tree takes `globalFog` or walks `1..numfogs`, and none of them takes a surface's number. There is no per-surface world pointer at the backend, so the oracle cannot resolve anything else.

An instance surface can carry a number out of the main world's range. `R_LoadFogs` runs unguarded for every world at `tr_bsp.cpp:2077`, and the four parsers number the surface out of the instance's own table:

```
oracle/codemp/renderer/tr_bsp.cpp:371:	surf->fogIndex = LittleLong( ds->fogNum ) + 1;
oracle/codemp/renderer/tr_bsp.cpp:372:	if (index && !surf->fogIndex && tr.world->globalFog != -1)
```

So the out-of-range case is real. The panic question answers no. The port bounds-checks:

```
crates/mp/renderer-gpu/src/pipeline3d.rs:4069:    let fog = fogs.get(fog_num as usize)?;
```

`resolve_surface_fog` is the only consumer of a decoded sort-key fog number, at five call sites in `pipeline3d.rs`. The other `fogs[i]` reads in the tree sit inside `for i in 1..fogs.len()` frontend loops, for example `R_SpriteFogNum` at `tr_main.rs:777-778`, so they cannot run past the end. An instance surface with a high fog number draws unfogged instead of crashing. That is a defined answer to an oracle out-of-bounds read, per porting rule 19. No work, and no panic risk.

### Row 6 - the lightmap clobber - CHALLENGED

The mechanism is right and the consequence is wrong.

The table replacement is real. The zeroing is main world only and the two writes are unconditional:

```
oracle/codemp/renderer/tr_bsp.cpp:176:	if (&worldData == &s_worldData)
oracle/codemp/renderer/tr_bsp.cpp:178:		tr.numLightmaps = 0;
oracle/codemp/renderer/tr_bsp.cpp:191:	tr.numLightmaps = len / (LIGHTMAP_SIZE * LIGHTMAP_SIZE * 3);
oracle/codemp/renderer/tr_bsp.cpp:240:		tr.lightmaps[i] = R_CreateImage( va("*%s/lightmap%d",sMapName,i), image,
```

The port matches at `tr_bsp.rs:2417` and `:2519`. So far the packet is correct.

The disputed sentence is this one: "An instance load therefore replaces the whole lightmap table, and the main world's surfaces then sample the instance's lightmaps." Every read of `tr.lightmaps` in the whole oracle renderer sits in shader creation, and none of them runs per frame:

```
oracle/codemp/renderer/tr_shader.cpp:1329:					stage->bundle[0].image = tr.lightmaps[shader.lightmapIndex[0]];
oracle/codemp/renderer/tr_shader.cpp:3023:					stages[lmStage+i+1].bundle[0].image = tr.lightmaps[shader.lightmapIndex[i+1]];
oracle/codemp/renderer/tr_shader.cpp:3543:		stages[0].bundle[0].image = tr.lightmaps[shader.lightmapIndex[0]];
oracle/codemp/renderer/tr_shader.cpp:3667:		stages[0].bundle[0].image = tr.lightmaps[shader.lightmapIndex[0]];
```

A shader stage keeps the `image_t*` it captured when the shader was built. The port does the same, and holds an `ImageHandle` copy:

```
crates/mp/renderer/src/tr_shader.rs:5734:        state.stages[0].bundle[0].image = Some(assets.lightmaps[lm_idx]);
```

The main world builds its shaders during its own load, while `tr.lightmaps` still holds its own images. A later instance load cannot reach back into those stages. So the main world keeps its lighting, and the clobber only steers shaders that are built after it. The true effect runs the other way: an instance surface whose shader name and lightmap index already sit in the shader cache gets the main world's shader, with the main world's lightmap image.

This matters twice. First, row 6 is written as a pause trigger, and it hands the worker a blanket excuse for any main-world lighting change in the new image. Second, row 8's defect conditions are built on it. Row 8 says an image "identical to `world_ffa2.png` except for lighting" means the submodel drew nothing. On the reading above the correct and stronger condition is simpler: the main world's pixels should match `world_ffa2.png`, and the only difference should be the instance geometry. A main-world lighting change is then a finding to report, not an expected result.

The default of "no change to the code" survives the challenge. The rationale and the pause trigger need the rewrite.

One residual risk, noted rather than challenged. Two port sites index the lightmap table unchecked, `tr_shader.rs:3883` and `:5734`, where the oracle is equally unchecked. They are reachable only from a BSP load, and each world builds its shaders against its own live table, so a self-consistent BSP cannot trip them. This step does not create the risk and does not need to close it.

### Row 7 - `RE_GetBModelVerts` - CLEARED

The defect is real and live. The function resolves the main world unconditionally and panics on a bad index:

```
crates/mp/renderer/src/tr_world.rs:631:    let world = assets
crates/mp/renderer/src/tr_world.rs:635:    let bmodel = &world.bmodels[idx];
```

It is reachable from the cgame trap table at `crates/mp/engine/client/src/cl_cgame.rs:2544`, and the call already carries both `rm` and `&re.sim.published`, so the two-arm resolve through `bmodel_location` needs no signature change. `r_model_bounds` at `frontend.rs:309-321` is the exact shape to copy. In scope, as its own commit, is right.

### Row 9 - the `bmodel_index` removal - CHALLENGED (count)

The removal is right. The count is wrong. `bmodel_index` has four callers, not three, and all four are assertions in the same test module:

```
crates/mp/renderer/src/tr_model/render_models.rs:485:        assert_eq!(rm.bmodel_index(h0), Some(0));
crates/mp/renderer/src/tr_model/render_models.rs:486:        assert_eq!(rm.bmodel_index(h1), Some(1));
crates/mp/renderer/src/tr_model/render_models.rs:493:        assert_eq!(rm.bmodel_index(h2), Some(2));
crates/mp/renderer/src/tr_model/render_models.rs:496:        assert_eq!(rm.bmodel_index(999), None);
```

A grep across every tracked `crates/**/*.rs` finds no other caller. Read "four" for "three" in the surface contract and in row 9. The fourth is the `None` case for an unregistered handle, and it must move to `bmodel_location` with the rest.

## User-ruling open rows

### Row 1 - the fix shape - CHALLENGED (the counts only)

The argument for shape (a) is sound and every load-bearing piece of it verifies.

The oracle's draw path really is world-agnostic. The submodel row carries a pointer, `msurface_t *firstSurface` at `tr_local.h:940`. The walk hands it straight down at `tr_world.cpp:608`. The draw surf carries `surfaceType_t *surface` and the backend dispatches on it at `tr_backend.cpp:756`. `R_DecomposeSort` returns a fog number, an entity number, a shader and a dlight bit, and nothing else:

```
oracle/codemp/renderer/tr_main.cpp:1291:	*fogNum = ( sort >> QSORT_FOGNUM_SHIFT ) & 31;
```

The one-pass claim verifies too. The index buffer binds once per pass at `pipeline3d.rs:1601` and the vertex buffer at `:1615`, inside a loop over items sorted by shader, so per-world buffers would force a rebind per item.

The DEC check is clean. DEC-43.1 states flat-index addressing preserves the oracle's one surface-index space and DEC-43.3 chose the bare `u32` handle. Shape (a) extends that space across worlds, which the oracle does not need, and the packet says so in the `world_surface_base` doc rather than hiding it. Shape (b) is correctly flagged as a DEC divergence that would need its own ruling row. No DEC is contradicted without a flag.

The challenge is the arithmetic. "Shape (a) changes eight sites" and "(b) changes all eight `WorldSurfaceRef` consumers" are both unsupported. The packet's own commit bundle touches fourteen files and about sixteen items. A grep for `WorldSurfaceRef` outside its own definition finds five real use sites, not eight: `tr_world.rs:1417`, `tr_main.rs:192` with the portal arm near `:755`, `pipeline3d.rs:2072`, `pipeline3d.rs:4082-4086`, and the boot tally at `boot.rs:808-812`. Neither number is close enough to carry weight in a ruling. Drop both counts or replace them with the real ones. The verdict on the shape does not depend on them.

### Row 3 - the late-arrival republish - CHALLENGED (no gate)

The mechanism works. It reaches the render thread, and the pattern it copies is real.

The pattern is two lines above the proposed block:

```
crates/mp/renderer/src/tr_cmds.rs:356:    if let Some(blocks) = rm.publish_blocks() {
crates/mp/renderer/src/tr_cmds.rs:357:        sim.publish_models(blocks);
```

The delivery path holds end to end. `RE_EndFrame` clones the whole asset set and takes the pending generation onto the package:

```
crates/mp/renderer/src/tr_cmds.rs:388:                assets: Arc::clone(&sim.published),
crates/mp/renderer/src/tr_cmds.rs:392:                world: pending_world.take(),
```

`execute_package` takes it and calls `set_world` at `frame_exec.rs:360-362`, so the instances on `package.assets.bsp_models` arrive on the same package as the generation that triggers the upload. That coupling is the important part, and it is correct: `build_world_mesh` and `world_surface_base` then read the same `bsp_models` list, so the uploaded ranges and the computed bases cannot disagree.

The generation shape the packet proposes is already written for the map-load case, so the drain is a copy of an existing block:

```
crates/mp/engine/client/src/cl_cgame.rs:1881:        re.pending_world = Some(WorldGeneration {
crates/mp/engine/client/src/cl_cgame.rs:1882:            world: re.sim.published.world.clone(),
crates/mp/engine/client/src/cl_cgame.rs:1883:            bmodels: BModelTable::build(rm),
```

`world` is `Option<Arc<WorldAsset>>` at `world_generation.rs:26`, so the clone costs a refcount. The write point is right too: `assets.bsp_models[slot] = world;` sits at `frontend.rs:964`, above both early returns of the `#` arm, so the flag cannot be skipped.

The challenge is the gate. No test in the workspace reaches `RE_EndFrame` or `execute_package`. The only caller of `execute_package` is `crates/mp/client-app/src/render_thread.rs:136`, and five test files carry the standing comment "`RE_EndFrame` drains the registered model blocks into the published registry, and no test reaches it", including `world_golden.rs:294`. The new golden does not change that. `run_golden_scene` runs its step closure, then calls `executor.set_world` directly at `world_golden.rs:274`, so it uploads after the registration and never touches the drain.

Commit 4 therefore lands with only an inertness proof. "All twenty fixtures byte-identical" shows the flag changes nothing when no instance loads. It shows nothing about the one piece that makes the fix work in the live client. `RE_EndFrame` is a sim-side function with no GPU in it, and `pending_world` is a plain `&mut Option<WorldGeneration>` parameter, so a unit test can call it and assert the drain filled the slot. The row should either carry that test or state the gap plainly for the user to rule on.

### Row 8 - the live gate - CHALLENGED (defect conditions)

The gate invocations are correct and complete. They match the `.claude/packets/31/step-010/packet.md` battery line for line, including `--test-threads=1` on every golden run and on `cargo test --workspace`, the two `hud_golden` runs, and the note about parallel engine boots and the pk3 inflate. The fixture count is right: `crates/mp/renderer-gpu/tests/goldens/` holds twenty files today, and the four world goldens are `duel1`, `ffa2`, `marks_duel1` and `dlights_duel1`.

The golden design is buildable. `boot::register_model` at `boot.rs:551` runs the real `RE_RegisterModel` chain, the `#` arm loads `maps/mp/duel1.bsp` and returns the `*1-0` handle from the hash, and the step closure runs before both `BModelTable::build` and `executor.set_world` in `run_golden_scene`. Picking `*1-0` is also the faithful handle, because retail cgame registers the `#` name itself and stores what it returns, at `cg_main.c:2323`.

Two challenges.

First, the defect conditions inherit row 6's wrong consequence. "An image identical to `world_ffa2.png` except for lighting means the instance submodel drew nothing" and "The main world's lighting differing from `world_ffa2.png` is expected, per row 6" both need to go. Per the row 6 finding the main world's pixels should not move at all, so the condition tightens to: the image must equal `world_ffa2.png` everywhere except where the instance geometry covers it.

Second, the row claims this golden is the live gate for the whole step, and it is not. It gates commits 2, 3 and 5. It bypasses commit 4 entirely. The row should say which commits it covers.

## Other load-bearing claims

- **"the six call sites pass it" (commit 2) - CONFIRMED.** `executor.set_world` has exactly six callers: `entity_golden.rs:359` and `:583`, `ghoul2_vertex_golden.rs:458`, `world_golden.rs:274`, `world_harness.rs:1017`, and `frame_exec.rs:362`.
- **"nine call sites" (synopsis dispatch flags) - DISPUTED.** A literal grep gives ten across the four functions: six for `FrameExecutor::set_world`, two for `WorldWalkScratch::set_world` at `frame_exec.rs:307` and `boot.rs:765`, one for `WorldGeometry::upload` at `frame_exec.rs:306`, and one for `build_world_mesh` at `pipeline3d.rs:242`. The file list in commit 2 is complete either way.
- **"the issue's root-cause line saying instance worlds never cross to the render thread is wrong about the CPU side" - CONFIRMED.** `assets.bsp_models[slot] = world` at `frontend.rs:964` writes through `Arc::make_mut(&mut re.sim.published)` at `cl_cgame.rs:1893`, and `tr_cmds.rs:388` clones that whole set onto every package.
- **"the symptom is silence rather than a crash" - CONFIRMED.** The stale table hands the default row, `bmodel_index = -1` at `bmodel_table.rs:29`, and the brush test at `tr_main.rs:1955` is `if model.bmodel_index >= 0`. The same staleness also keeps the intermediate state after commit 3 silent rather than a panic, because the offset walk cannot run until commit 4 makes the table fresh.
- **"The `#` branch has no cache check" - CONFIRMED in effect.** The oracle looks up the raw name in the hash at `tr_model.cpp:1211-1215`, but `R_LoadSubmodels` inserts submodels under `*k-n` at `tr_bsp.cpp:1442-1460`, never under the `#` name, so the `#` name never hits.
- **The one-past-the-end fog write - CONFIRMED, and already ported.** `worldData.fogs[worldData.numfogs]` at `tr_bsp.cpp:1694` writes past the `count + 1` allocation of `:1683-1684`, and `tr_bsp.rs:1992-2004` reaches the same logical index with a `push` and cites porting rule 19. The oracle guards it with `tr.world && tr.world->globalFog != -1`, which the packet omits and the port keeps.
- **DEC-54 - not cited.** The issue calls this ticket a DEC-54 graduation and the packet never names DEC-54. Nothing in the packet contradicts it. Adding the cite would close the provenance line.

## Cite drift

Two references are off and neither changes a verdict.

- `golden_world_duel1` is at `world_golden.rs:422-425`, not `:424-431`. The range named is `golden_world_ffa2`.
- The oracle's submodel loop is `tr_world.cpp:607-609`, not `:608-610`. The `R_AddWorldSurface` call inside it is at `:608`, as cited elsewhere in the packet.

## Verdict

Rows the walk must include beyond the user-ruling rows, one row per challenge:

1. **Row 6, the lightmap clobber.** The table replacement is real, the stated consequence is not. Every `tr.lightmaps` read is shader creation, so the main world keeps its own lighting and the pause trigger must not excuse a main-world change.
2. **Row 9, the `bmodel_index` count.** Four assertions, not three.
3. **Row 2, `total_world_surfaces`.** Zero callers in the whole packet. Name one or cut the method.
4. **Row 1, the site counts.** "Eight" and "eleven" are unsupported by any grep. The shape verdict stands without them.
5. **Row 3, the gate gap.** The republish is correct and no gate exercises it, because no test reaches `RE_EndFrame` or `execute_package`.
6. **Row 8, the defect conditions.** They rest on the row 6 error, and the row overstates its coverage by claiming commit 4.

Rows cleared with no walk needed: row 4, row 5, row 7.
