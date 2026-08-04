# Finished: gh#31 step-002 - DEC-65 ruling 1, model-block publication

Branch `gh31-step-002-block-publication`, four commits, held for lane-review. No merge, no push, no pull request.

## Commits and gate results

| # | Commit | Subject | Gates |
| - | ------ | ------- | ----- |
| 1 | `06aef52b` | `refactor(gh#31 s002): ModelBlock owns the cached model bytes` | `cargo build --workspace` green. `cargo test --workspace` green, zero failures. World goldens byte-identical, both fixtures. |
| 2 | `155fa02f` | `feat(gh#31 s002): the shader poke becomes copy-on-write` | `cargo build --workspace` green. `cargo test --workspace` green, zero failures. |
| 3 | `2a201deb` | `feat(gh#31 s002): publish the model blocks into RenderAssets` | `cargo build --workspace` green. `cargo test --workspace` green, zero failures. World goldens byte-identical, both fixtures. |
| 4 | `af9ee9cb` | `docs(gh#31 s002): correct the never-Arc-published claims` | `cargo build --workspace` green. `cargo test --workspace` green, zero failures. |

Every commit used `--no-gpg-sign` and carries no trailer.

### The world-golden invocation

The brief names `cargo test -p mp_renderer_gpu --test world_golden -- --ignored`. That form aborts before any pixel comparison: both tests boot an engine in parallel threads and crash in the pk3 inflate path (`unzip.rs:40`, subtract overflow). The test file's own module doc already records the reason and the fix: "Serial only: two engine boots in parallel threads crash in the GPU init." The gate ran as `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`, and `golden_world_duel1` plus `golden_world_ffa2` both pass byte-identical against the committed PNGs. This is a pre-existing property of the rig, not a change this lane made.

## Assumptions and choices, keyed to their commits

**Commit 1. The sidecars sit behind their own `Arc` inside `ModelBlock`.** The packet spells the private fields `parsed_mdxm: Option<MdxmParsed>` and `parsed_mdxa: Option<MdxaParsed>`. `Arc::make_mut` needs `ModelBlock: Clone`, and neither `MdxmParsed` nor `MdxaParsed` implements `Clone` today. Deriving it would write to `crates/mp/host-interface/src/mdx/`, which is read-only for this lane, so the fields are `Option<Arc<MdxmParsed>>` and `Option<Arc<MdxaParsed>>` instead. Every listed signature holds exactly as written: `set_parsed_mdxm(&mut self, parsed: MdxmParsed)` wraps, and `parsed_mdxm(&self) -> Option<&MdxmParsed>` unwraps through `as_deref`. Two side effects, both favorable. The index is shared by refcount rather than deep-copied on every copy-on-write poke, and its address stays stable across that poke, which matters because `model_mdxm_ptrs` hands it out as a `*const c_void`.

**Commit 1. `ModelBlock::new(AlignedBytes)`, `pub(crate)`.** The packet gives `ModelBlock` private fields, so the type needs a constructor to exist. It takes the ingested `AlignedBytes` and leaves both sidecars unset.

**Commit 1. The repeat arm of `re_register_server_models_malloc` returns `base_ptr() as *mut u8`.** This is the packet's own wording. The fresh arm uses `Arc::get_mut(...).base_ptr_mut()` with an expect naming the registration-completion mark as the first sharer.

**Commit 2. `ModelBlock::bump_generation`, `pub(crate)`.** `generation` is private and the packet asks the poke to bump it, so the type needs the mutator. It lands in commit 2 rather than commit 1, because an unused `pub(crate)` method warns.

**Commit 2. `RenderModels::block_base_ptr(name)`, `pub(crate)` in `cached_model_binary.rs`.** The packet's re-fetch lives in `frontend.rs`, and `disk_image` is private to `cached_model_binary.rs`, so the re-read needs a named reader. Keeping `poke_shader_index`'s signature, which the packet requires, rules out returning the new base from the poke itself.

**Commit 3. `RenderModels::block_containing(ptr)`, `pub(crate)` in `cached_model_binary.rs`.** Same privacy reason. `mark_block` must resolve each `model_t` block pointer to its owning cache entry, and vet finding 2 established that one MD3 slot spans up to three entries, so the resolution is by address containment across `self.cached`. A null pointer and a zero-length block both resolve to `None`.

**Commit 3. `mark_block` is `pub(super)`, not private.** The two registration-completion sites live in `server_load.rs` and `frontend.rs`, sibling modules inside `tr_model`.

**Commit 3. `blocks` and `blocks_dirty` are `pub(crate)`.** Every other `RenderModels` field is, for the same reason: the eviction sites that write them live in `cached_model_binary.rs`.

**Commit 3. `re_register_models_delete_all` calls `blocks.clear()` rather than a `remove_block` loop.** It clears the whole cache map, and every published block is owned by an entry in that map, so the two are equivalent and `clear` is one call.

**Commit 3. `PublishedModel::holds` is a private helper in `model_blocks.rs`,** colocated with `remove_block`, its only caller.

**Commit 3. `model_block.rs` and `model_blocks.rs` sit in `render_state` and hold a raw pointer through `AlignedBytes`.** That is against this module's interior-safety law, and the packet homes them there anyway. Both module docs record it as a justified exception, following the `handle.rs` precedent.

## Deviations

**One file outside the write scopes: `crates/mp/renderer-gpu/src/bin/dev_harness.rs`.** The packet records `empty_render_assets()` as "the single construction site" of `RenderAssets`. A grep found five `RenderAssets` producers, and three of them (`pipeline3d.rs:4815`, `stage2d.rs:522`, `ui_host/boot.rs:438`) delegate to `empty_render_assets`. The fourth, `dev_harness.rs:328`, is a second struct literal, so adding the `models` field to `RenderAssets` does not compile without naming it there. The edit is one field line plus one import, no behavior. It is the same class the packet's write scopes already authorize for the `RE_EndFrame` signature fallout, so I made it and flag it here rather than stop the lane over one line in a dev-harness binary.

**The sidecar `Arc`,** if the packet's private-field spelling is read as binding. Full reasoning under commit 1 above. No listed signature changed.

## Open gaps

- Nothing reads `RenderAssets.models` yet. `r_add_md3_surfaces`, `r_add_ghoul_surfaces` and the `tr_main.rs` dispatch arms keep their `models: &RenderModels` parameter and read exactly as before, which is the packet's stated scope. Step-003 migrates them.
- The dedicated server marks slots and never drains the flag, because `RE_EndFrame`'s one caller is the client's `SCR_UpdateScreen`. That is the packet's scope paragraph and vet finding 4, not a hole.
- `ModelBlocks::len` has no `is_empty` twin, so `clippy::len_without_is_empty` would fire on it. `len` is on the contract and `is_empty` is not, and clippy is not a gate for this step, so I did not add one. `ModelBlock::len` is in the same position.
- `mp_renderer` is not rustfmt-clean today, and this lane did not change that. Both new files are rustfmt-clean on their own.

## Pause triggers

None fired. No byte write through a `model_t` pointer turned up after a slot's mark beyond the poke the survey's section E already lists, the `Send` and `Sync` argument held at every reader I touched, the `RE_EndFrame` signature change reached its one caller cleanly through the existing `rm_from_view` seam, and no consumer needed `PublishedModel`.
