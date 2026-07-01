//! MP `bg_public.h` protection/dark saber sound definitions.
//!
//! Type definition source: `oracle/oracle/codemp/game/bg_public.h:734-743`

#![allow(non_camel_case_types)]

/// Raven `pdSounds_t`.
///
/// Type definition source: `oracle/oracle/codemp/game/bg_public.h:734-743`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum pdSounds_t {
    PDSOUND_NONE = 0,
    PDSOUND_PROTECTHIT = 1,
    PDSOUND_PROTECT = 2,
    PDSOUND_ABSORBHIT = 3,
    PDSOUND_ABSORB = 4,
    PDSOUND_FORCEJUMP = 5,
    PDSOUND_FORCEGRIP = 6,
}
