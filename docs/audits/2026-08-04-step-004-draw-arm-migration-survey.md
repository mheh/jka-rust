# Step-004 draw-arm migration survey (gh#31 packet ground)

This record freezes the read-only survey that grounds the gh#31 step-004 packet (`.claude/packets/31/step-004/packet.md`), the draw-arm migration onto the published `PublishedModel` entries. A survey agent ran it against master `54b2b5f7` on 2026-08-04, and no source file changed. `cargo check --workspace` was green at survey time. Rulings 3 and 4 from the step-003 packet Amendments bind this step: the arms read the published copy through view helpers on `PublishedModel`, and entity `model_type` resolves from the published entry with `BModelTable` keeping only its brush-submodel fields.

Ground read: the step-003 packet with its Amendments (`.claude/packets/31/step-003/packet.md`), the absorption landing (`docs/audits/2026-08-04-renderer-frontend-absorption-landing.md`), the step-003 survey sections D and F (`docs/audits/2026-08-04-step-003-draw-arm-survey.md`), the step-002 packet and landing, DEC-65 and DEC-54 (`docs/decisions.md:1526-1534,1411-1417`), and the gh#31 close-out comments. The step-003 survey's sections D and F were re-verified against today's master. They hold in substance, with small line drift in `tr_ghoul2.rs` (`get_model` sits at `:2368` not `:2372`, `render_surfaces` at `:2406` not `:2412`, `g2_setup_model_pointers_v` at `:2285` not `:2289`). Every cite below was re-read today.

## A. The migration inventory

Every read the four consumers make through `models: &RenderModels` today, with its published-copy replacement. The published entry is `PublishedModel` (`crates/mp/renderer/src/render_state/model_blocks.rs:22-33`): `model_type`, `num_lods`, and the three block-plus-offset families `md3`, `mdxm`, `mdxa`. The registry is `assets.models: Arc<ModelBlocks>` (`render_assets.rs:149`), which every arm already reaches through its `assets: &RenderAssets` parameter.

**`r_add_md3_surfaces` and its helpers (`crates/mp/renderer/src/tr_mesh.rs`).** Six reads, five with a direct published replacement:

- `models.get_model(ent.e.hModel)` (`:372`) becomes the published-entry lookup.
- `(*current_model.md3[0]).numFrames` (`:376`) becomes a read through `md3_ptr(0)`.
- `current_model.name` in the bad-frame warning (`:396`) has no published equivalent. See section B.
- `r_compute_lod` (`:404`, body `:210-265`) reads `numLods` (`:219,245,250-261`) and `md3[0]` (`:227`). It is a private fn, so its parameter changes from `&model_t` to the published entry.
- `current_model.md3[lod]` (`:406`) becomes `md3_ptr(lod)`.
- The surface walk (`:432-535`) reads the raw header, which now arrives from the helper. `r_cull_model` (`:410`, body `:281-329`) and `r_compute_fog_num` (`:427`, body `:146-184`) already take `*const md3Header_t` and do not change. The pushed `Md3SurfaceRef` (`:516-531`) is handle plus scalars and does not change.

**The `tr_main.rs` dispatch (`crates/mp/renderer/src/tr_main.rs:1961-2140`).** Three reads:

- `bmodels.get(ent.e.hModel)` (`:1980`) and the `MOD_BRUSH` test (`:1981`): under ruling 4 the brush test becomes `bmodel_index >= 0`, which is equivalent because only `register_bmodel` (`tr_model/render_models.rs:324-349`) sets a non-negative index, and `model_type` leaves `BModelEntry` (`render_state/bmodel_table.rs:14-24`).
- The `match model.model_type` (`:2038`) resolves from `assets.models.get(hModel)`, `MOD_BAD` when the entry is absent. `register_bmodel` never calls `mark_block`, so a brush handle has no published entry and the brush test must run first.
- `host.models` (`:2036`) is deleted. The host gate (`:2027-2035`) narrows to the two ghoul2 legs (`MOD_MDXM`, and the `MOD_BAD` ghoul2-token check at `:2107-2132`), because only those reach `EngineHostView`. The `MOD_MESH` arm and the `MOD_BAD` null-axis fallthrough run ungated, render-side, for the first time.

**`r_add_ghoul_surfaces` (`crates/mp/renderer/src/tr_ghoul2.rs:2254-2415`).** Three block reads and two host reads:

- `models.get_model(inst.model)` (`:2368`) becomes the published-entry lookup, with a `continue` on an absent entry.
- `g2_compute_lod` (`:2369`, body `:668-739`) reads only `numLods` (`:676,720,725-736`). It is `pub`, so its `current_model: &model_t` parameter change is a contract item.
- `render_surfaces` (`:2406`, body `:840-953`) reads `mdxm_view_of(current_model)` (`:849`) and `current_model.index` (`:920`). The view read becomes `mdxm_view()` on the entry, and the index equals the handle the caller already holds (`inst.model`), so the handle is passed instead. The only non-recursive caller is `:2406` (the `:944` site is the recursion).
- `g2_setup_model_pointers_v` (`:2285`) and `g2_construct_render_skeleton` (`:2327`) read through `host: &mut EngineHostView` and the sim-confined `CBoneCache`. DEC-65 ruling 2 (`docs/decisions.md:1532`) places the transform sim-side at scene-add with plain per-entity bone matrices crossing in the frame package. That crossing is not this step, so the ghoul2 legs keep the host parameter and stay gated on it.

**The `pipeline3d.rs` decoders and plumbing (`crates/mp/renderer-gpu/src/pipeline3d.rs`).** Eight sites:

- `draw` (`:1130`) and `collect_stage_items` (`:1427`) carry `models: Option<&RenderModels>`, and the two W2-F8 gates (`:1473`, `:1501`) skip the arms when it is `None`. All four delete: the decoders read `assets.models`.
- `collect_md3_surface` (`:2011`) and `collect_ghoul2_surface` (`:2176`) carry `models: &RenderModels` and pass it down. Both parameters delete.
- `decode_md3_surface` (`:4048-4102`): `get_model` (`:4054`) and `model.md3.get(lod)` (`:4055`) become the entry lookup and `md3_ptr(lod)`, both `None` arms keeping the existing skip-and-count shape.
- `decode_ghoul2_surface` (`:4117-4235`): `get_model` (`:4123`), the `model.mdxm.is_null()` check (`:4127`), and `mdxm_view_of(model)` (`:4130`) become the entry lookup and `mdxm_view()`, whose `None` covers the null check. `g2.bone_caches.get_mut(...).eval_render` (`:4147,:4162`) reads the sim-confined incremental cache and does not migrate. Under ruling 2 the crossed per-entity matrices replace it in a later step, and until then the ghoul2 decode runs only where the walk ran with a host, which is the harness and golden path.

**`frame_exec.rs` (`crates/mp/renderer-gpu/src/frame_exec.rs`).** `backend_models` (`:809`) deletes, the `pipeline3d.draw` argument (`:866`) deletes, and `execute_frame`'s `entity_host: Option<&mut EntityWalkHost>` (`:435`) changes shape with the struct below. `execute_package` keeps passing `None` (`:400`).

**`EntityWalkHost` (`tr_main.rs:175-178`).** The `models` field deletes, which leaves a one-field wrapper around `engine_view`. The survey recommendation is to dissolve the struct into `Option<&mut EngineHostView>` at `execute_frame`, `render_scene`, and `R_RenderView`, because a one-field pass-through wrapper is dead surface once its W2-F8 reason is gone. The construction sites are `tests/ghoul2_vertex_golden.rs:487`, `tests/world_golden.rs:256`, `tests/scene_golden.rs:427`, and `src/bin/world_harness.rs:941`, all already edited by this step.

**Thread placement.** The entity walk and the decoders run on whichever thread calls `execute_frame`: the render thread in the live client (`execute_package`), one thread in the harness and goldens. After the migration every block read goes through the published copy inside `assets`, which is exactly what `FramePackage.assets` carries across the thread split since step-002, so the MD3 arm becomes correct on either side. The ghoul2 legs stay host-gated because the skeleton work is sim-confined until the ruling 2 crossing.

**Other `mdxm_view_of` callers stay.** `g2_process_surface_bolt` (`:482`), `g2_find_surface_bc` (`:615`), the `:801` placeholder read, `process_model_bolt_surfaces` (`:973`), and `g2_construct_used_bone_list` (`:1051`) sit on the sim-side bolt and bone-list path, reached with a `&model_t` from the pool through the host. They are out of this step's scope, and `mdxm_view_of` itself stays for them.

## B. The gaps: reads with no published equivalent

**The model name.** `PublishedModel` carries no name, and the bad-frame warning (`tr_mesh.rs:396`) prints `tr.currentModel->name`, the registered file name. The on-disk `md3Header_t.name` is a different string, so reading it would change the warning text. The smallest faithful fix is a `pub name: String` field on `PublishedModel`, filled by `mark_block` (`render_models.rs:391-413`) from the slot's name. Cost: one `String` clone per registration mark. The packet carries this as a contract line.

**The harness publication drain, a scope implication.** Nothing in `crates/mp/renderer-gpu` calls `RE_EndFrame`, and `RE_EndFrame` (`tr_cmds.rs:356-358`) is the only drain of `RenderModels.blocks_dirty` into `sim.published.models`. The harness and every golden register models through boot and hooks, so after the migration their `assets.models` would stay the empty default and every arm would silently draw nothing. Each `execute_frame` caller that draws entities must therefore mirror the drain pair before it pins the registry: `if let Some(blocks) = host.models.publish_blocks() { host.re.sim.publish_models(blocks); }`, placed before the `Arc::clone` pin so the pinned assets carry the publication. The live client needs nothing: `SCR_UpdateScreen` reaches the drain every frame. The ghoul2 golden's non-empty capture assert (`ghoul2_vertex_golden.rs:511-518`) catches a missing or misordered drain.

**`mdxa`.** No draw arm or decoder reads the mdxa block. The transform path reads it sim-side through the host, and ruling 2 keeps that placement. An `mdxa_view()` helper would have zero callers in this step, so under the no-dead-surface rule (porting-rules §20) it does not land now, even though ruling 3 sketched its shape. It lands with its first consumer.

## C. The view-helper surface

Ruling 3 binds the shape: borrowed view helpers on `PublishedModel`, one SAFETY home for the base-plus-offset casts. The precedent is `mdxm_view_of` (`tr_model/frontend.rs:191-194`), which builds `MdxmView` from one raw pointer via the unsafe `MdxmView::from_block` (`host-interface/src/mdx/mdxm.rs:93`). Grounded in what the bodies consume, the surface is two helpers and one field:

```rust
impl PublishedModel {
    /// The `md3Header_t` one loaded LOD publishes, `None` for an absent slot.
    pub fn md3_ptr(&self, lod: usize) -> Option<*const md3Header_t>;

    /// The DEC-35 view over the published `.glm` block, `None` for a model with no mdxm block.
    pub fn mdxm_view(&self) -> Option<MdxmView<'_>>;
}
```

Both compute `block.base_ptr().add(offset)` from the stored `(Arc<ModelBlock>, usize)` pair, and both carry the one SAFETY comment: the offset was computed at mark time by subtracting the block base from the finished `model_t` pointer, the block is immutable while shared (the `ModelBlock` invariants, `model_block.rs:127-141`), and the pointer or view is valid for as long as the entry's `Arc` lives, which the borrow on `self` guarantees for the frame. `md3_ptr` returns a raw pointer because its consumers (`r_cull_model`, `r_compute_fog_num`, the surface walks) already take one, and their deref sites keep their existing SAFETY comments, now citing the helper's contract. `md3Header_t` is nameable in `model_blocks.rs` (`mp_engine_qcommon::qfiles::md3_header_t`), and `MdxmView` comes from `mp_host_interface`, which the crate already depends on. The `name` field from section B joins the struct. `mdxa_view()` is deferred per section B.

## D. The proof gate

The existing gates do not prove "players on screen". The world goldens draw no entities, the seven scene goldens draw sprites, lines, polys, and saber glow but no `RT_MODEL` (`scene_golden.rs`, grep confirmed), and the ghoul2 golden locks vertex streams, not pixels. After the migration the MD3 arm draws render-side for the first time and no gate watches it. The fork, with a recommendation:

**Option A, a new entity image golden (recommended).** One new test file `tests/entity_golden.rs` on the `world_golden.rs` pattern: boot duel1, register `models/map_objects/bespin/twinpodcc.md3` through `boot::register_model` (`boot.rs:551`, the model `world_harness` already mounts, `world_harness.rs:107`) and init `models/players/stormtrooper/model.glm` through the `init_ghoul2` recipe, add both as `RT_MODEL` entities at fixed origins under the frozen clock (`FROZEN_TIME_MS = 12345`, the zero-radius LOD-0 pin from `ghoul2_vertex_golden.rs:404-418`), render one frame, read back the pixels, and compare one committed PNG `tests/goldens/entity_duel1.png` at `CHANNEL_TOLERANCE = 0`. `#[ignore]`d like the world goldens, blessed with `JKA_GOLDEN_BLESS=1`, run serial with `--ignored --test-threads=1`. Cost: one new test file that reuses two existing recipes, one PNG fixture, one bless run under review. This is the DEC-54 verification shape ("image goldens on fixed scenes") and it covers both migrated arms in one scene, while the existing vertex golden still isolates the ghoul2 stream when the image moves.

**Option B, extend the ghoul2 vertex golden with an image read-back.** Near-zero new code and no second boot, but it has no MD3 coverage, and adding an MD3 to that scene would change a locked fixture's scene. It also couples the vertex gate and the pixel gate in one test, so one failure names two suspects. Cost: small, coverage: half.

**Option C, no new gate.** The byte-identity of the existing suites proves the migration moved nothing, but nothing proves the newly ungated MD3 path draws. Rejected against DEC-54.

Through the migration itself, the byte gates stand as the binding fact requires: the ghoul2 vertex fixture must stay byte-identical at every commit, because every stormtrooper surface carries an empty shader name, both register legs write `shaderIndex = 0`, the 22 sort keys stay tied, and the fixture has matched under three configurations. Any draw-surf order move is a defect signal, never a re-bless candidate.

## E. The per-slot `re` aliasing ruling

The step-003 close-out carried forward the ruling on the seated `re` slot's per-field aliasing discipline, to be taken before the harness grows a second in-frame hook or executes a console handler. Verified against this step's whole surface: step-004 adds no in-frame hook, no new cast of the seated slot, and no console-handler execution. The migration deletes parameters and reads `assets`, the harness drain uses direct field access on `host.models` and `host.re.sim` outside any view, and `hook_install.rs` is untouched, so the two ghoul2 registration hooks (`hook_install.rs:78-108`) remain the only in-frame `re`-slot casts, under the existing per-slot rule (`hook_install.rs:7-11`). The ruling therefore records as deferred again, with this section as the evidence, and it comes due with the first step that adds an in-frame hook or runs a console handler in the rig.

## F. The step boundary

"Players on screen" is the deliverable, and inside gh#31 it means the harness and golden path drawing entities with real shaders, proven by an image golden. In scope for step-004: the two view helpers plus the name field, the arm and decoder migration of section A, the ruling 4 dispatch change, the `EntityWalkHost` field deletion (and recommended dissolution), the harness publication drain, the render-side ungate of the MD3 and null-axis arms, and the entity image golden. One live-client consequence lands free: `execute_package` passes no host, so after the ungate the live client draws MD3 entities (weapons, map objects) from its published blocks, while ghoul2 players stay dark until the matrices cross.

Out of scope, named: the DEC-65 ruling 2 execution (sim-side transform at scene-add, per-entity matrices in the package, the leg that makes ghoul2 render-side), the renderfx census flags, mark shaders, dlight projection (`pipeline3d.rs:1405` TODO), the HUD and 2D set, FX, shadows and gore (existing DEFERRED blocks), `re_get_model_bounds` (`tr_mesh.rs:195`, dead stub), `mdxa_view()`, and the live-client cgame feed work outside gh#31.

## G. The hazard hunt

The step-002 vet caught a use-after-free and the step-003 draft hunt caught an `Arc::make_mut` aliasing bug, so this survey hunted the proposed mechanism for the same classes:

- **Stale pointers out of the helpers.** `md3_ptr` and `mdxm_view` derive from the entry's `Arc`, alive for the frame through the pinned (harness) or package-held (live) `RenderAssets`. Checked every value that outlives the walk: `Md3SurfaceRef` and `G2SurfaceRef` carry handles and scalars, never pointers, and the decoders re-resolve against the same `assets` reference in the same frame. No ref type stores a helper-derived pointer across a frame boundary.
- **The missed or misordered drain.** A lane that forgets the section B drain, or places it after the `Arc::clone` pin, draws nothing. The ghoul2 golden's non-empty and 22-surface asserts fail loudly, and the entity image golden fails on a blank scene. Recorded as a named pause condition for the packet.
- **Mid-frame hook interplay.** The in-frame re-register hook calls `Arc::make_mut(&mut re.sim.published)` (`hook_install.rs:87`), which clones `RenderAssets` and shares the inner `Arc<ModelBlocks>` by refcount. No path mutates a published `ModelBlocks` in place, and `publish_models` (`render_assets_sim.rs:34-36`) replaces the field wholesale, so the frame's reads stay on the pinned generation. A poke replay during the walk splits a block copy-on-write: `model_t` moves to the new block (the step-002 post-replay re-fetch) while the published entry keeps the old one. The two blocks differ only in the poked `i32`, the replay writes the same values, and the draw path reads shader indices through the published view, so no observable divergence exists this step.
- **Dispatch equivalence.** The `bmodel_index >= 0` brush test is equivalent to the old `MOD_BRUSH` match: only `register_bmodel` sets the index, brush handles are never marked into the published registry, and every other handle resolves through the published entry, `MOD_BAD` when absent, which reproduces `BModelTable::get`'s default-row behavior for bad handles (`bmodel_table.rs:95-103`).
- **Behavior divergence on absent entries.** `models.get_model` hands a bad handle slot 0's zeroed default, and the published lookup hands `None`. The arms only reach an entry the same-frame dispatch resolved, and the decoders keep their existing skip-and-count `None` arms, so the divergence is unreachable in a live frame and defined (a skip) where a stale ref could reach it.

## Unverified

- No golden, referee, or GPU test ran during this survey, per its instructions. The byte-identity expectation for the migration commits and the blank-scene failure mode of a missed drain are source-reading predictions, gated in the lane.
- The entity image golden's exact origins and the twinpodcc visibility at a fixed pose were not rendered. The constants are free at lane time inside the frozen-clock pattern, and the bless runs under review.
- Whether any live-client cgame trace submits an `RT_MODEL` whose handle resolves `MOD_MESH` today was not re-measured. DEC-54's census records RT_MODEL as the dominant type, and the ungate consequence rests on that record.
- The `String` name field's per-mark cost was not measured. `mark_block` runs at registration completion, not per frame, so the cost class is registration, matching the existing entry build.
