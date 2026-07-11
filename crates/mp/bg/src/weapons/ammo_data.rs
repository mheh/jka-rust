#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use super::ammo_data_t::ammoData_t;
use super::ammo_t::ammo_t;

/// Raven `ammoData[AMMO_MAX]` — per-ammo-type capacity table.
///
/// Source: `oracle/codemp/game/bg_weapons.c:358-400`
pub static ammoData: [ammoData_t; ammo_t::AMMO_MAX as usize] = [
    ammoData_t {
        // AMMO_NONE
        //		"",				//	char	icon[32];	// Name of ammo icon file
        max: 0,
    },
    ammoData_t {
        // AMMO_FORCE
        //		"",				//	char	icon[32];	// Name of ammo icon file
        max: 100,
    },
    ammoData_t {
        // AMMO_BLASTER
        //		"",				//	char	icon[32];	// Name of ammo icon file
        max: 300,
    },
    ammoData_t {
        // AMMO_POWERCELL
        //		"",				//	char	icon[32];	// Name of ammo icon file
        max: 300,
    },
    ammoData_t {
        // AMMO_METAL_BOLTS
        //		"",				//	char	icon[32];	// Name of ammo icon file
        max: 300,
    },
    ammoData_t {
        // AMMO_ROCKETS
        //		"",				//	char	icon[32];	// Name of ammo icon file
        max: 25,
    },
    ammoData_t {
        // AMMO_EMPLACED
        //		"",				//	char	icon[32];	// Name of ammo icon file
        max: 800,
    },
    ammoData_t {
        // AMMO_THERMAL
        //		"",				//	char	icon[32];	// Name of ammo icon file
        max: 10,
    },
    ammoData_t {
        // AMMO_TRIPMINE
        //		"",				//	char	icon[32];	// Name of ammo icon file
        max: 10,
    },
    ammoData_t {
        // AMMO_DETPACK
        //		"",				//	char	icon[32];	// Name of ammo icon file
        max: 10,
    },
];
