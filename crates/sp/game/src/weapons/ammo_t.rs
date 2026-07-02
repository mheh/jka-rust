#![allow(non_camel_case_types, non_snake_case)]

/// Raven `ammo_t` — ammunition type enumeration.
///
/// Type definition source: `oracle/oracle/code/game/weapons.h:65-78`
#[repr(i32)]
pub enum ammo_t {
    AMMO_NONE,
    AMMO_FORCE,        // AMMO_PHASER
    AMMO_BLASTER,      // AMMO_STARFLEET
    AMMO_POWERCELL,    // AMMO_ALIEN
    AMMO_METAL_BOLTS,
    AMMO_ROCKETS,
    AMMO_EMPLACED,
    AMMO_THERMAL,
    AMMO_TRIPMINE,
    AMMO_DETPACK,
    AMMO_MAX,
}
