//! MP `FL_*` game entity flags (`gentity->flags`).
//!
//! Source: `oracle/oracle/codemp/game/g_local.h:52-59`

use core::ffi::c_int;

pub const FL_GODMODE: c_int = 0x00000010;
pub const FL_NOTARGET: c_int = 0x00000020;
pub const FL_TEAMSLAVE: c_int = 0x00000400; // not the first on the team
pub const FL_NO_KNOCKBACK: c_int = 0x00000800;
pub const FL_DROPPED_ITEM: c_int = 0x00001000;
pub const FL_NO_BOTS: c_int = 0x00002000; // spawn point not for bot use
pub const FL_NO_HUMANS: c_int = 0x00004000; // spawn point just for bots
pub const FL_FORCE_GESTURE: c_int = 0x00008000; // force gesture on client
pub const FL_INACTIVE: c_int = 0x00010000; // inactive
pub const FL_NAVGOAL: c_int = 0x00020000; // for npc nav stuff
pub const FL_DONT_SHOOT: c_int = 0x00040000;
pub const FL_SHIELDED: c_int = 0x00080000;
pub const FL_UNDYING: c_int = 0x00100000; // takes damage down to 1, but never dies

// ex-eFlags -rww (FL_BOUNCE intentionally shares FL_UNDYING's value in the original)
pub const FL_BOUNCE: c_int = 0x00100000; // for missiles
pub const FL_BOUNCE_HALF: c_int = 0x00200000; // for missiles
pub const FL_BOUNCE_SHRAPNEL: c_int = 0x00400000; // special shrapnel flag

// vehicle game-local stuff -rww
pub const FL_VEH_BOARDING: c_int = 0x00800000;

// breakable flags -rww
pub const FL_DMG_BY_SABER_ONLY: c_int = 0x01000000; // only take dmg from saber
pub const FL_DMG_BY_HEAVY_WEAP_ONLY: c_int = 0x02000000; // only take dmg from explosives

pub const FL_BBRUSH: c_int = 0x04000000; // I am a breakable brush
