#![allow(non_camel_case_types, non_snake_case)]

/// Raven `bot_randomlist_t` — a named list of random strings.
///
/// Redesigned (porting-rules §F17): Raven's `bot_randomstring_t
/// *firstrandomstring` chain (each node a malloc'd `char *`) collapses to an
/// owned `Vec<String>`, and the `next` sibling pointer becomes
/// `BotLib.randomstrings`'s `Vec`. Raven builds `firstrandomstring` by
/// *prepending*, so `strings` is stored in that reversed order (the loader
/// inserts at the front) to keep `RandomString`'s index → string mapping
/// byte-identical; `numstrings` is `strings.len()`.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:84-90`
/// (`bot_randomstring_t` folded in from `:78-82`).
#[derive(Default)]
pub struct BotRandomList {
    pub string: String,
    pub strings: Vec<String>,
}
