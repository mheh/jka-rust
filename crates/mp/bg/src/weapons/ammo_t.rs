#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

/// Raven `ammo_t` — ammunition type enumeration.
///
/// Type definition source: `oracle/oracle/codemp/game/bg_weapons.h:45-58`
#[allow(non_camel_case_types)]
#[repr(i32)]
pub enum ammo_t {
	AMMO_NONE = 0,
	AMMO_FORCE = 1,		// AMMO_PHASER
	AMMO_BLASTER = 2,	// AMMO_STARFLEET
	AMMO_POWERCELL = 3,	// AMMO_ALIEN
	AMMO_METAL_BOLTS = 4,
	AMMO_ROCKETS = 5,
	AMMO_EMPLACED = 6,
	AMMO_THERMAL = 7,
	AMMO_TRIPMINE = 8,
	AMMO_DETPACK = 9,
	AMMO_MAX = 10,
}
