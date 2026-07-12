//! MP `bg_public.h` player stat index enumeration.
//!
//! Type definition source: `oracle/codemp/game/bg_public.h:520-532`

#![allow(non_camel_case_types)]

/// Raven `statIndex_t` — player statistics array indices.
///
/// Raven: Enumeration defining indices into the player statistics array
/// for health, armor, weapons, and other persistent player state values.
/// Type definition source: `oracle/codemp/game/bg_public.h:520-532`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum statIndex_t {
    STAT_HEALTH = 0,
    STAT_HOLDABLE_ITEM = 1,
    STAT_HOLDABLE_ITEMS = 2,
    STAT_PERSISTANT_POWERUP = 3,
    STAT_WEAPONS = 4,
    STAT_ARMOR = 5,
    STAT_DEAD_YAW = 6,
    STAT_CLIENTS_READY = 7,
    STAT_MAX_HEALTH = 8,
}
