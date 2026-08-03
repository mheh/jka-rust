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
use mp_qshared::shared::q_math::BoxOnPlaneSideRef;
use mp_qshared::shared::{cplane_t, qhandle_t, vec3_t, CONTENTS_SOLID};
use native_math::qmath::{
    _DotProduct, _VectorScale, _VectorSubtract, vec3_origin, ClearBoundsMP, CrossProduct,
    VectorCompare,
};
use native_types::fileHandle_t;

use crate::render_state::frame_data::FrameData;
use crate::render_state::frame_event::FrameEvent;
use crate::render_state::frame_state::FrameState;
use crate::render_state::placeholders::RefEntity;
use crate::render_state::arena::Arena;
use crate::render_state::bmodel_table::BModelEntry;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::render_cvar_snapshot::RenderCvarSnapshot;
use crate::render_state::world_load_state::WorldLoadState;
use crate::render_state::world_walk_scratch::WorldWalkScratch;
use crate::tr_bsp::{Node, Surface, SurfaceData, SurfaceFace, SurfaceTriangles};
use crate::tr_curve::GridMesh;
use crate::tr_light::{
    DlightBmodel, DlightSurface, DlightSurfaceData, R_DlightBmodel, R_SetupEntityLighting,
};
use crate::render_state::shader_asset::ShaderAsset;
use crate::tr_local::dlight_s::dlight_t;
use crate::tr_local::orientationr_t::orientationr_t;
use crate::tr_shader::CullType;
use crate::tr_main::{
    DrawSurf, R_AddDrawSurf, R_CullLocalBox, R_CullLocalPointAndRadius, R_CullPointAndRadius,
    SurfaceGeometry, WorldSurfaceRef, CULL_CLIP, CULL_IN, CULL_OUT,
};
use crate::tr_model::render_models::RenderModels;
use crate::tr_public::ref_flags::{RDF_NOFOG, RDF_NOWORLDMODEL};

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
/// W2-F4: the `surf->dlightBits = dlightBits` write-back moved to
/// [`R_DlightSurface`], which stamps `WorldWalkScratch::surf_dlight_bits`.
/// The face itself is read-only here.
pub fn R_DlightFace(
    face: &SurfaceFace,
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

    dlight_bits
}

/// Raven `R_DlightGrid` — dlight culling for bezier-patch (grid) surfaces,
/// bounds-box test against each active dlight's radius.
///
/// PORT-NOTE: same threading rationale as `R_DlightFace`.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:303-329`
/// W2-F4: same write-back move as [`R_DlightFace`].
pub fn R_DlightGrid(
    grid: &GridMesh,
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

    dlight_bits
}

/// Raven `R_DlightTrisurf` — dlight culling for triangle-soup surfaces is
/// unimplemented; the oracle's `#if 0` fallback body below the early return
/// never compiles and is dropped as dead surface (porting-rules §20).
///
/// Raven: FIXME: more dlight culling to trisurfs...
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:332-363`
///
/// W2-F4: same write-back move as [`R_DlightFace`], which leaves this body
/// with nothing but the pass-through the oracle's early return produces.
pub fn R_DlightTrisurf(_surf: &SurfaceTriangles, dlight_bits: i32) -> i32 {
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

/// Raven `R_PointInLeaf` — walk the BSP node tree from the root to the leaf a
/// point falls in, returning that leaf's index into `world.nodes`.
///
/// PORT-NOTE: the oracle returns `mnode_t *`; under the owned node arena the
/// leaf is its `WorldAsset::nodes` index (`node->plane` is an index into
/// `world.planes`, `node->children[k]` an index into `world.nodes`). The
/// oracle's `if (!tr.world) Com_Error(...)` guard moves up to the caller,
/// which already resolves the loaded world before calling in.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:1672-1700`
fn R_PointInLeaf(nodes: &[Node], planes: &[cplane_t], p: vec3_t) -> usize {
    let mut node_index = 0usize;
    loop {
        let node = &nodes[node_index];
        if node.contents != -1 {
            break;
        }
        let plane = &planes[node.plane.expect("R_PointInLeaf hit a decision node with no plane")];
        let d = _DotProduct(p, plane.normal) - plane.dist;
        node_index = if d > 0.0 {
            node.children[0].expect("R_PointInLeaf hit a decision node with no front child")
        } else {
            node.children[1].expect("R_PointInLeaf hit a decision node with no back child")
        };
    }

    node_index
}

/// Raven `R_ClusterPVS` — the PVS bit row for one cluster, or the "everything
/// visible" `novis` row when the cluster or the vis data is invalid.
///
/// PORT-NOTE: the oracle's `!tr.world->vis` null test becomes `vis.is_empty()`
/// (the owned `Vec<u8>` is empty when the map has no vis lump). The `_XBOX`
/// `Decompress` arm is dropped as dead surface (porting-rules §20); only the
/// plain `vis + cluster * clusterBytes` row is returned.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:1707-1718`
fn R_ClusterPVS<'a>(
    vis: &'a [u8],
    novis: &'a [u8],
    num_clusters: i32,
    cluster_bytes: i32,
    cluster: i32,
) -> &'a [u8] {
    if vis.is_empty() || cluster < 0 || cluster >= num_clusters {
        return novis;
    }

    let start = (cluster * cluster_bytes) as usize;
    &vis[start..start + cluster_bytes as usize]
}

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
/// the owned [`SurfaceData`] (DEC-43.1). The `Skip`/`Flare` arms are the
/// oracle's `default:` (no dlight handler).
///
/// W2-F4: the three per-kind `dlightBits` write-backs land in
/// `scratch.surf_dlight_bits[surf_index]` here, so `surf` is read-only. The
/// `Skip`/`Flare` arms leave the stored mask alone, exactly as the oracle's
/// handler-free `default:` does.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:374-390`
pub fn R_DlightSurface(
    surf: &Surface,
    surf_index: u32,
    dlight_bits: i32,
    dlights: &[dlight_t],
    dlight_surfaces_culled: &mut u32,
    dlight_surfaces: &mut u32,
    scratch: &mut WorldWalkScratch,
) -> i32 {
    let dlight_bits = match &surf.data {
        SurfaceData::Face(face) => {
            let bits = R_DlightFace(face, dlight_bits, dlights, dlight_surfaces_culled);
            scratch.surf_dlight_bits[surf_index as usize] = bits;
            bits
        }
        SurfaceData::Grid(grid) => {
            let bits = R_DlightGrid(grid, dlight_bits, dlights, dlight_surfaces_culled);
            scratch.surf_dlight_bits[surf_index as usize] = bits;
            bits
        }
        SurfaceData::Triangles(tris) => {
            let bits = R_DlightTrisurf(tris, dlight_bits);
            scratch.surf_dlight_bits[surf_index as usize] = bits;
            bits
        }
        SurfaceData::Skip | SurfaceData::Flare(_) => 0,
    };

    if dlight_bits != 0 {
        *dlight_surfaces += 1;
    }

    dlight_bits
}

/// The submodel surface as a planar face with the four corner points the quad
/// math reads.
///
/// Raven casts every submodel surface to `srfSurfaceFace_t` without a tag
/// test, which is undefined behavior for any other surface kind.
/// We return `None` there instead (porting-rules §19).
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:672`
fn bmodel_quad_face(surf: &Surface) -> Option<&SurfaceFace> {
    match &surf.data {
        SurfaceData::Face(face) if face.points.len() >= 4 => Some(face),
        _ => None,
    }
}

/// Raven `RE_GetBModelVerts` — the two largest-area faces of an inline
/// (brush) model's surfaces, picking whichever one faces the current view,
/// as its 4 corner verts.
///
/// The handle resolves to a `WorldAsset::bmodels` row through
/// `RenderModels::bmodel_index`, and that row's range addresses the owned
/// `WorldAsset::surfaces` (the path `R_AddBrushModelSurfaces` uses).
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
/// Source: `oracle/codemp/renderer/tr_world.cpp:653-744`
pub fn RE_GetBModelVerts(
    bmodel_index: qhandle_t,
    models: &RenderModels,
    assets: &RenderAssets,
    frame: &FrameState,
) -> [vec3_t; 4] {
    let idx = models
        .bmodel_index(bmodel_index)
        .expect("RE_GetBModelVerts reached a non-brush model handle");
    let world = assets
        .world
        .as_ref()
        .expect("RE_GetBModelVerts needs the loaded world");
    let bmodel = &world.bmodels[idx];
    let first = bmodel.first_surface;
    let num = bmodel.num_surfaces.max(0) as usize;
    let surfs = &world.surfaces[first..first + num];

    // Not sure if we really need to track the best two candidates
    let mut max_dist = [0i32; 2];
    let mut max_indx = [0usize; 2];

    for (i, surf) in surfs.iter().enumerate() {
        let Some(face) = bmodel_quad_face(surf) else {
            continue;
        };

        // It seems that the safest way to handle this is by finding the
        // area of the faces
        let dist = GetQuadArea(
            face.points[0].xyz,
            face.points[1].xyz,
            face.points[2].xyz,
            face.points[3].xyz,
        ) as i32;

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
    let Some(face0) = surfs.get(max_indx[0]).and_then(bmodel_quad_face) else {
        // The submodel holds no usable face, so we return a zero quad.
        return [[0.0; 3]; 4];
    };
    let dot1 = _DotProduct(face0.plane.normal, frame.refdef.view_axis[0]);

    // The second candidate falls back to the first when its surface is no
    // face, and then the first face wins the test below.
    let face1 = surfs.get(max_indx[1]).and_then(bmodel_quad_face);
    let dot2 = match face1 {
        Some(face) => _DotProduct(face.plane.normal, frame.refdef.view_axis[0]),
        None => dot1,
    };

    // Raven's two `use the first face` arms collapse into one `else`.
    let face = if dot2 < dot1 && dot2 < 0.0 {
        // use the second face
        face1.unwrap_or(face0)
    } else {
        // Possibly only have one face, so may as well use the first
        // face, which also should be the best one
        face0
    };

    [
        face.points[0].xyz,
        face.points[1].xyz,
        face.points[2].xyz,
        face.points[3].xyz,
    ]
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
    // bodies (the render thread owns the GL binding cache, DEC-63.4), so the
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

/// Raven `R_MarkLeaves` — mark the leaves and nodes that are in the PVS for
/// the current cluster.
///
/// `r_lockpvs_integer`/`r_novis_integer` mirror `R_CullLocalBox`'s established
/// cvar-value threading (porting-rules §4) rather than reaching
/// `RendererCvars` directly. `tr.viewParms.pvsOrigin` -> `frame.view.
/// pvs_origin`, `tr.viewCluster` -> `frame.view_cluster`, `tr.visCount` ->
/// `frame.vis_count`, `tr.refdef.areamask`/`areamaskModified` ->
/// `frame.refdef.areamask`/`areamask_modified` (all grown by this wave);
/// `tr.world->nodes`/`numClusters`/`vis`/`novis` -> `assets.world`.
///
/// PORT-NOTE: the oracle's `R_PointInLeaf` returns `mnode_t *`; here it
/// returns the leaf's `world.nodes` index. Its `if (!tr.world) Com_Error`
/// guard becomes this fn's `.expect` on the world unwrap, matching
/// `R_AddBrushModelSurfaces`'s own world-unwrap panic in this file (the
/// faithful analogue of `Com_Error(ERR_DROP)`).
///
/// PORT-NOTE (carrier gap): the `r_showcluster` diagnostic (the `->modified`
/// remark trigger, the `Com_Printf("cluster:%i area:%i")`) is dropped. The
/// renderer's cvar view (`vm_cvar_t`: `value`/`integer` only) exposes neither
/// the `modified` flag nor the cvar name, the same UNMAPPED limit the rest of
/// the renderer works under. Dropping it changes no visibility output: the
/// `!r_showcluster->modified` term of the early-out guard only forced a
/// remark on toggle, and that remark reproduces the identical `visframe`
/// result for an unchanged cluster. Escalate a cvar-view `modified` field if a
/// later wave needs the diagnostic itself.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:1830-1900`
pub fn R_MarkLeaves(
    r_lockpvs_integer: i32,
    r_novis_integer: i32,
    assets: &RenderAssets,
    frame: &mut FrameState,
    scratch: &mut WorldWalkScratch,
) {
    // lockpvs lets designers walk around to determine the extent of the
    // current pvs
    if r_lockpvs_integer != 0 {
        return;
    }

    let world = assets
        .world
        .as_ref()
        .expect("R_MarkLeaves reached with no world loaded");

    // current viewcluster
    let leaf_index = R_PointInLeaf(&world.nodes, &world.planes, frame.view.pvs_origin);
    let cluster = world.nodes[leaf_index].cluster;

    // if the cluster is the same and the area visibility matrix hasn't
    // changed, we don't need to mark everything again
    if frame.view_cluster == cluster && !frame.refdef.areamask_modified {
        return;
    }

    // PORT-NOTE: r_showcluster remark/print dropped — see this fn's own doc
    // comment above (carrier gap).
    // Source: oracle/codemp/renderer/tr_world.cpp:1858-1864

    scratch.vis_count += 1;
    frame.view_cluster = cluster;

    if r_novis_integer != 0 || frame.view_cluster == -1 {
        for (i, node) in world.nodes.iter().enumerate() {
            if node.contents != CONTENTS_SOLID {
                scratch.node_visframe[i] = scratch.vis_count;
            }
        }
        return;
    }

    let vis = R_ClusterPVS(
        &world.vis,
        &world.novis,
        world.num_clusters,
        world.cluster_bytes,
        frame.view_cluster,
    );

    let vis_count = scratch.vis_count;
    let num_clusters = world.num_clusters;
    let areamask = frame.refdef.areamask;

    for i in 0..world.nodes.len() {
        let cluster = world.nodes[i].cluster;
        if cluster < 0 || cluster >= num_clusters {
            continue;
        }

        // check general pvs
        if vis[(cluster >> 3) as usize] & (1 << (cluster & 7)) == 0 {
            continue;
        }

        // check for door connection
        let area = world.nodes[i].area;
        if areamask[(area >> 3) as usize] & (1 << (area & 7)) != 0 {
            // not visible
            continue;
        }

        // mark the leaf and every parent up to an already-marked node
        let mut parent = Some(i);
        while let Some(p) = parent {
            if scratch.node_visframe[p] == vis_count {
                break;
            }
            scratch.node_visframe[p] = vis_count;
            parent = world.nodes[p].parent;
        }
    }
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
/// W2-F1: the four cvars arrive on the frame's [`RenderCvarSnapshot`] instead
/// of one resolved int each, so the walk reads no live table.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:138-275`
#[allow(clippy::too_many_arguments)]
pub fn R_CullSurface(
    surf: &Surface,
    shader: &ShaderAsset,
    warned_roof_cull: &mut bool,
    cvars: RenderCvarSnapshot,
    ori: &orientationr_t,
    frustum: &[cplane_t; 4],
    current_entity_num: i32,
    c_sphere_cull_patch_out: &mut i32,
    c_sphere_cull_patch_clip: &mut i32,
    c_sphere_cull_patch_in: &mut i32,
    c_box_cull_patch_out: &mut i32,
    c_box_cull_patch_in: &mut i32,
    c_box_cull_patch_clip: &mut i32,
) -> bool {
    if cvars.nocull != 0 {
        return false;
    }

    let face = match &surf.data {
        SurfaceData::Grid(grid) => {
            return R_CullGrid(
                grid,
                current_entity_num,
                cvars.nocurves,
                cvars.nocull,
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
            return R_CullTriSurf(tris, cvars.nocull, ori, frustum);
        }
        SurfaceData::Face(face) => face,
        SurfaceData::Skip | SurfaceData::Flare(_) => return false,
    };

    if matches!(shader.cull_type, CullType::TwoSided) {
        return false;
    }

    // face culling
    if cvars.face_plane_cull == 0 {
        return false;
    }

    //TODO: Port R_CullSurface roof-cull traces
    // Source: oracle/codemp/renderer/tr_world.cpp:1305-1420
    // W2-F2: the roof cull runs three `CM_BoxTrace` calls against the collision
    // world, which the render thread does not reach. The feature is inert here
    // and reports itself once. `r_cullRoofFaces` is a CVAR_CHEAT that exists to
    // take automap shots, so retail play never sets it.
    if cvars.cull_roof_faces != 0 && !*warned_roof_cull {
        *warned_roof_cull = true;
        eprintln!(
            "mp_renderer: r_cullRoofFaces is set, and the roof cull is inert on the render thread",
        );
    }

    let d = _DotProduct(ori.viewOrigin, face.plane.normal);

    // don't cull exactly on the plane, because there are levels of rounding
    // through the BSP, ICD, and hardware that may cause pixel gaps if an
    // epsilon isn't allowed here
    if matches!(shader.cull_type, CullType::FrontSided) {
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
/// in explicitly as the owned `&ShaderAsset` rather than dereferenced from
/// `surf->shader`: under the owned world `Surface::shader` is a
/// `ShaderHandle` into `RenderAssets::shaders`, so the caller resolves the
/// handle once and hands the borrow down (#51). `R_CullSurface`'s own
/// signature takes the same `&ShaderAsset`, so the caller supplying both
/// `surf`/`shader` is the settled shape. `dlightBits`
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
pub fn R_AddWorldSurface<'a>(
    surf: &Surface,
    surf_index: u32,
    mut dlight_bits: i32,
    no_view_count: bool,
    view_count: i32,
    scratch: &mut WorldWalkScratch,
    shader: &ShaderAsset,
    current_entity_num: i32,
    cvars: RenderCvarSnapshot,
    rdf_nofog: bool,
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
    draw_surfs: &mut Vec<DrawSurf<SurfaceGeometry<'a>>>,
) {
    if !no_view_count {
        if scratch.surf_view_count[surf_index as usize] == view_count {
            // already in this view, but lets make sure all the dlight bits are set
            // W2-F4: the three per-kind `|= dlightBits` merges are one merge on
            // the scratch mask. `Skip`/`Flare` carry no `dlightBits` field in
            // Raven, so those kinds keep the oracle's no-op.
            if !matches!(surf.data, SurfaceData::Skip | SurfaceData::Flare(_)) {
                scratch.surf_dlight_bits[surf_index as usize] |= dlight_bits;
            }
            return;
        }
        scratch.surf_view_count[surf_index as usize] = view_count;
        // FIXME: bmodel fog?
    }

    // try to cull before dlighting or adding
    if R_CullSurface(
        surf,
        shader,
        &mut scratch.warnings.roof_cull,
        cvars,
        ori,
        frustum,
        current_entity_num,
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
            surf_index,
            dlight_bits,
            dlights,
            dlight_surfaces_culled,
            dlight_surfaces,
            scratch,
        );
        dlight_bits = (dlight_bits != 0) as i32;
    }

    let fog_index = surf.fog_index;
    let shifted_entity_num = current_entity_num << QSORT_ENTITYNUM_SHIFT;
    R_AddDrawSurf(
        SurfaceGeometry::World(WorldSurfaceRef::of(surf, surf_index)),
        shader.sorted_index,
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
/// PORT-NOTE: `pModel->bmodel` is resolved through the owned path — the handle
/// maps to a `WorldAsset::bmodels` index (`RenderModels::bmodel_index`, filled
/// by `R_LoadSubmodels`), and both the cull bounds and the surface range read
/// off `assets.world`, never the retired `model_t::bmodel` raw pointer.
/// `pModel->bspInstance` still reads the `model_t` (`RenderModels::get_model`).
/// `R_DlightBmodel`'s own already-ported signature (wave 1) takes an owned
/// `DlightBmodel` snapshot; it is built here from the submodel bounds and each
/// surface's owned `dlight_bits` over the `[first, first + num)` range into
/// `WorldAsset::surfaces` (no `unsafe` — the interior-safety law).
///
/// The closing `for (i = 0; i < bmodel->numSurfaces; i++) R_AddWorldSurface(
/// bmodel->firstSurface + i, tr.currentEntity->dlightBits, qtrue)` loop is
/// live: it hands each submodel surface to `R_AddWorldSurface`, which appends
/// it through the `SurfaceGeometry::World` arm of the one frontend draw-surf
/// list (DEC-43.3). `tr.currentEntity->dlightBits` is `frame.current_entity.
/// dlight_bits`, the mask `R_DlightBmodel` just wrote; `noViewCount` is
/// `qtrue`. The eight `tr.pc.c_*` cull/dlight counters stay UNMAPPED
/// `frontEndCounters_t` scratch, owned here and threaded down exactly as
/// `R_AddWorldSurfaces` owns them.
/// Source: `oracle/codemp/renderer/tr_world.cpp:608-610`
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:570-611`
#[allow(clippy::too_many_arguments)]
pub fn R_AddBrushModelSurfaces<'a>(
    ent: &mut RefEntity,
    model: BModelEntry,
    cvars: RenderCvarSnapshot,
    ori: &orientationr_t,
    frustum: &[cplane_t; 4],
    assets: &RenderAssets,
    world_load: &WorldLoadState,
    frame: &mut FrameState,
    scratch: &mut WorldWalkScratch,
    refdef_rdflags: i32,
    current_entity_num: i32,
    dlights: &mut [dlight_t],
    draw_surfs: &mut Vec<DrawSurf<SurfaceGeometry<'a>>>,
) {
    // `pModel->bmodel` — the handle resolves to its `WorldAsset::bmodels` index
    // through the side map `R_LoadSubmodels` fills, and the owned submodel row
    // lives on the loaded world. W2-F8 crosses that index, and `bspInstance`
    // beside it, on the frame package's `BModelTable`, because the model
    // registry itself cannot reach the render thread.
    assert!(
        model.bmodel_index >= 0,
        "R_AddBrushModelSurfaces reached a non-brush model handle",
    );
    let world = assets
        .world
        .as_ref()
        .expect("R_AddBrushModelSurfaces needs the loaded world");
    let bmodel = &world.bmodels[model.bmodel_index as usize];

    let clip = R_CullLocalBox(bmodel.bounds, cvars.nocull, ori, frustum);
    if clip == CULL_OUT {
        return;
    }

    if model.bsp_instance != 0 {
        // rwwRMG - added
        R_SetupEntityLighting(cvars, assets, world_load, frame, refdef_rdflags, dlights, ent);
    }

    // rww - Take this into account later?
    let first = bmodel.first_surface;
    let num = bmodel.num_surfaces.max(0) as usize;
    // W2-F4: each surface's stored mask now comes from
    // `scratch.surf_dlight_bits` at its flat index. The snapshot the port hands
    // `R_DlightBmodel` stays a copy, so the fn's write-back is still discarded
    // and the loop below passes `entity_dlight_bits` down instead, unchanged
    // from before this ruling.
    let mut dlight_bmodel = DlightBmodel {
        bounds: bmodel.bounds,
        surfaces: world.surfaces[first..first + num]
            .iter()
            .enumerate()
            .map(|(i, s)| DlightSurface {
                data: match &s.data {
                    SurfaceData::Face(_) => DlightSurfaceData::Face {
                        dlightBits: scratch.surf_dlight_bits[first + i],
                    },
                    SurfaceData::Grid(_) => DlightSurfaceData::Grid {
                        dlightBits: scratch.surf_dlight_bits[first + i],
                    },
                    SurfaceData::Triangles(_) => DlightSurfaceData::Triangles {
                        dlightBits: scratch.surf_dlight_bits[first + i],
                    },
                    SurfaceData::Skip | SurfaceData::Flare(_) => DlightSurfaceData::Other,
                },
            })
            .collect(),
    };
    R_DlightBmodel(&mut dlight_bmodel, false, dlights, ori, frame);

    // `tr.currentEntity->dlightBits` — the mask `R_DlightBmodel` just wrote,
    // passed to every submodel surface below.
    let entity_dlight_bits = frame
        .current_entity
        .as_ref()
        .expect("R_AddBrushModelSurfaces: tr.currentEntity not set")
        .dlight_bits;
    let view_count = scratch.view_count;
    let rdf_nofog = refdef_rdflags & RDF_NOFOG != 0;

    // `frontEndCounters_t` scratch — UNMAPPED across the renderer, owned here
    // and threaded down (the `R_AddWorldSurfaces` precedent).
    let mut c_sphere_cull_patch_out = 0i32;
    let mut c_sphere_cull_patch_clip = 0i32;
    let mut c_sphere_cull_patch_in = 0i32;
    let mut c_box_cull_patch_out = 0i32;
    let mut c_box_cull_patch_in = 0i32;
    let mut c_box_cull_patch_clip = 0i32;
    let mut dlight_surfaces_culled = 0u32;
    let mut dlight_surfaces = 0u32;

    // Borrow the shader registry and the world surfaces disjointly, exactly as
    // `R_RecursiveWorldNode`'s own leaf loop does.
    let shaders = &assets.shaders;
    let world = assets
        .world
        .as_ref()
        .expect("R_AddBrushModelSurfaces needs the loaded world");

    for i in 0..num {
        let surf_index = (first + i) as u32;
        let shader_handle = world.surfaces[first + i].shader;
        let shader = shaders
            .get(shader_handle)
            .expect("R_AddWorldSurface reached a surface with an unresolved shader handle");
        R_AddWorldSurface(
            &world.surfaces[first + i],
            surf_index,
            entity_dlight_bits,
            true,
            view_count,
            scratch,
            shader,
            current_entity_num,
            cvars,
            rdf_nofog,
            ori,
            frustum,
            &mut c_sphere_cull_patch_out,
            &mut c_sphere_cull_patch_clip,
            &mut c_sphere_cull_patch_in,
            &mut c_box_cull_patch_out,
            &mut c_box_cull_patch_in,
            &mut c_box_cull_patch_clip,
            dlights,
            &mut dlight_surfaces_culled,
            &mut dlight_surfaces,
            draw_surfs,
        );
    }
}

/// Raven `R_RecursiveWorldNode` — walk the BSP tree from a node, frustum-cull
/// each subtree, and hand every potentially-visible leaf's mark surfaces to
/// `R_AddWorldSurface`.
///
/// STATE HOMES: `node->visframe`/`tr.visCount` -> `Node::visframe`/
/// `frame.vis_count` (both grown by this wave). `r_nocull` -> the resolved
/// `r_nocull_integer` (DEC-37 A13.1). `tr.world->nodes`/`.planes`/
/// `.marksurfaces`/`.surfaces` and `tr.sortedShaders`-adjacent shader lookup
/// are the `WorldAsset` arrays the caller destructures and threads in.
/// `tr.viewParms.frustum` -> `frame.view.frustum` (snapshot once at entry,
/// constant across the walk); `tr.viewParms.visBounds` -> `frame.view.
/// vis_bounds` (grown by this wave). `tr.refdef.dlights`/`num_dlights` -> the
/// `dlights` slice.
///
/// PORT-NOTE: the eight `tr.pc.c_*` counters (`c_leafs`, the six
/// `c_*_cull_patch_*`, `c_dlightSurfaces`/`c_dlightSurfacesCulled`) belong to
/// `frontEndCounters_t`, which the whole renderer leaves UNMAPPED (no
/// `FrameState`/placeholder home — `tr_cmds.rs`/`tr_mesh.rs`/`tr_ghoul2.rs`
/// each record this and thread the counters as bare `&mut i32`). This wave
/// keeps that convention: the counters thread through as `&mut i32`/`&mut u32`
/// and `R_AddWorldSurfaces` owns them as scratch. They are faithfully
/// computed; their only reader, `R_PerformanceCounters` (`tr_cmds`), is the
/// deferred R4 backend perf report.
///
/// PORT-NOTE: `node->children[k]`/`node->plane` are indices into the threaded
/// `nodes`/`planes` arrays; a decision node (`contents == -1`) always has both
/// set by `R_LoadNodesAndLeafs`, so the `.expect`s name an invariant Raven's
/// own unchecked pointer deref assumes. `node->mins`/`maxs` are stored `i32`
/// (the BSP int lump) and read as `f32`, reproducing the oracle's float
/// `mnode_t.mins` (an implicit int->float assignment there).
///
/// PORT-NOTE: the oracle's `#ifdef _ALT_AUTOMAP_METHOD` visframe-forcing arm
/// is dropped as dead surface (porting-rules §20) — no such `#define` exists,
/// matching this file's treatment of the same guard elsewhere.
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:1503-1663`
#[allow(clippy::too_many_arguments)]
pub fn R_RecursiveWorldNode<'a>(
    node_index: usize,
    plane_bits: i32,
    dlight_bits: i32,
    nodes: &[Node],
    planes: &[cplane_t],
    mark_surfaces: &[u32],
    surfaces: &[Surface],
    shaders: &Arena<ShaderAsset>,
    frame: &mut FrameState,
    scratch: &mut WorldWalkScratch,
    cvars: RenderCvarSnapshot,
    ori: &orientationr_t,
    dlights: &[dlight_t],
    view_count: i32,
    current_entity_num: i32,
    rdf_nofog: bool,
    c_leafs: &mut i32,
    c_sphere_cull_patch_out: &mut i32,
    c_sphere_cull_patch_clip: &mut i32,
    c_sphere_cull_patch_in: &mut i32,
    c_box_cull_patch_out: &mut i32,
    c_box_cull_patch_in: &mut i32,
    c_box_cull_patch_clip: &mut i32,
    dlight_surfaces_culled: &mut u32,
    dlight_surfaces: &mut u32,
    draw_surfs: &mut Vec<DrawSurf<SurfaceGeometry<'a>>>,
) {
    // The frustum planes are constant across the walk; snapshot once so the
    // leaf branch's `R_AddWorldSurface` call borrows a local, not `frame`.
    let frustum = frame.view.frustum;

    let mut node_index = node_index;
    let mut plane_bits = plane_bits;
    let mut dlight_bits = dlight_bits;

    loop {
        // if the node wasn't marked as potentially visible, exit
        if scratch.node_visframe[node_index] != scratch.vis_count {
            return;
        }

        // if the bounding volume is outside the frustum, nothing inside can be
        // visible
        if cvars.nocull != 1 {
            let mins = [
                nodes[node_index].mins[0] as f32,
                nodes[node_index].mins[1] as f32,
                nodes[node_index].mins[2] as f32,
            ];
            let maxs = [
                nodes[node_index].maxs[0] as f32,
                nodes[node_index].maxs[1] as f32,
                nodes[node_index].maxs[2] as f32,
            ];

            for bit in 0..4usize {
                if plane_bits & (1i32 << bit) == 0 {
                    continue;
                }
                // `BoxOnPlaneSide` only reads the plane, so a local copy of
                // the (already signbits-cached) frustum plane gives the same
                // result without a `frame` borrow.
                let mut plane = frustum[bit];
                let r = BoxOnPlaneSideRef(mins, maxs, &mut plane);
                if r == 2 {
                    return; // culled
                }
                if r == 1 {
                    // all descendants will also be in front
                    plane_bits &= !(1i32 << bit);
                }
            }
        }

        // leaf node reached
        if nodes[node_index].contents != -1 {
            break;
        }

        // node is just a decision point, so go down both sides. determine
        // which dlights are needed
        let mut new_dlights = [0i32; 2];
        if cvars.nocull != 2 {
            if dlight_bits != 0 {
                let plane = &planes[nodes[node_index]
                    .plane
                    .expect("R_RecursiveWorldNode hit a decision node with no plane")];
                for (i, dl) in dlights.iter().enumerate() {
                    if dlight_bits & (1 << i) == 0 {
                        continue;
                    }
                    let dist = _DotProduct(dl.origin, plane.normal) - plane.dist;
                    if dist > -dl.radius {
                        new_dlights[0] |= 1 << i;
                    }
                    if dist < dl.radius {
                        new_dlights[1] |= 1 << i;
                    }
                }
            }
        } else {
            new_dlights[0] = dlight_bits;
            new_dlights[1] = dlight_bits;
        }

        let child0 = nodes[node_index].children[0]
            .expect("R_RecursiveWorldNode hit a decision node with no front child");
        let child1 = nodes[node_index].children[1]
            .expect("R_RecursiveWorldNode hit a decision node with no back child");

        // recurse down the children, front side first
        R_RecursiveWorldNode(
            child0,
            plane_bits,
            new_dlights[0],
            nodes,
            planes,
            mark_surfaces,
            surfaces,
            shaders,
            frame,
            scratch,
            cvars,
            ori,
            dlights,
            view_count,
            current_entity_num,
            rdf_nofog,
            c_leafs,
            c_sphere_cull_patch_out,
            c_sphere_cull_patch_clip,
            c_sphere_cull_patch_in,
            c_box_cull_patch_out,
            c_box_cull_patch_in,
            c_box_cull_patch_clip,
            dlight_surfaces_culled,
            dlight_surfaces,
            draw_surfs,
        );

        // tail recurse the back side
        node_index = child1;
        dlight_bits = new_dlights[1];
    }

    // leaf node, so add mark surfaces
    *c_leafs += 1;

    // add to z buffer bounds
    let mins = nodes[node_index].mins;
    let maxs = nodes[node_index].maxs;
    let vis_bounds = &mut frame.view.vis_bounds;
    if (mins[0] as f32) < vis_bounds[0][0] {
        vis_bounds[0][0] = mins[0] as f32;
    }
    if (mins[1] as f32) < vis_bounds[0][1] {
        vis_bounds[0][1] = mins[1] as f32;
    }
    if (mins[2] as f32) < vis_bounds[0][2] {
        vis_bounds[0][2] = mins[2] as f32;
    }
    if (maxs[0] as f32) > vis_bounds[1][0] {
        vis_bounds[1][0] = maxs[0] as f32;
    }
    if (maxs[1] as f32) > vis_bounds[1][1] {
        vis_bounds[1][1] = maxs[1] as f32;
    }
    if (maxs[2] as f32) > vis_bounds[1][2] {
        vis_bounds[1][2] = maxs[2] as f32;
    }

    // add the individual surfaces
    let first = nodes[node_index].firstmarksurface;
    let count = nodes[node_index].nummarksurfaces;
    for k in 0..count {
        // the surface may have already been added if it spans multiple leafs;
        // `R_AddWorldSurface`'s viewCount guard handles that.
        let surf_index = mark_surfaces[first + k as usize];
        let shader_handle = surfaces[surf_index as usize].shader;
        let shader = shaders
            .get(shader_handle)
            .expect("R_AddWorldSurface reached a surface with an unresolved shader handle");
        R_AddWorldSurface(
            &surfaces[surf_index as usize],
            surf_index,
            dlight_bits,
            false,
            view_count,
            scratch,
            shader,
            current_entity_num,
            cvars,
            rdf_nofog,
            ori,
            &frustum,
            c_sphere_cull_patch_out,
            c_sphere_cull_patch_clip,
            c_sphere_cull_patch_in,
            c_box_cull_patch_out,
            c_box_cull_patch_in,
            c_box_cull_patch_clip,
            dlights,
            dlight_surfaces_culled,
            dlight_surfaces,
            draw_surfs,
        );
    }
}

// ===== wave 6 =====

/// Raven `R_AddWorldSurfaces` — entry point for adding the world's
/// potentially-visible surfaces to this frame's draw-surf list: mark this
/// frame's visible leaves, clear the view's visibility bounds, clamp the
/// active dlight count, and recurse the BSP tree.
///
/// STATE HOMES: `r_drawworld` -> `RendererCvars` (DEC-37 A13.1), read
/// through the live engine cvar table via `view.common.cvar(cvars.
/// r_drawworld)` (this packet's STATE HOMES row). `tr.refdef.rdflags` is
/// threaded in as `refdef_rdflags: i32` — `TrRefdef` (`FrameState::refdef`)
/// has no `rdflags` field yet, the same gap `tr_scene.rs`/`tr_main.rs`/
/// `tr_terrain.rs`'s own `RDF_NOWORLDMODEL`/`RDF_NOFOG` PORT-NOTEs already
/// name; mirrors `tr_terrain.rs::R_AddTerrainSurfaces`'s identical
/// `refdef.rdflags & RDF_NOWORLDMODEL` guard threading. `tr.refdef.
/// num_dlights` is threaded in as `refdef_num_dlights: i32` for the same
/// reason (no `TrRefdef::num_dlights` field yet). `r_lockpvs`/`r_novis`/
/// `r_nocull`/`r_nocurves`/`r_facePlaneCull`/`r_cullRoofFaces`/
/// `r_roofCullCeilDist` are resolved through `view.common.cvar(...)` and
/// threaded into `R_MarkLeaves`/`R_RecursiveWorldNode` (this file's own
/// "cvar ints threaded in resolved, not reached for" convention).
///
/// PORT-NOTE: `tr.currentEntityNum = TR_WORLDENT; tr.shiftedEntityNum =
/// tr.currentEntityNum << QSORT_ENTITYNUM_SHIFT` has no carrier —
/// `FrameState` has no `current_entity_num`/`shifted_entity_num` fields yet
/// (the same gap `tr_scene.rs::R_AddPolygonSurfaces`'s own PORT-NOTE names).
/// `current_entity_num` is the local `TR_WORLDENT`, threaded into
/// `R_RecursiveWorldNode` -> `R_AddWorldSurface`, which derives the shift
/// itself; the `tr.shiftedEntityNum` write is inert (nothing else in the
/// current call graph reads it). Escalate a `FrameState` field-merge if a
/// later wave needs either value read back outside this call.
///
/// PORT-NOTE: `draw_surfs` (the one `Vec<DrawSurf<SurfaceGeometry>>` the
/// frontend threads end to end, world surfaces landing through the
/// `SurfaceGeometry::World` arm), `view` (`EngineHostView`, for
/// `R_CullSurface`'s roof-cull `CM_BoxTrace`) and `ori` (`tr.ori`) are
/// threaded in from this fn's caller (`R_GenerateDrawSurfs`). The eight `tr.pc.c_*` `frontEndCounters_t` counters are owned
/// as scratch here and threaded down — that type stays UNMAPPED across the
/// renderer, and its only reader is the deferred R4 `R_PerformanceCounters`
/// (see `R_RecursiveWorldNode`'s own note).
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
/// `tr.world->nodes` is the root of the node/leaf tree — index 0 into
/// `WorldAsset::nodes` (tier-2 transition audit, Group 1); `15` is Raven's
/// own literal `planeBits` (all 4 `FRUSTUM_PLANES` bits set, `R2-D7`(a)).
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:1934-1958`
#[allow(clippy::too_many_arguments)]
pub fn R_AddWorldSurfaces<'a>(
    cvars: RenderCvarSnapshot,
    assets: &RenderAssets,
    frame: &mut FrameState,
    scratch: &mut WorldWalkScratch,
    ori: &orientationr_t,
    dlights: &[dlight_t],
    refdef_rdflags: i32,
    refdef_num_dlights: i32,
    draw_surfs: &mut Vec<DrawSurf<SurfaceGeometry<'a>>>,
) {
    if cvars.drawworld == 0 {
        return;
    }

    if refdef_rdflags & RDF_NOWORLDMODEL != 0 {
        return;
    }

    // PORT-NOTE: tr.currentEntityNum/tr.shiftedEntityNum — see this fn's own
    // doc comment above.
    // Source: oracle/codemp/renderer/tr_world.cpp:1943-1944
    let current_entity_num = TR_WORLDENT;

    // determine which leaves are in the PVS / areamask
    R_MarkLeaves(cvars.lockpvs, cvars.novis, assets, frame, scratch);

    // clear out the visible min/max
    let [vb_mins, vb_maxs] = &mut frame.view.vis_bounds;
    ClearBoundsMP(vb_mins, vb_maxs);

    // perform frustum culling and add all the potentially visible surfaces
    let clamped_num_dlights = refdef_num_dlights.min(32);
    // DEFERRED: tr.refdef.num_dlights = 32 write-back — see this fn's own
    // doc comment above (ruling 19 — UB pick).
    // Source: oracle/codemp/renderer/tr_world.cpp:1953-1955
    let dlight_bits = 1i32
        .wrapping_shl(clamped_num_dlights as u32)
        .wrapping_sub(1);

    let view_count = scratch.view_count;
    let rdf_nofog = refdef_rdflags & RDF_NOFOG != 0;

    // `frontEndCounters_t` scratch — UNMAPPED across the renderer, owned here
    // and threaded down (see `R_RecursiveWorldNode`'s own PORT-NOTE).
    let mut c_leafs = 0i32;
    let mut c_sphere_cull_patch_out = 0i32;
    let mut c_sphere_cull_patch_clip = 0i32;
    let mut c_sphere_cull_patch_in = 0i32;
    let mut c_box_cull_patch_out = 0i32;
    let mut c_box_cull_patch_in = 0i32;
    let mut c_box_cull_patch_clip = 0i32;
    let mut dlight_surfaces_culled = 0u32;
    let mut dlight_surfaces = 0u32;

    // `tr.sortedShaders`-adjacent shader registry sits beside `tr.world`;
    // borrow it disjointly from the world arrays the walk mutates.
    let shaders = &assets.shaders;
    let world = assets
        .world
        .as_ref()
        .expect("R_AddWorldSurfaces needs the loaded world");

    // `tr.world->nodes` (the tree root), `15` = all four frustum planeBits.
    R_RecursiveWorldNode(
        0,
        15,
        dlight_bits,
        &world.nodes,
        &world.planes,
        &world.mark_surfaces,
        &world.surfaces,
        shaders,
        frame,
        scratch,
        cvars,
        ori,
        dlights,
        view_count,
        current_entity_num,
        rdf_nofog,
        &mut c_leafs,
        &mut c_sphere_cull_patch_out,
        &mut c_sphere_cull_patch_clip,
        &mut c_sphere_cull_patch_in,
        &mut c_box_cull_patch_out,
        &mut c_box_cull_patch_in,
        &mut c_box_cull_patch_clip,
        &mut dlight_surfaces_culled,
        &mut dlight_surfaces,
        draw_surfs,
    );
}

#[cfg(test)]
mod tests {
    use super::{R_ClusterPVS, R_PointInLeaf};
    use crate::tr_bsp::Node;
    use mp_qshared::shared::cplane_t;

    // A leaf node with the given contents (0) — only the fields R_PointInLeaf
    // reads matter, the rest zero out.
    fn leaf(contents: i32) -> Node {
        Node {
            parent: None,
            children: [None, None],
            contents,
            mins: [0; 3],
            maxs: [0; 3],
            plane: None,
            cluster: 0,
            area: 0,
            firstmarksurface: 0,
            nummarksurfaces: 0,
        }
    }

    fn decision(plane: usize, front: usize, back: usize) -> Node {
        Node {
            parent: None,
            children: [Some(front), Some(back)],
            contents: -1,
            mins: [0; 3],
            maxs: [0; 3],
            plane: Some(plane),
            cluster: 0,
            area: 0,
            firstmarksurface: 0,
            nummarksurfaces: 0,
        }
    }

    #[test]
    fn point_in_leaf_follows_front_and_back_children() {
        // node0 splits on the +x plane through the origin; front child is the
        // leaf at index 1, back child the leaf at index 2.
        let nodes = [decision(0, 1, 2), leaf(0), leaf(0)];
        let planes = [cplane_t {
            normal: [1.0, 0.0, 0.0],
            dist: 0.0,
            r#type: 0,
            signbits: 0,
            pad: [0, 0],
        }];

        // d = DotProduct(p, normal) - dist > 0 takes the front child.
        assert_eq!(R_PointInLeaf(&nodes, &planes, [5.0, 0.0, 0.0]), 1);
        // d <= 0 takes the back child.
        assert_eq!(R_PointInLeaf(&nodes, &planes, [-5.0, 0.0, 0.0]), 2);
    }

    #[test]
    fn cluster_pvs_returns_the_cluster_row() {
        // three clusters, two bytes each; cluster 1's row is bytes [2..4).
        let vis = vec![0x01, 0x00, 0xAB, 0xCD, 0x00, 0x00];
        let novis = vec![0xFF, 0xFF];

        assert_eq!(R_ClusterPVS(&vis, &novis, 3, 2, 1), &[0xAB, 0xCD]);
        assert_eq!(R_ClusterPVS(&vis, &novis, 3, 2, 0), &[0x01, 0x00]);
    }

    #[test]
    fn cluster_pvs_falls_back_to_novis_when_invalid() {
        let vis = vec![0x01, 0x00, 0xAB, 0xCD];
        let novis = vec![0xFF, 0xFF];

        // out-of-range clusters and a missing vis lump all return novis.
        assert_eq!(R_ClusterPVS(&vis, &novis, 2, 2, -1), &novis[..]);
        assert_eq!(R_ClusterPVS(&vis, &novis, 2, 2, 2), &novis[..]);
        assert_eq!(R_ClusterPVS(&[], &novis, 2, 2, 0), &novis[..]);
    }
}
