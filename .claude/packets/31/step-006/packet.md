# Packet gh#31 step-006 - the mark fragments go live

## Scope

This step lights up the marks census group (DEC-54: `marks/MarkFragments`, 92,322 trap calls across the four traces, plus the three mark shaders `gfx/damage/rivetmark`, `gfx/effects/saberDamageGlow`, and `markShadow` at 443,515 poly submissions). The whole mark chain is already transcribed: `R_MarkFragments`, `R_BoxSurfaces_r`, `R_ChopPolyBehindPlane`, and `R_AddMarkFragments` sit in `crates/mp/renderer/src/tr_marks.rs`, and `RE_AddDecalToScene` sits in `crates/mp/renderer/src/tr_scene.rs`. None of it runs, because the walk targets a scoped-local `MarkNode` tree that nothing builds, and the `CG_CM_MARKFRAGMENTS` arm returns zero fragments (`crates/mp/engine/client/src/cl_cgame.rs:1685-1692`). This step retargets the walk onto the real world arena (`WorldAsset::nodes`, `surfaces`, `mark_surfaces`, `planes` - `crates/mp/renderer/src/render_state/placeholders.rs:449-514`), homes the walk's generation state on `RendererFrontend`, and wires the trap arm live. On the live client this restores impact marks, saber damage glow, and the blob player shadow, because cgame's `CG_ImpactMark` feeds every one of them through this trap and then draws them through the already-gated poly path (`RE_AddPolyToScene`, `tr_scene.rs:261`; SF_POLY resolve, `crates/mp/renderer-gpu/src/pipeline3d.rs:1561`).

The step does not touch the poly draw path, the dlight backend, the 2D path, the FX primitives, or any renderfx flag. It does not change `tr_bsp.rs`, `tr_world.rs`, or any `WorldAsset` field: the survey confirmed the arena already carries every field the walk reads, so the retarget is the field-merge the `tr_marks.rs` module note planned, not a world reshape.

Mechanics, grounded in the survey. The stand-ins `MarkNode`, `MarkSurface`, `MarkSurfaceData`, and `MarkGridVert` (`tr_marks.rs:158-229`) delete. The walk reads the real shapes instead: `Node.contents == -1` marks a decision node (`tr_bsp.rs:179-210`), `Node.plane` indexes `WorldAsset::planes`, a leaf's `firstmarksurface`/`nummarksurfaces` window `WorldAsset::mark_surfaces`, and each mark index resolves a flat `Surface` whose `data` is the owned `SurfaceData` enum (`tr_bsp.rs:2264-2303`). Surface and content flags resolve through `assets.shaders.get(surface.shader)` (`ShaderAsset::surface_flags`/`content_flags`, `render_state/shader_asset.rs:44-47`). The `msurface_t::viewCount` stamp moves to a `MarkState`-owned parallel array, the same W2-F4 pattern `WorldWalkScratch::surf_view_count` uses for the world walk. `BoxOnPlaneSideRef` wants `&mut cplane_t`, and the shared world is immutable, so the walk hands it a local plane copy: `BoxOnPlaneSide` only reads the plane (`crates/mp/qshared/src/shared/q_math.rs:56-95`), so the copy is behavior-identical, and one comment line at each site records that (porting-rules §10).

## Rulings, taken by the user 2026-08-05

Both rulings are settled and bind this packet. No open question remains.

**Ruling A - the decal arms (settled: wire them live).** `RE_AddDecalToScene` is fully transcribed (`tr_scene.rs:1001`), and its two arms are parked on the exact owner this step delivers: `CG_R_ADDDECALTOSCENE` (`cl_cgame.rs:2114-2121`) and `FxHost::AddDecalToScene` (`fx/fx_host.rs:377-386`) both carry `//TODO: Port RE_AddDecalToScene world root` with the reason "no carrier owns a root until the renderer census merges that node arena (gh#31)". This step is that merge, so the reason evaporates. The census traces recorded zero decal calls, but the transcription is complete and oracle-faithful, the arm is a thin argument shim, and leaving a fully ported function dark behind a stale reason costs more than it saves. The user ruled on the recommendation: both arms wire live in this step, and the wiring is unconditional in the surface contract and the commit bundle.

**Ruling B - the new golden's scene (settled: the duel1-floor rivetmark, the user's eyes at bless).** The new image golden projects one mark onto the duel1 floor: eye at the `world_duel1` fixture's position, a 16-unit-radius mark straight down under the eye, shader `gfx/damage/rivetmark` (the census's top poly shader, 292,999 submissions). The user reviews the blessed PNG before the commit that adds it, per the bless procedure below.

## Surface contract

**`MarkState` (`tr_marks.rs`) grows the stamp array and stays `Default`.**

```rust
#[derive(Default)]
pub struct MarkState {
    /// `tr.viewCount` as the decal walk reads it. `R_MarkFragments` bumps it once per call.
    pub view_count: i32,
    /// The per-surface stamp that replaces `msurface_t::viewCount` for this walk, indexed by the flat `WorldAsset::surfaces` index (W2-F4 pattern).
    /// `R_MarkFragments` resizes it with zeros when the world's surface count changes, so a map change resets every stamp.
    pub surf_view_count: Vec<i32>,
}
```

**`R_BoxSurfaces_r` retargets onto the arena.** The stand-in parameters delete, the list collects flat surface indices instead of cloned geometry (Raven stores `surfaceType_t *` pointers into the world; the index is this codebase's pointer replacement), and `mark` turns `&mut` because the stamps now live on it.

```rust
pub fn R_BoxSurfaces_r(
    world: &WorldAsset,
    shaders: &Arena<ShaderAsset>,
    node_index: usize,
    mins: vec3_t,
    maxs: vec3_t,
    list: &mut Vec<u32>,
    listsize: usize,
    dir: vec3_t,
    mark: &mut MarkState,
);
```

Behavior map, item by item: `node.contents == -1` keeps the decision-node test; `children: [Option<usize>; 2]` recurses by index; the plane test copies `world.planes[node.plane]` into a local for `BoxOnPlaneSideRef`; a leaf iterates `world.mark_surfaces[leaf.firstmarksurface .. leaf.firstmarksurface + leaf.nummarksurfaces as usize]`; the flag test reads the resolved `ShaderAsset` (`.expect` on the lookup - a loaded world surface always has a registered shader); the Face plane test copies `SurfaceFace::plane`; a non-Face non-Grid surface stamps and skips exactly as today; the double-add guard compares and stamps `mark.surf_view_count[surface_index]`.

**`R_MarkFragments` retargets the same way.** `world_root` deletes, `assets` arrives instead, and the fn guards the no-world case itself.

```rust
pub fn R_MarkFragments(
    assets: &RenderAssets,
    mark: &mut MarkState,
    points: &[vec3_t],
    projection: vec3_t,
    max_points: usize,
    point_buffer: &mut Vec<vec3_t>,
    max_fragments: usize,
    fragment_buffer: &mut Vec<markFragment_t>,
) -> i32;
```

At entry: `let Some(world) = assets.world.as_deref() else { return 0; }` (cgame only calls with a map loaded, so the guard is the arm's no-map degradation, not new behavior), then the stamp-array resize when `mark.surf_view_count.len() != world.surfaces.len()`, then the body unchanged except that the surface loop matches `&world.surfaces[i as usize].data`: `SurfaceData::Grid(grid)` reads `grid.width`/`grid.height`/`grid.verts[k].xyz`/`grid.verts[k].normal` (`drawVert_t`, `crates/mp/engine/qcommon/src/qfiles/draw_vert_t.rs:20-25`), and `SurfaceData::Face(face)` reads `face.plane`, `face.points[k].xyz` (`FaceVertex`), and `face.indices`. `Skip`, `Triangles`, and `Flare` fall to the ignore arm, the oracle's "all other world surfaces" case. The clip math, the plane construction, the `chunks_exact(3)` §19 divergence, and `R_AddMarkFragments`/`R_ChopPolyBehindPlane` do not change at all.

**`RE_AddDecalToScene` (`tr_scene.rs:1001`) drops `world_root: &mut MarkNode`.** It already receives `assets: &RenderAssets`, so the `R_MarkFragments` call site passes `assets` and `mark` and nothing else changes. Its doc paragraphs that narrate the `MarkNode` threading rewrite to the new shape.

**`RendererFrontend` (`renderer_frontend.rs:69`) gains one field.**

```rust
/// The decal walk's generation state (`tr.viewCount` and the per-surface stamps), which `R_MarkFragments` reads and bumps at trap time.
pub mark_state: MarkState,
```

Initialized `MarkState::default()` at every `RendererFrontend` construction site.

**The `CG_CM_MARKFRAGMENTS` arm goes live (`cl_cgame.rs:1685`).** Oracle: `re.MarkFragments(args[1], VMA(2), VMA(3), args[4], VMA(5), args[6], VMA(7))`, `oracle/codemp/client/cl_cgame.cpp:806`. The arm reads `num_points = args[1]` and the `vec3_t` slice at `VMA(2)`, the projection at `VMA(3)`, `max_points = args[4]`, `max_fragments = args[6]`, calls `R_MarkFragments` against `&*re.sim.published` and `&mut re.mark_state` with fresh local `Vec` buffers (the fn's documented empty-buffer precondition), then copies out: each returned `vec3_t` as three floats into the module's `float *` buffer at `VMA(5)`, each `markFragment_t` into the module buffer at `VMA(7)`, and returns the fragment count. The two borrows are disjoint fields of the one `re`, so no aliasing arises. The `//TODO: Port R_MarkFragments world root` marker deletes.

**The decal arms go live (Ruling A).** `CG_R_ADDDECALTOSCENE` (`cl_cgame.rs:2114`) unpacks its args per `oracle/codemp/client/cl_cgame.cpp:1027` and calls `RE_AddDecalToScene` with the seated `re` fields, and `FxHost::AddDecalToScene`'s live arm (`fx/fx_host.rs`) does the same through its view. Both `//TODO: Port RE_AddDecalToScene world root` markers delete.

**The new golden, in `world_golden.rs`.** A new `#[ignore]` test `golden_world_marks_duel1` boots `maps/mp/duel1.bsp` through the file's existing machinery, registers `gfx/damage/rivetmark`, computes fragments with `R_MarkFragments` for a 4-point quad of radius 16 projected straight down from under the fixture eye, builds the per-fragment `PolyVert`s with `CG_ImpactMark`'s texture math (`st = 0.5 + DotProduct(delta, axis) * 0.5 / radius`, `oracle/codemp/cgame/cg_marks.c:110-220`, cited in the test), submits them through `RE_AddPolyToScene`, renders, and compares `tests/goldens/world_marks_duel1.png` at `CHANNEL_TOLERANCE = 0`. The test asserts the fragment count is nonzero, so a silently empty walk can never pass as a matching empty image. Private helpers stay in the test file.

Anything not on this list is out of scope, and the agent must not add it. No new third-party crate (DEC-49-class rulings come from the user only). No `WorldAsset`, `Node`, `Surface`, or `SurfaceData` change. No `tr_bsp.rs`, `tr_world.rs`, `tr_marks`-adjacent backend, `FrameEvent`, cvar, or `#[repr]` change. No change to `RE_AddPolyToScene` or the SF_POLY draw path. No existing fixture changes: every committed golden and `ghoul2_verts_stormtrooper.bin` is read-only, and the one new PNG this step blesses is the only fixture it may create.

## Bless procedure for the new golden

1. Build and run the new test once with `JKA_GOLDEN_BLESS=1 cargo test -p mp_renderer_gpu --test world_golden golden_world_marks_duel1 -- --ignored --test-threads=1`. This writes `tests/goldens/world_marks_duel1.png` and passes.
2. Re-run without the bless variable and confirm the byte-identical pass.
3. STOP before the commit that adds the PNG. The user looks at the image and approves it. A mark must be visible on the floor; an image with no visible mark is a defect, not a blessable golden.

## Pause triggers, named for this step

- Any existing golden moves - world, scene, entity, or ghoul2 - in any byte or pixel. STOP: this step adds a frontend walk and two trap arms, and no existing scene contains a mark, so a moved golden means a shared path changed.
- The walk needs a `WorldAsset`, `Node`, or `Surface` field this contract does not name, or a shader lookup ever fails on a loaded world. STOP: widening the world surface is a ruling.
- The retargeted walk produces zero fragments in the new golden's scene. STOP: do not tune inputs to force output; the walk or the scene is wrong and the user decides which.
- The stand-in deletion breaks a caller this contract does not list. STOP and name it.
- Verification is `cargo build` / `cargo check` plus the golden suites, never rust-analyzer, which is stale in this workspace.
- `dedicated` stays `"0"` in every rig run.

## Commit bundle

1. **The arena retarget, inert.** The `tr_marks.rs` rewrite (stand-ins deleted, arena walk in, `MarkState` stamps), the `RE_AddDecalToScene` parameter swap, the `RendererFrontend::mark_state` field, and the module-note rewrite that records the merge as done. No live caller exists yet, so behavior is unchanged. Gates: `cargo build --workspace` with zero warnings, `cargo test --workspace`, both world goldens byte-identical (`cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`), the scene suite green (`cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`, no `--ignored`).
2. **The live arms.** The `CG_CM_MARKFRAGMENTS` arm and the two decal arms (Ruling A), with the markers deleted. Gates: `cargo build --workspace`, `cargo test --workspace`, both world goldens byte-identical, the scene suite green, the entity golden byte-identical (`cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`), the ghoul2 fixture byte-identical (`cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`).
3. **The marks golden.** The new test and its blessed PNG, after the user approves the image per the bless procedure. Gates: the full battery of commit 2 plus the new golden passing at tolerance zero.
4. **The finished file**, per the packet skill: assumptions keyed to commits, deviations or the word "none", the commit list with gate results, and open gaps.

Every commit uses `--no-gpg-sign`, a heading subject, an STE body, and no trailer of any kind. Gate results are written as plain sentences inside the body, so no line parses as a git trailer (step-005 lesson). All golden runs are serial with `--test-threads=1`, each as one foreground command with a long timeout. The lockstep referee is not required: no commit touches `mp_game`, the server, or any `jampded` link-set crate.

## Write scopes

Branch `gh31-step-006-marks`, cut from master.

- `crates/mp/renderer/src/tr_marks.rs` - the retarget rewrite.
- `crates/mp/renderer/src/tr_scene.rs` - the `RE_AddDecalToScene` signature and doc.
- `crates/mp/renderer/src/renderer_frontend.rs` - the `mark_state` field.
- `crates/mp/engine/client/src/cl_cgame.rs` - the `CG_CM_MARKFRAGMENTS` and `CG_R_ADDDECALTOSCENE` arms.
- `crates/mp/engine/client/src/fx/fx_host.rs` - the decal arm.
- `crates/mp/renderer-gpu/tests/world_golden.rs` and the new `crates/mp/renderer-gpu/tests/goldens/world_marks_duel1.png`.
- Any caller `cargo check` shows broken by the signature swaps, edit-only to pass the new shape.
- `.claude/packets/31/step-006/` for `finished.md`.

Everything else is read-only, including `oracle/`, `tr_bsp.rs`, `tr_world.rs`, every existing fixture under `tests/goldens/`, `ghoul2_verts_stormtrooper.bin`, and `~/Developer/jka/` beyond read-only pk3 reads.

## Disposition

Both rulings are settled and the packet is ready for the lane, which spawns only on the user's explicit go. After a clean lane-review: merge to master locally. No push, and no pull request.

## Amendments

**2026-08-05 - the draft awaits the user audit.**

**2026-08-05 - rulings A and B are taken.** The user ruled both open questions on the recommendations: ruling A wires the `CG_R_ADDDECALTOSCENE` and `FxHost::AddDecalToScene` arms live in this step, and ruling B accepts the duel1-floor rivetmark golden with the blessed image gated on the user's eyes. The decal wiring is now unconditional in the surface contract, the commit bundle, and the write scopes. No open question remains, and the packet is ready for the lane. The lane go is pending.

**2026-08-05 - lane-review closed with dispositioned findings.** The clerk walked the whole diff and reproduced every gate, with the new mark golden verified live at one fragment. Findings and dispositions:

- Two `#[allow(clippy::too_many_arguments)]` attributes are outside the contract's quoted signatures. Accepted: clippy is not a repo gate, the attributes are inert, and the parked clippy lane meets them later.
- The `build_refdef` and `record_scene` test-helper signature changes (confessed deviation 2) are private test machinery, not `pub` surface. Accepted.
- Four unconfessed fixture details are accepted: the `PerpendicularVectorMP` MP twin, the literal `0.0` orientation, the `register_shader` raw-pointer split on the `boot.rs` precedent, and the diagnostic `println`.
- The pitch-90 view, the measured `MARK_DROP = 64`, and the corrected oracle cites (deviations 1, 3, 4) fill gaps the packet left. Accepted.
- Style items close in one commit on the branch: the four over-150-column module-note lines in `tr_marks.rs`, the 80-column-wrapped doc block and the `cg_marks.c:110-211` cite in `world_golden.rs`, the continued wrapped sentence in `tr_scene.rs`, and the import order in `cl_cgame.rs`.
- Discarded clerk flags: the SAFETY noun-phrase form, the `#[ignore]` string, the assert-message form, and the preserved Raven comments are established repo convention.

The frozen record of the review is `docs/audits/2026-08-05-mark-fragments-landing.md`.
