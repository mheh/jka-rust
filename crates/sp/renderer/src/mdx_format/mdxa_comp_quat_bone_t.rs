#![allow(non_camel_case_types, non_snake_case)]

/// Raven `mdxaCompQuatBone_t` — compressed quaternion bone (14 bytes packed).
///
/// Raven: I'm defining this '<' operator so this struct can be used as an STL
/// <map> key... (C++-only comparator, not ported).
/// Type definition source: `oracle/oracle/code/game/../game/../renderer/mdx_format.h:119-131`
#[repr(C)]
pub struct mdxaCompQuatBone_t {
    pub Comp: [u8; 14],
}

const _: () = assert!(core::mem::size_of::<mdxaCompQuatBone_t>() == 14);
const _: () = assert!(core::mem::offset_of!(mdxaCompQuatBone_t, Comp) == 0);
