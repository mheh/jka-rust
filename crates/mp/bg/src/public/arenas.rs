//! MP arena/bot info text-buffer limits from `bg_public.h`.

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

/// Raven `MAX_ARENAS_TEXT`.
///
/// Source: `oracle/codemp/game/bg_public.h:1674`
pub const MAX_ARENAS_TEXT: usize = 8192;

/// Raven `MAX_BOTS_TEXT`.
///
/// Source: `oracle/codemp/game/bg_public.h:1677`
pub const MAX_BOTS_TEXT: usize = 8192;
