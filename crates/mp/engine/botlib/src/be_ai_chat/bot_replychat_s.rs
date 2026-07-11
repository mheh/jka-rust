#![allow(non_camel_case_types, non_snake_case)]

use super::bot_chatmessage_s::bot_chatmessage_t;
use super::bot_replychatkey_s::bot_replychatkey_t;

/// Raven `bot_replychat_t` — a reply chat (keys plus reply messages).
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:142-149`
#[repr(C)]
pub struct bot_replychat_t {
    pub keys: *mut bot_replychatkey_t,
    pub priority: f32,
    pub numchatmessages: i32,
    pub firstchatmessage: *mut bot_chatmessage_t,
    pub next: *mut bot_replychat_t,
}

pub type bot_replychat_s = bot_replychat_t;

const _: () = assert!(core::mem::size_of::<bot_replychat_t>() == 32);
const _: () = assert!(core::mem::offset_of!(bot_replychat_t, keys) == 0);
const _: () = assert!(core::mem::offset_of!(bot_replychat_t, priority) == 8);
const _: () = assert!(core::mem::offset_of!(bot_replychat_t, numchatmessages) == 12);
const _: () = assert!(core::mem::offset_of!(bot_replychat_t, firstchatmessage) == 16);
const _: () = assert!(core::mem::offset_of!(bot_replychat_t, next) == 24);
