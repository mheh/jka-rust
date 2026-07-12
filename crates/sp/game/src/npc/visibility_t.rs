#![allow(non_camel_case_types, non_snake_case)]

/// Raven `visibility_t` — visibility enumeration.
///
/// Type definition source: `oracle/code/game/b_public.h:88-88`
#[repr(i32)]
pub enum visibility_t {
    VIS_UNKNOWN = 0,
    VIS_NOT,
    VIS_PVS,
    VIS_360,
    VIS_FOV,
    VIS_SHOOT,
}
