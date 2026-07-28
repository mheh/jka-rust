//! Raven `tr_world.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_world.cpp`

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_engine_qcommon::cm_load::{CM_LeafArea, CM_LeafCluster};
use mp_engine_qcommon::cm_test::{CM_ClusterPVSBits, CM_PointLeafnum};
use mp_engine_qcommon::cm_trace::CM_BoxTrace;
use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::common::{Common, EngineHostView};
use mp_engine_qcommon::files_common::{
    FS_FCloseFile, FS_FOpenFileRead, FS_FOpenFileWrite, FS_Read, FS_Write,
};
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::{
    cplane_t, qhandle_t, vec3_t, CONTENTS_SOLID, CONTENTS_TERRAIN, SURF_NOIMPACT,
};
use native_math::qmath::{
    _DotProduct, _VectorMA, _VectorScale, _VectorSubtract, vec3_origin, CrossProduct,
    VectorCompare, VectorInverse, VectorLength, VectorSet,
};
use native_types::fileHandle_t;

use crate::render_state::frame_data::FrameData;
use crate::render_state::frame_event::FrameEvent;
use crate::render_state::frame_state::FrameState;
use crate::render_state::placeholders::RefEntity;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::tr_bsp::{Surface, SurfaceData, SurfaceFace, SurfaceTriangles};
use crate::tr_curve::GridMesh;
use crate::tr_light::{
    DlightBmodel, DlightSurface, DlightSurfaceData, R_DlightBmodel, R_SetupEntityLighting,
};
use crate::tr_local::cull_type_t::cullType_t;
use crate::tr_local::dlight_s::dlight_t;
use crate::tr_local::msurface_s::SurfaceRef;
use crate::tr_local::orientationr_t::orientationr_t;
use crate::tr_local::shader_s::shader_t;
use crate::tr_main::{
    DrawSurf, R_AddDrawSurf, R_CullLocalBox, R_CullLocalPointAndRadius, R_CullPointAndRadius,
    WorldSurfaceRef, CULL_CLIP, CULL_IN, CULL_OUT,
};
use crate::tr_model::render_models::RenderModels;
use crate::tr_public::ref_flags::RDF_NOWORLDMODEL;

/// Raven `Q_CastShort2Float` — widen a packed lightgrid short to a float.
/// Out-param `float *f` becomes a return value (§C7).
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:16-19`
pub fn Q_CastShort2Float(s: i16) -> f32 {
    s as f32
}

/// Raven `R_DlightFace` — cull a planar face's dlight bitmask against the
/// dlights active in this refdef, returning (and stashing on the face) the
/// surviving mask.
///
/// PORT-NOTE: `tr.refdef.dlights`/`num_dlights` (owned by `FrameState::refdef:
/// TrRefdef`, fields land with the `tr_scene` R3 wave — still an empty
/// placeholder) and `tr.pc.c_dlightSurfacesCulled` (`FrameState::counters:
/// BackEndCounters`, R4 backend wave) are threaded in directly rather than
/// via a whole `FrameState` reference (porting-rules §4, "state is threaded,
/// not reached") — this stays a private per-surface helper either way; the
/// caller slices them off `FrameState` once those waves land the fields.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:278-301`
pub fn R_DlightFace(
    face: &mut SurfaceFace,
    mut dlight_bits: i32,
    dlights: &[dlight_t],
    dlight_surfaces_culled: &mut u32,
) -> i32 {
    for (i, dl) in dlights.iter().enumerate() {
        if dlight_bits & (1 << i) == 0 {
            continue;
        }
        let d = _DotProduct(dl.origin, face.plane.normal) - face.plane.dist;
        if !VectorCompare(face.plane.normal, vec3_origin) && (d < -dl.radius || d > dl.radius) {
            // dlight doesn't reach the plane
            dlight_bits &= !(1 << i);
        }
    }

    if dlight_bits == 0 {
        *dlight_surfaces_culled += 1;
    }

    face.dlight_bits = dlight_bits;
    dlight_bits
}

/// Raven `R_DlightGrid` — dlight culling for bezier-patch (grid) surfaces,
/// bounds-box test against each active dlight's radius.
///
/// PORT-NOTE: same threading rationale as `R_DlightFace`.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:303-329`
pub fn R_DlightGrid(
    grid: &mut GridMesh,
    mut dlight_bits: i32,
    dlights: &[dlight_t],
    dlight_surfaces_culled: &mut u32,
) -> i32 {
    for (i, dl) in dlights.iter().enumerate() {
        if dlight_bits & (1 << i) == 0 {
            continue;
        }
        if dl.origin[0] - dl.radius > grid.mesh_bounds[1][0]
            || dl.origin[0] + dl.radius < grid.mesh_bounds[0][0]
            || dl.origin[1] - dl.radius > grid.mesh_bounds[1][1]
            || dl.origin[1] + dl.radius < grid.mesh_bounds[0][1]
            || dl.origin[2] - dl.radius > grid.mesh_bounds[1][2]
            || dl.origin[2] + dl.radius < grid.mesh_bounds[0][2]
        {
            // dlight doesn't reach the bounds
            dlight_bits &= !(1 << i);
        }
    }

    if dlight_bits == 0 {
        *dlight_surfaces_culled += 1;
    }

    grid.dlight_bits = dlight_bits;
    dlight_bits
}

/// Raven `R_DlightTrisurf` — dlight culling for triangle-soup surfaces is
/// unimplemented; the oracle's `#if 0` fallback body below the early return
/// never compiles and is dropped as dead surface (porting-rules §20).
///
/// Raven: FIXME: more dlight culling to trisurfs...
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:332-363`
pub fn R_DlightTrisurf(surf: &mut SurfaceTriangles, dlight_bits: i32) -> i32 {
    surf.dlight_bits = dlight_bits;
    dlight_bits
}

/// Raven `GetQuadArea` — sum of two triangles' squared cross-product-derived
/// "disSqr" areas for a quad `v1..v4`.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:613-632`
pub fn GetQuadArea(v1: vec3_t, v2: vec3_t, v3: vec3_t, v4: vec3_t) -> f32 {
    let mut vec1: vec3_t = [0.0; 3];
    let mut vec2: vec3_t = [0.0; 3];
    let mut dis1: vec3_t = [0.0; 3];
    let mut dis2: vec3_t = [0.0; 3];

    // Get area of tri1
    _VectorSubtract(v1, v2, &mut vec1);
    _VectorSubtract(v1, v4, &mut vec2);
    CrossProduct(vec1, vec2, &mut dis1);
    _VectorScale(dis1, 0.25, &mut dis1);

    // Get area of tri2
    _VectorSubtract(v3, v2, &mut vec1);
    _VectorSubtract(v3, v4, &mut vec2);
    CrossProduct(vec1, vec2, &mut dis2);
    _VectorScale(dis2, 0.25, &mut dis2);

    // Return addition of disSqr of each tri area
    dis1[0] * dis1[0]
        + dis1[1] * dis1[1]
        + dis1[2] * dis1[2]
        + dis2[0] * dis2[0]
        + dis2[1] * dis2[1]
        + dis2[2] * dis2[2]
}

/// Per-subsystem state for the wireframe-automap generator — Raven's
/// `g_autoMapFrame`/`g_autoMapNextFree`/`g_autoMapValid` file statics, NAMED
/// BY THIS WAVE (DEC-37 A13.3): `tr_world` owns this subsystem. The
/// intrusive singly-linked `wireframeMapSurf_t` list becomes an owned `Vec`;
/// the `g_autoMapNextFree` "resume from here" cursor becomes an index into
/// it instead of a `wireframeMapSurf_t **` cursor.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:782,784` (externs)
pub struct WireframeAutomap {
    pub surfs: Vec<WireframeMapSurf>,
    pub next_free: usize,
    pub valid: bool,
    /// `g_lastHeight` — the `g_playerHeight` value the per-point alpha/color
    /// table was last computed for (`R_DrawWireframeAutomap`'s recompute
    /// guard). NAMED BY THIS WAVE (DEC-37 A13.3): a per-subsystem field
    /// alongside `valid`/`next_free`, not a bare file-scope static.
    ///
    /// Source: `oracle/codemp/renderer/tr_world.cpp:1361-1480` (usage)
    pub last_height: f32,
    /// `g_lastHeightValid` — whether `last_height` holds a real value yet.
    ///
    /// Source: `oracle/codemp/renderer/tr_world.cpp:1361-1480` (usage)
    pub last_height_valid: bool,
}

impl Default for WireframeAutomap {
    fn default() -> Self {
        WireframeAutomap {
            surfs: Vec::new(),
            next_free: 0,
            valid: false,
            last_height: 0.0,
            last_height_valid: false,
        }
    }
}

/// Raven `wireframeSurfPoint_t` — one point of an automap wireframe outline.
///
/// Type definition source: `oracle/codemp/renderer/tr_world.cpp:760-765`
#[derive(Clone, Copy, Default)]
pub struct WireframeSurfPoint {
    pub xyz: vec3_t,
    pub alpha: f32,
    pub color: vec3_t,
}

impl WireframeSurfPoint {
    /// Byte width of Raven's `wireframeSurfPoint_t` — what
    /// `R_WriteWireframeMapToFile`/`R_GetWireframeMapFromFile` size the
    /// on-disk payload from (`sizeof(wireframeSurfPoint_t)`: three `vec3_t`
    /// floats, one `float`, three more, no padding).
    ///
    /// Source: `oracle/codemp/renderer/tr_world.cpp:1110`
    pub const WIRE_SIZE: usize = 7 * core::mem::size_of::<f32>();

    /// This point's on-disk bytes, in Raven's field order.
    fn write_to(&self, out: &mut Vec<u8>) {
        for f in [
            self.xyz[0],
            self.xyz[1],
            self.xyz[2],
            self.alpha,
            self.color[0],
            self.color[1],
            self.color[2],
        ] {
            out.extend_from_slice(&f.to_ne_bytes());
        }
    }

    /// The read-back counterpart of `write_to`, for
    /// `R_GetWireframeMapFromFile` — `bytes` must be exactly `WIRE_SIZE`
    /// long, in the same field order `write_to` emits.
    fn read_from(bytes: &[u8]) -> Self {
        let mut f = [0.0f32; 7];
        for (i, chunk) in bytes.chunks_exact(4).take(7).enumerate() {
            f[i] = f32::from_ne_bytes(chunk.try_into().unwrap());
        }
        WireframeSurfPoint {
            xyz: [f[0], f[1], f[2]],
            alpha: f[3],
            color: [f[4], f[5], f[6]],
        }
    }
}

/// Byte width of the per-surface record header the automap file format
/// carries ahead of a surface's points.
///
/// Raven's writer/reader both work from the *whole* `wireframeMapSurf_t`
/// struct, whose first two members are `bool completelyTransparent` (1 byte,
/// padded to the `int`'s alignment) then `int numPoints`
/// (`oracle/codemp/renderer/tr_world.cpp:767-774`). The reader's
/// `memcpy(newSurf->points, &surfs->points, ...)` therefore starts the point
/// payload at `offsetof(points)` = 8, with `numPoints` read at 4 — the header
/// is 8 bytes, not 4.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:767-774,1136,1187`
const WIRE_SURF_HEADER_SIZE: usize = 2 * core::mem::size_of::<i32>();

/// Raven `wireframeMapSurf_t` — one automap wireframe outline. The oracle's
/// `wireframeSurfPoint_t *points` heap array becomes an owned `Vec`
/// (porting-rules §C9) and its `next` list link is dissolved into
/// `WireframeAutomap::surfs`.
///
/// Type definition source: `oracle/codemp/renderer/tr_world.cpp:767-775`
pub struct WireframeMapSurf {
    pub num_points: i32,
    pub points: Vec<WireframeSurfPoint>,
    /// `completelyTransparent` — `R_DrawWireframeAutomap`'s per-surface
    /// skip flag (every point alpha'd fully out this recompute pass).
    ///
    /// Source: `oracle/codemp/renderer/tr_world.cpp:1365,1430`
    pub completely_transparent: bool,
}

/// Raven `R_GetNewWireframeMapSurf` — hand back the next wireframe-surface
/// slot.
///
/// PORT-NOTE: the oracle walks `g_autoMapFrame.surfs`'s intrusive list from
/// `g_autoMapNextFree` until it finds a null `next` (a perf shortcut over
/// rescanning from the head every call); since the list only ever grows
/// during one generation pass (`R_DestroyWireframeMap` clears it wholesale,
/// nothing frees an individual mid-list node), that walk always lands at the
/// tail — appending to the owned `Vec` reproduces the same observable
/// behaviour without the pointer walk (porting-rules §10).
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:787-805`
pub fn R_GetNewWireframeMapSurf(automap: &mut WireframeAutomap) -> usize {
    let index = automap.surfs.len();
    automap.surfs.push(WireframeMapSurf {
        num_points: 0,
        points: Vec::new(),
        completely_transparent: false,
    });
    automap.next_free = automap.surfs.len();
    index
}

//TODO: Port R_NodeHasOppositeFaces
// Source: oracle/codemp/renderer/tr_world.cpp:927-986
// Both carriers it waited on are landed now — `Node::firstmarksurface`/
// `nummarksurfaces` and `WorldAsset::mark_surfaces` (tr_bsp wave 1) and
// `WorldAsset::surfaces`' owned `SurfaceData::Face` payload (DEC-43) — so
// this fn is unblocked; it simply has no body yet. Its sole caller is
// `R_RecursiveWorldNode`, itself unported (see its own note below).

/// Raven `R_DestroyWireframeMap` — invalidate and free the wireframe
/// automap. Owned `Vec`/`Vec<u8>` drops replace the oracle's manual `Z_Free`
/// walk (porting-rules §9).
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:1067-1095`
pub fn R_DestroyWireframeMap(automap: &mut WireframeAutomap) {
    if !automap.valid {
        // not valid to begin with
        return;
    }

    automap.surfs.clear();

    // invalidate everything
    automap.valid = false;
    automap.next_free = 0;
}

/// Raven `R_WriteWireframeMapToFile` — serialize the wireframe automap to
/// `blahblah.bla` (Raven's own placeholder filename).
///
/// The oracle's per-surface `memcpy(out, surf, sizeof(wireframeSurfPoint_t)*
/// surf->numPoints + sizeof(int))` copies from the `wireframeMapSurf_t`
/// header, not from `surf->points`, so it reads past the struct — UB, and it
/// never emits the point data it sizes for. Ported as the evident intent
/// (porting-rules §19): the record layout the *reader*'s own offsets imply,
/// `completelyTransparent` (padded to 4) then `numPoints` then that
/// surface's points, with the record advancing by its full width so the two
/// halves round-trip. See [`WIRE_SURF_HEADER_SIZE`].
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:1098-1155`
pub fn R_WriteWireframeMapToFile(
    view: &mut EngineHostView<'_>,
    automap: &WireframeAutomap,
) -> bool {
    // let's go through and see how much space we're going to need to stuff
    // all this data into
    let required_size: usize = automap
        .surfs
        .iter()
        .map(|surf| {
            // memory for each point, then memory for the record header
            WireframeSurfPoint::WIRE_SIZE * surf.num_points as usize + WIRE_SURF_HEADER_SIZE
        })
        .sum();

    if required_size == 0 {
        // nothing to do..?
        return false;
    }

    let f = FS_FOpenFileWrite(view.common, "blahblah.bla");
    if f == 0 {
        // can't create?
        return false;
    }

    // allocate the memory we will need, then go through and put the data
    // into it
    let mut out = Vec::with_capacity(required_size);
    for surf in &automap.surfs {
        // `bool completelyTransparent` + its 3 bytes of tail padding.
        out.push(surf.completely_transparent as u8);
        out.extend_from_slice(&[0u8; 3]);
        out.extend_from_slice(&surf.num_points.to_ne_bytes());
        for point in &surf.points {
            point.write_to(&mut out);
        }
    }

    // now write the buffer, and close
    FS_Write(
        view.common,
        out.as_ptr() as *const (),
        required_size as c_int,
        f,
    );
    FS_FCloseFile(view.common, f);

    true
}

/// Raven `R_AutomapElevationAdjustment` — `g_playerHeight` crosses in
/// `FrameData` as `FrameEvent::AutomapElevAdj` (R2 `## State ownership`).
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:1239-1242`
pub fn R_AutomapElevationAdjustment(frame: &mut FrameData, new_height: f32) {
    frame.events.push(FrameEvent::AutomapElevAdj(new_height));
}

// DEFERRED: R_PointInLeaf — needs WorldAsset's owned node arena
// (RenderAssets::world: Option<WorldAsset>, still an empty placeholder
// pending the tr_bsp wave, tier-2 transition audit Group 1) to walk in place
// of mnode_t's raw parent/plane/children pointers.
// Source: oracle/codemp/renderer/tr_world.cpp:1672-1700

// DEFERRED: R_ClusterPVS — needs WorldAsset::vis/novis/numClusters/
// clusterBytes, still an empty placeholder pending the tr_bsp wave (tier-2
// transition audit Group 1).
// Source: oracle/codemp/renderer/tr_world.cpp:1707-1718

/// Raven `R_inPVS` — potential-visibility test between two points via the
/// collision world's PVS clusters.
///
/// PORT-NOTE: Raven's `byte *mask` in-parameter is immediately overwritten
/// (`mask = (byte *) CM_ClusterPVS(...)`) before any read, so it carries no
/// observable behaviour in — dropped per §C7 (out-param already folded into
/// the `bool` return). The commented-out `CM_AreasConnected` check stays
/// dropped, matching the oracle's own dead `//` line.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:1725-1746`
pub fn R_inPVS(cm: &mut CollisionWorld, p1: vec3_t, p2: vec3_t) -> bool {
    let leafnum = CM_PointLeafnum(cm, p1);
    let mut cluster = CM_LeafCluster(cm, leafnum);
    let _area1 = CM_LeafArea(cm, leafnum);

    // agh, the damn snapshot mask doesn't work for this
    let mask = CM_ClusterPVSBits(cm, cluster).map(<[u8]>::to_vec);

    let leafnum2 = CM_PointLeafnum(cm, p2);
    cluster = CM_LeafCluster(cm, leafnum2);
    let _area2 = CM_LeafArea(cm, leafnum2);

    if let Some(mask) = mask {
        let byte = mask[(cluster >> 3) as usize];
        if byte & (1 << (cluster & 7)) == 0 {
            return false;
        }
    }

    // this doesn't freakin work
    // if !CM_AreasConnected(area1, area2) { return false; } // a door blocks sight
    true
}

/// Raven `R_CullTriSurf` — box-cull a triangle-soup surface against the
/// view frustum, `qboolean` collapsed to `bool` (§C7).
///
/// `r_nocull_integer`/`ori`/`frustum` are threaded straight through to
/// `R_CullLocalBox` exactly as that already-ported fn's own doc comment
/// establishes ("read through the live engine cvar table by the caller,
/// threaded in here rather than reached for") — this fn is a thin wrapper,
/// so it inherits the same threading rather than reaching into
/// `RendererCvars`/`FrameState` itself.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:58-67`
pub fn R_CullTriSurf(
    cv: &SurfaceTriangles,
    r_nocull_integer: i32,
    ori: &orientationr_t,
    frustum: &[cplane_t; 4],
) -> bool {
    let box_cull = R_CullLocalBox(cv.bounds, r_nocull_integer, ori, frustum);

    box_cull == CULL_OUT
}

/// Raven `R_DlightSurface` — dispatch dlight culling to the surface's
/// concrete kind, then tally the surface as dlit if any bits survived.
///
/// PORT-NOTE: `dlights`/`dlight_surfaces_culled` are threaded straight
/// through to `R_DlightFace`/`R_DlightGrid`, mirroring those fns' own
/// established threading (see their doc comments above); `dlight_surfaces`
/// is `tr.pc.c_dlightSurfaces` threaded the same way — `FrameState::counters
/// : BackEndCounters` is still an empty placeholder pending the R4 backend
/// wave, so the counter is threaded in directly rather than via a whole
/// `FrameState` reference (porting-rules §4).
///
/// PORT-NOTE: the tagged-union dispatch over `msurface_t.data` is a match on
/// the owned [`SurfaceData`] (DEC-43.1); each arm mutates its concrete
/// surface exactly as the oracle does, so `surf` is `&mut`, matching the
/// oracle's own non-const `msurface_t *`. The `Skip`/`Flare` arms are the
/// oracle's `default:` (no dlight handler).
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:374-390`
pub fn R_DlightSurface(
    surf: &mut Surface,
    dlight_bits: i32,
    dlights: &[dlight_t],
    dlight_surfaces_culled: &mut u32,
    dlight_surfaces: &mut u32,
) -> i32 {
    let dlight_bits = match &mut surf.data {
        SurfaceData::Face(face) => R_DlightFace(face, dlight_bits, dlights, dlight_surfaces_culled),
        SurfaceData::Grid(grid) => R_DlightGrid(grid, dlight_bits, dlights, dlight_surfaces_culled),
        SurfaceData::Triangles(tris) => R_DlightTrisurf(tris, dlight_bits),
        SurfaceData::Skip | SurfaceData::Flare(_) => 0,
    };

    if dlight_bits != 0 {
        *dlight_surfaces += 1;
    }

    dlight_bits
}

/// Raven `RE_GetBModelVerts` — the two largest-area faces of an inline
/// (brush) model's surfaces, picking whichever one faces the current view,
/// as its 4 corner verts.
///
/// PORT-NOTE: the oracle's `vec3_t normal` out-param is declared but never
/// written anywhere in the body — a dead out-param, dropped per
/// porting-rules §20 rather than returning caller-visible garbage.
///
/// PORT-NOTE: `dist`/`maxDist[2]` are C `int`, but `GetQuadArea` (already
/// ported, wave 0) returns `f32` — the oracle's `dist = GetQuadArea(...)`
/// assignment truncates toward zero on every call; `as i32` reproduces that
/// truncation rather than comparing as float (wave-0 ruling 12 applies to
/// double-promoted math, not this int-truncation quirk, so it is called out
/// separately here).
///
/// PORT-NOTE: `RenderModels::get_model` (the already-ported
/// `R_GetModelByHandle`) returns `&model_t`, whose `.bmodel: *mut bmodel_t`
/// is real on the client-rendering path; the walk goes through the tier-2
/// accessors `model_t::bmodel`, `bmodel_t::surfaces`, `msurface_t::face` and
/// `srfSurfaceFace_t::point` (§D11 quarantine). A null `bmodel`, a
/// `firstSurface` array shorter than `numSurfaces`, or a surface that is not
/// really an `SF_FACE` mirrors the oracle's own unchecked dereference — not a
/// divergence, Raven has no guard here either (porting-rules §19: this is
/// oracle UB, not a case this port need reproduce more safely, but neither
/// does it invent a defined-behavior guard the oracle lacks). This walk is
/// the reason `msurface_t`'s quarantine survives DEC-43: the world's own
/// surfaces are owned now, but a brush model still reaches its range through
/// `model_t::bmodel`, whose registration `R_LoadSubmodels` has not ported.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:653-744`
pub fn RE_GetBModelVerts(
    bmodel_index: qhandle_t,
    models: &RenderModels,
    frame: &FrameState,
) -> [vec3_t; 4] {
    let model = models.get_model(bmodel_index);

    let surfs = model.bmodel().surfaces();

    // Not sure if we really need to track the best two candidates
    let mut max_dist = [0i32; 2];
    let mut max_indx = [0usize; 2];

    for (i, surf) in surfs.iter().enumerate() {
        let face = surf.face();

        // It seems that the safest way to handle this is by finding the
        // area of the faces
        let dist = GetQuadArea(face.point(0), face.point(1), face.point(2), face.point(3)) as i32;

        // Check against the highest max
        if dist > max_dist[0] {
            // Shuffle our current maxes down
            max_dist[1] = max_dist[0];
            max_indx[1] = max_indx[0];

            max_dist[0] = dist;
            max_indx[0] = i;
        } else if dist >= max_dist[1] {
            // Check against the second highest max — just stomp the old
            max_dist[1] = dist;
            max_indx[1] = i;
        }
    }

    // Hopefully we've found two best case candidates. Now we should see
    // which of these faces the viewer
    let face0 = surfs[max_indx[0]].face();
    let dot1 = _DotProduct(face0.plane.normal, frame.refdef.view_axis[0]);

    let face1 = surfs[max_indx[1]].face();
    let dot2 = _DotProduct(face1.plane.normal, frame.refdef.view_axis[0]);

    let i = if dot2 < dot1 && dot2 < 0.0 {
        max_indx[1] // use the second face
    } else if dot1 < dot2 && dot1 < 0.0 {
        max_indx[0] // use the first face
    } else {
        // Possibly only have one face, so may as well use the first
        // face, which also should be the best one
        max_indx[0]
    };

    let face = surfs[i].face();
    [face.point(0), face.point(1), face.point(2), face.point(3)]
}

/// Raven `R_EvaluateWireframeSurf` — bake one BSP face surface into a new
/// automap wireframe outline; triangle-soup and bezier-grid surfaces are not
/// handled (dead `return;` arms in the oracle, dropped surface per
/// porting-rules §20).
///
/// PORT-NOTE: `face->numIndices` and the trailing index array Raven reaches
/// at the byte offset `face->ofsIndices` are the owned
/// `SurfaceFace::indices` `Vec` (DEC-43.1), so the count is its length;
/// `points && numPoints > 0` collapses to `numPoints > 0` — `points` is
/// `&face->points[0][0]`, the address of a field embedded in `face` itself,
/// never null.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:856-921`
pub fn R_EvaluateWireframeSurf(surf: &Surface, automap: &mut WireframeAutomap) {
    match &surf.data {
        SurfaceData::Face(face) => {
            let num_points = face.indices.len() as i32;

            if num_points > 0 {
                // we can add it, now go through the indices and add a point
                // for each
                let next_idx = R_GetNewWireframeMapSurf(automap);
                let mut points = Vec::with_capacity(num_points as usize);
                for &index in &face.indices {
                    points.push(WireframeSurfPoint {
                        xyz: face.points[index as usize].xyz,
                        ..Default::default()
                    });
                }
                automap.surfs[next_idx].points = points;
                automap.surfs[next_idx].num_points = num_points;
            }
        }
        // srfTriangles_t / srfGridMesh_t: not handled
        SurfaceData::Triangles(_) | SurfaceData::Grid(_) => {}
        // ...unknown type?
        SurfaceData::Skip | SurfaceData::Flare(_) => {}
    }
}

/// Raven `R_GetWireframeMapFromFile` — load a previously-written automap
/// wireframe (`blahblah.bla`, Raven's own placeholder filename) back into
/// the automap subsystem, `qboolean` collapsed to `bool` (§C7).
///
/// PORT-NOTE: the oracle reads each record through a `wireframeMapSurf_t *`
/// laid over the raw file bytes, so `surfs->numPoints` sits at offset 4 and
/// `&surfs->points` (the point payload) at offset 8 — past the struct's
/// leading `bool completelyTransparent` and its padding. Read back
/// symmetrically with `R_WriteWireframeMapToFile` using owned `Vec<u8>`
/// cursor arithmetic (porting-rules §9: manual alloc/free -> ownership)
/// instead of that raw cursor walk. See [`WIRE_SURF_HEADER_SIZE`].
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:1158-1202`
pub fn R_GetWireframeMapFromFile(
    view: &mut EngineHostView<'_>,
    automap: &mut WireframeAutomap,
) -> bool {
    let mut f: fileHandle_t = 0;
    let len = FS_FOpenFileRead(view, "blahblah.bla", &mut f as *mut fileHandle_t, false);
    if f == 0 || len <= 0 {
        // it doesn't exist
        return false;
    }

    let mut buf = vec![0u8; len as usize];
    FS_Read(view.common, buf.as_mut_ptr() as *mut (), len, f);

    let mut i: usize = 0;
    while i < buf.len() {
        // `completelyTransparent` (+ padding) at +0, `numPoints` at +4.
        let completely_transparent = buf[i] != 0;
        let num_points = i32::from_ne_bytes(buf[i + 4..i + 8].try_into().unwrap());
        let mut cursor = i + WIRE_SURF_HEADER_SIZE;

        // copy the surf data into the new surf
        let mut points = Vec::with_capacity(num_points.max(0) as usize);
        for _ in 0..num_points {
            points.push(WireframeSurfPoint::read_from(
                &buf[cursor..cursor + WireframeSurfPoint::WIRE_SIZE],
            ));
            cursor += WireframeSurfPoint::WIRE_SIZE;
        }

        let next_idx = R_GetNewWireframeMapSurf(automap);
        automap.surfs[next_idx].points = points;
        automap.surfs[next_idx].num_points = num_points;
        automap.surfs[next_idx].completely_transparent = completely_transparent;

        // the size of the point data, plus the record header
        let step_bytes =
            WireframeSurfPoint::WIRE_SIZE * num_points as usize + WIRE_SURF_HEADER_SIZE;
        i += step_bytes;
    }

    // it should end up being equal, if not something was wrong with this file.
    debug_assert_eq!(i, buf.len());

    FS_FCloseFile(view.common, f);

    true
}

/// Raven `R_DrawWireframeAutomap` — recompute per-point automap wireframe
/// alpha/color when the player's height has changed, then draw. The GL
/// emission itself (backdrop quad, polymode/blend state, per-triangle
/// color+vertex calls) is DEFERRED to R4 per this packet's `STATE HOMES`
/// row ("A frontend fn must not grow a GL dependency ... port the CPU
/// logic"); the CPU-side recompute (which is real, persisted state on
/// `WireframeAutomap`) is transcribed in full.
///
/// PORT-NOTE: `r_auto_map_integer` is `r_autoMap->integer` and `player_height`
/// is `g_playerHeight`, both threaded straight in as plain parameters rather
/// than reached from `RendererCvars`/a `FrameEvent` consumer — mirrors
/// `R_CullLocalBox`'s established `r_nocull_integer` threading (porting-rules
/// §4) and the R2 `## State ownership` row for `g_playerHeight` ("crosses in
/// `FrameData` as `FrameEvent::AutomapElevAdj`"): the not-yet-built R4
/// backend dispatch of `RC_AUTO_MAP` is what will extract this value from
/// the event stream and call in, exactly as it will thread `r_autoMap`'s
/// live cvar value — no render-side landing field for either exists yet to
/// reach for instead (state home marked, not populated, is an escalation
/// per the preamble, not an invention).
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:1257-1494`
pub fn R_DrawWireframeAutomap(
    automap: &mut WireframeAutomap,
    player_height: f32,
    r_auto_map_integer: i32,
) {
    if r_auto_map_integer == 0 {
        return;
    }

    if !automap.valid {
        // data is not valid, don't draw
        return;
    }

    // DEFERRED: R4 — qglDisable(GL_TEXTURE_2D); backdrop-quad draw
    // (qglPushMatrix/qglLoadIdentity/qglBegin(GL_QUADS)/4x qglVertex3f/
    // qglEnd/qglPopMatrix using backEnd.viewParms.zFar — `FrameState::view`
    // is still an empty `ViewParms` placeholder pending the `tr_main` wave);
    // GL_State(0) select (the `r_autoMapBackAlpha` alpha branch is oracle
    // `#if 0` dead code, dropped per porting-rules §20); qglColor4f black.
    // Source: oracle/codemp/renderer/tr_world.cpp:1295-1337

    // DEFERRED: R4 — GL_State(GLS_POLYMODE_LINE|GLS_SRCBLEND_SRC_ALPHA|
    // GLS_DSTBLEND_SRC_COLOR|GLS_DEPTHMASK_TRUE) vs GL_State(GLS_DEPTHMASK_TRUE)
    // line/fill-mode select, GL_Cull(CT_TWO_SIDED). The bit values are known
    // (`GLS_DEPTHMASK_TRUE` 0x100, `GLS_POLYMODE_LINE` 0x1000,
    // `GLS_SRCBLEND_SRC_ALPHA` 0x5, `GLS_DSTBLEND_SRC_COLOR` 0x30 —
    // oracle/codemp/renderer/tr_local.h:1652,1661,1669,1671); what is
    // deferred is the call itself: `GL_Cull`/`GL_State` are DEFERRED-R4 no-op
    // bodies (`GpuResources::gl_state` placeholder, DEC-37 A13.2), so the
    // mode choice has no observable effect yet either way.
    // Source: oracle/codemp/renderer/tr_world.cpp:1340-1352

    let mut s = 0usize;
    while s < automap.surfs.len() {
        // first, loop through and set the alpha on every point for this
        // surf. if the alpha ends up being completely transparent for every
        // point, we don't even need to draw it
        if player_height != automap.last_height || !automap.last_height_valid {
            let surf = &mut automap.surfs[s];
            surf.completely_transparent = true;

            for i in 0..surf.num_points as usize {
                // base the color on the elevation... for now, just check the
                // first point height
                let mut e = if surf.points[i].xyz[2] < player_height {
                    surf.points[i].xyz[2] - player_height
                } else {
                    player_height - surf.points[i].xyz[2]
                };
                if e < 0.0 {
                    e = -e;
                }

                if r_auto_map_integer != 2 {
                    // fill mode
                    if surf.points[i].xyz[2] > (player_height + 64.0) {
                        surf.points[i].alpha = 1.0;
                    } else {
                        surf.points[i].alpha = e / 256.0;
                    }
                } else {
                    // set alpha and color based on relative height of point
                    surf.points[i].alpha = e / 256.0;
                }
                e /= 512.0;

                // cap color
                if e > 1.0 {
                    e = 1.0;
                } else if e < 0.0 {
                    e = 0.0;
                }
                surf.points[i].color = [e, 1.0 - e, 0.0];

                // cap alpha
                if surf.points[i].alpha > 1.0 {
                    surf.points[i].alpha = 1.0;
                } else if surf.points[i].alpha < 0.0 {
                    surf.points[i].alpha = 0.0;
                }

                if surf.points[i].alpha != 1.0 {
                    // this point is not entirely alpha'd out, so still draw
                    // the surface
                    surf.completely_transparent = false;
                }
            }
        }

        if automap.surfs[s].completely_transparent {
            s += 1;
            continue;
        }

        // DEFERRED: R4 — qglBegin(GL_TRIANGLES)/per-point qglColor4f
        // (line-mode plain per-point color vs fill-mode plane-normal-derived
        // color — the planeNormal computation feeds only the deferred
        // qglColor4f call, no persisted effect, so it is not transcribed
        // separately) / qglVertex3f / qglEnd.
        // Source: oracle/codemp/renderer/tr_world.cpp:1436-1470

        s += 1;
    }

    automap.last_height = player_height;
    automap.last_height_valid = true;

    // DEFERRED: R4 — qglEnable(GL_TEXTURE_2D); qglColor4f(1,1,1,1) (restore
    // state after the automap draw).
    // Source: oracle/codemp/renderer/tr_world.cpp:1487-1491
}

/// Raven `R_MarkLeaves` — mark this frame's visible BSP leaves from the
/// current PVS cluster.
///
/// DEFERRED whole: every step past the entry cvar guard is blocked by
/// dependencies this wave cannot supply:
/// - `R_PointInLeaf`/`R_ClusterPVS` — this packet's own `RESOLVED CALL
///   SURFACE` lists both as "wave 0, already ported", but they are
///   themselves `// DEFERRED:` stubs above (no callable body) pending
///   `RenderAssets::world`'s owned node/vis arrays — a wave-planning defect
///   per the preamble ("every occurrence is a wave-planning defect fed back
///   into the manifest"), not something this wave can invent around;
/// - `tr.world->nodes`/`numnodes`/`numClusters` have no field on
///   `WorldAsset` yet at all (`crate::render_state::placeholders::
///   WorldAsset` — tier-2 transition audit, Group 1, `tr_bsp`/`tr_world`
///   wave) — there is no array to walk even if the two fns above existed;
/// - `tr.viewParms.pvsOrigin`/`tr.viewCluster`/`tr.refdef.areamaskModified`
///   have no `FrameState` landing field (`ViewParms`/`TrRefdef` are still
///   partial placeholders pending the `tr_main`/`tr_scene` waves).
///
/// `r_lockpvs_integer`/`r_novis_integer` mirror `R_CullLocalBox`'s
/// established cvar-value threading (porting-rules §4) rather than reaching
/// `RendererCvars` directly — kept on the signature even though the body is
/// empty, so the real body this wave cannot write slots in without a
/// signature change.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:1830-1900`
pub fn R_MarkLeaves(
    _r_lockpvs_integer: i32,
    _r_novis_integer: i32,
    _cvars: &mut RendererCvars,
    _assets: &mut RenderAssets,
) {
    // DEFERRED: R_MarkLeaves — see doc comment above.
    // Source: oracle/codemp/renderer/tr_world.cpp:1830-1900
}

/// Raven `MAX_ENTITIES` — local copy of the private const already ported at
/// `tr_main.rs` (not `pub` there); cited directly from the R2 design's
/// `backEndData_t` disposition entry ("entities[MAX_ENTITIES=2048]").
const MAX_ENTITIES: i32 = 2048;

/// Raven `TR_WORLDENT` — local copy of the private const already ported at
/// `tr_main.rs`/`tr_scene.rs` (neither `pub` there, so not reachable from
/// here); `MAX_ENTITIES - 1`, this file's own `MAX_ENTITIES` const above.
///
/// Source: `oracle/codemp/cgame/tr_types.h:15`
const TR_WORLDENT: i32 = MAX_ENTITIES - 1;

/// Raven `R_CullGrid` — frustum-cull a bezier-patch (grid) surface, tallying
/// the sphere/box cull-stat counters, `qboolean` collapsed to `bool` (§C7).
///
/// PORT-NOTE: `current_entity_num` is `tr.currentEntityNum` (STATE HOMES
/// SPLIT row → `RenderWorld::frame: FrameState`, still an empty placeholder
/// for this field), threaded straight in as a plain parameter — same
/// precedent as `R_GetPortalOrientations`'s `entity_num` (`tr_main.rs`).
///
/// PORT-NOTE: `r_nocurves_integer` (`r_nocurves->integer`, STATE HOMES row →
/// `RendererCvars`) and `r_nocull_integer`/`ori`/`frustum` (needed only to
/// satisfy `R_CullLocalPointAndRadius`/`R_CullPointAndRadius`/
/// `R_CullLocalBox`'s already-ported signatures) are threaded straight
/// through as plain parameters, mirroring `R_CullTriSurf`'s established
/// threading in this same file ("read through the live engine cvar table by
/// the caller, threaded in here rather than reached for").
///
/// PORT-NOTE: the six `tr.pc.c_*_cull_patch_*` counters (`BackEndCounters`,
/// R4 backend wave — still an empty placeholder) are threaded directly as
/// `&mut i32` outs, mirroring `R_DlightFace`/`R_DlightSurface`'s established
/// counter threading in this same file and `tr_ghoul2.rs`'s
/// `c_sphere_cull_md3_*` precedent.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:77-125`
#[allow(clippy::too_many_arguments)]
pub fn R_CullGrid(
    cv: &GridMesh,
    current_entity_num: i32,
    r_nocurves_integer: i32,
    r_nocull_integer: i32,
    ori: &orientationr_t,
    frustum: &[cplane_t; 4],
    c_sphere_cull_patch_out: &mut i32,
    c_sphere_cull_patch_clip: &mut i32,
    c_sphere_cull_patch_in: &mut i32,
    c_box_cull_patch_out: &mut i32,
    c_box_cull_patch_in: &mut i32,
    c_box_cull_patch_clip: &mut i32,
) -> bool {
    if r_nocurves_integer != 0 {
        return true;
    }

    let sphere_cull = if current_entity_num != TR_WORLDENT {
        R_CullLocalPointAndRadius(
            cv.local_origin,
            cv.mesh_radius,
            ori,
            r_nocull_integer,
            frustum,
        )
    } else {
        R_CullPointAndRadius(cv.local_origin, cv.mesh_radius, r_nocull_integer, frustum)
    };

    // check for trivial reject
    if sphere_cull == CULL_OUT {
        *c_sphere_cull_patch_out += 1;
        return true;
    } else if sphere_cull == CULL_CLIP {
        // check bounding box if necessary
        *c_sphere_cull_patch_clip += 1;

        let box_cull = R_CullLocalBox(cv.mesh_bounds, r_nocull_integer, ori, frustum);

        if box_cull == CULL_OUT {
            *c_box_cull_patch_out += 1;
            return true;
        } else if box_cull == CULL_IN {
            *c_box_cull_patch_in += 1;
        } else {
            *c_box_cull_patch_clip += 1;
        }
    } else {
        *c_sphere_cull_patch_in += 1;
    }

    false
}

// DEFERRED: R_RecursiveWireframeSurf — one carrier gap left: a per-node
// runtime visibility-frame field (`node->visframe`) to compare against
// `tr.visCount`. The tier-3 `Node` replacement (`tr_bsp::Node`, landed by the
// tr_bsp wave-1 node/leaf loader) carries only the fields
// `R_LoadNodesAndLeafs` parses from the BSP file and has no scratch field for
// this, and `FrameState` has no parallel per-node array either — a state-home
// omission per the preamble ("a state home this packet marks UNMAPPED is an
// ESCALATION, never an invention"), not something a wave can invent. The
// second gap this note used to name is CLOSED by DEC-43:
// `WorldAsset::mark_surfaces`' surface indices now resolve into
// `WorldAsset::surfaces`, and `R_EvaluateWireframeSurf` takes the resulting
// `&Surface` directly.
// Source: oracle/codemp/renderer/tr_world.cpp:990-1036

/// Raven `R_CullSurface` — decide whether a world surface is entirely
/// outside the current view: dispatch bezier-grid/triangle-soup surfaces to
/// their already-ported cullers, and for planar (`SF_FACE`) surfaces do
/// backface + (optional, cvar-gated) "roof" culling before the epsilon'd
/// front/back plane test. `qboolean` collapsed to `bool` (§C7).
///
/// PORT-NOTE: the oracle's `surfaceType_t *surface` parameter is the tagged
/// pointer `msurface_t::data` addresses; taking `surf: &Surface` and matching
/// on its owned [`SurfaceData`] (DEC-43.1) mirrors `R_DlightSurface`'s
/// treatment of the same tagged union in this file, rather than threading a
/// bare tag.
///
/// PORT-NOTE: `R_CullGrid`/`R_CullTriSurf` (this file, already ported)
/// needed their own globals threaded in as plain parameters once their
/// bodies landed; calling them from here means `R_CullSurface` inherits
/// that same threaded surface — `current_entity_num`/`r_nocurves_integer`
/// and the six `c_*_cull_patch_*` counters exist on this signature solely to
/// forward to `R_CullGrid`, not because `R_CullSurface` itself reads them.
///
/// PORT-NOTE: `r_nocull`/`r_facePlaneCull`/`r_cullRoofFaces`/
/// `r_roofCullCeilDist` (STATE HOMES → `RendererCvars`) and `tr.ori`
/// (STATE HOMES SPLIT → `FrameState::frame.ori`) are threaded straight
/// through as plain parameters, matching the cvar/`ori`/`frustum` threading
/// already established for `R_CullTriSurf`/`R_CullGrid` in this same file.
///
/// PORT-NOTE: the six `static` locals inside the (rarely-taken)
/// `r_cullRoofFaces` branch (`i`, `tr`, `basePoint`, `endPoint`, `nNormal`,
/// `v`) are always fully written before they are read on every path through
/// this block — Raven's `static` here is a stack-avoidance micro-optimization
/// for a "very slow, only for automap screenshots" branch (per the oracle's
/// own comment), not persisted cross-call state (three-kind rule: nothing
/// survives to the next call), so they become ordinary function-local
/// `let mut` bindings, never a carrier field.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:138-275`
#[allow(clippy::too_many_arguments)]
pub fn R_CullSurface(
    surf: &Surface,
    shader: &shader_t,
    view: &mut EngineHostView<'_>,
    ori: &orientationr_t,
    frustum: &[cplane_t; 4],
    current_entity_num: i32,
    r_nocull_integer: i32,
    r_nocurves_integer: i32,
    r_face_plane_cull_integer: i32,
    r_cull_roof_faces_integer: i32,
    r_roof_cull_ceil_dist_value: f32,
    c_sphere_cull_patch_out: &mut i32,
    c_sphere_cull_patch_clip: &mut i32,
    c_sphere_cull_patch_in: &mut i32,
    c_box_cull_patch_out: &mut i32,
    c_box_cull_patch_in: &mut i32,
    c_box_cull_patch_clip: &mut i32,
) -> bool {
    if r_nocull_integer != 0 {
        return false;
    }

    let face = match &surf.data {
        SurfaceData::Grid(grid) => {
            return R_CullGrid(
                grid,
                current_entity_num,
                r_nocurves_integer,
                r_nocull_integer,
                ori,
                frustum,
                c_sphere_cull_patch_out,
                c_sphere_cull_patch_clip,
                c_sphere_cull_patch_in,
                c_box_cull_patch_out,
                c_box_cull_patch_in,
                c_box_cull_patch_clip,
            );
        }
        SurfaceData::Triangles(tris) => {
            return R_CullTriSurf(tris, r_nocull_integer, ori, frustum);
        }
        SurfaceData::Face(face) => face,
        SurfaceData::Skip | SurfaceData::Flare(_) => return false,
    };

    if matches!(shader.cullType, cullType_t::CT_TWO_SIDED) {
        return false;
    }

    // face culling
    if r_face_plane_cull_integer == 0 {
        return false;
    }

    if r_cull_roof_faces_integer != 0 {
        // Very slow, but this is only intended for taking shots for automap images.
        if face.plane.normal[2] > 0.0 && !face.points.is_empty() {
            // it's facing up I guess

            // The fact that this point is in the middle of the array has no
            // relation to the orientation in the surface outline.
            let mut base_point = face.points[face.points.len() / 2].xyz;
            base_point[2] += 2.0;

            // the endpoint will be 8192 units from the chosen point in the
            // direction of the surface normal

            // just go straight up I guess, for now (slight hack)
            let mut n_normal: vec3_t = [0.0; 3];
            VectorSet(&mut n_normal, 0.0, 0.0, 1.0);
            let mut end_point: vec3_t = [0.0; 3];
            _VectorMA(base_point, 8192.0, n_normal, &mut end_point);

            let mut trace = trace_t::zeroed();
            // PORT-NOTE: the `*mut trace_t` out-param is the already-ported
            // engine `CM_BoxTrace` signature's own shape, not a new interior
            // type this file introduces; taking `&mut` as a pointer at the
            // call site needs no `unsafe`.
            CM_BoxTrace(
                view,
                &mut trace as *mut trace_t,
                base_point,
                end_point,
                vec3_origin,
                vec3_origin,
                0,
                CONTENTS_SOLID | CONTENTS_TERRAIN,
                0,
            );

            if trace.startsolid == 0
                && trace.allsolid == 0
                && (trace.fraction == 1.0 || (trace.surfaceFlags & SURF_NOIMPACT) != 0)
            {
                // either hit nothing or sky, so this surface is near the top
                // of the level I guess. Or the floor of a really tall room,
                // but if that's the case we're just screwed.
                let mut v: vec3_t = [0.0; 3];
                _VectorSubtract(base_point, trace.endpos, &mut v);
                if trace.fraction == 1.0 || VectorLength(v) < r_roof_cull_ceil_dist_value {
                    // ignore it if it's not close to the top, unless it just
                    // hit nothing

                    // Let's try to dig back into the brush based on the
                    // negative direction of the plane, and if we pop out on
                    // the other side we'll see if it's ground or not.
                    let mut i: i32 = 4;
                    n_normal = face.plane.normal;
                    VectorInverse(&mut n_normal);

                    while i < 4096 {
                        _VectorMA(base_point, i as f32, n_normal, &mut end_point);
                        CM_BoxTrace(
                            view,
                            &mut trace as *mut trace_t,
                            end_point,
                            end_point,
                            vec3_origin,
                            vec3_origin,
                            0,
                            CONTENTS_SOLID | CONTENTS_TERRAIN,
                            0,
                        );
                        if trace.startsolid == 0 && trace.allsolid == 0 && trace.fraction == 1.0 {
                            // in the clear
                            break;
                        }
                        i += 1;
                    }
                    if i < 4096 {
                        // Make sure we got into clearance
                        base_point = end_point;
                        base_point[2] -= 2.0;

                        // just go straight down I guess, for now (slight hack)
                        VectorSet(&mut n_normal, 0.0, 0.0, -1.0);
                        _VectorMA(base_point, 4096.0, n_normal, &mut end_point);

                        // trace a second time from the clear point in the
                        // inverse normal direction of the surface. If we hit
                        // something within a set amount of units, we will
                        // assume it's a bridge type object and leave it to be
                        // drawn. Otherwise we will assume it is a roof or
                        // other obstruction and cull it out.
                        CM_BoxTrace(
                            view,
                            &mut trace as *mut trace_t,
                            base_point,
                            end_point,
                            vec3_origin,
                            vec3_origin,
                            0,
                            CONTENTS_SOLID | CONTENTS_TERRAIN,
                            0,
                        );

                        if trace.startsolid == 0
                            && trace.allsolid == 0
                            && (trace.fraction != 1.0 && (trace.surfaceFlags & SURF_NOIMPACT) == 0)
                        {
                            // if we hit nothing or a noimpact going down then
                            // this is probably "ground".
                            _VectorSubtract(base_point, trace.endpos, &mut end_point);
                            if VectorLength(end_point) > r_roof_cull_ceil_dist_value {
                                // 128 (by default) is our maximum tolerance,
                                // above that will be removed
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    let d = _DotProduct(ori.viewOrigin, face.plane.normal);

    // don't cull exactly on the plane, because there are levels of rounding
    // through the BSP, ICD, and hardware that may cause pixel gaps if an
    // epsilon isn't allowed here
    if matches!(shader.cullType, cullType_t::CT_FRONT_SIDED) {
        if d < face.plane.dist - 8.0 {
            return true;
        }
    } else if d > face.plane.dist + 8.0 {
        return true;
    }

    false
}

// DEFERRED: R_GenerateWireframeMap — blocked by the same gap as
// `R_RecursiveWireframeSurf` above, and calls it as its only real work past
// the entry marking loop: (1) `tr.world->nodes[i].visframe = tr.visCount`
// needs a per-node runtime visibility-frame field the tier-3 `Node`
// replacement (`crate::tr_bsp::Node`) does not carry (only the fields
// `R_LoadNodesAndLeafs` parses from the BSP file) and `WorldAsset` has no
// parallel per-node scratch array either; (2) `R_RecursiveWireframeSurf`
// itself is a `// DEFERRED:` comment above with no callable body, for the
// same reason plus the `WorldAsset::mark_surfaces` index-resolution gap it
// documents. Both are state-home omissions per the preamble ("a state home
// this packet marks UNMAPPED is an ESCALATION, never an invention"), not
// something this wave can invent a carrier for — `memset(&g_autoMapFrame,
// 0, ...)` (the one step this wave *could* perform, as `WireframeAutomap`
// already exists) is not worth transcribing in isolation from the loop and
// recursive walk it exists to set up for.
// Source: oracle/codemp/renderer/tr_world.cpp:1039-1064

// ===== wave 4 =====

/// Raven `QSORT_ENTITYNUM_SHIFT` — restated from `tr_main.rs`'s own local
/// `const` (not `pub` there, so not reachable from this file); same value,
/// same rationale as `tr_scene.rs`'s own restatement of it.
///
/// Source: `oracle/codemp/renderer/tr_local.h:1226-1228`
const QSORT_ENTITYNUM_SHIFT: u32 = 7;

/// Raven `R_AddWorldSurface`.
///
/// STATE HOMES: `tr.viewCount` -> `view_count` (`FrameState::view_count`,
/// SPLIT `RenderAssets`/`FrameState`, R2 `## State ownership`); `tr.ori`/
/// `frustum`/the `r_*_integer` cvars/the `c_*_cull_patch_*` counters are the
/// same `RendererCvars`/`FrameState` carriers `R_CullSurface` already
/// threads (this fn is its caller, so it threads the identical set further
/// up rather than re-deriving them — porting-rules §4). `shader` is threaded
/// in explicitly rather than dereferenced from `surf->shader` (a raw
/// `*mut shader_s` tier-2 field with no quarantine accessor — interior-
/// safety law): `R_CullSurface`'s own already-ported signature establishes
/// this same convention (`surf`/`shader` as two separate params), so the
/// caller supplying both is the settled shape, not a new one. `dlightBits`
/// out-param-by-return stays a `mut` parameter per this file's own
/// `R_DlightFace`/`R_DlightGrid` precedent. `current_entity_num` (`tr.
/// currentEntityNum`) is threaded in rather than hardcoded to `TR_WORLDENT`
/// (unlike `tr_scene.rs`'s `R_AddPolygonSurfaces`): the oracle's caller for
/// this fn is not in this wave's packet and Raven's `R_AddWorldSurface` also
/// backs inline-brush-model surface addition (non-world entities), so a
/// fixed world-entity value would be speculative (porting-rules §A2 — no
/// guessing); `shifted_entity_num` is derived from it the same way
/// `R_AddPolygonSurfaces` derives its own. `rdf_nofog` (`tr.refdef.rdflags &
/// RDF_NOFOG`) is threaded in for the same reason — `TrRefdef` has no
/// `rdflags` field yet (same gap `tr_scene.rs`'s `R_AddPolygonSurfaces`
/// already flags), so the caller supplies the resolved bool.
///
/// PORT-NOTE: the oracle's `#ifdef _ALT_AUTOMAP_METHOD` branch (an alternate
/// immediate-mode-GL automap render path) is dropped as dead surface
/// (porting-rules §20) — no `#define _ALT_AUTOMAP_METHOD` exists anywhere in
/// the tree, and every other automap fn in this file
/// (`R_DrawWireframeAutomap`, `R_EvaluateWireframeSurf`) already lives under
/// the matching `#ifndef` arm. Only the compiled `#else` path (the plain
/// `R_AddDrawSurf` call) is transcribed. Raven hands `surf->data` — the
/// tagged-union pointer — straight to `R_AddDrawSurf`; under the owned world
/// that pointer is [`WorldSurfaceRef`], a `Copy` handle pairing the surface's
/// kind tag with its flat index into `WorldAsset::surfaces` (DEC-43.3). Hence
/// the extra `surf_index` parameter: it is the oracle's own
/// `worldData.surfaces` subscript, standing in for the pointer identity the
/// draw list cannot keep while the world walk mutates the same array
/// (porting-rules §B5).
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:408-555`
#[allow(clippy::too_many_arguments)]
pub fn R_AddWorldSurface(
    surf: &mut Surface,
    surf_index: u32,
    mut dlight_bits: i32,
    no_view_count: bool,
    view_count: i32,
    shader: &shader_t,
    current_entity_num: i32,
    r_nocull_integer: i32,
    r_nocurves_integer: i32,
    r_face_plane_cull_integer: i32,
    r_cull_roof_faces_integer: i32,
    r_roof_cull_ceil_dist_value: f32,
    rdf_nofog: bool,
    view: &mut EngineHostView<'_>,
    ori: &orientationr_t,
    frustum: &[cplane_t; 4],
    c_sphere_cull_patch_out: &mut i32,
    c_sphere_cull_patch_clip: &mut i32,
    c_sphere_cull_patch_in: &mut i32,
    c_box_cull_patch_out: &mut i32,
    c_box_cull_patch_in: &mut i32,
    c_box_cull_patch_clip: &mut i32,
    dlights: &[dlight_t],
    dlight_surfaces_culled: &mut u32,
    dlight_surfaces: &mut u32,
    draw_surfs: &mut Vec<DrawSurf<WorldSurfaceRef>>,
) {
    if !no_view_count {
        if surf.view_count == view_count {
            // already in this view, but lets make sure all the dlight bits are set
            match &mut surf.data {
                SurfaceData::Face(face) => face.dlight_bits |= dlight_bits,
                SurfaceData::Grid(grid) => grid.dlight_bits |= dlight_bits,
                SurfaceData::Triangles(tris) => tris.dlight_bits |= dlight_bits,
                SurfaceData::Skip | SurfaceData::Flare(_) => {}
            }
            return;
        }
        surf.view_count = view_count;
        // FIXME: bmodel fog?
    }

    // try to cull before dlighting or adding
    if R_CullSurface(
        surf,
        shader,
        view,
        ori,
        frustum,
        current_entity_num,
        r_nocull_integer,
        r_nocurves_integer,
        r_face_plane_cull_integer,
        r_cull_roof_faces_integer,
        r_roof_cull_ceil_dist_value,
        c_sphere_cull_patch_out,
        c_sphere_cull_patch_clip,
        c_sphere_cull_patch_in,
        c_box_cull_patch_out,
        c_box_cull_patch_in,
        c_box_cull_patch_clip,
    ) {
        return;
    }

    // check for dlighting
    if dlight_bits != 0 {
        dlight_bits = R_DlightSurface(
            surf,
            dlight_bits,
            dlights,
            dlight_surfaces_culled,
            dlight_surfaces,
        );
        dlight_bits = (dlight_bits != 0) as i32;
    }

    let fog_index = surf.fog_index;
    let shifted_entity_num = current_entity_num << QSORT_ENTITYNUM_SHIFT;
    R_AddDrawSurf(
        WorldSurfaceRef::of(surf, surf_index),
        shader.sortedIndex,
        shifted_entity_num,
        rdf_nofog,
        fog_index,
        dlight_bits,
        draw_surfs,
    );
}

/// Raven `R_InitializeWireframeAutomap`.
///
/// STATE HOMES: `r_autoMapDisable` -> `r_auto_map_disable_integer`
/// (`RendererCvars`, resolved by the caller — same "cvar ints threaded in
/// resolved, not reached for" convention `R_CullSurface` already
/// established in this file for `r_nocull_integer` and friends; a `None`
/// handle and a registered-but-zero cvar are observably identical for this
/// guard, so the collapse loses nothing). `tr.world`/`tr.world->nodes` ->
/// `assets.world`/`.nodes` (`RenderAssets`, SPLIT registries, R2 `## State
/// ownership`). `g_autoMapValid` -> `automap.valid` (`WireframeAutomap`,
/// already NAMED BY THIS FILE per DEC-37 A13.3, see the struct's own doc
/// comment above) — the qboolean out-of-band return is this fn's own return
/// value, matching the oracle's `return (qboolean)g_autoMapValid`.
///
/// DEFERRED: the `R_GenerateWireframeMap(tr.world->nodes)` call — that fn
/// has no callable body in this file (see the `// DEFERRED: R_GenerateWireframeMap`
/// note directly above), blocked on the identical `Node`/`WorldAsset`
/// per-node-visframe gap. The oracle's own observable contract does not
/// depend on what that call populates: `g_autoMapValid` is set whenever a
/// world with nodes is present, independent of the generate step's success,
/// so `automap.valid` is set here to match that contract exactly (no
/// speculative behavior invented — porting-rules §A2); the wireframe surface
/// list itself (`automap.surfs`) stays whatever `R_DestroyWireframeMap` just
/// cleared it to until the blocking wave lands and this call slots in
/// unchanged.
/// Source: `oracle/codemp/renderer/tr_world.cpp:1039-1064` (R_GenerateWireframeMap)
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:1205-1231`
pub fn R_InitializeWireframeAutomap(
    automap: &mut WireframeAutomap,
    assets: &RenderAssets,
    r_auto_map_disable_integer: i32,
) -> bool {
    if r_auto_map_disable_integer != 0 {
        return false;
    }

    if let Some(world) = assets.world.as_ref() {
        if !world.nodes.is_empty() {
            R_DestroyWireframeMap(automap);

            // DEFERRED: R_GenerateWireframeMap(tr.world->nodes) — see this
            // fn's own doc comment above.
            // Source: oracle/codemp/renderer/tr_world.cpp:1039-1064,1225

            automap.valid = true;
        }
    }

    automap.valid
}

// ===== wave 5 =====

/// Raven `R_AddBrushModelSurfaces` — cull an inline (brush) model against the
/// view frustum, dlight it, and hand its surfaces to the world's draw-surf
/// list.
///
/// PORT-NOTE: both `#ifdef VV_LIGHTING` branches (`R_SetupEntityLighting`/
/// `R_DlightBmodel` calls) are dead — no `#define VV_LIGHTING` exists
/// anywhere in the tree — matching this file's established treatment of
/// `_ALT_AUTOMAP_METHOD`; only the compiled `#else` calls are transcribed.
/// The `//rww` commented-out `com_RMG` branch below the `R_DlightBmodel`
/// call is plain dead prose in the oracle (not `#if 0`), nothing to port.
///
/// PORT-NOTE: `pModel`/`bmodel` are read through the already-established
/// tier-2 quarantine accessors `RenderModels::get_model`/`model_t::bmodel`
/// (§D11, `RE_GetBModelVerts` precedent, this file). `R_DlightBmodel`'s own
/// already-ported signature (wave 1) takes an owned `DlightBmodel` snapshot,
/// not `bmodel_t` directly; the snapshot is built here from `bmodel.bounds`
/// and a safe read of each surface's `surface_kind()` (no `unsafe` written
/// in this file — the interior-safety law).
///
/// DEFERRED: the closing `for (i = 0; i < bmodel->numSurfaces; i++)
/// R_AddWorldSurface(bmodel->firstSurface + i, tr.currentEntity->dlightBits,
/// qtrue)` loop — `R_AddWorldSurface` now takes `(&mut Surface, surf_index)`
/// into `WorldAsset::surfaces` (DEC-43.3), which is exactly the range
/// `BModel::first_surface`/`num_surfaces` describes; but this fn reaches its
/// brush model through the tier-2 `model_t::bmodel` raw pointer, and the
/// `R_LoadSubmodels` half that would register a `model_t` against a `BModel`
/// (and so let this walk address the owned array) is itself unported
/// (`tr_bsp.rs`, `//TODO: Port R_LoadSubmodels model_t registration`).
/// `R_AddWorldSurface` also needs `shader: &shader_t` per surface, resolved
/// from `Surface::shader`'s `ShaderHandle` — available once the walk is on
/// the owned array, unavailable through `msurface_t::shader` (a `*mut
/// shader_s` with no quarantine accessor; dereferencing it inline would be
/// new `unsafe`, banned here). `R_DlightBmodel`'s snapshot write-back
/// (`bmodel.surfaces` on the throwaway `DlightBmodel`, not the live
/// `msurface_t`s) is therefore also unobserved by this call — a narrow
/// fidelity gap only visible for a surface this loop never reaches anyway.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:570-611`
pub fn R_AddBrushModelSurfaces(
    ent: &mut RefEntity,
    models: &RenderModels,
    r_nocull_integer: i32,
    ori: &orientationr_t,
    frustum: &[cplane_t; 4],
    view: &mut EngineHostView<'_>,
    cvars: &RendererCvars,
    assets: &RenderAssets,
    frame: &mut FrameState,
    refdef_rdflags: i32,
    dlights: &mut [dlight_t],
) {
    let p_model = models.get_model(ent.h_model);
    let bmodel = p_model.bmodel();

    let clip = R_CullLocalBox(bmodel.bounds, r_nocull_integer, ori, frustum);
    if clip == CULL_OUT {
        return;
    }

    if p_model.bspInstance != 0 {
        // rwwRMG - added
        R_SetupEntityLighting(
            view.common,
            cvars,
            assets,
            frame,
            refdef_rdflags,
            dlights,
            ent,
        );
    }

    // rww - Take this into account later?
    let mut dlight_bmodel = DlightBmodel {
        bounds: bmodel.bounds,
        surfaces: bmodel
            .surfaces()
            .iter()
            .map(|s| DlightSurface {
                data: match s.surface_kind() {
                    SurfaceRef::Face(f) => DlightSurfaceData::Face {
                        dlightBits: f.dlightBits,
                    },
                    SurfaceRef::Grid(g) => DlightSurfaceData::Grid {
                        dlightBits: g.dlightBits,
                    },
                    SurfaceRef::Triangles(t) => DlightSurfaceData::Triangles {
                        dlightBits: t.dlightBits,
                    },
                    SurfaceRef::Other => DlightSurfaceData::Other,
                },
            })
            .collect(),
    };
    R_DlightBmodel(&mut dlight_bmodel, false, dlights, ori, frame);

    // DEFERRED: bmodel->numSurfaces R_AddWorldSurface loop — see this fn's
    // own doc comment above.
    // Source: oracle/codemp/renderer/tr_world.cpp:608-610
}

/// Raven `R_RecursiveWorldNode`.
///
/// DEFERRED: R_RecursiveWorldNode (whole fn) — the very first real check the oracle body makes —
/// `node->visframe != tr.visCount` (return early otherwise) — has no carrier
/// to read: the tier-3 `Node` replacement (`crate::tr_bsp::Node`, landed by
/// the tr_bsp wave-1 node/leaf loader) carries only the fields
/// `R_LoadNodesAndLeafs` parses from the BSP file (`parent`/`children`/
/// `contents`/`mins`/`maxs`/`plane`/`cluster`/`area`/`firstmarksurface`/
/// `nummarksurfaces`) and has no per-node runtime visited-scratch field, and
/// `FrameState` has no parallel per-node array either — the identical gap
/// this file's own `R_RecursiveWireframeSurf` note (above) already names for
/// a sibling fn. Every other branch (frustum planeBits culling against
/// `node->plane`, the front/back recursive descent, and the leaf branch's
/// `firstmarksurface`/`nummarksurfaces` walk) sits behind that gate in the
/// oracle, so skipping the check would invent which nodes get processed
/// (porting-rules §A2 — no speculative behavior), not faithfully transcribe
/// it. That gap is a state-home omission per the preamble ("a state home this
/// packet marks UNMAPPED is an ESCALATION, never an invention"), not
/// something a wave can invent a carrier for. The leaf branch's separate
/// blocker is CLOSED by DEC-43: `WorldAsset::mark_surfaces`' surface indices
/// now address `WorldAsset::surfaces`, and `R_AddWorldSurface` takes
/// `(&mut Surface, surf_index)` — destructuring `WorldAsset` once yields
/// `&nodes`/`&planes` alongside `&mut surfaces` for the whole walk.
///
/// STATE HOMES (for whichever wave lands the body): `r_nocull` ->
/// `RendererCvars` (DEC-37 A13.1); `tr` reads/writes SPLIT across
/// `RenderAssets` (`world.nodes`/`.planes`, registries) and `FrameState`
/// (`visCount`, `viewParms.frustum`/`.visBounds`, `pc.c_leafs`,
/// `refdef.dlights`/`.num_dlights`) per R2 `## State ownership`.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:1503-1663`
pub fn R_RecursiveWorldNode(
    _node_index: usize,
    _plane_bits: i32,
    _dlight_bits: i32,
    _r_nocull_integer: i32,
    _assets: &RenderAssets,
    _frame: &mut FrameState,
    _dlights: &[dlight_t],
) {
    todo!("Port R_RecursiveWorldNode — oracle/codemp/renderer/tr_world.cpp:1503-1663")
}

// ===== wave 6 =====

/// Raven `R_AddWorldSurfaces` — entry point for adding the world's
/// potentially-visible surfaces to this frame's draw-surf list: mark this
/// frame's visible leaves, clear the view's visibility bounds, clamp the
/// active dlight count, and recurse the BSP tree.
///
/// STATE HOMES: `r_drawworld` -> `RendererCvars` (DEC-37 A13.1), read
/// through the live engine cvar table via `common.cvar(cvars.r_drawworld)`
/// (this packet's STATE HOMES row). `tr.refdef.rdflags` is threaded in as
/// `refdef_rdflags: i32` — `TrRefdef` (`FrameState::refdef`) has no
/// `rdflags` field yet, the same gap `tr_scene.rs`/`tr_main.rs`/
/// `tr_terrain.rs`'s own `RDF_NOWORLDMODEL`/`RDF_NOFOG` PORT-NOTEs already
/// name; mirrors `tr_terrain.rs::R_AddTerrainSurfaces`'s identical
/// `refdef.rdflags & RDF_NOWORLDMODEL` guard threading. `tr.refdef.
/// num_dlights` is threaded in as `refdef_num_dlights: i32` for the same
/// reason (no `TrRefdef::num_dlights` field yet). `r_lockpvs`/`r_novis`/
/// `r_nocull` are resolved through `common.cvar(...)` immediately before
/// each already-ported callee that needs them (`R_MarkLeaves`/
/// `R_RecursiveWorldNode`'s own established "cvar ints threaded in
/// resolved, not reached for" signatures, this file).
///
/// PORT-NOTE: `tr.currentEntityNum = TR_WORLDENT; tr.shiftedEntityNum =
/// tr.currentEntityNum << QSORT_ENTITYNUM_SHIFT` has no carrier —
/// `FrameState` has no `current_entity_num`/`shifted_entity_num` fields yet
/// (the same gap `tr_scene.rs::R_AddPolygonSurfaces`'s own PORT-NOTE names)
/// — and unlike that fn, neither value is consumed anywhere else in this
/// body: `R_MarkLeaves`/`R_RecursiveWorldNode`'s own already-ported
/// signatures (this file, waves 0/5) take no `current_entity_num`/
/// `shifted_entity_num` parameter. Escalate a `FrameState` field-merge if a
/// later wave needs either value read back outside this call — nothing is
/// dropped here, the write is simply inert under the current call graph.
///
/// DEFERRED: `ClearBounds( tr.viewParms.visBounds[0], tr.viewParms.
/// visBounds[1] )` — `ViewParms` (`FrameState::view`) is still the empty
/// placeholder struct pending the `tr_main` wave
/// (`render_state/placeholders.rs`); `visBounds` has no landing field to
/// clear yet, so there is nothing to pass `ClearBounds` even if it were
/// called. `ClearBoundsMP` (`native_math::qmath`) is the confirmed MP-fork
/// pick for this packet's unresolved `ClearBounds` — mirroring the
/// established `ClearBoundsMP as ClearBounds` use already landed in
/// `tr_curve.rs`/`tr_marks.rs` — named here for whichever wave adds the
/// field.
/// Source: `oracle/codemp/renderer/tr_world.cpp:1950`
///
/// PORT-NOTE (ruling 19 — UB pick): `( 1 << tr.refdef.num_dlights ) - 1`
/// after the `> 32` clamp can leave `num_dlights == 32` (`MAX_DLIGHTS` is
/// 32 and the count is reachable, `oracle/codemp/cgame/tr_types.h:7`), and
/// `1 << 32` is undefined behavior for a 32-bit `int` in C. The defined
/// behavior picked here is the x86 one the retail binary actually produces:
/// `shl` masks its count modulo 32, so a width of 32 shifts by 0 and the
/// mask comes out `1 - 1 == 0` (no dlight bits), not "all bits set".
/// `wrapping_shl` applies exactly that mod-32 count masking, and
/// `wrapping_sub` keeps the `- 1` from panicking on the debug overflow
/// check. `tr.refdef.num_dlights = 32` itself (the write-back after the
/// clamp) has no carrier — `TrRefdef` has no `num_dlights` field yet (same
/// gap as `refdef_rdflags` above) — and is never re-read within this body,
/// so only the local clamped value is threaded into the mask below.
/// Source: `oracle/codemp/renderer/tr_world.cpp:1953-1957`
///
/// Panics via `R_RecursiveWorldNode`'s loud stub until its owning wave lands.
///
/// `tr.world->nodes` is the root of the node/leaf tree — index 0 into
/// `WorldAsset::nodes` (tier-2 transition audit, Group 1); `15` is Raven's
/// own literal `planeBits` (all 4 `FRUSTUM_PLANES` bits set, `R2-D7`(a)).
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:1934-1958`
pub fn R_AddWorldSurfaces(
    common: &Common,
    cvars: &mut RendererCvars,
    assets: &mut RenderAssets,
    frame: &mut FrameState,
    dlights: &[dlight_t],
    refdef_rdflags: i32,
    refdef_num_dlights: i32,
) {
    if common.cvar(cvars.r_drawworld).integer == 0 {
        return;
    }

    if refdef_rdflags & RDF_NOWORLDMODEL != 0 {
        return;
    }

    // PORT-NOTE: tr.currentEntityNum/tr.shiftedEntityNum — see this fn's own
    // doc comment above.
    // Source: oracle/codemp/renderer/tr_world.cpp:1943-1944

    // determine which leaves are in the PVS / areamask
    let r_lockpvs_integer = common.cvar(cvars.r_lockpvs).integer;
    let r_novis_integer = common.cvar(cvars.r_novis).integer;
    R_MarkLeaves(r_lockpvs_integer, r_novis_integer, cvars, assets);

    // DEFERRED: clear out the visible min/max (ClearBounds) — see this fn's
    // own doc comment above.
    // Source: oracle/codemp/renderer/tr_world.cpp:1950

    // perform frustum culling and add all the potentially visible surfaces
    let clamped_num_dlights = refdef_num_dlights.min(32);
    // DEFERRED: tr.refdef.num_dlights = 32 write-back — see this fn's own
    // doc comment above (ruling 19 — UB pick).
    // Source: oracle/codemp/renderer/tr_world.cpp:1953-1955
    let dlight_bits = 1i32
        .wrapping_shl(clamped_num_dlights as u32)
        .wrapping_sub(1);

    let r_nocull_integer = common.cvar(cvars.r_nocull).integer;
    let node_index = 0usize; // tr.world->nodes (the tree root)
    R_RecursiveWorldNode(
        node_index,
        15,
        dlight_bits,
        r_nocull_integer,
        assets,
        frame,
        dlights,
    );
}
