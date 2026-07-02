#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use sp_qshared::shared::{qboolean, qhandle_t, sfxHandle_t, MAX_QPATH};

use crate::teams::team::team_t;

/// Raven `MAX_CUSTOM_BASIC_SOUNDS`.
///
/// Source: `oracle/oracle/code/game/g_shared.h:70`
pub const MAX_CUSTOM_BASIC_SOUNDS: usize = 14;

/// Raven `MAX_CUSTOM_COMBAT_SOUNDS`.
///
/// Source: `oracle/oracle/code/game/g_shared.h:71`
pub const MAX_CUSTOM_COMBAT_SOUNDS: usize = 17;

/// Raven `MAX_CUSTOM_EXTRA_SOUNDS`.
///
/// Source: `oracle/oracle/code/game/g_shared.h:72`
pub const MAX_CUSTOM_EXTRA_SOUNDS: usize = 36;

/// Raven `MAX_CUSTOM_JEDI_SOUNDS`.
///
/// Source: `oracle/oracle/code/game/g_shared.h:73`
pub const MAX_CUSTOM_JEDI_SOUNDS: usize = 22;

/// Raven `MAX_CUSTOM_SOUNDS`.
///
/// Raven: `#define MAX_CUSTOM_SOUNDS (MAX_CUSTOM_JEDI_SOUNDS + MAX_CUSTOM_EXTRA_SOUNDS + MAX_CUSTOM_COMBAT_SOUNDS + MAX_CUSTOM_BASIC_SOUNDS)`.
/// Source: `oracle/oracle/code/game/g_shared.h:74`
pub const MAX_CUSTOM_SOUNDS: usize =
    MAX_CUSTOM_JEDI_SOUNDS + MAX_CUSTOM_EXTRA_SOUNDS + MAX_CUSTOM_COMBAT_SOUNDS + MAX_CUSTOM_BASIC_SOUNDS;

/// Raven `clientInfo_t` — per-client rendering info shared between game and cgame.
///
/// Type definition source: `oracle/oracle/code/game/g_shared.h:76-103`
#[repr(C)]
pub struct clientInfo_t {
    pub infoValid: qboolean,

    pub name: [c_char; MAX_QPATH],
    pub team: team_t,

    /// updated by score servercmds
    pub score: i32,

    pub handicap: i32,

    pub legsModel: qhandle_t,
    pub legsSkin: qhandle_t,

    pub torsoModel: qhandle_t,
    pub torsoSkin: qhandle_t,

    pub headModel: qhandle_t,
    pub headSkin: qhandle_t,

    pub animFileIndex: i32,

    pub sounds: [sfxHandle_t; MAX_CUSTOM_SOUNDS],

    pub customBasicSoundDir: *mut c_char,
    pub customCombatSoundDir: *mut c_char,
    pub customExtraSoundDir: *mut c_char,
    pub customJediSoundDir: *mut c_char,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<clientInfo_t>() == 496);
const _: () = assert!(core::mem::offset_of!(clientInfo_t, infoValid) == 0);
const _: () = assert!(core::mem::offset_of!(clientInfo_t, name) == 4);
const _: () = assert!(core::mem::offset_of!(clientInfo_t, team) == 68);
const _: () = assert!(core::mem::offset_of!(clientInfo_t, score) == 72);
const _: () = assert!(core::mem::offset_of!(clientInfo_t, handicap) == 76);
const _: () = assert!(core::mem::offset_of!(clientInfo_t, legsModel) == 80);
const _: () = assert!(core::mem::offset_of!(clientInfo_t, legsSkin) == 84);
const _: () = assert!(core::mem::offset_of!(clientInfo_t, torsoModel) == 88);
const _: () = assert!(core::mem::offset_of!(clientInfo_t, torsoSkin) == 92);
const _: () = assert!(core::mem::offset_of!(clientInfo_t, headModel) == 96);
const _: () = assert!(core::mem::offset_of!(clientInfo_t, headSkin) == 100);
const _: () = assert!(core::mem::offset_of!(clientInfo_t, animFileIndex) == 104);
const _: () = assert!(core::mem::offset_of!(clientInfo_t, sounds) == 108);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, customBasicSoundDir) == 464);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, customCombatSoundDir) == 472);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, customExtraSoundDir) == 480);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, customJediSoundDir) == 488);
