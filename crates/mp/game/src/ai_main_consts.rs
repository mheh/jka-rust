//! MP `ai_main.h` `LEVELFLAG_*` bit flags (`level.levelFlags` / `gLevelFlags`
//! AI hint bits).
//!
//! Plain `#define` bit flags (not an enum), so §C8 makes them `const`s
//! directly.
//!
//! Source: `oracle/oracle/codemp/game/ai_main.h:40-42`

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
