#![allow(non_camel_case_types, non_snake_case)]

use super::bot_characteristic_s::Characteristic;

/// Raven `bot_character_t` — a loaded bot character, redesigned
/// (porting-rules §F17; botlib is statically linked, so layout is free) from
/// the fixed `filename[MAX_QPATH]` + trailing variable-sized characteristic
/// array into owned fields. `c` holds `MAX_CHARACTERISTICS + 1` slots to match
/// Raven's allocation (`sizeof(bot_character_t)` — whose `c[1]` is one slot —
/// `+ MAX_CHARACTERISTICS * sizeof(bot_characteristic_t)`), so the parser's
/// inclusive `index <= MAX_CHARACTERISTICS` bound stays in range; every slot
/// defaults to `Characteristic::None` (Raven's zero-init).
///
/// Type definition source: `oracle/codemp/botlib/be_ai_char.cpp:53-58`
pub struct BotCharacter {
    pub filename: String,
    pub skill: f32,
    /// variable sized (`MAX_CHARACTERISTICS + 1` slots, see above)
    pub c: Vec<Characteristic>,
}
