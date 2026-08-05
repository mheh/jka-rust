# Finished gh#31 step-006 - the mark fragments go live

Branch `gh31-step-006-marks`, cut from master after `git merge master --no-gpg-sign`.

## Assumptions and choices, keyed to their commits

**Commit 1 (the arena retarget, inert).**

- `R_BoxSurfaces_r` takes `world` and `shaders` separately rather than the whole `RenderAssets`. The walk reads only those two, and `R_MarkFragments` already resolves the world through its own no-map guard, so the recursion never repeats that guard.
- The recursion starts at node index 0. `tr_bsp`'s loader writes the BSP root there, which is the arena form of Raven's `tr.world->nodes`.
- `Node::plane` is `Option<usize>`, and the decision-node arm unwraps it with an `.expect`. Raven dereferences the same pointer with no check, so a `None` there is a loader defect and not a shape this walk can serve.
- The shader lookup uses `.expect` for the same reason. A loaded world surface always carries a registered shader, and a miss is a registry defect, not a mark-path case.
- `MarkState::surf_view_count` is compared against `world.surfaces.len()` and re-zeroed on a mismatch. A same-size world change would keep the old stamps, but the stamps stay internally consistent within one `R_MarkFragments` call because the counter bumps first, so no stale stamp can suppress a surface.
- The leaf loop copies `mark.view_count` into a local before it takes the `&mut` stamp reference. The two would otherwise be overlapping borrows of one `MarkState`.
- Both plane tests copy the plane into a local. `BoxOnPlaneSideRef` wants `&mut cplane_t` and the loaded world is shared immutably. `BoxOnPlaneSide` only reads the plane, so the copy is behavior-identical (porting-rules §10), and one comment line at each site records it.
- The oracle's "all other world surfaces" arm becomes `Skip | Triangles(_) | Flare(_)`. Those are every `SurfaceData` variant the two payload arms do not take, so the match stays exhaustive with no wildcard.
- `RE_AddDecalToScene` keeps its `assets` parameter and passes it straight to `R_MarkFragments`. No new receiver was needed.
- `RendererFrontend::new` is the only construction site in the workspace, so `mark_state` is seated once.

**Commit 2 (the live arms).**

- The mark arm builds fresh `Vec` accumulators per call. `R_MarkFragments` counts its output through the buffer lengths, which is the empty-buffer precondition its doc states.
- The two borrows in the mark arm are `&re.sim.published` and `&mut re.mark_state`, disjoint fields of the one `re`, so no aliasing arises and no second cast is needed.
- The copy-out slices are sized by the returned buffer lengths, not by the caller's caps. `R_MarkFragments` never exceeds the caps, so the shorter slice is always inside the module's buffer.
- Both decal arms read `SceneState::last_time` for `tr.refdef.time`. `RE_RenderScene` writes `refdef.time` and `last_time` from the same `fd.time`, and cgame adds its decals before it calls `trap_R_RenderScene`, so `last_time` is the value the oracle reads at that point in the frame.
- `FxHost::AddDecalToScene`'s harness arm is untouched. Only the `Engine` arm changed, so the parity recording still emits its `DECAL` line.

**Commit 3 (the marks golden).**

- The mark scene reproduces `CG_ImpactMark`'s `temporary` path only. The persistent path allocates a `markPoly_t` in cgame, which has no renderer-side effect.
- `MAX_VERTS_ON_POLY` is declared locally at the cgame value of 10, cited to `cg_local.h:56`. The renderer-local constant of the same name is 64 and is a different quantity.
- The per-vertex `modulate` is opaque white. `CG_ImpactMark` derives it from the caller's rgba, and white is the neutral choice that leaves the shader's own blend as the only color source.
- The fixture asserts a nonzero fragment count before it renders. An empty walk therefore fails loud rather than blessing an empty floor.

## Deviations

1. **The marks fixture looks straight down (pitch 90).** Ruling B named the eye, the mark, the radius, and the shader, but not the view direction. `build_refdef`'s existing `[0, 0, 0]` looks horizontally, which puts a mark under the eye outside the frustum. That would bless an image with no visible mark, which the bless procedure calls a defect. The pitch fills the gap the ruling left and changes nothing the ruling named.
2. **`world_golden.rs` gained a small refactor.** `build_refdef` takes the view angles, `record_scene` takes the frame the caller already filled, and `run_golden` delegates to `run_golden_scene`, which adds the angles and one optional scene step. The render command must sit after the scene polygons, which the old "record returns a fresh frame" shape could not express. The two older fixtures pass the old values and their goldens did not move.
3. **`MARK_DROP` is 64, measured rather than assumed.** The duel1 spawn eye sits at z 192 and the floor under it at z 128. A first probe at 45 also produced one fragment, but it left the mark plane 19 units above the floor against a 20-unit clip window. The 64-unit drop puts the mark on the floor, which is `CG_ImpactMark`'s "within a unit of the plane" contract, and the projected points and texture coordinates are identical either way.
4. **The oracle cites in the packet were off, and the code carries the real lines.** The mark arm is `oracle/codemp/client/cl_cgame.cpp:805-806`, not 806 or 719. The decal arm is `oracle/codemp/client/cl_cgame.cpp:903-904`, not 1027.

## Pause triggers hit

None. No existing golden moved in any byte or pixel. The walk needed no `WorldAsset`, `Node`, or `Surface` field the contract did not name, and no shader lookup failed. The retargeted walk returned one fragment in the new scene, so no input was tuned to force output. The stand-in deletion broke exactly one caller, `tr_scene.rs`, which the contract lists.

## Commits and gate results

1. `ce238644` **renderer: the mark walk reads the world arena**
   - `cargo build --workspace`: green, zero warnings.
   - `cargo test --workspace`: green.
   - `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`: 2 passed, byte-identical.
   - `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`: 7 passed.

2. `f0a953db` **client: the mark and decal trap arms go live**
   - `cargo build --workspace`: green, zero warnings.
   - `cargo test --workspace`: green.
   - `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`: 2 passed, byte-identical.
   - `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`: 7 passed.
   - `cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`: 1 passed, byte-identical.
   - `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`: 1 passed, byte-identical.
   - `grep -rn "TODO: Port R_MarkFragments\|TODO: Port RE_AddDecalToScene"`: no hit.

3. `58e8fdba` **test: the marks image golden for duel1**
   - `cargo build --workspace`: green, zero warnings.
   - `cargo test --workspace`: green.
   - `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`: 3 passed at `CHANNEL_TOLERANCE = 0`. The two older goldens are byte-identical and `world_marks_duel1.png` is the new one.
   - `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`: 7 passed.
   - `cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`: 1 passed, byte-identical.
   - `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`: 1 passed, byte-identical.
   - The user approved the blessed image before this commit, per the packet's bless procedure.

Every golden run was one foreground command with `--test-threads=1`, and `dedicated` stayed `"0"` in every rig run. The lockstep referee was not run: no commit touches `mp_game`, the server, or any `jampded` link-set crate.

## Open gaps

- The blessed image reads as a white square in a viewer that composites alpha. The RGB is correct: the mark background is `(163, 129, 108)` against a floor of `(157, 125, 103)`, which is the near-identity result `blendFunc GL_DST_COLOR GL_SRC_COLOR` gives for `gfx/damage/rivetmark`'s neutral `(128, 128, 128)` background. The same blend factors apply to alpha, and that texture's background alpha is 0, so the target's alpha goes to 0 there. Raven uses plain `glBlendFunc`, which applies one factor pair to all four channels, so this is faithful. It only affects how a viewer displays the fixture, never the on-screen frame.
- `crates/mp/renderer/src/tr_light.rs:314` still names `tr_marks::MarkNode` in a prose doc comment, as the precedent for its own `DlightBmodel` stand-in. That type is deleted. The file is not in this packet's write scopes, so it was left untouched.
- The mark walk always starts at node index 0, so a submodel decal (Raven's `tr.world->bmodels`) has no entry point. No caller asks for one, and the oracle's `R_MarkFragments` has the same single root.
- Live play is the remaining gate on the three restored effects. The fixed-scene golden covers the projection and the poly draw, not the cgame code paths that feed impact marks, saber damage glow, and the blob player shadow.
