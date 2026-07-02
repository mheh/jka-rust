#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::vec2_t;

/// Raven `surfaceSprite_t` — sprite-generation parameters attached to a shader
/// stage (billboards, wind sway, fade, and facing behavior).
///
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:350-357`
#[repr(C)]
pub struct surfaceSprite_t {
    pub surfaceSpriteType: i32,
    pub width: f32,
    pub height: f32,
    pub density: f32,
    pub wind: f32,
    pub windIdle: f32,
    pub fadeDist: f32,
    pub fadeMax: f32,
    pub fadeScale: f32,
    pub fxAlphaStart: f32,
    pub fxAlphaEnd: f32,
    pub fxDuration: f32,
    pub vertSkew: f32,
    pub variance: vec2_t,
    pub fxGrow: vec2_t,
    /// Hangdown on vertical sprites, faceup on others.
    pub facing: i32,
}

/// Manifest tag name alias.
pub type surfaceSprite_s = surfaceSprite_t;

const _: () = assert!(core::mem::size_of::<surfaceSprite_t>() == 72);
const _: () = assert!(core::mem::offset_of!(surfaceSprite_t, surfaceSpriteType) == 0);
const _: () = assert!(core::mem::offset_of!(surfaceSprite_t, width) == 4);
const _: () = assert!(core::mem::offset_of!(surfaceSprite_t, height) == 8);
const _: () = assert!(core::mem::offset_of!(surfaceSprite_t, density) == 12);
const _: () = assert!(core::mem::offset_of!(surfaceSprite_t, wind) == 16);
const _: () = assert!(core::mem::offset_of!(surfaceSprite_t, windIdle) == 20);
const _: () = assert!(core::mem::offset_of!(surfaceSprite_t, fadeDist) == 24);
const _: () = assert!(core::mem::offset_of!(surfaceSprite_t, fadeMax) == 28);
const _: () = assert!(core::mem::offset_of!(surfaceSprite_t, fadeScale) == 32);
const _: () = assert!(core::mem::offset_of!(surfaceSprite_t, fxAlphaStart) == 36);
const _: () = assert!(core::mem::offset_of!(surfaceSprite_t, fxAlphaEnd) == 40);
const _: () = assert!(core::mem::offset_of!(surfaceSprite_t, fxDuration) == 44);
const _: () = assert!(core::mem::offset_of!(surfaceSprite_t, vertSkew) == 48);
const _: () = assert!(core::mem::offset_of!(surfaceSprite_t, variance) == 52);
const _: () = assert!(core::mem::offset_of!(surfaceSprite_t, fxGrow) == 60);
const _: () = assert!(core::mem::offset_of!(surfaceSprite_t, facing) == 68);
