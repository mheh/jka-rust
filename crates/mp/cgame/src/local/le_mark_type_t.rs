#![allow(non_camel_case_types, non_snake_case)]

/// Raven `leMarkType_t` — mark types for fragment local entities.
///
/// Raven: fragment local entities can leave marks on walls.
/// Type definition source: `oracle/oracle/codemp/cgame/cg_local.h:505-509`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum leMarkType_t {
    LEMT_NONE = 0,
    LEMT_BURN = 1,
    LEMT_BLOOD = 2,
}
