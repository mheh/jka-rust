//! MP `bg_public.h` item-type definitions.
//!
//! Type definition source: `oracle/codemp/game/bg_public.h:1103-1116`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

/// Raven `gitem_t::type` (`itemType_t`).
///
/// Raven declares this as `typedef int itemType_t` alongside a separate
/// anonymous enum, so the alias stays an int and the enumerators are
/// `const`s (enum-vs-alias fidelity rule).
///
/// Type definition source: `oracle/codemp/game/bg_public.h:1103-1116`
pub type itemType_t = c_int;

pub const IT_BAD: itemType_t = 0;
pub const IT_WEAPON: itemType_t = 1; // EFX: rotate + upscale + minlight
pub const IT_AMMO: itemType_t = 2; // EFX: rotate
pub const IT_ARMOR: itemType_t = 3; // EFX: rotate + minlight
pub const IT_HEALTH: itemType_t = 4; // EFX: static external sphere + rotating internal
pub const IT_POWERUP: itemType_t = 5; // instant on, timer based
pub const IT_HOLDABLE: itemType_t = 6; // single use, holdable item
pub const IT_PERSISTANT_POWERUP: itemType_t = 7;
pub const IT_TEAM: itemType_t = 8;
