//! MP `bg_public.h` persistent player statistics enumeration.
//!
//! Type definition source: `oracle/codemp/game/bg_public.h:539-556`

#![allow(non_camel_case_types)]

/// Raven `persEnum_t` — player persistent statistics indices.
///
/// Raven: Enumeration for various persistent player statistics tracked across
/// respawns and the game session, including score, rank, team, and awards.
/// Type definition source: `oracle/codemp/game/bg_public.h:539-556`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum persEnum_t {
    // !!! MUST NOT CHANGE, SERVER AND GAME BOTH REFERENCE !!!
    PERS_SCORE = 0,
    PERS_HITS = 1,
    PERS_RANK = 2,
    PERS_TEAM = 3,
    PERS_SPAWN_COUNT = 4,
    PERS_PLAYEREVENTS = 5,
    PERS_ATTACKER = 6,
    PERS_ATTACKEE_ARMOR = 7,
    PERS_KILLED = 8,
    PERS_IMPRESSIVE_COUNT = 9,
    PERS_EXCELLENT_COUNT = 10,
    PERS_DEFEND_COUNT = 11,
    PERS_ASSIST_COUNT = 12,
    PERS_GAUNTLET_FRAG_COUNT = 13,
    PERS_CAPTURES = 14,
}
