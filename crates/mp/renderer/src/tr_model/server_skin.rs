//! `ServerSkin` — the `tr.skins[]` pool entry of the server-skins name-pool
//! slice (user ruling 2026-07-12 (server skins name-pool), amending the FROZEN
//! `tr-model.md`).

use super::server_skin_surface::ServerSkinSurface;

/// Raven `skin_t`, reshaped for the server-skins name-pool slice (user ruling
/// 2026-07-12): `surfaces` replaces the fixed `surfaces[128]` pointer array +
/// `numSurfaces` pair (the 128 cap is enforced at parse). Internal-only — the
/// skin never crosses the ABI seam (the game module holds just the
/// `qhandle_t`), so layout is free (§F17).
///
/// Raven: game path, including extension.
/// Type reshape source: `oracle/codemp/renderer/tr_local.h:609-613`
pub struct ServerSkin {
    /// `skin_t.name` — game path, including extension.
    pub(crate) name: String,
    /// `skin_t.surfaces[128]` + `numSurfaces`.
    pub(crate) surfaces: Vec<ServerSkinSurface>,
}
