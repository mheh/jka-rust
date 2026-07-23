#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::botlib::bot_consolemessage_s::bot_consolemessage_t;

use super::bot_chat_s::BotChatHandle;

/// `MAX_MESSAGE_SIZE`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:53`
pub const MAX_MESSAGE_SIZE: usize = 256;

/// Raven `bot_chatstate_t` — the chat state of a single bot.
///
/// Redesigned (porting-rules §F17): the fixed `char name[32]` and
/// `char chatmessage[MAX_MESSAGE_SIZE]` become owned `String`s. `chatmessage`
/// is copied across the seam by `BotGetChatMessage` (the field itself is
/// botlib-internal; the export bounds the copy-out), and the 256-byte
/// truncation Raven applies while *building* it is preserved at the write
/// sites. `chat` becomes an arena `BotChatHandle` (§B5). The
/// `firstmessage`/`lastmessage` console-queue pointers stay raw: they index the
/// `bot_consolemessage_t` pool (a seam-visible type copied out by
/// `BotNextConsoleMessage`), which is retained unchanged.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:159-173`
pub struct BotChatState {
    /// 0=it, 1=female, 2=male
    pub gender: i32,
    /// client number
    pub client: i32,
    /// name of the bot
    pub name: String,
    pub chatmessage: String,
    pub handle: i32,
    /// first message is the first typed message
    pub firstmessage: *mut bot_consolemessage_t,
    /// last message is the last typed message, bottom of console
    pub lastmessage: *mut bot_consolemessage_t,
    /// number of console messages stored in the state
    pub numconsolemessages: i32,
    /// the bot chat lines (arena handle; `None` = Raven's null `chat`)
    pub chat: Option<BotChatHandle>,
}

impl Default for BotChatState {
    /// Raven allocates chat states with `GetClearedMemory` (all-zero). The
    /// `String`s default to empty and the raw pool pointers to null, matching
    /// that zeroed state.
    fn default() -> Self {
        BotChatState {
            gender: 0,
            client: 0,
            name: String::new(),
            chatmessage: String::new(),
            handle: 0,
            firstmessage: core::ptr::null_mut(),
            lastmessage: core::ptr::null_mut(),
            numconsolemessages: 0,
            chat: None,
        }
    }
}
