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
use mp_qshared::shared::{cplane_t, markFragment_t, vec3_t};

// `R_BoxSurfaces_r` walks the world's BSP node/surface tree, which per the
// tier-2 transition audit (Group 1: `mnode_t`/`msurface_t` rows) resolves to
// an index-linked node/surface arena owned by `tr_bsp`/`tr_world` and homed
// at `RenderAssets::world` (an `Option<WorldAsset>`, still an empty R3-wave
// placeholder in `render_state::placeholders`). This wave (`tr_marks`) may
// not touch `tr_bsp.rs`/`placeholders.rs` to grow that shape, so `MarkNode`/
// `MarkSurface`/`MarkSurfaceData` below are a scoped-local stand-in carrying
// only the fields this file's walk reads — owned (`Box` for the child links,
// never a raw pointer), per the interior-safety law. Reconciling this shape
// with `tr_bsp`'s own node/surface arena is an integration-time field-merge,
// not a second port of `tr.world`.
//
// `tr.viewCount` is threaded via [`MarkState::view_count`], this file's own
// counter. Raven shares one `tr.viewCount` between the world walk and the
// decal walk, and the two walks stamp different arrays, so W2-F4 gives the
// decal path its own generation. The stamps stay internally consistent inside
// each walk, which is all either one reads the counter for.

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
    /// `tr.viewCount` as this walk reads it. `R_MarkFragments` bumps it once
    /// per call, and `R_BoxSurfaces_r` compares each candidate surface to it.
    pub view_count: i32,
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

/// Scoped-local stand-in for Raven `mnode_t`'s node/leaf shape — see the
/// module-level note above. `contents == -1` marks an interior node (Raven
/// convention preserved); leaves carry their `mark_surfaces` directly rather
/// than a `firstmarksurface`/`nummarksurfaces` pointer+count pair into a
/// shared table.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:917-934`
pub struct MarkNode {
    /// -1 for nodes, to differentiate from leafs
    pub contents: i32,
    pub plane: cplane_t,
    pub children: [Option<Box<MarkNode>>; 2],
    pub mark_surfaces: Vec<MarkSurface>,
}

/// Scoped-local stand-in for the fields of Raven `msurface_t` (plus its
/// `shader_t`'s two flag words) that `R_BoxSurfaces_r` reads/writes — see the
/// module-level note above.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:872-878`
pub struct MarkSurface {
    /// if == the current view count, already added
    pub view_count: i32,
    pub surface_flags: i32,
    pub content_flags: i32,
    /// any of srf*_t
    pub data: MarkSurfaceData,
}

/// Owned replacement for `msurface_t.data`'s tagged `surfaceType_t *` union.
///
/// `R_BoxSurfaces_r` (wave 0) only ever needed `SF_FACE`'s plane to decide
/// view-count skips, so it defined `Face`/`Grid`/`Other` with the plane as
/// the sole payload. `R_MarkFragments` (this wave) walks the *geometry*
/// itself — the grid's triangulated quads and the face's index-addressed
/// point soup — which the skip-only shape can't carry. Both payload-bearing
/// variants are extended here with the same fields the tier-2 quarantine
/// accessors already expose for the real oracle types (`srfSurfaceFace_t
/// ::point`/`::indices`, `srfGridMesh_t::width`/`height`/`verts` —
/// `tr_local/{srf_surface_face_t,srf_grid_mesh_s}.rs`), so reconciling this
/// scoped-local stand-in with the real world arena at integration is a
/// straight field rename, not a reshape. All-owned per the interior-safety
/// law — no raw pointers.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:656-678,799-812`
#[derive(Clone)]
pub enum MarkSurfaceData {
    /// `srfSurfaceFace_t`'s plane plus its point/index soup — `points[i]` is
    /// the `srfSurfaceFace_t::point(i)` result (leading xyz triple of vertex
    /// `i`), `indexes` is `srfSurfaceFace_t::indices()`.
    Face {
        plane: cplane_t,
        points: Vec<vec3_t>,
        indexes: Vec<i32>,
    },
    /// `srfGridMesh_t`'s `width`/`height`/flattened `verts` (row-major,
    /// `width * height` entries) — only `xyz`/`normal` survive per vertex,
    /// the only fields `R_MarkFragments`'s grid walk reads.
    Grid {
        width: i32,
        height: i32,
        verts: Vec<MarkGridVert>,
    },
    Other,
}

/// Scoped-local stand-in for the `drawVert_t` fields `R_MarkFragments`'s grid
/// walk reads (`xyz`, `normal`) — see [`MarkSurfaceData::Grid`]'s doc. Not a
/// second port of `drawVert_t` (`crates/mp/engine/qcommon/src/qfiles/
/// draw_vert_t.rs`, which carries `st`/`lightmap`/`color` too and isn't
/// `Clone`); this file needs `Clone` (`MarkSurfaceData` derives it) and only
/// these two fields.
///
/// Type definition source: `oracle/codemp/qcommon/../qcommon/qfiles.h:514-520`
#[derive(Clone, Copy)]
pub struct MarkGridVert {
    pub xyz: vec3_t,
    pub normal: vec3_t,
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
/// Source: `oracle/codemp/renderer/tr_marks.cpp:116-178`
pub fn R_BoxSurfaces_r(
    node: &mut MarkNode,
    mins: vec3_t,
    maxs: vec3_t,
    list: &mut Vec<MarkSurfaceData>,
    listsize: usize,
    dir: vec3_t,
    mark: &MarkState,
) {
    if node.contents == -1 {
        let s = BoxOnPlaneSideRef(mins, maxs, &mut node.plane);
        if s == 1 {
            if let Some(child) = node.children[0].as_deref_mut() {
                R_BoxSurfaces_r(child, mins, maxs, list, listsize, dir, mark);
            }
        } else if s == 2 {
            if let Some(child) = node.children[1].as_deref_mut() {
                R_BoxSurfaces_r(child, mins, maxs, list, listsize, dir, mark);
            }
        } else {
            if let Some(child) = node.children[0].as_deref_mut() {
                R_BoxSurfaces_r(child, mins, maxs, list, listsize, dir, mark);
            }
            if let Some(child) = node.children[1].as_deref_mut() {
                R_BoxSurfaces_r(child, mins, maxs, list, listsize, dir, mark);
            }
        }
        return;
    }

    // add the individual surfaces
    for surf in node.mark_surfaces.iter_mut() {
        if list.len() >= listsize {
            break;
        }

        // check if the surface has NOIMPACT or NOMARKS set
        if (surf.surface_flags & (SURF_NOIMPACT | SURF_NOMARKS)) != 0
            || (surf.content_flags & CONTENTS_FOG) != 0
        {
            surf.view_count = mark.view_count;
        }
        // extra check for surfaces to avoid list overflows
        else if let MarkSurfaceData::Face { plane, .. } = &mut surf.data {
            // the face plane should go through the box. Raven hands
            // `BoxOnPlaneSide` the stored `srfSurfaceFace_t::plane` itself
            // (`tr_marks.cpp:153`), not a copy.
            let s = BoxOnPlaneSideRef(mins, maxs, plane);
            let normal = plane.normal;
            if s == 1 || s == 2 {
                surf.view_count = mark.view_count;
            } else if normal[0] * dir[0] + normal[1] * dir[1] + normal[2] * dir[2] > -0.5 {
                // don't add faces that make sharp angles with the projection direction
                surf.view_count = mark.view_count;
            }
        } else if !matches!(surf.data, MarkSurfaceData::Grid { .. }) {
            surf.view_count = mark.view_count;
        }
        // check the viewCount because the surface may have
        // already been added if it spans multiple leafs
        if surf.view_count != mark.view_count {
            surf.view_count = mark.view_count;
            list.push(surf.data.clone());
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
/// PORT-NOTE: `tr.world->nodes` (the BSP root `R_BoxSurfaces_r` walks) has no
/// live carrier yet — `RenderAssets::world` is still an empty R3-wave
/// placeholder this file may not grow (module-level note above) — so the
/// root is threaded in explicitly as `world_root`, the same shape
/// `R_BoxSurfaces_r` itself already takes (state threaded, not reached:
/// porting-rules §B4). `tr.viewCount` threads via `FrameState::view_count`
/// (`frame`), mutated here then read by `R_BoxSurfaces_r`.
///
/// Source: `oracle/codemp/renderer/tr_marks.cpp:245-448`
pub fn R_MarkFragments(
    points: &[vec3_t],
    projection: vec3_t,
    max_points: usize,
    point_buffer: &mut Vec<vec3_t>,
    max_fragments: usize,
    fragment_buffer: &mut Vec<markFragment_t>,
    world_root: &mut MarkNode,
    mark: &mut MarkState,
) -> i32 {
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

    let mut surfaces: Vec<MarkSurfaceData> = Vec::new();
    R_BoxSurfaces_r(
        world_root,
        mins,
        maxs,
        &mut surfaces,
        64,
        projection_dir,
        mark,
    );
    //assert(numsurfaces <= 64);
    //assert(numsurfaces != 64);

    for surf in &surfaces {
        match surf {
            MarkSurfaceData::Grid {
                width,
                height,
                verts,
            } => {
                let width = *width as usize;
                let height = *height as usize;
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
            MarkSurfaceData::Face {
                plane,
                points: face_points,
                indexes,
            } => {
                // check the normal of this face
                if _DotProduct(plane.normal, projection_dir) > -0.5 {
                    continue;
                }

                // §19: `chunks_exact` drops a malformed non-multiple-of-3 index
                // tail; the C `k += 3` loop over-reads past `numIndices` and
                // emits a garbage fragment (`tr_marks.cpp:413`) — defined
                // behavior picked over the UB.
                for chunk in indexes.chunks_exact(3) {
                    let clip_points: [vec3_t; 3] = [
                        marker_point(face_points[chunk[0] as usize], plane.normal),
                        marker_point(face_points[chunk[1] as usize], plane.normal),
                        marker_point(face_points[chunk[2] as usize], plane.normal),
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
            MarkSurfaceData::Other => {
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
