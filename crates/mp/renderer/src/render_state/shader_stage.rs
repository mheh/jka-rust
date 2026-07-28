//! `ShaderStage` — `ShaderAsset::stages`' element.

use crate::render_state::image_asset::ImageHandle;
use crate::tr_local::alpha_gen_t::alphaGen_t;
use crate::tr_local::color_gen_t::colorGen_t;
use crate::tr_local::surface_sprite_s::surfaceSprite_t;
use crate::tr_local::wave_form_t::waveForm_t;

/// The owned form of Raven `shaderStage_t` — `ShaderAsset::stages`' element
/// (`R2-D3`), landed field-by-field as call sites need them, same precedent
/// as `ShaderAsset` itself (`shader_asset.rs`'s own doc comment). Real fields
/// so far: `bundle[0].image` -> `image: Option<ImageHandle>` (`RB_RotatePic`/
/// `RB_RotatePic2`'s `if (image)` guard), `stateBits` -> `state_bits: u32`
/// (`RB_RotatePic2`'s `GL_State` call), `active` -> `active: bool`
/// (`R_CopyStage`'s whole-struct copy, read by the still-deferred
/// `R_CreateBlendedStage`), plus the color/alpha generators and the
/// surface-sprite block added by campaign #41 batch 1. The remaining
/// `shaderStage_t` fields (`bundle[1]`, `constantColor`,
/// `adjustColorsForFog`, `mGLFogColorOverride`, `glow`, `index`,
/// `lightmapStyle`, `isDetail`) have no reader yet and land with the wave
/// that adds one.
///
/// The generator fields use the tier-2 `tr_local` enums directly; the
/// parse-side mirror (`tr_shader::ShaderStageParse`) keeps its own
/// file-local `ColorGen`/`AlphaGen`/`WaveForm`/`SurfaceSpriteParse` copies,
/// so the parse -> registered per-field transcription
/// (`GeneratePermanentShader`, still DEFERRED) converts between the two.
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
    /// `rgbWave`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:409`
    pub rgb_wave: waveForm_t,
    /// `rgbGen`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:410`
    pub rgb_gen: colorGen_t,
    /// `alphaWave`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:412`
    pub alpha_wave: waveForm_t,
    /// `alphaGen`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:413`
    pub alpha_gen: alphaGen_t,
    /// `ss` (`surfaceSprite_t *`, `Hunk_Alloc`'d in the oracle) — owned
    /// inline, `None` when the stage declared no `surfaceSprites` keyword.
    /// `surfaceSprite_t` is pointer-free (all `float`/`int`/`vec2_t`), so the
    /// interior-safety law admits it by value; boxed because it is ~72 bytes
    /// on a stage that is usually spriteless.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:423`
    pub ss: Option<Box<surfaceSprite_t>>,
}
