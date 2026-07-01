//! MP `bg_public.h` force hand animation definitions.
//!
//! Type definition source: `oracle/oracle/codemp/game/bg_public.h:149-170`

#![allow(non_camel_case_types)]

/// Raven `forceHandAnims_t`.
///
/// Type definition source: `oracle/oracle/codemp/game/bg_public.h:149-170`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum forceHandAnims_t {
    HANDEXTEND_NONE = 0,
    HANDEXTEND_FORCEPUSH = 1,
    HANDEXTEND_FORCEPULL = 2,
    HANDEXTEND_FORCE_HOLD = 3,
    HANDEXTEND_SABERPULL = 4,
    HANDEXTEND_CHOKE = 5,
    HANDEXTEND_WEAPONREADY = 6,
    HANDEXTEND_DODGE = 7,
    HANDEXTEND_KNOCKDOWN = 8,
    HANDEXTEND_DUELCHALLENGE = 9,
    HANDEXTEND_TAUNT = 10,
    HANDEXTEND_PRETHROW = 11,
    HANDEXTEND_POSTTHROW = 12,
    HANDEXTEND_PRETHROWN = 13,
    HANDEXTEND_POSTTHROWN = 14,
    HANDEXTEND_DRAGGING = 15,
    HANDEXTEND_JEDITAUNT = 16,
}
