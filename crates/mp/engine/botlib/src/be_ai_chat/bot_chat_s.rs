#![allow(non_camel_case_types, non_snake_case)]

use super::bot_chattype_s::bot_chattype_t;

/// Raven `bot_chat_t` — the set of chat types loaded for a bot.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:72-75`
#[repr(C)]
pub struct bot_chat_t {
    pub types: *mut bot_chattype_t,
}

pub type bot_chat_s = bot_chat_t;

const _: () = assert!(core::mem::size_of::<bot_chat_t>() == 8);
const _: () = assert!(core::mem::offset_of!(bot_chat_t, types) == 0);
