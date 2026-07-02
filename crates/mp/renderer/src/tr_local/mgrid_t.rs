#![allow(non_camel_case_types, non_snake_case)]

// Source: oracle/oracle/codemp/qcommon/qfiles.h:310
const MAXLIGHTMAPS: usize = 4;

/// Raven `mgrid_t` — per-vertex light-grid sample (ambient/direct light,
/// styles, direction).
///
// Raven: `byte pad[2]; // to align to a cache line` was left commented out
// in the original source.
/// Type definition source: `oracle/oracle/codemp/renderer/tr_local.h:970-977`
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
