//! `SkinAsset` — the skin registry's arena payload, its per-surface element,
//! and its handle alias.

use crate::render_state::handle::Handle;
use crate::render_state::shader_asset::ShaderHandle;

/// The owned form of Raven `skinSurface_t` — one `(surface name, shader)` row
/// of a `.skin` file. Colocated with [`SkinAsset`] (its only holder) per
/// porting-rules §F21's "private helpers colocate"; `shader_t *shader`
/// becomes a `ShaderHandle` per the interior-safety law.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:604-607`
#[derive(Clone)]
pub struct SkinSurface {
    /// `name[MAX_QPATH]` — lower-cased at registration "so skin compares are
    /// faster" (`RE_RegisterIndividualSkin`,
    /// `oracle/codemp/renderer/tr_image.cpp:3061`).
    pub name: String,
    /// `shader`.
    pub shader: ShaderHandle,
}

/// The owned form of Raven `skin_t` — `RenderAssets::skins`' element
/// (`R2-D3`), in the shape the tier-2 transition audit assigns (`name` →
/// `String`, the `surfaces[128]` pointer array + its `numSurfaces` count →
/// one owned `Vec<SkinSurface>`). Raven's 128-entry cap survives as
/// `RE_RegisterIndividualSkin`'s overflow warning, not as a fixed-size field.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:609-613`
// `Default` = the zeroed pre-`R_InitSkins` slot-0 placeholder (A12
// constructor, harness boot support); `R_InitSkins`' reset re-seats the real
// `"<default skin>"` entry.
#[derive(Clone, Default)]
pub struct SkinAsset {
    /// `name[MAX_QPATH]` — Raven: "game path, including extension".
    pub name: String,
    /// `surfaces[128]` + `numSurfaces`.
    pub surfaces: Vec<SkinSurface>,
}

/// A generation-counted handle into `RenderAssets::skins` (A2).
pub type SkinHandle = Handle<SkinAsset>;
