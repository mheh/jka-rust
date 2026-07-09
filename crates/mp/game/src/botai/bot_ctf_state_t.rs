#![allow(non_camel_case_types, non_snake_case)]

/// Raven `bot_ctf_state_t` — CTF bot state enumeration.
///
/// Type definition source: `oracle/codemp/game/ai_main.h:81-90`
#[repr(i32)]
pub enum bot_ctf_state_t {
    CTFSTATE_NONE = 0,
    CTFSTATE_ATTACKER,
    CTFSTATE_DEFENDER,
    CTFSTATE_RETRIEVAL,
    CTFSTATE_GUARDCARRIER,
    CTFSTATE_GETFLAGHOME,
    CTFSTATE_MAXCTFSTATES,
}
