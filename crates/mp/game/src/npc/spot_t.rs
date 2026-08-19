#![allow(non_camel_case_types, non_snake_case)]

/// Raven `spot_t`: body spot enum for targeting.
///
/// Type definition source: `oracle/codemp/game/b_public.h:69-69`
#[repr(i32)]
pub enum spot_t {
    SPOT_ORIGIN,
    SPOT_CHEST,
    SPOT_HEAD,
    SPOT_HEAD_LEAN,
    SPOT_WEAPON,
    SPOT_LEGS,
    SPOT_GROUND,
}
