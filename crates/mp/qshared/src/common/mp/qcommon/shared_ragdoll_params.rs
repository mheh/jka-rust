//! MP `sharedRagDollParams_t` copied from Raven `codemp/game/q_shared.h`.
//!
//! Source: `oracle/oracle/codemp/game/q_shared.h:876-922`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

use crate::shared::{qboolean, vec3_t};

/// Raven `sharedRagDollParams_t`.
///
/// Raven comment: `rww - a C-ified structure version of the class which fires off callbacks and gives arguments to update ragdoll status.`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct sharedRagDollParams_t {
    pub angles: vec3_t,
    pub position: vec3_t,
    pub scale: vec3_t,
    /// Raven `pelvisAnglesOffset`: always set on return, an argument for RP_SET_PELVIS_OFFSET
    pub pelvis_angles_offset: vec3_t,
    /// Raven `pelvisPositionOffset`: always set on return, an argument for RP_SET_PELVIS_OFFSET
    pub pelvis_position_offset: vec3_t,
    /// Raven `fImpactStrength`: should be applicable when RagPhase is RP_DEATH_COLLISION
    pub f_impact_strength: f32,
    /// Raven `fShotStrength`: should be applicable for setting velocity of corpse on shot (probably only on RP_CORPSE_SHOT)
    pub f_shot_strength: f32,
    /// Raven `me`: index of entity giving this update
    pub me: c_int,
    /// Raven `startFrame`.
    ///
    /// Raven comment: `rww - we have convenient animation/frame access in the game, so just send this info over from there.`
    pub start_frame: c_int,
    pub end_frame: c_int,
    /// Raven `collisionType`: 1 = from a fall, 0 from effectors, this will be going away soon, hence no enum
    pub collision_type: c_int,
    /// Raven `CallRagDollBegin`: a return value, means that we are now begininng ragdoll and the NPC stuff needs to happen
    pub call_rag_doll_begin: qboolean,
    pub rag_phase: c_int,
    /// Raven `effectorsToTurnOff`: set this to an | of the above flags for a RP_DISABLE_EFFECTORS
    ///
    /// Raven comment: `effector control, used for RP_DISABLE_EFFECTORS call`
    pub effectors_to_turn_off: c_int,
}

const _: () = assert!(core::mem::size_of::<sharedRagDollParams_t>() == 96);
const _: () = assert!(core::mem::offset_of!(sharedRagDollParams_t, angles) == 0);
const _: () = assert!(core::mem::offset_of!(sharedRagDollParams_t, position) == 12);
const _: () = assert!(core::mem::offset_of!(sharedRagDollParams_t, scale) == 24);
const _: () = assert!(core::mem::offset_of!(sharedRagDollParams_t, pelvis_angles_offset) == 36);
const _: () =
    assert!(core::mem::offset_of!(sharedRagDollParams_t, pelvis_position_offset) == 48);
const _: () = assert!(core::mem::offset_of!(sharedRagDollParams_t, f_impact_strength) == 60);
const _: () = assert!(core::mem::offset_of!(sharedRagDollParams_t, me) == 68);
const _: () = assert!(core::mem::offset_of!(sharedRagDollParams_t, start_frame) == 72);
const _: () = assert!(core::mem::offset_of!(sharedRagDollParams_t, collision_type) == 80);
const _: () = assert!(core::mem::offset_of!(sharedRagDollParams_t, effectors_to_turn_off) == 92);
