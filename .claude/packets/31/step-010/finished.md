# Finished gh#31 step-010 - the renderfx closure

Branch `gh31-step-010-renderfx`, cut from `gh31-step-009-fx-minirefents`. Fourteen commits, twelve from the build and two from the lane-review fix round. Every gate the packet names ran for every commit, with the exact invocations the packet gives.

The base branch no longer exists. Step-009 merged into `wf/31-renderer-census` as pull request #48, and its tip is `61a5062b`, which is the range the vet walked.

## Commits and gate results

| # | commit | subject |
|---|---|---|
| 1 | `feae3d9d` | `feat(gh#31 s010): the frame target texture reaches the backend` |
| 2 | `69971bf1` | `feat(gh#31 s010): the post-render deferral` |
| 3 | `a110a866` | `feat(gh#31 s010): the per-entity screen capture` |
| 4 | `78461ecf` | `feat(gh#31 s010): the RF_DISTORTION stage arm` |
| — | `a09b8e38` | `process(gh#31 s010): the mid-lane amendment` |
| 5 | `af7f04d9` | `test(gh#31 s010): the distortion golden` |
| — | `9b47e5c9` | `fix(gh#31 s010): the disintegrate arms read their own entity fields` |
| — | `c7da3493` | `process(gh#31 s010): the dead disintegrate arms amendment` |
| 6 | `ee2693d5` | `test(gh#31 s010): the disintegrate and volumetric golden` |
| — | `035334af` | `process(gh#31 s010): the renderfx golden bless amendment` |
| 7 | `92641c97` | `docs(gh#31 s010): the RF_NOSHADOW disposition` |
| 8 | `9f7e2ca4` | `process(gh#31 s010): finished file` |
| — | `8adbfa9a` | `fix(gh#31 s010): the lane-review findings` |
| — | `74480cdc` | `process(gh#31 s010): the lane-review walk amendment` |
| — | `7187ec04` | `process(gh#31 s010): the finished file fix-round update` |
| — | this commit | `process(gh#31 s010): the fix-round walk amendment` |

The last four are the fix round the lane-review walk of 2026-08-30 ordered, and the two-row ledger its own walk returned.

Gate results, per commit:

- **Commit 1.** `cargo build --workspace` green with the one expected unused-parameter warning on `target_texture`. `cargo test --workspace -- --test-threads=1` passed. All eighteen committed fixtures byte-identical across the five golden suites, which is the proof that the parameter threading is pure.
- **Commit 2.** Build green with two intermediate warnings, the unread `target_texture` and the unread `StageDrawItem::screen_image`. Workspace tests passed. All eighteen fixtures byte-identical. `scene_renderfx_tint.png` did not move, so row 5 did not fire and no re-bless was needed.
- **Commit 3.** Build green with two intermediate warnings, the unread `screen_image` field and the unread `ScreenImage::view`. Workspace tests passed. All eighteen fixtures byte-identical.
- **Commit 4.** Build green at zero warnings. Workspace tests passed. All eighteen fixtures byte-identical, which is the proof that the stage arm is inert on every committed scene.
- **Commit 5.** Build green at zero warnings. Workspace tests passed. All nineteen fixtures byte-identical, the eleven scene goldens now included.
- **The fix commit.** Build green at zero warnings. Workspace tests passed. All nineteen fixtures byte-identical, as predicted, because no committed scene submits either disintegrate flag.
- **Commit 6.** Build green at zero warnings. Workspace tests passed. All twenty fixtures byte-identical.
- **Commit 7.** Build green at zero warnings. Workspace tests passed. All twenty fixtures byte-identical, which is what a comment-only change must produce.

- **The fix round, `8adbfa9a`.** Build green at zero warnings. Workspace tests passed. All twenty fixtures byte-identical, and `git status --porcelain` was empty after the runs, so no `.actual.png` was written. Comment text and one arithmetic widening only, so no fixture could move.
- **The walk amendment, `74480cdc`.** Packet folder only, no code.
- **The finished file update, `7187ec04`, and the fix-round walk amendment.** Packet folder only, no code. Both gate on `cargo build --workspace` and `cargo test --workspace -- --test-threads=1`, which the last entry confirms green and passing.

Golden invocations used throughout, each as one serial foreground run: `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`, `--test world_golden -- --ignored --test-threads=1`, `--test entity_golden -- --ignored --test-threads=1`, `--test ghoul2_vertex_golden -- --ignored --test-threads=1`, and `--test hud_golden` both with and without `--ignored`.

The lockstep referee did not run. No commit touches `mp_game`, the server, or any `jampded` link-set crate.

## Assumptions and choices, keyed to their commits

**Commit 1. The headless callers clone the texture handle.** `Gpu::headless_texture` returns `&wgpu::Texture` as the contract specifies, and the executor takes `&mut Gpu`, so the two borrows would clash. The five headless test callers write `gpu.headless_texture().clone()`, which ends the borrow before the executor takes the gpu mutably. `wgpu::Texture` is an `Arc` handle in wgpu 30, so the clone is a handle copy. The windowed callers pass `&frame.texture` off the acquired `SurfaceTexture`, which is owned outside the gpu and needs no clone.

**Commit 2. `collect_stage_items` returns the deferred surface ranges.** The packet puts the partition in `Pipeline3d::draw`, and `draw` still owns it. It cannot rebuild the surface boundaries from `post_render_ent` alone, though: one entity contributes several adjacent draw surfs, so a contiguity test would merge them into one run and the last-in-first-out drain would then draw them in submission order inside that entity. `collect_stage_items` therefore returns one `Range<usize>` per deferred surface beside the item list, and `draw` partitions and reverses those ranges. `collect_stage_items` is a private helper and is not in the surface contract.

**Commit 2. `screen_image` is set where `EntityFx` is resolved.** The packet says `collect_stage_items` fills both new fields and that `screen_image` is `EntityFx::resolve`'s `distortion` bit. The bit is set at the four item-construction sites inside `build_stage_item` and `build_cpu_surface_stage_item`, where `fx` is already in hand. That yields the same value and it also covers a distortion surface past the `MAX_POST_RENDERS` cap, which never reaches the partition block. The fog, sky and dlight items take `false`.

**Commit 2. A multitextured stage never binds the screen image.** The two-texture collapse arm sets `screen_image: false`. The oracle's distortion arm sits in the single-texture branch, and a multitextured stage diverts to `DrawMultitextured`, which has no distortion handling (`oracle/codemp/renderer/tr_shade.cpp:2147-2150`).

**Commit 2. The per-item draw body moved into `draw_items`.** Two callers now share it, the sorted pass and each post-render pass. The body is unchanged, and the depth-window tracker starts at `(false, DepthRange::Normal)` per pass, which matches the pre-seam semantics: a run that never leaves the normal window sets no viewport and keeps the pass default.

**Commit 2. The `MAX_POST_RENDERS` counter counts draw surfs, not items.** The oracle enqueues before it dispatches the surface, so a draw surf that produces no stage item still consumes a queue slot. The port increments on the same test and only records a range when the surface produced items.

**Commit 3. The target texture's own dimensions stand in for `glConfig.vidWidth`/`vidHeight`.** The oracle reads the framebuffer size, and the copy rectangle is clamped against it. `target_texture.width()` and `.height()` are that size on both the windowed and the headless arm.

**Commit 3. Three divergence notes at their sites.** A zero or negative radius skips the capture, so the stage binds its own diffuse. A radius past a target dimension clamps the side length first, which keeps the rectangle inside the frame. Both are rule-19 choices over undefined oracle behavior.

**Commit 4. The captured group overrides the cached one.** The bind-group cache key gains the `screen_image` flag as the packet specifies, and the cached entry for a distortion stage holds its own diffuse. That entry is the fallback the packet names for a frame where the capture did not run. Each deferred surface additionally builds a group against the square just captured, and `draw_items` prefers it for that surface's screen-image items.

**Commit 4. The PBR backend keeps its own diffuse.** `GpuImages::view_bind_group` builds the two-texture world group, and the PBR pipeline takes a four-texture layout it does not fit. A distortion stage under `BackendMode::Pbr` therefore binds its own diffuse, with a site note. The packet grants only `view_bind_group`, so no PBR variant was added.

**Commit 4. A distortion stage takes the dynamic path.** `build_dynamic_block` carries the `v` flip, so a screen-image stage joins `fx.rewrites_colors()` and the fog modulation in forcing the per-frame block. Without it a static distortion stage would sample the captured square with unflipped coordinates.

**Commit 6. The scene needed two placements the packet did not anticipate.** Both are recorded in the packet's Amendments and in the commit body. `twinpodcc.md3` is 315 units deep, so at the sibling test's 260 units one copy spans about 700 pixels of the 800 pixel frame, and 600 units is what lets three copies share the image. At 600 units all three sit behind the duel1 wall, so the scene carries `RDF_NOWORLDMODEL`.

**Commit 7. The `_G2_GORE` deferral gained its own marker.** The old ghoul2 note covered the shadow pushes and the gore chain in one sentence. Splitting it for the marker convention would have dropped the gore subject, so it kept a marker of its own. Comment text only, and no gore work landed.

## Deviations

Three, each ratified by the user mid-lane and folded into the packet's Amendments section.

1. **The horizontal mirror does not exist.** The packet's capture section, divergence note 3, and row 4's defect condition all rested on the claim that `cX = vidWidth - x - (rad/2)` mirrors the capture square. `R_WorldCoordToScreenCoordFloat` projects onto `viewaxis[1]`, which `AnglesToAxis` fills with the left vector (`oracle/codemp/game/q_math.c:530-536`), and the GL view transform maps our `y` to GL `-x` (`oracle/codemp/renderer/tr_main.cpp:17-27`). The helper's `x` therefore counts from the right edge, and Raven's term converts it back, so the capture lands on the entity's own screen position. Confirmed against a rendered frame, where a backdrop sprite at world `y = -115` drew at screen x 252 rather than 68. Ratified 2026-08-30: divergence note 3 struck, row 4's mirror clause and its off-centre-sprite rationale struck, and row 4's `v`-flip defect reading inverted. No code changed, because the port already transcribes both terms as written.

2. **Both disintegrate arms were dead.** The packet states they are "already ported and live" and that this step adds no code for them. `lighting_ref_entity` built the `RefEntity` the colour short-circuits read and defaulted `renderfx`, `old_origin` and `end_time`, so `RB_CalcDisintegrateColors` matched neither branch and left its colour buffer at zeros, and `RB_CalcDisintegrateVertDeform` never fired. The `RF_DISINTEGRATE1` copy vanished behind its own forced alpha test and the `RF_DISINTEGRATE2` copy drew as a black silhouette. The user ratified the one-hunk fix in this lane, and it landed as `9b47e5c9` ahead of the golden. Every fixture committed before it stayed byte-identical.

3. **Two of row 4's defect clauses are unattainable on this model.** The `RF_DISINTEGRATE1` band clause and the `RF_DISINTEGRATE2` deform clause were struck. `RB_CalcDisintegrateColors` puts its three bands in a shell about `90/threshold` units thick, and `twinpodcc.md3` carries 702 vertices over a 315 unit span, so vertices sit roughly 30 units apart. At the threshold of 120 that cuts a visible hole the shell is 0.75 units thick and catches almost no vertex, and a lower threshold shrinks the sphere until it cuts no hole at all. The vert deform moves the vertices inside the sphere, and those are the ones that turn transparent. The hole and the hard edge both landed and are locked.

The lane-review walk of 2026-08-30 ratified four more, each recorded in the packet's Amendments.

4. **`screen_image` is false at three of five construction sites.** The contract writes the bit as `EntityFx::resolve`'s `distortion` bit everywhere. The fog pass binds `tr.fogImage` and a multitextured stage diverts to `DrawMultitextured`, which has no distortion arm, so both take `false`. The third is the non-dynamic single-texture arm, which commit 4 made unreachable when the bit is set. The clause now scopes the bit to the single-texture path.
5. **The PBR backend never binds the capture.** `view_bind_group` builds the two-texture world group and the PBR pipeline takes a four-texture layout, so a distortion stage under `BackendMode::Pbr` keeps its own diffuse. Ratified onto the divergence list.
6. **Five markers, not four.** The `_G2_GORE` deferral shared the note the row-8 conversion replaced, so it took a marker of its own rather than lose its greppable subject.
7. **Three forced surfaces.** `draw_items`, the paired return on `collect_stage_items`, and the two parameterized path helpers in `entity_golden.rs` are outside the contract's enumerated items and were accepted as mechanically forced.

The fix-round walk of 2026-08-30 ratified two more.

8. **The F9 note runs to three lines.** The divergence section asks for two or fewer. This site is the one exception, because the ratified F9 ruling named three facts and they do not fit two lines under the 150 column rule.
9. **The step-folder write scope covers `vet.md`.** The grant now reads `finished.md`, session-directed `packet.md` tail appends, and the vet's own record.

No other deviation. Every other row, scope line, write scope and gate stands as written.

## Open gaps

- **The bands and the vert deform have no image gate.** `entity_renderfx_duel1.png` locks the `RF_DISINTEGRATE1` hole, the `RF_DISINTEGRATE2` hard edge, and the `RF_VOLUMETRIC` fade. A denser model, for example the Ghoul2 stormtrooper the sibling test already boots, would resolve the three colour bands and the normal displacement. That is a scene of its own and was ruled out of this lane.
- **The distortion stage under the PBR backend binds its own diffuse.** It needs a four-texture variant of `view_bind_group`, which this packet did not grant.
- **The screen image is one slot.** The interleave consumes each capture in the pass that immediately follows it, so one slot serves any number of distortion entities. A future batching pass over the post-render list would break that and would need a slot per capture.
- **A distortion surface past the `MAX_POST_RENDERS` cap draws in the sorted pass.** It still binds the screen image if a capture stands behind it, which matches the oracle, where the stage arm reads `tr.screenImage` whatever the queue did. No committed scene reaches the cap.
- **`R_WorldCoordToScreenCoordFloat`'s vertical scale is Raven's own approximation.** It uses `90.0 / fov_y` where the real projection uses `tan(fov_y / 2)`, so a capture for an entity off the view centreline sits a pixel or two from the entity's rendered position. The port transcribes the approximation, and the distortion golden's sprite sits at `z = 0`, where the two agree.
- **Two stale `tr_surface.rs` doc comments** (`:108-116` and `:1217-1243`) that step-009 flagged stay untouched, as the packet's carried-forward section requires.
- **The `oldorigin` portal-view divergence note** from step-009's finished file stands untouched.
- **The vet's unverified list stands.** `.claude/packets/31/step-010/vet.md` names thirteen items no fixture exercises, including the windowed capture path, two distortion entities in one frame, the `MAX_POST_RENDERS` cap, the screen-image rebuild, a false projection, and the PBR arm. The fix round changed no coverage.
