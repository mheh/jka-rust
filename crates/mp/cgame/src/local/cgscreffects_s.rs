#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::qboolean;

/// Raven `cgscreffects_t` — cgame-side screen effect state (FOV kicks, shake, music ducking).
///
/// Type definition source: `oracle/codemp/cgame/cg_local.h:1030-1042`
#[repr(C)]
pub struct cgscreffects_t {
    pub FOV: f32,
    pub FOV2: f32,

    pub shake_intensity: f32,
    pub shake_duration: i32,
    pub shake_start: i32,

    pub music_volume_multiplier: f32,
    pub music_volume_time: i32,
    pub music_volume_set: qboolean,
}

const _: () = assert!(core::mem::size_of::<cgscreffects_t>() == 32);
const _: () = assert!(core::mem::offset_of!(cgscreffects_t, FOV) == 0);
const _: () = assert!(core::mem::offset_of!(cgscreffects_t, FOV2) == 4);
const _: () = assert!(core::mem::offset_of!(cgscreffects_t, shake_intensity) == 8);
const _: () = assert!(core::mem::offset_of!(cgscreffects_t, shake_duration) == 12);
const _: () = assert!(core::mem::offset_of!(cgscreffects_t, shake_start) == 16);
const _: () = assert!(core::mem::offset_of!(cgscreffects_t, music_volume_multiplier) == 20);
const _: () = assert!(core::mem::offset_of!(cgscreffects_t, music_volume_time) == 24);
const _: () = assert!(core::mem::offset_of!(cgscreffects_t, music_volume_set) == 28);
