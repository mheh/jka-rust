#![allow(non_camel_case_types, non_snake_case)]

/// Raven `mdxaSkelOffsets_t` — variable-length skeleton offset table.
///
/// Raven: variable sized (mdxaHeader_t->numBones), each offset points to an
/// mdxaSkel_t below.
/// Type definition source: `oracle/oracle/code/game/../game/../renderer/mdx_format.h:376-379`
#[repr(C)]
pub struct mdxaSkelOffsets_t {
    pub offsets: [i32; 1],
}

const _: () = assert!(core::mem::size_of::<mdxaSkelOffsets_t>() == 4);
const _: () = assert!(core::mem::offset_of!(mdxaSkelOffsets_t, offsets) == 0);
