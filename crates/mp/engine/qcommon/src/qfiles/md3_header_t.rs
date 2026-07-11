#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_qshared::shared::MAX_QPATH;

/// Raven `md3Header_t` — MD3 model file header.
///
/// Type definition source: `oracle/codemp/qcommon/../qcommon/qfiles.h:169-188`
#[repr(C)]
pub struct md3Header_t {
    pub ident: i32,
    pub version: i32,

    /// model name
    pub name: [c_char; MAX_QPATH],

    pub flags: i32,

    pub numFrames: i32,
    pub numTags: i32,
    pub numSurfaces: i32,

    pub numSkins: i32,

    /// offset for first frame
    pub ofsFrames: i32,
    /// numFrames * numTags
    pub ofsTags: i32,
    /// first surface, others follow
    pub ofsSurfaces: i32,

    /// end of file
    pub ofsEnd: i32,
}

const _: () = assert!(core::mem::size_of::<md3Header_t>() == 108);
const _: () = assert!(core::mem::offset_of!(md3Header_t, ident) == 0);
const _: () = assert!(core::mem::offset_of!(md3Header_t, version) == 4);
const _: () = assert!(core::mem::offset_of!(md3Header_t, name) == 8);
const _: () = assert!(core::mem::offset_of!(md3Header_t, flags) == 72);
const _: () = assert!(core::mem::offset_of!(md3Header_t, numFrames) == 76);
const _: () = assert!(core::mem::offset_of!(md3Header_t, numTags) == 80);
const _: () = assert!(core::mem::offset_of!(md3Header_t, numSurfaces) == 84);
const _: () = assert!(core::mem::offset_of!(md3Header_t, numSkins) == 88);
const _: () = assert!(core::mem::offset_of!(md3Header_t, ofsFrames) == 92);
const _: () = assert!(core::mem::offset_of!(md3Header_t, ofsTags) == 96);
const _: () = assert!(core::mem::offset_of!(md3Header_t, ofsSurfaces) == 100);
const _: () = assert!(core::mem::offset_of!(md3Header_t, ofsEnd) == 104);
