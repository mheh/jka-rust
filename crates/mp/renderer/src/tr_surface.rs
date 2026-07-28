//! Raven `tr_surface.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_surface.cpp`

// Raven-named functions/types keep their original casing across this
// transcription, matching the rest of the renderer/engine crates.
#![allow(non_snake_case)]

use core::f64::consts::PI;

use mp_engine_qcommon::common::{com_printf, Common};
use mp_engine_qcommon::qfiles::md3_surface_t::md3Surface_t;
use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::shared::{vec3_t, vec4_t};
// PORT-NOTE: `native_math` is not yet a direct `mp_renderer` dependency
// (Cargo.toml wiring gap, same finding as `tr_curve.rs`) — `Q_rsqrt` is
// LAW-cited at `crates/native/math/src/qmath.rs` and has no re-export
// reachable from this crate today. Flagged for the integrate phase to add
// the dependency edge; the call site below is otherwise final.
use native_math::qmath::{
    _DotProduct as DotProduct, _VectorAdd as VectorAdd, _VectorMA as VectorMA,
    _VectorScale as VectorScale, _VectorSubtract as VectorSubtract, vec3_origin, CrossProduct,
    MakeNormalVectors, PerpendicularVectorMP, Q_rsqrt, RotatePointAroundVector, VectorNormalize,
};

use crate::render_state::frame_state::FrameState;
use crate::render_state::gpu_resources::GpuResources;
use crate::render_state::placeholders::RefEntity;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::tr_local::orientationr_t::orientationr_t;
use crate::tr_local::srf_display_list_s::srfDisplayList_t;
use crate::tr_local::srf_flare_s::srfFlare_t;
use crate::tr_local::srf_grid_mesh_s::srfGridMesh_t;
use crate::tr_local::srf_poly_s::srfPoly_t;
use crate::tr_local::srf_surface_face_t::srfSurfaceFace_t;
use crate::tr_local::srf_triangles_t::srfTriangles_t;
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

/// Raven `RB_CheckOverflow` — flushes the current tessellation buffer
/// (`RB_EndSurface`/`RB_BeginSurface`) if adding `verts` more vertexes /
/// `indexes` more indexes would overflow `SHADER_MAX_VERTEXES`/
/// `SHADER_MAX_INDEXES` (half the vertex budget when the current shader is
/// `tr.shadowShader`), then `Com_Error(ERR_DROP, ...)`s if the request alone
/// can never fit even a fresh buffer.
///
/// DEFERRED: R4 — every read/write target is `tess`
/// (`shaderCommands_t`: `.shader`, `.numVertexes`, `.numIndexes`), which
/// dissolves entirely into R4's tessellation/vertex-building pipeline with no
/// replacement scratch carrier at R3 (packet STATE HOMES row `RB_CheckOverflow`
/// / `tess`; R2 `## State ownership` row `tess`). `tr.shadowShader` (the
/// other read) is the packet's `tr` SPLIT row's shadow-shader field, but
/// comparing it against `tess.shader` is meaningless without `tess` itself.
/// The two in-module callees this fn guards with (`RB_EndSurface`,
/// `RB_BeginSurface`) are themselves still `todo!()`/DEFERRED tess-dependent
/// stubs (`tr_shade.rs:156-159,448-450`) — no partial CPU logic survives
/// dropping `tess`.
///
/// Whole-body deferral: no partial body survives, so this lands as a loud
/// `todo!()` rather than a silent no-op (whole-fn-deferral convention —
/// partial-body fns keep DEFERRED comments instead).
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:30-52`
pub fn RB_CheckOverflow(verts: i32, indexes: i32, frame: &mut FrameState) {
    let _ = (verts, indexes, frame);
    todo!("Port RB_CheckOverflow — oracle/codemp/renderer/tr_surface.cpp:30-52")
}

/// Raven `RB_SurfaceGrid` — tessellates a bezier-patch grid surface (`cv`)
/// into the tessellation buffer at the current LOD, splitting the LOD'd
/// width/height point sets into vertex/index strips and issuing multiple
/// `RB_EndSurface`/`RB_BeginSurface` flush passes if a single pass would
/// overflow the buffer.
///
/// DEFERRED: R4/escalation — every write target is `tess`
/// (`shaderCommands_t`: `.xyz`, `.normal`, `.texCoords`, `.vertexColors`,
/// `.vertexDlightBits`, `.dlightBits`, `.indexes`, `.numVertexes`,
/// `.numIndexes`), which dissolves entirely into R4's tessellation/
/// vertex-building pipeline with no replacement scratch carrier at R3
/// (packet STATE HOMES row `RB_SurfaceGrid` / `tess`; R2 `## State ownership`
/// row `tess`). Even the read side is blocked independently: `cv`'s
/// `widthLodError`/`heightLodError`/`verts` fields are raw pointers on the
/// tier-2 `srfGridMesh_t` (`tr_local/srf_grid_mesh_s.rs`) with no quarantine
/// accessor in this wave's licensed list (`SurfaceRef`/`surface_kind`,
/// `srf_surface_face_t` point/indices, `bmodel_t::surfaces`, `model_s::
/// bmodel`, `srf_terrain_s::landscape`, `ctrland_scape` accessors,
/// `mdxm_view_of`) — dereferencing them here would be new unsafe, banned by
/// this wave's law. The two in-module callees this fn flushes through
/// (`RB_EndSurface`, `RB_BeginSurface`) are themselves still `todo!()`/
/// DEFERRED tess-dependent stubs (`tr_shade.rs:156-159,448-450`); the other
/// two callees (`ComputeFinalVertexColor`, `LodErrorForVolume`) are
/// themselves `todo!()` stubs in this same file pending the same `tess`/
/// `FrameState::ori`/`::view` gaps. No partial CPU logic survives dropping
/// both the write target and the read source.
///
/// Whole-body deferral: no partial body survives, so this lands as a loud
/// `todo!()` rather than a silent no-op (whole-fn-deferral convention —
/// partial-body fns keep DEFERRED comments instead).
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:1572-1764`
pub fn RB_SurfaceGrid(cv: &srfGridMesh_t, frame: &mut FrameState) {
    let _ = (cv, frame);
    todo!("Port RB_SurfaceGrid — oracle/codemp/renderer/tr_surface.cpp:1572-1764")
}

/// Raven `RB_AddQuadStampExt` — appends a screen-facing quad (4 verts / 6
/// indices) to the tessellation buffer: `origin`±`left`±`up` for the four
/// corners, a constant normal (`vec3_origin - backEnd.viewParms.ori.axis[0]`),
/// standard `s1`/`t1`..`s2`/`t2` UVs, and `color` broadcast to all four
/// verts.
///
/// DEFERRED: R4/escalation — every write target is `tess`
/// (`shaderCommands_t`: `.indexes`, `.xyz`, `.normal`, `.texCoords`,
/// `.vertexColors`, `.numVertexes`, `.numIndexes`), which dissolves entirely
/// into R4's tessellation/vertex-building pipeline with no replacement
/// scratch carrier at R3 (packet STATE HOMES row `RB_AddQuadStampExt` /
/// `tess`; R2 `## State ownership` row `tess`). The one non-tess computation
/// (the constant normal) is itself blocked: `backEnd.viewParms.ori.axis[0]`
/// has no R3 field (`FrameState::ori` is still the empty `OrientationR`
/// landing placeholder, `render_state/placeholders.rs`), and `vec3_origin` is
/// engine-owned with no confirmed renderer-side receiver yet (packet STATE
/// HOMES row `RB_AddQuadStampExt` / `vec3_origin`: "confirm the exact
/// receiver at port time" — an unresolved escalation, not an invention). The
/// in-module callee this fn guards with (`RB_CheckOverflow`) is itself a
/// `todo!()` tess-dependent stub above. No partial CPU logic survives
/// dropping all three.
///
/// Whole-body deferral: no partial body survives, so this lands as a loud
/// `todo!()` rather than a silent no-op (whole-fn-deferral convention —
/// partial-body fns keep DEFERRED comments instead).
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:60-125`
pub fn RB_AddQuadStampExt(
    origin: vec3_t,
    left: vec3_t,
    up: vec3_t,
    color: [u8; 4],
    s1: f32,
    t1: f32,
    s2: f32,
    t2: f32,
    frame: &mut FrameState,
) {
    let _ = (origin, left, up, color, s1, t1, s2, t2, frame);
    todo!("Port RB_AddQuadStampExt — oracle/codemp/renderer/tr_surface.cpp:60-125")
}

/// Raven `RB_SurfacePolychain` — fans a `srfPoly_t`'s `numVerts` verts into
/// the tessellation buffer as a triangle fan (copying `xyz`/`st`/`modulate`
/// per vert, then emitting `numVerts - 2` fan-index triples).
///
/// DEFERRED: R4/escalation — every write target is `tess`
/// (`shaderCommands_t`: `.xyz`, `.texCoords`, `.vertexColors`, `.indexes`,
/// `.numVertexes`, `.numIndexes`), which dissolves entirely into R4's
/// tessellation/vertex-building pipeline with no replacement scratch carrier
/// at R3 (packet STATE HOMES row `RB_SurfacePolychain` / `tess`; R2 `##
/// State ownership` row `tess`). The read side is blocked independently too:
/// `p->verts` is `srfPoly_t::verts: *mut polyVert_t`
/// (`tr_local/srf_poly_s.rs`), a tier-2 raw pointer with no quarantine
/// accessor — walking it here would be new unsafe, banned by this wave's
/// law (`UNSAFE IS BANNED`). The in-module callee this fn guards with
/// (`RB_CheckOverflow`) is itself a `todo!()` tess-dependent stub above. No
/// partial CPU logic survives dropping both the write target and the read
/// source.
///
/// Whole-body deferral: no partial body survives, so this lands as a loud
/// `todo!()` rather than a silent no-op (whole-fn-deferral convention —
/// partial-body fns keep DEFERRED comments instead).
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:227-253`
pub fn RB_SurfacePolychain(p: &srfPoly_t, frame: &mut FrameState) {
    let _ = (p, frame);
    todo!("Port RB_SurfacePolychain — oracle/codemp/renderer/tr_surface.cpp:227-253")
}

/// Raven `RB_SurfaceTriangles` — appends a `srfTriangles_t`'s full index +
/// vertex soup (xyz/normal/texCoords/lightmap-STs/color via
/// `ComputeFinalVertexColor`/per-vertex `vertexDlightBits`) to the
/// tessellation buffer, offsetting indices by the buffer's current
/// `numVertexes`. The `_XBOX`/`VV_LIGHTING` branches are dead on every
/// target this port ships; only the retail (non-`_XBOX`) body is in scope.
///
/// DEFERRED: R4/escalation — every write target is `tess`
/// (`shaderCommands_t`: `.dlightBits`, `.indexes`, `.xyz`, `.normal`,
/// `.texCoords`, `.vertexColors`, `.vertexDlightBits`, `.numIndexes`,
/// `.numVertexes`), which dissolves entirely into R4's tessellation/
/// vertex-building pipeline with no replacement scratch carrier at R3
/// (packet STATE HOMES row `RB_SurfaceTriangles` / `tess`; R2 `## State
/// ownership` row `tess`). The read side is blocked independently too:
/// `srf->indexes`/`srf->verts` are `srfTriangles_t::indexes: *mut i32`/
/// `::verts: *mut drawVert_t` (`tr_local/srf_triangles_t.rs`), tier-2 raw
/// pointers with no quarantine accessor — walking them here would be new
/// unsafe, banned by this wave's law. The two in-module callees this fn uses
/// (`ComputeFinalVertexColor`, `RB_CheckOverflow`) are themselves `todo!()`
/// stubs in this same file, the former for the same `tess` gap
/// (`tess.shader` has no R3 carrier). No partial CPU logic survives dropping
/// both the write target and the read source.
///
/// Whole-body deferral: no partial body survives, so this lands as a loud
/// `todo!()` rather than a silent no-op (whole-fn-deferral convention —
/// partial-body fns keep DEFERRED comments instead).
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:353-469`
pub fn RB_SurfaceTriangles(srf: &srfTriangles_t, frame: &mut FrameState) {
    let _ = (srf, frame);
    todo!("Port RB_SurfaceTriangles — oracle/codemp/renderer/tr_surface.cpp:353-469")
}

/// Raven `DoLine` — appends a screen-oriented quad (two triangles) for the
/// line segment `start`..`end` of half-width `spanWidth`, colored from the
/// current entity's `shaderRGBA`. (Distinct from the already-ported
/// `DoLine_Oriented`: this variant writes UVs `0/0`,`1/0`,`0/1`,`1/1`
/// directly rather than from the entity's `data.line.stscale`.)
///
/// DEFERRED: R4/escalation — every write target is `tess`
/// (`shaderCommands_t`: `.xyz`, `.texCoords`, `.vertexColors`, `.indexes`,
/// `.numVertexes`, `.numIndexes`), which dissolves entirely into R4's
/// tessellation/vertex-building pipeline with no replacement scratch carrier
/// at R3 (packet STATE HOMES row `DoLine` / `tess`; R2 `## State ownership`
/// row `tess`). Unlike `DoLine_Oriented`, the read side
/// (`backEnd.currentEntity->e.shaderRGBA`) IS available — `FrameState::
/// current_entity`'s `RefEntity::shader_rgba` landed at wave 0 — but every
/// statement in the oracle body writes its computed value straight into a
/// `tess.*` array slot (`VectorMA(..., tess.xyz[tess.numVertexes])`,
/// `tess.vertexColors[tess.numVertexes][k] = ...`); there is no independent
/// scratch computation to salvage once the write target is gone. The
/// in-module callee this fn guards with (`RB_CheckOverflow`) is itself a
/// `todo!()` tess-dependent stub above.
///
/// Whole-body deferral: no partial body survives, so this lands as a loud
/// `todo!()` rather than a silent no-op (whole-fn-deferral convention —
/// partial-body fns keep DEFERRED comments instead).
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:601-656`
pub fn DoLine(start: vec3_t, end: vec3_t, up: vec3_t, span_width: f32, frame: &mut FrameState) {
    let _ = (start, end, up, span_width, frame);
    todo!("Port DoLine — oracle/codemp/renderer/tr_surface.cpp:601-656")
}

/// Raven `DoLine2` — `DoLine`'s twin with independent half-widths at each
/// end (`spanWidth` at `start`, `spanWidth2` at `end`), otherwise identical
/// shape/coloring.
///
/// DEFERRED: R4/escalation — identical reasoning to `DoLine` immediately
/// above: every write target is `tess` (`.xyz`, `.texCoords`,
/// `.vertexColors`, `.indexes`, `.numVertexes`, `.numIndexes`), dissolved
/// into R4 with no R3 carrier (packet STATE HOMES row `DoLine2` / `tess`; R2
/// `## State ownership` row `tess`); the readable
/// `backEnd.currentEntity->e.shaderRGBA` input has no independent
/// computation to salvage once every write target is gone. The in-module
/// callee this fn guards with (`RB_CheckOverflow`) is itself a `todo!()`
/// tess-dependent stub above.
///
/// Whole-body deferral: no partial body survives, so this lands as a loud
/// `todo!()` rather than a silent no-op (whole-fn-deferral convention —
/// partial-body fns keep DEFERRED comments instead).
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:658-710`
pub fn DoLine2(
    start: vec3_t,
    end: vec3_t,
    up: vec3_t,
    span_width: f32,
    span_width2: f32,
    frame: &mut FrameState,
) {
    let _ = (start, end, up, span_width, span_width2, frame);
    todo!("Port DoLine2 — oracle/codemp/renderer/tr_surface.cpp:658-710")
}

/// Raven `DoCylinderPart` — appends one quad segment (4 verts / 6 indices,
/// a triangle-strip-style quad rather than a fan) of a `NUM_CYLINDER_SEGMENTS`
/// -sided cylinder from 4 caller-supplied `polyVert_t`s, copied verbatim
/// (`xyz`/`st`/`modulate`) into the tessellation buffer.
///
/// DEFERRED: R4/escalation — every write target is `tess`
/// (`shaderCommands_t`: `.xyz`, `.texCoords`, `.vertexColors`, `.indexes`,
/// `.numVertexes`, `.numIndexes`), which dissolves entirely into R4's
/// tessellation/vertex-building pipeline with no replacement scratch carrier
/// at R3 (packet STATE HOMES row `DoCylinderPart` / `tess`; R2 `## State
/// ownership` row `tess`). The read side (`verts[0..4]`) is the tier-1
/// `polyVert_t` — already ported and safe to read — but nothing survives the
/// write side's loss: every read is copied straight into a `tess.*` slot
/// with no independent computation. The in-module callee this fn guards
/// with (`RB_CheckOverflow`) is itself a `todo!()` tess-dependent stub
/// above. `NUM_CYLINDER_SEGMENTS` (`#define NUM_CYLINDER_SEGMENTS 32`,
/// packet FILE-SCOPE CONSTANTS, `tr_surface.cpp:815`) is not read by this
/// fn's own body (it walks a fixed 4 verts per call) — the caller that loops
/// `NUM_CYLINDER_SEGMENTS` times to build a full cylinder lands in a higher
/// wave.
///
/// Whole-body deferral: no partial body survives, so this lands as a loud
/// `todo!()` rather than a silent no-op (whole-fn-deferral convention —
/// partial-body fns keep DEFERRED comments instead).
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:818-847`
pub fn DoCylinderPart(verts: &[polyVert_t; 4], frame: &mut FrameState) {
    let _ = (verts, frame);
    todo!("Port DoCylinderPart — oracle/codemp/renderer/tr_surface.cpp:818-847")
}

/// Raven `RB_SurfaceMesh` — the `SF_MD3` dispatch-table entry: lerps the
/// current entity's MD3 keyframe (`LerpMeshVertexes`) then appends the
/// surface's triangle indices (offset by the buffer's current
/// `numVertexes`/`numIndexes`) and per-vertex UVs to the tessellation
/// buffer.
///
/// DEFERRED: R4/escalation — every write target is `tess`
/// (`shaderCommands_t`: `.indexes`, `.texCoords`, `.numIndexes`,
/// `.numVertexes`), which dissolves entirely into R4's tessellation/
/// vertex-building pipeline with no replacement scratch carrier at R3
/// (packet STATE HOMES row `RB_SurfaceMesh` / `tess`; R2 `## State
/// ownership` row `tess`). The read side is blocked independently too:
/// `surface->ofsTriangles`/`::ofsSt` are byte offsets the oracle walks via
/// raw pointer arithmetic off `surface` itself (`(int *)((byte *)surface +
/// surface->ofsTriangles)`) — `md3Surface_t`
/// (`mp_engine_qcommon::qfiles::md3_surface_t`) is an on-disk header with no
/// quarantine accessor for that trailing-data walk; performing it here
/// would be new unsafe, banned by this wave's law. `backEnd.currentEntity->
/// e.oldframe`/`.frame`/`.backlerp` (the `backlerp` computation) are also
/// not among the fields wave 0 landed on `RefEntity`
/// (`render_state/placeholders.rs`). The sole in-module callees
/// (`LerpMeshVertexes`, `RB_CheckOverflow`) are both `todo!()`/DEFERRED
/// stubs in this same file already. No partial CPU logic survives dropping
/// the write target, the read source, and the entity fields.
///
/// Whole-body deferral: no partial body survives, so this lands as a loud
/// `todo!()` rather than a silent no-op (whole-fn-deferral convention —
/// partial-body fns keep DEFERRED comments instead).
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:1353-1397`
pub fn RB_SurfaceMesh(surface: &md3Surface_t, frame: &mut FrameState) {
    let _ = (surface, frame);
    todo!("Port RB_SurfaceMesh — oracle/codemp/renderer/tr_surface.cpp:1353-1397")
}

/// Raven `RB_SurfaceFace` — the `SF_FACE` dispatch-table entry: appends a
/// planar BSP face's index list (offset by the buffer's current
/// `numVertexes`) and per-point xyz/normal/UVs/color to the tessellation
/// buffer. The `_XBOX` branch (16-bit indices, packed `srfPoints`, tangent
/// unpacking) is dead on every target this port ships; only the retail
/// (non-`_XBOX`) body is in scope.
///
/// DEFERRED: R4/escalation — every write target is `tess`
/// (`shaderCommands_t`: `.dlightBits`, `.indexes`, `.normal`, `.xyz`,
/// `.texCoords`, `.vertexColors`, `.vertexDlightBits`, `.numIndexes`), which
/// dissolves entirely into R4's tessellation/vertex-building pipeline with
/// no replacement scratch carrier at R3 (packet STATE HOMES row
/// `RB_SurfaceFace` / `tess`; R2 `## State ownership` row `tess`). Unlike
/// the other fns in this packet, the read side here IS reachable —
/// `srfSurfaceFace_t::point`/`::indices` (`tr_local/srf_surface_face_t.rs`)
/// are already-licensed quarantine accessors for the trailing-array walk —
/// but every one of those reads is copied straight into a `tess.*` slot with
/// no independent computation (`ComputeFinalVertexColor`, the other
/// in-module callee, is itself a `todo!()` stub in this same file for the
/// same `tess.shader` gap). No partial CPU logic survives dropping the
/// write target.
///
/// Whole-body deferral: no partial body survives, so this lands as a loud
/// `todo!()` rather than a silent no-op (whole-fn-deferral convention —
/// partial-body fns keep DEFERRED comments instead).
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:1405-1532`
pub fn RB_SurfaceFace(surf: &srfSurfaceFace_t, frame: &mut FrameState) {
    let _ = (surf, frame);
    todo!("Port RB_SurfaceFace — oracle/codemp/renderer/tr_surface.cpp:1405-1532")
}

/// Raven `RB_AddQuadStamp` — the default-UV wrapper around
/// `RB_AddQuadStampExt` (`s1`/`t1`/`s2`/`t2` fixed at `0,0,1,1`).
///
/// Panics via `RB_AddQuadStampExt`'s loud stub until its owning wave lands —
/// that callee is still a `todo!()` in this same file for the `tess` gap its
/// own doc names. The wrapper's one line of behavior (the fixed UVs) is
/// transcribed regardless: a faithfully-transcribed body whose callee is a
/// loud stub is landed code, not a deferral.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:132-134`
pub fn RB_AddQuadStamp(
    origin: vec3_t,
    left: vec3_t,
    up: vec3_t,
    color: [u8; 4],
    frame: &mut FrameState,
) {
    RB_AddQuadStampExt(origin, left, up, color, 0.0, 0.0, 1.0, 1.0, frame);
}

/// Raven `RB_SurfaceLine` — the `SF_ENTITY` "line" surface: builds a
/// screen-oriented quad from the current entity's `origin`/`oldorigin`,
/// computing the quad's "up" (side) vector as the normalized cross product of
/// the two endpoints' view-relative direction vectors, via `DoLine`.
///
/// `backEnd.currentEntity` becomes `Option<&RefEntity>` (the `RB_SurfaceBeam`
/// precedent, this file, wave 0). `backEnd.viewParms.ori.origin` is threaded
/// as the already-ported tier-2 `viewParms_t` directly rather than through
/// `FrameState::view` — that field is still the empty `ViewParms` landing
/// placeholder (`render_state/placeholders.rs`), the `RB_TestZFlare`
/// precedent (this file).
///
/// DEFERRED: escalation — the final `DoLine(start, end, right, e->radius)`
/// call is unreachable: `refEntity_t::radius`
/// (`oracle/codemp/cgame/tr_types.h:158`) is not among the fields wave 0
/// landed on `RefEntity` (`render_state/placeholders.rs`) — a state home this
/// packet marks mapped-but-not-yet-populated is an escalation, not an
/// invention (preamble). `DoLine` is itself still a `todo!()` tess-dependent
/// stub in this same file regardless. The `right` vector computation above it
/// is real CPU logic and is transcribed.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:771-790`
pub fn RB_SurfaceLine(current_entity: Option<&RefEntity>, view: &viewParms_t) {
    let Some(e) = current_entity else {
        return;
    };

    let end = e.old_origin;
    let start = e.origin;

    // compute side vector
    let mut v1: vec3_t = [0.0; 3];
    let mut v2: vec3_t = [0.0; 3];
    VectorSubtract(start, view.ori.origin, &mut v1);
    VectorSubtract(end, view.ori.origin, &mut v2);
    let mut right: vec3_t = [0.0; 3];
    CrossProduct(v1, v2, &mut right);
    VectorNormalize(&mut right);

    // DEFERRED: escalation — DoLine(start, end, right, e.radius) (see doc
    // comment above): `RefEntity::radius` not landed; `DoLine` itself a
    // todo!() tess-dependent stub.
    // Source: oracle/codemp/renderer/tr_surface.cpp:789
    let _ = right;
}

/// Raven `RB_SurfaceCylinder` — the `SF_ENTITY` "cylinder" surface: builds a
/// LOD-scaled ring of quads (`DoCylinderPart` per segment) around the
/// current entity's `origin`..`oldorigin` axis.
///
/// `backEnd.currentEntity` becomes `Option<&RefEntity>` (the `RB_SurfaceBeam`
/// precedent, this file, wave 0); `backEnd.viewParms` is threaded as the
/// already-ported tier-2 `viewParms_t` directly (the `RB_TestZFlare`
/// precedent, this file) — `FrameState::view` is still the empty `ViewParms`
/// landing placeholder.
///
/// The LOD `detail`/`segments` computation (needs only `origin`/`oldorigin`/
/// `axis[0]`, all landed on `RefEntity`, plus `view.ori.origin`/`view.fovX`)
/// is real CPU logic and is transcribed.
///
/// DEFERRED: escalation — everything past `MakeNormalVectors` is
/// unreachable: `VectorScale( vu, e->radius, v1 )`/`VectorScale( vu,
/// e->rotation, vu )` need `refEntity_t::radius`/`::rotation`
/// (`oracle/codemp/cgame/tr_types.h:158-159`), neither among the fields wave
/// 0 landed on `RefEntity` (`render_state/placeholders.rs`) — a state home
/// this packet marks mapped-but-not-yet-populated is an escalation, not an
/// invention (preamble). The subsequent ring-building loops write into the
/// file-scope `static polyVert_t lower_points[NUM_CYLINDER_SEGMENTS]`/
/// `upper_points[...]`/`verts[4]` — classified per the three-kind rule as
/// kind-2 rotating per-call scratch (every element is written before it is
/// read, once per call; would become owned local
/// `[PolyVert; NUM_CYLINDER_SEGMENTS]`/`[PolyVert; 4]` arrays, never a field)
/// — but are not materialized here since the values they would hold
/// (`v1`/`vu`-derived) are themselves blocked, and their sole consumer
/// `DoCylinderPart` is itself still a `todo!()` tess-dependent stub in this
/// same file.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:853-953`
pub fn RB_SurfaceCylinder(current_entity: Option<&RefEntity>, view: &viewParms_t) {
    // `#define NUM_CYLINDER_SEGMENTS 32` — packet FILE-SCOPE CONSTANTS,
    // `tr_surface.cpp:815`.
    const NUM_CYLINDER_SEGMENTS: i32 = 32;

    let Some(e) = current_entity else {
        return;
    };

    // Work out the detail level of this cylinder
    let mut midpoint: vec3_t = [0.0; 3];
    VectorAdd(e.origin, e.old_origin, &mut midpoint);
    VectorScale(midpoint, 0.5, &mut midpoint); // Average start and end

    VectorSubtract(midpoint, view.ori.origin, &mut midpoint);
    let mut length = VectorNormalize(&mut midpoint);

    // this doesn't need to be perfect....just a rough compensation for zoom level is enough
    length *= view.fovX / 90.0;

    let detail = 1.0 - (length / 1024.0);
    let mut segments = (NUM_CYLINDER_SEGMENTS as f32 * detail) as i32;

    // 3 is the absolute minimum, but the pop between 3-8 is too noticeable
    if segments < 8 {
        segments = 8;
    }

    if segments > NUM_CYLINDER_SEGMENTS {
        segments = NUM_CYLINDER_SEGMENTS;
    }

    // Get the direction vector
    let mut vr: vec3_t = [0.0; 3];
    let mut vu: vec3_t = [0.0; 3];
    MakeNormalVectors(e.axis[0], &mut vr, &mut vu);
    let _ = vr;

    // DEFERRED: escalation — VectorScale(vu, e.radius, v1) onward, and the
    // ring-building loops below it (see doc comment above): `RefEntity::
    // radius`/`::rotation` not landed; `DoCylinderPart` itself a todo!()
    // tess-dependent stub.
    // Source: oracle/codemp/renderer/tr_surface.cpp:892-952
    let _ = (vu, segments);
}

/// Raven `ApplyShape` — recursively subdivides a straight radius-tapered
/// segment into a jittered "lightning bolt" shape (jitter driven by the
/// per-call random `sh1`/`sh2` vectors, `CreateShape`), bottoming out at
/// `DoLine2` once `count` reaches 0.
///
/// `sh1`/`sh2` are threaded via `TrSurfaceShapeState`, the carrier this
/// file's wave 0 already named for them (`CreateShape`'s doc comment, DEC-37
/// A13.3).
///
/// Panics via `DoLine2`'s loud stub until its owning wave lands — the
/// `count < 1` recursion base case calls it, and it is still a `todo!()`
/// `tess`-dependent stub in this same file. Every other line here is pure
/// vector math with no `tess` dependency, so the body is transcribed in
/// full.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:990-1036`
pub fn ApplyShape(
    start: vec3_t,
    end: vec3_t,
    right: vec3_t,
    sradius: f32,
    eradius: f32,
    count: i32,
    state: &mut TrSurfaceShapeState,
    frame: &mut FrameState,
) {
    if count < 1 {
        // done recursing
        DoLine2(start, end, right, sradius, eradius, frame);
        return;
    }

    CreateShape(state);

    let mut fwd: vec3_t = [0.0; 3];
    VectorSubtract(end, start, &mut fwd);
    let dis = VectorNormalize(&mut fwd) * 0.7;
    let mut rt: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];
    MakeNormalVectors(fwd, &mut rt, &mut up);

    let mut perc = state.sh1[0];

    let mut point1: vec3_t = [0.0; 3];
    VectorScale(start, perc, &mut point1);
    VectorMA(point1, 1.0 - perc, end, &mut point1);
    VectorMA(point1, dis * state.sh1[1], rt, &mut point1);
    VectorMA(point1, dis * state.sh1[2], up, &mut point1);

    // do a quick and dirty interpolation of the radius at that point
    let rads1 = sradius * 0.666 + eradius * 0.333;
    let rads2 = sradius * 0.333 + eradius * 0.666;

    // recursion
    ApplyShape(
        start,
        point1,
        right,
        sradius,
        rads1,
        count - 1,
        state,
        frame,
    );

    perc = state.sh2[0];

    let mut point2: vec3_t = [0.0; 3];
    VectorScale(start, perc, &mut point2);
    VectorMA(point2, 1.0 - perc, end, &mut point2);
    VectorMA(point2, dis * state.sh2[1], rt, &mut point2);
    VectorMA(point2, dis * state.sh2[2], up, &mut point2);

    // recursion
    ApplyShape(point2, point1, right, rads1, rads2, count - 1, state, frame);
    ApplyShape(point2, end, right, rads2, eradius, count - 1, state, frame);
}

/// Raven `RB_SurfaceSprite` — the `SF_ENTITY` "sprite" surface: builds a
/// screen-oriented quad, optionally rotated by the current entity's
/// `rotation` around the view axis, scaled to its `radius`.
///
/// DEFERRED: escalation — every statement reads `refEntity_t::radius`/
/// `::rotation` (`oracle/codemp/cgame/tr_types.h:158-159`) directly or via a
/// value (`left`/`up`) derived from them; neither field is among the ones
/// wave 0 landed on `RefEntity` (`render_state/placeholders.rs`). No
/// statement in this body is independent of them — unlike
/// `RB_SurfaceOrientedQuad` below, there is no unscaled-vector prefix to
/// salvage — so nothing survives dropping both inputs. A state home this
/// packet marks mapped-but-not-yet-populated is an escalation, not an
/// invention (preamble).
///
/// Whole-body deferral: no partial body survives, so this lands as a loud
/// `todo!()` rather than a silent no-op (whole-fn-deferral convention —
/// partial-body fns keep DEFERRED comments instead).
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:141-169`
pub fn RB_SurfaceSprite(
    current_entity: Option<&RefEntity>,
    view: &viewParms_t,
    frame: &mut FrameState,
) {
    let _ = (current_entity, view, frame);
    todo!("Port RB_SurfaceSprite — oracle/codemp/renderer/tr_surface.cpp:141-169")
}

/// Raven `RB_SurfaceOrientedQuad` — the `SF_ENTITY` "oriented quad" surface:
/// builds a quad from the current entity's `axis[1]`/`axis[2]`, optionally
/// rotated by `rotation`, scaled to `radius`.
///
/// The unscaled `left`/`up` copy (`axis[1]`/`axis[2]`) needs neither
/// `radius` nor `rotation` and is real CPU logic, so it is transcribed.
///
/// DEFERRED: escalation — everything past the copy (see doc comment above):
/// `refEntity_t::radius`/`::rotation` (`oracle/codemp/cgame/tr_types.h:
/// 158-159`) are not among the fields wave 0 landed on `RefEntity`
/// (`render_state/placeholders.rs`) — a state home this packet marks
/// mapped-but-not-yet-populated is an escalation, not an invention
/// (preamble). `RB_AddQuadStamp` is not called.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:177-220`
pub fn RB_SurfaceOrientedQuad(
    current_entity: Option<&RefEntity>,
    view: &viewParms_t,
    frame: &mut FrameState,
) {
    let Some(e) = current_entity else {
        return;
    };

    //	MakeNormalVectors( backEnd.currentEntity->e.axis[0], left, up ); --
    // commented out in the oracle itself; preserved as a Raven comment, not
    // reactivated (porting-rules: preserve comments, not dead code).
    let left = e.axis[1];
    let up = e.axis[2];

    // DEFERRED: escalation — the `rotation == 0` scale branch onward, the
    // `isMirror` flip, and the final `RB_AddQuadStamp` call (see doc comment
    // above): `RefEntity::radius`/`::rotation` not landed.
    // Source: oracle/codemp/renderer/tr_surface.cpp:188-220
    let _ = (left, up, view, frame);
}

/// Raven `DoSprite` — builds a screen-oriented quad of half-size `radius`,
/// rotated by `rotation` around the view axis, at `origin`, colored from the
/// current entity's `shaderRGBA`. (Called with `radius`/`rotation` already
/// resolved by its higher-wave caller, unlike `RB_SurfaceSprite` above which
/// reads them straight off the entity.)
///
/// Panics via `RB_AddQuadStampExt`'s loud stub (`tr_surface.rs:508`) until
/// its owning wave lands — reached through `RB_AddQuadStamp`, the real
/// one-line wrapper this fn calls (the same transitive path
/// `RB_AddQuadStamp`'s own doc comment above names).
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:533-555`
pub fn DoSprite(
    origin: vec3_t,
    radius: f32,
    rotation: f32,
    current_entity: Option<&RefEntity>,
    view: &viewParms_t,
    frame: &mut FrameState,
) {
    let Some(e) = current_entity else {
        return;
    };

    // ruling 12: `M_PI * rotation / 180.0f` promotes to `f64` (the
    // unsuffixed `M_PI` double constant), truncating back to `f32` at the
    // `ang` assignment.
    let ang = (PI * rotation as f64 / 180.0) as f32;
    let s = ang.sin();
    let c = ang.cos();

    let mut left: vec3_t = [0.0; 3];
    VectorScale(view.ori.axis[1], c * radius, &mut left);
    VectorMA(left, -s * radius, view.ori.axis[2], &mut left);

    let mut up: vec3_t = [0.0; 3];
    VectorScale(view.ori.axis[2], c * radius, &mut up);
    VectorMA(up, s * radius, view.ori.axis[1], &mut up);

    if view.isMirror != 0 {
        VectorSubtract(vec3_origin, left, &mut left);
    }

    RB_AddQuadStamp(origin, left, up, e.shader_rgba, frame);
}

/// Raven `DoBoltSeg` — recursively steps `start`..`end` in ~20-unit chunks,
/// jittering each point (`Q_crandom`) and passing the resulting segment to
/// `ApplyShape`, occasionally forking a tendril via a self-recursive call
/// when `RF_FORKED` is set.
///
/// The pre-loop setup (direction/normal vectors, the `old` running point,
/// the initial radii) needs none of the blocked inputs below and is real CPU
/// logic, so it is transcribed.
///
/// DEFERRED: engine seam — every statement inside the `for ( i = 20; i <=
/// dis; i += 20 )` stepping loop reads `Q_crandom(&e->frame)`/
/// `Q_random(&e->frame)` (`tr_surface.cpp:1075-1077,1100,1113`): both seed
/// from **the current entity's own `frame` field**, not an ambient/global
/// LCG (`Q_crandom`/`Q_random` are already ported, `native_math::qmath
/// ::{Q_crandom, Q_random}`, `*mut c_int` seed param). Blocker (1) — no
/// `frame` field to seed from — is now closed: `RefEntity::frame` exists
/// (campaign #41 batch 1,
/// `render_state/placeholders.rs`) — but (2) stands: each call also *writes*
/// the seed in-place through that pointer, and this fn's `Option<&RefEntity>`
/// dispatch shape (the `RB_SurfaceElectricity`/`RB_SurfaceFlare` precedent,
/// this file) is immutable, so there is still nowhere to commit the mutation
/// across the loop's repeated calls. Switching the dispatch shape to
/// `Option<&mut RefEntity>` is the follow-up rewire. The `RF_FORKED`
/// branch's `f_count--` write is this packet's STATE HOMES row
/// `DoBoltSeg`/`f_count`: "per-subsystem owned state struct, NAMED BY THIS
/// WAVE if this file's wave is where the subsystem lands" — this file
/// already names that carrier (`TrSurfaceShapeState`, `CreateShape`'s doc
/// comment) but `f_count` is not added to it here since the loop that would
/// read/write it never executes; note for whichever wave does add it that
/// oracle's `f_count` is **file-scope** (`static float f_count;`,
/// `tr_surface.cpp:956`, alongside `sh1`/`sh2`), not fn-scope, and is a
/// `float`, not an `int`. The loop also reads `e->renderfx & RF_TAPERED`
/// (`:1088`) — `RF_TAPERED`/`RF_FORKED`/`RF_GROW` are now ported in the
/// crate's canonical flag home (`tr_public::ref_flags`), so the masks are no
/// longer a blocker. Still unported is the `LIGHTNING_RECURSION_LEVEL`
/// constant it passes to `ApplyShape`
/// (`:958`, `#define LIGHTNING_RECURSION_LEVEL 1`). The
/// loop's two in-module callees (`ApplyShape`, the self-recursive
/// `DoBoltSeg`) are both blocked by the same gaps.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:1039-1124`
pub fn DoBoltSeg(
    start: vec3_t,
    end: vec3_t,
    right: vec3_t,
    radius: f32,
    current_entity: Option<&RefEntity>,
) {
    let mut fwd: vec3_t = [0.0; 3];
    VectorSubtract(end, start, &mut fwd);
    let dis = VectorNormalize(&mut fwd);

    let mut rt: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];
    MakeNormalVectors(fwd, &mut rt, &mut up);

    let old = start;
    let old_radius = radius;
    let new_radius = radius;

    // DEFERRED: `RefEntity::frame` + the stepping loop (see doc comment
    // above), which is where `e = &backEnd.currentEntity->e` is actually
    // read.
    // Source: oracle/codemp/renderer/tr_surface.cpp:1061-1123
    let _ = (
        dis,
        rt,
        up,
        old,
        old_radius,
        new_radius,
        right,
        current_entity,
    );
}

/// Raven `RB_SurfaceFlare` — the `SF_FLARE` dispatch-table entry: occlusion-
/// tests a flare surface (`RB_TestZFlare`), fades its color by the angle
/// between its normal and the view direction, then stamps a screen-oriented
/// quad scaled to the current shader's `portalRange` (falling back to `30`,
/// distance-attenuated, clamped to a `5.0` minimum). The `_XBOX` branch
/// (short-packed origin/normal) is dead on every target this port ships;
/// only the retail (non-`_XBOX`) body is in scope.
///
/// The `r_flares` early-out, the occlusion test, and the color-fade
/// computation need none of the blocked state below and are real CPU logic,
/// so they are transcribed.
///
/// DEFERRED: R4 — `tess.shader->portalRange` (`shaderCommands_t`) dissolves
/// entirely into R4's tessellation/vertex-building pipeline with no
/// "current shader being tessellated" carrier surviving at R3 (R2 `##
/// State ownership` row `tess`). Every statement past this point —
/// `radius`'s distance falloff/clamp, `left`/`up`, the `isMirror` flip, and
/// the final `RB_AddQuadStamp` call — depends on this unresolved value, so
/// none of it is transcribed.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:1929-2001`
pub fn RB_SurfaceFlare(
    surf: &srfFlare_t,
    ori: &orientationr_t,
    view: &viewParms_t,
    cvars: &RendererCvars,
    common: &Common,
    gpu: &mut GpuResources,
    frame: &mut FrameState,
) {
    if common.cvar(cvars.r_flares).integer == 0 {
        return;
    }

    if !RB_TestZFlare(surf.origin, ori, view, cvars, common, gpu) {
        return;
    }

    // calculate the xyz locations for the four corners
    let mut origin: vec3_t = [0.0; 3];
    VectorMA(surf.origin, 3.0, surf.normal, &mut origin);
    let snormal = surf.normal;

    let mut dir: vec3_t = [0.0; 3];
    VectorSubtract(origin, view.ori.origin, &mut dir);
    VectorNormalize(&mut dir);

    let mut d = -DotProduct(dir, snormal);
    if d < 0.0 {
        d = -d;
    }

    // fade the intensity of the flare down as the
    // light surface turns away from the viewer
    //
    // `byte color[4]` truncating float->u8 cast (Raven's own C conversion);
    // `d` is `abs(dot of two unit vectors)`, bounded to `[0, 1]`, so the
    // truncation never overflows.
    let color: [u8; 4] = [
        (d * 255.0) as u8,
        (d * 255.0) as u8,
        (d * 255.0) as u8,
        255, // only gets used if the shader has cgen exact_vertex!
    ];

    // DEFERRED: R4 — `radius = tess.shader->portalRange ? ... : 30` onward
    // (see doc comment above).
    // Source: oracle/codemp/renderer/tr_surface.cpp:1985-2000
    let _ = (color, frame);
}

/// Raven `RB_SurfaceSaberGlow` — the `SF_SABER_GLOW` dispatch-table entry:
/// stamps a shrinking trail of glow sprites down the saber blade
/// (`e->saberLength`..`0`, `DoSprite` per step) then a single larger,
/// slightly randomized "hilt glow" sprite at the entity's origin.
///
/// DEFERRED: escalation — the stepping loop's `e->saberLength`/`e->radius`
/// inputs are now real (`RefEntity::saber_length`/`radius`, campaign #41
/// batch 1; `oracle/codemp/cgame/tr_types.h:238,158`), so transcribing the
/// loop is a follow-up rewire. What still blocks the fn as a whole is the
/// trailing hilt-glow call's radius argument
/// (`5.5f + random() * 0.25f`), which needs `random()` — **not** the engine's own
/// `Q_random` LCG: `random()`/`crandom()` are a `#define` over libc `rand()`
/// (`oracle/codemp/game/q_shared.h:1591-1592`), a distinct generator this
/// crate's port convention places on the game tier's `bg_channel::rng::Rng`
/// (`Rng::random`/`Rng::crandom`, `native_math::qmath`'s module doc), not
/// reachable from the renderer — for which R2 assigns the renderer no
/// receiver (packet threading digest: "cite a `// DEFERRED:` if the wave
/// needs one" — the `CreateShape`/`DoBoltSeg` precedent, this file).
/// `DoSprite`, the sole callee, is itself already ported and not the
/// blocker.
///
/// Whole-body deferral: the trailing hilt-glow sprite is unconditional and
/// depends on `random()`, so this stays a loud `todo!()` rather than a
/// silent no-op (whole-fn-deferral convention — partial-body fns keep
/// DEFERRED comments instead).
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:560-580`
pub fn RB_SurfaceSaberGlow(
    current_entity: Option<&RefEntity>,
    view: &viewParms_t,
    frame: &mut FrameState,
) {
    let _ = (current_entity, view, frame);
    todo!("Port RB_SurfaceSaberGlow — oracle/codemp/renderer/tr_surface.cpp:560-580")
}

/// Raven `RB_SurfaceElectricity` — the `SF_ENTITY` "electricity" surface:
/// grows/anchors a lightning bolt from the current entity's `origin` to
/// `oldorigin` (optionally animating the endpoint under `RF_GROW`), then
/// hands the resulting segment plus a view-relative "right" vector to
/// `DoBoltSeg`.
///
/// `backEnd.currentEntity` becomes `Option<&RefEntity>` (the `RB_SurfaceBeam`
/// precedent, this file, wave 0); `backEnd.viewParms` is threaded as the
/// already-ported tier-2 `viewParms_t` directly (the `RB_TestZFlare`
/// precedent, this file) — `FrameState::view` is still the empty `ViewParms`
/// landing placeholder. `tr` is threaded via `frame: &FrameState` per the
/// packet's SPLIT `tr` row.
///
/// The `start`/`fwd`/`dis` setup (needs only `origin`/`oldorigin`, both
/// landed on `RefEntity`) is real CPU logic and is transcribed.
///
/// DEFERRED: escalation — everything past that setup (see doc comment
/// above). `radius = e->radius` and the `RF_GROW` mask are both closed now
/// (`RefEntity::radius`, campaign #41 batch 1; `tr_public::ref_flags
/// ::RF_GROW`), so those two are a follow-up rewire. What still blocks the
/// growth branch is `tr.refdef.time` (`FrameState::refdef`'s `TrRefdef` has
/// no `time` field yet, packet STATE HOMES row `RB_SurfaceElectricity`/`tr`).
/// The subsequent `VectorMA(..., e->oldorigin)` writes
/// the entity's `oldorigin`
/// in place — a mutation this file's `Option<&RefEntity>` shape cannot
/// express (the same shape as every other dispatch entry in this file). The
/// final `right`-vector computation and `DoBoltSeg` call are unreachable
/// without that write's output (`end`) and `radius`.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:1127-1169`
pub fn RB_SurfaceElectricity(
    current_entity: Option<&RefEntity>,
    view: &viewParms_t,
    frame: &FrameState,
) {
    let Some(e) = current_entity else {
        return;
    };

    let start = e.origin;

    let mut fwd: vec3_t = [0.0; 3];
    VectorSubtract(e.old_origin, start, &mut fwd);
    let dis = VectorNormalize(&mut fwd);

    // DEFERRED: escalation — `radius = e->radius` onward (see doc comment
    // above).
    // Source: oracle/codemp/renderer/tr_surface.cpp:1137,1145-1169
    let _ = (dis, view, frame);
}

/// Raven `RB_SurfaceEntity` — the `SF_ENTITY` dispatch table: routes to the
/// per-`reType` surface fn for the current entity (sprite/oriented-quad/beam/
/// electricity/line/oriented-line/saber-glow/cylinder), falling back to
/// `RB_SurfaceAxis` for every other type (`RT_MODEL`, `RT_POLY`,
/// `RT_PORTALSURFACE`, `RT_CLOUDS`, ...).
///
/// `backEnd.currentEntity` becomes `frame.current_entity: Option<RefEntity>`
/// (the `RB_SurfaceBeam` precedent, this file, wave 0), cloned once up front
/// so the dispatch arms below can pass `Option<&RefEntity>` downward while
/// still holding `frame: &mut FrameState` for the arms that need it mutably
/// (`RB_SurfaceSprite`/`RB_SurfaceOrientedQuad`/`RB_SurfaceSaberGlow`).
/// `backEnd.viewParms` is threaded as the already-ported tier-2
/// `viewParms_t` directly (the `RB_TestZFlare` precedent, this file) —
/// `FrameState::view` is still the empty `ViewParms` landing placeholder.
///
/// DEFERRED: escalation — the `RT_ENT_CHAIN` case's body
/// (`tr_surface.cpp:1840-1861`) is unreachable: its `static trRefEntity_t
/// tempEnt = *backEnd.currentEntity;` is this packet's own STATE HOMES/
/// THREADING DIGEST fn-scope static (`tempEnt`), a genuine cross-frame kind-3
/// static per the three-kind rule (it survives across calls, initialized only
/// once at first execution per the oracle's own `//rww` comment) — R2 assigns
/// the renderer no carrier for any kind-3 fn-scope static, so it is an
/// escalation, never an invented field (preamble). Independently blocked
/// too: `e->e.uRefEnt.uMini.miniStart`/`::miniCount` are fields of
/// `refEntity_t`'s `uRefEnt` union (`oracle/codemp/cgame/tr_types.h:135-231`)
/// not among the ones wave 0 landed on `RefEntity`
/// (`render_state/placeholders.rs`), and `backEnd.refdef.miniEntities` has no
/// `TrRefdef` field either — only `fov_x`/`fov_y`/`view_origin`/`view_axis`
/// are landed there. A state home this packet marks mapped-but-not-yet-
/// populated is an escalation, not an invention (preamble).
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:1812-1868`
pub fn RB_SurfaceEntity(surf_type: &surfaceType_t, frame: &mut FrameState, view: &viewParms_t) {
    let current_entity = frame.current_entity.clone();
    let Some(e) = &current_entity else {
        return;
    };

    match e.re_type {
        refEntityType_t::RT_SPRITE => {
            RB_SurfaceSprite(current_entity.as_ref(), view, frame);
        }
        refEntityType_t::RT_ORIENTED_QUAD => {
            RB_SurfaceOrientedQuad(current_entity.as_ref(), view, frame);
        }
        refEntityType_t::RT_BEAM => {
            RB_SurfaceBeam(current_entity.as_ref());
        }
        refEntityType_t::RT_ELECTRICITY => {
            RB_SurfaceElectricity(current_entity.as_ref(), view, &*frame);
        }
        refEntityType_t::RT_LINE => {
            RB_SurfaceLine(current_entity.as_ref(), view);
        }
        refEntityType_t::RT_ORIENTEDLINE => {
            RB_SurfaceOrientedLine(current_entity.as_ref());
        }
        refEntityType_t::RT_SABER_GLOW => {
            RB_SurfaceSaberGlow(current_entity.as_ref(), view, frame);
        }
        refEntityType_t::RT_CYLINDER => {
            RB_SurfaceCylinder(current_entity.as_ref(), view);
        }
        refEntityType_t::RT_ENT_CHAIN => {
            // DEFERRED: escalation — RT_ENT_CHAIN's miniEntities-chain fanout
            // (see doc comment above): `tempEnt` fn-scope static is an
            // unhomed kind-3 escalation; `RefEntity::uRefEnt`/`TrRefdef::
            // miniEntities` not landed.
            // Source: oracle/codemp/renderer/tr_surface.cpp:1839-1861
            let _ = surf_type;
        }
        _ => {
            RB_SurfaceAxis();
        }
    }
}
