//! Raven `NPC_FindCombatPoint` request flags (`CP_*`) and `combatPoint_t::flags` bits (`CPF_*`).
//!
//! Type definition source: `oracle/codemp/game/b_local.h:243-269`

use core::ffi::c_int;

/// Raven `CP_ANY`: no flags.
///
/// Source: `oracle/codemp/game/b_local.h:243`
pub const CP_ANY: c_int = 0;
/// Raven `CP_COVER`: the enemy cannot currently shoot this position.
///
/// Source: `oracle/codemp/game/b_local.h:244`
pub const CP_COVER: c_int = 0x00000001;
/// Raven `CP_CLEAR`: this cover point has a clear shot to the enemy.
///
/// Source: `oracle/codemp/game/b_local.h:245`
pub const CP_CLEAR: c_int = 0x00000002;
/// Raven `CP_FLEE`: this cover point is marked as a flee point.
///
/// Source: `oracle/codemp/game/b_local.h:246`
pub const CP_FLEE: c_int = 0x00000004;
/// Raven `CP_DUCK`: this cover point is marked as a duck point.
///
/// Source: `oracle/codemp/game/b_local.h:247`
pub const CP_DUCK: c_int = 0x00000008;
/// Raven `CP_NEAREST`: find the nearest combat point.
///
/// Source: `oracle/codemp/game/b_local.h:248`
pub const CP_NEAREST: c_int = 0x00000010;
/// Raven `CP_AVOID_ENEMY`: avoid our enemy.
///
/// Source: `oracle/codemp/game/b_local.h:249`
pub const CP_AVOID_ENEMY: c_int = 0x00000020;
/// Raven `CP_INVESTIGATE`: a special point worth enemy investigation if searching.
///
/// Source: `oracle/codemp/game/b_local.h:250`
pub const CP_INVESTIGATE: c_int = 0x00000040;
/// Raven `CP_SQUAD`: squad path.
///
/// Source: `oracle/codemp/game/b_local.h:251`
pub const CP_SQUAD: c_int = 0x00000080;
/// Raven `CP_AVOID`: avoid supplied position.
///
/// Source: `oracle/codemp/game/b_local.h:252`
pub const CP_AVOID: c_int = 0x00000100;
/// Raven `CP_APPROACH_ENEMY`: try to get closer to enemy.
///
/// Source: `oracle/codemp/game/b_local.h:253`
pub const CP_APPROACH_ENEMY: c_int = 0x00000200;
/// Raven `CP_CLOSEST`: take the closest combatPoint to the enemy that's available.
///
/// Source: `oracle/codemp/game/b_local.h:254`
pub const CP_CLOSEST: c_int = 0x00000400;
/// Raven `CP_FLANK`: pick a combatPoint behind the enemy.
///
/// Source: `oracle/codemp/game/b_local.h:255`
pub const CP_FLANK: c_int = 0x00000800;
/// Raven `CP_HAS_ROUTE`: pick a combatPoint that we have a route to.
///
/// Source: `oracle/codemp/game/b_local.h:256`
pub const CP_HAS_ROUTE: c_int = 0x00001000;
/// Raven `CP_SNIPE`: pick a combatPoint that is marked as a sniper spot.
///
/// Source: `oracle/codemp/game/b_local.h:257`
pub const CP_SNIPE: c_int = 0x00002000;
/// Raven `CP_SAFE`: pick a combatPoint with no danger time.
///
/// Source: `oracle/codemp/game/b_local.h:258`
pub const CP_SAFE: c_int = 0x00004000;
/// Raven `CP_HORZ_DIST_COLL`: collect combat points within *horizontal* dist.
///
/// Source: `oracle/codemp/game/b_local.h:259`
pub const CP_HORZ_DIST_COLL: c_int = 0x00008000;
/// Raven `CP_NO_PVS`: a combat point out of the PVS of enemy pos.
///
/// Source: `oracle/codemp/game/b_local.h:260`
pub const CP_NO_PVS: c_int = 0x00010000;
/// Raven `CP_RETREAT`: try to get farther from enemy.
///
/// Source: `oracle/codemp/game/b_local.h:261`
pub const CP_RETREAT: c_int = 0x00020000;

/// Raven `CPF_NONE`: no `combatPoint_t::flags` bits set.
///
/// Source: `oracle/codemp/game/b_local.h:263`
pub const CPF_NONE: c_int = 0;
/// Raven `CPF_DUCK`: `combatPoint_t::flags` bit.
///
/// Source: `oracle/codemp/game/b_local.h:264`
pub const CPF_DUCK: c_int = 0x00000001;
/// Raven `CPF_FLEE`: `combatPoint_t::flags` bit.
///
/// Source: `oracle/codemp/game/b_local.h:265`
pub const CPF_FLEE: c_int = 0x00000002;
/// Raven `CPF_INVESTIGATE`: `combatPoint_t::flags` bit.
///
/// Source: `oracle/codemp/game/b_local.h:266`
pub const CPF_INVESTIGATE: c_int = 0x00000004;
/// Raven `CPF_SQUAD`: `combatPoint_t::flags` bit.
///
/// Source: `oracle/codemp/game/b_local.h:267`
pub const CPF_SQUAD: c_int = 0x00000008;
/// Raven `CPF_LEAN`: `combatPoint_t::flags` bit.
///
/// Source: `oracle/codemp/game/b_local.h:268`
pub const CPF_LEAN: c_int = 0x00000010;
/// Raven `CPF_SNIPE`: `combatPoint_t::flags` bit.
///
/// Source: `oracle/codemp/game/b_local.h:269`
pub const CPF_SNIPE: c_int = 0x00000020;
