//! Player persistent state enumeration for `persistant[]` array indices.
//!
//! Type definition source: `oracle/oracle/codemp/game/bg_public.h:539-556`

/// Raven `PERS_SCORE` — player score index in `playerState_t::persistant[]`.
/// Source: `oracle/oracle/codemp/game/bg_public.h:540`
pub const PERS_SCORE: i32 = 0;

/// Raven `PERS_HITS` — total points damage inflicted.
/// Source: `oracle/oracle/codemp/game/bg_public.h:541`
pub const PERS_HITS: i32 = 1;

/// Raven `PERS_RANK` — player rank or team rank.
/// Source: `oracle/oracle/codemp/game/bg_public.h:542`
pub const PERS_RANK: i32 = 2;

/// Raven `PERS_TEAM` — player team.
/// Source: `oracle/oracle/codemp/game/bg_public.h:543`
pub const PERS_TEAM: i32 = 3;

/// Raven `PERS_SPAWN_COUNT` — incremented every respawn.
/// Source: `oracle/oracle/codemp/game/bg_public.h:544`
pub const PERS_SPAWN_COUNT: i32 = 4;

/// Raven `PERS_PLAYEREVENTS` — 16 bits that can be flipped for events.
/// Source: `oracle/oracle/codemp/game/bg_public.h:545`
pub const PERS_PLAYEREVENTS: i32 = 5;

/// Raven `PERS_ATTACKER` — clientnum of last damage inflicter.
/// Source: `oracle/oracle/codemp/game/bg_public.h:546`
pub const PERS_ATTACKER: i32 = 6;

/// Raven `PERS_ATTACKEE_ARMOR` — health/armor of last person we attacked.
/// Source: `oracle/oracle/codemp/game/bg_public.h:547`
pub const PERS_ATTACKEE_ARMOR: i32 = 7;

/// Raven `PERS_KILLED` — count of the number of times you died.
/// Source: `oracle/oracle/codemp/game/bg_public.h:548`
pub const PERS_KILLED: i32 = 8;

/// Raven `PERS_IMPRESSIVE_COUNT` — two railgun hits in a row.
/// Source: `oracle/oracle/codemp/game/bg_public.h:550`
pub const PERS_IMPRESSIVE_COUNT: i32 = 9;

/// Raven `PERS_EXCELLENT_COUNT` — two successive kills in a short amount of time.
/// Source: `oracle/oracle/codemp/game/bg_public.h:551`
pub const PERS_EXCELLENT_COUNT: i32 = 10;

/// Raven `PERS_DEFEND_COUNT` — defend awards.
/// Source: `oracle/oracle/codemp/game/bg_public.h:552`
pub const PERS_DEFEND_COUNT: i32 = 11;

/// Raven `PERS_ASSIST_COUNT` — assist awards.
/// Source: `oracle/oracle/codemp/game/bg_public.h:553`
pub const PERS_ASSIST_COUNT: i32 = 12;

/// Raven `PERS_GAUNTLET_FRAG_COUNT` — kills with the guantlet.
/// Source: `oracle/oracle/codemp/game/bg_public.h:554`
pub const PERS_GAUNTLET_FRAG_COUNT: i32 = 13;

/// Raven `PERS_CAPTURES` — captures.
/// Source: `oracle/oracle/codemp/game/bg_public.h:555`
pub const PERS_CAPTURES: i32 = 14;
