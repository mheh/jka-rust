#![allow(non_camel_case_types, non_snake_case)]

/// Raven `ECPType` — consonant-piece classification for random-terrain name
/// generation.
///
/// Type definition source: `oracle/codemp/qcommon/cm_randomterrain.cpp:830-839`
#[repr(i32)]
pub enum ECPType {
	CP_NONE = -1,
	CP_CONSONANT = 0,
	CP_COMPLEX_CONSONANT = 1,
	CP_VOWEL = 2,
	CP_COMPLEX_VOWEL = 3,
	CP_ENDING = 4,
	CP_NUM_PIECES = 5,
}

const _: () = assert!(core::mem::size_of::<ECPType>() == 4);
