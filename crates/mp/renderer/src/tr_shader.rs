//! Raven `tr_shader.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_shader.cpp`

// Raven's own dead stores are transcribed as written (porting-rules §A2/§C10).
#![allow(unused_assignments)]
// Raven-named functions/types keep their original casing across this
// transcription, matching the rest of the renderer/engine crates.
#![allow(non_snake_case)]

use core::array;
use core::f64::consts::PI;
use core::mem;

use mp_engine_qcommon::cm::cm_shader_consts::MAX_SHADER_FILES;
use mp_engine_qcommon::cmd_common::Cmd_Argc;
use mp_engine_qcommon::common::common::{com_printf, Common};
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::error::com_error;
use mp_engine_qcommon::common_fns::Com_DPrintf;
use mp_engine_qcommon::files_common::{FS_ListFiles, FS_ReadFileVec};
use mp_engine_qcommon::qfiles::draw_vert_t::MAXLIGHTMAPS;
use mp_engine_qcommon::qfiles::light_style_limits::{LS_LSNONE, LS_NORMAL, LS_UNUSED};
use mp_qshared::shared::com_parse::{
    COM_ParseExt, QSharedScratch, SkipBracedSection, SkipRestOfLine,
};
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::q_color::{S_COLOR_RED, S_COLOR_YELLOW};
use mp_qshared::shared::q_string::COM_StripExtension;
use mp_qshared::shared::surface_flags::{
    CONTENTS_ABSEIL, CONTENTS_BOTCLIP, CONTENTS_DETAIL, CONTENTS_FOG, CONTENTS_INSIDE,
    CONTENTS_LADDER, CONTENTS_LAVA, CONTENTS_MONSTERCLIP, CONTENTS_NODROP, CONTENTS_OPAQUE,
    CONTENTS_OUTSIDE, CONTENTS_PLAYERCLIP, CONTENTS_SHOTCLIP, CONTENTS_SLIME, CONTENTS_SOLID,
    CONTENTS_TERRAIN, CONTENTS_TRANSLUCENT, CONTENTS_TRIGGER, CONTENTS_WATER, MATERIALS,
    SURF_FORCEFIELD, SURF_METALSTEPS, SURF_NODAMAGE, SURF_NODLIGHT, SURF_NODRAW, SURF_NOIMPACT,
    SURF_NOMARKS, SURF_NOMISCENTS, SURF_NOSTEPS, SURF_SKY, SURF_SLICK,
};
use mp_qshared::shared::MAX_QPATH;
use native_string::atof::atof;
use native_string::Q_stricmpn;

use crate::gl_constants::{GL_CLAMP, GL_REPEAT};
use crate::render_state::world_load_state::WorldLoadState;
use crate::render_state::image_asset::ImageHandle;
use crate::render_state::placeholders::SkyParms;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::render_state::shader_asset::{ShaderAsset, ShaderHandle};
use crate::render_state::shader_stage::ShaderStage;
use crate::render_state::texture_bundle::TextureBundle;
use crate::render_state::walk_warnings::WalkWarnings;
use crate::tr_image::{R_FindImageFile, TrImageState};
use crate::tr_local::acff_t::acff_t;
use crate::tr_local::alpha_gen_t::alphaGen_t;
use crate::tr_local::color_gen_t::colorGen_t;
use crate::tr_local::eglfog_override::EGLFogOverride;
use crate::tr_local::gen_func_t::genFunc_t;
use crate::tr_local::shader_sort_t::shaderSort_t;
use crate::tr_local::surface_sprite_s::surfaceSprite_t;
use crate::tr_local::tex_coord_gen_t::texCoordGen_t;
use crate::tr_local::tex_mod_info_t::texModInfo_t;
use crate::tr_local::tex_mod_t::texMod_t;
use crate::tr_local::view_parms_t::viewParms_t;
use crate::tr_local::wave_form_t::waveForm_t;
use crate::tr_model::render_models::RenderModels;
use crate::tr_sky::R_InitSkyTexCoords;

// PORT-NOTE: this wave is `tr_shader`'s first (wave 0) — the R3 wave the
// tier-2 transition audit assigns `ShaderAsset`'s fields to
// (`docs/subsystems/renderer-r2-design.md` Group 2 row). Following the
// established multi-wave-fills-one-struct pattern (`tr_image.rs`/`tr_init.rs`
// against `GlConfig`/`FunctionTables`, both still empty placeholder structs
// at the time those files landed), this file assumes the following fields —
// snake_case of the oracle `shader_t`/`trGlobals_t` names — without defining
// them itself (`shader_asset.rs`/`render_assets.rs` are outside this file's
// APPEND scope; a later wave or the integrate pass adds them):
//   `ShaderAsset`: `name: String`, `lightmap_index: [i32; MAXLIGHTMAPS]`,
//   `styles: [u8; MAXLIGHTMAPS]`, `sort: f32`, `sorted_index: i32`,
//   `surface_flags: i32`, `content_flags: i32`, `multitexture_env: i32`,
//   `default_shader: bool`, `explicitly_defined: bool`,
//   `num_unfogged_passes: i32`, `sky: Option<_>` (existence-only — the tier-2
//   audit's `Option<SkyParms>` shape, never constructed by this wave).
//   `RenderAssets`: `sorted_shaders: Vec<ShaderHandle>` — the owned form of
//   `tr.sortedShaders[MAX_SHADERS]`, maintained by `SortNewShader` and walked
//   by `R_ShaderList_f`'s `Cmd_Argc() > 1` branch.

/// Raven `MAX_SHADER_STAGES`.
///
/// Source: `oracle/codemp/renderer/tr_local.h:190`
pub const MAX_SHADER_STAGES: usize = 8;

/// Raven `NUM_TEXTURE_BUNDLES`.
///
/// Source: `oracle/codemp/renderer/tr_local.h:107`
pub const NUM_TEXTURE_BUNDLES: usize = 2;

/// Raven `MAX_IMAGE_ANIMATIONS` — a fn-scope `#define` inside `ParseStage`'s
/// `animMap` branch (in-packet, `## FILE-SCOPE CONSTANTS`).
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:1402`
pub const MAX_IMAGE_ANIMATIONS: usize = 32;

/// Raven `MAX_SHADER_DEFORMS`. Not itself in this wave's packet, but already
/// ported (privately) as `tr_local::shader_s::MAX_SHADER_DEFORMS` — the value
/// is re-declared `pub`-visible here rather than re-guessed (that constant
/// isn't exported from its file).
///
/// Source: `oracle/codemp/renderer/tr_local.h:309`
const MAX_SHADER_DEFORMS: usize = 3;

// GLS_* state-bit `#define`s this file's `NameTo*` functions return
// (translation dictionary point 8: `#define` -> `const`), plus the field
// masks and depth-mask/default bits `FinishShader` tests.
// Source: `oracle/codemp/renderer/tr_local.h:1648-1682`
pub const GLS_SRCBLEND_ZERO: i32 = 0x0000_0001;
pub const GLS_SRCBLEND_ONE: i32 = 0x0000_0002;
pub const GLS_SRCBLEND_DST_COLOR: i32 = 0x0000_0003;
pub const GLS_SRCBLEND_ONE_MINUS_DST_COLOR: i32 = 0x0000_0004;
pub const GLS_SRCBLEND_SRC_ALPHA: i32 = 0x0000_0005;
pub const GLS_SRCBLEND_ONE_MINUS_SRC_ALPHA: i32 = 0x0000_0006;
pub const GLS_SRCBLEND_DST_ALPHA: i32 = 0x0000_0007;
pub const GLS_SRCBLEND_ONE_MINUS_DST_ALPHA: i32 = 0x0000_0008;
pub const GLS_SRCBLEND_ALPHA_SATURATE: i32 = 0x0000_0009;
/// Raven `GLS_SRCBLEND_BITS` — source-blend field mask.
/// Source: `oracle/codemp/renderer/tr_local.h:1657`
pub const GLS_SRCBLEND_BITS: i32 = 0x0000_000f;
pub const GLS_DSTBLEND_ZERO: i32 = 0x0000_0010;
pub const GLS_DSTBLEND_ONE: i32 = 0x0000_0020;
pub const GLS_DSTBLEND_SRC_COLOR: i32 = 0x0000_0030;
pub const GLS_DSTBLEND_ONE_MINUS_SRC_COLOR: i32 = 0x0000_0040;
pub const GLS_DSTBLEND_SRC_ALPHA: i32 = 0x0000_0050;
pub const GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA: i32 = 0x0000_0060;
pub const GLS_DSTBLEND_DST_ALPHA: i32 = 0x0000_0070;
pub const GLS_DSTBLEND_ONE_MINUS_DST_ALPHA: i32 = 0x0000_0080;
/// Raven `GLS_DSTBLEND_BITS` — destination-blend field mask.
/// Source: `oracle/codemp/renderer/tr_local.h:1667`
pub const GLS_DSTBLEND_BITS: i32 = 0x0000_00f0;
/// Raven `GLS_DEPTHMASK_TRUE`.
/// Source: `oracle/codemp/renderer/tr_local.h:1669`
pub const GLS_DEPTHMASK_TRUE: i32 = 0x0000_0100;
/// Raven `GLS_DEPTHTEST_DISABLE`.
/// Source: `oracle/codemp/renderer/tr_local.h:1673`
pub const GLS_DEPTHTEST_DISABLE: i32 = 0x0001_0000;
/// Raven `GLS_DEPTHFUNC_EQUAL`.
/// Source: `oracle/codemp/renderer/tr_local.h:1674`
pub const GLS_DEPTHFUNC_EQUAL: i32 = 0x0002_0000;
/// Raven `GLS_DEFAULT` — `#define GLS_DEFAULT GLS_DEPTHMASK_TRUE`.
/// Source: `oracle/codemp/renderer/tr_local.h:1682`
pub const GLS_DEFAULT: i32 = GLS_DEPTHMASK_TRUE;
pub const GLS_ATEST_GT_0: u32 = 0x1000_0000;
pub const GLS_ATEST_LT_80: u32 = 0x2000_0000;
pub const GLS_ATEST_GE_80: u32 = 0x4000_0000;
pub const GLS_ATEST_GE_C0: u32 = 0x8000_0000;

// Standard OpenGL 1.x texture-env-mode enum values `R_ShaderList_f` prints —
// not oracle-specific `#define`s (they come from the GL headers, not the
// oracle tree).
pub const GL_ADD: i32 = 0x0104;
pub const GL_MODULATE: i32 = 0x2100;
pub const GL_DECAL: i32 = 0x2101;

// Raven's lightmap-index sentinels.
// Source: `oracle/codemp/renderer/tr_local.h:431-434`
/// Raven: shader is for 2D rendering.
pub const LIGHTMAP_2D: i32 = -4;
pub const LIGHTMAP_BY_VERTEX: i32 = -3;
pub const LIGHTMAP_WHITEIMAGE: i32 = -2;
pub const LIGHTMAP_NONE: i32 = -1;

// Raven's four file-scope lightmap-index tables and the default style table.
// They are compared by *address* in `R_CreateExtendedName` (`:187-202`) and
// passed by pointer everywhere else; the address-identity role is carried by
// `LightmapNameMode` instead (see that enum), so these keep only their value
// role. Raven's lowerCamelCase names are preserved, hence the per-item
// `non_upper_case_globals` allow — the same casing-fidelity choice the
// file-level `non_snake_case` allow makes for functions.
// Source: `oracle/codemp/renderer/tr_shader.cpp:125-163`

/// Raven `const int lightmapsNone[MAXLIGHTMAPS]`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:125-131`
#[allow(non_upper_case_globals)]
pub const lightmapsNone: [i32; MAXLIGHTMAPS] = [LIGHTMAP_NONE; MAXLIGHTMAPS];

/// Raven `const int lightmaps2d[MAXLIGHTMAPS]`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:133-139`
#[allow(non_upper_case_globals)]
pub const lightmaps2d: [i32; MAXLIGHTMAPS] = [LIGHTMAP_2D; MAXLIGHTMAPS];

/// Raven `const int lightmapsVertex[MAXLIGHTMAPS]`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:141-147`
#[allow(non_upper_case_globals)]
pub const lightmapsVertex: [i32; MAXLIGHTMAPS] = [LIGHTMAP_BY_VERTEX; MAXLIGHTMAPS];

/// Raven `const int lightmapsFullBright[MAXLIGHTMAPS]`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:149-155`
#[allow(non_upper_case_globals)]
pub const lightmapsFullBright: [i32; MAXLIGHTMAPS] = [LIGHTMAP_WHITEIMAGE; MAXLIGHTMAPS];

/// Raven `const byte stylesDefault[MAXLIGHTMAPS]`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:157-163`
#[allow(non_upper_case_globals)]
pub const stylesDefault: [u8; MAXLIGHTMAPS] = [LS_NORMAL, LS_LSNONE, LS_LSNONE, LS_LSNONE];

// Raven's surface-sprite type/facing tags.
// Source: `oracle/codemp/renderer/tr_local.h:351-360`
pub const SURFSPRITE_NONE: i32 = 0;
pub const SURFSPRITE_VERTICAL: i32 = 1;
pub const SURFSPRITE_ORIENTED: i32 = 2;
pub const SURFSPRITE_EFFECT: i32 = 3;
pub const SURFSPRITE_WEATHERFX: i32 = 4;
pub const SURFSPRITE_FACING_NORMAL: i32 = 0;
pub const SURFSPRITE_FACING_UP: i32 = 1;
pub const SURFSPRITE_FACING_DOWN: i32 = 2;
pub const SURFSPRITE_FACING_ANY: i32 = 3;

/// Bucket count for `RenderAssets::shader_text_hash_table`. Capacity choice
/// only — see `shader_text_hash_bucket`'s note on why exact parity with the
/// oracle's `MAX_SHADERTEXT_HASH` doesn't matter here.
const MAX_SHADERTEXT_HASH: usize = 4096;

/// Raven `colorGen_t`, reproduced locally (rather than importing
/// `tr_local::color_gen_t::colorGen_t`) so `ShaderStageParse` can derive
/// `Clone` — the tier-2 enum carries no derives and this crate cannot add any
/// (out of this file's APPEND scope). Same reasoning for every other small
/// enum below.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:242-257`
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorGen {
    Bad,
    IdentityLighting,
    Identity,
    Entity,
    OneMinusEntity,
    ExactVertex,
    Vertex,
    OneMinusVertex,
    Waveform,
    LightingDiffuse,
    LightingDiffuseEntity,
    Fog,
    Const,
    LightmapStyle,
}
impl Default for ColorGen {
    fn default() -> ColorGen {
        ColorGen::Bad
    }
}

/// Raven `alphaGen_t`.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:226-240`
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AlphaGen {
    Identity,
    Skip,
    Entity,
    OneMinusEntity,
    Vertex,
    OneMinusVertex,
    LightingSpecular,
    Waveform,
    Portal,
    Blend,
    Const,
    Dot,
    OneMinusDot,
}
impl Default for AlphaGen {
    fn default() -> AlphaGen {
        AlphaGen::Identity
    }
}

/// Raven `genFunc_t`.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:192-204`
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GenFunc {
    None,
    Sin,
    Square,
    Triangle,
    Sawtooth,
    InverseSawtooth,
    Noise,
    Rand,
}
impl Default for GenFunc {
    fn default() -> GenFunc {
        GenFunc::None
    }
}

/// Raven `texCoordGen_t`.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:259-270`
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TexCoordGen {
    Bad,
    Identity,
    Lightmap,
    Lightmap1,
    Lightmap2,
    Lightmap3,
    Texture,
    EnvironmentMapped,
    Fog,
    Vector,
}
impl Default for TexCoordGen {
    fn default() -> TexCoordGen {
        TexCoordGen::Bad
    }
}

/// Raven `EGLFogOverride`.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:279-285`
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FogColorOverride {
    None,
    Black,
    White,
    Max,
}
impl Default for FogColorOverride {
    fn default() -> FogColorOverride {
        FogColorOverride::None
    }
}

/// Raven `acff_t` (alpha combine function format), reproduced locally — same
/// rationale as `ColorGen`/`GenFunc` above (needs `Clone`/`Copy`, out of scope
/// for the tier-2 `acff_t` file this wave may not touch).
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:272-277`
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AdjustColorsForFog {
    None,
    ModulateRgb,
    ModulateRgba,
    ModulateAlpha,
}
impl Default for AdjustColorsForFog {
    fn default() -> AdjustColorsForFog {
        AdjustColorsForFog::None
    }
}

/// Raven `deform_t`, reproduced locally — same rationale as `ColorGen`/
/// `GenFunc` above (needs `Clone`/`Copy`, out of scope for the tier-2
/// `deform_t` file this wave may not touch).
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:207-224`
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Deform {
    None,
    Wave,
    Normals,
    Bulge,
    Move,
    ProjectionShadow,
    Autosprite,
    Autosprite2,
    Text0,
    Text1,
    Text2,
    Text3,
    Text4,
    Text5,
    Text6,
    Text7,
}
impl Default for Deform {
    fn default() -> Deform {
        Deform::None
    }
}

/// Raven `waveForm_t`.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:287-294`
#[derive(Clone, Copy, Default, PartialEq)]
pub struct WaveForm {
    pub func: GenFunc,
    pub base: f32,
    pub amplitude: f32,
    pub phase: f32,
    pub frequency: f32,
}

/// Raven `texModInfo_t`. `r#type` (Raven `texMod_t`) stays a plain `i32` tag —
/// never touched by this wave's functions (`tex_mods` is always empty here).
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:323-348`
#[derive(Clone, Copy, Default)]
pub struct TexModInfo {
    pub kind: i32,
    pub wave: WaveForm,
    pub matrix: [[f32; 2]; 2],
    pub translate: [f32; 2],
}

/// Owned form of Raven `deformStage_t` (tier-2 transition audit, Group 2
/// `shader_t::deforms` row): `moveVector` (`vec3_t`) -> owned `[f32; 3]`,
/// `deformationWave` -> the local `WaveForm` (not the tier-2 `waveForm_t`, so
/// this type can derive `Clone`/`Copy`, same rationale as `ShaderStageParse`).
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:310-320`
#[derive(Clone, Copy, Default)]
pub struct DeformStage {
    pub deformation: Deform,
    pub move_vector: [f32; 3],
    pub deformation_wave: WaveForm,
    pub deformation_spread: f32,
    pub bulge_width: f32,
    pub bulge_height: f32,
    pub bulge_speed: f32,
}

/// Owned form of Raven `textureBundle_t` (tier-2 transition audit, Group 2
/// row): `image` -> `Option<ImageHandle>`, `tcGenVectors` -> owned
/// `[[f32; 3]; 2]`, `texMods` -> owned `Vec<TexModInfo>`.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:372-389`
#[derive(Clone, Default)]
pub struct TextureBundleParse {
    pub image: Option<ImageHandle>,
    pub tc_gen: TexCoordGen,
    pub tc_gen_vectors: [[f32; 3]; 2],
    pub tex_mods: Vec<TexModInfo>,
    pub num_image_animations: i16,
    pub image_animation_speed: f32,
    pub is_lightmap: bool,
    pub one_shot_anim_map: bool,
    pub vertex_lightmap: bool,
    pub is_video_map: bool,
    pub video_map_handle: i32,
    /// `image[MAX_IMAGE_ANIMATIONS]` — added by wave 5 (`ParseStage`'s
    /// `animMap`/`clampanimMap`/`oneshotanimMap` frame list). The oracle
    /// builds a local `image_t *images[MAX_IMAGE_ANIMATIONS]` array and
    /// copies it into `bundle[0].image` via `Hunk_Alloc`+`memcpy`; the owned
    /// `Vec` here IS that copy, no allocation step needed (§C9). `image`
    /// above stays the single-frame ("map"/"clampmap") slot — the two never
    /// populate simultaneously in the oracle either (`bundle[0].image` is
    /// reused/overloaded between the single-image and animated-array cases).
    ///
    /// Source: `oracle/codemp/renderer/tr_shader.cpp:1400-1443`
    pub image_animations: Vec<ImageHandle>,
}

/// Owned form of Raven `surfaceSprite_t`.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:363-370`
#[derive(Clone, Default)]
pub struct SurfaceSpriteParse {
    pub surface_sprite_type: i32,
    pub width: f32,
    pub height: f32,
    pub density: f32,
    pub wind: f32,
    pub wind_idle: f32,
    pub fade_dist: f32,
    pub fade_max: f32,
    pub fade_scale: f32,
    pub fx_alpha_start: f32,
    pub fx_alpha_end: f32,
    pub fx_duration: f32,
    pub vert_skew: f32,
    pub variance: [f32; 2],
    pub fx_grow: [f32; 2],
    pub facing: i32,
}

/// One shader stage under construction — the owned form of `shaderStage_t`
/// used only inside `ShaderParseState` (tier-2 transition audit, Group 2:
/// `ss` -> `Option<Box<SurfaceSpriteParse>>`, `bundle` -> owned
/// `TextureBundleParse`).
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:394-427`
#[derive(Clone, Default)]
pub struct ShaderStageParse {
    pub active: bool,
    pub state_bits: u32,
    pub rgb_gen: ColorGen,
    pub rgb_wave: WaveForm,
    pub alpha_gen: AlphaGen,
    pub alpha_wave: WaveForm,
    pub bundle: [TextureBundleParse; NUM_TEXTURE_BUNDLES],
    pub gl_fog_color_override: FogColorOverride,
    pub ss: Option<Box<SurfaceSpriteParse>>,
    /// `isDetail` — added by wave 2 (`FinishShader`'s detail-texture strip).
    pub is_detail: bool,
    /// `index` — added by wave 2 (`FinishShader`'s `stageIndex` stamp).
    /// Raven's field is a `u8`; widened to `i32` (layout-free interior,
    /// bounded by `MAX_SHADER_STAGES`, same widening rationale as
    /// `ShaderAsset::num_unfogged_passes`).
    pub index: i32,
    /// `lightmapStyle` — added by wave 2 (`FinishShader`'s multi-lightmap
    /// style loop).
    pub lightmap_style: u8,
    /// `adjustColorsForFog` — added by wave 2; computed by `FinishShader`'s
    /// "determine sort order and fog color adjustment" block.
    pub adjust_colors_for_fog: AdjustColorsForFog,
    /// `constantColor[4]` — added by wave 5 (`ParseStage`'s `rgbGen const`/
    /// `alphaGen const`); write sites `oracle/codemp/renderer/
    /// tr_shader.cpp:1605-1607,1688`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:415`
    pub constant_color: [u8; 4],
    /// `glow` — added by wave 5 (`ParseStage`'s `glow` keyword,
    /// `oracle/codemp/renderer/tr_shader.cpp:1813-1819`).
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:426`
    pub glow: bool,
}

// ---------------------------------------------------------------------------
// Parse-scratch <-> registered stage conversions.
//
// The oracle's `newShader->stages[i] = stages[i]`
// (`oracle/codemp/renderer/tr_shader.cpp:2789`) and
// `R_CopyStage(work->stages, stages + idx)` (`:4017`) are plain
// `shaderStage_t` struct assignments in both directions. This port splits
// that one C type in two — the parse-scratch `ShaderStageParse` (file-local
// `Clone`/`Copy` enum copies, so `ShaderParseState` can derive `Clone`) and
// the registered `ShaderStage` (tier-2 `tr_local` enums) — so each assignment
// becomes the field-for-field `From` impls below. They carry no behavior of
// their own: every arm is the identity mapping between two transcriptions of
// the same Raven enumerator list.
// ---------------------------------------------------------------------------

/// `ColorGen` -> `colorGen_t` (`oracle/codemp/renderer/tr_local.h:242-257`).
impl From<ColorGen> for colorGen_t {
    fn from(v: ColorGen) -> colorGen_t {
        match v {
            ColorGen::Bad => colorGen_t::CGEN_BAD,
            ColorGen::IdentityLighting => colorGen_t::CGEN_IDENTITY_LIGHTING,
            ColorGen::Identity => colorGen_t::CGEN_IDENTITY,
            ColorGen::Entity => colorGen_t::CGEN_ENTITY,
            ColorGen::OneMinusEntity => colorGen_t::CGEN_ONE_MINUS_ENTITY,
            ColorGen::ExactVertex => colorGen_t::CGEN_EXACT_VERTEX,
            ColorGen::Vertex => colorGen_t::CGEN_VERTEX,
            ColorGen::OneMinusVertex => colorGen_t::CGEN_ONE_MINUS_VERTEX,
            ColorGen::Waveform => colorGen_t::CGEN_WAVEFORM,
            ColorGen::LightingDiffuse => colorGen_t::CGEN_LIGHTING_DIFFUSE,
            ColorGen::LightingDiffuseEntity => colorGen_t::CGEN_LIGHTING_DIFFUSE_ENTITY,
            ColorGen::Fog => colorGen_t::CGEN_FOG,
            ColorGen::Const => colorGen_t::CGEN_CONST,
            ColorGen::LightmapStyle => colorGen_t::CGEN_LIGHTMAPSTYLE,
        }
    }
}

/// `colorGen_t` -> `ColorGen` (`oracle/codemp/renderer/tr_local.h:242-257`).
impl From<colorGen_t> for ColorGen {
    fn from(v: colorGen_t) -> ColorGen {
        match v {
            colorGen_t::CGEN_BAD => ColorGen::Bad,
            colorGen_t::CGEN_IDENTITY_LIGHTING => ColorGen::IdentityLighting,
            colorGen_t::CGEN_IDENTITY => ColorGen::Identity,
            colorGen_t::CGEN_ENTITY => ColorGen::Entity,
            colorGen_t::CGEN_ONE_MINUS_ENTITY => ColorGen::OneMinusEntity,
            colorGen_t::CGEN_EXACT_VERTEX => ColorGen::ExactVertex,
            colorGen_t::CGEN_VERTEX => ColorGen::Vertex,
            colorGen_t::CGEN_ONE_MINUS_VERTEX => ColorGen::OneMinusVertex,
            colorGen_t::CGEN_WAVEFORM => ColorGen::Waveform,
            colorGen_t::CGEN_LIGHTING_DIFFUSE => ColorGen::LightingDiffuse,
            colorGen_t::CGEN_LIGHTING_DIFFUSE_ENTITY => ColorGen::LightingDiffuseEntity,
            colorGen_t::CGEN_FOG => ColorGen::Fog,
            colorGen_t::CGEN_CONST => ColorGen::Const,
            colorGen_t::CGEN_LIGHTMAPSTYLE => ColorGen::LightmapStyle,
        }
    }
}

/// `AlphaGen` -> `alphaGen_t` (`oracle/codemp/renderer/tr_local.h:226-240`).
impl From<AlphaGen> for alphaGen_t {
    fn from(v: AlphaGen) -> alphaGen_t {
        match v {
            AlphaGen::Identity => alphaGen_t::AGEN_IDENTITY,
            AlphaGen::Skip => alphaGen_t::AGEN_SKIP,
            AlphaGen::Entity => alphaGen_t::AGEN_ENTITY,
            AlphaGen::OneMinusEntity => alphaGen_t::AGEN_ONE_MINUS_ENTITY,
            AlphaGen::Vertex => alphaGen_t::AGEN_VERTEX,
            AlphaGen::OneMinusVertex => alphaGen_t::AGEN_ONE_MINUS_VERTEX,
            AlphaGen::LightingSpecular => alphaGen_t::AGEN_LIGHTING_SPECULAR,
            AlphaGen::Waveform => alphaGen_t::AGEN_WAVEFORM,
            AlphaGen::Portal => alphaGen_t::AGEN_PORTAL,
            AlphaGen::Blend => alphaGen_t::AGEN_BLEND,
            AlphaGen::Const => alphaGen_t::AGEN_CONST,
            AlphaGen::Dot => alphaGen_t::AGEN_DOT,
            AlphaGen::OneMinusDot => alphaGen_t::AGEN_ONE_MINUS_DOT,
        }
    }
}

/// `alphaGen_t` -> `AlphaGen` (`oracle/codemp/renderer/tr_local.h:226-240`).
impl From<alphaGen_t> for AlphaGen {
    fn from(v: alphaGen_t) -> AlphaGen {
        match v {
            alphaGen_t::AGEN_IDENTITY => AlphaGen::Identity,
            alphaGen_t::AGEN_SKIP => AlphaGen::Skip,
            alphaGen_t::AGEN_ENTITY => AlphaGen::Entity,
            alphaGen_t::AGEN_ONE_MINUS_ENTITY => AlphaGen::OneMinusEntity,
            alphaGen_t::AGEN_VERTEX => AlphaGen::Vertex,
            alphaGen_t::AGEN_ONE_MINUS_VERTEX => AlphaGen::OneMinusVertex,
            alphaGen_t::AGEN_LIGHTING_SPECULAR => AlphaGen::LightingSpecular,
            alphaGen_t::AGEN_WAVEFORM => AlphaGen::Waveform,
            alphaGen_t::AGEN_PORTAL => AlphaGen::Portal,
            alphaGen_t::AGEN_BLEND => AlphaGen::Blend,
            alphaGen_t::AGEN_CONST => AlphaGen::Const,
            alphaGen_t::AGEN_DOT => AlphaGen::Dot,
            alphaGen_t::AGEN_ONE_MINUS_DOT => AlphaGen::OneMinusDot,
        }
    }
}

/// `GenFunc` -> `genFunc_t` (`oracle/codemp/renderer/tr_local.h:192-204`).
impl From<GenFunc> for genFunc_t {
    fn from(v: GenFunc) -> genFunc_t {
        match v {
            GenFunc::None => genFunc_t::GF_NONE,
            GenFunc::Sin => genFunc_t::GF_SIN,
            GenFunc::Square => genFunc_t::GF_SQUARE,
            GenFunc::Triangle => genFunc_t::GF_TRIANGLE,
            GenFunc::Sawtooth => genFunc_t::GF_SAWTOOTH,
            GenFunc::InverseSawtooth => genFunc_t::GF_INVERSE_SAWTOOTH,
            GenFunc::Noise => genFunc_t::GF_NOISE,
            GenFunc::Rand => genFunc_t::GF_RAND,
        }
    }
}

/// `genFunc_t` -> `GenFunc` (`oracle/codemp/renderer/tr_local.h:192-204`).
impl From<genFunc_t> for GenFunc {
    fn from(v: genFunc_t) -> GenFunc {
        match v {
            genFunc_t::GF_NONE => GenFunc::None,
            genFunc_t::GF_SIN => GenFunc::Sin,
            genFunc_t::GF_SQUARE => GenFunc::Square,
            genFunc_t::GF_TRIANGLE => GenFunc::Triangle,
            genFunc_t::GF_SAWTOOTH => GenFunc::Sawtooth,
            genFunc_t::GF_INVERSE_SAWTOOTH => GenFunc::InverseSawtooth,
            genFunc_t::GF_NOISE => GenFunc::Noise,
            genFunc_t::GF_RAND => GenFunc::Rand,
        }
    }
}

/// `TexCoordGen` -> `texCoordGen_t` (`oracle/codemp/renderer/tr_local.h:259-270`).
impl From<TexCoordGen> for texCoordGen_t {
    fn from(v: TexCoordGen) -> texCoordGen_t {
        match v {
            TexCoordGen::Bad => texCoordGen_t::TCGEN_BAD,
            TexCoordGen::Identity => texCoordGen_t::TCGEN_IDENTITY,
            TexCoordGen::Lightmap => texCoordGen_t::TCGEN_LIGHTMAP,
            TexCoordGen::Lightmap1 => texCoordGen_t::TCGEN_LIGHTMAP1,
            TexCoordGen::Lightmap2 => texCoordGen_t::TCGEN_LIGHTMAP2,
            TexCoordGen::Lightmap3 => texCoordGen_t::TCGEN_LIGHTMAP3,
            TexCoordGen::Texture => texCoordGen_t::TCGEN_TEXTURE,
            TexCoordGen::EnvironmentMapped => texCoordGen_t::TCGEN_ENVIRONMENT_MAPPED,
            TexCoordGen::Fog => texCoordGen_t::TCGEN_FOG,
            TexCoordGen::Vector => texCoordGen_t::TCGEN_VECTOR,
        }
    }
}

/// `texCoordGen_t` -> `TexCoordGen` (`oracle/codemp/renderer/tr_local.h:259-270`).
impl From<texCoordGen_t> for TexCoordGen {
    fn from(v: texCoordGen_t) -> TexCoordGen {
        match v {
            texCoordGen_t::TCGEN_BAD => TexCoordGen::Bad,
            texCoordGen_t::TCGEN_IDENTITY => TexCoordGen::Identity,
            texCoordGen_t::TCGEN_LIGHTMAP => TexCoordGen::Lightmap,
            texCoordGen_t::TCGEN_LIGHTMAP1 => TexCoordGen::Lightmap1,
            texCoordGen_t::TCGEN_LIGHTMAP2 => TexCoordGen::Lightmap2,
            texCoordGen_t::TCGEN_LIGHTMAP3 => TexCoordGen::Lightmap3,
            texCoordGen_t::TCGEN_TEXTURE => TexCoordGen::Texture,
            texCoordGen_t::TCGEN_ENVIRONMENT_MAPPED => TexCoordGen::EnvironmentMapped,
            texCoordGen_t::TCGEN_FOG => TexCoordGen::Fog,
            texCoordGen_t::TCGEN_VECTOR => TexCoordGen::Vector,
        }
    }
}

/// `AdjustColorsForFog` -> `acff_t` (`oracle/codemp/renderer/tr_local.h:272-277`).
impl From<AdjustColorsForFog> for acff_t {
    fn from(v: AdjustColorsForFog) -> acff_t {
        match v {
            AdjustColorsForFog::None => acff_t::ACFF_NONE,
            AdjustColorsForFog::ModulateRgb => acff_t::ACFF_MODULATE_RGB,
            AdjustColorsForFog::ModulateRgba => acff_t::ACFF_MODULATE_RGBA,
            AdjustColorsForFog::ModulateAlpha => acff_t::ACFF_MODULATE_ALPHA,
        }
    }
}

/// `acff_t` -> `AdjustColorsForFog` (`oracle/codemp/renderer/tr_local.h:272-277`).
impl From<acff_t> for AdjustColorsForFog {
    fn from(v: acff_t) -> AdjustColorsForFog {
        match v {
            acff_t::ACFF_NONE => AdjustColorsForFog::None,
            acff_t::ACFF_MODULATE_RGB => AdjustColorsForFog::ModulateRgb,
            acff_t::ACFF_MODULATE_RGBA => AdjustColorsForFog::ModulateRgba,
            acff_t::ACFF_MODULATE_ALPHA => AdjustColorsForFog::ModulateAlpha,
        }
    }
}

/// `FogColorOverride` -> `EGLFogOverride` (`oracle/codemp/renderer/tr_local.h:279-285`).
impl From<FogColorOverride> for EGLFogOverride {
    fn from(v: FogColorOverride) -> EGLFogOverride {
        match v {
            FogColorOverride::None => EGLFogOverride::GLFOGOVERRIDE_NONE,
            FogColorOverride::Black => EGLFogOverride::GLFOGOVERRIDE_BLACK,
            FogColorOverride::White => EGLFogOverride::GLFOGOVERRIDE_WHITE,
            FogColorOverride::Max => EGLFogOverride::GLFOGOVERRIDE_MAX,
        }
    }
}

/// `EGLFogOverride` -> `FogColorOverride` (`oracle/codemp/renderer/tr_local.h:279-285`).
impl From<EGLFogOverride> for FogColorOverride {
    fn from(v: EGLFogOverride) -> FogColorOverride {
        match v {
            EGLFogOverride::GLFOGOVERRIDE_NONE => FogColorOverride::None,
            EGLFogOverride::GLFOGOVERRIDE_BLACK => FogColorOverride::Black,
            EGLFogOverride::GLFOGOVERRIDE_WHITE => FogColorOverride::White,
            EGLFogOverride::GLFOGOVERRIDE_MAX => FogColorOverride::Max,
        }
    }
}

/// `WaveForm` -> `waveForm_t` (`oracle/codemp/renderer/tr_local.h:287-294`).
impl From<WaveForm> for waveForm_t {
    fn from(v: WaveForm) -> waveForm_t {
        waveForm_t {
            func: v.func.into(),
            base: v.base,
            amplitude: v.amplitude,
            phase: v.phase,
            frequency: v.frequency,
        }
    }
}

/// `waveForm_t` -> `WaveForm` (`oracle/codemp/renderer/tr_local.h:287-294`).
impl From<waveForm_t> for WaveForm {
    fn from(v: waveForm_t) -> WaveForm {
        WaveForm {
            func: v.func.into(),
            base: v.base,
            amplitude: v.amplitude,
            phase: v.phase,
            frequency: v.frequency,
        }
    }
}

/// `TexModInfo` -> `texModInfo_t` (`oracle/codemp/renderer/tr_local.h:323-348`).
///
/// The parse mirror keeps `kind` as a plain `i32` tag; every writer in this
/// file stores a `texMod_t::TMOD_* as i32`, so the `_` arm below is
/// unreachable and folds to Raven's own zero enumerator.
impl From<TexModInfo> for texModInfo_t {
    fn from(v: TexModInfo) -> texModInfo_t {
        let r#type = match v.kind {
            x if x == texMod_t::TMOD_TRANSFORM as i32 => texMod_t::TMOD_TRANSFORM,
            x if x == texMod_t::TMOD_TURBULENT as i32 => texMod_t::TMOD_TURBULENT,
            x if x == texMod_t::TMOD_SCROLL as i32 => texMod_t::TMOD_SCROLL,
            x if x == texMod_t::TMOD_SCALE as i32 => texMod_t::TMOD_SCALE,
            x if x == texMod_t::TMOD_STRETCH as i32 => texMod_t::TMOD_STRETCH,
            x if x == texMod_t::TMOD_ROTATE as i32 => texMod_t::TMOD_ROTATE,
            x if x == texMod_t::TMOD_ENTITY_TRANSLATE as i32 => texMod_t::TMOD_ENTITY_TRANSLATE,
            _ => texMod_t::TMOD_NONE,
        };
        texModInfo_t {
            r#type,
            wave: v.wave.into(),
            matrix: v.matrix,
            translate: v.translate,
        }
    }
}

/// `texModInfo_t` -> `TexModInfo` (`oracle/codemp/renderer/tr_local.h:323-348`).
impl From<texModInfo_t> for TexModInfo {
    fn from(v: texModInfo_t) -> TexModInfo {
        TexModInfo {
            kind: v.r#type as i32,
            wave: v.wave.into(),
            matrix: v.matrix,
            translate: v.translate,
        }
    }
}

/// `SurfaceSpriteParse` -> `surfaceSprite_t` (`oracle/codemp/renderer/tr_local.h:363-370`).
impl From<&SurfaceSpriteParse> for surfaceSprite_t {
    fn from(v: &SurfaceSpriteParse) -> surfaceSprite_t {
        surfaceSprite_t {
            surfaceSpriteType: v.surface_sprite_type,
            width: v.width,
            height: v.height,
            density: v.density,
            wind: v.wind,
            windIdle: v.wind_idle,
            fadeDist: v.fade_dist,
            fadeMax: v.fade_max,
            fadeScale: v.fade_scale,
            fxAlphaStart: v.fx_alpha_start,
            fxAlphaEnd: v.fx_alpha_end,
            fxDuration: v.fx_duration,
            vertSkew: v.vert_skew,
            variance: v.variance,
            fxGrow: v.fx_grow,
            facing: v.facing,
        }
    }
}

/// `surfaceSprite_t` -> `SurfaceSpriteParse` (`oracle/codemp/renderer/tr_local.h:363-370`).
impl From<&surfaceSprite_t> for SurfaceSpriteParse {
    fn from(v: &surfaceSprite_t) -> SurfaceSpriteParse {
        SurfaceSpriteParse {
            surface_sprite_type: v.surfaceSpriteType,
            width: v.width,
            height: v.height,
            density: v.density,
            wind: v.wind,
            wind_idle: v.windIdle,
            fade_dist: v.fadeDist,
            fade_max: v.fadeMax,
            fade_scale: v.fadeScale,
            fx_alpha_start: v.fxAlphaStart,
            fx_alpha_end: v.fxAlphaEnd,
            fx_duration: v.fxDuration,
            vert_skew: v.vertSkew,
            variance: v.variance,
            fx_grow: v.fxGrow,
            facing: v.facing,
        }
    }
}

/// `TextureBundleParse` -> `TextureBundle` (`oracle/codemp/renderer/tr_local.h:372-389`).
///
/// The `tex_mods` collect IS `GeneratePermanentShader`'s per-bundle
/// `Hunk_Alloc` + `Com_Memcpy` of the `numTexMods` block (§C9); the oracle's
/// `else { texMods = 0; }` leg is the empty-`Vec` case, needing no separate
/// arm.
/// Source: `oracle/codemp/renderer/tr_shader.cpp:2791-2802`
impl From<&TextureBundleParse> for TextureBundle {
    fn from(v: &TextureBundleParse) -> TextureBundle {
        TextureBundle {
            image: v.image,
            tc_gen: v.tc_gen.into(),
            tc_gen_vectors: v.tc_gen_vectors,
            tex_mods: v.tex_mods.iter().map(|m| (*m).into()).collect(),
            num_image_animations: v.num_image_animations,
            image_animation_speed: v.image_animation_speed,
            is_lightmap: v.is_lightmap,
            one_shot_anim_map: v.one_shot_anim_map,
            vertex_lightmap: v.vertex_lightmap,
            is_video_map: v.is_video_map,
            video_map_handle: v.video_map_handle,
            image_animations: v.image_animations.clone(),
        }
    }
}

/// `TextureBundle` -> `TextureBundleParse` (`oracle/codemp/renderer/tr_local.h:372-389`).
impl From<&TextureBundle> for TextureBundleParse {
    fn from(v: &TextureBundle) -> TextureBundleParse {
        TextureBundleParse {
            image: v.image,
            tc_gen: v.tc_gen.into(),
            tc_gen_vectors: v.tc_gen_vectors,
            tex_mods: v.tex_mods.iter().map(|m| (*m).into()).collect(),
            num_image_animations: v.num_image_animations,
            image_animation_speed: v.image_animation_speed,
            is_lightmap: v.is_lightmap,
            one_shot_anim_map: v.one_shot_anim_map,
            vertex_lightmap: v.vertex_lightmap,
            is_video_map: v.is_video_map,
            video_map_handle: v.video_map_handle,
            image_animations: v.image_animations.clone(),
        }
    }
}

/// `ShaderStageParse` -> `ShaderStage` — the field-for-field form of the
/// oracle's `newShader->stages[i] = stages[i]`.
///
/// Source: `oracle/codemp/renderer/tr_local.h:394-427`
/// (assignment site `oracle/codemp/renderer/tr_shader.cpp:2789-2802`)
impl From<&ShaderStageParse> for ShaderStage {
    fn from(v: &ShaderStageParse) -> ShaderStage {
        ShaderStage {
            active: v.active,
            is_detail: v.is_detail,
            index: v.index,
            lightmap_style: v.lightmap_style,
            bundle: array::from_fn(|b| TextureBundle::from(&v.bundle[b])),
            rgb_wave: v.rgb_wave.into(),
            rgb_gen: v.rgb_gen.into(),
            alpha_wave: v.alpha_wave.into(),
            alpha_gen: v.alpha_gen.into(),
            constant_color: v.constant_color,
            state_bits: v.state_bits,
            adjust_colors_for_fog: v.adjust_colors_for_fog.into(),
            gl_fog_color_override: v.gl_fog_color_override.into(),
            ss: v
                .ss
                .as_ref()
                .map(|s| Box::new(surfaceSprite_t::from(s.as_ref()))),
            glow: v.glow,
        }
    }
}

/// `ShaderStage` -> `ShaderStageParse` — the field-for-field form of
/// `R_CopyStage`'s registered-to-scratch direction (`work->stages` into
/// `stages + idx`).
///
/// Source: `oracle/codemp/renderer/tr_local.h:394-427`
/// (assignment site `oracle/codemp/renderer/tr_shader.cpp:4017`)
impl From<&ShaderStage> for ShaderStageParse {
    fn from(v: &ShaderStage) -> ShaderStageParse {
        ShaderStageParse {
            active: v.active,
            state_bits: v.state_bits,
            rgb_gen: v.rgb_gen.into(),
            rgb_wave: v.rgb_wave.into(),
            alpha_gen: v.alpha_gen.into(),
            alpha_wave: v.alpha_wave.into(),
            bundle: array::from_fn(|b| TextureBundleParse::from(&v.bundle[b])),
            gl_fog_color_override: v.gl_fog_color_override.into(),
            ss: v
                .ss
                .as_ref()
                .map(|s| Box::new(SurfaceSpriteParse::from(s.as_ref()))),
            is_detail: v.is_detail,
            index: v.index,
            lightmap_style: v.lightmap_style,
            adjust_colors_for_fog: v.adjust_colors_for_fog.into(),
            constant_color: v.constant_color,
            glow: v.glow,
        }
    }
}

/// Raven `cullType_t`, reproduced locally — same rationale as `ColorGen`/
/// `GenFunc` above (`ShaderParseState` needs `Clone`/`Copy`/`Default`, out of
/// scope for the tier-2 `cullType_t` file this wave may not touch).
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h` (`cullType_t`,
/// exact line outside this packet's slice — not guessed, per porting-rules
/// §A2; write site `oracle/codemp/renderer/tr_shader.cpp:2507-2530`)
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CullType {
    FrontSided,
    BackSided,
    TwoSided,
}
impl Default for CullType {
    fn default() -> CullType {
        CullType::FrontSided
    }
}

/// Raven `fogPass_t`, reproduced locally — same rationale as `CullType`
/// above. `ParseShader`'s `noglfog` keyword seeds `ShaderParseState::fog_pass`,
/// and `GeneratePermanentShader` copies it onto `ShaderAsset::fog_pass` and
/// then overrides it from the sort order and `CONTENTS_FOG` flag.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:442-447`
/// (write site `oracle/codemp/renderer/tr_shader.cpp:2444-2448`)
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FogPass {
    None,
    Equal,
    Le,
    GlFog,
}
impl Default for FogPass {
    fn default() -> FogPass {
        FogPass::None
    }
}

/// Owned form of Raven `fogParms_t` — only the two members this wave's
/// `ParseShader` touches (`color`, `depthForOpaque`); the type's full field
/// list sits outside this packet's slice (not guessed, per porting-rules
/// §A2).
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:2469-2488` (write sites)
#[derive(Clone, Copy, Default)]
pub struct FogParms {
    pub color: [f32; 3],
    pub depth_for_opaque: f32,
}

/// Per-`ParseShader`-call scratch — the oracle file-scope globals `shader`
/// (the `shader_t` under construction) and `stages[MAX_SHADER_STAGES]`,
/// alive only for the duration of one `ParseShader` call, never persisted
/// state (DEC-37 A13.4). `ClearGlobalShader` is this type's constructor —
/// Raven zeroes the globals in place at the start of every `ParseShader`
/// call; the idiomatic equivalent constructs a fresh value instead of
/// resetting a stored one. The oracle's separate `texMods[MAX_SHADER_STAGES]`
/// scratch array (aliased into `stages[i].bundle[0].texMods` by pointer)
/// dissolves entirely: `TextureBundleParse::tex_mods` is already its own
/// owned `Vec`, so there is nothing left to point at.
///
/// `default_shader`/`explicitly_defined`/`num_unfogged_passes`/`sky` mirror
/// the same-named `ShaderAsset` fields (already real, landed wave-0) — the
/// `shader_t` global's own copies of them. No wave-1 fn writes them (that is
/// `ParseShader`'s job, unported); they stay at `ClearGlobalShader`'s
/// zero-initialized default until that wave lands, exactly like every other
/// unread `shader_t` field this scratch doesn't carry yet.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp` (`shader`, `stages`,
/// `texMods`)
#[derive(Default)]
pub struct ShaderParseState {
    pub name: String,
    pub lightmap_index: [i32; MAXLIGHTMAPS],
    pub styles: [u8; MAXLIGHTMAPS],
    pub sort: f32,
    pub surface_flags: i32,
    pub content_flags: i32,
    pub multitexture_env: i32,
    pub default_shader: bool,
    pub explicitly_defined: bool,
    pub num_unfogged_passes: i32,
    pub sky: Option<SkyParms>,
    pub stages: Vec<ShaderStageParse>,
    /// `polygonOffset` — added by wave 2 (`FinishShader`'s decal-sort rule).
    pub polygon_offset: bool,
    /// `deforms[MAX_SHADER_DEFORMS]`/`numDeforms` — added by wave 2
    /// (`ParseDeform`); the owned `Vec`'s length is the count, so no separate
    /// `numDeforms` field is needed (§C9-style out-param -> the collection's
    /// own length).
    pub deforms: Vec<DeformStage>,
    /// `shader.noMipMaps` — added by wave 5 (`ParseStage`'s `map`/`clampmap`/
    /// `animMap`/sky-box image-load flag reads). Field declaration is on
    /// `shader_t` (`oracle/codemp/renderer/tr_local.h`, exact line outside
    /// this packet's slice — not guessed, per porting-rules §A2); the read
    /// sites this wave transcribes are `oracle/codemp/renderer/
    /// tr_shader.cpp:1339,1361,1389,1430,2106`.
    pub no_mip_maps: bool,
    /// `shader.noPicMip` — added by wave 5, same callers as `no_mip_maps`.
    pub no_pic_mip: bool,
    /// `shader.noTC` — added by wave 5, same callers as `no_mip_maps`.
    pub no_tc: bool,
    /// `shader.portalRange` — added by wave 5 (`ParseStage`'s `alphaGen
    /// portal`, `oracle/codemp/renderer/tr_shader.cpp:1729,1734`).
    pub portal_range: f32,
    /// `shader.hasGlow` — added by wave 6 (`ParseShader`'s per-stage `glow`
    /// aggregation, `#ifndef _XBOX` leg).
    ///
    /// Source: `oracle/codemp/renderer/tr_shader.cpp:2341-2346`
    pub has_glow: bool,
    /// `shader.clampTime` — added by wave 6 (`ParseShader`'s `clampTime`
    /// keyword).
    ///
    /// Source: `oracle/codemp/renderer/tr_shader.cpp:2415-2420`
    pub clamp_time: f32,
    /// `shader.entityMergable` — added by wave 6 (`ParseShader`'s
    /// `entityMergable` keyword).
    ///
    /// Source: `oracle/codemp/renderer/tr_shader.cpp:2460-2468`
    pub entity_mergable: bool,
    /// `shader.fogParms` — added by wave 6 (`ParseShader`'s `fogParms`
    /// keyword). `Hunk_Alloc`'d in the oracle; owned inline here (§C9), same
    /// pattern as `sky: Option<SkyParms>` above.
    ///
    /// Source: `oracle/codemp/renderer/tr_shader.cpp:2469-2488`
    pub fog_parms: Option<FogParms>,
    /// `shader.cullType` — added by wave 6 (`ParseShader`'s `cull` keyword).
    ///
    /// Source: `oracle/codemp/renderer/tr_shader.cpp:2507-2530`
    pub cull_type: CullType,
    /// `shader.fogPass` — added by wave 6 (`ParseShader`'s `noglfog`
    /// keyword). See `FogPass`'s doc comment for the `ShaderAsset`-sync
    /// caveat.
    ///
    /// Source: `oracle/codemp/renderer/tr_shader.cpp:2444-2448`
    pub fog_pass: FogPass,
}

/// Mechanical replacement for the oracle's `lightmapIndex ==
/// lightmaps2d/lightmapsFullBright/lightmapsNone/lightmapsVertex`
/// sentinel-pointer-identity checks in `R_CreateExtendedName` — the
/// interior-safety law forbids raw pointers, so the caller states its intent
/// directly instead of passing one of four magic pointers for
/// `R_CreateExtendedName` to compare by address.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp` (`lightmaps2d`,
/// `lightmapsFullBright`, `lightmapsNone`, `lightmapsVertex`, `:177-228`)
pub enum LightmapNameMode<'a> {
    None,
    TwoD,
    Vertex,
    FullBright,
    Styled {
        lightmap_index: &'a [i32],
        styles: &'a [u8],
    },
}

/// Decodes Latin-1 bytes into an owned `String` (each byte maps 1:1 to its
/// Unicode codepoint — `native_string::latin1_to_string`'s exact rule).
/// Inlined rather than importing `native_string` — not a dependency of
/// `mp_renderer` (same precedent as `tr_bsp.rs::latin1_name`).
fn latin1_decode(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| b as char).collect()
}

/// Encodes a `String` back to Latin-1 bytes (inverse of `latin1_decode`),
/// needed to re-derive a byte cursor for the byte-oriented `COM_ParseExt`
/// tokenizer from the owned `RenderAssets::shader_text` `String`.
fn latin1_encode(s: &str) -> Vec<u8> {
    s.chars().map(|c| c as u32 as u8).collect()
}

/// Bucket index for `RenderAssets::shader_text_hash_table`. Raven's
/// `generateHashValue` is deliberately not reproduced bit-for-bit (same
/// precedent as `tr_model/render_models.rs`'s hash-table replacement note):
/// this hash is an internal cache-partitioning optimization only — every
/// candidate in a bucket is still compared by full name
/// (`FindShaderInShaderText`) — so any hash the writer
/// (`ScanAndLoadShaderFiles`) and reader (`FindShaderInShaderText`) agree on
/// preserves observable behavior.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp` (`generateHashValue` calls
/// at `:3298`, `:3957`, `:3982`)
fn shader_text_hash_bucket(name: &str, table_size: usize) -> usize {
    let mut hash: u64 = 0;
    for (i, ch) in name.chars().enumerate() {
        if ch == '.' {
            break;
        }
        hash = hash.wrapping_add((ch.to_ascii_lowercase() as u64) * (i as u64 + 1));
    }
    hash as usize % table_size.max(1)
}

/// Raven `KillTheShaderHashTable`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:111-114`
pub fn KillTheShaderHashTable(assets: &mut RenderAssets) {
    for bucket in assets.shader_text_hash_table.iter_mut() {
        bucket.clear();
    }
}

/// Raven `ShaderHashTableExists`.
///
/// PORT-NOTE: the oracle tests `shaderTextHashTable[0]` (bucket 0's pointer)
/// for non-null — true exactly once `ScanAndLoadShaderFiles` has populated
/// every bucket slot at least once (`:3966-3970`), i.e. "has the table been
/// built." The owned `Vec<Vec<usize>>` translation tests table-non-empty
/// directly instead of proxying through bucket 0.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:116-123`
pub fn ShaderHashTableExists(assets: &RenderAssets) -> bool {
    !assets.shader_text_hash_table.is_empty()
}

/// Raven `R_CreateExtendedName`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:177-228`
pub fn R_CreateExtendedName(name: &str, mode: Option<LightmapNameMode>) -> String {
    // Set the basename
    let mut extended_name = COM_StripExtension(name);

    // Add in lightmaps
    if let Some(mode) = mode {
        match mode {
            LightmapNameMode::None => extended_name.push_str("_nolightmap"),
            LightmapNameMode::TwoD => extended_name.push_str("_2d"),
            LightmapNameMode::Vertex => extended_name.push_str("_vertex"),
            LightmapNameMode::FullBright => extended_name.push_str("_fullbright"),
            LightmapNameMode::Styled {
                lightmap_index,
                styles,
            } => {
                for i in 0..4 {
                    if i >= styles.len() || styles[i] == 255 {
                        break;
                    }
                    let idx = lightmap_index.get(i).copied().unwrap_or(0);
                    match idx {
                        LIGHTMAP_NONE => {
                            extended_name.push_str(&format!("_style({},none)", styles[i]))
                        }
                        LIGHTMAP_2D => extended_name.push_str(&format!("_style({},2d)", styles[i])),
                        LIGHTMAP_BY_VERTEX => {
                            extended_name.push_str(&format!("_style({},vert)", styles[i]))
                        }
                        LIGHTMAP_WHITEIMAGE => {
                            extended_name.push_str(&format!("_style({},fb)", styles[i]))
                        }
                        other => {
                            extended_name.push_str(&format!("_style({},{})", styles[i], other))
                        }
                    }
                }
            }
        }
    }
    extended_name
}

/// The bare `Com_Memset(&shader, 0, …)` + `Com_Memset(&stages, 0, …)` pair,
/// with no `contentFlags` seed — the reset the oracle open-codes at
/// `RE_RegisterShaderFromImage` (`tr_shader.cpp:3619-3620`) and
/// `CreateInternalShaders` (`tr_shader.cpp:4141-4142`). Only
/// `ClearGlobalShader` adds `CONTENTS_SOLID|CONTENTS_OPAQUE` on top, so those
/// two callers leave `content_flags` at 0.
///
/// `mGLFogColorOverride = GLFOGOVERRIDE_NONE` is the memset's own zero value
/// (`FogColorOverride::None` is `Default`), so the per-stage init is just
/// `MAX_SHADER_STAGES` default stages.
fn reset_global_shader_bare() -> ShaderParseState {
    let mut state = ShaderParseState::default();
    for _ in 0..MAX_SHADER_STAGES {
        state.stages.push(ShaderStageParse::default());
    }
    state
}

/// Raven `ClearGlobalShader` — constructs the per-parse scratch state fresh
/// (idiomatic equivalent of zeroing the file-scope `shader`/`stages` globals
/// at the start of every `ParseShader` call, §C9 out-param -> return value).
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:234-246`
pub fn ClearGlobalShader() -> ShaderParseState {
    let mut state = reset_global_shader_bare();
    state.content_flags = CONTENTS_SOLID | CONTENTS_OPAQUE;
    state
}

/// Raven `ParseVector`.
///
/// The file-scope `shader` global is read here only to name the shader in the
/// two warning strings, so this takes the name directly (`shader_name`)
/// rather than a whole `&ShaderParseState`: shader-path callers pass
/// `&state.name`, and the non-shader caller (`R_WorldEffectCommand`) passes
/// what it actually has instead of fabricating a parse state.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:323-350`
pub fn ParseVector<'a>(
    qs: &mut QSharedScratch,
    text: &mut Option<&'a [u8]>,
    common: &mut Common,
    shader_name: &str,
    count: usize,
    v: &mut [f32],
) -> bool {
    let warn = S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII");

    // FIXME: spaces are currently required after parens, should change parseext...
    let (token, rest) = COM_ParseExt(qs, *text, false);
    *text = rest;
    if token != "(" {
        com_printf(
            common,
            &format!(
                "{}WARNING: missing parenthesis in shader '{}'\n",
                warn, shader_name
            ),
        );
        return false;
    }

    for slot in v.iter_mut().take(count) {
        let (token, rest) = COM_ParseExt(qs, *text, false);
        *text = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing vector element in shader '{}'\n",
                    warn, shader_name
                ),
            );
            return false;
        }
        *slot = atof(&token) as f32;
    }

    let (token, rest) = COM_ParseExt(qs, *text, false);
    *text = rest;
    if token != ")" {
        com_printf(
            common,
            &format!(
                "{}WARNING: missing parenthesis in shader '{}'\n",
                warn, shader_name
            ),
        );
        return false;
    }

    true
}

/// Raven `NameToAFunc`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:358-379`
pub fn NameToAFunc(state: &ShaderParseState, common: &mut Common, funcname: &str) -> u32 {
    if funcname.eq_ignore_ascii_case("GT0") {
        GLS_ATEST_GT_0
    } else if funcname.eq_ignore_ascii_case("LT128") {
        GLS_ATEST_LT_80
    } else if funcname.eq_ignore_ascii_case("GE128") {
        GLS_ATEST_GE_80
    } else if funcname.eq_ignore_ascii_case("GE192") {
        GLS_ATEST_GE_C0
    } else {
        let warn = S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII");
        com_printf(
            common,
            &format!(
                "{}WARNING: invalid alphaFunc name '{}' in shader '{}'\n",
                warn, funcname, state.name
            ),
        );
        0
    }
}

/// Raven `NameToSrcBlendMode`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:387-428`
pub fn NameToSrcBlendMode(state: &ShaderParseState, common: &mut Common, name: &str) -> i32 {
    if name.eq_ignore_ascii_case("GL_ONE") {
        GLS_SRCBLEND_ONE
    } else if name.eq_ignore_ascii_case("GL_ZERO") {
        GLS_SRCBLEND_ZERO
    } else if name.eq_ignore_ascii_case("GL_DST_COLOR") {
        GLS_SRCBLEND_DST_COLOR
    } else if name.eq_ignore_ascii_case("GL_ONE_MINUS_DST_COLOR") {
        GLS_SRCBLEND_ONE_MINUS_DST_COLOR
    } else if name.eq_ignore_ascii_case("GL_SRC_ALPHA") {
        GLS_SRCBLEND_SRC_ALPHA
    } else if name.eq_ignore_ascii_case("GL_ONE_MINUS_SRC_ALPHA") {
        GLS_SRCBLEND_ONE_MINUS_SRC_ALPHA
    } else if name.eq_ignore_ascii_case("GL_DST_ALPHA") {
        GLS_SRCBLEND_DST_ALPHA
    } else if name.eq_ignore_ascii_case("GL_ONE_MINUS_DST_ALPHA") {
        GLS_SRCBLEND_ONE_MINUS_DST_ALPHA
    } else if name.eq_ignore_ascii_case("GL_SRC_ALPHA_SATURATE") {
        GLS_SRCBLEND_ALPHA_SATURATE
    } else {
        let warn = S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII");
        com_printf(
            common,
            &format!(
                "{}WARNING: unknown blend mode '{}' in shader '{}', substituting GL_ONE\n",
                warn, name, state.name
            ),
        );
        GLS_SRCBLEND_ONE
    }
}

/// Raven `NameToDstBlendMode`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:435-472`
pub fn NameToDstBlendMode(state: &ShaderParseState, common: &mut Common, name: &str) -> i32 {
    if name.eq_ignore_ascii_case("GL_ONE") {
        GLS_DSTBLEND_ONE
    } else if name.eq_ignore_ascii_case("GL_ZERO") {
        GLS_DSTBLEND_ZERO
    } else if name.eq_ignore_ascii_case("GL_SRC_ALPHA") {
        GLS_DSTBLEND_SRC_ALPHA
    } else if name.eq_ignore_ascii_case("GL_ONE_MINUS_SRC_ALPHA") {
        GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA
    } else if name.eq_ignore_ascii_case("GL_DST_ALPHA") {
        GLS_DSTBLEND_DST_ALPHA
    } else if name.eq_ignore_ascii_case("GL_ONE_MINUS_DST_ALPHA") {
        GLS_DSTBLEND_ONE_MINUS_DST_ALPHA
    } else if name.eq_ignore_ascii_case("GL_SRC_COLOR") {
        GLS_DSTBLEND_SRC_COLOR
    } else if name.eq_ignore_ascii_case("GL_ONE_MINUS_SRC_COLOR") {
        GLS_DSTBLEND_ONE_MINUS_SRC_COLOR
    } else {
        let warn = S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII");
        com_printf(
            common,
            &format!(
                "{}WARNING: unknown blend mode '{}' in shader '{}', substituting GL_ONE\n",
                warn, name, state.name
            ),
        );
        GLS_DSTBLEND_ONE
    }
}

/// Raven `NameToGenFunc`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:479-512`
pub fn NameToGenFunc(state: &ShaderParseState, common: &mut Common, funcname: &str) -> GenFunc {
    if funcname.eq_ignore_ascii_case("sin") {
        GenFunc::Sin
    } else if funcname.eq_ignore_ascii_case("square") {
        GenFunc::Square
    } else if funcname.eq_ignore_ascii_case("triangle") {
        GenFunc::Triangle
    } else if funcname.eq_ignore_ascii_case("sawtooth") {
        GenFunc::Sawtooth
    } else if funcname.eq_ignore_ascii_case("inversesawtooth") {
        GenFunc::InverseSawtooth
    } else if funcname.eq_ignore_ascii_case("noise") {
        GenFunc::Noise
    } else if funcname.eq_ignore_ascii_case("random") {
        GenFunc::Rand
    } else {
        let warn = S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII");
        com_printf(
            common,
            &format!(
                "{}WARNING: invalid genfunc name '{}' in shader '{}'\n",
                warn, funcname, state.name
            ),
        );
        GenFunc::Sin
    }
}

/// Raven `ParseSurfaceSprites`.
///
/// `_text`/`**text` collapses to a plain `&[u8]` cursor: the oracle builds
/// its local `const char **text = &_text` purely for internal iteration and
/// never returns the advanced cursor to the caller.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:808-942`
pub fn ParseSurfaceSprites(
    text: &[u8],
    stage: &mut ShaderStageParse,
    qs: &mut QSharedScratch,
    state: &ShaderParseState,
    common: &mut Common,
) {
    let warn = S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII");
    let mut cursor: Option<&[u8]> = Some(text);

    // spritetype
    let (token, rest) = COM_ParseExt(qs, cursor, false);
    cursor = rest;
    if token.is_empty() {
        com_printf(
            common,
            &format!(
                "{}WARNING: missing surfaceSprites params in shader '{}'\n",
                warn, state.name
            ),
        );
        return;
    }
    let sstype = if token.eq_ignore_ascii_case("vertical") {
        SURFSPRITE_VERTICAL
    } else if token.eq_ignore_ascii_case("oriented") {
        SURFSPRITE_ORIENTED
    } else if token.eq_ignore_ascii_case("effect") {
        SURFSPRITE_EFFECT
    } else {
        com_printf(
            common,
            &format!("{}WARNING: invalid type in shader '{}'\n", warn, state.name),
        );
        return;
    };

    // width
    let (token, rest) = COM_ParseExt(qs, cursor, false);
    cursor = rest;
    if token.is_empty() {
        com_printf(
            common,
            &format!(
                "{}WARNING: missing surfaceSprites params in shader '{}'\n",
                warn, state.name
            ),
        );
        return;
    }
    let width: f32 = atof(&token) as f32;
    if width <= 0.0 {
        com_printf(
            common,
            &format!(
                "{}WARNING: invalid width in shader '{}'\n",
                warn, state.name
            ),
        );
        return;
    }

    // height
    let (token, rest) = COM_ParseExt(qs, cursor, false);
    cursor = rest;
    if token.is_empty() {
        com_printf(
            common,
            &format!(
                "{}WARNING: missing surfaceSprites params in shader '{}'\n",
                warn, state.name
            ),
        );
        return;
    }
    let height: f32 = atof(&token) as f32;
    if height <= 0.0 {
        com_printf(
            common,
            &format!(
                "{}WARNING: invalid height in shader '{}'\n",
                warn, state.name
            ),
        );
        return;
    }

    // density
    let (token, rest) = COM_ParseExt(qs, cursor, false);
    cursor = rest;
    if token.is_empty() {
        com_printf(
            common,
            &format!(
                "{}WARNING: missing surfaceSprites params in shader '{}'\n",
                warn, state.name
            ),
        );
        return;
    }
    let density: f32 = atof(&token) as f32;
    if density <= 0.0 {
        com_printf(
            common,
            &format!(
                "{}WARNING: invalid density in shader '{}'\n",
                warn, state.name
            ),
        );
        return;
    }

    // fadedist
    let (token, rest) = COM_ParseExt(qs, cursor, false);
    cursor = rest;
    if token.is_empty() {
        com_printf(
            common,
            &format!(
                "{}WARNING: missing surfaceSprites params in shader '{}'\n",
                warn, state.name
            ),
        );
        return;
    }
    let fadedist: f32 = atof(&token) as f32;
    if fadedist < 32.0 {
        com_printf(
            common,
            &format!(
                "{}WARNING: invalid fadedist ({} < 32) in shader '{}'\n",
                warn, fadedist, state.name
            ),
        );
        return;
    }

    if stage.ss.is_none() {
        stage.ss = Some(Box::new(SurfaceSpriteParse::default()));
    }
    let ss = stage.ss.as_mut().expect("just set above");

    // These are all set by the command lines.
    ss.surface_sprite_type = sstype;
    ss.width = width;
    ss.height = height;
    ss.density = density;
    ss.fade_dist = fadedist;

    // These are defaults that can be overwritten.
    ss.fade_max = fadedist * 1.33;
    ss.fade_scale = 0.0;
    ss.wind = 0.0;
    ss.wind_idle = 0.0;
    ss.variance = [0.0, 0.0];
    ss.facing = SURFSPRITE_FACING_NORMAL;

    // A vertical parameter that needs a default regardless
    // PORT-NOTE: Raven's `stage->ss->vertSkew;` is a no-op statement (dead
    // code, porting-rules §19) — left at `SurfaceSpriteParse::default()`'s
    // zero value rather than reproduced.

    // These are effect parameters that need defaults nonetheless.
    ss.fx_duration = 1000.0; // 1 second
    ss.fx_grow = [0.0, 0.0];
    ss.fx_alpha_start = 1.0;
    ss.fx_alpha_end = 0.0;
}

/// Raven `ParseSurfaceSpritesOptional`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:970-1271`
pub fn ParseSurfaceSpritesOptional(
    param: &str,
    text: &[u8],
    stage: &mut ShaderStageParse,
    qs: &mut QSharedScratch,
    state: &ShaderParseState,
    common: &mut Common,
) {
    let warn = S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII");
    let mut cursor: Option<&[u8]> = Some(text);

    if stage.ss.is_none() {
        stage.ss = Some(Box::new(SurfaceSpriteParse::default()));
    }

    // fademax
    if param.eq_ignore_ascii_case("ssFademax") {
        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing surfacesprite fademax in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        let value: f32 = atof(&token) as f32;
        let ss = stage.ss.as_ref().expect("set above");
        if value <= ss.fade_dist {
            com_printf(
                common,
                &format!(
                    "{}WARNING: invalid surfacesprite fademax ({:.2} <= fadeDist({:.2})) in shader '{}'\n",
                    warn, value, ss.fade_dist, state.name
                ),
            );
            return;
        }
        stage.ss.as_mut().expect("set above").fade_max = value;
        return;
    }

    // fadescale
    if param.eq_ignore_ascii_case("ssFadescale") {
        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing surfacesprite fadescale in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        let value: f32 = atof(&token) as f32;
        stage.ss.as_mut().expect("set above").fade_scale = value;
        return;
    }

    // variance
    if param.eq_ignore_ascii_case("ssVariance") {
        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing surfacesprite variance width in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        let value: f32 = atof(&token) as f32;
        if value < 0.0 {
            com_printf(
                common,
                &format!(
                    "{}WARNING: invalid surfacesprite variance width in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.ss.as_mut().expect("set above").variance[0] = value;

        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing surfacesprite variance height in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        let value: f32 = atof(&token) as f32;
        if value < 0.0 {
            com_printf(
                common,
                &format!(
                    "{}WARNING: invalid surfacesprite variance height in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.ss.as_mut().expect("set above").variance[1] = value;
        let _ = cursor;
        return;
    }

    // hangdown
    if param.eq_ignore_ascii_case("ssHangdown") {
        let ss = stage.ss.as_mut().expect("set above");
        if ss.facing != SURFSPRITE_FACING_NORMAL {
            com_printf(
                common,
                &format!(
                    "{}WARNING: Hangdown facing overrides previous facing in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        ss.facing = SURFSPRITE_FACING_DOWN;
        return;
    }

    // anyangle
    if param.eq_ignore_ascii_case("ssAnyangle") {
        let ss = stage.ss.as_mut().expect("set above");
        if ss.facing != SURFSPRITE_FACING_NORMAL {
            com_printf(
                common,
                &format!(
                    "{}WARNING: Anyangle facing overrides previous facing in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        ss.facing = SURFSPRITE_FACING_ANY;
        return;
    }

    // faceup
    if param.eq_ignore_ascii_case("ssFaceup") {
        let ss = stage.ss.as_mut().expect("set above");
        if ss.facing != SURFSPRITE_FACING_NORMAL {
            com_printf(
                common,
                &format!(
                    "{}WARNING: Faceup facing overrides previous facing in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        ss.facing = SURFSPRITE_FACING_UP;
        return;
    }

    // wind
    if param.eq_ignore_ascii_case("ssWind") {
        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing surfacesprite wind in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        let value: f32 = atof(&token) as f32;
        if value < 0.0 {
            com_printf(
                common,
                &format!(
                    "{}WARNING: invalid surfacesprite wind in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        let ss = stage.ss.as_mut().expect("set above");
        ss.wind = value;
        if ss.wind_idle <= 0.0 {
            // Also override the windidle, it usually is the same as wind
            ss.wind_idle = value;
        }
        let _ = cursor;
        return;
    }

    // windidle
    if param.eq_ignore_ascii_case("ssWindidle") {
        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing surfacesprite windidle in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        let value: f32 = atof(&token) as f32;
        if value < 0.0 {
            com_printf(
                common,
                &format!(
                    "{}WARNING: invalid surfacesprite windidle in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.ss.as_mut().expect("set above").wind_idle = value;
        let _ = cursor;
        return;
    }

    // vertskew
    if param.eq_ignore_ascii_case("ssVertskew") {
        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing surfacesprite vertskew in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        let value: f32 = atof(&token) as f32;
        if value < 0.0 {
            com_printf(
                common,
                &format!(
                    "{}WARNING: invalid surfacesprite vertskew in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.ss.as_mut().expect("set above").vert_skew = value;
        let _ = cursor;
        return;
    }

    // fxduration
    if param.eq_ignore_ascii_case("ssFXDuration") {
        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing surfacesprite duration in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        let value: f32 = atof(&token) as f32;
        if value <= 0.0 {
            com_printf(
                common,
                &format!(
                    "{}WARNING: invalid surfacesprite duration in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.ss.as_mut().expect("set above").fx_duration = value;
        let _ = cursor;
        return;
    }

    // fxgrow
    if param.eq_ignore_ascii_case("ssFXGrow") {
        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing surfacesprite grow width in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        let value: f32 = atof(&token) as f32;
        if value < 0.0 {
            com_printf(
                common,
                &format!(
                    "{}WARNING: invalid surfacesprite grow width in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.ss.as_mut().expect("set above").fx_grow[0] = value;

        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing surfacesprite grow height in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        let value: f32 = atof(&token) as f32;
        if value < 0.0 {
            com_printf(
                common,
                &format!(
                    "{}WARNING: invalid surfacesprite grow height in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.ss.as_mut().expect("set above").fx_grow[1] = value;
        let _ = cursor;
        return;
    }

    // fxalpharange
    if param.eq_ignore_ascii_case("ssFXAlphaRange") {
        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing surfacesprite fxalpha start in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        let value: f32 = atof(&token) as f32;
        if !(0.0..=1.0).contains(&value) {
            com_printf(
                common,
                &format!(
                    "{}WARNING: invalid surfacesprite fxalpha start in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.ss.as_mut().expect("set above").fx_alpha_start = value;

        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing surfacesprite fxalpha end in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        let value: f32 = atof(&token) as f32;
        if !(0.0..=1.0).contains(&value) {
            com_printf(
                common,
                &format!(
                    "{}WARNING: invalid surfacesprite fxalpha end in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.ss.as_mut().expect("set above").fx_alpha_end = value;
        let _ = cursor;
        return;
    }

    // fxweather
    if param.eq_ignore_ascii_case("ssFXWeather") {
        let ss = stage.ss.as_mut().expect("set above");
        if ss.surface_sprite_type != SURFSPRITE_EFFECT {
            com_printf(
                common,
                &format!(
                    "{}WARNING: weather applied to non-effect surfacesprite in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        ss.surface_sprite_type = SURFSPRITE_WEATHERFX;
        return;
    }

    // invalid ss command.
    com_printf(
        common,
        &format!(
            "{}WARNING: invalid optional surfacesprite param '{}' in shader '{}'\n",
            warn, param, state.name
        ),
    );
}

/// Raven `ParseSort`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:2145-2186`
pub fn ParseSort(
    state: &mut ShaderParseState,
    qs: &mut QSharedScratch,
    text: &mut Option<&[u8]>,
    common: &mut Common,
) {
    let warn = S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII");
    let (token, rest) = COM_ParseExt(qs, *text, false);
    *text = rest;
    if token.is_empty() {
        com_printf(
            common,
            &format!(
                "{}WARNING: missing sort parameter in shader '{}'\n",
                warn, state.name
            ),
        );
        return;
    }

    state.sort = if token.eq_ignore_ascii_case("portal") {
        shaderSort_t::SS_PORTAL as i32 as f32
    } else if token.eq_ignore_ascii_case("sky") {
        shaderSort_t::SS_ENVIRONMENT as i32 as f32
    } else if token.eq_ignore_ascii_case("opaque") {
        shaderSort_t::SS_OPAQUE as i32 as f32
    } else if token.eq_ignore_ascii_case("decal") {
        shaderSort_t::SS_DECAL as i32 as f32
    } else if token.eq_ignore_ascii_case("seeThrough") {
        shaderSort_t::SS_SEE_THROUGH as i32 as f32
    } else if token.eq_ignore_ascii_case("banner") {
        shaderSort_t::SS_BANNER as i32 as f32
    } else if token.eq_ignore_ascii_case("additive") {
        shaderSort_t::SS_BLEND1 as i32 as f32
    } else if token.eq_ignore_ascii_case("nearest") {
        shaderSort_t::SS_NEAREST as i32 as f32
    } else if token.eq_ignore_ascii_case("underwater") {
        shaderSort_t::SS_UNDERWATER as i32 as f32
    } else if token.eq_ignore_ascii_case("inside") {
        shaderSort_t::SS_INSIDE as i32 as f32
    } else if token.eq_ignore_ascii_case("mid_inside") {
        shaderSort_t::SS_MID_INSIDE as i32 as f32
    } else if token.eq_ignore_ascii_case("middle") {
        shaderSort_t::SS_MIDDLE as i32 as f32
    } else if token.eq_ignore_ascii_case("mid_outside") {
        shaderSort_t::SS_MID_OUTSIDE as i32 as f32
    } else if token.eq_ignore_ascii_case("outside") {
        shaderSort_t::SS_OUTSIDE as i32 as f32
    } else {
        atof(&token) as f32
    };
}

// Raven `const char *materialNames[MATERIAL_LAST] = { MATERIALS };` — the
// `MATERIALS` X-macro's expansion is already ported as
// `mp_qshared::shared::surface_flags::MATERIALS`, used directly below.
// Source: `oracle/codemp/renderer/tr_shader.cpp:2193-2196`;
// `oracle/codemp/game/surfaceflags.h:90-123`

/// Raven `ParseMaterial`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:2198-2217`
pub fn ParseMaterial(
    state: &mut ShaderParseState,
    qs: &mut QSharedScratch,
    text: &mut Option<&[u8]>,
    common: &mut Common,
) {
    let warn = S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII");
    let (token, rest) = COM_ParseExt(qs, *text, false);
    *text = rest;
    if token.is_empty() {
        com_printf(
            common,
            &format!(
                "{}WARNING: missing material in shader '{}'\n",
                warn, state.name
            ),
        );
        return;
    }
    for (i, name) in MATERIALS.iter().enumerate() {
        if token.eq_ignore_ascii_case(name) {
            state.surface_flags |= i as i32;
            break;
        }
    }
}

/// Raven `infoParm_t` — one `ParseSurfaceParm` table row.
struct InfoParm {
    name: &'static str,
    surface_flags: i32,
    contents: i32,
    clear_solid: i32,
}

/// Raven `infoParms[]`.
///
/// Raven: this table is also present in q3map.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:2228-2273`
const INFO_PARMS: &[InfoParm] = &[
    // Game content Flags
    // special hack to clear solid flag
    InfoParm {
        name: "nonsolid",
        clear_solid: !CONTENTS_SOLID,
        surface_flags: 0,
        contents: 0,
    },
    // special hack to clear opaque flag
    InfoParm {
        name: "nonopaque",
        clear_solid: !CONTENTS_OPAQUE,
        surface_flags: 0,
        contents: 0,
    },
    // very damaging
    InfoParm {
        name: "lava",
        clear_solid: !CONTENTS_SOLID,
        surface_flags: 0,
        contents: CONTENTS_LAVA,
    },
    // mildly damaging
    InfoParm {
        name: "slime",
        clear_solid: !CONTENTS_SOLID,
        surface_flags: 0,
        contents: CONTENTS_SLIME,
    },
    InfoParm {
        name: "water",
        clear_solid: !CONTENTS_SOLID,
        surface_flags: 0,
        contents: CONTENTS_WATER,
    },
    // carves surfaces entering
    InfoParm {
        name: "fog",
        clear_solid: !CONTENTS_SOLID,
        surface_flags: 0,
        contents: CONTENTS_FOG,
    },
    // block shots, but not people
    InfoParm {
        name: "shotclip",
        clear_solid: !CONTENTS_SOLID,
        surface_flags: 0,
        contents: CONTENTS_SHOTCLIP,
    },
    // block only the player
    InfoParm {
        name: "playerclip",
        clear_solid: !(CONTENTS_SOLID | CONTENTS_OPAQUE),
        surface_flags: 0,
        contents: CONTENTS_PLAYERCLIP,
    },
    InfoParm {
        name: "monsterclip",
        clear_solid: !(CONTENTS_SOLID | CONTENTS_OPAQUE),
        surface_flags: 0,
        contents: CONTENTS_MONSTERCLIP,
    },
    // for bots
    InfoParm {
        name: "botclip",
        clear_solid: !(CONTENTS_SOLID | CONTENTS_OPAQUE),
        surface_flags: 0,
        contents: CONTENTS_BOTCLIP,
    },
    InfoParm {
        name: "trigger",
        clear_solid: !(CONTENTS_SOLID | CONTENTS_OPAQUE),
        surface_flags: 0,
        contents: CONTENTS_TRIGGER,
    },
    // don't drop items or leave bodies (death fog, lava, etc)
    InfoParm {
        name: "nodrop",
        clear_solid: !(CONTENTS_SOLID | CONTENTS_OPAQUE),
        surface_flags: 0,
        contents: CONTENTS_NODROP,
    },
    // use special terrain collsion
    InfoParm {
        name: "terrain",
        clear_solid: !(CONTENTS_SOLID | CONTENTS_OPAQUE),
        surface_flags: 0,
        contents: CONTENTS_TERRAIN,
    },
    // climb up in it like water
    InfoParm {
        name: "ladder",
        clear_solid: !(CONTENTS_SOLID | CONTENTS_OPAQUE),
        surface_flags: 0,
        contents: CONTENTS_LADDER,
    },
    // can abseil down this brush
    InfoParm {
        name: "abseil",
        clear_solid: !(CONTENTS_SOLID | CONTENTS_OPAQUE),
        surface_flags: 0,
        contents: CONTENTS_ABSEIL,
    },
    // volume is considered to be in the outside (i.e. not indoors)
    InfoParm {
        name: "outside",
        clear_solid: !(CONTENTS_SOLID | CONTENTS_OPAQUE),
        surface_flags: 0,
        contents: CONTENTS_OUTSIDE,
    },
    // volume is considered to be inside (i.e. indoors)
    InfoParm {
        name: "inside",
        clear_solid: !(CONTENTS_SOLID | CONTENTS_OPAQUE),
        surface_flags: 0,
        contents: CONTENTS_INSIDE,
    },
    // don't include in structural bsp
    InfoParm {
        name: "detail",
        clear_solid: -1,
        surface_flags: 0,
        contents: CONTENTS_DETAIL,
    },
    // surface has an alpha component
    InfoParm {
        name: "trans",
        clear_solid: -1,
        surface_flags: 0,
        contents: CONTENTS_TRANSLUCENT,
    },
    // Game surface flags
    // emit light from an environment map
    InfoParm {
        name: "sky",
        clear_solid: -1,
        surface_flags: SURF_SKY,
        contents: 0,
    },
    InfoParm {
        name: "slick",
        clear_solid: -1,
        surface_flags: SURF_SLICK,
        contents: 0,
    },
    InfoParm {
        name: "nodamage",
        clear_solid: -1,
        surface_flags: SURF_NODAMAGE,
        contents: 0,
    },
    // don't make impact explosions or marks
    InfoParm {
        name: "noimpact",
        clear_solid: -1,
        surface_flags: SURF_NOIMPACT,
        contents: 0,
    },
    // don't make impact marks, but still explode
    InfoParm {
        name: "nomarks",
        clear_solid: -1,
        surface_flags: SURF_NOMARKS,
        contents: 0,
    },
    // don't generate a drawsurface (or a lightmap)
    InfoParm {
        name: "nodraw",
        clear_solid: -1,
        surface_flags: SURF_NODRAW,
        contents: 0,
    },
    InfoParm {
        name: "nosteps",
        clear_solid: -1,
        surface_flags: SURF_NOSTEPS,
        contents: 0,
    },
    // don't ever add dynamic lights
    InfoParm {
        name: "nodlight",
        clear_solid: -1,
        surface_flags: SURF_NODLIGHT,
        contents: 0,
    },
    InfoParm {
        name: "metalsteps",
        clear_solid: -1,
        surface_flags: SURF_METALSTEPS,
        contents: 0,
    },
    // No misc ents on this surface
    InfoParm {
        name: "nomiscents",
        clear_solid: -1,
        surface_flags: SURF_NOMISCENTS,
        contents: 0,
    },
    InfoParm {
        name: "forcefield",
        clear_solid: -1,
        surface_flags: SURF_FORCEFIELD,
        contents: 0,
    },
];

/// Raven `ParseSurfaceParm`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:2275-2289`
pub fn ParseSurfaceParm(
    state: &mut ShaderParseState,
    qs: &mut QSharedScratch,
    text: &mut Option<&[u8]>,
) {
    let (token, rest) = COM_ParseExt(qs, *text, false);
    *text = rest;
    for parm in INFO_PARMS {
        if token.eq_ignore_ascii_case(parm.name) {
            state.surface_flags |= parm.surface_flags;
            state.content_flags |= parm.contents;
            state.content_flags &= parm.clear_solid;
            break;
        }
    }
}

/// Raven `collapse_t` — one blend-mode pair the multitexture collapse accepts.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:2564-2570`
struct Collapse {
    blend_a: i32,
    blend_b: i32,
    multitexture_env: i32,
    multitexture_blend: i32,
}

/// Raven `collapse[]` — six `GL_MODULATE` rows and two `GL_ADD` rows. The
/// oracle's trailing `{ -1 }` sentinel row is only the loop terminator, so the
/// idiomatic `find` over the real rows drops it. The `#if 0` `GL_DECAL` row is
/// compiled out in the oracle and stays out here.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:2573-2602`
static COLLAPSE: [Collapse; 8] = [
    Collapse {
        blend_a: 0,
        blend_b: GLS_DSTBLEND_SRC_COLOR | GLS_SRCBLEND_ZERO,
        multitexture_env: GL_MODULATE,
        multitexture_blend: 0,
    },
    Collapse {
        blend_a: 0,
        blend_b: GLS_DSTBLEND_ZERO | GLS_SRCBLEND_DST_COLOR,
        multitexture_env: GL_MODULATE,
        multitexture_blend: 0,
    },
    Collapse {
        blend_a: GLS_DSTBLEND_ZERO | GLS_SRCBLEND_DST_COLOR,
        blend_b: GLS_DSTBLEND_ZERO | GLS_SRCBLEND_DST_COLOR,
        multitexture_env: GL_MODULATE,
        multitexture_blend: GLS_DSTBLEND_ZERO | GLS_SRCBLEND_DST_COLOR,
    },
    Collapse {
        blend_a: GLS_DSTBLEND_SRC_COLOR | GLS_SRCBLEND_ZERO,
        blend_b: GLS_DSTBLEND_ZERO | GLS_SRCBLEND_DST_COLOR,
        multitexture_env: GL_MODULATE,
        multitexture_blend: GLS_DSTBLEND_ZERO | GLS_SRCBLEND_DST_COLOR,
    },
    Collapse {
        blend_a: GLS_DSTBLEND_ZERO | GLS_SRCBLEND_DST_COLOR,
        blend_b: GLS_DSTBLEND_SRC_COLOR | GLS_SRCBLEND_ZERO,
        multitexture_env: GL_MODULATE,
        multitexture_blend: GLS_DSTBLEND_ZERO | GLS_SRCBLEND_DST_COLOR,
    },
    Collapse {
        blend_a: GLS_DSTBLEND_SRC_COLOR | GLS_SRCBLEND_ZERO,
        blend_b: GLS_DSTBLEND_SRC_COLOR | GLS_SRCBLEND_ZERO,
        multitexture_env: GL_MODULATE,
        multitexture_blend: GLS_DSTBLEND_ZERO | GLS_SRCBLEND_DST_COLOR,
    },
    Collapse {
        blend_a: 0,
        blend_b: GLS_DSTBLEND_ONE | GLS_SRCBLEND_ONE,
        multitexture_env: GL_ADD,
        multitexture_blend: 0,
    },
    Collapse {
        blend_a: GLS_DSTBLEND_ONE | GLS_SRCBLEND_ONE,
        blend_b: GLS_DSTBLEND_ONE | GLS_SRCBLEND_ONE,
        multitexture_env: GL_ADD,
        multitexture_blend: GLS_DSTBLEND_ONE | GLS_SRCBLEND_ONE,
    },
];

/// Raven `CollapseMultitexture`.
///
/// This tries to combine stage 0 and stage 1 into one multitexture stage.
/// A match writes the blend into `stage 0`, records the texture-env on the
/// shader, and shifts the later stages down one slot.
///
/// The backend draws the `GL_MODULATE` collapse only when bundle 1 is a
/// lightmap. `is_modulate_collapse` in `pipeline3d.rs` reads
/// `multitexture_env == GL_MODULATE` with a lightmap in bundle 1. Any other
/// bundle 1 draws bundle 0 alone with a `warn_once`. DEC-53 rules the limit
/// stays: the retail census found six affected shaders, all SP content, and
/// `maps/t1_fatal.bsp` is the one BSP consumer.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:2612-2713`
pub fn CollapseMultitexture(
    state: &mut ShaderParseState,
    texture_env_add_available: bool,
) -> bool {
    // The oracle gates on `qglActiveTextureARB`. The wgpu backend has a real
    // two-texture path, so multitexture is always available and the gate is
    // dropped.

    // make sure both stages are active
    if !state.stages[0].active || !state.stages[1].active {
        return false;
    }

    let mut abits = state.stages[0].state_bits as i32;
    let mut bbits = state.stages[1].state_bits as i32;

    // make sure that both stages have identical state other than blend modes
    let non_blend_mask = !(GLS_DSTBLEND_BITS | GLS_SRCBLEND_BITS | GLS_DEPTHMASK_TRUE);
    if (abits & non_blend_mask) != (bbits & non_blend_mask) {
        return false;
    }

    abits &= GLS_DSTBLEND_BITS | GLS_SRCBLEND_BITS;
    bbits &= GLS_DSTBLEND_BITS | GLS_SRCBLEND_BITS;

    // search for a valid multitexture blend function
    // nothing found
    let Some(collapse) = COLLAPSE
        .iter()
        .find(|c| c.blend_a == abits && c.blend_b == bbits)
    else {
        return false;
    };

    // GL_ADD is a separate extension. Our synthesized `GlConfig` reports it
    // disabled (`texture_env_add_available == false`), so GL_ADD collapses do
    // not fire today.
    if collapse.multitexture_env == GL_ADD && !texture_env_add_available {
        return false;
    }

    // make sure waveforms have identical parameters
    if state.stages[0].rgb_gen != state.stages[1].rgb_gen
        || state.stages[0].alpha_gen != state.stages[1].alpha_gen
    {
        return false;
    }

    // an add collapse can only have identity colors
    if collapse.multitexture_env == GL_ADD && state.stages[0].rgb_gen != ColorGen::Identity {
        return false;
    }

    // The oracle uses `memcmp` on the wave structs. Rust value equality and the
    // memcmp differ only for a negative zero in a wave field. No retail shader
    // writes one.
    if state.stages[0].rgb_gen == ColorGen::Waveform
        && state.stages[0].rgb_wave != state.stages[1].rgb_wave
    {
        return false;
    }
    // Raven compares `alphaGen` against `CGEN_WAVEFORM`, a colorGen_t value.
    // `CGEN_WAVEFORM` is 8, and alphaGen_t value 8 is `AGEN_PORTAL`, not
    // `AGEN_WAVEFORM`. The port keeps this cross-enum bug and gates on Portal.
    // Source: oracle/codemp/renderer/tr_shader.cpp:2678, tr_local.h:226-256
    if state.stages[0].alpha_gen == AlphaGen::Portal
        && state.stages[0].alpha_wave != state.stages[1].alpha_wave
    {
        return false;
    }

    // make sure that lightmaps are in bundle 1 for 3dfx
    if state.stages[0].bundle[0].is_lightmap {
        let tmp_bundle = state.stages[0].bundle[0].clone();
        state.stages[0].bundle[0] = state.stages[1].bundle[0].clone();
        state.stages[0].bundle[1] = tmp_bundle;
    } else {
        state.stages[0].bundle[1] = state.stages[1].bundle[0].clone();
    }

    // set the new blend state bits
    state.multitexture_env = collapse.multitexture_env;
    let mut new_bits = state.stages[0].state_bits as i32;
    new_bits &= !(GLS_DSTBLEND_BITS | GLS_SRCBLEND_BITS);
    new_bits |= collapse.multitexture_blend;
    state.stages[0].state_bits = new_bits as u32;

    // move down subsequent shaders
    // The oracle memmoves `stages[2..]` into `stages[1..]` and zeroes the last
    // slot. `state.stages` is always `MAX_SHADER_STAGES` long, so the shift
    // reads slot `i + 1` before it overwrites slot `i`.
    for i in 1..MAX_SHADER_STAGES - 1 {
        state.stages[i] = state.stages[i + 1].clone();
    }
    state.stages[MAX_SHADER_STAGES - 1] = ShaderStageParse::default();

    true
}

/// Raven `SortNewShader`.
///
/// Signature: the oracle reads `tr.shaders[tr.numShaders - 1]` as
/// "the shader just registered"; the R3 caller already holds that handle
/// from the `insert()` call that created it, so it is threaded in directly
/// rather than re-derived from a count (§C7 out-param -> parameter, no
/// `numShaders` carrier exists on the `Arena`-backed registry — `R2-D3`).
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:2727-2745`
pub fn SortNewShader(assets: &mut RenderAssets, new_shader: ShaderHandle) {
    let sort = match assets.shaders.get(new_shader) {
        Some(s) => s.sort,
        None => return,
    };

    let mut insert_at = assets.sorted_shaders.len();
    while insert_at > 0 {
        let candidate = assets.sorted_shaders[insert_at - 1];
        let candidate_sort = assets
            .shaders
            .get(candidate)
            .map(|s| s.sort)
            .unwrap_or(f32::MIN);
        if candidate_sort <= sort {
            break;
        }
        insert_at -= 1;
    }

    assets.sorted_shaders.insert(insert_at, new_shader);
    if let Some(s) = assets.shaders.get_mut(new_shader) {
        s.sorted_index = insert_at as i32;
    }
    for (i, handle) in assets.sorted_shaders.iter().enumerate().skip(insert_at + 1) {
        if let Some(s) = assets.shaders.get_mut(*handle) {
            s.sorted_index = i as i32;
        }
    }
}

/// Raven `FindShaderInShaderText`.
///
/// Returns an owned copy of the remaining shader text from the matched
/// label onward (the oracle's `const char *` return, into `s_shaderText`,
/// becomes an owned `String` per the interior-safety law — no raw pointers).
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:3291-3331`
pub fn FindShaderInShaderText(
    assets: &RenderAssets,
    qs: &mut QSharedScratch,
    shadername: &str,
) -> Option<String> {
    let hash = shader_text_hash_bucket(shadername, MAX_SHADERTEXT_HASH);
    let text_bytes = latin1_encode(&assets.shader_text);

    if let Some(bucket) = assets.shader_text_hash_table.get(hash) {
        for &offset in bucket {
            let cursor: Option<&[u8]> = text_bytes.get(offset..);
            let (token, rest) = COM_ParseExt(qs, cursor, true);
            if token.eq_ignore_ascii_case(shadername) {
                return rest.map(latin1_decode);
            }
        }
    }

    if text_bytes.is_empty() {
        return None;
    }

    // look for label
    let mut cursor: Option<&[u8]> = Some(text_bytes.as_slice());
    loop {
        let (token, rest) = COM_ParseExt(qs, cursor, true);
        if token.is_empty() {
            break;
        }
        if token.eq_ignore_ascii_case(shadername) {
            return rest.map(latin1_decode);
        } else {
            // skip the definition
            cursor = SkipBracedSection(qs, rest);
        }
    }

    None
}

/// Raven `R_FindShaderByName`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:3342-3370`
pub fn R_FindShaderByName(assets: &RenderAssets, name: Option<&str>) -> ShaderHandle {
    let name = match name {
        Some(n) if !n.is_empty() => n,
        _ => return ShaderHandle::slot_zero(), // tr.defaultShader
    };

    let stripped_name = COM_StripExtension(name);

    // see if the shader is already loaded
    //
    // NOTE: if there was no shader or image available with the name strippedName
    // then a default shader is created with lightmapIndex == LIGHTMAP_NONE, so we
    // have to check all default shaders otherwise for every call to R_FindShader
    // with that same strippedName a new default shader is created.
    match assets.shader_lookup.get(&stripped_name) {
        Some(candidates) => candidates
            .first()
            .copied()
            .unwrap_or(ShaderHandle::slot_zero()),
        None => ShaderHandle::slot_zero(),
    }
}

/// Raven `IsShader`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:3373-3398`
pub fn IsShader(sh: &ShaderAsset, name: &str, lightmap_index: &[i32], styles: &[u8]) -> bool {
    if !sh.name.eq_ignore_ascii_case(name) {
        return false;
    }

    if !sh.default_shader {
        for i in 0..MAXLIGHTMAPS {
            if sh.lightmap_index[i] != lightmap_index[i] {
                return false;
            }
            if sh.styles[i] != styles[i] {
                return false;
            }
        }
    }

    true
}

/// Raven `RE_ShaderNameFromIndex`.
///
/// `index` arrives from another module as a bare slot number, so
/// `Arena::handle_at_slot` resolves it at the slot's current generation
/// (DEC-42.2 "slot = index") - the oracle's `tr.shaders[index]` read. Falls
/// back to the default shader's name rather than the oracle's debug-only
/// `assert` (§19: pick the one defined behavior).
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:3785-3789`
pub fn RE_ShaderNameFromIndex(assets: &RenderAssets, index: i32) -> &str {
    match assets
        .shaders
        .handle_at_slot(index.max(0) as u32)
        .and_then(|handle| assets.shaders.get(handle))
    {
        Some(shader) => shader.name.as_str(),
        None => assets
            .shaders
            .get(ShaderHandle::slot_zero())
            .map(|s| s.name.as_str())
            .unwrap_or(""),
    }
}

/// Raven `R_GetShaderByHandle`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:3800-3810`
pub fn R_GetShaderByHandle(
    assets: &RenderAssets,
    common: &mut Common,
    h_shader: i32,
) -> ShaderHandle {
    if h_shader < 0 {
        com_printf(
            common,
            &format!("R_GetShaderByHandle: out of range hShader '{}'\n", h_shader),
        );
        return ShaderHandle::slot_zero();
    }
    // `qhandle_t` crosses the module seam as a bare slot number, so the arena
    // resolves it at the slot's CURRENT generation (DEC-42.2 "slot = index").
    // This is the oracle's raw `tr.shaders[hShader]` read. A generation-0
    // handle misses every slot that a renderer restart recycled through
    // `CreateInternalShaders`' `Arena::reset` (DEC-42.1).
    match assets.shaders.handle_at_slot(h_shader as u32) {
        Some(handle) => handle,
        None => {
            com_printf(
                common,
                &format!("R_GetShaderByHandle: out of range hShader '{}'\n", h_shader),
            );
            ShaderHandle::slot_zero()
        }
    }
}

/// [`R_GetShaderByHandle`] for a render-thread caller.
///
/// The oracle's only difference between the two is the diagnostic on an
/// out-of-range handle. A render-thread caller has no `Common`, so this twin
/// prints the same text through `eprintln!` and prints it once per process
/// (`frame_exec`'s warn-once precedent). Both fns return the same handle for
/// the same input.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:3800-3810`
pub fn R_GetShaderByHandleQuiet(
    assets: &RenderAssets,
    h_shader: i32,
    warnings: &mut WalkWarnings,
) -> ShaderHandle {
    let resolved = if h_shader < 0 {
        None
    } else {
        // `qhandle_t` crosses the module seam as a bare slot number, so the
        // arena resolves it at the slot's CURRENT generation (DEC-42.2).
        assets.shaders.handle_at_slot(h_shader as u32)
    };
    match resolved {
        Some(handle) => handle,
        None => {
            if !warnings.shader_handle {
                warnings.shader_handle = true;
                eprintln!("R_GetShaderByHandle: out of range hShader '{}'", h_shader);
            }
            ShaderHandle::slot_zero()
        }
    }
}

/// Raven `R_ShaderList_f`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:3821-3873`
pub fn R_ShaderList_f(view: &mut EngineHostView, assets: &RenderAssets) {
    com_printf(view.common, "-----------------------\n");

    let use_sorted = Cmd_Argc(view.common) > 1;
    let mut count = 0i32;

    let shaders: Vec<&ShaderAsset> = if use_sorted {
        assets
            .sorted_shaders
            .iter()
            .filter_map(|h| assets.shaders.get(*h))
            .collect()
    } else {
        assets.shaders.iter().map(|(_, s)| s).collect()
    };

    for shader in shaders {
        com_printf(view.common, &format!("{} ", shader.num_unfogged_passes));

        if shader.lightmap_index[0] >= 0 {
            com_printf(view.common, "L ");
        } else {
            com_printf(view.common, "  ");
        }
        if shader.multitexture_env == GL_ADD {
            com_printf(view.common, "MT(a) ");
        } else if shader.multitexture_env == GL_MODULATE {
            com_printf(view.common, "MT(m) ");
        } else if shader.multitexture_env == GL_DECAL {
            com_printf(view.common, "MT(d) ");
        } else {
            com_printf(view.common, "      ");
        }
        if shader.explicitly_defined {
            com_printf(view.common, "E ");
        } else {
            com_printf(view.common, "  ");
        }

        if shader.sky.is_some() {
            com_printf(view.common, "sky ");
        } else {
            com_printf(view.common, "gen ");
        }
        if shader.default_shader {
            com_printf(view.common, &format!(": {} (DEFAULTED)\n", shader.name));
        } else {
            com_printf(view.common, &format!(": {}\n", shader.name));
        }
        count += 1;
    }
    com_printf(view.common, &format!("{} total shaders\n", count));
    com_printf(view.common, "------------------\n");
}

/// Raven `ScanAndLoadShaderFiles`.
///
/// Hunk allocation (`Hunk_Alloc` for `s_shaderText`/`hashMem`) dissolves
/// entirely under owned `String`/`Vec` — `RenderAssets::shader_text` and
/// `::shader_text_hash_table` self-manage their storage (`### FrameData`'s
/// owned-world precedent). `COM_Compress`'s in-place comment/whitespace
/// stripping is not applied: every downstream consumer of `shader_text`
/// walks it with `COM_ParseExt`, which already skips `//`/`/* */` comments
/// itself, so the compression pass changes no observable parse result here.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:3895-3989`
pub fn ScanAndLoadShaderFiles(
    assets: &mut RenderAssets,
    qs: &mut QSharedScratch,
    view: &mut EngineHostView,
    path: &str,
) {
    // scan for shader files
    let mut shader_files = FS_ListFiles(view, path, ".shader");

    if shader_files.is_empty() {
        com_error(
            errorParm_t::ERR_FATAL,
            "ERROR: no shader files found\n".to_string(),
        );
    }

    if shader_files.len() > MAX_SHADER_FILES {
        shader_files.truncate(MAX_SHADER_FILES);
    }

    // load and parse shader files
    let mut combined = String::new();
    // Concatenated in reverse file-list order (Raven's own "free in reverse
    // order, so the temp files are all dumped" loop, `:3940-3944`): whichever
    // file is LAST in `shader_files` ends up FIRST in `combined`, so on a
    // duplicate shader label across files, the last-listed file's definition
    // wins the `FindShaderInShaderText` first-match scan. Preserved exactly
    // rather than switched to forward order (porting-rules §20: preserve
    // emergent per-file precedence, even if the ordering looks accidental).
    for name in shader_files.iter().rev() {
        let filename = format!("{}/{}", path, name);
        //Com_Printf( "...loading '%s'\n", filename );
        let Some(buf) = FS_ReadFileVec(view, &filename) else {
            com_error(errorParm_t::ERR_DROP, format!("Couldn't load {}", filename))
        };
        combined.push('\n');
        combined.push_str(&latin1_decode(&buf));
    }

    assets.shader_text = combined;

    let text_bytes = latin1_encode(&assets.shader_text);

    // Pass 1: size each hash bucket.
    let mut bucket_sizes = vec![0usize; MAX_SHADERTEXT_HASH];
    let mut cursor: Option<&[u8]> = Some(text_bytes.as_slice());
    loop {
        let (token, rest) = COM_ParseExt(qs, cursor, true);
        if token.is_empty() {
            break;
        }
        let hash = shader_text_hash_bucket(&token, MAX_SHADERTEXT_HASH);
        bucket_sizes[hash] += 1;
        cursor = SkipBracedSection(qs, rest);
    }

    let mut buckets: Vec<Vec<usize>> = bucket_sizes
        .iter()
        .map(|&n| Vec::with_capacity(n))
        .collect();

    // Pass 2: record each label's byte offset in its bucket.
    let mut cursor: Option<&[u8]> = Some(text_bytes.as_slice());
    loop {
        let before = cursor;
        let (token, rest) = COM_ParseExt(qs, cursor, true);
        if token.is_empty() {
            break;
        }
        let hash = shader_text_hash_bucket(&token, MAX_SHADERTEXT_HASH);
        let offset = text_bytes.len() - before.map(|b| b.len()).unwrap_or(0);
        buckets[hash].push(offset);
        cursor = SkipBracedSection(qs, rest);
    }

    assets.shader_text_hash_table = buckets;
}

/// Raven `R_CopyStage`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:4006-4010`
pub fn R_CopyStage(orig: &ShaderStageParse, stage: &mut ShaderStageParse) {
    // Assumption: this stage has not been collapsed
    *stage = orig.clone(); // Just copy the whole thing!
}

/// Raven `ParseWaveForm`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:520-564`
pub fn ParseWaveForm(
    qs: &mut QSharedScratch,
    text: &mut Option<&[u8]>,
    common: &mut Common,
    state: &ShaderParseState,
    wave: &mut WaveForm,
) {
    let warn = S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII");

    let (token, rest) = COM_ParseExt(qs, *text, false);
    *text = rest;
    if token.is_empty() {
        com_printf(
            common,
            &format!(
                "{}WARNING: missing waveform parm in shader '{}'\n",
                warn, state.name
            ),
        );
        return;
    }
    wave.func = NameToGenFunc(state, common, &token);

    // BASE, AMP, PHASE, FREQ
    let (token, rest) = COM_ParseExt(qs, *text, false);
    *text = rest;
    if token.is_empty() {
        com_printf(
            common,
            &format!(
                "{}WARNING: missing waveform parm in shader '{}'\n",
                warn, state.name
            ),
        );
        return;
    }
    wave.base = atof(&token) as f32;

    let (token, rest) = COM_ParseExt(qs, *text, false);
    *text = rest;
    if token.is_empty() {
        com_printf(
            common,
            &format!(
                "{}WARNING: missing waveform parm in shader '{}'\n",
                warn, state.name
            ),
        );
        return;
    }
    wave.amplitude = atof(&token) as f32;

    let (token, rest) = COM_ParseExt(qs, *text, false);
    *text = rest;
    if token.is_empty() {
        com_printf(
            common,
            &format!(
                "{}WARNING: missing waveform parm in shader '{}'\n",
                warn, state.name
            ),
        );
        return;
    }
    wave.phase = atof(&token) as f32;

    let (token, rest) = COM_ParseExt(qs, *text, false);
    *text = rest;
    if token.is_empty() {
        com_printf(
            common,
            &format!(
                "{}WARNING: missing waveform parm in shader '{}'\n",
                warn, state.name
            ),
        );
        return;
    }
    wave.frequency = atof(&token) as f32;
}

/// Raven `TR_MAX_TEXMODS`.
///
/// Source: `oracle/codemp/renderer/tr_local.h:296`
pub const TR_MAX_TEXMODS: usize = 4;

/// Raven `ParseTexMod`.
///
/// The oracle takes `const char *_text` **by value** and parses through a
/// local `const char **text = &_text`, so the caller's own cursor is never
/// advanced — `text` stays a plain `&[u8]` here rather than the
/// `&mut Option<&[u8]>` cursor `ParseWaveForm` takes.
///
/// `stage->bundle[0].numTexMods` is `tex_mods.len()` (the owned `Vec`
/// replaced the count field); `tmi` is the freshly-pushed last element.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:572-794`
pub fn ParseTexMod(
    text: &[u8],
    stage: &mut ShaderStageParse,
    qs: &mut QSharedScratch,
    state: &ShaderParseState,
    common: &mut Common,
) {
    let warn = S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII");

    if stage.bundle[0].tex_mods.len() == TR_MAX_TEXMODS {
        com_error(
            errorParm_t::ERR_DROP,
            format!("ERROR: too many tcMod stages in shader '{}'\n", state.name),
        );
    }

    stage.bundle[0].tex_mods.push(TexModInfo::default());
    let tmi_index = stage.bundle[0].tex_mods.len() - 1;

    let mut cursor: Option<&[u8]> = Some(text);
    let (token, rest) = COM_ParseExt(qs, cursor, false);
    cursor = rest;

    //
    // turb
    //
    if token.eq_ignore_ascii_case("turb") {
        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing tcMod turb parms in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.bundle[0].tex_mods[tmi_index].wave.base = atof(&token) as f32;
        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing tcMod turb in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.bundle[0].tex_mods[tmi_index].wave.amplitude = atof(&token) as f32;
        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing tcMod turb in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.bundle[0].tex_mods[tmi_index].wave.phase = atof(&token) as f32;
        let (token, _rest) = COM_ParseExt(qs, cursor, false);
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing tcMod turb in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.bundle[0].tex_mods[tmi_index].wave.frequency = atof(&token) as f32;

        stage.bundle[0].tex_mods[tmi_index].kind = texMod_t::TMOD_TURBULENT as i32;
    }
    //
    // scale
    //
    else if token.eq_ignore_ascii_case("scale") {
        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing scale parms in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.bundle[0].tex_mods[tmi_index].translate[0] = atof(&token) as f32; //scale unioned

        let (token, _rest) = COM_ParseExt(qs, cursor, false);
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing scale parms in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.bundle[0].tex_mods[tmi_index].translate[1] = atof(&token) as f32; //scale unioned
        stage.bundle[0].tex_mods[tmi_index].kind = texMod_t::TMOD_SCALE as i32;
    }
    //
    // scroll
    //
    else if token.eq_ignore_ascii_case("scroll") {
        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing scale scroll parms in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.bundle[0].tex_mods[tmi_index].translate[0] = atof(&token) as f32; //scroll unioned
        let (token, _rest) = COM_ParseExt(qs, cursor, false);
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing scale scroll parms in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.bundle[0].tex_mods[tmi_index].translate[1] = atof(&token) as f32; //scroll unioned
        stage.bundle[0].tex_mods[tmi_index].kind = texMod_t::TMOD_SCROLL as i32;
    }
    //
    // stretch
    //
    else if token.eq_ignore_ascii_case("stretch") {
        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing stretch parms in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.bundle[0].tex_mods[tmi_index].wave.func = NameToGenFunc(state, common, &token);

        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing stretch parms in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.bundle[0].tex_mods[tmi_index].wave.base = atof(&token) as f32;

        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing stretch parms in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.bundle[0].tex_mods[tmi_index].wave.amplitude = atof(&token) as f32;

        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing stretch parms in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.bundle[0].tex_mods[tmi_index].wave.phase = atof(&token) as f32;

        let (token, _rest) = COM_ParseExt(qs, cursor, false);
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing stretch parms in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.bundle[0].tex_mods[tmi_index].wave.frequency = atof(&token) as f32;

        stage.bundle[0].tex_mods[tmi_index].kind = texMod_t::TMOD_STRETCH as i32;
    }
    //
    // transform
    //
    else if token.eq_ignore_ascii_case("transform") {
        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing transform parms in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.bundle[0].tex_mods[tmi_index].matrix[0][0] = atof(&token) as f32;

        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing transform parms in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.bundle[0].tex_mods[tmi_index].matrix[0][1] = atof(&token) as f32;

        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing transform parms in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.bundle[0].tex_mods[tmi_index].matrix[1][0] = atof(&token) as f32;

        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing transform parms in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.bundle[0].tex_mods[tmi_index].matrix[1][1] = atof(&token) as f32;

        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing transform parms in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.bundle[0].tex_mods[tmi_index].translate[0] = atof(&token) as f32;

        let (token, _rest) = COM_ParseExt(qs, cursor, false);
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing transform parms in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.bundle[0].tex_mods[tmi_index].translate[1] = atof(&token) as f32;

        stage.bundle[0].tex_mods[tmi_index].kind = texMod_t::TMOD_TRANSFORM as i32;
    }
    //
    // rotate
    //
    else if token.eq_ignore_ascii_case("rotate") {
        let (token, _rest) = COM_ParseExt(qs, cursor, false);
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing tcMod rotate parms in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        stage.bundle[0].tex_mods[tmi_index].translate[0] = atof(&token) as f32; //rotateSpeed unioned
        stage.bundle[0].tex_mods[tmi_index].kind = texMod_t::TMOD_ROTATE as i32;
    }
    //
    // entityTranslate
    //
    else if token.eq_ignore_ascii_case("entityTranslate") {
        stage.bundle[0].tex_mods[tmi_index].kind = texMod_t::TMOD_ENTITY_TRANSLATE as i32;
    } else {
        com_printf(
            common,
            &format!(
                "{}WARNING: unknown tcMod '{}' in shader '{}'\n",
                warn, token, state.name
            ),
        );
    }
}

/// Raven `GeneratePermanentShader`.
///
/// The per-stage copy loop (`:2782-2803`) is transcribed below: the
/// `Hunk_Alloc` sizing (`:2782-2783`) and the per-bundle `texMods`
/// `Hunk_Alloc`+`Com_Memcpy`/`= 0` split (`:2791-2802`) both dissolve into
/// the owned `Vec`s the `From` impls build (§C9), and the loop's `break` on
/// the first inactive stage is kept, so a registered shader's `stages` may be
/// shorter than its `num_unfogged_passes`.
///
/// Order: the Rust shape fills `stages` before `assets.shaders.insert(...)`,
/// where the oracle writes through the already-`Hunk_Alloc`'d `newShader`
/// after registering it. Unobservable — nothing between the two reads the
/// registry, and `SortNewShader` (which does) still runs after both.
///
/// The `fogPass` assignment (`:2768-2772`) lands on `ShaderAsset::fog_pass`
/// (the fog wave added the field): the whole-struct copy carries the scratch
/// `noglfog` value, then the sort-order/`CONTENTS_FOG` rule overrides it.
/// Every other field of the whole-struct copy (`:2766`), the arena
/// registration + capacity guard (`Arena::insert`'s existing `MAX_SHADERS`
/// soft cap returning `Handle{0,0}` — A5/A12), `SortNewShader`, and the
/// `hashTable` chain (`:2807-2809`, folded into `RenderAssets::shader_lookup`
/// per this packet's STATE HOMES row) are transcribed here.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:2753-2812`
pub fn GeneratePermanentShader(
    assets: &mut RenderAssets,
    common: &mut Common,
    state: &ShaderParseState,
) -> ShaderHandle {
    // size = newShader->numUnfoggedPasses ? ... : sizeof( stages[0] );
    // newShader->stages = Hunk_Alloc( size, h_low );
    // for ( i = 0 ; i < newShader->numUnfoggedPasses ; i++ ) {
    //     if ( !stages[i].active ) break;
    //     newShader->stages[i] = stages[i];
    //     for ( b = 0 ; b < NUM_TEXTURE_BUNDLES ; b++ ) { ...texMods... }
    // }
    // The `Hunk_Alloc` sizing has no owned-`Vec` counterpart, and the inner
    // per-bundle `texMods` copy is `TextureBundle`'s own `tex_mods` collect.
    let mut stages: Vec<ShaderStage> = Vec::new();
    for i in 0..state.num_unfogged_passes.max(0) as usize {
        let Some(src) = state.stages.get(i) else {
            break;
        };
        if !src.active {
            break;
        }
        stages.push(ShaderStage::from(src));
    }

    // *newShader = shader; — whole-struct copy of every field `ShaderAsset`
    // currently declares (interior-safety law: no `..Default::default()`
    // masking payload).
    let mut new_shader = ShaderAsset {
        name: state.name.clone(),
        lightmap_index: state.lightmap_index,
        styles: state.styles,
        sort: state.sort,
        sorted_index: 0, // set below by `SortNewShader`
        cull_type: state.cull_type,
        polygon_offset: state.polygon_offset,
        surface_flags: state.surface_flags,
        content_flags: state.content_flags,
        multitexture_env: state.multitexture_env,
        default_shader: state.default_shader,
        explicitly_defined: state.explicitly_defined,
        num_unfogged_passes: state.num_unfogged_passes,
        sky: state.sky.clone(),
        fog_parms: state.fog_parms,
        fog_pass: state.fog_pass,
        // `newShader->stages` — filled by the per-stage copy loop above.
        stages,
        // `shader.timeOffset`/`shader.remappedShader` are never written by
        // the parser — the file-scope `shader` scratch is memset before
        // `ParseShader`, and only `R_RemapShader` ever writes them, on the
        // *registered* shader — so the whole-struct copy lands the zeroed
        // values here.
        time_offset: 0.0,
        remapped_shader: None,
    };

    // if ( shader.sort <= SS_SEE_THROUGH ) newShader->fogPass = FP_EQUAL;
    // else if ( shader.contentFlags & CONTENTS_FOG ) newShader->fogPass = FP_LE;
    if new_shader.sort <= shaderSort_t::SS_SEE_THROUGH as i32 as f32 {
        new_shader.fog_pass = FogPass::Equal;
    } else if new_shader.content_flags & CONTENTS_FOG != 0 {
        new_shader.fog_pass = FogPass::Le;
    }

    // tr.shaders[tr.numShaders] = newShader; newShader->index = tr.numShaders;
    // tr.sortedShaders[tr.numShaders] = newShader; newShader->sortedIndex = tr.numShaders;
    // tr.numShaders++;
    let new_shader_handle = assets.shaders.insert(new_shader);
    if new_shader_handle == ShaderHandle::slot_zero() {
        // `Arena::insert` already returned the live default entry on
        // overflow (A12); this mutator owns the print — the packet's STATE
        // HOMES row: "overflow warns (Com_Printf) and returns Handle{0,0}".
        //Com_Printf (S_COLOR_YELLOW  "WARNING: GeneratePermanentShader - MAX_SHADERS hit\n");
        com_printf(
            common,
            "WARNING: GeneratePermanentShader - MAX_SHADERS hit\n",
        );
        return new_shader_handle;
    }

    SortNewShader(assets, new_shader_handle);

    // const int hash = generateHashValue(newShader->name, FILE_HASH_SIZE);
    // newShader->next = hashTable[hash]; hashTable[hash] = newShader;
    // `generateHashValue` deliberately not reproduced (same precedent as
    // `tr_model/render_models.rs`) — `shader_lookup` is a name-keyed map,
    // walked by `R_FindShaderByName`/`IsShader` (already ported wave-0), not
    // a numeric-hash bucket chain.
    let lookup_key = COM_StripExtension(&state.name);
    assets
        .shader_lookup
        .entry(lookup_key)
        .or_default()
        .push(new_shader_handle);

    new_shader_handle
}

/// Raven `R_CreateBlendedStage`.
///
/// `R_CopyStage(work->stages, stages + idx)` crosses the parse/registered
/// split this port introduced, so the whole-struct copy goes through
/// `ShaderStageParse::from(&ShaderStage)` (declared at this file's scope)
/// before the already-ported `R_CopyStage` performs the assignment.
///
/// `work->stages[0]` on a shader whose stage list came out short (the
/// `GeneratePermanentShader` loop `break`s on the first inactive stage) reads
/// the zeroed tail of the `Hunk_Alloc`'d block in the oracle — `Hunk_Alloc`
/// zeroes — which is exactly `ShaderStageParse::default()`, every scratch
/// enum's `Default` being its own zero enumerator.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:4012-4026`
pub fn R_CreateBlendedStage(
    assets: &RenderAssets,
    common: &mut Common,
    state: &mut ShaderParseState,
    handle: i32,
    idx: usize,
) {
    // work = R_GetShaderByHandle(handle);
    let work = R_GetShaderByHandle(assets, common, handle);

    // R_CopyStage(work->stages, stages + idx);
    let orig = assets
        .shaders
        .get(work)
        .and_then(|s| s.stages.first())
        .map(ShaderStageParse::from)
        .unwrap_or_default();
    R_CopyStage(&orig, &mut state.stages[idx]);

    // stages[idx].rgbGen = CGEN_EXACT_VERTEX;
    state.stages[idx].rgb_gen = ColorGen::ExactVertex;
    // stages[idx].alphaGen = AGEN_BLEND;
    state.stages[idx].alpha_gen = AlphaGen::Blend;
    // stages[idx].stateBits = GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_ONE | GLS_DEPTHMASK_TRUE;
    state.stages[idx].state_bits =
        (GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_ONE | GLS_DEPTHMASK_TRUE) as u32;

    if let Some(ss) = state.stages[idx].ss.as_mut() {
        ss.density *= 0.33;
    }
}

/// Raven `ParseDeform`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:1936-2076`
pub fn ParseDeform(
    state: &mut ShaderParseState,
    qs: &mut QSharedScratch,
    text: &mut Option<&[u8]>,
    common: &mut Common,
) {
    let warn = S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII");

    let (token, rest) = COM_ParseExt(qs, *text, false);
    *text = rest;
    if token.is_empty() {
        com_printf(
            common,
            &format!(
                "{}WARNING: missing deform parm in shader '{}'\n",
                warn, state.name
            ),
        );
        return;
    }

    if state.deforms.len() == MAX_SHADER_DEFORMS {
        com_printf(
            common,
            &format!("{}WARNING: MAX_SHADER_DEFORMS in '{}'\n", warn, state.name),
        );
        return;
    }

    // `shader.deforms[shader.numDeforms] = (deformStage_t *)Hunk_Alloc(...); ds
    // = shader.deforms[shader.numDeforms]; shader.numDeforms++;` collapses to
    // a `Vec::push` — the owned `deforms` Vec is this wave's replacement for
    // the fixed `deforms[MAX_SHADER_DEFORMS]` slot array (§C9 out-param ->
    // owned collection; the Vec's own length replaces the separate
    // `numDeforms` counter).
    state.deforms.push(DeformStage::default());
    let idx = state.deforms.len() - 1;

    if token.eq_ignore_ascii_case("projectionShadow") {
        state.deforms[idx].deformation = Deform::ProjectionShadow;
        return;
    }

    if token.eq_ignore_ascii_case("autosprite") {
        state.deforms[idx].deformation = Deform::Autosprite;
        return;
    }

    if token.eq_ignore_ascii_case("autosprite2") {
        state.deforms[idx].deformation = Deform::Autosprite2;
        return;
    }

    if Q_stricmpn(&token, "text", 4) == 0 {
        // n = token[4] - '0' — `token[4]` reads the byte one past a
        // 4-character-matched prefix (the oracle's null-terminated buffer
        // guarantees a defined byte there, `'\0'` when `token` is exactly
        // "text"); the owned `String` has no trailing null, so a missing
        // 5th byte is treated as `0` — same effective result, since
        // `'\0' - '0'` is negative either way and the guard below clamps
        // both cases to `n = 0`.
        let byte4 = token.as_bytes().get(4).copied().unwrap_or(0);
        let mut n = byte4 as i32 - b'0' as i32;
        if n < 0 || n > 7 {
            n = 0;
        }
        state.deforms[idx].deformation = match n {
            0 => Deform::Text0,
            1 => Deform::Text1,
            2 => Deform::Text2,
            3 => Deform::Text3,
            4 => Deform::Text4,
            5 => Deform::Text5,
            6 => Deform::Text6,
            _ => Deform::Text7,
        };
        return;
    }

    if token.eq_ignore_ascii_case("bulge") {
        let (token, rest) = COM_ParseExt(qs, *text, false);
        *text = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing deformVertexes bulge parm in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        state.deforms[idx].bulge_width = atof(&token) as f32;

        let (token, rest) = COM_ParseExt(qs, *text, false);
        *text = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing deformVertexes bulge parm in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        state.deforms[idx].bulge_height = atof(&token) as f32;

        let (token, rest) = COM_ParseExt(qs, *text, false);
        *text = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing deformVertexes bulge parm in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        state.deforms[idx].bulge_speed = atof(&token) as f32;

        state.deforms[idx].deformation = Deform::Bulge;
        return;
    }

    if token.eq_ignore_ascii_case("wave") {
        let (token, rest) = COM_ParseExt(qs, *text, false);
        *text = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing deformVertexes parm in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }

        // wave-0 ruling 12: `atof` returns `double`; `1.0f / atof(token)`
        // promotes `1.0f` to `double` for the division, rounding once at the
        // assignment to the `float` field. The oracle calls `atof(token)`
        // twice (once in the `!= 0` guard, once in the division) — collapsed
        // to a single call here since `atof` is a pure parse of the same
        // token with no side effects (porting-rules §C10, preserve behavior
        // not shape).
        let divisor = atof(&token);
        if divisor != 0.0 {
            state.deforms[idx].deformation_spread = (1.0f64 / divisor) as f32;
        } else {
            state.deforms[idx].deformation_spread = 100.0;
            com_printf(
                common,
                &format!(
                    "{}WARNING: illegal div value of 0 in deformVertexes command for shader '{}'\n",
                    warn, state.name
                ),
            );
        }

        let mut wave = WaveForm::default();
        ParseWaveForm(qs, text, common, state, &mut wave);
        state.deforms[idx].deformation_wave = wave;
        state.deforms[idx].deformation = Deform::Wave;
        return;
    }

    if token.eq_ignore_ascii_case("normal") {
        let (token, rest) = COM_ParseExt(qs, *text, false);
        *text = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing deformVertexes parm in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        state.deforms[idx].deformation_wave.amplitude = atof(&token) as f32;

        let (token, rest) = COM_ParseExt(qs, *text, false);
        *text = rest;
        if token.is_empty() {
            com_printf(
                common,
                &format!(
                    "{}WARNING: missing deformVertexes parm in shader '{}'\n",
                    warn, state.name
                ),
            );
            return;
        }
        state.deforms[idx].deformation_wave.frequency = atof(&token) as f32;

        state.deforms[idx].deformation = Deform::Normals;
        return;
    }

    if token.eq_ignore_ascii_case("move") {
        for i in 0..3usize {
            let (token, rest) = COM_ParseExt(qs, *text, false);
            *text = rest;
            if token.is_empty() {
                com_printf(
                    common,
                    &format!(
                        "{}WARNING: missing deformVertexes parm in shader '{}'\n",
                        warn, state.name
                    ),
                );
                return;
            }
            state.deforms[idx].move_vector[i] = atof(&token) as f32;
        }

        let mut wave = WaveForm::default();
        ParseWaveForm(qs, text, common, state, &mut wave);
        state.deforms[idx].deformation_wave = wave;
        state.deforms[idx].deformation = Deform::Move;
        return;
    }

    com_printf(
        common,
        &format!(
            "{}WARNING: unknown deformVertexes subtype '{}' found in shader '{}'\n",
            warn, token, state.name
        ),
    );
}

/// Raven `FinishShader`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:2941-3275`
pub fn FinishShader(
    assets: &mut RenderAssets,
    common: &mut Common,
    cvars: &RendererCvars,
    state: &mut ShaderParseState,
) -> ShaderHandle {
    let warn = S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII");

    let mut has_lightmap_stage = false;
    // Raven's `vertexLightmap` — set `qfalse` once, never reassigned
    // anywhere in this function (the `rgbGen == CGEN_VERTEX` setter is
    // commented out in the oracle); the `if (vertexLightmap)` branch below
    // is dead but kept for fidelity.
    let vertex_lightmap = false;

    //
    // set sky stuff appropriate
    //
    if state.sky.is_some() {
        state.sort = shaderSort_t::SS_ENVIRONMENT as i32 as f32;
    }

    //
    // set polygon offset
    //
    if state.polygon_offset && state.sort == 0.0 {
        state.sort = shaderSort_t::SS_DECAL as i32 as f32;
    }

    let mut lm_stage = MAX_SHADER_STAGES;
    for i in 0..MAX_SHADER_STAGES {
        if state.stages[i].active && state.stages[i].bundle[0].is_lightmap {
            lm_stage = i;
            break;
        }
    }

    if lm_stage < MAX_SHADER_STAGES {
        if state.lightmap_index[0] == LIGHTMAP_BY_VERTEX {
            if lm_stage == 0 {
                // copy the rest down over the lightmap slot
                for i in lm_stage..MAX_SHADER_STAGES - 1 {
                    state.stages[i] = state.stages[i + 1].clone();
                }
                state.stages[MAX_SHADER_STAGES - 1] = ShaderStageParse::default();
                // change blending on the moved down stage
                state.stages[lm_stage].state_bits = GLS_DEFAULT as u32;
            }
            // change anything that was moved down (or the *white if LM is
            // first) to use vertex color
            state.stages[lm_stage].rgb_gen = ColorGen::ExactVertex;
            state.stages[lm_stage].alpha_gen = AlphaGen::Skip;
            lm_stage = MAX_SHADER_STAGES; // skip the style checking below
        }
    }

    if lm_stage < MAX_SHADER_STAGES {
        let mut num_styles: i32 = 0;
        while (num_styles as usize) < MAXLIGHTMAPS {
            if state.styles[num_styles as usize] >= LS_UNUSED {
                break;
            }
            num_styles += 1;
        }
        num_styles -= 1;

        if num_styles > 0 {
            let n_styles = num_styles as usize;

            let mut i = MAX_SHADER_STAGES - 1;
            while i > lm_stage + n_styles {
                state.stages[i] = state.stages[i - n_styles].clone();
                i -= 1;
            }

            for i in 0..n_styles {
                state.stages[lm_stage + i + 1] = state.stages[lm_stage].clone();
                if state.lightmap_index[i + 1] == LIGHTMAP_BY_VERTEX {
                    state.stages[lm_stage + i + 1].bundle[0].image = assets.white_image;
                } else if state.lightmap_index[i + 1] < 0 {
                    com_error(
                        errorParm_t::ERR_DROP,
                        format!(
                            "FinishShader: light style with no light map or vertex color for shader {}",
                            state.name
                        ),
                    );
                } else {
                    let lm_idx = state.lightmap_index[i + 1] as usize;
                    state.stages[lm_stage + i + 1].bundle[0].image = Some(assets.lightmaps[lm_idx]);
                    state.stages[lm_stage + i + 1].bundle[0].tc_gen = match i {
                        0 => TexCoordGen::Lightmap1,
                        1 => TexCoordGen::Lightmap2,
                        _ => TexCoordGen::Lightmap3,
                    };
                }
                state.stages[lm_stage + i + 1].rgb_gen = ColorGen::LightmapStyle;
                state.stages[lm_stage + i + 1].state_bits &=
                    !((GLS_SRCBLEND_BITS | GLS_DSTBLEND_BITS) as u32);
                state.stages[lm_stage + i + 1].state_bits |=
                    (GLS_SRCBLEND_ONE | GLS_DSTBLEND_ONE) as u32;
            }
        }

        let mut i = 0i32;
        while i <= num_styles {
            state.stages[lm_stage + i as usize].lightmap_style = state.styles[i as usize];
            i += 1;
        }
    }

    //
    // set appropriate stage information
    //
    let mut stage_index: i32 = 0;
    let mut stage: usize = 0;
    while stage < MAX_SHADER_STAGES {
        if !state.stages[stage].active {
            break;
        }

        // check for a missing texture
        if state.stages[stage].bundle[0].image.is_none() {
            com_printf(
                common,
                &format!("{}Shader {} has a stage with no image\n", warn, state.name),
            );
            state.stages[stage].active = false;
            stage += 1;
            continue;
        }

        //
        // ditch this stage if it's detail and detail textures are disabled
        //
        if state.stages[stage].is_detail && common.cvar(cvars.r_detailTextures).integer == 0 {
            if stage < MAX_SHADER_STAGES - 1 {
                for i in stage..MAX_SHADER_STAGES - 1 {
                    state.stages[i] = state.stages[i + 1].clone();
                }
                // rww - 9-13-01 [1-26-01-sof2] — clear the last one moved down
                state.stages[MAX_SHADER_STAGES - 1] = ShaderStageParse::default();
                // Raven's `stage--` here (paired with the mandatory
                // `continue` -> for-loop `stage++`) nets to "leave `stage`
                // unchanged" — re-examine the same index next iteration
                // ("look at this stage next time around").
            } else {
                stage += 1;
            }
            continue;
        }

        state.stages[stage].index = stage_index;

        //
        // default texture coordinate generation
        //
        if state.stages[stage].bundle[0].is_lightmap {
            if state.stages[stage].bundle[0].tc_gen == TexCoordGen::Bad {
                state.stages[stage].bundle[0].tc_gen = TexCoordGen::Lightmap;
            }
            has_lightmap_stage = true;
        } else if state.stages[stage].bundle[0].tc_gen == TexCoordGen::Bad {
            state.stages[stage].bundle[0].tc_gen = TexCoordGen::Texture;
        }

        // not a true lightmap but we want to leave existing behaviour in
        // place and not print out a warning
        // PORT-NOTE: Raven's `rgbGen == CGEN_VERTEX` -> `vertexLightmap =
        // qtrue` setter is commented out in the oracle; stays commented out.

        // Scalar reads hoisted ahead of the field writes below (one `&mut`
        // element at a time); every one is the same value the oracle reads at
        // its own site.
        let blend_bits_mask = (GLS_SRCBLEND_BITS | GLS_DSTBLEND_BITS) as u32;
        let state_bits = state.stages[stage].state_bits;
        let stage0_state_bits = state.stages[0].state_bits;
        let blend_bits = (state_bits & blend_bits_mask) as i32;

        //
        // determine sort order and fog color adjustment
        //
        if blend_bits != 0 && (stage0_state_bits & blend_bits_mask) != 0 {
            let blend_src_bits = (state_bits & GLS_SRCBLEND_BITS as u32) as i32;
            let blend_dst_bits = (state_bits & GLS_DSTBLEND_BITS as u32) as i32;

            // fog color adjustment only works for blend modes that have a
            // contribution that aproaches 0 as the modulate values aproach 0 --
            // GL_ONE, GL_ONE
            // GL_ZERO, GL_ONE_MINUS_SRC_COLOR
            // GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA

            // modulate, additive
            if (blend_src_bits == GLS_SRCBLEND_ONE && blend_dst_bits == GLS_DSTBLEND_ONE)
                || (blend_src_bits == GLS_SRCBLEND_ZERO
                    && blend_dst_bits == GLS_DSTBLEND_ONE_MINUS_SRC_COLOR)
            {
                state.stages[stage].adjust_colors_for_fog = AdjustColorsForFog::ModulateRgb;
            }
            // strict blend
            else if blend_src_bits == GLS_SRCBLEND_SRC_ALPHA
                && blend_dst_bits == GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA
            {
                state.stages[stage].adjust_colors_for_fog = AdjustColorsForFog::ModulateAlpha;
            }
            // premultiplied alpha
            else if blend_src_bits == GLS_SRCBLEND_ONE
                && blend_dst_bits == GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA
            {
                state.stages[stage].adjust_colors_for_fog = AdjustColorsForFog::ModulateRgba;
            } else {
                // we can't adjust this one correctly, so it won't be exactly
                // correct in fog
            }

            // don't screw with sort order if this is a portal or environment
            if state.sort == 0.0 {
                // see through item, like a grill or grate
                if (state_bits & GLS_DEPTHMASK_TRUE as u32) != 0 {
                    state.sort = shaderSort_t::SS_SEE_THROUGH as i32 as f32;
                } else if blend_src_bits == GLS_SRCBLEND_ONE && blend_dst_bits == GLS_DSTBLEND_ONE {
                    // GL_ONE GL_ONE needs to come a bit later
                    state.sort = shaderSort_t::SS_BLEND1 as i32 as f32;
                } else {
                    state.sort = shaderSort_t::SS_BLEND0 as i32 as f32;
                }
                // Raven's commented-out SS_BLEND2/SS_BLEND1/SS_BLEND0 variant
                // of this else-branch stays commented out.
            }
        }

        //rww - begin hw fog
        let is_lightmap = state.stages[stage].bundle[0].is_lightmap;
        let alpha_gen = state.stages[stage].alpha_gen;
        let next_is_lightmap =
            stage < MAX_SHADER_STAGES - 1 && state.stages[stage + 1].bundle[0].is_lightmap;
        state.stages[stage].gl_fog_color_override =
            if blend_bits == (GLS_SRCBLEND_ONE | GLS_DSTBLEND_ONE) {
                FogColorOverride::Black
            } else if blend_bits == (GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_ONE)
                && alpha_gen == AlphaGen::LightingSpecular
                && stage != 0
            {
                FogColorOverride::Black
            } else if blend_bits == (GLS_SRCBLEND_ZERO | GLS_DSTBLEND_ZERO) {
                FogColorOverride::White
            } else if blend_bits == (GLS_SRCBLEND_ONE | GLS_DSTBLEND_ZERO) {
                FogColorOverride::White
            } else if blend_bits == 0 && stage != 0 {
                FogColorOverride::White
            } else if blend_bits == 0 && is_lightmap && next_is_lightmap {
                // multiple light map blending
                FogColorOverride::White
            } else if blend_bits == (GLS_SRCBLEND_DST_COLOR | GLS_DSTBLEND_ZERO) && is_lightmap {
                //I don't know, it works. -rww
                FogColorOverride::White
            } else if blend_bits == (GLS_SRCBLEND_DST_COLOR | GLS_DSTBLEND_ZERO) {
                //I don't know, it works. -rww
                FogColorOverride::Black
            } else if blend_bits == (GLS_SRCBLEND_ONE | GLS_DSTBLEND_ONE_MINUS_SRC_COLOR) {
                //I don't know, it works. -rww
                FogColorOverride::Black
            } else {
                FogColorOverride::None
            };
        //rww - end hw fog
        // Source: oracle/codemp/renderer/tr_shader.cpp:3095-3211

        stage_index += 1;
        stage += 1;
    }

    // there are times when you will need to manually apply a sort to
    // opaque alpha tested shaders that have later blend passes
    if state.sort == 0.0 {
        state.sort = shaderSort_t::SS_OPAQUE as i32 as f32;
    }

    //
    // if we are in r_vertexLight mode, never use a lightmap texture
    //
    if stage > 1
        && common.cvar(cvars.r_vertexLight).integer != 0
        && common.cvar(cvars.r_uiFullScreen).integer == 0
    {
        // Raven: `stage = VertexLightingCollapse();` is commented out
        // ("since this does bad things, I am commenting it out for now").
        has_lightmap_stage = false;
    }

    //
    // look for multitexture potential
    //
    if stage > 1 && CollapseMultitexture(state, assets.glconfig.texture_env_add_available) {
        stage -= 1;
    }

    if state.lightmap_index[0] >= 0 && !has_lightmap_stage {
        if vertex_lightmap {
            // Raven: commented-out `ri.DPrintf` — dead branch, `vertex_lightmap`
            // is never set `true` anywhere in this function.
        } else {
            com_printf(
                common,
                &format!(
                    "WARNING: shader '{}' has lightmap but no lightmap stage!\n",
                    state.name
                ),
            );
            // Source: oracle/codemp/renderer/tr_shader.cpp:3244-3245
            state.lightmap_index = lightmapsNone;
            state.styles = stylesDefault;
        }
    }

    //
    // compute number of passes
    //
    state.num_unfogged_passes = stage as i32;

    // fogonly shaders don't have any normal passes
    if stage == 0 {
        state.sort = shaderSort_t::SS_FOG as i32 as f32;
    }

    // PORT-NOTE: the oracle's final `for ( stage = 1; stage <
    // shader.numUnfoggedPasses; stage++ )` loop (`:3260-3272`) only reads
    // `stages[stage].isDetail`/`.active`/`.bundle[0].isLightmap` and performs
    // no mutation, I/O, or other observable effect — dead code, dropped (same
    // precedent as this file's `ss->vertSkew` no-op note above).

    GeneratePermanentShader(assets, common, state)
}

/// Raven `R_FindServerShader`.
///
/// `mipRawImage` is read nowhere in the oracle body — its presence is kept
/// for call-site fidelity (porting-rules §A2: no speculative behavior
/// removal), unused here (`RE_RegisterShaderFromImage` below has the same
/// property).
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:3560-3596`
pub fn R_FindServerShader(
    assets: &mut RenderAssets,
    common: &mut Common,
    cvars: &RendererCvars,
    name: &str,
    lightmap_index: &[i32],
    styles: &[u8],
    _mip_raw_image: bool,
) -> ShaderHandle {
    if name.is_empty() {
        return ShaderHandle::slot_zero(); // tr.defaultShader
    }

    let stripped_name = COM_StripExtension(name);

    //
    // see if the shader is already loaded
    //
    // NOTE: if there was no shader or image available with the name strippedName
    // then a default shader is created with lightmapIndex == LIGHTMAP_NONE, so we
    // have to check all default shaders otherwise for every call to R_FindShader
    // with that same strippedName a new default shader is created.
    if let Some(candidates) = assets.shader_lookup.get(&stripped_name) {
        for &candidate in candidates {
            if let Some(sh) = assets.shaders.get(candidate) {
                if IsShader(sh, &stripped_name, lightmap_index, styles) {
                    return candidate;
                }
            }
        }
    }

    // clear the global shader
    let mut state = ClearGlobalShader();
    state.name = stripped_name;
    state
        .lightmap_index
        .copy_from_slice(&lightmap_index[..MAXLIGHTMAPS]);
    state.styles.copy_from_slice(&styles[..MAXLIGHTMAPS]);

    state.default_shader = true;
    FinishShader(assets, common, cvars, &mut state)
}

/// Raven `RE_RegisterShaderFromImage`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:3598-3682`
pub fn RE_RegisterShaderFromImage(
    assets: &mut RenderAssets,
    common: &mut Common,
    cvars: &RendererCvars,
    name: &str,
    lightmap_index: &[i32],
    styles: &[u8],
    image: ImageHandle,
    _mip_raw_image: bool,
) -> ShaderHandle {
    // `generateHashValue`/`hashTable` walk -> the same `shader_lookup` bucket
    // walk `R_FindServerShader` above uses; the lookup key is always the
    // *stripped* name (matching `GeneratePermanentShader`'s key derivation),
    // even though — unlike `R_FindServerShader` — this fn stores the raw,
    // unstripped `name` into `shader.name`/compares candidates against it
    // (oracle: `IsShader(sh, name, ...)`, not `IsShader(sh, strippedName,
    // ...)`). That asymmetry is safe: `IsShader` compares the full,
    // unstripped name, so the stripped-key bucket is a superset of the
    // raw-key bucket — every candidate the raw name would reach is walked,
    // and the extra ones are rejected by the same full-name compare the
    // oracle applies. Accept/reject is identical either way.
    let lookup_key = COM_StripExtension(name);

    //
    // see if the shader is already loaded
    //
    // NOTE: if there was no shader or image available with the name strippedName
    // then a default shader is created with lightmapIndex == LIGHTMAP_NONE, so we
    // have to check all default shaders otherwise for every call to R_FindShader
    // with that same strippedName a new default shader is created.
    if let Some(candidates) = assets.shader_lookup.get(&lookup_key) {
        for &candidate in candidates {
            if let Some(sh) = assets.shaders.get(candidate) {
                if IsShader(sh, name, lightmap_index, styles) {
                    return candidate;
                }
            }
        }
    }

    // clear the global shader
    //
    // The oracle open-codes the bare `Com_Memset(&shader…)`/`Com_Memset(
    // &stages…)` pair here rather than calling `ClearGlobalShader`, so
    // `content_flags` stays 0 — the `CONTENTS_SOLID|CONTENTS_OPAQUE` seed is
    // `ClearGlobalShader`'s alone (`tr_shader.cpp:238-245`).
    // Source: oracle/codemp/renderer/tr_shader.cpp:3619-3620
    let mut state = reset_global_shader_bare();
    state.name = name.to_string();
    state
        .lightmap_index
        .copy_from_slice(&lightmap_index[..MAXLIGHTMAPS]);
    state.styles.copy_from_slice(&styles[..MAXLIGHTMAPS]);

    // PORT-NOTE: the oracle's `for (i = 0; i < MAX_SHADER_STAGES; i++)
    // stages[i].bundle[0].texMods = texMods[i];` loop dissolves —
    // `texMods[MAX_SHADER_STAGES]` was a scratch array aliased by pointer
    // into `stages[i].bundle[0].texMods`; `TextureBundleParse::tex_mods` is
    // already its own owned `Vec` per stage (`ShaderParseState`'s doc
    // comment), so there is nothing left to copy from.

    //
    // create the default shading commands
    //
    if state.lightmap_index[0] == LIGHTMAP_NONE {
        // dynamic colors at vertexes
        state.stages[0].bundle[0].image = Some(image);
        state.stages[0].active = true;
        state.stages[0].rgb_gen = ColorGen::LightingDiffuse;
        state.stages[0].state_bits = GLS_DEFAULT as u32;
    } else if state.lightmap_index[0] == LIGHTMAP_BY_VERTEX {
        // explicit colors at vertexes
        state.stages[0].bundle[0].image = Some(image);
        state.stages[0].active = true;
        state.stages[0].rgb_gen = ColorGen::ExactVertex;
        state.stages[0].alpha_gen = AlphaGen::Skip;
        state.stages[0].state_bits = GLS_DEFAULT as u32;
    } else if state.lightmap_index[0] == LIGHTMAP_2D {
        // GUI elements
        state.stages[0].bundle[0].image = Some(image);
        state.stages[0].active = true;
        state.stages[0].rgb_gen = ColorGen::Vertex;
        state.stages[0].alpha_gen = AlphaGen::Vertex;
        state.stages[0].state_bits = (GLS_DEPTHTEST_DISABLE
            | GLS_SRCBLEND_SRC_ALPHA
            | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA) as u32;
    } else if state.lightmap_index[0] == LIGHTMAP_WHITEIMAGE {
        // fullbright level
        state.stages[0].bundle[0].image = assets.white_image;
        state.stages[0].active = true;
        state.stages[0].rgb_gen = ColorGen::IdentityLighting;
        state.stages[0].state_bits = GLS_DEFAULT as u32;

        state.stages[1].bundle[0].image = Some(image);
        state.stages[1].active = true;
        state.stages[1].rgb_gen = ColorGen::Identity;
        state.stages[1].state_bits |= (GLS_SRCBLEND_DST_COLOR | GLS_DSTBLEND_ZERO) as u32;
    } else {
        // two pass lightmap
        //
        // `tr.lightmaps[shader.lightmapIndex[0]]` is unchecked oracle array
        // indexing (UB on an out-of-range index); `.get(...).copied()` picks
        // the one defined behavior — `None` — per porting-rules §19.
        let lm_idx = state.lightmap_index[0] as usize;
        state.stages[0].bundle[0].image = assets.lightmaps.get(lm_idx).copied();
        state.stages[0].bundle[0].is_lightmap = true;
        state.stages[0].active = true;
        // lightmaps are scaled on creation for identitylight
        state.stages[0].rgb_gen = ColorGen::Identity;
        state.stages[0].state_bits = GLS_DEFAULT as u32;

        state.stages[1].bundle[0].image = Some(image);
        state.stages[1].active = true;
        state.stages[1].rgb_gen = ColorGen::Identity;
        state.stages[1].state_bits |= (GLS_SRCBLEND_DST_COLOR | GLS_DSTBLEND_ZERO) as u32;
    }

    FinishShader(assets, common, cvars, &mut state)
}

/// Raven `CreateInternalShaders`.
///
/// PORT-NOTE: `tr.numShaders = 0` is a whole-registry invalidation — the
/// array AND every `hashTable` bucket pointing into it — which the
/// `Arena`-backed registry spells as `Arena::reset` (DEC-42.1). `reset`
/// re-seats slot 0 with a value its caller supplies, and this registry's
/// slot-0 value is the `"<default>"` shader below, which `FinishShader` can
/// only hand back by inserting; so the purge lands right after that first
/// `FinishShader`, which lifts the new default straight back out of the
/// outgoing arena and re-seats it at slot 0 — the oracle's "index 0 is
/// re-created, never re-numbered" (A12), with no second `<default>` copy left
/// in the registry. `sorted_shaders`/`shader_lookup` are then re-registered
/// against `ShaderHandle::slot_zero()`; at that point each holds only the one
/// now-stale pre-reset handle, since this fn clears `sorted_shaders` on entry
/// and `R_InitShaders` — its only caller, here and in the oracle — clears
/// `shader_lookup` immediately before calling it.
///
/// The oracle reuses the SAME file-scope `shader`/`stages` globals across all
/// three `FinishShader` calls below — only `.name`/`.sort`/`.defaultShader`
/// are overwritten between blocks 2 and 3, the reset happens only once, at
/// the top. This transcription reuses the same `state` binding for the same
/// reason (matches oracle exactly, not a simplification).
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:4137-4251`
pub fn CreateInternalShaders(
    assets: &mut RenderAssets,
    common: &mut Common,
    cvars: &RendererCvars,
) {
    // tr.numShaders = 0; — the Arena-backed registry has no explicit counter
    // to reset (R2-D3): the array half of the invalidation is
    // `Arena::reset`, which runs below once the new slot-0 value exists (see
    // doc comment); the `hashTable` half is `R_InitShaders`' own
    // `shader_lookup` clear.
    assets.sorted_shaders.clear();

    // init the default shader
    //
    // Bare `Com_Memset(&shader…)`/`Com_Memset(&stages…)`, not
    // `ClearGlobalShader` — `content_flags` stays 0 here (the
    // `CONTENTS_SOLID|CONTENTS_OPAQUE` seed lives only in
    // `ClearGlobalShader`, `tr_shader.cpp:238-245`).
    // Source: oracle/codemp/renderer/tr_shader.cpp:4141-4142
    let mut state = reset_global_shader_bare();
    state.name = "<default>".to_string();

    // Source: oracle/codemp/renderer/tr_shader.cpp:4146-4147
    state.lightmap_index = lightmapsNone;
    state.styles = stylesDefault;

    // PORT-NOTE: the texMods-copy loop dissolves — see
    // `RE_RegisterShaderFromImage`'s identical note above.

    state.stages[0].bundle[0].image = assets.default_image;
    state.stages[0].active = true;
    state.stages[0].state_bits = GLS_DEFAULT as u32;

    let default_shader = FinishShader(assets, common, cvars, &mut state);

    // tr.numShaders = 0 (cont.) — the registry purge (DEC-42.1): every
    // pre-reset handle goes stale and the `<default>` shader just built takes
    // index 0.
    let default_entry = assets
        .shaders
        .remove(default_shader)
        .expect("FinishShader registered the <default> shader in the arena");
    assets.shaders.reset(default_entry);

    // The two registrations `GeneratePermanentShader` made for the now-stale
    // pre-reset handle, redone against the reserved slot 0.
    assets.sorted_shaders.clear();
    SortNewShader(assets, ShaderHandle::slot_zero());
    assets.shader_lookup.clear();
    assets
        .shader_lookup
        .entry(COM_StripExtension(&state.name))
        .or_default()
        .push(ShaderHandle::slot_zero());

    // tr.defaultShader = ... — no separate field needed: `ShaderHandle::
    // slot_zero()` already IS the live default shader by construction (A12),
    // the convention every other fn in this file uses (`R_FindShaderByName`,
    // `R_GetShaderByHandle`, `GeneratePermanentShader`'s overflow fallback).

    // shadow shader is just a marker
    state.name = "<stencil shadow>".to_string();
    state.sort = shaderSort_t::SS_BANNER as i32 as f32;
    let _shadow_shader = FinishShader(assets, common, cvars, &mut state);
    // tr.shadowShader = ... — DEFERRED: no `RenderAssets`/`FrameState` field
    // homes this handle yet (same "singleton shader pointer" gap as
    // `defaultShader`'s tier-2-audit row); no in-packet consumer reads it.

    // distortion shader is just a marker
    state.name = "internal_distortion".to_string();
    state.sort = shaderSort_t::SS_BLEND0 as i32 as f32;
    state.default_shader = false;
    let _distortion_shader = FinishShader(assets, common, cvars, &mut state);
    state.default_shader = true;
    // tr.distortionShader = ... — DEFERRED, same as shadowShader above.

    // #ifndef _XBOX — GLOWXXX glow vertex/pixel program setup.
    // DEFERRED: R4 — every touched surface (`qglGenProgramsARB`,
    // `qglBindProgramARB`, `qglProgramStringARB`, `qglGetIntegerv`,
    // `qglCombinerParameteriNV`, `qglGenLists`, `qglNewList`,
    // `qglCombinerInputNV`, `qglCombinerOutputNV`, `qglFinalCombinerInputNV`,
    // `qglEndList`) is `qgl*`/`qwgl*` GL/WGL surface — no R3 home, dissolves
    // into the R4 wgpu rewrite (DEC-01/DEC-37 A13.2, packet STATE HOMES row).
    // `g_strGlowVShaderARB`/`g_strGlowPShaderARB` (the ASCII program source
    // byte strings) and `tr.glowVShader`/`tr.glowPShader` (the resulting
    // program-handle fields) are also genuinely absent from this packet (not
    // in FILE-SCOPE CONSTANTS, not in this fn's own oracle slice) —
    // never-guess rule.
    // Source: oracle/codemp/renderer/tr_shader.cpp:4170-4250
}

/// Raven `ParseStage`.
///
/// `#ifdef DEDICATED`/`#else` splits throughout this fn take the `#else`
/// (real-load) leg — `R_FindImageFile` is already a full, already-ported
/// implementation that itself short-circuits at runtime on the
/// `com_dedicated` cvar (`tr_image.rs`), not a compile-time stub, so the
/// non-`DEDICATED` branch is the one that actually reaches live code (same
/// precedent as `R_Splash`, `tr_init.rs`). `#ifdef VV_LIGHTING`
/// (`specularmap`) and `#ifdef _XBOX` (`bumpmap`, and the `_XBOX`-only
/// `needsNormal`/`needsTangent` set-asides inside `lightingDiffuse`/
/// `tcGen environment`) are dropped — MP retail builds neither
/// (established file precedent, e.g. `CollapseMultitexture`/`R_Splash`).
///
/// `continue;` in the oracle's `while(1)` body is, with one exception,
/// behaviorally identical to just letting the enclosing `if`/`else if` arm
/// finish (nothing follows the dispatch chain inside the loop) — transcribed
/// as plain if/else-if fall-through, not literal `continue`. The exception
/// is `blendfunc`: its two early "missing parm" exits skip a trailing
/// `depthMaskBits` clear that sits *after* the token-dispatch chain but
/// still inside the `blendfunc` arm, so those two use a real Rust `continue`
/// to match.
///
/// `Hunk_Alloc`+`memcpy` (`animMap`'s frame-array copy, `tcGen vector`'s
/// `tcGenVectors`) dissolve — `TextureBundleParse::image_animations`/
/// `tc_gen_vectors` are already owned storage, nothing to allocate into
/// (§C9).
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:1279-1920`
pub fn ParseStage<'a>(
    stage: &mut ShaderStageParse,
    text: &mut Option<&'a [u8]>,
    qs: &mut QSharedScratch,
    state: &mut ShaderParseState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    models: &RenderModels,
    img_state: &mut TrImageState,
) -> bool {
    let warn = S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII");

    let mut depth_mask_bits: i32 = GLS_DEPTHMASK_TRUE;
    let mut blend_src_bits: i32 = 0;
    let mut blend_dst_bits: i32 = 0;
    let mut atest_bits: u32 = 0;
    let mut depth_func_bits: i32 = 0;
    let mut depth_mask_explicit = false;

    stage.active = true;

    loop {
        let (token, rest) = COM_ParseExt(qs, *text, true);
        *text = rest;
        if token.is_empty() {
            com_printf(
                view.common,
                &format!("{}WARNING: no matching '}}' found\n", warn),
            );
            return false;
        }

        // Faithful: the oracle tests `token[0]`, the first byte only — an
        // empty token reads its NUL terminator, which matches no brace.
        // Source: oracle/codemp/renderer/tr_shader.cpp:1296
        if token.as_bytes().first() == Some(&b'}') {
            break;
        } else if token.eq_ignore_ascii_case("map") {
            //
            // map <name>
            //
            let (t, rest) = COM_ParseExt(qs, *text, false);
            *text = rest;
            if t.is_empty() {
                com_printf(
                    view.common,
                    &format!(
                        "{}WARNING: missing parameter for 'map' keyword in shader '{}'\n",
                        warn, state.name
                    ),
                );
                return false;
            }

            if t.eq_ignore_ascii_case("$whiteimage") {
                stage.bundle[0].image = assets.white_image;
            } else if t.eq_ignore_ascii_case("$lightmap") {
                stage.bundle[0].is_lightmap = true;
                if state.lightmap_index[0] < 0
                    || state.lightmap_index[0] as usize >= assets.lightmaps.len()
                {
                    // `#ifndef FINAL_BUILD` diagnostic dropped — retail
                    // compiles it out entirely (DEC-37 A13.5, `tr_scene.rs`'s
                    // precedent).
                    // Source: oracle/codemp/renderer/tr_shader.cpp:1322-1324
                    stage.bundle[0].image = assets.white_image;
                } else {
                    stage.bundle[0].image = assets
                        .lightmaps
                        .get(state.lightmap_index[0] as usize)
                        .copied();
                }
            } else {
                let handle = R_FindImageFile(
                    view,
                    cvars,
                    assets,
                    models,
                    img_state,
                    Some(t.as_str()),
                    !state.no_mip_maps,
                    !state.no_pic_mip,
                    !state.no_tc,
                    GL_REPEAT,
                );
                match handle {
                    Some(h) => stage.bundle[0].image = Some(h),
                    None => {
                        com_printf(
                            view.common,
                            &format!(
                                "{}WARNING: R_FindImageFile could not find '{}' in shader '{}'\n",
                                warn, t, state.name
                            ),
                        );
                        return false;
                    }
                }
            }
        } else if token.eq_ignore_ascii_case("clampmap") {
            //
            // clampmap <name>
            //
            let (t, rest) = COM_ParseExt(qs, *text, false);
            *text = rest;
            if t.is_empty() {
                com_printf(
                    view.common,
                    &format!(
                        "{}WARNING: missing parameter for 'clampmap' keyword in shader '{}'\n",
                        warn, state.name
                    ),
                );
                return false;
            }

            let handle = R_FindImageFile(
                view,
                cvars,
                assets,
                models,
                img_state,
                Some(t.as_str()),
                !state.no_mip_maps,
                !state.no_pic_mip,
                !state.no_tc,
                GL_CLAMP,
            );
            match handle {
                Some(h) => stage.bundle[0].image = Some(h),
                None => {
                    com_printf(
                        view.common,
                        &format!(
                            "{}WARNING: R_FindImageFile could not find '{}' in shader '{}'\n",
                            warn, t, state.name
                        ),
                    );
                    return false;
                }
            }
        } else if token.eq_ignore_ascii_case("animMap")
            || token.eq_ignore_ascii_case("clampanimMap")
            || token.eq_ignore_ascii_case("oneshotanimMap")
        {
            //
            // animMap <frequency> <image1> .... <imageN>
            //
            let b_clamp = token.eq_ignore_ascii_case("clampanimMap");
            let one_shot = token.eq_ignore_ascii_case("oneshotanimMap");

            let (freq, rest) = COM_ParseExt(qs, *text, false);
            *text = rest;
            if freq.is_empty() {
                // PORT-NOTE: the oracle's own warning names are swapped —
                // `(bClamp ? "animMap" : "clampanimMap")` prints "animMap"
                // when `bClamp` is true (i.e. the keyword actually parsed
                // WAS "clampanimMap") and vice versa. Transcribed verbatim
                // (porting-rules §A2: port faithfully, even if buggy).
                com_printf(
                    view.common,
                    &format!(
                        "{}WARNING: missing parameter for '{}' keyword in shader '{}'\n",
                        warn,
                        if b_clamp { "animMap" } else { "clampanimMap" },
                        state.name
                    ),
                );
                return false;
            }
            stage.bundle[0].image_animation_speed = atof(&freq) as f32;
            stage.bundle[0].one_shot_anim_map = one_shot;

            // parse up to MAX_IMAGE_ANIMATIONS animations
            let mut images: Vec<ImageHandle> = Vec::new();
            loop {
                let (img_token, rest) = COM_ParseExt(qs, *text, false);
                *text = rest;
                if img_token.is_empty() {
                    break;
                }
                if images.len() < MAX_IMAGE_ANIMATIONS {
                    let handle = R_FindImageFile(
                        view,
                        cvars,
                        assets,
                        models,
                        img_state,
                        Some(img_token.as_str()),
                        !state.no_mip_maps,
                        !state.no_pic_mip,
                        !state.no_tc,
                        if b_clamp { GL_CLAMP } else { GL_REPEAT },
                    );
                    match handle {
                        Some(h) => images.push(h),
                        None => {
                            com_printf(
                                view.common,
                                &format!(
                                    "{}WARNING: R_FindImageFile could not find '{}' in shader '{}'\n",
                                    warn, img_token, state.name
                                ),
                            );
                            return false;
                        }
                    }
                }
            }
            // Copy image ptrs into an array of ptrs — collapses to owning
            // the Vec directly; Hunk_Alloc+memcpy has no owned-Vec
            // counterpart (§C9).
            stage.bundle[0].num_image_animations = images.len() as i16;
            stage.bundle[0].image_animations = images;
        } else if token.eq_ignore_ascii_case("videoMap") {
            let (t, rest) = COM_ParseExt(qs, *text, false);
            *text = rest;
            if t.is_empty() {
                com_printf(
                    view.common,
                    &format!(
                        "{}WARNING: missing parameter for 'videoMap' keyword in shader '{}'\n",
                        warn, state.name
                    ),
                );
                return false;
            }
            // DEFERRED: `CIN_PlayCinematic`/`tr.scratchImage[client]` —
            // `CIN_PlayCinematic` is genuinely unresolved client-side
            // cinematic-playback surface (packet RESOLVED CALL SURFACE:
            // "escalate, never stub"), and `tr.scratchImage` carries no R2
            // carrier (same class as `RE_UploadCinematic`'s own
            // DEFERRED: R4, `tr_backend.rs:444-455`, DEC-37 A13.2).
            // `videoMapHandle` stays `-1` — the oracle's own DEDICATED-build
            // default — so the `if (videoMapHandle != -1)` guard below is
            // faithfully never entered and `isVideoMap`/`image` stay unset.
            // Source: oracle/codemp/renderer/tr_shader.cpp:1444-1461
            stage.bundle[0].video_map_handle = -1;
        } else if token.eq_ignore_ascii_case("alphaFunc") {
            //
            // alphafunc <func>
            //
            let (t, rest) = COM_ParseExt(qs, *text, false);
            *text = rest;
            if t.is_empty() {
                com_printf(
                    view.common,
                    &format!(
                        "{}WARNING: missing parameter for 'alphaFunc' keyword in shader '{}'\n",
                        warn, state.name
                    ),
                );
                return false;
            }
            atest_bits = NameToAFunc(state, view.common, &t);
        } else if token.eq_ignore_ascii_case("depthfunc") {
            //
            // depthFunc <func>
            //
            let (t, rest) = COM_ParseExt(qs, *text, false);
            *text = rest;
            if t.is_empty() {
                com_printf(
                    view.common,
                    &format!(
                        "{}WARNING: missing parameter for 'depthfunc' keyword in shader '{}'\n",
                        warn, state.name
                    ),
                );
                return false;
            }

            if t.eq_ignore_ascii_case("lequal") {
                depth_func_bits = 0;
            } else if t.eq_ignore_ascii_case("equal") {
                depth_func_bits = GLS_DEPTHFUNC_EQUAL;
            } else if t.eq_ignore_ascii_case("disable") {
                depth_func_bits = GLS_DEPTHTEST_DISABLE;
            } else {
                com_printf(
                    view.common,
                    &format!(
                        "{}WARNING: unknown depthfunc '{}' in shader '{}'\n",
                        warn, t, state.name
                    ),
                );
            }
        } else if token.eq_ignore_ascii_case("detail") {
            stage.is_detail = true;
        } else if token.eq_ignore_ascii_case("blendfunc") {
            //
            // blendfunc <srcFactor> <dstFactor>
            // or blendfunc <add|filter|blend>
            //
            let (t, rest) = COM_ParseExt(qs, *text, false);
            *text = rest;
            if t.is_empty() {
                com_printf(
                    view.common,
                    &format!(
                        "{}WARNING: missing parm for blendFunc in shader '{}'\n",
                        warn, state.name
                    ),
                );
                continue;
            }
            // check for "simple" blends first
            if t.eq_ignore_ascii_case("add") {
                blend_src_bits = GLS_SRCBLEND_ONE;
                blend_dst_bits = GLS_DSTBLEND_ONE;
            } else if t.eq_ignore_ascii_case("filter") {
                blend_src_bits = GLS_SRCBLEND_DST_COLOR;
                blend_dst_bits = GLS_DSTBLEND_ZERO;
            } else if t.eq_ignore_ascii_case("blend") {
                blend_src_bits = GLS_SRCBLEND_SRC_ALPHA;
                blend_dst_bits = GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA;
            } else {
                // complex double blends
                blend_src_bits = NameToSrcBlendMode(state, view.common, &t);

                let (t2, rest2) = COM_ParseExt(qs, *text, false);
                *text = rest2;
                if t2.is_empty() {
                    com_printf(
                        view.common,
                        &format!(
                            "{}WARNING: missing parm for blendFunc in shader '{}'\n",
                            warn, state.name
                        ),
                    );
                    continue;
                }
                blend_dst_bits = NameToDstBlendMode(state, view.common, &t2);
            }

            // clear depth mask for blended surfaces
            if !depth_mask_explicit {
                depth_mask_bits = 0;
            }
        } else if token.eq_ignore_ascii_case("rgbGen") {
            //
            // rgbGen
            //
            let (t, rest) = COM_ParseExt(qs, *text, false);
            *text = rest;
            if t.is_empty() {
                com_printf(
                    view.common,
                    &format!(
                        "{}WARNING: missing parameters for rgbGen in shader '{}'\n",
                        warn, state.name
                    ),
                );
            } else if t.eq_ignore_ascii_case("wave") {
                ParseWaveForm(qs, text, view.common, state, &mut stage.rgb_wave);
                stage.rgb_gen = ColorGen::Waveform;
            } else if t.eq_ignore_ascii_case("const") {
                let mut color: [f32; 3] = [0.0; 3];
                ParseVector(qs, text, view.common, &state.name, 3, &mut color);
                stage.constant_color[0] = (255.0 * color[0]) as u8;
                stage.constant_color[1] = (255.0 * color[1]) as u8;
                stage.constant_color[2] = (255.0 * color[2]) as u8;
                stage.rgb_gen = ColorGen::Const;
            } else if t.eq_ignore_ascii_case("identity") {
                stage.rgb_gen = ColorGen::Identity;
            } else if t.eq_ignore_ascii_case("identityLighting") {
                stage.rgb_gen = ColorGen::IdentityLighting;
            } else if t.eq_ignore_ascii_case("entity") {
                stage.rgb_gen = ColorGen::Entity;
            } else if t.eq_ignore_ascii_case("oneMinusEntity") {
                stage.rgb_gen = ColorGen::OneMinusEntity;
            } else if t.eq_ignore_ascii_case("vertex") {
                stage.rgb_gen = ColorGen::Vertex;
                if stage.alpha_gen == AlphaGen::Identity {
                    stage.alpha_gen = AlphaGen::Vertex;
                }
            } else if t.eq_ignore_ascii_case("exactVertex") {
                stage.rgb_gen = ColorGen::ExactVertex;
            } else if t.eq_ignore_ascii_case("lightingDiffuse") {
                stage.rgb_gen = ColorGen::LightingDiffuse;
                // `#ifdef _XBOX shader.needsNormal = true;` dropped — MP
                // retail builds the non-`_XBOX` configuration (established
                // file precedent).
            } else if t.eq_ignore_ascii_case("lightingDiffuseEntity") {
                if state.lightmap_index[0] != LIGHTMAP_NONE {
                    let err = S_COLOR_RED.to_str().expect("S_COLOR_RED is ASCII");
                    com_printf(
                        view.common,
                        &format!(
                            "{}ERROR: rgbGen lightingDiffuseEntity used on a misc_model! in shader '{}'\n",
                            err, state.name
                        ),
                    );
                }
                stage.rgb_gen = ColorGen::LightingDiffuseEntity;
                // `#ifdef _XBOX shader.needsNormal = true;` dropped, as above.
            } else if t.eq_ignore_ascii_case("oneMinusVertex") {
                stage.rgb_gen = ColorGen::OneMinusVertex;
            } else {
                com_printf(
                    view.common,
                    &format!(
                        "{}WARNING: unknown rgbGen parameter '{}' in shader '{}'\n",
                        warn, t, state.name
                    ),
                );
            }
        } else if token.eq_ignore_ascii_case("alphaGen") {
            //
            // alphaGen
            //
            let (t, rest) = COM_ParseExt(qs, *text, false);
            *text = rest;
            if t.is_empty() {
                com_printf(
                    view.common,
                    &format!(
                        "{}WARNING: missing parameters for alphaGen in shader '{}'\n",
                        warn, state.name
                    ),
                );
            } else if t.eq_ignore_ascii_case("wave") {
                ParseWaveForm(qs, text, view.common, state, &mut stage.alpha_wave);
                stage.alpha_gen = AlphaGen::Waveform;
            } else if t.eq_ignore_ascii_case("const") {
                // Faithful: the oracle does not check this token for
                // emptiness before `atof`-ing it.
                let (t2, rest2) = COM_ParseExt(qs, *text, false);
                *text = rest2;
                // `255 * atof(token)` is `int * double` -> `double` in C; the
                // multiply happens at `f64` width, not `f32`.
                stage.constant_color[3] = (255.0f64 * atof(&t2)) as u8;
                stage.alpha_gen = AlphaGen::Const;
            } else if t.eq_ignore_ascii_case("identity") {
                stage.alpha_gen = AlphaGen::Identity;
            } else if t.eq_ignore_ascii_case("entity") {
                stage.alpha_gen = AlphaGen::Entity;
            } else if t.eq_ignore_ascii_case("oneMinusEntity") {
                stage.alpha_gen = AlphaGen::OneMinusEntity;
            } else if t.eq_ignore_ascii_case("vertex") {
                stage.alpha_gen = AlphaGen::Vertex;
            } else if t.eq_ignore_ascii_case("lightingSpecular") {
                stage.alpha_gen = AlphaGen::LightingSpecular;
            } else if t.eq_ignore_ascii_case("oneMinusVertex") {
                stage.alpha_gen = AlphaGen::OneMinusVertex;
            } else if t.eq_ignore_ascii_case("dot") {
                stage.alpha_gen = AlphaGen::Dot;
            } else if t.eq_ignore_ascii_case("oneMinusDot") {
                stage.alpha_gen = AlphaGen::OneMinusDot;
            } else if t.eq_ignore_ascii_case("portal") {
                stage.alpha_gen = AlphaGen::Portal;
                let (t2, rest2) = COM_ParseExt(qs, *text, false);
                *text = rest2;
                if t2.is_empty() {
                    state.portal_range = 256.0;
                    com_printf(
                        view.common,
                        &format!(
                            "{}WARNING: missing range parameter for alphaGen portal in shader '{}', defaulting to 256\n",
                            warn, state.name
                        ),
                    );
                } else {
                    state.portal_range = atof(&t2) as f32;
                }
            } else {
                com_printf(
                    view.common,
                    &format!(
                        "{}WARNING: unknown alphaGen parameter '{}' in shader '{}'\n",
                        warn, t, state.name
                    ),
                );
            }
        } else if token.eq_ignore_ascii_case("texgen") || token.eq_ignore_ascii_case("tcGen") {
            //
            // tcGen <function>
            //
            let (t, rest) = COM_ParseExt(qs, *text, false);
            *text = rest;
            if t.is_empty() {
                com_printf(
                    view.common,
                    &format!(
                        "{}WARNING: missing texgen parm in shader '{}'\n",
                        warn, state.name
                    ),
                );
            } else if t.eq_ignore_ascii_case("environment") {
                stage.bundle[0].tc_gen = TexCoordGen::EnvironmentMapped;
                // `#ifdef _XBOX shader.needsNormal = true;` dropped, as above.
            } else if t.eq_ignore_ascii_case("lightmap") {
                stage.bundle[0].tc_gen = TexCoordGen::Lightmap;
            } else if t.eq_ignore_ascii_case("texture") || t.eq_ignore_ascii_case("base") {
                stage.bundle[0].tc_gen = TexCoordGen::Texture;
            } else if t.eq_ignore_ascii_case("vector") {
                ParseVector(
                    qs,
                    text,
                    view.common,
                    &state.name,
                    3,
                    &mut stage.bundle[0].tc_gen_vectors[0],
                );
                ParseVector(
                    qs,
                    text,
                    view.common,
                    &state.name,
                    3,
                    &mut stage.bundle[0].tc_gen_vectors[1],
                );
                stage.bundle[0].tc_gen = TexCoordGen::Vector;
            } else {
                com_printf(
                    view.common,
                    &format!(
                        "{}WARNING: unknown texgen parm in shader '{}'\n",
                        warn, state.name
                    ),
                );
            }
        } else if token.eq_ignore_ascii_case("tcMod") {
            //
            // tcMod <type> <...>
            //
            let mut buffer = String::new();
            loop {
                let (t, rest) = COM_ParseExt(qs, *text, false);
                *text = rest;
                if t.is_empty() {
                    break;
                }
                buffer.push_str(&t);
                buffer.push(' ');
            }
            ParseTexMod(buffer.as_bytes(), stage, qs, state, view.common);
        } else if token.eq_ignore_ascii_case("depthwrite") {
            depth_mask_bits = GLS_DEPTHMASK_TRUE;
            depth_mask_explicit = true;
        } else if token.eq_ignore_ascii_case("glow") {
            // If this stage has glow...	GLOWXXX
            stage.glow = true;
        } else if token.eq_ignore_ascii_case("surfaceSprites") {
            //
            // surfaceSprites <type> ...
            //
            let mut buffer = String::new();
            loop {
                let (t, rest) = COM_ParseExt(qs, *text, false);
                *text = rest;
                if t.is_empty() {
                    break;
                }
                buffer.push_str(&t);
                buffer.push(' ');
            }
            ParseSurfaceSprites(buffer.as_bytes(), stage, qs, state, view.common);
        } else if Q_stricmpn(&token, "ss", 2) == 0 {
            // <--- NOTE ONLY COMPARING FIRST TWO LETTERS
            //
            // ssFademax <fademax>
            // ssFadescale <fadescale>
            // ssVariance <varwidth> <varheight>
            // ssHangdown
            // ssAnyangle
            // ssFaceup
            // ssWind <wind>
            // ssWindIdle <windidle>
            // ssDuration <duration>
            // ssGrow <growwidth> <growheight>
            // ssWeather
            //
            let param = token.clone();
            let mut buffer = String::new();
            loop {
                let (t, rest) = COM_ParseExt(qs, *text, false);
                *text = rest;
                if t.is_empty() {
                    break;
                }
                buffer.push_str(&t);
                buffer.push(' ');
            }
            ParseSurfaceSpritesOptional(&param, buffer.as_bytes(), stage, qs, state, view.common);
        } else {
            com_printf(
                view.common,
                &format!(
                    "{}WARNING: unknown parameter '{}' in shader '{}'\n",
                    warn, token, state.name
                ),
            );
            return false;
        }
    }

    //
    // if cgen isn't explicitly specified, use either identity or identitylighting
    //
    if stage.rgb_gen == ColorGen::Bad {
        if blend_src_bits == GLS_SRCBLEND_ONE || blend_src_bits == GLS_SRCBLEND_SRC_ALPHA {
            stage.rgb_gen = ColorGen::IdentityLighting;
        } else {
            stage.rgb_gen = ColorGen::Identity;
        }
    }

    //
    // implicitly assume that a GL_ONE GL_ZERO blend mask disables blending
    //
    if blend_src_bits == GLS_SRCBLEND_ONE && blend_dst_bits == GLS_DSTBLEND_ZERO {
        blend_dst_bits = 0;
        blend_src_bits = 0;
        depth_mask_bits = GLS_DEPTHMASK_TRUE;
    }

    // decide which agens we can skip
    if stage.alpha_gen == AlphaGen::Identity
        && (stage.rgb_gen == ColorGen::Identity || stage.rgb_gen == ColorGen::LightingDiffuse)
    {
        stage.alpha_gen = AlphaGen::Skip;
    }

    //
    // compute state bits
    //
    stage.state_bits =
        (depth_mask_bits | blend_src_bits | blend_dst_bits | depth_func_bits) as u32 | atest_bits;

    true
}

/// Raven `ParseSkyParms`.
///
/// `shader.sky = Hunk_Alloc(...)` constructs an owned zeroed `SkyParms` up
/// front, so a malformed `skyParms` shader still reads as a sky shader
/// (`sky.is_some()`) on the early-return paths, matching the oracle's
/// non-null `shader->sky`. The six `R_FindImageFile` calls now store their
/// handles into `SkyParms::outerbox` and reproduce the oracle fallback chain:
/// a face whose file does not load takes the previous face's image, and face
/// 0 takes `tr.defaultImage` (`assets.default_image`). `cloudHeight`
/// stores into `SkyParms::cloud_height` with the default-512 rule. The
/// `#ifdef DEDICATED` arm (`outerbox[i] = NULL`) is not reproduced: this
/// crate is the client renderer, and `R_FindImageFile` already returns `None`
/// on a dedicated server through its own `com_dedicated` guard.
///
/// `Com_sprintf` into a `pathname[MAX_QPATH]` buffer collapses to `format!`
/// (established `char[N]` -> `String` translation, no truncation modeled,
/// same as `R_CreateExtendedName` elsewhere in this file) rather than calling
/// the LAW `Com_sprintf(dest: *mut c_char, ...)` — that signature is
/// raw-pointer C ABI and would require `unsafe`, banned by the
/// interior-safety law.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:2086-2137`
pub fn ParseSkyParms<'a>(
    text: &mut Option<&'a [u8]>,
    qs: &mut QSharedScratch,
    state: &mut ShaderParseState,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    assets: &mut RenderAssets,
    models: &RenderModels,
    img_state: &mut TrImageState,
    sky_view: &mut viewParms_t,
) {
    let warn = S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII");
    const SUF: [&str; 6] = ["rt", "lf", "bk", "ft", "up", "dn"];

    state.sky = Some(SkyParms {
        cloud_height: 0.0,
        outerbox: [None; 6],
    });

    // outerbox
    let (token, rest) = COM_ParseExt(qs, *text, false);
    *text = rest;
    if token.is_empty() {
        com_printf(
            view.common,
            &format!(
                "{}WARNING: 'skyParms' missing parameter in shader '{}'\n",
                warn, state.name
            ),
        );
        return;
    }
    if token != "-" {
        let default_image = assets.default_image;
        let mut outerbox: [Option<ImageHandle>; 6] = [None; 6];
        for (i, suf) in SUF.iter().enumerate() {
            let pathname = format!("{}_{}", token, suf);
            let mut image = R_FindImageFile(
                view,
                cvars,
                assets,
                models,
                img_state,
                Some(pathname.as_str()),
                true,
                true,
                !state.no_tc,
                GL_CLAMP,
            );
            if image.is_none() {
                if i != 0 {
                    // not found, so let's use the previous image
                    image = outerbox[i - 1];
                } else {
                    image = default_image;
                }
            }
            outerbox[i] = image;
        }
        if let Some(sky) = state.sky.as_mut() {
            sky.outerbox = outerbox;
        }
    }

    // cloudheight
    let (token, rest) = COM_ParseExt(qs, *text, false);
    *text = rest;
    if token.is_empty() {
        com_printf(
            view.common,
            &format!(
                "{}WARNING: 'skyParms' missing cloudheight in shader '{}'\n",
                warn, state.name
            ),
        );
        return;
    }
    let mut cloud_height = atof(&token) as f32;
    if cloud_height == 0.0 {
        cloud_height = 512.0;
    }
    if let Some(sky) = state.sky.as_mut() {
        sky.cloud_height = cloud_height;
    }
    // W2-F3: the two cloud tables live on the published registry, so this
    // parse-time writer reaches them through `assets` and the whole
    // registration chain drops its `SkyState` parameter.
    R_InitSkyTexCoords(cloud_height, sky_view, &mut assets.sky_parse);

    // innerbox
    let (token, rest) = COM_ParseExt(qs, *text, false);
    *text = rest;
    if token != "-" {
        com_printf(
            view.common,
            &format!(
                "{}WARNING: in shader '{}' 'skyParms', innerbox is not supported!",
                warn, state.name
            ),
        );
    }
}

/// Raven `ParseShader` — wave 6.
///
/// `#ifdef _XBOX shader.needsNormal = false; shader.needsTangent = false;`
/// dropped — MP retail builds the non-`_XBOX` configuration (established
/// file precedent).
///
/// The stage branch (`token[0] == '{'`) takes `&mut state.stages[s]` out of
/// `state` via `mem::take` before calling `ParseStage`, which also wants the
/// whole `state: &mut ShaderParseState` for its own reads/writes — the
/// disjoint-borrow workaround any caller owning the stage inside the same
/// `ShaderParseState` it passes alongside needs; the taken value is put back
/// immediately after the call, `ShaderStageParse::default()` never observed
/// outside that window.
///
/// DIVERGE (porting-rules §19): the oracle's `stages[MAX_SHADER_STAGES]` is
/// a fixed array with no bounds check on `s` here — a 9th stage block is a
/// silent OOB write (UB). `state.stages` is an owned `Vec`; the defined
/// behavior picked is a warning + shader rejection, mirroring `ParseDeform`'s
/// `MAX_SHADER_DEFORMS` guard already in this file.
///
/// `tr.sunLight[3]` and `tr.sunSurfaceLight` have no R2 carrier (`##
/// State ownership` names `tr.sunDirection`'s home on `FrameState` but not
/// these two) — their tokens are still consumed, in order, so the shared
/// parser cursor stays correct; the parsed values themselves go nowhere
/// (`sun`/`q3map_sun` and `surfacelight`/`q3map_surfacelight` arms below).
/// `a`/`b` (the two angle tokens `sunDirection` is actually built from) are
/// real locals: ruling 12 — `a = a/180*M_PI` and `b = b/180*M_PI` are
/// evaluated in `f64` and truncated back to `f32` at the assignment (`a`/`b`
/// are C `float`s); the `cos`/`sin` calls likewise promote their `float`
/// argument to `double`. (Note on the operand's own width: Raven's fallback
/// `#define M_PI 3.14159265358979323846f` is `float`-suffixed
/// (`oracle/codemp/game/q_shared.h:547-549`) and only takes effect when
/// `math.h` did not already define `M_PI`; which of the two definitions is in
/// scope here — and therefore the promotion's exact width — is a deferred
/// user ruling, so the computation below is left as landed.)
///
/// `SkipRestOfLine` here resolves to the byte-cursor overload
/// (`mp_qshared::shared::com_parse::SkipRestOfLine(qs, Option<&[u8]>) ->
/// Option<&[u8]>`), not the `&str` overload the packet's call-surface table
/// names (`q_string.rs`) — this file's `text` cursor is `Option<&[u8]>`
/// throughout, and `com_parse`'s twin is the exact-signature match already
/// established by `COM_ParseExt`/`SkipBracedSection` in this same module.
///
/// `continue` in the oracle's `while(1)` body is, with one exception,
/// behaviorally identical to letting the enclosing `if`/`else if` arm finish
/// (nothing follows the dispatch chain inside the loop) — transcribed as
/// plain if/else-if fall-through, matching this file's `ParseStage`
/// precedent. The exception is `fogParms`: its "missing parm" warning exits
/// via `continue` *before* the trailing `depthForOpaque` assignment and
/// `SkipRestOfLine` call still inside that arm, so that leg uses a real Rust
/// `continue` to match.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:2300-2554`
#[allow(clippy::too_many_arguments)]
pub fn ParseShader<'a>(
    text: &mut Option<&'a [u8]>,
    qs: &mut QSharedScratch,
    state: &mut ShaderParseState,
    world_load: &mut WorldLoadState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    models: &RenderModels,
    img_state: &mut TrImageState,
    sky_view: &mut viewParms_t,
) -> bool {
    let warn = S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII");
    let mut s: usize = 0;

    let (token, rest) = COM_ParseExt(qs, *text, true);
    *text = rest;
    // Faithful: the oracle tests `token[0]`, the first byte only — an empty
    // token reads its NUL terminator, which is not `{`.
    // Source: oracle/codemp/renderer/tr_shader.cpp:2313
    if token.as_bytes().first() != Some(&b'{') {
        com_printf(
            view.common,
            &format!(
                "{}WARNING: expecting '{{', found '{}' instead in shader '{}'\n",
                warn, token, state.name
            ),
        );
        return false;
    }

    loop {
        let (token, rest) = COM_ParseExt(qs, *text, true);
        *text = rest;
        if token.is_empty() {
            com_printf(
                view.common,
                &format!(
                    "{}WARNING: no concluding '}}' in shader {}\n",
                    warn, state.name
                ),
            );
            return false;
        }

        // end of shader definition
        // Faithful: the oracle tests `token[0]`, the first byte only — an
        // empty token reads its NUL terminator, which matches no brace.
        // Source: oracle/codemp/renderer/tr_shader.cpp:2329
        if token.as_bytes().first() == Some(&b'}') {
            break;
        }
        // stage definition
        // Source: oracle/codemp/renderer/tr_shader.cpp:2333
        else if token.as_bytes().first() == Some(&b'{') {
            if s >= MAX_SHADER_STAGES {
                // DIVERGE (porting-rules §19) — see fn doc above.
                com_printf(
                    view.common,
                    &format!(
                        "{}WARNING: too many stages in shader '{}'\n",
                        warn, state.name
                    ),
                );
                return false;
            }
            let mut current_stage = mem::take(&mut state.stages[s]);
            let ok = ParseStage(
                &mut current_stage,
                text,
                qs,
                state,
                assets,
                view,
                cvars,
                models,
                img_state,
            );
            state.stages[s] = current_stage;
            if !ok {
                return false;
            }
            state.stages[s].active = true;
            // #ifndef _XBOX // GLOWXXX
            if state.stages[s].glow {
                state.has_glow = true;
            }
            s += 1;
        }
        // skip stuff that only the QuakeEdRadient needs
        else if Q_stricmpn(&token, "qer", 3) == 0 {
            *text = SkipRestOfLine(qs, *text);
        }
        // material deprecated as of 11 Jan 01
        // material undeprecated as of 7 May 01 - q3map_material deprecated
        else if token.eq_ignore_ascii_case("material")
            || token.eq_ignore_ascii_case("q3map_material")
        {
            ParseMaterial(state, qs, text, view.common);
        }
        // sun parms
        else if token.eq_ignore_ascii_case("sun") || token.eq_ignore_ascii_case("q3map_sun") {
            // DEFERRED: `tr.sunLight[3]` — see fn doc above. Tokens still
            // consumed to keep the cursor correct; values go nowhere.
            let (_, rest) = COM_ParseExt(qs, *text, false); // sunLight[0]
            *text = rest;
            let (_, rest) = COM_ParseExt(qs, *text, false); // sunLight[1]
            *text = rest;
            let (_, rest) = COM_ParseExt(qs, *text, false); // sunLight[2]
            *text = rest;
            let (_, rest) = COM_ParseExt(qs, *text, false); // intensity
            *text = rest;

            let (token, rest) = COM_ParseExt(qs, *text, false);
            *text = rest;
            // ruling 12: `a/180*M_PI` promotes to `f64`, truncating back to
            // `f32` at the assignment.
            let mut a: f32 = atof(&token) as f32;
            a = (a as f64 / 180.0 * PI) as f32;

            let (token, rest) = COM_ParseExt(qs, *text, false);
            *text = rest;
            let mut b: f32 = atof(&token) as f32;
            b = (b as f64 / 180.0 * PI) as f32;

            world_load.sun_direction[0] = ((a as f64).cos() * (b as f64).cos()) as f32;
            world_load.sun_direction[1] = ((a as f64).sin() * (b as f64).cos()) as f32;
            world_load.sun_direction[2] = (b as f64).sin() as f32;
        }
        // q3map_surfacelight deprecated as of 16 Jul 01
        else if token.eq_ignore_ascii_case("surfacelight")
            || token.eq_ignore_ascii_case("q3map_surfacelight")
        {
            // DEFERRED: `tr.sunSurfaceLight` — see fn doc above.
            let (_, rest) = COM_ParseExt(qs, *text, false);
            *text = rest;
        } else if token.eq_ignore_ascii_case("lightColor") {
            // SP skips this so I'm skipping it here too.
            *text = SkipRestOfLine(qs, *text);
        } else if token.eq_ignore_ascii_case("deformvertexes")
            || token.eq_ignore_ascii_case("deform")
        {
            ParseDeform(state, qs, text, view.common);
        } else if token.eq_ignore_ascii_case("tesssize") {
            *text = SkipRestOfLine(qs, *text);
        } else if token.eq_ignore_ascii_case("clampTime") {
            let (token, rest) = COM_ParseExt(qs, *text, false);
            *text = rest;
            if !token.is_empty() {
                state.clamp_time = atof(&token) as f32;
            }
        }
        // skip stuff that only the q3map needs
        else if Q_stricmpn(&token, "q3map", 5) == 0 {
            *text = SkipRestOfLine(qs, *text);
        }
        // skip stuff that only q3map or the server needs
        else if token.eq_ignore_ascii_case("surfaceParm") {
            ParseSurfaceParm(state, qs, text);
        }
        // no mip maps
        else if token.eq_ignore_ascii_case("nomipmaps") {
            state.no_mip_maps = true;
            state.no_pic_mip = true;
        }
        // no picmip adjustment
        else if token.eq_ignore_ascii_case("nopicmip") {
            state.no_pic_mip = true;
        } else if token.eq_ignore_ascii_case("noglfog") {
            state.fog_pass = FogPass::None;
        }
        // polygonOffset
        else if token.eq_ignore_ascii_case("polygonOffset") {
            state.polygon_offset = true;
        } else if token.eq_ignore_ascii_case("noTC") {
            state.no_tc = true;
        }
        // entityMergable, allowing sprite surfaces from multiple entities
        // to be merged into one batch.  This is a savings for smoke
        // puffs and blood, but can't be used for anything where the
        // shader calcs (not the surface function) reference the entity color or scroll
        else if token.eq_ignore_ascii_case("entityMergable") {
            state.entity_mergable = true;
        }
        // fogParms
        else if token.eq_ignore_ascii_case("fogParms") {
            // `Hunk_Alloc(sizeof(fogParms_t), h_low)` dissolves — constructs
            // the owned value directly instead (§C9, `ParseDeform`/
            // `ParseStage` precedent elsewhere in this file).
            state.fog_parms = Some(FogParms::default());
            if !ParseVector(
                qs,
                text,
                view.common,
                &state.name,
                3,
                &mut state.fog_parms.as_mut().expect("just set above").color,
            ) {
                return false;
            }

            let (token, rest) = COM_ParseExt(qs, *text, false);
            *text = rest;
            if token.is_empty() {
                com_printf(
                    view.common,
                    &format!(
                        "{}WARNING: missing parm for 'fogParms' keyword in shader '{}'\n",
                        warn, state.name
                    ),
                );
                continue;
            }
            state
                .fog_parms
                .as_mut()
                .expect("just set above")
                .depth_for_opaque = atof(&token) as f32;

            // skip any old gradient directions
            *text = SkipRestOfLine(qs, *text);
        }
        // portal
        else if token.eq_ignore_ascii_case("portal") {
            state.sort = shaderSort_t::SS_PORTAL as i32 as f32;
        }
        // skyparms <cloudheight> <outerbox> <innerbox>
        else if token.eq_ignore_ascii_case("skyparms") {
            ParseSkyParms(
                text, qs, state, view, cvars, assets, models, img_state, sky_view,
            );
        }
        // light <value> determines flaring in q3map, not needed here
        else if token.eq_ignore_ascii_case("light") {
            let (_, rest) = COM_ParseExt(qs, *text, false);
            *text = rest;
        }
        // cull <face>
        else if token.eq_ignore_ascii_case("cull") {
            let (token, rest) = COM_ParseExt(qs, *text, false);
            *text = rest;
            if token.is_empty() {
                com_printf(
                    view.common,
                    &format!(
                        "{}WARNING: missing cull parms in shader '{}'\n",
                        warn, state.name
                    ),
                );
            } else if token.eq_ignore_ascii_case("none")
                || token.eq_ignore_ascii_case("twosided")
                || token.eq_ignore_ascii_case("disable")
            {
                state.cull_type = CullType::TwoSided;
            } else if token.eq_ignore_ascii_case("back")
                || token.eq_ignore_ascii_case("backside")
                || token.eq_ignore_ascii_case("backsided")
            {
                state.cull_type = CullType::BackSided;
            } else {
                com_printf(
                    view.common,
                    &format!(
                        "{}WARNING: invalid cull parm '{}' in shader '{}'\n",
                        warn, token, state.name
                    ),
                );
            }
        }
        // sort
        else if token.eq_ignore_ascii_case("sort") {
            ParseSort(state, qs, text, view.common);
        } else {
            com_printf(
                view.common,
                &format!(
                    "{}WARNING: unknown general shader parameter '{}' in '{}'\n",
                    warn, token, state.name
                ),
            );
            return false;
        }
    }

    //
    // ignore shaders that don't have any stages, unless it is a sky or fog
    //
    if s == 0 && state.sky.is_none() && (state.content_flags & CONTENTS_FOG) == 0 {
        return false;
    }

    state.explicitly_defined = true;

    true
}

/// Raven `R_FindShader` — wave 7 (topological leaf: every in-module callee
/// below landed in a lower wave).
///
/// `#ifdef DEDICATED`/`#else` split (`:3491-3501`): takes the `#else`
/// (real-load) leg unconditionally, same precedent as `ParseStage`/
/// `R_Splash` in this file — `R_FindImageFile` is already a full,
/// already-ported implementation that itself short-circuits at runtime on
/// the `com_dedicated` cvar, not a compile-time stub, so calling it
/// uniformly reproduces both builds' observable behavior (dedicated: `image`
/// comes back `None`, falls into the "couldn't find image" branch below,
/// same net `default_shader = true` result as the oracle's compiled-out
/// `#ifdef DEDICATED` early return — the one difference is this leg also
/// emits the `Com_DPrintf` the `DEDICATED` build never compiles, a
/// diagnostic-only divergence).
///
/// `COM_StripExtension(name, fileName)` (`:3490`) is not re-run as a second
/// call: it is the same pure function applied to the same `name` already
/// captured in `stripped_name` above (porting-rules §C10, preserve behavior
/// not shape).
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:3428-3557`
#[allow(clippy::too_many_arguments)]
pub fn R_FindShader(
    name: &str,
    lightmap_index: &[i32],
    styles: &[u8],
    mip_raw_image: bool,
    qs: &mut QSharedScratch,
    world_load: &mut WorldLoadState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    models: &RenderModels,
    img_state: &mut TrImageState,
    sky_view: &mut viewParms_t,
) -> ShaderHandle {
    if name.is_empty() {
        return ShaderHandle::slot_zero(); // tr.defaultShader
    }

    // use (fullbright) vertex lighting if the bsp file doesn't have
    // lightmaps
    //
    // `tr.numLightmaps` is `RenderAssets::lightmaps.len()` (that Vec is the
    // owned form of `tr.lightmaps[MAX_LIGHTMAPS]` + its count).
    // Source: oracle/codemp/renderer/tr_shader.cpp:3441-3446
    let lightmap_index: &[i32] =
        if lightmap_index[0] >= 0 && lightmap_index[0] >= assets.lightmaps.len() as i32 {
            &lightmapsVertex
        } else {
            lightmap_index
        };

    let stripped_name = COM_StripExtension(name);

    // see if the shader is already loaded
    //
    // NOTE: if there was no shader or image available with the name strippedName
    // then a default shader is created with lightmapIndex == LIGHTMAP_NONE, so we
    // have to check all default shaders otherwise for every call to R_FindShader
    // with that same strippedName a new default shader is created.
    if let Some(candidates) = assets.shader_lookup.get(&stripped_name) {
        for &candidate in candidates {
            if let Some(sh) = assets.shaders.get(candidate) {
                if IsShader(sh, &stripped_name, lightmap_index, styles) {
                    return candidate;
                }
            }
        }
    }

    // clear the global shader
    let mut state = ClearGlobalShader();
    state.name = stripped_name.clone();
    state
        .lightmap_index
        .copy_from_slice(&lightmap_index[..MAXLIGHTMAPS]);
    state.styles.copy_from_slice(&styles[..MAXLIGHTMAPS]);

    //
    // attempt to define shader from an explicit parameter file
    //
    if let Some(shader_text) = FindShaderInShaderText(assets, qs, &stripped_name) {
        let text_bytes = latin1_encode(&shader_text);
        let mut cursor: Option<&[u8]> = Some(text_bytes.as_slice());
        if !ParseShader(
            &mut cursor,
            qs,
            &mut state,
            world_load,
            assets,
            view,
            cvars,
            models,
            img_state,
            sky_view,
        ) {
            // had errors, so use default shader
            state.default_shader = true;
        }
        let sh = FinishShader(assets, view.common, cvars, &mut state);
        return sh;
    }

    //
    // if not defined in the in-memory shader descriptions,
    // look for a single TGA, BMP, or PCX
    //
    // (`fileName` collapses to `stripped_name` — see fn doc above)
    let gl_wrap_clamp_mode = if mip_raw_image { GL_REPEAT } else { GL_CLAMP };
    let image = R_FindImageFile(
        view,
        cvars,
        assets,
        models,
        img_state,
        Some(stripped_name.as_str()),
        mip_raw_image,
        mip_raw_image,
        true,
        gl_wrap_clamp_mode,
    );
    let image = match image {
        Some(image) => image,
        None => {
            Com_DPrintf(
                view.common,
                &format!(
                    "{}Couldn't find image for shader {}\n",
                    S_COLOR_RED.to_str().expect("S_COLOR_RED is ASCII"),
                    name
                ),
            );
            state.default_shader = true;
            return FinishShader(assets, view.common, cvars, &mut state);
        }
    };

    //
    // create the default shading commands
    //
    if state.lightmap_index[0] == LIGHTMAP_NONE {
        // dynamic colors at vertexes
        state.stages[0].bundle[0].image = Some(image);
        state.stages[0].active = true;
        state.stages[0].rgb_gen = ColorGen::LightingDiffuse;
        state.stages[0].state_bits = GLS_DEFAULT as u32;
        // `#ifdef _XBOX shader.needsNormal = true;` (`:3511-3513`) — Xbox-only
        // dead surface on the PC build this port targets (porting-rules §20).
    } else if state.lightmap_index[0] == LIGHTMAP_BY_VERTEX {
        // explicit colors at vertexes
        state.stages[0].bundle[0].image = Some(image);
        state.stages[0].active = true;
        state.stages[0].rgb_gen = ColorGen::ExactVertex;
        state.stages[0].alpha_gen = AlphaGen::Skip;
        state.stages[0].state_bits = GLS_DEFAULT as u32;
    } else if state.lightmap_index[0] == LIGHTMAP_2D {
        // GUI elements
        state.stages[0].bundle[0].image = Some(image);
        state.stages[0].active = true;
        state.stages[0].rgb_gen = ColorGen::Vertex;
        state.stages[0].alpha_gen = AlphaGen::Vertex;
        state.stages[0].state_bits = (GLS_DEPTHTEST_DISABLE
            | GLS_SRCBLEND_SRC_ALPHA
            | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA) as u32;
    } else if state.lightmap_index[0] == LIGHTMAP_WHITEIMAGE {
        // fullbright level
        state.stages[0].bundle[0].image = assets.white_image;
        state.stages[0].active = true;
        state.stages[0].rgb_gen = ColorGen::IdentityLighting;
        state.stages[0].state_bits = GLS_DEFAULT as u32;

        state.stages[1].bundle[0].image = Some(image);
        state.stages[1].active = true;
        state.stages[1].rgb_gen = ColorGen::Identity;
        state.stages[1].state_bits |= (GLS_SRCBLEND_DST_COLOR | GLS_DSTBLEND_ZERO) as u32;
    } else {
        // two pass lightmap
        let lm_idx = state.lightmap_index[0] as usize;
        state.stages[0].bundle[0].image = Some(assets.lightmaps[lm_idx]);
        state.stages[0].bundle[0].is_lightmap = true;
        state.stages[0].active = true;
        // lightmaps are scaled on creation for identitylight
        state.stages[0].rgb_gen = ColorGen::Identity;
        state.stages[0].state_bits = GLS_DEFAULT as u32;

        state.stages[1].bundle[0].image = Some(image);
        state.stages[1].active = true;
        state.stages[1].rgb_gen = ColorGen::Identity;
        state.stages[1].state_bits |= (GLS_SRCBLEND_DST_COLOR | GLS_DSTBLEND_ZERO) as u32;
    }

    FinishShader(assets, view.common, cvars, &mut state)
}

/// Raven `RE_RegisterShaderLightMap`.
///
/// `lightmapIndex`/`styles` are caller-supplied parameters here, where
/// `RE_RegisterShader`/`RE_RegisterShaderNoMip` below pass the fixed
/// file-scope `lightmaps2d`/`stylesDefault` tables; the three bodies are
/// otherwise identical.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:3696-3717`
#[allow(clippy::too_many_arguments)]
pub fn RE_RegisterShaderLightMap(
    name: &str,
    lightmap_index: &[i32],
    styles: &[u8],
    qs: &mut QSharedScratch,
    world_load: &mut WorldLoadState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    models: &RenderModels,
    img_state: &mut TrImageState,
    sky_view: &mut viewParms_t,
) -> i32 {
    if name.len() >= MAX_QPATH as usize {
        com_printf(view.common, "Shader name exceeds MAX_QPATH\n");
        return 0;
    }

    let sh = R_FindShader(
        name,
        lightmap_index,
        styles,
        true,
        qs,
        world_load,
        assets,
        view,
        cvars,
        models,
        img_state,
        sky_view,
    );

    // we want to return 0 if the shader failed to
    // load for some reason, but R_FindShader should
    // still keep a name allocated for it, so if
    // something calls RE_RegisterShader again with
    // the same name, we don't try looking for it again
    match assets.shaders.get(sh) {
        Some(shader) if !shader.default_shader => sh.index() as i32,
        _ => 0,
    }
}

/// Raven `RE_RegisterShader`.
///
/// Raven: This is the exported shader entry point for the rest of the system.
/// It will always return an index that will be valid.
///
/// Raven: This should really only be used for explicit shaders, because there
/// is no way to ask for different implicit lighting modes (vertex, lightmap,
/// etc).
///
/// Structurally identical to `RE_RegisterShaderLightMap` above; the only
/// differences are the fixed `lightmaps2d`/`stylesDefault` arguments.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:3731-3751`
#[allow(clippy::too_many_arguments)]
pub fn RE_RegisterShader(
    name: &str,
    qs: &mut QSharedScratch,
    world_load: &mut WorldLoadState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    models: &RenderModels,
    img_state: &mut TrImageState,
    sky_view: &mut viewParms_t,
) -> i32 {
    if name.len() >= MAX_QPATH as usize {
        com_printf(view.common, "Shader name exceeds MAX_QPATH\n");
        return 0;
    }

    let sh = R_FindShader(
        name,
        &lightmaps2d,
        &stylesDefault,
        true,
        qs,
        world_load,
        assets,
        view,
        cvars,
        models,
        img_state,
        sky_view,
    );

    // we want to return 0 if the shader failed to
    // load for some reason, but R_FindShader should
    // still keep a name allocated for it, so if
    // something calls RE_RegisterShader again with
    // the same name, we don't try looking for it again
    match assets.shaders.get(sh) {
        Some(shader) if !shader.default_shader => sh.index() as i32,
        _ => 0,
    }
}

/// Raven `RE_RegisterShaderNoMip`.
///
/// Raven: For menu graphics that should never be picmiped.
///
/// Identical to `RE_RegisterShader` above except for the `mipRawImage`
/// argument to `R_FindShader` (`qfalse` here vs `qtrue` there).
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:3761-3781`
#[allow(clippy::too_many_arguments)]
pub fn RE_RegisterShaderNoMip(
    name: &str,
    qs: &mut QSharedScratch,
    world_load: &mut WorldLoadState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    models: &RenderModels,
    img_state: &mut TrImageState,
    sky_view: &mut viewParms_t,
) -> i32 {
    if name.len() >= MAX_QPATH as usize {
        com_printf(view.common, "Shader name exceeds MAX_QPATH\n");
        return 0;
    }

    let sh = R_FindShader(
        name,
        &lightmaps2d,
        &stylesDefault,
        false,
        qs,
        world_load,
        assets,
        view,
        cvars,
        models,
        img_state,
        sky_view,
    );

    // we want to return 0 if the shader failed to
    // load for some reason, but R_FindShader should
    // still keep a name allocated for it, so if
    // something calls RE_RegisterShader again with
    // the same name, we don't try looking for it again
    match assets.shaders.get(sh) {
        Some(shader) if !shader.default_shader => sh.index() as i32,
        _ => 0,
    }
}

/// Raven `CreateExternalShaders` — wave 8.
///
/// This finds the `projectionShadow` and `sun` shaders and homes both handles
/// on `RenderAssets` (`projection_shadow_shader`, `sun_shader`). It also sets
/// the projection-shadow shader's sort to `SS_STENCIL_SHADOW`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:4253-4257`
#[allow(clippy::too_many_arguments)]
pub fn CreateExternalShaders(
    qs: &mut QSharedScratch,
    world_load: &mut WorldLoadState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    models: &RenderModels,
    img_state: &mut TrImageState,
    sky_view: &mut viewParms_t,
) {
    let projection_shadow = R_FindShader(
        "projectionShadow",
        &lightmapsNone,
        &stylesDefault,
        true,
        qs,
        world_load,
        assets,
        view,
        cvars,
        models,
        img_state,
        sky_view,
    );
    assets.projection_shadow_shader = projection_shadow;
    // tr.projectionShadowShader->sort = SS_STENCIL_SHADOW;
    if let Some(shader) = assets.shaders.get_mut(projection_shadow) {
        shader.sort = shaderSort_t::SS_STENCIL_SHADOW as i32 as f32;
    }
    assets.sun_shader = R_FindShader(
        "sun",
        &lightmapsNone,
        &stylesDefault,
        true,
        qs,
        world_load,
        assets,
        view,
        cvars,
        models,
        img_state,
        sky_view,
    );
}

/// Raven `R_RemapShader` — wave 9.
///
/// DEFERRED, whole-fn loud stub — but no longer field-blocked. The
/// `lightmapsNone`/`stylesDefault` arguments to `RE_RegisterShaderLightMap`
/// (`:281`, `:291`) landed as file-scope consts above, and campaign #41
/// batch 1 landed both destination fields on `ShaderAsset`
/// (`render_state/shader_asset.rs`):
/// - `remapped_shader: Option<ShaderHandle>` — `sh->remappedShader = sh2;`/
///   `= NULL;` (`:307,309`), Raven `struct shader_s *remappedShader`
///   (`oracle/codemp/renderer/tr_local.h:528`), under the tier-2 transition
///   audit's Group 2 self-pointer -> handle row; and
/// - `time_offset: f32` — `sh2->timeOffset = atof(timeOffset);` (`:314`),
///   Raven `float timeOffset` (`oracle/codemp/renderer/tr_local.h:511`).
///
/// What is left is the transcription itself — the two `R_FindShaderByName`/
/// `RE_RegisterShaderLightMap` lookups plus the `hashTable[hash]` chain walk
/// (`:304-312`), representable via `RenderAssets::shader_lookup`'s
/// stripped-name bucket per this packet's STATE HOMES row — which is a
/// follow-up port, not a field gap.
///
/// `sh == NULL || sh == tr.defaultShader` collapses to
/// `sh == ShaderHandle::slot_zero()` once implemented (`R_FindShaderByName`'s
/// existing doc comment: slot zero already IS the live default shader by
/// construction, A12) — noted here for whichever wave finishes this stub.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:273-316`
#[allow(clippy::too_many_arguments)]
pub fn R_RemapShader(
    shader_name: &str,
    new_shader_name: &str,
    time_offset: Option<&str>,
    qs: &mut QSharedScratch,
    world_load: &mut WorldLoadState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    models: &RenderModels,
    img_state: &mut TrImageState,
    sky_view: &mut viewParms_t,
) {
    let _ = (
        shader_name,
        new_shader_name,
        time_offset,
        qs,
        world_load,
        assets,
        view,
        cvars,
        models,
        img_state,
        sky_view,
    );
    // DEFERRED: body not transcribed — the destination fields exist now, the
    // lookup/hash-chain walk does not. See doc comment above.
    todo!(
        "Port R_RemapShader — oracle/codemp/renderer/tr_shader.cpp:273-316 (lookup + hashTable chain walk not transcribed)"
    );
}

/// Raven `R_MergeShaders` — wave 9.
///
/// DEFERRED, whole-fn loud stub — but no longer stage-shape-blocked. The
/// `lightmapsVertex`/`stylesDefault` `memcpy` sources (`:4039-4040`) are real
/// file-scope consts above; `ShaderStage` is complete and
/// `GeneratePermanentShader` now populates a registered shader's `stages`, so
/// `R_CopyStage(work->stages, stages)` (`:4046`), the `work->stages[i].ss`
/// reads (`:4062,4074,4086`) and the two further `R_CreateBlendedStage` calls
/// (`:4053-4054`, itself ported) all have live payloads.
///
/// What stands is this fn's own body, which was never transcribed: the
/// `shader`/`stages` file-scope scratch reset the oracle performs inline
/// before each `RE_RegisterShaderLightMap` pass, the `current`/`i` pass
/// bookkeeping, and:
/// - `R_SyncRenderThread();` (`:4034`) — no renderer-thread sync exists in
///   this single-threaded port (threading is out of scope for this repo), so
///   it is dropped rather than transcribed.
/// - `shader.multitextureEnv = work->multitextureEnv;` (`:4050`, Raven's
///   "jic") — a plain `ShaderParseState`/`ShaderAsset` field copy that lands
///   with the surrounding pass-0 setup.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:4028-4098`
#[allow(clippy::too_many_arguments)]
pub fn R_MergeShaders(
    blended_name: &str,
    a: i32,
    b: i32,
    c: i32,
    surface_sprites: bool,
    assets: &mut RenderAssets,
    common: &mut Common,
    cvars: &RendererCvars,
) -> ShaderHandle {
    let _ = (
        blended_name,
        a,
        b,
        c,
        surface_sprites,
        assets,
        common,
        cvars,
    );
    // DEFERRED: body not transcribed — the multi-pass `shader`/`stages`
    // scratch reset and pass bookkeeping; `R_SyncRenderThread()` (`:4034`)
    // is dropped outright. See doc comment above.
    todo!(
        "Port R_MergeShaders — oracle/codemp/renderer/tr_shader.cpp:4028-4098 (multi-pass shader/stages scratch reset + pass bookkeeping not transcribed; R_SyncRenderThread dropped)"
    );
}

/// Raven `R_InitShaders` — wave 9.
///
/// `#ifndef DEDICATED` wraps the `if (!server)` block, but this packet's own
/// wave-partition call graph places `CreateInternalShaders`/
/// `ScanAndLoadShaderFiles`/`CreateExternalShaders` as real dependency edges
/// of this fn (THREADING DIGEST "in-module callees (wave < 9)") — the three
/// have no other caller anywhere in this crate. Taken as settled: unlike the
/// `Hunk_Clear`/`R_Register` precedent (`crates/mp/engine/qcommon/src/
/// z_memman_pc.rs:808-811`, `tr_init.rs:551-556`, files genuinely shared
/// between the dedicated and client builds), `tr_shader.cpp` is client-only
/// source the dedicated server never compiles in the first place, so the
/// guard is vestigial here; `server` (Raven `qboolean server`) is
/// transcribed as a real runtime `bool` gate, matching the oracle's own
/// `if (!server)` check.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:4265-4283`
#[allow(clippy::too_many_arguments)]
pub fn R_InitShaders(
    server: bool,
    qs: &mut QSharedScratch,
    world_load: &mut WorldLoadState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    models: &RenderModels,
    img_state: &mut TrImageState,
    sky_view: &mut viewParms_t,
) {
    //Com_Printf ("Initializing Shaders\n" );

    // Com_Memset(hashTable, 0, sizeof(hashTable));
    //
    // PORT-NOTE: the threading digest classifies `hashTable` as
    // read-only for this fn (a tooling artifact of the `Com_Memset(hashTable,
    // …)` address-of-array pattern) but the oracle body clearly zeroes it;
    // transcribed as the write it observably is (porting-rules §A1
    // behavioral parity) — `shader_lookup` is the STATE HOMES-assigned
    // carrier for `hashTable` (this packet, `R_RemapShader`/`R_InitShaders`
    // rows). The array half of the same invalidation is `Arena::reset`
    // (DEC-42.1), owned by `CreateInternalShaders` below, so this clear
    // precedes it and leaves no stale handle behind.
    assets.shader_lookup.clear();

    // deferLoad = qfalse;
    assets.defer_load = false;

    // #ifndef DEDICATED
    if !server {
        CreateInternalShaders(assets, view.common, cvars);

        ScanAndLoadShaderFiles(assets, qs, view, "shaders");

        CreateExternalShaders(
            qs, world_load, assets, view, cvars, models, img_state, sky_view,
        );
    }
    // #endif
}

/// Raven `R_CreateBlendedShader` — wave 10.
///
/// `Com_sprintf(blendedName, MAX_QPATH, "blend(%d,%d,%d)", a, b, c)` collapses
/// to `format!` (established `char[N]` -> `String` translation, no truncation
/// modeled — same precedent as `ParseSkyParms`/`R_CreateExtendedName` above:
/// the LAW raw-pointer `Com_sprintf(dest: *mut c_char, ...)` signature would
/// require `unsafe`, banned by the interior-safety law); `strcat` ->
/// `String::push_str`.
///
/// The `hashTable[generateHashValue(extendedName, FILE_HASH_SIZE)]` bucket
/// walk + `Q_stricmp(work->name, extendedName) == 0` compare is the same
/// `RenderAssets::shader_lookup` name-keyed lookup `R_FindShader`/
/// `RE_RegisterShaderFromImage` above use (`generateHashValue` deliberately
/// not reproduced, same precedent as `GeneratePermanentShader`); the lookup
/// key is `COM_StripExtension`'d (`GeneratePermanentShader`'s own insertion
/// key, `:2774`), candidates compared against the full, unstripped
/// `extendedName` via `eq_ignore_ascii_case` — identical asymmetry to
/// `RE_RegisterShaderFromImage`'s doc comment above (the stripped-key bucket
/// is a superset of the exact-key bucket; the full-name compare rejects the
/// extras).
///
/// `R_CreateExtendedName(extendedName, blendedName, lightmapsVertex,
/// stylesDefault)` (`:4117`) hits the `lightmapIndex == lightmapsVertex`
/// address-identity arm (`:195-198`), which appends `"_vertex"` — so the
/// mode passed here is `LightmapNameMode::Vertex` and the extended name is
/// `blend(a,b,c)[noSS]_vertex`.
///
/// Panics via `R_MergeShaders`'s loud stub until its owning wave lands (see
/// that fn's doc comment) whenever no existing blended shader is found.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:4103-4130`
pub fn R_CreateBlendedShader(
    a: i32,
    b: i32,
    c: i32,
    surface_sprites: bool,
    assets: &mut RenderAssets,
    common: &mut Common,
    cvars: &RendererCvars,
) -> ShaderHandle {
    let mut blended_name = format!("blend({},{},{})", a, b, c);
    if !surface_sprites {
        blended_name.push_str("noSS");
    }

    // Find if this shader has already been created
    let extended_name = R_CreateExtendedName(&blended_name, Some(LightmapNameMode::Vertex));
    let lookup_key = COM_StripExtension(&extended_name);
    if let Some(candidates) = assets.shader_lookup.get(&lookup_key) {
        for &candidate in candidates {
            if let Some(work) = assets.shaders.get(candidate) {
                if work.name.eq_ignore_ascii_case(&extended_name) {
                    return candidate;
                }
            }
        }
    }

    // Create new shader if it doesn't already exist
    R_MergeShaders(
        &extended_name,
        a,
        b,
        c,
        surface_sprites,
        assets,
        common,
        cvars,
    )
}
