#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::{mdxaBone_t, vec3_t};

/// Raven `boneInfo_t` — per-bone animation/override + ragdoll state for a Ghoul2
/// model instance.
///
/// Raven: (none).
/// Type definition source: `oracle/code/game/ghoul2_shared.h:80-183`
#[repr(C)]
pub struct boneInfo_t {
    /// what bone are we overriding?
    pub boneNumber: i32,
    /// details of bone angle overrides - some are pre-done on the server, some in ghoul2
    pub matrix: mdxaBone_t,
    /// flags for override
    pub flags: i32,
    /// start frame for animation
    pub startFrame: i32,
    /// end frame for animation NOTE anim actually ends on endFrame+1
    pub endFrame: i32,
    /// time we started this animation
    pub startTime: i32,
    /// time we paused this animation - 0 if not paused
    pub pauseTime: i32,
    /// speed at which this anim runs. 1.0f means full speed of animation incoming - ie if anim is 20hrtz, we run at 20hrts. If 5hrts, we run at 5 hrts
    pub animSpeed: f32,
    /// frame PLUS LERP value to blend from
    pub blendFrame: f32,
    /// frame to lerp the blend frame with.
    pub blendLerpFrame: i32,
    /// Duration time for blending - used to calc amount each frame of new anim is blended with last frame of the last anim
    pub blendTime: i32,
    /// Time when blending starts - not necessarily the same as startTime since we might start half way through an anim
    pub blendStart: i32,
    /// time for duration of bone angle blend with normal animation
    pub boneBlendTime: i32,
    /// time bone angle blend with normal animation began
    pub boneBlendStart: i32,
    /// This is the lerped matrix that Ghoul2 uses on the client side - does not go across the network
    pub newMatrix: mdxaBone_t,

    // rww - RAGDOLL_BEGIN
    /// if non-zero this is all intialized
    pub lastTimeUpdated: i32,
    pub lastContents: i32,
    pub lastPosition: vec3_t,
    /// I am really tired of recomiling the whole game to add a param here
    pub velocityEffector: vec3_t,
    pub lastAngles: vec3_t,
    pub minAngles: vec3_t,
    pub maxAngles: vec3_t,
    pub currentAngles: vec3_t,
    pub anglesOffset: vec3_t,
    pub positionOffset: vec3_t,
    pub radius: f32,
    /// current radius cubed
    pub weight: f32,
    pub ragIndex: i32,
    /// I am really tired of recomiling the whole game to add a param here
    pub velocityRoot: vec3_t,
    pub ragStartTime: i32,
    pub firstTime: i32,
    pub firstCollisionTime: i32,
    pub restTime: i32,
    pub RagFlags: i32,
    pub DependentRagIndexMask: i32,
    pub originalTrueBoneMatrix: mdxaBone_t,
    /// figure I will need this sooner or later
    pub parentTrueBoneMatrix: mdxaBone_t,
    /// figure I will need this sooner or later
    pub parentOriginalTrueBoneMatrix: mdxaBone_t,
    pub originalOrigin: vec3_t,
    pub originalAngles: vec3_t,
    pub lastShotDir: vec3_t,
    pub basepose: *mut mdxaBone_t,
    pub baseposeInv: *mut mdxaBone_t,
    pub baseposeParent: *mut mdxaBone_t,
    pub baseposeInvParent: *mut mdxaBone_t,
    pub parentRawBoneIndex: i32,
    /// figure I will need this sooner or later
    pub ragOverrideMatrix: mdxaBone_t,

    /// figure I will need this sooner or later
    pub extraMatrix: mdxaBone_t,
    /// I am really tired of recomiling the whole game to add a param here
    pub extraVec1: vec3_t,
    pub extraFloat1: f32,
    pub extraInt1: i32,

    pub ikPosition: vec3_t,
    pub ikSpeed: f32,

    // new ragdoll stuff -rww
    /// velocity factor, can be set, and is also maintained by physics based on gravity, mass, etc.
    pub epVelocity: vec3_t,
    /// gravity factor maintained by bone physics
    pub epGravFactor: f32,
    /// incremented every time we try to move and are in solid - if we get out of solid, it is reset to 0
    pub solidCount: i32,
    /// true when the bone is on ground and finished bouncing, etc. but may still be pushed into solid by other bones
    pub physicsSettled: bool,
    /// the bone is broken out of standard constraints
    pub snapped: bool,

    pub parentBoneIndex: i32,

    pub offsetRotation: f32,

    // user api overrides
    pub overGradSpeed: f32,

    pub overGoalSpot: vec3_t,
    pub hasOverGoal: bool,

    /// matrix for the bone in the desired settling pose -rww
    pub animFrameMatrix: mdxaBone_t,
    pub hasAnimFrameMatrix: i32,

    /// base is in air, be more quick and sensitive about collisions
    pub airTime: i32,
    // rww - RAGDOLL_END
}

const _: () = assert!(core::mem::size_of::<boneInfo_t>() == 760);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, boneNumber) == 0);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, matrix) == 4);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, flags) == 52);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, startFrame) == 56);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, endFrame) == 60);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, startTime) == 64);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, pauseTime) == 68);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, animSpeed) == 72);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, blendFrame) == 76);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, blendLerpFrame) == 80);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, blendTime) == 84);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, blendStart) == 88);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, boneBlendTime) == 92);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, boneBlendStart) == 96);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, newMatrix) == 100);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, lastTimeUpdated) == 148);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, lastContents) == 152);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, lastPosition) == 156);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, velocityEffector) == 168);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, lastAngles) == 180);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, minAngles) == 192);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, maxAngles) == 204);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, currentAngles) == 216);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, anglesOffset) == 228);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, positionOffset) == 240);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, radius) == 252);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, weight) == 256);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, ragIndex) == 260);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, velocityRoot) == 264);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, ragStartTime) == 276);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, firstTime) == 280);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, firstCollisionTime) == 284);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, restTime) == 288);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, RagFlags) == 292);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, DependentRagIndexMask) == 296);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, originalTrueBoneMatrix) == 300);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, parentTrueBoneMatrix) == 348);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, parentOriginalTrueBoneMatrix) == 396);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, originalOrigin) == 444);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, originalAngles) == 456);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, lastShotDir) == 468);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, basepose) == 480);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, baseposeInv) == 488);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, baseposeParent) == 496);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, baseposeInvParent) == 504);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, parentRawBoneIndex) == 512);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, ragOverrideMatrix) == 516);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, extraMatrix) == 564);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, extraVec1) == 612);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, extraFloat1) == 624);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, extraInt1) == 628);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, ikPosition) == 632);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, ikSpeed) == 644);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, epVelocity) == 648);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, epGravFactor) == 660);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, solidCount) == 664);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, physicsSettled) == 668);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, snapped) == 669);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, parentBoneIndex) == 672);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, offsetRotation) == 676);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, overGradSpeed) == 680);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, overGoalSpot) == 684);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, hasOverGoal) == 696);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, animFrameMatrix) == 700);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, hasAnimFrameMatrix) == 748);
const _: () = assert!(core::mem::offset_of!(boneInfo_t, airTime) == 752);
