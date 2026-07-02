#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_ushort;

/// Raven `TargaHeader` — TGA image file header.
///
/// Type definition source: `oracle/oracle/code/qcommon/../qcommon/qfiles.h:73-79`
#[repr(C)]
pub struct TargaHeader {
    pub id_length: u8,
    pub colormap_type: u8,
    pub image_type: u8,
    pub colormap_index: c_ushort,
    pub colormap_length: c_ushort,
    pub colormap_size: u8,
    pub x_origin: c_ushort,
    pub y_origin: c_ushort,
    pub width: c_ushort,
    pub height: c_ushort,
    pub pixel_size: u8,
    pub attributes: u8,
}

/// Raven typedef alias: `typedef struct _TargaHeader { ... } TargaHeader;`.
pub type _TargaHeader = TargaHeader;

const _: () = assert!(core::mem::size_of::<TargaHeader>() == 20);
const _: () = assert!(core::mem::offset_of!(TargaHeader, id_length) == 0);
const _: () = assert!(core::mem::offset_of!(TargaHeader, colormap_type) == 1);
const _: () = assert!(core::mem::offset_of!(TargaHeader, image_type) == 2);
const _: () = assert!(core::mem::offset_of!(TargaHeader, colormap_index) == 4);
const _: () = assert!(core::mem::offset_of!(TargaHeader, colormap_length) == 6);
const _: () = assert!(core::mem::offset_of!(TargaHeader, colormap_size) == 8);
const _: () = assert!(core::mem::offset_of!(TargaHeader, x_origin) == 10);
const _: () = assert!(core::mem::offset_of!(TargaHeader, y_origin) == 12);
const _: () = assert!(core::mem::offset_of!(TargaHeader, width) == 14);
const _: () = assert!(core::mem::offset_of!(TargaHeader, height) == 16);
const _: () = assert!(core::mem::offset_of!(TargaHeader, pixel_size) == 18);
const _: () = assert!(core::mem::offset_of!(TargaHeader, attributes) == 19);
