//! Engine-side NPC navigation graph (`server/NPCNav`) — §F idiomatic
//! reimplementation of `oracle/codemp/server/NPCNav/navigator.cpp` +
//! `gameCallbacks.cpp`.
//!
//! Module root: the nav-owned constants (node/edge flags, header ids, the
//! file-scope waypoint bounds and checked-node sentinels) and the module
//! roster. The [`Navigator`] owner struct (RULING 12 — the
//! `mp_engine_core::Engine.nav` field, reached via the `EngineHostView`
//! split-borrow, NAV-D3 / RULING 43) lives in [`navigator`].
//!
//! `Q3_INFINITE` / `WORLD_SIZE` / `STEPSIZE` / `WAYPOINT_NONE` and the vec3
//! primitives are **not** defined here — they are imported from `mp_qshared`
//! (NAV-D3 / RULING 22 / 39d), never re-declared as local copies.
//!
//! Type definition source: `oracle/codemp/server/NPCNav/navigator.h`

use core::ffi::c_int;
use mp_qshared::common::mp::bg::stepsize::STEPSIZE;
use mp_qshared::shared::vec3_t;

pub mod callbacks;
pub mod edge;
pub mod navigator;
pub mod node;
pub mod priority_queue;

pub use edge::Edge;
pub use navigator::Navigator;
pub use node::{Node, NodeEdge};
pub use priority_queue::PriorityQueue;

// --- Node flags (navigator.h:9-12) --------------------------------------

/// Raven `NF_ANY`.
///
/// Source: `oracle/codemp/server/NPCNav/navigator.h:9`
pub const NF_ANY: c_int = 0;

// Raven's `NF_CLEAR_LOS` (0x00000001) is commented out in navigator.h:10 — not
// ported.

/// Raven `NF_CLEAR_PATH`.
///
/// Source: `oracle/codemp/server/NPCNav/navigator.h:11`
pub const NF_CLEAR_PATH: c_int = 0x0000_0002;

/// Raven `NF_RECALC`.
///
/// Source: `oracle/codemp/server/NPCNav/navigator.h:12`
pub const NF_RECALC: c_int = 0x0000_0004;

// --- Edge flags (navigator.h:15-17) -------------------------------------

/// Raven `EFLAG_NONE`.
///
/// Source: `oracle/codemp/server/NPCNav/navigator.h:15`
pub const EFLAG_NONE: c_int = 0;

/// Raven `EFLAG_BLOCKED`.
///
/// Source: `oracle/codemp/server/NPCNav/navigator.h:16`
pub const EFLAG_BLOCKED: c_int = 0x0000_0001;

/// Raven `EFLAG_FAILED`.
///
/// Source: `oracle/codemp/server/NPCNav/navigator.h:17`
pub const EFLAG_FAILED: c_int = 0x0000_0002;

// --- Miscellaneous (navigator.h:20-22) ----------------------------------

/// Raven `NODE_NONE`.
///
/// Source: `oracle/codemp/server/NPCNav/navigator.h:20`
pub const NODE_NONE: c_int = -1;

// Raven's `NAV_HEADER_ID`/`NODE_HEADER_ID` are multi-char `int` literals
// (`'JNV5'` = 'J'<<24|'N'<<16|'V'<<8|'5'; `'NODE'` likewise). Inside the `.nav`
// binary format these header ids are pinned to exactly 4 bytes (NAV-D1 / RULING
// 44) — the `Load`/`Save` cursor reads/writes them as `u32` and compares with
// `NAV_HEADER_ID as u32` / `NODE_HEADER_ID as u32`; never `c_long`/`c_ulong`.

/// Raven `NAV_HEADER_ID` — the `.nav` magic `'JNV5'`.
///
/// Source: `oracle/codemp/server/NPCNav/navigator.h:21`
pub const NAV_HEADER_ID: c_int = 0x4A4E_5635; // 'JNV5'

/// Raven `NODE_HEADER_ID` — the per-node magic `'NODE'`.
///
/// Source: `oracle/codemp/server/NPCNav/navigator.h:22`
pub const NODE_HEADER_ID: c_int = 0x4E4F_4445; // 'NODE'

/// Raven `MAX_FAILED_EDGES`.
///
/// `usize` so it sizes `Navigator::failed_edges: [failedEdge_t; MAX_FAILED_EDGES]`.
/// Source: `oracle/codemp/server/NPCNav/navigator.h:133`
pub const MAX_FAILED_EDGES: usize = 32;

// --- File-scope waypoint bounds (navigator.cpp:50-51) -------------------

/// Raven `wpMaxs` — the waypoint trace-box maxs `{ 16, 16, 32 }`.
///
/// Source: `oracle/codemp/server/NPCNav/navigator.cpp:50`
pub const WP_MAXS: vec3_t = [16.0, 16.0, 32.0];

/// Raven `wpMins` — the waypoint trace-box mins `{ -16, -16, -24+STEPSIZE }`.
///
/// Raven: `//WTF:  was 16??!!!`. The z-bound reads `STEPSIZE` from `mp_qshared`
/// (NAV-D3 / RULING 39d).
/// Source: `oracle/codemp/server/NPCNav/navigator.cpp:51`
pub const WP_MINS: vec3_t = [-16.0, -16.0, -24.0 + STEPSIZE];

// --- Checked-node sentinels (navigator.cpp:54-56) -----------------------

/// Raven `CHECKED_NO`.
///
/// Source: `oracle/codemp/server/NPCNav/navigator.cpp:54`
pub const CHECKED_NO: u8 = 0;

/// Raven `CHECKED_FAILED`.
///
/// Source: `oracle/codemp/server/NPCNav/navigator.cpp:55`
pub const CHECKED_FAILED: u8 = 1;

/// Raven `CHECKED_PASSED`.
///
/// Source: `oracle/codemp/server/NPCNav/navigator.cpp:56`
pub const CHECKED_PASSED: u8 = 2;
