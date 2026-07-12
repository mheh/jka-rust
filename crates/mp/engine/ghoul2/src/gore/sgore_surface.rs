#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_float, c_int};

/// Raven `SGoreSurface` — per-surface gore-decal state (shader, tag, fade/grow timing).
///
/// Type definition source: `oracle/codemp/ghoul2/G2_gore.h:44-57`
#[repr(C)]
pub struct SGoreSurface {
    pub shader: c_int,
    pub mGoreTag: c_int,
    pub mDeleteTime: c_int,
    pub mFadeTime: c_int,
    pub mFadeRGB: bool,

    pub mGoreGrowStartTime: c_int,
    pub mGoreGrowEndTime: c_int, // set this to -1 to disable growing
    // curscale = (curtime-mGoreGrowStartTime)*mGoreGrowFactor + mGoreGrowOffset;
    pub mGoreGrowFactor: c_float,
    pub mGoreGrowOffset: c_float,
}

const _: () = assert!(core::mem::size_of::<SGoreSurface>() == 36);
const _: () = assert!(core::mem::offset_of!(SGoreSurface, shader) == 0);
const _: () = assert!(core::mem::offset_of!(SGoreSurface, mGoreTag) == 4);
const _: () = assert!(core::mem::offset_of!(SGoreSurface, mDeleteTime) == 8);
const _: () = assert!(core::mem::offset_of!(SGoreSurface, mFadeTime) == 12);
const _: () = assert!(core::mem::offset_of!(SGoreSurface, mFadeRGB) == 16);
const _: () = assert!(core::mem::offset_of!(SGoreSurface, mGoreGrowStartTime) == 20);
const _: () = assert!(core::mem::offset_of!(SGoreSurface, mGoreGrowEndTime) == 24);
const _: () = assert!(core::mem::offset_of!(SGoreSurface, mGoreGrowFactor) == 28);
const _: () = assert!(core::mem::offset_of!(SGoreSurface, mGoreGrowOffset) == 32);
