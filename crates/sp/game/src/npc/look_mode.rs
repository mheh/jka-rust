//! SP `lookMode_t`.
//!
//! Type definition source: `oracle/code/game/b_public.h:91-95`

#![allow(non_camel_case_types)]

/// Raven SP `lookMode_t` — identical to MP.
///
/// Type definition source: `oracle/code/game/b_public.h:91-95`
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum lookMode_t {
    LM_ENT = 0,
    LM_INTEREST,
}
