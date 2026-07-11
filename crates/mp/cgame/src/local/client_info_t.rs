#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;
use core::ffi::c_void;

use mp_bg::public::gender::gender_t;
use mp_bg::public::team::team_t;
use mp_qshared::common::mp::qcommon::saber::saber_info::{saberInfo_t, MAX_SABERS};
use mp_qshared::shared::{qboolean, qhandle_t, sfxHandle_t, vec3_t, MAX_QPATH};

/// Raven `MAX_TEAMNAME`.
///
/// Source: `oracle/codemp/game/q_shared.h:12`
pub const MAX_TEAMNAME: usize = 32;

/// Raven `MAX_CUSTOM_SOUNDS`.
///
/// Raven: rww - Note that for now these must all be the same, because of the way I am
/// Source: `oracle/codemp/cgame/cg_local.h:193`
pub const MAX_CUSTOM_SOUNDS: usize = 40;

/// Raven `MAX_CUSTOM_COMBAT_SOUNDS`.
///
/// Source: `oracle/codemp/cgame/cg_local.h:187`
pub const MAX_CUSTOM_COMBAT_SOUNDS: usize = 40;

/// Raven `MAX_CUSTOM_EXTRA_SOUNDS`.
///
/// Source: `oracle/codemp/cgame/cg_local.h:188`
pub const MAX_CUSTOM_EXTRA_SOUNDS: usize = 40;

/// Raven `MAX_CUSTOM_JEDI_SOUNDS`.
///
/// Source: `oracle/codemp/cgame/cg_local.h:189`
pub const MAX_CUSTOM_JEDI_SOUNDS: usize = 40;

/// Raven `MAX_CUSTOM_SIEGE_SOUNDS`.
///
/// Source: `oracle/codemp/game/bg_public.h:140`
pub const MAX_CUSTOM_SIEGE_SOUNDS: usize = 30;

/// Raven `MAX_CUSTOM_DUEL_SOUNDS`.
///
/// Source: `oracle/codemp/cgame/cg_local.h:191`
pub const MAX_CUSTOM_DUEL_SOUNDS: usize = 40;

/// Raven `clientInfo_t` — per-client rendering/gameplay info cached by cgame.
///
/// Type definition source: `oracle/codemp/cgame/cg_local.h:196-315`
#[repr(C)]
pub struct clientInfo_t {
    pub infoValid: qboolean,

    pub colorOverride: [f32; 3],

    pub saber: [saberInfo_t; MAX_SABERS],
    //TODO: Port ghoul2Weapons element type (CGhoul2Info_v*)
    // Source: oracle/codemp/cgame/cg_local.h:202
    pub ghoul2Weapons: [*mut c_void; MAX_SABERS],

    pub saberName: [c_char; 64],
    pub saber2Name: [c_char; 64],

    pub name: [c_char; MAX_QPATH],
    pub team: team_t,

    pub duelTeam: i32,

    /// 0 = not bot, 1-5 = bot
    pub botSkill: i32,

    pub frame: i32,

    pub color1: vec3_t,
    pub color2: vec3_t,

    pub icolor1: i32,
    pub icolor2: i32,

    /// updated by score servercmds
    pub score: i32,
    /// location index for team mode
    pub location: i32,
    /// you only get this info about your teammates
    pub health: i32,
    pub armor: i32,
    pub curWeapon: i32,

    pub handicap: i32,
    /// in tourney mode
    pub wins: i32,
    pub losses: i32,

    /// task in teamplay (offence/defence)
    pub teamTask: i32,
    /// true when this is a team leader
    pub teamLeader: qboolean,

    /// so can display quad/flag status
    pub powerups: i32,

    pub medkitUsageTime: i32,

    pub breathPuffTime: i32,

    // when clientinfo is changed, the loading of models/skins/sounds
    // can be deferred until you are dead, to prevent hitches in
    // gameplay
    pub modelName: [c_char; MAX_QPATH],
    pub skinName: [c_char; MAX_QPATH],
    pub forcePowers: [c_char; MAX_QPATH],

    pub teamName: [c_char; MAX_TEAMNAME],

    pub corrTime: i32,

    pub lastHeadAngles: vec3_t,
    pub lookTime: i32,

    pub brokenLimbs: i32,

    pub deferred: qboolean,

    /// true if using the new mission pack animations
    pub newAnims: qboolean,
    /// true if legs yaw is always the same as torso yaw
    pub fixedlegs: qboolean,
    /// true if torso never changes yaw
    pub fixedtorso: qboolean,

    /// move head in icon views
    pub headOffset: vec3_t,
    /// from model
    pub gender: gender_t,

    pub legsModel: qhandle_t,
    pub legsSkin: qhandle_t,

    pub torsoModel: qhandle_t,
    pub torsoSkin: qhandle_t,

    //TODO: Port ghoul2Model (CGhoul2Info_v*)
    // Source: oracle/codemp/cgame/cg_local.h:279
    pub ghoul2Model: *mut c_void,

    pub modelIcon: qhandle_t,

    pub bolt_rhand: qhandle_t,
    pub bolt_lhand: qhandle_t,

    pub bolt_head: qhandle_t,

    pub bolt_motion: qhandle_t,

    pub bolt_llumbar: qhandle_t,

    pub siegeIndex: i32,
    pub siegeDesiredTeam: i32,

    pub sounds: [sfxHandle_t; MAX_CUSTOM_SOUNDS],
    pub combatSounds: [sfxHandle_t; MAX_CUSTOM_COMBAT_SOUNDS],
    pub extraSounds: [sfxHandle_t; MAX_CUSTOM_EXTRA_SOUNDS],
    pub jediSounds: [sfxHandle_t; MAX_CUSTOM_JEDI_SOUNDS],
    pub siegeSounds: [sfxHandle_t; MAX_CUSTOM_SIEGE_SOUNDS],
    pub duelSounds: [sfxHandle_t; MAX_CUSTOM_DUEL_SOUNDS],

    pub legsAnim: i32,
    pub torsoAnim: i32,

    /// time before next blink. If a minus value, we are in blink mode
    pub facial_blink: f32,
    /// time before next frown. If a minus value, we are in frown mode
    pub facial_frown: f32,
    /// time before next aux. If a minus value, we are in aux mode
    pub facial_aux: f32,

    /// do crazy amount of smoothing
    pub superSmoothTime: i32,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<clientInfo_t>() == 5920);
const _: () = assert!(core::mem::offset_of!(clientInfo_t, infoValid) == 0);
const _: () = assert!(core::mem::offset_of!(clientInfo_t, colorOverride) == 4);
const _: () = assert!(core::mem::offset_of!(clientInfo_t, saber) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, ghoul2Weapons) == 4328);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, saberName) == 4344);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, saber2Name) == 4408);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, name) == 4472);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, team) == 4536);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, duelTeam) == 4540);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, botSkill) == 4544);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, frame) == 4548);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, color1) == 4552);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, color2) == 4564);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, icolor1) == 4576);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, icolor2) == 4580);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, score) == 4584);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, location) == 4588);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, health) == 4592);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, armor) == 4596);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, curWeapon) == 4600);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, handicap) == 4604);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, wins) == 4608);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, losses) == 4612);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, teamTask) == 4616);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, teamLeader) == 4620);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, powerups) == 4624);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, medkitUsageTime) == 4628);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, breathPuffTime) == 4632);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, modelName) == 4636);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, skinName) == 4700);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, forcePowers) == 4764);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, teamName) == 4828);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, corrTime) == 4860);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, lastHeadAngles) == 4864);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, lookTime) == 4876);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, brokenLimbs) == 4880);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, deferred) == 4884);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, newAnims) == 4888);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, fixedlegs) == 4892);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, fixedtorso) == 4896);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, headOffset) == 4900);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, gender) == 4912);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, legsModel) == 4916);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, legsSkin) == 4920);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, torsoModel) == 4924);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, torsoSkin) == 4928);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, ghoul2Model) == 4936);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, modelIcon) == 4944);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, bolt_rhand) == 4948);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, bolt_lhand) == 4952);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, bolt_head) == 4956);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, bolt_motion) == 4960);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, bolt_llumbar) == 4964);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, siegeIndex) == 4968);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, siegeDesiredTeam) == 4972);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, sounds) == 4976);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, combatSounds) == 5136);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, extraSounds) == 5296);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, jediSounds) == 5456);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, siegeSounds) == 5616);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, duelSounds) == 5736);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, legsAnim) == 5896);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, torsoAnim) == 5900);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, facial_blink) == 5904);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, facial_frown) == 5908);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, facial_aux) == 5912);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientInfo_t, superSmoothTime) == 5916);
