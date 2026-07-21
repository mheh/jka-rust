//! MP `bg_public.h` loaded animation config.
//!
//! Type definition source: `oracle/codemp/game/bg_public.h:326-333`

#![allow(non_camel_case_types)]

use core::ptr::null_mut;

use super::animation::animation_t;

/// Raven `bgLoadedAnim_t`.
///
/// `filename` is an owned `String` (the `MAX_QPATH` byte bound is applied at the
/// write sites in `bg_panimate::BG_ParseAnimationFile`); the struct is
/// bg-internal (game/bg-island only, never memcpy'd across the trap ABI —
/// census-verified), so `#[repr(C)]` and the layout asserts are dropped.
/// Type definition source: `oracle/codemp/game/bg_public.h:326-333`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct bgLoadedAnim_t {
    pub filename: String,
    pub anims: *mut animation_t,
}

impl Default for bgLoadedAnim_t {
    fn default() -> Self {
        // Matches Raven's zero-initialized `bgLoadedAnim_t` static: empty name,
        // null animation pointer.
        bgLoadedAnim_t {
            filename: String::new(),
            anims: null_mut(),
        }
    }
}
