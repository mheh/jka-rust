//! MP `b_public.h` `NPCInfo->aiFlags` bit values.
//!
//! Plain `#define` bit flags (not an enum), so §C8 makes them `const`s
//! directly.
//!
//! Source: `oracle/codemp/game/b_public.h:6-24`

use core::ffi::c_int;

pub const NPCAI_CHECK_WEAPON: c_int = 0x0000_0001;
pub const NPCAI_BURST_WEAPON: c_int = 0x0000_0002;
pub const NPCAI_MOVING: c_int = 0x0000_0004;
pub const NPCAI_TOUCHED_GOAL: c_int = 0x0000_0008;
pub const NPCAI_PUSHED: c_int = 0x0000_0010;
pub const NPCAI_NO_COLL_AVOID: c_int = 0x0000_0020;
pub const NPCAI_BLOCKED: c_int = 0x0000_0040;
pub const NPCAI_OFF_PATH: c_int = 0x0000_0100;
pub const NPCAI_IN_SQUADPOINT: c_int = 0x0000_0200;
pub const NPCAI_STRAIGHT_TO_DESTPOS: c_int = 0x0000_0400;
pub const NPCAI_NO_SLOWDOWN: c_int = 0x0000_1000;
pub const NPCAI_LOST: c_int = 0x0000_2000; //Can't nav to his goal
pub const NPCAI_SHIELDS: c_int = 0x0000_4000; //Has shields, borg can adapt
pub const NPCAI_GREET_ALLIES: c_int = 0x0000_8000; //Say hi to nearby allies
pub const NPCAI_FORM_TELE_NAV: c_int = 0x0001_0000; //Tells formation people to use nav info to get to
pub const NPCAI_ENROUTE_TO_HOMEWP: c_int = 0x0002_0000; //Lets us know to run our lostenemyscript when we get to homeWp
pub const NPCAI_MATCHPLAYERWEAPON: c_int = 0x0004_0000; //Match the player's weapon except when it changes during cinematics
pub const NPCAI_DIE_ON_IMPACT: c_int = 0x0010_0000; //Next time you crashland, die!
pub const NPCAI_CUSTOM_GRAVITY: c_int = 0x0020_0000; //Don't use g_gravity, I fly!
