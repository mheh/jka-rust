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

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<bot_chatmessage_t>() == 24);
    assert!(core::mem::offset_of!(bot_chatmessage_t, chatmessage) == 0);
    assert!(core::mem::offset_of!(bot_chatmessage_t, time) == 8);
    assert!(core::mem::offset_of!(bot_chatmessage_t, next) == 16);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<bot_chatmessage_t>() == 12);
    assert!(core::mem::offset_of!(bot_chatmessage_t, chatmessage) == 0);
    assert!(core::mem::offset_of!(bot_chatmessage_t, time) == 4);
    assert!(core::mem::offset_of!(bot_chatmessage_t, next) == 8);
};
