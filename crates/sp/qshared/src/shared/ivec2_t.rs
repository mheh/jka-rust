#![allow(non_camel_case_types)]

use core::ffi::c_int;

/// Raven `ivec2_t` — 2-component integer vector. SP-only (MP has no `ivec2_t`).
///
/// Type definition source: `oracle/code/game/q_shared.h:322`
pub type ivec2_t = [c_int; 2];
