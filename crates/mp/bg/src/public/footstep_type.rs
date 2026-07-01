//! MP `bg_public.h` footstep type definitions.
//!
//! Type definition source: `oracle/oracle/codemp/game/bg_public.h:258-265`

#![allow(non_camel_case_types)]

/// Raven `footstepType_t`.
///
/// Type definition source: `oracle/oracle/codemp/game/bg_public.h:258-265`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum footstepType_t {
    FOOTSTEP_R = 0,
    FOOTSTEP_L = 1,
    FOOTSTEP_HEAVY_R = 2,
    FOOTSTEP_HEAVY_L = 3,
    NUM_FOOTSTEP_TYPES = 4,
}
