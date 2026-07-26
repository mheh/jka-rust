//! `ImageAsset` — the image registry's arena payload, plus its handle alias.

use crate::render_state::handle::Handle;

/// The owned form of Raven `image_t` — `RenderAssets::images`' element
/// (`R2-D3`). Fields land with the `tr_image` R3 wave, in the shape the
/// tier-2 transition audit assigns (`imgName: [c_char; 64]` → `String`); the
/// tier-2 `tr_local::image_s` file survives only until that wave.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:136-151`
#[derive(Clone)]
pub struct ImageAsset {}

/// A generation-counted handle into `RenderAssets::images` (A2).
pub type ImageHandle = Handle<ImageAsset>;
