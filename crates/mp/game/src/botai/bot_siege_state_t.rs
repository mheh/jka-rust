#![allow(non_camel_case_types, non_snake_case)]

/// Raven `bot_siege_state_t`: siege bot state enumeration.
///
/// Type definition source: `oracle/codemp/game/ai_main.h:92-98`
#[repr(i32)]
pub enum bot_siege_state_t {
    SIEGESTATE_NONE = 0,
    SIEGESTATE_ATTACKER,
    SIEGESTATE_DEFENDER,
    SIEGESTATE_MAXSIEGESTATES,
}
