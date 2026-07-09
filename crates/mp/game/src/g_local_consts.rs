//! MP `g_local.h` file-scope `#define` constants.
//!
//! Plain `#define`s (not an enum), so §C8 makes them `const`s directly.
//!
//! Source: `oracle/codemp/game/g_local.h:41-42`

use core::ffi::c_int;

/// Raven `INTERMISSION_DELAY_TIME`.
///
/// Source: `oracle/codemp/game/g_local.h:41`
pub const INTERMISSION_DELAY_TIME: c_int = 1000;

/// Raven `SP_INTERMISSION_DELAY_TIME`.
///
/// Source: `oracle/codemp/game/g_local.h:42`
pub const SP_INTERMISSION_DELAY_TIME: c_int = 5000;

/// Raven `CARNAGE_REWARD_TIME`.
///
/// Source: `oracle/codemp/game/g_local.h:38`
pub const CARNAGE_REWARD_TIME: c_int = 3000;

/// Raven `START_TIME_LINK_ENTS` — time-delay after map start at which all
/// ents have been spawned, so can link them.
///
/// Source: `oracle/codemp/game/g_local.h:45`
pub const START_TIME_LINK_ENTS: c_int = crate::g_items::FRAMETIME * 1;

/// Raven `START_TIME_FIND_LINKS` — time-delay after map start at which you
/// can find linked entities.
///
/// Source: `oracle/codemp/game/g_local.h:46`
pub const START_TIME_FIND_LINKS: c_int = crate::g_items::FRAMETIME * 2;

/// Raven `START_TIME_NAV_CALC` — time-delay after map start to connect
/// waypoints and calc routes.
///
/// Source: `oracle/codemp/game/g_local.h:49`
pub const START_TIME_NAV_CALC: c_int = crate::g_items::FRAMETIME * 4;

/// Raven `MAX_G_SHARED_BUFFER_SIZE` — size of `gSharedBuffer`, the module's
/// engine-registered shared-memory region.
///
/// Source: `oracle/codemp/game/g_local.h:85`
pub const MAX_G_SHARED_BUFFER_SIZE: usize = 8192;

/// Raven `SP_PODIUM_MODEL`.
///
/// Source: `oracle/codemp/game/g_local.h:96`
pub const SP_PODIUM_MODEL: &core::ffi::CStr = c"models/mapobjects/podium/podium4.md3";
