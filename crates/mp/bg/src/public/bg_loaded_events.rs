//! MP `bg_public.h` loaded animation-event cache.
//!
//! Type definition source: `oracle/codemp/game/bg_public.h:335-341`

#![allow(non_camel_case_types)]

use core::ffi::c_char;

use mp_qshared::shared::{qboolean, MAX_QPATH};

use super::animevent::animevent_t;

/// Raven `MAX_ANIM_EVENTS`.
///
/// Source: `oracle/codemp/game/bg_public.h:256`
pub const MAX_ANIM_EVENTS: usize = 300;

/// Raven `bgLoadedEvents_t`.
///
/// Type definition source: `oracle/codemp/game/bg_public.h:335-341`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct bgLoadedEvents_t {
    pub filename: [c_char; MAX_QPATH],
    pub torsoAnimEvents: [animevent_t; MAX_ANIM_EVENTS],
    pub legsAnimEvents: [animevent_t; MAX_ANIM_EVENTS],
    pub eventsParsed: qboolean,
}

const _: () = assert!(core::mem::offset_of!(bgLoadedEvents_t, filename) == 0);
const _: () = assert!(core::mem::offset_of!(bgLoadedEvents_t, torsoAnimEvents) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<bgLoadedEvents_t>() == 19272);
    assert!(core::mem::offset_of!(bgLoadedEvents_t, legsAnimEvents) == 9664);
    assert!(core::mem::offset_of!(bgLoadedEvents_t, eventsParsed) == 19264);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<bgLoadedEvents_t>() == 14468);
    assert!(core::mem::offset_of!(bgLoadedEvents_t, legsAnimEvents) == 7264);
    assert!(core::mem::offset_of!(bgLoadedEvents_t, eventsParsed) == 14464);
};
