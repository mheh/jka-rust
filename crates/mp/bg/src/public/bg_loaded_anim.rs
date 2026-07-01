//! MP `bg_public.h` loaded animation config.
//!
//! Type definition source: `oracle/oracle/codemp/game/bg_public.h:326-333`

#![allow(non_camel_case_types)]

use core::ffi::c_char;

use mp_qshared::shared::MAX_QPATH;

use super::animation::animation_t;

/// Raven `bgLoadedAnim_t`.
///
/// Type definition source: `oracle/oracle/codemp/game/bg_public.h:326-333`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct bgLoadedAnim_t {
    pub filename: [c_char; MAX_QPATH],
    pub anims: *mut animation_t,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<bgLoadedAnim_t>() == 72);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(bgLoadedAnim_t, filename) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(bgLoadedAnim_t, anims) == 64);
