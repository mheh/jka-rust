# Finished gh#50 step-001 - the sub-BSP instance draw side

Branch `gh50-step-001-sub-bsp-draw`, cut from `master`, merged with `master` as the lane's first act (already up to date). Six code commits plus this file. Nothing pushed, no pull request opened.

## Assumptions and choices

**Commit 1 - the two index-space methods.** `world_surface_base` returns `0` for the main world without reading anything, and sums the main world plus the first `world_index - 1` instances otherwise. `Iterator::take` bounds the sum, so an out-of-range `world_index` returns the total rather than panicking. `resolve_world_surface` walks the same order and returns `None` past the last loaded world. Both live on `RenderAssets` because it already owns `world` and `bsp_models`, so neither can go stale. The tests build their synthetic registry through `renderer_frontend::empty_render_assets`, which is the crate's own empty-registry constructor.

**Commit 2 - the concatenation.** `build_world_mesh` iterates `once(world).chain(instances)` and flattens each world's surfaces, so the range vector is one flat list in the order `world_surface_base` sums. `std::iter::once` is imported at the file top. `WorldWalkScratch::set_world` sums the same total for the surface marks and passes `world.nodes.len()` unchanged for the node marks. The `WorldWalkScratch` struct-level doc still says the surface marks are indexed by "the `WorldAsset::surfaces` subscript". That sentence stays true for the main world, and the new `set_world` doc directly below it states the whole law. The packet's write scope for that file is `set_world` only, so the struct doc was left alone.

`FrameExecutor::execute_package` reads the instance slice off `package.assets.bsp_models`, the package's own `Arc<RenderAssets>`, so the geometry and the registry the frame draws against are the same generation. Every other call site binds the published registry to a local `assets` first and passes `&assets.bsp_models`, which keeps both borrows immutable and reads one object.

The `assets.bsp_models.clear()` of amendment F1 sits directly under the `assets.world` write, inside the `index == 0` block, so it runs on the main-world leg alone. Its comment records why the oracle needs no twin: `tr.numBSPModels` bounds every read of Raven's array, while this port's flat index space sums over the whole `Vec`.

**Commit 3 - the owning world.** `BModelTable::build` reads `bmodel_location` once per slot and destructures the pair, so a non-brush handle takes the `(0, -1)` arm and keeps the default row's meaning. `R_AddBrushModelSurfaces` resolves the owning world twice, once for the bounds and the dlight snapshot and once for the draw loop, because the two sites are separated by the `R_DlightBmodel` call and the borrow of `assets.shaders`. `base` is computed once and reused. `WorldAsset` was added to the file's existing `render_state::placeholders` import group.

**Commit 4 - the republish.** `take_worlds_dirty` is `pub(crate)`, matching the packet's contract. The drain copies the `publish_blocks` shape two lines above it and sits above the sink match, so it runs with no sink installed.

**Commit 5 - `RE_GetBModelVerts`.** The two-arm resolve is the `r_model_bounds` shape. `RenderModels::bmodel_index` is removed. The four test assertions now read `bmodel_location` and check the world half too, so the main-world rows assert `Some((0, n))` and the RMG row asserts `Some((3, 2))`.

**Commit 6 - the golden.** The fixture runs through the existing `run_golden_scene` with `require_sky_and_fog` true, the same knob `golden_world_ffa2` uses, and its `SceneStep` registers the instance and adds the entity before the scene records. That ordering matters: `run_golden_scene` builds the `BModelTable` and calls `FrameExecutor::set_world` after the step, so both cover the instance. The step asserts the registration returned a handle and that the instance world loaded surfaces, so an empty instance cannot bless the plain ffa2 image silently.

## Deviations

None.

## Commits and gate results

The full battery, run per commit: `cargo build --workspace`, `cargo test --workspace -- --test-threads=1`, the world goldens with `--ignored --test-threads=1`, the scene goldens, the entity goldens with `--ignored`, the ghoul2-vertex golden with `--ignored`, and the hud goldens both with and without `--ignored`. Every golden run was serial and foreground. The lockstep referee is not a gate here, because no commit touches `mp_game`, the server, or any `jampded` link-set crate.

1. `ff4d949d` **feat(gh#50 s001): the flat world surface index space.** Battery green. Build clean with zero warnings, all twenty committed image fixtures byte-identical.
2. `1c1dfc64` **feat(gh#50 s001): the world geometry spans every loaded world.** Battery green. Build clean with zero warnings, all twenty fixtures byte-identical, which is the proof the concatenation is inert with no instance loaded.
3. `342ce626` **feat(gh#50 s001): the brush walk resolves its owning world.** Battery green. Build clean with zero warnings, all twenty fixtures byte-identical, because a main-world submodel has base zero.
4. `91f10a29` **feat(gh#50 s001): a late sub-BSP world republishes its generation.** Battery green. Build clean with zero warnings, all twenty fixtures byte-identical. That battery proves inertness alone, per the open gap below.
5. `a552bbcd` **fix(gh#50 s001): RE_GetBModelVerts reads the owning world.** Battery green. Build clean with zero warnings, all twenty fixtures byte-identical.
6. `28db5c7f` **test(gh#50 s001): the sub-BSP instance golden.** Battery green. Build clean with zero warnings, the new `world_subbsp_ffa2` golden green at tolerance zero on a bless-then-verify run, and the other twenty fixtures byte-identical. The user blessed the image on 2026-08-30 and the packet carries the ruling as an Amendment.

No committed fixture other than the new PNG moved at any point in the bundle.

The six commits above were replayed once, after the last gate run, to rewrite their bodies. The first draft opened each gate paragraph with `Gates: `, which git parses as a trailer, and the packet forbids that. The replay changed the messages alone. `git diff` against the pre-replay branch is empty, so every tree is unchanged and the gate results above still hold.

## Open gaps

**Commit 4 has no automated gate.** No test in this workspace reaches `RE_EndFrame`, which all five golden test files record verbatim, and each one drains the published model blocks by hand instead. The row-8 golden hands the loaded world to `FrameExecutor::set_world` directly, so it does not exercise the drain either. Commit 4's battery proves the flag and the drain are inert, and nothing more. Live play on a mod server that loads sub-BSP instances is the verification.

**The registration window is safe by timing, not by construction (vet finding F4).** cgame registers its instances at `CG_Init`, before its first draw frame, so the walk scratch is always sized before an instance surface can reach it. A future caller that registered an instance and drew in the same frame would panic in the walk scratch.

**The `world_subbsp_ffa2` fixture depends on the `mp/ffa2` spawn view staying enclosed.** The instance sits 64 units in front of the eye so ffa2's own walls cannot occlude it. A change to `boot::find_spawn_origin` or to the eye-height bump would move the instance with the camera, but a change to the fixture's angles could put the instance behind a wall again and re-bless a baseline image.
