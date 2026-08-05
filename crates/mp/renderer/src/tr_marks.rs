//! Raven `tr_marks.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_marks.cpp`

// Raven-named functions/types keep their original casing across this
// transcription, matching the rest of the renderer/engine crates.
#![allow(non_snake_case)]

use native_math::qmath::{
    _DotProduct, _VectorAdd, _VectorCopy, _VectorMA, _VectorSubtract, AddPointToBounds,
    ClearBoundsMP, CrossProduct, Q_rsqrt, VectorInverse, VectorNormalize2,
};

use mp_qshared::shared::q_math::BoxOnPlaneSideRef;
use mp_qshared::shared::surface_flags::{CONTENTS_FOG, SURF_NOIMPACT, SURF_NOMARKS};
use mp_qshared::shared::{markFragment_t, vec3_t};

use crate::render_state::arena::Arena;
use crate::render_state::placeholders::WorldAsset;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::shader_asset::ShaderAsset;
use crate::tr_bsp::SurfaceData;

// `R_BoxSurfaces_r` walks the world's BSP node/surface tree, which per the tier-2 transition audit (Group 1: `mnode_t`/`msurface_t` rows) resolves to the index-linked node/surface arena `tr_bsp` loads into `RenderAssets::world`.
// gh#31 step-006 completed that field-merge: the scoped-local `MarkNode`/`MarkSurface` stand-ins are gone, and both walks read `WorldAsset::nodes`, `surfaces`, `mark_surfaces`, and `planes` directly.
// A leaf's surfaces resolve through `WorldAsset::mark_surfaces`, and the collected list carries flat `WorldAsset::surfaces` indices in place of Raven's `surfaceType_t *` pointers.
//
// `tr.viewCount` is threaded via [`MarkState::view_count`], this file's own counter.
// Raven shares one `tr.viewCount` between the world walk and the decal walk, and the two walks stamp different arrays, so W2-F4 gives the decal path its own generation.
// The stamps stay internally consistent inside each walk, which is all either one reads the counter for.

/// Renderer-local `MAX_VERTS_ON_POLY` — the max vertex count
/// `R_ChopPolyBehindPlane` clips against (distinct from `mp_cgame`'s
/// mark-poly cap of the same name, `crates/mp/cgame/src/local/
/// mark_poly_s.rs`).
///
/// Source: `oracle/codemp/renderer/tr_marks.cpp:23-24`
/// (`vec3_t inPoints[MAX_VERTS_ON_POLY]`, resolved 64 per the packet's oracle
/// signature)
const MAX_VERTS_ON_POLY: usize = 64;

/// The decal walk's own generation counter.
///
/// Raven stamps `msurface_t::viewCount` from the one `tr.viewCount` in both
/// the world walk and this file's `R_BoxSurfaces_r`. The two walks stamp
/// separate arrays, so W2-F4 gives this one its own counter and leaves the
/// world walk's on `WorldWalkScratch`.
///
/// Source: `oracle/codemp/renderer/tr_local.h:1315`
#[derive(Default)]
pub struct MarkState {
    /// `tr.viewCount` as this walk reads it.
    /// `R_MarkFragments` bumps it once per call, and `R_BoxSurfaces_r` compares each candidate surface to it.
    pub view_count: i32,
    /// The per-surface stamp that replaces `msurface_t::viewCount` for this walk, indexed by the flat `WorldAsset::surfaces` index (W2-F4 pattern).
    /// `R_MarkFragments` resizes it with zeros when the world's surface count changes, so a map change resets every stamp.
    pub surf_view_count: Vec<i32>,
}

/// `SIDE_FRONT`/`SIDE_BACK`/`SIDE_ON` — planar-side classification, used here
/// purely as `counts`/`sides` array indices.
///
/// Source: `oracle/codemp/game/q_shared.h` (`SIDE_FRONT`/`SIDE_BACK`/`SIDE_ON`)
const SIDE_FRONT: usize = 0;
const SIDE_BACK: usize = 1;
const SIDE_ON: usize = 2;

/// Raven `R_ChopPolyBehindPlane` — clips `in_points` against a plane, keeping
/// the front (and on-plane) side. `numOutPoints`'s out-param becomes the
/// returned `Vec`'s length; `outPoints`'s fixed buffer becomes the `Vec`
/// itself.
///
/// Source: `oracle/codemp/renderer/tr_marks.cpp:23-108`
pub fn R_ChopPolyBehindPlane(
    in_points: &[vec3_t],
    normal: vec3_t,
    dist: f32,
    epsilon: f32,
) -> Vec<vec3_t> {
    let num_in_points = in_points.len();

    // don't clip if it might overflow
    if num_in_points >= MAX_VERTS_ON_POLY - 2 {
        return Vec::new();
    }

    let mut dists = vec![0.0f32; num_in_points + 1];
    let mut sides = vec![SIDE_ON; num_in_points + 1];
    let mut counts = [0usize; 3];

    // determine sides for each point
    for i in 0..num_in_points {
        let mut dot =
            in_points[i][0] * normal[0] + in_points[i][1] * normal[1] + in_points[i][2] * normal[2];
        dot -= dist;
        dists[i] = dot;
        sides[i] = if dot > epsilon {
            SIDE_FRONT
        } else if dot < -epsilon {
            SIDE_BACK
        } else {
            SIDE_ON
        };
        counts[sides[i]] += 1;
    }
    sides[num_in_points] = sides[0];
    dists[num_in_points] = dists[0];

    if counts[SIDE_FRONT] == 0 {
        return Vec::new();
    }
    if counts[SIDE_BACK] == 0 {
        return in_points.to_vec();
    }

    let mut out_points = Vec::with_capacity(num_in_points + 4);
    for i in 0..num_in_points {
        let p1 = in_points[i];

        if sides[i] == SIDE_ON {
            out_points.push(p1);
            continue;
        }

        if sides[i] == SIDE_FRONT {
            out_points.push(p1);
        }

        if sides[i + 1] == SIDE_ON || sides[i + 1] == sides[i] {
            continue;
        }

        // generate a split point
        let p2 = in_points[(i + 1) % num_in_points];

        let d = dists[i] - dists[i + 1];
        let dot = if d == 0.0 { 0.0 } else { dists[i] / d };

        // clip xyz
        let mut clip = [0.0f32; 3];
        for j in 0..3 {
            clip[j] = p1[j] + dot * (p2[j] - p1[j]);
        }
        out_points.push(clip);
    }

    out_points
}

/// Raven `R_BoxSurfaces_r`.
///
/// PORT-NOTE: Raven does the descent as "tail recursion in a loop" (an
/// explicit C micro-optimization noted in its own comment) with one child
/// explicitly recursed and the other looped into. Reshaped here as plain
/// recursion on both children — same node-visitation order and the same
/// `list`/`mark.view_count` accumulation, control-flow shape only
/// (porting-rules §10).
///
/// PORT-NOTE: Raven collects `surfaceType_t *` pointers into `tr.world`.
/// This walk collects flat `WorldAsset::surfaces` indices instead, which is this codebase's pointer replacement (DEC-43.1).
/// The `msurface_t::viewCount` stamp lives on `MarkState::surf_view_count` at the same index, because the loaded world is immutable (W2-F4).
///
/// Source: `oracle/codemp/renderer/tr_marks.cpp:116-178`
#[allow(clippy::too_many_arguments)]
pub fn R_BoxSurfaces_r(
    world: &WorldAsset,
    shaders: &Arena<ShaderAsset>,
    node_index: usize,
    mins: vec3_t,
    maxs: vec3_t,
    list: &mut Vec<u32>,
    listsize: usize,
    dir: vec3_t,
    mark: &mut MarkState,
) {
    let node = &world.nodes[node_index];

    if node.contents == -1 {
        // `BoxOnPlaneSideRef` wants `&mut cplane_t` and the loaded world is shared immutably, so the walk hands it a copy.
        // `BoxOnPlaneSide` only reads the plane, so the copy is behavior-identical (porting-rules §10).
        let mut plane = world.planes[node.plane.expect("decision node carries a plane index")];
        let s = BoxOnPlaneSideRef(mins, maxs, &mut plane);
        if s == 1 {
            if let Some(child) = node.children[0] {
                R_BoxSurfaces_r(world, shaders, child, mins, maxs, list, listsize, dir, mark);
            }
        } else if s == 2 {
            if let Some(child) = node.children[1] {
                R_BoxSurfaces_r(world, shaders, child, mins, maxs, list, listsize, dir, mark);
            }
        } else {
            if let Some(child) = node.children[0] {
                R_BoxSurfaces_r(world, shaders, child, mins, maxs, list, listsize, dir, mark);
            }
            if let Some(child) = node.children[1] {
                R_BoxSurfaces_r(world, shaders, child, mins, maxs, list, listsize, dir, mark);
            }
        }
        return;
    }

    // add the individual surfaces
    let view_count = mark.view_count;
    let first = node.firstmarksurface;
    let last = first + node.nummarksurfaces as usize;
    for &surface_index in &world.mark_surfaces[first..last] {
        if list.len() >= listsize {
            break;
        }

        let surf = &world.surfaces[surface_index as usize];
        let shader = shaders
            .get(surf.shader)
            .expect("a loaded world surface always resolves a registered shader");
        let stamp = &mut mark.surf_view_count[surface_index as usize];

        // check if the surface has NOIMPACT or NOMARKS set
        if (shader.surface_flags & (SURF_NOIMPACT | SURF_NOMARKS)) != 0
            || (shader.content_flags & CONTENTS_FOG) != 0
        {
            *stamp = view_count;
        }
        // extra check for surfaces to avoid list overflows
        else if let SurfaceData::Face(face) = &surf.data {
            // the face plane should go through the box.
            // Same immutable-world plane copy as the decision-node test above.
            let mut plane = face.plane;
            let s = BoxOnPlaneSideRef(mins, maxs, &mut plane);
            let normal = plane.normal;
            if s == 1 || s == 2 {
                *stamp = view_count;
            } else if normal[0] * dir[0] + normal[1] * dir[1] + normal[2] * dir[2] > -0.5 {
                // don't add faces that make sharp angles with the projection direction
                *stamp = view_count;
            }
        } else if !matches!(surf.data, SurfaceData::Grid(_)) {
            *stamp = view_count;
        }
        // check the viewCount because the surface may have
        // already been added if it spans multiple leafs
        if *stamp != view_count {
            *stamp = view_count;
            list.push(surface_index);
        }
    }
}

/// Raven `R_AddMarkFragments` — clips a candidate polygon (typically a
/// `srfSurfaceFace_t`/`srfGridMesh_t` fragment collected by `R_BoxSurfaces_r`)
/// against `normals`/`dists`, then appends the surviving points/fragment to
/// the caller's accumulator buffers.
///
/// PORT-NOTE: Raven's `pointBuffer`/`fragmentBuffer` are caller-owned raw
/// arrays with `returnedPoints`/`returnedFragments` as running write-offset
/// out-params; per the interior-safety law and the out-params→returns
/// dictionary entry, both become `&mut Vec<T>` and the running counts fold
/// into `.len()` — no separate offset out-param needed. Likewise `Com_Memcpy`
/// (the only listed engine callee) copied the final ping-pong buffer into
/// `pointBuffer`; with owned `Vec<vec3_t>` throughout, `extend_from_slice` is
/// the direct idiomatic equivalent — no raw-pointer copy exists to call
/// through.
///
/// PORT-NOTE: Raven's `maxFragments`/`mins`/`maxs` params are dropped — the
/// oracle body (`tr_marks.cpp:186-237`) never reads `maxFragments` (a dead
/// signature param, no guard to preserve), and `mins`/`maxs` are read only by
/// the bounding-box sanity check Raven itself left commented out
/// (`tr_marks.cpp:217-228`, dead code, not transcribed).
///
/// Source: `oracle/codemp/renderer/tr_marks.cpp:186-237`
pub fn R_AddMarkFragments(
    clip_points: &[vec3_t],
    normals: &[vec3_t],
    dists: &[f32],
    max_points: usize,
    point_buffer: &mut Vec<vec3_t>,
    fragment_buffer: &mut Vec<markFragment_t>,
) {
    // chop the surface by all the bounding planes of the to be projected polygon
    let mut points: Vec<vec3_t> = clip_points.to_vec();

    for i in 0..normals.len() {
        points = R_ChopPolyBehindPlane(&points, normals[i], dists[i], 0.5);
        if points.is_empty() {
            break;
        }
    }
    // completely clipped away?
    if points.is_empty() {
        return;
    }

    // add this fragment to the returned list
    if points.len() + point_buffer.len() > max_points {
        return; // not enough space for this polygon
    }

    fragment_buffer.push(markFragment_t {
        firstPoint: point_buffer.len() as i32,
        numPoints: points.len() as i32,
    });
    point_buffer.extend_from_slice(&points);
}

/// Renderer-local `MARKER_OFFSET` — the oracle's own trailing comment reads
/// `// 1`, but the shipped literal is `0`; every `VectorMA(point,
/// MARKER_OFFSET, normal, out)` call below is transcribed literally rather
/// than collapsed away, matching the oracle's arithmetic shape.
///
/// Source: `oracle/codemp/renderer/tr_marks.cpp:11`
const MARKER_OFFSET: f32 = 0.0;

/// Raven's repeated `VectorCopy(dv->xyz, out); VectorMA(out, MARKER_OFFSET,
/// dv->normal, out);` pair (`tr_marks.cpp:342-343` and five further sites) —
/// the in-place `VectorMA` reads and writes the same slot it was just copied
/// into, so the pair collapses to one `_VectorMA` call against the original
/// `xyz` (porting-rules §10: control-flow shape is free, behavior is fixed).
fn marker_point(xyz: vec3_t, normal: vec3_t) -> vec3_t {
    let mut out: vec3_t = [0.0; 3];
    _VectorMA(xyz, MARKER_OFFSET, normal, &mut out);
    out
}

/// Raven `R_MarkFragments`.
///
/// PORT-NOTE: out-params→returns (dictionary): `pointBuffer`/`fragmentBuffer`
/// become `&mut Vec<T>` (as `R_AddMarkFragments` already established);
/// `returnedPoints`/`returnedFragments` fold into `.len()`. `numPoints` folds
/// into `points.len()`. The oracle zeroes `returnedPoints`/`returnedFragments`
/// at entry (`tr_marks.cpp:307-308`), so `.len()` stands in for them only
/// because callers pass freshly-empty buffers — the precondition this fn
/// assumes.
///
/// PORT-NOTE: `tr.world->nodes` is `RenderAssets::world` here, so the walk starts at node 0 of that arena.
/// The oracle reaches `tr.world` as a global, and this fn takes `assets` instead (state threaded, not reached: porting-rules §B4).
/// A caller with no map loaded gets zero fragments, which is the degradation the `CG_CM_MARKFRAGMENTS` arm needs and not new oracle behavior.
/// `tr.viewCount` threads via `MarkState::view_count`, bumped here then read by `R_BoxSurfaces_r`.
///
/// Source: `oracle/codemp/renderer/tr_marks.cpp:245-448`
#[allow(clippy::too_many_arguments)]
pub fn R_MarkFragments(
    assets: &RenderAssets,
    mark: &mut MarkState,
    points: &[vec3_t],
    projection: vec3_t,
    max_points: usize,
    point_buffer: &mut Vec<vec3_t>,
    max_fragments: usize,
    fragment_buffer: &mut Vec<markFragment_t>,
) -> i32 {
    let Some(world) = assets.world.as_deref() else {
        return 0;
    };

    // The stamps are indexed by the flat surface index, so a map change re-sizes them and resets every generation.
    if mark.surf_view_count.len() != world.surfaces.len() {
        mark.surf_view_count.clear();
        mark.surf_view_count.resize(world.surfaces.len(), 0);
    }

    // increment view count for double check prevention
    mark.view_count += 1;

    let mut projection_dir: vec3_t = [0.0; 3];
    VectorNormalize2(projection, &mut projection_dir);

    // find all the brushes that are to be considered
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    ClearBoundsMP(&mut mins, &mut maxs);
    for p in points {
        AddPointToBounds(*p, &mut mins, &mut maxs);

        let mut temp: vec3_t = [0.0; 3];
        _VectorAdd(*p, projection, &mut temp);
        AddPointToBounds(temp, &mut mins, &mut maxs);

        // make sure we get all the leafs (also the one(s) in front of the hit surface)
        _VectorMA(*p, -20.0, projection_dir, &mut temp);
        AddPointToBounds(temp, &mut mins, &mut maxs);
    }

    let num_points = points.len().min(MAX_VERTS_ON_POLY);

    // create the bounding planes for the to be projected polygon
    let mut normals: Vec<vec3_t> = vec![[0.0; 3]; num_points + 2];
    let mut dists: Vec<f32> = vec![0.0; num_points + 2];
    for i in 0..num_points {
        let mut v1: vec3_t = [0.0; 3];
        let mut v2: vec3_t = [0.0; 3];
        _VectorSubtract(points[(i + 1) % num_points], points[i], &mut v1);
        _VectorAdd(points[i], projection, &mut v2);
        let v2_copy = v2;
        _VectorSubtract(points[i], v2_copy, &mut v2);
        CrossProduct(v1, v2, &mut normals[i]);
        // VectorNormalizeFast( normals[i] ) — inline header helper, no
        // existing equivalent; `Q_rsqrt`-based body matches the standard
        // `ilength = Q_rsqrt(DotProduct(v,v)); v *= ilength;` shape (same
        // pattern as `tr_shade_calc.rs::RB_CalcEnvironmentTexCoords`).
        let ilength = Q_rsqrt(_DotProduct(normals[i], normals[i]));
        normals[i][0] *= ilength;
        normals[i][1] *= ilength;
        normals[i][2] *= ilength;
        dists[i] = _DotProduct(normals[i], points[i]);
    }
    // add near and far clipping planes for projection
    _VectorCopy(projection_dir, &mut normals[num_points]);
    dists[num_points] = _DotProduct(normals[num_points], points[0]) - 32.0;
    _VectorCopy(projection_dir, &mut normals[num_points + 1]);
    VectorInverse(&mut normals[num_points + 1]);
    dists[num_points + 1] = _DotProduct(normals[num_points + 1], points[0]) - 20.0;
    // Raven: numPlanes = numPoints + 2; — folds into normals/dists.len()
    // (both sized exactly num_points + 2 above), no separate count needed.

    let mut surfaces: Vec<u32> = Vec::new();
    R_BoxSurfaces_r(
        world,
        &assets.shaders,
        0,
        mins,
        maxs,
        &mut surfaces,
        64,
        projection_dir,
        mark,
    );
    //assert(numsurfaces <= 64);
    //assert(numsurfaces != 64);

    for &surface_index in &surfaces {
        match &world.surfaces[surface_index as usize].data {
            SurfaceData::Grid(grid) => {
                let verts = &grid.verts;
                let width = grid.width as usize;
                let height = grid.height as usize;
                for m in 0..height.saturating_sub(1) {
                    for n in 0..width.saturating_sub(1) {
                        // We triangulate the grid and chop all triangles within
                        // the bounding planes of the to be projected polygon.
                        // LOD is not taken into account, not such a big deal though.
                        //
                        // It's probably much nicer to chop the grid itself and deal
                        // with this grid as a normal SF_GRID surface so LOD will
                        // be applied. However the LOD of that chopped grid must
                        // be synced with the LOD of the original curve.
                        // One way to do this; the chopped grid shares vertices with
                        // the original curve. When LOD is applied to the original
                        // curve the unused vertices are flagged. Now the chopped curve
                        // should skip the flagged vertices. This still leaves the
                        // problems with the vertices at the chopped grid edges.
                        //
                        // To avoid issues when LOD applied to "hollow curves" (like
                        // the ones around many jump pads) we now just add a 2 unit
                        // offset to the triangle vertices.
                        // The offset is added in the vertex normal vector direction
                        // so all triangles will still fit together.
                        // The 2 unit offset should avoid pretty much all LOD problems.
                        let base = m * width + n;
                        let dv0 = verts[base];
                        let dv_w = verts[base + width];
                        let dv1 = verts[base + 1];
                        let dv_w1 = verts[base + width + 1];

                        // first triangle: dv0, dv[width], dv1
                        let mut clip_points = [
                            marker_point(dv0.xyz, dv0.normal),
                            marker_point(dv_w.xyz, dv_w.normal),
                            marker_point(dv1.xyz, dv1.normal),
                        ];
                        // check the normal of this triangle
                        let mut v1: vec3_t = [0.0; 3];
                        let mut v2: vec3_t = [0.0; 3];
                        _VectorSubtract(clip_points[0], clip_points[1], &mut v1);
                        _VectorSubtract(clip_points[2], clip_points[1], &mut v2);
                        let mut normal: vec3_t = [0.0; 3];
                        CrossProduct(v1, v2, &mut normal);
                        let ilength = Q_rsqrt(_DotProduct(normal, normal));
                        normal[0] *= ilength;
                        normal[1] *= ilength;
                        normal[2] *= ilength;
                        if _DotProduct(normal, projection_dir) < -0.1 {
                            // add the fragments of this triangle
                            R_AddMarkFragments(
                                &clip_points,
                                &normals,
                                &dists,
                                max_points,
                                point_buffer,
                                fragment_buffer,
                            );
                            if fragment_buffer.len() == max_fragments {
                                // not enough space for more fragments
                                return fragment_buffer.len() as i32;
                            }
                        }

                        // second triangle: dv1, dv[width], dv[width+1]
                        clip_points = [
                            marker_point(dv1.xyz, dv1.normal),
                            marker_point(dv_w.xyz, dv_w.normal),
                            marker_point(dv_w1.xyz, dv_w1.normal),
                        ];
                        // check the normal of this triangle
                        _VectorSubtract(clip_points[0], clip_points[1], &mut v1);
                        _VectorSubtract(clip_points[2], clip_points[1], &mut v2);
                        CrossProduct(v1, v2, &mut normal);
                        let ilength = Q_rsqrt(_DotProduct(normal, normal));
                        normal[0] *= ilength;
                        normal[1] *= ilength;
                        normal[2] *= ilength;
                        if _DotProduct(normal, projection_dir) < -0.05 {
                            // add the fragments of this triangle
                            R_AddMarkFragments(
                                &clip_points,
                                &normals,
                                &dists,
                                max_points,
                                point_buffer,
                                fragment_buffer,
                            );
                            if fragment_buffer.len() == max_fragments {
                                // not enough space for more fragments
                                return fragment_buffer.len() as i32;
                            }
                        }
                    }
                }
            }
            SurfaceData::Face(face) => {
                let plane = &face.plane;
                // check the normal of this face
                if _DotProduct(plane.normal, projection_dir) > -0.5 {
                    continue;
                }

                // §19: `chunks_exact` drops a malformed non-multiple-of-3 index
                // tail; the C `k += 3` loop over-reads past `numIndices` and
                // emits a garbage fragment (`tr_marks.cpp:413`) — defined
                // behavior picked over the UB.
                for chunk in face.indices.chunks_exact(3) {
                    let clip_points: [vec3_t; 3] = [
                        marker_point(face.points[chunk[0] as usize].xyz, plane.normal),
                        marker_point(face.points[chunk[1] as usize].xyz, plane.normal),
                        marker_point(face.points[chunk[2] as usize].xyz, plane.normal),
                    ];
                    // add the fragments of this face
                    R_AddMarkFragments(
                        &clip_points,
                        &normals,
                        &dists,
                        max_points,
                        point_buffer,
                        fragment_buffer,
                    );
                    if fragment_buffer.len() == max_fragments {
                        // not enough space for more fragments
                        return fragment_buffer.len() as i32;
                    }
                }
            }
            SurfaceData::Skip | SurfaceData::Triangles(_) | SurfaceData::Flare(_) => {
                // ignore all other world surfaces
                // might be cool to also project polygons on a triangle soup
                // however this will probably create huge amounts of extra polys
                // even more than the projection onto curves
                continue;
            }
        }
    }
    fragment_buffer.len() as i32
}
