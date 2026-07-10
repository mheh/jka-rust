#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use super::bot_matchpiece_s::bot_matchpiece_t;

/// Raven `bot_replychatkey_t` — a reply-chat key.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:134-140`
#[repr(C)]
pub struct bot_replychatkey_t {
	pub flags: i32,
	pub string: *mut c_char,
	pub r#match: *mut bot_matchpiece_t,
	pub next: *mut bot_replychatkey_t,
}

pub type bot_replychatkey_s = bot_replychatkey_t;

const _: () = assert!(core::mem::size_of::<bot_replychatkey_t>() == 32);
const _: () = assert!(core::mem::offset_of!(bot_replychatkey_t, flags) == 0);
const _: () = assert!(core::mem::offset_of!(bot_replychatkey_t, string) == 8);
const _: () = assert!(core::mem::offset_of!(bot_replychatkey_t, r#match) == 16);
const _: () = assert!(core::mem::offset_of!(bot_replychatkey_t, next) == 24);
