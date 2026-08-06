# Packet gh#31 step-007 - the dlight projection pass

## Scope

This step lights up the dlight census group (DEC-54: `dlight/calls`, 112,514 submissions across the four traces). The frontend chain is live end to end: `RE_AddLightToScene` records the trap (`crates/mp/renderer/src/tr_scene.rs:625`), the executor replays the scene window into `dlight_t` values (`crates/mp/renderer-gpu/src/frame_exec.rs:767-775`), the world walk culls per-surface masks into `WorldWalkScratch::surf_dlight_bits` (`R_DlightSurface`, `crates/mp/renderer/src/tr_world.rs:547-580`, for world and brush-model surfaces alike), and every sort key carries the dlight-map bit (`R_DecomposeSort`, `tr_main.rs:868`). The one missing piece is the backend pass the `//TODO: Port ProjectDlightTexture` marker names (`crates/mp/renderer-gpu/src/pipeline3d.rs:1403-1411`): no decoder reads the masks, so every scene draws its lights dark.

This step ports both style passes behind the oracle's dispatch (`oracle/codemp/renderer/tr_shade.cpp:2320-2336`: `r_dlightStyle->integer > 0` runs `ProjectDlightTexture2`, otherwise `ProjectDlightTexture`; `r_dlightStyle` defaults to `"1"`, `oracle/codemp/renderer/tr_init.cpp:1098`, ported at `crates/mp/renderer/src/tr_init.rs:813`). Each pass runs per world draw surf whose mask is nonzero, after the surface's shader stages and before its fog pass, exactly where the oracle runs it.

The style-1 pass (`ProjectDlightTexture2`, `tr_shade.cpp:523-838`), per light: a per-vertex six-bit box clip against the light radius (`:566-604`), then per triangle a face normal from the edge cross product (`:628-637` - this pass derives its normals from positions), a backface and junk-triangle reject, a plane-distance reject at the radius, the `modulate = 1 - fac^2/radius^2` falloff, an orthonormal `e1`/`e2` basis scaled by `0.5/sqrt(radius^2 - fac^2)`, projected texcoords `dot(dist, e) + 0.5`, and the all-out-of-range triangle reject (`:638-716`). The style-0 pass (`ProjectDlightTexture`, `tr_shade.cpp:840-1180`), per light: a per-vertex projection along the dominant axis of the vertex normal - the greatest-axis scan with the zero-normal fallback (`:884-910`), the `dUse` distance scale with its `maxScale` and `0.1` clamps and the `lightScaleTolerance` flat-surface test (`:912-1017`), texcoords `0.5 + dist * scale`, clip bits from the texcoord range plus the height reject and the height modulate (`:1019-1051`), and triangles kept where the clip bits do not all overlap (`:1053-1068`). This is the pass that reads per-vertex normals, and ruling 2 widens `WorldVertex` to carry them. Both passes then share the same tail: the inert fog note, the `dStage` opaque-stage selection, and the two draw arms with the built-in dlight image (`RenderAssets::dlight_image`, built by `R_CreateDlightImage`, `crates/mp/renderer/src/tr_image.rs:2578`).

The step does not touch the frontend walk, `tr_world.rs`, `tr_light.rs`, the marks path, the 2D path, the FX primitives, or any renderfx flag. Entity-model surfaces (MD3, Ghoul2, sprites, lines, polys) never carry dlight bits in the oracle - only world and brush surfaces do - so nothing changes on any entity path.

One scoped-out corner, marked in code: a brush-model entity's surfaces do carry masks in `surf_dlight_bits` (`R_AddBrushModelSurfaces` stamps them, `tr_world.rs:1512-1560`), but their pass needs the light transformed into the entity's frame (`R_TransformDlights` per entity, `oracle/codemp/renderer/tr_backend.cpp` `RB_RenderDrawSurfList`), and this step transforms once with the world orientation only. The pass therefore gates on the world entity, and a `//TODO: Port ProjectDlightTexture2 bmodel-entity pass` marker with the reason line records the debt at the gate site. The four traces light world geometry; a mover under a dlight is the marked fog.

## Rulings, taken by the user 2026-08-05

Both rulings are settled and bind this packet.

**Ruling 1 - the `scene_dlights.png` disposition (settled: the fixture stays frozen, and a new world golden carries the proof).** The survey showed the scene image cannot change: the scene is `RDF_NOWORLDMODEL` (sprites plus three lights, `crates/mp/renderer-gpu/tests/scene_golden.rs:663-687`), and generated entity surfaces never carry dlight bits in the oracle - only world and brush surfaces reach the dlight passes (`tess.dlightBits` is ORed in only by the world surface functions). The user ruled on the recommendation: `scene_dlights.png` stays read-only and byte-identical, and after this step it is the leak guard that the pass does not touch entity surfaces. A moved pixel there is a STOP defect. The visible-light proof is the NEW world-map golden (`world_dlights_duel1.png`, below) with the step-006 bless-STOP, and the user's eyes gate that image.

**Ruling 2 - the style coverage (settled: BOTH styles port, overriding the style-1-only recommendation).** The user ruled that `ProjectDlightTexture` (style 0) ports beside `ProjectDlightTexture2` (style 1), behind the oracle's `r_dlightStyle` dispatch. The consequence: `WorldVertex` gains per-vertex normals for the style-0 projection, with the layout change and its expectation pinned in the surface contract below. No style-0 marker remains, and the `Warned::DlightStyle0` variant of the earlier draft is dropped from the contract.

## Surface contract

**`RenderCvarSnapshot` (`crates/mp/renderer/src/render_state/render_cvar_snapshot.rs`) gains one field**, filled in `from_cvars` from the registered handle (`RendererCvars::r_dlightStyle`, `renderer_cvars.rs:103-104`):

```rust
/// `r_dlightStyle` - default `"1"`. Style greater than zero runs the `ProjectDlightTexture2` pass; style zero is unported fog.
pub dlight_style: i32,
```

**`WorldVertex` (`pipeline3d.rs:118`) gains per-vertex normals (Ruling 2).** The field lands last in declaration order, so every existing attribute offset stays put:

```rust
/// The vertex normal, which only the CPU-side style-0 dlight projection reads.
/// No WGSL attribute maps it: the stride covers it and the GPU skips it.
normal: [f32; 3],
```

The layout consequence, pinned: the struct grows from 32 to 44 bytes with no padding (offsets 0, 12, 20, 28 unchanged, `normal` at 32), `Pod`/`Zeroable` stay derivable, and the one shared `build_world_pipeline` reads `array_stride` from `size_of::<WorldVertex>()` (`:2960`), so both backends' pipelines take the new stride automatically. `VERTEX_ATTRIBUTES` does not change, no WGSL input declares a new location (wgpu permits a stride beyond the covered attributes), and the static world plus per-frame dynamic vertex buffers grow 12 bytes per vertex. The fill sites, each exact: `build_world_mesh` writes the face plane normal replicated per vertex for a Face (the oracle floods `tess.normal` from `surf->plane.normal`, `oracle/codemp/renderer/tr_surface.cpp:1405-1506`), the `drawVert_t::normal` for Grid and Triangles verts, and `from_face_vertex` (`:128`) gains the plane normal as a parameter. The poly path fills zero with a one-line note (the oracle's `RB_SurfacePolychain` writes no `tess.normal`, and no dlight ever reads a poly). The MD3 decode fills the normal it already computes into its parallel slice, and the slice stays for the lighting evaluators. The generated entity geometry and the dlight collectors' own vertices fill zero. The binding expectation of ruling 2: every existing golden stays byte-identical through this widening, because no shader arm reads the new attribute and no existing CPU path changes an output. Any moved golden is a STOP.

**The dlights transform and thread into the backend.** In `render_world` (`frame_exec.rs`), after `R_RenderView` returns and before the draw, the local `dlights` vec transforms once with the world orientation: `R_TransformDlights(dlights.as_mut_slice(), &view.world)` (`crates/mp/renderer/src/tr_light.rs:89`). This is the oracle's world-entity transform site (`RB_RenderDrawSurfList` transforms per entity change; the world orientation is the one this step serves), and it fills the `dl.transformed` field the pass reads. The draw call then passes `&dlights` and `&self.walk_scratch.surf_dlight_bits` - two disjoint field borrows beside `self.pipeline3d`.

**`Pipeline3d::draw` (`pipeline3d.rs:1117`) and `collect_stage_items` (`:1413`) each gain two parameters**, threaded unchanged:

```rust
dlights: &[dlight_t],
surf_dlight_bits: &[i32],
```

**`collect_world_surface` (`pipeline3d.rs:1793`) appends the dlight items.** The `_dlight_map` decode binding renames to `dlight_map` and gates the hook. After the surface's shader-stage items and before its fog item (the oracle's order: stages, then dlights, then fog, `tr_shade.cpp:2288-2345`), the fn calls the collector when every gate holds: `dlight_map != 0`, `entity_num` is the world entity (the bmodel marker sits on this gate), `shader.sort <= SS_OPAQUE as f32`, and `shader.surface_flags & (SURF_NODLIGHT | SURF_SKY) == 0` (`tr_shade.cpp:2320-2321`). Inside the collector the style dispatch transcribes the oracle branch: `cvars.dlight_style > 0` runs the style-1 body, anything else runs the style-0 body (`tr_shade.cpp:2330-2336`). The triangles come from the exact index list the surface's stage items drew - the static range, or the grid's LOD-reduced dynamic block when the LOD path produced one - so the pass lights the same lattice the stages rendered, as the oracle's shared `tess` guarantees. The vertices come from the surface's `cpu` slice, whose new `normal` field feeds the style-0 projection.

**The new collector, private to `pipeline3d.rs`:**

```rust
/// Builds the additive dlight passes for one world surface: the `ProjectDlightTexture2` and `ProjectDlightTexture` transcriptions behind the `r_dlightStyle` dispatch.
/// One item per reaching light, appended after the surface's stage items and before its fog item.
/// Source: `oracle/codemp/renderer/tr_shade.cpp:523-1180,2330-2336`
#[allow(clippy::too_many_arguments)]
fn collect_dlight_items(
    &mut self,
    surf_bits: i32,
    dlights: &[dlight_t],
    shader: &ShaderAsset,
    cpu: &[WorldVertex],
    indexes: &[u32],
    entity_float_time: f32,
    assets: &RenderAssets,
    globals_offset: u32,
    dynamic_vertices: &mut Vec<WorldVertex>,
    dynamic_indices: &mut Vec<u32>,
    stats: &mut WorldStats,
    items: &mut Vec<StageDrawItem>,
);
```

The signature gains `dlight_style: i32` beside the listed parameters, and the fn also receives the surface's per-vertex normals through the `cpu` slice's new field. Per light `l` with `surf_bits & (1 << l) != 0`, the selected style body runs. Style 1: the clip-bits pass, the triangle loop, and the texcoord build transcribe `tr_shade.cpp:554-716` byte for byte on `dl.transformed` and the `cpu` positions, including the `SHADER_MAX_VERTEXES - 3` break, with per-triangle colors `myftol(color * 255 * modulate)` at alpha 255. Style 0: the per-vertex body transcribes `tr_shade.cpp:884-1068` on `dl.transformed`, the `cpu` positions, and the `cpu` normals - the dominant-axis scan, the clamped `dUse` scale, the `lightScaleTolerance` flat test, the texcoord and height clip bits, the per-vertex height modulate and colors, and the triangle keep on non-overlapping clip bits. Either body's surviving triangles append to the dynamic buffers as `WorldVertex` values (normals zero), and one `StageDrawItem` per light emits with `dynamic: true`, `index_dynamic: true`, `alpha_func: 0`, `depth_range: Normal`, `depth_far: false`, and the two oracle arms both styles share (style 0's tail at `:1074-1180` mirrors style 1's at `:722-819`):

- **The multitexture arm** (`tr_shade.cpp:741-799`), when the shader has a qualifying stage - the first active stage with a bundle image that is not a lightmap, has no tex mods, and whose `tc_gen` is neither environment nor fog, with zero blend bits in `state_bits` (`:747-756`). The item: `mode: MODE_DLIGHT` (new, below), `diffuse` = that bundle's image through `stage_image`, `lightmap` = `assets.dlight_image`, `st` = the surface's base texcoords (`tess.texCoords` bundle 0, `:692-697` - the `cpu` verts' own `st`), `lightmap_st` = the projected dlight texcoords, vertex color = the modulated light color, blend `GLS_SRCBLEND_ONE | GLS_DSTBLEND_ONE` with `depth_equal: true`, `depth_write: false` (`:793`).
- **The plain arm** (`tr_shade.cpp:800-819`): `mode: MODE_SINGLE`, `diffuse` = `assets.dlight_image`, `st` = the projected dlight texcoords, vertex color = the modulated light color, and the blend by `dl.additive`: `GLS_SRCBLEND_ONE | GLS_DSTBLEND_ONE` when set, `GLS_SRCBLEND_DST_COLOR | GLS_DSTBLEND_ONE` when not, both with `depth_equal: true`, `depth_write: false` (`:811-816`).

Both blends decode through the existing `blend.rs` GLS mapping, which already carries `GLS_SRCBLEND_DST_COLOR` and `GLS_DSTBLEND_ONE`. The oracle's `GL_FOG` disable around the pass (`:722-738`) is inert here: this backend has no global fog state to disable, fog is its own pass, and one comment line at the site records that. `stats` gains a `dlight_passes: u32` counter field on `WorldStats`, the port of `backEnd.pc.c_dlightIndexes` reduced to a pass count.

**`MODE_DLIGHT`, the one WGSL addition.** The existing `MODE_MULTITEXTURE` arm computes `diffuse.rgb * lightmap.rgb` and drops the vertex color (`crates/mp/renderer-gpu/src/shaders/world.wgsl:66-69`), so the oracle's `base * dlight * color` chain needs its own arm in `world.wgsl` and `world_pbr.wgsl`:

```rust
/// The dlight multitexture pass: bundle 0 times the dlight texture times the per-vertex light color, `ProjectDlightTexture2`'s `dStage` arm.
const MODE_DLIGHT: u32 = 2;
```

```wgsl
// world.wgsl and world_pbr.wgsl, beside the mode == 1u arm.
if (surface.mode == 2u) {
    let diffuse = textureSample(t_diffuse, s_diffuse, input.st);
    let dlight = textureSample(t_lightmap, s_lightmap, input.lightmap_st);
    color = vec4<f32>(diffuse.rgb * dlight.rgb * input.color.rgb, input.color.a);
}
```

A dlight item routes the PBR shader's faithful arms (`pbr_lit` stays zero), so both backends draw the pass identically.

**The markers move.** The `//TODO: Port ProjectDlightTexture` block at `pipeline3d.rs:1403-1411` deletes with its stale vertex-format rationale, and no style marker replaces it, because both styles port (Ruling 2). One marker lands at the gate: `//TODO: Port ProjectDlightTexture2 bmodel-entity pass` (the per-entity transform, `oracle/codemp/renderer/tr_backend.cpp` `RB_RenderDrawSurfList` dlight transform site), with its `// Source:` line. The `Warned` enum does not change.

**The doc comment in `scene_golden.rs` updates.** The `scene_dlights` recorder's doc (`tests/scene_golden.rs:654-662`) drops "ProjectDlightTexture is the remaining piece" and records the landed state: the pass is live, and this scene stays dark because a no-world scene has no dlight-receiving surface, which makes the golden the leak guard. The `Scene::expect_dlights` field doc (`:87-89`) drops its "not visible yet" clause the same way.

**The new golden, in `world_golden.rs` (Ruling 1).** A new `#[ignore]` test `golden_world_dlights_duel1` boots `maps/mp/duel1.bsp` through the file's existing machinery, records the same eye as `world_duel1`, adds three dynamic lights through `RE_AddDynamicLightToScene` before the scene renders - one plain warm light near the floor, one plain cool light near a wall, one additive - with radii large enough to reach visible geometry, renders, and compares `tests/goldens/world_dlights_duel1.png` at `CHANNEL_TOLERANCE = 0`. The test asserts `WorldStats::dlight_passes` is nonzero, so an inert pass can never pass as a matching unlit image.

Anything not on this list is out of scope, and the agent must not add it. No new third-party crate (DEC-49-class rulings come from the user only). No `WorldVertex` change beyond the one contracted `normal` field, no new WGSL attribute or vertex location, no `dlight_t` or any `#[repr]` change, no frontend change (`tr_world.rs`, `tr_light.rs`, `tr_scene.rs`, `tr_main.rs` are read-only), no `blend.rs` change, no `FrameEvent` variant, no new cvar registration (`r_dlightStyle` is already registered), and no fixture change of any kind: every committed golden including `scene_dlights.png` is read-only, and the one new PNG this step blesses is the only fixture it may create.

## Bless procedure for the new golden

1. Build and run the new test once with `JKA_GOLDEN_BLESS=1 cargo test -p mp_renderer_gpu --test world_golden golden_world_dlights_duel1 -- --ignored --test-threads=1`. This writes `tests/goldens/world_dlights_duel1.png` and passes.
2. Re-run without the bless variable and confirm the byte-identical pass.
3. STOP before the commit that adds the PNG. The user looks at the image and approves it. The lights must be visibly brighter pools on the duel1 geometry against the committed `world_duel1.png`; an image indistinguishable from the unlit fixture is a defect, not a blessable golden.

## Pause triggers, named for this step

- Any existing golden moves - world, scene (including `scene_dlights.png`), entity, or ghoul2 - in any byte or pixel. STOP: the pass must add light only where a world surface carries a mask, and none of the fixed scenes has one.
- The new golden shows no visible light at bless time. STOP: do not tune radii past the contract values without a ruling; the pass or the frontend masks are wrong and the user decides which.
- Any golden moves at the commit that widens `WorldVertex`. STOP: ruling 2's binding expectation is byte-identity through the widening, so a moved image means the stride or a fill site leaked into an existing path.
- The dlight item cannot express the oracle chain through `MODE_DLIGHT` plus the existing blend decode, or a needed blend factor is missing from `blend.rs`. STOP: the GPU surface is this packet's contract and widening it is a ruling.
- The pass turns out to need a field this contract does not thread (an extra cvar, a per-entity orientation). STOP: that is the bmodel territory this packet marks as fog, or a contract gap the user rules on.
- Verification is `cargo build` / `cargo check` plus the golden suites, never rust-analyzer, which is stale in this workspace.
- `dedicated` stays `"0"` in every rig run.

## Commit bundle

1. **The plumbing, inert.** The `WorldVertex::normal` field with every fill site (ruling 2), the `dlight_style` snapshot field, the `R_TransformDlights` call in `render_world`, the two new parameters through `draw` and `collect_stage_items`, the `MODE_DLIGHT` constant and both WGSL arms, and the `WorldStats::dlight_passes` counter. No item is emitted yet and nothing reads the new attribute, so behavior is unchanged. Gates: `cargo build --workspace` with zero warnings, `cargo test --workspace`, both world goldens byte-identical (`cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`), the scene suite green (`cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`, no `--ignored`), the entity golden byte-identical (`cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`), the ghoul2 fixture byte-identical (`cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`) - the full battery, because the vertex widening touches every draw path and ruling 2 binds byte-identity through it.
2. **The passes.** `collect_dlight_items` with both style bodies and the shared arms, the gate in `collect_world_surface` between stages and fog, the marker moves, and the `scene_golden.rs` doc updates. Gates: `cargo build --workspace`, `cargo test --workspace`, both world goldens byte-identical, the scene suite green with `scene_dlights.png` byte-identical, the entity golden byte-identical, the ghoul2 fixture byte-identical.
3. **The dlight golden.** The new test and its blessed PNG, after the user approves the image per the bless procedure. Gates: the full battery of commit 2 plus the new golden passing at tolerance zero.
4. **The finished file**, per the packet skill: assumptions keyed to commits, deviations or the word "none", the commit list with gate results, and open gaps.

Every commit uses `--no-gpg-sign`, a heading subject, an STE body, and no trailer of any kind. Gate results are written as plain sentences inside the body, so no line parses as a git trailer (step-005 lesson). All golden runs are serial with `--test-threads=1`, each as one foreground command with a long timeout. The lockstep referee is not required: no commit touches `mp_game`, the server, or any `jampded` link-set crate.

## Write scopes

Branch `gh31-step-007-dlights`, cut from master.

- `crates/mp/renderer-gpu/src/pipeline3d.rs` - the collector, the gate, the mode constant, the markers, the stats field.
- `crates/mp/renderer-gpu/src/frame_exec.rs` - the transform call and the two threaded arguments.
- `crates/mp/renderer-gpu/src/shaders/world.wgsl` and `world_pbr.wgsl` - the `mode == 2u` arm.
- `crates/mp/renderer/src/render_state/render_cvar_snapshot.rs` - the `dlight_style` field.
- `crates/mp/renderer-gpu/tests/scene_golden.rs` - doc comments only, no scene or fixture change.
- `crates/mp/renderer-gpu/tests/world_golden.rs` and the new `crates/mp/renderer-gpu/tests/goldens/world_dlights_duel1.png`.
- Any caller `cargo check` shows broken by the new arities, edit-only to pass the new shape.
- `.claude/packets/31/step-007/` for `finished.md`.

Everything else is read-only, including `oracle/`, `tr_world.rs`, `tr_light.rs`, `tr_scene.rs`, `tr_main.rs`, `blend.rs`, every existing fixture under `tests/goldens/` (with `scene_dlights.png` named explicitly), `ghoul2_verts_stormtrooper.bin`, and `~/Developer/jka/` beyond read-only pk3 reads.

## Disposition

Both rulings are settled, step-006b is merged (`f3b3239f`), and the post-006b audit passed with the go given 2026-08-05. The packet is ready for the lane. After a clean lane-review: merge to master locally. No push, and no pull request.

## Amendments

**2026-08-05 - the draft awaits the user audit.**

**2026-08-05 - lane-review closed with dispositioned findings.** A Fable spot check (user-ordered) walked both style bodies line by line against the oracle and found zero float-width or operator divergences. All six confessed deviations verified sound, and the `depth_bias: shader.polygon_offset` key is the faithful reading of the oracle's offset-enable window, not a deviation. The full gate battery re-ran green under the reviewer's own runs, and the blessed golden shows the light. Dispositions: the style-0 texcoord divergence is accepted with near-nil blast radius (style 0 never runs in a default configuration, and the oracle's own binding there samples stale coordinates its comment calls wrong); the per-surface vertex cap versus the oracle's tess batching is recorded as a standing backend architecture note, unexercised by any fixture; six over-150-column comment lines close in one style commit on the branch, and the packet's own contract text seeded the longest, so future packets keep their sketched doc lines under the limit.

**2026-08-05 - rulings 1 and 2 are taken, and step-006b runs first.** The user ruled: ruling 1 on the recommendation - `scene_dlights.png` stays frozen as the leak guard, and the new `world_dlights_duel1.png` golden carries the visible-light proof under the bless-STOP. Ruling 2 overrides the recommendation - BOTH styles port: `WorldVertex` gains a trailing CPU-read `normal: [f32; 3]` (32 to 44 bytes, no new WGSL attribute, byte-identity binding on every existing golden), the style-0 `ProjectDlightTexture` body ports beside style 1 behind the oracle's `r_dlightStyle` dispatch, no style marker remains, and the `Warned::DlightStyle0` variant is dropped. The brush-model-entity arm stays named fog as drafted. The disposition is ready-for-audit-after-006b, not ready-for-lane.
