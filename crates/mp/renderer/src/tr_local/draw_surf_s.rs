#![allow(non_camel_case_types, non_snake_case)]

use super::surface_type_t::surfaceType_t;

/// Raven `drawSurf_s` (typedef `drawSurf_t`).
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:680-683`
#[repr(C)]
pub struct drawSurf_t {
    /// bit combination for fast compares
    pub sort: u32,
    /// any of surface*_t
    pub surface: *mut surfaceType_t,
}

/// Raven manifest tag name; the typedef is `drawSurf_t`.
pub type drawSurf_s = drawSurf_t;

const _: () = assert!(core::mem::offset_of!(drawSurf_t, sort) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<drawSurf_t>() == 16);
    assert!(core::mem::offset_of!(drawSurf_t, surface) == 8);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<drawSurf_t>() == 8);
    assert!(core::mem::offset_of!(drawSurf_t, surface) == 4);
};
