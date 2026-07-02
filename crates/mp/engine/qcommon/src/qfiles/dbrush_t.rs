#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `dbrush_t` — on-disk BSP brush.
///
/// Type definition source: `oracle/oracle/codemp/qcommon/../qcommon/qfiles.h:487-491`
#[repr(C)]
pub struct dbrush_t {
    pub firstSide: c_int,
    pub numSides: c_int,
    /// the shader that determines the contents flags
    pub shaderNum: c_int,
}

const _: () = assert!(core::mem::size_of::<dbrush_t>() == 12);
const _: () = assert!(core::mem::offset_of!(dbrush_t, firstSide) == 0);
const _: () = assert!(core::mem::offset_of!(dbrush_t, numSides) == 4);
const _: () = assert!(core::mem::offset_of!(dbrush_t, shaderNum) == 8);
