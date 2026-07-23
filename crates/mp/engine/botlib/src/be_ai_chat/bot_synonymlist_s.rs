#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_ulong;

use super::bot_synonym_s::BotSynonym;

/// Raven `bot_synonymlist_t` — a list with synonyms, keyed by context bitmask.
///
/// Redesigned (porting-rules §F17): Raven's malloc'd `firstsynonym`/`next`
/// pointer chain becomes an owned `Vec<BotSynonym>` (file order preserved) and
/// the list itself lives in `BotLib.synonyms` as one element of a `Vec`.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:100-106`
#[derive(Default)]
pub struct BotSynonymList {
    pub context: c_ulong,
    pub totalweight: f32,
    pub synonyms: Vec<BotSynonym>,
}
