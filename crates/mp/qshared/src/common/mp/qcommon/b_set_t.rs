#![allow(non_camel_case_types, non_snake_case)]

/// Raven `bSet_t` — behavior set type for NPC script selection.
///
/// Raven: This should check to matching a behavior state name first, then look for a script.
/// Type definition source: `oracle/codemp/game/g_public.h:641-664`
#[repr(i32)]
pub enum bSet_t {
    BSET_INVALID = -1,
    // BSET_FIRST = 0,   // alias for BSET_SPAWN (not duplicated in Rust enum)
    BSET_SPAWN = 0, // script to use when first spawned
    BSET_USE,       // script to use when used
    BSET_AWAKE,     // script to use when awoken/startled
    BSET_ANGER,     // script to use when aquire an enemy
    BSET_ATTACK,    // script to run when you attack
    BSET_VICTORY,   // script to run when you kill someone
    BSET_LOSTENEMY, // script to run when you can't find your enemy
    BSET_PAIN,      // script to use when take pain
    BSET_FLEE,      // script to use when take pain below 50% of health
    BSET_DEATH,     // script to use when killed
    BSET_DELAYED,   // script to run when self->delayScriptTime is reached
    BSET_BLOCKED,   // script to run when blocked by a friendly NPC or player
    BSET_BUMPED,    // script to run when bumped into a friendly NPC or player (can set bumpRadius)
    BSET_STUCK,     // script to run when blocked by a wall
    BSET_FFIRE,     // script to run when player shoots their own teammates
    BSET_FFDEATH,   // script to run when player kills a teammate
    BSET_MINDTRICK, // script to run when player does a mind trick on this NPC

    NUM_BSETS,
}
