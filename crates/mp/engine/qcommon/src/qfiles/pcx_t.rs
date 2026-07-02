#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_ushort};

/// Raven `pcx_t` — PCX image file header.
///
/// Type definition source: `oracle/oracle/codemp/qcommon/../qcommon/qfiles.h:49-63`
#[repr(C)]
pub struct pcx_t {
    pub manufacturer: c_char,
    pub version: c_char,
    pub encoding: c_char,
    pub bits_per_pixel: c_char,
    pub xmin: c_ushort,
    pub ymin: c_ushort,
    pub xmax: c_ushort,
    pub ymax: c_ushort,
    pub hres: c_ushort,
    pub vres: c_ushort,
    pub palette: [u8; 48],
    pub reserved: c_char,
    pub color_planes: c_char,
    pub bytes_per_line: c_ushort,
    pub palette_type: c_ushort,
    pub filler: [c_char; 58],
    pub data: u8, // unbounded
}

const _: () = assert!(core::mem::size_of::<pcx_t>() == 130);
const _: () = assert!(core::mem::offset_of!(pcx_t, manufacturer) == 0);
const _: () = assert!(core::mem::offset_of!(pcx_t, version) == 1);
const _: () = assert!(core::mem::offset_of!(pcx_t, encoding) == 2);
const _: () = assert!(core::mem::offset_of!(pcx_t, bits_per_pixel) == 3);
const _: () = assert!(core::mem::offset_of!(pcx_t, xmin) == 4);
const _: () = assert!(core::mem::offset_of!(pcx_t, ymin) == 6);
const _: () = assert!(core::mem::offset_of!(pcx_t, xmax) == 8);
const _: () = assert!(core::mem::offset_of!(pcx_t, ymax) == 10);
const _: () = assert!(core::mem::offset_of!(pcx_t, hres) == 12);
const _: () = assert!(core::mem::offset_of!(pcx_t, vres) == 14);
const _: () = assert!(core::mem::offset_of!(pcx_t, palette) == 16);
const _: () = assert!(core::mem::offset_of!(pcx_t, reserved) == 64);
const _: () = assert!(core::mem::offset_of!(pcx_t, color_planes) == 65);
const _: () = assert!(core::mem::offset_of!(pcx_t, bytes_per_line) == 66);
const _: () = assert!(core::mem::offset_of!(pcx_t, palette_type) == 68);
const _: () = assert!(core::mem::offset_of!(pcx_t, filler) == 70);
const _: () = assert!(core::mem::offset_of!(pcx_t, data) == 128);
