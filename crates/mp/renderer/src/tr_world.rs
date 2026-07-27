//! Raven `tr_world.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_world.cpp`

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_engine_qcommon::cm_load::{CM_LeafArea, CM_LeafCluster};
use mp_engine_qcommon::cm_test::{CM_ClusterPVSBits, CM_PointLeafnum};
use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::common::EngineHostView;
use mp_engine_qcommon::files_common::{FS_FCloseFile, FS_FOpenFileWrite, FS_Write};
use mp_qshared::shared::vec3_t;
use native_math::qmath::{
    _DotProduct, _VectorScale, _VectorSubtract, vec3_origin, CrossProduct, VectorCompare,
};

use crate::render_state::frame_data::FrameData;
use crate::render_state::frame_event::FrameEvent;
use crate::tr_local::dlight_s::dlight_t;
use crate::tr_local::srf_grid_mesh_s::srfGridMesh_t;
use crate::tr_local::srf_surface_face_t::srfSurfaceFace_t;
use crate::tr_local::srf_triangles_t::srfTriangles_t;

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
}

impl Default for WireframeAutomap {
    fn default() -> Self {
        WireframeAutomap {
            surfs: Vec::new(),
            next_free: 0,
            valid: false,
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
}

/// Raven `wireframeMapSurf_t` — one automap wireframe outline. The oracle's
/// `wireframeSurfPoint_t *points` heap array becomes an owned `Vec`
/// (porting-rules §C9) and its `next` list link is dissolved into
/// `WireframeAutomap::surfs`.
///
/// Type definition source: `oracle/codemp/renderer/tr_world.cpp:767-775`
pub struct WireframeMapSurf {
    pub num_points: i32,
    pub points: Vec<WireframeSurfPoint>,
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
/// never emits the point data it sizes for. Ported as the evident intent,
/// `numPoints` followed by that surface's points (porting-rules §19).
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
            // memory for each point, then memory for numPoints
            WireframeSurfPoint::WIRE_SIZE * surf.num_points as usize + core::mem::size_of::<i32>()
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
