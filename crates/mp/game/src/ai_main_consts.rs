//! MP `ai_main.h`/`ai_main.c` plain `#define` constants: `LEVELFLAG_*` bit
//! flags (`level.levelFlags` / `gLevelFlags` AI hint bits) and bot AI
//! range/distance/interval tunables.
//!
//! Plain `#define`s (not an enum), so §C8 makes them `const`s directly.
//!
//! Source: `oracle/oracle/codemp/game/ai_main.h:40-79`,
//! `oracle/oracle/codemp/game/ai_main.c:43`

use core::ffi::c_int;

/// Raven `LEVELFLAG_NOPOINTPREDICTION` — don't take waypoint beyond current
/// into account when adjusting path view angles.
///
/// Source: `oracle/oracle/codemp/game/ai_main.h:40`
pub const LEVELFLAG_NOPOINTPREDICTION: c_int = 1;

/// Raven `LEVELFLAG_IGNOREINFALLBACK` — ignore enemies when in a fallback
/// navigation routine.
///
/// Source: `oracle/oracle/codemp/game/ai_main.h:41`
pub const LEVELFLAG_IGNOREINFALLBACK: c_int = 2;

/// Raven `LEVELFLAG_IMUSTNTRUNAWAY` — don't be scared.
///
/// Source: `oracle/oracle/codemp/game/ai_main.h:42`
pub const LEVELFLAG_IMUSTNTRUNAWAY: c_int = 4;

/// Raven `MELEE_ATTACK_RANGE`.
///
/// Source: `oracle/oracle/codemp/game/ai_main.h:51`
pub const MELEE_ATTACK_RANGE: c_int = 256;

/// Raven `SABER_ATTACK_RANGE`.
///
/// Source: `oracle/oracle/codemp/game/ai_main.h:52`
pub const SABER_ATTACK_RANGE: c_int = 128;

/// Raven `BOT_WPTOUCH_DISTANCE`.
///
/// Source: `oracle/oracle/codemp/game/ai_main.h:56`
pub const BOT_WPTOUCH_DISTANCE: c_int = 32;

/// Raven `BOT_PLANT_DISTANCE` — plant if within this radius from the last
/// spotted enemy position.
///
/// Source: `oracle/oracle/codemp/game/ai_main.h:61`
pub const BOT_PLANT_DISTANCE: c_int = 256;

/// Raven `BOT_PLANT_INTERVAL` — only plant once per 15 seconds at max.
///
/// Source: `oracle/oracle/codemp/game/ai_main.h:62`
pub const BOT_PLANT_INTERVAL: c_int = 15000;

/// Raven `BOT_PLANT_BLOW_DISTANCE` — blow det packs if enemy is within this
/// radius and I am further away than the enemy.
///
/// Source: `oracle/oracle/codemp/game/ai_main.h:63`
pub const BOT_PLANT_BLOW_DISTANCE: c_int = 256;

/// Raven `BOT_FLAG_GET_DISTANCE`.
///
/// Source: `oracle/oracle/codemp/game/ai_main.h:77`
pub const BOT_FLAG_GET_DISTANCE: c_int = 256;

/// Raven `BOT_SABER_THROW_RANGE`.
///
/// Source: `oracle/oracle/codemp/game/ai_main.h:79`
pub const BOT_SABER_THROW_RANGE: c_int = 800;

/// Raven `BOT_THINK_TIME` — bot think interval (0, i.e. re-evaluated every
/// server frame).
///
/// Source: `oracle/oracle/codemp/game/ai_main.c:43`
pub const BOT_THINK_TIME: c_int = 0;
