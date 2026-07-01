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

const _: () = assert!(core::mem::size_of::<sharedIKMoveParams_t>() == 540);
const _: () = assert!(core::mem::offset_of!(sharedIKMoveParams_t, bone_name) == 0);
const _: () = assert!(core::mem::offset_of!(sharedIKMoveParams_t, desired_origin) == 512);
const _: () = assert!(core::mem::offset_of!(sharedIKMoveParams_t, origin) == 524);
const _: () = assert!(core::mem::offset_of!(sharedIKMoveParams_t, movement_speed) == 536);
