#![allow(non_camel_case_types, non_snake_case)]

/// Raven `bot_matchpiece_t` — one piece of a match template.
///
/// Redesigned (porting-rules §F17): Raven's `bot_matchstring_t *firststring`
/// chain (each node a malloc'd `char *`) collapses to an owned `Vec<String>`
/// (the `|`-separated alternatives, in file order), and the sibling `next`
/// pointer becomes the parent's `Vec<BotMatchPiece>`. `variable` is only
/// meaningful for `MT_VARIABLE` pieces; `strings` for `MT_STRING` pieces.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:109-122`
/// (`bot_matchstring_t` folded in from `:109-113`).
#[derive(Default, Clone)]
pub struct BotMatchPiece {
    /// `MT_STRING` or `MT_VARIABLE` (Raven `type`).
    pub type_: i32,
    /// `MT_STRING` alternatives (`firststring` chain); empty for `MT_VARIABLE`.
    pub strings: Vec<String>,
    pub variable: i32,
}
