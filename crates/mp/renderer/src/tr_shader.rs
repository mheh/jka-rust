//! Raven `tr_shader.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_shader.cpp`

// Raven's own dead stores are transcribed as written (porting-rules §A2/§C10).
#![allow(unused_assignments)]
// Raven-named functions/types keep their original casing across this
// transcription, matching the rest of the renderer/engine crates.
#![allow(non_snake_case)]

use mp_engine_qcommon::cm::cm_shader_consts::MAX_SHADER_FILES;
use mp_engine_qcommon::cmd_common::Cmd_Argc;
use mp_engine_qcommon::common::common::{com_printf, Common};
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::error::com_error;
use mp_engine_qcommon::files_common::{FS_ListFiles, FS_ReadFileVec};
use mp_engine_qcommon::qfiles::draw_vert_t::MAXLIGHTMAPS;
use mp_qshared::shared::com_parse::{COM_ParseExt, QSharedScratch, SkipBracedSection};
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::q_color::S_COLOR_YELLOW;
use mp_qshared::shared::q_string::COM_StripExtension;
use mp_qshared::shared::surface_flags::{
    CONTENTS_ABSEIL, CONTENTS_BOTCLIP, CONTENTS_DETAIL, CONTENTS_FOG, CONTENTS_INSIDE,
    CONTENTS_LADDER, CONTENTS_LAVA, CONTENTS_MONSTERCLIP, CONTENTS_NODROP, CONTENTS_OPAQUE,
    CONTENTS_OUTSIDE, CONTENTS_PLAYERCLIP, CONTENTS_SHOTCLIP, CONTENTS_SLIME, CONTENTS_SOLID,
    CONTENTS_TERRAIN, CONTENTS_TRANSLUCENT, CONTENTS_TRIGGER, CONTENTS_WATER, MATERIALS,
    SURF_FORCEFIELD, SURF_METALSTEPS, SURF_NODAMAGE, SURF_NODLIGHT, SURF_NODRAW, SURF_NOIMPACT,
    SURF_NOMARKS, SURF_NOMISCENTS, SURF_NOSTEPS, SURF_SKY, SURF_SLICK,
};
use native_string::atof::atof;

use crate::render_state::image_asset::ImageHandle;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::shader_asset::{ShaderAsset, ShaderHandle};
use crate::tr_local::shader_sort_t::shaderSort_t;

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

// GLS_* state-bit `#define`s this file's `NameTo*` functions return
// (translation dictionary point 8: `#define` -> `const`). Only the subset
// `NameToAFunc`/`NameToSrcBlendMode`/`NameToDstBlendMode` need.
// Source: `oracle/codemp/renderer/tr_local.h:1648-1667,1676-1680`
pub const GLS_SRCBLEND_ZERO: i32 = 0x0000_0001;
pub const GLS_SRCBLEND_ONE: i32 = 0x0000_0002;
pub const GLS_SRCBLEND_DST_COLOR: i32 = 0x0000_0003;
pub const GLS_SRCBLEND_ONE_MINUS_DST_COLOR: i32 = 0x0000_0004;
pub const GLS_SRCBLEND_SRC_ALPHA: i32 = 0x0000_0005;
pub const GLS_SRCBLEND_ONE_MINUS_SRC_ALPHA: i32 = 0x0000_0006;
pub const GLS_SRCBLEND_DST_ALPHA: i32 = 0x0000_0007;
pub const GLS_SRCBLEND_ONE_MINUS_DST_ALPHA: i32 = 0x0000_0008;
pub const GLS_SRCBLEND_ALPHA_SATURATE: i32 = 0x0000_0009;
pub const GLS_DSTBLEND_ZERO: i32 = 0x0000_0010;
pub const GLS_DSTBLEND_ONE: i32 = 0x0000_0020;
pub const GLS_DSTBLEND_SRC_COLOR: i32 = 0x0000_0030;
pub const GLS_DSTBLEND_ONE_MINUS_SRC_COLOR: i32 = 0x0000_0040;
pub const GLS_DSTBLEND_SRC_ALPHA: i32 = 0x0000_0050;
pub const GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA: i32 = 0x0000_0060;
pub const GLS_DSTBLEND_DST_ALPHA: i32 = 0x0000_0070;
pub const GLS_DSTBLEND_ONE_MINUS_DST_ALPHA: i32 = 0x0000_0080;
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

/// Raven `waveForm_t`.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:287-294`
#[derive(Clone, Copy, Default)]
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
    pub stages: Vec<ShaderStageParse>,
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

/// Raven `ClearGlobalShader` — constructs the per-parse scratch state fresh
/// (idiomatic equivalent of zeroing the file-scope `shader`/`stages` globals
/// at the start of every `ParseShader` call, §C9 out-param -> return value).
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:234-246`
pub fn ClearGlobalShader() -> ShaderParseState {
    let mut state = ShaderParseState::default();
    for _ in 0..MAX_SHADER_STAGES {
        let mut stage = ShaderStageParse::default();
        stage.gl_fog_color_override = FogColorOverride::None;
        state.stages.push(stage);
    }
    state.content_flags = CONTENTS_SOLID | CONTENTS_OPAQUE;
    state
}

/// Raven `ParseVector`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:323-350`
pub fn ParseVector<'a>(
    qs: &mut QSharedScratch,
    text: &mut Option<&'a [u8]>,
    common: &mut Common,
    state: &ShaderParseState,
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
                warn, state.name
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
                    warn, state.name
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
                warn, state.name
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

/// Raven `CollapseMultitexture`.
///
/// DEFERRED: R4 — the oracle's `#ifdef DEDICATED` leg takes exactly this path
/// (`return qfalse`) without touching GL at all; matched here rather than
/// transcribing the fixed-function multitexture collapse (`qglActiveTextureARB`
/// gate, the static `collapse[]` blend-mode table — a literal data table not
/// in this wave's packet slice, `GL_ADD`/`GL_MODULATE` texture-env fixed-
/// function state), which has no R4 wgpu-backend equivalent (DEC-01/DEC-37
/// A13.2) and no oracle-cited table to transcribe faithfully. A frontend fn
/// must not grow a GL dependency.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:2612-2713`
pub fn CollapseMultitexture(state: &mut ShaderParseState) -> bool {
    let _ = state;
    false
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
/// The oracle's `qhandle_t`/positional-index seam convention: `Handle {
/// index, generation: 0 }` is the identity mapping at the seam (`R2-D3`
/// `Handle` doc). Falls back to the default shader's name rather than the
/// oracle's debug-only `assert` (§19: pick the one defined behavior).
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:3785-3789`
pub fn RE_ShaderNameFromIndex(assets: &RenderAssets, index: i32) -> &str {
    let handle = ShaderHandle::new(index.max(0) as u32, 0);
    match assets.shaders.get(handle) {
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
    let handle = ShaderHandle::new(h_shader as u32, 0);
    if assets.shaders.get(handle).is_none() {
        com_printf(
            common,
            &format!("R_GetShaderByHandle: out of range hShader '{}'\n", h_shader),
        );
        return ShaderHandle::slot_zero();
    }
    handle
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
