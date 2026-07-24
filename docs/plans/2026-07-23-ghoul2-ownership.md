# Ghoul2 block-ownership campaign (task #17)

Ratified 2026-07-23 (sit-down; decisions D1-D4 below → DEC-35). Goal: retire
raw `*mut c_void` model-block traffic across the EngineHost seam and the
per-call `MdxaView::from_block`/`MdxmView::from_block` re-derivation pattern
(59 non-test sites, 14 files), and add a parsed-once index for the structures
the bone-transform/surface paths re-decode every frame (the debug-build perf
finding, memory `ghoul2-perf-debug-builds`).

Survey ground truth (2026-07-23): views are the crate's only byte-decode
chokepoint (zero bypasses); block owner is `AlignedBytes` inside
`RenderModels.cached` (`crates/mp/renderer/src/tr_model/`); handout is
`EngineHost::model_mdxm/model_mdxa -> *mut c_void`
(`crates/mp/host-interface/src/engine_host.rs:148,158`) via installed hooks
(`crates/mp/renderer/src/hook_install.rs:37-38`,
`engine_host_view.rs:223-241`); pointer caches are
`CGhoul2Info.{current_model,anim_model,a_header}` and `CBoneCache.header`
(both internal-only, reshape-free); `CBoneCache.header` is captured once and
never revalidated (latent use-after-eviction). Model bytes are mutated only
during one-time load ingest (endian swap), immutable after.

## Decisions

- **D1 — view types move to `mp_host_interface`.** The `mdx/` module
  (`MdxaView`/`MdxmView` + primitives) hoists from `mp_engine_ghoul2` into the
  seam crate both renderer and ghoul2 depend on; `EngineHost` returns views.
  This keeps G2SV-D5's substance (no `mp_engine_ghoul2 -> mp_renderer` edge,
  no duplicate file parse) — amend the letter of G2SV-D5/D15 in
  `docs/subsystems/ghoul2-server.md` with a pointer here. Motivation beyond
  ghoul2: the coming ui/cgame/renderer MP work will consume the same types.
- **D2 — caches store views.** `CGhoul2Info` and `CBoneCache` hold
  `MdxmView`/`MdxaView` (later `MdxmRef`/`MdxaRef`) instead of raw pointers.
  The unsafe pointer→view conjure collapses to the EngineHost impl seam.
  `G2_SetupModelPointers` stays the revalidation point; `CBoneCache` gains the
  same revalidation (closes the use-after-eviction hole).
- **D3 — targeted parse-once sidecar.** At ingest the renderer builds an owned
  immutable index per model — `MdxaParsed` (header constants + `mdxaSkel_t`
  table: name, parent, children, both base-pose matrices) and `MdxmParsed`
  (header constants + surface-hierarchy index) — stored beside the raw block.
  Handout becomes a Copy pair `MdxaRef { parsed: &MdxaParsed, view: MdxaView }`
  (mdxm likewise). Dividing line: **read more than once per model lifetime →
  parsed once; read once per frame → stays byte view** (compressed bone pool,
  vertices/triangles stay view-based). `MdxaParsed::parse(view)` lives in the
  hoisted mdx module (pure over bytes); the renderer calls it once at ingest —
  no second parse path, so G2SV-D15's re-parse rejection stands.
- **D4 — phases, each referee-gated** (`cargo build -p jampgame` then
  `cargo test -p jampgame --test referee -- --ignored`; workspace build+tests):
  ① hoist + EngineHost flip; ② cache reshape + revalidation; ③ parsed sidecar.
  After ③: live soak (`tools/live-soak/soak.sh`) + in-person saber-crowd test
  (referee doesn't exercise mass bone-transform load; the win condition is
  debug-build playability under NPC crowds).

## Lifetime shape (settled at sit-down close)

Hook fn pointers keep `*mut c_void` transport (ABI-simple). The `EngineHost`
impl performs the one documented `from_block` conjure and returns views.
Cache-stored views carry the same soundness contract today's raw pointers do
(valid until model eviction, revalidated by `G2_SetupModelPointers`) — the
unsafety is not eliminated, it is concentrated at one audited seam and made
revalidatable. Phase ③'s `&MdxaParsed` in `MdxaRef` follows the same contract
(parsed sidecar owned by the registry entry, dropped at eviction).

## Phase file map (from survey)

- ①: `crates/mp/engine/ghoul2/src/mdx/{mod,mdxa,mdxm}.rs` → hoist to
  `crates/mp/host-interface/src/mdx/`; `engine_host.rs` (trait, 2 methods +
  mock); renderer `engine_host_view.rs`, `hook_install.rs`; ghoul2 import
  rewire across the 14 consumer files (mechanical — module path only).
- ②: `shared/cghoul2_info.rs` (3 fields), `render/bone_cache.rs` (`header`
  field + `new`), `misc.rs` (`g2_setup_model_pointers` family), then the 59
  `from_block` sites collapse onto stored views: `ragdoll.rs`, `bolts.rs`,
  `api_collision.rs`, `api_bones.rs`, `surfaces.rs`, `api_surfaces.rs`,
  `misc.rs`, `api_saveload.rs`, `render/bone_transform.rs`, `api_ragdoll.rs`,
  `api_models.rs`, `bones.rs`, `render/skeleton.rs`, `render/bone_cache.rs`.
- ③: mdx module (`MdxaParsed`/`MdxmParsed` + `parse`), renderer
  `server_load.rs`/`cached_model_binary.rs`/`render_models.rs` (build + store
  + hand out), hot-path consumers (`bone_transform.rs`, `bone_cache.rs`,
  `skeleton.rs`, `surfaces.rs`, `api_collision.rs`) onto `parsed` fields.
