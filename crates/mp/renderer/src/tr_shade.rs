//! Raven `tr_shade.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_shade.cpp`

// Raven-named functions/types keep their original casing across this
// transcription, matching the rest of the renderer/engine crates.
#![allow(non_snake_case)]
// Wave-0 ports of Raven `static` helpers: private by fidelity, with their
// callers landing in later R3 waves.
#![allow(dead_code)]

use crate::render_state::frame_state::FrameState;
use crate::render_state::gpu_resources::GpuResources;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::render_state::shader_asset::ShaderHandle;
use crate::tr_local::gl_index_t::glIndex_t;
use crate::tr_shader::TextureBundleParse;

/// Per-subsystem render-thread counters `R_DrawStripElements` accumulates
/// across calls.
///
/// Raven `static int c_begins`/`static int c_vertexes` — file-scope statics
/// in `tr_shade.cpp`, not part of `trGlobals_t`/`backEndCounters_t`; named
/// here per DEC-37 A13.3 since this wave lands their one write site.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:63,75` (used throughout
/// `R_DrawStripElements`)
#[derive(Default)]
pub struct TrShadeCounters {
    pub c_begins: i32,
    pub c_vertexes: i32,
}

/// Raven `R_ArrayElementDiscrete` — emits one tessellated vertex (color,
/// texcoords, position) to the fixed-function GL immediate-mode pipeline by
/// index.
///
/// DEFERRED: R4 — every touched value lives on `tess` (dissolved into R4's
/// tessellation/vertex-building pipeline, R2 `## State ownership` row
/// `tess`; no R3 carrier holds `tess.svars.colors`/`texcoords`/`xyz` to read
/// from) and `glState.currenttmu` (`GpuResources::gl_state`, a named
/// placeholder until R4). The body is GL calls only (DEC-37 A13.2).
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:37-48`
fn R_ArrayElementDiscrete(_gpu: &mut GpuResources, _index: i32) {
    // DEFERRED: R4 — R_ArrayElementDiscrete (see doc comment above)
    // Source: oracle/codemp/renderer/tr_shade.cpp:37-48
}

/// Raven `R_DrawStripElements` — batches a triangle-strip index list into
/// one or more GL `GL_TRIANGLE_STRIP` runs, restarting the strip whenever the
/// next triangle doesn't share an edge with the previous one.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:58-149`
fn R_DrawStripElements(
    indexes: &[glIndex_t],
    counters: &mut TrShadeCounters,
    mut element: impl FnMut(glIndex_t),
) {
    counters.c_begins += 1;

    if indexes.is_empty() {
        return;
    }

    // DEFERRED: R4 — qglBegin(GL_TRIANGLE_STRIP) (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_shade.cpp:69

    // prime the strip
    element(indexes[0]);
    element(indexes[1]);
    element(indexes[2]);
    counters.c_vertexes += 3;

    let mut last = [indexes[0], indexes[1], indexes[2]];
    let mut even = false;

    let mut i = 3;
    while i < indexes.len() {
        // odd numbered triangle in potential strip
        if !even {
            // check previous triangle to see if we're continuing a strip
            if indexes[i] == last[2] && indexes[i + 1] == last[1] {
                element(indexes[i + 2]);
                counters.c_vertexes += 1;
                // PORT-NOTE: dropped `assert( indexes[i+2] < tess.numVertexes )`
                // — tess is R4-dissolved (R2 `## State ownership` row `tess`),
                // no R3 carrier; debug-only bound check, no behavioral effect.
                even = true;
            }
            // otherwise we're done with this strip so finish it and start
            // a new one
            else {
                // DEFERRED: R4 — qglEnd/qglBegin(GL_TRIANGLE_STRIP) (DEC-37 A13.2)
                // Source: oracle/codemp/renderer/tr_shade.cpp:100-102
                counters.c_begins += 1;

                element(indexes[i]);
                element(indexes[i + 1]);
                element(indexes[i + 2]);

                counters.c_vertexes += 3;

                even = false;
            }
        } else {
            // check previous triangle to see if we're continuing a strip
            if last[2] == indexes[i + 1] && last[0] == indexes[i] {
                element(indexes[i + 2]);
                counters.c_vertexes += 1;

                even = false;
            }
            // otherwise we're done with this strip so finish it and start
            // a new one
            else {
                // DEFERRED: R4 — qglEnd/qglBegin(GL_TRIANGLE_STRIP) (DEC-37 A13.2)
                // Source: oracle/codemp/renderer/tr_shade.cpp:128-130
                counters.c_begins += 1;

                element(indexes[i]);
                element(indexes[i + 1]);
                element(indexes[i + 2]);
                counters.c_vertexes += 3;

                even = false;
            }
        }

        // cache the last three vertices
        last[0] = indexes[i];
        last[1] = indexes[i + 1];
        last[2] = indexes[i + 2];

        i += 3;
    }

    // DEFERRED: R4 — qglEnd (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_shade.cpp:148
}

/// Raven `RB_BeginSurface` — resets the tessellation buffer for a new
/// surface batch under `shader` (or its `remappedShader` twin), snapshotting
/// the shader's stage list, fog index, iterator function, and shader-relative
/// time into `tess`.
///
/// DEFERRED: R4 — the entire body writes `tess` (dissolved into R4's
/// tessellation/vertex-building pipeline, R2 `## State ownership` row
/// `tess`) from `shader_t` fields (`remappedShader`, `stages`, `sky`,
/// `numUnfoggedPasses`, `timeOffset`, `clampTime`) not yet landed on
/// `ShaderAsset` (`tr_shader` wave, `render_state::shader_asset`). No R3
/// carrier for either side.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:362-382`
pub fn RB_BeginSurface(_frame: &mut FrameState, _shader: ShaderHandle, _fog_num: i32) {
    // DEFERRED: R4 — RB_BeginSurface (see doc comment above)
    // Source: oracle/codemp/renderer/tr_shade.cpp:362-382
}

/// Raven `ForceAlpha` — overwrites the alpha byte of every tessellated
/// vertex color in `dstColors` with a fixed value.
///
/// DEFERRED: R4 — the loop bound is `tess.numVertexes` (dissolved into R4's
/// tessellation/vertex-building pipeline, R2 `## State ownership` row
/// `tess`); no R3 carrier holds it.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:1929-1939`
pub fn ForceAlpha(_dst_colors: &mut [u8], _force_ent_alpha: i32) {
    // DEFERRED: R4 — ForceAlpha (see doc comment above)
    // Source: oracle/codemp/renderer/tr_shade.cpp:1929-1939
}

/// Raven `R_DrawElements` — dispatches an index list either straight to
/// `qglDrawElements` in one shot, or to `R_DrawStripElements`'s
/// triangle-strip batching, chosen by the `r_primitives` cvar (falling back
/// on whether `qglLockArraysEXT` is bound when the cvar is unset).
///
/// DEFERRED: R4 — every branch is gated by state with no R3 carrier yet:
/// `r_primitives->integer` needs a live cvar-value read (`RendererCvars`
/// holds only `Option<CvarHandle>` — the renderer's cvar-value-read seam
/// isn't wired yet, same DEFERRED reason as `RB_DrawBuffer`'s `r_clear`
/// dependency, DEC-37 A13.1); `qglLockArraysEXT`'s bound-or-not fallback has
/// no R3 home (this packet's `STATE HOMES` row for it: DEFERRED R4, DEC-37
/// A13.2); and every terminal action is a GL call
/// (`qglDrawElements`/`qglArrayElement`, `R_ArrayElementDiscrete`) or the
/// `qglArrayElement` engine-seam receiver this wave doesn't resolve (this
/// packet's `STATE HOMES` rows for `qglArrayElement`/`qglDrawElements`: "NOT
/// renderer state ... confirm the exact receiver at port time"). No branch
/// has an observable non-GL effect once the cvar and the GL calls are
/// removed. The wave-0 in-module callee `R_DrawStripElements` is not
/// invoked — every call site downstream of it is gated by the unresolved
/// state above (same treatment as `RB_BeginDrawingView`'s deferred callees).
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:162-216`
pub fn R_DrawElements(_gpu: &mut GpuResources, _cvars: &RendererCvars, _indexes: &[glIndex_t]) {
    // DEFERRED: R4 — R_DrawElements (see doc comment above)
    // Source: oracle/codemp/renderer/tr_shade.cpp:162-216
}

/// Raven `R_BindAnimatedImage` — binds the correct frame of an animated (or
/// video-mapped, or fullbright-lightmap, or single-frame) texture bundle as
/// the active GL texture.
///
/// DEFERRED: R4 for every branch except the video-map one:
/// - the fullbright/lightmap fast path needs a live `r_fullbright->value`
///   read (`RendererCvars` holds only `Option<CvarHandle>` — the cvar-value
///   -read seam isn't wired yet, same reason as `RB_DrawBuffer`'s `r_clear`
///   dependency, DEC-37 A13.1) and `tr.whiteImage`, which has no
///   `RenderAssets` field landed by any prior wave;
/// - the single-frame path (`bundle->numImageAnimations <= 1`) is reachable
///   only once the fullbright branch above is known not to have returned,
///   so it inherits that same block;
/// - the animated-index path needs `backEnd.currentEntity->e.skinNum`
///   (`RefEntity` — `crate::render_state::placeholders` — has no `skin_num`
///   field landed by any prior wave; this file may not add one, porting-rules
///   §17/the wave contract restricts it to `tr_shade.rs`) in the
///   `RF_SETANIMINDEX` leg, and `tess.shaderTime` in the `else` leg (`tess`
///   is R4-dissolved outright — R2 `## State ownership` row `tess` — no R3
///   carrier exists for it, ever);
/// - every leg's terminal action is `GL_Bind`, itself a DEFERRED-R4 GL call
///   (DEC-37 A13.2).
///
/// The video-map branch (`bundle->isVideoMap`) is otherwise fully
/// determined by `bundle` alone (no cvar/global dependency), so it is
/// transcribed; its two callees are a genuine out-of-packet dependency (this
/// packet's `RESOLVED CALL SURFACE`: "`CIN_RunCinematic`/`CIN_UploadCinematic`:
/// NOT RESOLVED in the workspace ... escalate, never stub").
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:239-290`
pub fn R_BindAnimatedImage(
    _gpu: &mut GpuResources,
    _assets: &RenderAssets,
    _frame: &FrameState,
    _cvars: &RendererCvars,
    bundle: &TextureBundleParse,
) {
    if bundle.is_video_map {
        //TODO: Port CIN_RunCinematic
        //TODO: Port CIN_UploadCinematic
        // Source: oracle/codemp/renderer/tr_shade.cpp:243-244 (client-side
        // cinematic surface, not resolved anywhere in the workspace)
        todo!("Port CIN_RunCinematic/CIN_UploadCinematic — oracle/codemp/renderer/tr_shade.cpp:243-244");
    }

    // DEFERRED: R4 — everything past the video-map check (see doc comment
    // above)
    // Source: oracle/codemp/renderer/tr_shade.cpp:248-289
}

/// Raven `DrawNormals` — debug-draws each tessellated vertex's normal as a
/// short `GL_LINES` segment, gated behind `r_shownormals`.
///
/// DEFERRED: R4 — every touched value lives on `input` (`shaderCommands_t
/// *`, the same type as the dissolved `tess` — R2 `## State ownership` row
/// `tess`: "dissolved into R4's tessellation/vertex-building pipeline ...
/// no single global scratch buffer survives the new topology"; no R3 type
/// exists for `numVertexes`/`xyz`/`normal` to be read from) and every call
/// is GL-only (`qglBegin`/`qglColor3f`/`qglDepthRange`/`qglEnd`/
/// `qglVertex3fv`, plus the wave-0 `GL_Bind`/`GL_State`, DEC-37 A13.2). No
/// computation survives once both are removed — the loop's only output
/// (`temp`, via `VectorMA`) feeds straight into `qglVertex3fv` and is never
/// otherwise observed.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:333-351`
pub fn DrawNormals(_gpu: &mut GpuResources) {
    // DEFERRED: R4 — DrawNormals (see doc comment above)
    // Source: oracle/codemp/renderer/tr_shade.cpp:333-351
}
