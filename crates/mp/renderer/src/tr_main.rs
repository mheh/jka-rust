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

use crate::render_state::renderer_cvars::RendererCvars;
use crate::render_state::shader_asset::ShaderHandle;
use crate::tr_local::fog_t::fog_t;
use crate::tr_local::orientationr_t::orientationr_t;
use crate::tr_local::tr_ref_entity_t::trRefEntity_t;
use crate::tr_local::view_parms_t::viewParms_t;

use core::f64::consts::PI;

use mp_engine_qcommon::common::Common;
use mp_engine_qcommon::qfiles::draw_vert_t::drawVert_t;
use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::shared::q_math::{
    _DotProduct as DotProduct, _VectorAdd as VectorAdd, _VectorCopy as VectorCopy,
    _VectorMA as VectorMA, _VectorScale as VectorScale, _VectorSubtract as VectorSubtract,
    vec3_origin, CrossProduct, DistanceSquared, PerpendicularVector, SetPlaneSignbits, VectorClear,
    VectorLength,
};
use mp_qshared::shared::{cplane_t, orientation_t, vec3_t, vec4_t};
// `PlaneFromPoints`/`RotatePointAroundVector` have no `mp_qshared::shared::
// q_math` re-export (unlike the other `q_math` helpers above); taken from
// their canonical `native_math` home, the same edge `tr_shade_calc` uses for
// `Q_rsqrt`.
use native_math::qmath::{PlaneFromPoints, RotatePointAroundVector};

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

/// Raven `RDF_NOWORLDMODEL`.
///
/// Raven: used for player configuration screen.
///
/// Source: `oracle/codemp/cgame/tr_types.h:57`
const RDF_NOWORLDMODEL: i32 = 1;

/// Raven `RDF_AUTOMAP`.
///
/// Raven: means this scene is to draw the automap -rww.
///
/// Source: `oracle/codemp/cgame/tr_types.h:63`
const RDF_AUTOMAP: i32 = 32;

/// Raven `RDF_NOFOG`.
///
/// Raven: no global fog in this scene (but still brush fog) -rww.
///
/// Source: `oracle/codemp/cgame/tr_types.h:64`
const RDF_NOFOG: i32 = 64;

/// Raven `MAX_SHADERS` (non-`_XBOX` branch) — local copy of the private
/// const already ported at `tr_local::tr_globals_t` (not `pub`, so not
/// reachable from here).
///
/// Source: `oracle/codemp/renderer/tr_local.h:40-46`
const MAX_SHADERS: usize = 16384;

/// Raven `MAX_ENTITIES` — cited directly from the R2 design's `backEndData_t`
/// disposition entry ("entities[MAX_ENTITIES=2048]").
const MAX_ENTITIES: i32 = 2048;

/// Raven `TR_WORLDENT` — local copy of the private const already ported at
/// `tr_scene.rs` (not `pub` there, so not reachable from here); `MAX_ENTITIES
/// - 1`, this file's own `MAX_ENTITIES` const above.
///
/// Source: `oracle/codemp/cgame/tr_types.h:15`
const TR_WORLDENT: i32 = MAX_ENTITIES - 1;

/// Raven `QSORT_*` sort-key shifts.
///
/// Source: `oracle/codemp/renderer/tr_local.h:1226-1228`
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

// ===== wave 1 =====

/// Raven `R_CullLocalPointAndRadius`.
///
/// `ori`/`r_nocull_integer`/`frustum` as `R_CullLocalBox`/`R_CullPointAndRadius`
/// above (`R_LocalPointToWorld` needs `ori`; `R_CullPointAndRadius` needs the
/// cvar + frustum pair).
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:109-116`
pub fn R_CullLocalPointAndRadius(
    pt: vec3_t,
    radius: f32,
    ori: &orientationr_t,
    r_nocull_integer: i32,
    frustum: &[cplane_t; 4],
) -> i32 {
    let transformed = R_LocalPointToWorld(pt, ori);
    R_CullPointAndRadius(transformed, radius, r_nocull_integer, frustum)
}

/// Raven `R_RotateForEntity`. Out-param `ori` -> return value.
///
/// `ent` is `trRefEntity_t` (tier-2 `tr_local::tr_ref_entity_t`, read through
/// its existing shape per the interior-safety law's carve-out — the owned
/// `RefEntity` placeholder (`render_state::placeholders`) doesn't carry
/// `axis`/`nonNormalizedAxes` yet, so this wave threads the tier-2 shape
/// directly, the same precedent this file's top-of-file PORT-NOTE set for
/// `orientationr_t`/`viewParms_t`). `view` is `tr.viewParms` (read only:
/// `.world`, `.ori.origin`). `scratch` carries `preTransEntMatrix` (DEC-37
/// A13.3, `TrMainScratch` above — written here, read by
/// `R_WorldNormalToEntity`).
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:302-360`
pub fn R_RotateForEntity(
    ent: &trRefEntity_t,
    view: &viewParms_t,
    scratch: &mut TrMainScratch,
) -> orientationr_t {
    if ent.e.reType != refEntityType_t::RT_MODEL {
        // Whole-struct copy of `viewParms->world` — every `orientationr_t`
        // field, no defaulted tail.
        return orientationr_t {
            origin: view.world.origin,
            axis: view.world.axis,
            viewOrigin: view.world.viewOrigin,
            modelMatrix: view.world.modelMatrix,
        };
    }

    let mut ori = orientationr_t {
        origin: ent.e.origin,
        axis: ent.e.axis,
        viewOrigin: [0.0; 3],
        modelMatrix: [0.0; 16],
    };

    let m = &mut scratch.pre_trans_ent_matrix;
    m[0] = ori.axis[0][0];
    m[4] = ori.axis[1][0];
    m[8] = ori.axis[2][0];
    m[12] = ori.origin[0];

    m[1] = ori.axis[0][1];
    m[5] = ori.axis[1][1];
    m[9] = ori.axis[2][1];
    m[13] = ori.origin[1];

    m[2] = ori.axis[0][2];
    m[6] = ori.axis[1][2];
    m[10] = ori.axis[2][2];
    m[14] = ori.origin[2];

    m[3] = 0.0;
    m[7] = 0.0;
    m[11] = 0.0;
    m[15] = 1.0;

    ori.modelMatrix = myGlMultMatrix(&scratch.pre_trans_ent_matrix, &view.world.modelMatrix);

    // calculate the viewer origin in the model's space
    // needed for fog, specular, and environment mapping
    let mut delta: vec3_t = [0.0; 3];
    VectorSubtract(view.ori.origin, ori.origin, &mut delta);

    // compensate for scale in the axes if necessary
    let axis_length = if ent.e.nonNormalizedAxes != 0 {
        let len = VectorLength(ori.axis[0]);
        if len == 0.0 {
            0.0
        } else {
            1.0 / len
        }
    } else {
        1.0
    };

    ori.viewOrigin[0] = DotProduct(delta, ori.axis[0]) * axis_length;
    ori.viewOrigin[1] = DotProduct(delta, ori.axis[1]) * axis_length;
    ori.viewOrigin[2] = DotProduct(delta, ori.axis[2]) * axis_length;

    ori
}

/// Raven `s_flipMatrix` — file-scope const table (kind 1 of the fn-scope
/// statics rule, DEC-37 A13.3).
///
/// Raven: convert from our coordinate system (looking down X) to OpenGL's
/// coordinate system (looking down -Z). The non-`_XBOX` branch is the one MP
/// retail builds.
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:17-31`
// Raven's own `static float s_flipMatrix[16]` name is kept (file-wide
// Raven-casing convention); it is a static, not a `#define`.
#[allow(non_upper_case_globals)]
const s_flipMatrix: [f32; 16] = [
    0.0, 0.0, -1.0, 0.0, //
    -1.0, 0.0, 0.0, 0.0, //
    0.0, 1.0, 0.0, 0.0, //
    0.0, 0.0, 0.0, 1.0,
];

/// Raven `R_RotateForViewer`. Out-param (via `tr.ori`) -> return value.
///
/// `view` is `tr.viewParms` (`.ori.origin` read; `.world` written — the
/// STATE HOMES SPLIT row's `RenderWorld::frame: FrameState` bucket, still
/// the not-yet-populated `ViewParms`/`OrientationR` placeholders at this
/// wave, so threaded as the tier-2 `viewParms_t` directly per this file's
/// top-of-file PORT-NOTE). The return value is `tr.ori`.
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:369-409`
pub fn R_RotateForViewer(view: &mut viewParms_t) -> orientationr_t {
    // `Com_Memset(&tr.ori, 0, sizeof(tr.ori))` — the struct literal below is
    // the zeroed starting point; the diagonal writes right after it match
    // the oracle's follow-up `axis[i][i] = 1` assignments exactly.
    let mut ori = orientationr_t {
        origin: [0.0; 3],
        axis: [[0.0; 3]; 3],
        viewOrigin: view.ori.origin,
        modelMatrix: [0.0; 16],
    };
    ori.axis[0][0] = 1.0;
    ori.axis[1][1] = 1.0;
    ori.axis[2][2] = 1.0;

    // transform by the camera placement
    let origin = view.ori.origin;

    let mut viewer_matrix = [0.0f32; 16];
    viewer_matrix[0] = view.ori.axis[0][0];
    viewer_matrix[4] = view.ori.axis[0][1];
    viewer_matrix[8] = view.ori.axis[0][2];
    viewer_matrix[12] = -origin[0] * viewer_matrix[0]
        + -origin[1] * viewer_matrix[4]
        + -origin[2] * viewer_matrix[8];

    viewer_matrix[1] = view.ori.axis[1][0];
    viewer_matrix[5] = view.ori.axis[1][1];
    viewer_matrix[9] = view.ori.axis[1][2];
    viewer_matrix[13] = -origin[0] * viewer_matrix[1]
        + -origin[1] * viewer_matrix[5]
        + -origin[2] * viewer_matrix[9];

    viewer_matrix[2] = view.ori.axis[2][0];
    viewer_matrix[6] = view.ori.axis[2][1];
    viewer_matrix[10] = view.ori.axis[2][2];
    viewer_matrix[14] = -origin[0] * viewer_matrix[2]
        + -origin[1] * viewer_matrix[6]
        + -origin[2] * viewer_matrix[10];

    viewer_matrix[3] = 0.0;
    viewer_matrix[7] = 0.0;
    viewer_matrix[11] = 0.0;
    viewer_matrix[15] = 1.0;

    // convert from our coordinate system (looking down X)
    // to OpenGL's coordinate system (looking down -Z)
    ori.modelMatrix = myGlMultMatrix(&viewer_matrix, &s_flipMatrix);

    // Whole-struct copy of `tr.ori` into `tr.viewParms.world` — every
    // `orientationr_t` field, no defaulted tail.
    view.world = orientationr_t {
        origin: ori.origin,
        axis: ori.axis,
        viewOrigin: ori.viewOrigin,
        modelMatrix: ori.modelMatrix,
    };

    ori
}

/// Raven `R_SetupProjection`.
///
/// `view` is `tr.viewParms` (`.zFar` read after `SetFarClip` writes it,
/// `.projectionMatrix` written). `refdef_rdflags`/`refdef_fov_x`/
/// `refdef_fov_y` are `tr.refdef.rdflags`/`fov_x`/`fov_y` — threaded as bare
/// scalars rather than `render_state::placeholders::TrRefdef` to match this
/// wave's own `SetFarClip` precedent (wave 0, already takes
/// `refdef_rdflags: i32` directly; `rdflags` itself isn't a landed
/// `TrRefdef` field yet). `distance_cull` is `tr.distanceCull`
/// (`RenderAssets::distance_cull`, B11) — passed straight through to
/// `SetFarClip`. `r_znear` reads through the live engine cvar table
/// (`RendererCvars::r_znear`, DEC-37 A13.1), the `tr_light.rs`
/// `R_SetupEntityLightingGrid` precedent for `common.cvar(handle)`.
///
/// Only the non-`_XBOX` projection-matrix branch is transcribed — MP never
/// builds `_XBOX`.
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:494-559`
pub fn R_SetupProjection(
    view: &mut viewParms_t,
    refdef_rdflags: i32,
    refdef_fov_x: f32,
    refdef_fov_y: f32,
    distance_cull: f32,
    common: &Common,
    cvars: &RendererCvars,
) {
    // dynamically compute far clip plane distance
    SetFarClip(refdef_rdflags, view, distance_cull);

    //
    // set up projection matrix
    //
    let z_near = common.cvar(cvars.r_znear).value;
    let z_far = view.zFar;

    // C promotes to double (M_PI, tan()); f64 intermediate per wave-0 ruling
    // 12, rounded to f32 once at the assignment (C's own narrowing point).
    let ymax = (z_near as f64 * f64::tan(refdef_fov_y as f64 * PI / 360.0)) as f32;
    let ymin = -ymax;

    let xmax = (z_near as f64 * f64::tan(refdef_fov_x as f64 * PI / 360.0)) as f32;
    let xmin = -xmax;

    let width = xmax - xmin;
    let height = ymax - ymin;
    let depth = z_far - z_near;

    view.projectionMatrix[0] = 2.0 * z_near / width;
    view.projectionMatrix[4] = 0.0;
    view.projectionMatrix[8] = (xmax + xmin) / width; // normally 0
    view.projectionMatrix[12] = 0.0;

    view.projectionMatrix[1] = 0.0;
    view.projectionMatrix[5] = 2.0 * z_near / height;
    view.projectionMatrix[9] = (ymax + ymin) / height; // normally 0
    view.projectionMatrix[13] = 0.0;

    view.projectionMatrix[2] = 0.0;
    view.projectionMatrix[6] = 0.0;
    view.projectionMatrix[10] = -(z_far + z_near) / depth;
    view.projectionMatrix[14] = -2.0 * z_far * z_near / depth;

    view.projectionMatrix[3] = 0.0;
    view.projectionMatrix[7] = 0.0;
    view.projectionMatrix[11] = -1.0;
    view.projectionMatrix[15] = 0.0;
}

/// Raven `CUTOFF` — `qsortFast`'s small-array-switches-to-`shortsort`
/// threshold.
///
/// Raven: testing shows that this is good value.
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:1074`
const CUTOFF: usize = 8;

/// Raven `qsortFast` — iterative quicksort (explicit `lostk`/`histk` stack in
/// place of the C `goto recurse` pseudo-recursion) over `drawSurf_t`,
/// falling back to `shortsort` under `CUTOFF` elements. Operates on the whole
/// slice (Raven's `void *base, unsigned num, unsigned width` triple collapses
/// to a slice per the out-param/pointer-walk dictionary entries); `lo`/`hi`/
/// `loguy`/`higuy` become `isize` slice-index cursors instead of `char*`
/// addresses (the C code transiently holds `higuy` one-past-`hi` and
/// decrements before every dereference, so the loop invariant that keeps
/// every actual index access in bounds is preserved exactly).
///
/// PORT-NOTE: the oracle's leading `if (sizeof(drawSurf_t) != 8) Com_Error(
/// ERR_DROP, "change SWAP_DRAW_SURF macro")` guards the C `SWAP_DRAW_SURF`
/// macro's 2-word (sort + pointer) block-swap trick — a packed-layout
/// precondition for that specific 8-byte pointer-swap optimization, not a
/// bounds/overflow guard on the sort itself. The owned generic `DrawSurf<S>`
/// this port swaps via `[T]::swap` has no fixed size and no equivalent
/// macro, so the check has no Rust counterpart; dropped rather than ported.
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:1097-1252`
pub fn qsortFast<S>(surfs: &mut [DrawSurf<S>]) {
    let num = surfs.len();
    if num < 2 {
        return; // nothing to do
    }

    // Note: the number of stack entries required is no more than
    // 1 + log2(size), so 30 is sufficient for any array
    let mut lostk: [isize; 30] = [0; 30];
    let mut histk: [isize; 30] = [0; 30];
    let mut stkptr: usize = 0; // initialize stack

    let mut lo: isize = 0;
    let mut hi: isize = num as isize - 1; // initialize limits

    // this entry point is for pseudo-recursion calling: setting
    // lo and hi and jumping to here is like recursion, but stkptr is
    // prserved, locals aren't, so we preserve stuff on the stack
    'recurse: loop {
        let size = hi - lo + 1; // number of el's to sort

        // below a certain size, it is faster to use a O(n^2) sorting method
        if size as usize <= CUTOFF {
            shortsort(&mut surfs[lo as usize..=hi as usize]);
        } else {
            // First we pick a partititioning element. The efficiency of the
            // algorithm demands that we find one that is approximately the
            // median of the values, but also that we select one fast. Using
            // the first one produces bad performace if the array is already
            // sorted, so we use the middle one, which would require a very
            // wierdly arranged array for worst case performance. Testing
            // shows that a median-of-three algorithm does not, in general,
            // increase performance.

            let mid = lo + size / 2; // find middle element
            surfs.swap(mid as usize, lo as usize); // swap it to beginning of array

            // We now wish to partition the array into three pieces, one
            // consisiting of elements <= partition element, one of elements
            // equal to the parition element, and one of element >= to it.
            // This is done below; comments indicate conditions established
            // at every step.

            let mut loguy = lo;
            let mut higuy = hi + 1;

            // Note that higuy decreases and loguy increases on every
            // iteration, so loop must terminate.
            loop {
                // lo <= loguy < hi, lo < higuy <= hi + 1,
                // A[i] <= A[lo] for lo <= i <= loguy,
                // A[i] >= A[lo] for higuy <= i <= hi
                loop {
                    loguy += 1;
                    if !(loguy <= hi && surfs[loguy as usize].sort <= surfs[lo as usize].sort) {
                        break;
                    }
                }
                // lo < loguy <= hi+1, A[i] <= A[lo] for lo <= i < loguy,
                // either loguy > hi or A[loguy] > A[lo]

                loop {
                    higuy -= 1;
                    if !(higuy > lo && surfs[higuy as usize].sort >= surfs[lo as usize].sort) {
                        break;
                    }
                }
                // lo-1 <= higuy <= hi, A[i] >= A[lo] for higuy < i <= hi,
                // either higuy <= lo or A[higuy] < A[lo]

                if higuy < loguy {
                    break;
                }
                // if loguy > hi or higuy <= lo, then we would have exited, so
                // A[loguy] > A[lo], A[higuy] < A[lo],
                // loguy < hi, highy > lo

                surfs.swap(loguy as usize, higuy as usize);
                // A[loguy] < A[lo], A[higuy] > A[lo]; so condition at top
                // of loop is re-established
            }

            //     A[i] >= A[lo] for higuy < i <= hi,
            //     A[i] <= A[lo] for lo <= i < loguy,
            //     higuy < loguy, lo <= higuy <= hi
            // implying:
            //     A[i] >= A[lo] for loguy <= i <= hi,
            //     A[i] <= A[lo] for lo <= i <= higuy,
            //     A[i] = A[lo] for higuy < i < loguy

            surfs.swap(lo as usize, higuy as usize); // put partition element in place

            // OK, now we have the following:
            //    A[i] >= A[higuy] for loguy <= i <= hi,
            //    A[i] <= A[higuy] for lo <= i < higuy
            //    A[i] = A[lo] for higuy <= i < loguy

            // We've finished the partition, now we want to sort the
            // subarrays [lo, higuy-1] and [loguy, hi].
            // We do the smaller one first to minimize stack usage.
            // We only sort arrays of length 2 or more.
            if higuy - 1 - lo >= hi - loguy {
                if lo + 1 < higuy {
                    lostk[stkptr] = lo;
                    histk[stkptr] = higuy - 1;
                    stkptr += 1; // save big recursion for later
                }

                if loguy < hi {
                    lo = loguy;
                    continue 'recurse; // do small recursion
                }
            } else {
                if loguy < hi {
                    lostk[stkptr] = loguy;
                    histk[stkptr] = hi;
                    stkptr += 1; // save big recursion for later
                }

                if lo + 1 < higuy {
                    hi = higuy - 1;
                    continue 'recurse; // do small recursion
                }
            }
        }

        // We have sorted the array, except for any pending sorts on the
        // stack. Check if there are any, and do them.
        if stkptr == 0 {
            return; // all subarrays done
        }
        stkptr -= 1;
        lo = lostk[stkptr];
        hi = histk[stkptr];
        // pop subarray from stack, continue 'recurse
    }
}

/// Raven `R_DebugPolygon`.
///
/// DEFERRED: R4 — the entire body is fixed-function GL calls
/// (`qglColor3f`/`qglBegin`/`qglVertex3fv`/`qglEnd`/`qglDepthRange`) plus
/// `GL_State` (already-ported wave-0, but purely a GL binding-state write —
/// no CPU-only remainder to extract here). DEC-01/DEC-37 rule the R4 backend
/// an idiomatic wgpu rewrite, not a GL transcription, and R2 leaves these
/// entry points unhomed (`GpuResources::gl_state` a named placeholder until
/// R4). No CPU logic survives the deferral: `color`/`points` only exist to
/// feed the deferred draw calls (the bit-unpack `color&1`/`(color>>1)&1`/
/// `(color>>2)&1` is itself only a GL color-component argument).
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:1540-1564`
pub fn R_DebugPolygon(_color: i32, _num_points: i32, _points: &[f32]) {
    // DEFERRED: R4 — R_DebugPolygon (see doc comment above) (DEC-37 A13.2 / DEC-01)
    // Source: oracle/codemp/renderer/tr_main.cpp:1540-1564
}

// ===== wave 2 =====

/// Raven `R_GetPortalOrientations`. Out-params `surface`/`camera`/
/// `pvsOrigin`/`mirror` -> return value: `Some((surface, camera, pvsOrigin,
/// mirror))` on `qtrue`, `None` on `qfalse`. On the `qfalse` path the oracle
/// leaves `surface`/`camera` only partially written (axis set, origin not) —
/// dead data no caller reads once the `qboolean` return says "don't render
/// anything" (see the oracle's own trailing comment), so the `None` arm
/// carries nothing, per the out-param -> return-value dictionary entry.
///
/// `draw_surf_surface` is `drawSurf->surface` (the only `drawSurf_t` field
/// this fn reads). `entities` is `tr.refdef.entities` (length =
/// `tr.refdef.num_entities`, STATE HOMES SPLIT — threaded as a slice per this
/// file's `R_SpriteFogNum`/`fogs` precedent). `refdef_time` is
/// `tr.refdef.time` (only read in the continuous/bobbing camera-rotation
/// branch). `view` is `tr.viewParms` (threaded through to
/// `R_RotateForEntity`, read-only here). `scratch` carries
/// `preTransEntMatrix` (DEC-37 A13.3, `TrMainScratch`), threaded through to
/// `R_RotateForEntity`.
///
/// PORT-NOTE: the oracle also writes `tr.currentEntityNum`/`tr.currentEntity`
/// (STATE HOMES SPLIT row: `RenderWorld::frame: FrameState`) right before
/// calling `R_RotateForEntity`. This wave is scoped to `tr_main.rs` only
/// (cannot add a field to `render_state/frame_state.rs`) — same precedent as
/// `tr_scene.rs`'s `R_AddPolygonSurfaces` — so the write stays a local
/// computation (`current_entity` below); escalate a `FrameState` field-merge
/// if a later wave needs to read either value back outside this call.
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:687-804`
pub fn R_GetPortalOrientations(
    draw_surf_surface: Option<&SurfaceGeometry>,
    entity_num: i32,
    entities: &[trRefEntity_t],
    refdef_time: i32,
    view: &viewParms_t,
    scratch: &mut TrMainScratch,
) -> Option<(orientation_t, orientation_t, vec3_t, bool)> {
    // create plane axis for the portal we are seeing
    let original_plane = R_PlaneForSurface(draw_surf_surface);
    let mut plane_normal = original_plane.normal;
    let mut plane_dist = original_plane.dist;
    let mut original_plane_dist = original_plane.dist;

    // rotate the plane if necessary
    if entity_num != TR_WORLDENT {
        let current_entity = &entities[entity_num as usize];

        // get the orientation of the entity
        let ori = R_RotateForEntity(current_entity, view, scratch);

        // rotate the plane, but keep the non-rotated version for matching
        // against the portalSurface entities
        plane_normal = R_LocalNormalToWorld(original_plane.normal, &ori);
        plane_dist = original_plane.dist + DotProduct(plane_normal, ori.origin);

        // translate the original plane
        original_plane_dist = original_plane.dist + DotProduct(original_plane.normal, ori.origin);
    }

    let mut surface_axis: [vec3_t; 3] = [[0.0; 3]; 3];
    VectorCopy(plane_normal, &mut surface_axis[0]);
    // Raven reads `surfaceAxis[0]` while writing `surfaceAxis[1]`; the read
    // value is snapshotted first so the read-before-write order is preserved
    // under Rust's whole-array borrow of the index expression.
    let surface_axis_0 = surface_axis[0];
    PerpendicularVector(&mut surface_axis[1], surface_axis_0);
    CrossProduct(surface_axis[0], surface_axis[1], &mut surface_axis[2]);

    // locate the portal entity closest to this plane.
    // origin will be the origin of the portal, origin2 will be
    // the origin of the camera
    for e in entities {
        if e.e.reType != refEntityType_t::RT_PORTALSURFACE {
            continue;
        }

        let d = DotProduct(e.e.origin, original_plane.normal) - original_plane_dist;
        if d > 64.0 || d < -64.0 {
            continue;
        }

        // get the pvsOrigin from the entity
        let pvs_origin = e.e.oldorigin;

        // if the entity is just a mirror, don't use as a camera point
        if e.e.oldorigin[0] == e.e.origin[0]
            && e.e.oldorigin[1] == e.e.origin[1]
            && e.e.oldorigin[2] == e.e.origin[2]
        {
            let mut surface_origin: vec3_t = [0.0; 3];
            VectorScale(plane_normal, plane_dist, &mut surface_origin);

            let mut camera_origin: vec3_t = [0.0; 3];
            VectorCopy(surface_origin, &mut camera_origin);

            let mut camera_axis0: vec3_t = [0.0; 3];
            VectorSubtract(vec3_origin, surface_axis[0], &mut camera_axis0);

            let surface = orientation_t {
                origin: surface_origin,
                axis: surface_axis,
            };
            let camera = orientation_t {
                origin: camera_origin,
                axis: [camera_axis0, surface_axis[1], surface_axis[2]],
            };

            return Some((surface, camera, pvs_origin, true));
        }

        // project the origin onto the surface plane to get
        // an origin point we can rotate around
        let d = DotProduct(e.e.origin, plane_normal) - plane_dist;
        let mut surface_origin: vec3_t = [0.0; 3];
        VectorMA(e.e.origin, -d, surface_axis[0], &mut surface_origin);

        // now get the camera origin and orientation
        let mut camera_origin: vec3_t = [0.0; 3];
        VectorCopy(e.e.oldorigin, &mut camera_origin);
        // PORT-NOTE: `AxisCopy(e->e.axis, camera->axis)` is a straight
        // 3-element `vec3_t` array copy (`native_math::qmath::AxisCopy`'s own
        // body: `out[0]=in[0]; out[1]=in[1]; out[2]=in[2];`); transcribed as
        // a plain array assignment rather than round-tripping through that
        // raw-pointer signature (interior-safety law: no new pointer casts
        // in this file for a same-effect array copy).
        let mut camera_axis = e.e.axis;

        VectorSubtract(vec3_origin, camera_axis[0], &mut camera_axis[0]);
        VectorSubtract(vec3_origin, camera_axis[1], &mut camera_axis[1]);

        // optionally rotate
        if e.e.oldframe != 0 {
            if e.e.frame != 0 {
                // continuous rotate
                let rot_d = (refdef_time as f32 / 1000.0) * e.e.frame as f32;
                let mut transformed: vec3_t = [0.0; 3];
                VectorCopy(camera_axis[1], &mut transformed);
                // Read `cameraAxis[0]` before the write to `cameraAxis[1]`, as
                // Raven's argument evaluation does; Rust borrows the whole array.
                let camera_axis_0 = camera_axis[0];
                RotatePointAroundVector(&mut camera_axis[1], camera_axis_0, transformed, rot_d);
                CrossProduct(camera_axis[0], camera_axis[1], &mut camera_axis[2]);
            } else {
                // bobbing rotate, with skinNum being the rotation offset
                // C `sin` is a double fn; f64 intermediate per wave-0 ruling
                // 12 (the `* 0.003f` multiply itself stays float — `0.003f`
                // is a float literal, not a double one).
                let bob = refdef_time as f32 * 0.003;
                let mut rot_d = f64::sin(bob as f64) as f32;
                rot_d = e.e.skinNum as f32 + rot_d * 4.0;
                let mut transformed: vec3_t = [0.0; 3];
                VectorCopy(camera_axis[1], &mut transformed);
                let camera_axis_0 = camera_axis[0];
                RotatePointAroundVector(&mut camera_axis[1], camera_axis_0, transformed, rot_d);
                CrossProduct(camera_axis[0], camera_axis[1], &mut camera_axis[2]);
            }
        } else if e.e.skinNum != 0 {
            let rot_d = e.e.skinNum as f32;
            let mut transformed: vec3_t = [0.0; 3];
            VectorCopy(camera_axis[1], &mut transformed);
            let camera_axis_0 = camera_axis[0];
            RotatePointAroundVector(&mut camera_axis[1], camera_axis_0, transformed, rot_d);
            CrossProduct(camera_axis[0], camera_axis[1], &mut camera_axis[2]);
        }

        let surface = orientation_t {
            origin: surface_origin,
            axis: surface_axis,
        };
        let camera = orientation_t {
            origin: camera_origin,
            axis: camera_axis,
        };

        return Some((surface, camera, pvs_origin, false));
    }

    // if we didn't locate a portal entity, don't render anything.
    // We don't want to just treat it as a mirror, because without a
    // portal entity the server won't have communicated a proper entity set
    // in the snapshot

    // unfortunately, with local movement prediction it is easily possible
    // to see a surface before the server has communicated the matching
    // portal surface entity, so we don't want to print anything here...

    None
}

/// Raven `IsMirror`.
///
/// `draw_surf_surface`/`entity_num`/`entities`/`view`/`scratch` as
/// `R_GetPortalOrientations` above (this fn is the oracle's near-verbatim
/// duplicate of that one's plane-setup prefix, minus the camera/pvsOrigin
/// outputs — it only needs to answer "is this portal surface a plain
/// mirror?").
///
/// PORT-NOTE: the oracle's rotated-plane branch here also computes
/// `plane.normal`/`plane.dist` (`R_LocalNormalToWorld` + a `DotProduct`
/// offset, exactly as in `R_GetPortalOrientations`), but `IsMirror` never
/// reads either afterward — a dead store, kept in the oracle only because
/// this fn is a copy-paste of `R_GetPortalOrientations`' prefix. Dropped
/// here (porting-rules §10: preserve behavior, not shape; `R_LocalNormalToWorld`
/// is pure, so dropping its unread result changes no observable behavior).
/// The `R_RotateForEntity` call itself is kept — its `ori.origin` return
/// value feeds `original_plane_dist`'s translation below, and its side
/// effects are the same `tr.currentEntityNum`/`tr.currentEntity`/`tr.ori`
/// writes documented on `R_GetPortalOrientations` (same wave-scope
/// escalation: computed locally here, not persisted to `FrameState`).
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:806-864`
pub fn IsMirror(
    draw_surf_surface: Option<&SurfaceGeometry>,
    entity_num: i32,
    entities: &[trRefEntity_t],
    view: &viewParms_t,
    scratch: &mut TrMainScratch,
) -> bool {
    // create plane axis for the portal we are seeing
    let original_plane = R_PlaneForSurface(draw_surf_surface);
    let mut original_plane_dist = original_plane.dist;

    // rotate the plane if necessary
    if entity_num != TR_WORLDENT {
        let current_entity = &entities[entity_num as usize];

        // get the orientation of the entity
        let ori = R_RotateForEntity(current_entity, view, scratch);

        // translate the original plane
        original_plane_dist = original_plane.dist + DotProduct(original_plane.normal, ori.origin);
    }

    // locate the portal entity closest to this plane.
    // origin will be the origin of the portal, origin2 will be
    // the origin of the camera
    for e in entities {
        if e.e.reType != refEntityType_t::RT_PORTALSURFACE {
            continue;
        }

        let d = DotProduct(e.e.origin, original_plane.normal) - original_plane_dist;
        if d > 64.0 || d < -64.0 {
            continue;
        }

        // if the entity is just a mirror, don't use as a camera point
        if e.e.oldorigin[0] == e.e.origin[0]
            && e.e.oldorigin[1] == e.e.origin[1]
            && e.e.oldorigin[2] == e.e.origin[2]
        {
            return true;
        }

        return false;
    }
    false
}
