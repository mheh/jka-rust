//! SP `team_t`.
//!
//! Type definition source: `oracle/code/game/teams.h:4-13`

#![allow(non_camel_case_types)]

/// Raven SP `team_t`.
///
/// Unlike MP (`typedef int team_t` + a free/red/blue/spectator value space), SP's
/// `team_t` is a **named** enum with faction semantics, and SP has no separate
/// `npcteam_t` — `team_t` does that job too.
/// Type definition source: `oracle/code/game/teams.h:4-13`
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum team_t {
    TEAM_FREE = 0,
    TEAM_PLAYER,
    TEAM_ENEMY,
    TEAM_NEUTRAL, // most droids are team_neutral (Probe/Seeker/Interrogator are exceptions)

    TEAM_NUM_TEAMS,
}
