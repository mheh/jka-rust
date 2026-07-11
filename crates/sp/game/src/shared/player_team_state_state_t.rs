#![allow(non_camel_case_types, non_snake_case)]

/// Raven `playerTeamStateState_t` — player team state.
///
/// Raven: Beginning a team game, spawn at base / Now actively playing.
/// Type definition source: `oracle/code/game/g_shared.h:246-249`
#[repr(i32)]
pub enum playerTeamStateState_t {
    TEAM_BEGIN = 0,  // Beginning a team game, spawn at base
    TEAM_ACTIVE = 1, // Now actively playing
}
