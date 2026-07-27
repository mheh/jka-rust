//! `ShaderStage` — `ShaderAsset::stages`' element.

use crate::render_state::image_asset::ImageHandle;

/// The owned form of Raven `shaderStage_t` — `ShaderAsset::stages`' element
/// (`R2-D3`), landed field-by-field as call sites need them, same precedent
/// as `ShaderAsset` itself (`shader_asset.rs`'s own doc comment). Real fields
/// so far: `bundle[0].image` -> `image: Option<ImageHandle>` (`RB_RotatePic`/
/// `RB_RotatePic2`'s `if (image)` guard), `stateBits` -> `state_bits: u32`
/// (`RB_RotatePic2`'s `GL_State` call), `active` -> `active: bool`
/// (`R_CopyStage`'s whole-struct copy, read by the still-deferred
/// `R_CreateBlendedStage`). The remaining `shaderStage_t` fields (`bundle[1]`,
/// `rgbGen`/`rgbWave`/`alphaGen`/`alphaWave`, `constantColor`,
/// `adjustColorsForFog`, `mGLFogColorOverride`, `ss`, `glow`, `index`,
/// `lightmapStyle`, `isDetail`) have no reader yet and land with the wave
/// that adds one.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:394-427`
#[derive(Clone)]
pub struct ShaderStage {
    /// `bundle[0].image` — Raven: the first texture bundle's bound image.
    /// `image = &shader->stages[0].bundle[0].image[0]` in `RB_RotatePic`/
    /// `RB_RotatePic2` is a plain re-fetch of this field: indexing a pointer
    /// field with `[0]` is `*image`, so `&image[0]` cancels back to `image`
    /// itself without a dereference — a real nullable pointer, mapped
    /// directly to `None` here.
    pub image: Option<ImageHandle>,
    /// `stateBits` — Raven: `GLS_xxxx` mask.
    pub state_bits: u32,
    /// `active`.
    pub active: bool,
}
