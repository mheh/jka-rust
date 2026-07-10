#![allow(non_snake_case, non_upper_case_globals, clippy::too_many_arguments)]

//! `cm_patch.cpp` — patch (bezier curve) collision generation/trace (the
//! C-track TU). Its `_fns` basename is the DESTINATION-rule collision escape:
//! `cm_patch.rs` holds the §F `CCMPatch` terrain class (ruling 40), so the
//! Raven `cm_patch.cpp` free functions land here.
//!
//! STATUS — full recovery DEFERRED (integration open item). The C-track
//! `cm_patch.cpp` transcription (`git 437fb790:.../cm_patch.rs`) never compiled
//! and cannot be dropped in as-is:
//!   * `CM_DrawDebugSurface` (`cm_patch.cpp:1651`) needs a `drawPoly` callback
//!     fn-pointer type (`DrawPolyFn`) and `BotDrawDebugPolygons` that exist
//!     nowhere in the tree — defining that debug-surface seam is a design
//!     decision, not a mechanical fix (and the recovered 7-arg
//!     `BotDrawDebugPolygons` call has no oracle basis).
//!   * The recovered bodies use value-returning `VectorSubtract`/`CrossProduct`/
//!     `VectorAdd`/`VectorMA`/`DotProduct` (~33 sites) that this crate's
//!     out-param `q_math` does not provide, with NO differential test to guard
//!     the rewrite (porting-rules §18).
//! Only the two functions the live C-track `cm_trace.rs` collision path calls
//! are declared here, as loud unported stubs (marker rules), so the crate
//! compiles and the §F parity tests run while the full recovery is scheduled.

use mp_host_interface::engine_host::EngineHost;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::qboolean;

use crate::cm::patch_collide_s::patchCollide_s;
use crate::cm::trace_work_s::traceWork_t;
use crate::cm_load::RenderModels;
use crate::collision_world::CollisionWorld;
use crate::common::Common;

/// Raven `CM_PositionTestInPatchCollide` — box-position test against a patch's
/// facets.
//TODO: Port CM_PositionTestInPatchCollide — full cm_patch.cpp recovery deferred
// (see module STATUS); the terrain-patch collision path is not wired live.
// Source: oracle/codemp/qcommon/cm_patch.cpp:1214-1291
pub fn CM_PositionTestInPatchCollide(
    _tw: *mut traceWork_t,
    _pc: *const patchCollide_s,
) -> qboolean {
    todo!("Port CM_PositionTestInPatchCollide — oracle/codemp/qcommon/cm_patch.cpp:1214")
}

/// Raven `CM_TraceThroughPatchCollide` — sweep a trace through a patch's
/// facets.
//TODO: Port CM_TraceThroughPatchCollide — full cm_patch.cpp recovery deferred
// (see module STATUS); the terrain-patch collision path is not wired live.
// Source: oracle/codemp/qcommon/cm_patch.cpp:1392-1527
pub fn CM_TraceThroughPatchCollide(
    _common: &mut Common,
    _cm: &mut CollisionWorld,
    _rm: &mut RenderModels,
    _host: &mut dyn EngineHost,
    _tw: *mut traceWork_t,
    _trace: &mut trace_t,
    _pc: *const patchCollide_s,
) {
    todo!("Port CM_TraceThroughPatchCollide — oracle/codemp/qcommon/cm_patch.cpp:1392")
}
