//! MP botlib `be_aas_routealt.cpp` alternate-routing constants.
//!
//! The alternate-routing logic itself is not yet ported; this module exists
//! to house its `#define` constants (const-backfill).

pub mod be_aas_routealt_cpp_consts;

/// Raven `midrangearea_t`.
///
/// Per-area alternate-routing scratch state (mid-range-area flood/goal pick).
/// Source: `oracle/codemp/botlib/be_aas_routealt.cpp:32-37`
#[repr(C)]
pub struct midrangearea_t {
    pub valid: core::ffi::c_int,
    pub starttime: u16,
    pub goaltime: u16,
}
