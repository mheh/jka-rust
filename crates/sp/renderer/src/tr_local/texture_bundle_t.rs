#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::vec3_t;

use super::image_s::image_t;
use super::tex_coord_gen_t::texCoordGen_t;
use super::tex_mod_info_t::texModInfo_t;

/// Raven `textureBundle_t` — a single texture layer of a shader stage: image,
/// tex-coord generation, tex-mod stack, and animation state.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:359-376`
#[repr(C)]
pub struct textureBundle_t {
    pub image: *mut image_t,

    pub tcGen: texCoordGen_t,
    pub tcGenVectors: *mut vec3_t,

    pub texMods: *mut texModInfo_t,
    pub numTexMods: i16,
    pub numImageAnimations: i16,
    pub imageAnimationSpeed: f32,

    pub isLightmap: bool,
    pub oneShotAnimMap: bool,
    pub vertexLightmap: bool,
    pub isVideoMap: bool,

    pub videoMapHandle: i32,
}

const _: () = assert!(core::mem::size_of::<textureBundle_t>() == 48);
const _: () = assert!(core::mem::offset_of!(textureBundle_t, image) == 0);
const _: () = assert!(core::mem::offset_of!(textureBundle_t, tcGen) == 8);
const _: () = assert!(core::mem::offset_of!(textureBundle_t, tcGenVectors) == 16);
const _: () = assert!(core::mem::offset_of!(textureBundle_t, texMods) == 24);
const _: () = assert!(core::mem::offset_of!(textureBundle_t, numTexMods) == 32);
const _: () = assert!(core::mem::offset_of!(textureBundle_t, numImageAnimations) == 34);
const _: () = assert!(core::mem::offset_of!(textureBundle_t, imageAnimationSpeed) == 36);
const _: () = assert!(core::mem::offset_of!(textureBundle_t, isLightmap) == 40);
const _: () = assert!(core::mem::offset_of!(textureBundle_t, oneShotAnimMap) == 41);
const _: () = assert!(core::mem::offset_of!(textureBundle_t, vertexLightmap) == 42);
const _: () = assert!(core::mem::offset_of!(textureBundle_t, isVideoMap) == 43);
const _: () = assert!(core::mem::offset_of!(textureBundle_t, videoMapHandle) == 44);
