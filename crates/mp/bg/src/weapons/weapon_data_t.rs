#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

/// Raven `weaponData_t` — per-weapon-type data.
///
/// Type definition source: `oracle/oracle/codemp/game/bg_weapons.h:61-84`
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct weaponData_t {
	//	char	classname[32];		// Spawning name
	pub ammoIndex: i32,      // Index to proper ammo slot
	pub ammoLow: i32,        // Count when ammo is low

	pub energyPerShot: i32,  // Amount of energy used per shot
	pub fireTime: i32,       // Amount of time between firings
	pub range: i32,          // Range of weapon

	pub altEnergyPerShot: i32, // Amount of energy used for alt-fire
	pub altFireTime: i32,      // Amount of time between alt-firings
	pub altRange: i32,         // Range of alt-fire

	pub chargeSubTime: i32,    // ms interval for subtracting ammo during charge
	pub altChargeSubTime: i32, // above for secondary

	pub chargeSub: i32,        // amount to subtract during charge on each interval
	pub altChargeSub: i32,     // above for secondary

	pub maxCharge: i32,        // stop subtracting once charged for this many ms
	pub altMaxCharge: i32,     // above for secondary
}

const _: () = assert!(core::mem::size_of::<weaponData_t>() == 56);
const _: () = assert!(core::mem::offset_of!(weaponData_t, ammoIndex) == 0);
const _: () = assert!(core::mem::offset_of!(weaponData_t, ammoLow) == 4);
const _: () = assert!(core::mem::offset_of!(weaponData_t, energyPerShot) == 8);
const _: () = assert!(core::mem::offset_of!(weaponData_t, fireTime) == 12);
const _: () = assert!(core::mem::offset_of!(weaponData_t, range) == 16);
const _: () = assert!(core::mem::offset_of!(weaponData_t, altEnergyPerShot) == 20);
const _: () = assert!(core::mem::offset_of!(weaponData_t, altFireTime) == 24);
const _: () = assert!(core::mem::offset_of!(weaponData_t, altRange) == 28);
const _: () = assert!(core::mem::offset_of!(weaponData_t, chargeSubTime) == 32);
const _: () = assert!(core::mem::offset_of!(weaponData_t, altChargeSubTime) == 36);
const _: () = assert!(core::mem::offset_of!(weaponData_t, chargeSub) == 40);
const _: () = assert!(core::mem::offset_of!(weaponData_t, altChargeSub) == 44);
const _: () = assert!(core::mem::offset_of!(weaponData_t, maxCharge) == 48);
const _: () = assert!(core::mem::offset_of!(weaponData_t, altMaxCharge) == 52);
