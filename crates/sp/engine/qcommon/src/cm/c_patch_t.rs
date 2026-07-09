#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use super::patch_collide_s::patchCollide_s;

/// Raven `cPatch_t` — a collision-model patch surface (bezier collision facets).
///
/// Type definition source: `oracle/code/qcommon/cm_local.h:87-92`
#[repr(C)]
pub struct cPatch_t {
    pub checkcount: c_int, // to avoid repeated testings
    pub surfaceFlags: c_int,
    pub contents: c_int,
    pub pc: *mut patchCollide_s,
}

const _: () = assert!(core::mem::size_of::<cPatch_t>() == 24);
const _: () = assert!(core::mem::offset_of!(cPatch_t, checkcount) == 0);
const _: () = assert!(core::mem::offset_of!(cPatch_t, surfaceFlags) == 4);
const _: () = assert!(core::mem::offset_of!(cPatch_t, contents) == 8);
const _: () = assert!(core::mem::offset_of!(cPatch_t, pc) == 16);
