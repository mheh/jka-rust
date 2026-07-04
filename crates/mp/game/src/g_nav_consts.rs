//! MP `g_nav.h` waypoint/nav-node shared constants.
//!
//! Raven: "This file is shared by the exe nav code. If you modify it without
//! recompiling the exe with new code, there could be issues."
//!
//! Source: `oracle/oracle/codemp/game/g_nav.h`

use core::ffi::c_int;

/// Raven `WAYPOINT_NONE`.
///
/// Source: `oracle/oracle/codemp/game/g_nav.h:7`
pub const WAYPOINT_NONE: c_int = -1;

/// Raven `MAX_STORED_WAYPOINTS`.
///
/// Source: `oracle/oracle/codemp/game/g_nav.h:9`
pub const MAX_STORED_WAYPOINTS: usize = 512;

/// Raven `MAX_WAYPOINT_REACHED_DIST_SQUARED` (32 squared).
///
/// Source: `oracle/oracle/codemp/game/g_nav.h:10`
pub const MAX_WAYPOINT_REACHED_DIST_SQUARED: c_int = 1024;

/// Raven `MAX_COLL_AVOID_DIST`.
///
/// Source: `oracle/oracle/codemp/game/g_nav.h:11`
pub const MAX_COLL_AVOID_DIST: c_int = 128;

/// Raven `NAVGOAL_USE_RADIUS` — used to force the waypoint_navgoals with a
/// manually set radius to actually do a `DistanceSquared` check, not just
/// bounds overlap.
///
/// Source: `oracle/oracle/codemp/game/g_nav.h:12`
pub const NAVGOAL_USE_RADIUS: c_int = 16384;

/// Raven `MIN_STOP_DIST`.
///
/// Source: `oracle/oracle/codemp/game/g_nav.h:14`
pub const MIN_STOP_DIST: c_int = 64;

/// Raven `MIN_BLOCKED_SPEECH_TIME`.
///
/// Source: `oracle/oracle/codemp/game/g_nav.h:15`
pub const MIN_BLOCKED_SPEECH_TIME: c_int = 4000;

/// Raven `MIN_DOOR_BLOCK_DIST`.
///
/// Source: `oracle/oracle/codemp/game/g_nav.h:16`
pub const MIN_DOOR_BLOCK_DIST: c_int = 16;

/// Raven `MIN_DOOR_BLOCK_DIST_SQR`.
///
/// Source: `oracle/oracle/codemp/game/g_nav.h:17`
pub const MIN_DOOR_BLOCK_DIST_SQR: c_int = MIN_DOOR_BLOCK_DIST * MIN_DOOR_BLOCK_DIST;

/// Raven `SHOVE_SPEED`.
///
/// Source: `oracle/oracle/codemp/game/g_nav.h:18`
pub const SHOVE_SPEED: c_int = 200;

/// Raven `SHOVE_LIFT`.
///
/// Source: `oracle/oracle/codemp/game/g_nav.h:19`
pub const SHOVE_LIFT: c_int = 10;

/// Raven `MAX_RADIUS_CHECK`.
///
/// Source: `oracle/oracle/codemp/game/g_nav.h:20`
pub const MAX_RADIUS_CHECK: c_int = 1024;

/// Raven `YAW_ITERATIONS`.
///
/// Source: `oracle/oracle/codemp/game/g_nav.h:21`
pub const YAW_ITERATIONS: c_int = 16;

// Node flags.
/// Raven `NF_ANY`. Source: `oracle/oracle/codemp/game/g_nav.h:31`
pub const NF_ANY: c_int = 0;
/// Raven `NF_CLEAR_PATH`. Source: `oracle/oracle/codemp/game/g_nav.h:33`
pub const NF_CLEAR_PATH: c_int = 0x0000_0002;
/// Raven `NF_RECALC`. Source: `oracle/oracle/codemp/game/g_nav.h:34`
pub const NF_RECALC: c_int = 0x0000_0004;

// Edge flags.
/// Raven `EFLAG_NONE`. Source: `oracle/oracle/codemp/game/g_nav.h:37`
pub const EFLAG_NONE: c_int = 0;
/// Raven `EFLAG_BLOCKED`. Source: `oracle/oracle/codemp/game/g_nav.h:38`
pub const EFLAG_BLOCKED: c_int = 0x0000_0001;
/// Raven `EFLAG_FAILED`. Source: `oracle/oracle/codemp/game/g_nav.h:39`
pub const EFLAG_FAILED: c_int = 0x0000_0002;

/// Raven `NODE_NONE`.
///
/// Source: `oracle/oracle/codemp/game/g_nav.h:43`
pub const NODE_NONE: c_int = -1;
