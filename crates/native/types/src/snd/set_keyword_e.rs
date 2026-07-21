#![allow(non_camel_case_types, non_snake_case)]

/// Raven `setKeyword_e` — keyword types for ambient set parsing.
///
/// Type definition source: `oracle/codemp/client/snd_ambient.h:42-55`
/// Type definition source: `oracle/code/client/snd_ambient.h:42-55`
#[repr(i32)]
pub enum setKeyword_e {
    SET_KEYWORD_TIMEBETWEENWAVES = 0,
    SET_KEYWORD_SUBWAVES = 1,
    SET_KEYWORD_LOOPEDWAVE = 2,
    SET_KEYWORD_VOLRANGE = 3,
    SET_KEYWORD_RADIUS = 4,
    SET_KEYWORD_TYPE = 5,
    SET_KEYWORD_AMSDIR = 6,
    SET_KEYWORD_OUTDIR = 7,
    SET_KEYWORD_BASEDIR = 8,
    NUM_AS_KEYWORDS = 9,
}

const _: () = assert!(core::mem::size_of::<setKeyword_e>() == 4);
