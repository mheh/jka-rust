#![allow(non_camel_case_types, non_snake_case)]

// Source: oracle/code/qcommon/qfiles.h:310
const MAXLIGHTMAPS: usize = 4;

/// Raven `mgrid_t` — per-vertex light-grid sample (ambient/direct light,
/// styles, direction).
///
/// Type definition source: `oracle/code/renderer/tr_local.h:883-888`
#[repr(C)]
pub struct mgrid_t {
    pub ambientLight: [[u8; 3]; MAXLIGHTMAPS],
    pub directLight: [[u8; 3]; MAXLIGHTMAPS],
    pub styles: [u8; MAXLIGHTMAPS],
    pub latLong: [u8; 2],
}

const _: () = assert!(core::mem::size_of::<mgrid_t>() == 30);
const _: () = assert!(core::mem::offset_of!(mgrid_t, ambientLight) == 0);
const _: () = assert!(core::mem::offset_of!(mgrid_t, directLight) == 12);
const _: () = assert!(core::mem::offset_of!(mgrid_t, styles) == 24);
const _: () = assert!(core::mem::offset_of!(mgrid_t, latLong) == 28);
