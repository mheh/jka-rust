//! Raven `tr_main.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_main.cpp`

#![allow(non_snake_case)]
// Wave-0 ports of Raven `static` helpers and consts: private by fidelity, with
// their callers landing in later R3 waves.
#![allow(dead_code)]

// PORT-NOTE (wave 0, `tr_main.wave0.md`): the R2-frozen carriers this file's
// STATE HOMES rows name (`FrameState::view`/`::ori`/`::refdef`,
// `RenderAssets`) are still empty landing placeholders
// (`render_state/placeholders.rs`'s `ViewParms`/`OrientationR`/`TrRefdef`) —
// their doc comments name "the `tr_main` R3 wave" as the wave that fills
// them, but this packet restricts a wave-0 transcriber to this one file. The
// functions below thread the concrete pieces of that state (`orientationr_t`,
// `viewParms_t`, `fog_t` — the already-ported, already pointer-free tier-2
// shapes under `tr_local/`) as explicit parameters/returns instead of the
// not-yet-populated R2 carrier types, per the interior-safety law's own
// carve-out ("Tier-2 fields may be *read* through their existing shapes
// until their owning wave replaces them") and porting-rules §B4 ("state is
// threaded, not reached"). Flagged for the integrator: once `FrameState`'s
// `view`/`ori`/`refdef` fields land with real shapes, call sites here take
// `&frame.view`/`&frame.ori` slices instead of the tier-2 types directly.

use crate::render_state::shader_asset::ShaderHandle;
use crate::tr_local::fog_t::fog_t;
use crate::tr_local::orientationr_t::orientationr_t;
use crate::tr_local::view_parms_t::viewParms_t;

use core::f64::consts::PI;

use mp_engine_qcommon::qfiles::draw_vert_t::drawVert_t;
use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::shared::q_math::{
    _DotProduct as DotProduct, _VectorAdd as VectorAdd, _VectorCopy as VectorCopy,
    _VectorMA as VectorMA, _VectorScale as VectorScale, _VectorSubtract as VectorSubtract,
    DistanceSquared, SetPlaneSignbits, VectorClear,
};
use mp_qshared::shared::{cplane_t, orientation_t, vec3_t, vec4_t};
// `PlaneFromPoints` has no `mp_qshared::shared::q_math` re-export (unlike the
// other `q_math` helpers above); taken from its canonical `native_math` home,
// the same edge `tr_shade_calc` uses for `Q_rsqrt`.
use native_math::qmath::PlaneFromPoints;

/// Raven `CULL_IN` — completely unclipped.
///
/// Source: `oracle/codemp/renderer/tr_local.h`
pub const CULL_IN: i32 = 0;

/// Raven `CULL_CLIP` — clipped by one or more planes.
///
/// Source: `oracle/codemp/renderer/tr_local.h`
pub const CULL_CLIP: i32 = 1;

/// Raven `CULL_OUT` — completely outside the clipping planes.
///
/// Source: `oracle/codemp/renderer/tr_local.h`
pub const CULL_OUT: i32 = 2;

/// Raven `PLANE_NON_AXIAL` — `cplane_t::type`'s "not one of the three cardinal
/// axes" value (`PLANE_X`/`PLANE_Y`/`PLANE_Z` = 0/1/2, already ported at
/// `mp_qshared::shared::collision`).
///
/// Source: `oracle/codemp/game/q_shared.h`
const PLANE_NON_AXIAL: u8 = 3;

/// Raven `RDF_NOWORLDMODEL`. Value confirmed against the already-ported
/// `crates/mp/uishared/src/ui_shared.rs:171` (`refdef.rdflags =
/// RDF_NOWORLDMODEL`), which reuses the same literal.
///
/// Source: `oracle/codemp/cgame/tr_types.h`
const RDF_NOWORLDMODEL: i32 = 1;

// PORT-NOTE: RDF_AUTOMAP/RDF_NOFOG have no already-ported anchor anywhere in
// this tree to confirm their bit value against — `tr_types.h`'s RDF_* block
// isn't in this packet's oracle slice, and cgame (the only other rdflags
// writer) isn't ported yet. Values below follow the same RDF_* bit-flag
// family as the confirmed `RDF_NOWORLDMODEL`; flagged for confirmation
// against `oracle/codemp/cgame/tr_types.h` the next time that header's RDF_*
// block is actually read by a porter.
const RDF_AUTOMAP: i32 = 1 << 12;
const RDF_NOFOG: i32 = 1 << 8;

/// Raven `MAX_SHADERS` (non-`_XBOX` branch) — local copy of the private
/// const already ported at `tr_local::tr_globals_t` (not `pub`, so not
/// reachable from here).
///
/// Source: `oracle/codemp/renderer/tr_local.h:40-46`
const MAX_SHADERS: usize = 16384;

/// Raven `MAX_ENTITIES` — cited directly from the R2 design's `backEndData_t`
/// disposition entry ("entities[MAX_ENTITIES=2048]").
const MAX_ENTITIES: i32 = 2048;

// PORT-NOTE: QSORT_FOGNUM_SHIFT/QSORT_ENTITYNUM_SHIFT/QSORT_SHADERNUM_SHIFT
// (tr_local.h #defines) aren't in this packet's resolved call surface and
// this file's oracle slice doesn't show their header block. `R_AddDrawSurf`/
// `R_DecomposeSort`'s sort key is pure renderer interior (DEC-37 ruling 1:
// never serialized, never compared against the oracle byte-for-byte — only
// its resulting *relative order* is observable) so the exact shift amounts
// don't need to match Raven's literal macro values, only preserve field
// priority (shader > entity > fog > dlight) with non-overlapping bit ranges.
// Derived here from what IS certain from the verbatim oracle body:
// `dlightMap = sort & 3` (2 bits, bits 0-1) and `fogNum = (sort>>SHIFT) & 31`
// (5 bits) are literal in the packet's `R_DecomposeSort` source; entity needs
// `MAX_ENTITIES=2048` => 11 bits, shader needs `MAX_SHADERS=16384` => 14
// bits. 2+5+11+14 = 32, so this packing exactly fills a u32 with zero
// overlap: dlight[0-1], fog[2-6], entity[7-17], shader[18-31].
const QSORT_FOGNUM_SHIFT: u32 = 2;
const QSORT_ENTITYNUM_SHIFT: u32 = 7;
const QSORT_SHADERNUM_SHIFT: u32 = 18;

/// Raven `Com_Clamp`, inlined locally: the canonical body lives at
/// `crates/native/math/src/qmath.rs::Com_Clamp`, but `mp_renderer` has no
/// `native_math` crate dependency to reach it through (only `mp_qshared`'s
/// re-export surface, which doesn't carry this one). Trivial 3-line body,
/// reproduced rather than adding a crate dependency from this file alone.
///
/// Source: `oracle/codemp/game/q_shared.c:64-72`
fn com_clamp(min: f32, max: f32, value: f32) -> f32 {
    if value < min {
        return min;
    }
    if value > max {
        return max;
    }
    value
}

/// Per-subsystem carrier for `tr_main.cpp`'s file-scope `preTransEntMatrix`
/// static — set by `R_RotateForEntity` (not in this wave's packet) and read
/// by `R_WorldNormalToEntity`. NAMED BY THIS WAVE per DEC-37 A13.3: a
/// cross-call scratch value that is neither a const table nor a per-call
/// return value (kind 3 of the fn-scope-statics three-kind rule).
pub struct TrMainScratch {
    /// Raven `preTransEntMatrix` — row-major 4x4 model matrix set by
    /// `R_RotateForEntity`, consumed by `R_WorldNormalToEntity`.
    ///
    /// Source: `oracle/codemp/renderer/tr_main.cpp` (file-scope static)
    pub pre_trans_ent_matrix: [f32; 16],
}

/// A minimal borrowed view over Raven's `surfaceType_t *` tagged-union
/// pointer, covering only the surface kinds `R_PlaneForSurface` reads
/// (`SF_FACE`/`SF_TRIANGLES`/`SF_POLY`); every other kind (including the
/// oracle's default arm) maps to `Other`. The eventual world-owned surface
/// representation lands with `tr_bsp`/`tr_world` (tier-2 transition audit,
/// `msurface_t` row: "an owned `Surface` enum ... replacing the
/// `surfaceType_t` dispatch pointer") — this wave only needs read access for
/// the one pure function below.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:656-678`
pub enum SurfaceGeometry<'a> {
    Face(cplane_t),
    Triangles {
        verts: &'a [drawVert_t],
        indexes: &'a [i32],
    },
    Poly {
        verts: &'a [polyVert_t],
    },
    Other,
}

/// The owned replacement for Raven `drawSurf_t` (`tr_local::draw_surf_s`,
/// `sort: u32, surface: *mut surfaceType_t`) — `surface` becomes an owned/
/// borrowed value instead of a raw tagged-union pointer (interior-safety
/// law). Generic over the concrete surface representation because the real
/// arena/handle shape lands with `tr_bsp`/`tr_world` (tier-2 transition
/// audit, `drawSurf_t` row: "`surface` -> a `Handle`/index into the surface
/// arena").
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:680-683`
pub struct DrawSurf<S> {
    pub sort: u32,
    pub surface: S,
}

/// Raven `R_CullLocalBox`.
///
/// Raven: transforms `bounds` into world space then checks against the
/// frustum planes.
///
/// `r_nocull_integer` is `r_nocull->integer` (`RendererCvars::r_nocull`,
/// DEC-37 A13.1 — read through the live engine cvar table by the caller,
/// threaded in here rather than reached for); `ori`/`frustum` are
/// `tr.ori`/`tr.viewParms.frustum` (STATE HOMES: SPLIT `RenderAssets` +
/// `FrameState`, see this file's top-of-file PORT-NOTE for the tier-2
/// stand-in).
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:48-102`
pub fn R_CullLocalBox(
    bounds: [vec3_t; 2],
    r_nocull_integer: i32,
    ori: &orientationr_t,
    frustum: &[cplane_t; 4],
) -> i32 {
    if r_nocull_integer == 1 {
        return CULL_CLIP;
    }

    // transform into world space
    let mut transformed = [[0.0f32; 3]; 8];
    for i in 0..8usize {
        let v = [
            bounds[i & 1][0],
            bounds[(i >> 1) & 1][1],
            bounds[(i >> 2) & 1][2],
        ];

        VectorCopy(ori.origin, &mut transformed[i]);
        VectorMA(transformed[i], v[0], ori.axis[0], &mut transformed[i]);
        VectorMA(transformed[i], v[1], ori.axis[1], &mut transformed[i]);
        VectorMA(transformed[i], v[2], ori.axis[2], &mut transformed[i]);
    }

    // check against frustum planes
    let mut any_back = false;
    for i in 0..4usize {
        let frust = &frustum[i];

        let mut front = false;
        let mut back = false;
        for j in 0..8usize {
            let dist = DotProduct(transformed[j], frust.normal);
            if dist > frust.dist {
                front = true;
                if back {
                    break; // a point is in front
                }
            } else {
                back = true;
            }
        }
        if !front {
            // all points were behind one of the planes
            return CULL_OUT;
        }
        any_back |= back;
    }

    if !any_back {
        return CULL_IN; // completely inside frustum
    }

    CULL_CLIP // partially clipped
}

/// Raven `R_CullPointAndRadius`.
///
/// `r_nocull_integer`/`frustum` as `R_CullLocalBox` above.
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:121-154`
pub fn R_CullPointAndRadius(
    pt: vec3_t,
    radius: f32,
    r_nocull_integer: i32,
    frustum: &[cplane_t; 4],
) -> i32 {
    if r_nocull_integer == 1 {
        return CULL_CLIP;
    }

    let mut might_be_clipped = false;

    // check against frustum planes
    for i in 0..4usize {
        let frust = &frustum[i];

        let dist = DotProduct(pt, frust.normal) - frust.dist;
        if dist < -radius {
            return CULL_OUT;
        } else if dist <= radius {
            might_be_clipped = true;
        }
    }

    if might_be_clipped {
        return CULL_CLIP;
    }

    CULL_IN // completely inside frustum
}

/// Raven `R_LocalNormalToWorld`. Out-param `world` -> return value.
///
/// `ori` is `tr.ori` (STATE HOMES SPLIT, see top-of-file PORT-NOTE).
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:164-168`
pub fn R_LocalNormalToWorld(local: vec3_t, ori: &orientationr_t) -> vec3_t {
    [
        local[0] * ori.axis[0][0] + local[1] * ori.axis[1][0] + local[2] * ori.axis[2][0],
        local[0] * ori.axis[0][1] + local[1] * ori.axis[1][1] + local[2] * ori.axis[2][1],
        local[0] * ori.axis[0][2] + local[1] * ori.axis[1][2] + local[2] * ori.axis[2][2],
    ]
}

/// Raven `R_LocalPointToWorld`. Out-param `world` -> return value.
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:177-181`
pub fn R_LocalPointToWorld(local: vec3_t, ori: &orientationr_t) -> vec3_t {
    [
        local[0] * ori.axis[0][0]
            + local[1] * ori.axis[1][0]
            + local[2] * ori.axis[2][0]
            + ori.origin[0],
        local[0] * ori.axis[0][1]
            + local[1] * ori.axis[1][1]
            + local[2] * ori.axis[2][1]
            + ori.origin[1],
        local[0] * ori.axis[0][2]
            + local[1] * ori.axis[1][2]
            + local[2] * ori.axis[2][2]
            + ori.origin[2],
    ]
}

/// Raven `R_WorldNormalToEntity`. Out-param `entvec` -> return value.
///
/// `scratch` carries `preTransEntMatrix` (DEC-37 A13.3, `TrMainScratch`
/// above).
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:193-198`
pub fn R_WorldNormalToEntity(worldvec: vec3_t, scratch: &TrMainScratch) -> vec3_t {
    let m = &scratch.pre_trans_ent_matrix;
    [
        -worldvec[0] * m[0] - worldvec[1] * m[4] + worldvec[2] * m[8],
        -worldvec[0] * m[1] - worldvec[1] * m[5] + worldvec[2] * m[9],
        -worldvec[0] * m[2] - worldvec[1] * m[6] + worldvec[2] * m[10],
    ]
}

/// Raven `R_WorldToLocal`. Out-param `local` -> return value.
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:220-224`
pub fn R_WorldToLocal(world: vec3_t, ori: &orientationr_t) -> vec3_t {
    [
        DotProduct(world, ori.axis[0]),
        DotProduct(world, ori.axis[1]),
        DotProduct(world, ori.axis[2]),
    ]
}

/// Raven `R_TransformModelToClip`. Out-params `eye`/`dst` -> return value.
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:232-251`
pub fn R_TransformModelToClip(
    src: vec3_t,
    model_matrix: &[f32; 16],
    projection_matrix: &[f32; 16],
) -> (vec4_t, vec4_t) {
    let mut eye: vec4_t = [0.0; 4];
    for i in 0..4usize {
        eye[i] = src[0] * model_matrix[i]
            + src[1] * model_matrix[i + 1 * 4]
            + src[2] * model_matrix[i + 2 * 4]
            + 1.0 * model_matrix[i + 3 * 4];
    }

    let mut dst: vec4_t = [0.0; 4];
    for i in 0..4usize {
        dst[i] = eye[0] * projection_matrix[i]
            + eye[1] * projection_matrix[i + 1 * 4]
            + eye[2] * projection_matrix[i + 2 * 4]
            + eye[3] * projection_matrix[i + 3 * 4];
    }

    (eye, dst)
}

/// Raven `R_TransformClipToWindow`. Out-params `normalized`/`window` ->
/// return value.
///
/// `view` is `tr.viewParms` (`viewportWidth`/`viewportHeight` only). `window
/// [3]` is left `0.0` — the oracle's `vec4_t window` leaves its 4th component
/// unwritten too (§19: a defined stand-in for what was caller-stack garbage,
/// out of scope of anything either function reads).
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:259-270`
pub fn R_TransformClipToWindow(clip: vec4_t, view: &viewParms_t) -> (vec4_t, vec4_t) {
    let mut normalized: vec4_t = [0.0; 4];
    normalized[0] = clip[0] / clip[3];
    normalized[1] = clip[1] / clip[3];
    normalized[2] = (clip[2] + clip[3]) / (2.0 * clip[3]);

    let mut window: vec4_t = [0.0; 4];
    window[0] = 0.5 * (1.0 + normalized[0]) * view.viewportWidth as f32;
    window[1] = 0.5 * (1.0 + normalized[1]) * view.viewportHeight as f32;
    window[2] = normalized[2];

    window[0] = (window[0] + 0.5) as i32 as f32;
    window[1] = (window[1] + 0.5) as i32 as f32;

    (normalized, window)
}

/// Raven `myGlMultMatrix`. Out-param `out` -> return value.
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:279-291`
pub fn myGlMultMatrix(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
    let mut out = [0.0f32; 16];
    for i in 0..4usize {
        for j in 0..4usize {
            out[i * 4 + j] = a[i * 4] * b[j]
                + a[i * 4 + 1] * b[1 * 4 + j]
                + a[i * 4 + 2] * b[2 * 4 + j]
                + a[i * 4 + 3] * b[3 * 4 + j];
        }
    }
    out
}

/// Raven `SetFarClip`.
///
/// Raven: if not rendering the world (icons, menus, etc) set a 2k far clip
/// plane; otherwise set far clipping planes dynamically, bringing in `zFar`
/// to the distance-cull distance (the sky renders at `zFar` so needs to move
/// out a little; a minimum `zFar` prevents problems).
///
/// `refdef_rdflags` is `tr.refdef.rdflags`; `view` is `tr.viewParms`
/// (written); `distance_cull` is `tr.distanceCull` (`RenderAssets
/// ::distance_cull`, B11).
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:414-486`
pub fn SetFarClip(refdef_rdflags: i32, view: &mut viewParms_t, distance_cull: f32) {
    // if not rendering the world (icons, menus, etc)
    // set a 2k far clip plane
    if refdef_rdflags & RDF_NOWORLDMODEL != 0 {
        if refdef_rdflags & RDF_AUTOMAP != 0 {
            // override the zfar then
            view.zFar = 32768.0;
        } else {
            view.zFar = 2048.0;
        }
        return;
    }

    //
    // set far clipping planes dynamically
    //
    let mut farthest_corner_distance = 0.0f32;
    for i in 0..8usize {
        let mut v: vec3_t = [0.0; 3];

        v[0] = if i & 1 != 0 {
            view.visBounds[0][0]
        } else {
            view.visBounds[1][0]
        };

        v[1] = if i & 2 != 0 {
            view.visBounds[0][1]
        } else {
            view.visBounds[1][1]
        };

        v[2] = if i & 4 != 0 {
            view.visBounds[0][2]
        } else {
            view.visBounds[1][2]
        };

        let distance = DistanceSquared(view.ori.origin, v);
        if distance > farthest_corner_distance {
            farthest_corner_distance = distance;
        }
    }
    // Bring in the zFar to the distanceCull distance
    // The sky renders at zFar so need to move it out a little
    // ...and make sure there is a minimum zfar to prevent problems
    view.zFar = com_clamp(
        2048.0,
        // C promotes to double; f64 intermediate per wave-0 ruling 12.
        (distance_cull as f64 * 1.732) as f32,
        farthest_corner_distance.sqrt(),
    );
}

/// Raven `R_SetupFrustum`.
///
/// `view` is `tr.viewParms` (written: `frustum`, via its nested `ori`
/// read).
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:568-598`
pub fn R_SetupFrustum(view: &mut viewParms_t) {
    // C promotes to double; f64 intermediate per wave-0 ruling 12. Raven's
    // `fovX / 180` is still a float divide (both operands float after the int
    // conversion); `M_PI` is what widens the chain.
    let mut ang = ((view.fovX / 180.0) as f64 * PI * 0.5) as f32;
    // C `sin`/`cos` are double fns; f64 intermediate per wave-0 ruling 12.
    let mut xs = f64::sin(ang as f64) as f32;
    let mut xc = f64::cos(ang as f64) as f32;

    VectorScale(view.ori.axis[0], xs, &mut view.frustum[0].normal);
    let n0 = view.frustum[0].normal;
    VectorMA(n0, xc, view.ori.axis[1], &mut view.frustum[0].normal);

    VectorScale(view.ori.axis[0], xs, &mut view.frustum[1].normal);
    let n1 = view.frustum[1].normal;
    VectorMA(n1, -xc, view.ori.axis[1], &mut view.frustum[1].normal);

    // C promotes to double; f64 intermediate per wave-0 ruling 12.
    ang = ((view.fovY / 180.0) as f64 * PI * 0.5) as f32;
    xs = f64::sin(ang as f64) as f32;
    xc = f64::cos(ang as f64) as f32;

    VectorScale(view.ori.axis[0], xs, &mut view.frustum[2].normal);
    let n2 = view.frustum[2].normal;
    VectorMA(n2, xc, view.ori.axis[2], &mut view.frustum[2].normal);

    VectorScale(view.ori.axis[0], xs, &mut view.frustum[3].normal);
    let n3 = view.frustum[3].normal;
    VectorMA(n3, -xc, view.ori.axis[2], &mut view.frustum[3].normal);

    for i in 0..4usize {
        view.frustum[i].r#type = PLANE_NON_AXIAL;
        view.frustum[i].dist = DotProduct(view.ori.origin, view.frustum[i].normal);
        SetPlaneSignbits(&mut view.frustum[i]);
    }
}

/// Raven `R_MirrorPoint`. Out-param `out` -> return value.
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:606-621`
pub fn R_MirrorPoint(in_pt: vec3_t, surface: &orientation_t, camera: &orientation_t) -> vec3_t {
    let mut local: vec3_t = [0.0; 3];
    VectorSubtract(in_pt, surface.origin, &mut local);

    let mut transformed: vec3_t = [0.0; 3];
    VectorClear(&mut transformed);
    for i in 0..3usize {
        let d = DotProduct(local, surface.axis[i]);
        VectorMA(transformed, d, camera.axis[i], &mut transformed);
    }

    let mut out: vec3_t = [0.0; 3];
    VectorAdd(transformed, camera.origin, &mut out);
    out
}

/// Raven `R_MirrorVector`. Out-param `out` -> return value.
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:623-632`
pub fn R_MirrorVector(in_v: vec3_t, surface: &orientation_t, camera: &orientation_t) -> vec3_t {
    let mut out: vec3_t = [0.0; 3];
    VectorClear(&mut out);
    for i in 0..3usize {
        let d = DotProduct(in_v, surface.axis[i]);
        VectorMA(out, d, camera.axis[i], &mut out);
    }
    out
}

/// Raven `R_PlaneForSurface`. Out-param `plane` -> return value.
///
/// PORT-NOTE: the oracle mutates only `plane->normal`/`plane->dist` on a
/// caller-owned `cplane_t`, leaving `type`/`signbits` at whatever the
/// caller's stack held. The out-param -> return-value translation
/// constructs a fresh `cplane_t` instead, so `type`/`signbits` default to
/// `0` rather than carrying forward caller state; callers that need real
/// `type`/`signbits` call `SetPlaneSignbits`/`PlaneTypeForNormal` themselves
/// (already-ported, `mp_qshared::shared::q_math`).
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:640-675`
pub fn R_PlaneForSurface(surf_type: Option<&SurfaceGeometry>) -> cplane_t {
    let default_plane = || cplane_t {
        normal: [1.0, 0.0, 0.0],
        dist: 0.0,
        r#type: 0,
        signbits: 0,
        pad: [0, 0],
    };

    let Some(surf_type) = surf_type else {
        return default_plane();
    };

    match surf_type {
        SurfaceGeometry::Face(plane) => *plane,
        SurfaceGeometry::Triangles { verts, indexes } => {
            let v1 = verts[indexes[0] as usize].xyz;
            let v2 = verts[indexes[1] as usize].xyz;
            let v3 = verts[indexes[2] as usize].xyz;
            let mut plane4: vec4_t = [0.0; 4];
            PlaneFromPoints(&mut plane4, v1, v2, v3);
            cplane_t {
                normal: [plane4[0], plane4[1], plane4[2]],
                dist: plane4[3],
                r#type: 0,
                signbits: 0,
                pad: [0, 0],
            }
        }
        SurfaceGeometry::Poly { verts } => {
            let mut plane4: vec4_t = [0.0; 4];
            PlaneFromPoints(&mut plane4, verts[0].xyz, verts[1].xyz, verts[2].xyz);
            cplane_t {
                normal: [plane4[0], plane4[1], plane4[2]],
                dist: plane4[3],
                r#type: 0,
                signbits: 0,
                pad: [0, 0],
            }
        }
        SurfaceGeometry::Other => default_plane(),
    }
}

/// Raven `R_SpriteFogNum`.
///
/// `rdflags` is `tr.refdef.rdflags`; `fogs` is `tr.world->fogs` (index 0 is
/// the oracle's reserved "no fog" slot, matching `for (i=1; i<numfogs; i++)`
/// below); `ent_origin`/`ent_radius` are `ent->e.origin`/`ent->e.radius`.
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:1028-1052`
pub fn R_SpriteFogNum(rdflags: i32, fogs: &[fog_t], ent_origin: vec3_t, ent_radius: f32) -> i32 {
    if rdflags & RDF_NOWORLDMODEL != 0 {
        return 0;
    }

    for i in 1..fogs.len() {
        let fog = &fogs[i];
        let mut j = 0usize;
        while j < 3 {
            if ent_origin[j] - ent_radius >= fog.bounds[1][j] {
                break;
            }
            if ent_origin[j] + ent_radius <= fog.bounds[0][j] {
                break;
            }
            j += 1;
        }
        if j == 3 {
            return i as i32;
        }
    }

    0
}

/// Raven `shortsort`. Operates on the whole slice (Raven's `[lo, hi]` pointer
/// range with `lo` fixed at the start of the range it's called with).
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:1076-1090`
pub fn shortsort<S>(surfs: &mut [DrawSurf<S>]) {
    if surfs.is_empty() {
        return;
    }
    let mut hi = surfs.len() - 1;
    while hi > 0 {
        let mut max = 0usize;
        for p in 1..=hi {
            if surfs[p].sort > surfs[max].sort {
                max = p;
            }
        }
        surfs.swap(max, hi);
        hi -= 1;
    }
}

/// Raven `R_AddDrawSurf`.
///
/// `shader_sorted_index` is `shader->sortedIndex`; `shifted_entity_num` is
/// `tr.shiftedEntityNum`; `rdf_nofog` is `tr.refdef.rdflags & RDF_NOFOG`;
/// `draw_surfs` is `tr.refdef.drawSurfs` (STATE HOMES SPLIT, `FrameData`'s
/// append-validation carrier per the R2 design).
///
/// PORT-NOTE: the oracle masks the append index with `DRAWSURF_MASK` so a
/// full `drawSurfs[MAX_DRAWSURFS]` ring buffer silently wraps and overwrites
/// its oldest entry ("instead of checking for overflow, we just mask the
/// index so it wraps around"). Per the R2 design's own A1 disposition
/// ("`backEndData_t` dissolves ... the reference vocabulary for `FrameData`'s
/// event payloads, not a struct that survives"), the fixed-size ring buffer
/// is replaced by a plain `Vec` append — no wraparound-mask hack needed.
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:1262-1284`
pub fn R_AddDrawSurf<S>(
    surface: S,
    shader_sorted_index: i32,
    shifted_entity_num: i32,
    rdf_nofog: bool,
    fog_index: i32,
    dlight_map: i32,
    draw_surfs: &mut Vec<DrawSurf<S>>,
) {
    let fog_index = if rdf_nofog { 0 } else { fog_index };

    // the sort data is packed into a single 32 bit value so it can be
    // compared quickly during the qsorting process
    let sort = ((shader_sorted_index as u32) << QSORT_SHADERNUM_SHIFT)
        | (shifted_entity_num as u32)
        | ((fog_index as u32) << QSORT_FOGNUM_SHIFT)
        | (dlight_map as u32);

    draw_surfs.push(DrawSurf { sort, surface });
}

/// Raven `R_DecomposeSort`. Out-params `entityNum`/`shader`/`fogNum`/
/// `dlightMap` -> return value (same order).
///
/// `sorted_shaders` is `tr.sortedShaders` (`RenderAssets`, `R2-D3`/`R2-D4`).
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:1291-1297`
pub fn R_DecomposeSort(
    sort: u32,
    sorted_shaders: &[ShaderHandle],
) -> (i32, ShaderHandle, i32, i32) {
    let fog_num = ((sort >> QSORT_FOGNUM_SHIFT) & 31) as i32;
    let shader = sorted_shaders[((sort >> QSORT_SHADERNUM_SHIFT) as usize) & (MAX_SHADERS - 1)];
    let entity_num = ((sort >> QSORT_ENTITYNUM_SHIFT) as i32) & (MAX_ENTITIES - 1);
    let dlight_map = (sort & 3) as i32;
    (entity_num, shader, fog_num, dlight_map)
}
