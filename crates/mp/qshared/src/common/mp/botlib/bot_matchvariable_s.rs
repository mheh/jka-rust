#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

/// Raven `bot_matchvariable_t` — a matched variable within a bot chat match.
///
/// Type definition source: `oracle/oracle/codemp/game/be_ai_chat.h:39-43`
#[repr(C)]
pub struct bot_matchvariable_t {
	pub offset: c_char,
	pub length: i32,
}

pub type bot_matchvariable_s = bot_matchvariable_t;

const _: () = assert!(core::mem::size_of::<bot_matchvariable_t>() == 8);
const _: () = assert!(core::mem::offset_of!(bot_matchvariable_t, offset) == 0);
const _: () = assert!(core::mem::offset_of!(bot_matchvariable_t, length) == 4);
