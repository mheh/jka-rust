//! MP `bot_settings_t`.
//!
//! Type definition source: `oracle/codemp/game/g_local.h:1490-1496`

#![allow(non_camel_case_types)]

/// Raven `MAX_FILEPATH`. Source: `oracle/codemp/game/g_local.h:1480`
pub const MAX_FILEPATH: usize = 144;

/// Raven `bot_settings_t` — per-bot personality/skill selectors.
///
/// Raven's `personalityfile`/`team` are `char[MAX_FILEPATH]`; here they are
/// owned `String`s (`""` ≡ Raven's empty `[0]`). The struct is game-internal —
/// embedded by value in `bot_state_t` and never crossing the ABI seam — so it
/// carries no fixed layout (no `#[repr(C)]`, no size assert).
/// Type definition source: `oracle/codemp/game/g_local.h:1490-1496`
#[derive(Clone, Default)]
pub struct bot_settings_t {
    pub personalityfile: String,
    pub skill: f32,
    pub team: String,
}
