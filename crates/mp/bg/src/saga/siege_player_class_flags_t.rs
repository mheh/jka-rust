#![allow(non_camel_case_types, non_snake_case)]

/// Raven `siegePlayerClassFlags_t` — player class type flags.
///
/// Type definition source: `oracle/codemp/game/bg_saga.h:20-29`
#[repr(i32)]
pub enum siegePlayerClassFlags_t {
    SPC_INFANTRY = 0,
    SPC_VANGUARD,
    SPC_SUPPORT,
    SPC_JEDI,
    SPC_DEMOLITIONIST,
    SPC_HEAVY_WEAPONS,
    SPC_MAX,
}
