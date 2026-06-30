//! SP `sharedSetBoneIKStateParams_t` copied from Raven `code/game/q_shared.h`.
//!
//! Source: `oracle/oracle/code/game/q_shared.h:2591-2602`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

use crate::shared::vec3_t;

/// Raven `sharedSetBoneIKStateParams_t`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct sharedSetBoneIKStateParams_t {
    /// Raven `pcjMins`: ik joint limit
    pub pcj_mins: vec3_t,
    /// Raven `pcjMaxs`: ik joint limit
    pub pcj_maxs: vec3_t,
    /// Raven `origin`: origin of caller
    pub origin: vec3_t,
    /// Raven `angles`: angles of caller
    pub angles: vec3_t,
    /// Raven `scale`: scale of caller
    pub scale: vec3_t,
    /// Raven `radius`: bone rad
    pub radius: f32,
    /// Raven `blendTime`: bone blend time
    pub blend_time: c_int,
    /// Raven `pcjOverrides`: override ik bone flags
    pub pcj_overrides: c_int,
    /// Raven `startFrame`: base pose start
    pub start_frame: c_int,
    /// Raven `endFrame`: base pose end
    pub end_frame: c_int,
}
