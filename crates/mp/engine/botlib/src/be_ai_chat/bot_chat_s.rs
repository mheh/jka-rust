#![allow(non_camel_case_types, non_snake_case)]

use super::bot_chattype_s::BotChatType;

/// Raven `bot_chat_t` — the set of chat types loaded from one chat file.
///
/// Redesigned (porting-rules §F17): Raven's `types` pointer chain becomes an
/// owned `Vec<BotChatType>` (Raven prepends, so the loader inserts at the
/// front). Instances live in the `BotLib.botchats` arena, reached by
/// `BotChatHandle`, because Raven shares one loaded chat between a bot's
/// `bot_chatstate_t.chat` and the `ichatdata[]` file cache (§B5).
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:72-75`
#[derive(Default)]
pub struct BotChat {
    pub types: Vec<BotChatType>,
}

/// Arena handle for a `BotChat` owned by `BotLib.botchats` (§B5).
///
/// Replaces Raven's `bot_chat_t *` (held by both `bot_chatstate_t.chat` and
/// `bot_ichatdata_t.chat`, which alias the same loaded chat in the cached
/// path). A `None` slot in the arena is Raven's freed pointer; the handle is
/// the slot index.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BotChatHandle(pub usize);
