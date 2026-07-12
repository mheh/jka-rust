#![allow(non_camel_case_types, non_snake_case)]

/// Raven `evasionType_t` — evasion type enum.
///
/// Type definition source: `oracle/code/game/wp_saber.h:191-204`
#[repr(i32)]
pub enum evasionType_t {
    EVASION_NONE = 0,
    EVASION_PARRY,
    EVASION_DUCK_PARRY,
    EVASION_JUMP_PARRY,
    EVASION_DODGE,
    EVASION_JUMP,
    EVASION_DUCK,
    EVASION_FJUMP,
    EVASION_CARTWHEEL,
    EVASION_OTHER,
    NUM_EVASION_TYPES,
}
