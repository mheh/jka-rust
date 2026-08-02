//! Raven `MusicFile_t` — one dynamic-music piece as `dms.dat` describes it.

#![allow(non_camel_case_types, non_snake_case)]

use std::collections::BTreeMap;

use crate::snd::music_exit_point_t::MusicExitPoint_t;
use crate::snd::music_exit_time_t::MusicExitTime_t;

/// One music piece: the file base name, its named entry markers, and its exit
/// points with the times they may be taken at.
///
/// All three collections are empty for a boss or death piece, which has no
/// transitions. `MusicEntryTimes` keeps Raven's `map` ordering, because
/// `Music_GetRandomEntryTime` picks the Nth entry in iteration order.
/// Type definition source: `oracle/codemp/client/snd_music.cpp:83-90`
#[derive(Clone, Default)]
pub struct MusicFile_t {
    pub sFileNameBase: String,
    /// key eg "marker1"
    pub MusicEntryTimes: BTreeMap<String, f32>,
    pub MusicExitPoints: Vec<MusicExitPoint_t>,
    /// Kept sorted by time, the way Raven sorts it after the parse.
    pub MusicExitTimes: Vec<MusicExitTime_t>,
}
