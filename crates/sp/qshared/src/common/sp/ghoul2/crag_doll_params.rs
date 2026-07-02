#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use crate::shared::vec3_t;

/// Raven `CRagDollParams::ERagPhase` — ragdoll update callback phases.
///
/// Type definition source: `oracle/oracle/code/ghoul2/ghoul2_gore.h:124-133`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ERagPhase {
    RP_START_DEATH_ANIM,
    RP_END_DEATH_ANIM,
    RP_DEATH_COLLISION,
    RP_CORPSE_SHOT,
    /// Raven: this actually does nothing but set the pelvisAnglesOffset, and pelvisPositionOffset
    RP_GET_PELVIS_OFFSET,
    /// Raven: this actually does nothing but set the pelvisAnglesOffset, and pelvisPositionOffset
    RP_SET_PELVIS_OFFSET,
    /// Raven: this removes effectors given by the effectorsToTurnOff member
    RP_DISABLE_EFFECTORS,
}

/// Raven `CRagDollParams::ERagEffector` — ragdoll effector bone bit flags,
/// used for the `RP_DISABLE_EFFECTORS` call.
///
/// Type definition source: `oracle/oracle/code/ghoul2/ghoul2_gore.h:159-186`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ERagEffector {
    RE_MODEL_ROOT = 0x00000001,   // "model_root"
    RE_PELVIS = 0x00000002,       // "pelvis"
    RE_LOWER_LUMBAR = 0x00000004, // "lower_lumbar"
    RE_UPPER_LUMBAR = 0x00000008, // "upper_lumbar"
    RE_THORACIC = 0x00000010,     // "thoracic"
    RE_CRANIUM = 0x00000020,      // "cranium"
    RE_RHUMEROUS = 0x00000040,    // "rhumerus"
    RE_LHUMEROUS = 0x00000080,    // "lhumerus"
    RE_RRADIUS = 0x00000100,      // "rradius"
    RE_LRADIUS = 0x00000200,      // "lradius"
    RE_RFEMURYZ = 0x00000400,     // "rfemurYZ"
    RE_LFEMURYZ = 0x00000800,     // "lfemurYZ"
    RE_RTIBIA = 0x00001000,       // "rtibia"
    RE_LTIBIA = 0x00002000,       // "ltibia"
    RE_RHAND = 0x00004000,        // "rhand"
    RE_LHAND = 0x00008000,        // "lhand"
    RE_RTARSAL = 0x00010000,      // "rtarsal"
    RE_LTARSAL = 0x00020000,      // "ltarsal"
    RE_RTALUS = 0x00040000,       // "rtalus"
    RE_LTALUS = 0x00080000,       // "ltalus"
    RE_RRADIUSX = 0x00100000,     // "rradiusX"
    RE_LRADIUSX = 0x00200000,     // "lradiusX"
    RE_RFEMURX = 0x00400000,      // "rfemurX"
    RE_LFEMURX = 0x00800000,      // "lfemurX"
    RE_CEYEBROW = 0x01000000,     // "ceyebrow"
}

/// Raven `CRagDollParams` — argument/return block passed to the ragdoll
/// update callback across all ragdoll phases.
///
/// Type definition source: `oracle/oracle/code/ghoul2/ghoul2_gore.h:120-190`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CRagDollParams {
    pub angles: vec3_t,
    pub position: vec3_t,
    pub scale: vec3_t,
    /// Raven: always set on return, an argument for RP_SET_PELVIS_OFFSET
    pub pelvisAnglesOffset: vec3_t,
    /// Raven: always set on return, an argument for RP_SET_PELVIS_OFFSET
    pub pelvisPositionOffset: vec3_t,
    /// Raven: should be applicable when RagPhase is RP_DEATH_COLLISION
    pub fImpactStrength: f32,
    /// Raven: should be applicable for setting velocity of corpse on shot (probably only on RP_CORPSE_SHOT)
    pub fShotStrength: f32,
    pub me: c_int,
    pub groundEnt: c_int,
    /// Raven: rww - we have convenient animation/frame access in the game, so just send this info over from there.
    pub startFrame: c_int,
    pub endFrame: c_int,
    /// Raven: 1 = from a fall, 0 from effectors, this will be going away soon, hence no enum
    pub collisionType: c_int,
    /// Raven: a return value, means that we are now begininng ragdoll and the NPC stuff needs to happen
    pub CallRagDollBegin: bool,
    pub RagPhase: ERagPhase,
    /// Raven: set this to an | of the above flags for a RP_DISABLE_EFFECTORS
    ///
    /// Raven comment: effector control, used for RP_DISABLE_EFFECTORS call
    pub effectorsToTurnOff: ERagEffector,
}

const _: () = assert!(core::mem::size_of::<CRagDollParams>() == 100);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, angles) == 0);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, position) == 12);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, scale) == 24);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, pelvisAnglesOffset) == 36);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, pelvisPositionOffset) == 48);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, fImpactStrength) == 60);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, fShotStrength) == 64);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, me) == 68);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, groundEnt) == 72);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, startFrame) == 76);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, endFrame) == 80);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, collisionType) == 84);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, CallRagDollBegin) == 88);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, RagPhase) == 92);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, effectorsToTurnOff) == 96);
