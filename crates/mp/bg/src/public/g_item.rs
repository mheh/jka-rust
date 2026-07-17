//! Idiomatic Raven `gitem_t` — one master-item-table entry.

#![allow(non_snake_case)]

use core::ffi::c_int;

use super::item_kind::ItemKind;
use super::item_type::{
    itemType_t, IT_AMMO, IT_ARMOR, IT_BAD, IT_HEALTH, IT_HOLDABLE, IT_POWERUP, IT_TEAM, IT_WEAPON,
};

/// Raven `#define MAX_ITEM_MODELS 4`.
/// Source: `oracle/codemp/game/bg_public.h:1120`
pub const MAX_ITEM_MODELS: usize = 4;

/// Raven `gitem_t` — one master-item-table entry.
///
/// The eight `*mut c_char` fields become borrowed `'static` strings (Raven NULL
/// → `None`); the `giType`+`giTag` pair becomes [`ItemKind`]. Only the table
/// index (`s.modelindex`) crosses the engine seam, so this struct's layout is
/// free and the retail `#[repr(C)]`/`unsafe impl Sync`/layout asserts retire.
/// Type definition source: `oracle/codemp/game/bg_public.h:1122-1138`
pub struct GItem {
    /// Spawning name.
    pub classname: &'static str,
    pub pickup_sound: Option<&'static str>,
    /// Raven `world_model[MAX_ITEM_MODELS]` — the null-padded `[*;4]` becomes a
    /// slice of the real entries.
    pub world_model: &'static [&'static str],
    pub view_model: Option<&'static str>,
    pub icon: Option<&'static str>,
    /// For ammo how much, or duration of powerup.
    pub quantity: i32,
    /// Replaces the Raven `giType`+`giTag` pair.
    pub kind: ItemKind,
    /// String of all models and images this item will use.
    pub precaches: &'static str,
    /// String of all sounds this item will use.
    pub sounds: &'static str,
    pub description: Option<&'static str>,
}

impl GItem {
    /// Raven `giType` (`IT_*`), reconstructed from [`Self::kind`].
    #[inline]
    pub fn giType(&self) -> itemType_t {
        match self.kind {
            ItemKind::Bad => IT_BAD,
            ItemKind::Weapon(_) => IT_WEAPON,
            ItemKind::Ammo(_) => IT_AMMO,
            ItemKind::Armor { .. } => IT_ARMOR,
            ItemKind::Health => IT_HEALTH,
            ItemKind::Powerup(_) => IT_POWERUP,
            ItemKind::Holdable(_) => IT_HOLDABLE,
            ItemKind::Team(_) => IT_TEAM,
        }
    }

    /// Raven `giTag`, reconstructed from [`Self::kind`].
    #[inline]
    pub fn giTag(&self) -> c_int {
        match self.kind {
            ItemKind::Bad | ItemKind::Health => 0,
            ItemKind::Armor { rating } => rating,
            ItemKind::Holdable(t)
            | ItemKind::Powerup(t)
            | ItemKind::Weapon(t)
            | ItemKind::Team(t) => t,
            ItemKind::Ammo(t) => t,
        }
    }
}
