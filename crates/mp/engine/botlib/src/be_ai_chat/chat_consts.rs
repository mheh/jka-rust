//! `be_ai_chat.h` chat-type / gender / recipient constants.
//!
//! Source: `oracle/codemp/game/be_ai_chat.h:17-26`

/// Raven `MAX_CHATTYPE_NAME`.
/// Source: `oracle/codemp/game/be_ai_chat.h:17`
pub const MAX_CHATTYPE_NAME: usize = 32;

/// Raven `CHAT_GENDERLESS`.
/// Source: `oracle/codemp/game/be_ai_chat.h:20`
pub const CHAT_GENDERLESS: i32 = 0;
/// Raven `CHAT_GENDERFEMALE`.
/// Source: `oracle/codemp/game/be_ai_chat.h:21`
pub const CHAT_GENDERFEMALE: i32 = 1;
/// Raven `CHAT_GENDERMALE`.
/// Source: `oracle/codemp/game/be_ai_chat.h:22`
pub const CHAT_GENDERMALE: i32 = 2;

/// Raven `CHAT_ALL` — chat message sent to everyone.
/// Source: `oracle/codemp/game/be_ai_chat.h:24`
pub const CHAT_ALL: i32 = 0;
/// Raven `CHAT_TEAM` — chat message sent to the bot's team.
/// Source: `oracle/codemp/game/be_ai_chat.h:25`
pub const CHAT_TEAM: i32 = 1;
/// Raven `CHAT_TELL` — chat message sent to a single client.
/// Source: `oracle/codemp/game/be_ai_chat.h:26`
pub const CHAT_TELL: i32 = 2;
