#![allow(non_camel_case_types, non_snake_case)]

use crate::shared::vec3_t;

/// Raven `bot_moveresult_t` — result of a bot movement prediction/execution.
///
/// Type definition source: `oracle/codemp/game/be_ai_move.h:74-85`
#[repr(C)]
pub struct bot_moveresult_t {
	pub failure: i32,             // true if movement failed all together
	pub r#type: i32,              // failure or blocked type
	pub blocked: i32,             // true if blocked by an entity
	pub blockentity: i32,         // entity blocking the bot
	pub traveltype: i32,          // last executed travel type
	pub flags: i32,               // result flags
	pub weapon: i32,              // weapon used for movement
	pub movedir: vec3_t,          // movement direction
	pub ideal_viewangles: vec3_t, // ideal viewangles for the movement
}

/// The oracle tag is `bot_moveresult_s`; alias it so siblings referencing the
/// tag name stay green.
pub type bot_moveresult_s = bot_moveresult_t;

const _: () = assert!(core::mem::size_of::<bot_moveresult_t>() == 52);
const _: () = assert!(core::mem::offset_of!(bot_moveresult_t, failure) == 0);
const _: () = assert!(core::mem::offset_of!(bot_moveresult_t, r#type) == 4);
const _: () = assert!(core::mem::offset_of!(bot_moveresult_t, blocked) == 8);
const _: () = assert!(core::mem::offset_of!(bot_moveresult_t, blockentity) == 12);
const _: () = assert!(core::mem::offset_of!(bot_moveresult_t, traveltype) == 16);
const _: () = assert!(core::mem::offset_of!(bot_moveresult_t, flags) == 20);
const _: () = assert!(core::mem::offset_of!(bot_moveresult_t, weapon) == 24);
const _: () = assert!(core::mem::offset_of!(bot_moveresult_t, movedir) == 28);
const _: () = assert!(core::mem::offset_of!(bot_moveresult_t, ideal_viewangles) == 40);
