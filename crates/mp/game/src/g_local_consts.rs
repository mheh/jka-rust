//! MP `g_local.h` file-scope `#define` constants.
//!
//! Plain `#define`s (not an enum), so §C8 makes them `const`s directly.
//!
//! Source: `oracle/oracle/codemp/game/g_local.h:41-42`

use core::ffi::c_int;

/// Raven `INTERMISSION_DELAY_TIME`.
///
/// Source: `oracle/oracle/codemp/game/g_local.h:41`
pub const INTERMISSION_DELAY_TIME: c_int = 1000;

/// Raven `SP_INTERMISSION_DELAY_TIME`.
///
/// Source: `oracle/oracle/codemp/game/g_local.h:42`
pub const SP_INTERMISSION_DELAY_TIME: c_int = 5000;

/// Raven `CARNAGE_REWARD_TIME`.
///
/// Source: `oracle/oracle/codemp/game/g_local.h:38`
pub const CARNAGE_REWARD_TIME: c_int = 3000;

/// Raven `START_TIME_LINK_ENTS` — time-delay after map start at which all
/// ents have been spawned, so can link them.
///
/// Source: `oracle/oracle/codemp/game/g_local.h:45`
pub const START_TIME_LINK_ENTS: c_int = crate::g_items::FRAMETIME * 1;

/// Raven `START_TIME_FIND_LINKS` — time-delay after map start at which you
/// can find linked entities.
///
/// Source: `oracle/oracle/codemp/game/g_local.h:46`
pub const START_TIME_FIND_LINKS: c_int = crate::g_items::FRAMETIME * 2;
