#![allow(non_camel_case_types, non_snake_case)]

//! SP `g_local.h` per-model animation config.
//!
//! Type definition source: `oracle/oracle/code/game/g_local.h:68-76`

use core::ffi::c_char;

use sp_bg::public::anim_number::animNumber_t;
use sp_bg::public::animation::animation_t;
use sp_bg::public::animevent::animevent_t;
use sp_qshared::shared::MAX_QPATH;

/// Raven `MAX_ANIM_EVENTS`.
///
/// Source: `oracle/oracle/code/game/bg_public.h:484`
pub const MAX_ANIM_EVENTS: usize = 300;

/// Raven `MAX_ANIMATIONS` — sentinel of the `animNumber_t` enum.
///
/// Source: `oracle/oracle/code/game/anims.h:1789`
pub const MAX_ANIMATIONS: usize = 1543;

const _: () = assert!(animNumber_t::MAX_ANIMATIONS as usize == MAX_ANIMATIONS);

/// Raven `animFileSet_t`.
///
/// Type definition source: `oracle/oracle/code/game/g_local.h:68-76`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct animFileSet_t {
    pub filename: [c_char; MAX_QPATH],
    pub animations: [animation_t; MAX_ANIMATIONS],
    pub torsoAnimEvents: [animevent_t; MAX_ANIM_EVENTS],
    pub legsAnimEvents: [animevent_t; MAX_ANIM_EVENTS],
    pub torsoAnimEventCount: u8,
    pub legsAnimEventCount: u8,
}

const _: () = assert!(core::mem::size_of::<animFileSet_t>() == 36416);
const _: () = assert!(core::mem::offset_of!(animFileSet_t, filename) == 0);
const _: () = assert!(core::mem::offset_of!(animFileSet_t, animations) == 64);
const _: () = assert!(core::mem::offset_of!(animFileSet_t, torsoAnimEvents) == 12408);
const _: () = assert!(core::mem::offset_of!(animFileSet_t, legsAnimEvents) == 24408);
const _: () = assert!(core::mem::offset_of!(animFileSet_t, torsoAnimEventCount) == 36408);
const _: () = assert!(core::mem::offset_of!(animFileSet_t, legsAnimEventCount) == 36409);
