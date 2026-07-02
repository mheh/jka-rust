#![allow(non_camel_case_types, non_snake_case)]

use crate::shared::vec3_t;

/// Raven `bot_initmove_t` — initial movement state for the bot movement code.
///
/// Type definition source: `oracle/oracle/codemp/game/be_ai_move.h:60-71`
#[repr(C)]
pub struct bot_initmove_t {
	pub origin: vec3_t,       //origin of the bot
	pub velocity: vec3_t,     //velocity of the bot
	pub viewoffset: vec3_t,   //view offset
	pub entitynum: i32,       //entity number of the bot
	pub client: i32,          //client number of the bot
	pub thinktime: f32,       //time the bot thinks
	pub presencetype: i32,    //presencetype of the bot
	pub viewangles: vec3_t,   //view angles of the bot
	pub or_moveflags: i32,    //values ored to the movement flags
}

pub type bot_initmove_s = bot_initmove_t;

const _: () = assert!(core::mem::size_of::<bot_initmove_t>() == 68);
const _: () = assert!(core::mem::offset_of!(bot_initmove_t, origin) == 0);
const _: () = assert!(core::mem::offset_of!(bot_initmove_t, velocity) == 12);
const _: () = assert!(core::mem::offset_of!(bot_initmove_t, viewoffset) == 24);
const _: () = assert!(core::mem::offset_of!(bot_initmove_t, entitynum) == 36);
const _: () = assert!(core::mem::offset_of!(bot_initmove_t, client) == 40);
const _: () = assert!(core::mem::offset_of!(bot_initmove_t, thinktime) == 44);
const _: () = assert!(core::mem::offset_of!(bot_initmove_t, presencetype) == 48);
const _: () = assert!(core::mem::offset_of!(bot_initmove_t, viewangles) == 52);
const _: () = assert!(core::mem::offset_of!(bot_initmove_t, or_moveflags) == 64);
