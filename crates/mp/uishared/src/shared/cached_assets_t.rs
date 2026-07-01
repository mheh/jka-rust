#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::{qhandle_t, qboolean, sfxHandle_t, vec4_t};

/// Number of crosshair shaders cached in `cachedAssets_t::crosshairShader`.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_shared.h:104`
pub const NUM_CROSSHAIRS: usize = 9;

/// Raven `cachedAssets_t` — UI-wide cached shader/sound/font handles and fade
/// settings shared across menu rendering.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_shared.h:338-392`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct cachedAssets_t {
    pub fontStr: *const i8,
    pub cursorStr: *const i8,
    pub gradientStr: *const i8,
    pub qhSmallFont: qhandle_t,
    pub qhSmall2Font: qhandle_t,
    pub qhMediumFont: qhandle_t,
    pub qhBigFont: qhandle_t,
    pub cursor: qhandle_t,
    pub gradientBar: qhandle_t,
    pub scrollBarArrowUp: qhandle_t,
    pub scrollBarArrowDown: qhandle_t,
    pub scrollBarArrowLeft: qhandle_t,
    pub scrollBarArrowRight: qhandle_t,
    pub scrollBar: qhandle_t,
    pub scrollBarThumb: qhandle_t,
    pub buttonMiddle: qhandle_t,
    pub buttonInside: qhandle_t,
    pub solidBox: qhandle_t,
    pub sliderBar: qhandle_t,
    pub sliderThumb: qhandle_t,
    pub menuEnterSound: sfxHandle_t,
    pub menuExitSound: sfxHandle_t,
    pub menuBuzzSound: sfxHandle_t,
    pub itemFocusSound: sfxHandle_t,
    pub fadeClamp: f32,
    pub fadeCycle: i32,
    pub fadeAmount: f32,
    pub shadowX: f32,
    pub shadowY: f32,
    pub shadowColor: vec4_t,
    pub shadowFadeClamp: f32,
    pub fontRegistered: qboolean,

    pub needPass: qhandle_t,
    pub noForce: qhandle_t,
    pub forceRestrict: qhandle_t,
    pub saberOnly: qhandle_t,
    pub trueJedi: qhandle_t,

    pub moveRollSound: sfxHandle_t,
    pub moveJumpSound: sfxHandle_t,
    pub datapadmoveSaberSound1: sfxHandle_t,
    pub datapadmoveSaberSound2: sfxHandle_t,
    pub datapadmoveSaberSound3: sfxHandle_t,
    pub datapadmoveSaberSound4: sfxHandle_t,
    pub datapadmoveSaberSound5: sfxHandle_t,
    pub datapadmoveSaberSound6: sfxHandle_t,

    // player settings
    pub fxBasePic: qhandle_t,
    pub fxPic: [qhandle_t; 7],
    pub crosshairShader: [qhandle_t; NUM_CROSSHAIRS],
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<cachedAssets_t>() == 272);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, fontStr) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, cursorStr) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, gradientStr) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, qhSmallFont) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, qhSmall2Font) == 28);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, qhMediumFont) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, qhBigFont) == 36);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, cursor) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, gradientBar) == 44);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, scrollBarArrowUp) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, scrollBarArrowDown) == 52);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, scrollBarArrowLeft) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, scrollBarArrowRight) == 60);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, scrollBar) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, scrollBarThumb) == 68);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, buttonMiddle) == 72);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, buttonInside) == 76);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, solidBox) == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, sliderBar) == 84);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, sliderThumb) == 88);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, menuEnterSound) == 92);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, menuExitSound) == 96);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, menuBuzzSound) == 100);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, itemFocusSound) == 104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, fadeClamp) == 108);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, fadeCycle) == 112);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, fadeAmount) == 116);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, shadowX) == 120);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, shadowY) == 124);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, shadowColor) == 128);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, shadowFadeClamp) == 144);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, fontRegistered) == 148);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, needPass) == 152);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, noForce) == 156);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, forceRestrict) == 160);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, saberOnly) == 164);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, trueJedi) == 168);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, moveRollSound) == 172);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, moveJumpSound) == 176);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, datapadmoveSaberSound1) == 180);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, datapadmoveSaberSound2) == 184);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, datapadmoveSaberSound3) == 188);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, datapadmoveSaberSound4) == 192);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, datapadmoveSaberSound5) == 196);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, datapadmoveSaberSound6) == 200);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, fxBasePic) == 204);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, fxPic) == 208);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, crosshairShader) == 236);
