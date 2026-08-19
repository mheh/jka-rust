//! MP `bot_settings_t`.
//!
//! Type definition source: `oracle/codemp/game/g_local.h:1490-1496`

#![allow(non_camel_case_types)]

/// Raven `MAX_FILEPATH`. Source: `oracle/codemp/game/g_local.h:1480`
pub const MAX_FILEPATH: usize = 144;

/// Raven `bot_settings_t`, the per-bot personality and skill selectors.
///
/// Raven's `personalityfile` and `team` fields are `char[MAX_FILEPATH]`.
/// Here they are owned `String`s, and an empty string matches Raven's empty `[0]`.
/// The struct is game-internal: `bot_state_t` embeds it by value, and it never crosses the ABI seam.
/// It carries no fixed layout: no `#[repr(C)]`, no size assert.
/// Type definition source: `oracle/codemp/game/g_local.h:1490-1496`
#[derive(Clone, Default)]
pub struct bot_settings_t {
    pub personalityfile: String,
    pub skill: f32,
    pub team: String,
}
