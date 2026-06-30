//! MP `bot_goal_t` copied from Raven `codemp/game/be_ai_goal.h`.
//!
//! Source: `oracle/oracle/codemp/game/be_ai_goal.h:25-34`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

use crate::shared::vec3_t;

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
