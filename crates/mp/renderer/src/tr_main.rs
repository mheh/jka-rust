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

use crate::render_state::frame_data::FrameData;
use crate::render_state::frame_state::FrameState;
use crate::render_state::gpu_resources::GpuResources;
use crate::render_state::image_asset::ImageHandle;
use crate::render_state::placeholders::RefEntity;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::render_state::shader_asset::ShaderHandle;
use crate::tr_backend::{GL_Bind, GL_Cull};
use crate::tr_bsp::{Surface, SurfaceData};
use crate::tr_cmds::R_SyncRenderThread;
use crate::tr_ghoul2::r_add_ghoul_surfaces;
use crate::tr_local::cull_type_t::cullType_t;
use crate::tr_local::dlight_s::dlight_t;
use crate::tr_local::fog_t::fog_t;
use crate::tr_local::modtype_t::modtype_t;
use crate::tr_local::orientationr_t::orientationr_t;
use crate::tr_local::shader_sort_t::shaderSort_t;
use crate::tr_local::srf_terrain_s::srfTerrain_t;
use crate::tr_local::tr_ref_entity_t::trRefEntity_t;
use crate::tr_local::tr_refdef_t::trRefdef_t;
use crate::tr_local::view_parms_t::viewParms_t;
use crate::tr_mesh::r_add_md3_surfaces;
use crate::tr_model::render_models::RenderModels;
use crate::tr_public::ref_flags::RDF_NOWORLDMODEL;
use crate::tr_scene::R_AddPolygonSurfaces;
use crate::tr_shader::R_GetShaderByHandle;
use crate::tr_terrain::R_AddTerrainSurfaces;
use crate::tr_world::R_AddBrushModelSurfaces;

use core::f64::consts::PI;

use mp_engine_qcommon::cm_terrain::CmLandScape;
use mp_engine_qcommon::common::{com_error, Common, EngineHostView};
use mp_engine_qcommon::common_fns::Com_DPrintf;
use mp_engine_qcommon::qfiles::draw_vert_t::drawVert_t;
use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::q_color::S_COLOR_RED;
use mp_qshared::shared::q_math::{
    _DotProduct as DotProduct, _VectorAdd as VectorAdd, _VectorCopy as VectorCopy,
    _VectorMA as VectorMA, _VectorScale as VectorScale, _VectorSubtract as VectorSubtract,
    vec3_origin, CrossProduct, DistanceSquared, PerpendicularVector, SetPlaneSignbits, VectorClear,
    VectorLength,
};
use mp_qshared::shared::{cplane_t, orientation_t, qfalse, qtrue, vec3_t, vec4_t};
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

/// Raven `RF_FIRST_PERSON` — only draw through eyes (view weapon, damage
/// blood blob). Local copy of the private const already ported at
/// `tr_light.rs` (not `pub` there, so not reachable from here); same value,
/// same oracle line.
///
/// Source: `oracle/codemp/cgame/tr_types.h:20`
const RF_FIRST_PERSON: i32 = 0x00004;

/// Raven `MAX_SHADERS` (non-`_XBOX` branch).
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
// `Clone`/`Copy` added wave 12: `R_SortDrawSurfs`'s recursion through
// `R_MirrorViewBySurface` -> `R_RenderView` -> `R_GenerateDrawSurfs` pushes
// new entries onto the very `Vec<DrawSurf<SurfaceGeometry>>` a live borrowed
// slice/reference into it would alias across that call (Rust has no
// equivalent to Raven's fixed `drawSurfs[MAX_DRAWSURFS]` array, which never
// reallocates under the C ring-buffer scheme). Every field here is already
// `Copy` (`cplane_t`, or a borrowed slice); deriving it lets the sort loop
// copy one element out by value before recursing instead of holding a
// borrow across it.
#[derive(Clone, Copy)]
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

/// The `drawSurf_t::surface` payload for a **world** surface (DEC-43.3): a
/// `Copy` index handle into `WorldAsset::surfaces`, carrying a cached copy
/// of that surface's kind tag. The `u32` is the flat surface index — the
/// oracle's own `worldData.surfaces` subscript, which under the owned world
/// replaces its `msurface_t.data` pointer (porting-rules §B5) — so the
/// backend re-fetches the surface from the world instead of the draw list
/// holding a borrow of it across the world walk (the borrow
/// `R_RecursiveWorldNode` cannot hand out while it is mutating the same
/// array).
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:656-678`
/// (`surfaceType_t`)
#[derive(Clone, Copy)]
pub enum WorldSurfaceRef {
    /// `&skipData` (`SF_SKIP`).
    Skip(u32),
    /// `srfSurfaceFace_t` (`SF_FACE`).
    Face(u32),
    /// `srfGridMesh_t` (`SF_GRID`).
    Grid(u32),
    /// `srfTriangles_t` (`SF_TRIANGLES`).
    Triangles(u32),
    /// `srfFlare_t` (`SF_FLARE`).
    Flare(u32),
}

impl WorldSurfaceRef {
    /// The handle for `WorldAsset::surfaces[index]`, tagged with that
    /// surface's current kind — the owned analogue of Raven passing
    /// `surf->data` (the tagged-union pointer) straight to `R_AddDrawSurf`.
    pub fn of(surf: &Surface, index: u32) -> Self {
        match &surf.data {
            SurfaceData::Skip => WorldSurfaceRef::Skip(index),
            SurfaceData::Face(_) => WorldSurfaceRef::Face(index),
            SurfaceData::Grid(_) => WorldSurfaceRef::Grid(index),
            SurfaceData::Triangles(_) => WorldSurfaceRef::Triangles(index),
            SurfaceData::Flare(_) => WorldSurfaceRef::Flare(index),
        }
    }
}

/// The owned replacement for Raven `drawSurf_t` (`tr_local::draw_surf_s`,
/// `sort: u32, surface: *mut surfaceType_t`) — `surface` becomes an owned
/// value or handle instead of a raw tagged-union pointer (interior-safety
/// law). Still generic over the concrete surface representation: world
/// surfaces instantiate it at [`WorldSurfaceRef`] (DEC-43.3, the tier-2
/// transition audit's `drawSurf_t` row: "`surface` -> a `Handle`/index into
/// the surface arena"), while the non-world payloads `SurfaceGeometry`
/// stands in for (`Poly`, entity md3/ghoul2, `tr.landScape`) still need a
/// carrier of their own before one unified payload enum can replace `S`.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:680-683`
// `Clone`/`Copy` added wave 12 — see `SurfaceGeometry`'s own derive note
// just above; `DrawSurf<S>`'s derive requires `S: Copy`, which
// `SurfaceGeometry<'a>` now satisfies.
#[derive(Clone, Copy)]
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

// ===== wave 3 =====

/// Raven `SurfIsOffscreen` — tessellates `drawSurf` and answers whether its
/// screen-space footprint can be culled before a portal/mirror scene is
/// rendered through it (trivial-reject on clip-plane flags, then a
/// backface/nearest-vertex distance test against the surface's
/// `portalRange`, deferring to `IsMirror` for the mirror early-out).
///
/// DEFERRED: R4 — every step past `R_DecomposeSort`/`RB_BeginSurface` reads
/// or writes `tess` (`tess.numVertexes`, `tess.xyz`, `tess.indexes`,
/// `tess.normal`, `tess.shader->portalRange`), which R2's `## State
/// ownership` row dissolves into R4's tessellation/vertex-building pipeline
/// ("no single global scratch buffer survives the new topology" — R4
/// concern, not an R3 field; same reasoning `tr_shade.rs::RB_BeginSurface`,
/// `tr_shadows.rs::R_RenderShadowEdges`/`RB_ProjectionShadowDeform` already
/// carry). The surface-kind dispatch this fn drives to populate `tess`
/// (`rb_surfaceTable[*drawSurf->surface](drawSurf->surface)`, this file's
/// packet STATE HOMES row) is itself the R4 tessellation step, not a
/// separable CPU computation — there is no partial body to transcribe.
/// `clipDest` is an unused out-param in the oracle itself (never referenced
/// in the 871-961 body) — dropped from the port entirely, not merely
/// deferred. `R_RotateForViewer`/`R_DecomposeSort`/`RB_BeginSurface`/
/// `IsMirror` (in-module, lower-wave) and `VectorLengthSquared` (inline
/// header helper) are threaded through the signature per the R_CullModel
/// precedent (`tr_mesh.rs`) so a later R4 wave's fix is a body-only fill, not
/// a signature change.
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:871-961`
pub fn SurfIsOffscreen<S>(
    _draw_surf: &DrawSurf<S>,
    _sorted_shaders: &[ShaderHandle],
    _entities: &[trRefEntity_t],
    _view: &mut viewParms_t,
    _scratch: &mut TrMainScratch,
    _frame: &mut FrameState,
) -> bool {
    todo!("Port SurfIsOffscreen — oracle/codemp/renderer/tr_main.cpp:871-961 (R4: tess tessellation pipeline, R2 `## State ownership` row `tess`)")
}

// ===== wave 9 =====

/// Raven `R_DebugGraphics`.
///
/// `r_debug_surface_integer` is `r_debugSurface->integer`
/// (`RendererCvars::r_debugSurface`, DEC-37 A13.1, `common.cvar(handle)`
/// read through the live engine cvar table — the `R_SetupProjection`/
/// `tr_world.rs::R_AddWorldSurfaces` precedent for cvar threading).
/// `white_image` is `tr.whiteImage` (`RenderAssets::white_image`, STATE
/// HOMES SPLIT row — a registry field, `R2-D3`/`R2-D4`). `assets`/`common`/
/// `cvars` thread straight through to `R_SyncRenderThread`'s own landed
/// signature (`tr_cmds.rs`); `frame`/`gpu` thread `GL_Cull`/`GL_Bind`'s own
/// parameters straight through (both already-landed DEFERRED-R4 stubs,
/// `tr_backend.rs`).
///
/// DEFERRED: `CM_DrawDebugSurface( R_DebugPolygon )` — the collision-debug
/// surface walk `cm_patch_fns.rs` explicitly dropped as dead surface (§20):
/// "Renderer-debug surface dropped ... `CM_DrawDebugSurface` itself is not
/// ported (it has no callers here)" (that file's module doc comment). Its
/// sole payload, `R_DebugPolygon`, is itself an already-landed DEFERRED-R4
/// stub in this file (every argument it would receive only feeds fixed-
/// function GL calls). No CPU logic is lost: the walk exists purely to feed
/// GL debug-draw calls on both ends.
/// (cm_patch_fns.rs module doc comment §20; DEC-37 A13.2 / DEC-01)
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:1573-1584`
pub fn R_DebugGraphics(
    r_debug_surface_integer: i32,
    white_image: Option<ImageHandle>,
    assets: &RenderAssets,
    common: &Common,
    cvars: &RendererCvars,
    frame: &FrameState,
    gpu: &mut GpuResources,
) {
    if r_debug_surface_integer == 0 {
        return;
    }

    // the render thread can't make callbacks to the main thread
    R_SyncRenderThread(assets, common, cvars);

    GL_Bind(gpu, white_image);
    GL_Cull(frame, gpu, cullType_t::CT_FRONT_SIDED);

    // DEFERRED: CM_DrawDebugSurface( R_DebugPolygon ) — see doc comment above.
    // Source: oracle/codemp/renderer/tr_main.cpp:1583
}

// ===== wave 10 =====

/// Builds the `RefEntity` (R2 placeholder, `render_state::placeholders`) view
/// of a `trRefEntity_t` that `R_AddBrushModelSurfaces`/`r_add_ghoul_surfaces`
/// (their own already-ported signatures, waves 5/9) take. Extends the
/// whole-struct `refEntity_t` -> `RefEntity` mapping `RE_AddRefEntityToScene`
/// (`tr_scene.rs`) established with `trRefEntity_t`'s own five lighting
/// -output fields, read as their *live* values here (that call site zeroes
/// them instead, because there the entity is freshly submitted and
/// `lighting_calculated: false` forces a recompute before any read).
///
/// `ambient_light_int` unpacks `ambientLightInt`'s packed 32-bit value the
/// same way Raven's own byte writes do (`((byte *)&ent->ambientLightInt)[N]`,
/// cited on [`RefEntity::ambient_light_int`]'s own doc comment) — a
/// little-endian byte reinterpretation, matching x86.
fn ref_entity_from_tr(ent: &trRefEntity_t) -> RefEntity {
    RefEntity {
        re_type: ent.e.reType,
        renderfx: ent.e.renderfx,
        h_model: ent.e.hModel,
        axis: ent.e.axis,
        origin: ent.e.origin,
        old_origin: ent.e.oldorigin,
        custom_shader: ent.e.customShader,
        shader_rgba: ent.e.shaderRGBA,
        radius: ent.e.radius,
        rotation: ent.e.rotation,
        frame: ent.e.frame,
        lighting_origin: ent.e.lightingOrigin,
        end_time: ent.e.endTime,
        saber_length: ent.e.saberLength,
        angles: ent.e.angles,
        model_scale: ent.e.modelScale,
        has_ghoul2: !ent.e.ghoul2.is_null(),
        need_dlights: ent.needDlights != 0,
        lighting_calculated: ent.lightingCalculated != 0,
        light_dir: ent.lightDir,
        ambient_light: ent.ambientLight,
        ambient_light_int: ent.ambientLightInt.to_le_bytes(),
        directed_light: ent.directedLight,
        dlight_bits: ent.dlightBits,
    }
}

/// Writes back the seven of `trRefEntity_t`'s eight non-`e` fields a
/// `RefEntity` carries (all but `axisLength`, which no `R_AddEntitySurfaces`
/// callee touches) from a `RefEntity` that `R_AddBrushModelSurfaces` may have
/// mutated through its own `R_SetupEntityLighting` call
/// (`lighting_calculated`/`ambient_light`/`ambient_light_int`/
/// `directed_light`/`light_dir` writes, `tr_light.rs`) and its own
/// `R_DlightBmodel` call (`need_dlights`/`dlight_bits`, `tr_light.rs`).
/// Raven mutates `*ent` in place, so a later per-frame stage reading
/// `tr.refdef.entities[n]`'s lighting fields must observe them; the reverse
/// of [`ref_entity_from_tr`] above.
fn write_back_lighting(ent: &mut trRefEntity_t, re: &RefEntity) {
    ent.needDlights = re.need_dlights as i32;
    ent.lightingCalculated = re.lighting_calculated as i32;
    ent.lightDir = re.light_dir;
    ent.ambientLight = re.ambient_light;
    ent.ambientLightInt = i32::from_le_bytes(re.ambient_light_int);
    ent.directedLight = re.directed_light;
    ent.dlightBits = re.dlight_bits;
}

/// Raven `R_AddEntitySurfaces`.
///
/// `entities` is `tr.refdef.entities` (length = `tr.refdef.num_entities`,
/// this file's established `R_SpriteFogNum`/`R_GetPortalOrientations` slice
/// -threading precedent); `view` is `tr.viewParms` (`.isPortal`/`.frustum`
/// read, threaded through to `R_RotateForEntity`); `scratch` carries
/// `preTransEntMatrix` (`TrMainScratch`, threaded through to
/// `R_RotateForEntity`); `models` is the live `RenderModels` registry
/// (`R_GetModelByHandle` -> `models.get_model`/`models.num_models`);
/// `engine_view` is the host bundle `R_AddBrushModelSurfaces`'s own already
/// -ported signature demands (`view` there — renamed here to avoid colliding
/// with this fn's own `view: &viewParms_t`); `fogs` is `tr.world->fogs`
/// (`R_SpriteFogNum`'s own established parameter); `refdef_rdflags` is
/// `tr.refdef.rdflags` (`R_SpriteFogNum`'s `rdflags` +
/// `R_AddBrushModelSurfaces`'s `refdef_rdflags`); `dlights`/`draw_surfs` are
/// `tr.refdef.dlights`/`.drawSurfs` (STATE HOMES SPLIT, `FrameData`'s append
/// -validation carriers — threaded as a slice/`Vec` per this file's
/// established `R_AddDrawSurf` precedent, `tr_scene.rs`'s
/// `R_AddPolygonSurfaces` twin).
///
/// `r_drawentities`/`r_nocull`/`r_shadows` (`RendererCvars`, DEC-37 A13.1)
/// are read through the live cvar table (`engine_view.common.cvar`) at the
/// point they are needed rather than threaded as pre-resolved integers —
/// this fn already carries `engine_view`/`cvars` for the shader/model
/// -lookup calls below, so there is no leaf-function reason to split the
/// cvar reads out as separate parameters the way `R_CullLocalBox`'s
/// `r_nocull_integer` does.
///
/// `entitySurface` (file-scope `static surfaceType_t entitySurface =
/// SF_ENTITY;`, a kind-1 const per the fn-scope-statics three-kind rule,
/// DEC-37 A13.3) has no dedicated `SurfaceGeometry` payload — it carries no
/// data of its own (`SF_ENTITY`'s whole purpose is "this draw surf's shape
/// comes from the entity itself", `tr_surface.rs`'s `RB_SurfaceEntity`
/// dispatch family) — mapped to `SurfaceGeometry::Other`, this file's
/// established catch-all for not-yet-modeled surface kinds
/// (`R_AddTerrainSurfaces`, `tr_terrain.rs`, and `R_AddPolygonSurfaces`,
/// `tr_scene.rs`, both already use it the same way).
///
/// PORT-NOTE: `tr.currentEntityNum`/`tr.shiftedEntityNum`/`tr.currentModel`
/// (STATE HOMES SPLIT row's `RenderWorld::frame: FrameState` bucket) stay
/// local loop computations, not persisted to `FrameState` — same wave-scope
/// precedent as `R_GetPortalOrientations`/`IsMirror` (this file) and
/// `R_AddPolygonSurfaces` (`tr_scene.rs`). `tr.currentEntity` is the one
/// exception: `FrameState::current_entity` is a landed field and
/// `R_DlightBmodel` (`tr_light.cpp:78-79`, reached through
/// `R_AddBrushModelSurfaces`) writes `needDlights`/`dlightBits` through it,
/// so the oracle's `ent = tr.currentEntity = &tr.refdef.entities[n]`
/// (`:1380`) is transcribed as a per-iteration `frame.current_entity` write.
///
/// PORT-NOTE: `R_GetModelByHandle` never returns a null-equivalent — its
/// oracle body returns `tr.models[0]`, the reserved `MOD_BAD` NULL model,
/// for any out-of-range handle (`oracle/codemp/renderer/tr_model.cpp:
/// 593-604`), which the already-ported `RenderModels::get_model` reproduces.
/// The oracle's `if (!tr.currentModel)` arm (`:1425-1426`) is therefore
/// unreachable in either tree and is not transcribed: an out-of-range or
/// zero `hModel` falls into the switch's own `MOD_BAD` arm below, exactly as
/// it does in the oracle.
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:1369-1482`
pub fn R_AddEntitySurfaces<'a>(
    entities: &mut [trRefEntity_t],
    view: &viewParms_t,
    scratch: &mut TrMainScratch,
    engine_view: &mut EngineHostView<'_>,
    assets: &RenderAssets,
    models: &RenderModels,
    cvars: &RendererCvars,
    frame: &mut FrameState,
    refdef_rdflags: i32,
    fogs: &[fog_t],
    dlights: &mut [dlight_t],
    draw_surfs: &mut Vec<DrawSurf<SurfaceGeometry<'a>>>,
) {
    if engine_view.common.cvar(cvars.r_drawentities).integer == 0 {
        return;
    }

    let rdf_nofog = refdef_rdflags & RDF_NOFOG != 0;

    for current_entity_num in 0..entities.len() {
        // preshift the value we are going to OR into the drawsurf sort
        let shifted_entity_num = (current_entity_num as i32) << QSORT_ENTITYNUM_SHIFT;
        let ent = &mut entities[current_entity_num];

        debug_assert!(ent.e.renderfx >= 0);

        ent.needDlights = qfalse;

        // `ent = tr.currentEntity = &tr.refdef.entities[tr.currentEntityNum]`
        // — one object in the oracle, two carriers here (`entities[n]` and
        // `FrameState::current_entity`, which holds a `RefEntity` by value
        // per R2 ruling 1). Snapshotted after the `needDlights` clear above
        // so both carriers start the iteration equal.
        frame.current_entity = Some(ref_entity_from_tr(ent));

        // the weapon model must be handled special --
        // we don't want the hacked weapon position showing in
        // mirrors, because the true body position will already be drawn
        if (ent.e.renderfx & RF_FIRST_PERSON) != 0 && view.isPortal != 0 {
            continue;
        }

        // simple generated models, like sprites and beams, are not culled
        match ent.e.reType {
            refEntityType_t::RT_PORTALSURFACE => {
                // don't draw anything
            }

            refEntityType_t::RT_SPRITE
            | refEntityType_t::RT_BEAM
            | refEntityType_t::RT_ORIENTED_QUAD
            | refEntityType_t::RT_ELECTRICITY
            | refEntityType_t::RT_LINE
            | refEntityType_t::RT_ORIENTEDLINE
            | refEntityType_t::RT_CYLINDER
            | refEntityType_t::RT_SABER_GLOW => {
                // self blood sprites, talk balloons, etc should not be drawn
                // in the primary view. We can't just do this check for all
                // entities, because md3 entities may still want to cast
                // shadows from them
                //
                // DEFERRED: RF_THIRD_PERSON — never-guess-a-constant: not in
                // this packet's FILE-SCOPE CONSTANTS section and not ported
                // anywhere else in the crate (the identical absence
                // `tr_mesh.rs::r_add_md3_surfaces`'s own DEFERRED note
                // already records for this same flag). The oracle's very
                // first statement in this arm reads it, so nothing past it
                // is reachable without guessing the bitmask.
                // Source: oracle/codemp/renderer/tr_main.cpp:1410-1417
                todo!(
                    "Port R_AddEntitySurfaces RT_SPRITE-family arm — RF_THIRD_PERSON unported, oracle/codemp/renderer/tr_main.cpp:1399-1418"
                )
            }

            refEntityType_t::RT_MODEL => {
                // we must set up parts of tr.ori for model culling
                let ori = R_RotateForEntity(ent, view, scratch);

                // DEFERRED: `tr.ori` — the oracle writes `R_RotateForEntity`'s
                // orientation into the `tr.ori` global (Raven's own comment
                // above: "we must set up parts of tr.ori for model culling"),
                // which `R_AddMD3Surfaces` and `R_AddGhoulSurfaces` then read
                // back for culling. Here it is only handed to
                // `R_AddBrushModelSurfaces` (whose already-ported signature
                // takes it as a parameter); `FrameState::ori` is the
                // field-less `OrientationR` placeholder
                // (`render_state/placeholders.rs`), so there is no carrier to
                // publish it through for the other two callees.
                // Source: oracle/codemp/renderer/tr_main.cpp:1421-1442
                let current_model = models.get_model(ent.e.hModel);
                match current_model.r#type {
                    modtype_t::MOD_MESH => {
                        let r_shadows_integer = engine_view.common.cvar(cvars.r_shadows).integer;
                        r_add_md3_surfaces(
                            engine_view.common,
                            cvars,
                            r_shadows_integer,
                            assets,
                            &*frame,
                            ent,
                        );
                    }

                    modtype_t::MOD_BRUSH => {
                        let r_nocull_integer = engine_view.common.cvar(cvars.r_nocull).integer;
                        // `R_AddBrushModelSurfaces`'s two mutators both
                        // target `*tr.currentEntity` in the oracle — one
                        // object — but land in different carriers here:
                        // `R_SetupEntityLighting` writes the `&mut
                        // RefEntity` this call passes, `R_DlightBmodel`
                        // writes `frame.current_entity`
                        // (`oracle/codemp/renderer/tr_light.cpp:78-79`).
                        // The lighting writes are folded back onto
                        // `frame.current_entity` so the single reconciled
                        // entity is the one written back to
                        // `entities[n]`.
                        let mut re = ref_entity_from_tr(ent);
                        R_AddBrushModelSurfaces(
                            &mut re,
                            models,
                            r_nocull_integer,
                            &ori,
                            &view.frustum,
                            engine_view,
                            cvars,
                            assets,
                            frame,
                            refdef_rdflags,
                            dlights,
                        );
                        let current_entity = frame
                            .current_entity
                            .as_mut()
                            .expect("R_AddEntitySurfaces: current_entity set above");
                        current_entity.lighting_calculated = re.lighting_calculated;
                        current_entity.light_dir = re.light_dir;
                        current_entity.ambient_light = re.ambient_light;
                        current_entity.ambient_light_int = re.ambient_light_int;
                        current_entity.directed_light = re.directed_light;
                        write_back_lighting(ent, current_entity);
                    }

                    // g2r
                    modtype_t::MOD_MDXM => {
                        if !ent.e.ghoul2.is_null() {
                            let re = ref_entity_from_tr(ent);
                            r_add_ghoul_surfaces(&re, assets, frame, cvars, &*engine_view.common);
                        }
                    }

                    // null model axis
                    modtype_t::MOD_BAD => {
                        // DEFERRED: RF_THIRD_PERSON/RF_SHADOW_ONLY — same
                        // never-guess-a-constant absence as the
                        // RT_SPRITE-family arm above; this arm's own
                        // first statement reads RF_THIRD_PERSON before
                        // anything else. The trailing
                        // `G2API_HaveWeGhoul2Models` check (once
                        // reachable) resolves to
                        // `mp_engine_ghoul2::api_models::
                        // g2api_have_we_ghoul2_models` (contra this
                        // packet's resolved-call-surface note marking it
                        // "NOT RESOLVED" — it exists, ported at wave-9
                        // adjacent work) but reaching it still needs a
                        // `&CGhoul2Info_v` for this entity, the same
                        // per-entity Ghoul2System threading gap
                        // `r_add_ghoul_surfaces`'s own doc comment
                        // (`tr_ghoul2.rs`) already blocks on.
                        // Source: oracle/codemp/renderer/tr_main.cpp:1445-1461
                        todo!(
                            "Port R_AddEntitySurfaces MOD_BAD arm — RF_THIRD_PERSON/RF_SHADOW_ONLY unported, oracle/codemp/renderer/tr_main.cpp:1445-1461"
                        )
                    }

                    modtype_t::MOD_MDXA => {
                        com_error(
                            errorParm_t::ERR_DROP,
                            "R_AddEntitySurfaces: Bad modeltype".to_string(),
                        );
                    }
                }
            }

            refEntityType_t::RT_ENT_CHAIN => {
                let shader = R_GetShaderByHandle(assets, engine_view.common, ent.e.customShader);
                let shader_sorted_index = assets
                    .shaders
                    .get(shader)
                    .map(|s| s.sorted_index)
                    .unwrap_or(0);
                let fog_index = R_SpriteFogNum(refdef_rdflags, fogs, ent.e.origin, ent.e.radius);
                R_AddDrawSurf(
                    SurfaceGeometry::Other,
                    shader_sorted_index,
                    shifted_entity_num,
                    rdf_nofog,
                    fog_index,
                    0,
                    draw_surfs,
                );
            }

            refEntityType_t::RT_POLY | refEntityType_t::RT_MAX_REF_ENTITY_TYPE => {
                com_error(
                    errorParm_t::ERR_DROP,
                    "R_AddEntitySurfaces: Bad reType".to_string(),
                );
            }
        }
    }
}

/// Raven `R_GenerateDrawSurfs` — appends this view's world, polygon, terrain,
/// and entity draw surfaces, setting up the projection matrix in between
/// (entities need it for LOD, so it must run after the world is bounded but
/// before entities are added, per the oracle's own ordering comment).
///
/// This fn has no globals of its own (`void R_GenerateDrawSurfs(void)`); its
/// signature is the union of its five already-ported callees' own threaded
/// parameters (wave 1/2/6/10, all below wave 11 — signatures are LAW, not
/// reshaped here). `frame` is `RenderWorld::frame: FrameState`
/// (`R_AddWorldSurfaces`/`R_AddEntitySurfaces`'s own parameter, this file's
/// established name); `frame_data` is this frame's `FrameData` event stream
/// (`R_AddPolygonSurfaces`'s own `frame: &'a FrameData` — renamed here only
/// to avoid colliding with the `FrameState` parameter, ties this fn's own
/// `'a` to the `DrawSurf<SurfaceGeometry<'a>>` payload). `refdef_rdflags`/
/// `refdef_fov_x`/`refdef_fov_y`/`refdef_num_dlights` are `tr.refdef`'s
/// bare-scalar fields, threaded exactly as `R_AddWorldSurfaces`/
/// `R_SetupProjection`/`R_AddEntitySurfaces` already require them (no
/// `TrRefdef` field yet — the same gap those fns' own PORT-NOTEs name).
/// `refdef` (the `trRefdef_t` tier-2 value) and `land_scape`/`land` are
/// `R_AddTerrainSurfaces`'s own already-ported parameters, passed straight
/// through. `dlights` is `tr.refdef.dlights`, threaded as `&mut [dlight_t]`
/// to match `R_AddEntitySurfaces`'s own signature; downgraded to `&[dlight_t]`
/// for `R_AddWorldSurfaces`'s read-only use via the standard `&mut T -> &T`
/// coercion. `fogs` is `tr.world->fogs`, `R_AddEntitySurfaces`'s own
/// established parameter (no `WorldAsset::fogs` field yet — the tier-2
/// transition audit's Group 1 `world_t` row names this as still pending the
/// `tr_bsp`/`tr_world` fog-array wave). `distance_cull` is
/// `tr.distanceCull` (`RenderAssets::distance_cull`, B11), `R_SetupProjection`'s
/// own parameter. `shifted_entity_num` is `tr.shiftedEntityNum` as
/// `R_AddTerrainSurfaces`'s own PORT-NOTE already documents (ambient state
/// set by this fn's caller, `R_RenderView`, not yet in this wave's packet —
/// threaded in rather than guessed at a fixed value, porting-rules §A2).
/// `engine_view`/`assets`/`cvars`/`models`/`entities`/`scratch`/`view`/
/// `draw_surfs` are `R_AddEntitySurfaces`/`R_AddTerrainSurfaces`/
/// `R_SetupProjection`'s own already-ported parameters, threaded straight
/// through; `common` for the calls that don't take the full `engine_view`
/// bundle is `engine_view.common`, reborrowed per call (this file's
/// established `engine_view.common` pattern, e.g. `R_AddEntitySurfaces`'s
/// `MOD_MESH`/`RT_ENT_CHAIN` arms above) rather than threaded as a second
/// top-level parameter.
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:1516-1531`
pub fn R_GenerateDrawSurfs<'a>(
    engine_view: &mut EngineHostView<'_>,
    assets: &mut RenderAssets,
    cvars: &mut RendererCvars,
    frame: &mut FrameState,
    frame_data: &'a FrameData,
    view: &mut viewParms_t,
    refdef: &trRefdef_t,
    refdef_rdflags: i32,
    refdef_fov_x: f32,
    refdef_fov_y: f32,
    // Held for the deferred `R_AddWorldSurfaces` call below (its only reader in
    // this fn); threaded by `R_RenderView` alongside the rest of the bundle.
    _refdef_num_dlights: i32,
    dlights: &mut [dlight_t],
    fogs: &[fog_t],
    distance_cull: f32,
    land_scape: &srfTerrain_t,
    land: &CmLandScape,
    shifted_entity_num: i32,
    entities: &mut [trRefEntity_t],
    scratch: &mut TrMainScratch,
    models: &RenderModels,
    draw_surfs: &mut Vec<DrawSurf<SurfaceGeometry<'a>>>,
) {
    // DEFERRED: the R_AddWorldSurfaces call. Its whole four-fn PVS chain
    // (R_AddWorldSurfaces -> R_MarkLeaves -> R_RecursiveWorldNode ->
    // R_AddWorldSurface) is landed and compiles, and its `ori`/`engine_view`
    // inputs are already available here (`&view.world` is R_RotateForViewer's
    // `tr.ori`). The one missing carrier is the world draw-surf sink
    // `Vec<DrawSurf<WorldSurfaceRef>>` (DEC-43.3): the frontend threads only
    // the `Vec<DrawSurf<SurfaceGeometry>>` list `R_SortDrawSurfs` consumes, so
    // a world list here would be a sink nothing reads (porting-rules §14 —
    // the identical block that keeps `R_AddBrushModelSurfaces`'s own loop
    // deferred). It lands when the R4 world draw-surf wave threads that list
    // and its backend consumer through R_RenderView (out of scope this wave).
    // Source: oracle/codemp/renderer/tr_main.cpp:1519 (R_AddWorldSurfaces call)

    R_AddPolygonSurfaces(frame_data, assets, engine_view.common, draw_surfs);

    R_AddTerrainSurfaces(
        engine_view.common,
        cvars,
        refdef,
        land_scape,
        land,
        view,
        shifted_entity_num,
        draw_surfs,
    );

    // set the projection matrix with the minimum zfar and now that we have
    // the world bounded this needs to be done before entities are added,
    // because they use the projection matrix for lod calculation
    R_SetupProjection(
        view,
        refdef_rdflags,
        refdef_fov_x,
        refdef_fov_y,
        distance_cull,
        engine_view.common,
        cvars,
    );

    R_AddEntitySurfaces(
        entities,
        view,
        scratch,
        engine_view,
        assets,
        models,
        cvars,
        frame,
        refdef_rdflags,
        fogs,
        dlights,
        draw_surfs,
    );
}

// ===== wave 12 =====
//
// `R_MirrorViewBySurface`/`R_SortDrawSurfs`/`R_RenderView` are mutually
// recursive (packet SCC 462): `R_RenderView` calls `R_SortDrawSurfs`, which
// calls `R_MirrorViewBySurface`, which calls `R_RenderView` again for the
// mirrored/portal sub-scene. All three thread the same giant parameter
// bundle this file's `R_GenerateDrawSurfs` (wave 11, LAW) already
// established — `R_RenderView` calls it directly, so its own signature is
// that bundle plus this fn's own inputs (`parms`, `frame_scene_num`,
// `refdef_time`, `gpu`); `R_SortDrawSurfs`/`R_MirrorViewBySurface` need the
// same bundle purely to forward it through the recursion.

/// Raven `MAX_DRAWSURFS` — `backEndData_t::drawSurfs` capacity, cited
/// directly from this wave's packet preamble (`_PREAMBLE.md`'s `## Seam
/// definition`: "`drawSurfs[MAX_DRAWSURFS=0x10000]`"); not itself in this
/// packet's own FILE-SCOPE CONSTANTS section, but given verbatim there
/// rather than guessed (never-guess-a-constant, porting-rules §A2).
///
/// The `#define` has an `_XBOX` twin (`0x4000`); MP retail builds the
/// non-`_XBOX` `0x10000`, this file's established platform-guard precedent.
///
/// Source: `oracle/codemp/renderer/tr_local.h:1207-1211` (use site:
/// `:2264`)
const MAX_DRAWSURFS: usize = 0x10000;

/// Whole-struct copy of a `viewParms_t` — every field the Rust struct has,
/// per the whole-struct-copy rule (`oldParms = tr.viewParms;`/`newParms =
/// tr.viewParms;` in `R_MirrorViewBySurface`, `tr.viewParms = *parms;` in
/// `R_RenderView`). `viewParms_t` (`tr_local::view_parms_t`) has no
/// `Clone`/`Copy` derive and this wave is scoped to `tr_main.rs` only, so a
/// manual memberwise copy stands in for one rather than editing that file.
fn clone_view_parms(v: &viewParms_t) -> viewParms_t {
    viewParms_t {
        ori: orientationr_t {
            origin: v.ori.origin,
            axis: v.ori.axis,
            viewOrigin: v.ori.viewOrigin,
            modelMatrix: v.ori.modelMatrix,
        },
        world: orientationr_t {
            origin: v.world.origin,
            axis: v.world.axis,
            viewOrigin: v.world.viewOrigin,
            modelMatrix: v.world.modelMatrix,
        },
        pvsOrigin: v.pvsOrigin,
        isPortal: v.isPortal,
        isMirror: v.isMirror,
        frameSceneNum: v.frameSceneNum,
        frameCount: v.frameCount,
        portalPlane: v.portalPlane,
        viewportX: v.viewportX,
        viewportY: v.viewportY,
        viewportWidth: v.viewportWidth,
        viewportHeight: v.viewportHeight,
        fovX: v.fovX,
        fovY: v.fovY,
        projectionMatrix: v.projectionMatrix,
        frustum: v.frustum,
        visBounds: v.visBounds,
        zFar: v.zFar,
    }
}

/// Raven `R_MirrorViewBySurface`. Out-param (`qboolean` return) -> `bool`.
///
/// Panics via `SurfIsOffscreen`'s loud stub (this file) until its owning R4
/// wave lands — the trivial-reject call below is this fn's third statement.
///
/// `draw_surf` is taken **by value** (not `&DrawSurf<..>`) — see this file's
/// `SurfaceGeometry`/`DrawSurf` `Copy`-derive note above: the caller
/// (`R_SortDrawSurfs`) copies the element out of `draw_surfs` before calling
/// this fn, so this fn can hold `draw_surfs: &mut Vec<..>` (needed for the
/// `R_RenderView` recursion below, which appends to it) without aliasing a
/// borrow into the same `Vec`.
///
/// `view` is `tr.viewParms` (read/written — this file's established tier-2
/// stand-in, top-of-file PORT-NOTE). `frame_scene_num` is `tr.frameSceneNum`
/// (STATE HOMES: no `FrameState` field exists yet for it — same gap
/// `tr_cmds.rs`'s own `R_ToggleSmpFrame`-family DEFERRED note records for
/// this exact global; threaded as a bare scalar here instead, this file's
/// established `refdef_rdflags`-style precedent for "no landed carrier field
/// yet", forwarded straight through to the `R_RenderView` call).
/// `refdef_time` is `tr.refdef.time`, `R_GetPortalOrientations`'s own
/// already-ported parameter. `gpu` is `RenderWorld::frame`-adjacent render
/// -thread-local `GpuResources` (R2 `glState` row) — not part of
/// `R_GenerateDrawSurfs`'s own parameter list, but required to forward
/// through to `R_RenderView`'s own `R_DebugGraphics` call. Every other
/// parameter is `R_GenerateDrawSurfs`'s own already-ported bundle (wave 11),
/// forwarded straight through the recursion.
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:971-1019`
#[allow(clippy::too_many_arguments)]
pub fn R_MirrorViewBySurface<'a>(
    draw_surf: DrawSurf<SurfaceGeometry<'a>>,
    entity_num: i32,
    frame_scene_num: i32,
    refdef_time: i32,
    engine_view: &mut EngineHostView<'_>,
    assets: &mut RenderAssets,
    cvars: &mut RendererCvars,
    frame: &mut FrameState,
    gpu: &mut GpuResources,
    frame_data: &'a FrameData,
    view: &mut viewParms_t,
    refdef: &trRefdef_t,
    refdef_rdflags: i32,
    refdef_fov_x: f32,
    refdef_fov_y: f32,
    refdef_num_dlights: i32,
    dlights: &mut [dlight_t],
    fogs: &[fog_t],
    distance_cull: f32,
    land_scape: &srfTerrain_t,
    land: &CmLandScape,
    shifted_entity_num: i32,
    entities: &mut [trRefEntity_t],
    scratch: &mut TrMainScratch,
    models: &RenderModels,
    draw_surfs: &mut Vec<DrawSurf<SurfaceGeometry<'a>>>,
) -> bool {
    // don't recursively mirror
    if view.isPortal != 0 {
        Com_DPrintf(
            engine_view.common,
            &format!(
                "{}WARNING: recursive mirror/portal found\n",
                S_COLOR_RED.to_str().expect("S_COLOR_RED is ASCII")
            ),
        );
        return false;
    }

    if engine_view.common.cvar(cvars.r_noportals).integer != 0
        || engine_view.common.cvar(cvars.r_fastsky).integer == 1
    {
        return false;
    }

    // trivially reject portal/mirror
    if SurfIsOffscreen(
        &draw_surf,
        &assets.sorted_shaders,
        entities,
        view,
        scratch,
        frame,
    ) {
        return false;
    }

    // save old viewParms so we can return to it after the mirror view
    let old_parms = clone_view_parms(view);

    let mut new_parms = clone_view_parms(view);
    new_parms.isPortal = qtrue;

    let (surface, camera, pvs_origin, mirror) = match R_GetPortalOrientations(
        Some(&draw_surf.surface),
        entity_num,
        entities,
        refdef_time,
        view,
        scratch,
    ) {
        Some(v) => v,
        None => return false, // bad portal, no portalentity
    };

    new_parms.ori.origin = R_MirrorPoint(old_parms.ori.origin, &surface, &camera);

    VectorSubtract(
        vec3_origin,
        camera.axis[0],
        &mut new_parms.portalPlane.normal,
    );
    new_parms.portalPlane.dist = DotProduct(camera.origin, new_parms.portalPlane.normal);

    new_parms.ori.axis[0] = R_MirrorVector(old_parms.ori.axis[0], &surface, &camera);
    new_parms.ori.axis[1] = R_MirrorVector(old_parms.ori.axis[1], &surface, &camera);
    new_parms.ori.axis[2] = R_MirrorVector(old_parms.ori.axis[2], &surface, &camera);

    new_parms.pvsOrigin = pvs_origin;
    // bool -> qboolean (dictionary): `mirror` already came back through
    // `R_GetPortalOrientations`'s own `bool` out-param translation.
    new_parms.isMirror = mirror as i32;

    // OPTIMIZE: restrict the viewport on the mirrored view

    // render the mirror view
    R_RenderView(
        &new_parms,
        frame_scene_num,
        refdef_time,
        view,
        engine_view,
        assets,
        cvars,
        frame,
        gpu,
        frame_data,
        refdef,
        refdef_rdflags,
        refdef_fov_x,
        refdef_fov_y,
        refdef_num_dlights,
        dlights,
        fogs,
        distance_cull,
        land_scape,
        land,
        shifted_entity_num,
        entities,
        scratch,
        models,
        draw_surfs,
    );

    *view = old_parms;

    true
}

/// Raven `R_SortDrawSurfs`.
///
/// `draw_surfs`/`first_draw_surf` replace the oracle's `drawSurf_t
/// *drawSurfs, int numDrawSurfs` pointer+length pair: `draw_surfs` is the
/// file's owned, growing `Vec` standing in for `tr.refdef.drawSurfs`
/// (established by `R_AddDrawSurf`'s own PORT-NOTE — "the fixed-size ring
/// buffer is replaced by a plain `Vec` append"), and `first_draw_surf` is
/// the oracle's pointer offset into it. A borrowed slice of the range being
/// sorted was rejected: `R_MirrorViewBySurface`'s recursion through
/// `R_RenderView` -> `R_GenerateDrawSurfs` appends new entries onto this
/// same `Vec` mid-loop (exactly as Raven's own fixed
/// `drawSurfs[MAX_DRAWSURFS]` array does — new entries land past the range
/// being sorted, never observed by this fn's own bounded loop), which would
/// alias a live slice/reallocate out from under it; indexing `draw_surfs`
/// fresh each iteration instead avoids that without any `unsafe`.
///
/// `sorted_shaders`/shader lookups read `assets.sorted_shaders`/
/// `assets.shaders` (`RenderAssets`, `R2-D3`/`R2-D4`). `r_portalOnly` reads
/// through the live cvar table (`RendererCvars::r_portalOnly`, DEC-37
/// A13.1). Every other parameter is `R_MirrorViewBySurface`'s own bundle
/// above, forwarded straight through.
///
/// PORT-NOTE: `R_AddDrawSurfCmd`'s two call sites (the early-return "we
/// still need to add it for hyperspace cases" branch, and the fall-through
/// at the end) are both dropped, not merely deferred — `tr_cmds.rs`'s own
/// `R_AddDrawSurfCmd` DEFERRED note (`:96-107`) already establishes there is
/// "no remaining R2-carrier behavior for this fn to perform": `drawSurfs` is
/// already this file's owned `Vec` (no ring-buffer command needed to expose
/// it), and `viewParms`/`refdef` already cross via `FrameEvent::RenderScene`
/// pushed by the not-yet-ported `RE_RenderScene`.
/// (R2 `### A1 disposition table` row `RC_DRAW_SURFS`)
/// Source: `oracle/codemp/renderer/tr_cmds.cpp:169-183`
///
/// Only the non-`_XBOX` `qsortFast` call site is transcribed (the `_XBOX`
/// build calls it a second time, post-loop, instead of pre-loop) — MP never
/// builds `_XBOX`, the same `R_SetupProjection`-established precedent for
/// dropping `_XBOX`-only branches in this file.
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:1304-1362`
#[allow(clippy::too_many_arguments)]
pub fn R_SortDrawSurfs<'a>(
    draw_surfs: &mut Vec<DrawSurf<SurfaceGeometry<'a>>>,
    first_draw_surf: usize,
    frame_scene_num: i32,
    refdef_time: i32,
    engine_view: &mut EngineHostView<'_>,
    assets: &mut RenderAssets,
    cvars: &mut RendererCvars,
    frame: &mut FrameState,
    gpu: &mut GpuResources,
    frame_data: &'a FrameData,
    view: &mut viewParms_t,
    refdef: &trRefdef_t,
    refdef_rdflags: i32,
    refdef_fov_x: f32,
    refdef_fov_y: f32,
    refdef_num_dlights: i32,
    dlights: &mut [dlight_t],
    fogs: &[fog_t],
    distance_cull: f32,
    land_scape: &srfTerrain_t,
    land: &CmLandScape,
    shifted_entity_num: i32,
    entities: &mut [trRefEntity_t],
    scratch: &mut TrMainScratch,
    models: &RenderModels,
) {
    // it is possible for some views to not have any surfaces
    if draw_surfs.len() <= first_draw_surf {
        // R_AddDrawSurfCmd( drawSurfs, numDrawSurfs ) — PORT-NOTE above:
        // dropped, no remaining behavior.
        return;
    }

    // if we overflowed MAX_DRAWSURFS, the drawsurfs wrapped around in the
    // buffer and we will be missing the first surfaces, not the last ones
    let mut num_draw_surfs = draw_surfs.len() - first_draw_surf;
    if num_draw_surfs > MAX_DRAWSURFS {
        num_draw_surfs = MAX_DRAWSURFS;
    }

    // sort the drawsurfs by sort type, then orientation, then shader
    qsortFast(&mut draw_surfs[first_draw_surf..first_draw_surf + num_draw_surfs]);

    // check for any pass through drawing, which may cause another view to
    // be rendered first
    for i in 0..num_draw_surfs {
        let sort = draw_surfs[first_draw_surf + i].sort;
        let (entity_num, shader_handle, _fog_num, _dlighted) =
            R_DecomposeSort(sort, &assets.sorted_shaders);

        let shader_entry = assets.shaders.get(shader_handle);
        let shader_sort = shader_entry.map(|s| s.sort).unwrap_or(0.0);

        if shader_sort > shaderSort_t::SS_PORTAL as i32 as f32 {
            break;
        }

        // no shader should ever have this sort type
        if shader_sort == shaderSort_t::SS_BAD as i32 as f32 {
            let shader_name = shader_entry.map(|s| s.name.as_str()).unwrap_or("");
            com_error(
                errorParm_t::ERR_DROP,
                format!("Shader '{}'with sort == SS_BAD", shader_name),
            );
        }

        // owned copy — see `R_MirrorViewBySurface`'s own doc comment for why
        // this can't be a borrow into `draw_surfs`.
        let draw_surf = draw_surfs[first_draw_surf + i];

        // if the mirror was completely clipped away, we may need to check
        // another surface
        if R_MirrorViewBySurface(
            draw_surf,
            entity_num,
            frame_scene_num,
            refdef_time,
            engine_view,
            assets,
            cvars,
            frame,
            gpu,
            frame_data,
            view,
            refdef,
            refdef_rdflags,
            refdef_fov_x,
            refdef_fov_y,
            refdef_num_dlights,
            dlights,
            fogs,
            distance_cull,
            land_scape,
            land,
            shifted_entity_num,
            entities,
            scratch,
            models,
            draw_surfs,
        ) {
            // this is a debug option to see exactly what is being mirrored
            if engine_view.common.cvar(cvars.r_portalOnly).integer != 0 {
                return;
            }
            break; // only one mirror view at a time
        }
    }

    // R_AddDrawSurfCmd( drawSurfs, numDrawSurfs ) — PORT-NOTE above: dropped,
    // no remaining behavior.
}

/// Raven `R_RenderView`.
///
/// `parms` is the oracle's `viewParms_t *parms` in-param (the new view to
/// install); `view` is `tr.viewParms` itself (written wholesale from
/// `parms`, then read/written by every callee below — this file's
/// established tier-2 stand-in). `frame` is `RenderWorld::frame: FrameState`
/// — `tr.viewCount` (incremented twice, faithfully kept as two separate
/// `+= 1`s rather than folded into `+= 2`) and `tr.frameCount` (`view
/// .frameCount = frame.frame_count`, `FrameState::frame_count`, the same
/// carrier `tr_cmds.rs`/`tr_image.rs` already use for `tr.frameCount`).
/// `frame_scene_num` is `tr.frameSceneNum` — see `R_MirrorViewBySurface`'s
/// own doc comment for why it's threaded as a bare scalar rather than a
/// `FrameState` field. `gpu` is `RenderWorld`'s render-thread-local
/// `GpuResources`, needed only for the trailing `R_DebugGraphics` call
/// (`R_GenerateDrawSurfs` itself doesn't touch GL state). Every other
/// parameter is `R_GenerateDrawSurfs`'s own already-ported bundle (wave 11),
/// forwarded straight through; `tr.refdef.numDrawSurfs`/the oracle's
/// `firstDrawSurf` local are `draw_surfs.len()` snapshots (see
/// `R_SortDrawSurfs`'s own doc comment for the `Vec`-append equivalence).
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:1595-1627`
#[allow(clippy::too_many_arguments)]
pub fn R_RenderView<'a>(
    parms: &viewParms_t,
    frame_scene_num: i32,
    refdef_time: i32,
    view: &mut viewParms_t,
    engine_view: &mut EngineHostView<'_>,
    assets: &mut RenderAssets,
    cvars: &mut RendererCvars,
    frame: &mut FrameState,
    gpu: &mut GpuResources,
    frame_data: &'a FrameData,
    refdef: &trRefdef_t,
    refdef_rdflags: i32,
    refdef_fov_x: f32,
    refdef_fov_y: f32,
    refdef_num_dlights: i32,
    dlights: &mut [dlight_t],
    fogs: &[fog_t],
    distance_cull: f32,
    land_scape: &srfTerrain_t,
    land: &CmLandScape,
    shifted_entity_num: i32,
    entities: &mut [trRefEntity_t],
    scratch: &mut TrMainScratch,
    models: &RenderModels,
    draw_surfs: &mut Vec<DrawSurf<SurfaceGeometry<'a>>>,
) {
    if parms.viewportWidth <= 0 || parms.viewportHeight <= 0 {
        return;
    }

    frame.view_count += 1;

    *view = clone_view_parms(parms);
    view.frameSceneNum = frame_scene_num;
    view.frameCount = frame.frame_count;

    let first_draw_surf = draw_surfs.len();

    // Raven increments `tr.viewCount` a second time here.
    frame.view_count += 1;

    // set viewParms.world
    //
    // DEFERRED: `tr.ori` — `R_RotateForViewer`'s return value IS the oracle's
    // `tr.ori` write (its own doc comment: "Out-param (via `tr.ori`) ->
    // return value"), read for the rest of this view by the whole
    // `R_GenerateDrawSurfs` subtree. It is discarded here because
    // `FrameState::ori` is the field-less `OrientationR` placeholder
    // (`render_state/placeholders.rs`) — no carrier to store it in, and this
    // fn may not add one.
    // Source: oracle/codemp/renderer/tr_main.cpp:1612-1613
    R_RotateForViewer(view);

    R_SetupFrustum(view);

    R_GenerateDrawSurfs(
        engine_view,
        assets,
        cvars,
        frame,
        frame_data,
        view,
        refdef,
        refdef_rdflags,
        refdef_fov_x,
        refdef_fov_y,
        refdef_num_dlights,
        dlights,
        fogs,
        distance_cull,
        land_scape,
        land,
        shifted_entity_num,
        entities,
        scratch,
        models,
        draw_surfs,
    );

    R_SortDrawSurfs(
        draw_surfs,
        first_draw_surf,
        frame_scene_num,
        refdef_time,
        engine_view,
        assets,
        cvars,
        frame,
        gpu,
        frame_data,
        view,
        refdef,
        refdef_rdflags,
        refdef_fov_x,
        refdef_fov_y,
        refdef_num_dlights,
        dlights,
        fogs,
        distance_cull,
        land_scape,
        land,
        shifted_entity_num,
        entities,
        scratch,
        models,
    );

    // draw main system development information (surface outlines, etc)
    R_DebugGraphics(
        engine_view.common.cvar(cvars.r_debugSurface).integer,
        assets.white_image,
        assets,
        engine_view.common,
        cvars,
        frame,
        gpu,
    );
}
