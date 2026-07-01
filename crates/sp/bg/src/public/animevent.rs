//! SP `bg_public.h` animation event descriptor.
//!
//! Type definition source: `oracle/oracle/code/game/bg_public.h:537-545`

#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_short, c_ushort};

use super::anim_event_type::animEventType_t;

/// Raven `MAX_RANDOM_ANIM_SOUNDS`.
///
/// Source: `oracle/oracle/code/game/bg_public.h:487`
pub const MAX_RANDOM_ANIM_SOUNDS: usize = 8;

/// Raven `AED_ARRAY_SIZE`.
///
/// Source: `oracle/oracle/code/game/bg_public.h:488`
pub const AED_ARRAY_SIZE: usize = MAX_RANDOM_ANIM_SOUNDS + 3;

/// Raven `animevent_s` (`animevent_t`).
///
/// Type definition source: `oracle/oracle/code/game/bg_public.h:537-545`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct animevent_t {
    pub eventType: animEventType_t,
    /// event is specific to a modelname to skeleton
    pub modelOnly: c_short,
    pub glaIndex: c_ushort,
    /// Frame to play event on
    pub keyFrame: c_ushort,
    /// Unique IDs, can be soundIndex of sound file to play OR effect index or footstep type, etc.
    pub eventData: [c_short; AED_ARRAY_SIZE],
    /// we allow storage of one string, temporarily (in case we have to look up an index
    /// later, then make sure to set stringData to NULL so we only do the look-up once)
    pub stringData: *mut c_char,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<animevent_t>() == 40);
const _: () = assert!(core::mem::offset_of!(animevent_t, eventType) == 0);
const _: () = assert!(core::mem::offset_of!(animevent_t, modelOnly) == 4);
const _: () = assert!(core::mem::offset_of!(animevent_t, glaIndex) == 6);
const _: () = assert!(core::mem::offset_of!(animevent_t, keyFrame) == 8);
const _: () = assert!(core::mem::offset_of!(animevent_t, eventData) == 10);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(animevent_t, stringData) == 32);
