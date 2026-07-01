//! MP `bg_public.h` CTF message type definitions.
//!
//! Type definition source: `oracle/oracle/codemp/game/bg_public.h:707-713`

#![allow(non_camel_case_types)]

/// Raven `ctfMsg_t`.
///
/// Type definition source: `oracle/oracle/codemp/game/bg_public.h:707-713`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ctfMsg_t {
    CTFMESSAGE_FRAGGED_FLAG_CARRIER = 0,
    CTFMESSAGE_FLAG_RETURNED = 1,
    CTFMESSAGE_PLAYER_RETURNED_FLAG = 2,
    CTFMESSAGE_PLAYER_CAPTURED_FLAG = 3,
    CTFMESSAGE_PLAYER_GOT_FLAG = 4,
}
