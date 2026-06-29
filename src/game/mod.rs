//! Legacy staging area for the MP server game module (`jampgame`).
//!
//! Migration target: `crate::modules::mp::game`.
//! Raven: `codemp/game/g_local.h` holds these game-private constants and the
//! type blocks staged in `client`, `entity`, and `level`.
//! Source: `oracle/oracle/codemp/game/g_local.h:28`

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::c_int;

/// `GAMEVERSION`.
///
/// Raven: the "gameversion" client command will print this plus compile date.
/// Source: `oracle/oracle/codemp/game/g_local.h:28`
pub const GAMEVERSION: &str = "basejka";

/// `INFINITE`.
///
/// Raven: `#ifndef INFINITE`.
/// Source: `oracle/oracle/codemp/game/g_local.h:33`
pub const INFINITE: c_int = 1000000;

/// `FRAMETIME`.
///
/// Raven: msec.
/// Source: `oracle/oracle/codemp/game/g_local.h:37`
pub const FRAMETIME: c_int = 100; // msec

/// `CARNAGE_REWARD_TIME`.
///
/// Source: `oracle/oracle/codemp/game/g_local.h:38`
pub const CARNAGE_REWARD_TIME: c_int = 3000;

/// `REWARD_SPRITE_TIME`.
///
/// Source: `oracle/oracle/codemp/game/g_local.h:39`
pub const REWARD_SPRITE_TIME: c_int = 2000;

/// `INTERMISSION_DELAY_TIME`.
///
/// Source: `oracle/oracle/codemp/game/g_local.h:41`
pub const INTERMISSION_DELAY_TIME: c_int = 1000;

/// `SP_INTERMISSION_DELAY_TIME`.
///
/// Source: `oracle/oracle/codemp/game/g_local.h:42`
pub const SP_INTERMISSION_DELAY_TIME: c_int = 5000;

/// `START_TIME_LINK_ENTS`.
///
/// Raven: primarily used by NPCs; time-delay after map start at which all ents
/// have been spawned, so can link them.
/// Source: `oracle/oracle/codemp/game/g_local.h:44`
pub const START_TIME_LINK_ENTS: c_int = FRAMETIME * 1; // time-delay after map start at which all ents have been spawned, so can link them

/// `START_TIME_FIND_LINKS`.
///
/// Raven: time-delay after map start at which you can find linked entities.
/// Source: `oracle/oracle/codemp/game/g_local.h:46`
pub const START_TIME_FIND_LINKS: c_int = FRAMETIME * 2; // time-delay after map start at which you can find linked entities

/// `START_TIME_MOVERS_SPAWNED`.
///
/// Raven: time-delay after map start at which all movers should be spawned.
/// Source: `oracle/oracle/codemp/game/g_local.h:47`
pub const START_TIME_MOVERS_SPAWNED: c_int = FRAMETIME * 2; // time-delay after map start at which all movers should be spawned

/// `START_TIME_REMOVE_ENTS`.
///
/// Raven: time-delay after map start to remove temporary ents.
/// Source: `oracle/oracle/codemp/game/g_local.h:48`
pub const START_TIME_REMOVE_ENTS: c_int = FRAMETIME * 3; // time-delay after map start to remove temporary ents

/// `START_TIME_NAV_CALC`.
///
/// Raven: time-delay after map start to connect waypoints and calc routes.
/// Source: `oracle/oracle/codemp/game/g_local.h:49`
pub const START_TIME_NAV_CALC: c_int = FRAMETIME * 4; // time-delay after map start to connect waypoints and calc routes

/// `START_TIME_FIND_WAYPOINT`.
///
/// Raven: time-delay after map start after which it's okay to try to find your
/// best waypoint.
/// Source: `oracle/oracle/codemp/game/g_local.h:50`
pub const START_TIME_FIND_WAYPOINT: c_int = FRAMETIME * 5; // time-delay after map start after which it's okay to try to find your best waypoint

/// `MAX_G_SHARED_BUFFER_SIZE`.
///
/// Source: `oracle/oracle/codemp/game/g_local.h:85`
pub const MAX_G_SHARED_BUFFER_SIZE: usize = 8192;

/// `SP_PODIUM_MODEL`.
///
/// Source: `oracle/oracle/codemp/game/g_local.h:96`
pub const SP_PODIUM_MODEL: &str = "models/mapobjects/podium/podium4.md3";

// Type modules — copied from the oracle as faithful starting material (Raven comments
// retained). Not yet wired into the build: they reference dependency types
// (entityState_t, gitem_t, AIGroupInfo_t, …) pending port into shared/, bg/, boundary/.
// Migration target: src/modules/mp/game/{client,entity,level}.
// pub mod client;
// pub mod entity;
// pub mod level;
