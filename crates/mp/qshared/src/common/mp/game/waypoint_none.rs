//! MP `g_nav.h` `WAYPOINT_NONE`.
//!
//! NAV-D3 / RULING 39d migration: moved here from `mp_game`
//! (`crates/mp/game/src/g_nav_consts.rs:13`) so the engine-side nav code
//! (`mp_engine_server`) shares the single referee-compared definition.
//!
//! Raven: "This file is shared by the exe nav code. If you modify it without
//! recompiling the exe with new code, there could be issues."
//!
//! Source: `oracle/codemp/game/g_nav.h:7`

use core::ffi::c_int;

/// Raven `WAYPOINT_NONE`.
///
/// Source: `oracle/codemp/game/g_nav.h:7`
pub const WAYPOINT_NONE: c_int = -1;
