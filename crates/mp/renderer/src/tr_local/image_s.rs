#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_qshared::shared::MAX_QPATH;

/// Raven `image_t` — a loaded GL texture: name, dimensions, GL binding, and
/// mip/clamp state.
///
/// Raven: game path, including extension / after power of two and picmip
/// but not including clamp to MAX_TEXTURE_SIZE / gl texture binding / for
/// texture usage in frame statistics / GL_CLAMP or GL_REPEAT.
/// Type definition source: `oracle/codemp/renderer/tr_local.h:136-151`
#[repr(C)]
pub struct image_t {
    pub imgName: [c_char; MAX_QPATH as usize],
    pub width: u16,
    pub height: u16,
    pub texnum: u32,

    pub frameUsed: i32,

    pub internalFormat: i32,
    pub wrapClampMode: i32,

    pub mipmap: bool,
    pub allowPicmip: bool,

    pub iLastLevelUsedOn: i16,
}

/// Manifest tag name alias.
pub type image_s = image_t;

const _: () = assert!(core::mem::size_of::<image_t>() == 88);
const _: () = assert!(core::mem::offset_of!(image_t, imgName) == 0);
const _: () = assert!(core::mem::offset_of!(image_t, width) == 64);
const _: () = assert!(core::mem::offset_of!(image_t, height) == 66);
const _: () = assert!(core::mem::offset_of!(image_t, texnum) == 68);
const _: () = assert!(core::mem::offset_of!(image_t, frameUsed) == 72);
const _: () = assert!(core::mem::offset_of!(image_t, internalFormat) == 76);
const _: () = assert!(core::mem::offset_of!(image_t, wrapClampMode) == 80);
const _: () = assert!(core::mem::offset_of!(image_t, mipmap) == 84);
const _: () = assert!(core::mem::offset_of!(image_t, allowPicmip) == 85);
const _: () = assert!(core::mem::offset_of!(image_t, iLastLevelUsedOn) == 86);
