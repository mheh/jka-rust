//! `CachedAssets` — Raven `cachedAssets_t`.

use core::ffi::c_int;

use mp_qshared::shared::{qhandle_t, sfxHandle_t, vec4_t};

/// Number of crosshair shaders cached in `CachedAssets::crosshairShader`.
///
/// Source: `oracle/codemp/ui/ui_shared.h:104`
pub const NUM_CROSSHAIRS: usize = 9;

/// Number of player-settings FX preview shaders (`cachedAssets_t::fxPic`).
///
/// Source: `oracle/codemp/ui/ui_local.h:561` (`UI_NUMFX`)
pub const UI_NUMFX: usize = 7;

/// Raven `cachedAssets_t` — the UI-wide cached shader/sound/font handles and
/// fade settings menu rendering draws from, owned by
/// [`DisplayState`](super::display_state::DisplayState).
///
/// Type definition source: `oracle/codemp/ui/ui_shared.h:338-392`
#[derive(Debug, Clone, PartialEq, Default)]
#[doc(alias = "cachedAssets_t")]
#[allow(non_snake_case)]
pub struct CachedAssets {
    pub fontStr: String,
    pub cursorStr: String,
    pub gradientStr: String,
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
    pub fadeCycle: c_int,
    pub fadeAmount: f32,
    pub shadowX: f32,
    pub shadowY: f32,
    pub shadowColor: vec4_t,
    pub shadowFadeClamp: f32,
    pub fontRegistered: bool,

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
    pub fxPic: [qhandle_t; UI_NUMFX],
    pub crosshairShader: [qhandle_t; NUM_CROSSHAIRS],
}
