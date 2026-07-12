//! `ServerSkinSurface` — the per-surface row of a [`ServerSkin`] (user ruling
//! 2026-07-12 (server skins name-pool), amending the FROZEN `tr-model.md`).
//!
//! [`ServerSkin`]: super::server_skin::ServerSkin

/// Raven `skinSurface_t`, reshaped for the server-skins name-pool slice (user
/// ruling 2026-07-12): `shader: *mut shader_s` becomes an index into
/// `RenderModels.server_shaders` — server shader objects carry only the name,
/// the sole field the dedicated path ever reads (`G2_surfaces.cpp:212`).
///
/// Type reshape source: `oracle/codemp/renderer/tr_local.h:604-607`
pub struct ServerSkinSurface {
    /// `skinSurface_t.name` — lowercased at parse ("so skin compares are
    /// faster", `tr_image.cpp:3062`).
    pub(crate) name: String,
    /// `skinSurface_t.shader`, flattened to a `RenderModels.server_shaders`
    /// pool index.
    pub(crate) shader: usize,
}
