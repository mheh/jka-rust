#![allow(non_camel_case_types, non_snake_case)]

/// Raven `bSet_t` — AI behavior-set script enumeration.
///
/// This should check to matching a behavior state name first, then look for a script.
/// Type definition source: `oracle/code/game/bset.h:1-24`
#[repr(i32)]
pub enum bSet_t {
    /// Script to use when first spawned.
    BSET_SPAWN = 0,
    /// Script to use when used.
    BSET_USE = 1,
    /// Script to use when awoken/startled.
    BSET_AWAKE = 2,
    /// Script to use when acquire an enemy.
    BSET_ANGER = 3,
    /// Script to run when you attack.
    BSET_ATTACK = 4,
    /// Script to run when you kill someone.
    BSET_VICTORY = 5,
    /// Script to run when you can't find your enemy.
    BSET_LOSTENEMY = 6,
    /// Script to use when take pain.
    BSET_PAIN = 7,
    /// Script to use when take pain below 50% of health.
    BSET_FLEE = 8,
    /// Script to use when killed.
    BSET_DEATH = 9,
    /// Script to run when self->delayScriptTime is reached.
    BSET_DELAYED = 10,
    /// Script to run when blocked by a friendly NPC or player.
    BSET_BLOCKED = 11,
    /// Script to run when bumped into a friendly NPC or player (can set bumpRadius).
    BSET_BUMPED = 12,
    /// Script to run when blocked by a wall.
    BSET_STUCK = 13,
    /// Script to run when player shoots their own teammates.
    BSET_FFIRE = 14,
    /// Script to run when player kills a teammate.
    BSET_FFDEATH = 15,
    /// Script to run when player does a mind trick on this NPC.
    BSET_MINDTRICK = 16,
    /// Number of behavior sets.
    NUM_BSETS = 17,
    /// Invalid behavior set.
    BSET_INVALID = -1,
}

/// Alias for `BSET_SPAWN` (first element).
pub const BSET_FIRST: i32 = 0;
