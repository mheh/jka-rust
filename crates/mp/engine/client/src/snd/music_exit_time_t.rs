//! Raven `MusicExitTime_t` — one time an exit point may be taken at.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// One exit time, and the exit point it belongs to.
///
/// Raven declares `operator <` on `fTime` alone so the STL sort and
/// `equal_range` work on a time-ordered array, and the port sorts the same way.
/// Type definition source: `oracle/codemp/client/snd_music.cpp:67-75`
#[derive(Clone, Copy, Default, PartialEq)]
pub struct MusicExitTime_t {
    pub fTime: f32,
    pub iExitPoint: c_int,
}
