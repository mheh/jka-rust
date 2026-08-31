# Synopsis gh#50 step-001 - the sub-BSP instance draw side

Ratified 2026-08-30. All nine open rows closed, plus the vet's row A, and the packet body carries the folded shape. Audit: `.claude/packets/50/step-001/audit.md`.

## Intent

This step makes a sub-BSP instance's brush models render, the draw-side half of the fix whose sim side landed as `e8c175d4`. It gives the renderer one flat surface-index space that spans the main world and every loaded instance, uploads every world into the one buffer pair, teaches `R_AddBrushModelSurfaces` to resolve its owning world and offset into that space, and gives a late instance world a way to reach the render thread.

## Surface contract

- `RenderAssets::world_surface_base` and `RenderAssets::resolve_world_surface` (new methods).
- `BModelEntry::world_index` (new field).
- `RenderModels::worlds_dirty` (new private field) and `RenderModels::take_worlds_dirty` (new method).
- `RenderModels::bmodel_index` removed.
- `WorldWalkScratch::set_world`, `build_world_mesh`, `WorldGeometry::upload`, `FrameExecutor::set_world` each gain an instances parameter.
- `R_AddBrushModelSurfaces`, `RE_GetBModelVerts`, `world_surface_grid`, `BModelTable::build`, `RE_EndFrame`, `RE_LoadWorldMap_Actual`, `RenderModels::model_init`, `RenderModels::hunk_clear` change bodies only.
- `golden_world_subbsp_ffa2` with its PNG.

Anything not on this list is out of scope. No per-world fog table, no per-world lightmap table, no per-world buffers, no world tag on `WorldSurfaceRef`, no engine-crate edit, no WGSL change, no cvar, no ABI change, no new crate.

## Commits

1. `feat(gh#50 s001): the flat world surface index space` - the two methods and their unit tests, no caller yet.
2. `feat(gh#50 s001): the world geometry spans every loaded world` - the mesh build, the upload, the walk marks, six call sites, and the F1 `bsp_models` truncation.
3. `feat(gh#50 s001): the brush walk resolves its owning world` - `world_index`, the offset walk, the grid resolve.
4. `feat(gh#50 s001): a late sub-BSP world republishes its generation` - the dirty flag and the `RE_EndFrame` drain.
5. `fix(gh#50 s001): RE_GetBModelVerts reads the owning world` - plus the `bmodel_index` removal and the F2 teardown clears.
6. `test(gh#50 s001): the sub-BSP instance golden` - one new PNG after its bless STOP.
7. `process(gh#50 s001): finished file`.

Every commit gates on `cargo build --workspace`, `cargo test --workspace -- --test-threads=1`, and the five golden suites run serially. Commits 1 through 5 must leave all twenty committed fixtures byte-identical. The lockstep referee is not a gate, because no `mp_game`, server, or `jampded` crate is touched.

## The settled rows

1. **The fix shape, amended.** Shape (a), append with offsets. The oracle's draw chain is pointer-carried and holds no world id anywhere, and DEC-43.3 ruling 3's handle is a kind-tagged `Copy` index-handle enum over the flat `u32` index, which shape (a) leaves untouched. The blast-radius sentence is struck as unsupported.
2. **The index resolver's home, amended.** Two methods on `RenderAssets`, `world_surface_base` and `resolve_world_surface`, computing the base by summation with no stored table. `total_world_surfaces` is struck for having no caller.
3. **The late-arrival republish, amended.** The `worlds_dirty` flag and the `RE_EndFrame` drain stand. Commit 4's battery proves inertness alone, because no automated test reaches `RE_EndFrame`. Live play verifies the drain and the finished file records the gap.
4. **The walk-scratch sizing, as proposed.** Surface marks cover every world, node marks cover the main world alone, because the oracle never walks an instance's nodes.
5. **The fog-table shape, the issue's open question, as proposed.** No change. `R_LoadFogs` does run for every world and an instance surface can carry its own world's fog number, but every backend read is `tr.world->fogs + tess.fogNum`, so the main-world table is the faithful answer and the port already does it.
6. **The lightmap clobber, amended.** No code change, and the consequence claim is corrected. Every read of the table captures the image handle at shader-build time, so the main world keeps its own lighting after an instance load. Only a shader built after the clobber reads the instance's table.
7. **`RE_GetBModelVerts`, as proposed.** In scope. It is the last main-world-only brush read and it panics on an instance handle today.
8. **The live gate, amended.** One new world golden that boots `mp/ffa2` and registers `#mp/duel1`, plus unit tests on the index space. Recast per row 6: the main world must render identically, and any main-world change is a defect. The golden claims no coverage of commit 4. The camera hunt stands as the cost.
9. **The `bmodel_index` removal, amended.** Remove it in commit 5 with its last live caller, and move four test assertions to `bmodel_location`.

Row A, the vet's guard rail, is ratified and folded. F1 truncates `assets.bsp_models` on a main-world load, in commit 2, so a map rotation cannot leave stale instance worlds in the flat space. F2 clears `bmodel_indices` at both `RenderModels` teardowns, in commit 5. F3 records the proof that a same-frame load and registration cross together. F4 records that the registration window is safe by timing, not by construction.

## Dispatch flags

- Oracle ambiguity: **true**. An instance fog number resolves against the main world's table, an instance load clobbers the lightmap table so a later shader build reads it, and `R_LoadFogs` writes one element past its own allocation.
- New state home: **true**. `RenderModels` gains a `worlds_dirty` flag and `BModelEntry` gains a world index.
- ABI or parity-gate surface: **true**. One new committed golden joins the battery, and four functions change signature across nine call sites. No ABI change.
- Divergence proposal: **false**. Every behavior in scope is faithful, and the two quirks in rows 5 and 6 are preserved rather than diverged.
