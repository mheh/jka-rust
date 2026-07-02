#![allow(non_camel_case_types, non_snake_case)]

/// Raven `weapon_t` — weapon type enumeration.
///
/// Type definition source: `oracle/oracle/code/game/weapons.h:9-59`
#[repr(i32)]
pub enum weapon_t {
    WP_NONE,

    // Player weapons
    WP_SABER,           // player and NPC weapon
    WP_BLASTER_PISTOL,  // player and NPC weapon
    WP_BLASTER,         // player and NPC weapon
    WP_DISRUPTOR,       // player and NPC weapon
    WP_BOWCASTER,       // NPC weapon - player can pick this up, but never starts with them
    WP_REPEATER,        // NPC weapon - player can pick this up, but never starts with them
    WP_DEMP2,           // NPC weapon - player can pick this up, but never starts with them
    WP_FLECHETTE,       // NPC weapon - player can pick this up, but never starts with them
    WP_ROCKET_LAUNCHER, // NPC weapon - player can pick this up, but never starts with them
    WP_THERMAL,         // player and NPC weapon
    WP_TRIP_MINE,       // NPC weapon - player can pick this up, but never starts with them
    WP_DET_PACK,        // NPC weapon - player can pick this up, but never starts with them
    WP_CONCUSSION,      // NPC weapon - player can pick this up, but never starts with them

    // extras
    WP_MELEE, // player and NPC weapon - Any ol' melee attack

    // when in atst
    WP_ATST_MAIN,
    WP_ATST_SIDE,

    // These can never be gotten directly by the player
    WP_STUN_BATON, // stupid weapon, should remove

    // NPC weapons
    WP_BRYAR_PISTOL, // NPC weapon - player can pick this up, but never starts with them

    WP_EMPLACED_GUN,

    WP_BOT_LASER, // Probe droid - Laser blast

    WP_TURRET, // turret guns

    WP_TIE_FIGHTER,

    WP_RAPID_FIRE_CONC,

    WP_JAWA,
    WP_TUSKEN_RIFLE,
    WP_TUSKEN_STAFF,
    WP_SCEPTER,
    WP_NOGHRI_STICK,

    WP_NUM_WEAPONS,
}
