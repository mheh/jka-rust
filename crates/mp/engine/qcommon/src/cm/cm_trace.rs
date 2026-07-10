//! `cm_trace.cpp` terrain-collision free functions.
//!
//! These are the two `cm_trace.cpp` helpers `CCMLandScape::PatchCollide`
//! (`cm_terrain.rs`) calls per frame — decls `cm_public.h:56-57`. They belong
//! to the `cm` C-track qcommon packet (the wider clipmap-trace lane), NOT the
//! RMG/terrain doc (`docs/subsystems/rmg-terrain.md` scopes them out — it froze
//! `PatchCollide` LIVE, ruling 28/RMG-D1, but its two callees land with the
//! wave-0–4 clipmap-trace lane, which has not landed in this tree yet — see the
//! sibling `//TODO: Port CollisionWorld fields` in `collision_world.rs`, the
//! same not-yet-landed lane).
//!
//! `calc_extents` is a leaf that ports faithfully now; `handle_patch_collision`
//! needs `CM_TraceThroughBrush`/`CM_GenericBoxCollide` + the `CmPatch`
//! collision-brush accessors, none of which have landed, so it is a marked
//! placeholder. The `PatchCollide` path is currently unreachable (its only
//! caller, `CollisionWorld::terrain_patch_collide`, is itself reached only from
//! the not-yet-landed `cm-trace`/`cm-test` packets), so the placeholder is
//! inert until that lane lands and takes ownership.
//!
//! Source: `oracle/codemp/qcommon/cm_trace.cpp`; `cm_public.h:56-57`

use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::{vec3_t, vec3pair_t};

use crate::cm::trace_work_s::traceWork_s;
use crate::cm_patch::CmPatch;

/// `CM_CalcExtents(start, end, tw, bounds)` — the AABB of a swept segment,
/// expanding each axis by the trace box's `size[0]`/`size[1]` corners.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:1550-1569`
pub fn calc_extents(start: vec3_t, end: vec3_t, tw: &traceWork_s, bounds: &mut vec3pair_t) {
    for i in 0..3 {
        if start[i] < end[i] {
            bounds[0][i] = start[i] + tw.size[0][i];
            bounds[1][i] = end[i] + tw.size[1][i];
        } else {
            bounds[0][i] = end[i] + tw.size[0][i];
            bounds[1][i] = start[i] + tw.size[1][i];
        }
    }
}

/// `CM_HandlePatchCollision(tw, trace, tStart, tEnd, patch, checkcount)` —
/// collides the swept AABB against a terrain patch's brushes, keeping the
/// shortest fraction (`patch->GetCollisionData()`/`GetNumBrushes()`,
/// `CM_GenericBoxCollide`, `CM_TraceThroughBrush`).
///
/// **Owned by the `cm` C-track packet** (its `CM_TraceThroughBrush`/
/// `CM_GenericBoxCollide` deps and the `CmPatch` collision-brush accessors land
/// with the wave-0–4 clipmap-trace lane). That lane has not landed, and the
/// only caller of this path (`CollisionWorld::terrain_patch_collide` via
/// `CmLandScape::patch_collide`) is itself unreachable until the same lane
/// lands its `cm-trace`/`cm-test` callers, so this is a callable no-op that
/// leaves `trace` unchanged (no terrain collision) — reconciled to the faithful
/// brush walk when the cm-trace wave lands.
//TODO: Port CM_HandlePatchCollision
// Source: oracle/codemp/qcommon/cm_trace.cpp:914-946
pub fn handle_patch_collision(
    tw: &mut traceWork_s,
    trace: &mut trace_t,
    t_start: vec3_t,
    t_end: vec3_t,
    patch: &mut CmPatch,
    checkcount: i32,
) {
    let _ = (tw, trace, t_start, t_end, patch, checkcount);
}
