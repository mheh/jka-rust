//! MP `bg_public.h` animation frame range descriptor.
//!
//! Type definition source: `oracle/oracle/codemp/game/bg_public.h:241-247`

#![allow(non_camel_case_types)]

use core::ffi::{c_schar, c_short, c_ushort};

/// Raven `animation_s` (`animation_t`).
///
/// Raven wraps this in `#pragma pack(push, 1)`; `repr(C, packed)` matches that layout.
/// Type definition source: `oracle/oracle/codemp/game/bg_public.h:241-247`
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct animation_t {
    pub firstFrame: c_ushort,
    pub numFrames: c_ushort,
    /// msec between frames
    ///
    /// initialLerp is abs(frameLerp)
    pub frameLerp: c_short,
    /// 0 to numFrames
    pub loopFrames: c_schar,
}

const _: () = assert!(core::mem::size_of::<animation_t>() == 7);
const _: () = assert!(core::mem::offset_of!(animation_t, firstFrame) == 0);
const _: () = assert!(core::mem::offset_of!(animation_t, numFrames) == 2);
const _: () = assert!(core::mem::offset_of!(animation_t, frameLerp) == 4);
const _: () = assert!(core::mem::offset_of!(animation_t, loopFrames) == 6);
