#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_qshared::shared::{qboolean, NUM_FORCE_POWERS};

/// Raven `MAX_SIEGE_CLASSES` — "up to 128 classes".
///
/// Source: `oracle/codemp/game/bg_saga.h:12`
pub const MAX_SIEGE_CLASSES: usize = 128;

/// Raven `siegeClass_t` — a siege gametype player class definition.
///
/// Type definition source: `oracle/codemp/game/bg_saga.h:54-80`
#[repr(C)]
pub struct siegeClass_t {
    pub name: [c_char; 512],
    pub forcedModel: [c_char; 256],
    pub forcedSkin: [c_char; 256],
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

const _: () = assert!(core::mem::size_of::<siegeClass_t>() == 1548);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, name) == 0);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, forcedModel) == 512);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, forcedSkin) == 768);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, saber1) == 1024);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, saber2) == 1088);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, saberStance) == 1152);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, weapons) == 1156);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, forcePowerLevels) == 1160);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, classflags) == 1232);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, maxhealth) == 1236);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, starthealth) == 1240);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, maxarmor) == 1244);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, startarmor) == 1248);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, speed) == 1252);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, hasForcedSaberColor) == 1256);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, forcedSaberColor) == 1260);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, hasForcedSaber2Color) == 1264);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, forcedSaber2Color) == 1268);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, invenItems) == 1272);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, powerups) == 1276);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, uiPortraitShader) == 1280);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, uiPortrait) == 1284);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, classShader) == 1540);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, playerClass) == 1544);
