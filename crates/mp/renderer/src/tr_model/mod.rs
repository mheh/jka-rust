//! `tr_model` — the MP `jamp` dedicated-server model loader + cache (§F
//! idiomatic reimplementation of the WinDed DEDICATED renderer's live model
//! surface).
//!
//! Design: `docs/subsystems/tr-model.md` (FROZEN). The `RenderModels` aggregate
//! owner (`render_models`) holds the registry state; the sibling per-class
//! modules carry the disk-image buffer (`aligned_bytes`), the cache-entry type +
//! cache free-fns (`cached_model_binary`), and the sole live model entry
//! (`server_load`). `matcomp` lives in `mp_engine_ghoul2`, not here
//! (`TRM-D1`(a)/ruling 56a).

pub mod aligned_bytes;
pub mod cached_model_binary;
pub mod render_models;
pub mod server_load;
