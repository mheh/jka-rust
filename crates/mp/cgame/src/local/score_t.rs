#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::qboolean;

/// Raven `score_t` — scoreboard entry sent from server-side score commands.
///
/// Type definition source: `oracle/codemp/cgame/cg_local.h:630-646`
#[repr(C)]
pub struct score_t {
	pub client: i32,
	pub score: i32,
	pub ping: i32,
	pub time: i32,
	pub scoreFlags: i32,
	pub powerUps: i32,
	pub accuracy: i32,
	pub impressiveCount: i32,
	pub excellentCount: i32,
	pub guantletCount: i32,
	pub defendCount: i32,
	pub assistCount: i32,
	pub captures: i32,
	pub perfect: qboolean,
	pub team: i32,
}

const _: () = assert!(core::mem::size_of::<score_t>() == 60);
const _: () = assert!(core::mem::offset_of!(score_t, client) == 0);
const _: () = assert!(core::mem::offset_of!(score_t, score) == 4);
const _: () = assert!(core::mem::offset_of!(score_t, ping) == 8);
const _: () = assert!(core::mem::offset_of!(score_t, time) == 12);
const _: () = assert!(core::mem::offset_of!(score_t, scoreFlags) == 16);
const _: () = assert!(core::mem::offset_of!(score_t, powerUps) == 20);
const _: () = assert!(core::mem::offset_of!(score_t, accuracy) == 24);
const _: () = assert!(core::mem::offset_of!(score_t, impressiveCount) == 28);
const _: () = assert!(core::mem::offset_of!(score_t, excellentCount) == 32);
const _: () = assert!(core::mem::offset_of!(score_t, guantletCount) == 36);
const _: () = assert!(core::mem::offset_of!(score_t, defendCount) == 40);
const _: () = assert!(core::mem::offset_of!(score_t, assistCount) == 44);
const _: () = assert!(core::mem::offset_of!(score_t, captures) == 48);
const _: () = assert!(core::mem::offset_of!(score_t, perfect) == 52);
const _: () = assert!(core::mem::offset_of!(score_t, team) == 56);
