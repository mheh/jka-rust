//! SP `playerState_t` placeholder for Raven `code/game/q_shared.h`.
//!
//! Source: `oracle/oracle/code/game/q_shared.h:2066-2361`

#![allow(non_camel_case_types)]

/// Raven SP `playerState_t` — placeholder stub (full layout not yet ported).
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:2077-2361`
//TODO: Port playerState_t
// Source: oracle/oracle/code/game/q_shared.h:2077-2361
// Deferred full heavy-struct port. No longer blocked: SP `saberInfo_t` is ported
// in-crate and is embedded by value as `saber[MAX_SABERS]` (oracle:2168). The C++
// helper methods interspersed in the field list do not affect layout. Remaining
// work is transcribing the full ~284-line SP field layout + offset/size asserts.
#[repr(C)]
#[derive(Debug)]
pub struct playerState_t {
    _private: [u8; 0],
}
