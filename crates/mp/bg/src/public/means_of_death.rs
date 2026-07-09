//! MP `bg_public.h` means of death definitions.
//!
//! Type definition source: `oracle/codemp/game/bg_public.h:1046-1099`

#![allow(non_camel_case_types)]

/// Raven `meansOfDeath_t`.
///
/// Type definition source: `oracle/codemp/game/bg_public.h:1046-1099`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum meansOfDeath_t {
    MOD_UNKNOWN = 0,
    MOD_STUN_BATON = 1,
    MOD_MELEE = 2,
    MOD_SABER = 3,
    MOD_BRYAR_PISTOL = 4,
    MOD_BRYAR_PISTOL_ALT = 5,
    MOD_BLASTER = 6,
    MOD_TURBLAST = 7,
    MOD_DISRUPTOR = 8,
    MOD_DISRUPTOR_SPLASH = 9,
    MOD_DISRUPTOR_SNIPER = 10,
    MOD_BOWCASTER = 11,
    MOD_REPEATER = 12,
    MOD_REPEATER_ALT = 13,
    MOD_REPEATER_ALT_SPLASH = 14,
    MOD_DEMP2 = 15,
    MOD_DEMP2_ALT = 16,
    MOD_FLECHETTE = 17,
    MOD_FLECHETTE_ALT_SPLASH = 18,
    MOD_ROCKET = 19,
    MOD_ROCKET_SPLASH = 20,
    MOD_ROCKET_HOMING = 21,
    MOD_ROCKET_HOMING_SPLASH = 22,
    MOD_THERMAL = 23,
    MOD_THERMAL_SPLASH = 24,
    MOD_TRIP_MINE_SPLASH = 25,
    MOD_TIMED_MINE_SPLASH = 26,
    MOD_DET_PACK_SPLASH = 27,
    MOD_VEHICLE = 28,
    MOD_CONC = 29,
    MOD_CONC_ALT = 30,
    MOD_FORCE_DARK = 31,
    MOD_SENTRY = 32,
    MOD_WATER = 33,
    MOD_SLIME = 34,
    MOD_LAVA = 35,
    MOD_CRUSH = 36,
    MOD_TELEFRAG = 37,
    MOD_FALLING = 38,
    MOD_COLLISION = 39,
    MOD_VEH_EXPLOSION = 40,
    MOD_SUICIDE = 41,
    MOD_TARGET_LASER = 42,
    MOD_TRIGGER_HURT = 43,
    MOD_TEAM_CHANGE = 44,
    MOD_MAX = 45,
}
