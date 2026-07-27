//! Raven `tr_marks.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_marks.cpp`

// Raven-named functions/types keep their original casing across this
// transcription, matching the rest of the renderer/engine crates.
#![allow(non_snake_case)]

use mp_qshared::shared::q_math::BoxOnPlaneSideRef;
use mp_qshared::shared::surface_flags::{CONTENTS_FOG, SURF_NOIMPACT, SURF_NOMARKS};
use mp_qshared::shared::{cplane_t, markFragment_t, vec3_t};

use crate::render_state::frame_state::FrameState;

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
// `tr.viewCount` (part of `trGlobals_t` frontend scratch, `## State
// ownership`'s "frontend scratch/counters" row) is threaded via
// `FrameState::view_count` — not yet a field on that struct (owned by
// `render_state/frame_state.rs`, also out of this file's reach); expected to
// land there at integration, same as any other cross-file state field.

/// Renderer-local `MAX_VERTS_ON_POLY` — the max vertex count
/// `R_ChopPolyBehindPlane` clips against (distinct from `mp_cgame`'s
/// mark-poly cap of the same name, `crates/mp/cgame/src/local/
/// mark_poly_s.rs`).
///
/// Source: `oracle/codemp/renderer/tr_marks.cpp:23-24`
/// (`vec3_t inPoints[MAX_VERTS_ON_POLY]`, resolved 64 per the packet's oracle
/// signature)
const MAX_VERTS_ON_POLY: usize = 64;

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

/// Owned replacement for `msurface_t.data`'s tagged `surfaceType_t *` union —
/// only the variants `R_BoxSurfaces_r` inspects (`SF_FACE`'s embedded plane,
/// and `SF_GRID`); every other kind collapses to `Other`.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:656-678,799-812`
#[derive(Clone)]
pub enum MarkSurfaceData {
    Face { plane: cplane_t },
    Grid,
    Other,
}

/// Raven `R_BoxSurfaces_r`.
///
/// PORT-NOTE: Raven does the descent as "tail recursion in a loop" (an
/// explicit C micro-optimization noted in its own comment) with one child
/// explicitly recursed and the other looped into. Reshaped here as plain
/// recursion on both children — same node-visitation order and the same
/// `list`/`frame.view_count` accumulation, control-flow shape only
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
    frame: &FrameState,
) {
    if node.contents == -1 {
        let s = BoxOnPlaneSideRef(mins, maxs, &mut node.plane);
        if s == 1 {
            if let Some(child) = node.children[0].as_deref_mut() {
                R_BoxSurfaces_r(child, mins, maxs, list, listsize, dir, frame);
            }
        } else if s == 2 {
            if let Some(child) = node.children[1].as_deref_mut() {
                R_BoxSurfaces_r(child, mins, maxs, list, listsize, dir, frame);
            }
        } else {
            if let Some(child) = node.children[0].as_deref_mut() {
                R_BoxSurfaces_r(child, mins, maxs, list, listsize, dir, frame);
            }
            if let Some(child) = node.children[1].as_deref_mut() {
                R_BoxSurfaces_r(child, mins, maxs, list, listsize, dir, frame);
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
            surf.view_count = frame.view_count;
        }
        // extra check for surfaces to avoid list overflows
        else if let MarkSurfaceData::Face { plane } = &mut surf.data {
            // the face plane should go through the box. Raven hands
            // `BoxOnPlaneSide` the stored `srfSurfaceFace_t::plane` itself
            // (`tr_marks.cpp:153`), not a copy.
            let s = BoxOnPlaneSideRef(mins, maxs, plane);
            let normal = plane.normal;
            if s == 1 || s == 2 {
                surf.view_count = frame.view_count;
            } else if normal[0] * dir[0] + normal[1] * dir[1] + normal[2] * dir[2] > -0.5 {
                // don't add faces that make sharp angles with the projection direction
                surf.view_count = frame.view_count;
            }
        } else if !matches!(surf.data, MarkSurfaceData::Grid) {
            surf.view_count = frame.view_count;
        }
        // check the viewCount because the surface may have
        // already been added if it spans multiple leafs
        if surf.view_count != frame.view_count {
            surf.view_count = frame.view_count;
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
