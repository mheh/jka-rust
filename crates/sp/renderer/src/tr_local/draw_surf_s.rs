#![allow(non_camel_case_types, non_snake_case)]

use super::surface_type_t::surfaceType_t;

/// Raven `drawSurf_s` (typedef `drawSurf_t`).
///
/// Type definition source: `oracle/code/renderer/tr_local.h:608-611`
#[repr(C)]
pub struct drawSurf_t {
    /// bit combination for fast compares
    pub sort: u32,
    /// any of surface*_t
    pub surface: *mut surfaceType_t,
}

/// Raven manifest tag name; the typedef is `drawSurf_t`.
pub type drawSurf_s = drawSurf_t;

const _: () = assert!(core::mem::size_of::<drawSurf_t>() == 16);
const _: () = assert!(core::mem::offset_of!(drawSurf_t, sort) == 0);
const _: () = assert!(core::mem::offset_of!(drawSurf_t, surface) == 8);
