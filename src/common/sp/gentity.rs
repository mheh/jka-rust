//! SP opaque `gentity_t` handle copied from Raven `code/game/g_public.h`.
//!
//! Source: `oracle/oracle/code/game/g_public.h:51`

#![allow(non_camel_case_types)]

/// Raven SP `gentity_t`.
///
/// Type declaration source: `oracle/oracle/code/game/g_public.h:51`
/// Type declaration source: `oracle/oracle/code/game/bg_public.h:129`
/// Full struct layout source: `oracle/oracle/code/game/g_public.h:67`
/// Full struct layout source: `oracle/oracle/code/game/g_shared.h:514`
///
/// This is intentionally opaque until the full SP `struct gentity_s` layout is ported.
#[repr(C)]
#[derive(Debug)]
pub struct gentity_t {
    _private: [u8; 0],
}
