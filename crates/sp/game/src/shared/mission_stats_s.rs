#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::force_powers::NUM_FORCE_POWERS;

use crate::weapons::weapon_t::weapon_t;

/// Raven `missionStats_t` — per-mission player statistics.
///
/// Type definition source: `oracle/code/game/g_shared.h:302-318`
#[repr(C)]
pub struct missionStats_t {
    pub secretsFound: i32,                           // # of secret areas found
    pub totalSecrets: i32,                           // # of secret areas that could have been found
    pub shotsFired: i32,                             // total number of shots fired
    pub hits: i32,                                   // Shots that did damage
    pub enemiesSpawned: i32,                         // # of enemies spawned
    pub enemiesKilled: i32,                          // # of enemies killed
    pub saberThrownCnt: i32,                         // # of times saber was thrown
    pub saberBlocksCnt: i32,                         // # of times saber was used to block
    pub legAttacksCnt: i32,                          // # of times legs were hit with saber
    pub armAttacksCnt: i32,                          // # of times arm were hit with saber
    pub torsoAttacksCnt: i32,                        // # of times torso was hit with saber
    pub otherAttacksCnt: i32, // # of times anything else on a monster was hit with saber
    pub forceUsed: [i32; NUM_FORCE_POWERS as usize], // # of times each force power was used
    pub weaponUsed: [i32; weapon_t::WP_NUM_WEAPONS as usize], // # of times each weapon was used
}

const _: () = assert!(core::mem::size_of::<missionStats_t>() == 228);
const _: () = assert!(core::mem::offset_of!(missionStats_t, secretsFound) == 0);
const _: () = assert!(core::mem::offset_of!(missionStats_t, totalSecrets) == 4);
const _: () = assert!(core::mem::offset_of!(missionStats_t, shotsFired) == 8);
const _: () = assert!(core::mem::offset_of!(missionStats_t, hits) == 12);
const _: () = assert!(core::mem::offset_of!(missionStats_t, enemiesSpawned) == 16);
const _: () = assert!(core::mem::offset_of!(missionStats_t, enemiesKilled) == 20);
const _: () = assert!(core::mem::offset_of!(missionStats_t, saberThrownCnt) == 24);
const _: () = assert!(core::mem::offset_of!(missionStats_t, saberBlocksCnt) == 28);
const _: () = assert!(core::mem::offset_of!(missionStats_t, legAttacksCnt) == 32);
const _: () = assert!(core::mem::offset_of!(missionStats_t, armAttacksCnt) == 36);
const _: () = assert!(core::mem::offset_of!(missionStats_t, torsoAttacksCnt) == 40);
const _: () = assert!(core::mem::offset_of!(missionStats_t, otherAttacksCnt) == 44);
const _: () = assert!(core::mem::offset_of!(missionStats_t, forceUsed) == 48);
const _: () = assert!(core::mem::offset_of!(missionStats_t, weaponUsed) == 112);
