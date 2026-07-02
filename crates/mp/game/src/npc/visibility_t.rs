#![allow(non_camel_case_types, non_snake_case)]

/// Raven `visibility_t` — visibility state of a target.
///
/// Type definition source: `oracle/oracle/codemp/game/b_public.h:68-68`
#[repr(i32)]
pub enum visibility_t {
    VIS_UNKNOWN,
    VIS_NOT,
    VIS_PVS,
    VIS_360,
    VIS_FOV,
    VIS_SHOOT,
}
