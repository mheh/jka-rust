#![allow(non_camel_case_types, non_snake_case)]

//! SP `level_locals_t` — the world container.
//!
//! Type definition source: `oracle/oracle/code/game/g_local.h:161-220`

use core::ffi::{c_char, c_int};

use sp_qshared::common::sp::gentity::gentity_t;
use sp_qshared::shared::{fileHandle_t, qboolean, vec3_t, MAX_QPATH};

use crate::ai::group_info::AIGroupInfo_t;
use crate::shared::gclient_s::gclient_t;
use crate::ai::consts::MAX_FRAME_GROUPS;

use super::alert_event_s::alertEvent_t;
use super::anim_file_set_t::animFileSet_t;
use super::combat_point_t::combatPoint_t;
use super::interest_point_t::interestPoint_t;

/// Raven `MAX_ALERT_EVENTS`.
///
/// Source: `oracle/oracle/code/game/g_local.h:107`
pub const MAX_ALERT_EVENTS: usize = 32;

/// Raven `MAX_ANIM_FILES` (non-`_XBOX` build; jka-rust targets desktop only).
///
/// Source: `oracle/oracle/code/game/bg_public.h:480-482`
pub const MAX_ANIM_FILES: usize = 16;

/// Raven `MAX_INTEREST_POINTS`.
///
/// Source: `oracle/oracle/code/game/g_local.h:82`
pub const MAX_INTEREST_POINTS: usize = 64;

/// Raven `MAX_COMBAT_POINTS`.
///
/// Source: `oracle/oracle/code/game/g_local.h:92`
pub const MAX_COMBAT_POINTS: usize = 512;

/// Raven `level_locals_t` — game-internal world state; cleared as each map is
/// entered.
///
/// Raven: NOTE!!!!!! The only things beyond `LEVEL_LOCALS_T_SAVESTOP` (`logFile`)
/// in the structure should be the ones you do NOT wish to be affected by loading
/// saved-games. Since loading a game first starts the map and then loads over
/// things like entities etc then these fields are usually the ones setup by the
/// map loader. If they ever get modified in-game let me know and I'll include
/// them in the save. -Ste
///
/// Pointer-bearing => arch-dependent; asserts pin the host-64-bit layout.
///
/// Type definition source: `oracle/oracle/code/game/g_local.h:161-220`
#[repr(C)]
pub struct level_locals_t {
	/// `[maxclients]`
	pub clients: *mut gclient_t,

	// store latched cvars here that we want to get at often
	pub maxclients: c_int,

	pub framenum: c_int,
	pub time: c_int,         // in msec
	pub previousTime: c_int, // so movers can back up when blocked

	pub globalTime: c_int, // global time at level initialization

	pub mapname: [c_char; MAX_QPATH], // the server name (base1, etc)

	pub locationLinked: qboolean, // target_locations get linked
	pub locationHead: *mut gentity_t, // head of the location list

	pub alertEvents: [alertEvent_t; MAX_ALERT_EVENTS],
	pub numAlertEvents: c_int,
	pub curAlertID: c_int,

	pub groups: [AIGroupInfo_t; MAX_FRAME_GROUPS],

	pub knownAnimFileSets: [animFileSet_t; MAX_ANIM_FILES],
	pub numKnownAnimFileSets: c_int,

	pub worldFlags: c_int,

	pub dmState: c_int, //actually, we do want save/load the dynamic music state
	// =====================================
	//
	// NOTE!!!!!!   The only things beyond this point in the structure should be the ones you do NOT wish to be
	//              affected by loading saved-games. Since loading a game first starts the map and then loads
	//              over things like entities etc then these fields are usually the ones setup by the map loader.
	//              If they ever get modified in-game let me know and I'll include them in the save. -Ste
	//
	pub logFile: fileHandle_t,

	//Interest points- squadmates automatically look at these if standing around and close to them
	pub interestPoints: [interestPoint_t; MAX_INTEREST_POINTS],
	pub numInterestPoints: c_int,

	//Combat points- NPCs in bState BS_COMBAT_POINT will find their closest empty combat_point
	pub combatPoints: [combatPoint_t; MAX_COMBAT_POINTS],
	pub numCombatPoints: c_int,
	pub spawntarget: [c_char; MAX_QPATH], // the targetname of the spawnpoint you want the player to start at

	pub dmDebounceTime: c_int,
	pub dmBeatTime: c_int,

	pub mNumBSPInstances: c_int,
	pub mBSPInstanceDepth: c_int,
	pub mOriginAdjust: vec3_t,
	pub mRotationAdjust: f32,
	pub mTargetAdjust: *mut c_char,
	pub hasBspInstances: qboolean,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<level_locals_t>() == 620536);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, clients) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, maxclients) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, framenum) == 12);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, time) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, previousTime) == 20);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, globalTime) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, mapname) == 28);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, locationLinked) == 92);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, locationHead) == 96);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, alertEvents) == 104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, numAlertEvents) == 1896);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, curAlertID) == 1900);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, groups) == 1904);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, knownAnimFileSets) == 21872);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, numKnownAnimFileSets) == 604528);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, worldFlags) == 604532);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, dmState) == 604536);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, logFile) == 604540);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, interestPoints) == 604544);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, numInterestPoints) == 606080);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, combatPoints) == 606084);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, numCombatPoints) == 620420);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, spawntarget) == 620424);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, dmDebounceTime) == 620488);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, dmBeatTime) == 620492);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, mNumBSPInstances) == 620496);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, mBSPInstanceDepth) == 620500);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, mOriginAdjust) == 620504);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, mRotationAdjust) == 620516);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, mTargetAdjust) == 620520);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, hasBspInstances) == 620528);

// The STATE-D9 zeroed-construction contract (round-5 STATE-Q10 resolution):
// all-zero bytes are a valid level_locals_t — the same property the layout asserts above
// pin and Raven's memset/static zero-init relies on.
// Source: oracle/oracle/code/game/g_local.h (all-zero-valid #[repr(C)]; SP zero-inits `level`, g_main.cpp:46)
unsafe impl native_platform::ZeroValid for level_locals_t {}
