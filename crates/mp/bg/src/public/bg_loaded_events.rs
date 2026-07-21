//! MP `bg_public.h` loaded animation-event cache.
//!
//! Type definition source: `oracle/codemp/game/bg_public.h:335-341`

#![allow(non_camel_case_types)]

use mp_qshared::shared::qboolean;

use super::animevent::animevent_t;

/// Raven `MAX_ANIM_EVENTS`.
///
/// Source: `oracle/codemp/game/bg_public.h:256`
pub const MAX_ANIM_EVENTS: usize = 300;

/// Raven `bgLoadedEvents_t`.
///
/// `filename` is an owned `String` (the `MAX_QPATH` byte bound is applied at the
/// write sites); the struct is bg-internal (game/bg-island only, never memcpy'd
/// across the trap ABI — census-verified; its parse loader is CGAME-side and
/// dropped), so `#[repr(C)]` and the layout asserts (incl. the ILP32 twin) are
/// dropped.
/// Type definition source: `oracle/codemp/game/bg_public.h:335-341`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct bgLoadedEvents_t {
    pub filename: String,
    pub torsoAnimEvents: [animevent_t; MAX_ANIM_EVENTS],
    pub legsAnimEvents: [animevent_t; MAX_ANIM_EVENTS],
    pub eventsParsed: qboolean,
}

impl Default for bgLoadedEvents_t {
    fn default() -> Self {
        // Matches Raven's zero-initialized static: `animevent_t` is `Copy` POD
        // (all-zero image valid), so only `filename` needs a real default.
        bgLoadedEvents_t {
            filename: String::new(),
            torsoAnimEvents: unsafe { core::mem::zeroed() },
            legsAnimEvents: unsafe { core::mem::zeroed() },
            eventsParsed: 0,
        }
    }
}
