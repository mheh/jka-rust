#![allow(non_camel_case_types, non_snake_case)]

/// Raven `edgeName_t` — named enum of grid-edge directions.
///
/// `EN_LEFT` is ported alongside the other three variants for enum
/// completeness (porting-rules enum-vs-alias fidelity: a named C enum ports
/// with ALL variants, not just the subset a given call site references).
///
/// Type definition source: `oracle/codemp/qcommon/cm_patch.cpp:970-975`
#[repr(i32)]
pub enum edgeName_t {
    EN_TOP = 0,
    EN_RIGHT = 1,
    EN_BOTTOM = 2,
    EN_LEFT = 3,
}
