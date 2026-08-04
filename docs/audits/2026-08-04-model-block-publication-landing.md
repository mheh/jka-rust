# Model-block publication, the landing (gh#31 step-002, DEC-65 ruling 1)

This record freezes the work lane that put the parsed md3, mdxm and mdxa byte blocks behind `Arc` and published them to the render thread inside `RenderAssets`. One opus work lane ran the four-commit bundle in `.claude/packets/31/step-002/packet.md` on branch `gh31-step-002-block-publication`, and the lane's own account is `.claude/packets/31/step-002/finished.md`. The landed range is `06aef52b..df358d32`, merged to master at `292f4039`. The pre-flight survey that grounded the packet is `docs/audits/2026-08-04-model-block-publication-survey.md`, and this record is its landing twin. Follow-up work goes to gh#31 step-003, the draw-arm migration.

## The lane report, verbatim

The lane is complete and held on the branch.

### Commits

| Commit | Subject |
| --- | --- |
| `06aef52b` | `refactor(gh#31 s002): ModelBlock owns the cached model bytes` |
| `155fa02f` | `feat(gh#31 s002): the shader poke becomes copy-on-write` |
| `2a201deb` | `feat(gh#31 s002): publish the model blocks into RenderAssets` |
| `af9ee9cb` | `docs(gh#31 s002): correct the never-Arc-published claims` |
| `1d8f061d` | `process(gh#31 s002): record the lane finished file` |
| `b326fbc6` | `style(gh#31 s002): lint the comments this lane touched` |
| `df358d32` | `process(gh#31 s002): close the finished file over the last two commits` |

### Gates

Every code commit: `cargo build --workspace` green, `cargo test --workspace` green with zero failures. World goldens byte-identical on commits 1 and 3, both fixtures. No `todo!()`, no `TODO: Port`, no extern forward-decl block in the diff. No pause trigger fired.

**The world-golden command in the brief needs `--test-threads=1`.** Without it both tests abort in the pk3 inflate path (`unzip.rs:40`) before any pixel comparison. The test file's own module doc already records this: "Serial only: two engine boots in parallel threads crash in the GPU init." Serially, both pass byte-identical. Pre-existing rig property, not this lane.

### What a reviewer should read first

1. **The sidecar `Arc` in `ModelBlock`** (`crates/mp/renderer/src/render_state/model_block.rs`). The packet spells the fields `Option<MdxmParsed>` / `Option<MdxaParsed>`. `Arc::make_mut` needs `ModelBlock: Clone`, and neither parsed type implements `Clone`. Deriving it would write to `crates/mp/host-interface/`, which this lane cannot touch, so the fields are `Option<Arc<MdxmParsed>>` / `Option<Arc<MdxaParsed>>`. Every listed signature holds unchanged, and the parsed index gains a stable address across a copy-on-write poke, which `model_mdxm_ptrs` hands out as a raw pointer.

2. **One file outside the write scopes: `crates/mp/renderer-gpu/src/bin/dev_harness.rs`.** The packet calls `empty_render_assets()` "the single construction site" of `RenderAssets`. Three of the five producers do delegate to it, but `dev_harness.rs:328` is a second struct literal, so the new field does not compile without naming it there. One field line plus one import, no behavior. Same class as the `RE_EndFrame` fallout the write scopes already authorize.

3. **Two private helpers the contract implies but does not list**, both in `cached_model_binary.rs` because `disk_image` is private to that file: `block_base_ptr(name)` for the packet's post-replay re-fetch in `frontend.rs`, and `block_containing(ptr)` for `mark_block`'s pointer-to-block resolution. Also `ModelBlock::new`, `ModelBlock::bump_generation`, and `mark_block` as `pub(super)` rather than private.

Full reasoning for all of these is in `.claude/packets/31/step-002/finished.md`, keyed to commits.

## Lane-review, 2026-08-04

The review ran in full-read mode, because this leg carries the renderer's first `unsafe impl Send`/`Sync` and its first cross-thread publication. The session read the whole diff itself, spawned no conformance clerk, and re-ran every gate rather than trusting the lane's claims. The re-run results: `cargo check --workspace --all-targets` reports zero warnings and zero errors, `cargo test --workspace` is green with zero failures, `golden_world_duel1` and `golden_world_ffa2` both pass byte-identical when run with `--test-threads=1`, and no commit carries a trailer of any kind.

The memory-safety argument behind the two `unsafe impl` lines was checked against the diff and holds. Every live `model_t` pointer's block is kept alive by either its cache entry or the published registry. Where a copy-on-write poke splits a block in two, the post-replay re-fetch in `re_register_models_malloc` moves `model_t` onto the live block, while a frame already in flight keeps the old block alive for as long as it reads it. On the fresh-load arm `Arc::make_mut` finds a unique block and writes it in place, so no split occurs there at all. All three cache-removal sites (`models_level_load_end`, `re_register_models_dump_non_pure`, `re_register_models_delete_all`) call `remove_block` or `clear`, which is what keeps `r_modelpoolmegs` reclamation freeing memory instead of leaving evicted bytes resident behind a registry clone.

## Ruling: the four deviations are accepted, 2026-08-04

The user accepted all four deviations and ruled the merge. Each one is the packet being wrong rather than the lane widening scope, and none adds functionality or changes a listed signature. The living text is the packet's Amendment 9 (`.claude/packets/31/step-002/packet.md`), and the four in short form:

1. The sidecars sit behind their own `Arc`, because `Arc::make_mut` needs `ModelBlock: Clone` and the parsed types do not implement it. The resulting shape is better than the packet's spelling.
2. `empty_render_assets()` is not the single `RenderAssets` construction site. `dev_harness.rs:328` is a second struct literal, and the lane edited that one file outside the write scopes.
3. Six helpers the contract implies but does not list, none of them `pub`. Two exist only because `disk_image` is private to `cached_model_binary.rs`.
4. Three commits past the four-commit bundle, one of which the lane-review skill itself requires.

The lane brief carried a wrong world-golden invocation. The gate needs `--test-threads=1`, which is a pre-existing property of the rig.
