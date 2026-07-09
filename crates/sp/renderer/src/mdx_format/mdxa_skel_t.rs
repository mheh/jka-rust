#![allow(non_camel_case_types, non_snake_case)]
use core::ffi::c_char;
use sp_qshared::shared::{mdxaBone_t, MAX_QPATH};

/// Raven `mdxaSkel_t` — one bone entry in a `.gla` skeleton.
///
/// Raven: struct size = (int)( &((mdxaSkel_t *)0)->children[ mdxaSkel_t->numChildren ] );
/// Type definition source: `oracle/code/game/../game/../renderer/mdx_format.h:388-396`
#[repr(C)]
pub struct mdxaSkel_t {
    pub name: [c_char; MAX_QPATH], // name of bone
    pub flags: u32,
    pub parent: i32,                // index of bone that is parent to this one, -1 = NULL/root
    pub BasePoseMat: mdxaBone_t,    // base pose
    pub BasePoseMatInv: mdxaBone_t, // inverse, to save run-time calc
    pub numChildren: i32,           // number of children bones
    pub children: [i32; 1],         // [mdxaSkel_t->numChildren] (variable sized)
}

const _: () = assert!(core::mem::size_of::<mdxaSkel_t>() == 176);
const _: () = assert!(core::mem::offset_of!(mdxaSkel_t, name) == 0);
const _: () = assert!(core::mem::offset_of!(mdxaSkel_t, flags) == 64);
const _: () = assert!(core::mem::offset_of!(mdxaSkel_t, parent) == 68);
const _: () = assert!(core::mem::offset_of!(mdxaSkel_t, BasePoseMat) == 72);
const _: () = assert!(core::mem::offset_of!(mdxaSkel_t, BasePoseMatInv) == 120);
const _: () = assert!(core::mem::offset_of!(mdxaSkel_t, numChildren) == 168);
const _: () = assert!(core::mem::offset_of!(mdxaSkel_t, children) == 172);
