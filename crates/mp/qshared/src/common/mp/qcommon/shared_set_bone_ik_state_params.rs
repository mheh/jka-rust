//! MP `sharedSetBoneIKStateParams_t` copied from Raven `codemp/game/q_shared.h`.
//!
//! Source: `oracle/oracle/codemp/game/q_shared.h:946-958`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

use crate::shared::{qboolean, vec3_t};

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
    /// Raven `forceAnimOnBone`: normally if the bone has specified start/end frames already it will leave it alone.. if this is true, then the animation will be restarted on the bone with the specified frames anyway.
    pub force_anim_on_bone: qboolean,
}

const _: () = assert!(core::mem::size_of::<sharedSetBoneIKStateParams_t>() == 84);
const _: () = assert!(core::mem::offset_of!(sharedSetBoneIKStateParams_t, pcj_mins) == 0);
const _: () = assert!(core::mem::offset_of!(sharedSetBoneIKStateParams_t, origin) == 24);
const _: () = assert!(core::mem::offset_of!(sharedSetBoneIKStateParams_t, scale) == 48);
const _: () = assert!(core::mem::offset_of!(sharedSetBoneIKStateParams_t, radius) == 60);
const _: () = assert!(core::mem::offset_of!(sharedSetBoneIKStateParams_t, start_frame) == 72);
const _: () = assert!(core::mem::offset_of!(sharedSetBoneIKStateParams_t, force_anim_on_bone) == 80);
