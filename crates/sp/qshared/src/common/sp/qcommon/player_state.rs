//! SP `playerState_t` placeholder for Raven `code/game/q_shared.h`.
//!
//! Source: `oracle/oracle/code/game/q_shared.h:2066-2361`

#![allow(non_camel_case_types)]

/// Raven SP `playerState_t`.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:2077-2361`
///
/// FIXME: create type `saberInfo_t` before porting the full SP `playerState_t`
/// layout. Raven's SP struct embeds `saberInfo_t saber[MAX_SABERS]` and C++
/// helper methods in the middle of the field list; the methods do not affect
/// layout, but the embedded saber type must be ported before this can be ABI
/// complete.
#[repr(C)]
#[derive(Debug)]
pub struct playerState_t {
    _private: [u8; 0],
}
