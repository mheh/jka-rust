//! `ShaderAsset` — the shader registry's arena payload, plus its handle alias.

use crate::render_state::handle::Handle;

/// The owned form of Raven `shader_t` — `RenderAssets::shaders`' element
/// (`R2-D3`). Fields land with the `tr_shader` R3 wave, in the shape the
/// tier-2 transition audit assigns (`name` → `String`, `stages`/`deforms` →
/// owned `Vec`s, `remappedShader` → `Handle<ShaderAsset>`, the intrusive
/// `next` chain dissolved).
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:459-530`
#[derive(Clone)]
pub struct ShaderAsset {}

/// A generation-counted handle into `RenderAssets::shaders` (A2).
pub type ShaderHandle = Handle<ShaderAsset>;
