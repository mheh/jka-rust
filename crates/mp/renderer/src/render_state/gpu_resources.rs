//! `GpuResources` — render-thread-only GPU state (`R2-D1`).

use crate::render_state::placeholders::GlStatePlaceholder;

/// Render-thread-only. Never touched by a trap query (ruling 3 invariant).
///
/// R2 freezes only `gl_state`: the wgpu-facing fields the design sketches
/// (`device`, `queue`, `surface`, `gpu_images: SecondaryArena<ImageHandle,
/// GpuImage>`, `pipelines: HashMap<PipelineKey, RenderPipeline>`) land with
/// R4, together with the wgpu dependency itself (DEC-01) — this crate is the
/// CPU-only one `jampded` links (ruling 16), so it grows no GPU dependency at
/// R3.
pub struct GpuResources {
    /// `glstate_t` equivalent (B6) — the GL binding cache has no wgpu meaning;
    /// a named placeholder until R4 defines the real pipeline/bind-group
    /// cache.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1253-1260`
    pub gl_state: GlStatePlaceholder,
}
