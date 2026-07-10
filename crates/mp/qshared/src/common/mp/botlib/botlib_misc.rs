//! MP `botlib.h` top-level `#define` constants: API version and the bot
//! files base folder name.
//!
//! Source: `oracle/codemp/game/botlib.h:16,30`

use core::ffi::c_int;

/// Raven `BOTLIB_API_VERSION`.
///
/// Source: `oracle/codemp/game/botlib.h:16`
pub const BOTLIB_API_VERSION: c_int = 2;

/// Raven `BOTFILESBASEFOLDER`.
///
/// Source: `oracle/codemp/game/botlib.h:30`
pub const BOTFILESBASEFOLDER: &str = "botfiles";
