#![allow(non_camel_case_types, non_snake_case)]

/// Raven `mdxaIndex_t` — bone index wrapper.
///
/// Raven: this struct for pointing purposes, need to and with 0x00FFFFFF to
/// be meaningful.
/// Type definition source: `oracle/code/game/../game/../renderer/mdx_format.h:410-413`
#[repr(C)]
pub struct mdxaIndex_t {
    pub iIndex: i32,
}

const _: () = assert!(core::mem::size_of::<mdxaIndex_t>() == 4);
const _: () = assert!(core::mem::offset_of!(mdxaIndex_t, iIndex) == 0);
