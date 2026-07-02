#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_void};

use sp_qshared::shared::vec3_t;

/// Raven `weaponData_t` — per-weapon-type data.
///
/// Type definition source: `oracle/oracle/code/game/weapons.h:81-139`
#[repr(C)]
pub struct weaponData_t {
	pub classname: [c_char; 32], // Spawning name
	pub weaponMdl: [c_char; 64], // Weapon Model
	pub firingSnd: [c_char; 64], // Sound made when fired
	pub altFiringSnd: [c_char; 64], // Sound made when alt-fired
	//	char	flashSnd[64];		// Sound made by flash
	//	char	altFlashSnd[64];	// Sound made by an alt-flash
	pub stopSnd: [c_char; 64], // Sound made when weapon stops firing
	pub chargeSnd: [c_char; 64], // sound to start when the weapon initiates the charging sequence
	pub altChargeSnd: [c_char; 64], // alt sound to start when the weapon initiates the charging sequence
	pub selectSnd: [c_char; 64], // the sound to play when this weapon gets selected

	// #ifdef _IMMERSION
	pub firingFrc: [c_char; 64],
	pub altFiringFrc: [c_char; 64],
	pub stopFrc: [c_char; 64],
	pub chargeFrc: [c_char; 64],
	pub altChargeFrc: [c_char; 64],
	pub selectFrc: [c_char; 64],
	// #endif // _IMMERSION
	pub ammoIndex: i32, // Index to proper ammo slot
	pub ammoLow: i32,   // Count when ammo is low

	pub energyPerShot: i32, // Amount of energy used per shot
	pub fireTime: i32,      // Amount of time between firings
	pub range: i32,         // Range of weapon

	pub altEnergyPerShot: i32, // Amount of energy used for alt-fire
	pub altFireTime: i32,      // Amount of time between alt-firings
	pub altRange: i32,         // Range of alt-fire

	pub weaponIcon: [c_char; 64], // Name of weapon icon file
	pub numBarrels: i32,          // how many barrels should we expect for this weapon?

	pub missileMdl: [c_char; 64],       // Missile Model
	pub missileSound: [c_char; 64],     // Missile flight sound
	pub missileDlight: f32,             // what is says
	pub missileDlightColor: vec3_t,     // ditto

	pub alt_missileMdl: [c_char; 64],   // Missile Model
	pub alt_missileSound: [c_char; 64], // Missile sound
	pub alt_missileDlight: f32,         // what is says
	pub alt_missileDlightColor: vec3_t, // ditto

	pub missileHitSound: [c_char; 64],    // Missile impact sound
	pub altmissileHitSound: [c_char; 64], // alt Missile impact sound

	// #ifndef _USRDLL
	pub func: *mut c_void,
	pub altfunc: *mut c_void,

	pub mMuzzleEffect: [c_char; 64],
	pub mMuzzleEffectID: i32,
	pub mAltMuzzleEffect: [c_char; 64],
	pub mAltMuzzleEffectID: i32,
	// #endif
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<weaponData_t>() == 1536);
const _: () = assert!(core::mem::offset_of!(weaponData_t, classname) == 0);
const _: () = assert!(core::mem::offset_of!(weaponData_t, weaponMdl) == 32);
const _: () = assert!(core::mem::offset_of!(weaponData_t, firingSnd) == 96);
const _: () = assert!(core::mem::offset_of!(weaponData_t, altFiringSnd) == 160);
const _: () = assert!(core::mem::offset_of!(weaponData_t, stopSnd) == 224);
const _: () = assert!(core::mem::offset_of!(weaponData_t, chargeSnd) == 288);
const _: () = assert!(core::mem::offset_of!(weaponData_t, altChargeSnd) == 352);
const _: () = assert!(core::mem::offset_of!(weaponData_t, selectSnd) == 416);
const _: () = assert!(core::mem::offset_of!(weaponData_t, firingFrc) == 480);
const _: () = assert!(core::mem::offset_of!(weaponData_t, altFiringFrc) == 544);
const _: () = assert!(core::mem::offset_of!(weaponData_t, stopFrc) == 608);
const _: () = assert!(core::mem::offset_of!(weaponData_t, chargeFrc) == 672);
const _: () = assert!(core::mem::offset_of!(weaponData_t, altChargeFrc) == 736);
const _: () = assert!(core::mem::offset_of!(weaponData_t, selectFrc) == 800);
const _: () = assert!(core::mem::offset_of!(weaponData_t, ammoIndex) == 864);
const _: () = assert!(core::mem::offset_of!(weaponData_t, ammoLow) == 868);
const _: () = assert!(core::mem::offset_of!(weaponData_t, energyPerShot) == 872);
const _: () = assert!(core::mem::offset_of!(weaponData_t, fireTime) == 876);
const _: () = assert!(core::mem::offset_of!(weaponData_t, range) == 880);
const _: () = assert!(core::mem::offset_of!(weaponData_t, altEnergyPerShot) == 884);
const _: () = assert!(core::mem::offset_of!(weaponData_t, altFireTime) == 888);
const _: () = assert!(core::mem::offset_of!(weaponData_t, altRange) == 892);
const _: () = assert!(core::mem::offset_of!(weaponData_t, weaponIcon) == 896);
const _: () = assert!(core::mem::offset_of!(weaponData_t, numBarrels) == 960);
const _: () = assert!(core::mem::offset_of!(weaponData_t, missileMdl) == 964);
const _: () = assert!(core::mem::offset_of!(weaponData_t, missileSound) == 1028);
const _: () = assert!(core::mem::offset_of!(weaponData_t, missileDlight) == 1092);
const _: () = assert!(core::mem::offset_of!(weaponData_t, missileDlightColor) == 1096);
const _: () = assert!(core::mem::offset_of!(weaponData_t, alt_missileMdl) == 1108);
const _: () = assert!(core::mem::offset_of!(weaponData_t, alt_missileSound) == 1172);
const _: () = assert!(core::mem::offset_of!(weaponData_t, alt_missileDlight) == 1236);
const _: () = assert!(core::mem::offset_of!(weaponData_t, alt_missileDlightColor) == 1240);
const _: () = assert!(core::mem::offset_of!(weaponData_t, missileHitSound) == 1252);
const _: () = assert!(core::mem::offset_of!(weaponData_t, altmissileHitSound) == 1316);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponData_t, func) == 1384);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponData_t, altfunc) == 1392);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponData_t, mMuzzleEffect) == 1400);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponData_t, mMuzzleEffectID) == 1464);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponData_t, mAltMuzzleEffect) == 1468);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponData_t, mAltMuzzleEffectID) == 1532);
