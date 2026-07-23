#![allow(non_camel_case_types, non_snake_case)]

/// Raven `bot_synonym_t` — one synonym (string + weight) in a synonym list.
///
/// Redesigned (porting-rules §F17) from Raven's malloc'd `char *string` +
/// `next` pointer into an owned value: the `String` owns its text and the
/// sibling chain becomes the parent `BotSynonymList`'s `Vec<BotSynonym>`.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:93-98`
#[derive(Default, Clone)]
pub struct BotSynonym {
    pub string: String,
    pub weight: f32,
}
