#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

/// Raven `bot_stringlist_t` — a string list node.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:152-156`
#[repr(C)]
pub struct bot_stringlist_t {
	pub string: *mut c_char,
	pub next: *mut bot_stringlist_t,
}

pub type bot_stringlist_s = bot_stringlist_t;

const _: () = assert!(core::mem::size_of::<bot_stringlist_t>() == 16);
const _: () = assert!(core::mem::offset_of!(bot_stringlist_t, string) == 0);
const _: () = assert!(core::mem::offset_of!(bot_stringlist_t, next) == 8);
