//! The server game module (jampgame, Raven `codemp/game`): the game-private types and logic.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::c_int;

/// `GAMEVERSION` — the "gameversion" client command prints this plus compile date.
pub const GAMEVERSION: &str = "basejka";

/// `INFINITE` (g_local.h `#ifndef INFINITE`).
pub const INFINITE: c_int = 1000000;

pub const FRAMETIME: c_int = 100; // msec
pub const CARNAGE_REWARD_TIME: c_int = 3000;
pub const REWARD_SPRITE_TIME: c_int = 2000;

pub const INTERMISSION_DELAY_TIME: c_int = 1000;
pub const SP_INTERMISSION_DELAY_TIME: c_int = 5000;

//primarily used by NPCs
pub const START_TIME_LINK_ENTS: c_int = FRAMETIME * 1; // time-delay after map start at which all ents have been spawned, so can link them
pub const START_TIME_FIND_LINKS: c_int = FRAMETIME * 2; // time-delay after map start at which you can find linked entities
pub const START_TIME_MOVERS_SPAWNED: c_int = FRAMETIME * 2; // time-delay after map start at which all movers should be spawned
pub const START_TIME_REMOVE_ENTS: c_int = FRAMETIME * 3; // time-delay after map start to remove temporary ents
pub const START_TIME_NAV_CALC: c_int = FRAMETIME * 4; // time-delay after map start to connect waypoints and calc routes
pub const START_TIME_FIND_WAYPOINT: c_int = FRAMETIME * 5; // time-delay after map start after which it's okay to try to find your best waypoint

pub const MAX_G_SHARED_BUFFER_SIZE: usize = 8192;

/// `SP_PODIUM_MODEL` (g_local.h).
pub const SP_PODIUM_MODEL: &str = "models/mapobjects/podium/podium4.md3";

// Type modules — copied from the oracle as faithful starting material (Raven comments
// retained). Not yet wired into the build: they reference dependency types
// (entityState_t, gitem_t, AIGroupInfo_t, …) pending port into shared/, bg/, boundary/.
// pub mod client;
// pub mod entity;
// pub mod level;
