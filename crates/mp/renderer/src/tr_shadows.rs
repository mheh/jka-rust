//! Raven `tr_shadows.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_shadows.cpp`

// Raven-named functions/types keep their original casing across this
// transcription, matching the rest of the renderer/engine crates.
#![allow(non_snake_case)]

use crate::render_state::placeholders::GlConfig;

/// Raven `edgeDef_t` — one shadow-silhouette edge record `R_AddEdgeDef`
/// appends and `R_RenderShadowEdges` walks.
///
/// `facing` is a plain C `int` flag (not `qboolean`), translated to `bool`
/// per the interior-safety law.
///
/// Type definition source: `oracle/codemp/renderer/tr_shadows.cpp:20-24`
#[derive(Clone, Copy)]
pub struct EdgeDef {
    pub i2: i32,
    pub facing: bool,
}

/// Raven `MAX_EDGE_DEFS` — per-vertex silhouette-edge slot count.
///
/// Source: `oracle/codemp/renderer/tr_shadows.cpp:26`
const MAX_EDGE_DEFS: usize = 32;

/// Per-subsystem render-thread state `R_AddEdgeDef`/`R_RenderShadowEdges`
/// share while building one shadow volume's silhouette edge list.
///
/// Raven `edgeDefs`/`numEdgeDefs` — file-scope statics in `tr_shadows.cpp`,
/// not part of `trGlobals_t`; named here per DEC-37 A13.3 since this wave
/// lands both their write site (`R_AddEdgeDef`) and read site
/// (`R_RenderShadowEdges`).
///
/// The oracle's `[SHADER_MAX_VERTEXES][MAX_EDGE_DEFS]` fixed scratch array
/// plus its `numEdgeDefs` fill counts become one `Vec<EdgeDef>` per vertex
/// (idiom translation dictionary item 9), each capped at `MAX_EDGE_DEFS` so
/// `R_AddEdgeDef` keeps the oracle's silent overflow discard.
///
/// Source: `oracle/codemp/renderer/tr_shadows.cpp:28-29`
#[derive(Default)]
pub struct TrShadowsState {
    pub edge_defs: Vec<Vec<EdgeDef>>,
}

/// Raven `R_AddEdgeDef` — appends one silhouette-edge candidate to vertex
/// `i1`'s edge list.
///
/// Source: `oracle/codemp/renderer/tr_shadows.cpp:32-43`
pub fn R_AddEdgeDef(state: &mut TrShadowsState, i1: i32, i2: i32, facing: bool) {
    let idx = i1 as usize;
    if state.edge_defs.len() <= idx {
        state.edge_defs.resize_with(idx + 1, Vec::new);
    }
    if state.edge_defs[idx].len() == MAX_EDGE_DEFS {
        return; // overflow
    }
    state.edge_defs[idx].push(EdgeDef { i2, facing });
}

/// Raven `R_RenderShadowEdges` — draws one `GL_TRIANGLE_STRIP` quad per
/// front-facing silhouette edge, forming the shadow volume's side walls.
///
/// DEFERRED: R4 — every touched value lives on `tess` (dissolved into R4's
/// tessellation/vertex-building pipeline, R2 `## State ownership` row
/// `tess`; no R3 carrier holds `tess.numVertexes` to bound the outer loop or
/// `tess.xyz` to read vertex positions from) and the loop body is
/// `qglBegin`/`qglVertex3fv`/`qglEnd` calls only (DEC-37 A13.2).
/// `numEdgeDefs`/`edgeDefs` (`TrShadowsState`, A13.3) are real state but have
/// nothing to drive without the tess vertex count.
///
/// PORT-NOTE: the retail `#if 1` block (the only compiled path — its `#else`
/// alternate edge-dedup pass is `#if 0`'d dead code) is the branch this
/// digest reflects; the trailing `#ifdef _STENCIL_REVERSE` Carmack-Reverse
/// capping pass is also dropped as dead code (`_STENCIL_REVERSE` is not
/// defined in the retail build).
///
/// Source: `oracle/codemp/renderer/tr_shadows.cpp:45-143`
pub fn R_RenderShadowEdges(_state: &TrShadowsState) {
    // DEFERRED: R4 — R_RenderShadowEdges (see doc comment above)
    // Source: oracle/codemp/renderer/tr_shadows.cpp:45-143
}

/// Raven `RB_ProjectionShadowDeform` — projects every tessellated vertex
/// along the current entity's light direction onto its shadow ground plane.
///
/// DEFERRED: R4 — the entire effect is a `tess.xyz` mutation bounded by
/// `tess.numVertexes` (dissolved into R4's tessellation/vertex-building
/// pipeline, R2 `## State ownership` row `tess`); no R3 carrier holds either.
/// `backEnd.ori`/`backEnd.currentEntity` (`RenderWorld::frame: FrameState`,
/// a real R3 carrier — `## State ownership` row `backEnd`) are readable, but
/// computing the ground/light vectors from them has nowhere to write its
/// result without the tess vertex buffer, so no partial signature is
/// invented ahead of the R4 tessellation pipeline landing.
///
/// Source: `oracle/codemp/renderer/tr_shadows.cpp:470-508`
pub fn RB_ProjectionShadowDeform() {
    // DEFERRED: R4 — RB_ProjectionShadowDeform (see doc comment above)
    // Source: oracle/codemp/renderer/tr_shadows.cpp:470-508
}

/// Raven `RB_DoShadowTessEnd` — casts tessellated shadow-volume vertices
/// away from the light direction and records each triangle's front/back
/// facing edges for the stencil-shadow silhouette pass, then draws the
/// silhouette side walls into the stencil buffer.
///
/// DEFERRED: R4 — every touched value lives on `tess` (dissolved into R4's
/// tessellation/vertex-building pipeline, R2 `## State ownership` row `tess`;
/// no R3 carrier holds `tess.numVertexes`/`tess.xyz`/`tess.indexes`/
/// `tess.numIndexes` to bound or drive either loop) or is itself GL-call
/// choreography (`GL_Bind`/`GL_Cull`/`GL_State`/`qglColor3f`/`qglColorMask`/
/// `qglEnable`/`qglDepthFunc`/`qglStencilFunc`/`qglStencilOp` — the
/// fixed-function GL surface, unhomed until R4, DEC-01/DEC-37;
/// `GpuResources::gl_state` is a named placeholder). `numEdgeDefs`/
/// `edgeDefs` (`TrShadowsState`/`R_AddEdgeDef`/`R_RenderShadowEdges`, A13.3)
/// and `glConfig.stencilBits`/`backEnd.ori`/`backEnd.currentEntity` are real
/// R3 carriers but have nothing to drive without the tess vertex/index
/// buffers, so no partial signature is invented ahead of the R4
/// tessellation pipeline landing (matches this file's
/// `RB_ProjectionShadowDeform` precedent).
///
/// PORT-NOTE: the retail `#if 1` block (the only compiled path — its `#else`
/// alternate light-position-based projection is `#if 0`'d dead code) is the
/// branch this digest reflects; the `#ifdef _STENCIL_REVERSE` Carmack-Reverse
/// capping pass and the `_DEBUG_STENCIL_SHADOWS` wireframe branch are also
/// dropped as dead code (neither macro is defined in the retail build).
///
/// Source: `oracle/codemp/renderer/tr_shadows.cpp:202-393`
pub fn RB_DoShadowTessEnd() {
    // DEFERRED: R4 — RB_DoShadowTessEnd (see doc comment above)
    // Source: oracle/codemp/renderer/tr_shadows.cpp:202-393
}

/// Raven `RB_ShadowFinish` — draws the stencil-shadow blend quad that
/// darkens the ground under the accumulated shadow-volume stencil counts.
///
/// `r_shadows_integer` is `r_shadows->integer` (`RendererCvars::r_shadows`,
/// DEC-37 A13.1 — read through the live engine cvar table by the caller,
/// threaded in here rather than reached for, matching this crate's
/// `R_CullLocalBox`/`r_nocull_integer` precedent); `glconfig` is
/// `RenderAssets::glconfig` (STATE HOMES row, sim-readable — B11).
///
/// DEFERRED: R4 — past the two guards below, the entire remaining body is
/// the fixed-function GL stencil/blend-quad sequence
/// (`qglEnable`/`qglStencilFunc`/`qglStencilOp`/`qglIsEnabled`/`GL_Cull`/
/// `GL_Bind(tr.whiteImage)`/`qglPushMatrix`/`qglLoadIdentity`/`qglColor4f`/
/// `GL_State`/`qglBegin`..`qglEnd`/`qglDisable`/`qglPopMatrix`) — unhomed
/// until R4 (DEC-01/DEC-37; `GpuResources::gl_state` is a named
/// placeholder), matching this file's `GL_Cull`-guard-then-defer precedent
/// (`tr_backend.rs`). `tr.whiteImage` also has no R3 carrier yet (STATE
/// HOMES `tr` SPLIT row names only the registries/`FrameState` scratch, not
/// this frontend singleton handle).
///
/// Source: `oracle/codemp/renderer/tr_shadows.cpp:406-461`
pub fn RB_ShadowFinish(r_shadows_integer: i32, glconfig: &GlConfig) {
    if r_shadows_integer != 2 {
        return;
    }
    if glconfig.stencil_bits < 4 {
        return;
    }

    // DEFERRED: R4 — RB_ShadowFinish stencil/blend-quad GL sequence (see doc
    // comment above).
    // Source: oracle/codemp/renderer/tr_shadows.cpp:418-461
}

/// Raven `RB_ShadowTessEnd` — retail dispatcher for the tessellated-shadow
/// silhouette pass: always forwards to `RB_DoShadowTessEnd` with no light
/// position.
///
/// PORT-NOTE: the compiled retail body is the `#else` leg of `#if 0` (line
/// 197-198); the `#if 0`-guarded per-entity `directedLight` early return and
/// the (further `/* */`-commented-out) per-dlight `RB_DoShadowTessEnd` loop
/// above it are dead code in the retail build — matches this file's dead-code
/// drop precedent (`R_RenderShadowEdges`, `RB_DoShadowTessEnd`'s own
/// `#if 1`/`#if 0` note). `RB_DoShadowTessEnd`'s ported (wave-1) Rust
/// signature takes no `lightPos` argument (its body is itself deferred whole
/// to R4), so this call passes nothing rather than an unused `None`.
///
/// Source: `oracle/codemp/renderer/tr_shadows.cpp:160-200`
pub fn RB_ShadowTessEnd() {
    RB_DoShadowTessEnd();
}

/// Raven `RB_DistortionFill` — draws the full-screen refraction/distortion
/// blend quad(s) (saber-trail and similar effects) into the previously
/// captured screen-copy texture, stencil-masked to the affected region.
///
/// `glconfig` is `RenderAssets::glconfig` (STATE HOMES row, sim-readable —
/// B11); `alpha`/`stretch`/`pre_post`/`negate` are the
/// `FrameEvent::SetRefractionProp` payload fields (STATE HOMES rows
/// `tr_distortion*`), threaded in rather than reached for.
///
/// DEFERRED: R4 — past the two guards below, the entire remaining body is
/// the fixed-function GL sequence (`qglEnable`/`qglStencilFunc`/
/// `qglStencilOp`/`qglDisable(GL_CLIP_PLANE0)`/`GL_Cull`/`qglMatrixMode`/
/// `qglPushMatrix`/`qglLoadIdentity`/`qglOrtho`/`GL_State`/
/// `qglBegin`..`qglEnd`/`qglColor4f`/`qglTexCoord2f`/`qglVertex2f`/
/// `qglPopMatrix`) — unhomed until R4 (DEC-01/DEC-37; `GpuResources::gl_state`
/// is a named placeholder), matching this file's `RB_ShadowFinish`
/// guard-then-defer precedent. The `spost`/`spost2` stretch-animation
/// arithmetic (`sin(tr.refdef.time*0.0005f)`/`sin(tr.refdef.time*0.0008f)`)
/// exists solely to parameterize that GL sequence and additionally has no R3
/// carrier to read from yet: `FrameState::refdef` (`TrRefdef`) does not carry
/// a `time` field (landed with a later `tr_scene`/`tr_main` wave), matching
/// this crate's `RB_Hyperspace` (`tr_backend.rs`) precedent of deferring
/// `refdef.time`-driven arithmetic alongside its GL consumer.
///
/// Source: `oracle/codemp/renderer/tr_shadows.cpp:579-708`
pub fn RB_DistortionFill(
    glconfig: &GlConfig,
    alpha: f32,
    stretch: f32,
    pre_post: bool,
    negate: bool,
) {
    if glconfig.stencil_bits < 4 {
        return;
    }

    //ok, cap the stupid thing now I guess
    if !pre_post {
        RB_CaptureScreenImage();
    }

    // DEFERRED: R4 — RB_DistortionFill stencil/blend-quad GL sequence (see
    // doc comment above).
    // Source: oracle/codemp/renderer/tr_shadows.cpp:596-707
    let _ = (alpha, stretch, negate);
}

/// Raven `RB_CaptureScreenImage` — copies a centered screen region into
/// `tr.screenImage`, clamped to the GL implementation's max texture size and
/// the current viewport.
///
/// DEFERRED: R4 — the capture-rect arithmetic (`radX`/`radY`/`cX`/`cY`)
/// exists solely to parameterize `qglCopyTexImage2D`, the fixed-function GL
/// call itself (unhomed until R4, DEC-01/DEC-37; `GpuResources::gl_state` is
/// a named placeholder) — matches this crate's `RB_Hyperspace`
/// (`tr_backend.rs`) precedent of deferring GL-only-consumed arithmetic
/// alongside its GL call rather than stranding an orphan return value.
/// `GL_Bind(tr.screenImage)` is also unhomed: `tr.screenImage`/
/// `tr.whiteImage` are frontend singleton image handles with no R3 carrier
/// (STATE HOMES `tr` SPLIT row names only the registries and `FrameState`
/// scratch, not these). `glConfig.vidWidth`/`vidHeight`/`maxTextureSize`
/// (`RenderAssets::glconfig`) are real R3 carriers and would drive this
/// arithmetic once `GL_Bind`/`qglCopyTexImage2D` land.
///
/// Source: `oracle/codemp/renderer/tr_shadows.cpp:511-572`
pub fn RB_CaptureScreenImage() {
    // DEFERRED: R4 — RB_CaptureScreenImage (see doc comment above)
    // Source: oracle/codemp/renderer/tr_shadows.cpp:511-572
}
