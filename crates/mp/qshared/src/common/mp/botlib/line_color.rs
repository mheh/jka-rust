//! MP `botlib.h` debug line color constants.
//!
//! Plain `#define`s (not an enum), so §C8 makes them `const`s directly.
//!
//! Source: `oracle/codemp/game/botlib.h:31-37`

use core::ffi::c_int;

/// Raven `LINECOLOR_NONE`.
///
/// Source: `oracle/codemp/game/botlib.h:32`
pub const LINECOLOR_NONE: c_int = -1;

/// Raven `LINECOLOR_RED`.
///
/// Source: `oracle/codemp/game/botlib.h:33`
pub const LINECOLOR_RED: c_int = 1;

/// Raven `LINECOLOR_GREEN`.
///
/// Source: `oracle/codemp/game/botlib.h:34`
pub const LINECOLOR_GREEN: c_int = 2;

/// Raven `LINECOLOR_BLUE`.
///
/// Source: `oracle/codemp/game/botlib.h:35`
pub const LINECOLOR_BLUE: c_int = 3;

/// Raven `LINECOLOR_YELLOW`.
///
/// Source: `oracle/codemp/game/botlib.h:36`
pub const LINECOLOR_YELLOW: c_int = 4;

/// Raven `LINECOLOR_ORANGE`.
///
/// Source: `oracle/codemp/game/botlib.h:37`
pub const LINECOLOR_ORANGE: c_int = 5;
