# Step-003 draw-arm and absorption survey (gh#31 packet ground)

This record freezes the read-only survey that grounds the gh#31 step-003 packet (`.claude/packets/31/step-003/packet.md`). A survey agent ran it against the working tree on 2026-08-04, and no source file changed. `cargo check --workspace` was green at survey time. The step's ruled prerequisite is the wholesale absorption, `UiHost` owns a real `RendererFrontend`, ruled at the gh#35 close (`docs/audits/2026-08-04-ghoul2-golden-null-re-slot.md`, the appended ruling).

Ground read for this survey: gh issues 31, 2, and 35 with comments, DEC-54 and DEC-65 (`docs/decisions.md:1411-1424,1526-1534`), the step-002 packet with its Amendments (`.claude/packets/31/step-002/packet.md`), both step-002 audits (`docs/audits/2026-08-04-model-block-publication-survey.md`, `docs/audits/2026-08-04-model-block-publication-landing.md`), the whole gh#35 record, the gh#35 step-001 packet folder including `finished.md` and `a2-attempt.patch`, and `docs/plans/2026-07-24-client-port/`.

## A. Half 1: the absorption inventory

**The twin fields.** `UiHost` (`crates/mp/renderer-gpu/src/ui_host/state.rs:30-70`) holds 12 exact `RendererFrontend` twins under the DEC-42.3 header: `cvars`, `sim`, `img_state`, `frame`, `world_load`, `scene`, `noise`, `rng`, `font`, `world_effects`, `qs`, `sky_view` (`state.rs:40-55` against `crates/mp/renderer/src/renderer_frontend.rs:68-130`). `models: RenderModels` (`state.rs:39`) stays outside, because the frontend's own doc excludes the model registry (`renderer_frontend.rs:62-63`). The 7 frontend-only fields are `frame_data`, `frame_sink`, `pending_capture`, `pending_world`, `screenshot_last_number`, `screenshot_jpeg_last_number`, `automap` (`renderer_frontend.rs:86-129`).

**The counts, recounted.** 14 `let UiHost {` destructure sites plus 1 struct literal, matching the gh#35 audit: the literal at `boot.rs:186`, destructures at `crates/mp/renderer-gpu/src/ui_host/boot.rs:214,357,487,540,559,600,713`, `tests/world_golden.rs:229`, `tests/scene_golden.rs:259,398`, `tests/ghoul2_vertex_golden.rs:133,448`, `src/bin/world_harness.rs:899,1134`. Dot accesses to the 12 twins on receiver `host.`: 56 occurrences on 47 lines, which reconciles the audit's 47 (a line count) with the earlier survey's 53 (an occurrence count under a slightly different rule). Per field: `sim` 31, `scene` 11, `img_state` 5, `cvars` 4, `frame` 3, `noise` 1, `world_load` 1, the other five 0. Per file (lines): `world_harness.rs` 12, `ghoul2_vertex_golden.rs` 10, `scene_golden.rs` 9, `world_golden.rs` 8, `ui_harness.rs` 4, `boot.rs` 4.

**`boot_renderer` construction order** (`boot.rs:121-261`): `Engine::new`, the two hook installs (`boot.rs:130-131`), the standing gh#35 A1 override (`boot.rs:133-138`, `hooks.RE_RegisterModel = hooks.R_RegisterServerModel`), `RenderModels::default` (`boot.rs:143`), the engine-subset boot block with its own `host_view` (`boot.rs:146-180`), the `UiHost` literal (`boot.rs:186-210`), then the `R_Init` destructure block (`boot.rs:213-253`). The literal's 12 twin seeds equal `RendererFrontend::new()`'s seeds field for field (`renderer_frontend.rs:253-281`), so the absorption replaces the 12 initializers with `re: RendererFrontend::new()` and changes no seed value.

**`host_view` and its callers.** `host_view` (`boot.rs:421-442`) sets `re: SlotRenderer::from_raw(null_mut())` at `boot.rs:437`. It has 16 call sites: 9 inside `boot.rs` (`:150,235,285,385,503,550,574,616,739`) and 7 outside (`world_golden.rs:241`, `scene_golden.rs:273,410`, `ghoul2_vertex_golden.rs:139,460`, `world_harness.rs:911,1138`). How it gains the `re` pointer: a fifth parameter `re: *mut RendererFrontend`, so each of the 16 sites states its seat. The gh#35 a2 attempt shaped the alternative, assigning `view.re` after construction (`.claude/packets/35/step-001/a2-attempt.patch`), which keeps the signature but hides the choice at 15 sites. `with_dc` cannot seat the slot, because it destructures `re`'s fields into disjoint `&mut` borrows for the `HarnessDc`, and it needs no seat: the only code that reaches the `re` slot is ghoul2 registration (`crates/mp/engine/ghoul2/src/api_models.rs:146-162` through `crates/mp/renderer/src/hook_install.rs:78-108`), and the 2D paint path reaches neither hook. `dev_harness.rs` uses neither `UiHost` nor `host_view` and is out of this step entirely.

**The paint path and `frame_data`.** `with_dc` (`boot.rs:356-403`) builds a fresh `FrameData` per call (`boot.rs:397`). `ui_harness` extracts it with `mem::replace` (`src/bin/ui_harness.rs:112`) and hands it to `execute_frame` with `entity_host` `None` (`ui_harness.rs:242-250`). `world_harness` builds one per `record_scene` (`world_harness.rs:709-726`). `RE_ClearScene` appends a `ClearScene` event and empties nothing (`crates/mp/renderer/src/tr_scene.rs:245-248`). The only drain is `RE_EndFrame` (`crates/mp/renderer/src/tr_cmds.rs:336-400`), and nothing in `crates/mp/renderer-gpu` calls it (grep: zero hits). Adopting `re.frame_data` wholesale would therefore accumulate events into every later frame, exactly as the gh#35 audit's section C states. Recommendation: keep per-call construction, and `re.frame_data` stays inert beside `frame_sink` and the two `pending_*` fields.

## B. The hazard hunt (the step-002-vet equivalent)

The step-002 vet exists because a drafted packet once shipped a use-after-free. This survey hunted the naive absorption for the equivalent and found one real defect class.

**The frame-time re-register aliasing.** `G2_SetupModelPointers` re-registers the model on every frame's entity walk (`crates/mp/engine/ghoul2/src/misc.rs:396-404`, faithful to `oracle/codemp/ghoul2/G2_API.cpp:2675-2693`), which the gh#35 lane proved at draw time (the backtrace in `.claude/packets/35/step-001/finished.md`). The client hook `re_register_model_hook` (`hook_install.rs:78-96`) forms `&mut RendererFrontend` from the seated slot and calls `Arc::make_mut(&mut re.sim.published)` unconditionally (`hook_install.rs:87`). Today every entity-walk caller passes `&sim.published` as `execute_frame`'s `assets` argument (`ghoul2_vertex_golden.rs:473`, `world_harness.rs:927`), a shared borrow of that same allocation. With the slot seated and the refcount at 1, `Arc::make_mut` mutates the allocation in place while the executor's `&RenderAssets` is live. That is mutable aliasing, undefined behavior, and it fires on the first steady-state ghoul2 frame.

**The fix is a frame-pinned registry clone.** The caller clones the `Arc` before the split borrow and passes the pinned deref: `Arc::make_mut` then sees a refcount of 2 and clones, the frame keeps reading the old allocation, and the hook's writes land in the next generation. This is `FramePackage`'s own semantics (`crates/mp/renderer/src/render_state/frame_package.rs:41-43`, `tr_cmds.rs:389`) and the copy-on-write cadence `RenderAssetsSim`'s doc describes (`render_assets_sim.rs:13-21`). Cost: one registry deep-clone per frame that re-registers a ghoul2 model, which is the same cadence the live client pays when a registration lands while a package holds the previous generation.

**Site disposition.** The pin is mandatory at the two ghoul2-drawing sites (`ghoul2_vertex_golden.rs:469`, `world_harness.rs:921`). `world_golden.rs` and `scene_golden.rs` draw no ghoul2 entity (gh#35 `finished.md`), so their hooks are unreachable, and they take the pin for uniformity. `load_world_and_render` (`boot.rs:722`) keeps `Arc::make_mut`, because its entity list is empty by construction (`boot.rs:673`) and `R_TerrainInit` needs the `&mut`.

**Checked and clear.** No boot-time pointer capture into a `UiHost` field exists that the absorption would invalidate: `arm_botlib_slot` captures from the `Box`-pinned engine only (`boot.rs:273-287`). The flat-struct split-borrow rationale (`state.rs:26-29`) survives as two-level disjoint field borrows, per the gh#35 audit section C. The raw-pointer-then-reborrow ordering the sites already use for `models_ptr` (`ghoul2_vertex_golden.rs:457-466`) extends to the `re` pointer unchanged.

**Pre-existing sibling class, not this step's.** The register path casts `&mut RenderModels` out of the `rm` slot while `EntityWalkHost` holds `models: &RenderModels` in the same call stack. The per-slot rule at `hook_install.rs:7-11` governs it, and the A1 configuration takes this shape today. The absorption neither widens nor narrows it.

## C. The re-bless consequence

The fixture `crates/mp/renderer-gpu/tests/goldens/ghoul2_verts_stormtrooper.bin` was blessed at `bc856508` under the server registration path, and the A1 override preserves that configuration (`boot.rs:133-138`). The absorption removes the override, so ghoul2 registration runs Raven's client path and every surface gets a real shader index where the server path forces 0 (`server_load.rs:508`, per the gh#35 audit). Real indices untie the 22 tied sort keys, so the draw-surf order moves and the fixture re-blesses.

The re-bless validity check is content, not order: the 22-surface digest multiset must equal the recorded list in `.claude/packets/35/step-001/finished.md`, whose multiset digest is `f3551f6abb73953d71b6d293306a8889914e34c6a467cb11d44fc632fbef7264`. Geometry is expected unchanged: the stormtrooper reports `numBones = 53`, so the 72-bone `_humanoid` remap gate (`crates/mp/renderer/src/tr_ghoul2.rs:2044-2046`) does not fire (gh#35 audit claim 8). The two world goldens and the scene golden stay byte-identical: none draws a ghoul2 entity, and their model registration goes through direct `RE_RegisterModel` calls, never the hook.

Two rig traps bind the lane. All goldens need `--test-threads=1`, or two engine boots crash in the pk3 inflate path before any comparison. A nonzero `dedicated` cvar stubs every image through `tr_image.rs:2499` and also masks every `dedicated || g2_should_register_server` guard, so it must never be used as a shortcut.

## D. Half 2: the draw-arm read inventory

**`r_add_md3_surfaces`** (`crates/mp/renderer/src/tr_mesh.rs:352-537`) reads from `models: &RenderModels`:

- `models.get_model(ent.e.hModel)` (`tr_mesh.rs:372`), then `(*md3[0]).numFrames` (`:376`).
- `current_model.name` in the bad-frame warning only (`:396`).
- `r_compute_lod` (`:404`, body `:210-240`): `numLods`, plus `read_md3_frame(md3[0], frame)` for the projected bounds.
- `md3[lod]` (`:406`), the cull-box frame reads in `r_cull_model` (`:410`), and the fog-volume frame read in `r_compute_fog_num` (`:427`, body `:146-180`).
- The surface walk over `ofsSurfaces`, stepping by each surface's `ofsEnd`, reading `numShaders`, the surface name, and the `md3Shader_t` array (`:432-535`).
- The push stores a `Copy` `Md3SurfaceRef` of handle plus scalars (`:516-531`), no pointer.

Every one of these is a shared read of frozen file bytes reachable from `PublishedModel.md3[lod]` as block plus offset, with `num_lods` and `model_type` already on the entry (`crates/mp/renderer/src/render_state/model_blocks.rs:22-33`). One gap: `PublishedModel` carries no model name, and the warning at `:396` prints one. The `md3Header_t`'s own on-disk name field or a new entry field serves it.

**The `tr_main.rs:2017-2119` dispatch.** The arm gates on `entity_host: Option<&mut EntityWalkHost>` (`tr_main.rs:2027-2035`), and the live render thread passes `None` (`crates/mp/renderer-gpu/src/frame_exec.rs:394-406`), so no MD3 or ghoul2 entity draws render-side today. The arm switches on `model.model_type` resolved from `BModelTable` (`tr_main.rs:1980`), which is built at world load and travels with the world generation (`crates/mp/renderer/src/render_state/bmodel_table.rs`). Fork found: a model registered after the world generation resolves `MOD_BAD` in that table. A ghoul2 player still draws through the `MOD_BAD` arm's token check (`tr_main.rs:2094-2129`), and a post-load MD3 registration would silently miss the `MOD_MESH` arm. `PublishedModel.model_type` republishes at every `RE_EndFrame` drain (`tr_cmds.rs:356-360`), so the published entry is the fresh source for this dispatch. The resolution belongs to the draw-arm step.

**`r_add_ghoul_surfaces`** (`crates/mp/renderer/src/tr_ghoul2.rs:2254-2420`) reads:

- `g2_setup_model_pointers_v(g2, host, ...)` (`:2289`): the `EngineHostView` reads, cvar, `shader_hash_table_exists`, model registration through the `re` slot, and the mdx views through the `rm` slot. Sim-side only, and the reason the absorption is this step's prerequisite.
- `g2_construct_render_skeleton(g2, host, ...)` (`:2327`): the bone transform. DEC-65 ruling 2 places it sim-side at scene-add time, with plain per-entity matrices crossing in the frame package. It cannot migrate onto the published blocks.
- `models.get_model(inst.model)` (`:2372`) feeding `g2_compute_lod`, which reads `numLods` only (`:668-739`).
- `render_surfaces(current_model, ...)` (`:2412`, body `:840-953`): `mdxm_view_of(model.mdxm)` (`:849`, view built at `crates/mp/renderer/src/tr_model/frontend.rs:191-195`) and `current_model.index` for the `G2SurfaceRef` (`:920`).

The two block reads (`num_lods`, the mdxm view, and `index`, which equals the handle the ref already carries) map onto the published entry. The host and skeleton reads stay sim-side.

**The backend decoders** (`crates/mp/renderer-gpu/src/pipeline3d.rs`):

- `decode_md3_surface` (`:4048-4102`): `models.get_model`, the `md3[lod]` header, the surface walk, and the keyframe lerp. Migrates fully onto the published entry.
- `decode_ghoul2_surface` (`:4117-4235`): `models.get_model`, the `model.mdxm` null check, the mdxm view, and `g2.bone_caches.get_mut(...).eval_render` (`:4147,4162`). The block reads migrate. `eval_render` reads the sim-confined incremental `CBoneCache`, so under DEC-65 ruling 2 the crossed per-entity matrices replace it, and until that crossing lands the ghoul2 decode stays sim-thread-only.

**The thread split.** `jamp-sim` spawns at `crates/mp/client-app/src/main.rs:69-72` and `jamp-render` at `crates/mp/client-app/src/pump.rs:185-188` (per the step-002 packet's grounded facts), with `FramePackage` crossing over a bounded channel. The package's `assets: Arc<RenderAssets>` (`frame_package.rs:43`) already carries `models: Arc<ModelBlocks>` since step-002, so the render thread holds the published blocks today and nothing reads them. The live client seats `engine.re` on the sim thread (`crates/mp/client-app/src/sim.rs:85`).

## E. What "players on screen" minimally requires

The chain, from the map and DEC-65: first the absorption, so entity surfaces carry real shader indices and the rig runs Raven's client path. Second the draw-arm and decoder migration onto `PublishedModel`, which makes the MD3 path fully render-side capable and resolves the dispatch `model_type` fork. Third the DEC-65 ruling 2 execution, the sim-side transform at scene-add plus per-entity matrices in the package, which makes the ghoul2 decode render-side. The client island feeding real `RT_MODEL` entities is the separate client-hosting track, outside gh#31. gh#31's own gate is the DEC-54 image goldens, so inside this ticket "players on screen" means the harness and golden path drawing entities with real shaders: the absorption now, the block migration as step-004, and the ruling-2 matrix crossing as the leg after that.

## F. The consumption shape (ground for the user's ruling)

Two shapes fit the migration: direct `(Arc<ModelBlock>, usize)` reads at every site, or borrowed view helpers on `PublishedModel`. The bodies argue for helpers. The mdxm path already reads through a view built from one raw pointer (`MdxmView::from_block`, `frontend.rs:191-195`), so `PublishedModel::mdxm_view()` is a three-line helper over base pointer plus offset. The MD3 path repeats unsafe base-plus-offset header casts at three or more sites (the `tr_mesh.rs` walk, `read_md3_frame`, `decode_md3_surface`), and one `md3_ptr(lod)` helper gives the offset math and its SAFETY argument one home. The arms already take `assets: &RenderAssets`, and the published registry rides at `assets.models`, so the migration deletes the `models: &RenderModels` parameters rather than replacing them.

## G. The split decision inputs

The absorption alone touches 8 files, 15 construction and destructure sites, 56 dot accesses, 16 `host_view` call sites, one soundness mechanism (the frame-pinned clone), and one fixture re-bless. That is a full lane on the step-002 scale. The draw-arm migration carries two unruled design points (the consumption shape, the dispatch `model_type` source) and its own gate story. The deciding fact is re-bless attribution: the gh#35 audit's section B correction exists because an experiment changed two things at once, and a combined packet would re-bless the fixture under the absorption and the arm migration together, so an unexpected order or content move could not be pinned on either. The packet therefore scopes the absorption alone as step-003, and the draw arms go to step-004 against the re-blessed fixture.

## Unverified

- No golden, referee, or GPU test ran during this survey, per its instructions. The byte-identity expectations for the mechanical commit and the multiset expectation for the re-bless are source-reading predictions, gated in the lane.
- The per-surface content stability under the client path rests on gh#35 audit claim 8, which diffed the `.glm` walk. The `.gla` load-leg diff was that audit's own unverified item and stays unverified here.
- The `a2-attempt.patch` shapes were compile-checked by the gh#35 lane, and this survey read them without re-running the check.
- Whether any live-client path outside the harness ever calls `execute_frame` with a seated `EntityWalkHost` was checked by grep only: the one `execute_package` call site passes `None`.
- The live client's cgame feed of `RT_MODEL` entities is outside gh#31 and was not surveyed.
