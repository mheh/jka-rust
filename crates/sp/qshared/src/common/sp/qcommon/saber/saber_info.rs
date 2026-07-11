//! SP `saberInfo_t` and `MAX_SABERS`.
//!
//! Type definition source: `oracle/code/game/q_shared.h:1724-1944`

#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int};

use crate::shared::{qhandle_t, MAX_QPATH};

use super::blade_info::{bladeInfo_t, MAX_BLADES};
use super::saber_styles::saber_styles_t;
use super::saber_type::saberType_t;

/// Raven SP `saberInfo_t` (as loaded from `sabers.cfg`).
///
/// Raven: `!!!! loadsave affecting struct !!!!` — SP serializes this into
/// savegames (see also the retail-compat `saberInfoRetail_t`, not yet ported).
///
/// Diverges sharply from MP: `name`/`fullName`/`model`/`skin`/`brokenSaber1`/
/// `brokenSaber2` are heap `char *` (MP uses fixed buffers / a `qhandle_t` skin);
/// `g2MarksShader*` are `char[MAX_QPATH]` strings (MP: `int` handles); SP adds
/// `fallSound[3]` and keeps `brokenSaber1/2` live (dead code in MP). Pointer-
/// bearing => arch-dependent layout; asserts pin the host-64-bit size/offsets.
/// Type definition source: `oracle/code/game/q_shared.h:1724-1944`
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct saberInfo_t {
    pub name: *mut c_char,     // entry in sabers.cfg, if any
    pub fullName: *mut c_char, // the "Proper Name" of the saber, shown in the UI
    pub r#type: saberType_t,   // none, single or staff
    pub model: *mut c_char,    // hilt model
    pub skin: *mut c_char,     // hilt custom skin
    pub soundOn: c_int,
    pub soundLoop: c_int,
    pub soundOff: c_int,
    pub numBlades: c_int,
    pub blade: [bladeInfo_t; MAX_BLADES],
    pub stylesLearned: c_int,
    pub stylesForbidden: c_int,
    pub maxChain: c_int,
    pub forceRestrictions: c_int,
    pub lockBonus: c_int,
    pub parryBonus: c_int,
    pub breakParryBonus: c_int,
    pub breakParryBonus2: c_int,
    pub disarmBonus: c_int,
    pub disarmBonus2: c_int,
    pub singleBladeStyle: saber_styles_t,
    pub brokenSaber1: *mut c_char, // replacement saber for right hand when cut in half/broken
    pub brokenSaber2: *mut c_char, // replacement saber for left hand when cut in half/broken

    pub saberFlags: c_int,
    pub saberFlags2: c_int,
    pub spinSound: qhandle_t,
    pub swingSound: [qhandle_t; 3],
    pub fallSound: [qhandle_t; 3], // SP-only: sound when weapon drops to the ground
    pub moveSpeedScale: f32,
    pub animSpeedScale: f32,
    pub kataMove: c_int,
    pub lungeAtkMove: c_int,
    pub jumpAtkUpMove: c_int,
    pub jumpAtkFwdMove: c_int,
    pub jumpAtkBackMove: c_int,
    pub jumpAtkRightMove: c_int,
    pub jumpAtkLeftMove: c_int,
    pub readyAnim: c_int,
    pub drawAnim: c_int,
    pub putawayAnim: c_int,
    pub tauntAnim: c_int,
    pub bowAnim: c_int,
    pub meditateAnim: c_int,
    pub flourishAnim: c_int,
    pub gloatAnim: c_int,
    pub bladeStyle2Start: c_int,

    // ===PRIMARY BLADES=====================
    pub trailStyle: c_int,
    pub g2MarksShader: [c_char; MAX_QPATH], // SP: raw shader-name string, not a handle
    pub g2WeaponMarkShader: [c_char; MAX_QPATH],
    pub hitSound: [qhandle_t; 3],
    pub blockSound: [qhandle_t; 3],
    pub bounceSound: [qhandle_t; 3],
    pub blockEffect: c_int,
    pub hitPersonEffect: c_int,
    pub hitOtherEffect: c_int,
    pub bladeEffect: c_int,
    pub knockbackScale: f32,
    pub damageScale: f32,
    pub splashRadius: f32,
    pub splashDamage: c_int,
    pub splashKnockback: f32,

    // ===SECONDARY BLADES===================
    pub trailStyle2: c_int,
    pub g2MarksShader2: [c_char; MAX_QPATH],
    pub g2WeaponMarkShader2: [c_char; MAX_QPATH],
    pub hit2Sound: [qhandle_t; 3],
    pub block2Sound: [qhandle_t; 3],
    pub bounce2Sound: [qhandle_t; 3],
    pub blockEffect2: c_int,
    pub hitPersonEffect2: c_int,
    pub hitOtherEffect2: c_int,
    pub bladeEffect2: c_int,
    pub knockbackScale2: f32,
    pub damageScale2: f32,
    pub splashRadius2: f32,
    pub splashDamage2: c_int,
    pub splashKnockback2: f32,
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(saberInfo_t, model) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(saberInfo_t, blade) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(saberInfo_t, brokenSaber1) == 1416);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<saberInfo_t>() == 1952);

/// Raven SP `MAX_SABERS`.
///
/// Source: `oracle/code/game/q_shared.h:2064`
pub const MAX_SABERS: usize = 2;
