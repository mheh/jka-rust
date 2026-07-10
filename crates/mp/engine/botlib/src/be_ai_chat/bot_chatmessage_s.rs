#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

/// Raven `bot_chatmessage_t` — a single chat message line in a chat type.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:57-62`
#[repr(C)]
pub struct bot_chatmessage_t {
	pub chatmessage: *mut c_char,
	pub time: f32,
	pub next: *mut bot_chatmessage_t,
}

pub type bot_chatmessage_s = bot_chatmessage_t;

const _: () = assert!(core::mem::size_of::<bot_chatmessage_t>() == 24);
const _: () = assert!(core::mem::offset_of!(bot_chatmessage_t, chatmessage) == 0);
const _: () = assert!(core::mem::offset_of!(bot_chatmessage_t, time) == 8);
const _: () = assert!(core::mem::offset_of!(bot_chatmessage_t, next) == 16);
