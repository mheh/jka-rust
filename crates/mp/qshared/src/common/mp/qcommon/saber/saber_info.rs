//! MP `saberInfo_t` and `MAX_SABERS`.
//!
//! Type definition source: `oracle/codemp/game/q_shared.h:735-841`

#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int};

use crate::shared::{qhandle_t, MAX_QPATH};

use super::blade_info::{bladeInfo_t, MAX_BLADES};
use super::saber_styles::saber_styles_t;
use super::saber_type::saberType_t;

/// Raven `saberInfo_t` — a full saber definition (as loaded from `sabers.cfg`).
///
/// Type definition source: `oracle/codemp/game/q_shared.h:735-840`
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct saberInfo_t {
    pub name: [c_char; 64],     // entry in sabers.cfg, if any
    pub fullName: [c_char; 64], // the "Proper Name" of the saber, shown in UI
    pub r#type: saberType_t,    // none, single or staff
    pub model: [c_char; MAX_QPATH], // hilt model
    pub skin: qhandle_t,        // registered skin id
    pub soundOn: c_int,         // game soundindex for turning on sound
    pub soundLoop: c_int,       // game soundindex for hum/loop sound
    pub soundOff: c_int,        // game soundindex for turning off sound
    pub numBlades: c_int,
    pub blade: [bladeInfo_t; MAX_BLADES], // blade info - like length, trail, origin, dir, etc.
    pub stylesLearned: c_int,   // styles you get when you get this saber, if any
    pub stylesForbidden: c_int, // styles you cannot use with this saber, if any
    pub maxChain: c_int,        // how many moves can be chained in a row with this weapon (-1 is infinite, 0 is use default behavior)
    pub forceRestrictions: c_int, // force powers that cannot be used while this saber is on (bitfield)
    pub lockBonus: c_int,       // in saberlocks, this type of saber pushes harder or weaker
    pub parryBonus: c_int,      // added to strength of parry with this saber
    pub breakParryBonus: c_int, // added to strength when hit a parry
    pub breakParryBonus2: c_int, // for bladeStyle2 (see bladeStyle2Start below)
    pub disarmBonus: c_int,     // added to disarm chance when win saberlock or have a good parry (knockaway)
    pub disarmBonus2: c_int,    // for bladeStyle2 (see bladeStyle2Start below)
    pub singleBladeStyle: saber_styles_t, // makes it so that you use a different style if you only have the first blade active

    // these values are global to the saber, like all of the ones above
    pub saberFlags: c_int,  // from SFL_ list above
    pub saberFlags2: c_int, // from SFL2_ list above

    // done in cgame (client-side code)
    pub spinSound: qhandle_t,      // none - if set, plays this sound as it spins when thrown
    pub swingSound: [qhandle_t; 3], // none - if set, plays one of these 3 sounds when swung during an attack

    // done in game (server-side code)
    pub moveSpeedScale: f32, // 1.0 - you move faster/slower when using this saber
    pub animSpeedScale: f32, // 1.0 - plays normal attack animations faster/slower

    // done in both cgame and game (BG code)
    pub kataMove: c_int,         // LS_INVALID - executed when both attack buttons pressed together
    pub lungeAtkMove: c_int,     // LS_INVALID - executed on crouch+fwd+attack
    pub jumpAtkUpMove: c_int,    // LS_INVALID - executed on jump+attack
    pub jumpAtkFwdMove: c_int,   // LS_INVALID - executed on jump+fwd+attack
    pub jumpAtkBackMove: c_int,  // LS_INVALID - executed on jump+back+attack
    pub jumpAtkRightMove: c_int, // LS_INVALID - executed on jump+right+attack
    pub jumpAtkLeftMove: c_int,  // LS_INVALID - executed on jump+left+attack
    pub readyAnim: c_int,        // -1 - anim to use when standing idle
    pub drawAnim: c_int,         // -1 - anim to use when drawing weapon
    pub putawayAnim: c_int,      // -1 - anim to use when putting weapon away
    pub tauntAnim: c_int,        // -1 - anim to use when hit "taunt"
    pub bowAnim: c_int,          // -1 - anim to use when hit "bow"
    pub meditateAnim: c_int,     // -1 - anim to use when hit "meditate"
    pub flourishAnim: c_int,     // -1 - anim to use when hit "flourish"
    pub gloatAnim: c_int,        // -1 - anim to use when hit "gloat"

    pub bladeStyle2Start: c_int, // 0 - if set, blades from this number and higher use the secondary values

    // ===PRIMARY BLADES=====================
    // done in cgame (client-side code)
    pub trailStyle: c_int,         // 0 - normal, 1 motion blur, 2 no trail
    pub g2MarksShader: c_int,      // none - shader for marks on enemies
    pub g2WeaponMarkShader: c_int, // none - shader projected onto the weapon on damage
    pub hitSound: [qhandle_t; 3],   // none - 3 sounds when saber hits a person
    pub blockSound: [qhandle_t; 3], // none - 3 sounds when saber/sword hits another saber/sword
    pub bounceSound: [qhandle_t; 3], // none - 3 sounds when saber/sword hits a wall and bounces
    pub blockEffect: c_int,      // none - effect when saber/sword hits another saber/sword
    pub hitPersonEffect: c_int,  // none - effect when saber/sword hits a person
    pub hitOtherEffect: c_int,   // none - effect when saber/sword hits something else damagable
    pub bladeEffect: c_int,      // none - effect at the blade tag

    // done in game (server-side code)
    pub knockbackScale: f32, // 0 - if non-zero, uses damage done to calculate knockback
    pub damageScale: f32,    // 1 - scale up or down the damage done by the saber
    pub splashRadius: f32,   // 0 - radius of splashDamage
    pub splashDamage: c_int, // 0 - amount of splashDamage
    pub splashKnockback: f32, // 0 - amount of splashKnockback

    // ===SECONDARY BLADES===================
    // done in cgame (client-side code)
    pub trailStyle2: c_int,
    pub g2MarksShader2: c_int,
    pub g2WeaponMarkShader2: c_int,
    pub hit2Sound: [qhandle_t; 3],
    pub block2Sound: [qhandle_t; 3],
    pub bounce2Sound: [qhandle_t; 3],
    pub blockEffect2: c_int,
    pub hitPersonEffect2: c_int,
    pub hitOtherEffect2: c_int,
    pub bladeEffect2: c_int,

    // done in game (server-side code)
    pub knockbackScale2: f32,
    pub damageScale2: f32,
    pub splashRadius2: f32,
    pub splashDamage2: c_int,
    pub splashKnockback2: f32,
}
const _: () = assert!(core::mem::size_of::<saberInfo_t>() == 2156);

/// Raven `MAX_SABERS`.
///
/// Source: `oracle/codemp/game/q_shared.h:841`
pub const MAX_SABERS: usize = 2;
