//! `ModelAsset` — the model registry's arena payload, plus its handle alias.

use crate::render_state::handle::Handle;

/// The owned form of Raven `model_t` — `RenderAssets::models`' element
/// (`R2-D3`). Fields land with the client-rendering `tr_model` R3 wave; the
/// live dedicated-server model path keeps its own frozen shape
/// (`docs/subsystems/tr-model.md`, `crate::tr_model`) and is untouched by this
/// type.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:1117-1135`
#[derive(Clone)]
pub struct ModelAsset {}

/// A generation-counted handle into `RenderAssets::models` (A2).
pub type ModelHandle = Handle<ModelAsset>;
