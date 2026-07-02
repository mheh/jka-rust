#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_float, c_int};

/// Raven `bot_consolemessage_t` — a console message queued for a bot's chat
/// state, in a doubly linked list.
///
/// Type definition source: `oracle/oracle/codemp/game/be_ai_chat.h:29-36`
#[repr(C)]
pub struct bot_consolemessage_t {
    pub handle: c_int,
    pub time: c_float, //message time
    pub r#type: c_int, //message type
    // MAX_MESSAGE_SIZE == 256. Source: oracle/oracle/codemp/game/be_ai_chat.h:16
    pub message: [c_char; 256], //message
    pub prev: *mut bot_consolemessage_t, //prev and next in list
    pub next: *mut bot_consolemessage_t,
}

/// Raven `bot_consolemessage_s` tag alias (`bot_consolemessage_t`'s C struct tag).
pub type bot_consolemessage_s = bot_consolemessage_t;

const _: () = assert!(core::mem::size_of::<bot_consolemessage_t>() == 288);
const _: () = assert!(core::mem::offset_of!(bot_consolemessage_t, handle) == 0);
const _: () = assert!(core::mem::offset_of!(bot_consolemessage_t, time) == 4);
const _: () = assert!(core::mem::offset_of!(bot_consolemessage_t, r#type) == 8);
const _: () = assert!(core::mem::offset_of!(bot_consolemessage_t, message) == 12);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(bot_consolemessage_t, prev) == 272);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(bot_consolemessage_t, next) == 280);
