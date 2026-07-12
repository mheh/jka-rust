#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use super::patch_collide_s::patchCollide_s;

/// Raven `cPatch_t` — a collision-model patch surface (bezier collision facets).
///
/// Type definition source: `oracle/codemp/qcommon/cm_local.h:91-96`
#[repr(C)]
pub struct cPatch_t {
    pub checkcount: c_int, // to avoid repeated testings
    pub surfaceFlags: c_int,
    pub contents: c_int,
    pub pc: *mut patchCollide_s,
}

const _: () = assert!(core::mem::offset_of!(cPatch_t, checkcount) == 0);
const _: () = assert!(core::mem::offset_of!(cPatch_t, surfaceFlags) == 4);
const _: () = assert!(core::mem::offset_of!(cPatch_t, contents) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<cPatch_t>() == 24);
    assert!(core::mem::offset_of!(cPatch_t, pc) == 16);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<cPatch_t>() == 16);
    assert!(core::mem::offset_of!(cPatch_t, pc) == 12);
};
