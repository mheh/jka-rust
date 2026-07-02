#![allow(non_camel_case_types, non_snake_case)]

/// Raven `leBounceSound_t` — bounce sound types for fragment local entities.
///
/// Raven: fragment local entities can make sounds on impacts.
/// Type definition source: `oracle/oracle/code/cgame/cg_local.h:215-220`
#[repr(i32)]
pub enum leBounceSound_t {
    LEBS_NONE = 0,
    LEBS_METAL = 1,
    LEBS_ROCK = 2,
}
