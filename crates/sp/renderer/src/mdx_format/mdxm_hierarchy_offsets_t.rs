#![allow(non_camel_case_types, non_snake_case)]

/// Raven `mdxmHierarchyOffsets_t` — table of offsets to `mdxmSurfHierarchy_t` entries.
///
/// Raven: variable sized (mdxmHeader_t->numSurfaces), each offset points to a
/// mdxmSurfHierarchy_t below.
/// Type definition source: `oracle/oracle/code/game/../game/../renderer/mdx_format.h:177-180`
#[repr(C)]
pub struct mdxmHierarchyOffsets_t {
    pub offsets: [i32; 1],
}

const _: () = assert!(core::mem::size_of::<mdxmHierarchyOffsets_t>() == 4);
const _: () = assert!(core::mem::offset_of!(mdxmHierarchyOffsets_t, offsets) == 0);
