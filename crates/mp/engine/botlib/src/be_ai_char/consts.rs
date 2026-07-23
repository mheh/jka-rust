use core::ffi::c_int;

/// Raven `MAX_CHARACTERISTICS` — max characteristics in one `bot_character_t`.
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:30`
pub const MAX_CHARACTERISTICS: c_int = 80;

// Raven's `CT_INTEGER`/`CT_FLOAT`/`CT_STRING` `type`-tag `#define`s are dropped:
// the tag folds into the `Characteristic` enum discriminant (§F17 redesign,
// `bot_characteristic_s.rs`). Source: `oracle/codemp/botlib/be_ai_char.cpp:32-34`

/// Raven `DEFAULT_CHARACTER` — default bot character script loaded when a bot's
/// requested character file is missing.
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:36`
pub const DEFAULT_CHARACTER: &str = "bots/default_c.c";
