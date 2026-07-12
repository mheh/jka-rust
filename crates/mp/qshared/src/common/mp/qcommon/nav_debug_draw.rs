#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

// Two anonymous `enum { ... }` blocks (no typedef) used as raw `int` args to
// `G_DrawEdge`/`G_DrawNode` (nav debug-draw helpers); ported as plain consts
// per the typedef-less-anonymous-enum rule.
// Type definition source: `oracle/codemp/game/g_public.h:608-623`

/// Raven edge-debug-draw kind.
pub const EDGE_NORMAL: c_int = 0;
/// Raven edge-debug-draw kind.
pub const EDGE_PATH: c_int = 1;
/// Raven edge-debug-draw kind.
pub const EDGE_BLOCKED: c_int = 2;
/// Raven edge-debug-draw kind.
pub const EDGE_FAILED: c_int = 3;
/// Raven edge-debug-draw kind.
pub const EDGE_MOVEDIR: c_int = 4;

/// Raven node-debug-draw kind.
pub const NODE_NORMAL: c_int = 0;
/// Raven node-debug-draw kind.
pub const NODE_START: c_int = 1;
/// Raven node-debug-draw kind.
pub const NODE_GOAL: c_int = 2;
/// Raven node-debug-draw kind.
pub const NODE_NAVGOAL: c_int = 3;
