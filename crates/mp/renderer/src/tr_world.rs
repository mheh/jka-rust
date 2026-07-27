//! Raven `tr_world.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_world.cpp`

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_engine_qcommon::cm_load::{CM_LeafArea, CM_LeafCluster};
use mp_engine_qcommon::cm_test::{CM_ClusterPVSBits, CM_PointLeafnum};
use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::common::EngineHostView;
use mp_engine_qcommon::files_common::{
    FS_FCloseFile, FS_FOpenFileRead, FS_FOpenFileWrite, FS_Read, FS_Write,
};
use mp_qshared::shared::{cplane_t, qhandle_t, vec3_t};
use native_math::qmath::{
    _DotProduct, _VectorScale, _VectorSubtract, vec3_origin, CrossProduct, VectorCompare,
};
use native_types::fileHandle_t;

use crate::render_state::frame_data::FrameData;
use crate::render_state::frame_event::FrameEvent;
use crate::render_state::frame_state::FrameState;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::tr_local::dlight_s::dlight_t;
use crate::tr_local::msurface_s::{msurface_t, SurfaceRef, SurfaceRefMut};
use crate::tr_local::orientationr_t::orientationr_t;
use crate::tr_local::srf_grid_mesh_s::srfGridMesh_t;
use crate::tr_local::srf_surface_face_t::srfSurfaceFace_t;
use crate::tr_local::srf_triangles_t::srfTriangles_t;
use crate::tr_main::{R_CullLocalBox, CULL_OUT};
use crate::tr_model::render_models::RenderModels;

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
    face: &mut srfSurfaceFace_t,
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

    face.dlightBits = dlight_bits;
    dlight_bits
}

/// Raven `R_DlightGrid` — dlight culling for bezier-patch (grid) surfaces,
/// bounds-box test against each active dlight's radius.
///
/// PORT-NOTE: same threading rationale as `R_DlightFace`.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:303-329`
pub fn R_DlightGrid(
    grid: &mut srfGridMesh_t,
    mut dlight_bits: i32,
    dlights: &[dlight_t],
    dlight_surfaces_culled: &mut u32,
) -> i32 {
    for (i, dl) in dlights.iter().enumerate() {
        if dlight_bits & (1 << i) == 0 {
            continue;
        }
        if dl.origin[0] - dl.radius > grid.meshBounds[1][0]
            || dl.origin[0] + dl.radius < grid.meshBounds[0][0]
            || dl.origin[1] - dl.radius > grid.meshBounds[1][1]
            || dl.origin[1] + dl.radius < grid.meshBounds[0][1]
            || dl.origin[2] - dl.radius > grid.meshBounds[1][2]
            || dl.origin[2] + dl.radius < grid.meshBounds[0][2]
        {
            // dlight doesn't reach the bounds
            dlight_bits &= !(1 << i);
        }
    }

    if dlight_bits == 0 {
        *dlight_surfaces_culled += 1;
    }

    grid.dlightBits = dlight_bits;
    dlight_bits
}

/// Raven `R_DlightTrisurf` — dlight culling for triangle-soup surfaces is
/// unimplemented; the oracle's `#if 0` fallback body below the early return
/// never compiles and is dropped as dead surface (porting-rules §20).
///
/// Raven: FIXME: more dlight culling to trisurfs...
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:332-363`
pub fn R_DlightTrisurf(surf: &mut srfTriangles_t, dlight_bits: i32) -> i32 {
    surf.dlightBits = dlight_bits;
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

// DEFERRED: R_NodeHasOppositeFaces — needs the owned node/mark-surface arena
// (mnode_t.firstmarksurface, msurface_t.data union) that WorldAsset lands
// with at the tr_bsp wave; RenderAssets::world is still an empty placeholder
// here (tier-2 transition audit, Group 1) and walking the raw pointers today
// would adopt exactly the pattern the interior-safety law forbids for new
// code.
// Source: oracle/codemp/renderer/tr_world.cpp:927-986

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
    cv: &srfTriangles_t,
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
/// PORT-NOTE: the tagged-union dispatch over `msurface_t.data` goes through
/// `msurface_t::surface_kind_mut` (tier-2 quarantine, §D11); each arm mutates
/// its concrete surface exactly as the oracle does, so `surf` is `&mut`,
/// matching the oracle's own non-const `msurface_t *`.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:374-390`
pub fn R_DlightSurface(
    surf: &mut msurface_t,
    dlight_bits: i32,
    dlights: &[dlight_t],
    dlight_surfaces_culled: &mut u32,
    dlight_surfaces: &mut u32,
) -> i32 {
    let dlight_bits = match surf.surface_kind_mut() {
        SurfaceRefMut::Face(face) => {
            R_DlightFace(face, dlight_bits, dlights, dlight_surfaces_culled)
        }
        SurfaceRefMut::Grid(grid) => {
            R_DlightGrid(grid, dlight_bits, dlights, dlight_surfaces_culled)
        }
        SurfaceRefMut::Triangles(tris) => R_DlightTrisurf(tris, dlight_bits),
        SurfaceRefMut::Other => 0,
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
/// does it invent a defined-behavior guard the oracle lacks).
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
/// PORT-NOTE: `face->ofsIndices` is a byte offset from `face` to its
/// trailing index array (another flexible-array-member layout, read through
/// the tier-2 `srfSurfaceFace_t::indices` accessor); `points && numPoints
/// > 0` collapses to `numPoints > 0` — `points` is `&face->points[0][0]`,
/// the address of a field embedded in `face` itself, never null.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:856-921`
pub fn R_EvaluateWireframeSurf(surf: &msurface_t, automap: &mut WireframeAutomap) {
    match surf.surface_kind() {
        SurfaceRef::Face(face) => {
            let num_points = face.numIndices;

            if num_points > 0 {
                // we can add it, now go through the indices and add a point
                // for each
                let next_idx = R_GetNewWireframeMapSurf(automap);
                let mut points = Vec::with_capacity(num_points as usize);
                for &index in face.indices() {
                    points.push(WireframeSurfPoint {
                        xyz: face.point(index as usize),
                        ..Default::default()
                    });
                }
                automap.surfs[next_idx].points = points;
                automap.surfs[next_idx].num_points = num_points;
            }
        }
        // srfTriangles_t / srfGridMesh_t: not handled
        SurfaceRef::Triangles(_) | SurfaceRef::Grid(_) => {}
        // ...unknown type?
        SurfaceRef::Other => {}
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
