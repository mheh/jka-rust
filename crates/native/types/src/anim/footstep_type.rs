//! `bg_public.h` footstep type definitions.
//!
//! Type definition source: `oracle/codemp/game/bg_public.h:258-265`
//! Type definition source: `oracle/code/game/bg_public.h:550-557`

#![allow(non_camel_case_types)]

/// Raven `footstepType_t`.
///
/// Type definition source: `oracle/codemp/game/bg_public.h:258-265`
/// Type definition source: `oracle/code/game/bg_public.h:550-557`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum footstepType_t {
    FOOTSTEP_R = 0,
    FOOTSTEP_L = 1,
    FOOTSTEP_HEAVY_R = 2,
    FOOTSTEP_HEAVY_L = 3,
    NUM_FOOTSTEP_TYPES = 4,
}
