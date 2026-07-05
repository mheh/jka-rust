#![allow(non_camel_case_types, non_snake_case)]

/// Raven `evasionType_t` — saber evasion/parry/dodge types.
///
/// Type definition source: `oracle/oracle/codemp/game/w_saber.h:44-57`
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum evasionType_t {
    EVASION_NONE = 0,
    EVASION_PARRY = 1,
    EVASION_DUCK_PARRY = 2,
    EVASION_JUMP_PARRY = 3,
    EVASION_DODGE = 4,
    EVASION_JUMP = 5,
    EVASION_DUCK = 6,
    EVASION_FJUMP = 7,
    EVASION_CARTWHEEL = 8,
    EVASION_OTHER = 9,
    NUM_EVASION_TYPES = 10,
}
