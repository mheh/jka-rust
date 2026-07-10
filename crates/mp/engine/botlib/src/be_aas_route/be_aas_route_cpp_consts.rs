#![allow(non_camel_case_types)]

//! `be_aas_route.cpp`-local route-table constants.
//!
//! Source: `oracle/codemp/botlib/be_aas_route.cpp:31-42,911-912,1124`

/// Raven `ROUTING_DEBUG` — unconditionally-defined feature guard for the
/// routing-debug print/timing paths. Ported as `bool` since Raven never
/// gives it a value, only tests it with `#ifdef`, and it is defined
/// unconditionally at this site.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:31`
pub const ROUTING_DEBUG: bool = true;

/// Raven `DISTANCEFACTOR_CROUCH` — crouch speed = 100.
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:34`
pub const DISTANCEFACTOR_CROUCH: f32 = 1.3;

/// Raven `DISTANCEFACTOR_SWIM` — Raven's own comment says "should be 0.66,
/// swim speed = 150"; ported faithfully as the actual `1`, not the value the
/// comment suggests (§A2: no speculative behavior).
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:35`
pub const DISTANCEFACTOR_SWIM: f32 = 1.0;

/// Raven `DISTANCEFACTOR_WALK` — walk speed = 300.
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:36`
pub const DISTANCEFACTOR_WALK: f32 = 0.33;

/// Raven `MAX_FRAMEROUTINGUPDATES`.
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:42`
pub const MAX_FRAMEROUTINGUPDATES: i32 = 10;

/// Raven `MAX_REACHABILITYPASSAREAS` — sizes the `areas[]` scratch array in
/// `AAS_ReachabilityArea_Passable`-style traces.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:1124`
pub const MAX_REACHABILITYPASSAREAS: usize = 32;

/// Raven `RCID` — route-cache file magic id, `('C'<<24)+('R'<<16)+('E'<<8)+'M'`.
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:911`
pub const RCID: i32 = 0x4352454D;

/// Raven `RCVERSION` — route-cache file version.
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:912`
pub const RCVERSION: i32 = 2;
