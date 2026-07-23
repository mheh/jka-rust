#![allow(non_camel_case_types, non_snake_case)]

use super::bot_chatmessage_s::BotChatMessage;
use super::bot_replychatkey_s::BotReplyChatKey;

/// Raven `bot_replychat_t` — a reply chat (keys plus reply messages).
///
/// Redesigned (porting-rules §F17): the `keys` and `firstchatmessage` chains
/// become owned `Vec`s and `next` becomes `BotLib.replychats`'s `Vec`. Raven
/// prepends reply chats, keys, and messages, so the loader inserts each at the
/// front — the resulting iteration order drives the reply RNG stream and the
/// priority tie-break, so it is preserved exactly. `numchatmessages` is
/// `chatmessages.len()`.
///
/// (Namespace note: the identically-named exported `fn BotReplyChat` lives in
/// the value namespace, so this type name does not clash.)
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:142-149`
#[derive(Default)]
pub struct BotReplyChat {
    pub keys: Vec<BotReplyChatKey>,
    pub priority: f32,
    pub chatmessages: Vec<BotChatMessage>,
}
