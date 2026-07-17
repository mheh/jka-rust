//! Idiomatic replacement for Raven `gitem_t`'s `giType`+`giTag` pair.

#![allow(non_camel_case_types)]

use core::ffi::c_int;

use super::holdable::holdable_t;
use super::item_type::{
    itemType_t, IT_AMMO, IT_ARMOR, IT_BAD, IT_HEALTH, IT_HOLDABLE, IT_POWERUP, IT_TEAM, IT_WEAPON,
};
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

impl ItemKind {
    /// Exact inverse of the `GItem::giType()`/`giTag()` pair — builds the kind
    /// a Raven `(giType, giTag)` int pair denotes, for callers that still carry
    /// the two ints (e.g. `BG_GetItemIndexByTag`): comparing against `from_gi`
    /// equals Raven's paired `giType == type && giTag == tag`. `None` for pairs
    /// that don't round-trip (`IT_BATTERY`/`IT_HOLOCRON`, or a payload-less
    /// type with a nonzero tag).
    pub fn from_gi(giType: itemType_t, giTag: c_int) -> Option<ItemKind> {
        Some(match giType {
            IT_BAD if giTag == 0 => ItemKind::Bad,
            IT_ARMOR => ItemKind::Armor { rating: giTag },
            IT_HEALTH if giTag == 0 => ItemKind::Health,
            IT_HOLDABLE => ItemKind::Holdable(giTag),
            IT_POWERUP => ItemKind::Powerup(giTag),
            IT_WEAPON => ItemKind::Weapon(giTag),
            IT_AMMO => ItemKind::Ammo(giTag),
            IT_TEAM => ItemKind::Team(giTag),
            _ => return None,
        })
    }
}
