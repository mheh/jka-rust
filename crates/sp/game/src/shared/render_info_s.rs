//! SP `renderInfo_t`.
//!
//! Type definition source: `oracle/code/game/g_shared.h:135-224`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use sp_qshared::shared::vec3_t;

use crate::npc::lookMode_t;

/// Anonymous union for `renderInfo_t` (no Raven name — anonymous *and* unnamed in the
/// header, so `legsModelName`/`modelName` are promoted directly onto `renderInfo_t` via
/// GCC's anonymous-union extension; here it needs a field name to exist in Rust).
///
/// Type definition source: `oracle/code/game/g_shared.h:139-143`
#[repr(C)]
#[derive(Clone, Copy)]
pub union renderInfo_t_uModelName {
    /// Legs model, or full model on one piece entities
    pub legsModelName: [u8; 32],
    pub modelName: [u8; 32],
}

/// Raven `renderInfo_t` — per-entity model-rendering state: part yaw/pitch ranges,
/// bone/tag points, muzzle points, look target, head/torso/eye/hand/foot points.
///
/// Type definition source: `oracle/code/game/g_shared.h:135-224`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct renderInfo_t {
    // Legs model, or full model on one piece entities
    pub uModelName: renderInfo_t_uModelName,

    pub torsoModelName: [u8; 32],
    pub headModelName: [u8; 32],

    // In whole degrees, How far to let the different model parts yaw and pitch
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

    // Fields to apply to entire model set, individual model's equivalents will modify this value
    pub customRGBA: [u8; 4], // Red Green Blue, 0 = don't apply

    // Allow up to 4 PCJ lookup values to be stored here.
    // The resolve to configstrings which contain the name of the
    // desired bone.
    pub boneIndex1: c_int,
    pub boneIndex2: c_int,
    pub boneIndex3: c_int,
    pub boneIndex4: c_int,

    // packed with x, y, z orientations for bone angles
    pub boneOrient: c_int,

    // I.. feel bad for doing this, but NPCs really just need to
    // be able to control this sort of thing from the server sometimes.
    // At least it's at the end so this stuff is never going to get sent
    // over for anything that isn't an NPC.
    pub boneAngles1: vec3_t, // angles of boneIndex1
    pub boneAngles2: vec3_t, // angles of boneIndex2
    pub boneAngles3: vec3_t, // angles of boneIndex3
    pub boneAngles4: vec3_t, // angles of boneIndex4

    // RF?
    pub renderFlags: c_int,

    pub muzzlePoint: vec3_t,
    pub muzzleDir: vec3_t,
    pub muzzlePointOld: vec3_t,
    pub muzzleDirOld: vec3_t,
    // vec3_t muzzlePointNext; // Muzzle point one server frame in the future!
    // vec3_t muzzleDirNext;
    pub mPCalcTime: c_int, // Last time muzzle point was calced

    pub lockYaw: f32, //

    pub headPoint: vec3_t,               // Where your tag_head is
    pub headAngles: vec3_t,              // where the tag_head in the torso is pointing
    pub handRPoint: vec3_t,              // where your right hand is
    pub handLPoint: vec3_t,              // where your left hand is
    pub crotchPoint: vec3_t,             // Where your crotch is
    pub footRPoint: vec3_t,              // where your right hand is
    pub footLPoint: vec3_t,              // where your left hand is
    pub torsoPoint: vec3_t,              // Where your chest is
    pub torsoAngles: vec3_t,             // Where the chest is pointing
    pub eyePoint: vec3_t,                // Where your eyes are
    pub eyeAngles: vec3_t,               // Where your eyes face
    pub lookTarget: c_int,               // Which ent to look at with lookAngles
    pub lookMode: lookMode_t,            //
    pub lookTargetClearTime: c_int,      // Time to clear the lookTarget
    pub lastVoiceVolume: c_int,          // Last frame's voice volume
    pub lastHeadAngles: vec3_t,          // Last headAngles, NOT actual facing of head model
    pub headBobAngles: vec3_t,           // headAngle offsets
    pub targetHeadBobAngles: vec3_t,     // head bob angles will try to get to targetHeadBobAngles
    pub lookingDebounceTime: c_int,      // When we can stop using head looking angle behavior
    pub legsYaw: f32,                    // yaw angle your legs are actually rendering at
}
const _: () = assert!(core::mem::size_of::<renderInfo_t>() == 468);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, torsoModelName) == 32);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, headModelName) == 64);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, headYawRangeLeft) == 96);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, headYawRangeRight) == 100);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, headPitchRangeUp) == 104);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, headPitchRangeDown) == 108);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, torsoYawRangeLeft) == 112);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, torsoYawRangeRight) == 116);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, torsoPitchRangeUp) == 120);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, torsoPitchRangeDown) == 124);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, legsFrame) == 128);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, torsoFrame) == 132);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, legsFpsMod) == 136);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, torsoFpsMod) == 140);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, customRGBA) == 144);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, boneIndex1) == 148);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, boneIndex2) == 152);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, boneIndex3) == 156);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, boneIndex4) == 160);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, boneOrient) == 164);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, boneAngles1) == 168);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, boneAngles2) == 180);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, boneAngles3) == 192);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, boneAngles4) == 204);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, renderFlags) == 216);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, muzzlePoint) == 220);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, muzzleDir) == 232);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, muzzlePointOld) == 244);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, muzzleDirOld) == 256);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, mPCalcTime) == 268);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, lockYaw) == 272);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, headPoint) == 276);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, headAngles) == 288);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, handRPoint) == 300);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, handLPoint) == 312);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, crotchPoint) == 324);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, footRPoint) == 336);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, footLPoint) == 348);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, torsoPoint) == 360);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, torsoAngles) == 372);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, eyePoint) == 384);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, eyeAngles) == 396);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, lookTarget) == 408);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, lookMode) == 412);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, lookTargetClearTime) == 416);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, lastVoiceVolume) == 420);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, lastHeadAngles) == 424);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, headBobAngles) == 436);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, targetHeadBobAngles) == 448);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, lookingDebounceTime) == 460);
const _: () = assert!(core::mem::offset_of!(renderInfo_t, legsYaw) == 464);
