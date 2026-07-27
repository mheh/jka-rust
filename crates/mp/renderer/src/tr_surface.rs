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
use native_math::qmath::{
    _VectorAdd as VectorAdd, _VectorScale as VectorScale, PerpendicularVectorMP, Q_rsqrt,
    RotatePointAroundVector, VectorNormalize,
};

use crate::render_state::frame_state::FrameState;
use crate::render_state::gpu_resources::GpuResources;
use crate::render_state::placeholders::RefEntity;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::tr_local::orientationr_t::orientationr_t;
use crate::tr_local::srf_display_list_s::srfDisplayList_t;
use crate::tr_local::surface_type_t::surfaceType_t;
use crate::tr_local::view_parms_t::viewParms_t;
use crate::tr_main::{R_TransformClipToWindow, R_TransformModelToClip};

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

/// Raven `RB_SurfaceBeam` — the `SF_ENTITY` "beam" surface: builds a
/// hexagonal (`NUM_BEAM_SEGS`-sided) tube of quads around the segment from
/// the current entity's `origin` to `oldorigin`.
///
/// `backEnd.currentEntity` (a nullable pointer) becomes `Option<&RefEntity>`,
/// the `tr_shade_calc.rs` `RB_CalcColorFromEntity` precedent.
///
/// DEFERRED: R4 — the vertex emission (`GL_Bind`/`GL_State`/`qglColor3f`/
/// `qglBegin`/`qglVertex3fv`/`qglEnd`) is the fixed-function GL surface;
/// DEC-01/DEC-37: the backend is an idiomatic wgpu rewrite, not a GL
/// transcription, and R2 leaves these entry points unhomed
/// (`GpuResources::gl_state` is a named placeholder until R4). The CPU-side
/// geometry (`start_points`/`end_points`) is still computed below per this
/// wave's threading digest ("port the CPU logic").
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:478-528`
pub fn RB_SurfaceBeam(current_entity: Option<&RefEntity>) {
    // `#define NUM_BEAM_SEGS 6` — a literal from the oracle source itself
    // (`tr_surface.cpp:480`), not a guessed constant.
    const NUM_BEAM_SEGS: usize = 6;

    let Some(e) = current_entity else {
        return;
    };

    let old_origin = e.old_origin;
    let origin = e.origin;

    let direction: vec3_t = [
        old_origin[0] - origin[0],
        old_origin[1] - origin[1],
        old_origin[2] - origin[2],
    ];
    let mut normalized_direction = direction;
    if VectorNormalize(&mut normalized_direction) == 0.0 {
        return;
    }

    let mut perpvec: vec3_t = [0.0; 3];
    PerpendicularVectorMP(&mut perpvec, normalized_direction);
    let unscaled_perpvec = perpvec;
    VectorScale(unscaled_perpvec, 4.0, &mut perpvec);

    let mut start_points: [vec3_t; NUM_BEAM_SEGS] = [[0.0; 3]; NUM_BEAM_SEGS];
    let mut end_points: [vec3_t; NUM_BEAM_SEGS] = [[0.0; 3]; NUM_BEAM_SEGS];
    for i in 0..NUM_BEAM_SEGS {
        // wave-0 ruling 12: `(360.0/NUM_BEAM_SEGS)*i` is a double-literal
        // expression in the oracle (`360.0` against int `NUM_BEAM_SEGS`/`i`);
        // evaluated in f64 and rounded to f32 once, at the call into
        // `RotatePointAroundVector`'s `float degrees` parameter.
        let degrees = (360.0f64 / NUM_BEAM_SEGS as f64 * i as f64) as f32;
        RotatePointAroundVector(&mut start_points[i], normalized_direction, perpvec, degrees);
        // VectorAdd( start_points[i], origin, start_points[i] ); -- commented
        // out in the oracle itself; preserved as a Raven comment, not
        // reactivated (porting-rules: preserve comments, not dead code).
        VectorAdd(start_points[i], direction, &mut end_points[i]);
    }

    // DEFERRED: R4 — GL_Bind(tr.whiteImage) / GL_State(GLS_SRCBLEND_ONE |
    // GLS_DSTBLEND_ONE) / qglColor3f(1,0,0) / qglBegin(GL_TRIANGLE_STRIP) /
    // qglVertex3fv ×2×(NUM_BEAM_SEGS+1) / qglEnd(): the triangle-strip vertex
    // emission from `start_points`/`end_points` computed above (see doc
    // comment above).
    // Source: oracle/codemp/renderer/tr_surface.cpp:516-527
    let _ = (start_points, end_points);
}

/// Raven `RB_SurfaceOrientedLine` — the oriented-line surface: builds a
/// screen-oriented quad from the current entity's `origin`/`oldorigin`,
/// using the entity's normalized `axis[1]` as the "up" (side) vector and
/// `data.line.width` as the half-width, via `DoLine_Oriented`.
///
/// DEFERRED: R4/escalation — `RefEntity` (`render_state/placeholders.rs`)
/// carries only the fields wave 0 landed (`origin`/`old_origin`/
/// `shader_rgba`/…, its own doc comment: "the rest of `refEntity_t`/
/// `trRefEntity_t` lands with the waves that read it"); `axis[3]` and the
/// `data.line` union's `width` are not among them. No R3 carrier exists for
/// either input this fn needs, and this file may not invent one (preamble:
/// "a state home this packet marks UNMAPPED is an ESCALATION, never an
/// invention"). The sole callee, `DoLine_Oriented`, is itself a full
/// DEFERRED no-op today (its `tess.*` write targets dissolved into R4,
/// `tr_surface.rs` wave 0) — no partial CPU logic survives dropping both
/// inputs.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:792-807`
pub fn RB_SurfaceOrientedLine(current_entity: Option<&RefEntity>) {
    let _ = current_entity;
    // DEFERRED: R4/escalation — RB_SurfaceOrientedLine (see doc comment
    // above): needs `RefEntity::axis`/`::data.line.width`, neither landed.
    // Source: oracle/codemp/renderer/tr_surface.cpp:792-807
}

/// Raven `LerpMeshVertexes` — MD3 mesh vertex lerp: decompresses/
/// interpolates per-vertex position+normal between the current entity's
/// `frame`/`oldframe` MD3 keyframes into the tessellation buffer.
///
/// DEFERRED: R4/escalation — every target this fn writes into is
/// unavailable at R3: `tess.xyz`/`tess.normal` (`shaderCommands_t`) dissolved
/// entirely into R4's tessellation/vertex-building pipeline with no
/// replacement scratch carrier (R2 `## State ownership` row `tess`); its
/// `md3Surface_t *surf` input has no ported Rust type anywhere in the crate
/// yet (out-of-packet — no `tr_model`/MD3 wave has landed it); and
/// `backEnd.currentEntity->e.frame`/`.oldframe` are not among the fields
/// wave 0 landed on `RefEntity` (`render_state/placeholders.rs`). No partial
/// CPU logic survives dropping all three — this file may not invent a state
/// home for any of them (preamble: "a state home this packet marks UNMAPPED
/// is an ESCALATION, never an invention").
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:1235-1346`
pub fn LerpMeshVertexes(backlerp: f32) {
    let _ = backlerp;
    // DEFERRED: R4/escalation — LerpMeshVertexes (see doc comment above):
    // needs `tess.xyz`/`tess.normal` (dissolved), `md3Surface_t` (unported),
    // and `RefEntity::frame`/`::oldframe` (not landed).
    // Source: oracle/codemp/renderer/tr_surface.cpp:1235-1346
}

/// Raven `RB_SurfaceAxis` — debug/dev draw: emits a 16-unit RGB axis triad
/// (red X / green Y / blue Z) at the origin via three colored line segments.
///
/// DEFERRED: R4 — every statement is fixed-function GL
/// (`GL_Bind`/`qglLineWidth`/`qglBegin`/`qglColor3f`/`qglVertex3f`/`qglEnd`),
/// with no CPU-side computation at all (every vertex is a literal constant);
/// DEC-01/DEC-37: the backend is an idiomatic wgpu rewrite, not a GL
/// transcription, and R2 leaves these entry points unhomed
/// (`GpuResources::gl_state` is a named placeholder until R4).
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:1782-1801`
pub fn RB_SurfaceAxis() {
    // DEFERRED: R4 — RB_SurfaceAxis (see doc comment above)
    // Source: oracle/codemp/renderer/tr_surface.cpp:1782-1801
}

/// Raven `RB_TestZFlare` — occlusion test for a screen-space flare point:
/// clip-tests then window-transforms `point`, bounds-checks it against the
/// viewport, and (unless `r_flares` disables the depth test) compares the
/// read-back depth-buffer sample against the point's own eye-space depth.
///
/// `ori`/`view` are `backEnd.ori`/`backEnd.viewParms`, threaded as the
/// already-ported tier-2 `orientationr_t`/`viewParms_t` directly rather than
/// through `FrameState::ori`/`::view` — those two `FrameState` fields are
/// still the empty `OrientationR`/`ViewParms` landing placeholders
/// (`render_state/placeholders.rs`), untouched by any wave-0 fn (same gap,
/// same fix as `tr_main.rs`'s wave-0 top-of-file PORT-NOTE: tier-2 fields may
/// be *read* through their existing shapes until their owning wave replaces
/// them).
///
/// DEFERRED: R4 — `glState.finishCalled = qfalse` has no target
/// (`GpuResources::gl_state` is the named `GlStatePlaceholder {}`, B6, with
/// no `finishCalled` field yet) and `qglReadPixels` is the fixed-function GL
/// surface (DEC-01/DEC-37 A13.2). `depth` stays at the oracle's own `float
/// depth = 0.0f;` initializer until R4 fills it in (`R_TakeScreenshot`
/// precedent, `tr_init.rs:475-482`); every other statement — the two
/// transform calls, both bounds checks, the `r_flares` early-out, and the
/// final `screenZ`/`visible` formula — is real CPU logic, ported below.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:1881-1927`
pub fn RB_TestZFlare(
    point: vec3_t,
    ori: &orientationr_t,
    view: &viewParms_t,
    cvars: &RendererCvars,
    common: &Common,
    gpu: &mut GpuResources,
) -> bool {
    // DEFERRED: R4 — `glState.finishCalled` write target (see doc comment
    // above); `gpu` is threaded for when R4 supplies it.
    let _ = gpu;

    // if the point is off the screen, don't bother adding it
    // calculate screen coordinates and depth
    let (eye, clip) = R_TransformModelToClip(point, &ori.modelMatrix, &view.projectionMatrix);

    // check to see if the point is completely off screen
    for i in 0..3usize {
        if clip[i] >= clip[3] || clip[i] <= -clip[3] {
            return false;
        }
    }

    let (_normalized, window) = R_TransformClipToWindow(clip, view);

    if window[0] < 0.0
        || window[0] >= view.viewportWidth as f32
        || window[1] < 0.0
        || window[1] >= view.viewportHeight as f32
    {
        // shouldn't happen, since we check the clip[] above, except for FP rounding
        return false;
    }

    // do test

    // read back the z buffer contents
    if common.cvar(cvars.r_flares).integer != 1 {
        // skipping the the z-test
        return true;
    }
    // doing a readpixels is as good as doing a glFinish(), so
    // don't bother with another sync
    // DEFERRED: R4 — glState.finishCalled = qfalse; qglReadPixels(...) (see
    // doc comment above)
    let depth: f32 = 0.0;

    let screen_z = view.projectionMatrix[14]
        / ((2.0 * depth - 1.0) * view.projectionMatrix[11] - view.projectionMatrix[10]);

    (-eye[2] - -screen_z) < 24.0
}
