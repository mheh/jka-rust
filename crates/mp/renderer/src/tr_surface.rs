//! Raven `tr_surface.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_surface.cpp`

// Raven-named functions/types keep their original casing across this
// transcription, matching the rest of the renderer/engine crates.
#![allow(non_snake_case)]

use mp_engine_qcommon::common::{com_printf, Common};
use mp_qshared::shared::{vec3_t, vec4_t};
// PORT-NOTE: `native_math` is not yet a direct `mp_renderer` dependency
// (Cargo.toml wiring gap, same finding as `tr_curve.rs`) — `Q_rsqrt` is
// LAW-cited at `crates/native/math/src/qmath.rs` and has no re-export
// reachable from this crate today. Flagged for the integrate phase to add
// the dependency edge; the call site below is otherwise final.
use native_math::qmath::Q_rsqrt;

use crate::render_state::frame_state::FrameState;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::tr_local::srf_display_list_s::srfDisplayList_t;
use crate::tr_local::surface_type_t::surfaceType_t;

/// Per-subsystem render-thread state for `CreateShape`'s file-scope
/// `sh1`/`sh2` vectors (random per-call shape-color vectors read by this
/// file's higher-wave shape-drawing fns, not yet ported). Named here per
/// DEC-37 A13.3: this wave (`tr_surface.cpp` wave 0) is where the subsystem's
/// globals first land.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp` (`sh1`/`sh2` file-scope
/// statics, read/written by `CreateShape`,
/// `oracle/codemp/renderer/tr_surface.cpp:976-987`)
pub struct TrSurfaceShapeState {
    pub sh1: vec3_t,
    pub sh2: vec3_t,
}

/// Raven `ComputeFinalVertexColor` — folds a vertex's base color with the
/// surface's lightstyle table entries (or forces full-bright), returning the
/// packed final vertex color.
///
/// DEFERRED: R4 — `tess.shader` (`lightmapIndex`/`styles`) has no R3 carrier:
/// `tess` (`shaderCommands_t`) dissolves entirely into R4's
/// tessellation/vertex-building pipeline (R2 `## State ownership` row
/// `tess`) and no replacement "current shader being tessellated" carrier
/// exists at R3. `styleColors` (`LightStyleTable`/`FrameState::
/// scene_light_styles`) and `r_fullbright` (`RendererCvars::r_fullbright`)
/// are both available and threaded below for when R4 supplies the missing
/// shader data.
// Source: `oracle/codemp/renderer/tr_surface.cpp:255-296`
pub fn ComputeFinalVertexColor(
    colors: [u8; 4],
    frame: &FrameState,
    cvars: &RendererCvars,
    common: &Common,
) -> [u8; 4] {
    let _ = (colors, frame, cvars, common);
    todo!("Port ComputeFinalVertexColor — oracle/codemp/renderer/tr_surface.cpp:255-296")
}

/// Raven `DoLine_Oriented` — emits a screen-oriented quad (two triangles) for
/// a line segment `start`..`end` of half-width `spanWidth`, colored from the
/// current entity's `shaderRGBA`.
///
/// DEFERRED: R4 — every touched field lives on `tess` (`shaderCommands_t`),
/// which dissolves entirely into R4's tessellation/vertex-building pipeline
/// (R2 `## State ownership` row `tess`); no R3 carrier holds
/// `tess.xyz`/`tess.texCoords`/`tess.vertexColors`/`tess.indexes`/
/// `tess.numVertexes`/`tess.numIndexes` to write into. Also reads
/// `backEnd.currentEntity->e.shaderRGBA`/`.data.line.stscale`
/// (`FrameState::current_entity`'s `RefEntity`, an empty placeholder pending
/// the `tr_scene` R3 wave).
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:712-766`
pub fn DoLine_Oriented(
    start: vec3_t,
    end: vec3_t,
    up: vec3_t,
    span_width: f32,
    frame: &mut FrameState,
) {
    let _ = (start, end, up, span_width, frame);
    // DEFERRED: R4 — DoLine_Oriented (see doc comment above)
    // Source: oracle/codemp/renderer/tr_surface.cpp:712-766
}

/// Raven `CreateShape` — recomputes the two random shape-color vectors
/// `sh1`/`sh2` used by the (not-yet-ported) shape-drawing fns in this file.
///
/// DEFERRED: R4/engine seam — needs the engine's own `rand`/`srand` LCG
/// (`crandom`, engine-fork ruling 21, NEVER libc `rand` and never the game
/// tier's `bg_channel::rng::Rng`); R2 assigns the renderer no receiver for it
/// (packet threading digest, DEC-37 A13.3 row).
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:976-987`
pub fn CreateShape(state: &mut TrSurfaceShapeState) {
    let _ = state;
    // DEFERRED: R4/engine seam — CreateShape (see doc comment above)
    // Source: oracle/codemp/renderer/tr_surface.cpp:976-987
}

/// Raven `VectorArrayNormalize` — normalizes `count` `vec4_t` entries in
/// place, leaving each entry's 4th component untouched. The oracle's `idppc`
/// branch (PowerPC `frsqrte` inline asm) is dead on every target this port
/// ships; only the portable `VectorNormalizeFast`-per-entry loop survives.
///
/// The per-entry `VectorNormalizeFast` body is inlined into the loop.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:1180-1228`;
/// `oracle/codemp/game/q_math.c:172-182`
pub fn VectorArrayNormalize(normals: &mut [vec4_t]) {
    for n in normals.iter_mut() {
        let ilength = Q_rsqrt(n[0] * n[0] + n[1] * n[1] + n[2] * n[2]);
        n[0] *= ilength;
        n[1] *= ilength;
        n[2] *= ilength;
    }
}

/// Raven `LodErrorForVolume` — the LOD error metric for a bounding volume of
/// `radius` centered at local-space `local`: transforms it to world space,
/// projects onto the view axis, and divides `r_lodCurveError` by the
/// (clamped) view-relative distance.
// DEFERRED: LodErrorForVolume — depends on `FrameState::ori`/`FrameState::
// view` (`OrientationR`/`ViewParms`, both empty placeholder structs pending
// the `tr_main` wave's `ori.axis`/`ori.origin`/`view.ori.axis`/
// `view.ori.origin` fields) — see
// `crates/mp/renderer/src/render_state/placeholders.rs`, out of this file's
// edit scope. A state home this packet marks mapped-but-not-yet-populated is
// an escalation, not an invention (preamble "state home ... ESCALATION").
// `RendererCvars::r_lodCurveError` is available and threaded below.
// Source: `oracle/codemp/renderer/tr_surface.cpp:1535-1563`
pub fn LodErrorForVolume(
    local: vec3_t,
    radius: f32,
    frame: &FrameState,
    cvars: &RendererCvars,
    common: &Common,
) -> f32 {
    let _ = (local, radius, frame, cvars, common);
    todo!("Port LodErrorForVolume — oracle/codemp/renderer/tr_surface.cpp:1535-1563")
}

/// Raven `RB_SurfaceBad` — the `SF_BAD` dispatch-table entry: logs a warning
/// for a surface that failed to tessellate. `surfType` is unused in the
/// oracle body too.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:1870-1872`
pub fn RB_SurfaceBad(_surf_type: &surfaceType_t, common: &mut Common) {
    com_printf(common, "Bad surface tesselated.\n");
}

/// Raven `RB_SurfaceDisplayList` — the `SF_DISPLAY_LIST` dispatch-table
/// entry: replays a compiled GL display list. Marked in the oracle as "not
/// implemented yet" itself.
///
/// DEFERRED: R4 — pure fixed-function GL (`qglCallList`); DEC-01/DEC-37: the
/// backend is an idiomatic wgpu rewrite, not a GL transcription, and R2
/// leaves this entry point unhomed (`GpuResources::gl_state` is a named
/// placeholder until R4).
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:2004-2008`
pub fn RB_SurfaceDisplayList(surf: &srfDisplayList_t) {
    let _ = surf;
    // DEFERRED: R4 — RB_SurfaceDisplayList qglCallList(surf.listNum)
    // (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_surface.cpp:2004-2008
}

/// Raven `RB_SurfaceSkip` — the `SF_SKIP` dispatch-table entry: a deliberate
/// no-op (`surf` is unused in the oracle body).
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:2010-2011`
pub fn RB_SurfaceSkip() {}
