//! Raven `tr_shadows.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_shadows.cpp`

// Raven-named functions/types keep their original casing across this
// transcription, matching the rest of the renderer/engine crates.
#![allow(non_snake_case)]

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
