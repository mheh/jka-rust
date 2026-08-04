# Packet gh#31 step-002 - DEC-65 ruling 1: model-block publication

## Scope

This step puts the parsed md3/mdxm/mdxa byte blocks behind `Arc` and publishes them to the render thread inside `RenderAssets`, per DEC-65 ruling 1 (`docs/decisions.md:1526-1534`). It delivers three things: the block type that owns the bytes, the published registry that the frame package carries, and the copy-on-write rule for the one post-load write.

The step does not migrate any consumer. `r_add_md3_surfaces` (`tr_mesh.rs:352`), `r_add_ghoul_surfaces` (`tr_ghoul2.rs:2254`), and the `tr_main.rs:2017-2119` dispatch arms keep their `models: &RenderModels` parameter and read exactly as they do today. The draw-arm migration is step-003. The step also does not unify the server and client model registries (`render_assets.rs:133-139`), does not touch `ModelPool`'s `Box` entries or `model_t`'s layout, and does not touch bone matrices (DEC-65 ruling 2).

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
    pub fn len(&self) -> usize;
    pub fn generation(&self) -> u32;
    pub fn parsed_mdxm(&self) -> Option<&MdxmParsed>;
    pub fn parsed_mdxa(&self) -> Option<&MdxaParsed>;
}

impl Clone for ModelBlock { /* deep-copies the AlignedBytes, for Arc::make_mut */ }

// SAFETY comment required at the impl, stating: the block owns its bytes, the
// bytes are immutable while an Arc is shared (ruling B), and the type has no
// interior mutability.
unsafe impl Send for ModelBlock {}
unsafe impl Sync for ModelBlock {}
```

Private fields are the moved `AlignedBytes`, the two sidecars `parsed_mdxm: Option<MdxmParsed>` / `parsed_mdxa: Option<MdxaParsed>` relocated from `CachedEndianedModelBinary`, and `generation: u32`.

**New file `crates/mp/renderer/src/render_state/model_blocks.rs`:**

```rust
/// One entry per registered model slot. Offsets, never pointers.
pub struct PublishedModel {
    pub block: Arc<ModelBlock>,
    pub model_type: modtype_t,
    pub num_lods: i32,
    pub md3_offsets: [Option<usize>; 3],
    pub mdxm_offset: Option<usize>,
    pub mdxa_offset: Option<usize>,
}

#[derive(Clone, Default)]
pub struct ModelBlocks { /* private: Vec<Option<PublishedModel>> keyed by slot */ }

impl ModelBlocks {
    pub fn get(&self, handle: qhandle_t) -> Option<&PublishedModel>;
    pub fn len(&self) -> usize;
}
```

`PublishedModel` derives `Clone`. It carries byte offsets into its own block, computed at publish time by subtracting the block base from `model_t`'s `md3`/`mdxm`/`mdxa` pointers. It holds no raw pointer, so `ModelBlocks` is `Send`/`Sync` through `ModelBlock`'s impls alone.

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

Every current reader reaches the bytes through `block.bytes()` or `block.base_ptr()`. The `AlignedBytes` allocation itself never moves, so `model_t`'s three raw pointers stay valid and the `aligned_bytes.rs:13-15` pinning contract holds unchanged.

**`RenderModels` (`render_models.rs:46-123`)** gains two fields and two methods:

```rust
blocks: ModelBlocks,
blocks_dirty: bool,

impl RenderModels {
    pub fn publish_blocks(&mut self) -> Option<ModelBlocks>;   // Some only when dirty, clears the flag
    fn mark_block(&mut self, handle: qhandle_t);               // records one slot into self.blocks
}
```

`mark_block` runs at each load-completion site: `register_server_model` (`server_load.rs:135`), `server_load_mdxa` (`:280`), `server_load_mdxm` (`:388`), and `r_load_md3` (`frontend.rs:491`). No producer signature changes. The dedicated-server path never gains a `RenderAssets` parameter, which is why the dirty flag exists.

**`poke_shader_index` (`cached_model_binary.rs:456-478`)** keeps its signature and changes its body to ruling B: `Arc::make_mut` on the entry's block, the unaligned `i32` write into the resulting unique block, then a generation bump on that block, then `mark_block` for the affected slot. The `unsafe` write stays, with its safety comment updated to state the block is unique at the write.

**`RenderAssetsSim` (`render_assets_sim.rs`)** gains the first publish body:

```rust
pub fn publish_models(&mut self, blocks: ModelBlocks);   // Arc::make_mut(&mut self.published).models = Arc::new(blocks)
```

**`RE_EndFrame` (`tr_cmds.rs:332-381`)** changes its `sim` parameter from `&RenderAssetsSim` to `&mut RenderAssetsSim`, and calls `publish_models` before the `Arc::clone(&sim.published)` on line `:377` when `RenderModels::publish_blocks` returns `Some`. This mirrors the `pending_world.take()` handoff on line `:380`.

Anything not on this list is out of scope, and the agent must not add it. No new third-party crate. No `#[repr]` change, no layout assert change, no cvar, no `FrameEvent` variant, no trap arm.

## Pause triggers, named for this step

- The `Send`/`Sync` safety argument does not hold for some reader found during the work, for example a live `&mut` into a block that is already published. STOP and report; do not widen the impl to cover it.
- `RE_EndFrame`'s `&mut` change touches a caller that cannot supply `&mut`. STOP; the alternative is a ruling.
- Any consumer needs `PublishedModel` before step-003. STOP; the draw-arm migration is not in this scope.

## Commit bundle

1. **`ModelBlock` lands and owns the bytes.** New `model_block.rs`, `CachedEndianedModelBinary.disk_image` becomes `Option<Arc<ModelBlock>>`, both sidecars move into the block, every existing reader in `cached_model_binary.rs`, `server_load.rs`, and `frontend.rs` reaches through the accessors. Behavior is unchanged at this commit. Gates: `cargo build --workspace`, `cargo test --workspace`, plus the world goldens byte-identical, run locally with `cargo test -p mp_renderer_gpu --test world_golden -- --ignored`.
2. **Ruling B: the poke becomes copy-on-write.** `poke_shader_index` goes through `Arc::make_mut` and bumps the block generation. Add a unit test proving a held `Arc<ModelBlock>` keeps its old bytes across a poke and the new block carries the new value and a higher generation. Gates: `cargo build --workspace`, `cargo test --workspace`.
3. **Ruling A: `ModelBlocks` publishes into `RenderAssets`.** New `model_blocks.rs`, the `RenderAssets.models` field, the `RenderModels` dirty flag with `mark_block` at the four load-completion sites, `publish_models` on `RenderAssetsSim`, and the `RE_EndFrame` handoff with its `&mut` change. Gates: `cargo build --workspace`, `cargo test --workspace`, world goldens byte-identical.
4. **Doc close-out.** The `ModelPool` type doc (`model_pool.rs:12-15,58-61`) and the `EntityWalkHost` doc (`tr_main.rs:167-171`) both state today that the blocks can never enter an `Arc`-published registry. Correct both to the post-DEC-65 truth: the pool keeps its `Box` entries and its address-stability contract, and the published copy is `Arc<ModelBlocks>`. Gates: `cargo build --workspace`, `cargo test --workspace`.

Every commit uses `--no-gpg-sign`, a heading subject, an STE body, and no trailer of any kind.

## Write scopes

Branch `gh31-step-002-block-publication`, cut from master.

- `crates/mp/renderer/src/render_state/` - `model_block.rs`, `model_blocks.rs`, `mod.rs`, `render_assets.rs`, `render_assets_sim.rs`.
- `crates/mp/renderer/src/tr_model/` - `cached_model_binary.rs`, `render_models.rs`, `server_load.rs`, `frontend.rs`, `model_pool.rs` (doc only), `aligned_bytes.rs` (doc only).
- `crates/mp/renderer/src/` - `tr_cmds.rs`, `renderer_frontend.rs`, `tr_main.rs` (doc only).
- Any caller that `cargo check` shows broken by the `RE_EndFrame` `&mut` change, edit-only to pass `&mut`.
- `.claude/packets/31/step-002/` for `finished.md`.

Everything else is read-only, including `oracle/`.

## Disposition

Hold on the branch. Lane-review runs against this packet, and a clean review merges to master locally. No push, and no pull request.

## Amendments

None yet.
