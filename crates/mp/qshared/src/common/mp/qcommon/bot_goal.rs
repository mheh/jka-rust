//! MP `bot_goal_t` copied from Raven `codemp/game/be_ai_goal.h`.
//!
//! Source: `oracle/codemp/game/be_ai_goal.h:25-34`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

use crate::shared::vec3_t;

/// Raven `GFL_NONE` — no goal flags set.
/// Source: `oracle/codemp/game/be_ai_goal.h:19`
pub const GFL_NONE: c_int = 0;

/// Raven `GFL_ITEM` — goal is an item.
/// Source: `oracle/codemp/game/be_ai_goal.h:20`
pub const GFL_ITEM: c_int = 1;

/// Raven `GFL_ROAM` — goal is a roam goal.
/// Source: `oracle/codemp/game/be_ai_goal.h:21`
pub const GFL_ROAM: c_int = 2;

/// Raven `GFL_DROPPED` — goal is a dropped item.
/// Source: `oracle/codemp/game/be_ai_goal.h:22`
pub const GFL_DROPPED: c_int = 4;

/// Raven `bot_goal_t`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct bot_goal_t {
    /// Raven `origin`: origin of the goal
    pub origin: vec3_t,
    /// Raven `areanum`: area number of the goal
    pub areanum: c_int,
    /// Raven `mins`: mins of the goal
    pub mins: vec3_t,
    /// Raven `maxs`: maxs of the goal
    pub maxs: vec3_t,
    /// Raven `entitynum`: number of the goal entity
    pub entitynum: c_int,
    /// Raven `number`: goal number
    pub number: c_int,
    /// Raven `flags`: goal flags
    pub flags: c_int,
    /// Raven `iteminfo`: item information
    pub iteminfo: c_int,
}

const _: () = assert!(core::mem::size_of::<bot_goal_t>() == 56);
const _: () = assert!(core::mem::offset_of!(bot_goal_t, origin) == 0);
const _: () = assert!(core::mem::offset_of!(bot_goal_t, areanum) == 12);
const _: () = assert!(core::mem::offset_of!(bot_goal_t, mins) == 16);
const _: () = assert!(core::mem::offset_of!(bot_goal_t, maxs) == 28);
const _: () = assert!(core::mem::offset_of!(bot_goal_t, entitynum) == 40);
const _: () = assert!(core::mem::offset_of!(bot_goal_t, number) == 44);
const _: () = assert!(core::mem::offset_of!(bot_goal_t, flags) == 48);
const _: () = assert!(core::mem::offset_of!(bot_goal_t, iteminfo) == 52);
