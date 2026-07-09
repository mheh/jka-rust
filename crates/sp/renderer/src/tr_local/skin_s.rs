#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::c_int;

use sp_qshared::shared::MAX_QPATH;

use super::skin_surface_t::skinSurface_t;

/// Raven `skin_t` — a named skin binding surface names to shaders.
///
/// Raven: game path, including extension.
/// Type definition source: `oracle/code/renderer/tr_local.h:536-540`
#[repr(C)]
pub struct skin_t {
    pub name: [u8; MAX_QPATH as usize],
    pub numSurfaces: c_int,
    pub surfaces: [*mut skinSurface_t; 128],
}

pub type skin_s = skin_t;

const _: () = assert!(core::mem::size_of::<skin_t>() == 1096);
const _: () = assert!(core::mem::offset_of!(skin_t, name) == 0);
const _: () = assert!(core::mem::offset_of!(skin_t, numSurfaces) == 64);
const _: () = assert!(core::mem::offset_of!(skin_t, surfaces) == 72);
