#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_qshared::shared::MAX_QPATH;

/// Raven `mdxaHeader_t` — GLA animation-file header.
///
/// Type definition source: `oracle/codemp/renderer/../ghoul2/../renderer/mdx_format.h:351-371`
#[repr(C)]
pub struct mdxaHeader_t {
    // ( first 3 fields are same format as MD3/MDR so we can apply easy model-format-type checks )
    pub ident: i32,   // 	"IDP3" = MD3, "RDM5" = MDR, "2LGA"(GL2 Anim) = MDXA
    pub version: i32, // 1,2,3 etc as per format revision
    //
    pub name: [c_char; MAX_QPATH], // GLA name (eg "skeletons/marine")	// note: extension missing
    pub fScale: f32, // will be zero if build before this field was defined, else scale it was built with

    // frames and bones are shared by all levels of detail
    //
    pub numFrames: i32,
    pub ofsFrames: i32,       // points at mdxaFrame_t array
    pub numBones: i32,        // (no offset to these since they're inside the frames array)
    pub ofsCompBonePool: i32, // offset to global compressed-bone pool that all frames use
    pub ofsSkel: i32,         // offset to mdxaSkel_t info

    pub ofsEnd: i32, // EOF, which of course gives overall file size
}

const _: () = assert!(core::mem::size_of::<mdxaHeader_t>() == 100);
const _: () = assert!(core::mem::offset_of!(mdxaHeader_t, ident) == 0);
const _: () = assert!(core::mem::offset_of!(mdxaHeader_t, version) == 4);
const _: () = assert!(core::mem::offset_of!(mdxaHeader_t, name) == 8);
const _: () = assert!(core::mem::offset_of!(mdxaHeader_t, fScale) == 72);
const _: () = assert!(core::mem::offset_of!(mdxaHeader_t, numFrames) == 76);
const _: () = assert!(core::mem::offset_of!(mdxaHeader_t, ofsFrames) == 80);
const _: () = assert!(core::mem::offset_of!(mdxaHeader_t, numBones) == 84);
const _: () = assert!(core::mem::offset_of!(mdxaHeader_t, ofsCompBonePool) == 88);
const _: () = assert!(core::mem::offset_of!(mdxaHeader_t, ofsSkel) == 92);
const _: () = assert!(core::mem::offset_of!(mdxaHeader_t, ofsEnd) == 96);
