#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::{qboolean, vec3_t};

/// Raven `combatPoint_t` — an AI combat waypoint (cover/tactical) point.
///
/// Type definition source: `oracle/oracle/code/game/g_local.h:94-103`
#[repr(C)]
pub struct combatPoint_t {
	pub origin: vec3_t,
	pub flags: i32,
	//	char		*NPC_targetname;
	//	team_t		team;
	pub occupied: qboolean,
	pub waypoint: i32,
	pub dangerTime: i32,
}

const _: () = assert!(core::mem::size_of::<combatPoint_t>() == 28);
const _: () = assert!(core::mem::offset_of!(combatPoint_t, origin) == 0);
const _: () = assert!(core::mem::offset_of!(combatPoint_t, flags) == 12);
const _: () = assert!(core::mem::offset_of!(combatPoint_t, occupied) == 16);
const _: () = assert!(core::mem::offset_of!(combatPoint_t, waypoint) == 20);
const _: () = assert!(core::mem::offset_of!(combatPoint_t, dangerTime) == 24);
