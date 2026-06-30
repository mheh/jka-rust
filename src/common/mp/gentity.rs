//! MP opaque `gentity_t` handle copied from Raven `codemp/game/g_local.h`.
//!
//! Source: `oracle/oracle/codemp/game/g_local.h:16`

#![allow(non_camel_case_types)]

/// Raven MP `gentity_t`.
///
/// Type declaration source: `oracle/oracle/codemp/game/g_local.h:16`
/// Full struct layout source: `oracle/oracle/codemp/game/g_local.h:133`
///
/// This is intentionally opaque until the full MP `struct gentity_s` layout is ported.
#[repr(C)]
#[derive(Debug)]
pub struct gentity_t {
    _private: [u8; 0],
}
