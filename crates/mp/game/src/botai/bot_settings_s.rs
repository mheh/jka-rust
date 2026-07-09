//! MP `bot_settings_t`.
//!
//! Type definition source: `oracle/codemp/game/g_local.h:1490-1496`

#![allow(non_camel_case_types)]

// `bot_settings_s` is a field of `bot_state_t` (`ai_main.h:156`) but is
// canonically ported once, in `crate::level::bot_settings`, since it is
// declared in `g_local.h` rather than a bot-AI header. Re-export it here so
// `botai::bot_state_s` can reach it without a duplicate definition.
pub use crate::level::bot_settings::{bot_settings_t, MAX_FILEPATH};
