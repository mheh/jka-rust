#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `dbrushside_t` — on-disk BSP brush side.
///
/// Type definition source: `oracle/code/qcommon/../qcommon/qfiles.h:491-495`
#[repr(C)]
pub struct dbrushside_t {
    /// positive plane side faces out of the leaf
    pub planeNum: c_int,
    pub shaderNum: c_int,
    pub drawSurfNum: c_int,
}

const _: () = assert!(core::mem::size_of::<dbrushside_t>() == 12);
const _: () = assert!(core::mem::offset_of!(dbrushside_t, planeNum) == 0);
const _: () = assert!(core::mem::offset_of!(dbrushside_t, shaderNum) == 4);
const _: () = assert!(core::mem::offset_of!(dbrushside_t, drawSurfNum) == 8);
