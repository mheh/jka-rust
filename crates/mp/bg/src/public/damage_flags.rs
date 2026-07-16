//! bg-tier twin of the one `g_local.h` `DAMAGE_*` flag `bg_slidemove.c`
//! consumes (`PM_VehicleImpact` passes `DAMAGE_NO_ARMOR` to `G_Damage`).
//!
//! `g_local.h` is a game-only header, so the full `DAMAGE_*` set stays in the
//! game tier (`mp_game`'s `level::damage_flags`). Only `DAMAGE_NO_ARMOR` is read
//! from the bg tier, so it is duplicated here as a bit-identical twin per the
//! safe-state tier rule. Canonical game copy:
//! `crates/mp/game/src/level/damage_flags.rs` (value must stay in sync).
//!
//! Source: `oracle/codemp/game/g_local.h:1170-1190`

use core::ffi::c_int;

/// Raven `DAMAGE_NO_ARMOR` — armour does not protect from this damage.
/// bg twin of `mp_game`'s `damage_flags::DAMAGE_NO_ARMOR`.
/// Source: `oracle/codemp/game/g_local.h:1172`
pub const DAMAGE_NO_ARMOR: c_int = 0x00000002;
