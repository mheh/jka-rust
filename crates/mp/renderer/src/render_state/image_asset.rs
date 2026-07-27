//! `ImageAsset` — the image registry's arena payload, plus its handle alias.

use crate::render_state::handle::Handle;

/// The owned form of Raven `image_t` — `RenderAssets::images`' element
/// (`R2-D3`), in the shape the tier-2 transition audit assigns (`imgName:
/// [c_char; 64]` → `String`); the tier-2 `tr_local::image_s` file survives
/// only until this type replaces it. The CPU-side field set is real (landed
/// with the `tr_image` R3 wave-1); `texnum` (the GL binding) has no R3 home
/// and lands with the R4 GPU wave.
///
/// Raven's narrow storage widths (`USHORT width/height`, `short
/// iLastLevelUsedOn`) widen to `i32` here: every oracle read is already
/// int-promoted (`width * height`), and the renderer interior carries no
/// layout obligation (DEC-37 ruling 1).
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:136-151`
#[derive(Clone, Default)]
pub struct ImageAsset {
    /// `imgName` — Raven: game path, including extension.
    pub img_name: String,
    /// `width` — Raven: after power of two and picmip but not including
    /// clamp to `MAX_TEXTURE_SIZE`.
    pub width: i32,
    /// `height`, same qualification as `width`.
    pub height: i32,
    /// `frameUsed` — Raven: for texture usage in frame statistics.
    pub frame_used: i32,
    /// `internalFormat`.
    pub internal_format: i32,
    /// `wrapClampMode` — Raven: `GL_CLAMP` or `GL_REPEAT`.
    pub wrap_clamp_mode: i32,
    /// `mipmap`.
    pub mipmap: bool,
    /// `allowPicmip`.
    pub allow_picmip: bool,
    /// `iLastLevelUsedOn`.
    pub last_level_used_on: i32,
}

/// A generation-counted handle into `RenderAssets::images` (A2).
pub type ImageHandle = Handle<ImageAsset>;
