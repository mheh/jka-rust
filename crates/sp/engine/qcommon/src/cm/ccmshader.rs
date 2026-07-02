#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::MAX_QPATH;
use core::ffi::c_char;

/// Raven `CCMShader` — a collision-model shader entry (name + surface/content flags),
/// linked into a hash-bucket chain via `mNext`.
///
/// Type definition source: `oracle/oracle/code/qcommon/cm_local.h:73-85`
#[repr(C)]
pub struct CCMShader {
    pub shader: [c_char; MAX_QPATH as usize],
    pub mNext: *mut CCMShader,
    pub surfaceFlags: i32,
    pub contentFlags: i32,
}

const _: () = assert!(core::mem::size_of::<CCMShader>() == 80);
const _: () = assert!(core::mem::offset_of!(CCMShader, shader) == 0);
const _: () = assert!(core::mem::offset_of!(CCMShader, mNext) == 64);
const _: () = assert!(core::mem::offset_of!(CCMShader, surfaceFlags) == 72);
const _: () = assert!(core::mem::offset_of!(CCMShader, contentFlags) == 76);
