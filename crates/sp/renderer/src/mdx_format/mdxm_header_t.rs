#![allow(non_camel_case_types, non_snake_case)]
use core::ffi::c_char;
use sp_qshared::shared::MAX_QPATH;

/// Raven `mdxmHeader_t` — `.glm` mesh file header.
///
/// Raven: ( first 3 fields are same format as MD3/MDR so we can apply easy model-format-type
/// checks ).
/// Type definition source: `oracle/code/game/../game/../renderer/mdx_format.h:153-172`
#[repr(C)]
pub struct mdxmHeader_t {
    pub ident: i32,   // "IDP3" = MD3, "RDM5" = MDR, "2LGM"(GL2 Mesh) = MDX   (cruddy char order I know, but I'm following what was there in other versions)
    pub version: i32, // 1,2,3 etc as per format revision
    pub name: [c_char; MAX_QPATH], // model name (eg "models/players/marine.glm") // note: extension supplied
    pub animName: [c_char; MAX_QPATH], // name of animation file this mesh requires // note: extension missing
    pub animIndex: i32,                // filled in by game (carcass defaults it to 0)

    pub numBones: i32, // (for ingame version-checks only, ensure we don't ref more bones than skel file has)

    pub numLODs: i32,
    pub ofsLODs: i32,

    pub numSurfaces: i32, // now that surfaces are drawn hierarchically, we have same # per LOD
    pub ofsSurfHierarchy: i32,

    pub ofsEnd: i32, // EOF, which of course gives overall file size
}

const _: () = assert!(core::mem::size_of::<mdxmHeader_t>() == 164);
const _: () = assert!(core::mem::offset_of!(mdxmHeader_t, ident) == 0);
const _: () = assert!(core::mem::offset_of!(mdxmHeader_t, version) == 4);
const _: () = assert!(core::mem::offset_of!(mdxmHeader_t, name) == 8);
const _: () = assert!(core::mem::offset_of!(mdxmHeader_t, animName) == 72);
const _: () = assert!(core::mem::offset_of!(mdxmHeader_t, animIndex) == 136);
const _: () = assert!(core::mem::offset_of!(mdxmHeader_t, numBones) == 140);
const _: () = assert!(core::mem::offset_of!(mdxmHeader_t, numLODs) == 144);
const _: () = assert!(core::mem::offset_of!(mdxmHeader_t, ofsLODs) == 148);
const _: () = assert!(core::mem::offset_of!(mdxmHeader_t, numSurfaces) == 152);
const _: () = assert!(core::mem::offset_of!(mdxmHeader_t, ofsSurfHierarchy) == 156);
const _: () = assert!(core::mem::offset_of!(mdxmHeader_t, ofsEnd) == 160);
