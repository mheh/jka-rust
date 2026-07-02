//! MP `stereoFrame_t` from `codemp/cgame/tr_types.h`.

#![allow(non_camel_case_types)]

use core::ffi::c_int;

// Raven's MP `typedef int stereoFrame_t` sits next to a separate anonymous
// enum (unlike SP's named enum), so this stays an int alias + consts.
/// Raven `stereoFrame_t` — which eye a frame is rendered for.
///
/// Type definition source: `oracle/oracle/codemp/cgame/tr_types.h:278-284`
pub type stereoFrame_t = c_int;

/// Raven `STEREO_CENTER`.
pub const STEREO_CENTER: stereoFrame_t = 0;
/// Raven `STEREO_LEFT`.
pub const STEREO_LEFT: stereoFrame_t = 1;
/// Raven `STEREO_RIGHT`.
pub const STEREO_RIGHT: stereoFrame_t = 2;
