//! MP `botlib.h` console message type constants.
//!
//! Plain `#define`s (not an enum), so §C8 makes them `const`s directly.
//!
//! Source: `oracle/codemp/game/botlib.h:46-48`

use core::ffi::c_int;

/// Raven `CMS_NORMAL`.
///
/// Source: `oracle/codemp/game/botlib.h:47`
pub const CMS_NORMAL: c_int = 0;

/// Raven `CMS_CHAT`.
///
/// Source: `oracle/codemp/game/botlib.h:48`
pub const CMS_CHAT: c_int = 1;
