#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_qshared::common::mp::botlib::bot_consolemessage_s::bot_consolemessage_t;

use super::bot_chat_s::bot_chat_t;

/// `MAX_MESSAGE_SIZE`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:53`
pub const MAX_MESSAGE_SIZE: usize = 256;

/// Raven `bot_chatstate_t` — the chat state of a single bot.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:159-173`
#[repr(C)]
pub struct bot_chatstate_t {
    /// 0=it, 1=female, 2=male
    pub gender: i32,
    /// client number
    pub client: i32,
    /// name of the bot
    pub name: [c_char; 32],
    pub chatmessage: [c_char; MAX_MESSAGE_SIZE],
    pub handle: i32,
    /// first message is the first typed message
    pub firstmessage: *mut bot_consolemessage_t,
    /// last message is the last typed message, bottom of console
    pub lastmessage: *mut bot_consolemessage_t,
    /// number of console messages stored in the state
    pub numconsolemessages: i32,
    /// the bot chat lines
    pub chat: *mut bot_chat_t,
}

pub type bot_chatstate_s = bot_chatstate_t;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<bot_chatstate_t>() == 336);
    assert!(core::mem::offset_of!(bot_chatstate_t, gender) == 0);
    assert!(core::mem::offset_of!(bot_chatstate_t, client) == 4);
    assert!(core::mem::offset_of!(bot_chatstate_t, name) == 8);
    assert!(core::mem::offset_of!(bot_chatstate_t, chatmessage) == 40);
    assert!(core::mem::offset_of!(bot_chatstate_t, handle) == 296);
    assert!(core::mem::offset_of!(bot_chatstate_t, firstmessage) == 304);
    assert!(core::mem::offset_of!(bot_chatstate_t, lastmessage) == 312);
    assert!(core::mem::offset_of!(bot_chatstate_t, numconsolemessages) == 320);
    assert!(core::mem::offset_of!(bot_chatstate_t, chat) == 328);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<bot_chatstate_t>() == 316);
    assert!(core::mem::offset_of!(bot_chatstate_t, gender) == 0);
    assert!(core::mem::offset_of!(bot_chatstate_t, client) == 4);
    assert!(core::mem::offset_of!(bot_chatstate_t, name) == 8);
    assert!(core::mem::offset_of!(bot_chatstate_t, chatmessage) == 40);
    assert!(core::mem::offset_of!(bot_chatstate_t, handle) == 296);
    assert!(core::mem::offset_of!(bot_chatstate_t, firstmessage) == 300);
    assert!(core::mem::offset_of!(bot_chatstate_t, lastmessage) == 304);
    assert!(core::mem::offset_of!(bot_chatstate_t, numconsolemessages) == 308);
    assert!(core::mem::offset_of!(bot_chatstate_t, chat) == 312);
};
