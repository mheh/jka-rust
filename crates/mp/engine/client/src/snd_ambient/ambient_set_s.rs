//! Raven `ambientSet_s` — one ambient sound set.
//!
//! The set never crosses the ABI seam, so the port takes the idiomatic shape
//! (porting-rules §D12) rather than the `native_types` layout twin the SP tree
//! still re-exports.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

pub use native_types::{MAX_SET_NAME_LENGTH, MAX_WAVES_PER_GROUP};

/// Raven `MAX_SET_VOLUME` — full volume for a set.
///
/// Source: `oracle/codemp/client/snd_ambient.cpp:15`
pub const MAX_SET_VOLUME: c_int = 255;

/// One ambient set: a looping bed plus a pool of one-shot subwaves.
///
/// `radius` of -1 means global, and `masterVolume` is an int rather than a byte
/// so a fade cannot wrap. `subWaves` and `loopedWave` hold registered
/// `sfxHandle_t` values, because the parser precaches every name it reads.
/// Type definition source: `oracle/codemp/client/snd_ambient.h:60-73`
#[derive(Clone)]
pub struct ambientSet_t {
    pub name: String,
    pub loopedVolume: u8,
    pub time_start: u32,
    pub time_end: u32,
    pub volRange_start: u32,
    pub volRange_end: u32,
    pub numSubWaves: u8,
    pub subWaves: [c_int; MAX_WAVES_PER_GROUP],
    pub loopedWave: c_int,
    /// NOTENOTE: -1 is global
    pub radius: c_int,
    /// Used for fading ambient sets (not a byte to prevent wrapping)
    pub masterVolume: c_int,
    /// Used for easier referencing of sets
    pub id: c_int,
    /// When the fade was started on this set
    pub fadeTime: c_int,
}

impl Default for ambientSet_t {
    /// Raven `CSetGroup::AddSet` zeroes the block, then writes these defaults.
    /// Source: `oracle/codemp/client/snd_ambient.cpp:119-145`
    fn default() -> ambientSet_t {
        ambientSet_t {
            name: String::new(),
            loopedVolume: MAX_SET_VOLUME as u8,
            time_start: 10,
            time_end: 25,
            volRange_start: MAX_SET_VOLUME as u32,
            volRange_end: MAX_SET_VOLUME as u32,
            numSubWaves: 0,
            subWaves: [0; MAX_WAVES_PER_GROUP],
            loopedWave: 0,
            radius: 250,
            masterVolume: MAX_SET_VOLUME,
            id: 0,
            fadeTime: 0,
        }
    }
}

/// Raven typedef `ambientSet_s` (the tagged struct name) for `ambientSet_t`.
pub type ambientSet_s = ambientSet_t;
