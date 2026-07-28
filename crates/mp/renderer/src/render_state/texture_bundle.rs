//! `TextureBundle` — `ShaderStage::bundle`'s element.

use crate::render_state::image_asset::ImageHandle;
use crate::tr_local::tex_coord_gen_t::texCoordGen_t;
use crate::tr_local::tex_mod_info_t::texModInfo_t;

/// The owned form of Raven `textureBundle_t` — one texture bundle of a
/// registered shader stage, in the shape
/// `docs/subsystems/renderer-r2-design.md`'s Group 2 `textureBundle_t` row
/// assigns: `image: *mut image_t` -> `Option<ImageHandle>`,
/// `tcGenVectors: *mut vec3_t` -> owned `[vec3_t; 2]` (Raven's pointer
/// addresses a fixed 2-element array), `texMods: *mut texModInfo_t` ->
/// owned `Vec<texModInfo_t>`.
///
/// `texMods` holds the tier-2 `texModInfo_t` (not `tr_shader`'s parse-local
/// `TexModInfo`) so the registered side stores the same Raven-shaped type the
/// rest of `tr_local` uses; that type gained `Clone`/`Copy` for this Vec, a
/// layout-neutral change noted at its declaration.
///
/// `numTexMods` is dropped: it is exactly `tex_mods.len()` here, since the
/// oracle's count and its `Hunk_Alloc`'d block always move together.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:372-389`
#[derive(Clone)]
pub struct TextureBundle {
    /// `image` — Raven: the bound image, `NULL` until a `map`/`clampmap`
    /// keyword loads one.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:373`
    pub image: Option<ImageHandle>,
    /// `tcGen`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:375`
    pub tc_gen: texCoordGen_t,
    /// `tcGenVectors` (`vec3_t *`, a fixed 2-element array in the oracle) —
    /// owned inline.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:376`
    pub tc_gen_vectors: [[f32; 3]; 2],
    /// `texMods` + `numTexMods` — the oracle's `Hunk_Alloc`'d block plus its
    /// count, collapsed into one owned `Vec` (§C9).
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:378-379`
    pub tex_mods: Vec<texModInfo_t>,
    /// `numImageAnimations`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:380`
    pub num_image_animations: i16,
    /// `imageAnimationSpeed`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:381`
    pub image_animation_speed: f32,
    /// `isLightmap`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:383`
    pub is_lightmap: bool,
    /// `oneShotAnimMap`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:384`
    pub one_shot_anim_map: bool,
    /// `vertexLightmap`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:385`
    pub vertex_lightmap: bool,
    /// `isVideoMap`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:386`
    pub is_video_map: bool,
    /// `videoMapHandle`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:388`
    pub video_map_handle: i32,
    /// `image[MAX_IMAGE_ANIMATIONS]` — the animated-frame list, carried
    /// separately from the single-frame `image` slot above for the same
    /// reason the parse mirror does (`tr_shader::TextureBundleParse::
    /// image_animations`: the oracle overloads `bundle[0].image` between the
    /// `map`/`clampmap` single-image case and the `animMap` array, and the
    /// owned `Vec` here IS the `Hunk_Alloc`+`memcpy`'d array, §C9).
    ///
    /// Source: `oracle/codemp/renderer/tr_shader.cpp:1400-1443`
    pub image_animations: Vec<ImageHandle>,
}
