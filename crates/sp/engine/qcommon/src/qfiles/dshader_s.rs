#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use sp_qshared::shared::MAX_QPATH;

/// Raven `dshader_t` — BSP shader reference.
///
/// Type definition source: `oracle/code/qcommon/../qcommon/qfiles.h:457-461`
#[repr(C)]
pub struct dshader_t {
    pub shader: [c_char; MAX_QPATH],
    pub surfaceFlags: i32,
    pub contentFlags: i32,
}

pub type dshader_s = dshader_t;

const _: () = assert!(core::mem::size_of::<dshader_t>() == 72);
const _: () = assert!(core::mem::offset_of!(dshader_t, shader) == 0);
const _: () = assert!(core::mem::offset_of!(dshader_t, surfaceFlags) == 64);
const _: () = assert!(core::mem::offset_of!(dshader_t, contentFlags) == 68);
