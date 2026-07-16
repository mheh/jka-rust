//! bg-tier twins of the `w_saber.h` saber geometry / event constants that
//! `bg_saber.c`/`bg_saberLoad.c` consume.
//!
//! `w_saber.h` is a game-only (`QAGAME`) header, so its canonical Rust home is
//! the game tier (`mp_game`'s `saber::w_saber_consts`). These few constants are
//! also read from the bg tier (saber-throw bbox/geometry, the saber-lock-won
//! event bit, blade radius), which cannot reach the game crate; per the
//! safe-state tier rule (`docs/plans/2026-07-12-safe-state-migration.md`) they
//! are duplicated here as bit-identical twins. Canonical game copy:
//! `crates/mp/game/src/saber/w_saber_consts.rs` (values must stay in sync).
//!
//! Source: `oracle/codemp/game/w_saber.h`

use core::ffi::c_int;

/// Raven `SEF_LOCK_WON` — won a saberLock (`saberEventFlags` bit).
/// bg twin of `mp_game`'s `w_saber_consts::SEF_LOCK_WON`.
/// Source: `oracle/codemp/game/w_saber.h`
pub const SEF_LOCK_WON: c_int = 0x100;

/// Raven `SABER_RADIUS_STANDARD` — default blade collision radius.
/// bg twin of `mp_game`'s `w_saber_consts::SABER_RADIUS_STANDARD`.
/// Source: `oracle/codemp/game/w_saber.h`
pub const SABER_RADIUS_STANDARD: f32 = 3.0;

/// Raven `SABERMINS_X/Y/Z` — thrown-saber bbox mins.
/// bg twin of `mp_game`'s `w_saber_consts::SABERMINS_*`.
/// Source: `oracle/codemp/game/w_saber.h`
pub const SABERMINS_X: f32 = -3.0;
pub const SABERMINS_Y: f32 = -3.0;
pub const SABERMINS_Z: f32 = -3.0;

/// Raven `SABERMAXS_X/Y/Z` — thrown-saber bbox maxs.
/// bg twin of `mp_game`'s `w_saber_consts::SABERMAXS_*`.
/// Source: `oracle/codemp/game/w_saber.h`
pub const SABERMAXS_X: f32 = 3.0;
pub const SABERMAXS_Y: f32 = 3.0;
pub const SABERMAXS_Z: f32 = 3.0;

/// Raven `SABER_MIN_THROW_DIST` — minimum saber-throw distance.
/// bg twin of `mp_game`'s `w_saber_consts::SABER_MIN_THROW_DIST`.
/// Source: `oracle/codemp/game/w_saber.h`
pub const SABER_MIN_THROW_DIST: f32 = 80.0;
