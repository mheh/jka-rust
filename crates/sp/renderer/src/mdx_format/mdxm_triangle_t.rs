#![allow(non_camel_case_types, non_snake_case)]

/// Raven `mdxmTriangle_t` — triangle vertex indexes.
///
/// Type definition source: `oracle/oracle/code/game/../game/../renderer/mdx_format.h:250-252`
#[repr(C)]
pub struct mdxmTriangle_t {
    pub indexes: [i32; 3],
}

const _: () = assert!(core::mem::size_of::<mdxmTriangle_t>() == 12);
const _: () = assert!(core::mem::offset_of!(mdxmTriangle_t, indexes) == 0);
