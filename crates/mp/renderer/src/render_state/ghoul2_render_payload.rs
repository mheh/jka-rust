//! `Ghoul2RenderPayload` - the per-entity Ghoul2 crossing of DEC-65 ruling 2.

use crate::render_state::ghoul2_model_render::Ghoul2ModelRender;

/// The per-entity Ghoul2 crossing of DEC-65 ruling 2: everything the render thread reads for one entity's skinned models.
/// `build_ghoul2_render_payload` (`tr_ghoul2.rs`) fills it at scene-add, and the entity walk plus the vertex decoder read it render-side.
/// It has no Raven counterpart, and every field is POD or owned, so the type is `Send + Sync` without unsafe.
pub struct Ghoul2RenderPayload {
    /// The `G2API_HaveWeGhoul2Models` answer at scene-add, which the `MOD_BAD` arm reads in place of the live instance list.
    pub have_models: bool,
    /// The render-visible models in `G2_Sort_Models` order, empty when `r_noServerGhoul2` suppressed the transform.
    pub models: Vec<Ghoul2ModelRender>,
}
