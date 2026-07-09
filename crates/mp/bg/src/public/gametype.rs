//! MP `bg_public.h` game-type definitions.
//!
//! Type definition source: `oracle/codemp/game/bg_public.h:183-199`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

/// Raven `gametype_t`.
///
/// Raven names the game types via an anonymous `enum { GT_FFA..GT_MAX_GAME_TYPE }`,
/// then `typedef int gametype_t` for storage.
/// Type definition source: `oracle/codemp/game/bg_public.h:183-199`
pub type gametype_t = c_int;

/// free for all
pub const GT_FFA: gametype_t = 0;
/// holocron ffa
pub const GT_HOLOCRON: gametype_t = 1;
/// jedi master
pub const GT_JEDIMASTER: gametype_t = 2;
/// one on one tournament
pub const GT_DUEL: gametype_t = 3;
pub const GT_POWERDUEL: gametype_t = 4;
/// single player ffa
pub const GT_SINGLE_PLAYER: gametype_t = 5;

// -- team games go after this --

/// team deathmatch
pub const GT_TEAM: gametype_t = 6;
/// siege
pub const GT_SIEGE: gametype_t = 7;
/// capture the flag
pub const GT_CTF: gametype_t = 8;
pub const GT_CTY: gametype_t = 9;
pub const GT_MAX_GAME_TYPE: gametype_t = 10;
