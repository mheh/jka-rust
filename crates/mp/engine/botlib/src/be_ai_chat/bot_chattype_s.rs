#![allow(non_camel_case_types, non_snake_case)]

use super::bot_chatmessage_s::BotChatMessage;

/// Raven `bot_chattype_t` — a named group of chat messages.
///
/// Redesigned (porting-rules §F17): Raven's fixed `char name[MAX_CHATTYPE_NAME]`
/// becomes an owned `String` (still truncated to `MAX_CHATTYPE_NAME - 1` at
/// load, matching `Q_strncpyz`), the `firstchatmessage` chain becomes a
/// `Vec<BotChatMessage>` (Raven prepends, so the loader inserts at the front),
/// and `next` becomes the parent `BotChat`'s `Vec`. `numchatmessages` is
/// `chatmessages.len()`.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:64-70`
#[derive(Default)]
pub struct BotChatType {
    pub name: String,
    pub chatmessages: Vec<BotChatMessage>,
}
