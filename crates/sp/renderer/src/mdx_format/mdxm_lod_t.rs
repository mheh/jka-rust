#![allow(non_camel_case_types, non_snake_case)]

/// Raven `mdxmLOD_t` — per-LOD end-offset marker.
///
/// Raven: (used to contain numSurface/ofsSurfaces fields, but these are same
/// per LOD level now).
/// Type definition source: `oracle/code/game/../game/../renderer/mdx_format.h:203-207`
#[repr(C)]
pub struct mdxmLOD_t {
    /// offset to next LOD
    pub ofsEnd: i32,
}

const _: () = assert!(core::mem::size_of::<mdxmLOD_t>() == 4);
const _: () = assert!(core::mem::offset_of!(mdxmLOD_t, ofsEnd) == 0);
