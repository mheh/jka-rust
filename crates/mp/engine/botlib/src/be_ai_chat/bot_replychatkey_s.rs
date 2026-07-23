#![allow(non_camel_case_types, non_snake_case)]

use super::bot_matchpiece_s::BotMatchPiece;

/// Raven `bot_replychatkey_t` — a reply-chat key.
///
/// Redesigned (porting-rules §F17): Raven's malloc'd `char *string` becomes an
/// owned `String`, the `match` piece chain becomes an owned
/// `Vec<BotMatchPiece>`, and the `next` sibling pointer becomes the parent
/// `BotReplyChat`'s `Vec` (Raven prepends the keys, so the loader inserts at
/// the front to keep match-variable extraction order identical).
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:134-140`
#[derive(Default)]
pub struct BotReplyChatKey {
    pub flags: i32,
    pub string: String,
    /// Raven `match` — the match-piece list for an `RCKFL_VARIABLES` key.
    pub match_: Vec<BotMatchPiece>,
}
