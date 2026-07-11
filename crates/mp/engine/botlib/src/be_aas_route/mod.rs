//! MP botlib `be_aas_route.cpp` route-table constants.
//!
//! The route-table computation logic itself is not yet ported; this module
//! exists to house its `#define` constants (const-backfill), following the
//! same per-header layout as `be_aas_bsp`/`be_aas_reach`/etc.

pub mod be_aas_route_cpp_consts;

pub use be_aas_route_cpp_consts::{
    DISTANCEFACTOR_CROUCH, DISTANCEFACTOR_SWIM, DISTANCEFACTOR_WALK, MAX_REACHABILITYPASSAREAS,
    RCID, RCVERSION,
};

/// Raven `routecacheheader_t` — `.rcd` route-cache dump file header.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:899-909`
#[repr(C)]
#[derive(Default)]
pub struct routecacheheader_t {
    pub ident: std::os::raw::c_int,
    pub version: std::os::raw::c_int,
    pub numareas: std::os::raw::c_int,
    pub numclusters: std::os::raw::c_int,
    pub areacrc: std::os::raw::c_int,
    pub clustercrc: std::os::raw::c_int,
    pub numportalcache: std::os::raw::c_int,
    pub numareacache: std::os::raw::c_int,
}
