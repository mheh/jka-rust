#![allow(non_camel_case_types)]

/// Raven `itemUseFail_t` reasons an item use failed.
///
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:2126-2131`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum itemUseFail_t {
    SENTRY_NOROOM = 1,
    SENTRY_ALREADYPLACED,
    SHIELD_NOROOM,
    SEEKER_ALREADYDEPLOYED,
}
