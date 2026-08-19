//! MP `spectatorState_t`.
//!
//! Type definition source: `oracle/codemp/game/g_local.h:374-378`

#![allow(non_camel_case_types)]

/// Raven `spectatorState_t`.
///
/// This is a named enum in the oracle, not a `typedef int`.
/// Type definition source: `oracle/codemp/game/g_local.h:374-378`
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum spectatorState_t {
    SPECTATOR_NOT = 0,
    SPECTATOR_FREE,
    SPECTATOR_FOLLOW,
    SPECTATOR_SCOREBOARD,
}
