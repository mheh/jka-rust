#![allow(non_camel_case_types, non_snake_case)]

/// Raven `bot_teamplay_state_t` — Teamplay bot state enumeration.
///
/// Type definition source: `oracle/codemp/game/ai_main.h:100-107`
#[repr(i32)]
pub enum bot_teamplay_state_t {
    TEAMPLAYSTATE_NONE = 0,
    TEAMPLAYSTATE_FOLLOWING,
    TEAMPLAYSTATE_ASSISTING,
    TEAMPLAYSTATE_REGROUP,
    TEAMPLAYSTATE_MAXTPSTATES,
}
