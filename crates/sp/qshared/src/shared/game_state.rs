#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};

/// Raven `MAX_CONFIGSTRINGS` (SP).
///
/// SP-vs-MP: SP is 1300 (MP 1700).
/// Source: `oracle/code/game/q_shared.h:1482`
pub const MAX_CONFIGSTRINGS: usize = 1300;

/// Raven `MAX_GAMESTATE_CHARS`.
///
/// Source: `oracle/code/game/q_shared.h:1531`
pub const MAX_GAMESTATE_CHARS: usize = 16000;

/// Raven `gameState_t` — the config-string table exchanged server→client.
///
/// Same shape as MP but SP's smaller `MAX_CONFIGSTRINGS` makes it 21204 B (MP 22804).
/// Type definition source: `oracle/code/game/q_shared.h:1532-1536`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct gameState_t {
    pub stringOffsets: [c_int; MAX_CONFIGSTRINGS],
    pub stringData: [c_char; MAX_GAMESTATE_CHARS],
    pub dataCount: c_int,
}

const _: () = {
    use core::mem::{offset_of, size_of};
    assert!(size_of::<gameState_t>() == 21204);
    assert!(offset_of!(gameState_t, stringData) == 5200);
    assert!(offset_of!(gameState_t, dataCount) == 21200);
};
