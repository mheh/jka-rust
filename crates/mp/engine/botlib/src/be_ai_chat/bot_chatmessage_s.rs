#![allow(non_camel_case_types, non_snake_case)]

/// Raven `bot_chatmessage_t` — a single chat message line (with `\x01v`/`\x01r`
/// escape sequences still embedded), shared by chat types and reply chats.
///
/// Redesigned (porting-rules §F17): Raven's malloc'd `char *chatmessage` +
/// `next` pointer become an owned `String` and membership in the parent's
/// `Vec<BotChatMessage>`.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:57-62`
#[derive(Default, Clone)]
pub struct BotChatMessage {
    pub chatmessage: String,
    pub time: f32,
}
