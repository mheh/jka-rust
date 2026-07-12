#![allow(non_camel_case_types, non_snake_case)]

use crate::tr_local::image_s::image_t;

/// Raven `skyParms_t`.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:449-452`
#[repr(C)]
pub struct skyParms_t {
    pub cloudHeight: f32,
    pub outerbox: [*mut image_t; 6],
}

const _: () = assert!(core::mem::offset_of!(skyParms_t, cloudHeight) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<skyParms_t>() == 56);
    assert!(core::mem::offset_of!(skyParms_t, outerbox) == 8);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<skyParms_t>() == 28);
    assert!(core::mem::offset_of!(skyParms_t, outerbox) == 4);
};
