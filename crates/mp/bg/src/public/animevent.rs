//! MP `bg_public.h` animation event descriptor.
//!
//! Type definition source: `oracle/oracle/codemp/game/bg_public.h:318-324`

#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_short, c_ushort};

use super::anim_event_type::animEventType_t;

/// Raven `MAX_RANDOM_ANIM_SOUNDS`.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:271`
pub const MAX_RANDOM_ANIM_SOUNDS: usize = 4;

/// Raven `AED_ARRAY_SIZE`.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:272`
pub const AED_ARRAY_SIZE: usize = MAX_RANDOM_ANIM_SOUNDS + 3;

/// Raven `animevent_s` (`animevent_t`).
///
/// Type definition source: `oracle/oracle/codemp/game/bg_public.h:318-324`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct animevent_t {
    pub eventType: animEventType_t,
    /// Frame to play event on
    pub keyFrame: c_ushort,
    /// Unique IDs, can be soundIndex of sound file to play OR effect index or footstep type, etc.
    pub eventData: [c_short; AED_ARRAY_SIZE],
    /// we allow storage of one string, temporarily (in case we have to look up an index
    /// later, then make sure to set stringData to NULL so we only do the look-up once)
    pub stringData: *mut c_char,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<animevent_t>() == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(animevent_t, eventType) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(animevent_t, keyFrame) == 4);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(animevent_t, eventData) == 6);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(animevent_t, stringData) == 24);
