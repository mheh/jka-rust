//! `MusicData_t` — the owned home of every `snd_music.cpp` file-scope global.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;
use std::collections::BTreeMap;

use crate::snd::music_file_t::MusicFile_t;

/// The dynamic-music description of one level, plus the level names the loader
/// and the "uses" redirection track.
///
/// Raven keeps `MusicData` as a heap `map` it deletes on free, so `None` here is
/// Raven's null pointer and an empty map is a cleared one. `Music_Parse_Error`
/// clears the map, which is why a parse error must exit its loop at once.
/// Source: `oracle/codemp/client/snd_music.cpp:92-99,1112-1113`
pub struct MusicData_t {
    /// The pieces by state key: "explore", "action", "boss", "death".
    pub MusicData: Option<BTreeMap<String, MusicFile_t>>,
    /// eg "kejim_base", the dir name every music path is built under.
    pub gsLevelNameForLoad: String,
    /// eg "kejim_base", the name a repeat load compares against.
    pub gsLevelNameForCompare: String,
    /// eg "kejim_base", the special case that lets boss music come from
    /// another directory.
    pub gsLevelNameForBossLoad: String,
    /// Raven `gsLevelNameFromServer`, set by `Music_SetLevelName`.
    pub gsLevelNameFromServer: String,
    /// Raven's `Music_GetRandomEntryTime` function statics.
    /// Source: `oracle/codemp/client/snd_music.cpp:1112-1113`
    pub iPrevRandomNumber: c_int,
    pub iCallCount: c_int,
}

impl MusicData_t {
    /// The C loader's zero fill, with `iPrevRandomNumber` at Raven's `-1`.
    pub fn new() -> MusicData_t {
        MusicData_t {
            MusicData: None,
            gsLevelNameForLoad: String::new(),
            gsLevelNameForCompare: String::new(),
            gsLevelNameForBossLoad: String::new(),
            gsLevelNameFromServer: String::new(),
            iPrevRandomNumber: -1,
            iCallCount: 0,
        }
    }
}

impl Default for MusicData_t {
    fn default() -> MusicData_t {
        MusicData_t::new()
    }
}
