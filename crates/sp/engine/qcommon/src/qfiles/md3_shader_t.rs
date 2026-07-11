#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use sp_qshared::shared::MAX_QPATH;

/// Raven `md3Shader_t` — MD3 shader reference.
///
/// Type definition source: `oracle/code/qcommon/../qcommon/qfiles.h:150-153`
#[repr(C)]
pub struct md3Shader_t {
    pub name: [c_char; MAX_QPATH],
    /// for in-game use
    pub shaderIndex: i32,
}

const _: () = assert!(core::mem::size_of::<md3Shader_t>() == 68);
const _: () = assert!(core::mem::offset_of!(md3Shader_t, name) == 0);
const _: () = assert!(core::mem::offset_of!(md3Shader_t, shaderIndex) == 64);
