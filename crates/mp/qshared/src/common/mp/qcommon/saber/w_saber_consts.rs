//! The `w_saber.h` saber geometry / event constants that both the game tier
//! (`mp_game`'s `saber::w_saber_consts`) and the bg tier (`bg_saber.c`/
//! `bg_saberLoad.c`) consume. `w_saber.h` is a game-only (`QAGAME`) header, but
//! this handful of geometry/event constants is read from both tiers, which
//! cannot reach each other; they live here in the shared qshared tier so both
//! import the one definition.
//!
//! Source: `oracle/codemp/game/w_saber.h`

use core::ffi::c_int;

/// Raven `SEF_LOCK_WON` — won a saberLock (`saberEventFlags` bit).
/// Source: `oracle/codemp/game/w_saber.h`
pub const SEF_LOCK_WON: c_int = 0x100;

/// Raven `SABER_RADIUS_STANDARD` — default blade collision radius.
/// Source: `oracle/codemp/game/w_saber.h`
pub const SABER_RADIUS_STANDARD: f32 = 3.0;

/// Raven `SABERMINS_X/Y/Z` — thrown-saber bbox mins.
/// Source: `oracle/codemp/game/w_saber.h`
pub const SABERMINS_X: f32 = -3.0;
pub const SABERMINS_Y: f32 = -3.0;
pub const SABERMINS_Z: f32 = -3.0;

/// Raven `SABERMAXS_X/Y/Z` — thrown-saber bbox maxs.
/// Source: `oracle/codemp/game/w_saber.h`
pub const SABERMAXS_X: f32 = 3.0;
pub const SABERMAXS_Y: f32 = 3.0;
pub const SABERMAXS_Z: f32 = 3.0;

/// Raven `SABER_MIN_THROW_DIST` — minimum saber-throw distance.
/// Source: `oracle/codemp/game/w_saber.h`
pub const SABER_MIN_THROW_DIST: f32 = 80.0;
