#![allow(non_camel_case_types)]

/// Raven `saberBlockedType_t` saber block-direction states.
///
/// Type definition source: `oracle/codemp/game/q_shared.h:558-573`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum saberBlockedType_t {
    BLOCKED_NONE,
    BLOCKED_BOUNCE_MOVE,
    BLOCKED_PARRY_BROKEN,
    BLOCKED_ATK_BOUNCE,
    BLOCKED_UPPER_RIGHT,
    BLOCKED_UPPER_LEFT,
    BLOCKED_LOWER_RIGHT,
    BLOCKED_LOWER_LEFT,
    BLOCKED_TOP,
    BLOCKED_UPPER_RIGHT_PROJ,
    BLOCKED_UPPER_LEFT_PROJ,
    BLOCKED_LOWER_RIGHT_PROJ,
    BLOCKED_LOWER_LEFT_PROJ,
    BLOCKED_TOP_PROJ,
}
