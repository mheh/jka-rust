use core::ffi::c_char;

use crate::shared::vec3_t;

/// Raven `sharedIKMoveParams_t`.
///
/// Raven comment: `rww - update parms for ik bone stuff`
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:2578`
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:933`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct sharedIKMoveParams_t {
    /// Raven `boneName[512]`: name of bone
    pub bone_name: [c_char; 512],
    /// Raven `desiredOrigin`: world coordinate that this bone should be attempting to reach
    pub desired_origin: vec3_t,
    /// Raven `origin`: world coordinate of the entity who owns the g2 instance that owns the bone
    pub origin: vec3_t,
    /// Raven `movementSpeed`: how fast the bone should move toward the destination
    pub movement_speed: f32,
}
