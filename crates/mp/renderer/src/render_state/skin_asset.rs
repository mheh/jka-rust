//! `SkinAsset` — the skin registry's arena payload, plus its handle alias.

use crate::render_state::handle::Handle;

/// The owned form of Raven `skin_t` — `RenderAssets::skins`' element
/// (`R2-D3`). Fields land with the `tr_image` skin-registration R3 wave, in
/// the shape the tier-2 transition audit assigns (`name` → `String`,
/// `surfaces` → `Vec<SkinSurface>`).
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:609-613`
#[derive(Clone)]
pub struct SkinAsset {}

/// A generation-counted handle into `RenderAssets::skins` (A2).
pub type SkinHandle = Handle<SkinAsset>;
