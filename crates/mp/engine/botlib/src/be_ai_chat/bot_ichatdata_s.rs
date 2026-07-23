#![allow(non_camel_case_types, non_snake_case)]

use super::bot_chat_s::BotChatHandle;

/// Raven `bot_ichatdata_t` — a cached initial-chat file entry (one per client
/// slot in `ichatdata[]`), keyed by `filename`/`chatname`.
///
/// Redesigned (porting-rules §F17): the fixed `char filename[MAX_QPATH]` /
/// `char chatname[MAX_QPATH]` become owned `String`s (truncated to
/// `MAX_QPATH - 1` at store, matching `Q_strncpyz`), and `chat` becomes a
/// `BotChatHandle` into the `BotLib.botchats` arena — the same handle the
/// sharing `bot_chatstate_t.chat` holds in the cached path.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:175-179`
#[derive(Default)]
pub struct BotIChatData {
    pub chat: Option<BotChatHandle>,
    pub filename: String,
    pub chatname: String,
}
