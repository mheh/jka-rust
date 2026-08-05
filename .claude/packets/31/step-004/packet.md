# Packet gh#31 step-004 - the draw-arm migration

## Scope

This step migrates the entity draw arms and their backend decoders off the sim-side `RenderModels` registry and onto the published `PublishedModel` entries, per DEC-65 ruling 1 and the two pre-taken rulings below. It delivers five things: the view-helper surface on `PublishedModel`, the migration of `r_add_md3_surfaces`, `r_add_ghoul_surfaces`, the `tr_main.rs` dispatch, and the two `pipeline3d.rs` decoders, the deletion of the `models: &RenderModels` plumbing end to end, the harness publication drain, and the entity image golden that proves players on screen. Ground truth is the survey record `docs/audits/2026-08-04-step-004-draw-arm-migration-survey.md`.

Two rulings from the step-003 packet Amendments are pre-taken and bind this packet:

- **Ruling 3 (taken 2026-08-04).** The migrated arms read the published copy through borrowed view helpers on `PublishedModel`, not through direct `(Arc<ModelBlock>, usize)` reads at every site. One SAFETY home holds the base-plus-offset casts.
- **Ruling 4 (taken 2026-08-04).** Entity `model_type` resolves from the published entry, which republishes at every `RE_EndFrame` drain. `BModelTable` keeps only its brush-submodel fields.

The step does not execute DEC-65 ruling 2: the ghoul2 skeleton transform stays sim-side inside `r_add_ghoul_surfaces`, no bone matrix crosses in the frame package, and the ghoul2 legs keep their `EngineHostView` gate. The step does not touch renderfx census flags, mark shaders, dlight projection, the HUD set, FX, the shadow and gore DEFERRED blocks, `re_get_model_bounds`, or any live-client cgame work. `mdxa_view()` does not land, because no consumer exists this step (survey section B).

**Two binding facts from the step-003 close-out.** First, the ghoul2 draw order is NOT free to move: the fixture `ghoul2_verts_stormtrooper.bin` has matched byte for byte under three configurations, because every stormtrooper surface carries an empty shader name and both register legs write `shaderIndex = 0`. The fixture is the byte-identical gate through this whole migration, and any order or content move is a defect signal, never a re-bless candidate. Second, the frame-pinned registry clone at the four `execute_frame` sites is a soundness mechanism and must not be disturbed.

## Open rulings for the user

**Ruling A, the proof gate (binds this packet).** The world goldens draw no entities, the scene goldens draw no `RT_MODEL`, and the ghoul2 golden locks vertex streams, not pixels, so after the migration the newly ungated MD3 path has no gate. The fork: (A) a new entity image golden, one test file `tests/entity_golden.rs` on the `world_golden.rs` pattern that boots duel1, registers `models/map_objects/bespin/twinpodcc.md3` and inits `models/players/stormtrooper/model.glm`, adds both as `RT_MODEL` entities at fixed origins under the frozen clock (`FROZEN_TIME_MS = 12345`, the zero-radius LOD-0 pin from `ghoul2_vertex_golden.rs:404-418`), and compares one committed PNG at `CHANNEL_TOLERANCE = 0`; (B) an image read-back added to the existing ghoul2 vertex golden, near-zero new code but no MD3 coverage and two gates coupled in one test; (C) no new gate, byte-identity only, which proves the migration moved nothing but never proves the MD3 arm draws. Costs: A is one new test file reusing two existing recipes, one PNG fixture, one bless run under review; B is a scene change to a locked test; C is free and blind. **Recommendation: A**, the DEC-54 verification shape, both arms in one scene, with the existing vertex golden still isolating the ghoul2 stream when the image moves.

**Ruling B, the per-slot `re` aliasing discipline (recorded as deferred, not open here).** Step-004 adds no in-frame hook, no new cast of the seated slot, and no console-handler execution: the migration deletes parameters and reads `assets`, the drain uses direct field access outside any view, and `hook_install.rs` is untouched, so the two ghoul2 registration hooks stay the only in-frame `re`-slot casts under the existing per-slot rule (`hook_install.rs:7-11`). The evidence is survey section E. The ruling comes due with the first step that adds an in-frame hook or runs a console handler in the rig.

## Surface contract

**`PublishedModel` (`crates/mp/renderer/src/render_state/model_blocks.rs`)** gains one field and two methods:

```rust
/// The registered model name, which the bad-frame warning prints.
pub name: String,

impl PublishedModel {
    /// The `md3Header_t` one loaded LOD publishes, `None` for an absent slot.
    pub fn md3_ptr(&self, lod: usize) -> Option<*const md3Header_t>;

    /// The DEC-35 view over the published `.glm` block, `None` for a model with no mdxm block.
    pub fn mdxm_view(&self) -> Option<MdxmView<'_>>;
}
```

Both methods compute `block.base_ptr().add(offset)` from the stored pair and carry the step's one SAFETY home: the offset was computed at mark time from the finished `model_t` pointer, the block is immutable while shared (`model_block.rs:127-141`), and the result is valid while the entry's `Arc` lives, which the borrow on `self` guarantees. `mark_block` (`tr_model/render_models.rs:391-413`) fills `name` from the slot's registered name. No `mdxa_view`.

**`r_add_md3_surfaces` (`crates/mp/renderer/src/tr_mesh.rs:352`)** loses its `models: &RenderModels` parameter and resolves the entry from `assets.models` (`:372`). The `numFrames` read (`:376`), the LOD header (`:406`), and the warning name (`:396`) read through the entry. The private `r_compute_lod` (`:210`) takes the entry in place of `&model_t`. `r_cull_model`, `r_compute_fog_num`, the surface walk, and the `Md3SurfaceRef` push are unchanged in shape.

**`r_add_ghoul_surfaces` (`crates/mp/renderer/src/tr_ghoul2.rs:2254`)** loses its `models: &RenderModels` parameter. `inst.model` resolves through `assets.models`, with a `continue` on an absent entry. `g2_compute_lod` (`:668`, `pub`) takes `num_lods: i32` (or the entry) in place of `current_model: &model_t`. `render_surfaces` (`:840`, `pub`) takes the published entry plus the model handle in place of `current_model: &model_t`, reads the hierarchy through `mdxm_view()`, and stores the handle in `G2SurfaceRef` (`:920`). The host reads (`g2_setup_model_pointers_v` `:2285`, `g2_construct_render_skeleton` `:2327`) are unchanged. The sim-side bolt and bone-list callers of `mdxm_view_of` (`:482,:615,:801,:973,:1051`) are unchanged, and `mdxm_view_of` itself stays for them.

**The `tr_main.rs` dispatch (`:1961-2140`)** per ruling 4: the brush test becomes `bmodels.get(hModel).bmodel_index >= 0` and runs first, then `model_type` resolves from `assets.models.get(hModel)`, `MOD_BAD` when absent. `BModelEntry` (`render_state/bmodel_table.rs:14-24`) drops its `model_type` field and keeps `bmodel_index` and `bsp_instance`. The host gate (`:2027-2035`) narrows to the two ghoul2 legs, its warning text updates to name ghoul2 only, and the `//TODO: Port R_AddEntitySurfaces MD3 and Ghoul2 arms render-side` marker (`:2020`) rewrites to a ghoul2-only marker citing DEC-65 ruling 2. The `MOD_MESH` arm and the `MOD_BAD` null-axis fallthrough run ungated.

**`EntityWalkHost` (`tr_main.rs:175-178`) dissolves.** The `models` field deletes, and the remaining one-field wrapper is replaced by `Option<&mut EngineHostView>` through `R_RenderView`, `frame_exec::render_scene`, and `FrameExecutor::execute_frame` (`frame_exec.rs:435`). The four construction sites (`tests/ghoul2_vertex_golden.rs:487`, `tests/world_golden.rs:256`, `tests/scene_golden.rs:427`, `src/bin/world_harness.rs:941`) pass the view directly.

**The `pipeline3d.rs` plumbing deletes.** `draw` (`:1130`) and `collect_stage_items` (`:1427`) lose `models: Option<&RenderModels>` and the two W2-F8 gates (`:1473`, `:1501`). `collect_md3_surface` (`:2011`) and `collect_ghoul2_surface` (`:2176`) lose `models: &RenderModels`. `decode_md3_surface` (`:4048`) and `decode_ghoul2_surface` (`:4117`) resolve the entry from `assets.models` (passed as `&ModelBlocks` or through `assets`), keep their skip-and-count `None` arms, and keep `eval_render` reading the sim-confined bone cache. `frame_exec.rs` deletes `backend_models` (`:809`) and the draw argument (`:866`).

**The harness publication drain.** Each entity-drawing `execute_frame` caller mirrors the `RE_EndFrame` drain pair (`tr_cmds.rs:356-358`) BEFORE the frame-pinned `Arc::clone`:

```rust
if let Some(blocks) = host.models.publish_blocks() {
    host.re.sim.publish_models(blocks);
}
let pinned = Arc::clone(&host.re.sim.published);
```

Sites: the four existing `execute_frame` callers and the new entity golden. The ordering is a contract line: a drain after the pin publishes into a generation the frame does not read. The pin itself is unchanged at all four sites.

**The entity image golden (on ruling A's answer; written for option A).** New `crates/mp/renderer-gpu/tests/entity_golden.rs` and fixture `tests/goldens/entity_duel1.png`: the duel1 boot, `boot::register_model` for `models/map_objects/bespin/twinpodcc.md3`, the `init_ghoul2` recipe for `models/players/stormtrooper/model.glm`, both entities at fixed origins in front of the eye under the frozen clock, zero entity radius to pin LOD 0, one frame, `read_target_rgba`, `CHANNEL_TOLERANCE = 0`, `#[ignore]`, bless via `JKA_GOLDEN_BLESS=1`, mismatch writes `entity_duel1.actual.png`. Exact origin constants are free at lane time inside the frozen-clock pattern. The module doc records the bless provenance.

**Doc lines**: the `EntityWalkHost` story moves into the `R_AddEntitySurfaces` ghoul2 gate comment, the `BModelTable` module doc drops its "arms stay gated off render-side" paragraph and records the ruling 4 split, and `execute_package`'s "MD3 and Ghoul2 entity arms stay dark" comment (`frame_exec.rs:363-370`) updates to ghoul2-only.

Anything not on this list is out of scope, and the agent must not add it. No new third-party crate. No `RE_*` signature change, no `FrameEvent` variant, no cvar, no trap arm, no `#[repr]` change, no `hook_install.rs` change, no change to `ModelBlock`, `ModelBlocks`, `RenderAssets`, `RenderAssetsSim`, or `RE_EndFrame`.

## Pause triggers, named for this step

- The ghoul2 vertex golden is not byte-identical at any commit, in order, content, or count. STOP: the migration moved behavior it must not move, and the fixture is not a re-bless candidate.
- A world or scene golden moves. STOP: no entity draws in those scenes, so the migration touched a shared path.
- The entity golden or the ghoul2 golden comes up blank, or the 22-surface assert fires. STOP and check the drain placement against the pin before anything else (survey section G, the missed-drain hazard).
- Any migrated read turns out to need a `model_t` field that `PublishedModel` does not carry, beyond the `name` field this contract adds. STOP: the entry's surface is a step-002 type and widening it is a ruling.
- The step turns out to need an in-frame `re`-slot cast, a new hook, or a console-handler execution. STOP: that fires the deferred aliasing ruling (open ruling B).
- The dispatch rewrite needs `model_type` kept on `BModelEntry` for some reader this survey missed. STOP: ruling 4 said brush-submodel fields only.
- `dedicated` must stay `"0"` in every rig run. A nonzero value stubs images and masks the register-path guards, so it is never a shortcut.

## Commit bundle

1. **The view-helper surface.** `PublishedModel` gains `name`, `md3_ptr`, `mdxm_view` with the SAFETY home, and `mark_block` fills the name. A unit test beside the step-002 publication test covers the helpers: mark a slot, assert `md3_ptr`/`mdxm_view` reproduce the `model_t` pointers, and assert `None` on absent slots. Gates: `cargo build --workspace`, `cargo test --workspace`.
2. **The sim-arm migration.** `tr_mesh.rs`, `tr_ghoul2.rs`, the `tr_main.rs` dispatch with the ruling 4 split and the narrowed gate, `BModelEntry` drops `model_type`, `EntityWalkHost` dissolves, the four caller sites re-spell, and the harness drain lands before each pin. Gates: `cargo build --workspace`, `cargo test --workspace`, the ghoul2 golden byte-identical (`cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`), both world goldens byte-identical (`cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`), the scene goldens green (`cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`, no `--ignored`).
3. **The decoder migration and the plumbing deletion.** `pipeline3d.rs` decoders and parameters, `frame_exec.rs` `backend_models`, the doc lines. Gates: the full battery of commit 2.
4. **The entity image golden.** `tests/entity_golden.rs`, the blessed `entity_duel1.png`, the module-doc provenance. Gates: the full battery of commit 2 plus the new golden green on a clean confirming run (`cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`).
5. **The finished file**, per the packet skill: assumptions keyed to commits, deviations or the word "none", the commit list with gate results, and open gaps.

Every commit uses `--no-gpg-sign`, a heading subject, an STE body, and no trailer of any kind. All golden runs are serial with `--test-threads=1`, and `dedicated` stays `"0"`.

## Write scopes

Branch `gh31-step-004-draw-arms`, cut from master.

- `crates/mp/renderer/src/render_state/` - `model_blocks.rs`, `bmodel_table.rs`.
- `crates/mp/renderer/src/tr_model/` - `render_models.rs` (`mark_block` and its test), `cached_model_binary.rs` (test-side helper asserts only, if the helper test lives there).
- `crates/mp/renderer/src/` - `tr_mesh.rs`, `tr_ghoul2.rs`, `tr_main.rs`.
- `crates/mp/renderer-gpu/src/` - `pipeline3d.rs`, `frame_exec.rs`.
- `crates/mp/renderer-gpu/src/bin/world_harness.rs`.
- `crates/mp/renderer-gpu/tests/` - `world_golden.rs`, `scene_golden.rs`, `ghoul2_vertex_golden.rs`, the new `entity_golden.rs`, and `goldens/entity_duel1.png` (the new fixture only; `ghoul2_verts_stormtrooper.bin` and every existing PNG are read-only).
- Any caller `cargo check` shows broken by the `EntityWalkHost` dissolution or a parameter deletion, edit-only to pass the new shape.
- `.claude/packets/31/step-004/` for `finished.md`.

Everything else is read-only, including `oracle/`, `hook_install.rs`, `model_block.rs`, `render_assets.rs`, `render_assets_sim.rs`, `tr_cmds.rs`, and `~/Developer/jka/` beyond read-only pk3 reads.

## Disposition

The draft awaits the user audit, and no lane spawns before the approval. After approval and a clean lane-review: merge to master locally. No push, and no pull request.

## Amendments

**2026-08-04 - the draft awaits the user audit.**

**2026-08-04 - ruling A is taken.** The user ruled option A at the packet audit: the new entity image golden lands as commit 4, written as the contract already states. Ruling B stays recorded as deferred on the survey's evidence. The lane go is pending.
