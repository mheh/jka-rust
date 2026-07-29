// PORT-COMPLETE: bg_pmove.c
//! FAITHFUL port of `oracle/codemp/game/bg_pmove.c`.
//!
//! Raven's file-static pmove working set (`pmove_t *pm`, `pml_t pml`,
//! `bgEntity_t *pm_entSelf`, `pm_entVeh`, `pm_flying`, `gPMDoSlowFall`,
//! `pm_cancelOutZoom`) lives on the `PmoveContext` receiver (§B3 — no
//! `static mut`/hidden globals).
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::public::pmove_t::MAXTOUCH;

// Raven `qboolean` is `c_int`; keep the source spelling at assignment sites.
// Source: `oracle/codemp/game/q_shared.h`

/// Raven `#define MAX_WEAPON_CHARGE_TIME 5000`.
/// Source: `oracle/codemp/game/bg_pmove.c:16`
const MAX_WEAPON_CHARGE_TIME: c_int = 5000;
use crate::bg_panimate::{
    BG_InRoll, BG_SaberInAttack, BG_SaberInSpecial, BG_SaberLockBreakAnim, BG_SpinningSaberAnim,
    PM_CanRollFromSoulCal, PM_SaberInTransition,
};
use mp_qshared::shared::q_math::{
    _VectorAdd, _VectorCopy, _VectorMA, _VectorScale, _VectorSubtract, AngleNormalize180,
    AngleNormalize360, AnglesSubtract, AnglesToAxis, VectorClear, VectorCompare, VectorNormalize,
    VectorSet,
};
use mp_qshared::shared::q_math::{vec3_origin, AngleDelta, Distance, VectorLengthSquared};
use mp_qshared::shared::q_math::{vectoangles, AngleVectors, Q_fabs};
use mp_qshared::shared::q_math::{AngleMod, AngleSubtract};
use mp_qshared::shared::q_math::{PITCH, ROLL, YAW};
// Additional bg helpers reached by the pmove pipeline (pass-3 call surface).
use crate::bg_misc::{
    vectoyaw, BG_AddPredictableEventToPlayerstate, BG_CanUseFPNow, BG_CycleInven, BG_HasYsalamiri,
    BG_IsItemSelectable,
};
use crate::bg_panimate::{
    BG_FlippingAnim, BG_FullBodyTauntAnim, BG_InBackFlip, BG_InDeathAnim, BG_InGrappleMove,
    BG_InKataAnim, BG_InReboundHold, BG_InReboundJump, BG_InSpecialJump, BG_KickMove,
    BG_KickingAnim, BG_SaberInKata, BG_SaberInSpecialAttack, PM_InKnockDown, PM_InOnGroundAnim,
    PM_InRollComplete, PM_InSaberAnim, PM_LandingAnim, PM_PainAnim, PM_SaberInStart,
};
use crate::bg_saber::BG_ForcePowerDrain;
use crate::public::anim_number::animNumber_t;
use crate::vehicles::MIN_LANDING_SLOPE;
use mp_qshared::probe;
use mp_qshared::shared::error_parm::errorParm_t::ERR_DROP;
use mp_qshared::shared::q_math::{CrossProduct, VectorLength};
use mp_qshared::shared::shared_eik_move_state::sharedEIKMoveState::{IKS_DYNAMIC, IKS_NONE};

// Pass-3 bg state channel: the per-call working set + the
// two seam traits + the session state. `PmoveContext` replaces the file-static
// pmove working set the skeletons parked on.
use crate::bg_channel::{BgState, BgTraps, GameCallbacks, PmoveContext};
// Vehicle-type discriminants for the `PM_Friction` vehicle-friction branch.
use crate::vehicles::vehicle_type_t::vehicleType_t;

// --- `bg_pmove.c` file-scope movement parameters (globals 41-55). These are
// read-only tunables, so they stay module `const`s.
// Source: `oracle/codemp/game/bg_pmove.c:41-55`
pub const pm_stopspeed: f32 = 100.0;
pub const pm_duckScale: f32 = 0.50;
pub const pm_swimScale: f32 = 0.50;
pub const pm_vehicleaccelerate: f32 = 36.0;
pub const pm_accelerate: f32 = 10.0;
pub const pm_airaccelerate: f32 = 1.0;
pub const pm_wateraccelerate: f32 = 4.0;
pub const pm_flyaccelerate: f32 = 8.0;
pub const pm_friction: f32 = 6.0;
pub const pm_waterfriction: f32 = 1.0;
pub const pm_spectatorfriction: f32 = 5.0;

// --- `bg_pmove.c` local `FLY_*` enum (bg_pmove.c:441-444). Mirrors `pm_flying`.
// Source: `oracle/codemp/game/bg_pmove.c:441-444`
pub const FLY_NONE: c_int = 0;
pub const FLY_NORMAL: c_int = 1;
pub const FLY_VEHICLE: c_int = 2;
pub const FLY_HOVER: c_int = 3;

// --- Constants the pmove slice reads that have no central export; defined here
// per the codebase's per-file `#define` convention. Each cites its Raven
// `#define`.
//
// The const sweep removed the local shadows of `SURF_SLICK`, `MASK_WATER`,
// `PMF_STUCK_TO_WALL`, `PMF_TIME_KNOCKBACK`, `PMF_JUMP_HELD`, `BUTTON_ATTACK`
// and `BUTTON_ALT_ATTACK` — the qshared canonicals (`surface_flags`,
// `pm_flags`, `usercmd_button`) and `BONE_ANGLES_POSTMULT`
// (`ghoul2::bone_flags`) reach this file identically through
// `crate::prelude::*`.
/// `MINS_Z`. Source: `oracle/codemp/game/bg_public.h:46`
pub const MINS_Z: c_int = -24;
/// `MIN_WALK_NORMAL`. Source: `oracle/codemp/game/bg_local.h:5`
pub const MIN_WALK_NORMAL: f32 = 0.7;
/// `TIMER_LAND`. Source: `oracle/codemp/game/bg_local.h:7`
pub const TIMER_LAND: c_int = 130;
/// `USE_DELAY` — local `#define` at its single call site in `PM_Use`.
/// Source: `oracle/codemp/game/bg_pmove.c:4557`
pub const USE_DELAY: c_int = 2000;
/// `JUMP_OFF_WALL_SPEED` — local `#define` at its single call site in
/// `PM_AdjustAngleForWallJump`.
/// Source: `oracle/codemp/game/bg_pmove.c:1600`
pub const JUMP_OFF_WALL_SPEED: f32 = 200.0;
/// `SLOPE_RECALC_INT` — local `#define` at its single call site in
/// `PM_AdjustStandAnimForSlope`.
/// Source: `oracle/codemp/game/bg_pmove.c:4802`
pub const SLOPE_RECALC_INT: c_int = 100;
/// `PS_PMOVEFRAMECOUNTBITS`. Source: `oracle/codemp/game/q_shared.h:2141`
pub const PS_PMOVEFRAMECOUNTBITS: c_int = 6;

// `PM_BGEntForNum` is a `PmoveContext<'_>` method below (already filled); the stale
// free-fn stub is removed (no dead duplicate).

/// Raven `BG_SabersOff`.
///
/// Source: `oracle/codemp/game/bg_pmove.c:201-216`
pub fn BG_SabersOff(ps: *mut playerState_t) -> qboolean {
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
/// Source: `oracle/codemp/game/bg_pmove.c:218-237`
pub fn BG_KnockDownable(ps: *mut playerState_t) -> qboolean {
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
/// Source: `oracle/codemp/game/bg_pmove.c:247-259`
pub fn PM_IsRocketTrooper() -> qboolean {
    qfalse
}

impl PmoveContext<'_> {
    /// Raven `PM_GetSaberStance`.
    /// Source: `oracle/codemp/game/bg_pmove.c:261-319`
    pub fn PM_GetSaberStance(&mut self) -> c_int {
        use animNumber_t::*;
        unsafe {
            let ps = (*self.pm).ps;
            let mut anim = BOTH_STAND2 as c_int;
            let saber1 = self.callbacks.my_saber((*ps).clientNum, 0);
            let saber2 = self.callbacks.my_saber((*ps).clientNum, 1);

            if (*ps).saberEntityNum == 0 {
                //lost it
                return BOTH_STAND1 as c_int;
            }

            if BG_SabersOff(ps) != qfalse {
                return BOTH_STAND1 as c_int;
            }

            if !saber1.is_null() && (*saber1).readyAnim != -1 {
                return (*saber1).readyAnim;
            }

            if !saber2.is_null() && (*saber2).readyAnim != -1 {
                return (*saber2).readyAnim;
            }

            if !saber1.is_null() && !saber2.is_null() && (*ps).saberHolstered == 0 {
                //dual sabers, both on
                return BOTH_SABERDUAL_STANCE as c_int;
            }

            let lvl = (*ps).fd.saberAnimLevel;
            if lvl == saber_styles_t::SS_DUAL as c_int {
                anim = BOTH_SABERDUAL_STANCE as c_int;
            } else if lvl == saber_styles_t::SS_STAFF as c_int {
                anim = BOTH_SABERSTAFF_STANCE as c_int;
            } else if lvl == saber_styles_t::SS_FAST as c_int
                || lvl == saber_styles_t::SS_TAVION as c_int
            {
                anim = BOTH_SABERFAST_STANCE as c_int;
            } else if lvl == saber_styles_t::SS_STRONG as c_int {
                anim = BOTH_SABERSLOW_STANCE as c_int;
            } else {
                // SS_NONE / SS_MEDIUM / SS_DESANN / default
                anim = BOTH_STAND2 as c_int;
            }
            anim
        }
    }

    /// Raven `PM_pitch_roll_for_slope`.
    /// Source: `oracle/codemp/game/bg_pmove.c:346-439`
    // `storeAngles` out-param ported as `&mut vec3_t`; Raven's `if (storeAngles)`
    // NULL test is always-true here, so the viewangles else-branch is dead and dropped.
    pub fn PM_pitch_roll_for_slope(
        &mut self,
        forwhom: *mut bgEntity_t,
        pass_slope: vec3_t,
        storeAngles: &mut vec3_t,
    ) {
        unsafe {
            let mut slope: vec3_t = [0.0; 3];
            let mut nvf: vec3_t = [0.0; 3];
            let mut ovf: vec3_t = [0.0; 3];
            let mut ovr: vec3_t = [0.0; 3];
            let mut startspot: vec3_t = [0.0; 3];
            let mut endspot: vec3_t = [0.0; 3];
            let mut new_angles: vec3_t = [0.0, 0.0, 0.0];

            //if we don't have a slope, get one
            if VectorCompare(vec3_origin, pass_slope) {
                let mut trace: trace_t = core::mem::zeroed();

                _VectorCopy((*(*self.pm).ps).origin, &mut startspot);
                startspot[2] += (*self.pm).mins[2] + 4.0;
                _VectorCopy(startspot, &mut endspot);
                endspot[2] -= 300.0;
                let vec3_origin_local = vec3_origin;
                self.traps.trace(
                    &mut trace,
                    core::ptr::addr_of!((*(*self.pm).ps).origin) as *const vec3_t,
                    core::ptr::addr_of!(vec3_origin_local) as *const vec3_t,
                    core::ptr::addr_of!(vec3_origin_local) as *const vec3_t,
                    core::ptr::addr_of!(endspot) as *const vec3_t,
                    (*forwhom).s.number,
                    MASK_SOLID,
                );

                if trace.fraction >= 1.0 {
                    return;
                }

                if VectorCompare(vec3_origin, trace.plane.normal) {
                    return;
                }

                _VectorCopy(trace.plane.normal, &mut slope);
            } else {
                _VectorCopy(pass_slope, &mut slope);
            }

            if (*forwhom).s.NPC_class == CLASS_VEHICLE as c_int {
                //special code for vehicles
                let pVeh = (*forwhom).m_pVehicle as *mut Vehicle_t;
                let mut tempAngles: vec3_t = [0.0; 3];

                tempAngles[PITCH] = 0.0;
                tempAngles[ROLL] = 0.0;
                tempAngles[YAW] = (*(*pVeh).m_vOrientation.add(YAW));
                AngleVectors(tempAngles, Some(&mut ovf), Some(&mut ovr), None);
            } else {
                AngleVectors(
                    (*(*self.pm).ps).viewangles,
                    Some(&mut ovf),
                    Some(&mut ovr),
                    None,
                );
            }

            vectoangles(slope, &mut new_angles);
            let pitch = new_angles[PITCH] + 90.0;
            new_angles[ROLL] = 0.0;
            new_angles[PITCH] = 0.0;

            AngleVectors(new_angles, Some(&mut nvf), None, None);

            let mut r#mod = _DotProduct(nvf, ovr);
            if r#mod < 0.0 {
                r#mod = -1.0;
            } else {
                r#mod = 1.0;
            }

            let dot = _DotProduct(nvf, ovf);

            // storeAngles is always "present" (non-NULL &mut) for the live caller.
            storeAngles[PITCH] = dot * pitch;
            storeAngles[ROLL] = (1.0 - Q_fabs(dot)) * pitch * r#mod;
        }
    }

    /// Raven `PM_SetSpecialMoveValues`.
    /// Source: `oracle/codemp/game/bg_pmove.c:447-480`
    pub fn PM_SetSpecialMoveValues(&mut self) {
        unsafe {
            if (*(*self.pm).ps).clientNum < MAX_CLIENTS as c_int {
                //we know that real players aren't vehs
                self.pm_flying = FLY_NONE;
                return;
            }

            //default until we decide otherwise
            self.pm_flying = FLY_NONE;

            let pEnt = self.pm_entSelf;

            if !pEnt.is_null() {
                if (*(*self.pm).ps).eFlags2 & EF2_FLYING != 0 {
                    self.pm_flying = FLY_NORMAL;
                } else if (*pEnt).s.NPC_class == CLASS_VEHICLE as c_int {
                    let pv = (*pEnt).m_pVehicle as *mut Vehicle_t;
                    if (*(*pv).m_pVehicleInfo).r#type as c_int == vehicleType_t::VH_FIGHTER as c_int
                    {
                        self.pm_flying = FLY_VEHICLE;
                    } else if (*(*pv).m_pVehicleInfo).hoverHeight > 0.0 {
                        self.pm_flying = FLY_HOVER;
                    }
                }
            }
        }
    }

    /// Raven `PM_SetVehicleAngles`.
    /// Source: `oracle/codemp/game/bg_pmove.c:482-635`
    // The Raven C parameter is `vec3_t normal` (a `float*`), and the body branches
    // on `else if (normal)` vs `else` (NULL == in air). Ported as `Option<vec3_t>`
    // so the NULL test is faithful: `Some` = valid ground surface, `None` = in air.
    pub fn PM_SetVehicleAngles(&mut self, normal: Option<vec3_t>) {
        unsafe {
            let pEnt = self.pm_entSelf;
            if pEnt.is_null() || (*pEnt).s.NPC_class != CLASS_VEHICLE as c_int {
                return;
            }

            let pVeh = (*pEnt).m_pVehicle as *mut Vehicle_t;
            let info = (*pVeh).m_pVehicleInfo;
            let mut vAngles: vec3_t = [0.0; 3];
            let pitchBias: f32;

            let mut vehicleBankingSpeed = ((*info).bankingSpeed * 32.0) * self.pml.frametime;

            if vehicleBankingSpeed <= 0.0 || ((*info).pitchLimit == 0.0 && (*info).rollLimit == 0.0)
            {
                //don't bother, this vehicle doesn't bank
                return;
            }

            if (*info).r#type as c_int == vehicleType_t::VH_FIGHTER as c_int {
                pitchBias = 0.0;
            } else {
                pitchBias = 90.0 * (*info).centerOfGravity[0];
            }

            VectorClear(&mut vAngles);
            if (*self.pm).waterlevel > 0 {
                //in water
                vAngles[PITCH] = (vAngles[PITCH] as f64
                    + ((((*(*self.pm).ps).viewangles[PITCH] - vAngles[PITCH]) * 0.75) as f64
                        + pitchBias as f64 * 0.5)) as f32;
            } else if let Some(normal) = normal {
                //have a valid surface below me
                self.PM_pitch_roll_for_slope(pEnt, normal, &mut vAngles);
                if self.pml.groundTrace.contents & (CONTENTS_WATER | CONTENTS_SLIME | CONTENTS_LAVA)
                    != 0
                {
                    //on water
                    vAngles[PITCH] += ((*(*self.pm).ps).viewangles[PITCH] - vAngles[PITCH]) * 0.5
                        + (pitchBias * 0.5);
                }
            } else {
                //in air, let pitch match view...?
                vAngles[PITCH] = (*(*self.pm).ps).viewangles[PITCH] * 0.5 + pitchBias;
                //don't bank so fast when in the air
                vehicleBankingSpeed *= 0.125 * self.pml.frametime;
            }

            //NOTE: if angles are flat and we're moving through air (not on ground), then pitch/bank?
            if (*info).rollLimit > 0.0 {
                //roll when banking
                let mut velocity: vec3_t = [0.0; 3];
                _VectorCopy((*(*self.pm).ps).velocity, &mut velocity);
                velocity[2] = 0.0;
                let mut speed = VectorNormalize(&mut velocity);
                if speed > 32.0 || speed < -32.0 {
                    let mut rt: vec3_t = [0.0; 3];
                    let mut tempVAngles: vec3_t = [0.0; 3];

                    // modulate the speed by a sine wave
                    speed =
                        (speed as f64 * ((150.0 + self.pml.frametime) as f64 * 0.003).sin()) as f32;

                    if speed > 60.0 {
                        speed = 60.0;
                    }

                    _VectorCopy(*(*pVeh).m_vOrientation.cast::<vec3_t>(), &mut tempVAngles);
                    tempVAngles[ROLL] = 0.0;
                    AngleVectors(tempVAngles, None, Some(&mut rt), None);
                    let dp = _DotProduct(velocity, rt);
                    let side = speed * dp;
                    vAngles[ROLL] -= side;
                }
            }

            //cap
            if (*info).pitchLimit != -1.0 {
                if vAngles[PITCH] > (*info).pitchLimit {
                    vAngles[PITCH] = (*info).pitchLimit;
                } else if vAngles[PITCH] < -(*info).pitchLimit {
                    vAngles[PITCH] = -(*info).pitchLimit;
                }
            }

            if vAngles[ROLL] > (*info).rollLimit {
                vAngles[ROLL] = (*info).rollLimit;
            } else if vAngles[ROLL] < -(*info).rollLimit {
                vAngles[ROLL] = -(*info).rollLimit;
            }

            //do it
            for i in 0..3usize {
                if i == YAW {
                    //yawing done elsewhere
                    continue;
                }
                {
                    if (*(*pVeh).m_vOrientation.add(i)) >= vAngles[i] + vehicleBankingSpeed {
                        (*(*pVeh).m_vOrientation.add(i)) -= vehicleBankingSpeed;
                    } else if (*(*pVeh).m_vOrientation.add(i)) <= vAngles[i] - vehicleBankingSpeed {
                        (*(*pVeh).m_vOrientation.add(i)) += vehicleBankingSpeed;
                    } else {
                        (*(*pVeh).m_vOrientation.add(i)) = vAngles[i];
                    }
                }
            }
            let _ = &mut vehicleBankingSpeed;
        }
    }
}

/// Raven `BG_ExternThisSoICanRecompileInDebug`.
///
/// Raven: the entire body is commented out in the oracle (a debug-recompile
/// hook); it is a no-op.
/// Source: `oracle/codemp/game/bg_pmove.c:641-674`
pub fn BG_ExternThisSoICanRecompileInDebug(pVeh: *mut Vehicle_t, riderPS: *mut playerState_t) {
    // No-op: the oracle body is entirely `/* ... */`-commented.
}

/// Raven `BG_VehicleTurnRateForSpeed`.
///
/// Source: `oracle/codemp/game/bg_pmove.c:676-706`
pub fn BG_VehicleTurnRateForSpeed(
    pVeh: *mut Vehicle_t,
    speed: f32,
    mPitchOverride: *mut f32,
    mYawOverride: *mut f32,
) {
    unsafe {
        if !pVeh.is_null() && !(*pVeh).m_pVehicleInfo.is_null() {
            let info = (*pVeh).m_pVehicleInfo;
            let mut speedFrac: f32 = 1.0;
            if (*info).speedDependantTurning != 0 {
                if (*pVeh).m_LandTrace.fraction >= 1.0
                    || (*pVeh).m_LandTrace.plane.normal[2] < MIN_LANDING_SLOPE
                {
                    speedFrac = speed / ((*info).speedMax * 0.75);
                    if speedFrac < 0.25 {
                        speedFrac = 0.25;
                    } else if speedFrac > 1.0 {
                        speedFrac = 1.0;
                    }
                }
            }
            if (*info).mousePitch != 0.0 {
                *mPitchOverride = (*info).mousePitch * speedFrac;
            }
            if (*info).mouseYaw != 0.0 {
                *mYawOverride = (*info).mouseYaw * speedFrac;
            }
        }
    }
}

impl PmoveContext<'_> {
    /// Raven `PM_HoverTrace`.
    /// Source: `oracle/codemp/game/bg_pmove.c:719-901`
    pub fn PM_HoverTrace(&mut self) {
        unsafe {
            let pEnt = self.pm_entSelf;
            if pEnt.is_null() || (*pEnt).s.NPC_class != CLASS_VEHICLE as c_int {
                return;
            }

            let pVeh = (*pEnt).m_pVehicle as *mut Vehicle_t;
            let info = (*pVeh).m_pVehicleInfo;
            let hoverHeight = (*info).hoverHeight;
            let trace = core::ptr::addr_of_mut!(self.pml.groundTrace);

            let mut point: vec3_t = [0.0; 3];
            let mut vAng: vec3_t = [0.0; 3];
            let mut fxAxis: [vec3_t; 3] = [[0.0; 3]; 3];

            self.pml.groundPlane = qfalse;

            let relativeWaterLevel = (*self.pm).waterlevel as f32;
            if (*self.pm).waterlevel != 0 && relativeWaterLevel >= 0.0 {
                //in water
                if (*info).bouyancy <= 0.0 {
                    //sink like a rock
                } else {
                    //rise up
                    let floatHeight = ((*info).bouyancy
                        * (((*self.pm).maxs[2] - (*self.pm).mins[2]) * 0.5))
                        - (hoverHeight * 0.5);
                    if relativeWaterLevel > floatHeight {
                        //too low, should rise up
                        (*(*self.pm).ps).velocity[2] +=
                            (relativeWaterLevel - floatHeight) * (*pVeh).m_fTimeModifier;
                    }
                }
                if (*self.pm).waterlevel <= 1 {
                    //part of us is sticking out of water
                    if ((*(*self.pm).ps).velocity[0] as f64).abs()
                        + ((*(*self.pm).ps).velocity[1] as f64).abs()
                        > 100.0
                    {
                        //moving at a decent speed
                        if self.bg.rng.Q_irand(self.pml.frametime as c_int, 100) >= 50 {
                            //splash
                            let mut wakeOrg: vec3_t = [0.0; 3];

                            vAng[PITCH] = 0.0;
                            vAng[ROLL] = 0.0;
                            vAng[YAW] = (*(*pVeh).m_vOrientation.add(YAW));
                            {
                                let (fx01, fx2) = fxAxis.split_at_mut(2);
                                let (fx0, fx1) = fx01.split_at_mut(1);
                                AngleVectors(
                                    vAng,
                                    Some(&mut fx2[0]),
                                    Some(&mut fx1[0]),
                                    Some(&mut fx0[0]),
                                );
                            }
                            _VectorCopy((*(*self.pm).ps).origin, &mut wakeOrg);
                            if (*self.pm).waterlevel >= 2 {
                                wakeOrg[2] = (*(*self.pm).ps).origin[2] + 16.0;
                            } else {
                                wakeOrg[2] = (*(*self.pm).ps).origin[2];
                            }
                            // QAGAME: tempent use bad!
                            if (*info).iWakeFX != 0 {
                                self.callbacks.add_event(
                                    (*pEnt).s.number,
                                    EV_PLAY_EFFECT_ID as c_int,
                                    (*info).iWakeFX,
                                );
                            }
                        }
                    }
                }
            } else {
                let mut minNormal = MIN_WALK_NORMAL;
                minNormal = (*info).maxSlope;

                point[0] = (*(*self.pm).ps).origin[0];
                point[1] = (*(*self.pm).ps).origin[1];
                point[2] = (*(*self.pm).ps).origin[2] - hoverHeight;

                let mut traceContents = (*self.pm).tracemask;
                if (*info).bouyancy >= 2.0 {
                    //sit on water
                    traceContents |= CONTENTS_WATER | CONTENTS_SLIME | CONTENTS_LAVA;
                }
                self.traps.trace(
                    trace,
                    core::ptr::addr_of!((*(*self.pm).ps).origin) as *const vec3_t,
                    core::ptr::addr_of!((*self.pm).mins) as *const vec3_t,
                    core::ptr::addr_of!((*self.pm).maxs) as *const vec3_t,
                    core::ptr::addr_of!(point) as *const vec3_t,
                    (*(*self.pm).ps).clientNum,
                    traceContents,
                );
                if (*trace).plane.normal[0] > 0.5
                    || (*trace).plane.normal[0] < -0.5
                    || (*trace).plane.normal[1] > 0.5
                    || (*trace).plane.normal[1] < -0.5
                {
                    //steep slanted hill, don't go up it.
                    let mut d = Q_fabs((*trace).plane.normal[0]);
                    let e = Q_fabs((*trace).plane.normal[1]);
                    if e > d {
                        d = e;
                    }
                    (*(*self.pm).ps).velocity[2] = -300.0 * d;
                } else if (*trace).plane.normal[2] >= minNormal {
                    //not a steep slope, so push us up
                    if (*trace).fraction < 1.0 {
                        //push up off ground
                        let hoverForce = (*info).hoverStrength;
                        if (*trace).fraction > 0.5 {
                            (*(*self.pm).ps).velocity[2] +=
                                (1.0 - (*trace).fraction) * hoverForce * (*pVeh).m_fTimeModifier;
                        } else {
                            (*(*self.pm).ps).velocity[2] += (0.5
                                - ((*trace).fraction * (*trace).fraction))
                                * hoverForce
                                * 2.0
                                * (*pVeh).m_fTimeModifier;
                        }
                        if (*trace).contents & (CONTENTS_WATER | CONTENTS_SLIME | CONTENTS_LAVA)
                            != 0
                        {
                            //hovering on water, make a spash if moving
                            if ((*(*self.pm).ps).velocity[0] as f64).abs()
                                + ((*(*self.pm).ps).velocity[1] as f64).abs()
                                > 100.0
                            {
                                //moving at a decent speed
                                if self.bg.rng.Q_irand(self.pml.frametime as c_int, 100) >= 50 {
                                    //splash
                                    vAng[PITCH] = 0.0;
                                    vAng[ROLL] = 0.0;
                                    vAng[YAW] = (*(*pVeh).m_vOrientation.add(YAW));
                                    {
                                        let (fx01, fx2) = fxAxis.split_at_mut(2);
                                        let (fx0, fx1) = fx01.split_at_mut(1);
                                        AngleVectors(
                                            vAng,
                                            Some(&mut fx2[0]),
                                            Some(&mut fx1[0]),
                                            Some(&mut fx0[0]),
                                        );
                                    }
                                    if (*info).iWakeFX != 0 {
                                        self.callbacks.play_effect_id(
                                            (*info).iWakeFX,
                                            core::ptr::addr_of!((*trace).endpos),
                                            core::ptr::addr_of!(fxAxis[0]),
                                        );
                                    }
                                }
                            }
                        }
                        self.pml.groundPlane = qtrue;
                    }
                }
                let _ = &mut minNormal;
            }
            if self.pml.groundPlane != qfalse {
                let n = self.pml.groundTrace.plane.normal;
                self.PM_SetVehicleAngles(Some(n));
                // We're on the ground.
                (*pVeh).m_ulFlags &= !(VEH_FLYING as u64);

                (*pVeh).m_vAngularVelocity = 0.0;
            } else {
                // NULL call: flying-in-air (Raven passes NULL for `normal`).
                self.PM_SetVehicleAngles(None);
                // We're flying in the air.
                (*pVeh).m_ulFlags |= VEH_FLYING as u64;

                if (*pVeh).m_vAngularVelocity == 0.0 {
                    (*pVeh).m_vAngularVelocity =
                        (*(*pVeh).m_vOrientation.add(YAW)) - (*pVeh).m_vPrevOrientation[YAW];
                    if (*pVeh).m_vAngularVelocity < -15.0 {
                        (*pVeh).m_vAngularVelocity = -15.0;
                    }
                    if (*pVeh).m_vAngularVelocity > 15.0 {
                        (*pVeh).m_vAngularVelocity = 15.0;
                    }
                }
                if (*pVeh).m_vAngularVelocity > 0.0 {
                    (*pVeh).m_vAngularVelocity -= self.pml.frametime;
                    if (*pVeh).m_vAngularVelocity < 0.0 {
                        (*pVeh).m_vAngularVelocity = 0.0;
                    }
                } else if (*pVeh).m_vAngularVelocity < 0.0 {
                    (*pVeh).m_vAngularVelocity += self.pml.frametime;
                    if (*pVeh).m_vAngularVelocity > 0.0 {
                        (*pVeh).m_vAngularVelocity = 0.0;
                    }
                }
            }
            self.PM_GroundTraceMissed();
        }
    }

    /// Raven `PM_AddEvent`.
    /// Source: `oracle/codemp/game/bg_pmove.c:910-912`
    pub fn PM_AddEvent(&mut self, newEvent: c_int) {
        unsafe {
            BG_AddPredictableEventToPlayerstate(newEvent, 0, (*self.pm).ps);
        }
    }

    /// Raven `PM_AddEventWithParm`.
    /// Source: `oracle/codemp/game/bg_pmove.c:914-917`
    pub fn PM_AddEventWithParm(&mut self, newEvent: c_int, parm: c_int) {
        unsafe {
            BG_AddPredictableEventToPlayerstate(newEvent, parm, (*self.pm).ps);
        }
    }

    /// Raven `PM_AddTouchEnt`.
    /// Source: `oracle/codemp/game/bg_pmove.c:924-944`
    pub fn PM_AddTouchEnt(&mut self, entityNum: c_int) {
        unsafe {
            let pm = self.pm;
            if entityNum == ENTITYNUM_WORLD {
                return;
            }
            if (*pm).numtouch == MAXTOUCH as c_int {
                return;
            }

            // see if it is already added
            for i in 0..(*pm).numtouch {
                if (*pm).touchents[i as usize] == entityNum {
                    return;
                }
            }

            // add it
            (*pm).touchents[(*pm).numtouch as usize] = entityNum;
            (*pm).numtouch += 1;
        }
    }
}

// The pmove pipeline as `PmoveContext` methods. Each was
// a no-arg C function reaching the file-static working set; the set now lives in
// `self` (`self.pm`/`self.pml`/`self.pm_entSelf`/… + `self.bg`/`self.traps`).
// The `unsafe` that dereferences the faithful `pm`/entity pointers is confined
// to these bodies (porting-rules §D11).
impl PmoveContext<'_> {
    /// Raven `PM_BGEntForNum` — the faithful `baseEnt`/`entSize` head-overlay.
    /// Returns the `bgEntity_t` at index `num` in the base array
    /// the engine handed us. Raven's `assert`s become defensive null/zero
    /// returns (out-of-pmove calls / unset base are the UB cases §19 covers).
    /// Source: `oracle/codemp/game/bg_pmove.c:172-199`
    pub fn PM_BGEntForNum(&self, num: c_int) -> *mut bgEntity_t {
        unsafe {
            if self.pm.is_null() {
                // "You cannot call PM_BGEntForNum outside of pm functions!"
                return core::ptr::null_mut();
            }
            let pm = &*self.pm;
            if pm.baseEnt.is_null() {
                // "Base entity address not set"
                return core::ptr::null_mut();
            }
            if pm.entSize == 0 {
                // "sizeof(ent) is 0, impossible (not set?)"
                return core::ptr::null_mut();
            }
            debug_assert!(num >= 0 && num < MAX_GENTITIES as c_int);
            // ent = (bgEntity_t *)((byte *)pm->baseEnt + pm->entSize*(num));
            (pm.baseEnt as *mut byte).offset((pm.entSize * num) as isize) as *mut bgEntity_t
        }
    }

    /// Raven `PM_ClipVelocity` — slide `in` off the impacting surface `normal`
    /// into `out` (§C7 out-param shape; `in`/`normal` by value permit the
    /// `PM_ClipVelocity(pml.forward, …, pml.forward, …)` self-aliasing callers).
    /// Source: `oracle/codemp/game/bg_pmove.c:954-988`
    pub fn PM_ClipVelocity(&self, r#in: vec3_t, normal: vec3_t, out: &mut vec3_t, overbounce: f32) {
        unsafe {
            let ps = &*(*self.pm).ps;
            if ps.pm_flags & PMF_STUCK_TO_WALL != 0 {
                // no sliding!
                *out = r#in; // VectorCopy( in, out )
                return;
            }
            let oldInZ = r#in[2];

            let mut backoff = _DotProduct(r#in, normal);

            if backoff < 0.0 {
                backoff *= overbounce;
            } else {
                backoff /= overbounce;
            }

            for i in 0..3 {
                let change = normal[i] * backoff;
                out[i] = r#in[i] - change;
            }
            if (*self.pm).stepSlideFix != 0
                && ps.clientNum < MAX_CLIENTS as c_int// normal player
                && ps.groundEntityNum != ENTITYNUM_NONE // on the ground
                && normal[2] < MIN_WALK_NORMAL
            {
                // sliding against a steep slope: don't slide up slopes too steep to walk on
                out[2] = oldInZ;
            }
        }
    }

    /// Raven `PM_Friction` — ground + water friction on `pm->ps->velocity`.
    /// Source: `oracle/codemp/game/bg_pmove.c:998-1123`
    pub fn PM_Friction(&mut self) {
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;

            // vec = velocity, ignoring slope movement while walking.
            let mut vec = (*ps).velocity;
            if self.pml.walking != 0 {
                vec[2] = 0.0;
            }

            // speed = VectorLength(vec);
            // sqrt is the double libm call rounded back to float; an f32 sqrt
            // double-rounds and diverges from the oracle.
            let speed = VectorLength(vec);
            if speed < 1.0 {
                (*ps).velocity[0] = 0.0;
                (*ps).velocity[1] = 0.0; // allow sinking underwater
                if (*ps).pm_type == PM_SPECTATOR as c_int {
                    (*ps).velocity[2] = 0.0;
                }
                return;
            }

            let mut drop: f32 = 0.0;

            let mut pEnt: *mut bgEntity_t = core::ptr::null_mut();
            if (*ps).clientNum >= MAX_CLIENTS as c_int {
                pEnt = self.pm_entSelf;
            }

            // apply ground friction, even if on ladder
            if self.pm_flying != FLY_VEHICLE
                && !pEnt.is_null()
                && (*pEnt).s.NPC_class == CLASS_VEHICLE as c_int
                && !((*pEnt).m_pVehicle as *mut Vehicle_t).is_null()
                && (*(*((*pEnt).m_pVehicle as *mut Vehicle_t)).m_pVehicleInfo).r#type as c_int
                    != vehicleType_t::VH_ANIMAL as c_int
                && (*(*((*pEnt).m_pVehicle as *mut Vehicle_t)).m_pVehicleInfo).r#type as c_int
                    != vehicleType_t::VH_WALKER as c_int
                && (*(*((*pEnt).m_pVehicle as *mut Vehicle_t)).m_pVehicleInfo).friction != 0.0
            {
                let friction = (*(*((*pEnt).m_pVehicle as *mut Vehicle_t)).m_pVehicleInfo).friction;
                if (*ps).pm_flags & PMF_TIME_KNOCKBACK == 0 {
                    let control = if speed < pm_stopspeed {
                        pm_stopspeed
                    } else {
                        speed
                    };
                    drop += control * friction * self.pml.frametime;
                }
            } else if self.pm_flying != FLY_NORMAL && self.pm_flying != FLY_VEHICLE {
                // apply ground friction
                if (*pm).waterlevel <= 1
                    && self.pml.walking != 0
                    && self.pml.groundTrace.surfaceFlags & SURF_SLICK == 0
                    && (*ps).pm_flags & PMF_TIME_KNOCKBACK == 0
                {
                    // if getting knocked back, no friction
                    let control = if speed < pm_stopspeed {
                        pm_stopspeed
                    } else {
                        speed
                    };
                    drop += control * pm_friction * self.pml.frametime;
                }
            }

            if self.pm_flying == FLY_VEHICLE && (*ps).pm_flags & PMF_TIME_KNOCKBACK == 0 {
                let control = speed;
                drop += control * pm_friction * self.pml.frametime;
            }

            // apply water friction even if just wading
            if (*pm).waterlevel != 0 {
                drop += speed * pm_waterfriction * (*pm).waterlevel as f32 * self.pml.frametime;
            }
            // If on a client then there is no friction
            else if (*ps).groundEntityNum < MAX_CLIENTS as c_int {
                drop = 0.0;
            }

            if (*ps).pm_type == PM_SPECTATOR as c_int || (*ps).pm_type == PM_FLOAT as c_int {
                if (*ps).pm_type == PM_FLOAT as c_int {
                    // almost no friction while floating (Raven's `0.1` is a
                    // `double` literal; compute in f64 to preserve parity).
                    drop = (drop as f64 + speed as f64 * 0.1 * self.pml.frametime as f64) as f32;
                } else {
                    drop += speed * pm_spectatorfriction * self.pml.frametime;
                }
            }

            // scale the velocity
            let mut newspeed = speed - drop;
            if newspeed < 0.0 {
                newspeed = 0.0;
            }
            newspeed /= speed;

            (*ps).velocity[0] *= newspeed;
            (*ps).velocity[1] *= newspeed;
            (*ps).velocity[2] *= newspeed;
        }
    }

    /// Raven `PM_SetWaterLevel` — set `pm->waterlevel`/`watertype` by sampling
    /// point contents at three heights (accounting for ducking). Exercises the
    /// `BgTraps::pointcontents` seam.
    /// Source: `oracle/codemp/game/bg_pmove.c:4285-4320`
    pub fn PM_SetWaterLevel(&mut self) {
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;

            // get waterlevel, accounting for ducking
            (*pm).waterlevel = 0;
            (*pm).watertype = 0;

            let mut point: vec3_t = [
                (*ps).origin[0],
                (*ps).origin[1],
                (*ps).origin[2] + MINS_Z as f32 + 1.0,
            ];
            let mut cont = self
                .traps
                .pointcontents(core::ptr::addr_of!(point), (*ps).clientNum);

            if cont & MASK_WATER != 0 {
                let sample2 = (*ps).viewheight - MINS_Z;
                let sample1 = sample2 / 2;

                (*pm).watertype = cont;
                (*pm).waterlevel = 1;
                point[2] = (*ps).origin[2] + MINS_Z as f32 + sample1 as f32;
                cont = self
                    .traps
                    .pointcontents(core::ptr::addr_of!(point), (*ps).clientNum);
                if cont & MASK_WATER != 0 {
                    (*pm).waterlevel = 2;
                    point[2] = (*ps).origin[2] + MINS_Z as f32 + sample2 as f32;
                    cont = self
                        .traps
                        .pointcontents(core::ptr::addr_of!(point), (*ps).clientNum);
                    if cont & MASK_WATER != 0 {
                        (*pm).waterlevel = 3;
                    }
                }
            }
        }
    }

    /// Raven `PM_DoSlowFall`.
    /// Source: oracle/codemp/game/bg_pmove.c:321-329
    pub fn PM_DoSlowFall(&mut self) -> qboolean {
        use animNumber_t::*;
        unsafe {
            let ps = (*self.pm).ps;
            if ((*ps).legsAnim == BOTH_WALL_RUN_RIGHT as c_int
                || (*ps).legsAnim == BOTH_WALL_RUN_LEFT as c_int)
                && (*ps).legsTimer > 500
            {
                return qtrue;
            }
            qfalse
        }
    }

    /// Raven `PmoveSingle` — one fixed-timestep move (`QAGAME` build: game-side
    /// `#ifdef QAGAME` branches compiled, cgame `#else` branches dropped). The
    /// game-tier vehicle-NPC virtual dispatch
    /// (`m_pVehicleInfo->Update`/`Animate`/`UpdateRider`/`AttachRiders`) crosses
    /// the seam via the `GameCallbacks` upcalls (`update_vehicle`/
    /// `pm_animate_vehicle`/`update_rider`/`attach_riders`, by entity number).
    /// Source: `oracle/codemp/game/bg_pmove.c:10174-11157`
    pub fn PmoveSingle(&mut self, pmove: *mut pmove_t) {
        unsafe {
            self.pm = pmove;
            let pm = self.pm;

            if (*(*pm).ps).emplacedIndex != 0 && (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                // hackerrific.
                (*pm).cmd.buttons &= !BUTTON_ALT_ATTACK;
                (*pm).cmd.buttons |= BUTTON_ATTACK;
            }

            // set up these "global" bg ents
            self.pm_entSelf = self.PM_BGEntForNum((*(*pm).ps).clientNum);
            if (*(*pm).ps).m_iVehicleNum != 0 {
                if (*(*pm).ps).clientNum < MAX_CLIENTS as c_int {
                    // player riding vehicle
                    self.pm_entVeh = self.PM_BGEntForNum((*(*pm).ps).m_iVehicleNum);
                } else {
                    // vehicle with player pilot
                    self.pm_entVeh = self.PM_BGEntForNum((*(*pm).ps).m_iVehicleNum - 1);
                }
            } else {
                // no vehicle ent
                self.pm_entVeh = core::ptr::null_mut();
            }

            self.gPMDoSlowFall = self.PM_DoSlowFall();

            // this counter lets us debug movement problems with a journal
            self.bg.c_pmove += 1;

            // clear results
            (*pm).numtouch = 0;
            (*pm).watertype = 0;
            (*pm).waterlevel = 0;

            if PM_IsRocketTrooper() != 0 {
                // don't let a nonhumanoid (probably a rockettrooper) crouch
                if (*pm).cmd.upmove < 0 {
                    (*pm).cmd.upmove = 0;
                }
            }

            // Raven `#define JETPACK_HOVER_HEIGHT 64` (bg_pmove.c:10088).
            const JETPACK_HOVER_HEIGHT: c_int = 64;

            let ps = (*pm).ps;
            let mut stiffenedUp: qboolean = qfalse;
            let mut gDist: f32 = 0.0;
            let mut noAnimate: qboolean = qfalse;
            let mut savedGravity: c_int = 0;

            if (*ps).pm_type == PM_FLOAT as c_int {
                // You get no control over where you go in grip movement
                stiffenedUp = qtrue;
            } else if (*ps).eFlags & EF_DISINTEGRATION != 0 {
                stiffenedUp = qtrue;
            } else if BG_SaberLockBreakAnim((*ps).legsAnim) != qfalse
                || BG_SaberLockBreakAnim((*ps).torsoAnim) != qfalse
                || (*ps).saberLockTime >= (*pm).cmd.serverTime
            {
                // can't move or turn
                stiffenedUp = qtrue;
                PM_SetPMViewAngle(ps, (*ps).viewangles, &mut (*pm).cmd);
            } else if (*ps).saberMove == LS_A_BACK
                || (*ps).saberMove == LS_A_BACK_CR
                || (*ps).saberMove == LS_A_BACKSTAB
                || (*ps).saberMove == LS_A_FLIP_STAB
                || (*ps).saberMove == LS_A_FLIP_SLASH
                || (*ps).saberMove == LS_A_JUMP_T__B_
                || (*ps).saberMove == LS_DUAL_LR
                || (*ps).saberMove == LS_DUAL_FB
            {
                if (*ps).legsAnim == BOTH_JUMPFLIPSTABDOWN as c_int
                    || (*ps).legsAnim == BOTH_JUMPFLIPSLASHDOWN1 as c_int
                {
                    // flipover medium stance attack
                    if (*ps).legsTimer < 1600 && (*ps).legsTimer > 900 {
                        (*ps).viewangles[YAW] += self.pml.frametime * 240.0;
                        PM_SetPMViewAngle(ps, (*ps).viewangles, &mut (*pm).cmd);
                    }
                }
                stiffenedUp = qtrue;
            } else if (*ps).legsAnim == BOTH_A2_STABBACK1 as c_int
                || (*ps).legsAnim == BOTH_ATTACK_BACK as c_int
                || (*ps).legsAnim == BOTH_CROUCHATTACKBACK1 as c_int
                || (*ps).legsAnim == BOTH_FORCELEAP2_T__B_ as c_int
                || (*ps).legsAnim == BOTH_JUMPFLIPSTABDOWN as c_int
                || (*ps).legsAnim == BOTH_JUMPFLIPSLASHDOWN1 as c_int
            {
                stiffenedUp = qtrue;
            } else if (*ps).legsAnim == BOTH_ROLL_STAB as c_int {
                stiffenedUp = qtrue;
                PM_SetPMViewAngle(ps, (*ps).viewangles, &mut (*pm).cmd);
            } else if (*ps).heldByClient != 0 {
                stiffenedUp = qtrue;
            } else if BG_KickMove((*ps).saberMove) != qfalse
                || BG_KickingAnim((*ps).legsAnim) != qfalse
            {
                stiffenedUp = qtrue;
            } else if BG_InGrappleMove((*ps).torsoAnim) != 0 {
                stiffenedUp = qtrue;
                PM_SetPMViewAngle(ps, (*ps).viewangles, &mut (*pm).cmd);
            } else if (*ps).saberMove == LS_STABDOWN_DUAL
                || (*ps).saberMove == LS_STABDOWN_STAFF
                || (*ps).saberMove == LS_STABDOWN
            {
                // FIXME (Raven): need to only move forward until we bump into our target...?
                if (*ps).legsTimer < 800 {
                    // freeze movement near end of anim
                    stiffenedUp = qtrue;
                    PM_SetPMViewAngle(ps, (*ps).viewangles, &mut (*pm).cmd);
                } else {
                    // force forward til then
                    (*pm).cmd.rightmove = 0;
                    (*pm).cmd.upmove = 0;
                    (*pm).cmd.forwardmove = 64;
                }
            } else if (*ps).saberMove == LS_PULL_ATTACK_STAB
                || (*ps).saberMove == LS_PULL_ATTACK_SWING
            {
                stiffenedUp = qtrue;
            } else if BG_SaberInKata((*ps).saberMove) != qfalse
                || BG_InKataAnim((*ps).torsoAnim) != qfalse
                || BG_InKataAnim((*ps).legsAnim) != qfalse
            {
                self.PM_MoveForKata(&mut (*pm).cmd);
            } else if BG_FullBodyTauntAnim((*ps).legsAnim) != qfalse
                && BG_FullBodyTauntAnim((*ps).torsoAnim) != qfalse
            {
                if (*pm).cmd.buttons & BUTTON_ATTACK != 0
                    || (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0
                    || (*pm).cmd.buttons & BUTTON_FORCEPOWER != 0
                    || (*pm).cmd.buttons & BUTTON_FORCEGRIP != 0
                    || (*pm).cmd.buttons & BUTTON_FORCE_LIGHTNING != 0
                    || (*pm).cmd.buttons & BUTTON_FORCE_DRAIN != 0
                    || (*pm).cmd.upmove != 0
                {
                    // stop the anim
                    if (*ps).legsAnim == BOTH_MEDITATE as c_int
                        && (*ps).torsoAnim == BOTH_MEDITATE as c_int
                    {
                        self.PM_SetAnim(
                            SETANIM_BOTH,
                            BOTH_MEDITATE_END as c_int,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            0,
                        );
                    } else {
                        (*ps).legsTimer = 0;
                        (*ps).torsoTimer = 0;
                    }
                    if (*ps).forceHandExtend == HANDEXTEND_TAUNT as c_int {
                        (*ps).forceHandExtend = 0;
                    }
                } else {
                    if (*ps).legsAnim == BOTH_MEDITATE as c_int {
                        if (*ps).legsTimer < 100 {
                            (*ps).legsTimer = 100;
                        }
                    }
                    if (*ps).torsoAnim == BOTH_MEDITATE as c_int {
                        // Raven bug: sets `legsTimer` (not `torsoTimer`) inside this
                        // `torsoTimer < 100` guard; preserved. Source: bg_pmove.c:10349-10352.
                        if (*ps).torsoTimer < 100 {
                            (*ps).legsTimer = 100;
                        }
                        (*ps).forceHandExtend = HANDEXTEND_TAUNT as c_int;
                        (*ps).forceHandExtendTime = (*pm).cmd.serverTime + 100;
                    }
                    if (*ps).legsTimer > 0 || (*ps).torsoTimer > 0 {
                        stiffenedUp = qtrue;
                        PM_SetPMViewAngle(ps, (*ps).viewangles, &mut (*pm).cmd);
                        (*pm).cmd.rightmove = 0;
                        (*pm).cmd.upmove = 0;
                        (*pm).cmd.forwardmove = 0;
                        (*pm).cmd.buttons = 0;
                    }
                }
            } else if (*ps).legsAnim == BOTH_MEDITATE_END as c_int && (*ps).legsTimer > 0 {
                stiffenedUp = qtrue;
                PM_SetPMViewAngle(ps, (*ps).viewangles, &mut (*pm).cmd);
                (*pm).cmd.rightmove = 0;
                (*pm).cmd.upmove = 0;
                (*pm).cmd.forwardmove = 0;
                (*pm).cmd.buttons = 0;
            } else if (*ps).legsAnim == BOTH_FORCELAND1 as c_int
                || (*ps).legsAnim == BOTH_FORCELANDBACK1 as c_int
                || (*ps).legsAnim == BOTH_FORCELANDRIGHT1 as c_int
                || (*ps).legsAnim == BOTH_FORCELANDLEFT1 as c_int
            {
                // can't move while in a force land
                stiffenedUp = qtrue;
            }

            if (*ps).saberMove == LS_A_LUNGE {
                // can't move during lunge
                (*pm).cmd.rightmove = 0;
                (*pm).cmd.upmove = 0;
                if (*ps).legsTimer > 500 {
                    (*pm).cmd.forwardmove = 127;
                } else {
                    (*pm).cmd.forwardmove = 0;
                }
            }

            if (*ps).saberMove == LS_A_JUMP_T__B_ {
                // can't move during leap
                if (*ps).groundEntityNum != ENTITYNUM_NONE {
                    // hit the ground
                    (*pm).cmd.forwardmove = 0;
                }
                (*pm).cmd.rightmove = 0;
                (*pm).cmd.upmove = 0;
            }

            if (*ps).emplacedIndex != 0 {
                if (*pm).cmd.forwardmove < 0 || self.PM_GroundDistance() > 32.0 {
                    (*ps).emplacedIndex = 0;
                    (*ps).saberHolstered = 0;
                } else {
                    stiffenedUp = qtrue;
                }
            }

            if (*ps).weapon == WP_DISRUPTOR as c_int
                && (*ps).weaponstate == WEAPON_CHARGING_ALT as c_int
            {
                // not allowed to move while charging the disruptor
                if (*pm).cmd.forwardmove != 0 || (*pm).cmd.rightmove != 0 || (*pm).cmd.upmove > 0 {
                    // get out
                    (*ps).weaponstate = WEAPON_READY as c_int;
                    (*ps).weaponTime = 1000;
                    // cut the weapon charge sound
                    self.PM_AddEventWithParm(EV_WEAPON_CHARGE as c_int, WP_DISRUPTOR as c_int);
                    (*pm).cmd.upmove = 0;
                }
            } else if (*ps).weapon == WP_DISRUPTOR as c_int && (*ps).zoomMode == 1 {
                // can't jump
                if (*pm).cmd.upmove > 0 {
                    (*pm).cmd.upmove = 0;
                }
            }

            if stiffenedUp != qfalse {
                (*pm).cmd.forwardmove = 0;
                (*pm).cmd.rightmove = 0;
                (*pm).cmd.upmove = 0;
            }

            if (*ps).fd.forceGripCripple != 0 {
                // don't let attack or alt attack if being gripped I guess
                (*pm).cmd.buttons &= !BUTTON_ATTACK;
                (*pm).cmd.buttons &= !BUTTON_ALT_ATTACK;
            }

            if BG_InRoll(ps, (*ps).legsAnim) != qfalse {
                // can't roll unless you're able to move normally
                BG_CmdForRoll(ps, (*ps).legsAnim, &mut (*pm).cmd, self.bg);
            }

            self.PM_CmdForSaberMoves(&mut (*pm).cmd);

            self.BG_AdjustClientSpeed(ps, &mut (*pm).cmd, (*pm).cmd.serverTime);

            if (*ps).stats[STAT_HEALTH as usize] <= 0 {
                // corpses can fly through bodies
                (*pm).tracemask &= !CONTENTS_BODY;
            }

            // make sure walking button is clear if they are running, to avoid
            // proxy no-footsteps cheats
            if ((*pm).cmd.forwardmove as c_int).abs() > 64
                || ((*pm).cmd.rightmove as c_int).abs() > 64
            {
                (*pm).cmd.buttons &= !BUTTON_WALKING;
            }

            // set the talk balloon flag
            if (*pm).cmd.buttons & BUTTON_TALK != 0 {
                (*ps).eFlags |= EF_TALK;
            } else {
                (*ps).eFlags &= !EF_TALK;
            }

            self.pm_cancelOutZoom = qfalse;
            if (*ps).weapon == WP_DISRUPTOR as c_int && (*ps).zoomMode == 1 {
                if (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0
                    && (*pm).cmd.buttons & BUTTON_ATTACK == 0
                    && (*ps).zoomLocked != 0
                {
                    self.pm_cancelOutZoom = qtrue;
                }
            }
            // In certain situations, we may want to control which attack buttons are pressed
            // and what kind of functionality is attached to them
            self.PM_AdjustAttackStates(pm);

            // clear the respawned flag if attack and use are cleared
            if (*ps).stats[STAT_HEALTH as usize] > 0
                && (*pm).cmd.buttons & (BUTTON_ATTACK | BUTTON_USE_HOLDABLE) == 0
            {
                (*ps).pm_flags &= !PMF_RESPAWNED;
            }

            // if talk button is down, disallow all other input; this is to prevent any
            // possible intercept proxy from adding fake talk balloons
            if (*pm).cmd.buttons & BUTTON_TALK != 0 {
                // keep the talk button set tho for when the cmd.serverTime > 66 msec
                // and the same cmd is used multiple times in Pmove
                (*pm).cmd.buttons = BUTTON_TALK;
                (*pm).cmd.forwardmove = 0;
                (*pm).cmd.rightmove = 0;
                (*pm).cmd.upmove = 0;
            }

            // clear all pmove local vars
            self.pml = core::mem::zeroed();

            // determine the time
            self.pml.msec = (*pm).cmd.serverTime - (*ps).commandTime;
            if self.pml.msec < 1 {
                self.pml.msec = 1;
            } else if self.pml.msec > 200 {
                self.pml.msec = 200;
            }

            (*ps).commandTime = (*pm).cmd.serverTime;

            // save old org in case we get stuck
            _VectorCopy((*ps).origin, &mut self.pml.previous_origin);

            // save old velocity for crashlanding
            _VectorCopy((*ps).velocity, &mut self.pml.previous_velocity);

            // Raven: `pml.msec * 0.001` — `msec` is int, `0.001` a double literal, so
            // the product is computed in f64 and narrowed to the float `frametime`. An
            // f32 multiply double-rounds and diverges by 1 ULP for some msec (e.g. 18).
            self.pml.frametime = (self.pml.msec as f64 * 0.001) as f32;

            if (*ps).clientNum >= MAX_CLIENTS as c_int
                && !self.pm_entSelf.is_null()
                && (*self.pm_entSelf).s.NPC_class == CLASS_VEHICLE as c_int
            {
                // we are a vehicle
                let veh = self.pm_entSelf;
                if !veh.is_null() && !(*veh).m_pVehicle.is_null() {
                    (*((*veh).m_pVehicle as *mut crate::vehicles::Vehicle_t)).m_fTimeModifier =
                        self.pml.frametime * 60.0;
                }
            } else if (*self.pm_entSelf).s.NPC_class != CLASS_VEHICLE as c_int
                && (*ps).m_iVehicleNum != 0
            {
                let veh = self.pm_entVeh;
                if !veh.is_null()
                    && !(*veh).playerState.is_null()
                    && ((*pm).cmd.serverTime - (*(*veh).playerState).hyperSpaceTime)
                        < HYPERSPACE_TIME as c_int
                {
                    // going into hyperspace, turn to face the right angles
                    self.PM_VehFaceHyperspacePoint(veh);
                } else if !veh.is_null()
                    && !(*veh).playerState.is_null()
                    && (*(*veh).playerState).vehTurnaroundIndex != 0
                    && (*(*veh).playerState).vehTurnaroundTime > (*pm).cmd.serverTime
                {
                    // riding this vehicle, turn my view too
                    self.PM_VehForcedTurning(veh);
                }
            }

            if (*ps).legsAnim == BOTH_FORCEWALLRUNFLIP_ALT as c_int && (*ps).legsTimer > 0 {
                let mut vFwd: vec3_t = [0.0; 3];
                let mut fwdAng: vec3_t = [0.0; 3];
                VectorSet(&mut fwdAng, 0.0, (*ps).viewangles[YAW], 0.0);

                AngleVectors(fwdAng, Some(&mut vFwd), None, None);
                if (*ps).groundEntityNum == ENTITYNUM_NONE {
                    let savZ = (*ps).velocity[2];
                    _VectorScale(vFwd, 100.0, &mut (*ps).velocity);
                    (*ps).velocity[2] = savZ;
                }
                (*pm).cmd.forwardmove = 0;
                (*pm).cmd.rightmove = 0;
                (*pm).cmd.upmove = 0;
                self.PM_AdjustAnglesForWallRunUpFlipAlt(&mut (*pm).cmd);
            }

            self.PM_AdjustAngleForWallJump(ps, &mut (*pm).cmd, qtrue);
            self.PM_AdjustAngleForWallRunUp(ps, &mut (*pm).cmd, qtrue);
            self.PM_AdjustAngleForWallRun(ps, &mut (*pm).cmd, qtrue);

            if (*ps).saberMove == LS_A_JUMP_T__B_
                || (*ps).saberMove == LS_A_LUNGE
                || (*ps).saberMove == LS_A_BACK_CR
                || (*ps).saberMove == LS_A_BACK
                || (*ps).saberMove == LS_A_BACKSTAB
            {
                PM_SetPMViewAngle(ps, (*ps).viewangles, &mut (*pm).cmd);
            }

            self.PM_SetSpecialMoveValues();

            // update the viewangles
            self.PM_UpdateViewAngles(ps, &(*pm).cmd);

            AngleVectors(
                (*ps).viewangles,
                Some(&mut self.pml.forward),
                Some(&mut self.pml.right),
                Some(&mut self.pml.up),
            );

            if ((*pm).cmd.upmove as c_int) < 10 && (*ps).pm_flags & PMF_STUCK_TO_WALL == 0 {
                // not holding jump
                (*ps).pm_flags &= !PMF_JUMP_HELD;
            }

            // decide if backpedaling animations should be used
            if (*pm).cmd.forwardmove < 0 {
                (*ps).pm_flags |= PMF_BACKWARDS_RUN;
            } else if (*pm).cmd.forwardmove > 0
                || ((*pm).cmd.forwardmove == 0 && (*pm).cmd.rightmove != 0)
            {
                (*ps).pm_flags &= !PMF_BACKWARDS_RUN;
            }

            if (*ps).pm_type >= PM_DEAD as c_int {
                (*pm).cmd.forwardmove = 0;
                (*pm).cmd.rightmove = 0;
                (*pm).cmd.upmove = 0;
            }

            if (*ps).saberLockTime >= (*pm).cmd.serverTime {
                (*pm).cmd.upmove = 0;
                (*pm).cmd.forwardmove = 0;
                (*pm).cmd.rightmove = 0;
            }

            if (*ps).pm_type == PM_SPECTATOR as c_int {
                self.PM_CheckDuck();
                if (*pm).noSpecMove == 0 {
                    self.PM_FlyMove();
                }
                self.PM_DropTimers();
                return;
            }

            if (*ps).pm_type == PM_NOCLIP as c_int {
                if (*ps).clientNum < MAX_CLIENTS as c_int {
                    self.PM_NoclipMove();
                    self.PM_DropTimers();
                    return;
                }
            }

            if (*ps).pm_type == PM_FREEZE as c_int {
                return; // no movement at all
            }

            if (*ps).pm_type == PM_INTERMISSION as c_int
                || (*ps).pm_type == PM_SPINTERMISSION as c_int
            {
                return; // no movement at all
            }

            // set watertype, and waterlevel
            self.PM_SetWaterLevel();
            self.pml.previous_waterlevel = (*pm).waterlevel;

            // set mins, maxs, and viewheight
            self.PM_CheckDuck();

            if (*ps).pm_type == PM_JETPACK as c_int {
                gDist = self.PM_GroundDistance();
                savedGravity = (*ps).gravity;

                if gDist < (JETPACK_HOVER_HEIGHT + 64) as f32 {
                    (*ps).gravity = ((*ps).gravity as f32 * 0.1) as c_int;
                } else {
                    (*ps).gravity = ((*ps).gravity as f32 * 0.25) as c_int;
                }
            } else if self.gPMDoSlowFall != qfalse {
                savedGravity = (*ps).gravity;
                (*ps).gravity = ((*ps).gravity as f32 * 0.5) as c_int;
            }

            // if we're in jetpack mode then see if we should be jetting around
            if (*ps).pm_type == PM_JETPACK as c_int {
                if (*pm).cmd.rightmove > 0 {
                    self.PM_ContinueLegsAnim(BOTH_INAIRRIGHT1 as c_int);
                } else if (*pm).cmd.rightmove < 0 {
                    self.PM_ContinueLegsAnim(BOTH_INAIRLEFT1 as c_int);
                } else if (*pm).cmd.forwardmove > 0 {
                    self.PM_ContinueLegsAnim(BOTH_INAIR1 as c_int);
                } else if (*pm).cmd.forwardmove < 0 {
                    self.PM_ContinueLegsAnim(BOTH_INAIRBACK1 as c_int);
                } else {
                    self.PM_ContinueLegsAnim(BOTH_INAIR1 as c_int);
                }

                if (*ps).weapon == WP_SABER as c_int
                    && BG_SpinningSaberAnim((*ps).legsAnim) != qfalse
                {
                    // make him stir around since he shouldn't have any real control when spinning
                    (*ps).velocity[0] += self.bg.rng.Q_irand(-100, 100) as f32;
                    (*ps).velocity[1] += self.bg.rng.Q_irand(-100, 100) as f32;
                }

                if (*pm).cmd.upmove > 0 && (*ps).velocity[2] < 256.0 {
                    // cap upward velocity off at 256. Seems reasonable.
                    let mut addIn: f32 = 12.0;

                    if (*ps).velocity[2] > 0.0 {
                        addIn = 12.0 - (gDist / 64.0);
                    }

                    if addIn > 0.0 {
                        (*ps).velocity[2] += addIn;
                    }

                    (*ps).eFlags |= EF_JETPACK_FLAMING; // going up
                } else {
                    (*ps).eFlags &= !EF_JETPACK_FLAMING; // idling

                    if (*ps).velocity[2] < 256.0 {
                        if (*ps).velocity[2] < -100.0 {
                            (*ps).velocity[2] = -100.0;
                        }
                        if gDist < JETPACK_HOVER_HEIGHT as f32 {
                            // make sure we're always hovering off the ground somewhat while jetpack is active
                            (*ps).velocity[2] += 2.0;
                        }
                    }
                }
            }

            if (*ps).clientNum >= MAX_CLIENTS as c_int
                && !self.pm_entSelf.is_null()
                && !(*self.pm_entSelf).m_pVehicle.is_null()
            {
                // Now update our mins/maxs to match our m_vOrientation based on our
                // length, width & height
                self.BG_VehicleAdjustBBoxForOrientation(
                    (*self.pm_entSelf).m_pVehicle as *mut Vehicle_t,
                    (*ps).origin,
                    &mut (*pm).mins,
                    &mut (*pm).maxs,
                    (*ps).clientNum,
                    (*pm).tracemask,
                );
            }

            // set groundentity
            self.PM_GroundTrace();
            if self.pm_flying == FLY_HOVER {
                // never stick to the ground
                self.PM_HoverTrace();
            }

            if (*ps).groundEntityNum != ENTITYNUM_NONE {
                // on ground
                (*ps).fd.forceJumpZStart = 0.0;
            }

            if (*ps).pm_type == PM_DEAD as c_int {
                if (*ps).clientNum >= MAX_CLIENTS as c_int
                    && !self.pm_entSelf.is_null()
                    && (*self.pm_entSelf).s.NPC_class == CLASS_VEHICLE as c_int
                    && (*(*((*self.pm_entSelf).m_pVehicle as *mut crate::vehicles::Vehicle_t))
                        .m_pVehicleInfo)
                        .r#type as c_int
                        != VH_ANIMAL as c_int
                {
                    // vehicles don't use deadmove
                } else {
                    self.PM_DeadMove();
                }
            }

            self.PM_DropTimers();

            if (*self.pm_entSelf).s.NPC_class != CLASS_VEHICLE as c_int && (*ps).m_iVehicleNum != 0
            {
                // a player riding a vehicle
                let veh = self.pm_entVeh;

                if !veh.is_null()
                    && !(*veh).m_pVehicle.is_null()
                    && ((*(*((*veh).m_pVehicle as *mut crate::vehicles::Vehicle_t)).m_pVehicleInfo)
                        .r#type as c_int
                        == VH_WALKER as c_int
                        || (*(*((*veh).m_pVehicle as *mut crate::vehicles::Vehicle_t))
                            .m_pVehicleInfo)
                            .r#type as c_int
                            == VH_FIGHTER as c_int)
                {
                    // *sigh*, until we get forced weapon-switching working?
                    (*pm).cmd.buttons &= !(BUTTON_ATTACK | BUTTON_ALT_ATTACK);
                    (*ps).eFlags &= !(EF_FIRING | EF_ALT_FIRING);
                }
            }

            if (*ps).m_iVehicleNum == 0
                && (*self.pm_entSelf).s.NPC_class != CLASS_VEHICLE as c_int
                && (*self.pm_entSelf).s.NPC_class != CLASS_RANCOR as c_int
                && (*ps).groundEntityNum < ENTITYNUM_WORLD
                && (*ps).groundEntityNum >= MAX_CLIENTS as c_int
            {
                // I am a player client, not riding on a vehicle, and potentially standing on an NPC
                let pEnt = self.PM_BGEntForNum((*ps).groundEntityNum);

                if !pEnt.is_null()
                    && (*pEnt).s.eType == entityType_t::ET_NPC as c_int
                    && (*pEnt).s.NPC_class != CLASS_VEHICLE as c_int
                {
                    // this is actually an NPC, let's try to bounce off its head to make
                    // sure we can't just stand around on top of it.
                    if (*ps).velocity[2] < 270.0 {
                        // try forcing velocity up and also force him to jump
                        (*ps).velocity[2] = 270.0; // seems reasonable
                        (*pm).cmd.upmove = 127;
                    }
                }
                // QAGAME: if land on an empty, suspended vehicle, get in it
                else if (*ps).zoomMode == 0
                    && !self.pm_entSelf.is_null()
                    && !(*pEnt).m_pVehicle.is_null()
                {
                    // S5-2: the client/m_iVehicleNum/spawnflags gate is a game-side
                    // read; reach it by entity number through the upcall.
                    if self.callbacks.suspended_vehicle_boardable((*pEnt).s.number) != 0
                    // SUSPENDED
                    {
                        // it's a vehicle, get in it. The vehicle `Board` body is
                        // game-tier; bg reaches it via the GameCallbacks upcall
                        // (by entity number), which dispatches through
                        // `crate::veh_dispatch::board`.
                        self.callbacks
                            .board_vehicle((*pEnt).s.number, (*self.pm_entSelf).s.number);
                    }
                }
            }

            if (*ps).clientNum >= MAX_CLIENTS as c_int
                && !self.pm_entSelf.is_null()
                && (*self.pm_entSelf).s.NPC_class == CLASS_VEHICLE as c_int
            {
                // we are a vehicle
                let veh = self.pm_entSelf;
                let pVeh = (*veh).m_pVehicle as *mut crate::vehicles::Vehicle_t;

                debug_assert!(
                    !veh.is_null()
                        && !(*veh).playerState.is_null()
                        && !pVeh.is_null()
                        && (*veh).s.number >= MAX_CLIENTS as c_int
                );

                if (*(*pVeh).m_pVehicleInfo).r#type as c_int != VH_FIGHTER as c_int {
                    // kind of hacky, don't want to do this for flying vehicles
                    *(*pVeh).m_vOrientation.add(PITCH) = (*ps).viewangles[PITCH];
                }

                if (*ps).m_iVehicleNum == 0 {
                    // no one is driving, just update and get out (QAGAME). The
                    // `Update`/`Animate` virtuals are game-tier; bg reaches them via
                    // the GameCallbacks upcalls (by entity number).
                    // Source: oracle/codemp/game/bg_pmove.c:10919-10922
                    self.callbacks
                        .update_vehicle((*veh).s.number, &(*self.pm).cmd);
                    self.callbacks.pm_animate_vehicle((*veh).s.number);
                } else {
                    let selfEnt = self.pm_entVeh;

                    debug_assert!(
                        !selfEnt.is_null()
                            && !(*selfEnt).playerState.is_null()
                            && (*selfEnt).s.number < MAX_CLIENTS as c_int
                    );

                    if (*ps).pm_type == PM_DEAD as c_int
                        && (*pVeh).m_ulFlags & (VEH_CRASHING as u64) != 0
                    {
                        (*pVeh).m_ulFlags &= !(VEH_CRASHING as u64);
                    }

                    if !(*selfEnt).playerState.is_null()
                        && (*(*selfEnt).playerState).m_iVehicleNum != 0
                    {
                        // only do it if they still have a vehicle (didn't get ejected this update)
                        PM_VehicleViewAngles(
                            (*selfEnt).playerState,
                            veh,
                            &mut (*pVeh).m_ucmd,
                            self.bg,
                        );
                    }

                    // The `Update`/`Animate`/`UpdateRider` virtuals and the passenger
                    // `UpdateRider` loop are game-tier; bg reaches them via the
                    // GameCallbacks upcalls. The driver's cmd is the bg-reachable
                    // `m_ucmd`; each passenger's `client->pers.cmd` is game-side, so a
                    // null `ucmd` signals the impl to use the rider's own pers.cmd
                    // (and to guard `inuse && client`).
                    // Source: oracle/codemp/game/bg_pmove.c:10944-10961
                    self.callbacks
                        .update_vehicle((*veh).s.number, &(*pVeh).m_ucmd);
                    self.callbacks.pm_animate_vehicle((*veh).s.number);
                    self.callbacks.update_rider(
                        (*veh).s.number,
                        (*selfEnt).s.number,
                        &mut (*pVeh).m_ucmd,
                    );
                    // update the passengers
                    let mut i: c_int = 0;
                    while i < (*pVeh).m_iNumPassengers {
                        if !(*pVeh).m_ppPassengers[i as usize].is_null() {
                            let passNum = (*(*pVeh).m_ppPassengers[i as usize]).s.number;
                            self.callbacks.update_rider(
                                (*veh).s.number,
                                passNum,
                                core::ptr::null_mut(),
                            );
                        }
                        i += 1;
                    }
                }
                noAnimate = qtrue;
            }

            if (*self.pm_entSelf).s.NPC_class != CLASS_VEHICLE as c_int && (*ps).m_iVehicleNum != 0
            {
                // don't even run physics on a player if he's on a vehicle -
                // he goes where the vehicle goes
            } else {
                if (*ps).pm_type == PM_FLOAT as c_int || self.pm_flying == FLY_NORMAL {
                    self.PM_FlyMove();
                } else if self.pm_flying == FLY_VEHICLE {
                    self.PM_FlyVehicleMove();
                } else if (*ps).pm_flags & PMF_TIME_WATERJUMP != 0 {
                    self.PM_WaterJumpMove();
                } else if (*pm).waterlevel > 1 {
                    // swimming
                    self.PM_WaterMove();
                } else if self.pml.walking != qfalse {
                    // walking on ground
                    self.PM_WalkMove();
                } else {
                    // airborne
                    self.PM_AirMove();
                }
            }

            if noAnimate == qfalse {
                self.PM_Animate();
            }

            // set groundentity, watertype, and waterlevel
            self.PM_GroundTrace();
            if self.pm_flying == FLY_HOVER {
                // never stick to the ground
                self.PM_HoverTrace();
            }
            self.PM_SetWaterLevel();
            if (*pm).cmd.forcesel as c_int != -1
                && (*ps).fd.forcePowersKnown & 1i32.wrapping_shl((*pm).cmd.forcesel as u32) != 0
            {
                // `cmd.forcesel` is `byte`, so `1 << sel` is x86 shift-masked at the 255
                // "none" sentinel; `wrapping_shl` reproduces that masked shift (§19).
                (*ps).fd.forcePowerSelected = (*pm).cmd.forcesel as c_int;
            }
            if (*pm).cmd.invensel as c_int != -1
                && (*ps).stats[STAT_HOLDABLE_ITEMS as usize]
                    & 1i32.wrapping_shl((*pm).cmd.invensel as u32)
                    != 0
            {
                (*ps).stats[STAT_HOLDABLE_ITEM as usize] =
                    BG_GetItemIndexByTag((*pm).cmd.invensel as c_int, IT_HOLDABLE as c_int);
            }

            if (*ps).m_iVehicleNum != 0 && (*ps).clientNum < MAX_CLIENTS as c_int {
                // a client riding a vehicle
                if (*ps).eFlags & EF_NODRAW != 0 {
                    // inside the vehicle, do nothing
                } else if PM_WeaponOkOnVehicle((*pm).cmd.weapon as c_int) == qfalse
                    || PM_WeaponOkOnVehicle((*ps).weapon) == qfalse
                {
                    // this weapon is not legal for the vehicle, force to our current one
                    if PM_WeaponOkOnVehicle((*ps).weapon) == qfalse {
                        // uh-oh!
                        let weap = self.PM_GetOkWeaponForVehicle();

                        if weap != -1 {
                            (*pm).cmd.weapon = weap as u8;
                            (*ps).weapon = weap;
                        }
                    } else {
                        (*pm).cmd.weapon = (*ps).weapon as u8;
                    }
                }
            }

            if (*ps).m_iVehicleNum == 0
                || (*self.pm_entSelf).s.NPC_class == CLASS_VEHICLE as c_int
                || ((*ps).eFlags & EF_NODRAW == 0
                    && PM_WeaponOkOnVehicle((*pm).cmd.weapon as c_int) != qfalse)
            {
                // only run weapons if a valid weapon is selected
                self.PM_Weapon();
            }

            self.PM_Use();

            if (*ps).m_iVehicleNum == 0
                && ((*ps).clientNum < MAX_CLIENTS as c_int
                    || self.pm_entSelf.is_null()
                    || (*self.pm_entSelf).s.NPC_class != CLASS_VEHICLE as c_int)
            {
                // don't do this if we're on a vehicle, or we are one
                // footstep events / legs animations
                self.PM_Footsteps();
            }

            // entering / leaving water splashes
            self.PM_WaterEvents();

            // snap some parts of playerstate to save network bandwidth
            self.traps.snap_vector((*ps).velocity.as_mut_ptr());

            if (*ps).pm_type == PM_JETPACK as c_int || self.gPMDoSlowFall != qfalse {
                (*ps).gravity = savedGravity;
            }

            if (*ps).clientNum >= MAX_CLIENTS as c_int
                && !self.pm_entSelf.is_null()
                && (*self.pm_entSelf).s.NPC_class == CLASS_VEHICLE as c_int
            {
                // a vehicle with passengers
                let veh = self.pm_entSelf;

                debug_assert!(!(*veh).m_pVehicle.is_null());

                // this could be kind of "inefficient" because it's called after every
                // passenger pmove too.
                if !(*veh).m_pVehicle.is_null() && !(*veh).ghoul2.is_null() {
                    // `AttachRiders` is a game-tier virtual; bg reaches it via the
                    // GameCallbacks upcall (by entity number).
                    // Source: oracle/codemp/game/bg_pmove.c:11146-11149
                    self.callbacks.attach_riders((*veh).s.number);
                }
            }

            if (*self.pm_entSelf).s.NPC_class != CLASS_VEHICLE as c_int && (*ps).m_iVehicleNum != 0
            {
                // riding a vehicle, see if we should do some anim overrides
                self.PM_VehicleWeaponAnimate();
            }
        }
    }
}

impl PmoveContext<'_> {
    /// Raven `PM_Accelerate`.
    /// Source: `oracle/codemp/game/bg_pmove.c:1133-1186`
    pub fn PM_Accelerate(&mut self, wishdir: vec3_t, wishspeed: f32, accel: f32) {
        unsafe {
            let ps = (*self.pm).ps;
            if (*self.pm).gametype != GT_SIEGE as c_int
                || (*ps).m_iVehicleNum != 0
                || (*ps).clientNum >= MAX_CLIENTS as c_int
                || (*ps).pm_type != PM_NORMAL as c_int
            {
                //standard method, allows "bunnyhopping" and whatnot
                let currentspeed = (*ps).velocity[0] * wishdir[0]
                    + (*ps).velocity[1] * wishdir[1]
                    + (*ps).velocity[2] * wishdir[2];
                let addspeed = wishspeed - currentspeed;
                if addspeed <= 0.0 && (*ps).clientNum < MAX_CLIENTS as c_int {
                    return;
                }

                let accelspeed;
                if addspeed < 0.0 {
                    let mut a = (-accel) * self.pml.frametime * wishspeed;
                    if a < addspeed {
                        a = addspeed;
                    }
                    accelspeed = a;
                } else {
                    let mut a = accel * self.pml.frametime * wishspeed;
                    if a > addspeed {
                        a = addspeed;
                    }
                    accelspeed = a;
                }

                for i in 0..3 {
                    (*ps).velocity[i] += accelspeed * wishdir[i];
                }
            } else {
                //use the proper way for siege
                let mut wishVelocity: vec3_t = [0.0; 3];
                let mut pushDir: vec3_t = [0.0; 3];

                _VectorScale(wishdir, wishspeed, &mut wishVelocity);
                _VectorSubtract(wishVelocity, (*ps).velocity, &mut pushDir);
                let pushLen = VectorNormalize(&mut pushDir);

                let mut canPush = accel * self.pml.frametime * wishspeed;
                if canPush > pushLen {
                    canPush = pushLen;
                }

                let v = (*ps).velocity;
                _VectorMA(v, canPush, pushDir, &mut (*ps).velocity);
            }
        }
    }

    /// Raven `PM_CmdScale`.
    /// Source: `oracle/codemp/game/bg_pmove.c:1199-1222`
    pub fn PM_CmdScale(&mut self, cmd: *mut usercmd_t) -> f32 {
        unsafe {
            let umove: c_int = 0; //cmd->upmove; don't factor upmove into scaling speed

            let mut max = ((*cmd).forwardmove as c_int).abs();
            if ((*cmd).rightmove as c_int).abs() > max {
                max = ((*cmd).rightmove as c_int).abs();
            }
            if umove.abs() > max {
                max = umove.abs();
            }
            if max == 0 {
                return 0.0;
            }

            let sum: c_int = ((*cmd).forwardmove as c_int) * ((*cmd).forwardmove as c_int)
                + ((*cmd).rightmove as c_int) * ((*cmd).rightmove as c_int)
                + umove * umove;
            // C: `(float)(int sum)` then `sqrt` promotes to double, result truncated to float.
            let total = ((sum as f32) as f64).sqrt() as f32;
            // C divides through `double` (the `127.0` literal); replicate to stay bit-exact.
            let a: f32 = (*(*self.pm).ps).speed * max as f32;
            let scale = (a as f64 / (127.0_f64 * total as f64)) as f32;

            scale
        }
    }

    /// Raven `PM_SetMovementDir`.
    /// Source: `oracle/codemp/game/bg_pmove.c:1233-1262`
    pub fn PM_SetMovementDir(&mut self) {
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            if (*pm).cmd.forwardmove != 0 || (*pm).cmd.rightmove != 0 {
                if (*pm).cmd.rightmove == 0 && (*pm).cmd.forwardmove > 0 {
                    (*ps).movementDir = 0;
                } else if (*pm).cmd.rightmove < 0 && (*pm).cmd.forwardmove > 0 {
                    (*ps).movementDir = 1;
                } else if (*pm).cmd.rightmove < 0 && (*pm).cmd.forwardmove == 0 {
                    (*ps).movementDir = 2;
                } else if (*pm).cmd.rightmove < 0 && (*pm).cmd.forwardmove < 0 {
                    (*ps).movementDir = 3;
                } else if (*pm).cmd.rightmove == 0 && (*pm).cmd.forwardmove < 0 {
                    (*ps).movementDir = 4;
                } else if (*pm).cmd.rightmove > 0 && (*pm).cmd.forwardmove < 0 {
                    (*ps).movementDir = 5;
                } else if (*pm).cmd.rightmove > 0 && (*pm).cmd.forwardmove == 0 {
                    (*ps).movementDir = 6;
                } else if (*pm).cmd.rightmove > 0 && (*pm).cmd.forwardmove > 0 {
                    (*ps).movementDir = 7;
                }
            } else {
                // if they aren't actively going directly sideways, change the
                // animation to the diagonal so they don't stop too crooked
                if (*ps).movementDir == 2 {
                    (*ps).movementDir = 1;
                } else if (*ps).movementDir == 6 {
                    (*ps).movementDir = 7;
                }
            }
        }
    }

    /// Raven `PM_ForceJumpingUp`.
    /// Source: `oracle/codemp/game/bg_pmove.c:1266-1306`
    pub fn PM_ForceJumpingUp(&mut self) -> qboolean {
        unsafe {
            let ps = (*self.pm).ps;
            if (*ps).fd.forcePowersActive & (1 << FP_LEVITATION) == 0
                && (*ps).fd.forceJumpCharge != 0.0
            {
                //already jumped and let go
                return qfalse;
            }

            if BG_InSpecialJump((*ps).legsAnim) != qfalse {
                return qfalse;
            }

            if BG_SaberInSpecial((*ps).saberMove) != qfalse {
                return qfalse;
            }

            if BG_SaberInSpecialAttack((*ps).legsAnim) != qfalse {
                return qfalse;
            }

            if BG_HasYsalamiri((*self.pm).gametype, ps) != qfalse {
                return qfalse;
            }

            if BG_CanUseFPNow(
                (*self.pm).gametype,
                ps,
                (*self.pm).cmd.serverTime,
                FP_LEVITATION,
            ) == qfalse
            {
                return qfalse;
            }

            if (*ps).groundEntityNum == ENTITYNUM_NONE //in air
                && (*ps).pm_flags & PMF_JUMP_HELD != 0 //jumped
                && (*ps).fd.forcePowerLevel[FP_LEVITATION as usize] > FORCE_LEVEL_0 //force-jump capable
                && (*ps).velocity[2] > 0.0
            //going up
            {
                return qtrue;
            }
            qfalse
        }
    }

    /// Raven `PM_JumpForDir`.
    /// Source: `oracle/codemp/game/bg_pmove.c:1308-1340`
    pub fn PM_JumpForDir(&mut self) {
        use animNumber_t::*;
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let anim;
            if (*pm).cmd.forwardmove > 0 {
                anim = BOTH_JUMP1 as c_int;
                (*ps).pm_flags &= !PMF_BACKWARDS_JUMP;
            } else if (*pm).cmd.forwardmove < 0 {
                anim = BOTH_JUMPBACK1 as c_int;
                (*ps).pm_flags |= PMF_BACKWARDS_JUMP;
            } else if (*pm).cmd.rightmove > 0 {
                anim = BOTH_JUMPRIGHT1 as c_int;
                (*ps).pm_flags &= !PMF_BACKWARDS_JUMP;
            } else if (*pm).cmd.rightmove < 0 {
                anim = BOTH_JUMPLEFT1 as c_int;
                (*ps).pm_flags &= !PMF_BACKWARDS_JUMP;
            } else {
                anim = BOTH_JUMP1 as c_int;
                (*ps).pm_flags &= !PMF_BACKWARDS_JUMP;
            }
            if BG_InDeathAnim((*ps).legsAnim) == qfalse {
                self.PM_SetAnim(SETANIM_LEGS, anim, SETANIM_FLAG_OVERRIDE, 100);
            }
        }
    }
}

/// Raven `PM_SetPMViewAngle`.
///
/// Source: `oracle/codemp/game/bg_pmove.c:1342-1354`
pub fn PM_SetPMViewAngle(ps: *mut playerState_t, angle: vec3_t, ucmd: *mut usercmd_t) {
    unsafe {
        for i in 0..3 {
            // set the delta angle. Raven `ANGLE2SHORT(x)` == `((int)((x)*65536/360) & 65535)`.
            let cmdAngle: c_int = ((angle[i] * 65536.0 / 360.0) as c_int) & 65535;
            (*ps).delta_angles[i] = cmdAngle - (*ucmd).angles[i];
        }
        (*ps).viewangles = angle;
    }
}

impl PmoveContext<'_> {
    /// Raven `PM_AdjustAngleForWallRun`.
    /// Source: `oracle/codemp/game/bg_pmove.c:1356-1462`
    pub fn PM_AdjustAngleForWallRun(
        &mut self,
        ps: *mut playerState_t,
        ucmd: *mut usercmd_t,
        doMove: qboolean,
    ) -> qboolean {
        use animNumber_t::*;
        unsafe {
            if ((*ps).legsAnim == BOTH_WALL_RUN_RIGHT as c_int
                || (*ps).legsAnim == BOTH_WALL_RUN_LEFT as c_int)
                && (*ps).legsTimer > 500
            {
                //wall-running and not at end of anim
                let mut fwd: vec3_t = [0.0; 3];
                let mut rt: vec3_t = [0.0; 3];
                let mut traceTo: vec3_t = [0.0; 3];
                let mut mins: vec3_t = [0.0; 3];
                let mut maxs: vec3_t = [0.0; 3];
                let mut fwdAngles: vec3_t = [0.0; 3];
                let mut trace: trace_t = core::mem::zeroed();
                let dist;
                let yawAdjust;

                VectorSet(&mut mins, -15.0, -15.0, 0.0);
                VectorSet(&mut maxs, 15.0, 15.0, 24.0);
                VectorSet(&mut fwdAngles, 0.0, (*(*self.pm).ps).viewangles[YAW], 0.0);

                AngleVectors(fwdAngles, Some(&mut fwd), Some(&mut rt), None);
                if (*ps).legsAnim == BOTH_WALL_RUN_RIGHT as c_int {
                    dist = 128.0;
                    yawAdjust = -90.0;
                } else {
                    dist = -128.0;
                    yawAdjust = 90.0;
                }
                _VectorMA((*ps).origin, dist, rt, &mut traceTo);

                self.traps.trace(
                    &mut trace,
                    core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                    core::ptr::addr_of!(mins) as *const vec3_t,
                    core::ptr::addr_of!(maxs) as *const vec3_t,
                    core::ptr::addr_of!(traceTo) as *const vec3_t,
                    (*ps).clientNum,
                    MASK_PLAYERSOLID,
                );

                if trace.fraction < 1.0
                    && (trace.plane.normal[2] >= 0.0 && trace.plane.normal[2] <= 0.4)
                {
                    let mut trace2: trace_t = core::mem::zeroed();
                    let mut traceTo2: vec3_t = [0.0; 3];
                    let mut wallRunFwd: vec3_t = [0.0; 3];
                    let mut wallRunAngles: vec3_t = [0.0; 3];

                    VectorClear(&mut wallRunAngles);
                    wallRunAngles[YAW] = vectoyaw(trace.plane.normal) + yawAdjust;
                    AngleVectors(wallRunAngles, Some(&mut wallRunFwd), None, None);

                    _VectorMA((*(*self.pm).ps).origin, 32.0, wallRunFwd, &mut traceTo2);
                    self.traps.trace(
                        &mut trace2,
                        core::ptr::addr_of!((*(*self.pm).ps).origin) as *const vec3_t,
                        core::ptr::addr_of!(mins) as *const vec3_t,
                        core::ptr::addr_of!(maxs) as *const vec3_t,
                        core::ptr::addr_of!(traceTo2) as *const vec3_t,
                        (*(*self.pm).ps).clientNum,
                        MASK_PLAYERSOLID,
                    );
                    if trace2.fraction < 1.0
                        && (trace2.plane.normal[0] * wallRunFwd[0]
                            + trace2.plane.normal[1] * wallRunFwd[1]
                            + trace2.plane.normal[2] * wallRunFwd[2])
                            <= -0.999
                    {
                        //wall we can't run on in front of us
                        trace.fraction = 1.0; //just a way to get it to kick us off the wall below
                    }
                }

                if trace.fraction < 1.0
                    && (trace.plane.normal[2] >= 0.0 && trace.plane.normal[2] <= 0.4)
                {
                    //still a wall there
                    if (*ps).legsAnim == BOTH_WALL_RUN_RIGHT as c_int {
                        (*ucmd).rightmove = 127;
                    } else {
                        (*ucmd).rightmove = -127;
                    }
                    if (*ucmd).upmove < 0 {
                        (*ucmd).upmove = 0;
                    }
                    //make me face perpendicular to the wall
                    (*ps).viewangles[YAW] = vectoyaw(trace.plane.normal) + yawAdjust;

                    PM_SetPMViewAngle(ps, (*ps).viewangles, ucmd);

                    (*ucmd).angles[YAW] =
                        ((((*ps).viewangles[YAW] * 65536.0 / 360.0) as c_int & 65535)
                            - (*ps).delta_angles[YAW] as c_int) as c_int;
                    if doMove != qfalse {
                        //push me forward
                        let zVel = (*ps).velocity[2];
                        if (*ps).legsTimer > 500 {
                            //not at end of anim yet
                            let mut speed = 175.0;
                            if (*ucmd).forwardmove < 0 {
                                speed = 100.0;
                            } else if (*ucmd).forwardmove > 0 {
                                speed = 250.0; //running speed
                            }
                            _VectorScale(fwd, speed, &mut (*ps).velocity);
                        }
                        (*ps).velocity[2] = zVel; //preserve z velocity
                                                  //pull me toward the wall, too
                        let v = (*ps).velocity;
                        _VectorMA(v, dist, rt, &mut (*ps).velocity);
                    }
                    (*ucmd).forwardmove = 0;
                    return qtrue;
                } else if doMove != qfalse {
                    //stop it
                    if (*ps).legsAnim == BOTH_WALL_RUN_RIGHT as c_int {
                        self.PM_SetAnim(
                            SETANIM_BOTH,
                            BOTH_WALL_RUN_RIGHT_STOP as c_int,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            0,
                        );
                    } else if (*ps).legsAnim == BOTH_WALL_RUN_LEFT as c_int {
                        self.PM_SetAnim(
                            SETANIM_BOTH,
                            BOTH_WALL_RUN_LEFT_STOP as c_int,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            0,
                        );
                    }
                }
            }

            qfalse
        }
    }

    /// Raven `PM_AdjustAnglesForWallRunUpFlipAlt`.
    /// Source: `oracle/codemp/game/bg_pmove.c:1464-1470`
    pub fn PM_AdjustAnglesForWallRunUpFlipAlt(&mut self, ucmd: *mut usercmd_t) -> qboolean {
        unsafe {
            PM_SetPMViewAngle((*self.pm).ps, (*(*self.pm).ps).viewangles, ucmd);
            qtrue
        }
    }

    /// Raven `PM_AdjustAngleForWallRunUp`.
    /// Source: `oracle/codemp/game/bg_pmove.c:1472-1598`
    pub fn PM_AdjustAngleForWallRunUp(
        &mut self,
        ps: *mut playerState_t,
        ucmd: *mut usercmd_t,
        doMove: qboolean,
    ) -> qboolean {
        use animNumber_t::*;
        unsafe {
            if (*ps).legsAnim == BOTH_FORCEWALLRUNFLIP_START as c_int {
                //wall-running up
                let mut fwd: vec3_t = [0.0; 3];
                let mut traceTo: vec3_t = [0.0; 3];
                let mut mins: vec3_t = [0.0; 3];
                let mut maxs: vec3_t = [0.0; 3];
                let mut fwdAngles: vec3_t = [0.0; 3];
                let mut trace: trace_t = core::mem::zeroed();
                let dist = 128.0f32;

                VectorSet(&mut mins, -15.0, -15.0, 0.0);
                VectorSet(&mut maxs, 15.0, 15.0, 24.0);
                VectorSet(&mut fwdAngles, 0.0, (*(*self.pm).ps).viewangles[YAW], 0.0);

                AngleVectors(fwdAngles, Some(&mut fwd), None, None);
                _VectorMA((*ps).origin, dist, fwd, &mut traceTo);
                self.traps.trace(
                    &mut trace,
                    core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                    core::ptr::addr_of!(mins) as *const vec3_t,
                    core::ptr::addr_of!(maxs) as *const vec3_t,
                    core::ptr::addr_of!(traceTo) as *const vec3_t,
                    (*ps).clientNum,
                    MASK_PLAYERSOLID,
                );
                if trace.fraction > 0.5 {
                    //hmm, some room, see if there's a floor right here
                    let mut trace2: trace_t = core::mem::zeroed();
                    let mut top: vec3_t = [0.0; 3];
                    let mut bottom: vec3_t = [0.0; 3];

                    _VectorCopy(trace.endpos, &mut top);
                    top[2] += ((*self.pm).mins[2] * -1.0) + 4.0;
                    _VectorCopy(top, &mut bottom);
                    bottom[2] -= 64.0;
                    self.traps.trace(
                        &mut trace2,
                        core::ptr::addr_of!(top) as *const vec3_t,
                        core::ptr::addr_of!((*self.pm).mins) as *const vec3_t,
                        core::ptr::addr_of!((*self.pm).maxs) as *const vec3_t,
                        core::ptr::addr_of!(bottom) as *const vec3_t,
                        (*ps).clientNum,
                        MASK_PLAYERSOLID,
                    );
                    if trace2.allsolid == 0
                        && trace2.startsolid == 0
                        && trace2.fraction < 1.0
                        && trace2.plane.normal[2] > 0.7
                    {
                        //slope we can stand on
                        _VectorScale(fwd, 100.0, &mut (*(*self.pm).ps).velocity);
                        (*(*self.pm).ps).velocity[2] += 400.0;
                        self.PM_SetAnim(
                            SETANIM_BOTH,
                            BOTH_FORCEWALLRUNFLIP_ALT as c_int,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            0,
                        );
                        (*(*self.pm).ps).pm_flags |= PMF_JUMP_HELD;
                        self.PM_AddEvent(EV_JUMP as c_int);
                        (*ucmd).upmove = 0;
                        return qfalse;
                    }
                }

                if (*ps).legsTimer > 0
                    && (*ucmd).forwardmove > 0
                    && trace.fraction < 1.0
                    && (trace.plane.normal[2] >= 0.0 && trace.plane.normal[2] <= 0.4)
                {
                    //still a vertical wall there
                    //make sure there's not a ceiling above us!
                    let mut trace2: trace_t = core::mem::zeroed();
                    _VectorCopy((*ps).origin, &mut traceTo);
                    traceTo[2] += 64.0;
                    self.traps.trace(
                        &mut trace2,
                        core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                        core::ptr::addr_of!(mins) as *const vec3_t,
                        core::ptr::addr_of!(maxs) as *const vec3_t,
                        core::ptr::addr_of!(traceTo) as *const vec3_t,
                        (*ps).clientNum,
                        MASK_PLAYERSOLID,
                    );
                    if trace2.fraction < 1.0 {
                        //will hit a ceiling, so force jump-off right now
                    } else {
                        //all clear, keep going
                        (*ucmd).forwardmove = 127;
                        if (*ucmd).upmove < 0 {
                            (*ucmd).upmove = 0;
                        }
                        //make me face the wall
                        (*ps).viewangles[YAW] = vectoyaw(trace.plane.normal) + 180.0;
                        PM_SetPMViewAngle(ps, (*ps).viewangles, ucmd);
                        (*ucmd).angles[YAW] = ((((*ps).viewangles[YAW] * 65536.0 / 360.0) as c_int
                            & 65535)
                            - (*ps).delta_angles[YAW] as c_int)
                            as c_int;
                        if true
                        //aslkfhsakf
                        {
                            if doMove != qfalse {
                                //pull me toward the wall
                                _VectorScale(
                                    trace.plane.normal,
                                    -dist * trace.fraction,
                                    &mut (*ps).velocity,
                                );
                                //push me up
                                if (*ps).legsTimer > 200 {
                                    //not at end of anim yet
                                    let speed = 300.0;
                                    (*ps).velocity[2] = speed; //preserve z velocity
                                }
                            }
                        }
                        (*ucmd).forwardmove = 0;
                        return qtrue;
                    }
                }
                //failed!
                if doMove != qfalse {
                    //stop it
                    _VectorScale(fwd, -300.0, &mut (*ps).velocity);
                    (*ps).velocity[2] += 200.0;
                    self.PM_SetAnim(
                        SETANIM_BOTH,
                        BOTH_FORCEWALLRUNFLIP_END as c_int,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        0,
                    );
                    (*ps).pm_flags |= PMF_JUMP_HELD;
                    self.PM_AddEvent(EV_JUMP as c_int);
                    (*ucmd).upmove = 0;
                }
            }
            qfalse
        }
    }
}

/// Raven `BG_ForceWallJumpStrength`.
/// Source: `oracle/codemp/game/bg_pmove.c:1602-1605`
pub fn BG_ForceWallJumpStrength() -> f32 {
    crate::local::forceJumpStrength[FORCE_LEVEL_3 as usize] / 2.5
}

impl PmoveContext<'_> {
    /// Raven `PM_AdjustAngleForWallJump`.
    /// Source: `oracle/codemp/game/bg_pmove.c:1607-1756`
    pub fn PM_AdjustAngleForWallJump(
        &mut self,
        ps: *mut playerState_t,
        ucmd: *mut usercmd_t,
        doMove: qboolean,
    ) -> qboolean {
        use animNumber_t::*;
        unsafe {
            if ((BG_InReboundJump((*ps).legsAnim) != qfalse
                || BG_InReboundHold((*ps).legsAnim) != qfalse)
                && (BG_InReboundJump((*ps).torsoAnim) != qfalse
                    || BG_InReboundHold((*ps).torsoAnim) != qfalse))
                || ((*(*self.pm).ps).pm_flags & PMF_STUCK_TO_WALL != 0)
            {
                //hugging wall, getting ready to jump off
                let mut checkDir: vec3_t = [0.0; 3];
                let mut traceTo: vec3_t = [0.0; 3];
                let mut mins: vec3_t = [0.0; 3];
                let mut maxs: vec3_t = [0.0; 3];
                let mut fwdAngles: vec3_t = [0.0; 3];
                let mut trace: trace_t = core::mem::zeroed();
                let dist = 128.0f32;
                let yawAdjust;

                VectorSet(&mut mins, (*self.pm).mins[0], (*self.pm).mins[1], 0.0);
                VectorSet(&mut maxs, (*self.pm).maxs[0], (*self.pm).maxs[1], 24.0);
                VectorSet(&mut fwdAngles, 0.0, (*(*self.pm).ps).viewangles[YAW], 0.0);

                let la = (*ps).legsAnim;
                if la == BOTH_FORCEWALLREBOUND_RIGHT as c_int
                    || la == BOTH_FORCEWALLHOLD_RIGHT as c_int
                {
                    AngleVectors(fwdAngles, None, Some(&mut checkDir), None);
                    yawAdjust = -90.0;
                } else if la == BOTH_FORCEWALLREBOUND_LEFT as c_int
                    || la == BOTH_FORCEWALLHOLD_LEFT as c_int
                {
                    AngleVectors(fwdAngles, None, Some(&mut checkDir), None);
                    let c = checkDir;
                    _VectorScale(c, -1.0, &mut checkDir);
                    yawAdjust = 90.0;
                } else if la == BOTH_FORCEWALLREBOUND_FORWARD as c_int
                    || la == BOTH_FORCEWALLHOLD_FORWARD as c_int
                {
                    AngleVectors(fwdAngles, Some(&mut checkDir), None, None);
                    yawAdjust = 180.0;
                } else if la == BOTH_FORCEWALLREBOUND_BACK as c_int
                    || la == BOTH_FORCEWALLHOLD_BACK as c_int
                {
                    AngleVectors(fwdAngles, Some(&mut checkDir), None, None);
                    let c = checkDir;
                    _VectorScale(c, -1.0, &mut checkDir);
                    yawAdjust = 0.0;
                } else {
                    //WTF???
                    (*(*self.pm).ps).pm_flags &= !PMF_STUCK_TO_WALL;
                    return qfalse;
                }
                if (*self.pm).debugMelee != 0 {
                    //uber-skillz
                    if (*ucmd).upmove > 0 {
                        //hold on until you let go manually
                        if BG_InReboundHold((*ps).legsAnim) != qfalse {
                            //keep holding
                            if (*ps).legsTimer < 150 {
                                (*ps).legsTimer = 150;
                            }
                        } else {
                            //if got to hold part of anim, play hold anim
                            if (*ps).legsTimer <= 300 {
                                (*ps).saberHolstered = 2;
                                self.PM_SetAnim(
                                    SETANIM_BOTH,
                                    BOTH_FORCEWALLRELEASE_FORWARD as c_int
                                        + ((*ps).legsAnim - BOTH_FORCEWALLHOLD_FORWARD as c_int),
                                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                                    0,
                                );
                                (*ps).legsTimer = 150;
                                (*ps).torsoTimer = 150;
                            }
                        }
                    }
                }
                _VectorMA((*ps).origin, dist, checkDir, &mut traceTo);
                self.traps.trace(
                    &mut trace,
                    core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                    core::ptr::addr_of!(mins) as *const vec3_t,
                    core::ptr::addr_of!(maxs) as *const vec3_t,
                    core::ptr::addr_of!(traceTo) as *const vec3_t,
                    (*ps).clientNum,
                    MASK_PLAYERSOLID,
                );
                if (*ps).legsTimer > 100
                    && trace.fraction < 1.0
                    && Q_fabs(trace.plane.normal[2]) <= 0.2
                {
                    //still a vertical wall there
                    if (*ucmd).upmove < 0 {
                        (*ucmd).upmove = 0;
                    }
                    //align me to the wall
                    (*ps).viewangles[YAW] = vectoyaw(trace.plane.normal) + yawAdjust;
                    PM_SetPMViewAngle(ps, (*ps).viewangles, ucmd);
                    (*ucmd).angles[YAW] =
                        ((((*ps).viewangles[YAW] * 65536.0 / 360.0) as c_int & 65535)
                            - (*ps).delta_angles[YAW] as c_int) as c_int;
                    if true {
                        if doMove != qfalse {
                            //pull me toward the wall
                            _VectorScale(trace.plane.normal, -128.0, &mut (*ps).velocity);
                        }
                    }
                    (*ucmd).upmove = 0;
                    (*ps).pm_flags |= PMF_STUCK_TO_WALL;
                    return qtrue;
                } else if doMove != qfalse && (*ps).pm_flags & PMF_STUCK_TO_WALL != 0 {
                    //jump off
                    //push off of it!
                    (*ps).pm_flags &= !PMF_STUCK_TO_WALL;
                    (*ps).velocity[0] = 0.0;
                    (*ps).velocity[1] = 0.0;
                    _VectorScale(checkDir, -(JUMP_OFF_WALL_SPEED as f32), &mut (*ps).velocity);
                    (*ps).velocity[2] = BG_ForceWallJumpStrength();
                    (*ps).pm_flags |= PMF_JUMP_HELD;
                    (*ps).fd.forceJumpSound = 1; //this is a stupid thing, i should fix it.
                    if (*ps).origin[2] < (*ps).fd.forceJumpZStart {
                        (*ps).fd.forceJumpZStart = (*ps).origin[2];
                    }

                    BG_ForcePowerDrain(ps, FP_LEVITATION, 10);
                    //no control for half a second
                    (*ps).pm_flags |= PMF_TIME_KNOCKBACK;
                    (*ps).pm_time = 500;
                    (*ucmd).forwardmove = 0;
                    (*ucmd).rightmove = 0;
                    (*ucmd).upmove = 127;

                    if BG_InReboundHold((*ps).legsAnim) != qfalse {
                        //if was in hold pose, release now
                        self.PM_SetAnim(
                            SETANIM_BOTH,
                            BOTH_FORCEWALLRELEASE_FORWARD as c_int
                                + ((*ps).legsAnim - BOTH_FORCEWALLHOLD_FORWARD as c_int),
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            0,
                        );
                    } else {
                        self.PM_SetAnim(
                            SETANIM_LEGS,
                            BOTH_FORCEJUMP1 as c_int,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_RESTART,
                            0,
                        );
                    }
                }
            }
            (*ps).pm_flags &= !PMF_STUCK_TO_WALL;
            qfalse
        }
    }

    /// Raven `PM_SetForceJumpZStart`.
    /// Source: `oracle/codemp/game/bg_pmove.c:1759-1766`
    pub fn PM_SetForceJumpZStart(&mut self, value: f32) {
        unsafe {
            let ps = (*self.pm).ps;
            (*ps).fd.forceJumpZStart = value;
            if (*ps).fd.forceJumpZStart == 0.0 {
                (*ps).fd.forceJumpZStart -= 0.1;
            }
        }
    }

    /// Raven `PM_GrabWallForJump`.
    /// Source: `oracle/codemp/game/bg_pmove.c:1776-1781`
    //NOTE!!! assumes an appropriate anim is being passed in!!!
    pub fn PM_GrabWallForJump(&mut self, anim: c_int) {
        unsafe {
            self.PM_SetAnim(
                SETANIM_BOTH,
                anim,
                SETANIM_FLAG_RESTART | SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                0,
            );
            self.PM_AddEvent(EV_JUMP as c_int); //make sound for grab
            (*(*self.pm).ps).pm_flags |= PMF_STUCK_TO_WALL;
        }
    }
}

impl PmoveContext<'_> {
    /// Raven `PM_CheckJump`. `METROID_JUMP` is defined, so that block is compiled.
    /// Source: `oracle/codemp/game/bg_pmove.c:1788-2775`
    pub fn PM_CheckJump(&mut self) -> qboolean {
        use crate::public::jump_velocity::JUMP_VELOCITY;
        use animNumber_t::*;
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let mut allowFlips = qtrue;

            if (*ps).clientNum >= MAX_CLIENTS as c_int {
                let pEnt = self.pm_entSelf;
                if (*pEnt).s.eType == entityType_t::ET_NPC as c_int
                    && (*pEnt).s.NPC_class == CLASS_VEHICLE as c_int
                {
                    //no!
                    return qfalse;
                }
            }

            if (*ps).forceHandExtend == HANDEXTEND_KNOCKDOWN as c_int
                || (*ps).forceHandExtend == HANDEXTEND_PRETHROWN as c_int
                || (*ps).forceHandExtend == HANDEXTEND_POSTTHROWN as c_int
            {
                return qfalse;
            }

            if (*ps).pm_type == PM_JETPACK as c_int {
                //there's no actual jumping while we jetpack
                return qfalse;
            }

            //Don't allow jump until all buttons are up
            if (*ps).pm_flags & PMF_RESPAWNED != 0 {
                return qfalse;
            }

            if PM_InKnockDown(ps) != qfalse || BG_InRoll(ps, (*ps).legsAnim) != qfalse {
                //in knockdown
                return qfalse;
            }

            if (*ps).weapon == WP_SABER as c_int {
                let saber1 = self.callbacks.my_saber((*ps).clientNum, 0);
                let saber2 = self.callbacks.my_saber((*ps).clientNum, 1);
                if !saber1.is_null() && (*saber1).saberFlags & SFL_NO_FLIPS != 0 {
                    allowFlips = qfalse;
                }
                if !saber2.is_null() && (*saber2).saberFlags & SFL_NO_FLIPS != 0 {
                    allowFlips = qfalse;
                }
            }

            if (*ps).groundEntityNum != ENTITYNUM_NONE || (*ps).origin[2] < (*ps).fd.forceJumpZStart
            {
                (*ps).fd.forcePowersActive &= !(1 << FP_LEVITATION);
            }

            if (*ps).fd.forcePowersActive & (1 << FP_LEVITATION) != 0 {
                //Force jump is already active.. continue draining power appropriately until we land.
                if (*ps).fd.forcePowerDebounce[FP_LEVITATION as usize] < (*pm).cmd.serverTime {
                    if (*pm).gametype == GT_DUEL as c_int || (*pm).gametype == GT_POWERDUEL as c_int
                    {
                        //jump takes less power
                        BG_ForcePowerDrain(ps, FP_LEVITATION, 1);
                    } else {
                        BG_ForcePowerDrain(ps, FP_LEVITATION, 5);
                    }
                    if (*ps).fd.forcePowerLevel[FP_LEVITATION as usize] >= FORCE_LEVEL_2 {
                        (*ps).fd.forcePowerDebounce[FP_LEVITATION as usize] =
                            (*pm).cmd.serverTime + 300;
                    } else {
                        (*ps).fd.forcePowerDebounce[FP_LEVITATION as usize] =
                            (*pm).cmd.serverTime + 200;
                    }
                }
            }

            if (*ps).forceJumpFlip != 0 {
                //Forced jump anim
                let mut anim = BOTH_FORCEINAIR1 as c_int;
                let mut parts = SETANIM_BOTH;
                if allowFlips != qfalse {
                    if (*pm).cmd.forwardmove > 0 {
                        anim = BOTH_FLIP_F as c_int;
                    } else if (*pm).cmd.forwardmove < 0 {
                        anim = BOTH_FLIP_B as c_int;
                    } else if (*pm).cmd.rightmove > 0 {
                        anim = BOTH_FLIP_R as c_int;
                    } else if (*pm).cmd.rightmove < 0 {
                        anim = BOTH_FLIP_L as c_int;
                    }
                } else {
                    if (*pm).cmd.forwardmove > 0 {
                        anim = BOTH_FORCEINAIR1 as c_int;
                    } else if (*pm).cmd.forwardmove < 0 {
                        anim = BOTH_FORCEINAIRBACK1 as c_int;
                    } else if (*pm).cmd.rightmove > 0 {
                        anim = BOTH_FORCEINAIRRIGHT1 as c_int;
                    } else if (*pm).cmd.rightmove < 0 {
                        anim = BOTH_FORCEINAIRLEFT1 as c_int;
                    }
                }
                if (*ps).weaponTime != 0 {
                    parts = SETANIM_LEGS;
                }

                self.PM_SetAnim(parts, anim, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD, 150);
                (*ps).forceJumpFlip = qfalse;
                return qtrue;
            }

            // #if METROID_JUMP (defined)
            if (*pm).waterlevel < 2 {
                if (*ps).gravity > 0 {
                    //can't do this in zero-G
                    if self.PM_ForceJumpingUp() != qfalse {
                        //holding jump in air
                        let curHeight = (*ps).origin[2] - (*ps).fd.forceJumpZStart;
                        let lvl = (*ps).fd.forcePowerLevel[FP_LEVITATION as usize] as usize;
                        //check for max force jump level and cap off & cut z vel
                        if (curHeight <= forceJumpHeight[0]
                            || ((*ps).fd.forcePower != 0 && (*pm).cmd.upmove >= 10))
                            && curHeight < forceJumpHeight[lvl]
                            && (*ps).fd.forceJumpZStart != 0.0
                        {
                            //can still go up
                            if curHeight > forceJumpHeight[0] {
                                //passed normal jump height  *2?
                                if (*ps).fd.forcePowersActive & (1 << FP_LEVITATION) == 0 {
                                    //haven't started forcejump yet
                                    (*ps).fd.forcePowersActive |= 1 << FP_LEVITATION;
                                    (*ps).fd.forceJumpSound = 1;
                                    //play flip
                                    if ((*pm).cmd.forwardmove != 0 || (*pm).cmd.rightmove != 0)
                                        && (*ps).legsAnim != BOTH_FLIP_F as c_int
                                        && (*ps).legsAnim != BOTH_FLIP_B as c_int
                                        && (*ps).legsAnim != BOTH_FLIP_R as c_int
                                        && (*ps).legsAnim != BOTH_FLIP_L as c_int
                                        && allowFlips != qfalse
                                    {
                                        let mut anim = BOTH_FORCEINAIR1 as c_int;
                                        let mut parts = SETANIM_BOTH;

                                        if (*pm).cmd.forwardmove > 0 {
                                            anim = BOTH_FLIP_F as c_int;
                                        } else if (*pm).cmd.forwardmove < 0 {
                                            anim = BOTH_FLIP_B as c_int;
                                        } else if (*pm).cmd.rightmove > 0 {
                                            anim = BOTH_FLIP_R as c_int;
                                        } else if (*pm).cmd.rightmove < 0 {
                                            anim = BOTH_FLIP_L as c_int;
                                        }
                                        if (*ps).weaponTime != 0 {
                                            parts = SETANIM_LEGS;
                                        }

                                        self.PM_SetAnim(
                                            parts,
                                            anim,
                                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                                            150,
                                        );
                                    } else if (*ps).fd.forcePowerLevel[FP_LEVITATION as usize]
                                        > FORCE_LEVEL_1
                                    {
                                        let mut facingFwd: vec3_t = [0.0; 3];
                                        let mut facingRight: vec3_t = [0.0; 3];
                                        let mut facingAngles: vec3_t = [0.0; 3];
                                        let mut anim = -1;

                                        VectorSet(
                                            &mut facingAngles,
                                            0.0,
                                            (*ps).viewangles[YAW],
                                            0.0,
                                        );

                                        AngleVectors(
                                            facingAngles,
                                            Some(&mut facingFwd),
                                            Some(&mut facingRight),
                                            None,
                                        );
                                        let dotR = facingRight[0] * (*ps).velocity[0]
                                            + facingRight[1] * (*ps).velocity[1]
                                            + facingRight[2] * (*ps).velocity[2];
                                        let dotF = facingFwd[0] * (*ps).velocity[0]
                                            + facingFwd[1] * (*ps).velocity[1]
                                            + facingFwd[2] * (*ps).velocity[2];

                                        if (dotR as f64).abs() > (dotF as f64).abs() * 1.5 {
                                            if dotR > 150.0 {
                                                anim = BOTH_FORCEJUMPRIGHT1 as c_int;
                                            } else if dotR < -150.0 {
                                                anim = BOTH_FORCEJUMPLEFT1 as c_int;
                                            }
                                        } else {
                                            if dotF > 150.0 {
                                                anim = BOTH_FORCEJUMP1 as c_int;
                                            } else if dotF < -150.0 {
                                                anim = BOTH_FORCEJUMPBACK1 as c_int;
                                            }
                                        }
                                        if anim != -1 {
                                            let mut parts = SETANIM_BOTH;
                                            if (*ps).weaponTime != 0 {
                                                parts = SETANIM_LEGS;
                                            }

                                            self.PM_SetAnim(
                                                parts,
                                                anim,
                                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                                                150,
                                            );
                                        }
                                    }
                                } else {
                                    //jump is already active (the anim has started)
                                    if (*ps).legsTimer < 1 {
                                        //not in the middle of a legsAnim
                                        let anim = (*ps).legsAnim;
                                        let mut newAnim = -1;
                                        if anim == BOTH_FORCEJUMP1 as c_int {
                                            newAnim = BOTH_FORCELAND1 as c_int;
                                        } else if anim == BOTH_FORCEJUMPBACK1 as c_int {
                                            newAnim = BOTH_FORCELANDBACK1 as c_int;
                                        } else if anim == BOTH_FORCEJUMPLEFT1 as c_int {
                                            newAnim = BOTH_FORCELANDLEFT1 as c_int;
                                        } else if anim == BOTH_FORCEJUMPRIGHT1 as c_int {
                                            newAnim = BOTH_FORCELANDRIGHT1 as c_int;
                                        }
                                        if newAnim != -1 {
                                            let mut parts = SETANIM_BOTH;
                                            if (*ps).weaponTime != 0 {
                                                parts = SETANIM_LEGS;
                                            }

                                            self.PM_SetAnim(
                                                parts,
                                                newAnim,
                                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                                                150,
                                            );
                                        }
                                    }
                                }
                            }

                            //need to scale this down, start with height velocity and scale down to regular jump vel
                            (*ps).velocity[2] = (forceJumpHeight[lvl] - curHeight)
                                / forceJumpHeight[lvl]
                                * forceJumpStrength[lvl];
                            (*ps).velocity[2] /= 10.0;
                            (*ps).velocity[2] += JUMP_VELOCITY;
                            (*ps).pm_flags |= PMF_JUMP_HELD;
                        } else if curHeight > forceJumpHeight[0]
                            && curHeight < forceJumpHeight[lvl] - forceJumpHeight[0]
                        {
                            //still have some headroom, don't totally stop it
                            if (*ps).velocity[2] > JUMP_VELOCITY {
                                (*ps).velocity[2] = JUMP_VELOCITY;
                            }
                        } else {
                            //rww - changed for the sake of balance in multiplayer
                            if (*ps).velocity[2] > JUMP_VELOCITY {
                                (*ps).velocity[2] = JUMP_VELOCITY;
                            }
                        }
                        (*pm).cmd.upmove = 0;
                        return qfalse;
                    }
                }
            }
            // #endif

            //Not jumping
            if (*pm).cmd.upmove < 10 && (*ps).groundEntityNum != ENTITYNUM_NONE {
                return qfalse;
            }

            // must wait for jump to be released
            if (*ps).pm_flags & PMF_JUMP_HELD != 0 {
                // clear upmove so cmdscale doesn't lower running speed
                (*pm).cmd.upmove = 0;
                return qfalse;
            }

            if (*ps).gravity <= 0 {
                //in low grav, you push in the dir you're facing as long as there is something behind you to shove off of
                let mut forward: vec3_t = [0.0; 3];
                let mut back: vec3_t = [0.0; 3];
                let mut trace: trace_t = core::mem::zeroed();

                AngleVectors((*ps).viewangles, Some(&mut forward), None, None);
                _VectorMA((*ps).origin, -8.0, forward, &mut back);
                self.traps.trace(
                    &mut trace,
                    core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                    core::ptr::addr_of!((*pm).mins) as *const vec3_t,
                    core::ptr::addr_of!((*pm).maxs) as *const vec3_t,
                    core::ptr::addr_of!(back) as *const vec3_t,
                    (*ps).clientNum,
                    (*pm).tracemask,
                );

                if trace.fraction <= 1.0 {
                    let v = (*ps).velocity;
                    _VectorMA(v, JUMP_VELOCITY * 2.0, forward, &mut (*ps).velocity);
                    self.PM_SetAnim(
                        SETANIM_LEGS,
                        BOTH_FORCEJUMP1 as c_int,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_RESTART,
                        150,
                    );
                } //else no surf close enough to push off of
                (*pm).cmd.upmove = 0;
            } else if (*pm).cmd.upmove > 0
                && (*pm).waterlevel < 2
                && (*ps).fd.forcePowerLevel[FP_LEVITATION as usize] > FORCE_LEVEL_0
                && (*ps).pm_flags & PMF_JUMP_HELD == 0
                && ((*ps).weapon == WP_SABER as c_int || (*ps).weapon == WP_MELEE as c_int)
                && PM_IsRocketTrooper() == qfalse
                && BG_HasYsalamiri((*pm).gametype, ps) == qfalse
                && BG_CanUseFPNow((*pm).gametype, ps, (*pm).cmd.serverTime, FP_LEVITATION) != qfalse
            {
                let mut allowWallRuns = qtrue;
                let mut allowWallFlips = qtrue;
                let mut allowFlips = qtrue;
                let mut allowWallGrabs = qtrue;
                if (*ps).weapon == WP_SABER as c_int {
                    let saber1 = self.callbacks.my_saber((*ps).clientNum, 0);
                    let saber2 = self.callbacks.my_saber((*ps).clientNum, 1);
                    if !saber1.is_null() && (*saber1).saberFlags & SFL_NO_WALL_RUNS != 0 {
                        allowWallRuns = qfalse;
                    }
                    if !saber2.is_null() && (*saber2).saberFlags & SFL_NO_WALL_RUNS != 0 {
                        allowWallRuns = qfalse;
                    }
                    if !saber1.is_null() && (*saber1).saberFlags & SFL_NO_WALL_FLIPS != 0 {
                        allowWallFlips = qfalse;
                    }
                    if !saber2.is_null() && (*saber2).saberFlags & SFL_NO_WALL_FLIPS != 0 {
                        allowWallFlips = qfalse;
                    }
                    if !saber1.is_null() && (*saber1).saberFlags & SFL_NO_FLIPS != 0 {
                        allowFlips = qfalse;
                    }
                    if !saber2.is_null() && (*saber2).saberFlags & SFL_NO_FLIPS != 0 {
                        allowFlips = qfalse;
                    }
                    if !saber1.is_null() && (*saber1).saberFlags & SFL_NO_WALL_GRAB != 0 {
                        allowWallGrabs = qfalse;
                    }
                    if !saber2.is_null() && (*saber2).saberFlags & SFL_NO_WALL_GRAB != 0 {
                        allowWallGrabs = qfalse;
                    }
                }

                if (*ps).groundEntityNum != ENTITYNUM_NONE {
                    //on the ground
                    //check for left-wall and right-wall special jumps
                    let mut anim = -1;
                    let mut vertPush = 0.0f32;
                    if (*pm).cmd.rightmove > 0
                        && (*ps).fd.forcePowerLevel[FP_LEVITATION as usize] > FORCE_LEVEL_1
                    {
                        //strafing right
                        if (*pm).cmd.forwardmove > 0 {
                            //wall-run
                            if allowWallRuns != qfalse {
                                vertPush = forceJumpStrength[FORCE_LEVEL_2 as usize] / 2.0;
                                anim = BOTH_WALL_RUN_RIGHT as c_int;
                            }
                        } else if (*pm).cmd.forwardmove == 0 {
                            //wall-flip
                            if allowWallFlips != qfalse {
                                vertPush = forceJumpStrength[FORCE_LEVEL_2 as usize] / 2.25;
                                anim = BOTH_WALL_FLIP_RIGHT as c_int;
                            }
                        }
                    } else if (*pm).cmd.rightmove < 0
                        && (*ps).fd.forcePowerLevel[FP_LEVITATION as usize] > FORCE_LEVEL_1
                    {
                        //strafing left
                        if (*pm).cmd.forwardmove > 0 {
                            //wall-run
                            if allowWallRuns != qfalse {
                                vertPush = forceJumpStrength[FORCE_LEVEL_2 as usize] / 2.0;
                                anim = BOTH_WALL_RUN_LEFT as c_int;
                            }
                        } else if (*pm).cmd.forwardmove == 0 {
                            //wall-flip
                            if allowWallFlips != qfalse {
                                vertPush = forceJumpStrength[FORCE_LEVEL_2 as usize] / 2.25;
                                anim = BOTH_WALL_FLIP_LEFT as c_int;
                            }
                        }
                    } else if (*pm).cmd.forwardmove < 0 && (*pm).cmd.buttons & BUTTON_ATTACK == 0 {
                        //backflip
                        if allowFlips != qfalse {
                            vertPush = JUMP_VELOCITY;
                            anim = BOTH_FLIP_BACK1 as c_int;
                        }
                    }

                    vertPush += 128.0; //give them an extra shove

                    if anim != -1 {
                        let mut fwd: vec3_t = [0.0; 3];
                        let mut right: vec3_t = [0.0; 3];
                        let mut traceto: vec3_t = [0.0; 3];
                        let mut mins: vec3_t = [0.0; 3];
                        let mut maxs: vec3_t = [0.0; 3];
                        let mut fwdAngles: vec3_t = [0.0; 3];
                        let mut idealNormal: vec3_t = [0.0; 3];
                        let mut wallNormal: vec3_t = [0.0; 3];
                        let mut trace: trace_t = core::mem::zeroed();
                        let mut doTrace = qfalse;
                        let contents = MASK_SOLID;

                        VectorSet(&mut mins, (*pm).mins[0], (*pm).mins[1], 0.0);
                        VectorSet(&mut maxs, (*pm).maxs[0], (*pm).maxs[1], 24.0);
                        VectorSet(&mut fwdAngles, 0.0, (*ps).viewangles[YAW], 0.0);

                        AngleVectors(fwdAngles, Some(&mut fwd), Some(&mut right), None);

                        //trace-check for a wall, if necc.
                        if anim == BOTH_WALL_FLIP_LEFT as c_int
                            || anim == BOTH_WALL_RUN_LEFT as c_int
                        {
                            doTrace = qtrue;
                            _VectorMA((*ps).origin, -16.0, right, &mut traceto);
                        } else if anim == BOTH_WALL_FLIP_RIGHT as c_int
                            || anim == BOTH_WALL_RUN_RIGHT as c_int
                        {
                            doTrace = qtrue;
                            _VectorMA((*ps).origin, 16.0, right, &mut traceto);
                        } else if anim == BOTH_WALL_FLIP_BACK1 as c_int {
                            doTrace = qtrue;
                            _VectorMA((*ps).origin, 16.0, fwd, &mut traceto);
                        }

                        if doTrace != qfalse {
                            self.traps.trace(
                                &mut trace,
                                core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                                core::ptr::addr_of!(mins) as *const vec3_t,
                                core::ptr::addr_of!(maxs) as *const vec3_t,
                                core::ptr::addr_of!(traceto) as *const vec3_t,
                                (*ps).clientNum,
                                contents,
                            );
                            _VectorCopy(trace.plane.normal, &mut wallNormal);
                            VectorNormalize(&mut wallNormal);
                            _VectorSubtract((*ps).origin, traceto, &mut idealNormal);
                            VectorNormalize(&mut idealNormal);
                        }

                        if doTrace == qfalse
                            || (trace.fraction < 1.0
                                && ((trace.entityNum as c_int) < MAX_CLIENTS as c_int
                                    || (wallNormal[0] * idealNormal[0]
                                        + wallNormal[1] * idealNormal[1]
                                        + wallNormal[2] * idealNormal[2])
                                        > 0.7))
                        {
                            //there is a wall there.. or hit a client
                            if (anim != BOTH_WALL_RUN_LEFT as c_int
                                && anim != BOTH_WALL_RUN_RIGHT as c_int
                                && anim != BOTH_FORCEWALLRUNFLIP_START as c_int)
                                || (wallNormal[2] >= 0.0 && wallNormal[2] <= 0.4)
                            {
                                //wall-runs can only run on perfectly flat walls, sorry.
                                let mut parts;
                                //move me to side
                                if anim == BOTH_WALL_FLIP_LEFT as c_int {
                                    (*ps).velocity[0] = 0.0;
                                    (*ps).velocity[1] = 0.0;
                                    let v = (*ps).velocity;
                                    _VectorMA(v, 150.0, right, &mut (*ps).velocity);
                                } else if anim == BOTH_WALL_FLIP_RIGHT as c_int {
                                    (*ps).velocity[0] = 0.0;
                                    (*ps).velocity[1] = 0.0;
                                    let v = (*ps).velocity;
                                    _VectorMA(v, -150.0, right, &mut (*ps).velocity);
                                } else if anim == BOTH_FLIP_BACK1 as c_int
                                    || anim == BOTH_FLIP_BACK2 as c_int
                                    || anim == BOTH_FLIP_BACK3 as c_int
                                    || anim == BOTH_WALL_FLIP_BACK1 as c_int
                                {
                                    (*ps).velocity[0] = 0.0;
                                    (*ps).velocity[1] = 0.0;
                                    let v = (*ps).velocity;
                                    _VectorMA(v, -150.0, fwd, &mut (*ps).velocity);
                                }

                                //up
                                if vertPush != 0.0 {
                                    (*ps).velocity[2] = vertPush;
                                    (*ps).fd.forcePowersActive |= 1 << FP_LEVITATION;
                                }
                                //animate me
                                parts = SETANIM_LEGS;
                                if anim == BOTH_BUTTERFLY_LEFT as c_int {
                                    parts = SETANIM_BOTH;
                                    (*pm).cmd.buttons &= !BUTTON_ATTACK;
                                    (*ps).saberMove = LS_NONE as c_int;
                                } else if (*ps).weaponTime == 0 {
                                    parts = SETANIM_BOTH;
                                }
                                self.PM_SetAnim(
                                    parts,
                                    anim,
                                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                                    0,
                                );
                                if anim == BOTH_BUTTERFLY_LEFT as c_int {
                                    (*ps).weaponTime = (*ps).torsoTimer;
                                }
                                self.PM_SetForceJumpZStart((*ps).origin[2]); //so we don't take damage if we land at same height
                                (*ps).pm_flags |= PMF_JUMP_HELD;
                                (*pm).cmd.upmove = 0;
                                (*ps).fd.forceJumpSound = 1;
                            }
                        }
                    }
                } else {
                    //in the air
                    let legsAnim = (*ps).legsAnim;

                    if legsAnim == BOTH_WALL_RUN_LEFT as c_int
                        || legsAnim == BOTH_WALL_RUN_RIGHT as c_int
                    {
                        //running on a wall
                        let mut right: vec3_t = [0.0; 3];
                        let mut traceto: vec3_t = [0.0; 3];
                        let mut mins: vec3_t = [0.0; 3];
                        let mut maxs: vec3_t = [0.0; 3];
                        let mut fwdAngles: vec3_t = [0.0; 3];
                        let mut trace: trace_t = core::mem::zeroed();
                        let mut anim = -1;

                        VectorSet(&mut mins, (*pm).mins[0], (*pm).mins[0], 0.0);
                        VectorSet(&mut maxs, (*pm).maxs[0], (*pm).maxs[0], 24.0);
                        VectorSet(&mut fwdAngles, 0.0, (*ps).viewangles[YAW], 0.0);

                        AngleVectors(fwdAngles, None, Some(&mut right), None);

                        if legsAnim == BOTH_WALL_RUN_LEFT as c_int {
                            if (*ps).legsTimer > 400 {
                                //not at the end of the anim
                                let animLen =
                                    self.PM_AnimLength(0, BOTH_WALL_RUN_LEFT as c_int) as f32;
                                if ((*ps).legsTimer as f32) < animLen - 400.0 {
                                    //not at start of anim
                                    _VectorMA((*ps).origin, -16.0, right, &mut traceto);
                                    anim = BOTH_WALL_RUN_LEFT_FLIP as c_int;
                                }
                            }
                        } else if legsAnim == BOTH_WALL_RUN_RIGHT as c_int {
                            if (*ps).legsTimer > 400 {
                                //not at the end of the anim
                                let animLen =
                                    self.PM_AnimLength(0, BOTH_WALL_RUN_RIGHT as c_int) as f32;
                                if ((*ps).legsTimer as f32) < animLen - 400.0 {
                                    //not at start of anim
                                    _VectorMA((*ps).origin, 16.0, right, &mut traceto);
                                    anim = BOTH_WALL_RUN_RIGHT_FLIP as c_int;
                                }
                            }
                        }
                        if anim != -1 {
                            self.traps.trace(
                                &mut trace,
                                core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                                core::ptr::addr_of!(mins) as *const vec3_t,
                                core::ptr::addr_of!(maxs) as *const vec3_t,
                                core::ptr::addr_of!(traceto) as *const vec3_t,
                                (*ps).clientNum,
                                CONTENTS_SOLID | CONTENTS_BODY,
                            );
                            if trace.fraction < 1.0 {
                                //flip off wall
                                let mut parts;

                                if anim == BOTH_WALL_RUN_LEFT_FLIP as c_int {
                                    (*ps).velocity[0] *= 0.5;
                                    (*ps).velocity[1] *= 0.5;
                                    let v = (*ps).velocity;
                                    _VectorMA(v, 150.0, right, &mut (*ps).velocity);
                                } else if anim == BOTH_WALL_RUN_RIGHT_FLIP as c_int {
                                    (*ps).velocity[0] *= 0.5;
                                    (*ps).velocity[1] *= 0.5;
                                    let v = (*ps).velocity;
                                    _VectorMA(v, -150.0, right, &mut (*ps).velocity);
                                }
                                parts = SETANIM_LEGS;
                                if (*ps).weaponTime == 0 {
                                    parts = SETANIM_BOTH;
                                }
                                self.PM_SetAnim(
                                    parts,
                                    anim,
                                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                                    0,
                                );
                                (*pm).cmd.upmove = 0;
                            }
                        }
                        if (*pm).cmd.upmove != 0 {
                            //jump failed, so don't try to do normal jump code, just return
                            return qfalse;
                        }
                    }
                    //NEW JKA
                    else if (*ps).legsAnim == BOTH_FORCEWALLRUNFLIP_START as c_int {
                        let mut fwd: vec3_t = [0.0; 3];
                        let mut traceto: vec3_t = [0.0; 3];
                        let mut mins: vec3_t = [0.0; 3];
                        let mut maxs: vec3_t = [0.0; 3];
                        let mut fwdAngles: vec3_t = [0.0; 3];
                        let mut trace: trace_t = core::mem::zeroed();
                        let mut anim = -1;

                        VectorSet(&mut mins, (*pm).mins[0], (*pm).mins[0], 0.0);
                        VectorSet(&mut maxs, (*pm).maxs[0], (*pm).maxs[0], 24.0);
                        VectorSet(&mut fwdAngles, 0.0, (*ps).viewangles[YAW], 0.0);
                        AngleVectors(fwdAngles, Some(&mut fwd), None, None);

                        let animLen = self.BG_AnimLength(
                            (*self.pm_entSelf).localAnimIndex,
                            BOTH_FORCEWALLRUNFLIP_START as c_int,
                        ) as f32;
                        if ((*ps).legsTimer as f32) < animLen - 400.0 {
                            //not at start of anim
                            _VectorMA((*ps).origin, 16.0, fwd, &mut traceto);
                            anim = BOTH_FORCEWALLRUNFLIP_END as c_int;
                        }
                        if anim != -1 {
                            self.traps.trace(
                                &mut trace,
                                core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                                core::ptr::addr_of!(mins) as *const vec3_t,
                                core::ptr::addr_of!(maxs) as *const vec3_t,
                                core::ptr::addr_of!(traceto) as *const vec3_t,
                                (*ps).clientNum,
                                CONTENTS_SOLID | CONTENTS_BODY,
                            );
                            if trace.fraction < 1.0 {
                                //flip off wall
                                let mut parts = SETANIM_LEGS;

                                (*ps).velocity[0] *= 0.5;
                                (*ps).velocity[1] *= 0.5;
                                let v = (*ps).velocity;
                                _VectorMA(v, -300.0, fwd, &mut (*ps).velocity);
                                (*ps).velocity[2] += 200.0;
                                if (*ps).weaponTime == 0 {
                                    //not attacking, set anim on both
                                    parts = SETANIM_BOTH;
                                }
                                self.PM_SetAnim(
                                    parts,
                                    anim,
                                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                                    0,
                                );
                                (*pm).cmd.upmove = 0;
                                self.PM_AddEvent(EV_JUMP as c_int);
                            }
                        }
                        if (*pm).cmd.upmove != 0 {
                            //jump failed, so don't try to do normal jump code, just return
                            return qfalse;
                        }
                    } else if (*pm).cmd.forwardmove > 0 //pushing forward
                        && (*ps).fd.forceRageRecoveryTime < (*pm).cmd.serverTime //not in a force Rage recovery period
                        && (*ps).fd.forcePowerLevel[FP_LEVITATION as usize] > FORCE_LEVEL_1
                        && self.PM_WalkableGroundDistance() <= 80.0
                        && ((*ps).legsAnim == BOTH_JUMP1 as c_int
                            || (*ps).legsAnim == BOTH_INAIR1 as c_int)
                    {
                        //run up wall, flip backwards
                        if allowWallRuns != qfalse {
                            let mut wallWalkAnim = BOTH_WALL_FLIP_BACK1 as c_int;
                            let mut parts = SETANIM_LEGS;
                            let contents = MASK_SOLID;
                            if (*ps).fd.forcePowerLevel[FP_LEVITATION as usize] > FORCE_LEVEL_2 {
                                wallWalkAnim = BOTH_FORCEWALLRUNFLIP_START as c_int;
                                parts = SETANIM_BOTH;
                            } else {
                                if (*ps).weaponTime == 0 {
                                    parts = SETANIM_BOTH;
                                }
                            }
                            if true {
                                let mut fwd: vec3_t = [0.0; 3];
                                let mut traceto: vec3_t = [0.0; 3];
                                let mut mins: vec3_t = [0.0; 3];
                                let mut maxs: vec3_t = [0.0; 3];
                                let mut fwdAngles: vec3_t = [0.0; 3];
                                let mut trace: trace_t = core::mem::zeroed();
                                let mut idealNormal: vec3_t = [0.0; 3];

                                VectorSet(&mut mins, (*pm).mins[0], (*pm).mins[1], 0.0);
                                VectorSet(&mut maxs, (*pm).maxs[0], (*pm).maxs[1], 24.0);
                                VectorSet(&mut fwdAngles, 0.0, (*ps).viewangles[YAW], 0.0);

                                AngleVectors(fwdAngles, Some(&mut fwd), None, None);
                                _VectorMA((*ps).origin, 32.0, fwd, &mut traceto);

                                self.traps.trace(
                                    &mut trace,
                                    core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                                    core::ptr::addr_of!(mins) as *const vec3_t,
                                    core::ptr::addr_of!(maxs) as *const vec3_t,
                                    core::ptr::addr_of!(traceto) as *const vec3_t,
                                    (*ps).clientNum,
                                    contents,
                                );
                                _VectorSubtract((*ps).origin, traceto, &mut idealNormal);
                                VectorNormalize(&mut idealNormal);
                                let traceEnt = self.PM_BGEntForNum(trace.entityNum as c_int);

                                if trace.fraction < 1.0
                                    && (((trace.entityNum as c_int) < ENTITYNUM_WORLD
                                        && !traceEnt.is_null()
                                        && (*traceEnt).s.solid != SOLID_BMODEL)
                                        || (trace.plane.normal[0] * idealNormal[0]
                                            + trace.plane.normal[1] * idealNormal[1]
                                            + trace.plane.normal[2] * idealNormal[2])
                                            > 0.7)
                                {
                                    //there is a wall there
                                    (*ps).velocity[0] = 0.0;
                                    (*ps).velocity[1] = 0.0;
                                    if wallWalkAnim == BOTH_FORCEWALLRUNFLIP_START as c_int {
                                        (*ps).velocity[2] =
                                            forceJumpStrength[FORCE_LEVEL_3 as usize] / 2.0;
                                    } else {
                                        let v = (*ps).velocity;
                                        _VectorMA(v, -150.0, fwd, &mut (*ps).velocity);
                                        (*ps).velocity[2] += 150.0;
                                    }
                                    //animate me
                                    self.PM_SetAnim(
                                        parts,
                                        wallWalkAnim,
                                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                                        0,
                                    );
                                    self.PM_SetForceJumpZStart((*ps).origin[2]); //so we don't take damage if we land at same height
                                    (*pm).cmd.upmove = 0;
                                    (*ps).fd.forceJumpSound = 1;
                                    BG_ForcePowerDrain(ps, FP_LEVITATION, 5);

                                    (*pm).cmd.rightmove = 0;
                                    (*pm).cmd.forwardmove = 0;
                                }
                            }
                        }
                    } else if (BG_InSpecialJump(legsAnim) == qfalse //not in a special jump anim
                        || BG_InReboundJump(legsAnim) != qfalse //we're already in a rebound
                        || BG_InBackFlip(legsAnim) != qfalse)//a backflip
                        && (*ps).velocity[2] > -1200.0 //not falling down very fast
                        && (*ps).pm_flags & PMF_JUMP_HELD == 0 //have to have released jump since last press
                        && ((*pm).cmd.forwardmove != 0 || (*pm).cmd.rightmove != 0) //pushing in a direction
                        && (*ps).fd.forcePowerLevel[FP_LEVITATION as usize] > FORCE_LEVEL_2 //level 3 jump or better
                        && BG_CanUseFPNow((*pm).gametype, ps, (*pm).cmd.serverTime, FP_LEVITATION)
                            != qfalse
                        && ((*ps).origin[2] - (*ps).fd.forceJumpZStart)
                            < (forceJumpHeightMax[FORCE_LEVEL_3 as usize]
                                - (BG_ForceWallJumpStrength() / 2.0))
                    {
                        //see if we're pushing at a wall and jump off it if so
                        if allowWallGrabs != qfalse {
                            let mut checkDir: vec3_t = [0.0; 3];
                            let mut traceto: vec3_t = [0.0; 3];
                            let mut mins: vec3_t = [0.0; 3];
                            let mut maxs: vec3_t = [0.0; 3];
                            let mut fwdAngles: vec3_t = [0.0; 3];
                            let mut trace: trace_t = core::mem::zeroed();
                            let mut idealNormal: vec3_t = [0.0; 3];
                            let mut anim = -1;

                            VectorSet(&mut mins, (*pm).mins[0], (*pm).mins[1], 0.0);
                            VectorSet(&mut maxs, (*pm).maxs[0], (*pm).maxs[1], 24.0);
                            VectorSet(&mut fwdAngles, 0.0, (*ps).viewangles[YAW], 0.0);

                            if (*pm).cmd.rightmove != 0 {
                                if (*pm).cmd.rightmove > 0 {
                                    anim = BOTH_FORCEWALLREBOUND_RIGHT as c_int;
                                    AngleVectors(fwdAngles, None, Some(&mut checkDir), None);
                                } else if (*pm).cmd.rightmove < 0 {
                                    anim = BOTH_FORCEWALLREBOUND_LEFT as c_int;
                                    AngleVectors(fwdAngles, None, Some(&mut checkDir), None);
                                    let c = checkDir;
                                    _VectorScale(c, -1.0, &mut checkDir);
                                }
                            } else if (*pm).cmd.forwardmove > 0 {
                                anim = BOTH_FORCEWALLREBOUND_FORWARD as c_int;
                                AngleVectors(fwdAngles, Some(&mut checkDir), None, None);
                            } else if (*pm).cmd.forwardmove < 0 {
                                anim = BOTH_FORCEWALLREBOUND_BACK as c_int;
                                AngleVectors(fwdAngles, Some(&mut checkDir), None, None);
                                let c = checkDir;
                                _VectorScale(c, -1.0, &mut checkDir);
                            }
                            if anim != -1 {
                                //trace in the dir we're pushing in and see if there's a vertical wall there
                                _VectorMA((*ps).origin, 8.0, checkDir, &mut traceto);
                                self.traps.trace(
                                    &mut trace,
                                    core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                                    core::ptr::addr_of!(mins) as *const vec3_t,
                                    core::ptr::addr_of!(maxs) as *const vec3_t,
                                    core::ptr::addr_of!(traceto) as *const vec3_t,
                                    (*ps).clientNum,
                                    CONTENTS_SOLID,
                                );
                                _VectorSubtract((*ps).origin, traceto, &mut idealNormal);
                                VectorNormalize(&mut idealNormal);
                                let traceEnt = self.PM_BGEntForNum(trace.entityNum as c_int);
                                if trace.fraction < 1.0
                                    && Q_fabs(trace.plane.normal[2]) <= 0.2
                                    && (((trace.entityNum as c_int) < ENTITYNUM_WORLD
                                        && !traceEnt.is_null()
                                        && (*traceEnt).s.solid != SOLID_BMODEL)
                                        || (trace.plane.normal[0] * idealNormal[0]
                                            + trace.plane.normal[1] * idealNormal[1]
                                            + trace.plane.normal[2] * idealNormal[2])
                                            > 0.7)
                                {
                                    //there is a wall there
                                    let dot = (*ps).velocity[0] * trace.plane.normal[0]
                                        + (*ps).velocity[1] * trace.plane.normal[1]
                                        + (*ps).velocity[2] * trace.plane.normal[2];
                                    if dot < 1.0 {
                                        //can't be heading *away* from the wall!
                                        //grab it!
                                        self.PM_GrabWallForJump(anim);
                                    }
                                }
                            }
                        }
                    } else {
                        //FIXME: if in a butterfly, kick people away?
                    }
                    //END NEW JKA
                }
            }

            if (*ps).groundEntityNum == ENTITYNUM_NONE {
                return qfalse;
            }
            if (*pm).cmd.upmove > 0 {
                //no special jumps
                (*ps).velocity[2] = JUMP_VELOCITY;
                self.PM_SetForceJumpZStart((*ps).origin[2]); //so we don't take damage if we land at same height
                (*ps).pm_flags |= PMF_JUMP_HELD;
            }

            //Jumping
            self.pml.groundPlane = qfalse;
            self.pml.walking = qfalse;
            (*ps).pm_flags |= PMF_JUMP_HELD;
            (*ps).groundEntityNum = ENTITYNUM_NONE;
            self.PM_SetForceJumpZStart((*ps).origin[2]);

            self.PM_AddEvent(EV_JUMP as c_int);

            //Set the animations
            if (*ps).gravity > 0 && BG_InSpecialJump((*ps).legsAnim) == qfalse {
                self.PM_JumpForDir();
            }

            qtrue
        }
    }
}

impl PmoveContext<'_> {
    /// Raven `PM_CheckWaterJump`.
    /// Source: `oracle/codemp/game/bg_pmove.c:2781-2821`
    pub fn PM_CheckWaterJump(&mut self) -> qboolean {
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let mut spot: vec3_t = [0.0; 3];
            let mut flatforward: vec3_t = [0.0; 3];

            if (*ps).pm_time != 0 {
                return qfalse;
            }

            // check for water jump
            if (*pm).waterlevel != 2 {
                return qfalse;
            }

            flatforward[0] = self.pml.forward[0];
            flatforward[1] = self.pml.forward[1];
            flatforward[2] = 0.0;
            VectorNormalize(&mut flatforward);

            _VectorMA((*ps).origin, 30.0, flatforward, &mut spot);
            spot[2] += 4.0;
            let mut cont = self
                .traps
                .pointcontents(core::ptr::addr_of!(spot), (*ps).clientNum);
            if cont & CONTENTS_SOLID == 0 {
                return qfalse;
            }

            spot[2] += 16.0;
            cont = self
                .traps
                .pointcontents(core::ptr::addr_of!(spot), (*ps).clientNum);
            if cont != 0 {
                return qfalse;
            }

            // jump out of water
            _VectorScale(self.pml.forward, 200.0, &mut (*ps).velocity);
            (*ps).velocity[2] = 350.0;

            (*ps).pm_flags |= PMF_TIME_WATERJUMP;
            (*ps).pm_time = 2000;

            qtrue
        }
    }

    /// Raven `PM_WaterJumpMove`.
    /// Source: `oracle/codemp/game/bg_pmove.c:2833-2844`
    pub fn PM_WaterJumpMove(&mut self) {
        unsafe {
            // waterjump has no control, but falls
            self.PM_StepSlideMove(qtrue);

            let ps = (*self.pm).ps;
            (*ps).velocity[2] -= (*ps).gravity as f32 * self.pml.frametime;
            if (*ps).velocity[2] < 0.0 {
                // cancel as soon as we are falling down again
                (*ps).pm_flags &= !PMF_ALL_TIMES;
                (*ps).pm_time = 0;
            }
        }
    }

    /// Raven `PM_WaterMove`.
    /// Source: `oracle/codemp/game/bg_pmove.c:2852-2916`
    pub fn PM_WaterMove(&mut self) {
        use crate::bg_slidemove::OVERCLIP;
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let mut wishvel: vec3_t = [0.0; 3];
            let mut wishdir: vec3_t = [0.0; 3];

            if self.PM_CheckWaterJump() != qfalse {
                self.PM_WaterJumpMove();
                return;
            }

            self.PM_Friction();

            let scale = self.PM_CmdScale(core::ptr::addr_of_mut!((*pm).cmd));
            //
            // user intentions
            //
            if scale == 0.0 {
                wishvel[0] = 0.0;
                wishvel[1] = 0.0;
                wishvel[2] = -60.0; // sink towards bottom
            } else {
                for i in 0..3 {
                    wishvel[i] = scale * self.pml.forward[i] * (*pm).cmd.forwardmove as f32
                        + scale * self.pml.right[i] * (*pm).cmd.rightmove as f32;
                }
                wishvel[2] += scale * (*pm).cmd.upmove as f32;
            }

            _VectorCopy(wishvel, &mut wishdir);
            let mut wishspeed = VectorNormalize(&mut wishdir);

            if wishspeed > (*ps).speed * pm_swimScale {
                wishspeed = (*ps).speed * pm_swimScale;
            }

            self.PM_Accelerate(wishdir, wishspeed, pm_wateraccelerate);

            // make sure we can go up slopes easily under water
            let dp = (*ps).velocity[0] * self.pml.groundTrace.plane.normal[0]
                + (*ps).velocity[1] * self.pml.groundTrace.plane.normal[1]
                + (*ps).velocity[2] * self.pml.groundTrace.plane.normal[2];
            if self.pml.groundPlane != qfalse && dp < 0.0 {
                let vel = VectorLength((*ps).velocity);
                // slide along the ground plane
                let inp = (*ps).velocity;
                let norm = self.pml.groundTrace.plane.normal;
                self.PM_ClipVelocity(inp, norm, &mut (*ps).velocity, OVERCLIP);

                VectorNormalize(&mut (*ps).velocity);
                let v = (*ps).velocity;
                _VectorScale(v, vel, &mut (*ps).velocity);
            }

            self.PM_SlideMove(qfalse);
        }
    }

    /// Raven `PM_FlyVehicleMove`.
    /// Source: `oracle/codemp/game/bg_pmove.c:2924-3012`
    pub fn PM_FlyVehicleMove(&mut self) {
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let mut wishvel: vec3_t = [0.0; 3];
            let mut wishdir: vec3_t = [0.0; 3];
            let mut wishspeed;
            let fmove = 0.0f32;
            let smove = 0.0f32;

            // normal slowdown
            if (*ps).gravity != 0
                && (*ps).velocity[2] < 0.0
                && (*ps).groundEntityNum == ENTITYNUM_NONE
            {
                //falling
                let zVel = (*ps).velocity[2];
                self.PM_Friction();
                (*ps).velocity[2] = zVel;
            } else {
                self.PM_Friction();
                if (*ps).velocity[2] < 0.0 && (*ps).groundEntityNum != ENTITYNUM_NONE {
                    (*ps).velocity[2] = 0.0; // ignore slope movement
                }
            }

            let scale = self.PM_CmdScale(core::ptr::addr_of_mut!((*pm).cmd));

            // Get The WishVel And WishSpeed
            if (*ps).clientNum >= MAX_CLIENTS as c_int {
                //NPC
                if (fmove != 0.0 || smove != 0.0) && VectorCompare((*ps).moveDir, vec3_origin) {
                    for i in 0..3 {
                        wishvel[i] = self.pml.forward[i] * fmove + self.pml.right[i] * smove;
                    }

                    _VectorCopy(wishvel, &mut wishdir);
                    wishspeed = VectorNormalize(&mut wishdir);
                    wishspeed *= scale;
                } else {
                    wishspeed = (*ps).speed;
                    _VectorScale((*ps).moveDir, (*ps).speed, &mut wishvel);
                    _VectorCopy((*ps).moveDir, &mut wishdir);
                }
            } else {
                for i in 0..3 {
                    wishvel[i] = self.pml.forward[i] * fmove + self.pml.right[i] * smove;
                }
                _VectorCopy(wishvel, &mut wishdir);
                wishspeed = VectorNormalize(&mut wishdir);
                wishspeed *= scale;
            }

            // Handle negative speed.
            if wishspeed < 0.0 {
                wishspeed = wishspeed * -1.0;
                let wv = wishvel;
                _VectorScale(wv, -1.0, &mut wishvel);
                let wd = wishdir;
                _VectorScale(wd, -1.0, &mut wishdir);
            }

            _VectorCopy(wishvel, &mut wishdir);
            wishspeed = VectorNormalize(&mut wishdir);

            self.PM_Accelerate(wishdir, wishspeed, 100.0);

            self.PM_StepSlideMove(qtrue);
        }
    }

    /// Raven `PM_FlyMove`.
    /// Source: `oracle/codemp/game/bg_pmove.c:3021-3059`
    pub fn PM_FlyMove(&mut self) {
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let mut wishvel: vec3_t = [0.0; 3];
            let mut wishdir: vec3_t = [0.0; 3];

            // normal slowdown
            self.PM_Friction();

            let mut scale = self.PM_CmdScale(core::ptr::addr_of_mut!((*pm).cmd));

            if (*ps).pm_type == PM_SPECTATOR as c_int && (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0
            {
                //turbo boost
                scale *= 10.0;
            }

            //
            // user intentions
            //
            if scale == 0.0 {
                wishvel[0] = 0.0;
                wishvel[1] = 0.0;
                wishvel[2] = (*ps).speed * ((*pm).cmd.upmove as f32 / 127.0);
            } else {
                for i in 0..3 {
                    wishvel[i] = scale * self.pml.forward[i] * (*pm).cmd.forwardmove as f32
                        + scale * self.pml.right[i] * (*pm).cmd.rightmove as f32;
                }
                wishvel[2] += scale * (*pm).cmd.upmove as f32;
            }

            _VectorCopy(wishvel, &mut wishdir);
            let wishspeed = VectorNormalize(&mut wishdir);

            self.PM_Accelerate(wishdir, wishspeed, pm_flyaccelerate);

            self.PM_StepSlideMove(qfalse);
        }
    }
}

impl PmoveContext<'_> {
    /// Raven `PM_AirMove`. `METROID_JUMP` is defined, so the unconditional `PM_CheckJump`
    /// path is compiled. The `#if 0` hover strafe block is dropped (dead).
    /// Source: `oracle/codemp/game/bg_pmove.c:3068-3297`
    pub fn PM_AirMove(&mut self) {
        use crate::bg_slidemove::OVERCLIP;
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let mut wishvel: vec3_t = [0.0; 3];
            let mut wishdir: vec3_t = [0.0; 3];
            let mut wishspeed;
            let mut pVeh: *mut Vehicle_t = core::ptr::null_mut();

            if (*ps).clientNum >= MAX_CLIENTS as c_int {
                let pEnt = self.pm_entSelf;
                if !pEnt.is_null() && (*pEnt).s.NPC_class == CLASS_VEHICLE as c_int {
                    pVeh = (*pEnt).m_pVehicle as *mut Vehicle_t;
                }
            }

            if (*ps).pm_type != PM_SPECTATOR as c_int {
                // #if METROID_JUMP
                self.PM_CheckJump();
            }
            self.PM_Friction();

            let fmove = (*pm).cmd.forwardmove as f32;
            let smove = (*pm).cmd.rightmove as f32;

            let mut scale = self.PM_CmdScale(core::ptr::addr_of_mut!((*pm).cmd));

            // set the movementDir so clients can rotate the legs for strafing
            self.PM_SetMovementDir();

            // project moves down to flat plane
            self.pml.forward[2] = 0.0;
            self.pml.right[2] = 0.0;
            VectorNormalize(&mut self.pml.forward);
            VectorNormalize(&mut self.pml.right);

            if !pVeh.is_null() && (*(*pVeh).m_pVehicleInfo).hoverHeight > 0.0 {
                //in a hovering vehicle, have air control
                wishspeed = (*ps).speed;
                _VectorScale((*ps).moveDir, (*ps).speed, &mut wishvel);
                _VectorCopy((*ps).moveDir, &mut wishdir);
                scale = 1.0;
            } else if self.gPMDoSlowFall != qfalse {
                //no air-control
                VectorClear(&mut wishvel);
            } else if (*ps).pm_type == PM_JETPACK as c_int {
                //reduced air control while not jetting
                for i in 0..2 {
                    wishvel[i] = self.pml.forward[i] * fmove + self.pml.right[i] * smove;
                }
                wishvel[2] = 0.0;

                if (*pm).cmd.upmove <= 0 {
                    let wv = wishvel;
                    _VectorScale(wv, 0.8, &mut wishvel);
                } else {
                    //if we are jetting then we have more control than usual
                    let wv = wishvel;
                    _VectorScale(wv, 2.0, &mut wishvel);
                }
            } else {
                for i in 0..2 {
                    wishvel[i] = self.pml.forward[i] * fmove + self.pml.right[i] * smove;
                }
                wishvel[2] = 0.0;
            }

            _VectorCopy(wishvel, &mut wishdir);
            wishspeed = VectorNormalize(&mut wishdir);
            wishspeed *= scale;

            let mut accelerate = pm_airaccelerate;
            if !pVeh.is_null()
                && (*(*pVeh).m_pVehicleInfo).r#type as c_int == vehicleType_t::VH_SPEEDER as c_int
            {
                //speeders have more control in air
                accelerate = (*(*pVeh).m_pVehicleInfo).traction;
                if self.pml.groundPlane != qfalse {
                    //on a slope of some kind, shouldn't have much control and should slide a lot
                    accelerate *= 0.5;
                }
            }
            // not on ground, so little effect on velocity
            self.PM_Accelerate(wishdir, wishspeed, accelerate);

            // we may have a ground plane that is very steep, even though we don't have a groundentity
            if self.pml.groundPlane != qfalse {
                if (*ps).pm_flags & PMF_STUCK_TO_WALL == 0 {
                    //don't slide when stuck to a wall
                    if self.PM_GroundSlideOkay(self.pml.groundTrace.plane.normal[2]) != qfalse {
                        let inp = (*ps).velocity;
                        let norm = self.pml.groundTrace.plane.normal;
                        self.PM_ClipVelocity(inp, norm, &mut (*ps).velocity, OVERCLIP);
                    }
                }
            }

            if (*ps).pm_flags & PMF_STUCK_TO_WALL != 0 {
                //no grav when stuck to wall
                self.PM_StepSlideMove(qfalse);
            } else {
                self.PM_StepSlideMove(qtrue);
            }
        }
    }

    /// Raven `PM_WalkMove`.
    /// Source: `oracle/codemp/game/bg_pmove.c:3305-3484`
    pub fn PM_WalkMove(&mut self) {
        use crate::bg_slidemove::OVERCLIP;
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let mut wishvel: vec3_t = [0.0; 3];
            let mut wishdir: vec3_t = [0.0; 3];
            let mut wishspeed = 0.0f32;
            let mut npcMovement = qfalse;

            let dpf = self.pml.forward[0] * self.pml.groundTrace.plane.normal[0]
                + self.pml.forward[1] * self.pml.groundTrace.plane.normal[1]
                + self.pml.forward[2] * self.pml.groundTrace.plane.normal[2];
            if (*pm).waterlevel > 2 && dpf > 0.0 {
                // begin swimming
                self.PM_WaterMove();
                return;
            }

            if (*ps).pm_type != PM_SPECTATOR as c_int {
                if self.PM_CheckJump() != qfalse {
                    // jumped away
                    if (*pm).waterlevel > 1 {
                        self.PM_WaterMove();
                    } else {
                        self.PM_AirMove();
                    }
                    return;
                }
            }

            self.PM_Friction();

            let fmove = (*pm).cmd.forwardmove as f32;
            let smove = (*pm).cmd.rightmove as f32;

            let scale = self.PM_CmdScale(core::ptr::addr_of_mut!((*pm).cmd));

            // set the movementDir so clients can rotate the legs for strafing
            self.PM_SetMovementDir();

            // project moves down to flat plane
            self.pml.forward[2] = 0.0;
            self.pml.right[2] = 0.0;

            // project the forward and right directions onto the ground plane
            let inp = self.pml.forward;
            let norm = self.pml.groundTrace.plane.normal;
            let mut out = self.pml.forward;
            self.PM_ClipVelocity(inp, norm, &mut out, OVERCLIP);
            self.pml.forward = out;
            let inp = self.pml.right;
            let mut out = self.pml.right;
            self.PM_ClipVelocity(inp, norm, &mut out, OVERCLIP);
            self.pml.right = out;
            //
            VectorNormalize(&mut self.pml.forward);
            VectorNormalize(&mut self.pml.right);

            // Get The WishVel And WishSpeed
            if (*ps).clientNum >= MAX_CLIENTS as c_int && !VectorCompare((*ps).moveDir, vec3_origin)
            {
                //NPC
                let pEnt = self.pm_entSelf;

                if !pEnt.is_null() && (*pEnt).s.NPC_class == CLASS_VEHICLE as c_int {
                    if (fmove != 0.0 || smove != 0.0) && VectorCompare((*ps).moveDir, vec3_origin) {
                        for i in 0..3 {
                            wishvel[i] = self.pml.forward[i] * fmove + self.pml.right[i] * smove;
                        }

                        _VectorCopy(wishvel, &mut wishdir);
                        wishspeed = VectorNormalize(&mut wishdir);
                        wishspeed *= scale;
                    } else {
                        _VectorScale((*ps).moveDir, (*ps).speed, &mut wishvel);
                        _VectorCopy(wishvel, &mut wishdir);
                        wishspeed = VectorNormalize(&mut wishdir);
                    }

                    npcMovement = qtrue;
                }
            }

            if npcMovement == qfalse {
                for i in 0..3 {
                    wishvel[i] = self.pml.forward[i] * fmove + self.pml.right[i] * smove;
                }
                _VectorCopy(wishvel, &mut wishdir);
                wishspeed = VectorNormalize(&mut wishdir);
                wishspeed *= scale;
            }

            // clamp the speed lower if ducking
            if (*ps).pm_flags & PMF_DUCKED != 0 {
                if wishspeed > (*ps).speed * pm_duckScale {
                    wishspeed = (*ps).speed * pm_duckScale;
                }
            } else if (*ps).pm_flags & PMF_ROLLING != 0
                && BG_InRoll(ps, (*ps).legsAnim) == qfalse
                && PM_InRollComplete(ps, (*ps).legsAnim) == qfalse
            {
                if wishspeed > (*ps).speed * pm_duckScale {
                    wishspeed = (*ps).speed * pm_duckScale;
                }
            }

            // clamp the speed lower if wading or walking on the bottom
            if (*pm).waterlevel != 0 {
                let mut waterScale = ((*pm).waterlevel as f64 / 3.0) as f32;
                waterScale = (1.0 - (1.0 - pm_swimScale as f64) * waterScale as f64) as f32;
                if wishspeed > (*ps).speed * waterScale {
                    wishspeed = (*ps).speed * waterScale;
                }
            }

            // when a player gets hit, they temporarily lose full control
            let accelerate;
            if self.pm_flying == FLY_HOVER {
                accelerate = pm_vehicleaccelerate;
            } else if self.pml.groundTrace.surfaceFlags & SURF_SLICK != 0
                || (*ps).pm_flags & PMF_TIME_KNOCKBACK != 0
            {
                accelerate = pm_airaccelerate;
            } else {
                accelerate = pm_accelerate;
            }

            self.PM_Accelerate(wishdir, wishspeed, accelerate);

            if self.pml.groundTrace.surfaceFlags & SURF_SLICK != 0
                || (*ps).pm_flags & PMF_TIME_KNOCKBACK != 0
            {
                (*ps).velocity[2] -= (*ps).gravity as f32 * self.pml.frametime;
            }

            let vel = VectorLength((*ps).velocity);

            // slide along the ground plane
            let inp = (*ps).velocity;
            let norm = self.pml.groundTrace.plane.normal;
            self.PM_ClipVelocity(inp, norm, &mut (*ps).velocity, OVERCLIP);

            // don't decrease velocity when going up or down a slope
            VectorNormalize(&mut (*ps).velocity);
            let v = (*ps).velocity;
            _VectorScale(v, vel, &mut (*ps).velocity);

            // don't do anything if standing still
            if (*ps).velocity[0] == 0.0 && (*ps).velocity[1] == 0.0 {
                return;
            }

            self.PM_StepSlideMove(qfalse);
        }
    }

    /// Raven `PM_DeadMove`.
    /// Source: `oracle/codemp/game/bg_pmove.c:3492-3509`
    pub fn PM_DeadMove(&mut self) {
        unsafe {
            let ps = (*self.pm).ps;
            if self.pml.walking == qfalse {
                return;
            }

            // extra friction
            let mut forward = VectorLength((*ps).velocity);
            forward -= 20.0;
            if forward <= 0.0 {
                VectorClear(&mut (*ps).velocity);
            } else {
                VectorNormalize(&mut (*ps).velocity);
                let v = (*ps).velocity;
                _VectorScale(v, forward, &mut (*ps).velocity);
            }
        }
    }
}

impl PmoveContext<'_> {
    /// Raven `PM_NoclipMove`.
    /// Source: `oracle/codemp/game/bg_pmove.c:3517-3576`
    pub fn PM_NoclipMove(&mut self) {
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let mut wishvel: vec3_t = [0.0; 3];
            let mut wishdir: vec3_t = [0.0; 3];

            (*ps).viewheight = DEFAULT_VIEWHEIGHT;

            // friction
            let speed = VectorLength((*ps).velocity);
            if speed < 1.0 {
                _VectorCopy(vec3_origin, &mut (*ps).velocity);
            } else {
                let mut drop = 0.0f32;

                let friction = (pm_friction as f64 * 1.5) as f32; // extra friction
                let control = if speed < pm_stopspeed {
                    pm_stopspeed
                } else {
                    speed
                };
                drop += control * friction * self.pml.frametime;

                // scale the velocity
                let mut newspeed = speed - drop;
                if newspeed < 0.0 {
                    newspeed = 0.0;
                }
                newspeed /= speed;

                let v = (*ps).velocity;
                _VectorScale(v, newspeed, &mut (*ps).velocity);
            }

            // accelerate
            let mut scale = self.PM_CmdScale(core::ptr::addr_of_mut!((*pm).cmd));
            if (*pm).cmd.buttons & BUTTON_ATTACK != 0 {
                //turbo boost
                scale *= 10.0;
            }
            if (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                //turbo boost
                scale *= 10.0;
            }

            let fmove = (*pm).cmd.forwardmove as f32;
            let smove = (*pm).cmd.rightmove as f32;

            for i in 0..3 {
                wishvel[i] = self.pml.forward[i] * fmove + self.pml.right[i] * smove;
            }
            wishvel[2] += (*pm).cmd.upmove as f32;

            _VectorCopy(wishvel, &mut wishdir);
            let mut wishspeed = VectorNormalize(&mut wishdir);
            wishspeed *= scale;

            self.PM_Accelerate(wishdir, wishspeed, pm_accelerate);

            // move
            let o = (*ps).origin;
            _VectorMA(o, self.pml.frametime, (*ps).velocity, &mut (*ps).origin);
        }
    }

    /// Raven `PM_FootstepForSurface`.
    /// Source: `oracle/codemp/game/bg_pmove.c:3587-3594`
    pub fn PM_FootstepForSurface(&mut self) -> c_int {
        if self.pml.groundTrace.surfaceFlags & SURF_NOSTEPS != 0 {
            return 0;
        }
        self.pml.groundTrace.surfaceFlags & MATERIAL_MASK
    }

    /// Raven `PM_TryRoll`.
    /// Source: `oracle/codemp/game/bg_pmove.c:3597-3681`
    pub fn PM_TryRoll(&mut self) -> c_int {
        use crate::bg_slidemove::STEPSIZE;
        use animNumber_t::*;
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let mut anim = -1;
            let mut fwd: vec3_t = [0.0; 3];
            let mut right: vec3_t = [0.0; 3];
            let mut traceto: vec3_t = [0.0; 3];
            let mut mins: vec3_t = [0.0; 3];
            let mut maxs: vec3_t = [0.0; 3];
            let mut fwdAngles: vec3_t = [0.0; 3];

            if BG_SaberInAttack((*ps).saberMove) != qfalse
                || BG_SaberInSpecialAttack((*ps).torsoAnim) != qfalse
                || BG_SpinningSaberAnim((*ps).legsAnim) != qfalse
                || PM_SaberInStart((*ps).saberMove) != qfalse
            {
                //attacking or spinning (or, if player, starting an attack)
                if PM_CanRollFromSoulCal(ps) != qfalse {
                    //hehe
                } else {
                    return 0;
                }
            }

            if ((*ps).weapon != WP_SABER as c_int && (*ps).weapon != WP_MELEE as c_int)
                || PM_IsRocketTrooper() != qfalse
                || BG_HasYsalamiri((*pm).gametype, ps) != qfalse
                || BG_CanUseFPNow((*pm).gametype, ps, (*pm).cmd.serverTime, FP_LEVITATION) == qfalse
            {
                //Not using saber, or can't use jump
                return 0;
            }

            if (*ps).weapon == WP_SABER as c_int {
                let mut saber = self.callbacks.my_saber((*ps).clientNum, 0);
                if !saber.is_null() && (*saber).saberFlags & SFL_NO_ROLLS != 0 {
                    return 0;
                }
                saber = self.callbacks.my_saber((*ps).clientNum, 1);
                if !saber.is_null() && (*saber).saberFlags & SFL_NO_ROLLS != 0 {
                    return 0;
                }
            }

            VectorSet(
                &mut mins,
                (*pm).mins[0],
                (*pm).mins[1],
                (*pm).mins[2] + STEPSIZE,
            );
            VectorSet(
                &mut maxs,
                (*pm).maxs[0],
                (*pm).maxs[1],
                (*ps).crouchheight as f32,
            );

            VectorSet(&mut fwdAngles, 0.0, (*ps).viewangles[YAW], 0.0);

            AngleVectors(fwdAngles, Some(&mut fwd), Some(&mut right), None);

            if (*pm).cmd.forwardmove != 0 {
                //check forward/backward rolls
                if (*ps).pm_flags & PMF_BACKWARDS_RUN != 0 {
                    anim = BOTH_ROLL_B as c_int;
                    _VectorMA((*ps).origin, -64.0, fwd, &mut traceto);
                } else {
                    anim = BOTH_ROLL_F as c_int;
                    _VectorMA((*ps).origin, 64.0, fwd, &mut traceto);
                }
            } else if (*pm).cmd.rightmove > 0 {
                //right
                anim = BOTH_ROLL_R as c_int;
                _VectorMA((*ps).origin, 64.0, right, &mut traceto);
            } else if (*pm).cmd.rightmove < 0 {
                //left
                anim = BOTH_ROLL_L as c_int;
                _VectorMA((*ps).origin, -64.0, right, &mut traceto);
            }

            if anim != -1 {
                //We want to roll. Perform a trace to see if we can, and if so, send us into one.
                let mut trace: trace_t = core::mem::zeroed();
                self.traps.trace(
                    &mut trace,
                    core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                    core::ptr::addr_of!(mins) as *const vec3_t,
                    core::ptr::addr_of!(maxs) as *const vec3_t,
                    core::ptr::addr_of!(traceto) as *const vec3_t,
                    (*ps).clientNum,
                    CONTENTS_SOLID,
                );
                if trace.fraction >= 1.0 {
                    (*ps).saberMove = LS_NONE as c_int;
                    return anim;
                }
            }
            0
        }
    }
}

impl PmoveContext<'_> {
    /// Raven `PM_CrashLandEffect`.
    /// Source: `oracle/codemp/game/bg_pmove.c:3684-3722`
    pub fn PM_CrashLandEffect(&mut self) {
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            if (*pm).waterlevel != 0 {
                return;
            }
            let delta = ((self.pml.previous_velocity[2] as f64).abs() / 10.0) as f32;
            if delta >= 30.0 {
                let mut bottom: vec3_t = [0.0; 3];
                let mut effectID = -1;
                let material = self.pml.groundTrace.surfaceFlags & MATERIAL_MASK;
                VectorSet(
                    &mut bottom,
                    (*ps).origin[0],
                    (*ps).origin[1],
                    (*ps).origin[2] + (*pm).mins[2] + 1.0,
                );
                if material == MATERIAL_MUD {
                    effectID = EFFECT_LANDING_MUD as c_int;
                } else if material == MATERIAL_SAND {
                    effectID = EFFECT_LANDING_SAND as c_int;
                } else if material == MATERIAL_DIRT {
                    effectID = EFFECT_LANDING_DIRT as c_int;
                } else if material == MATERIAL_SNOW {
                    effectID = EFFECT_LANDING_SNOW as c_int;
                } else if material == MATERIAL_GRAVEL {
                    effectID = EFFECT_LANDING_GRAVEL as c_int;
                }

                if effectID != -1 {
                    let normal = self.pml.groundTrace.plane.normal;
                    self.callbacks.play_effect(
                        effectID,
                        core::ptr::addr_of!(bottom),
                        core::ptr::addr_of!(normal),
                    );
                }
            }
        }
    }

    /// Raven `PM_CrashLand`. `QAGAME` is defined, so the `PM_CrashLandEffect` call is compiled.
    /// Source: `oracle/codemp/game/bg_pmove.c:3731-4002`
    pub fn PM_CrashLand(&mut self) {
        use animNumber_t::*;
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let mut didRoll = qfalse;

            // calculate the exact velocity on landing
            let dist = (*ps).origin[2] - self.pml.previous_origin[2];
            let vel = self.pml.previous_velocity[2];
            let acc = -(*ps).gravity as f32;

            let a = acc / 2.0;
            let b = vel;
            let c = -dist;

            let den = b * b - 4.0 * a * c;
            if den < 0.0 {
                (*ps).inAirAnim = qfalse;
                return;
            }
            let t = ((-b as f64 - (den as f64).sqrt()) / (2.0 * a as f64)) as f32;

            let mut delta = vel + t * acc;
            delta = ((delta * delta) as f64 * 0.0001) as f32;

            // QAGAME
            self.PM_CrashLandEffect();

            // ducking while falling doubles damage
            if (*ps).pm_flags & PMF_DUCKED != 0 {
                delta *= 2.0;
            }

            let la = (*ps).legsAnim;
            if la == BOTH_A7_KICK_F_AIR as c_int
                || la == BOTH_A7_KICK_B_AIR as c_int
                || la == BOTH_A7_KICK_R_AIR as c_int
                || la == BOTH_A7_KICK_L_AIR as c_int
            {
                let mut landAnim = -1;
                if la == BOTH_A7_KICK_F_AIR as c_int {
                    landAnim = BOTH_FORCELAND1 as c_int;
                } else if la == BOTH_A7_KICK_B_AIR as c_int {
                    landAnim = BOTH_FORCELANDBACK1 as c_int;
                } else if la == BOTH_A7_KICK_R_AIR as c_int {
                    landAnim = BOTH_FORCELANDRIGHT1 as c_int;
                } else if la == BOTH_A7_KICK_L_AIR as c_int {
                    landAnim = BOTH_FORCELANDLEFT1 as c_int;
                }
                if landAnim != -1 {
                    if (*ps).torsoAnim == (*ps).legsAnim {
                        self.PM_SetAnim(
                            SETANIM_BOTH,
                            landAnim,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            0,
                        );
                    } else {
                        self.PM_SetAnim(
                            SETANIM_LEGS,
                            landAnim,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            0,
                        );
                    }
                }
            } else if la == BOTH_FORCEJUMPLEFT1 as c_int
                || la == BOTH_FORCEJUMPRIGHT1 as c_int
                || la == BOTH_FORCEJUMPBACK1 as c_int
                || la == BOTH_FORCEJUMP1 as c_int
            {
                let fjAnim;
                if la == BOTH_FORCEJUMPLEFT1 as c_int {
                    fjAnim = BOTH_LANDLEFT1 as c_int;
                } else if la == BOTH_FORCEJUMPRIGHT1 as c_int {
                    fjAnim = BOTH_LANDRIGHT1 as c_int;
                } else if la == BOTH_FORCEJUMPBACK1 as c_int {
                    fjAnim = BOTH_LANDBACK1 as c_int;
                } else {
                    fjAnim = BOTH_LAND1 as c_int;
                }
                self.PM_SetAnim(
                    SETANIM_BOTH,
                    fjAnim,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    0,
                );
            }
            // decide which landing animation to use
            else if BG_InRoll(ps, (*ps).legsAnim) == qfalse
                && (*ps).inAirAnim != 0
                && (*ps).m_iVehicleNum == 0
            {
                //only play a land animation if we transitioned into an in-air animation while off the ground
                if BG_SaberInSpecial((*ps).saberMove) == qfalse {
                    if (*ps).pm_flags & PMF_BACKWARDS_JUMP != 0 {
                        self.PM_ForceLegsAnim(BOTH_LANDBACK1 as c_int);
                    } else {
                        self.PM_ForceLegsAnim(BOTH_LAND1 as c_int);
                    }
                }
            }

            if (*ps).weapon != WP_SABER as c_int
                && (*ps).weapon != WP_MELEE as c_int
                && PM_IsRocketTrooper() == qfalse
            {
                //saber handles its own anims
                //This will push us back into our weaponready stance from the land anim.
                if (*ps).weapon == WP_DISRUPTOR as c_int && (*ps).zoomMode == 1 {
                    self.PM_StartTorsoAnim(TORSO_WEAPONREADY4 as c_int);
                } else {
                    if (*ps).weapon == WP_EMPLACED_GUN as c_int {
                        self.PM_StartTorsoAnim(BOTH_GUNSIT1 as c_int);
                    } else {
                        self.PM_StartTorsoAnim(WeaponReadyAnim[(*ps).weapon as usize]);
                    }
                }
            }

            if BG_InSpecialJump((*ps).legsAnim) == qfalse
                || (*ps).legsTimer < 1
                || (*ps).legsAnim == BOTH_WALL_RUN_LEFT as c_int
                || (*ps).legsAnim == BOTH_WALL_RUN_RIGHT as c_int
            {
                //Only set the timer if we're in an anim that can be interrupted (this would not be, say, a flip)
                if BG_InRoll(ps, (*ps).legsAnim) == qfalse && (*ps).inAirAnim != 0 {
                    if BG_SaberInSpecial((*ps).saberMove) == qfalse
                        || (*ps).weapon != WP_SABER as c_int
                    {
                        if (*ps).legsAnim != BOTH_FORCELAND1 as c_int
                            && (*ps).legsAnim != BOTH_FORCELANDBACK1 as c_int
                            && (*ps).legsAnim != BOTH_FORCELANDRIGHT1 as c_int
                            && (*ps).legsAnim != BOTH_FORCELANDLEFT1 as c_int
                        {
                            //don't override if we have started a force land
                            (*ps).legsTimer = TIMER_LAND;
                        }
                    }
                }
            }

            (*ps).inAirAnim = qfalse;

            if (*ps).m_iVehicleNum != 0 {
                //don't do fall stuff while on a vehicle
                return;
            }

            // never take falling damage if completely underwater
            if (*pm).waterlevel == 3 {
                return;
            }

            // reduce falling damage if there is standing water
            if (*pm).waterlevel == 2 {
                delta *= 0.25;
            }
            if (*pm).waterlevel == 1 {
                delta *= 0.5;
            }

            if delta < 1.0 {
                return;
            }

            if (*ps).pm_flags & PMF_DUCKED != 0 {
                if delta >= 2.0
                    && PM_InOnGroundAnim((*ps).legsAnim) == qfalse
                    && PM_InKnockDown(ps) == qfalse
                    && BG_InRoll(ps, (*ps).legsAnim) == qfalse
                    && (*ps).forceHandExtend == HANDEXTEND_NONE as c_int
                {
                    //roll!
                    let mut anim = self.PM_TryRoll();

                    if PM_InRollComplete(ps, (*ps).legsAnim) != qfalse {
                        anim = 0;
                        (*ps).legsTimer = 0;
                        (*ps).legsAnim = 0;
                        self.PM_SetAnim(
                            SETANIM_BOTH,
                            BOTH_LAND1 as c_int,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            150,
                        );
                        (*ps).legsTimer = TIMER_LAND;
                    }

                    if anim != 0 {
                        //absorb some impact
                        (*ps).legsTimer = 0;
                        delta /= 3.0;
                        (*ps).legsAnim = 0;
                        if (*ps).torsoAnim == BOTH_A7_SOULCAL as c_int {
                            //get out of it on torso
                            (*ps).torsoTimer = 0;
                        }
                        self.PM_SetAnim(
                            SETANIM_BOTH,
                            anim,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            150,
                        );
                        didRoll = qtrue;
                    }
                }
            }

            // SURF_NODAMAGE is used for bounce pads
            if self.pml.groundTrace.surfaceFlags & SURF_NODAMAGE == 0 {
                if delta > 7.0 {
                    let mut delta_send = delta as c_int;

                    if delta_send > 600 {
                        delta_send = 600;
                    }

                    if (*ps).fd.forceJumpZStart != 0.0 {
                        if ((*ps).origin[2] as c_int) >= ((*ps).fd.forceJumpZStart as c_int) {
                            //was force jumping, landed on higher or same level as when force jump was started
                            if delta_send > 8 {
                                delta_send = 8;
                            }
                        } else {
                            if delta_send > 8 {
                                let dif = ((*ps).fd.forceJumpZStart as c_int)
                                    - ((*ps).origin[2] as c_int);
                                let lvl = (*ps).fd.forcePowerLevel[FP_LEVITATION as usize] as usize;
                                let mut dmgLess = (forceJumpHeight[lvl] - dif as f32) as c_int;

                                if dmgLess < 0 {
                                    dmgLess = 0;
                                }

                                delta_send = (delta_send as f64 - dmgLess as f64 * 0.3) as c_int;

                                if delta_send < 8 {
                                    delta_send = 8;
                                }
                            }
                        }
                    }

                    if didRoll != qfalse {
                        //Add the appropriate event..
                        self.PM_AddEventWithParm(EV_ROLL as c_int, delta_send);
                    } else {
                        self.PM_AddEventWithParm(EV_FALL as c_int, delta_send);
                    }
                } else {
                    if didRoll != qfalse {
                        self.PM_AddEventWithParm(EV_ROLL as c_int, 0);
                    } else {
                        let fs = self.PM_FootstepForSurface();
                        self.PM_AddEventWithParm(EV_FOOTSTEP as c_int, fs);
                    }
                }
            }

            // make sure velocity resets so we don't bounce back up again
            (*ps).velocity[2] = 0.0;

            // start footstep cycle over
            (*ps).bobCycle = 0;
        }
    }
}

impl PmoveContext<'_> {
    /// Raven `PM_CorrectAllSolid`.
    /// Source: `oracle/codemp/game/bg_pmove.c:4009-4044`
    pub fn PM_CorrectAllSolid(&mut self, trace: *mut trace_t) -> c_int {
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let mut point: vec3_t = [0.0; 3];

            if (*pm).debugLevel != 0 {
                self.traps
                    .com_printf(&format!("{}:allsolid\n", self.bg.c_pmove));
            }

            // jitter around
            for i in -1..=1 {
                for j in -1..=1 {
                    for k in -1..=1 {
                        _VectorCopy((*ps).origin, &mut point);
                        point[0] += i as f32;
                        point[1] += j as f32;
                        point[2] += k as f32;
                        self.traps.trace(
                            trace,
                            core::ptr::addr_of!(point) as *const vec3_t,
                            core::ptr::addr_of!((*pm).mins) as *const vec3_t,
                            core::ptr::addr_of!((*pm).maxs) as *const vec3_t,
                            core::ptr::addr_of!(point) as *const vec3_t,
                            (*ps).clientNum,
                            (*pm).tracemask,
                        );
                        if (*trace).allsolid == 0 {
                            point[0] = (*ps).origin[0];
                            point[1] = (*ps).origin[1];
                            point[2] = (*ps).origin[2] - 0.25;

                            self.traps.trace(
                                trace,
                                core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                                core::ptr::addr_of!((*pm).mins) as *const vec3_t,
                                core::ptr::addr_of!((*pm).maxs) as *const vec3_t,
                                core::ptr::addr_of!(point) as *const vec3_t,
                                (*ps).clientNum,
                                (*pm).tracemask,
                            );
                            self.pml.groundTrace = *trace;
                            return qtrue;
                        }
                    }
                }
            }

            (*ps).groundEntityNum = ENTITYNUM_NONE;
            self.pml.groundPlane = qfalse;
            self.pml.walking = qfalse;

            qfalse
        }
    }

    /// Raven `PM_GroundTraceMissed`.
    /// Source: `oracle/codemp/game/bg_pmove.c:4053-4133`
    pub fn PM_GroundTraceMissed(&mut self) {
        use animNumber_t::*;
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let mut trace: trace_t = core::mem::zeroed();
            let mut point: vec3_t = [0.0; 3];

            if (*ps).pm_type == PM_FLOAT as c_int {
                //we're assuming this is because you're being choked
                self.PM_SetAnim(
                    SETANIM_LEGS,
                    BOTH_CHOKE3 as c_int,
                    SETANIM_FLAG_OVERRIDE,
                    100,
                );
            } else if (*ps).pm_type == PM_JETPACK as c_int {
                //jetpacking (nothing)
            }
            //If the anim is choke3, act like we just went into the air because we aren't in a float
            else if (*ps).groundEntityNum != ENTITYNUM_NONE
                || (*ps).legsAnim == BOTH_CHOKE3 as c_int
            {
                // we just transitioned into freefall
                if (*pm).debugLevel != 0 {
                    self.traps
                        .com_printf(&format!("{}:lift\n", self.bg.c_pmove));
                }

                _VectorCopy((*ps).origin, &mut point);
                point[2] -= 64.0;

                self.traps.trace(
                    &mut trace,
                    core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                    core::ptr::addr_of!((*pm).mins) as *const vec3_t,
                    core::ptr::addr_of!((*pm).maxs) as *const vec3_t,
                    core::ptr::addr_of!(point) as *const vec3_t,
                    (*ps).clientNum,
                    (*pm).tracemask,
                );
                if trace.fraction == 1.0 || (*ps).pm_type == PM_FLOAT as c_int {
                    if (*ps).velocity[2] <= 0.0 && (*ps).pm_flags & PMF_JUMP_HELD == 0 {
                        self.PM_SetAnim(SETANIM_LEGS, BOTH_INAIR1 as c_int, 0, 100);
                        (*ps).pm_flags &= !PMF_BACKWARDS_JUMP;
                    } else if (*pm).cmd.forwardmove >= 0 {
                        self.PM_SetAnim(
                            SETANIM_LEGS,
                            BOTH_JUMP1 as c_int,
                            SETANIM_FLAG_OVERRIDE,
                            100,
                        );
                        (*ps).pm_flags &= !PMF_BACKWARDS_JUMP;
                    } else {
                        self.PM_SetAnim(
                            SETANIM_LEGS,
                            BOTH_JUMPBACK1 as c_int,
                            SETANIM_FLAG_OVERRIDE,
                            100,
                        );
                        (*ps).pm_flags |= PMF_BACKWARDS_JUMP;
                    }

                    (*ps).inAirAnim = qtrue;
                }
            } else if (*ps).inAirAnim == 0 {
                _VectorCopy((*ps).origin, &mut point);
                point[2] -= 64.0;

                self.traps.trace(
                    &mut trace,
                    core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                    core::ptr::addr_of!((*pm).mins) as *const vec3_t,
                    core::ptr::addr_of!((*pm).maxs) as *const vec3_t,
                    core::ptr::addr_of!(point) as *const vec3_t,
                    (*ps).clientNum,
                    (*pm).tracemask,
                );
                if trace.fraction == 1.0 || (*ps).pm_type == PM_FLOAT as c_int {
                    (*ps).inAirAnim = qtrue;
                }
            }

            if PM_InRollComplete(ps, (*ps).legsAnim) != qfalse {
                self.PM_SetAnim(
                    SETANIM_BOTH,
                    BOTH_INAIR1 as c_int,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    150,
                );
                (*ps).inAirAnim = qtrue;
            }

            (*ps).groundEntityNum = ENTITYNUM_NONE;
            self.pml.groundPlane = qfalse;
            self.pml.walking = qfalse;
        }
    }
}

impl PmoveContext<'_> {
    /// Raven `PM_GroundTrace`. `QAGAME` is defined, so the vehicle-board block is compiled.
    /// Source: `oracle/codemp/game/bg_pmove.c:4141-4277`
    pub fn PM_GroundTrace(&mut self) {
        use animNumber_t::*;
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let mut point: vec3_t = [0.0; 3];
            let mut trace: trace_t = core::mem::zeroed();
            let mut minNormal = MIN_WALK_NORMAL;

            if (*ps).clientNum >= MAX_CLIENTS as c_int {
                let pEnt = self.pm_entSelf;
                if !pEnt.is_null() && (*pEnt).s.NPC_class == CLASS_VEHICLE as c_int {
                    let pv = (*pEnt).m_pVehicle as *mut Vehicle_t;
                    minNormal = (*(*pv).m_pVehicleInfo).maxSlope;
                }
            }

            point[0] = (*ps).origin[0];
            point[1] = (*ps).origin[1];
            point[2] = (*ps).origin[2] - 0.25;

            self.traps.trace(
                &mut trace,
                core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                core::ptr::addr_of!((*pm).mins) as *const vec3_t,
                core::ptr::addr_of!((*pm).maxs) as *const vec3_t,
                core::ptr::addr_of!(point) as *const vec3_t,
                (*ps).clientNum,
                (*pm).tracemask,
            );
            self.pml.groundTrace = trace;

            // do something corrective if the trace starts in a solid...
            if trace.allsolid != 0 {
                if self.PM_CorrectAllSolid(&mut trace) == qfalse {
                    return;
                }
            }

            if (*ps).pm_type == PM_FLOAT as c_int || (*ps).pm_type == PM_JETPACK as c_int {
                self.PM_GroundTraceMissed();
                self.pml.groundPlane = qfalse;
                self.pml.walking = qfalse;
                return;
            }

            // if the trace didn't hit anything, we are in free fall
            if trace.fraction == 1.0 {
                self.PM_GroundTraceMissed();
                self.pml.groundPlane = qfalse;
                self.pml.walking = qfalse;
                return;
            }

            // check if getting thrown off the ground
            let dp = (*ps).velocity[0] * trace.plane.normal[0]
                + (*ps).velocity[1] * trace.plane.normal[1]
                + (*ps).velocity[2] * trace.plane.normal[2];
            if (*ps).velocity[2] > 0.0 && dp > 10.0 {
                if (*pm).debugLevel != 0 {
                    self.traps
                        .com_printf(&format!("{}:kickoff\n", self.bg.c_pmove));
                }
                // go into jump animation
                if (*pm).cmd.forwardmove >= 0 {
                    self.PM_ForceLegsAnim(BOTH_JUMP1 as c_int);
                    (*ps).pm_flags &= !PMF_BACKWARDS_JUMP;
                } else {
                    self.PM_ForceLegsAnim(BOTH_JUMPBACK1 as c_int);
                    (*ps).pm_flags |= PMF_BACKWARDS_JUMP;
                }

                (*ps).groundEntityNum = ENTITYNUM_NONE;
                self.pml.groundPlane = qfalse;
                self.pml.walking = qfalse;
                return;
            }

            // slopes that are too steep will not be considered onground
            if trace.plane.normal[2] < minNormal {
                if (*pm).debugLevel != 0 {
                    self.traps
                        .com_printf(&format!("{}:steep\n", self.bg.c_pmove));
                }
                (*ps).groundEntityNum = ENTITYNUM_NONE;
                self.pml.groundPlane = qtrue;
                self.pml.walking = qfalse;
                return;
            }

            self.pml.groundPlane = qtrue;
            self.pml.walking = qtrue;

            // hitting solid ground will end a waterjump
            if (*ps).pm_flags & PMF_TIME_WATERJUMP != 0 {
                (*ps).pm_flags &= !(PMF_TIME_WATERJUMP | PMF_TIME_LAND);
                (*ps).pm_time = 0;
            }

            if (*ps).groundEntityNum == ENTITYNUM_NONE {
                // just hit the ground
                if (*pm).debugLevel != 0 {
                    self.traps
                        .com_printf(&format!("{}:Land\n", self.bg.c_pmove));
                }

                self.PM_CrashLand();

                // QAGAME: check if we landed on a vehicle
                if (*ps).clientNum < MAX_CLIENTS as c_int
                    && (*ps).m_iVehicleNum == 0
                    && (trace.entityNum as c_int) < ENTITYNUM_WORLD
                    && (trace.entityNum as c_int) >= MAX_CLIENTS as c_int
                    && (*ps).zoomMode == 0
                    && !self.pm_entSelf.is_null()
                {
                    //it's a vehicle alright, let's board it.. if it's not an atst or ship
                    // S5-2: the boardable gate (inuse/client/eType/NPC_class/
                    // m_iVehicleNum/m_pVehicle type) and the team gate (alliedTeam/
                    // sess.sessionTeam) are game-side reads folded into one upcall.
                    // gametype stays a bg-side read (`pm->gametype`, same value as
                    // `g_gametype.integer`) passed in. The bg-side gate
                    // (BG_SaberInSpecial/forceHandExtend/weaponTime) stays here; its
                    // pure reads now follow the game gate with no observable change.
                    let trEntNum = trace.entityNum as c_int;
                    if self.callbacks.landed_vehicle_boardable(
                        trEntNum,
                        (*self.pm_entSelf).s.number,
                        (*pm).gametype,
                    ) != 0
                        && BG_SaberInSpecial((*ps).saberMove) == qfalse
                        && (*ps).forceHandExtend == HANDEXTEND_NONE as c_int
                        && (*ps).weaponTime <= 0
                    {
                        //not belonging to a team, or client is on same team
                        // The vehicle `Board` body is game-tier; bg reaches it via
                        // the GameCallbacks upcall (by entity number), which
                        // dispatches through `crate::veh_dispatch::board`.
                        self.callbacks
                            .board_vehicle(trEntNum, (*self.pm_entSelf).s.number);
                    }
                }

                // don't do landing time if we were just going down a slope
                if self.pml.previous_velocity[2] < -200.0 {
                    // don't allow another jump for a little while
                    (*ps).pm_flags |= PMF_TIME_LAND;
                    (*ps).pm_time = 250;
                }
            }

            (*ps).groundEntityNum = trace.entityNum as c_int;
            (*ps).lastOnGround = (*pm).cmd.serverTime;

            self.PM_AddTouchEnt(trace.entityNum as c_int);
        }
    }
}

// `PM_SetWaterLevel` is a `PmoveContext` method above (it reads the pmove
// working set and drives `BgTraps::pointcontents`).

impl PmoveContext<'_> {
    /// Raven `PM_CheckDualForwardJumpDuck`.
    /// Source: `oracle/codemp/game/bg_pmove.c:4322-4339`
    pub fn PM_CheckDualForwardJumpDuck(&mut self) -> qboolean {
        use animNumber_t::*;
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let mut resized = qfalse;
            if (*ps).legsAnim == BOTH_JUMPATTACK6 as c_int {
                //dynamically reduce bounding box to let character sail over heads of enemies
                let animLen = self.PM_AnimLength(0, BOTH_JUMPATTACK6 as c_int);
                if ((*ps).legsTimer >= 1450 && animLen - (*ps).legsTimer >= 400)
                    || ((*ps).legsTimer >= 400 && animLen - (*ps).legsTimer >= 1100)
                {
                    //in a part of the anim that we're pretty much sideways in, raise up the mins
                    (*pm).mins[2] = 0.0;
                    (*ps).pm_flags |= PMF_FIX_MINS;
                    resized = qtrue;
                }
            }
            resized
        }
    }

    /// Raven `PM_CheckFixMins`.
    /// Source: `oracle/codemp/game/bg_pmove.c:4341-4401`
    pub fn PM_CheckFixMins(&mut self) {
        use animNumber_t::*;
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            if (*ps).pm_flags & PMF_FIX_MINS != 0 {
                //drop the mins back down
                let mut trace: trace_t = core::mem::zeroed();
                let mut end: vec3_t = [0.0; 3];
                let mut curMins: vec3_t = [0.0; 3];
                let mut curMaxs: vec3_t = [0.0; 3];

                VectorSet(
                    &mut end,
                    (*ps).origin[0],
                    (*ps).origin[1],
                    (*ps).origin[2] + MINS_Z as f32,
                );
                VectorSet(&mut curMins, (*pm).mins[0], (*pm).mins[1], 0.0);
                VectorSet(
                    &mut curMaxs,
                    (*pm).maxs[0],
                    (*pm).maxs[1],
                    (*ps).standheight as f32,
                );

                self.traps.trace(
                    &mut trace,
                    core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                    core::ptr::addr_of!(curMins) as *const vec3_t,
                    core::ptr::addr_of!(curMaxs) as *const vec3_t,
                    core::ptr::addr_of!(end) as *const vec3_t,
                    (*ps).clientNum,
                    (*pm).tracemask,
                );
                if trace.allsolid == 0 && trace.startsolid == 0 {
                    //should never start in solid
                    if trace.fraction >= 1.0 {
                        //all clear: drop the bottom of my bbox back down
                        (*pm).mins[2] = MINS_Z as f32;
                        (*ps).pm_flags &= !PMF_FIX_MINS;
                    } else {
                        //move me up so the bottom of my bbox will be where the trace ended
                        let updist = (1.0 - trace.fraction) * -(MINS_Z as f32);
                        end[2] = (*ps).origin[2] + updist;
                        self.traps.trace(
                            &mut trace,
                            core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                            core::ptr::addr_of!(curMins) as *const vec3_t,
                            core::ptr::addr_of!(curMaxs) as *const vec3_t,
                            core::ptr::addr_of!(end) as *const vec3_t,
                            (*ps).clientNum,
                            (*pm).tracemask,
                        );
                        if trace.allsolid == 0 && trace.startsolid == 0 {
                            if trace.fraction >= 1.0 {
                                //all clear: move me up
                                (*ps).origin[2] += updist;
                                (*pm).mins[2] = MINS_Z as f32;
                                (*ps).pm_flags &= !PMF_FIX_MINS;
                            } else {
                                //no room to expand, so just crouch us
                                if (*ps).legsAnim != BOTH_JUMPATTACK6 as c_int
                                    || (*ps).legsTimer <= 200
                                {
                                    //drop the maxs, put the mins back and move us up
                                    (*pm).maxs[2] += MINS_Z as f32;
                                    (*ps).origin[2] -= MINS_Z as f32;
                                    (*pm).mins[2] = MINS_Z as f32;
                                    //this way we'll be in a crouch when we're done
                                    if (*ps).legsAnim == BOTH_JUMPATTACK6 as c_int {
                                        (*ps).legsTimer = 0;
                                        (*ps).torsoTimer = 0;
                                    }
                                    (*ps).pm_flags |= PMF_DUCKED;
                                    (*ps).pm_flags &= !PMF_FIX_MINS;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Raven `PM_CheckDuck`. `QAGAME` is defined, so the solidHack block is compiled.
    /// Source: `oracle/codemp/game/bg_pmove.c:4410-4542`
    pub fn PM_CheckDuck(&mut self) {
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let mut trace: trace_t = core::mem::zeroed();

            if (*ps).m_iVehicleNum > 0 && (*ps).m_iVehicleNum < ENTITYNUM_NONE {
                //riding a vehicle or are a vehicle: no ducking or rolling when on a vehicle
                (*ps).pm_flags &= !PMF_DUCKED;
                (*ps).pm_flags &= !PMF_ROLLING;

                if (*ps).clientNum >= MAX_CLIENTS as c_int {
                    return;
                }
                if !self.pm_entVeh.is_null() && {
                    let pv = (*self.pm_entVeh).m_pVehicle as *mut Vehicle_t;
                    !pv.is_null()
                        && ((*(*pv).m_pVehicleInfo).r#type as c_int
                            == vehicleType_t::VH_SPEEDER as c_int
                            || (*(*pv).m_pVehicleInfo).r#type as c_int
                                == vehicleType_t::VH_ANIMAL as c_int)
                } {
                    let mut solidTr: trace_t = core::mem::zeroed();

                    (*pm).mins[0] = -16.0;
                    (*pm).mins[1] = -16.0;
                    (*pm).mins[2] = MINS_Z as f32;

                    (*pm).maxs[0] = 16.0;
                    (*pm).maxs[1] = 16.0;
                    (*pm).maxs[2] = (*ps).standheight as f32;
                    (*ps).viewheight = DEFAULT_VIEWHEIGHT;

                    self.traps.trace(
                        &mut solidTr,
                        core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                        core::ptr::addr_of!((*pm).mins) as *const vec3_t,
                        core::ptr::addr_of!((*pm).maxs) as *const vec3_t,
                        core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                        (*ps).m_iVehicleNum,
                        (*pm).tracemask,
                    );
                    if solidTr.startsolid != 0 || solidTr.allsolid != 0 || solidTr.fraction != 1.0 {
                        //whoops, can't fit here. Down to 0!
                        VectorClear(&mut (*pm).mins);
                        VectorClear(&mut (*pm).maxs);
                        // QAGAME solidHack — game-side inuse/client guard + the
                        // `client->solidHack = level.time + 200` write, by number.
                        self.callbacks.set_solid_hack((*ps).clientNum);
                    }
                }
            } else {
                if (*ps).clientNum < MAX_CLIENTS as c_int {
                    (*pm).mins[0] = -15.0;
                    (*pm).mins[1] = -15.0;

                    (*pm).maxs[0] = 15.0;
                    (*pm).maxs[1] = 15.0;
                }

                if self.PM_CheckDualForwardJumpDuck() != qfalse {
                    //special anim resizing us
                } else {
                    self.PM_CheckFixMins();

                    if (*pm).mins[2] == 0.0 {
                        (*pm).mins[2] = MINS_Z as f32;
                    }
                }

                if (*ps).pm_type == PM_DEAD as c_int && (*ps).clientNum < MAX_CLIENTS as c_int {
                    (*pm).maxs[2] = -8.0;
                    (*ps).viewheight = DEAD_VIEWHEIGHT;
                    return;
                }

                if BG_InRoll(ps, (*ps).legsAnim) != qfalse
                    && BG_KickingAnim((*ps).legsAnim) == qfalse
                {
                    (*pm).maxs[2] = (*ps).crouchheight as f32;
                    (*ps).viewheight = DEFAULT_VIEWHEIGHT;
                    (*ps).pm_flags &= !PMF_DUCKED;
                    (*ps).pm_flags |= PMF_ROLLING;
                    return;
                } else if (*ps).pm_flags & PMF_ROLLING != 0 {
                    // try to stand up
                    (*pm).maxs[2] = (*ps).standheight as f32;
                    self.traps.trace(
                        &mut trace,
                        core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                        core::ptr::addr_of!((*pm).mins) as *const vec3_t,
                        core::ptr::addr_of!((*pm).maxs) as *const vec3_t,
                        core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                        (*ps).clientNum,
                        (*pm).tracemask,
                    );
                    if trace.allsolid == 0 {
                        (*ps).pm_flags &= !PMF_ROLLING;
                    }
                } else if (*pm).cmd.upmove < 0
                    || (*ps).forceHandExtend == HANDEXTEND_KNOCKDOWN as c_int
                    || (*ps).forceHandExtend == HANDEXTEND_PRETHROWN as c_int
                    || (*ps).forceHandExtend == HANDEXTEND_POSTTHROWN as c_int
                {
                    // duck
                    (*ps).pm_flags |= PMF_DUCKED;
                } else {
                    // stand up if possible
                    if (*ps).pm_flags & PMF_DUCKED != 0 {
                        // try to stand up
                        (*pm).maxs[2] = (*ps).standheight as f32;
                        self.traps.trace(
                            &mut trace,
                            core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                            core::ptr::addr_of!((*pm).mins) as *const vec3_t,
                            core::ptr::addr_of!((*pm).maxs) as *const vec3_t,
                            core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                            (*ps).clientNum,
                            (*pm).tracemask,
                        );
                        if trace.allsolid == 0 {
                            (*ps).pm_flags &= !PMF_DUCKED;
                        }
                    }
                }
            }

            if (*ps).pm_flags & PMF_DUCKED != 0 {
                (*pm).maxs[2] = (*ps).crouchheight as f32;
                (*ps).viewheight = CROUCH_VIEWHEIGHT;
            } else if (*ps).pm_flags & PMF_ROLLING != 0 {
                (*pm).maxs[2] = (*ps).crouchheight as f32;
                (*ps).viewheight = DEFAULT_VIEWHEIGHT;
            } else {
                (*pm).maxs[2] = (*ps).standheight as f32;
                (*ps).viewheight = DEFAULT_VIEWHEIGHT;
            }
        }
    }

    /// Raven `PM_Use`.
    /// Source: `oracle/codemp/game/bg_pmove.c:4559-4577`
    pub fn PM_Use(&mut self) {
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            if (*ps).useTime > 0 {
                (*ps).useTime -= 100; //pm->cmd.msec;
            }

            if (*ps).useTime > 0 {
                return;
            }

            if (*pm).cmd.buttons & BUTTON_USE == 0 {
                (*pm).useEvent = 0;
                (*ps).useTime = 0;
                return;
            }

            (*pm).useEvent = EV_USE as c_int;
            (*ps).useTime = USE_DELAY;
        }
    }
}

/// Raven `PM_WalkingAnim`.
///
/// Source: `oracle/codemp/game/bg_pmove.c:4579-4598`
pub fn PM_WalkingAnim(anim: c_int) -> qboolean {
    use animNumber_t::*;
    const ANIMS: &[animNumber_t] = &[
        BOTH_WALK1,          //# Normal walk
        BOTH_WALK2,          //# Normal walk with saber
        BOTH_WALK_STAFF,     //# Normal walk with staff
        BOTH_WALK_DUAL,      //# Normal walk with staff
        BOTH_WALK5,          //# Tavion taunting Kyle (cin 22)
        BOTH_WALK6,          //# Slow walk for Luke (cin 12)
        BOTH_WALK7,          //# Fast walk
        BOTH_WALKBACK1,      //# Walk1 backwards
        BOTH_WALKBACK2,      //# Walk2 backwards
        BOTH_WALKBACK_STAFF, //# Walk backwards with staff
        BOTH_WALKBACK_DUAL,  //# Walk backwards with dual
    ];
    if ANIMS.iter().any(|&a| a as c_int == anim) {
        qtrue
    } else {
        qfalse
    }
}

/// Raven `PM_RunningAnim`.
///
/// Source: `oracle/codemp/game/bg_pmove.c:4600-4620`
pub fn PM_RunningAnim(anim: c_int) -> qboolean {
    use animNumber_t::*;
    const ANIMS: &[animNumber_t] = &[
        BOTH_RUN1,
        BOTH_RUN2,
        BOTH_RUN_STAFF,
        BOTH_RUN_DUAL,
        BOTH_RUNBACK1,
        BOTH_RUNBACK2,
        BOTH_RUNBACK_STAFF,
        BOTH_RUNBACK_DUAL,
        BOTH_RUN1START,        //# Start into full run1
        BOTH_RUN1STOP,         //# Stop from full run1
        BOTH_RUNSTRAFE_LEFT1,  //# Sidestep left: should loop
        BOTH_RUNSTRAFE_RIGHT1, //# Sidestep right: should loop
    ];
    if ANIMS.iter().any(|&a| a as c_int == anim) {
        qtrue
    } else {
        qfalse
    }
}

/// Raven `PM_SwimmingAnim`.
///
/// Source: `oracle/codemp/game/bg_pmove.c:4622-4633`
pub fn PM_SwimmingAnim(anim: c_int) -> qboolean {
    use animNumber_t::*;
    const ANIMS: &[animNumber_t] = &[
        BOTH_SWIM_IDLE1,   //# Swimming Idle 1
        BOTH_SWIMFORWARD,  //# Swim forward loop
        BOTH_SWIMBACKWARD, //# Swim backward loop
    ];
    if ANIMS.iter().any(|&a| a as c_int == anim) {
        qtrue
    } else {
        qfalse
    }
}

/// Raven `PM_RollingAnim`.
///
/// Source: `oracle/codemp/game/bg_pmove.c:4635-4647`
pub fn PM_RollingAnim(anim: c_int) -> qboolean {
    use animNumber_t::*;
    const ANIMS: &[animNumber_t] = &[
        BOTH_ROLL_F, //# Roll forward
        BOTH_ROLL_B, //# Roll backward
        BOTH_ROLL_L, //# Roll left
        BOTH_ROLL_R, //# Roll right
    ];
    if ANIMS.iter().any(|&a| a as c_int == anim) {
        qtrue
    } else {
        qfalse
    }
}

/// Raven `PM_AnglesForSlope`.
///
/// Source: `oracle/codemp/game/bg_pmove.c:4649-4675`
// `angles` is a written-through out-param → `&mut [f32;3]`; `slope` stays a
// read-only by-value input. Cross-file callers are updated by the fixer.
pub fn PM_AnglesForSlope(yaw: f32, slope: vec3_t, angles: &mut [f32; 3]) {
    let mut nvf: vec3_t = [0.0; 3];
    let mut ovf: vec3_t = [0.0; 3];
    let mut ovr: vec3_t = [0.0; 3];
    let mut new_angles: vec3_t = [0.0; 3];

    // VectorSet( angles, 0, yaw, 0 )
    angles[0] = 0.0;
    angles[1] = yaw;
    angles[2] = 0.0;
    AngleVectors(*angles, Some(&mut ovf), Some(&mut ovr), None);

    vectoangles(slope, &mut new_angles);
    let pitch = new_angles[PITCH] + 90.0;
    new_angles[ROLL] = 0.0;
    new_angles[PITCH] = 0.0;

    AngleVectors(new_angles, Some(&mut nvf), None, None);

    let mut r#mod = _DotProduct(nvf, ovr);
    if r#mod < 0.0 {
        r#mod = -1.0;
    } else {
        r#mod = 1.0;
    }

    let dot = _DotProduct(nvf, ovf);

    angles[YAW] = 0.0;
    angles[PITCH] = dot * pitch;
    angles[ROLL] = (1.0 - Q_fabs(dot)) * pitch * r#mod;
}

impl PmoveContext<'_> {
    /// Raven `PM_FootSlopeTrace`.
    /// Source: `oracle/codemp/game/bg_pmove.c:4677-4740`
    pub fn PM_FootSlopeTrace(&mut self, pDiff: *mut f32, pInterval: *mut f32) {
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let mut footLOrg: vec3_t = [0.0; 3];
            let mut footROrg: vec3_t = [0.0; 3];
            let mut footLBot: vec3_t = [0.0; 3];
            let mut footRBot: vec3_t = [0.0; 3];
            let mut footLPoint: vec3_t = [0.0; 3];
            let mut footRPoint: vec3_t = [0.0; 3];
            let mut footMins: vec3_t = [0.0; 3];
            let mut footMaxs: vec3_t = [0.0; 3];
            let mut footLSlope: vec3_t = [0.0; 3];
            let mut footRSlope: vec3_t = [0.0; 3];

            let mut trace: trace_t = core::mem::zeroed();

            let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
            let mut G2Angles: vec3_t = [0.0; 3];

            VectorSet(&mut G2Angles, 0.0, (*ps).viewangles[YAW], 0.0);

            let interval = 4.0f32;

            self.traps.g2api_get_bolt_matrix(
                (*pm).ghoul2,
                0,
                (*pm).g2Bolts_LFoot,
                &mut boltMatrix,
                &G2Angles,
                &(*ps).origin,
                (*pm).cmd.serverTime,
                core::ptr::null_mut(),
                &(*pm).modelScale,
            );
            footLPoint[0] = boltMatrix.matrix[0][3];
            footLPoint[1] = boltMatrix.matrix[1][3];
            footLPoint[2] = boltMatrix.matrix[2][3];

            self.traps.g2api_get_bolt_matrix(
                (*pm).ghoul2,
                0,
                (*pm).g2Bolts_RFoot,
                &mut boltMatrix,
                &G2Angles,
                &(*ps).origin,
                (*pm).cmd.serverTime,
                core::ptr::null_mut(),
                &(*pm).modelScale,
            );
            footRPoint[0] = boltMatrix.matrix[0][3];
            footRPoint[1] = boltMatrix.matrix[1][3];
            footRPoint[2] = boltMatrix.matrix[2][3];

            _VectorCopy(footLPoint, &mut footLOrg);
            _VectorCopy(footRPoint, &mut footROrg);

            //step 2: adjust foot tag z height to bottom of bbox+1
            footLOrg[2] = (*ps).origin[2] + (*pm).mins[2] + 1.0;
            footROrg[2] = (*ps).origin[2] + (*pm).mins[2] + 1.0;
            VectorSet(
                &mut footLBot,
                footLOrg[0],
                footLOrg[1],
                footLOrg[2] - interval * 10.0,
            );
            VectorSet(
                &mut footRBot,
                footROrg[0],
                footROrg[1],
                footROrg[2] - interval * 10.0,
            );

            //step 3: trace down from each, find difference
            VectorSet(&mut footMins, -3.0, -3.0, 0.0);
            VectorSet(&mut footMaxs, 3.0, 3.0, 1.0);

            self.traps.trace(
                &mut trace,
                core::ptr::addr_of!(footLOrg) as *const vec3_t,
                core::ptr::addr_of!(footMins) as *const vec3_t,
                core::ptr::addr_of!(footMaxs) as *const vec3_t,
                core::ptr::addr_of!(footLBot) as *const vec3_t,
                (*ps).clientNum,
                (*pm).tracemask,
            );
            _VectorCopy(trace.endpos, &mut footLBot);
            _VectorCopy(trace.plane.normal, &mut footLSlope);

            self.traps.trace(
                &mut trace,
                core::ptr::addr_of!(footROrg) as *const vec3_t,
                core::ptr::addr_of!(footMins) as *const vec3_t,
                core::ptr::addr_of!(footMaxs) as *const vec3_t,
                core::ptr::addr_of!(footRBot) as *const vec3_t,
                (*ps).clientNum,
                (*pm).tracemask,
            );
            _VectorCopy(trace.endpos, &mut footRBot);
            _VectorCopy(trace.plane.normal, &mut footRSlope);

            let diff = footLBot[2] - footRBot[2];

            if !pDiff.is_null() {
                *pDiff = diff;
            }
            if !pInterval.is_null() {
                *pInterval = interval;
            }
            let _ = (footLSlope, footRSlope);
        }
    }
}

/// Raven `BG_InSlopeAnim`.
///
/// Source: `oracle/codemp/game/bg_pmove.c:4742-4800`
pub fn BG_InSlopeAnim(anim: c_int) -> qboolean {
    use animNumber_t::*;
    const ANIMS: &[animNumber_t] = &[
        LEGS_LEFTUP1,  //# On a slope with left foot 4 higher than right
        LEGS_LEFTUP2,  //# On a slope with left foot 8 higher than right
        LEGS_LEFTUP3,  //# On a slope with left foot 12 higher than right
        LEGS_LEFTUP4,  //# On a slope with left foot 16 higher than right
        LEGS_LEFTUP5,  //# On a slope with left foot 20 higher than right
        LEGS_RIGHTUP1, //# On a slope with RIGHT foot 4 higher than left
        LEGS_RIGHTUP2, //# On a slope with RIGHT foot 8 higher than left
        LEGS_RIGHTUP3, //# On a slope with RIGHT foot 12 higher than left
        LEGS_RIGHTUP4, //# On a slope with RIGHT foot 16 higher than left
        LEGS_RIGHTUP5, //# On a slope with RIGHT foot 20 higher than left
        LEGS_S1_LUP1,
        LEGS_S1_LUP2,
        LEGS_S1_LUP3,
        LEGS_S1_LUP4,
        LEGS_S1_LUP5,
        LEGS_S1_RUP1,
        LEGS_S1_RUP2,
        LEGS_S1_RUP3,
        LEGS_S1_RUP4,
        LEGS_S1_RUP5,
        LEGS_S3_LUP1,
        LEGS_S3_LUP2,
        LEGS_S3_LUP3,
        LEGS_S3_LUP4,
        LEGS_S3_LUP5,
        LEGS_S3_RUP1,
        LEGS_S3_RUP2,
        LEGS_S3_RUP3,
        LEGS_S3_RUP4,
        LEGS_S3_RUP5,
        LEGS_S4_LUP1,
        LEGS_S4_LUP2,
        LEGS_S4_LUP3,
        LEGS_S4_LUP4,
        LEGS_S4_LUP5,
        LEGS_S4_RUP1,
        LEGS_S4_RUP2,
        LEGS_S4_RUP3,
        LEGS_S4_RUP4,
        LEGS_S4_RUP5,
        LEGS_S5_LUP1,
        LEGS_S5_LUP2,
        LEGS_S5_LUP3,
        LEGS_S5_LUP4,
        LEGS_S5_LUP5,
        LEGS_S5_RUP1,
        LEGS_S5_RUP2,
        LEGS_S5_RUP3,
        LEGS_S5_RUP4,
        LEGS_S5_RUP5,
    ];
    if ANIMS.iter().any(|&a| a as c_int == anim) {
        qtrue
    } else {
        qfalse
    }
}

impl PmoveContext<'_> {
    /// Raven `PM_AdjustStandAnimForSlope`. `SLOPERECALCVAR` = `pm->ps->slopeRecalcTime`.
    /// Source: `oracle/codemp/game/bg_pmove.c:4804-5102`
    pub fn PM_AdjustStandAnimForSlope(&mut self) -> qboolean {
        use animNumber_t::*;
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;

            if (*pm).ghoul2.is_null() {
                //probably just changed models and not quite in sync yet
                return qfalse;
            }

            if (*pm).g2Bolts_LFoot == -1 || (*pm).g2Bolts_RFoot == -1 {
                //need these bolts!
                return qfalse;
            }

            //step 1: find the 2 foot tags
            let mut diff = 0.0f32;
            let mut interval = 0.0f32;
            self.PM_FootSlopeTrace(&mut diff, &mut interval);

            //step 4: choose left/right slope-match interval
            let mut destAnim;
            if diff >= interval * 5.0 {
                destAnim = LEGS_LEFTUP5 as c_int;
            } else if diff >= interval * 4.0 {
                destAnim = LEGS_LEFTUP4 as c_int;
            } else if diff >= interval * 3.0 {
                destAnim = LEGS_LEFTUP3 as c_int;
            } else if diff >= interval * 2.0 {
                destAnim = LEGS_LEFTUP2 as c_int;
            } else if diff >= interval {
                destAnim = LEGS_LEFTUP1 as c_int;
            } else if diff <= interval * -5.0 {
                destAnim = LEGS_RIGHTUP5 as c_int;
            } else if diff <= interval * -4.0 {
                destAnim = LEGS_RIGHTUP4 as c_int;
            } else if diff <= interval * -3.0 {
                destAnim = LEGS_RIGHTUP3 as c_int;
            } else if diff <= interval * -2.0 {
                destAnim = LEGS_RIGHTUP2 as c_int;
            } else if diff <= interval * -1.0 {
                destAnim = LEGS_RIGHTUP1 as c_int;
            } else {
                return qfalse;
            }

            let mut legsAnim = (*ps).legsAnim;
            //adjust for current legs anim (remap to stance family)
            if legsAnim == BOTH_STAND1 as c_int
                || (legsAnim >= LEGS_S1_LUP1 as c_int && legsAnim <= LEGS_S1_LUP5 as c_int)
                || (legsAnim >= LEGS_S1_RUP1 as c_int && legsAnim <= LEGS_S1_RUP5 as c_int)
            {
                destAnim = LEGS_S1_LUP1 as c_int + (destAnim - LEGS_LEFTUP1 as c_int);
            } else if legsAnim == BOTH_STAND2 as c_int
                || legsAnim == BOTH_SABERFAST_STANCE as c_int
                || legsAnim == BOTH_SABERSLOW_STANCE as c_int
                || legsAnim == BOTH_CROUCH1IDLE as c_int
                || legsAnim == BOTH_CROUCH1 as c_int
                || (legsAnim >= LEGS_LEFTUP1 as c_int && legsAnim <= LEGS_LEFTUP5 as c_int)
                || (legsAnim >= LEGS_RIGHTUP1 as c_int && legsAnim <= LEGS_RIGHTUP5 as c_int)
            {
                //fine
            } else if legsAnim == BOTH_STAND3 as c_int
                || (legsAnim >= LEGS_S3_LUP1 as c_int && legsAnim <= LEGS_S3_LUP5 as c_int)
                || (legsAnim >= LEGS_S3_RUP1 as c_int && legsAnim <= LEGS_S3_RUP5 as c_int)
            {
                destAnim = LEGS_S3_LUP1 as c_int + (destAnim - LEGS_LEFTUP1 as c_int);
            } else if legsAnim == BOTH_STAND4 as c_int
                || (legsAnim >= LEGS_S4_LUP1 as c_int && legsAnim <= LEGS_S4_LUP5 as c_int)
                || (legsAnim >= LEGS_S4_RUP1 as c_int && legsAnim <= LEGS_S4_RUP5 as c_int)
            {
                destAnim = LEGS_S4_LUP1 as c_int + (destAnim - LEGS_LEFTUP1 as c_int);
            } else if legsAnim == BOTH_STAND5 as c_int
                || (legsAnim >= LEGS_S5_LUP1 as c_int && legsAnim <= LEGS_S5_LUP5 as c_int)
                || (legsAnim >= LEGS_S5_RUP1 as c_int && legsAnim <= LEGS_S5_RUP5 as c_int)
            {
                destAnim = LEGS_S5_LUP1 as c_int + (destAnim - LEGS_LEFTUP1 as c_int);
            } else {
                // BOTH_STAND6 / default
                return qfalse;
            }

            let in_leftup = (legsAnim >= LEGS_LEFTUP1 as c_int
                && legsAnim <= LEGS_LEFTUP5 as c_int)
                || (legsAnim >= LEGS_S1_LUP1 as c_int && legsAnim <= LEGS_S1_LUP5 as c_int)
                || (legsAnim >= LEGS_S3_LUP1 as c_int && legsAnim <= LEGS_S3_LUP5 as c_int)
                || (legsAnim >= LEGS_S4_LUP1 as c_int && legsAnim <= LEGS_S4_LUP5 as c_int)
                || (legsAnim >= LEGS_S5_LUP1 as c_int && legsAnim <= LEGS_S5_LUP5 as c_int);
            let in_rightup = (legsAnim >= LEGS_RIGHTUP1 as c_int
                && legsAnim <= LEGS_RIGHTUP5 as c_int)
                || (legsAnim >= LEGS_S1_RUP1 as c_int && legsAnim <= LEGS_S1_RUP5 as c_int)
                || (legsAnim >= LEGS_S3_RUP1 as c_int && legsAnim <= LEGS_S3_RUP5 as c_int)
                || (legsAnim >= LEGS_S4_RUP1 as c_int && legsAnim <= LEGS_S4_RUP5 as c_int)
                || (legsAnim >= LEGS_S5_RUP1 as c_int && legsAnim <= LEGS_S5_RUP5 as c_int);

            if in_leftup {
                //already in left-side up
                if destAnim > legsAnim && (*ps).slopeRecalcTime < (*pm).cmd.serverTime {
                    legsAnim += 1;
                    (*ps).slopeRecalcTime = (*pm).cmd.serverTime + SLOPE_RECALC_INT;
                } else if destAnim < legsAnim && (*ps).slopeRecalcTime < (*pm).cmd.serverTime {
                    legsAnim -= 1;
                    (*ps).slopeRecalcTime = (*pm).cmd.serverTime + SLOPE_RECALC_INT;
                } else {
                    legsAnim = destAnim;
                }
                destAnim = legsAnim;
            } else if in_rightup {
                //already in right-side up
                if destAnim > legsAnim && (*ps).slopeRecalcTime < (*pm).cmd.serverTime {
                    legsAnim += 1;
                    (*ps).slopeRecalcTime = (*pm).cmd.serverTime + SLOPE_RECALC_INT;
                } else if destAnim < legsAnim && (*ps).slopeRecalcTime < (*pm).cmd.serverTime {
                    legsAnim -= 1;
                    (*ps).slopeRecalcTime = (*pm).cmd.serverTime + SLOPE_RECALC_INT;
                } else {
                    legsAnim = destAnim;
                }
                destAnim = legsAnim;
            } else {
                //in a stand of some sort?
                if legsAnim == BOTH_STAND1 as c_int
                    || legsAnim == TORSO_WEAPONREADY1 as c_int
                    || legsAnim == TORSO_WEAPONREADY2 as c_int
                    || legsAnim == TORSO_WEAPONREADY3 as c_int
                    || legsAnim == TORSO_WEAPONREADY10 as c_int
                {
                    if destAnim >= LEGS_S1_LUP1 as c_int && destAnim <= LEGS_S1_LUP5 as c_int {
                        destAnim = LEGS_S1_LUP1 as c_int;
                        (*ps).slopeRecalcTime = (*pm).cmd.serverTime + SLOPE_RECALC_INT;
                    } else if destAnim >= LEGS_S1_RUP1 as c_int && destAnim <= LEGS_S1_RUP5 as c_int
                    {
                        destAnim = LEGS_S1_RUP1 as c_int;
                        (*ps).slopeRecalcTime = (*pm).cmd.serverTime + SLOPE_RECALC_INT;
                    } else {
                        return qfalse;
                    }
                } else if legsAnim == BOTH_STAND2 as c_int
                    || legsAnim == BOTH_SABERFAST_STANCE as c_int
                    || legsAnim == BOTH_SABERSLOW_STANCE as c_int
                    || legsAnim == BOTH_CROUCH1IDLE as c_int
                {
                    if destAnim >= LEGS_LEFTUP1 as c_int && destAnim <= LEGS_LEFTUP5 as c_int {
                        destAnim = LEGS_LEFTUP1 as c_int;
                        (*ps).slopeRecalcTime = (*pm).cmd.serverTime + SLOPE_RECALC_INT;
                    } else if destAnim >= LEGS_RIGHTUP1 as c_int
                        && destAnim <= LEGS_RIGHTUP5 as c_int
                    {
                        destAnim = LEGS_RIGHTUP1 as c_int;
                        (*ps).slopeRecalcTime = (*pm).cmd.serverTime + SLOPE_RECALC_INT;
                    } else {
                        return qfalse;
                    }
                } else if legsAnim == BOTH_STAND3 as c_int {
                    if destAnim >= LEGS_S3_LUP1 as c_int && destAnim <= LEGS_S3_LUP5 as c_int {
                        destAnim = LEGS_S3_LUP1 as c_int;
                        (*ps).slopeRecalcTime = (*pm).cmd.serverTime + SLOPE_RECALC_INT;
                    } else if destAnim >= LEGS_S3_RUP1 as c_int && destAnim <= LEGS_S3_RUP5 as c_int
                    {
                        destAnim = LEGS_S3_RUP1 as c_int;
                        (*ps).slopeRecalcTime = (*pm).cmd.serverTime + SLOPE_RECALC_INT;
                    } else {
                        return qfalse;
                    }
                } else if legsAnim == BOTH_STAND4 as c_int {
                    if destAnim >= LEGS_S4_LUP1 as c_int && destAnim <= LEGS_S4_LUP5 as c_int {
                        destAnim = LEGS_S4_LUP1 as c_int;
                        (*ps).slopeRecalcTime = (*pm).cmd.serverTime + SLOPE_RECALC_INT;
                    } else if destAnim >= LEGS_S4_RUP1 as c_int && destAnim <= LEGS_S4_RUP5 as c_int
                    {
                        destAnim = LEGS_S4_RUP1 as c_int;
                        (*ps).slopeRecalcTime = (*pm).cmd.serverTime + SLOPE_RECALC_INT;
                    } else {
                        return qfalse;
                    }
                } else if legsAnim == BOTH_STAND5 as c_int {
                    if destAnim >= LEGS_S5_LUP1 as c_int && destAnim <= LEGS_S5_LUP5 as c_int {
                        destAnim = LEGS_S5_LUP1 as c_int;
                        (*ps).slopeRecalcTime = (*pm).cmd.serverTime + SLOPE_RECALC_INT;
                    } else if destAnim >= LEGS_S5_RUP1 as c_int && destAnim <= LEGS_S5_RUP5 as c_int
                    {
                        destAnim = LEGS_S5_RUP1 as c_int;
                        (*ps).slopeRecalcTime = (*pm).cmd.serverTime + SLOPE_RECALC_INT;
                    } else {
                        return qfalse;
                    }
                } else {
                    // BOTH_STAND6 / default
                    return qfalse;
                }
            }
            //step 7: set the anim
            self.PM_ContinueLegsAnim(destAnim);

            qtrue
        }
    }

    /// Raven `PM_LegsSlopeBackTransition`.
    /// Source: `oracle/codemp/game/bg_pmove.c:5107-5168`
    pub fn PM_LegsSlopeBackTransition(&mut self, desiredAnim: c_int) -> c_int {
        use animNumber_t::*;
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let anim = (*ps).legsAnim;
            let mut resultingAnim = desiredAnim;

            let match_case = (anim >= LEGS_LEFTUP2 as c_int && anim <= LEGS_LEFTUP5 as c_int)
                || (anim >= LEGS_RIGHTUP2 as c_int && anim <= LEGS_RIGHTUP5 as c_int)
                || (anim >= LEGS_S1_LUP2 as c_int && anim <= LEGS_S1_LUP5 as c_int)
                || (anim >= LEGS_S1_RUP2 as c_int && anim <= LEGS_S1_RUP5 as c_int)
                || (anim >= LEGS_S3_LUP2 as c_int && anim <= LEGS_S3_LUP5 as c_int)
                || (anim >= LEGS_S3_RUP2 as c_int && anim <= LEGS_S3_RUP5 as c_int)
                || (anim >= LEGS_S4_LUP2 as c_int && anim <= LEGS_S4_LUP5 as c_int)
                || (anim >= LEGS_S4_RUP2 as c_int && anim <= LEGS_S4_RUP5 as c_int)
                || (anim >= LEGS_S5_LUP2 as c_int && anim <= LEGS_S5_LUP5 as c_int)
                || (anim >= LEGS_S5_RUP2 as c_int && anim <= LEGS_S5_RUP5 as c_int);

            if match_case {
                if (*ps).slopeRecalcTime < (*pm).cmd.serverTime {
                    resultingAnim = anim - 1;
                    (*ps).slopeRecalcTime = (*pm).cmd.serverTime + 8; //SLOPE_RECALC_INT
                } else {
                    resultingAnim = anim;
                }
                VectorClear(&mut (*ps).velocity);
            }

            resultingAnim
        }
    }
}

impl PmoveContext<'_> {
    /// Raven `PM_Footsteps`.
    /// Source: `oracle/codemp/game/bg_pmove.c:5175-5661`
    pub fn PM_Footsteps(&mut self) {
        use animNumber_t::*;
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let mut bobmove = 0.0f32;
            let mut footstep = qfalse;
            let mut setAnimFlags = 0;

            let la = (*ps).legsAnim;
            if (PM_InSaberAnim(la) != qfalse && BG_SpinningSaberAnim(la) == qfalse)
                || la == BOTH_STAND1 as c_int
                || la == BOTH_STAND1TO2 as c_int
                || la == BOTH_STAND2TO1 as c_int
                || la == BOTH_STAND2 as c_int
                || la == BOTH_SABERFAST_STANCE as c_int
                || la == BOTH_SABERSLOW_STANCE as c_int
                || la == BOTH_BUTTON_HOLD as c_int
                || la == BOTH_BUTTON_RELEASE as c_int
                || PM_LandingAnim(la) != qfalse
                || PM_PainAnim(la) != qfalse
            {
                //legs are in a saber anim, and not spinning, be sure to override it
                setAnimFlags |= SETANIM_FLAG_OVERRIDE;
            }

            // calculate speed and cycle
            // sqrt is the double libm call rounded back to float; an f32 sqrt
            // double-rounds and diverges from the oracle.
            (*pm).xyspeed = (((*ps).velocity[0] * (*ps).velocity[0]
                + (*ps).velocity[1] * (*ps).velocity[1]) as f64)
                .sqrt() as f32;

            if (*ps).saberMove == LS_SPINATTACK as c_int {
                self.PM_ContinueLegsAnim((*ps).torsoAnim);
            } else if (*ps).groundEntityNum == ENTITYNUM_NONE {
                // airborne leaves position in cycle intact, but doesn't advance
                if (*pm).waterlevel > 1 {
                    if (*pm).xyspeed > 60.0 {
                        self.PM_ContinueLegsAnim(BOTH_SWIMFORWARD as c_int);
                    } else {
                        self.PM_ContinueLegsAnim(BOTH_SWIM_IDLE1 as c_int);
                    }
                }
                return;
            }
            // if not trying to move
            else if (*pm).cmd.forwardmove == 0 && (*pm).cmd.rightmove == 0 {
                if (*pm).xyspeed < 5.0 {
                    (*ps).bobCycle = 0; // start at beginning of cycle again
                    if (*ps).clientNum >= MAX_CLIENTS as c_int
                        && !self.pm_entSelf.is_null()
                        && (*self.pm_entSelf).s.NPC_class == CLASS_RANCOR as c_int
                    {
                        if (*ps).eFlags2 & EF2_USE_ALT_ANIM != 0 {
                            self.PM_ContinueLegsAnim(BOTH_STAND4 as c_int);
                        } else if (*ps).eFlags2 & EF2_ALERTED != 0 {
                            self.PM_ContinueLegsAnim(BOTH_STAND2 as c_int);
                        } else {
                            self.PM_ContinueLegsAnim(BOTH_STAND1 as c_int);
                        }
                    } else if (*ps).clientNum >= MAX_CLIENTS as c_int
                        && !self.pm_entSelf.is_null()
                        && (*self.pm_entSelf).s.NPC_class == CLASS_WAMPA as c_int
                    {
                        if (*ps).eFlags2 & EF2_USE_ALT_ANIM != 0 {
                            self.PM_ContinueLegsAnim(BOTH_STAND2 as c_int);
                        } else {
                            self.PM_ContinueLegsAnim(BOTH_STAND1 as c_int);
                        }
                    } else if (*ps).pm_flags & PMF_DUCKED != 0 || (*ps).pm_flags & PMF_ROLLING != 0
                    {
                        if (*ps).legsAnim != BOTH_CROUCH1IDLE as c_int {
                            self.PM_SetAnim(
                                SETANIM_LEGS,
                                BOTH_CROUCH1IDLE as c_int,
                                setAnimFlags,
                                100,
                            );
                        } else {
                            self.PM_ContinueLegsAnim(BOTH_CROUCH1IDLE as c_int);
                        }
                    } else {
                        if (*ps).weapon == WP_DISRUPTOR as c_int && (*ps).zoomMode == 1 {
                            self.PM_ContinueLegsAnim(TORSO_WEAPONREADY4 as c_int);
                        } else {
                            if (*ps).weapon == WP_SABER as c_int && BG_SabersOff(ps) != qfalse {
                                if self.PM_AdjustStandAnimForSlope() == qfalse {
                                    let a = self.PM_LegsSlopeBackTransition(BOTH_STAND1 as c_int);
                                    self.PM_ContinueLegsAnim(a);
                                }
                            } else {
                                if (*ps).weapon != WP_SABER as c_int
                                    || self.PM_AdjustStandAnimForSlope() == qfalse
                                {
                                    if (*ps).weapon == WP_SABER as c_int {
                                        let st = self.PM_GetSaberStance();
                                        let a = self.PM_LegsSlopeBackTransition(st);
                                        self.PM_ContinueLegsAnim(a);
                                    } else {
                                        let a = self.PM_LegsSlopeBackTransition(
                                            WeaponReadyLegsAnim[(*ps).weapon as usize],
                                        );
                                        self.PM_ContinueLegsAnim(a);
                                    }
                                }
                            }
                        }
                    }
                }
                return;
            }

            let _ = footstep;
            footstep = qfalse;

            if (*ps).saberMove == LS_SPINATTACK as c_int {
                bobmove = 0.2;
                self.PM_ContinueLegsAnim((*ps).torsoAnim);
            } else if (*ps).pm_flags & PMF_DUCKED != 0 {
                let mut rolled = 0;

                bobmove = 0.5; // ducked characters bob much faster

                if ((PM_RunningAnim((*ps).legsAnim) != qfalse
                    && VectorLengthSquared((*ps).velocity) >= 40000.0)
                    || PM_CanRollFromSoulCal(ps) != qfalse)
                    && BG_InRoll(ps, (*ps).legsAnim) == qfalse
                {
                    //roll!
                    rolled = self.PM_TryRoll();
                }
                if rolled == 0 {
                    //standard crouching anim stuff
                    if (*ps).pm_flags & PMF_BACKWARDS_RUN != 0 {
                        if (*ps).legsAnim != BOTH_CROUCH1WALKBACK as c_int {
                            self.PM_SetAnim(
                                SETANIM_LEGS,
                                BOTH_CROUCH1WALKBACK as c_int,
                                setAnimFlags,
                                100,
                            );
                        } else {
                            self.PM_ContinueLegsAnim(BOTH_CROUCH1WALKBACK as c_int);
                        }
                    } else {
                        if (*ps).legsAnim != BOTH_CROUCH1WALK as c_int {
                            self.PM_SetAnim(
                                SETANIM_LEGS,
                                BOTH_CROUCH1WALK as c_int,
                                setAnimFlags,
                                100,
                            );
                        } else {
                            self.PM_ContinueLegsAnim(BOTH_CROUCH1WALK as c_int);
                        }
                    }
                } else {
                    //send us into the roll
                    (*ps).legsTimer = 0;
                    (*ps).legsAnim = 0;
                    self.PM_SetAnim(
                        SETANIM_BOTH,
                        rolled,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        150,
                    );
                    self.PM_AddEventWithParm(EV_ROLL as c_int, 0);
                    (*pm).maxs[2] = (*ps).crouchheight as f32;
                    (*ps).viewheight = DEFAULT_VIEWHEIGHT;
                    (*ps).pm_flags &= !PMF_DUCKED;
                    (*ps).pm_flags |= PMF_ROLLING;
                }
            } else if (*ps).pm_flags & PMF_ROLLING != 0
                && BG_InRoll(ps, (*ps).legsAnim) == qfalse
                && PM_InRollComplete(ps, (*ps).legsAnim) == qfalse
            {
                bobmove = 0.5; // ducked characters bob much faster

                if (*ps).pm_flags & PMF_BACKWARDS_RUN != 0 {
                    if (*ps).legsAnim != BOTH_CROUCH1WALKBACK as c_int {
                        self.PM_SetAnim(
                            SETANIM_LEGS,
                            BOTH_CROUCH1WALKBACK as c_int,
                            setAnimFlags,
                            100,
                        );
                    } else {
                        self.PM_ContinueLegsAnim(BOTH_CROUCH1WALKBACK as c_int);
                    }
                } else {
                    if (*ps).legsAnim != BOTH_CROUCH1WALK as c_int {
                        self.PM_SetAnim(SETANIM_LEGS, BOTH_CROUCH1WALK as c_int, setAnimFlags, 100);
                    } else {
                        self.PM_ContinueLegsAnim(BOTH_CROUCH1WALK as c_int);
                    }
                }
            } else {
                let mut desiredAnim = -1;

                let sl = (*ps).fd.saberAnimLevel;
                if ((*ps).legsAnim == BOTH_FORCELAND1 as c_int
                    || (*ps).legsAnim == BOTH_FORCELANDBACK1 as c_int
                    || (*ps).legsAnim == BOTH_FORCELANDRIGHT1 as c_int
                    || (*ps).legsAnim == BOTH_FORCELANDLEFT1 as c_int)
                    && (*ps).legsTimer > 0
                {
                    //let it finish first
                    bobmove = 0.2;
                } else if (*pm).cmd.buttons & BUTTON_WALKING == 0 {
                    //running
                    bobmove = 0.4;
                    if (*ps).clientNum >= MAX_CLIENTS as c_int
                        && !self.pm_entSelf.is_null()
                        && (*self.pm_entSelf).s.NPC_class == CLASS_WAMPA as c_int
                    {
                        if (*ps).eFlags2 & EF2_USE_ALT_ANIM != 0 {
                            desiredAnim = BOTH_RUN1 as c_int;
                        } else {
                            desiredAnim = BOTH_RUN2 as c_int;
                        }
                    } else if (*ps).clientNum >= MAX_CLIENTS as c_int
                        && !self.pm_entSelf.is_null()
                        && (*self.pm_entSelf).s.NPC_class == CLASS_RANCOR as c_int
                    {
                        //no run anims
                        if (*ps).pm_flags & PMF_BACKWARDS_RUN != 0 {
                            desiredAnim = BOTH_WALKBACK1 as c_int;
                        } else {
                            desiredAnim = BOTH_WALK1 as c_int;
                        }
                    } else if (*ps).pm_flags & PMF_BACKWARDS_RUN != 0 {
                        if sl == saber_styles_t::SS_STAFF as c_int {
                            if (*ps).saberHolstered > 1 {
                                desiredAnim = BOTH_RUNBACK1 as c_int;
                            } else {
                                desiredAnim = BOTH_RUNBACK2 as c_int;
                            }
                        } else if sl == saber_styles_t::SS_DUAL as c_int {
                            if (*ps).saberHolstered > 1 {
                                desiredAnim = BOTH_RUNBACK1 as c_int;
                            } else {
                                desiredAnim = BOTH_RUNBACK2 as c_int;
                            }
                        } else {
                            if (*ps).saberHolstered != 0 {
                                desiredAnim = BOTH_RUNBACK1 as c_int;
                            } else {
                                desiredAnim = BOTH_RUNBACK2 as c_int;
                            }
                        }
                    } else {
                        if sl == saber_styles_t::SS_STAFF as c_int {
                            if (*ps).saberHolstered > 1 {
                                desiredAnim = BOTH_RUN1 as c_int;
                            } else if (*ps).saberHolstered == 1 {
                                desiredAnim = BOTH_RUN2 as c_int;
                            } else {
                                if (*ps).fd.forcePowersActive & (1 << FP_SPEED) != 0 {
                                    desiredAnim = BOTH_RUN1 as c_int;
                                } else {
                                    desiredAnim = BOTH_RUN_STAFF as c_int;
                                }
                            }
                        } else if sl == saber_styles_t::SS_DUAL as c_int {
                            if (*ps).saberHolstered > 1 {
                                desiredAnim = BOTH_RUN1 as c_int;
                            } else if (*ps).saberHolstered == 1 {
                                desiredAnim = BOTH_RUN2 as c_int;
                            } else {
                                desiredAnim = BOTH_RUN_DUAL as c_int;
                            }
                        } else {
                            if (*ps).saberHolstered != 0 {
                                desiredAnim = BOTH_RUN1 as c_int;
                            } else {
                                desiredAnim = BOTH_RUN2 as c_int;
                            }
                        }
                    }
                    footstep = qtrue;
                } else {
                    bobmove = 0.2; // walking bobs slow
                    if (*ps).pm_flags & PMF_BACKWARDS_RUN != 0 {
                        if sl == saber_styles_t::SS_STAFF as c_int {
                            if (*ps).saberHolstered > 1 {
                                desiredAnim = BOTH_WALKBACK1 as c_int;
                            } else if (*ps).saberHolstered != 0 {
                                desiredAnim = BOTH_WALKBACK2 as c_int;
                            } else {
                                desiredAnim = BOTH_WALKBACK_STAFF as c_int;
                            }
                        } else if sl == saber_styles_t::SS_DUAL as c_int {
                            if (*ps).saberHolstered > 1 {
                                desiredAnim = BOTH_WALKBACK1 as c_int;
                            } else if (*ps).saberHolstered != 0 {
                                desiredAnim = BOTH_WALKBACK2 as c_int;
                            } else {
                                desiredAnim = BOTH_WALKBACK_DUAL as c_int;
                            }
                        } else {
                            if (*ps).saberHolstered != 0 {
                                desiredAnim = BOTH_WALKBACK1 as c_int;
                            } else {
                                desiredAnim = BOTH_WALKBACK2 as c_int;
                            }
                        }
                    } else {
                        if (*ps).weapon == WP_MELEE as c_int {
                            desiredAnim = BOTH_WALK1 as c_int;
                        } else if BG_SabersOff(ps) != qfalse {
                            desiredAnim = BOTH_WALK1 as c_int;
                        } else {
                            if sl == saber_styles_t::SS_STAFF as c_int {
                                if (*ps).saberHolstered > 1 {
                                    desiredAnim = BOTH_WALK1 as c_int;
                                } else if (*ps).saberHolstered != 0 {
                                    desiredAnim = BOTH_WALK2 as c_int;
                                } else {
                                    desiredAnim = BOTH_WALK_STAFF as c_int;
                                }
                            } else if sl == saber_styles_t::SS_DUAL as c_int {
                                if (*ps).saberHolstered > 1 {
                                    desiredAnim = BOTH_WALK1 as c_int;
                                } else if (*ps).saberHolstered != 0 {
                                    desiredAnim = BOTH_WALK2 as c_int;
                                } else {
                                    desiredAnim = BOTH_WALK_DUAL as c_int;
                                }
                            } else {
                                if (*ps).saberHolstered != 0 {
                                    desiredAnim = BOTH_WALK1 as c_int;
                                } else {
                                    desiredAnim = BOTH_WALK2 as c_int;
                                }
                            }
                        }
                    }
                }

                if desiredAnim != -1 {
                    let ires = self.PM_LegsSlopeBackTransition(desiredAnim);

                    if (*ps).legsAnim != desiredAnim && ires == desiredAnim {
                        self.PM_SetAnim(SETANIM_LEGS, desiredAnim, setAnimFlags, 100);
                    } else {
                        self.PM_ContinueLegsAnim(ires);
                    }
                }
            }

            // check for footstep / splash sounds
            let old = (*ps).bobCycle;
            (*ps).bobCycle = ((old as f32 + bobmove * self.pml.msec as f32) as c_int) & 255;

            // if we just crossed a cycle boundary, play an appropriate footstep event
            if ((old + 64) ^ ((*ps).bobCycle + 64)) & 128 != 0 {
                (*ps).footstepTime = (*pm).cmd.serverTime + 300;
                if (*pm).waterlevel == 1 {
                    // splashing
                    self.PM_AddEvent(EV_FOOTSPLASH as c_int);
                } else if (*pm).waterlevel == 2 {
                    // wading / swimming at surface
                    self.PM_AddEvent(EV_SWIM as c_int);
                } else if (*pm).waterlevel == 3 {
                    // no sound when completely underwater
                }
            }
            let _ = footstep;
        }
    }
}

impl PmoveContext<'_> {
    /// Raven `PM_WaterEvents`. `QAGAME` is defined, so the impact-splash block is compiled.
    /// Source: `oracle/codemp/game/bg_pmove.c:5670-5748`
    pub fn PM_WaterEvents(&mut self) {
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let mut impact_splash = qfalse;

            // if just entered a water volume, play a sound
            if self.pml.previous_waterlevel == 0 && (*pm).waterlevel != 0 {
                if VectorLengthSquared((*ps).velocity) > 40000.0 {
                    impact_splash = qtrue;
                }
                self.PM_AddEvent(EV_WATER_TOUCH as c_int);
            }

            // if just completely exited a water volume, play a sound
            if self.pml.previous_waterlevel != 0 && (*pm).waterlevel == 0 {
                if VectorLengthSquared((*ps).velocity) > 40000.0 {
                    impact_splash = qtrue;
                }
                self.PM_AddEvent(EV_WATER_LEAVE as c_int);
            }

            if impact_splash != qfalse {
                //play the splash effect
                let mut tr: trace_t = core::mem::zeroed();
                let mut start: vec3_t = [0.0; 3];
                let mut end: vec3_t = [0.0; 3];

                _VectorCopy((*ps).origin, &mut start);
                _VectorCopy((*ps).origin, &mut end);

                start[2] += 10.0;
                end[2] -= 40.0;

                let vec3_origin_local = vec3_origin;
                self.traps.trace(
                    &mut tr,
                    core::ptr::addr_of!(start) as *const vec3_t,
                    core::ptr::addr_of!(vec3_origin_local) as *const vec3_t,
                    core::ptr::addr_of!(vec3_origin_local) as *const vec3_t,
                    core::ptr::addr_of!(end) as *const vec3_t,
                    (*ps).clientNum,
                    MASK_WATER,
                );

                if tr.fraction < 1.0 {
                    let fx = if tr.contents & CONTENTS_LAVA != 0 {
                        EFFECT_LAVA_SPLASH as c_int
                    } else if tr.contents & CONTENTS_SLIME != 0 {
                        EFFECT_ACID_SPLASH as c_int
                    } else {
                        EFFECT_WATER_SPLASH as c_int
                    };
                    self.callbacks.play_effect(
                        fx,
                        core::ptr::addr_of!(tr.endpos),
                        core::ptr::addr_of!(tr.plane.normal),
                    );
                }
            }

            // check for head just going under water
            if self.pml.previous_waterlevel != 3 && (*pm).waterlevel == 3 {
                self.PM_AddEvent(EV_WATER_UNDER as c_int);
            }

            // check for head just coming out of water
            if self.pml.previous_waterlevel == 3 && (*pm).waterlevel != 3 {
                self.PM_AddEvent(EV_WATER_CLEAR as c_int);
            }
        }
    }
}

/// Raven `BG_ClearRocketLock`.
///
/// Source: `oracle/codemp/game/bg_pmove.c:5750-5759`
pub fn BG_ClearRocketLock(ps: *mut playerState_t) {
    unsafe {
        if !ps.is_null() {
            (*ps).rocketLockIndex = ENTITYNUM_NONE;
            (*ps).rocketLastValidTime = 0.0;
            (*ps).rocketLockTime = -1.0;
            (*ps).rocketTargetTime = 0.0;
        }
    }
}

impl PmoveContext<'_> {
    /// Raven `PM_BeginWeaponChange`.
    /// Source: `oracle/codemp/game/bg_pmove.c:5766-5793`
    pub fn PM_BeginWeaponChange(&mut self, weapon: c_int) {
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            if weapon <= WP_NONE as c_int || weapon >= WP_NUM_WEAPONS as c_int {
                return;
            }

            if (*ps).stats[statIndex_t::STAT_WEAPONS as usize] & (1 << weapon) == 0 {
                return;
            }

            if (*ps).weaponstate == WEAPON_DROPPING as c_int {
                return;
            }

            // turn off any kind of zooming when weapon switching.
            if (*ps).zoomMode != 0 {
                (*ps).zoomMode = 0;
                (*ps).zoomTime = (*ps).commandTime;
            }

            self.PM_AddEventWithParm(EV_CHANGE_WEAPON as c_int, weapon);
            (*ps).weaponstate = WEAPON_DROPPING as c_int;
            (*ps).weaponTime += 200;
            self.PM_SetAnim(
                SETANIM_TORSO,
                TORSO_DROPWEAP1 as c_int,
                SETANIM_FLAG_OVERRIDE,
                0,
            );

            BG_ClearRocketLock((*pm).ps);
        }
    }

    /// Raven `PM_FinishWeaponChange`.
    /// Source: `oracle/codemp/game/bg_pmove.c:5801-5825`
    pub fn PM_FinishWeaponChange(&mut self) {
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let mut weapon = (*pm).cmd.weapon as c_int;
            if weapon < WP_NONE as c_int || weapon >= WP_NUM_WEAPONS as c_int {
                weapon = WP_NONE as c_int;
            }

            if (*ps).stats[statIndex_t::STAT_WEAPONS as usize] & (1 << weapon) == 0 {
                weapon = WP_NONE as c_int;
            }

            if weapon == WP_SABER as c_int {
                self.PM_SetSaberMove(LS_DRAW as c_short);
            } else {
                self.PM_SetAnim(
                    SETANIM_TORSO,
                    TORSO_RAISEWEAP1 as c_int,
                    SETANIM_FLAG_OVERRIDE,
                    0,
                );
            }
            (*ps).weapon = weapon;
            (*ps).weaponstate = WEAPON_RAISING as c_int;
            (*ps).weaponTime += 250;
        }
    }
}

/// Raven `MAX_XHAIR_DIST_ACCURACY` — auto-aim crosshair trace reach. bg_pmove
/// keeps its own private `#define`; `g_weapon.c` and `cg_draw.c` each define an
/// identical private copy (the game-tier twin lives in `g_weapon.rs`).
/// Source: `oracle/codemp/game/bg_pmove.c:5832`
const MAX_XHAIR_DIST_ACCURACY: f32 = 20000.0;

/// Raven `BG_VehTraceFromCamPos`. `QAGAME` is defined, so the game-side branch is compiled.
/// Source: `oracle/codemp/game/bg_pmove.c:5833-5872`
pub fn BG_VehTraceFromCamPos(
    camTrace: *mut trace_t,
    bgEnt: *mut bgEntity_t,
    entOrg: vec3_t,
    shotStart: vec3_t,
    end: vec3_t,
    newEnd: &mut vec3_t,
    shotDir: &mut vec3_t,
    bestDist: f32,
    bg: &BgState,
    traps: &dyn BgTraps,
    callbacks: &mut dyn GameCallbacks,
) -> c_int {
    let _ = bg;
    unsafe {
        let mut viewDir2End: vec3_t = [0.0; 3];
        let mut extraEnd: vec3_t = [0.0; 3];
        let mut camPos: vec3_t = [0.0; 3];

        let veh = (*bgEnt).m_pVehicle as *mut Vehicle_t;
        // QAGAME: `WP_GetVehicleCamPos` is a game-tier body (needs `GameContext`
        // for `G_EstimateCamPos`); reached by entity number through the upcall.
        callbacks.wp_get_vehicle_cam_pos(
            (*bgEnt).s.number,
            // `m_pVehicle->m_pPilot` is already `*mut bgEntity_t`; the old
            // `as *mut gentity_t` cast was spurious. `.s.number` is bg-visible.
            // Source: `oracle/codemp/game/bg_pmove.c` (WP_GetVehicleCamPos call site)
            (*(*veh).m_pPilot).s.number,
            &mut camPos as *mut vec3_t,
        );

        let minAutoAimDist =
            Distance(entOrg, camPos) + ((*(*veh).m_pVehicleInfo).length / 2.0) + 200.0;

        _VectorCopy(end, newEnd);
        _VectorSubtract(end, camPos, &mut viewDir2End);
        VectorNormalize(&mut viewDir2End);
        _VectorMA(camPos, MAX_XHAIR_DIST_ACCURACY, viewDir2End, &mut extraEnd);

        // QAGAME
        let vec3_origin_local = vec3_origin;
        traps.trace(
            camTrace,
            core::ptr::addr_of!(camPos) as *const vec3_t,
            core::ptr::addr_of!(vec3_origin_local) as *const vec3_t,
            core::ptr::addr_of!(vec3_origin_local) as *const vec3_t,
            core::ptr::addr_of!(extraEnd) as *const vec3_t,
            (*bgEnt).s.number,
            CONTENTS_SOLID | CONTENTS_BODY,
        );

        if (*camTrace).allsolid == 0
            && (*camTrace).startsolid == 0
            && (*camTrace).fraction < 1.0
            && ((*camTrace).fraction * MAX_XHAIR_DIST_ACCURACY) > minAutoAimDist
            && (((*camTrace).fraction * MAX_XHAIR_DIST_ACCURACY) - Distance(entOrg, camPos))
                < bestDist
        {
            //this trace hit something closer than the main trace hit, so use this result instead
            _VectorCopy((*camTrace).endpos, newEnd);
            _VectorSubtract(*newEnd, shotStart, shotDir);
            VectorNormalize(shotDir);
            return (*camTrace).entityNum as c_int + 1;
        }
        0
    }
}

impl PmoveContext<'_> {
    /// Raven `PM_RocketLock`.
    /// Source: `oracle/codemp/game/bg_pmove.c:5874-5977`
    pub fn PM_RocketLock(&mut self, lockDist: f32, vehicleLock: qboolean) {
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let mut ang: vec3_t = [0.0; 3];
            let mut tr: trace_t = core::mem::zeroed();

            let mut muzzleOffPoint: vec3_t = [0.0; 3];
            let mut muzzlePoint: vec3_t = [0.0; 3];
            let mut forward: vec3_t = [0.0; 3];
            let mut right: vec3_t = [0.0; 3];
            let mut up: vec3_t = [0.0; 3];

            if vehicleLock != qfalse {
                AngleVectors(
                    (*ps).viewangles,
                    Some(&mut forward),
                    Some(&mut right),
                    Some(&mut up),
                );
                _VectorCopy((*ps).origin, &mut muzzlePoint);
                _VectorMA(muzzlePoint, lockDist, forward, &mut ang);
            } else {
                AngleVectors(
                    (*ps).viewangles,
                    Some(&mut forward),
                    Some(&mut right),
                    Some(&mut up),
                );

                AngleVectors((*ps).viewangles, Some(&mut ang), None, None);

                _VectorCopy((*ps).origin, &mut muzzlePoint);
                _VectorCopy(
                    WP_MuzzlePoint[WP_ROCKET_LAUNCHER as usize],
                    &mut muzzleOffPoint,
                );

                let mp = muzzlePoint;
                _VectorMA(mp, muzzleOffPoint[0], forward, &mut muzzlePoint);
                let mp = muzzlePoint;
                _VectorMA(mp, muzzleOffPoint[1], right, &mut muzzlePoint);
                muzzlePoint[2] += (*ps).viewheight as f32 + muzzleOffPoint[2];
                ang[0] = muzzlePoint[0] + ang[0] * lockDist;
                ang[1] = muzzlePoint[1] + ang[1] * lockDist;
                ang[2] = muzzlePoint[2] + ang[2] * lockDist;
            }

            self.traps.trace(
                &mut tr,
                core::ptr::addr_of!(muzzlePoint) as *const vec3_t,
                core::ptr::null(),
                core::ptr::null(),
                core::ptr::addr_of!(ang) as *const vec3_t,
                (*ps).clientNum,
                MASK_PLAYERSOLID,
            );

            if vehicleLock != qfalse {
                //vehicles also do a trace from the camera point if the main one misses
                if tr.fraction >= 1.0 {
                    let mut camTrace: trace_t = core::mem::zeroed();
                    let mut newEnd: vec3_t = [0.0; 3];
                    let mut shotDir: vec3_t = [0.0; 3];
                    let ent = self.PM_BGEntForNum((*ps).clientNum);
                    if BG_VehTraceFromCamPos(
                        &mut camTrace,
                        ent,
                        (*ps).origin,
                        muzzlePoint,
                        tr.endpos,
                        &mut newEnd,
                        &mut shotDir,
                        tr.fraction * lockDist,
                        self.bg,
                        self.traps,
                        self.callbacks,
                    ) != 0
                    {
                        tr = camTrace;
                    }
                }
            }

            if tr.fraction != 1.0
                && (tr.entityNum as c_int) < ENTITYNUM_NONE
                && tr.entityNum as c_int != (*ps).clientNum
            {
                let bgEnt = self.PM_BGEntForNum(tr.entityNum as c_int);
                // Preserved oracle quirk: masks with the raw PW_CLOAKED value (11),
                // not `1 << PW_CLOAKED` — so it tests bits 0/1/3, not bit 11. This is
                // the only cloak test in the tree using the shift-less form
                // (bg_pmove.c:5925); every other site uses `1 << PW_CLOAKED`.
                if !bgEnt.is_null() && (*bgEnt).s.powerups & PW_CLOAKED != 0 {
                    (*ps).rocketLockIndex = ENTITYNUM_NONE;
                    (*ps).rocketLockTime = 0.0;
                } else if !bgEnt.is_null()
                    && ((*bgEnt).s.eType == entityType_t::ET_PLAYER as c_int
                        || (*bgEnt).s.eType == entityType_t::ET_NPC as c_int)
                {
                    if (*ps).rocketLockIndex == ENTITYNUM_NONE {
                        (*ps).rocketLockIndex = tr.entityNum as c_int;
                        (*ps).rocketLockTime = (*pm).cmd.serverTime as f32;
                    } else if (*ps).rocketLockIndex != tr.entityNum as c_int
                        && (*ps).rocketTargetTime < (*pm).cmd.serverTime as f32
                    {
                        (*ps).rocketLockIndex = tr.entityNum as c_int;
                        (*ps).rocketLockTime = (*pm).cmd.serverTime as f32;
                    } else if (*ps).rocketLockIndex == tr.entityNum as c_int {
                        if (*ps).rocketLockTime == -1.0 {
                            (*ps).rocketLockTime = (*ps).rocketLastValidTime;
                        }
                    }

                    if (*ps).rocketLockIndex == tr.entityNum as c_int {
                        (*ps).rocketTargetTime = ((*pm).cmd.serverTime + 500) as f32;
                    }
                } else if vehicleLock == qfalse {
                    if (*ps).rocketTargetTime < (*pm).cmd.serverTime as f32 {
                        (*ps).rocketLockIndex = ENTITYNUM_NONE;
                        (*ps).rocketLockTime = 0.0;
                    }
                }
            } else if (*ps).rocketTargetTime < (*pm).cmd.serverTime as f32 {
                (*ps).rocketLockIndex = ENTITYNUM_NONE;
                (*ps).rocketLockTime = 0.0;
            } else {
                if (*ps).rocketLockTime != -1.0 {
                    (*ps).rocketLastValidTime = (*ps).rocketLockTime;
                }
                (*ps).rocketLockTime = -1.0;
            }
        }
    }
}

impl PmoveContext<'_> {
    /// Raven `PM_DoChargedWeapons`. `_DEBUG` prints are dropped. The C `goto rest` is
    /// modeled with a labeled block whose value short-circuits the `return qtrue`.
    /// Source: `oracle/codemp/game/bg_pmove.c:5980-6233`
    pub fn PM_DoChargedWeapons(
        &mut self,
        vehicleRocketLock: qboolean,
        veh: *mut bgEntity_t,
    ) -> qboolean {
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let mut charging = qfalse;
            let mut altFire = qfalse;

            if vehicleRocketLock != qfalse {
                if (*pm).cmd.buttons & (BUTTON_ATTACK | BUTTON_ALT_ATTACK) != 0 {
                    //actually charging
                    if !veh.is_null() && !((*veh).m_pVehicle as *mut Vehicle_t).is_null() {
                        let pv = (*veh).m_pVehicle as *mut Vehicle_t;
                        let info = (*pv).m_pVehicleInfo;
                        let id0 = (*info).weapon[0].ID as usize;
                        let id1 = (*info).weapon[1].ID as usize;
                        if ((*pm).cmd.buttons & BUTTON_ATTACK != 0
                            && self.bg.g_vehWeaponInfo[id0].fHoming != 0.0
                            && (*ps).ammo[0] >= self.bg.g_vehWeaponInfo[id0].iAmmoPerShot)
                            || ((*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0
                                && self.bg.g_vehWeaponInfo[id1].fHoming != 0.0
                                && (*ps).ammo[1] >= self.bg.g_vehWeaponInfo[id1].iAmmoPerShot)
                        {
                            //pressing the appropriate fire button for the lock-on/charging weapon
                            self.PM_RocketLock(16384.0, qtrue);
                            charging = qtrue;
                        }
                        if (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                            altFire = qtrue;
                        }
                    }
                }
            } else {
                let w = (*ps).weapon;
                if w == WP_BRYAR_PISTOL as c_int {
                    if (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                        charging = qtrue;
                        altFire = qtrue;
                    }
                } else if w == WP_CONCUSSION as c_int {
                    if (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                        altFire = qtrue;
                    }
                } else if w == WP_BRYAR_OLD as c_int {
                    if (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                        charging = qtrue;
                        altFire = qtrue;
                    }
                } else if w == WP_BOWCASTER as c_int {
                    if (*pm).cmd.buttons & BUTTON_ATTACK != 0 {
                        charging = qtrue;
                    }
                } else if w == WP_ROCKET_LAUNCHER as c_int {
                    if (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0
                        && (*ps).ammo[weaponData[(*ps).weapon as usize].ammoIndex as usize]
                            >= weaponData[(*ps).weapon as usize].altEnergyPerShot
                    {
                        self.PM_RocketLock(2048.0, qfalse);
                        charging = qtrue;
                        altFire = qtrue;
                    }
                } else if w == WP_THERMAL as c_int {
                    if (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                        altFire = qtrue;
                        charging = qtrue;
                    } else if (*pm).cmd.buttons & BUTTON_ATTACK != 0 {
                        charging = qtrue;
                    }
                } else if w == WP_DEMP2 as c_int {
                    if (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                        altFire = qtrue;
                        charging = qtrue;
                    }
                } else if w == WP_DISRUPTOR as c_int {
                    if (*pm).cmd.buttons & BUTTON_ATTACK != 0
                        && (*ps).zoomMode == 1
                        && (*ps).zoomLocked != qfalse
                    {
                        if (*pm).cmd.forwardmove == 0
                            && (*pm).cmd.rightmove == 0
                            && (*pm).cmd.upmove <= 0
                        {
                            charging = qtrue;
                            altFire = qtrue;
                        } else {
                            charging = qfalse;
                            altFire = qfalse;
                        }
                    }

                    if (*ps).zoomMode != 1 && (*ps).weaponstate == WEAPON_CHARGING_ALT as c_int {
                        (*ps).weaponstate = WEAPON_READY as c_int;
                        charging = qfalse;
                        altFire = qfalse;
                    }
                }
            }

            // set up the appropriate weapon state based on the button that's down.
            if charging != qfalse {
                let short_circuit = 'chg: {
                    if altFire != qfalse {
                        if (*ps).weaponstate != WEAPON_CHARGING_ALT as c_int {
                            // charge isn't started, so do it now
                            (*ps).weaponstate = WEAPON_CHARGING_ALT as c_int;
                            (*ps).weaponChargeTime = (*pm).cmd.serverTime;
                            (*ps).weaponChargeSubtractTime = (*pm).cmd.serverTime
                                + weaponData[(*ps).weapon as usize].altChargeSubTime;
                            BG_AddPredictableEventToPlayerstate(
                                EV_WEAPON_CHARGE_ALT as c_int,
                                (*ps).weapon,
                                ps,
                            );
                        }

                        if vehicleRocketLock != qfalse {
                            if !veh.is_null() {
                                let pv = (*veh).m_pVehicle as *mut Vehicle_t;
                                let id1 = (*(*pv).m_pVehicleInfo).weapon[1].ID as usize;
                                if (*ps).ammo[1] < self.bg.g_vehWeaponInfo[id1].iAmmoPerShot {
                                    (*ps).weaponstate = WEAPON_CHARGING_ALT as c_int;
                                    break 'chg false;
                                }
                            }
                        } else if (*ps).ammo[weaponData[(*ps).weapon as usize].ammoIndex as usize]
                            < (weaponData[(*ps).weapon as usize].altChargeSub
                                + weaponData[(*ps).weapon as usize].altEnergyPerShot)
                        {
                            (*ps).weaponstate = WEAPON_CHARGING_ALT as c_int;
                            break 'chg false;
                        } else if ((*pm).cmd.serverTime - (*ps).weaponChargeTime)
                            < weaponData[(*ps).weapon as usize].altMaxCharge
                        {
                            if (*ps).weaponChargeSubtractTime < (*pm).cmd.serverTime {
                                (*ps).ammo[weaponData[(*ps).weapon as usize].ammoIndex as usize] -=
                                    weaponData[(*ps).weapon as usize].altChargeSub;
                                (*ps).weaponChargeSubtractTime = (*pm).cmd.serverTime
                                    + weaponData[(*ps).weapon as usize].altChargeSubTime;
                            }
                        }
                    } else {
                        if (*ps).weaponstate != WEAPON_CHARGING as c_int {
                            (*ps).weaponstate = WEAPON_CHARGING as c_int;
                            (*ps).weaponChargeTime = (*pm).cmd.serverTime;
                            (*ps).weaponChargeSubtractTime = (*pm).cmd.serverTime
                                + weaponData[(*ps).weapon as usize].chargeSubTime;
                            BG_AddPredictableEventToPlayerstate(
                                EV_WEAPON_CHARGE as c_int,
                                (*ps).weapon,
                                ps,
                            );
                        }

                        if vehicleRocketLock != qfalse {
                            if !veh.is_null() {
                                let pv = (*veh).m_pVehicle as *mut Vehicle_t;
                                let id0 = (*(*pv).m_pVehicleInfo).weapon[0].ID as usize;
                                if (*ps).ammo[0] < self.bg.g_vehWeaponInfo[id0].iAmmoPerShot {
                                    (*ps).weaponstate = WEAPON_CHARGING as c_int;
                                    break 'chg false;
                                }
                            }
                        } else if (*ps).ammo[weaponData[(*ps).weapon as usize].ammoIndex as usize]
                            < (weaponData[(*ps).weapon as usize].chargeSub
                                + weaponData[(*ps).weapon as usize].energyPerShot)
                        {
                            (*ps).weaponstate = WEAPON_CHARGING as c_int;
                            break 'chg false;
                        } else if ((*pm).cmd.serverTime - (*ps).weaponChargeTime)
                            < weaponData[(*ps).weapon as usize].maxCharge
                        {
                            if (*ps).weaponChargeSubtractTime < (*pm).cmd.serverTime {
                                (*ps).ammo[weaponData[(*ps).weapon as usize].ammoIndex as usize] -=
                                    weaponData[(*ps).weapon as usize].chargeSub;
                                (*ps).weaponChargeSubtractTime = (*pm).cmd.serverTime
                                    + weaponData[(*ps).weapon as usize].chargeSubTime;
                            }
                        }
                    }
                    true // did not goto rest
                };

                if short_circuit {
                    return qtrue; // short-circuit rest of weapon code
                }
            }
            // rest:
            if (*ps).weaponstate == WEAPON_CHARGING as c_int {
                // weapon has a charge, so let us do an attack
                (*pm).cmd.buttons |= BUTTON_ATTACK;
                (*ps).eFlags |= EF_FIRING;
            } else if (*ps).weaponstate == WEAPON_CHARGING_ALT as c_int {
                // weapon has a charge, so let us do an alt-attack
                (*pm).cmd.buttons |= BUTTON_ALT_ATTACK;
                (*ps).eFlags |= EF_FIRING | EF_ALT_FIRING;
            }

            qfalse
        }
    }

    /// Raven `PM_ItemUsable`.
    /// Source: `oracle/codemp/game/bg_pmove.c:6239-6366`
    pub fn PM_ItemUsable(&mut self, ps: *mut playerState_t, forcedUse: c_int) -> c_int {
        unsafe {
            let mut fwd: vec3_t = [0.0; 3];
            let mut fwdorg: vec3_t = [0.0; 3];
            let mut dest: vec3_t = [0.0; 3];
            let mut pos: vec3_t = [0.0; 3];
            let mut yawonly: vec3_t = [0.0; 3];
            let mut mins: vec3_t = [0.0; 3];
            let mut maxs: vec3_t = [0.0; 3];
            let mut trtest: vec3_t = [0.0; 3];
            let mut tr: trace_t = core::mem::zeroed();

            if (*ps).m_iVehicleNum != 0 {
                return 0;
            }

            if (*ps).pm_flags & PMF_USE_ITEM_HELD != 0 {
                //force to let go first
                return 0;
            }

            if (*ps).duelInProgress != 0 {
                //not allowed to use holdables while in a private duel.
                return 0;
            }

            let mut forcedUse = forcedUse;
            if forcedUse == 0 {
                forcedUse = selected_holdable_tag(ps);
            }

            if BG_IsItemSelectable(ps, forcedUse) == qfalse {
                return 0;
            }

            if forcedUse == HI_MEDPAC as c_int || forcedUse == HI_MEDPAC_BIG as c_int {
                if (*ps).stats[statIndex_t::STAT_HEALTH as usize]
                    >= (*ps).stats[statIndex_t::STAT_MAX_HEALTH as usize]
                {
                    return 0;
                }
                if (*ps).stats[statIndex_t::STAT_HEALTH as usize] <= 0
                    || (*ps).eFlags & EF_DEAD != 0
                {
                    return 0;
                }
                return 1;
            } else if forcedUse == HI_SEEKER as c_int {
                if (*ps).eFlags & EF_SEEKERDRONE != 0 {
                    self.PM_AddEventWithParm(
                        EV_ITEMUSEFAIL as c_int,
                        mp_qshared::shared::itemUseFail_t::SEEKER_ALREADYDEPLOYED as c_int,
                    );
                    return 0;
                }
                return 1;
            } else if forcedUse == HI_SENTRY_GUN as c_int {
                if (*ps).fd.sentryDeployed != 0 {
                    self.PM_AddEventWithParm(
                        EV_ITEMUSEFAIL as c_int,
                        mp_qshared::shared::itemUseFail_t::SENTRY_ALREADYPLACED as c_int,
                    );
                    return 0;
                }

                yawonly[ROLL] = 0.0;
                yawonly[PITCH] = 0.0;
                yawonly[YAW] = (*ps).viewangles[YAW];

                VectorSet(&mut mins, -8.0, -8.0, 0.0);
                VectorSet(&mut maxs, 8.0, 8.0, 24.0);

                AngleVectors(yawonly, Some(&mut fwd), None, None);

                fwdorg[0] = (*ps).origin[0] + fwd[0] * 64.0;
                fwdorg[1] = (*ps).origin[1] + fwd[1] * 64.0;
                fwdorg[2] = (*ps).origin[2] + fwd[2] * 64.0;

                trtest[0] = fwdorg[0] + fwd[0] * 16.0;
                trtest[1] = fwdorg[1] + fwd[1] * 16.0;
                trtest[2] = fwdorg[2] + fwd[2] * 16.0;

                self.traps.trace(
                    &mut tr,
                    core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                    core::ptr::addr_of!(mins) as *const vec3_t,
                    core::ptr::addr_of!(maxs) as *const vec3_t,
                    core::ptr::addr_of!(trtest) as *const vec3_t,
                    (*ps).clientNum,
                    MASK_PLAYERSOLID,
                );

                if (tr.fraction != 1.0 && tr.entityNum as c_int != (*ps).clientNum)
                    || tr.startsolid != 0
                    || tr.allsolid != 0
                {
                    self.PM_AddEventWithParm(
                        EV_ITEMUSEFAIL as c_int,
                        mp_qshared::shared::itemUseFail_t::SENTRY_NOROOM as c_int,
                    );
                    return 0;
                }
                return 1;
            } else if forcedUse == HI_SHIELD as c_int {
                mins[0] = -8.0;
                mins[1] = -8.0;
                mins[2] = 0.0;

                maxs[0] = 8.0;
                maxs[1] = 8.0;
                maxs[2] = 8.0;

                AngleVectors((*ps).viewangles, Some(&mut fwd), None, None);
                fwd[2] = 0.0;
                _VectorMA((*ps).origin, 64.0, fwd, &mut dest);
                self.traps.trace(
                    &mut tr,
                    core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                    core::ptr::addr_of!(mins) as *const vec3_t,
                    core::ptr::addr_of!(maxs) as *const vec3_t,
                    core::ptr::addr_of!(dest) as *const vec3_t,
                    (*ps).clientNum,
                    MASK_SHOT,
                );
                if tr.fraction > 0.9 && tr.startsolid == 0 && tr.allsolid == 0 {
                    _VectorCopy(tr.endpos, &mut pos);
                    VectorSet(&mut dest, pos[0], pos[1], pos[2] - 4096.0);
                    self.traps.trace(
                        &mut tr,
                        core::ptr::addr_of!(pos) as *const vec3_t,
                        core::ptr::addr_of!(mins) as *const vec3_t,
                        core::ptr::addr_of!(maxs) as *const vec3_t,
                        core::ptr::addr_of!(dest) as *const vec3_t,
                        (*ps).clientNum,
                        MASK_SOLID,
                    );
                    if tr.startsolid == 0 && tr.allsolid == 0 {
                        return 1;
                    }
                }
                self.PM_AddEventWithParm(
                    EV_ITEMUSEFAIL as c_int,
                    mp_qshared::shared::itemUseFail_t::SHIELD_NOROOM as c_int,
                );
                return 0;
            } else if forcedUse == HI_JETPACK as c_int {
                return 1;
            } else if forcedUse == HI_HEALTHDISP as c_int {
                return 1;
            } else if forcedUse == HI_AMMODISP as c_int {
                return 1;
            } else if forcedUse == HI_EWEB as c_int {
                return 1;
            } else if forcedUse == HI_CLOAK as c_int {
                return 1;
            } else {
                return 1;
            }
        }
    }

    /// Raven `PM_CanSetWeaponAnims`.
    /// Source: `oracle/codemp/game/bg_pmove.c:6369-6377`
    pub fn PM_CanSetWeaponAnims(&mut self) -> qboolean {
        unsafe {
            if (*(*self.pm).ps).m_iVehicleNum != 0 {
                return qfalse;
            }
            qtrue
        }
    }
}

impl PmoveContext<'_> {
    /// Raven `PM_VehicleWeaponAnimate`. The `backAgain` goto is a loop; the `VEH_FLYING`/
    /// `VEH_CRASHING` branches are `if (0)` (dead) in the oracle.
    /// Source: `oracle/codemp/game/bg_pmove.c:6381-6631`
    pub fn PM_VehicleWeaponAnimate(&mut self) {
        use animNumber_t::*;
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let veh = self.pm_entVeh;
            let mut iFlags = 0;
            let mut iBlend = 0;
            let mut Anim = -1;

            if veh.is_null() || ((*veh).m_pVehicle as *mut Vehicle_t).is_null() {
                return;
            }
            let pVeh = (*veh).m_pVehicle as *mut Vehicle_t;
            if (*pVeh).m_pPilot.is_null()
                || (*(*pVeh).m_pPilot).playerState.is_null()
                || (*ps).clientNum != (*(*(*pVeh).m_pPilot).playerState).clientNum
            {
                //make sure the vehicle exists, and its pilot is this player
                return;
            }

            if (*(*pVeh).m_pVehicleInfo).r#type as c_int == vehicleType_t::VH_WALKER as c_int
                || (*(*pVeh).m_pVehicleInfo).r#type as c_int == vehicleType_t::VH_FIGHTER as c_int
            {
                return;
            }
            'back: loop {
                // If they're firing, play the right fire animation.
                if (*pm).cmd.buttons & (BUTTON_ATTACK | BUTTON_ALT_ATTACK) != 0 {
                    iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;
                    iBlend = 200;

                    if (*ps).weapon == WP_SABER as c_int {
                        if (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                            //don't do anything.. I guess.
                            (*pm).cmd.buttons &= !BUTTON_ALT_ATTACK;
                            continue 'back;
                        }
                        // If we're already in an attack animation, leave.
                        if (*ps).torsoTimer <= 0 {
                            self.PM_AddEvent(EV_SABER_ATTACK as c_int);
                        }

                        (*ps).saberMove = LS_R_TL2BR as c_int;

                        if (*ps).torsoTimer > 0
                            && ((*ps).torsoAnim == BOTH_VS_ATR_S as c_int
                                || (*ps).torsoAnim == BOTH_VS_ATL_S as c_int)
                        {
                            return;
                        }

                        // Start the attack.
                        if (*pm).cmd.rightmove > 0 {
                            Anim = BOTH_VS_ATR_S as c_int;
                        } else if (*pm).cmd.rightmove < 0 {
                            Anim = BOTH_VS_ATL_S as c_int;
                        } else {
                            if self.PM_irand_timesync(0, 1) == 0 {
                                Anim = BOTH_VS_ATR_S as c_int;
                            } else {
                                Anim = BOTH_VS_ATL_S as c_int;
                            }
                        }

                        if (*ps).torsoTimer <= 0 {
                            iFlags |= SETANIM_FLAG_RESTART;
                        }
                    } else if (*ps).weapon == WP_BLASTER as c_int {
                        // Override the shoot anim.
                        if (*ps).torsoAnim == BOTH_ATTACK3 as c_int {
                            if (*pm).cmd.rightmove > 0 {
                                Anim = BOTH_VS_ATR_G as c_int;
                            } else if (*pm).cmd.rightmove < 0 {
                                Anim = BOTH_VS_ATL_G as c_int;
                            } else {
                                Anim = BOTH_VS_ATF_G as c_int;
                            }
                        }
                    } else {
                        Anim = BOTH_VS_IDLE as c_int;
                    }
                } else if !(*veh).playerState.is_null()
                    && (*(*veh).playerState).speed < 0.0
                    && (*(*pVeh).m_pVehicleInfo).r#type as c_int
                        == vehicleType_t::VH_ANIMAL as c_int
                {
                    //tauntaun is going backwards
                    Anim = BOTH_VT_WALK_REV as c_int;
                    iBlend = 600;
                } else if !(*veh).playerState.is_null()
                    && (*(*veh).playerState).speed < 0.0
                    && (*(*pVeh).m_pVehicleInfo).r#type as c_int
                        == vehicleType_t::VH_SPEEDER as c_int
                {
                    //speeder is going backwards
                    Anim = BOTH_VS_REV as c_int;
                    iBlend = 600;
                } else {
                    iFlags = SETANIM_FLAG_NORMAL;

                    if (*ps).weapon == WP_SABER as c_int {
                        if BG_SabersOff(ps) != qfalse {
                            Anim = BOTH_VS_IDLE as c_int;
                        } else if false {
                            iBlend = 800;
                            Anim = BOTH_VS_AIR_G as c_int;
                            iFlags = SETANIM_FLAG_OVERRIDE;
                        } else if false {
                            iBlend = 800;
                            Anim = BOTH_VS_LAND_SR as c_int;
                            iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;
                        } else {
                            Anim = BOTH_VS_IDLE_SR as c_int;
                        }
                    } else if (*ps).weapon == WP_BLASTER as c_int {
                        if false {
                            iBlend = 800;
                            Anim = BOTH_VS_AIR_G as c_int;
                            iFlags = SETANIM_FLAG_OVERRIDE;
                        } else if false {
                            iBlend = 800;
                            Anim = BOTH_VS_LAND_G as c_int;
                            iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;
                        } else {
                            Anim = BOTH_VS_IDLE_G as c_int;
                        }
                    } else {
                        Anim = BOTH_VS_IDLE as c_int;
                    }
                }
                break;
            }

            if Anim != -1 {
                //override it
                if (*(*pVeh).m_pVehicleInfo).r#type as c_int == vehicleType_t::VH_ANIMAL as c_int {
                    //remap anims for the tauntaun
                    if Anim == BOTH_VS_IDLE as c_int {
                        if !(*veh).playerState.is_null() && (*(*veh).playerState).speed > 0.0 {
                            if (*(*veh).playerState).speed > (*(*pVeh).m_pVehicleInfo).speedMax {
                                Anim = BOTH_VT_TURBO as c_int;
                            } else {
                                Anim = BOTH_VT_RUN_FWD as c_int;
                            }
                        } else {
                            Anim = BOTH_VT_IDLE as c_int;
                        }
                    } else if Anim == BOTH_VS_ATR_S as c_int {
                        Anim = BOTH_VT_ATR_S as c_int;
                    } else if Anim == BOTH_VS_ATL_S as c_int {
                        Anim = BOTH_VT_ATL_S as c_int;
                    } else if Anim == BOTH_VS_ATR_G as c_int {
                        Anim = BOTH_VT_ATR_G as c_int;
                    } else if Anim == BOTH_VS_ATL_G as c_int {
                        Anim = BOTH_VT_ATL_G as c_int;
                    } else if Anim == BOTH_VS_ATF_G as c_int {
                        Anim = BOTH_VT_ATF_G as c_int;
                    } else if Anim == BOTH_VS_IDLE_SL as c_int {
                        Anim = BOTH_VT_IDLE_S as c_int;
                    } else if Anim == BOTH_VS_IDLE_SR as c_int {
                        Anim = BOTH_VT_IDLE_S as c_int;
                    } else if Anim == BOTH_VS_IDLE_G as c_int {
                        Anim = BOTH_VT_IDLE_G as c_int;
                    } else if Anim == BOTH_VS_AIR_G as c_int
                        || Anim == BOTH_VS_LAND_SL as c_int
                        || Anim == BOTH_VS_LAND_SR as c_int
                        || Anim == BOTH_VS_LAND_G as c_int
                    {
                        //should not happen for tauntaun
                        return;
                    }
                }

                self.PM_SetAnim(SETANIM_BOTH, Anim, iFlags, iBlend);
            }
        }
    }
}

impl PmoveContext<'_> {
    /// Raven `PM_Weapon`. `QAGAME` is defined (npc/grapple/vehicle-fire branches compiled);
    /// `#if 0` melee-grab and dead alt-fire branches are dropped.
    /// Source: `oracle/codemp/game/bg_pmove.c:6641-7672`
    pub fn PM_Weapon(&mut self) {
        use animNumber_t::*;
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            let addTime;
            let mut amount;
            let mut killAfterItem = 0;
            let mut veh: *mut bgEntity_t = core::ptr::null_mut();
            let mut vehicleRocketLock = qfalse;

            // QAGAME: npc with no weapon
            if (*ps).clientNum >= MAX_CLIENTS as c_int
                && (*ps).weapon == WP_NONE as c_int
                && (*pm).cmd.weapon as c_int == WP_NONE as c_int
                && !self.pm_entSelf.is_null()
            {
                // S5-2: inuse/client/localAnimIndex are game-side reads, by number.
                if self
                    .callbacks
                    .humanoid_inuse_client((*self.pm_entSelf).s.number)
                    != 0
                {
                    //humanoid
                    (*ps).torsoAnim = (*ps).legsAnim;
                    (*ps).torsoTimer = (*ps).legsTimer;
                    return;
                }
            }

            if (*ps).emplacedIndex == 0 && (*ps).weapon == WP_EMPLACED_GUN as c_int {
                //oh no!
                let mut i = 0;
                let mut weap = -1;

                while i < WP_NUM_WEAPONS as c_int {
                    if (*ps).stats[statIndex_t::STAT_WEAPONS as usize] & (1 << i) != 0
                        && i != WP_NONE as c_int
                    {
                        weap = i;
                        break;
                    }
                    i += 1;
                }

                if weap != -1 {
                    (*pm).cmd.weapon = weap as u8;
                    (*ps).weapon = weap;
                    return;
                }
            }

            if (*self.pm_entSelf).s.NPC_class != CLASS_VEHICLE as c_int && (*ps).m_iVehicleNum != 0
            {
                //riding a vehicle
                veh = self.pm_entVeh;
                if !veh.is_null() && {
                    let pv = (*veh).m_pVehicle as *mut Vehicle_t;
                    (!((*veh).m_pVehicle as *mut Vehicle_t).is_null()
                        && (*(*pv).m_pVehicleInfo).r#type as c_int
                            == vehicleType_t::VH_WALKER as c_int)
                        || (!((*veh).m_pVehicle as *mut Vehicle_t).is_null()
                            && (*(*pv).m_pVehicleInfo).r#type as c_int
                                == vehicleType_t::VH_FIGHTER as c_int)
                } {
                    //riding a walker/fighter: keep saber off, do no weapon stuff at all!
                    (*ps).saberHolstered = 2;
                    // QAGAME
                    (*pm).cmd.buttons &= !(BUTTON_ATTACK | BUTTON_ALT_ATTACK);
                }
            }

            if (*ps).weapon != WP_DISRUPTOR as c_int
                && (*ps).weapon != WP_ROCKET_LAUNCHER as c_int
                && (*ps).weapon != WP_THERMAL as c_int
                && (*ps).m_iVehicleNum == 0
            {
                //check for exceeding max charge time
                if (*ps).weaponstate == WEAPON_CHARGING_ALT as c_int {
                    let timeDif = (*pm).cmd.serverTime - (*ps).weaponChargeTime;
                    if timeDif > MAX_WEAPON_CHARGE_TIME {
                        (*pm).cmd.buttons &= !BUTTON_ALT_ATTACK;
                    }
                }

                if (*ps).weaponstate == WEAPON_CHARGING as c_int {
                    let timeDif = (*pm).cmd.serverTime - (*ps).weaponChargeTime;
                    if timeDif > MAX_WEAPON_CHARGE_TIME {
                        (*pm).cmd.buttons &= !BUTTON_ATTACK;
                    }
                }
            }

            if (*ps).forceHandExtend == HANDEXTEND_WEAPONREADY as c_int
                && self.PM_CanSetWeaponAnims() != qfalse
            {
                //reset into weapon stance
                if (*ps).weapon != WP_SABER as c_int
                    && (*ps).weapon != WP_MELEE as c_int
                    && PM_IsRocketTrooper() == qfalse
                {
                    if (*ps).weapon == WP_DISRUPTOR as c_int && (*ps).zoomMode == 1 {
                        self.PM_StartTorsoAnim(TORSO_RAISEWEAP1 as c_int);
                    } else {
                        if (*ps).weapon == WP_EMPLACED_GUN as c_int {
                            self.PM_StartTorsoAnim(BOTH_GUNSIT1 as c_int);
                        } else {
                            self.PM_StartTorsoAnim(TORSO_RAISEWEAP1 as c_int);
                        }
                    }
                }

                (*ps).weaponstate = WEAPON_RAISING as c_int;
                (*ps).weaponTime += 250;

                (*ps).forceHandExtend = HANDEXTEND_NONE as c_int;
            } else if (*ps).forceHandExtend != HANDEXTEND_NONE as c_int {
                //nothing else should be allowed to happen during this time
                let mut desiredAnim = 0;
                let mut seperateOnTorso = qfalse;
                let mut playFullBody = qfalse;
                let mut desiredOnTorso = 0;

                let fhe = (*ps).forceHandExtend;
                if fhe == HANDEXTEND_FORCEPUSH as c_int {
                    desiredAnim = BOTH_FORCEPUSH as c_int;
                } else if fhe == HANDEXTEND_FORCEPULL as c_int {
                    desiredAnim = BOTH_FORCEPULL as c_int;
                } else if fhe == HANDEXTEND_FORCE_HOLD as c_int {
                    if (*ps).fd.forcePowersActive & (1 << FP_GRIP) != 0 {
                        desiredAnim = BOTH_FORCEGRIP_HOLD as c_int;
                    } else if (*ps).fd.forcePowersActive & (1 << FP_LIGHTNING) != 0 {
                        if (*ps).weapon == WP_MELEE as c_int
                            && (*ps).activeForcePass > FORCE_LEVEL_2
                        {
                            desiredAnim = BOTH_FORCE_2HANDEDLIGHTNING_HOLD as c_int;
                        } else {
                            desiredAnim = BOTH_FORCELIGHTNING_HOLD as c_int;
                        }
                    } else if (*ps).fd.forcePowersActive & (1 << FP_DRAIN) != 0 {
                        desiredAnim = BOTH_FORCEGRIP_HOLD as c_int;
                    } else {
                        desiredAnim = BOTH_FORCEGRIP_HOLD as c_int;
                    }
                } else if fhe == HANDEXTEND_SABERPULL as c_int {
                    desiredAnim = BOTH_SABERPULL as c_int;
                } else if fhe == HANDEXTEND_CHOKE as c_int {
                    desiredAnim = BOTH_CHOKE3 as c_int;
                } else if fhe == HANDEXTEND_DODGE as c_int {
                    desiredAnim = (*ps).forceDodgeAnim as c_int;
                } else if fhe == HANDEXTEND_KNOCKDOWN as c_int {
                    if (*ps).forceDodgeAnim != 0 {
                        if (*ps).forceDodgeAnim > 4 {
                            let originalDAnim = (*ps).forceDodgeAnim as c_int - 8;
                            if originalDAnim == 2 {
                                desiredAnim = BOTH_FORCE_GETUP_B1 as c_int;
                            } else if originalDAnim == 3 {
                                desiredAnim = BOTH_FORCE_GETUP_B3 as c_int;
                            } else {
                                desiredAnim = BOTH_GETUP1 as c_int;
                            }
                            seperateOnTorso = qtrue;
                            desiredOnTorso = BOTH_FORCEPUSH as c_int;
                        } else if (*ps).forceDodgeAnim == 2 {
                            desiredAnim = BOTH_FORCE_GETUP_B1 as c_int;
                        } else if (*ps).forceDodgeAnim == 3 {
                            desiredAnim = BOTH_FORCE_GETUP_B3 as c_int;
                        } else {
                            desiredAnim = BOTH_GETUP1 as c_int;
                        }
                    } else {
                        desiredAnim = BOTH_KNOCKDOWN1 as c_int;
                    }
                } else if fhe == HANDEXTEND_DUELCHALLENGE as c_int {
                    desiredAnim = BOTH_ENGAGETAUNT as c_int;
                } else if fhe == HANDEXTEND_TAUNT as c_int {
                    desiredAnim = (*ps).forceDodgeAnim as c_int;
                    if desiredAnim != BOTH_ENGAGETAUNT as c_int
                        && VectorCompare((*ps).velocity, vec3_origin)
                        && (*ps).groundEntityNum != ENTITYNUM_NONE
                    {
                        playFullBody = qtrue;
                    }
                } else if fhe == HANDEXTEND_PRETHROW as c_int {
                    desiredAnim = BOTH_A3_TL_BR as c_int;
                    playFullBody = qtrue;
                } else if fhe == HANDEXTEND_POSTTHROW as c_int {
                    desiredAnim = BOTH_D3_TL___ as c_int;
                    playFullBody = qtrue;
                } else if fhe == HANDEXTEND_PRETHROWN as c_int {
                    desiredAnim = BOTH_KNEES1 as c_int;
                    playFullBody = qtrue;
                } else if fhe == HANDEXTEND_POSTTHROWN as c_int {
                    if (*ps).forceDodgeAnim != 0 {
                        desiredAnim = BOTH_FORCE_GETUP_F2 as c_int;
                    } else {
                        desiredAnim = BOTH_KNOCKDOWN5 as c_int;
                    }
                    playFullBody = qtrue;
                } else if fhe == HANDEXTEND_DRAGGING as c_int {
                    desiredAnim = BOTH_B1_BL___ as c_int;
                } else if fhe == HANDEXTEND_JEDITAUNT as c_int {
                    desiredAnim = BOTH_GESTURE1 as c_int;
                } else {
                    desiredAnim = BOTH_FORCEPUSH as c_int;
                }

                if seperateOnTorso == qfalse {
                    self.PM_SetAnim(
                        SETANIM_TORSO,
                        desiredAnim,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        100,
                    );
                    (*ps).torsoTimer = 1;
                }

                if playFullBody != qfalse {
                    self.PM_SetAnim(
                        SETANIM_BOTH,
                        desiredAnim,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        100,
                    );
                    (*ps).legsTimer = 1;
                    (*ps).torsoTimer = 1;
                } else if (*ps).forceHandExtend == HANDEXTEND_DODGE as c_int
                    || (*ps).forceHandExtend == HANDEXTEND_KNOCKDOWN as c_int
                    || ((*ps).forceHandExtend == HANDEXTEND_CHOKE as c_int
                        && (*ps).groundEntityNum == ENTITYNUM_NONE)
                {
                    if seperateOnTorso != qfalse {
                        self.PM_SetAnim(
                            SETANIM_LEGS,
                            desiredAnim,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            100,
                        );
                        (*ps).legsTimer = 1;

                        self.PM_SetAnim(
                            SETANIM_TORSO,
                            desiredOnTorso,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            100,
                        );
                        (*ps).torsoTimer = 1;
                    } else {
                        self.PM_SetAnim(
                            SETANIM_LEGS,
                            desiredAnim,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            100,
                        );
                        (*ps).legsTimer = 1;
                    }
                }

                return;
            }

            if BG_InSpecialJump((*ps).legsAnim) != qfalse
                || BG_InRoll(ps, (*ps).legsAnim) != qfalse
                || PM_InRollComplete(ps, (*ps).legsAnim) != qfalse
            {
                if (*ps).weaponTime < (*ps).legsTimer {
                    (*ps).weaponTime = (*ps).legsTimer;
                }
            }

            if (*ps).duelInProgress != 0 {
                (*pm).cmd.weapon = WP_SABER as u8;
                (*ps).weapon = WP_SABER as c_int;

                if (*ps).duelTime >= (*pm).cmd.serverTime {
                    (*pm).cmd.upmove = 0;
                    (*pm).cmd.forwardmove = 0;
                    (*pm).cmd.rightmove = 0;
                }
            }

            if (*ps).weapon == WP_SABER as c_int
                && (*ps).saberMove != LS_READY as c_int
                && (*ps).saberMove != LS_NONE as c_int
            {
                (*pm).cmd.weapon = WP_SABER as u8; //don't allow switching out mid-attack
            }

            if (*ps).weapon == WP_SABER as c_int {
                self.PM_WeaponLightsaber();
                killAfterItem = 1;
            } else if (*ps).weapon != WP_EMPLACED_GUN as c_int {
                (*ps).saberHolstered = 0;
            }

            if self.PM_CanSetWeaponAnims() != qfalse {
                if (*ps).weapon == WP_THERMAL as c_int
                    || (*ps).weapon == WP_TRIP_MINE as c_int
                    || (*ps).weapon == WP_DET_PACK as c_int
                {
                    if (*ps).weapon == WP_THERMAL as c_int {
                        if (*ps).torsoAnim == WeaponAttackAnim[(*ps).weapon as usize]
                            && ((*ps).weaponTime - 200) <= 0
                        {
                            self.PM_StartTorsoAnim(WeaponReadyAnim[(*ps).weapon as usize]);
                        }
                    } else {
                        if (*ps).torsoAnim == WeaponAttackAnim[(*ps).weapon as usize]
                            && ((*ps).weaponTime - 700) <= 0
                        {
                            self.PM_StartTorsoAnim(WeaponReadyAnim[(*ps).weapon as usize]);
                        }
                    }
                }
            }

            // don't allow attack until all buttons are up
            if (*ps).pm_flags & PMF_RESPAWNED != 0 {
                return;
            }

            // ignore if spectator
            if (*ps).clientNum < MAX_CLIENTS as c_int
                && (*ps).persistant[PERS_TEAM as usize] == TEAM_SPECTATOR as c_int
            {
                return;
            }

            // check for dead player
            if (*ps).stats[statIndex_t::STAT_HEALTH as usize] <= 0 {
                (*ps).weapon = WP_NONE as c_int;
                return;
            }

            // check for item using
            if (*pm).cmd.buttons & BUTTON_USE_HOLDABLE != 0 {
                if (*ps).pm_flags & PMF_USE_ITEM_HELD == 0 {
                    if (*self.pm_entSelf).s.NPC_class != CLASS_VEHICLE as c_int
                        && (*ps).m_iVehicleNum != 0
                    {
                        return;
                    }

                    if (*ps).stats[statIndex_t::STAT_HOLDABLE_ITEM as usize] == 0 {
                        return;
                    }

                    if self.PM_ItemUsable(ps, 0) == 0 {
                        (*ps).pm_flags |= PMF_USE_ITEM_HELD;
                        return;
                    } else {
                        let giTag = selected_holdable_tag(ps);
                        if (*ps).stats[statIndex_t::STAT_HOLDABLE_ITEMS as usize] & (1 << giTag)
                            != 0
                        {
                            if giTag != HI_BINOCULARS as c_int
                                && giTag != HI_JETPACK as c_int
                                && giTag != HI_HEALTHDISP as c_int
                                && giTag != HI_AMMODISP as c_int
                                && giTag != HI_CLOAK as c_int
                                && giTag != HI_EWEB as c_int
                            {
                                (*ps).stats[statIndex_t::STAT_HOLDABLE_ITEMS as usize] -=
                                    1 << giTag;
                            }
                        } else {
                            return; //this should not happen...
                        }

                        (*ps).pm_flags |= PMF_USE_ITEM_HELD;
                        self.PM_AddEvent(EV_USE_ITEM0 as c_int + giTag);

                        if giTag != HI_BINOCULARS as c_int
                            && giTag != HI_JETPACK as c_int
                            && giTag != HI_HEALTHDISP as c_int
                            && giTag != HI_AMMODISP as c_int
                            && giTag != HI_CLOAK as c_int
                            && giTag != HI_EWEB as c_int
                        {
                            (*ps).stats[statIndex_t::STAT_HOLDABLE_ITEM as usize] = 0;
                            BG_CycleInven(ps, 1);
                        }
                    }
                    return;
                }
            } else {
                (*ps).pm_flags &= !PMF_USE_ITEM_HELD;
            }

            if killAfterItem != 0 {
                return;
            }

            // make weapon function
            if (*ps).weaponTime > 0 {
                (*ps).weaponTime -= self.pml.msec;
            }

            if (*ps).isJediMaster != 0 && (*ps).emplacedIndex != 0 {
                (*ps).emplacedIndex = 0;
                (*ps).saberHolstered = 0;
            }

            if (*ps).duelInProgress != 0 && (*ps).emplacedIndex != 0 {
                (*ps).emplacedIndex = 0;
                (*ps).saberHolstered = 0;
            }

            if (*ps).weapon == WP_EMPLACED_GUN as c_int && (*ps).emplacedIndex != 0 {
                (*pm).cmd.weapon = WP_EMPLACED_GUN as u8;
                self.PM_StartTorsoAnim(BOTH_GUNSIT1 as c_int);
            }

            if (*ps).isJediMaster != 0 || (*ps).duelInProgress != 0 || (*ps).trueJedi != 0 {
                (*pm).cmd.weapon = WP_SABER as u8;
                (*ps).weapon = WP_SABER as c_int;

                if (*ps).isJediMaster != 0 || (*ps).trueJedi != 0 {
                    (*ps).stats[statIndex_t::STAT_WEAPONS as usize] = 1 << WP_SABER as c_int;
                }
            }

            amount = weaponData[(*ps).weapon as usize].energyPerShot;

            // take an ammo away if not infinite
            if (*ps).weapon != WP_NONE as c_int
                && (*ps).weapon == (*pm).cmd.weapon as c_int
                && ((*ps).weaponTime <= 0 || (*ps).weaponstate != WEAPON_FIRING as c_int)
            {
                if (*ps).clientNum < MAX_CLIENTS as c_int
                    && (*ps).ammo[weaponData[(*ps).weapon as usize].ammoIndex as usize] != -1
                {
                    if (*ps).ammo[weaponData[(*ps).weapon as usize].ammoIndex as usize]
                        < weaponData[(*ps).weapon as usize].energyPerShot
                        && (*ps).ammo[weaponData[(*ps).weapon as usize].ammoIndex as usize]
                            < weaponData[(*ps).weapon as usize].altEnergyPerShot
                    {
                        self.PM_AddEventWithParm(
                            EV_NOAMMO as c_int,
                            WP_NUM_WEAPONS as c_int + (*ps).weapon,
                        );

                        if (*ps).weaponTime < 500 {
                            (*ps).weaponTime += 500;
                        }
                        return;
                    }

                    if (*ps).weapon == WP_DET_PACK as c_int
                        && (*ps).hasDetPackPlanted == 0
                        && (*ps).ammo[weaponData[(*ps).weapon as usize].ammoIndex as usize] < 1
                    {
                        self.PM_AddEventWithParm(
                            EV_NOAMMO as c_int,
                            WP_NUM_WEAPONS as c_int + (*ps).weapon,
                        );

                        if (*ps).weaponTime < 500 {
                            (*ps).weaponTime += 500;
                        }
                        return;
                    }
                }
            }

            // check for weapon change
            if (*ps).weaponTime <= 0 || (*ps).weaponstate != WEAPON_FIRING as c_int {
                if (*ps).weapon != (*pm).cmd.weapon as c_int {
                    self.PM_BeginWeaponChange((*pm).cmd.weapon as c_int);
                }
            }

            if (*ps).weaponTime > 0 {
                return;
            }

            if (*ps).weapon == WP_DISRUPTOR as c_int && (*ps).zoomMode == 1 {
                if self.pm_cancelOutZoom != qfalse {
                    (*ps).zoomMode = 0;
                    (*ps).zoomFov = 0.0;
                    (*ps).zoomLocked = qfalse;
                    (*ps).zoomLockTime = 0;
                    self.PM_AddEvent(EV_DISRUPTOR_ZOOMSOUND as c_int);
                    return;
                }

                if (*pm).cmd.forwardmove != 0 || (*pm).cmd.rightmove != 0 || (*pm).cmd.upmove > 0 {
                    return;
                }
            }

            // change weapon if time
            if (*ps).weaponstate == WEAPON_DROPPING as c_int {
                self.PM_FinishWeaponChange();
                return;
            }

            if (*ps).weaponstate == WEAPON_RAISING as c_int {
                (*ps).weaponstate = WEAPON_READY as c_int;
                if self.PM_CanSetWeaponAnims() != qfalse {
                    if (*ps).weapon == WP_SABER as c_int {
                        let st = self.PM_GetSaberStance();
                        self.PM_StartTorsoAnim(st);
                    } else if (*ps).weapon == WP_MELEE as c_int || PM_IsRocketTrooper() != qfalse {
                        self.PM_StartTorsoAnim((*ps).legsAnim);
                    } else {
                        if (*ps).weapon == WP_DISRUPTOR as c_int && (*ps).zoomMode == 1 {
                            self.PM_StartTorsoAnim(TORSO_WEAPONREADY4 as c_int);
                        } else {
                            if (*ps).weapon == WP_EMPLACED_GUN as c_int {
                                self.PM_StartTorsoAnim(BOTH_GUNSIT1 as c_int);
                            } else {
                                self.PM_StartTorsoAnim(WeaponReadyAnim[(*ps).weapon as usize]);
                            }
                        }
                    }
                }
                return;
            }

            if self.PM_CanSetWeaponAnims() != qfalse
                && PM_IsRocketTrooper() == qfalse
                && (*ps).weaponstate == WEAPON_READY as c_int
                && (*ps).weaponTime <= 0
                && ((*ps).weapon >= WP_BRYAR_PISTOL as c_int
                    || (*ps).weapon == WP_STUN_BATON as c_int)
                && (*ps).torsoTimer <= 0
                && (*ps).torsoAnim != WeaponReadyAnim[(*ps).weapon as usize]
                && (*ps).torsoAnim != TORSO_WEAPONIDLE3 as c_int
                && (*ps).weapon != WP_EMPLACED_GUN as c_int
            {
                self.PM_StartTorsoAnim(WeaponReadyAnim[(*ps).weapon as usize]);
            } else if self.PM_CanSetWeaponAnims() != qfalse && (*ps).weapon == WP_MELEE as c_int {
                if (*ps).weaponTime <= 0 && (*ps).forceHandExtend == HANDEXTEND_NONE as c_int {
                    let mut desTAnim = (*ps).legsAnim;

                    if desTAnim == BOTH_STAND1 as c_int || desTAnim == BOTH_STAND2 as c_int {
                        desTAnim = BOTH_STAND6 as c_int;
                    }

                    if (*pm).cmd.buttons & (BUTTON_ATTACK | BUTTON_ALT_ATTACK) == 0 {
                        if (*ps).torsoAnim != desTAnim {
                            self.PM_StartTorsoAnim(desTAnim);
                        }
                    }
                }
            } else if self.PM_CanSetWeaponAnims() != qfalse && PM_IsRocketTrooper() != qfalse {
                let desTAnim = (*ps).legsAnim;

                if (*pm).cmd.buttons & (BUTTON_ATTACK | BUTTON_ALT_ATTACK) == 0 {
                    if (*ps).torsoAnim != desTAnim {
                        self.PM_StartTorsoAnim(desTAnim);
                    }
                }
            }

            if ((*ps).torsoAnim == TORSO_WEAPONREADY4 as c_int
                || (*ps).torsoAnim == BOTH_ATTACK4 as c_int)
                && ((*ps).weapon != WP_DISRUPTOR as c_int || (*ps).zoomMode != 1)
            {
                if (*ps).weapon == WP_EMPLACED_GUN as c_int {
                    self.PM_StartTorsoAnim(BOTH_GUNSIT1 as c_int);
                } else if self.PM_CanSetWeaponAnims() != qfalse {
                    self.PM_StartTorsoAnim(WeaponReadyAnim[(*ps).weapon as usize]);
                }
            } else if (*ps).torsoAnim != TORSO_WEAPONREADY4 as c_int
                && (*ps).torsoAnim != BOTH_ATTACK4 as c_int
                && self.PM_CanSetWeaponAnims() != qfalse
                && ((*ps).weapon == WP_DISRUPTOR as c_int && (*ps).zoomMode == 1)
            {
                self.PM_StartTorsoAnim(TORSO_WEAPONREADY4 as c_int);
            }

            if (*ps).clientNum >= MAX_CLIENTS as c_int
                && !self.pm_entSelf.is_null()
                && (*self.pm_entSelf).s.NPC_class == CLASS_VEHICLE as c_int
            {
                //we are a vehicle
                veh = self.pm_entSelf;
            }
            if !veh.is_null() && !((*veh).m_pVehicle as *mut Vehicle_t).is_null() {
                let pv = (*veh).m_pVehicle as *mut Vehicle_t;
                let id0 = (*(*pv).m_pVehicleInfo).weapon[0].ID as usize;
                let id1 = (*(*pv).m_pVehicleInfo).weapon[1].ID as usize;
                if self.bg.g_vehWeaponInfo[id0].fHoming != 0.0
                    || self.bg.g_vehWeaponInfo[id1].fHoming != 0.0
                {
                    vehicleRocketLock = qtrue;
                }
            }

            if vehicleRocketLock == qfalse {
                if (*ps).weapon != WP_ROCKET_LAUNCHER as c_int {
                    if (*self.pm_entSelf).s.NPC_class != CLASS_VEHICLE as c_int
                        && (*ps).m_iVehicleNum != 0
                    {
                        //riding a vehicle, the vehicle will tell me my rocketlock stuff...
                    } else {
                        (*ps).rocketLockIndex = ENTITYNUM_NONE;
                        (*ps).rocketLockTime = 0.0;
                        (*ps).rocketTargetTime = 0.0;
                    }
                }
            }

            if self.PM_DoChargedWeapons(vehicleRocketLock, veh) != qfalse {
                return;
            }

            // check for fire
            if (*pm).cmd.buttons & (BUTTON_ATTACK | BUTTON_ALT_ATTACK) == 0 {
                (*ps).weaponTime = 0;
                (*ps).weaponstate = WEAPON_READY as c_int;
                return;
            }

            if (*ps).weapon == WP_EMPLACED_GUN as c_int {
                addTime = weaponData[(*ps).weapon as usize].fireTime;
                (*ps).weaponTime += addTime;
                if (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                    self.PM_AddEvent(EV_ALT_FIRE as c_int);
                } else {
                    self.PM_AddEvent(EV_FIRE_WEAPON as c_int);
                }
                return;
            } else if (*ps).m_iVehicleNum != 0
                && (*self.pm_entSelf).s.NPC_class == CLASS_VEHICLE as c_int
            {
                //a vehicle NPC that has a pilot
                (*ps).weaponstate = WEAPON_FIRING as c_int;
                (*ps).weaponTime += 100;
                // QAGAME
                if (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                    self.callbacks
                        .cheap_weapon_fire((*ps).clientNum, EV_ALT_FIRE as c_int);
                } else {
                    self.callbacks
                        .cheap_weapon_fire((*ps).clientNum, EV_FIRE_WEAPON as c_int);
                }
                return;
            }

            if (*ps).weapon == WP_DISRUPTOR as c_int
                && (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0
                && (*ps).zoomLocked == qfalse
            {
                return;
            }

            if (*ps).weapon == WP_DISRUPTOR as c_int
                && (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0
                && (*ps).zoomMode == 2
            {
                return;
            }

            if (*ps).weapon == WP_DISRUPTOR as c_int && (*ps).zoomMode == 1 {
                self.PM_StartTorsoAnim(BOTH_ATTACK4 as c_int);
            } else if (*ps).weapon == WP_MELEE as c_int {
                //special anims for standard melee attacks
                if (*ps).m_iVehicleNum == 0 {
                    if (*pm).debugMelee != 0
                        && (*pm).cmd.buttons & BUTTON_ATTACK != 0
                        && (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0
                    {
                        //ok, grapple time (QAGAME)
                        if !self.pm_entSelf.is_null() {
                            if self.callbacks.try_grapple((*self.pm_entSelf).s.number) != qfalse {
                                return;
                            }
                        }
                    } else if (*pm).debugMelee != 0 && (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                        //kicks
                        if BG_KickingAnim((*ps).torsoAnim) == qfalse
                            && BG_KickingAnim((*ps).legsAnim) == qfalse
                        {
                            let mut kickMove = self.PM_KickMoveForConditions();
                            if kickMove == LS_HILT_BASH as c_int {
                                kickMove = LS_KICK_F as c_int;
                            }

                            if kickMove != -1 {
                                if (*ps).groundEntityNum == ENTITYNUM_NONE {
                                    //if in air, convert kick to an in-air kick
                                    let gDist = self.PM_GroundDistance();
                                    if (BG_FlippingAnim((*ps).legsAnim) == qfalse
                                        || (*ps).legsTimer <= 0)
                                        && gDist > 64.0
                                        && gDist > (-(*ps).velocity[2]) - 64.0
                                    {
                                        if kickMove == LS_KICK_F as c_int {
                                            kickMove = LS_KICK_F_AIR as c_int;
                                        } else if kickMove == LS_KICK_B as c_int {
                                            kickMove = LS_KICK_B_AIR as c_int;
                                        } else if kickMove == LS_KICK_R as c_int {
                                            kickMove = LS_KICK_R_AIR as c_int;
                                        } else if kickMove == LS_KICK_L as c_int {
                                            kickMove = LS_KICK_L_AIR as c_int;
                                        } else {
                                            kickMove = -1;
                                        }
                                    } else {
                                        //off ground, but too close to ground
                                        kickMove = -1;
                                    }
                                }
                            }

                            if kickMove != -1 {
                                let kickAnim = self.bg.saberMoveData[kickMove as usize].animToUse;

                                if kickAnim != -1 {
                                    self.PM_SetAnim(
                                        SETANIM_BOTH,
                                        kickAnim,
                                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                                        0,
                                    );
                                    if (*ps).legsAnim == kickAnim {
                                        (*ps).weaponTime = (*ps).legsTimer;
                                        return;
                                    }
                                }
                            }
                        }

                        //if got here then no move to do so put torso into leg idle or whatever
                        if (*ps).torsoAnim != (*ps).legsAnim {
                            self.PM_SetAnim(
                                SETANIM_BOTH,
                                (*ps).legsAnim,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                                0,
                            );
                        }
                        (*ps).weaponTime = 0;
                        return;
                    } else {
                        //just punch
                        let mut desTAnim = BOTH_MELEE1 as c_int;
                        if (*ps).torsoAnim == BOTH_MELEE1 as c_int {
                            desTAnim = BOTH_MELEE2 as c_int;
                        }
                        self.PM_StartTorsoAnim(desTAnim);

                        if (*ps).torsoAnim == desTAnim {
                            (*ps).weaponTime = (*ps).torsoTimer;
                        }
                    }
                }
            } else {
                self.PM_StartTorsoAnim(WeaponAttackAnim[(*ps).weapon as usize]);
            }

            if (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                amount = weaponData[(*ps).weapon as usize].altEnergyPerShot;
            } else {
                amount = weaponData[(*ps).weapon as usize].energyPerShot;
            }

            (*ps).weaponstate = WEAPON_FIRING as c_int;

            // take an ammo away if not infinite
            if (*ps).clientNum < MAX_CLIENTS as c_int
                && (*ps).ammo[weaponData[(*ps).weapon as usize].ammoIndex as usize] != -1
            {
                if ((*ps).ammo[weaponData[(*ps).weapon as usize].ammoIndex as usize] - amount) >= 0
                {
                    (*ps).ammo[weaponData[(*ps).weapon as usize].ammoIndex as usize] -= amount;
                } else {
                    // Not enough energy: Switch weapons
                    if (*ps).weapon != WP_DET_PACK as c_int || (*ps).hasDetPackPlanted == 0 {
                        self.PM_AddEventWithParm(
                            EV_NOAMMO as c_int,
                            WP_NUM_WEAPONS as c_int + (*ps).weapon,
                        );
                        if (*ps).weaponTime < 500 {
                            (*ps).weaponTime += 500;
                        }
                    }
                    return;
                }
            }

            let mut addTime = if (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                if (*ps).weapon == WP_DISRUPTOR as c_int && (*ps).zoomMode != 1 {
                    self.PM_AddEvent(EV_FIRE_WEAPON as c_int);
                    weaponData[(*ps).weapon as usize].fireTime
                } else {
                    if (*ps).weapon != WP_MELEE as c_int || (*ps).m_iVehicleNum == 0 {
                        self.PM_AddEvent(EV_ALT_FIRE as c_int);
                    }
                    weaponData[(*ps).weapon as usize].altFireTime
                }
            } else {
                if (*ps).weapon != WP_MELEE as c_int || (*ps).m_iVehicleNum == 0 {
                    self.PM_AddEvent(EV_FIRE_WEAPON as c_int);
                }
                let mut at = weaponData[(*ps).weapon as usize].fireTime;
                if (*pm).gametype == GT_SIEGE as c_int && (*ps).weapon == WP_DET_PACK as c_int {
                    at *= 2;
                }
                at
            };

            if (*ps).fd.forcePowersActive & (1 << FP_RAGE) != 0 {
                addTime = (addTime as f64 * 0.75) as c_int;
            } else if (*ps).fd.forceRageRecoveryTime > (*pm).cmd.serverTime {
                addTime = (addTime as f64 * 1.5) as c_int;
            }

            (*ps).weaponTime += addTime;
            let _ = amount;
        }
    }
}

impl PmoveContext<'_> {
    /// Raven `PM_Animate`. The `#if 0` TA gesture-button block is dropped.
    /// Source: `oracle/codemp/game/bg_pmove.c:7680-7740`
    pub fn PM_Animate(&mut self) {
        use animNumber_t::*;
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;
            if (*pm).cmd.buttons & BUTTON_GESTURE != 0 {
                if (*ps).m_iVehicleNum != 0 {
                    //eh, fine, clear it
                    if (*ps).forceHandExtendTime < (*pm).cmd.serverTime {
                        (*ps).forceHandExtend = HANDEXTEND_NONE as c_int;
                    }
                }

                if (*ps).torsoTimer < 1
                    && (*ps).forceHandExtend == HANDEXTEND_NONE as c_int
                    && (*ps).legsTimer < 1
                    && (*ps).weaponTime < 1
                    && (*ps).saberLockTime < (*pm).cmd.serverTime
                {
                    (*ps).forceHandExtend = HANDEXTEND_TAUNT as c_int;

                    (*ps).forceDodgeAnim = BOTH_ENGAGETAUNT as _;

                    (*ps).forceHandExtendTime = (*pm).cmd.serverTime + 1000;

                    self.PM_AddEvent(EV_TAUNT as c_int);
                }
            }
        }
    }

    /// Raven `PM_DropTimers`.
    /// Source: `oracle/codemp/game/bg_pmove.c:7748-7773`
    pub fn PM_DropTimers(&mut self) {
        unsafe {
            let ps = (*self.pm).ps;
            // drop misc timing counter
            if (*ps).pm_time != 0 {
                if self.pml.msec >= (*ps).pm_time {
                    (*ps).pm_flags &= !PMF_ALL_TIMES;
                    (*ps).pm_time = 0;
                } else {
                    (*ps).pm_time -= self.pml.msec;
                }
            }

            // drop animation counter
            if (*ps).legsTimer > 0 {
                (*ps).legsTimer -= self.pml.msec;
                if (*ps).legsTimer < 0 {
                    (*ps).legsTimer = 0;
                }
            }

            if (*ps).torsoTimer > 0 {
                (*ps).torsoTimer -= self.pml.msec;
                if (*ps).torsoTimer < 0 {
                    (*ps).torsoTimer = 0;
                }
            }
        }
    }
}

/// Raven `BG_UnrestrainedPitchRoll`.
/// Source: `oracle/codemp/game/bg_pmove.c:7784-7798`
// ESCALATED (worker W4): Raven reads the game-tier `bg_fighterAltControl` cvar
// (a cached `vmCvar_t` registered in `g_main.c`) as `.integer`. This bg-tier free
// fn only has `bg: &BgState`; the mirror ladder resolves to a BgState field:
//  * no cvar-mirror field exists on `BgState` yet;
//  * threading it as a param fails — the `PmoveContext` callers
//    (`PM_UpdateViewAngles` @7079, `PM_UpdateViewAngles`-vehicle @8992) have no
//    cvar/world handle (`PmoveContext` holds only pm/pml/bg/traps/callbacks),
//    only the `ctx.world` callers (FighterNPC.rs, g_vehicles.rs) do.
// RECOMMENDED (needs bg_channel owner): add `pub bg_fighterAltControl: c_int` to
// `BgState`, written from the game-tier cvar-update init (where `g_main.c`
// registers/refreshes the cvar), and read it here as `bg.bg_fighterAltControl`.
// Source: oracle/codemp/game/bg_pmove.c:7783-7798, g_main.c:177,420
pub fn BG_UnrestrainedPitchRoll(
    ps: *mut playerState_t,
    pVeh: *mut Vehicle_t,
    bg: &BgState,
) -> qboolean {
    unsafe {
        if bg.bg_fighterAltControl != 0
            && (*ps).clientNum < MAX_CLIENTS as c_int //real client
            && (*ps).m_iVehicleNum != 0 //in a vehicle
            && !pVeh.is_null() //valid vehicle data pointer
            && !(*pVeh).m_pVehicleInfo.is_null() //valid vehicle info
            && (*(*pVeh).m_pVehicleInfo).r#type as c_int == vehicleType_t::VH_FIGHTER as c_int
        //fighter
        {
            //can roll and pitch without limitation!
            return qtrue;
        }
        qfalse
    }
}

/// Raven `PM_UpdateViewAngles` — circularly clamp view angles with deltas.
///
/// `VEH_CONTROL_SCHEME_4` is undefined, so the `#else` branch (the
/// `BG_UnrestrainedPitchRoll` fighter test whose body is dead code, otherwise
/// the ±16000 short pitch clamp) is the compiled one.
/// Source: `oracle/codemp/game/bg_pmove.c:7813-7894`
impl PmoveContext<'_> {
    pub fn PM_UpdateViewAngles(&mut self, ps: *mut playerState_t, cmd: *const usercmd_t) {
        unsafe {
            if (*ps).pm_type == PM_INTERMISSION as c_int
                || (*ps).pm_type == PM_SPINTERMISSION as c_int
            {
                return; // no view changes at all
            }

            if (*ps).pm_type != PM_SPECTATOR as c_int
                && (*ps).stats[statIndex_t::STAT_HEALTH as usize] <= 0
            {
                return; // no view changes at all
            }

            // circularly clamp the angles with deltas
            for i in 0..3usize {
                let mut temp: i16 = ((*cmd).angles[i] + (*ps).delta_angles[i]) as i16;
                if !self.pm_entVeh.is_null()
                    && BG_UnrestrainedPitchRoll(
                        ps,
                        (*self.pm_entVeh).m_pVehicle as *mut Vehicle_t,
                        self.bg,
                    ) == qtrue
                {
                    // in a fighter (Raven's ROLL passthrough here is commented out — nothing)
                } else {
                    if i == PITCH as usize {
                        // don't let the player look up or down more than 90 degrees
                        if temp > 16000 {
                            (*ps).delta_angles[i] = 16000 - (*cmd).angles[i];
                            temp = 16000;
                        } else if temp < -16000 {
                            (*ps).delta_angles[i] = -16000 - (*cmd).angles[i];
                            temp = -16000;
                        }
                    }
                }
                // SHORT2ANGLE(temp) == temp * (360.0/65536)
                (*ps).viewangles[i] = (temp as f64 * (360.0 / 65536.0)) as f32;
            }
        }
    }
}

/// Raven `PM_AdjustAttackStates` — set the firing eFlags + disruptor zoom state
/// from the current buttons/ammo.
/// Source: `oracle/codemp/game/bg_pmove.c:8031-8199`
impl PmoveContext<'_> {
    pub fn PM_AdjustAttackStates(&mut self, pm: *mut pmove_t) {
        unsafe {
            let mut amount: c_int;

            if (*self.pm_entSelf).s.NPC_class != CLASS_VEHICLE as c_int
                && (*(*pm).ps).m_iVehicleNum != 0
            {
                // riding a vehicle
                let veh = self.pm_entVeh;
                if !veh.is_null() && {
                    let pv = (*veh).m_pVehicle as *mut Vehicle_t;
                    !pv.is_null()
                        && ((*(*pv).m_pVehicleInfo).r#type as c_int
                            == vehicleType_t::VH_WALKER as c_int
                            || (*(*pv).m_pVehicleInfo).r#type as c_int
                                == vehicleType_t::VH_FIGHTER as c_int)
                } {
                    // riding a walker/fighter — not firing, ever
                    (*(*pm).ps).eFlags &= !(EF_FIRING | EF_ALT_FIRING);
                    return;
                }
            }

            // get ammo usage
            if (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                amount = (*(*pm).ps).ammo
                    [weaponData[(*(*pm).ps).weapon as usize].ammoIndex as usize]
                    - weaponData[(*(*pm).ps).weapon as usize].altEnergyPerShot;
            } else {
                amount = (*(*pm).ps).ammo
                    [weaponData[(*(*pm).ps).weapon as usize].ammoIndex as usize]
                    - weaponData[(*(*pm).ps).weapon as usize].energyPerShot;
            }

            // disruptor alt-fire should toggle the zoom mode
            if (*(*pm).ps).weapon == WP_DISRUPTOR as c_int
                && (*(*pm).ps).weaponstate == WEAPON_READY as c_int
            {
                if (*(*pm).ps).eFlags & EF_ALT_FIRING == 0
                    && (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0
                {
                    // We just pressed the alt-fire key
                    if (*(*pm).ps).zoomMode == 0 && (*(*pm).ps).pm_type != PM_DEAD as c_int {
                        // not already zooming, so do it now
                        (*(*pm).ps).zoomMode = 1;
                        (*(*pm).ps).zoomLocked = qfalse;
                        (*(*pm).ps).zoomFov = 80.0; //cg_fov.value;
                        (*(*pm).ps).zoomLockTime = (*pm).cmd.serverTime + 50;
                        self.PM_AddEvent(EV_DISRUPTOR_ZOOMSOUND as c_int);
                    } else if (*(*pm).ps).zoomMode == 1
                        && (*(*pm).ps).zoomLockTime < (*pm).cmd.serverTime
                    {
                        // check for == 1 so we can't turn binoculars off with disruptor alt fire
                        // already zooming, so must be wanting to turn it off
                        (*(*pm).ps).zoomMode = 0;
                        (*(*pm).ps).zoomTime = (*(*pm).ps).commandTime;
                        (*(*pm).ps).zoomLocked = qfalse;
                        self.PM_AddEvent(EV_DISRUPTOR_ZOOMSOUND as c_int);
                        (*(*pm).ps).weaponTime = 1000;
                    }
                } else if (*pm).cmd.buttons & BUTTON_ALT_ATTACK == 0
                    && (*(*pm).ps).zoomLockTime < (*pm).cmd.serverTime
                {
                    // Not pressing zoom any more
                    if (*(*pm).ps).zoomMode != 0 {
                        if (*(*pm).ps).zoomMode == 1 && (*(*pm).ps).zoomLocked == qfalse {
                            // approximate what level the client should be zoomed at based on how long zoom was held
                            (*(*pm).ps).zoomFov =
                                (((*pm).cmd.serverTime + 50) - (*(*pm).ps).zoomLockTime) as f32
                                    * 0.035;
                            if (*(*pm).ps).zoomFov > 50.0 {
                                (*(*pm).ps).zoomFov = 50.0;
                            }
                            if (*(*pm).ps).zoomFov < 1.0 {
                                (*(*pm).ps).zoomFov = 1.0;
                            }
                        }
                        // were zooming in, so now lock the zoom
                        (*(*pm).ps).zoomLocked = qtrue;
                    }
                }

                if (*pm).cmd.buttons & BUTTON_ATTACK != 0 {
                    // If we are zoomed, switch the ammo usage to the alt-fire
                    if (*(*pm).ps).zoomMode != 0 {
                        amount = (*(*pm).ps).ammo
                            [weaponData[(*(*pm).ps).weapon as usize].ammoIndex as usize]
                            - weaponData[(*(*pm).ps).weapon as usize].altEnergyPerShot;
                    }
                } else {
                    // alt-fire button pressing doesn't use any ammo
                    amount = 0;
                }
            }

            // set the firing flag for continuous beam weapons, saber fires even if out of ammo
            if (*(*pm).ps).pm_flags & PMF_RESPAWNED == 0
                && (*(*pm).ps).pm_type != PM_INTERMISSION as c_int
                && (*pm).cmd.buttons & (BUTTON_ATTACK | BUTTON_ALT_ATTACK) != 0
                && (amount >= 0 || (*(*pm).ps).weapon == WP_SABER as c_int)
            {
                if (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                    (*(*pm).ps).eFlags |= EF_ALT_FIRING;
                } else {
                    (*(*pm).ps).eFlags &= !EF_ALT_FIRING;
                }

                // This flag should always get set, even when alt-firing
                (*(*pm).ps).eFlags |= EF_FIRING;
            } else {
                // Clear 'em out
                (*(*pm).ps).eFlags &= !(EF_FIRING | EF_ALT_FIRING);
            }

            // disruptor should convert a main fire to an alt-fire if currently zoomed
            if (*(*pm).ps).weapon == WP_DISRUPTOR as c_int {
                if (*pm).cmd.buttons & BUTTON_ATTACK != 0
                    && (*(*pm).ps).zoomMode == 1
                    && (*(*pm).ps).zoomLocked == qtrue
                {
                    // converting the main fire to an alt-fire
                    (*pm).cmd.buttons |= BUTTON_ALT_ATTACK;
                    (*(*pm).ps).eFlags |= EF_ALT_FIRING;
                } else if (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0
                    && (*(*pm).ps).zoomMode == 1
                    && (*(*pm).ps).zoomLocked == qtrue
                {
                    (*pm).cmd.buttons &= !BUTTON_ALT_ATTACK;
                    (*(*pm).ps).eFlags &= !EF_ALT_FIRING;
                }
            }
        }
    }
}

/// Raven `BG_CmdForRoll`.
///
/// Source: `oracle/codemp/game/bg_pmove.c:8201-8327`
pub fn BG_CmdForRoll(
    ps: *mut playerState_t,
    anim: c_int,
    pCmd: *mut usercmd_t,
    bg: &crate::bg_channel::BgState,
) {
    use crate::bg_panimate::BG_AnimLength;
    use animNumber_t::*;
    unsafe {
        if anim == BOTH_ROLL_F as c_int {
            (*pCmd).forwardmove = 127;
            (*pCmd).rightmove = 0;
        } else if anim == BOTH_ROLL_B as c_int {
            (*pCmd).forwardmove = -127;
            (*pCmd).rightmove = 0;
        } else if anim == BOTH_ROLL_R as c_int {
            (*pCmd).forwardmove = 0;
            (*pCmd).rightmove = 127;
        } else if anim == BOTH_ROLL_L as c_int {
            (*pCmd).forwardmove = 0;
            (*pCmd).rightmove = -127;
        } else if anim == BOTH_GETUP_BROLL_R as c_int {
            (*pCmd).forwardmove = 0;
            (*pCmd).rightmove = 48;
            //NOTE: speed is 400
        } else if anim == BOTH_GETUP_FROLL_R as c_int {
            if (*ps).legsTimer <= 250 {
                //end of anim
                (*pCmd).forwardmove = 0;
                (*pCmd).rightmove = 0;
            } else {
                (*pCmd).forwardmove = 0;
                (*pCmd).rightmove = 48;
            }
        } else if anim == BOTH_GETUP_BROLL_L as c_int {
            (*pCmd).forwardmove = 0;
            (*pCmd).rightmove = -48;
        } else if anim == BOTH_GETUP_FROLL_L as c_int {
            if (*ps).legsTimer <= 250 {
                (*pCmd).forwardmove = 0;
                (*pCmd).rightmove = 0;
            } else {
                (*pCmd).forwardmove = 0;
                (*pCmd).rightmove = -48;
            }
        } else if anim == BOTH_GETUP_BROLL_B as c_int {
            if (*ps).torsoTimer <= 250 {
                //end of anim
                (*pCmd).forwardmove = 0;
                (*pCmd).rightmove = 0;
            } else if BG_AnimLength(bg, 0, (*ps).legsAnim) - (*ps).torsoTimer < 350 {
                //beginning of anim
                (*pCmd).forwardmove = 0;
                (*pCmd).rightmove = 0;
            } else {
                (*pCmd).forwardmove = -64;
                (*pCmd).rightmove = 0;
            }
        } else if anim == BOTH_GETUP_FROLL_B as c_int {
            if (*ps).torsoTimer <= 100 {
                (*pCmd).forwardmove = 0;
                (*pCmd).rightmove = 0;
            } else if BG_AnimLength(bg, 0, (*ps).legsAnim) - (*ps).torsoTimer < 200 {
                (*pCmd).forwardmove = 0;
                (*pCmd).rightmove = 0;
            } else {
                (*pCmd).forwardmove = -64;
                (*pCmd).rightmove = 0;
            }
        } else if anim == BOTH_GETUP_BROLL_F as c_int {
            if (*ps).torsoTimer <= 550 {
                (*pCmd).forwardmove = 0;
                (*pCmd).rightmove = 0;
            } else if BG_AnimLength(bg, 0, (*ps).legsAnim) - (*ps).torsoTimer < 150 {
                (*pCmd).forwardmove = 0;
                (*pCmd).rightmove = 0;
            } else {
                (*pCmd).forwardmove = 64;
                (*pCmd).rightmove = 0;
            }
        } else if anim == BOTH_GETUP_FROLL_F as c_int {
            if (*ps).torsoTimer <= 100 {
                (*pCmd).forwardmove = 0;
                (*pCmd).rightmove = 0;
            } else {
                (*pCmd).forwardmove = 64;
                (*pCmd).rightmove = 0;
            }
        }
        (*pCmd).upmove = 0;
    }
}

/// Raven `BG_AdjustClientSpeed` — reset `ps->speed` to base and apply the force
/// power / saber-move / roll / grip modifiers.
///
/// Note: Raven's `float *= <double literal>` promotes to double then narrows;
/// the `f64` casts below preserve that rounding. `<float>f` literals compute in
/// f32.
/// Source: `oracle/codemp/game/bg_pmove.c:8331-8510`
impl PmoveContext<'_> {
    pub fn BG_AdjustClientSpeed(
        &mut self,
        ps: *mut playerState_t,
        cmd: *mut usercmd_t,
        svTime: c_int,
    ) {
        unsafe {
            if (*ps).clientNum >= MAX_CLIENTS as c_int {
                let bgEnt = self.pm_entSelf;
                if !bgEnt.is_null() && (*bgEnt).s.NPC_class == CLASS_VEHICLE as c_int {
                    // vehicles manage their own speed
                    return;
                }
            }

            // For prediction, always reset speed back to the last known server base speed
            (*ps).speed = (*ps).basespeed as f32;

            if (*ps).forceHandExtend == HANDEXTEND_DODGE as c_int {
                (*ps).speed = 0.0;
            }

            if (*ps).forceHandExtend == HANDEXTEND_KNOCKDOWN as c_int
                || (*ps).forceHandExtend == HANDEXTEND_PRETHROWN as c_int
                || (*ps).forceHandExtend == HANDEXTEND_POSTTHROWN as c_int
            {
                (*ps).speed = 0.0;
            }

            if (*cmd).forwardmove < 0
                && (*cmd).buttons & BUTTON_WALKING == 0
                && (*(*self.pm).ps).groundEntityNum != ENTITYNUM_NONE
            {
                // running backwards is slower than running forwards (like SP)
                (*ps).speed = ((*ps).speed as f64 * 0.75) as f32;
            }

            if (*ps).fd.forcePowersActive & (1 << FP_GRIP) != 0 {
                (*ps).speed = ((*ps).speed as f64 * 0.4) as f32;
            }

            if (*ps).fd.forcePowersActive & (1 << FP_SPEED) != 0 {
                (*ps).speed = ((*ps).speed as f64 * 1.7) as f32;
            } else if (*ps).fd.forcePowersActive & (1 << FP_RAGE) != 0 {
                (*ps).speed = ((*ps).speed as f64 * 1.3) as f32;
            } else if (*ps).fd.forceRageRecoveryTime > svTime {
                (*ps).speed = ((*ps).speed as f64 * 0.75) as f32;
            }

            if (*(*self.pm).ps).weapon == WP_DISRUPTOR as c_int
                && (*(*self.pm).ps).zoomMode == 1
                && (*(*self.pm).ps).zoomLockTime < (*self.pm).cmd.serverTime
            {
                (*ps).speed *= 0.5;
            }

            if (*ps).fd.forceGripCripple != 0 {
                if (*ps).fd.forcePowersActive & (1 << FP_RAGE) != 0 {
                    (*ps).speed = ((*ps).speed as f64 * 0.9) as f32;
                } else if (*ps).fd.forcePowersActive & (1 << FP_SPEED) != 0 {
                    // force speed will help us escape
                    (*ps).speed = ((*ps).speed as f64 * 0.8) as f32;
                } else {
                    (*ps).speed = ((*ps).speed as f64 * 0.2) as f32;
                }
            }

            if BG_SaberInAttack((*ps).saberMove) == qtrue && (*cmd).forwardmove < 0 {
                // if running backwards while attacking, don't run as fast.
                let lvl = (*ps).fd.saberAnimLevel;
                if lvl == FORCE_LEVEL_1 {
                    (*ps).speed *= 0.75;
                } else if lvl == FORCE_LEVEL_2
                    || lvl == saber_styles_t::SS_DUAL as c_int
                    || lvl == saber_styles_t::SS_STAFF as c_int
                {
                    (*ps).speed *= 0.60;
                } else if lvl == FORCE_LEVEL_3 {
                    (*ps).speed *= 0.45;
                }
            } else if BG_SpinningSaberAnim((*ps).legsAnim) == qtrue {
                if (*ps).fd.saberAnimLevel == FORCE_LEVEL_3 {
                    (*ps).speed *= 0.3;
                } else {
                    (*ps).speed *= 0.5;
                }
            } else if (*ps).weapon == WP_SABER as c_int
                && BG_SaberInAttack((*ps).saberMove) == qtrue
            {
                // if attacking with saber while running, drop your speed
                let lvl = (*ps).fd.saberAnimLevel;
                if lvl == FORCE_LEVEL_2
                    || lvl == saber_styles_t::SS_DUAL as c_int
                    || lvl == saber_styles_t::SS_STAFF as c_int
                {
                    (*ps).speed *= 0.85;
                } else if lvl == FORCE_LEVEL_3 {
                    (*ps).speed *= 0.55;
                }
            } else if (*ps).weapon == WP_SABER as c_int
                && (*ps).fd.saberAnimLevel == FORCE_LEVEL_3
                && PM_SaberInTransition((*ps).saberMove) == qtrue
            {
                // slow down in transitions for level 3 (it has chains now)
                if (*cmd).forwardmove < 0 {
                    (*ps).speed *= 0.4;
                } else {
                    (*ps).speed *= 0.6;
                }
            }

            if BG_InRoll(ps, (*ps).legsAnim) == qtrue && (*ps).speed > 50.0 {
                // can't roll unless you're able to move normally
                if (*ps).legsAnim == BOTH_ROLL_B as c_int {
                    // backwards roll is pretty fast, should also be slower
                    if (*ps).legsTimer > 800 {
                        (*ps).speed = ((*ps).legsTimer as f64 / 2.5) as f32;
                    } else {
                        (*ps).speed = ((*ps).legsTimer as f64 / 6.0) as f32; //450;
                    }
                } else {
                    if (*ps).legsTimer > 800 {
                        (*ps).speed = ((*ps).legsTimer as f64 / 1.5) as f32; //450;
                    } else {
                        (*ps).speed = ((*ps).legsTimer as f64 / 5.0) as f32; //450;
                    }
                }
                if (*ps).speed > 600.0 {
                    (*ps).speed = 600.0;
                }
                // Automatically slow down as the roll ends.
            }

            let mut saber = self.callbacks.my_saber((*ps).clientNum, 0);
            if !saber.is_null() && (*saber).moveSpeedScale != 1.0 {
                (*ps).speed *= (*saber).moveSpeedScale;
            }
            saber = self.callbacks.my_saber((*ps).clientNum, 1);
            if !saber.is_null() && (*saber).moveSpeedScale != 1.0 {
                (*ps).speed *= (*saber).moveSpeedScale;
            }
        }
    }
}

/// Raven `BG_InRollAnim`.
///
/// Source: `oracle/codemp/game/bg_pmove.c:8512-8523`
pub fn BG_InRollAnim(cent: *mut entityState_t) -> qboolean {
    use animNumber_t::*;
    unsafe {
        let a = (*cent).legsAnim;
        if a == BOTH_ROLL_F as c_int
            || a == BOTH_ROLL_B as c_int
            || a == BOTH_ROLL_R as c_int
            || a == BOTH_ROLL_L as c_int
        {
            return qtrue;
        }
        qfalse
    }
}

/// Raven `BG_InKnockDown`.
///
/// Source: `oracle/codemp/game/bg_pmove.c:8525-8560`
pub fn BG_InKnockDown(anim: c_int) -> qboolean {
    use animNumber_t::*;
    if anim == BOTH_KNOCKDOWN1 as c_int
        || anim == BOTH_KNOCKDOWN2 as c_int
        || anim == BOTH_KNOCKDOWN3 as c_int
        || anim == BOTH_KNOCKDOWN4 as c_int
        || anim == BOTH_KNOCKDOWN5 as c_int
    {
        return qtrue;
    }
    if anim == BOTH_GETUP1 as c_int
        || anim == BOTH_GETUP2 as c_int
        || anim == BOTH_GETUP3 as c_int
        || anim == BOTH_GETUP4 as c_int
        || anim == BOTH_GETUP5 as c_int
        || anim == BOTH_FORCE_GETUP_F1 as c_int
        || anim == BOTH_FORCE_GETUP_F2 as c_int
        || anim == BOTH_FORCE_GETUP_B1 as c_int
        || anim == BOTH_FORCE_GETUP_B2 as c_int
        || anim == BOTH_FORCE_GETUP_B3 as c_int
        || anim == BOTH_FORCE_GETUP_B4 as c_int
        || anim == BOTH_FORCE_GETUP_B5 as c_int
        || anim == BOTH_GETUP_BROLL_B as c_int
        || anim == BOTH_GETUP_BROLL_F as c_int
        || anim == BOTH_GETUP_BROLL_L as c_int
        || anim == BOTH_GETUP_BROLL_R as c_int
        || anim == BOTH_GETUP_FROLL_B as c_int
        || anim == BOTH_GETUP_FROLL_F as c_int
        || anim == BOTH_GETUP_FROLL_L as c_int
        || anim == BOTH_GETUP_FROLL_R as c_int
    {
        return qtrue;
    }
    qfalse
}

/// Raven `BG_InRollES`.
///
/// Source: `oracle/codemp/game/bg_pmove.c:8562-8574`
pub fn BG_InRollES(ps: *mut entityState_t, anim: c_int) -> qboolean {
    use animNumber_t::*;
    // Raven's `ps` param is unreferenced; the switch keys off `anim`.
    let _ = ps;
    if anim == BOTH_ROLL_F as c_int
        || anim == BOTH_ROLL_B as c_int
        || anim == BOTH_ROLL_R as c_int
        || anim == BOTH_ROLL_L as c_int
    {
        return qtrue;
    }
    qfalse
}

/// Raven `BG_IK_MoveArm` — drive the left-arm inverse-kinematics bone chain
/// toward `desiredPos` (used to fling people in throws). `bgHumanoidAnimations`
/// is threaded via `bg: &BgState`.
/// Source: `oracle/codemp/game/bg_pmove.c:8576-8730`
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
    bg: &BgState,
    traps: &dyn BgTraps,
) {
    unsafe {
        let mut lHandMatrix: mdxaBone_t = core::mem::zeroed();
        let mut lHand: vec3_t = [0.0; 3];
        let mut torg: vec3_t = [0.0; 3];
        let distToDest: f32;

        if ghoul2.is_null() {
            return;
        }

        debug_assert!(bg.bgHumanoidAnimations[basePose as usize].firstFrame > 0);

        if *ikInProgress == qfalse && forceHalt == qfalse {
            let baseposeAnim = basePose;
            let mut ikP: sharedSetBoneIKStateParams_t = core::mem::zeroed();

            // leaving the shoulder unrestricted, but restricting the elbow joint.
            VectorSet(&mut ikP.pcj_mins, 0.0, 0.0, 0.0);
            VectorSet(&mut ikP.pcj_maxs, 0.0, 0.0, 0.0);

            // give the info on our entity.
            ikP.blend_time = blendTime;
            _VectorCopy(origin, &mut ikP.origin);
            _VectorCopy(angles, &mut ikP.angles);
            ikP.angles[PITCH] = 0.0;
            ikP.pcj_overrides = 0;
            ikP.radius = 10.0;
            _VectorCopy(scale, &mut ikP.scale);

            // base pose frames for the limb
            ikP.start_frame = bg.bgHumanoidAnimations[baseposeAnim as usize].firstFrame as c_int
                + bg.bgHumanoidAnimations[baseposeAnim as usize].numFrames as c_int;
            ikP.end_frame = bg.bgHumanoidAnimations[baseposeAnim as usize].firstFrame as c_int
                + bg.bgHumanoidAnimations[baseposeAnim as usize].numFrames as c_int;

            ikP.force_anim_on_bone = qfalse; // let it use existing anim if same as this one

            // call with a null bone name first to init the ik system on the g2 instance
            if traps.g2api_set_bone_ik_state(ghoul2, time, None, IKS_DYNAMIC as c_int, &mut ikP)
                == qfalse
            {
                debug_assert!(false, "Failed to init IK system for g2 instance!");
            }

            // Now create our IK bone state.
            if traps.g2api_set_bone_ik_state(
                ghoul2,
                time,
                Some("lhumerus"),
                IKS_DYNAMIC as c_int,
                &mut ikP,
            ) != qfalse
            {
                // restrict the elbow joint
                VectorSet(&mut ikP.pcj_mins, -90.0, -20.0, -20.0);
                VectorSet(&mut ikP.pcj_maxs, 30.0, 20.0, -20.0);

                if traps.g2api_set_bone_ik_state(
                    ghoul2,
                    time,
                    Some("lradius"),
                    IKS_DYNAMIC as c_int,
                    &mut ikP,
                ) != qfalse
                {
                    // everything went alright.
                    *ikInProgress = qtrue;
                }
            }
        }

        if *ikInProgress != qfalse && forceHalt == qfalse {
            // actively update our ik state.
            let mut ikM: sharedIKMoveParams_t = core::mem::zeroed();
            let mut tuParms: sharedRagDollUpdateParams_t = core::mem::zeroed();
            let mut tAngles: vec3_t = [0.0; 3];

            // set the argument struct up
            _VectorCopy(desiredPos, &mut ikM.desired_origin); // move the bone here if possible

            _VectorCopy(angles, &mut tAngles);
            tAngles[PITCH] = 0.0;
            tAngles[ROLL] = 0.0;

            traps.g2api_get_bolt_matrix(
                ghoul2,
                0,
                lHandBolt,
                &mut lHandMatrix,
                &tAngles,
                &origin,
                time,
                core::ptr::null_mut(),
                &scale,
            );
            // Get the point position from the matrix.
            lHand[0] = lHandMatrix.matrix[0][3];
            lHand[1] = lHandMatrix.matrix[1][3];
            lHand[2] = lHandMatrix.matrix[2][3];

            _VectorSubtract(lHand, desiredPos, &mut torg);
            // distToDest = VectorLength(torg);
            // sqrt is the double libm call rounded back to float; an f32 sqrt
            // double-rounds and diverges from the oracle.
            distToDest = VectorLength(torg);

            // closer we are, more we want to keep updated.
            if distToDest < 2.0 {
                ikM.movement_speed = 0.4;
            } else if distToDest < 16.0 {
                ikM.movement_speed = 0.9;
            } else if distToDest < 32.0 {
                ikM.movement_speed = 0.8;
            } else if distToDest < 64.0 {
                ikM.movement_speed = 0.7;
            } else {
                ikM.movement_speed = 0.6;
            }
            _VectorCopy(origin, &mut ikM.origin); // our position in the world.

            ikM.bone_name[0] = 0;
            if traps.g2api_ik_move(ghoul2, time, &mut ikM) != qfalse {
                // now do the standard model animate stuff with ragdoll update params.
                _VectorCopy(angles, &mut tuParms.angles);
                tuParms.angles[PITCH] = 0.0;

                _VectorCopy(origin, &mut tuParms.position);
                _VectorCopy(scale, &mut tuParms.scale);

                tuParms.me = (*ent).number;
                VectorClear(&mut tuParms.velocity);

                traps.g2api_animate_g2_models(ghoul2, time, &mut tuParms);
            } else {
                *ikInProgress = qfalse;
            }
        } else if *ikInProgress != qfalse {
            // kill it
            let mut cFrame: f32 = 0.0;
            let mut animSpeed: f32 = 0.0;
            let mut sFrame: c_int = 0;
            let mut eFrame: c_int = 0;
            let mut flags: c_int = 0;

            traps.g2api_set_bone_ik_state(
                ghoul2,
                time,
                Some("lhumerus"),
                IKS_NONE as c_int,
                core::ptr::null_mut(),
            );
            traps.g2api_set_bone_ik_state(
                ghoul2,
                time,
                Some("lradius"),
                IKS_NONE as c_int,
                core::ptr::null_mut(),
            );

            // then reset the angles/anims on these PCJs
            traps.g2api_set_bone_angles(
                ghoul2,
                0,
                "lhumerus",
                &vec3_origin,
                BONE_ANGLES_POSTMULT,
                POSITIVE_X as c_int,
                NEGATIVE_Y as c_int,
                NEGATIVE_Z as c_int,
                core::ptr::null_mut(),
                0,
                time,
            );
            traps.g2api_set_bone_angles(
                ghoul2,
                0,
                "lradius",
                &vec3_origin,
                BONE_ANGLES_POSTMULT,
                POSITIVE_X as c_int,
                NEGATIVE_Y as c_int,
                NEGATIVE_Z as c_int,
                core::ptr::null_mut(),
                0,
                time,
            );

            // Match the left arm back up with the pelvis anim/frames again.
            traps.g2api_get_bone_anim(
                ghoul2,
                "pelvis",
                time,
                &mut cFrame,
                &mut sFrame,
                &mut eFrame,
                &mut flags,
                &mut animSpeed,
                core::ptr::null_mut(),
                0,
            );
            traps.g2api_set_bone_anim(
                ghoul2,
                0,
                "lhumerus",
                sFrame,
                eFrame,
                flags,
                animSpeed,
                time,
                sFrame as f32,
                300,
            );
            traps.g2api_set_bone_anim(
                ghoul2,
                0,
                "lradius",
                sFrame,
                eFrame,
                flags,
                animSpeed,
                time,
                sFrame as f32,
                300,
            );

            // finally, get rid of all the ik state effector data (null bone name).
            traps.g2api_set_bone_ik_state(
                ghoul2,
                time,
                None,
                IKS_NONE as c_int,
                core::ptr::null_mut(),
            );

            *ikInProgress = qfalse;
        }
    }
}

/// Raven `BG_UpdateLookAngles`.
///
/// Source: `oracle/codemp/game/bg_pmove.c:8733-8787`
// `lastHeadAngles`/`lookAngles` are written in place → `&mut vec3_t`
// (never NULL in the oracle callers).
pub fn BG_UpdateLookAngles(
    lookingDebounceTime: c_int,
    lastHeadAngles: &mut vec3_t,
    time: c_int,
    lookAngles: &mut vec3_t,
    lookSpeed: f32,
    minPitch: f32,
    maxPitch: f32,
    minYaw: f32,
    maxYaw: f32,
    minRoll: f32,
    maxRoll: f32,
) {
    let fFrameInter: f32 = 0.1;
    // Raven's function-scope `static` scratch (oldLookAngles/lookAnglesDiff/ang)
    // are single-call temporaries → plain locals.
    let mut oldLookAngles: vec3_t = [0.0; 3];
    let mut lookAnglesDiff: vec3_t = [0.0; 3];

    if lookingDebounceTime > time {
        //clamp so don't get "Exorcist" effect
        if lookAngles[PITCH] > maxPitch {
            lookAngles[PITCH] = maxPitch;
        } else if lookAngles[PITCH] < minPitch {
            lookAngles[PITCH] = minPitch;
        }
        if lookAngles[YAW] > maxYaw {
            lookAngles[YAW] = maxYaw;
        } else if lookAngles[YAW] < minYaw {
            lookAngles[YAW] = minYaw;
        }
        if lookAngles[ROLL] > maxRoll {
            lookAngles[ROLL] = maxRoll;
        } else if lookAngles[ROLL] < minRoll {
            lookAngles[ROLL] = minRoll;
        }

        //slowly lerp to this new value; remember last headAngles
        oldLookAngles = *lastHeadAngles;
        for i in 0..3 {
            lookAnglesDiff[i] = lookAngles[i] - oldLookAngles[i];
        }

        for ang in 0..3 {
            lookAnglesDiff[ang] = AngleNormalize180(lookAnglesDiff[ang]);
        }

        if VectorLengthSquared(lookAnglesDiff) != 0.0 {
            lookAngles[PITCH] = AngleNormalize180(
                oldLookAngles[PITCH] + (lookAnglesDiff[PITCH] * fFrameInter * lookSpeed),
            );
            lookAngles[YAW] = AngleNormalize180(
                oldLookAngles[YAW] + (lookAnglesDiff[YAW] * fFrameInter * lookSpeed),
            );
            lookAngles[ROLL] = AngleNormalize180(
                oldLookAngles[ROLL] + (lookAnglesDiff[ROLL] * fFrameInter * lookSpeed),
            );
        }
    }
    //Remember current lookAngles next time
    *lastHeadAngles = *lookAngles;
}

/// Raven `BG_G2ClientNeckAngles`.
///
/// Source: `oracle/codemp/game/bg_pmove.c:8790-8866`
// `headAngles`/`neckAngles`/`thoracicAngles` are written in place → `&mut vec3_t`;
// `lookAngles`/`headClampMin/MaxAngles` are read-only → keep by-value `vec3_t`.
pub fn BG_G2ClientNeckAngles(
    ghoul2: *mut c_void,
    time: c_int,
    lookAngles: vec3_t,
    headAngles: &mut vec3_t,
    neckAngles: &mut vec3_t,
    thoracicAngles: &mut vec3_t,
    headClampMinAngles: vec3_t,
    headClampMaxAngles: vec3_t,
    traps: &dyn BgTraps,
) {
    let mut lA: vec3_t = lookAngles;
    //clamp the headangles (which should now be relative to the cervical (neck) angles
    if lA[PITCH] < headClampMinAngles[PITCH] {
        lA[PITCH] = headClampMinAngles[PITCH];
    } else if lA[PITCH] > headClampMaxAngles[PITCH] {
        lA[PITCH] = headClampMaxAngles[PITCH];
    }
    if lA[YAW] < headClampMinAngles[YAW] {
        lA[YAW] = headClampMinAngles[YAW];
    } else if lA[YAW] > headClampMaxAngles[YAW] {
        lA[YAW] = headClampMaxAngles[YAW];
    }
    if lA[ROLL] < headClampMinAngles[ROLL] {
        lA[ROLL] = headClampMinAngles[ROLL];
    } else if lA[ROLL] > headClampMaxAngles[ROLL] {
        lA[ROLL] = headClampMaxAngles[ROLL];
    }

    //split it up between the neck and cranium
    // Raven's `0.4`/`0.1`/`0.6` are UNSUFFIXED double literals (the `0.5f` and
    // neck `0.2f`/`0.3f` are floats): the thoracic/head expressions run in
    // f64 and narrow once on store. f32-flattening here was a 1-ULP cranium
    // divergence (lockstep t=10250 find, 2026-07-14).
    if thoracicAngles[PITCH] != 0.0 {
        //already been set above, blend them
        thoracicAngles[PITCH] =
            ((thoracicAngles[PITCH] as f64 + (lA[PITCH] as f64 * 0.4)) * 0.5) as f32;
    } else {
        thoracicAngles[PITCH] = (lA[PITCH] as f64 * 0.4) as f32;
    }
    if thoracicAngles[YAW] != 0.0 {
        thoracicAngles[YAW] = ((thoracicAngles[YAW] as f64 + (lA[YAW] as f64 * 0.1)) * 0.5) as f32;
    } else {
        thoracicAngles[YAW] = (lA[YAW] as f64 * 0.1) as f32;
    }
    if thoracicAngles[ROLL] != 0.0 {
        thoracicAngles[ROLL] =
            ((thoracicAngles[ROLL] as f64 + (lA[ROLL] as f64 * 0.1)) * 0.5) as f32;
    } else {
        thoracicAngles[ROLL] = (lA[ROLL] as f64 * 0.1) as f32;
    }

    neckAngles[PITCH] = lA[PITCH] * 0.2;
    neckAngles[YAW] = lA[YAW] * 0.3;
    neckAngles[ROLL] = lA[ROLL] * 0.3;

    headAngles[PITCH] = (lA[PITCH] as f64 * 0.4) as f32;
    headAngles[YAW] = (lA[YAW] as f64 * 0.6) as f32;
    headAngles[ROLL] = (lA[ROLL] as f64 * 0.6) as f32;

    unsafe {
        traps.g2api_set_bone_angles(
            ghoul2,
            0,
            "cranium",
            &*headAngles,
            BONE_ANGLES_POSTMULT,
            POSITIVE_X as c_int,
            NEGATIVE_Y as c_int,
            NEGATIVE_Z as c_int,
            core::ptr::null_mut(),
            0,
            time,
        );
        traps.g2api_set_bone_angles(
            ghoul2,
            0,
            "cervical",
            &*neckAngles,
            BONE_ANGLES_POSTMULT,
            POSITIVE_X as c_int,
            NEGATIVE_Y as c_int,
            NEGATIVE_Z as c_int,
            core::ptr::null_mut(),
            0,
            time,
        );
        traps.g2api_set_bone_angles(
            ghoul2,
            0,
            "thoracic",
            &*thoracicAngles,
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

/// Raven `BG_G2ClientSpineAngles`.
///
/// Source: `oracle/codemp/game/bg_pmove.c:8869-8990`
// `viewAngles`/`thoracicAngles`/`ulAngles`/`llAngles` are written in place → `&mut vec3_t`;
// `cent_lerpOrigin`/`cent_lerpAngles`/`angles`/`modelScale` are read-only → keep by-value `vec3_t`.
// Only Raven's active `#if 1` correction path is ported (the `#else` branch is dead); with `#if 1`,
// `tPitchAngle`/`tYawAngle`/`corrTime` are unreferenced.
pub fn BG_G2ClientSpineAngles(
    ghoul2: *mut c_void,
    motionBolt: c_int,
    cent_lerpOrigin: vec3_t,
    cent_lerpAngles: vec3_t,
    cent: *mut entityState_t,
    time: c_int,
    viewAngles: &mut vec3_t,
    ciLegs: c_int,
    ciTorso: c_int,
    angles: vec3_t,
    thoracicAngles: &mut vec3_t,
    ulAngles: &mut vec3_t,
    llAngles: &mut vec3_t,
    modelScale: vec3_t,
    tPitchAngle: *mut f32,
    tYawAngle: *mut f32,
    corrTime: *mut c_int,
    traps: &dyn BgTraps,
) {
    use crate::bg_panimate::{
        BG_FlippingAnim, BG_InDeathAnim, BG_InSpecialJump, BG_SaberInSpecial,
        BG_SaberInSpecialAttack, BG_SpinningSaberAnim,
    };
    unsafe {
        let mut doCorr = qfalse;

        //*tPitchAngle = viewAngles[PITCH];
        viewAngles[YAW] = AngleDelta(cent_lerpAngles[YAW], angles[YAW]);
        //*tYawAngle = viewAngles[YAW];

        if BG_FlippingAnim((*cent).legsAnim) == qfalse
            && BG_SpinningSaberAnim((*cent).legsAnim) == qfalse
            && BG_SpinningSaberAnim((*cent).torsoAnim) == qfalse
            && BG_InSpecialJump((*cent).legsAnim) == qfalse
            && BG_InSpecialJump((*cent).torsoAnim) == qfalse
            && BG_InDeathAnim((*cent).legsAnim) == qfalse
            && BG_InDeathAnim((*cent).torsoAnim) == qfalse
            && BG_InRollES(cent, (*cent).legsAnim) == qfalse
            && BG_InRollAnim(cent) == qfalse
            && BG_SaberInSpecial((*cent).saberMove) == qfalse
            && BG_SaberInSpecialAttack((*cent).torsoAnim) == qfalse
            && BG_SaberInSpecialAttack((*cent).legsAnim) == qfalse
            && BG_InKnockDown((*cent).torsoAnim) == qfalse
            && BG_InKnockDown((*cent).legsAnim) == qfalse
            && BG_InKnockDown(ciTorso) == qfalse
            && BG_InKnockDown(ciLegs) == qfalse
            && BG_FlippingAnim(ciLegs) == qfalse
            && BG_SpinningSaberAnim(ciLegs) == qfalse
            && BG_SpinningSaberAnim(ciTorso) == qfalse
            && BG_InSpecialJump(ciLegs) == qfalse
            && BG_InSpecialJump(ciTorso) == qfalse
            && BG_InDeathAnim(ciLegs) == qfalse
            && BG_InDeathAnim(ciTorso) == qfalse
            && BG_SaberInSpecialAttack(ciTorso) == qfalse
            && BG_SaberInSpecialAttack(ciLegs) == qfalse
            && ((*cent).eFlags & EF_DEAD) == 0
            && (*cent).legsAnim != (*cent).torsoAnim
            && ciLegs != ciTorso
            && (*cent).m_iVehicleNum == 0
        {
            doCorr = qtrue;
        }

        if doCorr == qtrue {
            //FIXME: no need to do this if legs and torso on are same frame
            //adjust for motion offset
            let mut boltMatrix = mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            };
            let mut motionFwd: vec3_t = [0.0; 3];
            let mut motionAngles: vec3_t = [0.0; 3];
            let mut motionRt: vec3_t = [0.0; 3];
            let mut tempAng: vec3_t = [0.0; 3];

            traps.g2api_get_bolt_matrix_no_rec_no_rot(
                ghoul2,
                0,
                motionBolt,
                &mut boltMatrix,
                &vec3_origin,
                &cent_lerpOrigin,
                time,
                core::ptr::null_mut(),
                &modelScale,
            );
            //BG_GiveMeVectorFromMatrix( &boltMatrix, NEGATIVE_Y, motionFwd );
            motionFwd[0] = -boltMatrix.matrix[0][1];
            motionFwd[1] = -boltMatrix.matrix[1][1];
            motionFwd[2] = -boltMatrix.matrix[2][1];

            vectoangles(motionFwd, &mut motionAngles);

            //BG_GiveMeVectorFromMatrix( &boltMatrix, NEGATIVE_X, motionRt );
            motionRt[0] = -boltMatrix.matrix[0][0];
            motionRt[1] = -boltMatrix.matrix[1][0];
            motionRt[2] = -boltMatrix.matrix[2][0];

            vectoangles(motionRt, &mut tempAng);
            motionAngles[ROLL] = -tempAng[PITCH];

            for ang in 0..3 {
                viewAngles[ang] =
                    AngleNormalize180(viewAngles[ang] - AngleNormalize180(motionAngles[ang]));
            }
        }

        //distribute the angles differently up the spine
        //NOTE: each of these distributions must add up to 1.0f
        thoracicAngles[PITCH] = viewAngles[PITCH] * 0.20;
        llAngles[PITCH] = viewAngles[PITCH] * 0.40;
        ulAngles[PITCH] = viewAngles[PITCH] * 0.40;

        thoracicAngles[YAW] = viewAngles[YAW] * 0.20;
        ulAngles[YAW] = viewAngles[YAW] * 0.35;
        llAngles[YAW] = viewAngles[YAW] * 0.45;

        thoracicAngles[ROLL] = viewAngles[ROLL] * 0.20;
        ulAngles[ROLL] = viewAngles[ROLL] * 0.35;
        llAngles[ROLL] = viewAngles[ROLL] * 0.45;
    }
}

/// Raven `BG_SwingAngles`.
///
/// Raven: `CG_SwingAngles` — swing an angle towards a destination, modifying
/// speed by the delta and clamping to tolerance.
/// Source: `oracle/codemp/game/bg_pmove.c:8997-9053`
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
/// Source: `oracle/codemp/game/bg_pmove.c:9058-9078`
pub fn BG_InRoll2(es: *mut entityState_t) -> qboolean {
    use animNumber_t::*;
    unsafe {
        let a = (*es).legsAnim;
        if a == BOTH_GETUP_BROLL_B as c_int
            || a == BOTH_GETUP_BROLL_F as c_int
            || a == BOTH_GETUP_BROLL_L as c_int
            || a == BOTH_GETUP_BROLL_R as c_int
            || a == BOTH_GETUP_FROLL_B as c_int
            || a == BOTH_GETUP_FROLL_F as c_int
            || a == BOTH_GETUP_FROLL_L as c_int
            || a == BOTH_GETUP_FROLL_R as c_int
            || a == BOTH_ROLL_F as c_int
            || a == BOTH_ROLL_B as c_int
            || a == BOTH_ROLL_R as c_int
            || a == BOTH_ROLL_L as c_int
        {
            return qtrue;
        }
        qfalse
    }
}

/// Raven `BG_G2PlayerAngles` — compute the torso/legs/neck bone angles for a
/// player skeleton and drive them into the g2 instance.
///
/// `WeaponReadyAnim` is the bg const weapon-ready-anim table. Raven's
/// function-scope `static` scratch is single-call temporaries → plain
/// locals. `VEH_CONTROL_SCHEME_4`/`BONE_BASED_LEG_ANGLES` are undefined.
/// `legsAngles`/`turAngles` are written out-params (`&mut`); `legs` is
/// the axis matrix out (`*mut vec3_t`).
/// Source: `oracle/codemp/game/bg_pmove.c:9082-9457`
pub fn BG_G2PlayerAngles(
    ghoul2: *mut c_void,
    motionBolt: c_int,
    cent: *mut entityState_t,
    time: c_int,
    cent_lerpOrigin: vec3_t,
    cent_lerpAngles: vec3_t,
    legs: *mut vec3_t,
    legsAngles: &mut vec3_t,
    tYawing: *mut qboolean,
    tPitching: *mut qboolean,
    lYawing: *mut qboolean,
    tYawAngle: *mut f32,
    tPitchAngle: *mut f32,
    lYawAngle: *mut f32,
    frametime: c_int,
    turAngles: &mut vec3_t,
    modelScale: vec3_t,
    ciLegs: c_int,
    ciTorso: c_int,
    corrTime: *mut c_int,
    lookAngles: vec3_t,
    lastHeadAngles: &mut vec3_t,
    lookTime: c_int,
    emplaced: *mut entityState_t,
    crazySmoothFactor: *mut c_int,
    traps: &dyn BgTraps,
) {
    // C `vec3_t lastHeadAngles` decays to float*: BG_UpdateLookAngles' final
    // VectorCopy writes back into the caller's `client->lastHeadAngles` — that
    // persistence is read next frame, so it must stay `&mut`. `lookAngles` is a
    // caller local never read after the call → by value.
    let mut lookAngles: vec3_t = lookAngles;

    let mut adddir: c_int = 0;
    let dir: c_int;
    let mut degrees_negative: f32 = 0.0;
    let mut degrees_positive: f32 = 0.0;
    let mut dif: f32;
    let dest: f32;
    let mut speed: f32;
    let lookSpeed: f32 = 1.5;
    let mut eyeAngles: vec3_t = [0.0; 3];
    let mut neckAngles: vec3_t = [0.0; 3];
    let mut velocity: vec3_t = [0.0; 3];
    let mut torsoAngles: vec3_t = [0.0; 3];
    let mut headAngles: vec3_t = [0.0; 3];
    let mut velPos: vec3_t = [0.0; 3];
    let mut velAng: vec3_t = [0.0; 3];
    let mut ulAngles: vec3_t = [0.0; 3];
    let mut llAngles: vec3_t = [0.0; 3];
    let mut viewAngles: vec3_t = [0.0; 3];
    let mut angles: vec3_t = [0.0; 3];
    let mut thoracicAngles: vec3_t = [0.0, 0.0, 0.0];
    let headClampMinAngles: vec3_t = [-25.0, -55.0, -10.0];
    let headClampMaxAngles: vec3_t = [50.0, 50.0, 10.0];

    unsafe {
        if (*cent).m_iVehicleNum != 0
            || (*cent).forceFrame != 0
            || BG_SaberLockBreakAnim((*cent).legsAnim) == qtrue
            || BG_SaberLockBreakAnim((*cent).torsoAnim) == qtrue
        {
            // a vehicle or riding a vehicle - we don't need to be in here
            let mut forcedAngles: vec3_t = [0.0; 3];

            VectorClear(&mut forcedAngles);
            forcedAngles[YAW] = cent_lerpAngles[YAW];
            forcedAngles[ROLL] = cent_lerpAngles[ROLL];
            AnglesToAxis(forcedAngles, legs);
            _VectorCopy(forcedAngles, legsAngles);

            if (*cent).number < MAX_CLIENTS as c_int {
                for bone in [
                    "lower_lumbar",
                    "upper_lumbar",
                    "cranium",
                    "thoracic",
                    "cervical",
                ] {
                    traps.g2api_set_bone_angles(
                        ghoul2,
                        0,
                        bone,
                        &vec3_origin,
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
            return;
        }

        if (time + 2000) < *corrTime {
            *corrTime = 0;
        }

        _VectorCopy(cent_lerpAngles, &mut headAngles);
        headAngles[YAW] = AngleMod(headAngles[YAW]);
        VectorClear(legsAngles);
        VectorClear(&mut torsoAngles);
        // --------- yaw -------------

        // allow yaw to drift a bit
        if ((*cent).legsAnim) != BOTH_STAND1 as c_int
            || ((*cent).torsoAnim) != WeaponReadyAnim[(*cent).weapon as usize]
        {
            // if not standing still, always point all in the same direction
            *tYawing = qtrue; // always center
            *tPitching = qtrue; // always center
            *lYawing = qtrue; // always center
        }

        // adjust legs for movement dir
        if (*cent).eFlags & EF_DEAD != 0 {
            // don't let dead bodies twitch
            dir = 0;
        } else {
            dir = (*cent).angles2[YAW] as c_int;
            if dir < 0 || dir > 7 {
                traps.com_error(
                    ERR_DROP as c_int,
                    &format!("Bad player movement angle ({})", dir),
                );
            }
        }

        torsoAngles[YAW] = headAngles[YAW];

        // for now, turn torso instantly and let the legs swing to follow
        *tYawAngle = torsoAngles[YAW];

        // --------- pitch -------------

        _VectorCopy((*cent).pos.trDelta, &mut velocity);

        if BG_InRoll2(cent) == qtrue {
            // don't affect angles based on vel then
            VectorClear(&mut velocity);
        } else if (*cent).weapon == WP_SABER as c_int
            && BG_SaberInSpecial((*cent).saberMove) == qtrue
        {
            VectorClear(&mut velocity);
        }

        speed = VectorNormalize(&mut velocity);

        if speed == 0.0 {
            torsoAngles[YAW] = headAngles[YAW];
        }

        // only show a fraction of the pitch angle in the torso
        if headAngles[PITCH] > 180.0 {
            dest = (((-360.0f32 + headAngles[PITCH]) as f64) * 0.75) as f32;
        } else {
            dest = (headAngles[PITCH] as f64 * 0.75) as f32;
        }

        if (*cent).m_iVehicleNum != 0 {
            // swing instantly on vehicles
            *tPitchAngle = dest;
        } else {
            BG_SwingAngles(dest, 15.0, 30.0, 0.1, tPitchAngle, tPitching, frametime);
        }
        torsoAngles[PITCH] = *tPitchAngle;

        // --------- roll -------------

        if speed != 0.0 {
            let mut axis: [vec3_t; 3] = [[0.0; 3]; 3];
            let mut side: f32;

            speed = (speed as f64 * 0.05) as f32;

            AnglesToAxis(*legsAngles, axis.as_mut_ptr());
            side = speed * _DotProduct(velocity, axis[1]);
            legsAngles[ROLL] -= side;

            side = speed * _DotProduct(velocity, axis[0]);
            legsAngles[PITCH] += side;
        }

        // rww - crazy velocity-based leg angle calculation
        legsAngles[YAW] = headAngles[YAW];
        velPos[0] = cent_lerpOrigin[0] + velocity[0];
        velPos[1] = cent_lerpOrigin[1] + velocity[1];
        velPos[2] = cent_lerpOrigin[2]; // + velocity[2];

        if (*cent).groundEntityNum == ENTITYNUM_NONE
            || (*cent).forceFrame != 0
            || ((*cent).weapon == WP_EMPLACED_GUN as c_int && !emplaced.is_null())
        {
            // off the ground, no direction-based leg angles (same if in saberlock)
            _VectorCopy(cent_lerpOrigin, &mut velPos);
        }

        _VectorSubtract(cent_lerpOrigin, velPos, &mut velAng);

        if !VectorCompare(velAng, vec3_origin) {
            vectoangles(velAng, &mut velAng);

            if velAng[YAW] <= legsAngles[YAW] {
                degrees_negative = legsAngles[YAW] - velAng[YAW];
                degrees_positive = (360.0 - legsAngles[YAW]) + velAng[YAW];
            } else {
                degrees_negative = legsAngles[YAW] + (360.0 - velAng[YAW]);
                degrees_positive = velAng[YAW] - legsAngles[YAW];
            }

            if degrees_negative < degrees_positive {
                dif = degrees_negative;
                adddir = 0;
            } else {
                dif = degrees_positive;
                adddir = 1;
            }

            if dif > 90.0 {
                dif = 180.0 - dif;
            }

            if dif > 60.0 {
                dif = 60.0;
            }

            // Slight hack for when playing is running backward
            if dir == 3 || dir == 5 {
                dif = -dif;
            }

            if adddir != 0 {
                legsAngles[YAW] -= dif;
            } else {
                legsAngles[YAW] += dif;
            }
        }

        if (*cent).m_iVehicleNum != 0 {
            // swing instantly on vehicles
            *lYawAngle = legsAngles[YAW];
        } else {
            BG_SwingAngles(
                legsAngles[YAW],
                0.0,
                90.0,
                0.65,
                lYawAngle,
                lYawing,
                frametime,
            );
        }
        legsAngles[YAW] = *lYawAngle;

        legsAngles[ROLL] = 0.0;
        torsoAngles[ROLL] = 0.0;

        // pull the angles back out of the hierarchial chain
        AnglesSubtract(headAngles, torsoAngles, &mut headAngles);
        AnglesSubtract(torsoAngles, *legsAngles, &mut torsoAngles);

        legsAngles[PITCH] = 0.0;

        if (*cent).heldByClient != 0 {
            // keep the base angles clear when doing the IK stuff
            VectorClear(legsAngles);
            legsAngles[YAW] = cent_lerpAngles[YAW];
        }

        _VectorCopy(*legsAngles, turAngles);

        AnglesToAxis(*legsAngles, legs);

        _VectorCopy(cent_lerpAngles, &mut viewAngles);
        viewAngles[YAW] = 0.0;
        viewAngles[ROLL] = 0.0;
        viewAngles[PITCH] = (viewAngles[PITCH] as f64 * 0.5) as f32;

        VectorSet(&mut angles, 0.0, legsAngles[1], 0.0);

        angles[0] = legsAngles[0];
        if angles[0] > 30.0 {
            angles[0] = 30.0;
        } else if angles[0] < -30.0 {
            angles[0] = -30.0;
        }

        if (*cent).weapon == WP_EMPLACED_GUN as c_int && !emplaced.is_null() {
            // if using an emplaced gun, make sure we're angled to "hold" it right
            let mut facingAngles: vec3_t = [0.0; 3];

            _VectorSubtract((*emplaced).pos.trBase, cent_lerpOrigin, &mut facingAngles);
            vectoangles(facingAngles, &mut facingAngles);

            if (*emplaced).weapon == WP_NONE as c_int {
                // e-web
                _VectorCopy(facingAngles, legsAngles);
                AnglesToAxis(*legsAngles, legs);
            } else {
                // misc emplaced
                let dif2 = AngleSubtract(cent_lerpAngles[YAW], facingAngles[YAW]);

                VectorSet(&mut facingAngles, -16.0, -dif2, 0.0);

                if (*cent).legsAnim == BOTH_STRAFE_LEFT1 as c_int
                    || (*cent).legsAnim == BOTH_STRAFE_RIGHT1 as c_int
                {
                    // try to adjust so it doesn't look wrong
                    if !crazySmoothFactor.is_null() {
                        // want to smooth a lot during this because it chops around
                        *crazySmoothFactor = time + 1000;
                    }

                    BG_G2ClientSpineAngles(
                        ghoul2,
                        motionBolt,
                        cent_lerpOrigin,
                        cent_lerpAngles,
                        cent,
                        time,
                        &mut viewAngles,
                        ciLegs,
                        ciTorso,
                        angles,
                        &mut thoracicAngles,
                        &mut ulAngles,
                        &mut llAngles,
                        modelScale,
                        tPitchAngle,
                        tYawAngle,
                        corrTime,
                        traps,
                    );
                    traps.g2api_set_bone_angles(
                        ghoul2,
                        0,
                        "lower_lumbar",
                        &llAngles,
                        BONE_ANGLES_POSTMULT,
                        POSITIVE_X as c_int,
                        NEGATIVE_Y as c_int,
                        NEGATIVE_Z as c_int,
                        core::ptr::null_mut(),
                        0,
                        time,
                    );
                    traps.g2api_set_bone_angles(
                        ghoul2,
                        0,
                        "upper_lumbar",
                        &ulAngles,
                        BONE_ANGLES_POSTMULT,
                        POSITIVE_X as c_int,
                        NEGATIVE_Y as c_int,
                        NEGATIVE_Z as c_int,
                        core::ptr::null_mut(),
                        0,
                        time,
                    );
                    traps.g2api_set_bone_angles(
                        ghoul2,
                        0,
                        "cranium",
                        &vec3_origin,
                        BONE_ANGLES_POSTMULT,
                        POSITIVE_X as c_int,
                        NEGATIVE_Y as c_int,
                        NEGATIVE_Z as c_int,
                        core::ptr::null_mut(),
                        0,
                        time,
                    );

                    _VectorAdd(facingAngles, thoracicAngles, &mut facingAngles);

                    if (*cent).legsAnim == BOTH_STRAFE_LEFT1 as c_int {
                        // this one needs some further correction
                        facingAngles[YAW] -= 32.0;
                    }
                } else {
                    traps.g2api_set_bone_angles(
                        ghoul2,
                        0,
                        "cranium",
                        &vec3_origin,
                        BONE_ANGLES_POSTMULT,
                        POSITIVE_X as c_int,
                        NEGATIVE_Y as c_int,
                        NEGATIVE_Z as c_int,
                        core::ptr::null_mut(),
                        0,
                        time,
                    );
                }

                _VectorScale(facingAngles, 0.6, &mut facingAngles);
                traps.g2api_set_bone_angles(
                    ghoul2,
                    0,
                    "lower_lumbar",
                    &vec3_origin,
                    BONE_ANGLES_POSTMULT,
                    POSITIVE_X as c_int,
                    NEGATIVE_Y as c_int,
                    NEGATIVE_Z as c_int,
                    core::ptr::null_mut(),
                    0,
                    time,
                );
                _VectorScale(facingAngles, 0.8, &mut facingAngles);
                traps.g2api_set_bone_angles(
                    ghoul2,
                    0,
                    "upper_lumbar",
                    &facingAngles,
                    BONE_ANGLES_POSTMULT,
                    POSITIVE_X as c_int,
                    NEGATIVE_Y as c_int,
                    NEGATIVE_Z as c_int,
                    core::ptr::null_mut(),
                    0,
                    time,
                );
                _VectorScale(facingAngles, 0.8, &mut facingAngles);
                traps.g2api_set_bone_angles(
                    ghoul2,
                    0,
                    "thoracic",
                    &facingAngles,
                    BONE_ANGLES_POSTMULT,
                    POSITIVE_X as c_int,
                    NEGATIVE_Y as c_int,
                    NEGATIVE_Z as c_int,
                    core::ptr::null_mut(),
                    0,
                    time,
                );

                // Now we want the head angled toward where we are facing
                VectorSet(&mut facingAngles, 0.0, dif2, 0.0);
                _VectorScale(facingAngles, 0.6, &mut facingAngles);
                traps.g2api_set_bone_angles(
                    ghoul2,
                    0,
                    "cervical",
                    &facingAngles,
                    BONE_ANGLES_POSTMULT,
                    POSITIVE_X as c_int,
                    NEGATIVE_Y as c_int,
                    NEGATIVE_Z as c_int,
                    core::ptr::null_mut(),
                    0,
                    time,
                );

                return; // don't have to bother with the rest then
            }
        }

        BG_G2ClientSpineAngles(
            ghoul2,
            motionBolt,
            cent_lerpOrigin,
            cent_lerpAngles,
            cent,
            time,
            &mut viewAngles,
            ciLegs,
            ciTorso,
            angles,
            &mut thoracicAngles,
            &mut ulAngles,
            &mut llAngles,
            modelScale,
            tPitchAngle,
            tYawAngle,
            corrTime,
            traps,
        );

        _VectorCopy(cent_lerpAngles, &mut eyeAngles);

        for i in 0..3usize {
            lookAngles[i] = AngleNormalize180(lookAngles[i]);
            eyeAngles[i] = AngleNormalize180(eyeAngles[i]);
        }
        AnglesSubtract(lookAngles, eyeAngles, &mut lookAngles);

        // Referee probe: BG_UpdateLookAngles inputs (look angles in, lookTime, last head angles).
        probe!(
            "LOOK_UPD",
            "t={} en={} li={:08x},{:08x},{:08x} lt={} lh={:08x},{:08x},{:08x}",
            time,
            (*cent).number,
            lookAngles[0].to_bits(),
            lookAngles[1].to_bits(),
            lookAngles[2].to_bits(),
            lookTime,
            lastHeadAngles[0].to_bits(),
            lastHeadAngles[1].to_bits(),
            lastHeadAngles[2].to_bits(),
        );
        BG_UpdateLookAngles(
            lookTime,
            lastHeadAngles,
            time,
            &mut lookAngles,
            lookSpeed,
            -50.0,
            50.0,
            -70.0,
            70.0,
            -30.0,
            30.0,
        );
        // Referee probe: BG_UpdateLookAngles output look angles (write-back).
        probe!(
            "LOOK_WB",
            "t={} en={} lo={:08x},{:08x},{:08x}",
            time,
            (*cent).number,
            lookAngles[0].to_bits(),
            lookAngles[1].to_bits(),
            lookAngles[2].to_bits(),
        );

        BG_G2ClientNeckAngles(
            ghoul2,
            time,
            lookAngles,
            &mut headAngles,
            &mut neckAngles,
            &mut thoracicAngles,
            headClampMinAngles,
            headClampMaxAngles,
            traps,
        );

        traps.g2api_set_bone_angles(
            ghoul2,
            0,
            "lower_lumbar",
            &llAngles,
            BONE_ANGLES_POSTMULT,
            POSITIVE_X as c_int,
            NEGATIVE_Y as c_int,
            NEGATIVE_Z as c_int,
            core::ptr::null_mut(),
            0,
            time,
        );
        traps.g2api_set_bone_angles(
            ghoul2,
            0,
            "upper_lumbar",
            &ulAngles,
            BONE_ANGLES_POSTMULT,
            POSITIVE_X as c_int,
            NEGATIVE_Y as c_int,
            NEGATIVE_Z as c_int,
            core::ptr::null_mut(),
            0,
            time,
        );
        traps.g2api_set_bone_angles(
            ghoul2,
            0,
            "thoracic",
            &thoracicAngles,
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

/// Raven `BG_G2ATSTAngles`.
///
/// Source: `oracle/codemp/game/bg_pmove.c:9459-9462`
pub fn BG_G2ATSTAngles(
    ghoul2: *mut c_void,
    time: c_int,
    cent_lerpAngles: vec3_t,
    traps: &dyn BgTraps,
) {
    unsafe {
        // up = POSITIVE_X, right = NEGATIVE_Y, fwd = NEGATIVE_Z
        traps.g2api_set_bone_angles(
            ghoul2,
            0,
            "thoracic",
            &cent_lerpAngles,
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
/// Source: `oracle/codemp/game/bg_pmove.c:9464-9469`
pub fn PM_AdjustAnglesForDualJumpAttack(ps: *mut playerState_t, ucmd: *mut usercmd_t) -> qboolean {
    qtrue
}

/// Raven `PM_CmdForSaberMoves` — force movement/jump commands for the special
/// dual/staff jump/spin saber attacks.
/// Source: `oracle/codemp/game/bg_pmove.c:9474-9639`
impl PmoveContext<'_> {
    pub fn PM_CmdForSaberMoves(&mut self, ucmd: *mut usercmd_t) {
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;

            // DUAL FORWARD+JUMP+ATTACK
            if ((*ps).legsAnim == BOTH_JUMPATTACK6 as c_int
                && (*ps).saberMove == LS_JUMPATTACK_DUAL)
                || ((*ps).legsAnim == BOTH_BUTTERFLY_FL1 as c_int
                    && (*ps).saberMove == LS_JUMPATTACK_STAFF_LEFT)
                || ((*ps).legsAnim == BOTH_BUTTERFLY_FR1 as c_int
                    && (*ps).saberMove == LS_JUMPATTACK_STAFF_RIGHT)
                || ((*ps).legsAnim == BOTH_BUTTERFLY_RIGHT as c_int
                    && (*ps).saberMove == LS_BUTTERFLY_RIGHT)
                || ((*ps).legsAnim == BOTH_BUTTERFLY_LEFT as c_int
                    && (*ps).saberMove == LS_BUTTERFLY_LEFT)
            {
                let aLen = self.PM_AnimLength(0, BOTH_JUMPATTACK6 as c_int);

                (*ucmd).forwardmove = 0;
                (*ucmd).rightmove = 0;
                (*ucmd).upmove = 0;

                if (*ps).legsAnim == BOTH_JUMPATTACK6 as c_int {
                    // dual stance attack
                    if (*ps).legsTimer >= 100 // not at end
                        && (aLen - (*ps).legsTimer) >= 250
                    {
                        // middle of anim — push forward
                        (*ucmd).forwardmove = 127;
                    }

                    if ((*ps).legsTimer >= 900 && aLen - (*ps).legsTimer >= 950)
                        || ((*ps).legsTimer >= 1600 && aLen - (*ps).legsTimer >= 400)
                    {
                        // one of the two jumps
                        if (*ps).groundEntityNum != ENTITYNUM_NONE {
                            // still on ground?
                            if (*ps).groundEntityNum >= MAX_CLIENTS as c_int {
                                // jump!
                                (*ps).velocity[2] = 250.0; //400;
                                (*ps).fd.forceJumpZStart = (*ps).origin[2]; //so we don't take damage if we land at same height
                                self.PM_AddEvent(EV_JUMP as c_int);
                            }
                        }
                    }
                } else {
                    // saberstaff attacks
                    let aLen = self.PM_AnimLength(0, (*ps).legsAnim);
                    let mut lenMin: f32 = 1700.0;
                    let mut lenMax: f32 = 1800.0;

                    if (*ps).legsAnim == BOTH_BUTTERFLY_LEFT as c_int {
                        lenMin = 1200.0;
                        lenMax = 1400.0;
                    }

                    if (*ps).legsAnim == BOTH_BUTTERFLY_RIGHT as c_int
                        || (*ps).legsAnim == BOTH_BUTTERFLY_LEFT as c_int
                    {
                        if (*ps).legsTimer > 450 {
                            if (*ps).legsAnim == BOTH_BUTTERFLY_LEFT as c_int {
                                (*ucmd).rightmove = -127;
                            } else if (*ps).legsAnim == BOTH_BUTTERFLY_RIGHT as c_int {
                                (*ucmd).rightmove = 127;
                            }
                        }
                    } else {
                        if (*ps).legsTimer >= 100 // not at end
                            && aLen - (*ps).legsTimer >= 250
                        {
                            // middle of anim — push forward
                            (*ucmd).forwardmove = 127;
                        }
                    }

                    if (*ps).legsTimer >= lenMin as c_int && (*ps).legsTimer < lenMax as c_int {
                        // one of the two jumps
                        if (*ps).groundEntityNum != ENTITYNUM_NONE {
                            // still on ground? jump!
                            if (*ps).legsAnim == BOTH_BUTTERFLY_LEFT as c_int {
                                (*ps).velocity[2] = 350.0;
                            } else {
                                (*ps).velocity[2] = 250.0;
                            }
                            (*ps).fd.forceJumpZStart = (*ps).origin[2]; //so we don't take damage if we land at same height
                            self.PM_AddEvent(EV_JUMP as c_int);
                        }
                    }
                }

                if (*ps).groundEntityNum == ENTITYNUM_NONE {
                    // can only turn when your feet hit the ground
                    if PM_AdjustAnglesForDualJumpAttack(ps, ucmd) == qtrue {
                        PM_SetPMViewAngle(ps, (*ps).viewangles, ucmd);
                    }
                }
            }
            // STAFF BACK+JUMP+ATTACK
            else if (*ps).saberMove == LS_A_BACKFLIP_ATK
                && (*ps).legsAnim == BOTH_JUMPATTACK7 as c_int
            {
                let aLen = self.PM_AnimLength(0, BOTH_JUMPATTACK7 as c_int);

                if (*ps).legsTimer > 800 // not at end
                    && aLen - (*ps).legsTimer >= 400
                {
                    // middle of anim
                    if (*ps).groundEntityNum != ENTITYNUM_NONE {
                        // still on ground?
                        let mut yawAngles: vec3_t = [0.0; 3];
                        let mut backDir: vec3_t = [0.0; 3];

                        // push backwards some?
                        VectorSet(&mut yawAngles, 0.0, (*ps).viewangles[YAW] + 180.0, 0.0);
                        AngleVectors(yawAngles, Some(&mut backDir), None, None);
                        _VectorScale(backDir, 100.0, &mut (*ps).velocity);

                        // jump!
                        (*ps).velocity[2] = 300.0;
                        (*ps).fd.forceJumpZStart = (*ps).origin[2]; //so we don't take damage if we land at same height

                        self.PM_AddEvent(EV_JUMP as c_int);
                        (*ucmd).upmove = 0; // clear any actual jump command
                    }
                }
                (*ucmd).forwardmove = 0;
                (*ucmd).rightmove = 0;
                (*ucmd).upmove = 0;
            }
            // STAFF/DUAL SPIN ATTACK
            else if (*ps).saberMove == LS_SPINATTACK || (*ps).saberMove == LS_SPINATTACK_DUAL {
                (*ucmd).forwardmove = 0;
                (*ucmd).rightmove = 0;
                (*ucmd).upmove = 0;
                // lock their viewangles during these attacks.
                PM_SetPMViewAngle(ps, (*ps).viewangles, ucmd);
            }
        }
    }
}

/// Raven `PM_VehicleViewAngles`.
///
/// Raven: constrain the rider's viewangles based on the vehicle's caps (or leave
/// a turret-operating passenger unclamped). `VEH_CONTROL_SCHEME_4` is undefined,
/// so the `#else` (BG_UnrestrainedPitchRoll) branch is the compiled one.
/// Source: `oracle/codemp/game/bg_pmove.c:9642-9713`
pub fn PM_VehicleViewAngles(
    ps: *mut playerState_t,
    veh: *mut bgEntity_t,
    ucmd: *mut usercmd_t,
    bg: &BgState,
) {
    unsafe {
        let pVeh: *mut Vehicle_t = ((*veh).m_pVehicle as *mut Vehicle_t);
        let mut setAngles: qboolean = qtrue;
        // §19: oracle reads clampMin/clampMax uninitialized for a non-pilot,
        // non-turret passenger (UB); zero-init picks the "no allowance" clamp arm.
        let mut clampMin: vec3_t = [0.0; 3];
        let mut clampMax: vec3_t = [0.0; 3];

        if !(*((*veh).m_pVehicle as *mut Vehicle_t)).m_pPilot.is_null()
            && (*(*((*veh).m_pVehicle as *mut Vehicle_t)).m_pPilot)
                .s
                .number
                == (*ps).clientNum
        {
            // set the pilot's viewangles to the vehicle's viewangles, but only if
            // not doing special free-roll/pitch control
            if BG_UnrestrainedPitchRoll(ps, ((*veh).m_pVehicle as *mut Vehicle_t), bg) == qfalse {
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
                if (*(*((*veh).m_pVehicle as *mut Vehicle_t)).m_pVehicleInfo).turret[i as usize]
                    .passengerNum
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
/// Source: `oracle/codemp/game/bg_pmove.c:9745-9759`
pub fn PM_WeaponOkOnVehicle(weapon: c_int) -> qboolean {
    // FIXME (Raven): check g_vehicleInfo for our vehicle?
    if weapon == WP_MELEE as c_int || weapon == WP_SABER as c_int || weapon == WP_BLASTER as c_int {
        return qtrue;
    }
    qfalse
}

/// Raven `PM_GetOkWeaponForVehicle` — first weapon the client owns that is
/// usable on a vehicle, or -1.
/// Source: `oracle/codemp/game/bg_pmove.c:9762-9780`
impl PmoveContext<'_> {
    pub fn PM_GetOkWeaponForVehicle(&mut self) -> c_int {
        unsafe {
            let ps = (*self.pm).ps;
            let mut i: c_int = 0;

            while i < WP_NUM_WEAPONS as c_int {
                if (*ps).stats[statIndex_t::STAT_WEAPONS as usize] & (1 << i) != 0
                    && PM_WeaponOkOnVehicle(i) == qtrue
                {
                    // this one's good
                    return i;
                }
                i += 1;
            }

            // oh dear!
            -1
        }
    }
}

/// Raven `PM_VehForcedTurning` — steer a vehicle to face its turnaround target.
/// `VEH_CONTROL_SCHEME_4` is undefined, so the `#else` branch is compiled.
/// Source: `oracle/codemp/game/bg_pmove.c:9783-9830`
impl PmoveContext<'_> {
    pub fn PM_VehForcedTurning(&mut self, veh: *mut bgEntity_t) {
        unsafe {
            let dst = self.PM_BGEntForNum((*(*veh).playerState).vehTurnaroundIndex);
            let mut pitchD: f32;
            let mut yawD: f32;
            let mut dir: vec3_t = [0.0; 3];

            if veh.is_null() || ((*veh).m_pVehicle as *mut Vehicle_t).is_null() {
                return;
            }

            if dst.is_null() {
                // can't find dest ent?
                return;
            }

            let pv = (*veh).m_pVehicle as *mut Vehicle_t;
            (*pv).m_ucmd.upmove = 127;
            (*self.pm).cmd.upmove = 127;
            (*pv).m_ucmd.forwardmove = 0;
            (*self.pm).cmd.forwardmove = 0;
            (*pv).m_ucmd.rightmove = 0;
            (*self.pm).cmd.rightmove = 0;

            _VectorSubtract((*dst).s.origin, (*(*veh).playerState).origin, &mut dir);
            vectoangles(dir, &mut dir);

            yawD = AngleSubtract((*(*self.pm).ps).viewangles[YAW], dir[YAW]);
            pitchD = AngleSubtract((*(*self.pm).ps).viewangles[PITCH], dir[PITCH]);

            yawD *= 0.6 * self.pml.frametime;
            pitchD *= 0.6 * self.pml.frametime;

            (*(*self.pm).ps).viewangles[YAW] =
                AngleSubtract((*(*self.pm).ps).viewangles[YAW], yawD);
            (*(*self.pm).ps).viewangles[PITCH] =
                AngleSubtract((*(*self.pm).ps).viewangles[PITCH], pitchD);

            PM_SetPMViewAngle(
                (*self.pm).ps,
                (*(*self.pm).ps).viewangles,
                core::ptr::addr_of_mut!((*self.pm).cmd),
            );
        }
    }
}

/// Raven `PM_VehFaceHyperspacePoint` — rotate a vehicle to face its hyperspace
/// angles, flagging it ready to jump once aligned.
/// Source: `oracle/codemp/game/bg_pmove.c:9916-9989`
impl PmoveContext<'_> {
    pub fn PM_VehFaceHyperspacePoint(&mut self, veh: *mut bgEntity_t) {
        unsafe {
            if veh.is_null() || ((*veh).m_pVehicle as *mut Vehicle_t).is_null() {
                return;
            } else {
                let pv = (*veh).m_pVehicle as *mut Vehicle_t;
                let timeFrac = ((*self.pm).cmd.serverTime - (*(*veh).playerState).hyperSpaceTime)
                    as f32
                    / HYPERSPACE_TIME as f32;
                let turnRate: f32;
                let mut aDelta: f32;
                let mut matchedAxes: c_int = 0;

                (*pv).m_ucmd.upmove = 127;
                (*self.pm).cmd.upmove = 127;
                (*pv).m_ucmd.forwardmove = 0;
                (*self.pm).cmd.forwardmove = 0;
                (*pv).m_ucmd.rightmove = 0;
                (*self.pm).cmd.rightmove = 0;

                turnRate = 90.0 * self.pml.frametime;
                for i in 0..3usize {
                    aDelta = AngleSubtract(
                        (*(*veh).playerState).hyperSpaceAngles[i],
                        *(*pv).m_vOrientation.add(i),
                    );
                    if Q_fabs(aDelta) < turnRate {
                        // all is good
                        (*(*self.pm).ps).viewangles[i] = (*(*veh).playerState).hyperSpaceAngles[i];
                        matchedAxes += 1;
                    } else {
                        aDelta = AngleSubtract(
                            (*(*veh).playerState).hyperSpaceAngles[i],
                            (*(*self.pm).ps).viewangles[i],
                        );
                        if Q_fabs(aDelta) < turnRate {
                            (*(*self.pm).ps).viewangles[i] =
                                (*(*veh).playerState).hyperSpaceAngles[i];
                        } else if aDelta > 0.0 {
                            if i == YAW as usize {
                                (*(*self.pm).ps).viewangles[i] =
                                    AngleNormalize360((*(*self.pm).ps).viewangles[i] + turnRate);
                            } else {
                                (*(*self.pm).ps).viewangles[i] =
                                    AngleNormalize180((*(*self.pm).ps).viewangles[i] + turnRate);
                            }
                        } else {
                            if i == YAW as usize {
                                (*(*self.pm).ps).viewangles[i] =
                                    AngleNormalize360((*(*self.pm).ps).viewangles[i] - turnRate);
                            } else {
                                (*(*self.pm).ps).viewangles[i] =
                                    AngleNormalize180((*(*self.pm).ps).viewangles[i] - turnRate);
                            }
                        }
                    }
                }

                PM_SetPMViewAngle(
                    (*self.pm).ps,
                    (*(*self.pm).ps).viewangles,
                    core::ptr::addr_of_mut!((*self.pm).cmd),
                );

                if timeFrac < HYPERSPACE_TELEPORT_FRAC {
                    // haven't gone through yet
                    if matchedAxes < 3 {
                        // not facing the right dir yet — keep hyperspace time up to date
                        (*(*veh).playerState).hyperSpaceTime += self.pml.msec;
                    } else if (*(*veh).playerState).eFlags2 & EF2_HYPERSPACE == 0 {
                        // flag us as ready to hyperspace!
                        (*(*veh).playerState).eFlags2 |= EF2_HYPERSPACE;
                    }
                }
            }
        }
    }
}

/// The trace channel [`BG_VehicleAdjustBBoxForOrientation`] validates the
/// oriented box through — Raven's `localTrace` fn-ptr param, `None` being
/// Raven's NULL ("don't care about solid stuff", cgame's
/// `CG_ClipMoveToEntities` arm).
pub type VehBBoxTraceFn<'a> = &'a dyn Fn(
    &mut trace_t,
    *const vec3_t,
    *const vec3_t,
    *const vec3_t,
    *const vec3_t,
    c_int,
    c_int,
);

/// Raven `BG_VehicleAdjustBBoxForOrientation` — resize a fighter/flier vehicle's
/// bbox to its oriented extents, tracing to confirm the new box is valid.
///
/// Source: `oracle/codemp/game/bg_pmove.c:9993-10076`
pub fn BG_VehicleAdjustBBoxForOrientation(
    veh: *mut Vehicle_t,
    origin: vec3_t,
    mins: &mut vec3_t,
    maxs: &mut vec3_t,
    clientNum: c_int,
    tracemask: c_int,
    localTrace: Option<VehBBoxTraceFn>,
) {
    // `DEFAULT_MINS_2` canonical in `crate::public::viewheight` (`c_int`,
    // cast here to match the `vec3_t` component it seeds).
    // Source: `oracle/codemp/game/bg_public.h:41`
    const DEFAULT_MINS_2: f32 = crate::public::viewheight::DEFAULT_MINS_2 as f32;

    unsafe {
        if veh.is_null() {
            return;
        }
        let vi = (*veh).m_pVehicleInfo;
        if (*vi).length == 0.0 || (*vi).width == 0.0 || (*vi).height == 0.0 {
            return;
        } else if (*vi).r#type as c_int != vehicleType_t::VH_FIGHTER as c_int
            && (*vi).r#type as c_int != vehicleType_t::VH_FLIER as c_int
        {
            // only those types have dynamic bboxes, the rest use a static bbox
            VectorSet(
                maxs,
                (*vi).width / 2.0,
                (*vi).width / 2.0,
                (*vi).height + DEFAULT_MINS_2,
            );
            VectorSet(mins, (*vi).width / -2.0, (*vi).width / -2.0, DEFAULT_MINS_2);
            return;
        } else {
            let mut axis: [vec3_t; 3] = [[0.0; 3]; 3];
            let mut point: [vec3_t; 8] = [[0.0; 3]; 8];
            let mut newMins: vec3_t = [0.0; 3];
            let mut newMaxs: vec3_t = [0.0; 3];
            let mut trace: trace_t = core::mem::zeroed();

            let len = (*vi).length;
            let wid = (*vi).width;
            let hgt = (*vi).height;

            // m_vOrientation is a `vec3_t*` into the owner; read the 3 floats.
            let orient: vec3_t = [
                *(*veh).m_vOrientation.add(0),
                *(*veh).m_vOrientation.add(1),
                *(*veh).m_vOrientation.add(2),
            ];
            AnglesToAxis(orient, axis.as_mut_ptr());
            _VectorMA(origin, len / 2.0, axis[0], &mut point[0]);
            _VectorMA(origin, -len / 2.0, axis[0], &mut point[1]);
            // extrapolate each side up and down
            let p0 = point[0];
            _VectorMA(p0, hgt / 2.0, axis[2], &mut point[0]);
            let p0 = point[0];
            _VectorMA(p0, -hgt, axis[2], &mut point[2]);
            let p1 = point[1];
            _VectorMA(p1, hgt / 2.0, axis[2], &mut point[1]);
            let p1 = point[1];
            _VectorMA(p1, -hgt, axis[2], &mut point[3]);

            _VectorMA(origin, wid / 2.0, axis[1], &mut point[4]);
            _VectorMA(origin, -wid / 2.0, axis[1], &mut point[5]);
            // extrapolate each side up and down
            let p4 = point[4];
            _VectorMA(p4, hgt / 2.0, axis[2], &mut point[4]);
            let p4 = point[4];
            _VectorMA(p4, -hgt, axis[2], &mut point[6]);
            let p5 = point[5];
            _VectorMA(p5, hgt / 2.0, axis[2], &mut point[5]);
            let p5 = point[5];
            _VectorMA(p5, -hgt, axis[2], &mut point[7]);

            // Now inflate a bbox around these points
            _VectorCopy(origin, &mut newMins);
            _VectorCopy(origin, &mut newMaxs);
            for curAxis in 0..3usize {
                for i in 0..8usize {
                    if point[i][curAxis] > newMaxs[curAxis] {
                        newMaxs[curAxis] = point[i][curAxis];
                    } else if point[i][curAxis] < newMins[curAxis] {
                        newMins[curAxis] = point[i][curAxis];
                    }
                }
            }
            let nmn = newMins;
            _VectorSubtract(nmn, origin, &mut newMins);
            let nmx = newMaxs;
            _VectorSubtract(nmx, origin, &mut newMaxs);
            // now see if that's a valid way to be
            if let Some(localTrace) = localTrace {
                localTrace(
                    &mut trace,
                    &origin as *const vec3_t,
                    &newMins as *const vec3_t,
                    &newMaxs as *const vec3_t,
                    &origin as *const vec3_t,
                    clientNum,
                    tracemask,
                );
            } else {
                // don't care about solid stuff then
                trace.startsolid = 0;
                trace.allsolid = 0;
            }
            if trace.startsolid == 0 && trace.allsolid == 0 {
                // let's use it!
                _VectorCopy(newMins, mins);
                _VectorCopy(newMaxs, maxs);
            }
            // else: just use the last one
        }
    }
}

impl PmoveContext<'_> {
    /// The pmove caller's arm: `localTrace` is the `pm->trace` channel
    /// (`self.traps.trace` via `BgTraps`), always non-null in Raven.
    pub fn BG_VehicleAdjustBBoxForOrientation(
        &self,
        veh: *mut Vehicle_t,
        origin: vec3_t,
        mins: &mut vec3_t,
        maxs: &mut vec3_t,
        clientNum: c_int,
        tracemask: c_int,
    ) {
        BG_VehicleAdjustBBoxForOrientation(
            veh,
            origin,
            mins,
            maxs,
            clientNum,
            tracemask,
            Some(&|results, start, bmins, bmaxs, end, pass, mask| {
                self.traps
                    .trace(results, start, bmins, bmaxs, end, pass, mask)
            }),
        )
    }
}

/// Raven `PM_MoveForKata` — force movement/jump commands during the soulcal and
/// medium/strong kata special attacks.
/// Source: `oracle/codemp/game/bg_pmove.c:10092-10172`
impl PmoveContext<'_> {
    pub fn PM_MoveForKata(&mut self, ucmd: *mut usercmd_t) {
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;

            if (*ps).legsAnim == BOTH_A7_SOULCAL as c_int && (*ps).saberMove == LS_STAFF_SOULCAL {
                // forward spinning staff attack
                (*ucmd).upmove = 0;

                if PM_CanRollFromSoulCal(ps) == qtrue {
                    (*ucmd).upmove = -127;
                    (*ucmd).rightmove = 0;
                    if (*ucmd).forwardmove < 0 {
                        (*ucmd).forwardmove = 0;
                    }
                } else {
                    (*ucmd).rightmove = 0;
                    if (*ps).legsTimer >= 2750 {
                        // not at end — push forward
                        (*ucmd).forwardmove = 64;
                    } else {
                        (*ucmd).forwardmove = 0;
                    }
                }
                if (*ps).legsTimer >= 2650 && (*ps).legsTimer < 2850 {
                    // the jump
                    if (*ps).groundEntityNum != ENTITYNUM_NONE {
                        // still on ground? jump!
                        (*ps).velocity[2] = 250.0;
                        (*ps).fd.forceJumpZStart = (*ps).origin[2]; //so we don't take damage if we land at same height
                        self.PM_AddEvent(EV_JUMP as c_int);
                    }
                }
            } else if (*ps).legsAnim == BOTH_A2_SPECIAL as c_int {
                // medium kata
                (*pm).cmd.rightmove = 0;
                (*pm).cmd.upmove = 0;
                if (*ps).legsTimer < 2700 && (*ps).legsTimer > 2300 {
                    (*pm).cmd.forwardmove = 127;
                } else if (*ps).legsTimer < 900 && (*ps).legsTimer > 500 {
                    (*pm).cmd.forwardmove = 127;
                } else {
                    (*pm).cmd.forwardmove = 0;
                }
            } else if (*ps).legsAnim == BOTH_A3_SPECIAL as c_int {
                // strong kata
                (*pm).cmd.rightmove = 0;
                (*pm).cmd.upmove = 0;
                if (*ps).legsTimer < 1700 && (*ps).legsTimer > 1000 {
                    (*pm).cmd.forwardmove = 127;
                } else {
                    (*pm).cmd.forwardmove = 0;
                }
            } else {
                (*pm).cmd.forwardmove = 0;
                (*pm).cmd.rightmove = 0;
                (*pm).cmd.upmove = 0;
            }
        }
    }
}

// `PmoveSingle` is a `PmoveContext` method above (it owns the per-call working
// set the C file-statics used to hold).

/// Raven `Pmove` — the public pmove entrypoint. Constructs one `PmoveContext`
/// per call from the bg channel handles the game tier supplies,
/// then chops the move into fixed timesteps and runs `PmoveSingle` for each.
/// Source: `oracle/codemp/game/bg_pmove.c:11167-11215`
pub fn Pmove(
    pmove: *mut pmove_t,
    bg: &mut BgState,
    traps: &dyn BgTraps,
    callbacks: &mut dyn GameCallbacks,
) {
    unsafe {
        let finalTime = (*pmove).cmd.serverTime;

        if finalTime < (*(*pmove).ps).commandTime {
            return; // should not happen
        }

        if finalTime > (*(*pmove).ps).commandTime + 1000 {
            (*(*pmove).ps).commandTime = finalTime - 1000;
        }

        if (*(*pmove).ps).fallingToDeath != 0 {
            (*pmove).cmd.forwardmove = 0;
            (*pmove).cmd.rightmove = 0;
            (*pmove).cmd.upmove = 0;
            (*pmove).cmd.buttons = 0;
        }

        (*(*pmove).ps).pmove_framecount =
            ((*(*pmove).ps).pmove_framecount + 1) & ((1 << PS_PMOVEFRAMECOUNTBITS) - 1);

        // One working-set context for the whole (possibly multi-step) move.
        let mut pmc = PmoveContext::new(bg, traps, callbacks);

        // chop the move up if it is too long, to prevent framerate-dependent behavior
        while (*(*pmove).ps).commandTime != finalTime {
            let mut msec = finalTime - (*(*pmove).ps).commandTime;

            if (*pmove).pmove_fixed != 0 {
                if msec > (*pmove).pmove_msec {
                    msec = (*pmove).pmove_msec;
                }
            } else if msec > 66 {
                msec = 66;
            }
            (*pmove).cmd.serverTime = (*(*pmove).ps).commandTime + msec;

            pmc.PmoveSingle(pmove);

            if (*(*pmove).ps).pm_flags & PMF_JUMP_HELD != 0 {
                (*pmove).cmd.upmove = 20;
            }
        }
    }
}
