//! `tr_model` — the MP `jamp` dedicated-server model loader + cache (§F
//! idiomatic reimplementation of the WinDed DEDICATED renderer's live model
//! surface).
//!
//! Design: `docs/subsystems/tr-model.md` (FROZEN). The `RenderModels` aggregate
//! owner (`render_models`) holds the registry state; the sibling per-class
//! modules carry the disk-image buffer (`aligned_bytes`), the cache-entry type +
//! cache free-fns (`cached_model_binary`), the sole live model entry
//! (`server_load`), and the dedicated `.skin` registration family
//! (`server_skin`/`server_skin_surface`/`server_skins`, user ruling 2026-07-12
//! (server skins name-pool)). `matcomp` lives in `mp_engine_ghoul2`, not here
//! (`TRM-D1`(a)/ruling 56a).

pub mod aligned_bytes;
pub mod cached_model_binary;
pub mod render_models;
pub mod server_load;
pub mod server_skin;
pub mod server_skin_surface;
pub mod server_skins;
