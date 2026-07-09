#![allow(non_camel_case_types, non_snake_case)]
use mp_qshared::shared::vec2_t;

/// Raven `mdxmVertexTexCoord_t` — Ghoul2 mesh vertex texture coordinate.
///
/// Type definition source: `oracle/codemp/renderer/../ghoul2/../renderer/mdx_format.h:328-334`
#[repr(C)]
pub struct mdxmVertexTexCoord_t {
    pub texCoords: vec2_t,
}
const _: () = assert!(core::mem::size_of::<mdxmVertexTexCoord_t>() == 8);
const _: () = assert!(core::mem::offset_of!(mdxmVertexTexCoord_t, texCoords) == 0);
