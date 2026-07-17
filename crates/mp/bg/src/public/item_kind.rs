//! Idiomatic replacement for Raven `gitem_t`'s `giType`+`giTag` pair.

#![allow(non_camel_case_types)]

use core::ffi::c_int;

use super::holdable::holdable_t;
use super::powerup::powerup_t;
use crate::weapons::weapon_t::weapon_t;

/// Raven `gitem_t`'s `giType`+`giTag` pair, as a real tagged union.
///
/// Replaces the manual `itemType_t` discriminant + bare `int giTag` payload.
/// `Weapon`/`Holdable`/`Powerup`/`Team` keep the existing `c_int` aliases;
/// `Ammo` carries a raw `c_int` (not the real `ammo_t` enum) because `ammo_all`
/// stores giTag `-1`, which is no `ammo_t` variant — real-enum payload upgrades
/// are a later slice.
/// Type definition source: `oracle/codemp/game/bg_public.h:1122-1138`
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ItemKind {
    /// Raven `IT_BAD` — the index-0 sentinel only.
    Bad,
    /// giTag 1|2 (small/large shield).
    Armor {
        rating: i32,
    },
    Health,
    Holdable(holdable_t),
    Powerup(powerup_t),
    Weapon(weapon_t),
    /// `ammo_all` carries Raven's giTag `-1` (give-all dispenser refill).
    Ammo(c_int),
    /// CTF flags (`PW_*FLAG`); the red/blue cubes carry 0.
    Team(powerup_t),
}
