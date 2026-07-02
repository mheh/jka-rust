#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::c_char;

use sp_qshared::shared::MAX_QPATH;

/// Raven `mdxmSurfHierarchy_t` — per-surface hierarchy/parenting info.
///
/// Raven: struct size = (int)( &((mdxmSurfHierarch_t *)0)->childIndexes[
/// mdxmSurfHierarch_t->numChildren] ).
/// Type definition source: `oracle/oracle/code/game/../game/../renderer/mdx_format.h:187-195`
#[repr(C)]
pub struct mdxmSurfHierarchy_t {
    pub name: [c_char; MAX_QPATH as usize],
    pub flags: u32,
    pub shader: [c_char; MAX_QPATH as usize],
    /// for in-game use (carcass defaults to 0)
    pub shaderIndex: i32,
    /// this points to the index in the file of the parent surface. -1 if null/root
    pub parentIndex: i32,
    /// number of surfaces which are children of this one
    pub numChildren: i32,
    /// \[mdxmSurfHierarch_t->numChildren\] (variable sized)
    pub childIndexes: [i32; 1],
}

const _: () = assert!(core::mem::size_of::<mdxmSurfHierarchy_t>() == 148);
const _: () = assert!(core::mem::offset_of!(mdxmSurfHierarchy_t, name) == 0);
const _: () = assert!(core::mem::offset_of!(mdxmSurfHierarchy_t, flags) == 64);
const _: () = assert!(core::mem::offset_of!(mdxmSurfHierarchy_t, shader) == 68);
const _: () = assert!(core::mem::offset_of!(mdxmSurfHierarchy_t, shaderIndex) == 132);
const _: () = assert!(core::mem::offset_of!(mdxmSurfHierarchy_t, parentIndex) == 136);
const _: () = assert!(core::mem::offset_of!(mdxmSurfHierarchy_t, numChildren) == 140);
const _: () = assert!(core::mem::offset_of!(mdxmSurfHierarchy_t, childIndexes) == 144);
