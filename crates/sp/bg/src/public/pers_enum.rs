//! SP `bg_public.h` player persistence enumeration.
//!
//! Type definition source: `oracle/oracle/code/game/bg_public.h:195-208`

#![allow(non_camel_case_types)]

/// Raven `persEnum_t`.
///
/// Type definition source: `oracle/oracle/code/game/bg_public.h:195-208`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum persEnum_t {
    PERS_SCORE = 0,                 // !!! MUST NOT CHANGE, SERVER AND GAME BOTH REFERENCE !!!
    PERS_HITS = 1,                  // total points damage inflicted so damage beeps can sound on change
    PERS_TEAM = 2,
    PERS_SPAWN_COUNT = 3,           // incremented every respawn
    //	PERS_REWARD_COUNT,				// incremented for each reward sound
    PERS_ATTACKER = 4,              // clientnum of last damage inflicter
    PERS_KILLED = 5,                // count of the number of times you died
    PERS_ACCURACY_SHOTS = 6,        // scoreboard - number of player shots
    PERS_ACCURACY_HITS = 7,         // scoreboard - number of player shots that hit an enemy
    PERS_ENEMIES_KILLED = 8,        // scoreboard - number of enemies player killed
    PERS_TEAMMATES_KILLED = 9,      // scoreboard - number of teammates killed
}
