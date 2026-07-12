#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_float, c_int};

use mp_qshared::shared::{qboolean, sharedERagEffector, sharedERagPhase, vec3_t};

/// Raven `CRagDollParams` — ragdoll update callback argument/result block.
///
/// Raven: nests `enum ERagPhase` and `enum ERagEffector`. Both are duplicated
/// verbatim (same variants/values) by `q_shared.h`'s "C-ified" `sharedERagPhase`
/// / `sharedERagEffector` — already ported at `mp_qshared::shared` — and reused
/// here rather than re-declaring identical enums.
/// Type definition source: `oracle/codemp/ghoul2/G2_gore.h:131-199`
/// (`ERagPhase`: 135-144, `ERagEffector`: 168-195; cf.
/// `oracle/codemp/game/q_shared.h:856-894`)
#[repr(C)]
pub struct CRagDollParams {
    pub angles: vec3_t,
    pub position: vec3_t,
    pub scale: vec3_t,
    /// Raven: always set on return, an argument for RP_SET_PELVIS_OFFSET
    pub pelvisAnglesOffset: vec3_t,
    /// Raven: always set on return, an argument for RP_SET_PELVIS_OFFSET
    pub pelvisPositionOffset: vec3_t,

    /// Raven: should be applicable when RagPhase is RP_DEATH_COLLISION
    pub fImpactStrength: c_float,
    /// Raven: should be applicable for setting velocity of corpse on shot (probably only on RP_CORPSE_SHOT)
    pub fShotStrength: c_float,
    // Raven: `CServerEntity *me;` — replaced by an index, see comment below.
    pub me: c_int,

    // rww - we have convenient animation/frame access in the game, so just send this info over from there.
    pub startFrame: c_int,
    pub endFrame: c_int,

    /// Raven: 1 = from a fall, 0 from effectors, this will be going away soon, hence no enum
    pub collisionType: c_int,

    /// Raven: a return value, means that we are now begininng ragdoll and the NPC stuff needs to happen
    pub CallRagDollBegin: qboolean,

    pub RagPhase: sharedERagPhase,

    // effector control, used for RP_DISABLE_EFFECTORS call
    /// Raven: set this to an | of the above flags for a RP_DISABLE_EFFECTORS
    pub effectorsToTurnOff: sharedERagEffector,
}

const _: () = assert!(core::mem::size_of::<CRagDollParams>() == 96);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, angles) == 0);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, position) == 12);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, scale) == 24);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, pelvisAnglesOffset) == 36);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, pelvisPositionOffset) == 48);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, fImpactStrength) == 60);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, fShotStrength) == 64);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, me) == 68);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, startFrame) == 72);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, endFrame) == 76);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, collisionType) == 80);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, CallRagDollBegin) == 84);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, RagPhase) == 88);
const _: () = assert!(core::mem::offset_of!(CRagDollParams, effectorsToTurnOff) == 92);
