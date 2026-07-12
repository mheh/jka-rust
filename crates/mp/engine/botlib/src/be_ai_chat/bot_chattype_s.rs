#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use super::bot_chatmessage_s::bot_chatmessage_t;
use super::chat_consts::MAX_CHATTYPE_NAME;

/// Raven `bot_chattype_t` — a named group of chat messages.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:64-70`
#[repr(C)]
pub struct bot_chattype_t {
    pub name: [c_char; MAX_CHATTYPE_NAME],
    pub numchatmessages: i32,
    pub firstchatmessage: *mut bot_chatmessage_t,
    pub next: *mut bot_chattype_t,
}

pub type bot_chattype_s = bot_chattype_t;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<bot_chattype_t>() == 56);
    assert!(core::mem::offset_of!(bot_chattype_t, name) == 0);
    assert!(core::mem::offset_of!(bot_chattype_t, numchatmessages) == 32);
    assert!(core::mem::offset_of!(bot_chattype_t, firstchatmessage) == 40);
    assert!(core::mem::offset_of!(bot_chattype_t, next) == 48);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<bot_chattype_t>() == 44);
    assert!(core::mem::offset_of!(bot_chattype_t, name) == 0);
    assert!(core::mem::offset_of!(bot_chattype_t, numchatmessages) == 32);
    assert!(core::mem::offset_of!(bot_chattype_t, firstchatmessage) == 36);
    assert!(core::mem::offset_of!(bot_chattype_t, next) == 40);
};
