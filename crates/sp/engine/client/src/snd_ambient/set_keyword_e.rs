#![allow(non_camel_case_types, non_snake_case)]

/// Raven `setKeyword_e` — keywords for ambient set parsing.
///
/// Type definition source: `oracle/code/client/snd_ambient.h:42-55`
#[repr(i32)]
pub enum setKeyword_e {
    SET_KEYWORD_TIMEBETWEENWAVES,
    SET_KEYWORD_SUBWAVES,
    SET_KEYWORD_LOOPEDWAVE,
    SET_KEYWORD_VOLRANGE,
    SET_KEYWORD_RADIUS,
    SET_KEYWORD_TYPE,
    SET_KEYWORD_AMSDIR,
    SET_KEYWORD_OUTDIR,
    SET_KEYWORD_BASEDIR,
    NUM_AS_KEYWORDS,
}
const _: () = assert!(core::mem::size_of::<setKeyword_e>() == 4);
