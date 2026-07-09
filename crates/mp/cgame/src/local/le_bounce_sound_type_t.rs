#![allow(non_camel_case_types, non_snake_case)]

/// Raven `leBounceSoundType_t` — bounce sound types for fragment local entities.
///
/// Raven: fragment local entities can make sounds on impacts.
/// Type definition source: `oracle/codemp/cgame/cg_local.h:511-517`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum leBounceSoundType_t {
    LEBS_NONE = 0,
    LEBS_BLOOD = 1,
    LEBS_BRASS = 2,
    LEBS_METAL = 3,
    LEBS_ROCK = 4,
}
