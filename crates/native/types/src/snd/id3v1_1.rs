#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

/// Raven `id3v1_1` — ID3v1.1 MP3 tag trailer, 128 bytes in size.
///
/// Type definition source: `oracle/codemp/client/snd_mp3.h:15-25`
/// Type definition source: `oracle/code/client/cl_mp3.h:15-25`
#[repr(C)]
pub struct id3v1_1 {
    pub id: [c_char; 3],
    /// <file basename>
    pub title: [c_char; 30],
    /// "Raven Software"
    pub artist: [c_char; 30],
    /// "#UNCOMP %d"		// needed
    pub album: [c_char; 30],
    /// "2000"
    pub year: [c_char; 4],
    /// "#MAXVOL %g"		// needed
    pub comment: [c_char; 28],
    pub zero: c_char,
    pub track: c_char,
    pub genre: c_char,
}

const _: () = assert!(core::mem::size_of::<id3v1_1>() == 128);
const _: () = assert!(core::mem::offset_of!(id3v1_1, id) == 0);
const _: () = assert!(core::mem::offset_of!(id3v1_1, title) == 3);
const _: () = assert!(core::mem::offset_of!(id3v1_1, artist) == 33);
const _: () = assert!(core::mem::offset_of!(id3v1_1, album) == 63);
const _: () = assert!(core::mem::offset_of!(id3v1_1, year) == 93);
const _: () = assert!(core::mem::offset_of!(id3v1_1, comment) == 97);
const _: () = assert!(core::mem::offset_of!(id3v1_1, zero) == 125);
const _: () = assert!(core::mem::offset_of!(id3v1_1, track) == 126);
const _: () = assert!(core::mem::offset_of!(id3v1_1, genre) == 127);
