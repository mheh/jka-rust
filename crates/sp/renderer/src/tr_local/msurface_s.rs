#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use super::shader_s::shader_s;
use super::surface_type_t::surfaceType_t;

/// Raven `msurface_s` (typedef `msurface_t`) — a renderable BSP surface.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:784-790`
#[repr(C)]
pub struct msurface_t {
    /// if == tr.viewCount, already added
    pub viewCount: c_int,
    pub shader: *mut shader_s,
    pub fogIndex: c_int,

    /// any of srf*_t
    pub data: *mut surfaceType_t,
}

pub type msurface_s = msurface_t;

const _: () = assert!(core::mem::size_of::<msurface_t>() == 32);
const _: () = assert!(core::mem::offset_of!(msurface_t, viewCount) == 0);
const _: () = assert!(core::mem::offset_of!(msurface_t, shader) == 8);
const _: () = assert!(core::mem::offset_of!(msurface_t, fogIndex) == 16);
const _: () = assert!(core::mem::offset_of!(msurface_t, data) == 24);
