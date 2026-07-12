#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};

/// Raven `MAX_CONFIGSTRINGS`.
///
/// Source: `oracle/codemp/game/q_shared.h:2037`
pub const MAX_CONFIGSTRINGS: usize = 1700;

/// Raven `MAX_GAMESTATE_CHARS`.
///
/// Source: `oracle/codemp/game/q_shared.h:2046`
pub const MAX_GAMESTATE_CHARS: usize = 16000;

/// Raven `gameState_t` — the config-string table exchanged server→client.
///
/// Type definition source: `oracle/codemp/game/q_shared.h:2047-2051`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct gameState_t {
    pub stringOffsets: [c_int; MAX_CONFIGSTRINGS],
    pub stringData: [c_char; MAX_GAMESTATE_CHARS],
    pub dataCount: c_int,
}

const _: () = {
    use core::mem::{offset_of, size_of};
    assert!(size_of::<gameState_t>() == 22804);
    assert!(offset_of!(gameState_t, stringData) == 6800);
    assert!(offset_of!(gameState_t, dataCount) == 22800);
};
