#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_ulong;

use super::bot_matchpiece_s::BotMatchPiece;

/// Raven `bot_matchtemplate_t` — a match template keyed by context bitmask.
///
/// Redesigned (porting-rules §F17): Raven's `first` piece chain becomes an
/// owned `Vec<BotMatchPiece>` and the `next` sibling pointer becomes
/// `BotLib.matchtemplates`'s `Vec` (file order preserved).
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:124-131`
#[derive(Default)]
pub struct BotMatchTemplate {
    pub context: c_ulong,
    /// Raven `type` — the match result type.
    pub type_: i32,
    pub subtype: i32,
    pub first: Vec<BotMatchPiece>,
}
