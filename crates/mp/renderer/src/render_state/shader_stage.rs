//! `ShaderStage` — `ShaderAsset::stages`' element.

use crate::render_state::texture_bundle::TextureBundle;
use crate::tr_local::acff_t::acff_t;
use crate::tr_local::alpha_gen_t::alphaGen_t;
use crate::tr_local::color_gen_t::colorGen_t;
use crate::tr_local::eglfog_override::EGLFogOverride;
use crate::tr_local::surface_sprite_s::surfaceSprite_t;
use crate::tr_local::wave_form_t::waveForm_t;
use crate::tr_shader::NUM_TEXTURE_BUNDLES;

/// The owned form of Raven `shaderStage_t` — `ShaderAsset::stages`' element
/// (`R2-D3`). Complete as of the R4a-prep wave that wired
/// `GeneratePermanentShader`'s per-stage copy loop: every `shaderStage_t`
/// member is present, in the oracle's declaration order. The `bundle` array
/// replaced the earlier single `image: Option<ImageHandle>` shortcut, whose
/// only two readers (`RB_RotatePic`/`RB_RotatePic2`) now re-fetch
/// `stages[0].bundle[0].image` exactly as the oracle spells it.
///
/// The `_XBOX`/`VV_LIGHTING` members (`isEnvironment`, `isSpecular`,
/// `isBumpMap`) are omitted: neither macro is defined in the JKA build this
/// port targets.
///
/// The generator fields use the tier-2 `tr_local` enums directly; the
/// parse-side mirror (`tr_shader::ShaderStageParse`) keeps its own
/// file-local `ColorGen`/`AlphaGen`/`WaveForm`/`TexCoordGen`/
/// `AdjustColorsForFog`/`FogColorOverride`/`TexModInfo`/`SurfaceSpriteParse`
/// copies, so the parse -> registered per-field transcription
/// (`GeneratePermanentShader`) converts between the two through the `From`
/// impls at `tr_shader.rs` file scope.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:394-427`
#[derive(Clone)]
pub struct ShaderStage {
    /// `active`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:395`
    pub active: bool,
    /// `isDetail`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:396`
    pub is_detail: bool,
    /// `index` — Raven: index of stage. Raven's field is a `byte`; widened to
    /// `i32` to match the parse mirror (`ShaderStageParse::index`), whose own
    /// note records the widening rationale — layout-free interior, bounded by
    /// `MAX_SHADER_STAGES`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:404`
    pub index: i32,
    /// `lightmapStyle` — stays `u8`, matching both Raven's `byte` and the
    /// parse mirror (`ShaderStageParse::lightmap_style`).
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:405`
    pub lightmap_style: u8,
    /// `bundle[NUM_TEXTURE_BUNDLES]`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:407`
    pub bundle: [TextureBundle; NUM_TEXTURE_BUNDLES],
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
    /// `constantColor[4]` — Raven: for `CGEN_CONST` and `AGEN_CONST`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:415`
    pub constant_color: [u8; 4],
    /// `stateBits` — Raven: `GLS_xxxx` mask.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:417`
    pub state_bits: u32,
    /// `adjustColorsForFog`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:419`
    pub adjust_colors_for_fog: acff_t,
    /// `mGLFogColorOverride`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:421`
    pub gl_fog_color_override: EGLFogOverride,
    /// `ss` (`surfaceSprite_t *`, `Hunk_Alloc`'d in the oracle) — owned
    /// inline, `None` when the stage declared no `surfaceSprites` keyword.
    /// `surfaceSprite_t` is pointer-free (all `float`/`int`/`vec2_t`), so the
    /// interior-safety law admits it by value; boxed because it is ~72 bytes
    /// on a stage that is usually spriteless.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:423`
    pub ss: Option<Box<surfaceSprite_t>>,
    /// `glow` — Raven: whether this object emits a glow or not.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:426`
    pub glow: bool,
}
