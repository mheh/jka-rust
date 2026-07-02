#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_float;

/// Raven `#define MAX_LODS (8)` — max mesh LODs a gore texture-coordinate set covers.
///
/// Type definition source: `oracle/oracle/code/ghoul2/ghoul2_gore.h:3`
pub const MAX_LODS: usize = 8;

/// Raven `GoreTextureCoordinates` — per-LOD gore-decal texture coordinate buffers.
///
/// Raven: constructor zero-inits `tex`; destructor `Z_Free`s any non-null entry.
/// Ownership/free-on-drop behavior is not modeled at the ABI-layout level here.
/// Type definition source: `oracle/oracle/code/ghoul2/ghoul2_gore.h:4-29`
#[repr(C)]
pub struct GoreTextureCoordinates {
    pub tex: [*mut c_float; MAX_LODS],
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<GoreTextureCoordinates>() == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(GoreTextureCoordinates, tex) == 0);
