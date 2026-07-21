#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

/// `MAX_WAVES_PER_GROUP`.
///
/// Source: `oracle/codemp/client/snd_ambient.h:28`
/// Source: `oracle/code/client/snd_ambient.h:28`
pub const MAX_WAVES_PER_GROUP: usize = 8;

/// `MAX_SET_NAME_LENGTH`.
///
/// Source: `oracle/codemp/client/snd_ambient.h:29`
/// Source: `oracle/code/client/snd_ambient.h:29`
pub const MAX_SET_NAME_LENGTH: usize = 64;

/// Raven `ambientSet_s` — a named set of ambient background waves that can be
/// looped and cross-faded.
///
/// Type definition source: `oracle/codemp/client/snd_ambient.h:60-73`
/// Type definition source: `oracle/code/client/snd_ambient.h:60-73`
#[repr(C)]
pub struct ambientSet_s {
    pub name: [c_char; MAX_SET_NAME_LENGTH],
    pub loopedVolume: u8,
    pub time_start: u32,
    pub time_end: u32,
    pub volRange_start: u32,
    pub volRange_end: u32,
    pub numSubWaves: u8,
    pub subWaves: [i32; MAX_WAVES_PER_GROUP],
    pub loopedWave: i32,
    /// NOTENOTE: -1 is global
    pub radius: i32,
    /// Used for fading ambient sets (not a byte to prevent wrapping)
    pub masterVolume: i32,
    /// Used for easier referencing of sets
    pub id: i32,
    /// When the fade was started on this set
    pub fadeTime: i32,
}

/// Raven `ambientSet_t` — typedef alias for `ambientSet_s`.
pub type ambientSet_t = ambientSet_s;

const _: () = assert!(core::mem::size_of::<ambientSet_t>() == 140);
const _: () = assert!(core::mem::offset_of!(ambientSet_t, name) == 0);
const _: () = assert!(core::mem::offset_of!(ambientSet_t, loopedVolume) == 64);
const _: () = assert!(core::mem::offset_of!(ambientSet_t, time_start) == 68);
const _: () = assert!(core::mem::offset_of!(ambientSet_t, time_end) == 72);
const _: () = assert!(core::mem::offset_of!(ambientSet_t, volRange_start) == 76);
const _: () = assert!(core::mem::offset_of!(ambientSet_t, volRange_end) == 80);
const _: () = assert!(core::mem::offset_of!(ambientSet_t, numSubWaves) == 84);
const _: () = assert!(core::mem::offset_of!(ambientSet_t, subWaves) == 88);
const _: () = assert!(core::mem::offset_of!(ambientSet_t, loopedWave) == 120);
const _: () = assert!(core::mem::offset_of!(ambientSet_t, radius) == 124);
const _: () = assert!(core::mem::offset_of!(ambientSet_t, masterVolume) == 128);
const _: () = assert!(core::mem::offset_of!(ambientSet_t, id) == 132);
const _: () = assert!(core::mem::offset_of!(ambientSet_t, fadeTime) == 136);
