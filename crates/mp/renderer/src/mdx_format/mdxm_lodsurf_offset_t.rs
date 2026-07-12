#![allow(non_camel_case_types, non_snake_case)]

/// Raven `mdxmLODSurfOffset_t` — added in GLM version 3 for ingame use at Jake's request.
///
/// Raven: variable sized (mdxmHeader_t->numSurfaces), each offset points to surfaces below.
/// Type definition source: `oracle/codemp/renderer/../ghoul2/../renderer/mdx_format.h:210-212`
#[repr(C)]
pub struct mdxmLODSurfOffset_t {
    pub offsets: [i32; 1],
}

const _: () = assert!(core::mem::size_of::<mdxmLODSurfOffset_t>() == 4);
const _: () = assert!(core::mem::offset_of!(mdxmLODSurfOffset_t, offsets) == 0);
