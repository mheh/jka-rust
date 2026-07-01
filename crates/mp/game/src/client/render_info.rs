//! MP `renderInfo_t`.
//!
//! Type definition source: `oracle/oracle/codemp/game/g_local.h:460-532`

#![allow(non_camel_case_types)]

use core::ffi::{c_int, c_void};

use mp_qshared::shared::vec3_t;

use crate::npc::lookMode_t;

/// Raven `renderInfo_t` — per-client model-rendering state: part yaw/pitch ranges,
/// muzzle points, tag points, look target, bolt indices, `lastG2`.
///
/// Pointer-bearing (`lastG2`) => arch-dependent; asserts pin the host-64-bit layout.
/// Type definition source: `oracle/oracle/codemp/game/g_local.h:460-532`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct renderInfo_t {
    // In whole degrees, how far to let the different model parts yaw and pitch
    pub headYawRangeLeft: c_int,
    pub headYawRangeRight: c_int,
    pub headPitchRangeUp: c_int,
    pub headPitchRangeDown: c_int,

    pub torsoYawRangeLeft: c_int,
    pub torsoYawRangeRight: c_int,
    pub torsoPitchRangeUp: c_int,
    pub torsoPitchRangeDown: c_int,

    pub legsFrame: c_int,
    pub torsoFrame: c_int,

    pub legsFpsMod: f32,
    pub torsoFpsMod: f32,

    pub customRGB: vec3_t,  // Red Green Blue, 0 = don't apply
    pub customAlpha: c_int, // Alpha to apply, 0 = none?

    pub renderFlags: c_int,

    pub muzzlePoint: vec3_t,
    pub muzzleDir: vec3_t,
    pub muzzlePointOld: vec3_t,
    pub muzzleDirOld: vec3_t,
    pub mPCalcTime: c_int, // Last time muzzle point was calced

    pub lockYaw: f32,

    pub headPoint: vec3_t,   // Where your tag_head is
    pub headAngles: vec3_t,  // where the tag_head in the torso is pointing
    pub handRPoint: vec3_t,  // where your right hand is
    pub handLPoint: vec3_t,  // where your left hand is
    pub crotchPoint: vec3_t, // Where your crotch is
    pub footRPoint: vec3_t,  // where your right foot is
    pub footLPoint: vec3_t,  // where your left foot is
    pub torsoPoint: vec3_t,  // Where your chest is
    pub torsoAngles: vec3_t, // Where the chest is pointing
    pub eyePoint: vec3_t,    // Where your eyes are
    pub eyeAngles: vec3_t,   // Where your eyes face
    pub lookTarget: c_int,   // Which ent to look at with lookAngles
    pub lookMode: lookMode_t,
    pub lookTargetClearTime: c_int,  // Time to clear the lookTarget
    pub lastVoiceVolume: c_int,      // Last frame's voice volume
    pub lastHeadAngles: vec3_t,      // Last headAngles, NOT actual facing of head model
    pub headBobAngles: vec3_t,       // headAngle offsets
    pub targetHeadBobAngles: vec3_t, // head bob angles will try to get to targetHeadBobAngles
    pub lookingDebounceTime: c_int,  // When we can stop using head looking angle behavior
    pub legsYaw: f32,                // yaw angle your legs are actually rendering at

    // for tracking legitimate bolt indecies
    pub lastG2: *mut c_void, // if it doesn't match ent->ghoul2, the bolts are considered invalid.
    pub headBolt: c_int,
    pub handRBolt: c_int,
    pub handLBolt: c_int,
    pub torsoBolt: c_int,
    pub crotchBolt: c_int,
    pub footRBolt: c_int,
    pub footLBolt: c_int,
    pub motionBolt: c_int,

    pub boltValidityTime: c_int,
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<renderInfo_t>() == 368);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(renderInfo_t, lookMode) == 260);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(renderInfo_t, lastG2) == 320);
