#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use sp_qshared::shared::MAX_QPATH;

/// Raven `image_t` — a loaded GL texture: name, frame-used stat, dimensions,
/// GL binding, and mip/clamp state.
///
/// Raven: game path, including extension / for texture usage in frame
/// statistics / source image / gl texture binding / GL_CLAMP or GL_REPEAT.
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:115-139`
#[repr(C)]
pub struct image_t {
    pub imgName: [c_char; MAX_QPATH as usize],
    /// for texture usage in frame statistics
    pub frameUsed: i32,

    /// source image
    pub width: u16,
    pub height: u16,

    /// gl texture binding
    pub texnum: u32,
    pub internalFormat: i32,
    /// GL_CLAMP or GL_REPEAT
    pub wrapClampMode: i32,

    pub mipmap: bool,

    pub allowPicmip: bool,
    pub iLastLevelUsedOn: i16,
}

/// Manifest tag name alias.
pub type image_s = image_t;

const _: () = assert!(core::mem::size_of::<image_t>() == 88);
const _: () = assert!(core::mem::offset_of!(image_t, imgName) == 0);
const _: () = assert!(core::mem::offset_of!(image_t, frameUsed) == 64);
const _: () = assert!(core::mem::offset_of!(image_t, width) == 68);
const _: () = assert!(core::mem::offset_of!(image_t, height) == 70);
const _: () = assert!(core::mem::offset_of!(image_t, texnum) == 72);
const _: () = assert!(core::mem::offset_of!(image_t, internalFormat) == 76);
const _: () = assert!(core::mem::offset_of!(image_t, wrapClampMode) == 80);
const _: () = assert!(core::mem::offset_of!(image_t, mipmap) == 84);
const _: () = assert!(core::mem::offset_of!(image_t, allowPicmip) == 85);
const _: () = assert!(core::mem::offset_of!(image_t, iLastLevelUsedOn) == 86);
