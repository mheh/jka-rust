#![allow(non_camel_case_types, non_snake_case)]
use core::ffi::c_int;

use crate::tr_local::shader_s::shader_t;

use super::ctrpatch::CTRPatch;

/// Raven `SPatchInfo` (typedef'd as `TPatchInfo`) — a patch plus the triangle-half
/// shader and part index to render for it.
///
/// Type definition source: `oracle/oracle/codemp/renderer/tr_landscape.h:108-113`
#[repr(C)]
pub struct TPatchInfo {
    pub mPatch: *mut CTRPatch,
    pub mShader: *mut shader_t,
    pub mPart: c_int,
}

/// Raven `SPatchInfo` is the struct tag; `TPatchInfo` is the typedef name used
/// throughout the codebase.
pub type SPatchInfo = TPatchInfo;

const _: () = assert!(core::mem::size_of::<TPatchInfo>() == 24);
const _: () = assert!(core::mem::offset_of!(TPatchInfo, mPatch) == 0);
const _: () = assert!(core::mem::offset_of!(TPatchInfo, mShader) == 8);
const _: () = assert!(core::mem::offset_of!(TPatchInfo, mPart) == 16);
