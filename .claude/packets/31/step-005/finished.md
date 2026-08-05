# Finished gh#31 step-005 - the bone matrices

Branch `gh31-step-005-bone-matrices`, cut from master after `git merge master --no-gpg-sign`.

## Assumptions and choices, keyed to their commits

**Commit 1 (the crossing surface, inert).**

- Neither payload type derives anything. `Arc<T>` clones without `T: Clone`, and `RefEntity`'s own `Clone` therefore still derives. The contract asked for no derive, so none was added.
- The builder passes `0` as the `argTime` of `g2api_get_time`. The oracle body ignores that argument (`oracle/codemp/ghoul2/G2_API.cpp:179-188`), and the packet records the same fact, so the builder needs no refdef clock. One comment line at the site carries the cite.
- `have_models` is taken right after the validity check, before the `r_noServerGhoul2` gate and before the setup. This reproduces the old order exactly: the `MOD_BAD` arm read `G2API_HaveWeGhoul2Models` on the live instance list before `r_add_ghoul_surfaces` ran any setup for that entity.
- Two paths cross as an empty-models payload rather than `None`: a suppressed `r_noServerGhoul2` instance (ruling B) and a `g2_setup_model_pointers_v` that returns false. Both preserve the old draw-nothing behavior in the `MOD_MDXM` arm and the old skip-the-null-axis behavior in the `MOD_BAD` arm.
- `ref_entity_from_tr` sets `ghoul2_render: None`. The ABI `trRefEntity_t` carries no crossing, and the walk reads the payload off its own parallel slice.
- `fx/fx_host.rs` needed no edit. It calls `RE_AddMiniRefEntityToScene`, whose signature is unchanged, and the `None` the contract names is passed inside `tr_scene.rs` at the one internal `RE_AddRefEntityToScene` call. This is the contract as written, not a deviation.

**Commit 2 (the migration, one atomic swap).**

- The two dispatcher arms read `r_noServerGhoul2` inside a scoped block that casts `re`, reads `view.common.cvar(re.cvars.r_noServerGhoul2).integer`, and ends. The build then runs with no `re` binding in scope, and the `re` cast for the add comes after it. Both arms carry the order in a comment above the build.
- `r_add_ghoul_surfaces` borrows `slist` straight off the payload instead of cloning it. `bltlist` still clones, because `CRenderSurface::bolt_list` is `&mut [boltInfo_t]`.
- The model-loop skips (`valid`, `GHOUL2_NOMODEL`, `GHOUL2_NORENDER`) moved into the builder, so the render loop walks `payload.models` by ordinal with no skip. One comment line records that the builder already dropped them.
- `decode_ghoul2_surface` indexes `bones[...]` directly rather than through `get`. `eval_render` panicked on an out-of-range bone index too (a `Vec` index after a `debug_assert!`), so the panic behavior is unchanged. The out-of-range guards the packet names sit on the payload lookup and the ordinal lookup, both `?` arms that count a decode failure.
- Three doc paragraphs in `tr_main.rs` named the `engine_view` parameter this step deletes. They were corrected to name `payloads` and the `cvars` snapshot instead. Leaving a doc that names a deleted parameter would be a stale claim my own change created.
- `world_golden.rs`, `scene_golden.rs`, `ghoul2_vertex_golden.rs` and `entity_golden.rs` no longer build an `EngineHostView` for the frame, so the `re_ptr`/`models_ptr`/`sv_ptr` split shrank to the `RendererFrontend` destructure the call still needs. Three now-unused imports dropped from `world_golden.rs`.
- The pin comments in `world_harness.rs`, `ghoul2_vertex_golden.rs` and `entity_golden.rs` now say the setup re-registers at scene-add, ahead of the drain. The drain-before-pin blocks and the `Arc::clone` pin itself are untouched.
- `dev_harness.rs` and `ui_harness.rs` are parameter-deletion fallout only, under the write scope's edit-only clause.
- `boot.rs` is compile fallout only. The `Ghoul2System` local deletes with the chain parameter, and an empty `payloads` vector takes its place. The registry the spike resolves against stays undrained, and the item stays parked.

## Deviations

None.

## Pause triggers hit

None. No fixture moved, no golden moved, the payload carried every field the builder needed, no new `pub` item on `mp_engine_ghoul2` was required, both dispatcher arms satisfied the order contract, and every frame-pinned `Arc::clone` stayed where it was.

## Commits and gate results

1. `c34ec3fc` **feat(gh#31 s005): the Ghoul2 crossing surface, inert**
   - `cargo build --workspace`: green.
   - `cargo test --workspace`: green.
   - `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`: 2 passed, byte-identical.

2. `37cfa349` **feat(gh#31 s005): the bone matrices cross, and Ghoul2 draws render-side**
   - `cargo build --workspace`: green.
   - `cargo test --workspace`: green.
   - `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`: 1 passed, byte-identical. This is the fixture's fifth configuration.
   - `cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`: 1 passed, `entity_duel1.png` byte-identical at `CHANNEL_TOLERANCE = 0`.
   - `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`: 2 passed, byte-identical.
   - `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`: 7 passed.

Every golden run was one foreground command with `--test-threads=1`, and `dedicated` stayed `"0"` in every rig run. `cargo check --workspace --all-targets` reports zero warnings on the final tree. The lockstep referee was not run: no commit touches `mp_game`, the server, or any `jampded` link-set crate.

## Open gaps

- The eager bone pass marks every bone rendered where Raven marks only the referenced ones. `CBoneCache::was_rendered` is the only observer, and its one consumer is the deferred gore chain. One comment line at the builder records it.
- Ruling A's transform-before-cull divergence is live. No fixed-scene golden can see it, because nothing in those scenes is culled. Live play is the remaining gate.
- The `boot.rs` spike still resolves entities against a registry no drain fills. It adds zero entities, so the read is inert, and the item stays parked for the step that gives the spike entities or deletes it.
- `Ghoul2System` is now fully sim-confined, so `FrameExecutor` owns no Ghoul2 state at all. No caller of the deleted `set_ghoul2`/`ghoul2_mut` remains anywhere in the workspace.
