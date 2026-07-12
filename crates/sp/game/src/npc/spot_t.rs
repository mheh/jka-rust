#![allow(non_camel_case_types, non_snake_case)]

/// Raven `spot_t` — body spot enumeration.
///
/// Type definition source: `oracle/code/game/b_public.h:89-89`
#[repr(i32)]
pub enum spot_t {
    SPOT_ORIGIN = 0,
    SPOT_CHEST,
    SPOT_HEAD,
    SPOT_HEAD_LEAN,
    SPOT_WEAPON,
    SPOT_LEGS,
    SPOT_GROUND,
}
