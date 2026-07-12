#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

/// `MAX_PLAYERMODELS`.
///
/// Source: `oracle/codemp/ui/ui_local.h:594`
const MAX_PLAYERMODELS: usize = 32;

/// Raven `playerSpeciesInfo_t`.
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:716-727`
#[repr(C)]
pub struct playerSpeciesInfo_t {
    pub Name: [c_char; 64],
    pub SkinHeadCount: i32,
    pub SkinHeadNames: [[c_char; 16]; MAX_PLAYERMODELS],
    pub SkinTorsoCount: i32,
    pub SkinTorsoNames: [[c_char; 16]; MAX_PLAYERMODELS],
    pub SkinLegCount: i32,
    pub SkinLegNames: [[c_char; 16]; MAX_PLAYERMODELS],
    pub ColorShader: [[c_char; 64]; MAX_PLAYERMODELS],
    pub ColorCount: i32,
    pub ColorActionText: [[c_char; 128]; MAX_PLAYERMODELS],
}

const _: () = assert!(core::mem::size_of::<playerSpeciesInfo_t>() == 7760);
const _: () = assert!(core::mem::offset_of!(playerSpeciesInfo_t, Name) == 0);
const _: () = assert!(core::mem::offset_of!(playerSpeciesInfo_t, SkinHeadCount) == 64);
const _: () = assert!(core::mem::offset_of!(playerSpeciesInfo_t, SkinHeadNames) == 68);
const _: () = assert!(core::mem::offset_of!(playerSpeciesInfo_t, SkinTorsoCount) == 580);
const _: () = assert!(core::mem::offset_of!(playerSpeciesInfo_t, SkinTorsoNames) == 584);
const _: () = assert!(core::mem::offset_of!(playerSpeciesInfo_t, SkinLegCount) == 1096);
const _: () = assert!(core::mem::offset_of!(playerSpeciesInfo_t, SkinLegNames) == 1100);
const _: () = assert!(core::mem::offset_of!(playerSpeciesInfo_t, ColorShader) == 1612);
const _: () = assert!(core::mem::offset_of!(playerSpeciesInfo_t, ColorCount) == 3660);
const _: () = assert!(core::mem::offset_of!(playerSpeciesInfo_t, ColorActionText) == 3664);
