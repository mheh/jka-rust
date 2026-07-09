//! MP `bg_public.h` gender definitions.
//!
//! Type definition source: `oracle/codemp/game/bg_public.h:201-201`

#![allow(non_camel_case_types)]

/// Raven `gender_t`.
///
/// Type definition source: `oracle/codemp/game/bg_public.h:201-201`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum gender_t {
    GENDER_MALE = 0,
    GENDER_FEMALE = 1,
    GENDER_NEUTER = 2,
}
