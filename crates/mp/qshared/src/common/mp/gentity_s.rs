//! MP `gentity_s` forward declaration.
//!
//! Type declaration source: `oracle/codemp/game/g_local.h:16`

#![allow(non_camel_case_types)]

/// Raven `struct gentity_s;` forward declaration — the abi tier carries entity
/// pointers opaquely (DEC-26). The real `gentity_t` layout lives in `mp_game`
/// (`crate::entity::gentity`); the sub-game abi seam (`mp_abi`) only ever names
/// `*mut gentity_s` in its ~18 entity-carrying syscalls, so it needs the name
/// but not the fields.
///
/// A zero-length `#[repr(C)]` struct (not an uninhabited enum) because no value
/// of this type is ever formed — it exists only behind pointers, exactly as the
/// C forward declaration does.
///
/// Type declaration source: `oracle/codemp/game/g_local.h:16`
#[repr(C)]
pub struct gentity_s {
    _opaque: [u8; 0],
}
