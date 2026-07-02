#![allow(non_camel_case_types, non_snake_case)]

/// Raven `saberBlockedType_t` — saber block reaction types.
///
/// Type definition source: `oracle/oracle/code/game/g_shared.h:358-372`
#[repr(i32)]
pub enum saberBlockedType_t {
    BLOCKED_NONE = 0,
    BLOCKED_PARRY_BROKEN = 1,
    BLOCKED_ATK_BOUNCE = 2,
    BLOCKED_UPPER_RIGHT = 3,
    BLOCKED_UPPER_LEFT = 4,
    BLOCKED_LOWER_RIGHT = 5,
    BLOCKED_LOWER_LEFT = 6,
    BLOCKED_TOP = 7,
    BLOCKED_UPPER_RIGHT_PROJ = 8,
    BLOCKED_UPPER_LEFT_PROJ = 9,
    BLOCKED_LOWER_RIGHT_PROJ = 10,
    BLOCKED_LOWER_LEFT_PROJ = 11,
    BLOCKED_TOP_PROJ = 12,
}
