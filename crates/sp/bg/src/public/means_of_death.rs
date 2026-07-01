//! SP `bg_public.h` means of death enumeration.
//!
//! Type definition source: `oracle/oracle/code/game/bg_public.h:560-617`

#![allow(non_camel_case_types)]

/// Raven `meansOfDeath_t`.
///
/// Type definition source: `oracle/oracle/code/game/bg_public.h:560-617`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum meansOfDeath_t {
    MOD_UNKNOWN = 0,

    // weapons
    MOD_SABER = 1,
    MOD_BRYAR = 2,
    MOD_BRYAR_ALT = 3,
    MOD_BLASTER = 4,
    MOD_BLASTER_ALT = 5,
    MOD_DISRUPTOR = 6,
    MOD_SNIPER = 7,
    MOD_BOWCASTER = 8,
    MOD_BOWCASTER_ALT = 9,
    MOD_REPEATER = 10,
    MOD_REPEATER_ALT = 11,
    MOD_DEMP2 = 12,
    MOD_DEMP2_ALT = 13,
    MOD_FLECHETTE = 14,
    MOD_FLECHETTE_ALT = 15,
    MOD_ROCKET = 16,
    MOD_ROCKET_ALT = 17,
    //NEW for JKA weapons:
    MOD_CONC = 18,
    MOD_CONC_ALT = 19,
    //END JKA weapons.
    MOD_THERMAL = 20,
    MOD_THERMAL_ALT = 21,
    MOD_DETPACK = 22,
    MOD_LASERTRIP = 23,
    MOD_LASERTRIP_ALT = 24,
    MOD_MELEE = 25,
    MOD_SEEKER = 26,
    MOD_FORCE_GRIP = 27,
    MOD_FORCE_LIGHTNING = 28,
    MOD_FORCE_DRAIN = 29,
    MOD_EMPLACED = 30,

    // world / generic
    MOD_ELECTROCUTE = 31,
    MOD_EXPLOSIVE = 32,
    MOD_EXPLOSIVE_SPLASH = 33,
    MOD_KNOCKOUT = 34,
    MOD_ENERGY = 35,
    MOD_ENERGY_SPLASH = 36,
    MOD_WATER = 37,
    MOD_SLIME = 38,
    MOD_LAVA = 39,
    MOD_CRUSH = 40,
    MOD_IMPACT = 41,
    MOD_FALLING = 42,
    MOD_SUICIDE = 43,
    MOD_TRIGGER_HURT = 44,
    MOD_GAS = 45,

    NUM_MODS = 46,
}
