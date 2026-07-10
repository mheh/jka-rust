#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

/// Raven `bot_matchstring_t` — a fixed match string list node.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:109-113`
#[repr(C)]
pub struct bot_matchstring_t {
	pub string: *mut c_char,
	pub next: *mut bot_matchstring_t,
}

pub type bot_matchstring_s = bot_matchstring_t;

const _: () = assert!(core::mem::size_of::<bot_matchstring_t>() == 16);
const _: () = assert!(core::mem::offset_of!(bot_matchstring_t, string) == 0);
const _: () = assert!(core::mem::offset_of!(bot_matchstring_t, next) == 8);
