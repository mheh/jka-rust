#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_float;

use mp_qshared::shared::vec3_t;

use super::image_s::image_t;
use super::tex_coord_gen_t::texCoordGen_t;
use super::tex_mod_info_t::texModInfo_t;

/// Raven `textureBundle_t`.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:372-389`
#[repr(C)]
pub struct textureBundle_t {
    pub image: *mut image_t,

    pub tcGen: texCoordGen_t,
    pub tcGenVectors: *mut vec3_t,

    pub texMods: *mut texModInfo_t,
    pub numTexMods: i16,
    pub numImageAnimations: i16,
    pub imageAnimationSpeed: c_float,

    pub isLightmap: bool,
    pub oneShotAnimMap: bool,
    pub vertexLightmap: bool,
    pub isVideoMap: bool,

    pub videoMapHandle: i32,
}

const _: () = assert!(core::mem::offset_of!(textureBundle_t, image) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<textureBundle_t>() == 48);
    assert!(core::mem::offset_of!(textureBundle_t, tcGen) == 8);
    assert!(core::mem::offset_of!(textureBundle_t, tcGenVectors) == 16);
    assert!(core::mem::offset_of!(textureBundle_t, texMods) == 24);
    assert!(core::mem::offset_of!(textureBundle_t, numTexMods) == 32);
    assert!(core::mem::offset_of!(textureBundle_t, numImageAnimations) == 34);
    assert!(core::mem::offset_of!(textureBundle_t, imageAnimationSpeed) == 36);
    assert!(core::mem::offset_of!(textureBundle_t, isLightmap) == 40);
    assert!(core::mem::offset_of!(textureBundle_t, oneShotAnimMap) == 41);
    assert!(core::mem::offset_of!(textureBundle_t, vertexLightmap) == 42);
    assert!(core::mem::offset_of!(textureBundle_t, isVideoMap) == 43);
    assert!(core::mem::offset_of!(textureBundle_t, videoMapHandle) == 44);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<textureBundle_t>() == 32);
    assert!(core::mem::offset_of!(textureBundle_t, tcGen) == 4);
    assert!(core::mem::offset_of!(textureBundle_t, tcGenVectors) == 8);
    assert!(core::mem::offset_of!(textureBundle_t, texMods) == 12);
    assert!(core::mem::offset_of!(textureBundle_t, numTexMods) == 16);
    assert!(core::mem::offset_of!(textureBundle_t, numImageAnimations) == 18);
    assert!(core::mem::offset_of!(textureBundle_t, imageAnimationSpeed) == 20);
    assert!(core::mem::offset_of!(textureBundle_t, isLightmap) == 24);
    assert!(core::mem::offset_of!(textureBundle_t, oneShotAnimMap) == 25);
    assert!(core::mem::offset_of!(textureBundle_t, vertexLightmap) == 26);
    assert!(core::mem::offset_of!(textureBundle_t, isVideoMap) == 27);
    assert!(core::mem::offset_of!(textureBundle_t, videoMapHandle) == 28);
};
