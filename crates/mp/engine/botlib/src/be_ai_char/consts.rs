use core::ffi::{c_char, c_int};

/// Raven `MAX_CHARACTERISTICS` — max characteristics in one `bot_character_t`.
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:30`
pub const MAX_CHARACTERISTICS: c_int = 80;

/// Raven `CT_INTEGER` — `bot_characteristic_t::type` tag: value is `cvalue::integer`.
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:32`
pub const CT_INTEGER: c_char = 1;

/// Raven `CT_FLOAT` — `bot_characteristic_t::type` tag: value is `cvalue::_float`.
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:33`
pub const CT_FLOAT: c_char = 2;

/// Raven `CT_STRING` — `bot_characteristic_t::type` tag: value is `cvalue::string`.
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:34`
pub const CT_STRING: c_char = 3;

/// Raven `DEFAULT_CHARACTER` — default bot character script loaded when a bot's
/// requested character file is missing.
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:36`
pub const DEFAULT_CHARACTER: &str = "bots/default_c.c";
