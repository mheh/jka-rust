//! SP `bg_public.h` animation frame range descriptor.
//!
//! Type definition source: `oracle/oracle/code/game/bg_public.h:468-475`

#![allow(non_camel_case_types)]

use core::ffi::{c_schar, c_short, c_uchar, c_ushort};

/// Raven `animation_s` (`animation_t`).
///
/// Type definition source: `oracle/oracle/code/game/bg_public.h:468-475`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct animation_t {
    pub firstFrame: c_ushort,
    pub numFrames: c_ushort,
    /// msec between frames
    ///
    /// initial lerp is abs(frameLerp)
    pub frameLerp: c_short,
    /// 0 to numFrames, -1 = no loop
    pub loopFrames: c_schar,
    pub glaIndex: c_uchar,
}

const _: () = assert!(core::mem::size_of::<animation_t>() == 8);
const _: () = assert!(core::mem::offset_of!(animation_t, firstFrame) == 0);
const _: () = assert!(core::mem::offset_of!(animation_t, numFrames) == 2);
const _: () = assert!(core::mem::offset_of!(animation_t, frameLerp) == 4);
const _: () = assert!(core::mem::offset_of!(animation_t, loopFrames) == 6);
const _: () = assert!(core::mem::offset_of!(animation_t, glaIndex) == 7);
