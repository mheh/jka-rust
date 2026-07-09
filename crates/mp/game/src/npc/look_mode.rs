//! MP `lookMode_t`.
//!
//! Type definition source: `oracle/codemp/game/b_public.h:70-75`

#![allow(non_camel_case_types)]

/// Raven `lookMode_t`.
///
/// `typedef enum` → int-wide discriminants.
/// Type definition source: `oracle/codemp/game/b_public.h:70-75`
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum lookMode_t {
    LM_ENT = 0,
    LM_INTEREST,
}
