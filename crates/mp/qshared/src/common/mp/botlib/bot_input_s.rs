#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_float, c_int};

use crate::shared::vec3_t;

/// Raven `bot_input_t` — bot movement/view input for a single AI frame.
///
/// Type definition source: `oracle/oracle/codemp/game/botlib.h:93-101`
#[repr(C)]
pub struct bot_input_t {
	/// time since last output (in seconds)
	pub thinktime: c_float,
	/// movement direction
	pub dir: vec3_t,
	/// speed in the range [0, 400]
	pub speed: c_float,
	/// the view angles
	pub viewangles: vec3_t,
	/// one of the ACTION_? flags
	pub actionflags: c_int,
	/// weapon to use
	pub weapon: c_int,
}

pub type bot_input_s = bot_input_t;

const _: () = assert!(core::mem::size_of::<bot_input_t>() == 40);
const _: () = assert!(core::mem::offset_of!(bot_input_t, thinktime) == 0);
const _: () = assert!(core::mem::offset_of!(bot_input_t, dir) == 4);
const _: () = assert!(core::mem::offset_of!(bot_input_t, speed) == 16);
const _: () = assert!(core::mem::offset_of!(bot_input_t, viewangles) == 20);
const _: () = assert!(core::mem::offset_of!(bot_input_t, actionflags) == 32);
const _: () = assert!(core::mem::offset_of!(bot_input_t, weapon) == 36);
