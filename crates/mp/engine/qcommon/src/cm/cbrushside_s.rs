#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;
use mp_qshared::shared::collision::cplane_t;

/// Raven `cbrushside_t` — one side (plane + shader) of a collision-model brush.
///
/// Type definition source: `oracle/codemp/qcommon/cm_local.h:61-64`
#[repr(C)]
pub struct cbrushside_s {
    pub plane: *mut cplane_t,
    pub shaderNum: c_int,
}

pub type cbrushside_t = cbrushside_s;

const _: () = assert!(core::mem::offset_of!(cbrushside_t, plane) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<cbrushside_t>() == 16);
    assert!(core::mem::offset_of!(cbrushside_t, shaderNum) == 8);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<cbrushside_t>() == 8);
    assert!(core::mem::offset_of!(cbrushside_t, shaderNum) == 4);
};
