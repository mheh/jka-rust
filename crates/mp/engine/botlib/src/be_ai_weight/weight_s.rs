#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use super::fuzzyseperator_s::fuzzyseperator_t;

/// Raven `weight_t` — named fuzzy weight (root of a separator tree).
///
/// Type definition source: `oracle/codemp/botlib/be_ai_weight.h:32-36`
#[repr(C)]
pub struct weight_t {
    pub name: *mut c_char,
    pub firstseperator: *mut fuzzyseperator_t,
}

pub type weight_s = weight_t;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<weight_t>() == 16);
    assert!(core::mem::offset_of!(weight_t, name) == 0);
    assert!(core::mem::offset_of!(weight_t, firstseperator) == 8);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<weight_t>() == 8);
    assert!(core::mem::offset_of!(weight_t, name) == 0);
    assert!(core::mem::offset_of!(weight_t, firstseperator) == 4);
};
