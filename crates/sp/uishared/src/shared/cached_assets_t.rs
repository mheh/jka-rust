#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::common::sp::ff::ff_handle_t::ffHandle_t;
use sp_qshared::shared::{qboolean, qhandle_t, sfxHandle_t, vec4_t};

/// Number of crosshair shaders cached in `cachedAssets_t::crosshairShader`.
///
/// Type definition source: `oracle/oracle/code/ui/ui_shared.h:111`
pub const NUM_CROSSHAIRS: usize = 9;

/// Raven `cachedAssets_t` — UI-wide cached shader/sound/force-feedback/font
/// handles and fade settings shared across menu rendering.
///
/// Type definition source: `oracle/oracle/code/ui/ui_shared.h:113-165`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct cachedAssets_t {
    pub qhMediumFont: qhandle_t,
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
    pub forceChosenSound: sfxHandle_t,
    pub forceUnchosenSound: sfxHandle_t,
    pub datapadmoveRollSound: sfxHandle_t,
    pub datapadmoveJumpSound: sfxHandle_t,
    pub datapadmoveSaberSound1: sfxHandle_t,
    pub datapadmoveSaberSound2: sfxHandle_t,
    pub datapadmoveSaberSound3: sfxHandle_t,
    pub datapadmoveSaberSound4: sfxHandle_t,
    pub datapadmoveSaberSound5: sfxHandle_t,
    pub datapadmoveSaberSound6: sfxHandle_t,

    pub nullSound: sfxHandle_t,

    // Raven: `#ifdef _IMMERSION` — force-feedback handles; layout reflects the
    // `_IMMERSION`-enabled build the offsets were captured against.
    pub menuEnterForce: ffHandle_t,
    pub menuExitForce: ffHandle_t,
    pub menuBuzzForce: ffHandle_t,
    pub itemFocusForce: ffHandle_t,

    pub fadeClamp: f32,
    pub fadeCycle: i32,
    pub fadeAmount: f32,
    pub shadowX: f32,
    pub shadowY: f32,
    pub shadowColor: vec4_t,
    pub shadowFadeClamp: f32,
    pub fontRegistered: qboolean,

    // player settings
    pub crosshairShader: [qhandle_t; NUM_CROSSHAIRS],
}

const _: () = assert!(core::mem::size_of::<cachedAssets_t>() == 212);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, qhMediumFont) == 0);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, cursor) == 4);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, gradientBar) == 8);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, scrollBarArrowUp) == 12);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, scrollBarArrowDown) == 16);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, scrollBarArrowLeft) == 20);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, scrollBarArrowRight) == 24);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, scrollBar) == 28);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, scrollBarThumb) == 32);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, buttonMiddle) == 36);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, buttonInside) == 40);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, solidBox) == 44);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, sliderBar) == 48);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, sliderThumb) == 52);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, menuEnterSound) == 56);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, menuExitSound) == 60);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, menuBuzzSound) == 64);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, itemFocusSound) == 68);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, forceChosenSound) == 72);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, forceUnchosenSound) == 76);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, datapadmoveRollSound) == 80);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, datapadmoveJumpSound) == 84);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, datapadmoveSaberSound1) == 88);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, datapadmoveSaberSound2) == 92);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, datapadmoveSaberSound3) == 96);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, datapadmoveSaberSound4) == 100);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, datapadmoveSaberSound5) == 104);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, datapadmoveSaberSound6) == 108);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, nullSound) == 112);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, menuEnterForce) == 116);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, menuExitForce) == 120);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, menuBuzzForce) == 124);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, itemFocusForce) == 128);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, fadeClamp) == 132);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, fadeCycle) == 136);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, fadeAmount) == 140);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, shadowX) == 144);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, shadowY) == 148);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, shadowColor) == 152);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, shadowFadeClamp) == 168);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, fontRegistered) == 172);
const _: () = assert!(core::mem::offset_of!(cachedAssets_t, crosshairShader) == 176);
