#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_qshared::shared::{qboolean, qfalse, NUM_FORCE_POWERS};

/// Raven `MAX_SIEGE_CLASSES` — "up to 128 classes".
///
/// Source: `oracle/codemp/game/bg_saga.h:12`
pub const MAX_SIEGE_CLASSES: usize = 128;

/// Raven `siegeClass_t` — a siege gametype player class definition.
///
/// `forcedModel`/`forcedSkin` are owned `String`s (loaded once from the class
/// file, read as `&str` at the seam). The struct is bg-internal (census: only
/// stored in the `bgSiegeClasses` `Vec` and reached by `*mut siegeClass_t`,
/// never crossing the ABI), so the former `#[repr(C)]` + `offset_of!`/`size_of`
/// layout asserts no longer bind and are dropped. The remaining `[c_char; N]`
/// fields (`name`/`saber1`/`saber2`/`uiPortrait`) still feed pointer-shaped
/// consumers (`BG_SiegeCheckClassLegality`'s out-param, `WP_SaberParseParms`,
/// `BG_GetUIPortraitFile`) and migrate with those reshapes in a later batch.
/// Type definition source: `oracle/codemp/game/bg_saga.h:54-80`
pub struct siegeClass_t {
    pub name: [c_char; 512],
    pub forcedModel: String,
    pub forcedSkin: String,
    pub saber1: [c_char; 64],
    pub saber2: [c_char; 64],
    pub saberStance: i32,
    pub weapons: i32,
    pub forcePowerLevels: [i32; NUM_FORCE_POWERS as usize],
    pub classflags: i32,
    pub maxhealth: i32,
    pub starthealth: i32,
    pub maxarmor: i32,
    pub startarmor: i32,
    pub speed: f32,
    pub hasForcedSaberColor: qboolean,
    pub forcedSaberColor: i32,
    pub hasForcedSaber2Color: qboolean,
    pub forcedSaber2Color: i32,
    pub invenItems: i32,
    pub powerups: i32,
    pub uiPortraitShader: i32,
    pub uiPortrait: [c_char; 256],
    pub classShader: i32,
    // SPC_INFANTRY . ..
    pub playerClass: i16,
}

impl Default for siegeClass_t {
    /// Raven's `siegeClass_t bgSiegeClasses[MAX_SIEGE_CLASSES]` zeroed static:
    /// every scalar field starts at `0`, the `[c_char; N]` buffers NUL-filled,
    /// the owned `String`s empty.
    fn default() -> Self {
        siegeClass_t {
            name: [0; 512],
            forcedModel: String::new(),
            forcedSkin: String::new(),
            saber1: [0; 64],
            saber2: [0; 64],
            saberStance: 0,
            weapons: 0,
            forcePowerLevels: [0; NUM_FORCE_POWERS as usize],
            classflags: 0,
            maxhealth: 0,
            starthealth: 0,
            maxarmor: 0,
            startarmor: 0,
            speed: 0.0,
            hasForcedSaberColor: qfalse,
            forcedSaberColor: 0,
            hasForcedSaber2Color: qfalse,
            forcedSaber2Color: 0,
            invenItems: 0,
            powerups: 0,
            uiPortraitShader: 0,
            uiPortrait: [0; 256],
            classShader: 0,
            playerClass: 0,
        }
    }
}
