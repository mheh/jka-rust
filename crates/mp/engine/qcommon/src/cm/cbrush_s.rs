#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec3_t;

use super::cbrushside_s::cbrushside_t;

/// Raven `cbrush_t` — a collision-model brush (convex hull of `cbrushside_t` planes).
///
/// Type definition source: `oracle/codemp/qcommon/cm_local.h:68-75`
#[repr(C)]
pub struct cbrush_s {
    pub shaderNum: i32, // the shader that determined the contents
    pub contents: i32,
    pub bounds: [vec3_t; 2],
    pub sides: *mut cbrushside_t,
    pub numsides: u16,
    pub checkcount: u16, // to avoid repeated testings
}

pub type cbrush_t = cbrush_s;

const _: () = assert!(core::mem::offset_of!(cbrush_t, shaderNum) == 0);
const _: () = assert!(core::mem::offset_of!(cbrush_t, contents) == 4);
const _: () = assert!(core::mem::offset_of!(cbrush_t, bounds) == 8);
const _: () = assert!(core::mem::offset_of!(cbrush_t, sides) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<cbrush_t>() == 48);
    assert!(core::mem::offset_of!(cbrush_t, numsides) == 40);
    assert!(core::mem::offset_of!(cbrush_t, checkcount) == 42);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<cbrush_t>() == 40);
    assert!(core::mem::offset_of!(cbrush_t, numsides) == 36);
    assert!(core::mem::offset_of!(cbrush_t, checkcount) == 38);
};
