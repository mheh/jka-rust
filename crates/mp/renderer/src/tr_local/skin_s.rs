#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_qshared::shared::MAX_QPATH;

use super::skin_surface_t::skinSurface_t;

/// Raven `skin_t` — a named skin binding surface names to shaders.
///
/// Raven: game path, including extension.
/// Type definition source: `oracle/codemp/renderer/tr_local.h:609-613`
#[repr(C)]
pub struct skin_t {
    pub name: [u8; MAX_QPATH as usize],
    pub numSurfaces: c_int,
    pub surfaces: [*mut skinSurface_t; 128],
}

pub type skin_s = skin_t;

const _: () = assert!(core::mem::offset_of!(skin_t, name) == 0);
const _: () = assert!(core::mem::offset_of!(skin_t, numSurfaces) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<skin_t>() == 1096);
    assert!(core::mem::offset_of!(skin_t, surfaces) == 72);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<skin_t>() == 580);
    assert!(core::mem::offset_of!(skin_t, surfaces) == 68);
};
