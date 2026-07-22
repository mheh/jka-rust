#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::{qboolean, qfalse, NUM_FORCE_POWERS};

/// Raven `MAX_SIEGE_CLASSES` — "up to 128 classes".
///
/// Source: `oracle/codemp/game/bg_saga.h:12`
pub const MAX_SIEGE_CLASSES: usize = 128;

/// Raven `siegeClass_t` — a siege gametype player class definition.
///
/// Every text field is an owned `String` (loaded once from the class file, read
/// as `&str` at the seam). The struct is bg-internal (census: only stored in the
/// `bgSiegeClasses` `Vec` and reached by `*mut siegeClass_t`, never crossing the
/// ABI), so the former `#[repr(C)]` + `offset_of!`/`size_of` layout asserts no
/// longer bind and are dropped.
/// Type definition source: `oracle/codemp/game/bg_saga.h:54-80`
pub struct siegeClass_t {
    pub name: String,
    pub forcedModel: String,
    pub forcedSkin: String,
    pub saber1: String,
    pub saber2: String,
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
    pub uiPortrait: String,
    pub classShader: i32,
    // SPC_INFANTRY . ..
    pub playerClass: i16,
}

impl Default for siegeClass_t {
    /// Raven's `siegeClass_t bgSiegeClasses[MAX_SIEGE_CLASSES]` zeroed static:
    /// every scalar field starts at `0`, the owned `String`s empty.
    fn default() -> Self {
        siegeClass_t {
            name: String::new(),
            forcedModel: String::new(),
            forcedSkin: String::new(),
            saber1: String::new(),
            saber2: String::new(),
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
            uiPortrait: String::new(),
            classShader: 0,
            playerClass: 0,
        }
    }
}
