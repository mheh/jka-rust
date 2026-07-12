//! MP `botlib.h` print type constants (used with `BotAI_Print`/`BotImport_Print`).
//!
//! Plain `#define`s (not an enum), so §C8 makes them `const`s directly.
//!
//! Source: `oracle/codemp/game/botlib.h:39-44`

use core::ffi::c_int;

/// Raven `PRT_MESSAGE`.
///
/// Source: `oracle/codemp/game/botlib.h:40`
pub const PRT_MESSAGE: c_int = 1;

/// Raven `PRT_WARNING`.
///
/// Source: `oracle/codemp/game/botlib.h:41`
pub const PRT_WARNING: c_int = 2;

/// Raven `PRT_ERROR`.
///
/// Source: `oracle/codemp/game/botlib.h:42`
pub const PRT_ERROR: c_int = 3;

/// Raven `PRT_FATAL`.
///
/// Source: `oracle/codemp/game/botlib.h:43`
pub const PRT_FATAL: c_int = 4;

/// Raven `PRT_EXIT`.
///
/// Source: `oracle/codemp/game/botlib.h:44`
pub const PRT_EXIT: c_int = 5;
