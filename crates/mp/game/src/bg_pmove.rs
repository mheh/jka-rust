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
use crate::g_strap::{
    strap_G2API_AnimateG2Models, strap_G2API_GetBoltMatrix, strap_G2API_GetBoneAnim,
    strap_G2API_IKMove, strap_G2API_SetBoneAnim, strap_G2API_SetBoneIKState,
};
use crate::q_math::{AngleMod, AngleSubtract};
use crate::q_math::{AngleVectors, Q_fabs, vectoangles};
use crate::q_math::{PITCH, ROLL, YAW};
use crate::q_math::{
    AngleNormalize180, AngleNormalize360, AnglesSubtract, AnglesToAxis, VectorClear, VectorCompare,
    VectorNormalize, VectorSet, _VectorAdd, _VectorCopy, _VectorMA, _VectorScale, _VectorSubtract,
};
use crate::bg_panimate::{
    BG_InRoll, BG_SaberInAttack, BG_SaberInSpecial, BG_SaberLockBreakAnim, BG_SpinningSaberAnim,
    PM_CanRollFromSoulCal, PM_SaberInTransition,
};
use crate::bg_saber::BG_MySaber;
use mp_bg::public::anim_number::animNumber_t;
use mp_bg::vehicles::MIN_LANDING_SLOPE;
use mp_qshared::shared::error_parm::errorParm_t::ERR_DROP;
use mp_qshared::shared::shared_eik_move_state::sharedEIKMoveState::{IKS_DYNAMIC, IKS_NONE};

// Pass-3 bg state channel (fork rulings 12-16): the per-call working set + the
// two seam traits + the session state. `PmoveContext` replaces the file-static
// pmove working set the skeletons parked on.
use crate::bg_channel::{BgState, BgTraps, GameCallbacks, PmoveContext};
// Vehicle-type discriminants for the `PM_Friction` vehicle-friction branch.
use mp_bg::vehicles::vehicle_type_t::vehicleType_t;

// --- `bg_pmove.c` file-scope movement parameters (globals 41-55). These are
// read-only tunables, so they stay module `const`s (post-mega-pass ruling 15:
// "the pm_* float constants can stay consts").
// Source: `oracle/oracle/codemp/game/bg_pmove.c:41-55`
pub const pm_stopspeed: f32 = 100.0;
pub const pm_friction: f32 = 6.0;
pub const pm_waterfriction: f32 = 1.0;
pub const pm_spectatorfriction: f32 = 5.0;

// --- `bg_pmove.c` local `FLY_*` enum (bg_pmove.c:441-444). Mirrors `pm_flying`.
// Source: `oracle/oracle/codemp/game/bg_pmove.c:441-444`
pub const FLY_NONE: c_int = 0;
pub const FLY_NORMAL: c_int = 1;
pub const FLY_VEHICLE: c_int = 2;
pub const FLY_HOVER: c_int = 3;

// --- Constants the pmove slice reads that are not (yet) centrally exported;
// defined here per the codebase's per-file `#define` convention (cf. w_force.rs
// defining its own `PMF_STUCK_TO_WALL`). Each cites its Raven `#define`.
/// `MINS_Z`. Source: `oracle/oracle/codemp/game/bg_public.h:46`
pub const MINS_Z: c_int = -24;
/// `MIN_WALK_NORMAL`. Source: `oracle/oracle/codemp/game/bg_local.h:5`
pub const MIN_WALK_NORMAL: f32 = 0.7;
/// `SURF_SLICK`. Source: `oracle/oracle/codemp/game/surfaceflags.h:39`
const SURF_SLICK: c_int = 0x0000_4000;
/// `CONTENTS_LAVA|WATER|SLIME`. Source: `oracle/oracle/codemp/game/surfaceflags.h:11,12,30`
const MASK_WATER: c_int = 0x0000_0002 | 0x0000_0004 | 0x0002_0000;
/// `PMF_STUCK_TO_WALL`. Source: `oracle/oracle/codemp/game/bg_public.h:417`
const PMF_STUCK_TO_WALL: c_int = 16384;
/// `PMF_TIME_KNOCKBACK`. Source: `oracle/oracle/codemp/game/bg_public.h:409`
const PMF_TIME_KNOCKBACK: c_int = 64;
/// `PMF_JUMP_HELD`. Source: `oracle/oracle/codemp/game/bg_public.h:404`
const PMF_JUMP_HELD: c_int = 2;
/// `PS_PMOVEFRAMECOUNTBITS`. Source: `oracle/oracle/codemp/game/q_shared.h:2141`
pub const PS_PMOVEFRAMECOUNTBITS: c_int = 6;
/// `BUTTON_ATTACK`. Source: `oracle/oracle/codemp/game/q_shared.h:2451`
const BUTTON_ATTACK: c_int = 1;
/// `BUTTON_ALT_ATTACK`. Source: `oracle/oracle/codemp/game/q_shared.h:2462`
const BUTTON_ALT_ATTACK: c_int = 128;

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

// The pmove pipeline as `PmoveContext` methods (pass-3 rulings 12/8a). Each was
// a no-arg C function reaching the file-static working set; the set now lives in
// `self` (`self.pm`/`self.pml`/`self.pm_entSelf`/… + `self.bg`/`self.traps`).
// The `unsafe` that dereferences the faithful `pm`/entity pointers is confined
// to these bodies (porting-rules §D11; ruling 14).
impl PmoveContext<'_> {
    /// Raven `PM_BGEntForNum` — the faithful `baseEnt`/`entSize` head-overlay
    /// (ruling 14). Returns the `bgEntity_t` at index `num` in the base array
    /// the engine handed us. Raven's `assert`s become defensive null/zero
    /// returns (out-of-pmove calls / unset base are the UB cases §19 covers).
    /// Source: `oracle/oracle/codemp/game/bg_pmove.c:172-199`
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
    /// Source: `oracle/oracle/codemp/game/bg_pmove.c:954-988`
    pub fn PM_ClipVelocity(
        &self,
        r#in: vec3_t,
        normal: vec3_t,
        out: &mut vec3_t,
        overbounce: f32,
    ) {
        unsafe {
            let ps = &*(*self.pm).ps;
            if ps.pm_flags & PMF_STUCK_TO_WALL != 0 {
                // no sliding!
                *out = r#in; // VectorCopy( in, out )
                return;
            }
            let oldInZ = r#in[2];

            // backoff = DotProduct (in, normal);
            let mut backoff = r#in[0] * normal[0] + r#in[1] * normal[1] + r#in[2] * normal[2];

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
    /// Source: `oracle/oracle/codemp/game/bg_pmove.c:998-1123`
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
            let speed = (vec[0] * vec[0] + vec[1] * vec[1] + vec[2] * vec[2]).sqrt();
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
            if (*ps).clientNum >= MAX_CLIENTS as c_int{
                pEnt = self.pm_entSelf;
            }

            // apply ground friction, even if on ladder
            if self.pm_flying != FLY_VEHICLE
                && !pEnt.is_null()
                && (*pEnt).s.NPC_class == CLASS_VEHICLE as c_int
                && !(*pEnt).m_pVehicle.is_null()
                && (*(*(*pEnt).m_pVehicle).m_pVehicleInfo).r#type as c_int
                    != vehicleType_t::VH_ANIMAL as c_int
                && (*(*(*pEnt).m_pVehicle).m_pVehicleInfo).r#type as c_int
                    != vehicleType_t::VH_WALKER as c_int
                && (*(*(*pEnt).m_pVehicle).m_pVehicleInfo).friction != 0.0
            {
                let friction = (*(*(*pEnt).m_pVehicle).m_pVehicleInfo).friction;
                if (*ps).pm_flags & PMF_TIME_KNOCKBACK == 0 {
                    let control = if speed < pm_stopspeed { pm_stopspeed } else { speed };
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
                    let control = if speed < pm_stopspeed { pm_stopspeed } else { speed };
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
            else if (*ps).groundEntityNum < MAX_CLIENTS as c_int{
                drop = 0.0;
            }

            if (*ps).pm_type == PM_SPECTATOR as c_int || (*ps).pm_type == PM_FLOAT as c_int {
                if (*ps).pm_type == PM_FLOAT as c_int {
                    // almost no friction while floating (Raven's `0.1` is a
                    // `double` literal; compute in f64 to preserve parity).
                    drop = (drop as f64
                        + speed as f64 * 0.1 * self.pml.frametime as f64)
                        as f32;
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
    /// `BgTraps::pointcontents` seam (ruling 13).
    /// Source: `oracle/oracle/codemp/game/bg_pmove.c:4285-4320`
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

    /// Raven `PM_DoSlowFall` — reads `pm->ps`; ported in the pass-3 remainder.
    //TODO: Port PM_DoSlowFall
    // Source: oracle/oracle/codemp/game/bg_pmove.c:321-329
    pub fn PM_DoSlowFall(&mut self) -> qboolean {
        todo!("Port PM_DoSlowFall — pass 3 (bg_pmove.c:321-329)")
    }

    /// Raven `PmoveSingle` — one fixed-timestep move. The opening (proxy button
    /// fix-up, working-set entity setup, slow-fall latch, result clear, rocket-
    /// trooper crouch clamp) is ported faithfully; the ~930-line move pipeline
    /// remainder is the pass-3 target (single `todo!` per task).
    /// Source: `oracle/oracle/codemp/game/bg_pmove.c:10174-11157`
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
                if (*(*pm).ps).clientNum < MAX_CLIENTS as c_int{
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

            //TODO: Port PmoveSingle remainder
            // Source: oracle/oracle/codemp/game/bg_pmove.c:10228-11157
            todo!("Port PmoveSingle remainder — pass 3")
        }
    }
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
// PORT-ESCALATION(missing-global-table): returns `forceJumpStrength[FORCE_LEVEL_3]/2.5f`. The
// backfill supplies only `JUMP_VELOCITY` (element 0); the full `forceJumpStrength` file-scope
// table (a bg_pmove.c global, also parked in NPC_AI_Jedi.rs) is unresolved — no invention allowed.
pub fn BG_ForceWallJumpStrength() -> f32 {
    todo!("Port BG_ForceWallJumpStrength — parked: missing-global-table")
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

// `PM_SetWaterLevel` is a `PmoveContext` method above (it reads the pmove
// working set and drives `BgTraps::pointcontents`).

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
pub fn PM_WalkingAnim(
    anim: c_int,
) -> qboolean {
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
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4600-4620`
pub fn PM_RunningAnim(
    anim: c_int,
) -> qboolean {
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
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4622-4633`
pub fn PM_SwimmingAnim(
    anim: c_int,
) -> qboolean {
    use animNumber_t::*;
    const ANIMS: &[animNumber_t] = &[
        BOTH_SWIM_IDLE1, //# Swimming Idle 1
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
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4635-4647`
pub fn PM_RollingAnim(
    anim: c_int,
) -> qboolean {
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
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4649-4675`
// fork-9: `angles` is a written-through out-param → `&mut [f32;3]`; `slope` stays a
// read-only by-value input. Cross-file callers are updated by the fixer.
pub fn PM_AnglesForSlope(
    yaw: f32,
    slope: vec3_t,
    angles: &mut [f32; 3],
) {
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

    // mod = DotProduct( nvf, ovr )
    let mut r#mod = nvf[0] * ovr[0] + nvf[1] * ovr[1] + nvf[2] * ovr[2];
    if r#mod < 0.0 {
        r#mod = -1.0;
    } else {
        r#mod = 1.0;
    }

    // dot = DotProduct( nvf, ovf )
    let dot = nvf[0] * ovf[0] + nvf[1] * ovf[1] + nvf[2] * ovf[2];

    angles[YAW] = 0.0;
    angles[PITCH] = dot * pitch;
    angles[ROLL] = (1.0 - Q_fabs(dot)) * pitch * r#mod;
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
pub fn BG_InSlopeAnim(
    anim: c_int,
) -> qboolean {
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
        LEGS_S1_LUP1, LEGS_S1_LUP2, LEGS_S1_LUP3, LEGS_S1_LUP4, LEGS_S1_LUP5,
        LEGS_S1_RUP1, LEGS_S1_RUP2, LEGS_S1_RUP3, LEGS_S1_RUP4, LEGS_S1_RUP5,
        LEGS_S3_LUP1, LEGS_S3_LUP2, LEGS_S3_LUP3, LEGS_S3_LUP4, LEGS_S3_LUP5,
        LEGS_S3_RUP1, LEGS_S3_RUP2, LEGS_S3_RUP3, LEGS_S3_RUP4, LEGS_S3_RUP5,
        LEGS_S4_LUP1, LEGS_S4_LUP2, LEGS_S4_LUP3, LEGS_S4_LUP4, LEGS_S4_LUP5,
        LEGS_S4_RUP1, LEGS_S4_RUP2, LEGS_S4_RUP3, LEGS_S4_RUP4, LEGS_S4_RUP5,
        LEGS_S5_LUP1, LEGS_S5_LUP2, LEGS_S5_LUP3, LEGS_S5_LUP4, LEGS_S5_LUP5,
        LEGS_S5_RUP1, LEGS_S5_RUP2, LEGS_S5_RUP3, LEGS_S5_RUP4, LEGS_S5_RUP5,
    ];
    if ANIMS.iter().any(|&a| a as c_int == anim) {
        qtrue
    } else {
        qfalse
    }
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

/// Raven `PM_UpdateViewAngles` — circularly clamp view angles with deltas.
///
/// `VEH_CONTROL_SCHEME_4` is undefined, so the `#else` branch (the
/// `BG_UnrestrainedPitchRoll` fighter test whose body is dead code, otherwise
/// the ±16000 short pitch clamp) is the compiled one.
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:7813-7894`
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
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8031-8199`
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
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8201-8327`
pub fn BG_CmdForRoll(
    ps: *mut playerState_t,
    anim: c_int,
    pCmd: *mut usercmd_t,
) {
    use animNumber_t::*;
    use crate::bg_panimate::PM_AnimLength;
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
            } else if PM_AnimLength(0, (*ps).legsAnim) - (*ps).torsoTimer < 350 {
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
            } else if PM_AnimLength(0, (*ps).legsAnim) - (*ps).torsoTimer < 200 {
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
            } else if PM_AnimLength(0, (*ps).legsAnim) - (*ps).torsoTimer < 150 {
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
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8331-8510`
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
            } else if (*ps).weapon == WP_SABER as c_int && BG_SaberInAttack((*ps).saberMove) == qtrue
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

            let mut saber = BG_MySaber((*ps).clientNum, 0, self.bg);
            if !saber.is_null() && (*saber).moveSpeedScale != 1.0 {
                (*ps).speed *= (*saber).moveSpeedScale;
            }
            saber = BG_MySaber((*ps).clientNum, 1, self.bg);
            if !saber.is_null() && (*saber).moveSpeedScale != 1.0 {
                (*ps).speed *= (*saber).moveSpeedScale;
            }
        }
    }
}

/// Raven `BG_InRollAnim`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8512-8523`
pub fn BG_InRollAnim(
    cent: *mut entityState_t,
) -> qboolean {
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
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8525-8560`
pub fn BG_InKnockDown(
    anim: c_int,
) -> qboolean {
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
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8562-8574`
pub fn BG_InRollES(
    ps: *mut entityState_t,
    anim: c_int,
) -> qboolean {
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
/// is threaded via `bg: &BgState` (ruling 11/15).
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8576-8730`
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
            if strap_G2API_SetBoneIKState(
                ghoul2,
                time,
                core::ptr::null(),
                IKS_DYNAMIC as c_int,
                &mut ikP,
            ) == qfalse
            {
                debug_assert!(false, "Failed to init IK system for g2 instance!");
            }

            // Now create our IK bone state.
            if strap_G2API_SetBoneIKState(
                ghoul2,
                time,
                b"lhumerus\0".as_ptr() as *const c_char,
                IKS_DYNAMIC as c_int,
                &mut ikP,
            ) != qfalse
            {
                // restrict the elbow joint
                VectorSet(&mut ikP.pcj_mins, -90.0, -20.0, -20.0);
                VectorSet(&mut ikP.pcj_maxs, 30.0, 20.0, -20.0);

                if strap_G2API_SetBoneIKState(
                    ghoul2,
                    time,
                    b"lradius\0".as_ptr() as *const c_char,
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

            strap_G2API_GetBoltMatrix(
                ghoul2,
                0,
                lHandBolt,
                &mut lHandMatrix,
                tAngles,
                origin,
                time,
                core::ptr::null_mut(),
                scale,
            );
            // Get the point position from the matrix.
            lHand[0] = lHandMatrix.matrix[0][3];
            lHand[1] = lHandMatrix.matrix[1][3];
            lHand[2] = lHandMatrix.matrix[2][3];

            _VectorSubtract(lHand, desiredPos, &mut torg);
            distToDest = (torg[0] * torg[0] + torg[1] * torg[1] + torg[2] * torg[2]).sqrt();

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
            if strap_G2API_IKMove(ghoul2, time, &mut ikM) != qfalse {
                // now do the standard model animate stuff with ragdoll update params.
                _VectorCopy(angles, &mut tuParms.angles);
                tuParms.angles[PITCH] = 0.0;

                _VectorCopy(origin, &mut tuParms.position);
                _VectorCopy(scale, &mut tuParms.scale);

                tuParms.me = (*ent).number;
                VectorClear(&mut tuParms.velocity);

                strap_G2API_AnimateG2Models(ghoul2, time, &mut tuParms);
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

            strap_G2API_SetBoneIKState(
                ghoul2,
                time,
                b"lhumerus\0".as_ptr() as *const c_char,
                IKS_NONE as c_int,
                core::ptr::null_mut(),
            );
            strap_G2API_SetBoneIKState(
                ghoul2,
                time,
                b"lradius\0".as_ptr() as *const c_char,
                IKS_NONE as c_int,
                core::ptr::null_mut(),
            );

            // then reset the angles/anims on these PCJs
            strap_G2API_SetBoneAngles(
                ghoul2,
                0,
                b"lhumerus\0".as_ptr() as *const c_char,
                crate::q_math::vec3_origin,
                BONE_ANGLES_POSTMULT,
                POSITIVE_X as c_int,
                NEGATIVE_Y as c_int,
                NEGATIVE_Z as c_int,
                core::ptr::null_mut(),
                0,
                time,
            );
            strap_G2API_SetBoneAngles(
                ghoul2,
                0,
                b"lradius\0".as_ptr() as *const c_char,
                crate::q_math::vec3_origin,
                BONE_ANGLES_POSTMULT,
                POSITIVE_X as c_int,
                NEGATIVE_Y as c_int,
                NEGATIVE_Z as c_int,
                core::ptr::null_mut(),
                0,
                time,
            );

            // Match the left arm back up with the pelvis anim/frames again.
            strap_G2API_GetBoneAnim(
                ghoul2,
                b"pelvis\0".as_ptr() as *const c_char,
                time,
                &mut cFrame,
                &mut sFrame,
                &mut eFrame,
                &mut flags,
                &mut animSpeed,
                core::ptr::null_mut(),
                0,
            );
            strap_G2API_SetBoneAnim(
                ghoul2,
                0,
                b"lhumerus\0".as_ptr() as *const c_char,
                sFrame,
                eFrame,
                flags,
                animSpeed,
                time,
                sFrame as f32,
                300,
            );
            strap_G2API_SetBoneAnim(
                ghoul2,
                0,
                b"lradius\0".as_ptr() as *const c_char,
                sFrame,
                eFrame,
                flags,
                animSpeed,
                time,
                sFrame as f32,
                300,
            );

            // finally, get rid of all the ik state effector data (null bone name).
            strap_G2API_SetBoneIKState(
                ghoul2,
                time,
                core::ptr::null(),
                IKS_NONE as c_int,
                core::ptr::null_mut(),
            );

            *ikInProgress = qfalse;
        }
    }
}

/// Raven `BG_UpdateLookAngles`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8733-8787`
// fork-9: `lastHeadAngles`/`lookAngles` are written in place → `&mut vec3_t`
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
    // are single-call temporaries (ruling 5) → plain locals.
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
            lookAnglesDiff[ang] = crate::q_math::AngleNormalize180(lookAnglesDiff[ang]);
        }

        if crate::q_math::VectorLengthSquared(lookAnglesDiff) != 0.0 {
            lookAngles[PITCH] = crate::q_math::AngleNormalize180(
                oldLookAngles[PITCH] + (lookAnglesDiff[PITCH] * fFrameInter * lookSpeed),
            );
            lookAngles[YAW] = crate::q_math::AngleNormalize180(
                oldLookAngles[YAW] + (lookAnglesDiff[YAW] * fFrameInter * lookSpeed),
            );
            lookAngles[ROLL] = crate::q_math::AngleNormalize180(
                oldLookAngles[ROLL] + (lookAnglesDiff[ROLL] * fFrameInter * lookSpeed),
            );
        }
    }
    //Remember current lookAngles next time
    *lastHeadAngles = *lookAngles;
}

/// Raven `BG_G2ClientNeckAngles`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8790-8866`
// fork-9: `headAngles`/`neckAngles`/`thoracicAngles` are written in place → `&mut vec3_t`;
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
    if thoracicAngles[PITCH] != 0.0 {
        //already been set above, blend them
        thoracicAngles[PITCH] = (thoracicAngles[PITCH] + (lA[PITCH] * 0.4)) * 0.5;
    } else {
        thoracicAngles[PITCH] = lA[PITCH] * 0.4;
    }
    if thoracicAngles[YAW] != 0.0 {
        thoracicAngles[YAW] = (thoracicAngles[YAW] + (lA[YAW] * 0.1)) * 0.5;
    } else {
        thoracicAngles[YAW] = lA[YAW] * 0.1;
    }
    if thoracicAngles[ROLL] != 0.0 {
        thoracicAngles[ROLL] = (thoracicAngles[ROLL] + (lA[ROLL] * 0.1)) * 0.5;
    } else {
        thoracicAngles[ROLL] = lA[ROLL] * 0.1;
    }

    neckAngles[PITCH] = lA[PITCH] * 0.2;
    neckAngles[YAW] = lA[YAW] * 0.3;
    neckAngles[ROLL] = lA[ROLL] * 0.3;

    headAngles[PITCH] = lA[PITCH] * 0.4;
    headAngles[YAW] = lA[YAW] * 0.6;
    headAngles[ROLL] = lA[ROLL] * 0.6;

    unsafe {
        strap_G2API_SetBoneAngles(
            ghoul2,
            0,
            b"cranium\0".as_ptr() as *const c_char,
            *headAngles,
            BONE_ANGLES_POSTMULT,
            POSITIVE_X as c_int,
            NEGATIVE_Y as c_int,
            NEGATIVE_Z as c_int,
            core::ptr::null_mut(),
            0,
            time,
        );
        strap_G2API_SetBoneAngles(
            ghoul2,
            0,
            b"cervical\0".as_ptr() as *const c_char,
            *neckAngles,
            BONE_ANGLES_POSTMULT,
            POSITIVE_X as c_int,
            NEGATIVE_Y as c_int,
            NEGATIVE_Z as c_int,
            core::ptr::null_mut(),
            0,
            time,
        );
        strap_G2API_SetBoneAngles(
            ghoul2,
            0,
            b"thoracic\0".as_ptr() as *const c_char,
            *thoracicAngles,
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
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8869-8990`
// fork-9: `viewAngles`/`thoracicAngles`/`ulAngles`/`llAngles` are written in place → `&mut vec3_t`;
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
) {
    use crate::bg_panimate::{
        BG_FlippingAnim, BG_InDeathAnim, BG_InSpecialJump, BG_SaberInSpecial,
        BG_SaberInSpecialAttack, BG_SpinningSaberAnim,
    };
    use crate::g_strap::strap_G2API_GetBoltMatrix_NoRecNoRot;
    unsafe {
        let mut doCorr = qfalse;

        //*tPitchAngle = viewAngles[PITCH];
        viewAngles[YAW] = crate::q_math::AngleDelta(cent_lerpAngles[YAW], angles[YAW]);
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
            let mut boltMatrix = mdxaBone_t { matrix: [[0.0; 4]; 3] };
            let mut motionFwd: vec3_t = [0.0; 3];
            let mut motionAngles: vec3_t = [0.0; 3];
            let mut motionRt: vec3_t = [0.0; 3];
            let mut tempAng: vec3_t = [0.0; 3];

            strap_G2API_GetBoltMatrix_NoRecNoRot(
                ghoul2,
                0,
                motionBolt,
                &mut boltMatrix,
                crate::q_math::vec3_origin,
                cent_lerpOrigin,
                time,
                core::ptr::null_mut(),
                modelScale,
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
                viewAngles[ang] = crate::q_math::AngleNormalize180(
                    viewAngles[ang] - crate::q_math::AngleNormalize180(motionAngles[ang]),
                );
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
pub fn BG_InRoll2(
    es: *mut entityState_t,
) -> qboolean {
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
/// `WeaponReadyAnim` is the bg const weapon-ready-anim table (ruling 12). Raven's
/// function-scope `static` scratch is single-call temporaries (ruling 5) → plain
/// locals. `VEH_CONTROL_SCHEME_4`/`BONE_BASED_LEG_ANGLES` are undefined.
/// fork-9: `legsAngles`/`turAngles` are written out-params (`&mut`); `legs` is
/// the axis matrix out (`*mut vec3_t`).
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9082-9457`
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
    lastHeadAngles: vec3_t,
    lookTime: c_int,
    emplaced: *mut entityState_t,
    crazySmoothFactor: *mut c_int,
) {
    // by-value angle params the body mutates in place (LAW keeps them by value —
    // the caller does not read the write-back).
    let mut lookAngles: vec3_t = lookAngles;
    let mut lastHeadAngles: vec3_t = lastHeadAngles;

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
                    b"lower_lumbar\0".as_ptr() as *const c_char,
                    b"upper_lumbar\0".as_ptr() as *const c_char,
                    b"cranium\0".as_ptr() as *const c_char,
                    b"thoracic\0".as_ptr() as *const c_char,
                    b"cervical\0".as_ptr() as *const c_char,
                ] {
                    strap_G2API_SetBoneAngles(
                        ghoul2,
                        0,
                        bone,
                        crate::q_math::vec3_origin,
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
                let e = cstr(&format!("Bad player movement angle ({})", dir));
                crate::g_main::Com_Error(ERR_DROP as c_int, e.as_ptr());
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

        if VectorCompare(velAng, crate::q_math::vec3_origin) == qfalse {
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
            BG_SwingAngles(legsAngles[YAW], 0.0, 90.0, 0.65, lYawAngle, lYawing, frametime);
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
                    );
                    strap_G2API_SetBoneAngles(
                        ghoul2,
                        0,
                        b"lower_lumbar\0".as_ptr() as *const c_char,
                        llAngles,
                        BONE_ANGLES_POSTMULT,
                        POSITIVE_X as c_int,
                        NEGATIVE_Y as c_int,
                        NEGATIVE_Z as c_int,
                        core::ptr::null_mut(),
                        0,
                        time,
                    );
                    strap_G2API_SetBoneAngles(
                        ghoul2,
                        0,
                        b"upper_lumbar\0".as_ptr() as *const c_char,
                        ulAngles,
                        BONE_ANGLES_POSTMULT,
                        POSITIVE_X as c_int,
                        NEGATIVE_Y as c_int,
                        NEGATIVE_Z as c_int,
                        core::ptr::null_mut(),
                        0,
                        time,
                    );
                    strap_G2API_SetBoneAngles(
                        ghoul2,
                        0,
                        b"cranium\0".as_ptr() as *const c_char,
                        crate::q_math::vec3_origin,
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
                    strap_G2API_SetBoneAngles(
                        ghoul2,
                        0,
                        b"cranium\0".as_ptr() as *const c_char,
                        crate::q_math::vec3_origin,
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
                strap_G2API_SetBoneAngles(
                    ghoul2,
                    0,
                    b"lower_lumbar\0".as_ptr() as *const c_char,
                    crate::q_math::vec3_origin,
                    BONE_ANGLES_POSTMULT,
                    POSITIVE_X as c_int,
                    NEGATIVE_Y as c_int,
                    NEGATIVE_Z as c_int,
                    core::ptr::null_mut(),
                    0,
                    time,
                );
                _VectorScale(facingAngles, 0.8, &mut facingAngles);
                strap_G2API_SetBoneAngles(
                    ghoul2,
                    0,
                    b"upper_lumbar\0".as_ptr() as *const c_char,
                    facingAngles,
                    BONE_ANGLES_POSTMULT,
                    POSITIVE_X as c_int,
                    NEGATIVE_Y as c_int,
                    NEGATIVE_Z as c_int,
                    core::ptr::null_mut(),
                    0,
                    time,
                );
                _VectorScale(facingAngles, 0.8, &mut facingAngles);
                strap_G2API_SetBoneAngles(
                    ghoul2,
                    0,
                    b"thoracic\0".as_ptr() as *const c_char,
                    facingAngles,
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
                strap_G2API_SetBoneAngles(
                    ghoul2,
                    0,
                    b"cervical\0".as_ptr() as *const c_char,
                    facingAngles,
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
        );

        _VectorCopy(cent_lerpAngles, &mut eyeAngles);

        for i in 0..3usize {
            lookAngles[i] = AngleNormalize180(lookAngles[i]);
            eyeAngles[i] = AngleNormalize180(eyeAngles[i]);
        }
        AnglesSubtract(lookAngles, eyeAngles, &mut lookAngles);

        BG_UpdateLookAngles(
            lookTime,
            &mut lastHeadAngles,
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

        BG_G2ClientNeckAngles(
            ghoul2,
            time,
            lookAngles,
            &mut headAngles,
            &mut neckAngles,
            &mut thoracicAngles,
            headClampMinAngles,
            headClampMaxAngles,
        );

        strap_G2API_SetBoneAngles(
            ghoul2,
            0,
            b"lower_lumbar\0".as_ptr() as *const c_char,
            llAngles,
            BONE_ANGLES_POSTMULT,
            POSITIVE_X as c_int,
            NEGATIVE_Y as c_int,
            NEGATIVE_Z as c_int,
            core::ptr::null_mut(),
            0,
            time,
        );
        strap_G2API_SetBoneAngles(
            ghoul2,
            0,
            b"upper_lumbar\0".as_ptr() as *const c_char,
            ulAngles,
            BONE_ANGLES_POSTMULT,
            POSITIVE_X as c_int,
            NEGATIVE_Y as c_int,
            NEGATIVE_Z as c_int,
            core::ptr::null_mut(),
            0,
            time,
        );
        strap_G2API_SetBoneAngles(
            ghoul2,
            0,
            b"thoracic\0".as_ptr() as *const c_char,
            thoracicAngles,
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

/// Raven `PM_CmdForSaberMoves` — force movement/jump commands for the special
/// dual/staff jump/spin saber attacks.
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9474-9639`
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

/// Raven `PM_GetOkWeaponForVehicle` — first weapon the client owns that is
/// usable on a vehicle, or -1.
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9762-9780`
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
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9783-9830`
impl PmoveContext<'_> {
    pub fn PM_VehForcedTurning(&mut self, veh: *mut bgEntity_t) {
        unsafe {
            let dst = self.PM_BGEntForNum((*(*veh).playerState).vehTurnaroundIndex);
            let mut pitchD: f32;
            let mut yawD: f32;
            let mut dir: vec3_t = [0.0; 3];

            if veh.is_null() || (*veh).m_pVehicle.is_null() {
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
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9916-9989`
impl PmoveContext<'_> {
    pub fn PM_VehFaceHyperspacePoint(&mut self, veh: *mut bgEntity_t) {
        unsafe {
            if veh.is_null() || (*veh).m_pVehicle.is_null() {
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
                        (*(*self.pm).ps).viewangles[i] =
                            (*(*veh).playerState).hyperSpaceAngles[i];
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

/// Raven `BG_VehicleAdjustBBoxForOrientation` — resize a fighter/flier vehicle's
/// bbox to its oriented extents, tracing to confirm the new box is valid.
///
/// `localTrace` is Raven's `void (*)(trace_t*, const vec3_t start, mins, maxs,
/// end, int, int)` callback; the resolved signature keeps it as `*mut c_void`
/// (ruling 11 fn-ptr param unsettled) so we transmute at the call site.
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9993-10076`
pub fn BG_VehicleAdjustBBoxForOrientation(
    veh: *mut Vehicle_t,
    origin: vec3_t,
    mins: &mut vec3_t,
    maxs: &mut vec3_t,
    clientNum: c_int,
    tracemask: c_int,
    localTrace: *mut c_void,
) {
    /// `DEFAULT_MINS_2`. Source: `oracle/oracle/codemp/game/bg_public.h`
    const DEFAULT_MINS_2: f32 = -24.0;
    // PORT-NOTE(fn-pointer-param): `localTrace` arrives as `*mut c_void` (LAW
    // signature); transmute to Raven's callback type to invoke it.
    type LocalTraceFn = unsafe extern "C" fn(
        *mut trace_t,
        *const vec3_t,
        *const vec3_t,
        *const vec3_t,
        *const vec3_t,
        c_int,
        c_int,
    );

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
            VectorSet(
                mins,
                (*vi).width / -2.0,
                (*vi).width / -2.0,
                DEFAULT_MINS_2,
            );
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
            if !localTrace.is_null() {
                let f: LocalTraceFn = core::mem::transmute(localTrace);
                f(
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

/// Raven `PM_MoveForKata` — force movement/jump commands during the soulcal and
/// medium/strong kata special attacks.
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:10092-10172`
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
/// per call (ruling 12) from the bg channel handles the game tier supplies,
/// then chops the move into fixed timesteps and runs `PmoveSingle` for each.
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:11167-11215`
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
