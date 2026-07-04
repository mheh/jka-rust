// PORT-COMPLETE: bg_pmove.c 11/91
//! FAITHFUL signature skeleton for `oracle/oracle/codemp/game/bg_pmove.c`.
//!
//! Bodies filled per the settled fork rulings. The vast majority of this file
//! is built on the file-static pmove working set (`pmove_t *pm`, `pml_t pml`,
//! `bgEntity_t *pm_entSelf`, `pm_entVeh`, `pm_flying`, `gPMDoSlowFall`,
//! `pm_cancelOutZoom`). Porting-rules §B3 forbids `static mut`/hidden globals,
//! but the faithful no-arg C signatures here thread no `pm`/engine context, so
//! the representation of that working set is a genuine unsettled design fork.
//! Every function that reads/writes it — and every function whose skeleton
//! signature passes a mutated `vec3_t` out-param BY VALUE (`[f32;3]` is `Copy`,
//! so in-place writes cannot propagate) — is parked with a `PORT-ESCALATION`.
//! The clean, pointer-parameterized / pure functions are ported.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

// Raven `qboolean` is `c_int`; keep the source spelling at assignment sites.
// Source: `oracle/oracle/codemp/game/q_shared.h`
const qtrue: qboolean = 1;
const qfalse: qboolean = 0;
use crate::g_strap::strap_G2API_SetBoneAngles;
use crate::q_math::{AngleMod, AngleSubtract};
use crate::q_math::{PITCH, ROLL, YAW};

// Unported types referenced in this file (need porting before this compiles):
// void ()(trace_t , vec_t , vec_t , vec_t , vec_t , int, int)

/// Raven `BONE_ANGLES_POSTMULT` (ghoul2 bone-angle apply mode).
/// Source: `oracle/oracle/code/game/ghoul2_shared.h:54`
const BONE_ANGLES_POSTMULT: c_int = 0x0002;


/// Raven `PM_BGEntForNum`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:172-199`
// PORT-ESCALATION(pmove-working-state): how is the file-static `pmove_t *pm` (and
// `pml`/`pm_entSelf`) threaded into these no-arg C-signature fns without a §B3 static?
pub fn PM_BGEntForNum(
    num: c_int,
) -> *mut bgEntity_t {
    todo!("Port PM_BGEntForNum — parked: pmove-working-state")
}

/// Raven `BG_SabersOff`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:201-216`
pub fn BG_SabersOff(
    ps: *mut playerState_t,
) -> qboolean {
    unsafe {
        if (*ps).saberHolstered == 0 {
            return qfalse;
        }
        if (*ps).fd.saberAnimLevelBase as c_int == saber_styles_t::SS_DUAL as c_int
            || (*ps).fd.saberAnimLevelBase as c_int == saber_styles_t::SS_STAFF as c_int
        {
            if (*ps).saberHolstered < 2 {
                return qfalse;
            }
        }
        qtrue
    }
}

/// Raven `BG_KnockDownable`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:218-237`
pub fn BG_KnockDownable(
    ps: *mut playerState_t,
) -> qboolean {
    unsafe {
        if ps.is_null() {
            // just for safety
            return qfalse;
        }
        if (*ps).m_iVehicleNum != 0 {
            // riding a vehicle, don't knock me down
            return qfalse;
        }
        if (*ps).emplacedIndex != 0 {
            // using emplaced gun or eweb, can't be knocked down
            return qfalse;
        }
        // ok, I guess?
        qtrue
    }
}

/// Raven `PM_IsRocketTrooper`.
///
/// Raven: hacky assumption check — the humanoid/siege check is commented out in
/// the oracle; the live path always returns qfalse.
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:247-259`
pub fn PM_IsRocketTrooper() -> qboolean {
    qfalse
}

/// Raven `PM_GetSaberStance`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:261-319`
// PORT-ESCALATION(pmove-working-state): reads `pm->ps->...`; needs the pmove working-set threading decision.
pub fn PM_GetSaberStance() -> c_int {
    todo!("Port PM_GetSaberStance — parked: pmove-working-state")
}

/// Raven `PM_DoSlowFall`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:321-329`
// PORT-ESCALATION(pmove-working-state): reads `pm->ps`; needs the pmove working-set threading decision.
pub fn PM_DoSlowFall() -> qboolean {
    todo!("Port PM_DoSlowFall — parked: pmove-working-state")
}

/// Raven `PM_pitch_roll_for_slope`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:346-439`
// PORT-ESCALATION(pmove-working-state): writes `pm`; `storeAngles` is also a by-value vec3_t out-param.
pub fn PM_pitch_roll_for_slope(
    forwhom: *mut bgEntity_t,
    pass_slope: vec3_t,
    storeAngles: vec3_t,
) {
    todo!("Port PM_pitch_roll_for_slope — parked: pmove-working-state")
}

/// Raven `PM_SetSpecialMoveValues`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:447-480`
// PORT-ESCALATION(pmove-working-state): reads `pm`/`pm_entSelf`, writes `pm_flying`.
pub fn PM_SetSpecialMoveValues() {
    todo!("Port PM_SetSpecialMoveValues — parked: pmove-working-state")
}

/// Raven `PM_SetVehicleAngles`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:482-635`
// PORT-ESCALATION(pmove-working-state): reads `pm`/`pm_entSelf`/`pml`.
pub fn PM_SetVehicleAngles(
    normal: vec3_t,
) {
    todo!("Port PM_SetVehicleAngles — parked: pmove-working-state")
}

/// Raven `BG_ExternThisSoICanRecompileInDebug`.
///
/// Raven: the entire body is commented out in the oracle (a debug-recompile
/// hook); it is a no-op.
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:641-674`
pub fn BG_ExternThisSoICanRecompileInDebug(
    pVeh: *mut Vehicle_t,
    riderPS: *mut playerState_t,
) {
    // No-op: the oracle body is entirely `/* ... */`-commented.
}

/// Raven `BG_VehicleTurnRateForSpeed`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:676-706`
// PORT-ESCALATION(missing-const): `MIN_LANDING_SLOPE` (#define, bg_vehicles.h) is not
// resolved in the packet; the slope compare needs its value — no invention allowed.
pub fn BG_VehicleTurnRateForSpeed(
    pVeh: *mut Vehicle_t,
    speed: f32,
    mPitchOverride: *mut f32,
    mYawOverride: *mut f32,
) {
    todo!("Port BG_VehicleTurnRateForSpeed — parked: missing-const")
}

/// Raven `PM_HoverTrace`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:719-901`
// PORT-ESCALATION(pmove-working-state): reads `pm_entSelf`, writes `pm`/`pml`.
pub fn PM_HoverTrace() {
    todo!("Port PM_HoverTrace — parked: pmove-working-state")
}

/// Raven `PM_AddEvent`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:910-912`
// PORT-ESCALATION(pmove-working-state): reads `pm`.
pub fn PM_AddEvent(
    newEvent: c_int,
) {
    todo!("Port PM_AddEvent — parked: pmove-working-state")
}

/// Raven `PM_AddEventWithParm`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:914-917`
// PORT-ESCALATION(pmove-working-state): reads `pm`.
pub fn PM_AddEventWithParm(
    newEvent: c_int,
    parm: c_int,
) {
    todo!("Port PM_AddEventWithParm — parked: pmove-working-state")
}

/// Raven `PM_AddTouchEnt`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:924-944`
// PORT-ESCALATION(pmove-working-state): writes `pm->touchents`.
pub fn PM_AddTouchEnt(
    entityNum: c_int,
) {
    todo!("Port PM_AddTouchEnt — parked: pmove-working-state")
}

/// Raven `PM_ClipVelocity`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:954-988`
// PORT-ESCALATION(pmove-working-state): reads `pm`; `out` is also a by-value vec3_t out-param.
pub fn PM_ClipVelocity(
    r#in: vec3_t,
    normal: vec3_t,
    out: vec3_t,
    overbounce: f32,
) {
    todo!("Port PM_ClipVelocity — parked: pmove-working-state")
}

/// Raven `PM_Friction`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:998-1123`
// PORT-ESCALATION(pmove-working-state): reads `pm`/`pm_entSelf`/`pm_flying`/`pml` + movement-param globals.
pub fn PM_Friction() {
    todo!("Port PM_Friction — parked: pmove-working-state")
}

/// Raven `PM_Accelerate`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1133-1186`
// PORT-ESCALATION(pmove-working-state): reads `pml`, writes `pm`.
pub fn PM_Accelerate(
    wishdir: vec3_t,
    wishspeed: f32,
    accel: f32,
) {
    todo!("Port PM_Accelerate — parked: pmove-working-state")
}

/// Raven `PM_CmdScale`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1199-1222`
// PORT-ESCALATION(pmove-working-state): reads `pm`.
pub fn PM_CmdScale(
    cmd: *mut usercmd_t,
) -> f32 {
    todo!("Port PM_CmdScale — parked: pmove-working-state")
}

/// Raven `PM_SetMovementDir`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1233-1262`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_SetMovementDir() {
    todo!("Port PM_SetMovementDir — parked: pmove-working-state")
}

/// Raven `PM_ForceJumpingUp`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1266-1306`
// PORT-ESCALATION(pmove-working-state): reads `pm`.
pub fn PM_ForceJumpingUp() -> qboolean {
    todo!("Port PM_ForceJumpingUp — parked: pmove-working-state")
}

/// Raven `PM_JumpForDir`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1308-1340`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_JumpForDir() {
    todo!("Port PM_JumpForDir — parked: pmove-working-state")
}

/// Raven `PM_SetPMViewAngle`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1342-1354`
pub fn PM_SetPMViewAngle(
    ps: *mut playerState_t,
    angle: vec3_t,
    ucmd: *mut usercmd_t,
) {
    unsafe {
        for i in 0..3 {
            // set the delta angle. Raven `ANGLE2SHORT(x)` == `((int)((x)*65536/360) & 65535)`.
            let cmdAngle: c_int = ((angle[i] * 65536.0 / 360.0) as c_int) & 65535;
            (*ps).delta_angles[i] = cmdAngle - (*ucmd).angles[i];
        }
        (*ps).viewangles = angle;
    }
}

/// Raven `PM_AdjustAngleForWallRun`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1356-1462`
// PORT-ESCALATION(pmove-working-state): reads `pm`.
pub fn PM_AdjustAngleForWallRun(
    ps: *mut playerState_t,
    ucmd: *mut usercmd_t,
    doMove: qboolean,
) -> qboolean {
    todo!("Port PM_AdjustAngleForWallRun — parked: pmove-working-state")
}

/// Raven `PM_AdjustAnglesForWallRunUpFlipAlt`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1464-1470`
// PORT-ESCALATION(pmove-working-state): reads `pm`.
pub fn PM_AdjustAnglesForWallRunUpFlipAlt(
    ucmd: *mut usercmd_t,
) -> qboolean {
    todo!("Port PM_AdjustAnglesForWallRunUpFlipAlt — parked: pmove-working-state")
}

/// Raven `PM_AdjustAngleForWallRunUp`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1472-1598`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_AdjustAngleForWallRunUp(
    ps: *mut playerState_t,
    ucmd: *mut usercmd_t,
    doMove: qboolean,
) -> qboolean {
    todo!("Port PM_AdjustAngleForWallRunUp — parked: pmove-working-state")
}

/// Raven `BG_ForceWallJumpStrength`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1602-1605`
// PORT-ESCALATION(missing-const): needs the `forceJumpStrength` table whose element 0 is
// the #define `JUMP_VELOCITY` (not resolved in packet) — can't define the table without it.
pub fn BG_ForceWallJumpStrength() -> f32 {
    todo!("Port BG_ForceWallJumpStrength — parked: missing-const")
}

/// Raven `PM_AdjustAngleForWallJump`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1607-1756`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_AdjustAngleForWallJump(
    ps: *mut playerState_t,
    ucmd: *mut usercmd_t,
    doMove: qboolean,
) -> qboolean {
    todo!("Port PM_AdjustAngleForWallJump — parked: pmove-working-state")
}

/// Raven `PM_SetForceJumpZStart`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1759-1766`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_SetForceJumpZStart(
    value: f32,
) {
    todo!("Port PM_SetForceJumpZStart — parked: pmove-working-state")
}

/// Raven `PM_GrabWallForJump`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1776-1781`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_GrabWallForJump(
    anim: c_int,
) {
    todo!("Port PM_GrabWallForJump — parked: pmove-working-state")
}

/// Raven `PM_CheckJump`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1788-2775`
// PORT-ESCALATION(pmove-working-state): reads/writes `pm`/`pml`/`pm_entSelf`.
pub fn PM_CheckJump() -> qboolean {
    todo!("Port PM_CheckJump — parked: pmove-working-state")
}

/// Raven `PM_CheckWaterJump`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:2781-2821`
// PORT-ESCALATION(pmove-working-state): reads `pml`, writes `pm`.
pub fn PM_CheckWaterJump() -> qboolean {
    todo!("Port PM_CheckWaterJump — parked: pmove-working-state")
}

/// Raven `PM_WaterJumpMove`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:2833-2844`
// PORT-ESCALATION(pmove-working-state): reads `pml`, writes `pm`.
pub fn PM_WaterJumpMove() {
    todo!("Port PM_WaterJumpMove — parked: pmove-working-state")
}

/// Raven `PM_WaterMove`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:2852-2916`
// PORT-ESCALATION(pmove-working-state): reads `pm`/`pml` + movement-param globals.
pub fn PM_WaterMove() {
    todo!("Port PM_WaterMove — parked: pmove-working-state")
}

/// Raven `PM_FlyVehicleMove`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:2924-3012`
// PORT-ESCALATION(pmove-working-state): reads `pml`, writes `pm`.
pub fn PM_FlyVehicleMove() {
    todo!("Port PM_FlyVehicleMove — parked: pmove-working-state")
}

/// Raven `PM_FlyMove`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:3021-3059`
// PORT-ESCALATION(pmove-working-state): reads `pm`/`pml`.
pub fn PM_FlyMove() {
    todo!("Port PM_FlyMove — parked: pmove-working-state")
}

/// Raven `PM_AirMove`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:3068-3297`
// PORT-ESCALATION(pmove-working-state): reads `gPMDoSlowFall`/`pm`/`pm_entSelf`, writes `pml`.
pub fn PM_AirMove() {
    todo!("Port PM_AirMove — parked: pmove-working-state")
}

/// Raven `PM_WalkMove`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:3305-3484`
// PORT-ESCALATION(pmove-working-state): reads/writes `pm`/`pml`/`pm_entSelf` + movement-param globals.
pub fn PM_WalkMove() {
    todo!("Port PM_WalkMove — parked: pmove-working-state")
}

/// Raven `PM_DeadMove`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:3492-3509`
// PORT-ESCALATION(pmove-working-state): reads `pm`/`pml`.
pub fn PM_DeadMove() {
    todo!("Port PM_DeadMove — parked: pmove-working-state")
}

/// Raven `PM_NoclipMove`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:3517-3576`
// PORT-ESCALATION(pmove-working-state): reads `pml`, writes `pm`.
pub fn PM_NoclipMove() {
    todo!("Port PM_NoclipMove — parked: pmove-working-state")
}

/// Raven `PM_FootstepForSurface`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:3587-3594`
// PORT-ESCALATION(pmove-working-state): reads `pml`.
pub fn PM_FootstepForSurface() -> c_int {
    todo!("Port PM_FootstepForSurface — parked: pmove-working-state")
}

/// Raven `PM_TryRoll`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:3597-3681`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_TryRoll() -> c_int {
    todo!("Port PM_TryRoll — parked: pmove-working-state")
}

/// Raven `PM_CrashLandEffect`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:3684-3722`
// PORT-ESCALATION(pmove-working-state): reads `pm`/`pml`.
pub fn PM_CrashLandEffect() {
    todo!("Port PM_CrashLandEffect — parked: pmove-working-state")
}

/// Raven `PM_CrashLand`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:3731-4002`
// PORT-ESCALATION(pmove-working-state): reads `pml`/`WeaponReadyAnim`, writes `pm`.
pub fn PM_CrashLand() {
    todo!("Port PM_CrashLand — parked: pmove-working-state")
}

/// Raven `PM_CorrectAllSolid`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4009-4044`
// PORT-ESCALATION(pmove-working-state): reads `c_pmove`, writes `pm`/`pml`.
pub fn PM_CorrectAllSolid(
    trace: *mut trace_t,
) -> c_int {
    todo!("Port PM_CorrectAllSolid — parked: pmove-working-state")
}

/// Raven `PM_GroundTraceMissed`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4053-4133`
// PORT-ESCALATION(pmove-working-state): writes `pm`/`pml`.
pub fn PM_GroundTraceMissed() {
    todo!("Port PM_GroundTraceMissed — parked: pmove-working-state")
}

/// Raven `PM_GroundTrace`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4141-4277`
// PORT-ESCALATION(pmove-working-state): reads `g_entities`/`g_gametype`/`pm_entSelf`, writes `pm`/`pml`.
pub fn PM_GroundTrace() {
    todo!("Port PM_GroundTrace — parked: pmove-working-state")
}

/// Raven `PM_SetWaterLevel`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4285-4320`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_SetWaterLevel() {
    todo!("Port PM_SetWaterLevel — parked: pmove-working-state")
}

/// Raven `PM_CheckDualForwardJumpDuck`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4322-4339`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_CheckDualForwardJumpDuck() -> qboolean {
    todo!("Port PM_CheckDualForwardJumpDuck — parked: pmove-working-state")
}

/// Raven `PM_CheckFixMins`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4341-4401`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_CheckFixMins() {
    todo!("Port PM_CheckFixMins — parked: pmove-working-state")
}

/// Raven `PM_CheckDuck`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4410-4542`
// PORT-ESCALATION(pmove-working-state): reads `g_entities`/`level`/`pm_entVeh`, writes `pm`.
pub fn PM_CheckDuck() {
    todo!("Port PM_CheckDuck — parked: pmove-working-state")
}

/// Raven `PM_Use`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4559-4577`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_Use() {
    todo!("Port PM_Use — parked: pmove-working-state")
}

/// Raven `PM_WalkingAnim`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4579-4598`
// PORT-ESCALATION(animNumber_t): switches on `BOTH_*` anim constants; `animNumber_t` is unported per packet.
pub fn PM_WalkingAnim(
    anim: c_int,
) -> qboolean {
    todo!("Port PM_WalkingAnim — parked: animNumber_t")
}

/// Raven `PM_RunningAnim`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4600-4620`
// PORT-ESCALATION(animNumber_t): switches on `BOTH_*` anim constants; `animNumber_t` is unported per packet.
pub fn PM_RunningAnim(
    anim: c_int,
) -> qboolean {
    todo!("Port PM_RunningAnim — parked: animNumber_t")
}

/// Raven `PM_SwimmingAnim`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4622-4633`
// PORT-ESCALATION(animNumber_t): switches on `BOTH_*` anim constants; `animNumber_t` is unported per packet.
pub fn PM_SwimmingAnim(
    anim: c_int,
) -> qboolean {
    todo!("Port PM_SwimmingAnim — parked: animNumber_t")
}

/// Raven `PM_RollingAnim`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4635-4647`
// PORT-ESCALATION(animNumber_t): switches on `BOTH_*` anim constants; `animNumber_t` is unported per packet.
pub fn PM_RollingAnim(
    anim: c_int,
) -> qboolean {
    todo!("Port PM_RollingAnim — parked: animNumber_t")
}

/// Raven `PM_AnglesForSlope`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4649-4675`
// PORT-ESCALATION(vec3-outparam-signature): `angles` is written in place but the skeleton passes
// vec3_t ([f32;3], Copy) by value, so the result cannot propagate to the caller.
pub fn PM_AnglesForSlope(
    yaw: f32,
    slope: vec3_t,
    angles: vec3_t,
) {
    todo!("Port PM_AnglesForSlope — parked: vec3-outparam-signature")
}

/// Raven `PM_FootSlopeTrace`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4677-4740`
// PORT-ESCALATION(pmove-working-state): reads `pm` (ghoul2, trace, modelScale, mins, ...).
pub fn PM_FootSlopeTrace(
    pDiff: *mut f32,
    pInterval: *mut f32,
) {
    todo!("Port PM_FootSlopeTrace — parked: pmove-working-state")
}

/// Raven `BG_InSlopeAnim`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4742-4800`
// PORT-ESCALATION(animNumber_t): switches on `LEGS_*` anim constants; `animNumber_t` is unported per packet.
pub fn BG_InSlopeAnim(
    anim: c_int,
) -> qboolean {
    todo!("Port BG_InSlopeAnim — parked: animNumber_t")
}

/// Raven `PM_AdjustStandAnimForSlope`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4804-5102`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_AdjustStandAnimForSlope() -> qboolean {
    todo!("Port PM_AdjustStandAnimForSlope — parked: pmove-working-state")
}

/// Raven `PM_LegsSlopeBackTransition`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:5107-5168`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_LegsSlopeBackTransition(
    desiredAnim: c_int,
) -> c_int {
    todo!("Port PM_LegsSlopeBackTransition — parked: pmove-working-state")
}

/// Raven `PM_Footsteps`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:5175-5661`
// PORT-ESCALATION(pmove-working-state): reads `WeaponReadyLegsAnim`/`pm_entSelf`/`pml`, writes `pm`.
pub fn PM_Footsteps() {
    todo!("Port PM_Footsteps — parked: pmove-working-state")
}

/// Raven `PM_WaterEvents`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:5670-5748`
// PORT-ESCALATION(pmove-working-state): reads `pm`/`pml`.
pub fn PM_WaterEvents() {
    todo!("Port PM_WaterEvents — parked: pmove-working-state")
}

/// Raven `BG_ClearRocketLock`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:5750-5759`
pub fn BG_ClearRocketLock(
    ps: *mut playerState_t,
) {
    unsafe {
        if !ps.is_null() {
            (*ps).rocketLockIndex = ENTITYNUM_NONE;
            (*ps).rocketLastValidTime = 0.0;
            (*ps).rocketLockTime = -1.0;
            (*ps).rocketTargetTime = 0.0;
        }
    }
}

/// Raven `PM_BeginWeaponChange`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:5766-5793`
// PORT-ESCALATION(pmove-working-state): reads/writes `pm->ps`.
pub fn PM_BeginWeaponChange(
    weapon: c_int,
) {
    todo!("Port PM_BeginWeaponChange — parked: pmove-working-state")
}

/// Raven `PM_FinishWeaponChange`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:5801-5825`
// PORT-ESCALATION(pmove-working-state): reads/writes `pm->ps`/`pm->cmd`.
pub fn PM_FinishWeaponChange() {
    todo!("Port PM_FinishWeaponChange — parked: pmove-working-state")
}

/// Raven `BG_VehTraceFromCamPos`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:5833-5872`
// PORT-ESCALATION(engine-trap-threading): the QAGAME path calls `trap_Trace`, whose SEAM-D13
// wrapper needs an engine handle (`trap::Trace(engine, ..)`), but this C-signature fn threads none.
pub fn BG_VehTraceFromCamPos(
    camTrace: *mut trace_t,
    bgEnt: *mut bgEntity_t,
    entOrg: vec3_t,
    shotStart: vec3_t,
    end: vec3_t,
    newEnd: vec3_t,
    shotDir: vec3_t,
    bestDist: f32,
) -> c_int {
    todo!("Port BG_VehTraceFromCamPos — parked: engine-trap-threading")
}

/// Raven `PM_RocketLock`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:5874-5977`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_RocketLock(
    lockDist: f32,
    vehicleLock: qboolean,
) {
    todo!("Port PM_RocketLock — parked: pmove-working-state")
}

/// Raven `PM_DoChargedWeapons`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:5980-6233`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_DoChargedWeapons(
    vehicleRocketLock: qboolean,
    veh: *mut bgEntity_t,
) -> qboolean {
    todo!("Port PM_DoChargedWeapons — parked: pmove-working-state")
}

/// Raven `PM_ItemUsable`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:6239-6366`
// PORT-ESCALATION(pmove-working-state): reads `pm`/`bg_itemlist`.
pub fn PM_ItemUsable(
    ps: *mut playerState_t,
    forcedUse: c_int,
) -> c_int {
    todo!("Port PM_ItemUsable — parked: pmove-working-state")
}

/// Raven `PM_CanSetWeaponAnims`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:6369-6377`
// PORT-ESCALATION(pmove-working-state): reads `pm`.
pub fn PM_CanSetWeaponAnims() -> qboolean {
    todo!("Port PM_CanSetWeaponAnims — parked: pmove-working-state")
}

/// Raven `PM_VehicleWeaponAnimate`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:6381-6631`
// PORT-ESCALATION(pmove-working-state): reads `pm_entVeh`, writes `pm`.
pub fn PM_VehicleWeaponAnimate() {
    todo!("Port PM_VehicleWeaponAnimate — parked: pmove-working-state")
}

/// Raven `PM_Weapon`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:6641-7672`
// PORT-ESCALATION(pmove-working-state): reads `pm_entSelf`/`pm_entVeh`/`pml`/many tables, writes `pm`.
pub fn PM_Weapon() {
    todo!("Port PM_Weapon — parked: pmove-working-state")
}

/// Raven `PM_Animate`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:7680-7740`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_Animate() {
    todo!("Port PM_Animate — parked: pmove-working-state")
}

/// Raven `PM_DropTimers`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:7748-7773`
// PORT-ESCALATION(pmove-working-state): reads `pml`, writes `pm`.
pub fn PM_DropTimers() {
    todo!("Port PM_DropTimers — parked: pmove-working-state")
}

/// Raven `BG_UnrestrainedPitchRoll`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:7784-7798`
// PORT-ESCALATION(bg-global): reads the extern cvar `bg_fighterAltControl.integer`, which is
// not resolved in the packet (a bg-tier vmCvar global; ruling 1 places it in GameCvars, no threading here).
pub fn BG_UnrestrainedPitchRoll(
    ps: *mut playerState_t,
    pVeh: *mut Vehicle_t,
) -> qboolean {
    todo!("Port BG_UnrestrainedPitchRoll — parked: bg-global")
}

/// Raven `PM_UpdateViewAngles`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:7813-7894`
// PORT-ESCALATION(pmove-working-state): reads `pm_entVeh`.
pub fn PM_UpdateViewAngles(
    ps: *mut playerState_t,
    cmd: *const usercmd_t,
) {
    todo!("Port PM_UpdateViewAngles — parked: pmove-working-state")
}

/// Raven `PM_AdjustAttackStates`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8031-8199`
// PORT-ESCALATION(pmove-working-state): reads `pm_entSelf`/`pm_entVeh`/`weaponData` globals.
pub fn PM_AdjustAttackStates(
    pm: *mut pmove_t,
) {
    todo!("Port PM_AdjustAttackStates — parked: pmove-working-state")
}

/// Raven `BG_CmdForRoll`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8201-8327`
// PORT-ESCALATION(animNumber_t): switches on `BOTH_*` roll anim constants; `animNumber_t` is unported per packet.
pub fn BG_CmdForRoll(
    ps: *mut playerState_t,
    anim: c_int,
    pCmd: *mut usercmd_t,
) {
    todo!("Port BG_CmdForRoll — parked: animNumber_t")
}

/// Raven `BG_AdjustClientSpeed`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8331-8510`
// PORT-ESCALATION(pmove-working-state): reads `pm`/`pm_entSelf`.
pub fn BG_AdjustClientSpeed(
    ps: *mut playerState_t,
    cmd: *mut usercmd_t,
    svTime: c_int,
) {
    todo!("Port BG_AdjustClientSpeed — parked: pmove-working-state")
}

/// Raven `BG_InRollAnim`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8512-8523`
// PORT-ESCALATION(animNumber_t): switches on `BOTH_ROLL_*` anim constants; `animNumber_t` is unported per packet.
pub fn BG_InRollAnim(
    cent: *mut entityState_t,
) -> qboolean {
    todo!("Port BG_InRollAnim — parked: animNumber_t")
}

/// Raven `BG_InKnockDown`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8525-8560`
// PORT-ESCALATION(animNumber_t): switches on `BOTH_KNOCKDOWN*`/`BOTH_GETUP*` constants; `animNumber_t` unported.
pub fn BG_InKnockDown(
    anim: c_int,
) -> qboolean {
    todo!("Port BG_InKnockDown — parked: animNumber_t")
}

/// Raven `BG_InRollES`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8562-8574`
// PORT-ESCALATION(animNumber_t): switches on `BOTH_ROLL_*` anim constants; `animNumber_t` is unported per packet.
pub fn BG_InRollES(
    ps: *mut entityState_t,
    anim: c_int,
) -> qboolean {
    todo!("Port BG_InRollES — parked: animNumber_t")
}

/// Raven `BG_IK_MoveArm`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8576-8730`
// PORT-ESCALATION(bg-global): indexes the extern `bgHumanoidAnimations[basePose]` table, which is
// not resolved in the packet (a bg_panimate.c global).
pub fn BG_IK_MoveArm(
    ghoul2: *mut c_void,
    lHandBolt: c_int,
    time: c_int,
    ent: *mut entityState_t,
    basePose: c_int,
    desiredPos: vec3_t,
    ikInProgress: *mut qboolean,
    origin: vec3_t,
    angles: vec3_t,
    scale: vec3_t,
    blendTime: c_int,
    forceHalt: qboolean,
) {
    todo!("Port BG_IK_MoveArm — parked: bg-global")
}

/// Raven `BG_UpdateLookAngles`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8733-8787`
// PORT-ESCALATION(vec3-outparam-signature): `lookAngles`/`lastHeadAngles` are clamped/written in
// place, but the skeleton passes vec3_t ([f32;3], Copy) by value so the writes cannot propagate.
pub fn BG_UpdateLookAngles(
    lookingDebounceTime: c_int,
    lastHeadAngles: vec3_t,
    time: c_int,
    lookAngles: vec3_t,
    lookSpeed: f32,
    minPitch: f32,
    maxPitch: f32,
    minYaw: f32,
    maxYaw: f32,
    minRoll: f32,
    maxRoll: f32,
) {
    todo!("Port BG_UpdateLookAngles — parked: vec3-outparam-signature")
}

/// Raven `BG_G2ClientNeckAngles`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8790-8866`
// PORT-ESCALATION(vec3-outparam-signature): `headAngles`/`neckAngles`/`thoracicAngles` are written
// in place, but the skeleton passes vec3_t ([f32;3], Copy) by value so the writes cannot propagate.
pub fn BG_G2ClientNeckAngles(
    ghoul2: *mut c_void,
    time: c_int,
    lookAngles: vec3_t,
    headAngles: vec3_t,
    neckAngles: vec3_t,
    thoracicAngles: vec3_t,
    headClampMinAngles: vec3_t,
    headClampMaxAngles: vec3_t,
) {
    todo!("Port BG_G2ClientNeckAngles — parked: vec3-outparam-signature")
}

/// Raven `BG_G2ClientSpineAngles`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8869-8990`
// PORT-ESCALATION(vec3-outparam-signature): `thoracicAngles`/`ulAngles`/`llAngles`/`viewAngles` are
// written in place, but the skeleton passes vec3_t ([f32;3], Copy) by value — writes cannot propagate.
pub fn BG_G2ClientSpineAngles(
    ghoul2: *mut c_void,
    motionBolt: c_int,
    cent_lerpOrigin: vec3_t,
    cent_lerpAngles: vec3_t,
    cent: *mut entityState_t,
    time: c_int,
    viewAngles: vec3_t,
    ciLegs: c_int,
    ciTorso: c_int,
    angles: vec3_t,
    thoracicAngles: vec3_t,
    ulAngles: vec3_t,
    llAngles: vec3_t,
    modelScale: vec3_t,
    tPitchAngle: *mut f32,
    tYawAngle: *mut f32,
    corrTime: *mut c_int,
) {
    todo!("Port BG_G2ClientSpineAngles — parked: vec3-outparam-signature")
}

/// Raven `BG_SwingAngles`.
///
/// Raven: `CG_SwingAngles` — swing an angle towards a destination, modifying
/// speed by the delta and clamping to tolerance.
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8997-9053`
pub fn BG_SwingAngles(
    destination: f32,
    swingTolerance: f32,
    clampTolerance: f32,
    speed: f32,
    angle: *mut f32,
    swinging: *mut qboolean,
    frametime: c_int,
) -> f32 {
    unsafe {
        let mut swing: f32 = 0.0;
        let mut r#move: f32;
        let mut scale: f32;

        if *swinging == qfalse {
            // see if a swing should be started
            swing = AngleSubtract(*angle, destination);
            if swing > swingTolerance || swing < -swingTolerance {
                *swinging = qtrue;
            }
        }

        if *swinging == qfalse {
            return 0.0;
        }

        // modify the speed depending on the delta so it doesn't seem so linear
        swing = AngleSubtract(destination, *angle);
        scale = swing.abs();
        if scale < swingTolerance * 0.5 {
            scale = 0.5;
        } else if scale < swingTolerance {
            scale = 1.0;
        } else {
            scale = 2.0;
        }

        // swing towards the destination angle
        if swing >= 0.0 {
            r#move = frametime as f32 * scale * speed;
            if r#move >= swing {
                r#move = swing;
                *swinging = qfalse;
            }
            *angle = AngleMod(*angle + r#move);
        } else if swing < 0.0 {
            r#move = frametime as f32 * scale * -speed;
            if r#move <= swing {
                r#move = swing;
                *swinging = qfalse;
            }
            *angle = AngleMod(*angle + r#move);
        }

        // clamp to no more than tolerance
        swing = AngleSubtract(destination, *angle);
        if swing > clampTolerance {
            *angle = AngleMod(destination - (clampTolerance - 1.0));
        } else if swing < -clampTolerance {
            *angle = AngleMod(destination + (clampTolerance - 1.0));
        }

        swing
    }
}

/// Raven `BG_InRoll2`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9058-9078`
// PORT-ESCALATION(animNumber_t): switches on `BOTH_GETUP_*ROLL_*`/`BOTH_ROLL_*` constants; `animNumber_t` unported.
pub fn BG_InRoll2(
    es: *mut entityState_t,
) -> qboolean {
    todo!("Port BG_InRoll2 — parked: animNumber_t")
}

/// Raven `BG_G2PlayerAngles`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9082-9457`
// PORT-ESCALATION(vec3-outparam-signature): writes many vec3_t out-params (`legsAngles`, `turAngles`, …)
// passed by value ([f32;3], Copy); also indexes the unresolved extern `WeaponReadyAnim` table.
pub fn BG_G2PlayerAngles(
    ghoul2: *mut c_void,
    motionBolt: c_int,
    cent: *mut entityState_t,
    time: c_int,
    cent_lerpOrigin: vec3_t,
    cent_lerpAngles: vec3_t,
    legs: *mut vec3_t,
    legsAngles: vec3_t,
    tYawing: *mut qboolean,
    tPitching: *mut qboolean,
    lYawing: *mut qboolean,
    tYawAngle: *mut f32,
    tPitchAngle: *mut f32,
    lYawAngle: *mut f32,
    frametime: c_int,
    turAngles: vec3_t,
    modelScale: vec3_t,
    ciLegs: c_int,
    ciTorso: c_int,
    corrTime: *mut c_int,
    lookAngles: vec3_t,
    lastHeadAngles: vec3_t,
    lookTime: c_int,
    emplaced: *mut entityState_t,
    crazySmoothFactor: *mut c_int,
) {
    todo!("Port BG_G2PlayerAngles — parked: vec3-outparam-signature")
}

/// Raven `BG_G2ATSTAngles`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9459-9462`
pub fn BG_G2ATSTAngles(
    ghoul2: *mut c_void,
    time: c_int,
    cent_lerpAngles: vec3_t,
) {
    unsafe {
        // up = POSITIVE_X, right = NEGATIVE_Y, fwd = NEGATIVE_Z
        strap_G2API_SetBoneAngles(
            ghoul2,
            0,
            b"thoracic\0".as_ptr() as *const c_char,
            cent_lerpAngles,
            BONE_ANGLES_POSTMULT,
            POSITIVE_X as c_int,
            NEGATIVE_Y as c_int,
            NEGATIVE_Z as c_int,
            core::ptr::null_mut(),
            0,
            time,
        );
    }
}

/// Raven `PM_AdjustAnglesForDualJumpAttack`.
///
/// Raven: the pitch/yaw ucmd override is commented out in the oracle; the live
/// path unconditionally returns qtrue.
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9464-9469`
pub fn PM_AdjustAnglesForDualJumpAttack(
    ps: *mut playerState_t,
    ucmd: *mut usercmd_t,
) -> qboolean {
    qtrue
}

/// Raven `PM_CmdForSaberMoves`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9474-9639`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_CmdForSaberMoves(
    ucmd: *mut usercmd_t,
) {
    todo!("Port PM_CmdForSaberMoves — parked: pmove-working-state")
}

/// Raven `PM_VehicleViewAngles`.
///
/// Raven: constrain the rider's viewangles based on the vehicle's caps (or leave
/// a turret-operating passenger unclamped). `VEH_CONTROL_SCHEME_4` is undefined,
/// so the `#else` (BG_UnrestrainedPitchRoll) branch is the compiled one.
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9642-9713`
pub fn PM_VehicleViewAngles(
    ps: *mut playerState_t,
    veh: *mut bgEntity_t,
    ucmd: *mut usercmd_t,
) {
    unsafe {
        let pVeh: *mut Vehicle_t = (*veh).m_pVehicle;
        let mut setAngles: qboolean = qtrue;
        let mut clampMin: vec3_t = [0.0; 3];
        let mut clampMax: vec3_t = [0.0; 3];

        if !(*(*veh).m_pVehicle).m_pPilot.is_null()
            && (*(*(*veh).m_pVehicle).m_pPilot).s.number == (*ps).clientNum
        {
            // set the pilot's viewangles to the vehicle's viewangles, but only if
            // not doing special free-roll/pitch control
            if BG_UnrestrainedPitchRoll(ps, (*veh).m_pVehicle) == qfalse {
                setAngles = qtrue;
                clampMin[PITCH as usize] = -(*(*pVeh).m_pVehicleInfo).lookPitch;
                clampMax[PITCH as usize] = (*(*pVeh).m_pVehicleInfo).lookPitch;
                clampMin[YAW as usize] = 0.0;
                clampMax[YAW as usize] = 0.0;
                clampMin[ROLL as usize] = -1.0;
                clampMax[ROLL as usize] = -1.0;
            }
        } else {
            // passengers can look around freely, UNLESS they're controlling a turret!
            for i in 0..MAX_VEHICLE_TURRETS {
                if (*(*(*veh).m_pVehicle).m_pVehicleInfo).turret[i as usize].passengerNum
                    == (*ps).generic1
                {
                    // this turret is my station — don't clamp
                    return;
                }
            }
        }

        if setAngles == qtrue {
            for i in 0..3usize {
                // clamp viewangles
                if clampMin[i] == -1.0 || clampMax[i] == -1.0 {
                    // no clamp
                } else if clampMin[i] == 0.0 && clampMax[i] == 0.0 {
                    // no allowance
                } else {
                    // allowance
                    if (*ps).viewangles[i] > clampMax[i] {
                        (*ps).viewangles[i] = clampMax[i];
                    } else if (*ps).viewangles[i] < clampMin[i] {
                        (*ps).viewangles[i] = clampMin[i];
                    }
                }
            }

            PM_SetPMViewAngle(ps, (*ps).viewangles, ucmd);
        }
    }
}

/// Raven `PM_WeaponOkOnVehicle`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9745-9759`
pub fn PM_WeaponOkOnVehicle(
    weapon: c_int,
) -> qboolean {
    // FIXME (Raven): check g_vehicleInfo for our vehicle?
    if weapon == WP_MELEE as c_int
        || weapon == WP_SABER as c_int
        || weapon == WP_BLASTER as c_int
    {
        return qtrue;
    }
    qfalse
}

/// Raven `PM_GetOkWeaponForVehicle`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9762-9780`
// PORT-ESCALATION(pmove-working-state): reads `pm`.
pub fn PM_GetOkWeaponForVehicle() -> c_int {
    todo!("Port PM_GetOkWeaponForVehicle — parked: pmove-working-state")
}

/// Raven `PM_VehForcedTurning`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9783-9830`
// PORT-ESCALATION(pmove-working-state): reads `pml`, writes `pm`.
pub fn PM_VehForcedTurning(
    veh: *mut bgEntity_t,
) {
    todo!("Port PM_VehForcedTurning — parked: pmove-working-state")
}

/// Raven `PM_VehFaceHyperspacePoint`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9916-9989`
// PORT-ESCALATION(pmove-working-state): reads `pml`, writes `pm`.
pub fn PM_VehFaceHyperspacePoint(
    veh: *mut bgEntity_t,
) {
    todo!("Port PM_VehFaceHyperspacePoint — parked: pmove-working-state")
}

/// Raven `BG_VehicleAdjustBBoxForOrientation`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9993-10076`
// PORT-ESCALATION(fn-pointer-param): `localTrace` is a raw C function-pointer param whose type is
// unported (`void (*)(trace_t*, const vec_t*, const vec_t*, const vec_t*, const vec_t*, int, int)`);
// the body invokes it, so it needs the trap/dispatch shape settled first.
pub fn BG_VehicleAdjustBBoxForOrientation(
    veh: *mut Vehicle_t,
    origin: vec3_t,
    mins: vec3_t,
    maxs: vec3_t,
    clientNum: c_int,
    tracemask: c_int,
    //TODO: Port void ()(trace_t , vec_t , vec_t , vec_t , vec_t , int, int)  (C: `void (*)(trace_t *, const vec_t *, const vec_t *, const vec_t *, const vec_t *, int, int)`)
    localTrace: *mut c_void,
) {
    todo!("Port BG_VehicleAdjustBBoxForOrientation — parked: fn-pointer-param")
}

/// Raven `PM_MoveForKata`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:10092-10172`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_MoveForKata(
    ucmd: *mut usercmd_t,
) {
    todo!("Port PM_MoveForKata — parked: pmove-working-state")
}

/// Raven `PmoveSingle`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:10174-11157`
// PORT-ESCALATION(pmove-working-state): the pmove driver — sets `pm`/`pm_entSelf`/`pm_entVeh`/`pml`
// and every working-set global, then dispatches the whole move pipeline. Blocked on the working-set
// threading decision (and `trap_SnapVector`, which needs an engine handle).
pub fn PmoveSingle(
    pmove: *mut pmove_t,
) {
    todo!("Port PmoveSingle — parked: pmove-working-state")
}

/// Raven `Pmove`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:11167-11215`
// PORT-ESCALATION(pmove-working-state): the public entrypoint — loops PmoveSingle; blocked on the
// same working-set threading decision.
pub fn Pmove(
    pmove: *mut pmove_t,
) {
    todo!("Port Pmove — parked: pmove-working-state")
}
