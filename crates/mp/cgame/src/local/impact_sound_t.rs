#![allow(non_camel_case_types, non_snake_case)]

/// Raven `impactSound_t` — impact sound type enumeration.
///
/// Type definition source: `oracle/oracle/codemp/cgame/cg_local.h:120-124`
#[repr(i32)]
pub enum impactSound_t {
    IMPACTSOUND_DEFAULT = 0,
    IMPACTSOUND_METAL = 1,
    IMPACTSOUND_FLESH = 2,
}
