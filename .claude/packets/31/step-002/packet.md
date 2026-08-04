# Packet gh#31 step-002 - DEC-65 ruling 1: model-block publication

## Scope

This step puts the parsed md3/mdxm/mdxa byte blocks behind `Arc` and publishes them to the render thread inside `RenderAssets`, per DEC-65 ruling 1 (`docs/decisions.md:1526-1534`). It delivers three things: the block type that owns the bytes, the published registry that the frame package carries, and the copy-on-write rule for the one post-load write.

The step does not migrate any consumer. `r_add_md3_surfaces` (`tr_mesh.rs:352`), `r_add_ghoul_surfaces` (`tr_ghoul2.rs:2254`), and the `tr_main.rs:2017-2119` dispatch arms keep their `models: &RenderModels` parameter and read exactly as they do today. The draw-arm migration is step-003. The step also does not unify the server and client model registries (`render_assets.rs:133-139`), does not touch `ModelPool`'s `Box` entries or `model_t`'s layout, and does not touch bone matrices (DEC-65 ruling 2).

The dedicated server never publishes, and that is correct for this step. `RE_EndFrame`'s single caller is the client's `SCR_UpdateScreen` (`cl_scrn.rs:892`), so on `jampded` the dirty flag sets at registration and never drains. No render thread exists there to read a publication, so the undrained flag is inert bookkeeping, not a hole.

Two user rulings from 2026-08-04 bind this packet:

- **Ruling A.** The published blocks live as a nested `Arc` field inside `RenderAssets`, beside `world: Option<Arc<WorldAsset>>`. They do not stand as a sibling registry, and they do not become a flat `Arena` entry.
- **Ruling B.** `poke_shader_index` becomes per-block copy-on-write. It calls `Arc::make_mut` on the one affected block, writes the `i32`, and bumps that block's generation. No in-place write lands in a block another thread may hold.

Ground truth is the survey record `docs/audits/2026-08-04-model-block-publication-survey.md` and the signature survey of 2026-08-04. Three facts shape the design:

1. The render thread is a real OS thread. `jamp-sim` spawns at `crates/mp/client-app/src/main.rs:69-72`, `jamp-render` at `crates/mp/client-app/src/pump.rs:185-188`, and `FramePackage` crosses between them over an `mpsc::sync_channel` (`pump.rs:15,34`).
2. `AlignedBytes` holds `ptr: NonNull<u8>` (`aligned_bytes.rs:40-43`) and carries no `Send`/`Sync` impl, so it is `!Send` and `!Sync` today. `RenderAssets` is `Send`/`Sync` only because no current field holds a raw pointer. A published block is the first raw-pointer holder in this scheme, and no `unsafe impl Send`/`Sync` exists anywhere in `crates/mp/renderer/` or `crates/mp/host-interface/` to copy.
3. `RenderAssetsSim` has no `Arc::make_mut` call body today (`render_assets_sim.rs:25-61`). Its type doc describes the mechanics, and this step writes the first publish body.

## Surface contract

**New file `crates/mp/renderer/src/render_state/model_block.rs`:**

```rust
/// One published model's bytes plus its parse-once sidecars.
pub struct ModelBlock { /* private fields */ }

impl ModelBlock {
    pub fn bytes(&self) -> &[u8];
    pub fn base_ptr(&self) -> *const u8;
    pub(crate) fn base_ptr_mut(&mut self) -> *mut u8;  // load-time writes only, reached through Arc::get_mut or Arc::make_mut
    pub(crate) fn set_parsed_mdxm(&mut self, parsed: MdxmParsed);
    pub(crate) fn set_parsed_mdxa(&mut self, parsed: MdxaParsed);
    pub fn len(&self) -> usize;
    pub fn generation(&self) -> u32;
    pub fn parsed_mdxm(&self) -> Option<&MdxmParsed>;
    pub fn parsed_mdxa(&self) -> Option<&MdxaParsed>;
}

impl Clone for ModelBlock { /* deep-copies the AlignedBytes, for Arc::make_mut */ }

// SAFETY comment required at the impl, stating the four invariants:
// 1. The block owns its allocation, and the type has no interior mutability.
// 2. Every byte write happens while the Arc is unique: the load-time endian
//    swaps and the sidecar parse run before the slot's registration-completion
//    mark (Arc::get_mut), and the shader poke runs through Arc::make_mut.
// 3. The raw pointers model_t derives from base_ptr are sim-thread-only and
//    read-only after the mark. The render thread reads bytes and offsets only.
// 4. The allocation is freed exactly once, by the last Arc drop.
unsafe impl Send for ModelBlock {}
unsafe impl Sync for ModelBlock {}
```

Private fields are the moved `AlignedBytes`, the two sidecars `parsed_mdxm: Option<MdxmParsed>` / `parsed_mdxa: Option<MdxaParsed>` relocated from `CachedEndianedModelBinary`, and `generation: u32`. The two sidecar setters replace the direct field writes in `store_parsed_mdxm`/`store_parsed_mdxa` (`cached_model_binary.rs:302-326`), which run on the fresh-load path only, before the slot's mark, so `Arc::get_mut` reaches them.

**New file `crates/mp/renderer/src/render_state/model_blocks.rs`:**

```rust
/// One entry per registered model slot. Blocks plus offsets, never pointers.
pub struct PublishedModel {
    pub model_type: modtype_t,
    pub num_lods: i32,
    pub md3: [Option<(Arc<ModelBlock>, usize)>; 3],
    pub mdxm: Option<(Arc<ModelBlock>, usize)>,
    pub mdxa: Option<(Arc<ModelBlock>, usize)>,
}

#[derive(Clone, Default)]
pub struct ModelBlocks { /* private: Vec<Option<PublishedModel>> keyed by slot */ }

impl ModelBlocks {
    pub fn get(&self, handle: qhandle_t) -> Option<&PublishedModel>;
    pub fn len(&self) -> usize;
    pub(crate) fn set(&mut self, handle: qhandle_t, entry: PublishedModel);
    pub(crate) fn remove_block(&mut self, block: &Arc<ModelBlock>);  // drops every entry that holds this block, by Arc::ptr_eq
    pub(crate) fn clear(&mut self);
}
```

`PublishedModel` derives `Clone`. Each family field pairs the owning `Arc<ModelBlock>` with a byte offset into that block, computed at mark time by subtracting the block base from `model_t`'s `md3`/`mdxm`/`mdxa` pointers. One slot's LODs live in up to three cache entries, because `RE_RegisterModel_Actual` builds a per-LOD file name (`frontend.rs:998-1005`) and each file is its own cache entry, so a single per-slot block cannot represent an MD3 model. The struct holds no raw pointer, so `ModelBlocks` is `Send`/`Sync` through `ModelBlock`'s impls alone. The three `pub(crate)` writers exist for `RenderModels`, which lives in the sibling `tr_model` module.

**`RenderAssets` (`render_assets.rs`)** gains exactly one field, placed beside `world`:

```rust
pub models: Arc<ModelBlocks>,
```

The struct comment at `:133-139` gains one sentence recording that DEC-65 ruling 1 publishes the blocks here while the registry itself stays on `RenderModels`. The single construction site `empty_render_assets()` (`renderer_frontend.rs:155-157`) gains the field.

**`CachedEndianedModelBinary` (`cached_model_binary.rs:104-159`)** changes one field and drops two:

```rust
disk_image: Option<Arc<ModelBlock>>,   // was Option<AlignedBytes>
// parsed_mdxm and parsed_mdxa move into ModelBlock
```

Every current reader reaches the bytes through `block.bytes()` or `block.base_ptr()`. The load-time writers (`re_register_server_models_malloc`'s returned `*mut u8` and the endian swaps behind it) reach them through `base_ptr_mut` behind `Arc::get_mut` on the fresh arm, where the Arc is provably unique. The repeat arm returns `base_ptr()` cast to `*mut u8` with a comment that no write follows it, which the three loaders' early already-found returns guarantee (`frontend.rs:557`, `server_load.rs`, `tr_ghoul2.rs`). The `AlignedBytes` allocation never moves while its Arc lives, so `model_t`'s raw pointers stay valid and the `aligned_bytes.rs:13-15` pinning contract holds. The one exception is commit 2's copy-on-write poke, and the base-pointer re-fetch below closes it.

**`re_register_models_malloc` (`frontend.rs:371-428`)** keeps its signature and gains one body change: after the `already_found` shader-poke replay loop (`:396-425`), it re-reads the entry's current block base and returns that pointer, never the pre-replay one. `Arc::make_mut` inside a poke replaces the entry's block when the published registry still holds the old Arc, so the pre-replay base can point at the clone source. All three callers (`r_load_md3` at `frontend.rs:554-555`, `r_load_mdxm` and `r_load_mdxa` in `tr_ghoul2.rs`) store the returned pointer into `model_t` after the call, so this one re-fetch keeps every `model_t` pointer on the live block.

**`RenderModels` (`render_models.rs:46-123`)** gains two fields and two methods:

```rust
blocks: ModelBlocks,
blocks_dirty: bool,

impl RenderModels {
    pub fn publish_blocks(&mut self) -> Option<ModelBlocks>;   // Some only when dirty, clears the flag
    fn mark_block(&mut self, handle: qhandle_t);               // records one slot into self.blocks
}
```

`mark_block` runs at the two top-level registration-completion sites, after each LOD-duplicate loop so the recorded pairs match the final `model_t` pointers: `register_server_model`'s success return (`server_load.rs`, the `num_loaded != 0` arm) and `RE_RegisterModel_Actual`'s success return (`frontend.rs:889`, after the duplicate loop at `:1108-1110`). The per-format loaders do not mark, because the top-level mark sees the finished slot and also covers the already-found poke replay. No producer signature changes. The dedicated-server path never gains a `RenderAssets` parameter, which is why the dirty flag exists.

Eviction and reset keep `blocks` honest. Every cache-entry removal (`models_level_load_end`, `re_register_models_dump_non_pure`, `re_register_models_delete_all`) also calls `blocks.remove_block` with the removed entry's Arcs and sets the dirty flag, so an evicted block's bytes actually free instead of staying alive through the `blocks` clone. `model_init` and `hunk_clear` call `blocks.clear()` and set the dirty flag, matching their pool resets. Without this, `r_modelpoolmegs` reclamation stops freeing memory on the live dedicated server.

**`poke_shader_index` (`cached_model_binary.rs:456-478`)** keeps its signature and changes its body to ruling B: `Arc::make_mut` on the entry's block, the unaligned `i32` write into the resulting unique block, then a generation bump on that block. The `unsafe` write stays, with its safety comment updated to state the block is unique at the write. The poke does not mark, because at poke time the model's name is not in the hash yet (`RE_RegisterModel_Actual` inserts it at completion), so no name-to-handle resolution exists there. The top-level completion mark records the slot with the post-poke block. A load that pokes and then fails its post-malloc checks leaves the old published entry in place, which is acceptable: the slot resolves as dead, so no consumer reaches the stale entry.

**`RenderAssetsSim` (`render_assets_sim.rs`)** gains the first publish body:

```rust
pub fn publish_models(&mut self, blocks: ModelBlocks);   // Arc::make_mut(&mut self.published).models = Arc::new(blocks)
```

**`RE_EndFrame` (`tr_cmds.rs:332-381`)** changes its `sim` parameter from `&RenderAssetsSim` to `&mut RenderAssetsSim` and gains one parameter, `rm: &mut RenderModels`. The current signature carries no path to `RenderModels`, which lives on the engine as `view.rm`, not on `RendererFrontend`, so the drain cannot compile without it. After the `:345` registered guard and before the sink match, the body calls `publish_models` when `rm.publish_blocks()` returns `Some`, so the flag drains whether or not a sink is installed. This mirrors the `pending_world.take()` handoff on line `:380`. The single caller is `SCR_UpdateScreen` (`cl_scrn.rs:892`). It already holds `&mut re` through `re_from_view`, and it passes `rm` from the `view.rm` slot the way `RE_RegisterMedia_LevelLoadBegin` does (`sv_renderer.rs:57`).

Anything not on this list is out of scope, and the agent must not add it. No new third-party crate. No `#[repr]` change, no layout assert change, no cvar, no `FrameEvent` variant, no trap arm.

## Pause triggers, named for this step

- The `Send`/`Sync` safety argument does not hold for some reader found during the work, for example a live `&mut` into a block that is already published. STOP and report; do not widen the impl to cover it.
- `RE_EndFrame`'s signature change (`&mut sim`, the `rm` parameter) touches a caller that cannot supply the new arguments. STOP; the alternative is a ruling.
- A byte write through a `model_t` raw pointer turns up after a slot's mark, anywhere the pre-flight survey (`docs/audits/2026-08-04-model-block-publication-survey.md`, section E) did not list. STOP; that breaks the `Send`/`Sync` invariant 2.
- Any consumer needs `PublishedModel` before step-003. STOP; the draw-arm migration is not in this scope.

## Commit bundle

1. **`ModelBlock` lands and owns the bytes.** New `model_block.rs`, `CachedEndianedModelBinary.disk_image` becomes `Option<Arc<ModelBlock>>`, both sidecars move into the block, every existing reader in `cached_model_binary.rs`, `server_load.rs`, `frontend.rs`, and `tr_ghoul2.rs` reaches through the accessors. At this commit the Arc is never shared, so the writers (`base_ptr_mut`, the sidecar setters, and the interim `poke_shader_index`) reach unique access through `Arc::get_mut` with an expect that names commit 3 as the first sharer. Behavior is unchanged at this commit. Gates: `cargo build --workspace`, `cargo test --workspace`, plus the world goldens byte-identical, run locally with `cargo test -p mp_renderer_gpu --test world_golden -- --ignored`.
2. **Ruling B: the poke becomes copy-on-write.** `poke_shader_index` goes through `Arc::make_mut` and bumps the block generation, and `re_register_models_malloc` re-fetches the base pointer after the replay loop. Add a unit test proving a held `Arc<ModelBlock>` keeps its old bytes across a poke and the entry's new block carries the new value and a higher generation. Gates: `cargo build --workspace`, `cargo test --workspace`.
3. **Ruling A: `ModelBlocks` publishes into `RenderAssets`.** New `model_blocks.rs`, the `RenderAssets.models` field, the `RenderModels` dirty flag with `mark_block` at the two registration-completion sites, the eviction and reset hooks (`remove_block`/`clear` plus dirty), `publish_models` on `RenderAssetsSim`, and the `RE_EndFrame` handoff with its `&mut sim` and `rm` parameter changes. Add a unit test that ingests a block, marks a slot, and asserts `publish_blocks` returns pairs whose Arcs are `ptr_eq` with the cache entry's, that offsets match the `model_t` pointer subtraction, that a second call returns `None`, and that eviction plus re-publish drops the slot. Add a compile-time `Send + Sync` assert for `ModelBlock` and `ModelBlocks`. Gates: `cargo build --workspace`, `cargo test --workspace`, world goldens byte-identical.
4. **Doc close-out.** The `ModelPool` type doc (`model_pool.rs:12-15,58-61`) and the `EntityWalkHost` doc (`tr_main.rs:167-171`) both state today that the blocks can never enter an `Arc`-published registry. Correct both to the post-DEC-65 truth: the pool keeps its `Box` entries and its address-stability contract, and the published copy is `Arc<ModelBlocks>`. Gates: `cargo build --workspace`, `cargo test --workspace`.

Every commit uses `--no-gpg-sign`, a heading subject, an STE body, and no trailer of any kind.

## Write scopes

Branch `gh31-step-002-block-publication`, cut from master.

- `crates/mp/renderer/src/render_state/` - `model_block.rs`, `model_blocks.rs`, `mod.rs`, `render_assets.rs`, `render_assets_sim.rs`.
- `crates/mp/renderer/src/tr_model/` - `cached_model_binary.rs`, `render_models.rs`, `server_load.rs`, `frontend.rs`, `model_pool.rs` (doc only), `aligned_bytes.rs` (doc only).
- `crates/mp/renderer/src/` - `tr_cmds.rs`, `renderer_frontend.rs`, `tr_main.rs` (doc only), `tr_ghoul2.rs` (reader-accessor conversion in `r_load_mdxm`/`r_load_mdxa` only).
- Any caller that `cargo check` shows broken by the `RE_EndFrame` signature change (`&mut sim` and the new `rm` parameter), edit-only to pass the new arguments. `cl_scrn.rs:892` is the known one.
- `.claude/packets/31/step-002/` for `finished.md`.

Everything else is read-only, including `oracle/`.

## Disposition

Hold on the branch. Lane-review runs against this packet, and a clean review merges to master locally. No push, and no pull request.

## Amendments

**2026-08-04 - packet vet, seven findings. Every body correction below is already folded in place, so the body reads as one contract.**

### 1. The copy-on-write poke stranded every `model_t` pointer on the clone source

Ruling: confirmed defect in the draft, closed by one mechanism. `re_register_models_malloc` computed the base pointer before the replay (`frontend.rs:388`), poked at `:423`, and returned the pre-replay pointer at `:427`. All three callers store that pointer into `model_t` after the call (`frontend.rs:554-555`, `r_load_mdxm`/`r_load_mdxa` in `tr_ghoul2.rs`). When the `blocks` registry or a published frame holds a second Arc, `Arc::make_mut` clones, so the stored pointer targets the old allocation. The reads go stale at once and dangle when the render thread drops the old Arc, a use-after-free at `tr_mesh.rs:372` and through `model_mdxm_ptrs`/`model_mdxa_ptrs` (`cached_model_binary.rs:328-376`). The `CBoneCache` `MdxaRef` is safe once `model_t` is fresh, because `skeleton.rs:450` re-fetches it at the top of every transform pass and no pass spans a poke on the single sim thread. The smallest mechanism that satisfies ruling B as stated: re-fetch the entry's base pointer after the replay loop and return that. It is now in the surface contract and in commit 2. Changed lines: the `CachedEndianedModelBinary` paragraph, the new `re_register_models_malloc` paragraph, commit 2.

### 2. One `PublishedModel` block cannot represent a multi-LOD MD3

Ruling: shape defect, corrected. `RE_RegisterModel_Actual` builds a per-LOD file name (`frontend.rs:998-1005`), so `model.md3[0..3]` point into up to three distinct cache entries with three distinct blocks. The draft's single `block: Arc<ModelBlock>` plus offset arrays cannot name which block an offset indexes. `PublishedModel` now pairs each family slot with its owning Arc. Ruling A is untouched: the blocks still live as the one nested `Arc<ModelBlocks>` field beside `world`. Changed lines: the `PublishedModel` struct and its paragraph.

### 3. `RE_EndFrame` had no path to `RenderModels`

Ruling: the draft's drain could not compile. `RE_EndFrame`'s parameter list (`tr_cmds.rs:332-344`) carries no `RenderModels` and no view, and `RenderModels` lives on the engine as `view.rm` (`renderer_frontend.rs:62`), not on `RendererFrontend`. The `&mut sim` half is feasible: the single caller `SCR_UpdateScreen` (`cl_scrn.rs:892`) holds `&mut re` through `re_from_view`, and disjoint field borrows cover it. The fix is the added `rm: &mut RenderModels` parameter, passed from the `view.rm` slot on the `sv_renderer.rs:57` pattern. The drain also moved above the sink match so it runs with no sink installed. Changed lines: the `RE_EndFrame` paragraph, commit 3, the write-scope caller line, the pause trigger.

### 4. The dedicated server never reaches `RE_EndFrame`, and that is correct

Ruling: not a hole, now stated. The one call site is the client's `SCR_UpdateScreen` (`cl_scrn.rs:892`), so on `jampded` the dirty flag never drains and no render thread exists to want the publication. Changed lines: the new scope paragraph.

### 5. Eviction and reset would have kept evicted bytes alive

Ruling: hole, closed. `mark_block` clones an Arc into `RenderModels.blocks`, and the draft removed those clones nowhere. Every eviction (`models_level_load_end`, `re_register_models_dump_non_pure`, `re_register_models_delete_all`) would drop the cache entry's Arc while the `blocks` clone kept the bytes resident, so `r_modelpoolmegs` reclamation (`cached_model_binary.rs:510-513`) stops freeing memory on the live dedicated server, and `get_model_data_alloc_size` undercounts residency. The pool resets (`model_init`, `hunk_clear`) had the same gap. The contract now carries `remove_block`/`clear` hooks at those sites, with the dirty flag set. Changed lines: the `ModelBlocks` impl, the eviction paragraph under `RenderModels`, commit 3.

### 6. The `Send`/`Sync` argument is sound at this scope, with four named invariants

Ruling: a sound argument exists, and the impl comment must carry it in full. The four invariants now sit in the surface contract: no interior mutability, writes only under a unique Arc (load-time swaps and sidecar parse before the completion mark, poke through `Arc::make_mut`), `model_t`-derived raw pointers sim-thread-only and read-only after the mark, single free at last drop. The ground for invariant 2 is the survey's section E: the only post-load byte write in the tree is the poke. The sidecars are safe to share because `MdxmParsed` stores owned data and absolute byte offsets, not pointers (`mdxm.rs:437-453`), so a clone stays valid. The impls stay on `ModelBlock` only, and `AlignedBytes` stays `!Send`, so no other holder inherits the claim. A new pause trigger names the one event that would break invariant 2. The mark ordering constraint this argument needs, sidecar parse before `mark_block`, holds because the loaders call `store_parsed_*` on the fresh path and the mark now runs at top-level completion. Changed lines: the SAFETY comment block, the sidecar paragraph, the pause triggers.

### 7. The step shipped unverified, and the poke could not mark

Ruling: two verification gaps, closed. First, no gate exercised publication, because the world goldens carry no models and no consumer reads the published blocks until step-003. Commit 3 now carries a publication unit test (mark, publish, `ptr_eq` and offset asserts, second-call `None`, eviction drop) and a compile-time `Send + Sync` assert. Second, the draft had the poke call `mark_block`, but at poke time the name-to-handle hash entry does not exist yet (`RE_RegisterModel_Actual` inserts it at completion), so the poke cannot resolve a slot. The mark moved to the two top-level registration-completion sites, which also fixes the draft's four-site list: it missed the `tr_ghoul2.rs` client loaders entirely, and a per-loader mark would have recorded MD3 offsets before the LOD-duplicate loop finished the slot. Changed lines: the mark paragraph, the poke paragraph, commits 2 and 3, the write scopes (`tr_ghoul2.rs` reader conversion).

**2026-08-04 - ratified.** The user ratified the packet as amended, with no further rulings and no edits to the body. The seven vet findings above stand as written, and the lane spawns on branch `gh31-step-002-block-publication`.
