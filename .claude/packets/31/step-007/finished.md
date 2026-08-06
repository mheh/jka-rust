# Finished gh#31 step-007 - the dlight projection pass

Branch `gh31-step-007-dlights`, cut from master after `git merge master --no-gpg-sign`.

## Assumptions and choices, keyed to their commits

**Commit 1 (the plumbing, inert).**

- `WorldVertex` grows from 32 to 44 bytes with no padding, and five static asserts pin the size plus all five field offsets. The four mapped attributes keep offsets 0, 12, 20 and 28, and `normal` lands at 32.
- `VERTEX_ATTRIBUTES` does not change. One comment above it records that the stride covers `normal` and the GPU steps over it.
- `from_face_vertex` takes the plane normal as a second parameter. A face has no per-vertex normal, and `RB_SurfaceFace` floods `surf->plane.normal` across the whole surface.
- The fog block and the dynamic stage block both copy the source vertex's normal. Both are per-frame rewrites of the same surface, so carrying the normal keeps every copy of one vertex consistent.
- The sky box, the sky clouds, the poly fan, the sprite quad, the line quad and the ghoul2 decode all fill a zero normal. None of them can receive a dlight.
- The MD3 decode fills the normal it already computes and keeps its parallel `normals` slice, which the lighting evaluators still read.
- `MODE_DLIGHT` carried `#[allow(dead_code)]` for this commit only. The pass that selects it lands in commit 2, and commit 1's gate is a zero-warning build.
- `R_TransformDlights` runs once in `render_world`, right after `R_RenderView` returns. The oracle transforms per entity change inside `RB_RenderDrawSurfList`, and the world orientation is the one frame this step serves.
- `collect_world_surface` takes the two new parameters underscore-prefixed in this commit, so the inert threading builds with no warning. Commit 2 renames them.

**Commit 2 (the passes).**

- `collect_dlight_items` is a method on `Pipeline3d`, and the two style bodies are free functions beside it. Each returns an `Option<DlightPass>`, which is the surviving vertices plus their 0-based indices.
- `DlightPass` carries the surface's own base texcoords in `st` and the projected dlight texcoords in `lightmap_st`, so both draw arms read one block.
- The plain arm draws with `MODE_SINGLE` and `tex_from_lightmap: true`, which makes the shader read the projected texcoords from `lightmap_st` while the diffuse slot binds the dlight image. That reproduces the oracle's single-unit bind with no second vertex block.
- The multitexture arm draws with `MODE_DLIGHT`, the qualifying stage's image in the diffuse slot and the dlight image in the lightmap slot.
- `reads_lightmap` is false on both arms. The dlight image is not a lightmap, so `WorldStats::lightmapped` keeps counting the real lightmapping path only.
- The three oracle locals `maxScale`, `maxGroundScale` and `lightScaleTolerance` became module-level constants with their own source cites, so each carries a doc comment.
- Style 0's `scale` is declared per vertex rather than per function. The oracle declares it outside the vertex loop, but all three dominant-axis arms write it before the read, so the carry is dead.
- The triangle loops walk `chunks_exact(3)` over the caller's index slice. The style-1 loop breaks at `SHADER_MAX_VERTEXES - 3` emitted vertices, the same cap the oracle applies to its `numIndexes` counter.
- The style-1 texcoord build runs one loop over the three corners. Each iteration recomputes `dist` fresh in the oracle's order, so no per-component expression is hoisted.
- `collect_world_surface` keeps the LOD-reduced index block in a local, because the dlight passes must light the same lattice the stage items drew. A surface with no LOD reduction hands the static block from `geometry.cpu_indices`.
- The gate reads the mask from `surf_dlight_bits` at the world surface index and skips the collector when the mask is zero. `dlight_map` from the sort key is the flag, not the mask.

**Commit 3 (the dlight golden).**

- The three lights are declared as one `const` table of eye-relative offsets, so the fixture reads as data.
- The first light drops 56 units under the eye. The marks fixture already measured the duel1 floor at 64 units under the eye, so this light sits just above a surface known to exist.
- The radii are 200 to 250. The room is small, so those reach the walls and the floor in frame.
- `require_dlights` prints the pass count as well as asserting it. The count is part of the bless evidence.

## Deviations

1. **`MODE_DLIGHT` carried `#[allow(dead_code)]` through commit 1 only.** The packet places the constant in commit 1, where nothing selects it, and commit 1's gate is a zero-warning build. Commit 2 removes the attribute.
2. **The ghoul2 decode fills a zero normal.** The contract enumerates the MD3 decode as a fill site and does not name ghoul2, so the literal reading applies. A ghoul2 surface never carries a dlight mask, and its parallel `normals` slice stays its only normal home. One comment at the site records this.
3. **Style 0's multitexture arm binds the surface's base texcoords, not `tess.svars.texcoords[0]`.** The oracle binds the last stage's computed texcoords on this arm, while style 1 binds the raw ones. The packet contracts one shared arm reading the `cpu` verts' own `st`, so both styles do. A two-line note on `project_dlight_texture` records it.
4. **`dlight_stage_bundle` returns the bound bundle instead of the stage.** The oracle's stage-selection predicate and its bundle-bind predicate are the same test, so the two steps collapse into one with no behavior change. The fn doc records both oracle line ranges.
5. **`run_golden_scene` gains a sixth parameter, `require_dlights`,** under `#[allow(clippy::too_many_arguments)]`. The three older fixtures pass `false` and their goldens did not move.
6. **A dlight item keys `depth_bias: shader.polygon_offset`,** which the contract does not state. The oracle enables `GL_POLYGON_OFFSET_FILL` at the head of `RB_StageIteratorGeneric` and keeps it on through the fog pass, so a decal's dlight pass biases with its stages. This matches the fog-pass precedent in the same file.

## Pause triggers hit

None. No existing golden moved a byte at any commit, `scene_dlights.png` included, and the widening commit held byte-identity across the whole battery as ruling 2 binds. The new golden showed visible light at bless time and needed no radius tuning. The dlight item expressed the oracle chain through `MODE_DLIGHT` plus the existing GLS decode, so `blend.rs` needed no change. The pass needed no field the contract did not thread.

## Commits and gate results

1. `628ee500` **feat(gh#31 s007): the dlight plumbing, inert**
   - `cargo build --workspace`: green, zero warnings.
   - `cargo test --workspace`: green.
   - `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`: 3 passed, byte-identical.
   - `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`: 7 passed, `scene_dlights.png` byte-identical.
   - `cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`: 1 passed, byte-identical.
   - `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`: 1 passed, byte-identical.

2. `b3de5f6c` **feat(gh#31 s007): the two dlight projection passes**
   - `cargo build --workspace`: green, zero warnings.
   - `cargo test --workspace`: green.
   - `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`: 3 passed, byte-identical.
   - `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`: 7 passed, `scene_dlights.png` byte-identical.
   - `cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`: 1 passed, byte-identical.
   - `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`: 1 passed, byte-identical.
   - `grep -rn "TODO: Port ProjectDlightTexture"`: one hit, the bmodel-entity marker at the gate.

3. `35022be2` **test(gh#31 s007): the dlight world golden**
   - `cargo build --workspace`: green, zero warnings.
   - `cargo test --workspace`: green.
   - `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`: 4 passed at `CHANNEL_TOLERANCE = 0`. The three older goldens are byte-identical and `world_dlights_duel1.png` is the new one.
   - `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`: 7 passed, `scene_dlights.png` byte-identical.
   - `cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`: 1 passed, byte-identical.
   - `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`: 1 passed, byte-identical.
   - The bless ran once with `JKA_GOLDEN_BLESS=1`, then re-ran clean and passed byte-identical. The run reports 20 dlight passes. The user approved the image before this commit, per the packet's bless procedure.

Every golden run was one foreground command with `--test-threads=1`, and `dedicated` stayed `"0"` in every rig run. The lockstep referee was not run: no commit touches `mp_game`, the server, or any `jampded` link-set crate.

## Open gaps

- A brush-model entity's surfaces carry dlight masks, and their pass needs the light transformed into the entity's own frame. This step transforms once with the world orientation, so the gate keeps the pass on the world entity and `//TODO: Port ProjectDlightTexture2 bmodel-entity pass` at the gate names the debt. A mover under a dlight draws unlit.
- Style 0 never runs in a default frame. `r_dlightStyle` ships at 1, and no golden sets it to 0, so `project_dlight_texture` has no image gate. Only its transcription against the oracle stands behind it.
- The style-0 multitexture arm's texcoord source is a knowing divergence, listed as deviation 3. It only shows on a shader whose first opaque stage carries a tcMod, drawn at `r_dlightStyle 0`.
- The dlight passes append one draw item per light per surface, and the duel1 fixture reports 20 for three lights. A scene with many lights over many surfaces multiplies the item count, and no batching stands between the collector and the draw loop.
- Live play is the remaining gate. The fixed-scene golden covers the projection and the two draw arms, not the cgame paths that submit weapon fire, saber glow and explosion lights.
