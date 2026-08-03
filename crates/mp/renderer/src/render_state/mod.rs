//! `render_state` — the renderer's root types (R3 skeleton).
//!
//! Design: `docs/subsystems/renderer-r2-design.md` (FROZEN) — `## Seam
//! definition`, `## State ownership`, `### FrameData`, and `R2-D1`…`R2-D11`;
//! DEC-37 rulings 1/3/11/14 + addenda A1-A13. Nothing here has a Raven
//! counterpart struct-for-struct: `trGlobals_t`/`backEndState_t`/
//! `backEndData_t` are re-partitioned into `RenderAssets` (CPU, `Arc`-shared,
//! sim-readable), `FrameState` (render-thread scratch) and
//! `FrameData`/`FrameEvent` (the ordered event stream that replaces
//! `renderCommandList_t`). The GPU tier is `mp_renderer_gpu`'s alone since
//! DEC-63.4 deleted the empty `GpuResources` carrier.
//!
//! **Interior-safety law** (`### Type tiers and the interior-safety law`,
//! binding on every R3/R4 wave): no type in this module carries raw pointers,
//! `c_char` buffers, or `qboolean`-style ints — handles, indices, owned
//! `String`/`Vec`, and `bool` only. `#[repr(C)]` belongs to the tier-1 seam
//! set, never here.
//!
//! R2 scope is struct/enum shapes: the `RenderAssets` registration algorithms
//! land at R3, the GPU crate internals at R4. The names the design lists as
//! "named but not defined here" live in `placeholders` until their owning wave
//! gives them a real shape.

pub mod arena;
pub mod frame_data;
pub mod frame_event;
pub mod frame_state;
pub mod handle;
pub mod image_asset;
pub mod light_style_table;
// `model_asset` (the `ModelAsset` payload + `ModelHandle` alias) is retired:
// the model registry keeps its arena mechanics inside `RenderModels`' own pool
// (`crate::tr_model::model_pool`), per `docs/subsystems/tr-model.md`
// `## Amendment 2026-07-27 — models pool: arena mechanics` (#51). Unifying the
// server and client model registries is deferred to the client-engine island.
pub mod placeholders;
pub mod render_assets;
pub mod render_assets_sim;
pub mod render_cvar_snapshot;
pub mod render_world;
pub mod renderer_cvars;
pub mod shader_asset;
pub mod shader_stage;
pub mod skin_asset;
pub mod texture_bundle;
