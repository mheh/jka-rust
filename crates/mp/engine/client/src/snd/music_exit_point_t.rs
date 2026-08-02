//! Raven `MusicExitPoint_t` — where one dynamic-music piece can leave for another.

#![allow(non_camel_case_types, non_snake_case)]

/// One exit point of a music file.
///
/// `sNextMark` is blank for an explore piece, which exits to silence, and names
/// the marker to enter the new file at for an action piece.
/// Type definition source: `oracle/codemp/client/snd_music.cpp:60-65`
#[derive(Clone, Default)]
pub struct MusicExitPoint_t {
    pub sNextFile: String,
    /// blank if used for an explore piece, name of marker point to enter new file at
    pub sNextMark: String,
}
