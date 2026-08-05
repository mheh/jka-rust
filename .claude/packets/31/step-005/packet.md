# Packet gh#31 step-005 - the bone matrices

## Scope

This step executes DEC-65 ruling 2: the Ghoul2 skeleton transform runs sim-side at scene-add time, plain per-entity bone matrices cross in the frame package, and the ghoul2 draw legs run render-side with no host gate. This is the step that puts players on the live-client draw path. It delivers five things: the crossing payload types and their sim-side builder, the scene-add crossing through `RE_AddRefEntityToScene` and every add site including the two live dispatcher arms, the render-side migration of `r_add_ghoul_surfaces` and `decode_ghoul2_surface` onto the payload, the deletion of the `entity_host` and `g2: &mut Ghoul2System` plumbing through the whole walk chain and the executor, and the doc lines that record the new shape. After this step `Ghoul2System` is fully sim-confined: the executor's own copy, `set_ghoul2`, and `ghoul2_mut` delete, and `execute_frame` takes no engine host at all.

The mechanics, grounded in the survey. Today `r_add_ghoul_surfaces` (`crates/mp/renderer/src/tr_ghoul2.rs:2265`) calls `g2_setup_model_pointers_v` and `g2_construct_render_skeleton` through a live `EngineHostView`, and the backend `decode_ghoul2_surface` (`crates/mp/renderer-gpu/src/pipeline3d.rs:4102`) evaluates bones lazily through `g2.bone_caches.get_mut(...).eval_render(...)`. Both reads move sim-side: the builder runs the setup, the transform, and an eager `eval_render` pass over every bone in the cache, and the flat `mdxaBone_t` tables cross on the scene entity. Raven runs the per-bone evaluation at backend draw time (`oracle/codemp/renderer/tr_ghoul2.cpp:4060-4451`, `RB_SurfaceGhoul`), and DEC-65 records the move to scene-add as the one accepted timing deviation, with the image golden as the parity gate. `g2api_get_time` never reads its time argument (`crates/mp/engine/ghoul2/src/api_collision.rs:421-429`, the oracle body ignores `argTime`), so the builder needs no refdef and the frozen-clock goldens see the same transform time as today.

The step does not touch gore, shadows, the renderfx census flags, mark shaders, dlight projection, the HUD set, FX beyond the one `None` argument in `fx_host.rs`, the deferred `RE_AddRefEntityToScene` ghoul2-model diagnostic (`tr_scene.rs:400-408`), or any `mp_engine_ghoul2` file. The eager evaluation marks every bone rendered where Raven marks only the referenced ones, which is observable only through `CBoneCache::was_rendered`, whose sole consumer is the deferred gore chain. That divergence gets one comment line at the builder and nothing else.

**Two radar items from the step-004 close-out, dispositioned here.**

- **The `boot.rs` spike (parked, with reason).** `load_world_and_render` (`crates/mp/renderer-gpu/src/ui_host/boot.rs:595`) resolves entities against a registry no drain fills, and it adds zero entities, so the read is inert. This step edits the spike only as compile fallout: its `Some(&mut engine_view)` and `&mut g2_system` arguments to `R_RenderView` delete with the chain parameters. The registry stays undrained and the item stays parked, because a drain in a function that renders no entity gates nothing and adds an untested branch. The fix belongs to the step that gives the spike entities, or deletes the spike.
- **The per-slot `re` aliasing ruling (the in-frame class resolves by removal, the ruling stays parked).** The two ghoul2 registration hooks were the only in-frame `re`-slot casts, fired through `execute_frame`'s `entity_host` when `g2_setup_model_pointers_v` re-registered mid-frame. This step moves that work to scene-add and deletes the `entity_host` parameter, so zero in-frame `re`-slot casts remain and the deferred ruling has nothing due. The new sim-side casts at the dispatcher arms are the pre-existing per-slot class (`g2_from_view` is an established helper, `crates/mp/engine/server/src/sv_ccmds.rs:916` and `crates/mp/engine/client/src/cl_cgame.rs:715`), under the order contract below: the payload build completes before any `re`-slot borrow is taken, because the build can reach `re_register_model_hook`, which calls `Arc::make_mut(&mut re.sim.published)` (`crates/mp/renderer/src/hook_install.rs:78-96`). A build that runs while a `re` borrow is live is an aliasing violation, and the order contract is what prevents it.

**Binding facts carried forward from step-004.** The ghoul2 vertex fixture `ghoul2_verts_stormtrooper.bin` has matched byte for byte under four configurations and is the byte-identical tripwire through this migration too, never a re-bless candidate. `eval_render` is deterministic per bone, so the eager pass must reproduce the lazy values exactly, and the fixture proves it. The frame-pinned registry clone at the `execute_frame` call sites is a soundness mechanism and must not be disturbed, though its comment text updates where it names the now-moved in-frame register path. The committed `entity_duel1.png` is the ready-made proof gate: the stormtrooper in it must draw identically through the new crossing, at `CHANNEL_TOLERANCE = 0`, with no re-bless.

## Rulings, taken by the user 2026-08-05

Both rulings are settled and bind this packet. No open question remains, and the packet is ready for the lane on the user's go.

**Ruling A, the transform-before-cull deviation (settled: accept).** Raven culls the entity, then transforms (`oracle/codemp/renderer/tr_ghoul2.cpp:3383-3538`: `R_GCullModel` returns before the transform loop), and the render-side `r_drawentities` and portal checks also run before any ghoul2 work. At scene-add the view does not exist yet, so the builder transforms every added ghoul2 entity, including ones the render-side cull or `r_drawentities` would skip. The observable surface is the bone caches' smoothing history (`mLastTouch`/`mLastLerp` advance for entities Raven would have left stale) plus wasted work, and no fixed-scene golden can see it because nothing in those scenes is culled. The user accepted this as the direct consequence of DEC-65's scene-add ruling. The builder records the divergence in two comment lines, and the entity golden plus live play gate it.

**Ruling B, the `r_noServerGhoul2` check placement (settled: the builder).** Raven checks the cvar before the transform (`tr_ghoul2.cpp` head of `R_AddGhoulSurfaces`, ported at `tr_ghoul2.rs:2291`). The user ruled that the builder keeps that order: the check moves sim-side into the builder, reading the value its caller passes from the live snapshot, and a suppressed instance crosses as an empty payload so the dispatch arms keep their draw-nothing behavior.

## Surface contract

**Two new payload types, one per file, under `crates/mp/renderer/src/render_state/`.** New constructs with no Raven counterpart, the DEC-65 ruling 2 crossing carrier. All fields are POD or owned, so both types are `Send + Sync` without unsafe.

```rust
// ghoul2_model_render.rs
/// One render-visible model of a Ghoul2 entity, snapshotted sim-side at scene-add.
pub struct Ghoul2ModelRender {
    /// The instance's registered model handle, which the walk resolves against the published registry.
    pub model: qhandle_t,
    /// `mCustomSkin` on the instance.
    pub custom_skin: qhandle_t,
    /// `mSkin` on the instance.
    pub skin: qhandle_t,
    /// `mLodBias` on the instance.
    pub lod_bias: i32,
    /// `mSurfaceRoot` on the instance.
    pub surface_root: i32,
    /// The instance's surface-override list, cloned at scene-add.
    pub slist: Vec<surfaceInfo_t>,
    /// The instance's bolt list, cloned at scene-add.
    pub bltlist: Vec<boltInfo_t>,
    /// The composed render matrix per bone, indexed by the global bone index `surface.bone_ref` yields.
    /// Empty when the instance has no built bone cache, and the walk then drops the model's surfaces.
    pub bones: Vec<mdxaBone_t>,
}

// ghoul2_render_payload.rs
/// The per-entity Ghoul2 crossing of DEC-65 ruling 2: everything the render thread reads for one entity's skinned models.
pub struct Ghoul2RenderPayload {
    /// The `G2API_HaveWeGhoul2Models` answer at scene-add, which the `MOD_BAD` arm reads in place of the live instance list.
    pub have_models: bool,
    /// The render-visible models in `G2_Sort_Models` order, empty when `r_noServerGhoul2` suppressed the transform.
    pub models: Vec<Ghoul2ModelRender>,
}
```

**The builder, in `tr_ghoul2.rs`.** It is the sim-side front half of today's `r_add_ghoul_surfaces`: decode the token, `is_valid`, the `no_server_ghoul2` early-out (ruling B), `g2_setup_model_pointers_v`, `g2api_get_time`, `g2_construct_render_skeleton`, then per model in list order the valid/`GHOUL2_NOMODEL`/`GHOUL2_NORENDER` skips, the field snapshot, and the eager bone pass `(0..cache.final_bones.len()).map(|i| cache.eval_render(i as i32))` through `g2.bone_caches.get_mut(inst.bone_cache)`. It returns `None` when the token is null or the instance is invalid, so a stale replay token degrades to the `MOD_BAD` null-axis arm exactly as an absent instance does today.

```rust
/// Builds the DEC-65 ruling 2 crossing for one scene entity, or `None` when the entity carries no live Ghoul2 instance.
/// The caller must finish this call before it takes any `re`-slot borrow, because the setup path can re-register a model through the `re` hooks.
pub fn build_ghoul2_render_payload(
    g2: &mut Ghoul2System,
    host: &mut EngineHostView,
    ent: &refEntity_t,
    no_server_ghoul2: i32,
) -> Option<Arc<Ghoul2RenderPayload>>;
```

**`RefEntity` (`render_state/placeholders.rs`) gains one field**, and its `ghoul2` field doc drops the sentence about the render side threading a `&mut Ghoul2System`:

```rust
/// The DEC-65 ruling 2 crossing, built sim-side at scene-add and read by the render-side walk and decoder.
/// The `Arc` keeps the scene-list clone cheap.
pub ghoul2_render: Option<Arc<Ghoul2RenderPayload>>,
```

**`RE_AddRefEntityToScene` (`tr_scene.rs:318`) gains one parameter**, `ghoul2_render: Option<Arc<Ghoul2RenderPayload>>`, stored onto the `RefEntity` it pushes. `RE_AddMiniRefEntityToScene` passes `None` (the mini entity carries no ghoul2). Every caller updates: `cl_cgame.rs:2050` and `cl_ui.rs:1316` build the payload per the order contract below, `fx/fx_host.rs:308` passes `None` through the mini path, the scene-golden helper and the world goldens pass `None`, `world_harness.rs` builds at its ghoul2 site (`:829`) and passes `None` at the brush and MD3 sites, and the two model goldens build at their ghoul2 sites.

**The dispatcher order contract (`cl_cgame.rs` `CG_R_ADDREFENTITYTOSCENE`, `cl_ui.rs` `UI_R_ADDREFENTITYTOSCENE`).** The arm runs in this order: (1) cast `g2` with the existing `g2_from_view`, (2) read `r_noServerGhoul2` and call `build_ghoul2_render_payload(g2, view, ent, ...)` with no `re` borrow live, (3) cast `re` with `re_from_view`, (4) call `RE_AddRefEntityToScene(&mut re.frame_data, &re.sim.published, &mut re.scene, ent, payload)`. Step 2 may re-register a model and `Arc::make_mut` the published registry through the `re` hooks, which is sound only because step 3 has not run yet. The SAFETY comments at both arms state this order.

**`r_add_ghoul_surfaces` (`tr_ghoul2.rs:2265`) loses `handle`, `host`, and `g2`, and gains `payload: &Ghoul2RenderPayload`.** The token decode, validity check, cvar check, setup, time, and transform calls delete from the body, because the builder ran them. The model loop iterates `payload.models` by ordinal, the skin and shader resolution reads the payload fields against `assets` unchanged, the published-entry resolve and `g2_compute_lod` are unchanged, and an empty `bones` table maps to the dropped-surface arm the missing bone cache maps to today. The cull, `R_SetupEntityLighting`, and fog reads are unchanged.

**`CRenderSurface` (`tr_ghoul2.rs:224`)** retypes `bone_cache: Option<BoneCacheId>` to `model_ordinal: Option<u32>`, the index into the entity's `payload.models`, `None` when the model's `bones` table is empty. `render_surfaces` (`tr_ghoul2.rs:846`) keeps its signature and pushes the ordinal instead of the cache id. No other `CRenderSurface` field moves, and `process_model_bolt_surfaces`, `mdxm_view_of`, and the sim-side bolt and bone-list paths are untouched.

**`G2SurfaceRef` (`tr_main.rs:220`)** stays `Copy` and swaps its `bone_cache: BoneCacheId` field for `model_ordinal: u32`. The entity number needs no field, because `R_DecomposeSort` already yields it at the decode site.

**The walk chain swaps two parameters for one.** `R_AddEntitySurfaces`, `R_GenerateDrawSurfs`, `R_MirrorViewBySurface`, `R_SortDrawSurfs`, and `R_RenderView` (`tr_main.rs`) each delete `entity_host: Option<&mut EngineHostView>` and `g2: &mut Ghoul2System` and gain `payloads: &[Option<Arc<Ghoul2RenderPayload>>]`, parallel to `entities` under the same scene-window rule. Step-004's close-out flagged this exact chain as under-enumerated, so all five are named here and no sixth exists in `tr_main.rs`. The `MOD_MDXM` arm drops its host gate and warning and calls `r_add_ghoul_surfaces` when `payloads[n]` is `Some`. The `MOD_BAD` arm replaces `g2api_have_we_ghoul2_models` with `payloads[n].as_ref().is_some_and(|p| p.have_models)`, keeping the skip-the-null-axis behavior for an instance whose models all decline to render. The `//TODO: Port R_AddEntitySurfaces Ghoul2 arms render-side` marker (`tr_main.rs:2000`) deletes, because the subject completes. `WalkWarnings.entity_models` (`render_state/walk_warnings.rs:38`) deletes with the warning.

**The executor sheds its Ghoul2 state (`frame_exec.rs`).** The `ghoul2: Ghoul2System` field (`:217`), `set_ghoul2` (`:333`), and `ghoul2_mut` (`:339`) delete. `execute_frame` (`:425`) loses `entity_host: Option<&mut EngineHostView>` entirely, and `render_world` (`:674`) loses the same parameter, builds `let payloads: Vec<Option<Arc<Ghoul2RenderPayload>>>` from the same scene window that builds `entities` (`:729`), and threads it into `R_RenderView` and `pipeline3d.draw`. `execute_package` is unchanged in signature, and its doc line "The Ghoul2 entity arms stay dark" (`:367`) rewrites to record that both entity arms draw from the package. The module comment at `:190-191` drops its engine-host sentence.

**The decoder migrates (`pipeline3d.rs`).** `draw` (`:1129`), `collect_stage_items` (`:1424`), and `collect_ghoul2_surface` (`:2152`) swap `g2: &mut Ghoul2System` for `payloads: &[Option<Arc<Ghoul2RenderPayload>>]`. `decode_ghoul2_surface` (`:4102`) takes the payload slice plus the already-computed `entity_num`, resolves `payloads[entity_num].models[g2_ref.model_ordinal]`, and replaces every `cache.eval_render(surface.bone_ref(idx))` with `bones[surface.bone_ref(idx) as usize]`, keeping the weight arithmetic byte for byte. Its `None` arms keep the skip-and-count behavior, with an out-of-range ordinal or missing payload counting as a decode failure.

**The goldens and the harness re-spell.** `ghoul2_vertex_golden.rs` and `entity_golden.rs` build the payload at their add sites with their local `Ghoul2System` and the same host view they already construct for `init_ghoul2`, delete their `executor.set_ghoul2(g2)` lines, and drop the `Some(&mut engine_view)` argument from `execute_frame`. The drain-before-pin blocks stay exactly where they are, and their pin comments update where they name the in-frame register path, because the setup now re-registers at scene-add, before the drain runs. `world_golden.rs` and `scene_golden.rs` change only for the deleted parameter. `world_harness.rs` builds the payload at its ghoul2 entity site and drops its `set_ghoul2` handoff (`:1007`).

Anything not on this list is out of scope, and the agent must not add it. No new third-party crate. No `mp_engine_ghoul2` change: `eval_render`, `final_bones`, and the arena accessors are already `pub` and suffice. No `RE_*` trap arm beyond the two scene-add arms, no `FrameEvent` variant, no cvar, no `#[repr]` change, no `hook_install.rs` change, no fixture change of any kind, and no change to `FramePackage`, `ModelBlock`, `ModelBlocks`, `PublishedModel`, `RenderAssets`, or `RE_EndFrame`.

## Pause triggers, named for this step

- The ghoul2 vertex golden is not byte-identical at any commit, in order, content, or count. STOP: the eager bone pass or the payload move changed behavior it must not change, and the fixture is not a re-bless candidate. This would be its fifth configuration.
- The entity golden moves by any pixel. STOP: the crossing must reproduce the committed image exactly, and `entity_duel1.png` is not a re-bless candidate this step.
- A world or scene golden moves. STOP: no ghoul2 entity draws in those scenes, so the migration touched a shared path.
- The builder turns out to need a field this contract's payload does not carry, or a new `pub` item on `mp_engine_ghoul2`. STOP: the crossing surface is this packet's contract and widening it is a ruling.
- A dispatcher arm cannot satisfy the order contract, or any caller needs a live `re` borrow across the payload build. STOP: that is the aliasing territory of the parked per-slot ruling.
- The frame-pinned `Arc::clone` at any `execute_frame` caller appears movable or deletable. STOP: the pin is a binding soundness mechanism from step-004.
- Verification is `cargo build` / `cargo check` plus the golden suites, never rust-analyzer, which is stale in this workspace.
- `dedicated` stays `"0"` in every rig run. A nonzero value stubs images and masks the register-path guards.

## Commit bundle

1. **The crossing surface, inert.** The two payload types, `build_ghoul2_render_payload`, the `RefEntity` field, the `RE_AddRefEntityToScene` parameter, and every caller passing `None`. No builder call is wired and the render path is untouched, so behavior is unchanged. Gates: `cargo build --workspace`, `cargo test --workspace`, both world goldens byte-identical (`cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`).
2. **The migration, one atomic swap.** The builder calls at every ghoul2 add site including both dispatcher arms under the order contract, the walk-chain parameter swap across all five functions, the `MOD_MDXM`/`MOD_BAD` arm rewrites and gate deletion, the `CRenderSurface` and `G2SurfaceRef` retypes, the decoder migration, the executor state deletion, the `execute_frame` parameter deletion with every caller re-spelled, the `boot.rs` fallout, the `WalkWarnings` field deletion, and the doc lines. The step-004 lesson stands: the old path's parameters are the exact state the new path replaces, so no smaller compiling split exists, and the lane may split only if it finds one that compiles and keeps every gate green at the boundary. Gates: `cargo build --workspace`, `cargo test --workspace`, the ghoul2 golden byte-identical (`cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`), the entity golden byte-identical (`cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`), both world goldens byte-identical (`cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`), the scene goldens green (`cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`, no `--ignored`).
3. **The finished file**, per the packet skill: assumptions keyed to commits, deviations or the word "none", the commit list with gate results, and open gaps.

Every commit uses `--no-gpg-sign`, a heading subject, an STE body, and no trailer of any kind. All golden runs are serial with `--test-threads=1`, each as one foreground command with a long timeout, and `dedicated` stays `"0"`. The lockstep referee is not required: no commit touches `mp_game`, the server, or any `jampded` link-set crate.

## Write scopes

Branch `gh31-step-005-bone-matrices`, cut from master.

- `crates/mp/renderer/src/render_state/` - new `ghoul2_model_render.rs` and `ghoul2_render_payload.rs`, plus `placeholders.rs` and `walk_warnings.rs`.
- `crates/mp/renderer/src/` - `tr_scene.rs`, `tr_ghoul2.rs`, `tr_main.rs`.
- `crates/mp/renderer-gpu/src/` - `frame_exec.rs`, `pipeline3d.rs`, `ui_host/boot.rs` (compile fallout only).
- `crates/mp/renderer-gpu/src/bin/world_harness.rs`.
- `crates/mp/renderer-gpu/tests/` - `ghoul2_vertex_golden.rs`, `entity_golden.rs`, `world_golden.rs`, `scene_golden.rs`. Every fixture under `tests/goldens/` and `ghoul2_verts_stormtrooper.bin` is read-only.
- `crates/mp/engine/client/src/` - `cl_cgame.rs`, `cl_ui.rs`, `fx/fx_host.rs`.
- Any caller `cargo check` shows broken by a parameter deletion or the new `RE_AddRefEntityToScene` arity, edit-only to pass the new shape.
- `.claude/packets/31/step-005/` for `finished.md`.

Everything else is read-only, including `oracle/`, every `crates/mp/engine/ghoul2/` file, `hook_install.rs`, `model_blocks.rs`, `frame_package.rs`, `tr_cmds.rs`, and `~/Developer/jka/` beyond read-only pk3 reads.

## Disposition

Both rulings are settled and the packet is ready for the lane, which spawns only on the user's explicit go. After a clean lane-review: merge to master locally. No push, and no pull request.

## Amendments

**2026-08-05 - the draft awaits the user audit.**

**2026-08-05 - rulings A and B are taken.** The user ruled both open questions on the recommendations: ruling A accepts the transform-before-cull deviation as the direct consequence of DEC-65's scene-add ruling, with two comment lines at the builder recording the divergence, and ruling B puts the `r_noServerGhoul2` check in the builder, preserving Raven's check-before-transform order, with a suppressed instance crossing as an empty payload. No open question remains, and the packet is ready for the lane. The lane go is pending.

**2026-08-05 - packet-skill audit folded at spawn time.** The audit adds the world-golden gate to commit 1, because the packet skill requires that gate on every renderer-touching commit. No other change.

**2026-08-05 - lane-review closed with dispositioned findings.** The clerk walked all 21 files and reproduced every gate. Findings and dispositions:

- The two empty-payload paths (a suppressed `r_noServerGhoul2` instance, a failed model-pointer setup) now run the render-side cull, `R_SetupEntityLighting`, and the lighting write-back, where Raven returned first. Accepted: this is the direct consequence of ruling B's empty-payload crossing, the entity draws nothing either way, and no gate exercises the nonzero cvar path.
- The `MOD_MDXM` guard reads the payload where it read the token, so an invalid instance skips the branch instead of running a no-op lighting copy. Accepted: the output is identical.
- `render_state/mod.rs` was edited outside the enumerated write scope. Accepted: the two contracted new files need module declarations, so this is a packet enumeration gap.
- The two feat commit messages end on a `Gates:` line that git parses as a trailer. Accepted for these commits. Future packets write gate results as plain sentences, so no line parses as a trailer.
- `RenderCvarSnapshot::no_server_ghoul2` keeps one reader in the tree (`world_harness.rs`). Noted, no action: field removal is out of this packet's scope.
- House-style findings on added lines (one semicolon, the word "rides", six column-wrapped comment blocks, one mechanics-narrating line) close in one style commit on the branch. The "seam" flag is discarded: seam is repo vocabulary.
