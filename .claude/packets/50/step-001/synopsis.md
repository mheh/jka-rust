# Synopsis gh#50 step-001 - the sub-BSP instance draw side

## Intent

This step makes a sub-BSP instance's brush models render, the draw-side half of the fix whose sim side landed as `e8c175d4`. It gives the renderer one flat surface-index space that spans the main world and every loaded instance, uploads every world into the one buffer pair, teaches `R_AddBrushModelSurfaces` to resolve its owning world and offset into that space, and gives a late instance world a way to reach the render thread.

## Surface contract

- `RenderAssets::world_surface_base`, `RenderAssets::resolve_world_surface`, `RenderAssets::total_world_surfaces` (new methods).
- `BModelEntry::world_index` (new field).
- `RenderModels::worlds_dirty` (new private field) and `RenderModels::take_worlds_dirty` (new method).
- `RenderModels::bmodel_index` removed.
- `WorldWalkScratch::set_world`, `build_world_mesh`, `WorldGeometry::upload`, `FrameExecutor::set_world` each gain an instances parameter.
- `R_AddBrushModelSurfaces`, `RE_GetBModelVerts`, `world_surface_grid`, `BModelTable::build`, `RE_EndFrame` change bodies only.
- `golden_world_subbsp_ffa2` with its PNG.

Anything not on this list is out of scope. No per-world fog table, no per-world lightmap table, no per-world buffers, no world tag on `WorldSurfaceRef`, no engine-crate edit, no WGSL change, no cvar, no ABI change, no new crate.

## Commits

1. `feat(gh#50 s001): the flat world surface index space` - the three methods and their unit tests, no caller yet.
2. `feat(gh#50 s001): the world geometry spans every loaded world` - the mesh build, the upload, the walk marks, six call sites.
3. `feat(gh#50 s001): the brush walk resolves its owning world` - `world_index`, the offset walk, the grid resolve.
4. `feat(gh#50 s001): a late sub-BSP world republishes its generation` - the dirty flag and the `RE_EndFrame` drain.
5. `fix(gh#50 s001): RE_GetBModelVerts reads the owning world` - plus the `bmodel_index` removal.
6. `test(gh#50 s001): the sub-BSP instance golden` - one new PNG after its bless STOP.
7. `process(gh#50 s001): finished file`.

Every commit gates on `cargo build --workspace`, `cargo test --workspace -- --test-threads=1`, and the five golden suites run serially. Commits 1 through 5 must leave all twenty committed fixtures byte-identical. The lockstep referee is not a gate, because no `mp_game`, server, or `jampded` crate is touched.

## Open rows

1. **The fix shape (user ruling).** Default: shape (a), append with offsets. The oracle's draw path never asks which world a surface came from, DEC-43.3 ruling 3 chose the bare `u32` index handle, the backend binds the world buffers once per pass, and (a) changes eight sites against (b)'s eleven.
2. **The index resolver's home (mechanical).** Default: three methods on `RenderAssets` computing the base by summation, with no stored table that can go stale.
3. **The late-arrival republish (user ruling).** Default: a `worlds_dirty` flag on `RenderModels`, drained in `RE_EndFrame`, which rebuilds and overwrites `pending_world`. It coalesces a batch of registrations into one upload, copies the `publish_blocks` pattern two lines away, and keeps the change inside `mp_renderer`.
4. **The walk-scratch sizing (mechanical).** Default: surface marks cover every world, node marks cover the main world alone, because the oracle never walks an instance's nodes.
5. **The fog-table shape, the issue's open question (mechanical).** Default: no change. `R_LoadFogs` does run for every world and an instance surface can carry its own world's fog number, but every backend read is `tr.world->fogs + tess.fogNum`, so the main-world table is the faithful answer and the port already does it.
6. **The lightmap clobber (mechanical).** Default: no change, and a pause trigger. An instance load replaces the whole lightmap table in the oracle too, so the golden's main world will light differently and that is correct.
7. **`RE_GetBModelVerts` (mechanical).** Default: in scope. It is the last main-world-only brush read and it panics on an instance handle today.
8. **The live gate (user ruling).** Default: one new world golden that boots `mp/ffa2` and registers `#mp/duel1`, plus unit tests on the index space. The cost is a camera hunt to put the instance geometry in frame, and a bless STOP.
9. **The `bmodel_index` removal (mechanical).** Default: remove it with its last live caller and move three test assertions to `bmodel_location`.

## Dispatch flags

- Oracle ambiguity: **true**. An instance fog number resolves against the main world's table, an instance load clobbers the lightmap table, and `R_LoadFogs` writes one element past its own allocation.
- New state home: **true**. `RenderModels` gains a `worlds_dirty` flag and `BModelEntry` gains a world index.
- ABI or parity-gate surface: **true**. One new committed golden joins the battery, and four functions change signature across nine call sites. No ABI change.
- Divergence proposal: **false**. Every behavior in scope is faithful, and the two quirks in rows 5 and 6 are preserved rather than diverged.
