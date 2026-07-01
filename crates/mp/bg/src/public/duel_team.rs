//! MP `bg_public.h` duel team type definitions.
//!
//! Type definition source: `oracle/oracle/codemp/game/bg_public.h:1019-1025`

#![allow(non_camel_case_types)]

/// Raven `duelTeam_t`.
///
/// Type definition source: `oracle/oracle/codemp/game/bg_public.h:1019-1025`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum duelTeam_t {
    DUELTEAM_FREE = 0,
    DUELTEAM_LONE = 1,
    DUELTEAM_DOUBLE = 2,
    DUELTEAM_SINGLE = 3,
}
